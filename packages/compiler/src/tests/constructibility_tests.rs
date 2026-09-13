//! Constructibility analysis tests (Guide-3 §17.3).

use architecture::{ObjectId, OperationId};
use realization::{
    AvailabilityClass, ConstructibilityAuthorization, ProofKind, Relation, RelationDeclaration,
    RelationId, RelationKind, RelationSubject, WitnessRole,
};

use crate::{capability::CapabilityView, proof::enumerate_feasible_plans};
use architecture::OperationId::AnnounceMaturity;

use super::{announcement_input, bound_input};
use crate::{
    CompileError,
    constructibility::{build_constructibility_analysis, validate_source_constructibility},
    source::derive_source_requirements,
};

#[test]
fn pilot_cases_derive_and_validate() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let analysis = build_constructibility_analysis(&input).expect("constructibility builds");

    let operation = |id: OperationId| {
        analysis
            .operations
            .iter()
            .find(|row| row.operation == id)
            .expect("operation analyzed")
    };

    // Compact ASH is permissionless; its sponsor witness is optional.
    let ash = operation(OperationId::CompactAsh);
    assert_eq!(ash.cases.len(), 1);
    assert_eq!(
        ash.cases[0].authorization,
        ConstructibilityAuthorization::Permissionless
    );
    assert!(
        ash.cases[0]
            .required_availability
            .iter()
            .all(|availability| *availability == AvailabilityClass::Public),
        "permissionless required availability must be public only",
    );
    assert!(ash.cases[0].optional_nodes.iter().any(|node| {
        matches!(
            node,
            realization::ConstructibilityNodeId::Witness {
                role: WitnessRole::SponsorAuthorization,
                availability: AvailabilityClass::SponsorLocal,
                ..
            }
        )
    }));

    // Live transfer is input-owner authorized, and the owner witness is
    // available in its case.
    let transfer = operation(OperationId::TransferLive);
    assert_eq!(transfer.cases.len(), 1);
    assert!(matches!(
        &transfer.cases[0].authorization,
        ConstructibilityAuthorization::InputOwners { objects }
            if objects.contains(&ObjectId::ReceiptLive)
    ));
    assert!(
        transfer.cases[0]
            .required_availability
            .contains(&AvailabilityClass::InputOwners {
                object: ObjectId::ReceiptLive,
            })
    );
}

#[test]
fn source_witnesses_are_welded_to_constructibility() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let analysis = build_constructibility_analysis(&input).expect("constructibility builds");

    // The live-transfer owner-authorization sources are consistent.
    let owner = RelationDeclaration {
        id: RelationId::new(
            OperationId::TransferLive,
            RelationKind::Authorization,
            RelationSubject::Operation,
        ),
        relation: Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        },
        proof_alternatives: std::collections::BTreeSet::new(),
    };
    let rows = derive_source_requirements(&owner, ProofKind::SignerMembership).unwrap();
    validate_source_constructibility(&analysis, &owner.id, &rows).expect("consistent witnesses");

    // The same owner witness does not exist for compact ASH: the
    // requirement fails closed rather than assuming a witness.
    let misplaced = RelationDeclaration {
        id: RelationId::new(
            OperationId::CompactAsh,
            RelationKind::Authorization,
            RelationSubject::Operation,
        ),
        relation: Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        },
        proof_alternatives: std::collections::BTreeSet::new(),
    };
    let rows = derive_source_requirements(&misplaced, ProofKind::SignerMembership).unwrap();
    assert!(matches!(
        validate_source_constructibility(&analysis, &misplaced.id, &rows).unwrap_err(),
        CompileError::SourceConstructibilityMismatch { .. },
    ));

    // Sponsor witnesses stay in the optional sponsor subtree.
    let sponsor = RelationDeclaration {
        id: RelationId::new(
            OperationId::CompactAsh,
            RelationKind::SponsorIsolation,
            RelationSubject::Sponsor,
        ),
        relation: Relation::SponsorIsolation,
        proof_alternatives: std::collections::BTreeSet::new(),
    };
    let rows = derive_source_requirements(&sponsor, ProofKind::ManifestShape).unwrap();
    validate_source_constructibility(&analysis, &sponsor.id, &rows)
        .expect("sponsor witness is optional and present");
}

#[test]
fn single_pilot_scopes_analyze_independently() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let input = bound_input(&[operation]);
        let analysis = build_constructibility_analysis(&input).expect("builds");

        assert_eq!(analysis.operations.len(), 1);
        assert_eq!(analysis.operations[0].operation, operation);

        // Diagnostic-free construction is repeatable and equal.
        let again = build_constructibility_analysis(&input).expect("builds again");
        assert_eq!(analysis.operations, again.operations);
    }
}

#[test]
fn announcement_requires_operator_and_keeps_sponsor_optional() {
    use petgraph::visit::EdgeRef;
    use realization::{
        ConstructibilityEdgeRole as Role, ConstructibilityNodeId as Node,
        RequirementStrength as Strength,
    };
    use std::collections::BTreeSet;

    let input = announcement_input();
    let analysis = build_constructibility_analysis(&input).unwrap();
    let operation = AnnounceMaturity;
    let operator = Node::Witness {
        operation,
        role: WitnessRole::OperatorAuthorization,
        availability: AvailabilityClass::Operator,
    };
    let sponsor = Node::Witness {
        operation,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    assert_eq!(
        analysis.node_by_id.keys().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            Node::Operation(operation),
            operator.clone(),
            sponsor.clone()
        ])
    );
    assert_eq!(analysis.graph.node_count(), 3);
    let edges: BTreeSet<_> = analysis
        .graph
        .edge_references()
        .map(|edge| {
            (
                analysis.graph[edge.source()].id.clone(),
                analysis.graph[edge.target()].id.clone(),
                edge.weight().role,
                edge.weight().strength,
            )
        })
        .collect();
    assert_eq!(
        edges,
        BTreeSet::from([
            (
                operator.clone(),
                Node::Operation(operation),
                Role::RequiredWitness,
                Strength::Required
            ),
            (
                sponsor.clone(),
                Node::Operation(operation),
                Role::SponsorOnly,
                Strength::Optional
            ),
        ])
    );
    assert_eq!(analysis.graph.edge_count(), 2);
    assert_eq!(analysis.operations.len(), 1);
    assert_eq!(analysis.operations[0].cases.len(), 1);
    let case = &analysis.operations[0].cases[0];
    assert_eq!(case.authorization, ConstructibilityAuthorization::Operator);
    assert_eq!(case.required_nodes, vec![operator]);
    assert_eq!(case.optional_nodes, vec![sponsor]);
    assert_eq!(
        case.required_availability,
        BTreeSet::from([AvailabilityClass::Operator])
    );
    let relations = crate::relation::build_relation_analysis(&input).unwrap();
    for candidate in enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .unwrap()
        .candidates
    {
        let cases = crate::case::execution_cases(&relations, &candidate).unwrap();
        let absent = cases
            .iter()
            .find(|case| case.id.sponsor == crate::case::SponsorCase::Absent)
            .unwrap();
        assert!(
            absent
                .active_sources
                .iter()
                .all(|row| row.availability != AvailabilityClass::SponsorLocal)
        );
        assert_eq!(
            case.required_availability,
            BTreeSet::from([AvailabilityClass::Operator])
        );
    }
}
