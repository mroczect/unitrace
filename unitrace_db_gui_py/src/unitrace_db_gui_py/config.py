"""Path resolution untuk FFI library dan database artifacts."""

from __future__ import annotations

import os
from pathlib import Path


def _walk_up_for(name: str) -> Path | None:
    """Cari file/dir bernama `name` mulai dari file ini ke atas."""
    for parent in Path(__file__).resolve().parents:
        candidate = parent / name
        if candidate.exists():
            return candidate
    return None


def find_ffi_lib() -> Path:
    if env := os.environ.get("UNITRACE_FFI_LIB"):
        p = Path(env).expanduser().resolve()
        if not p.is_file():
            raise FileNotFoundError(f"UNITRACE_FFI_LIB points to missing file: {p}")
        return p

    for parent in Path(__file__).resolve().parents:
        for profile in ("debug", "release"):
            candidate = parent / "target" / profile / "libunitrace_db_ffi.so"
            if candidate.is_file():
                return candidate

    raise FileNotFoundError(
        "libunitrace_db_ffi.so not found. Run `cargo build -p unitrace_db_ffi` "
        "or set UNITRACE_FFI_LIB=/absolute/path/to/libunitrace_db_ffi.so"
    )


def find_data_files() -> tuple[Path, Path]:
    env_db = os.environ.get("UNITRACE_DB_PATH")
    env_meta = os.environ.get("UNITRACE_META_PATH")
    if env_db and env_meta:
        return Path(env_db).resolve(), Path(env_meta).resolve()

    for parent in Path(__file__).resolve().parents:
        db = parent / "data.db"
        meta = parent / "data.meta.json"
        if db.is_file() and meta.is_file():
            return db, meta

    raise FileNotFoundError(
        "data.db and data.meta.json not found. Run:\n"
        "  cargo run -p unitrace_db_cli -- dump/data.csv data.db data.meta.json\n"
        "or set UNITRACE_DB_PATH and UNITRACE_META_PATH."
    )


def get_password() -> str:
    return os.environ.get("UNITRACE_DB_KEY", "supersecretkey123")
