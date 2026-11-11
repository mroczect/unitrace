SHELL = /bin/bash
.SHELLFLAGS = -euo pipefail -c

CARGO          = cargo
MEMBERS        = unitrace_db_handler unitrace_db_core unitrace_db
PUBLISH_ORDER  = unitrace_db_handler unitrace_db_core unitrace_db

PKG            ?= unitrace_db_handler
CLIPPY_FLAGS   ?= -- -D warnings

.DEFAULT_GOAL  := help
.PHONY: help
help: ## Show this help
	@echo "Unitrace workspace — dev shortcuts"
	@echo ""
	@echo "Usage: make <target> [PKG=<crate>] [CLIPPY_FLAGS=<flags>]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-24s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Quick start:"
	@echo "  make install-tools   # one-time: install dev tooling"
	@echo "  make check           # fast compile check (all crates)"
	@echo "  make handler         # strict CI on unitrace_db_handler"
	@echo "  make core            # strict CI on unitrace_db_core"
	@echo "  make db              # strict CI on unitrace_db"
	@echo "  make ci              # full pre-commit"
	@echo "  make ready           # max safety before push"
	@echo ""
	@echo "Available crates (PKG):"
	@echo "  unitrace_db_handler  (contract, no_std, only thiserror)"
	@echo "  unitrace_db_core     (implementation, holds rusqlite/sha2/csv)"
	@echo "  unitrace_db          (facade, thin re-export)"

# ─────────────────────────────────────────────────────────────
#  BUILD / CHECK  (fast feedback loop)
# ─────────────────────────────────────────────────────────────
.PHONY: check
check: ## Fast compile check (workspace)
	$(CARGO) check --workspace

.PHONY: check-all
check-all: ## Compile check (all targets & features)
	$(CARGO) check --workspace --all-targets --all-features

.PHONY: check-pkg
check-pkg: ## Compile check single crate (PKG=<name>)
	$(CARGO) check -p $(PKG) --all-targets

.PHONY: build
build: ## Debug build (workspace)
	$(CARGO) build --workspace

.PHONY: build-pkg
build-pkg: ## Debug build single crate
	$(CARGO) build -p $(PKG)

.PHONY: release
release: ## Release build (LTO + strip)
	$(CARGO) build --release --workspace

.PHONY: release-pkg
release-pkg: ## Release build single crate
	$(CARGO) build --release -p $(PKG)

# ─────────────────────────────────────────────────────────────
#  TEST
# ─────────────────────────────────────────────────────────────
.PHONY: test
test: ## Run all tests
	$(CARGO) test --workspace

.PHONY: test-verbose
test-verbose: ## Run tests with backtrace + output
	RUST_BACKTRACE=1 $(CARGO) test --workspace -- --nocapture

.PHONY: test-pkg
test-pkg: ## Run tests for single crate
	$(CARGO) test -p $(PKG)

.PHONY: test-verbose-pkg
test-verbose-pkg: ## Run single-crate tests with backtrace
	RUST_BACKTRACE=1 $(CARGO) test -p $(PKG) -- --nocapture

.PHONY: test-doc
test-doc: ## Run doctests only
	$(CARGO) test --workspace --doc

.PHONY: test-lib
test-lib: ## Run lib unit tests only (skip integration)
	$(CARGO) test --workspace --lib

.PHONY: coverage
coverage: ## HTML coverage report (requires cargo-llvm-cov)
	$(CARGO) llvm-cov --workspace --html
	@echo "→ Open: target/llvm-cov/html/index.html"

# ─────────────────────────────────────────────────────────────
#  LINT / FORMAT
# ─────────────────────────────────────────────────────────────
.PHONY: fmt
fmt: ## Apply rustfmt (workspace)
	$(CARGO) fmt --all

.PHONY: fmt-pkg
fmt-pkg: ## Apply rustfmt single crate
	$(CARGO) fmt -p $(PKG)

.PHONY: fmt-check
fmt-check: ## Verify formatting (no write)
	$(CARGO) fmt --all -- --check

.PHONY: fmt-check-pkg
fmt-check-pkg: ## Verify formatting single crate
	$(CARGO) fmt -p $(PKG) -- --check

.PHONY: clippy
clippy: ## Clippy (workspace, warnings-as-errors)
	$(CARGO) clippy --workspace --all-targets --all-features $(CLIPPY_FLAGS)

.PHONY: clippy-strict
clippy-strict: ## Clippy with -- -D warnings (explicit)
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: clippy-pkg
clippy-pkg: ## Clippy single crate
	$(CARGO) clippy -p $(PKG) --all-targets --all-features $(CLIPPY_FLAGS)

.PHONY: clippy-pkg-strict
clippy-pkg-strict: ## Clippy single crate with -D warnings
	$(CARGO) clippy -p $(PKG) --all-targets --all-features -- -D warnings

.PHONY: lint
lint: fmt clippy ## Format + clippy (applies, not check)

.PHONY: lint-check
lint-check: fmt-check clippy-strict ## Format-check + strict clippy

# ─────────────────────────────────────────────────────────────
#  CI  (pre-commit gauntlets)
# ─────────────────────────────────────────────────────────────
.PHONY: ci
ci: fmt-check clippy test ## Fast pre-commit: fmt + clippy + test
	@echo "✓ ci passed"

.PHONY: ci-fast
ci-fast: fmt-check clippy ## Skip tests (for fast iterative work)
	@echo "✓ ci-fast passed"

.PHONY: ci-strict
ci-strict: fmt-check clippy-strict test verify-layers ## Strict pre-commit + layer check
	@echo "✓ ci-strict passed"

.PHONY: ci-pkg
ci-pkg: fmt-check-pkg clippy-pkg test-verbose-pkg ## Pre-commit single crate
	@echo "✓ ci-pkg ($(PKG)) passed"

.PHONY: ci-pkg-strict
ci-pkg-strict: fmt-check-pkg clippy-pkg-strict test-verbose-pkg ## Strict single crate
	@echo "✓ ci-pkg-strict ($(PKG)) passed"

.PHONY: ready
ready: ci-strict deny ## Max safety: everything + deny + audit
	@if command -v cargo-audit >/dev/null 2>&1; then \
		echo "→ cargo audit..."; \
		$(CARGO) audit; \
	else \
		echo "→ cargo-audit not installed (skip). Install: cargo install cargo-audit"; \
	fi
	@echo "✓ ready: safe to push"

# ─────────────────────────────────────────────────────────────
#  PER-LAYER SHORTCUTS  (dependency order: handler → core → db)
# ─────────────────────────────────────────────────────────────
.PHONY: handler
handler: PKG=unitrace_db_handler
handler: ci-pkg-strict ## Strict CI: unitrace_db_handler

.PHONY: core
core: PKG=unitrace_db_core
core: ci-pkg-strict ## Strict CI: unitrace_db_core

.PHONY: db
db: PKG=unitrace_db
db: ci-pkg-strict ## Strict CI: unitrace_db

.PHONY: all-layers
all-layers: ## Run handler → core → db in order (fail-fast)
	@for crate in $(PUBLISH_ORDER); do \
		echo ""; \
		echo "═══ $$crate ═══"; \
		$(MAKE) --no-print-directory ci-pkg-strict PKG=$$crate || exit 1; \
	done
	@echo ""; echo "✓ All 3 layers passed"

# ─────────────────────────────────────────────────────────────
#  VERIFY  (architecture & security)
# ─────────────────────────────────────────────────────────────
.PHONY: verify-layers
verify-layers: ## Enforce db → core → handler dependency direction
	@printf "→ handler must NOT depend on core/db ... "
	@if $(CARGO) tree -p unitrace_db_handler --depth 1 2>/dev/null \
		| grep -qE "unitrace_db_core v|unitrace_db v"; then \
		echo "FAIL"; \
		echo "  ✗ unitrace_db_handler depends on core/db — forbidden"; \
		exit 1; \
	fi
	@echo "OK"
	@printf "→ core must NOT depend on db ... "
	@if $(CARGO) tree -p unitrace_db_core --depth 1 2>/dev/null \
		| grep -qE "unitrace_db v"; then \
		echo "FAIL"; \
		echo "  ✗ unitrace_db_core depends on unitrace_db — forbidden"; \
		exit 1; \
	fi
	@echo "OK"
	@printf "→ db must NOT depend directly on rusqlite/csv/sha2/hex ... "
	@for dep in rusqlite csv sha2 hex; do \
		if $(CARGO) tree -p unitrace_db --depth 1 2>/dev/null \
			| grep -qE "$$dep v"; then \
			echo "FAIL"; \
			echo "  ✗ unitrace_db depends directly on $$dep — should go through core"; \
			exit 1; \
		fi; \
	done
	@echo "OK"
	@echo "✓ Dependency layers OK"

.PHONY: deny
deny: ## cargo-deny (advisories, licenses, sources, bans)
	@if ! command -v cargo-deny >/dev/null 2>&1; then \
		echo "✗ cargo-deny not installed. Run: cargo install cargo-deny"; \
		exit 1; \
	fi
	$(CARGO) deny check

.PHONY: audit
audit: ## Security audit (cargo-audit)
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "✗ cargo-audit not installed. Run: cargo install cargo-audit"; \
		exit 1; \
	fi
	$(CARGO) audit

.PHONY: machete
machete: ## Detect unused dependencies (cargo-machete)
	@if ! command -v cargo-machete >/dev/null 2>&1; then \
		echo "✗ cargo-machete not installed. Run: cargo install cargo-machete"; \
		exit 1; \
	fi
	$(CARGO) machete

.PHONY: outdated
outdated: ## Show outdated dependencies
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "✗ cargo-outdated not installed. Run: cargo install cargo-outdated"; \
		exit 1; \
	fi
	$(CARGO) outdated --workspace

.PHONY: udeps
udeps: ## Nightly-only unused deps (RUSTC_BOOTSTRAP workaround)
	@echo "→ Requires nightly + cargo-udeps. Trying RUSTC_BOOTSTRAP=1..."
	RUSTC_BOOTSTRAP=1 $(CARGO) udeps --workspace

# ─────────────────────────────────────────────────────────────
#  DOCS
# ─────────────────────────────────────────────────────────────
.PHONY: doc
doc: ## Build workspace docs (no deps)
	$(CARGO) doc --workspace --no-deps

.PHONY: doc-open
doc-open: doc ## Build and open docs in browser
	$(CARGO) doc --workspace --no-deps --open

.PHONY: doc-pkg
doc-pkg: ## Build single-crate docs
	$(CARGO) doc -p $(PKG) --no-deps

# ─────────────────────────────────────────────────────────────
#  WATCH  (live dev loop)
# ─────────────────────────────────────────────────────────────
.PHONY: watch-check
watch-check: ## Watch + cargo check (fastest loop)
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "✗ cargo-watch not installed. Run: cargo install cargo-watch"; \
		exit 1; \
	fi
	$(CARGO) watch -q -x 'check --workspace'

.PHONY: watch-test
watch-test: ## Watch + test
	$(CARGO) watch -q -x 'test --workspace'

.PHONY: watch-clippy
watch-clippy: ## Watch + clippy (strict)
	$(CARGO) watch -q -x 'clippy --workspace --all-targets -- -D warnings'

.PHONY: watch-check-pkg
watch-check-pkg: ## Watch + check single crate
	$(CARGO) watch -q -x 'check -p $(PKG)'

.PHONY: watch-test-pkg
watch-test-pkg: ## Watch + test single crate
	$(CARGO) watch -q -x 'test -p $(PKG)'

# ─────────────────────────────────────────────────────────────
#  MAINTENANCE
# ─────────────────────────────────────────────────────────────
.PHONY: update
update: ## Update dependencies (respect semver)
	$(CARGO) update --workspace

.PHONY: clean
clean: ## Remove target/
	$(CARGO) clean

.PHONY: clean-incremental
clean-incremental: ## Clear incremental cache only (fast rebuild reset)
	@rm -rf target/debug/incremental
	@echo "✓ incremental cache cleared"

.PHONY: install-tools
install-tools: ## Install essential dev tools (watch, audit, deny, machete, outdated)
	$(CARGO) install --locked \
		cargo-watch \
		cargo-audit \
		cargo-deny \
		cargo-machete \
		cargo-outdated

.PHONY: install-tools-optional
install-tools-optional: ## Install coverage & size tools (llvm-cov, bloat)
	$(CARGO) install --locked \
		cargo-llvm-cov \
		cargo-bloat

# ─────────────────────────────────────────────────────────────
#  CONTEXT  (for AI / review snapshots)
# ─────────────────────────────────────────────────────────────
.PHONY: snap
snap: ## Snapshot all crate sources into dev/root.md
	@mkdir -p dev
	@for crate in $(MEMBERS); do \
		if [ -d "$$crate/src" ]; then \
			echo "→ $$crate/src"; \
			snapcat "$$crate/src" --format markdown > "dev/$$crate.src.snapcat.md" 2>/dev/null || true; \
		fi; \
		if [ -d "$$crate/tests" ]; then \
			echo "→ $$crate/tests"; \
			snapcat "$$crate/tests" --format markdown > "dev/$$crate.tests.snapcat.md" 2>/dev/null || true; \
		fi; \
	done
	@cat dev/*.snapcat.md > dev/root.md 2>/dev/null || true
	@echo "✓ See dev/root.md"

.PHONY: tree
tree: ## Dependency tree for one crate (PKG=<name>)
	$(CARGO) tree -p $(PKG)

.PHONY: tree-all
tree-all: ## Full workspace dependency tree
	$(CARGO) tree --workspace

# ─────────────────────────────────────────────────────────────
#  INFO
# ─────────────────────────────────────────────────────────────
.PHONY: info
info: ## Show toolchain + workspace summary
	@echo "toolchain : $$(rustc --version)"
	@echo "cargo     : $$(cargo --version)"
	@echo "target    : $$(rustc -vV | grep host | cut -d' ' -f2)"
	@echo ""
	@echo "workspace members:"
	@for m in $(MEMBERS); do echo "  · $$m"; done
	@echo ""
	@echo "publish order: $(PUBLISH_ORDER)"
