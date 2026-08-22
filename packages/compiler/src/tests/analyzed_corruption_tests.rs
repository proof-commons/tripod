//! Corruption tests for the complete assembly validator (Guide-7 §15).
//!
//! Every test here takes the real assembled pilot program, applies one
//! focused mutation, and requires a typed rejection. The mutations are
//! applied to the assembled value directly rather than to a synthetic
//! stand-in: a validator that only rejects hand-built nonsense would say
//! nothing about a value the assembler could actually produce.
//!
//! The expectations are derived once per fixture and shared. Nothing a
//! mutation does can reach them — they are derived from the bound input
//! and never from the program under test — so sharing them weakens no
//! check while keeping each mutation's cost the cost of one comparison
//! rather than one whole re-analysis.
//!
//! Where a mutation is unconstructible under the analyzed program's own
//! storage, the nearest constructible corruption is applied instead and
//! the reason is stated on the test. A set keyed by a complete typed
//! value cannot hold that value twice, so "duplicate X" is not a state
//! this type system admits; the defect it stands for — one obligation
//! counted twice — is reachable only as a disagreement between a key and
//! the value stored under it, and that is what those tests assert.

use std::{collections::BTreeSet, sync::LazyLock};

use architecture::{ObjectId, OperationId};
use realization::{
    AvailabilityClass, ProofAlternativeId, ProofKind, RelationId, RepresentationMode,
    TransactionSide,
};

use super::bound_input;
use crate::{
    BoundCompilerInput, CompileError, OperandId,
    analyzed::{
        AnalyzedProofPlan, ArchitectureScopeStatus, LifecycleCompleteness, ScopedAnalyzedProgram,
        analyze_scoped_program,
    },
    analyzed_operation::AnalyzedOperation,
    analyzed_validate::{
        AnalyzedCoverageDefect, AnalyzedExecutionReportDefect, AnalyzedExpectations,
        AnalyzedFoundationDefect, AnalyzedRelationCaseDefect, AnalyzedSourceDefect,
        derive_expectations, validate_against_expectations,
    },
    capability::RequiredCapability,
    case::{ExecutionCaseId, SponsorCase},
    coverage::{
        CollateralPolicy, CoverageBoundary, CoveragePurpose, CoverageRequirementId, EvidenceRole,
        NegativeCoverageRequirement, PositiveCoverageRequirement, RelationCoverageProjection,
        RelationMutation,
    },
    coverage_graph::{CoverageDependency, CoverageEdge},
    input::CompilationScope,
    layout::LayoutRequirement,
    lifecycle::{LifecycleExitStatus, LifecycleRequirement},
    placement::{PlacedCarrier, PlacementSearchLimits, RelationActivity, RelationCaseKey},
    proof::ProofPlanCandidate,
    requirement::{ProofDisposition, RelationRequirements},
    source::{OperandRole, RequiredSourceKind, RequirementActivation},
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// One assembled pilot program with the independent expectations it is
/// checked against.
struct Fixture {
    input: BoundCompilerInput,
    expectations: AnalyzedExpectations,
    program: ScopedAnalyzedProgram,
}

fn fixture(operations: &[OperationId]) -> Fixture {
    let input = bound_input(operations);
    let program = analyze_scoped_program(&input, limits()).expect("scoped analyzed program");
    let expectations = derive_expectations(&input, limits()).expect("expectations");

    Fixture {
        input,
        expectations,
        program,
    }
}

static COMPACT_ASH: LazyLock<Fixture> = LazyLock::new(|| fixture(&[OperationId::CompactAsh]));
static COMBINED: LazyLock<Fixture> =
    LazyLock::new(|| fixture(&[OperationId::CompactAsh, OperationId::TransferLive]));

impl Fixture {
    fn validate(&self, program: &ScopedAnalyzedProgram) -> Result<(), CompileError> {
        validate_against_expectations(&self.input, &self.expectations, program)
    }

    /// Apply one mutation to a clone of the assembled program and
    /// require a typed rejection.
    fn corrupt(&self, mutate: impl FnOnce(&mut ScopedAnalyzedProgram)) -> CompileError {
        let mut program = self.program.clone();

        mutate(&mut program);
        self.validate(&program)
            .expect_err("the corrupted program is rejected")
    }
}

// --- accessors into the assembled value ---

fn first_plan(program: &ScopedAnalyzedProgram) -> ProofPlanCandidate {
    program
        .proof_plans
        .keys()
        .next()
        .expect("an analyzed plan")
        .clone()
}

fn plan_mut(program: &mut ScopedAnalyzedProgram) -> &mut AnalyzedProofPlan {
    let key = first_plan(program);

    program
        .proof_plans
        .get_mut(&key)
        .expect("the analyzed plan")
}

fn factor_mut(
    program: &mut ScopedAnalyzedProgram,
    operation: OperationId,
) -> &mut AnalyzedOperation {
    plan_mut(program)
        .operations
        .get_mut(&operation)
        .expect("the operation factor")
}

fn ash_factor_mut(program: &mut ScopedAnalyzedProgram) -> &mut AnalyzedOperation {
    factor_mut(program, OperationId::CompactAsh)
}

/// The first relation of the first plan whose bundle satisfies a
/// predicate.
fn relation_where(
    program: &ScopedAnalyzedProgram,
    predicate: impl Fn(&RelationRequirements) -> bool,
) -> RelationId {
    let key = first_plan(program);

    program.proof_plans[&key]
        .relation_requirements
        .iter()
        .find(|(_, bundle)| predicate(bundle))
        .map(|(relation, _)| relation.clone())
        .expect("a relation matching the predicate")
}

/// The first relation-case key of one factor whose bundle satisfies a
/// predicate.
fn relation_case_where(
    program: &ScopedAnalyzedProgram,
    operation: OperationId,
    predicate: impl Fn(&RelationCoverageProjection) -> bool,
) -> RelationCaseKey {
    let key = first_plan(program);

    program.proof_plans[&key].operations[&operation]
        .coverage
        .requirements
        .iter()
        .find(|(_, projection)| predicate(projection))
        .map(|(key, _)| key.clone())
        .expect("a relation-case matching the predicate")
}

/// A source row naming the erased sponsor family's amount.
fn sponsor_amount_row(relation: &RelationId) -> crate::source::SourceRequirement {
    crate::source::SourceRequirement {
        operand: OperandId::new(
            relation.clone(),
            OperandRole::ObjectFamilyAmount {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        ),
        source: RequiredSourceKind::AuthenticatedConsensusValue,
        availability: AvailabilityClass::Public,
        activation: RequirementActivation::Always,
    }
}

// --- the assembled pilot programs validate ---

#[test]
fn the_assembled_pilot_program_validates() {
    let fixture = &COMPACT_ASH;

    fixture
        .validate(&fixture.program)
        .expect("the assembled single-operation program validates");
}

#[test]
fn the_assembled_combined_program_validates() {
    let fixture = &COMBINED;

    fixture
        .validate(&fixture.program)
        .expect("the assembled two-operation program validates");
}

#[test]
fn the_construction_path_itself_runs_the_complete_validator() {
    // SR3-03: step 12 of the assembler used to run only the narrow
    // assembly-closure check, so the production entry point did not
    // perform the corruption-resistant validation this package
    // documents. The assembler's own validation is not directly
    // observable, so the property is pinned from the other side: every
    // expectation the complete validator derives is derived from the
    // same input the assembler was given, and the assembled program
    // satisfies all of them rather than only the closure subset.
    let fixture = &COMPACT_ASH;

    crate::analyzed::validate_assembly_closure(&fixture.input, &fixture.program)
        .expect("the narrow closure check passes");
    crate::analyzed_validate::validate_scoped_analyzed_program(
        &fixture.input,
        limits(),
        &fixture.program,
    )
    .expect("the complete validator passes");

    // And the complete validator is strictly stronger: a program the
    // narrow check accepts can still fail it.
    let mut corrupted = fixture.program.clone();
    corrupted.execution_report.proof_search.states_visited += 1;

    crate::analyzed::validate_assembly_closure(&fixture.input, &corrupted)
        .expect("the narrow check does not read the execution report");
    crate::analyzed_validate::validate_scoped_analyzed_program(
        &fixture.input,
        limits(),
        &corrupted,
    )
    .expect_err("the complete validator does");
}

// --- §15.1 source and scope (7) ---

#[test]
fn a_changed_architecture_binding_is_rejected() {
    // A binding built from an architecture whose envelope names another
    // realization version: a different identity for the same relations.
    let mut architecture = architecture::ARCHITECTURE;
    architecture.document.realization_version = "9.0.0";

    let binding =
        realization::ArchitectureBinding::from_architecture(&architecture).expect("a binding");

    let error = COMPACT_ASH.corrupt(|program| program.source.architecture = binding);

    assert_eq!(
        error,
        CompileError::AnalyzedSourceBindingMismatch {
            defect: AnalyzedSourceDefect::ArchitectureBinding,
        },
    );
}

#[test]
fn a_changed_realization_projection_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        program.source.realization.relations.nodes.pop();
    });

    assert_eq!(
        error,
        CompileError::AnalyzedSourceBindingMismatch {
            defect: AnalyzedSourceDefect::RealizationProjection,
        },
    );
}

#[test]
fn a_dropped_scope_operation_is_rejected() {
    let error = COMBINED.corrupt(|program| {
        program.source.compilation_scope =
            CompilationScope::from_operations([OperationId::CompactAsh]).expect("scope");
    });

    assert_eq!(
        error,
        CompileError::AnalyzedSourceBindingMismatch {
            defect: AnalyzedSourceDefect::CompilationScope,
        },
    );
}

#[test]
fn an_added_scope_operation_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        program.source.compilation_scope =
            CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
                .expect("scope");
    });

    assert_eq!(
        error,
        CompileError::AnalyzedSourceBindingMismatch {
            defect: AnalyzedSourceDefect::CompilationScope,
        },
    );
}

#[test]
fn a_claimed_complete_architecture_scope_is_rejected() {
    let error = COMPACT_ASH
        .corrupt(|program| program.architecture_scope = ArchitectureScopeStatus::Complete);

    let CompileError::AnalyzedArchitectureScopeStatusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("a claimed complete architecture scope is not reported as a status defect");
    };

    assert_eq!(missing.len(), 12);
    assert_eq!(unexpected, [] as [OperationId; 0]);
}

#[test]
fn an_omitted_missing_architecture_operation_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let ArchitectureScopeStatus::Partial { missing } = &mut program.architecture_scope else {
            panic!("the pilot scope is not partial");
        };
        let dropped = *missing.iter().next().expect("a missing operation");

        missing.remove(&dropped);
    });

    let CompileError::AnalyzedArchitectureScopeStatusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("an omitted architecture operation is not reported as a status defect");
    };

    assert_eq!(missing.len(), 1);
    assert_eq!(unexpected, [] as [OperationId; 0]);
}

#[test]
fn an_analyzed_operation_named_missing_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let ArchitectureScopeStatus::Partial { missing } = &mut program.architecture_scope else {
            panic!("the pilot scope is not partial");
        };

        missing.insert(OperationId::CompactAsh);
    });

    let CompileError::AnalyzedArchitectureScopeStatusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("an analyzed operation named missing is not reported as a status defect");
    };

    assert_eq!(missing, [] as [OperationId; 0]);
    assert_eq!(unexpected, vec![OperationId::CompactAsh]);
}

// --- §15.2 proof plans (8) ---

#[test]
fn a_removed_proof_plan_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let key = first_plan(program);

        program.proof_plans.remove(&key);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofPlanCensusMismatch {
            missing: 1,
            unexpected: 0,
        },
    );
}

#[test]
fn an_infeasible_proof_plan_is_rejected() {
    // A plan with one relation's selected proof removed: nothing the
    // exact search offers, and consistent with its own key, so the
    // census is the only thing that can reject it.
    let error = COMPACT_ASH.corrupt(|program| {
        let key = first_plan(program);
        let mut analyzed = program.proof_plans[&key].clone();
        let relation = analyzed
            .proof_plan
            .proofs
            .keys()
            .next()
            .expect("a selected proof")
            .clone();

        analyzed.proof_plan.proofs.remove(&relation);
        program
            .proof_plans
            .insert(analyzed.proof_plan.clone(), analyzed);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofPlanCensusMismatch {
            missing: 0,
            unexpected: 1,
        },
    );
}

#[test]
fn a_duplicated_proof_plan_is_rejected() {
    // A map keyed by the complete typed plan cannot hold one plan twice,
    // so the constructible form of a duplicated plan is one entry
    // carrying another plan's analysis: the same analysis reachable
    // under two keys.
    let error = COMPACT_ASH.corrupt(|program| {
        let mut keys = program.proof_plans.keys().cloned();
        let first = keys.next().expect("a first plan");
        let second = keys.next().expect("a second plan");
        let analysis = program.proof_plans[&second].clone();

        program.proof_plans.insert(first, analysis);
    });

    assert_eq!(error, CompileError::AnalyzedProofPlanKeyMismatch);
}

#[test]
fn a_replaced_selected_proof_is_rejected() {
    // Re-keyed consistently, so the entry agrees with itself and the
    // altered plan is simply not one the exact search offers.
    let error = COMPACT_ASH.corrupt(|program| {
        let key = first_plan(program);
        let mut analyzed = program.proof_plans[&key].clone();
        let relation = analyzed
            .proof_plan
            .proofs
            .keys()
            .next()
            .expect("a selected proof")
            .clone();

        let current = analyzed.proof_plan.proofs[&relation].proof();
        let replacement = if current == ProofKind::ManifestShape {
            ProofKind::SignerMembership
        } else {
            ProofKind::ManifestShape
        };

        analyzed.proof_plan.proofs.insert(
            relation.clone(),
            ProofAlternativeId::new(relation, replacement),
        );
        program.proof_plans.remove(&key);
        program
            .proof_plans
            .insert(analyzed.proof_plan.clone(), analyzed);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofPlanCensusMismatch {
            missing: 1,
            unexpected: 1,
        },
    );
}

#[test]
fn a_proof_on_a_statically_validated_relation_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        bundle.proof == ProofDisposition::StaticallyValidated
    });
    let owner = relation.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        let bundle = plan_mut(program)
            .relation_requirements
            .get_mut(&owner)
            .expect("the relation bundle");

        bundle.proof = ProofDisposition::Selected {
            proof: ProofAlternativeId::new(owner.clone(), ProofKind::ManifestShape),
        };
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofDispositionMismatch { relation },
    );
}

#[test]
fn a_removed_external_proof_class_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        matches!(bundle.proof, ProofDisposition::ExternalEvidence { .. })
    });
    let owner = relation.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        let bundle = plan_mut(program)
            .relation_requirements
            .get_mut(&owner)
            .expect("the relation bundle");

        bundle.proof = ProofDisposition::StaticallyValidated;
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofDispositionMismatch { relation },
    );
}

#[test]
fn a_representation_changed_without_its_proof_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let key = first_plan(program);
        let mut analyzed = program.proof_plans[&key].clone();
        let choice = analyzed
            .proof_plan
            .representations
            .keys()
            .next()
            .expect("a representation choice")
            .clone();
        let mode = analyzed.proof_plan.representations[&choice];
        let replacement = if mode == RepresentationMode::Explicit {
            RepresentationMode::PublicCommitted
        } else {
            RepresentationMode::Explicit
        };

        analyzed
            .proof_plan
            .representations
            .insert(choice, replacement);
        program.proof_plans.remove(&key);
        program
            .proof_plans
            .insert(analyzed.proof_plan.clone(), analyzed);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofPlanCensusMismatch {
            missing: 1,
            unexpected: 1,
        },
    );
}

#[test]
fn removing_one_of_two_plans_with_identical_cases_is_rejected() {
    // Measured property of the pilot plan set: every plan fixes a
    // distinct representation combination, and an execution case
    // embeds the representations it was derived under, so no two plans
    // have equal case censuses. The plans do share a case census
    // *shape* — the same number of cases with the same sponsor
    // structure — which is exactly the situation the mutation is about:
    // counting cases cannot notice that one plan is gone, and only the
    // exact plan census can.
    let program = &COMPACT_ASH.program;
    let census = |plan: &AnalyzedProofPlan| {
        plan.operations
            .values()
            .map(|factor| factor.execution_cases.clone())
            .collect::<Vec<_>>()
    };
    let shape = |plan: &AnalyzedProofPlan| {
        plan.operations
            .values()
            .map(|factor| {
                factor
                    .execution_cases
                    .iter()
                    .map(|case| case.sponsor)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };

    let mut shared = None;

    for (key, analyzed) in &program.proof_plans {
        for (other_key, other) in &program.proof_plans {
            if key == other_key {
                continue;
            }

            assert_ne!(
                census(analyzed),
                census(other),
                "two pilot plans share an execution-case census",
            );

            if shape(analyzed) == shape(other) {
                shared = Some(key.clone());
            }
        }
    }

    let removed = shared.expect("two plans with the same case-census shape");
    let error = COMPACT_ASH.corrupt(|program| {
        program.proof_plans.remove(&removed);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofPlanCensusMismatch {
            missing: 1,
            unexpected: 0,
        },
    );
}

// --- §15.3 relation requirements (9) ---

#[test]
fn a_removed_relation_requirement_is_rejected() {
    // A bundle owning nothing at all, so no aggregate or lifecycle
    // closure has anything to report and the census is what rejects it.
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        bundle.required_capabilities.is_empty()
            && bundle.source_requirements.is_empty()
            && bundle.external_evidence.is_empty()
            && bundle.lifecycle.is_empty()
    });
    let owner = relation.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        plan_mut(program).relation_requirements.remove(&owner);
    });

    let CompileError::AnalyzedRelationRequirementCensusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("a removed relation requirement is not reported as a census defect");
    };

    assert_eq!(missing, vec![relation]);
    assert_eq!(unexpected, [] as [RelationId; 0]);
}

#[test]
fn an_unexpected_relation_requirement_is_rejected() {
    // A bundle for an out-of-scope relation, owning nothing, so the
    // aggregate closure has nothing to complain about and the census is
    // what rejects it.
    let foreign = bound_input(&[OperationId::TransferLive])
        .realization()
        .project()
        .relations
        .nodes
        .iter()
        .map(|declaration| declaration.id.clone())
        .find(|relation| relation.operation() == OperationId::TransferLive)
        .expect("a live-transfer relation");
    let unexpected_relation = foreign.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        let bundle = RelationRequirements {
            relation: foreign.clone(),
            proof: ProofDisposition::StaticallyValidated,
            required_capabilities: BTreeSet::new(),
            source_requirements: BTreeSet::new(),
            external_evidence: BTreeSet::new(),
            representation: None,
            lifecycle: BTreeSet::new(),
        };

        plan_mut(program)
            .relation_requirements
            .insert(foreign, bundle);
    });

    let CompileError::AnalyzedRelationRequirementCensusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("an unexpected relation requirement is not reported as a census defect");
    };

    assert_eq!(missing, [] as [RelationId; 0]);
    assert_eq!(unexpected, vec![unexpected_relation]);
}

#[test]
fn a_removed_capability_is_rejected() {
    // A capability exactly one relation owns: dropping a shared one
    // would leave the aggregate union intact and be reported as the
    // owning relation's bundle instead of as an unowned aggregate item.
    let key = first_plan(&COMPACT_ASH.program);
    let bundles = &COMPACT_ASH.program.proof_plans[&key].relation_requirements;
    let (relation, capability) = bundles
        .iter()
        .flat_map(|(relation, bundle)| {
            bundle
                .required_capabilities
                .iter()
                .map(move |capability| (relation.clone(), *capability))
        })
        .find(|(_, capability)| {
            bundles
                .values()
                .filter(|bundle| bundle.required_capabilities.contains(capability))
                .count()
                == 1
        })
        .expect("a uniquely owned capability");

    let error = COMPACT_ASH.corrupt(move |program| {
        plan_mut(program)
            .relation_requirements
            .get_mut(&relation)
            .expect("the relation bundle")
            .required_capabilities
            .remove(&capability);
    });

    // The aggregate still carries it and no relation owns it: having no
    // owner to name is the defect.
    assert!(matches!(
        error,
        CompileError::AnalyzedCapabilityClosureMismatch { relation: None, .. },
    ));
}

#[test]
fn an_added_capability_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        !bundle.required_capabilities.is_empty()
    });
    let owner = relation.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        plan_mut(program)
            .relation_requirements
            .get_mut(&owner)
            .expect("the relation bundle")
            .required_capabilities
            .insert(RequiredCapability::RefundAuthorization);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedCapabilityClosureMismatch {
            relation: Some(relation),
            capability: RequiredCapability::RefundAuthorization,
        },
    );
}

#[test]
fn a_removed_source_row_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        !bundle.source_requirements.is_empty()
    });

    let error = COMPACT_ASH.corrupt(move |program| {
        let bundle = plan_mut(program)
            .relation_requirements
            .get_mut(&relation)
            .expect("the relation bundle");
        let dropped = bundle
            .source_requirements
            .iter()
            .next()
            .expect("a source row")
            .clone();

        bundle.source_requirements.remove(&dropped);
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedSourceClosureMismatch { relation: None, .. },
    ));
}

#[test]
fn an_added_unauthenticated_source_row_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        !bundle.source_requirements.is_empty()
    });
    let owner = relation.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        let row = crate::source::SourceRequirement {
            operand: OperandId::new(owner.clone(), OperandRole::ProtocolSignerSet),
            source: RequiredSourceKind::PublicConstructionData,
            availability: AvailabilityClass::Public,
            activation: RequirementActivation::Always,
        };

        plan_mut(program)
            .relation_requirements
            .get_mut(&owner)
            .expect("the relation bundle")
            .source_requirements
            .insert(row);
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedSourceClosureMismatch {
            relation: Some(named),
            ..
        } if named == relation,
    ));
}

#[test]
fn a_removed_external_evidence_requirement_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        !bundle.external_evidence.is_empty()
    });

    let error = COMPACT_ASH.corrupt(move |program| {
        plan_mut(program)
            .relation_requirements
            .get_mut(&relation)
            .expect("the relation bundle")
            .external_evidence
            .clear();
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedEvidenceClosureMismatch { relation: None, .. },
    ));
}

#[test]
fn evidence_attached_to_the_wrong_relation_is_rejected() {
    let owner = relation_where(&COMPACT_ASH.program, |bundle| {
        !bundle.external_evidence.is_empty()
    });
    let other = relation_where(&COMPACT_ASH.program, |bundle| {
        bundle.external_evidence.is_empty()
    });
    let wrong = other.clone();

    // The union over relations is unchanged, so every aggregate closure
    // still holds: only the re-derived per-relation ownership can see it.
    let error = COMPACT_ASH.corrupt(move |program| {
        let plan = plan_mut(program);
        let requirement = plan.relation_requirements[&owner]
            .external_evidence
            .iter()
            .next()
            .expect("an evidence requirement")
            .clone();

        plan.relation_requirements
            .get_mut(&other)
            .expect("the relation bundle")
            .external_evidence
            .insert(requirement);
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedEvidenceClosureMismatch {
            relation: Some(named),
            ..
        } if named == wrong,
    ));
}

#[test]
fn a_changed_proof_disposition_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        matches!(bundle.proof, ProofDisposition::Selected { .. })
    });
    let owner = relation.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        plan_mut(program)
            .relation_requirements
            .get_mut(&owner)
            .expect("the relation bundle")
            .proof = ProofDisposition::Selected {
            proof: ProofAlternativeId::new(owner.clone(), ProofKind::SubstrateConservation),
        };
    });

    assert_eq!(
        error,
        CompileError::AnalyzedProofDispositionMismatch { relation },
    );
}

// --- §15.4 execution cases (6) ---

/// One factor's case with the given sponsor disposition.
fn case_with(program: &ScopedAnalyzedProgram, sponsor: SponsorCase) -> ExecutionCaseId {
    let key = first_plan(program);

    program.proof_plans[&key].operations[&OperationId::CompactAsh]
        .execution_cases
        .iter()
        .find(|case| case.sponsor == sponsor)
        .expect("a case with the requested sponsor disposition")
        .clone()
}

#[test]
fn a_removed_sponsorless_case_is_rejected() {
    let case = case_with(&COMPACT_ASH.program, SponsorCase::Absent);

    let error = COMPACT_ASH.corrupt(move |program| {
        ash_factor_mut(program).execution_cases.remove(&case);
    });

    assert!(matches!(
        error,
        CompileError::ExecutionCaseCensusMismatch { .. },
    ));
}

#[test]
fn a_removed_sponsored_case_is_rejected() {
    let case = case_with(&COMPACT_ASH.program, SponsorCase::Present);

    let error = COMPACT_ASH.corrupt(move |program| {
        ash_factor_mut(program).execution_cases.remove(&case);
    });

    assert!(matches!(
        error,
        CompileError::ExecutionCaseCensusMismatch { .. },
    ));
}

#[test]
fn a_duplicated_execution_case_is_rejected() {
    // The case census is a set keyed by the complete typed case, so one
    // case cannot appear twice. The nearest constructible corruption is
    // a second case that repeats an existing case's whole identity and
    // adds one representation entry — the same execution counted twice.
    let case = case_with(&COMPACT_ASH.program, SponsorCase::Absent);

    let error = COMPACT_ASH.corrupt(move |program| {
        let mut duplicate = case;

        duplicate
            .representations
            .insert(ObjectId::CpfpAnchor, RepresentationMode::Explicit);
        ash_factor_mut(program).execution_cases.insert(duplicate);
    });

    assert!(matches!(
        error,
        CompileError::ExecutionCaseCensusMismatch { .. },
    ));
}

#[test]
fn a_case_belonging_to_another_operation_is_rejected() {
    let foreign = {
        let key = first_plan(&COMBINED.program);
        COMBINED.program.proof_plans[&key].operations[&OperationId::TransferLive]
            .execution_cases
            .iter()
            .next()
            .expect("a live-transfer case")
            .clone()
    };

    let error = COMBINED.corrupt(move |program| {
        factor_mut(program, OperationId::CompactAsh)
            .execution_cases
            .insert(foreign);
    });

    assert!(matches!(
        error,
        CompileError::ExecutionCaseCensusMismatch { .. },
    ));
}

#[test]
fn a_representation_changed_on_one_relation_case_is_rejected() {
    let key = relation_case_where(&COMPACT_ASH.program, OperationId::CompactAsh, |_| true);

    let error = COMPACT_ASH.corrupt(move |program| {
        let factor = ash_factor_mut(program);
        let bundle = factor
            .relation_cases
            .remove(&key)
            .expect("the relation-case bundle");
        let mut moved = key.clone();

        moved
            .case
            .representations
            .insert(ObjectId::PlainLbtc, RepresentationMode::PrivateCommitted);
        factor.relation_cases.insert(moved, bundle);
    });

    assert!(matches!(
        error,
        CompileError::RelationCaseCensusMismatch { .. },
    ));
}

#[test]
fn an_object_specific_representation_condition_activated_elsewhere_is_rejected() {
    // A source row whose activation names one object may not become
    // active because a different object selected the same mode. The
    // corruption states exactly that: the row is retained as active in a
    // case whose own object never chose it.
    let key = relation_case_where(
        &COMPACT_ASH.program,
        OperationId::CompactAsh,
        |projection| projection.activity == RelationActivity::Active,
    );
    let owner = key.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        let bundle = ash_factor_mut(program)
            .relation_cases
            .get_mut(&owner)
            .expect("the relation-case bundle");
        let row = crate::source::SourceRequirement {
            operand: OperandId::new(owner.relation.clone(), OperandRole::ProtocolSignerSet),
            source: RequiredSourceKind::PublicConstructionData,
            availability: AvailabilityClass::Public,
            activation: RequirementActivation::WhenRepresentation {
                object: ObjectId::CpfpAnchor,
                mode: RepresentationMode::Explicit,
            },
        };

        bundle.active_sources.insert(row);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedRelationCaseMismatch {
            operation: OperationId::CompactAsh,
            relation: key.relation,
            case: key.case,
            defect: AnalyzedRelationCaseDefect::ActiveSources,
        },
    );
}

// --- §15.5 placement (9) ---

#[test]
fn a_removed_feasible_placement_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let dropped = factor
            .feasible_placements
            .iter()
            .next()
            .expect("a placement")
            .clone();

        factor.feasible_placements.remove(&dropped);
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedPlacementFactorMismatch {
            missing: 1,
            unexpected: 0,
            ..
        },
    ));
}

#[test]
fn an_infeasible_placement_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let mut placement = factor
            .feasible_placements
            .iter()
            .next()
            .expect("a placement")
            .clone();
        let dropped = placement
            .assignments
            .keys()
            .next()
            .expect("an assignment")
            .clone();

        placement.assignments.remove(&dropped);
        factor.feasible_placements.insert(placement);
    });

    assert!(matches!(
        error,
        CompileError::PlacementCensusMismatch { .. },
    ));
}

#[test]
fn a_placement_from_another_operation_is_rejected() {
    let foreign = {
        let key = first_plan(&COMBINED.program);
        COMBINED.program.proof_plans[&key].operations[&OperationId::TransferLive]
            .feasible_placements
            .iter()
            .next()
            .expect("a live-transfer placement")
            .clone()
    };

    let error = COMBINED.corrupt(move |program| {
        factor_mut(program, OperationId::CompactAsh)
            .feasible_placements
            .insert(foreign);
    });

    assert!(matches!(
        error,
        CompileError::PlacementCensusMismatch { .. },
    ));
}

#[test]
fn a_duplicated_carrier_is_rejected() {
    // A carrier set cannot hold one placed carrier twice, so the
    // constructible duplication is the same carrier role assigned a
    // second quantification: one obligation discharged twice.
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let mut placement = factor
            .feasible_placements
            .iter()
            .next()
            .expect("a placement")
            .clone();
        let key = placement
            .assignments
            .keys()
            .next()
            .expect("an assignment")
            .clone();
        let carriers = placement
            .assignments
            .get_mut(&key)
            .expect("the carrier set");
        let existing = carriers.iter().next().expect("a carrier").clone();
        let quantification =
            if existing.quantification == crate::carrier::CarrierQuantification::PerMember {
                crate::carrier::CarrierQuantification::Single
            } else {
                crate::carrier::CarrierQuantification::PerMember
            };

        carriers.insert(PlacedCarrier {
            carrier: existing.carrier,
            quantification,
        });
        factor.feasible_placements.insert(placement);
    });

    assert!(matches!(error, CompileError::DuplicatePlacedCarrier { .. },));
}

#[test]
fn a_redundant_second_carrier_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let mut placement = factor
            .feasible_placements
            .iter()
            .next()
            .expect("a placement")
            .clone();
        let first = placement
            .assignments
            .keys()
            .next()
            .expect("an assignment")
            .clone();
        let selected = placement.assignments[&first].clone();
        let borrowed = placement
            .assignments
            .values()
            .flatten()
            .find(|carrier| !selected.contains(carrier))
            .expect("a carrier the assignment did not select")
            .clone();

        placement
            .assignments
            .get_mut(&first)
            .expect("the carrier set")
            .insert(borrowed);
        factor.feasible_placements.insert(placement);
    });

    assert!(matches!(
        error,
        CompileError::NonCanonicalCarrierAssignment { .. },
    ));
}

#[test]
fn a_per_member_carrier_replaced_by_a_coordinator_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let mut placement = factor
            .feasible_placements
            .iter()
            .find(|placement| {
                placement.assignments.values().any(|carriers| {
                    carriers.iter().any(|placed| {
                        matches!(
                            placed.carrier,
                            crate::carrier::CarrierRole::EveryInputFamilyMember { .. }
                        )
                    })
                })
            })
            .expect("a placement selecting a per-member carrier")
            .clone();
        let key = placement
            .assignments
            .iter()
            .find(|(_, carriers)| {
                carriers.iter().any(|placed| {
                    matches!(
                        placed.carrier,
                        crate::carrier::CarrierRole::EveryInputFamilyMember { .. }
                    )
                })
            })
            .map(|(key, _)| key.clone())
            .expect("the per-member assignment");
        let carriers = placement
            .assignments
            .get_mut(&key)
            .expect("the carrier set");
        let existing = carriers.iter().next().expect("a carrier").clone();
        let crate::carrier::CarrierRole::EveryInputFamilyMember { object } = existing.carrier
        else {
            panic!("the selected carrier is not a per-member role");
        };

        carriers.clear();
        carriers.insert(PlacedCarrier {
            carrier: crate::carrier::CarrierRole::InputFamilyCoordinator { object },
            quantification: existing.quantification,
        });
        factor.feasible_placements.insert(placement);
    });

    assert!(matches!(
        error,
        CompileError::UnpermittedCarrierPlacement { .. },
    ));
}

#[test]
fn an_optional_sponsor_carrier_on_an_unconditional_relation_is_rejected() {
    // The sponsor family's per-member role is eligible only where the
    // sponsor region exists, so assigning it to an unconditional
    // obligation claims a carrier the sponsorless case does not have.
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let mut placement = factor
            .feasible_placements
            .iter()
            .next()
            .expect("a placement")
            .clone();
        let key = placement
            .assignments
            .keys()
            .find(|key| key.case.sponsor == SponsorCase::Absent)
            .expect("a sponsorless obligation")
            .clone();
        let carriers = placement
            .assignments
            .get_mut(&key)
            .expect("the carrier set");

        carriers.clear();
        carriers.insert(PlacedCarrier {
            carrier: crate::carrier::CarrierRole::EveryInputFamilyMember {
                object: ObjectId::PlainLbtc,
            },
            quantification: crate::carrier::CarrierQuantification::PerMember,
        });
        factor.feasible_placements.insert(placement);
    });

    assert!(matches!(
        error,
        CompileError::UnpermittedCarrierPlacement { .. },
    ));
}

#[test]
fn a_removed_selected_layout_requirement_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let mut placement = factor
            .feasible_placements
            .iter()
            .find(|placement| !placement.layout_requirements.is_empty())
            .expect("a placement with layout dependencies")
            .clone();
        let dropped = placement
            .layout_requirements
            .iter()
            .next()
            .expect("a layout requirement")
            .clone();

        placement.layout_requirements.remove(&dropped);
        factor.feasible_placements.insert(placement);
    });

    assert!(matches!(
        error,
        CompileError::MissingLayoutRequirement { .. },
    ));
}

#[test]
fn an_unrelated_layout_requirement_on_a_placement_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let factor = ash_factor_mut(program);
        let mut placement = factor
            .feasible_placements
            .iter()
            .find(|placement| {
                factor
                    .layout_requirements
                    .iter()
                    .any(|requirement| !placement.layout_requirements.contains(requirement))
            })
            .expect("a placement missing one census entry")
            .clone();
        let unrelated = factor
            .layout_requirements
            .iter()
            .find(|requirement| !placement.layout_requirements.contains(requirement))
            .expect("an unselected census entry")
            .clone();

        placement.layout_requirements.insert(unrelated);
        factor.feasible_placements.insert(placement);
    });

    assert!(matches!(
        error,
        CompileError::UnexpectedLayoutRequirement { .. },
    ));
}

// --- §15.6 coverage (12) ---

/// Mutate one relation-case coverage projection of the compact-ash
/// factor.
fn corrupt_coverage(
    select: impl Fn(&RelationCoverageProjection) -> bool + 'static,
    mutate: impl FnOnce(&RelationCaseKey, &mut RelationCoverageProjection) + 'static,
) -> CompileError {
    let key = relation_case_where(&COMPACT_ASH.program, OperationId::CompactAsh, select);

    COMPACT_ASH.corrupt(move |program| {
        let projection = ash_factor_mut(program)
            .coverage
            .requirements
            .get_mut(&key)
            .expect("the coverage projection");

        mutate(&key, projection);
    })
}

/// The coverage defect one corruption is reported as.
fn coverage_defect(error: &CompileError) -> AnalyzedCoverageDefect {
    let CompileError::AnalyzedCoverageCensusMismatch { defect, .. } = error else {
        panic!("the corruption is not reported as a coverage defect: {error:?}");
    };

    *defect
}

#[test]
fn removed_positive_coverage_is_rejected() {
    let error = corrupt_coverage(
        |projection| !projection.positive.is_empty(),
        |_, projection| {
            let dropped = projection
                .positive
                .iter()
                .next()
                .expect("a positive requirement")
                .clone();

            projection.positive.remove(&dropped);
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::MissingPositive,
    );
}

#[test]
fn one_removed_negative_among_several_is_rejected() {
    let error = corrupt_coverage(
        |projection| projection.negative.len() > 1,
        |_, projection| {
            let dropped = projection
                .negative
                .iter()
                .next()
                .expect("a negative requirement")
                .clone();

            projection.negative.remove(&dropped);
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::MissingNegative,
    );
}

#[test]
fn an_unrelated_mutation_class_is_rejected() {
    let error = corrupt_coverage(
        |projection| !projection.negative.is_empty(),
        |key, projection| {
            let existing = projection
                .negative
                .iter()
                .next()
                .expect("a negative requirement")
                .clone();
            let mutation = RelationMutation::UndeclaredOpenFlow;

            projection.negative.insert(NegativeCoverageRequirement {
                id: CoverageRequirementId {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                    boundary: existing.id.boundary,
                    purpose: CoveragePurpose::FocusedReject(mutation),
                },
                role: existing.role,
                mutation,
                collateral: existing.collateral,
            });
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::UnexpectedNegative,
    );
}

#[test]
fn a_mutation_moved_to_the_wrong_boundary_is_rejected() {
    // The moved obligation keeps a role that is valid at either
    // boundary, so what rejects it is the census rather than the
    // separate target-execution role rule.
    let movable = |requirement: &NegativeCoverageRequirement| {
        requirement.role != EvidenceRole::TargetExecution
            && requirement.id.boundary != CoverageBoundary::RuntimeCarrier
    };

    let error = corrupt_coverage(
        move |projection| projection.negative.iter().any(movable),
        move |_, projection| {
            let mut moved = projection
                .negative
                .iter()
                .find(|requirement| movable(requirement))
                .expect("a movable negative requirement")
                .clone();

            projection.negative.remove(&moved);
            moved.id.boundary = if moved.id.boundary == CoverageBoundary::CompilerStatic {
                CoverageBoundary::BackendStructural
            } else {
                CoverageBoundary::CompilerStatic
            };
            projection.negative.insert(moved);
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::MissingNegative,
    );
}

#[test]
fn a_target_execution_role_on_compiler_static_coverage_is_rejected() {
    let error = corrupt_coverage(
        |projection| {
            projection
                .positive
                .iter()
                .any(|requirement| requirement.id.boundary == CoverageBoundary::CompilerStatic)
        },
        |_, projection| {
            let mut moved = projection
                .positive
                .iter()
                .find(|requirement| requirement.id.boundary == CoverageBoundary::CompilerStatic)
                .expect("a compiler-static obligation")
                .clone();

            projection.positive.remove(&moved);
            moved.role = EvidenceRole::TargetExecution;
            projection.positive.insert(moved);
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::TargetExecutionRoleOutsideRuntime,
    );
}

#[test]
fn a_target_execution_role_on_external_evidence_is_rejected() {
    let error = corrupt_coverage(
        |projection| {
            projection
                .positive
                .iter()
                .any(|requirement| requirement.id.boundary == CoverageBoundary::ExternalEvidence)
        },
        |_, projection| {
            let mut moved = projection
                .positive
                .iter()
                .find(|requirement| requirement.id.boundary == CoverageBoundary::ExternalEvidence)
                .expect("an external-evidence obligation")
                .clone();

            projection.positive.remove(&moved);
            moved.role = EvidenceRole::TargetExecution;
            projection.positive.insert(moved);
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::TargetExecutionRoleOutsideRuntime,
    );
}

#[test]
fn removed_inactive_valid_coverage_is_rejected() {
    let error = corrupt_coverage(
        |projection| projection.activity == RelationActivity::Vacuous,
        |_, projection| {
            let dropped = projection
                .positive
                .iter()
                .find(|requirement| requirement.id.purpose == CoveragePurpose::InactiveAccept)
                .expect("an inactive-accept obligation")
                .clone();

            projection.positive.remove(&dropped);
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::MissingPositive,
    );
}

#[test]
fn a_rejection_added_to_an_inactive_relation_is_rejected() {
    let error = corrupt_coverage(
        |projection| projection.activity == RelationActivity::Vacuous,
        |key, projection| {
            let mutation = RelationMutation::AmountMismatch;

            projection.negative.insert(NegativeCoverageRequirement {
                id: CoverageRequirementId {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                    boundary: CoverageBoundary::RuntimeCarrier,
                    purpose: CoveragePurpose::FocusedReject(mutation),
                },
                role: EvidenceRole::InactiveCaseAcceptance,
                mutation,
                collateral: crate::coverage::CollateralRequirement {
                    policy: CollateralPolicy::ReportAdditional,
                    dependency_closure: BTreeSet::new(),
                },
            });
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::UnexpectedNegative,
    );
}

#[test]
fn a_removed_accepted_projection_is_rejected() {
    let error = corrupt_coverage(
        |projection| !projection.projections.is_empty(),
        |_, projection| {
            let dropped = *projection
                .projections
                .keys()
                .next()
                .expect("an accepted projection");

            projection.projections.remove(&dropped);
        },
    );

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::MissingProjection,
    );
}

#[test]
fn changed_dependency_collateral_is_rejected() {
    let error = corrupt_coverage(
        |projection| !projection.negative.is_empty(),
        |_, projection| {
            let mut changed = projection
                .negative
                .iter()
                .next()
                .expect("a negative requirement")
                .clone();

            projection.negative.remove(&changed);
            changed.collateral.policy = match changed.collateral.policy {
                CollateralPolicy::ReportAdditional => {
                    CollateralPolicy::RequireIntendedAndDependencyClosure
                }
                CollateralPolicy::RequireIntendedAndDependencyClosure => {
                    CollateralPolicy::ReportAdditional
                }
            };
            projection.negative.insert(changed);
        },
    );

    assert_eq!(coverage_defect(&error), AnalyzedCoverageDefect::Collateral);
}

#[test]
fn an_introduced_coverage_dependency_cycle_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;
        let edge = graph.edges.first().expect("a dependency edge").clone();

        graph.edges.push(CoverageDependency {
            source: edge.target,
            target: edge.source,
            edge: CoverageEdge::RelationPrerequisite,
        });
    });

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::UnexpectedDependencyEdge,
    );
}

#[test]
fn a_cross_operation_coverage_edge_is_rejected() {
    let foreign = {
        let key = first_plan(&COMBINED.program);
        COMBINED.program.proof_plans[&key].operations[&OperationId::TransferLive]
            .coverage_dependencies
            .nodes
            .first()
            .expect("a live-transfer coverage symbol")
            .id
            .clone()
    };

    let error = COMBINED.corrupt(move |program| {
        let graph = &mut factor_mut(program, OperationId::CompactAsh).coverage_dependencies;
        let edge = graph.edges.first().expect("a dependency edge").clone();

        graph.edges.push(CoverageDependency {
            source: edge.source,
            target: foreign,
            edge: CoverageEdge::RelationPrerequisite,
        });
    });

    assert!(matches!(
        error,
        CompileError::CrossOperationCoverageDependency { .. },
    ));
}

// --- SR3-03: the execution report is authenticated, not asserted ---

#[test]
fn a_rewritten_proof_search_count_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        program.execution_report.proof_search.states_visited += 1;
    });

    assert_eq!(
        error,
        CompileError::AnalyzedExecutionReportMismatch {
            defect: AnalyzedExecutionReportDefect::ProofSearch,
        },
    );
}

#[test]
fn a_rewritten_placement_search_count_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let report = program
            .execution_report
            .operation_placement_search
            .get_mut(&OperationId::CompactAsh)
            .expect("the pilot placement report");

        report.complete_assignments += 1;
    });

    assert_eq!(
        error,
        CompileError::AnalyzedExecutionReportMismatch {
            defect: AnalyzedExecutionReportDefect::PlacementSearch,
        },
    );
}

#[test]
fn a_misreported_proof_search_limit_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let limits = &mut program.execution_report.proof_search_limits;

        *limits = crate::ProofSearchLimits::new(
            std::num::NonZeroU64::new(limits.maximum_states.get() + 1).expect("nonzero"),
            limits.maximum_candidates,
        );
    });

    assert_eq!(
        error,
        CompileError::AnalyzedExecutionReportMismatch {
            defect: AnalyzedExecutionReportDefect::ProofSearchLimits,
        },
    );
}

#[test]
fn a_misreported_placement_search_limit_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let limits = &mut program.execution_report.placement_search_limits;

        *limits = PlacementSearchLimits::new(
            std::num::NonZeroU64::new(limits.maximum_states.get() + 1).expect("nonzero"),
            limits.maximum_candidates,
        );
    });

    assert_eq!(
        error,
        CompileError::AnalyzedExecutionReportMismatch {
            defect: AnalyzedExecutionReportDefect::PlacementSearchLimits,
        },
    );
}

// --- S2-04: the coverage graph projection is an exact canonical census ---

#[test]
fn a_duplicated_coverage_dependency_node_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;
        let repeated = graph.nodes.first().expect("a coverage symbol").clone();

        graph.nodes.insert(0, repeated);
    });

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::DuplicateDependencyNode,
    );
}

#[test]
fn a_duplicated_coverage_dependency_edge_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;
        let repeated = graph.edges.first().expect("a dependency edge").clone();

        graph.edges.insert(0, repeated);
    });

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::DuplicateDependencyEdge,
    );
}

#[test]
fn an_out_of_order_coverage_dependency_node_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;

        assert!(graph.nodes.len() > 1, "the pilot graph defines symbols");
        graph.nodes.swap(0, 1);
    });

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::NoncanonicalDependencyNodeOrder,
    );
}

#[test]
fn an_out_of_order_coverage_dependency_edge_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;

        assert!(graph.edges.len() > 1, "the pilot graph declares edges");
        graph.edges.swap(0, 1);
    });

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::NoncanonicalDependencyEdgeOrder,
    );
}

#[test]
fn a_duplicated_node_replacing_a_dropped_one_is_rejected() {
    // Length-preserving corruption: the vector still carries as many
    // entries as the re-derived census, so only a comparison that
    // reads repetition rather than membership alone catches it.
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;

        assert!(graph.nodes.len() > 1, "the pilot graph defines symbols");
        graph.nodes[1] = graph.nodes[0].clone();
    });

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::MissingDependencyNode,
        "the dropped symbol is the more informative diagnostic",
    );
}

#[test]
fn a_duplicated_edge_replacing_a_dropped_one_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;

        assert!(graph.edges.len() > 1, "the pilot graph declares edges");
        graph.edges[1] = graph.edges[0].clone();
    });

    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::MissingDependencyEdge,
    );
}

#[test]
fn a_repeated_self_edge_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let graph = &mut ash_factor_mut(program).coverage_dependencies;
        let node = graph.nodes.first().expect("a coverage symbol").id.clone();
        let loop_edge = CoverageDependency {
            source: node.clone(),
            target: node,
            edge: CoverageEdge::RelationPrerequisite,
        };

        graph.edges.push(loop_edge.clone());
        graph.edges.push(loop_edge);
    });

    // Neither repetition of a self-edge is derivable, so the census
    // rejects it before any order or duplication rule applies.
    assert_eq!(
        coverage_defect(&error),
        AnalyzedCoverageDefect::UnexpectedDependencyEdge,
    );
}

// --- §15.7 lifecycle (6) ---

#[test]
fn a_removed_future_exit_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let plan = plan_mut(program);
        let LifecycleCompleteness::Incomplete { obligations } = &mut plan.lifecycle else {
            panic!("the pilot plan claims a complete lifecycle");
        };
        let dropped = obligations.iter().next().expect("an obligation").clone();

        obligations.remove(&dropped);
    });

    assert_eq!(error, CompileError::AnalyzedLifecycleStatusMismatch);
}

#[test]
fn an_outside_scope_exit_marked_available_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let plan = plan_mut(program);
        let relation = plan
            .relation_requirements
            .iter()
            .find(|(_, bundle)| {
                bundle
                    .lifecycle
                    .iter()
                    .any(|row| row.status == LifecycleExitStatus::DeclaredOutsideCompilerScope)
            })
            .map(|(relation, _)| relation.clone())
            .expect("a relation owning an outside-scope exit");
        let bundle = plan
            .relation_requirements
            .get_mut(&relation)
            .expect("the relation bundle");
        let mut row = bundle
            .lifecycle
            .iter()
            .find(|row| row.status == LifecycleExitStatus::DeclaredOutsideCompilerScope)
            .expect("an exit declared outside the compiler scope")
            .clone();

        bundle.lifecycle.remove(&row);
        row.status = LifecycleExitStatus::AvailableInCompilerScope;
        bundle.lifecycle.insert(row);
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedLifecycleClosureMismatch { .. },
    ));
}

#[test]
fn an_undeclared_lifecycle_exit_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        let plan = plan_mut(program);
        let relation = plan
            .relation_requirements
            .iter()
            .find(|(_, bundle)| !bundle.lifecycle.is_empty())
            .map(|(relation, _)| relation.clone())
            .expect("a lifecycle-exit relation");

        plan.relation_requirements
            .get_mut(&relation)
            .expect("the relation bundle")
            .lifecycle
            .insert(LifecycleRequirement {
                object: ObjectId::PlainLbtc,
                representation: RepresentationMode::Explicit,
                exit: OperationId::Burn,
                status: LifecycleExitStatus::DeclaredOutsideCompilerScope,
            });
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedLifecycleClosureMismatch { .. },
    ));
}

#[test]
fn a_removed_allowed_representation_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        bundle.representation.is_some()
    });
    let owner = relation.clone();

    let error = COMPACT_ASH.corrupt(move |program| {
        plan_mut(program)
            .relation_requirements
            .get_mut(&owner)
            .expect("the relation bundle")
            .representation = None;
    });

    assert_eq!(
        error,
        CompileError::AnalyzedRelationRequirementMismatch { relation },
    );
}

#[test]
fn a_graph_node_left_after_removing_its_semantic_relation_is_rejected() {
    // The lifecycle weld is re-derived from the input, so a stored
    // analysis that dropped the semantic relation while its source still
    // declares the exit disagrees with the re-derived relation census.
    let error = COMPACT_ASH.corrupt(|program| {
        let position = program
            .foundation
            .relations
            .nodes
            .iter()
            .position(|node| {
                matches!(
                    node.source.relation,
                    realization::Relation::LifecycleExit { .. }
                )
            })
            .expect("a lifecycle-exit relation");

        program.foundation.relations.nodes.remove(position);
    });

    assert_eq!(
        error,
        CompileError::AnalyzedFoundationMismatch {
            defect: AnalyzedFoundationDefect::RelationCensus,
        },
    );
}

#[test]
fn a_semantic_relation_left_after_removing_its_graph_node_is_rejected() {
    // The opposite direction: the retained source's lifecycle graph
    // loses the exit while the analyzed relation keeps requiring it.
    let error = COMPACT_ASH.corrupt(|program| {
        program.source.realization.lifecycle.edges.pop();
    });

    assert_eq!(
        error,
        CompileError::AnalyzedSourceBindingMismatch {
            defect: AnalyzedSourceDefect::RealizationProjection,
        },
    );
}

// --- §15.8 sponsor opacity (7) ---

#[test]
fn a_sponsor_amount_operand_is_rejected() {
    let error = corrupt_coverage(
        |projection| !projection.positive.is_empty(),
        |key, projection| {
            let mut named = projection
                .positive
                .iter()
                .next()
                .expect("a positive requirement")
                .clone();

            projection.positive.remove(&named);
            named.operands.push(OperandId::new(
                key.relation.clone(),
                OperandRole::ObjectFamilyAmount {
                    side: TransactionSide::Input,
                    object: ObjectId::PlainLbtc,
                },
            ));
            projection.positive.insert(named);
        },
    );

    assert_eq!(error, CompileError::SponsorValueRead);
}

#[test]
fn a_public_sponsor_sum_is_rejected() {
    let relation = relation_where(&COMPACT_ASH.program, |bundle| {
        !bundle.source_requirements.is_empty()
    });

    let error = COMPACT_ASH.corrupt(move |program| {
        let row = sponsor_amount_row(&relation);

        plan_mut(program)
            .relation_requirements
            .get_mut(&relation)
            .expect("the relation bundle")
            .source_requirements
            .insert(row);
    });

    assert_eq!(error, CompileError::SponsorValueRead);
}

#[test]
fn a_sponsor_positivity_mutation_is_rejected() {
    // The typed mutation vocabulary has no sponsor-amount class, so the
    // constructible form of this corruption is a focused rejection
    // obligation whose compared operands name the erased amount — a
    // mutation requirement that can only be answered by reading it.
    let error = corrupt_coverage(
        |projection| !projection.positive.is_empty(),
        |key, projection| {
            let existing = projection
                .positive
                .iter()
                .next()
                .expect("a positive requirement")
                .clone();

            projection.positive.insert(PositiveCoverageRequirement {
                id: CoverageRequirementId {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                    boundary: existing.id.boundary,
                    purpose: CoveragePurpose::FocusedReject(RelationMutation::AmountMismatch),
                },
                role: existing.role,
                representation: None,
                operands: vec![OperandId::new(
                    key.relation.clone(),
                    OperandRole::ObjectFamilyAmount {
                        side: TransactionSide::Input,
                        object: ObjectId::PlainLbtc,
                    },
                )],
            });
        },
    );

    assert_eq!(error, CompileError::SponsorValueRead);
}

#[test]
fn a_sponsor_amount_source_row_is_rejected() {
    let key = relation_case_where(
        &COMPACT_ASH.program,
        OperationId::CompactAsh,
        |projection| projection.activity == RelationActivity::Active,
    );

    let error = COMPACT_ASH.corrupt(move |program| {
        let row = sponsor_amount_row(&key.relation);

        ash_factor_mut(program)
            .relation_cases
            .get_mut(&key)
            .expect("the relation-case bundle")
            .active_sources
            .insert(row);
    });

    assert_eq!(error, CompileError::SponsorValueRead);
}

#[test]
fn sponsor_amount_layout_routing_is_rejected() {
    let key = relation_case_where(&COMPACT_ASH.program, OperationId::CompactAsh, |_| true);

    let error = COMPACT_ASH.corrupt(move |program| {
        let requirement = LayoutRequirement::MakeSourceAvailable {
            relation: key.relation.clone(),
            case: key.case.clone(),
            carrier: crate::carrier::CarrierRole::EveryInputFamilyMember {
                object: ObjectId::PlainLbtc,
            },
            source: sponsor_amount_row(&key.relation),
        };

        ash_factor_mut(program)
            .layout_requirements
            .insert(requirement);
    });

    assert_eq!(error, CompileError::SponsorValueRead);
}

#[test]
fn a_sponsor_amount_in_an_accepted_projection_is_rejected() {
    let error = corrupt_coverage(
        |projection| !projection.projections.is_empty(),
        |key, projection| {
            let boundary = *projection
                .projections
                .keys()
                .next()
                .expect("an accepted projection");

            projection
                .projections
                .get_mut(&boundary)
                .expect("the accepted projection")
                .operands
                .push(OperandId::new(
                    key.relation.clone(),
                    OperandRole::ObjectFamilyAmount {
                        side: TransactionSide::Input,
                        object: ObjectId::PlainLbtc,
                    },
                ));
        },
    );

    assert_eq!(error, CompileError::SponsorValueRead);
}

#[test]
fn a_sponsor_amount_in_an_evidence_subject_is_rejected() {
    let error = corrupt_coverage(
        |projection| {
            projection
                .projections
                .contains_key(&CoverageBoundary::ExternalEvidence)
        },
        |key, projection| {
            projection
                .projections
                .get_mut(&CoverageBoundary::ExternalEvidence)
                .expect("the external-evidence projection")
                .sources
                .push(sponsor_amount_row(&key.relation));
        },
    );

    assert_eq!(error, CompileError::SponsorValueRead);
}

// --- beyond §15: the factorization and census errors with no listed
// mutation of their own ---

#[test]
fn a_stored_combined_placement_product_is_rejected() {
    // The union of two operations' assignments: one member of the
    // product the analyzed program stores the factors of.
    let foreign = {
        let key = first_plan(&COMBINED.program);
        COMBINED.program.proof_plans[&key].operations[&OperationId::TransferLive]
            .feasible_placements
            .iter()
            .next()
            .expect("a live-transfer placement")
            .clone()
    };

    let error = COMBINED.corrupt(move |program| {
        let factor = factor_mut(program, OperationId::CompactAsh);
        let mut combined = factor
            .feasible_placements
            .iter()
            .next()
            .expect("a compact-ash placement")
            .clone();

        combined.assignments.extend(foreign.assignments);
        combined
            .layout_requirements
            .extend(foreign.layout_requirements);
        factor.feasible_placements.insert(combined);
    });

    assert_eq!(
        error,
        CompileError::UnexpectedCombinedPlacementProduct {
            operation: OperationId::CompactAsh,
        },
    );
}

#[test]
fn a_removed_operation_factor_is_rejected() {
    let error = COMBINED.corrupt(|program| {
        plan_mut(program)
            .operations
            .remove(&OperationId::TransferLive);
    });

    let CompileError::AnalyzedOperationCensusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("a removed operation factor is not reported as a census defect");
    };

    assert_eq!(missing, vec![OperationId::TransferLive]);
    assert_eq!(unexpected, [] as [OperationId; 0]);
}

#[test]
fn an_added_operation_layout_requirement_is_rejected() {
    let key = relation_case_where(&COMPACT_ASH.program, OperationId::CompactAsh, |_| true);

    let error = COMPACT_ASH.corrupt(move |program| {
        ash_factor_mut(program).layout_requirements.insert(
            LayoutRequirement::IsolateSponsorRegion {
                relation: key.relation,
                case: key.case,
            },
        );
    });

    assert_eq!(
        error,
        CompileError::AnalyzedLayoutCensusMismatch {
            operation: OperationId::CompactAsh,
        },
    );
}

#[test]
fn a_corrupted_bundle_coverage_copy_is_rejected() {
    // The relation-case bundle keeps its own copy of its coverage
    // projection. A corruption of that copy alone is internally
    // consistent with nothing and must still be caught.
    let key = relation_case_where(
        &COMPACT_ASH.program,
        OperationId::CompactAsh,
        |projection| !projection.positive.is_empty(),
    );

    let error = COMPACT_ASH.corrupt(move |program| {
        let bundle = ash_factor_mut(program)
            .relation_cases
            .get_mut(&key)
            .expect("the relation-case bundle");
        let dropped = bundle
            .coverage
            .positive
            .iter()
            .next()
            .expect("a positive requirement")
            .clone();

        bundle.coverage.positive.remove(&dropped);
    });

    assert!(matches!(
        error,
        CompileError::AnalyzedRelationCaseMismatch {
            defect: AnalyzedRelationCaseDefect::Coverage,
            ..
        },
    ));
}

#[test]
fn a_corrupted_foundation_dependency_census_is_rejected() {
    let error = COMPACT_ASH.corrupt(|program| {
        program.foundation.relations.edges.clear();
    });

    assert_eq!(
        error,
        CompileError::AnalyzedFoundationMismatch {
            defect: AnalyzedFoundationDefect::RelationDependencies,
        },
    );
}
