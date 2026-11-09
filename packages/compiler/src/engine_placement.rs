//! Shared exact placement search and independent assignment validation.
//! Owner-specific atoms below are used only for equality and ordering; policy
//! arrives as finite admissibility records, never as an identity dispatch.
use crate::{
    carrier::CarrierQuantification,
    engine_records::RelationKey,
    placement::{CarrierMultiplicity, PlacementSearchLimits, PlacementSearchReport},
    search_counter::{admit_search_state, record_search_event},
};
#[cfg(test)]
use crate::{engine_lifecycle::ObjectKey, engine_records::OperationKey};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CaseKey {
    Attestation(crate::case::ExecutionCaseId),
    #[cfg(test)]
    Keyed(OperationKey),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CarrierKey {
    Attestation(crate::carrier::CarrierRole),
    #[cfg(test)]
    Member {
        operation: OperationKey,
        object: ObjectKey,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayoutKey(pub crate::layout::LayoutRequirement);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub relation: RelationKey,
    pub case: CaseKey,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Placed {
    pub carrier: CarrierKey,
    pub quantification: CarrierQuantification,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Assignment {
    pub key: Key,
    pub carriers: Vec<Placed>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Candidate {
    pub assignments: Vec<Assignment>,
    pub layout: Vec<LayoutKey>,
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub placed: Placed,
    pub layout: BTreeSet<LayoutKey>,
}
#[derive(Clone, Copy, Debug)]
pub enum EmptyCause {
    NoEligible,
    GlobalOnlyLocal,
    UnconditionalOptional,
}
#[derive(Clone, Debug)]
pub struct Eligibility {
    pub key: Key,
    pub multiplicity: CarrierMultiplicity,
    pub entries: Vec<Entry>,
    pub options: Vec<Entry>,
    pub empty_cause: EmptyCause,
}
#[derive(Clone, Debug)]
pub struct Records {
    pub plans: Vec<(Key, bool)>,
    pub eligibility: Vec<Eligibility>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    PlacementCensusMismatch {
        missing: Vec<Key>,
        unexpected: Vec<Key>,
    },
    NoEligibleCarrier(Key),
    GlobalRelationHasOnlyLocalCarrier(Key),
    UnconditionalRelationOnOptionalCarrier(Key),
    DuplicatePlacedCarrier {
        key: Key,
        carrier: CarrierKey,
    },
    NonCanonicalCarrierAssignment(Key),
    UnpermittedCarrierPlacement {
        key: Key,
        carrier: CarrierKey,
    },
    MissingLayoutRequirement(Key),
    UnexpectedLayoutRequirement(Vec<LayoutKey>),
    StateLimit(u64),
    CandidateLimit(u64),
}

pub fn validate_census(records: &Records) -> Result<(), Error> {
    let required = records
        .plans
        .iter()
        .filter(|(_, runtime)| *runtime)
        .map(|(key, _)| key.clone())
        .collect::<BTreeSet<_>>();
    let mut carried = BTreeSet::new();
    let mut duplicated = BTreeSet::new();
    for row in &records.eligibility {
        if !carried.insert(row.key.clone()) {
            duplicated.insert(row.key.clone());
        }
    }
    if carried != required || !duplicated.is_empty() {
        let unexpected = carried
            .difference(&required)
            .cloned()
            .chain(duplicated)
            .collect::<BTreeSet<_>>();
        return Err(Error::PlacementCensusMismatch {
            missing: required.difference(&carried).cloned().collect(),
            unexpected: unexpected.into_iter().collect(),
        });
    }
    Ok(())
}
pub fn validate_placement(records: &Records, placement: &Candidate) -> Result<(), Error> {
    validate_census(records)?;
    let assigned = placement
        .assignments
        .iter()
        .map(|row| row.key.clone())
        .collect::<BTreeSet<_>>();
    let carried = records
        .eligibility
        .iter()
        .map(|row| row.key.clone())
        .collect::<BTreeSet<_>>();
    if assigned != carried || assigned.len() != placement.assignments.len() {
        return Err(Error::PlacementCensusMismatch {
            missing: carried.difference(&assigned).cloned().collect(),
            unexpected: assigned.difference(&carried).cloned().collect(),
        });
    }
    let mut stated = BTreeSet::new();
    let mut surplus = BTreeSet::new();
    for requirement in &placement.layout {
        if !stated.insert(requirement.clone()) {
            surplus.insert(requirement.clone());
        }
    }
    let mut depended = BTreeSet::new();
    for assignment in &placement.assignments {
        depended.extend(validate_assignment(records, assignment, &stated)?);
    }
    surplus.extend(
        stated
            .into_iter()
            .filter(|requirement| !depended.contains(requirement)),
    );
    if !surplus.is_empty() {
        return Err(Error::UnexpectedLayoutRequirement(
            surplus.into_iter().collect(),
        ));
    }
    Ok(())
}
fn validate_assignment(
    records: &Records,
    assignment: &Assignment,
    stated: &BTreeSet<LayoutKey>,
) -> Result<BTreeSet<LayoutKey>, Error> {
    let key = &assignment.key;
    let census = || Error::PlacementCensusMismatch {
        missing: vec![key.clone()],
        unexpected: Vec::new(),
    };
    let row = records
        .eligibility
        .iter()
        .find(|row| row.key == *key)
        .ok_or_else(census)?;
    if !records.plans.iter().any(|(plan, _)| plan == key) {
        return Err(census());
    }
    if assignment.carriers.is_empty() {
        return Err(Error::NoEligibleCarrier(key.clone()));
    }
    let mut selected = BTreeSet::new();
    for placed in &assignment.carriers {
        if !selected.insert(&placed.carrier) {
            return Err(Error::DuplicatePlacedCarrier {
                key: key.clone(),
                carrier: placed.carrier.clone(),
            });
        }
    }
    let canonical = match row.multiplicity {
        CarrierMultiplicity::ExactlyOne
        | CarrierMultiplicity::EveryMember
        | CarrierMultiplicity::AtLeastOne => assignment.carriers.len() == 1,
        CarrierMultiplicity::DeliberateDuplication => {
            selected
                == row
                    .entries
                    .iter()
                    .map(|entry| &entry.placed.carrier)
                    .collect::<BTreeSet<_>>()
        }
    };
    if !canonical {
        return Err(Error::NonCanonicalCarrierAssignment(key.clone()));
    }
    let mut depended = BTreeSet::new();
    for placed in &assignment.carriers {
        let unpermitted = || Error::UnpermittedCarrierPlacement {
            key: key.clone(),
            carrier: placed.carrier.clone(),
        };
        let entry = row
            .entries
            .iter()
            .find(|entry| entry.placed.carrier == placed.carrier)
            .ok_or_else(unpermitted)?;
        if entry.placed.quantification != placed.quantification {
            return Err(unpermitted());
        }
        for requirement in &entry.layout {
            if !stated.contains(requirement) {
                return Err(Error::MissingLayoutRequirement(key.clone()));
            }
            depended.insert(requirement.clone());
        }
    }
    Ok(depended)
}
#[derive(Clone)]
struct OptionRow {
    carriers: Vec<Placed>,
    layout: BTreeSet<LayoutKey>,
}
fn options(row: &Eligibility) -> Result<Vec<OptionRow>, Error> {
    if row.options.is_empty() {
        return Err(match row.empty_cause {
            EmptyCause::NoEligible => Error::NoEligibleCarrier(row.key.clone()),
            EmptyCause::GlobalOnlyLocal => {
                Error::GlobalRelationHasOnlyLocalCarrier(row.key.clone())
            }
            EmptyCause::UnconditionalOptional => {
                Error::UnconditionalRelationOnOptionalCarrier(row.key.clone())
            }
        });
    }
    let option = |entries: Vec<&Entry>| {
        let mut carriers = entries
            .iter()
            .map(|entry| entry.placed.clone())
            .collect::<Vec<_>>();
        carriers.sort();
        carriers.dedup();
        OptionRow {
            carriers,
            layout: entries
                .into_iter()
                .flat_map(|entry| entry.layout.iter().cloned())
                .collect(),
        }
    };
    Ok(match row.multiplicity {
        CarrierMultiplicity::ExactlyOne
        | CarrierMultiplicity::AtLeastOne
        | CarrierMultiplicity::EveryMember => row
            .options
            .iter()
            .map(|entry| option(vec![entry]))
            .collect(),
        CarrierMultiplicity::DeliberateDuplication => vec![option(row.options.iter().collect())],
    })
}
pub fn enumerate(
    records: &Records,
    limits: PlacementSearchLimits,
    report: &mut PlacementSearchReport,
) -> Result<Vec<Candidate>, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_placement", "enumerate", obligation_count = crate::instrumentation::count(records.eligibility.len()), state_limit = limits.maximum_states.get(), candidate_limit = limits.maximum_candidates.get());
    let _entered = span.enter();
    let result = enumerate_core(records, limits, report);
    let refusal_kind = result.as_ref().err().map(|error| match error {
        Error::PlacementCensusMismatch { .. } => "placement_census_mismatch",
        Error::NoEligibleCarrier(_) => "no_eligible_carrier",
        Error::GlobalRelationHasOnlyLocalCarrier(_) => "global_relation_has_only_local_carrier",
        Error::UnconditionalRelationOnOptionalCarrier(_) => {
            "unconditional_relation_on_optional_carrier"
        }
        Error::DuplicatePlacedCarrier { .. } => "duplicate_placed_carrier",
        Error::NonCanonicalCarrierAssignment(_) => "non_canonical_carrier_assignment",
        Error::UnpermittedCarrierPlacement { .. } => "unpermitted_carrier_placement",
        Error::MissingLayoutRequirement(_) => "missing_layout_requirement",
        Error::UnexpectedLayoutRequirement(_) => "unexpected_layout_requirement",
        Error::StateLimit(_) => "placement_search_state_limit_exceeded",
        Error::CandidateLimit(_) => "placement_candidate_limit_exceeded",
    });
    tracing::debug!(target: "compiler::engine_placement",
        outcome = if result.is_ok() { "completed" } else { "refused" }, refusal_kind,
        visited_state_count = report.states_visited,
        complete_assignment_count = report.complete_assignments,
        feasible_placement_count = report.feasible_placements,
        retained_option_count = report.retained_options,
        message = "placement search completed");
    result
}
fn enumerate_core(
    records: &Records,
    limits: PlacementSearchLimits,
    report: &mut PlacementSearchReport,
) -> Result<Vec<Candidate>, Error> {
    validate_census(records)?;
    let rows = records
        .eligibility
        .iter()
        .map(|row| (row.key.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut keys = Vec::new();
    let mut retained = Vec::new();
    for (key, row) in rows {
        keys.push(key);
        retained.push(options(row)?);
    }
    report.retained_options = retained
        .iter()
        .map(|rows| u64::try_from(rows.len()).unwrap_or(u64::MAX))
        .sum();
    let mut state = Search {
        records,
        keys,
        options: retained,
        limits,
        report,
        candidates: Vec::new(),
    };
    visit(&mut state, &mut Vec::new(), 0)?;
    state.candidates.sort();
    state.candidates.dedup();
    state.report.feasible_placements = u64::try_from(state.candidates.len()).unwrap_or(u64::MAX);
    Ok(state.candidates)
}
struct Search<'a> {
    records: &'a Records,
    keys: Vec<Key>,
    options: Vec<Vec<OptionRow>>,
    limits: PlacementSearchLimits,
    report: &'a mut PlacementSearchReport,
    candidates: Vec<Candidate>,
}
fn visit(state: &mut Search<'_>, chosen: &mut Vec<usize>, depth: usize) -> Result<(), Error> {
    admit_search_state(
        &mut state.report.states_visited,
        state.limits.maximum_states,
    )
    .map_err(Error::StateLimit)?;
    if depth == state.options.len() {
        record_search_event(&mut state.report.complete_assignments);
        let mut assignments = Vec::new();
        let mut layout = BTreeSet::new();
        for ((key, options), chosen) in state.keys.iter().zip(&state.options).zip(chosen.iter()) {
            let option = &options[*chosen];
            assignments.push(Assignment {
                key: key.clone(),
                carriers: option.carriers.clone(),
            });
            layout.extend(option.layout.iter().cloned());
        }
        assignments.sort();
        let candidate = Candidate {
            assignments,
            layout: layout.into_iter().collect(),
        };
        validate_placement(state.records, &candidate)?;
        if u64::try_from(state.candidates.len()).unwrap_or(u64::MAX)
            >= state.limits.maximum_candidates.get()
        {
            return Err(Error::CandidateLimit(state.limits.maximum_candidates.get()));
        }
        state.candidates.push(candidate);
        return Ok(());
    }
    for index in 0..state.options[depth].len() {
        chosen.push(index);
        let result = visit(state, chosen, depth + 1);
        chosen.pop();
        result?;
    }
    Ok(())
}
