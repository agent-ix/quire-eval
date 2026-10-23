---
id: SR-16305
title: "Gap analysis of quire-eval PR #5: DeclarationKey and AdmittedIntegerDivision into quire-checked (IR-586 S2 PR A)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-eval@5bcb5dfc8143a32dbb3c089ef8917724f5eeca4a; PR #5 own diff 012eca4..5bcb5df; acceptance criteria FR-400-AC-1, AC-3, AC-4, AC-5, AC-6, AC-9 and FR-400 Behavior (Crate and dependencies; What moves; Checking authority rules 1-5) on QSL origin/main spec/functional/FR-400-separate-checked-terms-from-the-checker.md; FR-403 S4 table rows; FR-060 T12-G; FR-059 rule"
review_set: subset
date: '2026-10-23'
---
# Gap analysis of quire-eval PR #5

## Summary

Ticket: IR-586 (S2 PR A). Plan completion: not assessed (planless). I
checked the FR-400 acceptance criteria this repository can satisfy, then
judged whether the QSL-side checks the brief names (FR-060 T12-G, TC-390,
FR-059's construct rule, arch-lint, xtask, crate map) belong to this move.

Covered in this repository, and clean:

- **FR-400-AC-1, first sentence (manifest).** The dependency set is
  unchanged. No QSL crate is named.
- **FR-400-AC-4, quire-eval clause.** The default-features doctest on
  `ConstructOff` shows that a `construct` call does not compile with the
  feature off. Verified with a probe: E0433. The struct-literal guard
  for `AdmittedIntegerDivision` has its `no_run` pair (the `profile()`
  reader). The construct round trip is tested under `--all-features`.
- **FR-400-AC-6.** `make mint-probe` still refuses the banned mint. The new
  code calls no `from_digest`.
- **FR-400-AC-9, first clause.** quire-checked builds for
  `thumbv7em-none-eabi` with default features off. Unsafe code is still
  forbidden.
- **FR-400 rule 3 (fixture).** `DeclarationKey::fixture` is behind
  `test-support`, and its test runs in the all-features lane.
- **FR-403, S4 table row 306.** `DeclarationKey` is in C, as the row
  requires. Row 307 (`divide`/`modulo` move to quire-eval over C's
  `AdmittedIntegerDivision`) is now unblocked. It is S4 (IR-588) work and
  is not in this PR.

QSL-side items (not in this repository):

- FR-400-AC-5 (old `qsl_semantics::model::key::DeclarationKey` path does not
  resolve) belongs to QSL #1384. Its body says the removed paths are pinned
  by compile_fail doctests. Not verified here.
- FR-400-AC-1's second sentence (TC-390's `construct` table check), AC-3
  (T12-G and FR-059's FB-05 construct rule) and Checking authority rules 3
  and 4 are QSL tree checks. QSL #1384's body says they are not done. See
  FND-001.
- Crate map, `tools/arch-lint` and `xtask` updates are also QSL-side. I did
  not measure whether adding `quire-checked` as a QSL dependency needs edits
  to arch-lint or xtask allow lists. QSL #1384's own gate settles that.

## Verdict

Within this repository, the move is complete for the two types. One medium
gap, owned by the QSL half: FR-400 makes T12-G, the TC-390 `construct`
table check and FR-059's `construct` rule part of this move, not a later
one. They should land in QSL #1384. This does not block quire-eval#5 on its
own, but IR-586 S2 PR A is not done until they land. One low trace finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | This PR adds the first `construct` constructor (`admitted_integer_division`), and QSL #1384 enables `construct` in qsl-semantics. Cargo feature unification then turns it on for every QSL crate that depends on quire-checked (qsl-eval, qsl-package, qsl-replay, qsl-analyze, qsl-bench). Any of them can mint an `AdmittedIntegerDivision` and compile. FR-400 Checking authority rule 3 names this type explicitly: T12-G SHALL allow `construct` calls "from `value::definition` for `AdmittedIntegerDivision`, with no debt list". AC-1 and AC-3 also require TC-390's `construct` table check and FR-059's FB-05 construct rule. These guards make the checking-authority claim true in QSL. They are part of this move, not "not part of these two types" as QSL #1384's body says. The quire-eval side cannot supply them. Fix in QSL #1384: add T12-G (allow `check` and its descendants, `library`, and `value::definition` only), the TC-390 construct-enable table check with its failing fixture, and the FR-059 FB-05 rule with its RT fixture. | crates/quire-checked/src/lib.rs:21-33 |
| FND-002 | low | `tests/values.rs` does not name the requirements its tests back: FR-400 rule 3 for the fixture test, and Checking authority rule 2 / AC-4 for the construct round trip. The repository's other test modules name their ACs in the module doc (`tests/integer.rs` FR-403-AC-2, `tests/cause.rs` FR-403-AC-11). Fix: name the ACs in the module doc. | crates/quire-checked/tests/values.rs:1-3 |
