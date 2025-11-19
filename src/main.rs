mod api;
mod cli;
mod commands;
mod config;
mod models;

use anyhow::{Context, Result, anyhow, bail};
use api::serve_sync;
use cli::{Cli, CliCommand, ServeArgs};
use config::Config;
use daemonize::Daemonize;
use schemars::schema_for;
use std::{
    fs::{File, exists},
    net::SocketAddr,
    path::{Path, PathBuf},
};
use utils_rs::prelude::{FindWalkingBack, ResolveEnvParts};

const DEFAULT_PATH: &str = "~/.config/hrdr/hrdr.json";

fn resolve_config_path(provided_path: Option<impl AsRef<Path>>) -> Result<PathBuf> {
    let path = if let Some(path) = provided_path {
        path.as_ref().resolve_env_parts()
    } else {
        PathBuf::from(DEFAULT_PATH)
            .file_name()
            .ok_or(anyhow!("No file name in default path"))
            .and_then(|file_name| {
                file_name
                    .find_walking_back()
                    .context("Failed walking back to look for file name")
            })?
            .unwrap_or_else(|| DEFAULT_PATH.resolve_env_parts())
    };

    if !exists(&path)? {
        bail!("{path:?} does not exist :(")
    }

    Ok(path)
}

fn main() -> Result<()> {
    match Cli::command() {
        CliCommand::Schema => {
            println!(
                "{}",
                serde_json::to_string_pretty(&schema_for!(Config)).unwrap()
            );
        }
        CliCommand::Start(ServeArgs {
            ip,
            config_path,
            port,
        })
        | CliCommand::Serve {
            args:
                ServeArgs {
                    ip,
                    config_path,
                    port,
                },
            detach: true,
        } => {
            let address = SocketAddr::new(ip, port);

            let config_path = resolve_config_path(config_path)?;
            let config = Config::load(&config_path)?;

            let stdout = File::create(config.detach.out_file_path)?;
            let stderr = File::create(config.detach.err_file_path)?;

            Daemonize::new()
                .pid_file(config.detach.pid_file_path)
                .stdout(stdout)
                .stderr(stderr)
                .working_directory(config.detach.working_dir)
                .start()?;

            serve_sync(&address, config_path)?;
        }
        CliCommand::Serve {
            args:
                ServeArgs {
                    ip,
                    config_path,
                    port,
                },
            detach: false,
        } => {
            let address = SocketAddr::new(ip, port);

            let config_path = resolve_config_path(config_path)?;
            Config::load(&config_path)?;

            serve_sync(&address, config_path)?;
        }
        CliCommand::Stop { config_path } => {
            let config_path = resolve_config_path(config_path)?;
            let config = Config::load(&config_path)?;

            let pid = std::fs::read_to_string(config.detach.pid_file_path)?
                .trim()
                .to_string();

            let output = std::process::Command::new("kill").arg(pid).output()?;

            let std = String::from_utf8(output.stdout)?;
            let err = String::from_utf8(output.stderr)?;

            if !std.is_empty() {
                println!("{std}");
            }

            if !err.is_empty() {
                println!("{err}")
            }
        }
        CliCommand::Models {
            config_path,
            command,
        } => {
            let config_path = resolve_config_path(config_path)?;

            match command.unwrap_or(cli::ModelCommand::List) {
                cli::ModelCommand::List => {
                    commands::list_sync(&config_path)?;
                }
                cli::ModelCommand::Config { alias_or_index } => {
                    commands::config_sync(&config_path, alias_or_index)?;
                }
                cli::ModelCommand::Load {
                    port,
                    tail,
                    json,
                    alias_or_index,
                } => {
                    commands::load_sync(alias_or_index, port, tail, json)?;
                }
                cli::ModelCommand::Loaded { port } => {
                    commands::loaded_sync(port)?;
                }
                cli::ModelCommand::Unload {
                    port,
                    alias_or_index,
                } => {
                    commands::unload_sync(alias_or_index, port)?;
                }
                cli::ModelCommand::Logs {
                    port,
                    tail,
                    json,
                    alias_or_index,
                } => {
                    commands::logs_sync(alias_or_index, port, tail, json)?;
                }
            }
        }
    }

    Ok(())
}
