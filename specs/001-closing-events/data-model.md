# Data Model: Markets Closing Soon Command

**Feature**: 001-closing-events  
**Date**: 2026-03-31  
**Phase**: 1 (Design & Contracts)

## Overview

This document describes the data entities and their relationships for the `/closing` command implementation. Since this is a read-only query command with no persistence, the data model is minimal and focuses on API response structures and transformation logic.

---

## Entities

### 1. Event (External SDK Type)

**Source**: `polymarket_client_sdk::gamma::types::response::Event`

**Key Fields Used**:
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `title` | `Option<String>` | Event title/question | Must be present for display |
| `slug` | `Option<String>` | URL-friendly identifier | Must be present |
| `end_date` | `Option<DateTime<Utc>>` | When event closes | **Critical**: Must be present for filtering |
| `volume_24hr` | `Option<Decimal>` | 24-hour trading volume | Used for secondary sorting |
| `markets` | `Option<Vec<Market>>` | Associated markets | Must have at least one for price display |
| `closed` | `Option<bool>` | Whether event is closed | Must be `false` |
| `archived` | `Option<bool>` | Whether event is archived | Must be `false` |

**Relationships**:
- One Event has many Markets (1:N)
- For display, we use the first market's first outcome price

**State Transitions**: None (immutable API response)

---

### 2. Market (External SDK Type)

**Source**: `polymarket_client_sdk::gamma::types::response::Market`

**Key Fields Used**:
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `outcome_prices` | `Option<Vec<Decimal>>` | Prices for each outcome (0.0-1.0) | Must have at least one element |
| `outcomes` | `Option<Vec<String>>` | Outcome labels (e.g., ["Yes", "No"]) | Optional (we show price only) |

**Usage**: Extract `outcome_prices[0]` for "Yes" price display

---

### 3. ClosingEventInfo (Internal Type - Not Persisted)

**Purpose**: Intermediate representation for formatting

**Definition** (conceptual, may not need explicit struct):
```rust
struct ClosingEventInfo {
    title: String,
    slug: String,
    time_until_close: chrono::Duration,
    yes_price_cents: u32,
    volume_24hr: Decimal,
}
```

**Lifecycle**: 
1. Created during formatting from `Event`
2. Used for display rendering
3. Dropped after response sent

**Validation Rules**:
- `time_until_close` must be positive (negative means already closed, should be filtered)
- `yes_price_cents` must be in range [0, 100]

---

## Data Transformations

### Transformation 1: API Response → Filtered Events

**Input**: `Vec<Event>` from Gamma API  
**Output**: `Vec<Event>` (filtered and sorted)

**Steps**:
1. Filter where `end_date.is_some()`
2. Filter where `end_date > now`
3. Filter where `end_date <= now + 24h`
4. Filter where `closed != Some(true)`
5. Filter where `archived != Some(true)`
6. Sort by `end_date` ASC, then `volume_24hr` DESC
7. Truncate to 10 elements

**Implementation Location**: `src/polymarket/gamma.rs::list_closing_events()`

---

### Transformation 2: Event → Display String

**Input**: `Vec<Event>` (filtered)  
**Output**: `String` (formatted response)

**Steps**:
1. Check if empty → return "No markets closing in the next 24 hours."
2. Build header: "Markets Closing Soon:\n\n"
3. For each event (up to 10):
   - Extract title
   - Calculate `end_date - now` → format as "Xh Ym"
   - Extract `markets[0].outcome_prices[0]` → convert to cents
   - Extract `volume_24hr` → format as "$X.XM"
   - Format as numbered list entry
4. Join all entries with newlines

**Implementation Location**: `src/format.rs::format_closing_events()`

---

### Transformation 3: Duration → Human-Readable String

**Input**: `chrono::Duration`  
**Output**: `String` (e.g., "5h 23m", "45m", "<1m")

**Logic**:
```rust
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

**Implementation Location**: `src/format.rs::format_duration()`

---

## Data Flow Diagram

```
┌─────────────────────┐
│  User sends DM:     │
│  "/closing"         │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│  bot.rs::handle_notifications()             │
│  Extracts message, routes to command        │
└──────────┬──────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│  commands/mod.rs::handle_command()          │
│  Matches "/closing" → closing::handle()     │
└──────────┬──────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│  commands/closing.rs::handle()              │
│  Calls gamma.list_closing_events(24, 10)    │
└──────────┬──────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│  polymarket/gamma.rs::list_closing_events() │
│  1. Call SDK: gamma.events()                │
│     with active=true, closed=false          │
│  2. Filter: now < end_date <= now + 24h     │
│  3. Sort: end_date ASC, volume DESC         │
│  4. Truncate: 10 events                     │
└──────────┬──────────────────────────────────┘
           │
           ▼ Result<Vec<Event>, String>
           │
┌─────────────────────────────────────────────┐
│  commands/closing.rs::handle()              │
│  Match on Result:                           │
│    Ok → format::format_closing_events()     │
│    Err → user-friendly error message        │
└──────────┬──────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│  format.rs::format_closing_events()         │
│  For each event:                            │
│    - format_duration(end_date - now)        │
│    - format_volume(volume_24hr)             │
│    - format_price_cents(price)              │
│  Build numbered list string                 │
└──────────┬──────────────────────────────────┘
           │
           ▼ String
           │
┌─────────────────────────────────────────────┐
│  bot.rs::handle_notifications()             │
│  Sends response via NIP-17/NIP-04 DM        │
└──────────┬──────────────────────────────────┘
           │
           ▼
┌─────────────────────┐
│  User receives:     │
│  Markets Closing... │
│  1. Title           │
│     Closes in 5h    │
│  ...                │
└─────────────────────┘
```

---

## Validation Rules

### Input Validation
- **Command**: No arguments required; any arguments are ignored

### API Response Validation
1. **Event.end_date**: Must be `Some(datetime)`, otherwise skip event
2. **Event.closed**: Must be `Some(false)` or `None`, otherwise skip event
3. **Event.archived**: Must be `Some(false)` or `None`, otherwise skip event
4. **Event.markets**: Must be `Some(vec)` with length > 0, otherwise skip event
5. **Market.outcome_prices**: Must be `Some(vec)` with length > 0, otherwise skip event

### Time Window Validation
- `end_date` must satisfy: `now < end_date <= now + Duration::hours(24)`
- If `end_date - now` is negative, event is filtered out (already closed)

### Display Validation
- All formatted strings are UTF-8 safe
- No HTML/markdown injection (plain text only)
- Price cents converted safely: `(price * 100.0).round() as u32`

---

## Error Handling

### API Errors
- **Network failure**: Return `"Error fetching closing markets: {error}"`
- **Timeout**: Return `"Error fetching closing markets: request timeout"`
- **Invalid JSON**: Return `"Error fetching closing markets: invalid response"`

### Empty Results
- No events in 24h window: Return `"No markets closing in the next 24 hours."`

### Malformed Data
- Event missing required fields: Log warning, skip event (don't fail entire request)
- Market missing prices: Skip event, continue processing others

---

## Performance Characteristics

### Time Complexity
- Filtering: O(n) where n = API results (max 200)
- Sorting: O(n log n) where n ≤ 200
- Formatting: O(m) where m = 10 (display limit)

**Overall**: O(n log n) dominated by sort, where n ≤ 200

### Space Complexity
- API response: ~50 KB for 200 events
- Filtered list: ~2.5 KB for 10 events
- Formatted output: ~2 KB string

**Overall**: O(n) linear in API response size

### Expected Latency
- API call: 200-500ms
- Client processing: <5ms
- **Total**: 205-505ms end-to-end

---

## No Persistence Layer

**Important**: This command is **stateless and read-only**. No data is:
- Written to database
- Cached in memory
- Stored in user session

Each invocation fetches fresh data from Polymarket API.

---

## Testing Data Scenarios

### Test Scenario 1: Normal Case
- 15 events closing in next 24h
- All have valid end_dates and prices
- Expected: Return first 10, sorted by time

### Test Scenario 2: Empty Results
- 0 events closing in next 24h
- Expected: "No markets closing in the next 24 hours."

### Test Scenario 3: Boundary Case
- Event closes in exactly 24h 0m 0s
- Expected: Included in results

### Test Scenario 4: Just Passed
- Event closed 1 minute ago but `closed=false`
- Expected: Filtered out (end_date in past)

### Test Scenario 5: Partial Data
- 5 events with valid data, 3 with missing end_date
- Expected: Return 5 valid events, skip 3

### Test Scenario 6: Same Close Time
- 3 events close at same time, different volumes
- Expected: Sorted by volume (highest first)

---

## Schema (Conceptual)

Since no persistence, this shows the "shape" of data at each stage:

```rust
// Stage 1: Raw API Response
Vec<Event> {
    title: Some("Market 1"),
    end_date: Some(2026-04-01T12:00:00Z),
    closed: Some(false),
    archived: Some(false),
    volume_24hr: Some(Decimal(2300000)),
    markets: Some(vec![
        Market {
            outcome_prices: Some(vec![0.68, 0.32]),
            ...
        }
    ]),
    ...
}

// Stage 2: Filtered & Sorted
Vec<Event> (10 elements, sorted by end_date)

// Stage 3: Formatted String
"Markets Closing Soon:

1. Market 1
   Closes in 5h 23m  |  Vol 24h: $2.3M  |  Yes: 68¢
   Slug: market-1
...
"
```

---

## Future Enhancements (Out of Scope)

- **Persistence**: Cache results for 60s (reduce API load)
- **Pagination**: Support `/closing next` for results 11-20
- **Custom Windows**: Support `/closing 48h` or `/closing 6h`
- **Category Filters**: Support `/closing crypto` or `/closing sports`
- **Notifications**: Integrate with alert system (notify when favorite market closes soon)

---

**Status**: ✅ Data Model Complete — Proceed to Contracts
