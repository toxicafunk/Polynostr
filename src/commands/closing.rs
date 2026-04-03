use crate::format;
use crate::polymarket::gamma::GammaClient;

/// Handle the `/closing` command - show markets closing within 24 hours.
pub async fn handle(gamma: &GammaClient) -> String {
    match gamma.list_closing_events(24, 10).await {
        Ok(events) => format::format_closing_events(&events),
        Err(e) => format!("Error fetching closing markets: {}", e),
    }
}
