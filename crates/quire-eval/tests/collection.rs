// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native shared-kernel collection removal controls (FR-003).

mod support;

use ix_trace_rs::trace;
use quire_eval::collection::{drop_first, remove_one, CollectionFailure};
use quire_exact::{
    collection::{form_collection, form_grouped, member_equal},
    CardinalityBound, ChargePoint, CollectionKind, CollectionType, CollectionValue, FloatType,
    IeeeValue, IeeeWidth, IllTyped, IllTypedCause, InjectedDenial, Meter, Outcome, Value,
    ValueType,
};
use quire_semantic_value::declaration::{CompositeDeclaration, CompositeShape, TypeEnvironment};

fn source(kind: CollectionKind, bound: Option<CardinalityBound>, values: &[bool]) -> Value {
    form_collection(
        &CollectionType::new(kind, ValueType::Boolean, bound),
        values.iter().copied().map(Value::Boolean).collect(),
        &mut support::meter(10_000),
    )
    .expect("typed source")
    .completed()
    .expect("source formation")
}

fn collection(value: &Value) -> &CollectionValue {
    let Value::Collection(value) = value else {
        panic!("collection result")
    };
    value
}

fn bits(value: &Value) -> Vec<bool> {
    collection(value)
        .elements()
        .iter()
        .map(|value| {
            let Value::Boolean(value) = value else {
                panic!("Boolean element")
            };
            *value
        })
        .collect()
}

fn completed(result: Result<Outcome<Value>, CollectionFailure>) -> Value {
    result
        .expect("operation admits")
        .completed()
        .expect("operation completes")
}

#[trace("FR-003-AC-2", "FR-003-AC-6")]
#[test]
fn drop_keeps_survivor_order_and_charges_formation_only() {
    for (kind, expected) in [
        (CollectionKind::Sequence, vec![false, true]),
        (CollectionKind::OrderedSet, vec![false]),
    ] {
        let input = source(kind, None, &[true, false, true]);
        let mut meter = support::meter(10_000);
        let result = completed(drop_first(collection(&input), &mut meter));
        assert_eq!(bits(&result), expected);
        assert_eq!(collection(&result).collection_type().kind(), kind);
        assert_eq!(
            meter
                .admitted_charges()
                .iter()
                .map(|c| c.point)
                .collect::<Vec<_>>(),
            [
                ChargePoint::CollectionBound,
                ChargePoint::CollectionResultRetain
            ]
        );
        let mut expected_meter = support::meter(10_000);
        let target = CollectionType::new(kind, ValueType::Boolean, None);
        assert!(matches!(form_grouped(&target, expected.iter().copied().map(Value::Boolean).collect(), &mut expected_meter), Outcome::Completed(_)));
        assert_eq!(meter.admitted_charges(), expected_meter.admitted_charges());
        assert_eq!(meter.consumed(quire_exact::LimitKind::ResultUnits), expected_meter.consumed(quire_exact::LimitKind::ResultUnits));
        let empty = source(kind, Some(CardinalityBound::at_most(0)), &[]);
        assert!(bits(&completed(drop_first(
            collection(&empty),
            &mut support::meter(10_000)
        )))
        .is_empty());
    }
    for kind in [CollectionKind::Set, CollectionKind::Bag] {
        let input = source(kind, None, &[true, false]);
        let mut meter = support::meter(10_000);
        assert_eq!(
            drop_first(collection(&input), &mut meter),
            Err(CollectionFailure::IllTyped(IllTyped {
                cause: IllTypedCause::OperatorIneligible
            }))
        );
        assert!(meter.admitted_charges().is_empty());
    }
}

#[trace("FR-003-AC-3")]
#[test]
fn remove_drops_only_one_match_and_preserves_misses_and_empty_inputs() {
    let environment = TypeEnvironment::default();
    for (kind, expected) in [
        (CollectionKind::Sequence, vec![false, true]),
        (CollectionKind::OrderedSet, vec![false]),
        (CollectionKind::Set, vec![false]),
        (CollectionKind::Bag, vec![false, true]),
    ] {
        let input = source(kind, None, &[true, false, true]);
        let result = completed(remove_one(
            collection(&input),
            &Value::Boolean(true),
            &environment,
            &mut support::meter(10_000),
        ));
        assert_eq!(bits(&result), expected, "{kind:?}");
        assert_eq!(collection(&result).collection_type().kind(), kind);
        for values in [&[true, true][..], &[][..]] {
            let input = source(kind, None, values);
            let result = completed(remove_one(
                collection(&input),
                &Value::Boolean(false),
                &environment,
                &mut support::meter(10_000),
            ));
            assert_eq!(bits(&result), bits(&input), "miss for {kind:?}");
        }
    }
}

#[trace("FR-003-AC-4")]
#[test]
fn result_bounds_follow_the_operation_and_preserve_unboundedness() {
    for kind in CollectionKind::ALL {
        for (bound, values) in [
            (None, &[true, false][..]),
            (
                Some(CardinalityBound::new(2, 3).unwrap()),
                &[true, false][..],
            ),
            (Some(CardinalityBound::at_most(0)), &[][..]),
            (Some(CardinalityBound::at_most(u64::MAX)), &[][..]),
        ] {
            let input = source(kind, bound, values);
            let result = completed(remove_one(
                collection(&input),
                &Value::Boolean(true),
                &TypeEnvironment::default(),
                &mut support::meter(10_000),
            ));
            let actual = collection(&result).collection_type();
            assert_eq!(actual.kind(), kind);
            assert_eq!(actual.element(), &ValueType::Boolean);
            assert_eq!(
                actual.bound(),
                bound.map(
                    |b| CardinalityBound::new(b.minimum().saturating_sub(1), b.maximum()).unwrap()
                )
            );
            if matches!(kind, CollectionKind::Sequence | CollectionKind::OrderedSet) {
                let result = completed(drop_first(collection(&input), &mut support::meter(10_000)));
                assert_eq!(
                    collection(&result).collection_type().bound(),
                    bound.map(|b| CardinalityBound::new(
                        b.minimum().saturating_sub(1),
                        b.maximum().saturating_sub(1)
                    )
                    .unwrap())
                );
            }
        }
    }
}

#[trace("FR-003-AC-5")]
#[test]
fn a_wrong_probe_type_refuses_before_scan_or_formation() {
    let input = source(CollectionKind::Sequence, None, &[true]);
    let mut meter = support::meter(10_000);
    assert_eq!(
        remove_one(
            collection(&input),
            &Value::Integer(1_i64.into()),
            &TypeEnvironment::default(),
            &mut meter
        ),
        Err(CollectionFailure::IllTyped(IllTyped {
            cause: IllTypedCause::TypeMismatch
        }))
    );
    assert!(meter.admitted_charges().is_empty());
}

#[trace("FR-003-AC-5")]
#[test]
fn removal_uses_schema_conformance_instead_of_exact_leaf_membership() {
    let key = |name: &str| quire_exact::NodeKey::new("fixture", vec![quire_exact::PathSegment::Text(name.into())], 0).unwrap();
    let parent = key("parent");
    let child = key("child");
    let environment = TypeEnvironment::new([
        CompositeDeclaration::new(parent.clone(), "Parent", CompositeShape::Record(vec![])),
        CompositeDeclaration::new(child.clone(), "Child", CompositeShape::Record(vec![])).with_supertypes(vec![parent.clone()]),
    ], []).expect("the subtype registry admits");
    let value = environment.record(child, vec![]).expect("the child record admits");
    let element_type = ValueType::Composite(parent);
    assert!(!element_type.admits(&value), "exact membership is insufficient");
    assert_eq!(environment.admits(&element_type, &value), Ok(true));
    let ty = CollectionType::new(CollectionKind::Sequence, element_type, None);
    let input = form_grouped(&ty, vec![value.clone()], &mut support::meter(10_000)).completed().unwrap();
    let result = completed(remove_one(collection(&input), &value, &environment, &mut support::meter(10_000)));
    assert!(collection(&result).elements().is_empty());
    assert_eq!(collection(&result).collection_type(), &ty);
}

#[trace("FR-003-AC-3", "FR-003-AC-6")]
#[test]
fn a_bag_miss_scans_one_distinct_member_then_forms_without_recomparison() {
    let input = source(CollectionKind::Bag, None, &[true, true, true]);
    let mut meter = support::meter(10_000);
    let result = completed(remove_one(
        collection(&input),
        &Value::Boolean(false),
        &TypeEnvironment::default(),
        &mut meter,
    ));
    assert_eq!(bits(&result), [true, true, true]);
    assert_eq!(
        meter
            .admitted_charges()
            .iter()
            .map(|c| c.point)
            .collect::<Vec<_>>(),
        [
            ChargePoint::CollectionMemberWalk,
            ChargePoint::CollectionMemberTest,
            ChargePoint::CollectionBound,
            ChargePoint::CollectionResultRetain,
        ]
    );
    let mut expected = support::meter(10_000);
    assert_eq!(
        member_equal(&Value::Boolean(false), &Value::Boolean(true), &mut expected),
        Outcome::Completed(false)
    );
    let target = CollectionType::new(CollectionKind::Bag, ValueType::Boolean, None);
    assert!(matches!(
        form_grouped(&target, vec![Value::Boolean(true); 3], &mut expected),
        Outcome::Completed(_)
    ));
    assert_eq!(meter.admitted_charges(), expected.admitted_charges());
    assert_eq!(
        meter.consumed(quire_exact::LimitKind::WorkUnits),
        expected.consumed(quire_exact::LimitKind::WorkUnits)
    );
    assert_eq!(
        meter.consumed(quire_exact::LimitKind::ResultUnits),
        expected.consumed(quire_exact::LimitKind::ResultUnits)
    );
}

#[trace("FR-003-AC-3")]
#[test]
fn floating_membership_removes_the_matching_nan_payload_only() {
    let first = Value::Float(IeeeValue::binary64(0x7ff8_0000_0000_0001));
    let second = Value::Float(IeeeValue::binary64(0x7ff8_0000_0000_0002));
    let ty = CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Float(FloatType::exact(IeeeWidth::Binary64)),
        None,
    );
    let input = form_collection(
        &ty,
        vec![first.clone(), second.clone(), first.clone()],
        &mut support::meter(10_000),
    )
    .unwrap()
    .completed()
    .unwrap();
    let result = completed(remove_one(
        collection(&input),
        &first,
        &TypeEnvironment::default(),
        &mut support::meter(10_000),
    ));
    assert_eq!(collection(&result).elements(), [second, first]);
}

#[trace("FR-003-AC-7")]
#[test]
fn every_denied_charge_keeps_the_prefix_and_exposes_no_partial_result() {
    let input = source(CollectionKind::Sequence, None, &[false, true]);
    for drop in [true, false] {
        let run = |meter: &mut Meter| {
            if drop {
                drop_first(collection(&input), meter)
            } else {
                remove_one(
                    collection(&input),
                    &Value::Boolean(true),
                    &TypeEnvironment::default(),
                    meter,
                )
            }
        };
        let mut complete = support::meter(10_000);
        completed(run(&mut complete));
        let charges = complete.admitted_charges();
        for (index, charge) in charges.iter().enumerate() {
            let occurrence = charges[..=index]
                .iter()
                .filter(|c| c.point == charge.point)
                .count();
            let mut meter = support::meter(10_000).with_injected_denial(InjectedDenial {
                point: charge.point,
                occurrence: u64::try_from(occurrence).unwrap().try_into().unwrap(),
            });
            let Outcome::Incomplete(stopped) = run(&mut meter).unwrap() else {
                panic!("denied operation completed");
            };
            assert_eq!(stopped.charge_point, charge.point);
            assert_eq!(meter.admitted_charges(), &charges[..index]);
        }
    }
}
