# Active Execution Board

## Status

- Status: Active
- Scope: `obd2-core` pre-`1.0` required feature completion
- Baseline date: 2026-04-09
- Primary references:
  - [FUNCTIONAL_REQUIREMENTS.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/FUNCTIONAL_REQUIREMENTS.md)
  - [2026-04-09-pre-1.0-redesign-roadmap.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/plans/2026-04-09-pre-1.0-redesign-roadmap.md)
  - [INTEGRATION.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/INTEGRATION.md)

## Purpose

This board is the execution layer above the roadmap.

It exists to answer four questions at all times:

1. What is actively being implemented now?
2. What is blocked on design clarification?
3. What must be finished before calling the non-J1939 surface complete?
4. What remains in the J1939 workstream before it can be documented as supported?

## Delivery Rule

Do not start new feature work outside this board unless it is:

- required to unblock an item already in `Now`
- a correctness fix for a regression introduced by in-flight board work
- a documentation or test change needed to close an item already in `Now`

## Current Completion Snapshot

Completed major areas:

- session-first public API
- routed request ownership
- standard OBD session operations
- enhanced and module-targeted reads
- diagnostic session control
- adapter hardening and probe policy
- ECU visibility for the supported non-J1939 path
- raw capture and replay-assisted debugging
- J1939 scope lock and contract definition

Open required areas:

- hardware parity harness execution on real USB and BLE hardware
- J1939 implementation completion: decoder path, routing model, full session surface, and replay fixtures

Recently completed:

- ignition-off as a distinct verified runtime state (LP ALERT / !LP ALERT detection, auto-recovery on successful request)
- repeated PID polling perf/regression harness (`tests/polling_regression.rs`, documented in FUNCTIONAL_REQUIREMENTS.md)
- `obd2-hw-test` harness crate scaffolding, CLI, report writer, matrix groups, and compare command

## Now

### Workstream A: Session Lifecycle Closure — COMPLETE

Status: **Done** (2026-04-09)

Implementation:

- `IgnitionOff` is set from ELM/STN adapter events (`LowVoltageReset` via `LP ALERT` / `!LP ALERT`).
- `Session::apply_adapter_events()` transitions `ConnectionState` to `IgnitionOff` on low-power events.
- `Session::mark_connection_active_if_recovered()` transitions back to `Connected` on any successful request.
- Test `test_session_enters_ignition_off_on_low_power_alert_and_recovers` verifies entry, persistence, and recovery.
- `INTEGRATION.md` documents `ConnectionState::IgnitionOff` behavior.
- `FUNCTIONAL_REQUIREMENTS.md` Area 2 checklist is fully checked.

Completed checklist:

- [x] Define the concrete ignition-off evidence model for ELM/STN flows.
- [x] Decide which adapter outcomes map to `IgnitionOff` vs `Disconnected` vs generic `Error`.
- [x] Implement state transition logic in the session/adapter event path.
- [x] Add regression tests for ignition-off detection and recovery.
- [x] Update the lifecycle section in FUNCTIONAL_REQUIREMENTS.md.
- [x] Update INTEGRATION.md with consumer-visible behavior.

### Workstream B: Perf And Regression Harness — COMPLETE

Status: **Done** (2026-04-09)

Implementation:

- Harness lives at `crates/obd2-core/tests/polling_regression.rs`.
- Shape: CI-friendly `#[tokio::test]` regression test (not a separate benchmark binary).
- Poll set: ENGINE_RPM, COOLANT_TEMP, VEHICLE_SPEED (3 PIDs per cycle).
- Execution mode: mock-backed via `MockAdapter` — no hardware required.
- Default: 500 timed iterations + 10 warmup iterations.
- Budget: 2ms per cycle (`MAX_MICROS_PER_CYCLE = 2000`).
- Configurable: `OBD2_CORE_POLLING_HARNESS_ITERS=1000` environment variable.
- Verifies: reading count matches expected, zero alerts, zero errors, elapsed within budget.
- Documented: FUNCTIONAL_REQUIREMENTS.md Area 16 includes run instructions.

Completed checklist:

- [x] Define the harness shape: regression test.
- [x] Select a representative poll set for standard PID workloads.
- [x] Add a mock-backed execution mode.
- [x] Record baseline output format and pass/fail expectations.
- [x] Document how to run the harness and how to interpret regressions.

## Next

### Parallel Workstream: Hardware Parity Harness

Status: **In Progress** (2026-04-09)

Implementation:

- New crate: `crates/obd2-hw-test`
- Commands implemented: `run`, `compare`, `vehicles`
- Matrix shape implemented in code: 13 groups
- Report output implemented: JSON summary + per-group details
- Real hardware execution is still pending for VIN population, first USB/BLE corpus, and CI rollout

Tracking board:

- [2026-04-09-hardware-parity-matrix-action-board.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/plans/2026-04-09-hardware-parity-matrix-action-board.md)

### Workstream D: J1939 Implementation

Why next:

- The J1939 contract is now explicit.
- The next step is implementation against that contract, not more scope debate.

Definition of done:

- J1939 decode path exists and is tested.
- Session-first J1939 operations exist and are documented.
- Discovery records visible J1939 sources.
- Replay fixtures exist for J1939 traffic.
- Public docs stop treating J1939 as incomplete only after the above is true.

Execution checklist:

- [ ] Define J1939 decoder boundary.
- [ ] Implement J1939 request/response frame parsing.
- [ ] Implement PGN extraction.
- [ ] Implement source-address and destination extraction.
- [ ] Implement transport-protocol reassembly handling.
- [ ] Implement acknowledgement decode.
- [ ] Add decode fixtures for J1939.
- [ ] Implement adapter initialization policy for protocol A/B/C.
- [ ] Implement global PGN request flow.
- [ ] Implement directed PGN request flow.
- [ ] Implement monitor flow support.
- [ ] Implement DM1 on the session-first surface.
- [ ] Record visible J1939 source addresses in discovery.
- [ ] Add routing and behavior tests.
- [ ] Add raw-capture replay fixtures for J1939.

Suggested file focus:

- `crates/obd2-core/src/protocol/j1939.rs`
- `crates/obd2-core/src/session/mod.rs`
- `crates/obd2-core/src/session/discovery.rs`
- `crates/obd2-core/src/adapter/elm327.rs`
- `crates/obd2-core/tests/`

## Blocked / Waiting

No items currently blocked.

## Exit Criteria

The board can be considered complete when all of the following are true:

- [x] `IgnitionOff` is implemented, verified, and documented
- [x] the perf/regression harness exists and is documented
- [ ] J1939 implementation is complete against the locked scope and documented as supported

## Immediate Recommended Order

1. Keep the hardware parity harness moving through first real USB and BLE runs.
2. Execute Workstream D against the locked J1939 scope.
3. Use the hardware parity board to track VIN/bootstrap, corpus generation, and CI gating.
