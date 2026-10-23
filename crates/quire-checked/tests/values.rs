// SPDX-License-Identifier: AGPL-3.0-or-later
//! Behaviour of the moved values: `DeclarationKey`'s order and equality, and
//! the admitted division law surviving construction.

use quire_checked::key::DeclarationKey;

fn key(package: &str, node: &str) -> DeclarationKey {
    DeclarationKey {
        package: package.into(),
        node: node.into(),
    }
}

#[test]
fn keys_order_by_package_then_node_with_prefix_first() {
    let mut keys = vec![key("b", "a"), key("a", "z"), key("a", "ab"), key("a", "a")];
    keys.sort();
    assert_eq!(
        keys,
        vec![key("a", "a"), key("a", "ab"), key("a", "z"), key("b", "a")]
    );
}

#[test]
fn equal_nodes_in_one_package_are_equal_keys() {
    assert_eq!(key("p", "n"), key("p", "n"));
    assert_ne!(key("p", "n"), key("q", "n"));
    assert_ne!(key("p", "n"), key("p", "m"));
}

#[cfg(feature = "test-support")]
#[test]
fn fixture_key_lives_in_test_orders() {
    assert_eq!(DeclarationKey::fixture("n"), key("test/orders", "n"));
}

#[cfg(feature = "construct")]
#[test]
fn constructed_division_reports_the_admitted_law() {
    use quire_checked::construct::admitted_integer_division;
    use quire_exact::DivisionProfile;

    for profile in DivisionProfile::ALL {
        assert_eq!(admitted_integer_division(profile).profile(), profile);
    }
}
