use std::{collections::BTreeMap, path::Path, process::Command, time::Duration};

use anyhow::{Context, Result, bail};

use crate::core::tool;

use self::support::{
    ArgSnapshot, CaptureMode, CommandOutput, SourceState, case_matches, find_spec,
    map_source_state, option_name, shell_words, split_words, write_stderr, write_stderr_line,
    write_stdout, write_stdout_line, write_stream_file,
};
use super::ast::{
    ArgvKind, ArgvPositional, ArgvSpec, ExpansionOp, Item, Predicate, Program, Statement, Value,
    ValueSource,
};
use super::parse::parse_seal;

mod args;
mod support;

pub(crate) fn run_seal_file(
    path: &Path,
    argv: &[String],
    env_overlay: &[(String, String)],
) -> Result<i32> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let program = parse_seal(&source)?;
    let mut runner = Runner::new(&program, argv, env_overlay);
    runner.run_program()
}

struct Runner<'a> {
    program: &'a Program,
    vars: BTreeMap<String, String>,
    env: BTreeMap<String, String>,
    stdout_stack: Vec<String>,
}

enum Flow {
    Continue,
    Break,
    Exit(i32),
}

impl<'a> Runner<'a> {
    fn new(program: &'a Program, argv: &[String], env_overlay: &[(String, String)]) -> Self {
        let mut env = std::env::vars().collect::<BTreeMap<_, _>>();
        env.extend(env_overlay.iter().cloned());
        let mut vars = BTreeMap::new();
        vars.insert("__seal_argv".to_string(), shell_words(argv));
        vars.insert("0".to_string(), argv.len().to_string());
        for (index, value) in argv.iter().enumerate() {
            vars.insert((index + 1).to_string(), value.clone());
        }
        Self {
            program,
            vars,
            env,
            stdout_stack: Vec::new(),
        }
    }

    fn run_program(&mut self) -> Result<i32> {
        let statements = self
            .program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Statement { statement } => Some(statement),
                Item::Function { .. } => None,
            })
            .collect::<Vec<_>>();
        match self.run_statements(&statements)? {
            Flow::Continue | Flow::Break => Ok(0),
            Flow::Exit(code) => Ok(code),
        }
    }

    fn run_statements(&mut self, statements: &[&Statement]) -> Result<Flow> {
        for statement in statements {
            match self.run_statement(statement)? {
                Flow::Continue => {}
                flow => return Ok(flow),
            }
        }
        Ok(Flow::Continue)
    }

    fn run_body(&mut self, statements: &[Statement]) -> Result<Flow> {
        let refs = statements.iter().collect::<Vec<_>>();
        self.run_statements(&refs)
    }

    fn run_statement(&mut self, statement: &Statement) -> Result<Flow> {
        match statement {
            Statement::Assign { name, value } => {
                let value = self.try_value(value)?;
                self.vars.insert(name.clone(), value);
            }
            Statement::ArgvParse { specs, positional } => {
                self.parse_argv(specs, positional.as_ref())?
            }
            Statement::Shift { count } => self.shift_args(*count),
            Statement::ExecChecked { argv } => {
                let code = self.run_external(argv, CaptureMode::None)?.code;
                if code != 0 {
                    return Ok(Flow::Exit(code));
                }
            }
            Statement::ExecWrite {
                stream,
                path,
                append,
                argv,
            } => {
                let output = self.run_external(argv, CaptureMode::All)?;
                let path = self.try_value(path)?;
                write_stream_file(stream, Path::new(&path), *append, &output)?;
                if output.code != 0 {
                    return Ok(Flow::Exit(output.code));
                }
            }
            Statement::EnvExecChecked { env, argv } => {
                let overlay = env
                    .iter()
                    .map(|item| Ok((item.name.clone(), self.try_value(&item.value)?)))
                    .collect::<Result<Vec<_>>>()?;
                let code = self
                    .run_external_with_env(argv, CaptureMode::None, &overlay)?
                    .code;
                if code != 0 {
                    return Ok(Flow::Exit(code));
                }
            }
            Statement::CaptureChecked { name, argv } => {
                let output = self.run_external(argv, CaptureMode::Stdout)?;
                if output.code != 0 {
                    return Ok(Flow::Exit(output.code));
                }
                self.vars
                    .insert(name.clone(), output.stdout.trim().to_string());
            }
            Statement::CaptureFunction {
                name,
                function,
                argv,
            } => {
                let old_args = self.set_function_args(argv)?;
                self.stdout_stack.push(String::new());
                let flow = self.run_function(function)?;
                let captured = self
                    .stdout_stack
                    .pop()
                    .expect("stdout capture stack should be balanced");
                self.restore_function_args(old_args);
                if !matches!(flow, Flow::Continue) {
                    return Ok(flow);
                }
                self.vars.insert(name.clone(), captured.trim().to_string());
            }
            Statement::If {
                predicate,
                then_body,
                else_body,
            } => {
                let flow = if self.predicate(predicate)? {
                    self.run_body(then_body)?
                } else {
                    self.run_body(else_body)?
                };
                if !matches!(flow, Flow::Continue) {
                    return Ok(flow);
                }
            }
            Statement::While { predicate, body } => {
                while self.predicate(predicate)? {
                    match self.run_body(body)? {
                        Flow::Continue => {}
                        Flow::Break => break,
                        flow => return Ok(flow),
                    }
                }
            }
            Statement::Case { value, arms } => {
                let value = self.try_value(value)?;
                for arm in arms {
                    if arm
                        .patterns
                        .iter()
                        .any(|pattern| case_matches(pattern, &value))
                    {
                        let flow = self.run_body(&arm.body)?;
                        if !matches!(flow, Flow::Continue) {
                            return Ok(flow);
                        }
                        break;
                    }
                }
            }
            Statement::CallFunction { name, argv } => {
                let old_args = self.set_function_args(argv)?;
                let flow = self.run_function(name)?;
                self.restore_function_args(old_args);
                if !matches!(flow, Flow::Continue) {
                    return Ok(flow);
                }
            }
            Statement::Print { value } => {
                let text = self.try_value(value)?;
                write_stdout_line(&mut self.stdout_stack, &text)?
            }
            Statement::Error { value } => write_stderr_line(&self.try_value(value)?)?,
            Statement::Fail { value } => {
                write_stderr_line(&self.try_value(value)?)?;
                return Ok(Flow::Exit(1));
            }
            Statement::Exit { code } => return Ok(Flow::Exit(*code)),
            Statement::Break => return Ok(Flow::Break),
            Statement::Sleep { seconds } => std::thread::sleep(Duration::from_secs(*seconds)),
        }
        Ok(Flow::Continue)
    }

    fn run_function(&mut self, name: &str) -> Result<Flow> {
        let Some(body) = self.program.items.iter().find_map(|item| match item {
            Item::Function {
                name: function_name,
                body,
            } if function_name == name => Some(body),
            _ => None,
        }) else {
            bail!("unknown function: {name}");
        };
        self.run_body(body)
    }

    fn try_value(&self, value: &Value) -> Result<String> {
        Ok(match value {
            Value::Literal { text } => text.clone(),
            Value::Argc => self.argc().to_string(),
            Value::Args => shell_words(&self.current_args()),
            Value::Expand { source, op } => self.expand_value(source, op)?,
            Value::Concat { parts } => {
                let mut combined = String::new();
                for part in parts {
                    combined.push_str(&self.try_value(part)?);
                }
                combined
            }
        })
    }

    fn predicate(&mut self, predicate: &Predicate) -> Result<bool> {
        Ok(match predicate {
            Predicate::Command { argv } => self.run_external(argv, CaptureMode::None)?.code == 0,
            Predicate::Empty { value } => self.try_value(value)?.is_empty(),
            Predicate::NotEmpty { value } => !self.try_value(value)?.is_empty(),
            Predicate::Eq { left, right } => self.try_value(left)? == self.try_value(right)?,
            Predicate::Neq { left, right } => self.try_value(left)? != self.try_value(right)?,
            Predicate::IntLt { left, right } => self.int_value(left)? < self.int_value(right)?,
            Predicate::IntLte { left, right } => self.int_value(left)? <= self.int_value(right)?,
            Predicate::IntGt { left, right } => self.int_value(left)? > self.int_value(right)?,
            Predicate::IntGte { left, right } => self.int_value(left)? >= self.int_value(right)?,
            Predicate::JsonEmpty { value } => {
                self.tool_path(&["json", "empty"], std::slice::from_ref(value))? == "true"
            }
            Predicate::JsonNotEmpty { value } => {
                self.tool_path(&["json", "empty"], std::slice::from_ref(value))? == "false"
            }
            Predicate::FileExists { path } => Path::new(&self.try_value(path)?).is_file(),
            Predicate::DirExists { path } => Path::new(&self.try_value(path)?).is_dir(),
        })
    }

    fn int_value(&self, value: &Value) -> Result<i64> {
        let value = self.try_value(value)?;
        value
            .parse::<i64>()
            .with_context(|| format!("invalid integer: {value}"))
    }

    fn expand_value(&self, source: &ValueSource, op: &ExpansionOp) -> Result<String> {
        let state = self.source_state(source);
        match op {
            ExpansionOp::Plain => Ok(match state {
                SourceState::Unset => String::new(),
                SourceState::Empty => String::new(),
                SourceState::Present(value) => value,
            }),
            ExpansionOp::DefaultIfUnsetOrEmpty { fallback } => Ok(match state {
                SourceState::Present(value) => value,
                SourceState::Unset | SourceState::Empty => fallback.clone(),
            }),
            ExpansionOp::RequireNonEmpty { message } => match state {
                SourceState::Present(value) => Ok(value),
                SourceState::Unset | SourceState::Empty => bail!("{message}"),
            },
        }
    }

    fn source_state(&self, source: &ValueSource) -> SourceState {
        match source {
            ValueSource::Env { name } => map_source_state(self.env.get(name).cloned()),
            ValueSource::Var { name } => map_source_state(self.vars.get(name).cloned()),
        }
    }

    fn tool_path(&self, path: &[&str], argv: &[Value]) -> Result<String> {
        let mut args = path.iter().map(|part| part.to_string()).collect::<Vec<_>>();
        args.extend(self.expanded_values(argv)?);
        Ok(tool::eval(&args)?.unwrap_or_default())
    }

    fn run_external(&mut self, argv: &[Value], capture: CaptureMode) -> Result<CommandOutput> {
        self.run_external_with_env(argv, capture, &[])
    }

    fn run_external_with_env(
        &mut self,
        argv: &[Value],
        capture: CaptureMode,
        env_overlay: &[(String, String)],
    ) -> Result<CommandOutput> {
        let argv = self.expanded_values(argv)?;
        let Some((program, args)) = argv.split_first() else {
            bail!("external command cannot be empty");
        };
        let mut command = Command::new(program);
        command.args(args).envs(&self.env);
        command.envs(env_overlay.iter().map(|(key, value)| (key, value)));
        if matches!(capture, CaptureMode::None) && self.stdout_stack.is_empty() {
            let status = command
                .status()
                .with_context(|| format!("failed to execute command: {program}"))?;
            return Ok(CommandOutput {
                code: status.code().unwrap_or(1),
                stdout: String::new(),
                stderr: String::new(),
            });
        }
        let output = command
            .output()
            .with_context(|| format!("failed to execute command: {program}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        if matches!(capture, CaptureMode::None) {
            write_stdout(&mut self.stdout_stack, &stdout)?;
            write_stderr(&stderr)?;
        }
        Ok(CommandOutput {
            code: output.status.code().unwrap_or(1),
            stdout,
            stderr,
        })
    }
}
