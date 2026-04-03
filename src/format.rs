use chrono::{DateTime, Utc};
use polymarket_client_sdk::gamma::types::response::{Event, Market, SearchResults};
use polymarket_client_sdk::types::Decimal;

use crate::alerts::model::AlertSubscription;

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

/// Format a duration as human-readable time (e.g., "5h 23m", "45m", "<1m").
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 0 {
        return String::from("Closed");
    }

    if total_seconds < 60 {
        return String::from("<1m");
    }

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}


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

            if let Some(markets) = &event.markets {
                if let Some(market) = markets.first() {
                    if let Some(prices) = &market.outcome_prices {
                        if let Some(yes_price) = prices.first() {
                            output
                                .push_str(&format!("  |  Yes: {}", format_price_cents(yes_price)));
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

pub fn format_event_detail(event: &Event) -> String {
    let title = event.title.as_deref().unwrap_or("Untitled Event");
    let mut output = format!("{title}\n");
    output.push_str(&"─".repeat(title.len().min(40)));
    output.push('\n');

    if let Some(desc) = &event.description {
        let truncated = if desc.len() > 500 {
            format!("{}...", &desc[..497])
        } else {
            desc.clone()
        };
        output.push_str(&format!("\n{truncated}\n"));
    }

    output.push('\n');

    if let Some(volume) = &event.volume {
        output.push_str(&format!("Total Volume: {}\n", format_volume(volume)));
    }
    if let Some(liquidity) = &event.liquidity {
        output.push_str(&format!("Total Liquidity: {}\n", format_volume(liquidity)));
    }
    if let Some(end_date) = &event.end_date {
        output.push_str(&format!("End Date: {}\n", end_date.format("%b %-d, %Y")));
    }

    if let Some(markets) = &event.markets {
        if !markets.is_empty() {
            output.push_str(&format!("\nMarkets ({}):\n", markets.len()));
            for (i, market) in markets.iter().enumerate() {
                let question = market.question.as_deref().unwrap_or("Unknown");
                output.push_str(&format!("\n{}. {}\n", i + 1, question));

                if let Some(prices) = &market.outcome_prices {
                    if let Some(outcomes) = &market.outcomes {
                        output.push_str("   Prices: ");
                        let price_str: Vec<String> = outcomes
                            .iter()
                            .zip(prices.iter())
                            .map(|(name, price)| format!("{}: {}", name, format_price_cents(price)))
                            .collect();
                        output.push_str(&price_str.join(", "));
                        output.push('\n');
                    }
                }

                if let Some(volume) = &market.volume {
                    output.push_str(&format!("   Volume: {}\n", format_volume(volume)));
                }

                if let Some(slug) = &market.slug {
                    output.push_str(&format!("   Slug: {slug}\n"));
                }
            }
        }
    }

    if let Some(slug) = &event.slug {
        output.push_str(&format!("\npolymarket.com/event/{slug}"));
    }

    output
}

pub fn format_trending_events(events: &[Event]) -> String {
    if events.is_empty() {
        return String::from("No trending markets found at the moment.");
    }

    let mut output = String::from("Trending Markets on Polymarket:\n\n");

    for (i, event) in events.iter().take(10).enumerate() {
        let title = event.title.as_deref().unwrap_or("Untitled");
        let volume_24h = event
            .volume_24hr
            .as_ref()
            .map(format_volume)
            .unwrap_or_else(|| String::from("—"));
        let total_volume = event
            .volume
            .as_ref()
            .map(format_volume)
            .unwrap_or_else(|| String::from("—"));

        output.push_str(&format!("{}. {}\n", i + 1, title));
        output.push_str(&format!(
            "   Vol 24h: {volume_24h}  |  Total: {total_volume}"
        ));

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

/// Format events closing soon into a user-friendly message.
pub fn format_closing_events(events: &[Event]) -> String {
    if events.is_empty() {
        return String::from("No markets closing in the next 24 hours.");
    }

    let mut output = String::from("Markets Closing Soon:\n\n");
    let now = Utc::now();

    for (i, event) in events.iter().enumerate() {
        let title = event
            .title
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Unknown Event");

        // Calculate time until close
        let time_left = if let Some(end_date) = &event.end_date {
            let duration = *end_date - now;
            format_duration(duration)
        } else {
            String::from("Unknown")
        };

        // Get first market's first outcome price
        let price = event
            .markets
            .as_ref()
            .and_then(|markets| markets.first())
            .and_then(|market| market.outcome_prices.as_ref())
            .and_then(|prices| prices.first())
            .map(|price| format_price_cents(price))
            .unwrap_or_else(|| String::from("N/A"));

        // Get 24h volume
        let vol_24h = event
            .volume_24hr
            .as_ref()
            .map(format_volume)
            .unwrap_or_else(|| String::from("N/A"));

        // Get slug
        let slug = event
            .slug
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        output.push_str(&format!(
            "{}. {}\n   Closes in {}  |  Vol 24h: {}  |  Yes: {}\n   Slug: {}\n\n",
            i + 1,
            title,
            time_left,
            vol_24h,
            price,
            slug
        ));
    }

    output.trim_end().to_owned()
}


pub fn format_alert_created(alert: &AlertSubscription) -> String {
    format!(
        "Alert created\n\nID: {}\nMarket: {}\nRule: {}\nStatus: active",
        alert.id,
        alert.slug,
        alert.rule.describe()
    )
}

pub fn format_alert_list(alerts: &[AlertSubscription]) -> String {
    if alerts.is_empty() {
        return "You have no alerts configured. Use: alert add <slug> <above|below|move> <value>"
            .to_owned();
    }

    let mut out = String::from("Your alerts:\n\n");
    for (idx, alert) in alerts.iter().enumerate() {
        out.push_str(&format!(
            "{}. {}\n   ID: {}\n   Rule: {}\n   Status: {:?}\n\n",
            idx + 1,
            alert.slug,
            alert.id,
            alert.rule.describe(),
            alert.status
        ));
    }
    out.trim_end().to_owned()
}

pub fn format_alert_removed(alert_id: &str) -> String {
    format!("Alert removed: {alert_id}")
}

pub fn format_alert_paused(alert_id: &str) -> String {
    format!("Alert paused: {alert_id}")
}

pub fn format_alert_resumed(alert_id: &str) -> String {
    format!("Alert resumed: {alert_id}")
}

pub fn format_alert_trigger(
    alert: &AlertSubscription,
    price: f64,
    triggered_at: DateTime<Utc>,
) -> String {
    format!(
        "Price alert triggered\n\nMarket: {}\nRule: {}\nCurrent Price: {:.2}¢\nTriggered: {}\nAlert ID: {}",
        alert.slug,
        alert.rule.describe(),
        price * 100.0,
        triggered_at.format("%Y-%m-%d %H:%M:%SZ"),
        alert.id
    )
}

pub fn format_alert_test(alert: &AlertSubscription) -> String {
    format!(
        "Alert test notification\n\nMarket: {}\nRule: {}\nAlert ID: {}",
        alert.slug,
        alert.rule.describe(),
        alert.id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_format_duration_hours_and_minutes() {
        let duration = Duration::hours(5) + Duration::minutes(23);
        assert_eq!(format_duration(duration), "5h 23m");
    }

    #[test]
    fn test_format_duration_minutes_only() {
        let duration = Duration::minutes(45);
        assert_eq!(format_duration(duration), "45m");
    }

    #[test]
    fn test_format_duration_less_than_minute() {
        let duration = Duration::seconds(30);
        assert_eq!(format_duration(duration), "<1m");
    }

    #[test]
    fn test_format_duration_exactly_one_minute() {
        let duration = Duration::seconds(60);
        assert_eq!(format_duration(duration), "1m");
    }

    #[test]
    fn test_format_duration_negative() {
        let duration = Duration::seconds(-100);
        assert_eq!(format_duration(duration), "Closed");
    }

    #[test]
    fn test_format_duration_zero_hours() {
        let duration = Duration::minutes(5);
        assert_eq!(format_duration(duration), "5m");
    }

    #[test]
    fn test_format_duration_23_hours() {
        let duration = Duration::hours(23) + Duration::minutes(59);
        assert_eq!(format_duration(duration), "23h 59m");
    }

    #[test]
    fn test_format_closing_events_empty() {
        let events: Vec<Event> = vec![];
        assert_eq!(
            format_closing_events(&events),
            "No markets closing in the next 24 hours."
        );
    }

    // Note: We can't create Event instances for more complex tests because Event is a
    // non-exhaustive struct from the SDK. The formatter logic is validated through
    // integration testing with real API responses.
}

