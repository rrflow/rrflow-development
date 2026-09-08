"""Current pre-qualification loopback endpoint policy."""

import ipaddress
from urllib.parse import urlsplit

from .error import RrdClientError


def loopback_url(value: str) -> str:
    parsed = urlsplit(value)
    host = parsed.hostname
    loopback = host == "localhost"
    if host and host != "localhost":
        try:
            loopback = ipaddress.ip_address(host).is_loopback
        except ValueError:
            loopback = False
    if (
        parsed.scheme != "http"
        or not loopback
        or parsed.username
        or parsed.password
        or parsed.query
        or parsed.fragment
    ):
        raise RrdClientError(
            "RRD Python client permits only credential-free loopback HTTP before TLS qualification"
        )
    return value.rstrip("/") + "/"
