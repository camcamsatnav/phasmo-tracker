use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use phasmo_evidence_tracker::{config, ghosts, tracker};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, default_value = config::DEFAULT_CONFIG_PATH)]
    config: PathBuf,

    #[arg(long, default_value = ghosts::DEFAULT_GHOSTS_PATH)]
    ghosts: PathBuf,

    #[arg(long, help = "Emit newline-delimited JSON events for desktop clients")]
    json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.json {
        tracker::run_with_output_mode(&cli.config, &cli.ghosts, tracker::OutputMode::Json)
    } else {
        tracker::run(&cli.config, &cli.ghosts)
    }
}
