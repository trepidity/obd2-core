use std::time::Instant;

use serde_json::json;
use tokio::sync::mpsc;

use obd2_core::protocol::pid::Pid;
use obd2_core::session::poller::{execute_poll_cycle, PollConfig, PollEvent};

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "polling",
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
    let cycles = 100u32;
    let config = PollConfig::new(vec![Pid::ENGINE_RPM, Pid::COOLANT_TEMP, Pid::VEHICLE_SPEED])
        .with_voltage(false);
    let (tx, mut rx) = mpsc::channel(512);

    for _ in 0..cycles {
        execute_poll_cycle(ctx.session, &config, &tx, None).await;
    }

    drop(tx);

    let mut reading_count = 0u32;
    let mut error_count = 0u32;
    while let Some(event) = rx.recv().await {
        match event {
            PollEvent::Reading { .. } => reading_count += 1,
            PollEvent::Error { .. } => error_count += 1,
            _ => {}
        }
    }

    let duration = started.elapsed();
    let reads_per_sec = if duration.as_secs_f64() > 0.0 {
        reading_count as f64 / duration.as_secs_f64()
    } else {
        0.0
    };
    let details = json!({
        "cycles": cycles,
        "total_readings": reading_count,
        "errors": error_count,
        "duration_ms": duration.as_millis(),
        "reads_per_sec": (reads_per_sec * 10.0).round() / 10.0,
    });

    if error_count == 0 {
        tests::pass(started, details)
    } else {
        tests::fail(
            started,
            format!("{error_count} polling errors occurred"),
            details,
        )
    }
}
