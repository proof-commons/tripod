//! The independent mapping oracle (Guide-8 §21.4).
//!
//! The table below was written from the Guide-8 Appendix E.3 mapping
//! and the reviewed target's own capability statuses, not from the
//! production match. It calls no helper in `capability.rs` except the
//! public assessment entry point it is checking, so a mistake copied
//! into the production table does not silently reproduce itself here.
//! The duplication is the method: the mapping *is* the property under
//! test, and a mapping compared only against itself agrees with itself.

use std::collections::BTreeSet;

use compiler::target::RequiredCapability;
use target_elements::{ElementsCapability, TargetEvidenceRequirementId};

use super::reviewed_target;
use crate::capability::{AssessmentDisposition, BackendFoundationRequirement, assess_capability};

/// One independently stated expectation for the reviewed target.
struct Expected {
    required: RequiredCapability,
    disposition: AssessmentDisposition,
    primitives: &'static [ElementsCapability],
    structural: &'static [BackendFoundationRequirement],
    evidence: &'static [TargetEvidenceRequirementId],
}

/// The expected assessment of every compiler capability.
///
/// The three authorization rows are written out separately rather than
/// grouped. They coincide today, and a table that hid the coincidence
/// behind a shared row would stop noticing if one of them stopped
/// coinciding.
#[expect(
    clippy::too_many_lines,
    reason = "the whole census, one row each; that is what an oracle is"
)]
fn oracle() -> Vec<Expected> {
    use AssessmentDisposition as D;
    use BackendFoundationRequirement as S;
    use ElementsCapability as P;
    use RequiredCapability as C;
    use TargetEvidenceRequirementId as R;

    // The reviewed contract records the sighash construction as
    // incomplete, so every authorization row blocks on it. Expecting a
    // pattern obligation here instead would be this oracle agreeing to
    // paper over the one honest gap the target review left.
    const UNREVIEWED_SIGHASH: &[ElementsCapability] =
        &[P::OutputCommittingSighash, P::InputCommitmentControl];

    vec![
        Expected {
            required: C::AuthenticatedObjectRecognition,
            disposition: D::BackendPatternRequired,
            primitives: &[
                P::InputAssetInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputProgramInspection,
            ],
            structural: &[],
            evidence: &[
                R::EncodingSemantics,
                R::InputIntrospectionSemantics,
                R::OutputIntrospectionSemantics,
            ],
        },
        Expected {
            required: C::AuthenticatedFamilyCardinality,
            disposition: D::BackendStructural,
            primitives: &[
                P::InputCountInspection,
                P::OutputCountInspection,
                P::CurrentInputIndexInspection,
            ],
            structural: &[
                S::CanonicalFamilyLayout,
                S::CompleteFamilyCensus,
                S::CanonicalCoordinator,
            ],
            evidence: &[],
        },
        Expected {
            required: C::AuthenticatedCanonicalPartition,
            disposition: D::BackendPatternRequired,
            primitives: &[
                P::InputCountInspection,
                P::OutputCountInspection,
                P::InputAssetInspection,
                P::InputValueInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputValueInspection,
                P::OutputProgramInspection,
            ],
            structural: &[
                S::CanonicalFamilyLayout,
                S::CompleteAndDisjointProtocolFamilies,
                S::CanonicalCoordinator,
            ],
            evidence: &[
                R::EncodingSemantics,
                R::InputIntrospectionSemantics,
                R::OutputIntrospectionSemantics,
                R::TransactionIntrospectionSemantics,
            ],
        },
        Expected {
            required: C::AuthenticatedOpenFlowPartition,
            disposition: D::BackendPatternRequired,
            // No value inspection, deliberately: the open flow is
            // separated from the sponsor region by role, never by
            // reading an amount.
            primitives: &[
                P::InputCountInspection,
                P::OutputCountInspection,
                P::CurrentInputIndexInspection,
                P::InputAssetInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputProgramInspection,
            ],
            structural: &[
                S::CanonicalFamilyLayout,
                S::CompleteFamilyCensus,
                S::ProtocolSponsorRegionSeparation,
            ],
            evidence: &[
                R::EncodingSemantics,
                R::InputIntrospectionSemantics,
                R::OutputIntrospectionSemantics,
                R::TransactionIntrospectionSemantics,
                R::ConfidentialValueConservation,
            ],
        },
        Expected {
            required: C::AuthenticatedRootEffects,
            disposition: D::BackendPatternRequired,
            primitives: &[
                P::InputOutpointInspection,
                P::InputAssetInspection,
                P::InputProgramInspection,
                P::OutputAssetInspection,
                P::OutputProgramInspection,
                P::StreamingSha256,
                P::TweakVerification,
            ],
            structural: &[S::RootConstructorContinuity],
            evidence: &[
                R::EncodingSemantics,
                R::InputIntrospectionSemantics,
                R::OutputIntrospectionSemantics,
                R::StreamingHashSemantics,
                R::EllipticCurveSemantics,
            ],
        },
        Expected {
            required: C::AuthenticatedProjectionSet,
            disposition: D::BackendPatternRequired,
            primitives: &[P::OutputCountInspection, P::OutputProgramInspection],
            structural: &[S::ProjectionShape],
            evidence: &[
                R::EncodingSemantics,
                R::OutputIntrospectionSemantics,
                R::TransactionIntrospectionSemantics,
            ],
        },
        Expected {
            required: C::ExactPublicAmountArithmetic,
            disposition: D::BackendPatternRequired,
            primitives: &[
                P::SignedFixedWidthArithmetic,
                P::SignedFixedWidthComparison,
                P::ScriptNumberConversion,
                P::ExplicitValueInspection,
            ],
            structural: &[],
            evidence: &[
                R::EncodingSemantics,
                R::ArithmeticSemantics,
                R::ComparisonSemantics,
                R::ConversionSemantics,
            ],
        },
        Expected {
            required: C::ConfidentialValueConservation,
            disposition: D::ExternalEvidenceRequired,
            primitives: &[],
            structural: &[],
            evidence: &[R::ConfidentialValueConservation],
        },
        Expected {
            required: C::OwnerAuthorization,
            disposition: D::MissingTargetPrimitives,
            primitives: UNREVIEWED_SIGHASH,
            structural: &[],
            evidence: &[],
        },
        Expected {
            required: C::OperatorAuthorization,
            disposition: D::MissingTargetPrimitives,
            primitives: UNREVIEWED_SIGHASH,
            structural: &[],
            evidence: &[],
        },
        Expected {
            required: C::RefundAuthorization,
            disposition: D::MissingTargetPrimitives,
            primitives: UNREVIEWED_SIGHASH,
            structural: &[],
            evidence: &[],
        },
        Expected {
            required: C::PublicConstructibility,
            disposition: D::BackendStructural,
            primitives: &[],
            structural: &[
                S::CanonicalFamilyLayout,
                S::PublicConstructionData,
                S::SecretFreePermissionlessPath,
            ],
            evidence: &[],
        },
        Expected {
            required: C::WholeTransactionValueConservation,
            disposition: D::ExternalEvidenceRequired,
            primitives: &[],
            structural: &[],
            evidence: &[R::ConfidentialValueConservation],
        },
    ]
}

#[test]
fn the_oracle_covers_the_whole_compiler_census_exactly_once() {
    let stated: Vec<_> = oracle().into_iter().map(|row| row.required).collect();
    let distinct: BTreeSet<_> = stated.iter().copied().collect();

    assert_eq!(stated.len(), distinct.len(), "no row is written twice");
    assert_eq!(
        distinct,
        RequiredCapability::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        "the oracle and the compiler census name the same capabilities",
    );
}

#[test]
fn the_production_mapping_matches_the_independent_table() {
    let target = reviewed_target();

    for row in oracle() {
        let projection = assess_capability(&target, row.required).projection();

        assert_eq!(projection.required(), row.required, "{:?}", row.required);
        assert_eq!(
            projection.disposition(),
            row.disposition,
            "{:?}",
            row.required
        );
        assert_eq!(
            projection
                .primitives()
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            row.primitives.iter().copied().collect::<BTreeSet<_>>(),
            "primitives for {:?}",
            row.required,
        );
        assert_eq!(
            projection
                .structural()
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            row.structural.iter().copied().collect::<BTreeSet<_>>(),
            "structural obligations for {:?}",
            row.required,
        );
        assert_eq!(
            projection
                .evidence()
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            row.evidence.iter().copied().collect::<BTreeSet<_>>(),
            "evidence for {:?}",
            row.required,
        );
    }
}

#[test]
fn every_projection_census_is_canonically_ordered() {
    // The oracle above compares sets, so ordering is asserted here
    // instead: a canonical vector projection that merely happened to
    // contain the right members would still be the wrong projection.
    let target = reviewed_target();

    for capability in RequiredCapability::ALL {
        let projection = assess_capability(&target, *capability).projection();

        assert!(
            projection
                .primitives()
                .windows(2)
                .all(|pair| pair[0] < pair[1]),
            "primitives for {capability:?} ascend strictly, so none repeats",
        );
        assert!(
            projection
                .structural()
                .windows(2)
                .all(|pair| pair[0] < pair[1]),
            "structural obligations for {capability:?} ascend strictly",
        );
        assert!(
            projection
                .evidence()
                .windows(2)
                .all(|pair| pair[0] < pair[1]),
            "evidence for {capability:?} ascends strictly",
        );
    }
}

#[test]
fn every_structural_obligation_is_reachable_from_some_capability() {
    // A structural requirement no mapping row ever states would be
    // vocabulary without a consumer, and the next reader could not tell
    // whether it was forgotten or reserved.
    let target = reviewed_target();
    let stated: BTreeSet<_> = RequiredCapability::ALL
        .iter()
        .flat_map(|capability| {
            assess_capability(&target, *capability)
                .projection()
                .structural()
                .to_vec()
        })
        .collect();

    // Every authorization row blocks on the unreviewed sighash, and a
    // blocked assessment states no structural obligation; none of the
    // three carries one in any case.
    assert_eq!(
        stated,
        BackendFoundationRequirement::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
    );
}

#[test]
fn every_named_evidence_requirement_exists_in_the_target_registry() {
    // The adapter never returns a "missing evidence requirement" error
    // because a validated target cannot omit one — the registry census
    // must be complete for the definition to validate. That invariant
    // is what makes the absent error variant honest, so it is checked
    // here rather than assumed.
    let target = reviewed_target();
    let registry = target.definition().definition().evidence_requirements();

    for capability in RequiredCapability::ALL {
        for evidence in assess_capability(&target, *capability)
            .projection()
            .evidence()
        {
            assert!(
                registry.contains_key(evidence),
                "{capability:?} names {evidence:?}, which the target must declare",
            );
        }
    }
}
