use asc_types::{model::CheckOutcome, Verdict};

pub fn decide(outcomes: &[CheckOutcome]) -> Verdict {
    outcomes
        .iter()
        .map(|o| o.verdict)
        .max_by_key(|v| v.precedence())
        .unwrap_or(Verdict::Allow)
}
