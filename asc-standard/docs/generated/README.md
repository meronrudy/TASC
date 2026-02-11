# Generated Documentation Scope

This directory is reserved for generated documentation artifacts.

## Policy

- Source-of-truth docs live under `docs/handbook/`, `docs/tutorials/`, and `docs/runbooks/`.
- Generated docs are optional and non-blocking for GA unless explicitly promoted by governance policy.
- Generated docs must not overwrite source-of-truth hand-authored files.

## Churn Control

To reduce report/document churn in commits:

- Commit only canonical release artifacts needed for verification or traceability.
- Avoid committing ad-hoc generated report variants unless they are referenced by release criteria.
- Prefer stable filenames for required generated docs.
