use std::time::Instant;

use serde_json::json;

use obd2_core::vehicle::ModuleId;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "enhanced",
    run: run_boxed,
    requires_j1939: false,
    requires_spec_match: true,
    requires_interactive: false,
};

fn run_boxed<'a>(ctx: &'a mut TestContext<'a>) -> GroupFuture<'a> {
    Box::pin(run(ctx))
}

async fn run(ctx: &mut TestContext<'_>) -> crate::report::TestGroupResult {
    let started = Instant::now();
    let module = ModuleId::new(ctx.vehicle.enhanced_module);
    let pids = ctx
        .session
        .module_pids(module.clone())
        .into_iter()
        .take(3)
        .map(|pid| {
            (
                pid.did,
                pid.name.clone(),
                pid.service_id,
                pid.unit.clone(),
                format!("{:?}", pid.confidence),
            )
        })
        .collect::<Vec<_>>();
    if pids.is_empty() {
        return tests::skip(format!(
            "matched spec has no enhanced PIDs for module {}",
            ctx.vehicle.enhanced_module
        ));
    }

    let mut sampled = Vec::new();
    let mut errors = Vec::new();
    for (did, name, service_id, unit, confidence) in pids {
        match ctx.session.read_enhanced(did, module.clone()).await {
            Ok(reading) => {
                sampled.push(json!({
                    "did": format!("0x{:04X}", did),
                    "name": name,
                    "service_id": format!("0x{:02X}", service_id),
                    "unit": unit,
                    "confidence": confidence,
                    "value": tests::value_to_json(&reading.value),
                    "raw_bytes": tests::hex_bytes(&reading.raw_bytes),
                }));
            }
            Err(error) => {
                errors.push(json!({
                    "did": format!("0x{:04X}", did),
                    "name": name,
                    "error": error.to_string(),
                }));
            }
        }
    }

    let details = json!({
        "module": ctx.vehicle.enhanced_module,
        "sampled": sampled,
        "errors": errors,
    });

    if errors.is_empty() && !sampled.is_empty() {
        tests::pass(started, details)
    } else if sampled.is_empty() {
        tests::fail(started, "no enhanced PID reads succeeded", details)
    } else {
        tests::fail(started, "one or more enhanced PID reads failed", details)
    }
}
