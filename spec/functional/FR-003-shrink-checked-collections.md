---
id: FR-003
title: Remove one occurrence through the shared checked collection kernel
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-145
    type: implements
---
# FR-003: Remove one occurrence through the shared checked collection kernel

## Description

The shared checked vocabulary SHALL distinguish `DropFirst(source)` from
`RemoveOne(source, value)`. Borrowed projection SHALL retain the operation and
its operands in source order. These descriptors grant no checking authority.

## Behavior

The `quire-eval` collection operations consume canonical collection values,
the canonical meter, and, for a removal probe, the caller's canonical type
environment. They SHALL delegate membership comparison and result formation
to `quire-exact`, preserving all non-completed outcomes and their causes.

`drop_first` SHALL accept sequences and ordered sets, remove the leading
occurrence when present, and form the remaining occurrences in their original
order. An empty operand stays empty. A set or bag SHALL return the canonical
`IllTyped` cause `OperatorIneligible` before any charge.

`remove_one` SHALL admit the probe against the source element type through
the supplied type environment before charging. A wrong type SHALL return
the canonical `IllTyped` cause `TypeMismatch`; an environment failure SHALL
remain that environment failure. It SHALL scan in `contains` visiting order,
visiting a bag's distinct members once each, stop at the first matching
member, and remove only that occurrence. A miss preserves every occurrence.

The result SHALL preserve kind and element type. An absent cardinality bound
stays absent. For an original bound `[a,b]`, the result bounds SHALL be
`[a.saturating_sub(1), b.saturating_sub(1)]` for `drop_first` and
`[a.saturating_sub(1), b]` for `remove_one`.

The drop operation SHALL charge only canonical survivor formation. The
remove operation SHALL charge canonical membership comparisons before
survivor formation, without a scalar Boolean result retain or additional
formation membership comparisons: surviving members are already distinct or grouped. When a scan charge is denied, the operation SHALL return without forming a result.
When a formation charge is denied, the operation SHALL expose no partial collection.

This requirement owns shared node descriptors, their borrowed projections,
and the shared value operations. QSL-1488 owns source syntax, source-level
static typing/lowering and its complete FR-145 source qualification. This
requirement does not claim that qualification from descriptor tests.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-003-AC-1 | Borrowed reads distinguish both operations and preserve each ordered child handle and checked type/location metadata. | Test |
| FR-003-AC-2 | Drop removes just the leading sequence/ordered-set occurrence, preserving duplicates/order and the empty case; set/bag operands refuse before charging. | Test |
| FR-003-AC-3 | Remove deletes exactly the first match for every collection kind, reducing bag multiplicity by one, and preserves all occurrences on misses and empty inputs. | Test |
| FR-003-AC-4 | Both operations preserve kind/element type and compute the declared bounded or unbounded result type, including zero and maximum bounds. | Test |
| FR-003-AC-5 | A wrong-type removal probe refuses before charges; canonical type-environment admission remains authoritative. | Test |
| FR-003-AC-6 | Drop charges exactly survivor formation; remove charges the contains scan then formation, with a bag's distinct-member visiting rule and no Boolean result retain. | Test |
| FR-003-AC-7 | A denied scan or formation charge preserves the admitted charge prefix and exposes no partial result. | Test |

## Dependencies

- QSpec FR-145 owns collection removal semantics and the charge schedule.
- `quire-exact` owns canonical formation, membership, values, bounds and meters.
- `quire-semantic-value` owns schema admission and its typed failures.
- QSL-1488 owns grammar, static source typing, lowering and source qualification.
