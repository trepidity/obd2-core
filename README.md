# obd2-core

Cross-platform OBD-II diagnostic library for Rust.

obd2-core is the shared foundation for [obd2-dash](https://github.com/trepidity/obd2-dash) (TUI diagnostic dashboard) and HaulLogic (commercial fleet management). It handles all OBD-II protocol logic, vehicle specifications, diagnostic intelligence, and adapter communication.

## Features

- **All 10 OBD-II diagnostic modes** — standard PIDs, freeze frame, DTCs (stored/pending/permanent), monitoring tests, vehicle info, enhanced manufacturer PIDs, session control, actuator tests
- **Protocol-agnostic** — supports J1850 VPW/PWM, ISO 9141, KWP2000, CAN 11/29-bit, and J1939 through a unified API
- **Multi-manufacturer** — GM, Ford, Ram, Honda, Toyota, BMW and more via YAML vehicle specs
- **Pluggable transports** — built-in serial (ELM327) and BLE, plus an open Transport trait for custom adapters
- **Vehicle spec system** — embedded default specs + runtime YAML loading with VIN-based matching
- **Diagnostic intelligence** — DTC enrichment, diagnostic rules, known issue detection, threshold alerting
- **Cross-platform** — Windows, macOS, Linux, iOS, Android

## Quick Start

```rust
use obd2_core::adapter::mock::MockAdapter;
use obd2_core::session::Session;
use obd2_core::protocol::pid::Pid;

#[tokio::main]
async fn main() -> Result<(), obd2_core::Obd2Error> {
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

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `serial` | Yes | Serial port transport (ELM327/STN via tokio-serial) |
| `embedded-specs` | Yes | Compile vehicle specs into the binary |
| `ble` | No | Bluetooth LE transport (btleplug) |
| `nhtsa` | No | Online VIN lookup via NHTSA vPIC API |
| `full` | No | Enable all features |

## Architecture

```
obd2-core/
├── protocol/    — OBD-II types: PIDs, DTCs, values, services (pure data, no I/O)
├── transport/   — Transport trait + serial, BLE, mock implementations
├── adapter/     — Adapter trait + ELM327, STN, mock implementations
├── vehicle/     — Vehicle specs, VIN decoder, NHTSA, YAML loader
├── session/     — Session orchestrator (primary consumer API)
└── store/       — Storage traits (VehicleStore, SessionStore)
```

## Vehicle Specs

Vehicle-specific data (enhanced PIDs, DTC descriptions, thresholds, diagnostic rules) lives in YAML spec files. The library ships with embedded defaults and supports runtime loading.

```rust
// Load additional specs at runtime
session.load_spec(Path::new("specs/ford_powerstroke_2020.yaml"))?;
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
