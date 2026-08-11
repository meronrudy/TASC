# Failure Schema and Exit Codes

`./tasc explain` emits the stable failure contract.

## Failure Record

- `code`: stable machine code, for example `VERIFY.CHECK.CHK_ATTESTATION_TA2`
- `severity`: `error` or `warning`
- `path`: affected artifact or bundle identifier
- `ruleId`: originating verifier or policy rule
- `remediationId`: stable remediation lookup key
- `help.anchor`: local handbook anchor
- `exitCode`: deterministic CLI exit mapping

## Output Modes

- human
- JSON
- SARIF

## Exit Codes

- `0`: success
- `2`: doctor warning
- `3`: verification or explain failure
- `4`: usage error
- `5`: unsupported or blocked operation
