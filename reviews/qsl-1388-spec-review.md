---
id: SR-17042
title: "Spec review: FR-002 and US-002 connective origin (QSL-1388, PR #18)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-eval PR #18 @730deae543360514f239da9bdda60a38694af214; spec/functional/FR-002-checked-connective-origin.md, spec/usecase/US-002-distinguish-authored-connectives.md, spec/spec.md; FR-002-AC-1 through FR-002-AC-3; QSL FR-265 read for context"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-eval/FR-002", type: references }
---
# Spec review: FR-002 and US-002

## Summary

`quire validate --scope .` on the three files exits 0 and prints only repo-wide module warnings (DuplicateArchetype, DuplicateInverseEdge, semantic.inline-data-schema) unrelated to these files. IDs FR-002 and US-002 are unused elsewhere in the repo. EARS, integrity and strength sub-checks were done by reading; no Jev run.

## Verdict

Mergeable with the findings below as non-blocking. AC-3 is single-claim and failable; AC-1 and AC-2 each bundle more than one claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | "Authored" and "synthesized" are not defined here, and QSL FR-265 (agent-ix/quire-spec-language main) does not state the decision rule either; AC-2 only says authored node exposes Authored | spec/functional/FR-002-checked-connective-origin.md:12-16 |
| FND-002 | low | FR-002-AC-2 asserts two outcomes (Authored and Synthesized) in one criterion | spec/functional/FR-002-checked-connective-origin.md:32 |
| FND-003 | low | FR-002-AC-1 bundles operator retention and operand order in one criterion | spec/functional/FR-002-checked-connective-origin.md:31 |
| FND-004 | low | Description holds two SHALLs (retain, expose) and Behavior states "a consumer never derives it from locations" with no SHALL or AC | spec/functional/FR-002-checked-connective-origin.md:16-25 |
| FND-005 | low | spec.md inventory sentence is spliced into one over-long line, breaking the file's wrapping | spec/spec.md:35 |

## Open question

FR-265 describes the walk as "the operand of and or or on which evaluation stopped" and says nothing about authored versus synthesized connectives or about the float_ordering synthesis. The authored-clause decision rule that motivates this carrier is therefore unspecified in QSL main; the QSL side needs to state it (IR-1276 says PR #2088 owns the behavior).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The Authored definition ends "or checked form for that clause", which does not say what checked form counts as authored; a producer could read it either way | spec/functional/FR-002-checked-connective-origin.md:23-25 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Behavior now defines Authored and Synthesized in terms of what the producer builds, with no location input, and forbids consumers from deriving origin from location data; decision semantics are delegated to the consumer's requirements (QSL FR-265, to be amended, still an open item in QSL) |
| FND-002 | fixed | Split into FR-002-AC-3 (Authored reads Authored) and FR-002-AC-4 (Synthesized reads Synthesized), each with its own tagged test |
| FND-003 | fixed | Split into FR-002-AC-1 (operator) and FR-002-AC-2 (operand order); test iterates all three operators and distinct operands |
| FND-004 | fixed | Description has one SHALL; the location prohibition is now a normative SHALL NOT, covered at read level by FR-002-AC-5 |
| FND-005 | fixed | spec.md paragraph is re-wrapped across lines |
