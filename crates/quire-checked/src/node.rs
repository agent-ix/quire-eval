// SPDX-License-Identifier: AGPL-3.0-or-later
//! The canonical operation vocabulary for checked expression nodes (FR-400).
//!
//! These are plain descriptors. Possessing one does not admit a checked term.

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;

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
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(usize);

impl NodeId {
    /// The position of this node in its body.
    pub fn index(self) -> usize {
        self.0
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{}", self.0)
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
    /// Logical conjunction.
    And,
    /// Logical disjunction.
    Or,
    /// Logical implication.
    Implies,
}

/// Whether a connective node was written in the source or introduced by the
/// checker.
///
/// A consumer that records decisions only for connectives the author wrote
/// reads this instead of inferring origin from locations. The origin is part
/// of the node's equality: an authored and a synthesized connective with the
/// same operator and operands are different nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectiveOrigin {
    /// The connective appears in the authored source.
    Authored,
    /// The connective was synthesized by the checker, for example to compose
    /// a derived ordering.
    Synthesized,
}

/// Integer addition, subtraction or multiplication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arithmetic {
    /// Addition.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
}

/// An integer division operator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerDivisionOperator {
    /// Quotient under the admitted law.
    Div,
    /// Remainder under the admitted law.
    Rem,
    /// Euclidean remainder independent of that law.
    Mod,
}

/// The value domain of an ordering comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderedKind {
    /// Integer values.
    Integers,
    /// Rational values.
    Rationals,
    /// Decimal values.
    Decimals,
    /// IEEE values in the distinct IEEE total-order operation.
    Float,
    /// Ordered enum members.
    Enums,
    /// Profile-ordered text.
    Texts,
    /// Equal-unit quantities.
    Quantities,
}

/// The near end of a relationship.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EndSide {
    /// Source end.
    Source,
    /// Target end.
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
                (Value::Float(a), Value::Float(b)) => {
                    if a.width() != b.width() || a.bits() != b.bits() {
                        return false;
                    }
                }
                (Value::Population(a), Value::Population(b)) => {
                    if a != b {
                        return false;
                    }
                }
                (Value::Option(a), Value::Option(b)) => {
                    if a.payload_type() != b.payload_type() {
                        return false;
                    }
                    match (a.payload(), b.payload()) {
                        (None, None) => {}
                        (Some(a), Some(b)) => pending.push((a, b)),
                        _ => return false,
                    }
                }
                (Value::Composite(a), Value::Composite(b)) => {
                    if a.declaration() != b.declaration() || a.slots().len() != b.slots().len() {
                        return false;
                    }
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
                (Value::Collection(a), Value::Collection(b)) => {
                    if a.collection_type() != b.collection_type()
                        || a.elements().len() != b.elements().len()
                    {
                        return false;
                    }
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
    /// Map each occurrence.
    Map,
    /// Keep admitted occurrences.
    Filter,
    /// Test that every occurrence is admitted.
    Forall,
    /// Test that some occurrence is admitted.
    Exists,
    /// Count admitted occurrences.
    Count,
    /// Sum integer results.
    Sum,
    /// Read the sole occurrence.
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
        /// The local slot.
        slot: Slot,
        /// The bound value operand.
        value: Child,
        /// The body operand.
        body: Child,
    },
    /// Choose a branch by a Boolean condition.
    If {
        /// The Boolean condition operand.
        condition: Child,
        /// The true branch.
        then: Child,
        /// The false branch.
        otherwise: Child,
    },
    /// Integer arithmetic.
    Arithmetic(Arithmetic, Child, Child),
    /// Integer division with its admitted law, absent only for modulus.
    IntegerDivision {
        /// The selected operator.
        operator: IntegerDivisionOperator,
        /// The admitted division law, when required.
        selection: Option<AdmittedIntegerDivision>,
        /// The left operand.
        left: Child,
        /// The right operand.
        right: Child,
    },
    /// Unary integer negation.
    Negate(Child),
    /// Integer division producing a rational.
    Divide {
        /// The left operand.
        left: Child,
        /// The right operand.
        right: Child,
        /// The result domain.
        domain: RationalDomain,
    },
    /// Rational arithmetic.
    Rational {
        /// The selected operator.
        operator: ArithmeticOperator,
        /// The left operand.
        left: Child,
        /// The right operand.
        right: Child,
        /// The result domain.
        domain: RationalDomain,
    },
    /// Unary rational negation.
    RationalNegate(Child, RationalDomain),
    /// Decimal arithmetic.
    Decimal {
        /// The selected operator.
        operator: ArithmeticOperator,
        /// The left operand.
        left: Child,
        /// The right operand.
        right: Child,
        /// The target type or operand.
        target: DecimalType,
    },
    /// Unary decimal negation.
    DecimalNegate(Child, DecimalType),
    /// IEEE arithmetic.
    Ieee(ArithmeticOperator, RoundingMode, Child, Child),
    /// IEEE square root.
    IeeeSquareRoot(RoundingMode, Child),
    /// `quire.op.ieee.float32.fma` with three ordered operands.
    IeeeFloat32Fma(RoundingMode, Child, Child, Child),
    /// `quire.op.ieee.float64.fma` with three ordered operands.
    IeeeFloat64Fma(RoundingMode, Child, Child, Child),
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
    /// `quire.op.ieee.total_order`, whose admitted domain is `OrderedKind::Float`.
    IeeeTotalOrder(OrderedKind, Child, Child),
    /// `quire.op.ieee.bit_identical` comparison.
    IeeeBitIdentical(Child, Child),
    /// Boolean connective with the origin of the node, its left operand and
    /// its skippable right operand.
    Connective(Connective, ConnectiveOrigin, Child, Child),
    /// Boolean negation.
    Not(Child),
    /// Read a record field.
    Field {
        /// The operand.
        operand: Child,
        /// The field identity or name.
        field: RecordField,
        /// Whether the field is optional.
        optional: bool,
    },
    /// Read a model attribute; `derefed` preserves its explicit-dereference charge.
    Attribute {
        /// The reference operand.
        reference: Child,
        /// The field identity or name.
        field: FieldRef,
        /// Whether the field is optional.
        optional: bool,
        /// Whether an explicit dereference is charged.
        derefed: bool,
    },
    /// Navigate a relationship end.
    Navigate {
        /// The reference operand.
        reference: Child,
        /// The relationship identity.
        relationship: DeclarationKey,
        /// The named relationship role.
        role: String,
        /// The end nearest the source reference.
        near: EndSide,
        /// Whether the destination end is unique.
        unique: bool,
    },
    /// Test optional presence.
    Present(Child),
    /// Test presence of a guarded clock place.
    PlacePresence {
        /// The reference operand.
        reference: Child,
        /// The field identity or name.
        field: FieldRef,
        /// The selected union case, if any.
        case: Option<String>,
    },
    /// Test presence of clock collection slots.
    SlotPresence {
        /// The reference operand.
        reference: Child,
        /// The field identity or name.
        field: FieldRef,
        /// Whether every element must be present.
        all: bool,
    },
    /// Extract an optional payload.
    Value(Child),
    /// Construct a present option.
    OptionSome(Child),
    /// Call a local function.
    Call {
        /// The checked function index.
        function: usize,
        /// The argument operands in order.
        arguments: Vec<Child>,
    },
    /// Call an admitted dependency function.
    ImportedCall {
        /// The admitted dependency callee identity.
        callee: NodeKey,
        /// The checked function index.
        function: usize,
        /// The argument operands in order.
        arguments: Vec<Child>,
    },
    /// Construct a tuple.
    Tuple {
        /// The declared type identity.
        declaration: NodeKey,
        /// The argument operands in order.
        arguments: Vec<Child>,
    },
    /// Construct a record.
    Record {
        /// The declared type identity.
        declaration: NodeKey,
        /// The record fields in declaration order.
        slots: Vec<RecordSlot<Child>>,
    },
    /// Construct a collection.
    Collection {
        /// The collection type.
        collection_type: CollectionType,
        /// The element operands in source order.
        elements: Vec<Child>,
    },
    /// Convert a collection kind.
    ConvertCollection {
        /// The target type or operand.
        target: CollectionType,
        /// The operand.
        operand: Child,
    },
    /// Convert an equality operand's scalar.
    ConvertScalar(EqualityOperand, Child),
    /// Convert an IEEE value to an exact rational.
    IeeeToRational(Child, RationalDomain),
    /// Run a one-binder query.
    Query {
        /// The query operation.
        visit: Visit,
        /// The local slot.
        slot: Slot,
        /// The source operand.
        source: Child,
        /// The body operand.
        body: Child,
    },
    /// Flatten a collection.
    Flatten(Child),
    /// Fold or reduce a collection.
    Fold {
        /// The accumulator slot.
        accumulator: Slot,
        /// The occurrence binder slot.
        binder: Slot,
        /// The source operand.
        source: Child,
        /// The fold step operand.
        step: Child,
        /// The fold identity, absent for reduce.
        identity: Option<Child>,
    },
    /// Read collection size.
    Size(Child),
    /// Test collection membership.
    Contains(Child, Child),
    /// Read all instances of a population.
    AllInstances {
        /// The population operand, bound, or identity.
        population: Child,
    },
    /// Look up a reference in a population.
    Lookup {
        /// The population operand, bound, or identity.
        population: Child,
        /// The reference operand.
        reference: Child,
        /// The authored absence mode.
        absence: AbsenceMode,
    },
    /// Dispatch an operation by receiver type.
    Dispatch {
        /// The receiver operand.
        receiver: Child,
        /// The dispatch table index.
        table: usize,
        /// The dispatch operation index.
        operation: usize,
        /// The argument operands in order.
        arguments: Vec<Child>,
    },
    /// Evaluate against the invocation pre population.
    Pre(Child),
    /// Test reachability through an object field.
    Reaches {
        /// The source operand.
        source: Child,
        /// The target type or operand.
        target: Child,
        /// The traversed edge field.
        edge: FieldRef,
    },
    /// Test reachability through a relationship end.
    ReachesEnd {
        /// The source operand.
        source: Child,
        /// The target type or operand.
        target: Child,
        /// The relationship identity.
        relationship: DeclarationKey,
        /// The named relationship role.
        role: String,
        /// The end nearest the source reference.
        near: EndSide,
        /// The population operand, bound, or identity.
        population: Option<AuthoredBound>,
    },
    /// Read a protocol declaration.
    ProtocolReference {
        /// The protocol declaration index.
        protocol: usize,
    },
    /// Select a union case arm.
    Case {
        /// The union value being selected.
        scrutinee: Child,
        /// The declared type identity.
        declaration: NodeKey,
        /// The case arms in member order.
        arms: Vec<CaseArm<Child>>,
    },
    /// Construct a union member.
    UnionValue {
        /// The declared type identity.
        declaration: NodeKey,
        /// The selected union member index.
        member: usize,
        /// The argument operands in order.
        arguments: Vec<Child>,
    },
    /// Read concrete population objects.
    PopulationObjects {
        /// The population operand, bound, or identity.
        population: DeclarationKey,
    },
    /// Name a domain-package population.
    NamedPopulation {
        /// The population operand, bound, or identity.
        population: DeclarationKey,
        /// The declared member types.
        members: Vec<EffectiveId>,
    },
    /// Read a refinement history field.
    History {
        /// The reference operand.
        reference: Child,
        /// The concrete object type.
        object_type: EffectiveId,
        /// The field identity or name.
        field: String,
    },
    /// Build an integer sequence.
    Seq {
        /// The local slot.
        slot: Slot,
        /// The first bound operand.
        low: Child,
        /// The last bound operand.
        high: Child,
        /// The body operand.
        body: Child,
    },
    /// Probe-only variant that forces consumers to handle new operations.
    #[cfg(seam_probe)]
    __SeamProbe,
}

/// A borrowed node read from an admitted, owner-qualified source.
///
/// This descriptor alone carries no admission authority.
/// Constructing one from plain values is allowed:
///
/// ```no_run
/// use quire_checked::{NodeKind, NodeRead};
/// use quire_exact::{Value, ValueType};
/// use quire_semantic_value::location::Location;
/// fn describe<'a>(value: &'a Value, value_type: &'a ValueType, location: &'a Location)
///     -> NodeRead<'a, usize>
/// {
///     NodeRead { operation: NodeKind::Literal(value), value_type, location }
/// }
/// ```
///
/// FR-400-AC-12: the descriptor cannot be used as a checked term:
///
/// ```compile_fail,E0308
/// use quire_checked::{NodeKind, NodeRead};
/// use quire_checked::integer_division::AdmittedIntegerDivision;
/// use quire_exact::{Value, ValueType};
/// use quire_semantic_value::location::Location;
/// fn admitted_only(_: &AdmittedIntegerDivision) {}
/// fn attempt(value: &Value, value_type: &ValueType, location: &Location) {
///     let read: NodeRead<'_, usize> = NodeRead {
///         operation: NodeKind::Literal(value), value_type, location,
///     };
///     admitted_only(&read);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct NodeRead<'a, Child> {
    /// The canonical operation, with owner-qualified child handles.
    pub operation: NodeKind<Child, &'a Value>,
    /// The checked static type.
    pub value_type: &'a ValueType,
    /// The checked source location.
    pub location: &'a Location,
}

impl<'a, Child> NodeRead<'a, Child> {
    /// Project a stored operation and its checked metadata into a borrowed
    /// descriptor. `child` supplies handles from the same owning view; the
    /// caller remains responsible for that ownership guarantee. The result
    /// is plain data and grants no evaluation authority.
    pub fn from_stored(
        operation: &'a NodeKind,
        value_type: &'a ValueType,
        location: &'a Location,
        child: impl FnMut(&'a NodeId) -> Child,
    ) -> Self {
        Self {
            operation: operation.map_ref(child, |literal| &literal.0),
            value_type,
            location,
        }
    }
}

impl<Child, Literal> NodeKind<Child, Literal> {
    /// Borrow this operation into the same vocabulary with another child
    /// source and literal representation. Metadata and admitted evidence are
    /// retained; every child, including nested slots and arms, is mapped.
    pub fn map_ref<'a, MappedChild, MappedLiteral>(
        &'a self,
        mut child: impl FnMut(&'a Child) -> MappedChild,
        literal: impl FnOnce(&'a Literal) -> MappedLiteral,
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
            K::IeeeFloat32Fma(rounding, first, second, third) => {
                K::IeeeFloat32Fma(*rounding, child(first), child(second), child(third))
            }
            K::IeeeFloat64Fma(rounding, first, second, third) => {
                K::IeeeFloat64Fma(*rounding, child(first), child(second), child(third))
            }
            K::Quantity(op, left, right) => K::Quantity(*op, child(left), child(right)),
            K::ConvertDecimal(value, target) => K::ConvertDecimal(child(value), target.clone()),
            K::Order(op, kind, left, right) => K::Order(*op, *kind, child(left), child(right)),
            K::Equality(op, checked, left, right) => {
                K::Equality(*op, checked.clone(), child(left), child(right))
            }
            K::IeeeNumericEquality(left, right) => {
                K::IeeeNumericEquality(child(left), child(right))
            }
            K::IeeeTotalOrder(kind, left, right) => {
                K::IeeeTotalOrder(*kind, child(left), child(right))
            }
            K::IeeeBitIdentical(left, right) => K::IeeeBitIdentical(child(left), child(right)),
            K::Connective(op, origin, left, right) => {
                K::Connective(*op, *origin, child(left), child(right))
            }
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

#[cfg(test)]
mod tests {
    use super::{Connective, ConnectiveOrigin, NodeId, NodeKind, NodeRead, OrderedKind};
    use quire_exact::{FloatType, IeeeWidth, RoundingMode, ValueType};
    use quire_semantic_value::location::{Location, Origin};

    /// Trace: FR-001-AC-1, FR-001-AC-6
    #[test]
    fn float32_fma_read_retains_rounding_and_three_operand_order() {
        let operation = NodeKind::IeeeFloat32Fma(
            RoundingMode::TowardPositive,
            NodeId(1),
            NodeId(2),
            NodeId(3),
        );
        let value_type = ValueType::Float(FloatType::new(
            IeeeWidth::Binary32,
            RoundingMode::TowardPositive,
        ));
        let location = Location::root(Origin::Expression);
        let read = NodeRead::from_stored(&operation, &value_type, &location, |id| id.index() + 10);
        let NodeKind::IeeeFloat32Fma(rounding, first, second, third) = read.operation else {
            panic!("float32 fma changed operation identity");
        };
        assert_eq!(rounding, RoundingMode::TowardPositive);
        assert_eq!((first, second, third), (11, 12, 13));
    }

    /// Trace: FR-001-AC-2, FR-001-AC-6
    #[test]
    fn float64_fma_read_has_distinct_identity_and_operand_order() {
        let operation = NodeKind::IeeeFloat64Fma(
            RoundingMode::TowardNegative,
            NodeId(4),
            NodeId(5),
            NodeId(6),
        );
        let value_type = ValueType::Float(FloatType::new(
            IeeeWidth::Binary64,
            RoundingMode::TowardNegative,
        ));
        let location = Location::root(Origin::Expression);
        let read = NodeRead::from_stored(&operation, &value_type, &location, |id| id.index() + 10);
        let NodeKind::IeeeFloat64Fma(rounding, first, second, third) = read.operation else {
            panic!("float64 fma changed operation identity");
        };
        assert_eq!(rounding, RoundingMode::TowardNegative);
        assert_eq!((first, second, third), (14, 15, 16));
    }

    /// Trace: FR-001-AC-3, FR-001-AC-5, FR-001-AC-6
    #[test]
    fn total_order_read_retains_float_domain_without_ordinary_order() {
        let operation = NodeKind::IeeeTotalOrder(OrderedKind::Float, NodeId(7), NodeId(8));
        let value_type = ValueType::Boolean;
        let location = Location::root(Origin::Expression);
        let read = NodeRead::from_stored(&operation, &value_type, &location, |id| id.index() + 10);
        let NodeKind::IeeeTotalOrder(kind, left, right) = read.operation else {
            panic!("IEEE total order changed operation identity");
        };
        assert_eq!(kind, OrderedKind::Float);
        assert_eq!((left, right), (17, 18));
    }

    /// Trace: FR-001-AC-4, FR-001-AC-6
    #[test]
    fn bit_identity_read_retains_distinct_identity_and_operand_order() {
        let operation = NodeKind::IeeeBitIdentical(NodeId(9), NodeId(10));
        let value_type = ValueType::Boolean;
        let location = Location::root(Origin::Expression);
        let read = NodeRead::from_stored(&operation, &value_type, &location, |id| id.index() + 10);
        let NodeKind::IeeeBitIdentical(left, right) = read.operation else {
            panic!("IEEE bit identity changed operation identity");
        };
        assert_eq!((left, right), (19, 20));
    }

    /// Trace: FR-002-AC-1, FR-002-AC-2
    #[test]
    fn connective_read_retains_operator_and_operand_order() {
        let value_type = ValueType::Boolean;
        let location = Location::root(Origin::Expression);
        for (op, left, right) in [
            (Connective::And, 1, 2),
            (Connective::Or, 3, 4),
            (Connective::Implies, 5, 6),
        ] {
            let operation =
                NodeKind::Connective(op, ConnectiveOrigin::Authored, NodeId(left), NodeId(right));
            let read =
                NodeRead::from_stored(&operation, &value_type, &location, |id| id.index() + 10);
            let NodeKind::Connective(read_op, _, read_left, read_right) = read.operation else {
                panic!("connective changed operation identity");
            };
            assert_eq!(read_op, op);
            assert_eq!((read_left, read_right), (left + 10, right + 10));
        }
    }

    fn read_origin(origin: ConnectiveOrigin, location: &Location) -> ConnectiveOrigin {
        let operation = NodeKind::Connective(Connective::Or, origin, NodeId(1), NodeId(2));
        let value_type = ValueType::Boolean;
        let read = NodeRead::from_stored(&operation, &value_type, location, |id| id.index() + 10);
        let NodeKind::Connective(_, read_origin, _, _) = read.operation else {
            panic!("connective changed operation identity");
        };
        read_origin
    }

    /// Trace: FR-002-AC-3
    #[test]
    fn authored_connective_reads_authored() {
        let location = Location::root(Origin::Expression);
        assert_eq!(
            read_origin(ConnectiveOrigin::Authored, &location),
            ConnectiveOrigin::Authored
        );
    }

    /// Trace: FR-002-AC-4
    #[test]
    fn synthesized_connective_reads_synthesized() {
        let location = Location::root(Origin::Expression);
        assert_eq!(
            read_origin(ConnectiveOrigin::Synthesized, &location),
            ConnectiveOrigin::Synthesized
        );
    }

    /// Trace: FR-002-AC-5
    #[test]
    fn connective_origin_does_not_depend_on_read_location() {
        let root = Location::root(Origin::Expression);
        let child = root.child(0);
        let other = Location::at(Origin::Expression, &[3, 1]);
        for origin in [ConnectiveOrigin::Authored, ConnectiveOrigin::Synthesized] {
            for location in [&root, &child, &other] {
                assert_eq!(read_origin(origin, location), origin);
            }
        }
    }

    /// Trace: FR-002-AC-6
    #[test]
    fn authored_and_synthesized_connectives_are_distinct_nodes() {
        let authored: NodeKind = NodeKind::Connective(
            Connective::And,
            ConnectiveOrigin::Authored,
            NodeId(1),
            NodeId(2),
        );
        let synthesized: NodeKind = NodeKind::Connective(
            Connective::And,
            ConnectiveOrigin::Synthesized,
            NodeId(1),
            NodeId(2),
        );
        assert_ne!(authored, synthesized);
        assert_eq!(authored.clone(), authored);
    }
}
