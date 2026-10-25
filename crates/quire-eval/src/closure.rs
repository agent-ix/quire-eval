// SPDX-License-Identifier: AGPL-3.0-or-later
//! The contract an evaluator implementation supplies for one checked closure.

use alloc::vec::Vec;

use quire_exact::{Meter, NodeKey, Value};
use quire_semantic_value::catalog::InternalFault;
use quire_semantic_value::location::Location;

use crate::body::Transition;
use crate::result::{CallFailure, Evaluation};
use crate::signature::Signature;
use quire_semantic_value::declaration::EnvironmentFailure;

/// Why a body's nested request failed validation (FR-262).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestRefusal {
    /// The callee is not admitted, the argument count differs from its
    /// parameter count, or an argument is not a member of its parameter's
    /// type: a broken body. The session ends as a `CallFailure::Fault`.
    NotAdmitted,
    /// A membership check could not decide for want of storage. The session
    /// ends as [`CallFailure::Storage`] with this parameter index.
    Storage {
        /// The index of the callee's parameter.
        parameter: usize,
        /// The kernel's storage failure.
        failure: EnvironmentFailure,
    },
}

/// One checked closure, as an evaluator implementation exposes it to the
/// leaf: the callables its checker admitted, and the interpreted
/// transitions of their bodies.
///
/// The leaf reads the closure only through this trait. It names no checker,
/// package or model type: the checked-expression input, the admitted object
/// environment and the saved state of an interpreted frame are types the
/// implementation chooses, and the leaf moves them without inspecting them.
/// A callable is identified by the kernel [`NodeKey`] the checker minted for
/// it; the leaf resolves no name.
pub trait CheckedClosure {
    /// A checked expression the closure can evaluate with parameters.
    type Expression;
    /// The admitted object environment of an application.
    type Environment;
    /// The saved state of one interpreted frame, between transitions.
    type Task: Send;

    /// The admitted signature of `callable`, or `None` when the identity is
    /// not in this closure.
    ///
    /// Only callables a caller may apply by identity are admitted; a
    /// synthesized candidate body that no checked call can name is not.
    fn signature(&self, callable: &NodeKey) -> Option<Signature>;

    /// The checked body root location of `callable` in its declaring package,
    /// or `None` if the admitted callable has no such location.
    ///
    /// `Begin` treats an absent location after admission as a broken closure
    /// invariant, before sealing the registry or charging the meter.
    fn root_call_site(&self, callable: &NodeKey) -> Option<Location>;

    /// Validate an application of `callable` to `arguments`, before any
    /// charge: the callable is in the closure, the arguments are complete and
    /// each is a value of its parameter type.
    ///
    /// `meter` is the session's meter. Its installed cancellation handle, if
    /// any, is the handle a supplied-value run polls, so a cancelled handle
    /// ends `Begin` with [`CallFailure::Cancelled`] before any charge. The
    /// method charges nothing.
    ///
    /// # Errors
    ///
    /// The input refusal, or the storage or supplied-value stop, that admission
    /// reports, as [`CallFailure`].
    fn admit_call(
        &self,
        callable: &NodeKey,
        arguments: &[Value],
        environment: &Self::Environment,
        meter: &Meter,
    ) -> Result<(), CallFailure>;

    /// Validate a body's nested request to apply `callable` to `arguments`:
    /// the callable is in the closure, the argument count is its parameter
    /// count, and each argument is a member of its parameter type.
    ///
    /// This is not [`Self::admit_call`]: it runs no supplied-value run,
    /// charges neither the meter nor a supplied-value budget, and observes
    /// no cancellation. [`RequestRefusal::NotAdmitted`] is a broken body,
    /// which the session reports as an internal fault, never as caller
    /// input; [`RequestRefusal::Storage`] ends the session as
    /// [`CallFailure::Storage`].
    ///
    /// # Errors
    ///
    /// The refusal above.
    fn admit_request(
        &self,
        callable: &NodeKey,
        arguments: &[Value],
        environment: &Self::Environment,
    ) -> Result<(), RequestRefusal>;

    /// Validate an evaluation of `expression` with `parameters`, before any
    /// charge.
    ///
    /// # Errors
    ///
    /// As [`Self::admit_call`].
    fn admit_expression(
        &self,
        expression: &Self::Expression,
        parameters: &[Value],
        environment: &Self::Environment,
        meter: &Meter,
    ) -> Result<(), CallFailure>;

    /// Bind already-admitted `arguments` to the parameters of `callable` and
    /// return its first interpreted frame. Runs after the call's
    /// `function.call` charge; it executes no body step.
    ///
    /// # Errors
    ///
    /// A broken invariant.
    fn bind_call(
        &self,
        callable: &NodeKey,
        arguments: Vec<Value>,
        environment: &Self::Environment,
    ) -> Result<Self::Task, InternalFault>;

    /// Bind already-admitted `parameters` to `expression` and return its first
    /// interpreted frame. An expression root incurs no call charge.
    ///
    /// # Errors
    ///
    /// A broken invariant.
    fn bind_expression(
        &self,
        expression: &Self::Expression,
        parameters: Vec<Value>,
        environment: &Self::Environment,
    ) -> Result<Self::Task, InternalFault>;

    /// Advance an interpreted frame by one finite transition.
    ///
    /// `delivered` is the matching child's evaluation when the frame is
    /// resumed after a [`Transition::Call`], and `None` otherwise. The frame
    /// commits its state in `task` before returning. A source application
    /// is returned as [`Transition::Call`] after its arguments are evaluated
    /// left to right; the frame never recurses into an evaluator for it. A
    /// non-completed child evaluation propagates under the language's
    /// expression rules, without inventing a value.
    ///
    /// # Errors
    ///
    /// A broken invariant.
    fn advance(
        &self,
        task: &mut Self::Task,
        delivered: Option<Evaluation>,
        environment: &Self::Environment,
        meter: &mut Meter,
    ) -> Result<Transition, InternalFault>;
}
