# Branch Protection Requirements

## Required Branch

- `main`

## Required Status Checks

- `ci / checks`
- `conformance / conformance`
- `kernel-ci / kernel`
- `release / package-evidence`

## Required Pull Request Rules

- Require pull request before merge.
- Require at least 1 approval.
- Dismiss stale approvals on new commits.
- Require all conversations resolved before merge.
- Require linear history.
- Restrict force pushes and deletions.

## Application Method

Apply host-side protection using `tools/governance/apply_branch_protection.sh` with:

- `GITHUB_OWNER`
- `GITHUB_REPO`
- `GITHUB_TOKEN`

If env vars are not provided, the script attempts to resolve owner/repo from `origin`
and token from the local git credential helper.

The script updates `governance/branch-protection.status.yaml` on success.
