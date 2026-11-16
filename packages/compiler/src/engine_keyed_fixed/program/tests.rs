use super::*;
use crate::keyed_fixed_plan::{Defect, Stage};
use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId as K};

pub(super) fn bound(scope: &[K]) -> BoundCompilerInput<KeyedFixedOwner> {
    let manifest = KeyedFixedManifest::contract().expect("manifest");
    let source = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("validation");
    crate::bind_input(
        &manifest,
        source,
        scope.to_vec(),
        crate::AnalysisPolicy::strict(crate::ProofSearchLimits::new(
            std::num::NonZeroU64::new(1_000_000).unwrap(),
            std::num::NonZeroU64::new(100_000).unwrap(),
        )),
    )
    .expect("bound")
}
pub(super) fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).unwrap(),
        std::num::NonZeroU64::new(100_000).unwrap(),
    )
}
#[test]
fn production_keyed_publication_equals_record_oracle_and_revalidates() {
    for scope in [vec![K::Source], vec![K::Merge], vec![K::Source, K::Merge]] {
        let input = bound(&scope);
        let prepared = Prepared::new(&input).expect("preparation");
        let oracle =
            crate::analyzed::analyze_bound_records(&input, limits()).expect("record oracle");
        assert_eq!(
            prepared.analyze(limits()).expect("prepared orchestration"),
            oracle
        );
        let plan = crate::keyed_fixed_plan::plan(&input, limits()).expect("production");
        assert_eq!(
            plan.project(),
            &prepared.publish(&oracle).expect("oracle publication")
        );
        assert!(crate::keyed_fixed_plan::revalidate(&input, limits(), &plan).is_ok());
        let mut missing = plan;
        missing.projection.evidence.pop_first();
        assert_eq!(
            crate::keyed_fixed_plan::revalidate(&input, limits(), &missing),
            Err(KeyedFixedPlanError::PublicationMismatch)
        );
    }
}
fn alternative(records: &mut core::Records) -> &mut crate::engine_proof::Alternative {
    records
        .proofs
        .obligations
        .values_mut()
        .find_map(|row| match row {
            crate::engine_proof::Obligation::Static => None,
            crate::engine_proof::Obligation::Evidence(value) => Some(value),
            crate::engine_proof::Obligation::Required(values) => values.first_mut(),
        })
        .expect("alternative")
}
#[test]
fn production_keyed_source_and_join_refusals_are_owner_typed() {
    let input = bound(&[K::Source, K::Merge]);
    let control = Prepared::new(&input).expect("control");
    assert!(control.analyze(limits()).is_ok());
    for (mutate, defect) in [
        (
            (|r: &mut core::Records| {
                r.sources.tables[0].rows.clear();
            }) as fn(&mut core::Records),
            Defect::MissingOperand,
        ),
        (
            |r| {
                r.sources.tables.push(r.sources.tables[0].clone());
            },
            Defect::DuplicateSourceTable,
        ),
        (
            |r| {
                let mut extra = r.sources.tables[0].clone();
                extra.proof_kind = u16::MAX;
                r.sources.tables.push(extra);
            },
            Defect::UnnamedSourceTable,
        ),
        (
            |r| {
                alternative(r).constructible = false;
            },
            Defect::Constructibility,
        ),
        (
            |r| {
                alternative(r).sources.as_mut().unwrap()[0]
                    .availability
                    .kind ^= 1;
            },
            Defect::SourceRows,
        ),
        (
            |r| {
                alternative(r).id.namespace = 1;
            },
            Defect::ProofSource,
        ),
    ] {
        let mut subject = Prepared::new(&input).expect("fresh");
        mutate(&mut subject.records);
        let key = subject.records.sources.tables[0].relation.clone();
        let relation = subject.relation(&key).unwrap();
        assert_eq!(
            subject.analyze(limits()).err(),
            Some(KeyedFixedPlanError::Refused {
                stage: if defect == Defect::MissingOperand {
                    Stage::Sources
                } else {
                    Stage::Join
                },
                defect,
                relations: vec![relation],
                operations: vec![relation.operation],
                unrestored_key_count: 0,
            })
        );
    }
}
#[test]
fn production_keyed_unknown_and_cyclic_dependencies_refuse() {
    use crate::engine_records::{DependencyKind, DependencyRecord};
    let input = bound(&[K::Source]);
    let mut prepared = Prepared::new(&input).expect("control");
    assert!(prepared.analyze(limits()).is_ok());
    let first = prepared.records.relations.nodes[0].id.clone();
    let second = prepared.records.relations.nodes[1].id.clone();
    prepared.records.relations.edges.push(DependencyRecord {
        source: first.clone(),
        target: second.clone(),
        kind: DependencyKind {
            namespace: 3,
            code: 1,
        },
    });
    assert!(prepared.analyze(limits()).is_ok());
    prepared.records.relations.edges.push(DependencyRecord {
        source: second,
        target: first,
        kind: DependencyKind {
            namespace: 3,
            code: 1,
        },
    });
    assert!(matches!(
        prepared.analyze(limits()),
        Err(KeyedFixedPlanError::Refused {
            defect: Defect::Cycle,
            ..
        })
    ));
    prepared.records.relations.edges.pop();
    prepared.records.relations.edges[0]
        .target
        .subject
        .push(u16::MAX);
    assert_eq!(
        prepared.analyze(limits()).err(),
        Some(KeyedFixedPlanError::Refused {
            stage: Stage::Relations,
            defect: Defect::UnknownEndpoint,
            relations: Vec::new(),
            operations: Vec::new(),
            unrestored_key_count: 1,
        })
    );
}
#[test]
fn production_keyed_local_rejection_remains_local() {
    use crate::{engine_proof as proof, proof::LocalProofRejection};
    let input = bound(&[K::Merge]);
    let mut prepared = Prepared::new(&input).expect("control");
    let control = prepared.analyze(limits()).expect("accepted");
    let (relation, alternatives) = prepared
        .records
        .proofs
        .obligations
        .iter_mut()
        .find_map(|(key, row)| {
            if let proof::Obligation::Required(values) = row {
                Some((key.clone(), values))
            } else {
                None
            }
        })
        .expect("runtime");
    let mut rejected = alternatives[0].clone();
    rejected.id.kind = u16::MAX;
    rejected.sources = Err(LocalProofRejection::SourceDerivation);
    rejected.constructible = false;
    alternatives.push(rejected);
    let result = prepared.analyze(limits()).expect("feasible sibling");
    assert_eq!(result.plans, control.plans);
    assert_eq!(result.proof_search.rejected_by_source, 1);
    if let proof::Obligation::Required(values) = prepared
        .records
        .proofs
        .obligations
        .get_mut(&relation)
        .unwrap()
    {
        values[0].sources = Err(LocalProofRejection::SourceDerivation);
        values[0].constructible = false;
    }
    assert_eq!(
        prepared.analyze(limits()).err(),
        Some(KeyedFixedPlanError::NoFeasiblePlan(vec![
            prepared.relation(&relation).unwrap()
        ]))
    );
}
#[test]
fn production_keyed_missing_reverse_identity_is_class_three() {
    let input = bound(&[K::Merge]);
    let mut prepared = Prepared::new(&input).unwrap();
    let program = prepared.analyze(limits()).unwrap();
    assert!(prepared.publish(&program).is_ok());
    prepared.identities.carriers.clear();
    assert_eq!(prepared.publish(&program), Err(missing()));
}

#[test]
fn production_keyed_binding_and_limits_are_revalidated_from_retained_source() {
    let source = bound(&[K::Source]);
    let merge = bound(&[K::Merge]);
    let plan = crate::keyed_fixed_plan::plan(&source, limits()).unwrap();
    assert_eq!(
        crate::keyed_fixed_plan::revalidate(&merge, limits(), &plan),
        Err(KeyedFixedPlanError::PublicationMismatch)
    );
    let other_limits = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(2_000_000).unwrap(),
        std::num::NonZeroU64::new(200_000).unwrap(),
    );
    assert_eq!(
        crate::keyed_fixed_plan::revalidate(&source, other_limits, &plan),
        Err(KeyedFixedPlanError::PublicationMismatch)
    );
    let mut subject = plan;
    subject.projection.binding.schema_version ^= 1;
    assert_eq!(
        crate::keyed_fixed_plan::revalidate(&source, limits(), &subject),
        Err(KeyedFixedPlanError::PublicationMismatch)
    );
    let tight = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1).unwrap(),
        std::num::NonZeroU64::new(1).unwrap(),
    );
    assert_eq!(
        crate::keyed_fixed_plan::plan(&merge, tight).err(),
        Some(KeyedFixedPlanError::SearchLimit {
            stage: Stage::Placement,
            candidates: false,
            limit: 1
        })
    );
}

#[test]
fn production_keyed_fixed_source_refusal_restores_its_relation() {
    let input = bound(&[K::Source]);
    let mut subject = Prepared::new(&input).unwrap();
    assert!(subject.analyze(limits()).is_ok());
    let alternative = alternative(&mut subject.records);
    let key = alternative.id.relation.clone();
    alternative.sources = Err(crate::proof::LocalProofRejection::SourceDerivation);
    alternative.constructible = false;
    assert_eq!(
        subject.analyze(limits()).err(),
        Some(KeyedFixedPlanError::FixedSource(
            subject.relation(&key).unwrap()
        ))
    );
}

#[test]
fn keyed_fixed_case_records_refuse_a_missing_coverage_source_without_indexing() {
    let input = bound(&[K::Source]);
    let mut source = super::super::lower_sources(input.owner());
    assert!(super::super::case_records(input.owner(), &source).is_ok());
    let missing = source.tables.remove(0).relation;
    assert!(matches!(super::super::case_records(input.owner(), &source),
        Err(super::super::KeyedFixedLoweringError::MissingSourceTable(key)) if key == missing));
}

#[test]
fn keyed_evidence_refuses_foreign_vocabulary_with_its_named_defect() {
    use crate::engine_source::Evidence;
    let input = bound(&[K::Source]);
    let prepared = Prepared::new(&input).unwrap();
    let control = prepared.evidence.keys().next().unwrap();
    assert!(super::premises(control).is_ok());
    let operation = prepared.records.relations.scope[0];
    for offered in [
        Evidence::Operator { operation },
        Evidence::ConfidentialValue {
            operation,
            asset: 0,
        },
        Evidence::SubstrateValue {
            operation,
            asset: 0,
        },
    ] {
        assert_eq!(
            super::premises(&offered),
            Err(super::fact_refusal(Defect::ForeignEvidence))
        );
    }
    for side in [
        realization::TransactionSide::Input,
        realization::TransactionSide::Output,
    ] {
        assert_eq!(
            super::decode_side(crate::engine_identity::side_code(side)),
            Ok(side)
        );
    }
    assert_eq!(
        super::decode_side(u16::MAX),
        Err(super::fact_refusal(Defect::UnsupportedEvidenceSide))
    );
}

// Fixed-form-specific controls, following the mirrored first-edition battery.
#[test]
fn fixed_form_clauses_join_both_owners_and_refuse_each_omission() {
    use realization::keyed_fixed_relation::KeyedFixedRelation as R;
    let input = bound(&[K::Source, K::Merge]);
    let manifest = input.owner().manifest();
    let declarations = input.owner().source().project();
    assert_eq!(super::super::check_clauses(manifest, &declarations), Ok(()));
    for clause in [R::BornFirst, R::OwnQualification, R::SuccessorQualification] {
        let mut subject = declarations.clone();
        subject.retain(|row| row.relation != clause);
        assert_eq!(
            super::super::check_clauses(manifest, &subject),
            Err(super::super::KeyedFixedLoweringError::ClauseCensus)
        );
    }
    for index in 0..manifest.clauses.len() {
        let mut subject = manifest.clone();
        subject.clauses.remove(index);
        assert_eq!(
            super::super::check_clauses(&subject, &declarations),
            Err(super::super::KeyedFixedLoweringError::ClauseCensus)
        );
    }
    assert_eq!(
        super::refusal::lowering(
            input.owner(),
            super::super::KeyedFixedLoweringError::ClauseCensus
        ),
        super::fact_refusal(Defect::CensusMismatch)
    );
}

#[test]
fn fixed_form_lowering_reaches_the_clause_join() {
    use crate::engine_keyed_fixed::KeyedFixedLoweringError::ClauseCensus;
    use crate::engine_records::RelationOwner;
    let input = bound(&[K::Source, K::Merge]);
    assert!(input.owner().lower_relations().is_ok());
    assert!(input.owner().lower_sources().is_ok());
    for index in 0..input.owner().manifest().clauses.len() {
        let mut manifest = input.owner().manifest().clone();
        manifest.clauses.remove(index);
        let subject = KeyedFixedOwner::new(
            manifest,
            input.owner().source().clone(),
            input.owner().scope().to_vec(),
        );
        assert_eq!(subject.lower_relations().err(), Some(ClauseCensus));
        assert_eq!(subject.lower_sources().err(), Some(ClauseCensus));
    }
}

#[test]
fn qualification_sources_retain_the_constant_triple_and_authenticated_objects() {
    use crate::engine_source::Payload;
    use crate::source::RequiredSourceKind as S;
    use realization::keyed_fixed_relation::KeyedFixedRelationKind as Kind;
    let input = bound(&[K::Merge]);
    let prepared = Prepared::new(&input).unwrap();
    assert!(prepared.analyze(limits()).is_ok());
    for (kind, count) in [
        (Kind::BornFirst, 1),
        (Kind::OwnQualification, 4),
        (Kind::SuccessorQualification, 5),
    ] {
        let table = prepared
            .records
            .sources
            .tables
            .iter()
            .find(|table| prepared.relation(&table.relation).unwrap().kind == kind)
            .unwrap();
        assert_eq!(table.rows.len(), count);
        assert_eq!(table.rows[0].source, S::AuthenticatedInputObject);
        let constants = table
            .rows
            .iter()
            .filter_map(|row| {
                if let Payload::Bound { namespace, code } = row.operand.payload {
                    assert_eq!(row.source, S::PublicConstructionData);
                    Some((namespace, code, row.operand.role))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            constants,
            if kind == Kind::BornFirst {
                Vec::new()
            } else {
                vec![(3, 1, 2), (3, 2, 3), (3, 3, 4)]
            }
        );
        assert_eq!(
            table
                .rows
                .iter()
                .filter(|row| row.source == S::AuthenticatedOutputObject)
                .count(),
            usize::from(kind == Kind::SuccessorQualification)
        );
        for index in 0..count {
            let mut subject = Prepared::new(&input).unwrap();
            subject
                .records
                .sources
                .tables
                .iter_mut()
                .find(|row| row.relation == table.relation)
                .unwrap()
                .rows
                .remove(index);
            assert_eq!(
                subject.analyze(limits()).err(),
                Some(KeyedFixedPlanError::Refused {
                    stage: Stage::Sources,
                    defect: Defect::MissingOperand,
                    relations: vec![prepared.relation(&table.relation).unwrap()],
                    operations: vec![K::Merge],
                    unrestored_key_count: 0,
                })
            );
        }
    }
}

#[test]
fn new_owner_registry_and_namespace_are_disjoint_from_both_earlier_owners() {
    use crate::engine_records::{LoweredRelations, RelationOwner};
    use architecture::semantic::{KeyedOperationId as Old, SemanticOperationId as Op};
    let input = bound(&[K::Source, K::Merge]);
    let lowered = input.owner().lower_relations().unwrap();
    assert_eq!(lowered.records().nodes.len(), 10);
    let encodings = lowered
        .records()
        .nodes
        .iter()
        .map(|row| {
            assert_eq!(row.id.subject.len(), 2);
            [
                row.id.operation.namespace,
                row.id.operation.code,
                row.id.subject[0],
                row.id.subject[1],
                row.id.kind,
            ]
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        encodings,
        std::collections::BTreeSet::from([
            [3, 1, 3, 1, 1],
            [3, 1, 3, 1, 2],
            [3, 1, 3, 1, 6],
            [3, 1, 3, 1, 7],
            [3, 2, 3, 1, 1],
            [3, 2, 3, 1, 2],
            [3, 2, 3, 1, 3],
            [3, 2, 3, 1, 4],
            [3, 2, 3, 1, 5],
            [3, 2, 3, 1, 7],
        ])
    );
    let coverage = super::super::lower_coverage(input.owner());
    assert_eq!(coverage.len(), 10);
    for op in [K::Source, K::Merge] {
        let operation = super::super::operation_key(op);
        let key = super::super::keyed_fixed_case(operation);
        assert_eq!(lowered.identities.case(&key), Ok(op));
        let old_op = if op == K::Source {
            Old::Source
        } else {
            Old::Merge
        };
        let old =
            crate::engine_keyed::keyed_case(crate::engine_keyed::operation_key(Op::Keyed(old_op)));
        assert_ne!(key, old);
        assert_eq!(
            lowered.identities.case(&old),
            Err(crate::CompileError::MissingOwnerIdentity)
        );
        for profile in [
            architecture::semantic::SemanticProfileId::Attestation,
            architecture::semantic::SemanticProfileId::KeyedProvenance,
        ] {
            let collision = crate::engine_identity::CaseKey::new(
                profile,
                vec![
                    crate::engine_identity::word(3u16),
                    crate::engine_identity::word(operation.code),
                ],
            );
            assert_ne!(key, collision);
            let object = crate::engine_lifecycle::ObjectKey {
                namespace: 3,
                code: 1,
            };
            let carrier = super::super::keyed_fixed_carrier(operation, object);
            let collision = crate::engine_identity::CarrierKey::new(
                profile,
                vec![
                    crate::engine_identity::word(3u16),
                    crate::engine_identity::word(operation.code),
                    crate::engine_identity::word(3u16),
                    crate::engine_identity::word(1u16),
                ],
            );
            assert_ne!(carrier, collision);
            assert_eq!(
                lowered.identities.carrier(&collision),
                Err(crate::CompileError::MissingOwnerIdentity)
            );
            let purpose =
                super::super::keyed_fixed_purpose(&super::super::KeyedFixedPurpose::ClauseAccepted);
            let collision = crate::engine_identity::PurposeKey::new(
                profile,
                vec![crate::engine_identity::word(2u16)],
            );
            assert_ne!(purpose, collision);
            assert_eq!(
                lowered.identities.purpose(&collision),
                Err(crate::CompileError::MissingOwnerIdentity)
            );
        }
    }
}

#[test]
fn binding_types_are_distinct_in_both_directions() {
    trait Distinct<A> {
        fn check() {}
    }
    impl<T> Distinct<()> for T {}
    impl Distinct<u8> for <KeyedFixedManifest as crate::input::InputBinding>::Realization {}
    trait Earlier<A> {
        fn check() {}
    }
    impl<T> Earlier<()> for T {}
    impl Earlier<u8>
        for <architecture::keyed::KeyedManifest as crate::input::InputBinding>::Realization
    {
    }
    impl Earlier<u16> for <architecture::Architecture as crate::input::InputBinding>::Realization {}
    <realization::keyed::KeyedScopedRealization as Distinct<_>>::check();
    <realization::ScopedRealizationSpec as Distinct<_>>::check();
    <realization::keyed_fixed::KeyedFixedScopedRealization as Earlier<_>>::check();
}

#[test]
fn fixed_form_binding_refuses_empty_duplicate_and_foreign_manifest() {
    let input = bound(&[K::Source]);
    let manifest = input.owner().manifest();
    let source = input.owner().source();
    assert!(crate::bind_input(manifest, source.clone(), vec![K::Source], input.policy()).is_ok());
    assert_eq!(
        crate::bind_input(manifest, source.clone(), Vec::new(), input.policy()).err(),
        Some(crate::input::KeyedFixedBindingError::EmptyScope)
    );
    assert_eq!(
        crate::bind_input(
            manifest,
            source.clone(),
            vec![K::Source, K::Source],
            input.policy()
        )
        .err(),
        Some(crate::input::KeyedFixedBindingError::DuplicateOperation(
            K::Source
        ))
    );
    let mut other = manifest.clone();
    other.schema_version = 2;
    assert_eq!(
        crate::bind_input(&other, source.clone(), vec![K::Source], input.policy()).err(),
        Some(crate::input::KeyedFixedBindingError::InvalidRealization(
            realization::keyed_fixed::KeyedFixedRealizationError::InvalidManifest(
                architecture::keyed_fixed::KeyedFixedManifestError::SchemaVersion
            )
        ))
    );
}

#[test]
fn keyed_lowering_reads_every_declaration_shell_field() {
    use crate::engine_keyed_fixed::{KeyedFixedLoweringError, ShellField as F, read_shells};
    let manifest = KeyedFixedManifest::contract().expect("manifest");
    let control = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .expect("declarations");
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
    let reference = crate::tests::bound_input(&[architecture::OperationId::CompactAsh])
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
            Some(KeyedFixedLoweringError::UnsupportedShell {
                operation: scope[0],
                field,
                count: 1
            })
        );
    }
}
