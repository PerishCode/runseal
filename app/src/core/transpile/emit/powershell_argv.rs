use crate::core::transpile::ast::{ArgvKind, ArgvPositional, ArgvSpec};
use crate::core::transpile::emit::support::option_name;

use super::powershell_quote;

pub(super) fn emit_argv_parse(
    out: &mut String,
    specs: &[ArgvSpec],
    positional: Option<&ArgvPositional>,
    indent: usize,
) {
    let pad = "    ".repeat(indent);
    out.push_str(&format!("{pad}$__seal_argc = $args.Count\n"));
    out.push_str(&format!("{pad}$__seal_help = 'false'\n"));
    for spec in specs {
        let value = match spec.kind {
            ArgvKind::String => powershell_quote(spec.default.as_deref().unwrap_or("")),
            ArgvKind::Flag => "'false'".to_string(),
        };
        out.push_str(&format!("{pad}${} = {value}\n", spec.name));
    }
    if let Some(positional) = positional {
        out.push_str(&format!(
            "{pad}${} = {}\n",
            positional.name,
            powershell_quote(&positional.default)
        ));
    }
    out.push_str(&format!("{pad}$__seal_index = 0\n"));
    out.push_str(&format!("{pad}while ($__seal_index -lt $args.Count) {{\n"));
    out.push_str(&format!("{pad}    $__seal_arg = $args[$__seal_index]\n"));
    out.push_str(&format!("{pad}    switch -Regex ($__seal_arg) {{\n"));
    for spec in specs {
        match spec.kind {
            ArgvKind::String => emit_string_option(out, spec, indent),
            ArgvKind::Flag => emit_flag_option(out, spec, indent),
        }
    }
    out.push_str(&format!("{pad}        '^--$' {{\n"));
    out.push_str(&format!("{pad}            $__seal_index = $args.Count\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
    out.push_str(&format!("{pad}        '^(-h|--help|help)$' {{\n"));
    out.push_str(&format!("{pad}            $__seal_help = 'true'\n"));
    out.push_str(&format!("{pad}            $__seal_index += 1\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
    if let Some(positional) = positional {
        out.push_str(&format!("{pad}        default {{\n"));
        out.push_str(&format!(
            "{pad}            if ([string]::IsNullOrEmpty(${})) {{\n",
            positional.name
        ));
        out.push_str(&format!(
            "{pad}                ${} = $__seal_arg\n",
            positional.name
        ));
        out.push_str(&format!("{pad}                $__seal_index += 1\n"));
        out.push_str(&format!("{pad}                break\n"));
        out.push_str(&format!("{pad}            }} else {{\n"));
        out.push_str(&format!(
            "{pad}                throw \"{}\"\n",
            positional.extra_error.replace("$1", "$__seal_arg")
        ));
        out.push_str(&format!("{pad}            }}\n"));
        out.push_str(&format!("{pad}        }}\n"));
    } else {
        out.push_str(&format!(
            "{pad}        default {{ throw \"unknown option: $__seal_arg\" }}\n"
        ));
    }
    out.push_str(&format!("{pad}    }}\n"));
    out.push_str(&format!("{pad}}}\n"));
}

fn emit_string_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "    ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}        '^{}$' {{\n", regex_quote(&option)));
    out.push_str(&format!(
        "{pad}            if ($__seal_index + 1 -ge $args.Count) {{ throw 'missing value for {option}' }}\n"
    ));
    out.push_str(&format!(
        "{pad}            ${} = $args[$__seal_index + 1]\n",
        spec.name
    ));
    out.push_str(&format!("{pad}            $__seal_index += 2\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
    out.push_str(&format!("{pad}        '^{}=' {{\n", regex_quote(&option)));
    out.push_str(&format!(
        "{pad}            ${} = $__seal_arg.Substring({})\n",
        spec.name,
        option.len() + 1
    ));
    out.push_str(&format!("{pad}            $__seal_index += 1\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
}

fn emit_flag_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "    ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}        '^{}$' {{\n", regex_quote(&option)));
    out.push_str(&format!("{pad}            ${} = 'true'\n", spec.name));
    out.push_str(&format!("{pad}            $__seal_index += 1\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
}

fn regex_quote(value: &str) -> String {
    value.replace('-', "\\-")
}
