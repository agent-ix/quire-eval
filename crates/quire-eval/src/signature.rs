// SPDX-License-Identifier: AGPL-3.0-or-later
//! The admitted signature of a callable.

use alloc::vec::Vec;

use quire_exact::ValueType;

/// The admitted signature of a callable: its ordered parameter types and its
/// result type.
///
/// Two signatures are equal exactly when they have the same number of
/// parameters and every position's type and the result type are equal.
/// Parameter names are not part of a signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signature {
    parameters: Vec<ValueType>,
    result: ValueType,
}

impl Signature {
    /// A signature of `parameters`, in declared order, returning `result`.
    pub fn new(parameters: impl Into<Vec<ValueType>>, result: ValueType) -> Self {
        Self {
            parameters: parameters.into(),
            result,
        }
    }

    /// The parameter types, in declared order.
    #[must_use]
    pub fn parameters(&self) -> &[ValueType] {
        &self.parameters
    }

    /// The result type.
    #[must_use]
    pub const fn result(&self) -> &ValueType {
        &self.result
    }
}
