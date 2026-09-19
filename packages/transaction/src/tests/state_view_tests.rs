//! The public current-STATE view: its two laws, its one check, and its
//! residual.
//!
//! # The bundle under test is a real one
//!
//! The view's eighth entry is a linked maturity bundle, and the one
//! obtained here is obtained the way an external consumer would: derive
//! the realization, bind the compiler input, plan the announcement
//! operation, compose the announcement record through the backend's
//! public builders, bind the deployment, and link. A hand-assembled
//! bundle would let the reconstruction agree with a policy and a static
//! subtree that no link produced, which is the one thing the check is
//! for.
//!
//! # Why the fixture curve is a function of the root
//!
//! Curve arithmetic is a behaviour the caller supplies, and the stand-in
//! below computes no point sum. What it must do is answer differently for
//! different merkle roots: the check under test recomputes a program from
//! a caller's metadata and nonce and compares it with the supplied one,
//! so a stub answering with one key for every root would make that
//! comparison succeed at every nonce and each negative here would pass
//! while checking nothing.

use std::collections::BTreeMap;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::maturity_announcement_plan::{
    ValidatedMaturityAnnouncementOperationPlan, plan_maturity_announcement_target_operation,
};
use compiler::operation_plan::PlacementSearchLimits;
use linker::{
    CandidateDeploymentIdentity, CandidateLinkedMaturityBundle, OperatorDeploymentBinding,
    StateLeadBoundOrigin, StateLeadBounds, StateLinkDeploymentParameters, StateLinkSources,
    StateSingletonAsset, link_state_candidate,
};
use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, Maturity, ProtocolAmount,
    RealizationScope, StateMetadata, StateRepresentationNonce, StateSingletonDeclaration, derive,
};
use sha2::{Digest, Sha256};
use tapscript::{
    CandidateStateConstructor, EstablishedOperatorProfile, OperatorKey, STATE_NUMS_KEY, StackItem,
    StateAnnouncementBindings, StateAnnouncementProgram, StateAnnouncementSymbol,
    StateConstructorRefusal, StateCurveCapability, StateInternalKeyPolicy, StateNonceBudget,
    StateOperatorBindings, StateOperatorSymbol, StatePatternBindings, StatePatternSymbol,
    StateTweakOutcome, build_state_announcement_program, build_state_operator_pattern,
    operator_key_encoding_closure, production_static_subtree, selected_operator_profile,
    state_announcement_patterns, state_announcement_program, state_operator_fragment,
    state_output_program_at_nonce, state_structural_patterns,
};
use target_elements::EncodingClass;

use crate::bytes::{AssetField, AssetId, ValueField};
use crate::error::TransactionRefusal;
use crate::operator_right::BranchContext;
use crate::state_view::{
    MaturityViewEntry, MaturityViewResidual, MaturityViewStatement, PublicMaturityStateView,
};

/// A deterministic stand-in for public curve arithmetic.
///
/// Public, meaningless test material: it proves nothing about any curve
/// and holds no scalar. The assertion keeps it from answering for a key
/// it was not asked about, and the digest over the root is what makes a
/// wrong nonce produce a different program rather than the same one.
struct FixtureStateCurve;

impl StateCurveCapability for FixtureStateCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        assert_eq!(key, &STATE_NUMS_KEY);
        true
    }

    fn output_key(&self, key: &[u8; 32], root: &[u8; 32]) -> StateTweakOutcome {
        assert_eq!(key, &STATE_NUMS_KEY);
        let mut hash = Sha256::new();
        hash.update(b"fixture-state-curve");
        hash.update(key);
        hash.update(root);
        let derived: [u8; 32] = hash.finalize().into();
        StateTweakOutcome::OutputKey {
            key: derived,
            parity: (derived[31] & 1) == 1,
        }
    }
}

/// The validated announcement plan, derived once and handed out by clone.
fn plan() -> ValidatedMaturityAnnouncementOperationPlan {
    static PLAN: LazyLock<ValidatedMaturityAnnouncementOperationPlan> = LazyLock::new(|| {
        let limit = |value: u64| NonZeroU64::new(value).expect("the fixture limits are nonzero");
        let operations = [
            OperationId::AnnounceMaturity,
            OperationId::CompactAsh,
            OperationId::TransferLive,
        ];
        let scope = RealizationScope::from_operations(operations).expect("a three-operation scope");
        let realization = derive(&ARCHITECTURE, scope).expect("the operations derive");
        let scope = CompilationScope::from_operations([OperationId::AnnounceMaturity])
            .expect("a one-operation scope");
        let policy =
            AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
        let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

        plan_maturity_announcement_target_operation(
            &input,
            PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
        )
        .expect("the plan validates")
    });
    PLAN.clone()
}

/// The composed announcement record, built once and handed out by clone.
fn record() -> StateAnnouncementProgram {
    static RECORD: LazyLock<StateAnnouncementProgram> = LazyLock::new(|| {
        let target = super::reviewed_target();
        let item = |bytes| StackItem::new(&target, bytes).expect("fixture bytes are a stack item");

        let structural = StatePatternBindings::new(
            &target,
            BTreeMap::from([
                (StatePatternSymbol::StateAsset, item(vec![0x11; 32])),
                (
                    StatePatternSymbol::StateAmount,
                    StackItem::signed_le64(&target, 1),
                ),
            ]),
        )
        .expect("the structural census is complete");
        let structural =
            state_structural_patterns(&target, &structural).expect("the structural recipe builds");

        let semantic = StateAnnouncementBindings::new(
            &target,
            BTreeMap::from([
                (
                    StateAnnouncementSymbol::InternalKey,
                    item(STATE_NUMS_KEY.to_vec()),
                ),
                (
                    StateAnnouncementSymbol::MaturityLeadMin,
                    StackItem::unsigned_le64(&target, 2),
                ),
                (
                    StateAnnouncementSymbol::MaturityLeadMax,
                    StackItem::unsigned_le64(&target, 4),
                ),
                (StateAnnouncementSymbol::StateAsset, item(vec![0x11; 32])),
                (
                    StateAnnouncementSymbol::StateAmount,
                    StackItem::signed_le64(&target, 1),
                ),
            ]),
        )
        .expect("the semantic census is complete");
        let semantic =
            state_announcement_patterns(&target, &semantic).expect("the semantic recipe builds");

        let operator = StateOperatorBindings::new(
            &target,
            &BTreeMap::from([(
                StateOperatorSymbol::CommittedOperatorKey,
                StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0x33; 32])
                    .expect("the fixture key has the reviewed width"),
            )]),
        )
        .expect("the operator census is complete");
        let fragment = state_operator_fragment(&operator).expect("the operator fragment builds");
        let operator = build_state_operator_pattern(&target, &operator, fragment)
            .expect("the operator pattern builds");

        let raw = state_announcement_program(&target, &structural, &semantic, &operator)
            .expect("the composed program assembles");
        build_state_announcement_program(&target, &structural, &semantic, &operator, raw)
            .expect("the composed record is admitted")
    });
    RECORD.clone()
}

/// The semantic metadata the demonstration link is run over.
///
/// The four quantities are pairwise distinct, so a copy-through that
/// swapped two of them would fail rather than pass by coincidence.
fn state_metadata() -> StateMetadata {
    StateMetadata {
        omega: ProtocolAmount::new(1).expect("the fixture quantities are in domain"),
        y_l: ProtocolAmount::new(2).expect("the fixture quantities are in domain"),
        y_t: ProtocolAmount::new(3).expect("the fixture quantities are in domain"),
        q: ProtocolAmount::new(4).expect("the fixture quantities are in domain"),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    }
}

/// The bound deployment sources of the demonstration link.
fn bridge() -> StateLinkDeploymentParameters {
    let target = super::reviewed_target();
    let bounds = AnnouncementLeadBounds::new(Cycle::new(2), Cycle::new(4))
        .expect("the fixture window is nonzero and ordered");
    let closure = operator_key_encoding_closure(target.definition().authorization());
    let key = OperatorKey::new(&closure, closure.approved(), vec![0x33; 32])
        .expect("fixture public bytes have the approved shape");
    let internal_key = StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0xb6; 32])
        .expect("fixture internal key has the reviewed width");
    let profile = EstablishedOperatorProfile::establish(selected_operator_profile(), &target)
        .expect("the reviewed target establishes the source selection");
    let identity = CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32])
        .expect("fixture identifiers are nonzero");
    let binding = OperatorDeploymentBinding::bind(&target, key, profile, identity, &internal_key)
        .expect("the candidate deployment binds");

    StateLinkDeploymentParameters::bind(
        &target,
        plan(),
        StateLeadBounds::new(bounds, StateLeadBoundOrigin::Fixture),
        CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32])
            .expect("fixture identifiers are nonzero"),
        binding,
        NonZeroU32::new(8).expect("eight is nonzero"),
        &record(),
    )
    .expect("the demonstration sources bind")
}

/// The candidate linked maturity bundle, linked once and cloned.
fn linked_bundle() -> CandidateLinkedMaturityBundle {
    static BUNDLE: LazyLock<CandidateLinkedMaturityBundle> = LazyLock::new(|| {
        let target = super::reviewed_target();
        let subtree = production_static_subtree(&target, &record())
            .expect("the composed record yields the production subtree");
        let constructor = CandidateStateConstructor::derive(
            &target,
            &state_metadata(),
            &subtree,
            StateInternalKeyPolicy::new(STATE_NUMS_KEY, &FixtureStateCurve)
                .expect("the reviewed internal key is the derived one"),
            StateNonceBudget::default(),
            &FixtureStateCurve,
        )
        .expect("the demonstration constructor derives");
        let spec = ARCHITECTURE
            .asset(architecture::AssetId::Pid)
            .expect("the identity asset is declared");

        link_state_candidate(
            &target,
            &StateLinkSources::new(
                &record(),
                &bridge(),
                &constructor,
                &StateSingletonAsset::new([0x11; 32]),
                &StateSingletonDeclaration::from_architecture_asset(spec)
                    .expect("the declaration is a singleton"),
                &state_metadata(),
                &FixtureStateCurve,
            ),
        )
        .expect("the demonstration sources link")
    });
    BUNDLE.clone()
}

/// The program the link's own application committed, at its own nonce.
fn linked_pair(bundle: &CandidateLinkedMaturityBundle) -> (StateRepresentationNonce, Vec<u8>) {
    let constructor = bundle.instances()[0].constructor();
    (constructor.nonce(), constructor.output_program())
}

/// The seven statements of a view over the demonstration bundle.
fn statements(
    bundle: &CandidateLinkedMaturityBundle,
    metadata: StateMetadata,
    nonce: StateRepresentationNonce,
    program: Vec<u8>,
) -> Vec<MaturityViewStatement> {
    vec![
        MaturityViewStatement::CurrentStateOutpoint(super::outpoint(0x71, 0)),
        MaturityViewStatement::AssetAndAmount(
            AssetField::Explicit(AssetId::from_internal([0x11; 32])),
            ValueField::Explicit(1),
        ),
        MaturityViewStatement::PredecessorMetadata(metadata),
        MaturityViewStatement::PredecessorRepresentationNonce(nonce),
        MaturityViewStatement::CurrentRootBinding(
            BranchContext::new([0x9b; 32], 12).expect("the fixture branch identifier is nonzero"),
        ),
        MaturityViewStatement::PredecessorProgram(program),
        MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
    ]
}

/// The view the demonstration bundle's own application states.
fn demonstration_view() -> PublicMaturityStateView {
    let bundle = linked_bundle();
    let (nonce, program) = linked_pair(&bundle);
    PublicMaturityStateView::new(statements(&bundle, state_metadata(), nonce, program))
        .expect("the seven statements name distinct entries")
}

/// Commit one metadata and nonce the way the view's check commits them.
fn commit(
    bundle: &CandidateLinkedMaturityBundle,
    metadata: StateMetadata,
    representation: StateRepresentationNonce,
) -> Result<Vec<u8>, StateConstructorRefusal> {
    state_output_program_at_nonce(
        &super::reviewed_target(),
        &EncodedStateMetadata {
            semantic: metadata,
            representation,
        },
        bundle.static_subtree(),
        bundle.policy().internal_key(),
        &FixtureStateCurve,
    )
}

/// The first nonce within the reviewed scan's reach at which `metadata`
/// commits, other than `besides`.
fn nonce_that_commits(
    bundle: &CandidateLinkedMaturityBundle,
    metadata: StateMetadata,
    besides: Option<StateRepresentationNonce>,
) -> StateRepresentationNonce {
    (0..64)
        .map(StateRepresentationNonce::new)
        .find(|nonce| Some(*nonce) != besides && commit(bundle, metadata, *nonce).is_ok())
        .expect("one of the first sixty-four nonces commits")
}

#[test]
fn a_view_over_the_demonstration_bundle_validates_and_names_freshness_as_its_residual() {
    let view = demonstration_view();
    let validated = view
        .validate(&super::reviewed_target(), &FixtureStateCurve)
        .expect("the stated pair reproduces the stated program");
    assert_eq!(
        validated.residual(),
        MaturityViewResidual::CurrentStateRootFreshness
    );
    assert_eq!(validated.view(), &view);
}

#[test]
fn the_entry_census_is_the_guides_ten_entries_in_its_order() {
    let names: Vec<_> = MaturityViewEntry::ALL
        .iter()
        .map(|entry| entry.name())
        .collect();
    assert_eq!(
        names,
        vec![
            "current-state-outpoint",
            "asset-and-amount",
            "predecessor-metadata",
            "predecessor-representation-nonce",
            "current-root-binding",
            "predecessor-program",
            "current-cycle",
            "accepted-linked-bundle",
            "operator-public-identity",
            "deployment-symbols",
        ]
    );
}

#[test]
fn the_three_projected_entries_are_read_from_the_entries_they_project() {
    let bundle = linked_bundle();
    let view = demonstration_view();
    let projected: Vec<_> = MaturityViewEntry::ALL
        .iter()
        .filter_map(|entry| entry.projected_from().map(|source| (*entry, source)))
        .collect();
    assert_eq!(
        projected,
        vec![
            (
                MaturityViewEntry::CurrentCycle,
                MaturityViewEntry::PredecessorMetadata
            ),
            (
                MaturityViewEntry::OperatorPublicIdentity,
                MaturityViewEntry::AcceptedLinkedBundle
            ),
            (
                MaturityViewEntry::DeploymentSymbols,
                MaturityViewEntry::AcceptedLinkedBundle
            ),
        ]
    );
    assert_eq!(view.current_cycle(), state_metadata().cycle);
    assert_eq!(
        view.operator_public_identity(),
        bundle.deployment().operator().key()
    );
    assert_eq!(view.deployment_symbols(), bundle.resolved());
    assert_eq!(view.accepted_linked_bundle(), &bundle);
}

#[test]
fn a_second_statement_of_any_stated_entry_refuses_naming_that_entry() {
    let bundle = linked_bundle();
    let (nonce, program) = linked_pair(&bundle);
    let stated = statements(&bundle, state_metadata(), nonce, program);
    for statement in &stated {
        let mut offered = stated.clone();
        offered.push(statement.clone());
        assert_eq!(
            PublicMaturityStateView::new(offered).unwrap_err(),
            TransactionRefusal::DuplicateMaturityViewEntry(statement.entry())
        );
    }
}

#[test]
fn an_entry_never_stated_refuses_naming_that_entry() {
    let bundle = linked_bundle();
    let (nonce, program) = linked_pair(&bundle);
    let stated = statements(&bundle, state_metadata(), nonce, program);
    for (position, statement) in stated.iter().enumerate() {
        let mut offered = stated.clone();
        offered.remove(position);
        assert_eq!(
            PublicMaturityStateView::new(offered).unwrap_err(),
            TransactionRefusal::MissingMaturityViewEntry(statement.entry())
        );
    }
}

#[test]
fn another_nonce_that_commits_refuses_by_name_with_both_lengths() {
    let bundle = linked_bundle();
    let (selected, program) = linked_pair(&bundle);
    let other = nonce_that_commits(&bundle, state_metadata(), Some(selected));
    let view = PublicMaturityStateView::new(statements(
        &bundle,
        state_metadata(),
        other,
        program.clone(),
    ))
    .expect("the seven statements name distinct entries");
    assert_eq!(
        view.validate(&super::reviewed_target(), &FixtureStateCurve)
            .unwrap_err(),
        TransactionRefusal::MaturityViewProgramNotReconstructed {
            supplied: program.len(),
            recomputed: program.len(),
        }
    );
}

#[test]
fn a_nonce_that_commits_to_nothing_refuses_carrying_the_constructors_own_refusal() {
    let bundle = linked_bundle();
    let program = linked_pair(&bundle).1;
    let (nonce, refusal) = (0..64)
        .map(StateRepresentationNonce::new)
        .find_map(|nonce| {
            commit(&bundle, state_metadata(), nonce)
                .err()
                .map(|refusal| (nonce, refusal))
        })
        .expect("the fixed outer branch side goes unsatisfied at one of the first sixty-four");
    let view = PublicMaturityStateView::new(statements(&bundle, state_metadata(), nonce, program))
        .expect("the seven statements name distinct entries");
    assert_eq!(
        view.validate(&super::reviewed_target(), &FixtureStateCurve)
            .unwrap_err(),
        TransactionRefusal::MaturityViewCommitmentRefused { refusal }
    );
}

#[test]
fn metadata_at_another_cycle_refuses_by_name_and_moves_the_projected_cycle() {
    let bundle = linked_bundle();
    let program = linked_pair(&bundle).1;
    let altered = StateMetadata {
        cycle: Cycle::new(6),
        ..state_metadata()
    };
    let nonce = nonce_that_commits(&bundle, altered, None);
    let view = PublicMaturityStateView::new(statements(&bundle, altered, nonce, program.clone()))
        .expect("the seven statements name distinct entries");
    assert_eq!(view.current_cycle(), Cycle::new(6));
    assert_eq!(
        view.validate(&super::reviewed_target(), &FixtureStateCurve)
            .unwrap_err(),
        TransactionRefusal::MaturityViewProgramNotReconstructed {
            supplied: program.len(),
            recomputed: program.len(),
        }
    );
}
