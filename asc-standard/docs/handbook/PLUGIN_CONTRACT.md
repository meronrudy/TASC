# Plugin and Extension Contract

The repo now treats extensions as explicit contracts, not internal hook accidents.

## Supported Classes

- verifier check plugin
- report renderer plugin
- external evidence-source adapter
- policy bundle provider
- pack post-processor

## Required Contract Elements

- declared input/output schema
- lifecycle hook definition
- compatibility statement
- sandbox and security constraints
- upgrade policy

## Current Status

The contract is documented now so third parties do not bind to internal code paths.

Current executable seam:

- `./tasc explain --renderer plugins/renderers/markdown_summary.py`

Reference implementations remain inside the repo and are not yet a stable plugin ABI.
