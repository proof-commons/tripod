//! End-to-end pilot acceptance (Guide-7 §19; §21 Wave 6).
//!
//! The three scoped analyzed programs the Phase-2 compiler boundary
//! exists to produce — compact ASH alone, live transfer alone, and the
//! combined two-operation scope — are assembled, validated by the
//! complete assembly validator, and then compared with the §19
//! acceptance requirements restated here as data. A matrix derived from
//! the analyzed program would only prove the program agrees with
//! itself, so every list below is written out: the relation census, the
//! discharge of each relation, the capabilities each relation requires,
//! the strategy each representation admits, and the measured size of
//! each operation factor.
//!
//! Nothing asserted here claims more than a target-independent
//! compiler can know (§5.2). Retained external evidence is a
//! requirement and never a verdict; a backend-structural obligation is
//! not an emitted structure; a carrier alternative is not a target
//! program; and the acceptance of "target absent", "deployment
//! lifecycle incomplete", and "compiler identity not minted" is that
//! these are *explicitly* absent from the assembled value rather than
//! quietly missing from the assertions.

use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    sync::LazyLock,
};

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    ExternalEvidenceRequirement, ProofKind, RelationId, RelationKind, RelationSubject,
    RepresentationMode, TransactionSide,
};

use super::bound_input;
use crate::{
    BoundCompilerInput,
    analyzed::{
        AnalyzedProofPlan, AnalyzedProofPlanProjection, AnalyzedSource, ArchitectureScopeStatus,
        LifecycleCompleteness, ScopedAnalyzedProgram, ScopedAnalyzedProgramProjection,
        analyze_scoped_program,
    },
    analyzed_operation::AnalyzedOperation,
    analyzed_validate::validate_scoped_analyzed_program,
    capability::{CapabilityView, RequiredCapability},
    carrier::{CarrierQuantification, CarrierRole},
    case::SponsorCase,
    layout::names_sponsor_amount,
    placement::{PlacedCarrier, PlacementSearchLimits, RelationActivity},
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    requirement::ProofDisposition,
    source::{RequiredSourceKind, is_sponsor_amount_operand},
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// One scope analyzed end to end and independently revalidated.
struct Accepted {
    input: BoundCompilerInput,
    program: ScopedAnalyzedProgram,
}

/// Assemble one scope and run the complete assembly validator over the
/// result.
///
/// The assembler already validates its own output; running the
/// validator again from the test is not redundant, because the
/// acceptance claim is that the *assembled value* satisfies §13
/// against a fresh re-derivation of its input, not that one call
/// happened to return `Ok`.
fn analyze(operations: &[OperationId]) -> Accepted {
    let input = bound_input(operations);
    let program = analyze_scoped_program(&input, limits()).expect("scoped analyzed program");

    validate_scoped_analyzed_program(&input, limits(), &program).expect("assembly closure");

    Accepted { input, program }
}

// Each scope is assembled once and shared: assembly runs a proof search
// and one placement search per plan and operation, so a per-assertion
// analysis would pay for the whole pipeline repeatedly.
static COMPACT_ASH: LazyLock<Accepted> = LazyLock::new(|| analyze(&[OperationId::CompactAsh]));
static TRANSFER_LIVE: LazyLock<Accepted> = LazyLock::new(|| analyze(&[OperationId::TransferLive]));
static COMBINED: LazyLock<Accepted> =
    LazyLock::new(|| analyze(&[OperationId::CompactAsh, OperationId::TransferLive]));

fn pilots() -> [&'static Accepted; 2] {
    [&COMPACT_ASH, &TRANSFER_LIVE]
}

impl Accepted {
    fn operation(&self) -> OperationId {
        *self
            .input
            .scope()
            .operations()
            .first()
            .expect("a nonempty scope")
    }

    fn scope(&self) -> BTreeSet<OperationId> {
        self.input.scope().operations().iter().copied().collect()
    }

    /// The representation modes one plan selects, across every object
    /// it decides.
    fn modes(plan: &ProofPlanCandidate) -> BTreeSet<RepresentationMode> {
        plan.representations.values().copied().collect()
    }

    /// The single mode one plan selects for one operation.
    fn mode_of(plan: &ProofPlanCandidate, operation: OperationId) -> RepresentationMode {
        let modes = plan
            .representations
            .iter()
            .filter(|(choice, _)| choice.operation == operation)
            .map(|(_, mode)| *mode)
            .collect::<BTreeSet<_>>();

        assert_eq!(modes.len(), 1, "one representation decision per operation");

        modes.into_iter().next().expect("one mode")
    }

    /// Every capability the analyzed program requires, across plans and
    /// relations.
    fn capabilities(&self) -> BTreeSet<RequiredCapability> {
        self.program
            .proof_plans
            .values()
            .flat_map(|plan| plan.relation_requirements.values())
            .flat_map(|bundle| bundle.required_capabilities.iter().copied())
            .collect()
    }
}

// --- §19.1 and §19.2 acceptance matrices, restated as data ---

/// How the guide expects one relation to be discharged.
///
/// The three cases are not degrees of the same thing: a selected proof
/// is a decision among realization-approved alternatives, a statically
/// validated relation had no decision to make, and an externally
/// evidenced relation has an approved proof class whose completion the
/// compiler cannot claim at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedProof {
    Selected(ProofKind),
    /// The conservation strategy the plan's representation mode admits
    /// (§19.2 proof strategies).
    ConservationStrategy,
    StaticallyValidated,
    ExternalEvidence,
}

/// The conservation strategy each representation mode admits (§19.2).
///
/// The two rejected pairings — private-committed with public
/// arithmetic, explicit with confidential conservation — are exactly
/// the entries this function does not contain, and
/// [`no_pilot_plan_pairs_a_representation_with_a_rejected_strategy`]
/// checks the plan set never produces them.
const fn conservation_strategy(mode: RepresentationMode) -> ProofKind {
    match mode {
        // An explicit amount is arithmetic on published values; a
        // public-committed amount is published through an
        // authenticated opening, so the arithmetic is public too.
        RepresentationMode::Explicit | RepresentationMode::PublicCommitted => {
            ProofKind::PublicArithmetic
        }
        RepresentationMode::PrivateCommitted => ProofKind::ConfidentialConservation,
    }
}

/// The capability that conservation strategy requires.
const fn conservation_capability(mode: RepresentationMode) -> RequiredCapability {
    match conservation_strategy(mode) {
        ProofKind::ConfidentialConservation => RequiredCapability::ConfidentialValueConservation,
        _ => RequiredCapability::ExactPublicAmountArithmetic,
    }
}

/// The source kind one amount operand is read through, per mode
/// (§19.2, representation-specific amount source).
const fn amount_source(mode: RepresentationMode) -> RequiredSourceKind {
    match mode {
        RepresentationMode::Explicit | RepresentationMode::PublicCommitted => {
            RequiredSourceKind::AuthenticatedConsensusValue
        }
        RepresentationMode::PrivateCommitted => RequiredSourceKind::AuthenticatedCommitmentRelation,
    }
}

/// One row of the §19 pilot acceptance matrix.
struct AcceptanceRow {
    relation: RelationId,
    proof: ExpectedProof,
    /// The capabilities this relation requires of a future target,
    /// independent of the representation mode.
    capabilities: BTreeSet<RequiredCapability>,
}

fn row(
    operation: OperationId,
    kind: RelationKind,
    subject: RelationSubject,
    proof: ExpectedProof,
    capabilities: &[RequiredCapability],
) -> AcceptanceRow {
    AcceptanceRow {
        relation: RelationId::new(operation, kind, subject),
        proof,
        capabilities: capabilities.iter().copied().collect(),
    }
}

fn family(side: TransactionSide, object: ObjectId) -> RelationSubject {
    RelationSubject::ObjectFamily { side, object }
}

/// A whole transaction side, which is what a closure relation is about.
const fn whole_side(side: TransactionSide) -> RelationSubject {
    RelationSubject::TransactionSide { side }
}

/// The §19.1 compact-ASH relation list and its requirements.
///
/// Written out row by row rather than folded into a loop: a table that
/// computed its own rows would restate the derivation instead of the
/// guide.
#[allow(clippy::too_many_lines)]
fn compact_ash_rows() -> Vec<AcceptanceRow> {
    use ExpectedProof::{ConservationStrategy, ExternalEvidence, Selected, StaticallyValidated};
    use ProofKind as Proof;
    use RelationKind as Kind;
    use RequiredCapability as Needs;

    let operation = OperationId::CompactAsh;
    let at = |kind, subject, proof, capabilities: &[Needs]| {
        row(operation, kind, subject, proof, capabilities)
    };
    let shape = Selected(Proof::ManifestShape);
    let cardinality = [
        Needs::AuthenticatedObjectRecognition,
        Needs::AuthenticatedFamilyCardinality,
    ];
    let recognition = [Needs::AuthenticatedObjectRecognition];

    vec![
        // ASH and sponsor input/output cardinality. The sponsor family
        // is optional; its cardinality relation is present in every
        // plan and inactive-valid in the unsponsored case.
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, ObjectId::Ash),
            shape,
            &cardinality,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, ObjectId::Ash),
            shape,
            &cardinality,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            shape,
            &cardinality,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            shape,
            &cardinality,
        ),
        // ASH and sponsor input/output recognition.
        at(
            Kind::Recognition,
            family(TransactionSide::Input, ObjectId::Ash),
            shape,
            &recognition,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, ObjectId::Ash),
            shape,
            &recognition,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            shape,
            &recognition,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            shape,
            &recognition,
        ),
        // Input and output family closure.
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Input),
            shape,
            &recognition,
        ),
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Output),
            shape,
            &recognition,
        ),
        // Permissionless authorization: a public construction datum,
        // never an owner or operator witness.
        at(
            Kind::Authorization,
            RelationSubject::Operation,
            Selected(Proof::PublicConstructibility),
            &[Needs::PublicConstructibility],
        ),
        // Public constructibility of the operation itself.
        at(
            Kind::Constructibility,
            RelationSubject::Operation,
            Selected(Proof::PublicConstructibility),
            &[Needs::PublicConstructibility],
        ),
        // Ownerless U conservation, by public arithmetic.
        at(
            Kind::Conservation,
            RelationSubject::Asset { asset: AssetId::U },
            ConservationStrategy,
            &recognition,
        ),
        // Canonical ownerless-lateral delta and the fee-sponsor-only
        // open-flow policy.
        at(
            Kind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedCanonicalPartition,
            ],
        ),
        at(
            Kind::OpenFlowPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedOpenFlowPartition,
            ],
        ),
        // Sponsor isolation and sponsor envelope multiplicity.
        at(
            Kind::SponsorIsolation,
            RelationSubject::Sponsor,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedOpenFlowPartition,
            ],
        ),
        at(
            Kind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedOpenFlowPartition,
            ],
        ),
        // The no-root policy and the transition-certificate-only
        // projection policy.
        at(
            Kind::RootPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedRootEffects,
            ],
        ),
        at(
            Kind::ProjectionPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedProjectionSet,
            ],
        ),
        // Representation selection, and the compact and clear lifecycle
        // exits: statically validated, so no proof variable and no
        // capability.
        at(
            Kind::Representation,
            RelationSubject::Representation {
                object: ObjectId::Ash,
            },
            StaticallyValidated,
            &[],
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::CompactAsh,
            },
            StaticallyValidated,
            &[],
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::Clear,
            },
            StaticallyValidated,
            &[],
        ),
        // Whole-transaction L-BTC substrate conservation stays an
        // external-evidence obligation.
        at(
            Kind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
            ExternalEvidence,
            &[Needs::WholeTransactionValueConservation],
        ),
    ]
}

/// The §19.2 live-transfer relation list and its requirements.
#[allow(clippy::too_many_lines)]
fn transfer_live_rows() -> Vec<AcceptanceRow> {
    use ExpectedProof::{ConservationStrategy, ExternalEvidence, Selected, StaticallyValidated};
    use ProofKind as Proof;
    use RelationKind as Kind;
    use RequiredCapability as Needs;

    let operation = OperationId::TransferLive;
    let at = |kind, subject, proof, capabilities: &[Needs]| {
        row(operation, kind, subject, proof, capabilities)
    };
    let shape = Selected(Proof::ManifestShape);
    let cardinality = [
        Needs::AuthenticatedObjectRecognition,
        Needs::AuthenticatedFamilyCardinality,
    ];
    let recognition = [Needs::AuthenticatedObjectRecognition];
    let object = ObjectId::ReceiptLive;

    vec![
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, object),
            shape,
            &cardinality,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, object),
            shape,
            &cardinality,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            shape,
            &cardinality,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            shape,
            &cardinality,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Input, object),
            shape,
            &recognition,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, object),
            shape,
            &recognition,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            shape,
            &recognition,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            shape,
            &recognition,
        ),
        // Every-owner authorization, quantified over the consumed
        // live-receipt family rather than over the transaction.
        at(
            Kind::Authorization,
            family(TransactionSide::Input, object),
            Selected(Proof::SignerMembership),
            &[Needs::OwnerAuthorization],
        ),
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Input),
            shape,
            &recognition,
        ),
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Output),
            shape,
            &recognition,
        ),
        // Aggregate U conservation, by the strategy the mode admits.
        at(
            Kind::Conservation,
            RelationSubject::Asset { asset: AssetId::U },
            ConservationStrategy,
            &recognition,
        ),
        at(
            Kind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedCanonicalPartition,
            ],
        ),
        at(
            Kind::OpenFlowPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedOpenFlowPartition,
            ],
        ),
        at(
            Kind::SponsorIsolation,
            RelationSubject::Sponsor,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedOpenFlowPartition,
            ],
        ),
        at(
            Kind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedOpenFlowPartition,
            ],
        ),
        at(
            Kind::RootPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedRootEffects,
            ],
        ),
        at(
            Kind::ProjectionPolicy,
            RelationSubject::Operation,
            shape,
            &[
                Needs::AuthenticatedObjectRecognition,
                Needs::AuthenticatedProjectionSet,
            ],
        ),
        // Owner constructibility: the live operation is constructible
        // by its owners, not permissionlessly.
        at(
            Kind::Constructibility,
            RelationSubject::Operation,
            Selected(Proof::SignerMembership),
            &[Needs::OwnerAuthorization],
        ),
        at(
            Kind::Representation,
            RelationSubject::Representation { object },
            StaticallyValidated,
            &[],
        ),
        // The transfer, burn, and redemption lifecycle exits.
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object,
                exit: OperationId::TransferLive,
            },
            StaticallyValidated,
            &[],
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object,
                exit: OperationId::Burn,
            },
            StaticallyValidated,
            &[],
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object,
                exit: OperationId::Redeem,
            },
            StaticallyValidated,
            &[],
        ),
        at(
            Kind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
            ExternalEvidence,
            &[Needs::WholeTransactionValueConservation],
        ),
    ]
}

fn rows(operation: OperationId) -> Vec<AcceptanceRow> {
    match operation {
        OperationId::CompactAsh => compact_ash_rows(),
        OperationId::TransferLive => transfer_live_rows(),
        other => panic!("{other:?} is not a pilot"),
    }
}

/// The representation modes §19.1 and §19.2 expect across each pilot's
/// complete plan set.
fn approved_modes(operation: OperationId) -> BTreeSet<RepresentationMode> {
    match operation {
        OperationId::CompactAsh => BTreeSet::from([
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ]),
        OperationId::TransferLive => BTreeSet::from([
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
        ]),
        other => panic!("{other:?} is not a pilot"),
    }
}

/// The measured §19 factorized-output census of one pilot operation.
///
/// One factor per proof plan. The relation, case, and relation-case
/// sizes are the same under every plan of one pilot, as §6.6 requires:
/// no relation appears or disappears with the representation.
///
/// The layout and coverage sizes are not, and that is §19.4. A plan
/// that holds the conserved amounts as commitments places no carrier
/// for conservation, so it states none of the layout that would route
/// authenticated family totals to one, and it answers at the evidence
/// boundary, where the report owes three negatives where a carrier owed
/// one focused rejection.
struct FactorCensus {
    relations: usize,
    cases: usize,
    relation_cases: usize,
    placements: usize,
    layout_requirements: usize,
    coverage_nodes: usize,
    coverage_edges: usize,
}

const fn factor_census(operation: OperationId, committed: bool) -> FactorCensus {
    match (operation, committed) {
        (OperationId::CompactAsh, _) => FactorCensus {
            relations: 23,
            cases: 2,
            relation_cases: 46,
            placements: 216,
            layout_requirements: 69,
            coverage_nodes: 344,
            coverage_edges: 365,
        },
        (OperationId::TransferLive, false) => FactorCensus {
            relations: 24,
            cases: 2,
            relation_cases: 48,
            placements: 216,
            layout_requirements: 69,
            coverage_nodes: 362,
            coverage_edges: 489,
        },
        (OperationId::TransferLive, true) => FactorCensus {
            relations: 24,
            cases: 2,
            relation_cases: 48,
            placements: 216,
            layout_requirements: 61,
            coverage_nodes: 359,
            coverage_edges: 475,
        },
        _ => panic!("not a pilot"),
    }
}

/// Whether one analyzed plan holds the pilot's conserved amounts as
/// commitments.
///
/// Read from the plan's own selected alternative rather than from the
/// case representation, because it is the selection that decides where
/// conservation is discharged and the two agree only because the
/// compiler makes them agree.
fn plan_is_committed(analysis: &AnalyzedProofPlan, operation: OperationId) -> bool {
    analysis
        .relation_requirements
        .iter()
        .any(|(relation, bundle)| {
            relation.operation() == operation
                && relation.kind() == RelationKind::Conservation
                && matches!(
                    &bundle.proof,
                    ProofDisposition::Selected { proof }
                        if proof.proof() == ProofKind::ConfidentialConservation
                )
        })
}

// --- §19.1 and §19.2 scope, cases, and relations ---

#[test]
fn every_pilot_relation_list_is_exactly_the_analyzed_relation_census() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let expected = rows(operation)
            .into_iter()
            .map(|row| row.relation)
            .collect::<BTreeSet<_>>();

        assert_eq!(expected.len(), factor_census(operation, false).relations);

        for analysis in pilot.program.proof_plans.values() {
            // The guide's list is the complete relation census of every
            // plan, not a sample of it.
            assert_eq!(
                analysis
                    .relation_requirements
                    .keys()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                expected,
                "{operation:?}",
            );

            // And the same list indexes the operation factor's
            // relation-cases.
            let factor = &analysis.operations[&operation];

            assert_eq!(
                factor
                    .relation_cases
                    .keys()
                    .map(|key| key.relation.clone())
                    .collect::<BTreeSet<_>>(),
                expected,
                "{operation:?}",
            );
        }
    }
}

#[test]
fn every_pilot_plan_set_carries_exactly_its_approved_representations() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let modes = pilot
            .program
            .proof_plans
            .keys()
            .flat_map(Accepted::modes)
            .collect::<BTreeSet<_>>();

        assert_eq!(modes, approved_modes(operation), "{operation:?}");

        // One plan per approved mode: each plan fixes one mode, and the
        // unconstrained view retains every approved one.
        assert_eq!(pilot.program.proof_plans.len(), modes.len());
    }
}

#[test]
fn every_pilot_plan_analyzes_exactly_the_unsponsored_and_sponsored_cases() {
    for pilot in pilots() {
        let operation = pilot.operation();

        for (plan, analysis) in &pilot.program.proof_plans {
            let factor = &analysis.operations[&operation];
            let mode = Accepted::mode_of(plan, operation);

            // The only case expansion is the optional sponsor family;
            // counts, denominations, owners, and object order remain
            // vector dimensions rather than compiler case dimensions.
            assert_eq!(
                factor
                    .execution_cases
                    .iter()
                    .map(|case| case.sponsor)
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([SponsorCase::Absent, SponsorCase::Present]),
                "{operation:?}",
            );
            assert_eq!(factor.execution_cases.len(), 2, "{operation:?}");

            // The plan fixes the representation, so both cases carry it.
            for case in &factor.execution_cases {
                assert_eq!(case.operation, operation);
                assert_eq!(
                    case.representations.values().copied().collect::<Vec<_>>(),
                    vec![mode],
                );
            }
        }
    }
}

// --- §19.1 and §19.2 requirements ---

#[test]
fn every_pilot_relation_is_discharged_exactly_as_its_acceptance_row_states() {
    for pilot in pilots() {
        let operation = pilot.operation();

        for (plan, analysis) in &pilot.program.proof_plans {
            let mode = Accepted::mode_of(plan, operation);

            for row in rows(operation) {
                let bundle = &analysis.relation_requirements[&row.relation];

                let expected_proof = match row.proof {
                    ExpectedProof::Selected(kind) => ProofDisposition::Selected {
                        proof: realization::ProofAlternativeId::new(row.relation.clone(), kind),
                    },
                    ExpectedProof::ConservationStrategy => ProofDisposition::Selected {
                        proof: realization::ProofAlternativeId::new(
                            row.relation.clone(),
                            conservation_strategy(mode),
                        ),
                    },
                    ExpectedProof::StaticallyValidated => ProofDisposition::StaticallyValidated,
                    ExpectedProof::ExternalEvidence => ProofDisposition::ExternalEvidence {
                        approved_proof: realization::ProofAlternativeId::new(
                            row.relation.clone(),
                            ProofKind::SubstrateConservation,
                        ),
                        requirement: ExternalEvidenceRequirement::SubstrateConservation {
                            operation,
                            asset: AssetId::Lbtc,
                        },
                    },
                };

                assert_eq!(
                    bundle.proof, expected_proof,
                    "{:?} under {mode:?}",
                    row.relation,
                );

                let mut expected = row.capabilities.clone();

                if row.proof == ExpectedProof::ConservationStrategy {
                    expected.insert(conservation_capability(mode));
                }

                assert_eq!(
                    bundle.required_capabilities, expected,
                    "{:?} under {mode:?}",
                    row.relation,
                );
            }
        }
    }
}

#[test]
fn no_pilot_plan_pairs_a_representation_with_a_rejected_strategy() {
    // §19.2 rejects private-committed with public arithmetic and
    // explicit with confidential conservation. The claim is that the
    // *plan set* contains no such pairing — the upstream realization
    // never offers one, and the analyzed program is where that would
    // become visible if it did.
    let rejected = [
        (
            RepresentationMode::PrivateCommitted,
            ProofKind::PublicArithmetic,
        ),
        (
            RepresentationMode::Explicit,
            ProofKind::ConfidentialConservation,
        ),
    ];

    for pilot in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        for (plan, analysis) in &pilot.program.proof_plans {
            for (relation, bundle) in &analysis.relation_requirements {
                if relation.kind() != RelationKind::Conservation {
                    continue;
                }

                let ProofDisposition::Selected { proof } = &bundle.proof else {
                    panic!("conservation is discharged by a selected proof");
                };

                let mode = Accepted::mode_of(plan, relation.operation());

                assert_eq!(proof.proof(), conservation_strategy(mode));

                for (rejected_mode, rejected_proof) in rejected {
                    assert!(
                        !(mode == rejected_mode && proof.proof() == rejected_proof),
                        "{relation:?} pairs {mode:?} with {rejected_proof:?}",
                    );
                }
            }
        }
    }
}

#[test]
fn compact_ash_requires_no_owner_or_operator_witness() {
    let capabilities = COMPACT_ASH.capabilities();

    // Permissionless authorization is what makes this a requirement
    // rather than an accident: no plan of the compact-ASH scope may
    // require an owner, operator, or refund authorization capability.
    for absent in [
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::OperatorAuthorization,
        RequiredCapability::RefundAuthorization,
    ] {
        assert!(!capabilities.contains(&absent), "{absent:?}");
    }

    // Nor any owner-side witness source row. The sponsor-local witness
    // is deliberately not in this list: sponsor owner authorization is
    // a retained sponsor property (§13.12), and erasing it would be a
    // different defect from requiring a protocol owner's signature.
    for analysis in COMPACT_ASH.program.proof_plans.values() {
        for bundle in analysis.relation_requirements.values() {
            for row in &bundle.source_requirements {
                assert!(
                    !matches!(
                        row.source,
                        RequiredSourceKind::InputOwnerWitness
                            | RequiredSourceKind::OperatorWitness
                            | RequiredSourceKind::RefundKeyWitness
                    ),
                    "{:?} requires {:?}",
                    bundle.relation,
                    row.source,
                );
            }
        }
    }

    // The complete compact-ASH capability census: the six authenticated
    // capabilities its policy relations need, public arithmetic for the
    // conservation proof, public constructibility, and the
    // whole-transaction substrate obligation that stays external.
    assert_eq!(
        capabilities,
        BTreeSet::from([
            RequiredCapability::AuthenticatedObjectRecognition,
            RequiredCapability::AuthenticatedFamilyCardinality,
            RequiredCapability::AuthenticatedCanonicalPartition,
            RequiredCapability::AuthenticatedOpenFlowPartition,
            RequiredCapability::AuthenticatedRootEffects,
            RequiredCapability::AuthenticatedProjectionSet,
            RequiredCapability::ExactPublicAmountArithmetic,
            RequiredCapability::PublicConstructibility,
            RequiredCapability::WholeTransactionValueConservation,
        ]),
    );
}

#[test]
fn compact_ash_conserves_ownerless_u_by_public_arithmetic() {
    let relation = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );

    for (plan, analysis) in &COMPACT_ASH.program.proof_plans {
        let bundle = &analysis.relation_requirements[&relation];
        let mode = Accepted::mode_of(plan, OperationId::CompactAsh);

        // Both approved compact-ASH modes publish the amount, so both
        // conserve by public arithmetic.
        assert_eq!(conservation_strategy(mode), ProofKind::PublicArithmetic);

        // The operands are the ASH family amounts on both sides, read
        // as authenticated consensus values — and nothing else.
        assert_eq!(
            bundle
                .source_requirements
                .iter()
                .map(|row| (row.source, row.operand.role().clone()))
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                (
                    amount_source(mode),
                    crate::source::OperandRole::ObjectFamilyAmount {
                        side: TransactionSide::Input,
                        object: ObjectId::Ash,
                    },
                ),
                (
                    amount_source(mode),
                    crate::source::OperandRole::ObjectFamilyAmount {
                        side: TransactionSide::Output,
                        object: ObjectId::Ash,
                    },
                ),
            ]),
        );
    }
}

#[test]
fn live_transfer_ties_the_every_owner_witness_to_the_live_input_family() {
    let relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::Authorization,
        family(TransactionSide::Input, ObjectId::ReceiptLive),
    );

    for analysis in TRANSFER_LIVE.program.proof_plans.values() {
        let bundle = &analysis.relation_requirements[&relation];

        // The witness is required of the consumed live-receipt family,
        // and the owner census it is checked against names the same
        // family. An operation-wide authorization would authorize a
        // transfer of receipts their owners never signed for.
        assert_eq!(
            bundle
                .source_requirements
                .iter()
                .map(|row| (row.source, row.operand.role().clone()))
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                (
                    RequiredSourceKind::AuthenticatedFamilyCensus,
                    crate::source::OperandRole::ObjectFamilyOwners {
                        object: ObjectId::ReceiptLive,
                    },
                ),
                (
                    RequiredSourceKind::InputOwnerWitness,
                    crate::source::OperandRole::ProtocolSignerSet,
                ),
            ]),
        );
        assert_eq!(
            bundle.required_capabilities,
            BTreeSet::from([RequiredCapability::OwnerAuthorization]),
        );

        // Every active case places it per member of that family — never
        // on a coordinator standing in for the owners.
        let factor = &analysis.operations[&OperationId::TransferLive];

        for (key, requirements) in &factor.relation_cases {
            if key.relation != relation {
                continue;
            }

            assert_eq!(requirements.activity, RelationActivity::Active);
            assert_eq!(
                requirements
                    .carrier_assignments
                    .iter()
                    .flat_map(|alternative| alternative.carriers.iter().cloned())
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([PlacedCarrier {
                    carrier: CarrierRole::EveryInputFamilyMember {
                        object: ObjectId::ReceiptLive,
                    },
                    quantification: CarrierQuantification::PerMember,
                }]),
            );
        }
    }
}

#[test]
fn live_transfer_reads_its_amounts_through_the_selected_representation() {
    let relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );

    let mut observed = BTreeMap::new();

    for (plan, analysis) in &TRANSFER_LIVE.program.proof_plans {
        let bundle = &analysis.relation_requirements[&relation];
        let mode = Accepted::mode_of(plan, OperationId::TransferLive);

        // The amount source follows the representation: an explicit
        // amount is an authenticated consensus value, a private
        // committed amount is only ever an authenticated commitment
        // relation. Reading a private amount as a consensus value would
        // publish it.
        assert_eq!(
            bundle
                .source_requirements
                .iter()
                .map(|row| row.source)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([amount_source(mode)]),
            "{mode:?}",
        );

        observed.insert(mode, amount_source(mode));
    }

    assert_eq!(
        observed,
        BTreeMap::from([
            (
                RepresentationMode::Explicit,
                RequiredSourceKind::AuthenticatedConsensusValue,
            ),
            (
                RepresentationMode::PrivateCommitted,
                RequiredSourceKind::AuthenticatedCommitmentRelation,
            ),
        ]),
    );
}

#[test]
fn every_pilot_places_aggregate_conservation_on_one_operation_global_carrier() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let anchor = match operation {
            OperationId::CompactAsh => ObjectId::Ash,
            _ => ObjectId::ReceiptLive,
        };
        let relation = RelationId::new(
            operation,
            RelationKind::Conservation,
            RelationSubject::Asset { asset: AssetId::U },
        );

        for analysis in pilot.program.proof_plans.values() {
            let factor = &analysis.operations[&operation];

            // A plan holding the conserved amounts as commitments
            // places no carrier for conservation at all, because there
            // is no arithmetic to place (§10.6). The expectation
            // follows the plan rather than tolerating either answer.
            let expected = if plan_is_committed(analysis, operation) {
                BTreeSet::new()
            } else {
                BTreeSet::from([PlacedCarrier {
                    carrier: CarrierRole::OperationGlobal { operation, anchor },
                    quantification: CarrierQuantification::Single,
                }])
            };

            for (key, requirements) in &factor.relation_cases {
                if key.relation != relation {
                    continue;
                }

                // One global carrier for a whole-family identity: a
                // per-member carrier could only prove a member's share.
                assert_eq!(
                    requirements
                        .carrier_assignments
                        .iter()
                        .flat_map(|alternative| alternative.carriers.iter().cloned())
                        .collect::<BTreeSet<_>>(),
                    expected,
                    "{operation:?}",
                );
            }
        }
    }
}

// --- §19.1 and §19.2 factorized outputs ---

#[test]
fn every_pilot_factor_is_exactly_its_measured_census() {
    for pilot in pilots() {
        let operation = pilot.operation();
        for analysis in pilot.program.proof_plans.values() {
            let census = factor_census(operation, plan_is_committed(analysis, operation));
            let factor = &analysis.operations[&operation];

            assert_eq!(factor.execution_cases.len(), census.cases, "{operation:?}");
            assert_eq!(
                factor.relation_cases.len(),
                census.relation_cases,
                "{operation:?}",
            );
            assert_eq!(
                factor.relation_cases.len(),
                census.relations * census.cases,
                "{operation:?}",
            );
            assert_eq!(
                factor.feasible_placements.len(),
                census.placements,
                "{operation:?}",
            );
            assert_eq!(
                factor.layout_requirements.len(),
                census.layout_requirements,
                "{operation:?}",
            );

            // Coverage is keyed by exactly the relation-cases, and the
            // dependency graph is the one those requirements imply.
            assert_eq!(
                factor.coverage.requirements.keys().collect::<Vec<_>>(),
                factor.relation_cases.keys().collect::<Vec<_>>(),
                "{operation:?}",
            );
            assert_eq!(
                factor.coverage.cases, factor.execution_cases,
                "{operation:?}"
            );
            assert_eq!(
                factor.coverage_dependencies.nodes.len(),
                census.coverage_nodes,
                "{operation:?}",
            );
            assert_eq!(
                factor.coverage_dependencies.edges.len(),
                census.coverage_edges,
                "{operation:?}",
            );

            // Every carrier alternative's layout obligation is part of
            // the operation's own census: an alternative depending on a
            // layout the factor does not state would be an obligation
            // with no owner.
            for requirements in factor.relation_cases.values() {
                for alternative in &requirements.carrier_assignments {
                    for layout in &alternative.layout {
                        assert!(factor.layout_requirements.contains(layout), "{operation:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn no_analyzed_pilot_value_names_a_sponsor_amount() {
    // Sponsor erasure means the amount is absent, not secret. The
    // traversal covers the three places a value could reappear: a
    // relation's source rows, an operation's layout obligations, and
    // the coverage projections.
    for accepted in [&COMPACT_ASH, &TRANSFER_LIVE, &COMBINED] {
        let projection = accepted.program.project();

        for analysis in projection.proof_plans.values() {
            for bundle in analysis.relation_requirements.values() {
                for row in &bundle.source_requirements {
                    assert!(!is_sponsor_amount_operand(row.operand.role()));
                }
            }

            for factor in analysis.operations.values() {
                for layout in &factor.layout_requirements {
                    assert!(!names_sponsor_amount(layout));
                }

                for requirements in factor.relation_cases.values() {
                    for row in &requirements.active_sources {
                        assert!(!is_sponsor_amount_operand(row.operand.role()));
                    }

                    for alternative in &requirements.carrier_assignments {
                        for layout in &alternative.layout {
                            assert!(!names_sponsor_amount(layout));
                        }
                    }
                }
            }
        }
    }
}

// --- §19.3 combined scoped program ---

#[test]
fn the_combined_program_analyzes_exactly_the_two_pilot_operations() {
    assert_eq!(
        COMBINED.scope(),
        BTreeSet::from([OperationId::CompactAsh, OperationId::TransferLive]),
    );

    for analysis in COMBINED.program.proof_plans.values() {
        assert_eq!(
            analysis.operations.keys().copied().collect::<BTreeSet<_>>(),
            COMBINED.scope(),
        );
    }
}

#[test]
fn the_combined_program_reports_eleven_missing_architecture_operations() {
    // Derived from the typed architecture census, never from a
    // written-down remainder: a fourteenth architecture operation must
    // change this number rather than leave the claim stale.
    let census = architecture::ARCHITECTURE
        .operations
        .iter()
        .map(|operation| operation.id)
        .collect::<BTreeSet<_>>();

    let ArchitectureScopeStatus::Partial { missing } = &COMBINED.program.architecture_scope else {
        panic!("the pilot scope does not cover the architecture");
    };

    assert_eq!(
        missing,
        &census.difference(&COMBINED.scope()).copied().collect()
    );
    assert_eq!(missing.len(), 11);
    assert_eq!(census.len(), 13);
}

#[test]
fn the_combined_plan_set_is_the_exact_unconstrained_feasible_set() {
    let expected = enumerate_feasible_plans(&COMBINED.input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        COMBINED
            .program
            .proof_plans
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected,
    );

    // The combined plan set is the product of the two operations'
    // independent representation decisions — four plans, each fixing
    // one mode per operation.
    assert_eq!(expected.len(), 4);
    assert_eq!(
        expected.len(),
        COMPACT_ASH.program.proof_plans.len() * TRANSFER_LIVE.program.proof_plans.len(),
    );

    for plan in &expected {
        // One decision per operation — counted as decisions, not as
        // distinct modes: the plan that chooses `Explicit` for both
        // operations made two decisions that happen to agree.
        assert_eq!(
            plan.representations
                .keys()
                .map(|choice| choice.operation)
                .collect::<BTreeSet<_>>(),
            COMBINED.scope(),
        );
        assert_eq!(plan.representations.len(), 2);
    }
}

#[test]
fn the_combined_factors_equal_the_single_scope_analyses() {
    // The factorization claim, stated at its strongest: analyzing two
    // operations together produces exactly the analyses of each
    // operation alone under the matching representation decision.
    // Nothing about compact ASH changes because live transfer is in
    // scope, which is what makes the stored factors a complete
    // description of the product.
    for (plan, analysis) in &COMBINED.program.proof_plans {
        for pilot in pilots() {
            let operation = pilot.operation();
            let mode = Accepted::mode_of(plan, operation);
            let single = pilot
                .program
                .proof_plans
                .iter()
                .find(|(candidate, _)| Accepted::mode_of(candidate, operation) == mode)
                .map(|(_, single)| single)
                .expect("a single-scope plan with the same representation");

            assert_eq!(
                analysis.operations[&operation], single.operations[&operation],
                "{operation:?} under {mode:?}",
            );
        }
    }
}

#[test]
fn the_combined_program_stores_no_cross_operation_dependency() {
    for analysis in COMBINED.program.proof_plans.values() {
        for (operation, factor) in &analysis.operations {
            assert_eq!(factor.operation, *operation);

            // Every relation-case, placement, layout, coverage key, and
            // dependency symbol belongs to the operation that stores
            // it. A symbol naming another operation would make one
            // factor's validity depend on another's, and the product
            // semantics the analyzed value relies on would no longer
            // hold.
            for key in factor.relation_cases.keys() {
                assert_eq!(key.relation.operation(), *operation);
                assert_eq!(key.case.operation, *operation);
            }

            for placement in &factor.feasible_placements {
                for key in placement.assignments.keys() {
                    assert_eq!(key.relation.operation(), *operation);
                    assert_eq!(key.case.operation, *operation);
                }
            }

            for node in &factor.coverage_dependencies.nodes {
                assert_eq!(node.id.operation(), *operation);
            }

            for edge in &factor.coverage_dependencies.edges {
                assert_eq!(edge.source.operation(), *operation);
                assert_eq!(edge.target.operation(), *operation);
            }
        }

        // No combined placement product is stored: the two factors are
        // the whole placement content, and their product is far larger
        // than what the analyzed value holds.
        let stored = analysis
            .operations
            .values()
            .map(|factor| factor.feasible_placements.len())
            .collect::<Vec<_>>();

        assert_eq!(stored, vec![216, 216]);
        assert_eq!(stored.iter().sum::<usize>(), 432);
        assert_eq!(stored.iter().product::<usize>(), 46_656);
    }
}

#[test]
fn the_combined_program_selects_no_target() {
    // The retained plan set is the unconstrained one. Proving that
    // needs a pruning comparison rather than an inspection: a plan set
    // that had already been narrowed to some target's capabilities
    // would look like a perfectly good plan set on its own.
    let mut without_confidential = COMBINED.capabilities();

    assert!(without_confidential.remove(&RequiredCapability::ConfidentialValueConservation));

    let pruned = enumerate_feasible_plans(
        &COMBINED.input,
        &CapabilityView::Available(without_confidential),
    )
    .expect("feasible plans under a constrained view")
    .candidates;

    // Withdrawing one capability removes the plans that needed it, so
    // the analyzed set is the wider unconstrained one.
    assert!(pruned.len() < COMBINED.program.proof_plans.len());
    assert_ne!(pruned, [] as [ProofPlanCandidate; 0]);

    for plan in &pruned {
        assert!(COMBINED.program.proof_plans.contains_key(plan));
    }
}

#[test]
fn the_combined_program_requires_unresolved_external_evidence_with_no_verdict() {
    let evidence = &COMBINED.program.required_external_evidence;

    // One whole-transaction substrate obligation per analyzed
    // operation. The exhaustive destructuring is the point: the typed
    // requirement carries an operation and an asset and has no field a
    // verdict, report digest, or discharge status could live in, so
    // "required and unresolved" is a property of the type rather than
    // of this assertion.
    let operations = evidence
        .iter()
        .filter_map(|requirement| match requirement {
            ExternalEvidenceRequirement::SubstrateConservation { operation, asset } => {
                assert_eq!(*asset, AssetId::Lbtc);

                Some(*operation)
            }
            ExternalEvidenceRequirement::ConfidentialValueConservation { .. }
                | ExternalEvidenceRequirement::OperatorAuthorization { .. } => None,
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(operations, COMBINED.scope());

    // The confidential class is counted separately and pinned by the
    // operations whose plans hold a protocol amount as a commitment, so
    // the total below cannot absorb a stray requirement of either class.
    let confidential = evidence
        .iter()
        .filter_map(|requirement| match requirement {
            ExternalEvidenceRequirement::ConfidentialValueConservation { operation, asset } => {
                assert_eq!(*asset, AssetId::U);

                Some(*operation)
            }
            ExternalEvidenceRequirement::SubstrateConservation { .. }
            | ExternalEvidenceRequirement::OperatorAuthorization { .. } => None,
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(evidence.len(), operations.len() + confidential.len());

    // Deployment lifecycle: every plan retains the exits declared
    // outside the compiler scope rather than reporting completeness.
    let mut exits = BTreeSet::new();

    for analysis in COMBINED.program.proof_plans.values() {
        let LifecycleCompleteness::Incomplete { obligations } = &analysis.lifecycle else {
            panic!("a pilot plan claims a complete deployment lifecycle");
        };

        exits.extend(obligations.iter().map(|obligation| obligation.exit));
    }

    assert_eq!(
        exits,
        BTreeSet::from([OperationId::Burn, OperationId::Clear, OperationId::Redeem]),
    );
}

// --- §19.3 compiler identity ---

/// A compile-time probe for a hashable analyzed value.
///
/// The inherent method applies only when `T` can be hashed; when it
/// cannot, method resolution falls through to the trait's default. A
/// derived `Hash` on any analyzed type would therefore turn this from
/// `false` to `true` and fail the acceptance below.
struct IdentityProbe<T>(PhantomData<T>);

trait NoMintedIdentity {
    fn identity_minted(&self) -> bool {
        false
    }
}

impl<T> NoMintedIdentity for IdentityProbe<T> {}

impl<T: std::hash::Hash> IdentityProbe<T> {
    // Deliberately unreachable, and the expectation is the second lock:
    // if any analyzed type gains a `Hash` implementation this method
    // becomes the resolved one, the expectation goes unfulfilled, and
    // the crate stops compiling cleanly — before the assertion below
    // has to catch it at run time.
    #[expect(dead_code, reason = "no analyzed type is hashable (Guide-7 §19.3)")]
    #[expect(
        clippy::unused_self,
        reason = "the receiver is what selects between the two implementations"
    )]
    fn identity_minted(&self) -> bool {
        true
    }
}

#[test]
fn the_analyzed_program_mints_no_compiler_identity() {
    // Two independent proofs, both structural rather than textual.
    //
    // First, exhaustive destructuring. Every field of the stable
    // projection is named here, so a digest, plan hash, or report
    // identity added to any of these types stops this test compiling
    // rather than silently gaining a consumer. The one hash reachable
    // anywhere below is the architecture binding's own semantic hash,
    // which the analysis *binds* as the identity of its source; the
    // compiler mints no identity of its own.
    let ScopedAnalyzedProgramProjection {
        source:
            AnalyzedSource {
                architecture,
                realization,
                compilation_scope,
            },
        foundation,
        architecture_scope,
        proof_plans,
        required_external_evidence,
    } = COMBINED.program.project();

    assert_eq!(&architecture, COMBINED.input.architecture_binding());
    assert_eq!(realization, COMBINED.input.realization().project());
    assert_eq!(&compilation_scope, COMBINED.input.scope());
    assert_ne!(
        foundation.relations.nodes,
        [] as [crate::relation::CompilerRelationNodeProjection; 0]
    );
    assert!(matches!(
        architecture_scope,
        ArchitectureScopeStatus::Partial { .. },
    ));
    // Two whole-transaction substrate obligations, one per analyzed
    // operation, and one confidential obligation for the live transfer,
    // whose private plan holds its protocol amounts as commitments.
    assert_eq!(required_external_evidence.len(), 3);

    for analysis in proof_plans.values() {
        let AnalyzedProofPlanProjection {
            relation_requirements,
            operations,
            lifecycle,
        } = analysis;

        assert!(!relation_requirements.is_empty());
        assert!(matches!(
            lifecycle,
            LifecycleCompleteness::Incomplete { .. }
        ));

        for factor in operations.values() {
            let AnalyzedOperation {
                operation,
                execution_cases,
                relation_cases,
                feasible_placements,
                layout_requirements,
                coverage,
                coverage_dependencies,
            } = factor;

            assert_eq!(coverage.operation, *operation);
            assert_eq!(execution_cases.len(), 2);
            assert!(!relation_cases.is_empty());
            assert!(!feasible_placements.is_empty());
            assert!(!layout_requirements.is_empty());
            assert_ne!(
                coverage_dependencies.nodes,
                [] as [crate::coverage_graph::CoverageNode; 0]
            );
        }
    }

    // Second, hashability. An identity has to be mintable before it can
    // be minted, and none of these types can be hashed.
    assert!(!IdentityProbe::<ScopedAnalyzedProgramProjection>(PhantomData).identity_minted());
    assert!(!IdentityProbe::<AnalyzedProofPlanProjection>(PhantomData).identity_minted());
    assert!(!IdentityProbe::<AnalyzedOperation>(PhantomData).identity_minted());
    assert!(!IdentityProbe::<ScopedAnalyzedProgram>(PhantomData).identity_minted());
}
