use super::{
    tests::{bound, limits},
    *,
};
use crate::keyed_plan::{Defect, Stage};
use architecture::semantic::KeyedOperationId as K;

#[test]
fn production_keyed_representation_and_lifecycle_joins_refuse_disagreement() {
    use crate::engine_lifecycle::{LifecycleRequirementRecord, ModeKey, ObjectKey};
    let input = bound(&[K::Source]);
    let mut subject = Prepared::new(&input).unwrap();
    assert!(subject.analyze(limits()).is_ok());
    let operation = subject.records.relations.scope[0];
    let object = ObjectKey {
        namespace: 2,
        code: 1,
    };
    subject.records.proofs.choices.push((
        crate::engine_proof::ChoiceKey { operation, object },
        Vec::new(),
    ));
    assert!(matches!(
        subject.analyze(limits()),
        Err(KeyedPlanError::Refused {
            defect: Defect::RepresentationChoices,
            ..
        })
    ));
    subject.records.proofs.choices.clear();
    subject
        .records
        .proofs
        .lifecycle
        .push(LifecycleRequirementRecord {
            object,
            representation: ModeKey {
                namespace: 2,
                code: 0,
            },
            exit: operation,
            status: crate::lifecycle::LifecycleExitStatus::AvailableInCompilerScope,
        });
    assert!(matches!(
        subject.analyze(limits()),
        Err(KeyedPlanError::Refused {
            defect: Defect::LifecycleRequirements,
            ..
        })
    ));
}

#[test]
fn production_keyed_unavailable_source_preserves_constructibility_boundary() {
    use crate::engine_constructibility as C;
    let input = bound(&[K::Source]);
    let mut subject = Prepared::new(&input).unwrap();
    let operation = subject.records.relations.scope[0];
    let availability = C::Availability {
        namespace: 2,
        kind: 1,
        object: 1,
        sponsor_local: false,
    };
    let root = C::NodeKey::operation(operation);
    let source = C::NodeKey {
        kind: 1,
        operation,
        subject: vec![1],
        availability: Some(availability),
        independent: false,
    };
    subject.records.constructibility = C::Records {
        scope: vec![operation],
        nodes: vec![C::Node { id: root.clone() }, C::Node { id: source.clone() }],
        edges: vec![C::Dependency {
            source,
            target: root,
            edge: C::Edge {
                kind: 0,
                required: true,
                sponsor_only: false,
            },
        }],
        operations: vec![(
            operation,
            vec![C::Authorization {
                key: crate::engine_identity::AuthorizationKey::new(
                    architecture::semantic::SemanticProfileId::KeyedProvenance,
                    vec![crate::engine_identity::word(0u16)],
                ),
                permissionless: true,
                allowed: std::collections::BTreeSet::from([availability]),
            }],
        )],
    };
    assert!(subject.analyze(limits()).is_ok());
    subject.records.constructibility.operations[0].1[0]
        .allowed
        .clear();
    assert_eq!(
        subject.analyze(limits()).err(),
        Some(KeyedPlanError::Refused {
            stage: Stage::Constructibility,
            defect: Defect::PermissionlessPrivateDependency,
            relations: Vec::new(),
            operations: vec![architecture::semantic::SemanticOperationId::Keyed(
                K::Source
            )],
            unrestored_key_count: 0,
        })
    );
}

#[test]
fn production_keyed_reader_assignment_refuses_at_placement_census() {
    let input = bound(&[K::Source, K::Merge]);
    let mut subject = Prepared::new(&input).unwrap();
    assert!(subject.analyze(limits()).is_ok());
    let mut carrier = subject
        .records
        .cases
        .rows
        .iter()
        .find(|r| r.runtime)
        .unwrap()
        .eligibility[0]
        .clone();
    let reader = subject
        .records
        .cases
        .rows
        .iter_mut()
        .find(|r| {
            matches!(
                subject.relations[&r.key.relation].relation,
                realization::relation::KeyedRelation::MembershipReading { .. }
            )
        })
        .unwrap();
    carrier.key = reader.key.clone();
    reader.eligibility.push(carrier);
    let reader_key = reader.key.relation.clone();
    let relation = subject.relation(&reader_key).unwrap();
    assert_eq!(
        relation.kind,
        realization::relation::KeyedRelationKind::MembershipReading
    );
    assert_eq!(
        subject.analyze(limits()).err(),
        Some(KeyedPlanError::Refused {
            stage: Stage::Placement,
            defect: Defect::CensusMismatch,
            relations: vec![relation],
            operations: vec![relation.operation],
            unrestored_key_count: 0,
        })
    );
}

#[test]
fn production_routes_refuse_foreign_bindings_and_vocabulary_both_directions() {
    use crate::engine_records::RelationOwner;
    let keyed = bound(&[K::Source]);
    let attestation = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
    let mut prepared = Prepared::new(&keyed).unwrap();
    let mut program = prepared.analyze(limits()).unwrap();
    assert!(prepared.publish(&program).is_ok());
    program.source = attestation.owner().source_binding();
    assert_eq!(
        prepared.publish(&program),
        Err(KeyedPlanError::PublicationMismatch)
    );
    prepared.records.proofs = crate::input::AttestationOwner::lower_proofs(&attestation).unwrap();
    let unknown = prepared.records.proofs.obligations.len();
    let relations = keyed
        .owner()
        .source()
        .project()
        .into_iter()
        .filter(|r| keyed.owner().scope().contains(&r.id.operation))
        .map(|r| r.id)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(
        prepared.analyze(limits()).err(),
        Some(KeyedPlanError::Refused {
            stage: Stage::Proof,
            defect: Defect::CensusMismatch,
            relations,
            operations: vec![architecture::semantic::SemanticOperationId::Keyed(
                K::Source
            )],
            unrestored_key_count: unknown,
        })
    );
    let (mut records, stages) =
        crate::engine_attestation::program::test_preparation(&attestation).unwrap();
    let (control_records, control_stages) =
        crate::engine_attestation::program::test_preparation(&attestation).unwrap();
    assert!(
        crate::analyzed::analyze_bound_prepared(
            &attestation,
            limits(),
            (control_records, control_stages)
        )
        .is_ok()
    );
    records.proofs = KeyedOwner::lower_proofs(&keyed).unwrap();
    let result = crate::analyzed::analyze_bound_prepared(&attestation, limits(), (records, stages));
    assert!(matches!(
        result,
        Err(core::Error::StageCensus {
            stage: core::Stage::Proof,
            ..
        })
    ));
}

#[test]
fn keyed_publication_names_unexpected_layouts_and_runtime_scope() {
    let input = bound(&[K::Merge]);
    let prepared = Prepared::new(&input).unwrap();
    let program = prepared.analyze(limits()).unwrap();
    assert!(prepared.publish(&program).is_ok());
    let legacy_input = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
    let (legacy, _) = crate::engine_attestation::program::test_preparation(&legacy_input).unwrap();
    let layout = legacy.cases.layout_scopes.keys().next().unwrap();
    for variant in 0..6 {
        let mut subject = program.clone();
        let operation = subject
            .plans
            .values_mut()
            .next()
            .unwrap()
            .operations
            .values_mut()
            .next()
            .unwrap();
        let mut relations = Vec::new();
        let (stage, defect) = match variant {
            0 => {
                operation.layout.insert(layout.clone());
                (Stage::Coverage, Defect::UnexpectedLayout)
            }
            1 => {
                operation.placements[0].layout.push(layout.clone());
                (Stage::Placement, Defect::UnexpectedLayout)
            }
            _ => {
                let (key, bundle) = operation
                    .bundles
                    .iter_mut()
                    .find(|(_, bundle)| !bundle.facts.runtime.is_empty())
                    .unwrap();
                relations.push(prepared.relation(&key.relation).unwrap());
                if variant == 2 {
                    let mut alternative = bundle.carriers.pop_first().unwrap();
                    alternative.layout.insert(layout.clone());
                    bundle.carriers.insert(alternative);
                    (Stage::Coverage, Defect::UnexpectedLayout)
                } else if variant == 3 {
                    bundle.facts.runtime[0].scope = crate::engine_case_facts::Scope::Transaction;
                    (Stage::Cases, Defect::UnsupportedRuntimeScope)
                } else {
                    let crate::engine_case_facts::Scope::Member { side, object } =
                        &mut bundle.facts.runtime[0].scope
                    else {
                        panic!("member")
                    };
                    if variant == 4 {
                        *side = u16::MAX;
                        (Stage::Cases, Defect::UnsupportedRuntimeScope)
                    } else {
                        object.code = u16::MAX;
                        (Stage::Cases, Defect::RuntimeObjectMismatch)
                    }
                }
            }
        };
        assert_eq!(
            prepared.publish(&subject).err(),
            Some(KeyedPlanError::Refused {
                stage,
                defect,
                relations,
                operations: vec![architecture::semantic::SemanticOperationId::Keyed(K::Merge)],
                unrestored_key_count: 0,
            })
        );
    }
}

#[test]
fn keyed_layout_symbol_refuses_at_coverage_with_one_unknown_key() {
    let input = bound(&[K::Merge]);
    let prepared = Prepared::new(&input).unwrap();
    assert!(
        prepared
            .publish(&prepared.analyze(limits()).unwrap())
            .is_ok()
    );
    let legacy_input = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
    let (legacy, _) = crate::engine_attestation::program::test_preparation(&legacy_input).unwrap();
    let layout = legacy.cases.layout_scopes.keys().next().unwrap().clone();
    assert_eq!(
        prepared.symbol(&crate::engine_coverage_graph::Key::Layout(layout)),
        Err(KeyedPlanError::Refused {
            stage: Stage::Coverage,
            defect: Defect::UnexpectedLayout,
            relations: Vec::new(),
            operations: Vec::new(),
            unrestored_key_count: 1,
        })
    );
}
