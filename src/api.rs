mod catchall;
mod models;
mod open_ai;
mod result;
mod state;

use anyhow::Result;
use axum::Router;
use std::{net::SocketAddr, path::PathBuf};
use tokio::{net::TcpListener, runtime::Runtime};

pub fn serve_sync(address: &SocketAddr, config_path: PathBuf) -> Result<()> {
    Runtime::new()?.block_on(async { serve(address, config_path).await })
}

pub async fn serve(address: &SocketAddr, config_path: PathBuf) -> Result<()> {
    let state = state::ApiState::init(config_path).await?;

    let api = Router::new()
        .nest("/herder", models::router())
        .nest("/v1", open_ai::router())
        .route("/{*path}", catchall::handler())
        .with_state(state);

    let listener = TcpListener::bind(address).await?;

    println!("herder listening on {address}");

    axum::serve(listener, api).await?;

    Ok(())
}
