pub mod go;
pub mod plain;

pub use go::GoCompiler;
pub use plain::PlainCompiler;

use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

use crate::{config, walk};

pub trait Compiler: Clone {
    fn add_include<P: Into<PathBuf>>(&mut self, path: P) -> io::Result<()>;
    fn set_protos<W: walk::Walker>(&mut self, protos: W) -> io::Result<()>;
    fn command(self) -> Vec<OsString>;
}

#[derive(Clone)]
pub enum AnyCompiler {
    Plain(PlainCompiler),
    Go(GoCompiler),
}

impl Compiler for AnyCompiler {
    fn add_include<P: Into<PathBuf>>(&mut self, path: P) -> io::Result<()> {
        match self {
            AnyCompiler::Plain(c) => c.add_include(path),
            AnyCompiler::Go(c) => c.add_include(path),
        }
    }

    fn set_protos<W: walk::Walker>(&mut self, protos: W) -> io::Result<()> {
        match self {
            AnyCompiler::Plain(c) => c.set_protos(protos),
            AnyCompiler::Go(c) => c.set_protos(protos),
        }
    }

    fn command(self) -> Vec<OsString> {
        match self {
            AnyCompiler::Plain(c) => c.command(),
            AnyCompiler::Go(c) => c.command(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plugin {
    name: String,
    path: Option<PathBuf>,
    output: PathBuf,
    options: Vec<String>,
}

impl Plugin {
    pub fn new(name: String, output: PathBuf) -> Self {
        Self {
            name,
            path: None,
            output,
            options: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn add_option<O: Into<String>>(&mut self, option: O) {
        self.options.push(option.into());
    }

    pub fn set_path<P: Into<PathBuf>>(&mut self, path: P) {
        self.path = Some(path.into());
    }

    pub fn args(self) -> Vec<OsString> {
        let mut plugin = OsString::new();

        plugin.push(format!("protoc-gen-{}", self.name));
        if let Some(path) = self.path {
            plugin.push("=");
            plugin.push(path);
        }

        let mut args = vec![
            OsString::from("--plugin"),
            plugin,
            format!("--{}_out", self.name).into(),
            self.output.into_os_string(),
        ];

        args.push(format!("--{}_opt", self.name).into());
        args.push(self.options.join(",").into());

        args
    }
}

impl From<config::Plugin> for Plugin {
    fn from(p: config::Plugin) -> Self {
        let mut plugin = Plugin::new(p.name, p.output);
        if let Some(path) = p.path {
            plugin.set_path(path);
        }

        if let Some(options) = p.options.as_ref() {
            for option in options.split(",") {
                plugin.add_option(option);
            }
        }

        plugin
    }
}

impl From<&config::Plugin> for Plugin {
    fn from(p: &config::Plugin) -> Self {
        p.clone().into()
    }
}
