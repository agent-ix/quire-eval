---
id: FR-005
title: Name package-level checked-call admission failures
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-390
    type: implements
---
# FR-005: Name package-level checked-call admission failures

## Description

The evaluation result vocabulary SHALL provide `CallAdmissionLocus::Package`
as a payload-free locus for a checked-call admission failure that applies to
the package as a whole. It SHALL retain the existing `Node`, `Population`,
`Member`, and `Binder` loci and their payloads.

## Behavior

`Package` SHALL remain distinguishable from every existing locus through the
public enum and through `CallAdmissionRecord`. The record SHALL retain the
original diagnostic with a package locus. This locus does not itself contain
package identity or refusal reason; QSL FR-390 owns the ordering of unsupported
view failures across packages.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | `Package` has no payload and remains distinguishable from every existing locus. | Test |
| FR-005-AC-2 | A package admission record retains its package locus and original diagnostic; existing node, population, member, and binder loci retain their distinct variants and payloads. | Test |

## Dependencies

- QSL FR-390 owns package identity, unsupported-view reasons, and ordering of
  failures across packages.
- `quire-exact` owns `NodeKey` identity and its order.
