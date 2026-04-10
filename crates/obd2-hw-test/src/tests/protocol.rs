use std::time::Instant;

use serde_json::json;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "protocol",
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
    if let Err(error) = ctx.session.initialize().await {
        return tests::fail(
            started,
            format!("initialization failed: {error}"),
            json!({ "error": error.to_string() }),
        );
    }

    let detected = ctx.session.adapter_info().protocol;
    let expected = ctx.vehicle.expected_protocol;
    let details = json!({
        "expected_protocol": format!("{expected:?}"),
        "detected_protocol": format!("{detected:?}"),
    });

    if detected == expected {
        tests::pass(started, details)
    } else {
        tests::fail(
            started,
            "detected protocol did not match the vehicle expectation",
            details,
        )
    }
}
