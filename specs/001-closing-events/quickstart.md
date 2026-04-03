# Quickstart: Implementing the `/closing` Command

**Feature**: 001-closing-events  
**Date**: 2026-03-31  
**Estimated Time**: 2-3 hours  
**Difficulty**: ⭐⭐☆☆☆ (Moderate)

## Overview

This guide walks you through implementing the `/closing` command step by step. By following this guide, you'll add a new command to the Polynostr bot that shows markets closing within 24 hours.

---

## Prerequisites

- ✅ Rust 1.88.0+ installed
- ✅ Polynostr repository cloned and building successfully
- ✅ Basic familiarity with async Rust and the codebase
- ✅ Read `specs/001-closing-events/spec.md` and `research.md`

---

## Implementation Steps

### Step 1: Add the Command Handler (15 minutes)

**Create**: `src/commands/closing.rs`

```rust
use crate::format;
use crate::polymarket::gamma::GammaClient;

/// Handle the /closing command - show markets closing within 24 hours
pub async fn handle(gamma: &GammaClient) -> String {
    match gamma.list_closing_events(24, 10).await {
        Ok(events) => format::format_closing_events(&events),
        Err(e) => format!("Error fetching closing markets: {}", e),
    }
}
```

**Why**: This follows the established pattern from `trending.rs` and other commands. Simple delegation to gamma client with error handling.

---

### Step 2: Register the Command (5 minutes)

**Modify**: `src/commands/mod.rs`

**Add module declaration** (around line 10):
```rust
pub mod closing;
```

**Add routing** (in `handle_command` function, around line 64):
```rust
"/closing" | "closing" => closing::handle(gamma).await,
```

**Why**: This makes the bot aware of the new command and routes user input to the handler.

---

### Step 3: Implement API Method (30 minutes)

**Modify**: `src/polymarket/gamma.rs`

**Add the new method** (after `list_active_events`, around line 83):

```rust
/// List events closing within the specified number of hours
pub async fn list_closing_events(
    &self,
    within_hours: u64,
    limit: usize,
) -> Result<Vec<Event>, String> {
    use chrono::Utc;
    
    let now = Utc::now();
    let deadline = now + chrono::Duration::hours(within_hours as i64);
    
    // Over-fetch to account for filtering
    let fetch_limit = (limit * 20).min(200);
    
    let request = EventsRequest::builder()
        .active(true)
        .closed(false)
        .archived(false)
        .order(vec!["volume24hr".to_owned()])
        .ascending(false)
        .limit(fetch_limit)
        .build()
        .map_err(|e| format!("Failed to build request: {e}"))?;
    
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
            (Some(end_a), Some(end_b)) => {
                match end_a.cmp(end_b) {
                    std::cmp::Ordering::Equal => {
                        // Secondary sort by volume (descending)
                        let vol_a = a.volume_24hr.as_ref();
                        let vol_b = b.volume_24hr.as_ref();
                        vol_b.cmp(&vol_a)
                    }
                    other => other,
                }
            }
            _ => std::cmp::Ordering::Equal,
        }
    });
    
    // Truncate to requested limit
    filtered.truncate(limit);
    
    Ok(filtered)
}
```

**Why**: This implements the time-based filtering and sorting logic researched in Phase 0. Over-fetching ensures we get enough results after filtering.

---

### Step 4: Implement Formatters (45 minutes)

**Modify**: `src/format.rs`

**Add duration formatter** (around line 24, after `format_volume`):

```rust
/// Format a duration as human-readable time (e.g., "5h 23m", "45m")
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
```

**Add main formatter** (after `format_trending_events`, around line 259):

```rust
/// Format events closing soon into a user-friendly message
pub fn format_closing_events(events: &[Event]) -> String {
    use chrono::Utc;
    
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
            .map(|price| format_price_cents(*price))
            .unwrap_or_else(|| String::from("N/A"));
        
        // Get 24h volume
        let vol_24h = event
            .volume_24hr
            .as_ref()
            .map(|v| format_volume(*v))
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
    
    output
}
```

**Why**: These formatters transform API data into human-readable text following the established pattern.

---

### Step 5: Update Help Text (5 minutes)

**Modify**: `src/commands/help.rs`

**Add line** (in the commands list, around line 18):

```rust
/closing                        Markets closing in next 24 hours
```

**Why**: Documents the new command for users who run `/help`.

---

### Step 6: Add Tests (45 minutes)

**Add to**: `src/polymarket/gamma.rs` (at end of file)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_list_closing_events_filters_time_window() {
        // This test would require mocking the SDK client
        // For now, document the test structure
        
        // TODO: Create mock GammaClient
        // TODO: Inject events with various end_dates
        // TODO: Assert only events within 24h are returned
    }
    
    #[tokio::test]
    async fn test_list_closing_events_sorts_by_end_date() {
        // TODO: Mock events with different end_dates
        // TODO: Assert results are sorted soonest first
    }
    
    #[tokio::test]
    async fn test_list_closing_events_excludes_closed() {
        // TODO: Mock events with closed=true
        // TODO: Assert they are filtered out
    }
}
```

**Add to**: `src/format.rs` (at end of file or in existing test module)

```rust
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
    fn test_format_duration_negative() {
        let duration = Duration::seconds(-100);
        assert_eq!(format_duration(duration), "Closed");
    }
    
    #[test]
    fn test_format_closing_events_empty() {
        let events: Vec<Event> = vec![];
        assert_eq!(
            format_closing_events(&events),
            "No markets closing in the next 24 hours."
        );
    }
}
```

**Why**: Tests ensure correctness and satisfy constitutional requirement (Principle IV).

---

### Step 7: Build and Test (15 minutes)

**Build the project**:
```bash
cargo build
```

**Run tests**:
```bash
cargo test
```

**Run the bot locally**:
```bash
cargo run
```

**Test the command**:
1. Note the bot's `npub` from the logs
2. Open a Nostr client (Damus, Primal, etc.)
3. Send DM: `/closing`
4. Verify response shows markets closing in next 24h

**Why**: Ensures the implementation works end-to-end.

---

## Verification Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test`)
- [ ] Bot starts successfully
- [ ] Command responds to `/closing` via DM
- [ ] Command responds to `closing` (no slash)
- [ ] Results show time-until-close
- [ ] Results sorted by closing time (soonest first)
- [ ] Empty state handled gracefully
- [ ] API errors produce user-friendly messages
- [ ] Help text includes new command
- [ ] Works via NIP-17, NIP-04, and public mention

---

## Common Issues & Solutions

### Issue 1: "Method not found: list_closing_events"
**Cause**: Method not added to `GammaClient`  
**Fix**: Ensure Step 3 is complete and you've added the method to `impl GammaClient`

### Issue 2: "Cannot find module `closing`"
**Cause**: Module not registered in `mod.rs`  
**Fix**: Check Step 2 - ensure `pub mod closing;` is added

### Issue 3: Tests don't compile
**Cause**: Missing imports or incorrect type references  
**Fix**: Ensure all `use` statements are present:
```rust
use chrono::{Duration, Utc};
use super::*;
```

### Issue 4: Command doesn't respond
**Cause**: Routing not added or typo in match arm  
**Fix**: Check Step 2 - ensure match arm is `"/closing" | "closing" =>`

### Issue 5: Duration shows strange values
**Cause**: Timezone issues or incorrect duration calculation  
**Fix**: Ensure you're using `chrono::Utc` consistently, not local time

---

## Testing Strategy

### Manual Testing

**Test Case 1: Normal Operation**
```
Input: /closing
Expected: List of markets closing in <24h
```

**Test Case 2: No Markets**
```
Input: /closing
Expected: "No markets closing in the next 24 hours."
Note: May need to test at a time when no markets are closing
```

**Test Case 3: API Error**
```
Simulate: Disconnect internet
Input: /closing
Expected: "Error fetching closing markets: ..."
```

### Automated Testing

Run unit tests:
```bash
cargo test format_duration
cargo test format_closing_events
```

Run integration tests (if SDK mocking implemented):
```bash
cargo test list_closing_events
```

---

## Performance Validation

**Expected**: Command responds in 200-600ms

**Measure**:
```bash
# In one terminal:
cargo run

# In another terminal (requires nostr-sdk CLI tools):
time nostr-dm <bot-npub> "/closing"
```

**Acceptable**: <2 seconds end-to-end

---

## Deployment

**No special steps required** beyond standard bot deployment:

1. Build release binary:
```bash
cargo build --release
```

2. Deploy to server (copy binary + .env)

3. Restart bot service

4. Monitor logs for errors

---

## Next Steps

After implementation:

1. **Monitor Usage**: Check logs to see if users discover the command
2. **Gather Feedback**: Ask users if the format is useful
3. **Consider Enhancements**:
   - Add to alert system (notify when favorite market closes soon)
   - Support custom time windows (`/closing 48h`)
   - Add pagination for >10 results

---

## Resources

- **Reference Implementation**: `src/commands/trending.rs` (similar pattern)
- **API Filtering Example**: `src/polymarket/gamma.rs:50-82`
- **Format Examples**: `src/format.rs:216-258`
- **Testing Examples**: `src/alerts/evaluator.rs:82-200`

---

## Time Estimates

| Step | Time | Cumulative |
|------|------|------------|
| Step 1: Handler | 15 min | 15 min |
| Step 2: Registration | 5 min | 20 min |
| Step 3: API Method | 30 min | 50 min |
| Step 4: Formatters | 45 min | 95 min |
| Step 5: Help Text | 5 min | 100 min |
| Step 6: Tests | 45 min | 145 min |
| Step 7: Build & Test | 15 min | 160 min |

**Total**: ~2.5 hours (experienced Rust developer)

**Add 30-60 minutes** if new to the codebase.

---

**Status**: ✅ Quickstart Complete — Ready for implementation
