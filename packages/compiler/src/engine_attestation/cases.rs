//! Finite attestation tables prepared before a proof candidate is selected.
use crate::{CompileError, engine_cases as core, proof::RelationObligationClass};
use std::collections::BTreeMap;

#[cfg(test)]
pub fn lower(input: &crate::BoundCompilerInput) -> Result<core::Tables, CompileError> {
    Ok(lower_registered(input)?.tables)
}
pub struct Registered {
    pub tables: core::Tables,
    pub payloads: super::operation::Rows,
    pub layouts: super::operation::LayoutRegistry,
}
#[cfg(test)]
pub fn lower_registered(input: &crate::BoundCompilerInput) -> Result<Registered, CompileError> {
    let relations = crate::relation::build_relation_analysis(input)?;
    let lifecycle = crate::lifecycle::build_lifecycle_analysis(input, &relations)?;
    lower_prepared(input, &relations, &lifecycle)
}

pub fn lower_prepared(
    input: &crate::BoundCompilerInput,
    relations: &crate::relation::CompilerRelationAnalysis,
    lifecycle: &crate::lifecycle::CompilerLifecycleAnalysis,
) -> Result<Registered, CompileError> {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_cases", scope_count = crate::instrumentation::count(input.scope().operations().len()));
    let _entered = span.enter();
    let obligations = super::proof::classify_obligations(relations)?;
    let mut layouts = super::operation::LayoutRegistry::new();
    let mut payloads = super::operation::Rows::new();
    let mut tables = empty_tables(relations, lifecycle);
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
            for sponsor in crate::case::sponsor_cases(relations, *operation) {
                let case = crate::case::ExecutionCaseId {
                    operation: *operation,
                    sponsor,
                    representations: representations.clone(),
                };
                tables.cases.push(core::Case {
                    id: crate::engine_attestation::identity::attestation_case(&case),
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
                        let (row, payload) = row(
                            relations,
                            &node.source,
                            &case,
                            proof,
                            selector,
                            &mut layouts,
                        );
                        if let Some(payload) = payload {
                            payloads.insert((row.key.clone(), row.proof.clone()), payload);
                        }
                        tables.rows.push(row);
                    }
                }
            }
        }
    }
    tables.layout_scopes = layout_scopes(&layouts);
    Ok(Registered {
        tables,
        payloads,
        layouts,
    })
}
fn empty_tables(
    relations: &crate::relation::CompilerRelationAnalysis,
    lifecycle: &crate::lifecycle::CompilerLifecycleAnalysis,
) -> core::Tables {
    core::Tables {
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
        layout_scopes: BTreeMap::new(),
    }
}

fn layout_scopes(
    layouts: &super::operation::LayoutRegistry,
) -> BTreeMap<crate::engine_placement::LayoutKey, core::LayoutScope> {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_layout_scopes", row_count = crate::instrumentation::count(layouts.len()));
    let _entered = span.enter();
    layouts
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                core::LayoutScope {
                    relation: value.relation().map(super::relation_key),
                    case: crate::analyzed_operation::layout_case(value)
                        .map(crate::engine_attestation::identity::attestation_case),
                },
            )
        })
        .collect()
}

fn row(
    relations: &crate::relation::CompilerRelationAnalysis,
    declaration: &realization::RelationDeclaration,
    case: &crate::case::ExecutionCaseId,
    proof: Option<&realization::ProofAlternativeId>,
    selector: Option<crate::engine_proof::AlternativeKey>,
    layouts: &mut super::operation::LayoutRegistry,
) -> (core::Row, Option<super::operation::Payload>) {
    let key = super::placement::key(&crate::placement::RelationCaseKey {
        relation: declaration.id.clone(),
        case: case.clone(),
    });
    let result = derive(relations, declaration, case, proof, layouts);
    match result {
        Ok(derived) => (
            core::Row {
                key,
                proof: selector,
                runtime: derived.runtime,
                eligibility: derived.eligibility,
                coverage: Ok(derived.coverage),
                refusal: None,
                facts: Some(derived.facts),
                projections: derived.projections,
                collateral: derived.collateral,
            },
            Some(derived.payload),
        ),
        Err(error) => (
            core::Row {
                key,
                proof: selector,
                runtime: false,
                eligibility: Vec::new(),
                coverage: Err(error.clone()),
                refusal: Some(error),
                facts: None,
                projections: std::collections::BTreeSet::new(),
                collateral: std::collections::BTreeSet::new(),
            },
            None,
        ),
    }
}
struct Derived {
    payload: super::operation::Payload,
    runtime: bool,
    eligibility: Vec<crate::engine_placement::Eligibility>,
    coverage: crate::engine_coverage::Plan,
    facts: crate::engine_case_facts::Facts,
    projections: std::collections::BTreeSet<crate::coverage::CoverageBoundary>,
    collateral: std::collections::BTreeSet<crate::engine_coverage::Requirement>,
}
fn derive(
    relations: &crate::relation::CompilerRelationAnalysis,
    declaration: &realization::RelationDeclaration,
    case: &crate::case::ExecutionCaseId,
    proof: Option<&realization::ProofAlternativeId>,
    layouts: &mut super::operation::LayoutRegistry,
) -> Result<Derived, CompileError> {
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
    let lowered = super::placement::eligibility_records(std::slice::from_ref(&plan), &eligible);
    let coverage = super::coverage::derive_relation_coverage(declaration, &plan)?;
    let layout =
        crate::layout::layout_requirements(relations, std::slice::from_ref(&plan), &eligible)?;
    let facts = super::case_facts::lower(
        &plan,
        &execution.active_sources.iter().cloned().collect(),
        &layout,
    );
    let projections = coverage
        .boundaries
        .iter()
        .filter(|boundary| {
            crate::coverage::projection_requirement(&coverage.key(), &coverage, &plan, **boundary)
                .is_some()
        })
        .copied()
        .collect();
    let collateral = coverage
        .negative
        .iter()
        .filter(|row| {
            row.collateral.policy
                == crate::coverage::CollateralPolicy::RequireIntendedAndDependencyClosure
        })
        .map(|row| super::coverage::requirement(&row.id))
        .collect();
    layouts.extend(
        layout
            .iter()
            .map(|value| (super::identity::attestation_layout(value), value.clone())),
    );
    Ok(Derived {
        payload: super::operation::Payload::new(
            &plan,
            &coverage,
            &execution.active_sources,
            &eligible,
        ),
        runtime: !plan.runtime_requirements.is_empty(),
        eligibility: lowered,
        coverage: super::coverage::lowered(&coverage),
        facts,
        projections,
        collateral,
    })
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
