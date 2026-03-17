use polymarket_client_sdk::data::types::request::OpenInterestRequest;
use polymarket_client_sdk::data::types::response::OpenInterest;
use polymarket_client_sdk::data::Client;

/// Wrapper around the Polymarket Data API client.
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

    /// Get open interest data for all markets.
    pub async fn get_open_interest(&self) -> Result<Vec<OpenInterest>, String> {
        self.client
            .open_interest(&OpenInterestRequest::default())
            .await
            .map_err(|e| format!("Failed to fetch open interest: {e}"))
    }
}
