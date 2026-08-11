# Rust SDK Contract (Phase 1)

## Baseline
- Rust edition: 2021 for all SDK crates.
- MSRV: 1.70.0.
- Workspace crates: `tasc-sdk`, `tasc-sdk-types`, `tasc-sdk-cli`.
- API stability: Phase 1 public APIs are stable within minor releases; breaking
  changes require a major version bump.

## Output formats
`OutputFormat` values are fixed to: `human`, `json`, `sarif`.

## Public API (SdkClient)
`SdkClient` wraps the repo-local `tasc` wrapper. Methods accept request structs
from `tasc-sdk-types` and return typed responses:
- `doctor(DoctorRequest) -> DoctorResponse`
- `verify(VerifyRequest) -> VerifyResponse`
- `explain(ExplainRequest) -> ExplainResponse`
- `support_bundle(SupportBundleRequest) -> SupportBundleResponse`
- `ci_preflight(CiPreflightRequest) -> CiPreflightResponse`
- `demo(DemoRequest) -> DemoResponse`
- `diagnostics() -> DiagnosticReport`

Trust-mode expectations:
- `trust_mode` is passed through to the wrapper for `verify`, `ci-preflight`, and `demo`.
- Demo flows target `examples/minimal-local` and are not production trust.

## Request/Response Schemas
All request/response structs are serialized with `serde` using default field
names (snake_case). Types live in `tasc-sdk-types`:

Requests:
- `DoctorRequest { operation, profile_bundle, trust_mode, format, timeout_ms }`
- `VerifyRequest { target_path, profile_bundle, trust_mode, profile, format, timeout_ms }`
- `ExplainRequest { report_path, format, renderer, timeout_ms }`
- `SupportBundleRequest { target_path, archive, output, redact, timeout_ms }`
- `CiPreflightRequest { examples, format, strict, output, profile_bundle, trust_mode, timeout_ms }`
- `DemoRequest { format, profile_bundle, trust_mode, timeout_ms }`

Responses:
- `DoctorResponse { result, output, json }`
- `VerifyResponse { status, verdict, report_path, output, json }`
- `ExplainResponse { result, output, json }`
- `SupportBundleResponse { bundle_path, output }`
- `CiPreflightResponse { result, output, json }`
- `DemoResponse { verify, explain }`

## Error Model and Exit Codes
`SdkErrorCode` values are fixed to:
`io`, `repo_not_found`, `tasc_not_found`, `invalid_config`, `command_failed`,
`timeout`, `parse_error`, `canonicalization`, `unsupported_format`, `unknown`.

CLI exit codes map from `SdkError` (via `tasc_sdk::exit_code_for`):
- `2`: config/repo/tasc resolution errors
- `3`: command failures, timeouts, parse/canonicalization/format errors
- `4`: I/O errors
- `1`: not-implemented or unknown

## Command execution rule
Use `std::process::Command` with explicit argument vectors only. No shell
interpolation is permitted.

## Path resolution
Repo root resolution walks ancestors to find a directory containing both
`tasc` and `tasc.yaml`. The SDK also recognizes nested repo roots at
`TASC/asc-standard` for convenience. The `tasc` binary can be overridden by an
explicit path; missing or non-file overrides return `tasc_not_found`.

## Timeout behavior
Default is no timeout unless configured. Per-request `timeout_ms` overrides
`SdkConfig.timeout_ms` (all values are milliseconds).

## Additive library modules
The SDK also exposes additive modules that do not change wrapper behavior:
- `artifact_templates`: deterministic instant artifact constructors and writers.
- `mock_data`: deterministic fixture bundle generation and serialization.
- `diagnostics`: SDK-internal environment report (`run`, `is_healthy`).
- `reports`: `.tasc` report/explain discovery, loading, and summarization.
