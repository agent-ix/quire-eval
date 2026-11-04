---
id: FR-004
title: Read the checked quantity integer-power operation
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-322
    type: implements
---
# FR-004: Read the checked quantity integer-power operation

## Description

The checked node vocabulary SHALL represent quantity integer power separately
from two-quantity arithmetic, as `QuantityPower(base, exponent)`.

## Behavior

Borrowed projection SHALL visit the base before the exponent exactly once,
retain their mapped handles, and borrow the supplied checked result type and
location unchanged. Projection SHALL neither execute the power operation nor
infer admission from the descriptor. The canonical value operation remains
`quire_semantic_value::quantity::QuantityOperation::Power`; this node adds no
replacement value-operation enum.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-004-AC-1 | A quantity-power read retains the distinct operation and mapped base/exponent handles in that order, including when the source handles have descending indices. | Test |
| FR-004-AC-2 | The read borrows the original checked type and location without reconstructing either metadata value. | Test |

## Dependencies

- QSpec FR-322-AC-383 owns package admission, operand restrictions and result-unit checking.
- QSpec TC-187 owns quantity-power semantic vectors.
- `quire-semantic-value` owns quantity value operations and exact integer exponents.
- IR owns admitted-package projection into this checked vocabulary.

Descriptor controls do not claim package admission or execution qualification.
