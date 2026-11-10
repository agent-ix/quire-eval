---
id: SR-006
title: IR-1434 checked clock-reference code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-eval 0.1.0; crates/quire-checked/src/node.rs; crates/quire-checked/tests/node_projection.rs;
  spec/functional
review_set: subset
relationships:
- target: ix://agent-ix/quire-eval/FR-008
  type: references
---
## Summary

Ticket: IR-1434. Reviewed the checked descriptor and its borrowed projection. Existing NodeKey is retained, owner maps first, slots preserve every occurrence and source order, and checked type/location remain borrowed. No wire, value, snapshot, encoding or admission implementation was added.

## Verdict

**PASS** — no findings within the reviewed scope. This qualifies the in-process descriptor only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Evidence

The scalar test asserts the distinct variant, mapped owner, unchanged key, empty slots and pointer identity for both metadata values. Descending source handles [9,7,2] map to [1,2,3] by invocation order; repeated [8,3,3] remains [28,23,23]. The generic projection roundtrip now includes PlaceReference. No duplicate helper or identity vocabulary found. No CI or lint gates changed. No applicable AssuranceProfile or repository Rust idiom document exists.

cargo fmt --check and quire matrix --scope . --strict passed. Inspected supplied coder evidence: three focused unit tests passed, generic projection test passed, and no-default-features check passed. No duplicate Cargo run, Clippy, deny, Kani or full suite was run under the explicit focused-review gate; the lead owns the final full make test.

## Coverage

Plan completion: not assessed

Computed repository matrix: 33 criteria, 31 tagged Test criteria and two Inspection criteria marked method-without-symbol; none untagged or ignored. FR-008-AC-5 was inspected directly. Existing broader vocabulary is explicitly outside the local inventory in spec/spec.md and owned through upstream contracts; no newly unowned behavior, stubs or coverage suppression found. Repository-wide mechanical trace/stub checks ran; detailed behavior review is limited to the changed descriptor. Optional gap semantic fan-out was not run; direct intent/assertion review was performed in the code-review lane.

## Examined Scope

```yaml
scope:
- id: FR-008-AC-1
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: A scalar clock-place reference projects as the distinct `PlaceReference`
    operation with its mapped owner-reference child, the original template `NodeKey`,
    and an empty slot list; it is neither a presence predicate nor an ordinary object-field
    read.
- id: FR-008-AC-2
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: A clock-place reference with two slot selectors maps the owner first and
    each selector in path order, exactly once per occurrence, retaining the mapped
    handles even when source arena indices are descending.
- id: FR-008-AC-3
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: Repeated selector handles remain repeated in the projected slot list; the
    template key is unchanged when mapped child handles differ from stored handles.
- id: FR-008-AC-4
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: A borrowed clock-place-reference read borrows the supplied checked result
    type and location without reconstructing either metadata value.
- id: FR-008-AC-5
  path: spec/functional/FR-008-clock-place-reference-node.md
  role: examined
  excerpt: The descriptor and projection use the existing owner-qualified child, `NodeKey`,
    type and location vocabulary, with no new wire node, kernel reference value or
    local place-id encoder; descriptor construction makes no admission claim.
bindings:
- test_id: place_reference_tests::scalar_place_reference_preserves_template_and_borrows_checked_metadata
  ac_id: FR-008-AC-1
  trace: correct
- test_id: place_reference_tests::place_reference_maps_owner_then_slots_in_path_order
  ac_id: FR-008-AC-2
  trace: correct
- test_id: place_reference_tests::place_reference_maps_every_repeated_selector_occurrence
  ac_id: FR-008-AC-3
  trace: correct
- test_id: place_reference_tests::scalar_place_reference_preserves_template_and_borrows_checked_metadata
  ac_id: FR-008-AC-4
  trace: correct
```
