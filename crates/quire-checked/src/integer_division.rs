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
/// ```compile_fail,E0451
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
#[cfg_attr(
    not(feature = "construct"),
    doc = "\nWithout the `construct` feature there is no constructor to call. (This block is\ncompiled only in that configuration: with `construct` on it would compile.)\n\n```compile_fail,E0433\nuse quire_checked::construct::admitted_integer_division;\nuse quire_exact::DivisionProfile;\nlet _ = admitted_integer_division(DivisionProfile::Floor);\n```"
)]
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
