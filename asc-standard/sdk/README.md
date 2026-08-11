# TASC Rust SDK Workspace

This workspace hosts Rust crates that wrap the `tasc` CLI and provide typed
request/response models.

## Workspace layout
- `tasc-sdk-types`: shared request/response types and error codes.
- `tasc-sdk`: client that executes `tasc` safely and parses outputs.
- `tasc-sdk-cli`: CLI entrypoint mapped to `tasc-sdk` methods.

## Build
MSRV: 1.70.0

```bash
cd TASC/asc-standard/sdk
cargo build
cargo test
```

## Usage
### `tasc-sdk-types`
Use `tasc-sdk-types` for `OutputFormat`, `SdkError`, and the request/response
structs shared across the SDK and CLI.

### `tasc-sdk`
Use `tasc-sdk::SdkClient` to call `doctor`, `verify`, `explain`,
`support_bundle`, `ci_preflight`, and `demo` against the `tasc` wrapper.
The crate also exposes additive library modules:
- `artifact_templates` for deterministic instant artifacts.
- `mock_data` for deterministic fixture bundles.
- `diagnostics` for SDK-internal environment checks.
- `reports` for `.tasc` report location/loading/summarization.

### `tasc-sdk-cli`
Binary crate that maps CLI subcommands to `tasc-sdk` methods.
