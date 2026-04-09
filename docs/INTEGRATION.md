# obd2-core Integration Guide

This guide reflects the current pre-`1.0` library design.

The integration rule is simple:

- your application talks to `Session`
- `Session` owns discovery, routing, diagnostics, polling, and lifecycle
- adapters and transports are integration details below that boundary

Do not build new application code around direct adapter request flows.

## Install

```toml
[dependencies]
obd2-core = { git = "https://github.com/trepidity/obd2-core" }
tokio = { version = "1", features = ["rt", "macros"] }
```

Optional:

```toml
obd2-core = { git = "https://github.com/trepidity/obd2-core", features = ["serial", "embedded-specs"] }
```

## Architecture

```text
Your App
  -> Session
    -> Adapter
      -> Transport
```

Responsibilities:

- `Session`
  - initialization
  - protocol discovery and fallback
  - vehicle identification
  - module routing
  - diagnostic session state
  - polling
  - visibility and discovery reporting

- `Adapter`
  - adapter-specific setup and recovery
  - physical request emission
  - adapter quirks and sanitization

- `Transport`
  - raw bytes only

## Basic Setup

### Mock

```rust
use obd2_core::adapter::mock::MockAdapter;
use obd2_core::session::Session;

let adapter = MockAdapter::new();
let mut session = Session::new(adapter);
```

### ELM327 / STN over serial

```rust,no_run
use obd2_core::adapter::elm327::Elm327Adapter;
use obd2_core::transport::serial::SerialTransport;
use obd2_core::session::Session;

# async fn example() -> Result<(), obd2_core::error::Obd2Error> {
let transport = SerialTransport::new("/dev/ttyUSB0", 115200).await?;
let adapter = Elm327Adapter::new(Box::new(transport));
let mut session = Session::new(adapter);
# Ok(())
# }
```

### ELM327 / STN over BLE

```rust,no_run
use obd2_core::adapter::elm327::Elm327Adapter;
use obd2_core::transport::ble::BleTransport;
use obd2_core::session::Session;

# async fn example() -> Result<(), obd2_core::error::Obd2Error> {
let transport = BleTransport::connect_first().await?;
let adapter = Elm327Adapter::new(Box::new(transport));
let mut session = Session::new(adapter);
# Ok(())
# }
```

## Recommended Lifecycle

### 1. Create the session

```rust
let mut session = Session::new(adapter);
```

### 2. Optionally load additional specs

```rust
use std::path::Path;

session.load_spec(Path::new("specs/custom_vehicle.yaml"))?;
session.load_spec_dir(Path::new("specs"))?;
```

### 3. Initialize or identify

If you only need adapter state and discovery:

```rust
let info = session.initialize().await?;
println!("Adapter firmware: {}", info.firmware);
println!("Protocol: {:?}", info.protocol);
```

If you want the normal full path, call `identify_vehicle()`:

```rust
let profile = session.identify_vehicle().await?;
println!("VIN: {}", profile.vin);
```

`identify_vehicle()` will ensure initialization happened first.

### 4. Inspect discovery state

```rust
println!("Connection state: {:?}", session.connection_state());

if let Some(discovery) = session.discovery() {
    println!("Selected protocol: {:?}", discovery.selected_protocol);
    println!("Protocol choice source: {:?}", discovery.protocol_choice_source);
    println!("Visible ECUs: {}", discovery.visible_ecus.len());
}
```

## Standard OBD Operations

### Read one PID

```rust
use obd2_core::protocol::pid::Pid;

let rpm = session.read_pid(Pid::ENGINE_RPM).await?;
println!("RPM: {:?} {}", rpm.value, rpm.unit);
```

### Read multiple PIDs

```rust
let readings = session.read_pids(&[
    Pid::ENGINE_RPM,
    Pid::COOLANT_TEMP,
    Pid::VEHICLE_SPEED,
]).await?;
```

### Query supported PIDs

```rust
let supported = session.supported_pids().await?;
if supported.contains(&Pid::ENGINE_RPM) {
    println!("RPM is supported");
}
```

### DTC operations

```rust
let stored = session.read_dtcs().await?;
let pending = session.read_pending_dtcs().await?;
let permanent = session.read_permanent_dtcs().await?;
let all = session.read_all_dtcs().await?;

session.clear_dtcs().await?;
session.clear_dtcs_on_module(obd2_core::vehicle::ModuleId::new("ecm")).await?;
```

### VIN

```rust
let vin = session.read_vin().await?;
```

### Freeze frame, readiness, and monitor results

```rust
use obd2_core::protocol::pid::Pid;

let freeze = session.read_freeze_frame(Pid::ENGINE_RPM, 0).await?;
let readiness = session.read_readiness().await?;
let monitor_results = session.read_test_results(0x01).await?;
```

### Full vehicle information

```rust
let info = session.read_vehicle_info().await?;
```

### Battery voltage

```rust
let voltage = session.battery_voltage().await?;
```

## Enhanced / Module-Targeted Operations

These calls must go through `Session`, because logical module names are resolved there.

```rust
use obd2_core::vehicle::ModuleId;

let pids = session.module_pids(ModuleId::new("ecm"));
let reading = session.read_enhanced(0x162F, ModuleId::new("ecm")).await?;
```

If discovery cannot resolve the module to a routed address, the call fails explicitly.

Typical failure cases:

- no discovery profile
- unknown module
- module not available on the active bus

## Diagnostic Session Operations

Diagnostic session control is session-owned.

```rust
use obd2_core::protocol::service::{ActuatorCommand, DiagSession};
use obd2_core::vehicle::ModuleId;
use obd2_core::session::KeyFunction;

let module = ModuleId::new("tcm");

session
    .enter_diagnostic_session(DiagSession::Extended, module.clone())
    .await?;

let key_fn: KeyFunction = Box::new(|seed| {
    // Replace with the real manufacturer algorithm.
    seed.to_vec()
});

session.security_access(module.clone(), &key_fn).await?;

session
    .actuator_control(0x1196, module.clone(), &ActuatorCommand::Activate)
    .await?;

session.tester_present(module.clone()).await?;
session.end_diagnostic_session(module).await?;
```

Use `session.diagnostic_state()` to inspect current session state.

## Polling

Polling now runs through `Session`, not the raw adapter.

```rust
use obd2_core::protocol::pid::Pid;
use obd2_core::session::poller::{self, PollConfig, PollEvent};
use tokio::sync::mpsc;
use std::time::Duration;

let config = PollConfig {
    pids: vec![Pid::ENGINE_RPM, Pid::COOLANT_TEMP],
    interval: Duration::from_millis(250),
    read_voltage: true,
};

let (_handle, _unused_rx, _) = poller::start_poll_loop(config.clone());
let (tx, mut rx) = mpsc::channel(256);

poller::execute_poll_cycle(&mut session, &config, &tx, None).await;

while let Some(event) = rx.recv().await {
    match event {
        PollEvent::Reading { pid, reading } => {
            println!("{pid:?}: {:?}", reading.value);
        }
        PollEvent::Alert(result) => {
            println!("Alert: {}", result.message);
        }
        PollEvent::Voltage(v) => {
            println!("Voltage: {v}");
        }
        PollEvent::Error { pid, error } => {
            println!("Poll error on {pid:?}: {error}");
        }
        _ => {}
    }
}
```

If you want a long-running polling task, keep the loop in your application and repeatedly call `execute_poll_cycle()` on your `Session`.

## Raw Capture And Diagnostics

In debug builds, raw capture is enabled by default.

Behavior:

- capture starts on `Session::initialize()`
- the temporary filename is renamed after VIN discovery
- adapter events, probe decisions, routing changes, and recovery actions are annotated into the capture

Configuration:

```rust
session.set_raw_capture_enabled(true);
session.set_raw_capture_directory("raw-captures");
```

Current path:

```rust
if let Some(path) = session.raw_capture_path() {
    println!("Capturing to {}", path.display());
}
```

## Discovery And Visibility

You can inspect what the library discovered:

```rust
if let Some(discovery) = session.discovery() {
    println!("Protocol: {:?}", discovery.selected_protocol);
    println!("Probe attempts: {}", discovery.probe_attempts.len());

    for ecu in &discovery.visible_ecus {
        println!("Visible ECU: {}", ecu.id);
    }
}
```

Or use the direct accessor:

```rust
for ecu in session.visible_ecus() {
    println!("Observed {} {} times", ecu.id, ecu.observation_count);
}
```

## Error Handling

Important error classes:

- `Obd2Error::NoData`
- `Obd2Error::ModuleNotFound`
- `Obd2Error::BusNotAvailable`
- `Obd2Error::SecurityRequired`
- `Obd2Error::AdapterBusy`
- `Obd2Error::NegativeResponse`
- `Obd2Error::Adapter`

Typical pattern:

```rust
match session.read_pid(Pid::ENGINE_RPM).await {
    Ok(reading) => println!("{:?}", reading.value),
    Err(obd2_core::error::Obd2Error::NoData) => println!("PID not available"),
    Err(e) => eprintln!("OBD error: {e}"),
}
```

## Custom Transport Integration

If you need a custom physical connection, implement `Transport`.

```rust,no_run
use async_trait::async_trait;
use obd2_core::error::Obd2Error;
use obd2_core::transport::Transport;

struct WifiTransport;

#[async_trait]
impl Transport for WifiTransport {
    async fn write(&mut self, _data: &[u8]) -> Result<(), Obd2Error> { Ok(()) }
    async fn read(&mut self) -> Result<Vec<u8>, Obd2Error> { Ok(vec![]) }
    async fn reset(&mut self) -> Result<(), Obd2Error> { Ok(()) }
    fn name(&self) -> &str { "wifi" }
}
```

Then wrap it in an adapter and pass it into `Session`.

## Integration Rules

Use these rules when building on the library:

1. Treat `Session` as the only supported high-level API.
2. Do not send logical module requests directly through adapters.
3. Do not build application flows around adapter internals.
4. Keep your transport implementation byte-oriented only.
5. Load vehicle specs before identification if you need custom routing or enhanced data.
6. Use discovery and raw captures when validating support across vehicle years and adapter types.

## Current Scope

This guide covers the current rewritten non-J1939 surface.

J1939 remains a separate workstream and should not be treated as complete `1.0` integration guidance yet.
