//! Attestation authorization tables and lossless constructibility view.
use crate::{
    CompileError,
    constructibility::{
        CompilerConstructibilityAnalysis, ConstructibilityCaseAnalysis,
        OperationConstructibilityAnalysis,
    },
    engine_constructibility as core,
    engine_records::OperationKey,
};
use architecture::OperationId;
use realization::{
    AvailabilityClass, ConstructibilityAuthorization, ConstructibilityEdge,
    ConstructibilityEdgeRole, ConstructibilityNode, ConstructibilityNodeId, FactId,
    RequirementStrength, WitnessRole,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct Lowered {
    pub records: core::Records,
    nodes: BTreeMap<core::NodeKey, ConstructibilityNodeId>,
    operations: BTreeMap<OperationKey, OperationId>,
    availability: BTreeMap<core::Availability, AvailabilityClass>,
    authorizations: BTreeMap<Vec<u16>, ConstructibilityAuthorization>,
    edges: BTreeMap<core::Edge, ConstructibilityEdge>,
}

pub fn lower(input: &super::AttestationOwner) -> Result<Lowered, CompileError> {
    let projection = input.source().project().constructibility;
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_constructibility", node_count = crate::instrumentation::count(projection.nodes.len()));
    let _entered = span.enter();
    let nodes = projection
        .nodes
        .iter()
        .map(|node| &node.id)
        .chain(
            projection
                .edges
                .iter()
                .flat_map(|edge| [&edge.source, &edge.target]),
        )
        .map(|id| (node_key(id), id.clone()))
        .collect::<BTreeMap<_, _>>();
    let availability = nodes
        .values()
        .filter_map(node_availability)
        .map(|value| (availability_key(value), value))
        .collect::<BTreeMap<_, _>>();
    let mut authorizations = BTreeMap::new();
    let mut operations = Vec::new();
    for operation in input.scope().operations() {
        let cases = input
            .source()
            .constructibility_authorizations(*operation)
            .map_err(|_| CompileError::InvalidRealization)?;
        let lowered = cases
            .iter()
            .map(|case| {
                let row = authorization(case, availability.values().copied());
                authorizations.insert(row.key.clone(), case.clone());
                row
            })
            .collect();
        operations.push((super::operation_key(*operation), lowered));
    }
    Ok(Lowered {
        records: core::Records {
            scope: input
                .scope()
                .operations()
                .iter()
                .copied()
                .map(super::operation_key)
                .collect(),
            nodes: projection
                .nodes
                .iter()
                .map(|node| core::Node {
                    id: node_key(&node.id),
                })
                .collect(),
            edges: projection
                .edges
                .iter()
                .map(|edge| core::Dependency {
                    source: node_key(&edge.source),
                    target: node_key(&edge.target),
                    edge: edge_key(edge.edge),
                })
                .collect(),
            operations,
        },
        nodes,
        operations: OperationId::ALL
            .iter()
            .copied()
            .map(|id| (super::operation_key(id), id))
            .collect(),
        availability,
        authorizations,
        edges: projection
            .edges
            .iter()
            .map(|edge| (edge_key(edge.edge), edge.edge))
            .collect(),
    })
}

impl Lowered {
    #[cfg(test)]
    pub fn analyze(&self) -> Result<CompilerConstructibilityAnalysis, CompileError> {
        self.compatibility_view(core::analyze_constructibility(&self.records))
    }

    pub fn compatibility_view(
        &self,
        result: Result<core::Analysis, core::RecordError>,
    ) -> Result<CompilerConstructibilityAnalysis, CompileError> {
        let value = result.map_err(|error| self.refusal(error))?;
        Ok(CompilerConstructibilityAnalysis {
            graph: value.graph.map(
                |_, node| ConstructibilityNode {
                    id: self.nodes[&node.id].clone(),
                },
                |_, edge| self.edges[edge],
            ),
            node_by_id: value
                .node_by_id
                .into_iter()
                .map(|(key, index)| (self.nodes[&key].clone(), index))
                .collect(),
            operations: value
                .operations
                .into_iter()
                .map(|operation| OperationConstructibilityAnalysis {
                    operation: self.operations[&operation.operation],
                    cases: operation
                        .cases
                        .into_iter()
                        .map(|case| ConstructibilityCaseAnalysis {
                            authorization: self.authorizations[&case.authorization.key].clone(),
                            required_nodes: case
                                .required_nodes
                                .iter()
                                .map(|node| self.nodes[node].clone())
                                .collect(),
                            optional_nodes: case
                                .optional_nodes
                                .iter()
                                .map(|node| self.nodes[node].clone())
                                .collect(),
                            required_availability: case
                                .required_availability
                                .iter()
                                .map(|key| self.availability[key])
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
        })
    }

    pub(super) fn refusal(&self, error: core::RecordError) -> CompileError {
        let node = |key: &core::NodeKey| self.nodes[key].clone();
        let path = |keys: Vec<core::NodeKey>| keys.iter().map(node).collect();
        match error {
            core::RecordError::DuplicateConstructibilityNode { node: key } => {
                CompileError::DuplicateConstructibilityNode { node: node(&key) }
            }
            core::RecordError::DuplicateConstructibilityDependency { node: key } => {
                CompileError::DuplicateConstructibilityDependency { node: node(&key) }
            }
            core::RecordError::UnknownConstructibilityNode { node: key } => {
                CompileError::UnknownConstructibilityNode { node: node(&key) }
            }
            core::RecordError::ConstructibilityCycle { components } => {
                CompileError::ConstructibilityCycle {
                    components: components.into_iter().map(path).collect(),
                }
            }
            core::RecordError::ConstructibilityCrossOperationDependency {
                operation,
                node: key,
            } => CompileError::ConstructibilityCrossOperationDependency {
                operation: self.operations[&operation],
                node: node(&key),
            },
            core::RecordError::PermissionlessPrivateDependency {
                operation,
                node: key,
                path: keys,
            } => CompileError::PermissionlessPrivateDependency {
                operation: self.operations[&operation],
                node: node(&key),
                path: path(keys),
            },
            core::RecordError::ConstructibilityWitnessUnavailable {
                operation,
                authorization,
                node: key,
                path: keys,
            } => CompileError::ConstructibilityWitnessUnavailable {
                operation: self.operations[&operation],
                authorization: self.authorizations[&authorization.key].clone(),
                node: node(&key),
                path: path(keys),
            },
            core::RecordError::SponsorDependencyEscaped {
                operation,
                node: key,
                path: keys,
            } => CompileError::SponsorDependencyEscaped {
                operation: self.operations[&operation],
                node: node(&key),
                path: path(keys),
            },
        }
    }
}

pub fn dependency_allowed(
    case: &ConstructibilityAuthorization,
    node: &ConstructibilityNodeId,
) -> bool {
    let lowered = node_key(node);
    let case = authorization(case, node_availability(node));
    core::validate_required_dependency(lowered.operation, &case, &lowered, Vec::new).is_ok()
}

fn authorization(
    case: &ConstructibilityAuthorization,
    values: impl IntoIterator<Item = AvailabilityClass>,
) -> core::Authorization {
    use ConstructibilityAuthorization as A;
    let key = match case {
        A::Permissionless => vec![0],
        A::InputOwners { objects } => std::iter::once(1)
            .chain(objects.iter().map(|object| object.code()))
            .collect(),
        A::RefundKey { object } => vec![2, object.code()],
        A::ClientAuthorized { objects } => std::iter::once(3)
            .chain(objects.iter().map(|object| object.code()))
            .collect(),
        A::Operator => vec![4],
        A::CadenceOperator => vec![5],
        A::CadencePermissionless => vec![6],
    };
    core::Authorization {
        key,
        permissionless: matches!(case, A::Permissionless | A::CadencePermissionless),
        allowed: values
            .into_iter()
            .filter(|value| case.discharges(*value))
            .map(availability_key)
            .collect::<BTreeSet<_>>(),
    }
}

pub const fn availability_key(value: AvailabilityClass) -> core::Availability {
    let (kind, object) = match value {
        AvailabilityClass::Public => (0, 0),
        AvailabilityClass::InputOwners { object } => (1, object.code()),
        AvailabilityClass::Operator => (2, 0),
        AvailabilityClass::SponsorLocal => (3, 0),
        AvailabilityClass::RefundKey { object } => (4, object.code()),
        AvailabilityClass::ClientOwners { object } => (5, object.code()),
    };
    core::Availability {
        namespace: super::ATTESTATION,
        kind,
        object,
        sponsor_local: matches!(value, AvailabilityClass::SponsorLocal),
    }
}
const fn node_availability(node: &ConstructibilityNodeId) -> Option<AvailabilityClass> {
    match node {
        ConstructibilityNodeId::Operation(_) => None,
        ConstructibilityNodeId::Fact { availability, .. }
        | ConstructibilityNodeId::Witness { availability, .. } => Some(*availability),
    }
}
pub(super) fn node_key(node: &ConstructibilityNodeId) -> core::NodeKey {
    match node {
        ConstructibilityNodeId::Operation(operation) => {
            core::NodeKey::operation(super::operation_key(*operation))
        }
        ConstructibilityNodeId::Fact {
            operation,
            fact,
            availability,
        } => core::NodeKey {
            kind: 1,
            operation: super::operation_key(*operation),
            subject: fact_key(fact),
            availability: Some(availability_key(*availability)),
            independent: matches!(fact, FactId::BoundValue { .. }),
        },
        ConstructibilityNodeId::Witness {
            operation,
            role,
            availability,
        } => core::NodeKey {
            kind: 2,
            operation: super::operation_key(*operation),
            subject: vec![match role {
                WitnessRole::ProtocolOwnerAuthorization => 0,
                WitnessRole::OperatorAuthorization => 1,
                WitnessRole::SponsorAuthorization => 2,
            }],
            availability: Some(availability_key(*availability)),
            independent: false,
        },
    }
}
const fn edge_key(edge: ConstructibilityEdge) -> core::Edge {
    core::Edge {
        kind: match edge.role {
            ConstructibilityEdgeRole::RequiredFact => 0,
            ConstructibilityEdgeRole::RequiredWitness => 1,
            ConstructibilityEdgeRole::SponsorOnly => 2,
        },
        required: matches!(edge.strength, RequirementStrength::Required),
        sponsor_only: matches!(edge.role, ConstructibilityEdgeRole::SponsorOnly),
    }
}
pub fn fact_key(fact: &FactId) -> Vec<u16> {
    match fact {
        FactId::FamilyCount {
            operation,
            side,
            object,
        } => vec![0, operation.code(), super::side_code(*side), object.code()],
        FactId::FamilyAmount {
            operation,
            side,
            object,
        } => vec![1, operation.code(), super::side_code(*side), object.code()],
        FactId::StateField {
            operation,
            side,
            field,
        } => vec![2, operation.code(), super::side_code(*side), *field as u16],
        FactId::RequestedAnnouncementCycle { operation } => vec![3, operation.code()],
        FactId::AnnouncementLead { operation, bound } => vec![4, operation.code(), *bound as u16],
        FactId::InputOwners { operation, object } => vec![5, operation.code(), object.code()],
        FactId::Signers { operation } => vec![6, operation.code()],
        FactId::ProjectionPresent {
            operation,
            projection,
        } => vec![7, operation.code(), projection.code()],
        FactId::BoundValue { bound } => vec![8, bound.code()],
        FactId::FamilyRecognized {
            operation,
            side,
            object,
        } => vec![9, operation.code(), super::side_code(*side), object.code()],
        FactId::SponsorIsolated { operation } => vec![10, operation.code()],
        FactId::ProtocolSecretUsed { operation } => vec![11, operation.code()],
    }
}
