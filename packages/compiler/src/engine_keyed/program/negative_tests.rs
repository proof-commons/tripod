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
            )]
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
    assert!(matches!(
        subject.analyze(limits()),
        Err(KeyedPlanError::Refused {
            stage: Stage::Placement,
            defect: Defect::CensusMismatch,
            ..
        })
    ));
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
    assert_eq!(prepared.analyze(limits()).err(), Some(super::missing()));
    let (mut records, stages) =
        crate::engine_attestation::program::test_preparation(&attestation).unwrap();
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
