// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integer `mod` under the kernel's Euclidean law (FR-147, FR-403).
//!
//! Both evaluators call [`modulo`] here, so the operation has one definition.
//! It carries the kernel [`Outcome`] straight through.

use quire_exact::{Integer, IntegerDomain, Meter, Outcome};

/// Evaluate `mod`: always the Euclidean remainder, independent of any
/// selected `div`/`rem` law, charged only at the four `integer-modulus.*`
/// points. Only the result must lie in `domain`.
pub fn modulo(
    dividend: &Integer,
    divisor: &Integer,
    domain: &IntegerDomain,
    meter: &mut Meter,
) -> Outcome<Integer> {
    quire_exact::modulo(dividend, divisor, domain, meter)
}
