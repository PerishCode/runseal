use anyhow::Result;

use crate::plugins::host::{PluginHostOptions, run_plugin};

#[derive(Debug, Clone)]
pub struct PluginRunOptions {
    pub plugin: String,
    pub method: String,
    pub args: Vec<String>,
    pub force_install: bool,
}

pub fn run(options: PluginRunOptions) -> Result<()> {
    run_plugin(PluginHostOptions {
        plugin: options.plugin,
        method: options.method,
        args: options.args,
        force_install: options.force_install,
    })
}
