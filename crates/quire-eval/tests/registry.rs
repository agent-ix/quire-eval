// SPDX-License-Identifier: AGPL-3.0-or-later
//! Registration and the sealing boundary (FR-262, FR-401).

mod support;

use quire_eval::{Admission, CallFailure, Progress, Registry, Signature};
use quire_exact::ValueType;
use quire_semantic_value::call::InputRefusal;
use support::{
    absent, boom, completed_int, count, host_a, int, int_signature, meter, Counts, Expression,
    Finish, Relay, Toy,
};

fn relay(counts: &Counts) -> Relay {
    Relay {
        target: count(),
        finish: Finish::Direct,
        counts: counts.clone(),
    }
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn registration_decides_in_the_documented_order() {
    let mut registry = Registry::new(Toy::default());
    let counts = Counts::default();
    let wrong = Signature::new(vec![ValueType::Boolean], ValueType::Integer);

    assert_eq!(
        registry.register(absent(), &int_signature(), relay(&counts)),
        Admission::CallableNotAdmitted
    );
    assert_eq!(
        registry.register(host_a(), &wrong, relay(&counts)),
        Admission::SignatureMismatch
    );
    assert_eq!(
        registry.register(host_a(), &int_signature(), relay(&counts)),
        Admission::Accepted
    );
    // The same identity again is refused, with an equally valid factory too.
    assert_eq!(
        registry.register(host_a(), &int_signature(), relay(&counts)),
        Admission::DuplicateBinding
    );
    // Not-admitted outranks duplicate, and signature outranks duplicate.
    assert_eq!(
        registry.register(host_a(), &wrong, relay(&counts)),
        Admission::SignatureMismatch
    );
    assert_eq!(counts.begun(), 0, "registration invokes no factory");
    assert!(!registry.is_sealed());
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn a_failed_begin_leaves_the_registry_open_and_a_successful_begin_seals_it() {
    let mut registry = Registry::new(Toy::default());
    let mut m = meter(1_000);

    // Wrong arity: refused before any charge.
    let refused = registry.begin_call(count(), vec![], &(), &mut m);
    assert!(matches!(
        refused,
        Err(CallFailure::Input(InputRefusal::Arity {
            declared: 1,
            supplied: 0
        }))
    ));
    assert!(!registry.is_sealed());
    let unknown = registry.begin_call(absent(), vec![int(0)], &(), &mut m);
    assert!(matches!(
        unknown,
        Err(CallFailure::Input(InputRefusal::UnknownFunction(_)))
    ));
    assert!(!registry.is_sealed());
    let expression = registry.begin_expression(Expression::Literal(1), vec![int(0)], &(), &mut m);
    assert!(matches!(expression, Err(CallFailure::Input(_))));
    assert!(!registry.is_sealed());
    assert_eq!(spent(&m), 0, "a rejected root input charges nothing");
    assert_eq!(
        registry.register(host_a(), &int_signature(), relay(&Counts::default())),
        Admission::Accepted,
        "registration is still open after the failed Begin"
    );

    let session = registry
        .begin_call(count(), vec![int(0)], &(), &mut m)
        .expect("admitted call");
    assert!(registry.is_sealed());
    drop(session);
    assert!(registry.is_sealed(), "dropping a session never reopens it");
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn a_sealed_registry_keeps_its_bindings_for_every_session() {
    let mut registry = Registry::new(Toy::default());
    let first = Counts::default();
    let second = Counts::default();
    assert_eq!(
        registry.register(host_a(), &int_signature(), relay(&first)),
        Admission::Accepted
    );
    let mut m = meter(1_000);
    let ran = registry
        .begin_call(host_a(), vec![int(3)], &(), &mut m)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&ran), Some(3));

    // Sealed: even a duplicate reports the seal first, and so does a fresh identity.
    assert_eq!(
        registry.register(host_a(), &int_signature(), relay(&second)),
        Admission::RegistrySealed
    );
    assert_eq!(
        registry.register(boom(), &int_signature(), relay(&second)),
        Admission::RegistrySealed
    );
    assert_eq!(
        registry.register(absent(), &int_signature(), relay(&second)),
        Admission::RegistrySealed
    );

    // A later session still runs the first factory's body, not the refused one.
    let mut again = meter(1_000);
    let later = registry
        .begin_call(host_a(), vec![int(2)], &(), &mut again)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&later), Some(2));
    assert_eq!((first.begun(), second.begun()), (2, 0));
}

/// Toy-closure test of the scheduler behaviour its name states.
#[test]
fn a_literal_expression_charges_no_call_and_a_step_is_not_a_charge() {
    let mut registry = Registry::new(Toy::default());
    let mut m = meter(0);
    let mut session = registry
        .begin_expression(Expression::Literal(7), vec![], &(), &mut m)
        .expect("admitted expression");
    let mut steps = 0;
    let result = loop {
        steps += 1;
        match session.step() {
            Progress::Runnable(next) => session = next,
            Progress::Terminal(result) => break result,
        }
    };
    assert!(
        steps > 1,
        "the root is entered, then run: more than one step"
    );
    assert_eq!(completed_int(&result.expect("completes")), Some(7));
    // With zero fuel, only a charged application could have been denied.
    assert_eq!(spent(&m), 0);
}

fn spent(meter: &quire_exact::Meter) -> u64 {
    support::spent(meter)
}

/// A cancelled handle on the meter ends `Begin` with `Cancelled` before any
/// charge, and leaves the registry open (FR-262: the supplied-value run polls
/// the meter's handle).
#[test]
fn begin_observes_the_meters_cancelled_handle_before_any_charge() {
    let mut registry = Registry::new(Toy::default());
    let cancel = quire_exact::Cancel::new();
    cancel.cancel(quire_exact::CancelCause::Deadline);
    let mut m = meter(1_000).with_cancel(cancel);
    let refused = registry.begin_call(count(), vec![int(1)], &(), &mut m);
    assert!(matches!(
        refused,
        Err(CallFailure::Cancelled(quire_exact::CancelCause::Deadline))
    ));
    assert!(!registry.is_sealed());
    assert_eq!(spent(&m), 0);
}
