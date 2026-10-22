// SPDX-License-Identifier: AGPL-3.0-or-later
//! The evaluation result vocabulary (FR-090, ADR-013 O-16): what an
//! application of checked code returns, and how it fails before or outside
//! an outcome.

use alloc::boxed::Box;
use alloc::vec::Vec;

use quire_exact::{CancelCause, Outcome, Value};
use quire_semantic_value::call::InputRefusal;
use quire_semantic_value::catalog::{CatalogCoded, Category, InternalFault, UndefinedCoded};
use quire_semantic_value::declaration::EnvironmentFailure;
use quire_semantic_value::location::Location;
use quire_semantic_value::loss::LocatedLoss;
use quire_semantic_value::supplied_stop::{
    SuppliedCause, SuppliedFailure, SuppliedStop, UnsupportedHelperStop,
};

/// ADR-013 O-16: a family's own evaluation-time result. `Refused`, category
/// `refusal`, holds a family-owned refusal cause; `Undefined`, category
/// `undefined`, holds a family-owned undefined cause. Both are trait objects,
/// not a shared enum: each family owns its own cause enum and no shared enum
/// lists every family's causes (FR-090 Behavior).
///
/// There is no `Clone`, `PartialEq` or `Eq`: a consumer matches the arm and
/// compares `catalog_code()` or `undefined_record()`.
#[derive(Debug)]
pub enum FamilyResult {
    /// Category `refusal`.
    Refused(Box<dyn CatalogCoded>),
    /// Category `undefined`.
    Undefined(Box<dyn UndefinedCoded>),
}

/// What a family's `evaluate` hook returns inside `Ok`. `Err` is reserved for
/// a broken S6a invariant ([`InternalFault`]): refusal and undefined are
/// outcome categories, so a hook never returns them in `Err` (FR-090).
#[derive(Debug)]
pub enum EvalOutcome<T> {
    /// The kernel evaluation outcome, unchanged.
    Kernel(Outcome<T>),
    /// A family-owned evaluation-time refusal or undefined result.
    Family(FamilyResult),
}

/// ADR-013 O-16's S6a seam result. Two arms: S6a's input admits no
/// `Relation` (FR-090). `cfg(seam_probe)` adds one probe-only variant
/// (FR-063) that no non-probe code constructs or matches.
#[derive(Debug)]
pub enum FamilyOutcome<T> {
    /// The kernel evaluation outcome, unchanged.
    Evaluated(Outcome<T>),
    /// The family ran and produced its own evaluation-time result.
    FamilyEvaluated(FamilyResult),
    /// FR-063: exists only so `--cfg seam_probe` makes every `match` over
    /// `FamilyOutcome` non-exhaustive. Never constructed outside the probe
    /// build.
    #[cfg(seam_probe)]
    __SeamProbe,
}

/// The S6a seam's pass-through (FR-090): `Kernel(o)` becomes `Evaluated(o)`
/// and `Family(r)` becomes `FamilyEvaluated(r)`, each unchanged.
impl<T> From<EvalOutcome<T>> for FamilyOutcome<T> {
    fn from(outcome: EvalOutcome<T>) -> Self {
        match outcome {
            EvalOutcome::Kernel(outcome) => Self::Evaluated(outcome),
            EvalOutcome::Family(result) => Self::FamilyEvaluated(result),
        }
    }
}

/// A completed, undefined, refused, incomplete or family-owned evaluation
/// result, located at the expression where a non-completed outcome originated
/// (FR-090).
///
/// There is no `Clone`: [`FamilyOutcome::FamilyEvaluated`] holds a boxed
/// trait object that is not `Clone`.
#[derive(Debug)]
pub struct Evaluation {
    /// The outcome.
    pub outcome: FamilyOutcome<Value>,
    /// Where a non-completed outcome originated; `None` when completed.
    pub location: Option<Location>,
    /// The loss records of the operations a completed evaluation performed,
    /// in evaluation order; empty unless completed.
    pub losses: Vec<LocatedLoss>,
}

/// FR-090 "Bad arguments are refused at admission; a broken invariant is an
/// internal fault": the error of an application, split between an ordinary
/// caller-input refusal admission catches and a broken S6a invariant
/// admission cannot have let through, never conflated into one shape.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CallFailure {
    /// An argument refused at admission, before any S6a evaluation runs.
    #[error(transparent)]
    Input(#[from] InputRefusal),
    /// An S6a invariant broke (ADR-013 T-4): never a caller-input refusal,
    /// and never surfaced as a [`FamilyResult`] or as an `Evaluation` whose
    /// outcome is `Outcome::Refused`.
    #[error("internal fault in {}: {}", .0.stage(), .0.invariant())]
    Fault(InternalFault),
    /// The caller's [`quire_exact::Cancel`] handle was cancelled during a
    /// supplied-value phase before the call (ADR-029 LC-3, FR-276). A charge
    /// the handle denies during an application is an `Incomplete` result, not
    /// this failure (FR-262). It returns no value.
    #[error("the call was cancelled: {0:?}")]
    Cancelled(CancelCause),
    /// An argument's membership in its parameter type was not decided because
    /// the type environment's admission worklist could not reserve storage.
    /// It is `resource_exhausted`, never a wrong-value-kind refusal.
    #[error("admission of argument {parameter} decided nothing: {failure:?}")]
    Storage {
        /// The parameter index.
        parameter: usize,
        /// The kernel's storage failure.
        failure: EnvironmentFailure,
    },
    /// A supplied-value phase stopped before any call at a helper ceiling, a
    /// failed reservation or an unrepresentable size (FR-277), with its phase
    /// context. No evaluation ran and nothing was charged to the meter.
    /// Invalid input and cancellation keep [`Self::Input`] and
    /// [`Self::Cancelled`].
    #[error("supplied-value {phase} stopped before the call: {cause:?}", phase = .0.context.phase.as_str(), cause = .0.cause)]
    Supplied(Box<SuppliedStop>),
    /// A supplied-admission walk reached a helper family the semantic-value
    /// crate has not qualified, before that helper ran. Distinct from every
    /// other arm.
    #[error("supplied-value {phase} reached the unqualified {family:?} helper", phase = .0.context.phase.as_str(), family = .0.family)]
    UnsupportedHelper(Box<UnsupportedHelperStop>),
}

impl From<InternalFault> for CallFailure {
    fn from(fault: InternalFault) -> Self {
        Self::Fault(fault)
    }
}

impl From<SuppliedFailure> for CallFailure {
    fn from(failure: SuppliedFailure) -> Self {
        match failure {
            SuppliedFailure::Stop(stop) => match stop.cause {
                SuppliedCause::Input { refusal, .. } => Self::Input(refusal),
                SuppliedCause::Cancelled(cause) => Self::Cancelled(cause),
                SuppliedCause::Limit(_)
                | SuppliedCause::Storage(_)
                | SuppliedCause::Capacity(_) => Self::Supplied(stop),
            },
            SuppliedFailure::Unsupported(stop) => Self::UnsupportedHelper(stop),
            SuppliedFailure::Fault(fault) => Self::Fault(fault),
        }
    }
}

impl CallFailure {
    /// FR-285: an admission refusal is a refusal, a broken S6a invariant an
    /// internal failure, and a cancellation incomplete (ADR-013 O-16).
    #[must_use]
    pub fn category(&self) -> Category {
        match self {
            Self::Input(_) => Category::Refusal,
            Self::Fault(fault) => fault.category(),
            Self::Cancelled(_) | Self::Storage { .. } => Category::Incomplete,
            Self::Supplied(stop) => match stop.cause {
                SuppliedCause::Input { .. } => Category::Refusal,
                SuppliedCause::Limit(_)
                | SuppliedCause::Cancelled(_)
                | SuppliedCause::Storage(_)
                | SuppliedCause::Capacity(_) => Category::Incomplete,
            },
            Self::UnsupportedHelper(_) => Category::Unsupported,
        }
    }
}

impl Evaluation {
    /// A completed evaluation of `value`: no location and no losses.
    #[must_use]
    pub const fn completed(value: Value) -> Self {
        Self {
            outcome: FamilyOutcome::Evaluated(Outcome::Completed(value)),
            location: None,
            losses: Vec::new(),
        }
    }
}
