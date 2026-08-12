//! Relation-indexed requirement tests (Guide-7 §7, §21 Wave 1).

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    ExternalEvidenceRequirement, FactId, ProofKind, Relation, RelationDeclaration, RelationId,
    RelationKind, RelationSubject, RepresentationMode, TransactionSide,
};

use super::bound_input;
use crate::{
    BoundCompilerInput, CompileError, OperandId,
    capability::{CapabilityView, RequiredCapability},
    case::{ExecutionCaseId, SponsorCase, execution_cases},
    constructibility::{CompilerConstructibilityAnalysis, build_constructibility_analysis},
    lifecycle::{
        CompilerLifecycleAnalysis, LifecycleExitStatus, build_lifecycle_analysis,
        proof_supports_representation,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_analysis, build_relation_graph},
    requirement::{
        ProofDisposition, RelationRequirements, RepresentationSelection,
        active_source_requirements, derive_relation_requirements, owned_representation,
        relation_objects, relation_requirements, validate_aggregate_closure,
        validate_disclosure_closure, validate_lifecycle_closure, validate_sponsor_erasure,
    },
    source::{OperandRole, RequiredSourceKind, RequirementActivation, SourceRequirement},
};

struct Fixture {
    input: BoundCompilerInput,
    relations: CompilerRelationAnalysis,
    constructibility: CompilerConstructibilityAnalysis,
    lifecycle: CompilerLifecycleAnalysis,
    candidates: Vec<ProofPlanCandidate>,
}

impl Fixture {
    fn derive(&self, candidate: &ProofPlanCandidate) -> BTreeMap<RelationId, RelationRequirements> {
        derive_relation_requirements(&self.relations, &self.constructibility, candidate)
            .expect("relation requirements derive")
    }

    fn closed(&self, candidate: &ProofPlanCandidate) -> BTreeMap<RelationId, RelationRequirements> {
        relation_requirements(
            &self.input,
            &self.relations,
            &self.constructibility,
            &self.lifecycle,
            candidate,
        )
        .expect("relation requirements close")
    }

    fn declaration(&self, relation: &RelationId) -> RelationDeclaration {
        self.relations
            .graph
            .node_weights()
            .find(|node| node.source.id == *relation)
            .expect("relation is in scope")
            .source
            .clone()
    }
}

fn fixture(operations: &[OperationId]) -> Fixture {
    let input = bound_input(operations);
    let relations = build_relation_analysis(&input).expect("relations");
    let constructibility = build_constructibility_analysis(&input).expect("constructibility");
    let lifecycle = build_lifecycle_analysis(&input, &relations).expect("lifecycle");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("plans")
        .candidates;

    Fixture {
        input,
        relations,
        constructibility,
        lifecycle,
        candidates,
    }
}

fn pilots() -> Fixture {
    fixture(&[OperationId::CompactAsh, OperationId::TransferLive])
}

/// The relation owning the operation's substrate-conservation
/// requirement, with its bundle.
fn external_bundle(
    requirements: &BTreeMap<RelationId, RelationRequirements>,
) -> (&RelationId, &RelationRequirements) {
    requirements
        .iter()
        .find(|(_, bundle)| matches!(bundle.proof, ProofDisposition::ExternalEvidence { .. }))
        .expect("a pilot declares substrate conservation")
}

// --- proof disposition (§7.3) ---

#[test]
fn every_relation_class_takes_its_own_disposition() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        let requirements = fixture.closed(candidate);

        assert_eq!(
            requirements.len(),
            fixture.relations.project().nodes.len(),
            "every in-scope relation owns exactly one bundle",
        );

        for (relation, bundle) in &requirements {
            assert_eq!(&bundle.relation, relation);

            match fixture.declaration(relation).relation {
                Relation::Representation { .. } | Relation::LifecycleExit { .. } => {
                    assert_eq!(
                        bundle.proof,
                        ProofDisposition::StaticallyValidated,
                        "{relation:?}",
                    );
                    assert!(!candidate.proofs.contains_key(relation), "{relation:?}");
                    assert!(bundle.required_capabilities.is_empty(), "{relation:?}");
                    assert!(bundle.source_requirements.is_empty(), "{relation:?}");
                }

                Relation::SubstrateConservation { .. } => {
                    let ProofDisposition::ExternalEvidence {
                        approved_proof,
                        requirement,
                    } = &bundle.proof
                    else {
                        panic!("{relation:?} must retain external evidence");
                    };

                    // The approved class is retained, and nothing here
                    // claims a runtime completion of it.
                    assert_eq!(approved_proof.relation(), relation);
                    assert_eq!(approved_proof.proof(), ProofKind::SubstrateConservation);
                    assert!(!candidate.proofs.contains_key(relation));
                    assert_eq!(
                        bundle.external_evidence,
                        BTreeSet::from([requirement.clone()]),
                    );
                }

                _ => {
                    let ProofDisposition::Selected { proof } = &bundle.proof else {
                        panic!("{relation:?} must carry its selected proof");
                    };

                    // The same one proof the candidate selected, and a
                    // realization-approved alternative of this relation.
                    assert_eq!(Some(proof), candidate.proofs.get(relation));
                    assert!(
                        fixture
                            .declaration(relation)
                            .proof_alternatives
                            .contains(proof),
                    );
                }
            }
        }
    }
}

#[test]
fn a_representation_relation_has_no_proof_variable() {
    let fixture = pilots();
    let candidate = fixture.candidates.first().expect("a feasible candidate");
    let requirements = fixture.closed(candidate);

    let representations = requirements
        .iter()
        .filter(|(relation, _)| {
            matches!(
                fixture.declaration(relation).relation,
                Relation::Representation { .. }
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(representations.len(), 2, "one per pilot");

    for (relation, bundle) in representations {
        assert_eq!(bundle.proof, ProofDisposition::StaticallyValidated);
        assert!(fixture.declaration(relation).proof_alternatives.is_empty());
    }
}

#[test]
fn a_proof_required_relation_without_a_decision_is_rejected() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let relation = candidate
        .proofs
        .keys()
        .next()
        .expect("a proof-required relation")
        .clone();

    candidate.proofs.remove(&relation);

    assert_eq!(
        derive_relation_requirements(&fixture.relations, &fixture.constructibility, &candidate),
        Err(CompileError::MissingSelectedProof { relation }),
    );
}

#[test]
fn an_unapproved_decision_is_rejected() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let (relation, approved) = candidate
        .proofs
        .iter()
        .find(|(relation, alternative)| {
            fixture
                .declaration(relation)
                .proof_alternatives
                .iter()
                .all(|other| other.proof() != ProofKind::SignerMembership)
                && alternative.proof() != ProofKind::SignerMembership
        })
        .map(|(relation, alternative)| (relation.clone(), alternative.clone()))
        .expect("a relation not approving signer membership");

    candidate.proofs.insert(
        relation.clone(),
        realization::ProofAlternativeId::new(
            approved.relation().clone(),
            ProofKind::SignerMembership,
        ),
    );

    assert_eq!(
        derive_relation_requirements(&fixture.relations, &fixture.constructibility, &candidate),
        Err(CompileError::UnapprovedSelectedProof { relation }),
    );
}

#[test]
fn a_statically_validated_relation_carrying_a_proof_is_rejected() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let relation = fixture
        .relations
        .graph
        .node_weights()
        .find(|node| matches!(node.source.relation, Relation::Representation { .. }))
        .expect("a representation relation")
        .source
        .id
        .clone();

    candidate.proofs.insert(
        relation.clone(),
        realization::ProofAlternativeId::new(relation.clone(), ProofKind::ManifestShape),
    );

    assert_eq!(
        derive_relation_requirements(&fixture.relations, &fixture.constructibility, &candidate),
        Err(CompileError::UnexpectedSelectedProof { relation }),
    );
}

// --- external evidence ownership (§7.3, T6) ---

#[test]
fn the_external_substrate_capability_is_retained_by_its_relation() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        let requirements = fixture.closed(candidate);
        let (relation, bundle) = external_bundle(&requirements);

        assert!(
            bundle
                .required_capabilities
                .contains(&RequiredCapability::WholeTransactionValueConservation),
            "{relation:?} owns the whole-transaction capability",
        );

        // And the aggregate still carries it, owned rather than free.
        assert!(
            candidate
                .required_capabilities
                .contains(&RequiredCapability::WholeTransactionValueConservation),
        );
    }
}

#[test]
fn the_external_source_row_is_retained_by_its_relation() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        let requirements = fixture.closed(candidate);
        let (relation, bundle) = external_bundle(&requirements);

        let rows = bundle
            .source_requirements
            .iter()
            .filter(|row| row.source == RequiredSourceKind::ExternalEvidence)
            .collect::<Vec<_>>();

        assert_eq!(rows.len(), 1, "{relation:?} owns one evidence source row");
        assert_eq!(rows[0].operand.relation(), relation);
        assert!(
            matches!(rows[0].operand.role(), OperandRole::ExternalEvidence { .. }),
            "the row names the evidence operand",
        );
        assert!(candidate.source_requirements.contains(rows[0]));
    }
}

// --- representation ownership (§7.5) ---

#[test]
fn live_proofs_and_selected_modes_are_compatible() {
    let fixture = pilots();
    let mut seen = BTreeSet::new();

    for candidate in &fixture.candidates {
        for bundle in fixture.closed(candidate).values() {
            let (
                ProofDisposition::Selected { proof },
                Some(RepresentationSelection { object, mode }),
            ) = (&bundle.proof, bundle.representation)
            else {
                continue;
            };

            assert!(
                proof_supports_representation(proof.proof(), mode),
                "{:?} cannot support {mode:?} of {object:?}",
                bundle.relation,
            );

            if object == ObjectId::ReceiptLive {
                seen.insert((proof.proof(), mode));
            }
        }
    }

    // Both live strategies are exercised, so the assertion above is not
    // vacuous on one representation family.
    assert!(seen.contains(&(ProofKind::PublicArithmetic, RepresentationMode::Explicit)));
    assert!(seen.contains(&(
        ProofKind::ConfidentialConservation,
        RepresentationMode::PrivateCommitted,
    )));
}

#[test]
fn each_pilots_representation_relation_owns_its_own_objects_mode() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        for (relation, bundle) in &fixture.closed(candidate) {
            let Relation::Representation { object, .. } = fixture.declaration(relation).relation
            else {
                continue;
            };

            let selection = bundle.representation.expect("a represented relation");

            assert_eq!(selection.object, object);
            assert_eq!(
                selection.mode,
                candidate.representations[&crate::lifecycle::RepresentationChoiceId {
                    operation: relation.operation(),
                    object,
                }],
            );
        }
    }
}

/// Two independently represented families in one operation.
///
/// Each pilot decides one representation today, so no derived candidate
/// can distinguish an object-specific selection from a mode-only one. A
/// second represented family is exactly the shape that makes the
/// difference observable.
#[test]
fn a_relation_naming_one_object_never_reads_another_objects_mode() {
    let fixture = fixture(&[OperationId::TransferLive]);
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let live = crate::lifecycle::RepresentationChoiceId {
        operation: OperationId::TransferLive,
        object: ObjectId::ReceiptLive,
    };
    let ash = crate::lifecycle::RepresentationChoiceId {
        operation: OperationId::TransferLive,
        object: ObjectId::Ash,
    };

    candidate
        .representations
        .insert(live, RepresentationMode::PrivateCommitted);
    candidate
        .representations
        .insert(ash, RepresentationMode::Explicit);

    let synthetic = |relation: Relation| RelationDeclaration {
        id: RelationId::new(
            OperationId::TransferLive,
            RelationKind::Representation,
            RelationSubject::Operation,
        ),
        relation,
        proof_alternatives: BTreeSet::new(),
    };

    // A relation about the ash family reads the ash mode, not the
    // receipt mode, even though a receipt mode is also fixed.
    assert_eq!(
        owned_representation(
            &synthetic(Relation::Recognition {
                side: realization::ObservedSide::Input,
                object: ObjectId::Ash,
                asset: AssetId::Lbtc,
            }),
            &candidate,
        ),
        Some(RepresentationSelection {
            object: ObjectId::Ash,
            mode: RepresentationMode::Explicit,
        }),
    );

    assert_eq!(
        owned_representation(
            &synthetic(Relation::Recognition {
                side: realization::ObservedSide::Input,
                object: ObjectId::ReceiptLive,
                asset: AssetId::Lbtc,
            }),
            &candidate,
        ),
        Some(RepresentationSelection {
            object: ObjectId::ReceiptLive,
            mode: RepresentationMode::PrivateCommitted,
        }),
    );

    // A relation naming both represented families owns no single
    // selection: one of the two modes would be an invention.
    assert_eq!(
        owned_representation(
            &synthetic(Relation::AllowedObjectFamilies {
                side: realization::ObservedSide::Input,
                allowed: BTreeSet::from([ObjectId::Ash, ObjectId::ReceiptLive]),
            }),
            &candidate,
        ),
        None,
    );
}

#[test]
fn the_object_census_of_a_relation_is_its_named_families() {
    assert_eq!(
        relation_objects(&Relation::LifecycleExit {
            object: ObjectId::ReceiptLive,
            exit: OperationId::Burn,
        }),
        BTreeSet::from([ObjectId::ReceiptLive]),
    );
    assert_eq!(
        relation_objects(&Relation::AmountConservation {
            asset: AssetId::Lbtc,
            input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            output_objects: BTreeSet::from([ObjectId::Ash]),
        }),
        BTreeSet::from([ObjectId::Ash, ObjectId::ReceiptLive]),
    );
    assert!(relation_objects(&Relation::PermissionlessAuthorization).is_empty());
    assert!(
        relation_objects(&Relation::SubstrateConservation {
            asset: AssetId::Lbtc,
        })
        .is_empty(),
    );
}

// --- aggregate closure over both pilots (§7.6) ---

#[test]
fn every_pilot_plan_closes_its_capability_source_and_evidence_aggregates() {
    for scope in [
        vec![OperationId::CompactAsh],
        vec![OperationId::TransferLive],
        vec![OperationId::CompactAsh, OperationId::TransferLive],
    ] {
        let fixture = fixture(&scope);
        assert!(!fixture.candidates.is_empty());

        for candidate in &fixture.candidates {
            let requirements = fixture.closed(candidate);

            let mut capabilities = BTreeSet::new();
            let mut sources = BTreeSet::new();
            let mut evidence = BTreeSet::new();

            for bundle in requirements.values() {
                capabilities.extend(bundle.required_capabilities.iter().copied());
                sources.extend(bundle.source_requirements.iter().cloned());
                evidence.extend(bundle.external_evidence.iter().cloned());
            }

            assert_eq!(capabilities, candidate.required_capabilities, "{scope:?}");
            assert_eq!(
                sources,
                candidate
                    .source_requirements
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                "{scope:?}",
            );
            assert_eq!(evidence, candidate.external_evidence, "{scope:?}");

            // Nothing is empty: the equalities are not trivially true.
            assert!(!capabilities.is_empty());
            assert!(!sources.is_empty());
            assert!(!evidence.is_empty());
        }
    }
}

#[test]
fn a_missing_capability_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);

    // Drop the external-evidence capability from the aggregate: the
    // regression that made relation provenance necessary.
    let (relation, _) = external_bundle(&requirements);
    let relation = relation.clone();
    candidate
        .required_capabilities
        .remove(&RequiredCapability::WholeTransactionValueConservation);

    assert_eq!(
        validate_aggregate_closure(&candidate, &requirements),
        Err(CompileError::AnalyzedCapabilityClosureMismatch {
            relation: Some(relation),
            capability: RequiredCapability::WholeTransactionValueConservation,
        }),
    );
}

#[test]
fn an_unowned_aggregate_capability_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);

    candidate
        .required_capabilities
        .insert(RequiredCapability::RefundAuthorization);

    assert_eq!(
        validate_aggregate_closure(&candidate, &requirements),
        Err(CompileError::AnalyzedCapabilityClosureMismatch {
            relation: None,
            capability: RequiredCapability::RefundAuthorization,
        }),
    );
}

#[test]
fn a_missing_source_row_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);
    let dropped = candidate.source_requirements.remove(0);

    assert_eq!(
        validate_aggregate_closure(&candidate, &requirements),
        Err(CompileError::AnalyzedSourceClosureMismatch {
            relation: Some(dropped.operand.relation().clone()),
            operand: dropped.operand,
        }),
    );
}

#[test]
fn an_unowned_aggregate_source_row_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);
    let operand = OperandId::new(
        RelationId::new(
            OperationId::TransferLive,
            RelationKind::Recognition,
            RelationSubject::Operation,
        ),
        OperandRole::ProjectionSet,
    );

    candidate.source_requirements.push(SourceRequirement {
        operand: operand.clone(),
        source: RequiredSourceKind::AuthenticatedTransitionCertificate,
        availability: realization::AvailabilityClass::Public,
        activation: RequirementActivation::Always,
    });

    assert_eq!(
        validate_aggregate_closure(&candidate, &requirements),
        Err(CompileError::AnalyzedSourceClosureMismatch {
            relation: None,
            operand,
        }),
    );
}

#[test]
fn a_missing_evidence_requirement_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);
    let (relation, bundle) = external_bundle(&requirements);
    let requirement = bundle
        .external_evidence
        .first()
        .expect("an evidence requirement")
        .clone();
    let relation = relation.clone();

    candidate.external_evidence.remove(&requirement);

    assert_eq!(
        validate_aggregate_closure(&candidate, &requirements),
        Err(CompileError::AnalyzedEvidenceClosureMismatch {
            relation: Some(relation),
            requirement,
        }),
    );
}

#[test]
fn an_unowned_aggregate_evidence_requirement_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);
    let requirement = ExternalEvidenceRequirement::SubstrateConservation {
        operation: OperationId::Clear,
        asset: AssetId::Lbtc,
    };

    candidate.external_evidence.insert(requirement.clone());

    assert_eq!(
        validate_aggregate_closure(&candidate, &requirements),
        Err(CompileError::AnalyzedEvidenceClosureMismatch {
            relation: None,
            requirement,
        }),
    );
}

// --- disclosure closure (§7.7) ---

#[test]
fn a_disclosure_analysis_contradicting_its_representations_is_rejected() {
    let fixture = pilots();
    let mut candidate = fixture
        .candidates
        .iter()
        .find(|candidate| !candidate.disclosure.added_required_public.is_empty())
        .expect("a candidate that publishes an amount")
        .clone();
    let requirements = fixture.derive(&candidate);

    candidate.disclosure.added_required_public.clear();

    assert_eq!(
        validate_disclosure_closure(&fixture.input, &candidate, &requirements),
        Err(CompileError::AnalyzedDisclosureClosureMismatch),
    );
}

#[test]
fn an_added_disclosure_reason_naming_an_unselected_mode_is_rejected() {
    let fixture = pilots();
    let mut candidate = fixture
        .candidates
        .iter()
        .find(|candidate| !candidate.disclosure.added_required_public.is_empty())
        .expect("a candidate that publishes an amount")
        .clone();
    let requirements = fixture.derive(&candidate);

    for reasons in candidate.disclosure.added_required_public.values_mut() {
        *reasons = reasons
            .iter()
            .map(|reason| match reason {
                crate::disclosure::CompilerDisclosureReason::RepresentationSelection {
                    operation,
                    object,
                    ..
                } => crate::disclosure::CompilerDisclosureReason::RepresentationSelection {
                    operation: *operation,
                    object: *object,
                    // A mode no pilot representation relation admits.
                    representation: RepresentationMode::PublicCommitted,
                },
                other @ crate::disclosure::CompilerDisclosureReason::ProofRequirement { .. } => {
                    other.clone()
                }
            })
            .collect();
    }

    assert_eq!(
        validate_disclosure_closure(&fixture.input, &candidate, &requirements),
        Err(CompileError::AnalyzedDisclosureClosureMismatch),
    );
}

#[test]
fn a_relation_requiring_a_retained_private_amount_is_rejected() {
    let fixture = fixture(&[OperationId::TransferLive]);
    let mut candidate = fixture
        .candidates
        .iter()
        .find(|candidate| {
            candidate.source_requirements.iter().any(|row| {
                row.source == RequiredSourceKind::AuthenticatedConsensusValue
                    && matches!(
                        row.operand.role(),
                        OperandRole::ObjectFamilyAmount {
                            object: ObjectId::ReceiptLive,
                            ..
                        }
                    )
            })
        })
        .expect("an explicit live candidate")
        .clone();
    let requirements = fixture.derive(&candidate);

    let relation = candidate
        .source_requirements
        .iter()
        .find(|row| {
            row.source == RequiredSourceKind::AuthenticatedConsensusValue
                && matches!(
                    row.operand.role(),
                    OperandRole::ObjectFamilyAmount {
                        object: ObjectId::ReceiptLive,
                        ..
                    }
                )
        })
        .expect("the amount row")
        .operand
        .relation()
        .clone();
    let side = TransactionSide::Input;
    let fact = FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side,
        object: ObjectId::ReceiptLive,
    };

    candidate.disclosure.retained_private.insert(fact.clone());

    // The relation reads an exact public value the analysis now keeps
    // private: a contradiction, not a privacy improvement.
    assert_eq!(
        validate_disclosure_closure(&fixture.input, &candidate, &requirements),
        Err(CompileError::AnalyzedRequirementFactUnavailable { relation, fact }),
    );
}

#[test]
fn no_pilot_disclosure_class_carries_a_sponsor_amount() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        for fact in candidate
            .disclosure
            .inherited_required_public
            .keys()
            .chain(candidate.disclosure.added_required_public.keys())
            .chain(candidate.disclosure.retained_private.iter())
        {
            assert!(!crate::disclosure::is_sponsor_amount(fact), "{fact:?}");
        }
    }
}

// --- lifecycle closure (§7.8) ---

#[test]
fn pilot_lifecycle_rows_are_owned_and_keep_their_exit_status() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        let requirements = fixture.closed(candidate);
        let owned = requirements
            .values()
            .flat_map(|bundle| bundle.lifecycle.iter().cloned())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            owned,
            candidate.lifecycle.iter().cloned().collect::<BTreeSet<_>>(),
        );
        assert!(!owned.is_empty());

        // Both scope classes survive: compact ASH is in scope, the
        // receipt's burn and redeem exits are declared outside it.
        let statuses = owned
            .iter()
            .map(|requirement| requirement.status)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            statuses,
            BTreeSet::from([
                LifecycleExitStatus::AvailableInCompilerScope,
                LifecycleExitStatus::DeclaredOutsideCompilerScope,
            ]),
        );

        let outside = owned
            .iter()
            .filter(|requirement| {
                requirement.status == LifecycleExitStatus::DeclaredOutsideCompilerScope
            })
            .map(|requirement| requirement.exit)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            outside,
            BTreeSet::from([OperationId::Burn, OperationId::Clear, OperationId::Redeem]),
        );
    }
}

#[test]
fn an_unrequired_lifecycle_row_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);
    let extra = crate::lifecycle::LifecycleRequirement {
        object: ObjectId::ReceiptLive,
        // A mode this candidate did not select, so no requirement of
        // the analysis selects this row.
        representation: RepresentationMode::PublicCommitted,
        exit: OperationId::Burn,
        status: LifecycleExitStatus::DeclaredOutsideCompilerScope,
    };

    candidate.lifecycle.push(extra.clone());

    assert_eq!(
        validate_lifecycle_closure(&fixture.lifecycle, &candidate, &requirements),
        Err(CompileError::AnalyzedLifecycleClosureMismatch {
            missing: Vec::new(),
            unexpected: vec![extra],
        }),
    );
}

#[test]
fn a_dropped_lifecycle_row_breaks_the_closure() {
    let fixture = pilots();
    let mut candidate = fixture.candidates.first().expect("a candidate").clone();
    let requirements = fixture.derive(&candidate);
    let dropped = candidate.lifecycle.remove(0);

    assert_eq!(
        validate_lifecycle_closure(&fixture.lifecycle, &candidate, &requirements),
        Err(CompileError::AnalyzedLifecycleClosureMismatch {
            missing: vec![dropped],
            unexpected: Vec::new(),
        }),
    );
}

// --- case-level active sources (§7.4) ---

#[test]
fn the_case_source_set_is_the_relations_active_subset() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        let requirements = fixture.closed(candidate);

        for case in execution_cases(&fixture.relations, candidate).expect("cases") {
            let mut active = BTreeSet::new();

            for bundle in requirements.values() {
                let rows = active_source_requirements(bundle, &case.id);

                // Every active row is one of the relation's complete
                // rows: activation filters, it never adds.
                for row in &rows {
                    assert!(bundle.source_requirements.contains(row));
                    assert_eq!(row.operand.relation(), &bundle.relation);
                }

                active.extend(rows);
            }

            assert_eq!(
                active,
                case.active_sources.iter().cloned().collect::<BTreeSet<_>>(),
            );
        }
    }
}

#[test]
fn a_sponsor_local_row_is_active_only_where_the_sponsor_region_exists() {
    let fixture = fixture(&[OperationId::CompactAsh]);
    let candidate = fixture.candidates.first().expect("a candidate");
    let requirements = fixture.closed(candidate);
    let bundle = requirements
        .values()
        .find(|bundle| {
            bundle
                .source_requirements
                .iter()
                .any(|row| row.activation == RequirementActivation::WhenSponsorPresent)
        })
        .expect("a sponsor-conditional relation");

    let case = |sponsor| ExecutionCaseId {
        operation: OperationId::CompactAsh,
        sponsor,
        representations: BTreeMap::from([(ObjectId::Ash, RepresentationMode::Explicit)]),
    };

    let absent = active_source_requirements(bundle, &case(SponsorCase::Absent));
    let present = active_source_requirements(bundle, &case(SponsorCase::Present));

    assert!(absent.is_subset(&present));
    assert_ne!(absent, present);

    // A case of another operation selects nothing from this relation.
    let mut foreign = case(SponsorCase::Present);
    foreign.operation = OperationId::TransferLive;
    assert!(active_source_requirements(bundle, &foreign).is_empty());
}

// --- sponsor erasure (§7.4) ---

#[test]
fn a_sponsor_amount_row_is_rejected() {
    let fixture = pilots();
    let candidate = fixture.candidates.first().expect("a candidate");
    let mut bundle = fixture
        .derive(candidate)
        .into_values()
        .next()
        .expect("a bundle");

    assert_eq!(validate_sponsor_erasure(&bundle), Ok(()));

    bundle.source_requirements.insert(SourceRequirement {
        operand: OperandId::new(
            bundle.relation.clone(),
            OperandRole::ObjectFamilyAmount {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        ),
        source: RequiredSourceKind::AuthenticatedConsensusValue,
        availability: realization::AvailabilityClass::Public,
        activation: RequirementActivation::WhenSponsorPresent,
    });

    assert_eq!(
        validate_sponsor_erasure(&bundle),
        Err(CompileError::SponsorValueRead),
    );
}

#[test]
fn no_pilot_requirement_reads_a_sponsor_amount() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        for bundle in fixture.closed(candidate).values() {
            assert_eq!(validate_sponsor_erasure(bundle), Ok(()));

            for row in &bundle.source_requirements {
                assert!(!crate::source::is_sponsor_amount_operand(
                    row.operand.role()
                ));
            }
        }
    }
}

// --- determinism (§12.5) ---

#[test]
fn repeated_derivation_is_equal() {
    let fixture = pilots();

    for candidate in &fixture.candidates {
        assert_eq!(fixture.closed(candidate), fixture.closed(candidate));
    }
}

#[test]
fn a_permuted_relation_source_yields_equal_requirements() {
    let fixture = pilots();
    let source = fixture.input.realization().project();
    let mut nodes = source.relations.nodes;
    let mut edges = source.relations.edges;

    nodes.reverse();
    edges.reverse();

    let permuted = build_relation_graph(fixture.input.scope().operations(), &nodes, &edges)
        .expect("permuted relations");

    for candidate in &fixture.candidates {
        assert_eq!(
            derive_relation_requirements(&permuted, &fixture.constructibility, candidate)
                .expect("permuted requirements"),
            fixture.derive(candidate),
        );
    }
}
