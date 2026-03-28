# Polynostr Phase 2: Real-Time Price Alerts and Nostr DM Notifications

## Objective

Implement Phase 2 by adding user-managed, real-time price alerts that stream Polymarket updates and send notification messages back to users on Nostr (primarily private messages), while preserving the current Phase 1 command experience and reliability.

## Initial Assessment

### Project Structure Summary

- The current bot is a single-process async Rust service with Nostr event ingestion and command dispatch centered in `main.rs`, `bot.rs`, and `commands/*` (`src/main.rs:1-61`, `src/bot.rs:14-220`, `src/commands/mod.rs:1-35`).
  - **Implication**: Phase 2 alerting should be added as background async tasks + shared runtime state rather than introducing a second service.
- Polymarket API access is currently wrapped behind a Gamma client and a minimal Data client (`src/polymarket/gamma.rs:1-72`, `src/polymarket/data.rs:1-25`).
  - **Implication**: Alert trigger logic should use this wrapper boundary; if SDK streaming gaps exist, a transport abstraction can shield command/business layers from protocol specifics.
- Formatting is centralized and already aligned for plain-text Nostr output (`src/format.rs:1-256`).
  - **Implication**: Alert notifications should reuse this module to keep UX consistent and avoid fragmented message style.

### Relevant Files Examination (Source + Implication)

1. `src/bot.rs:24-45`, `src/bot.rs:88-174`, `src/bot.rs:176-220`
   - Current behavior handles inbound GiftWrap, NIP-04 DM, and mentions, then executes request-response replies only.
   - **Implication**: Phase 2 needs outbound notifications not tied to an inbound message, requiring durable recipient context and a scheduler/stream listener outside current handlers.
2. `src/commands/mod.rs:12-35`
   - Router only supports `search`, `price`, `market`, `trending`, `help`.
   - **Implication**: Add alert lifecycle commands (create/list/remove/pause/resume/test) and standardized argument parsing.
3. `src/config.rs:10-44`
   - Config currently includes keys and relays only.
   - **Implication**: Phase 2 needs env-configurable defaults (poll/stream strategy, debounce, max alerts per user, notification cooldown).
4. `src/polymarket/gamma.rs:20-70`
   - Wrapper supports market/event querying but no live stream API integration.
   - **Implication**: Introduce a market update source trait to support WebSocket-first and polling fallback without rewriting alert logic.
5. `src/main.rs:49-58`
   - Service wiring currently instantiates only Gamma client and starts bot loop.
   - **Implication**: Add alert manager startup, shared state injection into command handlers, and supervised background tasks.

### Prioritized Challenges and Risks

1. **State model for user alerts without a database** (highest priority)
   - Reason: Alert correctness depends on persistent identifiers, ownership checks, and last-trigger state. Stateless request/response is insufficient.
2. **Streaming reliability and reconnect behavior**
   - Reason: Phase 2 value depends on real-time continuity; dropped streams can silently break notifications.
3. **Nostr delivery model differences (NIP-17 vs NIP-04)**
   - Reason: The bot receives both and should reply safely; proactive notifications must choose a robust default channel per user/session.
4. **Rate limiting / anti-spam / duplicate notifications**
   - Reason: High-volume markets can generate excessive triggers, risking relay abuse and poor UX.
5. **Command ergonomics and validation complexity**
   - Reason: Alert rules (above/below/crossing/interval) require clear parsing and error messages to avoid support burden.

## Assumptions and Clarity Decisions

- Phase 2 remains **read-only** (no trading/authenticated CLOB actions).
- Alert subscriptions are persisted in a lightweight local store (preferred: SQLite via nostr-sdk ecosystem compatibility) to survive restarts.
- Delivery default is private DM; if user originally engages publicly only, the bot still attempts private notification first for anti-spam behavior.
- Real-time data source is WebSocket-first; on stream failure, degrade gracefully to bounded-interval polling.
- Initial alert types: price crossing (`above`, `below`) and percentage move over a rolling window; advanced conditions can be deferred.

## Implementation Plan

- [ ] **Step 1 (Status: Not Started): Define Phase 2 domain model for alerts and delivery state.**
  - Rationale: Introduce explicit entities (`AlertRule`, `AlertSubscription`, `TriggerState`, `DeliveryTarget`, `AlertEvent`) to remove ambiguity and prevent ad-hoc logic growth.
- [ ] **Step 2 (Status: Not Started): Add configuration surface for alert system behavior.**
  - Rationale: Extend config to include stream endpoint toggles, reconnect backoff bounds, per-user alert limits, cooldown/debounce, and fallback poll interval for operational control.
- [ ] **Step 3 (Status: Not Started): Introduce an alert repository interface and persistence backend.**
  - Rationale: Durable subscription state is required for restart recovery, idempotency, and ownership checks; an interface allows in-memory backend for tests.
- [ ] **Step 4 (Status: Not Started): Create a market update source abstraction (WebSocket + fallback polling).**
  - Rationale: Decouples data transport from trigger engine and enables resilient failover when SDK/endpoint behavior changes.
- [ ] **Step 5 (Status: Not Started): Build alert evaluation engine with deduplication and hysteresis.**
  - Rationale: Evaluates incoming ticks against rules while preventing repeated notifications from noisy price oscillations.
- [ ] **Step 6 (Status: Not Started): Implement outbound notification service integrated with existing Nostr client flow.**
  - Rationale: Notifications must be sent independently from inbound commands and support private delivery path consistency with current bot behavior.
- [ ] **Step 7 (Status: Not Started): Extend command router with alert lifecycle commands.**
  - Rationale: Add commands such as `alert add`, `alert list`, `alert remove`, `alert pause`, `alert resume`, `alert test` to make feature operable through existing UX surface.
- [ ] **Step 8 (Status: Not Started): Add strict command validation and user-facing error taxonomy.**
  - Rationale: Prevent invalid thresholds/slugs and provide deterministic guidance, reducing malformed subscriptions.
- [ ] **Step 9 (Status: Not Started): Wire application startup to initialize alert manager and supervised tasks.**
  - Rationale: `main` should bootstrap manager lifecycle and ensure clean shutdown/restart semantics.
- [ ] **Step 10 (Status: Not Started): Add observability for alert pipeline.**
  - Rationale: Structured logs and counters for stream status, trigger counts, send failures, and dropped notifications are necessary for reliability tuning.
- [ ] **Step 11 (Status: Not Started): Implement abuse controls and safety limits.**
  - Rationale: Enforce per-user limits, global throughput cap, and cooldown windows to avoid spam and relay degradation.
- [ ] **Step 12 (Status: Not Started): Execute end-to-end verification scenarios for normal, failure, and recovery paths.**
  - Rationale: Validates real-time correctness, resilience, and user command behavior under reconnects and API instability.

## Verification Criteria

- [ ] User can create, list, and remove alerts through commands with clear acknowledgements.
- [ ] Trigger notifications are delivered via DM when alert conditions are met.
- [ ] Duplicate notifications are suppressed during rapid oscillation around threshold.
- [ ] Alert subscriptions survive process restart and resume evaluation automatically.
- [ ] Stream disconnect causes automatic reconnect; prolonged failure triggers polling fallback.
- [ ] Invalid command syntax/threshold/slug returns actionable error messages.
- [ ] Per-user and global rate limits prevent message floods under volatile markets.
- [ ] Logs expose stream status, trigger events, send outcomes, and error causes.

## Potential Risks and Mitigations

1. **WebSocket API instability or SDK coverage gaps**  
   Mitigation: Implement transport abstraction with polling fallback and maintain endpoint adapters behind a common interface.
2. **Alert storm during high volatility**  
   Mitigation: Add hysteresis, cooldown windows, per-rule debounce, and per-user/global throughput caps.
3. **Incorrect recipient routing for proactive notifications**  
   Mitigation: Persist delivery preference + pubkey mapping at alert creation time and validate ownership on every mutation.
4. **State corruption or duplicate triggers after restart**  
   Mitigation: Persist last-evaluated marker and trigger watermark; run idempotency checks before sending.
5. **Operational blind spots in production**  
   Mitigation: Add structured diagnostics and alert-manager health telemetry from day one.

## Alternative Approaches

1. **In-memory subscriptions only**: Fastest to implement but loses all alerts on restart and weakens reliability expectations.
2. **Polling-only architecture**: Simpler operational model but lower timeliness and higher API usage; acceptable fallback, not ideal primary.
3. **Relay-public notifications instead of DM-first**: Better discoverability but risks feed spam and poor privacy; keep as optional future mode.
4. **Single generic `subscribe <slug>` command**: Minimal UX but ambiguous conditions; explicit alert verbs are clearer and safer.
