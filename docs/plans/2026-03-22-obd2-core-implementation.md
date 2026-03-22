# obd2-core Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a standalone cross-platform OBD-II diagnostic library at ~/Projects/obd2-core from scratch, based on the approved design doc at docs/plans/2026-03-22-obd2-core-design.md.

**Architecture:** Workspace with two crates: obd2-core (the library) and obd2-store-sqlite (optional storage). Core is layered: protocol types (pure data) > transport/adapter traits (I/O) > vehicle specs (YAML) > session (orchestrator). All manufacturer-specific data lives in YAML specs; standard OBD-II is hardcoded.

**Tech Stack:** Rust 2021 edition, tokio async runtime, thiserror for errors, serde_yaml for spec loading, async-trait for trait objects, tracing for logging. Optional: tokio-serial, btleplug, reqwest, rusqlite.

**Design doc:** /Users/jared/Projects/obd2-dash/docs/plans/2026-03-22-obd2-core-design.md

**Existing reference:** /Users/jared/Projects/obd2-dash/crates/obd2-core/ (old implementation, for reference only — do NOT copy directly)

**Vehicle specs:** /Users/jared/Projects/obd2-dash/vehicle-specs/ (copy into new repo)

---

## Dependency Graph & Parallelization

```
Phase 1 (parallel — no dependencies between these):
  Task 1: Scaffold workspace + Cargo.toml
  Task 2: Error types
  Task 3: Protocol types (Value, Pid, Dtc, Module, etc.)
  Task 4: Vehicle spec data structures

Phase 2 (parallel — depends on Phase 1):
  Task 5: PID parsing (43 standard PIDs with formulas)
  Task 6: DTC decoding (byte format + 200+ descriptions)
  Task 7: VIN decoder (offline)
  Task 8: Transport + Adapter traits
  Task 9: Store traits

Phase 3 (parallel — depends on Phase 2):
  Task 10: YAML spec loader + embedded specs
  Task 11: MockTransport + MockAdapter
  Task 12: ELM327 adapter (AT command protocol)

Phase 4 (parallel — depends on Phase 3):
  Task 13: Session core (construction, read_pid, read_dtcs, identify_vehicle)
  Task 14: Threshold evaluation
  Task 15: NHTSA online lookup

Phase 5 (parallel — depends on Phase 4):
  Task 16: Session enhanced (Mode 22, multi-module, bus switching)
  Task 17: Session diagnostics (Mode 19, freeze frame, test results, clear DTCs)
  Task 18: Diagnostic intelligence (rules, known issues, DTC enrichment)

Phase 6 (parallel — depends on Phase 5):
  Task 19: Polling loop + PollEvent channel
  Task 20: Session management (Mode 10/27/2F/3E)
  Task 21: Serial transport
  Task 22: BLE transport

Phase 7 (sequential — integration):
  Task 23: obd2-store-sqlite crate
  Task 24: Integration tests (full Session with MockAdapter)
  Task 25: Crate-level docs + README + publish prep
```

---

## Phase 1: Foundation (all parallel, no dependencies)

### Task 1: Scaffold Workspace

**Files:**
- Create: `~/Projects/obd2-core/Cargo.toml`
- Create: `~/Projects/obd2-core/crates/obd2-core/Cargo.toml`
- Create: `~/Projects/obd2-core/crates/obd2-core/src/lib.rs`
- Create: `~/Projects/obd2-core/crates/obd2-store-sqlite/Cargo.toml`
- Create: `~/Projects/obd2-core/crates/obd2-store-sqlite/src/lib.rs`
- Create: `~/Projects/obd2-core/.gitignore`
- Copy: `vehicle-specs/` from obd2-dash

**Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/obd2-core",
    "crates/obd2-store-sqlite",
]

[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
repository = "https://github.com/trepidity/obd2-core"

[workspace.dependencies]
tokio = { version = "1", features = ["rt", "time", "sync", "macros"] }
async-trait = "0.1"
thiserror = "2"
tracing = "0.1"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
```

**Step 2: Create obd2-core Cargo.toml**

```toml
[package]
name = "obd2-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "Cross-platform OBD-II diagnostic library"
keywords = ["obd2", "obd-ii", "diagnostics", "automotive", "elm327"]
categories = ["hardware-support"]

[features]
default = ["serial", "embedded-specs"]
serial = ["dep:tokio-serial"]
ble = ["dep:btleplug"]
embedded-specs = []
nhtsa = ["dep:reqwest"]
full = ["serial", "ble", "embedded-specs", "nhtsa"]

[dependencies]
tokio = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_yaml = { workspace = true }

# Optional transports
tokio-serial = { version = "5", optional = true }
btleplug = { version = "0.11", optional = true }

# Optional online services
reqwest = { version = "0.12", optional = true, default-features = false, features = ["json", "rustls-tls"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
tokio-test = "0.4"
```

**Step 3: Create lib.rs with module skeleton and deny(missing_docs)**

```rust
//! # obd2-core
//!
//! Cross-platform OBD-II diagnostic library for Rust.
//!
//! Supports serial (ELM327/STN), Bluetooth LE, and custom adapters
//! on Windows, macOS, Linux, iOS, and Android.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use obd2_core::{Pid, Session};
//!
//! # async fn example() -> Result<(), obd2_core::Obd2Error> {
//! // Session setup omitted — see Session docs
//! // let mut session = ...;
//! // let rpm = session.read_pid(Pid::ENGINE_RPM).await?;
//! // println!("RPM: {:?}", rpm.value);
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

pub mod error;
pub mod protocol;
pub mod transport;
pub mod adapter;
pub mod vehicle;
pub mod session;
pub mod store;

// Re-exports
pub use error::Obd2Error;
pub use protocol::{
    Pid, EnhancedPid, Value, Bitfield, Reading, ReadingSource, ValueType, Formula,
    Dtc, DtcCategory, DtcStatus, DtcStatusByte, Severity, Confidence,
    NegativeResponse, ReadinessStatus, MonitorStatus, TestResult, VehicleInfo,
};
pub use transport::Transport;
pub use adapter::{Adapter, AdapterInfo, Chipset, Capabilities};
pub use vehicle::{
    VehicleSpec, VehicleProfile, SpecRegistry, ModuleId, Module,
    PhysicalAddress, BusId, BusConfig, Protocol, KLineInit, Target,
};
pub use session::Session;
pub use store::{VehicleStore, SessionStore};
```

**Step 4: Create placeholder modules (just enough to compile)**

Each module file (error.rs, protocol/mod.rs, transport/mod.rs, etc.) starts as:

```rust
//! Module description here.
```

**Step 5: Create .gitignore, init git, copy vehicle-specs**

```bash
cd ~/Projects
mkdir -p obd2-core
cd obd2-core
git init
# Create .gitignore with /target, *.swp, .env*.local
# Copy vehicle-specs/ from obd2-dash
# mkdir -p docs/plans
# Copy design doc
```

**Step 6: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors (modules are empty but exist)

**Step 7: Commit**

```bash
git add -A
git commit -m "chore: scaffold obd2-core workspace"
```

---

### Task 2: Error Types

**Files:**
- Create: `crates/obd2-core/src/error.rs`
- Test: `crates/obd2-core/src/error.rs` (doc tests)

**Step 1: Write tests for error type behavior**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Obd2Error::Timeout;
        assert_eq!(err.to_string(), "timeout waiting for response");
    }

    #[test]
    fn test_nrc_display() {
        let err = Obd2Error::NegativeResponse {
            service: 0x22,
            nrc: NegativeResponse::RequestOutOfRange,
        };
        assert!(err.to_string().contains("RequestOutOfRange"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "gone");
        let obd_err: Obd2Error = io_err.into();
        assert!(matches!(obd_err, Obd2Error::Io(_)));
    }
}
```

**Step 2: Run tests — verify they fail**

Run: `cargo test -p obd2-core error`
Expected: compilation error (types don't exist yet)

**Step 3: Implement error types**

Implement the full `Obd2Error` enum and `NegativeResponse` enum from the design doc Section 6. Use `#[derive(Debug, thiserror::Error)]` and `#[non_exhaustive]` on both.

Reference: design doc Section 6 "Error Type" and Section 2G "Negative Response Codes".

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core error`
Expected: all 3 tests pass

**Step 5: Commit**

```bash
git commit -am "feat: add Obd2Error and NegativeResponse types"
```

---

### Task 3: Protocol Types

**Files:**
- Create: `crates/obd2-core/src/protocol/mod.rs`
- Create: `crates/obd2-core/src/protocol/pid.rs` (Pid struct + constants only, no parsing yet)
- Create: `crates/obd2-core/src/protocol/dtc.rs` (Dtc struct + enums only, no decoding yet)
- Create: `crates/obd2-core/src/protocol/enhanced.rs` (EnhancedPid, Formula)
- Create: `crates/obd2-core/src/protocol/service.rs` (ServiceRequest, diagnostic session types)
- Create: `crates/obd2-core/src/protocol/codec.rs` (placeholder)

This task defines ALL data types from design doc Section 2. No logic — just structs, enums, derives, and trait impls (Debug, Clone, PartialEq, etc.).

**Step 1: Write tests for core type construction and equality**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_constants() {
        assert_eq!(Pid::ENGINE_RPM.0, 0x0C);
        assert_eq!(Pid::COOLANT_TEMP.0, 0x05);
    }

    #[test]
    fn test_value_as_f64() {
        let v = Value::Scalar(42.0);
        assert_eq!(v.as_f64().unwrap(), 42.0);

        let v = Value::Bitfield(Bitfield { raw: 0xFF, flags: vec![] });
        assert!(v.as_f64().is_err());
    }

    #[test]
    fn test_dtc_category() {
        let dtc = Dtc {
            code: "P0420".into(),
            category: DtcCategory::Powertrain,
            status: DtcStatus::Stored,
            description: None,
            severity: None,
            source_module: None,
            notes: None,
        };
        assert_eq!(dtc.category, DtcCategory::Powertrain);
    }

    #[test]
    fn test_module_id_constants() {
        let ecm = ModuleId::new(ModuleId::ECM);
        let also_ecm = ModuleId::new("ecm");
        assert_eq!(ecm, also_ecm);
    }

    #[test]
    fn test_reading_source() {
        assert_ne!(ReadingSource::Live, ReadingSource::FreezeFrame);
    }
}
```

**Step 2: Run tests — verify they fail**

Run: `cargo test -p obd2-core protocol`
Expected: compilation error

**Step 3: Implement all types from design doc Sections 2A through 2G**

Implement in this order:
1. `protocol/mod.rs` — re-exports
2. `protocol/pid.rs` — Pid newtype, ValueType, all 43+ named constants, name()/unit()/response_bytes()/value_type() methods (parse() is Task 5)
3. `protocol/dtc.rs` — Dtc, DtcCategory, DtcStatus, DtcStatusByte, Severity (decoding is Task 6)
4. `protocol/enhanced.rs` — EnhancedPid, Formula, Confidence
5. `protocol/service.rs` — NegativeResponse (re-export from error), DiagSession, ActuatorCommand, ReadinessStatus, MonitorStatus, TestResult, VehicleInfo, DtcDetail
6. `protocol/codec.rs` — placeholder module doc

Also implement Value, Bitfield, Reading, ReadingSource with the `as_f64()` and `as_bitfield()` convenience methods.

Reference: design doc Sections 2A-2G for every type definition.

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core protocol`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: add all protocol types (Pid, Dtc, Value, Module, etc.)"
```

---

### Task 4: Vehicle Spec Data Structures

**Files:**
- Create: `crates/obd2-core/src/vehicle/mod.rs`
- Create: `crates/obd2-core/src/vehicle/spec.rs`

This task defines the vehicle spec types from design doc Section 4. No loading logic — just the data structures that YAML deserializes into.

**Step 1: Write tests for spec type construction**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_identity() {
        let id = SpecIdentity {
            name: "Test Vehicle".into(),
            model_years: (2020, 2022),
            makes: vec!["TestMake".into()],
            models: vec!["TestModel".into()],
            engine: EngineSpec {
                code: "TEST".into(),
                displacement_l: 2.0,
                cylinders: 4,
                layout: "I4".into(),
                aspiration: "Turbo".into(),
                fuel_type: "Gasoline".into(),
                fuel_system: None,
                compression_ratio: None,
                max_power_kw: None,
                max_torque_nm: None,
                redline_rpm: 6500,
                idle_rpm_warm: 700,
                idle_rpm_cold: 900,
                firing_order: None,
                ecm_hardware: None,
            },
            transmission: None,
            vin_match: None,
        };
        assert_eq!(id.model_years.0, 2020);
    }

    #[test]
    fn test_transmission_types() {
        let cvt = TransmissionType::Cvt {
            ratio_range: (0.4, 2.6),
            simulated_steps: Some(7),
        };
        assert!(matches!(cvt, TransmissionType::Cvt { .. }));
    }

    #[test]
    fn test_physical_address_variants() {
        let j1850 = PhysicalAddress::J1850 { node: 0x10, header: [0x6C, 0x10, 0xF1] };
        let can = PhysicalAddress::Can11Bit { request_id: 0x7E0, response_id: 0x7E8 };
        // Both construct without panic
        assert!(matches!(j1850, PhysicalAddress::J1850 { .. }));
        assert!(matches!(can, PhysicalAddress::Can11Bit { .. }));
    }

    #[test]
    fn test_threshold_evaluate_normal() {
        let t = Threshold {
            min: Some(0.0), max: Some(120.0),
            warning_low: None, warning_high: Some(105.0),
            critical_low: None, critical_high: Some(115.0),
            unit: "°C".into(),
        };
        assert!(t.evaluate(90.0, "coolant").is_none()); // normal
    }

    #[test]
    fn test_threshold_evaluate_warning() {
        let t = Threshold {
            min: Some(0.0), max: Some(120.0),
            warning_low: None, warning_high: Some(105.0),
            critical_low: None, critical_high: Some(115.0),
            unit: "°C".into(),
        };
        let result = t.evaluate(110.0, "coolant");
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, AlertLevel::Warning);
    }

    #[test]
    fn test_threshold_evaluate_critical() {
        let t = Threshold {
            min: Some(0.0), max: Some(120.0),
            warning_low: None, warning_high: Some(105.0),
            critical_low: None, critical_high: Some(115.0),
            unit: "°C".into(),
        };
        let result = t.evaluate(118.0, "coolant");
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, AlertLevel::Critical);
    }
}
```

**Step 2: Run tests — verify they fail**

Run: `cargo test -p obd2-core vehicle`
Expected: compilation error

**Step 3: Implement all vehicle spec types**

Implement in this order:
1. `vehicle/mod.rs` — ModuleId, Module, PhysicalAddress, BusId, BusConfig, Protocol, KLineInit, Target (re-exports)
2. `vehicle/spec.rs` — VehicleSpec, SpecIdentity, VinMatcher, EngineSpec, TransmissionSpec, TransmissionType, CommunicationSpec, InitSequence, InitCommand, StandardPidSpec, OperatingRange, Threshold, ThresholdSet, NamedThreshold, ThresholdResult, AlertLevel, AlertDirection, DtcLibrary, DtcEntry, PollingGroup, PollStep, DiagnosticRule, RuleTrigger, RuleAction, KnownIssue, QuickTest, ReadinessSpec, MonitorSpec

Implement `Threshold::evaluate()` method (pure logic, no I/O).

All types derive `Debug, Clone` and `Deserialize` (serde) where applicable.

Reference: design doc Section 4 for all type definitions.

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core vehicle`
Expected: all 5 tests pass

**Step 5: Commit**

```bash
git commit -am "feat: add vehicle spec data structures and threshold evaluation"
```

---

## Phase 2: Core Logic (all parallel, depends on Phase 1)

### Task 5: PID Parsing (43 Standard PIDs)

**Files:**
- Modify: `crates/obd2-core/src/protocol/pid.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write tests for PID parsing formulas**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rpm() {
        // RPM = (256*A + B) / 4
        let data = [0x0C, 0x00]; // 3072 / 4 = 768 RPM
        let val = Pid::ENGINE_RPM.parse(&data).unwrap();
        assert_eq!(val.as_f64().unwrap(), 768.0);
    }

    #[test]
    fn test_parse_speed() {
        let data = [0x3C]; // 60 km/h
        let val = Pid::VEHICLE_SPEED.parse(&data).unwrap();
        assert_eq!(val.as_f64().unwrap(), 60.0);
    }

    #[test]
    fn test_parse_coolant_temp() {
        // Temp = A - 40
        let data = [0x7E]; // 126 - 40 = 86°C
        let val = Pid::COOLANT_TEMP.parse(&data).unwrap();
        assert_eq!(val.as_f64().unwrap(), 86.0);
    }

    #[test]
    fn test_parse_fuel_trim() {
        // Fuel trim = (A - 128) * 100 / 128
        let data = [0x80]; // (128 - 128) * 100 / 128 = 0%
        let val = Pid(0x06).parse(&data).unwrap(); // Short fuel trim B1
        assert_eq!(val.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_parse_control_module_voltage() {
        // Voltage = (256*A + B) / 1000
        let data = [0x38, 0x5C]; // 14428 / 1000 = 14.428V
        let val = Pid(0x42).parse(&data).unwrap();
        assert!((val.as_f64().unwrap() - 14.428).abs() < 0.001);
    }

    #[test]
    fn test_parse_monitor_status_bitfield() {
        let data = [0x00, 0x07, 0x65, 0x00]; // MIL off, 0 DTCs, diesel
        let val = Pid::MONITOR_STATUS.parse(&data).unwrap();
        assert!(matches!(val, Value::Bitfield(_)));
    }

    #[test]
    fn test_parse_insufficient_bytes() {
        let data = [0x0C]; // RPM needs 2 bytes, only 1 provided
        let result = Pid::ENGINE_RPM.parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_pids_have_names() {
        for pid in Pid::all() {
            assert!(!pid.name().is_empty(), "PID {:#04x} has no name", pid.0);
            assert!(!pid.unit().is_empty(), "PID {:#04x} has no unit", pid.0);
        }
    }
}
```

**Step 2: Run tests — verify they fail**

Run: `cargo test -p obd2-core pid`
Expected: parse() not implemented

**Step 3: Implement PID parsing**

Port parsing logic from the existing `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/pid.rs` as reference, but rewrite to return `Value` enum (not just f64). Key changes:
- PID 0x01 (Monitor Status) returns `Value::Bitfield`
- All percentage PIDs return `Value::Scalar`
- All temperature PIDs return `Value::Scalar` with offset formula
- Add `Pid::all()` returning &[Pid] of all 43+ PIDs
- Validate byte count before parsing

Reference formulas: SAE J1979 (see spec_sources.yaml source SAE-J1979).

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core pid`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: implement PID parsing for 43 standard OBD-II PIDs"
```

---

### Task 6: DTC Decoding

**Files:**
- Modify: `crates/obd2-core/src/protocol/dtc.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write tests for DTC byte decoding and description lookup**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dtc_from_bytes() {
        // P0420: byte1 = 0x04, byte2 = 0x20
        let dtc = Dtc::from_bytes(0x04, 0x20);
        assert_eq!(dtc.code, "P0420");
        assert_eq!(dtc.category, DtcCategory::Powertrain);
    }

    #[test]
    fn test_dtc_from_bytes_chassis() {
        // C0035: byte1 = 0x40, byte2 = 0x35
        let dtc = Dtc::from_bytes(0x40, 0x35);
        assert_eq!(dtc.code, "C0035");
        assert_eq!(dtc.category, DtcCategory::Chassis);
    }

    #[test]
    fn test_dtc_from_bytes_body() {
        // B0083: byte1 = 0x80, byte2 = 0x83
        let dtc = Dtc::from_bytes(0x80, 0x83);
        assert_eq!(dtc.code, "B0083");
        assert_eq!(dtc.category, DtcCategory::Body);
    }

    #[test]
    fn test_dtc_from_bytes_network() {
        // U0100: byte1 = 0xC1, byte2 = 0x00
        let dtc = Dtc::from_bytes(0xC1, 0x00);
        assert_eq!(dtc.code, "U0100");
        assert_eq!(dtc.category, DtcCategory::Network);
    }

    #[test]
    fn test_universal_dtc_description() {
        let desc = universal_dtc_description("P0420");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("Catalyst"));
    }

    #[test]
    fn test_unknown_dtc_description() {
        let desc = universal_dtc_description("P9999");
        assert!(desc.is_none());
    }

    #[test]
    fn test_dtc_status_byte_decode() {
        let status = DtcStatusByte::from_byte(0x0B); // bits 0,1,3 set
        assert!(status.test_failed);
        assert!(status.test_failed_this_cycle);
        assert!(!status.pending);
        assert!(status.confirmed);
    }
}
```

**Step 2: Run tests — verify they fail**

Run: `cargo test -p obd2-core dtc`
Expected: compilation error

**Step 3: Implement DTC decoding**

1. `Dtc::from_bytes(a, b)` — decode 2-byte DTC per SAE J2012 format (bits 15-14 = category, etc.)
2. `DtcStatusByte::from_byte(b)` — decode GM/UDS Mode 19 status byte
3. `universal_dtc_description(code) -> Option<&'static str>` — static map of 200+ SAE J2012 P0xxx codes

Port the description table from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/dtc.rs` as reference.

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core dtc`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: implement DTC decoding and 200+ universal descriptions"
```

---

### Task 7: VIN Decoder (Offline)

**Files:**
- Create: `crates/obd2-core/src/vehicle/vin.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write tests for VIN decoding**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_year_from_10th_char() {
        assert_eq!(decode_year("1GCHK23164F000001"), Some(2004));
        assert_eq!(decode_year("WMWRE33546T000001"), Some(2006));
    }

    #[test]
    fn test_decode_manufacturer() {
        assert_eq!(decode_manufacturer("1GC"), Some("Chevrolet"));
        assert_eq!(decode_manufacturer("WMW"), Some("MINI"));
        assert_eq!(decode_manufacturer("1FT"), Some("Ford"));
    }

    #[test]
    fn test_decode_engine_code_8th_digit() {
        let vin = "1GCHK23124F000001"; // 8th digit = '2' = LLY
        assert_eq!(vin.chars().nth(7), Some('2'));
    }

    #[test]
    fn test_vin_matcher_matches() {
        let matcher = VinMatcher {
            vin_8th_digit: Some(vec!['2']),
            wmi_prefixes: vec!["1GC".into(), "1GT".into()],
            year_range: Some((2004, 2005)),
        };
        assert!(matcher.matches("1GCHK23124F000001")); // 2004, 1GC, digit 2
        assert!(!matcher.matches("1GCHK23114F000001")); // digit 1 = LB7, not LLY
    }

    #[test]
    fn test_detect_truck_class() {
        assert_eq!(detect_truck_class("1GCHK23124F000001"), Some("diesel-truck"));
    }

    #[test]
    fn test_invalid_vin_length() {
        assert_eq!(decode_year("SHORT"), None);
    }
}
```

**Step 2: Run tests — verify they fail**

Run: `cargo test -p obd2-core vin`
Expected: compilation error

**Step 3: Implement VIN decoder**

Port from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/vin.rs` as reference. Implement:
1. `decode_year(vin)` — 10th character year decoding
2. `decode_manufacturer(wmi)` — 50+ WMI codes
3. `detect_truck_class(vin)` — diesel-truck, gas-truck-v8, etc.
4. `VinMatcher::matches(vin)` — spec matching logic

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core vin`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: implement offline VIN decoder with 50+ manufacturers"
```

---

### Task 8: Transport & Adapter Traits

**Files:**
- Create: `crates/obd2-core/src/transport/mod.rs`
- Create: `crates/obd2-core/src/adapter/mod.rs`
- Create: `crates/obd2-core/src/adapter/detect.rs`

**Step 1: Write tests for adapter info detection**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chipset_detection_elm327() {
        let info = AdapterInfo::detect("ELM327 v1.5", None);
        assert!(matches!(info.chipset, Chipset::Elm327Clone));
    }

    #[test]
    fn test_chipset_detection_stn() {
        let info = AdapterInfo::detect("ELM327 v1.5", Some("STN2120"));
        assert!(matches!(info.chipset, Chipset::Stn));
    }

    #[test]
    fn test_stn_has_more_capabilities() {
        let elm = AdapterInfo::detect("ELM327 v1.5", None);
        let stn = AdapterInfo::detect("ELM327 v2.2", Some("STN1110"));
        assert!(!elm.capabilities.dual_can);
        assert!(stn.capabilities.dual_can);
    }
}
```

**Step 2: Run tests — verify they fail**

**Step 3: Implement traits and detection**

1. `Transport` trait with async write/read/reset/name
2. `Adapter` trait with async initialize/request/supported_pids/battery_voltage/info
3. `AdapterInfo`, `Chipset` enum, `Capabilities` struct
4. `AdapterInfo::detect()` — chipset detection from ATZ/STI responses

Port detection logic from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/types.rs`.

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core adapter`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: add Transport and Adapter traits with chipset detection"
```

---

### Task 9: Store Traits

**Files:**
- Create: `crates/obd2-core/src/store/mod.rs`

**Step 1: Implement VehicleStore and SessionStore traits**

These are trait definitions only — no implementation. Just the trait + associated types.

```rust
//! Storage traits for persisting vehicle and session data.
//!
//! obd2-core defines traits only. Implementations live in separate crates
//! (e.g., obd2-store-sqlite) or are provided by consumers.

use async_trait::async_trait;
use crate::{Obd2Error, Pid, Reading, Dtc};
use crate::vehicle::{VehicleProfile, ThresholdSet};

/// Persist and retrieve vehicle profiles and thresholds.
#[async_trait]
pub trait VehicleStore: Send + Sync {
    /// Save or update a vehicle profile.
    async fn save_vehicle(&self, profile: &VehicleProfile) -> Result<(), Obd2Error>;
    /// Retrieve a vehicle profile by VIN.
    async fn get_vehicle(&self, vin: &str) -> Result<Option<VehicleProfile>, Obd2Error>;
    /// Save threshold overrides for a VIN.
    async fn save_thresholds(&self, vin: &str, thresholds: &ThresholdSet) -> Result<(), Obd2Error>;
    /// Retrieve threshold overrides for a VIN.
    async fn get_thresholds(&self, vin: &str) -> Result<Option<ThresholdSet>, Obd2Error>;
}

/// Persist diagnostic session data for history.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Save a PID reading.
    async fn save_reading(&self, vin: &str, pid: Pid, reading: &Reading) -> Result<(), Obd2Error>;
    /// Save a DTC event.
    async fn save_dtc_event(&self, vin: &str, dtcs: &[Dtc]) -> Result<(), Obd2Error>;
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p obd2-core`
Expected: compiles

**Step 3: Commit**

```bash
git commit -am "feat: add VehicleStore and SessionStore traits"
```

---

## Phase 3: Integration Layer (depends on Phase 2)

### Task 10: YAML Spec Loader + Embedded Specs

**Files:**
- Create: `crates/obd2-core/src/vehicle/loader.rs`
- Create: `crates/obd2-core/src/specs/embedded/mod.rs`
- Copy: `chevy_duramax_2004_turbo.yaml` into `crates/obd2-core/src/specs/embedded/`

**Step 1: Write tests for YAML loading**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_duramax_spec() {
        let yaml = include_str!("../../specs/embedded/chevy_duramax_2004_turbo.yaml");
        let spec = load_spec_from_str(yaml).expect("failed to parse duramax spec");
        assert_eq!(spec.identity.engine.code, "LLY");
        assert_eq!(spec.identity.model_years, (2004, 2005));
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = SpecRegistry::with_defaults();
        assert!(registry.specs().len() >= 1); // at least the duramax
    }

    #[test]
    fn test_registry_match_vin() {
        let registry = SpecRegistry::with_defaults();
        // Duramax VIN: 8th digit = '2', WMI = "1GC", year = 2004
        let matched = registry.match_vin("1GCHK23124F000001");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().identity.engine.code, "LLY");
    }

    #[test]
    fn test_registry_no_match() {
        let registry = SpecRegistry::with_defaults();
        let matched = registry.match_vin("JH4KA7660PC000001"); // Acura, no spec
        assert!(matched.is_none());
    }

    #[test]
    fn test_load_spec_from_file() {
        let mut registry = SpecRegistry::with_defaults();
        // Load the same file from the vehicle-specs directory
        let result = registry.load_file(Path::new("../../vehicle-specs/chevy_duramax_2004_turbo.yaml"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_dtc_lookup_in_registry() {
        let registry = SpecRegistry::with_defaults();
        let entry = registry.lookup_dtc("P0087");
        // If the duramax spec has P0087, it should return it
        if let Some(e) = entry {
            assert!(e.meaning.contains("Fuel Rail"));
        }
    }
}
```

**Step 2: Run tests — verify they fail**

**Step 3: Implement spec loader**

1. `load_spec_from_str(yaml: &str) -> Result<VehicleSpec, Obd2Error>` — serde_yaml deserialization
2. `SpecRegistry::with_defaults()` — loads embedded specs via `include_str!()`
3. `SpecRegistry::load_file(path)` — loads YAML from filesystem
4. `SpecRegistry::load_directory(dir)` — loads all .yaml files in a directory
5. `SpecRegistry::match_vin(vin)` — matches using VinMatcher from specs
6. `SpecRegistry::match_vehicle(make, model, year)` — matches by identity fields
7. `SpecRegistry::lookup_dtc(code)` — searches DTC libraries across all specs

The YAML schema must match the VehicleSpec struct hierarchy. You may need to add `#[serde(rename)]` or custom deserializers for some fields in the Duramax YAML (it was authored before the Rust types existed, so field names may differ slightly). Adjust the YAML or add serde attributes as needed.

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core loader`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: implement YAML spec loader and SpecRegistry with embedded Duramax spec"
```

---

### Task 11: MockTransport + MockAdapter

**Files:**
- Create: `crates/obd2-core/src/transport/mock.rs`
- Create: `crates/obd2-core/src/adapter/mock.rs`

**Step 1: Write tests for mock adapter behavior**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_adapter_initialize() {
        let mut adapter = MockAdapter::new();
        let info = adapter.initialize().await.unwrap();
        assert!(matches!(info.chipset, Chipset::Elm327Genuine));
    }

    #[tokio::test]
    async fn test_mock_adapter_read_pid() {
        let mut adapter = MockAdapter::new();
        adapter.initialize().await.unwrap();
        let pids = adapter.supported_pids().await.unwrap();
        assert!(pids.contains(&Pid::ENGINE_RPM));
    }

    #[tokio::test]
    async fn test_mock_adapter_request_mode_01() {
        let mut adapter = MockAdapter::new();
        adapter.initialize().await.unwrap();
        let req = ServiceRequest::read_pid(Pid::ENGINE_RPM);
        let response = adapter.request(&req).await.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_mock_adapter_vin() {
        let mut adapter = MockAdapter::new();
        adapter.initialize().await.unwrap();
        let req = ServiceRequest::read_vin();
        let response = adapter.request(&req).await.unwrap();
        let vin = String::from_utf8_lossy(&response);
        assert_eq!(vin.len(), 17);
    }
}
```

**Step 2: Run tests — verify they fail**

**Step 3: Implement MockTransport and MockAdapter**

- `MockTransport` — in-memory request/response pairs, configurable
- `MockAdapter` — simulates a vehicle with realistic data:
  - Supported PIDs: all standard PIDs
  - VIN: configurable (defaults to Duramax VIN)
  - PID responses: generates realistic values (RPM 680-3000, temp 85-100, etc.)
  - DTCs: configurable list of active DTCs
  - Enhanced PIDs: returns mock data for any DID requested

Port mock vehicle profiles from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/mock.rs` as reference.

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core mock`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: add MockTransport and MockAdapter for testing"
```

---

### Task 12: ELM327 Adapter

**Files:**
- Create: `crates/obd2-core/src/adapter/elm327.rs`
- Create: `crates/obd2-core/src/adapter/stn.rs`

**Step 1: Write tests for ELM327 AT command protocol**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;

    #[tokio::test]
    async fn test_elm327_init_sequence() {
        let mut transport = MockTransport::new();
        transport.expect("ATZ", "ELM327 v2.1\r\r>");
        transport.expect("ATE0", "OK\r>");
        transport.expect("ATL0", "OK\r>");
        transport.expect("ATH0", "OK\r>");
        transport.expect("ATSP0", "OK\r>");
        transport.expect("0100", "41 00 BE 3E B8 11\r>");

        let mut elm = Elm327Adapter::new(Box::new(transport));
        let info = elm.initialize().await.unwrap();
        assert!(matches!(info.chipset, Chipset::Elm327Genuine));
    }

    #[tokio::test]
    async fn test_elm327_read_pid() {
        let mut transport = MockTransport::new();
        // Setup init sequence...
        setup_init(&mut transport);
        transport.expect("010C", "41 0C 0C 00\r>"); // RPM = 768

        let mut elm = Elm327Adapter::new(Box::new(transport));
        elm.initialize().await.unwrap();

        let req = ServiceRequest::read_pid(Pid::ENGINE_RPM);
        let response = elm.request(&req).await.unwrap();
        assert_eq!(response, vec![0x0C, 0x00]);
    }

    #[tokio::test]
    async fn test_elm327_header_switching() {
        let mut transport = MockTransport::new();
        setup_init(&mut transport);
        transport.expect("AT SH 6C 10 F1", "OK\r>");
        transport.expect("22 16 2F", "62 16 2F 80 00\r>");

        let mut elm = Elm327Adapter::new(Box::new(transport));
        elm.initialize().await.unwrap();

        let req = ServiceRequest::enhanced_read(0x162F, Target::Module(ModuleId::new("ecm")));
        // This should trigger header switching
        let response = elm.request(&req).await.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_elm327_no_data_response() {
        let mut transport = MockTransport::new();
        setup_init(&mut transport);
        transport.expect("015C", "NO DATA\r>");

        let mut elm = Elm327Adapter::new(Box::new(transport));
        elm.initialize().await.unwrap();

        let req = ServiceRequest::read_pid(Pid::ENGINE_OIL_TEMP);
        let result = elm.request(&req).await;
        assert!(matches!(result, Err(Obd2Error::NoData)));
    }
}
```

**Step 2: Run tests — verify they fail**

**Step 3: Implement ELM327 adapter**

Port from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/elm327.rs` as reference but redesign for new traits:
1. `Elm327Adapter` struct implementing `Adapter` trait
2. AT command init sequence (ATZ, ATE0, ATL0, ATH0, ATSP)
3. PID bitmap parsing (Mode 01 PID 00/20/40/60)
4. Hex response parsing (strip headers, parse bytes)
5. Header switching (AT SH) based on Target
6. Error handling (NO DATA, ?, UNABLE TO CONNECT, etc.)
7. STN detection and STN-specific extensions in `stn.rs`

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core elm327`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: implement ELM327 adapter with AT command protocol"
```

---

## Phase 4: Session Core (depends on Phase 3)

### Task 13: Session Core

**Files:**
- Create: `crates/obd2-core/src/session/mod.rs`

**Step 1: Write tests for basic session operations**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::mock::MockAdapter;

    #[tokio::test]
    async fn test_session_read_pid() {
        let adapter = MockAdapter::new();
        let mut session = Session::new(adapter);
        let reading = session.read_pid(Pid::ENGINE_RPM).await.unwrap();
        assert!(matches!(reading.value, Value::Scalar(_)));
        assert_eq!(reading.source, ReadingSource::Live);
    }

    #[tokio::test]
    async fn test_session_identify_vehicle() {
        let adapter = MockAdapter::with_vin("1GCHK23124F000001");
        let mut session = Session::new(adapter);
        let profile = session.identify_vehicle().await.unwrap();
        assert_eq!(profile.vin, "1GCHK23124F000001");
        // Should match the embedded Duramax spec
        assert!(profile.spec.is_some());
    }

    #[tokio::test]
    async fn test_session_read_dtcs() {
        let mut adapter = MockAdapter::new();
        adapter.set_dtcs(vec![
            Dtc { code: "P0420".into(), category: DtcCategory::Powertrain, status: DtcStatus::Stored, ..Default::default() },
        ]);
        let mut session = Session::new(adapter);
        let dtcs = session.read_dtcs().await.unwrap();
        assert_eq!(dtcs.len(), 1);
        assert_eq!(dtcs[0].code, "P0420");
    }

    #[tokio::test]
    async fn test_session_supported_pids() {
        let adapter = MockAdapter::new();
        let mut session = Session::new(adapter);
        let pids = session.supported_pids().await.unwrap();
        assert!(pids.contains(&Pid::ENGINE_RPM));
        assert!(pids.contains(&Pid::VEHICLE_SPEED));
    }

    #[tokio::test]
    async fn test_session_no_spec_still_works() {
        let adapter = MockAdapter::with_vin("JH4KA7660PC000001"); // no spec for this
        let mut session = Session::new(adapter);
        // Standard PIDs still work
        let reading = session.read_pid(Pid::ENGINE_RPM).await.unwrap();
        assert!(matches!(reading.value, Value::Scalar(_)));
        // Enhanced PIDs are empty
        assert!(session.module_pids(ModuleId::new("ecm")).is_empty());
    }
}
```

**Step 2: Run tests — verify they fail**

**Step 3: Implement Session core**

Implement the Session struct and these methods:
- `Session::new(adapter)` — creates session with default SpecRegistry
- `Session::builder(adapter)` — returns SessionBuilder for configuration
- `read_pid()` — delegates to adapter, parses response, returns Reading
- `read_pids()` — reads multiple PIDs in sequence
- `supported_pids()` — queries PID bitmaps, caches result
- `read_vin()` — Mode 09 InfoType 02
- `read_vehicle_info()` — VIN + CALIDs + CVNs + ECU name
- `identify_vehicle()` — read VIN, decode offline, match spec, optionally NHTSA lookup
- `read_dtcs()` — Mode 03 broadcast
- `read_dtcs_from(module)` — Mode 03 to specific module
- `module_pids(module)` — returns enhanced PIDs from matched spec
- `vehicle()`, `spec()`, `adapter_info()` — accessors

**Step 4: Run tests — verify they pass**

Run: `cargo test -p obd2-core session`
Expected: all tests pass

**Step 5: Commit**

```bash
git commit -am "feat: implement Session core (read_pid, identify_vehicle, read_dtcs)"
```

---

### Task 14: Threshold Evaluation (in Session)

**Files:**
- Modify: `crates/obd2-core/src/session/mod.rs`

**Step 1: Write tests for threshold evaluation in session context**

```rust
#[tokio::test]
async fn test_session_evaluate_threshold() {
    let adapter = MockAdapter::with_vin("1GCHK23124F000001");
    let mut session = Session::new(adapter);
    session.identify_vehicle().await.unwrap();

    // Duramax spec has coolant temp threshold: warning_high = 105, critical_high = 115
    let normal = session.evaluate_threshold(Pid::COOLANT_TEMP, 90.0);
    assert!(normal.is_none());

    let warning = session.evaluate_threshold(Pid::COOLANT_TEMP, 110.0);
    assert!(warning.is_some());
    assert_eq!(warning.unwrap().level, AlertLevel::Warning);
}
```

**Step 2-5: Implement, test, commit**

```bash
git commit -am "feat: add threshold evaluation to Session"
```

---

### Task 15: NHTSA Online Lookup

**Files:**
- Create: `crates/obd2-core/src/vehicle/nhtsa.rs`

**Step 1: Write tests (mocked HTTP)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nhtsa_response() {
        let json = include_str!("../../test_data/nhtsa_response.json");
        let vehicle = parse_nhtsa_response(json).unwrap();
        assert!(vehicle.is_some());
    }

    #[test]
    fn test_nhtsa_vehicle_to_profile() {
        let nhtsa = NhtsaVehicle {
            make: Some("Chevrolet".into()),
            model: Some("Silverado 2500 HD".into()),
            year: Some(2004),
            // ...
        };
        assert_eq!(nhtsa.make.as_deref(), Some("Chevrolet"));
    }
}
```

**Step 2-5: Implement, test, commit**

Port from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/nhtsa.rs`. Gate behind `#[cfg(feature = "nhtsa")]`. Add test_data/ directory with a sample NHTSA JSON response.

```bash
git commit -am "feat: add NHTSA VIN decoder (behind nhtsa feature flag)"
```

---

## Phase 5: Enhanced Session (depends on Phase 4)

### Task 16: Enhanced PID Reads + Multi-Module + Bus Switching

**Files:**
- Modify: `crates/obd2-core/src/session/mod.rs`

Implement: `read_enhanced()`, `read_all_enhanced()`, `switch_bus()`, `buses()`. Auto-switch bus when querying a module on a different bus.

```bash
git commit -am "feat: add enhanced PID reads with multi-module and bus switching"
```

### Task 17: Remaining Diagnostic Modes

**Files:**
- Modify: `crates/obd2-core/src/session/mod.rs`

Implement: `read_pending_dtcs()` (Mode 07), `read_permanent_dtcs()` (Mode 0A), `read_freeze_frame()` (Mode 02), `read_test_results()` (Mode 06), `read_readiness()`, `read_vehicle_info()`, `clear_dtcs()`, `clear_dtcs_on()`, `read_all_dtcs()`, `read_dtcs_extended()`, `read_dtc_detail()`.

```bash
git commit -am "feat: implement all diagnostic modes (02/06/07/0A/19/04/14)"
```

### Task 18: Diagnostic Intelligence

**Files:**
- Modify: `crates/obd2-core/src/session/mod.rs`

Implement: `active_rules()`, `matching_issues()`, `run_quick_test()`. DTC enrichment in `read_all_dtcs()` — look up each DTC in the spec's DtcLibrary, attach description/severity/notes.

```bash
git commit -am "feat: add diagnostic intelligence (rules, known issues, DTC enrichment)"
```

---

## Phase 6: Polling & Advanced Features (depends on Phase 5)

### Task 19: Polling Loop

**Files:**
- Create: `crates/obd2-core/src/session/poller.rs`
- Modify: `crates/obd2-core/src/session/mod.rs`

Implement: `start_polling()`, `start_polling_group()`, `stop_polling()`, `set_poll_interval()`. PollEvent channel. Threshold evaluation on every reading. BR-6 rules (cancellable, single-PID-failure doesn't stop loop, no unbounded tasks).

```bash
git commit -am "feat: implement polling loop with PollEvent channel and threshold alerts"
```

### Task 20: Session Management (Mode 10/27/2F/3E)

**Files:**
- Modify: `crates/obd2-core/src/session/mod.rs`

Implement: `enter_diagnostic_session()`, `security_access()`, `end_diagnostic_session()`, `actuator_control()`, `actuator_release()`. Auto-start tester-present keep-alive. BR-7 safety rules (3-step sequence, refuse unverified PIDs for actuators).

```bash
git commit -am "feat: implement diagnostic session management and actuator control"
```

### Task 21: Serial Transport

**Files:**
- Create: `crates/obd2-core/src/transport/serial.rs`

Implement: `SerialTransport` behind `#[cfg(feature = "serial")]`. Uses tokio-serial. Port from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/serial_transport.rs`.

```bash
git commit -am "feat: add serial transport (behind serial feature flag)"
```

### Task 22: BLE Transport

**Files:**
- Create: `crates/obd2-core/src/transport/ble.rs`

Implement: `BleTransport` behind `#[cfg(feature = "ble")]`. Uses btleplug. Port from `/Users/jared/Projects/obd2-dash/crates/obd2-core/src/obd2/ble_transport.rs`.

```bash
git commit -am "feat: add BLE transport (behind ble feature flag)"
```

---

## Phase 7: Integration & Polish (sequential)

### Task 23: obd2-store-sqlite

**Files:**
- Modify: `crates/obd2-store-sqlite/Cargo.toml`
- Create: `crates/obd2-store-sqlite/src/lib.rs`

Implement `VehicleStore` and `SessionStore` traits using rusqlite. Port schema from `/Users/jared/Projects/obd2-dash/crates/obd2-db/src/lib.rs`.

```bash
git commit -am "feat: implement obd2-store-sqlite crate"
```

### Task 24: Integration Tests

**Files:**
- Create: `crates/obd2-core/tests/integration_test.rs`

End-to-end tests using MockAdapter:
1. Full session lifecycle: init > identify > poll > stop
2. Enhanced PID reads with Duramax spec
3. DTC enrichment with spec descriptions
4. Threshold alerting during polling
5. Diagnostic rule firing (P0700 → query TCM)
6. Known issue matching
7. Multiple specs loaded, correct one matched by VIN

```bash
git commit -am "test: add integration tests for full session lifecycle"
```

### Task 25: Documentation + README + Publish Prep

**Files:**
- Create: `~/Projects/obd2-core/README.md`
- Modify: all `lib.rs` and `mod.rs` files for crate/module-level docs
- Create: `~/Projects/obd2-core/CHANGELOG.md`
- Create: `~/Projects/obd2-core/LICENSE-MIT` and `LICENSE-APACHE`

Run: `cargo doc --no-deps --open` — verify all public items have docs.
Run: `cargo clippy -- -D warnings` — fix all warnings.
Run: `cargo test` — all tests pass.

```bash
git commit -am "docs: add crate documentation, README, and publish prep"
```

---

## Parallelization Summary

| Phase | Tasks | Can Parallel | Depends On |
|-------|-------|-------------|------------|
| 1 | 1, 2, 3, 4 | All 4 parallel | Nothing |
| 2 | 5, 6, 7, 8, 9 | All 5 parallel | Phase 1 |
| 3 | 10, 11, 12 | All 3 parallel | Phase 2 |
| 4 | 13, 14, 15 | All 3 parallel | Phase 3 |
| 5 | 16, 17, 18 | All 3 parallel | Phase 4 |
| 6 | 19, 20, 21, 22 | All 4 parallel | Phase 5 |
| 7 | 23, 24, 25 | Sequential | Phase 6 |

**Total: 25 tasks across 7 phases. Maximum parallelism: 5 concurrent tasks (Phase 2).**
