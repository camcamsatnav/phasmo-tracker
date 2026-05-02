use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use phasmo_evidence_tracker::{config, tracker};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, default_value = config::DEFAULT_CONFIG_PATH)]
    config: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracker::run(&cli.config)
}
