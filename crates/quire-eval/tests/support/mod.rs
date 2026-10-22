// SPDX-License-Identifier: AGPL-3.0-or-later
//! A toy evaluator behind the `CheckedClosure` contract, and recording host
//! bodies, for exercising the leaf through its public API alone.
//!
//! The toy language has interpreted callables over non-negative integers:
//!
//! - `COUNT(n)`: `0` when `n = 0`, otherwise `COUNT(n - 1) + 1`;
//! - `BOOM(n)`: always undefined, located at its call site;
//! - `REFUSE(n)`: always refused, located at its call site;
//! - `LOSSY(n)`: completes with `1` and one loss record;
//! - `ADD(a, b)`: `a + b`;
//!
//! and `HOST_A` and `HOST_B`, admitted callables with no interpreted body, so
//! they can only run as registered host bodies. These tests exercise the
//! scheduler through the contract; they are not the std reference adapter's
//! tests.
#![allow(dead_code)] // Each integration test uses a different part of this module.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use quire_eval::{
    Body, BodyFactory, CallFailure, CheckedClosure, ChildRequest, Evaluation, FamilyOutcome,
    RequestRefusal, Resume, Signature, Transition,
};
use quire_exact::{
    Charge, ChargePoint, IeeeFlag, IeeeWidth, Integer, LimitKind, Meter, NodeKey, Outcome, Refusal,
    ScalarLimits, Undefined, Value, ValueType,
};
use quire_semantic_value::call::InputRefusal;
use quire_semantic_value::catalog::InternalFault;
use quire_semantic_value::declaration::{EnvironmentFailure, StorageRequest, StorageUnit};
use quire_semantic_value::location::{Location, Origin};
use quire_semantic_value::loss::{LocatedLoss, ValueLoss};

/// A fixture node key: the toy closure's own identities.
pub fn key(byte: u8) -> NodeKey {
    NodeKey::decode_admitted([byte; 32])
}

pub fn count() -> NodeKey {
    key(1)
}
pub fn boom() -> NodeKey {
    key(2)
}
pub fn host_a() -> NodeKey {
    key(3)
}
pub fn host_b() -> NodeKey {
    key(4)
}
pub fn add() -> NodeKey {
    key(5)
}
pub fn refuse() -> NodeKey {
    key(6)
}
pub fn lossy() -> NodeKey {
    key(7)
}
/// Analogue of a ranged parameter: admits arguments up to 9.
pub fn ranged() -> NodeKey {
    key(8)
}
/// Analogue of a storage-bound parameter: its membership never decides.
pub fn storage() -> NodeKey {
    key(9)
}
/// Analogue of a `Population` parameter of declared maximum 3: it admits only
/// 3; 4 is the wrong maximum and 99 an id the environment does not resolve.
pub fn population() -> NodeKey {
    key(10)
}
/// A key no function of the toy closure has.
pub fn absent() -> NodeKey {
    key(99)
}

pub fn signature_of_arity(arity: usize) -> Signature {
    Signature::new(vec![ValueType::Integer; arity], ValueType::Integer)
}

pub fn int_signature() -> Signature {
    signature_of_arity(1)
}

pub fn int(n: u64) -> Value {
    Value::Integer(Integer::from(n))
}

/// The call site of a request that does not depend on depth.
pub fn site() -> Location {
    Location::root(Origin::Expression)
}

/// The call site of the request that applies `COUNT` to `argument`, so a
/// denied call identifies how deep the run got.
pub fn count_site(argument: u64) -> Location {
    let step = usize::try_from(argument).unwrap_or(usize::MAX);
    Location::at(Origin::Expression, &[step])
}

/// The integer a completed evaluation holds.
pub fn completed_int(evaluation: &Evaluation) -> Option<u64> {
    match &evaluation.outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(value))) => value.to_u64(),
        _ => None,
    }
}

/// A meter with `work_units` fuel and every other limit generous.
pub fn meter(work_units: u64) -> Meter {
    let limit = u64::from(u32::MAX);
    Meter::new(ScalarLimits {
        integer_bits: limit,
        decimal_digits: limit,
        scale_expansion: limit,
        text_input_bytes: limit,
        text_scalars: limit,
        normalized_scalars: limit,
        unit_edges: limit,
        value_occurrences: limit,
        work_units,
        result_units: limit,
    })
}

pub fn spent(meter: &Meter) -> u64 {
    meter.consumed(LimitKind::WorkUnits)
}

pub enum Expression {
    /// A literal: completes with no application and no charge.
    Literal(u64),
    /// Apply `callee` to `argument`, completing with its value.
    Apply { callee: NodeKey, argument: u64 },
    /// Apply `outer` to the values of `left` and then `right`: the source call
    /// `outer(left(x), right(y))`, whose argument applications run left to
    /// right before `outer`'s own call.
    Nested {
        outer: NodeKey,
        left: (NodeKey, u64),
        right: (NodeKey, u64),
    },
}

pub enum Task {
    Count,
    Boom,
    Literal(u64),
    Apply {
        callee: NodeKey,
        argument: u64,
    },
    Applied,
    Add,
    Refuse,
    Lossy,
    Nested {
        outer: NodeKey,
        left: (NodeKey, u64),
        right: (NodeKey, u64),
        values: Vec<u64>,
    },
    /// Admitted but never given an interpreted body.
    Bodiless,
}

/// The callables entered, in order, with their integer arguments.
pub type Binds = Arc<Mutex<Vec<(NodeKey, Vec<u64>)>>>;

#[derive(Default)]
pub struct Toy {
    binds: Binds,
}

impl Toy {
    /// The log of interpreted callables bound, in the order they were entered.
    pub fn binds(&self) -> Binds {
        Arc::clone(&self.binds)
    }

    fn admitted(callable: &NodeKey) -> bool {
        [
            count(),
            boom(),
            host_a(),
            host_b(),
            add(),
            refuse(),
            lossy(),
            ranged(),
            storage(),
            population(),
        ]
        .contains(callable)
    }

    fn arity(callable: &NodeKey) -> usize {
        if *callable == add() {
            2
        } else {
            1
        }
    }
}

fn integers(values: &[Value]) -> Vec<u64> {
    values
        .iter()
        .filter_map(|value| match value {
            Value::Integer(n) => n.to_u64(),
            _ => None,
        })
        .collect()
}

impl CheckedClosure for Toy {
    type Expression = Expression;
    type Environment = ();
    type Task = (Task, Vec<Value>);

    fn signature(&self, callable: &NodeKey) -> Option<Signature> {
        Self::admitted(callable).then(|| signature_of_arity(Self::arity(callable)))
    }

    fn admit_call(
        &self,
        callable: &NodeKey,
        arguments: &[Value],
        _environment: &(),
        meter: &Meter,
    ) -> Result<(), CallFailure> {
        if let Some(cause) = meter.cancel().and_then(quire_exact::Cancel::cause) {
            return Err(CallFailure::Cancelled(cause));
        }
        if !Self::admitted(callable) {
            return Err(InputRefusal::UnknownFunction("absent".into()).into());
        }
        if arguments.len() != Self::arity(callable) {
            return Err(InputRefusal::Arity {
                declared: Self::arity(callable),
                supplied: arguments.len(),
            }
            .into());
        }
        Ok(())
    }

    fn admit_request(
        &self,
        callable: &NodeKey,
        arguments: &[Value],
        _environment: &(),
    ) -> Result<(), RequestRefusal> {
        if !Self::admitted(callable) || arguments.len() != Self::arity(callable) {
            return Err(RequestRefusal::NotAdmitted);
        }
        let first = integers(arguments).first().copied().unwrap_or_default();
        if *callable == ranged() && first > 9 {
            return Err(RequestRefusal::NotAdmitted);
        }
        if *callable == population() && first != 3 {
            return Err(RequestRefusal::NotAdmitted);
        }
        if *callable == storage() {
            return Err(RequestRefusal::Storage {
                parameter: 0,
                failure: EnvironmentFailure::Allocation(StorageRequest::new(
                    8,
                    StorageUnit::AdditionalElements,
                )),
            });
        }
        Ok(())
    }

    fn admit_expression(
        &self,
        _expression: &Expression,
        parameters: &[Value],
        _environment: &(),
        _meter: &Meter,
    ) -> Result<(), CallFailure> {
        if parameters.is_empty() {
            Ok(())
        } else {
            Err(InputRefusal::Arity {
                declared: 0,
                supplied: parameters.len(),
            }
            .into())
        }
    }

    fn bind_call(
        &self,
        callable: &NodeKey,
        arguments: Vec<Value>,
        _environment: &(),
    ) -> Result<Self::Task, InternalFault> {
        self.binds
            .lock()
            .expect("bind log")
            .push((*callable, integers(&arguments)));
        let task = if *callable == count() {
            Task::Count
        } else if *callable == boom() {
            Task::Boom
        } else if *callable == add() || *callable == ranged() || *callable == population() {
            Task::Add
        } else if *callable == refuse() {
            Task::Refuse
        } else if *callable == lossy() {
            Task::Lossy
        } else {
            Task::Bodiless
        };
        Ok((task, arguments))
    }

    fn bind_expression(
        &self,
        expression: &Expression,
        parameters: Vec<Value>,
        _environment: &(),
    ) -> Result<Self::Task, InternalFault> {
        let task = match expression {
            Expression::Literal(n) => Task::Literal(*n),
            Expression::Apply { callee, argument } => Task::Apply {
                callee: *callee,
                argument: *argument,
            },
            Expression::Nested { outer, left, right } => Task::Nested {
                outer: *outer,
                left: *left,
                right: *right,
                values: Vec::new(),
            },
        };
        Ok((task, parameters))
    }

    fn advance(
        &self,
        task: &mut Self::Task,
        delivered: Option<Evaluation>,
        _environment: &(),
        _meter: &mut Meter,
    ) -> Result<Transition, InternalFault> {
        let (state, arguments) = task;
        let argument = arguments
            .first()
            .and_then(|value| match value {
                Value::Integer(n) => n.to_u64(),
                _ => None,
            })
            .unwrap_or_default();
        match state {
            Task::Count => match delivered {
                Some(child) => Ok(Transition::Done(match completed_int(&child) {
                    Some(value) => Evaluation::completed(int(value + 1)),
                    // A non-completed child propagates unchanged.
                    None => child,
                })),
                None if argument == 0 => Ok(Transition::Done(Evaluation::completed(int(0)))),
                None => Ok(Transition::Call(ChildRequest {
                    callee: count(),
                    arguments: vec![int(argument - 1)],
                    site: count_site(argument - 1),
                })),
            },
            Task::Boom => Ok(Transition::Done(Evaluation {
                outcome: FamilyOutcome::Evaluated(Outcome::Undefined(Undefined::DivisionByZero)),
                location: Some(site()),
                losses: Vec::new(),
            })),
            Task::Add => Ok(Transition::Done(Evaluation::completed(int(integers(
                arguments,
            )
            .iter()
            .sum())))),
            Task::Refuse => Ok(Transition::Done(Evaluation {
                outcome: FamilyOutcome::Evaluated(Outcome::Refused(
                    Refusal::IeeeNanPayloadNotRepresentable {
                        target: IeeeWidth::Binary32,
                        source: IeeeWidth::Binary64,
                    },
                )),
                location: Some(site()),
                losses: Vec::new(),
            })),
            Task::Lossy => Ok(Transition::Done(Evaluation {
                outcome: FamilyOutcome::Evaluated(Outcome::Completed(int(1))),
                location: None,
                losses: vec![loss()],
            })),
            Task::Nested {
                outer,
                left,
                right,
                values,
            } => {
                if let Some(child) = delivered {
                    match completed_int(&child) {
                        Some(value) => values.push(value),
                        None => return Ok(Transition::Done(child)),
                    }
                }
                let (callee, arguments) = match values.as_slice() {
                    [] => (left.0, vec![int(left.1)]),
                    [_] => (right.0, vec![int(right.1)]),
                    [l, r] => (*outer, vec![int(*l), int(*r)]),
                    [.., last] => {
                        return Ok(Transition::Done(Evaluation::completed(int(*last))));
                    }
                };
                Ok(Transition::Call(ChildRequest {
                    callee,
                    arguments,
                    site: site(),
                }))
            }
            Task::Literal(n) => Ok(Transition::Done(Evaluation::completed(int(*n)))),
            Task::Apply { callee, argument } => {
                let request = ChildRequest {
                    callee: *callee,
                    arguments: vec![int(*argument)],
                    site: site(),
                };
                *state = Task::Applied;
                Ok(Transition::Call(request))
            }
            Task::Applied => delivered
                .map(Transition::Done)
                .ok_or_else(|| fault("applied-frame-resumed-without-a-child")),
            Task::Bodiless => Err(fault("callable-has-no-interpreted-body")),
        }
    }
}

fn fault(invariant: &'static str) -> InternalFault {
    InternalFault::new("toy", invariant)
}

/// The loss record `LOSSY` completes with.
pub fn loss() -> LocatedLoss {
    LocatedLoss {
        location: site(),
        loss: ValueLoss::IeeeFlags([IeeeFlag::Inexact].into_iter().collect()),
    }
}

/// What a recording body factory observed.
#[derive(Clone, Default)]
pub struct Counts {
    begun: Arc<AtomicUsize>,
    resumed: Arc<AtomicUsize>,
}

impl Counts {
    pub fn begun(&self) -> usize {
        self.begun.load(Ordering::SeqCst)
    }
    pub fn resumed(&self) -> usize {
        self.resumed.load(Ordering::SeqCst)
    }
}

/// How a host body finishes after its child returns.
#[derive(Clone, Copy)]
pub enum Finish {
    /// Return the child's evaluation unchanged (a direct-return caller).
    Direct,
    /// Return the child's integer plus the argument the body saved at start.
    AddSavedArgument,
}

/// A host body that applies `target` to its argument and finishes per `finish`.
pub struct Relay {
    pub target: NodeKey,
    pub finish: Finish,
    pub counts: Counts,
}

impl BodyFactory for Relay {
    fn begin(&self) -> Box<dyn Body> {
        self.counts.begun.fetch_add(1, Ordering::SeqCst);
        Box::new(RelayBody {
            target: self.target,
            finish: self.finish,
            counts: self.counts.clone(),
            saved: None,
        })
    }
}

struct RelayBody {
    target: NodeKey,
    finish: Finish,
    counts: Counts,
    saved: Option<u64>,
}

impl Body for RelayBody {
    fn resume(&mut self, input: Resume, _meter: &mut Meter) -> Result<Transition, InternalFault> {
        self.counts.resumed.fetch_add(1, Ordering::SeqCst);
        match input {
            Resume::Start(arguments) => {
                let argument = arguments.first().cloned().unwrap_or_else(|| int(0));
                if let Value::Integer(n) = &argument {
                    self.saved = n.to_u64();
                }
                Ok(Transition::Call(ChildRequest {
                    callee: self.target,
                    arguments: vec![argument],
                    site: site(),
                }))
            }
            Resume::Child(child) => Ok(Transition::Done(match self.finish {
                Finish::Direct => child,
                Finish::AddSavedArgument => match (completed_int(&child), self.saved) {
                    (Some(value), Some(saved)) => Evaluation::completed(int(value + saved)),
                    _ => child,
                },
            })),
            Resume::Again => Err(fault("relay-resumed-without-input")),
        }
    }
}

/// A host body for FR-262-AC-8: it charges the meter, saves a local,
/// requests a child, charges again and continues with the child's value,
/// keeping the child's loss records.
pub struct ChargingRelay {
    pub target: NodeKey,
    pub counts: Counts,
}

impl BodyFactory for ChargingRelay {
    fn begin(&self) -> Box<dyn Body> {
        self.counts.begun.fetch_add(1, Ordering::SeqCst);
        Box::new(ChargingBody {
            target: self.target,
            counts: self.counts.clone(),
            saved: 0,
        })
    }
}

struct ChargingBody {
    target: NodeKey,
    counts: Counts,
    saved: u64,
}

impl ChargingBody {
    /// One unit of the body's own kernel work. A denial ends the body with the
    /// located incomplete evaluation, as the `Body` contract says.
    fn work(meter: &mut Meter) -> Option<Transition> {
        let denied = meter
            .charge(Charge::new(ChargePoint::IntegerArithmeticArithmetic))
            .err()?;
        Some(Transition::Done(Evaluation {
            outcome: FamilyOutcome::Evaluated(Outcome::Incomplete(denied)),
            location: Some(site()),
            losses: Vec::new(),
        }))
    }
}

impl Body for ChargingBody {
    fn resume(&mut self, input: Resume, meter: &mut Meter) -> Result<Transition, InternalFault> {
        self.counts.resumed.fetch_add(1, Ordering::SeqCst);
        if let Some(done) = Self::work(meter) {
            return Ok(done);
        }
        match input {
            Resume::Start(arguments) => {
                self.saved = integers(&arguments).first().copied().unwrap_or_default();
                Ok(Transition::Call(ChildRequest {
                    callee: self.target,
                    arguments,
                    site: site(),
                }))
            }
            Resume::Child(child) => Ok(Transition::Done(match completed_int(&child) {
                Some(value) => Evaluation {
                    outcome: FamilyOutcome::Evaluated(Outcome::Completed(int(value + self.saved))),
                    location: None,
                    losses: child.losses,
                },
                None => child,
            })),
            Resume::Again => Err(fault("charging-body-resumed-without-input")),
        }
    }
}

/// A host body that requests `callee` with `arguments` as given, however
/// invalid, to exercise the scheduler's request validation.
pub struct BadRequest {
    pub callee: NodeKey,
    pub arguments: Vec<Value>,
}

impl BodyFactory for BadRequest {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(BadRequestBody {
            callee: self.callee,
            arguments: self.arguments.clone(),
        })
    }
}

struct BadRequestBody {
    callee: NodeKey,
    arguments: Vec<Value>,
}

impl Body for BadRequestBody {
    fn resume(&mut self, _input: Resume, _meter: &mut Meter) -> Result<Transition, InternalFault> {
        Ok(Transition::Call(ChildRequest {
            callee: self.callee,
            arguments: self.arguments.clone(),
            site: site(),
        }))
    }
}

/// A host body that yields once (a chargeless `Continue`) before requesting
/// `target` with its argument.
pub struct Yielding {
    pub target: NodeKey,
}

impl BodyFactory for Yielding {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(YieldingBody {
            target: self.target,
            arguments: Vec::new(),
        })
    }
}

struct YieldingBody {
    target: NodeKey,
    arguments: Vec<Value>,
}

impl Body for YieldingBody {
    fn resume(&mut self, input: Resume, _meter: &mut Meter) -> Result<Transition, InternalFault> {
        match input {
            Resume::Start(arguments) => {
                self.arguments = arguments;
                Ok(Transition::Continue)
            }
            Resume::Again => Ok(Transition::Call(ChildRequest {
                callee: self.target,
                arguments: self.arguments.clone(),
                site: site(),
            })),
            Resume::Child(child) => Ok(Transition::Done(child)),
        }
    }
}
