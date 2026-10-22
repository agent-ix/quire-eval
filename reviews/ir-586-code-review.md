---
id: SR-12338
title: "Code review of quire-eval PR #1: quire-checked skeleton (IR-586 S2 step 1)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-eval@e216c915f38eefef05ca896178e672ac1394b4dc; PR #1 diff origin/main...HEAD (.gitignore, CLA.md, CONTENT_RIGHTS.md, Cargo.lock, Cargo.toml, LICENSE, Makefile, clippy.toml, crates/quire-checked/Cargo.toml, crates/quire-checked/src/lib.rs, deny.toml, rust-toolchain.toml, rustfmt.toml); against QSL origin/main FR-400, FR-401, NFR-015, AD-001; rust-review lane folded in"
review_set: subset
---
# Code review of quire-eval PR #1

## Summary

Ticket: IR-586. Rust-review lane folded in. Specs read at
agent-ix/quire-spec-language origin/main: FR-400 (Crate and dependencies,
Checking authority, AC-1, AC-4, AC-6, AC-9), FR-401 rule 2 (licence, CLA,
content rights), NFR-015 (feature contract, target, toolchain), AD-001 S2.

Checked and clean:

- Dependency set. `cargo tree -p quire-checked -e normal --depth 1` lists
  exactly quire-canonical, quire-exact, quire-semantic-value, quire-walk,
  serde and thiserror. Cargo.lock holds no QSL crate. With
  `--no-default-features --target thumbv7em-none-eabi`, the feature tree
  enables no `std` feature anywhere (FR-400 rule 1).
- serde (`derive`, `alloc`), thiserror and quire-canonical are declared with
  default features off. SV, K and W declare no `default` feature at their
  locked commits, so nothing pulls `std` today.
- `#![no_std]` + `extern crate alloc`, `std` default-on, `unsafe_code =
  "forbid"` through `[lints] workspace = true` (FR-400 rule 2, AC-9).
  `cargo build --locked -p quire-checked --no-default-features --target
  thumbv7em-none-eabi` succeeds.
- `construct` and `test-support` are empty features; `construct` gates an
  empty public module. The public API is that one module. Nothing else is
  exposed, so CG gets a clean surface to build on.
- clippy.toml `disallowed-methods` enforces FR-400-AC-6. A probe calling
  `NodeKey::from_digest`, `EffectiveId::from_digest` and
  `PopulationId::from_digest`, plus a function-pointer reference to
  `EffectiveId::from_digest`, was refused 4/4 under both default features and
  `--no-default-features --target thumbv7em-none-eabi`. The paths resolve
  through quire-exact's root re-exports. The probe was reverted and not
  pushed.
- `cargo clippy --all-targets --all-features -D warnings`, `cargo fmt
  --check`, `cargo deny check` (licenses, bans, advisories and sources ok) and
  `RUSTDOCFLAGS=-D warnings cargo doc --all-features` all pass.
- deny.toml `allow-git` lists exactly the four git dependencies, with
  `unknown-git = "deny"`.
- .gitignore has unanchored `target/`, `*-target/`, `target-*/` and
  `.worktrees/`. No build artifacts are tracked.
- rust-toolchain.toml pins 1.98.1 with the thumbv7em target (NFR-015).
- LICENSE, CLA.md and CONTENT_RIGHTS.md are byte-identical to quire-walk's.
  FR-401 rule 2 requires the repository to carry them. A per-repo AGPL
  LICENSE and CLA.md are needed in the repo itself: GitHub licence detection,
  the AGPL distribution terms and the CLA workflow all read them there. This
  is org-standard boilerplate, not vendored code, and is accepted. FND-006
  covers one false claim inside CONTENT_RIGHTS.md.
- No pins or ceremony beyond what FND-005 names. The git dependencies use
  `branch = "main"`, and Cargo.lock is the repeatable-build record.

## Verdict

Approve with findings. There are no high findings. Fix FND-001 (a cheap
one-line lint level) before merge, because AC-6's guarantee is the point of
clippy.toml. FND-002 to FND-006 are low. Fix them in this PR or in the next
IR-586 step. FND-007 needs Peter's clearance for a workflow file and only
matters before the repo goes public.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-400-AC-6's mint ban can be bypassed by a local `#[allow(clippy::disallowed_methods)]`. The workspace sets `clippy::all = "deny"`, which an inner `allow` overrides. A probe fn with `#[allow(clippy::disallowed_methods)]` calling `EffectiveId::from_digest` passed `cargo clippy -D warnings` cleanly. Fix: add `disallowed_methods = { level = "forbid", priority = 1 }` to `[workspace.lints.clippy]`. With it, the same probe fails with E0453 "allow(clippy::disallowed_methods) incompatible with previous forbid" (verified). | Cargo.toml:22-23; clippy.toml:8-12 |
| FND-002 | low | FR-400-AC-1 says the manifest names each dependency "with default features off". quire-semantic-value, quire-exact and quire-walk are declared without `default-features = false`. None has a default feature today, so the build is unaffected. But a literal manifest check fails, and a future `default = ["std"]` in SV, K or W would silently turn `std` on. Fix: add `default-features = false` to the three entries. | crates/quire-checked/Cargo.toml:27-29 |
| FND-003 | low | `make lint` never runs clippy with `std` off. quire-checked has a `std` feature, so unused imports and dead `cfg` code that exist only in the `no_std` configuration go unlinted. `build-no-std` builds that configuration but does not lint it. quire-walk's lint runs a thumbv7em clippy pass, and NFR-015's gate rationale asks for one. Fix: add `$(CARGO) clippy -p quire-checked --no-default-features --target $(NO_STD_TARGET) -- -D warnings` to `lint`. | Makefile:18-20 |
| FND-004 | low | The `--all-features` test lane enables `construct`. FR-400-AC-4's `compile_fail` doctest (a `construct` call failing with default features) would then compile, and `make test` would fail once that doctest lands. Fix, when AC-4's doctest is added: gate it with `#![cfg_attr(not(feature = "construct"), doc = ...)]`, or run the doctest only in the default-features lane. | Makefile:22-24; crates/quire-checked/Cargo.toml:20-22 |
| FND-005 | low | rustfmt.toml sets `imports_granularity` and `group_imports`, which are nightly-only. On the pinned stable 1.98.1 they do nothing, and every `cargo fmt` run prints "unstable features are only available in nightly channel". Fix: delete the two lines. | rustfmt.toml:4-5 |
| FND-006 | low | CONTENT_RIGHTS.md says "The project preflight rejects unreviewed file types by default". No preflight exists in this repo (none in quire-walk either), so the document claims a control that is not there. Fix: drop that paragraph here, and correct the org template that new-project copies. | CONTENT_RIGHTS.md:26-28 |
| FND-007 | low | FR-401 rule 2: the repo carries "the CLA ... controls that agent-ix/quire-walk carries", and its CLA check must pass before it goes public. quire-walk's CLA control includes `.github/workflows/cla.yml`, which wires the org CLA assistant. This PR copies CLA.md but not the workflow. Fix: add cla.yml before the visibility flip. A workflow edit needs Peter's clearance. | CLA.md; (absent) .github/workflows/cla.yml |
