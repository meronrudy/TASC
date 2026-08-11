# Upgrade and Rollback

Adoption hardening requires rollback, not just forward migration.

## Commands

```bash
./tasc diff old.json new.json
./tasc downgrade --to dev-local@1.0.0 --write-lock
./tasc upgrade-simulate --to staging-signed@1.0.0 --format json
./tasc upgrade --to staging-signed@1.0.0 --dry-run
./tasc upgrade --to staging-signed@1.0.0 --write-lock
./tasc lock
./tasc check-lock --format json
```

## Policy

- pin CLI version in CI
- pin profile bundle version in CI
- pin schema, API, and contract versions in `tasc.yaml`
- commit `tasc.lock.yaml` and enforce it in CI
- simulate upgrades before mutating release-bound artifacts
- keep downgrade paths explicit where older profile bundle manifests still exist

## Current Scope

`./tasc downgrade` repins the workspace to another available profile bundle manifest.
`./tasc upgrade-simulate` previews contract-surface changes before mutating config.
`./tasc upgrade --dry-run` previews the post-upgrade `tasc.yaml`.
`./tasc upgrade` mutates `tasc.yaml` to the target bundle and can refresh `tasc.lock.yaml`.
`./tasc lock` writes `tasc.lock.yaml` from resolved settings.
`./tasc check-lock` fails on lockfile drift between `tasc.yaml` and `tasc.lock.yaml`.

Future work:
- report migration
- reversible pack migration
- downgrade simulation before mutation
