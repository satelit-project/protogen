use std::path::PathBuf;

use serde::Deserialize;
use semver::Version;

#[derive(Deserialize)]
pub struct Config {
    pub protoc: Protoc,
    pub excludes: Vec<PathBuf>,
    pub generation: Generation,
}

#[derive(Deserialize)]
pub struct Protoc {
    pub version: Version,
    pub includes: Vec<String>,
}

#[derive(Deserialize)]
pub struct Generation {
    pub plugins: Vec<Plugins>,
    pub go: Option<GoConfig>,
}

#[derive(Deserialize)]
pub struct Plugins {
    pub name: String,
    pub output: PathBuf,
    pub options: Vec<String>,
    pub path: Option<PathBuf>,
}

#[derive(Deserialize)]
pub struct GoConfig {
    pub import_path: String,
}