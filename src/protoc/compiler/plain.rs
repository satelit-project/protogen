use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use super::Compiler;
use super::Plugin;
use crate::walk;

#[derive(Debug, Clone)]
pub struct PlainCompiler {
    path: PathBuf,
    include_paths: Vec<PathBuf>,
    plugin: Plugin,
    proto_paths: Vec<PathBuf>,
}

impl PlainCompiler {
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

    pub(super) fn plugin_mut(&mut self) -> &mut Plugin {
        &mut self.plugin
    }
}

impl Compiler for PlainCompiler {
    fn add_include<P: Into<PathBuf>>(&mut self, path: P) -> io::Result<()> {
        self.include_paths.push(path.into());
        Ok(())
    }

    fn set_protos<W: walk::Walker>(&mut self, protos: W) -> io::Result<()> {
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

    fn command(self) -> Vec<OsString> {
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
