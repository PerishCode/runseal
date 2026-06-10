use anyhow::{Result, bail};

pub(super) fn assignment(text: &str) -> Option<(&str, &str)> {
    let (name, value) = text.split_once('=')?;
    is_valid_name(name).then_some((name, value))
}

pub(super) fn split_words(text: &str, line: usize) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut parameter_depth = 0_usize;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if quote.is_none() && ch == '$' && chars.peek() == Some(&'{') {
            current.push(ch);
            current.push(chars.next().expect("peeked char should exist"));
            parameter_depth += 1;
            continue;
        }
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
            None if ch == '}' && parameter_depth > 0 => {
                current.push(ch);
                parameter_depth -= 1;
            }
            None if ch == '2' && matches!(chars.peek(), Some('>')) => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
                chars.next();
                if matches!(chars.peek(), Some('>')) {
                    chars.next();
                    words.push("2>>".to_string());
                } else {
                    words.push("2>".to_string());
                }
            }
            None if ch == '>' => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
                if matches!(chars.peek(), Some('>')) {
                    chars.next();
                    words.push(">>".to_string());
                } else {
                    words.push(">".to_string());
                }
            }
            None if ch == '|' => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
                words.push("|".to_string());
            }
            None if ch.is_whitespace() && parameter_depth == 0 => {
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
    if parameter_depth != 0 {
        bail!("{line}: unterminated parameter expansion");
    }
    if !current.is_empty() {
        words.push(current);
    }
    Ok(words)
}

pub(super) fn split_test_words(text: &str, line: usize) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut command_depth = 0_usize;
    let mut parameter_depth = 0_usize;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if quote.is_none() && ch == '$' && chars.peek() == Some(&'{') {
            current.push(ch);
            current.push(chars.next().expect("peeked char should exist"));
            parameter_depth += 1;
            continue;
        }
        if ch == '$' && chars.peek() == Some(&'(') {
            current.push(ch);
            current.push(chars.next().expect("peeked char should exist"));
            command_depth += 1;
            continue;
        }

        match quote {
            Some(q) if ch == q && command_depth == 0 => {
                current.push(ch);
                quote = None;
            }
            Some(_) => {
                if ch == ')' && command_depth > 0 {
                    command_depth -= 1;
                }
                current.push(ch);
            }
            None if ch == '\'' || ch == '"' => {
                current.push(ch);
                quote = Some(ch);
            }
            None if ch == '}' && parameter_depth > 0 => {
                current.push(ch);
                parameter_depth -= 1;
            }
            None if ch.is_whitespace() && parameter_depth == 0 => {
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
    if command_depth != 0 {
        bail!("{line}: unterminated command substitution");
    }
    if parameter_depth != 0 {
        bail!("{line}: unterminated parameter expansion");
    }
    if !current.is_empty() {
        words.push(current);
    }
    Ok(words)
}

pub(super) fn strip_comment(line: &str) -> String {
    let mut output = String::new();
    let mut quote = None;
    let mut parameter_depth = 0_usize;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if quote.is_none() && ch == '$' && chars.peek() == Some(&'{') {
            output.push(ch);
            output.push(chars.next().expect("peeked char should exist"));
            parameter_depth += 1;
            continue;
        }
        if quote.is_none() && ch == '$' && chars.peek() == Some(&'#') {
            output.push(ch);
            output.push(chars.next().expect("peeked char should exist"));
            continue;
        }
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
            None if ch == '}' && parameter_depth > 0 => {
                parameter_depth -= 1;
                output.push(ch);
            }
            None if ch == '#' && parameter_depth == 0 => break,
            None => output.push(ch),
        }
    }
    output
}

pub(super) fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub(super) fn is_safe_command_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'-'))
}
