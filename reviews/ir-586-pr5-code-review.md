---
id: SR-16304
title: "Code review of quire-eval PR #5: DeclarationKey and AdmittedIntegerDivision into quire-checked (IR-586 S2 PR A)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-eval@5bcb5dfc8143a32dbb3c089ef8917724f5eeca4a; PR #5 own diff 012eca4..5bcb5df (S4 commit excluded, already squash-merged to main as a92145c): Cargo.lock, crates/quire-checked/{Cargo.toml,src/lib.rs,src/key.rs,src/integer_division.rs,tests/values.rs}; against QSL origin/main FR-400, FR-403, FR-401, NFR-015, FR-060, FR-059 and qsl-semantics/src/model/key.rs, qsl-semantics/src/value/definition.rs; rust-review lane folded in"
review_set: subset
date: '2026-10-23'
---
# Code review of quire-eval PR #5

## Summary

Ticket: IR-586 (S2 PR A, quire-eval side). Rust-review lane folded in.
QSL-side pair: agent-ix/quire-spec-language#1384 (not reviewed here).

The PR base `ir-588-s4-eval` is stale: S4 merged to main as a92145c (squash).
I reviewed only this PR's own two commits (7c7386e, 5bcb5df) and ignored the
S4 commit 012eca4 on the branch.

Checked and clean:

- **Dependencies (FR-400 rule 1, AC-1).** The manifest's normal dependencies
  are unchanged: SV, K, W, quire-canonical, serde and thiserror, each with
  default features off. `DivisionProfile` comes from `quire-exact` only. No
  QSL crate is named.
- **`DeclarationKey` ordering.** Compared with QSL origin/main
  `qsl-semantics/src/model/key.rs`: the same fields in the same order
  (`package`, then `node`), the same derives (`Clone Debug Eq Hash Ord
  PartialEq PartialOrd`) and `pub` fields. `derive(Ord)` order, and so the
  "package then node, prefix first" comparison, is unchanged. The ordering
  test asserts it.
- **`fixture`** is behind `#[cfg(any(test, feature = "test-support"))]`, as in
  the original (FR-400 rule 3).
- **What stays in QSL.** The only `DeclarationKey` method left behind is
  `wire()`, the preimage view over `SHA256_JCS_DIGEST_DOMAIN`. `identity()`
  and `validate_derivation()` are methods of `EffectiveDeclarationPreimage`
  (they mint an `EffectiveId` and return `ModelRefusalCause`). Keeping them in
  `model` is correct under FR-400 "What moves".
- **`AdmittedIntegerDivision`.** The derives (`Clone Copy Debug Eq Hash
  PartialEq`) and the `profile()` reader match the QSL original. The field is
  `pub(crate)`, so it is private to the crate (rule 1). The only constructor
  is `construct::admitted_integer_division`, under `#[cfg(feature =
  "construct")]` (rule 2). No `&mut` accessor.
- **Forgery outside `construct`.** A probe crate depending on quire-checked
  by path confirmed each failure reason. The struct literal fails with
  E0451 (private field), with and without `construct`. The field write fails
  with E0616. The `construct` call with the feature off fails with E0433
  (`construct` not found). With `--features construct` it compiles. The
  cfg-gated compile_fail doctest on `ConstructOff` runs in the default lane
  (3 doctests) and is absent from `--all-features` (2 doctests), as its doc
  says.
- **no_std / alloc.** `#![no_std]` and `extern crate alloc` are unchanged.
  `key.rs` uses `alloc::string::String`. clippy (`--lib
  --no-default-features --target thumbv7em-none-eabi`) and the thumbv7em
  build pass. quire-eval gains no `std` feature: this PR does not touch
  quire-eval.

Gates run (focused), on 7c7386e cherry-picked onto origin/main a92145c (the
pick is clean): fmt-check; `cargo test -p quire-checked` (2 tests and 3
doctests) and `--all-features` (4 tests and 2 doctests); clippy
`--all-targets` with default features and with `--all-features`; no_std
clippy and build; `make mint-probe`; `cargo doc --all-features` with `-D
warnings`. All passed.

## Verdict

**Not mergeable as it stands.** FND-001 is mechanical but blocks the merge.
GitHub reports the PR as CONFLICTING, and its second commit pins
quire-checked to a feature branch of SV that main no longer uses. Drop
5bcb5df, rebase onto main and retarget the base to main. Also fix FND-002
(pin the compile_fail error codes) in this PR. After that, the code is
correct and the move is faithful. FND-003 and FND-004 are low. Merge order:
this PR first, then QSL #1384 repointed at `branch = "main"`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Cannot merge as written. The base `ir-588-s4-eval` is already squash-merged (a92145c) and GitHub reports CONFLICTING. Commit 5bcb5df sets quire-checked's `quire-semantic-value` to `branch = "ir-588-s4-sv"` and rewrites Cargo.lock to that branch, but main now has SV at `branch = "main"` (472e609) for quire-eval. Merging the branch, or resolving the conflict by keeping 5bcb5df, would put quire-checked on a feature branch. Fix: drop 5bcb5df, rebase 7c7386e onto origin/main (it cherry-picks cleanly, and every focused gate above passed on the result) and retarget the PR base to main. | crates/quire-checked/Cargo.toml:27; Cargo.lock:145-190 |
| FND-002 | medium | Both compile_fail doctests are bare `compile_fail` and do not pin the error they guard. Any compile error passes them: a renamed `DivisionProfile` variant, a moved `quire_exact` path, or a typo. FR-400 rule 5 requires each guard to "keep failing to compile" for its stated reason. The QSL originals this crate inherits (`check/ir.rs`, `check/mod.rs`, `check/state_clause.rs`) pin codes with `compile_fail,E0xxx`. Today both fail for the right reason (probe: E0451 and E0433). Fix: change the struct-literal doctest to `compile_fail,E0451` and the construct-off doctest to `compile_fail,E0433`. | crates/quire-checked/src/integer_division.rs:15, 45 |
| FND-003 | low | `ConstructOff` is a public (`doc(hidden)`) unit struct that exists only to carry a doctest, and only when `construct` is off. The crate's public surface therefore differs by feature. Fix: put the doctest on existing docs with `#[cfg_attr(not(feature = "construct"), doc = "...")]`, for example on the `integer_division` module, and delete the item. | crates/quire-checked/src/integer_division.rs:41-52 |
| FND-004 | low | The `key.rs` doc names `DeclarationKey::identity` and `validate_derivation` as the model stage's functions over this key. Neither is a `DeclarationKey` method on QSL origin/main: both are `EffectiveDeclarationPreimage` methods. What the move leaves behind for this type is the preimage view `wire()`, which becomes `model::key::declaration_key_wire` in QSL #1384. A reader looking for `DeclarationKey::identity` finds nothing. FR-400's "What moves" paragraph has the same error (spec point below). Fix: name `declaration_key_wire` and `EffectiveDeclarationPreimage::{identity, validate_derivation}`. | crates/quire-checked/src/key.rs:17-19 |

## Spec points

- FR-400 "What moves", last paragraph, says "`DeclarationKey::identity`,
  which mints an `EffectiveId`, and `DeclarationKey::validate_derivation`".
  On QSL origin/main both are `EffectiveDeclarationPreimage` methods. Only
  `DeclarationKey::wire` is a method of the moved type. Correct the sentence
  to name the real items.
