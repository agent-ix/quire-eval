// SPDX-License-Identifier: AGPL-3.0-or-later
//! The unsupported-entry and prepare-limit arms of `CallFailure` (FR-007).

use quire_eval::{
    CallFailure, ExecutionBackendChoice, PrepareLimitReached, PrepareSetting, UnsupportedCause,
    UnsupportedEntry, UntranslatableConstruct,
};
use quire_exact::{DecimalType, Integer, NodeKey, PathSegment, RoundingMode, ValueType};
use quire_semantic_value::catalog::Category;
use quire_semantic_value::location::{ExpressionLocation, ExpressionOwner};

fn location() -> ExpressionLocation {
    ExpressionLocation::at(ExpressionOwner::Expression, &[2, 0])
}

fn node_key() -> NodeKey {
    NodeKey::new("pkg", vec![PathSegment::Text(String::from("op"))], 3).expect("valid key")
}

fn untranslatable(construct: UntranslatableConstruct) -> UnsupportedEntry {
    UnsupportedEntry {
        backend: ExecutionBackendChoice::Jit,
        cause: UnsupportedCause::Untranslatable {
            construct,
            location: location(),
        },
    }
}

fn not_supplied() -> UnsupportedEntry {
    UnsupportedEntry {
        backend: ExecutionBackendChoice::Jit,
        cause: UnsupportedCause::NotSupplied,
    }
}

/// Trace: FR-007-AC-1.
#[test]
fn an_unsupported_entry_is_category_unsupported_with_the_unimplemented_capability_code() {
    let entry = untranslatable(UntranslatableConstruct::ExpressionForm(String::from("let")));
    for entry in [entry, not_supplied()] {
        let failure = CallFailure::Unsupported(entry.clone());
        assert_eq!(failure.category(), Category::Unsupported);
        assert_eq!(entry.catalog_code().code(), "unimplemented_capability");
        assert_eq!(CallFailure::from(entry.clone()), failure);
        assert_eq!(failure.clone(), failure);
    }
}

/// Trace: FR-007-AC-2.
#[test]
fn an_untranslatable_entry_carries_backend_construct_name_and_location() {
    let entry = untranslatable(UntranslatableConstruct::ExpressionForm(String::from("let")));
    assert_eq!(entry.catalog_code().cause(), "untranslatable-entry");
    let payload = entry.payload();
    assert_eq!(payload.backend, "jit");
    assert_eq!(payload.construct, Some("expression-form"));
    assert_eq!(payload.construct_name.as_deref(), Some("let"));
    assert_eq!(payload.location, Some(&location()));
}

/// Trace: FR-007-AC-2.
#[test]
fn a_value_type_and_an_operation_construct_name_their_kind() {
    let decimal = DecimalType::new(
        Integer::from(0_i64),
        Integer::from(9_i64),
        0,
        2,
        RoundingMode::Exact,
    )
    .expect("valid decimal type");
    let by_type = untranslatable(UntranslatableConstruct::ValueType(Box::new(
        ValueType::Decimal(decimal),
    )));
    let payload = by_type.payload();
    assert_eq!(payload.construct, Some("value-type"));
    assert_eq!(payload.construct_name.as_deref(), Some("Decimal"));

    let key = node_key();
    let by_operation = untranslatable(UntranslatableConstruct::Operation(key.clone()));
    let payload = by_operation.payload();
    assert_eq!(payload.construct, Some("operation"));
    assert_eq!(payload.construct_name, Some(key.to_string()));
}

/// Trace: FR-007-AC-2.
#[test]
fn a_backend_that_was_not_supplied_has_a_backend_and_no_construct_or_location() {
    let entry = not_supplied();
    assert_eq!(entry.catalog_code().code(), "unimplemented_capability");
    assert_eq!(entry.catalog_code().cause(), "backend-not-supplied");
    let payload = entry.payload();
    assert_eq!(payload.backend, "jit");
    assert_eq!(payload.construct, None);
    assert_eq!(payload.construct_name, None);
    assert_eq!(payload.location, None);
}

/// Trace: FR-007-AC-2.
#[test]
fn every_backend_has_its_lower_case_name() {
    assert_eq!(ExecutionBackendChoice::Interpreter.name(), "interpreter");
    assert_eq!(ExecutionBackendChoice::Aot.name(), "aot");
    assert_eq!(ExecutionBackendChoice::Jit.name(), "jit");
}

/// Trace: FR-007-AC-3.
#[test]
fn a_reached_prepare_limit_is_incomplete_with_the_stage_limit_exceeded_code() {
    let reached = PrepareLimitReached {
        setting: PrepareSetting::CodeBytes,
        bound: 1,
        actual: 2,
    };
    let failure = CallFailure::Limit(reached);
    assert_eq!(failure.category(), Category::Incomplete);
    assert_eq!(reached.code(), "stage_limit_exceeded");
    assert_eq!(CallFailure::from(reached), failure);
    assert_ne!(
        failure,
        CallFailure::Limit(PrepareLimitReached {
            setting: PrepareSetting::CompileWorkUnits,
            bound: 1,
            actual: 2,
        })
    );
}

/// Trace: FR-007-AC-3.
#[test]
fn prepare_settings_have_their_published_names() {
    assert_eq!(
        PrepareSetting::CompileWorkUnits.name(),
        "prepare.compile_work_units"
    );
    assert_eq!(PrepareSetting::CodeBytes.name(), "prepare.code_bytes");
}

/// Trace: FR-296-AC-4.
#[test]
fn the_default_backend_choice_is_the_interpreter() {
    assert_eq!(
        ExecutionBackendChoice::default(),
        ExecutionBackendChoice::Interpreter
    );
}
