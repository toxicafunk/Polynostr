# Polynostr: Polymarket + Nostr Integration

## Objective

Build a nostr bot in Rust that bridges Polymarket prediction market data into the nostr protocol. The bot listens for DMs and mentions from nostr users, parses commands, queries the Polymarket APIs, and replies with formatted prediction market data. The project uses two mature async Rust SDKs — `nostr-sdk` and `polymarket-client-sdk` — that compose naturally in a single `tokio` runtime.

## Architecture

```
                    ┌─────────────────┐
                    │  Nostr Relays    │
                    │ (wss://...)      │
                    └────────┬────────┘
                             │ WebSocket
                             ▼
┌──────────────────────────────────────────────┐
│              polynostr bot                    │
│                                              │
│  ┌──────────────┐     ┌───────────────────┐  │
│  │  nostr-sdk   │     │ polymarket-client  │  │
│  │  Client      │◄───►│ -sdk              │  │
│  │              │     │                   │  │
│  │ • subscribe  │     │ • Gamma API       │  │
│  │ • publish    │     │ • Data API        │  │
│  │ • DMs        │     │ • CLOB (read)     │  │
│  │              │     │ • WebSocket       │  │
│  └──────────────┘     └───────────────────┘  │
│                                              │
│  ┌──────────────────────────────────────┐    │
│  │  Command Router                      │    │
│  │  "price <market>" → fetch & reply    │    │
│  │  "search <query>" → search & reply   │    │
│  │  "trending"       → list & reply     │    │
│  │  "subscribe <id>" → stream updates   │    │
│  └──────────────────────────────────────┘    │
└──────────────────────────────────────────────┘
```

## Why a Bot (Not a Web App)

1. **SDK Compatibility**: Both `nostr-sdk` and `polymarket-client-sdk` are async-first, tokio-based Rust libraries that compose perfectly in a single async process. A web app would introduce WASM compilation, browser wallet bridging, and UI framework overhead before any useful integration logic.
2. **Nostr-Native UX**: Nostr users are already inside clients (Damus, Amethyst, Primal). A bot delivers information where they are — no context-switching to another website.
3. **No Auth Needed for Read APIs**: Polymarket's Gamma API and Data API are fully public. A read-only bot works immediately with zero wallet/key management.
4. **Clean Trust Boundary for Trading**: If trading is added later, EVM keys (Polygon wallet) live server-side in the bot. Users authorize via signed nostr events. This is simpler and more secure than browser-side EVM wallet bridging.
5. **Polymarket Already Has a Web UI**: The value isn't recreating their dashboard — it's making their data accessible inside nostr.

## SDK Reference

### Polymarket Rust SDK (`polymarket-client-sdk v0.3`)
- Crate: https://crates.io/crates/polymarket-client-sdk
- Repo: https://github.com/Polymarket/rs-clob-client
- Three APIs:
  - **Gamma API** (`https://gamma-api.polymarket.com`) — markets, events, search, metadata (public, no auth)
  - **Data API** (`https://data-api.polymarket.com`) — positions, trades, leaderboards (public, no auth)
  - **CLOB API** (`https://clob.polymarket.com`) — orderbook, pricing, order management (trading requires EVM auth)
- Feature flags: `clob`, `ws`, `data`, `gamma`, `bridge`, `rtds`, `rfq`, `heartbeats`, `ctf`
- Uses `alloy::signers::Signer` for EVM authentication (Phase 4+ only)

### Nostr Rust SDK (`nostr-sdk v0.39`)
- Crate: https://crates.io/crates/nostr-sdk
- Repo: https://github.com/rust-nostr/nostr
- Full nostr protocol support: relay connections, event publishing, subscriptions, encrypted DMs
- Supports NIP-01, NIP-04 (encrypted DMs), NIP-17 (private DMs), NIP-28 (public chat), 50+ NIPs
- WASM support (for future web app if needed)
- Database backends: LMDB, SQLite, in-memory

## Full Project Roadmap

| Phase | Features | Complexity | Auth Required |
|-------|----------|------------|---------------|
| **Phase 1** | Market search, price queries, trending markets, help command | Low | None (Gamma + Data APIs are public) |
| **Phase 2** | Real-time price alerts via WebSocket streaming + nostr DM notifications | Medium | None |
| **Phase 3** | User portfolio tracking (Data API positions/trades lookup by address) | Medium | None |
| **Phase 4** | Trading commands with server-side EVM signer (CLOB authenticated) | High | EVM wallet (alloy signer) |
| **Phase 5** | Optional web dashboard (Axum backend + htmx/Leptos frontend) | High | Nostr + EVM |

---

## Phase 1 Implementation Plan

### Status: Completed

### Objective

Deliver a working nostr bot that responds to DMs and mentions with Polymarket prediction market data using the public Gamma and Data APIs. No authentication, no database, no trading — purely a read-only information bridge.

### Project Structure

```
polynostr/
├── Cargo.toml
├── .env.example
├── src/
│   ├── main.rs              # Entry point: init clients, spawn event loop
│   ├── config.rs            # Configuration (relay URLs, bot keys, env vars)
│   ├── bot.rs               # Core event loop: subscribe, dispatch, reply
│   ├── commands/
│   │   ├── mod.rs           # Command parser + router
│   │   ├── search.rs        # "search <query>" — search markets via Gamma API
│   │   ├── price.rs         # "price <market_slug>" — current price for a market
│   │   ├── trending.rs      # "trending" — list top active markets
│   │   ├── market.rs        # "market <slug>" — detailed market info
│   │   └── help.rs          # "help" — list available commands
│   ├── polymarket/
│   │   ├── mod.rs           # Re-exports
│   │   ├── gamma.rs         # Gamma API wrapper (markets, events, search)
│   │   └── data.rs          # Data API wrapper (open interest, volume)
│   └── format.rs            # Format API responses into nostr-friendly plain text
```

### Dependencies (Cargo.toml)

```toml
[package]
name = "polynostr"
version = "0.1.0"
edition = "2021"

[dependencies]
nostr-sdk = "0.39"
polymarket-client-sdk = { version = "0.3", features = ["gamma", "data"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenvy = "0.15"
```

### Environment Variables (.env.example)

```
NOSTR_SECRET_KEY=hex-or-bech32-secret-key
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.nostr.band
```

### Implementation Steps

- [ ] **Step 1: Scaffold the project** — Initialize the Cargo project with the directory structure above, create `Cargo.toml` with all dependencies, and create the `.env.example` file. Every `mod.rs` and source file should exist (even if just stubs) so the project compiles from the start.

- [ ] **Step 2: Implement `config.rs`** — Define a `Config` struct containing the bot's nostr `Keys` (parsed from `NOSTR_SECRET_KEY` env var) and a `Vec<String>` of relay URLs (parsed from `NOSTR_RELAYS` env var, comma-separated). Load env vars via `dotenvy::dotenv()`. Provide sensible defaults for relays if the env var is missing (e.g., `wss://relay.damus.io`). Return a clear error if the secret key is missing or unparseable.

- [ ] **Step 3: Implement `main.rs`** — Initialize `tracing_subscriber` with an env filter (defaulting to `info`). Load `Config`. Create a nostr `Client` from `nostr-sdk` using the bot's keys, add all configured relays, and call `client.connect().await`. Create a `polymarket-client-sdk` Gamma client (unauthenticated, just needs the base URL `https://gamma-api.polymarket.com`). Log the bot's public key in bech32 format so the operator knows where to send test messages. Call `bot::run(client, gamma_client).await`.

- [ ] **Step 4: Implement `bot.rs` — Core event loop** — Subscribe to nostr events using two filters: (a) Kind 4 (NIP-04 encrypted DMs) where `p` tag matches the bot's pubkey, and (b) Kind 1 (text notes) where `p` tag mentions the bot's pubkey. Loop on incoming events via `client.notifications()`. For each received event: decrypt if DM (using `nip04::decrypt`), extract the text content, pass it to `commands::handle_command()`, and send the response back. If the incoming event was a DM, reply as a DM. If it was a public mention, reply as a public text note with the appropriate `e` and `p` tags for threading.

- [ ] **Step 5: Implement `commands/mod.rs` — Command parser and router** — Parse the incoming message text: split on whitespace, take the first token as the command name (case-insensitive), and pass the remaining tokens as arguments. Route to the appropriate handler async function. Return a `String` result. Unrecognized commands should return the help text. Commands to support: `search`, `price`, `trending`, `market`, `help`.

- [ ] **Step 6: Implement `polymarket/gamma.rs` — Gamma API wrapper** — Create thin async wrapper functions around the `polymarket-client-sdk` Gamma client: `search_markets(query: &str, limit: usize)` returns a vec of market summaries (title, slug, outcome prices). `get_market_by_slug(slug: &str)` returns full market details (title, description, outcomes with prices, volume, end date, resolution source). `list_active_events(limit: usize)` returns active/trending events with their associated markets. Handle API errors gracefully — return user-friendly error strings rather than panicking.

- [ ] **Step 7: Implement `polymarket/data.rs` — Data API wrapper** — Create thin async wrapper functions around the Data API client: `get_open_interest(condition_id: &str)` returns the total open interest for a market. `get_volume(condition_id: &str)` returns volume data. These supplement the Gamma data with trading metrics. Handle errors gracefully.

- [ ] **Step 8: Implement `format.rs` — Response formatter** — Create formatting functions that take API response structs and produce clean plain-text strings suitable for nostr. No HTML, no markdown links (most nostr clients render plain text). Use Unicode box-drawing or simple separators for structure. Example output for a price query:
  ```
  US Presidential Election 2028

  Yes: 52¢  |  No: 48¢
  Volume: $12.4M  |  Liquidity: $2.1M
  Ends: Nov 3, 2028

  polymarket.com/event/us-presidential-election-2028
  ```
  For search results, format as a numbered list with title and price. Truncate long descriptions. Include the market slug so users can run follow-up commands.

- [ ] **Step 9: Implement `commands/search.rs`** — Handle the `search <query>` command. Call `gamma.search_markets(query, 5)`. Format the results using `format.rs` as a numbered list: each entry shows the market title, current Yes price as a percentage, and the slug. If no results found, return a helpful message suggesting alternative search terms.

- [ ] **Step 10: Implement `commands/price.rs`** — Handle the `price <market_slug>` command. Call `gamma.get_market_by_slug(slug)`. Format the response showing: market title, Yes/No prices (as cents), volume, and a link. If the slug is not found, return an error message suggesting the user run `search` first.

- [ ] **Step 11: Implement `commands/trending.rs`** — Handle the `trending` command (no arguments). Call `gamma.list_active_events(10)`. Format as a numbered list of the top 10 active events with their market titles and current prices. Group by event if an event has multiple markets (e.g., "Who will win?" with multiple candidates).

- [ ] **Step 12: Implement `commands/market.rs`** — Handle the `market <slug>` command. Call `gamma.get_market_by_slug(slug)` and optionally `data.get_open_interest()`. Format a detailed view: title, full description (truncated to ~500 chars), all outcome prices, volume, open interest, end date, resolution source, and a Polymarket link.

- [ ] **Step 13: Implement `commands/help.rs`** — Return a static help string listing all available commands with usage examples:
  ```
  Polynostr Bot — Polymarket data on Nostr

  Commands:
    search <query>    Search for prediction markets
    price <slug>      Get current price for a market
    market <slug>     Detailed market information
    trending          Top active markets
    help              Show this message

  Examples:
    search bitcoin
    price will-bitcoin-hit-100k
    trending
  ```

- [ ] **Step 14: End-to-end testing** — Run the bot locally pointed at a public relay. Use a separate nostr client (e.g., https://snort.social or another instance of nostr-sdk) to send DMs and mentions to the bot's pubkey. Verify each command produces correct, well-formatted output. Test error cases: invalid slugs, empty search results, malformed commands.

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| DMs (NIP-04) as primary interface | Private, no spam on public feeds, natural 1:1 interaction |
| Also support public mentions | Allows use in group settings; reply publicly so others benefit |
| Unauthenticated Polymarket APIs only | No EVM keys needed; Gamma + Data APIs are fully public |
| Stateless per-request | No database needed; each command is independent. Simplifies Phase 1. |
| Plain text formatting | Universal compatibility across all nostr clients (Damus, Amethyst, Primal, web clients) |
| Graceful error handling | API failures return user-friendly messages, never crash the bot |
| Logging via `tracing` | Structured, filterable logs for debugging relay connections and API calls |

### Verification Criteria

- Bot connects to configured nostr relays and stays connected
- Bot's bech32 public key is logged on startup for discoverability
- Sending a DM with `help` returns the command list
- Sending a DM with `search bitcoin` returns a list of matching Polymarket markets
- Sending a DM with `price <valid-slug>` returns current Yes/No prices
- Sending a DM with `trending` returns top active markets
- Sending a DM with `market <valid-slug>` returns detailed market info
- Sending a public text note mentioning the bot triggers a public reply
- Invalid commands return the help text
- Invalid slugs return a user-friendly error message
- Bot recovers gracefully from Polymarket API errors (timeout, 5xx, etc.)
- Bot recovers from relay disconnections (nostr-sdk handles reconnection automatically)

### Potential Risks and Mitigations

1. **Polymarket API rate limits or geo-blocking**
   Mitigation: The Gamma and Data APIs are public but may have undocumented rate limits. Add retry logic with exponential backoff. Consider caching frequent queries (e.g., trending markets) with a short TTL. Polymarket has geographic restrictions — the bot server must be in an allowed region.

2. **`polymarket-client-sdk` is relatively new (v0.3) and may have API gaps**
   Mitigation: If the SDK doesn't expose certain Gamma/Data endpoints, fall back to direct `reqwest` HTTP calls to the REST API. The API surface is well-documented REST.

3. **NIP-04 encrypted DMs are considered deprecated in favor of NIP-17**
   Mitigation: Start with NIP-04 (widely supported by all clients today). Add NIP-17 support as a follow-up. The `nostr-sdk` supports both.

4. **Nostr relay reliability**
   Mitigation: Configure multiple relays (3+). `nostr-sdk` handles automatic reconnection. Log relay connection state changes for observability.

5. **Bot gets spammed with high message volume**
   Mitigation: Add per-pubkey rate limiting in Phase 2. For Phase 1, low traffic is expected during development/testing.

### Alternative Approaches Considered

1. **Web App (Rust WASM + Leptos/Dioxus)**: Rejected for Phase 1 — adds UI framework complexity, WASM build pipeline, and browser wallet management without clear UX advantage over the bot approach. Can be layered on top in Phase 5.

2. **Direct REST calls instead of `polymarket-client-sdk`**: Rejected — the SDK provides typed request/response structs, proper error handling, and built-in serde support. Only fall back to raw REST if the SDK is missing needed endpoints.

3. **Public notes only (no DMs)**: Rejected as primary interface — public commands would spam relay feeds. DMs are the natural primary channel, with public mentions as a secondary option.
