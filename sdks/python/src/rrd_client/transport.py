"""Current bounded HTTP response carriage and envelope decoding."""

from __future__ import annotations

import json
from typing import Annotated, Any, Literal, cast

import httpx
from pydantic import BaseModel, ConfigDict, Field, ValidationError

from .error import RrdApiError, RrdClientError


class ErrorBody(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)

    code: str
    message: str
    retryable: bool
    details: dict[str, str] = Field(default_factory=dict)


class OkOutcome(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)

    status: Literal["ok"]
    payload: Any


class ErrorOutcome(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)

    status: Literal["error"]
    error: ErrorBody


class ResponseEnvelope(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)

    protocol: Literal["rrd"]
    protocol_version: Literal[1]
    request_id: str
    operation_id: str
    outcome: Annotated[OkOutcome | ErrorOutcome, Field(discriminator="status")]


def decode_response(
    response: httpx.Response,
    encoded: bytes,
    context: dict[str, Any] | None,
) -> dict[str, Any]:
    try:
        envelope = ResponseEnvelope.model_validate_json(encoded)
    except (ValidationError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RrdClientError(f"RRD response envelope is invalid: {error}") from error
    if context and (
        envelope.request_id != context["request_id"]
        or envelope.operation_id != context["operation_id"]
    ):
        raise RrdClientError("RRD response request/operation identity differs")
    if response.is_success != (envelope.outcome.status == "ok"):
        raise RrdClientError("RRD HTTP status and typed outcome disagree")
    if isinstance(envelope.outcome, ErrorOutcome):
        outcome_error = envelope.outcome.error
        raise RrdApiError(
            response.status_code,
            outcome_error.code,
            outcome_error.message,
            outcome_error.retryable,
        )
    if not isinstance(envelope.outcome.payload, dict):
        raise RrdClientError("RRD success payload must be an object")
    return cast(dict[str, Any], envelope.outcome.payload)


def read_response(response: httpx.Response, maximum: int) -> bytes:
    declared = response.headers.get("content-length")
    if declared and declared.isdigit() and int(declared) > maximum:
        raise RrdClientError("RRD response exceeded the configured byte limit")
    chunks = []
    length = 0
    for chunk in response.iter_bytes():
        length += len(chunk)
        if length > maximum:
            raise RrdClientError("RRD response exceeded the configured byte limit")
        chunks.append(chunk)
    return b"".join(chunks)
