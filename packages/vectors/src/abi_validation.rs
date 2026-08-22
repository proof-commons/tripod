//! Where a class refused before the target is asked actually lands.
//!
//! §4.3 asks for a typed ABI-validation result for the classes whose
//! boundary precedes target execution. Two of §18's classes are in that
//! position — a changed sequence and a changed transaction version —
//! and both got there the hard way: each was submitted, each was
//! *accepted*, and each was respecified once the acceptance showed the
//! class's boundary claim had been wrong.
//!
//! # The trap this closes
//!
//! An accepted mutation is not a target disagreeing with a class. It is
//! a target being asked a question it has no rule to answer, and the
//! answer costs a subject's coins. So the run withholds such an arm —
//! and withholding leaves the row with nowhere to land, which is how a
//! class ends up looking merely unrun. This module is the somewhere:
//! one row per withheld class, carrying what the first-party boundary
//! actually established about it.
//!
//! # Three outcomes, and only the first is evidence
//!
//! A class whose field the safe constructor pins is settled: there is no
//! request that produces the class, so the ABI's convention is enforced
//! where it is stated. A class with no such pin has only the bytes this
//! crate cut by hand, which say what surgery can do and nothing about
//! what the candidate admits. A class with neither has simply not been
//! looked at. None of the three is a target verdict, and the run never
//! asks for one.

use std::collections::BTreeMap;

use transaction::TargetTransaction;

use crate::bundle::FixtureBundle;
use crate::error::VectorError;
use crate::materialize::MaterializedTargetVector;
use crate::mutation::{NegativeMutation, apply};
use crate::subject::CanonicalSubject;
use crate::violation::TargetField;

/// Whether one arm's class is settled before any target is asked.
///
/// The one predicate the funding schedule, the staging loop, and this
/// index all read, so a class the §18 matrix moves cannot leave three
/// call sites disagreeing about whether it is submittable.
///
/// # Errors
///
/// [`VectorError::MatrixCoverageMismatch`] when §18 names no class by
/// this arm's name.
pub fn precedes_the_target(mutation: NegativeMutation) -> Result<bool, VectorError> {
    Ok(mutation.expected_boundary()?.is_pre_target())
}

/// What the ABI-validation boundary establishes about one class.
///
/// Exclusive by construction, in the order written: a class the safe
/// constructor forecloses is settled there and the other two questions
/// do not arise; a class it does not foreclose is only whatever raw
/// surgery made of it; and a class surgery cannot make either has not
/// been looked at at all.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AbiValidationOutcome {
    /// The safe constructor will not emit the class.
    ///
    /// Observed rather than asserted: every transaction the candidate
    /// constructor produced for this plan carries the value the ABI
    /// itself declares for the field this class changes, and the typed
    /// request has no field with which to ask for another. A refusal by
    /// unrepresentability, which is the only kind a constructor whose
    /// safety is its refusal to emit invalid transactions can offer.
    SafeConstructorRefused,
    /// Bytes of this class exist, and only because this crate cut them.
    ///
    /// The mutation module builds them by surgery on an accepted
    /// transaction, which is a fact about this crate's test machinery
    /// and not about what the candidate ABI admits. Recorded so a
    /// reader cannot mistake the existence of a malformed vector for a
    /// first-party refusal of one.
    UnsafeRawMutationExists,
    /// No first-party boundary settled the class, and no target was
    /// asked.
    ///
    /// The honest empty answer. It is not a refusal, it is not a
    /// verdict, and it is deliberately reachable so that a class
    /// nothing looks at reads differently from one something does.
    TargetWasNotAsked,
}

/// One withheld class, and what the ABI boundary made of it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AbiValidationRow {
    class: NegativeMutation,
    field: TargetField,
    outcome: AbiValidationOutcome,
    constructions_observed: usize,
}

impl AbiValidationRow {
    /// The §18 class this row is about.
    #[must_use]
    pub const fn class(self) -> NegativeMutation {
        self.class
    }

    /// The one target field the class changes.
    #[must_use]
    pub const fn field(self) -> TargetField {
        self.field
    }

    /// What the boundary established.
    #[must_use]
    pub const fn outcome(self) -> AbiValidationOutcome {
        self.outcome
    }

    /// How many constructed transactions the pin was read from.
    ///
    /// Zero would mean the outcome rests on nothing, which is why the
    /// figure travels with it rather than being implied by the variant.
    #[must_use]
    pub const fn constructions_observed(self) -> usize {
        self.constructions_observed
    }
}

/// Index every class whose refusal precedes the target.
///
/// One row per withheld arm, in arm order. An arm the run submits has
/// no row here: its answer comes from a target, and a row that could
/// hold either would be the conflation §4.3 exists to prevent.
///
/// # Errors
///
/// [`VectorError::MatrixCoverageMismatch`] when §18 names no class by
/// an arm's name, and
/// [`VectorError::AbiValidationFieldUnderdetermined`] when a withheld
/// arm's declaration does not name exactly one changed target field —
/// the boundary can pin one field, and a class changing several is not
/// a class this index can speak for.
pub fn index_abi_validation(
    bundle: &FixtureBundle,
    vectors: &[CanonicalSubject<MaterializedTargetVector>],
) -> Result<BTreeMap<NegativeMutation, AbiValidationRow>, VectorError> {
    use compiler::operation_plan::SponsorCase;

    let mut rows = BTreeMap::new();
    for &class in NegativeMutation::ALL {
        if !precedes_the_target(class)? {
            continue;
        }
        // The sponsor case does not enter: what is asked here is whether
        // the constructor can be made to emit the class at all, and the
        // constructor writes both fields the same way in both forms.
        let declaration = class.declaration(SponsorCase::Absent)?;
        let mut declared = declaration.target_fields().iter().copied();
        let (Some(field), None) = (declared.next(), declared.next()) else {
            return Err(VectorError::AbiValidationFieldUnderdetermined { class });
        };

        let observed = pinned_constructions(bundle, vectors, field)?;
        let outcome = if observed > 0 {
            AbiValidationOutcome::SafeConstructorRefused
        } else if raw_bytes_exist(bundle, vectors, class) {
            AbiValidationOutcome::UnsafeRawMutationExists
        } else {
            AbiValidationOutcome::TargetWasNotAsked
        };
        rows.insert(
            class,
            AbiValidationRow {
                class,
                field,
                outcome,
                constructions_observed: observed,
            },
        );
    }
    Ok(rows)
}

/// How many constructed transactions carry the ABI's own value for one
/// field.
///
/// Zero where any of them does not, so one nonconforming construction
/// takes the whole pin down rather than being outvoted by the rest.
///
/// # Errors
///
/// [`VectorError::UnmutatableVector`] when a materialized vector does
/// not decode, which is the same precondition every mutation has.
fn pinned_constructions(
    bundle: &FixtureBundle,
    vectors: &[CanonicalSubject<MaterializedTargetVector>],
    field: TargetField,
) -> Result<usize, VectorError> {
    let mut observed = 0_usize;
    for vector in vectors.iter().map(CanonicalSubject::subject) {
        let decoded = crate::mutation::round_trips(vector)?;
        if !carries_the_abi_value(bundle, vector, &decoded, field) {
            return Ok(0);
        }
        observed += 1;
    }
    Ok(observed)
}

/// Whether one constructed transaction carries the ABI's own value.
fn carries_the_abi_value(
    bundle: &FixtureBundle,
    vector: &MaterializedTargetVector,
    decoded: &TargetTransaction,
    field: TargetField,
) -> bool {
    match field {
        TargetField::InputSequence => {
            let wanted = bundle.abi().sequence().sequence();
            decoded
                .inputs()
                .iter()
                .all(|input| input.sequence() == wanted)
        }
        TargetField::Version => bundle
            .abi()
            .shape(vector.shape())
            .is_some_and(|shape| decoded.version() == shape.version().version()),
        // No other field of the vocabulary is one the ABI states a
        // single value for, so no other field can be pinned this way.
        // Answered `false` rather than by a wider reading, which would
        // have let a class claim a pin nothing enforces.
        _ => false,
    }
}

/// Whether raw surgery can build bytes of one class from these vectors.
fn raw_bytes_exist(
    bundle: &FixtureBundle,
    vectors: &[CanonicalSubject<MaterializedTargetVector>],
    class: NegativeMutation,
) -> bool {
    vectors
        .iter()
        .map(CanonicalSubject::subject)
        .any(|vector| apply(vector, bundle.closed_asset(), class).is_ok())
}

#[cfg(test)]
mod tests {
    use super::{AbiValidationOutcome, index_abi_validation, precedes_the_target};
    use crate::bundle::fixture_bundle;
    use crate::mutation::NegativeMutation;
    use crate::plan::derive_evidence_plan;
    use crate::subject::CanonicalSubject;
    use crate::violation::TargetField;

    /// Every vector the canonical plan materialized.
    fn vectors() -> (
        crate::bundle::FixtureBundle,
        Vec<CanonicalSubject<crate::materialize::MaterializedTargetVector>>,
    ) {
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = derive_evidence_plan(&bundle).expect("the evidence plan derives");
        let vectors = plan.target_cases().to_vec();
        (bundle, vectors)
    }

    #[test]
    fn exactly_the_two_respecified_arms_are_indexed_here() {
        // The index covers the withheld arms and nothing else. An arm
        // that gained or lost a pre-target boundary changes this count,
        // which is the point: both arms that are here arrived by being
        // accepted at a target and respecified afterwards.
        let (bundle, vectors) = vectors();
        let rows = index_abi_validation(&bundle, &vectors).expect("the index builds");
        assert_eq!(rows.len(), 2);
        assert!(rows.contains_key(&NegativeMutation::ChangeInputSequence));
        assert!(rows.contains_key(&NegativeMutation::ChangeTransactionVersion));
    }

    #[test]
    fn both_withheld_classes_are_settled_by_the_safe_constructor() {
        // The evidence, recomputed. Every transaction the candidate
        // constructor produced for this plan carries the ABI's own
        // sequence and its own shape's version, and the typed request
        // has no field to ask for another — so neither class is
        // reachable through the constructor at all.
        let (bundle, vectors) = vectors();
        let rows = index_abi_validation(&bundle, &vectors).expect("the index builds");
        assert!(!vectors.is_empty(), "the plan materialized nothing");
        for (class, field) in [
            (
                NegativeMutation::ChangeInputSequence,
                TargetField::InputSequence,
            ),
            (
                NegativeMutation::ChangeTransactionVersion,
                TargetField::Version,
            ),
        ] {
            let row = rows[&class];
            assert_eq!(row.field(), field);
            assert_eq!(row.outcome(), AbiValidationOutcome::SafeConstructorRefused);
            assert_eq!(
                row.constructions_observed(),
                vectors.len(),
                "{class:?} was read from fewer constructions than the plan holds",
            );
        }
    }

    #[test]
    fn an_indexed_class_is_never_one_the_run_submits() {
        // §4.3's rule as a property of the two sets: a class this index
        // speaks for is a class no run offers, so an available executor
        // can never be the reason a pre-target class reached a target.
        let (bundle, vectors) = vectors();
        let rows = index_abi_validation(&bundle, &vectors).expect("the index builds");
        for &class in NegativeMutation::ALL {
            let withheld = precedes_the_target(class).expect("the class is named");
            assert_eq!(
                withheld,
                rows.contains_key(&class),
                "{class:?} disagrees about whether it is withheld",
            );
        }
    }

    #[test]
    fn the_pin_is_read_from_the_constructions_and_not_from_a_constant() {
        // The check that keeps the outcome from being a restatement:
        // a transaction carrying another sequence takes the pin down,
        // and the index says so rather than reporting the constant it
        // hoped for. Staged by asking the reading directly, because the
        // constructor cannot be made to emit one.
        use crate::mutation::apply;
        use transaction::TargetTransaction;

        let (bundle, vectors) = vectors();
        let subject = vectors
            .first()
            .map(CanonicalSubject::subject)
            .expect("the plan materialized a vector");
        let mutated = apply(
            subject,
            bundle.closed_asset(),
            NegativeMutation::ChangeInputSequence,
        )
        .expect("the arm applies");
        let decoded = TargetTransaction::decode(mutated.bytes()).expect("the mutation decodes");
        assert!(
            !super::carries_the_abi_value(&bundle, subject, &decoded, TargetField::InputSequence),
            "a changed sequence was read as carrying the ABI's own",
        );
    }
}
