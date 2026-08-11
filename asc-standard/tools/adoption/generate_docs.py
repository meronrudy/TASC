#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
GENERATED = REPO_ROOT / "docs/generated"


def load_yaml(path: Path):
    with path.open("r", encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def write(path: Path, content: str) -> None:
    path.write_text(content.rstrip() + "\n", encoding="utf-8")


def render_public_surface() -> str:
    api = load_yaml(REPO_ROOT / "spec/interfaces/api.openapi.yaml")
    contract = load_yaml(REPO_ROOT / "reference/contracts/interfaces.v1.yaml")
    lines = [
        "# Public Surface",
        "",
        "Canonical external boundary:",
        "",
        "- `spec/interfaces/api.openapi.yaml`",
        "- `reference/contracts/interfaces.v1.yaml`",
        "",
        f"- API version: `{api['info']['version']}`",
        f"- Contract version: `{contract['version']}`",
        "",
        "## API Paths",
        "",
    ]
    for path_name in api.get("paths", {}).keys():
        lines.append(f"- `{path_name}`")
    lines.extend(
        [
            "",
            "## Contract Components",
            "",
        ]
    )
    for component in contract.get("components", {}).keys():
        lines.append(f"- `{component}`")
    return "\n".join(lines)


def render_profile_matrix() -> str:
    lines = [
        "# Profile Bundle Matrix",
        "",
        "| Bundle | Trust Mode | Offline Safe | Network Required | HSM Required | Schema | API | Contract |",
        "| --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for path in sorted((REPO_ROOT / "profile-bundles").glob("*.yaml")):
        bundle = load_yaml(path)
        compat = bundle["compatibility"]
        caps = bundle["capabilities"]
        lines.append(
            f"| `{bundle['bundle_id']}` | `{bundle['trust_mode']}` | `{caps['offline_safe']}` | `{bundle['network_required']}` | `{caps['hsm_required']}` | `{compat['schema_version']}` | `{compat['api_version']}` | `{compat['contract_version']}` |"
        )
    return "\n".join(lines)


def render_example_gallery() -> str:
    lines = [
        "# Example Gallery",
        "",
        "| Example | Bundle | Trust Mode | Verify Input | Purpose |",
        "| --- | --- | --- | --- | --- |",
    ]
    for path in sorted((REPO_ROOT / "examples").iterdir()):
        if not path.is_dir():
            continue
        config = load_yaml(path / "tasc.yaml")
        purpose = (path / "README.md").read_text(encoding="utf-8").splitlines()[2]
        lines.append(
            f"| `{path.name}` | `{config['profile_bundle']}` | `{config['trust_mode']}` | `{config['verify']['input']}` | {purpose} |"
        )
    return "\n".join(lines)


def render_check_catalog() -> str:
    remediation = load_yaml(REPO_ROOT / "spec/tasc/remediation.yaml")
    lines = [
        "# Check Catalog",
        "",
        "| Check ID | Remediation |",
        "| --- | --- |",
    ]
    for check_id, text in remediation["checks"].items():
        lines.append(f"| `{check_id}` | {text} |")
    return "\n".join(lines)


def render_remediation_catalog() -> str:
    remediation = load_yaml(REPO_ROOT / "spec/tasc/remediation.yaml")
    lines = [
        "# Remediation Catalog",
        "",
        "Generated from `spec/tasc/remediation.yaml`.",
        "",
    ]
    for check_id, text in remediation["checks"].items():
        lines.append(f"## {check_id}")
        lines.append("")
        lines.append(text)
        lines.append("")
    return "\n".join(lines)


def render_compatibility_matrix() -> str:
    api = load_yaml(REPO_ROOT / "spec/interfaces/api.openapi.yaml")
    contract = load_yaml(REPO_ROOT / "reference/contracts/interfaces.v1.yaml")
    lines = [
        "# Compatibility Matrix",
        "",
        "| Surface | Version | Source |",
        "| --- | --- | --- |",
        f"| Schema | `0.3.0` | `tasc.yaml` / profile bundles |",
        f"| API | `{api['info']['version']}` | `spec/interfaces/api.openapi.yaml` |",
        f"| Contract | `{contract['version']}` | `reference/contracts/interfaces.v1.yaml` |",
    ]
    return "\n".join(lines)


def main() -> int:
    GENERATED.mkdir(parents=True, exist_ok=True)
    write(GENERATED / "PUBLIC_SURFACE.md", render_public_surface())
    write(GENERATED / "PROFILE_MATRIX.md", render_profile_matrix())
    write(GENERATED / "EXAMPLE_GALLERY.md", render_example_gallery())
    write(GENERATED / "CHECK_CATALOG.md", render_check_catalog())
    write(GENERATED / "REMEDIATION_CATALOG.md", render_remediation_catalog())
    write(GENERATED / "COMPATIBILITY_MATRIX.md", render_compatibility_matrix())
    index = {
        "generated": [
            "PUBLIC_SURFACE.md",
            "PROFILE_MATRIX.md",
            "EXAMPLE_GALLERY.md",
            "CHECK_CATALOG.md",
            "REMEDIATION_CATALOG.md",
            "COMPATIBILITY_MATRIX.md",
        ]
    }
    write(GENERATED / "index.json", json.dumps(index, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
