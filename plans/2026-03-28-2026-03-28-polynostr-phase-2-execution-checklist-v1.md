# Polynostr Phase 2.1: Execution Checklist (Implementation-Agent Ready)

## Objective

Translate the approved Phase 2 strategy into an executable, file-targeted checklist for implementing real-time price alerts and Nostr DM notifications.

## Source References

- Approved Phase 2 strategy: `plans/2026-03-28-2026-03-28-polynostr-phase-2-realtime-alerts-plan-v1.md:57-113`
- Runtime wiring and bot loop: `src/main.rs:27-58`, `src/bot.rs:24-45`
- Inbound handlers: `src/bot.rs:88-220`
- Router: `src/commands/mod.rs:12-35`
- Config baseline: `src/config.rs:5-44`
- Polymarket wrappers: `src/polymarket/gamma.rs:12-72`, `src/polymarket/data.rs:10-25`
- Formatting baseline: `src/format.rs:22-256`

## Implementation Plan

- [x] **Task 1 (Status: Completed): Introduce alert domain module and types.**
  - [x] Add `alerts` module namespace and wire it through crate composition.
  - [x] Define `AlertRule`, `AlertSubscription`, `TriggerState`, `DeliveryTarget`, `AlertEvent`, and identifier types.
  - Rationale: Establishes explicit state model required by Phase 2.

- [x] **Task 2 (Status: Completed): Extend config for alert system controls.**
  - [x] Add env-configurable settings for stream toggle, polling fallback interval, reconnect backoff, per-user caps, and cooldown/debounce.
  - [x] Validate config bounds at startup.
  - Rationale: Enables safe runtime behavior and operational tuning.

- [x] **Task 3 (Status: Completed): Implement repository abstraction and persistence backends.**
  - [x] Define an `AlertRepository` trait for CRUD, ownership checks, and trigger watermark updates.
  - [x] Provide in-memory backend for tests/dev.
  - [x] Provide durable backend (SQLite preferred) for restart-safe subscriptions.
  - Rationale: Durable state is mandatory for reliability and restart recovery.

- [x] **Task 4 (Status: Completed): Implement market update ingestion abstraction.**
  - [x] Define a `MarketUpdateSource` interface.
  - [x] Add WebSocket adapter as primary update source.
  - [x] Add polling adapter as failure fallback.
  - Rationale: Decouples transport from alert logic and improves resilience.

- [x] **Task 5 (Status: Completed): Build alert evaluation engine.**
  - [x] Support initial rule types: threshold crossing (`above`, `below`) and percentage move.
  - [x] Add deduplication, hysteresis, and cooldown to suppress noise-triggered repeats.
  - Rationale: Ensures trigger correctness and anti-spam behavior.

- [x] **Task 6 (Status: Completed): Add outbound alert notification service.**
  - [x] Implement proactive DM notifications independent of inbound request-response handlers.
  - [x] Standardize delivery behavior with NIP-17-first compatibility strategy.
  - Rationale: Phase 2 requires asynchronous outbound delivery.

- [x] **Task 7 (Status: Completed): Expand command surface for alert lifecycle.**
  - [x] Add routing for: `alert add`, `alert list`, `alert remove`, `alert pause`, `alert resume`, `alert test`.
  - [x] Implement dedicated command handlers.
  - [x] Update help output with alert usage examples.
  - Rationale: Users need complete in-band lifecycle control from Nostr.

- [x] **Task 8 (Status: Completed): Add strict command validation and user-facing error taxonomy.**
  - [x] Validate slug/rule/threshold syntax and bounds.
  - [x] Map transport/repo/evaluator failures to stable user-readable errors.
  - Rationale: Reduces malformed subscriptions and improves UX clarity.

- [x] **Task 9 (Status: Completed): Wire startup and supervision.**
  - [x] Initialize repository, update source, evaluator, notifier, and manager in `main`.
  - [x] Inject alert manager context into bot/command paths.
  - [x] Add graceful shutdown/cancellation behavior for background tasks.
  - Rationale: Ensures production-safe lifecycle management.

- [x] **Task 10 (Status: Completed): Extend output formatting for alert UX.**
  - [x] Add formatter outputs for alert acknowledgements, list views, and trigger notifications.
  - [x] Preserve plain-text compatibility and existing style consistency.
  - Rationale: Keeps output coherent across all commands and clients.

- [x] **Task 11 (Status: Completed): Add observability and abuse controls.**
  - [x] Add structured logs for stream state, trigger outcomes, send results, retries, and fallbacks.
  - [x] Enforce per-user alert limits and outbound notification throughput caps.
  - Rationale: Needed for reliability, safety, and operations.

- [x] **Task 12 (Status: Completed): Execute verification and recovery testing.**
  - [x] Unit tests: parser/evaluator edge cases.
  - [x] Integration tests: repository persistence and restart behavior.
  - [x] End-to-end tests: command → subscription → trigger → DM notification.
  - Rationale: Confirms Phase 2 criteria before completion.

## Verification Criteria

- [x] Users can create, list, pause/resume, and remove alerts with clear acknowledgements.
- [x] Alert triggers deliver private notifications when conditions are met.
- [x] Duplicate notifications are suppressed during threshold oscillation.
- [x] Subscriptions survive restart and resume evaluation automatically.
- [x] Stream failure triggers reconnect and bounded fallback polling.
- [x] Invalid command syntax and invalid rule values produce actionable errors.
- [x] Per-user/global limits prevent notification floods.
- [x] Logs expose health, trigger flow, and delivery outcomes.

## Potential Risks and Mitigations

1. **WebSocket instability or SDK/API mismatch**  
   Mitigation: Keep transport adapters behind one interface and always support polling fallback.
2. **High-volatility alert storms**  
   Mitigation: Hysteresis, cooldown, and throughput caps.
3. **Wrong recipient routing for proactive notifications**  
   Mitigation: Persist and validate delivery target ownership on every alert mutation.
4. **Restart duplication / stale trigger state**  
   Mitigation: Persist trigger watermark and run idempotency checks before notification send.
5. **Insufficient production visibility**  
   Mitigation: Structured logs and alert-manager health instrumentation from initial implementation.

## Alternative Approaches

1. **In-memory only subscriptions**: Lower implementation effort, but no restart durability.
2. **Polling-only ingestion**: Simpler, but less real-time and potentially higher API cost.
3. **Public note notifications**: More discoverable, but higher feed noise and lower privacy.
