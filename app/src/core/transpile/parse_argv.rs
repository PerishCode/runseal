use std::collections::BTreeMap;

use anyhow::{Result, bail};

use super::ast::{ArgvKind, ArgvSpec, Statement};
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
        specs: argv_specs(order, defaults, kinds)?,
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
    if parser
        .peek_text()
        .is_some_and(|text| text.starts_with("if [ \"$#\" -lt 2 ]; then "))
    {
        parser.next();
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

fn argv_specs(
    order: Vec<String>,
    mut defaults: BTreeMap<String, String>,
    mut kinds: BTreeMap<String, ArgvKind>,
) -> Result<Vec<ArgvSpec>> {
    let mut specs = Vec::new();
    for name in order {
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
