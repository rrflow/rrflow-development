"""Synchronous client construction, discovery, and generic HTTP dispatch."""

from __future__ import annotations

import json
from collections.abc import Mapping
from typing import Any
from urllib.parse import urljoin

import httpx

from .endpoint import loopback_url
from .error import RrdClientError
from .generated import ENDPOINTS, OperationId
from .operation import (
    RequestOptions,
    canonical_id,
    default_resource,
    request_context,
    resolve_path,
    resource_segments,
)
from .retry import attempt_limit, remaining_timeout
from .session import Session
from .transport import decode_response, read_response


class RrdClient:
    def __init__(
        self,
        base_url: str,
        instance: str,
        *,
        request_timeout: float = 5.0,
        max_attempts: int = 2,
        max_response_bytes: int = 4 * 1024 * 1024,
        transport: httpx.BaseTransport | None = None,
    ) -> None:
        self.base_url = loopback_url(base_url)
        self.instance = canonical_id(instance, "instance")
        if not 0 < request_timeout <= 300:
            raise RrdClientError("request timeout must be in (0, 300] seconds")
        if not 1 <= max_attempts <= 8:
            raise RrdClientError("max attempts must be in 1..=8")
        if not 1 <= max_response_bytes <= 16 * 1024 * 1024:
            raise RrdClientError("response limit must be in 1..=16777216 bytes")
        self.request_timeout = request_timeout
        self.max_attempts = max_attempts
        self.max_response_bytes = max_response_bytes
        self._http = httpx.Client(transport=transport, follow_redirects=False)

    def close(self) -> None:
        self._http.close()

    def __enter__(self) -> RrdClient:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def capabilities(self) -> dict[str, Any]:
        result = self.call("capabilities-read")
        instance = result.get("instance")
        if (
            result.get("protocol") != "rrd"
            or result.get("protocol_version") != 1
            or not isinstance(instance, dict)
            or instance.get("id") != self.instance
        ):
            raise RrdClientError("RRD capability protocol or instance identity differs")
        return result

    def endpoint_catalogue(self) -> dict[str, Any]:
        return self.call("endpoint-catalogue")

    def openapi(self) -> dict[str, Any]:
        return self.call("openapi-read")

    def create_session(
        self,
        principal_id: str,
        api_key: str,
        payload: Mapping[str, Any],
        options: RequestOptions,
    ) -> Session:
        principal = canonical_id(principal_id, "principal")
        if not api_key:
            raise RrdClientError("API-key credential must not be empty")
        request = RequestOptions(
            request_id=options.request_id,
            operation_id=options.operation_id,
            idempotency_key=options.idempotency_key,
            deadline_unix_ms=options.deadline_unix_ms,
            path_parameters=options.path_parameters,
            resource=options.resource,
            principal_id=principal,
            api_key=api_key,
        )
        lease = self.call("session-create", payload, request)
        return Session(principal_id=principal, lease=lease)

    def call(
        self,
        operation: OperationId,
        payload: Mapping[str, Any] | None = None,
        options: RequestOptions | None = None,
    ) -> dict[str, Any]:
        descriptor = ENDPOINTS[operation]
        options = options or RequestOptions()
        context = (
            None
            if descriptor["method"] == "GET"
            else request_context(options, descriptor["mutation"])
        )
        headers = {"Accept": "application/json"}
        if descriptor["authentication"] == "api_key":
            if options.principal_id is None or options.api_key is None:
                raise RrdClientError(f"{operation} requires API-key authentication")
            headers["X-RRD-Principal"] = canonical_id(options.principal_id, "principal")
            headers["Authorization"] = f"ApiKey {options.api_key}"
        elif descriptor["authentication"] == "session_bearer":
            if options.session is None:
                raise RrdClientError(f"{operation} requires a session")
            headers["X-RRD-Session"] = str(options.session.lease["session_id"])
            headers["Authorization"] = f"Bearer {options.session.lease['token']}"
        path = resolve_path(descriptor["path"], options.path_parameters)
        body = None
        if descriptor["method"] != "GET":
            headers["Content-Type"] = "application/json"
            resource = options.resource or default_resource(self.instance, options.path_parameters)
            body = json.dumps(
                {
                    "protocol": "rrd",
                    "protocol_version": 1,
                    "context": context,
                    "resource": {"segments": resource_segments(resource)},
                    "payload": dict(payload or {}),
                },
                separators=(",", ":"),
            )
        attempts = attempt_limit(
            descriptor["method"],
            descriptor["mutation"],
            options.idempotency_key,
            self.max_attempts,
        )
        last_error: Exception | None = None
        for _ in range(attempts):
            timeout = remaining_timeout(self.request_timeout, options.deadline_unix_ms)
            try:
                with self._http.stream(
                    descriptor["method"],
                    urljoin(self.base_url, path.lstrip("/")),
                    headers=headers,
                    content=body,
                    timeout=timeout,
                ) as response:
                    encoded = read_response(response, self.max_response_bytes)
                return decode_response(response, encoded, context)
            except (httpx.TransportError, httpx.TimeoutException) as error:
                last_error = error
                continue
        message = str(last_error) if last_error else "attempt budget exhausted"
        raise RrdClientError(f"RRD transport failed: {message}")
