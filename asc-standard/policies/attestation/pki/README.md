# PKI Fixtures

These files are repo-shipped demo fixtures.

Rules:
- They are non-production.
- `prod-hsm` must refuse them.
- `dev-local` may use them only for bootstrap and smoke paths.
- Any real production deployment must externalize trust roots, signer material, and HSM configuration.
