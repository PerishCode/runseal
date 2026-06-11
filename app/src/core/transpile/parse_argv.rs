use std::collections::BTreeMap;

use anyhow::{Result, bail};

use super::ast::{ArgvKind, ArgvPositional, ArgvSpec, Statement};
use super::parse::{Parser, option_to_name};
use super::parse_lex::assignment;

pub(super) fn parse_argv_block(parser: &mut Parser) -> Result<Statement> {
    parser.expect_exact("__seal_argc=$#")?;
    parser.expect_exact("__seal_help=false")?;

    let mut order = Vec::new();
    let mut defaults = BTreeMap::new();
    while let Some(line) = parser.peek().cloned() {
        if line.text == "while [ \"$#\" -gt 0 ]; do" {
            break;
        }
        let Some((name, value)) = assignment(&line.text) else {
            bail!("{}: expected argv variable default", line.number);
        };
        parser.next();
        order.push(name.to_string());
        defaults.insert(name.to_string(), value.to_string());
    }

    parser.expect_exact("while [ \"$#\" -gt 0 ]; do")?;
    parser.expect_exact("case \"$1\" in")?;

    let mut kinds = BTreeMap::new();
    let mut positional = None;
    loop {
        let Some(line) = parser.peek().cloned() else {
            bail!("missing esac for argv parser");
        };
        match line.text.as_str() {
            "esac" => {
                parser.next();
                break;
            }
            "--)" => parse_double_dash(parser)?,
            "-h|--help|help)" => parse_help(parser)?,
            "*) fail \"unknown option: $1\" ;;" | "*) seal_fail \"unknown option: $1\" ;;" => {
                parser.next();
            }
            "*)" => {
                positional = Some(parse_positional_arm(parser, &defaults)?);
            }
            text if text.starts_with("--") && text.ends_with("=*)") => {
                parse_eq_arm(parser, &mut kinds, text, line.number)?;
            }
            text if text.starts_with("--") && text.ends_with(')') => {
                parse_option_arm(parser, &mut kinds, text, line.number)?;
            }
            _ => bail!(
                "{}: unsupported argv parser arm: {}",
                line.number,
                line.text
            ),
        }
    }
    parser.expect_exact("done")?;
    Ok(Statement::ArgvParse {
        specs: argv_specs(order, defaults, kinds, positional.as_ref())?,
        positional,
    })
}

fn parse_eq_arm(
    parser: &mut Parser,
    kinds: &mut BTreeMap<String, ArgvKind>,
    text: &str,
    line: usize,
) -> Result<()> {
    let option = text.trim_end_matches("=*)");
    let name = option_to_name(option, line)?;
    parser.next();
    parser.expect_exact(&format!("{name}=${{1#{option}=}}"))?;
    parser.expect_exact("shift")?;
    parser.expect_exact(";;")?;
    kinds.insert(name, ArgvKind::String);
    Ok(())
}

fn parse_option_arm(
    parser: &mut Parser,
    kinds: &mut BTreeMap<String, ArgvKind>,
    text: &str,
    line: usize,
) -> Result<()> {
    let option = text.trim_end_matches(')');
    let name = option_to_name(option, line)?;
    parser.next();
    if parse_missing_value_guard(parser)? {
        parser.expect_exact(&format!("{name}=$2"))?;
        parser.expect_exact("shift 2")?;
        parser.expect_exact(";;")?;
        kinds.insert(name, ArgvKind::String);
    } else {
        parser.expect_exact(&format!("{name}=true"))?;
        parser.expect_exact("shift")?;
        parser.expect_exact(";;")?;
        kinds.insert(name, ArgvKind::Flag);
    }
    Ok(())
}

fn parse_missing_value_guard(parser: &mut Parser) -> Result<bool> {
    let Some(line) = parser.peek().cloned() else {
        bail!("missing argv option arm body");
    };
    if line.text.starts_with("if [ \"$#\" -lt 2 ]; then ") {
        parser.next();
        return Ok(true);
    }
    if line.text != "if [ \"$#\" -lt 2 ]; then" {
        return Ok(false);
    }

    parser.next();
    while let Some(next) = parser.peek().cloned() {
        parser.next();
        if next.text == "fi" {
            return Ok(true);
        }
    }
    bail!("missing fi for argv missing-value guard");
}

fn parse_double_dash(parser: &mut Parser) -> Result<()> {
    parser.expect_exact("--)")?;
    parser.expect_exact("shift")?;
    parser.expect_exact("break")?;
    parser.expect_exact(";;")
}

fn parse_help(parser: &mut Parser) -> Result<()> {
    parser.expect_exact("-h|--help|help)")?;
    parser.expect_exact("__seal_help=true")?;
    parser.expect_exact("shift")?;
    parser.expect_exact(";;")
}

fn parse_positional_arm(
    parser: &mut Parser,
    defaults: &BTreeMap<String, String>,
) -> Result<ArgvPositional> {
    parser.expect_exact("*)")?;
    let line = parser
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing positional argv arm body"))?;
    let Some(name) = line
        .text
        .strip_prefix("if [ -z \"$")
        .and_then(|text| text.strip_suffix("\" ]; then"))
    else {
        bail!("{}: unsupported argv positional arm", line.number);
    };
    parser.expect_exact(&format!("{name}=$1"))?;
    parser.expect_exact("shift")?;
    parser.expect_exact("else")?;
    let fail_line = parser
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing argv positional else body"))?;
    let extra_error = fail_line
        .text
        .strip_prefix("fail \"")
        .and_then(|text| text.strip_suffix('"'))
        .or_else(|| {
            fail_line
                .text
                .strip_prefix("seal_fail \"")
                .and_then(|text| text.strip_suffix('"'))
        })
        .ok_or_else(|| {
            anyhow::anyhow!("{}: unsupported argv positional else arm", fail_line.number)
        })?;
    parser.expect_exact("fi")?;
    parser.expect_exact(";;")?;
    Ok(ArgvPositional {
        name: name.to_string(),
        default: defaults.get(name).cloned().unwrap_or_default(),
        extra_error: extra_error.to_string(),
    })
}

fn argv_specs(
    order: Vec<String>,
    mut defaults: BTreeMap<String, String>,
    mut kinds: BTreeMap<String, ArgvKind>,
    positional: Option<&ArgvPositional>,
) -> Result<Vec<ArgvSpec>> {
    let mut specs = Vec::new();
    for name in order {
        if positional.is_some_and(|positional| positional.name == name) {
            defaults.remove(&name);
            continue;
        }
        let Some(kind) = kinds.remove(&name) else {
            bail!("missing argv parser arm for {name}");
        };
        let default = defaults.remove(&name).unwrap_or_default();
        let default = match kind {
            ArgvKind::String => Some(default),
            ArgvKind::Flag => None,
        };
        specs.push(ArgvSpec {
            name,
            kind,
            default,
        });
    }
    Ok(specs)
}
