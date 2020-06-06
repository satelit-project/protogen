use std::{collections::HashSet, error, fmt, io, path::PathBuf, process::Command, rc::Rc};

use directories::BaseDirs;

use crate::{
    config::Config,
    protoc::{
        compiler::{go::GoError, AnyCompiler, Compiler, GoCompiler, PlainCompiler, Plugin},
        provider::{DownloadError, GithubDownloader, ProtocProvider},
    },
    walk::{deep::DeepProtoWalker, PagingProtoWalker, Walker},
};

#[derive(Debug)]
pub enum GenerateError {
    NoProtoc(Box<dyn fmt::Debug + Send + Sync>),
    ReadDirFailed(io::Error),
    InvocationFailed(Box<dyn fmt::Debug + Send + Sync>),
    ProtocFailed(io::Error),
}

#[derive(Debug)]
pub struct Generator {
    root_path: PathBuf,
    config: Config,
}

impl Generator {
    pub fn new<P>(root_path: P, config: Config) -> Self
    where
        P: Into<PathBuf>,
    {
        let root_path = root_path.into();
        Self { root_path, config }
    }

    pub fn generate(&self) -> Result<(), GenerateError> {
        let walker = self.make_walker(self.config.protoc.exclude.clone())?;

        for plugin_cfg in &self.config.plugins {
            let plugin: Plugin = plugin_cfg.into();
            let compiler = self.make_compiler(plugin)?;

            for page in walker.clone() {
                let mut page = page
                    .map_err(|e| GenerateError::ReadDirFailed(e))?
                    .peekable();

                if page.peek().is_none() {
                    continue;
                }

                let mut command = self.command_for_page(compiler.clone(), page)?;
                command.current_dir(&self.root_path);

                let mut child = command
                    .spawn()
                    .map_err(|e| GenerateError::ProtocFailed(e))?;
                child.wait().map_err(|e| GenerateError::ProtocFailed(e))?;
            }
        }

        Ok(())
    }

    fn command_for_page<C, W>(&self, mut compiler: C, page: W) -> Result<Command, GenerateError>
    where
        C: Compiler,
        W: Walker,
    {
        compiler
            .set_protos(page)
            .map_err(|e| GenerateError::ReadDirFailed(e))?;
        let mut raw_command = compiler.command();
        if raw_command.is_empty() {
            return Err(GenerateError::InvocationFailed(Box::new(
                "empty invocation",
            )));
        }

        let mut cmd = Command::new(raw_command.remove(0));
        cmd.args(raw_command);

        Ok(cmd)
    }

    fn make_compiler(&self, plugin: Plugin) -> Result<impl Compiler, GenerateError> {
        let dirs = BaseDirs::new()
            .ok_or_else(|| GenerateError::NoProtoc(Box::new("can't create protoc cache")))?;
        let caches_path = dirs.cache_dir();

        let downloader = GithubDownloader::default();
        let provider = ProtocProvider::new(&self.config.protoc.version, downloader, caches_path);

        if provider.binary_path().is_none() {
            provider.download()?;
        }

        let protoc_path = provider
            .binary_path()
            .ok_or_else(|| GenerateError::NoProtoc(Box::new("no protoc binary found")))?;

        let mut compiler = match plugin.name() {
            "go" => {
                let mut compiler = GoCompiler::new(protoc_path, plugin)?;

                if let Some(path) = provider.include_path() {
                    compiler.set_compiler_includes(path);
                }

                if let Some(ref paths) = self.config.protoc.exclude {
                    compiler.set_output_exludes(paths.iter())
                }

                AnyCompiler::Go(compiler)
            }
            _ => AnyCompiler::Plain(PlainCompiler::new(protoc_path, plugin)),
        };

        if let Some(path) = provider.include_path() {
            compiler
                .add_include(path)
                .map_err(|e| GenerateError::ProtocFailed(e))?;
        }

        compiler
            .add_include(&self.root_path)
            .map_err(|e| GenerateError::ProtocFailed(e))?;

        if let Some(ref includes) = self.config.protoc.include {
            for include in includes.iter() {
                compiler
                    .add_include(include)
                    .map_err(|e| GenerateError::ProtocFailed(e))?;
            }
        }

        Ok(compiler)
    }

    fn make_walker(
        &self,
        excludes: Option<Vec<PathBuf>>,
    ) -> Result<
        PagingProtoWalker<
            impl Fn(PathBuf, Rc<HashSet<PathBuf>>) -> DeepProtoWalker + Clone,
            DeepProtoWalker,
        >,
        GenerateError,
    > {
        let root_dir = self.root_path.clone();
        let mut walker =
            PagingProtoWalker::new(root_dir, |p: PathBuf, e| DeepProtoWalker::new(p, e));

        if let Some(excludes) = excludes {
            match walker.set_exclude(excludes.into_iter()) {
                Err(e) => return Err(GenerateError::ReadDirFailed(e)),
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

impl From<GoError> for GenerateError {
    fn from(e: GoError) -> Self {
        GenerateError::InvocationFailed(Box::new(e))
    }
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateError::NoProtoc(e) => write!(f, "Proto compiler not found: {:?}", e),
            GenerateError::ReadDirFailed(e) => write!(f, "Failed to read directory: {}", e),
            GenerateError::InvocationFailed(e) => write!(f, "Protoc invocation failed: {:?}", e),
            GenerateError::ProtocFailed(e) => write!(f, "Protoc returned error: {}", e),
        }
    }
}

impl error::Error for GenerateError {}

impl Walker for std::iter::Peekable<DeepProtoWalker> {}
