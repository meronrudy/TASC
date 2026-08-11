use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};

use crate::SdkError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockScenario {
    FlightLog,
    MaintenanceLog,
    IncidentReplay,
}

impl MockScenario {
    fn slug(self) -> &'static str {
        match self {
            MockScenario::FlightLog => "flight_log",
            MockScenario::MaintenanceLog => "maintenance_log",
            MockScenario::IncidentReplay => "incident_replay",
        }
    }

    fn tag(self) -> u64 {
        match self {
            MockScenario::FlightLog => 0x101,
            MockScenario::MaintenanceLog => 0x202,
            MockScenario::IncidentReplay => 0x303,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MockDataRequest {
    pub scenario: MockScenario,
    pub seed: u64,
    pub records: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MockDataBundle {
    pub scenario: MockScenario,
    pub seed: u64,
    pub events_json: Value,
    pub summary_json: Value,
    pub bundle_bytes: Vec<u8>,
}

pub fn generate_bundle(request: &MockDataRequest) -> Result<MockDataBundle, SdkError> {
    if request.records == 0 {
        return Err(SdkError::InvalidConfig(
            "mock data request must contain at least one record".to_string(),
        ));
    }

    let mut rng = DeterministicRng::new(request.seed ^ request.scenario.tag());
    let mut events = Vec::with_capacity(request.records);

    for index in 0..request.records {
        let event = match request.scenario {
            MockScenario::FlightLog => json!({
                "index": index,
                "timestampMs": 1_700_000_000_000_u64 + index as u64 * 1000,
                "altitudeM": rng.next_range_u32(80, 420),
                "speedMps": rng.next_range_u32(10, 70),
                "latitude": rng.next_range_f64(36.0, 45.0),
                "longitude": rng.next_range_f64(-123.0, -70.0),
                "heartbeatOk": rng.next_bool(),
            }),
            MockScenario::MaintenanceLog => json!({
                "index": index,
                "component": format!("component-{}", rng.next_range_u32(1, 12)),
                "hoursSinceService": rng.next_range_u32(1, 500),
                "status": if rng.next_bool() { "PASS" } else { "WARN" },
                "action": if rng.next_bool() { "inspection" } else { "replacement" },
            }),
            MockScenario::IncidentReplay => json!({
                "index": index,
                "impactG": rng.next_range_f64(0.0, 78.0),
                "interlockTriggered": rng.next_bool(),
                "recoveryMs": rng.next_range_u32(40, 5000),
                "heartbeatLost": !rng.next_bool(),
            }),
        };
        events.push(event);
    }

    let events_json = Value::Array(events);
    let summary_json = json!({
        "scenario": request.scenario.slug(),
        "seed": request.seed,
        "records": request.records,
        "schemaVersion": "1.0.0",
    });

    let bundle_json = json!({
        "scenario": request.scenario.slug(),
        "seed": request.seed,
        "events": events_json,
        "summary": summary_json,
    });

    let bundle_bytes =
        serde_json::to_vec(&bundle_json).map_err(|err| SdkError::ParseError(err.to_string()))?;
    let events_json = bundle_json
        .get("events")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    let summary_json = bundle_json
        .get("summary")
        .cloned()
        .unwrap_or_else(|| json!({}));

    Ok(MockDataBundle {
        scenario: request.scenario,
        seed: request.seed,
        events_json,
        summary_json,
        bundle_bytes,
    })
}

pub fn write_bundle(path: &Path, bundle: &MockDataBundle) -> Result<(), SdkError> {
    let mut file = File::create(path)?;
    file.write_all(&bundle.bundle_bytes)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

pub fn to_verify_bytes(bundle: &MockDataBundle) -> Vec<u8> {
    bundle.bundle_bytes.clone()
}

#[derive(Debug, Clone)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    fn next_range_u32(&mut self, min: u32, max: u32) -> u32 {
        let span = max.saturating_sub(min).max(1);
        min + (self.next_u64() as u32 % (span + 1))
    }

    fn next_range_f64(&mut self, min: f64, max: f64) -> f64 {
        let unit = (self.next_u64() as f64) / (u64::MAX as f64);
        min + (max - min) * unit
    }
}
