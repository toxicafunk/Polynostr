use nostr_sdk::prelude::*;
use std::env;

/// Bot configuration loaded from environment variables.
pub struct Config {
    pub keys: Keys,
    pub relays: Vec<String>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required:
    ///   - `NOSTR_SECRET_KEY`: hex or bech32 nostr secret key
    ///
    /// Optional:
    ///   - `NOSTR_RELAYS`: comma-separated relay URLs (defaults to popular relays)
    pub fn load() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let secret_key = env::var("NOSTR_SECRET_KEY").map_err(|e| {
            nostr_sdk::client::Error::Signer(SignerError::backend(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("NOSTR_SECRET_KEY environment variable is required: {}", e),
            )))
        })?;

        let keys = Keys::parse(&secret_key)?;

        let relays = match env::var("NOSTR_RELAYS") {
            Ok(val) => val
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => vec![
                String::from("wss://relay.damus.io"),
                String::from("wss://nos.lol"),
                String::from("wss://relay.nostr.band"),
            ],
        };

        Ok(Self { keys, relays })
    }
}
