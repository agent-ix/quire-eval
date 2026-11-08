// SPDX-License-Identifier: AGPL-3.0-or-later
//! The resumable body contract: how a registered host body, and an
//! interpreted frame, run one finite transition at a time and hand every
//! nested application to the scheduler.

use alloc::boxed::Box;
use alloc::vec::Vec;

use quire_exact::{Cancel, CancelCause, Meter, NodeKey, Value};
use quire_semantic_value::catalog::InternalFault;
use quire_semantic_value::declaration::{EnvironmentFailure, EnvironmentLimitKind};
use quire_semantic_value::location::ExpressionLocation;

use crate::result::Evaluation;

/// A failure returned by a checked body while it is producing one transition.
///
/// QSV environment failures remain distinct from faults in the body's own
/// invariants. The cancellation snapshot is populated only for a `WorkUnits`
/// limit and only when the exact cancellation handle used by that QSV
/// operation reports that a charge saw its cancellation.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BodyFailure {
    /// Type-environment construction or membership failed in QSV.
    #[error("QSV type environment failed: {failure:?}")]
    QsvEnvironment {
        /// The unchanged QSV failure and its full payload.
        failure: EnvironmentFailure,
        /// The exact cancellation observed by the QSV operation, if any.
        cancellation: Option<CancelCause>,
    },
    /// A genuine invariant fault in the body implementation.
    #[error("internal fault in {}: {}", .0.stage(), .0.invariant())]
    Fault(InternalFault),
}

impl BodyFailure {
    /// Preserve a QSV failure and snapshot cancellation from the same handle
    /// used by the failed operation. Cancellation is meaningful only when a
    /// `WorkUnits` limit was reached; other limit kinds and failure variants
    /// carry no cancellation cause.
    #[must_use]
    pub fn qsv_environment(failure: EnvironmentFailure, cancel: Option<&Cancel>) -> Self {
        let cancellation = match &failure {
            EnvironmentFailure::Limit(limit) if limit.kind() == EnvironmentLimitKind::WorkUnits => {
                cancel.and_then(Cancel::tripped)
            }
            EnvironmentFailure::Limit(_)
            | EnvironmentFailure::Refused(_)
            | EnvironmentFailure::Allocation(_)
            | EnvironmentFailure::Capacity(_) => None,
        };
        Self::QsvEnvironment {
            failure,
            cancellation,
        }
    }
}

impl From<InternalFault> for BodyFailure {
    fn from(fault: InternalFault) -> Self {
        Self::Fault(fault)
    }
}

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
    pub site: ExpressionLocation,
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
    Child(Box<Evaluation>),
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
/// `Done` with the located incomplete evaluation. A failure constructing or
/// querying a QSV type environment is returned as [`BodyFailure::QsvEnvironment`];
/// it must not be converted to a completed value or an internal fault. Only a
/// broken body invariant is [`BodyFailure::Fault`].
pub trait Body: Send {
    /// Run one finite transition.
    ///
    /// # Errors
    ///
    /// A typed QSV environment failure or a broken invariant of the body.
    fn resume(&mut self, input: Resume, meter: &mut Meter) -> Result<Transition, BodyFailure>;
}

/// Supplies fresh invocation state for each call of a registered callable.
///
/// A factory is stored at registration and invoked only when the scheduler
/// applies its callable, once per call.
pub trait BodyFactory: Send + Sync {
    /// The fresh invocation state of one call.
    fn begin(&self) -> Box<dyn Body>;
}
