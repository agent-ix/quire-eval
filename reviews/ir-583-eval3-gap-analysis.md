---
id: SR-12353
title: "Gap analysis of quire-eval PR #3: scheduler aligned with merged FR-262 (IR-583 S0)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-eval@942a1b4efd65baf363f66a0a1266cfa28d12778f; acceptance criteria FR-262-AC-7, AC-10, AC-11 (QSL origin/main 066f5a1c9 spec/functional/FR-262-evaluate-values-and-calls-at-any-depth.md) against crates/quire-eval/tests/scheduler.rs and tests/support/mod.rs; reviewed together with quire-spec-language PR #1269"
review_set: subset
date: '2026-10-22'
---
# Gap analysis of quire-eval PR #3

## Summary

Ticket: IR-583 (S0). quire-eval has no spec of its own. Its criteria live in
QSL, so `quoin matrix` over quire-eval reports nothing. I mapped the
scheduler tests to the FR-262 criteria this PR changes, by hand. Plan
completion: not assessed.

Examined and clean:

- **FR-262-AC-7, first clause.** `cancelling_after_a_request_is_saved_runs_no_further_charge_or_body`
  asserts the located incomplete result at the child's `function.call`:
  `limit_kind`, charge point, `limit == consumed`, the request's call site,
  no recorded charge, no child body, and `tripped() == Requested`. The
  refusal-child, undefined-child and drop clauses have their own tests.
- **FR-262-AC-10, first two clauses.** The kernel-charge case
  (`cancellation_observed_by_the_root_bodys_last_charge_is_an_incomplete_result`)
  and the `function.call` case (`a_charge_denied_by_cancellation_is_an_incomplete_result_not_a_call_failure`)
  both assert the incomplete form and `tripped()`. Begin takes no `Cancel`.
- **FR-262-AC-11, unknown-callee and arity runs.** Both assert stage
  `quire-eval` and invariant `child-request-not-admitted`, and that only the
  root call was charged.

## Verdict

Mergeable on coverage once SR-12352's trait findings are fixed. The gaps
below are missing branches of criteria the PR rewrote. Fix them in this PR,
alongside FND-001 of SR-12352, which adds the storage arm these tests need.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-262-AC-11 covers two of its six runs. The toy `admits_request` checks only that the callable is admitted and the arity, so no test reaches a membership failure: an argument out of its parameter's range (`f` with `10`), a `Population` whose id the environment does not resolve, a `Population` resolving to the wrong declared maximum, or the sixth run, which must apply with one `function.call`. No test covers the storage outcome either (see SR-12352 FND-001). Fix: give the toy one ranged parameter and one undecidable-membership case, and add the out-of-range, wrong-maximum and storage runs. | crates/quire-eval/tests/support/mod.rs:230-232; crates/quire-eval/tests/scheduler.rs:487-516, 592-620 |
| FND-002 | medium | FR-262-AC-10's third clause is untested: "When a caller cancels between two steps and the next transition makes no charge, that transition runs and the session terminates at the first charge after it." Removing the before-transition poll is exactly what makes this clause true, so it needs a test: cancel between steps where the next transition is a chargeless `Continue` or child delivery, then assert that the transition ran and the result is the incomplete result at the following charge. | crates/quire-eval/src/session.rs:127-165; crates/quire-eval/tests/scheduler.rs |
| FND-003 | medium | Nine test doc comments say "the reference adapter in qsl-eval carries the criterion itself". That is false at QSL #1269's head: `quoin matrix` over QSL reports every FR-262 criterion, AC-1 to AC-11, as `untagged`, and no qsl-eval test exercises the adapter on AC-4, AC-6, AC-7, AC-10 or AC-11 (SR-12355 FND-002). The comment tells a reader the criterion is proven elsewhere when it is not. Fix: drop the claim, or name the QSL test once it exists. | crates/quire-eval/tests/scheduler.rs:78, 129, 170, 214, 245, 276, 312, 341 |
