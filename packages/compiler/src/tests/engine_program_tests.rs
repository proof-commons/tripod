//! Whole-record pipeline and statically bound re-derivation controls.
use crate::{engine_program as core, engine_records::RelationOwner};
use architecture::{OperationId, semantic::KeyedOperationId};

#[test]
fn each_independent_derivation_builds_the_shared_inputs_once() {
    use super::tracing_capture::{capture, isolate_capture_test};
    if isolate_capture_test(
        "tests::engine_program_tests::each_independent_derivation_builds_the_shared_inputs_once",
    ) {
        return;
    }
    let input = super::bound_input(&[OperationId::CompactAsh]);
    // Each call starts from retained source; no cache crosses the independent
    // producer/validator derivations. Count work within each call separately.
    for _ in 0..2 {
        let (result, events) =
            capture(|| crate::engine_attestation::program::derive(&input, limits()));
        assert!(result.is_ok());
        for (target, name) in [
            ("compiler::engine_records", "analyze_relations"),
            (
                "compiler::engine_constructibility",
                "analyze_constructibility",
            ),
            ("compiler::engine_attestation", "lower_proofs"),
            ("compiler::engine_attestation", "lower_cases"),
            ("compiler::engine_attestation", "lower_requirement_tables"),
        ] {
            assert_eq!(
                events
                    .iter()
                    .filter(|event| event.is_span && event.target == target && event.name == name)
                    .count(),
                1,
                "{target}::{name}"
            );
        }
    }
}

fn limits() -> crate::placement::PlacementSearchLimits {
    crate::placement::PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("states"),
        std::num::NonZeroU64::new(1_000_000).expect("candidates"),
    )
}

#[test]
fn both_owners_refuse_proof_inputs_that_disagree_with_analyzed_stages() {
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    let keyed = super::engine_coverage_tests::bound(vec![
        KeyedOperationId::Source,
        KeyedOperationId::Merge,
    ]);
    for (control, policy) in [
        (
            core::lower(&attestation).expect("attestation"),
            attestation.policy(),
        ),
        (core::lower(&keyed).expect("keyed"), keyed.policy()),
    ] {
        assert!(core::analyze(&control, policy.proof_search_limits, limits()).is_ok());
        assert_stage_join_negatives(&control, policy);
    }
}

fn first_alternative(records: &mut core::Records) -> &mut crate::engine_proof::Alternative {
    records
        .proofs
        .obligations
        .values_mut()
        .find_map(|obligation| match obligation {
            crate::engine_proof::Obligation::Static => None,
            crate::engine_proof::Obligation::Evidence(value) => Some(value),
            crate::engine_proof::Obligation::Required(values) => values.first_mut(),
        })
        .expect("accepted owner has a proof obligation")
}

#[test]
fn both_owners_refuse_duplicate_and_unnamed_source_tables() {
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    let keyed = super::engine_coverage_tests::bound(vec![
        KeyedOperationId::Source,
        KeyedOperationId::Merge,
    ]);
    for (control, policy) in [
        (
            core::lower(&attestation).expect("attestation"),
            attestation.policy(),
        ),
        (core::lower(&keyed).expect("keyed"), keyed.policy()),
    ] {
        assert!(core::analyze(&control, policy.proof_search_limits, limits()).is_ok());
        let table = control.sources.tables[0].clone();
        let key = crate::engine_proof::AlternativeKey {
            relation: table.relation.clone(),
            namespace: table.proof_namespace,
            kind: table.proof_kind,
        };
        let mut duplicate = control.clone();
        duplicate.sources.tables.push(table.clone());
        assert_eq!(
            core::analyze(&duplicate, policy.proof_search_limits, limits()).err(),
            Some(core::Error::StageJoin(
                core::StageJoinError::DuplicateSourceTable(key)
            ))
        );
        let mut unnamed = control;
        let mut extra = table;
        extra.proof_kind = u16::MAX;
        let key = crate::engine_proof::AlternativeKey {
            relation: extra.relation.clone(),
            namespace: extra.proof_namespace,
            kind: extra.proof_kind,
        };
        unnamed.sources.tables.push(extra);
        assert_eq!(
            core::analyze(&unnamed, policy.proof_search_limits, limits()).err(),
            Some(core::Error::StageJoin(
                core::StageJoinError::UnnamedSourceTable(key)
            ))
        );
    }
}

#[test]
fn keyed_malformed_source_rows_reach_the_shared_source_refusal() {
    let input = super::engine_coverage_tests::bound(vec![KeyedOperationId::Source]);
    let mut records = core::lower(&input).expect("lowering records");
    assert!(core::analyze(&records, input.policy().proof_search_limits, limits()).is_ok());
    let missing = records.sources.tables[0]
        .rows
        .pop()
        .expect("source row")
        .operand;
    assert_eq!(
        core::analyze(&records, input.policy().proof_search_limits, limits()).err(),
        Some(core::Error::Sources(Box::new(
            crate::engine_source::SourceError::MissingOperand(missing)
        )))
    );
}

fn assert_stage_join_negatives(control: &core::Records, policy: crate::AnalysisPolicy) {
    use crate::engine_lifecycle::{LifecycleRequirementRecord, ModeKey, ObjectKey};
    use core::StageJoinError as J;
    let operation = control.relations.scope[0];
    let object = ObjectKey {
        namespace: u16::MAX,
        code: 1,
    };
    let mut subject = control.clone();
    subject.proofs.choices.push((
        crate::engine_proof::ChoiceKey { operation, object },
        Vec::new(),
    ));
    assert_eq!(
        core::analyze(&subject, policy.proof_search_limits, limits()).err(),
        Some(core::Error::StageJoin(J::RepresentationChoices))
    );
    let mut subject = control.clone();
    subject.proofs.lifecycle.push(LifecycleRequirementRecord {
        object,
        representation: ModeKey {
            namespace: u16::MAX,
            code: 1,
        },
        exit: operation,
        status: crate::lifecycle::LifecycleExitStatus::AvailableInCompilerScope,
    });
    assert_eq!(
        core::analyze(&subject, policy.proof_search_limits, limits()).err(),
        Some(core::Error::StageJoin(J::LifecycleRequirements))
    );
    let mut subject = control.clone();
    let alternative = first_alternative(&mut subject);
    alternative.constructible = !alternative.constructible;
    let id = alternative.id.clone();
    assert_eq!(
        core::analyze(&subject, policy.proof_search_limits, limits()).err(),
        Some(core::Error::StageJoin(J::Constructibility(id)))
    );
    let mut subject = control.clone();
    let alternative = first_alternative(&mut subject);
    let id = alternative.id.clone();
    // Keep the relation/table identities unchanged; alter one row's value.
    alternative.sources.as_mut().expect("accepted rows")[0]
        .availability
        .kind ^= 1;
    assert_eq!(
        core::analyze(&subject, policy.proof_search_limits, limits()).err(),
        Some(core::Error::StageJoin(J::SourceRows(id)))
    );
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

#[test]
fn source_rejected_alternative_is_local_to_orchestrated_search() {
    use crate::{engine_proof as proof, proof::LocalProofRejection};
    let input = super::engine_coverage_tests::bound(vec![KeyedOperationId::Merge]);
    let control = core::lower(&input).expect("lowering");
    let accepted = core::analyze(&control, input.policy().proof_search_limits, limits())
        .expect("accepted control");
    assert_eq!(accepted.proof_search.rejected_by_source, 0);
    for table_present in [false, true] {
        let mut subject = control.clone();
        let alternatives = subject
            .proofs
            .obligations
            .values_mut()
            .find_map(|row| {
                if let proof::Obligation::Required(alternatives) = row {
                    Some(alternatives)
                } else {
                    None
                }
            })
            .expect("runtime obligation");
        assert_eq!(alternatives.len(), 1);
        let mut rejected = alternatives[0].clone();
        let original = rejected.id.clone();
        rejected.id.kind = u16::MAX;
        rejected.sources = Err(LocalProofRejection::SourceDerivation);
        rejected.construction.clear();
        // The attestation lowerer derives false when source derivation fails.
        rejected.constructible = false;
        if table_present {
            let mut table = subject
                .sources
                .tables
                .iter()
                .find(|table| {
                    (
                        table.relation.clone(),
                        table.proof_namespace,
                        table.proof_kind,
                    ) == (original.relation.clone(), original.namespace, original.kind)
                })
                .expect("control table")
                .clone();
            table.proof_kind = rejected.id.kind;
            subject.sources.tables.push(table);
        }
        alternatives.push(rejected);
        let program = core::analyze(&subject, input.policy().proof_search_limits, limits())
            .expect("one feasible alternative remains");
        assert_eq!(program.plans, accepted.plans);
        assert_eq!(program.proof_search.rejected_by_source, 1);
    }
}

#[test]
fn all_source_rejected_alternatives_reach_the_search_refusal() {
    use crate::{engine_proof as proof, proof::LocalProofRejection};
    let input = super::engine_coverage_tests::bound(vec![KeyedOperationId::Merge]);
    let mut subject = core::lower(&input).expect("lowering");
    assert!(core::analyze(&subject, input.policy().proof_search_limits, limits()).is_ok());
    let (relation, alternatives) = subject
        .proofs
        .obligations
        .iter_mut()
        .find_map(|(key, row)| {
            if let proof::Obligation::Required(alternatives) = row {
                Some((key.clone(), alternatives))
            } else {
                None
            }
        })
        .expect("runtime obligation");
    for alternative in alternatives {
        alternative.sources = Err(LocalProofRejection::SourceDerivation);
        alternative.construction.clear();
        alternative.constructible = false;
    }
    let expected = proof::Error::NoFeasiblePlan(vec![relation]);
    assert_eq!(
        proof::enumerate(
            &subject.proofs,
            &crate::capability::CapabilityView::Unconstrained,
            input.policy().proof_search_limits,
            &mut crate::proof::ProofSearchReport::default(),
        )
        .err(),
        Some(expected.clone())
    );
    assert_eq!(
        core::analyze(&subject, input.policy().proof_search_limits, limits()).err(),
        Some(core::Error::Proof(expected))
    );
}
