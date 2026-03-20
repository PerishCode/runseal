use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::core::alias_store::AliasStore;
use crate::core::config::{RawEnv, resolve_runseal_home};

pub struct AliasAppendOptions {
    pub name: String,
    pub profile: String,
}

pub fn run_list() -> Result<()> {
    let runseal_home = resolve_runseal_home(&RawEnv::from_process())?;
    let store = AliasStore::load(&runseal_home)?;
    if store.list().next().is_none() {
        println!("No aliases configured.");
        return Ok(());
    }

    println!("Aliases:");
    for (name, entry) in store.list() {
        println!("- {} -> {}", name, entry.profile);
    }
    Ok(())
}

pub fn run_append(options: AliasAppendOptions) -> Result<()> {
    if options.name.trim().is_empty() {
        bail!("alias name cannot be empty");
    }

    let runseal_home = resolve_runseal_home(&RawEnv::from_process())?;
    let mut store = AliasStore::load(&runseal_home)?;

    if !Path::new(&options.profile).is_file() {
        bail!("profile file not found: {}", options.profile);
    }

    store.append(options.name.clone(), options.profile.clone())?;
    let path = store.save(&runseal_home)?;
    println!(
        "Appended alias: {} -> {} ({})",
        options.name,
        options.profile,
        path.display()
    );
    Ok(())
}

pub fn resolve_profile_for_alias(name: &str) -> Result<Option<String>> {
    let runseal_home = resolve_runseal_home(&RawEnv::from_process())
        .context("unable to resolve runseal home for alias lookup")?;
    let store = AliasStore::load(&runseal_home)?;
    Ok(store.get(name).map(|entry| entry.profile.clone()))
}
