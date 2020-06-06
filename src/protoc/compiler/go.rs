use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    string,
};

use super::{plain::PlainCompiler, Compiler, Plugin};
use crate::walk;

#[derive(Debug)]
pub enum GoError {
    /// Indicates that 'go.mod' file wasn't found
    NoModules,
    Io(io::Error),
    Parsing(String),
}

#[derive(Debug, Clone)]
pub struct GoCompiler {
    compiler: PlainCompiler,
    import_path: String,
    output_excludes: Option<Rc<HashSet<PathBuf>>>,

    // TODO: use protoc to get go_module option from google's protos
    compiler_includes: Option<PathBuf>,
}

impl GoCompiler {
    pub fn new<P: Into<PathBuf>>(path: P, plugin: Plugin) -> Result<Self, GoError> {
        let module = derive_module(plugin.output())?;
        let mut package_path = package_path(plugin.output())?;
        package_path.push(module);
        package_path.reverse();

        let compiler = PlainCompiler::new(path.into(), plugin);
        let import_path = package_path.join("/");
        Ok(Self {
            compiler,
            import_path,
            output_excludes: None,
            compiler_includes: None,
        })
    }

    pub fn set_compiler_includes<I>(&mut self, includes: I)
    where
        I: Into<PathBuf>,
    {
        self.compiler_includes = Some(includes.into());
    }

    pub fn set_output_exludes<I, P>(&mut self, excludes: I)
    where
        I: Iterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.output_excludes = Some(Rc::new(excludes.map(|p| p.into()).collect()));
    }

    // TODO: protoc to get 'go_module' and handle conversion errors
    fn map_packages(&mut self, path: &Path) -> io::Result<()> {
        let excludes = self
            .output_excludes
            .get_or_insert_with(|| Rc::new(HashSet::new()));
        let walker = walk::deep::DeepProtoWalker::new(path, excludes.clone());

        for proto in walker {
            let proto = proto?;
            let mut relative_path = proto.strip_prefix(path).expect("unrelated proto");
            let mut mapping = format!("M{}=", relative_path.to_str().expect("utf-8 path expected"));

            relative_path = relative_path.parent().expect("not a proto");
            mapping.push_str(&self.import_path);
            mapping.push_str("/");
            mapping.push_str(relative_path.to_str().expect("utf-8 path expected"));

            self.compiler.plugin_mut().add_option(mapping);
        }

        Ok(())
    }
}

impl Compiler for GoCompiler {
    fn add_include<P: Into<PathBuf>>(&mut self, path: P) -> io::Result<()> {
        let path = path.into();
        match self.compiler_includes {
            Some(ref p) if &path == p => (),
            _ => self.map_packages(&path)?,
        }

        self.compiler.add_include(path)
    }

    fn set_protos<W: walk::Walker>(&mut self, protos: W) -> io::Result<()> {
        self.compiler.set_protos(protos)
    }

    fn command(self) -> Vec<OsString> {
        self.compiler.command()
    }
}

fn derive_module(out_path: &Path) -> Result<String, GoError> {
    let output = Command::new("go")
        .args(&["list", "-m"])
        .current_dir(&out_path)
        .output()?;

    let mut module = String::from_utf8(output.stdout)?;
    if module.ends_with("\n") {
        module.truncate(module.len() - 1);
    }

    Ok(module)
}

fn package_path(out_path: &Path) -> Result<Vec<String>, GoError> {
    let mut path = vec![];
    for ancestor in out_path.ancestors() {
        let content = fs::read_dir(ancestor)?;
        for entry in content {
            let entry = entry?;
            if entry.file_name() == "go.mod" {
                return Ok(path);
            }
        }

        match ancestor.file_name() {
            Some(dir) => {
                let package = package_path_from_path(dir)?;
                path.push(package.to_owned());
            }
            None => break, // reached top dir and didn't find anything
        };
    }

    Err(GoError::NoModules)
}

fn package_path_from_path(path: &OsStr) -> Result<&str, GoError> {
    let package = match path.to_str() {
        Some(p) => p,
        None => {
            return Err(GoError::Parsing(
                "failed to convert Path to String".to_owned(),
            ));
        }
    };

    Ok(package)
}

impl From<io::Error> for GoError {
    fn from(e: io::Error) -> Self {
        GoError::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for GoError {
    fn from(e: string::FromUtf8Error) -> Self {
        GoError::Parsing(e.to_string())
    }
}
