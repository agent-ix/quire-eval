// SPDX-License-Identifier: AGPL-3.0-or-later
//! `quire-checked`: the checked terms an evaluator reads, at ADR-011 layer C.
//!
//! The checker builds checked terms; the evaluator and the simulation read
//! them. This crate holds the terms so a reader never depends on the checker.
//! It is `no_std` with `alloc`, with a default-on `std` feature, and it
//! forbids unsafe code.
//!
//! A checked term's existence claims that the checker admitted it, so every
//! field is private and the only constructors are the functions of the
//! `construct` module, compiled only under the `construct` feature.
#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

/// The only constructors of a checked term. Enabled by the checker alone.
#[cfg(feature = "construct")]
pub mod construct {}

/// FR-400-AC-6 fixture, built only by `make mint-probe`: clippy must refuse
/// this banned identity mint. Never compiled in a normal build.
#[cfg(mint_probe)]
pub fn banned_mint() -> quire_exact::EffectiveId {
    quire_exact::EffectiveId::from_digest([0; 32])
}
