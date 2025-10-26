use crate::config::{AliasOrIndex, Config, LlamaCppModelConfig, ModelConfig, ModelTypeConfig};
use anyhow::Result;
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, process::Stdio, sync::Arc};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    sync::{Mutex, broadcast},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedMessage {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl TimestampedMessage {
    pub fn new(message: impl Display) -> Self {
        Self {
            timestamp: Utc::now(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stream")]
pub enum Log {
    StdOut(TimestampedMessage),
    StdErr(TimestampedMessage),
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (stream, TimestampedMessage { timestamp, message }) = match &self {
            Log::StdOut(timestamped_message) => ("StdOut", timestamped_message),
            Log::StdErr(timestamped_message) => ("StdErr", timestamped_message),
        };

        f.write_fmt(format_args!("{stream} - {timestamp} - {message}"))
    }
}

struct Spawned {
    io_sender: broadcast::Sender<Log>,
    logs: Arc<Mutex<Vec<Log>>>,
    _child: Child, // Unused, but will be killed on drop
}

pub struct LogsAndTailReceiver {
    pub tail_receiver: broadcast::Receiver<Log>,
    pub logs: Vec<Log>,
}

impl LogsAndTailReceiver {
    async fn from_ref(spawned: &Spawned) -> Self {
        LogsAndTailReceiver {
            logs: spawned.logs.lock().await.clone(),
            tail_receiver: spawned.io_sender.subscribe(),
        }
    }
}

struct LoadedModel {
    config: ModelConfig,
    spawned: Option<Spawned>,
}

#[derive(Default, Clone)]
pub struct Models {
    loaded: Arc<Mutex<HashMap<String, LoadedModel>>>,
}

// #[derive(Debug, thiserror::Error)]
// pub enum ModelsError {
//     #[error(transparent)]
//     GetModelConfig(#[from] ConfigError),
//     #[error(transparent)]
//     Io(#[from] std::io::Error),
// }

impl Models {
    pub async fn get_logs_and_receiver(&self, alias: &str) -> Option<LogsAndTailReceiver> {
        let loaded = self.loaded.lock().await;
        let spawned = loaded.get(alias)?.spawned.as_ref()?;

        Some(LogsAndTailReceiver::from_ref(spawned).await)
    }

    pub async fn get_loaded_config(&self, alias: &str) -> Option<ModelConfig> {
        self.loaded
            .lock()
            .await
            .get(alias)
            .map(|m| m.config.clone())
    }

    pub async fn get_loaded_configs(&self) -> Vec<ModelConfig> {
        self.loaded
            .lock()
            .await
            .values()
            .map(|m| m.config.clone())
            .collect()
    }

    pub async fn unload(&self, alias: &str) -> Result<Option<ModelConfig>> {
        let mut loaded_models = self.loaded.lock().await;

        Ok(loaded_models.remove(alias).map(|m| m.config))
    }

    #[async_recursion]
    pub async fn load(
        &self,
        config: &Config,
        alias_or_index: impl Into<AliasOrIndex> + Send + 'async_recursion,
    ) -> Result<(ModelConfig, Option<LogsAndTailReceiver>)> {
        let mut loaded_models = self.loaded.lock().await;

        let model_config = config.get_model_config(alias_or_index)?;
        let alias = model_config.alias();

        let spawned = match &model_config.config {
            ModelTypeConfig::LlamaCpp(LlamaCppModelConfig {
                hf_repo,
                port,
                alias,
                api_key,
                host,
                additional_properties,
            }) => {
                let mut command = Command::new("llama-server");

                command
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .arg("--alias")
                    .arg(alias)
                    .arg("--hf-repo")
                    .arg(hf_repo)
                    .arg("--port")
                    .arg(port.to_string());

                if let Some(api_key) = api_key {
                    command.arg("--api-key").arg(api_key.expose_ref());
                }

                if let Some(host) = host {
                    command.arg("--host").arg(host);
                }

                for (key, value) in additional_properties {
                    if value.as_null().is_some() || value.as_bool().is_some_and(|b| !b) {
                        continue;
                    }

                    command.arg(format!("--{key}"));

                    if value.as_bool().is_none() {
                        command.arg(serde_json::to_string(&value).unwrap());
                    }
                }

                command.kill_on_drop(true);

                let mut child = command.spawn()?;

                let mut stdout = child.stdout.take().unwrap();

                let mut stderr = child.stderr.take().unwrap();

                let (sender, mut receiver) = broadcast::channel(1024 * 1024);

                tokio::spawn({
                    let sender = sender.clone();

                    async move {
                        let mut buf = [0u8; 1024];

                        loop {
                            let Ok(num_read) = stdout.read(&mut buf).await else {
                                break;
                            };

                            if num_read == 0 {
                                break;
                            }

                            if let Ok(string) = String::from_utf8(buf[0..num_read].to_vec()) {
                                if sender
                                    .send(Log::StdOut(TimestampedMessage::new(string)))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    }
                });

                tokio::spawn({
                    let sender = sender.clone();

                    async move {
                        let mut buf = [0u8; 1024];

                        loop {
                            let Ok(num_read) = stderr.read(&mut buf).await else {
                                break;
                            };

                            if num_read == 0 {
                                break;
                            }

                            if let Ok(string) = String::from_utf8(buf[0..num_read].to_vec()) {
                                if sender
                                    .send(Log::StdErr(TimestampedMessage::new(string)))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    }
                });

                let logs: Arc<Mutex<Vec<Log>>> = Default::default();

                tokio::spawn({
                    let logs = logs.clone();

                    async move {
                        while let Ok(x) = receiver.recv().await {
                            logs.lock().await.push(x)
                        }
                    }
                });

                Some(Spawned {
                    io_sender: sender,
                    _child: child,
                    logs,
                })
            }
            ModelTypeConfig::External(_) => None,
        };

        let logs_and_tail_receiver = if let Some(spawned) = spawned.as_ref() {
            Some(LogsAndTailReceiver::from_ref(spawned).await)
        } else {
            None
        };

        loaded_models.insert(
            model_config.alias().to_string(),
            LoadedModel {
                config: model_config.clone(),
                spawned,
            },
        );

        if let Some(to_unload) = &model_config.unloads {
            for alias_or_index in to_unload {
                let model_config = config.get_model_config(alias_or_index)?;

                if model_config.alias() == alias {
                    continue;
                }

                loaded_models.remove(model_config.alias());
            }
        }

        drop(loaded_models);

        if let Some(to_load) = &model_config.loads {
            for alias_or_index in to_load {
                if model_config.alias() == alias {
                    continue;
                }

                self.load(config, alias_or_index).await?;
            }
        }

        Ok((model_config.clone(), logs_and_tail_receiver))
    }
}
