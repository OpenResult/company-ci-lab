mod cli;
mod commands;
mod container_engine;
mod context;
mod error;
mod image_config;
mod impact;
mod plan;
mod runner;

use cli::Cli;
use commands::dispatch;
use runner::ShellRunner;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse(std::env::args().skip(1))?;
    let runner = ShellRunner;
    dispatch(cli, &runner)
}
