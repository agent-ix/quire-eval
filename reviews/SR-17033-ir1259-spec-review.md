---
id: SR-17033
title: "IR-1259 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-eval; spec/spec.md, spec/usecase/US-001-inspect-ieee-checked-nodes.md, spec/functional/FR-001-ieee-checked-node-vocabulary.md; IR-1259"
review_set: subset
---
# IR-1259 base review

## Summary

The master specification, US-001, FR-001 and all six criteria were checked against the current quire-checked node shape, contract-ir catalogue identities, and QSpec IEEE admission rule. The planned status correctly leaves implementation evidence pending.

## Verdict

FINDING. No applicable AssuranceProfile exists in this repository. The review covers US-001, FR-001, and FR-001-AC-1 through FR-001-AC-6; the planned criteria are not yet implemented, so the untagged matrix rows are expected at this spec-only stage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-5 does not identify which admitted checked node may carry OrderedKind::Float. QSpec excludes ordinary IEEE ordering, while the current Order node accepts OrderedKind; this wording permits an invalid generic float-ordering implementation or a test of an unattested node. Name the admissible node/read path and preserve the distinction from ordinary Order. | spec/functional/FR-001-ieee-checked-node-vocabulary.md:45 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-5 now binds Float to the admitted IEEE total-order read and excludes ordinary NodeKind::Order for IEEE identities. |
