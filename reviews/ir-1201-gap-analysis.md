---
id: SR-17031
title: 'IR-1201 PR 15 gap analysis'
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-eval; IR-1201; Cargo.lock, crates/quire-checked/src/lib.rs, crates/quire-checked/src/node.rs, crates/quire-checked/tests/node_read.rs; FR-400-AC-11, FR-400-AC-12'
review_set: subset
---

## Summary

Performed a manual acceptance-criteria-to-tests audit of the frozen diff. Quoin's computed matrix is unavailable because this repository has no `spec/` document root; QSL origin/main supplies the governing FR-400 criteria.

## Verdict

**FAIL** — the tagged tests exercise selected cases but do not establish either full criterion.

## Coverage

- Reconciliation: manual AC-to-test mapping; `quoin matrix --repo . --json` refused because no `spec/` root exists. The five new test functions are tagged to FR-400-AC-11, one also to FR-400-AC-12.
- Plan completion: not assessed
- FR-400-AC-11 examined: “A stored `NodeKind` and its `NodeRead` projection match the same exhaustive generic enum: every variant, nested operand position, literal value, metadata/evidence identity, checked type and location are preserved. A probe variant makes both evaluator paths non-exhaustive. No parallel operation enum exists; the layer-C read descriptors compile without `std`, QSL or IR dependencies.”
- FR-400-AC-12 examined: “Constructing a `NodeRead` or a parameterized operation descriptor cannot create a checked term or an admitted FR-390 view: passing a descriptor to the public evaluator as a checked input fails to compile. Existing checked-term constructor and mutation rejection checks remain failing, with their positive pairs compiling.”
- FR-390 view contract and ADR-011 layer boundary read as context. Optional semantic review was not requested.
- Focused runtime evidence: 4 default and 5 construct-feature node_read tests passed; no-default-features host check passed. No thumbv7em target build or compile-fail acceptance probe ran.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-400-AC-11 exhaustive projection has only selected-variant tests; most variants and the evaluator seam probe are untested | crates/quire-checked/tests/node_read.rs:8 |
| FND-002 | high | FR-400-AC-12 is tagged to a successful literal read but has no negative compile test or positive pair for admission separation | crates/quire-checked/tests/node_read.rs:98 |

## Evidence

- FND-001: The tests construct `Record`, `Fold`, `Case`, `Literal`, `Attribute` and `IntegerDivision`. They do not compare stored and borrowed reads for the remaining variants, including `ImportedCall`, `Lookup` and the authored absence mode. They do not invoke an evaluator-path seam probe. A mutation to `map_ref` for any untested arm would leave the suite green.
- FND-002: `borrowed_literal_and_checked_metadata_have_one_read_shape` asserts successful values and pointer identity only. It cannot fail if an evaluator accepts a constructed descriptor or if checked-term constructor rejection becomes invalid. No new compile-fail fixture tests those conditions.
