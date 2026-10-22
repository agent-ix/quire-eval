---
id: SR-12339
title: "Gap analysis of quire-eval PR #1: quire-checked skeleton (IR-586 S2 step 1)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-eval@e216c915f38eefef05ca896178e672ac1394b4dc; PR #1 diff origin/main...HEAD; acceptance criteria FR-400-AC-1, AC-4, AC-6, AC-9 (QSL origin/main spec/functional/FR-400-separate-checked-terms-from-the-checker.md) and FR-401 rule 2"
review_set: subset
---
# Gap analysis of quire-eval PR #1

## Summary

Ticket: IR-586. Plan completion: not assessed. `quoin matrix` refuses this
repository with "no document root at .../spec": the specs live in
quire-spec-language, so this AC-to-test check is manual.

The PR is step 1 of S2: an empty crate. Only the criteria a skeleton can
already meet were examined. AC-2, AC-3, AC-5, AC-7 and AC-8 are QSL-side or
need moved types. AC-4's quire-eval doctest needs a `construct` function,
which a later step adds.

- FR-400-AC-6: the clippy.toml configuration is present and enforces the ban
  (see SR-12338). No test in the gate checks it, so it can regress silently
  (FND-001).
- FR-400-AC-9: `make build-no-std` builds thumbv7em with default features
  off, and `ci` runs it. The manifest forbids unsafe code. Covered.
- FR-400-AC-1, first sentence: the manifest meets it. Nothing in this repo's
  gate asserts it (FND-002). The `construct` half of AC-1 is QSL's TC-390.
- No production code without an owning requirement: the only item, `pub mod
  construct {}`, is FR-400 Checking authority rule 2.
- No stubs pretending to work, and no coverage inflation. There are no
  tests.

## Verdict

One medium gap: AC-6 has a working configuration but no test, while the
criterion's verification method is Test and names a failing fixture. One
low gap on AC-1. Both are cheap to add in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-400-AC-6 requires that "a fixture call to `EffectiveId::from_digest` in the crate fails that clippy run". No such fixture or negative check is in the repo or in `make ci`. The PR body says it was checked once, locally. If clippy.toml's paths stop resolving (quire-exact moves or renames a type), clippy's disallowed-method rule goes quiet and the gate still passes. Fix: add a `#[cfg(mint_probe)]` fixture fn calling `EffectiveId::from_digest` (declare the cfg under `[lints.rust] unexpected_cfgs check-cfg`). Add a Makefile target, wired into `ci`, that runs `RUSTFLAGS='--cfg mint_probe' cargo clippy -p quire-checked -- -D warnings` in its own target dir and passes only when that run fails with `disallowed method`. | clippy.toml:8-12; Makefile:39 |
| FND-002 | low | FR-400-AC-1's first sentence (the manifest names only SV, K, W, quire-canonical, serde and thiserror, each with default features off, and no QSL crate) has no check in this repo's gate, and nothing outside QSL's TC-390 owns it. Fix: a `make` check over `cargo tree --locked -p quire-checked -e normal --depth 1 --prefix none` that compares against the six names, or fold it into FND-001's target. | crates/quire-checked/Cargo.toml:26-32; Makefile:39 |
