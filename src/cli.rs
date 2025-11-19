use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[clap(long, short, default_value_t = IpAddr::V4(Ipv4Addr::UNSPECIFIED))]
    pub ip: IpAddr,
    #[clap(long, short)]
    pub config_path: Option<PathBuf>,
    #[clap(long, short, default_value_t = 3100)]
    pub port: u16,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Schema,
    Serve {
        #[clap(flatten)]
        args: ServeArgs,
        #[clap(long, short, action)]
        detach: bool,
    },
    Start(ServeArgs),
    Stop {
        #[clap(long, short)]
        config_path: Option<PathBuf>,
    },
    Models {
        #[clap(long, short)]
        config_path: Option<PathBuf>,
        #[clap(subcommand)]
        command: Option<ModelCommand>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ModelCommand {
    List,
    Config {
        alias_or_index: Option<String>,
    },
    Load {
        #[clap(long, short, default_value_t = 3100)]
        port: u16,
        #[clap(long, short, action)]
        tail: bool,
        #[clap(long, short, action)]
        json: bool,
        alias_or_index: String,
    },
    Loaded {
        #[clap(long, short, default_value_t = 3100)]
        port: u16,
    },
    Unload {
        #[clap(long, short, default_value_t = 3100)]
        port: u16,
        alias_or_index: String,
    },
    Logs {
        #[clap(long, short, default_value_t = 3100)]
        port: u16,
        #[clap(long, short, action)]
        tail: bool,
        #[clap(long, short, action)]
        json: bool,
        alias_or_index: String,
    },
}

impl Cli {
    pub fn command() -> CliCommand {
        Self::parse().command
    }
}
