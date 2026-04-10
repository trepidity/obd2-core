use std::time::Instant;

use serde_json::json;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "vin",
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

    let profile = match ctx.session.identify_vehicle().await {
        Ok(profile) => profile,
        Err(error) => {
            return tests::fail(
                started,
                format!("vehicle identification failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    let manufacturer = profile
        .decoded_vin
        .as_ref()
        .and_then(|decoded| decoded.manufacturer.clone());
    let vin_ok = ctx.vehicle.vin.matches(&profile.vin);
    let make_ok = manufacturer
        .as_deref()
        .is_some_and(|make| make.eq_ignore_ascii_case(ctx.vehicle.expected_make));
    let spec_matched = profile.spec.is_some();
    let spec_ok = !ctx.vehicle.has_spec_match || spec_matched;

    let details = json!({
        "vin": profile.vin,
        "vin_expectation": ctx.vehicle.vin.describe(),
        "expected_vin": ctx.vehicle.vin.expected(),
        "manufacturer": manufacturer,
        "expected_make": ctx.vehicle.expected_make,
        "spec_matched": spec_matched,
    });

    if vin_ok && make_ok && spec_ok {
        tests::pass(started, details)
    } else if !vin_ok {
        tests::fail(
            started,
            "VIN did not match the vehicle expectation",
            details,
        )
    } else if !make_ok {
        tests::fail(
            started,
            "decoded manufacturer did not match the vehicle expectation",
            details,
        )
    } else {
        tests::fail(
            started,
            "expected a matched vehicle spec but none was resolved",
            details,
        )
    }
}
