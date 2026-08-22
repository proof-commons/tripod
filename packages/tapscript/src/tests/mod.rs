//! Adapter tests and their shared target fixtures.
//!
//! # A degraded contract is never a reviewed one
//!
//! The reviewed contract is the only input the public assessment API
//! takes, and the target package alone constructs that wrapper. So the
//! non-weakening fixtures below cannot produce a reviewed value at all:
//! a contract with a capability downgraded is by construction not the
//! reviewed Elements contract, and `validate_as_reviewed_elements`
//! rejects it — which is exactly the guarantee wave 0A added and which
//! [`public_api_tests`] checks rather than assumes.
//!
//! They therefore assess through
//! [`crate::capability::assess_validated_capability`], the crate-private
//! core that the public entry point also calls. That keeps one mapping
//! rather than two that could drift, and it cannot leak: the function is
//! `pub(crate)`, no public re-export names it, and the public entry
//! points take the reviewed wrapper by type. Adding a public assessment
//! over a generic definition would be the backdoor this arrangement
//! exists to refuse.
//!
//! # Mutated targets are built through the target package's own API
//!
//! Every fixture below reaches a modified target the way an external
//! consumer would: read the reviewed contract's parts back out, change
//! one of them, and hand the result to the target package's validator.
//! Nothing here reaches inside `target-elements`, and no test-only
//! constructor was added there to make a mutation convenient. A backdoor
//! into a validator is a backdoor into the validator's guarantee, and
//! the guarantee is the reason the fixture is worth building.
//!
//! Two mutations a reader might expect are therefore absent, because
//! the target validator makes them unconstructible: a capability
//! removed from the registry, and an evidence requirement removed from
//! the registry. Both censuses must be complete for a definition to
//! validate at all. The reachable neighbour of each — downgrading a
//! capability's reviewed status — is used instead, and the tests say so
//! where it matters.

mod abstract_oracle_tests;
mod authorization_tests;
mod bundle_tests;
mod byte_census_tests;
mod census_tests;
mod guide12_reproductions;
mod guide13_reproductions;
mod mapping_tests;
mod non_weakening_tests;
mod operation_assessment_tests;
mod parser_tests;
mod pattern_tests;
mod policy_tests;
mod public_api_tests;
mod push_census_tests;
mod schedule_tests;
mod shape_tests;
mod stack_tests;

use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::operation_plan::{
    PlacementSearchLimits, ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
};
use realization::{RealizationScope, derive};
use target_elements::{
    CapabilityContract, ConfidentialCapabilityState, ConfidentialValueCapability,
    ConfidentialValueContract, ElementsCapability, ReviewedElementsTapscriptDefinition,
    StaticCapabilityStatus, TargetDefinition, TargetDefinitionParts, ValidatedTargetDefinition,
    reviewed_elements_tapscript, status_closure_violations, validate_target_definition,
};

/// Placeholder link-time symbols, shared by the pattern schedules and
/// the operation assessment.
///
/// Distinguishable byte strings standing in for symbols a later wave
/// resolves, not claims about any real object. Test material in the
/// sense `(´[ADR015-rule:security:test-material]´)` fixes: public,
/// meaningless, and never a key.
fn pattern_symbols(
    target: &ReviewedElementsTapscriptDefinition,
) -> crate::pattern::CompactAshSymbols {
    crate::pattern::CompactAshSymbols::new(
        target,
        vec![0x11; 32],
        vec![0x22; 32],
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .expect("the placeholder symbols are the reviewed widths")
}

/// The validated compact-ASH plan, from the compiler's own constructor.
///
/// One fixture for every test that needs a plan: §8.1 assesses what the
/// compiler actually required, and two fixtures could disagree about
/// what that was.
///
/// Derived once and handed out by clone. The derivation is the whole
/// scoped analysis, its validator, the Phase-4 filter, and an
/// independent re-derivation of the assembled plan; running it once per
/// test cost this crate's lane about three seconds of the wall time it
/// is supposed to be accounting for. A clone of the finished value is
/// the same value by construction, so nothing about what is tested
/// changes — only how many times the compiler is asked for it.
fn compact_ash_plan() -> ValidatedTargetOperationPlan {
    static PLAN: LazyLock<ValidatedTargetOperationPlan> = LazyLock::new(derive_compact_ash_plan);
    PLAN.clone()
}

/// The one derivation behind [`compact_ash_plan`].
fn derive_compact_ash_plan() -> ValidatedTargetOperationPlan {
    let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
    let realization =
        derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
    let scope = CompilationScope::from_operations([OperationId::CompactAsh])
        .expect("a one-operation scope");
    let policy = AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
    let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

    plan_compact_ash_target_operation(
        &input,
        PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
    )
    .expect("the plan validates")
}

/// The reviewed contract, unmodified.
fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// The reviewed contract's validated view, unmodified.
///
/// The baseline the degraded fixtures are compared against, assessed
/// through the same crate-private path they are, so a difference
/// between two assessments is a difference in the contract rather than
/// in the entry point.
fn reviewed_definition() -> ValidatedTargetDefinition {
    reviewed_target().into_validated()
}

/// The reviewed contract with one capability's reviewed status changed.
///
/// The closest reachable analogue of "remove one target prerequisite":
/// the registry must stay complete, so the prerequisite stops being
/// established rather than stops existing. The result is a validated
/// definition and can be nothing more — a downgraded contract is not the
/// reviewed Elements contract, and the target package refuses to say
/// otherwise.
fn target_with_status(
    capability: ElementsCapability,
    status: StaticCapabilityStatus,
) -> ValidatedTargetDefinition {
    target_with_statuses(&[(capability, status)])
}

/// The two capability rows that carry one confidential-value claim.
const VALUE_INSPECTION_PAIR: &[ElementsCapability] = &[
    ElementsCapability::InputValueInspection,
    ElementsCapability::OutputValueInspection,
];

/// Rewrites one capability row's status in place.
fn restate(
    capabilities: &mut BTreeMap<ElementsCapability, CapabilityContract>,
    capability: ElementsCapability,
    status: StaticCapabilityStatus,
) {
    let previous = capabilities[&capability].clone();
    capabilities.insert(
        capability,
        CapabilityContract::new(
            previous.capability(),
            previous.prerequisites().iter().copied(),
            previous.opcodes().iter().copied(),
            previous.encodings().iter().copied(),
            previous.evidence().iter().copied(),
            status,
        ),
    );
}

/// The confidential-value claim state a capability status implies.
const fn implied_state(status: StaticCapabilityStatus) -> ConfidentialCapabilityState {
    match status {
        StaticCapabilityStatus::Reviewed => ConfidentialCapabilityState::PrimitiveReviewed,
        StaticCapabilityStatus::Incomplete => ConfidentialCapabilityState::ExternalConsensusClaim,
        // The status enum is non-exhaustive, so unsupported is reached
        // through the wildcard along with anything added later. A
        // fixture erring toward the weakest claim is the safe default.
        _ => ConfidentialCapabilityState::Unsupported,
    }
}

/// The reviewed contract with several capabilities' statuses changed.
fn target_with_statuses(
    changes: &[(ElementsCapability, StaticCapabilityStatus)],
) -> ValidatedTargetDefinition {
    let reviewed = reviewed_definition();
    let contract = reviewed.definition();

    let mut capabilities: BTreeMap<_, _> = contract.capabilities().clone();

    for (capability, status) in changes {
        let previous = capabilities
            .get(capability)
            .expect("a validated contract's capability registry is complete");
        let restated = CapabilityContract::new(
            previous.capability(),
            previous.prerequisites().iter().copied(),
            previous.opcodes().iter().copied(),
            previous.encodings().iter().copied(),
            previous.evidence().iter().copied(),
            *status,
        );
        capabilities.insert(*capability, restated);
    }

    // Two target-package rules now constrain what a lone downgrade can
    // mean, and both have to be honoured here or the mutated contract
    // does not validate at all. Status is closed over the prerequisite
    // relation, so a downgrade carries its dependents down with it; and
    // input and output value inspection are two rows of one
    // confidential-value claim, so they move together. Settling both to
    // a fixed point keeps this fixture mechanical: it still says "this
    // capability stops being established", and now says it about
    // everything that stood on it.
    loop {
        let mut moved = false;

        let weakest = VALUE_INSPECTION_PAIR
            .iter()
            .map(|capability| capabilities[capability].status())
            .min_by_key(|status| status.strength())
            .expect("the pair is not empty");
        for capability in VALUE_INSPECTION_PAIR {
            if capabilities[capability].status() != weakest {
                restate(&mut capabilities, *capability, weakest);
                moved = true;
            }
        }

        for (capability, prerequisite) in status_closure_violations(&capabilities) {
            let bound = capabilities[&prerequisite].status();
            restate(&mut capabilities, capability, bound);
            moved = true;
        }

        if !moved {
            break;
        }
    }

    // The confidential-value claim states describe the same facts as
    // those capability rows, so they are re-derived rather than left
    // asserting the reviewed answer over a downgraded registry.
    let source = contract.confidential_values();
    let confidential_values = ConfidentialValueContract::new(
        [
            (
                ConfidentialValueCapability::ConsensusValueConservation,
                ElementsCapability::ConfidentialValueConservation,
            ),
            (
                ConfidentialValueCapability::CommitmentEquality,
                ElementsCapability::CommitmentEquality,
            ),
            (
                ConfidentialValueCapability::ExplicitValueInspection,
                ElementsCapability::ExplicitValueInspection,
            ),
            (
                ConfidentialValueCapability::ConfidentialValueInspection,
                ElementsCapability::InputValueInspection,
            ),
            (
                ConfidentialValueCapability::AuthenticatedOpening,
                ElementsCapability::AuthenticatedValueOpening,
            ),
        ]
        .map(|(claim, capability)| (claim, implied_state(capabilities[&capability].status()))),
        source.participating_encodings().iter().copied(),
        source.evidence().iter().copied(),
    );

    validate_target_definition(TargetDefinition::new(TargetDefinitionParts {
        version: contract.version(),
        execution_domain: contract.execution_domain(),
        leaf_version: contract.leaf_version(),
        opcodes: contract.opcodes().clone(),
        encodings: contract.encodings().clone(),
        pushes: contract.pushes().clone(),
        authorization: contract.authorization().clone(),
        confidential_values,
        issuance: contract.issuance().clone(),
        resources: contract.resources().clone(),
        capabilities,
        evidence_requirements: contract.evidence_requirements().clone(),
    }))
    .expect("restating a reviewed status leaves the contract structurally valid")
}
