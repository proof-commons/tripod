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
        [
            ExternalEvidenceRole::ConfidentialValueConservation,
            ExternalEvidenceRole::SubstrateConservation,
            ExternalEvidenceRole::OperatorAuthorization,
        ],
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

    // Both roles, because the pilot scope's live transfer is planned
    // under a private representation as well as an explicit one, and
    // the requirement set is the union over the retained alternatives.
    assert_eq!(
        evidence,
        [
            ExternalEvidenceRole::ConfidentialValueConservation,
            ExternalEvidenceRole::SubstrateConservation,
        ],
    );
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

// --- Guide-13 §8.3: the public live-transfer target-operation plan ---

use compiler::live_transfer_plan::{
    DeferredRepresentation, LiveTransferClause, LiveTransferComposition,
    LiveTransferRepresentationPlan, RepresentationDeferralGround,
    ValidatedLiveTransferOperationPlan, plan_live_transfer_target_operation,
};

/// The live-transfer plan module's own source, read as an external
/// consumer sees the crate rather than as the crate sees itself.
const LIVE_TRANSFER_SOURCE: &str = include_str!("../src/live_transfer_plan.rs");

fn live_transfer_plan(operations: &[OperationId]) -> ValidatedLiveTransferOperationPlan {
    let scope = CompilationScope::from_operations(operations.iter().copied()).expect("pilot scope");
    let input = bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        scope,
        test_policy(),
    )
    .expect("bind input");

    plan_live_transfer_target_operation(&input, placement_limits()).expect("pilot plan")
}

#[test]
fn the_live_transfer_plan_comes_only_from_a_completed_analysis() {
    // There is no public constructor, no `Default`, and no builder: this
    // call is the only route to the type, and it runs the complete
    // analysis, that analysis's own validator, the §5 contract checks,
    // the §8.2 representation filter, and the plan's independent
    // assembly validator before returning. A plan assembled from
    // arbitrary fields would be a request rather than an analysis, and
    // nothing downstream could tell the two apart once they shared a
    // type.
    let plan = live_transfer_plan(&[OperationId::CompactAsh, OperationId::TransferLive]);

    assert_eq!(plan.operation(), OperationId::TransferLive);
    assert_eq!(plan.representations().count(), 2);

    for projection in plan.representations() {
        // The relation and case censuses agree across the two
        // representations, as §6.6 requires. The layout and coverage
        // censuses do not, as §19.4 requires: the private plan places no
        // carrier for conservation, so it states neither the layout that
        // would route family totals to one nor that carrier's runtime
        // coverage, and it answers at the evidence boundary instead.
        let (layout, coverage, evidence) = match projection.plan() {
            LiveTransferRepresentationPlan::Explicit => {
                (69, 223, vec![ExternalEvidenceRole::SubstrateConservation])
            }
            LiveTransferRepresentationPlan::PrivateCommitted => (
                61,
                225,
                vec![
                    ExternalEvidenceRole::ConfidentialValueConservation,
                    ExternalEvidenceRole::SubstrateConservation,
                ],
            ),
        };

        assert_eq!(projection.relations().count(), 24);
        assert_eq!(projection.cases().count(), 2);
        assert_eq!(projection.layout().count(), layout);
        assert_eq!(projection.coverage().count(), coverage);
        assert_eq!(projection.external_evidence().collect::<Vec<_>>(), evidence,);
        assert!(projection.carriers().count() > 0);
    }
}

#[test]
fn an_incomplete_analysis_publishes_no_live_transfer_plan() {
    let scope =
        CompilationScope::from_operations([OperationId::TransferLive]).expect("pilot scope");
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
        plan_live_transfer_target_operation(&input, truncated).is_err(),
        "a truncated search is a typed failure, never a smaller plan",
    );
}

#[test]
fn declaration_permutations_produce_equal_live_transfer_projections() {
    let first = live_transfer_plan(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let permuted = live_transfer_plan(&[OperationId::TransferLive, OperationId::CompactAsh]);

    assert_eq!(first, permuted);
    assert_eq!(
        first,
        live_transfer_plan(&[OperationId::CompactAsh, OperationId::TransferLive]),
    );
}

#[test]
fn the_plan_admits_both_representations_and_states_its_deferrals() {
    let plan = live_transfer_plan(&[OperationId::TransferLive]);
    let policy = plan.representation();

    // §6.1: the two admitted plans, checked against the realization's
    // own approved set rather than against the policy's own choices.
    assert_eq!(
        policy.admitted().collect::<Vec<_>>(),
        [
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ],
    );
    assert_eq!(
        policy.approved().collect::<Vec<_>>(),
        [
            realization::RepresentationMode::Explicit,
            realization::RepresentationMode::PrivateCommitted,
        ],
    );

    // §6.1, §6.5: what Guide 13 defers is data with a ground, not
    // silence — and a consumer can read both without leaving the type.
    assert_eq!(
        policy.deferral(DeferredRepresentation::PublicCommittedMode),
        Some(RepresentationDeferralGround::UnapprovedMode),
    );
    assert_eq!(
        policy.deferral(DeferredRepresentation::MixedComposition),
        Some(RepresentationDeferralGround::NoPerReferenceVariable),
    );

    // §1.12: the candidate is lifecycle-incomplete and says which exits
    // are missing.
    assert!(!plan.lifecycle().release_complete());
    assert_eq!(
        plan.lifecycle().implemented().collect::<Vec<_>>(),
        [OperationId::TransferLive],
    );
    assert_eq!(
        plan.lifecycle().outstanding().collect::<Vec<_>>(),
        [OperationId::Redeem, OperationId::Burn],
    );
}

#[test]
fn the_live_transfer_censuses_are_complete_and_duplicate_free() {
    // Independently written expectations, not folds over `ALL`: a census
    // compared only with itself agrees with itself. `census_enum!`
    // generates `ALL` from the enum, so drift between the two is
    // unwriteable; these literals pin the membership, and adding,
    // removing, or reordering a member is a visible test change.
    assert_eq!(
        LiveTransferRepresentationPlan::ALL,
        [
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ],
    );
    assert_eq!(
        DeferredRepresentation::ALL,
        [
            DeferredRepresentation::PublicCommittedMode,
            DeferredRepresentation::MixedComposition,
        ],
    );
    assert_eq!(
        RepresentationDeferralGround::ALL,
        [
            RepresentationDeferralGround::UnapprovedMode,
            RepresentationDeferralGround::NoPerReferenceVariable,
        ],
    );
    // §6.5's separate admission, and the ORDER is part of the claim:
    // the two homogeneous members come first and in the plan census's
    // own order, because this type's derived `Ord` breaks taptree ties
    // and reordering it would move tree roots that recorded fixture
    // digests were taken over.
    assert_eq!(
        LiveTransferComposition::ALL,
        [
            LiveTransferComposition::HomogeneousExplicit,
            LiveTransferComposition::HomogeneousPrivate,
            LiveTransferComposition::EntryBlinding,
            LiveTransferComposition::ExitUnblinding,
        ],
    );
    assert_eq!(
        LiveTransferClause::ALL.len(),
        19,
        "the §5 clause census is the sentences the contract check pins",
    );

    for census in [
        LiveTransferClause::ALL
            .iter()
            .map(|member| format!("{member:?}"))
            .collect::<Vec<_>>(),
        LiveTransferRepresentationPlan::ALL
            .iter()
            .map(|member| format!("{member:?}"))
            .collect::<Vec<_>>(),
        DeferredRepresentation::ALL
            .iter()
            .map(|member| format!("{member:?}"))
            .collect::<Vec<_>>(),
        LiveTransferComposition::ALL
            .iter()
            .map(|member| format!("{member:?}"))
            .collect::<Vec<_>>(),
    ] {
        assert_eq!(
            census.iter().collect::<BTreeSet<_>>().len(),
            census.len(),
            "a census that names one member twice is not a census",
        );
    }
}

#[test]
fn the_admitted_compositions_pair_the_two_plans_and_name_their_own_crossings() {
    // §6.5's separate admission read back as arithmetic rather than as
    // membership. Every claim below is recomputed from the sides, so a
    // member whose sides disagreed with its own name would fail here
    // rather than be documented into agreement.

    // Every composition's sides are drawn from the two-member plan
    // census, and every ordered pairing of that census appears exactly
    // once. That is the completeness claim: crossing admits a PAIRING
    // of the plans that exist and never a third representation.
    let pairs = LiveTransferComposition::ALL
        .iter()
        .map(|composition| (composition.consumed(), composition.created()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        pairs.len(),
        LiveTransferComposition::ALL.len(),
        "two compositions pairing the same two sides would be one composition",
    );
    assert_eq!(
        pairs.len(),
        LiveTransferRepresentationPlan::ALL.len() * LiveTransferRepresentationPlan::ALL.len(),
        "the compositions are every ordered pairing of the admitted plans",
    );

    // The homogeneous constructor agrees with the sides, in both
    // directions, so the two ways of naming a homogeneous composition
    // cannot drift apart.
    for plan in LiveTransferRepresentationPlan::ALL.iter().copied() {
        let composition = LiveTransferComposition::homogeneous(plan);
        assert_eq!(composition.consumed(), plan);
        assert_eq!(composition.created(), plan);
        assert!(!composition.crosses());
        assert_eq!(
            composition.conservation_capability(),
            plan.conservation_capability()
        );
    }

    // Exactly two members cross, they are the two crossings the two
    // plans can form, and each crosses in its own direction.
    let crossing = LiveTransferComposition::ALL
        .iter()
        .copied()
        .filter(|composition| composition.crosses())
        .collect::<Vec<_>>();
    assert_eq!(
        crossing,
        vec![
            LiveTransferComposition::EntryBlinding,
            LiveTransferComposition::ExitUnblinding,
        ],
    );
    assert_eq!(
        LiveTransferComposition::EntryBlinding.consumed(),
        LiveTransferRepresentationPlan::Explicit,
    );
    assert_eq!(
        LiveTransferComposition::EntryBlinding.created(),
        LiveTransferRepresentationPlan::PrivateCommitted,
    );
    assert_eq!(
        LiveTransferComposition::ExitUnblinding.consumed(),
        LiveTransferRepresentationPlan::PrivateCommitted,
    );
    assert_eq!(
        LiveTransferComposition::ExitUnblinding.created(),
        LiveTransferRepresentationPlan::Explicit,
    );

    // The absorber is declared by the ONE composition whose consumed
    // side presents a blinder sum its created side cannot otherwise
    // carry, and by no other. Recomputed from the sides rather than
    // read off the member, so the two cannot drift.
    for composition in LiveTransferComposition::ALL.iter().copied() {
        let consumes_commitments =
            composition.consumed() == LiveTransferRepresentationPlan::PrivateCommitted;
        let creates_explicit = composition.created() == LiveTransferRepresentationPlan::Explicit;
        assert_eq!(
            composition.declares_an_absorber(),
            consumes_commitments && creates_explicit,
            "{composition:?} declares an absorber exactly when a nonzero consumed blinder \
             sum meets a created side that contributes zero to it",
        );
    }

    // Only the wholly explicit composition may prove conservation
    // first-party. Both crossings take the target's own rule, because
    // each can read one total and not the other.
    for composition in LiveTransferComposition::ALL.iter().copied() {
        let first_party = composition.conservation_capability()
            == RequiredCapability::ExactPublicAmountArithmetic;
        assert_eq!(
            first_party,
            composition == LiveTransferComposition::HomogeneousExplicit,
            "{composition:?} may prove conservation first-party only if BOTH its sides are \
             explicit",
        );
    }
}

#[test]
fn no_sponsor_amount_enters_any_public_live_transfer_field() {
    // Structural rather than textual, under *every* admitted
    // representation: §1.9 does not weaken because a plan changed how it
    // proves conservation.
    //
    // The *amount* is what is erased. Sponsor membership, cardinality,
    // isolation, and envelope multiplicity are required relations of
    // §5.8, so an authenticated census of the ordinary L-BTC family is
    // expected here and is not a leak — and the sponsor projection
    // publishes exactly those and no amount-shaped field at all.
    let plan = live_transfer_plan(&[OperationId::CompactAsh, OperationId::TransferLive]);
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

    for projection in plan.representations() {
        for requirement in projection.layout() {
            assert!(!names_sponsor_amount(requirement), "{requirement:?}");
        }

        for source in projection
            .relations()
            .flat_map(|relation| relation.source_requirements.iter())
            .chain(
                projection
                    .relations()
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

    assert_eq!(plan.sponsor().object(), ORDINARY_LBTC);
    assert_eq!(
        plan.sponsor().envelope_maximum(),
        realization::Count::ONE,
        "the envelope census is a count, never an amount",
    );
}

#[test]
fn the_public_live_transfer_surface_carries_no_digest_and_no_graph_handle() {
    // §1.13 mints no plan digest and reserves no field for a future one;
    // §8.1 admits no Petgraph index. Both are properties of the
    // published source rather than of any one value, so the source is
    // what is checked — a field added later fails here even if no test
    // happens to read it.
    //
    // Comment lines are excluded deliberately. The module documents that
    // it carries no digest, and a scan that could not tell the
    // prohibition from a violation would forbid saying so.
    let code = LIVE_TRANSFER_SOURCE
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

    // §1.10: the private-committed plan states which proof a target must
    // supply. It is not an interface handling the material that proof is
    // built from, and no field here could hold such material.
    for banned in ["blinder", "Blinder", "opening", "Opening", "PrivateKey"] {
        assert!(
            !code.contains(banned),
            "the published plan surface must not name {banned}",
        );
    }

    // The corruption handles the in-crate oracles need are compiled out
    // of every non-test build, so no mutable route to a published field
    // survives into a consumer's copy of this crate.
    for line in LIVE_TRANSFER_SOURCE.lines() {
        let line = line.trim_start();

        assert!(
            !(line.starts_with("pub fn") || line.starts_with("pub const fn"))
                || !line.contains("_mut("),
            "no public mutable accessor may exist: {line}",
        );
    }
}

#[test]
fn no_internal_analysis_container_is_re_exported_by_the_live_transfer_plan() {
    let block = LIVE_TRANSFER_SOURCE
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

// --- The public maturity-announcement target-operation plan ---

use compiler::maturity_announcement_plan::{
    ValidatedMaturityAnnouncementOperationPlan, plan_maturity_announcement_target_operation,
};

const MATURITY_ANNOUNCEMENT_SOURCE: &str = include_str!("../src/maturity_announcement_plan.rs");
const ANNOUNCEMENT_OPERATIONS: [OperationId; 3] = [
    OperationId::AnnounceMaturity,
    OperationId::CompactAsh,
    OperationId::TransferLive,
];

fn maturity_announcement_input(operations: &[OperationId]) -> compiler::BoundCompilerInput {
    let realization_scope = realization::RealizationScope::from_operations(ANNOUNCEMENT_OPERATIONS)
        .expect("announcement realization scope");
    let realization = realization::derive(&architecture::ARCHITECTURE, realization_scope)
        .expect("announcement realization");
    let scope =
        CompilationScope::from_operations(operations.iter().copied()).expect("compiler scope");

    bind_input(
        &architecture::ARCHITECTURE,
        realization,
        scope,
        test_policy(),
    )
    .expect("bind input")
}

fn maturity_announcement_code() -> String {
    MATURITY_ANNOUNCEMENT_SOURCE
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !(line.starts_with("//") || line.starts_with("///") || line.starts_with("//!"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn maturity_announcement_plan(
    operations: &[OperationId],
) -> ValidatedMaturityAnnouncementOperationPlan {
    plan_maturity_announcement_target_operation(
        &maturity_announcement_input(operations),
        placement_limits(),
    )
    .expect("announcement plan")
}

#[test]
fn the_maturity_announcement_plan_comes_only_from_a_completed_analysis() {
    let plan = maturity_announcement_plan(&[OperationId::AnnounceMaturity]);

    assert_eq!(plan.operation(), OperationId::AnnounceMaturity);
    assert_eq!(plan.representations().count(), 2);
    for projection in plan.representations() {
        assert_eq!(projection.relations().count(), 26);
        assert_eq!(projection.cases().count(), 2);
        assert_eq!(projection.carriers().count(), 28);
        assert_eq!(projection.layout().count(), 59);
    }
}

#[test]
fn an_incomplete_analysis_publishes_no_maturity_announcement_plan() {
    let input = maturity_announcement_input(&[OperationId::AnnounceMaturity]);
    let truncated = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1).expect("nonzero"),
        std::num::NonZeroU64::new(1).expect("nonzero"),
    );
    let error = plan_maturity_announcement_target_operation(&input, truncated).unwrap_err();

    assert_eq!(
        error,
        CompileError::PlacementSearchStateLimitExceeded { maximum: 1 },
    );
    assert_eq!(error.to_string(), "placement search exceeded 1 states");
}

#[test]
fn a_scope_without_announcement_publishes_no_partial_plan() {
    let input = maturity_announcement_input(&[OperationId::CompactAsh]);
    let error =
        plan_maturity_announcement_target_operation(&input, placement_limits()).unwrap_err();

    assert_eq!(
        error,
        CompileError::TargetOperationOutOfScope {
            operation: OperationId::AnnounceMaturity,
        },
    );
    assert!(error.to_string().contains("out-of-scope"));
}

#[test]
fn scope_permutations_and_equal_inputs_produce_equal_announcement_plans() {
    let first = maturity_announcement_plan(&ANNOUNCEMENT_OPERATIONS);
    let permuted = maturity_announcement_plan(&[
        OperationId::TransferLive,
        OperationId::AnnounceMaturity,
        OperationId::CompactAsh,
    ]);

    assert_eq!(first, permuted);
    assert_eq!(first, maturity_announcement_plan(&ANNOUNCEMENT_OPERATIONS),);
}

#[test]
fn the_announcement_representation_and_clause_censuses_are_literal() {
    use compiler::maturity_announcement_plan::{
        MaturityAnnouncementClause as Clause, MaturityAnnouncementRepresentationPlan as Mode,
    };
    use realization::{Relation, RepresentationMode};

    let input = maturity_announcement_input(&[OperationId::AnnounceMaturity]);
    let plan = plan_maturity_announcement_target_operation(&input, placement_limits()).unwrap();
    let approved = input
        .realization()
        .operation(OperationId::AnnounceMaturity)
        .unwrap()
        .relations
        .iter()
        .find_map(|row| match &row.relation {
            Relation::Representation { object, allowed }
                if *object == architecture::ObjectId::State =>
            {
                Some(allowed)
            }
            _ => None,
        })
        .expect("STATE representation declaration");

    assert_eq!(Mode::ALL, [Mode::Explicit, Mode::PublicCommitted],);
    assert_eq!(
        plan.representation().admitted(),
        &BTreeSet::from([Mode::Explicit, Mode::PublicCommitted]),
    );
    assert_eq!(
        plan.representation().approved(),
        &BTreeSet::from([
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ]),
    );
    assert_eq!(plan.representation().approved(), approved);
    assert_eq!(
        Clause::ALL,
        [
            Clause::Declaration,
            Clause::ProtocolObject,
            Clause::ProtocolCardinality,
            Clause::ClassClosure,
            Clause::OperatorAuthorization,
            Clause::CanonicalDelta,
            Clause::SponsorObject,
            Clause::SponsorFlow,
            Clause::SponsorIsolation,
            Clause::SponsorEnvelope,
            Clause::SubstrateConservation,
            Clause::SponsorInputs,
            Clause::SponsorChange,
            Clause::RootPolicy,
            Clause::CertificateProjection,
            Clause::RepresentationApproval,
            Clause::LifecycleExits,
            Clause::PublicFacts,
            Clause::Transition,
        ],
    );
}

#[test]
fn the_announcement_lifecycle_closure_is_literal() {
    let plan = maturity_announcement_plan(&[OperationId::AnnounceMaturity]);

    assert_eq!(
        plan.lifecycle().implemented().collect::<Vec<_>>(),
        [OperationId::AnnounceMaturity],
    );
    assert_eq!(
        plan.lifecycle().outstanding().collect::<Vec<_>>(),
        [
            OperationId::AdmitDeposits,
            OperationId::Cycle,
            OperationId::Redeem,
            OperationId::ReceiptRelabel,
            OperationId::Clear,
        ],
    );
    assert!(!plan.lifecycle().release_complete());
}

#[test]
fn the_announcement_state_closure_and_empty_canonical_projection_are_literal() {
    use architecture::ObjectId;
    use realization::{RelationId, RelationKind, RelationSubject, TransactionSide};

    let plan = maturity_announcement_plan(&[OperationId::AnnounceMaturity]);
    let relation = |side| {
        RelationId::new(
            OperationId::AnnounceMaturity,
            RelationKind::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
        )
    };

    assert_eq!(
        plan.state().input_closure(),
        &relation(TransactionSide::Input)
    );
    assert_eq!(
        plan.state().output_closure(),
        &relation(TransactionSide::Output)
    );
    assert_eq!(
        plan.state().admitted(),
        &BTreeSet::from([ObjectId::State, ObjectId::PlainLbtc,]),
    );
    assert_eq!(
        plan.state().forbidden(),
        &BTreeSet::from([
            ObjectId::Resv,
            ObjectId::Pace,
            ObjectId::EntitlementAuthority,
            ObjectId::DistributionAuthority,
            ObjectId::ReceiptLive,
            ObjectId::ReceiptTimeLocked,
            ObjectId::DepositRequest,
            ObjectId::DepositEntitlement,
            ObjectId::DistributionControl,
            ObjectId::DistributionVault,
            ObjectId::Ash,
            ObjectId::CpfpAnchor,
        ]),
    );
    assert!(plan.canonical().expected().is_empty());
}

#[test]
fn validated_announcement_types_have_no_public_construction_route() {
    let code = maturity_announcement_code();

    for prefix in ["pub fn new(", "pub const fn new(", "pub fn default("] {
        assert!(
            !code
                .lines()
                .map(str::trim_start)
                .any(|line| line.starts_with(prefix))
        );
    }
    for banned in ["impl Default for", "Builder", "builder"] {
        assert!(
            !code.contains(banned),
            "validated plan must not expose {banned}"
        );
    }

    let re_exports = MATURITY_ANNOUNCEMENT_SOURCE
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
            !re_exports.contains(container),
            "{container} must not be re-exported"
        );
    }
}

#[test]
fn no_announcement_value_or_target_handle_reaches_the_public_surface() {
    let plan = maturity_announcement_plan(&[OperationId::AnnounceMaturity]);
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

    for projection in plan.representations() {
        assert!(!projection.layout().any(names_sponsor_amount));
        assert!(projection.relations().all(|relation| {
            relation
                .source_requirements
                .iter()
                .chain(
                    relation
                        .cases
                        .values()
                        .flat_map(|case| &case.active_sources),
                )
                .all(|source| {
                    !matches!(
                        source.operand.role(),
                        OperandRole::ObjectFamilyAmount {
                            object: ORDINARY_LBTC,
                            ..
                        },
                    )
                })
        }));
    }

    let code = maturity_announcement_code();
    for banned in [
        "digest",
        "Digest",
        "NodeIndex",
        "EdgeIndex",
        "petgraph",
        "DiGraph",
        "PlanHash",
        "TransactionPosition",
        "transaction_position",
        "output_index",
        "input_index",
        "MetadataBytes",
        "metadata_bytes",
        "metadata_byte_count",
        "METADATA_SIZE",
        "SponsorAmount",
        "sponsor_amount:",
        "sponsor_amount(&self",
        "PrivateKey",
        "Blinder",
        "target_encoding",
        "codec_constant",
    ] {
        assert!(!code.contains(banned), "public plan surface names {banned}");
    }
}

#[test]
fn the_public_announcement_validator_accepts_every_derived_plan() {
    for operations in [
        &[OperationId::AnnounceMaturity][..],
        &ANNOUNCEMENT_OPERATIONS[..],
    ] {
        let input = maturity_announcement_input(operations);
        let plan = plan_maturity_announcement_target_operation(&input, placement_limits()).unwrap();

        compiler::validate_maturity_announcement_plan(&plan, &input).unwrap();
    }
    let one = maturity_announcement_input(&[OperationId::AnnounceMaturity]);
    let all = maturity_announcement_input(&ANNOUNCEMENT_OPERATIONS);
    let plan = plan_maturity_announcement_target_operation(&one, placement_limits()).unwrap();

    assert_eq!(
        compiler::validate_maturity_announcement_plan(&plan, &all),
        Err(CompileError::TargetPlanSourceMismatch),
    );
}

#[test]
fn announcement_requirement_vocabulary_is_public_without_a_validated_plan() {
    use compiler::{
        AnnouncementMetadataRequirement, PublicRecoveryRequirement, StateFieldLawKind,
        StateLawOperand, StateSuccessionRequirement,
    };
    use realization::{AnnouncementLeadBound, FactId, StateField, TransactionSide};
    let metadata = AnnouncementMetadataRequirement::required();
    let facts = AnnouncementMetadataRequirement::public_facts();
    assert_eq!(facts.len(), 15);
    assert_eq!(
        facts
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        15
    );
    let succession = StateSuccessionRequirement::REQUIRED;
    for (side, keys) in [
        (TransactionSide::Input, succession.input_fields.unwrap()),
        (TransactionSide::Output, succession.output_fields.unwrap()),
    ] {
        let expected = StateField::ALL
            .iter()
            .map(|field| FactId::StateField {
                operation: OperationId::AnnounceMaturity,
                side,
                field: *field,
            })
            .collect::<Vec<_>>();
        assert_eq!(keys.as_slice(), expected);
        assert!(keys.iter().all(|key| facts.contains(key)));
    }
    assert_eq!(metadata.fields.len(), 6);
    for field in &metadata.fields {
        field.validate().unwrap();
        if field.field == StateField::Maturity {
            assert_eq!(field.law.kind, StateFieldLawKind::AnnounceRequestedCycle);
            assert_eq!(
                field.law.operands,
                [StateLawOperand::Fact(metadata.requested_cycle.clone())]
            );
        } else {
            assert_eq!(field.law.kind, StateFieldLawKind::Copy);
            assert_eq!(field.law.operands, []);
        }
    }
    assert_eq!(
        AnnouncementLeadBound::Minimum.bound_id(),
        architecture::BoundId::MaturityLeadMin
    );
    assert_eq!(
        AnnouncementLeadBound::Maximum.bound_id(),
        architecture::BoundId::MaturityLeadMax
    );
    assert_eq!(PublicRecoveryRequirement::REQUIRED.source_facts.len(), 7);
    assert_eq!(PublicRecoveryRequirement::REQUIRED.result_facts.len(), 6);
}
