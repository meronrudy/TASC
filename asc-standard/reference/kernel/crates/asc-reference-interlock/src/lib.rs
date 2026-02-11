use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterlockDecision {
    pub allowed: bool,
    pub reason: String,
    pub latched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterlockGate {
    pub min_heartbeat_hz: f64,
    pub latched: bool,
}

impl InterlockGate {
    pub fn new(min_heartbeat_hz: f64) -> Self {
        Self {
            min_heartbeat_hz,
            latched: false,
        }
    }

    pub fn reset_latch(&mut self) {
        self.latched = false;
    }

    pub fn evaluate(
        &mut self,
        authority_source: &str,
        heartbeat_hz: f64,
        within_safety_envelope: bool,
        interlock_enabled: bool,
    ) -> InterlockDecision {
        if self.latched {
            return InterlockDecision {
                allowed: false,
                reason: "latched_shutdown".to_string(),
                latched: true,
            };
        }

        if authority_source != "safety_kernel" {
            self.latched = true;
            return InterlockDecision {
                allowed: false,
                reason: "authority_exclusivity_violation".to_string(),
                latched: true,
            };
        }

        if !interlock_enabled {
            self.latched = true;
            return InterlockDecision {
                allowed: false,
                reason: "interlock_disabled".to_string(),
                latched: true,
            };
        }

        if heartbeat_hz < self.min_heartbeat_hz {
            self.latched = true;
            return InterlockDecision {
                allowed: false,
                reason: "heartbeat_below_threshold".to_string(),
                latched: true,
            };
        }

        if !within_safety_envelope {
            return InterlockDecision {
                allowed: false,
                reason: "outside_safety_envelope".to_string(),
                latched: false,
            };
        }

        InterlockDecision {
            allowed: true,
            reason: "authorized".to_string(),
            latched: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InterlockGate;

    #[test]
    fn interlock_latches_on_authority_violation() {
        let mut gate = InterlockGate::new(10.0);
        let first = gate.evaluate("pilot_override", 12.0, true, true);
        assert!(!first.allowed);
        assert!(first.latched);
        let second = gate.evaluate("safety_kernel", 12.0, true, true);
        assert!(!second.allowed);
        assert_eq!(second.reason, "latched_shutdown");
    }
}
