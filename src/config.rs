use nostr_sdk::prelude::*;
use std::env;

/// Bot configuration loaded from environment variables.
pub struct Config {
    pub keys: Keys,
    pub relays: Vec<String>,
    pub alert_stream_enabled: bool,
    pub alert_poll_interval_seconds: u64,
    pub alert_reconnect_backoff_initial_seconds: u64,
    pub alert_reconnect_backoff_max_seconds: u64,
    pub alert_max_per_user: usize,
    pub alert_cooldown_seconds: u64,
    pub alert_hysteresis_bps: u32,
    pub alert_notifications_per_minute: usize,
    pub alert_db_path: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required:
    ///   - `NOSTR_SECRET_KEY`: hex or bech32 nostr secret key
    ///
    /// Optional:
    ///   - `NOSTR_RELAYS`: comma-separated relay URLs (defaults to popular relays)
    pub fn load() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let _ = dotenvy::dotenv();

        let secret_key = env::var("NOSTR_SECRET_KEY").map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("NOSTR_SECRET_KEY environment variable is required: {e}"),
            )
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

        let alert_stream_enabled = env::var("ALERT_STREAM_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(true);
        let alert_poll_interval_seconds = parse_u64("ALERT_POLL_INTERVAL_SECONDS", 15)?;
        let alert_reconnect_backoff_initial_seconds =
            parse_u64("ALERT_RECONNECT_BACKOFF_INITIAL_SECONDS", 2)?;
        let alert_reconnect_backoff_max_seconds =
            parse_u64("ALERT_RECONNECT_BACKOFF_MAX_SECONDS", 60)?;
        let alert_max_per_user = parse_usize("ALERT_MAX_PER_USER", 25)?;
        let alert_cooldown_seconds = parse_u64("ALERT_COOLDOWN_SECONDS", 120)?;
        let alert_hysteresis_bps = parse_u32("ALERT_HYSTERESIS_BPS", 50)?;
        let alert_notifications_per_minute = parse_usize("ALERT_NOTIFICATIONS_PER_MINUTE", 10)?;
        let alert_db_path =
            env::var("ALERT_DB_PATH").unwrap_or_else(|_| "alerts.sqlite3".to_owned());

        if alert_poll_interval_seconds < 3 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "ALERT_POLL_INTERVAL_SECONDS must be >= 3",
            )
            .into());
        }
        if alert_reconnect_backoff_initial_seconds == 0
            || alert_reconnect_backoff_max_seconds < alert_reconnect_backoff_initial_seconds
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Reconnect backoff values are invalid",
            )
            .into());
        }

        Ok(Self {
            keys,
            relays,
            alert_stream_enabled,
            alert_poll_interval_seconds,
            alert_reconnect_backoff_initial_seconds,
            alert_reconnect_backoff_max_seconds,
            alert_max_per_user,
            alert_cooldown_seconds,
            alert_hysteresis_bps,
            alert_notifications_per_minute,
            alert_db_path,
        })
    }
}

fn parse_u64(
    key: &str,
    default_value: u64,
) -> std::result::Result<u64, Box<dyn std::error::Error>> {
    match env::var(key) {
        Ok(raw) => Ok(raw.parse::<u64>().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{key} must be a positive integer: {e}"),
            )
        })?),
        Err(_) => Ok(default_value),
    }
}

fn parse_u32(
    key: &str,
    default_value: u32,
) -> std::result::Result<u32, Box<dyn std::error::Error>> {
    match env::var(key) {
        Ok(raw) => Ok(raw.parse::<u32>().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{key} must be a positive integer: {e}"),
            )
        })?),
        Err(_) => Ok(default_value),
    }
}

fn parse_usize(
    key: &str,
    default_value: usize,
) -> std::result::Result<usize, Box<dyn std::error::Error>> {
    match env::var(key) {
        Ok(raw) => Ok(raw.parse::<usize>().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{key} must be a positive integer: {e}"),
            )
        })?),
        Err(_) => Ok(default_value),
    }
}
