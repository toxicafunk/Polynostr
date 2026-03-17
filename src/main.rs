mod bot;
mod commands;
mod config;
mod format;
mod polymarket;

use config::Config;
use nostr_sdk::prelude::*;
use polymarket::gamma::GammaClient;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Install default Rustls crypto provider (required for Rustls 0.23+)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok(); // Ignore error if already installed

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = Config::load()?;

    let bech32_pubkey = config.keys.public_key().to_bech32()?;
    info!(pubkey = %bech32_pubkey, "Polynostr bot starting");

    // Create nostr client
    let client = Client::builder().signer(config.keys.clone()).build();

    // Add relays
    for relay in &config.relays {
        info!(relay = %relay, "Adding relay");
        if let Err(e) = client.add_relay(relay).await {
            tracing::warn!(relay = %relay, error = %e, "Failed to add relay");
        }
    }

    // Connect to relays
    info!("Connecting to relays...");
    client.connect().await;
    info!("Connected to relays");

    // Create Polymarket clients
    let gamma = GammaClient::new();

    info!(
        pubkey = %bech32_pubkey,
        "Bot is online! Send a DM or mention this pubkey to interact."
    );

    // Run the bot event loop
    bot::run(&client, &gamma).await?;

    Ok(())
}
