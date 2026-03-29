mod alerts;
mod bot;
mod commands;
mod config;
mod format;
mod polymarket;

use std::sync::Arc;

use alerts::{
    AlertDelivery, AlertManager, AlertManagerConfig, AlertNotifier, MarketUpdateSource,
    PollingMarketUpdateSource, SqliteAlertRepository, WsMarketUpdateSource,
};
use config::Config;
use nostr_sdk::prelude::*;
use polymarket::gamma::GammaClient;
use tracing::info;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::load()?;

    let version = env!("CARGO_PKG_VERSION");
    let commit_hash = option_env!("GIT_COMMIT_SHORT_HASH").unwrap_or("unknown");

    info!(
        version = %version,
        commit_hash = %commit_hash,
        "Polynostr build info"
    );

    let bech32_pubkey = config.keys.public_key().to_bech32()?;
    info!(pubkey = %bech32_pubkey, "Polynostr bot starting");

    let client = Arc::new(Client::builder().signer(config.keys.clone()).build());

    for relay in &config.relays {
        info!(relay = %relay, "Adding relay");
        if let Err(e) = client.add_relay(relay).await {
            tracing::warn!(relay = %relay, error = %e, "Failed to add relay");
        }
    }

    info!("Connecting to relays...");
    client.connect().await;
    info!("Connected to relays");

    let gamma = Arc::new(GammaClient::new());

    let repo = Arc::new(SqliteAlertRepository::new(&config.alert_db_path)?);
    let polling_source =
        PollingMarketUpdateSource::new(gamma.clone(), config.alert_poll_interval_seconds);
    let source: Arc<dyn MarketUpdateSource> = if config.alert_stream_enabled {
        Arc::new(WsMarketUpdateSource::new(polling_source))
    } else {
        Arc::new(polling_source)
    };
    let notifier: Arc<dyn AlertDelivery> = Arc::new(AlertNotifier::new(client.clone()));

    let alert_manager = Arc::new(AlertManager::new(
        repo,
        source,
        notifier,
        AlertManagerConfig {
            max_alerts_per_user: config.alert_max_per_user,
            cooldown_seconds: config.alert_cooldown_seconds,
            hysteresis_bps: config.alert_hysteresis_bps,
            notifications_per_minute: config.alert_notifications_per_minute,
            refresh_seconds: config.alert_poll_interval_seconds,
            stream_enabled: config.alert_stream_enabled,
            reconnect_backoff_initial_seconds: config.alert_reconnect_backoff_initial_seconds,
            reconnect_backoff_max_seconds: config.alert_reconnect_backoff_max_seconds,
        },
    ));

    let manager_task = {
        let alert_manager = alert_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = alert_manager.run().await {
                tracing::error!(error = %e, "Alert manager stopped with error");
            }
        })
    };

    info!(
        pubkey = %bech32_pubkey,
        "Bot is online! Send a DM or mention this pubkey to interact."
    );

    let bot_result = bot::run(client.clone(), gamma, alert_manager).await;

    manager_task.abort();
    let _ = manager_task.await;

    bot_result.map_err(Box::<dyn std::error::Error>::from)
}
