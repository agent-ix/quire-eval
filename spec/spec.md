---
type: master-requirements
name: quire-eval
org: agent-ix
component_type: rust-library
implementation_language: rust
depends_on:
  - quire-contract-ir
  - quire-exact
  - quire-semantic-value
standards_alignment:
  - iso-iec-ieee-29148
---
# Master Requirements Specification

## System Overview

`quire-checked` exposes admitted expression nodes to downstream evaluation
consumers; `quire-eval` executes checked expressions. This initial specification
records the checked IEEE vocabulary slice.

## Purpose

This specification owns the public checked-node vocabulary and evaluation
interfaces of `quire-eval` and `quire-checked`. The checked-node vocabulary
retains distinctions established by the authoritative contract-ir operation
catalogue so that a downstream evaluator can inspect an admitted term without
reinterpreting its wire form.

## Scope

The current requirement inventory covers the IEEE checked-node vocabulary in
[FR-001](functional/FR-001-ieee-checked-node-vocabulary.md), derived from
[US-001](usecase/US-001-inspect-ieee-checked-nodes.md). FR-001 is implemented by
the checked-node descriptors and borrowed projections. The connective origin carried on checked connective nodes is specified in
[FR-002](functional/FR-002-checked-connective-origin.md), derived from
[US-002](usecase/US-002-distinguish-authored-connectives.md). Other existing crate
behavior is outside this initial inventory and is not assigned a new requirement
by this change.

The IEEE arithmetic, comparison, rounding, and diagnostic semantics remain
owned by QSpec; contract-ir owns operation identity and admission. This
repository owns the typed checked representation and its borrowed projection.

## Requirements Architecture

The [user story](usecase/US-001-inspect-ieee-checked-nodes.md) records the
consumer need. The [functional requirement](functional/FR-001-ieee-checked-node-vocabulary.md)
defines the implemented typed-read obligations and acceptance criteria.

## Verification

Each acceptance criterion in a functional requirement names its verification
method. Tests cite the criterion ID that they assert. The test matrix is
computed from those citations.

## References

- `ix://agent-ix/quire-contract-ir` — authoritative operation catalogue.
- `ix://agent-ix/quire-specification/FR-148` — IEEE evaluation semantics.
- `ix://agent-ix/quire-specification/FR-322` — checked-package operation identities.
