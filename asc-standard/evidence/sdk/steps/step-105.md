Step ID: Step 105
Status: DONE
Owner: codex
Date: 2026-03-29
Files touched:
- TASC/asc-standard/tasc
- TASC/asc-standard/sdk/tasc-sdk/src/commands/ci_preflight.rs
- TASC/asc-standard/sdk/tasc-sdk-types/src/request.rs
- TASC/asc-standard/sdk/tasc-sdk-cli/src/args.rs
- TASC/asc-standard/sdk/tasc-sdk-cli/src/main.rs
- TASC/asc-standard/sdk/tasc-sdk/tests/ci_preflight.rs
Commands run:
- cargo fmt --all --check --manifest-path TASC/asc-standard/sdk/Cargo.toml
- cargo clippy --manifest-path TASC/asc-standard/sdk/Cargo.toml -- -D warnings
- cargo test --manifest-path TASC/asc-standard/sdk/Cargo.toml
Result summary:
- Ci-preflight now accepts profile bundle and trust-mode overrides with consistent CLI/SDK mapping.
Follow-up:
- N/A
