use crate::keyed::{KeyedRealizationDraft, KeyedRealizationError};
use crate::relation::KeyedRelation;
use architecture::keyed::KeyedManifest;
use architecture::semantic::{KeyedOperationId, SemanticOperationId, SemanticProfileId};

fn control() -> (KeyedManifest, KeyedRealizationDraft) {
    let manifest = KeyedManifest::probe().unwrap();
    let draft = KeyedRealizationDraft::derive(&manifest).unwrap();
    draft.clone().validate(&manifest).unwrap();
    (manifest, draft)
}

#[test]
fn keyed_declaration_census_refuses_nonempty_shell_fields() {
    let (manifest, accepted) = control();
    let operation = accepted.operations[0].operation;
    let reference = crate::declarations::derive_operation(
        &architecture::ARCHITECTURE,
        architecture::OperationId::CompactAsh,
    )
    .expect("attestation fixture");
    macro_rules! refused_field {
        ($field:ident) => {{
            let mut subject = accepted.clone();
            subject.operations[0]
                .$field
                .push(reference.$field[0].clone());
            assert_eq!(
                subject.validate(&manifest).unwrap_err(),
                KeyedRealizationError::DeclarationCensus(operation)
            );
        }};
    }
    refused_field!(relation_dependencies);
    refused_field!(constructibility_nodes);
    refused_field!(constructibility_edges);
    refused_field!(lifecycle_nodes);
    refused_field!(lifecycle_edges);
    refused_field!(disclosure_nodes);
    refused_field!(disclosure_edges);
    refused_field!(disclosure_seeds);
    let mut subject = accepted;
    let fact = crate::FactId::FamilyCount {
        operation: architecture::OperationId::CompactAsh,
        side: crate::TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    };
    subject.operations[0]
        .expressions
        .push(crate::ExpressionDeclaration {
            id: crate::ExprId::fact(fact.clone()),
            ty: crate::SemanticType::Count,
            node: crate::ExpressionNode::Fact(fact),
        });
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::DeclarationCensus(operation)
    );
}

#[test]
fn a_changed_profile_binding_is_refused_by_the_realization_owner() {
    let (manifest, mut subject) = control();
    subject.binding.profile = SemanticProfileId::Attestation;
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::ArchitectureBindingMismatch
    );
}

#[test]
fn removing_either_clause_is_refused_even_without_dangling_edges() {
    for clause in [KeyedRelation::BornFirst, KeyedRelation::ContinueFirst] {
        let (manifest, mut subject) = control();
        let merge = SemanticOperationId::Keyed(KeyedOperationId::Merge);
        let operation = subject
            .operations
            .iter_mut()
            .find(|operation| operation.operation == merge)
            .unwrap();
        let before = operation.relations.len();
        operation.relations.retain(|row| row.relation != clause);
        assert_eq!(operation.relations.len() + 1, before);
        assert_eq!(
            subject.validate(&manifest).unwrap_err(),
            KeyedRealizationError::DeclarationCensus(merge)
        );
    }
}

#[test]
fn a_duplicated_relation_is_refused_by_the_graph_owner() {
    let (manifest, mut subject) = control();
    let repeated = subject.operations[0].relations[0];
    subject.operations[0].relations.push(repeated);
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::DuplicateRelation(repeated.id)
    );
}

#[test]
fn a_foreign_operation_is_refused_before_graph_assembly() {
    let (manifest, mut subject) = control();
    let foreign = SemanticOperationId::Attestation(architecture::OperationId::CompactAsh);
    subject.operations[0].operation = foreign;
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::ForeignOperation(foreign)
    );
}

#[test]
fn keyed_projection_is_permutation_stable_and_has_no_attestation_graphs() {
    let (manifest, mut subject) = control();
    let baseline = subject.clone().validate(&manifest).unwrap().project();
    assert_eq!(baseline.len(), 9);
    for operation in &mut subject.operations {
        assert_eq!(operation.expressions, []);
        assert_eq!(operation.constructibility_nodes, []);
        assert_eq!(operation.lifecycle_nodes, []);
        assert_eq!(operation.disclosure_nodes, []);
        operation.relations.reverse();
    }
    subject.operations.reverse();
    assert_eq!(subject.validate(&manifest).unwrap().project(), baseline);
}

#[test]
fn a_keyed_declaration_cannot_smuggle_a_lifecycle_constraint() {
    let (manifest, mut subject) = control();
    let operation = subject.operations[0].operation;
    subject.operations[0]
        .lifecycle_nodes
        .push(crate::LifecycleNode {
            id: crate::LifecycleNodeId::Representation {
                object: architecture::ObjectId::State,
                mode: crate::RepresentationMode::Explicit,
            },
        });
    assert_eq!(
        subject.validate(&manifest).unwrap_err(),
        KeyedRealizationError::DeclarationCensus(operation)
    );
}
