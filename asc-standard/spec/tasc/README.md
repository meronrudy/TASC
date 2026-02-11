# TASC Normative Specification

This directory defines the contract-enforced TASC assurance layer that sits above the ASC kernel.

## Normative files

- `checks.yaml` defines required verification checks and IDs.
- `policy-eu-north-star.yaml` defines baseline compliance defaults for procurement and underwriting use.
- `trust-anchor-levels.yaml` defines TA0/TA1/TA2 requirements and TA2 minimum controls.

Any implementation claiming TASC conformance MUST satisfy the required checks and policy constraints.
