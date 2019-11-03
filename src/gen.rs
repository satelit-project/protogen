use std::path::PathBuf;
use std::io;
use std::fmt;
use std::rc::Rc;
use std::collections::HashSet;
use std::process::Command;

use directories::BaseDirs;

use crate::config::Config;
use crate::protoc::provider::{ProtocProvider, GithubDownloader, DownloadError};
use crate::protoc::compiler::{Plugin, Compiler};
use crate::walk::{Walker, PagingProtoWalker, deep::DeepProtoWalker};

pub enum GenerateError {
    NoProtoc(Box<dyn fmt::Display>),
    ExcludeFailed(io::Error),
    ReadDirFailed(io::Error),
    GenerationFailed(Box<dyn fmt::Display>),
}

pub struct Generator {
    root_path: PathBuf,
    config: Config,
}

impl Generator {
    pub fn new<P>(root_path: P, config: Config) -> Self
    where
        P: Into<PathBuf>
    {
        let root_path = root_path.into();
        Self { root_path, config }
    }

    pub fn generate(&self) -> Result<(), GenerateError> {
        let walker = self.make_walker(self.config.exclude.clone())?;

        for plugin_cfg in &self.config.plugins {
            let plugin: Plugin = plugin_cfg.into();
            let compiler = self.make_compiler(plugin, self.config.protoc.includes.clone())?;

            for page in walker.clone() {
                let page = page.map_err(|e| GenerateError::ReadDirFailed(e))?;
                let command = self.command_for_page(compiler.clone(), page)?;

                println!("page: {:?}\n", command);
            }
        }

        Ok(())
    }

    fn command_for_page<W>(&self, mut compiler: Compiler, page: W) -> Result<Command, GenerateError>
    where
        W: Walker,
    {
        compiler.set_protos(page).map_err(|e| GenerateError::ReadDirFailed(e))?;
        let mut raw_command = compiler.command();
        if raw_command.is_empty() {
            return Err(GenerateError::GenerationFailed(Box::new("empty invocation")));
        }

        let mut cmd = Command::new(raw_command.remove(0));
        cmd.args(raw_command);

        Ok(cmd)
    }

    fn make_compiler(&self, plugin: Plugin, includes: Vec<PathBuf>) -> Result<Compiler, GenerateError> {
        let dirs = BaseDirs::new()
            .ok_or_else(|| GenerateError::NoProtoc(Box::new("can't create protoc cache")))?;
        let caches_path = dirs.cache_dir();

        let downloader = GithubDownloader::default();
        let provider = ProtocProvider::new(&self.config.protoc.version, downloader, caches_path);

        if provider.binary_path().is_none() {
            provider.download()?;
        }

        let protoc_path = provider.binary_path()
            .ok_or_else(|| GenerateError::NoProtoc(Box::new("no protoc binary found")))?;

        let mut compiler = Compiler::new(protoc_path, plugin);
        if let Some(path) = provider.include_path() {
            compiler.add_include(path);
        }

        for include in includes {
            compiler.add_include(include);
        }

        Ok(compiler)
    }

    fn make_walker(&self, excludes: Option<Vec<PathBuf>>) -> Result<PagingProtoWalker<impl Fn(PathBuf, Rc<HashSet<PathBuf>>) -> DeepProtoWalker + Clone, DeepProtoWalker>, GenerateError> {
        let root_dir = self.root_path.clone();
        let mut walker = PagingProtoWalker::new(root_dir, |p: PathBuf, e| {
            DeepProtoWalker::new(p, e)
        });

        if let Some(excludes) = excludes {
            match walker.set_exclude(excludes.into_iter()) {
                Err(e) => return Err(GenerateError::ExcludeFailed(e)),
                _ => (),
            }
        }

        Ok(walker)
    }
}

impl From<DownloadError> for GenerateError {
    fn from(e: DownloadError) -> Self {
        GenerateError::NoProtoc(Box::new(e))
    }
}
