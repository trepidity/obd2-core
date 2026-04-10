use std::time::Instant;

use serde_json::json;

use obd2_core::protocol::j1939::{
    decode_ccvs, decode_dm1, decode_eec1, decode_eflp1, decode_et1, decode_lfe, Pgn,
};

use crate::runner::{GroupFuture, TestContext, TestGroup};
use crate::tests;

pub const GROUP: TestGroup = TestGroup {
    name: "j1939",
    run: run_boxed,
    requires_j1939: true,
    requires_spec_match: false,
    requires_interactive: false,
};

fn run_boxed<'a>(ctx: &'a mut TestContext<'a>) -> GroupFuture<'a> {
    Box::pin(run(ctx))
}

async fn run(ctx: &mut TestContext<'_>) -> crate::report::TestGroupResult {
    let started = Instant::now();

    let eec1_raw = match ctx.session.read_j1939_pgn(Pgn::EEC1).await {
        Ok(data) => data,
        Err(error) => {
            return tests::fail(
                started,
                format!("J1939 EEC1 read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };
    let ccvs_raw = match ctx.session.read_j1939_pgn(Pgn::CCVS).await {
        Ok(data) => data,
        Err(error) => {
            return tests::fail(
                started,
                format!("J1939 CCVS read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };
    let et1_raw = match ctx.session.read_j1939_pgn(Pgn::ET1).await {
        Ok(data) => data,
        Err(error) => {
            return tests::fail(
                started,
                format!("J1939 ET1 read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };
    let eflp1_raw = match ctx.session.read_j1939_pgn(Pgn::EFLP1).await {
        Ok(data) => data,
        Err(error) => {
            return tests::fail(
                started,
                format!("J1939 EFLP1 read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };
    let lfe_raw = match ctx.session.read_j1939_pgn(Pgn::LFE).await {
        Ok(data) => data,
        Err(error) => {
            return tests::fail(
                started,
                format!("J1939 LFE read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };
    let dm1_raw = match ctx.session.read_j1939_pgn(Pgn::DM1).await {
        Ok(data) => data,
        Err(error) => {
            return tests::fail(
                started,
                format!("J1939 DM1 read failed: {error}"),
                json!({ "error": error.to_string() }),
            );
        }
    };

    let Some(eec1) = decode_eec1(&eec1_raw) else {
        return tests::fail(
            started,
            "failed to decode J1939 EEC1 payload",
            json!({ "raw": tests::hex_bytes(&eec1_raw) }),
        );
    };
    let Some(ccvs) = decode_ccvs(&ccvs_raw) else {
        return tests::fail(
            started,
            "failed to decode J1939 CCVS payload",
            json!({ "raw": tests::hex_bytes(&ccvs_raw) }),
        );
    };
    let Some(et1) = decode_et1(&et1_raw) else {
        return tests::fail(
            started,
            "failed to decode J1939 ET1 payload",
            json!({ "raw": tests::hex_bytes(&et1_raw) }),
        );
    };
    let Some(eflp1) = decode_eflp1(&eflp1_raw) else {
        return tests::fail(
            started,
            "failed to decode J1939 EFLP1 payload",
            json!({ "raw": tests::hex_bytes(&eflp1_raw) }),
        );
    };
    let Some(lfe) = decode_lfe(&lfe_raw) else {
        return tests::fail(
            started,
            "failed to decode J1939 LFE payload",
            json!({ "raw": tests::hex_bytes(&lfe_raw) }),
        );
    };
    let dm1 = decode_dm1(&dm1_raw);

    tests::pass(
        started,
        json!({
            "eec1": {
                "engine_rpm": eec1.engine_rpm,
                "driver_demand_torque_pct": eec1.driver_demand_torque_pct,
                "actual_torque_pct": eec1.actual_torque_pct,
                "torque_mode": eec1.torque_mode,
            },
            "ccvs": {
                "vehicle_speed": ccvs.vehicle_speed,
                "brake_switch": ccvs.brake_switch,
                "cruise_active": ccvs.cruise_active,
            },
            "et1": {
                "coolant_temp": et1.coolant_temp,
                "fuel_temp": et1.fuel_temp,
                "oil_temp": et1.oil_temp,
            },
            "eflp1": {
                "oil_pressure": eflp1.oil_pressure,
                "coolant_pressure": eflp1.coolant_pressure,
            },
            "lfe": {
                "fuel_rate": lfe.fuel_rate,
                "instantaneous_fuel_economy": lfe.instantaneous_fuel_economy,
            },
            "dm1_count": dm1.len(),
            "dm1": dm1.iter().map(|dtc| json!({
                "spn": dtc.spn,
                "fmi": dtc.fmi,
                "occurrence_count": dtc.occurrence_count,
            })).collect::<Vec<_>>(),
        }),
    )
}
