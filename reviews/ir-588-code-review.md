---
id: SR-12354
title: "Code review of quire-eval PR #4: Value-family runtime causes and modulo (IR-588 S4)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-eval@012eca45b7f6239f8b1647040722f8c91e37b56d; PR #4 diff origin/main...HEAD (Cargo.lock, crates/quire-eval/Cargo.toml, crates/quire-eval/src/cause.rs, crates/quire-eval/src/integer.rs, crates/quire-eval/src/lib.rs, crates/quire-eval/tests/cause.rs, crates/quire-eval/tests/integer.rs); against QSL origin/main FR-403, FR-401, FR-400, NFR-015; rust-review lane folded in"
review_set: subset
---
# Code review of quire-eval PR #4

## Summary

Ticket: IR-588. Rust-review lane folded in. Stacked on
quire-semantic-value#62 (reviewed separately). Specs read at
agent-ix/quire-spec-language origin/main: FR-403 (Where the implementation
lives, the S4 disposition table, AC-1, AC-2, AC-11), FR-401 (dependency set,
no `std` feature), FR-400 (moved value types).

Checked and clean:

- Faithful move. `cause.rs` matches `qsl-eval/src/value/expression/causes.rs`
  on QSL origin/main item for item: the same catalog codes, cause tags, field
  keys and renderings for `PreconditionFailure`, `StateModelUndefined` (four
  variants), `ProtocolReferenceUnsupported` and `ReferenceUnresolved`. The
  only changes are `pub(crate)` to `pub`, `std::` to `core::`/`alloc::`, the
  prefix read from SV's `STATE_MODEL_CATALOG_PREFIX` instead of
  `FamilyKind::StateModel.catalog_code_prefix()`, and `identity_string`
  replaced by SV's `render_identity` (the FR-403 table rows for both).
- `modulo` is the same pass-through of `quire_exact::modulo` that
  `qsl_semantics::value::definition::modulo` is on QSL origin/main. It is a
  defined function, not a `pub use` re-export (ADR-011 §7.2).
- The deferral holds. On QSL origin/main, `divide` takes
  `qsl_semantics::value::definition::AdmittedIntegerDivision` and
  `NavigationRefusal` carries `qsl_semantics::model::key::DeclarationKey`.
  FR-400 lists both types as moving to `quire-checked`, and
  `crates/quire-checked/src/lib.rs` here is still the 27-line skeleton
  (an empty `construct` module and the mint probe). `quire-eval` must name no
  QSL item (FR-401), so neither can move yet. `modulo` takes no
  `AdmittedIntegerDivision`, so moving it alone is consistent with FR-403's
  table. FR-403-AC-2's `divide` half stays open until FR-400's move lands.
- `no_std` + `alloc`, no `std` feature. The crate declares no `[features]`.
  `cargo clippy -p quire-eval --lib --target thumbv7em-none-eabi -D warnings`
  passes.
- Dependency set unchanged. `cargo tree -p quire-eval -e normal --depth 1` is
  quire-exact, quire-semantic-value and thiserror, all with default features
  off (FR-401/FR-403 item 2). Only the SV branch changed (FND-001).
- `cargo test -p quire-eval --locked` (30 tests),
  `cargo clippy -p quire-eval --all-targets --all-features -D warnings`,
  `cargo fmt --all --check`, `RUSTDOCFLAGS=-D warnings cargo doc -p
  quire-eval` and `cargo deny check sources bans` all pass.
- Rust idioms: no `unsafe`, no panic path, no narrowing integer conversion
  (`found: u64` is only rendered), and every field map is a `BTreeMap`, so
  field order is deterministic (FR-403 item 2).
- No vendoring. Nothing is copied from another repo. The causes moved here
  by FR-403's own disposition, and the QSL PR in the stack deletes the
  originals (FR-403-AC-11's QSL half).

## Verdict

Mergeable once FND-001 is done: repoint `quire-semantic-value` to
`branch = "main"` after quire-semantic-value#62 merges, and before this PR
merges. The PR body already states that step. FND-002 is low and can be fixed
in this PR or left.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `quire-semantic-value` is taken from `branch = "ir-588-s4-sv"`. Cargo.lock now resolves two SV copies: quire-checked on `main`, quire-eval on the PR branch. If this merges as-is and #62 is squash-merged and its branch deleted, a fresh fetch of the branch fails and main stops building. Fix: after #62 merges, set `branch = "main"` and refresh Cargo.lock, so the lock holds one SV entry, then merge. | crates/quire-eval/Cargo.toml:17; Cargo.lock:148,159,178-191 |
| FND-002 | low | `PreconditionFailure` is a public struct with all-`pub` fields and no constructor. CG and RT can then build one with any `selected` string. This matches the old crate-internal shape, so behaviour is unchanged. Optional fix: keep the fields `pub` but add `#[non_exhaustive]`, so a later added field (for example the call locus) is not a breaking change for CG. | crates/quire-eval/src/cause.rs:19-28 |
