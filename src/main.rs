mod captor;
mod control;
mod display;
mod normaliser;
mod settings;
mod transform;

use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use crossterm::terminal;

use crate::{
    control::{
        control_core::ControlCore,
        decimating::{
            self,
            controller::DecimatingController,
            fir_filter::{self, FirFilter},
        },
    },
    transform::{merger::ExponentialMerger, transformer},
};

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

    let control_core = ControlCore::new(
        settings.transform_size,
        settings.sample_rate,
        settings.framerate,
        settings.display,
        settings.merger,
    );

    let fir_filter = FirFilter::new(51, 0.54_f64, fir_filter::Window::Blackman);

    let mut controller = DecimatingController::new(control_core, fir_filter, 6, 6);

    controller.run();

    Ok(())
}
