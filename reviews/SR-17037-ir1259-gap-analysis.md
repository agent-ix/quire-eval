---
id: SR-17037
title: "Gap analysis — IEEE checked node reads"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-eval; spec/functional/FR-001-ieee-checked-node-vocabulary.md, crates/quire-checked/src/node.rs, crates/quire-checked/tests/node_projection.rs; FR-001-AC-1 through FR-001-AC-6"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-eval/FR-001", type: references }
---
# Gap analysis — IEEE checked node reads

## Summary

The computed matrix, changed production variants, and traced tests were compared for IR-1259 PR #17. The four added public variants and Float domain tag are owned by FR-001. No applicable AssuranceProfile exists in this repository.

## Verdict

PASS for the changed slice. The four Test-kind criteria are tagged to exercising tests, and the Inspection-kind criterion has four corresponding tests plus direct variant and mapping inspection. No changed source or test stub was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed.

`quoin matrix --repo . --json` reports FR-001-AC-1 through AC-5 as tagged; FR-001-AC-6 is `method-without-symbol`, appropriate to its Inspection verification. No run evidence is bound in Quoin. Four distinct IEEE variants, one Float domain tag, and four new tagged tests were inventoried; untraced changed behaviors: 0; changed source stubs: 0; changed test stubs: 0. The existing repository behavior outside FR-001 was not assigned a new requirement by this review.
