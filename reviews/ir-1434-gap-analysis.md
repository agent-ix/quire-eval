---
id: SR-007
title: IR-1434 checked clock-reference gap-analysis
type: SpecReview
analysis: gap-analysis
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
- id: FR-001-AC-1
  path: spec/functional/FR-001-ieee-checked-node-vocabulary.md
  role: examined
  excerpt: A checked `quire.op.ieee.float32.fma` node projects as a typed float32
    fused multiply-add read with its admitted rounding mode and three operands in
    source order.
- id: FR-001-AC-2
  path: spec/functional/FR-001-ieee-checked-node-vocabulary.md
  role: examined
  excerpt: A checked `quire.op.ieee.float64.fma` node projects as a typed float64
    fused multiply-add read with its admitted rounding mode and three operands in
    source order; the read remains distinguishable from float32.
- id: FR-001-AC-3
  path: spec/functional/FR-001-ieee-checked-node-vocabulary.md
  role: examined
  excerpt: A checked `quire.op.ieee.total_order` node projects as a typed two-operand
    total-order read, distinct from ordinary ordering and numeric equality.
- id: FR-001-AC-4
  path: spec/functional/FR-001-ieee-checked-node-vocabulary.md
  role: examined
  excerpt: A checked `quire.op.ieee.bit_identical` node projects as a typed two-operand
    bit-identity read, distinct from total order and numeric equality.
- id: FR-001-AC-5
  path: spec/functional/FR-001-ieee-checked-node-vocabulary.md
  role: examined
  excerpt: An admitted `quire.op.ieee.total_order` node carries `OrderedKind::Float`
    through its typed borrowed read, while its identity remains distinct from ordinary
    `NodeKind::Order`; no IEEE catalogue identity is represented as ordinary `NodeKind::Order`.
- id: FR-001-AC-6
  path: spec/functional/FR-001-ieee-checked-node-vocabulary.md
  role: examined
  excerpt: Each identity in the table has a corresponding checked-node representation
    whose typed read retains that identity without a local identity alias or copied
    catalogue.
- id: FR-002-AC-1
  path: spec/functional/FR-002-checked-connective-origin.md
  role: examined
  excerpt: A borrowed read of a stored connective node exposes the operator the node
    was stored with.
- id: FR-002-AC-2
  path: spec/functional/FR-002-checked-connective-origin.md
  role: examined
  excerpt: A borrowed read of a stored connective node exposes its left and right
    operands in stored order.
- id: FR-002-AC-3
  path: spec/functional/FR-002-checked-connective-origin.md
  role: examined
  excerpt: A borrowed read of a node stored with `ConnectiveOrigin::Authored` exposes
    `Authored`.
- id: FR-002-AC-4
  path: spec/functional/FR-002-checked-connective-origin.md
  role: examined
  excerpt: A borrowed read of a node stored with `ConnectiveOrigin::Synthesized` exposes
    `Synthesized`.
- id: FR-002-AC-5
  path: spec/functional/FR-002-checked-connective-origin.md
  role: examined
  excerpt: A borrowed read of the same stored connective node yields the same origin
    whichever location the read is given; this guards against a regression that makes
    the exposed origin depend on the location.
- id: FR-002-AC-6
  path: spec/functional/FR-002-checked-connective-origin.md
  role: examined
  excerpt: Two connective nodes that differ only in origin are not equal.
- id: FR-003-AC-1
  path: spec/functional/FR-003-shrink-checked-collections.md
  role: examined
  excerpt: Borrowed reads distinguish both operations and preserve each ordered child
    handle and checked type/location metadata.
- id: FR-003-AC-2
  path: spec/functional/FR-003-shrink-checked-collections.md
  role: examined
  excerpt: Drop removes just the leading sequence/ordered-set occurrence, preserving
    duplicates/order and the empty case; set/bag operands refuse before charging.
- id: FR-003-AC-3
  path: spec/functional/FR-003-shrink-checked-collections.md
  role: examined
  excerpt: Remove deletes exactly the first match for every collection kind, reducing
    bag multiplicity by one, and preserves all occurrences on misses and empty inputs.
- id: FR-003-AC-4
  path: spec/functional/FR-003-shrink-checked-collections.md
  role: examined
  excerpt: Both operations preserve kind/element type and compute the declared bounded
    or unbounded result type, including zero and maximum bounds.
- id: FR-003-AC-5
  path: spec/functional/FR-003-shrink-checked-collections.md
  role: examined
  excerpt: A wrong-type removal probe refuses before charges; canonical type-environment
    admission remains authoritative.
- id: FR-003-AC-6
  path: spec/functional/FR-003-shrink-checked-collections.md
  role: examined
  excerpt: Drop charges exactly survivor formation; remove charges the contains scan
    then formation, with a bag's distinct-member visiting rule and no Boolean result
    retain.
- id: FR-003-AC-7
  path: spec/functional/FR-003-shrink-checked-collections.md
  role: examined
  excerpt: A denied scan or formation charge preserves the admitted charge prefix
    and exposes no partial result.
- id: FR-004-AC-1
  path: spec/functional/FR-004-quantity-power-node.md
  role: examined
  excerpt: A quantity-power read retains the distinct operation and mapped base/exponent
    handles in that order, including when the source handles have descending indices.
- id: FR-004-AC-2
  path: spec/functional/FR-004-quantity-power-node.md
  role: examined
  excerpt: The read borrows the original checked type and location without reconstructing
    either metadata value.
- id: FR-005-AC-1
  path: spec/functional/FR-005-package-call-admission-locus.md
  role: examined
  excerpt: '`Package` has no payload and remains distinguishable from every existing
    locus.'
- id: FR-005-AC-2
  path: spec/functional/FR-005-package-call-admission-locus.md
  role: examined
  excerpt: A package admission record retains its package locus and original diagnostic;
    existing node, population, member, and binder loci retain their distinct variants
    and payloads.
- id: FR-006-AC-1
  path: spec/functional/FR-006-quantity-negation-node.md
  role: examined
  excerpt: A quantity-negation read retains a distinct unary `QuantityNegate` operation
    and maps its one operand exactly once, including when the mapped child handle
    differs from the stored operand handle.
- id: FR-006-AC-2
  path: spec/functional/FR-006-quantity-negation-node.md
  role: examined
  excerpt: The read borrows the supplied checked result type and location without
    reconstructing either metadata value; the node adds no unit, law, mode, member
    or magnitude field.
- id: FR-007-AC-1
  path: spec/functional/FR-007-unsupported-entry-and-prepare-limit-failures.md
  role: examined
  excerpt: '`CallFailure::Unsupported` over each of `NotSupplied` and `Untranslatable`
    has category `Category::Unsupported` and catalog code `unimplemented_capability`,
    and converts from its `UnsupportedEntry`.'
- id: FR-007-AC-2
  path: spec/functional/FR-007-unsupported-entry-and-prepare-limit-failures.md
  role: examined
  excerpt: An `UnsupportedEntry`'s catalog cause is `untranslatable-entry` or `backend-not-supplied`;
    its payload names the `backend` and, for `Untranslatable`, the `construct` kind,
    `construct_name` and `location` of an expression form, a value type (`Decimal`)
    and an operation node key, and carries none of them for `NotSupplied`.
- id: FR-007-AC-3
  path: spec/functional/FR-007-unsupported-entry-and-prepare-limit-failures.md
  role: examined
  excerpt: '`CallFailure::Limit` has category `Category::Incomplete` and catalog code
    `stage_limit_exceeded`, holds the setting, bound and actual unchanged, and `PrepareSetting::name()`
    is `prepare.compile_work_units` or `prepare.code_bytes`.'
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
