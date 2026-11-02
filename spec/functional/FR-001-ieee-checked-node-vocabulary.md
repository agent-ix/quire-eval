---
id: FR-001
title: Preserve IEEE operation identity in checked node reads
type: FR
status: implemented
relationships:
  - target: "ix://agent-ix/quire-eval/US-001"
    type: implements
---
# FR-001: Preserve IEEE operation identity in checked node reads

## Description

When a checked expression uses a catalogue-admitted IEEE fused multiply-add,
total-order, or bit-identity operation, `quire-checked` SHALL retain its
contract-ir operation identity and SHALL expose a typed `NodeKind` read with
the admitted operands.

## Behavior

| Contract-ir operation identity | Checked read shape |
| --- | --- |
| `quire.op.ieee.float32.fma` | IEEE fused multiply-add with float32 width, one admitted rounding mode, and three ordered operands |
| `quire.op.ieee.float64.fma` | IEEE fused multiply-add with float64 width, one admitted rounding mode, and three ordered operands |
| `quire.op.ieee.total_order` | IEEE total-order comparison with two ordered operands |
| `quire.op.ieee.bit_identical` | IEEE bit-identity comparison with two ordered operands |

The `NodeKind` vocabulary SHALL distinguish these four identities from each
other and from existing IEEE arithmetic, square root, numeric equality, and
ordinary ordering nodes. Its `OrderedKind` vocabulary SHALL include `Float`
as the domain tag of the distinct `quire.op.ieee.total_order` checked read.
An admitted IEEE total-order operation SHALL NOT use ordinary
`NodeKind::Order`, and IEEE values remain ineligible for ordinary ordering.
A borrowed node read SHALL preserve the
same identity, width where applicable, rounding mode where applicable, and
operand order as its stored checked node. This requirement defines a typed
representation; it does not define a second evaluator or a local operation
catalogue.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-001-AC-1 | A checked `quire.op.ieee.float32.fma` node projects as a typed float32 fused multiply-add read with its admitted rounding mode and three operands in source order. | Test |
| FR-001-AC-2 | A checked `quire.op.ieee.float64.fma` node projects as a typed float64 fused multiply-add read with its admitted rounding mode and three operands in source order; the read remains distinguishable from float32. | Test |
| FR-001-AC-3 | A checked `quire.op.ieee.total_order` node projects as a typed two-operand total-order read, distinct from ordinary ordering and numeric equality. | Test |
| FR-001-AC-4 | A checked `quire.op.ieee.bit_identical` node projects as a typed two-operand bit-identity read, distinct from total order and numeric equality. | Test |
| FR-001-AC-5 | An admitted `quire.op.ieee.total_order` node carries `OrderedKind::Float` through its typed borrowed read, while its identity remains distinct from ordinary `NodeKind::Order`; no IEEE catalogue identity is represented as ordinary `NodeKind::Order`. | Test |
| FR-001-AC-6 | Each identity in the table has a corresponding checked-node representation whose typed read retains that identity without a local identity alias or copied catalogue. | Inspection |

## Dependencies

- The authoritative operation identities are in the contract-ir catalogue.
- IEEE operation semantics and operand admission are in QSpec. The checked
  reader retains the admitted result and does not repeat those decisions.
