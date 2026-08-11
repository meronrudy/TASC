# Generated Documentation Scope

This directory is reserved for generated product-facing documentation artifacts.

## Policy

- Source-of-truth docs live under `docs/handbook/`, `docs/tutorials/`, and `docs/runbooks/`.
- Generated docs are produced by `python3 tools/adoption/generate_docs.py`.
- Generated docs are optional and non-blocking for GA unless explicitly promoted by governance policy.
- Generated docs must not overwrite source-of-truth hand-authored files.

## Churn Control

To reduce report/document churn in commits:

- Commit only canonical release artifacts needed for verification or traceability.
- Avoid committing ad-hoc generated report variants unless they are referenced by release criteria.
- Prefer stable filenames for required generated docs.

## Current Outputs

- `PUBLIC_SURFACE.md`
- `PROFILE_MATRIX.md`
- `EXAMPLE_GALLERY.md`
- `CHECK_CATALOG.md`
- `REMEDIATION_CATALOG.md`
- `COMPATIBILITY_MATRIX.md`
