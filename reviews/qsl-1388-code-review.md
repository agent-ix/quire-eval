---
id: SR-17040
title: "Code and Rust review: ConnectiveOrigin on checked connective nodes (QSL-1388, PR #18)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-eval PR #18 @730deae543360514f239da9bdda60a38694af214; crates/quire-checked/src/node.rs, crates/quire-checked/tests/node_projection.rs; FR-002-AC-1 through FR-002-AC-3"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-eval/FR-002", type: references }
---
# Code and Rust review: ConnectiveOrigin on checked connective nodes

## Summary

Ticket: QSL-1388 (related IR-1276). The PR adds `ConnectiveOrigin { Authored, Synthesized }` and the variant `NodeKind::Connective(Connective, ConnectiveOrigin, Child, Child)`. `map_ref` forwards the origin unchanged. Rust-review lane folded in.

## Verdict

PASS, mergeable. The carrier is the smallest typed one: a two-variant `Copy` enum, positional on the existing variant, no default and no shim. `NodeKind` derives only `Clone, Debug, PartialEq` (no Hash, Ord or serde), so origin enters equality only. A repo-wide grep finds the variant only in node.rs and the two test files; the repo has no graph construction, no interning and no keyed use of `NodeKind`. Mutation checks (force `Authored` in map_ref, swap operands, flip operator) each fail `connective_read_retains_origin_and_operand_order`. fmt, clippy (default, all-features, both no-std lib targets), doc -D warnings, test (default and all-features) and cargo deny pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

- Sharing risk: this repo never compares or dedups nodes. In QSL origin/main d1487ae the body builder (qsl-semantics/src/check/ir.rs BodyBuilder::push) appends without dedup, and the only interning found is the temporal formula arena keyed on its own Node type. So equal authored and synthesized applications stay two NodeIds as before; the change cannot alter sharing there. The risk exists only if a future consumer hash-conses on NodeKind, which would need a Hash derive the type lacks.
- Downstream QSL consumers pattern-match the old three-field variant (about 15 sites in qsl-eval, qsl-replay, qsl-semantics); they fail to compile until QSL adopts the new arity, which is intended (no compat layer).
- The AC-3 test exercises derived PartialEq; it fails if origin were removed from the variant, which is the regression it guards.
