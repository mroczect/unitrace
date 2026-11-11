# `unitrace_db_ffi` — C ABI bindings

Exposes the unitrace database GUI controller as a `cdylib` for the Tkinter
frontend (`unitrace_db_gui_py`).

## Contract

| Symbol                                                     | Signature                                             |
| ---------------------------------------------------------- | ----------------------------------------------------- |
| `unitrace_init(const char* pkg, const char* key) -> char*` | JSON `{status, message, columns, signature_verified}` |
| `unitrace_search(const char* kw) -> char*`                 | JSON `{status_text, rows, not_found_proof}`           |
| `unitrace_free_string(char* p)`                            | frees a string returned above                         |
| `unitrace_close()`                                         | drops the current session                             |

Every returned `char*` is owned by the caller and must be freed via
`unitrace_free_string`. `unitrace_init` performs: permission check on the
`.upkg` file (must be `0600` on Unix), Ed25519 signature verification against
the embedded public key, then decrypts into a `tempfile::TempDir`.

Rate limiting: three failed `unitrace_init` calls within 60 s trigger a
cooldown window. Signature failures sleep 1.5 s before returning.
