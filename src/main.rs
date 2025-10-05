mod api;
mod cli;
mod config;
mod models;

use anyhow::*;
use api::serve;
use cli::{Cli, CliCommand};
use config::Config;
use schemars::schema_for;

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::command() {
        CliCommand::Schema => {
            println!(
                "{}",
                serde_json::to_string_pretty(&schema_for!(Config)).unwrap()
            );
        }
        CliCommand::Serve { config_file } => {
            serve(Config::load(&config_file)?).await?;
        }
    }

    Ok(())
}
