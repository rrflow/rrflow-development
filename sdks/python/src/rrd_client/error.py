"""Current Python client and typed API errors."""

from collections.abc import Mapping


class RrdClientError(Exception):
    pass


class RrdApiError(RrdClientError):
    def __init__(
        self,
        status: int,
        code: str,
        message: str,
        retryable: bool,
        details: Mapping[str, str] | None = None,
    ) -> None:
        super().__init__(f"RRD API {status}: {code}: {message}")
        self.status = status
        self.code = code
        self.retryable = retryable
        self.details = dict(details) if details is not None else {}
