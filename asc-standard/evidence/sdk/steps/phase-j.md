Phase: J (Steps 176–195)
Status: DONE
Owner: codex
Date: 2026-03-23
Commands run:
- cargo fmt --all (in TASC/asc-standard/sdk)
- cargo clippy -- -D warnings (in TASC/asc-standard/sdk)
- TMPDIR=/Volumes/2.5SSDDD128/Run Orb Run/TASC/asc-standard/sdk/target/tmp cargo test --manifest-path TASC/asc-standard/sdk/Cargo.toml
Result summary:
- Sandbox/export/snippets/test_helpers implemented with tests; docs updated.
Notes:
- Sandbox tests are feature-gated; run with --features sandbox to exercise.
