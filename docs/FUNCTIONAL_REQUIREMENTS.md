# obd2-core Functional Requirements

## Document Status

- Status: Active working reference
- Scope: Whole project
- Applies to:
  - `crates/obd2-core`
  - `crates/obd2-store-sqlite`
  - top-level documentation, test matrix, and release-readiness tracking
- Current baseline date: 2026-04-09

## Purpose

This document defines the functional requirements for `obd2-core` as a pre-`1.0` diagnostic library.

It has four jobs:

1. Define what the project is required to do.
2. Record what is already implemented.
3. Record what remains incomplete.
4. Provide checklist-based feature tracking for development and release readiness.

This document is intended to be a standalone reference. It should be readable without having to reconstruct intent from design notes, implementation commits, or roadmap fragments.

## Product Definition

`obd2-core` is a session-first Rust library for:

- connecting to OBD-II adapters over multiple transports
- initializing and managing real vehicle communication
- discovering how to talk to a connected vehicle
- reading standard and enhanced diagnostic data
- performing targeted module diagnostics
- capturing raw communication for field debugging and replay
- supporting multiple vehicle families, model years, and communication patterns

The project also includes:

- a vehicle-spec system for VIN matching, topology, addressing, enhanced PIDs, rules, and thresholds
- a protocol layer for framing and decode behavior
- a storage abstraction layer, with a SQLite backend crate

## Project Goals

- Correctness before API stability
- Reliable operation across older and newer vehicles
- Reliable operation across USB and BLE adapters
- Clear separation between session, adapter, transport, protocol, and spec responsibilities
- Strong field-debuggability through raw capture and replay
- Testability of routing, discovery, protocol probing, and adapter behavior

## Non-Goals

- Preserving pre-`1.0` API compatibility
- Treating unfinished J1939 support as production-complete
- Hiding adapter/protocol limitations behind vague behavior
- Optimizing for convenience over explicit state and routing correctness

## Status Legend

- `[x]` Implemented and verified in the current codebase
- `[ ]` Not complete
- `[~]` Partially implemented or implemented with known gaps
- `[>]` Roadmap item for a future phase, intentionally deferred

## Source Documents

This document consolidates project intent from:

- [README.md](/Users/jared/Projects/HaulLogic/obd2-core/README.md)
- [docs/INTEGRATION.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/INTEGRATION.md)
- [2026-04-09-pre-1.0-redesign-roadmap.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/plans/2026-04-09-pre-1.0-redesign-roadmap.md)
- [docs/functional_reference/README.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/functional_reference/README.md)
- the current implementation in `crates/obd2-core`

## System Overview

### Architectural Rules

- `Session` is the primary supported public API.
- Discovery is a first-class lifecycle, not an incidental helper.
- Logical module names must be resolved by `Session`.
- Adapters must operate on resolved physical routing, not unresolved module names.
- Transports must move bytes only.
- Protocol framing and decode must be isolated from adapter setup logic.
- Real adapters must never receive overlapping commands.
- Raw capture and replay must be available for field debugging.

### Layered Model

```text
Application
  -> Session
    -> Adapter
      -> Transport
```

### Primary Runtime Flows

- Session construction
- Adapter initialization
- Protocol probing and selection
- Vehicle identification
- Discovery profile construction
- Standard PID / DTC / monitoring requests
- Enhanced and module-targeted requests
- Diagnostic session control
- Polling
- Raw capture and replay-assisted debugging

## Functional Areas

## Area 1: Public Session API

### Requirement

Consumers must be able to perform normal library usage entirely through `Session`, without building application logic directly on adapters.

### Required Behavior

- `Session` must own:
  - initialization
  - protocol discovery
  - vehicle identification
  - routing resolution
  - diagnostic session state
  - polling
  - visibility reporting
- High-level application flows must not require calling `Adapter::request(...)` directly.
- Remaining adapter access must be clearly treated as low-level escape-hatch behavior, not the normal supported model.

### Current Status

- [x] `Session` is the documented primary API.
- [x] Standard PID reads are session-owned.
- [x] DTC reads and clear operations are session-owned.
- [x] Freeze frame reads are session-owned.
- [x] Readiness reads are session-owned.
- [x] Mode 05 O2 monitoring is session-owned.
- [x] Mode 06 test results are session-owned.
- [x] VIN and vehicle info reads are session-owned.
- [x] Enhanced and module-targeted reads are session-owned.
- [x] Diagnostic session operations are session-owned.
- [x] Polling is session-owned.
- [x] Remaining adapter-first helpers in `session::modes` have been internalized.

### Acceptance Checklist

- [x] A consumer can initialize, identify, read, diagnose, and poll via `Session`.
- [x] Public docs describe `Session` as the supported high-level surface.
- [x] No normal supported flow depends on unresolved logical module names reaching an adapter.

## Area 2: Session Lifecycle And Connection State

### Requirement

The library must model adapter and communication lifecycle explicitly enough to support real adapter behavior and vehicle variability.

### Required Behavior

- The session must track lifecycle state.
- Initialization must be idempotent.
- High-level session methods must enforce initialization.
- Discovery and routing must only occur after initialization has established adapter state.
- Adapter-driven state transitions must be reflected back into the session.

### Current Status

- [x] `Session` tracks connection state.
- [x] Session methods call initialization automatically when needed.
- [x] Initialization is idempotent at the session and adapter level.
- [x] Adapter events update session connection state.
- [~] The lifecycle enum exists, but the distinction set is not yet as rich as the full target model described below.

### Target Lifecycle States

- [x] Adapter present
- [x] Adapter initialized
- [x] Protocol negotiating
- [x] Connected
- [x] Unsupported protocol
- [x] Disconnected/error
- [ ] Ignition-off as a distinct, verified runtime state

### Acceptance Checklist

- [x] Calling a high-level session method without explicit `initialize()` succeeds when possible.
- [x] Session state changes are observable.
- [x] Initialization outcomes are visible in discovery and raw capture.
- [ ] Ignition-off state is validated and exercised with adapter-specific behavior.

## Area 3: Discovery Profile

### Requirement

The library must produce a single authoritative discovery/profile model describing how it chose to talk to the current vehicle.

### Required Behavior

`DiscoveryProfile` must represent:

- adapter info and capabilities
- selected protocol
- source of protocol choice
- active bus
- resolved modules and addresses
- probe attempts and outcomes
- visible ECUs observed during traffic

### Current Status

- [x] Discovery profile is session-owned.
- [x] Adapter capabilities are included.
- [x] Selected protocol is included.
- [x] Protocol-choice source is included.
- [x] Active bus is included.
- [x] Resolved modules are included.
- [x] Probe attempts are included.
- [x] Visible ECUs are included.

### Acceptance Checklist

- [x] `Session::discovery()` returns enough information to understand routing and protocol selection.
- [x] Discovery is refreshed after identification/spec matching.
- [x] Discovery is refreshed after adapter events that alter visible state.

## Area 4: Vehicle Identification And Profiling

### Requirement

The library must identify vehicles and associate them with the correct spec/topology model when possible.

### Required Behavior

- Read VIN from the vehicle.
- Decode VIN offline.
- Match vehicle specs by VIN and related identity metadata.
- Retain usable standard OBD functionality even when no spec match exists.
- Rename active raw capture files using the VIN when available.

### Current Status

- [x] VIN read is implemented.
- [x] Offline VIN decode is implemented.
- [x] Spec matching is implemented.
- [x] Standard PIDs work without a spec.
- [x] Raw capture is renamed after VIN discovery.

### Known Limitations

- [~] Identification is still VIN-centric; deeper ECU-discovered topology refinement remains limited.
- [>] Broader ECU-discovery-driven profiling is on the roadmap.

### Acceptance Checklist

- [x] Vehicle identification populates a `VehicleProfile`.
- [x] Spec-backed and non-spec-backed vehicles both remain usable.
- [x] Discovery profile is refreshed after vehicle identification.

## Area 5: Vehicle Spec System

### Requirement

The library must represent vehicle-specific communication topology and diagnostic metadata in a way that supports real routing and interpretation.

### Required Behavior

- Runtime and embedded spec loading
- VIN-based matching
- Communication buses
- Module definitions and physical addresses
- Enhanced PID definitions
- DTC libraries
- Thresholds
- Diagnostic rules
- Known issues
- Polling groups
- Validation of conflicting or inconsistent topology

### Current Status

- [x] Embedded specs are supported.
- [x] Runtime YAML spec loading is supported.
- [x] VIN matching is supported.
- [x] Bus and module topology are modeled.
- [x] Physical addressing is modeled.
- [x] Enhanced PID definitions are modeled.
- [x] Thresholds are modeled.
- [x] Diagnostic rules are modeled.
- [x] Known issues are modeled.
- [x] Polling groups are modeled.
- [x] Duplicate/conflicting routing validation is supported.

### Acceptance Checklist

- [x] Specs can describe older and newer vehicle families.
- [x] Specs can describe multi-bus and per-module addressing.
- [x] Session routing can consume spec topology directly.

## Area 6: Routing Model

### Requirement

The library must resolve logical module targets into physical routing before adapter execution.

### Required Behavior

- Broadcast and addressed requests must be explicit.
- Session must resolve module names using discovery/spec data.
- Bus conflicts must fail explicitly.
- Missing module routing must fail explicitly.
- Adapters must accept resolved physical routing only.

### Current Status

- [x] Internal `RoutedRequest` exists.
- [x] Physical target types exist.
- [x] Session resolves module names into physical addresses.
- [x] Addressed requests are adapter-facing only after resolution.
- [x] Broadcast remains explicit.
- [x] Missing discovery profile fails explicitly.
- [x] Unknown module fails explicitly.
- [x] Wrong-bus target fails explicitly.

### Acceptance Checklist

- [x] `Target::Module` is never trusted by the adapter as a logical name.
- [x] Session-level tests exercise missing discovery, missing module, and wrong-bus cases.
- [x] Addressed routing behavior is validated through `Session -> RoutedRequest -> Elm327Adapter`.

## Area 7: Standard OBD-II Operations

### Requirement

The supported non-J1939 surface must cover the standard OBD-II operations needed for normal diagnostics.

### Required Behavior

- Mode 01 current data
- Mode 02 freeze frame
- Mode 03 stored DTCs
- Mode 04 clear DTCs
- Mode 05 O2 monitoring
- Mode 06 on-board monitor results
- Mode 07 pending DTCs
- Mode 09 vehicle information
- Mode 0A permanent DTCs
- Readiness/status interpretation
- Aggregated DTC read helpers

### Current Status

- [x] Mode 01 PID reads are implemented.
- [x] Multi-PID reads are implemented.
- [x] Supported-PID discovery is implemented.
- [x] Mode 02 freeze frame reads are implemented.
- [x] Mode 03 stored DTC reads are implemented.
- [x] Mode 04 clear DTCs is implemented.
- [x] Module-targeted Mode 04 clear is implemented.
- [x] Mode 05 O2 monitoring is implemented.
- [x] Mode 06 test results are implemented.
- [x] Mode 07 pending DTC reads are implemented.
- [x] Mode 09 VIN is implemented.
- [x] Mode 09 full vehicle info helper is implemented.
- [x] Mode 0A permanent DTC reads are implemented.
- [x] Readiness decode is implemented.
- [x] `read_all_dtcs()` is implemented.

### Acceptance Checklist

- [x] These operations are available on `Session`.
- [x] Tests cover the session-owned versions of the standard-mode flows.
- [x] Docs list the supported session-owned standard-mode operations.

## Area 8: Enhanced And Module-Targeted Operations

### Requirement

The library must support manufacturer-specific data reads and targeted communication against known modules.

### Required Behavior

- Enhanced PID lookup from specs
- Session-owned enhanced reads
- Module-targeted routing through physical addresses
- Explicit failure when module routing is unavailable

### Current Status

- [x] Enhanced PID definitions are loaded from specs.
- [x] Session-owned enhanced reads are implemented.
- [x] Module-targeted reads use resolved physical routing.
- [x] Explicit failure exists for missing routing context.

### Acceptance Checklist

- [x] Module-targeted reads succeed through session-owned routing.
- [x] Session tests validate missing discovery, unknown module, and wrong-bus behavior.

## Area 9: Diagnostic Session Operations

### Requirement

The library must support routed, session-owned diagnostic session control for targeted modules.

### Required Behavior

- Diagnostic session entry
- Security access
- Actuator control
- Actuator release
- Tester present
- Diagnostic session termination
- Security-aware session state tracking

### Current Status

- [x] Session entry is implemented.
- [x] Security access is implemented.
- [x] Actuator control is implemented.
- [x] Actuator release is implemented.
- [x] Tester present is implemented.
- [x] Session end is implemented.
- [x] Diagnostic state is tracked in `Session`.

### Acceptance Checklist

- [x] These flows go through session-owned routing.
- [x] Security gating is enforced for actuator operations.
- [x] Diagnostic state is externally inspectable.

## Area 10: ELM327/STN Adapter Behavior

### Requirement

The ELM/STN adapter path must be explicit, hardened, and adaptable across vehicle and adapter differences.

### Required Behavior

- Adapter initialization
- Chipset detection
- Capability reporting
- Protocol probing
- Protocol-specific runtime policy
- Addressed routing with `AT SH`
- Broadcast reset after targeted operations
- Clone/noise sanitization
- Typed adapter event classification
- Recovery from adapter fault states

### Current Status

- [x] Reset/init flow is implemented.
- [x] STN vs ELM detection is implemented.
- [x] Adapter capabilities are surfaced.
- [x] `AT SH` addressed routing is implemented for J1850, CAN 11-bit, and CAN 29-bit.
- [x] Broadcast reset after targeted routing is implemented.
- [x] Header caching is implemented.
- [x] NULL-byte sanitization is implemented.
- [x] Typed adapter events are implemented.
- [x] Recovery behavior exists for `ERR94` and `LV RESET`.
- [x] Status classification exists for `BUS BUSY`, `BUS ERROR`, `CAN ERROR`, `DATA ERROR`, `STOPPED`, and related cases.
- [x] K-line runtime policy exists for ISO/KWP initialization variants.

### Known Limitations

- [ ] J1939 addressed routing is not implemented.
- [~] Some ELM-specific advanced configuration remains policy-based rather than fully surfaced as consumer-level controls.

### Acceptance Checklist

- [x] Addressed routing tests exist for J1850 and CAN.
- [x] Header switching and reset behavior are tested.
- [x] Clone/noise fixtures cover NULL bytes and representative ELM error strings.

## Area 11: Protocol Probe And Selection

### Requirement

The library must support deterministic protocol selection, including fallback behavior for older vehicles.

### Required Behavior

- Auto-detect fast path
- Explicit fallback probing
- Recorded probe attempts and reasons
- Distinct unsupported-protocol handling
- K-line retry and init selection policy

### Current Status

- [x] Auto-detect path is implemented.
- [x] Explicit fallback probing is implemented.
- [x] Probe attempts are recorded.
- [x] Final protocol source is recorded.
- [x] Unsupported-protocol state is surfaced distinctly.
- [x] J1850/ISO/KWP/CAN ordering exists in the current policy.

### Acceptance Checklist

- [x] Fast-path and fallback-path tests exist.
- [x] Failed auto-detect followed by successful fallback is tested.
- [x] Replay fixtures cover at least one fallback probe path.

## Area 12: Protocol Decode Layer

### Requirement

Protocol framing and payload decode must live in dedicated protocol code, not be embedded primarily in adapter string parsing.

### Required Behavior

- Bus-family classification
- CAN headers-on and headers-off handling
- J1850 header parsing
- ISO/KWP header parsing
- ECU/source extraction where available
- Shared decoded-frame model
- Adapter payload decode through the codec path

### Current Status

- [x] A dedicated codec layer exists.
- [x] CAN decode support exists.
- [x] J1850 decode support exists.
- [x] ISO/KWP decode support exists.
- [x] Adapter payload decode now flows primarily through the codec path.
- [x] ECU/source extraction exists where protocol allows it.

### Known Limitations

- [ ] J1939 decoder boundary is not complete.
- [ ] J1939 reassembly and frame extraction are not complete.

### Acceptance Checklist

- [x] Bus-family fixtures exist for supported non-J1939 families.
- [x] Adapter response parsing uses codec helpers for primary decode behavior.

## Area 13: Transport Layer

### Requirement

The library must support multiple physical transport types while keeping protocol behavior out of the transport layer.

### Required Behavior

- Serial transport
- BLE transport
- Logging transport
- Mock transport
- Raw byte write/read/reset contract
- Optional chunk observation for capture/debugging

### Current Status

- [x] Transport trait exists.
- [x] Serial transport exists.
- [x] BLE transport exists.
- [x] Logging transport exists.
- [x] Mock transport exists.
- [x] Chunk observation exists.
- [x] Serial-like and BLE-like fragmented read shapes are tested in logging/capture coverage.

### Acceptance Checklist

- [x] Transport implementations remain byte-oriented.
- [x] Logging/capture can observe fragmented read behavior.

## Area 14: Raw Capture, Replay, And Field Debugging

### Requirement

The library must support field-debuggable operation through raw protocol capture, annotation, rename-on-identify behavior, and replay-friendly parsing.

### Required Behavior

- Debug-default raw capture
- Capture start on initialization
- Capture rename after VIN identification
- Structured annotations for:
  - session state
  - probe attempts
  - protocol selection
  - routing changes
  - adapter events
  - sanitization
  - recovery
- Replay parsing of capture files into command/response pairs

### Current Status

- [x] Raw capture defaults to enabled in debug builds.
- [x] Capture starts during initialization.
- [x] Capture is renamed using VIN after identification.
- [x] Capture annotations are implemented.
- [x] Logging transport can parse raw captures into replay pairs.
- [x] Replay-based tests exist for representative CAN and fallback/J1850 flows.

### Acceptance Checklist

- [x] Raw captures are useful for both field logging and automated replay tests.
- [x] Capture annotations include protocol/routing/adapter behavior.

## Area 15: ECU Visibility And Observability

### Requirement

The library must expose the difference between spec-declared topology and actually observed ECUs.

### Required Behavior

- Track visible ECUs during communication
- Record address/source observations
- Associate observed ECUs to known spec modules when possible
- Expose the observed set through discovery and session accessors

### Current Status

- [x] Visible ECU model exists.
- [x] Session tracks visible ECU observations.
- [x] Discovery includes visible ECUs.
- [x] Session exposes visible ECU access.

### Acceptance Checklist

- [x] Addressed traffic updates visible ECU state.
- [x] Observed ECUs can be correlated with spec modules when possible.

## Area 16: Polling

### Requirement

The library must support safe, session-owned polling without violating adapter single-flight behavior.

### Required Behavior

- Poll configuration
- Event emission
- Threshold alerting during poll cycles
- Optional battery-voltage reads
- Non-fatal error emission
- Session-owned poll-cycle execution

### Current Status

- [x] Poll configuration exists.
- [x] Poll events exist.
- [x] Poll-cycle execution is session-owned.
- [x] Threshold evaluation is integrated into polling.
- [x] Battery voltage can be included in polling.
- [x] Non-fatal poll errors are surfaced as events.

### Known Limitations

- [ ] A dedicated performance/regression harness for repeated PID polling is still open.

### Acceptance Checklist

- [x] Polling does not bypass session-owned routing/lifecycle behavior.
- [ ] Polling performance regression harness exists.

## Area 17: Single-Flight Safety

### Requirement

The library must prevent overlapping commands from reaching real adapters unsafely.

### Required Behavior

- Request contention must be prevented.
- Busy behavior must be explicit.
- Adapter-local operations that still touch the adapter must participate in the same guard model.

### Current Status

- [x] Single-flight guard exists in `Session`.
- [x] Adapter-busy error is surfaced.
- [x] Battery-voltage path participates in the busy guard.

### Acceptance Checklist

- [x] Concurrent unsafe session operations fail explicitly.
- [x] Polling and interactive requests share the same protection model.

## Area 18: Error Model

### Requirement

The library must expose explicit errors for the major failure classes users need to handle.

### Required Behavior

- Transport errors
- Adapter errors
- Busy errors
- Timeouts
- No data
- Unsupported PIDs
- Module-not-found
- Wrong-bus errors
- Negative responses
- Security-required errors
- Spec parse / parse / IO / generic errors

### Current Status

- [x] Core error model exists.
- [x] Negative response handling exists.
- [x] Routing-related explicit errors exist.
- [x] Busy errors exist.
- [x] Session tests cover missing discovery, unknown module, wrong bus, and busy cases.

### Acceptance Checklist

- [x] Common error classes are distinct and testable.
- [x] Errors are documented in the integration guide.

## Area 19: Testing And Verification

### Requirement

The project must have enough test coverage to prevent regressions across protocol, routing, lifecycle, and capture behavior.

### Required Coverage Categories

- unit tests for protocol/data decoding
- adapter tests for ELM behavior
- session tests for routing/lifecycle behavior
- integration tests for end-to-end session flows
- business-rule tests for spec/rule behavior
- raw-capture replay tests
- transport-shape tests

### Current Status

- [x] Unit tests exist broadly across protocol and session logic.
- [x] ELM adapter tests exist.
- [x] Session routing tests exist.
- [x] Integration tests exist.
- [x] Business-rule tests exist.
- [x] Replay tests exist.
- [x] Transport fragmentation tests exist.
- [~] Real hardware parity is still primarily simulated rather than exercised against physical devices in CI.

### Explicit Gaps

- [ ] Real USB vs BLE parity testing on physical hardware
- [ ] Real multi-vehicle hardware regression matrix
- [ ] Automated capture corpus from production hardware families beyond current representative fixtures

### Acceptance Checklist

- [x] The software test suite passes.
- [ ] The hardware compatibility matrix is fully exercised by repeatable automation.

## Area 20: Documentation

### Requirement

Documentation must clearly distinguish:

- supported current behavior
- incomplete behavior
- future roadmap behavior

### Required Documents

- README
- Integration guide
- Functional requirements
- Roadmap
- Protocol/reference material

### Current Status

- [x] README exists.
- [x] Integration guide exists.
- [x] Functional reference docs exist.
- [x] Roadmap exists.
- [x] This project-wide functional requirements document exists.
- [x] README, integration guide, and crate docs are aligned around the supported non-J1939 session-first surface.

### Acceptance Checklist

- [x] J1939 is explicitly treated as incomplete in the supported integration docs.
- [x] The docs identify `Session` as the primary surface.

## Area 21: Storage And Persistence

### Requirement

The project must provide a stable storage abstraction for persisting vehicle/session-related data, with at least one concrete backend crate.

### Required Behavior

- core storage traits in `obd2-core`
- concrete backend in `obd2-store-sqlite`
- storage separated from transport/adapter/session logic

### Current Status

- [x] Storage traits exist in `obd2-core`.
- [x] SQLite backend crate exists in the repository.
- [~] Storage is not the primary focus of the current pre-`1.0` communication redesign.

### Acceptance Checklist

- [x] Storage remains orthogonal to session/adapter correctness work.

## Area 22: J1939

### Requirement

J1939 must become a first-class, fully designed architecture before it can be considered part of the supported production surface.

### Required Behavior

- J1939 session-first public API
- J1939 discovery/profile representation
- J1939 physical routing model
- J1939 adapter initialization policy
- JE/JS byte-order policy
- request and monitor flows
- DM1 support
- transport-protocol reassembly
- address/PGN extraction
- tests and replay fixtures

### Current Status

- [~] J1939 data types and helper APIs exist.
- [ ] J1939 is not complete on the supported integration surface.
- [ ] J1939 addressed routing is not implemented.
- [ ] J1939 codec/reassembly is not complete.
- [ ] J1939 remains a separate workstream.

### Acceptance Checklist

- [ ] J1939 must not be represented as production-complete until the above is finished.

## Release-Readiness Summary

### Supported Now

- [x] Session-first non-J1939 public surface
- [x] Session-owned lifecycle, discovery, routing, diagnostics, and polling
- [x] J1850, ISO 9141, KWP2000, CAN 11-bit, and CAN 29-bit support on the supported surface
- [x] Raw capture, annotation, and replay parsing
- [x] Spec-backed module routing
- [x] Replay-assisted and transport-shape-aware tests

### Still Open Before Claiming “Complete”

- [ ] Full ignition-off/state richness validation
- [ ] Polling performance regression harness
- [ ] Real hardware parity matrix for USB and BLE adapters
- [ ] Broader capture corpus from representative real vehicles/adapters
- [ ] J1939 completion

## Master Feature Tracking Checklist

### Implemented

- [x] Session-first public API
- [x] Enforced session initialization
- [x] Discovery profile
- [x] Routed request model
- [x] Module-targeted routing
- [x] Standard OBD session methods
- [x] Enhanced PID support
- [x] Diagnostic session control
- [x] Polling
- [x] Single-flight execution
- [x] ELM/STN typed event model
- [x] Protocol probing and fallback
- [x] Raw capture and VIN rename
- [x] Raw-capture replay parsing
- [x] Transport-shape test coverage
- [x] Replay-based regression fixtures
- [x] Docs alignment for supported non-J1939 surface

### Remaining

- [ ] Ignition-off lifecycle verification
- [ ] Polling perf/regression harness
- [ ] Expanded real-hardware regression coverage
- [ ] Expanded real capture corpus
- [ ] J1939 first-class support

### Roadmap

- [>] Broader vehicle-family capture corpus
- [>] Physical USB/BLE adapter parity automation
- [>] ECU-discovered topology refinement beyond VIN-first identification
- [>] J1939 completion and promotion to supported surface

## Maintenance Rules

- Update this document whenever a feature meaningfully changes status.
- Do not mark a checklist item complete unless code, tests, and docs all support that claim.
- Use this document to answer “what is required?”, “what is done?”, and “what is still open?” before changing roadmap state.
