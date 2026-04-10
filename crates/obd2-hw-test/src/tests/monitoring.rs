use std::time::Instant;

use serde_json::json;

use obd2_core::error::Obd2Error;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "monitoring",
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

    let readiness = match ctx.session.read_readiness().await {
        Ok(readiness) => readiness,
        Err(error) => {
            return tests::fail(
                started,
                format!("readiness read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    let (o2_results, o2_no_data) = match ctx.session.read_all_o2_monitoring().await {
        Ok(results) => (results, false),
        Err(Obd2Error::NoData) => (Vec::new(), true),
        Err(error) => {
            return tests::fail(
                started,
                format!("O2 monitoring read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    let (mode6_results, mode6_no_data) = match ctx.session.read_test_results(0x01).await {
        Ok(results) => (results, false),
        Err(Obd2Error::NoData) => (Vec::new(), true),
        Err(error) => {
            return tests::fail(
                started,
                format!("Mode 06 read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    tests::pass(
        started,
        json!({
            "readiness": tests::readiness_to_json(&readiness),
            "o2_no_data": o2_no_data,
            "o2_results": tests::o2_results_to_json(&o2_results),
            "o2_result_count": o2_results.len(),
            "mode06_no_data": mode6_no_data,
            "mode06_results": tests::test_results_to_json(&mode6_results),
            "mode06_result_count": mode6_results.len(),
        }),
    )
}
