//! Finite attestation tables prepared before a proof candidate is selected.
use crate::{CompileError, engine_cases as core, proof::RelationObligationClass};
use std::collections::BTreeMap;

pub fn lower(input: &crate::BoundCompilerInput) -> Result<core::Tables, CompileError> {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_cases", scope_count = crate::instrumentation::count(input.scope().operations().len()));
    let _entered = span.enter();
    let relations = crate::relation::build_relation_analysis(input)?;
    let lifecycle = crate::lifecycle::build_lifecycle_analysis(input, &relations)?;
    let obligations = super::proof::classify_obligations(&relations)?;
    let mut tables = core::Tables {
        choices: lifecycle
            .choices
            .iter()
            .map(|choice| {
                (
                    super::proof::choice_key(&choice.id),
                    choice
                        .candidates
                        .iter()
                        .copied()
                        .map(super::proof::mode_key)
                        .collect(),
                )
            })
            .collect(),
        relations: relations
            .graph
            .node_weights()
            .map(|row| super::relation_key(&row.source.id))
            .collect(),
        cases: Vec::new(),
        rows: Vec::new(),
    };
    for operation in input.scope().operations() {
        let mut modes = vec![BTreeMap::new()];
        for choice in lifecycle
            .choices
            .iter()
            .filter(|choice| choice.id.operation == *operation)
        {
            modes = modes
                .into_iter()
                .flat_map(|selected| {
                    choice.candidates.iter().map(move |mode| {
                        let mut next = selected.clone();
                        next.insert(choice.id.object, *mode);
                        next
                    })
                })
                .collect();
        }
        for representations in modes {
            for sponsor in crate::case::sponsor_cases(&relations, *operation) {
                let case = crate::case::ExecutionCaseId {
                    operation: *operation,
                    sponsor,
                    representations: representations.clone(),
                };
                tables.cases.push(core::Case {
                    id: crate::engine_placement::CaseKey::Attestation(case.clone()),
                    operation: super::operation_key(*operation),
                    representations: representations
                        .iter()
                        .map(|(object, mode)| {
                            (
                                super::proof::choice_key(
                                    &crate::lifecycle::RepresentationChoiceId {
                                        operation: *operation,
                                        object: *object,
                                    },
                                ),
                                super::proof::mode_key(*mode),
                            )
                        })
                        .collect(),
                });
                for node in relations
                    .graph
                    .node_weights()
                    .filter(|node| node.source.id.operation() == *operation)
                {
                    let obligation = obligations
                        .iter()
                        .find(|row| row.relation == node.source.id)
                        .ok_or(CompileError::InvalidRealization)?;
                    let alternatives = alternatives(obligation);
                    for (proof, selector) in alternatives {
                        tables
                            .rows
                            .push(row(&relations, &node.source, &case, proof, selector));
                    }
                }
            }
        }
    }
    Ok(tables)
}
fn row(
    relations: &crate::relation::CompilerRelationAnalysis,
    declaration: &realization::RelationDeclaration,
    case: &crate::case::ExecutionCaseId,
    proof: Option<&realization::ProofAlternativeId>,
    selector: Option<crate::engine_proof::AlternativeKey>,
) -> core::Row {
    let key = super::placement::key(&crate::placement::RelationCaseKey {
        relation: declaration.id.clone(),
        case: case.clone(),
    });
    let result = derive(relations, declaration, case, proof);
    match result {
        Ok((runtime, eligibility, coverage, facts)) => core::Row {
            key,
            proof: selector,
            runtime,
            eligibility,
            coverage: Ok(coverage),
            refusal: None,
            facts: Some(facts),
        },
        Err(error) => core::Row {
            key,
            proof: selector,
            runtime: false,
            eligibility: Vec::new(),
            coverage: Err(error.clone()),
            refusal: Some(error),
            facts: None,
        },
    }
}
fn derive(
    relations: &crate::relation::CompilerRelationAnalysis,
    declaration: &realization::RelationDeclaration,
    case: &crate::case::ExecutionCaseId,
    proof: Option<&realization::ProofAlternativeId>,
) -> Result<
    (
        bool,
        Vec<crate::engine_placement::Eligibility>,
        crate::engine_coverage::Plan,
        crate::engine_case_facts::Facts,
    ),
    CompileError,
> {
    let sources = proof.map_or_else(
        || Ok(Vec::new()),
        |proof| crate::source::derive_source_requirements(declaration, proof.proof()),
    )?;
    let execution = crate::case::ExecutionCase {
        id: case.clone(),
        active_sources: sources
            .into_iter()
            .filter(|row| crate::case::is_active(case, row))
            .collect(),
    };
    let role = crate::sponsor_region::ordinary_lbtc_role(relations, case.operation);
    let plan = crate::placement::classify_relation_case(declaration, &execution, role)?;
    let eligible =
        crate::carrier::relation_case_eligibility(relations, std::slice::from_ref(&plan))?;
    let lowered = super::placement::lower(std::slice::from_ref(&plan), &eligible, None);
    let coverage = super::coverage::derive_relation_coverage(declaration, &plan)?;
    let layout =
        crate::layout::layout_requirements(relations, std::slice::from_ref(&plan), &eligible)?;
    let facts = super::case_facts::lower(
        &plan,
        &execution.active_sources.iter().cloned().collect(),
        &layout,
    );
    Ok((
        !plan.runtime_requirements.is_empty(),
        lowered.records.eligibility,
        super::coverage::lowered(&coverage),
        facts,
    ))
}

fn alternatives(
    obligation: &crate::proof::RelationObligation,
) -> Vec<(
    Option<&realization::ProofAlternativeId>,
    Option<crate::engine_proof::AlternativeKey>,
)> {
    match &obligation.class {
        RelationObligationClass::StaticallyValidated => vec![(None, None)],
        RelationObligationClass::ExternalEvidence { proof, .. } => {
            vec![(Some(proof), None)]
        }
        RelationObligationClass::ProofRequired { alternatives } => alternatives
            .iter()
            .map(|proof| {
                (
                    Some(proof),
                    Some(crate::engine_proof::AlternativeKey {
                        relation: super::relation_key(&obligation.relation),
                        namespace: super::ATTESTATION,
                        kind: proof.proof() as u16,
                    }),
                )
            })
            .collect(),
    }
}
