use std::time::Instant;

use serde_json::{json, Value as JsonValue};

use obd2_core::protocol::dtc::Dtc;
use obd2_core::protocol::enhanced::{Reading, Value};
use obd2_core::protocol::service::{MonitorStatus, O2TestResult, ReadinessStatus, TestResult};

use crate::report::{TestGroupResult, TestStatus};
use crate::runner::TestGroup;

pub mod capture;
pub mod dtcs;
pub mod enhanced;
pub mod init;
pub mod j1939;
pub mod monitoring;
pub mod pids;
pub mod polling;
pub mod protocol;
pub mod recovery;
pub mod supported;
pub mod vin;
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
        polling::GROUP,
        capture::GROUP,
        j1939::GROUP,
        enhanced::GROUP,
        monitoring::GROUP,
        recovery::GROUP,
    ]
}

pub fn group_names() -> Vec<&'static str> {
    all_test_groups()
        .into_iter()
        .map(|group| group.name)
        .collect()
}

pub fn pass(started: Instant, details: JsonValue) -> TestGroupResult {
    TestGroupResult {
        status: TestStatus::Pass,
        duration_ms: elapsed_ms(started),
        reason: None,
        details: Some(details),
    }
}

pub fn fail(started: Instant, reason: impl Into<String>, details: JsonValue) -> TestGroupResult {
    TestGroupResult {
        status: TestStatus::Fail,
        duration_ms: elapsed_ms(started),
        reason: Some(reason.into()),
        details: Some(details),
    }
}

pub fn skip(reason: impl Into<String>) -> TestGroupResult {
    TestGroupResult {
        status: TestStatus::Skipped,
        duration_ms: 0,
        reason: Some(reason.into()),
        details: None,
    }
}

pub fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

pub fn scalar_value(reading: &Reading) -> Result<f64, String> {
    reading.value.as_f64().map_err(|error| error.to_string())
}

pub fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Scalar(value) => json!(value),
        Value::State(value) => json!(value),
        Value::Raw(bytes) => json!({
            "kind": "raw",
            "hex": hex_bytes(bytes),
        }),
        Value::Bitfield(bitfield) => json!({
            "kind": "bitfield",
            "raw": bitfield.raw,
            "flags": bitfield.flags.iter().map(|(name, enabled)| json!({
                "name": name,
                "enabled": enabled,
            })).collect::<Vec<_>>(),
        }),
        _ => json!({
            "kind": "unsupported",
            "debug": format!("{value:?}"),
        }),
    }
}

pub fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn dtcs_to_json(dtcs: &[Dtc]) -> JsonValue {
    json!(dtcs
        .iter()
        .map(|dtc| json!({
            "code": dtc.code,
            "status": format!("{:?}", dtc.status),
            "description": dtc.description,
        }))
        .collect::<Vec<_>>())
}

pub fn readiness_to_json(readiness: &ReadinessStatus) -> JsonValue {
    json!({
        "mil_on": readiness.mil_on,
        "dtc_count": readiness.dtc_count,
        "compression_ignition": readiness.compression_ignition,
        "monitors": readiness.monitors.iter().map(monitor_to_json).collect::<Vec<_>>(),
    })
}

pub fn monitor_to_json(monitor: &MonitorStatus) -> JsonValue {
    json!({
        "name": monitor.name,
        "supported": monitor.supported,
        "complete": monitor.complete,
    })
}

pub fn o2_results_to_json(results: &[O2TestResult]) -> JsonValue {
    json!(results
        .iter()
        .map(|result| json!({
            "test_id": format!("0x{:02X}", result.test_id),
            "test_name": result.test_name,
            "sensor": result.sensor.to_string(),
            "value": result.value,
            "unit": result.unit,
        }))
        .collect::<Vec<_>>())
}

pub fn test_results_to_json(results: &[TestResult]) -> JsonValue {
    json!(results
        .iter()
        .map(|result| json!({
            "test_id": format!("0x{:02X}", result.test_id),
            "name": result.name,
            "value": result.value,
            "min": result.min,
            "max": result.max,
            "passed": result.passed,
            "unit": result.unit,
        }))
        .collect::<Vec<_>>())
}
