//! The shared STATE fixtures: one real linked maturity bundle, and the
//! validated view over it.
//!
//! # The bundle is a real one, and there is one of it
//!
//! Everything here reaches the linked bundle the way an external consumer
//! would: derive the realization, bind the compiler input, plan the
//! announcement operation, compose the announcement record through the
//! backend's public builders, bind the deployment, and link. A
//! hand-assembled bundle would let a check agree with a policy and a
//! static subtree no link produced, which is the one thing the checks
//! built on it are for.
//!
//! They live in their own module because two test files now need the same
//! bundle. Two copies of this construction would be two deployments
//! answering to one name, and a test reading the second while its
//! subject read the first would pass or fail for a reason nobody could
//! see.
//!
//! # Why the fixture curve is a function of the root
//!
//! Curve arithmetic is a behaviour the caller supplies, and the stand-in
//! below computes no point sum. What it must do is answer differently for
//! different merkle roots: the view's check recomputes a program from a
//! caller's metadata and nonce and compares it with the supplied one, so a
//! stub answering with one key for every root would make that comparison
//! succeed at every nonce and each negative built on it would pass while
//! checking nothing.
//!
//! The material is public and meaningless — it proves nothing about any
//! curve and holds no scalar.
//!
//! Two traits ask that question now, and one formula answers both. The
//! operator freeze recomputes an output key from a control block's
//! internal key and folded root through the *live* capability and
//! compares the program it builds with the spent output's, which the
//! *STATE* capability produced. A second formula would turn that
//! comparison into a test of whether this file agrees with itself, and
//! it would fail for a reason that says nothing about the code under
//! test.

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
    StateCurveCapability, StateInternalKeyPolicy, StateLeafRole, StateNonceBudget,
    StateOperatorBindings, StateOperatorSymbol, StatePatternBindings, StatePatternSymbol,
    StateStaticLeaf, StateStaticNode, StateStaticSubtree, StateTweakOutcome,
    build_state_announcement_program, build_state_operator_pattern, operator_key_encoding_closure,
    production_static_subtree, selected_operator_profile, state_announcement_patterns,
    state_announcement_program, state_metadata_leaf_program, state_operator_fragment,
    state_structural_patterns,
};
use target_elements::{EncodingClass, LeafVersion};

use super::{outpoint, reviewed_target};
use crate::bytes::{AssetField, AssetId, ValueField};
use crate::live_taproot::{LiveCurveCapability, TweakedOutputKey};
use crate::operator_right::BranchContext;
use crate::operator_signing::{ScriptPathSignatureVerifier, ScriptPathVerifierRejection};
use crate::state_view::{
    MaturityViewStatement, PublicMaturityStateView, ValidatedMaturityStateView,
};
use crate::taproot::{Digest32, OutputKeyParity};

/// The one tweak both curve vocabularies answer with.
///
/// The digest over the root is what makes a wrong nonce produce a
/// different program rather than the same one; the parity is the last
/// byte's, so a control block that stated the other one is caught.
fn fixture_tweak(key: &[u8], root: &[u8; 32]) -> ([u8; 32], bool) {
    let mut hash = Sha256::new();
    hash.update(b"fixture-state-curve");
    hash.update(key);
    hash.update(root);
    let derived: [u8; 32] = hash.finalize().into();
    (derived, (derived[31] & 1) == 1)
}

/// A deterministic stand-in for public curve arithmetic.
///
/// The assertion keeps it from answering for a key it was not asked
/// about.
pub(super) struct FixtureStateCurve;

impl StateCurveCapability for FixtureStateCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        assert_eq!(key, &STATE_NUMS_KEY);
        true
    }

    fn output_key(&self, key: &[u8; 32], root: &[u8; 32]) -> StateTweakOutcome {
        assert_eq!(key, &STATE_NUMS_KEY);
        let (key, parity) = fixture_tweak(key, root);
        StateTweakOutcome::OutputKey { key, parity }
    }
}

/// The same stand-in, answering the live curve vocabulary.
///
/// A separate type rather than a second trait on [`FixtureStateCurve`]:
/// both traits spell the method `output_key` with different signatures,
/// and one type carrying both would force every call site to say which
/// it meant for nothing gained. What matters is that the two answer
/// from one formula, and they do.
///
/// The membership question is answered by width alone. It stands for
/// public point arithmetic this crate deliberately does not have, and a
/// fixture that pretended to decide it would be asserting the very
/// thing the capability exists to leave outside.
pub(super) struct FixtureLiveCurve;

impl LiveCurveCapability for FixtureLiveCurve {
    fn owner_key_is_a_curve_point(&self, owner: &[u8]) -> bool {
        owner.len() == STATE_NUMS_KEY.len()
    }

    fn output_key(&self, internal_key: &[u8], merkle_root: &Digest32) -> Option<TweakedOutputKey> {
        assert_eq!(internal_key, STATE_NUMS_KEY.as_slice());
        let (key, parity) = fixture_tweak(internal_key, merkle_root);
        let parity = if parity {
            OutputKeyParity::Odd
        } else {
            OutputKeyParity::Even
        };
        Some(TweakedOutputKey::new(key, parity))
    }
}

/// The live stand-in, refusing to call the committed operator key a
/// point.
///
/// The tweak is unchanged, so the only thing this fixture moves is the
/// membership verdict — which is what lets a test reach the freeze's
/// first curve refusal without disturbing any tree.
pub(super) struct FixtureLiveCurveRefusingTheOperatorKey;

impl LiveCurveCapability for FixtureLiveCurveRefusingTheOperatorKey {
    fn owner_key_is_a_curve_point(&self, _owner: &[u8]) -> bool {
        false
    }

    fn output_key(&self, internal_key: &[u8], merkle_root: &Digest32) -> Option<TweakedOutputKey> {
        FixtureLiveCurve.output_key(internal_key, merkle_root)
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
pub(super) fn record() -> StateAnnouncementProgram {
    static RECORD: LazyLock<StateAnnouncementProgram> = LazyLock::new(|| {
        let target = reviewed_target();
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

        let raw = state_announcement_program(
            &target,
            &structural,
            &semantic,
            &operator,
            tapscript::StateWitnessSchedule::WholeMetadata,
        )
        .expect("the composed program assembles");
        build_state_announcement_program(
            &target,
            &structural,
            &semantic,
            &operator,
            tapscript::StateWitnessSchedule::WholeMetadata,
            raw,
        )
        .expect("the composed record is admitted")
    });
    RECORD.clone()
}

/// The semantic metadata the demonstration link is run over.
///
/// The four quantities are pairwise distinct, so a copy-through that
/// swapped two of them would fail rather than pass by coincidence.
pub(super) fn state_metadata() -> StateMetadata {
    StateMetadata {
        omega: ProtocolAmount::new(1).expect("the fixture quantities are in domain"),
        y_l: ProtocolAmount::new(2).expect("the fixture quantities are in domain"),
        y_t: ProtocolAmount::new(3).expect("the fixture quantities are in domain"),
        q: ProtocolAmount::new(4).expect("the fixture quantities are in domain"),
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    }
}

/// The demonstration deployment's identity.
///
/// Its genesis is byte-uniform, so it is its own reversal: a check that
/// converts between the printed and internal byte orders passes over it
/// whichever direction it converts in. That is why
/// [`asymmetric_genesis_identity`] exists.
pub(super) fn demonstration_identity() -> CandidateDeploymentIdentity {
    CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32])
        .expect("fixture identifiers are nonzero")
}

/// A second identity whose genesis is not its own reversal.
///
/// One byte moved, and moved at an end: the reversal of this genesis
/// differs from it in its first and last bytes, so a conversion run in
/// the wrong direction — or not run at all — produces a value the
/// binding does not commit to and is caught by name. Everything else
/// about the deployment is the demonstration one's, so a bundle linked
/// over it differs in exactly the value under test.
pub(super) fn asymmetric_genesis_identity() -> CandidateDeploymentIdentity {
    let mut genesis = [0x22; 32];
    genesis[0] = 0xa1;
    CandidateDeploymentIdentity::new([0x11; 32], genesis).expect("fixture identifiers are nonzero")
}

/// The bound deployment sources of one link, over one identity.
fn bridge(identity: CandidateDeploymentIdentity) -> StateLinkDeploymentParameters {
    let target = reviewed_target();
    let bounds = AnnouncementLeadBounds::new(Cycle::new(2), Cycle::new(4))
        .expect("the fixture window is nonzero and ordered");
    let closure = operator_key_encoding_closure(target.definition().authorization());
    let key = OperatorKey::new(&closure, closure.approved(), vec![0x33; 32])
        .expect("fixture public bytes have the approved shape");
    let internal_key = StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0xb6; 32])
        .expect("fixture internal key has the reviewed width");
    let profile = EstablishedOperatorProfile::establish(selected_operator_profile(), &target)
        .expect("the reviewed target establishes the source selection");
    let binding =
        OperatorDeploymentBinding::bind(&target, key, profile, identity.clone(), &internal_key)
            .expect("the candidate deployment binds");

    StateLinkDeploymentParameters::bind(
        &target,
        plan(),
        StateLeadBounds::new(bounds, StateLeadBoundOrigin::Fixture),
        identity,
        binding,
        NonZeroU32::new(8).expect("eight is nonzero"),
        &record(),
    )
    .expect("the demonstration sources bind")
}

/// The candidate linked maturity bundle over one budget and one
/// predecessor metadata.
///
/// The general form, because the nonce budget a bundle carries is the
/// budget its own constructor declared: a link over a one-attempt
/// constructor is the only way to reach an exhausted successor search
/// without a scan that runs for as long as the reviewed budget admits.
/// Everything else is the demonstration deployment's, so a bundle built
/// here differs from [`linked_bundle`] in exactly the two values named.
pub(super) fn linked_bundle_with(
    budget: StateNonceBudget,
    metadata: StateMetadata,
) -> CandidateLinkedMaturityBundle {
    linked_bundle_over(demonstration_identity(), budget, metadata)
}

/// The same bundle over one named deployment identity.
///
/// The identity reaches both the operator binding and the link's own
/// deployment parameters, because those two are the pair a genesis
/// check compares: a fixture that varied one of them would be testing
/// that they disagree rather than that the conversion between their
/// byte orders is performed.
pub(super) fn linked_bundle_over(
    identity: CandidateDeploymentIdentity,
    budget: StateNonceBudget,
    metadata: StateMetadata,
) -> CandidateLinkedMaturityBundle {
    let target = reviewed_target();
    let subtree = production_static_subtree(&target, &record())
        .expect("the composed record yields the production subtree");
    let constructor = CandidateStateConstructor::derive(
        &target,
        &metadata,
        &subtree,
        StateInternalKeyPolicy::new(STATE_NUMS_KEY, &FixtureStateCurve)
            .expect("the reviewed internal key is the derived one"),
        budget,
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
            &bridge(identity),
            &constructor,
            &StateSingletonAsset::new([0x11; 32]),
            &StateSingletonDeclaration::from_architecture_asset(spec)
                .expect("the declaration is a singleton"),
            &metadata,
            &FixtureStateCurve,
        ),
    )
    .expect("the demonstration sources link")
}

/// The candidate linked maturity bundle, linked once and cloned.
pub(super) fn linked_bundle() -> CandidateLinkedMaturityBundle {
    static BUNDLE: LazyLock<CandidateLinkedMaturityBundle> =
        LazyLock::new(|| linked_bundle_with(StateNonceBudget::default(), state_metadata()));
    BUNDLE.clone()
}

/// A static subtree differing from the production one in its root
/// alone.
///
/// The production subtree is the announcement leaf by itself; this one
/// branches that leaf with the canonical metadata leaf in a support
/// role, so the two commit different roots while the leaf version and
/// the internal key stay the production ones. That is the only one of
/// the three parameters the continuity equality compares that a fixture
/// can vary here: the leaf version is the reviewed target's, and the
/// stand-in curve answers for one internal key and asserts on any
/// other.
pub(super) fn second_static_subtree() -> StateStaticSubtree {
    let target = reviewed_target();
    let metadata_leaf = state_metadata_leaf_program(
        &target,
        &EncodedStateMetadata {
            semantic: state_metadata(),
            representation: StateRepresentationNonce::new(0),
        },
    )
    .expect("the canonical metadata encodes as a leaf program");

    StateStaticSubtree::new(
        &target,
        Some(StateStaticNode::Branch(
            Box::new(StateStaticNode::Leaf {
                identity: 0,
                leaf: StateStaticLeaf {
                    role: StateLeafRole::Announcement,
                    version: LeafVersion::TAPSCRIPT.get(),
                    program: record().program().clone(),
                },
            }),
            Box::new(StateStaticNode::Leaf {
                identity: 1,
                leaf: StateStaticLeaf {
                    role: StateLeafRole::Support(0),
                    version: LeafVersion::TAPSCRIPT.get(),
                    program: metadata_leaf,
                },
            }),
        )),
    )
    .expect("the two-leaf tree is complete and its roles are distinct")
}

/// The program the link's own application committed, at its own nonce.
pub(super) fn linked_pair(
    bundle: &CandidateLinkedMaturityBundle,
) -> (StateRepresentationNonce, Vec<u8>) {
    let constructor = bundle.instances()[0].constructor();
    (constructor.nonce(), constructor.output_program())
}

/// The seven statements of a view over the demonstration bundle.
pub(super) fn statements(
    bundle: &CandidateLinkedMaturityBundle,
    metadata: StateMetadata,
    nonce: StateRepresentationNonce,
    program: Vec<u8>,
) -> Vec<MaturityViewStatement> {
    vec![
        MaturityViewStatement::CurrentStateOutpoint(outpoint(0x71, 0)),
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
pub(super) fn demonstration_view() -> PublicMaturityStateView {
    let bundle = linked_bundle();
    let (nonce, program) = linked_pair(&bundle);
    PublicMaturityStateView::new(statements(&bundle, state_metadata(), nonce, program))
        .expect("the seven statements name distinct entries")
}

/// That view, with its one checkable relation computed.
pub(super) fn validated_view() -> ValidatedMaturityStateView {
    demonstration_view()
        .validate(&reviewed_target(), &FixtureStateCurve)
        .expect("the stated pair reproduces the stated program")
}

/// What the fixture verifier calls itself.
///
/// Named once so that a test can read the standing back and compare it
/// with the implementation that produced it, rather than restating the
/// string on both sides.
pub(super) const FIXTURE_VERIFIER: &str =
    "deterministic public-data test double; not Schnorr and not native evidence";

/// A deterministic test function over public material.
///
/// Not Schnorr, not cryptography and not signature evidence. Two digests
/// over the same two values in both orders, which gives the sixty-four
/// bytes the operator profile selects and which answers differently for
/// a different key or a different message — the second property being
/// the one a negative needs, since a function answering alike for every
/// message would let a signature over another candidate verify.
///
/// The material is public and meaningless. No scalar exists here, and
/// nothing this produces says anything about any curve.
pub(super) fn fixture_sign(key: &[u8], message: &Digest32) -> [u8; 64] {
    let first: [u8; 32] = Sha256::new()
        .chain_update(key)
        .chain_update(message)
        .finalize()
        .into();
    let second: [u8; 32] = Sha256::new()
        .chain_update(message)
        .chain_update(key)
        .finalize()
        .into();
    let mut signature = [0_u8; 64];
    signature[..32].copy_from_slice(&first);
    signature[32..].copy_from_slice(&second);
    signature
}

/// A test-only double accepting exactly [`fixture_sign`]'s output.
///
/// In-process evidence and explicitly not native evidence: it decides
/// nothing about any curve, and it stands in the place production fills
/// with an independently implemented verifier. It is deliberately the
/// only thing it accepts, so a signature this file did not produce over
/// this message fails here rather than being waved through.
pub(super) struct FixtureScriptPathVerifier;

impl ScriptPathSignatureVerifier for FixtureScriptPathVerifier {
    fn verify(
        &self,
        key: &[u8],
        message: &Digest32,
        signature: &[u8],
    ) -> Result<(), ScriptPathVerifierRejection> {
        if signature == fixture_sign(key, message).as_slice() {
            Ok(())
        } else {
            Err(ScriptPathVerifierRejection::new(
                "the fixture function does not produce these bytes".to_owned(),
            ))
        }
    }

    fn description(&self) -> &str {
        FIXTURE_VERIFIER
    }
}
