"""Current public session shape pending the H-04 credential redesign."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True, slots=True)
class Session:
    principal_id: str
    lease: dict[str, Any]
