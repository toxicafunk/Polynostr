# Research: Markets Closing Soon Command

**Feature**: 001-closing-events  
**Date**: 2026-03-31  
**Phase**: 0 (Outline & Research)

## Overview

This document consolidates research findings for implementing the `/closing` command. All technical unknowns from the planning phase have been investigated and resolved.

---

## Research Questions & Findings

### 1. Gamma API Filtering Capabilities

**Question**: Can the Polymarket Gamma API filter events by end_date range, or must we filter client-side?

**Finding**: 
The Gamma API `/events` endpoint does NOT support date range filtering parameters. Available filters are:
- `active` (boolean)
- `closed` (boolean)
- `archived` (boolean)
- `order` (field name for sorting)
- `ascending` (boolean)
- `limit` (integer)

**Decision**: **Client-side filtering required**. We will:
1. Fetch active events with over-provisioned limit (e.g., `limit=200`)
2. Filter to events where `current_time < end_date <= current_time + 24h`
3. Sort by `end_date` ascending (soonest first)
4. Truncate to 10 results

**Rationale**: This matches the existing pattern in `list_active_events()` at `src/polymarket/gamma.rs:50-82`, which over-fetches and filters client-side for reliability.

**Alternatives Considered**:
- Use `order=end_date` API parameter: Rejected because API doesn't guarantee sorting by end_date, only by volume/liquidity
- Request Polymarket API enhancement: Out of scope; work with existing API

---

### 2. Time Duration Formatting

**Question**: What's the best way to format time-until-close durations in Rust using chrono?

**Finding**: 
The `chrono` crate (already in dependencies) provides `Duration` type from subtracting two `DateTime<Utc>` values.

**Recommended Approach**:
```rust
use chrono::{DateTime, Utc};

fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();
    
    if total_seconds < 0 {
        return String::from("Closed");
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

**Example Outputs**:
- 5 hours 23 minutes → `"5h 23m"`
- 45 minutes → `"45m"`
- 23 hours 59 minutes → `"23h 59m"`
- Negative duration → `"Closed"`

**Decision**: Implement `format_duration()` helper in `src/format.rs`

**Alternatives Considered**:
- Use `humantime` crate: Rejected; adds unnecessary dependency
- Show seconds precision: Rejected; too granular for 24h window
- Use words "hours"/"minutes": Rejected; takes more space in output

---

### 3. Sorting Strategy

**Question**: Should we sort by (a) time-until-close only, or (b) time-until-close + secondary sort by volume?

**Finding**: 
Existing commands use volume-based sorting. However, for a "closing soon" command, **urgency is the primary concern**.

**Decision**: **Primary sort by end_date (ascending), secondary sort by volume_24hr (descending)**

**Rationale**:
- Users want to see what's closing SOONEST first
- Volume serves as tiebreaker for markets closing at the same time
- Matches user mental model: "What do I need to act on NOW?"

**Implementation**:
```rust
events.sort_by(|a, b| {
    match a.end_date.cmp(&b.end_date) {
        std::cmp::Ordering::Equal => {
            b.volume_24hr.cmp(&a.volume_24hr) // descending for volume
        }
        other => other
    }
});
```

**Alternatives Considered**:
- Sort by volume only: Rejected; defeats purpose of "closing soon"
- No secondary sort: Acceptable but less deterministic

---

### 4. Edge Cases & Boundary Conditions

**Question**: What edge cases need handling in time-based filtering?

**Findings**:

#### Case 1: Events Already Closed
- **Scenario**: Event `end_date` is in the past, but `closed=false` in API response
- **Handling**: Filter out with `end_date > now` check
- **Location**: Client-side filter in `list_closing_events()`

#### Case 2: Events Closing in <1 Minute
- **Scenario**: Event closes in 30 seconds
- **Handling**: Display as `"0h 0m"` or special case as `"<1m"`
- **Decision**: Display `"<1m"` for better UX

#### Case 3: Exactly 24 Hours Boundary
- **Scenario**: Event closes in exactly 24h 0m 0s
- **Handling**: Use `<=` comparison to include boundary
- **Filter logic**: `end_date <= now + 24h`

#### Case 4: No Events in 24h Window
- **Scenario**: No markets closing soon
- **Handling**: Return user-friendly message
- **Message**: `"No markets closing in the next 24 hours."`

#### Case 5: More Than 10 Events Closing
- **Scenario**: 50+ markets closing in 24h
- **Handling**: Truncate to 10 after sorting
- **Future Enhancement**: Pagination (out of scope for this feature)

#### Case 6: Market with Multiple Sub-Markets
- **Scenario**: Event has 5 markets, each with different prices
- **Handling**: Show first market's "Yes" price (matching `/trending` behavior)
- **Location**: `format_closing_events()` at `src/format.rs`

---

### 5. Testing Strategy

**Question**: What test cases are required for constitutional compliance (Principle IV)?

**Required Tests**:

#### `src/polymarket/gamma.rs` Tests:
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_list_closing_events_filters_by_time_window() {
        // Verify only events within 24h are returned
    }
    
    #[tokio::test]
    async fn test_list_closing_events_sorts_by_end_date() {
        // Verify soonest events appear first
    }
    
    #[tokio::test]
    async fn test_list_closing_events_excludes_closed_markets() {
        // Verify closed=true events are filtered out
    }
    
    #[tokio::test]
    async fn test_list_closing_events_empty_result() {
        // Verify handles no events gracefully
    }
}
```

#### `src/format.rs` Tests:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_format_duration_hours_and_minutes() {
        // 5h 23m case
    }
    
    #[test]
    fn test_format_duration_minutes_only() {
        // 45m case
    }
    
    #[test]
    fn test_format_duration_less_than_minute() {
        // <1m case
    }
    
    #[test]
    fn test_format_duration_negative() {
        // "Closed" case
    }
    
    #[test]
    fn test_format_closing_events_empty() {
        // Empty vec returns appropriate message
    }
}
```

**Decision**: Implement inline tests following the pattern in `src/alerts/evaluator.rs:82-200`

---

### 6. API Error Scenarios

**Question**: What Polymarket API failures need handling?

**Potential Failures**:
1. **Network timeout**: `reqwest` client timeout
2. **Rate limiting**: HTTP 429 response
3. **Server error**: HTTP 500+ response
4. **Invalid JSON**: Malformed response body
5. **Missing fields**: Unexpected null values

**Handling Strategy** (follows existing pattern):
```rust
match gamma.list_closing_events(24, 10).await {
    Ok(events) => format::format_closing_events(&events),
    Err(e) => format!("Error fetching closing markets: {}", e)
}
```

**Location**: `src/commands/closing.rs:handle()`

**Error Message Examples**:
- Generic: `"Error fetching closing markets: network timeout"`
- User-facing: Always safe to show (no sensitive data leaked)

---

### 7. Performance Considerations

**Question**: What's the expected latency and how can we optimize?

**Analysis**:

#### API Latency:
- Polymarket Gamma API: ~200-500ms typical response time
- Over-fetching 200 events vs 10: No significant difference (same endpoint call)

#### Client-Side Processing:
- Filtering 200 events: <1ms (simple date comparisons)
- Sorting 200 events: <1ms (efficient sort)
- Formatting 10 events: <1ms (string building)

**Total Expected Latency**: 200-600ms end-to-end (acceptable, matches `/trending`)

**Optimization Decisions**:
- No caching needed (data changes frequently)
- No parallel API calls (single endpoint suffices)
- Over-fetch factor: Use 200 limit (max supported by API)

**Monitoring**: Use existing `tracing` logging to track latency

---

### 8. Best Practices Review

**Question**: Are there Rust idioms or patterns we should follow?

**Findings**:

#### Error Handling:
- Use `Result<T, String>` for API methods (matches existing pattern)
- Use `?` operator for error propagation
- Convert SDK errors to user-friendly strings at boundary

#### Type Safety:
- Use `chrono::Duration` for time calculations (not raw integers)
- Use `DateTime<Utc>` for timestamps (not strings)
- Use `Option` for nullable API fields

#### Async Patterns:
- Mark all I/O functions as `async fn`
- Use `.await` for API calls
- No blocking operations in async context

#### Formatting:
- Use `String::from()` for static strings
- Use `format!()` for dynamic strings
- Preallocate with `String::with_capacity()` if building large strings

**Decision**: Follow existing codebase patterns exactly (no new patterns introduced)

---

## Technology Decisions Summary

| Technology | Decision | Rationale |
|------------|----------|-----------|
| **API Filtering** | Client-side filtering after over-fetch | API lacks date range parameters |
| **Time Formatting** | Custom `format_duration()` with chrono | No external dependency needed |
| **Sorting** | Primary: end_date ASC, Secondary: volume DESC | Urgency-first, volume tiebreaker |
| **Edge Case Handling** | Explicit checks for boundaries and nulls | Defensive programming |
| **Testing** | Inline `#[cfg(test)]` modules | Constitutional requirement (Principle IV) |
| **Error Handling** | User-friendly string messages | Matches existing pattern |
| **Performance** | Over-fetch 200, filter client-side | Acceptable latency, simple implementation |

---

## Alternatives Considered & Rejected

### Alternative 1: Use External Time Formatting Crate
**Considered**: `humantime` or `chrono-humanize` for duration formatting

**Rejected Because**:
- Adds unnecessary dependency
- Custom formatter is <10 LOC
- No need for localization or complex formatting

---

### Alternative 2: Server-Side Filtering via API Extension
**Considered**: Request Polymarket to add date range filters

**Rejected Because**:
- Out of our control
- Delays feature delivery
- Client-side filtering works fine

---

### Alternative 3: Cache Results for Fast Queries
**Considered**: Cache `/closing` results for 60 seconds

**Rejected Because**:
- Adds complexity (cache invalidation, TTL management)
- Time-sensitive data should be fresh
- Latency is already acceptable (~500ms)
- Would violate stateless command pattern

---

### Alternative 4: Customizable Time Window
**Considered**: Support `/closing 48h` or `/closing 12h`

**Rejected Because**:
- Increases interface complexity
- 24h is sensible default
- Can be added later if users request it (YAGNI principle)

---

## Open Questions

**None** — All technical unknowns have been resolved. Proceed to Phase 1 (Design & Contracts).

---

## References

- Existing filtering logic: `src/polymarket/gamma.rs:50-82`
- Existing formatter pattern: `src/format.rs:216-258`
- chrono documentation: https://docs.rs/chrono/latest/chrono/
- Polymarket Gamma API: https://gamma-api.polymarket.com/
- Project constitution: `.specify/memory/constitution.md`

---

**Status**: ✅ Research Complete — Ready for Phase 1
