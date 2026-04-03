# Implementation Plan: Markets Closing Soon Command

**Branch**: `001-closing-events` | **Date**: 2026-03-31 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-closing-events/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add a `/closing` command that returns prediction markets closing within 24 hours, sorted by time remaining. The command will use the existing Gamma API client with new filtering logic to identify time-sensitive markets, providing users with actionable information about markets approaching their deadline. Implementation follows the established pattern from `/trending` command with time-based filtering and duration formatting.

## Technical Context

**Language/Version**: Rust 1.88.0 (edition 2024)  
**Primary Dependencies**: 
- nostr-sdk v0.44 (Nostr protocol, NIP-17/NIP-04 DMs)
- polymarket-client-sdk v0.4 (Gamma API for market data)
- tokio v1 (async runtime)
- chrono v0.4 (datetime handling, duration formatting)
- tracing v0.1 (structured logging)

**Storage**: SQLite (alerts.sqlite3) for alert persistence; not needed for this command  
**Testing**: cargo test with inline `#[cfg(test)]` modules and `#[tokio::test]` for async  
**Target Platform**: Linux/macOS server (tokio async runtime)  
**Project Type**: Nostr bot (message-driven command processor)  
**Performance Goals**: 
- Command response time: 1-2 seconds (matching `/trending` performance)
- API call latency: <500ms p95 (Polymarket Gamma API dependency)

**Constraints**: 
- No authentication required (public Gamma API)
- Plain text output only (universal Nostr client compatibility)
- Stateless command execution (no user session)
- Over-fetch and client-side filter pattern (API pagination handling)

**Scale/Scope**: 
- Single command addition (~150 LOC across 3 files)
- No new dependencies required
- Integration with existing command router and formatter

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Trait-Based Abstraction for Testability
**Status**: ✅ PASS  
**Evaluation**: This command adds a new method to the existing `GammaClient` wrapper (not a trait, but a struct wrapping the SDK client). The command handler follows the established stateless pattern and requires no new abstractions. No domain components need trait extraction.

### Principle II: Async-First Concurrency
**Status**: ✅ PASS  
**Evaluation**: All new code will use async/await patterns:
- `list_closing_events()` will be `async fn` calling async SDK methods
- Command handler uses `async fn handle()` signature
- No blocking I/O or synchronous operations

### Principle III: Fail-Fast Validation
**Status**: ✅ PASS  
**Evaluation**: Input validation at command boundary:
- Command accepts no arguments (validation is trivial)
- API responses validated before formatting
- Time-based filtering validated against current time
- User-friendly error messages for API failures

### Principle IV: Inline Integration Testing (NON-NEGOTIABLE)
**Status**: ✅ PASS (with implementation requirement)  
**Evaluation**: Must include inline tests in the implementation:
- `src/polymarket/gamma.rs`: Test `list_closing_events()` with stub responses
- `src/format.rs`: Test duration formatting edge cases
- Tests to cover: empty results, time boundary conditions, sorting order

**Action Required**: Inline `#[cfg(test)]` modules must be added to modified files.

### Principle V: Single Responsibility at Module Level
**Status**: ✅ PASS  
**Evaluation**: Clean responsibility separation:
- `commands/closing.rs`: Command handling only (delegates to gamma client)
- `polymarket/gamma.rs`: API interaction only (new method in existing module)
- `format.rs`: Presentation only (new formatter function)
- No business logic in command handler

### Principle VI: Privacy and Security by Design
**Status**: ✅ PASS  
**Evaluation**: 
- Read-only public API (no authentication)
- No user data stored or processed
- Respects user's chosen channel (DM vs public mention)
- No privacy concerns for this command

### Principle VII: Configuration Over Code
**Status**: ✅ PASS  
**Evaluation**: 
- No new configuration needed
- Uses existing `GammaClient` initialization
- Time window hardcoded to 24h (sensible default, can be made configurable later)

### Principle VIII: Graceful Degradation
**Status**: ✅ PASS  
**Evaluation**: 
- API errors return user-friendly messages (no crash)
- Empty results handled with clear message
- No system failure modes

### Overall Assessment
**GATE STATUS**: ✅ **PASS** — All constitutional principles satisfied. No violations requiring justification.

This is a straightforward additive change following established patterns. Complexity remains low and architectural consistency is maintained.

## Project Structure

### Documentation (this feature)

```text
specs/001-closing-events/
├── plan.md              # This file (/speckit.plan command output)
├── spec.md              # Feature specification
├── research.md          # Phase 0 output (to be generated)
├── data-model.md        # Phase 1 output (to be generated)
├── quickstart.md        # Phase 1 output (to be generated)
└── contracts/           # Phase 1 output (to be generated)
```

### Source Code (repository root)

```text
src/
├── main.rs                  # (no changes)
├── config.rs                # (no changes)
├── bot.rs                   # (no changes)
├── format.rs                # ✏️  Add format_closing_events()
│                            # ✏️  Add format_duration()
│                            # ✏️  Add #[cfg(test)] module
├── commands/
│   ├── mod.rs               # ✏️  Add closing module and routing
│   ├── closing.rs           # ✨ NEW: Command handler
│   ├── help.rs              # ✏️  Update command list
│   ├── search.rs            # (no changes)
│   ├── price.rs             # (no changes)
│   ├── trending.rs          # (no changes - used as reference)
│   ├── market.rs            # (no changes)
│   └── alert_*.rs           # (no changes)
└── polymarket/
    ├── gamma.rs             # ✏️  Add list_closing_events()
    │                        # ✏️  Add #[cfg(test)] module
    └── data.rs              # (no changes)

tests/
└── (inline tests only, no separate test files needed)
```

**Structure Decision**: This is a single-project codebase (Option 1). The feature adds one new file (`commands/closing.rs`) and modifies three existing files (`commands/mod.rs`, `polymarket/gamma.rs`, `format.rs`). All integration tests will be inline using `#[cfg(test)]` modules following the project's established testing pattern.

**Files Modified Summary**:
- **New**: `src/commands/closing.rs` (~20 LOC)
- **Modified**: `src/commands/mod.rs` (~3 LOC added)
- **Modified**: `src/commands/help.rs` (~1 LOC added)
- **Modified**: `src/polymarket/gamma.rs` (~50 LOC added including tests)
- **Modified**: `src/format.rs` (~60 LOC added including formatter and duration helper)

**Total Estimated Addition**: ~135 LOC (excluding tests)

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**No violations** — This section is not applicable. All constitutional principles are satisfied without justification requirements.

---

## Post-Design Constitution Re-Check

*Required after Phase 1 design artifacts are complete*

### Re-Evaluation Summary

All Phase 1 artifacts have been generated:
- ✅ `research.md` - All technical decisions documented
- ✅ `data-model.md` - Data flow and transformations defined
- ✅ `contracts/command-interface.md` - Public interface specified
- ✅ `quickstart.md` - Implementation guide complete

### Principle Re-Assessment

**Principle IV: Inline Integration Testing** ✅ **CONFIRMED**

The design documents specify:
- Test structure in `quickstart.md` (Step 6)
- Test scenarios in `data-model.md` (Testing Data Scenarios section)
- Coverage requirements in `research.md` (Section 5: Testing Strategy)

**Implementation requirement**: Tests must be added during implementation phase (not part of planning).

### All Other Principles

No changes from initial assessment. All principles remain satisfied:
- ✅ I: Trait-Based Abstraction
- ✅ II: Async-First Concurrency  
- ✅ III: Fail-Fast Validation
- ✅ IV: Inline Integration Testing (confirmed above)
- ✅ V: Single Responsibility
- ✅ VI: Privacy and Security
- ✅ VII: Configuration Over Code
- ✅ VIII: Graceful Degradation

### Design Quality Gates

**Architecture Consistency**: ✅ PASS
- Follows established command pattern
- No new architectural components
- Clean layer separation maintained

**Interface Clarity**: ✅ PASS  
- Command contract well-defined
- Input/output specifications complete
- Error handling documented

**Implementation Feasibility**: ✅ PASS
- All dependencies available
- No blocking technical unknowns
- Straightforward implementation path

**Testing Adequacy**: ✅ PASS
- Test cases identified
- Edge cases covered
- Integration test structure defined

### Final Gate Status

**GATE STATUS**: ✅ **PASS** — Ready for Phase 2 (Implementation)

No constitutional violations. Design is complete, consistent, and implementable.
