// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `Value`-family runtime causes: one catalog code, prefix, field set and
//! rendering each (FR-403-AC-11).

use quire_eval::{ProtocolReferenceUnsupported, ReferenceUnresolved, StateModelUndefined};
use quire_semantic_value::catalog::{
    CatalogCode, CatalogCoded, UndefinedCoded, UndefinedReason, STATE_MODEL_CATALOG_PREFIX,
};

#[test]
fn state_model_undefined_carries_the_state_model_prefix_and_rendered_fields() {
    let absent = StateModelUndefined::AbsentKey {
        binding: "pop-1".to_owned(),
        key: quire_semantic_value::catalog::render_identity(&[0xFF, 0xFE]),
    };
    assert_eq!(absent.catalog_prefix(), Some("state-model"));
    assert_eq!(absent.catalog_prefix(), Some(STATE_MODEL_CATALOG_PREFIX));
    let record = absent.undefined_record();
    assert_eq!(record.reason, UndefinedReason::AbsentKey);
    assert_eq!(record.fields["binding"], "pop-1");
    assert_eq!(record.fields["key"], "identity bytes 0xfffe (not UTF-8)");
}

#[test]
fn incomplete_closure_and_only_not_one_fill_their_catalog_fields() {
    let closure = StateModelUndefined::IncompleteClosure {
        queried_type: "T".to_owned(),
        uncovered_subtypes: vec!["A".to_owned(), "B".to_owned()],
    }
    .undefined_record();
    assert_eq!(closure.fields["closure"], "subtype");
    assert_eq!(closure.fields["uncovered_subtypes"], "A, B");
    let only = StateModelUndefined::OnlyNotOne {
        population: "P".to_owned(),
        found: 2,
    }
    .undefined_record();
    assert_eq!(only.fields["found"], "2");
}

#[test]
fn unsupported_constructs_name_themselves() {
    let expected = CatalogCode::new("unknown_required_feature", "unsupported-feature");
    assert_eq!(ProtocolReferenceUnsupported.catalog_code(), expected);
    assert_eq!(
        ProtocolReferenceUnsupported.catalog_fields().unwrap()["construct"],
        "protocol"
    );
    assert_eq!(ReferenceUnresolved.catalog_code(), expected);
    assert_eq!(
        ReferenceUnresolved.catalog_fields().unwrap()["construct"],
        "dereference"
    );
}
