// SPDX-License-Identifier: AGPL-3.0-or-later
//! The scheduler: depth, fuel, handoff, cancellation and release (FR-262).

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;

use quire_eval::{Admission, CallFailure, Evaluation, FamilyOutcome, Progress, Registry, Session};
use quire_exact::{Cancel, CancelCause, ChargePoint, LimitKind, Outcome, Undefined};
use support::{
    boom, completed_int, count, host_a, host_b, int, int_signature, meter, no_cancel, site, spent,
    Counts, Expression, Finish, Relay, Toy,
};

/// Step a session to its terminal result, counting the steps.
fn drive(session: Session<'_, Toy>) -> (Result<Evaluation, CallFailure>, usize) {
    let mut steps = 0;
    let mut session = session;
    loop {
        steps += 1;
        match session.step() {
            Progress::Runnable(next) => session = next,
            Progress::Terminal(result) => return (result, steps),
        }
    }
}

fn runnable(progress: Progress<'_, Toy>) -> Session<'_, Toy> {
    match progress {
        Progress::Runnable(session) => session,
        Progress::Terminal(_) => panic!("the session ended before the expected suspension"),
    }
}

/// A registry with `HOST_A` relaying (adding its saved argument) to `HOST_B`,
/// which relays directly to the interpreted `COUNT`: a generated body calling
/// a generated body calling an interpreted function.
fn chain(a: &Counts, b: &Counts) -> Registry<Toy> {
    let mut registry = Registry::new(Toy);
    assert_eq!(
        registry.register(
            host_a(),
            &int_signature(),
            Relay {
                target: host_b(),
                finish: Finish::AddSavedArgument,
                counts: a.clone(),
            },
        ),
        Admission::Accepted
    );
    assert_eq!(
        registry.register(
            host_b(),
            &int_signature(),
            Relay {
                target: count(),
                finish: Finish::Direct,
                counts: b.clone(),
            },
        ),
        Admission::Accepted
    );
    registry
}

fn incomplete_at_call(evaluation: &Evaluation) -> bool {
    matches!(
        &evaluation.outcome,
        FamilyOutcome::Evaluated(Outcome::Incomplete(denied))
            if denied.limit_kind == LimitKind::WorkUnits
                && denied.charge_point == ChargePoint::FunctionCall
    )
}

/// Trace: FR-262-AC-1
#[test]
fn a_hundred_thousand_deep_call_runs_on_a_small_stack_and_stops_at_its_fuel() {
    const DEPTH: u64 = 100_000;
    let outcome = thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let mut registry = Registry::new(Toy);
            let mut measured = meter(u64::MAX);
            let done = registry
                .begin_call(count(), vec![int(DEPTH)], &(), &mut measured, no_cancel())
                .expect("admitted call")
                .run()
                .expect("application");
            assert_eq!(completed_int(&done), Some(DEPTH));
            let spend = spent(&measured);
            assert_eq!(spend, DEPTH + 1, "one function.call per application");

            let mut short = meter(spend - 1);
            let denied = registry
                .begin_call(count(), vec![int(DEPTH)], &(), &mut short, no_cancel())
                .expect("admitted call")
                .run()
                .expect("a denied charge is an evaluation, not a failure");
            assert!(incomplete_at_call(&denied));
            assert_eq!(denied.location, Some(site()), "located at the call site");
            assert_eq!(
                spent(&short),
                spend - 1,
                "the denied charge is not recorded"
            );

            let mut exact = meter(spend);
            let again = registry
                .begin_call(count(), vec![int(DEPTH)], &(), &mut exact, no_cancel())
                .expect("admitted call")
                .run()
                .expect("application");
            assert_eq!(completed_int(&again), Some(DEPTH));
        })
        .expect("spawn")
        .join();
    assert!(outcome.is_ok(), "the deep call must not overflow the stack");
}

/// Trace: FR-262-AC-4
#[test]
fn stepping_a_mixed_chain_matches_the_synchronous_run_and_resumes_each_caller_once() {
    let n = 5;
    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);

    let mut run_meter = meter(1_000);
    let run = registry
        .begin_call(host_a(), vec![int(n)], &(), &mut run_meter, no_cancel())
        .expect("admitted call")
        .run()
        .expect("application");
    let run_charges: Vec<_> = run_meter.admitted_charges().to_vec();

    let (a2, b2) = (Counts::default(), Counts::default());
    let mut registry = chain(&a2, &b2);
    let mut step_meter = meter(1_000);
    let (stepped, steps) = drive(
        registry
            .begin_call(host_a(), vec![int(n)], &(), &mut step_meter, no_cancel())
            .expect("admitted call"),
    );
    let stepped = stepped.expect("application");

    // A saves its argument across the child and adds it to the result.
    assert_eq!(completed_int(&run), Some(2 * n));
    assert_eq!(completed_int(&stepped), Some(2 * n));
    assert_eq!(step_meter.admitted_charges(), run_charges.as_slice());
    // Two host calls and `n + 1` interpreted applications, each one charge.
    assert_eq!(run_charges.len(), 2 + usize::try_from(n).unwrap() + 1);
    assert!(run_charges
        .iter()
        .all(|c| c.point == ChargePoint::FunctionCall));
    assert!(steps > 3, "a body is suspended across several steps");
    // Each caller is resumed once to start and once for its child, and begun once.
    for counts in [&a, &b, &a2, &b2] {
        assert_eq!((counts.begun(), counts.resumed()), (1, 2));
    }
}

/// Trace: FR-262-AC-6
#[test]
fn one_unit_below_the_chain_spend_stops_at_the_denied_call_and_runs_no_more_body() {
    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);
    let mut measured = meter(1_000);
    drop(
        registry
            .begin_call(host_a(), vec![int(3)], &(), &mut measured, no_cancel())
            .expect("admitted call")
            .run()
            .expect("application"),
    );
    let w = spent(&measured);

    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);
    let mut short = meter(w - 1);
    let (denied, _) = drive(
        registry
            .begin_call(host_a(), vec![int(3)], &(), &mut short, no_cancel())
            .expect("admitted call"),
    );
    let denied = denied.expect("a denied charge is an evaluation");
    assert!(incomplete_at_call(&denied));
    assert_eq!(
        spent(&short),
        w - 1,
        "earlier spend kept, denied charge unrecorded"
    );
    // Each host was resumed to start only: neither received its child.
    assert_eq!((a.resumed(), b.resumed()), (1, 1));

    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);
    let mut fresh = meter(w);
    let ok = registry
        .begin_call(host_a(), vec![int(3)], &(), &mut fresh, no_cancel())
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&ok), Some(6));
}

/// Trace: FR-262-AC-7
#[test]
fn cancelling_after_a_request_is_saved_runs_no_further_charge_or_body() {
    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);
    let cancel = Cancel::new();
    let mut m = meter(1_000);
    let session = registry
        .begin_call(host_a(), vec![int(4)], &(), &mut m, cancel.clone())
        .expect("admitted call");
    // Step 1 enters HOST_A (its call charge); step 2 runs its first
    // transition, which saves the request for HOST_B and charges nothing.
    let session = runnable(session.step());
    let session = runnable(session.step());
    cancel.cancel(CancelCause::Requested);
    match session.step() {
        Progress::Terminal(Err(CallFailure::Cancelled(CancelCause::Requested))) => {}
        Progress::Terminal(_) | Progress::Runnable(_) => panic!("expected a cancelled terminal"),
    }
    assert_eq!(spent(&m), 1, "only the entered call was charged");
    assert_eq!((a.resumed(), b.begun()), (1, 0));
}

/// Trace: FR-262-AC-7
#[test]
fn a_charge_denied_by_cancellation_is_a_call_failure_not_an_incomplete_result() {
    let cell: Arc<OnceLock<Cancel>> = Arc::new(OnceLock::new());
    let polls = Arc::new(AtomicUsize::new(0));
    let cancel = Cancel::observing({
        let (cell, polls) = (Arc::clone(&cell), Arc::clone(&polls));
        move || {
            if polls.fetch_add(1, Ordering::SeqCst) == 2 {
                if let Some(handle) = cell.get() {
                    handle.cancel(CancelCause::Deadline);
                }
            }
        }
    });
    assert!(cell.set(cancel.clone()).is_ok());
    let mut m = meter(1_000).with_cancel(cancel.clone());
    let mut registry = Registry::new(Toy);
    let (result, _) = drive(
        registry
            .begin_call(count(), vec![int(10)], &(), &mut m, cancel)
            .expect("admitted call"),
    );
    // The third `function.call` charge was the one cancellation denied.
    assert!(matches!(
        result,
        Err(CallFailure::Cancelled(CancelCause::Deadline))
    ));
    assert_eq!(spent(&m), 2);
}

/// Trace: FR-262-AC-7
#[test]
fn a_child_undefined_outcome_propagates_through_a_direct_return_caller() {
    let counts = Counts::default();
    let mut registry = Registry::new(Toy);
    assert_eq!(
        registry.register(
            host_a(),
            &int_signature(),
            Relay {
                target: boom(),
                finish: Finish::Direct,
                counts: counts.clone(),
            },
        ),
        Admission::Accepted
    );
    let mut m = meter(1_000);
    let result = registry
        .begin_call(host_a(), vec![int(1)], &(), &mut m, no_cancel())
        .expect("admitted call")
        .run()
        .expect("application");
    assert!(matches!(
        result.outcome,
        FamilyOutcome::Evaluated(Outcome::Undefined(Undefined::DivisionByZero))
    ));
    assert_eq!(
        result.location,
        Some(site()),
        "the child's location survives"
    );
    // Start, then the child's delivery; no body work after the propagation.
    assert_eq!(counts.resumed(), 2);
}

/// Trace: FR-262-AC-7
#[test]
fn an_interpreted_frame_propagates_a_non_completed_child_without_a_value() {
    let mut registry = Registry::new(Toy);
    let mut m = meter(1_000);
    let result = registry
        .begin_expression(
            Expression::Apply {
                callee: boom(),
                argument: 0,
            },
            vec![],
            &(),
            &mut m,
            no_cancel(),
        )
        .expect("admitted expression")
        .run()
        .expect("application");
    assert!(matches!(
        result.outcome,
        FamilyOutcome::Evaluated(Outcome::Undefined(Undefined::DivisionByZero))
    ));
    assert_eq!(
        spent(&m),
        1,
        "the expression root charged nothing; its one application did"
    );
}

/// Trace: FR-262-AC-7
#[test]
fn dropping_a_suspended_session_executes_no_continuation() {
    let counts = Counts::default();
    let mut registry = Registry::new(Toy);
    assert_eq!(
        registry.register(
            host_a(),
            &int_signature(),
            Relay {
                target: count(),
                finish: Finish::Direct,
                counts: counts.clone(),
            },
        ),
        Admission::Accepted
    );
    let mut m = meter(1_000);
    let session = registry
        .begin_call(host_a(), vec![int(3)], &(), &mut m, no_cancel())
        .expect("admitted call");
    let session = runnable(session.step());
    let session = runnable(session.step());
    assert_eq!(counts.resumed(), 1);
    drop(session);
    assert_eq!(
        counts.resumed(),
        1,
        "the suspended caller was released, not resumed"
    );
    assert_eq!(spent(&m), 1);
}
