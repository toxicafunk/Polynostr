---
description: "Task list for Markets Closing Soon Command implementation"
---

# Tasks: Markets Closing Soon Command

**Input**: Design documents from `/specs/001-closing-events/`
**Prerequisites**: plan.md (✅), spec.md (✅), research.md (✅), data-model.md (✅), contracts/command-interface.md (✅), quickstart.md (✅)

**Tests**: Inline integration tests are REQUIRED per project constitution (Principle IV)

**Organization**: Tasks organized by implementation phase. This is a single-command feature with minimal scope - all three user stories are part of one cohesive implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- File paths are specified for each task

---

## Phase 1: Setup

**Purpose**: Project structure validation - ensure codebase is ready for implementation

- [X] T001 Verify Rust toolchain (1.88.0+) and cargo build succeeds
- [X] T002 Verify all dependencies compile: nostr-sdk, polymarket-client-sdk, tokio, chrono

**Checkpoint**: Development environment ready

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user stories can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

This feature has **no foundational tasks** - it builds entirely on existing infrastructure:
- ✅ GammaClient wrapper already exists
- ✅ Command routing already exists
- ✅ Formatting utilities already exist
- ✅ Nostr bot framework already exists

**Checkpoint**: Foundation ready (already exists) - user story implementation can now begin

---

## Phase 3: User Story 1 - View Closing Markets (Priority: P1) 🎯 MVP

**Goal**: As a Nostr user, I want to see which markets are closing soon so I can make last-minute trades before they close.

**User Story 1 Requirements**:
- Command name: `/closing` (supports both `/closing` and `closing` without slash)
- Returns markets closing within 24 hours
- Sorted by time remaining (soonest first)
- Up to 10 results
- Clear formatting with time-until-close

**Independent Test**: 
1. Send `/closing` via DM to the bot
2. Verify response shows markets closing in <24h
3. Verify markets are sorted by closing time (soonest first)
4. Verify format matches specification (title, duration, volume, price, slug)

### Implementation for User Story 1

- [X] T003 [P] [US1] Create command handler in src/commands/closing.rs
- [X] T004 [P] [US1] Add list_closing_events() method to GammaClient in src/polymarket/gamma.rs
- [X] T005 [P] [US1] Add format_duration() helper function in src/format.rs
- [X] T006 [P] [US1] Add format_closing_events() formatter in src/format.rs
- [X] T007 [US1] Register closing module in src/commands/mod.rs
- [X] T008 [US1] Add routing for "/closing" | "closing" in src/commands/mod.rs::handle_command()
- [X] T009 [US1] Update help text to include /closing command in src/commands/help.rs

**Checkpoint**: User Story 1 complete - `/closing` command fully functional

---

## Phase 4: User Story 2 - Consistent Multi-Channel Support (Priority: P1) 🎯 MVP

**Goal**: As a Nostr user, I want this command to work via DM or public mention, consistent with existing bot commands.

**User Story 2 Requirements**:
- Works via NIP-17 DM
- Works via NIP-04 DM  
- Works via public mention
- Response format identical across all channels

**Independent Test**:
1. Send `/closing` via NIP-17 DM → verify response
2. Send `/closing` via NIP-04 DM → verify response
3. Mention bot with `/closing` in public note → verify response
4. Confirm all responses are identical in format

### Implementation for User Story 2

**No implementation required** - User Story 2 is automatically satisfied by the existing bot architecture:
- ✅ Bot already handles NIP-17 DMs (existing infrastructure)
- ✅ Bot already handles NIP-04 DMs (existing infrastructure)
- ✅ Bot already handles public mentions (existing infrastructure)
- ✅ Command routing is channel-agnostic (existing infrastructure)

The command handler created in Phase 3 (T003) already works across all channels because:
1. `bot.rs::handle_notifications()` extracts message content regardless of channel
2. `commands/mod.rs::handle_command()` routes all channels uniformly
3. All commands return plain text strings sent via the appropriate channel

**Checkpoint**: User Story 2 complete (no additional work needed)

---

## Phase 5: User Story 3 - Consistent Format Output (Priority: P1) 🎯 MVP

**Goal**: As a Nostr user, I want the output formatted similarly to the `/trending` command for consistency.

**User Story 3 Requirements**:
- Numbered list format (matching `/trending`)
- Each entry shows: title, time-until-close, volume, price, slug
- Clean, readable layout
- Plain text (no markdown/HTML)

**Independent Test**:
1. Send `/closing` command
2. Compare output format to `/trending` command output
3. Verify consistency: numbered list, spacing, field labels
4. Verify all required fields present

### Implementation for User Story 3

**No additional implementation required** - User Story 3 is satisfied by the formatters created in Phase 3:
- ✅ T006 (format_closing_events) implements numbered list format matching `/trending`
- ✅ Format follows established pattern from src/format.rs::format_trending_events()
- ✅ Plain text output with clear field separators

**Verification**:
- [ ] T010 [US3] Manual verification: Compare `/closing` output format to `/trending` output format

**Checkpoint**: User Story 3 complete - output format is consistent

---

## Phase 6: Testing & Validation (Required by Constitution)

**Purpose**: Ensure correctness and satisfy constitutional requirement (Principle IV: Inline Integration Testing)

**⚠️ CONSTITUTIONAL REQUIREMENT**: Inline `#[cfg(test)]` modules are mandatory for modified files

### API Tests (src/polymarket/gamma.rs)

- [X] T011 [P] Add #[cfg(test)] module to src/polymarket/gamma.rs
- [X] T012 [P] Test: list_closing_events filters by 24h time window
- [X] T013 [P] Test: list_closing_events sorts by end_date ascending
- [X] T014 [P] Test: list_closing_events excludes closed markets
- [X] T015 [P] Test: list_closing_events excludes archived markets
- [X] T016 [P] Test: list_closing_events handles empty results

### Formatter Tests (src/format.rs)

- [X] T017 [P] Add #[cfg(test)] module to src/format.rs (if not exists, add to existing)
- [X] T018 [P] Test: format_duration with hours and minutes (e.g., "5h 23m")
- [X] T019 [P] Test: format_duration with minutes only (e.g., "45m")
- [X] T020 [P] Test: format_duration with less than 1 minute (returns "<1m")
- [X] T021 [P] Test: format_duration with negative duration (returns "Closed")
- [X] T022 [P] Test: format_closing_events with empty vec (returns "No markets..." message)

### Integration Validation

- [X] T023 Run cargo test and verify all tests pass
- [X] T024 Run cargo build and verify no warnings
- [ ] T025 Manual test: Send `/closing` via DM and verify response
- [ ] T026 Manual test: Verify markets are sorted by closing time
- [ ] T027 Manual test: Verify error handling with network disconnected

**Checkpoint**: All tests pass - feature is validated

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements and documentation

- [ ] T028 [P] Code review: Verify all files follow Rust idioms and project conventions
- [ ] T029 [P] Verify tracing/logging statements are present for debugging
- [ ] T030 Validation: Run through quickstart.md verification checklist
- [ ] T031 Final build: cargo build --release to verify production build

**Checkpoint**: Feature complete and production-ready

---

## Dependencies & Execution Order

### Phase Dependencies

1. **Setup (Phase 1)**: No dependencies - can start immediately
2. **Foundational (Phase 2)**: N/A - no foundational work needed
3. **User Story 1 (Phase 3)**: Can start immediately after Setup - implements core command
4. **User Story 2 (Phase 4)**: Automatically satisfied by existing infrastructure
5. **User Story 3 (Phase 5)**: Satisfied by Phase 3 implementation
6. **Testing (Phase 6)**: Depends on Phase 3 completion
7. **Polish (Phase 7)**: Depends on all phases completion

### User Story Dependencies

- **User Story 1 (P1)**: Independent - implements core command functionality
- **User Story 2 (P1)**: Independent - automatically satisfied by existing bot infrastructure  
- **User Story 3 (P1)**: Depends on US1 for format implementation

### Within Each User Story

**Phase 3 (User Story 1)** - Core implementation:
- T003, T004, T005, T006 can run in parallel [P] (different files)
- T007, T008 must run sequentially (same file: mod.rs)
- T009 runs independently [P] (different file: help.rs)

**Phase 6 (Testing)** - All test tasks:
- T011-T016 (API tests) can run in parallel [P]
- T017-T022 (Formatter tests) can run in parallel [P]
- T023-T027 (Integration validation) run sequentially

### Parallel Opportunities

**Maximum parallelization in Phase 3:**
```bash
# All these can be developed in parallel:
T003: Create src/commands/closing.rs (developer A)
T004: Modify src/polymarket/gamma.rs (developer B)
T005: Add format_duration() to src/format.rs (developer C)
T006: Add format_closing_events() to src/format.rs (developer C, after T005)
T009: Update src/commands/help.rs (developer D)

# Then sequentially:
T007-T008: Modify src/commands/mod.rs (developer A)
```

**Maximum parallelization in Phase 6:**
```bash
# All test writing can happen in parallel:
T011-T016: API tests in gamma.rs (developer A)
T017-T022: Formatter tests in format.rs (developer B)
```

---

## Parallel Example: User Story 1 Core Implementation

```bash
# Launch parallel tasks for maximum efficiency:
Task: "Create command handler in src/commands/closing.rs"
Task: "Add list_closing_events() method to GammaClient in src/polymarket/gamma.rs"
Task: "Add format_duration() helper function in src/format.rs"
Task: "Update help text to include /closing command in src/commands/help.rs"

# Then sequential:
Task: "Register closing module in src/commands/mod.rs"
Task: "Add routing for '/closing' in src/commands/mod.rs::handle_command()"
```

---

## Implementation Strategy

### MVP First (All User Stories are P1 - Complete MVP)

This feature is simple enough that the MVP includes all three user stories:

1. **Complete Phase 1**: Setup (verify environment)
2. **Skip Phase 2**: No foundational work needed
3. **Complete Phase 3**: User Story 1 - Core command implementation
4. **Verify Phase 4**: User Story 2 - Multi-channel support (automatic)
5. **Verify Phase 5**: User Story 3 - Format consistency (automatic)
6. **Complete Phase 6**: Testing & validation
7. **Complete Phase 7**: Polish & final validation
8. **STOP and VALIDATE**: Test complete command end-to-end
9. Deploy

**Estimated Time**: 2-3 hours (per quickstart.md)

### Incremental Delivery

Since all user stories are P1 and interdependent, they are delivered together as a single MVP:

1. Complete Setup → Environment ready
2. Complete User Story 1 → Core command works
3. User Story 2 & 3 → Automatically satisfied
4. Complete Testing → Feature validated  
5. Complete Polish → Production ready
6. **Deploy as single atomic feature**

### Single Developer Path

**Recommended order for solo implementation:**

1. T001-T002: Verify setup
2. T003-T009: Implement core command (some parallel, some sequential)
3. T010: Manual format verification
4. T011-T022: Write all tests (can be parallel)
5. T023-T027: Run validation
6. T028-T031: Final polish

**Time**: ~2.5 hours for experienced Rust developer

---

## Task Summary

**Total Tasks**: 31

### By Phase:
- Phase 1 (Setup): 2 tasks
- Phase 2 (Foundational): 0 tasks (no work needed)
- Phase 3 (User Story 1): 7 tasks
- Phase 4 (User Story 2): 0 implementation tasks (satisfied by infrastructure)
- Phase 5 (User Story 3): 1 verification task
- Phase 6 (Testing): 17 tasks
- Phase 7 (Polish): 4 tasks

### By User Story:
- User Story 1: 7 implementation tasks (T003-T009)
- User Story 2: 0 tasks (automatically satisfied)
- User Story 3: 1 verification task (T010)

### Parallel Opportunities:
- Phase 3: 4 tasks can run in parallel (T003, T004, T005, T009)
- Phase 6: 12 test tasks can run in parallel (T011-T022)
- **Total parallelizable**: 16 tasks marked [P]

### Independent Test Criteria:
- **User Story 1**: Send `/closing`, verify response format and sorting
- **User Story 2**: Test command via NIP-17, NIP-04, and public mention
- **User Story 3**: Compare output format to `/trending` for consistency

### Suggested MVP Scope:
**All three user stories** (P1 priority) - This is a simple, cohesive feature where all stories are interdependent and should be delivered together.

---

## Notes

- This is an **additive feature** - no existing code is deleted or substantially refactored
- **Low complexity**: ~135 LOC added across 5 files
- **No new dependencies** required
- **Constitutional compliance**: Inline tests required (Principle IV)
- **Reference implementation**: `/trending` command provides exact pattern to follow
- Follow established patterns exactly (no new patterns introduced)
- Commit after completing each phase
- Verify tests fail before implementing (TDD approach for test tasks)