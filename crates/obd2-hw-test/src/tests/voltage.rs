use std::time::Instant;

use serde_json::json;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "voltage",
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
    let voltage = match ctx.session.battery_voltage().await {
        Ok(Some(voltage)) => voltage,
        Ok(None) => {
            return tests::fail(
                started,
                "adapter did not return a battery voltage reading",
                json!({ "voltage": null }),
            );
        }
        Err(error) => {
            return tests::fail(
                started,
                format!("battery voltage read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    let details = json!({
        "voltage": voltage,
        "range": [11.0, 15.5],
    });

    if (11.0..=15.5).contains(&voltage) {
        tests::pass(started, details)
    } else {
        tests::fail(
            started,
            "battery voltage was outside the expected range",
            details,
        )
    }
}
