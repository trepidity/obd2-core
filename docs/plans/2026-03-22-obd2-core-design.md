# obd2-core Design Document

**Date:** 2026-03-22
**Status:** Approved
**Location:** ~/Projects/obd2-core (new standalone repository)

## Overview

obd2-core is a cross-platform Rust library for OBD-II vehicle diagnostics. It is the shared foundation for obd2-dash (TUI diagnostic dashboard) and HaulLogic (commercial fleet management). Both consumers become pure UI/UX shells — all protocol logic, vehicle specs, diagnostic intelligence, and adapter communication lives in obd2-core.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Relationship to obd2-dash | Fresh start, informed by existing | Clean API, no legacy coupling |
| Transport adapters | Open trait, anyone can implement | OBD2 adapters are diverse (ELM327, STN, J2534, raw CAN) |
| Vehicle specs | Embedded defaults + runtime YAML override | Works out-of-box, extensible without recompile |
| Async runtime | Tokio-committed | Both consumers use tokio; serial/BLE deps require it anyway |
| Database/storage | Separate optional crate (obd2-store-sqlite) | Core stays lean; consumers choose their own storage backend |

## 1. Crate Architecture & File Structure

```
~/Projects/obd2-core/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── obd2-core/                # The library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API re-exports, crate-level docs
│   │       ├── error.rs          # Obd2Error (thiserror, #[non_exhaustive])
│   │       ├── protocol/
│   │       │   ├── mod.rs        # OBD-II protocol logic
│   │       │   ├── pid.rs        # Standard PID definitions, parsing (Mode 01)
│   │       │   ├── dtc.rs        # DTC decoding (Mode 03/07/0A)
│   │       │   ├── enhanced.rs   # Enhanced PID support (Mode 21/22)
│   │       │   ├── service.rs    # Diagnostic service definitions
│   │       │   └── codec.rs      # Raw byte parsing (J1850, CAN frame decoding)
│   │       ├── transport/
│   │       │   ├── mod.rs        # Transport trait (open, public)
│   │       │   ├── serial.rs     # #[cfg(feature = "serial")] ELM327/STN over serial
│   │       │   ├── ble.rs        # #[cfg(feature = "ble")] BLE GATT transport
│   │       │   └── mock.rs       # MockTransport for testing (always available)
│   │       ├── adapter/
│   │       │   ├── mod.rs        # Adapter trait — protocol interpreters
│   │       │   ├── elm327.rs     # ELM327 AT command protocol
│   │       │   ├── stn.rs        # STN-specific extensions
│   │       │   └── detect.rs     # Chipset detection, capability probing
│   │       ├── vehicle/
│   │       │   ├── mod.rs        # VehicleSpec, VehicleProfile types
│   │       │   ├── spec.rs       # Spec data structures (mirrors YAML schema)
│   │       │   ├── loader.rs     # YAML loading + embedded defaults
│   │       │   ├── vin.rs        # VIN decoding (offline)
│   │       │   └── nhtsa.rs      # #[cfg(feature = "nhtsa")] online VIN lookup
│   │       ├── session/
│   │       │   ├── mod.rs        # Session — the high-level orchestrator
│   │       │   ├── poller.rs     # PID polling loop with configurable groups
│   │       │   └── scanner.rs    # Device discovery (serial + BLE)
│   │       ├── store/
│   │       │   └── mod.rs        # Storage traits (VehicleStore, SessionStore)
│   │       └── specs/
│   │           └── embedded/     # Compiled-in YAML specs
│   │               ├── mod.rs
│   │               └── chevy_duramax_2004_turbo.yaml
│   │
│   └── obd2-store-sqlite/        # Optional SQLite storage implementation
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
│
├── vehicle-specs/                 # Reference YAML specs (development)
│   ├── chevy_duramax_2004_turbo.yaml
│   └── spec_sources.yaml
│
└── docs/
    └── plans/
```

**Key separation:**
- **transport/** = physical connection (bytes over wire)
- **adapter/** = protocol interpreter (ELM327 AT commands, STN extensions)
- **protocol/** = OBD-II protocol logic (PIDs, DTCs, services — pure data, no I/O)
- **vehicle/** = vehicle-specific knowledge (specs, VIN, enhanced PIDs)
- **session/** = high-level orchestrator that ties it all together
- **store/** = trait definitions only (implementations in separate crates)

## 2. Core Types

### 2A. Values & Readings

```rust
/// A decoded value from an OBD-II response. Not everything is a float.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Value {
    Scalar(f64),
    Bitfield(Bitfield),
    State(String),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Bitfield {
    pub raw: u32,
    pub flags: Vec<(String, bool)>,
}

#[derive(Debug, Clone)]
pub struct Reading {
    pub value: Value,
    pub unit: &'static str,
    pub timestamp: Instant,
    pub raw_bytes: Vec<u8>,
    pub source: ReadingSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingSource { Live, FreezeFrame, Replay }
```

### 2B. PIDs

```rust
/// Standard OBD-II PID (Mode 01/02). Newtype over u8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pid(pub u8);

impl Pid {
    pub const MONITOR_STATUS: Pid = Pid(0x01);
    pub const ENGINE_RPM: Pid = Pid(0x0C);
    pub const VEHICLE_SPEED: Pid = Pid(0x0D);
    pub const COOLANT_TEMP: Pid = Pid(0x05);
    pub const ENGINE_OIL_TEMP: Pid = Pid(0x5C);
    // ... all standard PIDs as named constants

    pub fn name(&self) -> &'static str;
    pub fn unit(&self) -> &'static str;
    pub fn response_bytes(&self) -> u8;
    pub fn value_type(&self) -> ValueType;
    pub fn parse(&self, data: &[u8]) -> Result<Value, Obd2Error>;
}

/// Enhanced manufacturer-specific PID. Defined by vehicle specs.
#[derive(Debug, Clone)]
pub struct EnhancedPid {
    pub service_id: u8,                    // 0x21 (Honda/Toyota) or 0x22 (GM/Ford/modern)
    pub did: u16,                          // 2-byte Data Identifier
    pub name: String,
    pub unit: String,
    pub formula: Formula,
    pub bytes: u8,
    pub module: ModuleId,
    pub value_type: ValueType,
    pub confidence: Confidence,
    pub range: Option<OperatingRange>,
    pub command_suffix: Option<Vec<u8>>,   // e.g., [0x01] data rate byte for Duramax
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Formula {
    Linear { scale: f64, offset: f64 },
    TwoByte { scale: f64, offset: f64 },
    Centered { center: f64, divisor: f64 },
    Bitmask { bits: Vec<(u8, String)> },
    Enumerated { values: Vec<(u8, String)> },
    Expression(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType { Scalar, Bitfield, State }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence { Verified, Community, Inferred, Unverified }
```

### 2C. DTCs

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dtc {
    pub code: String,
    pub category: DtcCategory,
    pub status: DtcStatus,
    pub description: Option<String>,
    pub severity: Option<Severity>,
    pub source_module: Option<ModuleId>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DtcCategory { Powertrain, Chassis, Body, Network }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtcStatus { Stored, Pending, Permanent }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity { Critical, High, Medium, Low, Info }

/// GM/UDS Mode 19 extended status byte.
#[derive(Debug, Clone, Copy, Default)]
pub struct DtcStatusByte {
    pub test_failed: bool,
    pub test_failed_this_cycle: bool,
    pub pending: bool,
    pub confirmed: bool,
    pub test_not_completed_since_clear: bool,
    pub test_failed_since_clear: bool,
    pub test_not_completed_this_cycle: bool,
    pub warning_indicator_requested: bool,
}
```

### 2D. Modules & Addressing (protocol-agnostic)

```rust
/// Logical module identifier. String newtype — extensible across manufacturers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId(pub String);

impl ModuleId {
    pub const ECM: &'static str = "ecm";
    pub const TCM: &'static str = "tcm";
    pub const BCM: &'static str = "bcm";
    pub const ABS: &'static str = "abs";
    pub const IPC: &'static str = "ipc";
    pub const AIRBAG: &'static str = "airbag";
    pub const HVAC: &'static str = "hvac";
    pub const FICM: &'static str = "ficm";
    // Manufacturer specs define their own: "vsa", "eps", "rear_diff", etc.
}

/// Protocol-specific physical address. Lives in the vehicle spec.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PhysicalAddress {
    J1850 { node: u8, header: [u8; 3] },
    Can11Bit { request_id: u16, response_id: u16 },
    Can29Bit { request_id: u32, response_id: u32 },
    J1939 { source_address: u8 },
}

/// A module on the vehicle bus, fully resolved from the spec.
#[derive(Debug, Clone)]
pub struct Module {
    pub id: ModuleId,
    pub name: String,
    pub address: PhysicalAddress,
    pub bus: BusId,
    pub enhanced_pids: Vec<EnhancedPid>,
}

/// Request targeting. Consumers use ModuleId; adapter resolves physical address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Broadcast,
    Module(ModuleId),
}
```

### 2E. Protocol & Multi-Bus

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Protocol {
    J1850Vpw,
    J1850Pwm,
    Iso9141(KLineInit),
    Kwp2000(KLineInit),
    Can11Bit500,
    Can11Bit250,
    Can29Bit500,
    Can29Bit250,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KLineInit { SlowInit, FastInit }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BusId(pub String);

#[derive(Debug, Clone)]
pub struct BusConfig {
    pub id: BusId,
    pub protocol: Protocol,
    pub speed_bps: u32,
    pub modules: Vec<Module>,
    pub description: Option<String>,
}
```

### 2F. Readiness, Test Results, Vehicle Info

```rust
#[derive(Debug, Clone)]
pub struct ReadinessStatus {
    pub mil_on: bool,
    pub dtc_count: u8,
    pub compression_ignition: bool,
    pub monitors: Vec<MonitorStatus>,
}

#[derive(Debug, Clone)]
pub struct MonitorStatus {
    pub name: String,
    pub supported: bool,
    pub complete: bool,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: u8,
    pub name: String,
    pub value: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub passed: bool,
    pub unit: String,
}

#[derive(Debug, Clone)]
pub struct VehicleInfo {
    pub vin: String,
    pub calibration_ids: Vec<String>,
    pub cvns: Vec<u32>,
    pub ecu_name: Option<String>,
}
```

### 2G. Negative Response Codes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NegativeResponse {
    GeneralReject,
    ServiceNotSupported,
    SubFunctionNotSupported,
    IncorrectMessageLength,
    ResponseTooLong,
    ConditionsNotCorrect,
    RequestOutOfRange,
    SecurityAccessDenied,
    InvalidKey,
    ExceededAttempts,
    TimeDelayNotExpired,
    GeneralProgrammingFailure,
    ResponsePending,
}
```

## 3. Transport & Adapter Traits

```rust
/// Physical connection to an OBD-II adapter. Open trait — anyone can implement.
#[async_trait]
pub trait Transport: Send + Sync {
    async fn write(&mut self, data: &[u8]) -> Result<(), Obd2Error>;
    async fn read(&mut self) -> Result<Vec<u8>, Obd2Error>;
    async fn reset(&mut self) -> Result<(), Obd2Error>;
    fn name(&self) -> &str;
}

/// Protocol interpreter. Translates OBD-II requests into adapter-specific commands.
#[async_trait]
pub trait Adapter: Send {
    async fn initialize(&mut self) -> Result<AdapterInfo, Obd2Error>;
    async fn request(&mut self, req: &ServiceRequest) -> Result<Vec<u8>, Obd2Error>;
    async fn supported_pids(&mut self) -> Result<HashSet<Pid>, Obd2Error>;
    async fn battery_voltage(&mut self) -> Result<Option<f64>, Obd2Error>;
    fn info(&self) -> &AdapterInfo;
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AdapterInfo {
    pub chipset: Chipset,
    pub firmware: String,
    pub protocol: Protocol,
    pub capabilities: Capabilities,
}
```

## 4. Vehicle Spec Model

```rust
/// Complete diagnostic specification for a vehicle platform.
pub struct VehicleSpec {
    pub spec_version: String,                // "1.0"
    pub identity: SpecIdentity,
    pub communication: CommunicationSpec,
    pub modules: Vec<Module>,
    pub standard_pids: Vec<StandardPidSpec>,
    pub dtc_library: DtcLibrary,
    pub thresholds: ThresholdSet,
    pub polling_groups: Vec<PollingGroup>,
    pub diagnostic_rules: Vec<DiagnosticRule>,
    pub known_issues: Vec<KnownIssue>,
    pub readiness: Option<ReadinessSpec>,
    pub init_sequence: Option<InitSequence>,
}
```

### Spec Identity & VIN Matching

```rust
pub struct SpecIdentity {
    pub name: String,
    pub model_years: (u16, u16),
    pub makes: Vec<String>,
    pub models: Vec<String>,
    pub engine: EngineSpec,
    pub transmission: Option<TransmissionSpec>,
    pub vin_match: Option<VinMatcher>,
}

pub struct VinMatcher {
    pub vin_8th_digit: Option<Vec<char>>,
    pub wmi_prefixes: Vec<String>,
    pub year_range: Option<(u16, u16)>,
}

pub struct TransmissionSpec {
    pub model: String,
    pub transmission_type: TransmissionType,
    pub fluid_capacity_l: Option<f64>,
}

#[non_exhaustive]
pub enum TransmissionType {
    Geared { speeds: u8, gear_ratios: Vec<(String, f64)> },
    Cvt { ratio_range: (f64, f64), simulated_steps: Option<u8> },
    Dct { speeds: u8, gear_ratios: Vec<(String, f64)> },
    Manual { speeds: u8, gear_ratios: Vec<(String, f64)> },
}
```

### Diagnostic Intelligence

```rust
/// Diagnostic rule — encodes "if X then Y" logic from vehicle spec.
pub struct DiagnosticRule {
    pub name: String,
    pub trigger: RuleTrigger,
    pub action: RuleAction,
    pub description: String,
}

#[non_exhaustive]
pub enum RuleTrigger {
    DtcPresent(String),
    DtcRange(String, String),
    ReadingOutOfRange { pid: u16, threshold: Threshold },
}

#[non_exhaustive]
pub enum RuleAction {
    QueryModule { module: ModuleId, service: u8 },
    CheckFirst { pid: u16, module: ModuleId, reason: String },
    Alert(String),
    MonitorPids(Vec<u16>),
}

/// Known platform failure mode.
pub struct KnownIssue {
    pub rank: u8,
    pub name: String,
    pub description: String,
    pub symptoms: Vec<String>,
    pub root_cause: String,
    pub quick_test: Option<QuickTest>,
    pub fix: String,
}
```

### Thresholds

```rust
pub struct Threshold {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub warning_low: Option<f64>,
    pub warning_high: Option<f64>,
    pub critical_low: Option<f64>,
    pub critical_high: Option<f64>,
    pub unit: String,
}

pub struct ThresholdResult {
    pub level: AlertLevel,
    pub reading: f64,
    pub limit: f64,
    pub direction: AlertDirection,
    pub message: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel { Normal, Warning, Critical }
```

### Spec Registry

```rust
pub struct SpecRegistry { /* ... */ }

impl SpecRegistry {
    pub fn with_defaults() -> Self;
    pub fn load_file(&mut self, path: &Path) -> Result<(), Obd2Error>;
    pub fn load_directory(&mut self, dir: &Path) -> Result<usize, Obd2Error>;
    pub fn match_vin(&self, vin: &str) -> Option<&VehicleSpec>;
    pub fn match_vehicle(&self, make: &str, model: &str, year: u16) -> Option<&VehicleSpec>;
    pub fn specs(&self) -> &[VehicleSpec];
    pub fn lookup_dtc(&self, code: &str) -> Option<&DtcEntry>;
}
```

## 5. Session API

The primary entry point for all consumers.

```rust
pub struct Session<A: Adapter> { /* ... */ }

impl<A: Adapter> Session<A> {
    // Construction
    pub fn new(adapter: A) -> Self;
    pub fn builder(adapter: A) -> SessionBuilder<A>;

    // Spec management
    pub fn load_spec(&mut self, path: &Path) -> Result<(), Obd2Error>;
    pub fn load_spec_dir(&mut self, dir: &Path) -> Result<usize, Obd2Error>;
    pub fn specs(&self) -> &SpecRegistry;

    // Mode 01 — Current data
    pub async fn read_pid(&mut self, pid: Pid) -> Result<Reading, Obd2Error>;
    pub async fn read_pids(&mut self, pids: &[Pid]) -> Result<Vec<(Pid, Reading)>, Obd2Error>;
    pub async fn supported_pids(&mut self) -> Result<HashSet<Pid>, Obd2Error>;
    pub async fn read_readiness(&mut self) -> Result<ReadinessStatus, Obd2Error>;

    // Mode 02 — Freeze frame
    pub async fn read_freeze_frame(&mut self, pid: Pid, frame: u8) -> Result<Reading, Obd2Error>;

    // Mode 03/07/0A — DTCs
    pub async fn read_dtcs(&mut self) -> Result<Vec<Dtc>, Obd2Error>;
    pub async fn read_dtcs_from(&mut self, module: ModuleId) -> Result<Vec<Dtc>, Obd2Error>;
    pub async fn read_pending_dtcs(&mut self) -> Result<Vec<Dtc>, Obd2Error>;
    pub async fn read_permanent_dtcs(&mut self) -> Result<Vec<Dtc>, Obd2Error>;
    pub async fn read_all_dtcs(&mut self) -> Result<Vec<Dtc>, Obd2Error>;

    // Mode 04/14 — Clear DTCs
    pub async fn clear_dtcs(&mut self) -> Result<(), Obd2Error>;
    pub async fn clear_dtcs_on(&mut self, module: ModuleId) -> Result<(), Obd2Error>;

    // Mode 06 — Monitoring test results
    pub async fn read_test_results(&mut self, test_id: u8) -> Result<Vec<TestResult>, Obd2Error>;
    pub async fn supported_test_ids(&mut self) -> Result<Vec<u8>, Obd2Error>;

    // Mode 09 — Vehicle information
    pub async fn read_vin(&mut self) -> Result<String, Obd2Error>;
    pub async fn read_vehicle_info(&mut self) -> Result<VehicleInfo, Obd2Error>;
    pub async fn identify_vehicle(&mut self) -> Result<VehicleProfile, Obd2Error>;

    // Mode 21/22 — Enhanced PIDs
    pub async fn read_enhanced(&mut self, did: u16, module: ModuleId) -> Result<Reading, Obd2Error>;
    pub async fn read_all_enhanced(&mut self, module: ModuleId) -> Result<Vec<(EnhancedPid, Reading)>, Obd2Error>;
    pub fn module_pids(&self, module: ModuleId) -> &[EnhancedPid];

    // Mode 19 — Extended DTCs (GM/UDS)
    pub async fn read_dtcs_extended(&mut self, module: ModuleId) -> Result<Vec<(Dtc, DtcStatusByte)>, Obd2Error>;
    pub async fn read_dtc_detail(&mut self, code: &str, module: ModuleId) -> Result<DtcDetail, Obd2Error>;

    // Session management (Mode 10/27/3E)
    pub async fn enter_diagnostic_session(&mut self, session: DiagSession) -> Result<(), Obd2Error>;
    pub async fn security_access(&mut self, module: ModuleId, key_fn: KeyFunction) -> Result<(), Obd2Error>;
    pub async fn end_diagnostic_session(&mut self) -> Result<(), Obd2Error>;

    // Mode 2F — Actuator tests
    pub async fn actuator_control(&mut self, did: u16, module: ModuleId, command: ActuatorCommand) -> Result<(), Obd2Error>;
    pub async fn actuator_release(&mut self, did: u16, module: ModuleId) -> Result<(), Obd2Error>;

    // Raw service access
    pub async fn raw_request(&mut self, service: u8, data: &[u8], target: Target) -> Result<Vec<u8>, Obd2Error>;

    // Multi-bus
    pub async fn switch_bus(&mut self, bus: &BusId) -> Result<(), Obd2Error>;
    pub fn buses(&self) -> &[BusConfig];

    // Polling
    pub fn start_polling(&mut self, pids: &[Pid], interval: Duration) -> mpsc::Receiver<PollEvent>;
    pub fn start_polling_group(&mut self, group_name: &str) -> Result<mpsc::Receiver<PollEvent>, Obd2Error>;
    pub fn stop_polling(&mut self);
    pub fn set_poll_interval(&mut self, interval: Duration);

    // Thresholds & alerts
    pub fn evaluate_threshold(&self, pid: Pid, value: f64) -> Option<ThresholdResult>;
    pub fn evaluate_enhanced_threshold(&self, did: u16, value: f64) -> Option<ThresholdResult>;

    // Diagnostic intelligence
    pub fn active_rules(&self) -> Vec<&DiagnosticRule>;
    pub fn matching_issues(&self) -> Vec<&KnownIssue>;
    pub async fn run_quick_test(&mut self, issue: &KnownIssue) -> Result<QuickTestResult, Obd2Error>;

    // State
    pub fn vehicle(&self) -> Option<&VehicleProfile>;
    pub fn spec(&self) -> Option<&VehicleSpec>;
    pub fn adapter_info(&self) -> &AdapterInfo;
    pub async fn battery_voltage(&mut self) -> Result<Option<f64>, Obd2Error>;
}
```

### Poll Events

```rust
#[non_exhaustive]
pub enum PollEvent {
    Reading { pid: Pid, reading: Reading },
    EnhancedReading { did: u16, module: ModuleId, reading: Reading },
    Alert(ThresholdResult),
    RuleFired(DiagnosticRule),
    Error { pid: Option<Pid>, error: Obd2Error },
    Voltage(f64),
}
```

### Store Traits

```rust
#[async_trait]
pub trait VehicleStore: Send + Sync {
    async fn save_vehicle(&self, profile: &VehicleProfile) -> Result<(), Obd2Error>;
    async fn get_vehicle(&self, vin: &str) -> Result<Option<VehicleProfile>, Obd2Error>;
    async fn save_thresholds(&self, vin: &str, thresholds: &ThresholdSet) -> Result<(), Obd2Error>;
    async fn get_thresholds(&self, vin: &str) -> Result<Option<ThresholdSet>, Obd2Error>;
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn save_reading(&self, vin: &str, pid: Pid, reading: &Reading) -> Result<(), Obd2Error>;
    async fn save_dtc_event(&self, vin: &str, dtcs: &[Dtc]) -> Result<(), Obd2Error>;
}
```

## 6. Error Type

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Obd2Error {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("adapter error: {0}")]
    Adapter(String),
    #[error("adapter is busy (stop polling first)")]
    AdapterBusy,
    #[error("timeout waiting for response")]
    Timeout,
    #[error("no data (vehicle did not respond)")]
    NoData,
    #[error("PID {pid:#04x} not supported by this vehicle")]
    UnsupportedPid { pid: u8 },
    #[error("module {0} not found in vehicle spec")]
    ModuleNotFound(ModuleId),
    #[error("negative response: {nrc:?} for service {service:#04x}")]
    NegativeResponse { service: u8, nrc: NegativeResponse },
    #[error("security access required")]
    SecurityRequired,
    #[error("no vehicle spec matched")]
    NoSpec,
    #[error("bus {0:?} not available on this vehicle")]
    BusNotAvailable(BusId),
    #[error("spec parse error: {0}")]
    SpecParse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
```

## 7. Feature Flags

```toml
[features]
default = ["serial", "embedded-specs"]
serial = ["dep:tokio-serial"]
ble = ["dep:btleplug"]
embedded-specs = []
nhtsa = ["dep:reqwest"]
serde = ["dep:serde", "serde/derive"]
full = ["serial", "ble", "embedded-specs", "nhtsa", "serde"]
```

### Consumer configurations

- **obd2-dash:** `features = ["serial", "ble", "nhtsa", "embedded-specs"]` + obd2-store-sqlite
- **HaulLogic:** `features = ["ble", "nhtsa", "embedded-specs", "serde"]` + custom storage
- **Embedded/mobile:** `default-features = false, features = ["ble"]`

## 8. Business Rules

### BR-1: Vehicle Spec Lifecycle

- BR-1.1: A vehicle spec YAML file is the single source of truth for all manufacturer-specific data.
- BR-1.2: Standard OBD-II data (Mode 01 PIDs, Mode 03 DTC decoding, Mode 09 VIN) is hardcoded. SAE J1979/J2012 are universal and stable.
- BR-1.3: Specs match vehicles by VIN 8th digit, WMI prefix, and year range. Most specific match wins.
- BR-1.4: Embedded specs ship compiled in. Runtime YAML specs override embedded specs for the same vehicle.
- BR-1.5: The library MUST function without any matched spec. Standard OBD-II always works.
- BR-1.6: Specs MUST NOT redefine standard PID decoding formulas. They add thresholds and ranges only.

### BR-2: Spec Data Confidence

- BR-2.1: Every enhanced PID, DTC entry, and threshold MUST have a confidence tag.
- BR-2.2: Every spec section MUST reference source IDs from spec_sources.yaml.
- BR-2.3: Confidence is propagated to consumers on every reading.
- BR-2.4: Actuator control (Mode 2F) MUST refuse PIDs marked "unverified".

### BR-3: Connection & Communication

- BR-3.1: Communication goes exclusively through the Adapter trait.
- BR-3.2: Header switching is the adapter's responsibility.
- BR-3.3: Bus switching is explicit; Session auto-switches based on module location.
- BR-3.4: NRC 0x78 (Response Pending) is handled transparently with 3 retries.
- BR-3.5: Inter-request timing is enforced by the adapter (P3 = 55ms for J1850).
- BR-3.6: Tester Present keep-alive is automatic during extended sessions.

### BR-4: Diagnostic Intelligence

- BR-4.1: read_all_dtcs() reads all modes, all modules, enriches with spec, fires rules.
- BR-4.2: Diagnostic rules fire automatically but are advisory only.
- BR-4.3: Known issues match by DTC codes AND out-of-range readings.
- BR-4.4: DTC description resolution: spec > universal J2012 table > "Unknown DTC".

### BR-5: Threshold Evaluation

- BR-5.1: Priority: VIN-specific > spec threshold > engine family default > none.
- BR-5.2: During polling, thresholds evaluate automatically; breaches emit PollEvent::Alert.
- BR-5.3: Evaluation is stateless — no debouncing or suppression.
- BR-5.4: Only Value::Scalar readings are threshold-evaluated.

### BR-6: Polling & Battery Conservation

- BR-6.1: Polling is cancellable; stop_polling() halts immediately.
- BR-6.2: Poll interval is adjustable at runtime (increase when backgrounded).
- BR-6.3: Polling groups follow the spec's step sequence to minimize header switches.
- BR-6.4: Single PID failure does not stop the polling loop.
- BR-6.5: No unbounded background tasks; everything is tracked and cancelled on drop.

### BR-7: Security & Safety

- BR-7.1: Actuator control requires 3-step sequence: session > security > control.
- BR-7.2: Seed/key algorithms are NOT in the library — consumer provides KeyFunction.
- BR-7.3: clear_dtcs() logs tracing::warn (consumer handles confirmation UX).
- BR-7.4: Library is read-only plus actuator tests. No reflashing, no calibration writes.

### BR-8: Spec Authoring

- BR-8.1: Specs MUST be valid YAML.
- BR-8.2: Enhanced PIDs MUST include: service_id, did, name, formula, bytes, module, confidence.
- BR-8.3: DTC entries MUST include: code, meaning, severity.
- BR-8.4: Formulas use A/B/C/D variable convention.
- BR-8.5: Sources reference spec_sources.yaml entries by ID.
- BR-8.6: Operating ranges use string values for readability; thresholds use numeric fields.
- BR-8.7: Specs SHOULD include at least one polling group.
- BR-8.8: New specs start as confidence: inferred, upgrade after on-vehicle testing.

### BR-9: Connection Lifecycle

- BR-9.1: Session is 1:1 with an adapter. One Session, one Adapter.
- BR-9.2: Init sequence: adapter.initialize() > supported_pids() > identify_vehicle().
- BR-9.3: On disconnect, all requests return Obd2Error::Transport. No auto-reconnect.
- BR-9.4: On Session drop: stop polling, stop keep-alive, release actuators, end session.
- BR-9.5: Initialization timeout is 10 seconds (configurable via SessionBuilder).

### BR-10: Error Recovery

- BR-10.1: NoData is not retried (usually means unsupported PID).
- BR-10.2: Timeout is retried once with doubled timeout.
- BR-10.3: NRC 0x78 waits up to 5 seconds, polling every 100ms.
- BR-10.4: Garbled data returns Obd2Error::Adapter with raw response, no retry.
- BR-10.5: PIDs that fail with NoData/RequestOutOfRange are marked unsupported for the session.

### BR-11: Concurrency

- BR-11.1: Session is NOT thread-safe. All methods take &mut self.
- BR-11.2: Polling has exclusive adapter access. Manual reads during polling return AdapterBusy.
- BR-11.3: mpsc channel is the only data path during polling.

### BR-12: Recording & Replay

- BR-12.1: Recording is a core capability, opt-in via SessionBuilder.
- BR-12.2: Every PollEvent is intercepted and written before delivery.
- BR-12.3: Replay implements the same Session API; consumers can't distinguish live from replay.
- BR-12.4: Recording format is versioned; library reads old versions, writes latest.

### BR-13: Unit Conversion

- BR-13.1: All values returned in SI/metric (C, kPa, km/h, L).
- BR-13.2: Display conversion is the consumer's responsibility.
- BR-13.3: Reading.unit reflects canonical metric unit, not display unit.

### BR-14: NHTSA & Online Services

- BR-14.1: NHTSA requires "nhtsa" feature flag; identify_vehicle() works without it.
- BR-14.2: NHTSA has 5-second timeout; failure is never a hard error.
- BR-14.3: NHTSA results cached per session (same VIN never looked up twice).
- BR-14.4: Persistent caching requires a connected VehicleStore.

### BR-15: Spec Format Versioning

- BR-15.1: Every spec YAML MUST include spec_version (current: "1.0").
- BR-15.2: Reject specs with unsupported major version.
- BR-15.3: Accept specs with higher minor version (ignore unknown fields).
- BR-15.4: Breaking format changes increment major version with migration guide.

### BR-16: Logging

- BR-16.1: Library uses tracing; MUST NOT configure a subscriber.
- BR-16.2: ERROR=unrecoverable, WARN=recoverable, INFO=lifecycle, DEBUG=protocol, TRACE=bytes.
- BR-16.3: VINs are PII — only logged at DEBUG level.
- BR-16.4: Raw adapter traffic logged at DEBUG.

## 9. User Stories

### Story 1: Query engine oil temp

```rust
let reading = session.read_pid(Pid::ENGINE_OIL_TEMP).await?;
let temp = reading.as_f64()?;
if let Some(result) = session.evaluate_threshold(Pid::ENGINE_OIL_TEMP, temp)? {
    match result.level {
        AlertLevel::Warning  => warn!("Oil temp warning: {}", result.message),
        AlertLevel::Critical => error!("Oil temp CRITICAL: {}", result.message),
        _ => {}
    }
}
```

### Story 2: New vehicle — full support

```rust
session.load_spec("vehicle-specs/ford_powerstroke_2020.yaml")?;
let profile = session.identify_vehicle().await?;
for module in profile.spec.modules.iter() {
    println!("{}: {} enhanced PIDs", module.name, module.enhanced_pids.len());
}
```

### Story 3: Deep diagnostics for 2020 Chevy Malibu 1.5T

```rust
let dtcs = session.read_all_dtcs().await?;
for dtc in &dtcs {
    println!("{} [{}] — {}", dtc.code, dtc.severity.unwrap(), dtc.description.unwrap());
}
for issue in session.matching_issues() {
    println!("Known issue: {} (rank #{})", issue.name, issue.rank);
    if let Some(test) = &issue.quick_test {
        println!("  Quick test: {}", test.description);
    }
}
```
