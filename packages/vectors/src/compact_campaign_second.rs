//! Additional compact subjects and candidate-specific closure arguments.

use std::collections::BTreeMap;

use architecture::ObjectId;
use compiler::operation_plan::{CoverageRequirementId, RelationMutation as M, SponsorCase};
use realization::{RelationKind as K, RelationSubject as S, TransactionSide as Side};

use crate::compact_campaign::{ByteRecipe as B, CampaignRow, ControlFamily, OperationCensus};

/// A formal closure states its ground, never an observed target outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Closure {
    SponsorAuthorizationCoversCoordinate,
    NoIndependentDeltaFamilyCoordinate,
    NoIndependentCanonicalMembershipCoordinate,
    NoIndependentOpenFlowClaimCoordinate,
    EmptySponsorClaimDomain,
    NoIndependentSponsorMembershipCoordinate,
    NoIndependentRootEffectCoordinate,
    DerivedTransitionCertificateHasNoRemovalCoordinate,
    NoIndependentSpecializedProjectionCoordinate,
}

impl Closure {
    /// The closure kind the record names beside the reason.
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            Self::SponsorAuthorizationCoversCoordinate => "discharged by sponsor authorization",
            _ => "absence of coordinate",
        }
    }

    #[must_use]
    pub const fn test(self) -> &'static str {
        match self {
            Self::SponsorAuthorizationCoversCoordinate => {
                "sponsor_authorization_covers_uninspected_programs"
            }
            Self::NoIndependentDeltaFamilyCoordinate => {
                "compact_delta_family_has_no_independent_carrier_coordinate"
            }
            Self::NoIndependentCanonicalMembershipCoordinate => {
                "compact_canonical_membership_is_derived_once_per_position"
            }
            Self::NoIndependentOpenFlowClaimCoordinate => {
                "compact_open_flow_claims_have_no_carrier_encoding"
            }
            Self::EmptySponsorClaimDomain => "compact_absent_sponsor_claim_domain_is_empty",
            Self::NoIndependentSponsorMembershipCoordinate => {
                "compact_sponsor_membership_is_fixed_by_disjoint_positions"
            }
            Self::NoIndependentRootEffectCoordinate => {
                "compact_root_effects_have_no_independent_carrier_coordinate"
            }
            Self::DerivedTransitionCertificateHasNoRemovalCoordinate => {
                "compact_transition_certificate_is_derived_not_serialized"
            }
            Self::NoIndependentSpecializedProjectionCoordinate => {
                "compact_specialized_projection_has_no_carrier_coordinate"
            }
        }
    }
}

/// Select only exact declaration/mutation pairs of this candidate.
#[must_use]
pub fn closure(row: &CampaignRow) -> Option<Closure> {
    let id = &row.manifest.requirement;
    Some(
        match (
            id.relation.kind(),
            id.relation.subject(),
            row.manifest.mutation,
        ) {
            (
                K::Recognition,
                S::ObjectFamily {
                    object: ObjectId::PlainLbtc,
                    ..
                },
                M::WrongRecognizedObject,
            ) if id.case.sponsor == SponsorCase::Present => {
                Closure::SponsorAuthorizationCoversCoordinate
            }
            (
                K::CanonicalDeltaPolicy,
                S::Operation,
                M::MissingCanonicalDeltaFamily | M::UnexpectedCanonicalDeltaFamily,
            ) => Closure::NoIndependentDeltaFamilyCoordinate,
            (K::CanonicalDeltaPolicy, S::Operation, M::DuplicateCanonicalSourceOrDestination) => {
                Closure::NoIndependentCanonicalMembershipCoordinate
            }
            (K::OpenFlowPolicy, S::Operation, M::UndeclaredOpenFlow) => {
                Closure::NoIndependentOpenFlowClaimCoordinate
            }
            (
                K::SponsorIsolation,
                S::Sponsor,
                M::SponsorProtocolOverlap | M::SponsorMemberUnclaimed,
            ) => {
                if id.case.sponsor == SponsorCase::Absent {
                    Closure::EmptySponsorClaimDomain
                } else {
                    Closure::NoIndependentSponsorMembershipCoordinate
                }
            }
            (K::RootPolicy, S::Operation, M::WrongRootEffect) => {
                Closure::NoIndependentRootEffectCoordinate
            }
            (K::ProjectionPolicy, S::Operation, M::MissingRequiredProjection) => {
                Closure::DerivedTransitionCertificateHasNoRemovalCoordinate
            }
            (K::ProjectionPolicy, S::Operation, M::ForbiddenProjectionPresent) => {
                Closure::NoIndependentSpecializedProjectionCoordinate
            }
            _ => return None,
        },
    )
}

/// Derive the second schedule without altering the first schedule's roster.
///
/// # Errors
/// Returns the manifest's refusal when its exact key join fails.
pub fn roster(
    bundle: &crate::bundle::FixtureBundle,
) -> Result<
    BTreeMap<CoverageRequirementId, CampaignRow>,
    crate::compact_negative_manifest::ManifestRefusal,
> {
    let mut rows = crate::compact_campaign::roster(bundle)?;
    for row in rows.values_mut() {
        let id = &row.manifest.requirement;
        let recipe = match (
            id.relation.kind(),
            id.relation.subject(),
            row.manifest.mutation,
        ) {
            (
                K::Recognition,
                S::ObjectFamily {
                    side: Side::Input,
                    object: ObjectId::Ash,
                },
                M::WrongRecognizedAsset,
            ) => B::AshInputAsset,
            (
                K::Recognition,
                S::ObjectFamily {
                    side: Side::Input,
                    object: ObjectId::Ash,
                },
                M::WrongRecognizedObject,
            )
            | (
                K::AllowedObjectFamilies,
                S::TransactionSide { side: Side::Input },
                M::UndeclaredObjectFamily,
            ) => B::AshInputObject,
            (
                K::Recognition,
                S::ObjectFamily {
                    side: Side::Input,
                    object: ObjectId::PlainLbtc,
                },
                M::WrongRecognizedAsset,
            ) => B::SponsorInputAsset,
            (
                K::Recognition,
                S::ObjectFamily {
                    side: Side::Output,
                    object: ObjectId::Ash,
                },
                M::WrongRecognizedAsset,
            ) => B::SuccessorAsset,
            (
                K::Recognition,
                S::ObjectFamily {
                    side: Side::Output,
                    object: ObjectId::PlainLbtc,
                },
                M::WrongRecognizedAsset,
            ) => B::ChangeAsset,
            (
                K::Cardinality,
                S::ObjectFamily {
                    side: Side::Input,
                    object: ObjectId::PlainLbtc,
                },
                M::CardinalityAboveMaximum { .. },
            ) => B::TwoSponsors,
            _ => continue,
        };
        row.recipe = Ok(recipe);
    }
    Ok(rows)
}

#[must_use]
pub const fn additional(recipe: B) -> bool {
    matches!(
        recipe,
        B::AshInputAsset
            | B::AshInputObject
            | B::SponsorInputAsset
            | B::SuccessorAsset
            | B::ChangeAsset
            | B::TwoSponsors
    )
}

/// Additional violated coordinates are retained alongside the intended key.
#[must_use]
pub const fn collateral(recipe: B) -> &'static str {
    match recipe {
        B::AshInputAsset => {
            "output cardinality; output closure; U aggregate conservation; canonical partition"
        }
        B::AshInputObject => "input recognition; input closure; authenticated family membership",
        B::SponsorInputAsset => {
            "input cardinality; output cardinality; output closure; sponsor isolation"
        }
        B::SuccessorAsset => {
            "input cardinality; input closure; output cardinality; output closure; U aggregate conservation; canonical partition"
        }
        B::ChangeAsset => {
            "input cardinality; input closure; output cardinality; output closure; sponsor isolation"
        }
        B::TwoSponsors => "compiled transaction-wide input count",
        _ => "first archive retains its original subject obligations",
    }
}

#[must_use]
pub fn staged(rows: &BTreeMap<CoverageRequirementId, CampaignRow>) -> Vec<&CampaignRow> {
    rows.values()
        .filter(|row| row.recipe.is_ok_and(additional))
        .collect()
}

#[must_use]
pub fn sponsor_coins(rows: &BTreeMap<CoverageRequirementId, CampaignRow>) -> usize {
    ControlFamily::ALL
        .iter()
        .filter(|family| family.sponsored)
        .count()
        + staged(rows)
            .iter()
            .map(|row| {
                usize::from(row.control.sponsored) + usize::from(row.recipe == Ok(B::TwoSponsors))
            })
            .sum::<usize>()
}

#[must_use]
pub fn operation_census(rows: &BTreeMap<CoverageRequirementId, CampaignRow>) -> OperationCensus {
    let jobs = staged(rows);
    let initial_signings = ControlFamily::ALL
        .iter()
        .filter(|family| family.sponsored)
        .count();
    OperationCensus {
        issuance: 1 + usize::from(jobs.iter().any(|row| {
            matches!(
                row.recipe,
                Ok(B::AshInputAsset | B::SponsorInputAsset | B::SuccessorAsset | B::ChangeAsset)
            )
        })),
        constructor_probes: 1,
        sponsor_batches: 1,
        ash_batches: ControlFamily::ALL.len()
            + jobs.len()
            + jobs
                .iter()
                .filter(|row| row.recipe != Ok(B::TwoSponsors))
                .count(),
        signings: initial_signings
            + jobs
                .iter()
                .map(|row| {
                    2 * usize::from(row.control.sponsored)
                        + usize::from(matches!(
                            row.recipe,
                            Ok(B::SponsorInputAsset | B::TwoSponsors)
                        ))
                })
                .sum::<usize>(),
        submissions: ControlFamily::ALL.len() + 2 * jobs.len(),
    }
}

/// Stable address inputs include the roster-derived staged count.
#[must_use]
pub fn address_input(
    rows: &BTreeMap<CoverageRequirementId, CampaignRow>,
    mode: &str,
    attempt: &str,
) -> String {
    format!(
        "tripod/compact-run/address/v2\0candidate rule:phase7:wave0-rulings 128\npins FIXTURE_MERKLE_ROOT FIXTURE_REFERENCE_OUTPUT_KEY PINNED_PROGRAM\nroster compact-native-requirements-v2 {}\ncontrols A2 A3 A4 N2 N3 N4 C2 C3 C4\nstaged {}\nceremony report\nmode {mode}\nattempt {attempt}\n",
        rows.len(),
        staged(rows).len()
    )
}

#[cfg(test)]
mod tests;
