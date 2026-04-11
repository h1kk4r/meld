mod app;
mod cli;
mod config;
mod error;
mod modules;
mod render;
mod util;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(err) = app::run(cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
