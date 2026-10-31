// SPDX-License-Identifier: AGPL-3.0-or-later
//! The canonical operation vocabulary for checked expression nodes (FR-400).
//!
//! These are plain descriptors. Possessing one does not admit a checked term.

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use quire_exact::{
    ArithmeticOperator, CollectionType, DecimalType, EffectiveId, FieldValue, IntegerInterval,
    NodeKey, OrderingOperator, RationalDomain, RoundingMode, Value, ValueType,
};
use quire_semantic_value::absence::AbsenceMode;
use quire_semantic_value::declaration::{
    CheckedEquality, EqualityOperand, EqualityOperator, FieldRef,
};
use quire_semantic_value::location::Location;

use crate::integer_division::AdmittedIntegerDivision;
use crate::key::DeclarationKey;

/// A local slot of one function frame or expression.
pub type Slot = usize;

/// An arena index naming a node in its owning checked body.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(usize);

impl NodeId {
    /// The position of this node in its body.
    pub fn index(self) -> usize {
        self.0
    }
}

/// A record field identified by its declaring record and name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordField {
    declaration: NodeKey,
    name: String,
}

impl RecordField {
    /// Identify `name` in `declaration`.
    pub fn new(declaration: NodeKey, name: impl Into<String>) -> Self {
        Self {
            declaration,
            name: name.into(),
        }
    }

    /// The declaring record.
    pub fn declaration(&self) -> NodeKey {
        self.declaration.clone()
    }

    /// The field name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A connective with a skippable right operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Connective {
    And,
    Or,
    Implies,
}

/// Integer addition, subtraction or multiplication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
}

/// An integer division operator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerDivisionOperator {
    Div,
    Rem,
    Mod,
}

/// The value domain of an ordering comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderedKind {
    Integers,
    Rationals,
    Decimals,
    Enums,
    Texts,
    Quantities,
}

/// The near end of a relationship.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EndSide {
    Source,
    Target,
}

/// An authored finite maximum of a population.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AuthoredBound {
    /// Inclusive maximum object count.
    pub maximum: u64,
    /// Label of the bound declaration.
    pub label: String,
    /// Bounded population identity.
    pub population: DeclarationKey,
}

/// A literal admitted by checking. The value is borrowed by a node read.
#[derive(Clone, Debug)]
pub struct CheckedLiteral(pub Value);

impl PartialEq for CheckedLiteral {
    fn eq(&self, other: &Self) -> bool {
        let mut pending = alloc::vec![(&self.0, &other.0)];
        while let Some((left, right)) = pending.pop() {
            match (left, right) {
                (Value::Float(a), Value::Float(b))
                    if a.width() == b.width() && a.bits() == b.bits() => {}
                (Value::Population(a), Value::Population(b)) if a == b => {}
                (Value::Option(a), Value::Option(b)) if a.payload_type() == b.payload_type() => {
                    match (a.payload(), b.payload()) {
                        (None, None) => {}
                        (Some(a), Some(b)) => pending.push((a, b)),
                        _ => return false,
                    }
                }
                (Value::Composite(a), Value::Composite(b))
                    if a.declaration() == b.declaration() && a.slots().len() == b.slots().len() =>
                {
                    for (a, b) in a.slots().iter().zip(b.slots()) {
                        match (a, b) {
                            (FieldValue::Present(a), FieldValue::Present(b)) => {
                                pending.push((a, b))
                            }
                            (FieldValue::Absent, FieldValue::Absent)
                            | (FieldValue::Null, FieldValue::Null) => {}
                            _ => return false,
                        }
                    }
                }
                (Value::Collection(a), Value::Collection(b))
                    if a.collection_type() == b.collection_type()
                        && a.elements().len() == b.elements().len() =>
                {
                    pending.extend(a.elements().iter().zip(b.elements()));
                }
                (a, b) if quire_exact::compare_keys(a, b) == Some(Ordering::Equal) => {}
                _ => return false,
            }
        }
        true
    }
}

/// One field slot of a record construction.
#[derive(Clone, Debug, PartialEq)]
pub enum RecordSlot<Child = NodeId> {
    /// An omitted optional field.
    Absent,
    /// An optional field explicitly set to null.
    Null,
    /// A present field.
    Present(Child),
}

/// One checked union case arm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseArm<Child = NodeId> {
    /// Selected member's declaration-order position.
    pub member: usize,
    /// Payload binders in position order.
    pub binders: Vec<Slot>,
    /// The arm body.
    pub body: Child,
}

/// A one-binder collection query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visit {
    Map,
    Filter,
    Forall,
    Exists,
    Count,
    Sum,
    Only,
}

/// What one expression node computes; every operand is of type `Child`.
///
/// The default child is an arena index. A borrowed read substitutes an
/// owner-qualified handle and borrows the literal value.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind<Child = NodeId, Literal = CheckedLiteral> {
    /// A literal value.
    Literal(Literal),
    /// Narrow an integer to an admitted interval.
    Coerce(Child, IntegerInterval),
    /// Read a local slot.
    Local(Slot),
    /// Bind a value and evaluate a body.
    Let {
        slot: Slot,
        value: Child,
        body: Child,
    },
    /// Choose a branch by a Boolean condition.
    If {
        condition: Child,
        then: Child,
        otherwise: Child,
    },
    /// Integer arithmetic.
    Arithmetic(Arithmetic, Child, Child),
    /// Integer division with its admitted law, absent only for modulus.
    IntegerDivision {
        operator: IntegerDivisionOperator,
        selection: Option<AdmittedIntegerDivision>,
        left: Child,
        right: Child,
    },
    /// Unary integer negation.
    Negate(Child),
    /// Integer division producing a rational.
    Divide {
        left: Child,
        right: Child,
        domain: RationalDomain,
    },
    /// Rational arithmetic.
    Rational {
        operator: ArithmeticOperator,
        left: Child,
        right: Child,
        domain: RationalDomain,
    },
    /// Unary rational negation.
    RationalNegate(Child, RationalDomain),
    /// Decimal arithmetic.
    Decimal {
        operator: ArithmeticOperator,
        left: Child,
        right: Child,
        target: DecimalType,
    },
    /// Unary decimal negation.
    DecimalNegate(Child, DecimalType),
    /// IEEE arithmetic.
    Ieee(ArithmeticOperator, RoundingMode, Child, Child),
    /// IEEE square root.
    IeeeSquareRoot(RoundingMode, Child),
    /// Quantity arithmetic.
    Quantity(ArithmeticOperator, Child, Child),
    /// Rational or decimal conversion.
    ConvertDecimal(Child, DecimalType),
    /// Ordering comparison.
    Order(OrderingOperator, OrderedKind, Child, Child),
    /// Checked equality schedule.
    Equality(
        EqualityOperator,
        alloc::boxed::Box<CheckedEquality>,
        Child,
        Child,
    ),
    /// IEEE numeric equality.
    IeeeNumericEquality(Child, Child),
    /// Boolean connective.
    Connective(Connective, Child, Child),
    /// Boolean negation.
    Not(Child),
    /// Read a record field.
    Field {
        operand: Child,
        field: RecordField,
        optional: bool,
    },
    /// Read a model attribute; `derefed` preserves its explicit-dereference charge.
    Attribute {
        reference: Child,
        field: FieldRef,
        optional: bool,
        derefed: bool,
    },
    /// Navigate a relationship end.
    Navigate {
        reference: Child,
        relationship: DeclarationKey,
        role: String,
        near: EndSide,
        unique: bool,
    },
    /// Test optional presence.
    Present(Child),
    /// Test presence of a guarded clock place.
    PlacePresence {
        reference: Child,
        field: FieldRef,
        case: Option<String>,
    },
    /// Test presence of clock collection slots.
    SlotPresence {
        reference: Child,
        field: FieldRef,
        all: bool,
    },
    /// Extract an optional payload.
    Value(Child),
    /// Construct a present option.
    OptionSome(Child),
    /// Call a local function.
    Call {
        function: usize,
        arguments: Vec<Child>,
    },
    /// Call an admitted dependency function.
    ImportedCall {
        callee: NodeKey,
        function: usize,
        arguments: Vec<Child>,
    },
    /// Construct a tuple.
    Tuple {
        declaration: NodeKey,
        arguments: Vec<Child>,
    },
    /// Construct a record.
    Record {
        declaration: NodeKey,
        slots: Vec<RecordSlot<Child>>,
    },
    /// Construct a collection.
    Collection {
        collection_type: CollectionType,
        elements: Vec<Child>,
    },
    /// Convert a collection kind.
    ConvertCollection {
        target: CollectionType,
        operand: Child,
    },
    /// Convert an equality operand's scalar.
    ConvertScalar(EqualityOperand, Child),
    /// Convert an IEEE value to an exact rational.
    IeeeToRational(Child, RationalDomain),
    /// Run a one-binder query.
    Query {
        visit: Visit,
        slot: Slot,
        source: Child,
        body: Child,
    },
    /// Flatten a collection.
    Flatten(Child),
    /// Fold or reduce a collection.
    Fold {
        accumulator: Slot,
        binder: Slot,
        source: Child,
        step: Child,
        identity: Option<Child>,
    },
    /// Read collection size.
    Size(Child),
    /// Test collection membership.
    Contains(Child, Child),
    /// Read all instances of a population.
    AllInstances { population: Child },
    /// Look up a reference in a population.
    Lookup {
        population: Child,
        reference: Child,
        absence: AbsenceMode,
    },
    /// Dispatch an operation by receiver type.
    Dispatch {
        receiver: Child,
        table: usize,
        operation: usize,
        arguments: Vec<Child>,
    },
    /// Evaluate against the invocation pre population.
    Pre(Child),
    /// Test reachability through an object field.
    Reaches {
        source: Child,
        target: Child,
        edge: FieldRef,
    },
    /// Test reachability through a relationship end.
    ReachesEnd {
        source: Child,
        target: Child,
        relationship: DeclarationKey,
        role: String,
        near: EndSide,
        population: Option<AuthoredBound>,
    },
    /// Read a protocol declaration.
    ProtocolReference { protocol: usize },
    /// Select a union case arm.
    Case {
        scrutinee: Child,
        declaration: NodeKey,
        arms: Vec<CaseArm<Child>>,
    },
    /// Construct a union member.
    UnionValue {
        declaration: NodeKey,
        member: usize,
        arguments: Vec<Child>,
    },
    /// Read concrete population objects.
    PopulationObjects { population: DeclarationKey },
    /// Name a domain-package population.
    NamedPopulation {
        population: DeclarationKey,
        members: Vec<EffectiveId>,
    },
    /// Read a refinement history field.
    History {
        reference: Child,
        object_type: EffectiveId,
        field: String,
    },
    /// Build an integer sequence.
    Seq {
        slot: Slot,
        low: Child,
        high: Child,
        body: Child,
    },
    /// Probe-only variant that forces consumers to handle new operations.
    #[cfg(seam_probe)]
    __SeamProbe,
}

/// A borrowed node read from an admitted, owner-qualified source.
///
/// This descriptor alone carries no admission authority.
#[derive(Clone, Debug)]
pub struct NodeRead<'a, Child> {
    /// The canonical operation, with owner-qualified child handles.
    pub operation: NodeKind<Child, &'a Value>,
    /// The checked static type.
    pub value_type: &'a ValueType,
    /// The checked source location.
    pub location: &'a Location,
}

impl<Child, Literal> NodeKind<Child, Literal> {
    /// Borrow this operation into the same vocabulary with another child
    /// source and literal representation. Metadata and admitted evidence are
    /// retained; every child, including nested slots and arms, is mapped.
    pub fn map_ref<MappedChild, MappedLiteral>(
        &self,
        mut child: impl FnMut(&Child) -> MappedChild,
        literal: impl FnOnce(&Literal) -> MappedLiteral,
    ) -> NodeKind<MappedChild, MappedLiteral> {
        use NodeKind as K;
        match self {
            K::Literal(value) => K::Literal(literal(value)),
            K::Coerce(value, domain) => K::Coerce(child(value), domain.clone()),
            K::Local(slot) => K::Local(*slot),
            K::Let { slot, value, body } => K::Let {
                slot: *slot,
                value: child(value),
                body: child(body),
            },
            K::If {
                condition,
                then,
                otherwise,
            } => K::If {
                condition: child(condition),
                then: child(then),
                otherwise: child(otherwise),
            },
            K::Arithmetic(op, left, right) => K::Arithmetic(*op, child(left), child(right)),
            K::IntegerDivision {
                operator,
                selection,
                left,
                right,
            } => K::IntegerDivision {
                operator: *operator,
                selection: *selection,
                left: child(left),
                right: child(right),
            },
            K::Negate(value) => K::Negate(child(value)),
            K::Divide {
                left,
                right,
                domain,
            } => K::Divide {
                left: child(left),
                right: child(right),
                domain: domain.clone(),
            },
            K::Rational {
                operator,
                left,
                right,
                domain,
            } => K::Rational {
                operator: *operator,
                left: child(left),
                right: child(right),
                domain: domain.clone(),
            },
            K::RationalNegate(value, domain) => K::RationalNegate(child(value), domain.clone()),
            K::Decimal {
                operator,
                left,
                right,
                target,
            } => K::Decimal {
                operator: *operator,
                left: child(left),
                right: child(right),
                target: target.clone(),
            },
            K::DecimalNegate(value, target) => K::DecimalNegate(child(value), target.clone()),
            K::Ieee(op, rounding, left, right) => {
                K::Ieee(*op, *rounding, child(left), child(right))
            }
            K::IeeeSquareRoot(rounding, value) => K::IeeeSquareRoot(*rounding, child(value)),
            K::Quantity(op, left, right) => K::Quantity(*op, child(left), child(right)),
            K::ConvertDecimal(value, target) => K::ConvertDecimal(child(value), target.clone()),
            K::Order(op, kind, left, right) => K::Order(*op, *kind, child(left), child(right)),
            K::Equality(op, checked, left, right) => {
                K::Equality(*op, checked.clone(), child(left), child(right))
            }
            K::IeeeNumericEquality(left, right) => {
                K::IeeeNumericEquality(child(left), child(right))
            }
            K::Connective(op, left, right) => K::Connective(*op, child(left), child(right)),
            K::Not(value) => K::Not(child(value)),
            K::Field {
                operand,
                field,
                optional,
            } => K::Field {
                operand: child(operand),
                field: field.clone(),
                optional: *optional,
            },
            K::Attribute {
                reference,
                field,
                optional,
                derefed,
            } => K::Attribute {
                reference: child(reference),
                field: field.clone(),
                optional: *optional,
                derefed: *derefed,
            },
            K::Navigate {
                reference,
                relationship,
                role,
                near,
                unique,
            } => K::Navigate {
                reference: child(reference),
                relationship: relationship.clone(),
                role: role.clone(),
                near: *near,
                unique: *unique,
            },
            K::Present(value) => K::Present(child(value)),
            K::PlacePresence {
                reference,
                field,
                case,
            } => K::PlacePresence {
                reference: child(reference),
                field: field.clone(),
                case: case.clone(),
            },
            K::SlotPresence {
                reference,
                field,
                all,
            } => K::SlotPresence {
                reference: child(reference),
                field: field.clone(),
                all: *all,
            },
            K::Value(value) => K::Value(child(value)),
            K::OptionSome(value) => K::OptionSome(child(value)),
            K::Call {
                function,
                arguments,
            } => K::Call {
                function: *function,
                arguments: arguments.iter().map(&mut child).collect(),
            },
            K::ImportedCall {
                callee,
                function,
                arguments,
            } => K::ImportedCall {
                callee: callee.clone(),
                function: *function,
                arguments: arguments.iter().map(&mut child).collect(),
            },
            K::Tuple {
                declaration,
                arguments,
            } => K::Tuple {
                declaration: declaration.clone(),
                arguments: arguments.iter().map(&mut child).collect(),
            },
            K::Record { declaration, slots } => K::Record {
                declaration: declaration.clone(),
                slots: slots
                    .iter()
                    .map(|slot| match slot {
                        RecordSlot::Absent => RecordSlot::Absent,
                        RecordSlot::Null => RecordSlot::Null,
                        RecordSlot::Present(value) => RecordSlot::Present(child(value)),
                    })
                    .collect(),
            },
            K::Collection {
                collection_type,
                elements,
            } => K::Collection {
                collection_type: collection_type.clone(),
                elements: elements.iter().map(&mut child).collect(),
            },
            K::ConvertCollection { target, operand } => K::ConvertCollection {
                target: target.clone(),
                operand: child(operand),
            },
            K::ConvertScalar(target, value) => K::ConvertScalar(target.clone(), child(value)),
            K::IeeeToRational(value, domain) => K::IeeeToRational(child(value), domain.clone()),
            K::Query {
                visit,
                slot,
                source,
                body,
            } => K::Query {
                visit: *visit,
                slot: *slot,
                source: child(source),
                body: child(body),
            },
            K::Flatten(value) => K::Flatten(child(value)),
            K::Fold {
                accumulator,
                binder,
                source,
                step,
                identity,
            } => {
                let source = child(source);
                let identity = identity.as_ref().map(&mut child);
                let step = child(step);
                K::Fold {
                    accumulator: *accumulator,
                    binder: *binder,
                    source,
                    step,
                    identity,
                }
            }
            K::Size(value) => K::Size(child(value)),
            K::Contains(left, right) => K::Contains(child(left), child(right)),
            K::AllInstances { population } => K::AllInstances {
                population: child(population),
            },
            K::Lookup {
                population,
                reference,
                absence,
            } => K::Lookup {
                population: child(population),
                reference: child(reference),
                absence: *absence,
            },
            K::Dispatch {
                receiver,
                table,
                operation,
                arguments,
            } => K::Dispatch {
                receiver: child(receiver),
                table: *table,
                operation: *operation,
                arguments: arguments.iter().map(&mut child).collect(),
            },
            K::Pre(value) => K::Pre(child(value)),
            K::Reaches {
                source,
                target,
                edge,
            } => K::Reaches {
                source: child(source),
                target: child(target),
                edge: edge.clone(),
            },
            K::ReachesEnd {
                source,
                target,
                relationship,
                role,
                near,
                population,
            } => K::ReachesEnd {
                source: child(source),
                target: child(target),
                relationship: relationship.clone(),
                role: role.clone(),
                near: *near,
                population: population.clone(),
            },
            K::ProtocolReference { protocol } => K::ProtocolReference {
                protocol: *protocol,
            },
            K::Case {
                scrutinee,
                declaration,
                arms,
            } => K::Case {
                scrutinee: child(scrutinee),
                declaration: declaration.clone(),
                arms: arms
                    .iter()
                    .map(|arm| CaseArm {
                        member: arm.member,
                        binders: arm.binders.clone(),
                        body: child(&arm.body),
                    })
                    .collect(),
            },
            K::UnionValue {
                declaration,
                member,
                arguments,
            } => K::UnionValue {
                declaration: declaration.clone(),
                member: *member,
                arguments: arguments.iter().map(&mut child).collect(),
            },
            K::PopulationObjects { population } => K::PopulationObjects {
                population: population.clone(),
            },
            K::NamedPopulation {
                population,
                members,
            } => K::NamedPopulation {
                population: population.clone(),
                members: members.clone(),
            },
            K::History {
                reference,
                object_type,
                field,
            } => K::History {
                reference: child(reference),
                object_type: object_type.clone(),
                field: field.clone(),
            },
            K::Seq {
                slot,
                low,
                high,
                body,
            } => K::Seq {
                slot: *slot,
                low: child(low),
                high: child(high),
                body: child(body),
            },
            #[cfg(seam_probe)]
            K::__SeamProbe => K::__SeamProbe,
        }
    }
}
