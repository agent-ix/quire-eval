---
id: SR-12347
title: "Gap analysis of quire-eval PR #2: quire-eval evaluation interface (IR-583 S0)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-eval@11d8af39a997f6b9b7e6d470fafdc293f51e612c (reviewed rebased onto origin/main 57712bd); acceptance criteria FR-262-AC-1, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8 and FR-401-AC-1, AC-3, AC-5, AC-6 (QSL origin/main 48108c9c1 spec/functional/FR-262-evaluate-values-and-calls-at-any-depth.md, FR-401-host-the-evaluation-interface-in-the-quire-eval-leaf.md); FR-401 S0 slice scope; reviewed together with quire-semantic-value PR #61"
review_set: subset
date: '2026-10-22'
---
# Gap analysis of quire-eval PR #2

## Summary

Ticket: IR-583 (S0). Plan completion: not assessed. The specs live in
quire-spec-language and this repository has no `spec/`, so `quoin matrix`
has nothing to read here. This AC-to-test check is manual. The tests carry
`/// Trace:` doc lines. No tool reads them.

- **FR-401-AC-1.** Covered by inspection and by `make build-no-std`. The
  manifest dependencies are exactly SV, K and thiserror, with defaults off.
  The crate root is `#![no_std]`, there is no QSL crate, and the workspace
  lints forbid unsafe code. The thumbv7em build succeeds.
- **FR-401-AC-6.** Met. Begin takes the registry and no separate closure.
  The source names no QSL item.
- **FR-401-AC-3, quire-eval half.** The five result types resolve at
  `quire_eval::`. The "old paths fail to compile" half needs the QSL
  repoint (FND-004).
- **FR-262-AC-3.** Covered by tests/registry.rs: the order, refusal of the
  duplicate, failed vs successful Begin, the sealed registry keeping A, and
  no factory call or charge during registration.
- **FR-262-AC-1, AC-4, AC-6, AC-7.** The scheduler properties are covered
  through the toy closure: depth with a 512 KiB stack, `w` / `w - 1` / `w`,
  mixed host/host/interpreted stepping matching `run`, one resume per caller,
  cancel after a saved request, and drop. FND-001 and FND-003 cover what
  these tests do not show.
- **Production code with no owning requirement.** None. Every public item is
  in FR-401's placement table or FR-262's operation table.

## Verdict

Request changes, together with SR-12346. FND-002 must be fixed in this PR:
it is the test that would have caught SR-12346 FND-001. FND-004 needs a
Linear decision before merge. IR-583 is marked Done, yet FR-401 S0's QSL
half has no ticket. FND-001 and FND-003 are medium test gaps. FND-005 is low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Several tests claim criteria they cannot meet. FR-262-AC-1 names the QSL source function `count` through the real evaluator. FR-401-AC-5 requires "the reference adapter over a checked package" with `f`, `g` and an FR-151 dispatch candidate. The tests tagged `Trace: FR-262-AC-1` and `Trace: FR-262-AC-3, FR-401-AC-5` run a toy closure. They prove the scheduler analogue, which is worth keeping, but a reader of the tags would think AC-1 and FR-401-AC-5 are covered before the reference adapter exists. Fix: drop `FR-401-AC-5` from the registry-order test. Mark the toy tests as the interface-level half, and leave the AC tag to the adapter tests in qsl-eval. | crates/quire-eval/tests/registry.rs:22; crates/quire-eval/tests/scheduler.rs:78 |
| FND-002 | medium | FR-262-AC-8 has no test: "A body performs charged work, saves a local, requests a child and continues with its returned value ... including a kernel loss record". No test has a body or frame charge the meter at all, because the toy `advance` ignores `_meter` and every host body ignores its meter. That is the untested path where SR-12346 FND-001 lives. Fix: add a host body that charges the meter, saves a local and continues with the child's value. Assert that each transition happens exactly once and that the charge sequence is right. Add the cancellation-observed-at-a-body-charge case from SR-12346 FND-001. | crates/quire-eval/tests/support/mod.rs:200-206, 309-333 |
| FND-003 | medium | Parts of FR-262-AC-5 and AC-7 are untested. AC-5's "a source call with two argument applications charges them left to right before its own call charge" has no test, because every toy callable takes one argument. AC-7's "a child refusal" case is not tested (only `Undefined` is), and neither is "reference ... loss behavior" (every `Evaluation` has empty losses). Fix: add a two-argument interpreted application and assert the charge order. Add a `Refused` child through a direct-return host. Add a child that completes with a loss record, so the test shows whether the caller keeps it. | crates/quire-eval/tests/scheduler.rs:262-326; crates/quire-eval/tests/registry.rs:144-167 |
| FND-004 | medium | FR-401 S0 is only half delivered, and the rest is not tracked. FR-401 gives S0 (IR-583) "the reference adapter in `qsl-eval`" and the repoint of every QSL caller "in the change that moves an item, with no re-export at the old path". Linear IR-583 is in state Done, has no child or follow-up ticket for that QSL work, and these two PRs are still open. If SV#61 and this PR merge before the QSL repoint, every moved type is defined twice (SV `catalog` and `supplied_stop` vs qsl-foundation and qsl-semantics; quire-eval `Evaluation`, `CallFailure` and `FamilyOutcome` vs qsl-eval and qsl-semantics). FR-401's "defined in exactly one crate ... no copy" is then false until the repoint lands. Fix: reopen IR-583 or file the S0 QSL follow-up ticket. Land SV#61, this PR and the QSL repoint as one merge batch, so the duplicate exists only between those merges. | crates/quire-eval/src/result.rs; Linear IR-583 |
| FND-005 | low | In the AC-1 test, `denied.location == Some(site())` cannot tell which call site was denied, because every toy request uses the same `Location::root(Origin::Expression)`. The message "the deep call must not overflow the stack" on `join().is_ok()` is also misleading: a native overflow aborts the test process and never reaches the assert. The test is still sound, because the abort fails the run. Fix: give the toy a depth-specific site, and reword the message. | crates/quire-eval/tests/scheduler.rs:103, 120; crates/quire-eval/tests/support/mod.rs:58-60 |
