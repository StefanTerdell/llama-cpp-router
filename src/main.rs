#![recursion_limit = "2048"]

mod api;
mod cli;
mod config;
mod models;
mod reqwest_ext;

use anyhow::{Result, anyhow};
use api::serve;
use cli::{Cli, CliCommand};
use config::{Config, ModelConfig};
use models::Log;
use reqwest_sse::EventSource;
use schemars::schema_for;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::command() {
        CliCommand::Schema => {
            println!(
                "{}",
                serde_json::to_string_pretty(&schema_for!(Config)).unwrap()
            );
        }
        CliCommand::Serve {
            port,
            config_path,
            detach,
        } => {
            if detach {
                todo!("Detach not implemented")
            }

            serve(port, config_path).await?;
        }
        CliCommand::Models {
            config_path,
            command,
        } => {
            match command.unwrap_or(cli::ModelCommand::List) {
                cli::ModelCommand::List => {
                    let config = Config::load(&config_path)?;

                    for (index, model_config) in config.models.into_iter().enumerate() {
                        let (alias, loads, unloads, type_info) = (
                            model_config.alias(),
                            &model_config.loads,
                            &model_config.unloads,
                            match &model_config.config {
                                config::ModelTypeConfig::LlamaCpp(x) => format!(
                                    "LlamaCpp ('{}' - http://{}:{})",
                                    x.hf_repo,
                                    x.host.as_deref().unwrap_or("localhost"),
                                    x.port
                                ),
                                config::ModelTypeConfig::External(x) => {
                                    format!("External ('{}' - {})", x.name, x.base_url)
                                }
                            },
                        );

                        let loads = loads.as_ref().map(|vec| {
                            vec.iter()
                                .map(|alias_or_index| format!("{alias_or_index:#?}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        });

                        let loads = loads
                            .map(|loads| format!(" - Loads: {loads}"))
                            .unwrap_or_default();

                        let unloads = unloads.as_ref().map(|vec| {
                            vec.iter()
                                .map(|alias_or_index| format!("{alias_or_index:#?}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        });

                        let unloads = unloads
                            .map(|unloads| format!(" - Unloads: {unloads}"))
                            .unwrap_or_default();

                        println!("[{index}]: Alias: '{alias}'{loads}{unloads} - {type_info}")
                    }
                }
                cli::ModelCommand::Config { alias_or_index } => {
                    let config = Config::load(&config_path)?;

                    if let Some(alias_or_index) = alias_or_index {
                        let model_config = config.get_model_config(alias_or_index)?;

                        println!("{model_config:#?}")
                    } else {
                        println!("{:#?}", config.models)
                    }
                }
                cli::ModelCommand::Load {
                    port,
                    tail,
                    json,
                    alias_or_index,
                } => {
                    let response = reqwest::Client::new()
                    .post(format!(
                        "http://localhost:{port}/hurrdurr/models/{alias_or_index}?tail={}&json={}", if tail { "true" } else { "false" }, if json { "true" } else { "false" }
                    ))
                    .send()
                    .await?.error_for_status()?;

                    if tail {
                        let mut es = response.events().await.map_err(|err| anyhow!("{err}"))?;

                        while let Some(Ok(event)) = es.next().await {
                            println!("{}", event.data);
                        }
                    } else {
                        let model_config = response.json::<ModelConfig>().await?;

                        println!("{model_config:#?}");
                    }
                }
                cli::ModelCommand::Loaded {
                    port,
                    alias_or_index: _,
                } => {
                    let response = reqwest::Client::new()
                        .get(format!("http://localhost:{port}/hurrdurr/models"))
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<Vec<ModelConfig>>()
                        .await?;

                    println!("{response:#?}")
                }
                cli::ModelCommand::Unload {
                    port,
                    alias_or_index,
                } => {
                    let response = reqwest::Client::new()
                        .delete(format!(
                            "http://localhost:{port}/hurrdurr/models/{alias_or_index}"
                        ))
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<Option<ModelConfig>>()
                        .await?;

                    println!("{response:#?}")
                }
                cli::ModelCommand::Logs {
                    port,
                    tail,
                    json,
                    alias_or_index,
                } => {
                    let response = reqwest::Client::new()
                    .get(format!(
                        "http://localhost:{port}/hurrdurr/models/{alias_or_index}/logs?tail={}&json={}", if tail { "true" } else { "false" }, if json { "true" } else { "false" }
                        ))
                        .send()
                        .await?
                        .error_for_status()?;

                    if tail {
                        let mut es = response.events().await.map_err(|err| anyhow!("{err}"))?;

                        while let Some(Ok(event)) = es.next().await {
                            println!("{}", event.data);
                        }
                    } else if json {
                        let logs = response.json::<Vec<Log>>().await?;

                        println!("{logs:#?}");
                    } else {
                        let text = response.text().await?;

                        println!("{text}")
                    }
                }
            }
        }
    }

    Ok(())
}
