use std::collections::{BTreeMap, BTreeSet};

use colored::Colorize;

use crate::report::{Report, TestStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

pub struct Difference {
    pub severity: Severity,
    pub field: String,
    pub report_a: String,
    pub report_b: String,
}

pub fn compare_reports(a: &Report, b: &Report) -> Vec<Difference> {
    let mut diffs = Vec::new();

    if a.meta.vehicle_id != b.meta.vehicle_id {
        diffs.push(Difference {
            severity: Severity::Critical,
            field: "meta.vehicle_id".into(),
            report_a: a.meta.vehicle_id.clone(),
            report_b: b.meta.vehicle_id.clone(),
        });
    }

    if a.fatal_error != b.fatal_error {
        diffs.push(Difference {
            severity: Severity::Critical,
            field: "fatal_error".into(),
            report_a: a.fatal_error.clone().unwrap_or_default(),
            report_b: b.fatal_error.clone().unwrap_or_default(),
        });
    }

    if a.meta.protocol_detected != b.meta.protocol_detected {
        diffs.push(Difference {
            severity: Severity::Critical,
            field: "meta.protocol_detected".into(),
            report_a: a.meta.protocol_detected.clone().unwrap_or_default(),
            report_b: b.meta.protocol_detected.clone().unwrap_or_default(),
        });
    }

    if let (Some(vin_a), Some(vin_b)) = (
        test_detail_string(a, "vin", "vin"),
        test_detail_string(b, "vin", "vin"),
    ) {
        if vin_a != vin_b {
            diffs.push(Difference {
                severity: Severity::Critical,
                field: "tests.vin.vin".into(),
                report_a: vin_a,
                report_b: vin_b,
            });
        }
    }

    let all_tests: BTreeSet<_> = a
        .tests
        .keys()
        .chain(b.tests.keys())
        .map(String::as_str)
        .collect();

    for name in all_tests {
        match (a.tests.get(name), b.tests.get(name)) {
            (Some(left), Some(right)) => {
                if left.status != right.status
                    && left.status != TestStatus::Skipped
                    && right.status != TestStatus::Skipped
                {
                    diffs.push(Difference {
                        severity: Severity::Critical,
                        field: format!("tests.{name}.status"),
                        report_a: format!("{:?}", left.status),
                        report_b: format!("{:?}", right.status),
                    });
                }
            }
            (Some(_), None) | (None, Some(_)) => {
                diffs.push(Difference {
                    severity: Severity::Critical,
                    field: format!("tests.{name}"),
                    report_a: if a.tests.contains_key(name) {
                        "present"
                    } else {
                        "missing"
                    }
                    .into(),
                    report_b: if b.tests.contains_key(name) {
                        "present"
                    } else {
                        "missing"
                    }
                    .into(),
                });
            }
            (None, None) => {}
        }
    }

    if let (Some(left), Some(right)) = (
        test_detail_array(a, "supported", "supported_codes"),
        test_detail_array(b, "supported", "supported_codes"),
    ) {
        let left_set: BTreeSet<_> = left.into_iter().collect();
        let right_set: BTreeSet<_> = right.into_iter().collect();
        if left_set != right_set {
            diffs.push(Difference {
                severity: Severity::Warning,
                field: "tests.supported.supported_codes".into(),
                report_a: format_set(&left_set),
                report_b: format_set(&right_set),
            });
        }
    }

    let pid_values_a = test_detail_pid_values(a, "pids", "readings");
    let pid_values_b = test_detail_pid_values(b, "pids", "readings");
    for (pid_name, value_a) in &pid_values_a {
        if let Some(value_b) = pid_values_b.get(pid_name) {
            let tolerance = pid_tolerance(pid_name);
            if (value_a - value_b).abs() > tolerance {
                diffs.push(Difference {
                    severity: Severity::Warning,
                    field: format!("tests.pids.readings.{pid_name}"),
                    report_a: format!("{value_a:.2}"),
                    report_b: format!("{value_b:.2}"),
                });
            }
        }
    }

    if let (Some(left), Some(right)) = (
        test_detail_f64(a, "polling", "reads_per_sec"),
        test_detail_f64(b, "polling", "reads_per_sec"),
    ) {
        diffs.push(Difference {
            severity: Severity::Info,
            field: "tests.polling.reads_per_sec".into(),
            report_a: format!("{left:.1}"),
            report_b: format!("{right:.1}"),
        });
    }

    diffs
}

pub fn print_comparison(a: &Report, b: &Report, diffs: &[Difference]) {
    println!("=== Parity Comparison ===");
    println!("  A: {} via {}", a.meta.vehicle_id, a.meta.transport);
    println!("  B: {} via {}", b.meta.vehicle_id, b.meta.transport);
    println!();

    for diff in diffs {
        let label = match diff.severity {
            Severity::Critical => "CRITICAL".red().bold(),
            Severity::Warning => "WARNING".yellow().bold(),
            Severity::Info => "INFO".blue().bold(),
        };
        println!(
            "  [{label}] {}: {} vs {}",
            diff.field, diff.report_a, diff.report_b
        );
    }

    let critical = diffs
        .iter()
        .filter(|diff| diff.severity == Severity::Critical)
        .count();
    let warning = diffs
        .iter()
        .filter(|diff| diff.severity == Severity::Warning)
        .count();
    let info = diffs
        .iter()
        .filter(|diff| diff.severity == Severity::Info)
        .count();

    println!();
    println!("  Critical: {critical} | Warning: {warning} | Info: {info}");
    if parity_ok(diffs) {
        println!("  {}", "PARITY OK".green().bold());
    } else {
        println!("  {}", "PARITY FAILED".red().bold());
    }
}

pub fn parity_ok(diffs: &[Difference]) -> bool {
    !diffs.iter().any(|diff| diff.severity == Severity::Critical)
}

fn test_detail_string(report: &Report, group: &str, field: &str) -> Option<String> {
    report
        .tests
        .get(group)?
        .details
        .as_ref()?
        .get(field)?
        .as_str()
        .map(ToOwned::to_owned)
}

fn test_detail_f64(report: &Report, group: &str, field: &str) -> Option<f64> {
    report
        .tests
        .get(group)?
        .details
        .as_ref()?
        .get(field)?
        .as_f64()
}

fn test_detail_array(report: &Report, group: &str, field: &str) -> Option<Vec<String>> {
    let array = report
        .tests
        .get(group)?
        .details
        .as_ref()?
        .get(field)?
        .as_array()?;
    Some(
        array
            .iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect(),
    )
}

fn test_detail_pid_values(report: &Report, group: &str, field: &str) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    let Some(object) = report
        .tests
        .get(group)
        .and_then(|result| result.details.as_ref())
        .and_then(|details| details.get(field))
        .and_then(|value| value.as_object())
    else {
        return out;
    };

    for (name, value) in object {
        if let Some(scalar) = value.get("value").and_then(|value| value.as_f64()) {
            out.insert(name.clone(), scalar);
        }
    }

    out
}

fn pid_tolerance(name: &str) -> f64 {
    match name {
        "Engine RPM" => 400.0,
        "Coolant Temperature" => 10.0,
        "Vehicle Speed" => 5.0,
        "Engine Load" => 15.0,
        "Throttle Position" => 15.0,
        _ => 5.0,
    }
}

fn format_set(values: &BTreeSet<String>) -> String {
    values.iter().cloned().collect::<Vec<_>>().join(",")
}
