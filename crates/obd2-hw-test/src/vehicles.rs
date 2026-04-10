use obd2_core::protocol::pid::Pid;
use obd2_core::vehicle::{KLineInit, Protocol};

#[derive(Debug, Clone, Copy)]
pub enum VinExpectation {
    Unknown,
    #[allow(dead_code)]
    Exact(&'static str),
}

impl VinExpectation {
    pub fn describe(&self) -> &'static str {
        match self {
            Self::Unknown => "bootstrap-discovery",
            Self::Exact(_) => "exact",
        }
    }

    pub fn matches(&self, vin: &str) -> bool {
        match self {
            Self::Unknown => vin.len() == 17,
            Self::Exact(expected) => expected.eq_ignore_ascii_case(vin),
        }
    }

    pub fn expected(&self) -> Option<&'static str> {
        match self {
            Self::Unknown => None,
            Self::Exact(vin) => Some(vin),
        }
    }
}

pub struct ExpectedVehicle {
    pub id: &'static str,
    pub display_name: &'static str,
    pub vin: VinExpectation,
    pub expected_protocol: Protocol,
    pub expected_make: &'static str,
    pub required_pids: &'static [Pid],
    pub has_j1939: bool,
    pub has_spec_match: bool,
    pub enhanced_module: &'static str,
    pub plausible_rpm_range: (f64, f64),
    pub plausible_coolant_range: (f64, f64),
}

impl ExpectedVehicle {
    pub fn plausible_range(&self, pid: Pid) -> Option<(f64, f64)> {
        if pid == Pid::ENGINE_RPM {
            Some(self.plausible_rpm_range)
        } else if pid == Pid::COOLANT_TEMP {
            Some(self.plausible_coolant_range)
        } else if pid == Pid::VEHICLE_SPEED {
            Some((0.0, 200.0))
        } else if pid == Pid::ENGINE_LOAD || pid == Pid::THROTTLE_POSITION {
            Some((0.0, 100.0))
        } else if pid == Pid::CONTROL_MODULE_VOLTAGE {
            Some((9.0, 16.0))
        } else {
            None
        }
    }
}

pub static VEHICLES: &[ExpectedVehicle] = &[
    ExpectedVehicle {
        id: "duramax-2006",
        display_name: "2006 Chevy Duramax 2500",
        vin: VinExpectation::Unknown,
        expected_protocol: Protocol::J1850Vpw,
        expected_make: "Chevrolet",
        required_pids: &[
            Pid::ENGINE_RPM,
            Pid::COOLANT_TEMP,
            Pid::VEHICLE_SPEED,
            Pid::ENGINE_LOAD,
        ],
        has_j1939: true,
        has_spec_match: true,
        enhanced_module: "ecm",
        plausible_rpm_range: (0.0, 3500.0),
        plausible_coolant_range: (-40.0, 120.0),
    },
    ExpectedVehicle {
        id: "malibu-2020",
        display_name: "2020 Chevy Malibu",
        vin: VinExpectation::Unknown,
        expected_protocol: Protocol::Can11Bit500,
        expected_make: "Chevrolet",
        required_pids: &[
            Pid::ENGINE_RPM,
            Pid::COOLANT_TEMP,
            Pid::VEHICLE_SPEED,
            Pid::ENGINE_LOAD,
            Pid::THROTTLE_POSITION,
        ],
        has_j1939: false,
        has_spec_match: false,
        enhanced_module: "ecm",
        plausible_rpm_range: (0.0, 7000.0),
        plausible_coolant_range: (-40.0, 120.0),
    },
    ExpectedVehicle {
        id: "accord-2001",
        display_name: "2001 Honda Accord",
        vin: VinExpectation::Unknown,
        expected_protocol: Protocol::Iso9141(KLineInit::SlowInit),
        expected_make: "Honda",
        required_pids: &[Pid::ENGINE_RPM, Pid::COOLANT_TEMP, Pid::VEHICLE_SPEED],
        has_j1939: false,
        has_spec_match: false,
        enhanced_module: "ecm",
        plausible_rpm_range: (0.0, 7000.0),
        plausible_coolant_range: (-40.0, 120.0),
    },
];

pub fn find_vehicle(id: &str) -> Option<&'static ExpectedVehicle> {
    VEHICLES.iter().find(|vehicle| vehicle.id == id)
}

pub fn list_vehicles() {
    println!("Known vehicles:\n");
    for vehicle in VEHICLES {
        println!(
            "  {:<16} {} (protocol: {:?}, J1939: {}, spec: {}, VIN: {})",
            vehicle.id,
            vehicle.display_name,
            vehicle.expected_protocol,
            vehicle.has_j1939,
            vehicle.has_spec_match,
            vehicle.vin.describe(),
        );
    }
}
