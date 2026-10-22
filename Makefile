# =============================================================================
# quire-eval Makefile
# =============================================================================

CARGO ?= cargo
# A target with no `std`: building for it proves quire-checked pulls in nothing
# from `std` (FR-400-AC-9).
NO_STD_TARGET ?= thumbv7em-none-eabi

.PHONY: fmt fmt-check lint test build-no-std deny docs ci clean

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --workspace --all-targets -- -D warnings
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

test:
	$(CARGO) test --workspace
	$(CARGO) test --workspace --all-features

build-no-std:
	$(CARGO) build -p quire-checked --no-default-features --target $(NO_STD_TARGET)

deny:
	$(CARGO) deny check

# cargo doc is a separate lint pass: rustdoc-only lints fire here, not in clippy.
docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps --all-features

clean:
	$(CARGO) clean

ci: fmt-check lint test build-no-std deny docs
