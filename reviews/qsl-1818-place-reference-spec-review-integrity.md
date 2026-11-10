---
id: SR-004
title: QSL-1818 place-reference spec-review-integrity review
type: SpecReview
analysis: integrity
scope: 'agent-ix/quire-eval; PR #27; spec/functional/FR-008-clock-place-reference-node.md;
  spec/spec.md'
review_set: subset
relationships:
- target: ix://agent-ix/quire-eval/FR-008
  type: references
---
## Summary

The descriptor/projection contract is internally consistent and bounded, with no new in-scope completeness or atomicity defect.

## Assurance Context

No applicable AssuranceProfile exists in this repository. Direct analyses selected by the code lead were used; no accepted enforced review workflow is claimed. Upstream FR-230/FR-400/FR-415/FR-322 and the actual checked-node/checker source supply architecture and ownership context.

## Verdict

**PASS** — no introduced or in-scope findings. This is specification readiness only; implementation and full clock-feature qualification remain pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Evidence

The owner reference plus producer-assigned template identity and slot selectors are the complete in-process inputs to the existing identity rule. Scalar empty selectors, repeated handles, traversal ordering, changed mapped handles, and borrowed metadata are independently observable. A plain descriptor may be represented without proving admission; validating selector type/arity/bounds, refusing Map references, evaluating children, constructing the encoded id and validating snapshots remain upstream producer/runtime obligations. They are explicitly excluded rather than silently satisfied by projection tests. Spec/spec.md indexes the new FR without declaring unrelated inventory complete. No additional wire or kernel type is introduced.

Full scoped specification validation passes. No Cargo, Kani, CBMC or aggregate gate ran. Frozen source head and worktree were unchanged during review.

## Examined Scope

```yaml
scope:
- id: FR-008-AC-1
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: A scalar clock-place reference projects as the distinct `PlaceReference` operation with its mapped owner-reference child, the original template `NodeKey`, and an empty slot list; it is neither a presence predicate nor an ordinary object-field read.
- id: FR-008-AC-2
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: A clock-place reference with two slot selectors maps the owner first and each selector in path order, exactly once per occurrence, retaining the mapped handles even when source arena indices are descending.
- id: FR-008-AC-3
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: Repeated selector handles remain repeated in the projected slot list; the template key is unchanged when mapped child handles differ from stored handles.
- id: FR-008-AC-4
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: A borrowed clock-place-reference read borrows the supplied checked result type and location without reconstructing either metadata value.
- id: FR-008-AC-5
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: The descriptor and projection use the existing owner-qualified child, `NodeKey`, type and location vocabulary, with no new wire node, kernel reference value or local place-id encoder; descriptor construction makes no admission claim.
- id: FR-008
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: When the checker admits a clock-place reference, the checked-node vocabulary SHALL retain its owner, template identity and slot selectors.
- id: master-requirements
  path: spec/spec.md
  role: examined
  excerpt: 'in-process clock-place-reference descriptor and projection are specified in

    [FR-008](functional/FR-008-clock-place-reference-node.md). Other crate behavior'
- id: FR-230
  path: ix://agent-ix/quire-spec-language/FR-230
  role: context_only
  excerpt: "The place id of a `Reference<Clock>` SHALL be a\n  canonical, one-to-one encoding of the triple (owner object identity, the\n  `clock_place` node key, slot indices)"
- id: FR-400
  path: ix://agent-ix/quire-spec-language/FR-400
  role: context_only
  excerpt: 'The checked terms SHALL live in the crate `quire-checked`, in the repository

    `agent-ix/quire-eval`'
- id: FR-322
  path: ix://agent-ix/quire-specification/FR-322
  role: context_only
  excerpt: 'none: no node stands for the clock or for the reference; the checker reads `ref` and `deref` over it and the state key holds the place id'
```
