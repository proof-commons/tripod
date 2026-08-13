//! Relation obligations and exact feasible proof-plan enumeration
//! (Guide-3 Tranches G–I).
//!
//! Every in-scope relation is classified exactly once: proof-required
//! (with its realization-approved alternatives), statically validated,
//! or an external evidence requirement retained unresolved. The exact
//! search then enumerates, depth-first and deterministically, every
//! complete assignment of one proof per proof-required relation and
//! one representation per representation variable, keeping the whole
//! feasible set — hard constraints prune, nothing is weighted, no
//! greedy fallback exists, and complexity exhaustion returns no
//! partial result. Search-state counts live in a diagnostic report,
//! never in a candidate.

use std::collections::{BTreeMap, BTreeSet};

use realization::{
    ProofAlternativeId, Relation, RelationDeclaration, RelationId, RepresentationMode,
};

use crate::{
    CompileError,
    capability::{CapabilityView, RequiredCapability},
    constructibility::{
        CompilerConstructibilityAnalysis, build_constructibility_analysis,
        validate_source_constructibility,
    },
    disclosure::{CompilerDisclosureAnalysis, derive_disclosure, validate_disclosure},
    input::BoundCompilerInput,
    lifecycle::{
        CompilerLifecycleAnalysis, LifecycleRequirement, RepresentationChoice,
        RepresentationChoiceId, build_lifecycle_analysis, proof_supports_representation,
    },
    relation::{CompilerRelationAnalysis, build_relation_analysis},
    search_counter::{admit_search_state, record_search_event},
    source::{
        OperandId, SourceRequirement, derive_source_requirements, proof_capabilities,
        relation_operands,
    },
};

/// How one relation is discharged.
///
/// An externally evidenced relation is not a proof variable, but it is
/// still planned: it carries the realization-approved proof class it
/// will be discharged under, together with the capabilities and source
/// rows that class fixes. Those requirements are not optional, so they
/// bind before the search rather than inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationObligationClass {
    ProofRequired {
        alternatives: Vec<ProofAlternativeId>,
    },
    StaticallyValidated,
    ExternalEvidence {
        requirement: realization::ExternalEvidenceRequirement,
        proof: ProofAlternativeId,
        required_capabilities: BTreeSet<RequiredCapability>,
        source_requirements: Vec<SourceRequirement>,
    },
}

/// One relation's discharge obligation with its operand census.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationObligation {
    pub relation: RelationId,
    pub class: RelationObligationClass,
    pub operands: Vec<OperandId>,
}

/// One complete feasible plan.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProofPlanCandidate {
    pub proofs: BTreeMap<RelationId, ProofAlternativeId>,
    pub representations: BTreeMap<RepresentationChoiceId, RepresentationMode>,
    pub required_capabilities: BTreeSet<RequiredCapability>,
    pub source_requirements: Vec<SourceRequirement>,
    pub external_evidence: BTreeSet<realization::ExternalEvidenceRequirement>,
    pub lifecycle: Vec<LifecycleRequirement>,
    pub disclosure: CompilerDisclosureAnalysis,
}

/// Diagnostic search statistics — never semantic identity.
///
/// Every rejection counter names the cause it counts, and only that
/// cause. A report that folded several distinct causes into one counter
/// would still be a typed account of how analysis was performed, and it
/// would be a false one.
///
/// No counter is carried for a rejection the search cannot perform. A
/// selected representation's lifecycle requirements are filtered into
/// the candidate rather than tested against it, so lifecycle rejection
/// has no site here; a permanently zero counter would read as evidence
/// that the search looked and found nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProofSearchReport {
    pub states_visited: u64,
    pub complete_assignments: u64,
    pub feasible_assignments: u64,
    pub rejected_by_capability: u64,
    pub rejected_by_source: u64,
    pub rejected_by_constructibility: u64,
    pub rejected_by_representation: u64,
    pub rejected_by_disclosure: u64,
}

impl ProofSearchReport {
    /// Record one local rejection under the cause that produced it.
    pub const fn record_local_rejection(&mut self, rejection: LocalProofRejection) {
        let counter = match rejection {
            LocalProofRejection::MissingCapability => &mut self.rejected_by_capability,
            LocalProofRejection::SourceDerivation => &mut self.rejected_by_source,
            LocalProofRejection::Constructibility => &mut self.rejected_by_constructibility,
        };

        record_search_event(counter);
    }
}

/// Why one proof alternative failed the local feasibility filters.
///
/// The three causes are genuinely different questions about the same
/// alternative — whether the target can express the proof, whether the
/// proof's source rows can be derived at all, and whether the derived
/// rows can actually be constructed from the operation's witnesses — so
/// collapsing them loses the only information the rejection carried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalProofRejection {
    /// The capability view does not support the proof's capabilities.
    MissingCapability,
    /// The proof's source requirements could not be derived.
    SourceDerivation,
    /// The derived source rows are not constructible for the relation.
    Constructibility,
}

/// The complete canonical feasible set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeasibleProofPlans {
    pub candidates: Vec<ProofPlanCandidate>,
    pub search: ProofSearchReport,
}

/// Classify every relation in the analysis exactly once.
pub fn classify_obligations(
    relations: &CompilerRelationAnalysis,
) -> Result<Vec<RelationObligation>, CompileError> {
    let mut obligations = Vec::new();

    for node in relations.graph.node_weights() {
        let declaration = &node.source;
        let operands = relation_operands(declaration)?;

        let class = match &declaration.relation {
            // A representation relation constrains which mode the
            // compiler may select; the mode itself remains a decision
            // variable derived from the relation's allowed set, and no
            // arithmetic proof variable is created for the constraint.
            Relation::LifecycleExit { .. } | Relation::Representation { .. } => {
                RelationObligationClass::StaticallyValidated
            }

            Relation::SubstrateConservation { asset } => {
                // The realization approves exactly one proof class for
                // this relation. The compiler validates that invariant
                // rather than manufacturing the class or taking an
                // arbitrary first element.
                let approved = ProofAlternativeId::new(
                    declaration.id.clone(),
                    realization::ProofKind::SubstrateConservation,
                );

                if declaration.proof_alternatives.len() != 1
                    || !declaration.proof_alternatives.contains(&approved)
                {
                    return Err(CompileError::InvalidExternalEvidenceProofAlternatives {
                        relation: declaration.id.clone(),
                    });
                }

                RelationObligationClass::ExternalEvidence {
                    requirement: realization::ExternalEvidenceRequirement::SubstrateConservation {
                        operation: declaration.id.operation(),
                        asset: *asset,
                    },
                    required_capabilities: proof_capabilities(declaration, approved.proof()),
                    source_requirements: derive_source_requirements(declaration, approved.proof())?,
                    proof: approved,
                }
            }

            _ => {
                let mut alternatives = declaration
                    .proof_alternatives
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                alternatives.sort();

                if alternatives.is_empty() {
                    return Err(CompileError::MissingProofAlternative {
                        relation: declaration.id.clone(),
                    });
                }

                RelationObligationClass::ProofRequired { alternatives }
            }
        };

        obligations.push(RelationObligation {
            relation: declaration.id.clone(),
            class,
            operands,
        });
    }

    obligations.sort_by(|left, right| left.relation.cmp(&right.relation));
    Ok(obligations)
}

/// Enumerate the complete feasible plan set for one bound input.
pub fn enumerate_feasible_plans(
    input: &BoundCompilerInput,
    view: &CapabilityView,
) -> Result<FeasibleProofPlans, CompileError> {
    let relations = build_relation_analysis(input)?;
    let constructibility = build_constructibility_analysis(input)?;
    let lifecycle = build_lifecycle_analysis(input, &relations)?;
    let obligations = classify_obligations(&relations)?;

    let declarations = relations
        .graph
        .node_weights()
        .map(|node| (node.source.id.clone(), node.source.clone()))
        .collect::<BTreeMap<_, _>>();

    // Stable variable order: proof obligations by relation ID, then
    // representation choices by choice ID.
    let proof_variables = obligations
        .iter()
        .filter_map(|obligation| match &obligation.class {
            RelationObligationClass::ProofRequired { alternatives } => {
                Some((obligation.relation.clone(), alternatives.clone()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    // External evidence fixes requirements no search choice can trade
    // away: they are collected and filtered before any assignment.
    let mut external_evidence = BTreeSet::new();
    let mut fixed_required_capabilities = BTreeSet::new();
    let mut fixed_source_requirements = Vec::new();
    let mut blocked = Vec::new();

    for obligation in &obligations {
        let RelationObligationClass::ExternalEvidence {
            requirement,
            required_capabilities,
            source_requirements,
            ..
        } = &obligation.class
        else {
            continue;
        };

        validate_source_constructibility(
            &constructibility,
            &obligation.relation,
            source_requirements,
        )?;

        if !view.supports(required_capabilities) {
            blocked.push(obligation.relation.clone());
        }

        external_evidence.insert(requirement.clone());
        fixed_required_capabilities.extend(required_capabilities.iter().copied());
        fixed_source_requirements.extend(source_requirements.iter().cloned());
    }

    fixed_source_requirements.sort();
    fixed_source_requirements.dedup();

    // Prepass: a relation whose every alternative fails the local
    // (representation-independent) filters is blocked.
    for (relation, alternatives) in &proof_variables {
        let declaration = &declarations[relation];
        let locally_feasible = alternatives.iter().any(|alternative| {
            locally_feasible(declaration, alternative, view, &constructibility).is_ok()
        });

        if !locally_feasible {
            blocked.push(relation.clone());
        }
    }

    if !blocked.is_empty() {
        blocked.sort();
        return Err(CompileError::NoFeasibleProofPlan {
            blocked_relations: blocked,
        });
    }

    let limits = input.policy().proof_search_limits;
    let mut search = ProofSearchReport::default();
    let mut candidates = Vec::new();
    let mut state = SearchState {
        input,
        view,
        declarations: &declarations,
        constructibility: &constructibility,
        lifecycle: &lifecycle,
        proof_variables: &proof_variables,
        external_evidence: &external_evidence,
        fixed_required_capabilities: &fixed_required_capabilities,
        fixed_source_requirements: &fixed_source_requirements,
        limits,
        search: &mut search,
        candidates: &mut candidates,
    };

    visit(&mut state, BTreeMap::new(), BTreeMap::new(), 0)?;

    if candidates.is_empty() {
        // Locally every relation was feasible: the conflict is global.
        let mut blocked = proof_variables
            .iter()
            .map(|(relation, _)| relation.clone())
            .collect::<Vec<_>>();
        blocked.sort();

        return Err(CompileError::NoFeasibleProofPlan {
            blocked_relations: blocked,
        });
    }

    candidates.sort();
    candidates.dedup();

    search.feasible_assignments =
        u64::try_from(candidates.len()).expect("candidate count fits u64");

    Ok(FeasibleProofPlans { candidates, search })
}

struct SearchState<'a> {
    input: &'a BoundCompilerInput,
    view: &'a CapabilityView,
    declarations: &'a BTreeMap<RelationId, RelationDeclaration>,
    constructibility: &'a CompilerConstructibilityAnalysis,
    lifecycle: &'a CompilerLifecycleAnalysis,
    proof_variables: &'a [(RelationId, Vec<ProofAlternativeId>)],
    external_evidence: &'a BTreeSet<realization::ExternalEvidenceRequirement>,
    fixed_required_capabilities: &'a BTreeSet<RequiredCapability>,
    fixed_source_requirements: &'a [SourceRequirement],
    limits: crate::input::ProofSearchLimits,
    search: &'a mut ProofSearchReport,
    candidates: &'a mut Vec<ProofPlanCandidate>,
}

/// Local (representation-independent) feasibility of one alternative:
/// capability containment and witness constructibility.
///
/// Returns the alternative's capabilities and source rows when
/// feasible, and otherwise the typed cause of the rejection, so the
/// search records the failure it actually observed.
///
/// # Errors
///
/// The [`LocalProofRejection`] the alternative failed under.
pub fn locally_feasible(
    declaration: &RelationDeclaration,
    alternative: &ProofAlternativeId,
    view: &CapabilityView,
    constructibility: &CompilerConstructibilityAnalysis,
) -> Result<(BTreeSet<RequiredCapability>, Vec<SourceRequirement>), LocalProofRejection> {
    let capabilities = proof_capabilities(declaration, alternative.proof());

    if !view.supports(&capabilities) {
        return Err(LocalProofRejection::MissingCapability);
    }

    let rows = derive_source_requirements(declaration, alternative.proof())
        .map_err(|_| LocalProofRejection::SourceDerivation)?;

    validate_source_constructibility(constructibility, &declaration.id, &rows)
        .map_err(|_| LocalProofRejection::Constructibility)?;

    Ok((capabilities, rows))
}

fn visit(
    state: &mut SearchState<'_>,
    proofs: BTreeMap<RelationId, ProofAlternativeId>,
    representations: BTreeMap<RepresentationChoiceId, RepresentationMode>,
    depth: usize,
) -> Result<(), CompileError> {
    admit_search_state(
        &mut state.search.states_visited,
        state.limits.maximum_states,
    )
    .map_err(|maximum| CompileError::ProofSearchStateLimitExceeded { maximum })?;

    let proof_count = state.proof_variables.len();
    let choices: &[RepresentationChoice] = &state.lifecycle.choices;

    if depth < proof_count {
        let (relation, alternatives) = &state.proof_variables[depth];
        let declaration = state.declarations[relation].clone();

        for alternative in alternatives {
            if let Err(rejection) = locally_feasible(
                &declaration,
                alternative,
                state.view,
                state.constructibility,
            ) {
                state.search.record_local_rejection(rejection);
                continue;
            }

            let mut next = proofs.clone();
            next.insert(relation.clone(), alternative.clone());
            visit(state, next, representations.clone(), depth + 1)?;
        }

        return Ok(());
    }

    let choice_index = depth - proof_count;

    if let Some(choice) = choices.get(choice_index) {
        for mode in &choice.candidates {
            // Prune a proof already incompatible with this mode.
            if representation_conflict(state.declarations, &proofs, &choice.id, *mode).is_some() {
                record_search_event(&mut state.search.rejected_by_representation);
                continue;
            }

            let mut next = representations.clone();
            next.insert(choice.id.clone(), *mode);
            visit(state, proofs.clone(), next, depth + 1)?;
        }

        return Ok(());
    }

    // Complete assignment.
    record_search_event(&mut state.search.complete_assignments);
    complete(state, proofs, representations)
}

/// The selected proof for an amount relation over the choice's object,
/// when that proof cannot support the mode.
fn representation_conflict(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    proofs: &BTreeMap<RelationId, ProofAlternativeId>,
    choice: &RepresentationChoiceId,
    mode: RepresentationMode,
) -> Option<RelationId> {
    for (relation, alternative) in proofs {
        if relation.operation() != choice.operation {
            continue;
        }

        let declaration = &declarations[relation];
        let touches_object = match &declaration.relation {
            Relation::AmountConservation {
                input_objects,
                output_objects,
                ..
            } => input_objects.contains(&choice.object) || output_objects.contains(&choice.object),
            _ => false,
        };

        if touches_object && !proof_supports_representation(alternative.proof(), mode) {
            return Some(relation.clone());
        }
    }

    None
}

fn complete(
    state: &mut SearchState<'_>,
    proofs: BTreeMap<RelationId, ProofAlternativeId>,
    representations: BTreeMap<RepresentationChoiceId, RepresentationMode>,
) -> Result<(), CompileError> {
    // Derive per-candidate capabilities and sources, beginning with
    // the fixed external-evidence requirements every candidate carries.
    let mut required_capabilities = state.fixed_required_capabilities.clone();
    let mut source_requirements = state.fixed_source_requirements.to_vec();

    for (relation, alternative) in &proofs {
        let declaration = &state.declarations[relation];
        required_capabilities.extend(proof_capabilities(declaration, alternative.proof()));

        // Source-row derivation, not constructibility: the rows could
        // not be produced at all, so nothing was ever checked against
        // the operation's witnesses.
        if let Ok(rows) = derive_source_requirements(declaration, alternative.proof()) {
            source_requirements.extend(rows);
        } else {
            record_search_event(&mut state.search.rejected_by_source);
            return Ok(());
        }
    }

    source_requirements.sort();
    source_requirements.dedup();

    // Candidate lifecycle: the requirements of the selected modes.
    let lifecycle = state
        .lifecycle
        .requirements
        .iter()
        .filter(|requirement| {
            representations.iter().any(|(choice, mode)| {
                choice.object == requirement.object && *mode == requirement.representation
            })
        })
        .cloned()
        .collect::<Vec<_>>();

    // Candidate disclosure, validated.
    let Ok(disclosure) = derive_disclosure(
        state.input.realization().declassification(),
        &representations,
    ) else {
        record_search_event(&mut state.search.rejected_by_disclosure);
        return Ok(());
    };

    if validate_disclosure(&disclosure).is_err() {
        record_search_event(&mut state.search.rejected_by_disclosure);
        return Ok(());
    }

    let candidate = ProofPlanCandidate {
        proofs,
        representations,
        required_capabilities,
        source_requirements,
        external_evidence: state.external_evidence.clone(),
        lifecycle,
        disclosure,
    };

    if u64::try_from(state.candidates.len()).expect("count fits u64")
        >= state.limits.maximum_candidates.get()
    {
        return Err(CompileError::ProofCandidateLimitExceeded {
            maximum: state.limits.maximum_candidates.get(),
        });
    }

    state.candidates.push(candidate);
    Ok(())
}
