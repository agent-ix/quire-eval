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
store on that node a `ConnectiveOrigin` stated by the producer of the node.

## Behavior

`NodeKind::Connective` carries the operator, a `ConnectiveOrigin`, and two
ordered operands.

- `ConnectiveOrigin::Authored` marks a connective node that the producer builds
  directly from a connective operator of the clause it is checking.
- `ConnectiveOrigin::Synthesized` marks a connective that the producer
  introduces to compose another construct, for example the composition of a
  float ordering from simpler comparisons.

The producer states the origin when it builds the node.

A borrowed node read SHALL expose the stored origin, the operator, and the left
and right operands in order.

A consumer of a connective node SHALL NOT derive its origin from location data;
the origin is read from the node.

The origin is part of node equality: two connective nodes with the same
operator and operands but different origins are different nodes.

The decision-recording semantics that depend on the origin are specified by the
requirements of the consuming evaluator (QSL FR-265, to be amended); this
requirement defines only the typed carrier.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | A borrowed read of a stored connective node exposes the operator the node was stored with. | Test |
| FR-002-AC-2 | A borrowed read of a stored connective node exposes its left and right operands in stored order. | Test |
| FR-002-AC-3 | A borrowed read of a node stored with `ConnectiveOrigin::Authored` exposes `Authored`. | Test |
| FR-002-AC-4 | A borrowed read of a node stored with `ConnectiveOrigin::Synthesized` exposes `Synthesized`. | Test |
| FR-002-AC-5 | A borrowed read of the same stored connective node yields the same origin whichever location the read is given; this guards against a regression that makes the exposed origin depend on the location. | Test |
| FR-002-AC-6 | Two connective nodes that differ only in origin are not equal. | Test |

## Dependencies

- The consuming evaluator's requirements own what is recorded for each origin.
