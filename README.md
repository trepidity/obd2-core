# obd2-core

Cross-platform OBD-II diagnostic library for Rust.

obd2-core is the shared foundation for [obd2-dash](https://github.com/trepidity/obd2-dash) (TUI diagnostic dashboard) and HaulLogic (commercial fleet management). It handles all OBD-II protocol logic, vehicle specifications, diagnostic intelligence, and adapter communication.

## Features

- **All 10 OBD-II diagnostic modes** — standard PIDs, freeze frame, DTCs (stored/pending/permanent), O2 monitoring, vehicle info, plus extended modes (session control, security access, actuator control)
- **Protocol-agnostic** — J1850 VPW/PWM, ISO 9141, KWP2000, CAN 11/29-bit, and J1939 through a unified API
- **Pluggable architecture** — open `Transport` and `Adapter` traits for custom hardware
- **Vehicle spec system** — embedded specs + runtime YAML loading with VIN-based matching
- **Diagnostic intelligence** — DTC enrichment with ~200 universal codes, diagnostic rules, known issue detection, threshold alerting
- **Offline VIN decoder** — manufacturer identification (50+), model year, engine/truck class — no network required
- **Polling engine** — configurable interval polling with event channels, threshold breach alerts, and battery monitoring
- **Cross-platform** — Windows, macOS, Linux, iOS, Android

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
obd2-core = { git = "https://github.com/trepidity/obd2-core" }
tokio = { version = "1", features = ["rt", "macros"] }
```

```rust
use obd2_core::adapter::mock::MockAdapter;
use obd2_core::session::Session;
use obd2_core::protocol::pid::Pid;

#[tokio::main]
async fn main() -> Result<(), obd2_core::error::Obd2Error> {
    let adapter = MockAdapter::new();
    let mut session = Session::new(adapter);

    // Identify vehicle (reads VIN, matches spec)
    let profile = session.identify_vehicle().await?;
    println!("Vehicle: {}", profile.vin);

    // Read engine RPM
    let rpm = session.read_pid(Pid::ENGINE_RPM).await?;
    println!("RPM: {:?}", rpm.value);

    // Read DTCs
    let dtcs = session.read_dtcs().await?;
    for dtc in &dtcs {
        println!("{}: {:?}", dtc.code, dtc.description);
    }

    Ok(())
}
```

## Crate Structure

```
obd2-core/
├── crates/
│   ├── obd2-core/           Main library
│   └── obd2-store-sqlite/   SQLite persistence backend
└── vehicle-specs/            Reference YAML spec files
```

## Module Overview

```
obd2-core/src/
├── protocol/     OBD-II types: PIDs, DTCs, values, services (pure data, no I/O)
├── transport/    Transport trait + serial, BLE, mock implementations
├── adapter/      Adapter trait + ELM327, mock implementations
├── vehicle/      Vehicle specs, VIN decoder, NHTSA, YAML loader
├── session/      Session orchestrator (primary consumer API)
├── store/        Storage traits (VehicleStore, SessionStore)
└── specs/        Embedded vehicle spec data
```

## Session API

`Session` is the primary entry point. It wraps an `Adapter` and provides high-level methods:

```rust
let mut session = Session::new(adapter);

// Vehicle identification
let profile = session.identify_vehicle().await?;     // Read VIN, match spec
let vin = session.read_vin().await?;                 // VIN only

// Standard PIDs (Mode 01)
let rpm = session.read_pid(Pid::ENGINE_RPM).await?;
let readings = session.read_pids(&[Pid::ENGINE_RPM, Pid::COOLANT_TEMP]).await?;
let supported = session.supported_pids().await?;

// DTCs (Mode 03/07/0A)
let stored = session.read_dtcs().await?;             // Confirmed codes
let pending = session.read_pending_dtcs().await?;    // Not yet confirmed
let permanent = session.read_permanent_dtcs().await?;// Cannot be cleared
session.clear_dtcs().await?;                         // Mode 04 — resets monitors

// Enhanced PIDs (Mode 22 — manufacturer-specific)
let boost = session.read_enhanced(0x0124, ModuleId::new("ecm")).await?;
let pids = session.module_pids(ModuleId::new("tcm"));

// O2 Sensor Monitoring (Mode 05)
let results = session.read_o2_monitoring(0x01).await?;
let all = session.read_all_o2_monitoring().await?;

// Battery voltage (adapter-level)
let voltage = session.battery_voltage().await?;

// Raw escape hatch
let data = session.raw_request(0x09, &[0x02], Target::Broadcast).await?;
```

## Polling

Continuous PID monitoring with event-driven updates:

```rust
use obd2_core::session::poller::{PollConfig, PollEvent};
use std::time::Duration;

let config = PollConfig {
    pids: vec![Pid::ENGINE_RPM, Pid::COOLANT_TEMP, Pid::VEHICLE_SPEED],
    interval: Duration::from_millis(250),
    read_voltage: true,
};

// Returns a handle for control and a receiver for events
let (handle, mut rx, _) = poller::start_poll_loop(config);

while let Some(event) = rx.recv().await {
    match event {
        PollEvent::Reading { pid, reading } => { /* update UI */ },
        PollEvent::Alert(result) => { /* threshold breach */ },
        PollEvent::Voltage(v) => { /* battery voltage */ },
        PollEvent::Error { pid, error } => { /* non-fatal */ },
        _ => {}
    }
}

handle.stop(); // Cancel polling
```

## Transport & Adapter Layers

### Architecture

```
Session  →  Adapter (protocol interpreter)  →  Transport (physical bytes)
```

- **Transport** = raw byte I/O over a physical medium (serial, BLE, WiFi)
- **Adapter** = protocol interpreter (translates OBD-II requests into adapter-specific commands)

### Built-in Transports

| Transport | Feature Flag | Description |
|-----------|-------------|-------------|
| `SerialTransport` | `serial` (default) | USB/RS-232 via tokio-serial, 115200 baud |
| `BleTransport` | `ble` | Bluetooth Low Energy via btleplug |
| `MockTransport` | always | Testing |

### Built-in Adapters

| Adapter | Description |
|---------|-------------|
| `Elm327Adapter` | ELM327/STN AT command protocol (genuine + clones) |
| `MockAdapter` | Realistic vehicle simulation for testing |

### Custom Transport

```rust
use obd2_core::transport::Transport;
use async_trait::async_trait;

struct WifiTransport { /* ... */ }

#[async_trait]
impl Transport for WifiTransport {
    async fn write(&mut self, data: &[u8]) -> Result<(), Obd2Error> { /* ... */ }
    async fn read(&mut self) -> Result<Vec<u8>, Obd2Error> { /* ... */ }
    async fn reset(&mut self) -> Result<(), Obd2Error> { /* ... */ }
    fn name(&self) -> &str { "wifi" }
}
```

## Vehicle Specs

Vehicle-specific data (enhanced PIDs, DTC descriptions, thresholds, diagnostic rules, known issues) lives in YAML spec files. The library ships with an embedded Duramax LLY spec and supports runtime loading.

```rust
// Load additional specs at runtime
session.load_spec(Path::new("specs/ford_powerstroke_2020.yaml"))?;
session.load_spec_dir(Path::new("specs/"))?;
```

Specs are matched by VIN (WMI prefix, 8th digit engine code, year range) or by make/model/year. See the [Integration Manual](docs/INTEGRATION.md) for the YAML schema.

## Diagnostic Intelligence

When a spec is matched, the library provides:

- **DTC enrichment** — manufacturer-specific descriptions layered over ~200 universal SAE J2012 codes
- **Diagnostic rules** — trigger actions when specific DTCs appear (e.g., "P0700 present → query TCM directly")
- **Known issues** — ranked list of common problems with symptom matching, root causes, quick tests, and repair guidance
- **Threshold alerting** — configurable warning/critical limits per PID per vehicle

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `serial` | Yes | Serial port transport (ELM327/STN via tokio-serial) |
| `embedded-specs` | Yes | Compile vehicle specs into the binary |
| `ble` | No | Bluetooth LE transport (btleplug) |
| `nhtsa` | No | Online VIN lookup via NHTSA vPIC API |
| `full` | No | Enable all features |

```toml
# BLE-only build (e.g., mobile app)
obd2-core = { git = "...", default-features = false, features = ["ble", "embedded-specs"] }

# Everything
obd2-core = { git = "...", features = ["full"] }
```

## Storage

obd2-core defines storage traits only — implementations live in separate crates:

```rust
// Traits in obd2-core
pub trait VehicleStore: Send + Sync {
    async fn save_vehicle(&self, profile: &VehicleProfile) -> Result<(), Obd2Error>;
    async fn get_vehicle(&self, vin: &str) -> Result<Option<VehicleProfile>, Obd2Error>;
    async fn save_thresholds(&self, vin: &str, ts: &ThresholdSet) -> Result<(), Obd2Error>;
    async fn get_thresholds(&self, vin: &str) -> Result<Option<ThresholdSet>, Obd2Error>;
}

pub trait SessionStore: Send + Sync {
    async fn save_reading(&self, vin: &str, pid: Pid, reading: &Reading) -> Result<(), Obd2Error>;
    async fn save_dtc_event(&self, vin: &str, dtcs: &[Dtc]) -> Result<(), Obd2Error>;
}
```

### obd2-store-sqlite

Ready-to-use SQLite backend:

```toml
[dependencies]
obd2-store-sqlite = { git = "https://github.com/trepidity/obd2-core" }
```

```rust
use obd2_store_sqlite::SqliteStore;

let store = SqliteStore::open(Path::new("diagnostics.db"))?;
// or
let store = SqliteStore::in_memory()?;
```

## Error Handling

All operations return `Result<T, Obd2Error>`. The error enum is `#[non_exhaustive]` for forward compatibility:

```rust
match session.read_pid(Pid::ENGINE_RPM).await {
    Ok(reading) => println!("RPM: {:?}", reading.value),
    Err(Obd2Error::Timeout) => println!("Vehicle didn't respond"),
    Err(Obd2Error::UnsupportedPid { pid }) => println!("PID {pid:#04x} not supported"),
    Err(Obd2Error::NoData) => println!("No data available"),
    Err(Obd2Error::NegativeResponse { service, nrc }) => {
        println!("ECU rejected: {nrc} for service {service:#04x}");
    }
    Err(e) => println!("Other error: {e}"),
}
```

## Testing

The `MockAdapter` simulates a realistic vehicle (configurable VIN, DTCs, standard PID responses):

```rust
let mut adapter = MockAdapter::with_vin("1GCHK23224F000001");
adapter.set_dtcs(vec![Dtc::from_code("P0420"), Dtc::from_code("P0171")]);
let mut session = Session::new(adapter);
```

Run the test suite:

```bash
cargo test --workspace
```

## Requirements

- Rust 1.75+
- Tokio runtime

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
