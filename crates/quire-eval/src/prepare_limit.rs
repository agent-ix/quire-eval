// SPDX-License-Identifier: AGPL-3.0-or-later
//! The payload of `CallFailure::Limit` (FR-279, FR-294, FR-401): a prepare
//! bound that a backend's `prepare` reached.

/// The catalog code of a reached prepare limit.
const CODE: &str = "stage_limit_exceeded";

/// A `PrepareLimits` field (FR-294).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrepareSetting {
    /// The translation work one `prepare` may use.
    CompileWorkUnits,
    /// The generated code size one `prepare` may use.
    CodeBytes,
}

impl PrepareSetting {
    /// The setting's name (FR-277).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CompileWorkUnits => "prepare.compile_work_units",
            Self::CodeBytes => "prepare.code_bytes",
        }
    }
}

/// A prepare limit that `prepare` reached (FR-279, FR-294): the payload of
/// `CallFailure::Limit`. A prepare budget is charged over the selected
/// entries, not at a source location, so the record has no locus.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{} reached: bound {bound}, counter {actual}", setting.name())]
pub struct PrepareLimitReached {
    /// The setting whose bound was reached.
    pub setting: PrepareSetting,
    /// The configured bound.
    pub bound: u64,
    /// The counter the refused charge reached.
    pub actual: u64,
}

impl PrepareLimitReached {
    /// The catalog code, `stage_limit_exceeded`.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        CODE
    }
}
