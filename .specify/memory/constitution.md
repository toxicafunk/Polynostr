<!--
Sync Impact Report:
- Version change: none → 1.0.0
- Modified principles: N/A (initial constitution)
- Added sections: All (initial constitution)
- Removed sections: N/A
- Templates requiring updates:
  ✅ plan-template.md - Constitution Check section already references this file
  ✅ spec-template.md - No changes needed, requirements align with principles
  ✅ tasks-template.md - Test-first sections align with principles
- Follow-up TODOs: None
-->

# Polynostr Constitution

## Core Principles

### I. Trait-Based Abstraction for Testability

All core domain components MUST be defined as traits to enable:
- **Dependency injection**: Concrete implementations passed at runtime via constructors
- **Test doubles**: Easy creation of stub/mock implementations for isolated testing
- **Future extensibility**: New implementations without modifying consumers

**Examples**: `AlertRepository`, `MarketUpdateSource`, `AlertDelivery`

**Rationale**: Enables comprehensive testing of business logic without external dependencies (databases, APIs, network). Facilitates graceful degradation (WebSocket → polling fallback) and supports multiple implementation strategies.

### II. Async-First Concurrency

All I/O operations MUST use async/await patterns:
- Database operations return `async fn`
- HTTP requests via async clients
- Nostr protocol interactions async throughout
- Background tasks spawned with `tokio::spawn`

**State sharing**: Use `Arc<T>` for immutable shared data, `Arc<RwLock<T>>` for mutable shared state.

**Rationale**: Maximizes scalability by avoiding thread blocking. Enables concurrent message handling and alert evaluation without resource contention. The bot's event-driven nature requires non-blocking I/O to maintain responsiveness.

### III. Fail-Fast Validation

Input validation MUST occur at system boundaries before persistence or processing:
- Configuration validated at startup with descriptive errors
- Command arguments parsed and validated before handler execution
- Alert rules validated before database persistence
- Domain constraints enforced in parsing layer

**Error handling**: Return `Result<T, E>` everywhere; no panics in production code. Provide dual error messages (developer detail, user-friendly guidance).

**Rationale**: Early validation prevents invalid state propagation. Clear error messages at boundaries improve debugging and user experience. Type-safe parsing eliminates runtime surprises.

### IV. Inline Integration Testing (NON-NEGOTIABLE)

Complex domain logic MUST include inline integration tests in the same file:
- Tests use `#[cfg(test)]` modules
- Tests marked with `#[tokio::test]` for async execution
- Stub implementations created for external dependencies
- Tests verify end-to-end flows with real components

**Examples**: `src/alerts/manager.rs:229-426`, `src/alerts/evaluator.rs:82-200`, `src/alerts/parser.rs:60-103`

**Coverage requirements**:
- Business logic evaluation (alert rules, cooldown, hysteresis)
- State transitions (alert lifecycle)
- Repository persistence (CRUD operations)
- Manager orchestration (command → trigger → notification flows)

**Rationale**: Co-located tests ensure maintainability. Integration tests catch boundary issues that unit tests miss. Testing actual components (not just mocks) validates real behavior. Inline placement encourages developers to update tests with code changes.

### V. Single Responsibility at Module Level

Each module MUST have one clear purpose with explicit boundaries:
- **bot.rs**: Protocol handling only, delegates to commands
- **commands/**: Pure command handlers, no I/O logic
- **alerts/evaluator.rs**: Pure business logic, stateless evaluation
- **alerts/manager.rs**: Orchestration only, delegates to repository/source/notifier
- **format.rs**: Presentation only, no business logic
- **config.rs**: Configuration loading and validation

**Anti-patterns**: God objects, cross-cutting I/O mixed with logic, command handlers doing persistence directly.

**Rationale**: Clear responsibilities simplify testing, debugging, and maintenance. Dependency direction flows inward (infrastructure → application → domain). Changes to formatting don't affect business logic.

### VI. Privacy and Security by Design

User privacy MUST be protected through protocol choices and authorization:
- Support modern privacy protocols first (NIP-17 Gift Wrap preferred over NIP-04)
- Respect user's chosen communication channel
- Verify ownership before mutations (e.g., `list_by_pubkey` filter in repository)
- Rate limiting per user to prevent abuse

**Data minimization**: Store only essential data (alert subscriptions, trigger state). No unnecessary user profiling.

**Rationale**: Nostr's decentralized nature requires client-side privacy enforcement. NIP-17 provides perfect forward secrecy. Authorization at data layer prevents privilege escalation. Rate limiting protects both users and infrastructure.

### VII. Configuration Over Code

Runtime behavior MUST be configurable via environment variables:
- Required: `NOSTR_SECRET_KEY` (validated at startup)
- Optional: Alert polling intervals, rate limits, database path
- Sensible defaults provided for all optional settings
- Validation ensures constraints (e.g., poll interval ≥3s to prevent API spam)

**Configuration management**: Single `Config` struct loaded at startup, passed to components via constructors.

**Rationale**: Enables deployment flexibility without recompilation. Supports different environments (dev/staging/production). Allows operators to tune behavior (polling frequency, rate limits) based on observed load.

### VIII. Graceful Degradation

Systems MUST provide fallback mechanisms when preferred approaches fail:
- WebSocket market updates with HTTP polling fallback
- Multiple relay connections (failure of one doesn't stop bot)
- Per-alert error isolation (one failing alert doesn't stop others)

**Logging**: Failed operations logged with context but don't crash the system.

**Rationale**: Increases reliability in distributed systems. External API availability should not cause total system failure. Users receive partial functionality rather than nothing.

## Data Management

### Persistence Strategy

**SQLite for application state**: Alerts, subscriptions, trigger history persisted to SQLite with:
- Schema initialization idempotent (`CREATE TABLE IF NOT EXISTS`)
- Proper indexing (pubkey, status) for query performance
- Connection wrapped in `Arc<Mutex<Connection>>` for async safety
- Flattened schema (denormalized) for query simplicity

**In-memory for runtime state**: Notification rate windows, active slug tracking rebuilt from persistent state on startup.

**Rationale**: SQLite simplifies deployment (no separate DB server). Single-instance design acceptable for bot workload. Synchronous SQLite sufficient for current scale (read-heavy, infrequent writes).

### State Management

**Stateful evaluation**: Alert trigger state includes:
- `last_seen_price`: Latest observed price (enables crossing detection)
- `last_triggered_price`: Price at last trigger (enables hysteresis)
- `last_triggered_at`: Timestamp of last trigger (enables cooldown)

**State synchronization**: Every evaluation updates persistent state, even if not triggered. In-memory caches (`subscribed_slugs`, `notif_window`) rebuilt from database on restart.

**Rationale**: Stateful evaluation enables sophisticated rules (crossing thresholds, preventing oscillation). Persistent state survives restarts without losing trigger history. In-memory caches optimize performance for frequently accessed data.

## Development Workflow

### Testing Discipline

**Test organization**:
- Inline tests in same file as implementation (`#[cfg(test)]` modules)
- Test infrastructure (stubs) co-located with usage
- Integration tests verify end-to-end flows
- Property-based testing not currently used (potential future addition)

**Test requirements**:
- Async tests use `#[tokio::test]` macro
- Tests use clear assertion messages via `expect()`
- Repository tests use temporary databases or in-memory implementations
- Manager tests use stub dependencies (DeliveryStub, SourceStub)

**Coverage gaps identified**: `bot.rs`, `commands/*`, `format.rs` lack tests (opportunity for improvement).

### Error Handling Standards

**Custom error types**: Domain-specific error enums using `thiserror`:
- Clear discriminants (InvalidCommand, AlertNotFound, Unauthorized, etc.)
- User-facing messages via `user_message()` method
- Developer details via `Display` trait
- No sensitive information in user messages

**Error propagation**: Errors converted at boundaries (external SDK errors → domain errors). Results returned everywhere; unwrap/expect only in tests.

### Code Organization

**Module hierarchy**:
```
src/
├── main.rs           # Entry point, dependency wiring
├── config.rs         # Environment configuration
├── bot.rs            # Nostr protocol event loop
├── format.rs         # User-facing message formatting
├── alerts/           # Alert domain (self-contained)
│   ├── model.rs      # Core entities
│   ├── parser.rs     # Input validation
│   ├── evaluator.rs  # Business logic
│   ├── manager.rs    # Orchestration
│   ├── source.rs     # Market data abstraction
│   ├── notifier.rs   # Notification delivery
│   ├── error.rs      # Domain errors
│   └── repository/   # Persistence abstraction
├── commands/         # Command handlers (stateless)
└── polymarket/       # External API wrappers
```

**Dependency direction**: Infrastructure → Application → Domain (alerts). Domain has no dependencies on infrastructure.

## Observability

### Logging Standards

**Structured logging**: Use `tracing` crate with:
- `info!` for significant events (startup, shutdown, command execution)
- `warn!` for degraded operation (relay connection failure, API errors)
- `error!` for unexpected failures (database errors, logic bugs)
- `debug!`/`trace!` for detailed flow (development only)

**Log context**: Include relevant identifiers (alert ID, pubkey, slug, command) for correlation.

**Configuration**: `RUST_LOG` environment variable controls verbosity (default: `info`).

**Current gaps**: No metrics collection (Prometheus), no error alerting for operators.

### Build Metadata

**Version tracking**: Git commit hash injected at build time via `build.rs`. Logged at startup for deployment correlation.

**Environment**: Log configuration summary at startup (relay count, alert settings, database path).

## Governance

### Constitution Amendments

Amendments to this constitution require:
1. Documented rationale for change
2. Impact analysis on existing codebase
3. Template updates (plan, spec, tasks) to reflect new principles
4. Semantic versioning of constitution:
   - **MAJOR**: Backward-incompatible principle removals or redefinitions
   - **MINOR**: New principles or materially expanded guidance
   - **PATCH**: Clarifications, wording fixes, non-semantic refinements

### Compliance Verification

All code reviews MUST verify:
- Trait abstraction for new domain components
- Async patterns for I/O operations
- Input validation at boundaries
- Integration tests for complex logic
- Single responsibility adherence
- Configuration externalization

### Complexity Justification

Deviations from principles (e.g., synchronous SQLite in async runtime) MUST be justified with:
- Specific reason (simplicity, performance, dependency constraints)
- Alternative considered and why rejected
- Future mitigation plan if technical debt

### Living Documentation

This constitution reflects the **current state** of Polynostr (Phase 2 complete: alert system with persistence). Future phases may introduce new principles:
- **Phase 3** (Portfolio tracking): May add data modeling standards
- **Phase 4** (Trading commands): May add transaction integrity principles
- **Phase 5** (Web dashboard): May add API versioning standards

The constitution evolves with the project while maintaining core architectural values.

**Version**: 1.0.0 | **Ratified**: 2026-03-31 | **Last Amended**: 2026-03-31
