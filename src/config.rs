use serde::Deserialize;

use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub protoc: Protoc,
    #[serde(default)]
    pub excludes: Vec<PathBuf>,
    pub generation: Generation,
}

#[derive(Deserialize)]
pub struct Protoc {
    pub version: String,
    #[serde(default)]
    pub includes: Vec<String>,
}

#[derive(Deserialize)]
pub struct Generation {
    pub plugins: Vec<Plugins>,
    #[serde(default)]
    pub go: Option<GoConfig>,
}

#[derive(Deserialize)]
pub struct Plugins {
    pub name: String,
    pub output: PathBuf,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
}

#[derive(Deserialize)]
pub struct GoConfig {
    pub import_path: String,
}
