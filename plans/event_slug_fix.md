# Event Slug Fix

## Problem

When users tried to use the `/market` command with a Polymarket event URL like:
```
https://polymarket.com/event/bitcoin-above-on-march-17
```

The bot would extract the slug `bitcoin-above-on-march-17` but fail with a 404 error:
```
Could not find market "bitcoin-above-on-march-17".
Failed to fetch market 'bitcoin-above-on-march-17': Status: error(404 Not Found) 
making GET call to /markets/slug/bitcoin-above-on-march-17
```

## Root Cause

Polymarket has two different entities:
- **Events**: Top-level groupings that contain multiple markets (e.g., "Bitcoin above ___ on March 17?" is an event)
- **Markets**: Individual prediction markets within an event (e.g., "Will Bitcoin be above $62,000 on March 17?")

The bot was only querying the `/markets/slug/` endpoint, which doesn't work for event slugs. Events are accessed via `/events/slug/` endpoint.

## Solution

### 1. Added Event Fetching Support

Added new method to `GammaClient` to fetch events by slug:

```rust
pub async fn get_event_by_slug(&self, slug: &str) -> Result<Event, String> {
    let request = EventBySlugRequest::builder().slug(slug).build();
    self.client
        .event_by_slug(&request)
        .await
        .map_err(|e| format!("Failed to fetch event '{slug}': {e}"))
}
```

### 2. Updated Market Command Logic

Modified `/market` command to:
1. First try to fetch as an event (which contains multiple markets)
2. If that fails, fall back to fetching as an individual market
3. This supports both event URLs and individual market URLs

### 3. Added Event Formatter

Created `format_event_detail()` function that displays:
- Event title and description
- Total volume and liquidity across all markets
- End date
- List of all markets within the event, showing:
  - Question
  - Current prices for each outcome
  - Individual market volume
  - Market slug for reference
- Link to the event on Polymarket

## Benefits

- Users can now use slugs from both event URLs and individual market URLs
- Better user experience - shows all related markets when given an event
- More flexible command that handles the most common use case (event URLs)
- Falls back gracefully to individual markets when needed

## Testing

Test cases:
1. Event slug: `/market bitcoin-above-on-march-17` ✓
2. Individual market slug: `/market bitcoin-above-62k-on-march-17` ✓
3. Invalid slug: `/market invalid-slug` → Shows helpful error message ✓

## Files Changed

- `polynostr/src/polymarket/gamma.rs`: Added `get_event_by_slug()` method
- `polynostr/src/commands/market.rs`: Updated to try event first, then market
- `polynostr/src/format.rs`: Added `format_event_detail()` function