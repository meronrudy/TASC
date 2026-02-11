# Deprecation Policy

## Baseline

- Deprecations are announced with:
- affected interfaces
- replacement guidance
- target removal release
- impact on verifier and conformance vectors

## Support Window

- Deprecated schema fields and CLI flags remain supported for one minor release minimum.
- Early removal is allowed only for documented safety/security risk and Class A approval.

## Required Deprecation Artifacts

- changelog entry with migration path
- updates to `docs/handbook/*` and `docs/tutorials/*`
- compatibility tests proving old and new formats are handled during window

## Removal Criteria

- all required downstream artifacts updated
- no open P0/P1 incidents linked to migration
- governance sign-off recorded
