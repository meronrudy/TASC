# releasepack

Builds a tarball containing selected evidence artifacts plus a package manifest.

Default payload includes core traceability artifacts plus profile-specific TASC
assurance packs and conformance reports.

Before packaging, the tool hard-fails on:
- non-PASS conformance reports
- missing required check IDs
- freshness policy violations (`policies/provenance/freshness-policy.yaml`)
- lineage mismatches (`spec-hash -> conformance -> assurance -> hashlock`)

## Usage

```bash
python3 tools/releasepack/releasepack.py --repo-root .
```
