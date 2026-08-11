Step ID: Step 104
Status: DONE
Owner: codex
Date: 2026-03-29
Files touched:
- TASC/asc-standard/tasc
- TASC/asc-standard/sdk/tasc-sdk/src/commands/support_bundle.rs
- TASC/asc-standard/sdk/tasc-sdk-types/src/request.rs
- TASC/asc-standard/sdk/tasc-sdk-cli/src/args.rs
- TASC/asc-standard/sdk/tasc-sdk-cli/src/main.rs
- TASC/asc-standard/sdk/tasc-sdk/tests/support_bundle.rs
Commands run:
- cargo fmt --all --check --manifest-path TASC/asc-standard/sdk/Cargo.toml
- cargo clippy --manifest-path TASC/asc-standard/sdk/Cargo.toml -- -D warnings
- cargo test --manifest-path TASC/asc-standard/sdk/Cargo.toml
Result summary:
- Support bundle now supports --output and --redact, with wrapper-side redaction support.
Follow-up:
- N/A
