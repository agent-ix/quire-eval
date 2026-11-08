// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-400 canonical operation projection tests.

use quire_checked::node::{CaseArm, CheckedLiteral, RecordSlot};
use quire_checked::{NodeKind, NodeRead};
use quire_exact::{Value, ValueType};
use quire_semantic_value::location::{ExpressionLocation, ExpressionOwner};

/// Trace: FR-400-AC-11
#[test]
fn nested_children_and_operand_order_survive_projection() {
    let record = NodeKind::<u8, ()>::Record {
        declaration: quire_exact::NodeKey::new(
            "fixture",
            vec![quire_exact::PathSegment::Text("n".into())],
            0,
        )
        .unwrap(),
        slots: vec![
            RecordSlot::Absent,
            RecordSlot::Present(2),
            RecordSlot::Null,
            RecordSlot::Present(5),
        ],
    };
    let projected = record.map_ref(|child| usize::from(*child) + 10, |_| unreachable!());
    let NodeKind::Record { slots, .. } = projected else {
        panic!("record changed operation")
    };
    assert!(matches!(
        slots.as_slice(),
        [
            RecordSlot::Absent,
            RecordSlot::Present(12),
            RecordSlot::Null,
            RecordSlot::Present(15)
        ]
    ));

    let fold = NodeKind::<u8, ()>::Fold {
        accumulator: 7,
        binder: 8,
        source: 1,
        step: 3,
        identity: Some(2),
    };
    let mut seen = Vec::new();
    let projected = fold.map_ref(
        |child| {
            seen.push(*child);
            usize::from(*child) + 10
        },
        |_| unreachable!(),
    );
    let NodeKind::Fold {
        accumulator,
        binder,
        source,
        step,
        identity,
    } = projected
    else {
        panic!("fold changed operation")
    };
    assert_eq!(
        (accumulator, binder, source, step, identity),
        (7, 8, 11, 13, Some(12))
    );
    assert_eq!(seen, [1, 2, 3]);

    let case = NodeKind::<u8, ()>::Case {
        scrutinee: 1,
        declaration: quire_exact::NodeKey::new(
            "fixture",
            vec![quire_exact::PathSegment::Text("union".into())],
            0,
        )
        .unwrap(),
        arms: vec![CaseArm {
            member: 3,
            binders: vec![7, 8],
            body: 4,
        }],
    };
    let projected = case.map_ref(|child| usize::from(*child) + 10, |_| unreachable!());
    let NodeKind::Case {
        scrutinee, arms, ..
    } = projected
    else {
        panic!("case changed operation")
    };
    assert_eq!(scrutinee, 11);
    assert_eq!(arms.len(), 1);
    assert_eq!(arms[0].member, 3);
    assert_eq!(arms[0].binders, [7, 8]);
    assert_eq!(arms[0].body, 14);
}

/// Trace: FR-400-AC-11
#[test]
fn borrowed_literal_and_checked_metadata_have_one_read_shape() {
    let value_type = ValueType::Boolean;
    let location = ExpressionLocation::root(ExpressionOwner::Expression);
    let stored = NodeKind::Literal(CheckedLiteral(Value::Boolean(true)));
    let read = NodeRead::<usize>::from_stored(&stored, &value_type, &location, |_| unreachable!());
    let NodeKind::Literal(literal) = read.operation else {
        panic!("literal changed operation")
    };
    assert!(matches!(literal, Value::Boolean(true)));
    let NodeKind::Literal(stored_literal) = &stored else {
        panic!("stored literal changed operation")
    };
    assert!(core::ptr::eq(literal, &stored_literal.0));
    assert!(core::ptr::eq(read.value_type, &value_type));
    assert!(core::ptr::eq(read.location, &location));
}

/// Trace: FR-400-AC-11
#[test]
fn explicit_dereference_charge_flag_survives_projection() {
    let field = quire_semantic_value::declaration::FieldRef::new(
        quire_exact::EffectiveId::new(
            None,
            quire_exact::OriginalDeclarationKey::new("fixture", "object"),
        ),
        "total",
    );
    for derefed in [false, true] {
        let stored = NodeKind::<u8, ()>::Attribute {
            reference: 4,
            field: field.clone(),
            optional: false,
            derefed,
        };
        let projected = stored.map_ref(|child| usize::from(*child), |_| unreachable!());
        let NodeKind::Attribute {
            reference,
            field: observed,
            optional,
            derefed: observed_flag,
        } = projected
        else {
            panic!("attribute changed operation")
        };
        assert_eq!(reference, 4);
        assert_eq!(observed, field);
        assert!(!optional);
        assert_eq!(observed_flag, derefed);
    }
}

/// Trace: FR-400-AC-11
#[test]
fn checked_literal_preserves_non_keyed_float_identity() {
    let one = CheckedLiteral(Value::Float(quire_exact::IeeeValue::binary64(
        0x7ff8_0000_0000_0001,
    )));
    let other = CheckedLiteral(Value::Float(quire_exact::IeeeValue::binary64(
        0x7ff8_0000_0000_0002,
    )));
    assert_eq!(one, one.clone());
    assert_ne!(one, other);
}

/// Trace: FR-400-AC-11
#[cfg(feature = "construct")]
#[test]
fn admitted_division_selection_survives_projection() {
    let selection =
        quire_checked::construct::admitted_integer_division(quire_exact::DivisionProfile::Floor);
    let stored = NodeKind::<u8, ()>::IntegerDivision {
        operator: quire_checked::node::IntegerDivisionOperator::Rem,
        selection: Some(selection),
        left: 1,
        right: 2,
    };
    let projected = stored.map_ref(|child| usize::from(*child) + 10, |_| unreachable!());
    let NodeKind::IntegerDivision {
        operator,
        selection,
        left,
        right,
    } = projected
    else {
        panic!("division changed operation")
    };
    assert_eq!(operator, quire_checked::node::IntegerDivisionOperator::Rem);
    assert_eq!(
        selection.unwrap().profile(),
        quire_exact::DivisionProfile::Floor
    );
    assert_eq!((left, right), (11, 12));
}
