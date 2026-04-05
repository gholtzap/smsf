mod config;
mod context;
mod db;
mod nf_client;
mod sbi;
mod sms;
mod tls;
mod utils;

use crate::config::Config;
use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::amf::AmfClient;
use crate::nf_client::nrf::NrfClient;
use crate::nf_client::udm::UdmClient;
use crate::sbi::server::{create_router, AppState};
use crate::sms::delivery::SmsDeliveryService;
use crate::sms::retry::SmsRetryService;
use crate::sms::status_report::StatusReportService;
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

    match db.load_all_ue_contexts().await {
        Ok(contexts) => {
            let count = contexts.len();
            context_store.load_contexts(contexts);
            info!("Restored {} UE SMS contexts from database", count);
        }
        Err(e) => {
            error!("Failed to load UE contexts from database: {}", e);
        }
    }
    let nrf_client = Arc::new(NrfClient::new(
        config.nrf_uri.clone(),
        config.nf_instance_id.clone(),
        Some(&config.tls),
    )?);

    let profile = nrf_client.build_smsf_profile(&config.smsf_host, config.sbi_bind_port);
    nrf_client.register(profile).await?;

    let udm_client = UdmClient::with_nrf(nrf_client.clone(), Some(config.udm_uri.clone()), Some(&config.tls));

    let amf_client = AmfClient::with_nrf(nrf_client.clone(), Some(&config.tls))?;

    let nrf_client_clone = nrf_client.clone();
    let smsf_host = config.smsf_host.clone();
    let smsf_port = config.sbi_bind_port;
    nrf_client_clone
        .start_heartbeat_task(smsf_host, smsf_port)
        .await;

    let delivery_service = Arc::new(SmsDeliveryService::new(
        context_store.clone(),
        db.clone(),
        amf_client.clone(),
        config.retry.default_validity_period_secs,
    ));

    let status_report_service = Arc::new(StatusReportService::new(
        db.clone(),
        delivery_service.clone(),
    ));

    let retry_service = Arc::new(SmsRetryService::new(
        db.clone(),
        delivery_service.clone(),
        config.retry.clone(),
    ));

    let retry_service_clone = retry_service.clone();
    tokio::spawn(async move {
        retry_service_clone.start().await;
    });

    let app_state = Arc::new(AppState {
        context_store,
        db,
        amf_client,
        udm_client,
        delivery_service,
        status_report_service,
        oauth2_config: config.oauth2.clone(),
    });

    let app = create_router(app_state);

    let bind_addr = format!("{}:{}", config.sbi_bind_addr, config.sbi_bind_port);

    let nrf_for_shutdown = nrf_client.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        info!("Shutting down gracefully...");
        if let Err(e) = nrf_for_shutdown.deregister().await {
            error!("Failed to deregister from NRF: {}", e);
        }
        std::process::exit(0);
    });

    if config.tls.enabled {
        let tls_config = crate::tls::load_tls_config(&config.tls).await?;
        info!("SMSF listening on {} with TLS enabled", bind_addr);
        axum_server::bind_rustls(bind_addr.parse()?, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        info!("SMSF listening on {} (no TLS)", bind_addr);
        axum::serve(listener, app).await?;
    }

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
