"""Pin an ordinary Windows qualification harness and its pathname ancestors.

The guard is qualification tooling, not installed product state. It protects
the path from the time it is acquired until its caller finishes the owned job.
It does not establish how a binary was produced before acquisition.
"""
from __future__ import annotations

from contextlib import ExitStack, contextmanager
import ctypes as c
from ctypes import wintypes as w
import hashlib
from pathlib import Path
import sys

from windows_offline import P, U32, api, boolean, close, kernel

_INVALID = P(-1).value
_MAX_BYTES = 2 * 1024**3
_READ_ATTRIBUTES = 0x80
_GENERIC_READ = 0x80000000
_SHARE_READ = 0x1
_SHARE_WRITE = 0x2
_BACKUP_SEMANTICS = 0x02000000
_OPEN_REPARSE_POINT = 0x00200000
_DIRECTORY = 0x10
_REPARSE_POINT = 0x400


class _FileTime(c.Structure):
    _fields_ = [("low", U32), ("high", U32)]


class _FileInformation(c.Structure):
    _fields_ = [("attributes", U32), ("created", _FileTime), ("accessed", _FileTime),
                ("written", _FileTime), ("volume", U32), ("size_high", U32),
                ("size_low", U32), ("links", U32), ("index_high", U32), ("index_low", U32)]


def _open(stack: ExitStack, path: Path, *, directory: bool):
    access = _READ_ATTRIBUTES if directory else _GENERIC_READ
    sharing = _SHARE_READ | _SHARE_WRITE if directory else _SHARE_READ
    flags = _OPEN_REPARSE_POINT | (_BACKUP_SEMANTICS if directory else 0)
    handle = api(kernel, "CreateFileW", P, w.LPCWSTR, U32, U32, P, U32, U32, P)(
        str(path), access, sharing, None, 3, flags, None)
    if handle == _INVALID:
        raise OSError(c.get_last_error(), f"cannot pin harness {'directory' if directory else 'file'}")
    stack.callback(close, handle)
    info = _information(handle)
    if api(kernel, "GetFileType", U32, P)(handle) != 1 or info.attributes & _REPARSE_POINT:
        raise RuntimeError("harness path must contain ordinary disk objects, not reparse points")
    if bool(info.attributes & _DIRECTORY) != directory:
        raise RuntimeError("harness path has an unexpected object type")
    if not directory and info.links != 1:
        raise RuntimeError("harness image must have exactly one hard link")
    return handle, info


def _information(handle) -> _FileInformation:
    info = _FileInformation()
    boolean(api(kernel, "GetFileInformationByHandle", w.BOOL, P, P)(handle, c.byref(info)),
            "read pinned harness identity")
    return info


def _identity(info: _FileInformation):
    return (info.attributes, info.volume, info.size_high, info.size_low,
            info.links, info.index_high, info.index_low, info.written.high, info.written.low)


def _digest(handle, before: _FileInformation) -> str:
    size = (int(before.size_high) << 32) | int(before.size_low)
    if size > _MAX_BYTES:
        raise RuntimeError("harness digest exceeds two-GiB input bound")
    read_file = api(kernel, "ReadFile", w.BOOL, P, P, U32, P, P)
    buffer = c.create_string_buffer(65536)
    amount = U32()
    total = 0
    digest = hashlib.sha256()
    while True:
        boolean(read_file(handle, buffer, len(buffer), c.byref(amount), None), "read pinned harness")
        if amount.value == 0:
            break
        total += amount.value
        if total > size:
            raise RuntimeError("harness image grew during identity read")
        digest.update(buffer.raw[:amount.value])
    if total != size or _identity(_information(handle)) != _identity(before):
        raise RuntimeError("harness image changed during identity read")
    return digest.hexdigest()


def verify_receipt_digest(receipt: dict, pinned_digest: str) -> None:
    if receipt.get("harness_sha256") != pinned_digest:
        raise RuntimeError("pinned harness digest differs from receipt")


@contextmanager
def pin_harness_image(path: Path):
    """Yield the held image's digest while the pathname and parents stay pinned."""
    if sys.platform != "win32":
        raise RuntimeError("Windows harness image pin requires Windows")
    path = Path(path)
    if not path.is_absolute() or len(path.drive) != 2 or not path.drive.endswith(":"):
        raise RuntimeError("harness image requires an absolute drive path")
    parts = path.parts
    if len(parts) < 2 or any(part in (".", "..") or ":" in part for part in parts[1:]):
        raise RuntimeError("invalid harness image path")
    with ExitStack() as stack:
        current = Path(parts[0])
        _, root_info = _open(stack, current, directory=True)
        for component in parts[1:-1]:
            current /= component
            _, info = _open(stack, current, directory=True)
            if info.volume != root_info.volume:
                raise RuntimeError("harness directory crosses a volume boundary")
        handle, info = _open(stack, path, directory=False)
        if info.volume != root_info.volume:
            raise RuntimeError("harness image crosses a volume boundary")
        yield _digest(handle, info)
