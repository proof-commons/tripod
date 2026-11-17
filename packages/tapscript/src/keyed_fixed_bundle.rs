//! Plan-owned selections lowered to immutable keyed emission material.
//!
//! No search, member expansion, evidence admission or transaction construction
//! occurs here. SOURCE's successor leaf is construction material, not a runtime
//! assignment. Transaction commitment remains a downstream boundary.

use std::collections::{BTreeMap, BTreeSet};

use architecture::keyed_fixed::{KeyedFixedArchitectureBinding, KeyedFixedOperationId};
use compiler::keyed_fixed_plan::{
    CarrierMultiplicity, EvidenceObligation, KeyedFixedPlan, ProofPlan, RelationPlan, RuntimeClause,
};
use realization::{
    TransactionSide,
    keyed_fixed_relation::{KeyedFixedRelation, KeyedFixedRelationId},
};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::{TapscriptError, TapscriptProgram};

pub use crate::keyed_fixed_constructor::KeyedFixedLeafRecipe;

/// An offer to select existing published candidates, never a new plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedBundleSelection {
    /// Must equal the sealed publication's complete binding.
    pub binding: KeyedFixedArchitectureBinding,
    /// Exact ordered scope of the sealed publication.
    pub scope: Vec<KeyedFixedOperationId>,
    /// Index of an existing proof candidate.
    pub proof: usize,
    /// One existing placement index for every selected operation.
    pub placements: BTreeMap<KeyedFixedOperationId, usize>,
}

/// Readable emission publication. Only the sealed bundle is accepted authority.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct KeyedFixedBundleProjection {
    pub selection: KeyedFixedBundleSelection,
    /// The selected proof and complete relation/coverage map. Its placement lists
    /// are restricted to the single selected placement for each operation.
    pub selected: ProofPlan,
    /// Unresolved evidence premises, including SOURCE recognition and reading.
    pub evidence: BTreeSet<EvidenceObligation>,
    /// Construction material even when SOURCE has no runtime assignment.
    pub successor: crate::keyed_fixed_constructor::KeyedFixedConstructionMaterial,
    /// One fragment per runtime relation; member instantiation is deferred.
    pub fragments: BTreeMap<KeyedFixedRelationId, TapscriptProgram>,
}

/// Immutable selected plan and its emitted material; no public constructor.
///
/// ```compile_fail
/// use tapscript::keyed_fixed_bundle::{KeyedFixedBundleProjection, KeyedFixedEmissionBundle};
/// fn forge(projection: KeyedFixedBundleProjection) -> KeyedFixedEmissionBundle {
///     KeyedFixedEmissionBundle { projection }
/// }
/// ```
///
/// ```compile_fail
/// use tapscript::keyed_fixed_bundle::{KeyedFixedBundleProjection, KeyedFixedEmissionBundle};
/// fn replace(bundle: &KeyedFixedEmissionBundle, replacement: KeyedFixedBundleProjection) {
///     *bundle.project() = replacement;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedEmissionBundle {
    projection: KeyedFixedBundleProjection,
}

impl KeyedFixedEmissionBundle {
    /// Inspect the publication without changing the retained plan.
    #[must_use]
    pub const fn project(&self) -> &KeyedFixedBundleProjection {
        &self.projection
    }
}

/// Adapter refusals preserve the boundary; none is a covenant execution result.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedBundleRefusal {
    /// The offered binding differs from the sealed plan.
    BindingMismatch,
    /// The offered ordered scope differs from the sealed plan.
    ScopeMismatch,
    /// No proof candidate exists at this index.
    ProofSelection { index: usize },
    /// The placement offer's operation set differs from the selected proof.
    PlacementScopeMismatch,
    /// No placement exists at this operation's index.
    PlacementSelection {
        operation: KeyedFixedOperationId,
        index: usize,
    },
    /// Internal retained-copy or fragment census lost a relation.
    MissingRelation(KeyedFixedRelationId),
    /// Internal retained-copy census repeated a relation.
    DuplicateRelation(KeyedFixedRelationId),
    /// Internal retained-copy or fragment census added a relation.
    UnexpectedRelation(KeyedFixedRelationId),
    /// A retained row differs from the sealed publication.
    RelationMismatch(KeyedFixedRelationId),
    /// A runtime row is inactive or is not Input/EveryMember.
    RuntimeClauseMismatch(KeyedFixedRelationId),
    /// An evidence relation has acquired runtime clauses.
    EvidenceRuntimeCarrier(KeyedFixedRelationId),
    /// Typed construction pass-through; unreachable for the reviewed literals.
    Fragment(TapscriptError),
}

impl KeyedFixedBundleRefusal {
    /// Payload-free tracing classification; identities remain in the typed value.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::BindingMismatch => "binding_mismatch",
            Self::ScopeMismatch => "scope_mismatch",
            Self::ProofSelection { .. } => "proof_selection",
            Self::PlacementScopeMismatch => "placement_scope_mismatch",
            Self::PlacementSelection { .. } => "placement_selection",
            Self::MissingRelation(_) => "missing_relation",
            Self::DuplicateRelation(_) => "duplicate_relation",
            Self::UnexpectedRelation(_) => "unexpected_relation",
            Self::RelationMismatch(_) => "relation_mismatch",
            Self::RuntimeClauseMismatch(_) => "runtime_clause_mismatch",
            Self::EvidenceRuntimeCarrier(_) => "evidence_runtime_carrier",
            Self::Fragment(_) => "fragment",
        }
    }
}

/// Lower only an explicitly selected published plan, retaining its obligations.
///
/// # Errors
/// Refuses a different binding/scope, unavailable candidate, inconsistent retained
/// relation, unsupported runtime clause, or checked fragment construction error.
pub fn emit_keyed_fixed_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &KeyedFixedPlan,
    selection: KeyedFixedBundleSelection,
    recipe: KeyedFixedLeafRecipe,
) -> Result<KeyedFixedEmissionBundle, KeyedFixedBundleRefusal> {
    let _span =
        tracing::debug_span!(target: "tapscript::keyed_fixed_bundle", "emit_keyed_fixed_bundle",
        operation_count = count(selection.scope.len()), proof_index = count(selection.proof))
        .entered();
    let result = emit(target, plan, selection, recipe);
    match &result {
        Ok(bundle) => {
            tracing::debug!(target: "tapscript::keyed_fixed_bundle", outcome = "completed",
            relation_count = count(bundle.projection.selected.selected_relations.len()),
            runtime_clause_count = count(bundle.projection.fragments.len()),
            evidence_count = count(bundle.projection.evidence.len()), "keyed bundle emitted");
        }
        Err(error) => tracing::debug!(target: "tapscript::keyed_fixed_bundle", outcome = "refused",
            refusal_kind = error.refusal_kind(), "keyed bundle refused"),
    }
    result
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn emit(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &KeyedFixedPlan,
    selection: KeyedFixedBundleSelection,
    recipe: KeyedFixedLeafRecipe,
) -> Result<KeyedFixedEmissionBundle, KeyedFixedBundleRefusal> {
    let published = plan.project();
    if selection.binding != published.binding {
        return Err(KeyedFixedBundleRefusal::BindingMismatch);
    }
    if selection.scope != published.scope {
        return Err(KeyedFixedBundleRefusal::ScopeMismatch);
    }
    let proof =
        published
            .plans
            .get(selection.proof)
            .ok_or(KeyedFixedBundleRefusal::ProofSelection {
                index: selection.proof,
            })?;
    if selection
        .placements
        .keys()
        .copied()
        .collect::<BTreeSet<_>>()
        != proof.operations.keys().copied().collect()
    {
        return Err(KeyedFixedBundleRefusal::PlacementScopeMismatch);
    }
    let selected = restrict_placements(proof, &selection.placements)?;
    let fragments = lower_selected(target, proof, &selected, recipe)?;
    let evidence = proof.requirements.values().flatten().cloned().collect();
    let successor = recipe
        .construct(target)
        .map_err(KeyedFixedBundleRefusal::Fragment)?;
    Ok(KeyedFixedEmissionBundle {
        projection: KeyedFixedBundleProjection {
            selection,
            selected,
            evidence,
            successor,
            fragments,
        },
    })
}

fn restrict_placements(
    proof: &ProofPlan,
    placements: &BTreeMap<KeyedFixedOperationId, usize>,
) -> Result<ProofPlan, KeyedFixedBundleRefusal> {
    let mut selected = proof.clone();
    for (operation, index) in placements {
        let original = proof
            .operations
            .get(operation)
            .ok_or(KeyedFixedBundleRefusal::PlacementScopeMismatch)?;
        let placement =
            original
                .placements
                .get(*index)
                .ok_or(KeyedFixedBundleRefusal::PlacementSelection {
                    operation: *operation,
                    index: *index,
                })?;
        if let Some(row) = selected.operations.get_mut(operation) {
            row.placements = vec![placement.clone()];
        }
    }
    Ok(selected)
}

fn lower_selected(
    target: &ReviewedElementsTapscriptDefinition,
    proof: &ProofPlan,
    selected: &ProofPlan,
    recipe: KeyedFixedLeafRecipe,
) -> Result<BTreeMap<KeyedFixedRelationId, TapscriptProgram>, KeyedFixedBundleRefusal> {
    let rows = selected
        .operations
        .values()
        .flat_map(|operation| {
            operation
                .relations
                .iter()
                .map(|(id, row)| (*id, row.clone()))
        })
        .collect::<Vec<_>>();
    let fragments = lower_rows(target, proof, &rows, recipe)?;
    validate_fragment_census(selected, &fragments)?;
    Ok(fragments)
}

fn validate_fragment_census(
    selected: &ProofPlan,
    fragments: &BTreeMap<KeyedFixedRelationId, TapscriptProgram>,
) -> Result<(), KeyedFixedBundleRefusal> {
    let runtime = selected
        .operations
        .values()
        .flat_map(|o| &o.relations)
        .filter(|(_, row)| !row.runtime.is_empty())
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    if let Some(id) = runtime.iter().find(|id| !fragments.contains_key(id)) {
        return Err(KeyedFixedBundleRefusal::MissingRelation(*id));
    }
    if let Some(id) = fragments.keys().find(|id| !runtime.contains(id)) {
        return Err(KeyedFixedBundleRefusal::UnexpectedRelation(*id));
    }
    Ok(())
}

// Validate retained rows against the selected sealed publication, not a second
// derivation of SOURCE/MERGE. The vector also makes duplicate-copy defects visible.
fn lower_rows(
    target: &ReviewedElementsTapscriptDefinition,
    proof: &ProofPlan,
    rows: &[(KeyedFixedRelationId, RelationPlan)],
    recipe: KeyedFixedLeafRecipe,
) -> Result<BTreeMap<KeyedFixedRelationId, TapscriptProgram>, KeyedFixedBundleRefusal> {
    let expected = proof
        .operations
        .values()
        .flat_map(|o| o.relations.iter().map(|(id, row)| (*id, row)))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut fragments = BTreeMap::new();
    for (id, row) in rows {
        if !seen.insert(*id) {
            return Err(KeyedFixedBundleRefusal::DuplicateRelation(*id));
        }
        let original = expected
            .get(id)
            .ok_or(KeyedFixedBundleRefusal::UnexpectedRelation(*id))?;
        match row.declaration {
            KeyedFixedRelation::BornFirst
            | KeyedFixedRelation::OwnQualification
            | KeyedFixedRelation::SuccessorQualification => {
                if !row.active
                    || row.runtime
                        != [RuntimeClause {
                            side: TransactionSide::Input,
                            multiplicity: CarrierMultiplicity::EveryMember,
                        }]
                {
                    return Err(KeyedFixedBundleRefusal::RuntimeClauseMismatch(*id));
                }
            }
            KeyedFixedRelation::HandleFamily { .. }
            | KeyedFixedRelation::SourceClassification
            | KeyedFixedRelation::MembershipReading { .. } => {
                if !row.runtime.is_empty() {
                    return Err(KeyedFixedBundleRefusal::EvidenceRuntimeCarrier(*id));
                }
            }
        }
        if row != *original {
            return Err(KeyedFixedBundleRefusal::RelationMismatch(*id));
        }
        let fragment = match row.declaration {
            KeyedFixedRelation::BornFirst => {
                Some(crate::keyed_pattern::born_first_fragment_untraced(target))
            }
            KeyedFixedRelation::OwnQualification | KeyedFixedRelation::SuccessorQualification => {
                Some(crate::keyed_fixed_pattern::qualification(
                    target,
                    recipe,
                    row.declaration == KeyedFixedRelation::OwnQualification,
                ))
            }
            KeyedFixedRelation::HandleFamily { .. }
            | KeyedFixedRelation::SourceClassification
            | KeyedFixedRelation::MembershipReading { .. } => None,
        };
        if let Some(fragment) = fragment {
            fragments.insert(*id, fragment.map_err(KeyedFixedBundleRefusal::Fragment)?);
        }
    }
    if let Some(id) = expected.keys().find(|id| !seen.contains(*id)) {
        return Err(KeyedFixedBundleRefusal::MissingRelation(*id));
    }
    Ok(fragments)
}
