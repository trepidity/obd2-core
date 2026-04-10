# Hardware Parity Matrix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a CLI harness (`obd2-hw-test`) that runs a 13-group diagnostic test matrix against real OBD-II hardware over USB and BLE, produces JSON reports, and compares transport parity.

**Architecture:** A new binary crate `crates/obd2-hw-test/` using clap for CLI, serde for JSON reports, and the existing `obd2-core` Session API. Each test group is a function that takes a `&mut Session<Elm327Adapter>` and returns a `TestGroupResult`. The runner orchestrates groups, the reporter writes JSON, and the comparator diffs two reports.

**Tech Stack:** Rust, clap 4, serde/serde_json, chrono, colored (for terminal output), obd2-core with `features = ["full"]`

**Design doc:** `docs/plans/2026-04-09-hardware-parity-matrix-design.md`
**Execution board:** [2026-04-09-hardware-parity-matrix-action-board.md](/Users/jared/Projects/HaulLogic/obd2-core/docs/plans/2026-04-09-hardware-parity-matrix-action-board.md)

---

## Task 1: Scaffold the crate and workspace registration

**Files:**
- Create: `crates/obd2-hw-test/Cargo.toml`
- Modify: `Cargo.toml` (workspace root, line 4 — add member)
- Create: `crates/obd2-hw-test/src/main.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "obd2-hw-test"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false
description = "Hardware parity test harness for obd2-core"

[dependencies]
obd2-core = { path = "../obd2-core", features = ["full"] }
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", default-features = false, features = ["clock", "serde"] }
colored = "2"
async-trait = "0.1"
```

**Step 2: Add to workspace members**

In root `Cargo.toml`, add `"crates/obd2-hw-test"` to the `members` array.

**Step 3: Create minimal main.rs**

```rust
fn main() {
    println!("obd2-hw-test harness");
}
```

**Step 4: Verify build**

Run: `cargo build -p obd2-hw-test`
Expected: compiles without error

**Step 5: Commit**

```bash
git add crates/obd2-hw-test/ Cargo.toml Cargo.lock
git commit -m "feat(hw-test): scaffold obd2-hw-test binary crate"
```

---

## Task 2: CLI with clap — run, compare, vehicles subcommands

**Files:**
- Modify: `crates/obd2-hw-test/src/main.rs`

**Step 1: Implement CLI structure**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "obd2-hw-test", about = "Hardware parity test harness for obd2-core")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the test matrix against real hardware
    Run {
        /// Transport type
        #[arg(long)]
        transport: TransportArg,

        /// Serial port path (required for USB)
        #[arg(long)]
        port: Option<String>,

        /// Vehicle identifier (e.g., duramax-2006)
        #[arg(long)]
        vehicle: String,

        /// Output JSON report path
        #[arg(long, default_value = "results/report.json")]
        output: String,

        /// Only run specific test groups (comma-separated)
        #[arg(long)]
        only: Option<String>,

        /// Enable interactive tests (recovery)
        #[arg(long)]
        interactive: bool,
    },
    /// Compare two JSON reports for parity
    Compare {
        /// First report (typically USB)
        report_a: String,
        /// Second report (typically BLE)
        report_b: String,
    },
    /// List known vehicle definitions
    Vehicles,
}

#[derive(Clone, clap::ValueEnum)]
enum TransportArg {
    Usb,
    Ble,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { transport, port, vehicle, output, only, interactive } => {
            println!("run: transport={transport:?}, vehicle={vehicle}, output={output}");
        }
        Command::Compare { report_a, report_b } => {
            println!("compare: {report_a} vs {report_b}");
        }
        Command::Vehicles => {
            println!("vehicles: (none defined yet)");
        }
    }
}
```

**Step 2: Verify CLI parses**

Run: `cargo run -p obd2-hw-test -- run --transport usb --port /dev/ttyUSB0 --vehicle duramax-2006`
Expected: prints the placeholder message

Run: `cargo run -p obd2-hw-test -- vehicles`
Expected: prints the placeholder message

Run: `cargo run -p obd2-hw-test -- --help`
Expected: shows help with run/compare/vehicles subcommands

**Step 3: Commit**

```bash
git add crates/obd2-hw-test/src/main.rs
git commit -m "feat(hw-test): add clap CLI with run/compare/vehicles subcommands"
```

---

## Task 3: Vehicle definitions

**Files:**
- Create: `crates/obd2-hw-test/src/vehicles.rs`
- Modify: `crates/obd2-hw-test/src/main.rs` (add mod, wire up Vehicles command)

**Step 1: Create vehicles.rs**

```rust
use obd2_core::protocol::pid::Pid;
use obd2_core::vehicle::Protocol;

pub struct ExpectedVehicle {
    pub id: &'static str,
    pub display_name: &'static str,
    pub vin: &'static str,
    pub expected_protocol: Protocol,
    pub expected_make: &'static str,
    pub required_pids: &'static [Pid],
    pub has_j1939: bool,
    pub has_spec_match: bool,
    pub plausible_rpm_range: (f64, f64),
    pub plausible_coolant_range: (f64, f64),
}

// NOTE: Replace VINs with actual VINs from your vehicles before first run.
pub static VEHICLES: &[ExpectedVehicle] = &[
    ExpectedVehicle {
        id: "duramax-2006",
        display_name: "2006 Chevy Duramax 2500",
        vin: "REPLACE_WITH_ACTUAL_VIN",
        expected_protocol: Protocol::J1850Vpw,
        expected_make: "Chevrolet",
        required_pids: &[
            Pid::ENGINE_RPM,
            Pid::COOLANT_TEMP,
            Pid::VEHICLE_SPEED,
            Pid::ENGINE_LOAD,
        ],
        has_j1939: true,
        has_spec_match: true,
        plausible_rpm_range: (0.0, 3500.0),
        plausible_coolant_range: (-40.0, 120.0),
    },
    ExpectedVehicle {
        id: "malibu-2020",
        display_name: "2020 Chevy Malibu",
        vin: "REPLACE_WITH_ACTUAL_VIN",
        expected_protocol: Protocol::Can11Bit500,
        expected_make: "Chevrolet",
        required_pids: &[
            Pid::ENGINE_RPM,
            Pid::COOLANT_TEMP,
            Pid::VEHICLE_SPEED,
            Pid::ENGINE_LOAD,
            Pid::THROTTLE_POSITION,
        ],
        has_j1939: false,
        has_spec_match: false,
        plausible_rpm_range: (0.0, 7000.0),
        plausible_coolant_range: (-40.0, 120.0),
    },
    ExpectedVehicle {
        id: "accord-2001",
        display_name: "2001 Honda Accord",
        vin: "REPLACE_WITH_ACTUAL_VIN",
        expected_protocol: Protocol::Iso9141(obd2_core::vehicle::KLineInit::SlowInit),
        expected_make: "Honda",
        required_pids: &[
            Pid::ENGINE_RPM,
            Pid::COOLANT_TEMP,
            Pid::VEHICLE_SPEED,
        ],
        has_j1939: false,
        has_spec_match: false,
        plausible_rpm_range: (0.0, 7000.0),
        plausible_coolant_range: (-40.0, 120.0),
    },
];

pub fn find_vehicle(id: &str) -> Option<&'static ExpectedVehicle> {
    VEHICLES.iter().find(|v| v.id == id)
}

pub fn list_vehicles() {
    println!("Known vehicles:\n");
    for v in VEHICLES {
        println!(
            "  {:<16} {} (protocol: {:?}, J1939: {}, spec: {})",
            v.id, v.display_name, v.expected_protocol, v.has_j1939, v.has_spec_match,
        );
    }
}
```

**Step 2: Wire into main.rs**

Add `mod vehicles;` and replace the `Vehicles` command handler with `vehicles::list_vehicles()`. Replace the `Run` handler to validate the vehicle exists:

```rust
Command::Run { vehicle, .. } => {
    let v = vehicles::find_vehicle(&vehicle)
        .unwrap_or_else(|| { eprintln!("Unknown vehicle: {vehicle}"); std::process::exit(1); });
    println!("Selected: {} ({})", v.display_name, v.id);
}
```

**Step 3: Verify**

Run: `cargo run -p obd2-hw-test -- vehicles`
Expected: lists 3 vehicles with protocol info

Run: `cargo run -p obd2-hw-test -- run --transport usb --vehicle bogus`
Expected: error "Unknown vehicle: bogus"

**Step 4: Commit**

```bash
git add crates/obd2-hw-test/src/vehicles.rs crates/obd2-hw-test/src/main.rs
git commit -m "feat(hw-test): add vehicle definitions for Duramax, Malibu, Accord"
```

---

## Task 4: Report types and JSON serialization

**Files:**
- Create: `crates/obd2-hw-test/src/report.rs`

**Step 1: Define report types**

```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub meta: ReportMeta,
    pub summary: ReportSummary,
    pub tests: BTreeMap<String, TestGroupResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMeta {
    pub timestamp: String,
    pub harness_version: String,
    pub vehicle_id: String,
    pub transport: String,
    pub port: Option<String>,
    pub adapter_chipset: Option<String>,
    pub adapter_firmware: Option<String>,
    pub protocol_detected: Option<String>,
    pub raw_capture_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGroupResult {
    pub status: TestStatus,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Pass,
    Fail,
    Skipped,
}

impl Report {
    pub fn compute_summary(&mut self) {
        let mut total = 0u32;
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;
        for result in self.tests.values() {
            total += 1;
            match result.status {
                TestStatus::Pass => passed += 1,
                TestStatus::Fail => failed += 1,
                TestStatus::Skipped => skipped += 1,
            }
        }
        self.summary.total = total;
        self.summary.passed = passed;
        self.summary.failed = failed;
        self.summary.skipped = skipped;
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    pub fn read_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}
```

**Step 2: Add mod to main.rs**

Add `mod report;`.

**Step 3: Verify build**

Run: `cargo build -p obd2-hw-test`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/obd2-hw-test/src/report.rs crates/obd2-hw-test/src/main.rs
git commit -m "feat(hw-test): add JSON report types with serialization"
```

---

## Task 5: Test group trait and runner orchestrator

**Files:**
- Create: `crates/obd2-hw-test/src/runner.rs`
- Create: `crates/obd2-hw-test/src/tests/mod.rs`

**Step 1: Create runner.rs**

The runner takes a live `Session<Elm327Adapter>`, a vehicle definition, and a set of enabled test groups. It runs each group, collects results, and builds a `Report`.

```rust
use std::collections::BTreeMap;
use std::time::Instant;

use obd2_core::adapter::elm327::Elm327Adapter;
use obd2_core::session::Session;

use crate::report::{Report, ReportMeta, ReportSummary, TestGroupResult, TestStatus};
use crate::vehicles::ExpectedVehicle;

pub type SessionRef<'a> = &'a mut Session<Elm327Adapter>;

/// Context passed to each test group.
pub struct TestContext<'a> {
    pub session: SessionRef<'a>,
    pub vehicle: &'a ExpectedVehicle,
    pub interactive: bool,
}

pub struct TestGroup {
    pub name: &'static str,
    pub run: fn(&mut TestContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = TestGroupResult> + '_>>,
    /// If true, only runs when the vehicle has this capability.
    pub requires_j1939: bool,
    pub requires_spec_match: bool,
    pub requires_interactive: bool,
}

pub async fn run_matrix(
    session: &mut Session<Elm327Adapter>,
    vehicle: &ExpectedVehicle,
    meta: ReportMeta,
    only: Option<&[&str]>,
    interactive: bool,
) -> Report {
    let groups = crate::tests::all_test_groups();
    let mut results = BTreeMap::new();
    let started = Instant::now();

    for group in &groups {
        // Filter by --only
        if let Some(only) = only {
            if !only.contains(&group.name) {
                continue;
            }
        }

        // Skip conditional groups
        if group.requires_j1939 && !vehicle.has_j1939 {
            results.insert(group.name.to_string(), TestGroupResult {
                status: TestStatus::Skipped,
                duration_ms: 0,
                reason: Some("vehicle does not support J1939".into()),
                details: None,
            });
            continue;
        }
        if group.requires_spec_match && !vehicle.has_spec_match {
            results.insert(group.name.to_string(), TestGroupResult {
                status: TestStatus::Skipped,
                duration_ms: 0,
                reason: Some("no matched vehicle spec".into()),
                details: None,
            });
            continue;
        }
        if group.requires_interactive && !interactive {
            results.insert(group.name.to_string(), TestGroupResult {
                status: TestStatus::Skipped,
                duration_ms: 0,
                reason: Some("requires --interactive flag".into()),
                details: None,
            });
            continue;
        }

        println!("  [{:>12}] running...", group.name);
        let mut ctx = TestContext { session, vehicle, interactive };
        let result = (group.run)(&mut ctx).await;
        let status_str = match &result.status {
            TestStatus::Pass => "PASS",
            TestStatus::Fail => "FAIL",
            TestStatus::Skipped => "SKIP",
        };
        println!("  [{:>12}] {} ({}ms)", group.name, status_str, result.duration_ms);
        results.insert(group.name.to_string(), result);
    }

    let elapsed = started.elapsed();
    let mut report = Report {
        meta,
        summary: ReportSummary {
            total: 0, passed: 0, failed: 0, skipped: 0,
            duration_secs: elapsed.as_secs_f64(),
        },
        tests: results,
    };
    report.compute_summary();
    report
}
```

**Step 2: Create tests/mod.rs stub**

```rust
use crate::runner::TestGroup;

pub mod init;

pub fn all_test_groups() -> Vec<TestGroup> {
    vec![
        init::GROUP,
    ]
}
```

**Step 3: Add mods to main.rs**

Add `mod runner;` and `mod tests;`.

**Step 4: Verify build**

Run: `cargo build -p obd2-hw-test`
Expected: compiles (tests/init.rs will be created next)

**Step 5: Commit**

```bash
git add crates/obd2-hw-test/src/runner.rs crates/obd2-hw-test/src/tests/mod.rs crates/obd2-hw-test/src/main.rs
git commit -m "feat(hw-test): add test runner orchestrator and test group trait"
```

---

## Task 6: First test group — init

**Files:**
- Create: `crates/obd2-hw-test/src/tests/init.rs`

**Step 1: Implement init test group**

This test initializes the adapter and validates chipset detection, firmware version, and capabilities.

```rust
use std::time::Instant;

use obd2_core::adapter::Chipset;

use crate::report::{TestGroupResult, TestStatus};
use crate::runner::{TestContext, TestGroup};

pub const GROUP: TestGroup = TestGroup {
    name: "init",
    run: |ctx| Box::pin(run(ctx)),
    requires_j1939: false,
    requires_spec_match: false,
    requires_interactive: false,
};

async fn run(ctx: &mut TestContext<'_>) -> TestGroupResult {
    let started = Instant::now();

    let info = match ctx.session.initialize().await {
        Ok(info) => info,
        Err(e) => {
            return TestGroupResult {
                status: TestStatus::Fail,
                duration_ms: started.elapsed().as_millis() as u64,
                reason: Some(format!("initialization failed: {e}")),
                details: None,
            };
        }
    };

    let chipset_ok = !matches!(info.chipset, Chipset::Unknown);
    let firmware_ok = !info.firmware.is_empty();

    let status = if chipset_ok && firmware_ok {
        TestStatus::Pass
    } else {
        TestStatus::Fail
    };

    let details = serde_json::json!({
        "chipset": format!("{:?}", info.chipset),
        "firmware": info.firmware,
        "protocol": format!("{:?}", info.protocol),
        "capabilities": {
            "can_clear_dtcs": info.capabilities.can_clear_dtcs,
            "dual_can": info.capabilities.dual_can,
            "enhanced_diag": info.capabilities.enhanced_diag,
            "battery_voltage": info.capabilities.battery_voltage,
            "adaptive_timing": info.capabilities.adaptive_timing,
        }
    });

    TestGroupResult {
        status,
        duration_ms: started.elapsed().as_millis() as u64,
        reason: if !chipset_ok {
            Some("chipset not detected".into())
        } else if !firmware_ok {
            Some("empty firmware string".into())
        } else {
            None
        },
        details: Some(details),
    }
}
```

**Step 2: Verify build**

Run: `cargo build -p obd2-hw-test`
Expected: compiles

**Step 3: Commit**

```bash
git add crates/obd2-hw-test/src/tests/init.rs
git commit -m "feat(hw-test): add init test group (chipset, firmware, capabilities)"
```

---

## Task 7: Wire up the `run` command to create a session and execute

**Files:**
- Modify: `crates/obd2-hw-test/src/main.rs`

**Step 1: Implement the full run command**

Replace the `Command::Run` handler with code that:
1. Looks up the vehicle
2. Creates the appropriate transport (USB or BLE)
3. Wraps in `Elm327Adapter` then `Session`
4. Calls `runner::run_matrix()`
5. Writes the JSON report

```rust
Command::Run { transport, port, vehicle, output, only, interactive } => {
    let v = vehicles::find_vehicle(&vehicle)
        .unwrap_or_else(|| { eprintln!("Unknown vehicle: {vehicle}. Use 'vehicles' to list."); std::process::exit(1); });

    let only_groups: Option<Vec<&str>> = only.as_ref().map(|s| s.split(',').collect());

    println!("=== obd2-hw-test ===");
    println!("Vehicle:   {} ({})", v.display_name, v.id);
    println!("Transport: {:?}", transport);
    println!("Output:    {output}");
    println!();

    let transport_box: Box<dyn obd2_core::transport::Transport> = match transport {
        TransportArg::Usb => {
            let port = port.unwrap_or_else(|| { eprintln!("--port is required for USB transport"); std::process::exit(1); });
            println!("Opening serial port: {port}");
            let t = obd2_core::transport::serial::SerialTransport::new(&port, 115200)
                .unwrap_or_else(|e| { eprintln!("Failed to open serial port: {e}"); std::process::exit(1); });
            Box::new(t)
        }
        TransportArg::Ble => {
            println!("Scanning for BLE adapter...");
            let t = obd2_core::transport::ble::BleTransport::scan_and_connect(
                None,
                std::time::Duration::from_secs(10),
            ).await
            .unwrap_or_else(|e| { eprintln!("BLE connection failed: {e}"); std::process::exit(1); });
            Box::new(t)
        }
    };

    let adapter = obd2_core::adapter::elm327::Elm327Adapter::new(transport_box);
    let mut session = obd2_core::session::Session::new(adapter);

    let transport_name = match transport {
        TransportArg::Usb => "usb",
        TransportArg::Ble => "ble",
    };

    let meta = report::ReportMeta {
        timestamp: chrono::Utc::now().to_rfc3339(),
        harness_version: env!("CARGO_PKG_VERSION").to_string(),
        vehicle_id: v.id.to_string(),
        transport: transport_name.to_string(),
        port: port.clone(),
        adapter_chipset: None,  // filled after init test
        adapter_firmware: None,
        protocol_detected: None,
        raw_capture_path: None,
    };

    println!("\nRunning test matrix...\n");
    let report = runner::run_matrix(
        &mut session,
        v,
        meta,
        only_groups.as_deref(),
        interactive,
    ).await;

    println!("\n=== Summary ===");
    println!("Total: {} | Passed: {} | Failed: {} | Skipped: {}",
        report.summary.total, report.summary.passed,
        report.summary.failed, report.summary.skipped);
    println!("Duration: {:.1}s", report.summary.duration_secs);

    report.write_to_file(&output)
        .unwrap_or_else(|e| { eprintln!("Failed to write report: {e}"); std::process::exit(1); });
    println!("\nReport saved to: {output}");

    if report.summary.failed > 0 {
        std::process::exit(1);
    }
}
```

**Step 2: Verify build**

Run: `cargo build -p obd2-hw-test`
Expected: compiles

**Step 3: Commit**

```bash
git add crates/obd2-hw-test/src/main.rs
git commit -m "feat(hw-test): wire up run command with transport creation and report output"
```

---

## Task 8: Remaining test groups — protocol, vin, pids, supported, dtcs, voltage

**Files:**
- Create: `crates/obd2-hw-test/src/tests/protocol.rs`
- Create: `crates/obd2-hw-test/src/tests/vin.rs`
- Create: `crates/obd2-hw-test/src/tests/pids.rs`
- Create: `crates/obd2-hw-test/src/tests/supported.rs`
- Create: `crates/obd2-hw-test/src/tests/dtcs.rs`
- Create: `crates/obd2-hw-test/src/tests/voltage.rs`
- Modify: `crates/obd2-hw-test/src/tests/mod.rs` (register all groups)

Each follows the same pattern as `init.rs`. Key behaviors:

- **protocol**: calls `session.adapter_info().protocol`, compares to `vehicle.expected_protocol`
- **vin**: calls `session.identify_vehicle()`, asserts VIN matches `vehicle.vin`, checks decoded manufacturer matches `vehicle.expected_make`
- **pids**: reads each `vehicle.required_pids`, asserts value is within plausible range
- **supported**: calls `session.supported_pids()`, asserts all `vehicle.required_pids` are in the set
- **dtcs**: calls `session.read_dtcs()`, `read_pending_dtcs()`, `read_permanent_dtcs()` — pass if no transport error (content is vehicle-state dependent)
- **voltage**: calls `session.battery_voltage()`, asserts value is between 11.0 and 15.5

**Step 1:** Create each file following the `init.rs` pattern (const GROUP, async fn run).

**Step 2:** Register all in `tests/mod.rs`:

```rust
pub mod init;
pub mod protocol;
pub mod vin;
pub mod pids;
pub mod supported;
pub mod dtcs;
pub mod voltage;

pub fn all_test_groups() -> Vec<TestGroup> {
    vec![
        init::GROUP,
        protocol::GROUP,
        vin::GROUP,
        pids::GROUP,
        supported::GROUP,
        dtcs::GROUP,
        voltage::GROUP,
    ]
}
```

**Step 3: Verify build**

Run: `cargo build -p obd2-hw-test`

**Step 4: Commit**

```bash
git add crates/obd2-hw-test/src/tests/
git commit -m "feat(hw-test): add protocol, vin, pids, supported, dtcs, voltage test groups"
```

---

## Task 9: Polling throughput test group

**Files:**
- Create: `crates/obd2-hw-test/src/tests/polling.rs`
- Modify: `crates/obd2-hw-test/src/tests/mod.rs`

**Step 1: Implement polling throughput test**

Runs 100 cycles of 3-PID polling via `execute_poll_cycle()`, measures total time, computes reads/sec. Details include per-cycle timing.

```rust
async fn run(ctx: &mut TestContext<'_>) -> TestGroupResult {
    use obd2_core::protocol::pid::Pid;
    use obd2_core::session::poller::{execute_poll_cycle, PollConfig, PollEvent};
    use tokio::sync::mpsc;

    let started = Instant::now();
    let cycles = 100;
    let config = PollConfig::new(vec![Pid::ENGINE_RPM, Pid::COOLANT_TEMP, Pid::VEHICLE_SPEED])
        .with_voltage(false);

    let (tx, mut rx) = mpsc::channel(cycles * config.pids.len() + 16);
    let mut reading_count = 0u32;
    let mut error_count = 0u32;

    for _ in 0..cycles {
        execute_poll_cycle(ctx.session, &config, &tx, None).await;
    }

    while let Ok(event) = rx.try_recv() {
        match event {
            PollEvent::Reading { .. } => reading_count += 1,
            PollEvent::Error { .. } => error_count += 1,
            _ => {}
        }
    }

    let elapsed = started.elapsed();
    let reads_per_sec = reading_count as f64 / elapsed.as_secs_f64();

    let details = serde_json::json!({
        "cycles": cycles,
        "total_readings": reading_count,
        "errors": error_count,
        "duration_ms": elapsed.as_millis(),
        "reads_per_sec": (reads_per_sec * 10.0).round() / 10.0,
    });

    TestGroupResult {
        status: if error_count == 0 { TestStatus::Pass } else { TestStatus::Fail },
        duration_ms: elapsed.as_millis() as u64,
        reason: if error_count > 0 { Some(format!("{error_count} errors during polling")) } else { None },
        details: Some(details),
    }
}
```

**Step 2:** Register in `tests/mod.rs`, verify build.

**Step 3: Commit**

```bash
git add crates/obd2-hw-test/src/tests/polling.rs crates/obd2-hw-test/src/tests/mod.rs
git commit -m "feat(hw-test): add polling throughput test group"
```

---

## Task 10: Capture, J1939, enhanced, monitoring, recovery test groups

**Files:**
- Create: `crates/obd2-hw-test/src/tests/capture.rs`
- Create: `crates/obd2-hw-test/src/tests/j1939.rs`
- Create: `crates/obd2-hw-test/src/tests/enhanced.rs`
- Create: `crates/obd2-hw-test/src/tests/monitoring.rs`
- Create: `crates/obd2-hw-test/src/tests/recovery.rs`
- Modify: `crates/obd2-hw-test/src/tests/mod.rs`

Key behaviors:

- **capture**: checks `session.raw_capture_path()` is Some, reads file, calls `parse_raw_capture()`, verifies at least 1 command/response pair
- **j1939**: `requires_j1939: true`. Reads EEC1, CCVS, ET1, EFLP1, LFE, DM1 via `session.read_j1939_pgn()`. Decodes each, records values.
- **enhanced**: `requires_spec_match: true`. Calls `session.module_pids(ModuleId::new("ecm"))`, reads first 3 enhanced PIDs via `session.read_enhanced()`.
- **monitoring**: reads readiness, attempts Mode 05 (O2), Mode 06 (test results). NoData is acceptable (pass), transport errors are failures.
- **recovery**: `requires_interactive: true`. Prints "Turn ignition OFF now...", polls `session.connection_state()` until `IgnitionOff`, then prints "Turn ignition ON...", polls until `Connected`.

**Step 1:** Create each file.

**Step 2:** Register all in `tests/mod.rs` (final list of 12 groups).

**Step 3: Verify build**

Run: `cargo build -p obd2-hw-test`

**Step 4: Commit**

```bash
git add crates/obd2-hw-test/src/tests/
git commit -m "feat(hw-test): add capture, j1939, enhanced, monitoring, recovery test groups"
```

---

## Task 11: Compare command

**Files:**
- Create: `crates/obd2-hw-test/src/compare.rs`
- Modify: `crates/obd2-hw-test/src/main.rs` (wire up Compare command)

**Step 1: Implement comparison logic**

```rust
use colored::Colorize;
use crate::report::{Report, TestStatus};

#[derive(Debug)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

pub struct Difference {
    pub severity: Severity,
    pub field: String,
    pub report_a: String,
    pub report_b: String,
}

pub fn compare_reports(a: &Report, b: &Report) -> Vec<Difference> {
    let mut diffs = Vec::new();

    // Protocol mismatch
    if a.meta.protocol_detected != b.meta.protocol_detected {
        diffs.push(Difference {
            severity: Severity::Critical,
            field: "protocol_detected".into(),
            report_a: a.meta.protocol_detected.clone().unwrap_or_default(),
            report_b: b.meta.protocol_detected.clone().unwrap_or_default(),
        });
    }

    // Test status mismatches
    for (name, result_a) in &a.tests {
        if let Some(result_b) = b.tests.get(name) {
            if result_a.status != result_b.status
                && result_a.status != TestStatus::Skipped
                && result_b.status != TestStatus::Skipped
            {
                diffs.push(Difference {
                    severity: Severity::Critical,
                    field: format!("tests.{name}.status"),
                    report_a: format!("{:?}", result_a.status),
                    report_b: format!("{:?}", result_b.status),
                });
            }
        }
    }

    // Throughput comparison (info-level)
    if let (Some(a_poll), Some(b_poll)) = (a.tests.get("polling"), b.tests.get("polling")) {
        if let (Some(a_det), Some(b_det)) = (&a_poll.details, &b_poll.details) {
            if let (Some(a_rps), Some(b_rps)) = (
                a_det.get("reads_per_sec").and_then(|v| v.as_f64()),
                b_det.get("reads_per_sec").and_then(|v| v.as_f64()),
            ) {
                diffs.push(Difference {
                    severity: Severity::Info,
                    field: "polling.reads_per_sec".into(),
                    report_a: format!("{a_rps:.1}"),
                    report_b: format!("{b_rps:.1}"),
                });
            }
        }
    }

    diffs
}

pub fn print_comparison(a: &Report, b: &Report, diffs: &[Difference]) {
    println!("=== Parity Comparison ===");
    println!("  A: {} via {} ({})", a.meta.vehicle_id, a.meta.transport,
        a.meta.timestamp.get(..10).unwrap_or(&a.meta.timestamp));
    println!("  B: {} via {} ({})", b.meta.vehicle_id, b.meta.transport,
        b.meta.timestamp.get(..10).unwrap_or(&b.meta.timestamp));
    println!();

    let critical_count = diffs.iter().filter(|d| matches!(d.severity, Severity::Critical)).count();
    let warning_count = diffs.iter().filter(|d| matches!(d.severity, Severity::Warning)).count();
    let info_count = diffs.iter().filter(|d| matches!(d.severity, Severity::Info)).count();

    for diff in diffs {
        let label = match diff.severity {
            Severity::Critical => "CRITICAL".red().bold(),
            Severity::Warning => "WARNING".yellow().bold(),
            Severity::Info => "INFO".blue(),
        };
        println!("  [{label}] {}: {} vs {}", diff.field, diff.report_a, diff.report_b);
    }

    println!();
    println!("  Critical: {critical_count} | Warning: {warning_count} | Info: {info_count}");

    if critical_count == 0 {
        println!("  {}", "PARITY OK".green().bold());
    } else {
        println!("  {}", "PARITY FAILED".red().bold());
    }
}

/// Returns true if parity holds (no critical differences).
pub fn parity_ok(diffs: &[Difference]) -> bool {
    !diffs.iter().any(|d| matches!(d.severity, Severity::Critical))
}
```

**Step 2:** Wire into `main.rs`:

```rust
Command::Compare { report_a, report_b } => {
    let a = report::Report::read_from_file(&report_a)
        .unwrap_or_else(|e| { eprintln!("Failed to read {report_a}: {e}"); std::process::exit(1); });
    let b = report::Report::read_from_file(&report_b)
        .unwrap_or_else(|e| { eprintln!("Failed to read {report_b}: {e}"); std::process::exit(1); });
    let diffs = compare::compare_reports(&a, &b);
    compare::print_comparison(&a, &b, &diffs);
    if !compare::parity_ok(&diffs) {
        std::process::exit(1);
    }
}
```

**Step 3: Verify build**

Run: `cargo build -p obd2-hw-test`

**Step 4: Commit**

```bash
git add crates/obd2-hw-test/src/compare.rs crates/obd2-hw-test/src/main.rs
git commit -m "feat(hw-test): add compare command with parity diff and colored output"
```

---

## Task 12: CI workflow file

**Files:**
- Create: `.github/workflows/hardware-parity.yml`

**Step 1: Create workflow**

Use the YAML from the design doc (Task 4 of the design: CI Integration section). The workflow uses `self-hosted` runner, matrix strategy over vehicles x transports, and a compare job.

**Step 2: Commit**

```bash
git add .github/workflows/hardware-parity.yml
git commit -m "ci: add hardware parity matrix workflow (self-hosted, weekly)"
```

---

## Task 13: Update project documentation

**Files:**
- Modify: `docs/FUNCTIONAL_REQUIREMENTS.md` — mark real hardware parity as in-progress, reference the harness
- Modify: `docs/plans/2026-04-09-active-execution-board.md` — add hardware parity as a workstream
- Modify: `README.md` — add a "Hardware Testing" section pointing to the harness

**Step 1:** Update FUNCTIONAL_REQUIREMENTS.md Area 19 (Testing) to reference `obd2-hw-test`.

**Step 2:** Update the execution board to note the new workstream.

**Step 3:** Add brief section to README.

**Step 4: Commit**

```bash
git add docs/ README.md
git commit -m "docs: reference hardware parity harness in requirements and readme"
```

---

## Task 14: First real run and VIN population

**Step 1:** Connect the OBDLink EX via USB to one vehicle.

**Step 2:** Run with a placeholder VIN to discover the actual VIN:

```bash
cargo run -p obd2-hw-test --release -- run --transport usb --port /dev/ttyUSB0 --vehicle duramax-2006 --only init,vin
```

**Step 3:** Update `vehicles.rs` with the real VINs from the output.

**Step 4:** Run the full matrix on each vehicle x transport combination.

**Step 5:** Run the compare command on USB vs BLE results for the same vehicle.

**Step 6:** Commit initial results and VINs:

```bash
git add crates/obd2-hw-test/src/vehicles.rs test-results/
git commit -m "hw-test: populate real VINs and initial test results"
```
