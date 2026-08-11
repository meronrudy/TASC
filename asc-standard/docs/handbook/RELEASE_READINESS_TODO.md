# Release-Readiness TODO (Core Kernel Outward)

## 1. Core Kernel and Contracts
- [x] Expand ASC spec files with normative intent/version/reference fields.
- [x] Expand glossary with ASC/TASC runtime and evidence terms.
- [x] Add stricter runtime checks for finite state values and position bounds.
- [x] Add negative-path kernel conformance tests.
- [x] Add explicit deterministic replay fixture validation beyond metadata-level checks.

## 2. TASC Verifier and Assurance Integrity
- [x] Remove synthetic placeholder conformance report generation from assurance pack builder.
- [x] Embed verifier-produced `conformanceReport` in generated assurance packs.
- [x] Fix verifier report serde contract (`CheckResult` deserialize support).
- [x] Replace synthetic log signature model with real trust-chain verification material.

## 3. Conformance Program Realization
- [x] Replace static conformance index with data-derived report aggregation.
- [x] Add suite definitions for `syntax`, `semantics`, `timing`, `replay`, `fault-injection`, `tasc`.
- [x] Add negative vector assets for each check ID in `spec/tasc/checks.yaml`.
- [x] Execute negative vectors in CI as a required gate (all profiles).

## 4. Evidence Pipeline and Manifest Quality
- [x] Add release guard that rejects placeholder/starter/scaffold text in release-bound assets.
- [x] Ensure releasepack validates PASS conformance and full required check coverage.
- [x] Add hashlock metadata (`sizeBytes`, `mtimeUtc`, `generatedAtUtc`).
- [x] Add provenance freshness policy with explicit maximum artifact age thresholds.

## 5. Safety Case and Compliance Maps
- [x] Replace compliance map stubs with mapping tables and trace IDs.
- [x] Add initial high-level, low-level, and GSN argument artifacts.
- [x] Expand theorem/assumptions index with bidirectional links to evidence manifests.

## 6. Governance and Program Controls
- [x] Add decision/proposal/meeting templates for governance artifacts.
- [x] Add explicit approver roles, escalation matrix, and exception SLA in governance policy docs.

## 7. Architecture Surface Outside Kernel
- [x] Add explicit scope/contract READMEs for `reference/adapters`, `reference/interlock`, `reference/supervisor`.
- [x] Implement runnable reference components with integration tests for mediation/exclusivity and incident-pack flow.

## 8. Data Taxonomy and Operational Inputs
- [x] Populate `data/incident-taxonomy`, `data/logging-schema`, `data/replay` with versioned artifacts.
- [x] Wire these data assets into schema compatibility CI checks.

## 9. Documentation and Onboarding
- [x] Add full walkthrough: spec change -> generation -> conformance -> assurance pack -> verifier -> release bundle.
- [x] Add troubleshooting matrix keyed by check ID with remediation actions.
- [x] Add wrapper-first operational preflight (`doctor`, `check-lock`, `check-example`) to release docs/runbooks.

## 10. Repo Hygiene and Release Gates
- [x] Add workflow-level hard gate for placeholder/starter/scaffold detection.
- [x] Remove or justify all release-domain `.gitkeep` directories with scope notes or real artifacts.
- [x] Configure branch protection outside repo to require `ci`, `conformance`, `kernel-ci`, and `release` checks.
- [x] Add examples smoke workflow gate for lock enforcement and golden example checks.
