//! One exact proof search over finite owner tables, with no policy callbacks.
use crate::{
    capability::{CapabilityView, RequiredCapability},
    engine_lifecycle::{LifecycleRequirementRecord, ModeKey, ObjectKey},
    engine_records::{OperationKey, RelationKey},
    engine_source::{Evidence, SourceRow},
    input::ProofSearchLimits,
    proof::{LocalProofRejection, ProofSearchReport},
    search_counter::{admit_search_state, record_search_event},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AlternativeKey {
    pub relation: RelationKey,
    pub namespace: u16,
    pub kind: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChoiceKey {
    pub operation: OperationKey,
    pub object: ObjectKey,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactKey {
    pub namespace: u16,
    pub subject: Vec<u16>,
}
#[derive(Clone, Debug)]
pub struct Alternative {
    pub id: AlternativeKey,
    pub capabilities: BTreeSet<RequiredCapability>,
    pub sources: Result<Vec<SourceRow>, LocalProofRejection>,
    pub constructible: bool,
    pub evidence: BTreeSet<Evidence>,
    pub incompatible: BTreeSet<(ChoiceKey, ModeKey)>,
}
#[derive(Clone, Debug)]
pub enum Obligation {
    Static,
    Evidence(Alternative),
    Required(Vec<Alternative>),
}
#[derive(Clone, Debug)]
pub struct DisclosureRecords {
    pub valid: bool,
    pub retained: BTreeSet<FactKey>,
    pub public_effects: BTreeMap<(ChoiceKey, ModeKey), BTreeSet<FactKey>>,
}
#[derive(Clone, Debug)]
pub struct Records {
    pub obligations: BTreeMap<RelationKey, Obligation>,
    pub choices: Vec<(ChoiceKey, Vec<ModeKey>)>,
    pub lifecycle: Vec<LifecycleRequirementRecord>,
    pub disclosure: DisclosureRecords,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Candidate {
    pub proofs: BTreeMap<RelationKey, AlternativeKey>,
    pub representations: BTreeMap<ChoiceKey, ModeKey>,
    pub capabilities: BTreeSet<RequiredCapability>,
    pub sources: Vec<SourceRow>,
    pub evidence: BTreeSet<Evidence>,
    pub lifecycle: Vec<LifecycleRequirementRecord>,
    pub disclosed: BTreeMap<FactKey, BTreeSet<(ChoiceKey, ModeKey)>>,
    pub retained: BTreeSet<FactKey>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    NoFeasiblePlan(Vec<RelationKey>),
    StateLimit(u64),
    CandidateLimit(u64),
    FixedSource(RelationKey),
}

pub fn enumerate(
    records: &Records,
    view: &CapabilityView,
    limits: ProofSearchLimits,
    report: &mut ProofSearchReport,
) -> Result<Vec<Candidate>, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_proof", "enumerate", relation_count = crate::instrumentation::count(records.obligations.len()), choice_count = crate::instrumentation::count(records.choices.len()));
    let _entered = span.enter();
    let mut variables = Vec::new();
    let mut fixed = Vec::new();
    let mut blocked = Vec::new();
    for (relation, obligation) in &records.obligations {
        match obligation {
            Obligation::Static => {}
            Obligation::Evidence(alternative) => {
                if alternative.sources.is_err() || !alternative.constructible {
                    return Err(Error::FixedSource(relation.clone()));
                }
                if !view.supports(&alternative.capabilities) {
                    blocked.push(relation.clone());
                }
                fixed.push(alternative);
            }
            Obligation::Required(alternatives) => {
                if !alternatives
                    .iter()
                    .any(|alternative| local(alternative, view).is_ok())
                {
                    blocked.push(relation.clone());
                }
                variables.push((relation.clone(), alternatives));
            }
        }
    }
    if !blocked.is_empty() {
        blocked.sort();
        return Err(Error::NoFeasiblePlan(blocked));
    }
    let mut state = Search {
        records,
        variables,
        fixed,
        view,
        limits,
        report,
        candidates: Vec::new(),
    };
    visit(&mut state, BTreeMap::new(), BTreeMap::new(), 0)?;
    if state.candidates.is_empty() {
        return Err(Error::NoFeasiblePlan(
            state
                .variables
                .iter()
                .map(|(relation, _)| relation.clone())
                .collect(),
        ));
    }
    state.candidates.sort();
    state.candidates.dedup();
    state.report.feasible_assignments = u64::try_from(state.candidates.len()).unwrap_or(u64::MAX);
    tracing::debug!(target: "compiler::engine_proof", outcome = "completed", states = state.report.states_visited, candidates = state.report.feasible_assignments, "proof records searched");
    Ok(state.candidates)
}
fn local(alternative: &Alternative, view: &CapabilityView) -> Result<(), LocalProofRejection> {
    if !view.supports(&alternative.capabilities) {
        return Err(LocalProofRejection::MissingCapability);
    }
    alternative.sources.as_ref().map_err(|error| *error)?;
    if !alternative.constructible {
        return Err(LocalProofRejection::Constructibility);
    }
    Ok(())
}
struct Search<'a> {
    records: &'a Records,
    variables: Vec<(RelationKey, &'a Vec<Alternative>)>,
    fixed: Vec<&'a Alternative>,
    view: &'a CapabilityView,
    limits: ProofSearchLimits,
    report: &'a mut ProofSearchReport,
    candidates: Vec<Candidate>,
}
fn visit(
    state: &mut Search<'_>,
    proofs: BTreeMap<RelationKey, AlternativeKey>,
    representations: BTreeMap<ChoiceKey, ModeKey>,
    depth: usize,
) -> Result<(), Error> {
    admit_search_state(
        &mut state.report.states_visited,
        state.limits.maximum_states,
    )
    .map_err(Error::StateLimit)?;
    if let Some((relation, alternatives)) = state.variables.get(depth).cloned() {
        for alternative in alternatives {
            if let Err(rejection) = local(alternative, state.view) {
                state.report.record_local_rejection(rejection);
                continue;
            }
            let mut next = proofs.clone();
            next.insert(relation.clone(), alternative.id.clone());
            visit(state, next, representations.clone(), depth + 1)?;
        }
        return Ok(());
    }
    if let Some((choice, modes)) = state
        .records
        .choices
        .get(depth - state.variables.len())
        .cloned()
    {
        for mode in modes {
            if selected(state, &proofs)
                .into_iter()
                .any(|alternative| alternative.incompatible.contains(&(choice, mode)))
            {
                record_search_event(&mut state.report.rejected_by_representation);
                continue;
            }
            let mut next = representations.clone();
            next.insert(choice, mode);
            visit(state, proofs.clone(), next, depth + 1)?;
        }
        return Ok(());
    }
    record_search_event(&mut state.report.complete_assignments);
    complete(state, proofs, representations)
}
fn selected<'a>(
    state: &Search<'a>,
    proofs: &BTreeMap<RelationKey, AlternativeKey>,
) -> Vec<&'a Alternative> {
    state
        .variables
        .iter()
        .flat_map(|(relation, alternatives)| {
            alternatives
                .iter()
                .filter(move |alternative| proofs.get(relation) == Some(&alternative.id))
        })
        .collect()
}
fn complete(
    state: &mut Search<'_>,
    proofs: BTreeMap<RelationKey, AlternativeKey>,
    representations: BTreeMap<ChoiceKey, ModeKey>,
) -> Result<(), Error> {
    let mut capabilities = BTreeSet::new();
    let mut sources = Vec::new();
    let mut evidence = BTreeSet::new();
    for alternative in state.fixed.iter().copied().chain(selected(state, &proofs)) {
        capabilities.extend(alternative.capabilities.iter().copied());
        evidence.extend(alternative.evidence.iter().cloned());
        let Ok(rows) = &alternative.sources else {
            record_search_event(&mut state.report.rejected_by_source);
            return Ok(());
        };
        sources.extend(rows.iter().cloned());
    }
    sources.sort();
    sources.dedup();
    let Some(DisclosureProjection {
        retained,
        disclosed,
    }) = disclose(&state.records.disclosure, &representations)
    else {
        record_search_event(&mut state.report.rejected_by_disclosure);
        return Ok(());
    };
    let lifecycle = state
        .records
        .lifecycle
        .iter()
        .filter(|row| {
            representations
                .iter()
                .any(|(choice, mode)| choice.object == row.object && *mode == row.representation)
        })
        .cloned()
        .collect();
    if u64::try_from(state.candidates.len()).unwrap_or(u64::MAX)
        >= state.limits.maximum_candidates.get()
    {
        return Err(Error::CandidateLimit(state.limits.maximum_candidates.get()));
    }
    state.candidates.push(Candidate {
        proofs,
        representations,
        capabilities,
        sources,
        evidence,
        lifecycle,
        disclosed,
        retained,
    });
    Ok(())
}

pub struct DisclosureProjection {
    pub retained: BTreeSet<FactKey>,
    pub disclosed: BTreeMap<FactKey, BTreeSet<(ChoiceKey, ModeKey)>>,
}
pub fn disclose(
    records: &DisclosureRecords,
    representations: &BTreeMap<ChoiceKey, ModeKey>,
) -> Option<DisclosureProjection> {
    if !records.valid {
        return None;
    }
    let mut retained = records.retained.clone();
    let mut disclosed: BTreeMap<FactKey, BTreeSet<(ChoiceKey, ModeKey)>> = BTreeMap::new();
    for (choice, mode) in representations {
        if let Some(facts) = records.public_effects.get(&(*choice, *mode)) {
            for fact in facts {
                if retained.remove(fact) {
                    disclosed
                        .entry(fact.clone())
                        .or_default()
                        .insert((*choice, *mode));
                }
            }
        }
    }
    Some(DisclosureProjection {
        retained,
        disclosed,
    })
}
