# Minimal Signed

This example keeps the same wrapper flow but moves to a signed assurance-pack source.

Use it for:
- staging-oriented `doctor`
- report normalization against a signed profile report
- support-bundle generation for a signed path

Commands:

```bash
./tasc doctor --operation pack --profile-bundle staging-signed@1.0.0
./tasc verify examples/minimal-signed --profile-bundle staging-signed@1.0.0
./tasc explain last
```

What not to read yet:
- release day
- governance change control
- full runtime E2E fixtures

Next safe step:
- move to `prod-hsm@1.0.0` only after external HSM and transparency services are real.
