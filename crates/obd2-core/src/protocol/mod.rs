//! OBD-II and J1939 protocol types and parsing.

pub mod pid;
pub mod dtc;
pub mod enhanced;
pub mod service;
pub mod codec;
pub mod j1939;

// Re-export key types
pub use pid::{Pid, ValueType};
pub use dtc::{Dtc, DtcCategory, DtcStatus, DtcStatusByte, Severity};
pub use enhanced::{EnhancedPid, Formula, Confidence};
pub use service::{
    DiagSession, ActuatorCommand, ReadinessStatus, MonitorStatus,
    TestResult, VehicleInfo, ServiceRequest, O2TestResult, O2SensorLocation,
};
pub use j1939::{
    Pgn, J1939Dtc,
    Eec1, Ccvs, Et1, Eflp1, Lfe,
    decode_eec1, decode_ccvs, decode_et1, decode_eflp1, decode_lfe, decode_dm1,
};
