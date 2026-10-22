// SPDX-License-Identifier: AGPL-3.0-or-later
//! The session and its scheduler: one explicit heap task stack driving
//! interpreted frames and registered host bodies through one transition at a
//! time.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem;

use quire_exact::{Cancel, Charge, ChargePoint, Incomplete, Meter, NodeKey, Outcome, Value};
use quire_semantic_value::catalog::InternalFault;
use quire_semantic_value::location::Location;

use crate::body::{Body, ChildRequest, Resume, Transition};
use crate::closure::CheckedClosure;
use crate::registry::Bindings;
use crate::result::{CallFailure, Evaluation, FamilyOutcome};

/// What the scheduler enters next, before any frame of it exists.
pub(crate) enum Entry<C: CheckedClosure> {
    /// Apply `callable` to `arguments`. `site` is the call site of a body's
    /// request and `None` for the root; `admitted` is `true` when `Begin`
    /// already validated it.
    Call {
        callable: NodeKey,
        arguments: Vec<Value>,
        site: Option<Location>,
        admitted: bool,
    },
    /// Evaluate a checked expression root.
    Expression {
        expression: C::Expression,
        parameters: Vec<Value>,
    },
}

enum Engine<C: CheckedClosure> {
    Interpreted(C::Task),
    Host(Box<dyn Body>),
}

/// One suspended or running body, and what it is resumed with next.
struct Frame<C: CheckedClosure> {
    engine: Engine<C>,
    input: Resume,
}

/// The result of one [`Session::step`].
#[must_use = "dropping a runnable session abandons the application"]
pub enum Progress<'a, C: CheckedClosure> {
    /// The application is still runnable: step the session again.
    Runnable(Session<'a, C>),
    /// The application ended, with the evaluation or the call failure. The
    /// session is gone; suspended frames were released without executing
    /// their continuations.
    Terminal(Result<Evaluation, CallFailure>),
}

/// The opaque, live session of one application.
///
/// A session retains its task stack, call frames, lexical bindings, source
/// locations, losses and accounting across steps. It is bound for life to the
/// closure and sealed registry it began from, and to its environment, meter
/// and cancellation handle. It cannot be copied or rebound, and
/// [`Self::step`] consumes it, so a session cannot be resumed after a
/// terminal result.
///
/// Dropping a session releases its suspended frames and executes no
/// continuation.
pub struct Session<'a, C: CheckedClosure> {
    closure: Arc<C>,
    bindings: Arc<Bindings>,
    environment: &'a C::Environment,
    meter: &'a mut Meter,
    cancel: Cancel,
    /// What to enter at the next transition, when set.
    entry: Option<Entry<C>>,
    /// The task stack: the callee is on top, its suspended callers below.
    frames: Vec<Frame<C>>,
}

impl<'a, C: CheckedClosure> Session<'a, C> {
    pub(crate) fn new(
        closure: Arc<C>,
        bindings: Arc<Bindings>,
        entry: Entry<C>,
        environment: &'a C::Environment,
        meter: &'a mut Meter,
        cancel: Cancel,
    ) -> Self {
        Self {
            closure,
            bindings,
            environment,
            meter,
            cancel,
            entry: Some(entry),
            frames: Vec::new(),
        }
    }

    /// Advance the application by one finite saved transition.
    ///
    /// Before the transition the session polls its cancellation handle. A
    /// transition commits its state before returning. Yielding, saving a
    /// continuation, pushing or popping a frame and delivering a child add
    /// no charge; the only charges are the call charge of an entered
    /// application and the kernel charges its body makes.
    pub fn step(mut self) -> Progress<'a, C> {
        match self.transition() {
            Ok(None) => Progress::Runnable(self),
            Ok(Some(evaluation)) => Progress::Terminal(Ok(evaluation)),
            Err(failure) => Progress::Terminal(Err(failure)),
        }
    }

    /// Drive the application to its terminal result, as the synchronous
    /// `call` and `evaluate` do.
    ///
    /// # Errors
    ///
    /// The call failure that ended the application.
    pub fn run(self) -> Result<Evaluation, CallFailure> {
        let mut session = self;
        loop {
            match session.step() {
                Progress::Runnable(next) => session = next,
                Progress::Terminal(result) => return result,
            }
        }
    }

    fn transition(&mut self) -> Result<Option<Evaluation>, CallFailure> {
        if let Some(cause) = self.cancel.cause() {
            return Err(CallFailure::Cancelled(cause));
        }
        if let Some(entry) = self.entry.take() {
            return self.enter(entry);
        }
        let Some(frame) = self.frames.last_mut() else {
            return Err(fault("session-without-frame"));
        };
        let input = mem::replace(&mut frame.input, Resume::Again);
        let transition = match &mut frame.engine {
            Engine::Interpreted(task) => {
                let delivered = match input {
                    Resume::Child(evaluation) => Some(evaluation),
                    Resume::Again => None,
                    Resume::Start(_) => return Err(fault("interpreted-frame-resumed-with-start")),
                };
                self.closure
                    .advance(task, delivered, self.environment, self.meter)?
            }
            Engine::Host(body) => body.resume(input, self.meter)?,
        };
        match transition {
            Transition::Continue => {}
            Transition::Call(ChildRequest {
                callee,
                arguments,
                site,
            }) => {
                // The request is saved; the next transition validates,
                // charges and pushes the callee, after polling cancellation.
                self.entry = Some(Entry::Call {
                    callable: callee,
                    arguments,
                    site: Some(site),
                    admitted: false,
                });
            }
            Transition::Done(evaluation) => {
                self.frames.pop();
                match self.frames.last_mut() {
                    Some(caller) => caller.input = Resume::Child(evaluation),
                    None => return Ok(Some(evaluation)),
                }
            }
        }
        Ok(None)
    }

    /// Enter an application: validate, charge `function.call`, bind and push
    /// its frame, in that order. An expression root has no call charge.
    fn enter(&mut self, entry: Entry<C>) -> Result<Option<Evaluation>, CallFailure> {
        let frame = match entry {
            Entry::Call {
                callable,
                arguments,
                site,
                admitted,
            } => {
                if !admitted {
                    self.closure
                        .admit_call(&callable, &arguments, self.environment)?;
                }
                if let Err(denied) = self.meter.charge(Charge::new(ChargePoint::FunctionCall)) {
                    return self.denied(denied, site);
                }
                match self.bindings.get(&callable) {
                    Some(factory) => Frame {
                        engine: Engine::Host(factory.begin()),
                        input: Resume::Start(arguments),
                    },
                    None => Frame {
                        engine: Engine::Interpreted(self.closure.bind_call(
                            &callable,
                            arguments,
                            self.environment,
                        )?),
                        input: Resume::Again,
                    },
                }
            }
            Entry::Expression {
                expression,
                parameters,
            } => Frame {
                engine: Engine::Interpreted(self.closure.bind_expression(
                    &expression,
                    parameters,
                    self.environment,
                )?),
                input: Resume::Again,
            },
        };
        self.frames.push(frame);
        Ok(None)
    }

    /// A denied charge ends the application: a cancellation the meter
    /// observed is a call failure; any other denial is the located incomplete
    /// result at that charge, with earlier spend retained and the denied
    /// charge unrecorded.
    fn denied(
        &self,
        denied: Incomplete,
        site: Option<Location>,
    ) -> Result<Option<Evaluation>, CallFailure> {
        if let Some(cause) = self.cancel.tripped() {
            return Err(CallFailure::Cancelled(cause));
        }
        Ok(Some(Evaluation {
            outcome: FamilyOutcome::Evaluated(Outcome::Incomplete(denied)),
            location: site,
            losses: Vec::new(),
        }))
    }
}

/// A broken scheduler invariant, never a caller-input refusal.
fn fault(invariant: &'static str) -> CallFailure {
    CallFailure::Fault(InternalFault::new("eval", invariant))
}
