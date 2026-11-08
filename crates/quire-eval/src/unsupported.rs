// SPDX-License-Identifier: AGPL-3.0-or-later
//! The payload of `CallFailure::Unsupported` (FR-294, FR-296, FR-401): an
//! entry the chosen execution backend cannot run.

use alloc::boxed::Box;
use alloc::string::{String, ToString};

use quire_exact::{NodeKey, ValueType};
use quire_semantic_value::catalog::CatalogCode;
use quire_semantic_value::location::ExpressionLocation;

/// The catalog code of an unsupported entry (QSpec FR-271).
const CODE: &str = "unimplemented_capability";
/// QSpec FR-272's cause when the caller supplied no such backend.
const CAUSE_NOT_SUPPLIED: &str = "backend-not-supplied";
/// QSpec FR-272's cause when the backend cannot translate a construct.
const CAUSE_UNTRANSLATABLE: &str = "untranslatable-entry";

/// The execution backend a request chose (FR-296).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionBackendChoice {
    /// The interpreter.
    Interpreter,
    /// The ahead-of-time compiler.
    Aot,
    /// The just-in-time compiler.
    Jit,
}

impl ExecutionBackendChoice {
    /// The lower-case name a diagnostic carries as its `backend` key.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Interpreter => "interpreter",
            Self::Aot => "aot",
            Self::Jit => "jit",
        }
    }
}

/// The construct of an entry that a backend cannot translate (FR-294).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UntranslatableConstruct {
    /// A checked expression form, by its ADR-012 form name (`literal`,
    /// `operator`, `let`, `if`, `call`, `record`, `collection`, or the owning
    /// family's form name).
    ExpressionForm(String),
    /// A checked value type the backend has no representation for.
    ValueType(Box<ValueType>),
    /// The node key of a checked operation the backend does not implement.
    Operation(NodeKey),
}

impl UntranslatableConstruct {
    /// The construct kind a diagnostic carries as its `construct` key:
    /// `expression-form`, `value-type` or `operation`.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ExpressionForm(_) => "expression-form",
            Self::ValueType(_) => "value-type",
            Self::Operation(_) => "operation",
        }
    }

    /// The name a diagnostic carries as its `construct_name` key: the form
    /// name, the value type's canonical name, or the operation's node key.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::ExpressionForm(name) => name.clone(),
            Self::ValueType(value_type) => String::from(value_type_name(value_type.as_ref())),
            Self::Operation(key) => key.to_string(),
        }
    }
}

/// The canonical name of a value type's kind. The match has no wildcard, so a
/// new kernel type fails to compile here until it is named.
const fn value_type_name(value_type: &ValueType) -> &'static str {
    match value_type {
        ValueType::Boolean => "Boolean",
        ValueType::Integer => "Integer",
        ValueType::Int(_) => "Int",
        ValueType::Rational(_) => "Rational",
        ValueType::Decimal(_) => "Decimal",
        ValueType::Float(_) => "Float",
        ValueType::Quantity(_) => "Quantity",
        ValueType::CarriedQuantity(_) => "CarriedQuantity",
        ValueType::Text(_) => "Text",
        ValueType::String => "String",
        ValueType::Uuid => "Uuid",
        ValueType::Timestamp => "Timestamp",
        ValueType::Enum(_) => "Enum",
        ValueType::Option(_) => "Option",
        ValueType::Composite(_) => "Composite",
        ValueType::Collection(_) => "Collection",
        ValueType::Map(_) => "Map",
        ValueType::BoundedQuantity(_) => "BoundedQuantity",
        ValueType::Reference(_) => "Reference",
        ValueType::Population(_) => "Population",
    }
}

/// Why a backend did not run an entry (FR-294).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnsupportedCause {
    /// The caller supplied no such backend (FR-296).
    NotSupplied,
    /// The first construct of the entry, in clause-node order, that the
    /// backend cannot translate, and where it is.
    Untranslatable {
        /// The construct.
        construct: UntranslatableConstruct,
        /// The construct's location.
        location: ExpressionLocation,
    },
}

/// An entry the chosen execution backend cannot run (FR-294, FR-296): the
/// payload of `CallFailure::Unsupported`.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the {} backend cannot run the entry: {cause:?}", backend.name())]
pub struct UnsupportedEntry {
    /// The backend that did not run the entry.
    pub backend: ExecutionBackendChoice,
    /// Why it did not.
    pub cause: UnsupportedCause,
}

/// The payload keys of an unsupported entry's diagnostic (FR-286): `backend`,
/// and for an untranslatable entry `construct`, `construct_name` and
/// `location`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedPayload<'a> {
    /// The `backend` key: the backend's lower-case name.
    pub backend: &'static str,
    /// The `construct` key; `None` for a backend that was not supplied.
    pub construct: Option<&'static str>,
    /// The `construct_name` key; `None` for a backend that was not supplied.
    pub construct_name: Option<String>,
    /// The `location` key; `None` for a backend that was not supplied.
    pub location: Option<&'a ExpressionLocation>,
}

impl UnsupportedEntry {
    /// The catalog code: `unimplemented_capability` with QSpec FR-272's
    /// cause, `backend-not-supplied` or `untranslatable-entry`.
    #[must_use]
    pub const fn catalog_code(&self) -> CatalogCode {
        CatalogCode::new(
            CODE,
            match self.cause {
                UnsupportedCause::NotSupplied => CAUSE_NOT_SUPPLIED,
                UnsupportedCause::Untranslatable { .. } => CAUSE_UNTRANSLATABLE,
            },
        )
    }

    /// The diagnostic's payload keys.
    #[must_use]
    pub fn payload(&self) -> UnsupportedPayload<'_> {
        let backend = self.backend.name();
        match &self.cause {
            UnsupportedCause::NotSupplied => UnsupportedPayload {
                backend,
                construct: None,
                construct_name: None,
                location: None,
            },
            UnsupportedCause::Untranslatable {
                construct,
                location,
            } => UnsupportedPayload {
                backend,
                construct: Some(construct.kind()),
                construct_name: Some(construct.name()),
                location: Some(location),
            },
        }
    }
}
