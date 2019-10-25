use std::ffi::OsString;

use semver::Version;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub protoc: Protoc,
    pub excludes: Vec<OsString>,
    pub generation: Generation,
}

#[derive(Deserialize)]
pub struct Protoc {
    pub version: Version,
    pub includes: Vec<OsString>,
}

#[derive(Deserialize)]
pub struct Generation {
    pub plugins: Vec<Plugins>,
    pub go: Option<GoConfig>,
}

#[derive(Deserialize)]
pub struct Plugins {
    pub name: String,
    pub output: OsString,
    pub options: Vec<String>,
    pub path: Option<OsString>,
    pub strategy: GenerationStrategy,
}

#[derive(Deserialize)]
pub struct GoConfig {
    pub import_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStrategy {
    Recursive,
}

impl Default for GenerationStrategy {
    fn default() -> Self {
        GenerationStrategy::Recursive
    }
}
