use crate::format;
use crate::polymarket::gamma::GammaClient;

/// Handle the `/price <slug>` command.
pub async fn handle(gamma: &GammaClient, args: &str) -> String {
    let slug = args.trim();
    if slug.is_empty() {
        return String::from(
            "Usage: /price <market-slug>\n\n\
             Example: /price will-bitcoin-hit-100k\n\n\
             Tip: Use /search to find market slugs.",
        );
    }

    match gamma.get_market_by_slug(slug).await {
        Ok(market) => format::format_market_price(&market),
        Err(e) => format!(
            "Could not find market \"{slug}\".\n\n\
             {e}\n\n\
             Tip: Use /search to find the correct slug."
        ),
    }
}
