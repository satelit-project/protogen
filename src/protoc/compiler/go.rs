use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::string;
use std::rc::Rc;
use std::collections::HashSet;

use crate::walk;
use super::plain::PlainCompiler;
use super::Compiler;
use super::Plugin;

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
    
    // TODO: use protoc to get go_module option from google's protos
    compiler_includes: Option<PathBuf>,
}

impl GoCompiler {
    pub fn new<P: Into<PathBuf>, I: Into<PathBuf>>(path: P, plugin: Plugin, compiler_includes: Option<I>) -> Result<Self, GoError> {
        let path = path.into();
        let module = derive_module(&path)?;
        let mut package_path = package_path(&path)?;
        package_path.push(module);

        let compiler = PlainCompiler::new(path, plugin);
        let import_path = package_path.join("/");
        Ok(Self {
            compiler,
            import_path,
            compiler_includes: compiler_includes.map(|i| i.into()),
        })
    }

    // TODO: protoc to get 'go_module' and handle conversion errors
    fn map_packages(&mut self, path: &Path) -> io::Result<()> {
        let walker = walk::deep::DeepProtoWalker::new(path, Rc::new(HashSet::new()));    
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
    dbg!(out_path);
    let output = Command::new("go")
        .args(&["list", "-m"])
        .current_dir(out_path)
        .output()?;

    let module = String::from_utf8(output.stdout)?;
    Ok(dbg!(module))
}

fn package_path(out_path: &Path) -> Result<Vec<String>, GoError> {
    let mut path = vec![];
    for ancestor in out_path.ancestors() {
        let content = fs::read_dir(ancestor)?;
        for entry in content {
            let entry = entry?;
            dbg!(&entry);
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

fn package_path_from_path<'p>(path: &'p OsStr) -> Result<&'p str, GoError> {
    let package = match path.to_str() {
        Some(p) => p,
        None => {
            return Err(GoError::Parsing(
                "failed to convert Path to String".to_owned(),
            ))
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
