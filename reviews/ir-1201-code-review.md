---
id: SR-17030
title: 'IR-1201 PR 15 code review'
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-eval; IR-1201; Cargo.lock, crates/quire-checked/src/lib.rs, crates/quire-checked/src/node.rs, crates/quire-checked/tests/node_read.rs; FR-400-AC-11, FR-400-AC-12'
review_set: subset
---

## Summary

Reviewed the frozen c2354b45 PR diff with the Rust review lane against QSL origin/main FR-400, FR-390 and ADR-011. The new vocabulary cannot yet serve as the single canonical checked operation API: imported-call identity is narrowed, the specified root path is absent, and QSL retains a second operation catalog.

## Verdict

**FAIL** — three high findings require correction or an explicit scope decision before merge.

## Coverage

- FR-400-AC-11 examined: “A stored `NodeKind` and its `NodeRead` projection match the same exhaustive generic enum: every variant, nested operand position, literal value, metadata/evidence identity, checked type and location are preserved. A probe variant makes both evaluator paths non-exhaustive. No parallel operation enum exists; the layer-C read descriptors compile without `std`, QSL or IR dependencies.”
- FR-400-AC-12 examined: “Constructing a `NodeRead` or a parameterized operation descriptor cannot create a checked term or an admitted FR-390 view: passing a descriptor to the public evaluator as a checked input fails to compile. Existing checked-term constructor and mutation rejection checks remain failing, with their positive pairs compiling.”
- FR-390 checked-term view statement examined as context: the layer-4 view returns `quire_checked::NodeRead<'v, TermHandle<'v>>` and imported call metadata retains exact callee identity and closure association.
- ADR-011 §6.1 layer C examined as context: the checked-term vocabulary lives below QSL layers 4 and 5.
- Focused checks: `cargo test -p quire-checked --test node_read --locked` (4 passed), same with `--features construct` (5 passed), `cargo check -p quire-checked --no-default-features --locked` (passed), `git diff --check` (passed). Aggregate gate reserved for final PR head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `ImportedCall.callee` drops the package-qualified identity required to select the admitted dependency closure | crates/quire-checked/src/node.rs:412 |
| FND-002 | high | `NodeRead` is absent from the specified `quire_checked::NodeRead` public path | crates/quire-checked/src/lib.rs:20 |
| FND-003 | high | A second `NodeKind` catalog remains in QSL, so this change leaves two independently edited operation vocabularies | crates/quire-checked/src/node.rs:247 |

## Evidence

- FND-001: QSL origin/main `qsl-semantics/src/library/mod.rs:193-199` defines `PackageNodeKey { package: PackageId, node: WireNodeId }`. Its `qsl-eval/src/value/expression/evaluate.rs:1201` uses `callee.package` to choose the imported graph, and lowering emits both components. The new `NodeKey` field carries neither package identity nor the original wire node identity. A function index is only local to the selected imported graph.
- FND-002: FR-390's `read_node` signature names the root path exactly. `lib.rs` only declares `pub mod node`; there is no root `pub use`. A downstream `use quire_checked::NodeRead` fails to resolve.
- FND-003: QSL origin/main `qsl-semantics/src/check/ir.rs:294` still defines its own `NodeKind` with overlapping variants. The newly introduced enum is therefore a second source of truth for operation semantics, contrary to FR-400-AC-11 and the no-duplication rule in the code-review skill. The two already differ in `ImportedCall` metadata.
