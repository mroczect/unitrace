"""ctypes bindings ke libunitrace_db_ffi.so."""

from __future__ import annotations

import ctypes
import json
from pathlib import Path
from typing import Any

from .config import find_data_files, find_ffi_lib, get_password


class UnitraceFFI:
    def __init__(self, lib_path: Path) -> None:
        self._lib = ctypes.CDLL(str(lib_path))

        self._lib.unitrace_init.argtypes = [
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self._lib.unitrace_init.restype = ctypes.c_void_p

        self._lib.unitrace_search.argtypes = [ctypes.c_char_p]
        self._lib.unitrace_search.restype = ctypes.c_void_p

        self._lib.unitrace_free_string.argtypes = [ctypes.c_void_p]
        self._lib.unitrace_free_string.restype = None

    def _read_and_free(self, ptr: int | None) -> str | None:
        if not ptr:
            return None
        try:
            return ctypes.string_at(ptr).decode("utf-8")
        finally:
            self._lib.unitrace_free_string(ptr)

    def init(self, db_path: Path, meta_path: Path, key: str) -> dict[str, Any]:
        raw = self._read_and_free(
            self._lib.unitrace_init(
                str(db_path).encode("utf-8"),
                str(meta_path).encode("utf-8"),
                key.encode("utf-8"),
            )
        )
        if raw is None:
            return {"status": "error", "message": "FFI returned null", "columns": []}
        return json.loads(raw)

    def search(self, keyword: str) -> dict[str, Any] | None:
        raw = self._read_and_free(self._lib.unitrace_search(keyword.encode("utf-8")))
        return json.loads(raw) if raw else None


def open_default() -> tuple[UnitraceFFI, dict[str, Any]]:
    """Convenience: buka FFI dengan paths dari config, inisialisasi, return (ffi, init_result)."""
    ffi = UnitraceFFI(find_ffi_lib())
    db, meta = find_data_files()
    result = ffi.init(db, meta, get_password())
    return ffi, result
