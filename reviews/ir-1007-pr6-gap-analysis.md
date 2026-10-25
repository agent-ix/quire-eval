---
id: SR-17027
title: 'IR-1007 quire-eval PR #6 gap analysis'
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-eval; PR #6 6f36c8a5b5fb50779b6cda7273938cb7f0ca3a47; crates/quire-eval/src/{closure,registry,session}.rs,
  crates/quire-eval/tests/{registry,support/mod}.rs; QSL FR-262-AC-13, FR-485-AC-6,
  TC-1263'
review_set: subset
---

## Summary

Ticket IR-1007. Plan completion: not assessed. Quoin matrix is unavailable for this leaf because it has no `spec/` document root; I manually compared the QSL requirement and planned integration test with the three tagged leaf tests and scheduler behavior.

## Verdict

**CONDITIONAL** — root-location behavior is implemented, but the new denial tests do not observe the required absence of body invocation. The actual checked-graph own/imported oracle belongs to the QSL adapter and TC-1263; it is outstanding cross-repo evidence, not a second evaluator to add here.

## Coverage

- FR-262-AC-13: three new `/// Trace:` tests bind location, missing-location fault and post-Begin cancellation. The existing scheduler test binds nested child-site behavior.
- FR-485-AC-6 and TC-1263: checked-graph own/imported location proof is specified in the QSL draft; no real checker dependency is introduced to the leaf.
- Quoin matrix refused with `CORE_REFUSED: no document root .../quire-eval/spec`; manual requirement-to-test comparison used.
- Plan completion: not assessed. Optional semantic review was not invoked as a Quoin phase; an independent Opus code pass was consulted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new zero-work and post-Begin cancellation tests assert location and zero meter spend but never observe body invocation. `host_a` has no registered factory in these tests and no `Toy::binds()` or invocation counter is asserted. A regression that enters or invokes a body before the denied `function.call` and still returns the incomplete result would pass, leaving FR-262-AC-13's explicit "no body invocation" clause unverified. Register a counting body for a denied root and assert it never begins, or inspect the interpreted bind log. | crates/quire-eval/tests/registry.rs:209,226 |
