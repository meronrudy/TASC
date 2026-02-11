#!/usr/bin/env bash
set -euo pipefail

resolve_owner_repo() {
  if [[ -n "${GITHUB_OWNER:-}" && -n "${GITHUB_REPO:-}" ]]; then
    return 0
  fi

  local remote
  remote="$(git remote get-url origin 2>/dev/null || true)"
  if [[ -z "${remote}" ]]; then
    return 1
  fi
  if [[ "${remote}" =~ github.com[:/]([^/]+)/([^/.]+)(\.git)?$ ]]; then
    export GITHUB_OWNER="${GITHUB_OWNER:-${BASH_REMATCH[1]}}"
    export GITHUB_REPO="${GITHUB_REPO:-${BASH_REMATCH[2]}}"
    return 0
  fi
  return 1
}

resolve_token() {
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    return 0
  fi
  local token
  token="$(printf 'protocol=https\nhost=github.com\n\n' | git credential fill 2>/dev/null | awk -F= '/^password=/{print $2}')"
  if [[ -n "${token}" ]]; then
    export GITHUB_TOKEN="${token}"
    return 0
  fi
  return 1
}

if ! resolve_owner_repo; then
  echo "unable to resolve GITHUB_OWNER/GITHUB_REPO; set env vars explicitly" >&2
  exit 1
fi

if ! resolve_token; then
  echo "unable to resolve GITHUB_TOKEN from env or git credential helper" >&2
  exit 1
fi

branch="${1:-main}"

api="https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/branches/${branch}/protection"

payload="$(cat <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "ci / checks",
      "conformance / conformance",
      "kernel-ci / kernel",
      "release / package-evidence"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false
}
JSON
)"

status_code="$(
curl -sS -o /tmp/branch-protection-response.json -w "%{http_code}" -X PUT \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${GITHUB_TOKEN}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "${api}" \
  -d "${payload}"
)"

if [[ "${status_code}" != "200" ]]; then
  cat /tmp/branch-protection-response.json >&2
  echo "failed to apply branch protection (HTTP ${status_code})" >&2
  exit 1
fi

applied_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

python3 - <<PY
from pathlib import Path
import yaml

status_path = Path("governance/branch-protection.status.yaml")
doc = yaml.safe_load(status_path.read_text(encoding="utf-8"))
doc["branch"] = "${branch}"
doc["requiredChecks"] = ["ci", "conformance", "kernel-ci", "release"]
doc["applied"] = True
doc["appliedAtUtc"] = "${applied_at}"
doc["lastAttemptAtUtc"] = "${applied_at}"
doc["status"] = "applied"
doc["notes"] = "Applied via tools/governance/apply_branch_protection.sh for ${GITHUB_OWNER}/${GITHUB_REPO}:${branch}."
status_path.write_text(yaml.safe_dump(doc, sort_keys=False), encoding="utf-8")
PY

echo
echo "Applied branch protection for ${GITHUB_OWNER}/${GITHUB_REPO}:${branch} (HTTP ${status_code})"
