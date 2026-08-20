//! Public-API boundary test for the target-independent compiler
//! package.
//!
//! This integration test compiles as an external consumer. At the
//! crate-boundary stage it proves three things: the crate is
//! consumable from outside, its error root is public and carries
//! architecture-owned identifiers rather than compiler-local
//! restatements of them, and no analysis surface has leaked out ahead
//! of the deliverable that owns it.

use architecture::OperationId;
use compiler::CompileError;

#[test]
fn the_error_root_is_publicly_constructible_and_displays() {
    let error = CompileError::UnsupportedRealizationSchema { schema: 17 };
    assert_eq!(error.to_string(), "unsupported realization schema 17");

    let error = CompileError::InvalidRealization;
    assert_eq!(
        error.to_string(),
        "realization failed its owner's validation"
    );

    let error = CompileError::ArchitectureBindingMismatch;
    assert!(error.to_string().contains("does not match"));
}

#[test]
fn scope_errors_carry_architecture_owned_operation_ids() {
    // The compiler cites the architecture-owned identifier. It does
    // not mint a compiler-local operation identity, which would
    // duplicate an upstream ID and obscure its owner.
    let error = CompileError::IncompleteRealizationScope {
        operation: OperationId::CompactAsh,
    };
    let CompileError::IncompleteRealizationScope { operation } = error else {
        panic!("constructed variant must round-trip");
    };
    assert_eq!(operation, OperationId::CompactAsh);

    let duplicate = CompileError::DuplicateScopeOperation {
        operation: OperationId::TransferLive,
    };
    assert_ne!(
        duplicate,
        CompileError::IncompleteRealizationScope {
            operation: OperationId::TransferLive,
        },
        "an absent scope member and a repeated one are distinct failures"
    );
}

#[test]
fn the_error_root_is_a_standard_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}

    assert_error(&CompileError::InvalidRealization);
}

// --- P2-004: the validated input boundary ---

use compiler::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};

const fn test_policy() -> AnalysisPolicy {
    AnalysisPolicy::strict(ProofSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(10_000).expect("nonzero"),
    ))
}

fn phase1_realization() -> realization::ScopedRealizationSpec {
    realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::phase1_pilots(),
    )
    .expect("phase-1 pilots derive")
}

#[test]
fn phase1_realization_binds_with_explicit_scope() {
    let realization = phase1_realization();
    let expected_binding = realization.architecture().clone();
    let scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap();

    let bound = bind_input(
        &architecture::ARCHITECTURE,
        realization,
        scope,
        test_policy(),
    )
    .unwrap();

    assert_eq!(
        bound.scope().operations(),
        [OperationId::TransferLive, OperationId::CompactAsh],
        "scope is canonical stable-code order",
    );
    assert_eq!(bound.policy(), test_policy());
    assert_eq!(bound.architecture_binding(), &expected_binding);
    assert!(
        bound
            .realization()
            .operation(OperationId::CompactAsh)
            .is_some()
    );
}

#[test]
fn single_pilot_scopes_bind_and_permutations_are_equal() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let scope = CompilationScope::from_operations([operation]).unwrap();
        bind_input(
            &architecture::ARCHITECTURE,
            phase1_realization(),
            scope,
            test_policy(),
        )
        .unwrap();
    }

    assert_eq!(
        CompilationScope::from_operations([OperationId::TransferLive, OperationId::CompactAsh])
            .unwrap(),
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap(),
    );
}

#[test]
fn scope_construction_rejects_empty_and_duplicate() {
    assert_eq!(
        CompilationScope::from_operations([]).unwrap_err(),
        CompileError::EmptyCompilationScope,
    );
    assert_eq!(
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::CompactAsh])
            .unwrap_err(),
        CompileError::DuplicateScopeOperation {
            operation: OperationId::CompactAsh,
        },
    );
}

#[test]
fn scope_outside_the_realization_is_incomplete() {
    // The pilot realization is intentionally partial; requesting Burn
    // must fail rather than silently drop or complete the member.
    let scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::Burn]).unwrap();

    assert_eq!(
        bind_input(
            &architecture::ARCHITECTURE,
            phase1_realization(),
            scope,
            test_policy(),
        )
        .unwrap_err(),
        CompileError::IncompleteRealizationScope {
            operation: OperationId::Burn,
        },
    );
}

#[test]
fn a_different_architecture_semantic_body_is_a_binding_mismatch() {
    let mut mutated = architecture::ARCHITECTURE;
    mutated.document.specification.version = "0.0.0-binding-mismatch-test";

    let scope = CompilationScope::from_operations([OperationId::CompactAsh]).unwrap();

    assert_eq!(
        bind_input(&mutated, phase1_realization(), scope, test_policy(),).unwrap_err(),
        CompileError::ArchitectureBindingMismatch,
    );
}

// --- Guide-8 §15 and §20.4: the abstract target requirement boundary ---

use std::collections::BTreeSet;

use compiler::target::{
    ExternalEvidenceRole, PlacementSearchLimits, RequiredCapability, TargetRequirementSet,
    analyze_target_requirements,
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
const fn placement_limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

fn pilot_requirements() -> TargetRequirementSet {
    let scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .expect("pilot scope");
    let input = bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        scope,
        test_policy(),
    )
    .expect("bind input");

    analyze_target_requirements(&input, placement_limits()).expect("pilot requirements")
}

#[test]
fn every_required_capability_is_publicly_nameable() {
    // Written out rather than folded over the census: this test is the
    // external statement that each variant is reachable by name from
    // outside the crate, and a loop over the census would state
    // nothing.
    let named = [
        RequiredCapability::AuthenticatedObjectRecognition,
        RequiredCapability::AuthenticatedFamilyCardinality,
        RequiredCapability::AuthenticatedCanonicalPartition,
        RequiredCapability::AuthenticatedOpenFlowPartition,
        RequiredCapability::AuthenticatedRootEffects,
        RequiredCapability::AuthenticatedProjectionSet,
        RequiredCapability::ExactPublicAmountArithmetic,
        RequiredCapability::ConfidentialValueConservation,
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::OperatorAuthorization,
        RequiredCapability::RefundAuthorization,
        RequiredCapability::PublicConstructibility,
        RequiredCapability::WholeTransactionValueConservation,
    ];

    assert_eq!(RequiredCapability::ALL, named);
    assert_eq!(named.iter().collect::<BTreeSet<_>>().len(), named.len());
    assert!(named.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn every_evidence_role_is_publicly_nameable() {
    assert_eq!(
        ExternalEvidenceRole::ALL,
        [ExternalEvidenceRole::SubstrateConservation],
    );
}

#[test]
fn target_requirements_come_only_from_a_completed_analysis() {
    // There is no public constructor, no `Default`, and no builder:
    // this call is the only route to the type, and it runs the complete
    // analysis and its independent validator before returning.
    let requirements = pilot_requirements();
    let capabilities = requirements.capabilities().collect::<Vec<_>>();

    assert_ne!(
        capabilities,
        [] as [compiler::target::RequiredCapability; 0]
    );
    assert!(
        capabilities.windows(2).all(|pair| pair[0] < pair[1]),
        "the published census is canonically ordered, so no member repeats",
    );
    assert!(
        capabilities
            .iter()
            .all(|capability| RequiredCapability::ALL.contains(capability)),
        "no published capability is outside the census the adapter matches on",
    );

    let evidence = requirements.external_evidence().collect::<Vec<_>>();

    assert_eq!(evidence, [ExternalEvidenceRole::SubstrateConservation]);
}

#[test]
fn an_analysis_that_did_not_complete_publishes_no_requirements() {
    // The boundary is not reachable around an incomplete analysis. A
    // search bound too small to finish is the case that matters: the
    // analysis genuinely started and genuinely did not finish, and what
    // comes back is a typed failure rather than the requirements found
    // so far. A partial requirement set is precisely the weakening a
    // target assessment exists to prevent, so there is no value of the
    // type that carries one.
    let scope =
        CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .expect("pilot scope");
    let input = bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        scope,
        test_policy(),
    )
    .expect("bind input");

    let truncated = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1).expect("nonzero"),
        std::num::NonZeroU64::new(1).expect("nonzero"),
    );

    assert!(
        matches!(
            analyze_target_requirements(&input, truncated),
            Err(CompileError::PlacementSearchStateLimitExceeded { .. }
                | CompileError::PlacementCandidateLimitExceeded { .. })
        ),
        "a truncated search is a typed failure, never a smaller requirement set",
    );
}

#[test]
fn the_public_boundary_is_deterministic_and_target_free() {
    // The target package is not a dependency of this test target, so no
    // target-specific type can appear in a compiler signature this test
    // names: that absence is enforced by the package graph rather than
    // asserted here. What is asserted is that the boundary's whole
    // vocabulary is the two abstract censuses above, and that direct
    // typed comparison of two analyses is the whole comparison
    // mechanism — no identity is minted to stand in for one.
    assert_eq!(pilot_requirements(), pilot_requirements());

    let requirements = pilot_requirements();
    let capabilities = requirements.capabilities().collect::<BTreeSet<_>>();
    let evidence = requirements.external_evidence().collect::<BTreeSet<_>>();

    assert!(!capabilities.is_empty() && !evidence.is_empty());
}

// --- Guide-12 §7.4: the public compact-ASH target-operation plan ---

use compiler::operation_plan::{
    LayoutRequirement, OperandRole, RepresentationNarrowing, ValidatedTargetOperationPlan,
    plan_compact_ash_target_operation,
};

/// The erased sponsor region's object family, named rather than
/// inferred: the assertion below is about that family and not about
/// whichever family a plan happened to place first.
const ORDINARY_LBTC: architecture::ObjectId = architecture::ObjectId::PlainLbtc;

/// The plan module's own source, read as an external consumer sees the
/// crate rather than as the crate sees itself.
const PLAN_SOURCE: &str = include_str!("../src/operation_plan.rs");

fn pilot_plan(operations: &[OperationId]) -> ValidatedTargetOperationPlan {
    let scope = CompilationScope::from_operations(operations.iter().copied()).expect("pilot scope");
    let input = bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        scope,
        test_policy(),
    )
    .expect("bind input");

    plan_compact_ash_target_operation(&input, placement_limits()).expect("pilot plan")
}

#[test]
fn the_operation_plan_comes_only_from_a_completed_analysis() {
    // There is no public constructor, no `Default`, and no builder:
    // this call is the only route to the type, and it runs the complete
    // analysis, that analysis's own validator, the Phase-4 policy
    // filter, and the plan's independent assembly validator before
    // returning. A plan assembled from arbitrary fields would be a
    // request rather than an analysis, and nothing downstream could
    // tell the two apart once they shared a type.
    let plan = pilot_plan(&[OperationId::CompactAsh, OperationId::TransferLive]);

    assert_eq!(plan.operation(), OperationId::CompactAsh);
    assert_eq!(plan.relations().count(), 23);
    assert_eq!(plan.cases().count(), 2);
    assert_eq!(plan.layout().count(), 69);
    assert!(plan.carriers().count() > 0 && plan.coverage().count() > 0);
}

#[test]
fn an_incomplete_analysis_publishes_no_plan() {
    let scope = CompilationScope::from_operations([OperationId::CompactAsh]).expect("pilot scope");
    let input = bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        scope,
        test_policy(),
    )
    .expect("bind input");
    let truncated = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1).expect("nonzero"),
        std::num::NonZeroU64::new(1).expect("nonzero"),
    );

    assert!(
        plan_compact_ash_target_operation(&input, truncated).is_err(),
        "a truncated search is a typed failure, never a smaller plan",
    );
}

#[test]
fn equal_inputs_and_declaration_permutations_produce_equal_projections() {
    let first = pilot_plan(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let permuted = pilot_plan(&[OperationId::TransferLive, OperationId::CompactAsh]);

    assert_eq!(first, permuted);
    assert_eq!(
        first,
        pilot_plan(&[OperationId::CompactAsh, OperationId::TransferLive]),
    );
}

#[test]
fn the_plan_states_the_explicit_phase4_representation_selection() {
    let plan = pilot_plan(&[OperationId::CompactAsh]);
    let policy = plan.representation();

    assert_eq!(
        policy.selection(architecture::ObjectId::Ash),
        Some(realization::RepresentationMode::Explicit),
        "Guide 11 fixes the Phase-4 candidate to an explicit public boundary",
    );
    assert_eq!(
        policy.narrowing(architecture::ObjectId::Ash),
        Some(RepresentationNarrowing::DeploymentPolicy),
        "the alternative set was narrowed by policy, not by semantic necessity",
    );
    assert!(
        policy
            .approved(architecture::ObjectId::Ash)
            .is_some_and(|approved| approved.len() > 1),
        "the approved set records what the realization allows, not what policy chose",
    );

    // §5.7: the candidate is lifecycle-incomplete and says so.
    assert!(!plan.lifecycle().release_complete());
    assert_eq!(
        plan.lifecycle().implemented().collect::<Vec<_>>(),
        [OperationId::CompactAsh],
    );
    assert!(
        plan.lifecycle()
            .outstanding()
            .all(|requirement| requirement.exit == OperationId::Clear),
    );
}

#[test]
fn no_sponsor_amount_enters_any_public_plan_field() {
    // Structural rather than textual: every published layout
    // requirement and every published source row is matched against the
    // erased sponsor family by name.
    //
    // The *amount* is what is erased. Membership, cardinality,
    // authorization, and disjointness over the sponsor family are
    // required relations of §5.5, so an authenticated census of the
    // ordinary L-BTC family is expected here and is not a leak.
    let plan = pilot_plan(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let names_sponsor_amount = |requirement: &LayoutRequirement| {
        matches!(
            requirement,
            LayoutRequirement::MakeSourceAvailable { source, .. }
                if matches!(
                    source.operand.role(),
                    OperandRole::ObjectFamilyAmount { object: ORDINARY_LBTC, .. },
                )
        )
    };

    for requirement in plan.layout() {
        assert!(!names_sponsor_amount(requirement), "{requirement:?}");
    }

    for source in plan
        .relations()
        .flat_map(|relation| relation.source_requirements.iter())
        .chain(
            plan.relations()
                .flat_map(|relation| relation.cases.values())
                .flat_map(|case| case.active_sources.iter()),
        )
    {
        assert!(
            !matches!(
                source.operand.role(),
                OperandRole::ObjectFamilyAmount {
                    object: ORDINARY_LBTC,
                    ..
                },
            ),
            "{source:?}",
        );
    }
}

#[test]
fn the_public_plan_surface_carries_no_digest_and_no_graph_handle() {
    // §1.10 mints no plan digest and reserves no field for a future
    // one; §7.2 admits no Petgraph index. Both are properties of the
    // published source rather than of any one value, so the source is
    // what is checked — a field added later fails here even if no test
    // happens to read it.
    //
    // Comment lines are excluded deliberately. The module documents
    // that it carries no digest, and a scan that could not tell the
    // prohibition from a violation would forbid saying so.
    let code = PLAN_SOURCE
        .lines()
        .filter(|line| {
            let line = line.trim_start();

            !(line.starts_with("//") || line.starts_with("///") || line.starts_with("//!"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    for banned in [
        "digest",
        "Digest",
        "NodeIndex",
        "EdgeIndex",
        "petgraph",
        "DiGraph",
        "PlanHash",
    ] {
        assert!(
            !code.contains(banned),
            "the published plan surface must not name {banned}",
        );
    }

    // The corruption handles the in-crate oracles need are compiled out
    // of every non-test build, so no mutable route to a published field
    // survives into a consumer's copy of this crate.
    for line in PLAN_SOURCE.lines() {
        let line = line.trim_start();

        assert!(
            !(line.starts_with("pub fn") || line.starts_with("pub const fn"))
                || !line.contains("_mut("),
            "no public mutable accessor may exist: {line}",
        );
    }
}

#[test]
fn no_internal_analysis_container_is_re_exported() {
    // The analysis containers live in unexported modules, so this is a
    // second lock rather than the only one: a re-export added later
    // would make one nameable from outside, and this census is what
    // fails first.
    let block = PLAN_SOURCE
        .split("pub use crate::{")
        .nth(1)
        .and_then(|rest| rest.split("\n};").next())
        .expect("the module's re-export block");

    for container in [
        "ScopedAnalyzedProgram",
        "AnalyzedProofPlan",
        "AnalyzedOperation",
        "AnalyzedSource",
        "RelationCaseRequirements",
        "OperationPlacementAnalysis",
        "PlacementCandidate",
        "CoverageGraphProjection",
        "PlanCoverageAnalysis",
        "CompilerRelationAnalysis",
    ] {
        assert!(
            !block.contains(container),
            "{container} must not be re-exported",
        );
    }
}
