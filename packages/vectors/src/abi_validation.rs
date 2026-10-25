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
//! Constructor pinning and malformed-input refusal are separate checks.
//! Only a control accepted by the transaction-owned field validator and
//! a focused mutant refused by that same entry establish an ABI refusal.
//! Two supplemental rows have four sponsor-case identities; none enters
//! the compiler-derived coverage denominator.

use std::collections::BTreeMap;

use compiler::operation_plan::SponsorCase;
use transaction::TargetTransaction;
use transaction::abi::{CompactAbiValidationRefusal, validate_compact_abi_fields};

use crate::bundle::FixtureBundle;
use crate::error::VectorError;
use crate::materialize::MaterializedTargetVector;
use crate::mutation::{NegativeMutation, apply};
use crate::subject::CanonicalSubject;
use crate::violation::TargetField;

/// One supplemental ABI requirement; its relation is the named ABI convention.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbiBoundaryRequirement {
    pub mutation: NegativeMutation,
    pub sponsor: SponsorCase,
}

impl AbiBoundaryRequirement {
    /// Stable identity distinct from every compiler coverage identity.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match (self.mutation, self.sponsor) {
            (NegativeMutation::ChangeInputSequence, SponsorCase::Absent) => {
                "wrong-sequence-sponsorless"
            }
            (NegativeMutation::ChangeInputSequence, SponsorCase::Present) => {
                "wrong-sequence-sponsored"
            }
            (NegativeMutation::ChangeTransactionVersion, SponsorCase::Absent) => {
                "wrong-transaction-version-sponsorless"
            }
            (NegativeMutation::ChangeTransactionVersion, SponsorCase::Present) => {
                "wrong-transaction-version-sponsored"
            }
            _ => "not-an-abi-boundary-requirement",
        }
    }
}

/// ABI boundary rows: 2, instantiated in both sponsor cases (four identities).
#[must_use]
pub fn abi_boundary_requirements() -> Vec<AbiBoundaryRequirement> {
    [
        NegativeMutation::ChangeInputSequence,
        NegativeMutation::ChangeTransactionVersion,
    ]
    .into_iter()
    .flat_map(|mutation| {
        [SponsorCase::Absent, SponsorCase::Present]
            .into_iter()
            .map(move |sponsor| AbiBoundaryRequirement { mutation, sponsor })
    })
    .collect()
}

/// Explicit test staging at the ABI boundary, never a production construction.
/// A constructed control is retained beside one of two named field faults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactAbiTestStaging {
    control: TargetTransaction,
    malformed: TargetTransaction,
    refusal: CompactAbiValidationRefusal,
}

impl CompactAbiTestStaging {
    /// Clone a constructed control and apply only a sequence or version fault.
    ///
    /// # Errors
    /// Refuses non-ABI mutations, invalid controls, decode failures, and any
    /// result other than the exact intended typed ABI refusal.
    pub fn new(
        bundle: &FixtureBundle,
        vector: &MaterializedTargetVector,
        mutation: NegativeMutation,
    ) -> Result<Self, VectorError> {
        let fail = || VectorError::MatrixCoverageMismatch {
            class: "compact ABI test staging",
        };
        let control = crate::mutation::round_trips(vector)?;
        validate_compact_abi_fields(bundle.abi(), vector.shape(), &control).map_err(|_| fail())?;
        let expected = match mutation {
            NegativeMutation::ChangeInputSequence => {
                CompactAbiValidationRefusal::WrongInputSequence {
                    input: 0,
                    expected: bundle.abi().sequence().sequence(),
                    observed: bundle.abi().sequence().sequence() ^ 1,
                }
            }
            NegativeMutation::ChangeTransactionVersion => {
                CompactAbiValidationRefusal::WrongTransactionVersion {
                    expected: control.version(),
                    observed: control.version() ^ 1,
                }
            }
            _ => return Err(fail()),
        };
        let mutant = apply(vector, bundle.closed_asset(), mutation)?;
        let malformed = TargetTransaction::decode(mutant.bytes()).map_err(|_| fail())?;
        let refusal = validate_compact_abi_fields(bundle.abi(), vector.shape(), &malformed)
            .err()
            .ok_or_else(fail)?;
        if refusal != expected {
            return Err(fail());
        }
        Ok(Self {
            control,
            malformed,
            refusal,
        })
    }

    /// The unchanged constructed control.
    #[must_use]
    pub const fn control(&self) -> &TargetTransaction {
        &self.control
    }

    /// The exact malformed subject offered to the ABI validator.
    #[must_use]
    pub const fn malformed(&self) -> &TargetTransaction {
        &self.malformed
    }

    /// The refusal returned by the transaction-owned entry.
    #[must_use]
    pub const fn refusal(&self) -> &CompactAbiValidationRefusal {
        &self.refusal
    }
}

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
/// Constructor output pinning alone is not a malformed-input refusal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AbiValidationOutcome {
    /// The transaction-owned ABI entry accepted the control and returned
    /// the exact field refusal for the staged malformed subject.
    AbiFieldRefused,
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
            for vector in vectors.iter().map(CanonicalSubject::subject) {
                CompactAbiTestStaging::new(bundle, vector, class)?;
            }
            AbiValidationOutcome::AbiFieldRefused
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

    #[test]
    fn supplemental_abi_rows_have_four_staged_sponsor_case_identities() {
        use super::{CompactAbiTestStaging, abi_boundary_requirements};
        use crate::materialize::{
            AshFunding, SponsorCoin, materialize_sponsored, sponsor_signing_requests, vector_id,
        };
        use compiler::operation_plan::SponsorCase;
        use transaction::{Outpoint, SponsorSignature, Txid};
        let (bundle, vectors) = vectors();
        let case = crate::fixture::positive_semantic_census()
            .expect("cases")
            .into_iter()
            .find(|case| {
                crate::materialize::needs_authorization(case)
                    && crate::materialize::has_candidate_program(case)
            })
            .expect("sponsored shape");
        let funding = AshFunding::unexecutable_placeholder(vector_id(&case));
        let mut program = vec![0x00, 0x14];
        program.extend([0x22; 20]);
        let coins = vec![SponsorCoin::new(
            Outpoint::new(Txid::from_internal([0xc0; 32]), 0).expect("outpoint"),
            10_000,
            program,
        )];
        let tasks = sponsor_signing_requests(&bundle, &case, &funding, &coins).expect("tasks");
        // Test-only signature-shaped bytes establish construction, never authorization.
        let signatures = tasks
            .iter()
            .map(|task| {
                (
                    task.coin().outpoint(),
                    SponsorSignature::new(
                        task.request().transaction().to_vec(),
                        vec![vec![0x30; 71], vec![0x02; 33]],
                    ),
                )
            })
            .collect();
        let sponsored = materialize_sponsored(&bundle, &case, &funding, &coins, &signatures)
            .expect("constructed");
        let sponsorless = vectors.first().expect("control").subject();
        let requirements = abi_boundary_requirements();
        assert_eq!(requirements.len(), 4);
        assert_eq!(
            requirements
                .iter()
                .map(|r| r.mutation)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            2
        );
        assert_eq!(
            requirements
                .iter()
                .map(|r| r.name())
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from([
                "wrong-sequence-sponsorless",
                "wrong-sequence-sponsored",
                "wrong-transaction-version-sponsorless",
                "wrong-transaction-version-sponsored"
            ])
        );
        for requirement in requirements {
            let vector = match requirement.sponsor {
                SponsorCase::Absent => sponsorless,
                SponsorCase::Present => &sponsored,
            };
            let staged = CompactAbiTestStaging::new(&bundle, vector, requirement.mutation)
                .expect("staged refusal");
            let expected_field = if requirement.mutation == NegativeMutation::ChangeInputSequence {
                TargetField::InputSequence
            } else {
                TargetField::Version
            };
            for &field in TargetField::ALL {
                assert_eq!(
                    field.moved(staged.control(), staged.malformed()),
                    field == expected_field
                );
            }
            assert_eq!(
                super::validate_compact_abi_fields(
                    bundle.abi(),
                    vector.shape(),
                    staged.malformed()
                )
                .as_ref()
                .err(),
                Some(staged.refusal())
            );
            assert!(
                super::validate_compact_abi_fields(bundle.abi(), vector.shape(), staged.control())
                    .is_ok()
            );
        }
    }

    #[test]
    fn abi_test_staging_refuses_other_faults_and_wrong_shape_counts() {
        let (bundle, vectors) = vectors();
        let vector = vectors.first().expect("control").subject();
        assert!(
            super::CompactAbiTestStaging::new(
                &bundle,
                vector,
                NegativeMutation::SplitSuccessorInTwo
            )
            .is_err()
        );
        let decoded = crate::mutation::round_trips(vector).expect("decode");
        let (&shape, expected) = bundle
            .abi()
            .shapes()
            .iter()
            .find(|(_, shape)| usize::from(shape.sponsor_range().1) != decoded.inputs().len())
            .expect("other count");
        assert_eq!(
            super::validate_compact_abi_fields(bundle.abi(), shape, &decoded),
            Err(super::CompactAbiValidationRefusal::WrongInputCount {
                expected: usize::from(expected.sponsor_range().1),
                observed: decoded.inputs().len(),
            })
        );
    }

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
            // Constructor pinning is retained; refusal additionally drives the ABI entry.
            assert_eq!(row.outcome(), AbiValidationOutcome::AbiFieldRefused);
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
