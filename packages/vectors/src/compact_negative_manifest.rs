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

/// Historical missing-entry inventory, retained after explicit resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerEntryMissing {
    /// The existing dependency entry selects another refusal for compact ASH.
    CompactUnavailableWitness,
    /// Resolved by the immutable offered-representation comparison.
    CompilerRepresentation,
    /// Resolved by the offered-lifecycle-projection comparison.
    CompilerLifecycle,
    /// Resolved by the typed owned-envelope multiplicity comparison.
    CompilerSponsorMultiplicity,
    /// Resolved by the typed emitted secret-free-path comparison.
    EmittedSecretFreePath,
    /// Resolved by the typed recognition-fragment comparison.
    EmittedRepresentation,
    /// Resolved by the typed constructor and declaration comparison.
    EmittedLifecycle,
    /// Resolved by the prepared substrate-report comparison.
    SubstrateReport,
}

impl OwnerEntryMissing {
    /// Supplied entry; the unavailable-witness class still needs its formal argument.
    #[must_use]
    pub const fn entry(self) -> &'static str {
        match self {
            Self::CompactUnavailableWitness => "compiler::validate_required_dependency",
            Self::CompilerRepresentation => {
                "compiler::operation_plan::validate_compact_representation"
            }
            Self::CompilerLifecycle => "compiler::validate_compact_lifecycle_projection",
            Self::CompilerSponsorMultiplicity => "compiler::validate_sponsor_envelope_multiplicity",
            Self::EmittedSecretFreePath => "tapscript::validate_compact_secret_free_path",
            Self::EmittedRepresentation => "tapscript::validate_compact_emitted_representation",
            Self::EmittedLifecycle => "tapscript::validate_compact_emitted_lifecycle",
            Self::SubstrateReport => "vectors::validate_compact_substrate_report",
        }
    }
}

/// Supplied comparison routes, distinct from historical missing-entry names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactValidationEntry {
    /// Required-dependency comparison supporting the formal argument.
    RequiredDependency,
    /// Offered compact representation comparison.
    Representation,
    /// Offered lifecycle projection comparison.
    LifecycleProjection,
    /// Owned sponsor-envelope multiplicity comparison.
    SponsorMultiplicity,
    /// Emitted secret-free-path comparison.
    SecretFreePath,
    /// Emitted recognition-fragment comparison.
    Recognition,
    /// Emitted constructor and lifecycle declaration comparison.
    LifecycleDeclaration,
    /// Prepared substrate-report comparison.
    SubstrateComparison,
}

impl OwnerEntryMissing {
    /// Exhaustive resolution of the historical entry inventory.
    #[must_use]
    pub const fn supplied(self) -> CompactValidationEntry {
        use CompactValidationEntry as E;
        match self {
            Self::CompactUnavailableWitness => E::RequiredDependency,
            Self::CompilerRepresentation => E::Representation,
            Self::CompilerLifecycle => E::LifecycleProjection,
            Self::CompilerSponsorMultiplicity => E::SponsorMultiplicity,
            Self::EmittedSecretFreePath => E::SecretFreePath,
            Self::EmittedRepresentation => E::Recognition,
            Self::EmittedLifecycle => E::LifecycleDeclaration,
            Self::SubstrateReport => E::SubstrateComparison,
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
    /// A callable comparison; availability alone supplies no closure evidence.
    Supplied(CompactValidationEntry),
    /// The dependency entry supports the checked argument of ruling 137.
    FormallyResolved(CompactValidationEntry),
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
                "retained lifecycle subject versus removal of the exact keyed exit or outstanding Clear declaration"
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
            EvidenceRole::CompilerAnalysisResult
            | EvidenceRole::EmittedStructure
            | EvidenceRole::ModelScope => {
                let emitted = requirement.role == EvidenceRole::EmittedStructure;
                let (boundary, owner) = first_party_owner(negative.mutation, emitted)?;
                (NegativeClass::FirstParty, boundary, owner)
            }
            EvidenceRole::ExternalReport { .. } => (
                NegativeClass::Unreachable,
                CompactNegativeBoundary::ExternalReport,
                NegativeOwner::Supplied(OwnerEntryMissing::SubstrateReport.supplied()),
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
        (R::SponsorEnvelopeMultiplicityExceeded, false) => {
            (B::CompilerAnalysis, M::CompilerSponsorMultiplicity)
        }
        (R::UnexpectedProtocolSecret, true) => (B::EmittedStructure, M::EmittedSecretFreePath),
        (R::UnauthenticatedRepresentation, true) => (B::EmittedStructure, M::EmittedRepresentation),
        (R::RequiredLifecycleExitMissing, true) => (B::EmittedStructure, M::EmittedLifecycle),
        _ => return Err(ManifestRefusal::UnknownFirstPartyClass),
    };
    let resolved = if matches!(missing, M::CompactUnavailableWitness) {
        NegativeOwner::FormallyResolved(missing.supplied())
    } else {
        NegativeOwner::Supplied(missing.supplied())
    };
    Ok((boundary, resolved))
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
        // Before scope-only multiplicity: 50 target and 18 first-party rows.
        assert_eq!(count(NegativeClass::TargetExecutable), 48);
        assert_eq!(count(NegativeClass::FirstParty), 20);
        assert_eq!(count(NegativeClass::Unreachable), 6);
        let added = rows
            .values()
            .filter(|r| r.mutation == RelationMutation::SponsorMemberUnclaimed)
            .count();
        assert_eq!(added, 2);
        assert_eq!(rows.len() - added, 72);
        // Before scope-only multiplicity: 48, 47, 49 respectively.
        assert_eq!(count(NegativeClass::TargetExecutable) - added, 46);
        assert_eq!(count(NegativeClass::TargetExecutable) - added - 1, 45);
        assert_eq!(count(NegativeClass::TargetExecutable) - 1, 47);
        let scope: Vec<_> = rows
            .values()
            .filter(|r| r.mutation == RelationMutation::SponsorEnvelopeMultiplicityExceeded)
            .collect();
        assert_eq!(scope.len(), 2);
        for row in scope {
            assert_eq!(row.class, NegativeClass::FirstParty);
            assert_eq!(row.boundary, CompactNegativeBoundary::CompilerAnalysis);
            assert_eq!(
                row.owner,
                NegativeOwner::Supplied(CompactValidationEntry::SponsorMultiplicity)
            );
        }
    }

    #[test]
    fn compact_negative_manifest_pins_missing_owners() {
        use CompactValidationEntry as E;
        use OwnerEntryMissing as M;
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let rows = compact_negative_manifest(&bundle).expect("manifest");
        let missing: Vec<_> = rows
            .values()
            .filter(|r| matches!(r.owner, NegativeOwner::OwnerEntryMissing(_)))
            .collect();
        // Before scope-only multiplicity: 22 missing owners, 16 first-party.
        // Before compact closure: 24 = 18 first-party + 6 external.
        assert_eq!(missing.len(), 0);
        assert_eq!(
            missing
                .iter()
                .filter(|r| r.class == NegativeClass::FirstParty)
                .count(),
            0
        );
        assert_eq!(
            missing
                .iter()
                .filter(|r| r.class == NegativeClass::Unreachable)
                .count(),
            0
        );
        let resolved: Vec<_> = rows
            .values()
            .filter(|r| {
                matches!(
                    r.owner,
                    NegativeOwner::Supplied(_) | NegativeOwner::FormallyResolved(_)
                )
            })
            .collect();
        assert_eq!(resolved.len(), 24);
        for row in resolved {
            let (NegativeOwner::Supplied(entry) | NegativeOwner::FormallyResolved(entry)) =
                row.owner
            else {
                unreachable!()
            };
            let owner = match entry {
                E::RequiredDependency => M::CompactUnavailableWitness,
                E::Representation => M::CompilerRepresentation,
                E::LifecycleProjection => M::CompilerLifecycle,
                E::SponsorMultiplicity => M::CompilerSponsorMultiplicity,
                E::SecretFreePath => M::EmittedSecretFreePath,
                E::Recognition => M::EmittedRepresentation,
                E::LifecycleDeclaration => M::EmittedLifecycle,
                E::SubstrateComparison => M::SubstrateReport,
            };
            assert_ne!(owner.entry(), "");
            assert_ne!(row.subject_recipe(), "");
        }
    }

    #[test]
    fn all_eight_historical_missing_entries_have_explicit_resolutions() {
        use CompactValidationEntry as E;
        use OwnerEntryMissing as M;
        let mappings = [
            (
                M::CompactUnavailableWitness,
                E::RequiredDependency,
                "compiler::validate_required_dependency",
            ),
            (
                M::CompilerRepresentation,
                E::Representation,
                "compiler::operation_plan::validate_compact_representation",
            ),
            (
                M::CompilerLifecycle,
                E::LifecycleProjection,
                "compiler::validate_compact_lifecycle_projection",
            ),
            (
                M::CompilerSponsorMultiplicity,
                E::SponsorMultiplicity,
                "compiler::validate_sponsor_envelope_multiplicity",
            ),
            (
                M::EmittedSecretFreePath,
                E::SecretFreePath,
                "tapscript::validate_compact_secret_free_path",
            ),
            (
                M::EmittedRepresentation,
                E::Recognition,
                "tapscript::validate_compact_emitted_representation",
            ),
            (
                M::EmittedLifecycle,
                E::LifecycleDeclaration,
                "tapscript::validate_compact_emitted_lifecycle",
            ),
            (
                M::SubstrateReport,
                E::SubstrateComparison,
                "vectors::validate_compact_substrate_report",
            ),
        ];
        assert_eq!(mappings.len(), 8);
        for (historical, supplied, entry) in mappings {
            assert_eq!(historical.supplied(), supplied);
            assert_eq!(historical.entry(), entry);
        }
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let rows = compact_negative_manifest(&bundle).expect("manifest");
        assert_eq!(
            rows.values()
                .filter(|r| matches!(r.owner, NegativeOwner::Supplied(_)))
                .count(),
            22
        );
        assert_eq!(
            rows.values()
                .filter(|r| matches!(
                    r.owner,
                    NegativeOwner::FormallyResolved(E::RequiredDependency)
                ))
                .count(),
            2
        );
        assert_eq!(
            rows.values()
                .filter(|r| matches!(r.owner, NegativeOwner::CompilerRequiredDependency))
                .count(),
            2
        );
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
