use std::time::Instant;

use serde_json::json;

use obd2_core::adapter::Chipset;

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "init",
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

    let info = match ctx.session.initialize().await {
        Ok(info) => info,
        Err(error) => {
            return tests::fail(
                started,
                format!("initialization failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    let chipset_ok = !matches!(info.chipset, Chipset::Unknown);
    let firmware_ok = !info.firmware.trim().is_empty();
    let details = json!({
        "chipset": format!("{:?}", info.chipset),
        "firmware": info.firmware,
        "protocol": format!("{:?}", info.protocol),
        "capabilities": {
            "can_clear_dtcs": info.capabilities.can_clear_dtcs,
            "dual_can": info.capabilities.dual_can,
            "enhanced_diag": info.capabilities.enhanced_diag,
            "battery_voltage": info.capabilities.battery_voltage,
            "adaptive_timing": info.capabilities.adaptive_timing,
            "kline_init": info.capabilities.kline_init,
            "kline_wakeup": info.capabilities.kline_wakeup,
            "can_filtering": info.capabilities.can_filtering,
            "can_flow_control": info.capabilities.can_flow_control,
        }
    });

    if chipset_ok && firmware_ok {
        tests::pass(started, details)
    } else if !chipset_ok {
        tests::fail(started, "chipset was not detected", details)
    } else {
        tests::fail(started, "firmware string was empty", details)
    }
}
