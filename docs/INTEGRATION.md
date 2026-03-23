# obd2-core Integration Manual

This guide covers everything you need to integrate obd2-core into a diagnostic tool, fleet management system, or vehicle monitoring application.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Core Concepts](#core-concepts)
3. [Session Lifecycle](#session-lifecycle)
4. [Reading PIDs](#reading-pids)
5. [Diagnostic Trouble Codes](#diagnostic-trouble-codes)
6. [Vehicle Identification](#vehicle-identification)
7. [Enhanced PIDs (Manufacturer-Specific)](#enhanced-pids-manufacturer-specific)
8. [Diagnostic Sessions & Security](#diagnostic-sessions--security)
9. [Polling Engine](#polling-engine)
10. [Vehicle Spec System](#vehicle-spec-system)
11. [Diagnostic Intelligence](#diagnostic-intelligence)
12. [Storage Integration](#storage-integration)
13. [Custom Transports](#custom-transports)
14. [Custom Adapters](#custom-adapters)
15. [Error Handling](#error-handling)
16. [Feature Flags & Build Configurations](#feature-flags--build-configurations)
17. [Platform Notes](#platform-notes)

---

## Getting Started

### Dependencies

```toml
[dependencies]
obd2-core = { git = "https://github.com/trepidity/obd2-core" }
tokio = { version = "1", features = ["rt", "macros"] }

# Optional: SQLite storage
obd2-store-sqlite = { git = "https://github.com/trepidity/obd2-core" }
```

### Minimal Example

```rust
use obd2_core::adapter::elm327::Elm327Adapter;
use obd2_core::transport::serial::SerialTransport;
use obd2_core::session::Session;
use obd2_core::protocol::pid::Pid;

#[tokio::main]
async fn main() -> Result<(), obd2_core::error::Obd2Error> {
    // 1. Open physical connection
    let transport = SerialTransport::new("/dev/ttyUSB0", 115200).await?;

    // 2. Create adapter (protocol interpreter)
    let mut adapter = Elm327Adapter::new(transport);
    let info = adapter.initialize().await?;
    println!("Adapter: {:?} ({})", info.chipset, info.firmware);

    // 3. Create session (high-level API)
    let mut session = Session::new(adapter);

    // 4. Identify vehicle
    let profile = session.identify_vehicle().await?;
    println!("VIN: {}", profile.vin);
    if let Some(spec) = &profile.spec {
        println!("Matched: {} {}", spec.identity.name, spec.identity.engine.code);
    }

    // 5. Read data
    let rpm = session.read_pid(Pid::ENGINE_RPM).await?;
    println!("Engine RPM: {:?} {}", rpm.value, rpm.unit);

    Ok(())
}
```

### BLE Example

```rust
use obd2_core::transport::ble::BleTransport;
use obd2_core::adapter::elm327::Elm327Adapter;

// Scan for OBD-II adapter
let transport = BleTransport::connect_first().await?;
let mut adapter = Elm327Adapter::new(transport);
adapter.initialize().await?;
let mut session = Session::new(adapter);
```

---

## Core Concepts

### Layer Architecture

```
┌─────────────────────────────────────────────┐
│              Your Application                │
├─────────────────────────────────────────────┤
│   Session (orchestrator, high-level API)     │
│   - identify_vehicle()                       │
│   - read_pid(), read_dtcs()                  │
│   - polling, diagnostics, thresholds         │
├────────────────────┬────────────────────────┤
│   Adapter          │   SpecRegistry          │
│   (ELM327, mock)   │   (YAML vehicle specs)  │
│   Protocol logic   │   VIN matching          │
├────────────────────┘                         │
│   Transport                                  │
│   (serial, BLE, mock)                        │
│   Raw byte I/O                               │
└─────────────────────────────────────────────┘
```

**Transport** handles raw bytes on a physical medium. It knows nothing about OBD-II — just read/write/reset.

**Adapter** interprets OBD-II requests into adapter-specific commands. The ELM327 adapter translates `ServiceRequest` into AT commands and hex strings. A J2534 adapter would use a completely different protocol.

**Session** is the consumer-facing API. It combines an adapter with the spec registry, provides PID/DTC methods, runs the polling engine, and evaluates thresholds.

### Key Types

| Type | Module | Purpose |
|------|--------|---------|
| `Session<A>` | `session` | Primary entry point, generic over adapter |
| `Pid` | `protocol::pid` | Newtype `Pid(u8)` with named constants and parse logic |
| `Reading` | `protocol::enhanced` | Decoded value + unit + timestamp + source |
| `Value` | `protocol::enhanced` | `Scalar(f64)`, `Bitfield`, `State(String)`, `Raw(Vec<u8>)` |
| `Dtc` | `protocol::dtc` | Trouble code + category + status + description |
| `VehicleProfile` | `vehicle` | VIN + info + matched spec + supported PIDs |
| `VehicleSpec` | `vehicle` | Full vehicle specification from YAML |
| `Obd2Error` | `error` | Error enum covering all failure modes |
| `ServiceRequest` | `protocol::service` | Low-level service ID + data + target |
| `ModuleId` | `vehicle` | Logical module identifier (`"ecm"`, `"tcm"`, etc.) |

---

## Session Lifecycle

### 1. Create Session

```rust
use obd2_core::adapter::mock::MockAdapter;
use obd2_core::session::Session;

let adapter = MockAdapter::new(); // or Elm327Adapter, your custom adapter
let mut session = Session::new(adapter);
```

The session loads embedded specs automatically (if `embedded-specs` feature is enabled).

### 2. Load Additional Specs (Optional)

```rust
// Single file
session.load_spec(Path::new("specs/ford_powerstroke_2020.yaml"))?;

// Entire directory
let count = session.load_spec_dir(Path::new("specs/"))?;
println!("Loaded {count} spec files");
```

### 3. Identify Vehicle

```rust
let profile = session.identify_vehicle().await?;
```

This:
1. Reads the VIN via Mode 09
2. Decodes VIN offline (manufacturer, year, engine class)
3. Queries supported PIDs (Mode 01 PID 00/20/40/60 bitmaps)
4. Matches the best vehicle spec from the registry
5. Caches the profile for subsequent operations

After identification, `session.vehicle()` and `session.spec()` return the matched data.

### 4. Read Data

Standard PIDs, DTCs, enhanced PIDs — all methods are on `Session`.

### 5. Teardown

Session is dropped normally. No explicit disconnect is needed — the transport handles cleanup.

---

## Reading PIDs

### Standard PIDs (Mode 01)

```rust
use obd2_core::protocol::pid::Pid;

// Single PID
let reading = session.read_pid(Pid::ENGINE_RPM).await?;
let rpm: f64 = reading.value.as_f64()?;  // 680.0
let unit: &str = reading.unit;            // "RPM"

// Multiple PIDs
let readings = session.read_pids(&[
    Pid::ENGINE_RPM,
    Pid::COOLANT_TEMP,
    Pid::VEHICLE_SPEED,
    Pid::THROTTLE_POSITION,
]).await?;

for (pid, reading) in &readings {
    println!("{}: {:?} {}", pid.name(), reading.value, reading.unit);
}
```

### Available PID Constants

```rust
// Engine
Pid::ENGINE_RPM           // 0x0C — RPM
Pid::ENGINE_LOAD          // 0x04 — %
Pid::COOLANT_TEMP         // 0x05 — °C
Pid::INTAKE_AIR_TEMP      // 0x0F — °C
Pid::ENGINE_OIL_TEMP      // 0x5C — °C
Pid::MAF                  // 0x10 — g/s
Pid::TIMING_ADVANCE       // 0x0E — ° before TDC

// Vehicle
Pid::VEHICLE_SPEED        // 0x0D — km/h
Pid::RUN_TIME             // 0x1F — seconds

// Throttle & pedal
Pid::THROTTLE_POSITION    // 0x11 — %
Pid::ACCEL_PEDAL_POS_D    // 0x49 — %
Pid::ACCEL_PEDAL_POS_E    // 0x4A — %

// Fuel
Pid::FUEL_PRESSURE        // 0x0A — kPa
Pid::FUEL_TANK_LEVEL      // 0x2F — %
Pid::ENGINE_FUEL_RATE     // 0x5E — L/h
Pid::FUEL_RAIL_GAUGE_PRESSURE  // 0x23 — kPa
Pid::FUEL_RAIL_ABS_PRESSURE   // 0x59 — kPa
Pid::SHORT_FUEL_TRIM_B1   // 0x06 — %
Pid::LONG_FUEL_TRIM_B1    // 0x07 — %

// Intake & exhaust
Pid::INTAKE_MAP           // 0x0B — kPa
Pid::BAROMETRIC_PRESSURE  // 0x33 — kPa
Pid::COMMANDED_EGR        // 0x2C — %
Pid::EGR_ERROR            // 0x2D — %

// Catalysts
Pid::CATALYST_TEMP_B1S1   // 0x3C — °C
Pid::CATALYST_TEMP_B1S2   // 0x3E — °C

// System
Pid::CONTROL_MODULE_VOLTAGE // 0x42 — V
Pid::AMBIENT_AIR_TEMP     // 0x46 — °C
Pid::OBD_STANDARD         // 0x1C — enum
Pid::MONITOR_STATUS       // 0x01 — bitfield

// Distance
Pid::DISTANCE_WITH_MIL    // 0x21 — km
Pid::DISTANCE_SINCE_CLEAR // 0x31 — km
```

### Working with Values

```rust
match &reading.value {
    Value::Scalar(v) => println!("Value: {v}"),
    Value::Bitfield(bf) => {
        for (name, set) in &bf.flags {
            println!("  {name}: {set}");
        }
    }
    Value::State(s) => println!("State: {s}"),
    Value::Raw(bytes) => println!("Raw: {bytes:02X?}"),
}

// Convenience: get f64 from scalar values
let temp: f64 = reading.value.as_f64()?;
```

### Checking PID Support

```rust
let supported = session.supported_pids().await?;
if supported.contains(&Pid::ENGINE_OIL_TEMP) {
    let oil = session.read_pid(Pid::ENGINE_OIL_TEMP).await?;
}
```

---

## Diagnostic Trouble Codes

### Reading DTCs

```rust
// Mode 03: Stored/confirmed DTCs
let stored = session.read_dtcs().await?;

// Mode 07: Pending DTCs (test failed but not yet confirmed)
let pending = session.read_pending_dtcs().await?;

// Mode 0A: Permanent DTCs (cannot be cleared by scan tool)
let permanent = session.read_permanent_dtcs().await?;
```

### DTC Structure

```rust
pub struct Dtc {
    pub code: String,                // "P0420"
    pub category: DtcCategory,       // Powertrain, Chassis, Body, Network
    pub status: DtcStatus,           // Stored, Pending, Permanent
    pub description: Option<String>, // Human-readable description
    pub severity: Option<Severity>,  // Critical, High, Medium, Low, Info
    pub source_module: Option<String>,
    pub notes: Option<String>,
}
```

### DTC Categories

The first character of the code determines the category:

| Prefix | Category | Examples |
|--------|----------|---------|
| P | Powertrain (engine, transmission) | P0420, P0171 |
| C | Chassis (ABS, traction) | C0045, C0265 |
| B | Body (airbag, HVAC, lighting) | B0100, B1000 |
| U | Network (CAN bus communication) | U0100, U0401 |

### Clearing DTCs

```rust
// WARNING: This resets readiness monitors. The vehicle will need to complete
// drive cycles before all monitors are ready again.
session.clear_dtcs().await?;
```

### DTC Enrichment

When a vehicle spec is matched, DTCs are enriched with manufacturer-specific descriptions. Without a spec, the library falls back to ~200 universal SAE J2012 descriptions.

---

## Vehicle Identification

### Automatic Identification

```rust
let profile = session.identify_vehicle().await?;

println!("VIN: {}", profile.vin);
println!("Supported PIDs: {}", profile.supported_pids.len());

if let Some(spec) = &profile.spec {
    println!("Vehicle: {}", spec.identity.name);
    println!("Engine: {} {} {}L {}",
        spec.identity.engine.code,
        spec.identity.engine.fuel_type,
        spec.identity.engine.displacement_l,
        spec.identity.engine.cylinders,
    );
}
```

### Offline VIN Decoder

```rust
use obd2_core::vehicle::vin;

let decoded = vin::decode("1GCHK23224F000001");
println!("Manufacturer: {:?}", decoded.manufacturer);  // Some("General Motors")
println!("Year: {:?}", decoded.year);                   // Some(2004)
println!("Class: {:?}", decoded.truck_class);           // Some(DieselTruck)
```

The offline decoder identifies:
- **Manufacturer** from WMI (first 3 VIN characters) — 50+ manufacturers
- **Model year** from 10th character (30-year cycle)
- **Truck/vehicle class** from manufacturer patterns

### NHTSA Online Lookup (Optional)

```toml
obd2-core = { git = "...", features = ["nhtsa"] }
```

```rust
use obd2_core::vehicle::nhtsa;

let info = nhtsa::decode_vin("1GCHK23224F000001").await?;
println!("Make: {}, Model: {}, Year: {}", info.make, info.model, info.year);
```

---

## Enhanced PIDs (Manufacturer-Specific)

Enhanced PIDs (Mode 22) provide manufacturer-specific data not available through standard OBD-II. They require a matched vehicle spec.

### Reading Enhanced PIDs

```rust
use obd2_core::vehicle::ModuleId;

// Read turbo boost pressure from ECM
let reading = session.read_enhanced(0x0124, ModuleId::new("ecm")).await?;

// List available enhanced PIDs for a module
let ecm_pids = session.module_pids(ModuleId::new("ecm"));
for pid in ecm_pids {
    println!("DID {:#06x}: {} ({})", pid.did, pid.name, pid.unit);
}
```

### Common Module IDs

| ID | Module | Description |
|----|--------|-------------|
| `ecm` | Engine Control Module | Engine parameters, fuel, boost |
| `tcm` | Transmission Control Module | Gear, shift data, fluid temp |
| `bcm` | Body Control Module | Lights, doors, windows |
| `abs` | Anti-lock Brake System | Wheel speeds, brake pressure |
| `ipc` | Instrument Panel Cluster | Odometer, warnings |
| `airbag` | Airbag Module | Crash sensors, deployment status |
| `hvac` | HVAC Module | Climate control |
| `ficm` | Fuel Injection Control Module | Injector data (diesel) |

### Enhanced PID Formula Types

Enhanced PIDs include decoding formulas in the spec:

```yaml
enhanced_pids:
  - did: 0x0124
    name: "Turbo Boost Pressure"
    unit: "psi"
    formula: { type: linear, scale: 0.14504, offset: -14.696 }
    bytes: 2
    module: ecm
    confidence: verified
```

Formula types: `linear`, `two_byte`, `centered`, `bitmask`, `enumerated`, `expression`.

---

## Diagnostic Sessions & Security

For advanced operations (actuator control, calibration), vehicles require elevated diagnostic sessions with security access.

### Session Flow

```rust
use obd2_core::session::diag_session::*;

// 1. Enter extended diagnostic session
let state = enter_session(&mut adapter, DiagSession::Extended, "ecm").await?;

// 2. Authenticate with seed/key exchange
let key_fn: KeyFunction = Box::new(|seed: &[u8]| {
    // Your manufacturer-specific key algorithm
    compute_key(seed)
});
security_access(&mut adapter, "ecm", &key_fn).await?;

// 3. Perform actuator control
actuator_control(
    &mut adapter,
    0x0124,         // DID
    "ecm",          // Module
    &ActuatorCommand::Activate,
    &state,
).await?;

// 4. Return control to ECU
actuator_control(
    &mut adapter,
    0x0124,
    "ecm",
    &ActuatorCommand::ReturnToEcu,
    &state,
).await?;
```

### Actuator Commands

| Command | Description |
|---------|-------------|
| `ReturnToEcu` | Release control back to the ECU |
| `Activate` | Activate the actuator |
| `Adjust(Vec<u8>)` | Set a specific value |

### Keep-Alive

Extended sessions time out after ~5 seconds of inactivity. Use tester-present to keep alive:

```rust
// Mode 3E — send periodically in a background task
let req = ServiceRequest { service_id: 0x3E, data: vec![0x00], target: Target::Module("ecm".into()) };
adapter.request(&req).await?;
```

---

## Polling Engine

### Configuration

```rust
use obd2_core::session::poller::{PollConfig, PollEvent, PollHandle};
use std::time::Duration;

let config = PollConfig {
    pids: vec![
        Pid::ENGINE_RPM,
        Pid::COOLANT_TEMP,
        Pid::VEHICLE_SPEED,
        Pid::ENGINE_LOAD,
        Pid::THROTTLE_POSITION,
    ],
    interval: Duration::from_millis(250),
    read_voltage: true,
};
```

### Running the Poll Loop

```rust
let (handle, mut rx, _config) = poller::start_poll_loop(config);

tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        match event {
            PollEvent::Reading { pid, reading } => {
                update_gauge(pid, &reading);
            }
            PollEvent::EnhancedReading { did, module, reading } => {
                update_enhanced_display(did, &module, &reading);
            }
            PollEvent::Alert(result) => {
                show_warning(result.message, result.level);
            }
            PollEvent::RuleFired { rule_name, description } => {
                log_diagnostic_event(&rule_name, &description);
            }
            PollEvent::Voltage(v) => {
                update_battery_indicator(v);
            }
            PollEvent::Error { pid, error } => {
                // Non-fatal — polling continues
                log::warn!("Poll error for {pid:?}: {error}");
            }
        }
    }
});
```

### Controlling the Poll Loop

```rust
// Dynamic interval adjustment (e.g., slow down when on battery)
handle.set_interval(Duration::from_secs(2));

// Check status
if handle.is_running() { /* ... */ }

// Stop polling
handle.stop();
```

### Design Notes

- Single PID failures don't stop the loop — they emit `PollEvent::Error` and continue
- Threshold evaluation runs automatically if a spec is matched
- Battery voltage is read each cycle if `read_voltage` is true
- The event channel is bounded — slow consumers will cause backpressure

---

## Vehicle Spec System

### YAML Spec Schema

```yaml
spec_version: "1.0"

identity:
  name: "Chevrolet Silverado 2500HD Duramax LLY"
  model_years: [2004, 2006]
  makes: ["Chevrolet", "GMC"]
  models: ["Silverado 2500HD", "Sierra 2500HD"]
  engine:
    code: "LLY"
    fuel_type: "diesel"
    displacement_l: 6.6
    cylinders: 8
    aspiration: turbo
  transmission:
    type: automatic
    model: "Allison 1000"
    speeds: 5
  vin_match:
    wmi_prefixes: ["1GC", "1GT", "2GC", "3GC"]
    vin_8th_digit: ["2"]          # LLY engine code
    year_range: [2004, 2006]

communication:
  primary_bus:
    id: class2
    protocol: j1850_vpw
    speed_bps: 10400
    modules:
      - id: ecm
        name: "Engine Control Module"
        address: { type: j1850, node: 0x10, header: [0x68, 0x6A, 0x10] }
        bus: class2
      - id: tcm
        name: "Transmission Control Module"
        address: { type: j1850, node: 0x18, header: [0x68, 0x6A, 0x18] }
        bus: class2
  init_sequence:
    - "ATSH 68 6A F1"
    - "AT ST 96"

thresholds:
  coolant_temp:
    pid: 0x05
    min: -20.0
    max: 130.0
    warning_high: 105.0
    critical_high: 115.0
    unit: "°C"
  engine_rpm:
    pid: 0x0C
    max: 6000.0
    warning_high: 5500.0
    critical_high: 5800.0
    unit: "RPM"
  boost_pressure:
    pid: 0x0B
    max: 30.0
    warning_high: 25.0
    critical_high: 28.0
    unit: "psi"

polling_groups:
  - name: engine_vitals
    pids: [0x0C, 0x05, 0x04, 0x0B, 0x11]
    interval_ms: 250
  - name: transmission
    pids: [0x0D]
    interval_ms: 500

diagnostic_rules:
  - name: "TCM cascade"
    trigger: { type: dtc_present, code: "P0700" }
    action: { type: query_module, module: tcm, service: 0x03 }
    description: "P0700 means TCM has DTCs — query TCM directly"

known_issues:
  - rank: 1
    name: "Turbo Vane Sticking"
    description: "VGT vanes stick from carbon buildup"
    symptoms: ["P0234", "P0299"]
    root_cause: "Carbon deposits on VGT vane mechanism"
    quick_test:
      description: "Monitor VGT position vs. commanded"
      pass_criteria: "Position follows command within 5%"
    fix: "Remove and clean turbo; replace if vanes are damaged"

  - rank: 2
    name: "FICM Failure"
    description: "Fuel Injection Control Module capacitor degradation"
    symptoms: ["P0201", "P0611", "P2146"]
    root_cause: "Electrolytic capacitor failure on FICM board"
    quick_test:
      description: "Read FICM supply voltage"
      pass_criteria: "48V ± 2V"
    fix: "Replace or rebuild FICM (capacitor rework kit available)"

dtc_library:
  codes:
    P0087: "Fuel Rail Pressure Too Low"
    P0093: "Fuel System Large Leak Detected"
    P0234: "Turbo Overboost Condition"
    P0299: "Turbo Underboost Condition"
    P0380: "Glow Plug Circuit A Malfunction"
    P0700: "Transmission Control System Malfunction"

enhanced_pids:
  - did: 0x0124
    name: "Turbo Boost Pressure"
    unit: "psi"
    formula: { type: linear, scale: 0.14504, offset: -14.696 }
    bytes: 2
    module: ecm
    value_type: scalar
    confidence: verified

  - did: 0x0110
    name: "Fuel Rail Pressure"
    unit: "MPa"
    formula: { type: two_byte, scale: 0.1, offset: 0.0 }
    bytes: 2
    module: ecm
    value_type: scalar
    confidence: community
```

### SpecRegistry

```rust
use obd2_core::vehicle::SpecRegistry;

let mut registry = SpecRegistry::with_defaults(); // loads embedded specs

// Add runtime specs
registry.load_file(Path::new("my_spec.yaml"))?;
registry.load_directory(Path::new("specs/"))?;

// Match by VIN (preferred)
if let Some(spec) = registry.match_vin("1GCHK23224F000001") {
    println!("Matched: {}", spec.identity.name);
}

// Match by make/model/year (fallback)
if let Some(spec) = registry.match_vehicle("Chevrolet", "Silverado 2500HD", 2005) {
    println!("Matched: {}", spec.identity.name);
}
```

### VIN Matching Rules

Specs match VINs based on:

1. **WMI prefix** — first 3 characters identify the manufacturer + plant
2. **8th digit** — engine code (e.g., `"2"` = LLY Duramax)
3. **Year range** — model year from 10th character

All conditions in `vin_match` must pass. If `vin_match` is absent, the spec can still match via make/model/year.

---

## Diagnostic Intelligence

### DTC Enrichment

DTC descriptions are resolved in priority order:

1. **Spec DTC library** — manufacturer-specific (most accurate)
2. **Universal SAE J2012** — ~200 standard codes built into the library
3. **Fallback** — "Unknown DTC"

```rust
let dtcs = session.read_dtcs().await?;
for dtc in &dtcs {
    // description is already enriched
    println!("{}: {} ({:?})", dtc.code, dtc.description.as_deref().unwrap_or("Unknown"), dtc.severity);
}
```

### Diagnostic Rules

Rules trigger actions when specific DTCs are detected:

```rust
// From the spec:
// P0700 present → query TCM directly for real codes
// This happens automatically during DTC enrichment

pub enum RuleTrigger {
    DtcPresent(String),                // Single code
    DtcRange(String, String),          // Code range (e.g., P0201..P0208)
}

pub enum RuleAction {
    QueryModule { module: String, service: u8 },
    CheckFirst { pid: u16, module: String, reason: String },
    Alert(String),
    MonitorPids(Vec<u16>),
}
```

### Known Issues

Known issues are ranked by frequency (lower rank = more common). The library matches DTCs against known issue symptoms:

```rust
if let Some(spec) = session.spec() {
    for issue in &spec.known_issues {
        println!("#{}: {} — {}", issue.rank, issue.name, issue.description);
        println!("  Symptoms: {:?}", issue.symptoms);
        println!("  Root cause: {}", issue.root_cause);
        if let Some(test) = &issue.quick_test {
            println!("  Quick test: {}", test.description);
            println!("  Pass criteria: {}", test.pass_criteria);
        }
        println!("  Fix: {}", issue.fix);
    }
}
```

### Threshold Evaluation

```rust
use obd2_core::vehicle::ThresholdResult;
use obd2_core::session::threshold;

// Thresholds are evaluated automatically in the polling engine.
// You can also evaluate manually:

if let Some(spec) = session.spec() {
    if let Some(ts) = &spec.thresholds {
        let result = threshold::evaluate(Pid::COOLANT_TEMP, 110.0, ts);
        match result.level {
            AlertLevel::Normal => { /* OK */ },
            AlertLevel::Warning => { /* warn user */ },
            AlertLevel::Critical => { /* alert immediately */ },
        }
    }
}
```

Threshold structure:

| Field | Description |
|-------|-------------|
| `min` / `max` | Absolute valid range |
| `warning_low` / `warning_high` | Warning thresholds |
| `critical_low` / `critical_high` | Critical thresholds |

---

## Storage Integration

### Trait-Based Design

obd2-core defines storage traits. Implementations live in separate crates or your own code:

```rust
// In obd2-core (trait only)
#[async_trait]
pub trait VehicleStore: Send + Sync {
    async fn save_vehicle(&self, profile: &VehicleProfile) -> Result<(), Obd2Error>;
    async fn get_vehicle(&self, vin: &str) -> Result<Option<VehicleProfile>, Obd2Error>;
    async fn save_thresholds(&self, vin: &str, ts: &ThresholdSet) -> Result<(), Obd2Error>;
    async fn get_thresholds(&self, vin: &str) -> Result<Option<ThresholdSet>, Obd2Error>;
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn save_reading(&self, vin: &str, pid: Pid, reading: &Reading) -> Result<(), Obd2Error>;
    async fn save_dtc_event(&self, vin: &str, dtcs: &[Dtc]) -> Result<(), Obd2Error>;
}
```

### SQLite Backend

```rust
use obd2_store_sqlite::SqliteStore;

// File-based
let store = SqliteStore::open(Path::new("fleet.db"))?;

// In-memory (testing)
let store = SqliteStore::in_memory()?;

// Use as VehicleStore
store.save_vehicle(&profile).await?;
let loaded = store.get_vehicle("1GCHK23224F000001").await?;

// Use as SessionStore
store.save_reading("1GCHK23224F000001", Pid::ENGINE_RPM, &reading).await?;
store.save_dtc_event("1GCHK23224F000001", &dtcs).await?;
```

### SQLite Schema

```sql
vehicles    (vin TEXT PK, data TEXT, updated_at TEXT)
thresholds  (vin TEXT PK, data TEXT, updated_at TEXT)
readings    (id INTEGER PK, vin TEXT, pid_code INTEGER, value REAL, unit TEXT, timestamp TEXT)
dtc_events  (id INTEGER PK, vin TEXT, dtc_codes TEXT, timestamp TEXT)

-- Indices on vin for both readings and dtc_events
```

### Custom Storage Backend

```rust
use obd2_core::store::{VehicleStore, SessionStore};

struct PostgresStore { pool: PgPool }

#[async_trait]
impl VehicleStore for PostgresStore {
    async fn save_vehicle(&self, profile: &VehicleProfile) -> Result<(), Obd2Error> {
        // Your Postgres implementation
    }
    // ...
}
```

---

## Custom Transports

Implement the `Transport` trait for any physical medium:

```rust
use obd2_core::transport::Transport;
use obd2_core::error::Obd2Error;
use async_trait::async_trait;

struct WifiTransport {
    socket: TcpStream,
}

#[async_trait]
impl Transport for WifiTransport {
    async fn write(&mut self, data: &[u8]) -> Result<(), Obd2Error> {
        self.socket.write_all(data).await
            .map_err(|e| Obd2Error::Transport(e.to_string()))
    }

    async fn read(&mut self) -> Result<Vec<u8>, Obd2Error> {
        let mut buf = vec![0u8; 1024];
        let n = self.socket.read(&mut buf).await
            .map_err(|e| Obd2Error::Transport(e.to_string()))?;
        buf.truncate(n);
        Ok(buf)
    }

    async fn reset(&mut self) -> Result<(), Obd2Error> {
        // Reconnect or send reset command
        Ok(())
    }

    fn name(&self) -> &str { "wifi" }
}
```

Transports must be `Send + Sync`.

---

## Custom Adapters

For non-ELM327 hardware (J2534, raw CAN, etc.):

```rust
use obd2_core::adapter::{Adapter, AdapterInfo, Chipset, Capabilities};
use obd2_core::protocol::service::ServiceRequest;
use obd2_core::protocol::pid::Pid;
use obd2_core::error::Obd2Error;
use async_trait::async_trait;

struct J2534Adapter { /* ... */ }

#[async_trait]
impl Adapter for J2534Adapter {
    async fn initialize(&mut self) -> Result<AdapterInfo, Obd2Error> {
        // Open J2534 device, detect protocol
        Ok(AdapterInfo {
            chipset: Chipset::Unknown,
            firmware: "J2534 v2".into(),
            protocol: Protocol::Can11Bit500,
            capabilities: Capabilities {
                can_clear_dtcs: true,
                dual_can: true,
                enhanced_diag: true,
                battery_voltage: false,
                adaptive_timing: false,
            },
        })
    }

    async fn request(&mut self, req: &ServiceRequest) -> Result<Vec<u8>, Obd2Error> {
        // Frame the request as a CAN message, send, wait for response
        // Return data bytes only (no service ID echo)
        todo!()
    }

    async fn supported_pids(&mut self) -> Result<HashSet<Pid>, Obd2Error> {
        // Query PID 00, 20, 40, 60 bitmaps
        todo!()
    }

    async fn battery_voltage(&mut self) -> Result<Option<f64>, Obd2Error> {
        Ok(None) // Not supported on J2534
    }

    fn info(&self) -> &AdapterInfo { &self.info }
}
```

The adapter contract:
- `request()` returns **data bytes only** — strip the service ID echo and any padding
- `supported_pids()` returns the union of all PID support bitmaps
- Adapters must be `Send`

---

## Error Handling

### Error Variants

| Variant | When | Recovery |
|---------|------|----------|
| `Timeout` | Vehicle didn't respond within timeout | Retry, check connection |
| `NoData` | Vehicle responded with "NO DATA" | PID not available on this vehicle |
| `UnsupportedPid { pid }` | PID not in supported set | Check `supported_pids()` first |
| `NegativeResponse { service, nrc }` | ECU rejected the request | Check NRC code for reason |
| `Transport(String)` | Physical connection failure | Reconnect transport |
| `Adapter(String)` | Protocol-level error | Re-initialize adapter |
| `AdapterBusy` | Adapter in use (e.g., polling) | Stop polling first |
| `SecurityRequired` | Operation needs security access | Run security_access() flow |
| `NoSpec` | Operation requires vehicle spec | Call identify_vehicle() or load spec |
| `ModuleNotFound(String)` | Module not in spec | Check spec module list |
| `SpecParse(String)` | YAML spec is malformed | Fix the YAML file |
| `ParseError(String)` | Response data couldn't be decoded | Check response format |
| `Io(io::Error)` | File system or I/O error | Check paths and permissions |

### Negative Response Codes (NRC)

| Code | NRC | Meaning |
|------|-----|---------|
| 0x10 | GeneralReject | ECU rejected for unspecified reason |
| 0x11 | ServiceNotSupported | Mode not implemented by ECU |
| 0x12 | SubFunctionNotSupported | Sub-function not available |
| 0x13 | IncorrectMessageLength | Request was wrong size |
| 0x22 | ConditionsNotCorrect | Preconditions not met (e.g., engine off) |
| 0x31 | RequestOutOfRange | DID/PID not available |
| 0x33 | SecurityAccessDenied | Not authenticated |
| 0x35 | InvalidKey | Wrong security key |
| 0x36 | ExceededAttempts | Too many failed security attempts |
| 0x37 | TimeDelayNotExpired | Must wait before retrying security |
| 0x78 | ResponsePending | ECU is processing (wait and retry) |

### Pattern: Graceful Degradation

```rust
// Read what you can, skip what you can't
let pids_to_try = vec![
    Pid::ENGINE_RPM, Pid::COOLANT_TEMP, Pid::VEHICLE_SPEED,
    Pid::ENGINE_OIL_TEMP, Pid::BOOST_PRESSURE,
];

for pid in pids_to_try {
    match session.read_pid(pid).await {
        Ok(reading) => dashboard.update(pid, reading),
        Err(Obd2Error::NoData | Obd2Error::UnsupportedPid { .. }) => {
            dashboard.mark_unavailable(pid);
        }
        Err(Obd2Error::Timeout) => {
            log::warn!("Timeout reading {}, will retry", pid.name());
        }
        Err(e) => return Err(e), // Fatal
    }
}
```

---

## Feature Flags & Build Configurations

| Flag | Default | Adds |
|------|---------|------|
| `serial` | Yes | `tokio-serial` — USB/RS-232 transport |
| `embedded-specs` | Yes | Compiled-in vehicle specs |
| `ble` | No | `btleplug` + `futures` + `uuid` — BLE transport |
| `nhtsa` | No | `reqwest` — online VIN lookup |
| `full` | No | All of the above |

### Common Configurations

```toml
# Desktop diagnostic tool (USB adapter)
obd2-core = { git = "...", features = ["serial", "embedded-specs"] }

# Mobile app (BLE adapter)
obd2-core = { git = "...", default-features = false, features = ["ble", "embedded-specs"] }

# Server/fleet (no hardware, just spec parsing and storage)
obd2-core = { git = "...", default-features = false, features = ["embedded-specs"] }

# Everything
obd2-core = { git = "...", features = ["full"] }
```

---

## Platform Notes

### macOS
- Serial: works with `/dev/tty.usbserial-*` or `/dev/cu.usbserial-*`
- BLE: requires Bluetooth permission in entitlements

### Linux
- Serial: needs read/write access to `/dev/ttyUSB0` (add user to `dialout` group)
- BLE: requires `bluez` and appropriate D-Bus permissions

### Windows
- Serial: use COM port names (`COM3`, `COM4`, etc.)
- BLE: uses WinRT Bluetooth APIs via btleplug

### iOS / Android
- Use BLE transport only (no serial)
- BLE feature flag must be enabled
- Platform BLE permissions must be configured in the app manifest

### Minimum Rust Version

Rust 1.75+ (edition 2021).
