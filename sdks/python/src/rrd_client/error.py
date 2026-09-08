"""Current Python client and typed API errors."""


class RrdClientError(Exception):
    pass


class RrdApiError(RrdClientError):
    def __init__(self, status: int, code: str, message: str, retryable: bool) -> None:
        super().__init__(f"RRD API {status}: {code}: {message}")
        self.status = status
        self.code = code
        self.retryable = retryable
