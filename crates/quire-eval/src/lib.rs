// SPDX-License-Identifier: AGPL-3.0-or-later
//! `quire-eval`: the shared evaluation-core interface, at ADR-011 layer E.
//!
//! An evaluator implementation supplies the contract for one checked closure
//! ([`CheckedClosure`]); this crate owns everything around it: the registry
//! of resumable host bodies ([`Registry`]), the opaque [`Session`] that
//! begins an application and steps it one finite transition at a time, and
//! the evaluation result vocabulary ([`Evaluation`], [`CallFailure`],
//! [`FamilyOutcome`]).
//!
//! # One scheduler, two kinds of body
//!
//! Interpreted checked code and registered host bodies run on one explicit
//! heap task stack, so no evaluation step recurses on the native stack in
//! proportion to call depth, and there is no depth ceiling: the only bound is
//! the `work_units` fuel of the one [`quire_exact::Meter`] (QSpec FR-146). A
//! body that applies a callee never calls an evaluator. It returns a
//! [`ChildRequest`] and is suspended; the scheduler validates the request,
//! charges `function.call`, runs the callee and delivers its [`Evaluation`]
//! to the suspended body exactly once.
//!
//! # Checking authority
//!
//! A registry binds a factory to an exact callable identity and signature
//! already admitted in its checked closure. Registration mints no checked
//! authority and resolves no name: the leaf compares [`quire_exact::NodeKey`]s.
//!
//! The crate is `no_std` with `alloc`, has no `std` feature, and forbids
//! unsafe code. It names no item of the QSL compiler, package, model, routing
//! or replay crates.
#![no_std]

extern crate alloc;

mod body;
mod cause;
mod closure;
mod integer;
mod registry;
mod result;
mod session;
mod signature;

pub use body::{Body, BodyFactory, BodyFailure, ChildRequest, Resume, Transition};
pub use cause::{
    PreconditionFailure, ProtocolReferenceUnsupported, ReferenceUnresolved, StateModelUndefined,
};
pub use closure::{CheckedClosure, RequestRefusal};
pub use integer::modulo;
pub use registry::{Admission, Registry};
pub use result::{
    CallAdmissionFailure, CallAdmissionLocus, CallAdmissionRecord, CallFailure, EvalOutcome,
    Evaluation, FamilyOutcome, FamilyResult,
};
pub use session::{Progress, Session};
pub use signature::Signature;
