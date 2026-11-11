
from __future__ import annotations

import ctypes
import json
from pathlib import Path
from typing import Any

from .config import find_ffi_lib, find_package


class UnitraceFFI:
    def __init__(self, lib_path: Path) -> None:
        self._lib = ctypes.CDLL(str(lib_path))

        self._lib.unitrace_init.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        self._lib.unitrace_init.restype = ctypes.c_void_p

        self._lib.unitrace_search.argtypes = [ctypes.c_char_p]
        self._lib.unitrace_search.restype = ctypes.c_void_p

        self._lib.unitrace_free_string.argtypes = [ctypes.c_void_p]
        self._lib.unitrace_free_string.restype = None

        self._lib.unitrace_close.argtypes = []
        self._lib.unitrace_close.restype = None

    def _read(self, ptr: int | None) -> str | None:
        if not ptr:
            return None
        try:
            return ctypes.string_at(ptr).decode("utf-8")
        finally:
            self._lib.unitrace_free_string(ptr)

    def init(self, package_path: Path, password: str) -> dict[str, Any]:
        raw = self._read(
            self._lib.unitrace_init(
                str(package_path).encode("utf-8"),
                password.encode("utf-8"),
            )
        )
        if raw is None:
            return {
                "status": "error",
                "message": "FFI returned null",
                "columns": [],
                "signature_verified": False,
            }
        return json.loads(raw)

    def search(self, keyword: str) -> dict[str, Any] | None:
        raw = self._read(self._lib.unitrace_search(keyword.encode("utf-8")))
        return json.loads(raw) if raw else None

    def close(self) -> None:
        self._lib.unitrace_close()


def open_default(password: str) -> tuple[UnitraceFFI, dict[str, Any]]:
    ffi = UnitraceFFI(find_ffi_lib())
    pkg = find_package()
    result = ffi.init(pkg, password)
    return ffi, result
