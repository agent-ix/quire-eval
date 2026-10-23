// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `Value`-family runtime causes both evaluators raise (FR-090, FR-403),
//! each with one catalog code, prefix, field set and rendering.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use quire_exact::ObjectReference;
use quire_semantic_value::catalog::{
    render_identity, CatalogCode, CatalogCoded, UndefinedCoded, UndefinedReason, UndefinedRecord,
    STATE_MODEL_CATALOG_PREFIX,
};

/// The `precondition-false` payload (`native-diagnostics.md`): the called
/// effective operation, the selected method's effective identity, the
/// receiver reference and the call locus. The call locus is the evaluator's
/// own [`crate::Evaluation::location`], not repeated here.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PreconditionFailure {
    /// The called effective operation's unqualified member name.
    pub operation: String,
    /// The selected method's effective identity: the redefinition candidate
    /// the receiver's most-specific runtime type actually linked to.
    pub selected: String,
    /// The receiver reference the call was made on.
    pub receiver: ObjectReference,
}

/// A protocol reference expression (FR-308) reached evaluation. Its value is
/// the value of the protocol clause it names, and no stage evaluates a
/// protocol clause: both evaluators refuse it, typed, rather than answer with
/// a value no clause computed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProtocolReferenceUnsupported;

impl CatalogCoded for ProtocolReferenceUnsupported {
    /// `unknown_required_feature`/`unsupported-feature`, naming the
    /// construct.
    fn catalog_code(&self) -> CatalogCode {
        CatalogCode::new("unknown_required_feature", "unsupported-feature")
    }

    fn catalog_fields(&self) -> Option<BTreeMap<&'static str, String>> {
        Some(BTreeMap::from([("construct", "protocol".to_string())]))
    }
}

/// A read through a `Reference<T>` over an object environment that holds no
/// object: a replay (FR-098) admits a reference argument by its identity, and
/// a call that dereferences it completes no value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReferenceUnresolved;

impl CatalogCoded for ReferenceUnresolved {
    /// `unknown_required_feature`/`unsupported-feature`, naming the
    /// construct.
    fn catalog_code(&self) -> CatalogCode {
        CatalogCode::new("unknown_required_feature", "unsupported-feature")
    }

    fn catalog_fields(&self) -> Option<BTreeMap<&'static str, String>> {
        Some(BTreeMap::from([("construct", "dereference".to_string())]))
    }
}

/// ADR-013 O-16: the `StateModel` family's evaluation-time undefined cause
/// (FR-090-AC-11, AC-12). ADR-012 assigns dispatch, dispatch preconditions
/// and population lookup to `StateModel` (§1, §3, §4.3).
#[derive(Clone, Eq, PartialEq)]
pub enum StateModelUndefined {
    /// FR-151 (TC-196 D06): a dispatched `receiver.member(args)` call's
    /// selected method's effective precondition evaluated to `false`.
    PreconditionFalse(PreconditionFailure),
    /// FR-153: a `lookup<T>(p, r) absent undefined` query's reference `r`
    /// names no member of the population bound to `p`.
    AbsentKey {
        /// The population binding `p` is bound to, rendered for the
        /// catalog payload.
        binding: String,
        /// The requested reference key `r`, rendered by
        /// [`render_identity`].
        key: String,
    },
    /// FR-090-AC-21: an `allInstances<T>` query cannot produce a complete
    /// result because no declared population member type covers the queried
    /// type by reflexive conformance.
    IncompleteClosure {
        /// The queried effective type's canonical node name.
        queried_type: String,
        /// The queried type as the minimal uncovered-subtype witness.
        uncovered_subtypes: Vec<String>,
    },
    /// FR-139: an `only(x in P where c: e)` form found no object of the
    /// population satisfying `c`, or several.
    OnlyNotOne {
        /// The population's declaration identity.
        population: String,
        /// How many objects satisfied the condition.
        found: u64,
    },
}

impl core::fmt::Debug for StateModelUndefined {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("StateModelUndefined")
            .field("catalog_code", &self.catalog_code())
            .field("record", &self.undefined_record())
            .finish()
    }
}

impl UndefinedCoded for StateModelUndefined {
    fn catalog_prefix(&self) -> Option<&'static str> {
        Some(STATE_MODEL_CATALOG_PREFIX)
    }

    fn undefined_record(&self) -> UndefinedRecord {
        match self {
            Self::PreconditionFalse(failure) => UndefinedRecord {
                reason: UndefinedReason::PreconditionFalse,
                fields: BTreeMap::from([
                    ("operation", failure.operation.clone()),
                    ("selected", failure.selected.clone()),
                    (
                        "receiver",
                        render_identity(failure.receiver.object().as_str().as_bytes()),
                    ),
                ]),
            },
            Self::AbsentKey { binding, key } => UndefinedRecord {
                reason: UndefinedReason::AbsentKey,
                fields: BTreeMap::from([("binding", binding.clone()), ("key", key.clone())]),
            },
            Self::IncompleteClosure {
                queried_type,
                uncovered_subtypes,
            } => UndefinedRecord {
                reason: UndefinedReason::IncompleteClosure,
                fields: BTreeMap::from([
                    ("closure", "subtype".to_string()),
                    ("queried_type", queried_type.clone()),
                    ("uncovered_subtypes", uncovered_subtypes.join(", ")),
                ]),
            },
            Self::OnlyNotOne { population, found } => UndefinedRecord {
                reason: UndefinedReason::OnlyNotOne,
                fields: BTreeMap::from([
                    ("population", population.clone()),
                    ("found", found.to_string()),
                ]),
            },
        }
    }
}
