use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use crate::config;
use crate::walk;

#[derive(Debug, Clone)]
pub struct Compiler {
    path: PathBuf,
    include_paths: Vec<PathBuf>,
    plugin: Plugin,
    proto_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    name: String,
    path: Option<PathBuf>,
    output: PathBuf,
    options: Option<String>,
}

impl Compiler {
    pub fn new<P: Into<PathBuf>>(path: P, plugin: Plugin) -> Self {
        let include_paths = vec![];
        let proto_paths = vec![];

        Self {
            path: path.into(),
            include_paths,
            plugin,
            proto_paths,
        }
    }

    pub fn add_include<P: Into<PathBuf>>(&mut self, path: P) {
        self.include_paths.push(path.into());
    }

    pub fn set_protos<W: walk::Walker>(&mut self, protos: W) -> io::Result<()> {
        let mut buf = vec![];
        match protos.size_hint() {
            (x, Some(y)) => buf.reserve(y - x),
            _ => (),
        }

        for proto in protos {
            buf.push(proto?);
        }

        self.proto_paths = buf;
        Ok(())
    }

    pub fn command(self) -> Vec<OsString> {
        let mut buf = vec![];
        buf.reserve(self.include_paths.len() * 2 + self.proto_paths.len());

        buf.push(self.path.into_os_string());
        for include in self.include_paths {
            buf.push("-I".into());
            buf.push(include.into_os_string());
        }

        buf.append(&mut self.plugin.args());
        for proto in self.proto_paths {
            buf.push(proto.into_os_string());
        }

        buf
    }
}

impl Plugin {
    pub fn new(name: String, output: PathBuf) -> Self {
        Self {
            name,
            path: None,
            output,
            options: None,
        }
    }

    pub fn set_options(&mut self, options: String) {
        self.options = Some(options);
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

        if let Some(options) = self.options {
            args.push(format!("--{}_opt", self.name).into());
            args.push(options.into());
        }

        args
    }
}

impl From<config::Plugin> for Plugin {
    fn from(p: config::Plugin) -> Self {
        let mut plugin = Plugin::new(p.name, p.output);
        if let Some(path) = p.path {
            plugin.set_path(path);
        }

        plugin.set_options(p.options);
        plugin
    }
}

impl From<&config::Plugin> for Plugin {
    fn from(p: &config::Plugin) -> Self {
        p.clone().into()
    }
}
