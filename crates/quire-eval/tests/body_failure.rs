// SPDX-License-Identifier: AGPL-3.0-or-later
//! Body-side SV errors remain call failures, separate from values and faults.

mod support;

use quire_eval::{
    Admission, Body, BodyFactory, BodyFailure, CallFailure, Evaluation, FamilyOutcome, Progress,
    Registry, Resume, Transition,
};
use quire_exact::{Cancel, CancelCause, Charge, ChargePoint, Meter, Outcome, Value};
use quire_semantic_value::catalog::{Category, InternalFault};
use quire_semantic_value::declaration::{
    DeclarationCause, EnvironmentFailure, EnvironmentLimit, EnvironmentLimitKind,
    InvalidDeclaration, StorageRequest, StorageUnit,
};
use support::{host_a, int, int_signature, meter, spent, Toy};

#[derive(Clone)]
struct ReturnFailure(BodyFailure);

struct FailureBody(BodyFailure);

impl BodyFactory for ReturnFailure {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(FailureBody(self.0.clone()))
    }
}

impl Body for FailureBody {
    fn resume(&mut self, _input: Resume, meter: &mut Meter) -> Result<Transition, BodyFailure> {
        meter
            .charge(Charge::new(ChargePoint::IntegerArithmeticArithmetic))
            .expect("the test meter admits the body's preceding work");
        Err(self.0.clone())
    }
}

#[derive(Clone)]
struct ReturnFalse;

struct FalseBody;

impl BodyFactory for ReturnFalse {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(FalseBody)
    }
}

impl Body for FalseBody {
    fn resume(&mut self, _input: Resume, _meter: &mut Meter) -> Result<Transition, BodyFailure> {
        Ok(Transition::Done(Evaluation::completed(Value::Boolean(
            false,
        ))))
    }
}

fn environment_failures() -> Vec<(BodyFailure, Category)> {
    let ordinary_work_limit =
        EnvironmentFailure::Limit(EnvironmentLimit::new(EnvironmentLimitKind::WorkUnits, 8, 9));
    let cancel = Cancel::new();
    cancel.cancel(CancelCause::Deadline);
    assert!(
        cancel.poll(),
        "the exact test handle records its cancellation"
    );
    let cancelled_work_limit =
        EnvironmentFailure::Limit(EnvironmentLimit::new(EnvironmentLimitKind::WorkUnits, 8, 9));
    let ancestor_limit = EnvironmentFailure::Limit(EnvironmentLimit::new(
        EnvironmentLimitKind::AncestorSteps,
        2,
        3,
    ));
    let refusal = EnvironmentFailure::Refused(InvalidDeclaration {
        declaration: "Pair".to_owned(),
        cause: DeclarationCause::DuplicateKey,
    });
    let allocation = EnvironmentFailure::Allocation(StorageRequest::new(3, StorageUnit::Bytes));
    let capacity = EnvironmentFailure::Capacity(StorageRequest::new(
        usize::MAX,
        StorageUnit::AdditionalElements,
    ));

    vec![
        (
            BodyFailure::qsv_environment(ordinary_work_limit, None),
            Category::Incomplete,
        ),
        (
            BodyFailure::qsv_environment(cancelled_work_limit, Some(&cancel)),
            Category::Incomplete,
        ),
        (
            BodyFailure::qsv_environment(ancestor_limit, Some(&cancel)),
            Category::Incomplete,
        ),
        (
            BodyFailure::qsv_environment(refusal, Some(&cancel)),
            Category::Refusal,
        ),
        (
            BodyFailure::qsv_environment(allocation, Some(&cancel)),
            Category::Incomplete,
        ),
        (
            BodyFailure::qsv_environment(capacity, Some(&cancel)),
            Category::Incomplete,
        ),
        (
            BodyFailure::Fault(InternalFault::new("generated", "broken-invariant")),
            Category::InternalFailure,
        ),
    ]
}

fn register(factory: impl BodyFactory + 'static) -> Registry<Toy> {
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(host_a(), &int_signature(), factory),
        Admission::Accepted
    );
    registry
}

fn run_failure(failure: BodyFailure) -> (Result<Evaluation, CallFailure>, u64) {
    let mut registry = register(ReturnFailure(failure));
    let mut meter = meter(100);
    let result = registry
        .begin_call(host_a(), vec![int(1)], &(), &mut meter)
        .expect("the callable arguments are admitted")
        .run();
    (result, spent(&meter))
}

fn step_failure(failure: BodyFailure) -> (Result<Evaluation, CallFailure>, u64) {
    let mut registry = register(ReturnFailure(failure));
    let mut meter = meter(100);
    let session = registry
        .begin_call(host_a(), vec![int(1)], &(), &mut meter)
        .expect("the callable arguments are admitted");
    let mut session = session;
    let result = loop {
        match session.step() {
            Progress::Runnable(next) => session = next,
            Progress::Terminal(result) => break result,
        }
    };
    (result, spent(&meter))
}

#[test]
/// Trace: FR-401-AC-12, FR-401-AC-13, FR-485-AC-15.
fn body_failures_preserve_variants_categories_and_prior_charges_through_run_and_step() {
    for (failure, category) in environment_failures() {
        for execute in [run_failure as fn(BodyFailure) -> _, step_failure] {
            let expected = failure.clone();
            let (result, consumed) = execute(failure.clone());
            let Err(CallFailure::Body(found)) = result else {
                panic!("body failure remains a typed call failure: {result:?}");
            };
            assert_eq!(found.as_ref(), &expected, "the owner payload is unchanged");
            assert_eq!(CallFailure::Body(found).category(), category);
            assert_eq!(
                consumed, 2,
                "root and prior body charges remain; failure adds none"
            );
        }
    }
}

#[test]
/// Trace: FR-401-AC-12, FR-485-AC-15.
fn a_false_completed_value_is_not_a_body_failure_or_declaration_refusal() {
    let mut registry = register(ReturnFalse);
    let mut meter = meter(100);
    let result = registry
        .begin_call(host_a(), vec![int(1)], &(), &mut meter)
        .expect("the callable arguments are admitted")
        .run()
        .expect("false is a completed membership answer");
    assert!(matches!(
        result.outcome,
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Boolean(false)))
    ));
    assert_eq!(result.location, None);
    assert_eq!(spent(&meter), 1, "only the root call was charged");
}

#[test]
/// Trace: FR-401-AC-13.
fn qsv_cancellation_snapshot_requires_a_tripped_work_units_handle() {
    let ordinary = BodyFailure::qsv_environment(
        EnvironmentFailure::Limit(EnvironmentLimit::new(EnvironmentLimitKind::WorkUnits, 4, 5)),
        None,
    );
    assert_eq!(
        ordinary,
        BodyFailure::QsvEnvironment {
            failure: EnvironmentFailure::Limit(EnvironmentLimit::new(
                EnvironmentLimitKind::WorkUnits,
                4,
                5,
            )),
            cancellation: None,
        }
    );

    let cancel = Cancel::new();
    cancel.cancel(CancelCause::Requested);
    assert!(cancel.poll());
    let cancelled = BodyFailure::qsv_environment(
        EnvironmentFailure::Limit(EnvironmentLimit::new(EnvironmentLimitKind::WorkUnits, 4, 5)),
        Some(&cancel),
    );
    assert_eq!(
        cancelled,
        BodyFailure::QsvEnvironment {
            failure: EnvironmentFailure::Limit(EnvironmentLimit::new(
                EnvironmentLimitKind::WorkUnits,
                4,
                5,
            )),
            cancellation: Some(CancelCause::Requested),
        }
    );
}
