use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub meta: ReportMeta,
    pub summary: ReportSummary,
    pub tests: BTreeMap<String, TestGroupResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fatal_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMeta {
    pub timestamp: String,
    pub harness_version: String,
    pub vehicle_id: String,
    pub transport: String,
    pub port: Option<String>,
    pub adapter_chipset: Option<String>,
    pub adapter_firmware: Option<String>,
    pub protocol_detected: Option<String>,
    pub raw_capture_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGroupResult {
    pub status: TestStatus,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Pass,
    Fail,
    Skipped,
}

impl Report {
    pub fn new(meta: ReportMeta) -> Self {
        Self {
            meta,
            summary: ReportSummary::default(),
            tests: BTreeMap::new(),
            fatal_error: None,
        }
    }

    pub fn compute_summary(&mut self) {
        let mut summary = ReportSummary {
            duration_secs: self.summary.duration_secs,
            ..ReportSummary::default()
        };

        for result in self.tests.values() {
            summary.total += 1;
            match result.status {
                TestStatus::Pass => summary.passed += 1,
                TestStatus::Fail => summary.failed += 1,
                TestStatus::Skipped => summary.skipped += 1,
            }
        }

        self.summary = summary;
    }

    pub fn write_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}
