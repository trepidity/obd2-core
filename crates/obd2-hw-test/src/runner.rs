use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use obd2_core::adapter::elm327::Elm327Adapter;
use obd2_core::session::Session;

use crate::report::{Report, ReportMeta, TestGroupResult, TestStatus};
use crate::vehicles::ExpectedVehicle;

pub struct TestContext<'a> {
    pub session: &'a mut Session<Elm327Adapter>,
    pub vehicle: &'a ExpectedVehicle,
    pub interactive: bool,
}

pub type GroupFuture<'a> = Pin<Box<dyn Future<Output = TestGroupResult> + 'a>>;
pub type GroupRunner = for<'a> fn(&'a mut TestContext<'a>) -> GroupFuture<'a>;

pub struct TestGroup {
    pub name: &'static str,
    pub run: GroupRunner,
    pub requires_j1939: bool,
    pub requires_spec_match: bool,
    pub requires_interactive: bool,
}

pub async fn run_matrix(
    session: &mut Session<Elm327Adapter>,
    vehicle: &ExpectedVehicle,
    meta: ReportMeta,
    only: Option<&[String]>,
    interactive: bool,
) -> Report {
    let started = Instant::now();
    let mut report = Report::new(meta);

    for group in crate::tests::all_test_groups() {
        if let Some(only) = only {
            if !only.iter().any(|item| item == group.name) {
                continue;
            }
        }

        if group.requires_j1939 && !vehicle.has_j1939 {
            report.tests.insert(
                group.name.to_string(),
                TestGroupResult {
                    status: TestStatus::Skipped,
                    duration_ms: 0,
                    reason: Some("vehicle does not support J1939".into()),
                    details: None,
                },
            );
            continue;
        }

        if group.requires_spec_match && !vehicle.has_spec_match {
            report.tests.insert(
                group.name.to_string(),
                TestGroupResult {
                    status: TestStatus::Skipped,
                    duration_ms: 0,
                    reason: Some("vehicle does not have a matched spec".into()),
                    details: None,
                },
            );
            continue;
        }

        if group.requires_interactive && !interactive {
            report.tests.insert(
                group.name.to_string(),
                TestGroupResult {
                    status: TestStatus::Skipped,
                    duration_ms: 0,
                    reason: Some("requires --interactive".into()),
                    details: None,
                },
            );
            continue;
        }

        println!("  [{:>10}] running", group.name);
        let mut ctx = TestContext {
            session,
            vehicle,
            interactive,
        };
        let result = (group.run)(&mut ctx).await;
        println!(
            "  [{:>10}] {:?} ({}ms)",
            group.name, result.status, result.duration_ms
        );
        report.tests.insert(group.name.to_string(), result);
    }

    report.summary.duration_secs = started.elapsed().as_secs_f64();
    report.compute_summary();
    report.summary.duration_secs = started.elapsed().as_secs_f64();
    report
}
