Phase: E (Steps 95–114)
Status: DONE
Owner: codex
Date: 2026-03-29
Commands run:
- cargo fmt --all --check --manifest-path TASC/asc-standard/sdk/Cargo.toml
- cargo clippy --manifest-path TASC/asc-standard/sdk/Cargo.toml -- -D warnings
- cargo test --manifest-path TASC/asc-standard/sdk/Cargo.toml
Result summary:
- Wrapper/SDK/CLI now support trust-mode/profile-bundle passthrough plus support-bundle output/redaction.
Notes:
- Steps 102/104/105/109/110 unblocked.
