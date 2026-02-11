#!/usr/bin/env python3
"""Reference default-closed interlock gate."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass
class InterlockDecision:
    allowed: bool
    reason: str
    latched: bool

    def as_dict(self) -> dict[str, Any]:
        return {"allowed": self.allowed, "reason": self.reason, "latched": self.latched}


class InterlockGate:
    def __init__(self, min_heartbeat_hz: float = 10.0) -> None:
        self.min_heartbeat_hz = min_heartbeat_hz
        self._latched = False

    @property
    def latched(self) -> bool:
        return self._latched

    def reset_latch(self) -> None:
        self._latched = False

    def evaluate(
        self,
        *,
        authority_source: str,
        heartbeat_hz: float,
        within_safety_envelope: bool,
        interlock_enabled: bool,
    ) -> InterlockDecision:
        if self._latched:
            return InterlockDecision(False, "latched_shutdown", True)

        if authority_source != "safety_kernel":
            self._latched = True
            return InterlockDecision(False, "authority_exclusivity_violation", True)

        if not interlock_enabled:
            self._latched = True
            return InterlockDecision(False, "interlock_disabled", True)

        if heartbeat_hz < self.min_heartbeat_hz:
            self._latched = True
            return InterlockDecision(False, "heartbeat_below_threshold", True)

        if not within_safety_envelope:
            return InterlockDecision(False, "outside_safety_envelope", False)

        return InterlockDecision(True, "authorized", False)
