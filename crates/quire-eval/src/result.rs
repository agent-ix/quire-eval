// SPDX-License-Identifier: AGPL-3.0-or-later
//! The evaluation result vocabulary (FR-090, ADR-013 O-16): what an
//! application of checked code returns, and how it fails before or outside
//! an outcome.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use quire_exact::{CancelCause, NodeKey, Outcome, Value};
use quire_semantic_value::admission::AdmissionRecord;
use quire_semantic_value::call::InputRefusal;
use quire_semantic_value::catalog::{CatalogCoded, Category, InternalFault, UndefinedCoded};
use quire_semantic_value::declaration::EnvironmentFailure;
use quire_semantic_value::location::ExpressionLocation;
use quire_semantic_value::loss::LocatedLoss;
use quire_semantic_value::supplied_stop::{
    SuppliedCause, SuppliedFailure, SuppliedStop, UnsupportedHelperStop,
};

use crate::body::BodyFailure;
use crate::prepare_limit::PrepareLimitReached;
use crate::unsupported::UnsupportedEntry;

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
    pub location: Option<ExpressionLocation>,
    /// The loss records of the operations a completed evaluation performed,
    /// in evaluation order; empty unless completed.
    pub losses: Vec<LocatedLoss>,
}

/// The locus named by an admission failure for a checked package call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallAdmissionLocus {
    /// The checked package as a whole, without a node or member payload.
    Package,
    /// The selected checked node.
    Node(NodeKey),
    /// The population node whose input admission observed the defect.
    Population(NodeKey),
    /// A member of the selected node, optionally at one list index.
    Member {
        /// The member name.
        name: String,
        /// The list position, when the member is indexed.
        index: Option<usize>,
    },
    /// A supplied binder.
    Binder(NodeKey),
}

/// The original input-path diagnostic and checked-package locus of a call
/// admission failure. Admission failures occur before S6a and do not charge
/// the meter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallAdmissionRecord {
    /// The shared diagnostic, including catalog pair, named fields, path and
    /// kernel construction cause.
    pub record: AdmissionRecord,
    /// The checked-package input that admission names.
    pub locus: CallAdmissionLocus,
}

/// A caller-supplied checked-package input that admission rejected or could
/// not decide before evaluation.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CallAdmissionFailure {
    /// Admission found a caller-input defect.
    #[error("call admission refused: {0:?}")]
    Refused(CallAdmissionRecord),
    /// Admission lacked evidence required to decide the input.
    #[error("call admission incomplete: {0:?}")]
    Incomplete(CallAdmissionRecord),
}

/// The error of an application, preserving ordinary caller-input refusals,
/// checked-package admission failures, evaluator invariant faults, typed
/// body-side QSV failures, and failures before or outside an outcome as
/// distinct shapes.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CallFailure {
    /// An argument refused at admission, before any S6a evaluation runs.
    #[error(transparent)]
    Input(#[from] InputRefusal),
    /// A checked-package-specific selection or supply was refused or left
    /// incomplete before S6a. Its package/node/population/member/binder locus
    /// are preserved separately from function-argument [`Self::Input`] and
    /// checked-body [`Self::Body`] failures. The original QSV input-path
    /// diagnostic is retained with its checked-package locus.
    #[error(transparent)]
    Admission(Box<CallAdmissionFailure>),
    /// An S6a invariant broke (ADR-013 T-4): never a caller-input refusal,
    /// and never surfaced as a [`FamilyResult`] or as an `Evaluation` whose
    /// outcome is `Outcome::Refused`.
    #[error("internal fault in {}: {}", .0.stage(), .0.invariant())]
    Fault(InternalFault),
    /// A checked body returned a typed QSV environment failure or its own
    /// invariant fault. This preserves the body-side source of the failure,
    /// including QSV refusal and resource-exhaustion payloads. The box keeps
    /// this public error below the application's by-value error-size bound.
    #[error(transparent)]
    Body(Box<BodyFailure>),
    /// The caller's [`quire_exact::Cancel`] handle was cancelled during a
    /// supplied-value run at `Begin`, before any charge (ADR-029 LC-3, FR-276). A charge
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
    /// An entry the chosen execution backend cannot run (FR-294, FR-296).
    /// Category unsupported, catalog code `unimplemented_capability`. No
    /// evaluator in this crate returns it.
    #[error(transparent)]
    Unsupported(#[from] UnsupportedEntry),
    /// A `PrepareLimits` bound that `prepare` reached (FR-279, FR-294).
    /// Category incomplete, catalog code `stage_limit_exceeded`. No evaluator
    /// in this crate returns it.
    #[error(transparent)]
    Limit(#[from] PrepareLimitReached),
}

impl From<InternalFault> for CallFailure {
    fn from(fault: InternalFault) -> Self {
        Self::Fault(fault)
    }
}

impl From<BodyFailure> for CallFailure {
    fn from(failure: BodyFailure) -> Self {
        Self::Body(Box::new(failure))
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
    /// Input and package-admission refusals, and body-side QSV refusals, are
    /// refusals; evaluator and body invariant faults are internal failures;
    /// resource limits and cancellations are incomplete (FR-285, FR-401).
    #[must_use]
    pub fn category(&self) -> Category {
        match self {
            Self::Input(_) => Category::Refusal,
            Self::Admission(failure) => match failure.as_ref() {
                CallAdmissionFailure::Refused(_) => Category::Refusal,
                CallAdmissionFailure::Incomplete(_) => Category::Incomplete,
            },
            Self::Fault(fault) => fault.category(),
            Self::Body(body) => match body.as_ref() {
                BodyFailure::Fault(fault) => fault.category(),
                BodyFailure::QsvEnvironment { failure, .. } => match failure {
                    EnvironmentFailure::Refused(_) => Category::Refusal,
                    EnvironmentFailure::Limit(_)
                    | EnvironmentFailure::Allocation(_)
                    | EnvironmentFailure::Capacity(_) => Category::Incomplete,
                },
            },
            Self::Cancelled(_) | Self::Storage { .. } => Category::Incomplete,
            Self::Supplied(stop) => match stop.cause {
                SuppliedCause::Input { .. } => Category::Refusal,
                SuppliedCause::Limit(_)
                | SuppliedCause::Cancelled(_)
                | SuppliedCause::Storage(_)
                | SuppliedCause::Capacity(_) => Category::Incomplete,
            },
            Self::UnsupportedHelper(_) | Self::Unsupported(_) => Category::Unsupported,
            Self::Limit(_) => Category::Incomplete,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ObservationAdmission;
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::vec;
    use quire_exact::{ConstructionCause, PathSegment};
    use quire_semantic_value::admission::PathStep;
    use quire_semantic_value::catalog::CatalogCode;

    fn diagnostic() -> AdmissionRecord {
        AdmissionRecord {
            catalog: CatalogCode::new("invalid_runtime_input", "invalid-value"),
            fields: BTreeMap::from([
                ("population", String::from("customers")),
                ("field", String::from("address")),
            ]),
            path: vec![
                PathStep::Field(String::from("0")),
                PathStep::OptionValue,
                PathStep::Index(2),
            ],
            kernel: Some(ConstructionCause::TypeMismatch),
        }
    }

    /// Trace: FR-005-AC-1, FR-005-AC-2.
    #[test]
    fn package_admission_retains_locus_and_diagnostic() {
        let diagnostic = diagnostic();
        let failure = CallFailure::Admission(Box::new(CallAdmissionFailure::Refused(
            CallAdmissionRecord {
                record: diagnostic.clone(),
                locus: CallAdmissionLocus::Package,
            },
        )));

        assert_eq!(failure.category(), Category::Refusal);
        let CallFailure::Admission(payload) = failure else {
            panic!("expected an admission failure");
        };
        let CallAdmissionFailure::Refused(record) = *payload else {
            panic!("expected an admission refusal");
        };
        assert_eq!(record.locus, CallAdmissionLocus::Package);
        assert_eq!(record.record, diagnostic);
    }

    /// Trace: FR-005-AC-1, FR-005-AC-2.
    #[test]
    fn existing_call_admission_loci_keep_their_payloads() {
        let key = NodeKey::new("test", vec![PathSegment::Text(String::from("node"))], 1)
            .expect("valid test key");
        let loci = [
            CallAdmissionLocus::Node(key.clone()),
            CallAdmissionLocus::Population(key.clone()),
            CallAdmissionLocus::Member {
                name: String::from("captures"),
                index: Some(2),
            },
            CallAdmissionLocus::Binder(key.clone()),
        ];

        assert_eq!(loci[0], CallAdmissionLocus::Node(key.clone()));
        assert_eq!(loci[1], CallAdmissionLocus::Population(key.clone()));
        assert_eq!(
            loci[2],
            CallAdmissionLocus::Member {
                name: String::from("captures"),
                index: Some(2),
            }
        );
        assert_eq!(loci[3], CallAdmissionLocus::Binder(key));
        for locus in loci {
            assert_ne!(locus, CallAdmissionLocus::Package);
        }
    }

    /// Trace: FR-130-AC-3.
    /// Admission refusals retain the original input path at a population locus.
    #[test]
    fn admission_refusal_preserves_population_locus_and_diagnostic() {
        let population = NodeKey::new(
            "test",
            vec![PathSegment::Text(String::from("population"))],
            7,
        )
        .expect("valid test key");
        let diagnostic = diagnostic();
        let failure = CallFailure::Admission(Box::new(CallAdmissionFailure::Refused(
            CallAdmissionRecord {
                record: diagnostic.clone(),
                locus: CallAdmissionLocus::Population(population.clone()),
            },
        )));

        assert_eq!(failure.category(), Category::Refusal);
        let CallFailure::Admission(payload) = &failure else {
            panic!("expected a package admission failure");
        };
        let CallAdmissionFailure::Refused(record) = payload.as_ref() else {
            panic!("expected a refusal");
        };
        assert_eq!(record.locus, CallAdmissionLocus::Population(population));
        assert_eq!(record.record, diagnostic);
        assert_ne!(
            failure,
            CallFailure::Input(InputRefusal::WrongValueKind { parameter: 0 })
        );
    }

    /// Incomplete admission is distinct from refusal and retains the exact member/index
    /// locus that needs evidence.
    #[test]
    fn incomplete_admission_preserves_member_index_locus() {
        let failure = CallFailure::Admission(Box::new(CallAdmissionFailure::Incomplete(
            CallAdmissionRecord {
                record: AdmissionRecord {
                    catalog: CatalogCode::new(
                        "unavailable_observation",
                        "missing-required-artifact",
                    ),
                    fields: BTreeMap::new(),
                    path: Vec::new(),
                    kernel: None,
                },
                locus: CallAdmissionLocus::Member {
                    name: String::from("captures"),
                    index: Some(2),
                },
            },
        )));

        assert_eq!(failure.category(), Category::Incomplete);
        assert!(matches!(
            &failure,
            CallFailure::Admission(payload)
                if matches!(payload.as_ref(), CallAdmissionFailure::Incomplete(record)
                    if record.record.catalog == CatalogCode::new(
                        "unavailable_observation",
                        "missing-required-artifact",
                    ) && record.locus == CallAdmissionLocus::Member {
                        name: String::from("captures"),
                        index: Some(2),
                    })
        ));
        assert_ne!(
            failure,
            CallFailure::Fault(InternalFault::new("S6a", "bad-input"))
        );
    }

    /// Trace: FR-130-AC-4.
    /// Every observation result keeps its own kind and original payload.
    #[test]
    fn observation_authority_query_preserves_all_failure_kinds() {
        let diagnostic = diagnostic();
        let admitted = ObservationAdmission::Admitted(17_u8);
        assert_eq!(admitted.authority(), Ok(&17));

        let refused: ObservationAdmission<u8> = ObservationAdmission::Refused(diagnostic.clone());
        assert!(
            matches!(refused.authority(), Err(ObservationAdmission::Refused(record)) if record == &diagnostic)
        );
        let incomplete: ObservationAdmission<u8> =
            ObservationAdmission::Incomplete(diagnostic.clone());
        assert!(
            matches!(incomplete.authority(), Err(ObservationAdmission::Incomplete(record)) if record == &diagnostic)
        );
        let fault = InternalFault::new("admission", "missing-observation-authority");
        let failed: ObservationAdmission<u8> = ObservationAdmission::Fault(fault.clone());
        assert!(
            matches!(failed.authority(), Err(ObservationAdmission::Fault(actual)) if actual == &fault)
        );
    }
}
