# Command Interface Contract: `/closing`

**Feature**: 001-closing-events  
**Date**: 2026-03-31  
**Type**: Bot Command Interface

## Overview

This contract defines the user-facing interface for the `/closing` command in the Polynostr bot. This is a public command interface accessible via Nostr DMs (NIP-17/NIP-04) and public mentions.

---

## Command Signature

```
/closing
```

**Alternative Forms**:
- `closing` (without slash prefix)

**Arguments**: None

**Access Control**: Public (no authentication required)

**Rate Limiting**: Inherits bot-level rate limiting (not specific to this command)

---

## Input Contract

### Valid Inputs

| Input | Description | Example |
|-------|-------------|---------|
| `/closing` | Standard invocation | `/closing` |
| `closing` | No-slash variant | `closing` |
| `/closing <ignored-args>` | Extra args ignored | `/closing please` |

### Invalid Inputs

**None** — Command accepts any input starting with `/closing` or `closing`. Extra arguments are silently ignored.

---

## Output Contract

### Success Response (Markets Found)

**Format**: Plain text (UTF-8)

**Structure**:
```
Markets Closing Soon:

1. <Event Title>
   Closes in <duration>  |  Vol 24h: <volume>  |  Yes: <price>
   Slug: <slug>

2. <Event Title>
   Closes in <duration>  |  Vol 24h: <volume>  |  Yes: <price>
   Slug: <slug>

...

[Up to 10 entries]
```

**Field Specifications**:

| Field | Format | Example | Notes |
|-------|--------|---------|-------|
| **Event Title** | Plain text | `"Will Bitcoin hit $100K?"` | Market question/title |
| **Duration** | `Xh Ym` or `Xm` | `"5h 23m"`, `"45m"` | Time until market closes |
| **Volume** | `$X.XM` or `$X.XK` | `"$2.3M"`, `"$500K"` | 24-hour trading volume |
| **Price** | `XX¢` | `"68¢"` | "Yes" outcome price (0-100) |
| **Slug** | Kebab-case | `"bitcoin-100k-march"` | Market identifier |

**Ordering**: Markets sorted by closing time (soonest first)

**Limit**: Maximum 10 markets displayed

**Example**:
```
Markets Closing Soon:

1. Will Bitcoin hit $90K by March 31?
   Closes in 5h 23m  |  Vol 24h: $2.3M  |  Yes: 68¢
   Slug: bitcoin-90k-march-31

2. NBA Finals Winner 2026
   Closes in 18h 45m  |  Vol 24h: $5.7M  |  Yes: 42¢
   Slug: nba-finals-2026
```

---

### Success Response (No Markets Found)

**Format**: Plain text (UTF-8)

**Output**:
```
No markets closing in the next 24 hours.
```

**When**: Returned when zero markets meet the criteria (closing within 24h)

---

### Error Response

**Format**: Plain text (UTF-8)

**Output**:
```
Error fetching closing markets: <error-description>
```

**When**: Returned on API failures (network, timeout, server error)

**Example Error Messages**:
- `"Error fetching closing markets: network timeout"`
- `"Error fetching closing markets: API unavailable"`
- `"Error fetching closing markets: invalid response"`

**Guarantees**: Error messages never contain:
- Sensitive data
- Stack traces
- Internal implementation details

---

## Behavior Specification

### Time Window
- **Window**: 24 hours from current time
- **Inclusive**: Markets closing in exactly 24h 0m 0s are included
- **Exclusive**: Markets already closed or closing >24h are excluded

### Filtering Rules
1. `end_date` must be present
2. `end_date` must be > current time (not closed yet)
3. `end_date` must be ≤ current time + 24 hours
4. `closed` must not be `true`
5. `archived` must not be `true`

### Sorting Priority
1. **Primary**: End date (ascending - soonest first)
2. **Secondary**: 24-hour volume (descending - highest first)

### Display Logic
- Show first 10 markets only (no pagination in v1)
- Show first market's "Yes" price for multi-market events
- Format durations as `"Xh Ym"` (omit hours if <1h)
- Format volumes with `M`/`K` suffixes
- Format prices as cents (0-100 range)

---

## Delivery Channels

### NIP-17 Gift Wrap DM
- **When**: User sends DM using NIP-17 protocol
- **Response**: Encrypted private message (NIP-17)

### NIP-04 Encrypted DM
- **When**: User sends DM using legacy NIP-04 protocol
- **Response**: Encrypted direct message (NIP-04)

### Public Mention
- **When**: User mentions bot in public text note
- **Response**: Public reply note with `e` and `p` tags (threaded)

**Note**: Response format is identical across all channels (plain text)

---

## Idempotency

**Guarantee**: Multiple invocations return fresh data each time

**Not Cached**: Results are not cached; each call queries Polymarket API

**Stateless**: No side effects; command does not modify any state

---

## Performance Characteristics

**Expected Latency**: 200-600ms end-to-end

**Timeout**: Inherits bot's HTTP client timeout (default: 30s)

**Retry Behavior**: No automatic retries (user must re-invoke)

---

## Versioning

**Version**: 1.0.0 (initial implementation)

**Backward Compatibility**: N/A (new command)

**Future Changes**:
- May add pagination in v2 (`/closing page 2`)
- May add custom time windows in v2 (`/closing 48h`)
- May add category filters in v2 (`/closing crypto`)

**Deprecation Policy**: If command is deprecated, help text will be updated and warnings issued for 30 days before removal

---

## Integration Points

### Upstream Dependencies
- **Polymarket Gamma API**: `/events` endpoint
- **Nostr Relays**: Message delivery infrastructure

### Downstream Consumers
- **Nostr Users**: Human users querying market data
- **Future**: Could be integrated into alert workflows

---

## Examples

### Example 1: Standard Success

**Input**:
```
/closing
```

**Output**:
```
Markets Closing Soon:

1. US Presidential Election 2028
   Closes in 2h 15m  |  Vol 24h: $8.2M  |  Yes: 52¢
   Slug: us-election-2028

2. Will OpenAI release GPT-5 this year?
   Closes in 6h 0m  |  Vol 24h: $3.1M  |  Yes: 34¢
   Slug: openai-gpt5-2026

3. Bitcoin above $80K on March 31?
   Closes in 23h 59m  |  Vol 24h: $12.4M  |  Yes: 78¢
   Slug: btc-80k-march-31
```

---

### Example 2: No Markets

**Input**:
```
/closing
```

**Output**:
```
No markets closing in the next 24 hours.
```

---

### Example 3: API Error

**Input**:
```
/closing
```

**Output**:
```
Error fetching closing markets: network timeout
```

---

### Example 4: Arguments Ignored

**Input**:
```
/closing please show me markets
```

**Output**:
```
[Same as Example 1 - arguments ignored]
```

---

## Help Text Entry

The `/help` command will include this entry:

```
/closing                        Markets closing in next 24 hours
```

**Full Description** (in detailed help):
```
/closing - Show markets closing within 24 hours

Get a list of prediction markets that will close in the next 24 hours,
sorted by closing time (soonest first). Useful for finding time-sensitive
trading opportunities.

Example: /closing
```

---

## Testing Contract

### Acceptance Criteria

1. ✅ Command responds to `/closing` and `closing`
2. ✅ Returns markets closing in <24h only
3. ✅ Sorts by closing time (soonest first)
4. ✅ Displays up to 10 results
5. ✅ Shows duration, volume, price, slug for each market
6. ✅ Handles empty results gracefully
7. ✅ Handles API errors gracefully
8. ✅ Works via NIP-17 DM, NIP-04 DM, and public mention
9. ✅ Response latency <2 seconds under normal conditions
10. ✅ No sensitive data in error messages

---

## Compliance

### Accessibility
- **Plain Text**: Universal compatibility with all Nostr clients
- **No Media**: No images, videos, or embeds required
- **Readable**: Clear, concise formatting

### Privacy
- **No User Data**: Command does not collect or store user information
- **Public API**: Only queries public Polymarket data
- **No Tracking**: No analytics or telemetry

### Security
- **Read-Only**: No mutations or side effects
- **No Auth**: No credentials required or exposed
- **Input Sanitization**: Extra arguments safely ignored

---

**Status**: ✅ Contract Defined — Interface specification complete
