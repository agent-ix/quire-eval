---
id: FR-002
title: Retain connective origin in checked node reads
type: FR
status: implemented
relationships:
  - target: "ix://agent-ix/quire-eval/US-002"
    type: implements
---
# FR-002: Retain connective origin in checked node reads

## Description

When a checked expression contains a boolean connective, `quire-checked` SHALL
retain on that node whether the connective is authored or synthesized by the
checker, and a borrowed node read SHALL expose the same origin together with
the connective operator and the left and right operands in order.

## Behavior

`NodeKind::Connective` carries the operator, a `ConnectiveOrigin`
(`Authored` or `Synthesized`), and two ordered operands. The origin is part of
node equality: an authored and a synthesized connective with the same operator
and operands are different nodes. Producers state the origin explicitly; a
consumer never derives it from locations.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | A borrowed read of a stored connective node retains its operator and its left and right operands in source order. | Test |
| FR-002-AC-2 | A borrowed read of a stored connective node exposes `ConnectiveOrigin::Authored` for an authored node and `ConnectiveOrigin::Synthesized` for a synthesized node. | Test |
| FR-002-AC-3 | Two connective nodes that differ only in origin are not equal. | Test |

## Dependencies

- The decision-recording semantics that consume the origin are owned by the
  QSL evaluator specification; this requirement defines only the typed carrier.
