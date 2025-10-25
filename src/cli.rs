use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

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
        #[clap(long, short, default_value_t = IpAddr::V4(Ipv4Addr::UNSPECIFIED))]
        ip: IpAddr,
        #[clap(long, short, default_value = "./config.json")]
        config_path: PathBuf,
        #[clap(long, short, action)]
        detach: bool,
        #[clap(long, short, default_value_t = 3100)]
        port: u16,
    },
    Models {
        #[clap(long, short, default_value = "./config.json")]
        config_path: PathBuf,
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
        alias_or_index: Option<String>,
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
