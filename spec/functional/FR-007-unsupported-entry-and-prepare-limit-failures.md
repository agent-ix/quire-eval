---
id: FR-007
title: Name the unsupported-entry and prepare-limit call failures
type: FR
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-401
    type: implements
  - target: ix://agent-ix/quire-spec-language/FR-294
    type: implements
  - target: ix://agent-ix/quire-spec-language/FR-296
    type: implements
  - target: ix://agent-ix/quire-spec-language/FR-279
    type: implements
---
# FR-007: Name the unsupported-entry and prepare-limit call failures

## Description

`CallFailure` SHALL have the arm `Unsupported(UnsupportedEntry)` for an entry
the chosen execution backend cannot run, and the arm `Limit(PrepareLimitReached)`
for a prepare bound that a backend's `prepare` reached. Both payloads belong to
this crate so that it names no item of a QSL crate. No evaluator in this crate
returns either arm; only a non-interpreter backend and QSL's `execute` do.

## Behavior

1. `UnsupportedEntry { backend, cause }` SHALL name an `ExecutionBackendChoice`
   (`Interpreter`, `Aot` or `Jit`) and an `UnsupportedCause`: `NotSupplied`, or
   `Untranslatable { construct, location }` with an `UntranslatableConstruct`
   (`ExpressionForm(name)`, `ValueType(Box<ValueType>)` or `Operation(NodeKey)`) and
   an `ExpressionLocation`. `CallFailure::category` SHALL return
   `Category::Unsupported` for the arm, and its catalog code SHALL be
   `unimplemented_capability` with the cause `backend-not-supplied` for
   `NotSupplied` and `untranslatable-entry` for `Untranslatable`.
2. The diagnostic payload SHALL carry `backend`, the backend's lower-case name,
   and for `Untranslatable` also `construct` (`expression-form`, `value-type` or
   `operation`), `construct_name` (the form name, the value type's canonical
   name, or the operation's node key) and `location`.
3. `PrepareLimitReached { setting, bound, actual }` SHALL hold a
   `PrepareSetting` (`CompileWorkUnits` or `CodeBytes`, whose `name()` is
   `prepare.compile_work_units` or `prepare.code_bytes`), the configured bound
   and the counter the refused charge reached, as `u64`s, and no locus.
   `CallFailure::category` SHALL return `Category::Incomplete` for the arm, and
   its catalog code SHALL be `stage_limit_exceeded`.

All payloads are `Clone`, `Eq` and `PartialEq`, like `CallFailure`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-007-AC-1 | `CallFailure::Unsupported` over each of `NotSupplied` and `Untranslatable` has category `Category::Unsupported` and catalog code `unimplemented_capability`, and converts from its `UnsupportedEntry`. | Test |
| FR-007-AC-2 | An `UnsupportedEntry`'s catalog cause is `untranslatable-entry` or `backend-not-supplied`; its payload names the `backend` and, for `Untranslatable`, the `construct` kind, `construct_name` and `location` of an expression form, a value type (`Decimal`) and an operation node key, and carries none of them for `NotSupplied`. | Test |
| FR-007-AC-3 | `CallFailure::Limit` has category `Category::Incomplete` and catalog code `stage_limit_exceeded`, holds the setting, bound and actual unchanged, and `PrepareSetting::name()` is `prepare.compile_work_units` or `prepare.code_bytes`. | Test |

## Dependencies

- QSL FR-401, FR-294, FR-296 and FR-279 own the interface and the arms' use.
- `quire-semantic-value` owns `Category`, `CatalogCode` and `ExpressionLocation`.
