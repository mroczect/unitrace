# `unitrace_db_cli` — developer toolchain

Build, sign, and package unitrace databases.

## Commands

| Command                                | Purpose                                    |
| -------------------------------------- | ------------------------------------------ |
| `keygen <signing.key>`                 | generate Ed25519 keypair; print pubkey hex |
| `build <csv> <signing.key> <out.upkg>` | CSV → SQLCipher → sign → `.upkg`           |
| `verify <in.upkg>`                     | verify signature against embedded pubkey   |
| `inspect <in.upkg>`                    | show package metadata                      |

Requires `UNITRACE_DB_KEY` env var on `build` (SQLCipher passphrase).
Private key file must be `chmod 600`. This crate is **dev-only** — it is
not shipped to end users.
