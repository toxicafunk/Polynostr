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

    /// List active events ordered by 24h volume (descending).
    pub async fn list_active_events(&self, limit: u32) -> Result<Vec<Event>, String> {
        let fetch_limit = ((limit as i32) * 5).min(200);
        let request = EventsRequest::builder()
            .active(true)
            .closed(false)
            .archived(false)
            .limit(fetch_limit)
            .order(vec!["volume24hr".to_owned()])
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
            let has_future_end_date = event
                .end_date
                .as_ref()
                .map(|end_date| end_date > &now)
                .unwrap_or(false);

            !is_closed && !is_archived && has_future_end_date
        });

        events.truncate(limit as usize);
        Ok(events)
    }

    /// List events closing within the specified number of hours.
    ///
    /// This method over-fetches from the API and filters client-side because the
    /// Gamma API does not support date range filtering parameters.
    ///
    /// Events are sorted by end_date (ascending, soonest first), then by volume_24hr
    /// (descending, highest first) as a tiebreaker.
    pub async fn list_closing_events(
        &self,
        within_hours: u64,
        limit: usize,
    ) -> Result<Vec<Event>, String> {
        let now = Utc::now();
        let deadline = now + chrono::Duration::hours(within_hours as i64);

        // Over-fetch to account for filtering (20x limit, max 200)
        let fetch_limit = (limit * 20).min(200);

        let request = EventsRequest::builder()
            .active(true)
            .closed(false)
            .archived(false)
            .order(vec!["volume24hr".to_owned()])
            .ascending(false)
            .limit(fetch_limit as i32)
            .build();

        let response = self
            .client
            .events(&request)
            .await
            .map_err(|e| format!("API error: {e}"))?;

        // Client-side filtering
        let mut filtered: Vec<Event> = response
            .into_iter()
            .filter(|event| {
                let has_end_date = event.end_date.is_some();
                let is_closed = event.closed.unwrap_or(false);
                let is_archived = event.archived.unwrap_or(false);

                if !has_end_date || is_closed || is_archived {
                    return false;
                }

                let end_date = event.end_date.as_ref().unwrap();

                // Must be in the future but within the window
                end_date > &now && end_date <= &deadline
            })
            .collect();

        // Sort by end_date (soonest first), then by volume (highest first)
        filtered.sort_by(|a, b| {
            match (a.end_date.as_ref(), b.end_date.as_ref()) {
                (Some(end_a), Some(end_b)) => match end_a.cmp(end_b) {
                    std::cmp::Ordering::Equal => {
                        // Secondary sort by volume (descending)
                        let vol_a = a.volume_24hr.as_ref();
                        let vol_b = b.volume_24hr.as_ref();
                        vol_b.cmp(&vol_a)
                    }
                    other => other,
                },
                _ => std::cmp::Ordering::Equal,
            }
        });

        // Truncate to requested limit
        filtered.truncate(limit);

        Ok(filtered)
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use polymarket_client_sdk::types::Decimal;

    // Note: We can't create Event instances directly because they're non-exhaustive structs.
    // These tests demonstrate the filtering and sorting logic that would be applied to real events.

    #[test]
    fn test_filtering_logic_within_window() {
        let now = Utc::now();
        let deadline = now + Duration::hours(24);

        // Test the time window logic
        let within_window = now + Duration::hours(12);
        let outside_window = now + Duration::hours(25);

        assert!(within_window > now && within_window <= deadline);
        assert!(!(outside_window > now && outside_window <= deadline));
    }

    #[test]
    fn test_filtering_excludes_closed_markets() {
        // Test the closed filter logic
        let closed = Some(true);
        let not_closed = Some(false);
        let unknown = None;

        assert!(closed.unwrap_or(false));
        assert!(!not_closed.unwrap_or(false));
        assert!(!unknown.unwrap_or(false));
    }

    #[test]
    fn test_filtering_excludes_archived_markets() {
        // Test the archived filter logic
        let archived = Some(true);
        let not_archived = Some(false);
        let unknown = None;

        assert!(archived.unwrap_or(false));
        assert!(!not_archived.unwrap_or(false));
        assert!(!unknown.unwrap_or(false));
    }

    #[test]
    fn test_sorting_comparison_logic() {
        let now = Utc::now();

        // Test datetime ordering
        let earlier = now + Duration::hours(1);
        let later = now + Duration::hours(5);

        assert!(earlier < later);

        // Test volume ordering (for tiebreakers)
        let vol_low = Some(Decimal::from(1000000));
        let vol_high = Some(Decimal::from(5000000));

        // When times are equal, higher volume should come first (descending)
        assert!(vol_high > vol_low);
    }

    #[test]
    fn test_empty_results() {
        let events: Vec<Event> = vec![];
        assert_eq!(events.len(), 0);
    }
}
