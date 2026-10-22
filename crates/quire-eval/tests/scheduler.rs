// SPDX-License-Identifier: AGPL-3.0-or-later
//! The scheduler: depth, fuel, handoff, cancellation and release (FR-262).

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;

use quire_eval::{Admission, CallFailure, Evaluation, FamilyOutcome, Progress, Registry, Session};
use quire_exact::{Cancel, CancelCause, ChargePoint, LimitKind, Outcome, Undefined};
use support::{
    add, boom, completed_int, count, count_site, host_a, host_b, int, int_signature, lossy, meter,
    population, ranged, refuse, site, spent, storage, BadRequest, ChargingRelay, Counts,
    Expression, Finish, Relay, Toy, Yielding,
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
    let mut registry = Registry::new(Toy::default());
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

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn a_hundred_thousand_deep_call_runs_on_a_small_stack_and_stops_at_its_fuel() {
    const DEPTH: u64 = 100_000;
    let outcome = thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let mut registry = Registry::new(Toy::default());
            let mut measured = meter(u64::MAX);
            let done = registry
                .begin_call(count(), vec![int(DEPTH)], &(), &mut measured)
                .expect("admitted call")
                .run()
                .expect("application");
            assert_eq!(completed_int(&done), Some(DEPTH));
            let spend = spent(&measured);
            assert_eq!(spend, DEPTH + 1, "one function.call per application");

            let mut short = meter(spend - 1);
            let denied = registry
                .begin_call(count(), vec![int(DEPTH)], &(), &mut short)
                .expect("admitted call")
                .run()
                .expect("a denied charge is an evaluation, not a failure");
            assert!(incomplete_at_call(&denied));
            assert_eq!(
                denied.location,
                Some(count_site(0)),
                "the denied call is the deepest request, applying COUNT to 0"
            );
            assert_eq!(
                spent(&short),
                spend - 1,
                "the denied charge is not recorded"
            );

            let mut exact = meter(spend);
            let again = registry
                .begin_call(count(), vec![int(DEPTH)], &(), &mut exact)
                .expect("admitted call")
                .run()
                .expect("application");
            assert_eq!(completed_int(&again), Some(DEPTH));
        })
        .expect("spawn")
        .join();
    // A native stack overflow aborts the whole test process, so reaching here
    // already shows the call did not overflow; this checks the worker did not panic.
    assert!(outcome.is_ok(), "the worker thread panicked");
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn stepping_a_mixed_chain_matches_the_synchronous_run_and_resumes_each_caller_once() {
    let n = 5;
    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);

    let mut run_meter = meter(1_000);
    let run = registry
        .begin_call(host_a(), vec![int(n)], &(), &mut run_meter)
        .expect("admitted call")
        .run()
        .expect("application");
    let run_charges: Vec<_> = run_meter.admitted_charges().to_vec();

    let (a2, b2) = (Counts::default(), Counts::default());
    let mut registry = chain(&a2, &b2);
    let mut step_meter = meter(1_000);
    let (stepped, steps) = drive(
        registry
            .begin_call(host_a(), vec![int(n)], &(), &mut step_meter)
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

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn one_unit_below_the_chain_spend_stops_at_the_denied_call_and_runs_no_more_body() {
    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);
    let mut measured = meter(1_000);
    drop(
        registry
            .begin_call(host_a(), vec![int(3)], &(), &mut measured)
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
            .begin_call(host_a(), vec![int(3)], &(), &mut short)
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
        .begin_call(host_a(), vec![int(3)], &(), &mut fresh)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&ok), Some(6));
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn cancelling_after_a_request_is_saved_runs_no_further_charge_or_body() {
    let (a, b) = (Counts::default(), Counts::default());
    let mut registry = chain(&a, &b);
    let cancel = Cancel::new();
    let mut m = meter(1_000).with_cancel(cancel.clone());
    let session = registry
        .begin_call(host_a(), vec![int(4)], &(), &mut m)
        .expect("admitted call");
    // Step 1 enters HOST_A (its call charge); step 2 runs its first
    // transition, which saves the request for HOST_B and charges nothing.
    let session = runnable(session.step());
    let session = runnable(session.step());
    cancel.cancel(CancelCause::Requested);
    let Progress::Terminal(Ok(denied)) = session.step() else {
        panic!("expected the located incomplete result at the child's call charge");
    };
    // FR-262-AC-7: the denial is the result, at the request's call site.
    let FamilyOutcome::Evaluated(Outcome::Incomplete(record)) = &denied.outcome else {
        panic!("expected an incomplete outcome, got {:?}", denied.outcome);
    };
    assert_eq!(record.limit_kind, LimitKind::WorkUnits);
    assert_eq!(record.charge_point, ChargePoint::FunctionCall);
    assert_eq!(record.limit, record.consumed);
    assert_eq!(denied.location, Some(site()));
    assert_eq!(cancel.tripped(), Some(CancelCause::Requested));
    assert_eq!(spent(&m), 1, "only the entered call was charged");
    assert_eq!((a.resumed(), b.begun()), (1, 0));
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn a_charge_denied_by_cancellation_is_an_incomplete_result_not_a_call_failure() {
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
    let mut registry = Registry::new(Toy::default());
    let (result, _) = drive(
        registry
            .begin_call(count(), vec![int(10)], &(), &mut m)
            .expect("admitted call"),
    );
    // The third `function.call` charge was the one cancellation denied; it is
    // a denied charge like any other, and the handle reports the cause.
    let denied = result.expect("a denied charge is an evaluation");
    assert!(incomplete_at_call(&denied));
    assert_eq!(cancel.tripped(), Some(CancelCause::Deadline));
    assert_eq!(spent(&m), 2);
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn a_child_undefined_outcome_propagates_through_a_direct_return_caller() {
    let counts = Counts::default();
    let mut registry = Registry::new(Toy::default());
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
        .begin_call(host_a(), vec![int(1)], &(), &mut m)
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

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn an_interpreted_frame_propagates_a_non_completed_child_without_a_value() {
    let mut registry = Registry::new(Toy::default());
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

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn dropping_a_suspended_session_executes_no_continuation() {
    let counts = Counts::default();
    let mut registry = Registry::new(Toy::default());
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
        .begin_call(host_a(), vec![int(3)], &(), &mut m)
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

fn register(
    registry: &mut Registry<Toy>,
    callable: quire_exact::NodeKey,
    factory: impl quire_eval::BodyFactory + 'static,
) {
    assert_eq!(
        registry.register(callable, &int_signature(), factory),
        Admission::Accepted
    );
}

/// Toy-closure test: a body charges its own kernel
/// work, saves a local, requests a child and continues with the child's value,
/// keeping the child's loss record.
#[test]
fn a_charging_body_runs_each_transition_once_and_keeps_the_childs_losses() {
    let counts = Counts::default();
    let toy = Toy::default();
    let binds = toy.binds();
    let mut registry = Registry::new(toy);
    register(
        &mut registry,
        host_a(),
        ChargingRelay {
            target: lossy(),
            counts: counts.clone(),
        },
    );
    let mut m = meter(1_000);
    let (result, _) = drive(
        registry
            .begin_call(host_a(), vec![int(5)], &(), &mut m)
            .expect("admitted call"),
    );
    let result = result.expect("application");

    assert_eq!(
        completed_int(&result),
        Some(1 + 5),
        "child value plus the saved local"
    );
    assert_eq!(
        result.losses,
        vec![support::loss()],
        "the loss record survives"
    );
    // One start and one child delivery: each transition and child exactly once.
    assert_eq!((counts.begun(), counts.resumed()), (1, 2));
    assert_eq!(
        binds.lock().expect("bind log").len(),
        1,
        "LOSSY entered once"
    );
    let points: Vec<_> = m.admitted_charges().iter().map(|c| c.point).collect();
    assert_eq!(
        points,
        [
            ChargePoint::FunctionCall,
            ChargePoint::IntegerArithmeticArithmetic,
            ChargePoint::FunctionCall,
            ChargePoint::IntegerArithmeticArithmetic,
        ]
    );
}

/// A cancellation a body's own kernel charge observes in the root body's last
/// transition ends the session with that charge's located incomplete result,
/// as one the scheduler's `function.call` charge sees does (FR-262-AC-10).
#[test]
fn cancellation_observed_by_the_root_bodys_last_charge_is_an_incomplete_result() {
    let cell: Arc<OnceLock<Cancel>> = Arc::new(OnceLock::new());
    let polls = Arc::new(AtomicUsize::new(0));
    let cancel = Cancel::observing({
        let (cell, polls) = (Arc::clone(&cell), Arc::clone(&polls));
        move || {
            // Polls: 0 root call, 1 body start work, 2 COUNT call, 3 the body's
            // work after its child: its last transition.
            if polls.fetch_add(1, Ordering::SeqCst) == 3 {
                if let Some(handle) = cell.get() {
                    handle.cancel(CancelCause::Requested);
                }
            }
        }
    });
    assert!(cell.set(cancel.clone()).is_ok());
    let mut registry = Registry::new(Toy::default());
    register(
        &mut registry,
        host_a(),
        ChargingRelay {
            target: count(),
            counts: Counts::default(),
        },
    );
    let mut m = meter(1_000).with_cancel(cancel.clone());
    let (result, _) = drive(
        registry
            .begin_call(host_a(), vec![int(0)], &(), &mut m)
            .expect("admitted call"),
    );
    let denied = result.expect("a denied charge is an evaluation");
    let FamilyOutcome::Evaluated(Outcome::Incomplete(record)) = &denied.outcome else {
        panic!("expected an incomplete outcome, got {:?}", denied.outcome);
    };
    assert_eq!(record.limit_kind, LimitKind::WorkUnits);
    assert_eq!(
        record.charge_point,
        ChargePoint::IntegerArithmeticArithmetic
    );
    assert_eq!(record.limit, record.consumed);
    assert_eq!(cancel.tripped(), Some(CancelCause::Requested));
}

/// A request naming an admitted callable with the wrong argument count is as
/// broken as one naming an absent callable: an internal fault of stage
/// `quire-eval`, with no charge for the child.
#[test]
fn a_body_requesting_a_callee_with_the_wrong_arity_is_an_internal_fault() {
    let mut registry = Registry::new(Toy::default());
    register(
        &mut registry,
        host_a(),
        BadRequest {
            callee: add(),
            arguments: vec![int(1)],
        },
    );
    let mut m = meter(1_000);
    let result = registry
        .begin_call(host_a(), vec![int(1)], &(), &mut m)
        .expect("the caller's own input is admitted")
        .run();
    match result {
        Err(CallFailure::Fault(fault)) => {
            assert_eq!(
                (fault.stage(), fault.invariant()),
                ("quire-eval", "child-request-not-admitted")
            );
        }
        other => panic!("expected an internal fault, got {other:?}"),
    }
    assert_eq!(spent(&m), 1, "only the root call was charged");
}

/// Toy-closure test: in `add(count(2), count(3))` the
/// argument applications are entered left to right, each before `add`'s own
/// call.
#[test]
fn argument_applications_run_left_to_right_before_the_outer_call() {
    let toy = Toy::default();
    let binds = toy.binds();
    let mut registry = Registry::new(toy);
    let mut m = meter(1_000);
    let result = registry
        .begin_expression(
            Expression::Nested {
                outer: add(),
                left: (count(), 2),
                right: (count(), 3),
            },
            vec![],
            &(),
            &mut m,
        )
        .expect("admitted expression")
        .run()
        .expect("application");
    assert_eq!(completed_int(&result), Some(5));
    let entered: Vec<_> = binds.lock().expect("bind log").clone();
    let left_chain = [2, 1, 0].map(|n| (count(), vec![n]));
    let right_chain = [3, 2, 1, 0].map(|n| (count(), vec![n]));
    let expected: Vec<_> = left_chain
        .into_iter()
        .chain(right_chain)
        .chain([(add(), vec![2, 3])])
        .collect();
    assert_eq!(entered, expected);
    // The expression root charged nothing itself: one call per entered body.
    assert_eq!(spent(&m), u64::try_from(expected.len()).unwrap());
}

/// Toy-closure test: a child refusal propagates through
/// a direct-return caller with its location, and a child's loss record
/// survives the hand-off.
#[test]
fn a_child_refusal_and_a_child_loss_survive_a_direct_return_caller() {
    for (target, finish) in [(refuse(), Finish::Direct), (lossy(), Finish::Direct)] {
        let counts = Counts::default();
        let mut registry = Registry::new(Toy::default());
        register(
            &mut registry,
            host_a(),
            Relay {
                target,
                finish,
                counts: counts.clone(),
            },
        );
        let mut m = meter(1_000);
        let result = registry
            .begin_call(host_a(), vec![int(1)], &(), &mut m)
            .expect("admitted call")
            .run()
            .expect("application");
        assert_eq!(counts.resumed(), 2, "start and the child's delivery only");
        if target == refuse() {
            assert!(matches!(
                result.outcome,
                FamilyOutcome::Evaluated(Outcome::Refused(_))
            ));
            assert_eq!(result.location, Some(site()));
        } else {
            assert_eq!(completed_int(&result), Some(1));
            assert_eq!(result.losses, vec![support::loss()]);
        }
    }
}

/// A body that requests a callee the closure does not admit is broken: the
/// session ends as an internal fault, not a caller-input refusal.
#[test]
fn a_body_requesting_a_callee_that_is_not_admitted_is_an_internal_fault() {
    let mut registry = Registry::new(Toy::default());
    register(
        &mut registry,
        host_a(),
        Relay {
            target: support::absent(),
            finish: Finish::Direct,
            counts: Counts::default(),
        },
    );
    let mut m = meter(1_000);
    let result = registry
        .begin_call(host_a(), vec![int(1)], &(), &mut m)
        .expect("the caller's own input is admitted")
        .run();
    match result {
        Err(CallFailure::Fault(fault)) => {
            assert_eq!(
                (fault.stage(), fault.invariant()),
                ("quire-eval", "child-request-not-admitted")
            );
        }
        other => panic!("expected an internal fault, got {other:?}"),
    }
    assert_eq!(spent(&m), 1, "only the root call was charged");
}

fn run_bad_request(
    callee: quire_exact::NodeKey,
    arguments: &[quire_exact::Value],
) -> (Result<Evaluation, CallFailure>, u64, usize) {
    let counts = Counts::default();
    let mut registry = Registry::new(Toy::default());
    register(
        &mut registry,
        host_a(),
        Relay {
            target: callee,
            finish: Finish::Direct,
            counts: counts.clone(),
        },
    );
    // The relay forwards the one argument it is begun with to `callee`.
    let mut m = meter(1_000);
    let result = registry
        .begin_call(
            host_a(),
            vec![arguments.first().cloned().unwrap_or_else(|| int(0))],
            &(),
            &mut m,
        )
        .expect("the caller's own input is admitted")
        .run();
    (result, spent(&m), counts.resumed())
}

/// FR-262-AC-11 analogue: five invalid requests each end the session as an
/// internal fault with no child charge, no child body and the requesting body
/// not resumed; the sixth, valid, request is applied with one call charge.
#[test]
fn invalid_child_requests_fault_and_a_valid_one_is_applied() {
    let invalid: [(&str, quire_exact::NodeKey, Vec<quire_exact::Value>); 4] = [
        ("absent callee", support::absent(), vec![int(1)]),
        ("argument out of range", ranged(), vec![int(10)]),
        (
            "population of the wrong maximum",
            population(),
            vec![int(4)],
        ),
        (
            "population the environment does not resolve",
            population(),
            vec![int(99)],
        ),
    ];
    for (what, callee, arguments) in invalid {
        let (result, spend, resumed) = run_bad_request(callee, &arguments);
        match result {
            Err(CallFailure::Fault(fault)) => assert_eq!(
                (fault.stage(), fault.invariant()),
                ("quire-eval", "child-request-not-admitted"),
                "{what}"
            ),
            other => panic!("{what}: expected an internal fault, got {other:?}"),
        }
        assert_eq!(spend, 1, "{what}: only the root call was charged");
        assert_eq!(resumed, 1, "{what}: the requesting body is not resumed");
    }
    // Wrong argument count.
    let mut registry = Registry::new(Toy::default());
    register(
        &mut registry,
        host_a(),
        BadRequest {
            callee: ranged(),
            arguments: vec![int(1), int(2)],
        },
    );
    let mut m = meter(1_000);
    let result = registry
        .begin_call(host_a(), vec![int(1)], &(), &mut m)
        .expect("admitted")
        .run();
    assert!(matches!(result, Err(CallFailure::Fault(_))));
    // The valid request: the ranged callee at its bound and the population
    // analogue at the declared maximum are applied, one call charge each.
    for (callee, argument) in [(ranged(), 9), (population(), 3)] {
        let (result, spend, resumed) = run_bad_request(callee, &[int(argument)]);
        assert_eq!(completed_int(&result.expect("applied")), Some(argument));
        assert_eq!(spend, 2);
        assert_eq!(resumed, 2);
    }
}

/// A membership check that cannot decide for want of storage ends the session
/// as `CallFailure::Storage` with the callee's parameter index, before the
/// child's call charge (FR-262-AC-11).
#[test]
fn an_undecidable_membership_ends_the_session_as_a_storage_failure() {
    let (result, spend, resumed) = run_bad_request(storage(), &[int(1)]);
    match result {
        Err(CallFailure::Storage { parameter: 0, .. }) => {}
        other => panic!("expected CallFailure::Storage at parameter 0, got {other:?}"),
    }
    assert_eq!((spend, resumed), (1, 1));
}

/// FR-262-AC-10, third clause: a cancellation between two steps does not stop
/// a transition that makes no charge; the session ends at the first charge
/// after it.
#[test]
fn a_chargeless_transition_runs_after_a_cancellation_and_the_next_charge_ends_the_session() {
    let mut registry = Registry::new(Toy::default());
    register(&mut registry, host_a(), Yielding { target: count() });
    let cancel = Cancel::new();
    let mut m = meter(1_000).with_cancel(cancel.clone());
    let session = registry
        .begin_call(host_a(), vec![int(2)], &(), &mut m)
        .expect("admitted call");
    // Enter the host (its call charge), then its first transition (a yield).
    let session = runnable(session.step());
    let session = runnable(session.step());
    cancel.cancel(CancelCause::Deadline);
    // The next transition saves the child request and charges nothing: it runs.
    let session = runnable(session.step());
    // The child's call charge is the first charge after the cancellation.
    let Progress::Terminal(Ok(denied)) = session.step() else {
        panic!("expected the incomplete result at the child's call charge");
    };
    assert!(incomplete_at_call(&denied));
    assert_eq!(cancel.tripped(), Some(CancelCause::Deadline));
    assert_eq!(spent(&m), 1);
}
