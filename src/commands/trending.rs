use crate::format;
use crate::polymarket::gamma::GammaClient;

/// Handle the `/trending` command.
pub async fn handle(gamma: &GammaClient) -> String {
    match gamma.list_active_events(10).await {
        Ok(events) => format::format_trending_events(&events),
        Err(e) => format!("Error fetching trending markets: {e}"),
    }
}
