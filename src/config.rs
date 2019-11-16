use std::path::PathBuf;

use semver::Version;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub protoc: Protoc,
    pub plugins: Vec<Plugin>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Protoc {
    pub version: Version,
    pub include: Option<Vec<PathBuf>>,
    pub exclude: Option<Vec<PathBuf>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Plugin {
    pub name: String,
    pub output: PathBuf,
    pub options: String,
    pub path: Option<PathBuf>,
}
