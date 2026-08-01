mod captor;
mod control;
mod display;
mod normaliser;
mod settings;
mod transform;

use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// path to the YAML configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let settings = settings::Settings::load(&args.config)?;

    let mut controller = settings.controller.build(settings.display, settings.merger);

    controller.run();
    Ok(())
}
