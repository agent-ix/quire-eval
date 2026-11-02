---
id: SR-17041
title: "Gap analysis: FR-002 connective origin (QSL-1388, PR #18)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-eval PR #18 @730deae543360514f239da9bdda60a38694af214; spec/functional/FR-002-checked-connective-origin.md, crates/quire-checked/src/node.rs, crates/quire-checked/tests/node_projection.rs; FR-002-AC-1 through FR-002-AC-3"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-eval/FR-002", type: references }
---
# Gap analysis: FR-002 connective origin

## Summary

`quoin matrix --repo . --json` lists FR-002-AC-1, AC-2 and AC-3 as tagged. Tests: `connective_read_retains_origin_and_operand_order` (AC-1, AC-2) and `authored_and_synthesized_connectives_are_distinct_nodes` (AC-3); the exhaustive projection test goes from 60 to 61 cases.

## Verdict

PASS. No untraced changed behavior, no stub, no coverage inflation. Mutation results are in SR-17040. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Changed behaviors: 1 (origin carried and projected). Untraced: 0. Changed source stubs: 0. Changed test stubs: 0.
