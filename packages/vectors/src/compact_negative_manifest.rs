//! Complete compact negative inventory, separate from observations.
//!
//! Keys and collateral come from the validated coverage plan. A missing
//! validation entry remains an obligation, never an observed refusal.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::{
    CarrierAssignmentAlternative, CoverageRequirementId, EvidenceRole, NegativeCoverageRequirement,
    RelationMutation, SponsorCase, TargetCoverageObligation,
};
use tapscript::ConcreteCarrierSite;

use crate::bundle::FixtureBundle;
use crate::matrix::EvidenceBoundary;
use crate::mutation::NegativeMutation;
use crate::violation::{DeclarationLink, resolve_declaration};

/// The three published negative populations; ABI rows are supplemental.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NegativeClass {
    TargetExecutable,
    FirstParty,
    Unreachable,
}

/// Where the declared relation can be answered, not where a mutant failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactNegativeBoundary {
    RuntimeCarrier,
    DependencyAnalysis,
    CompilerAnalysis,
    EmittedStructure,
    ExternalReport,
}

/// An exact entry still needed for a retained obligation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerEntryMissing {
    /// The existing dependency entry selects another refusal for compact ASH.
    CompactUnavailableWitness,
    /// Private compiler plan validator and private analyzed subject.
    CompilerRepresentation,
    /// Private compiler lifecycle validator and private analysis inputs.
    CompilerLifecycle,
    /// A predicate exists, but no typed emitted-path refusal entry exists.
    EmittedSecretFreePath,
    /// No malformed emitted-representation validation entry exists.
    EmittedRepresentation,
    /// No malformed emitted-exit validation entry exists.
    EmittedLifecycle,
    /// No compact substrate report consumer supplies this evidence.
    SubstrateReport,
}

impl OwnerEntryMissing {
    /// Existing private entry, or explicitly required future entry.
    #[must_use]
    pub const fn entry(self) -> &'static str {
        match self {
            Self::CompactUnavailableWitness => "compiler::validate_required_dependency",
            Self::CompilerRepresentation => {
                "compiler::operation_plan::validate_target_operation_plan"
            }
            Self::CompilerLifecycle => "compiler::lifecycle::build_lifecycle_analysis",
            Self::EmittedSecretFreePath => "required: tapscript::validate_compact_secret_free_path",
            Self::EmittedRepresentation => {
                "required: tapscript::validate_compact_emitted_representation"
            }
            Self::EmittedLifecycle => "required: tapscript::validate_compact_emitted_lifecycle",
            Self::SubstrateReport => "required: vectors::validate_compact_substrate_report",
        }
    }
}

/// The actual owner, including exact native placement alternatives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NegativeOwner {
    /// Native execution of these linked sites; report validation is not execution.
    NativeCarrier {
        alternatives: BTreeSet<CarrierAssignmentAlternative>,
        sites: BTreeSet<ConcreteCarrierSite>,
    },
    /// `compiler::validate_required_dependency`.
    CompilerRequiredDependency,
    OwnerEntryMissing(OwnerEntryMissing),
}

/// A concrete existing mutation route, without claiming it answers the row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MutationAttempt {
    pub mutation: NegativeMutation,
    pub boundary: EvidenceBoundary,
}

/// One keyed obligation with its class, owner, recipe and retained collateral.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactNegativeRow {
    pub requirement: CoverageRequirementId,
    pub mutation: RelationMutation,
    pub class: NegativeClass,
    pub boundary: CompactNegativeBoundary,
    pub owner: NegativeOwner,
    pub obligation: NegativeCoverageRequirement,
    pub attempts: Vec<MutationAttempt>,
}

impl CompactNegativeRow {
    /// Focused change to the relation subject named by the requirement key.
    /// Native recipes still require accepted controls and exact funded bytes.
    #[must_use]
    pub const fn subject_recipe(&self) -> &'static str {
        use RelationMutation as M;
        match self.mutation {
            M::CardinalityBelowMinimum => "remove family members below the declared minimum",
            M::CardinalityAboveMaximum { .. } => "add family members above the declared ceiling",
            M::UndeclaredObjectFamily => "claim a foreign family inside the operation region",
            M::WrongRecognizedAsset => "replace the named member's asset; retain its object form",
            M::WrongRecognizedObject => "replace the named member's object form; retain its asset",
            M::AmountMismatch => "change the successor sum; retain the intended carrier obligation",
            M::SponsorProtocolOverlap => "claim a protocol member in the sponsor flow",
            M::SponsorMemberUnclaimed => {
                "omit a regional sponsor member from sponsor flow membership"
            }
            M::MissingSponsorAuthorization => "remove the named sponsor member's authorization",
            M::SponsorEnvelopeMultiplicityExceeded => "add a second owned sponsor envelope",
            M::WrongRootEffect => "add a forbidden root effect to the retained transition",
            M::MissingRequiredProjection => "remove the required transition certificate",
            M::ForbiddenProjectionPresent => "add a forbidden specialized projection",
            M::MissingCanonicalDeltaFamily => {
                "remove the required U delta from the owned partition"
            }
            M::UnexpectedCanonicalDeltaFamily => "add an undeclared delta to the owned partition",
            M::DuplicateCanonicalSourceOrDestination => {
                "repeat an owned canonical source or destination"
            }
            M::UndeclaredOpenFlow => "add an undeclared open flow owned by this operation",
            M::PermissionlessPrivateDependency | M::ConstructibilityWitnessUnavailable => {
                "published public dependency versus private availability under compact authorization"
            }
            M::UnsupportedRepresentation => "explicit selection versus private-committed selection",
            M::UnauthenticatedRepresentation => {
                "retained recognition program versus removed representation authentication"
            }
            M::UnexpectedProtocolSecret => {
                "retained protocol program versus inserted authorization instruction"
            }
            M::RequiredLifecycleExitMissing => {
                "retained lifecycle subject versus removal of the exact keyed exit; Clear has no accepting emitted control"
            }
            M::ExternalEvidenceMissing => "retained exact substrate subject with its report absent",
            M::ExternalEvidenceFailed => "retained exact substrate subject with a failed report",
            M::ExternalEvidenceIdentityMismatch => {
                "retained exact substrate subject with another subject's report"
            }
            M::MissingRequiredOwner => "remove the required owner's authorization",
            M::ExpressionPredicateFalse => {
                "change the named expression's operand to falsify its predicate"
            }
        }
    }
}

/// Inventory derivation failed; no incomplete manifest is returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestRefusal {
    UnknownRole,
    UnknownFirstPartyClass,
    MissingCarrier,
    DuplicateRequirement,
    MutationLink,
}

/// Derive the join from the published requirements and linked carrier closure.
///
/// # Errors
/// Returns a typed refusal for an unknown owner, absent carrier, duplicate key,
/// or a concrete mutation declaration that cannot be resolved unambiguously.
pub fn compact_negative_manifest(
    bundle: &FixtureBundle,
) -> Result<BTreeMap<CoverageRequirementId, CompactNegativeRow>, ManifestRefusal> {
    let mut rows = BTreeMap::new();
    for requirement in bundle.plan().coverage() {
        let TargetCoverageObligation::Negative(negative) = &requirement.obligation else {
            continue;
        };
        let (class, boundary, owner) = match &requirement.role {
            EvidenceRole::TargetExecution => {
                let sites = bundle
                    .linked()
                    .carrier_closure()
                    .reachable()
                    .get(&requirement.id.key())
                    .cloned()
                    .ok_or(ManifestRefusal::MissingCarrier)?;
                if requirement.carrier.is_empty() || sites.is_empty() {
                    return Err(ManifestRefusal::MissingCarrier);
                }
                (
                    NegativeClass::TargetExecutable,
                    CompactNegativeBoundary::RuntimeCarrier,
                    NegativeOwner::NativeCarrier {
                        alternatives: requirement.carrier.clone(),
                        sites,
                    },
                )
            }
            EvidenceRole::CompilerAnalysisResult | EvidenceRole::EmittedStructure => {
                let emitted = requirement.role == EvidenceRole::EmittedStructure;
                let (boundary, owner) = first_party_owner(negative.mutation, emitted)?;
                (NegativeClass::FirstParty, boundary, owner)
            }
            EvidenceRole::ExternalReport { .. } => (
                NegativeClass::Unreachable,
                CompactNegativeBoundary::ExternalReport,
                NegativeOwner::OwnerEntryMissing(OwnerEntryMissing::SubstrateReport),
            ),
            EvidenceRole::InactiveCaseAcceptance => return Err(ManifestRefusal::UnknownRole),
        };
        let row = CompactNegativeRow {
            requirement: requirement.id.clone(),
            mutation: negative.mutation,
            class,
            boundary,
            owner,
            obligation: negative.clone(),
            attempts: Vec::new(),
        };
        if rows.insert(requirement.id.clone(), row).is_some() {
            return Err(ManifestRefusal::DuplicateRequirement);
        }
    }
    for &mutation in NegativeMutation::ALL {
        for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
            let declaration = mutation
                .declaration(sponsor)
                .map_err(|_| ManifestRefusal::MutationLink)?;
            if let DeclarationLink::Resolved(id) = resolve_declaration(bundle.plan(), &declaration)
                .map_err(|_| ManifestRefusal::MutationLink)?
            {
                rows.get_mut(&id)
                    .ok_or(ManifestRefusal::MutationLink)?
                    .attempts
                    .push(MutationAttempt {
                        mutation,
                        boundary: declaration.boundary(),
                    });
            }
        }
    }
    Ok(rows)
}

const fn first_party_owner(
    mutation: RelationMutation,
    emitted: bool,
) -> Result<(CompactNegativeBoundary, NegativeOwner), ManifestRefusal> {
    use CompactNegativeBoundary as B;
    use OwnerEntryMissing as M;
    use RelationMutation as R;
    let (boundary, missing) = match (mutation, emitted) {
        (R::PermissionlessPrivateDependency, false) => {
            return Ok((
                B::DependencyAnalysis,
                NegativeOwner::CompilerRequiredDependency,
            ));
        }
        (R::ConstructibilityWitnessUnavailable, false) => {
            (B::DependencyAnalysis, M::CompactUnavailableWitness)
        }
        (R::UnsupportedRepresentation, false) => (B::CompilerAnalysis, M::CompilerRepresentation),
        (R::RequiredLifecycleExitMissing, false) => (B::CompilerAnalysis, M::CompilerLifecycle),
        (R::UnexpectedProtocolSecret, true) => (B::EmittedStructure, M::EmittedSecretFreePath),
        (R::UnauthenticatedRepresentation, true) => (B::EmittedStructure, M::EmittedRepresentation),
        (R::RequiredLifecycleExitMissing, true) => (B::EmittedStructure, M::EmittedLifecycle),
        _ => return Err(ManifestRefusal::UnknownFirstPartyClass),
    };
    Ok((boundary, NegativeOwner::OwnerEntryMissing(missing)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_negative_manifest_matches_every_published_negative() {
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let rows = compact_negative_manifest(&bundle).expect("manifest");
        let expected: BTreeSet<_> = bundle
            .plan()
            .coverage()
            .filter(|r| matches!(r.obligation, TargetCoverageObligation::Negative(_)))
            .map(|r| r.id.clone())
            .collect();
        assert_eq!(rows.keys().cloned().collect::<BTreeSet<_>>(), expected);
        assert_eq!(rows.len(), 74);
        let count = |class| rows.values().filter(|r| r.class == class).count();
        assert_eq!(count(NegativeClass::TargetExecutable), 50);
        assert_eq!(count(NegativeClass::FirstParty), 18);
        assert_eq!(count(NegativeClass::Unreachable), 6);
        let added = rows
            .values()
            .filter(|r| r.mutation == RelationMutation::SponsorMemberUnclaimed)
            .count();
        assert_eq!(added, 2);
        assert_eq!(rows.len() - added, 72);
        assert_eq!(count(NegativeClass::TargetExecutable) - added, 48);
        assert_eq!(count(NegativeClass::TargetExecutable) - added - 1, 47);
        assert_eq!(count(NegativeClass::TargetExecutable) - 1, 49);
    }

    #[test]
    fn compact_negative_manifest_pins_missing_owners() {
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let rows = compact_negative_manifest(&bundle).expect("manifest");
        let missing: Vec<_> = rows
            .values()
            .filter(|r| matches!(r.owner, NegativeOwner::OwnerEntryMissing(_)))
            .collect();
        assert_eq!(missing.len(), 22);
        assert_eq!(
            missing
                .iter()
                .filter(|r| r.class == NegativeClass::FirstParty)
                .count(),
            16
        );
        assert_eq!(
            missing
                .iter()
                .filter(|r| r.class == NegativeClass::Unreachable)
                .count(),
            6
        );
        for row in missing {
            let NegativeOwner::OwnerEntryMissing(owner) = row.owner else {
                unreachable!()
            };
            assert_ne!(owner.entry(), "");
            assert_ne!(row.subject_recipe(), "");
        }
    }

    #[test]
    fn compact_negative_manifest_has_unique_routes_and_preserves_collateral() {
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let rows = compact_negative_manifest(&bundle).expect("manifest");
        for (id, row) in &rows {
            let published = bundle.plan().coverage_requirement(id).expect("published");
            let TargetCoverageObligation::Negative(negative) = &published.obligation else {
                panic!("negative")
            };
            assert_eq!(row.requirement, *id);
            assert_eq!(row.mutation, negative.mutation);
            assert_eq!(row.obligation.collateral, negative.collateral);
            assert!(row.attempts.len() <= 1);
            if let NegativeOwner::NativeCarrier {
                alternatives,
                sites,
            } = &row.owner
            {
                assert_eq!(alternatives, &published.carrier);
                assert_eq!(
                    Some(sites),
                    bundle.linked().carrier_closure().reachable().get(&id.key())
                );
            }
        }
        let attempts: Vec<_> = rows.values().flat_map(|r| &r.attempts).collect();
        assert_eq!(attempts.len(), 4);
        assert_eq!(
            attempts
                .iter()
                .filter(|a| a.boundary == EvidenceBoundary::ConsensusRejectionBeforeScript)
                .count(),
            2
        );
    }
}
