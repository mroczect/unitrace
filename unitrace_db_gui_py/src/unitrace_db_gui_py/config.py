
from __future__ import annotations

import os
from pathlib import Path


def find_ffi_lib() -> Path:
    if env := os.environ.get("UNITRACE_FFI_LIB"):
        p = Path(env).expanduser().resolve()
        if not p.is_file():
            raise FileNotFoundError(f"UNITRACE_FFI_LIB missing: {p}")
        return p

    for parent in Path(__file__).resolve().parents:
        for profile in ("release", "debug"):
            cand = parent / "target" / profile / "libunitrace_db_ffi.so"
            if cand.is_file():
                return cand

    raise FileNotFoundError(
        "libunitrace_db_ffi.so not found. Run `cargo build --release -p unitrace_db_ffi`."
    )


def find_package() -> Path:
    if env := os.environ.get("UNITRACE_PKG_PATH"):
        p = Path(env).expanduser().resolve()
        if not p.is_file():
            raise FileNotFoundError(f"UNITRACE_PKG_PATH missing: {p}")
        return p

    for parent in Path(__file__).resolve().parents:
        cand = parent / "out.upkg"
        if cand.is_file():
            return cand

    raise FileNotFoundError(
        "out.upkg not found. Build dengan:\n"
        "  unitrace-db-cli build data.csv signing.key out.upkg\n"
        "atau set UNITRACE_PKG_PATH=/abs/path/to/out.upkg"
    )


def get_password() -> str | None:
    return os.environ.get("UNITRACE_DB_KEY")
