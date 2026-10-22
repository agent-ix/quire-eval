---
id: SR-12352
title: "Code review of quire-eval PR #3: scheduler aligned with merged FR-262 (IR-583 S0)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-eval@942a1b4efd65baf363f66a0a1266cfa28d12778f; crates/quire-eval/src/{body,closure,registry,result,session}.rs, crates/quire-eval/tests/{scheduler.rs,support/mod.rs}; against QSL origin/main 066f5a1c9 FR-262 (as merged by STD-319 #1243), FR-401, FR-276, NFR-015; rust-review lane folded in; reviewed together with quire-spec-language PR #1269 (SR-12354, SR-12355)"
review_set: subset
date: '2026-10-22'
---
# Code review of quire-eval PR #3

## Summary

Ticket: IR-583 (S0). Rust-review lane folded in. CG will depend on this crate,
so I reviewed the public trait surface as a consumer API.

**The cancellation rule, checked against the merged spec.** I read FR-262 as
it stands on QSL origin/main (last changed by b2c306eaf, STD-319, #1243). Its
"Fuel and terminal boundaries" section says the meter polls its installed
handle at every charge, and that the core observes cancellation only there.
A charge the handle denies is a denied charge like any other: the session
ends with that charge's located incomplete result, with `limit_kind:
work_units` and `limit == consumed`. The core SHALL NOT turn that result into
`CallFailure::Cancelled`. It returns `Cancelled` only as a pre-charge Begin
failure from the supplied-value run. FR-262-AC-7 and AC-10 say the same. The
earlier lead ruling, which #2 followed, is superseded. The coder followed the
merged text, and this PR is correct on that point:

- The before-transition `cancel.cause()` poll is gone, and so is the
  `finish()` conversion. `enter_call` returns `Self::incomplete(denied, site)`
  directly. A body's `Done` on an empty stack returns its evaluation
  unchanged.
- The tests now assert the located incomplete result, `limit == consumed`,
  the call site, and `tripped()`. They cover both a scheduler `function.call`
  charge (AC-7 shape) and a body's own kernel charge (AC-10 shape).
- The scheduler's fault stage is `quire-eval`, as FR-262 requires.
- Request validation runs before the child's `function.call`. A failed
  request ends with `Fault(child-request-not-admitted)`, with no child charge
  and no callee body. The new arity test covers this.

`cargo test -p quire-eval` passed (14 scheduler tests and 8 others).
`make lint build-no-std` passed. `cargo check` under `--cfg seam_probe`
passed.

The defects are both in the trait the evaluator implements. CG will depend on
this crate, so the trait should be fixed now, before any second implementor
exists.

## Verdict

**Not mergeable yet.** One high and one medium finding, both in
`CheckedClosure`. FND-001 is a contract defect. The merged FR-262 requires
validation to return `CallFailure::Storage` when membership cannot be decided
for want of storage. A `bool` return cannot express that, and the QSL adapter
turns it into a fault (SR-12354 FND-002). FND-002 is in Begin admission: it
cannot see the meter, so it cannot observe the session's cancel handle. The
cancellation rewrite is correct. Fix both trait signatures in this PR and
re-lock QSL #1269 against the result.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `CheckedClosure::admits_request` returns `bool`, so it cannot express a required outcome. FR-262, as merged, says: "A membership check that cannot decide for want of storage SHALL terminate the session with `CallFailure::Storage`, whose `parameter` is the index of the callee's parameter ... No other failure arises from validation." With `bool`, an implementation must either answer `false`, which the scheduler reports as `Fault(child-request-not-admitted)`, or `true`, which admits an undecided argument. The QSL reference adapter does the first (`types.admits(..).unwrap_or(false)`). CG will implement this trait, so fix it before it is depended on. Fix: return `Result<(), RequestRefusal>` with `NotAdmitted` and `Storage { parameter, failure: EnvironmentFailure }`. The scheduler then maps `NotAdmitted` to the fault and `Storage` to `CallFailure::Storage`, both before the child's `function.call`. Add a toy test for the `Storage` arm. | crates/quire-eval/src/closure.rs:53-66; crates/quire-eval/src/session.rs:180-188 |
| FND-002 | medium | `admit_call` and `admit_expression` take no `Meter` and no `Cancel`. Begin's supplied-value run therefore cannot poll the meter's installed handle, which FR-262's Begin row makes the session's handle. The new `CallFailure::Cancelled` doc says that arm arises during "a supplied-value phase before the call", and FR-262 says Cancelled is returned "only as a pre-charge Begin failure from the supplied-value run". Neither is reachable with the session's handle by any implementation of this trait. The QSL adapter builds its run with `Cancel::new()` (SR-12354 FND-004), so cancelling the caller's handle before Begin is never seen. Fix: pass `&Meter` (or `meter.cancel()`) to both admission methods, so the supplied-value run polls the same handle. | crates/quire-eval/src/closure.rs:38-51, 68-79; crates/quire-eval/src/registry.rs:127-160; crates/quire-eval/src/result.rs:104-109 |

## Spec points for draco

- FR-262's Begin row and "Fuel and terminal boundaries" do not say which
  handle Begin's supplied-value run polls. State that it is the meter's
  installed handle, the same one every charge polls. FND-002 is then
  testable.
