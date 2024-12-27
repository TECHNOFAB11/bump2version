use std::collections::HashMap;

use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    pub(crate) current_version: String,
    pub(crate) message: Option<String>,
    pub(crate) commit: bool,
    pub(crate) tag: bool,
    #[serde(default = "default_parse")]
    pub(crate) parse: String,
    #[serde(default = "default_serialize")]
    pub(crate) serialize: String,

    pub(crate) part: HashMap<String, Part>,
    pub(crate) file: HashMap<String, File>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PartType {
    String,
    #[serde(other)]
    Number,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Part {
    /// Type of the part (number or string)
    pub(crate) r#type: PartType,
    /// Valid values for the part (only used when type is string)
    pub(crate) values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct File {
    /// Format to replace, eg. `current_version = "{version}"`
    pub(crate) format: String,
}

fn default_parse() -> String {
    String::from(r"(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)")
}

fn default_serialize() -> String {
    String::from("{major}.{minor}.{patch}")
}
