use architecture::{ObjectId, OperationId};

use crate::{
    AvailabilityClass, ConstructibilityAuthorization, ConstructibilityDependencyDeclaration,
    ConstructibilityEdge, ConstructibilityEdgeRole, ConstructibilityNode, ConstructibilityNodeId,
    FactId, RealizationError, RequirementStrength, TransactionSide, WitnessRole,
    build_constructibility_graph, constructibility_authorizations, derive,
    validate_constructibility,
};

fn permissionless() -> ConstructibilityAuthorization {
    ConstructibilityAuthorization::Permissionless
}

fn input_owners(object: ObjectId) -> ConstructibilityAuthorization {
    ConstructibilityAuthorization::InputOwners {
        objects: std::iter::once(object).collect(),
    }
}

#[test]
fn phase1_constructibility_graph_is_direct_petgraph_and_validates() {
    let realization = derive(
        &architecture::ARCHITECTURE,
        crate::RealizationScope::phase1_pilots(),
    )
    .unwrap();
    let graph: &petgraph::graph::DiGraph<ConstructibilityNode, ConstructibilityEdge, u32> =
        &realization.constructibility_graph;

    assert!(graph.node_count() > 0);
    validate_constructibility(
        graph,
        &realization.constructibility_node_by_id,
        OperationId::CompactAsh,
        &permissionless(),
    )
    .unwrap();
    validate_constructibility(
        graph,
        &realization.constructibility_node_by_id,
        OperationId::TransferLive,
        &input_owners(ObjectId::ReceiptLive),
    )
    .unwrap();
}

#[test]
fn permissionless_private_dependency_fails() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let owner_witness = ConstructibilityNodeId::Witness {
        operation: OperationId::CompactAsh,
        role: WitnessRole::ProtocolOwnerAuthorization,
        availability: AvailabilityClass::InputOwners {
            object: ObjectId::Ash,
        },
    };
    let (graph, nodes, _) = build_constructibility_graph(
        [
            ConstructibilityNode {
                id: operation.clone(),
            },
            ConstructibilityNode {
                id: owner_witness.clone(),
            },
        ],
        [ConstructibilityDependencyDeclaration {
            source: owner_witness.clone(),
            target: operation,
            edge: ConstructibilityEdge {
                role: ConstructibilityEdgeRole::RequiredWitness,
                strength: RequirementStrength::Required,
            },
        }],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()),
        Err(RealizationError::PermissionlessPrivateDependency {
            operation: OperationId::CompactAsh,
            source_node: owner_witness.clone(),
            path: vec![
                owner_witness,
                ConstructibilityNodeId::Operation(OperationId::CompactAsh)
            ],
        }),
    );
}

#[test]
fn sponsor_local_dependency_must_stay_on_sponsor_edge() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let sponsor = ConstructibilityNodeId::Witness {
        operation: OperationId::CompactAsh,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    let (graph, nodes, _) = build_constructibility_graph(
        [
            ConstructibilityNode {
                id: operation.clone(),
            },
            ConstructibilityNode {
                id: sponsor.clone(),
            },
        ],
        [ConstructibilityDependencyDeclaration {
            source: sponsor.clone(),
            target: operation,
            edge: ConstructibilityEdge {
                role: ConstructibilityEdgeRole::RequiredWitness,
                strength: RequirementStrength::Required,
            },
        }],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()),
        Err(RealizationError::SponsorDependencyEscaped {
            operation: OperationId::CompactAsh,
            source_node: sponsor.clone(),
            path: vec![
                sponsor,
                ConstructibilityNodeId::Operation(OperationId::CompactAsh)
            ],
        }),
    );
}

#[test]
fn duplicate_constructibility_dependency_is_rejected() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let public_fact = public_count_fact(OperationId::CompactAsh);
    let dependency = edge(
        public_fact.clone(),
        operation.clone(),
        ConstructibilityEdgeRole::RequiredFact,
    );

    let error = build_constructibility_graph(
        [node(operation), node(public_fact)],
        [dependency.clone(), dependency.clone()],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::DuplicateConstructibilityDependency(dependency),
    );
}

#[test]
fn permissionless_private_dependency_cannot_be_laundered_through_public_fact() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let owner_witness = owner_witness(OperationId::CompactAsh);
    let public_fact = public_count_fact(OperationId::CompactAsh);
    let (graph, nodes, _) = build_constructibility_graph(
        [
            node(operation.clone()),
            node(public_fact.clone()),
            node(owner_witness.clone()),
        ],
        [
            edge(
                owner_witness.clone(),
                public_fact.clone(),
                ConstructibilityEdgeRole::RequiredWitness,
            ),
            edge(
                public_fact.clone(),
                operation.clone(),
                ConstructibilityEdgeRole::RequiredFact,
            ),
        ],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()),
        Err(RealizationError::PermissionlessPrivateDependency {
            operation: OperationId::CompactAsh,
            source_node: owner_witness.clone(),
            path: vec![owner_witness, public_fact, operation],
        }),
    );
}

#[test]
fn operator_dependency_cannot_be_laundered_through_two_facts() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let operator = ConstructibilityNodeId::Witness {
        operation: OperationId::CompactAsh,
        role: WitnessRole::OperatorAuthorization,
        availability: AvailabilityClass::Operator,
    };
    let first_fact = public_count_fact(OperationId::CompactAsh);
    let second_fact = public_amount_fact(OperationId::CompactAsh);
    let (graph, nodes, _) = build_constructibility_graph(
        [
            node(operation.clone()),
            node(operator.clone()),
            node(first_fact.clone()),
            node(second_fact.clone()),
        ],
        [
            edge(
                operator.clone(),
                first_fact.clone(),
                ConstructibilityEdgeRole::RequiredWitness,
            ),
            edge(
                first_fact.clone(),
                second_fact.clone(),
                ConstructibilityEdgeRole::RequiredFact,
            ),
            edge(
                second_fact.clone(),
                operation.clone(),
                ConstructibilityEdgeRole::RequiredFact,
            ),
        ],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()),
        Err(RealizationError::PermissionlessPrivateDependency {
            operation: OperationId::CompactAsh,
            source_node: operator.clone(),
            path: vec![operator, first_fact, second_fact, operation],
        }),
    );
}

#[test]
fn sponsor_local_dependency_cannot_escape_through_required_fact_edge() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let sponsor = sponsor_witness(OperationId::CompactAsh);
    let sponsor_fact = sponsor_fact(OperationId::CompactAsh);
    let (graph, nodes, _) = build_constructibility_graph(
        [
            node(operation.clone()),
            node(sponsor.clone()),
            node(sponsor_fact.clone()),
        ],
        [
            edge(
                sponsor,
                sponsor_fact.clone(),
                ConstructibilityEdgeRole::SponsorOnly,
            ),
            edge(
                sponsor_fact.clone(),
                operation.clone(),
                ConstructibilityEdgeRole::RequiredFact,
            ),
        ],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()),
        Err(RealizationError::SponsorDependencyEscaped {
            operation: OperationId::CompactAsh,
            source_node: sponsor_fact.clone(),
            path: vec![sponsor_fact, operation],
        }),
    );
}

#[test]
fn sponsor_local_chain_with_only_sponsor_edges_is_allowed() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let sponsor = sponsor_witness(OperationId::CompactAsh);
    let sponsor_fact = sponsor_fact(OperationId::CompactAsh);
    let (graph, nodes, _) = build_constructibility_graph(
        [
            node(operation.clone()),
            node(sponsor),
            node(sponsor_fact.clone()),
        ],
        [
            edge(
                sponsor_witness(OperationId::CompactAsh),
                sponsor_fact.clone(),
                ConstructibilityEdgeRole::SponsorOnly,
            ),
            edge(
                sponsor_fact,
                operation,
                ConstructibilityEdgeRole::SponsorOnly,
            ),
        ],
    )
    .unwrap();

    validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()).unwrap();
}

#[test]
fn public_dependency_chain_is_allowed_for_permissionless_operation() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let first_fact = public_count_fact(OperationId::CompactAsh);
    let second_fact = public_amount_fact(OperationId::CompactAsh);
    let (graph, nodes, _) = build_constructibility_graph(
        [
            node(operation.clone()),
            node(first_fact.clone()),
            node(second_fact.clone()),
        ],
        [
            edge(
                first_fact,
                second_fact.clone(),
                ConstructibilityEdgeRole::RequiredFact,
            ),
            edge(
                second_fact,
                operation,
                ConstructibilityEdgeRole::RequiredFact,
            ),
        ],
    )
    .unwrap();

    validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()).unwrap();
}

#[test]
fn cross_operation_private_dependency_is_rejected() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let transfer_witness = owner_witness(OperationId::TransferLive);
    let (graph, nodes, _) = build_constructibility_graph(
        [node(operation.clone()), node(transfer_witness.clone())],
        [edge(
            transfer_witness.clone(),
            operation.clone(),
            ConstructibilityEdgeRole::RequiredWitness,
        )],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(&graph, &nodes, OperationId::CompactAsh, &permissionless()),
        Err(RealizationError::CrossOperationConstructibilityDependency {
            operation: OperationId::CompactAsh,
            source_node: transfer_witness.clone(),
            path: vec![transfer_witness, operation],
        }),
    );
}

#[test]
fn permissionless_laundered_path_is_insertion_order_stable() {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let owner_witness = owner_witness(OperationId::CompactAsh);
    let public_fact = public_count_fact(OperationId::CompactAsh);
    let nodes = [
        node(operation.clone()),
        node(public_fact.clone()),
        node(owner_witness.clone()),
    ];
    let edges = [
        edge(
            owner_witness,
            public_fact.clone(),
            ConstructibilityEdgeRole::RequiredWitness,
        ),
        edge(
            public_fact,
            operation,
            ConstructibilityEdgeRole::RequiredFact,
        ),
    ];
    let (forward_graph, forward_nodes, _) =
        build_constructibility_graph(nodes.clone(), edges.clone()).unwrap();
    let (reversed_graph, reversed_nodes, _) =
        build_constructibility_graph(nodes.into_iter().rev(), edges.into_iter().rev()).unwrap();

    assert_eq!(
        validate_constructibility(
            &forward_graph,
            &forward_nodes,
            OperationId::CompactAsh,
            &permissionless()
        ),
        validate_constructibility(
            &reversed_graph,
            &reversed_nodes,
            OperationId::CompactAsh,
            &permissionless()
        ),
    );
}

#[test]
fn constructibility_authorizations_are_derived_from_permission_class() {
    let arch = &architecture::ARCHITECTURE;
    let cases = |id| constructibility_authorizations(arch.operation(id).unwrap()).unwrap();

    assert_eq!(
        cases(OperationId::AdmitDeposits),
        vec![ConstructibilityAuthorization::Permissionless],
    );
    assert_eq!(
        cases(OperationId::AnnounceMaturity),
        vec![ConstructibilityAuthorization::Operator],
    );
    assert_eq!(
        cases(OperationId::Cycle),
        vec![
            ConstructibilityAuthorization::CadenceOperator,
            ConstructibilityAuthorization::CadencePermissionless,
        ],
    );

    assert!(matches!(
        cases(OperationId::TransferLive).as_slice(),
        [ConstructibilityAuthorization::InputOwners { objects }]
            if objects.contains(&ObjectId::ReceiptLive)
    ));
    assert!(matches!(
        cases(OperationId::CreateRequest).as_slice(),
        [ConstructibilityAuthorization::ClientAuthorized { .. }],
    ));
    assert!(matches!(
        cases(OperationId::CancelRequest).as_slice(),
        [ConstructibilityAuthorization::RefundKey { .. }],
    ));
}

#[test]
fn live_transfer_operator_dependency_is_unavailable_to_owners() {
    // An operator witness is not discharged by receipt-owner
    // authorization: it is unavailable, not merely a permissionless leak.
    let operation = ConstructibilityNodeId::Operation(OperationId::TransferLive);
    let operator = ConstructibilityNodeId::Witness {
        operation: OperationId::TransferLive,
        role: WitnessRole::OperatorAuthorization,
        availability: AvailabilityClass::Operator,
    };
    let (graph, nodes, _) = build_constructibility_graph(
        [node(operation.clone()), node(operator.clone())],
        [edge(
            operator.clone(),
            operation.clone(),
            ConstructibilityEdgeRole::RequiredWitness,
        )],
    )
    .unwrap();

    assert_eq!(
        validate_constructibility(
            &graph,
            &nodes,
            OperationId::TransferLive,
            &input_owners(ObjectId::ReceiptLive),
        ),
        Err(RealizationError::ConstructibilityWitnessUnavailable {
            operation: OperationId::TransferLive,
            authorization: input_owners(ObjectId::ReceiptLive),
            source_node: operator.clone(),
            path: vec![operator, operation],
        }),
    );
}

#[test]
fn mutating_operation_authorization_changes_constructibility() {
    let realization = derive(
        &architecture::ARCHITECTURE,
        crate::RealizationScope::phase1_pilots(),
    )
    .unwrap();
    let graph = &realization.constructibility_graph;
    let nodes = &realization.constructibility_node_by_id;

    // Under the real receipt-owner authorization the live-transfer owner
    // witness is available.
    validate_constructibility(
        graph,
        nodes,
        OperationId::TransferLive,
        &input_owners(ObjectId::ReceiptLive),
    )
    .unwrap();

    // Mutating the typed operation row to permissionless makes the same
    // owner witness a forbidden private dependency: the decision follows
    // the architecture field, not the operation name.
    let mut mutated = *architecture::ARCHITECTURE
        .operation(OperationId::TransferLive)
        .unwrap();
    mutated.authorization = architecture::PermissionClass::Permissionless;

    let cases = constructibility_authorizations(&mutated).unwrap();
    assert_eq!(cases, vec![ConstructibilityAuthorization::Permissionless]);

    assert!(matches!(
        validate_constructibility(graph, nodes, OperationId::TransferLive, &cases[0]),
        Err(RealizationError::PermissionlessPrivateDependency { .. }),
    ));
}

#[test]
fn refund_key_authorization_names_the_refund_input() {
    let cancel = architecture::ARCHITECTURE
        .operation(OperationId::CancelRequest)
        .unwrap();

    let expected_object = cancel
        .inputs
        .iter()
        .find(|input| input.authorization == architecture::InputAuthorization::RefundKey)
        .map(|input| input.object)
        .expect("cancel-request has a refund-key input");

    assert_eq!(
        constructibility_authorizations(cancel).unwrap(),
        vec![ConstructibilityAuthorization::RefundKey {
            object: expected_object,
        }],
    );
}

fn owner_witness(operation: OperationId) -> ConstructibilityNodeId {
    ConstructibilityNodeId::Witness {
        operation,
        role: WitnessRole::ProtocolOwnerAuthorization,
        availability: AvailabilityClass::InputOwners {
            object: ObjectId::Ash,
        },
    }
}

fn sponsor_witness(operation: OperationId) -> ConstructibilityNodeId {
    ConstructibilityNodeId::Witness {
        operation,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    }
}

fn public_count_fact(operation: OperationId) -> ConstructibilityNodeId {
    ConstructibilityNodeId::Fact {
        operation,
        fact: FactId::FamilyCount {
            operation,
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
        availability: AvailabilityClass::Public,
    }
}

fn public_amount_fact(operation: OperationId) -> ConstructibilityNodeId {
    ConstructibilityNodeId::Fact {
        operation,
        fact: FactId::FamilyAmount {
            operation,
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
        availability: AvailabilityClass::Public,
    }
}

fn sponsor_fact(operation: OperationId) -> ConstructibilityNodeId {
    ConstructibilityNodeId::Fact {
        operation,
        fact: FactId::SponsorIsolated { operation },
        availability: AvailabilityClass::SponsorLocal,
    }
}

fn node(id: ConstructibilityNodeId) -> ConstructibilityNode {
    ConstructibilityNode { id }
}

fn edge(
    source: ConstructibilityNodeId,
    target: ConstructibilityNodeId,
    role: ConstructibilityEdgeRole,
) -> ConstructibilityDependencyDeclaration {
    ConstructibilityDependencyDeclaration {
        source,
        target,
        edge: ConstructibilityEdge {
            role,
            strength: RequirementStrength::Required,
        },
    }
}
