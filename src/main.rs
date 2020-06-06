use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{self, Context};
use structopt::StructOpt;
use toml;

use protogen::{config, gen};

#[derive(Debug, StructOpt)]
#[structopt(name = "protogen")]
struct Args {
    #[structopt(short, long)]
    verbose: bool,

    #[structopt(short, long, parse(from_os_str), default_value = "protogen.toml")]
    config: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    let config_path = root_dir(&args.config)?;
    let config = parse_config(&args.config)?;

    let generator = gen::Generator::new(config_path, config);
    generator.generate()?;

    Ok(())
}

fn parse_config(path: &Path) -> anyhow::Result<config::Config> {
    let mut file = File::open(path).with_context(|| format!("failed to open {:?}", path))?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    let config = toml::from_slice(&buf).with_context(|| "failed to parse config")?;
    Ok(config)
}

fn root_dir(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.into());
    }

    let mut cwd = std::env::current_dir()?;
    cwd.push(path);

    let mut root = cwd.canonicalize()?;
    root.pop();
    Ok(root)
}
