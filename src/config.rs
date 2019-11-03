use std::path::{Path, PathBuf};
use std::io;

use semver::Version;
use serde::Deserialize;
use serde_yaml;

#[derive(Deserialize)]
#[derive(Debug, Clone)]
pub struct Config {
    pub protoc: Protoc,
    pub exclude: Option<Vec<PathBuf>>,
    pub plugins: Vec<Plugin>,
    pub go: Option<GoConfig>,
}

#[derive(Deserialize)]
#[derive(Debug, Clone)]
pub struct Protoc {
    pub version: Version,
    pub includes: Vec<PathBuf>,
}

#[derive(Deserialize)]
#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub output: PathBuf,
    pub options: Vec<String>,
    pub path: Option<PathBuf>,
}

#[derive(Deserialize)]
#[derive(Debug, Clone)]
pub struct GoConfig {
    pub module: String,
}

pub enum ReadError {
    Io(io::Error),
    Format(serde_yaml::Error),
}

pub fn read_at_path(path: &Path) -> Result<Config, ReadError> {
    let file = std::fs::File::open(path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config)
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

impl From<serde_yaml::Error> for ReadError {
    fn from(e: serde_yaml::Error) -> Self {
        ReadError::Format(e)
    }
}
