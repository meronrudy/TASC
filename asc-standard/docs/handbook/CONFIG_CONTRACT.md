# Configuration Contract

`tasc.yaml` is the canonical workspace configuration file.

Resolution order is fixed:

1. explicit CLI flag
2. `tasc.yaml`
3. workspace default
4. built-in dev default

## Minimum Shape

```yaml
profile_bundle: dev-local@1.0.0
schema_version: 0.3.0
api_version: 0.2.0
contract_version: 1.0.0
trust_mode: dev-local
pack_format_version: 0.1.0
verify:
  input_kind: report
  input: conformance/fixtures/tasc/tasc-conformance-uas-small.json
  output: .tasc/last-report.json
pack:
  profile: uas-small
  output_json: evidence/manifests/tasc-assurance-pack-dev.json
  output_archive: evidence/manifests/tasc-assurance-pack-dev.tgz
  badge_id: badge-dev-local-active
```

## Rules

- Pin one profile bundle at a time.
- Pin schema, API, and contract versions explicitly.
- Generate `tasc.lock.yaml` with `./tasc lock` for CI reproducibility.
- Keep output paths inside the workspace.
- `prod-hsm` must not reference repo-shipped demo keys.
