# Reddit Pain Research Snapshot

Generated: `2026-04-01T03:10:08.953026+00:00`

Refresh command:

```bash
python3 tools/adoption/reddit_research.py
```

Collection mode:

- Discovery uses Reddit HTML search pages instead of JSON search APIs.
- Evidence rows are collected from Reddit comment partials at `/svc/shreddit/comments/...`.
- Every exported row keeps the direct Reddit comment permalink plus the exact fetch URL used to collect it.
- Rows are emitted only when aerospace terms, developer-surface terms, and pain terms all match.

## Coverage

| Metric | Value |
| --- | --- |
| Successful queries | `14` |
| Search results inspected | `81` |
| Seed threads | `8` |
| Threads fetched | `79` |
| Unique threads exported | `21` |
| Quote rows exported | `40` |
| Skipped query failures | `0` |
| Skipped fetch failures | `0` |

## Top Subreddits

| Subreddit | Quote Rows |
| --- | --- |
| `r/embedded` | `19` |
| `r/avionics` | `13` |
| `r/AerospaceEngineering` | `8` |

## Theme Summary

| Theme | Quote Rows | Suggested Ergonomics Surface | Example |
| --- | --- | --- | --- |
| `Certification Overhead` | `31` | evidence generation, guided workflows, reusable templates | To understand MISRA-C (or JSF-AV-C++) you need to first understand C/C++, Memory Management, Debugging, Test Driven D... |
| `Onboarding And Domain Knowledge Gap` | `30` | 60-second first win, demo fixtures, guided examples | You will need to understand things like requirements, traceability, and verification. |
| `Legacy And Change Constraints` | `20` | additive workflows, diff-safe outputs, migration guardrails | To understand MISRA-C (or JSF-AV-C++) you need to first understand C/C++, Memory Management, Debugging, Test Driven D... |
| `Toolchain Fragility` | `13` | doctor checks, wrapper-backed commands, environment diagnostics | The problem is that many companies/departments are very entrenched in their old tools, so it's difficult to make the... |
| `Requirements And Traceability Burden` | `11` | requirement linkage, stable schemas, machine-readable reports | You will see where the difficulty is in Aero-software development yourself… Edit: as for certification, AFuzion, but... |
| `Debugging And Observability Friction` | `8` | explainability, support bundles, deterministic failure output | To understand MISRA-C (or JSF-AV-C++) you need to first understand C/C++, Memory Management, Debugging, Test Driven D... |
| `Hardware And Lab Dependency` | `4` | offline-safe validation, simulation fixtures, sandboxed trust paths | I've never done an expensive training (although I may get to this summer!), but I have been the accountable person fo... |

## Query Inventory

| Query ID | Subreddit | Query | Search Results |
| --- | --- | --- | --- |
| `embedded_do178` | `embedded` | `DO-178` | `7` |
| `embedded_avionics` | `embedded` | `avionics software` | `7` |
| `embedded_do331` | `embedded` | `DO-331` | `7` |
| `embedded_arp4754` | `embedded` | `ARP4754` | `7` |
| `embedded_requirements_validation` | `embedded` | `requirements validation reporting` | `7` |
| `avionics_software` | `avionics` | `avionics software development` | `7` |
| `avionics_do178` | `avionics` | `DO-178` | `7` |
| `avionics_do254` | `avionics` | `DO-254` | `7` |
| `avionics_flight_software` | `avionics` | `flight software` | `7` |
| `aerospace_do178` | `aerospaceengineering` | `DO-178` | `7` |
| `aerospace_avionics_software` | `aerospaceengineering` | `avionics software` | `7` |
| `aerospace_arp4754` | `aerospaceengineering` | `ARP4754` | `3` |
| `global_flight_software` | `all` | `flight software certification embedded` | `0` |
| `global_arp4754_certification` | `all` | `ARP4754 software certification` | `1` |
