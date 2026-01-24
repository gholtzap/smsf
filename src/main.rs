mod config;
mod context;
mod db;
mod nf_client;
mod sbi;
mod sms;
mod utils;

use crate::config::Config;
use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::amf::AmfClient;
use crate::nf_client::nrf::NrfClient;
use crate::sbi::server::{create_router, AppState};
use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting SMSF...");

    let config = match std::env::var("CONFIG_FILE") {
        Ok(path) => Config::from_file(&path)?,
        Err(_) => Config::from_env()?,
    };

    info!(
        "SMSF binding to {}:{}",
        config.sbi_bind_addr, config.sbi_bind_port
    );

    let db = Database::new(&config.mongodb_uri).await?;
    info!("Database connected");

    let context_store = UeSmsContextStore::new();
    let amf_client = AmfClient::new();

    let nrf_client = Arc::new(NrfClient::new(
        config.nrf_uri.clone(),
        config.nf_instance_id.clone(),
    ));

    let profile = nrf_client.build_smsf_profile(&config.smsf_host, config.sbi_bind_port);
    nrf_client.register(profile).await?;

    let nrf_client_clone = nrf_client.clone();
    let smsf_host = config.smsf_host.clone();
    let smsf_port = config.sbi_bind_port;
    nrf_client_clone
        .start_heartbeat_task(smsf_host, smsf_port)
        .await;

    let app_state = Arc::new(AppState {
        context_store,
        db,
        amf_client,
    });

    let app = create_router(app_state);

    let bind_addr = format!("{}:{}", config.sbi_bind_addr, config.sbi_bind_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("SMSF listening on {}", bind_addr);

    let nrf_for_shutdown = nrf_client.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        info!("Shutting down gracefully...");
        if let Err(e) = nrf_for_shutdown.deregister().await {
            error!("Failed to deregister from NRF: {}", e);
        }
        std::process::exit(0);
    });

    axum::serve(listener, app).await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}
