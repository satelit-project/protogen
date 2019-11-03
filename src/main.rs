use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use exitfailure;
use failure;
use structopt::StructOpt;
use toml;

use failure::ResultExt;
use protogen::config;
use protogen::gen;
use protogen::gen::GenerateError;

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
    println!("{:#?}", args);

    let config = parse_config(&args.config)
        .with_context(|_| format!("could not parse config {:?}", &args.config))?;
    println!("\n{:#?}", config);

    Ok(())
}

fn parse_config(path: &Path) -> Result<config::Config, failure::Error> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    let config = toml::from_slice(&buf)?;
    Ok(config)
}
