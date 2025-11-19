use std::path::Path;

use crate::{
    config::{Config, ModelConfig, ModelTypeConfig},
    models::Log,
};
use anyhow::{Result, anyhow};
use reqwest_sse::EventSource;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use utils_rs::{option::as_bool::AsBool, prelude::*};

pub fn load_sync(alias_or_index: String, port: u16, tail: bool, json: bool) -> Result<()> {
    Runtime::new()?.block_on(async { load(alias_or_index, port, tail, json).await })
}

pub async fn load(alias_or_index: String, port: u16, tail: bool, json: bool) -> Result<()> {
    let response = reqwest::Client::new()
        .post(format!(
            "http://localhost:{port}/herder/{alias_or_index}?tail={}&json={}",
            tail.as_str(),
            json.as_str()
        ))
        .send()
        .await?
        .error_for_status()?;

    if tail {
        let mut es = response.events().await.map_err(|err| anyhow!("{err}"))?;

        while let Some(Ok(event)) = es.next().await {
            println!("{}", event.data);
        }
    } else {
        let model_config = response.json::<ModelConfig>().await?;

        println!("{model_config:#?}");
    }

    Ok(())
}

pub fn logs_sync(alias_or_index: String, port: u16, tail: bool, json: bool) -> Result<()> {
    Runtime::new()?.block_on(async { logs(alias_or_index, port, tail, json).await })
}

pub async fn logs(alias_or_index: String, port: u16, tail: bool, json: bool) -> Result<()> {
    let response = reqwest::Client::new()
        .get(format!(
            "http://localhost:{port}/herder/{alias_or_index}/logs?tail={}&json={}",
            tail.as_str(),
            json.as_str()
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

    Ok(())
}

pub fn unload_sync(alias_or_index: String, port: u16) -> Result<()> {
    Runtime::new()?.block_on(async { unload(alias_or_index, port).await })
}

pub async fn unload(alias_or_index: String, port: u16) -> Result<()> {
    let response = reqwest::Client::new()
        .delete(format!("http://localhost:{port}/herder/{alias_or_index}"))
        .send()
        .await?
        .error_for_status()?
        .json::<Option<ModelConfig>>()
        .await?;

    println!("{response:#?}");

    Ok(())
}

pub fn loaded_sync(port: u16) -> Result<()> {
    Runtime::new()?.block_on(async { loaded(port).await })
}

pub async fn loaded(port: u16) -> Result<()> {
    let response = reqwest::Client::new()
        .get(format!("http://localhost:{port}/herder"))
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<ModelConfig>>()
        .await?;

    println!("{response:#?}");

    Ok(())
}

pub fn config_sync(config_path: &Path, alias_or_index: Option<String>) -> Result<()> {
    let config = Config::load(config_path)?;

    if let Some(alias_or_index) = alias_or_index {
        let model_config = config.get_model_config(alias_or_index)?;

        println!("{model_config:#?}")
    } else {
        println!("{:#?}", config.models)
    }

    Ok(())
}

pub fn list_sync(config_path: &Path) -> Result<()> {
    let config = Config::load(config_path)?;

    for (index, model_config) in config.models.into_iter().enumerate() {
        let (alias, type_info) = (
            model_config.alias(),
            match &model_config.config {
                ModelTypeConfig::LlamaCpp(x) => format!(
                    "LlamaCpp ('{}' - http://{}:{})",
                    x.hf_repo,
                    x.host.as_deref().unwrap_or("localhost"),
                    x.port
                ),
                ModelTypeConfig::External(x) => {
                    format!(
                        "External ('{}' - {})",
                        x.model().id,
                        x.unwrap_provider().base_url
                    )
                }
            },
        );

        let loads = model_config.loads.as_ref().map(|vec| {
            vec.iter()
                .map(|alias_or_index| format!("{alias_or_index:#?}"))
                .collect::<Vec<_>>()
                .join(", ")
        });

        let loads = loads
            .map(|loads| format!(" - Loads: {loads}"))
            .unwrap_or_default();

        let unloads = model_config.unloads.as_ref().map(|vec| {
            vec.iter()
                .map(|alias_or_index| format!("{alias_or_index:#?}"))
                .collect::<Vec<_>>()
                .join(", ")
        });

        let unloads = unloads
            .map(|unloads| format!(" - Unloads: {unloads}"))
            .unwrap_or_default();

        let is_default = model_config
            .is_default
            .as_bool()
            .map(|| " (default)".to_string())
            .unwrap_or_default();

        println!("[{index}]: Alias '{alias}'{is_default}{loads}{unloads} - {type_info}")
    }

    Ok(())
}
