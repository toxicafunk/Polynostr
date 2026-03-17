use crate::format;
use crate::polymarket::gamma::GammaClient;

/// Handle the `/market <slug>` command.
/// Tries to fetch as an event first (which contains multiple markets),
/// then falls back to fetching as an individual market.
pub async fn handle(gamma: &GammaClient, args: &str) -> String {
    let slug = args.trim();
    if slug.is_empty() {
        return String::from(
            "Usage: /market <slug>\n\n\
             Example: /market bitcoin-above-on-march-17\n\
             Example: /market will-bitcoin-hit-100k\n\n\
             Tip: Use /search to find slugs.",
        );
    }

    // Try fetching as an event first (events contain multiple markets)
    match gamma.get_event_by_slug(slug).await {
        Ok(event) => return format::format_event_detail(&event),
        Err(_) => {
            // If event fetch fails, try as a market
            match gamma.get_market_by_slug(slug).await {
                Ok(market) => format::format_market_detail(&market),
                Err(e) => format!(
                    "Could not find event or market \"{slug}\".\n\n\
                     {e}\n\n\
                     Tip: Use /search to find the correct slug."
                ),
            }
        }
    }
}
