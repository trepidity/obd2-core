use std::collections::BTreeSet;
use std::time::Instant;

use serde_json::json;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "supported",
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
    let supported = match ctx.session.supported_pids().await {
        Ok(pids) => pids,
        Err(error) => {
            return tests::fail(
                started,
                format!("supported PID query failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    let supported_codes = supported
        .iter()
        .map(|pid| format!("0x{:02X}", pid.0))
        .collect::<BTreeSet<_>>();
    let required_present = ctx
        .vehicle
        .required_pids
        .iter()
        .filter(|pid| supported.contains(pid))
        .map(|pid| format!("0x{:02X}", pid.0))
        .collect::<Vec<_>>();
    let required_missing = ctx
        .vehicle
        .required_pids
        .iter()
        .filter(|pid| !supported.contains(pid))
        .map(|pid| format!("0x{:02X}", pid.0))
        .collect::<Vec<_>>();

    let details = json!({
        "supported_codes": supported_codes.into_iter().collect::<Vec<_>>(),
        "required_present": required_present,
        "required_missing": required_missing,
    });

    if ctx
        .vehicle
        .required_pids
        .iter()
        .all(|pid| supported.contains(pid))
    {
        tests::pass(started, details)
    } else {
        tests::fail(
            started,
            "one or more required PIDs are not reported as supported",
            details,
        )
    }
}
