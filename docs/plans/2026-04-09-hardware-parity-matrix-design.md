# Hardware Parity Matrix Design

## Status

- Status: Approved
- Date: 2026-04-09
- Scope: Real hardware testing across USB and BLE transports

## Goal

Verify that `obd2-core` produces identical diagnostic results regardless of
whether the adapter is connected via USB serial or Bluetooth Low Energy. Catch
transport-specific regressions that mock tests cannot detect.

## Hardware Inventory

| Adapter | Chipset | Transport | Connection |
|---------|---------|-----------|------------|
| OBDLink EX | STN2230 | USB Serial | /dev/ttyUSB0 (or platform equivalent) |
| OBDLink MX+ | STN2120 | BLE | Auto-scan by adapter name |

| Vehicle | Year | Expected Protocol | J1939 | Spec Match |
|---------|------|-------------------|-------|------------|
| Chevy Duramax 2500 | 2006 | J1850 VPW | Yes | Yes (embedded LLY spec) |
| Chevy Malibu | 2020 | CAN 11-bit 500 kbps | No | No |
| Honda Accord | 2001 | ISO 9141 or KWP2000 | No | No |

## Architecture

### Crate Structure

A new binary crate `crates/obd2-hw-test/`:

```
crates/obd2-hw-test/
├── Cargo.toml
└── src/
    ├── main.rs         # CLI entry point (clap)
    ├── runner.rs       # Test matrix orchestrator
    ├── tests/
    │   ├── mod.rs
    │   ├── init.rs         # Adapter init + chipset detection
    │   ├── protocol.rs     # Protocol detection + probe recording
    │   ├── vin.rs          # VIN read + offline decode
    │   ├── pids.rs         # Standard PID reads + plausibility
    │   ├── supported.rs    # Supported PID bitmap
    │   ├── dtcs.rs         # DTC read (stored/pending/permanent)
    │   ├── voltage.rs      # Battery voltage plausibility
    │   ├── polling.rs      # Throughput measurement
    │   ├── capture.rs      # Raw capture + parseability
    │   ├── j1939.rs        # J1939 PGN reads (conditional)
    │   ├── enhanced.rs     # Enhanced PID reads (conditional)
    │   ├── monitoring.rs   # Readiness, freeze frame, O2, Mode 06
    │   └── recovery.rs     # Ignition-off recovery (interactive)
    ├── report.rs       # JSON report generation
    ├── compare.rs      # Two-report comparison + diff
    └── vehicles.rs     # Expected vehicle definitions
```

### CLI Interface

```bash
# Run full matrix
obd2-hw-test run --transport usb --port /dev/ttyUSB0 --vehicle duramax-2006

# Run specific test groups
obd2-hw-test run --transport ble --vehicle malibu-2020 --only init,pids,polling

# BLE with interactive recovery test
obd2-hw-test run --transport ble --vehicle duramax-2006 --interactive

# Compare USB vs BLE reports
obd2-hw-test compare results/duramax-usb.json results/duramax-ble.json

# List known vehicles
obd2-hw-test vehicles
```

### Transport Configuration

- **USB**: `--transport usb --port /dev/ttyUSB0` creates `SerialTransport`
- **BLE**: `--transport ble` creates `BleTransport` via auto-scan (OBDLink MX+ name matching)
- Both wrap in `Elm327Adapter` then `Session`

## Vehicle Definitions

Each vehicle has expected values the harness validates against:

```rust
struct ExpectedVehicle {
    id: &'static str,
    vin: &'static str,
    expected_protocol: Protocol,
    expected_make: &'static str,
    required_pids: &'static [Pid],
    optional_pids: &'static [Pid],
    has_j1939: bool,
    has_spec_match: bool,
    plausible_rpm_range: (f64, f64),
    plausible_coolant_range: (f64, f64),
}
```

VINs are the actual VINs of the test vehicles. These are not secrets — VINs
are publicly visible on dashboards and in government databases.

## Test Matrix

| Test Group | What it checks | Pass criteria |
|-----------|---------------|---------------|
| init | Reset, chipset detect, firmware string | Chipset == STN, firmware non-empty |
| protocol | Auto-detect finds correct protocol | Matches `expected_protocol` |
| vin | VIN read + offline decode | Exact VIN match, manufacturer decoded |
| pids | Read each required PID, check plausibility | Value in plausible range, no errors |
| supported | PID bitmap query | Contains all `required_pids` |
| dtcs | Read stored/pending/permanent | No transport errors (content varies) |
| voltage | Battery voltage | 11.0 – 15.5V |
| polling | 100 cycles of 3 PIDs, measure throughput | Completes, reports reads/sec |
| capture | Raw capture file created | File exists, `parse_raw_capture()` succeeds |
| j1939 | EEC1, CCVS, ET1, EFLP1, LFE, DM1 | Decodes successfully (Duramax only) |
| enhanced | Read spec-defined enhanced PIDs | Returns data (spec-matched only) |
| monitoring | Readiness, Mode 05, Mode 06 | No transport errors (NoData acceptable) |
| recovery | Ignition off → wait → on | State → IgnitionOff → Connected (interactive only) |

Conditional tests:
- `j1939` runs only when `has_j1939 == true`
- `enhanced` runs only when `has_spec_match == true`
- `recovery` runs only with `--interactive` flag

## JSON Report Format

```json
{
  "meta": {
    "timestamp": "2026-04-09T14:32:00Z",
    "harness_version": "0.1.0",
    "vehicle_id": "duramax-2006",
    "transport": "usb",
    "port": "/dev/ttyUSB0",
    "adapter_chipset": "Stn",
    "adapter_firmware": "STN2230 v5.10.3",
    "protocol_detected": "J1850Vpw",
    "raw_capture_path": "results/captures/duramax-2006-usb-20260409.obd2raw"
  },
  "summary": {
    "total": 12,
    "passed": 11,
    "failed": 0,
    "skipped": 1,
    "duration_secs": 34.2
  },
  "tests": {
    "<group_name>": {
      "status": "pass | fail | skipped",
      "duration_ms": 1200,
      "reason": "optional skip/fail reason",
      "details": { "...group-specific data..." }
    }
  }
}
```

Each test group records its own structured details (PID values, throughput
numbers, probe attempts, etc.) so reports are self-contained.

## Compare Command

The compare command diffs two JSON reports and categorizes differences:

| Severity | What it catches | Example |
|----------|----------------|---------|
| Critical | Protocol mismatch, VIN mismatch, test status mismatch | USB=pass, BLE=fail on same test |
| Warning | PID value divergence beyond tolerance, supported PID bitmap difference | Coolant differs by >5°C |
| Info | Throughput difference (BLE expected slower), timing deltas | USB: 25 reads/sec, BLE: 12 reads/sec |

Exit code 0 if parity holds (no critical differences). Exit code 1 if critical
differences found.

## CI Integration

```yaml
name: Hardware Parity Matrix
on:
  workflow_dispatch:
  schedule:
    - cron: '0 6 * * 1'  # Weekly Monday 6am

jobs:
  hardware-test:
    runs-on: self-hosted
    strategy:
      matrix:
        vehicle: [duramax-2006, malibu-2020, accord-2001]
        transport: [usb, ble]
    steps:
      - uses: actions/checkout@v4
      - run: cargo build -p obd2-hw-test --release
      - run: |
          cargo run -p obd2-hw-test --release -- run \
            --transport ${{ matrix.transport }} \
            --vehicle ${{ matrix.vehicle }} \
            --output results/${{ matrix.vehicle }}-${{ matrix.transport }}.json
      - uses: actions/upload-artifact@v4
        with:
          name: hw-results-${{ matrix.vehicle }}-${{ matrix.transport }}
          path: results/

  compare:
    needs: hardware-test
    runs-on: self-hosted
    steps:
      - uses: actions/download-artifact@v4
      - run: |
          for vehicle in duramax-2006 malibu-2020 accord-2001; do
            cargo run -p obd2-hw-test --release -- compare \
              results/${vehicle}-usb.json \
              results/${vehicle}-ble.json
          done
```

Requirements for CI:
- Self-hosted runner on a machine with both adapters connected
- Vehicle connected with ignition on (or bench ECU simulator)
- `--interactive` is NOT passed in CI (recovery test skipped)

For manual field runs, results are committed to `test-results/` in the repo.

## Dependencies

```toml
[dependencies]
obd2-core = { path = "../obd2-core", features = ["full"] }
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
colored = "2"
```

## Non-Goals

- Simulating hardware in CI (mock tests already cover this)
- Testing every PID exhaustively (plausibility checks, not calibration)
- Flash/reprogram operations
- Performance benchmarking to sub-millisecond precision
- Supporting non-STN adapters in the first version

## Success Criteria

1. Same vehicle + same adapter over USB and BLE produces matching VIN, protocol, supported PIDs, and PID values within tolerance.
2. All three protocol families (CAN, J1850, ISO/KWP) are exercised.
3. J1939 reads decode successfully on the Duramax.
4. Polling throughput is captured for USB vs BLE comparison.
5. Raw captures are parseable and can serve as future replay fixtures.
6. CI runs weekly without manual intervention (given hardware is connected).
