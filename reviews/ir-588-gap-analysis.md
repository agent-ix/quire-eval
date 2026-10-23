---
id: SR-12355
title: "Gap analysis of quire-eval PR #4: Value-family runtime causes and modulo (IR-588 S4)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-eval@012eca45b7f6239f8b1647040722f8c91e37b56d; PR #4 diff origin/main...HEAD; acceptance criteria FR-403-AC-1, FR-403-AC-2 (modulo half), FR-403-AC-11 (quire-eval half) and the FR-403 S4 disposition table (QSL origin/main spec/functional/FR-403-implement-the-no-std-evaluator-in-quire-eval.md)"
review_set: subset
---
# Gap analysis of quire-eval PR #4

## Summary

Ticket: IR-588. Plan completion: not assessed. The specs live in
quire-spec-language, so this AC-to-test check is manual, as in SR-12339.

- FR-403-AC-2, `modulo` half: `tests/integer.rs` evaluates `-7 mod 2`
  through `quire_eval::modulo` and `quire_exact::modulo` on fresh meters and
  compares the outcome and the `work_units` counter. It does not compare the
  ordered admitted charges the AC names (FND-001). The `divide` half and the
  "fails to compile at the old path" clause are out of this PR: `divide`
  waits on FR-400's move of `AdmittedIntegerDivision`, and the old path is
  QSL's.
- FR-403-AC-11, quire-eval half: `tests/cause.rs` checks the `state-model`
  prefix (literal and SV constant), the `AbsentKey` reason, and the `binding`
  and `key` fields with a non-UTF-8 key rendered by `render_identity`.
  Covered. The QSL half (`FamilyKind` returns SV's constant, `qsl-eval`
  defines none of the causes) belongs to the QSL PR in the stack.
- FR-403 table, the causes row ("each catalog code, prefix, field set and
  rendering has one definition"): `ProtocolReferenceUnsupported` and
  `ReferenceUnresolved` code and `construct` field are tested.
  `IncompleteClosure` and `OnlyNotOne` fields are partly tested.
  `PreconditionFalse` is not tested at all (FND-002).
- FR-403-AC-1: unchanged by this PR; the dependency set is still within
  FR-401's. No new gate check is owed here.
- No production code without an owning requirement: every added item is a
  row of FR-403's S4 table. No stubs. The tests assert real values, not
  tautologies.

## Verdict

Two low gaps, both cheap test additions. Nothing blocks merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-403-AC-2 requires the same "value and admitted charges" as `quire_exact::modulo` called directly. The test compares only `consumed(WorkUnits)`, not the ordered `Meter::admitted_charges()` that the dev-dependency's `test-support` feature exposes (scheduler.rs already uses it). It also never asserts the Euclidean value `1`. Fix: `assert_eq!(here.admitted_charges(), direct.admitted_charges())` and assert the completed value is `1`. | crates/quire-eval/tests/integer.rs:19-24 |
| FND-002 | low | `StateModelUndefined::PreconditionFalse` has no test. Its `receiver` field is the one rendering this PR rewired, from `identity_string` to SV's `render_identity` over the object reference's bytes, and no test checks it. `IncompleteClosure`'s `queried_type` is not asserted either. Fix: one test building a `PreconditionFailure`, asserting `UndefinedReason::PreconditionFalse` and the `operation`, `selected` and `receiver` fields. | crates/quire-eval/src/cause.rs:121-131; crates/quire-eval/tests/cause.rs |
