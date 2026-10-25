//! Requirement-keyed compact subjects and their unresolved realization obligations.

use std::collections::BTreeMap;

use architecture::ObjectId;
use compiler::operation_plan::{CoverageRequirementId, RelationMutation, SponsorCase};
use realization::{RelationKind, RelationSubject, TransactionSide};

use crate::compact_negative_manifest::{
    CompactNegativeRow, NegativeClass, compact_negative_manifest,
};
use crate::matrix::EvidenceBoundary;

/// Accepted construction shape, independent of a mutation's malformed shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ControlFamily {
    pub ash: u8,
    pub sponsored: bool,
    pub change: bool,
}

impl ControlFamily {
    /// Every selected compact coordinator shape.
    pub const ALL: [Self; 9] = [
        Self::new(2, false, false),
        Self::new(3, false, false),
        Self::new(4, false, false),
        Self::new(2, true, false),
        Self::new(3, true, false),
        Self::new(4, true, false),
        Self::new(2, true, true),
        Self::new(3, true, true),
        Self::new(4, true, true),
    ];

    const fn new(ash: u8, sponsored: bool, change: bool) -> Self {
        Self {
            ash,
            sponsored,
            change,
        }
    }

    /// Stable shape name; no source identity enters it.
    #[must_use]
    pub fn name(self) -> String {
        format!(
            "{}{}",
            if !self.sponsored {
                "A"
            } else if self.change {
                "C"
            } else {
                "N"
            },
            self.ash
        )
    }
}

/// Byte changes staged only by the native integration target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteRecipe {
    OneInput,
    FiveInputs,
    RedirectSuccessor,
    SplitSuccessor,
    DivertUnit,
    SplitChange,
    MissingAuthorization,
    AshInputAsset,
    AshInputObject,
    SponsorInputAsset,
    SuccessorAsset,
    ChangeAsset,
    TwoSponsors,
}

/// A missing byte realization is retained, never funded or discharged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnresolvedRecipe {
    SpecializedInputFunding,
    BalancedAssetSubstitution,
    SponsorObjectRealization,
    AdditionalSponsorSigning,
    RegionalClaimEncodingUnproved,
    DeltaRealizationUnproved,
    RootEffectRealizationUnproved,
    ProjectionRealizationUnproved,
    AbsentSponsorAuthorization,
    UnclassifiedMutation,
    UnprobedRecipe,
}

/// Byte realization alone grants no observation credit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Representability {
    Representable { outcome: EvidenceBoundary },
    Unresolved(UnresolvedRecipe),
}

/// Exact keys demonstrated with accepted controls in the admitted compact archive.
/// A later recipe cannot inherit that demonstration from its mutation class.
#[must_use]
#[allow(
    clippy::unnested_or_patterns,
    reason = "retain one complete tuple per demonstrated requirement key"
)]
pub fn representability(row: &CampaignRow) -> Representability {
    use ByteRecipe as B;
    let key = requirement_name(&row.manifest.requirement);
    let family = row.control.name();
    let demonstrated = matches!(
        (key.as_str(), row.recipe, family.as_str()),
        (
            "Absent/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)",
            Ok(B::OneInput),
            "A2"
        ) | (
            "Absent/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Bound(AshBatchMax) })",
            Ok(B::FiveInputs),
            "A4"
        ) | (
            "Present/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)",
            Ok(B::OneInput),
            "N2"
        ) | (
            "Present/Cardinality/ObjectFamily { side: Input, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Bound(AshBatchMax) })",
            Ok(B::FiveInputs),
            "N4"
        ) | (
            "Absent/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)",
            Ok(B::RedirectSuccessor),
            "A2"
        ) | (
            "Absent/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Declared(Count(1)) })",
            Ok(B::SplitSuccessor),
            "A2"
        ) | (
            "Present/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityBelowMinimum)",
            Ok(B::RedirectSuccessor),
            "N2"
        ) | (
            "Present/Cardinality/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Declared(Count(1)) })",
            Ok(B::SplitSuccessor),
            "N2"
        ) | (
            "Present/Cardinality/ObjectFamily { side: Output, object: PlainLbtc }/RuntimeCarrier/FocusedReject(CardinalityAboveMaximum { ceiling: Declared(Count(1)) })",
            Ok(B::SplitChange),
            "C2"
        ) | (
            "Absent/Recognition/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)",
            Ok(B::RedirectSuccessor),
            "A2"
        ) | (
            "Present/Recognition/ObjectFamily { side: Output, object: Ash }/RuntimeCarrier/FocusedReject(WrongRecognizedObject)",
            Ok(B::RedirectSuccessor),
            "N2"
        ) | (
            "Absent/Conservation/Asset { asset: U }/RuntimeCarrier/FocusedReject(AmountMismatch)",
            Ok(B::DivertUnit),
            "A2"
        ) | (
            "Present/Conservation/Asset { asset: U }/RuntimeCarrier/FocusedReject(AmountMismatch)",
            Ok(B::DivertUnit),
            "N2"
        ) | (
            "Absent/AllowedObjectFamilies/TransactionSide { side: Output }/RuntimeCarrier/FocusedReject(UndeclaredObjectFamily)",
            Ok(B::DivertUnit),
            "A2"
        ) | (
            "Present/AllowedObjectFamilies/TransactionSide { side: Output }/RuntimeCarrier/FocusedReject(UndeclaredObjectFamily)",
            Ok(B::DivertUnit),
            "N2"
        ) | (
            "Present/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(MissingSponsorAuthorization)",
            Ok(B::MissingAuthorization),
            "N2"
        )
    );
    if demonstrated && row.expected == EvidenceBoundary::ScriptPathRejection {
        Representability::Representable {
            outcome: EvidenceBoundary::ScriptPathRejection,
        }
    } else {
        Representability::Unresolved(row.recipe.err().unwrap_or(UnresolvedRecipe::UnprobedRecipe))
    }
}

/// A staged recipe is not representable until a scratch execution demonstrates it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignRow {
    pub manifest: CompactNegativeRow,
    pub control: ControlFamily,
    pub recipe: Result<ByteRecipe, UnresolvedRecipe>,
    pub expected: EvidenceBoundary,
}

/// Complete schedule when every control accepts and every mutant refuses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationCensus {
    pub issuance: usize,
    pub constructor_probes: usize,
    pub sponsor_batches: usize,
    pub ash_batches: usize,
    pub signings: usize,
    pub submissions: usize,
}

impl OperationCensus {
    /// Protocol operations; internal RPC calls and mined blocks are separate.
    #[must_use]
    pub const fn total(self) -> usize {
        self.issuance
            + self.constructor_probes
            + self.sponsor_batches
            + self.ash_batches
            + self.signings
            + self.submissions
    }
}

/// Unresolved recipes contribute no funding, signing or submission work.
#[must_use]
pub fn operation_census(rows: &BTreeMap<CoverageRequirementId, CampaignRow>) -> OperationCensus {
    let staged: Vec<_> = rows
        .values()
        .filter(|row| {
            matches!(
                representability(row),
                Representability::Representable { .. }
            )
        })
        .collect();
    let sponsored = staged.iter().filter(|row| row.control.sponsored).count();
    let unsigned = staged
        .iter()
        .filter(|row| row.recipe == Ok(ByteRecipe::MissingAuthorization))
        .count();
    OperationCensus {
        issuance: 1,
        constructor_probes: 1,
        sponsor_batches: 1,
        ash_batches: ControlFamily::ALL.len() + staged.len(),
        signings: 6 + 2 * sponsored - unsigned,
        submissions: ControlFamily::ALL.len() + 2 * staged.len(),
    }
}

/// Derive every native requirement without changing its identity or collateral.
///
/// # Errors
/// Returns the manifest's refusal if the candidate's requirement join fails.
pub fn roster(
    bundle: &crate::bundle::FixtureBundle,
) -> Result<
    BTreeMap<CoverageRequirementId, CampaignRow>,
    crate::compact_negative_manifest::ManifestRefusal,
> {
    Ok(compact_negative_manifest(bundle)?
        .into_iter()
        .filter(|(_, row)| row.class == NegativeClass::TargetExecutable)
        .map(|(id, manifest)| {
            let (control, recipe) = route(&id, manifest.mutation);
            (
                id,
                CampaignRow {
                    manifest,
                    control,
                    recipe,
                    expected: EvidenceBoundary::ScriptPathRejection,
                },
            )
        })
        .collect())
}

fn route(
    id: &CoverageRequirementId,
    mutation: RelationMutation,
) -> (ControlFamily, Result<ByteRecipe, UnresolvedRecipe>) {
    let sponsored = id.case.sponsor == SponsorCase::Present;
    let mut control = ControlFamily::new(2, sponsored, false);
    if *id.relation.subject()
        == (RelationSubject::ObjectFamily {
            object: ObjectId::PlainLbtc,
            side: TransactionSide::Output,
        })
    {
        control.change = sponsored;
    }
    let recipe = route_recipe(id, mutation, &mut control);
    (control, recipe)
}

const fn route_recipe(
    id: &CoverageRequirementId,
    mutation: RelationMutation,
    control: &mut ControlFamily,
) -> Result<ByteRecipe, UnresolvedRecipe> {
    use ByteRecipe as B;
    use RelationMutation as M;
    use UnresolvedRecipe as U;
    match (id.relation.kind(), id.relation.subject(), mutation) {
        (
            RelationKind::Cardinality,
            RelationSubject::ObjectFamily {
                object: ObjectId::Ash,
                side: TransactionSide::Input,
            },
            M::CardinalityBelowMinimum,
        ) => Ok(B::OneInput),
        (
            RelationKind::Cardinality,
            RelationSubject::ObjectFamily {
                object: ObjectId::Ash,
                side: TransactionSide::Input,
            },
            M::CardinalityAboveMaximum { .. },
        ) => {
            control.ash = 4;
            Ok(B::FiveInputs)
        }
        (
            RelationKind::Cardinality,
            RelationSubject::ObjectFamily {
                object: ObjectId::Ash,
                side: TransactionSide::Output,
            },
            M::CardinalityBelowMinimum,
        )
        | (
            RelationKind::Recognition,
            RelationSubject::ObjectFamily {
                object: ObjectId::Ash,
                side: TransactionSide::Output,
            },
            M::WrongRecognizedObject,
        ) => Ok(B::RedirectSuccessor),
        (
            RelationKind::Cardinality,
            RelationSubject::ObjectFamily {
                object: ObjectId::Ash,
                side: TransactionSide::Output,
            },
            M::CardinalityAboveMaximum { .. },
        ) => Ok(B::SplitSuccessor),
        (
            RelationKind::Cardinality,
            RelationSubject::ObjectFamily {
                object: ObjectId::PlainLbtc,
                side: TransactionSide::Output,
            },
            _,
        ) => {
            control.change = true;
            Ok(B::SplitChange)
        }
        (RelationKind::Cardinality, _, _) => Err(U::AdditionalSponsorSigning),
        (
            RelationKind::Recognition,
            RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                ..
            },
            _,
        ) => Err(U::SpecializedInputFunding),
        (RelationKind::Recognition, _, M::WrongRecognizedAsset) => {
            Err(U::BalancedAssetSubstitution)
        }
        (RelationKind::Recognition, _, _) => {
            control.change = control.sponsored;
            Err(U::SponsorObjectRealization)
        }
        (
            RelationKind::AllowedObjectFamilies,
            RelationSubject::TransactionSide {
                side: TransactionSide::Output,
            },
            _,
        )
        | (RelationKind::Conservation, _, M::AmountMismatch) => Ok(B::DivertUnit),
        (RelationKind::AllowedObjectFamilies, _, _) => Err(U::SpecializedInputFunding),
        (RelationKind::SponsorIsolation, _, M::MissingSponsorAuthorization) => {
            if control.sponsored {
                Ok(B::MissingAuthorization)
            } else {
                Err(U::AbsentSponsorAuthorization)
            }
        }
        (RelationKind::SponsorIsolation | RelationKind::OpenFlowPolicy, _, _) => {
            Err(U::RegionalClaimEncodingUnproved)
        }
        (RelationKind::CanonicalDeltaPolicy, _, _) => Err(U::DeltaRealizationUnproved),
        (RelationKind::RootPolicy, _, _) => Err(U::RootEffectRealizationUnproved),
        (RelationKind::ProjectionPolicy, _, _) => Err(U::ProjectionRealizationUnproved),
        _ => Err(U::UnclassifiedMutation),
    }
}

/// Exact typed-key rendering for the scratch roster and retained census.
#[must_use]
pub fn requirement_name(id: &CoverageRequirementId) -> String {
    format!(
        "{:?}/{:?}/{:?}/{:?}/{:?}",
        id.case.sponsor,
        id.relation.kind(),
        id.relation.subject(),
        id.boundary,
        id.purpose
    )
}

/// Compare a native result with the requirement's declared boundary.
#[must_use]
pub fn matches(
    row: &CampaignRow,
    layer: target_elements_conformance::protocol::ObservedOutcomeLayer,
) -> bool {
    crate::observed_boundary::matches_boundary(row.expected, layer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use target_elements_conformance::protocol::ObservedOutcomeLayer as L;

    #[test]
    fn roster_preserves_all_forty_eight_native_keys_and_collateral() {
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let rows = roster(&bundle).expect("roster");
        assert_eq!(rows.len(), 48);
        let manifest = compact_negative_manifest(&bundle).expect("manifest");
        for (id, row) in &rows {
            assert_eq!(&row.manifest, &manifest[id]);
            assert_eq!(row.expected, EvidenceBoundary::ScriptPathRejection);
            assert!(ControlFamily::ALL.contains(&row.control));
            for other in [
                L::ConsensusRejectionBeforeScript,
                L::KeyPathRejection,
                L::RelayPolicyRejection,
                L::Accepted,
                L::FixtureConstructionFailure,
                L::ExecutorInfrastructureFailure,
            ] {
                assert!(!matches(row, other));
            }
        }
        assert_eq!(rows.values().filter(|row| row.recipe.is_ok()).count(), 16);
        assert_eq!(
            operation_census(&rows),
            OperationCensus {
                issuance: 1,
                constructor_probes: 1,
                sponsor_batches: 1,
                ash_batches: 25,
                signings: 23,
                submissions: 41,
            }
        );
        assert_eq!(operation_census(&rows).total(), 92);
        assert_eq!(
            rows.values()
                .filter(|row| matches!(
                    representability(row),
                    Representability::Representable { .. }
                ))
                .count(),
            16
        );
        for row in rows.values().filter(|row| row.recipe.is_ok()) {
            let mut changed = row.clone();
            changed.control.ash = 3;
            assert_eq!(
                representability(&changed),
                Representability::Unresolved(UnresolvedRecipe::UnprobedRecipe)
            );
            changed = row.clone();
            changed.expected = EvidenceBoundary::ConsensusRejectionBeforeScript;
            assert_eq!(
                representability(&changed),
                Representability::Unresolved(UnresolvedRecipe::UnprobedRecipe)
            );
        }
    }

    #[test]
    fn absent_sponsor_has_no_authorization_to_remove_but_remains_unresolved() {
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let rows = roster(&bundle).expect("roster");
        let absent = rows
            .values()
            .filter(|row| row.recipe == Err(UnresolvedRecipe::AbsentSponsorAuthorization))
            .collect::<Vec<_>>();
        assert_eq!(absent.len(), 1);
        assert_eq!(
            absent[0].manifest.requirement.case.sponsor,
            SponsorCase::Absent
        );
        assert_eq!(
            absent[0].manifest.mutation,
            RelationMutation::MissingSponsorAuthorization
        );
        assert!(!absent[0].control.sponsored);
    }
}
