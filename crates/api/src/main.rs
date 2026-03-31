use std::net::SocketAddr;

use provider_api::{build_app, config::AppConfig, state::AppState};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env();
    let state = AppState::new(config.clone()).await;
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;

    info!(
        port = config.port,
        swagger = "/swagger-ui",
        "starting providers api"
    );

    axum::serve(listener, build_app(state)).await?;
    Ok(())
}
