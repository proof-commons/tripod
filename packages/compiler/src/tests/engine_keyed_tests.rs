//! Both statically bound owners reach the same relation-record algorithm.
//! These are graph-stage controls, not a claim of complete keyed analysis.

use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};
use realization::keyed::KeyedRealizationDraft;

#[test]
fn keyed_case_facts_preserve_their_typed_lowering_refusals() {
    use crate::engine_keyed::{
        KeyedLoweringError as E, case_facts, lower_coverage, lower_placements,
    };
    use crate::engine_records::RelationOwner;
    let input = super::engine_coverage_tests::bound(vec![KeyedOperationId::Merge]);
    let placements = lower_placements(input.owner());
    let key = &placements
        .plans
        .iter()
        .find(|(_, runtime)| *runtime)
        .expect("runtime clause")
        .0;
    let coverage = lower_coverage(input.owner())
        .into_iter()
        .find(|plan| plan.key == *key)
        .expect("coverage");
    let control = input.owner().lower_sources().expect("source records");
    assert!(case_facts(key, true, &coverage, &control).is_ok());
    let mut subject = control.clone();
    subject
        .tables
        .retain(|table| table.relation != key.relation);
    assert_eq!(
        case_facts(key, true, &coverage, &subject).err(),
        Some(E::MissingSourceTable(key.relation.clone()))
    );
    let mut subject = control;
    let table = subject
        .tables
        .iter_mut()
        .find(|table| table.relation == key.relation)
        .expect("source table");
    table.rows[0].operand.payload = crate::engine_source::Payload::None;
    assert_eq!(
        case_facts(key, true, &coverage, &subject).err(),
        Some(E::RuntimeSourcePayload(key.relation.clone()))
    );
}

#[test]
fn keyed_lowering_reads_every_declaration_shell_field() {
    use crate::engine_keyed::{KeyedLoweringError, ShellField as F, read_shells};
    let manifest = KeyedManifest::contract().expect("manifest");
    let control = KeyedRealizationDraft::derive(&manifest).expect("declarations");
    control
        .clone()
        .validate(&manifest)
        .expect("accepted declaration");
    let scope = control
        .operations
        .iter()
        .map(|operation| operation.operation)
        .collect::<Vec<_>>();
    assert!(read_shells(&control, &scope).is_ok());
    let reference = super::bound_input(&[architecture::OperationId::CompactAsh])
        .realization()
        .operation(architecture::OperationId::CompactAsh)
        .expect("attestation fixture")
        .clone();
    let mut subjects = Vec::new();
    macro_rules! changed {
        ($field:ident, $kind:path) => {{
            let mut subject = control.clone();
            subject.operations[0]
                .$field
                .push(reference.$field[0].clone());
            subjects.push(($kind, subject));
        }};
    }
    changed!(relation_dependencies, F::RelationDependencies);
    changed!(constructibility_nodes, F::ConstructibilityNodes);
    changed!(constructibility_edges, F::ConstructibilityEdges);
    changed!(lifecycle_nodes, F::LifecycleNodes);
    changed!(lifecycle_edges, F::LifecycleEdges);
    changed!(disclosure_nodes, F::DisclosureNodes);
    changed!(disclosure_edges, F::DisclosureEdges);
    changed!(disclosure_seeds, F::DisclosureSeeds);
    let mut subject = control;
    let fact = realization::FactId::FamilyCount {
        operation: architecture::OperationId::CompactAsh,
        side: realization::TransactionSide::Input,
        object: architecture::ObjectId::Ash,
    };
    subject.operations[0]
        .expressions
        .push(realization::ExpressionDeclaration {
            id: realization::ExprId::fact(fact.clone()),
            ty: realization::SemanticType::Count,
            node: realization::ExpressionNode::Fact(fact),
        });
    subjects.push((F::Expressions, subject));
    for (field, subject) in subjects {
        assert_eq!(
            read_shells(&subject, &scope).err(),
            Some(KeyedLoweringError::UnsupportedShell {
                operation: scope[0],
                field,
                count: 1
            })
        );
    }
}

#[test]
fn attestation_binding_rejects_the_keyed_source_type() {
    // If these types become equal, inference has two valid implementations and
    // this unit test no longer compiles. The assertion concerns the binder's
    // actual associated input type, rather than a missing import or feature.
    trait Distinct<A> {
        fn assert_distinct() {}
    }
    impl<T> Distinct<()> for T {}
    impl Distinct<u8> for <architecture::Architecture as crate::input::InputBinding>::Realization {}
    <realization::keyed::KeyedScopedRealization as Distinct<_>>::assert_distinct();
}

#[test]
fn keyed_binding_rejects_the_attestation_source_type() {
    trait Distinct<A> {
        fn assert_distinct() {}
    }
    impl<T> Distinct<()> for T {}
    impl Distinct<u8> for <KeyedManifest as crate::input::InputBinding>::Realization {}
    <realization::ScopedRealizationSpec as Distinct<_>>::assert_distinct();
}

use crate::{
    bind_input,
    engine_records::{
        self, DependencyKind, DependencyRecord, LoweredRelations, RelationRecordError,
    },
    input::KeyedBindingError,
};

const SOURCE: SemanticOperationId = SemanticOperationId::Keyed(KeyedOperationId::Source);
const MERGE: SemanticOperationId = SemanticOperationId::Keyed(KeyedOperationId::Merge);

fn bound(scope: Vec<SemanticOperationId>) -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::contract().expect("paper binding");
    let source = KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("owner validation");
    bind_input(
        &manifest,
        source,
        scope,
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("keyed binding")
}

#[test]
fn source_merge_and_combined_scopes_use_the_shared_graph_entry() {
    for (scope, census) in [
        (vec![SOURCE], 4),
        (vec![MERGE], 5),
        (vec![SOURCE, MERGE], 9),
    ] {
        let input = bound(scope.clone());
        let analyzed = engine_records::analyze_bound_relations(&input).expect("owner lowering");
        let graph = analyzed.result.expect("keyed graph");
        let expected = input
            .owner()
            .source()
            .project()
            .into_iter()
            .filter(|row| scope.contains(&row.id.operation))
            .map(|row| crate::engine_keyed::relation_key(row.id))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(graph.graph.node_count(), census);
        assert_eq!(graph.graph.edge_count(), 0);
        assert_eq!(
            graph
                .graph
                .node_weights()
                .map(|row| row.id.clone())
                .collect::<std::collections::BTreeSet<_>>(),
            expected
        );
        assert_eq!(graph.evaluation_order.len(), census);
        // Each call statically lowers the retained owner again, with no cached
        // projected source accepted as authority.
        let repeated = engine_records::analyze_bound_relations(&input).expect("owner lowering");
        assert_eq!(analyzed.lowering.records(), repeated.lowering.records());
        assert!(repeated.result.is_ok());
    }
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    let analyzed = engine_records::analyze_bound_relations(&attestation).expect("owner lowering");
    assert!(analyzed.result.is_ok());
    assert!(
        analyzed
            .lowering
            .records()
            .nodes
            .iter()
            .all(|node| node.id.operation.namespace == 1)
    );
}

#[test]
fn keyed_binding_refuses_an_attestation_scope_with_its_numeric_collision() {
    let manifest = KeyedManifest::contract().expect("paper binding");
    let source = KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("owner validation");
    let policy = super::bound_input(&[OperationId::CompactAsh]).policy();
    assert!(bind_input(&manifest, source.clone(), vec![SOURCE], policy).is_ok());
    let foreign = SemanticOperationId::Attestation(OperationId::CreateRequest);
    assert_eq!(SOURCE.encoding()[1], foreign.encoding()[1]);
    assert_eq!(
        bind_input(&manifest, source, vec![foreign], policy).err(),
        Some(KeyedBindingError::ForeignOperation(foreign))
    );
}

#[test]
fn keyed_declared_graph_refuses_unknown_and_cyclic_dependencies() {
    let input = bound(vec![MERGE]);
    let analyzed = engine_records::analyze_bound_relations(&input).expect("owner lowering");
    assert!(analyzed.result.is_ok());
    let control = analyzed.lowering.records().clone();
    let first = control
        .nodes
        .iter()
        .find(|node| node.id.operation == crate::engine_keyed::operation_key(MERGE))
        .expect("MERGE relation")
        .id
        .clone();
    let mut unknown_id = first.clone();
    unknown_id.kind = u16::MAX;
    let kind = DependencyKind {
        namespace: 2,
        code: 1,
    };
    let mut unknown = control.clone();
    unknown.edges.push(DependencyRecord {
        source: first.clone(),
        target: unknown_id.clone(),
        kind,
    });
    assert_eq!(
        engine_records::analyze_relations(&unknown).err(),
        Some(RelationRecordError::UnknownEndpoint {
            relation: unknown_id
        })
    );
    let mut cyclic = control;
    cyclic.edges.push(DependencyRecord {
        source: first.clone(),
        target: first,
        kind,
    });
    assert!(matches!(
        engine_records::analyze_relations(&cyclic),
        Err(RelationRecordError::Cycle { .. })
    ));
}
