use polymarket_client_sdk::gamma::Client;
use polymarket_client_sdk::gamma::types::request::{
    EventBySlugRequest, EventsRequest, MarketBySlugRequest, MarketsRequest, SearchRequest,
};
use polymarket_client_sdk::gamma::types::response::{Event, Market, SearchResults};

/// Wrapper around the Polymarket Gamma API client.
#[derive(Clone)]
pub struct GammaClient {
    client: Client,
}

impl GammaClient {
    /// Create a new Gamma API client with the default endpoint.
    pub fn new() -> Self {
        Self {
            client: Client::default(),
        }
    }

    /// Search markets and events by a query string.
    pub async fn search(&self, query: &str) -> Result<SearchResults, String> {
        let request = SearchRequest::builder().q(query).build();
        self.client
            .search(&request)
            .await
            .map_err(|e| format!("Search failed: {e}"))
    }

    /// Get a market by its slug.
    pub async fn get_market_by_slug(&self, slug: &str) -> Result<Market, String> {
        let request = MarketBySlugRequest::builder().slug(slug).build();
        self.client
            .market_by_slug(&request)
            .await
            .map_err(|e| format!("Failed to fetch market '{slug}': {e}"))
    }

    /// Get an event by its slug.
    pub async fn get_event_by_slug(&self, slug: &str) -> Result<Event, String> {
        let request = EventBySlugRequest::builder().slug(slug).build();
        self.client
            .event_by_slug(&request)
            .await
            .map_err(|e| format!("Failed to fetch event '{slug}': {e}"))
    }

    /// List active events ordered by volume (descending).
    pub async fn list_active_events(&self, limit: u32) -> Result<Vec<Event>, String> {
        let request = EventsRequest::builder()
            .active(true)
            .limit(limit as i32)
            .order(vec!["volume".to_owned()])
            .ascending(false)
            .build();
        self.client
            .events(&request)
            .await
            .map_err(|e| format!("Failed to fetch events: {e}"))
    }

    /// List active markets ordered by volume.
    pub async fn list_active_markets(&self, limit: u32) -> Result<Vec<Market>, String> {
        let request = MarketsRequest::builder()
            .closed(false)
            .limit(limit as i32)
            .build();
        self.client
            .markets(&request)
            .await
            .map_err(|e| format!("Failed to fetch markets: {e}"))
    }
}
