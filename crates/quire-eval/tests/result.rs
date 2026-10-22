// SPDX-License-Identifier: AGPL-3.0-or-later
//! The evaluation result vocabulary through the public API (FR-090, FR-285).

use quire_eval::{CallFailure, EvalOutcome, FamilyOutcome};
use quire_exact::{CancelCause, Outcome, Value};
use quire_semantic_value::call::InputRefusal;
use quire_semantic_value::catalog::{Category, InternalFault};
use quire_semantic_value::supplied_stop::{
    AttemptedCount, SuppliedCause, SuppliedContext, SuppliedFailure, SuppliedLimit,
    SuppliedLimitKind, SuppliedPhase, SuppliedStop,
};

fn stop(cause: SuppliedCause) -> SuppliedStop {
    SuppliedStop {
        context: SuppliedContext {
            phase: SuppliedPhase::Admission,
            parameter: Some(0),
            event: None,
            locus: None,
            consumed: 0,
        },
        cause,
    }
}

fn limit() -> SuppliedCause {
    SuppliedCause::Limit(SuppliedLimit {
        limit_kind: SuppliedLimitKind::AdmissionWorkUnits,
        limit: 1,
        next_charge: 1,
        counter: AttemptedCount::Count(2),
        operand: None,
    })
}

#[test]
fn a_supplied_input_refusal_and_cancellation_keep_their_own_arms() {
    let input = stop(SuppliedCause::Input {
        refusal: InputRefusal::WrongValueKind { parameter: 0 },
        membership: None,
    });
    assert_eq!(
        CallFailure::from(SuppliedFailure::from(input)),
        CallFailure::Input(InputRefusal::WrongValueKind { parameter: 0 })
    );
    let cancelled = stop(SuppliedCause::Cancelled(CancelCause::Deadline));
    assert_eq!(
        CallFailure::from(SuppliedFailure::from(cancelled)),
        CallFailure::Cancelled(CancelCause::Deadline)
    );
    let fault = InternalFault::new("S6a", "x");
    assert_eq!(
        CallFailure::from(SuppliedFailure::Fault(fault.clone())),
        CallFailure::Fault(fault)
    );
}

#[test]
fn a_supplied_limit_stays_a_supplied_failure_and_is_incomplete() {
    let failure = CallFailure::from(SuppliedFailure::from(stop(limit())));
    assert!(matches!(failure, CallFailure::Supplied(_)));
    assert_eq!(failure.category(), Category::Incomplete);
}

#[test]
fn each_call_failure_arm_has_its_category() {
    assert_eq!(
        CallFailure::Input(InputRefusal::Arity {
            declared: 1,
            supplied: 0
        })
        .category(),
        Category::Refusal
    );
    assert_eq!(
        CallFailure::Fault(InternalFault::new("S6a", "x")).category(),
        Category::InternalFailure
    );
    assert_eq!(
        CallFailure::Cancelled(CancelCause::Requested).category(),
        Category::Incomplete
    );
}

#[test]
fn a_kernel_outcome_passes_through_the_seam_unchanged() {
    let outcome = FamilyOutcome::from(EvalOutcome::Kernel(Outcome::Completed(Value::Boolean(
        true,
    ))));
    assert!(matches!(
        outcome,
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(true)))
    ));
}
