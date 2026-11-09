//! Whole-record pipeline and statically bound re-derivation controls.
use crate::{engine_program as core, engine_records::RelationOwner};
use architecture::{OperationId, semantic::KeyedOperationId};

fn limits() -> crate::placement::PlacementSearchLimits {
    crate::placement::PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("states"),
        std::num::NonZeroU64::new(1_000_000).expect("candidates"),
    )
}

#[test]
fn keyed_whole_record_pipeline_reaches_seven_stages_and_rederives() {
    for (scope, relations, evidence, runtime) in [
        (vec![KeyedOperationId::Source], 4, 4, 0),
        (vec![KeyedOperationId::Merge], 5, 3, 2),
        (
            vec![KeyedOperationId::Source, KeyedOperationId::Merge],
            9,
            7,
            2,
        ),
    ] {
        let input = super::engine_coverage_tests::bound(scope);
        let program =
            crate::analyzed::analyze_bound_records(&input, limits()).expect("whole pipeline");
        assert_eq!(program.relations.nodes.len(), relations);
        assert_eq!(program.relations.edges.len(), 0);
        assert_eq!(program.sources.len(), relations);
        assert_eq!(program.constructibility.nodes.len(), 0);
        assert_eq!(program.construction.len(), 0);
        assert_eq!(program.lifecycle.nodes.len(), 0);
        assert_eq!(program.plans.len(), 1);
        assert_eq!(program.evidence.len(), evidence);
        let plan = program.plans.values().next().expect("plan");
        assert_eq!(plan.requirements.len(), relations);
        assert_eq!(
            plan.operations
                .values()
                .map(|operation| operation.bundles.len())
                .sum::<usize>(),
            relations
        );
        assert_eq!(
            plan.operations
                .values()
                .flat_map(|operation| &operation.placements)
                .flat_map(|placement| &placement.assignments)
                .count(),
            runtime
        );
        assert!(core::validate(&input, limits(), &program).is_ok());
        let mut missing = program;
        missing.evidence.pop_first();
        assert_eq!(
            core::validate(&input, limits(), &missing),
            Err(core::Error::ProgramMismatch)
        );
    }
}

#[test]
fn both_pipeline_directions_refuse_foreign_bindings_and_proof_vocabulary() {
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    let keyed = super::engine_coverage_tests::bound(vec![KeyedOperationId::Source]);
    let attestation_records = core::lower(&attestation).expect("attestation records");
    let keyed_records = core::lower(&keyed).expect("keyed records");
    let mut first = crate::analyzed::analyze_bound_records(&attestation, limits())
        .expect("attestation pipeline");
    let mut second =
        crate::analyzed::analyze_bound_records(&keyed, limits()).expect("keyed pipeline");
    assert!(core::validate(&attestation, limits(), &first).is_ok());
    assert!(core::validate(&keyed, limits(), &second).is_ok());
    std::mem::swap(&mut first.source, &mut second.source);
    assert_eq!(
        core::validate(&attestation, limits(), &first),
        Err(core::Error::SourceMismatch)
    );
    assert_eq!(
        core::validate(&keyed, limits(), &second),
        Err(core::Error::SourceMismatch)
    );
    for (mut records, policy, foreign) in [
        (attestation_records, attestation.policy(), 2),
        (keyed_records, keyed.policy(), 1),
    ] {
        let alternative = records
            .proofs
            .obligations
            .values_mut()
            .find_map(|obligation| match obligation {
                crate::engine_proof::Obligation::Static => None,
                crate::engine_proof::Obligation::Evidence(alternative) => Some(alternative),
                crate::engine_proof::Obligation::Required(alternatives) => alternatives.first_mut(),
            })
            .expect("proof control");
        alternative.id.namespace = foreign;
        let id = alternative.id.clone();
        assert_eq!(
            core::analyze(&records, policy.proof_search_limits, limits()).err(),
            Some(core::Error::ProofSourceMismatch(id))
        );
    }
    assert_ne!(
        attestation.owner().source_binding(),
        keyed.owner().source_binding()
    );
}

#[test]
fn whole_pipeline_refuses_a_reader_carrier_and_cross_stage_relation_leakage() {
    let input = super::engine_coverage_tests::bound(vec![
        KeyedOperationId::Source,
        KeyedOperationId::Merge,
    ]);
    let mut records = core::lower(&input).expect("lower");
    assert!(core::analyze(&records, input.policy().proof_search_limits, limits()).is_ok());
    let runtime = records
        .cases
        .rows
        .iter()
        .find(|row| row.runtime)
        .expect("runtime control")
        .eligibility[0]
        .clone();
    let reader = records
        .cases
        .rows
        .iter_mut()
        .find(|row| !row.runtime)
        .expect("evidence control");
    let mut foreign_carrier = runtime;
    foreign_carrier.key = reader.key.clone();
    reader.eligibility.push(foreign_carrier);
    assert!(
        matches!(core::analyze(&records, input.policy().proof_search_limits, limits()), Err(core::Error::Operations(error)) if matches!(&*error, crate::engine_operation::Error::Placement(error) if matches!(**error, crate::engine_placement::Error::PlacementCensusMismatch { .. })))
    );
    let mut records = core::lower(&input).expect("fresh control");
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    records.proofs =
        crate::input::AttestationOwner::lower_proofs(&attestation).expect("foreign vocabulary");
    assert!(matches!(
        core::analyze(&records, input.policy().proof_search_limits, limits()),
        Err(core::Error::StageCensus {
            stage: core::Stage::Proof,
            ..
        })
    ));
}

#[test]
fn whole_pipeline_refuses_unknown_and_cyclic_dependencies() {
    use crate::engine_records::{DependencyKind, DependencyRecord, RelationRecordError};
    let input = super::engine_coverage_tests::bound(vec![KeyedOperationId::Source]);
    let mut records = core::lower(&input).expect("lower");
    assert!(core::analyze(&records, input.policy().proof_search_limits, limits()).is_ok());
    let source = records.relations.nodes[0].id.clone();
    let target = records.relations.nodes[1].id.clone();
    let edge = DependencyRecord {
        source: source.clone(),
        target: target.clone(),
        kind: DependencyKind {
            namespace: 2,
            code: 1,
        },
    };
    records.relations.edges.push(edge.clone());
    assert!(core::analyze(&records, input.policy().proof_search_limits, limits()).is_ok());
    records.relations.edges.push(DependencyRecord {
        source: target,
        target: source,
        kind: edge.kind,
    });
    assert!(matches!(
        core::analyze(&records, input.policy().proof_search_limits, limits()),
        Err(core::Error::Relations(RelationRecordError::Cycle { .. }))
    ));
    records.relations.edges.pop();
    records.relations.edges[0].target.subject.push(u16::MAX);
    assert!(matches!(
        core::analyze(&records, input.policy().proof_search_limits, limits()),
        Err(core::Error::Relations(
            RelationRecordError::UnknownEndpoint { .. }
        ))
    ));
}

#[test]
fn whole_pipeline_refuses_unavailable_required_source() {
    use crate::engine_constructibility as construction;
    let input = super::engine_coverage_tests::bound(vec![KeyedOperationId::Source]);
    let mut records = core::lower(&input).expect("lower");
    let operation = records.relations.scope[0];
    let availability = construction::Availability {
        namespace: 2,
        kind: 1,
        object: 1,
        sponsor_local: false,
    };
    let root = construction::NodeKey::operation(operation);
    let source = construction::NodeKey {
        kind: 1,
        operation,
        subject: vec![1],
        availability: Some(availability),
        independent: false,
    };
    records.constructibility = construction::Records {
        scope: vec![operation],
        nodes: vec![
            construction::Node { id: root.clone() },
            construction::Node { id: source.clone() },
        ],
        edges: vec![construction::Dependency {
            source,
            target: root,
            edge: construction::Edge {
                kind: 0,
                required: true,
                sponsor_only: false,
            },
        }],
        operations: vec![(
            operation,
            vec![construction::Authorization {
                key: vec![0],
                permissionless: true,
                allowed: std::collections::BTreeSet::from([availability]),
            }],
        )],
    };
    assert!(core::analyze(&records, input.policy().proof_search_limits, limits()).is_ok());
    records.constructibility.operations[0].1[0].allowed.clear();
    assert!(
        matches!(core::analyze(&records, input.policy().proof_search_limits, limits()), Err(core::Error::Constructibility(error)) if matches!(*error, construction::RecordError::PermissionlessPrivateDependency { .. }))
    );
}
