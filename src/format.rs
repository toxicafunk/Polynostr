use polymarket_client_sdk::gamma::types::response::{Event, Market, SearchResults};
use polymarket_client_sdk::types::Decimal;

/// Format a price (Decimal from 0.0 to 1.0) as cents (e.g., "52¢").
fn format_price_cents(price: &Decimal) -> String {
    let cents = price * Decimal::from(100);
    format!("{cents:.0}¢")
}

/// Format a volume value into a human-readable string (e.g., "$12.4M").
fn format_volume(volume: &Decimal) -> String {
    let val = volume.to_string().parse::<f64>().unwrap_or(0.0);
    if val >= 1_000_000.0 {
        format!("${:.1}M", val / 1_000_000.0)
    } else if val >= 1_000.0 {
        format!("${:.1}K", val / 1_000.0)
    } else {
        format!("${val:.0}")
    }
}

/// Format search results for nostr output.
pub fn format_search_results(results: &SearchResults, query: &str) -> String {
    let mut output = format!("Search results for \"{query}\":\n\n");

    if let Some(events) = &results.events {
        if events.is_empty() {
            output.push_str("No markets found. Try a different search term.");
            return output;
        }

        for (i, event) in events.iter().take(5).enumerate() {
            let title = event.title.as_deref().unwrap_or("Untitled");
            let slug = event.slug.as_deref().unwrap_or("—");
            let volume = event
                .volume
                .as_ref()
                .map(format_volume)
                .unwrap_or_else(|| String::from("—"));

            output.push_str(&format!("{}. {}\n", i + 1, title));
            output.push_str(&format!("   Volume: {volume}"));

            // Show first market price if available
            if let Some(markets) = &event.markets {
                if let Some(market) = markets.first() {
                    if let Some(prices) = &market.outcome_prices {
                        if let Some(yes_price) = prices.first() {
                            output.push_str(&format!("  |  Yes: {}", format_price_cents(yes_price)));
                        }
                    }
                }
            }

            output.push_str(&format!("\n   Slug: {slug}\n\n"));
        }
    } else {
        output.push_str("No markets found. Try a different search term.");
    }

    output.trim_end().to_owned()
}

/// Format a single market for a price query.
pub fn format_market_price(market: &Market) -> String {
    let title = market.question.as_deref().unwrap_or("Untitled Market");
    let mut output = format!("{title}\n\n");

    if let Some(prices) = &market.outcome_prices {
        if let Some(outcomes) = &market.outcomes {
            let pairs: Vec<String> = outcomes
                .iter()
                .zip(prices.iter())
                .map(|(name, price)| format!("{name}: {}", format_price_cents(price)))
                .collect();
            output.push_str(&pairs.join("  |  "));
            output.push('\n');
        }
    }

    if let Some(volume) = &market.volume {
        output.push_str(&format!("Volume: {}", format_volume(volume)));
    }
    if let Some(liquidity) = &market.liquidity {
        output.push_str(&format!("  |  Liquidity: {}", format_volume(liquidity)));
    }
    output.push('\n');

    if let Some(end_date) = &market.end_date {
        output.push_str(&format!("Ends: {}\n", end_date.format("%b %-d, %Y")));
    }

    if let Some(slug) = &market.slug {
        output.push_str(&format!("\npolymarket.com/market/{slug}"));
    }

    output
}

/// Format a detailed market view.
pub fn format_market_detail(market: &Market) -> String {
    let title = market.question.as_deref().unwrap_or("Untitled Market");
    let mut output = format!("{title}\n");
    output.push_str(&"─".repeat(title.len().min(40)));
    output.push('\n');

    if let Some(desc) = &market.description {
        let truncated = if desc.len() > 500 {
            format!("{}...", &desc[..497])
        } else {
            desc.clone()
        };
        output.push_str(&format!("\n{truncated}\n"));
    }

    output.push('\n');

    if let Some(prices) = &market.outcome_prices {
        if let Some(outcomes) = &market.outcomes {
            output.push_str("Prices:\n");
            for (name, price) in outcomes.iter().zip(prices.iter()) {
                output.push_str(&format!("  {name}: {}\n", format_price_cents(price)));
            }
        }
    }

    output.push('\n');

    if let Some(volume) = &market.volume {
        output.push_str(&format!("Volume: {}\n", format_volume(volume)));
    }
    if let Some(liquidity) = &market.liquidity {
        output.push_str(&format!("Liquidity: {}\n", format_volume(liquidity)));
    }
    if let Some(end_date) = &market.end_date {
        output.push_str(&format!("End Date: {}\n", end_date.format("%b %-d, %Y")));
    }
    if let Some(source) = &market.resolution_source {
        if !source.is_empty() {
            output.push_str(&format!("Resolution Source: {source}\n"));
        }
    }

    if let Some(slug) = &market.slug {
        output.push_str(&format!("\npolymarket.com/market/{slug}"));
    }

    output
}

/// Format trending events list.
pub fn format_trending_events(events: &[Event]) -> String {
    if events.is_empty() {
        return String::from("No trending markets found at the moment.");
    }

    let mut output = String::from("Trending Markets on Polymarket:\n\n");

    for (i, event) in events.iter().take(10).enumerate() {
        let title = event.title.as_deref().unwrap_or("Untitled");
        let volume = event
            .volume
            .as_ref()
            .map(format_volume)
            .unwrap_or_else(|| String::from("—"));

        output.push_str(&format!("{}. {}\n", i + 1, title));
        output.push_str(&format!("   Volume: {volume}"));

        // Show first market price if available
        if let Some(markets) = &event.markets {
            if let Some(market) = markets.first() {
                if let Some(prices) = &market.outcome_prices {
                    if let Some(yes_price) = prices.first() {
                        output.push_str(&format!("  |  Yes: {}", format_price_cents(yes_price)));
                    }
                }
            }
        }

        if let Some(slug) = &event.slug {
            output.push_str(&format!("\n   Slug: {slug}"));
        }
        output.push_str("\n\n");
    }

    output.trim_end().to_owned()
}
