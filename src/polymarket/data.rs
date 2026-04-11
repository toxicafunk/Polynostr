use std::str::FromStr;

use polymarket_client_sdk::data::Client;
use polymarket_client_sdk::data::types::request::{
    OpenInterestRequest, PositionsRequest, TradedRequest, TradesRequest, ValueRequest,
};
use polymarket_client_sdk::data::types::response::{OpenInterest, Position, Trade, Traded, Value};
use polymarket_client_sdk::data::types::{PositionSortBy, SortDirection};
use polymarket_client_sdk::types::Address;

/// Wrapper around the Polymarket Data API client.
#[derive(Clone)]
pub struct DataClient {
    client: Client,
}

impl DataClient {
    /// Create a new Data API client with the default endpoint.
    pub fn new() -> Self {
        Self {
            client: Client::default(),
        }
    }

    /// Normalize and validate a wallet address.
    pub fn normalize_address(address: &str) -> Result<String, String> {
        parse_address(address).map(|addr| addr.to_string())
    }

    /// Get open interest data for all markets.
    pub async fn get_open_interest(&self) -> Result<Vec<OpenInterest>, String> {
        self.client
            .open_interest(&OpenInterestRequest::default())
            .await
            .map_err(|e| format!("Failed to fetch open interest: {e}"))
    }

    /// Get open positions for a wallet address.
    pub async fn get_positions(&self, address: &str, limit: i32) -> Result<Vec<Position>, String> {
        let user = parse_address(address)?;
        let request = PositionsRequest::builder()
            .user(user)
            .limit(limit)
            .map_err(|e| format!("Invalid positions limit: {e}"))?
            .sort_by(PositionSortBy::CashPnl)
            .sort_direction(SortDirection::Desc)
            .build();

        self.client
            .positions(&request)
            .await
            .map_err(|e| format!("Failed to fetch positions: {e}"))
    }

    /// Get recent trades for a wallet address.
    pub async fn get_trades(&self, address: &str, limit: i32) -> Result<Vec<Trade>, String> {
        let user = parse_address(address)?;
        let request = TradesRequest::builder()
            .user(user)
            .limit(limit)
            .map_err(|e| format!("Invalid trades limit: {e}"))?
            .build();

        self.client
            .trades(&request)
            .await
            .map_err(|e| format!("Failed to fetch trades: {e}"))
    }

    /// Get the total position value for a wallet address.
    pub async fn get_total_value(&self, address: &str) -> Result<Option<Value>, String> {
        let user = parse_address(address)?;
        let request = ValueRequest::builder().user(user).build();

        self.client
            .value(&request)
            .await
            .map(|mut values| values.drain(..).next())
            .map_err(|e| format!("Failed to fetch portfolio value: {e}"))
    }

    /// Get the number of unique markets traded by a wallet address.
    pub async fn get_traded_markets(&self, address: &str) -> Result<Traded, String> {
        let user = parse_address(address)?;
        let request = TradedRequest::builder().user(user).build();

        self.client
            .traded(&request)
            .await
            .map_err(|e| format!("Failed to fetch traded market count: {e}"))
    }
}

fn parse_address(address: &str) -> Result<Address, String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return Err("Wallet address is required.".to_owned());
    }

    let normalized = if trimmed.starts_with("0x") {
        trimmed.to_owned()
    } else {
        format!("0x{trimmed}")
    };

    Address::from_str(&normalized).map_err(|_| {
        format!(
            "Invalid wallet address: '{trimmed}'. Expected a 20-byte hex address (with or without 0x prefix)."
        )
    })
}
