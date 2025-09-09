mod config;
use crate::config::AppConfig;
use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn root() -> &'static str {
    "api gateway: okay"
}
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = AppConfig::load()?;
    tracing::info!(?cfg, "loaded config");
    let addr = cfg.addr();

    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(health));

    // Axum 0.8 style: bind a TcpListener, then axum::serve
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
