"""Current broad attempt and wall-clock deadline calculations."""

import time

from .error import RrdClientError


def attempt_limit(
    method: str,
    mutation: bool,
    idempotency_key: str | None,
    maximum: int,
) -> int:
    retry_safe = method == "GET" or not mutation or bool(idempotency_key)
    return maximum if retry_safe else 1


def remaining_timeout(configured: float, deadline_unix_ms: int | None) -> float:
    if deadline_unix_ms is None:
        return configured
    remaining = deadline_unix_ms / 1000 - time.time()
    if remaining <= 0:
        raise RrdClientError("RRD request deadline has expired")
    return min(configured, remaining)
