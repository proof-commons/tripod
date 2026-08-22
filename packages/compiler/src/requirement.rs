//! Relation-indexed requirement bundles (Guide-7 §7).
//!
//! A candidate carries aggregate capability, source, and evidence sets.
//! Aggregates alone cannot say which relation introduced an item, so an
//! item that silently disappears — an external-evidence capability, for
//! instance — leaves no owner to notice. This stage re-derives every
//! item against the relation that owns it and then proves, by exact set
//! equality, that the owned items union back to the aggregate.
//!
//! Ownership does not manufacture requirements. A relation's items come
//! from the same `proof_capabilities` and `derive_source_requirements`
//! rules the candidate search itself used, applied to the proof that
//! relation was assigned; a statically validated relation owns no
//! capability and no source row because no proof was selected for it,
//! and that emptiness is what makes the union equal the aggregate
//! rather than exceed it.
//!
//! Every requirement here states what a later boundary must provide. A
//! retained external-evidence requirement is not discharged evidence, a
//! backend obligation is not an emitted program, and a source row is a
//! statement about how a fact must be authenticated, never a claim that
//! it already was.

use std::collections::{BTreeMap, BTreeSet};

use architecture::ObjectId;
use realization::{
    ExternalEvidenceRequirement, ProofAlternativeId, Relation, RelationDeclaration, RelationId,
    RepresentationMode,
};

use crate::{
    CompileError,
    capability::RequiredCapability,
    case::{ExecutionCaseId, is_active},
    constructibility::{CompilerConstructibilityAnalysis, validate_source_constructibility},
    disclosure::{CompilerDisclosureReason, derive_disclosure, validate_disclosure},
    input::BoundCompilerInput,
    lifecycle::{CompilerLifecycleAnalysis, LifecycleRequirement, RepresentationChoiceId},
    proof::{ProofPlanCandidate, RelationObligationClass, classify_obligations},
    relation::CompilerRelationAnalysis,
    source::{
        OperandRole, RequiredSourceKind, SourceRequirement, derive_source_requirements,
        is_sponsor_amount_operand, proof_capabilities, proof_external_evidence,
    },
};

/// How one relation is discharged under one plan candidate.
///
/// The three cases are not degrees of the same thing. A selected proof
/// is a decision the search made among realization-approved
/// alternatives; a statically validated relation had no decision to
/// make; an externally evidenced relation has an approved proof class
/// whose completion the compiler cannot claim at all.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProofDisposition {
    /// The candidate selected this realization-approved alternative.
    Selected { proof: ProofAlternativeId },

    /// The relation is discharged by compiler-static validation, so the
    /// candidate holds no proof variable for it.
    StaticallyValidated,

    /// The relation's approved proof class is retained together with the
    /// external evidence a later boundary must supply. Nothing here
    /// asserts that the evidence exists.
    ExternalEvidence {
        approved_proof: ProofAlternativeId,
        requirement: ExternalEvidenceRequirement,
    },
}

/// One relation's own object-specific representation decision.
///
/// The object is carried with the mode deliberately. A relation about
/// one object family must never read a mode another family happened to
/// select, and a mode-only value cannot express that difference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepresentationSelection {
    pub object: ObjectId,
    pub mode: RepresentationMode,
}

/// Everything one relation owns under one plan candidate.
///
/// The source set is the relation's *complete* set. An execution case
/// narrows it to its active subset through
/// [`active_source_requirements`]; the complete set is what a later
/// boundary must be able to serve across every case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationRequirements {
    pub relation: RelationId,
    pub proof: ProofDisposition,
    pub required_capabilities: BTreeSet<RequiredCapability>,
    pub source_requirements: BTreeSet<SourceRequirement>,
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
    pub representation: Option<RepresentationSelection>,
    pub lifecycle: BTreeSet<LifecycleRequirement>,
}

/// Derive, validate, and close one candidate's relation-indexed
/// requirements in one step.
///
/// # Errors
///
/// Any failure of [`derive_relation_requirements`],
/// [`validate_aggregate_closure`], [`validate_disclosure_closure`], or
/// [`validate_lifecycle_closure`].
pub fn relation_requirements(
    input: &BoundCompilerInput,
    relations: &CompilerRelationAnalysis,
    constructibility: &CompilerConstructibilityAnalysis,
    lifecycle: &CompilerLifecycleAnalysis,
    candidate: &ProofPlanCandidate,
) -> Result<BTreeMap<RelationId, RelationRequirements>, CompileError> {
    let derived = derive_relation_requirements(relations, constructibility, candidate)?;

    validate_aggregate_closure(candidate, &derived)?;
    validate_disclosure_closure(input, candidate, &derived)?;
    validate_lifecycle_closure(lifecycle, candidate, &derived)?;

    Ok(derived)
}

/// Derive one requirement bundle per in-scope relation.
///
/// Classification is not repeated here: [`classify_obligations`] owns
/// which relations are proof-required, statically validated, or
/// externally evidenced, and this stage only joins the candidate's
/// decisions onto that classification.
///
/// # Errors
///
/// [`CompileError::MissingSelectedProof`] when a proof-required
/// relation has no decision; [`CompileError::UnapprovedSelectedProof`]
/// when the decision is not a realization-approved alternative;
/// [`CompileError::UnexpectedSelectedProof`] when a relation that needs
/// no proof carries one; [`CompileError::SponsorValueRead`] on any
/// sponsor amount operand; any failure of
/// [`validate_source_constructibility`].
pub fn derive_relation_requirements(
    relations: &CompilerRelationAnalysis,
    constructibility: &CompilerConstructibilityAnalysis,
    candidate: &ProofPlanCandidate,
) -> Result<BTreeMap<RelationId, RelationRequirements>, CompileError> {
    let obligations = classify_obligations(relations)?;
    let declarations = relations
        .graph
        .node_weights()
        .map(|node| (node.source.id.clone(), node.source.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut derived = BTreeMap::new();

    for obligation in &obligations {
        let relation = &obligation.relation;
        let declaration = &declarations[relation];
        let selected = candidate.proofs.get(relation);

        let (proof, capabilities, sources, evidence) = match &obligation.class {
            RelationObligationClass::ProofRequired { alternatives } => {
                let selected = selected.ok_or_else(|| CompileError::MissingSelectedProof {
                    relation: relation.clone(),
                })?;

                if !alternatives.contains(selected) {
                    return Err(CompileError::UnapprovedSelectedProof {
                        relation: relation.clone(),
                    });
                }

                (
                    ProofDisposition::Selected {
                        proof: selected.clone(),
                    },
                    proof_capabilities(declaration, selected.proof()),
                    derive_source_requirements(declaration, selected.proof())?,
                    proof_external_evidence(declaration, selected.proof()),
                )
            }

            RelationObligationClass::StaticallyValidated => {
                if selected.is_some() {
                    return Err(CompileError::UnexpectedSelectedProof {
                        relation: relation.clone(),
                    });
                }

                (
                    ProofDisposition::StaticallyValidated,
                    BTreeSet::new(),
                    Vec::new(),
                    BTreeSet::new(),
                )
            }

            RelationObligationClass::ExternalEvidence {
                requirement,
                proof,
                required_capabilities,
                source_requirements,
            } => {
                if selected.is_some() {
                    return Err(CompileError::UnexpectedSelectedProof {
                        relation: relation.clone(),
                    });
                }

                (
                    ProofDisposition::ExternalEvidence {
                        approved_proof: proof.clone(),
                        requirement: requirement.clone(),
                    },
                    required_capabilities.clone(),
                    source_requirements.clone(),
                    BTreeSet::from([requirement.clone()]),
                )
            }
        };

        validate_source_constructibility(constructibility, relation, &sources)?;

        let bundle = RelationRequirements {
            relation: relation.clone(),
            proof,
            required_capabilities: capabilities,
            source_requirements: sources.into_iter().collect(),
            external_evidence: evidence,
            representation: owned_representation(declaration, candidate),
            lifecycle: owned_lifecycle(declaration, candidate),
        };

        validate_sponsor_erasure(&bundle)?;
        derived.insert(relation.clone(), bundle);
    }

    Ok(derived)
}

/// Validate the candidate aggregate closure (§7.6).
///
/// Both directions are defects. A relation-owned item absent from the
/// aggregate means the aggregate lost a requirement; an aggregate item
/// no relation owns means a requirement exists with no semantic owner,
/// which is the failure mode the relation index exists to prevent.
///
/// # Errors
///
/// [`CompileError::AnalyzedCapabilityClosureMismatch`],
/// [`CompileError::AnalyzedSourceClosureMismatch`], or
/// [`CompileError::AnalyzedEvidenceClosureMismatch`] on the first
/// difference in canonical order.
pub fn validate_aggregate_closure(
    candidate: &ProofPlanCandidate,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
) -> Result<(), CompileError> {
    let mut capabilities = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut evidence = BTreeSet::new();

    for bundle in requirements.values() {
        capabilities.extend(bundle.required_capabilities.iter().copied());
        sources.extend(bundle.source_requirements.iter().cloned());
        evidence.extend(bundle.external_evidence.iter().cloned());
    }

    let aggregate_sources = candidate
        .source_requirements
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    if let Some(capability) = capabilities
        .difference(&candidate.required_capabilities)
        .next()
    {
        return Err(CompileError::AnalyzedCapabilityClosureMismatch {
            relation: capability_owner(requirements, *capability),
            capability: *capability,
        });
    }

    if let Some(capability) = candidate
        .required_capabilities
        .difference(&capabilities)
        .next()
    {
        return Err(CompileError::AnalyzedCapabilityClosureMismatch {
            relation: None,
            capability: *capability,
        });
    }

    if let Some(row) = sources.difference(&aggregate_sources).next() {
        return Err(CompileError::AnalyzedSourceClosureMismatch {
            relation: Some(row.operand.relation().clone()),
            operand: row.operand.clone(),
        });
    }

    if let Some(row) = aggregate_sources.difference(&sources).next() {
        return Err(CompileError::AnalyzedSourceClosureMismatch {
            relation: None,
            operand: row.operand.clone(),
        });
    }

    if let Some(requirement) = evidence.difference(&candidate.external_evidence).next() {
        return Err(CompileError::AnalyzedEvidenceClosureMismatch {
            relation: evidence_owner(requirements, requirement),
            requirement: requirement.clone(),
        });
    }

    if let Some(requirement) = candidate.external_evidence.difference(&evidence).next() {
        return Err(CompileError::AnalyzedEvidenceClosureMismatch {
            relation: None,
            requirement: requirement.clone(),
        });
    }

    Ok(())
}

/// Validate the candidate disclosure closure (§7.7).
///
/// The candidate's disclosure analysis stays authoritative; this stage
/// proves it is the analysis its own representation decisions imply,
/// that every added fact's typed reason names a decision the candidate
/// actually made, and that no relation requires a fact the analysis
/// keeps private.
///
/// # Errors
///
/// Any failure of [`validate_disclosure`] or [`derive_disclosure`];
/// [`CompileError::AnalyzedDisclosureClosureMismatch`] when the
/// candidate's analysis is not the one its representations imply;
/// [`CompileError::AnalyzedRequirementFactUnavailable`] when a relation
/// requires an exact public value the analysis retains private.
pub fn validate_disclosure_closure(
    input: &BoundCompilerInput,
    candidate: &ProofPlanCandidate,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
) -> Result<(), CompileError> {
    validate_disclosure(&candidate.disclosure)?;

    for (relation, bundle) in requirements {
        for row in &bundle.source_requirements {
            let Some(fact) = exact_public_fact(row) else {
                continue;
            };

            if candidate.disclosure.retained_private.contains(&fact) {
                return Err(CompileError::AnalyzedRequirementFactUnavailable {
                    relation: relation.clone(),
                    fact,
                });
            }
        }
    }

    for reasons in candidate.disclosure.added_required_public.values() {
        for reason in reasons {
            let CompilerDisclosureReason::RepresentationSelection {
                operation,
                object,
                representation,
            } = reason
            else {
                continue;
            };

            let choice = RepresentationChoiceId {
                operation: *operation,
                object: *object,
            };

            if candidate.representations.get(&choice) != Some(representation) {
                return Err(CompileError::AnalyzedDisclosureClosureMismatch);
            }
        }
    }

    let expected = derive_disclosure(
        input.realization().declassification(),
        &candidate.representations,
    )?;

    if expected != candidate.disclosure {
        return Err(CompileError::AnalyzedDisclosureClosureMismatch);
    }

    Ok(())
}

/// Validate the candidate lifecycle closure (§7.8).
///
/// Two equalities, not one: the candidate's rows must be exactly the
/// rows its selected representations require, and every one of those
/// rows must be owned by an in-scope lifecycle-exit relation. Each row
/// keeps its own exit status, so an exit declared outside compiler
/// scope stays an explicit future obligation rather than being dropped.
///
/// # Errors
///
/// [`CompileError::AnalyzedLifecycleClosureMismatch`] on either
/// inequality.
pub fn validate_lifecycle_closure(
    lifecycle: &CompilerLifecycleAnalysis,
    candidate: &ProofPlanCandidate,
    requirements: &BTreeMap<RelationId, RelationRequirements>,
) -> Result<(), CompileError> {
    let carried = candidate.lifecycle.iter().cloned().collect::<BTreeSet<_>>();

    let selected = lifecycle
        .requirements
        .iter()
        .filter(|requirement| {
            candidate.representations.iter().any(|(choice, mode)| {
                choice.object == requirement.object && *mode == requirement.representation
            })
        })
        .cloned()
        .collect::<BTreeSet<_>>();

    if carried != selected {
        return Err(CompileError::AnalyzedLifecycleClosureMismatch {
            missing: selected.difference(&carried).cloned().collect(),
            unexpected: carried.difference(&selected).cloned().collect(),
        });
    }

    let owned = requirements
        .values()
        .flat_map(|bundle| bundle.lifecycle.iter().cloned())
        .collect::<BTreeSet<_>>();

    if owned != carried {
        return Err(CompileError::AnalyzedLifecycleClosureMismatch {
            missing: owned.difference(&carried).cloned().collect(),
            unexpected: carried.difference(&owned).cloned().collect(),
        });
    }

    Ok(())
}

/// The relation's source rows active in one execution case (§7.4).
///
/// The relation-level set is complete; this is its activation-filtered
/// subset, using the same case activation test the execution-case stage
/// uses. A case belonging to another operation selects nothing: the
/// caller pairs a relation with its own operation's cases.
#[must_use]
pub fn active_source_requirements(
    requirements: &RelationRequirements,
    case: &ExecutionCaseId,
) -> BTreeSet<SourceRequirement> {
    if requirements.relation.operation() != case.operation {
        return BTreeSet::new();
    }

    requirements
        .source_requirements
        .iter()
        .filter(|row| is_active(case, row))
        .cloned()
        .collect()
}

/// Sponsor-erasure traversal over one complete requirement bundle.
///
/// Sponsor erasure means the amount is absent, not merely secret, so a
/// sponsor amount operand anywhere in the bundle is a structural
/// failure rather than a privacy classification.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`] on any sponsor amount operand.
pub fn validate_sponsor_erasure(requirements: &RelationRequirements) -> Result<(), CompileError> {
    for row in &requirements.source_requirements {
        if is_sponsor_amount_operand(row.operand.role()) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    Ok(())
}

/// The relation's own representation decision, when it names exactly
/// one represented object.
///
/// A relation naming several represented objects has no single owned
/// selection, and inventing one would reintroduce the mode-only match
/// this type exists to prevent; per-object activation still resolves
/// such a relation's source rows case by case.
#[must_use]
pub fn owned_representation(
    declaration: &RelationDeclaration,
    candidate: &ProofPlanCandidate,
) -> Option<RepresentationSelection> {
    let operation = declaration.id.operation();
    let objects = relation_objects(&declaration.relation);

    let mut selections = objects.into_iter().filter_map(|object| {
        candidate
            .representations
            .get(&RepresentationChoiceId { operation, object })
            .map(|mode| RepresentationSelection {
                object,
                mode: *mode,
            })
    });

    let selection = selections.next()?;

    if selections.next().is_some() {
        return None;
    }

    Some(selection)
}

/// The candidate lifecycle rows this relation declares.
fn owned_lifecycle(
    declaration: &RelationDeclaration,
    candidate: &ProofPlanCandidate,
) -> BTreeSet<LifecycleRequirement> {
    let Relation::LifecycleExit { object, exit } = &declaration.relation else {
        return BTreeSet::new();
    };

    candidate
        .lifecycle
        .iter()
        .filter(|requirement| requirement.object == *object && requirement.exit == *exit)
        .cloned()
        .collect()
}

/// The object families one relation names.
///
/// Exhaustive by construction: a new realization relation variant makes
/// this match fail to compile until its object census is stated.
#[must_use]
pub fn relation_objects(relation: &Relation) -> BTreeSet<ObjectId> {
    match relation {
        Relation::Cardinality { object, .. }
        | Relation::Recognition { object, .. }
        | Relation::OwnerAuthorization { object }
        | Relation::Representation { object, .. }
        | Relation::LifecycleExit { object, .. } => BTreeSet::from([*object]),

        Relation::AllowedObjectFamilies { allowed, .. } => allowed.clone(),

        Relation::AmountConservation {
            input_objects,
            output_objects,
            ..
        } => input_objects.union(output_objects).copied().collect(),

        Relation::PermissionlessAuthorization
        | Relation::SponsorIsolation
        | Relation::SponsorEnvelopeMultiplicity { .. }
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::Constructibility { .. }
        | Relation::ExpressionPredicate { .. }
        | Relation::SubstrateConservation { .. } => BTreeSet::new(),
    }
}

/// The fact one source row needs published as an exact public value.
///
/// Only an authenticated consensus value over a family amount makes
/// that demand: a commitment relation authenticates the amount without
/// publishing it, and every other row names a fact that is public by
/// class already.
fn exact_public_fact(row: &SourceRequirement) -> Option<realization::FactId> {
    if row.source != RequiredSourceKind::AuthenticatedConsensusValue {
        return None;
    }

    let OperandRole::ObjectFamilyAmount { side, object } = row.operand.role() else {
        return None;
    };

    Some(realization::FactId::FamilyAmount {
        operation: row.operand.relation().operation(),
        side: *side,
        object: *object,
    })
}

/// The first relation owning one capability, in canonical order.
fn capability_owner(
    requirements: &BTreeMap<RelationId, RelationRequirements>,
    capability: RequiredCapability,
) -> Option<RelationId> {
    requirements
        .iter()
        .find(|(_, bundle)| bundle.required_capabilities.contains(&capability))
        .map(|(relation, _)| relation.clone())
}

/// The first relation owning one evidence requirement, in canonical
/// order.
fn evidence_owner(
    requirements: &BTreeMap<RelationId, RelationRequirements>,
    requirement: &ExternalEvidenceRequirement,
) -> Option<RelationId> {
    requirements
        .iter()
        .find(|(_, bundle)| bundle.external_evidence.contains(requirement))
        .map(|(relation, _)| relation.clone())
}
