use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Allow,
    Clamp,
    Hold,
    Override,
    Shutdown,
}

impl Verdict {
    pub fn precedence(self) -> u8 {
        match self {
            Self::Allow => 0,
            Self::Clamp => 1,
            Self::Hold => 2,
            Self::Override => 3,
            Self::Shutdown => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Verdict;

    #[test]
    fn precedence_is_monotonic_with_safety_severity() {
        assert!(Verdict::Allow.precedence() < Verdict::Clamp.precedence());
        assert!(Verdict::Clamp.precedence() < Verdict::Hold.precedence());
        assert!(Verdict::Hold.precedence() < Verdict::Override.precedence());
        assert!(Verdict::Override.precedence() < Verdict::Shutdown.precedence());
    }
}
