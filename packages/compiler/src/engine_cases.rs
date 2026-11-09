//! Selection from finite owner-produced case tables; no owner callback.
use crate::{
    engine_coverage,
    engine_lifecycle::ModeKey,
    engine_placement::{self, CaseKey, Key},
    engine_proof::{AlternativeKey, Candidate, ChoiceKey},
    engine_records::{OperationKey, RelationKey},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Case {
    pub id: CaseKey,
    pub operation: OperationKey,
    pub representations: BTreeMap<ChoiceKey, ModeKey>,
}
#[derive(Clone, Debug)]
pub struct Row {
    pub key: Key,
    pub proof: Option<AlternativeKey>,
    pub runtime: bool,
    pub eligibility: Vec<engine_placement::Eligibility>,
    pub coverage: Result<engine_coverage::Plan, crate::CompileError>,
    pub refusal: Option<crate::CompileError>,
    pub facts: Option<crate::engine_case_facts::Facts>,
}
#[derive(Clone, Debug)]
pub struct Tables {
    pub choices: Vec<(ChoiceKey, BTreeSet<ModeKey>)>,
    pub relations: BTreeSet<RelationKey>,
    pub cases: Vec<Case>,
    pub rows: Vec<Row>,
}
#[derive(Clone, Debug)]
pub struct Selected {
    pub placement: engine_placement::Records,
    pub coverage: Vec<engine_coverage::Plan>,
    pub facts: Vec<crate::engine_case_facts::Facts>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    MissingRepresentationChoice(ChoiceKey),
    UnsupportedRepresentation {
        choice: ChoiceKey,
        mode: ModeKey,
    },
    CaseCensusMismatch {
        missing: Vec<Key>,
        unexpected: Vec<Key>,
    },
    DuplicateCase(Key),
    OwnerRefusal(Box<crate::CompileError>),
    CaseFactsMismatch(Key),
}
pub fn select(tables: &Tables, candidate: &Candidate) -> Result<Selected, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_cases", "select", case_count = crate::instrumentation::count(tables.cases.len()), row_count = crate::instrumentation::count(tables.rows.len()));
    let _entered = span.enter();
    for (choice, allowed) in &tables.choices {
        let mode = candidate
            .representations
            .get(choice)
            .ok_or(Error::MissingRepresentationChoice(*choice))?;
        if !allowed.contains(mode) {
            return Err(Error::UnsupportedRepresentation {
                choice: *choice,
                mode: *mode,
            });
        }
    }
    let cases = tables
        .cases
        .iter()
        .filter(|case| {
            case.representations
                .iter()
                .all(|(choice, mode)| candidate.representations.get(choice) == Some(mode))
        })
        .collect::<Vec<_>>();
    let expected = cases
        .iter()
        .flat_map(|case| {
            tables
                .relations
                .iter()
                .filter(|relation| relation.operation == case.operation)
                .map(|relation| Key {
                    relation: relation.clone(),
                    case: case.id.clone(),
                })
        })
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut plans = Vec::new();
    let mut eligibility = Vec::new();
    let mut coverage = Vec::new();
    let mut facts = Vec::new();
    for row in &tables.rows {
        if !cases.iter().any(|case| case.id == row.key.case)
            || row
                .proof
                .as_ref()
                .is_some_and(|proof| candidate.proofs.get(&row.key.relation) != Some(proof))
        {
            continue;
        }
        if !seen.insert(row.key.clone()) {
            return Err(Error::DuplicateCase(row.key.clone()));
        }
        if let Some(error) = &row.refusal {
            return Err(Error::OwnerRefusal(Box::new(error.clone())));
        }
        let fact = row
            .facts
            .as_ref()
            .ok_or_else(|| Error::CaseFactsMismatch(row.key.clone()))?;
        let covered = row
            .coverage
            .as_ref()
            .map_err(|error| Error::OwnerRefusal(Box::new(error.clone())))?;
        if fact.key != row.key
            || fact.case_operation != row.key.relation.operation
            || fact.active != covered.active
            || fact.boundaries != covered.boundaries
            || fact.evidence != covered.evidence
            || row.runtime == fact.runtime.is_empty()
        {
            return Err(Error::CaseFactsMismatch(row.key.clone()));
        }
        facts.push(fact.clone());
        plans.push((row.key.clone(), row.runtime));
        eligibility.extend(row.eligibility.clone());
        coverage.push(
            row.coverage
                .clone()
                .map_err(|error| Error::OwnerRefusal(Box::new(error)))?,
        );
    }
    if seen != expected {
        return Err(Error::CaseCensusMismatch {
            missing: expected.difference(&seen).cloned().collect(),
            unexpected: seen.difference(&expected).cloned().collect(),
        });
    }
    Ok(Selected {
        placement: engine_placement::Records { plans, eligibility },
        coverage,
        facts,
    })
}
