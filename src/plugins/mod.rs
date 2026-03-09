pub mod host;
pub mod patch;

pub(crate) fn builtin_plugin_script(name: &str) -> Option<&'static str> {
    match name {
        "node" => Some(include_str!("../../scripts/plugins/node.sh")),
        _ => None,
    }
}
