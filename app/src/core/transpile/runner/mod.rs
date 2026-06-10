use std::{collections::BTreeMap, path::Path, process::Command, time::Duration};

use anyhow::{Context, Result, bail};

use crate::core::tool;

use self::support::{
    CaptureMode, CommandOutput, case_matches, find_spec, option_name, shell_words, split_words,
    write_stream_file,
};
use super::ast::{
    ArgvKind, ArgvSpec, ExpansionOp, Item, Predicate, Program, Statement, Value, ValueSource,
};
use super::parse::parse_seal;

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
}

enum Flow {
    Continue,
    Break,
    Exit(i32),
}

struct ArgSnapshot {
    argv: Option<String>,
    values: Vec<Option<String>>,
}

enum SourceState {
    Unset,
    Empty,
    Present(String),
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
        Self { program, vars, env }
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
            Statement::ArgvParse { specs } => self.parse_argv(specs)?,
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
            Statement::Print { value } => println!("{}", self.try_value(value)?),
            Statement::Error { value } => eprintln!("{}", self.try_value(value)?),
            Statement::Fail { value } => {
                eprintln!("{}", self.try_value(value)?);
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

    fn parse_argv(&mut self, specs: &[ArgvSpec]) -> Result<()> {
        let argv = self
            .vars
            .get("__seal_argv")
            .map(|value| split_words(value))
            .unwrap_or_default();
        self.vars
            .insert("__seal_argc".to_string(), argv.len().to_string());
        self.vars
            .insert("__seal_help".to_string(), "false".to_string());
        for spec in specs {
            let value = match spec.kind {
                ArgvKind::String => spec.default.clone().unwrap_or_default(),
                ArgvKind::Flag => "false".to_string(),
            };
            self.vars.insert(spec.name.clone(), value);
        }
        let mut index = 0;
        while index < argv.len() {
            let arg = &argv[index];
            if arg == "--" {
                break;
            }
            if matches!(arg.as_str(), "-h" | "--help" | "help") {
                self.vars
                    .insert("__seal_help".to_string(), "true".to_string());
                index += 1;
                continue;
            }
            let Some(spec) = find_spec(specs, arg) else {
                eprintln!("unknown option: {arg}");
                bail!("argv parse failed");
            };
            match spec.kind {
                ArgvKind::Flag => {
                    self.vars.insert(spec.name.clone(), "true".to_string());
                    index += 1;
                }
                ArgvKind::String => {
                    let option = option_name(&spec.name);
                    if let Some(value) = arg.strip_prefix(&(option.clone() + "=")) {
                        self.vars.insert(spec.name.clone(), value.to_string());
                        index += 1;
                    } else {
                        let Some(value) = argv.get(index + 1) else {
                            eprintln!("missing value for {option}");
                            bail!("argv parse failed");
                        };
                        self.vars.insert(spec.name.clone(), value.clone());
                        index += 2;
                    }
                }
            }
        }
        Ok(())
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

    fn predicate(&self, predicate: &Predicate) -> Result<bool> {
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
            ValueSource::Env { name } => self.map_source_state(self.env.get(name).cloned()),
            ValueSource::Var { name } => self.map_source_state(self.vars.get(name).cloned()),
        }
    }

    fn map_source_state(&self, value: Option<String>) -> SourceState {
        match value {
            None => SourceState::Unset,
            Some(value) if value.is_empty() => SourceState::Empty,
            Some(value) => SourceState::Present(value),
        }
    }

    fn tool_path(&self, path: &[&str], argv: &[Value]) -> Result<String> {
        let mut args = path.iter().map(|part| part.to_string()).collect::<Vec<_>>();
        args.extend(self.expanded_values(argv)?);
        Ok(tool::eval(&args)?.unwrap_or_default())
    }

    fn run_external(&self, argv: &[Value], capture: CaptureMode) -> Result<CommandOutput> {
        self.run_external_with_env(argv, capture, &[])
    }

    fn run_external_with_env(
        &self,
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
        if matches!(capture, CaptureMode::None) {
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
        Ok(CommandOutput {
            code: output.status.code().unwrap_or(1),
            stdout,
            stderr,
        })
    }

    fn set_function_args(&mut self, argv: &[Value]) -> Result<ArgSnapshot> {
        let values = self.expanded_values(argv)?;
        let old_len = self.argc();
        let old = (0..=old_len)
            .map(|index| self.vars.remove(&index.to_string()))
            .collect::<Vec<_>>();
        let snapshot = ArgSnapshot {
            argv: self.vars.remove("__seal_argv"),
            values: old,
        };
        self.set_positional_args(&values);
        Ok(snapshot)
    }

    fn restore_function_args(&mut self, old: ArgSnapshot) {
        let current_len = self.argc();
        for index in 0..=current_len {
            self.vars.remove(&index.to_string());
        }
        for (index, value) in old.values.into_iter().enumerate() {
            match value {
                Some(value) => {
                    self.vars.insert(index.to_string(), value);
                }
                None => {
                    self.vars.remove(&index.to_string());
                }
            }
        }
        match old.argv {
            Some(value) => {
                self.vars.insert("__seal_argv".to_string(), value);
            }
            None => {
                self.vars.remove("__seal_argv");
            }
        }
    }

    fn expanded_values(&self, values: &[Value]) -> Result<Vec<String>> {
        let mut expanded = Vec::new();
        for value in values {
            match value {
                Value::Args => expanded.extend(self.current_args()),
                Value::Argc => expanded.push(self.argc().to_string()),
                _ => expanded.push(self.try_value(value)?),
            }
        }
        Ok(expanded)
    }

    fn argc(&self) -> usize {
        self.vars
            .get("0")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_default()
    }

    fn current_args(&self) -> Vec<String> {
        (1..=self.argc())
            .filter_map(|index| self.vars.get(&index.to_string()).cloned())
            .collect()
    }

    fn shift_args(&mut self, count: usize) {
        let args = self.current_args();
        let remaining = args.into_iter().skip(count).collect::<Vec<_>>();
        self.set_positional_args(&remaining);
    }

    fn set_positional_args(&mut self, values: &[String]) {
        let old_len = self.argc();
        for index in 0..=old_len {
            self.vars.remove(&index.to_string());
        }
        for (index, value) in values.iter().enumerate() {
            self.vars.insert((index + 1).to_string(), value.clone());
        }
        self.vars.insert("0".to_string(), values.len().to_string());
        self.vars
            .insert("__seal_argv".to_string(), shell_words(values));
    }
}
