SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

CARGO := cargo
PKG ?=
CRATES := unitrace_db_handler unitrace_db_core unitrace_db unitrace_db_gui_handler unitrace_db_gui_core unitrace_db_ffi unitrace_db_cli

.DEFAULT_GOAL := help
.PHONY: help
help: 
	@echo "usage: make <target> [PKG=<crate>]"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'
	@echo ""
	@echo "crates: $(CRATES)"

.PHONY: check build release
check: 
	$(CARGO) check $(if $(PKG),-p $(PKG),--workspace) --all-targets --all-features

build: 
	$(CARGO) build $(if $(PKG),-p $(PKG),--workspace)

release: 
	$(CARGO) build --release $(if $(PKG),-p $(PKG),--workspace)

.PHONY: test
test: 
	$(CARGO) test $(if $(PKG),-p $(PKG),--workspace)

.PHONY: fmt fmt-check clippy
fmt: 
	$(CARGO) fmt $(if $(PKG),-p $(PKG),--all)

fmt-check: 
	$(CARGO) fmt $(if $(PKG),-p $(PKG),--all) -- --check

clippy: 
	$(CARGO) clippy $(if $(PKG),-p $(PKG),--workspace) --all-targets --all-features -- -D warnings

.PHONY: ci ci-strict handler core db
ci: fmt-check clippy test 
	@echo "✓ ci passed"

ci-strict: ci verify-layers 
	@echo "✓ ci-strict passed"

handler: 
	$(MAKE) ci-strict PKG=unitrace_db_handler

core: 
	$(MAKE) ci-strict PKG=unitrace_db_core

db: 
	$(MAKE) ci-strict PKG=unitrace_db

.PHONY: verify-layers
verify-layers:
	@if $(CARGO) tree -p unitrace_db_handler --depth 1 2>/dev/null | grep -qE 'unitrace_db_(core|db|gui_core|gui_handler) v'; then \
		echo "FAIL: unitrace_db_handler depends on implementation crates"; exit 1; fi
	@if $(CARGO) tree -p unitrace_db_gui_handler --depth 1 2>/dev/null | grep -qE 'unitrace_db_(core|db|gui_core) v'; then \
		echo "FAIL: unitrace_db_gui_handler depends on implementation crates"; exit 1; fi
	@if $(CARGO) tree -p unitrace_db_core --depth 1 2>/dev/null | grep -q 'unitrace_db v'; then \
		echo "FAIL: unitrace_db_core depends on unitrace_db facade"; exit 1; fi
	@if $(CARGO) tree -p unitrace_db_gui_core --depth 1 2>/dev/null | grep -q 'unitrace_db v'; then \
		echo "FAIL: unitrace_db_gui_core depends on unitrace_db facade"; exit 1; fi
	@if $(CARGO) tree -p unitrace_db --depth 1 2>/dev/null | grep -qE '(rusqlite|csv|sha2 0\.11|hex) v'; then \
		echo "FAIL: unitrace_db depends directly on rusqlite/csv/sha2/hex"; exit 1; fi
	@if $(CARGO) tree -p unitrace_db_ffi --depth 1 2>/dev/null | grep -qE 'unitrace_db_(handler|core|gui_handler|gui_core) v'; then \
		echo "FAIL: unitrace_db_ffi depends on internal crates (must use facade only)"; exit 1; fi
	@if $(CARGO) tree -p unitrace_db_cli --depth 1 2>/dev/null | grep -qE 'unitrace_db_(handler|core|gui_handler|gui_core) v'; then \
		echo "FAIL: unitrace_db_cli depends on internal crates (must use facade only)"; exit 1; fi
	@echo "✓ layers OK (strict: handler ← core ← facade ← consumers)"

.PHONY: audit deny machete outdated
audit: 
	@command -v cargo-audit >/dev/null || { echo "install: cargo install cargo-audit"; exit 1; }
	$(CARGO) audit

deny: 
	@command -v cargo-deny >/dev/null || { echo "install: cargo install cargo-deny"; exit 1; }
	$(CARGO) deny check

machete: 
	@command -v cargo-machete >/dev/null || { echo "install: cargo install cargo-machete"; exit 1; }
	$(CARGO) machete

outdated: 
	@command -v cargo-outdated >/dev/null || { echo "install: cargo install cargo-outdated"; exit 1; }
	$(CARGO) outdated --workspace

.PHONY: doc coverage
doc: 
	$(CARGO) doc $(if $(PKG),-p $(PKG),--workspace) --no-deps

coverage: 
	$(CARGO) llvm-cov --workspace --html
	@echo "open target/llvm-cov/html/index.html"

.PHONY: update clean install-tools
update: 
	$(CARGO) update --workspace

clean: 
	$(CARGO) clean

install-tools: 
	$(CARGO) install --locked cargo-watch cargo-audit cargo-deny cargo-machete cargo-outdated
