// SPDX-License-Identifier: AGPL-3.0-or-later
//! `modulo` resolves at its `quire-eval` path and returns what the kernel
//! returns (FR-403-AC-2).

mod support;

use quire_exact::{Integer, IntegerDomain, Outcome};

#[test]
fn modulo_returns_the_kernel_value_and_ordered_charges() {
    let domain = IntegerDomain::Mathematical;
    let (dividend, divisor) = (Integer::from(-7_i64), Integer::from(2_i64));

    let mut here = support::meter(1_000);
    let through_eval = quire_eval::modulo(&dividend, &divisor, &domain, &mut here);
    let mut direct = support::meter(1_000);
    let through_kernel = quire_exact::modulo(&dividend, &divisor, &domain, &mut direct);

    assert_eq!(through_eval, through_kernel);
    assert_eq!(through_eval, Outcome::Completed(Integer::from(1_i64)));
    assert_eq!(here.admitted_charges(), direct.admitted_charges());
    assert!(!direct.admitted_charges().is_empty());
}
