use anyhow::{Result, bail};

use super::*;

impl<'a> Runner<'a> {
    pub(super) fn parse_argv(
        &mut self,
        specs: &[ArgvSpec],
        positional: Option<&ArgvPositional>,
    ) -> Result<()> {
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
        if let Some(positional) = positional {
            self.vars
                .insert(positional.name.clone(), positional.default.clone());
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
                if let Some(positional) = positional {
                    let current = self.vars.get(&positional.name).cloned().unwrap_or_default();
                    if current.is_empty() {
                        self.vars.insert(positional.name.clone(), arg.clone());
                        index += 1;
                        continue;
                    }
                    eprintln!("{}", positional.extra_error.replace("$1", arg));
                    bail!("argv parse failed");
                }
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

    pub(super) fn set_function_args(&mut self, argv: &[Value]) -> Result<ArgSnapshot> {
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

    pub(super) fn restore_function_args(&mut self, old: ArgSnapshot) {
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

    pub(super) fn expanded_values(&self, values: &[Value]) -> Result<Vec<String>> {
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

    pub(super) fn argc(&self) -> usize {
        self.vars
            .get("0")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_default()
    }

    pub(super) fn current_args(&self) -> Vec<String> {
        (1..=self.argc())
            .filter_map(|index| self.vars.get(&index.to_string()).cloned())
            .collect()
    }

    pub(super) fn shift_args(&mut self, count: usize) {
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
