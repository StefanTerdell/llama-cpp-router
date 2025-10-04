use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Schema,
    Serve {
        #[clap(
            long,
            short,
            default_value = "$HOME/.config/llama-cpp-router/config.json"
        )]
        config_file: PathBuf,
    },
}

impl Cli {
    pub fn command() -> CliCommand {
        Self::parse().command
    }
}
