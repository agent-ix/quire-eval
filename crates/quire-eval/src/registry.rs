// SPDX-License-Identifier: AGPL-3.0-or-later
//! The host-body registry of one checked closure, and `Begin`.

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem;

use quire_exact::{Meter, NodeKey, Value};

use crate::body::BodyFactory;
use crate::closure::CheckedClosure;
use crate::result::CallFailure;
use crate::session::{Entry, Session};
use crate::signature::Signature;

/// The bindings a sealed registry holds, shared by every session that uses it.
pub(crate) type Bindings = BTreeMap<NodeKey, Arc<dyn BodyFactory>>;

/// The typed result of registering a host body. These are in-process results,
/// not runtime refusal codes, kernel outcomes or wire causes.
///
/// Registration decides them in this order: a sealed registry is
/// [`Self::RegistrySealed`]; otherwise an identity absent from the closure is
/// [`Self::CallableNotAdmitted`]; otherwise a signature unequal to the
/// admitted one is [`Self::SignatureMismatch`]; otherwise an identity already
/// bound is [`Self::DuplicateBinding`]; otherwise the binding is stored and
/// the result is [`Self::Accepted`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use = "a refused registration binds nothing"]
pub enum Admission {
    /// The factory is bound to the callable.
    Accepted,
    /// A successful `Begin` already sealed the registry.
    RegistrySealed,
    /// The identity is not a callable of the registry's checked closure.
    CallableNotAdmitted,
    /// The signature differs from the callable's admitted signature.
    SignatureMismatch,
    /// The identity already has a binding. Registration is neither
    /// replacement nor idempotent acceptance, even for the same factory.
    DuplicateBinding,
}

enum State {
    Open(Bindings),
    Sealed(Arc<Bindings>),
}

/// The host-body registry of exactly one checked closure.
///
/// The registry is open for registration and is permanently sealed by its
/// first successful `Begin` ([`Self::begin_call`], [`Self::begin_expression`]).
/// A failed `Begin` leaves an open registry open. A session binds the sealed
/// registry; ending or dropping a session never reopens it, and later
/// sessions use the same bindings. Selecting different bodies needs a fresh
/// registry. Registration and `Begin` take `&mut self`, so a registration
/// cannot race the sealing boundary.
///
/// Registration neither invokes a factory nor takes part in any application's
/// meter, and it cannot create checked authority: it binds a body to an
/// identity the closure already admitted.
pub struct Registry<C> {
    closure: Arc<C>,
    state: State,
}

impl<C: CheckedClosure> Registry<C> {
    /// An open registry for `closure`, with no bindings.
    pub fn new(closure: C) -> Self {
        Self {
            closure: Arc::new(closure),
            state: State::Open(Bindings::new()),
        }
    }

    /// The checked closure this registry holds.
    #[must_use]
    pub fn closure(&self) -> &C {
        &self.closure
    }

    /// Whether a successful `Begin` has sealed this registry.
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        matches!(self.state, State::Sealed(_))
    }

    /// Bind `factory` to `callable`, which must be admitted in this
    /// registry's closure with exactly `signature`.
    ///
    /// A refused registration leaves every binding unchanged and never
    /// invokes the factory.
    pub fn register(
        &mut self,
        callable: NodeKey,
        signature: &Signature,
        factory: impl BodyFactory + 'static,
    ) -> Admission {
        let State::Open(bindings) = &mut self.state else {
            return Admission::RegistrySealed;
        };
        match self.closure.signature(&callable) {
            None => Admission::CallableNotAdmitted,
            Some(admitted) if admitted != *signature => Admission::SignatureMismatch,
            Some(_) if bindings.contains_key(&callable) => Admission::DuplicateBinding,
            Some(_) => {
                bindings.insert(callable, Arc::new(factory));
                Admission::Accepted
            }
        }
    }

    /// Begin the application of `callable` to the completed `arguments`.
    ///
    /// Admission runs first and charges nothing. On success the registry is
    /// sealed before any factory invocation or charge, and the returned
    /// session is bound to this closure, `environment` and `meter`. The
    /// cancellation handle the meter carries, if any, is the session's: a meter without one
    /// cannot be cancelled.
    /// The call's `function.call` charge is the session's first transition.
    ///
    /// # Errors
    ///
    /// The input failure of admission, before any charge. The registry stays
    /// as it was.
    pub fn begin_call<'a>(
        &mut self,
        callable: NodeKey,
        arguments: Vec<Value>,
        environment: &'a C::Environment,
        meter: &'a mut Meter,
    ) -> Result<Session<'a, C>, CallFailure> {
        self.closure
            .admit_call(&callable, &arguments, environment, meter)?;
        let entry = Entry::Root {
            callable,
            arguments,
        };
        Ok(self.session(entry, environment, meter))
    }

    /// Begin the evaluation of `expression` with the completed `parameters`.
    ///
    /// An expression root incurs no call charge unless it reaches an
    /// application. Otherwise as [`Self::begin_call`].
    ///
    /// # Errors
    ///
    /// As [`Self::begin_call`].
    pub fn begin_expression<'a>(
        &mut self,
        expression: C::Expression,
        parameters: Vec<Value>,
        environment: &'a C::Environment,
        meter: &'a mut Meter,
    ) -> Result<Session<'a, C>, CallFailure> {
        self.closure
            .admit_expression(&expression, &parameters, environment, meter)?;
        let entry = Entry::Expression {
            expression,
            parameters,
        };
        Ok(self.session(entry, environment, meter))
    }

    fn session<'a>(
        &mut self,
        entry: Entry<C>,
        environment: &'a C::Environment,
        meter: &'a mut Meter,
    ) -> Session<'a, C> {
        let bindings = self.seal();
        Session::new(
            Arc::clone(&self.closure),
            bindings,
            entry,
            environment,
            meter,
        )
    }

    /// Seal the registry (a no-op once sealed) and share its bindings.
    fn seal(&mut self) -> Arc<Bindings> {
        match &mut self.state {
            State::Sealed(bindings) => Arc::clone(bindings),
            State::Open(open) => {
                let sealed = Arc::new(mem::take(open));
                self.state = State::Sealed(Arc::clone(&sealed));
                sealed
            }
        }
    }
}
