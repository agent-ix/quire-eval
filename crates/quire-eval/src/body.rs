// SPDX-License-Identifier: AGPL-3.0-or-later
//! The resumable body contract: how a registered host body, and an
//! interpreted frame, run one finite transition at a time and hand every
//! nested application to the scheduler.

use alloc::boxed::Box;
use alloc::vec::Vec;

use quire_exact::{Meter, NodeKey, Value};
use quire_semantic_value::location::Location;

use crate::result::{CallFailure, Evaluation};

/// A body's request that the scheduler apply a callee.
///
/// The requesting body is suspended with its saved state as the continuation.
/// The scheduler validates the request, charges `function.call`, runs the
/// callee, and delivers the callee's [`Evaluation`] to that body as
/// [`Resume::Child`] exactly once.
#[derive(Clone, Debug)]
pub struct ChildRequest {
    /// The exact admitted callee, by its kernel node key.
    pub callee: NodeKey,
    /// The completed arguments, in declared parameter order.
    pub arguments: Vec<Value>,
    /// The call site, where a denied charge for this application is located.
    pub site: Location,
}

/// What a body is resumed with.
#[derive(Debug)]
pub enum Resume {
    /// The first resume of a registered host body: the completed arguments
    /// the scheduler admitted for its call.
    Start(Vec<Value>),
    /// The body yielded with [`Transition::Continue`] and is scheduled
    /// again with nothing new to deliver.
    Again,
    /// The matching child's evaluation, delivered once.
    Child(Evaluation),
}

/// What one finite body transition decided. The body has committed its state
/// before returning this.
#[derive(Debug)]
pub enum Transition {
    /// Still runnable: a scheduling boundary. It is not an outcome, a value or
    /// a charge point.
    Continue,
    /// Apply a callee and resume this body with its evaluation.
    Call(ChildRequest),
    /// This body's application is complete.
    Done(Evaluation),
}

/// The invocation state of one call of a registered host body.
///
/// A body must hand every governed nested application to the scheduler by
/// returning [`Transition::Call`]; it must never enter an evaluator
/// recursively to obtain a child's value. Each transition's local host work
/// must terminate; arbitrary host recursion or blocking I/O is outside the
/// checked evaluation contract.
///
/// The `meter` is the session's one meter, lent for this transition. A body
/// charges its own kernel work through it, at the charge points the
/// reference evaluator uses, and reports a denied charge by returning
/// `Done` with the located incomplete evaluation.
pub trait Body: Send {
    /// Run one finite transition.
    ///
    /// # Errors
    ///
    /// A broken invariant of the body's own, as [`CallFailure::Fault`].
    fn resume(&mut self, input: Resume, meter: &mut Meter) -> Result<Transition, CallFailure>;
}

/// Supplies fresh invocation state for each call of a registered callable.
///
/// A factory is stored at registration and invoked only when the scheduler
/// applies its callable, once per call.
pub trait BodyFactory: Send + Sync {
    /// The fresh invocation state of one call.
    fn begin(&self) -> Box<dyn Body>;
}
