//! The maturity link closed over its linked bytes (Guide-14 §11.5, §11.6).
//!
//! # The bytes are the fact, and the structures are the claim
//!
//! Everything here reads the DECODED BYTES of the linked announcement
//! leaf, together with the validated plan's and the composed record's own
//! statements. No check takes a linker field as the fact under test. The
//! reason is the ordinary one for independent evidence: a test that reads
//! the structure it is checking passes when the structure is wrong, so it
//! establishes that the structure is self-consistent and nothing about
//! the program a spend actually runs.
//!
//! The discipline is decode-then-forget. The bundle's typed program is
//! read exactly once, to take its encoding; from that point the encoding
//! is parsed back through the target's own decoder and every subsequent
//! question is asked of the parsed instructions. A linked program and its
//! bytes are two views of one artifact only if the encoder and the
//! decoder agree, and requiring the decode to re-encode to the same bytes
//! is what makes that an established fact here rather than an assumption.
//!
//! # What a predecessor-program literal would actually be
//!
//! The reduction removed the structural fragment's literal for the
//! consumed program, and a test for its absence has to know what such a
//! literal would look like. Four byte strings are compared against every
//! push, and each is a shape the removed literal could really have taken:
//! the constructor's witness-version-one output program, which is the
//! program an input of this family is spent from; the x-only output key
//! inside it, which is that program with its two framing bytes removed;
//! the encoded metadata commitment program, which is the sibling leaf's
//! script; and the canonical metadata bytes it carries. The first two are
//! the consumed program itself, stated at the two widths a program is
//! handled at; the second two are what commits to it. None occurs.
//!
//! # Where the site indices come from, and where the values come from
//!
//! Locating a discharge means re-emitting a component through the public
//! builders and finding its instructions in the decoded program. That
//! needs the linked values, and they are read from the decoded pushes.
//! The INDICES of those pushes are the composed record's own consumer
//! census, which is a claim side rather than the structure under test;
//! the linker's relocation census is compared against them as an
//! agreement and is never the source. The residual is exact and is stated
//! rather than absorbed: the values are the bytes' and the indices are
//! the record's, and nothing here takes an index from the link.
//!
//! # What the node-free verdict does not decide
//!
//! An adoption vector's verdict is decided by the equalities the leaf's
//! own bytes carry: the self-position pin, the asset and explicit-amount
//! equalities on both sides, and the script-version equalities. Output
//! zero's PROGRAM is authenticated against the witness-supplied metadata
//! by a curve relation rather than by a byte equality, and the operator's
//! signature is a witness fact. Neither is decided here. Both belong to a
//! run against a target node, which is why every vector retains its exact
//! submission subject.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU32, NonZeroU64};
use std::ops::Range;
use std::sync::LazyLock;

use architecture::{ARCHITECTURE, AssetId, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::maturity_announcement_plan::{
    ValidatedMaturityAnnouncementOperationPlan, plan_maturity_announcement_target_operation,
};
use compiler::operation_plan::PlacementSearchLimits;
use linker::{
    CandidateDeploymentIdentity, CandidateLinkedMaturityBundle, LinkRefusal,
    OperatorDeploymentBinding, StateCarrierClosure, StateCarrierRow, StateDischargeClass,
    StateLeadBoundOrigin, StateLeadBounds, StateLinkDeploymentParameters, StateLinkSources,
    StateLinkedCarrier, StateSingletonAsset, link_state_candidate,
};
use realization::{
    Cycle, ExternalEvidenceRequirement, Maturity, ProtocolAmount, RealizationScope, RelationId,
    StateMetadata, derive,
};
use tapscript::upstream::{AnnouncementLeadBounds, StateSingletonDeclaration};
use tapscript::{
    CandidateStateConstructor, EstablishedOperatorProfile, MaturityCarrier, OperatorKey,
    STATE_NUMS_KEY, StackItem, StateAnnouncementBindings, StateAnnouncementProgram,
    StateAnnouncementSymbol, StateCurveCapability, StateInternalKeyPolicy, StateLeafRole,
    StateNonceBudget, StateOperatorBindings, StateOperatorSymbol, StatePatternBindings,
    StatePatternSymbol, StateProgramComponent, StateProgramSymbol, StateTweakOutcome,
    TapscriptInstruction, TapscriptProgram, build_state_announcement_program,
    build_state_operator_pattern, operator_key_encoding_closure, production_static_subtree,
    selected_operator_profile, state_announcement_patterns, state_announcement_program,
    state_operator_fragment, state_structural_patterns,
};
use target_elements::{
    EncodingClass, OpcodeId, ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript,
};
use target_elements_conformance::constructor::curve::lift_x;
use target_elements_conformance::constructor::tree::{
    TweakDefect, branch_hash, leaf_hash, tweak, tweaked_key,
};
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::{OperationSubject, TargetSubmissionSubject};
use target_elements_conformance::test_material::PublicTestSignerHandle;
use transaction::TransactionRefusal;
use transaction::bytes::{
    AssetField, AssetId as TargetAssetId, InputWitness, NonceField, Outpoint, TargetInput,
    TargetOutput, TargetTransaction, Txid, ValueField,
};
use transaction::script_path_signing::SpentOutputCensusEntry;
use transaction::taproot::witness_program_script;

// --- Refusals -----------------------------------------------------------

/// Why a closure check over linked bytes refused.
///
/// The module's own closed vocabulary rather than a variant of the
/// crate-wide error, following the convention the crate already holds:
/// a module owns the names for its own failures, and the crate-wide
/// error wraps one only where a shared caller needs a single type.
/// Nothing outside this module calls these functions, so nothing needs
/// that wrapping yet.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MaturityClosureRefusal {
    /// The reviewed target contract did not validate.
    ReviewedTargetInvalid,
    /// The realization did not derive, the compiler did not bind its
    /// input, or the announcement operation did not plan.
    PlanUnavailable,
    /// The composed announcement record did not assemble or was not
    /// admitted.
    RecordUnavailable,
    /// A source of the link refused: the constructor derivation, the
    /// operator binding, the lead window, or the bound bridge.
    SourcesUnavailable,
    /// The link itself refused.
    LinkRefused(LinkRefusal),
    /// The bundle carries no announcement program to read bytes from.
    AnnouncementProgramAbsent,
    /// The linked bytes do not parse as a program of the reviewed
    /// subset.
    LeafBytesDoNotDecode,
    /// The linked bytes parse, and the parse does not re-encode to them.
    ///
    /// Two views of one artifact only if the encoder and the decoder
    /// agree; this is that agreement, put rather than assumed.
    LeafBytesDoNotRoundTrip,
    /// A primitive the reduction removed occurs in the decoded program.
    RemovedPrimitivePresent {
        /// The primitive that came back.
        opcode: OpcodeId,
        /// Where it occurs.
        instruction: usize,
    },
    /// An introspection is not preceded by a literal zero.
    ///
    /// The whole of what makes every other position free: an
    /// introspection whose position operand is anything else reads a
    /// position this operation does not claim.
    IntrospectionWithoutLiteralZero {
        /// The introspecting instruction.
        instruction: usize,
    },
    /// A push carries one of the program literals the reduction removed.
    ProgramLiteralPresent {
        /// Where the literal occurs.
        instruction: usize,
        /// How wide it is, which names which of the four shapes it is.
        width: usize,
    },
    /// A check the reduction kept is absent from the decoded program.
    KeptCheckAbsent {
        /// The check that did not occur.
        check: KeptCheck,
    },
    /// The record names a consumer site that the decoded program does
    /// not push at.
    ConsumerSiteIsNotAPush {
        /// The site the record names.
        instruction: usize,
    },
    /// Two sites of one consumer carry different bytes, so the symbol
    /// has no single linked value.
    ConsumerSitesDisagree {
        /// The consumer whose sites disagree.
        symbol: StateProgramSymbol,
    },
    /// The values recovered from the decoded pushes do not rebuild the
    /// component recipes through the public builders.
    ReEmissionRefused,
    /// A component the closure claims is not at the range it claims.
    ComponentNotLocated {
        /// The component that was re-emitted.
        component: StateProgramComponent,
        /// The range the closure claimed for it.
        claimed: Range<usize>,
    },
    /// A model-scope row claims bytes, which is the one thing its class
    /// means it does not.
    ModelScopeRowLocatesBytes {
        /// The relation whose row claimed them.
        relation: RelationId,
    },
    /// The census counted by locating disagrees with the census the
    /// closure publishes.
    DischargeCensusDisagrees {
        /// The class that disagreed.
        class: StateDischargeClass,
        /// What the closure publishes.
        read: usize,
        /// What locating counted.
        located: usize,
    },
    /// A recomputed golden figure is not the one the bundle carries.
    GoldenDisagrees {
        /// Which figure disagreed.
        figure: GoldenFigure,
    },
    /// The internal key the leaf pushes is not one key.
    ///
    /// Four sites push it and they must agree, because a tweak taken
    /// over one of them would say nothing about a spend that ran
    /// another.
    InternalKeySitesDisagree,
    /// The curve determined no output key for this internal key and
    /// root.
    OutputKeyUndetermined(StateTweakOutcome),
    /// Two deployments' linked bytes differ outside the spans of the
    /// sites that moved.
    BytesDifferOutsideMovedSites {
        /// The first byte position outside a moved span.
        position: usize,
    },
    /// Two deployments' linked programs do not have the same shape, so
    /// no site-by-site comparison is possible.
    LinkedProgramsAreNotComparable,
    /// An adoption transaction could not be built from public entries.
    AdoptionTransactionUnbuildable(TransactionRefusal),
    /// An adoption transaction's bytes do not decode back to it.
    AdoptionBytesDoNotRoundTrip,
}

// --- The curve ----------------------------------------------------------

/// The curve capability, answered by the independent host oracle.
///
/// Public-point arithmetic over the target's own tagged tweak, taken
/// from the conformance package that owns it. That package exists so
/// that an expectation about a constructor's output is produced by
/// something other than the constructor, and the same reason applies
/// here: the golden this module recomputes is worth recomputing only if
/// the arithmetic behind it is not the arithmetic the bundle used.
///
/// Nothing here is scripted. Unlike a stub that answers for one key and
/// asserts it was asked about no other, this answers correctly for any
/// key, so it makes no claim about which key it was handed and needs
/// none.
///
/// # No secret material
///
/// Both methods take public inputs and return public values. There is no
/// scalar in either signature.
pub struct OracleStateCurve;

impl StateCurveCapability for OracleStateCurve {
    fn internal_key_is_a_point(&self, x_only: &[u8; 32]) -> bool {
        lift_x(x_only).is_ok()
    }

    fn output_key(&self, internal_key: &[u8; 32], merkle_root: &[u8; 32]) -> StateTweakOutcome {
        let digest = tweak(internal_key, merkle_root);
        match tweaked_key(internal_key, &digest) {
            Ok((key, parity)) => StateTweakOutcome::OutputKey {
                key,
                parity: parity == 1,
            },
            Err(TweakDefect::InternalKeyNotOnCurve(_)) => StateTweakOutcome::InternalKeyNotAPoint,
            Err(TweakDefect::TweakNotAScalar) => StateTweakOutcome::TweakAboveGroupOrder,
            Err(TweakDefect::TweakedKeyIsIdentity) => StateTweakOutcome::TweakedPointIsIdentity,
        }
    }
}

// --- The deployments and their values -----------------------------------

/// The three values one deployment fixes: its issued singleton, the
/// operator key it commits, and its lead window.
///
/// Supplied rather than written into the deployment census, because
/// these three are what a deployment is free in and a link over them is
/// the same link either way. Nothing here constrains them. An asset
/// identity is whatever issued it, and a node derives one from its own
/// issuing outpoint, so a check shaped to a fixture fill would refuse a
/// really issued asset for being real; a committed operator key is
/// public x-only material, and the encoding closure the link binds it
/// through is what decides whether a value is one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityDeploymentParameters {
    singleton: [u8; 32],
    operator_key: [u8; 32],
    lead: (u64, u64),
}

impl MaturityDeploymentParameters {
    /// Take one deployment's three values as they are.
    #[must_use]
    pub const fn new(singleton: [u8; 32], operator_key: [u8; 32], lead: (u64, u64)) -> Self {
        Self {
            singleton,
            operator_key,
            lead,
        }
    }

    /// The issued singleton's identifier.
    #[must_use]
    pub const fn singleton(&self) -> &[u8; 32] {
        &self.singleton
    }

    /// The committed operator key.
    #[must_use]
    pub const fn operator_key(&self) -> &[u8; 32] {
        &self.operator_key
    }

    /// The lead window, as its two magnitudes.
    #[must_use]
    pub const fn lead(&self) -> (u64, u64) {
        self.lead
    }
}

/// Which fixture deployment a link is taken over.
///
/// Three, and each carries a reason the others do not. A link whose
/// resolved values are the ones the record was composed against is the
/// identity on bytes and cannot tell a substitution from a copy, which
/// is what the second is for: it differs in the issued asset, the
/// operator key and both lead magnitudes, and agrees in the internal key
/// and the amount, which the key policy and the architecture's
/// declaration fix for every deployment. The third commits the x-only
/// key of a published test signer. A key of meaningless fill is one
/// nobody holds a scalar for, so the signature the leaf's first
/// instruction pair verifies cannot be produced at all, and a committed
/// key somebody can sign under is what makes that verification reachable
/// rather than refused by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaturityDeployment {
    /// The deployment the composed record was itself composed against.
    Demonstration,
    /// A second deployment, differing in every value a deployment fixes.
    Second,
    /// A third, whose committed operator key a published test signer
    /// holds.
    PublishedSignerHeld,
}

impl MaturityDeployment {
    /// Every deployment, in declaration order.
    pub const ALL: [Self; 3] = [Self::Demonstration, Self::Second, Self::PublishedSignerHeld];

    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Demonstration => "demonstration",
            Self::Second => "second",
            Self::PublishedSignerHeld => "published-signer-held",
        }
    }

    /// The values this deployment fixes.
    ///
    /// The first two are fixture material throughout: public,
    /// meaningless fills beside lead windows standing for no calibrated
    /// deployment. The third's key is the published signer's own, read
    /// from the layer that owns that material rather than restated here,
    /// while its singleton and its window remain fixture values like the
    /// others.
    ///
    /// # Errors
    ///
    /// [`MaturityClosureRefusal::SourcesUnavailable`] when the published
    /// signer's committed material produces no key.
    pub fn parameters(self) -> Result<MaturityDeploymentParameters, MaturityClosureRefusal> {
        Ok(match self {
            Self::Demonstration => {
                MaturityDeploymentParameters::new([0x11; 32], [0x33; 32], (2, 4))
            }
            Self::Second => MaturityDeploymentParameters::new([0xa1; 32], [0xa3; 32], (3, 5)),
            Self::PublishedSignerHeld => MaturityDeploymentParameters::new(
                [0xd1; 32],
                published_signer_key().ok_or(MaturityClosureRefusal::SourcesUnavailable)?,
                (4, 6),
            ),
        })
    }
}

/// The published signer's x-only key, resolved once.
///
/// Read through the handle census that owns the disposable material,
/// which answers with a public key and never with the scalar behind it.
fn published_signer_key() -> Option<[u8; 32]> {
    static KEY: LazyLock<Option<[u8; 32]>> =
        LazyLock::new(|| PublicTestSignerHandle::Third.x_only_public_key().ok());
    *KEY
}

// --- The sources --------------------------------------------------------

/// One deployment's bound link sources, each as the type that validated
/// it.
///
/// Owned rather than borrowed because the link's own source type borrows
/// all seven of them, so a function handing back that type would be
/// handing back a borrow of its own temporaries.
#[derive(Clone, Debug)]
pub struct MaturitySources {
    target: ReviewedElementsTapscriptDefinition,
    record: StateAnnouncementProgram,
    bridge: StateLinkDeploymentParameters,
    constructor: CandidateStateConstructor,
    singleton: StateSingletonAsset,
    declaration: StateSingletonDeclaration,
    metadata: StateMetadata,
}

impl MaturitySources {
    /// The reviewed contract these sources are bound to.
    #[must_use]
    pub const fn target(&self) -> &ReviewedElementsTapscriptDefinition {
        &self.target
    }

    /// The composed announcement record, before substitution.
    #[must_use]
    pub const fn record(&self) -> &StateAnnouncementProgram {
        &self.record
    }

    /// The candidate constructor over the record's own static subtree.
    #[must_use]
    pub const fn constructor(&self) -> &CandidateStateConstructor {
        &self.constructor
    }

    /// Link the candidate bundle over these sources.
    ///
    /// # Errors
    ///
    /// [`MaturityClosureRefusal::LinkRefused`] carrying the linker's own
    /// refusal.
    pub fn link(
        &self,
        curve: &impl StateCurveCapability,
    ) -> Result<CandidateLinkedMaturityBundle, MaturityClosureRefusal> {
        link_state_candidate(
            &self.target,
            &StateLinkSources::new(
                &self.record,
                &self.bridge,
                &self.constructor,
                &self.singleton,
                &self.declaration,
                &self.metadata,
                curve,
            ),
        )
        .map_err(MaturityClosureRefusal::LinkRefused)
    }
}

/// The reviewed contract, validated once and handed out by clone.
///
/// # Errors
///
/// [`MaturityClosureRefusal::ReviewedTargetInvalid`].
pub fn closure_target() -> Result<ReviewedElementsTapscriptDefinition, MaturityClosureRefusal> {
    static TARGET: LazyLock<Result<ReviewedElementsTapscriptDefinition, MaturityClosureRefusal>> =
        LazyLock::new(|| {
            reviewed_elements_tapscript().map_err(|_| MaturityClosureRefusal::ReviewedTargetInvalid)
        });
    TARGET.clone()
}

/// The validated announcement plan, derived once and handed out by
/// clone.
fn announcement_plan() -> Result<ValidatedMaturityAnnouncementOperationPlan, MaturityClosureRefusal>
{
    static PLAN: LazyLock<
        Result<ValidatedMaturityAnnouncementOperationPlan, MaturityClosureRefusal>,
    > = LazyLock::new(build_announcement_plan);
    PLAN.clone()
}

/// The plan, through the compiler's own public planning entry.
fn build_announcement_plan()
-> Result<ValidatedMaturityAnnouncementOperationPlan, MaturityClosureRefusal> {
    let refused = || MaturityClosureRefusal::PlanUnavailable;
    let limit = |value: u64| NonZeroU64::new(value).ok_or_else(refused);
    let operations = [
        OperationId::AnnounceMaturity,
        OperationId::CompactAsh,
        OperationId::TransferLive,
    ];
    let scope = RealizationScope::from_operations(operations).map_err(|_| refused())?;
    let realization = derive(&ARCHITECTURE, scope).map_err(|_| refused())?;
    let scope = CompilationScope::from_operations([OperationId::AnnounceMaturity])
        .map_err(|_| refused())?;
    let policy = AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000)?, limit(10_000)?));
    let input = bind_input(&ARCHITECTURE, realization, scope, policy).map_err(|_| refused())?;

    plan_maturity_announcement_target_operation(
        &input,
        PlacementSearchLimits::new(limit(10_000_000)?, limit(1_000_000)?),
    )
    .map_err(|_| refused())
}

/// The composed announcement record, built once and handed out by clone.
fn composed_record() -> Result<StateAnnouncementProgram, MaturityClosureRefusal> {
    static RECORD: LazyLock<Result<StateAnnouncementProgram, MaturityClosureRefusal>> =
        LazyLock::new(build_composed_record);
    RECORD.clone()
}

/// The record, through the three public component entries and the
/// composition entry that admits them.
fn build_composed_record() -> Result<StateAnnouncementProgram, MaturityClosureRefusal> {
    let target = closure_target()?;
    let refused = || MaturityClosureRefusal::RecordUnavailable;
    let item = |bytes: Vec<u8>| StackItem::new(&target, bytes).map_err(|_| refused());

    let structural = StatePatternBindings::new(
        &target,
        BTreeMap::from([
            (StatePatternSymbol::StateAsset, item(vec![0x11; 32])?),
            (
                StatePatternSymbol::StateAmount,
                StackItem::signed_le64(&target, 1),
            ),
        ]),
    )
    .map_err(|_| refused())?;
    let structural = state_structural_patterns(&target, &structural).map_err(|_| refused())?;

    let semantic = StateAnnouncementBindings::new(
        &target,
        BTreeMap::from([
            (
                StateAnnouncementSymbol::InternalKey,
                item(STATE_NUMS_KEY.to_vec())?,
            ),
            (
                StateAnnouncementSymbol::MaturityLeadMin,
                StackItem::unsigned_le64(&target, 2),
            ),
            (
                StateAnnouncementSymbol::MaturityLeadMax,
                StackItem::unsigned_le64(&target, 4),
            ),
            (StateAnnouncementSymbol::StateAsset, item(vec![0x11; 32])?),
            (
                StateAnnouncementSymbol::StateAmount,
                StackItem::signed_le64(&target, 1),
            ),
        ]),
    )
    .map_err(|_| refused())?;
    let semantic = state_announcement_patterns(&target, &semantic).map_err(|_| refused())?;

    let operator = operator_bindings(&target, 0x33)?;
    let fragment = state_operator_fragment(&operator).map_err(|_| refused())?;
    let operator =
        build_state_operator_pattern(&target, &operator, fragment).map_err(|_| refused())?;

    let raw = state_announcement_program(&target, &structural, &semantic, &operator)
        .map_err(|_| refused())?;
    build_state_announcement_program(&target, &structural, &semantic, &operator, raw)
        .map_err(|_| refused())
}

/// One deployment's committed operator key, as bindings.
fn operator_bindings(
    target: &ReviewedElementsTapscriptDefinition,
    byte: u8,
) -> Result<StateOperatorBindings, MaturityClosureRefusal> {
    let refused = || MaturityClosureRefusal::RecordUnavailable;
    let key = StackItem::encoded(target, EncodingClass::XOnlyPublicKey, vec![byte; 32])
        .map_err(|_| refused())?;
    StateOperatorBindings::new(
        target,
        &BTreeMap::from([(StateOperatorSymbol::CommittedOperatorKey, key)]),
    )
    .map_err(|_| refused())
}

/// The candidate constructor over the record's own production subtree,
/// derived once with real arithmetic and handed out by clone.
fn fixture_constructor() -> Result<CandidateStateConstructor, MaturityClosureRefusal> {
    static CONSTRUCTOR: LazyLock<Result<CandidateStateConstructor, MaturityClosureRefusal>> =
        LazyLock::new(|| {
            let target = closure_target()?;
            let record = composed_record()?;
            let refused = || MaturityClosureRefusal::SourcesUnavailable;
            let subtree = production_static_subtree(&target, &record).map_err(|_| refused())?;
            CandidateStateConstructor::derive(
                &target,
                &fixture_metadata()?,
                &subtree,
                StateInternalKeyPolicy::new(STATE_NUMS_KEY, &OracleStateCurve)
                    .map_err(|_| refused())?,
                StateNonceBudget::default(),
                &OracleStateCurve,
            )
            .map_err(|_| refused())
        });
    CONSTRUCTOR.clone()
}

/// The semantic metadata every constructor here is derived from.
///
/// The four quantities are pairwise distinct, so a copy-through that
/// swapped two of them would fail rather than pass by coincidence.
fn fixture_metadata() -> Result<StateMetadata, MaturityClosureRefusal> {
    let refused = || MaturityClosureRefusal::SourcesUnavailable;
    let amount = |value: u64| ProtocolAmount::new(value).map_err(|_| refused());
    Ok(StateMetadata {
        omega: amount(1)?,
        y_l: amount(2)?,
        y_t: amount(3)?,
        q: amount(4)?,
        cycle: Cycle::new(5),
        maturity: Maturity::Unannounced,
    })
}

/// The sources over supplied values, each reached through a public
/// entry.
///
/// The entry a caller with values of its own uses, a deployment of the
/// census being one such caller and not the only admissible one.
///
/// # Errors
///
/// [`MaturityClosureRefusal::ReviewedTargetInvalid`],
/// [`MaturityClosureRefusal::PlanUnavailable`],
/// [`MaturityClosureRefusal::RecordUnavailable`] or
/// [`MaturityClosureRefusal::SourcesUnavailable`], naming the layer that
/// refused.
pub fn maturity_sources_with(
    parameters: MaturityDeploymentParameters,
) -> Result<MaturitySources, MaturityClosureRefusal> {
    let target = closure_target()?;
    let record = composed_record()?;
    let constructor = fixture_constructor()?;
    let refused = || MaturityClosureRefusal::SourcesUnavailable;

    let (minimum, maximum) = parameters.lead();
    let window = AnnouncementLeadBounds::new(Cycle::new(minimum), Cycle::new(maximum))
        .map_err(|_| refused())?;
    let identity =
        CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).map_err(|_| refused())?;
    let internal = StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0xb6; 32])
        .map_err(|_| refused())?;
    let closure = operator_key_encoding_closure(target.definition().authorization());
    let key = OperatorKey::new(
        &closure,
        closure.approved(),
        parameters.operator_key().to_vec(),
    )
    .map_err(|_| refused())?;
    let profile = EstablishedOperatorProfile::establish(selected_operator_profile(), &target)
        .map_err(|_| refused())?;
    let binding =
        OperatorDeploymentBinding::bind(&target, key, profile, identity.clone(), &internal)
            .map_err(|_| refused())?;
    let depth = NonZeroU32::new(8).ok_or_else(refused)?;
    let bridge = StateLinkDeploymentParameters::bind(
        &target,
        announcement_plan()?,
        StateLeadBounds::new(window, StateLeadBoundOrigin::Fixture),
        identity,
        binding,
        depth,
        &record,
    )
    .map_err(|_| refused())?;
    let specification = ARCHITECTURE.asset(AssetId::Pid).ok_or_else(refused)?;

    Ok(MaturitySources {
        target,
        record,
        bridge,
        constructor,
        singleton: StateSingletonAsset::new(*parameters.singleton()),
        declaration: StateSingletonDeclaration::from_architecture_asset(specification)
            .map_err(|_| refused())?,
        metadata: fixture_metadata()?,
    })
}

/// One deployment's sources, over the values that deployment fixes.
///
/// # Errors
///
/// Everything [`maturity_sources_with`] refuses, plus
/// [`MaturityClosureRefusal::SourcesUnavailable`] where the deployment's
/// own values do not resolve.
pub fn maturity_sources(
    deployment: MaturityDeployment,
) -> Result<MaturitySources, MaturityClosureRefusal> {
    maturity_sources_with(deployment.parameters()?)
}

/// One deployment's linked candidate bundle, through the real curve.
///
/// # Errors
///
/// Everything [`maturity_sources`] refuses, plus
/// [`MaturityClosureRefusal::LinkRefused`].
pub fn linked_maturity_bundle(
    deployment: MaturityDeployment,
) -> Result<CandidateLinkedMaturityBundle, MaturityClosureRefusal> {
    maturity_sources(deployment)?.link(&OracleStateCurve)
}

// --- The decoded leaf ---------------------------------------------------

/// The linked announcement leaf, as bytes and as the parse of those
/// bytes.
///
/// Built only from bytes. Every check in this module takes this, and the
/// plan's and the record's own statements, as its subject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedAnnouncementLeaf {
    bytes: Vec<u8>,
    program: TapscriptProgram,
}

impl DecodedAnnouncementLeaf {
    /// The exact linked bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The parse of those bytes.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }

    /// The byte span one instruction of the parse occupies.
    ///
    /// Measured by encoding the prefixes either side of it, so the span
    /// is the encoding's own rather than a second account of what an
    /// instruction costs.
    #[must_use]
    pub fn instruction_span(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        index: usize,
    ) -> Option<Range<usize>> {
        let instructions = self.program.instructions();
        if index >= instructions.len() {
            return None;
        }
        let prefix = TapscriptProgram::new(instructions[..index].to_vec()).ok()?;
        let through = TapscriptProgram::new(instructions[..=index].to_vec()).ok()?;
        Some(prefix.encode(target).len()..through.encode(target).len())
    }
}

/// The announcement leaf's linked bytes, taken from the bundle once.
///
/// # Errors
///
/// [`MaturityClosureRefusal::AnnouncementProgramAbsent`] when the bundle
/// carries no announcement program.
pub fn linked_announcement_bytes(
    bundle: &CandidateLinkedMaturityBundle,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<u8>, MaturityClosureRefusal> {
    Ok(bundle
        .program(StateLeafRole::Announcement)
        .ok_or(MaturityClosureRefusal::AnnouncementProgramAbsent)?
        .program()
        .encode(target))
}

/// Parse linked bytes, and require the parse to re-encode to them.
///
/// # Errors
///
/// [`MaturityClosureRefusal::LeafBytesDoNotDecode`] or
/// [`MaturityClosureRefusal::LeafBytesDoNotRoundTrip`].
pub fn decode_announcement_leaf(
    target: &ReviewedElementsTapscriptDefinition,
    bytes: &[u8],
) -> Result<DecodedAnnouncementLeaf, MaturityClosureRefusal> {
    let program = TapscriptProgram::decode(target, bytes)
        .map_err(|_| MaturityClosureRefusal::LeafBytesDoNotDecode)?;
    if program.encode(target) != bytes {
        return Err(MaturityClosureRefusal::LeafBytesDoNotRoundTrip);
    }
    Ok(DecodedAnnouncementLeaf {
        bytes: bytes.to_vec(),
        program,
    })
}

/// One deployment's decoded linked leaf, with the bundle it came from.
///
/// # Errors
///
/// Everything [`linked_maturity_bundle`] and
/// [`decode_announcement_leaf`] refuse.
pub fn decoded_deployment(
    deployment: MaturityDeployment,
) -> Result<(CandidateLinkedMaturityBundle, DecodedAnnouncementLeaf), MaturityClosureRefusal> {
    let target = closure_target()?;
    let bundle = linked_maturity_bundle(deployment)?;
    let bytes = linked_announcement_bytes(&bundle, &target)?;
    let leaf = decode_announcement_leaf(&target, &bytes)?;
    Ok((bundle, leaf))
}

// --- Reading instructions -----------------------------------------------

/// One instruction of a parse, or nothing beyond its end.
fn at(instructions: &[TapscriptInstruction], index: usize) -> Option<&TapscriptInstruction> {
    instructions.get(index)
}

/// The bytes one instruction pushes, or nothing if it pushes none.
fn push_bytes(instruction: &TapscriptInstruction) -> Option<&[u8]> {
    match instruction {
        TapscriptInstruction::Push(item) => Some(item.bytes()),
        TapscriptInstruction::Opcode(_) => None,
    }
}

/// Whether one instruction is one primitive.
fn is_opcode(instruction: &TapscriptInstruction, opcode: OpcodeId) -> bool {
    matches!(instruction, TapscriptInstruction::Opcode(found) if *found == opcode)
}

/// Whether the instruction at an index is one primitive.
fn opcode_at(instructions: &[TapscriptInstruction], index: usize, opcode: OpcodeId) -> bool {
    at(instructions, index).is_some_and(|found| is_opcode(found, opcode))
}

/// The script-number value one instruction pushes, if it pushes one.
fn pushed_number(
    instruction: &TapscriptInstruction,
    target: &ReviewedElementsTapscriptDefinition,
) -> Option<i64> {
    match instruction {
        TapscriptInstruction::Push(item) => item.script_number_value(target),
        TapscriptInstruction::Opcode(_) => None,
    }
}

/// Whether the instruction at an index pushes one script number.
fn number_at(
    instructions: &[TapscriptInstruction],
    target: &ReviewedElementsTapscriptDefinition,
    index: usize,
    value: i64,
) -> bool {
    at(instructions, index).and_then(|found| pushed_number(found, target)) == Some(value)
}

/// Whether the instruction at an index pushes exactly this many bytes.
fn width_at(instructions: &[TapscriptInstruction], index: usize, width: usize) -> bool {
    at(instructions, index)
        .and_then(push_bytes)
        .map(<[u8]>::len)
        == Some(width)
}

/// The prefix byte the reviewed contract selects one explicit form with.
///
/// Read from the target's own encoding table rather than restated, so a
/// contract revision that moved a prefix moves this with it.
fn explicit_prefix(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> Option<u8> {
    target
        .definition()
        .encodings()
        .get(&class)?
        .prefixes()
        .iter()
        .next()
        .copied()
}

/// The six primitives that read a position on either side.
const POSITIONAL_INTROSPECTIONS: [OpcodeId; 6] = [
    OpcodeId::InspectInputAsset,
    OpcodeId::InspectInputValue,
    OpcodeId::InspectInputScriptPubKey,
    OpcodeId::InspectOutputAsset,
    OpcodeId::InspectOutputValue,
    OpcodeId::InspectOutputScriptPubKey,
];

/// The three primitives the reduction removed outright.
const REMOVED_PRIMITIVES: [OpcodeId; 3] = [
    OpcodeId::InspectNumInputs,
    OpcodeId::InspectNumOutputs,
    OpcodeId::InspectInputIssuance,
];

// --- The literals a removed predecessor program would have been ---------

/// The byte strings a predecessor-program literal could have been.
///
/// Derived from the bundle's own retained constructor, because what the
/// removed literal would have equalled is a fact about the object the
/// leaf consumes rather than a width written down here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForbiddenLiterals {
    shapes: Vec<Vec<u8>>,
}

impl ForbiddenLiterals {
    /// Every forbidden byte string.
    #[must_use]
    pub fn shapes(&self) -> &[Vec<u8>] {
        &self.shapes
    }

    /// Their widths, in the same order.
    #[must_use]
    pub fn widths(&self) -> Vec<usize> {
        self.shapes.iter().map(Vec::len).collect()
    }
}

/// The four shapes, from the bundle's retained constructor.
///
/// # Errors
///
/// [`MaturityClosureRefusal::AnnouncementProgramAbsent`] when the bundle
/// retains no constructor to read them from.
pub fn forbidden_program_literals(
    bundle: &CandidateLinkedMaturityBundle,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<ForbiddenLiterals, MaturityClosureRefusal> {
    let constructor = bundle
        .instances()
        .first()
        .ok_or(MaturityClosureRefusal::AnnouncementProgramAbsent)?
        .constructor();
    Ok(ForbiddenLiterals {
        shapes: vec![
            constructor.output_program(),
            constructor.output_key().to_vec(),
            constructor.leaf_program().encode(target),
            constructor.metadata_bytes().to_vec(),
        ],
    })
}

// --- The checks the reduction kept --------------------------------------

/// One check the reduced leaf still carries, as an instruction pattern.
///
/// Thirteen rather than a shorter list, because each sits at its own
/// site and folding two together would let one go missing while the
/// other answered for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeptCheck {
    /// The executing input index, verified equal to zero.
    SelfPositionPin,
    /// Input zero's explicit asset, verified equal to the linked asset.
    InputZeroAsset,
    /// Input zero's explicit amount, verified equal to the linked
    /// amount.
    InputZeroExplicitAmount,
    /// Input zero's script version, verified and its program discarded.
    InputZeroScriptVersion,
    /// The consumed program's script version, inside the authentication
    /// that then binds the program itself.
    PredecessorProgramVersion,
    /// The predecessor's tweak relation over the internal key.
    PredecessorTweakVerify,
    /// The lead window's lower comparison.
    LeadLowerBound,
    /// The lead window's upper comparison.
    LeadUpperBound,
    /// Output zero's explicit asset, verified equal to the linked asset.
    OutputZeroAsset,
    /// Output zero's explicit amount, verified equal to the linked
    /// amount.
    OutputZeroExplicitAmount,
    /// The successor program's script version, inside its
    /// authentication.
    OutputZeroScriptVersion,
    /// The successor's tweak relation over the internal key.
    SuccessorTweakVerify,
    /// The committed operator key and its signature verification.
    OperatorAuthorization,
}

impl KeptCheck {
    /// Every kept check, in the order the leaf reaches them.
    pub const ALL: [Self; 13] = [
        Self::OperatorAuthorization,
        Self::SelfPositionPin,
        Self::InputZeroAsset,
        Self::InputZeroExplicitAmount,
        Self::InputZeroScriptVersion,
        Self::PredecessorProgramVersion,
        Self::PredecessorTweakVerify,
        Self::LeadLowerBound,
        Self::LeadUpperBound,
        Self::OutputZeroAsset,
        Self::OutputZeroExplicitAmount,
        Self::OutputZeroScriptVersion,
        Self::SuccessorTweakVerify,
    ];

    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SelfPositionPin => "self-position-pin",
            Self::InputZeroAsset => "input-zero-asset",
            Self::InputZeroExplicitAmount => "input-zero-explicit-amount",
            Self::InputZeroScriptVersion => "input-zero-script-version",
            Self::PredecessorProgramVersion => "predecessor-program-version",
            Self::PredecessorTweakVerify => "predecessor-tweak-verify",
            Self::LeadLowerBound => "lead-lower-bound",
            Self::LeadUpperBound => "lead-upper-bound",
            Self::OutputZeroAsset => "output-zero-asset",
            Self::OutputZeroExplicitAmount => "output-zero-explicit-amount",
            Self::OutputZeroScriptVersion => "output-zero-script-version",
            Self::SuccessorTweakVerify => "successor-tweak-verify",
            Self::OperatorAuthorization => "operator-authorization",
        }
    }

    /// How many instructions the pattern spans.
    #[must_use]
    pub const fn window(self) -> usize {
        match self {
            Self::SelfPositionPin => 3,
            Self::InputZeroAsset
            | Self::InputZeroExplicitAmount
            | Self::OutputZeroAsset
            | Self::OutputZeroExplicitAmount => 6,
            Self::InputZeroScriptVersion
            | Self::PredecessorProgramVersion
            | Self::OutputZeroScriptVersion => 5,
            Self::PredecessorTweakVerify
            | Self::SuccessorTweakVerify
            | Self::OperatorAuthorization
            | Self::LeadLowerBound
            | Self::LeadUpperBound => 2,
        }
    }

    /// Which instruction of the window makes the check absent when it
    /// is removed.
    ///
    /// Chosen so that removing it breaks this pattern and no other: the
    /// position literal of an introspection is never the one removed,
    /// because removing that would make the leaf introspect an
    /// unnamed position and the census would refuse for that reason
    /// instead.
    #[must_use]
    pub const fn breaking_offset(self) -> usize {
        match self {
            Self::SelfPositionPin => 2,
            Self::InputZeroAsset
            | Self::InputZeroExplicitAmount
            | Self::OutputZeroAsset
            | Self::OutputZeroExplicitAmount => 5,
            Self::InputZeroScriptVersion
            | Self::PredecessorProgramVersion
            | Self::OutputZeroScriptVersion => 4,
            Self::PredecessorTweakVerify
            | Self::SuccessorTweakVerify
            | Self::OperatorAuthorization => 1,
            Self::LeadLowerBound | Self::LeadUpperBound => 0,
        }
    }
}

/// Whether one field recognition sits at an index.
fn field_check_at(
    instructions: &[TapscriptInstruction],
    target: &ReviewedElementsTapscriptDefinition,
    index: usize,
    inspect: OpcodeId,
    payload: (EncodingClass, usize),
) -> bool {
    let (class, width) = payload;
    let Some(prefix) = explicit_prefix(target, class) else {
        return false;
    };
    let wanted = [prefix];
    number_at(instructions, target, index, 0)
        && opcode_at(instructions, index + 1, inspect)
        && at(instructions, index + 2).and_then(push_bytes) == Some(&wanted[..])
        && opcode_at(instructions, index + 3, OpcodeId::EqualVerify)
        && width_at(instructions, index + 4, width)
        && opcode_at(instructions, index + 5, OpcodeId::EqualVerify)
}

/// Whether one script-version recognition sits at an index.
fn version_check_at(
    instructions: &[TapscriptInstruction],
    target: &ReviewedElementsTapscriptDefinition,
    index: usize,
    inspect: OpcodeId,
    tail: OpcodeId,
) -> bool {
    number_at(instructions, target, index, 0)
        && opcode_at(instructions, index + 1, inspect)
        && number_at(instructions, target, index + 2, 1)
        && opcode_at(instructions, index + 3, OpcodeId::EqualVerify)
        && opcode_at(instructions, index + 4, tail)
}

/// Whether one pushed-key primitive pair sits at an index.
fn keyed_check_at(instructions: &[TapscriptInstruction], index: usize, opcode: OpcodeId) -> bool {
    width_at(instructions, index, 32) && opcode_at(instructions, index + 1, opcode)
}

/// Whether one verified comparison sits at an index.
fn compare_at(instructions: &[TapscriptInstruction], index: usize, opcode: OpcodeId) -> bool {
    opcode_at(instructions, index, opcode) && opcode_at(instructions, index + 1, OpcodeId::Verify)
}

/// Whether one kept check's pattern sits at an index.
fn kept_check_at(
    instructions: &[TapscriptInstruction],
    target: &ReviewedElementsTapscriptDefinition,
    check: KeptCheck,
    index: usize,
) -> bool {
    use OpcodeId as O;
    let asset = (EncodingClass::ExplicitAsset, 32);
    let value = (EncodingClass::ExplicitValue, 8);
    match check {
        KeptCheck::SelfPositionPin => {
            opcode_at(instructions, index, O::PushCurrentInputIndex)
                && number_at(instructions, target, index + 1, 0)
                && opcode_at(instructions, index + 2, O::EqualVerify)
        }
        KeptCheck::InputZeroAsset => {
            field_check_at(instructions, target, index, O::InspectInputAsset, asset)
        }
        KeptCheck::InputZeroExplicitAmount => {
            field_check_at(instructions, target, index, O::InspectInputValue, value)
        }
        KeptCheck::InputZeroScriptVersion => version_check_at(
            instructions,
            target,
            index,
            O::InspectInputScriptPubKey,
            O::Drop,
        ),
        KeptCheck::PredecessorProgramVersion => version_check_at(
            instructions,
            target,
            index,
            O::InspectInputScriptPubKey,
            O::Concatenate,
        ),
        KeptCheck::OutputZeroAsset => {
            field_check_at(instructions, target, index, O::InspectOutputAsset, asset)
        }
        KeptCheck::OutputZeroExplicitAmount => {
            field_check_at(instructions, target, index, O::InspectOutputValue, value)
        }
        KeptCheck::OutputZeroScriptVersion => version_check_at(
            instructions,
            target,
            index,
            O::InspectOutputScriptPubKey,
            O::Concatenate,
        ),
        KeptCheck::PredecessorTweakVerify | KeptCheck::SuccessorTweakVerify => {
            keyed_check_at(instructions, index, O::TweakVerify)
        }
        KeptCheck::OperatorAuthorization => keyed_check_at(instructions, index, O::CheckSigVerify),
        KeptCheck::LeadLowerBound => compare_at(instructions, index, O::LessThanOrEqual64),
        KeptCheck::LeadUpperBound => compare_at(instructions, index, O::GreaterThanOrEqual64),
    }
}

/// Where one kept check sits in the decoded program.
///
/// The successor's tweak relation is the second occurrence of the
/// pattern the predecessor's is the first of, because the two are the
/// same instructions at different sites and the recipe emits the
/// predecessor's authentication before the successor's.
#[must_use]
pub fn kept_check_site(
    leaf: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
    check: KeptCheck,
) -> Option<usize> {
    let instructions = leaf.program().instructions();
    let wanted = usize::from(check == KeptCheck::SuccessorTweakVerify);
    instructions
        .iter()
        .enumerate()
        .filter(|(index, _)| kept_check_at(instructions, target, check, *index))
        .map(|(index, _)| index)
        .nth(wanted)
}

// --- The census ---------------------------------------------------------

/// What the decoded bytes carry and what they do not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckCensus {
    introspections: usize,
    positions: BTreeSet<i64>,
    absent: BTreeSet<OpcodeId>,
    forbidden_literals: usize,
    kept: BTreeSet<KeptCheck>,
}

impl CheckCensus {
    /// How many positional introspections the leaf performs.
    #[must_use]
    pub const fn introspections(&self) -> usize {
        self.introspections
    }

    /// Every position those introspections name.
    #[must_use]
    pub const fn positions(&self) -> &BTreeSet<i64> {
        &self.positions
    }

    /// Every removed primitive confirmed absent.
    #[must_use]
    pub const fn absent(&self) -> &BTreeSet<OpcodeId> {
        &self.absent
    }

    /// How many forbidden literal shapes every push was compared
    /// against.
    #[must_use]
    pub const fn forbidden_literals(&self) -> usize {
        self.forbidden_literals
    }

    /// Every kept check found.
    #[must_use]
    pub const fn kept(&self) -> &BTreeSet<KeptCheck> {
        &self.kept
    }
}

/// Census the decoded bytes: what came back, what stayed away, and what
/// is still there.
///
/// The one function the row's own gate is put through. A removed check
/// that returned and a kept check that went missing are each a refusal
/// by name, and both are decided by reading instructions.
///
/// # Errors
///
/// [`MaturityClosureRefusal::RemovedPrimitivePresent`],
/// [`MaturityClosureRefusal::IntrospectionWithoutLiteralZero`],
/// [`MaturityClosureRefusal::ProgramLiteralPresent`] or
/// [`MaturityClosureRefusal::KeptCheckAbsent`].
pub fn removed_and_kept_checks(
    leaf: &DecodedAnnouncementLeaf,
    forbidden: &ForbiddenLiterals,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<CheckCensus, MaturityClosureRefusal> {
    let instructions = leaf.program().instructions();
    let mut absent = BTreeSet::new();
    for opcode in REMOVED_PRIMITIVES {
        if let Some(instruction) = instructions
            .iter()
            .position(|found| is_opcode(found, opcode))
        {
            return Err(MaturityClosureRefusal::RemovedPrimitivePresent {
                opcode,
                instruction,
            });
        }
        absent.insert(opcode);
    }

    let mut introspections = 0;
    let mut positions = BTreeSet::new();
    for (index, instruction) in instructions.iter().enumerate() {
        if !POSITIONAL_INTROSPECTIONS
            .iter()
            .any(|opcode| is_opcode(instruction, *opcode))
        {
            continue;
        }
        let position = index
            .checked_sub(1)
            .and_then(|previous| at(instructions, previous))
            .and_then(|previous| pushed_number(previous, target))
            .ok_or(MaturityClosureRefusal::IntrospectionWithoutLiteralZero {
                instruction: index,
            })?;
        if position != 0 {
            return Err(MaturityClosureRefusal::IntrospectionWithoutLiteralZero {
                instruction: index,
            });
        }
        introspections += 1;
        positions.insert(position);
    }

    for (index, instruction) in instructions.iter().enumerate() {
        let Some(bytes) = push_bytes(instruction) else {
            continue;
        };
        if forbidden.shapes.iter().any(|shape| shape == bytes) {
            return Err(MaturityClosureRefusal::ProgramLiteralPresent {
                instruction: index,
                width: bytes.len(),
            });
        }
    }

    let mut kept = BTreeSet::new();
    for check in KeptCheck::ALL {
        if kept_check_site(leaf, target, check).is_none() {
            return Err(MaturityClosureRefusal::KeptCheckAbsent { check });
        }
        kept.insert(check);
    }

    Ok(CheckCensus {
        introspections,
        positions,
        absent,
        forbidden_literals: forbidden.shapes.len(),
        kept,
    })
}

// --- Locating the discharge table ---------------------------------------

/// The linked values, recovered from the decoded pushes at the sites the
/// record's own consumer census names.
///
/// # Errors
///
/// [`MaturityClosureRefusal::ConsumerSiteIsNotAPush`] or
/// [`MaturityClosureRefusal::ConsumerSitesDisagree`].
pub fn recovered_values(
    leaf: &DecodedAnnouncementLeaf,
    record: &StateAnnouncementProgram,
) -> Result<BTreeMap<StateProgramSymbol, StackItem>, MaturityClosureRefusal> {
    let instructions = leaf.program().instructions();
    let mut values = BTreeMap::new();
    for (&symbol, consumer) in record.consumers() {
        let mut recovered: Option<StackItem> = None;
        for &site in &consumer.sites {
            let instruction = at(instructions, site)
                .ok_or(MaturityClosureRefusal::ConsumerSiteIsNotAPush { instruction: site })?;
            let TapscriptInstruction::Push(item) = instruction else {
                return Err(MaturityClosureRefusal::ConsumerSiteIsNotAPush { instruction: site });
            };
            match &recovered {
                Some(previous) if previous != item => {
                    return Err(MaturityClosureRefusal::ConsumerSitesDisagree { symbol });
                }
                Some(_) => (),
                None => recovered = Some(item.clone()),
            }
        }
        let item = recovered.ok_or(MaturityClosureRefusal::ConsumerSitesDisagree { symbol })?;
        values.insert(symbol, item);
    }
    Ok(values)
}

/// Re-emit every named component from the recovered values.
///
/// The components come back through the same public builders the record
/// was composed with, so a component located here is one those builders
/// produce from the values the bytes carry.
fn reemitted_components(
    values: &BTreeMap<StateProgramSymbol, StackItem>,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<BTreeMap<StateProgramComponent, Vec<TapscriptInstruction>>, MaturityClosureRefusal> {
    let refused = || MaturityClosureRefusal::ReEmissionRefused;
    let structural_value = |symbol: StatePatternSymbol| {
        values
            .get(&StateProgramSymbol::Structural(symbol))
            .cloned()
            .ok_or_else(refused)
    };

    let mut structural_values = BTreeMap::new();
    for symbol in StatePatternSymbol::ALL.iter().copied() {
        structural_values.insert(symbol, structural_value(symbol)?);
    }
    let structural = StatePatternBindings::new(target, structural_values).map_err(|_| refused())?;
    let structural = state_structural_patterns(target, &structural).map_err(|_| refused())?;

    let mut semantic_values = BTreeMap::new();
    for symbol in StateAnnouncementSymbol::ALL.iter().copied() {
        let item = match symbol.structural() {
            Some(shared) => structural_value(shared)?,
            None => values
                .get(&StateProgramSymbol::Semantic(symbol))
                .cloned()
                .ok_or_else(refused)?,
        };
        semantic_values.insert(symbol, item);
    }
    let semantic =
        StateAnnouncementBindings::new(target, semantic_values).map_err(|_| refused())?;
    let semantic = state_announcement_patterns(target, &semantic).map_err(|_| refused())?;

    let key = values
        .get(&StateProgramSymbol::Operator(
            StateOperatorSymbol::CommittedOperatorKey,
        ))
        .cloned()
        .ok_or_else(refused)?;
    let operator = StateOperatorBindings::new(
        target,
        &BTreeMap::from([(StateOperatorSymbol::CommittedOperatorKey, key)]),
    )
    .map_err(|_| refused())?;
    let fragment = state_operator_fragment(&operator).map_err(|_| refused())?;
    let operator =
        build_state_operator_pattern(target, &operator, fragment).map_err(|_| refused())?;

    let mut emitted = BTreeMap::new();
    emitted.insert(
        StateProgramComponent::Operator(operator.id()),
        operator.fragment().instructions().to_vec(),
    );
    for component in structural.components() {
        emitted.insert(
            StateProgramComponent::Structural(component.id()),
            component.fragment().instructions().to_vec(),
        );
    }
    for component in semantic.components() {
        emitted.insert(
            StateProgramComponent::Semantic(*component.id()),
            component.fragment().instructions().to_vec(),
        );
    }
    Ok(emitted)
}

/// Where one relation's discharge was found, or that it claims none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedRow {
    relation: RelationId,
    class: StateDischargeClass,
    component: Option<StateProgramComponent>,
    located: Option<Range<usize>>,
    in_script: Option<Range<usize>>,
}

impl LocatedRow {
    /// The relation this row is about.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    /// The class the closure gives it.
    #[must_use]
    pub const fn class(&self) -> StateDischargeClass {
        self.class
    }

    /// The component located, where one was.
    #[must_use]
    pub const fn component(&self) -> Option<StateProgramComponent> {
        self.component
    }

    /// The instruction range the component was found at.
    #[must_use]
    pub const fn located(&self) -> Option<&Range<usize>> {
        self.located.as_ref()
    }

    /// The in-script half of an outstanding external requirement, where
    /// the leaf carries one.
    #[must_use]
    pub const fn in_script(&self) -> Option<&Range<usize>> {
        self.in_script.as_ref()
    }
}

/// Every row located, with the class census that locating counted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedDischarges {
    rows: Vec<LocatedRow>,
    census: BTreeMap<StateDischargeClass, usize>,
}

impl LocatedDischarges {
    /// Every row, in the closure's own order.
    #[must_use]
    pub fn rows(&self) -> &[LocatedRow] {
        &self.rows
    }

    /// How many relations each class discharges, counted by locating.
    #[must_use]
    pub const fn census(&self) -> &BTreeMap<StateDischargeClass, usize> {
        &self.census
    }
}

/// Locate one row's discharge in the decoded program.
fn locate_row(
    row: &StateCarrierRow,
    leaf: &DecodedAnnouncementLeaf,
    record: &StateAnnouncementProgram,
    emitted: &BTreeMap<StateProgramComponent, Vec<TapscriptInstruction>>,
) -> Result<LocatedRow, MaturityClosureRefusal> {
    let model_scope = || MaturityClosureRefusal::ModelScopeRowLocatesBytes {
        relation: row.relation().clone(),
    };
    let (component, located, in_script) = match row.linked() {
        StateLinkedCarrier::Component {
            component,
            leaf: role,
            range,
        } => {
            if *role != StateLeafRole::Announcement
                || row.class() != StateDischargeClass::Emitted
                || row.emitted() != &MaturityCarrier::Emitted(*component)
                || record.components().get(component) != Some(range)
            {
                return Err(MaturityClosureRefusal::ComponentNotLocated {
                    component: *component,
                    claimed: range.clone(),
                });
            }
            let instructions = leaf.program().instructions();
            let found = emitted.get(component).filter(|reemitted| {
                range.end <= instructions.len()
                    && &instructions[range.clone()] == reemitted.as_slice()
            });
            if found.is_none() {
                return Err(MaturityClosureRefusal::ComponentNotLocated {
                    component: *component,
                    claimed: range.clone(),
                });
            }
            (Some(*component), Some(range.clone()), None)
        }
        StateLinkedCarrier::DeploymentFact(_) => {
            if row.class() != StateDischargeClass::Deployment {
                return Err(model_scope());
            }
            (None, None, None)
        }
        StateLinkedCarrier::ExternalRequirement(requirement) => {
            if row.class() != StateDischargeClass::External {
                return Err(model_scope());
            }
            (None, None, in_script_half(requirement, record))
        }
        StateLinkedCarrier::ModelScope | StateLinkedCarrier::Vacuous => {
            if row.class() != StateDischargeClass::ModelScope {
                return Err(model_scope());
            }
            (None, None, None)
        }
    };
    Ok(LocatedRow {
        relation: row.relation().clone(),
        class: row.class(),
        component,
        located,
        in_script,
    })
}

/// The range an outstanding external requirement's in-script half
/// occupies, where it has one.
///
/// The operator's authorization is the one requirement the leaf answers
/// half of: the committed key and its verification are in the bytes,
/// while whether the operator is a member is settled elsewhere.
/// Constructibility has no in-script half at all.
fn in_script_half(
    requirement: &ExternalEvidenceRequirement,
    record: &StateAnnouncementProgram,
) -> Option<Range<usize>> {
    if !matches!(
        requirement,
        ExternalEvidenceRequirement::OperatorAuthorization { .. }
    ) {
        return None;
    }
    record
        .components()
        .iter()
        .find(|(component, _)| matches!(component, StateProgramComponent::Operator(_)))
        .map(|(_, range)| range.clone())
}

/// Locate every discharge the closure claims, and check the census it
/// publishes against the one locating counts.
///
/// # Errors
///
/// [`MaturityClosureRefusal::ComponentNotLocated`],
/// [`MaturityClosureRefusal::ModelScopeRowLocatesBytes`],
/// [`MaturityClosureRefusal::DischargeCensusDisagrees`], and everything
/// [`recovered_values`] refuses.
pub fn locate_discharges(
    leaf: &DecodedAnnouncementLeaf,
    closure: &StateCarrierClosure,
    record: &StateAnnouncementProgram,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<LocatedDischarges, MaturityClosureRefusal> {
    let emitted = reemitted_components(&recovered_values(leaf, record)?, target)?;
    let mut rows = Vec::new();
    for row in closure.rows() {
        rows.push(locate_row(row, leaf, record, &emitted)?);
    }

    let mut classes: BTreeMap<RelationId, StateDischargeClass> = BTreeMap::new();
    for row in &rows {
        classes.insert(row.relation.clone(), row.class);
    }
    let mut census: BTreeMap<StateDischargeClass, usize> = BTreeMap::new();
    for class in classes.values() {
        *census.entry(*class).or_insert(0) += 1;
    }

    let published = closure.census();
    for class in published.keys().chain(census.keys()) {
        let read = published.get(class).copied().unwrap_or_default();
        let located = census.get(class).copied().unwrap_or_default();
        if read != located {
            return Err(MaturityClosureRefusal::DischargeCensusDisagrees {
                class: *class,
                read,
                located,
            });
        }
    }
    Ok(LocatedDischarges { rows, census })
}

// --- The golden, recomputed from linked bytes ---------------------------

/// One figure of the golden evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoldenFigure {
    /// The tagged leaf hash of the linked announcement bytes.
    AnnouncementLeafHash,
    /// The static subtree's root, which one leaf makes that same hash.
    StaticRoot,
    /// The tagged leaf hash of the metadata commitment program.
    MetadataLeafHash,
    /// The outer branch over the two.
    MerkleRoot,
    /// The tagged tweak over the internal key and that branch.
    Tweak,
    /// The tweaked point's x coordinate.
    OutputKey,
    /// That point's y parity.
    Parity,
    /// The announcement role's control block.
    AnnouncementControlBytes,
}

/// The golden evidence, recomputed rather than restated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoldenEvidence {
    announcement_leaf_hash: [u8; 32],
    static_root: [u8; 32],
    metadata_leaf_hash: [u8; 32],
    merkle_root: [u8; 32],
    internal_key: [u8; 32],
    tweak: [u8; 32],
    output_key: [u8; 32],
    parity: bool,
    control_bytes: Vec<u8>,
}

impl GoldenEvidence {
    /// The tagged leaf hash of the linked bytes.
    #[must_use]
    pub const fn announcement_leaf_hash(&self) -> &[u8; 32] {
        &self.announcement_leaf_hash
    }

    /// The static subtree's root.
    #[must_use]
    pub const fn static_root(&self) -> &[u8; 32] {
        &self.static_root
    }

    /// The metadata leaf's hash.
    #[must_use]
    pub const fn metadata_leaf_hash(&self) -> &[u8; 32] {
        &self.metadata_leaf_hash
    }

    /// The outer root.
    #[must_use]
    pub const fn merkle_root(&self) -> &[u8; 32] {
        &self.merkle_root
    }

    /// The internal key, as the leaf's own pushes carry it.
    #[must_use]
    pub const fn internal_key(&self) -> &[u8; 32] {
        &self.internal_key
    }

    /// The tagged tweak digest.
    #[must_use]
    pub const fn tweak(&self) -> &[u8; 32] {
        &self.tweak
    }

    /// The x-only output key.
    #[must_use]
    pub const fn output_key(&self) -> &[u8; 32] {
        &self.output_key
    }

    /// Whether the output point has odd y.
    #[must_use]
    pub const fn parity(&self) -> bool {
        self.parity
    }

    /// The announcement role's control block.
    #[must_use]
    pub fn control_bytes(&self) -> &[u8] {
        &self.control_bytes
    }
}

/// The internal key the leaf itself pushes.
///
/// Read from the decoded pushes at the record's own sites, and required
/// to be one key: a tweak taken over one of several would say nothing
/// about a spend that ran another.
///
/// # Errors
///
/// [`MaturityClosureRefusal::InternalKeySitesDisagree`], and everything
/// [`recovered_values`] refuses.
pub fn internal_key_from_bytes(
    leaf: &DecodedAnnouncementLeaf,
    record: &StateAnnouncementProgram,
) -> Result<[u8; 32], MaturityClosureRefusal> {
    let values = recovered_values(leaf, record)?;
    let item = values
        .get(&StateProgramSymbol::Semantic(
            StateAnnouncementSymbol::InternalKey,
        ))
        .ok_or(MaturityClosureRefusal::InternalKeySitesDisagree)?;
    <[u8; 32]>::try_from(item.bytes()).map_err(|_| MaturityClosureRefusal::InternalKeySitesDisagree)
}

/// Recompute the golden evidence from the linked bytes, and compare it
/// with what the bundle carries.
///
/// # Errors
///
/// [`MaturityClosureRefusal::GoldenDisagrees`] naming the figure,
/// [`MaturityClosureRefusal::OutputKeyUndetermined`] when the curve
/// determines none, and
/// [`MaturityClosureRefusal::AnnouncementProgramAbsent`] when the bundle
/// retains no constructor.
pub fn recompute_golden(
    leaf: &DecodedAnnouncementLeaf,
    bundle: &CandidateLinkedMaturityBundle,
    target: &ReviewedElementsTapscriptDefinition,
    curve: &impl StateCurveCapability,
) -> Result<GoldenEvidence, MaturityClosureRefusal> {
    let disagrees = |figure| MaturityClosureRefusal::GoldenDisagrees { figure };
    let constructor = bundle
        .instances()
        .first()
        .ok_or(MaturityClosureRefusal::AnnouncementProgramAbsent)?
        .constructor();
    let version = target.definition().leaf_version();
    let internal_key = internal_key_from_bytes(leaf, bundle.record())?;

    let announcement_leaf_hash = leaf_hash(version, leaf.bytes());
    if bundle.static_subtree().root() != &announcement_leaf_hash {
        return Err(disagrees(GoldenFigure::AnnouncementLeafHash));
    }
    let static_root = *bundle.taptree().static_root();
    if static_root != announcement_leaf_hash {
        return Err(disagrees(GoldenFigure::StaticRoot));
    }

    let metadata_leaf_hash = leaf_hash(version, &constructor.leaf_program().encode(target));
    let metadata_recipe = constructor
        .control_recipe(StateLeafRole::MetadataCommitment)
        .map_err(|_| disagrees(GoldenFigure::MetadataLeafHash))?;
    if metadata_recipe.executing_leaf_hash != metadata_leaf_hash
        || bundle.taptree().metadata_hash() != &metadata_leaf_hash
        || metadata_recipe.siblings != vec![static_root]
    {
        return Err(disagrees(GoldenFigure::MetadataLeafHash));
    }

    let merkle_root = branch_hash(&metadata_leaf_hash, &static_root);
    if bundle.taptree().merkle_root() != &merkle_root || constructor.merkle_root() != &merkle_root {
        return Err(disagrees(GoldenFigure::MerkleRoot));
    }
    let digest = tweak(&internal_key, &merkle_root);
    if constructor.tweak_hash() != digest {
        return Err(disagrees(GoldenFigure::Tweak));
    }

    let outcome = curve.output_key(&internal_key, &merkle_root);
    let StateTweakOutcome::OutputKey { key, parity } = outcome else {
        return Err(MaturityClosureRefusal::OutputKeyUndetermined(outcome));
    };
    if constructor.output_key() != &key {
        return Err(disagrees(GoldenFigure::OutputKey));
    }
    if constructor.parity() != parity {
        return Err(disagrees(GoldenFigure::Parity));
    }

    let control_bytes = constructor
        .control_recipe(StateLeafRole::Announcement)
        .and_then(|recipe| recipe.control_bytes())
        .map_err(|_| disagrees(GoldenFigure::AnnouncementControlBytes))?;
    let mut expected = vec![version.get() | u8::from(parity)];
    expected.extend(internal_key);
    expected.extend(metadata_leaf_hash);
    if control_bytes != expected {
        return Err(disagrees(GoldenFigure::AnnouncementControlBytes));
    }

    Ok(GoldenEvidence {
        announcement_leaf_hash,
        static_root,
        metadata_leaf_hash,
        merkle_root,
        internal_key,
        tweak: digest,
        output_key: key,
        parity,
        control_bytes,
    })
}

// --- What a second deployment moves -------------------------------------

/// The sites two deployments' linked programs differ at, and the byte
/// spans those sites occupy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovedSites {
    sites: Vec<usize>,
    spans: Vec<Range<usize>>,
}

impl MovedSites {
    /// Every instruction index that moved.
    #[must_use]
    pub fn sites(&self) -> &[usize] {
        &self.sites
    }

    /// Their byte spans, in the same order.
    #[must_use]
    pub fn spans(&self) -> &[Range<usize>] {
        &self.spans
    }

    /// How many sites moved.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.sites.len()
    }

    /// Whether nothing moved at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }
}

/// Compare two deployments' linked bytes, and confine their difference
/// to the sites that moved.
///
/// The difference between two links of one record is a substitution or
/// it is a defect. Confining every differing byte to the encoded span of
/// an instruction that differs is what tells the two apart, and it is
/// decided over bytes rather than over the relocation census.
///
/// # Errors
///
/// [`MaturityClosureRefusal::LinkedProgramsAreNotComparable`] when the
/// two parses have different shapes, and
/// [`MaturityClosureRefusal::BytesDifferOutsideMovedSites`] naming the
/// first byte outside every moved span.
pub fn moved_sites(
    first: &DecodedAnnouncementLeaf,
    second: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<MovedSites, MaturityClosureRefusal> {
    let incomparable = || MaturityClosureRefusal::LinkedProgramsAreNotComparable;
    let left = first.program().instructions();
    let right = second.program().instructions();
    if left.len() != right.len() || first.bytes().len() != second.bytes().len() {
        return Err(incomparable());
    }

    let mut sites = Vec::new();
    let mut spans = Vec::new();
    for (index, (one, other)) in left.iter().zip(right).enumerate() {
        if one == other {
            continue;
        }
        let span = first
            .instruction_span(target, index)
            .ok_or_else(incomparable)?;
        if second.instruction_span(target, index) != Some(span.clone()) {
            return Err(incomparable());
        }
        sites.push(index);
        spans.push(span);
    }

    for (position, (one, other)) in first.bytes().iter().zip(second.bytes()).enumerate() {
        if one == other {
            continue;
        }
        if !spans.iter().any(|span| span.contains(&position)) {
            return Err(MaturityClosureRefusal::BytesDifferOutsideMovedSites { position });
        }
    }
    Ok(MovedSites { sites, spans })
}

// --- The literals the leaf compares against -----------------------------

/// Every literal the decoded leaf compares an observation against.
///
/// Read out of the bytes at the kept checks' own sites, so the operands
/// of the verdict below are the leaf's and not a fixture's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeafLiterals {
    pinned_input_index: i64,
    input_asset: Vec<u8>,
    input_amount: Vec<u8>,
    input_script_version: i64,
    output_asset: Vec<u8>,
    output_amount: Vec<u8>,
    output_script_version: i64,
}

impl LeafLiterals {
    /// The index the pin admits.
    #[must_use]
    pub const fn pinned_input_index(&self) -> i64 {
        self.pinned_input_index
    }

    /// Input zero's required asset.
    #[must_use]
    pub fn input_asset(&self) -> &[u8] {
        &self.input_asset
    }

    /// Input zero's required amount, as the leaf pushes it.
    #[must_use]
    pub fn input_amount(&self) -> &[u8] {
        &self.input_amount
    }

    /// Input zero's required script version.
    #[must_use]
    pub const fn input_script_version(&self) -> i64 {
        self.input_script_version
    }

    /// Output zero's required asset.
    #[must_use]
    pub fn output_asset(&self) -> &[u8] {
        &self.output_asset
    }

    /// Output zero's required amount, as the leaf pushes it.
    #[must_use]
    pub fn output_amount(&self) -> &[u8] {
        &self.output_amount
    }

    /// Output zero's required script version.
    #[must_use]
    pub const fn output_script_version(&self) -> i64 {
        self.output_script_version
    }
}

/// The bytes one kept check's payload push carries.
fn literal_at(
    leaf: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
    check: KeptCheck,
    offset: usize,
) -> Result<Vec<u8>, MaturityClosureRefusal> {
    let site = kept_check_site(leaf, target, check)
        .ok_or(MaturityClosureRefusal::KeptCheckAbsent { check })?;
    at(leaf.program().instructions(), site + offset)
        .and_then(push_bytes)
        .map(<[u8]>::to_vec)
        .ok_or(MaturityClosureRefusal::KeptCheckAbsent { check })
}

/// The script number one kept check's literal push carries.
fn number_literal_at(
    leaf: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
    check: KeptCheck,
    offset: usize,
) -> Result<i64, MaturityClosureRefusal> {
    let site = kept_check_site(leaf, target, check)
        .ok_or(MaturityClosureRefusal::KeptCheckAbsent { check })?;
    at(leaf.program().instructions(), site + offset)
        .and_then(|instruction| pushed_number(instruction, target))
        .ok_or(MaturityClosureRefusal::KeptCheckAbsent { check })
}

/// Read every literal the leaf compares against out of its own bytes.
///
/// # Errors
///
/// [`MaturityClosureRefusal::KeptCheckAbsent`] when a check whose
/// literal is wanted is not in the bytes.
pub fn leaf_literals(
    leaf: &DecodedAnnouncementLeaf,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<LeafLiterals, MaturityClosureRefusal> {
    Ok(LeafLiterals {
        pinned_input_index: number_literal_at(leaf, target, KeptCheck::SelfPositionPin, 1)?,
        input_asset: literal_at(leaf, target, KeptCheck::InputZeroAsset, 4)?,
        input_amount: literal_at(leaf, target, KeptCheck::InputZeroExplicitAmount, 4)?,
        input_script_version: number_literal_at(
            leaf,
            target,
            KeptCheck::InputZeroScriptVersion,
            2,
        )?,
        output_asset: literal_at(leaf, target, KeptCheck::OutputZeroAsset, 4)?,
        output_amount: literal_at(leaf, target, KeptCheck::OutputZeroExplicitAmount, 4)?,
        output_script_version: number_literal_at(
            leaf,
            target,
            KeptCheck::OutputZeroScriptVersion,
            2,
        )?,
    })
}

// --- The adoption gate, as real transactions ----------------------------

/// A foreign asset: anything but the singleton.
const FOREIGN_ASSET: [u8; 32] = [0xc7; 32];

/// A foreign destination's key, of the same public, meaningless kind.
const FOREIGN_KEY: [u8; 32] = [0xc8; 32];

/// One case of the adoption gate, as a transaction to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdoptionCase {
    /// The announcement beside further inputs and outputs of another
    /// asset, at positions the leaf never names.
    AcceptedWithForeignInputsAndOutputs,
    /// The singleton spent at input one, with a foreign coin at zero.
    RefusedSingletonAtInputOne,
    /// The successor at output one, with a foreign output at zero.
    RefusedSingletonAtOutputOne,
    /// The successor at output zero carrying less than the pinned
    /// amount, with the rest at output one.
    RefusedShortOutputZero,
}

impl AdoptionCase {
    /// Every case, in declaration order.
    pub const ALL: [Self; 4] = [
        Self::AcceptedWithForeignInputsAndOutputs,
        Self::RefusedSingletonAtInputOne,
        Self::RefusedSingletonAtOutputOne,
        Self::RefusedShortOutputZero,
    ];

    /// The stable step name a retained subject carries.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AcceptedWithForeignInputsAndOutputs => "accepted-foreign-inputs-and-outputs",
            Self::RefusedSingletonAtInputOne => "refused-singleton-at-input-one",
            Self::RefusedSingletonAtOutputOne => "refused-singleton-at-output-one",
            Self::RefusedShortOutputZero => "refused-short-output-zero",
        }
    }
}

/// One field the decoded leaf compares.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObservedField {
    /// The index the executing input sits at.
    ExecutingInputIndex,
    /// Input zero's asset.
    InputZeroAsset,
    /// Input zero's explicit amount.
    InputZeroAmount,
    /// Input zero's script version.
    InputZeroScriptVersion,
    /// Output zero's asset.
    OutputZeroAsset,
    /// Output zero's explicit amount.
    OutputZeroAmount,
    /// Output zero's script version.
    OutputZeroScriptVersion,
}

/// What the decoded leaf reads of one transaction.
///
/// Exactly the fields its introspections name, at exactly the positions
/// they name them at. Nothing else of the transaction is here, because
/// nothing else is observable to these bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeafObservation {
    executing_input_index: usize,
    input_zero_asset: AssetField,
    input_zero_value: ValueField,
    input_zero_program: Vec<u8>,
    output_zero_asset: AssetField,
    output_zero_value: ValueField,
    output_zero_program: Vec<u8>,
}

impl LeafObservation {
    /// Where the executing input sits.
    #[must_use]
    pub const fn executing_input_index(&self) -> usize {
        self.executing_input_index
    }

    /// Input zero's asset field.
    #[must_use]
    pub const fn input_zero_asset(&self) -> AssetField {
        self.input_zero_asset
    }

    /// Input zero's value field.
    #[must_use]
    pub const fn input_zero_value(&self) -> ValueField {
        self.input_zero_value
    }

    /// Input zero's program.
    #[must_use]
    pub fn input_zero_program(&self) -> &[u8] {
        &self.input_zero_program
    }

    /// Output zero's asset field.
    #[must_use]
    pub const fn output_zero_asset(&self) -> AssetField {
        self.output_zero_asset
    }

    /// Output zero's value field.
    #[must_use]
    pub const fn output_zero_value(&self) -> ValueField {
        self.output_zero_value
    }

    /// Output zero's program.
    #[must_use]
    pub fn output_zero_program(&self) -> &[u8] {
        &self.output_zero_program
    }
}

/// What the leaf's own bytes decide about one observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Every compared field equals the literal the leaf carries.
    Accepted,
    /// At least one does not, named in the order the leaf reaches them.
    Refused {
        /// The fields that differ.
        fields: Vec<ObservedField>,
    },
}

/// One adoption vector: the transaction, what the leaf observes of it,
/// what its bytes decide, and the subject a run would submit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdoptionVector {
    case: AdoptionCase,
    transaction: TargetTransaction,
    spent: Vec<SpentOutputCensusEntry>,
    bytes: Vec<u8>,
    subject: OperationStep,
    observed: LeafObservation,
    expected: Verdict,
}

impl AdoptionVector {
    /// The case this vector is of.
    #[must_use]
    pub const fn case(&self) -> AdoptionCase {
        self.case
    }

    /// The transaction itself.
    #[must_use]
    pub const fn transaction(&self) -> &TargetTransaction {
        &self.transaction
    }

    /// The spent outputs its inputs consume, in position order.
    ///
    /// An input's asset and amount are facts of the output it spends,
    /// and the leaf reads them by introspection, so a vector without
    /// this census would have no observation to be judged on.
    #[must_use]
    pub fn spent(&self) -> &[SpentOutputCensusEntry] {
        &self.spent
    }

    /// Its exact bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The capture-ready submission step a run would send.
    #[must_use]
    pub const fn subject(&self) -> &OperationStep {
        &self.subject
    }

    /// What the leaf observes of it.
    #[must_use]
    pub const fn observed(&self) -> &LeafObservation {
        &self.observed
    }

    /// What the leaf's bytes decide.
    #[must_use]
    pub const fn expected(&self) -> &Verdict {
        &self.expected
    }
}

/// One funded coin: where it is, and what it holds.
struct Coin {
    outpoint: Outpoint,
    spent: SpentOutputCensusEntry,
}

/// A fixture outpoint, standing for no funded output.
fn fixture_outpoint(byte: u8, index: u32) -> Result<Outpoint, MaturityClosureRefusal> {
    Outpoint::new(Txid::from_internal([byte; 32]), index)
        .map_err(MaturityClosureRefusal::AdoptionTransactionUnbuildable)
}

/// The amount a little-endian literal states.
fn amount_from(bytes: &[u8]) -> Result<u64, MaturityClosureRefusal> {
    <[u8; 8]>::try_from(bytes)
        .map(u64::from_le_bytes)
        .map_err(|_| MaturityClosureRefusal::AdoptionBytesDoNotRoundTrip)
}

/// The asset an identifier literal states.
fn asset_from(bytes: &[u8]) -> Result<TargetAssetId, MaturityClosureRefusal> {
    TargetAssetId::from_slice(bytes).map_err(MaturityClosureRefusal::AdoptionTransactionUnbuildable)
}

/// A foreign destination's program, of the version a taproot output is
/// read at.
fn foreign_program(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<Vec<u8>, MaturityClosureRefusal> {
    witness_program_script(target, 1, &FOREIGN_KEY)
        .map_err(MaturityClosureRefusal::AdoptionTransactionUnbuildable)
}

/// The singleton's own coin, holding exactly what the leaf requires.
fn singleton_coin(
    literals: &LeafLiterals,
    program: &[u8],
    index: u32,
) -> Result<Coin, MaturityClosureRefusal> {
    Ok(Coin {
        outpoint: fixture_outpoint(0x51, index)?,
        spent: SpentOutputCensusEntry::new(
            AssetField::Explicit(asset_from(&literals.input_asset)?),
            ValueField::Explicit(amount_from(&literals.input_amount)?),
            program.to_vec(),
        ),
    })
}

/// A coin of another asset entirely.
fn foreign_coin(
    target: &ReviewedElementsTapscriptDefinition,
    byte: u8,
    amount: u64,
) -> Result<Coin, MaturityClosureRefusal> {
    Ok(Coin {
        outpoint: fixture_outpoint(byte, 0)?,
        spent: SpentOutputCensusEntry::new(
            AssetField::Explicit(TargetAssetId::from_internal(FOREIGN_ASSET)),
            ValueField::Explicit(amount),
            foreign_program(target)?,
        ),
    })
}

/// The successor output, at whatever amount a case gives it.
fn successor_output(
    literals: &LeafLiterals,
    program: &[u8],
    amount: u64,
) -> Result<TargetOutput, MaturityClosureRefusal> {
    Ok(TargetOutput::new(
        AssetField::Explicit(asset_from(&literals.output_asset)?),
        ValueField::Explicit(amount),
        NonceField::Null,
        program.to_vec(),
    ))
}

/// An output of another asset entirely.
fn foreign_output(
    target: &ReviewedElementsTapscriptDefinition,
    amount: u64,
) -> Result<TargetOutput, MaturityClosureRefusal> {
    Ok(TargetOutput::new(
        AssetField::Explicit(TargetAssetId::from_internal(FOREIGN_ASSET)),
        ValueField::Explicit(amount),
        NonceField::Null,
        foreign_program(target)?,
    ))
}

/// A fee output, which the reviewed test recognizes by its empty
/// program.
const fn fee_output(amount: u64) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(TargetAssetId::from_internal(FOREIGN_ASSET)),
        ValueField::Explicit(amount),
        NonceField::Null,
        Vec::new(),
    )
}

/// One case's coins, outputs and executing position.
///
/// The foreign amounts balance in every case, so a vector is refused by
/// the leaf's own comparisons rather than by an arithmetic a node would
/// have raised first.
fn adoption_shape(
    case: AdoptionCase,
    literals: &LeafLiterals,
    program: &[u8],
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<(Vec<Coin>, Vec<TargetOutput>, usize), MaturityClosureRefusal> {
    let settled = amount_from(&literals.output_amount)?;
    Ok(match case {
        AdoptionCase::AcceptedWithForeignInputsAndOutputs => (
            vec![
                singleton_coin(literals, program, 0)?,
                foreign_coin(target, 0x52, 5_000)?,
                foreign_coin(target, 0x53, 3_000)?,
            ],
            vec![
                successor_output(literals, program, settled)?,
                foreign_output(target, 4_000)?,
                fee_output(1_000),
                foreign_output(target, 3_000)?,
            ],
            0,
        ),
        AdoptionCase::RefusedSingletonAtInputOne => (
            vec![
                foreign_coin(target, 0x52, 5_000)?,
                singleton_coin(literals, program, 0)?,
            ],
            vec![
                successor_output(literals, program, settled)?,
                foreign_output(target, 4_000)?,
                fee_output(1_000),
            ],
            1,
        ),
        AdoptionCase::RefusedSingletonAtOutputOne => (
            vec![
                singleton_coin(literals, program, 0)?,
                foreign_coin(target, 0x52, 5_000)?,
            ],
            vec![
                foreign_output(target, 4_000)?,
                successor_output(literals, program, settled)?,
                fee_output(1_000),
            ],
            0,
        ),
        AdoptionCase::RefusedShortOutputZero => (
            vec![singleton_coin(literals, program, 0)?],
            vec![
                successor_output(literals, program, 0)?,
                successor_output(literals, program, settled)?,
            ],
            0,
        ),
    })
}

/// What the leaf reads of one built transaction.
fn observe(
    transaction: &TargetTransaction,
    spent: &[SpentOutputCensusEntry],
    executing: usize,
) -> Result<LeafObservation, MaturityClosureRefusal> {
    let absent = || MaturityClosureRefusal::AdoptionBytesDoNotRoundTrip;
    let input = spent.first().ok_or_else(absent)?;
    let output = transaction.outputs().first().ok_or_else(absent)?;
    Ok(LeafObservation {
        executing_input_index: executing,
        input_zero_asset: input.asset(),
        input_zero_value: input.value(),
        input_zero_program: input.program().to_vec(),
        output_zero_asset: output.asset(),
        output_zero_value: output.value(),
        output_zero_program: output.program().to_vec(),
    })
}

/// Whether one program is the witness program of one version over its
/// own payload.
///
/// Asked by rebuilding the program the reviewed push contract would
/// produce for that version and that payload, so the version is decided
/// by the target's own encoding rather than by a byte written here.
fn program_states_version(
    target: &ReviewedElementsTapscriptDefinition,
    program: &[u8],
    version: i64,
) -> bool {
    let Ok(version) = u8::try_from(version) else {
        return false;
    };
    if program.len() < 2 {
        return false;
    }
    witness_program_script(target, version, &program[2..]).is_ok_and(|expected| expected == program)
}

/// Whether one asset field is the explicit asset a literal names.
fn asset_matches(field: AssetField, literal: &[u8]) -> bool {
    matches!(field, AssetField::Explicit(asset) if asset.internal().as_slice() == literal)
}

/// Whether one value field is the explicit amount a literal names, in
/// the order the leaf compares it in.
///
/// An explicit amount is stored big-endian in a transaction field and
/// pushed to the stack little-endian, so one value has two orders and
/// the comparison has to be made in the one the leaf performs.
fn amount_matches(field: ValueField, literal: &[u8]) -> bool {
    matches!(field, ValueField::Explicit(amount) if amount.to_le_bytes().as_slice() == literal)
}

/// What the leaf's own literals decide about one observation.
///
/// The comparison is the leaf's, field for field, in the order its
/// instructions reach them. Output zero's program is deliberately absent
/// from the decision: it is authenticated by a curve relation over the
/// witness-supplied metadata rather than by an equality against a
/// literal, so no byte comparison here could settle it.
#[must_use]
pub fn verdict(
    observation: &LeafObservation,
    literals: &LeafLiterals,
    target: &ReviewedElementsTapscriptDefinition,
) -> Verdict {
    let mut fields = Vec::new();
    if i64::try_from(observation.executing_input_index) != Ok(literals.pinned_input_index) {
        fields.push(ObservedField::ExecutingInputIndex);
    }
    if !asset_matches(observation.input_zero_asset, &literals.input_asset) {
        fields.push(ObservedField::InputZeroAsset);
    }
    if !amount_matches(observation.input_zero_value, &literals.input_amount) {
        fields.push(ObservedField::InputZeroAmount);
    }
    if !program_states_version(
        target,
        &observation.input_zero_program,
        literals.input_script_version,
    ) {
        fields.push(ObservedField::InputZeroScriptVersion);
    }
    if !asset_matches(observation.output_zero_asset, &literals.output_asset) {
        fields.push(ObservedField::OutputZeroAsset);
    }
    if !amount_matches(observation.output_zero_value, &literals.output_amount) {
        fields.push(ObservedField::OutputZeroAmount);
    }
    if !program_states_version(
        target,
        &observation.output_zero_program,
        literals.output_script_version,
    ) {
        fields.push(ObservedField::OutputZeroScriptVersion);
    }
    if fields.is_empty() {
        Verdict::Accepted
    } else {
        Verdict::Refused { fields }
    }
}

/// Build one adoption case as a real target transaction, and record
/// what the leaf's own bytes decide about it.
///
/// # Errors
///
/// [`MaturityClosureRefusal::AdoptionTransactionUnbuildable`] when the
/// transaction layer refuses a part,
/// [`MaturityClosureRefusal::AdoptionBytesDoNotRoundTrip`] when its
/// bytes do not decode back to it, and everything [`leaf_literals`]
/// refuses.
pub fn adoption_transaction(
    case: AdoptionCase,
    bundle: &CandidateLinkedMaturityBundle,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<AdoptionVector, MaturityClosureRefusal> {
    let bytes = linked_announcement_bytes(bundle, target)?;
    let leaf = decode_announcement_leaf(target, &bytes)?;
    let literals = leaf_literals(&leaf, target)?;
    let program = bundle
        .instances()
        .first()
        .ok_or(MaturityClosureRefusal::AnnouncementProgramAbsent)?
        .constructor()
        .output_program();

    let (coins, outputs, executing) = adoption_shape(case, &literals, &program, target)?;
    let inputs: Vec<TargetInput> = coins
        .iter()
        .map(|coin| TargetInput::new(coin.outpoint, u32::MAX))
        .collect();
    let witnesses = vec![InputWitness::new(Vec::new()); inputs.len()];
    let spent: Vec<SpentOutputCensusEntry> = coins.into_iter().map(|coin| coin.spent).collect();

    let transaction = TargetTransaction::new(2, inputs, outputs, 0, witnesses)
        .map_err(MaturityClosureRefusal::AdoptionTransactionUnbuildable)?;
    let encoded = transaction.encode();
    if TargetTransaction::decode(&encoded).as_ref() != Ok(&transaction) {
        return Err(MaturityClosureRefusal::AdoptionBytesDoNotRoundTrip);
    }

    let observed = observe(&transaction, &spent, executing)?;
    let expected = verdict(&observed, &literals, target);
    let subject = OperationStep::new(
        case.name(),
        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
            transaction_bytes: encoded.clone(),
        })),
    );
    Ok(AdoptionVector {
        case,
        transaction,
        spent,
        bytes: encoded,
        subject,
        observed,
        expected,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        GoldenFigure, KeptCheck, LinkRefusal, MaturityClosureRefusal, OpcodeId, OperationId,
        RelationId, StateAnnouncementSymbol, StateDischargeClass, StateProgramComponent,
        StateProgramSymbol, StateTweakOutcome, TransactionRefusal,
    };
    use linker::StateLinkRefusal;
    use realization::{RelationKind, RelationSubject};
    use tapscript::{StateConstructorRefusal, StatePatternId};

    /// Every refusal this module owns, matched with no catch-all.
    ///
    /// The guard is the pattern list and nothing else. A variant added to
    /// the root stops this crate's test target compiling until somebody
    /// edits this function, which is the point at which they read this
    /// paragraph and learn that the new variant owes either a test that
    /// reaches it or an argument that nothing can reach it. A catch-all
    /// arm would make that stop silent again, which is the whole of why
    /// there is none here and why adding one later to quiet an
    /// inconvenient addition would give up the only guarantee this
    /// function has.
    ///
    /// The census lives beside the vocabulary rather than beside the
    /// closure tests because the root is `#[non_exhaustive]`, as the
    /// refusal roots of this workspace are: a match outside the crate
    /// that declares the enum must carry a wildcard, so the only place an
    /// exhaustive match over these names can be written is here.
    ///
    /// A match arm is not a test. What the comments below record is which
    /// test asserts each variant as an outcome — a real call refusing by
    /// that name — and, for the ten this module raises where no public
    /// call arranges them, the reason the raising code gives. Those ten
    /// are guards against a later change making one of them reachable
    /// while nothing exercises it, and a fixture that cannot arrange a
    /// failure is an argument rather than an omission.
    ///
    /// The arms are grouped rather than named one at a time because
    /// exhaustiveness is a property of the pattern set and not of the arm
    /// bodies: twenty-five bodies that each do nothing would be
    /// twenty-five copies of one nothing, which is a second place for a
    /// name to drift and a lint to collapse.
    fn every_closure_refusal_is_censused(refusal: &MaturityClosureRefusal) {
        match refusal {
            // Unreachable: the reviewed contract is a first-party
            // constant validated once behind a static, and no caller
            // value reaches the validation that could refuse it.
            MaturityClosureRefusal::ReviewedTargetInvalid
            // Unreachable: the plan is derived from the architecture
            // constant under the search limits this module writes down,
            // neither of which a caller supplies.
            | MaturityClosureRefusal::PlanUnavailable
            // Unreachable: the record is composed from this module's own
            // constants by the same route, so nothing a caller passes can
            // make the composition refuse.
            | MaturityClosureRefusal::RecordUnavailable
            // Reached by `a_zero_lead_minimum_leaves_the_sources_unavailable`:
            // the lead window is the one source built from a supplied
            // value, and its own constructor refuses a zero minimum.
            | MaturityClosureRefusal::SourcesUnavailable
            // Reached by `a_curve_that_finds_no_point_refuses_the_link_by_the_constructors_own_name`.
            | MaturityClosureRefusal::LinkRefused(..)
            // Unreachable: a bundle's only origin is the link, which
            // inserts the announcement program and retains one applied
            // constructor, and the bundle type offers no constructor of
            // its own.
            | MaturityClosureRefusal::AnnouncementProgramAbsent
            // Reached by `bytes_that_end_inside_a_push_do_not_decode`.
            | MaturityClosureRefusal::LeafBytesDoNotDecode
            // Unreachable: the decoder refuses any push that is not in
            // the minimal form the reviewed rule names, and the encoder
            // writes that same form, so a parse that succeeded re-encodes
            // to the bytes it was taken from.
            | MaturityClosureRefusal::LeafBytesDoNotRoundTrip
            // Reached by `a_removed_primitive_that_returns_is_refused_by_name`.
            | MaturityClosureRefusal::RemovedPrimitivePresent { .. }
            // Reached by `an_introspection_that_names_no_literal_zero_is_refused`.
            | MaturityClosureRefusal::IntrospectionWithoutLiteralZero { .. }
            // Reached by `a_predecessor_program_literal_that_returns_is_refused`.
            | MaturityClosureRefusal::ProgramLiteralPresent { .. }
            // Reached by `a_kept_check_that_goes_missing_is_refused_by_name`.
            | MaturityClosureRefusal::KeptCheckAbsent { .. }
            // Reached by `a_consumer_site_the_bytes_do_not_push_is_refused`.
            | MaturityClosureRefusal::ConsumerSiteIsNotAPush { .. }
            // Reached by the second half of that same test.
            | MaturityClosureRefusal::ConsumerSitesDisagree { .. }
            // Reached by `an_internal_key_of_another_width_refuses_re_emission`:
            // the semantic bindings admit an internal key of thirty-two
            // bytes and no other width.
            | MaturityClosureRefusal::ReEmissionRefused
            // Reached by `a_component_whose_primitive_moved_is_not_located`.
            | MaturityClosureRefusal::ComponentNotLocated { .. }
            // Unreachable: the link settles every row's class against the
            // carrier that row records, and a row is read-only outside
            // the linker, so no closure a caller can hold carries a
            // model-scope row that claims bytes.
            | MaturityClosureRefusal::ModelScopeRowLocatesBytes { .. }
            // Unreachable: the published census and the census counted by
            // locating are one derivation — relations counted once per
            // class — over one set of rows a caller cannot assemble.
            | MaturityClosureRefusal::DischargeCensusDisagrees { .. }
            // Reached by `a_golden_over_another_deployments_bytes_disagrees_by_name`.
            | MaturityClosureRefusal::GoldenDisagrees { .. }
            // Reached by `an_internal_key_of_another_width_is_not_one_key`.
            | MaturityClosureRefusal::InternalKeySitesDisagree
            // Reached by `a_curve_that_determines_no_output_key_is_named_with_its_outcome`.
            | MaturityClosureRefusal::OutputKeyUndetermined(..)
            // Unreachable: an encoding is a function of the instruction
            // list, so where two parses have equal counts, equal byte
            // lengths and agreeing spans, a byte that differs lies inside
            // an instruction that differs, whose span is the span
            // collected; parses whose spans disagree are refused as
            // incomparable before the bytes are walked at all.
            | MaturityClosureRefusal::BytesDifferOutsideMovedSites { .. }
            // Reached by `two_leaves_of_different_shapes_are_not_comparable`.
            | MaturityClosureRefusal::LinkedProgramsAreNotComparable
            // Unreachable: the entry re-derives its leaf from the bundle
            // it is handed, so the widths it could refuse on are the
            // link's own — a thirty-two-byte asset identifier and an
            // eight-byte explicit amount — and every case it builds
            // states at least one coin and one output.
            | MaturityClosureRefusal::AdoptionTransactionUnbuildable(..)
            // Unreachable: those same widths, and a transaction this
            // entry encoded itself decodes back to itself through the
            // codec the adoption vectors already read.
            | MaturityClosureRefusal::AdoptionBytesDoNotRoundTrip => (),
        }
    }

    #[test]
    fn the_whole_closure_refusal_root_is_censused() {
        // The guarantee is the match above and it is a compile-time one.
        // This test exists so the guard is reached by a test run and so
        // one value of every variant has to be nameable here: a variant
        // added to the root stops this list compiling beside the match.
        //
        // The values are constructed rather than provoked, and that is
        // deliberate. A constructed refusal is evidence about the
        // vocabulary and never about the code that raises it, so reaching
        // a variant by name stays with the closure tests, which assert
        // each of the fifteen as the outcome of a real call.
        for refusal in [
            MaturityClosureRefusal::ReviewedTargetInvalid,
            MaturityClosureRefusal::PlanUnavailable,
            MaturityClosureRefusal::RecordUnavailable,
            MaturityClosureRefusal::SourcesUnavailable,
            MaturityClosureRefusal::LinkRefused(LinkRefusal::StateLink(
                StateLinkRefusal::ConstructorApplication(
                    StateConstructorRefusal::InternalKeyNotAPoint,
                ),
            )),
            MaturityClosureRefusal::AnnouncementProgramAbsent,
            MaturityClosureRefusal::LeafBytesDoNotDecode,
            MaturityClosureRefusal::LeafBytesDoNotRoundTrip,
            MaturityClosureRefusal::RemovedPrimitivePresent {
                opcode: OpcodeId::InspectNumInputs,
                instruction: 0,
            },
            MaturityClosureRefusal::IntrospectionWithoutLiteralZero { instruction: 0 },
            MaturityClosureRefusal::ProgramLiteralPresent {
                instruction: 0,
                width: 32,
            },
            MaturityClosureRefusal::KeptCheckAbsent {
                check: KeptCheck::SelfPositionPin,
            },
            MaturityClosureRefusal::ConsumerSiteIsNotAPush { instruction: 0 },
            MaturityClosureRefusal::ConsumerSitesDisagree {
                symbol: StateProgramSymbol::Semantic(StateAnnouncementSymbol::InternalKey),
            },
            MaturityClosureRefusal::ReEmissionRefused,
            MaturityClosureRefusal::ComponentNotLocated {
                component: StateProgramComponent::Structural(
                    StatePatternId::StateCoordinatorRoleV1,
                ),
                claimed: 0..1,
            },
            MaturityClosureRefusal::ModelScopeRowLocatesBytes {
                relation: RelationId::new(
                    OperationId::AnnounceMaturity,
                    RelationKind::Lifecycle,
                    RelationSubject::Operation,
                ),
            },
            MaturityClosureRefusal::DischargeCensusDisagrees {
                class: StateDischargeClass::Emitted,
                read: 13,
                located: 12,
            },
            MaturityClosureRefusal::GoldenDisagrees {
                figure: GoldenFigure::StaticRoot,
            },
            MaturityClosureRefusal::InternalKeySitesDisagree,
            MaturityClosureRefusal::OutputKeyUndetermined(StateTweakOutcome::InternalKeyNotAPoint),
            MaturityClosureRefusal::BytesDifferOutsideMovedSites { position: 0 },
            MaturityClosureRefusal::LinkedProgramsAreNotComparable,
            MaturityClosureRefusal::AdoptionTransactionUnbuildable(
                TransactionRefusal::EmptyInputCensus,
            ),
            MaturityClosureRefusal::AdoptionBytesDoNotRoundTrip,
        ] {
            every_closure_refusal_is_censused(&refusal);
        }
    }
}
