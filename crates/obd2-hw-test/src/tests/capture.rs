use std::time::Instant;

use serde_json::json;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "capture",
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
    let Some(path) = ctx.session.raw_capture_path() else {
        return tests::fail(
            started,
            "raw capture path was not set",
            json!({ "raw_capture_path": null }),
        );
    };

    let pairs = match obd2_core::transport::parse_raw_capture(path) {
        Ok(pairs) => pairs,
        Err(error) => {
            return tests::fail(
                started,
                format!("failed to parse raw capture: {error}"),
                json!({
                    "raw_capture_path": path.display().to_string(),
                    "error": error.to_string(),
                }),
            );
        }
    };

    let details = json!({
        "raw_capture_path": path.display().to_string(),
        "pair_count": pairs.len(),
        "first_command": pairs.first().map(|pair| pair.0.clone()),
    });

    if pairs.is_empty() {
        tests::fail(
            started,
            "raw capture contained no command/response pairs",
            details,
        )
    } else {
        tests::pass(started, details)
    }
}
