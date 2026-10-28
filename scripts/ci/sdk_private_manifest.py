"""Private temporary workspace for the real-process SDK conformance harness.

This protects the harness rendezvous, not an installed RRFlow product secret.
"""

from __future__ import annotations

import contextlib
import os
from pathlib import Path
import secrets
import shutil
import stat
import tempfile


if os.name == "nt":
    import ctypes as c
    from ctypes import wintypes as w

    kernel = c.WinDLL("kernel32", use_last_error=True)
    adv = c.WinDLL("advapi32", use_last_error=True)

    def _api(dll, name, result, *arguments):
        function = getattr(dll, name)
        function.restype = result
        function.argtypes = arguments
        return function

    P = c.c_void_p
    U32 = w.DWORD
    close = _api(kernel, "CloseHandle", w.BOOL, P)
    free = _api(kernel, "LocalFree", P, P)
    current_process = _api(kernel, "GetCurrentProcess", P)
    open_token = _api(adv, "OpenProcessToken", w.BOOL, P, U32, c.POINTER(P))
    token_info = _api(adv, "GetTokenInformation", w.BOOL, P, U32, P, U32, c.POINTER(U32))
    sid_to_string = _api(adv, "ConvertSidToStringSidW", w.BOOL, P, c.POINTER(P))
    string_to_descriptor = _api(adv, "ConvertStringSecurityDescriptorToSecurityDescriptorW", w.BOOL, w.LPCWSTR, U32, c.POINTER(P), c.POINTER(U32))
    create_directory = _api(kernel, "CreateDirectoryW", w.BOOL, w.LPCWSTR, P)
    get_security = _api(adv, "GetNamedSecurityInfoW", U32, w.LPWSTR, U32, U32, c.POINTER(P), P, c.POINTER(P), P, c.POINTER(P))
    get_control = _api(adv, "GetSecurityDescriptorControl", w.BOOL, P, c.POINTER(w.WORD), c.POINTER(U32))
    get_ace = _api(adv, "GetAce", w.BOOL, P, U32, c.POINTER(P))

    class SecurityAttributes(c.Structure):
        _fields_ = [("length", U32), ("descriptor", P), ("inherit", w.BOOL)]

    def _require(value, label):
        if not value:
            raise OSError(c.get_last_error(), label)

    def _sid_text(sid):
        value = P()
        _require(sid_to_string(sid, c.byref(value)), "ConvertSidToStringSidW")
        try:
            return c.wstring_at(value)
        finally:
            free(value)

    def _user_sid():
        token = P()
        _require(open_token(current_process(), 0x0008, c.byref(token)), "OpenProcessToken")
        try:
            size = U32()
            token_info(token, 1, None, 0, c.byref(size))
            if not 0 < size.value <= 65536:
                raise RuntimeError("invalid Windows token user size")
            buf = c.create_string_buffer(size.value)
            _require(token_info(token, 1, buf, size, c.byref(size)), "GetTokenInformation")
            return _sid_text(c.cast(buf, c.POINTER(P))[0])
        finally:
            close(token)

    def _create_windows_directory(path, sddl):
        descriptor = P()
        _require(string_to_descriptor(sddl, 1, c.byref(descriptor), None), "parse private DACL")
        try:
            attributes = SecurityAttributes(c.sizeof(SecurityAttributes), descriptor, False)
            _require(create_directory(str(path), c.byref(attributes)), "create private directory")
        finally:
            free(descriptor)

    def _verify_windows(path, *, directory):
        if path.is_symlink() or path.lstat().st_file_attributes & 0x400:
            raise RuntimeError("private workspace refuses reparse objects")
        owner, acl, descriptor = P(), P(), P()
        error = get_security(str(path), 1, 0x1 | 0x4, c.byref(owner), None, c.byref(acl), None, c.byref(descriptor))
        if error:
            raise OSError(error, "read Windows security descriptor")
        try:
            user = _user_sid()
            if not owner or _sid_text(owner) != user or not acl:
                raise RuntimeError("private object requires current user ownership and a DACL")
            control, revision = w.WORD(), U32()
            _require(get_control(descriptor, c.byref(control), c.byref(revision)), "GetSecurityDescriptorControl")
            if directory and not control.value & 0x1000:
                raise RuntimeError("private workspace DACL is not protected")
            # ACL layout: revision, reserved, size, ACE count, reserved.
            count = c.cast(acl, c.POINTER(c.c_ushort))[2]
            if not 0 < count <= 2:
                raise RuntimeError("private DACL has unexpected ACE count")
            seen = set()
            for index in range(count):
                ace = P()
                _require(get_ace(acl, index, c.byref(ace)), "GetAce")
                header = c.string_at(ace, 4)
                if header[0] != 0 or int.from_bytes(header[2:4], "little") < 16:
                    raise RuntimeError("private DACL has unsupported ACE")
                flags = header[1]
                if directory and flags & 0x03 != 0x03:
                    raise RuntimeError("private directory ACE is not inheritable")
                identity = _sid_text(ace.value + 8)
                mask = c.c_uint32.from_address(ace.value + 4).value
                if identity not in {user, "S-1-5-18"} or identity in seen or mask & 0x001F01FF != 0x001F01FF:
                    raise RuntimeError("private DACL grants unexpected identity or access")
                seen.add(identity)
            if user not in seen:
                raise RuntimeError("private DACL does not grant the current user")
        finally:
            free(descriptor)


@contextlib.contextmanager
def private_workspace():
    if os.name == "nt":
        user = _user_sid()
        path = Path(tempfile.gettempdir()) / ("rrflow-sdk-" + secrets.token_hex(16))
        _create_windows_directory(path, f"O:{user}D:P(A;OICI;FA;;;{user})(A;OICI;FA;;;SY)")
        try:
            _verify_windows(path, directory=True)
            yield path
        finally:
            shutil.rmtree(path)
    elif os.name == "posix":
        with tempfile.TemporaryDirectory(prefix="rrflow-sdk-") as name:
            path = Path(name)
            if stat.S_IMODE(path.stat().st_mode) != 0o700:
                raise RuntimeError("private workspace mode is not 700")
            yield path
    else:
        raise RuntimeError("private SDK workspace is unsupported on this platform")


def verify_manifest(path):
    if os.name == "nt":
        _verify_windows(path.parent, directory=True)
        _verify_windows(path, directory=False)
    elif os.name == "posix":
        root = path.parent
        if root.is_symlink() or path.is_symlink():
            raise RuntimeError("private workspace refuses symlinks")
        root_stat, file_stat = root.stat(), path.stat()
        if (not stat.S_ISDIR(root_stat.st_mode) or root_stat.st_uid != os.geteuid()
                or stat.S_IMODE(root_stat.st_mode) != 0o700):
            raise RuntimeError("private workspace must be owned by this user with mode 700")
        if (not stat.S_ISREG(file_stat.st_mode) or file_stat.st_uid != os.geteuid()
                or stat.S_IMODE(file_stat.st_mode) != 0o600):
            raise RuntimeError("harness manifest must be owned by this user with mode 600")
    else:
        raise RuntimeError("private SDK manifest is unsupported on this platform")
