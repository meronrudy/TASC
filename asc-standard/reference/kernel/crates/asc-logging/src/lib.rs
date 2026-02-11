use asc_types::model::KernelOutput;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub seq: u64,
    pub payload: KernelOutput,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Debug, Default)]
pub struct EventLog {
    pub records: Vec<EventRecord>,
    pub tip_hash: String,
}

impl EventLog {
    pub fn append(&mut self, seq: u64, payload: &KernelOutput) {
        let prev_hash = self.tip_hash.clone();
        let material = event_material(seq, payload, &prev_hash);
        let mut hasher = Sha256::new();
        hasher.update(material.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.records.push(EventRecord {
            seq,
            payload: payload.clone(),
            prev_hash,
            hash: hash.clone(),
        });
        self.tip_hash = hash;
    }
}

fn event_material(seq: u64, payload: &KernelOutput, prev_hash: &str) -> String {
    format!(
        "{}|{:?}|{:?}|{:.17}|{:.17}|{:.17}|{:.17}|{}|{}|{}",
        seq,
        payload.verdict,
        payload.reasons,
        payload.command.applied_rates_dps[0],
        payload.command.applied_rates_dps[1],
        payload.command.applied_rates_dps[2],
        payload.command.applied_climb_mps,
        payload.command.shutdown,
        payload.contract_fingerprint,
        prev_hash
    )
}

#[cfg(test)]
mod tests {
    use super::EventLog;
    use asc_types::{model::ConstrainedCommand, model::KernelOutput, ReasonCode, Verdict};

    fn sample_output(applied: f64) -> KernelOutput {
        KernelOutput {
            verdict: Verdict::Clamp,
            reasons: vec![ReasonCode::FlowConstraintViolation],
            command: ConstrainedCommand {
                applied_rates_dps: [applied, 0.0, 0.0],
                applied_climb_mps: 0.0,
                shutdown: false,
            },
            contract_fingerprint: "fp-123".into(),
        }
    }

    #[test]
    fn append_is_deterministic_for_identical_payloads() {
        let mut left = EventLog::default();
        let mut right = EventLog::default();
        let payload = sample_output(1.25);

        left.append(1, &payload);
        right.append(1, &payload);

        assert_eq!(left.tip_hash, right.tip_hash);
        assert_eq!(left.records.len(), 1);
        assert_eq!(right.records.len(), 1);
    }

    #[test]
    fn append_handles_non_finite_command_values_without_panicking() {
        let mut log = EventLog::default();
        let payload = sample_output(f64::NAN);
        log.append(1, &payload);

        assert_eq!(log.records.len(), 1);
        assert!(!log.tip_hash.is_empty());
    }
}
