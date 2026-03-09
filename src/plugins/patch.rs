use anyhow::{Result, bail};
use serde::Deserialize;

const PATCH_SCHEMA: &str = "envlock.patch.v1";

pub fn validate_patch_json(raw: &str) -> Result<()> {
    let patch: PluginPatch = serde_json::from_str(raw)
        .map_err(|err| anyhow::anyhow!("invalid plugin patch JSON output: {err}"))?;
    if patch.schema != PATCH_SCHEMA {
        bail!(
            "unsupported plugin patch schema: {} (expected {})",
            patch.schema,
            PATCH_SCHEMA
        );
    }

    for entry in &patch.env {
        if entry.key.trim().is_empty() {
            bail!("env patch key cannot be empty");
        }
        match entry.op {
            EnvOp::Set | EnvOp::PrependPath => {
                if entry.value.as_deref().is_none() {
                    bail!(
                        "env patch entry missing value for op `{}`",
                        entry.op.as_str()
                    );
                }
                if matches!(entry.op, EnvOp::PrependPath)
                    && entry
                        .separator
                        .as_deref()
                        .unwrap_or_default()
                        .trim()
                        .is_empty()
                {
                    bail!("env patch entry for `prepend_path` requires non-empty separator");
                }
            }
            EnvOp::Unset => {
                if entry.value.is_some() {
                    bail!("env patch entry for `unset` must not include value");
                }
            }
        }
    }

    for entry in &patch.symlink {
        if !matches!(entry.op, SymlinkOp::Ensure) {
            bail!("unsupported symlink patch op: {}", entry.op.as_str());
        }
        if entry.source.trim().is_empty() || entry.target.trim().is_empty() {
            bail!("symlink patch source/target cannot be empty");
        }
        if let Some(on_exist) = &entry.on_exist
            && on_exist != "replace"
            && on_exist != "skip"
            && on_exist != "error"
        {
            bail!("unsupported symlink on_exist policy: {}", on_exist);
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PluginPatch {
    schema: String,
    env: Vec<EnvPatchEntry>,
    symlink: Vec<SymlinkPatchEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvPatchEntry {
    op: EnvOp,
    key: String,
    value: Option<String>,
    separator: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EnvOp {
    Set,
    Unset,
    PrependPath,
}

impl EnvOp {
    fn as_str(&self) -> &'static str {
        match self {
            EnvOp::Set => "set",
            EnvOp::Unset => "unset",
            EnvOp::PrependPath => "prepend_path",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SymlinkPatchEntry {
    op: SymlinkOp,
    source: String,
    target: String,
    on_exist: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SymlinkOp {
    Ensure,
}

impl SymlinkOp {
    fn as_str(&self) -> &'static str {
        match self {
            SymlinkOp::Ensure => "ensure",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_patch_json;

    #[test]
    fn patch_schema_validation_accepts_minimal_payload() {
        let payload = r#"{
          "schema":"envlock.patch.v1",
          "env":[{"op":"set","key":"A","value":"b"}],
          "symlink":[]
        }"#;
        validate_patch_json(payload).expect("payload should pass validation");
    }

    #[test]
    fn patch_schema_validation_rejects_unknown_schema() {
        let payload = r#"{
          "schema":"bad.schema",
          "env":[],
          "symlink":[]
        }"#;
        let err = validate_patch_json(payload).expect_err("validation should fail");
        assert!(err.to_string().contains("unsupported plugin patch schema"));
    }
}
