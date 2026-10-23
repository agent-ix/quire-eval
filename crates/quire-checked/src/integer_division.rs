// SPDX-License-Identifier: AGPL-3.0-or-later
//! The admitted `div`/`rem` law of a checked package (FR-400, "Checking
//! authority").

use quire_exact::DivisionProfile;

/// The admitted `div`/`rem` law of one checked package. A value of this type
/// claims the definition admission accepted the package's integer-division
/// closure, so its field is private to this crate and the only constructor is
/// `construct::admitted_integer_division`, compiled under the `construct`
/// feature.
///
/// A struct literal from another crate does not compile:
///
/// ```compile_fail
/// use quire_checked::integer_division::AdmittedIntegerDivision;
/// use quire_exact::DivisionProfile;
/// let _ = AdmittedIntegerDivision { profile: DivisionProfile::Floor };
/// ```
///
/// Reading the law does compile:
///
/// ```no_run
/// use quire_checked::integer_division::AdmittedIntegerDivision;
/// fn law(division: &AdmittedIntegerDivision) -> quire_exact::DivisionProfile {
///     division.profile()
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AdmittedIntegerDivision {
    pub(crate) profile: DivisionProfile,
}

impl AdmittedIntegerDivision {
    /// The selected law.
    pub fn profile(&self) -> DivisionProfile {
        self.profile
    }
}

/// Without the `construct` feature there is no constructor to call. This item
/// exists only in that configuration, so the doctest below never runs in the
/// `--all-features` lane, where it would compile.
///
/// ```compile_fail
/// use quire_checked::construct::admitted_integer_division;
/// use quire_exact::DivisionProfile;
/// let _ = admitted_integer_division(DivisionProfile::Floor);
/// ```
#[cfg(not(feature = "construct"))]
#[doc(hidden)]
pub struct ConstructOff;
