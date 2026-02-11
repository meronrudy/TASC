# GA Release Criteria

## Scope Lock
- ASC core naming remains unchanged.
- TASC layer remains additive.
- Required profiles: `uas-small`, `fixed-wing`, `hybrid-vtol`.
- Required policy: `eu-north-star`.
- Required trust floor: `TA2`.
- Required transparency logs: `rekor`, `mirror`.

## Must-Pass Matrix
| Gate | Requirement | Owner | Evidence |
| --- | --- | --- | --- |
| Kernel | All workspace tests pass and replay determinism checks pass | Kernel WG | `cargo test --manifest-path reference/kernel/Cargo.toml --workspace` |
| Reference surface | Adapter/interlock/supervisor integration tests pass | Systems WG | `python3 -m unittest reference.tests.test_reference_surface` |
| Conformance | All three profiles pass full check set | Assurance WG | `conformance/reports/tasc-*.json` |
| Negative coverage | Each check ID has at least one observed failing vector | Assurance WG | `conformance/reports/tasc-negative-vectors.json` |
| Trust | TA2 attestation chain/signature/revocation policy checks pass | Security WG | Conformance `CHK_ATTESTATION_TA2` |
| Transparency | Rekor + mirror proofs verify with freshness and binding | Security WG | Conformance `CHK_TRANSPARENCY_*` |
| Provenance | Lineage and freshness checks pass in releasepack preflight | Release Manager | `evidence/manifests/releasepack.json` |
| Governance | Class A policy and required records present for safety-critical changes | Governance Board | CI policy gate report |
| Docs | End-to-end release walkthrough and runbooks complete | Program Office | Handbook + runbooks |

## Class A Freeze
- `spec/tasc/checks.yaml` IDs are immutable for GA train.
- Schema major/minor changes require Class A review and explicit migration note.

## Sign-Off
- Program Manager:
- Safety Lead:
- Security Lead:
- Release Manager:
- Date:
