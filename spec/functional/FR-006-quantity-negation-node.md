---
id: FR-006
title: Read the checked quantity-negation operation
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-142
    type: implements
  - target: ix://agent-ix/quire-specification/FR-322
    type: implements
---
# FR-006: Read the checked quantity-negation operation

## Description

The checked node vocabulary SHALL represent unary quantity negation as
`QuantityNegate(Child)`, distinct from integer, rational and decimal negation
and from binary quantity arithmetic. The descriptor carries one operand and
no unit, law, mode, member or magnitude field.

## Behavior

A borrowed projection SHALL visit the operand exactly once, retain its mapped
handle, and borrow the supplied checked result type and location unchanged.
The descriptor SHALL not choose a unit, convert a magnitude, evaluate the
operation or infer admission from its shape. The admitted operation's result
is the operand's `Quantity<Unit>` type and unit; QSpec owns the non-affine
operand restriction and exact, total, unit-preserving semantics. QSL owns
lowering to this node, and `quire-semantic-value` owns the quantity value
operation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-006-AC-1 | A quantity-negation read retains a distinct unary `QuantityNegate` operation and maps its one operand exactly once, including when its source handle differs from the read node handle. | Test |
| FR-006-AC-2 | The read borrows the supplied checked result type and location without reconstructing either metadata value; the node adds no unit, law, mode, member or magnitude field. | Test, Inspection |

## Dependencies

- QSpec FR-142 "Quantity negation" owns exact, total, unit-preserving evaluation and affine-unit refusal.
- QSpec FR-322-AC-119 owns checked-package operation admission and its single-operand `result operand:0` typing rule.
- QSpec FR-146-AC-9 owns the nonzero divisor fact for negated quantity literals.
- QSL owns unary-minus lowering, and `quire-semantic-value` owns the quantity value operation.
