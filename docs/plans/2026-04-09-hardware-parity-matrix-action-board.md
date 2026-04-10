# Hardware Parity Matrix Action Board

## Status

- Status: Active
- Scope: `obd2-hw-test` harness, real USB/BLE parity runs, report comparison, and follow-on docs/CI wiring
- Baseline date: 2026-04-09
- Last implementation update: 2026-04-09
- Primary references:
  - [2026-04-09-hardware-parity-matrix-design.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/plans/2026-04-09-hardware-parity-matrix-design.md)
  - [2026-04-09-hardware-parity-matrix-implementation.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/plans/2026-04-09-hardware-parity-matrix-implementation.md)
  - [2026-04-09-active-execution-board.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/plans/2026-04-09-active-execution-board.md)
  - [FUNCTIONAL_REQUIREMENTS.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/FUNCTIONAL_REQUIREMENTS.md)
  - [README.md](/Users/jared/Projects/HaulLogic/obd2-core/README.md)

## Current Implementation Snapshot

Implemented in the repository:

- workspace registration and new crate `crates/obd2-hw-test`
- CLI commands: `run`, `compare`, `vehicles`
- report schema, report writer/reader, and fatal-startup report emission
- explicit raw-capture enablement and capture-directory wiring
- full 13-group registry and group implementations
- comparison logic with critical/warning/info severity classes

Locally verified without hardware:

- `cargo build -p obd2-hw-test`
- `cargo run -p obd2-hw-test -- --help`
- `cargo run -p obd2-hw-test -- vehicles`
- synthetic `compare` smoke test with two temporary JSON reports
- startup failure report emission via `run --transport usb` without `--port`

Still hardware-blocked:

- real VIN population
- first USB report
- first BLE report
- first full 3 vehicles x 2 transports corpus
- CI rollout on self-hosted hardware

## Purpose

This board is the execution layer for the hardware parity work.

The implementation plan already contains the intended crate shape and a large
task list. What it does not do well enough is control execution order,
separate harness work from hardware-dependent work, or lock down which parts
are blocked on existing `obd2-core` gaps. This board is the concrete tracker
for that.

## Review Summary

Review of the implementation plan surfaced these corrections and constraints:

- The matrix is **13 groups**, not 12.
- `capture` cannot rely on debug-only default raw capture behavior; the harness
  must explicitly enable and route capture output during manual and release runs.
- `compare` is only useful after report metadata is hydrated from live session
  state; placeholder `None` fields are not enough.
- Vehicle VIN checks need a bootstrap path before real VINs are populated.
- `j1939` is not a pure harness task. The harness may use the current
  `Session::read_j1939_pgn` and `Session::read_j1939_dtcs` surface, but it must
  not silently absorb unfinished core J1939 work.

## Locked Decisions

- Group count is fixed at `13`:
  - `init`
  - `protocol`
  - `vin`
  - `pids`
  - `supported`
  - `dtcs`
  - `voltage`
  - `polling`
  - `capture`
  - `j1939`
  - `enhanced`
  - `monitoring`
  - `recovery`
- The first implementation target is the harness crate at
  `crates/obd2-hw-test/`; do not expand scope into unrelated crate work.
- Real VINs are allowed to start as unknown placeholders, but the vehicle
  catalog must support a transition from bootstrap mode to strict-match mode.
- CI is a late-stage item. Manual USB and BLE runs must be stable before the
  workflow is added.
- `recovery` remains manual-only and should be skipped in CI by design.

## Delivery Rules

- Do not start advanced test groups before the harness can build, run, and emit
  a JSON report on one transport.
- Do not implement compare severity logic before report metadata and group
  status outputs are stable.
- Do not add the CI workflow before at least one successful USB run and one
  successful BLE run produce valid reports and raw captures.
- If a test group needs new `obd2-core` API work, split that into an explicit
  follow-up item instead of hiding it inside the harness task.
- Keep commit boundaries larger than the implementation plan suggests. The
  preferred slicing is foundation, run path, baseline groups, advanced groups,
  compare/docs/CI.

## File Focus

Primary harness files:

- `Cargo.toml`
- `crates/obd2-hw-test/Cargo.toml`
- `crates/obd2-hw-test/src/main.rs`
- `crates/obd2-hw-test/src/report.rs`
- `crates/obd2-hw-test/src/runner.rs`
- `crates/obd2-hw-test/src/compare.rs`
- `crates/obd2-hw-test/src/vehicles.rs`
- `crates/obd2-hw-test/src/tests/`

Read-only integration surfaces that the harness must align with:

- `crates/obd2-core/src/session/mod.rs`
- `crates/obd2-core/src/session/poller.rs`
- `crates/obd2-core/src/transport/serial.rs`
- `crates/obd2-core/src/transport/ble.rs`
- `crates/obd2-core/src/transport/logging.rs`
- `crates/obd2-core/src/protocol/j1939.rs`

Follow-on docs and automation files:

- `docs/FUNCTIONAL_REQUIREMENTS.md`
- `README.md`
- `docs/plans/2026-04-09-active-execution-board.md`
- `.github/workflows/hardware-parity.yml`

## Workstream A: Harness Foundation

Status: Done

- [x] `HW-01` Create `crates/obd2-hw-test/Cargo.toml` and add the crate to the
      workspace in root `Cargo.toml`.
  - Depends on: none
  - Verify: `cargo build -p obd2-hw-test`
  - Done when: the crate resolves and compiles as a binary target.
- [x] `HW-02` Create the initial module skeleton in `src/main.rs`,
      `src/report.rs`, `src/runner.rs`, `src/compare.rs`, `src/vehicles.rs`,
      and `src/tests/mod.rs`.
  - Depends on: `HW-01`
  - Verify: `cargo run -p obd2-hw-test -- --help`
  - Done when: `run`, `compare`, and `vehicles` are visible in CLI help.
- [x] `HW-03` Lock the report schema and failure model.
  - Scope:
    - `Report`
    - `ReportMeta`
    - `ReportSummary`
    - `TestGroupResult`
    - `TestStatus`
  - Depends on: `HW-02`
  - Verify: unit-free compile and a smoke write/read cycle from the CLI path
  - Done when: report writing works even when some groups fail or skip.
- [x] `HW-04` Build a vehicle catalog that supports bootstrap VIN discovery.
  - Scope:
    - replace strict required VIN-at-definition with an optional or staged
      expectation model
    - keep protocol, make, PID, and capability expectations strict
  - Depends on: `HW-02`
  - Verify: `cargo run -p obd2-hw-test -- vehicles`
  - Done when: the catalog can list known vehicles before VIN population.
- [x] `HW-05` Implement the group registry and `--only` parsing.
  - Depends on: `HW-02`, `HW-03`
  - Verify: `cargo run -p obd2-hw-test -- run --transport usb --port /tmp/fake --vehicle duramax-2006 --only init`
  - Done when: the runner can limit execution to named groups.

## Workstream B: Transport And Session Bootstrap

Status: Implemented in code; awaiting real-hardware execution

- [x] `HW-06` Implement transport construction for USB using
      `transport::serial::SerialTransport`.
  - Depends on: `HW-01`, `HW-02`
  - Verify: missing `--port` fails fast with a useful error
  - Done when: USB run path is fully wired through `Elm327Adapter` and `Session`.
- [x] `HW-07` Implement transport construction for BLE using
      `transport::ble::BleTransport::scan_and_connect(...)`.
  - Depends on: `HW-01`, `HW-02`
  - Verify: scan/connect errors are surfaced cleanly to the CLI
  - Done when: BLE run path creates a live session without special-case hacks.
- [x] `HW-08` Explicitly configure raw capture from the harness.
  - Scope:
    - call `Session::set_raw_capture_enabled(true)` for real runs
    - call `Session::set_raw_capture_directory(...)`
    - record the resulting `raw_capture_path`
  - Depends on: `HW-06` or `HW-07`
  - Verify: release/manual run produces a `.obd2raw` file
  - Done when: `capture` test does not depend on debug-only defaults.
- [x] `HW-09` Hydrate report metadata from live session state.
  - Scope:
    - `adapter_chipset`
    - `adapter_firmware`
    - `protocol_detected`
    - `raw_capture_path`
    - optionally discovery/probe information inside group details
  - Depends on: `HW-03`, `HW-06`, `HW-07`, `HW-08`
  - Verify: manual smoke report contains real values instead of placeholders
  - Done when: compare logic has stable metadata to consume.
- [x] `HW-10` Ensure the run path always writes a report on completion or
      early failure.
  - Depends on: `HW-03`, `HW-09`
  - Verify: a failed run still leaves behind JSON with failure details
  - Done when: hardware debugging does not require reproducing a failed session
    with stdout alone.

## Workstream C: Baseline Test Groups

Status: Implemented in code; awaiting first USB/BLE hardware execution

These are the groups that should work on all three vehicles without needing
interactive control or deep conditional logic.

- [x] `HW-11` `init`
  - Session/API: `Session::initialize()`
  - Done when: chipset, firmware, and capability details are recorded.
- [x] `HW-12` `protocol`
  - Session/API: `session.adapter_info().protocol`
  - Done when: detected protocol is compared against the vehicle definition.
- [x] `HW-13` `vin`
  - Session/API: `Session::identify_vehicle()`, `Session::read_vin()`
  - Done when: VIN/bootstrap handling and manufacturer checks are recorded.
- [x] `HW-14` `supported`
  - Session/API: `Session::supported_pids()`
  - Done when: required PID membership is checked and missing PIDs are listed.
- [x] `HW-15` `pids`
  - Session/API: `Session::read_pid()`
  - Done when: required PID values are read and plausibility-checked.
- [x] `HW-16` `dtcs`
  - Session/API:
    - `Session::read_dtcs()`
    - `Session::read_pending_dtcs()`
    - `Session::read_permanent_dtcs()`
  - Done when: transport/runtime failures are distinguished from vehicle-state
    content differences.
- [x] `HW-17` `voltage`
  - Session/API: `Session::battery_voltage()`
  - Done when: adapter voltage is checked against an explicit range.
- [x] `HW-18` `polling`
  - Session/API: `session::poller::execute_poll_cycle(...)`
  - Done when: reads/sec and error count are recorded for later USB/BLE comparison.
- [x] `HW-19` `capture`
  - Session/API: `Session::raw_capture_path()`,
    `transport::parse_raw_capture(...)`
  - Done when: the file exists and at least one parseable command/response pair
    is present.
- [x] `HW-20` `monitoring`
  - Session/API:
    - `Session::read_readiness()`
    - `Session::read_all_o2_monitoring()`
    - `Session::read_test_results(...)`
  - Done when: `NoData` is treated as acceptable and transport failures are not.

Workstream C exit gate:

- [ ] One manual USB run can execute `init,protocol,vin,supported,pids,dtcs,voltage,polling,capture,monitoring`
      and emit a valid JSON report plus raw capture.
- [ ] One manual BLE run can execute the same non-conditional set and emit a
      valid JSON report plus raw capture.

## Workstream D: Conditional And Advanced Groups

Status: Implemented in code; awaiting hardware validation

- [x] `HW-21` `enhanced`
  - Session/API:
    - `Session::module_pids(...)`
    - `Session::read_enhanced(...)`
  - Vehicle gate: only run when `has_spec_match == true`
  - Done when: the harness can read a small, deterministic subset of enhanced
    PIDs from a matched module and report unsupported/missing cases cleanly.
- [x] `HW-22` `j1939`
  - Session/API:
    - `Session::read_j1939_pgn(...)`
    - `Session::read_j1939_dtcs()`
  - Vehicle gate: only run when `has_j1939 == true`
  - Scope lock:
    - use the current session surface only
    - do not add hidden addressed-routing or transport-protocol work here
  - Done when: the harness can request and decode the locked PGN set already
    supported by `obd2-core`, or skip with an explicit core-gap reason.
- [x] `HW-23` `recovery`
  - Session/API: `Session::connection_state()`
  - Flag gate: `--interactive`
  - Done when: the harness can guide a manual ignition-off / ignition-on cycle
    and verify `IgnitionOff -> Connected`.

Workstream D exit gate:

- [ ] Conditional groups either pass or skip with explicit reasons based on
      vehicle capability and CLI flags.
- [ ] Any required core follow-up is written down explicitly instead of being
      buried in harness-specific code.

## Workstream E: Comparison Logic

Status: Implemented and smoke-verified

- [x] `HW-24` Implement `compare` report loading and diff output.
  - Depends on: `HW-09`, baseline report stability
  - Verify: `cargo run -p obd2-hw-test -- compare report-a.json report-b.json`
  - Done when: two existing JSON reports can be compared without manual parsing.
- [x] `HW-25` Lock severity classes and initial tolerance policy.
  - Critical:
    - protocol mismatch
    - VIN mismatch after VINs are populated
    - group status mismatch, excluding intentional skips
  - Warning:
    - supported PID bitmap deltas
    - numeric PID divergence beyond a documented tolerance
  - Info:
    - throughput/timing differences
  - Depends on: `HW-24`
  - Done when: severity rules are explicit and encoded, not implied.
- [ ] `HW-26` Record at least one same-vehicle USB vs BLE comparison result for
      each vehicle.
  - Depends on: Workstreams C and D as applicable
  - Done when: parity output is grounded in real artifacts, not synthetic JSON.

## Workstream F: Hardware Bootstrap And Corpus

Status: Parallel with late harness work where hardware is available

- [ ] `HW-27` Run a bootstrap VIN discovery pass on each vehicle.
  - Output: actual VIN values for `duramax-2006`, `malibu-2020`, and `accord-2001`
  - Depends on: `HW-13`
  - Done when: placeholder VINs are replaced or the board records why they are not.
- [ ] `HW-28` Lock a results directory layout for committed artifacts.
  - Recommended path: `test-results/hardware-parity/`
  - Depends on: `HW-10`
  - Done when: reports and captures have a stable home for manual and CI runs.
- [ ] `HW-29` Produce the first full matrix corpus:
  - `duramax-2006`: USB + BLE
  - `malibu-2020`: USB + BLE
  - `accord-2001`: USB + BLE
  - Depends on: Workstreams C and D as applicable
  - Done when: six reports exist and can be paired by vehicle for comparison.
- [ ] `HW-30` Review the first corpus and tighten pass/fail rules where needed.
  - Examples:
    - BLE timing tolerance
    - acceptable `NoData` cases in monitoring
    - whether supported PID deltas are warning or critical
  - Depends on: `HW-29`
  - Done when: parity policy is based on observed hardware behavior.

## Workstream G: Documentation And Automation

Status: Partial

- [x] `HW-31` Update `docs/FUNCTIONAL_REQUIREMENTS.md`.
  - Scope:
    - Area 19 testing coverage status
    - release-readiness line item for real hardware parity
  - Depends on: at least one successful real run
- [x] `HW-32` Update `README.md`.
  - Scope:
    - add a hardware testing section
    - explain manual run prerequisites
    - point to the harness crate
  - Depends on: CLI and report path being stable
- [x] `HW-33` Update `docs/plans/2026-04-09-active-execution-board.md`.
  - Scope:
    - either add hardware parity as a live workstream
    - or explicitly point from the active board to this dedicated board
  - Depends on: this board being the authoritative tracker
- [ ] `HW-34` Add `.github/workflows/hardware-parity.yml`.
  - Gate:
    - manual USB and BLE runs are stable
    - self-hosted runner requirements are known
  - Depends on: `HW-29`, `HW-30`
- [ ] `HW-35` Document runner prerequisites for self-hosted execution.
  - Scope:
    - connected adapters
    - BLE permissions
    - vehicle/ignition expectations
    - output path expectations
  - Depends on: `HW-34`

## Blocked / External Dependencies

- Access to the actual vehicles and adapters is required for VIN population and
  the first real corpus.
- BLE behavior depends on the host machine and permissions used by the eventual
  self-hosted runner.
- Duramax J1939 validation may surface gaps already called out in
  `FUNCTIONAL_REQUIREMENTS.md` Area 22. Those gaps must be tracked explicitly if
  encountered.

## Exit Criteria

The board is complete when all of the following are true:

- [ ] `obd2-hw-test` builds and exposes stable `run`, `compare`, and `vehicles`
      commands
- [ ] report JSON is emitted reliably for successful and failed runs
- [ ] raw capture is explicitly enabled and validated by the harness
- [ ] all baseline non-interactive groups are implemented and exercised on USB
      and BLE
- [ ] conditional groups pass or skip with explicit reasons
- [ ] VINs are populated for the three known vehicles
- [ ] one full 3 vehicles x 2 transports report corpus exists
- [ ] report comparison produces meaningful parity decisions
- [ ] functional requirements, README, and execution tracking docs are updated
- [ ] CI workflow is added only after manual parity is stable

## Immediate Recommended Order

1. Complete Workstream A.
2. Complete Workstream B.
3. Execute Workstream C on USB first.
4. Execute Workstream C on BLE.
5. Implement Workstream E once real reports exist.
6. Close Workstream F VIN/bootstrap and corpus collection.
7. Finish Workstream D conditional groups with explicit scope control.
8. Finish Workstream G docs and CI.
