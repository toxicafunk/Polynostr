use std::cmp::Ordering;

use chrono::Utc;
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

    /// List active events ranked by 24h volume (descending), with total volume fallback.
    pub async fn list_active_events(&self, limit: u32) -> Result<Vec<Event>, String> {
        let fetch_limit = ((limit as i32) * 5).min(200);
        let request = EventsRequest::builder()
            .active(true)
            .closed(false)
            .archived(false)
            .limit(fetch_limit)
            .order(vec!["volume".to_owned()])
            .ascending(false)
            .build();

        let mut events = self
            .client
            .events(&request)
            .await
            .map_err(|e| format!("Failed to fetch events: {e}"))?;

        let now = Utc::now();
        events.retain(|event| {
            let is_closed = event.closed.unwrap_or(false);
            let is_archived = event.archived.unwrap_or(false);
            let is_expired = event
                .end_date
                .as_ref()
                .map(|end_date| end_date < &now)
                .unwrap_or(false);

            !is_closed && !is_archived && !is_expired
        });

        events.sort_by(|a, b| {
            let a_volume = a.volume_24hr.as_ref().or(a.volume.as_ref());
            let b_volume = b.volume_24hr.as_ref().or(b.volume.as_ref());

            match (a_volume, b_volume) {
                (Some(a), Some(b)) => b.partial_cmp(a).unwrap_or(Ordering::Equal),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            }
        });

        events.truncate(limit as usize);
        Ok(events)
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
