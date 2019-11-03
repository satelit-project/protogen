use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use exitfailure;
use failure;
use structopt::StructOpt;
use toml;

use failure::ResultExt;
use protogen::config;
use protogen::gen;

#[derive(Debug, StructOpt)]
#[structopt(name = "protogen")]
struct Args {
    #[structopt(short, long)]
    verbose: bool,

    #[structopt(short, long, parse(from_os_str), default_value = "protogen.toml")]
    config: PathBuf,
}

fn main() -> Result<(), exitfailure::ExitFailure> {
    let args = Args::from_args();
    let config_path = root_dir(&args.config)?;
    let config = parse_config(&args.config)?;

    let generator = gen::Generator::new(config_path, config);
    generator.generate()?;

    Ok(())
}

fn parse_config(path: &Path) -> Result<config::Config, failure::Error> {
    let mut file = File::open(path).with_context(|_| format!("failed to open {:?}", path))?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    let config = toml::from_slice(&buf).with_context(|_| "failed to parse config")?;
    Ok(config)
}

fn root_dir(path: &Path) -> Result<PathBuf, failure::Error> {
    if path.is_absolute() {
        return Ok(path.into());
    }

    let mut cwd = std::env::current_dir()?;
    cwd.push(path);
    
    let mut root = cwd.canonicalize()?;
    root.pop();
    Ok(root)
}
