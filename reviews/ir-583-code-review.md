---
id: SR-12346
title: "Code review of quire-eval PR #2: quire-eval evaluation interface (IR-583 S0)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-eval@11d8af39a997f6b9b7e6d470fafdc293f51e612c; PR #2 reviewed as rebased onto origin/main 57712bd (Cargo.lock, Cargo.toml, Makefile, crates/quire-eval/**); built with a local cargo patch to quire-semantic-value#61 @1adf9ce; against QSL origin/main 48108c9c1 FR-262, FR-401, FR-400, FR-402, NFR-015, AD-001, and FR-276 and FR-090 as context; rust-review lane folded in; reviewed together with quire-semantic-value PR #61 (SR-12344, SR-12345)"
review_set: subset
date: '2026-10-22'
---
# Code review of quire-eval PR #2

## Summary

Ticket: IR-583 (S0). Rust-review lane folded in. CG will depend on this crate,
so I reviewed the public surface as a consumer API. I also checked the
behaviour against FR-262.

**Base.** The PR still targets `ir-586-skeleton`, and the branch still carries
the unsquashed skeleton commit e216c91. #1 has merged as 57712bd with its fix
round, so GitHub reports the PR CONFLICTING (FND-007). I reviewed the PR as
`git rebase --onto origin/main e216c91`. That rebase has one trivial conflict:
both sides add an `unexpected_cfgs` check-cfg, and both cfgs are kept. It was
not pushed.

Checked and clean:

- **FR-262 registration order.** `register` decides `RegistrySealed`, then
  `CallableNotAdmitted`, then `SignatureMismatch`, then `DuplicateBinding`,
  then `Accepted`, in exactly that order. It never invokes a factory, and a
  refusal changes no binding. `&mut self` on register and Begin gives the
  exclusive access the spec requires.
- **Sealing.** Admission runs before `seal()`. A failed Begin leaves the
  registry open, and a successful one seals it before any factory call or
  charge. Sessions share the sealed bindings through an `Arc`, and dropping
  a session cannot reopen the registry.
- **Heap stack, no depth cap.** `Session` keeps one `Vec<Frame>` and one
  pending `Entry`. `step` does constant native work per transition and never
  recurses. There is no depth counter, and fuel is the meter's `work_units`
  alone. Validation runs before `function.call`, which runs before bind and
  push. A frame push, a pop, a yield or a child delivery adds no charge.
  Each child's `Evaluation` is moved into its caller's `Resume::Child`
  exactly once.
- **The 100,000-deep test is sound.** The toy `advance` is O(1). Dropping
  100,000 frames is a `Vec` drop, which is iterative. The test runs on a
  512 KiB thread in a debug build. A native overflow would abort the test
  binary, so the test cannot pass by accident (see SR-12347 FND-005 for a
  wording nit).
- **No resume after terminal.** `step(self)` consumes the session. A terminal
  result returns no session, and `Progress` is `#[must_use]`.
- **Drop safety.** Dropping a session drops its boxed bodies and tasks
  without calling any of them. The crate has no `Drop` impl and no unsafe
  code, and no panic path exists in shipped code.
- **Cancel poll before each transition.** `transition` reads
  `cancel.cause()` before it enters or resumes anything, and the AC-7 test
  proves that no further charge or body runs.
- **`no_std` and dependencies.** The crate root is `#![no_std]` with `alloc`,
  and has no `std` feature, as FR-401 rule 3 requires. Its normal
  dependencies are exactly quire-semantic-value, quire-exact and thiserror,
  each with `default-features = false`, and each is named by the source. It
  names no QSL crate. `cargo build -p quire-eval --target
  thumbv7em-none-eabi` succeeds. The only dev-dependency feature is
  quire-exact `test-support`, and resolver 2 keeps it out of normal builds.
- **Vocabulary move.** `Evaluation`, `CallFailure` (its arms, `category`, and
  `From<SuppliedFailure>`), `FamilyOutcome` with its `seam_probe` variant,
  `FamilyResult` and `EvalOutcome` match qsl-eval and qsl-semantics. The
  `Cancelled` doc gains "or transition".
- **Gate.** `make ci` passed, run once on the rebased head with the SV#61
  patch.

## Verdict

Request changes. FND-001 and FND-002 are high, because they give wrong
results through the public API that CG will build on. Both need a spec
ruling on how cancellation is reported (see "Spec gaps" below), but the code
fix is small either way. FND-003 and FND-004 are medium API-shape findings.
They are cheap now and expensive once CG codes against the trait. Fix them
in this PR. FND-005 and FND-006 are low. FND-007 must be done before merge,
and SV#61 must merge first.

## Spec gaps the coder recorded, judged

1. **"quire-eval has no std feature" (FR-401) vs the brief's "std default-on".**
   The coder followed the spec correctly. FR-401 rule 3 and AC-1 say no
   `std` feature, and NFR-015's Scope excludes the leaf. No amendment is
   needed. The brief was wrong.
2. **Meter carries its own Cancel vs the explicit Cancel.** A spec amendment
   is needed (FND-002). FR-262's Begin row lists "one `Meter`; `Cancel`" as
   two inputs, but its fuel section requires "poll `Cancel` at every
   charge", and only the meter's installed handle can do that. The
   amendment should make them one handle: the session's Cancel is the
   meter's.
3. **FR-262-AC-7 "direct-return caller ... runs no subsequent body work".**
   The implementation's reading is right. The body has to be resumed once
   with the child: FR-262 requires delivery "to the matching continuation
   exactly once", and the scheduler cannot know a body is direct-return
   without resuming it. "No subsequent body work" can only mean no work
   after the body returns the propagated outcome. A low-severity wording
   amendment to AC-7 would remove the ambiguity. No code change is needed.
4. **A body requests a non-admitted callee.** A spec amendment is needed
   (FND-003). FR-262 says only "the scheduler SHALL validate the request".
   It must also say what a failed validation of a nested request returns.
   The reference treats an unresolvable nested callable as an internal
   invariant (`evaluation-callable-unresolved`), and FR-090 forbids
   reporting it as caller input. My recommendation is `CallFailure::Fault`.
5. **(New) How a charge-observed cancellation is reported.** A spec amendment
   is needed (FND-001). FR-276-AC-3 wants `CallFailure::Cancelled` from a
   cancelled `execute`. But today's reference `call` returns
   `Ok(Incomplete)`, and its callers check `cancel.tripped()`
   (qsl-replay/src/execute.rs:740). FR-262's "existing result contracts" and
   FR-401-AC-4's "no edit to any expected value" then pull the other way.
   FR-262 needs one rule.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The result of a cancellation depends on which charge observed it. If the meter's cancel handle denies the `function.call` charge, `denied()` reads `cancel.tripped()` and returns `Err(CallFailure::Cancelled)`. If it denies a kernel charge a body or interpreted frame makes, the body returns `Done` with the located incomplete result, as the `Body` contract tells it to. When that is the root frame's last transition, `Transition::Done` pops to an empty stack and returns `Ok(Incomplete{work_units})` without checking `tripped()`, so the caller gets a fuel-exhaustion result for a cancellation (probe-confirmed). This breaks FR-276-AC-3 ("returns `CallFailure::Cancelled` ... and no value") and FR-262's "Locations ... and outcome propagation SHALL match the reference" one way or the other. Fix after the spec ruling (gap 5): apply one rule at every terminal. If the handle tripped, return `Cancelled` from both the `Done`-to-empty-stack path and `denied()`. Otherwise drop the `denied()` special case and return `Ok(Incomplete)` everywhere, as the reference does. Add a test where a body's own charge observes the cancellation. | crates/quire-eval/src/session.rs:369-374, 432-445; crates/quire-eval/src/body.rs:63-67 |
| FND-002 | high | `begin_call` and `begin_expression` take a `Cancel` separate from the `Meter`, and nothing ties the two together. (a) If the meter has no handle installed, as with the test helper `meter()`, no charge polls the session's Cancel. Cancellation is then seen only between transitions, never "at every charge" (FR-262, FR-276). (b) If the meter carries a different handle and that handle trips at `function.call`, `denied()` checks the session's handle, finds it untripped, and returns `Ok(Incomplete{limit = consumed})`, a cancellation dressed as fuel exhaustion. Fix: drop the `cancel` parameter and use the meter's installed handle (`meter.cancel().cloned()`), so a meter with no handle is simply not cancellable. Or refuse Begin when the two differ. Amend FR-262's Begin row to match (gap 2). | crates/quire-eval/src/registry.rs:125-167; crates/quire-eval/src/session.rs:330-333, 432-438 |
| FND-003 | medium | A nested `ChildRequest` is validated through the same `CheckedClosure::admit_call` as the root call. When a body names a callee that is not admitted, or passes the wrong number of arguments, the whole session ends as `CallFailure::Input(UnknownFunction/Arity)`, a caller-input refusal, although the caller's own input was admitted (probe-confirmed). FR-090 and `CallFailure`'s own doc say a broken invariant is "never a caller-input refusal". Fix: for the nested path, map an admission failure to `CallFailure::Fault` (`InternalFault::new("eval", "child-request-not-admitted")`). Or give the trait a separate `admit_request` that returns an `InternalFault`. The spec must say which (gap 4). | crates/quire-eval/src/session.rs:390-393; crates/quire-eval/src/closure.rs:37-49 |
| FND-004 | medium | The public extension points `Body::resume`, `CheckedClosure::advance`, `bind_call` and `bind_expression` return `Result<_, CallFailure>`, but their docs allow only `CallFailure::Fault`. A body or evaluator that returns `Input`, `Storage`, `Supplied` or `Cancelled` mid-evaluation has that error propagated unchanged as the application's failure. Those are pre-call or caller-input arms, which is the conflation FR-090 forbids, and the type system does not stop it. CG will implement `Body`, so narrow these now: return `Result<_, InternalFault>` and wrap it in `CallFailure::Fault` at the scheduler. | crates/quire-eval/src/body.rs:67; crates/quire-eval/src/closure.rs:58-111 |
| FND-005 | low | `Entry::Call { site: Option<Location>, admitted: bool }` encodes "root call" twice: `site == None` always pairs with `admitted == true`. Two correlated fields allow states that never occur. An `Entry::Root` / `Entry::Request { site }` split, or one `Option<Location>` that also decides admission, makes them unrepresentable. Separately, `seal()` stores a throwaway `Sealed(Arc::default())` and then overwrites it. `if let State::Open(b) = &mut self.state { self.state = State::Sealed(Arc::new(mem::take(b))) }` is the plain form. | crates/quire-eval/src/session.rs:217-232, 384-393; crates/quire-eval/src/registry.rs:188-195 |
| FND-006 | low | `make lint` runs no clippy pass for quire-eval on the no-std target. Only quire-checked gets the thumbv7em clippy line. Code that exists only on that target, or that is dead there, goes unlinted. Fix: add `$(CARGO) clippy -p quire-eval --lib --target $(NO_STD_TARGET) -- -D warnings`. | Makefile:17-20 |
| FND-007 | medium | The base is stale. The PR targets `ir-586-skeleton` and contains e216c91, the pre-fix-round skeleton commit. Merged against main it conflicts in Cargo.toml, and it would also try to revert #1's fix round (CONTENT_RIGHTS.md, crates/quire-checked/src/lib.rs, the Makefile `mint-probe` target, rustfmt.toml, reviews/ir-586-*). Fix: `git rebase --onto origin/main e216c91`, keep both `cfg(mint_probe)` and `cfg(seam_probe)` in the check-cfg list, and retarget the PR to `main`. After SV#61 merges, run `cargo update -p quire-semantic-value`: the Cargo.lock on the branch pins an SV commit without `catalog`, so the branch does not build until then. | Cargo.toml:21-22; PR base ir-586-skeleton |
