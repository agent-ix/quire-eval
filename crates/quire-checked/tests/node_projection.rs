// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exhaustive FR-400-AC-11 projection of the canonical operation vocabulary.

use quire_checked::key::DeclarationKey;
use quire_checked::node::{
    Arithmetic, AuthoredBound, CaseArm, CheckedLiteral, Connective, ConnectiveOrigin, EndSide,
    IntegerDivisionOperator, OrderedKind, RecordField, RecordSlot, Visit,
};
use quire_checked::{NodeKind, NodeRead};
use quire_exact::{
    ArithmeticOperator, CollectionKind, CollectionType, DecimalType, EffectiveId, Integer,
    IntegerInterval, NodeKey, OrderingOperator, OriginalDeclarationKey, PathSegment,
    RationalDomain, RoundingMode, Value, ValueType,
};
use quire_semantic_value::absence::AbsenceMode;
use quire_semantic_value::declaration::{
    EqualityOperand, EqualityOperator, FieldRef, TypeEnvironment,
};
use quire_semantic_value::enumeration::EnumMemberIndex;

/// Trace: FR-400-AC-11
#[test]
fn every_operation_projects_all_children_and_retains_metadata() {
    let key = NodeKey::new("fixture", vec![PathSegment::Text("node".into())], 0).unwrap();
    let declaration = DeclarationKey {
        package: "fixture".into(),
        node: "object".into(),
    };
    let effective = EffectiveId::new(None, OriginalDeclarationKey::new("fixture", "object"));
    let field = FieldRef::new(effective.clone(), "total");
    let record_field = RecordField::new(key.clone(), "total");
    let interval = IntegerInterval::spanning(Integer::from(-2_i64), Integer::from(3_i64));
    let positive = IntegerInterval::spanning(Integer::from(1_i64), Integer::from(3_i64));
    let rational = RationalDomain::new(interval.clone(), positive).unwrap();
    let decimal = DecimalType::new(
        Integer::from(-2_i64),
        Integer::from(3_i64),
        0,
        2,
        RoundingMode::Exact,
    )
    .unwrap();
    let collection = CollectionType::new(CollectionKind::Sequence, ValueType::Boolean, None);
    let operand = EqualityOperand::typed(ValueType::Boolean);
    let equality = TypeEnvironment::default()
        .check_equality(
            EqualityOperator::Equal,
            operand.clone(),
            operand.clone(),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    let bound = AuthoredBound {
        maximum: 3,
        label: "three".into(),
        population: declaration.clone(),
    };

    // Each entry is a distinct variant. Operand ids are unique within each
    // operation so the observed sequence detects swaps, omissions and copies.
    let cases: Vec<(NodeKind<usize>, Vec<usize>)> = vec![
        (
            NodeKind::Literal(CheckedLiteral(Value::Boolean(true))),
            vec![],
        ),
        (NodeKind::Coerce(1, interval.clone()), vec![1]),
        (NodeKind::Local(7), vec![]),
        (
            NodeKind::Let {
                slot: 7,
                value: 1,
                body: 2,
            },
            vec![1, 2],
        ),
        (
            NodeKind::If {
                condition: 1,
                then: 2,
                otherwise: 3,
            },
            vec![1, 2, 3],
        ),
        (NodeKind::Arithmetic(Arithmetic::Add, 1, 2), vec![1, 2]),
        (
            NodeKind::IntegerDivision {
                operator: IntegerDivisionOperator::Mod,
                selection: None,
                left: 1,
                right: 2,
            },
            vec![1, 2],
        ),
        (NodeKind::Negate(1), vec![1]),
        (
            NodeKind::Divide {
                left: 1,
                right: 2,
                domain: rational.clone(),
            },
            vec![1, 2],
        ),
        (
            NodeKind::Rational {
                operator: ArithmeticOperator::Add,
                left: 1,
                right: 2,
                domain: rational.clone(),
            },
            vec![1, 2],
        ),
        (NodeKind::RationalNegate(1, rational.clone()), vec![1]),
        (
            NodeKind::Decimal {
                operator: ArithmeticOperator::Add,
                left: 1,
                right: 2,
                target: decimal.clone(),
            },
            vec![1, 2],
        ),
        (NodeKind::DecimalNegate(1, decimal.clone()), vec![1]),
        (
            NodeKind::Ieee(ArithmeticOperator::Add, RoundingMode::Exact, 1, 2),
            vec![1, 2],
        ),
        (NodeKind::IeeeSquareRoot(RoundingMode::Exact, 1), vec![1]),
        (
            NodeKind::IeeeFloat32Fma(RoundingMode::TowardZero, 1, 2, 3),
            vec![1, 2, 3],
        ),
        (
            NodeKind::IeeeFloat64Fma(RoundingMode::NearestEven, 1, 2, 3),
            vec![1, 2, 3],
        ),
        (
            NodeKind::Quantity(ArithmeticOperator::Add, 1, 2),
            vec![1, 2],
        ),
        (NodeKind::ConvertDecimal(1, decimal.clone()), vec![1]),
        (
            NodeKind::Order(OrderingOperator::Less, OrderedKind::Integers, 1, 2),
            vec![1, 2],
        ),
        (
            NodeKind::Equality(EqualityOperator::Equal, Box::new(equality), 1, 2),
            vec![1, 2],
        ),
        (NodeKind::IeeeNumericEquality(1, 2), vec![1, 2]),
        (
            NodeKind::IeeeTotalOrder(OrderedKind::Float, 1, 2),
            vec![1, 2],
        ),
        (NodeKind::IeeeBitIdentical(1, 2), vec![1, 2]),
        (
            NodeKind::Connective(Connective::And, ConnectiveOrigin::Authored, 1, 2),
            vec![1, 2],
        ),
        (
            NodeKind::Connective(Connective::Implies, ConnectiveOrigin::Synthesized, 1, 2),
            vec![1, 2],
        ),
        (NodeKind::Not(1), vec![1]),
        (
            NodeKind::Field {
                operand: 1,
                field: record_field,
                optional: true,
            },
            vec![1],
        ),
        (
            NodeKind::Attribute {
                reference: 1,
                field: field.clone(),
                optional: true,
                derefed: true,
            },
            vec![1],
        ),
        (
            NodeKind::Navigate {
                reference: 1,
                relationship: declaration.clone(),
                role: "source".into(),
                near: EndSide::Source,
                unique: true,
            },
            vec![1],
        ),
        (NodeKind::Present(1), vec![1]),
        (
            NodeKind::PlacePresence {
                reference: 1,
                field: field.clone(),
                case: Some("clock".into()),
            },
            vec![1],
        ),
        (
            NodeKind::SlotPresence {
                reference: 1,
                field: field.clone(),
                all: true,
            },
            vec![1],
        ),
        (NodeKind::Value(1), vec![1]),
        (NodeKind::OptionSome(1), vec![1]),
        (
            NodeKind::Call {
                function: 9,
                arguments: vec![1, 2, 3],
            },
            vec![1, 2, 3],
        ),
        (
            NodeKind::ImportedCall {
                callee: key.clone(),
                function: 9,
                arguments: vec![1, 2],
            },
            vec![1, 2],
        ),
        (
            NodeKind::Tuple {
                declaration: key.clone(),
                arguments: vec![1, 2],
            },
            vec![1, 2],
        ),
        (
            NodeKind::Record {
                declaration: key.clone(),
                slots: vec![
                    RecordSlot::Absent,
                    RecordSlot::Present(1),
                    RecordSlot::Null,
                    RecordSlot::Present(2),
                ],
            },
            vec![1, 2],
        ),
        (
            NodeKind::Collection {
                collection_type: collection.clone(),
                elements: vec![1, 2],
            },
            vec![1, 2],
        ),
        (
            NodeKind::ConvertCollection {
                target: collection,
                operand: 1,
            },
            vec![1],
        ),
        (NodeKind::ConvertScalar(operand, 1), vec![1]),
        (NodeKind::IeeeToRational(1, rational), vec![1]),
        (
            NodeKind::Query {
                visit: Visit::Map,
                slot: 7,
                source: 1,
                body: 2,
            },
            vec![1, 2],
        ),
        (NodeKind::Flatten(1), vec![1]),
        (
            NodeKind::Fold {
                accumulator: 7,
                binder: 8,
                source: 1,
                step: 3,
                identity: Some(2),
            },
            vec![1, 2, 3],
        ),
        (NodeKind::Size(1), vec![1]),
        (NodeKind::Contains(1, 2), vec![1, 2]),
        (NodeKind::AllInstances { population: 1 }, vec![1]),
        (
            NodeKind::Lookup {
                population: 1,
                reference: 2,
                absence: AbsenceMode::Refused,
            },
            vec![1, 2],
        ),
        (
            NodeKind::Dispatch {
                receiver: 1,
                table: 9,
                operation: 10,
                arguments: vec![2, 3],
            },
            vec![1, 2, 3],
        ),
        (NodeKind::Pre(1), vec![1]),
        (
            NodeKind::Reaches {
                source: 1,
                target: 2,
                edge: field,
            },
            vec![1, 2],
        ),
        (
            NodeKind::ReachesEnd {
                source: 1,
                target: 2,
                relationship: declaration.clone(),
                role: "source".into(),
                near: EndSide::Source,
                population: Some(bound),
            },
            vec![1, 2],
        ),
        (NodeKind::ProtocolReference { protocol: 4 }, vec![]),
        (
            NodeKind::Case {
                scrutinee: 1,
                declaration: key.clone(),
                arms: vec![
                    CaseArm {
                        member: 0,
                        binders: vec![7],
                        body: 2,
                    },
                    CaseArm {
                        member: 1,
                        binders: vec![],
                        body: 3,
                    },
                ],
            },
            vec![1, 2, 3],
        ),
        (
            NodeKind::UnionValue {
                declaration: key,
                member: 2,
                arguments: vec![1, 2],
            },
            vec![1, 2],
        ),
        (
            NodeKind::PopulationObjects {
                population: declaration.clone(),
            },
            vec![],
        ),
        (
            NodeKind::NamedPopulation {
                population: declaration,
                members: vec![effective.clone()],
            },
            vec![],
        ),
        (
            NodeKind::History {
                reference: 1,
                object_type: effective,
                field: "past".into(),
            },
            vec![1],
        ),
        (
            NodeKind::Seq {
                slot: 7,
                low: 1,
                high: 2,
                body: 3,
            },
            vec![1, 2, 3],
        ),
    ];

    assert_eq!(cases.len(), 61, "a canonical variant was omitted");
    for (stored, expected_children) in cases {
        let mut seen = Vec::new();
        let mut literal_reads = 0;
        let borrowed = stored.map_ref(
            |child| {
                seen.push(*child);
                *child + 100
            },
            |literal| {
                literal_reads += 1;
                literal.clone()
            },
        );
        assert_eq!(
            seen, expected_children,
            "operand sequence changed for {stored:?}"
        );
        assert_eq!(
            literal_reads,
            usize::from(matches!(stored, NodeKind::Literal(_)))
        );
        let restored = borrowed.map_ref(|child| child - 100, |literal| literal.clone());
        assert_eq!(
            restored, stored,
            "metadata or operand changed for {stored:?}"
        );
    }
}

/// Trace: FR-400-AC-11
#[test]
fn root_read_path_is_the_plain_descriptor_type() {
    fn accepts_read(_: &NodeRead<'_, usize>) {}
    let value = Value::Boolean(true);
    let value_type = ValueType::Boolean;
    let location = quire_semantic_value::location::Location::root(
        quire_semantic_value::location::Origin::Expression,
    );
    let read = NodeRead {
        operation: NodeKind::Literal(&value),
        value_type: &value_type,
        location: &location,
    };
    accepts_read(&read);
}

/// Trace: FR-400-AC-11
#[test]
fn imported_call_retains_the_structural_package_in_its_callee() {
    let path = vec![PathSegment::Text("function".into())];
    let first = NodeKey::new("first-package", path.clone(), 0).unwrap();
    let second = NodeKey::new("second-package", path, 0).unwrap();
    assert_ne!(first, second);

    let stored = NodeKind::<usize, ()>::ImportedCall {
        callee: first.clone(),
        function: 7,
        arguments: vec![1, 2],
    };
    let read = stored.map_ref(|child| *child + 10, |_| unreachable!());
    let NodeKind::ImportedCall {
        callee,
        function,
        arguments,
    } = read
    else {
        panic!("imported call changed operation");
    };
    assert_eq!(callee, first);
    assert_ne!(callee, second);
    assert_eq!(function, 7);
    assert_eq!(arguments, [11, 12]);
}
