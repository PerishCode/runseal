use anyhow::{Result, bail};

use super::ast::{CaseArm, EnvAssign, Item, Predicate, Program, Statement, Value};
use super::lower::lower_functions;
use super::parse_argv::parse_argv_block;
use super::parse_command::{parse_exec_write, validate_external_tokens, validate_shell_command};
use super::parse_lex::{
    assignment, is_safe_command_name, is_valid_name, split_test_words, split_words, strip_comment,
};
use super::value::parse_value_text;

#[derive(Debug, Clone)]
pub(super) struct SourceLine {
    pub(super) number: usize,
    pub(super) text: String,
}

pub(super) struct Parser {
    lines: Vec<SourceLine>,
    index: usize,
}
impl Parser {
    fn new(source: &str) -> Self {
        let lines = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                let text = strip_comment(line).trim().to_string();
                if text.is_empty() {
                    return None;
                }
                Some(SourceLine {
                    number: index + 1,
                    text,
                })
            })
            .collect();
        Self { lines, index: 0 }
    }

    fn parse_program(mut self) -> Result<Program> {
        let mut items = Vec::new();
        while self.peek().is_some() {
            if self
                .peek_text()
                .is_some_and(|text| function_header(text).is_some())
            {
                items.push(self.parse_function()?);
            } else {
                items.push(Item::Statement {
                    statement: self.parse_statement()?,
                });
            }
        }
        let program = Program { version: 1, items };
        Ok(lower_functions(program))
    }
    fn parse_function(&mut self) -> Result<Item> {
        let line = self.next().expect("function header should exist");
        let name = function_header(&line.text)
            .ok_or_else(|| anyhow::anyhow!("{}: expected function header", line.number))?
            .to_string();
        let body = self.parse_block(&["}"])?;
        self.expect_exact("}")?;
        Ok(Item::Function { name, body })
    }
    fn parse_block(&mut self, terminators: &[&str]) -> Result<Vec<Statement>> {
        let mut body = Vec::new();
        while let Some(line) = self.peek() {
            if terminators
                .iter()
                .any(|terminator| line.text == *terminator)
            {
                break;
            }
            body.push(self.parse_statement()?);
        }
        Ok(body)
    }
    fn parse_statement(&mut self) -> Result<Statement> {
        let Some(line) = self.peek().cloned() else {
            bail!("unexpected end of input");
        };
        if line.text == "__seal_argc=$#" {
            return parse_argv_block(self);
        }
        if line.text.starts_with("if ") {
            return self.parse_if();
        }
        if line.text.starts_with("while ") {
            return self.parse_while();
        }
        if line.text.starts_with("case ") {
            return self.parse_case();
        }
        self.next();
        parse_simple_statement(&line)
    }
    fn parse_if(&mut self) -> Result<Statement> {
        let line = self.next().expect("if line should exist");
        let inner = line
            .text
            .strip_prefix("if ")
            .and_then(|text| text.strip_suffix("; then"))
            .ok_or_else(|| anyhow::anyhow!("{}: expected `if <predicate>; then`", line.number))?;
        let predicate = parse_predicate(inner, line.number)?;
        let then_body = self.parse_block(&["else", "fi"])?;
        let else_body = if self.peek_text() == Some("else") {
            self.next();
            self.parse_block(&["fi"])?
        } else {
            Vec::new()
        };
        self.expect_exact("fi")?;
        Ok(Statement::If {
            predicate,
            then_body,
            else_body,
        })
    }
    fn parse_while(&mut self) -> Result<Statement> {
        let line = self.next().expect("while line should exist");
        let inner = line
            .text
            .strip_prefix("while ")
            .and_then(|text| text.strip_suffix("; do"))
            .ok_or_else(|| anyhow::anyhow!("{}: expected `while <predicate>; do`", line.number))?;
        let predicate = parse_predicate(inner, line.number)?;
        let body = self.parse_block(&["done"])?;
        self.expect_exact("done")?;
        Ok(Statement::While { predicate, body })
    }
    fn parse_case(&mut self) -> Result<Statement> {
        let line = self.next().expect("case line should exist");
        let value_text = line
            .text
            .strip_prefix("case ")
            .and_then(|text| text.strip_suffix(" in"))
            .ok_or_else(|| anyhow::anyhow!("{}: expected `case <value> in`", line.number))?;
        let value = parse_value_text(value_text, line.number)?;
        let mut arms = Vec::new();
        loop {
            let Some(line) = self.peek().cloned() else {
                bail!("{}: missing esac for case", line.number);
            };
            if line.text == "esac" {
                self.next();
                break;
            }
            let text = line.text.clone();
            let Some((patterns, remainder)) = text.split_once(')') else {
                bail!("{}: expected case arm pattern", line.number);
            };
            self.next();
            let patterns = patterns
                .split('|')
                .map(str::trim)
                .map(str::to_string)
                .collect::<Vec<_>>();
            if patterns.iter().any(|pattern| pattern.is_empty()) {
                bail!("{}: empty case pattern", line.number);
            }
            let mut body = Vec::new();
            let remainder = remainder.trim();
            if !remainder.is_empty() {
                let statement = remainder
                    .strip_suffix(";;")
                    .ok_or_else(|| {
                        anyhow::anyhow!("{}: inline case arms must end with `;;`", line.number)
                    })?
                    .trim();
                if !statement.is_empty() {
                    body.push(parse_simple_statement(&SourceLine {
                        number: line.number,
                        text: statement.to_string(),
                    })?);
                }
            } else {
                while let Some(next) = self.peek().cloned() {
                    if next.text == ";;" {
                        self.next();
                        break;
                    }
                    if next.text == "esac" {
                        bail!("{}: missing `;;` before esac", next.number);
                    }
                    body.push(self.parse_statement()?);
                }
            }
            arms.push(CaseArm { patterns, body });
        }
        Ok(Statement::Case { value, arms })
    }
    pub(super) fn expect_exact(&mut self, expected: &str) -> Result<()> {
        let Some(line) = self.next() else {
            bail!("expected `{expected}`, found end of input");
        };
        if line.text != expected {
            bail!(
                "{}: expected `{expected}`, got `{}`",
                line.number,
                line.text
            );
        }
        Ok(())
    }
    pub(super) fn peek(&self) -> Option<&SourceLine> {
        self.lines.get(self.index)
    }
    pub(super) fn peek_text(&self) -> Option<&str> {
        self.peek().map(|line| line.text.as_str())
    }
    pub(super) fn next(&mut self) -> Option<SourceLine> {
        let line = self.lines.get(self.index).cloned();
        self.index += usize::from(line.is_some());
        line
    }
}

fn function_header(text: &str) -> Option<&str> {
    let name = text.strip_suffix("() {")?;
    is_valid_name(name).then_some(name)
}

fn parse_simple_statement(line: &SourceLine) -> Result<Statement> {
    if let Some((name, value)) = assignment(&line.text)
        && let Some(argv) = capture_argv(value, line.number)?
    {
        return Ok(Statement::CaptureChecked {
            name: name.to_string(),
            argv,
        });
    }
    let tokens = split_words(&line.text, line.number)?;
    if let Some(statement) = parse_exec_write(&tokens, line.number)? {
        return Ok(statement);
    }
    if let Some(statement) = parse_env_exec(&tokens, line.number)? {
        return Ok(statement);
    }
    if let Some((name, value)) = assignment(&line.text) {
        return Ok(Statement::Assign {
            name: name.to_string(),
            value: parse_value_text(value, line.number)?,
        });
    }
    let Some((command, args)) = tokens.split_first() else {
        bail!("{}: expected statement", line.number);
    };
    validate_shell_command(command, args, line.number)?;
    match command.as_str() {
        "printf" => parse_printf(args, line.number),
        "eval" => bail!("{}: unsupported statement: eval", line.number),
        "seal" => bail!("{}: unsupported legacy seal helper statement", line.number),
        "shift" => {
            let count = match args {
                [] => 1,
                [count] => count
                    .parse::<usize>()
                    .map_err(|_| anyhow::anyhow!("{}: invalid shift count", line.number))?,
                _ => bail!("{}: shift accepts at most one argument", line.number),
            };
            Ok(Statement::Shift { count })
        }
        "print" => Ok(Statement::Print {
            value: one_value(args, line.number, "print")?,
        }),
        "error" => Ok(Statement::Error {
            value: one_value(args, line.number, "error")?,
        }),
        "fail" => Ok(Statement::Fail {
            value: one_value(args, line.number, "fail")?,
        }),
        "break" => {
            if !args.is_empty() {
                bail!("{}: break does not accept arguments", line.number);
            }
            Ok(Statement::Break)
        }
        "exit" => {
            if args.len() != 1 {
                bail!("{}: exit requires one code argument", line.number);
            }
            Ok(Statement::Exit {
                code: args[0]
                    .parse()
                    .map_err(|_| anyhow::anyhow!("{}: invalid exit code", line.number))?,
            })
        }
        "sleep" => {
            if args.len() != 1 {
                bail!("{}: sleep requires one seconds argument", line.number);
            }
            Ok(Statement::Sleep {
                seconds: args[0]
                    .parse()
                    .map_err(|_| anyhow::anyhow!("{}: invalid sleep seconds", line.number))?,
            })
        }
        _ if is_safe_command_name(command) => {
            validate_external_tokens(&tokens, line.number)?;
            Ok(Statement::ExecChecked {
                argv: parse_values(&tokens, line.number)?,
            })
        }
        _ => bail!("{}: unsupported statement: {}", line.number, line.text),
    }
}

fn parse_env_exec(tokens: &[String], line: usize) -> Result<Option<Statement>> {
    let mut env = Vec::new();
    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        let Some((name, value)) = assignment(token) else {
            break;
        };
        env.push(EnvAssign {
            name: name.to_string(),
            value: parse_value_text(value, line)?,
        });
        index += 1;
    }
    if env.is_empty() || index == tokens.len() {
        return Ok(None);
    }
    let argv_tokens = &tokens[index..];
    validate_external_tokens(argv_tokens, line)?;
    let Some(command) = argv_tokens.first() else {
        return Ok(None);
    };
    validate_shell_command(command, &argv_tokens[1..], line)?;
    if !is_safe_command_name(command) {
        bail!("{line}: unsupported statement: {}", tokens.join(" "));
    }
    Ok(Some(Statement::EnvExecChecked {
        env,
        argv: parse_values(argv_tokens, line)?,
    }))
}

fn parse_printf(args: &[String], line: usize) -> Result<Statement> {
    match args {
        [format, value] if format == "'%s\\n'" => Ok(Statement::Print {
            value: parse_value_text(value, line)?,
        }),
        [format, value, redirect] if format == "'%s\\n'" && redirect == ">&2" => {
            Ok(Statement::Error {
                value: parse_value_text(value, line)?,
            })
        }
        [format, value, redirect, target]
            if format == "'%s\\n'" && redirect == ">" && target == "&2" =>
        {
            Ok(Statement::Error {
                value: parse_value_text(value, line)?,
            })
        }
        _ => bail!("{line}: unsupported printf form"),
    }
}

pub(super) fn option_to_name(option: &str, line: usize) -> Result<String> {
    let Some(option) = option.strip_prefix("--") else {
        bail!("{line}: expected long option: {option}");
    };
    let name = option.replace('-', "_");
    if !is_valid_name(&name) {
        bail!("{line}: invalid option name: {option}");
    }
    Ok(name)
}

fn capture_argv(value: &str, line: usize) -> Result<Option<Vec<Value>>> {
    let Some(inner) = value
        .strip_prefix("$(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let tokens = split_words(inner, line)?;
    if tokens.is_empty() {
        bail!("{line}: capture command cannot be empty");
    }
    validate_external_tokens(&tokens, line)?;
    Ok(Some(parse_values(&tokens, line)?))
}

fn one_value(args: &[String], line: usize, command: &str) -> Result<Value> {
    if args.len() != 1 {
        bail!("{line}: {command} requires exactly one argument");
    }
    parse_value_text(&args[0], line)
}

fn parse_predicate(text: &str, line: usize) -> Result<Predicate> {
    if let Some(inner) = text
        .strip_prefix("[ ")
        .and_then(|text| text.strip_suffix(" ]"))
    {
        return parse_test_predicate(inner, line);
    }

    let tokens = split_words(text, line)?;
    let Some(command) = tokens.first() else {
        bail!("{line}: command predicate cannot be empty");
    };
    validate_shell_command(command, &tokens[1..], line)?;
    if !is_safe_command_name(command) {
        bail!("{line}: unsupported predicate: {text}");
    }
    validate_external_tokens(&tokens, line)?;
    Ok(Predicate::Command {
        argv: parse_values(&tokens, line)?,
    })
}

pub(super) fn parse_values(tokens: &[String], line: usize) -> Result<Vec<Value>> {
    tokens
        .iter()
        .map(|arg| parse_value_text(arg, line))
        .collect()
}

fn parse_test_predicate(text: &str, line: usize) -> Result<Predicate> {
    let tokens = split_test_words(text, line)?;
    match tokens.as_slice() {
        [flag, value] if flag == "-z" => Ok(Predicate::Empty {
            value: parse_value_text(value, line)?,
        }),
        [flag, value] if flag == "-n" => Ok(Predicate::NotEmpty {
            value: parse_value_text(value, line)?,
        }),
        [flag, path] if flag == "-f" => Ok(Predicate::FileExists {
            path: parse_value_text(path, line)?,
        }),
        [flag, path] if flag == "-d" => Ok(Predicate::DirExists {
            path: parse_value_text(path, line)?,
        }),
        [left, op, right] if op == "=" => {
            if let Some(value) = json_empty_value(left, line)? {
                return match right.as_str() {
                    "true" => Ok(Predicate::JsonEmpty { value }),
                    "false" => Ok(Predicate::JsonNotEmpty { value }),
                    _ => bail!("{line}: unsupported json empty comparison: {text}"),
                };
            }
            Ok(Predicate::Eq {
                left: parse_value_text(left, line)?,
                right: parse_value_text(right, line)?,
            })
        }
        [left, op, right] if op == "!=" => Ok(Predicate::Neq {
            left: parse_value_text(left, line)?,
            right: parse_value_text(right, line)?,
        }),
        [left, op, right] if op == "-lt" => Ok(Predicate::IntLt {
            left: parse_value_text(left, line)?,
            right: parse_value_text(right, line)?,
        }),
        [left, op, right] if op == "-le" => Ok(Predicate::IntLte {
            left: parse_value_text(left, line)?,
            right: parse_value_text(right, line)?,
        }),
        [left, op, right] if op == "-gt" => Ok(Predicate::IntGt {
            left: parse_value_text(left, line)?,
            right: parse_value_text(right, line)?,
        }),
        [left, op, right] if op == "-ge" => Ok(Predicate::IntGte {
            left: parse_value_text(left, line)?,
            right: parse_value_text(right, line)?,
        }),
        _ => bail!("{line}: unsupported test predicate: {text}"),
    }
}

fn json_empty_value(text: &str, line: usize) -> Result<Option<Value>> {
    let Some(inner) = text
        .strip_prefix("\"$(")
        .and_then(|text| text.strip_suffix(")\""))
    else {
        return Ok(None);
    };
    let tokens = split_words(inner, line)?;
    match tokens.as_slice() {
        [runseal, tool, json, empty, value]
            if runseal == "runseal" && tool == "@tool" && json == "json" && empty == "empty" =>
        {
            Ok(Some(parse_value_text(value, line)?))
        }
        _ => bail!("{line}: unsupported command substitution predicate: {text}"),
    }
}

pub(crate) fn parse_seal(source: &str) -> Result<Program> {
    Parser::new(source).parse_program()
}
