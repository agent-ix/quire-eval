// SPDX-License-Identifier: AGPL-3.0-or-later
//! One-occurrence removal over canonical checked collections (FR-003).

use alloc::vec::Vec;
use core::cmp::Ordering;

use quire_exact::{
    compare_keys, form_grouped, member_equal, CardinalityBound, CollectionKind, CollectionType,
    CollectionValue, EmptyCardinalityBound, IllTyped, IllTypedCause, Meter, Outcome, Value,
};
use quire_semantic_value::declaration::{EnvironmentFailure, TypeEnvironment};

/// A failure before the collection kernel produces an evaluation outcome.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CollectionFailure {
    /// The canonical static-type refusal, before any operation charge.
    #[error(transparent)]
    IllTyped(#[from] IllTyped),
    /// The caller's canonical schema authority could not decide admission.
    #[error("collection element admission: {0:?}")]
    Environment(EnvironmentFailure),
    /// The derived collection bound could not be constructed.
    #[error(transparent)]
    Bound(#[from] EmptyCardinalityBound),
}

fn result_type(
    source: &CollectionType,
    shrink_maximum: bool,
) -> Result<CollectionType, EmptyCardinalityBound> {
    let bound = source
        .bound()
        .map(|bound| {
            CardinalityBound::new(
                bound.minimum().saturating_sub(1),
                if shrink_maximum {
                    bound.maximum().saturating_sub(1)
                } else {
                    bound.maximum()
                },
            )
        })
        .transpose()?;
    Ok(CollectionType::new(
        source.kind(),
        source.element().clone(),
        bound,
    ))
}

/// Drop the first occurrence of a sequence or ordered set.
///
/// The source already admits its members. Canonical formation charges the
/// survivors without re-evaluating or re-admitting them. Other kinds refuse
/// before any charge, and an empty eligible collection stays empty.
pub fn drop_first(
    source: &CollectionValue,
    meter: &mut Meter,
) -> Result<Outcome<Value>, CollectionFailure> {
    match source.collection_type().kind() {
        CollectionKind::Sequence | CollectionKind::OrderedSet => {}
        CollectionKind::Set | CollectionKind::Bag => {
            return Err(IllTyped {
                cause: IllTypedCause::OperatorIneligible,
            }
            .into());
        }
    }
    let target = result_type(source.collection_type(), true)?;
    Ok(form_grouped(
        &target,
        source.elements().iter().skip(1).cloned().collect(),
        meter,
    ))
}

/// Remove one matching occurrence, retaining the canonical kind and order.
///
/// Admission uses the canonical schema environment, including conformance.
/// Membership charges follow `contains`: a bag visits each distinct member
/// once and the scan stops at the first match. Only then is the result formed.
/// Kernel refusal, undefined and incomplete outcomes pass through unchanged.
pub fn remove_one(
    source: &CollectionValue,
    probe: &Value,
    environment: &TypeEnvironment,
    meter: &mut Meter,
) -> Result<Outcome<Value>, CollectionFailure> {
    if !environment
        .admits(source.collection_type().element(), probe)
        .map_err(CollectionFailure::Environment)?
    {
        return Err(IllTyped {
            cause: IllTypedCause::TypeMismatch,
        }
        .into());
    }
    let target = result_type(source.collection_type(), false)?;
    let mut removed = None;
    let mut previous = None;
    for (index, member) in source.elements().iter().enumerate() {
        let repeated = source.collection_type().kind() == CollectionKind::Bag
            && previous
                .is_some_and(|previous| compare_keys(previous, member) == Some(Ordering::Equal));
        previous = Some(member);
        if repeated {
            continue;
        }
        match member_equal(probe, member, meter) {
            Outcome::Completed(true) => {
                removed = Some(index);
                break;
            }
            Outcome::Completed(false) => {}
            Outcome::Refused(cause) => return Ok(Outcome::Refused(cause)),
            Outcome::Undefined(cause) => return Ok(Outcome::Undefined(cause)),
            Outcome::Incomplete(cause) => return Ok(Outcome::Incomplete(cause)),
        }
    }
    let survivors: Vec<_> = source
        .elements()
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != removed)
        .map(|(_, value)| value.clone())
        .collect();
    Ok(form_grouped(&target, survivors, meter))
}
