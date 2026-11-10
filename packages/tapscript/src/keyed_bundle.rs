//! Plan-owned selections lowered to immutable keyed emission material.
//!
//! No search, member expansion, evidence admission or transaction construction
//! occurs here. SOURCE's successor leaf is construction material, not a runtime
//! assignment. Concrete resource and constructor closure are later boundaries.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{keyed::KeyedArchitectureBinding, semantic::SemanticOperationId};
use compiler::keyed_plan::{
    CarrierMultiplicity, EvidenceObligation, KeyedPlan, ProofPlan, RelationPlan, RuntimeClause,
};
use realization::{
    TransactionSide,
    relation::{KeyedRelation, KeyedRelationId},
};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::{TapscriptError, TapscriptProgram};

/// Public semantic bytes for the fixed key/drop, birth, continuation, true recipe.
/// This is not a committed output or an admitted control block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedLeafRecipe {
    key: [u8; 32],
}

impl KeyedLeafRecipe {
    /// Every 32-byte public semantic key is allowed; no creator secret is needed.
    #[must_use]
    pub const fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Exact public bytes committed by the successor leaf.
    #[must_use]
    pub const fn key(&self) -> &[u8; 32] {
        &self.key
    }
}

/// An offer to select existing published candidates, never a new plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedBundleSelection {
    /// Must equal the sealed publication's complete binding.
    pub binding: KeyedArchitectureBinding,
    /// Exact ordered scope of the sealed publication.
    pub scope: Vec<SemanticOperationId>,
    /// Index of an existing proof candidate.
    pub proof: usize,
    /// One existing placement index for every selected operation.
    pub placements: BTreeMap<SemanticOperationId, usize>,
}

/// Readable emission publication. Only the sealed bundle is accepted authority.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct KeyedBundleProjection {
    pub selection: KeyedBundleSelection,
    /// The selected proof and complete relation/coverage map. Its placement lists
    /// are restricted to the single selected placement for each operation.
    pub selected: ProofPlan,
    /// Unresolved evidence premises, including SOURCE recognition and reading.
    pub evidence: BTreeSet<EvidenceObligation>,
    pub recipe: KeyedLeafRecipe,
    /// Construction material even when SOURCE has no runtime assignment.
    pub successor_leaf: TapscriptProgram,
    /// One fragment per runtime relation; member instantiation is deferred.
    pub fragments: BTreeMap<KeyedRelationId, TapscriptProgram>,
}

/// Immutable selected plan and its emitted material; no public constructor.
///
/// ```compile_fail
/// use tapscript::keyed_bundle::{KeyedBundleProjection, KeyedEmissionBundle};
/// fn forge(projection: KeyedBundleProjection) -> KeyedEmissionBundle {
///     KeyedEmissionBundle { projection }
/// }
/// ```
///
/// ```compile_fail
/// use tapscript::keyed_bundle::{KeyedBundleProjection, KeyedEmissionBundle};
/// fn replace(bundle: &KeyedEmissionBundle, replacement: KeyedBundleProjection) {
///     *bundle.project() = replacement;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedEmissionBundle {
    projection: KeyedBundleProjection,
}

impl KeyedEmissionBundle {
    /// Inspect the publication without changing the retained plan.
    #[must_use]
    pub const fn project(&self) -> &KeyedBundleProjection {
        &self.projection
    }
}

/// Adapter refusals preserve the boundary; none is a covenant execution result.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedBundleRefusal {
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
        operation: SemanticOperationId,
        index: usize,
    },
    /// Internal retained-copy or fragment census lost a relation.
    MissingRelation(KeyedRelationId),
    /// Internal retained-copy census repeated a relation.
    DuplicateRelation(KeyedRelationId),
    /// Internal retained-copy or fragment census added a relation.
    UnexpectedRelation(KeyedRelationId),
    /// A retained row differs from the sealed publication.
    RelationMismatch(KeyedRelationId),
    /// A runtime row is inactive or is not Input/EveryMember.
    RuntimeClauseMismatch(KeyedRelationId),
    /// An evidence relation has acquired runtime clauses.
    EvidenceRuntimeCarrier(KeyedRelationId),
    /// Typed construction pass-through; unreachable for the reviewed literals.
    Fragment(TapscriptError),
}

impl KeyedBundleRefusal {
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
pub fn emit_keyed_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &KeyedPlan,
    selection: KeyedBundleSelection,
    recipe: KeyedLeafRecipe,
) -> Result<KeyedEmissionBundle, KeyedBundleRefusal> {
    let _span = tracing::debug_span!(target: "tapscript::keyed_bundle", "emit_keyed_bundle",
        operation_count = count(selection.scope.len()), proof_index = count(selection.proof))
    .entered();
    let result = emit(target, plan, selection, recipe);
    match &result {
        Ok(bundle) => tracing::debug!(target: "tapscript::keyed_bundle", outcome = "completed",
            relation_count = count(bundle.projection.selected.selected_relations.len()),
            runtime_clause_count = count(bundle.projection.fragments.len()),
            evidence_count = count(bundle.projection.evidence.len()), "keyed bundle emitted"),
        Err(error) => tracing::debug!(target: "tapscript::keyed_bundle", outcome = "refused",
            refusal_kind = error.refusal_kind(), "keyed bundle refused"),
    }
    result
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn emit(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &KeyedPlan,
    selection: KeyedBundleSelection,
    recipe: KeyedLeafRecipe,
) -> Result<KeyedEmissionBundle, KeyedBundleRefusal> {
    let published = plan.project();
    if selection.binding != published.binding {
        return Err(KeyedBundleRefusal::BindingMismatch);
    }
    if selection.scope != published.scope {
        return Err(KeyedBundleRefusal::ScopeMismatch);
    }
    let proof = published
        .plans
        .get(selection.proof)
        .ok_or(KeyedBundleRefusal::ProofSelection {
            index: selection.proof,
        })?;
    if selection
        .placements
        .keys()
        .copied()
        .collect::<BTreeSet<_>>()
        != proof.operations.keys().copied().collect()
    {
        return Err(KeyedBundleRefusal::PlacementScopeMismatch);
    }
    let selected = restrict_placements(proof, &selection.placements)?;
    let fragments = lower_selected(target, proof, &selected)?;
    let evidence = proof.requirements.values().flatten().cloned().collect();
    let successor_leaf = crate::keyed_pattern::compose_keyed_leaf(target, recipe.key())
        .map_err(KeyedBundleRefusal::Fragment)?;
    Ok(KeyedEmissionBundle {
        projection: KeyedBundleProjection {
            selection,
            selected,
            evidence,
            recipe,
            successor_leaf,
            fragments,
        },
    })
}

fn restrict_placements(
    proof: &ProofPlan,
    placements: &BTreeMap<SemanticOperationId, usize>,
) -> Result<ProofPlan, KeyedBundleRefusal> {
    let mut selected = proof.clone();
    for (operation, index) in placements {
        let original = proof
            .operations
            .get(operation)
            .ok_or(KeyedBundleRefusal::PlacementScopeMismatch)?;
        let placement =
            original
                .placements
                .get(*index)
                .ok_or(KeyedBundleRefusal::PlacementSelection {
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
) -> Result<BTreeMap<KeyedRelationId, TapscriptProgram>, KeyedBundleRefusal> {
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
    let fragments = lower_rows(target, proof, &rows)?;
    validate_fragment_census(selected, &fragments)?;
    Ok(fragments)
}

fn validate_fragment_census(
    selected: &ProofPlan,
    fragments: &BTreeMap<KeyedRelationId, TapscriptProgram>,
) -> Result<(), KeyedBundleRefusal> {
    let runtime = selected
        .operations
        .values()
        .flat_map(|o| &o.relations)
        .filter(|(_, row)| !row.runtime.is_empty())
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    if let Some(id) = runtime.iter().find(|id| !fragments.contains_key(id)) {
        return Err(KeyedBundleRefusal::MissingRelation(*id));
    }
    if let Some(id) = fragments.keys().find(|id| !runtime.contains(id)) {
        return Err(KeyedBundleRefusal::UnexpectedRelation(*id));
    }
    Ok(())
}

// Validate retained rows against the selected sealed publication, not a second
// derivation of SOURCE/MERGE. The vector also makes duplicate-copy defects visible.
fn lower_rows(
    target: &ReviewedElementsTapscriptDefinition,
    proof: &ProofPlan,
    rows: &[(KeyedRelationId, RelationPlan)],
) -> Result<BTreeMap<KeyedRelationId, TapscriptProgram>, KeyedBundleRefusal> {
    let expected = proof
        .operations
        .values()
        .flat_map(|o| o.relations.iter().map(|(id, row)| (*id, row)))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut fragments = BTreeMap::new();
    for (id, row) in rows {
        if !seen.insert(*id) {
            return Err(KeyedBundleRefusal::DuplicateRelation(*id));
        }
        let original = expected
            .get(id)
            .ok_or(KeyedBundleRefusal::UnexpectedRelation(*id))?;
        match row.declaration {
            KeyedRelation::BornFirst | KeyedRelation::ContinueFirst => {
                if !row.active
                    || row.runtime
                        != [RuntimeClause {
                            side: TransactionSide::Input,
                            multiplicity: CarrierMultiplicity::EveryMember,
                        }]
                {
                    return Err(KeyedBundleRefusal::RuntimeClauseMismatch(*id));
                }
            }
            KeyedRelation::HandleFamily { .. }
            | KeyedRelation::SourceClassification
            | KeyedRelation::MembershipReading { .. } => {
                if !row.runtime.is_empty() {
                    return Err(KeyedBundleRefusal::EvidenceRuntimeCarrier(*id));
                }
            }
        }
        if row != *original {
            return Err(KeyedBundleRefusal::RelationMismatch(*id));
        }
        let fragment = match row.declaration {
            KeyedRelation::BornFirst => Some(crate::keyed_pattern::born_first_fragment(target)),
            KeyedRelation::ContinueFirst => {
                Some(crate::keyed_pattern::continue_first_fragment(target))
            }
            KeyedRelation::HandleFamily { .. }
            | KeyedRelation::SourceClassification
            | KeyedRelation::MembershipReading { .. } => None,
        };
        if let Some(fragment) = fragment {
            fragments.insert(*id, fragment.map_err(KeyedBundleRefusal::Fragment)?);
        }
    }
    if let Some(id) = expected.keys().find(|id| !seen.contains(*id)) {
        return Err(KeyedBundleRefusal::MissingRelation(*id));
    }
    Ok(fragments)
}

#[cfg(test)]
mod tests;
