---
id: FR-008
title: Read a checked clock-place reference operation
type: FR
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-400
    type: implements
  - target: ix://agent-ix/quire-spec-language/FR-230
    type: implements
  - target: ix://agent-ix/quire-specification/FR-415
    type: references
---
# FR-008: Read a checked clock-place reference operation

## Description

When the checker admits a clock-place reference, the checked-node vocabulary SHALL retain its owner, template identity and slot selectors.
The in-process operation SHALL be
`PlaceReference { reference: Child, place: NodeKey, slots: Vec<Child> }`.
The descriptor supplies the three inputs of the place identity defined by
[QSL FR-230](ix://agent-ix/quire-spec-language/FR-230): an owner object,
the producer-assigned `clock_place` template key, and concrete slot selectors.

## Behavior

The `reference` child denotes the owner object reference. The `place` member
is the existing `quire-exact::NodeKey` of the checker-selected `clock_place`
template, not an expression's body-local index or a key reconstructed by a
reader. The checker supplies this key. The `slots` children denote admitted
Integer selectors in the order of the template's slot steps; a scalar place
has no slot children. The checked result type and source location remain the
metadata supplied to `NodeRead`.

A borrowed projection SHALL map `reference` first, then each occurrence in
`slots` in its original order. The projection SHALL retain the template key
unchanged. The projection SHALL borrow the supplied checked result type and
location unchanged. Repeated child handles remain repeated, and a child's
arena index does not determine traversal order.

This is a plain descriptor under
[FR-400](ix://agent-ix/quire-spec-language/FR-400), distinct from
`PlacePresence` and `SlotPresence`. Possessing it grants no admission or
evaluation authority. Projection neither evaluates children nor constructs
or validates the encoded place id. The descriptor adds no wire node,
kernel reference value, field-path schema, or alternate identity algorithm.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-008-AC-1 | A scalar clock-place reference projects as the distinct `PlaceReference` operation with its mapped owner-reference child, the original template `NodeKey`, and an empty slot list; it is neither a presence predicate nor an ordinary object-field read. | Test |
| FR-008-AC-2 | A clock-place reference with two slot selectors maps the owner first and each selector in path order, exactly once per occurrence, retaining the mapped handles even when source arena indices are descending. | Test |
| FR-008-AC-3 | Repeated selector handles remain repeated in the projected slot list; the template key is unchanged when mapped child handles differ from stored handles. | Test |
| FR-008-AC-4 | A borrowed clock-place-reference read borrows the supplied checked result type and location without reconstructing either metadata value. | Test |
| FR-008-AC-5 | The descriptor and projection use the existing owner-qualified child, `NodeKey`, type and location vocabulary, with no new wire node, kernel reference value or local place-id encoder; descriptor construction makes no admission claim. | Inspection |

## Dependencies

- [QSL FR-230](ix://agent-ix/quire-spec-language/FR-230) owns clock-reference
  admission, the place-identity triple and encoding, slot selection, Map-place
  refusal, `ref`/`deref`, and snapshot validation. QSL's clock-reference checker
  is the producer and its evaluator is the consumer of this descriptor.
- [QSL FR-400](ix://agent-ix/quire-spec-language/FR-400) owns checking authority
  and the checked-term boundary; only the checker constructs an admitted term.
- [QSpec FR-415](ix://agent-ix/quire-specification/FR-415) owns clock-place
  semantics. [QSpec FR-322](ix://agent-ix/quire-specification/FR-322) states that
  no wire node stands for `Reference<Clock>`; this in-process descriptor does
  not change that placement rule.
- `quire-exact` owns `NodeKey`, `ValueType` and the existing `ObjectReference`
  value carrier. This requirement adds no replacement for them.

Descriptor/projection controls qualify this read contract only. They do not
qualify source-level clock typing, reference evaluation, snapshot rejection,
or QSL FR-230-AC-24/43/44/45/62.
