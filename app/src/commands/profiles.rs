use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::core::config::{RawEnv, resolve_runseal_home};

pub enum InitProfileType {
    Minimal,
    Sample,
}

pub struct ProfilesInitOptions {
    pub profile_type: InitProfileType,
    pub name: Option<String>,
    pub force: bool,
}

pub fn run_status() -> Result<()> {
    let runseal_home = resolve_runseal_home(&RawEnv::from_process())?;
    let profiles_dir = runseal_home.join("profiles");
    let default_profile = profiles_dir.join("default.json");

    println!("runseal_home: {}", runseal_home.display());
    println!("profiles_dir: {}", profiles_dir.display());
    println!(
        "default_profile: {} ({})",
        default_profile.display(),
        if default_profile.is_file() {
            "present"
        } else {
            "missing"
        }
    );

    if !profiles_dir.is_dir() {
        println!("profiles_count: 0");
        return Ok(());
    }

    let mut profiles: Vec<PathBuf> = std::fs::read_dir(&profiles_dir)
        .with_context(|| {
            format!(
                "failed to read profiles directory: {}",
                profiles_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|x| x.to_str()) == Some("json"))
        .collect();
    profiles.sort();

    println!("profiles_count: {}", profiles.len());
    for path in profiles {
        let name = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("<invalid-utf8>");
        let status = match crate::core::profile::load(&path) {
            Ok(_) => "ok".to_string(),
            Err(err) => format!("invalid: {}", err),
        };
        println!("- {} [{}]", name, status);
    }

    Ok(())
}

pub fn run_init(options: ProfilesInitOptions) -> Result<()> {
    let runseal_home = resolve_runseal_home(&RawEnv::from_process())?;
    let profiles_dir = runseal_home.join("profiles");
    std::fs::create_dir_all(&profiles_dir).with_context(|| {
        format!(
            "failed to create profiles directory: {}",
            profiles_dir.display()
        )
    })?;

    let file_name = match options.name {
        Some(name) => {
            if name.trim().is_empty() {
                bail!("--name cannot be empty");
            }
            if name.ends_with(".json") {
                name
            } else {
                format!("{}.json", name)
            }
        }
        None => "default.json".to_string(),
    };

    let target = profiles_dir.join(file_name);
    if target.exists() && !options.force {
        bail!(
            "profile already exists: {} (use --force to overwrite)",
            target.display()
        );
    }

    let body = render_profile_template(options.profile_type);
    std::fs::write(&target, body)
        .with_context(|| format!("failed to write profile file: {}", target.display()))?;

    println!("Initialized profile: {}", target.display());
    Ok(())
}

fn render_profile_template(profile_type: InitProfileType) -> String {
    match profile_type {
        InitProfileType::Minimal => r#"{
  "injections": [
    {
      "type": "env",
      "vars": {
        "RUNSEAL_PROFILE": "default"
      }
    }
  ]
}
"#
        .to_string(),
        InitProfileType::Sample => r#"{
  "injections": [
    {
      "type": "env",
      "vars": {
        "RUNSEAL_PROFILE": "sample",
        "RUNSEAL_SCOPE": "child-only"
      }
    },
    {
      "type": "env",
      "ops": [
        {
          "op": "prepend",
          "key": "PATH",
          "value": "./bin",
          "separator": "os",
          "dedup": true
        }
      ]
    }
  ]
}
"#
        .to_string(),
    }
}
