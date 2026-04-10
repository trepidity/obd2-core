use std::time::Instant;

use serde_json::json;
use tokio::time::{sleep, Duration};

use obd2_core::protocol::pid::Pid;
use obd2_core::session::ConnectionState;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "recovery",
    run: run_boxed,
    requires_j1939: false,
    requires_spec_match: false,
    requires_interactive: true,
};

fn run_boxed<'a>(ctx: &'a mut TestContext<'a>) -> GroupFuture<'a> {
    Box::pin(run(ctx))
}

async fn run(ctx: &mut TestContext<'_>) -> crate::report::TestGroupResult {
    let started = Instant::now();
    let _interactive = ctx.interactive;

    println!("Turn ignition OFF now. Waiting up to 30 seconds...");
    let mut ignition_off_observed = false;
    for _ in 0..30 {
        let _ = ctx.session.read_pid(Pid::ENGINE_RPM).await;
        if matches!(ctx.session.connection_state(), ConnectionState::IgnitionOff) {
            ignition_off_observed = true;
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    if !ignition_off_observed {
        return tests::fail(
            started,
            "ignition-off state was not observed within 30 seconds",
            json!({
                "ignition_off_observed": false,
                "recovered": false,
            }),
        );
    }

    println!("Turn ignition ON now. Waiting up to 30 seconds for recovery...");
    let mut recovered = false;
    for _ in 0..30 {
        let _ = ctx.session.read_pid(Pid::ENGINE_RPM).await;
        if matches!(ctx.session.connection_state(), ConnectionState::Connected) {
            recovered = true;
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    let details = json!({
        "ignition_off_observed": ignition_off_observed,
        "recovered": recovered,
    });

    if recovered {
        tests::pass(started, details)
    } else {
        tests::fail(
            started,
            "session did not recover to Connected within 30 seconds",
            details,
        )
    }
}
