use anyhow::{Result, bail};

use super::ast::{CaseArm, Item, Predicate, Program, Statement, Value};
use super::lower::lower_functions;
use super::value::parse_value_text;

#[derive(Debug, Clone)]
struct SourceLine {
    number: usize,
    text: String,
}

struct Parser {
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
        if line.text.starts_with("if ") {
            return self.parse_if();
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
    fn expect_exact(&mut self, expected: &str) -> Result<()> {
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
    fn peek(&self) -> Option<&SourceLine> {
        self.lines.get(self.index)
    }
    fn peek_text(&self) -> Option<&str> {
        self.peek().map(|line| line.text.as_str())
    }
    fn next(&mut self) -> Option<SourceLine> {
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
    if let Some((name, value)) = assignment(&line.text) {
        if let Some(argv) = capture_argv(value, line.number)? {
            if let Some(statement) = helper_capture_statement(name, &argv) {
                return Ok(statement);
            }
            return Ok(Statement::CaptureChecked {
                name: name.to_string(),
                argv,
            });
        }
        return Ok(Statement::Assign {
            name: name.to_string(),
            value: parse_value_text(value, line.number)?,
        });
    }
    let tokens = split_words(&line.text, line.number)?;
    let Some((command, args)) = tokens.split_first() else {
        bail!("{}: expected statement", line.number);
    };
    match command.as_str() {
        "eval" => bail!("{}: unsupported statement: eval", line.number),
        "print" => Ok(Statement::Print {
            value: one_value(args, line.number, "print")?,
        }),
        "error" => Ok(Statement::Error {
            value: one_value(args, line.number, "error")?,
        }),
        "fail" => Ok(Statement::Fail {
            value: one_value(args, line.number, "fail")?,
        }),
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
                argv: tokens
                    .iter()
                    .map(|arg| parse_value_text(arg, line.number))
                    .collect::<Result<Vec<_>>>()?,
            })
        }
        _ => bail!("{}: unsupported statement: {}", line.number, line.text),
    }
}

fn validate_external_tokens(tokens: &[String], line: usize) -> Result<()> {
    for token in tokens {
        if token.starts_with('"') || token.starts_with('\'') {
            continue;
        }
        if token
            .chars()
            .any(|ch| matches!(ch, '|' | '>' | '<' | '&' | ';' | '`'))
        {
            bail!("{line}: unsupported shell metacharacter in token: {token}");
        }
    }
    Ok(())
}

fn assignment(text: &str) -> Option<(&str, &str)> {
    let (name, value) = text.split_once('=')?;
    is_valid_name(name).then_some((name, value))
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
    Ok(Some(
        tokens
            .iter()
            .map(|arg| parse_value_text(arg, line))
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn helper_capture_statement(name: &str, argv: &[Value]) -> Option<Statement> {
    match argv {
        [
            Value::Literal { text: seal },
            Value::Literal { text: string },
            Value::Literal { text: trim },
            value,
        ] if seal == "seal" && string == "string" && trim == "trim" => {
            Some(Statement::StringTrim {
                name: name.to_string(),
                value: value.clone(),
            })
        }
        _ => None,
    }
}

fn one_value(args: &[String], line: usize, command: &str) -> Result<Value> {
    if args.len() != 1 {
        bail!("{line}: {command} requires exactly one argument");
    }
    parse_value_text(&args[0], line)
}

fn parse_predicate(text: &str, line: usize) -> Result<Predicate> {
    let tokens = split_words(text, line)?;
    let Some((name, args)) = tokens.split_first() else {
        bail!("{line}: expected predicate");
    };
    match (name.as_str(), args) {
        ("empty", [value]) => Ok(Predicate::Empty {
            value: parse_value_text(value, line)?,
        }),
        ("not_empty", [value]) => Ok(Predicate::NotEmpty {
            value: parse_value_text(value, line)?,
        }),
        ("eq", [left, right]) => Ok(Predicate::Eq {
            left: parse_value_text(left, line)?,
            right: parse_value_text(right, line)?,
        }),
        ("neq", [left, right]) => Ok(Predicate::Neq {
            left: parse_value_text(left, line)?,
            right: parse_value_text(right, line)?,
        }),
        ("file_exists", [path]) => Ok(Predicate::FileExists {
            path: parse_value_text(path, line)?,
        }),
        ("dir_exists", [path]) => Ok(Predicate::DirExists {
            path: parse_value_text(path, line)?,
        }),
        ("tool_exists", [tool]) if is_valid_name(tool) => Ok(Predicate::ToolExists {
            name: tool.to_string(),
        }),
        _ => bail!("{line}: unsupported predicate: {text}"),
    }
}

fn split_words(text: &str, line: usize) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in text.chars() {
        match quote {
            Some(q) if ch == q => {
                current.push(ch);
                quote = None;
            }
            Some(_) => current.push(ch),
            None if ch == '\'' || ch == '"' => {
                current.push(ch);
                quote = Some(ch);
            }
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            None => current.push(ch),
        }
    }
    if let Some(q) = quote {
        bail!("{line}: unterminated {q} quote");
    }
    if !current.is_empty() {
        words.push(current);
    }
    Ok(words)
}

fn strip_comment(line: &str) -> String {
    let mut output = String::new();
    let mut quote = None;
    for ch in line.chars() {
        match quote {
            Some(q) if ch == q => {
                output.push(ch);
                quote = None;
            }
            Some(_) => output.push(ch),
            None if ch == '\'' || ch == '"' => {
                output.push(ch);
                quote = Some(ch);
            }
            None if ch == '#' => break,
            None => output.push(ch),
        }
    }
    output
}

fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_safe_command_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'-'))
}

pub(crate) fn parse_seal(source: &str) -> Result<Program> {
    Parser::new(source).parse_program()
}
