use std::{
    collections::BTreeMap,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result, bail};

use crate::core::tool;

use super::ast::{ArgvKind, ArgvSpec, Item, Predicate, Program, Statement, ToolInvocation, Value};
use super::json_path::json_path;
use super::parse::parse_seal;

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
                let value = self.value(value);
                self.vars.insert(name.clone(), value);
            }
            Statement::ArgvParse { specs } => self.parse_argv(specs)?,
            Statement::ExecChecked { argv } => {
                let code = self.run_external(argv, CaptureMode::None)?.code;
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
            Statement::CaptureOptional { name, status, argv } => {
                let output = self.run_external(argv, CaptureMode::Combined)?;
                self.vars
                    .insert(name.clone(), output.stdout.trim().to_string());
                self.vars.insert(status.clone(), output.code.to_string());
            }
            Statement::ToolExec { invocation } => {
                self.tool_output(invocation)?;
            }
            Statement::ToolPassthrough { start, invocation } => {
                if let Some(output) = self.tool_passthrough_output(*start, invocation)? {
                    println!("{output}");
                }
            }
            Statement::ToolCapture { name, invocation } => {
                let output = self.tool_output(invocation)?;
                self.vars.insert(name.clone(), output);
            }
            Statement::StringTrim { name, value } => {
                let output = self.tool_path(&["string", "trim"], std::slice::from_ref(value))?;
                self.vars.insert(name.clone(), output);
            }
            Statement::JsonGet { name, json, path } => {
                let output = self.tool_path(
                    &["json", "get"],
                    &[
                        json.clone(),
                        Value::Literal {
                            text: json_path(path),
                        },
                    ],
                )?;
                self.vars.insert(name.clone(), output);
            }
            Statement::RegexCapture {
                name,
                value,
                pattern,
                group,
            } => {
                let output = self.tool_path(
                    &["regex", "capture"],
                    &[
                        value.clone(),
                        Value::Literal {
                            text: pattern.clone(),
                        },
                        Value::Literal {
                            text: group.to_string(),
                        },
                    ],
                )?;
                self.vars.insert(name.clone(), output);
            }
            Statement::IntAdd { name, left, right } => {
                let output = self.tool_path(&["int", "add"], &[left.clone(), right.clone()])?;
                self.vars.insert(name.clone(), output);
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
                let value = self.value(value);
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
                let old_args = self.set_function_args(argv);
                let flow = self.run_function(name)?;
                self.restore_function_args(old_args);
                if !matches!(flow, Flow::Continue) {
                    return Ok(flow);
                }
            }
            Statement::Print { value } => println!("{}", self.value(value)),
            Statement::Error { value } => eprintln!("{}", self.value(value)),
            Statement::Fail { value } => {
                eprintln!("{}", self.value(value));
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
                bail!("seal argv parse failed");
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
                            bail!("seal argv parse failed");
                        };
                        self.vars.insert(spec.name.clone(), value.clone());
                        index += 2;
                    }
                }
            }
        }
        Ok(())
    }

    fn value(&self, value: &Value) -> String {
        match value {
            Value::Literal { text } => text.clone(),
            Value::Var { name } => self.vars.get(name).cloned().unwrap_or_default(),
            Value::Env { name } => self.env.get(name).cloned().unwrap_or_default(),
            Value::EnvDefault { name, default } => self
                .env
                .get(name)
                .filter(|value| !value.is_empty())
                .cloned()
                .unwrap_or_else(|| default.clone()),
            Value::Concat { parts } => parts.iter().map(|part| self.value(part)).collect(),
        }
    }

    fn predicate(&self, predicate: &Predicate) -> Result<bool> {
        Ok(match predicate {
            Predicate::Empty { value } => self.value(value).is_empty(),
            Predicate::NotEmpty { value } => !self.value(value).is_empty(),
            Predicate::Eq { left, right } => self.value(left) == self.value(right),
            Predicate::Neq { left, right } => self.value(left) != self.value(right),
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
            Predicate::FileExists { path } => Path::new(&self.value(path)).is_file(),
            Predicate::DirExists { path } => Path::new(&self.value(path)).is_dir(),
            Predicate::ToolExists { name } => command_exists(name),
        })
    }

    fn int_value(&self, value: &Value) -> Result<i64> {
        let value = self.value(value);
        value
            .parse::<i64>()
            .with_context(|| format!("invalid integer: {value}"))
    }

    fn tool_output(&self, invocation: &ToolInvocation) -> Result<String> {
        let mut args = invocation.path.clone();
        args.extend(invocation.argv.iter().map(|value| self.value(value)));
        Ok(tool::eval(&args)?.unwrap_or_default())
    }

    fn tool_passthrough_output(
        &self,
        start: usize,
        invocation: &ToolInvocation,
    ) -> Result<Option<String>> {
        let argc = self
            .vars
            .get("0")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_default();
        let mut args = invocation.path.clone();
        args.extend(invocation.argv.iter().map(|value| self.value(value)));
        for index in start..=argc {
            if let Some(value) = self.vars.get(&index.to_string()) {
                args.push(value.clone());
            }
        }
        tool::eval(&args)
    }

    fn tool_path(&self, path: &[&str], argv: &[Value]) -> Result<String> {
        let invocation = ToolInvocation {
            path: path.iter().map(|part| part.to_string()).collect(),
            argv: argv.to_vec(),
        };
        self.tool_output(&invocation)
    }

    fn run_external(&self, argv: &[Value], capture: CaptureMode) -> Result<CommandOutput> {
        let argv = argv
            .iter()
            .map(|value| self.value(value))
            .collect::<Vec<_>>();
        let Some((program, args)) = argv.split_first() else {
            bail!("external command cannot be empty");
        };
        let mut command = Command::new(program);
        command.args(args).envs(&self.env);
        if matches!(capture, CaptureMode::None) {
            let status = command
                .status()
                .with_context(|| format!("failed to execute command: {program}"))?;
            return Ok(CommandOutput {
                code: status.code().unwrap_or(1),
                stdout: String::new(),
            });
        }
        command.stdout(Stdio::piped());
        if matches!(capture, CaptureMode::Combined) {
            command.stderr(Stdio::piped());
        }
        let output = command
            .output()
            .with_context(|| format!("failed to execute command: {program}"))?;
        let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if matches!(capture, CaptureMode::Combined) {
            stdout.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        Ok(CommandOutput {
            code: output.status.code().unwrap_or(1),
            stdout,
        })
    }

    fn set_function_args(&mut self, argv: &[Value]) -> Vec<Option<String>> {
        let values = argv
            .iter()
            .map(|value| self.value(value))
            .collect::<Vec<_>>();
        let len = values.len();
        let old = (0..=values.len())
            .map(|index| self.vars.remove(&index.to_string()))
            .collect::<Vec<_>>();
        for (index, value) in values.into_iter().enumerate() {
            self.vars.insert((index + 1).to_string(), value);
        }
        self.vars.insert("0".to_string(), len.to_string());
        old
    }

    fn restore_function_args(&mut self, old: Vec<Option<String>>) {
        for (index, value) in old.into_iter().enumerate() {
            match value {
                Some(value) => {
                    self.vars.insert(index.to_string(), value);
                }
                None => {
                    self.vars.remove(&index.to_string());
                }
            }
        }
    }
}

enum CaptureMode {
    None,
    Stdout,
    Combined,
}

struct CommandOutput {
    code: i32,
    stdout: String,
}

fn find_spec<'a>(specs: &'a [ArgvSpec], arg: &str) -> Option<&'a ArgvSpec> {
    specs.iter().find(|spec| {
        let option = option_name(&spec.name);
        arg == option || arg.starts_with(&(option + "="))
    })
}

fn option_name(name: &str) -> String {
    format!("--{}", name.replace('_', "-"))
}

fn case_matches(pattern: &str, value: &str) -> bool {
    pattern == "*" || pattern == value
}

fn command_exists(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

fn shell_words(argv: &[String]) -> String {
    argv.join("\u{1f}")
}

fn split_words(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split('\u{1f}').map(str::to_string).collect()
    }
}
