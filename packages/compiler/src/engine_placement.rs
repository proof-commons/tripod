//! Shared exact placement search and independent assignment validation.
//! Owner-specific atoms below are used only for equality and ordering; policy
//! arrives as finite admissibility records, never as an identity dispatch.
use crate::{
    carrier::CarrierQuantification,
    engine_records::RelationKey,
    placement::{CarrierMultiplicity, PlacementSearchLimits, PlacementSearchReport},
    search_counter::{admit_search_state, record_search_event},
};
use std::collections::{BTreeMap, BTreeSet};

pub use crate::engine_identity::{CarrierKey, CaseKey, LayoutKey};
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct OptionRow {
    carriers: Vec<Placed>,
    layout: BTreeSet<LayoutKey>,
}

/// One checked local choice, with exactly its own layout dependencies.
#[allow(dead_code)] // The product path is wired into production by packet 3.2e.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactorOption {
    carriers: Vec<Placed>,
    layout: BTreeSet<LayoutKey>,
}

#[allow(dead_code)] // Product consumers arrive with the shared-storage packet.
impl FactorOption {
    /// The canonical carrier assignment fixed by this local choice.
    pub fn carriers(&self) -> &[Placed] {
        &self.carriers
    }

    /// The layout dependencies of this choice alone.
    pub const fn layout(&self) -> &BTreeSet<LayoutKey> {
        &self.layout
    }
}

/// The exact placement language as independent, checked relation-case factors.
/// A zero-factor product has one empty member.
#[allow(dead_code)] // Packet 3.2d exercises this through its selected tests only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Product {
    factors: BTreeMap<Key, Vec<FactorOption>>,
}

#[allow(dead_code)] // Packet 3.2e selects product storage for Clear.
impl Product {
    /// Inspect factors without expanding their Cartesian product.
    pub fn factors(&self) -> impl Iterator<Item = (&Key, &[FactorOption])> {
        self.factors
            .iter()
            .map(|(key, options)| (key, options.as_slice()))
    }

    /// Multiply factor sizes when the exact cardinality fits in `u128`.
    pub fn checked_cardinality(&self) -> Option<u128> {
        self.factors.values().try_fold(1_u128, |count, options| {
            count.checked_mul(u128::try_from(options.len()).ok()?)
        })
    }

    /// Check one complete member and the exact union of its local layouts.
    ///
    /// # Errors
    ///
    /// The existing census, carrier, quantification or layout refusal for the
    /// first changed coordinate.
    pub fn validate_member(&self, member: &Candidate) -> Result<(), Error> {
        let required = self.factors.keys().cloned().collect::<BTreeSet<_>>();
        let offered = member
            .assignments
            .iter()
            .map(|assignment| assignment.key.clone())
            .collect::<BTreeSet<_>>();
        if required != offered || offered.len() != member.assignments.len() {
            return Err(Error::PlacementCensusMismatch {
                missing: required.difference(&offered).cloned().collect(),
                unexpected: offered.difference(&required).cloned().collect(),
            });
        }
        let mut layout = BTreeSet::new();
        for assignment in &member.assignments {
            let mut carriers = BTreeSet::new();
            for placed in &assignment.carriers {
                if !carriers.insert(&placed.carrier) {
                    return Err(Error::DuplicatePlacedCarrier {
                        key: assignment.key.clone(),
                        carrier: placed.carrier.clone(),
                    });
                }
            }
            let options = &self.factors[&assignment.key];
            let option = options
                .iter()
                .find(|option| option.carriers == assignment.carriers)
                .ok_or_else(|| {
                    let carrier = assignment
                        .carriers
                        .first()
                        .map(|placed| placed.carrier.clone());
                    carrier.map_or_else(
                        || Error::NoEligibleCarrier(assignment.key.clone()),
                        |carrier| Error::UnpermittedCarrierPlacement {
                            key: assignment.key.clone(),
                            carrier,
                        },
                    )
                })?;
            layout.extend(option.layout.iter().cloned());
        }
        let mut stated = BTreeSet::new();
        for requirement in &member.layout {
            if !stated.insert(requirement.clone()) {
                return Err(Error::UnexpectedLayoutRequirement(vec![
                    requirement.clone(),
                ]));
            }
        }
        if let Some(missing) = layout.difference(&stated).next() {
            let key = self.factors.iter().find_map(|(key, options)| {
                member
                    .assignments
                    .iter()
                    .any(|assignment| {
                        assignment.key == *key
                            && options.iter().any(|option| {
                                option.carriers == assignment.carriers
                                    && option.layout.contains(missing)
                            })
                    })
                    .then_some((*key).clone())
            });
            return Err(key.map_or_else(
                || Error::UnexpectedLayoutRequirement(vec![missing.clone()]),
                Error::MissingLayoutRequirement,
            ));
        }
        let unexpected = stated.difference(&layout).cloned().collect::<Vec<_>>();
        if !unexpected.is_empty() {
            return Err(Error::UnexpectedLayoutRequirement(unexpected));
        }
        Ok(())
    }

    /// Materialize only when the complete product fits the explicit budgets.
    ///
    /// # Errors
    ///
    /// [`Error::CandidateLimit`] before expansion when cardinality exceeds
    /// the candidate cap; [`Error::StateLimit`] if bounded traversal exhausts.
    pub fn expand(
        &self,
        limits: PlacementSearchLimits,
        report: &mut PlacementSearchReport,
    ) -> Result<Vec<Candidate>, Error> {
        let span = tracing::debug_span!(target: "compiler::engine_placement", "expand_product", factor_count = crate::instrumentation::count(self.factors.len()), state_limit = limits.maximum_states.get(), candidate_limit = limits.maximum_candidates.get());
        let _entered = span.enter();
        let count = self.factors.values().try_fold(1_u64, |count, options| {
            count.checked_mul(u64::try_from(options.len()).ok()?)
        });
        if count.is_none_or(|count| count > limits.maximum_candidates.get()) {
            return Err(Error::CandidateLimit(limits.maximum_candidates.get()));
        }
        let mut result = Vec::new();
        let factors = self.factors().collect::<Vec<_>>();
        expand_factors(&factors, 0, &mut Vec::new(), &mut result, limits, report)?;
        report.feasible_placements = u64::try_from(result.len()).unwrap_or(u64::MAX);
        tracing::debug!(target: "compiler::engine_placement", candidate_count = report.feasible_placements, visited_state_count = report.states_visited, message = "bounded product expansion completed");
        Ok(result)
    }
}

#[allow(dead_code)] // Only the dormant product accessor calls this traversal.
fn expand_factors(
    factors: &[(&Key, &[FactorOption])],
    depth: usize,
    chosen: &mut Vec<Assignment>,
    result: &mut Vec<Candidate>,
    limits: PlacementSearchLimits,
    report: &mut PlacementSearchReport,
) -> Result<(), Error> {
    admit_search_state(&mut report.states_visited, limits.maximum_states)
        .map_err(Error::StateLimit)?;
    if depth == factors.len() {
        record_search_event(&mut report.complete_assignments);
        let layout = chosen
            .iter()
            .zip(factors)
            .flat_map(|(assignment, (_, options))| {
                options
                    .iter()
                    .find(|option| option.carriers == assignment.carriers)
                    .into_iter()
                    .flat_map(|option| option.layout.iter().cloned())
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        result.push(Candidate {
            assignments: chosen.clone(),
            layout,
        });
        return Ok(());
    }
    let (key, options) = factors[depth];
    for option in options {
        chosen.push(Assignment {
            key: (*key).clone(),
            carriers: option.carriers.clone(),
        });
        let step = expand_factors(factors, depth + 1, chosen, result, limits, report);
        chosen.pop();
        step?;
    }
    Ok(())
}

/// Check and retain every canonical local option without enumerating members.
///
/// # Errors
///
/// The existing census, local assignment or state-limit refusal. Malformed
/// generation and validation entries are refused rather than pruned.
#[allow(dead_code)] // The shared assembly selects this in packet 3.2e.
pub fn product(
    records: &Records,
    limits: PlacementSearchLimits,
    report: &mut PlacementSearchReport,
) -> Result<Product, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_placement", "product", obligation_count = crate::instrumentation::count(records.eligibility.len()), state_limit = limits.maximum_states.get());
    let _entered = span.enter();
    validate_census(records)?;
    admit_search_state(&mut report.states_visited, limits.maximum_states)
        .map_err(Error::StateLimit)?;
    let mut factors = BTreeMap::new();
    for row in &records.eligibility {
        admit_search_state(&mut report.states_visited, limits.maximum_states)
            .map_err(Error::StateLimit)?;
        let mut canonical = Vec::new();
        for option in options(row)? {
            admit_search_state(&mut report.states_visited, limits.maximum_states)
                .map_err(Error::StateLimit)?;
            let assignment = Assignment {
                key: row.key.clone(),
                carriers: option.carriers.clone(),
            };
            let depended = validate_assignment(records, &assignment, &option.layout)?;
            if depended != option.layout {
                return Err(Error::UnexpectedLayoutRequirement(
                    option.layout.difference(&depended).cloned().collect(),
                ));
            }
            canonical.push(FactorOption {
                carriers: option.carriers,
                layout: option.layout,
            });
        }
        canonical.sort();
        canonical.dedup();
        report.retained_options = report
            .retained_options
            .saturating_add(u64::try_from(canonical.len()).unwrap_or(u64::MAX));
        factors.insert(row.key.clone(), canonical);
    }
    tracing::debug!(target: "compiler::engine_placement", factor_count = crate::instrumentation::count(factors.len()), option_count = report.retained_options, visited_state_count = report.states_visited, complete_assignment_count = report.complete_assignments, materialized_candidate_count = report.feasible_placements, message = "placement product constructed");
    Ok(Product { factors })
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
