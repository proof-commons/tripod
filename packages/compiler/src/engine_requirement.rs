//! Relation-owned aggregate closure over the engine's concrete record domain.
use crate::{
    capability::RequiredCapability,
    engine_records::RelationKey,
    engine_source::{Evidence, SourceRow},
};
use std::collections::{BTreeMap, BTreeSet};

pub fn single_representation(
    operation: crate::engine_records::OperationKey,
    objects: &BTreeSet<crate::engine_lifecycle::ObjectKey>,
    selected: &BTreeMap<crate::engine_proof::ChoiceKey, crate::engine_lifecycle::ModeKey>,
) -> Option<(
    crate::engine_lifecycle::ObjectKey,
    crate::engine_lifecycle::ModeKey,
)> {
    let mut selections = objects.iter().filter_map(|object| {
        selected
            .get(&crate::engine_proof::ChoiceKey {
                operation,
                object: *object,
            })
            .map(|mode| (*object, *mode))
    });
    let selection = selections.next()?;
    if selections.next().is_some() {
        None
    } else {
        Some(selection)
    }
}
pub fn owned_lifecycle(
    association: Option<(
        crate::engine_lifecycle::ObjectKey,
        crate::engine_records::OperationKey,
    )>,
    candidate: &[crate::engine_lifecycle::LifecycleRequirementRecord],
) -> BTreeSet<crate::engine_lifecycle::LifecycleRequirementRecord> {
    let Some((object, exit)) = association else {
        return BTreeSet::new();
    };
    candidate
        .iter()
        .filter(|row| row.object == object && row.exit == exit)
        .cloned()
        .collect()
}

#[derive(Clone, Debug)]
pub struct Table {
    pub relation: RelationKey,
    pub required: bool,
    pub choices:
        BTreeMap<Option<crate::engine_proof::AlternativeKey>, Result<Bundle, crate::CompileError>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofBundleError {
    Constructibility(crate::engine_proof::AlternativeKey),
    Source {
        alternative: crate::engine_proof::AlternativeKey,
        reason: crate::proof::LocalProofRejection,
    },
}
pub fn proof_tables(
    records: &crate::engine_proof::Records,
) -> Result<Vec<Table>, ProofBundleError> {
    use crate::engine_proof::Obligation;
    records
        .obligations
        .iter()
        .map(|(relation, obligation)| {
            let (required, rows) = match obligation {
                Obligation::Static => (false, vec![(None, Ok(Bundle::default()))]),
                Obligation::Evidence(alternative) => {
                    (false, vec![(None, Ok(proof_bundle(alternative)?))])
                }
                Obligation::Required(alternatives) => (
                    true,
                    alternatives
                        .iter()
                        .map(|alternative| {
                            Ok((Some(alternative.id.clone()), Ok(proof_bundle(alternative)?)))
                        })
                        .collect::<Result<Vec<_>, ProofBundleError>>()?,
                ),
            };
            Ok(Table {
                relation: relation.clone(),
                required,
                choices: rows.into_iter().collect(),
            })
        })
        .collect()
}
fn proof_bundle(
    alternative: &crate::engine_proof::Alternative,
) -> Result<Bundle, ProofBundleError> {
    if !alternative.constructible {
        return Err(ProofBundleError::Constructibility(alternative.id.clone()));
    }
    Ok(Bundle {
        capabilities: alternative.capabilities.clone(),
        sources: alternative
            .sources
            .as_ref()
            .map_err(|reason| ProofBundleError::Source {
                alternative: alternative.id.clone(),
                reason: *reason,
            })?
            .iter()
            .cloned()
            .collect(),
        evidence: alternative.evidence.clone(),
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionError {
    Missing(RelationKey),
    Unexpected(RelationKey),
    Unapproved(RelationKey),
    Duplicate(RelationKey),
    OwnerRefusal(Box<crate::CompileError>),
}
pub fn derive(
    tables: &[Table],
    selected: &BTreeMap<RelationKey, crate::engine_proof::AlternativeKey>,
) -> Result<BTreeMap<RelationKey, Bundle>, SelectionError> {
    let span = tracing::debug_span!(target: "compiler::engine_requirement", "derive", table_count = crate::instrumentation::count(tables.len()), selected_count = crate::instrumentation::count(selected.len()));
    let _entered = span.enter();
    let mut result = BTreeMap::new();
    for table in tables {
        let choice = selected.get(&table.relation).cloned();
        if table.required && choice.is_none() {
            return Err(SelectionError::Missing(table.relation.clone()));
        }
        if !table.required && choice.is_some() {
            return Err(SelectionError::Unexpected(table.relation.clone()));
        }
        let bundle = table
            .choices
            .get(&choice)
            .ok_or_else(|| SelectionError::Unapproved(table.relation.clone()))?
            .clone()
            .map_err(|error| SelectionError::OwnerRefusal(Box::new(error)))?;
        if result.insert(table.relation.clone(), bundle).is_some() {
            return Err(SelectionError::Duplicate(table.relation.clone()));
        }
    }
    Ok(result)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bundle {
    pub capabilities: BTreeSet<RequiredCapability>,
    pub sources: BTreeSet<SourceRow>,
    pub evidence: BTreeSet<Evidence>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Capability {
        relation: Option<RelationKey>,
        capability: RequiredCapability,
    },
    Source {
        relation: Option<RelationKey>,
        row: Box<SourceRow>,
    },
    Evidence {
        relation: Option<RelationKey>,
        evidence: Evidence,
    },
}
pub fn validate_aggregate(
    aggregate: &Bundle,
    owned: &BTreeMap<RelationKey, Bundle>,
) -> Result<(), Error> {
    let span = tracing::debug_span!(target: "compiler::engine_requirement", "validate_aggregate", relation_count = crate::instrumentation::count(owned.len()), source_count = crate::instrumentation::count(aggregate.sources.len()), evidence_count = crate::instrumentation::count(aggregate.evidence.len()));
    let _entered = span.enter();
    let mut union = Bundle::default();
    for bundle in owned.values() {
        union.capabilities.extend(&bundle.capabilities);
        union.sources.extend(bundle.sources.iter().cloned());
        union.evidence.extend(bundle.evidence.iter().cloned());
    }
    if let Some(capability) = union
        .capabilities
        .difference(&aggregate.capabilities)
        .next()
    {
        return Err(Error::Capability {
            relation: owned
                .iter()
                .find(|(_, bundle)| bundle.capabilities.contains(capability))
                .map(|(relation, _)| relation.clone()),
            capability: *capability,
        });
    }
    if let Some(capability) = aggregate
        .capabilities
        .difference(&union.capabilities)
        .next()
    {
        return Err(Error::Capability {
            relation: None,
            capability: *capability,
        });
    }
    if let Some(row) = union.sources.difference(&aggregate.sources).next() {
        return Err(Error::Source {
            relation: Some(row.operand.relation.clone()),
            row: Box::new(row.clone()),
        });
    }
    if let Some(row) = aggregate.sources.difference(&union.sources).next() {
        return Err(Error::Source {
            relation: None,
            row: Box::new(row.clone()),
        });
    }
    if let Some(evidence) = union.evidence.difference(&aggregate.evidence).next() {
        return Err(Error::Evidence {
            relation: owned
                .iter()
                .find(|(_, bundle)| bundle.evidence.contains(evidence))
                .map(|(relation, _)| relation.clone()),
            evidence: evidence.clone(),
        });
    }
    if let Some(evidence) = aggregate.evidence.difference(&union.evidence).next() {
        return Err(Error::Evidence {
            relation: None,
            evidence: evidence.clone(),
        });
    }
    Ok(())
}
