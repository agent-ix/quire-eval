# =============================================================================
# quire-eval Makefile
# =============================================================================

CARGO ?= cargo
# A target with no `std`: building for it proves quire-checked pulls in nothing
# from `std` (FR-400-AC-9). `quire-eval` has no `std` feature at all (FR-401).
NO_STD_TARGET ?= thumbv7em-none-eabi

.PHONY: fmt fmt-check lint mint-probe test build-no-std deny docs ci clean

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --workspace --all-targets -- -D warnings
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings
	$(CARGO) clippy -p quire-checked --lib --no-default-features --target $(NO_STD_TARGET) -- -D warnings
	$(CARGO) clippy -p quire-eval --lib --target $(NO_STD_TARGET) -- -D warnings

# FR-400-AC-6: the fixture under `--cfg mint_probe` calls a banned mint, so
# clippy MUST fail on it with "disallowed method"; the target passes only then.
mint-probe:
	@out=$$(RUSTFLAGS="--cfg mint_probe" $(CARGO) clippy -p quire-checked --lib -- -D warnings 2>&1); \
	if echo "$$out" | grep -q "disallowed method"; then echo "mint-probe: banned mint refused"; \
	else echo "mint-probe: clippy did not refuse the banned mint"; echo "$$out"; exit 1; fi

test:
	$(CARGO) test --workspace
	$(CARGO) test --workspace --all-features

build-no-std:
	$(CARGO) build -p quire-checked --no-default-features --target $(NO_STD_TARGET)
	$(CARGO) build -p quire-eval --target $(NO_STD_TARGET)

deny:
	$(CARGO) deny check

# cargo doc is a separate lint pass: rustdoc-only lints fire here, not in clippy.
docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps --all-features

clean:
	$(CARGO) clean

ci: fmt-check lint mint-probe test build-no-std deny docs
