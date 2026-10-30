// SPDX-License-Identifier: AGPL-3.0-or-later
//! The original declaration identity a domain package supplies (FR-150).

use alloc::string::String;

/// One original declaration key: the domain `package` and the IR `node`
/// (`model-complete.md`, "Identity domains";
/// `model-effective-declaration.schema.json`, `$defs.DeclarationKey`). Two
/// versions of one domain package with equal nodes yield equal declaration
/// keys. It is a plain value and claims nothing, so its fields are public.
///
/// Ordering matches `model-complete.md`: `package`, then `node`, each as UTF-8
/// bytes, a proper prefix first. Field declaration order gives exactly that
/// comparison under `derive(Ord)`.
///
/// The model stage keeps `declaration_key_wire` for the key's wire view.
///
/// quire:canonical
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeclarationKey {
    /// The domain package identity (FR-154).
    pub package: String,
    /// The IR node identity.
    pub node: String,
}

impl DeclarationKey {
    /// A `test/orders` key over `node`, following TC-195's fixture convention:
    /// every TC-195 fixture is a domain package with identity `test/orders`.
    #[cfg(any(test, feature = "test-support"))]
    pub fn fixture(node: impl Into<String>) -> Self {
        Self {
            package: String::from("test/orders"),
            node: node.into(),
        }
    }
}
