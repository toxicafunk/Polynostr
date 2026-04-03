# Feature Specification: Markets Closing Soon Command

**Feature ID**: 001-closing-events  
**Date**: 2026-03-31  
**Status**: Planning (Phase 0)

## Overview

Add a `/closing` command to the Polynostr bot that returns prediction markets closing in the next 24 hours or less. This provides users with actionable information about time-sensitive markets they may want to trade before closure.

## User Stories

1. **As a Nostr user**, I want to see which markets are closing soon so I can make last-minute trades before they close.
2. **As a Nostr user**, I want this command to work via DM or public mention, consistent with existing bot commands.
3. **As a Nostr user**, I want the output formatted similarly to the `/trending` command for consistency.

## Requirements

### Functional Requirements

1. **Command Interface**
   - Command name: `/closing` (supports both `/closing` and `closing` without slash)
   - No arguments required (similar to `/trending`)
   - Available via NIP-17 DM, NIP-04 DM, and public mentions

2. **Data Filtering**
   - Query Polymarket Gamma API for events
   - Filter to events with `end_date` ≤ 24 hours from now
   - Filter to events with `end_date` > current time (exclude already-closed markets)
   - Filter out `closed: true` events
   - Filter out `archived: true` events
   - Return up to 10 results

3. **Display Format**
   - Numbered list (1-10)
   - For each event show:
     - Event title
     - First market's "Yes" price (in cents)
     - 24-hour volume
     - Time until closing (e.g., "Closes in 5h 23m" or "Closes in 23h")
     - Event slug for reference
   - If no markets closing soon: "No markets closing in the next 24 hours."

4. **Sorting**
   - Primary: Soonest closing time first
   - Secondary: Volume (if multiple markets close at same time)

### Non-Functional Requirements

1. **Performance**: Response time should match `/trending` command (~1-2 seconds)
2. **Reliability**: Handle API errors gracefully with user-friendly messages
3. **Consistency**: Follow existing code patterns from `/trending` command
4. **Testability**: Include inline integration tests following project constitution

## Technical Approach

### API Strategy

Use existing `GammaClient` with a new method `list_closing_events(within_hours: u64, limit: usize)`:
- Call `/events` endpoint with appropriate filters
- Over-fetch (similar to `list_active_events`) to account for client-side filtering
- Filter results by time window in Rust code
- Sort by `end_date` ascending (closest first)
- Truncate to limit

### Command Handler

Create `src/commands/closing.rs` following the existing pattern:
- Signature: `pub async fn handle(gamma: &GammaClient) -> String`
- Call `gamma.list_closing_events(24, 10)`
- Format using new `format::format_closing_events(&events)`
- Error handling with user-friendly messages

### Formatter

Add `format_closing_events()` to `src/format.rs`:
- Similar structure to `format_trending_events()`
- Add time-until-close calculation logic using `chrono`
- Format duration in human-readable form

## Success Criteria

- [ ] `/closing` command returns markets closing within 24 hours
- [ ] Results are sorted by closing time (soonest first)
- [ ] Output format is clear and consistent with existing commands
- [ ] Empty state is handled gracefully
- [ ] API errors produce user-friendly messages
- [ ] Command is documented in help text
- [ ] Inline integration tests pass
- [ ] Works via DM and public mention

## Examples

### Example 1: Markets Found

**Input**: `/closing`

**Output**:
```
Markets Closing Soon:

1. Will Bitcoin hit $90K by March 31?
   Closes in 5h 23m  |  Vol 24h: $2.3M  |  Yes: 68¢
   Slug: bitcoin-90k-march-31

2. NBA Finals Winner 2026
   Closes in 18h 45m  |  Vol 24h: $5.7M  |  Yes: 42¢
   Slug: nba-finals-2026

[... up to 10 total]
```

### Example 2: No Markets Closing Soon

**Input**: `/closing`

**Output**:
```
No markets closing in the next 24 hours.
```

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| 24-hour window | Balances actionable timeframe with useful result count |
| Sort by time, not volume | Users care most about urgency for closing markets |
| Same format as `/trending` | Consistency reduces cognitive load |
| No customizable time window | Keeps interface simple; can be added later if needed |
| Over-fetch and client-filter | Ensures accuracy despite API pagination quirks |

## Out of Scope

- Custom time windows (e.g., `/closing 48h`)
- Pagination for >10 results
- Filtering by category or minimum volume
- Alert creation for closing markets (can use existing `/alert` commands)
- Historical data about past market closures

## Dependencies

- `polymarket-client-sdk` v0.4 (already in use)
- `chrono` crate (already in use)
- Existing `GammaClient` infrastructure

## References

- Existing `/trending` command: `src/commands/trending.rs`
- Event filtering logic: `src/polymarket/gamma.rs:50-82`
- Response formatting: `src/format.rs:216-258`
- Project constitution: `.specify/memory/constitution.md`
