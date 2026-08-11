#!/usr/bin/env python3
from __future__ import annotations

import json
import sys


def main() -> int:
    payload = json.load(sys.stdin)
    print(f"# TASC Summary\n")
    print(f"- Result: `{payload['result']}`")
    print(f"- Profile bundle: `{payload['profileBundle']['id']}`")
    print(f"- Trust mode: `{payload['trustMode']}`")
    print(f"- Failed checks: `{payload['summary']['failedChecks']}`")
    if payload["failures"]:
        print("\n## Failures\n")
        for failure in payload["failures"]:
            print(f"- `{failure['code']}`: {failure['message']}")
    else:
        print("\nNo failures.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
