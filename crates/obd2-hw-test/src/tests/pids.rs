use std::time::Instant;

use serde_json::{json, Map, Value as JsonValue};

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "pids",
    run: run_boxed,
    requires_j1939: false,
    requires_spec_match: false,
    requires_interactive: false,
};

fn run_boxed<'a>(ctx: &'a mut TestContext<'a>) -> GroupFuture<'a> {
    Box::pin(run(ctx))
}

async fn run(ctx: &mut TestContext<'_>) -> crate::report::TestGroupResult {
    let started = Instant::now();
    let mut readings = Map::new();
    let mut errors = Vec::new();
    let mut out_of_range = Vec::new();

    for &pid in ctx.vehicle.required_pids {
        match ctx.session.read_pid(pid).await {
            Ok(reading) => {
                let range = ctx.vehicle.plausible_range(pid);
                let scalar = tests::scalar_value(&reading).ok();
                let within_range = match (scalar, range) {
                    (Some(value), Some((min, max))) => value >= min && value <= max,
                    _ => true,
                };

                if !within_range {
                    out_of_range.push(pid.name().to_string());
                }

                let mut entry = Map::new();
                entry.insert("pid_code".into(), json!(format!("0x{:02X}", pid.0)));
                entry.insert("value".into(), tests::value_to_json(&reading.value));
                entry.insert("unit".into(), json!(pid.unit()));
                entry.insert(
                    "raw_bytes".into(),
                    json!(tests::hex_bytes(&reading.raw_bytes)),
                );
                if let Some((min, max)) = range {
                    entry.insert("range".into(), json!([min, max]));
                    entry.insert("within_range".into(), json!(within_range));
                }

                readings.insert(pid.name().to_string(), JsonValue::Object(entry));
            }
            Err(error) => {
                errors.push(json!({
                    "pid_code": format!("0x{:02X}", pid.0),
                    "pid_name": pid.name(),
                    "error": error.to_string(),
                }));
            }
        }
    }

    let details = json!({
        "readings": readings,
        "errors": errors,
        "out_of_range": out_of_range,
    });

    if !errors.is_empty() {
        tests::fail(started, "one or more required PID reads failed", details)
    } else if !out_of_range.is_empty() {
        tests::fail(
            started,
            "one or more required PID values were outside the plausible range",
            details,
        )
    } else {
        tests::pass(started, details)
    }
}
