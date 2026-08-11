# Plugins

This directory contains supported extension examples for the product-facing wrapper.

Current executable extension seam:

- report renderer plugins for `./tasc explain --renderer <path>`

Plugin contract:
- input arrives as normalized failure-schema JSON on stdin
- output is free-form text or machine-readable content written to stdout
- non-zero exit marks the plugin as failed

Reference example:
- `plugins/renderers/markdown_summary.py`
