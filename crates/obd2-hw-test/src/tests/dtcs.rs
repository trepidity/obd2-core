use std::time::Instant;

use serde_json::json;

use obd2_core::error::Obd2Error;
use obd2_core::protocol::dtc::Dtc;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "dtcs",
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

    let stored = match dtc_result(ctx.session.read_dtcs().await) {
        Ok(dtcs) => dtcs,
        Err(error) => {
            return tests::fail(
                started,
                format!("stored DTC read failed: {error}"),
                json!({ "error": error }),
            );
        }
    };
    let pending = match dtc_result(ctx.session.read_pending_dtcs().await) {
        Ok(dtcs) => dtcs,
        Err(error) => {
            return tests::fail(
                started,
                format!("pending DTC read failed: {error}"),
                json!({ "error": error }),
            );
        }
    };
    let permanent = match dtc_result(ctx.session.read_permanent_dtcs().await) {
        Ok(dtcs) => dtcs,
        Err(error) => {
            return tests::fail(
                started,
                format!("permanent DTC read failed: {error}"),
                json!({ "error": error }),
            );
        }
    };

    tests::pass(
        started,
        json!({
            "stored": tests::dtcs_to_json(&stored),
            "pending": tests::dtcs_to_json(&pending),
            "permanent": tests::dtcs_to_json(&permanent),
            "stored_count": stored.len(),
            "pending_count": pending.len(),
            "permanent_count": permanent.len(),
        }),
    )
}

fn dtc_result(result: Result<Vec<Dtc>, Obd2Error>) -> Result<Vec<Dtc>, String> {
    match result {
        Ok(dtcs) => Ok(dtcs),
        Err(Obd2Error::NoData) => Ok(Vec::new()),
        Err(error) => Err(error.to_string()),
    }
}
