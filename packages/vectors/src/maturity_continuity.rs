//! Constructor projections from exact submitted maturity-announcement bytes.
//!
//! The input outpoint binds supplied funding evidence; the transaction does not contain the spent asset, amount or program. The witnessed predecessor encoding must agree with the retained recipe before either constructor is reconstructed. The successor is the realization's transition over that decoded predecessor and witnessed request, tested against the observed output commitment.
//!
//! Static byte bindings, whole retained-descriptor equality and semantic comparisons are separate results. Reusing one retained tree on both sides states the descriptor equality's premise; a matching root alone establishes neither descriptor identity nor hidden-subtree contents. The successor field comparison checks reconstruction consistency, not an independently observed semantic tuple. A different successor commitment is refused without attributing its cause to a semantic change or another static tree.
//!
//! Fixed-nonce reconstruction and host leastness are separate evidence. These projections verify no signature, authenticate no branch freshness and establish no accepted readback. Without a stated acceptance, the obligation remains outstanding, and experimental byte mutations answer no matrix row.
//!
//! A stated acceptance is checked by recomputing its schedule, readback bytes and identities against the projected bytes; a disagreement is refused. The projector makes no target observation of its own. An outstanding claim remains outstanding, while an established claim carries only the standing of the transcript or corpus that minted it.
//!
//! The retained witness schedule supplies the reconstruction recipe. Legalization
//! checks its carried width against the observed item; variable metadata is
//! rebuilt with the codec's constants before strict decoding. The same byte
//! bindings then compare the linked leaf and control path.

use linker::{
    CandidateDeploymentIdentity, CandidateLinkedMaturityBundle, StateLinkRefusal,
    state_bundle_continuity,
};
use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, Maturity, MaturityTransitionRefusal,
    STATE_METADATA_BYTES, STATE_METADATA_LAYOUT, STATE_METADATA_VARIABLE_BYTES, StateField,
    StateMetadata, StateMetadataRefusal, StateRepresentationNonce, TransactionSide,
    announce_maturity, decode_state_metadata, encode_state_metadata, rebuild_state_metadata,
};
use tapscript::{
    CandidateStateConstructor, StateConstructorRefusal, StateControlRecipe, StateCurveCapability,
    StateFieldCommitment, StateLeafRole, StateNonceEvidence, StateStaticSubtree, StateTweakOutcome,
    StateWitnessLegalization, StateWitnessLoweringRefusal, StateWitnessSchedule,
    legalize_state_witness_schedule, state_metadata_leaf_program, state_output_program_at_nonce,
};
use target_elements::{ResourceBound, ResourceDimension, ReviewedElementsTapscriptDefinition};
use target_elements_conformance::constructor::tagged::sha256;
use transaction::bytes::{AssetId, Outpoint, TargetTransaction, Txid};
use transaction::error::TransactionRefusal;
use transaction::operator_right::BranchContext;
use transaction::taproot::{branch_hash, leaf_hash, tagged_hash};

use crate::matrix::{EvidenceBoundary, MutationLayer};
use crate::maturity_closure::{
    MaturityClosureRefusal, OracleStateCurve, closure_target, linked_announcement_bytes,
};
use crate::maturity_evidence::{
    MaturityEvidenceCensus, MaturityGuaranteeQuantifier, MaturityObservationClass,
};
use crate::maturity_native::{MaturityAcceptanceObligation, MaturityAcceptanceRoute};
use crate::maturity_safety::{
    MaturityIntendedCarrier, MaturityRowBoundary, MaturitySafetyRow, MaturitySafetySection, rows,
};
use crate::subject::ExperimentalSubject;

/// The operand whose independent evidence computation disagreed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityEvidenceMismatch {
    /// The prefix was not exactly the consecutive candidates below the selection.
    NonceOrder,
    /// A candidate's independently evaluated refusal differed.
    RejectionCause,
    /// A selected or witnessed candidate was not admissible.
    Admissibility,
    /// The encoded tuple or its commitment differed.
    Metadata,
    /// The publicly derived internal key differed.
    InternalKey,
    /// The outer branch root differed.
    BranchRoot,
    /// The tagged tweak digest differed.
    Digest,
    /// The output point or its serialized program differed.
    OutputPoint,
    /// The output parity differed.
    Parity,
    /// The independently assembled announcement path differed.
    Recipe,
}

/// The limited claim made by a checked host search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityLeastnessResidual {
    /// Search establishes only the host's bounded leastness fact.
    HostSearchOnly,
}

impl MaturityLeastnessResidual {
    /// The constructor's unchanged residual, including later admissible nonces.
    #[must_use]
    pub const fn text(self) -> &'static str {
        StateNonceEvidence::RESIDUAL
    }
}

/// A separately evaluated metadata candidate, including the stage reached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityNonceCheck {
    candidate: StateRepresentationNonce,
    encoding: Vec<u8>,
    leaf_digest: [u8; 32],
    retained_root: [u8; 32],
    curve_outcome: Option<StateTweakOutcome>,
}

impl MaturityNonceCheck {
    /// The nonce evaluated by this computation.
    #[must_use]
    pub const fn candidate(&self) -> StateRepresentationNonce {
        self.candidate
    }
    /// Canonical metadata bytes rebuilt for the candidate.
    #[must_use]
    pub fn encoding(&self) -> &[u8] {
        &self.encoding
    }
    /// The tagged hash of the public metadata leaf builder's bytes.
    #[must_use]
    pub const fn leaf_digest(&self) -> &[u8; 32] {
        &self.leaf_digest
    }
    /// The supplied static commitment against which ordering is checked.
    #[must_use]
    pub const fn retained_root(&self) -> &[u8; 32] {
        &self.retained_root
    }
    /// Whether the metadata commitment is on the required outer side.
    #[must_use]
    pub fn branch_admissible(&self) -> bool {
        self.leaf_digest <= self.retained_root
    }
    /// The real curve evaluation, absent when branch ordering already refused.
    #[must_use]
    pub const fn curve_outcome(&self) -> Option<StateTweakOutcome> {
        self.curve_outcome
    }
}

/// Every lower rejection checked independently beside host and witnessed selections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityCheckedPrefix {
    side: TransactionSide,
    rejected: Vec<(MaturityNonceCheck, StateConstructorRefusal)>,
    selected: MaturityNonceCheck,
    witnessed: MaturityNonceCheck,
    host_least: bool,
    residual: MaturityLeastnessResidual,
}

impl MaturityCheckedPrefix {
    /// The transaction side whose semantic tuple was evaluated.
    #[must_use]
    pub const fn side(&self) -> TransactionSide {
        self.side
    }
    /// Consecutive lower candidates and their independently reproduced refusals.
    #[must_use]
    pub fn rejected(&self) -> &[(MaturityNonceCheck, StateConstructorRefusal)] {
        &self.rejected
    }
    /// The independently admissible host selection.
    #[must_use]
    pub const fn selected(&self) -> &MaturityNonceCheck {
        &self.selected
    }
    /// The separately checked representation actually named by the submission.
    #[must_use]
    pub const fn witnessed(&self) -> &MaturityNonceCheck {
        &self.witnessed
    }
    /// Equality of witnessed and host-selected nonces, not a target-validity rule.
    #[must_use]
    pub const fn is_host_least(&self) -> bool {
        self.host_least
    }
    /// The bounded host claim, without excluding later admissible representations.
    #[must_use]
    pub const fn residual(&self) -> MaturityLeastnessResidual {
        self.residual
    }
}

/// The origin claimed for submitted bytes, separate from acceptance and observation class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityByteSource {
    /// A planner driven in process, with no node observation.
    NodeFreeSubmitReady,
    /// A submission retained by a content-addressed corpus.
    ArchivedSubmission {
        /// The corpus report's content address, not an accepted transaction identity.
        run_address: String,
    },
}

/// Funding facts supplied alongside bytes that cannot contain their spent output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityFundedPredecessor {
    /// The reported spent outpoint.
    pub outpoint: Outpoint,
    /// The reported asset in internal byte order.
    pub asset: AssetId,
    /// The reported explicit amount.
    pub amount: u64,
    /// The reported spent output program.
    pub program: Vec<u8>,
}

/// Caller-supplied operands, with no caller-supplied verdict.
pub struct MaturityProjectionInput<'a> {
    /// The stated byte origin; the projector does not authenticate a capture.
    pub source: MaturityByteSource,
    /// The complete submitted serialization, including its witness.
    pub submitted_bytes: &'a [u8],
    /// The spent output as supplied by funding evidence.
    pub funded: &'a MaturityFundedPredecessor,
    /// Caller-stated branch identity and checkpoint, with no freshness claim.
    pub branch: BranchContext,
    /// The retained recipe, including bounds, policy and predecessor instance.
    pub bundle: &'a CandidateLinkedMaturityBundle,
    /// The caller's deployment identity, checked against the retained deployment.
    pub identity: &'a CandidateDeploymentIdentity,
}

/// Exact named byte operands of a binding comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityByteComparison {
    witnessed: Vec<u8>,
    reconstructed: Vec<u8>,
}

impl MaturityByteComparison {
    fn new(witnessed: &[u8], reconstructed: &[u8]) -> Self {
        Self {
            witnessed: witnessed.to_vec(),
            reconstructed: reconstructed.to_vec(),
        }
    }

    /// The first operand: submitted bytes for bindings, projected facts for independent evidence.
    #[must_use]
    pub fn witnessed(&self) -> &[u8] {
        &self.witnessed
    }

    /// The comparison operand reconstructed from the retained recipe and decoded metadata.
    #[must_use]
    pub fn reconstructed(&self) -> &[u8] {
        &self.reconstructed
    }

    /// Whether these exact operands agree.
    #[must_use]
    pub fn agrees(&self) -> bool {
        self.witnessed == self.reconstructed
    }
}

/// One field comparison with both sides and their byte ranges retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityFieldComparison {
    left: StateFieldCommitment,
    right: StateFieldCommitment,
}

impl MaturityFieldComparison {
    /// The realization-owned field selector.
    #[must_use]
    pub const fn field(&self) -> StateField {
        self.left.field
    }

    /// The left operand, named by the collection containing this comparison.
    #[must_use]
    pub const fn left(&self) -> &StateFieldCommitment {
        &self.left
    }

    /// The right operand, named by the collection containing this comparison.
    #[must_use]
    pub const fn right(&self) -> &StateFieldCommitment {
        &self.right
    }

    /// Equality of the selected field and committed bytes, independent of side labels.
    #[must_use]
    pub fn agrees(&self) -> bool {
        (self.left.field, &self.left.range, &self.left.bytes)
            == (self.right.field, &self.right.range, &self.right.bytes)
    }
}

/// The decoded predecessor versus the realization's expected successor, and optional reconstruction consistency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturitySemanticComparison {
    predecessor_against_expected: Vec<MaturityFieldComparison>,
    reconstructed_against_expected: Option<Vec<MaturityFieldComparison>>,
    expected_successor: StateMetadata,
    requested_cycle: Cycle,
}

impl MaturitySemanticComparison {
    /// Six comparisons: decoded predecessor on the left, expected successor on the right.
    #[must_use]
    pub fn predecessor_against_expected(&self) -> &[MaturityFieldComparison] {
        &self.predecessor_against_expected
    }

    /// Reconstructed successor on the left, expected successor on the right; absent before reconstruction.
    ///
    /// This is a consistency check, because the reconstruction takes its semantic input from the realization. It is not a second independently observed successor tuple.
    #[must_use]
    pub fn reconstructed_against_expected(&self) -> Option<&[MaturityFieldComparison]> {
        self.reconstructed_against_expected.as_deref()
    }

    /// The realization's result for the decoded predecessor and witnessed request.
    #[must_use]
    pub const fn expected_successor(&self) -> &StateMetadata {
        &self.expected_successor
    }

    /// The cycle read from witness item two, in big-endian order.
    #[must_use]
    pub const fn requested_cycle(&self) -> Cycle {
        self.requested_cycle
    }

    /// Whether all five unaffected fields agree and maturity moved to the request.
    #[must_use]
    pub fn transition_agrees(&self) -> bool {
        self.predecessor_against_expected
            .iter()
            .filter(|comparison| comparison.field() != StateField::Maturity)
            .all(MaturityFieldComparison::agrees)
            && self
                .predecessor_against_expected
                .iter()
                .filter(|comparison| comparison.field() == StateField::Maturity)
                .all(|comparison| !comparison.agrees())
            && self.expected_successor.maturity
                == Maturity::Announced {
                    cycle: self.requested_cycle,
                }
    }

    /// Whether the fixed-nonce reconstruction agrees field by field, if it was reached.
    #[must_use]
    pub fn reconstruction_agrees(&self) -> Option<bool> {
        self.reconstructed_against_expected
            .as_ref()
            .map(|comparisons| comparisons.iter().all(MaturityFieldComparison::agrees))
    }
}

/// Comparisons already reached before a commitment-binding refusal.
///
/// Absence means not evaluated, never a semantic failure. In particular the static-root, leaf, control and predecessor-prefix checks precede the realization transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityProjectionDiagnostics {
    static_root: Option<MaturityByteComparison>,
    leaf_script: Option<MaturityByteComparison>,
    control_block: Option<MaturityByteComparison>,
    predecessor_prefix: Option<MaturityByteComparison>,
    semantic: Option<MaturitySemanticComparison>,
    output_program: Option<MaturityByteComparison>,
    successor_prefix: Option<MaturityByteComparison>,
}

impl MaturityProjectionDiagnostics {
    const fn empty() -> Self {
        Self {
            static_root: None,
            leaf_script: None,
            control_block: None,
            predecessor_prefix: None,
            semantic: None,
            output_program: None,
            successor_prefix: None,
        }
    }
    /// Witnessed root versus retained root, if compared.
    #[must_use]
    pub const fn static_root(&self) -> Option<&MaturityByteComparison> {
        self.static_root.as_ref()
    }
    /// Witnessed script versus linked announcement script, if compared.
    #[must_use]
    pub const fn leaf_script(&self) -> Option<&MaturityByteComparison> {
        self.leaf_script.as_ref()
    }
    /// Witnessed predecessor path versus reconstructed path, if compared.
    #[must_use]
    pub const fn control_block(&self) -> Option<&MaturityByteComparison> {
        self.control_block.as_ref()
    }
    /// Witnessed predecessor prefix versus its reconstructed parity prefix, if compared.
    #[must_use]
    pub const fn predecessor_prefix(&self) -> Option<&MaturityByteComparison> {
        self.predecessor_prefix.as_ref()
    }
    /// The transition and reconstruction comparisons, only once reached.
    #[must_use]
    pub const fn semantic(&self) -> Option<&MaturitySemanticComparison> {
        self.semantic.as_ref()
    }
    /// Observed output versus the program for the expected successor, if compared.
    #[must_use]
    pub const fn output_program(&self) -> Option<&MaturityByteComparison> {
        self.output_program.as_ref()
    }
    /// Witnessed successor prefix versus its reconstructed parity prefix, if compared.
    #[must_use]
    pub const fn successor_prefix(&self) -> Option<&MaturityByteComparison> {
        self.successor_prefix.as_ref()
    }
}

/// A stated acceptance operand that disagreed with the projected transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityAcceptanceClaimMismatch {
    /// The claimed schedule differs from the retained bundle's schedule.
    Schedule {
        /// The schedule named by the acceptance claim.
        claimed: StateWitnessSchedule,
        /// The schedule retained by the bundle.
        retained: StateWitnessSchedule,
    },
    /// The claimed readback bytes differ from the submitted bytes.
    ReadbackBytes {
        /// The readback bytes named by the claim.
        claimed: Vec<u8>,
        /// The bytes submitted to the projector.
        submitted: Vec<u8>,
    },
    /// A claimed transaction identity differs from the witness-stripped digest.
    Identity {
        /// The first disagreeing identity, from the obligation or readback.
        claimed: Txid,
        /// The identity recomputed from the transaction.
        recomputed: Txid,
    },
    /// The claimed witness identity differs from the digest of bytes as sent.
    WitnessIdentity {
        /// The readback's witness identity.
        claimed: Txid,
        /// The witness identity recomputed from submitted bytes.
        recomputed: Txid,
    },
}

/// The first failed projection check, preserving inner roots and the operands already compared.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MaturityContinuityRefusal {
    /// The separate nonce-prefix computation disagreed with constructor evidence.
    PrefixEvidence {
        /// The semantic side being checked.
        side: TransactionSide,
        /// The failed independently checked operand.
        comparison: MaturityEvidenceMismatch,
    },
    /// Hash or real-curve recomputation disagreed with a constructor projection.
    TweakEvidence {
        /// The semantic side being checked.
        side: TransactionSide,
        /// The failed independently checked operand.
        comparison: MaturityEvidenceMismatch,
    },
    /// A control path disagreed with its independently rebuilt recipe.
    ControlEvidence {
        /// The semantic side being checked.
        side: TransactionSide,
        /// The failed independently checked operand.
        comparison: MaturityEvidenceMismatch,
    },
    /// The transaction decoder refused the submitted serialization.
    Decode(Box<TransactionRefusal>),
    /// The decoded input count was not one.
    InputCount {
        /// The decoded count.
        actual: usize,
    },
    /// The decoded output count was not one.
    OutputCount {
        /// The decoded count.
        actual: usize,
    },
    /// The one input's witness did not contain nine items.
    WitnessItemCount {
        /// The decoded stack length.
        actual: usize,
    },
    /// A witness item did not have its declared fixed width.
    WitnessWidth {
        /// Zero-based populated witness position.
        index: usize,
        /// The ABI width.
        expected: usize,
        /// The observed width.
        actual: usize,
    },
    /// The reviewed target could not lower the retained witness schedule.
    ///
    /// The two reviewed schedules and canonical layout fit this target's bound;
    /// a caller cannot alter those premises.
    WitnessLowering(Box<StateWitnessLoweringRefusal>),
    /// The decoded input spends another outpoint than the funding evidence names.
    SpentOutpoint {
        /// The input's outpoint.
        decoded: Outpoint,
        /// The funding evidence's outpoint.
        funded: Outpoint,
    },
    /// The canonical realization codec refused witness item four.
    MetadataDecode(Box<StateMetadataRefusal>),
    /// Witness, funding or deployment disagreed with the retained predecessor context.
    RetainedContext {
        /// The semantic tuple and nonce read from the witness.
        witnessed: Box<EncodedStateMetadata>,
        /// The bundle's first retained tuple, absent only for an impossible empty bundle.
        retained: Option<Box<EncodedStateMetadata>>,
        /// The caller's spent output program.
        funded_program: Vec<u8>,
        /// The first retained constructor's program, if present.
        retained_program: Option<Vec<u8>>,
        /// Caller identity followed by retained deployment identity.
        identities: Box<(CandidateDeploymentIdentity, CandidateDeploymentIdentity)>,
    },
    /// The fixed-nonce predecessor reconstruction refused.
    PredecessorReconstruction(Box<StateConstructorRefusal>),
    /// The reconstructed predecessor did not bind to the funded output.
    PredecessorProgram {
        /// The public fixed-nonce entry's result.
        reconstructed: Vec<u8>,
        /// The supplied spent output program.
        funded: Vec<u8>,
    },
    /// The separate predecessor leastness scan refused.
    PredecessorSearch(Box<StateConstructorRefusal>),
    /// The witnessed root differs from the retained root; no transition comparison has run.
    StaticRoot(Box<MaturityProjectionDiagnostics>),
    /// The reviewed target could not be acquired.
    Target(Box<MaturityClosureRefusal>),
    /// The retained announcement program could not be obtained.
    LeafReconstruction(Box<MaturityClosureRefusal>),
    /// The witnessed leaf differs from the linked announcement bytes.
    LeafScript(Box<MaturityProjectionDiagnostics>),
    /// Reconstruction of the predecessor announcement control refused.
    ControlRecipe(Box<StateConstructorRefusal>),
    /// The witnessed predecessor control differs from its reconstructed path.
    ControlBlock(Box<MaturityProjectionDiagnostics>),
    /// The witnessed predecessor prefix differs from its reconstructed parity prefix.
    PredecessorPrefix(Box<MaturityProjectionDiagnostics>),
    /// The realization refused the decoded predecessor and witnessed request under the retained bounds.
    Transition(Box<MaturityTransitionRefusal>),
    /// The fixed-nonce successor reconstruction refused.
    SuccessorReconstruction(Box<StateConstructorRefusal>),
    /// The observed output did not commit the expected successor; diagnostics do not attribute why.
    OutputProgram(Box<MaturityProjectionDiagnostics>),
    /// The witnessed successor prefix differs from its reconstructed parity prefix.
    SuccessorPrefix(Box<MaturityProjectionDiagnostics>),
    /// The separate successor leastness scan refused.
    SuccessorSearch(Box<StateConstructorRefusal>),
    /// The stated acceptance disagreed with projected bytes; no commitment-binding comparison failed, so no projection diagnostics are retained.
    AcceptanceClaim(Box<MaturityAcceptanceClaimMismatch>),
}

impl MaturityContinuityRefusal {
    /// Comparisons reached before a commitment-binding failure, without inventing later verdicts.
    #[must_use]
    pub const fn diagnostics(&self) -> Option<&MaturityProjectionDiagnostics> {
        match self {
            Self::StaticRoot(value)
            | Self::LeafScript(value)
            | Self::ControlBlock(value)
            | Self::PredecessorPrefix(value)
            | Self::OutputProgram(value)
            | Self::SuccessorPrefix(value) => Some(value),
            _ => None,
        }
    }
}

type Refusal = MaturityContinuityRefusal;
type ProjectionResult<T> = Result<T, Refusal>;

const fn declared_widths(schedule: StateWitnessSchedule) -> [usize; 7] {
    let metadata = match schedule {
        StateWitnessSchedule::WholeMetadata => STATE_METADATA_BYTES,
        StateWitnessSchedule::VariableMetadata => STATE_METADATA_VARIABLE_BYTES,
    };
    [1, 4, 8, 32, metadata, 1, 64]
}

fn witness(
    transaction: &TargetTransaction,
    schedule: StateWitnessSchedule,
) -> ProjectionResult<&[Vec<u8>]> {
    if transaction.inputs().len() != 1 {
        return Err(Refusal::InputCount {
            actual: transaction.inputs().len(),
        });
    }
    if transaction.outputs().len() != 1 {
        return Err(Refusal::OutputCount {
            actual: transaction.outputs().len(),
        });
    }
    // Successful decoding guarantees one witness per input, including a default witness when absent.
    let stack = transaction.witnesses()[0].stack();
    if stack.len() != 9 {
        return Err(Refusal::WitnessItemCount {
            actual: stack.len(),
        });
    }
    for (index, (item, expected)) in stack.iter().zip(declared_widths(schedule)).enumerate() {
        if item.len() != expected {
            return Err(Refusal::WitnessWidth {
                index,
                expected,
                actual: item.len(),
            });
        }
    }
    Ok(stack)
}

fn carried_width(legalization: &StateWitnessLegalization) -> usize {
    match legalization {
        StateWitnessLegalization::Whole { width } => *width,
        StateWitnessLegalization::ExpandFromVariable { range, .. } => range.len(),
    }
}

fn fixed_bytes<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut result = [0; N];
    result.copy_from_slice(bytes);
    result
}

fn decoded_witness_metadata(
    stack: &[Vec<u8>],
    schedule: StateWitnessSchedule,
    target: &ReviewedElementsTapscriptDefinition,
) -> ProjectionResult<EncodedStateMetadata> {
    let bound = target
        .definition()
        .resources()
        .policy()
        .bounds()
        .get(&ResourceDimension::InitialWitnessItemBytes)
        .copied()
        .unwrap_or(ResourceBound::Maximum(0));
    let legalization = legalize_state_witness_schedule(schedule, &STATE_METADATA_LAYOUT, bound)
        .map_err(|error| Refusal::WitnessLowering(Box::new(error)))?;
    let expected = carried_width(&legalization);
    debug_assert_eq!(expected, declared_widths(schedule)[4]);
    if stack[4].len() != expected {
        return Err(Refusal::WitnessWidth {
            index: 4,
            expected,
            actual: stack[4].len(),
        });
    }
    let metadata_bytes = match legalization {
        StateWitnessLegalization::Whole { .. } => stack[4].clone(),
        StateWitnessLegalization::ExpandFromVariable { .. } => {
            rebuild_state_metadata(&fixed_bytes::<STATE_METADATA_VARIABLE_BYTES>(&stack[4]))
                .to_vec()
        }
    };
    decode_state_metadata(&metadata_bytes).map_err(|error| Refusal::MetadataDecode(Box::new(error)))
}

fn retained_context(
    input: &MaturityProjectionInput<'_>,
    encoded: EncodedStateMetadata,
) -> ProjectionResult<()> {
    let retained = input.bundle.instances().first();
    let retained_metadata = retained.map(|instance| *instance.metadata());
    let retained_program = retained.map(|instance| instance.constructor().output_program());
    if retained_metadata != Some(encoded)
        || retained_program.as_deref() != Some(input.funded.program.as_slice())
        || input.identity != input.bundle.deployment().identity()
    {
        return Err(Refusal::RetainedContext {
            witnessed: Box::new(encoded),
            retained: retained_metadata.map(Box::new),
            funded_program: input.funded.program.clone(),
            retained_program,
            identities: Box::new((
                input.identity.clone(),
                input.bundle.deployment().identity().clone(),
            )),
        });
    }
    Ok(())
}

fn commitments(bytes: &[u8], side: TransactionSide) -> Vec<StateFieldCommitment> {
    [
        (StateField::Omega, 25..33),
        (StateField::YL, 33..41),
        (StateField::YT, 41..49),
        (StateField::Q, 49..57),
        (StateField::Cycle, 57..65),
        (StateField::Maturity, 65..74),
    ]
    .into_iter()
    .map(|(field, range)| StateFieldCommitment {
        side,
        field,
        bytes: bytes[range.clone()].to_vec(),
        range,
    })
    .collect()
}

fn field_comparisons(
    left: &[StateFieldCommitment],
    right: &[StateFieldCommitment],
) -> Vec<MaturityFieldComparison> {
    left.iter()
        .zip(right)
        .map(|(left, right)| MaturityFieldComparison {
            left: left.clone(),
            right: right.clone(),
        })
        .collect()
}

fn transition_comparison(
    predecessor: &MaturityConstructorProjection,
    expected: StateMetadata,
    requested_cycle: Cycle,
) -> MaturitySemanticComparison {
    let expected_fields = commitments(
        &encode_state_metadata(&expected, StateRepresentationNonce::ZERO),
        TransactionSide::Output,
    );
    MaturitySemanticComparison {
        predecessor_against_expected: field_comparisons(
            predecessor.field_commitments(),
            &expected_fields,
        ),
        reconstructed_against_expected: None,
        expected_successor: expected,
        requested_cycle,
    }
}

fn fixed_projection(
    target: &ReviewedElementsTapscriptDefinition,
    encoded: EncodedStateMetadata,
    bundle: &CandidateLinkedMaturityBundle,
    side: TransactionSide,
) -> Result<MaturityConstructorProjection, StateConstructorRefusal> {
    let tree = bundle.static_subtree();
    let policy = bundle.policy().internal_key();
    let output_program =
        state_output_program_at_nonce(target, &encoded, tree, policy, &OracleStateCurve)?;
    let metadata_bytes = encode_state_metadata(&encoded.semantic, encoded.representation);
    let metadata_program = state_metadata_leaf_program(target, &encoded)?;
    let version = target.definition().leaf_version();
    let metadata_hash = leaf_hash(version, &metadata_program.encode(target));
    let merkle_root = branch_hash(metadata_hash, *tree.root());
    let (output_key, parity) = match OracleStateCurve.output_key(policy.key(), &merkle_root) {
        StateTweakOutcome::OutputKey { key, parity } => (key, parity),
        StateTweakOutcome::InternalKeyNotAPoint => {
            return Err(StateConstructorRefusal::InternalKeyNotAPoint);
        }
        StateTweakOutcome::TweakAboveGroupOrder => {
            return Err(StateConstructorRefusal::TweakAboveGroupOrder);
        }
        StateTweakOutcome::TweakedPointIsIdentity => {
            return Err(StateConstructorRefusal::TweakedPointIsIdentity);
        }
    };
    let mut announcements = tree
        .leaves()
        .iter()
        .filter(|entry| entry.leaf.role == StateLeafRole::Announcement);
    let entry = announcements
        .next()
        .ok_or(StateConstructorRefusal::ExecutingLeafAbsent)?;
    if announcements.next().is_some() {
        return Err(StateConstructorRefusal::ExecutingLeafRepeated);
    }
    let mut siblings = entry.siblings.clone();
    siblings.push(metadata_hash);
    let control_recipe = StateControlRecipe {
        role: StateLeafRole::Announcement,
        leaf_version: version,
        parity,
        internal_key: *policy.key(),
        executing_leaf_hash: entry.hash,
        siblings,
    };
    let mut tweak_preimage = policy.key().to_vec();
    tweak_preimage.extend(merkle_root);
    Ok(MaturityConstructorProjection {
        encoded_metadata: encoded,
        field_commitments: commitments(&metadata_bytes, side),
        metadata_bytes,
        static_subtree: tree.clone(),
        merkle_root,
        tweak_hash: tagged_hash("TapTweak/elements", &tweak_preimage),
        output_key,
        parity,
        output_program,
        control_recipe,
    })
}

fn reconstruct(
    target: &ReviewedElementsTapscriptDefinition,
    encoded: EncodedStateMetadata,
    bundle: &CandidateLinkedMaturityBundle,
    side: TransactionSide,
) -> ProjectionResult<MaturityConstructorProjection> {
    fixed_projection(target, encoded, bundle, side).map_err(|error| match side {
        TransactionSide::Input => Refusal::PredecessorReconstruction(Box::new(error)),
        TransactionSide::Output => Refusal::SuccessorReconstruction(Box::new(error)),
    })
}

fn leastness(
    target: &ReviewedElementsTapscriptDefinition,
    projection: &MaturityConstructorProjection,
    bundle: &CandidateLinkedMaturityBundle,
    side: TransactionSide,
    budget: tapscript::StateNonceBudget,
) -> ProjectionResult<MaturityNonceLeastness> {
    search_leastness(target, projection, bundle, side, budget).map_err(|error| match side {
        TransactionSide::Input => Refusal::PredecessorSearch(Box::new(error)),
        TransactionSide::Output => Refusal::SuccessorSearch(Box::new(error)),
    })
}

fn search_leastness(
    target: &ReviewedElementsTapscriptDefinition,
    projection: &MaturityConstructorProjection,
    bundle: &CandidateLinkedMaturityBundle,
    side: TransactionSide,
    budget: tapscript::StateNonceBudget,
) -> Result<MaturityNonceLeastness, StateConstructorRefusal> {
    let host = CandidateStateConstructor::derive(
        target,
        &projection.encoded_metadata.semantic,
        bundle.static_subtree(),
        bundle.policy().internal_key(),
        budget,
        &OracleStateCurve,
    )?;
    let reconstructed_facts_match = if host.nonce() == projection.nonce() {
        Some(facts_match(projection, &host, side)?)
    } else {
        None
    };
    Ok(MaturityNonceLeastness {
        witnessed_nonce: projection.nonce(),
        host_least_constructor: host,
        reconstructed_facts_match,
    })
}

fn facts_match(
    projection: &MaturityConstructorProjection,
    host: &CandidateStateConstructor,
    side: TransactionSide,
) -> Result<bool, StateConstructorRefusal> {
    let control = host.control_recipe(StateLeafRole::Announcement)?;
    Ok(projection.encoded_metadata() == host.encoded_metadata()
        && projection.metadata_bytes() == host.metadata_bytes()
        && projection.static_subtree() == host.static_subtree()
        && projection.merkle_root() == host.merkle_root()
        && projection.tweak_hash() == &host.tweak_hash()
        && projection.output_key() == host.output_key()
        && projection.parity() == host.parity()
        && projection.output_program() == host.output_program()
        && projection.control_recipe() == &control
        && projection.control_recipe().control_bytes()? == control.control_bytes()?
        && projection.field_commitments() == host.field_commitments(side))
}

fn check_candidate(
    target: &ReviewedElementsTapscriptDefinition,
    semantic: StateMetadata,
    candidate: StateRepresentationNonce,
    bundle: &CandidateLinkedMaturityBundle,
) -> Result<MaturityNonceCheck, StateConstructorRefusal> {
    let encoded = EncodedStateMetadata {
        semantic,
        representation: candidate,
    };
    let program = state_metadata_leaf_program(target, &encoded)?.encode(target);
    let leaf_digest = leaf_hash(target.definition().leaf_version(), &program);
    let retained_root = *bundle.static_subtree().root();
    let curve_outcome = (leaf_digest <= retained_root).then(|| {
        // Ordering has been checked, so framing this pair directly is independent of the branch helper.
        let root = tagged_hash("TapBranch/elements", &[leaf_digest, retained_root].concat());
        OracleStateCurve.output_key(bundle.policy().internal_key().key(), &root)
    });
    Ok(MaturityNonceCheck {
        candidate,
        encoding: encode_state_metadata(&semantic, candidate),
        leaf_digest,
        retained_root,
        curve_outcome,
    })
}

const fn candidate_refusal(check: &MaturityNonceCheck) -> Option<StateConstructorRefusal> {
    match check.curve_outcome {
        None => Some(StateConstructorRefusal::CanonicalBranchSideNotSatisfied),
        Some(StateTweakOutcome::OutputKey { .. }) => None,
        Some(StateTweakOutcome::InternalKeyNotAPoint) => {
            Some(StateConstructorRefusal::InternalKeyNotAPoint)
        }
        Some(StateTweakOutcome::TweakAboveGroupOrder) => {
            Some(StateConstructorRefusal::TweakAboveGroupOrder)
        }
        Some(StateTweakOutcome::TweakedPointIsIdentity) => {
            Some(StateConstructorRefusal::TweakedPointIsIdentity)
        }
    }
}

fn checked_prefix(
    target: &ReviewedElementsTapscriptDefinition,
    projection: &MaturityConstructorProjection,
    least: &MaturityNonceLeastness,
    bundle: &CandidateLinkedMaturityBundle,
    side: TransactionSide,
) -> ProjectionResult<MaturityCheckedPrefix> {
    let refused = |comparison| Refusal::PrefixEvidence { side, comparison };
    let evidence = least.evidence();
    if usize::try_from(evidence.selected.get()).ok() != Some(evidence.rejected.len())
        || evidence.selected != least.host_least_constructor().nonce()
        || least.witnessed_nonce() != projection.nonce()
    {
        return Err(refused(MaturityEvidenceMismatch::NonceOrder));
    }
    let semantic = projection.encoded_metadata().semantic;
    if least.host_least_constructor().encoded_metadata().semantic != semantic {
        return Err(refused(MaturityEvidenceMismatch::Metadata));
    }
    let evaluate = |nonce| {
        check_candidate(target, semantic, nonce, bundle)
            .map_err(|_| refused(MaturityEvidenceMismatch::Metadata))
    };
    let mut rejected = Vec::with_capacity(evidence.rejected.len());
    for (index, (nonce, cause)) in evidence.rejected.iter().enumerate() {
        if usize::try_from(nonce.get()).ok() != Some(index) {
            return Err(refused(MaturityEvidenceMismatch::NonceOrder));
        }
        let check = evaluate(*nonce)?;
        if candidate_refusal(&check) != Some(*cause) {
            return Err(refused(MaturityEvidenceMismatch::RejectionCause));
        }
        rejected.push((check, *cause));
    }
    let selected = evaluate(evidence.selected)?;
    let witnessed = evaluate(projection.nonce())?;
    if candidate_refusal(&selected).is_some() || candidate_refusal(&witnessed).is_some() {
        return Err(refused(MaturityEvidenceMismatch::Admissibility));
    }
    if selected.encoding() != least.host_least_constructor().metadata_bytes()
        || witnessed.encoding() != projection.metadata_bytes()
    {
        return Err(refused(MaturityEvidenceMismatch::Metadata));
    }
    Ok(MaturityCheckedPrefix {
        side,
        rejected,
        selected,
        witnessed,
        host_least: projection.nonce() == evidence.selected,
        residual: MaturityLeastnessResidual::HostSearchOnly,
    })
}

/// Named assumptions under which commitment comparisons have their stated meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityCommitmentAssumption {
    /// Collision resistance of the metadata/static leaf domain.
    LeafCollisionResistance,
    /// Second-preimage resistance of the metadata/static leaf domain.
    LeafSecondPreimageResistance,
    /// Collision resistance of the ordered branch domain.
    BranchCollisionResistance,
    /// Second-preimage resistance of the ordered branch domain.
    BranchSecondPreimageResistance,
    /// Collision resistance of the tweak domain.
    TweakCollisionResistance,
    /// Second-preimage resistance of the tweak domain.
    TweakSecondPreimageResistance,
    /// Discrete-log hardness and SHA-256 preimage resistance of the NUMS construction.
    NumsDiscreteLogAndPreimageResistance,
}

impl MaturityCommitmentAssumption {
    /// The exact hash domain, or the internal-key policy's unchanged residual.
    #[must_use]
    pub const fn domain_or_residual(self) -> &'static str {
        match self {
            Self::LeafCollisionResistance | Self::LeafSecondPreimageResistance => {
                "TapLeaf/elements"
            }
            Self::BranchCollisionResistance | Self::BranchSecondPreimageResistance => {
                "TapBranch/elements"
            }
            Self::TweakCollisionResistance | Self::TweakSecondPreimageResistance => {
                "TapTweak/elements"
            }
            Self::NumsDiscreteLogAndPreimageResistance => {
                tapscript::StateInternalKeyPolicy::RESIDUAL
            }
        }
    }
}

/// Projection operands compared with a second hash and real-curve computation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityTweakEvidence {
    side: TransactionSide,
    internal: MaturityByteComparison,
    metadata: MaturityByteComparison,
    branch: MaturityByteComparison,
    digest: MaturityByteComparison,
    key: MaturityByteComparison,
    oddness: MaturityByteComparison,
    program: MaturityByteComparison,
    premises: [MaturityCommitmentAssumption; 7],
}

impl MaturityTweakEvidence {
    /// The constructor side; comparisons retain the projected operand followed by recomputation.
    #[must_use]
    pub const fn side(&self) -> TransactionSide {
        self.side
    }
    /// Projected internal key versus the generator-derived NUMS key.
    #[must_use]
    pub const fn internal(&self) -> &MaturityByteComparison {
        &self.internal
    }
    /// Recipe metadata sibling versus the rebuilt leaf digest.
    #[must_use]
    pub const fn metadata(&self) -> &MaturityByteComparison {
        &self.metadata
    }
    /// Projected outer root versus its tagged recomputation.
    #[must_use]
    pub const fn branch(&self) -> &MaturityByteComparison {
        &self.branch
    }
    /// Projected tweak digest versus its tagged recomputation.
    #[must_use]
    pub const fn digest(&self) -> &MaturityByteComparison {
        &self.digest
    }
    /// Projected output x coordinate versus the real curve result.
    #[must_use]
    pub const fn key(&self) -> &MaturityByteComparison {
        &self.key
    }
    /// Projected and recomputed parity as zero or one.
    #[must_use]
    pub const fn oddness(&self) -> &MaturityByteComparison {
        &self.oddness
    }
    /// Projected program versus the reconstructed output encoding.
    #[must_use]
    pub const fn program(&self) -> &MaturityByteComparison {
        &self.program
    }
    /// The named assumptions; none asserts that a key path is impossible.
    #[must_use]
    pub const fn premises(&self) -> &[MaturityCommitmentAssumption; 7] {
        &self.premises
    }
}

fn tweak_evidence(
    target: &ReviewedElementsTapscriptDefinition,
    projection: &MaturityConstructorProjection,
    bundle: &CandidateLinkedMaturityBundle,
    side: TransactionSide,
) -> ProjectionResult<MaturityTweakEvidence> {
    use MaturityCommitmentAssumption as A;
    use MaturityEvidenceMismatch as M;
    let refused = |comparison| Refusal::TweakEvidence { side, comparison };
    let check = check_candidate(
        target,
        projection.encoded_metadata().semantic,
        projection.nonce(),
        bundle,
    )
    .map_err(|_| refused(M::Metadata))?;
    let Some(StateTweakOutcome::OutputKey { key, parity }) = check.curve_outcome else {
        return Err(refused(M::Admissibility));
    };
    let nums = sha256(
        &[
            &[0x04][..],
            &tapscript::STATE_GENERATOR_X,
            &tapscript::STATE_GENERATOR_Y,
        ]
        .concat(),
    );
    let root = tagged_hash(
        "TapBranch/elements",
        &[check.leaf_digest, check.retained_root].concat(),
    );
    let digest = tagged_hash("TapTweak/elements", &[nums, root].concat());
    let outer = projection
        .control_recipe()
        .siblings
        .last()
        .ok_or_else(|| refused(M::Metadata))?;
    let result = MaturityTweakEvidence {
        side,
        internal: MaturityByteComparison::new(&projection.control_recipe().internal_key, &nums),
        metadata: MaturityByteComparison::new(outer, &check.leaf_digest),
        branch: MaturityByteComparison::new(projection.merkle_root(), &root),
        digest: MaturityByteComparison::new(projection.tweak_hash(), &digest),
        key: MaturityByteComparison::new(projection.output_key(), &key),
        oddness: MaturityByteComparison::new(&[u8::from(projection.parity())], &[u8::from(parity)]),
        program: MaturityByteComparison::new(
            projection.output_program(),
            &[&[0x51, 0x20][..], &key].concat(),
        ),
        premises: [
            A::LeafCollisionResistance,
            A::LeafSecondPreimageResistance,
            A::BranchCollisionResistance,
            A::BranchSecondPreimageResistance,
            A::TweakCollisionResistance,
            A::TweakSecondPreimageResistance,
            A::NumsDiscreteLogAndPreimageResistance,
        ],
    };
    for (comparison, field) in [
        (&result.internal, M::InternalKey),
        (&result.metadata, M::Metadata),
        (&result.branch, M::BranchRoot),
        (&result.digest, M::Digest),
        (&result.key, M::OutputPoint),
        (&result.oddness, M::Parity),
        (&result.program, M::OutputPoint),
    ] {
        if !comparison.agrees() {
            return Err(refused(field));
        }
    }
    if projection.metadata_bytes() != check.encoding()
        || bundle.policy().internal_key().key() != &nums
    {
        return Err(refused(M::Metadata));
    }
    Ok(result)
}

const fn prefix(odd: bool) -> u8 {
    if odd { 0x03 } else { 0x02 }
}

/// Whether a control path was read from a spend or only reconstructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityControlObservation {
    /// The submitted predecessor spend contains these bytes.
    ObservedPredecessorSpend,
    /// No successor spend is present in the submitted transaction.
    DerivedSuccessorUnobserved,
}

/// The observed predecessor, derived successor, and measured relation of their recipes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityControlPaths {
    before: StateControlRecipe,
    after: StateControlRecipe,
    observed: MaturityByteComparison,
    derived: Vec<u8>,
    inner: MaturityByteComparison,
    outer: [MaturityByteComparison; 2],
    first_xor: u8,
    parity_xor: u8,
}

impl MaturityControlPaths {
    /// The independently reconstructed predecessor recipe.
    #[must_use]
    pub const fn before(&self) -> &StateControlRecipe {
        &self.before
    }
    /// The independently reconstructed successor recipe, with no spend observation.
    #[must_use]
    pub const fn after(&self) -> &StateControlRecipe {
        &self.after
    }
    /// Submitted predecessor control bytes against independent reconstruction.
    #[must_use]
    pub const fn observed(&self) -> &MaturityByteComparison {
        &self.observed
    }
    /// Serialized successor recipe; these bytes were not observed in a successor spend.
    #[must_use]
    pub fn derived(&self) -> &[u8] {
        &self.derived
    }
    /// Explicitly different evidence kinds for the two sides.
    #[must_use]
    pub const fn observations(&self) -> [MaturityControlObservation; 2] {
        [
            MaturityControlObservation::ObservedPredecessorSpend,
            MaturityControlObservation::DerivedSuccessorUnobserved,
        ]
    }
    /// Measured predecessor and successor recipe lengths.
    #[must_use]
    pub fn lengths(&self) -> (usize, usize) {
        (self.observed.reconstructed().len(), self.derived.len())
    }
    /// Internal static siblings on the predecessor and successor paths.
    #[must_use]
    pub const fn inner(&self) -> &MaturityByteComparison {
        &self.inner
    }
    /// Each path's outer sibling against that side's recomputed metadata commitment.
    #[must_use]
    pub const fn outer(&self) -> &[MaturityByteComparison; 2] {
        &self.outer
    }
    /// Actual first-byte XOR followed by the XOR predicted solely by output parity.
    #[must_use]
    pub const fn first_byte_relation(&self) -> (u8, u8) {
        (self.first_xor, self.parity_xor)
    }
}

fn independent_control(
    target: &ReviewedElementsTapscriptDefinition,
    projection: &MaturityConstructorProjection,
    bundle: &CandidateLinkedMaturityBundle,
    side: TransactionSide,
) -> ProjectionResult<(StateControlRecipe, [u8; 32])> {
    let refused = |comparison| Refusal::ControlEvidence { side, comparison };
    let check = check_candidate(
        target,
        projection.encoded_metadata().semantic,
        projection.nonce(),
        bundle,
    )
    .map_err(|_| refused(MaturityEvidenceMismatch::Metadata))?;
    let Some(StateTweakOutcome::OutputKey { parity, .. }) = check.curve_outcome else {
        return Err(refused(MaturityEvidenceMismatch::Admissibility));
    };
    let mut leaves = bundle
        .static_subtree()
        .leaves()
        .iter()
        .filter(|entry| entry.leaf.role == StateLeafRole::Announcement);
    let entry = leaves
        .next()
        .ok_or_else(|| refused(MaturityEvidenceMismatch::Recipe))?;
    if leaves.next().is_some() {
        return Err(refused(MaturityEvidenceMismatch::Recipe));
    }
    let mut siblings = entry.siblings.clone();
    siblings.push(check.leaf_digest);
    let recipe = StateControlRecipe {
        role: StateLeafRole::Announcement,
        leaf_version: target.definition().leaf_version(),
        parity,
        internal_key: *bundle.policy().internal_key().key(),
        executing_leaf_hash: entry.hash,
        siblings,
    };
    if &recipe != projection.control_recipe() {
        return Err(refused(MaturityEvidenceMismatch::Recipe));
    }
    Ok((recipe, check.leaf_digest))
}

fn control_paths(
    target: &ReviewedElementsTapscriptDefinition,
    before: &MaturityConstructorProjection,
    after: &MaturityConstructorProjection,
    bundle: &CandidateLinkedMaturityBundle,
    witnessed: &[u8],
) -> ProjectionResult<MaturityControlPaths> {
    let (before, old_hash) = independent_control(target, before, bundle, TransactionSide::Input)?;
    let (after, new_hash) = independent_control(target, after, bundle, TransactionSide::Output)?;
    let bytes = |recipe: &StateControlRecipe, side| {
        recipe
            .control_bytes()
            .map_err(|_| Refusal::ControlEvidence {
                side,
                comparison: MaturityEvidenceMismatch::Recipe,
            })
    };
    let old = bytes(&before, TransactionSide::Input)?;
    let derived = bytes(&after, TransactionSide::Output)?;
    let observed = MaturityByteComparison::new(witnessed, &old);
    if !observed.agrees() {
        return Err(Refusal::ControlEvidence {
            side: TransactionSide::Input,
            comparison: MaturityEvidenceMismatch::Recipe,
        });
    }
    // Independently assembled recipes always have a header and the appended metadata sibling.
    let inner =
        MaturityByteComparison::new(&old[33..old.len() - 32], &derived[33..derived.len() - 32]);
    let outer = [
        MaturityByteComparison::new(&old[old.len() - 32..], &old_hash),
        MaturityByteComparison::new(&derived[derived.len() - 32..], &new_hash),
    ];
    let first_xor = old[0] ^ derived[0];
    let parity_xor = u8::from(before.parity) ^ u8::from(after.parity);
    Ok(MaturityControlPaths {
        before,
        after,
        observed,
        derived,
        inner,
        outer,
        first_xor,
        parity_xor,
    })
}

/// Signature provenance of an experimental carrier, without claiming verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturitySignatureDisposition {
    /// Copied from submitted bytes, with no verification by this projector.
    SubmittedUnverified,
    /// A designated 64-byte placeholder; establishes neither authorization nor target acceptance.
    PlaceholderNoAuthorization,
}

/// The actual experimental substitution, separate from the matrix's intended carrier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityMutationCarrier {
    description: String,
    witness_items: Vec<usize>,
    output_position: Option<usize>,
    retained_context: String,
    signature: MaturitySignatureDisposition,
}

impl MaturityMutationCarrier {
    /// Describe changed bytes and retained/deployment context; this is stated provenance, not a verdict.
    #[must_use]
    pub const fn new(
        description: String,
        witness_items: Vec<usize>,
        output_position: Option<usize>,
        retained_context: String,
        signature: MaturitySignatureDisposition,
    ) -> Self {
        Self {
            description,
            witness_items,
            output_position,
            retained_context,
            signature,
        }
    }
    /// What was substituted and how it was constructed.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
    /// Zero-based witness items changed or coherently rebuilt.
    #[must_use]
    pub fn witness_items(&self) -> &[usize] {
        &self.witness_items
    }
    /// The changed output position, if any.
    #[must_use]
    pub const fn output_position(&self) -> Option<usize> {
        self.output_position
    }
    /// The supplied deployment/context pairing, including any independently retained comparison tree.
    #[must_use]
    pub fn retained_context(&self) -> &str {
        &self.retained_context
    }
    /// Whether the signature is a placeholder; neither disposition asserts authorization.
    #[must_use]
    pub const fn signature(&self) -> MaturitySignatureDisposition {
        self.signature
    }
}

/// A matrix identity resolved from the one registry beside the actual experimental carrier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityMutationContext {
    row: &'static MaturitySafetyRow,
    applied_layer: MutationLayer,
    actual: MaturityMutationCarrier,
}

/// Refusal to represent a missing row or a successful projection as a negative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityMutantAssemblyRefusal {
    /// No row has this section and exact name.
    UnknownRow,
    /// The projector accepted the bytes; there is no negative result to record.
    ProjectionSucceeded,
}

impl MaturityMutationContext {
    /// Look up the original row without modifying its declared boundary or standing.
    ///
    /// # Errors
    /// Returns `UnknownRow` when section and name do not identify a matrix row.
    pub fn new(
        section: MaturitySafetySection,
        name: &str,
        applied_layer: MutationLayer,
        actual: MaturityMutationCarrier,
    ) -> Result<Self, MaturityMutantAssemblyRefusal> {
        let row = rows()
            .iter()
            .find(|row| (row.section(), row.name()) == (section, name))
            .ok_or(MaturityMutantAssemblyRefusal::UnknownRow)?;
        Ok(Self {
            row,
            applied_layer,
            actual,
        })
    }
    /// The original row, preserving its section and exact name.
    #[must_use]
    pub const fn row(&self) -> &'static MaturitySafetyRow {
        self.row
    }
    /// The matrix's declared boundary, never inferred from a host refusal.
    #[must_use]
    pub const fn declared_boundary(&self) -> MaturityRowBoundary {
        self.row.boundary()
    }
    /// The matrix's declared refusing layer, if it names one.
    #[must_use]
    pub const fn refusing_layer(&self) -> Option<EvidenceBoundary> {
        self.row.refusing_layer()
    }
    /// The matrix's declared mutation layer.
    #[must_use]
    pub const fn declared_layer(&self) -> Option<MutationLayer> {
        self.row.mutation()
    }
    /// The layer actually changed by this experiment.
    #[must_use]
    pub const fn applied_layer(&self) -> MutationLayer {
        self.applied_layer
    }
    /// The original intended carrier, distinct from what this experiment supplied.
    #[must_use]
    pub const fn intended_carrier(&self) -> MaturityIntendedCarrier {
        self.row.carrier()
    }
    /// The actual bytes/context and signature disposition.
    #[must_use]
    pub const fn actual(&self) -> &MaturityMutationCarrier {
        &self.actual
    }
}

/// A projector refusal over exact experimental bytes; never a matrix standing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityContinuityMutant {
    context: MaturityMutationContext,
    source: MaturityByteSource,
    branch: BranchContext,
    submitted: Vec<u8>,
    digest: [u8; 32],
    failure: MaturityContinuityRefusal,
}

impl MaturityContinuityMutant {
    /// Run the projector and retain only a genuinely refused experimental subject.
    ///
    /// # Errors
    /// Returns `ProjectionSucceeded` if the supplied bytes project successfully.
    pub fn observe(
        context: MaturityMutationContext,
        input: MaturityProjectionInput<'_>,
    ) -> Result<ExperimentalSubject<Self>, MaturityMutantAssemblyRefusal> {
        let source = input.source.clone();
        let branch = input.branch;
        let submitted = input.submitted_bytes.to_vec();
        let failure = project_maturity_continuity(input)
            .err()
            .ok_or(MaturityMutantAssemblyRefusal::ProjectionSucceeded)?;
        Ok(ExperimentalSubject::observe(Self {
            context,
            source,
            branch,
            digest: sha256(&submitted),
            submitted,
            failure,
        }))
    }
    /// The resolved matrix identity and actual carrier, with no row standing.
    #[must_use]
    pub const fn context(&self) -> &MaturityMutationContext {
        &self.context
    }
    /// The source class supplied to the refused projection.
    #[must_use]
    pub const fn source(&self) -> &MaturityByteSource {
        &self.source
    }
    /// Caller-stated branch context supplied to the refused projection.
    #[must_use]
    pub const fn branch(&self) -> BranchContext {
        self.branch
    }
    /// Exact bytes supplied to the projector.
    #[must_use]
    pub fn submitted(&self) -> &[u8] {
        &self.submitted
    }
    /// SHA-256 of the exact supplied serialization.
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    /// The observed first refusal, preserving all diagnostics actually reached.
    #[must_use]
    pub const fn failure(&self) -> &MaturityContinuityRefusal {
        &self.failure
    }
}

fn predecessor_bindings(
    target: &ReviewedElementsTapscriptDefinition,
    stack: &[Vec<u8>],
    predecessor: &MaturityConstructorProjection,
    bundle: &CandidateLinkedMaturityBundle,
    diagnostics: &mut MaturityProjectionDiagnostics,
) -> ProjectionResult<(MaturityByteComparison, MaturityByteComparison)> {
    let root = MaturityByteComparison::new(&stack[3], bundle.static_subtree().root());
    diagnostics.static_root = Some(root.clone());
    if !root.agrees() {
        return Err(Refusal::StaticRoot(Box::new(diagnostics.clone())));
    }
    let linked = linked_announcement_bytes(bundle, target)
        .map_err(|error| Refusal::LeafReconstruction(Box::new(error)))?;
    let leaf = MaturityByteComparison::new(&stack[7], &linked);
    diagnostics.leaf_script = Some(leaf.clone());
    if !leaf.agrees() {
        return Err(Refusal::LeafScript(Box::new(diagnostics.clone())));
    }
    let control_bytes = predecessor
        .control_recipe()
        .control_bytes()
        .map_err(|error| Refusal::ControlRecipe(Box::new(error)))?;
    let control = MaturityByteComparison::new(&stack[8], &control_bytes);
    diagnostics.control_block = Some(control.clone());
    if !control.agrees() {
        return Err(Refusal::ControlBlock(Box::new(diagnostics.clone())));
    }
    let parity = MaturityByteComparison::new(&stack[5], &[prefix(predecessor.parity())]);
    diagnostics.predecessor_prefix = Some(parity.clone());
    if !parity.agrees() {
        return Err(Refusal::PredecessorPrefix(Box::new(diagnostics.clone())));
    }
    Ok((root, control))
}

fn predecessor_program(reconstructed: &[u8], funded: &[u8]) -> ProjectionResult<()> {
    if reconstructed != funded {
        return Err(Refusal::PredecessorProgram {
            reconstructed: reconstructed.to_vec(),
            funded: funded.to_vec(),
        });
    }
    Ok(())
}

fn successor_bindings(
    transaction: &TargetTransaction,
    stack: &[Vec<u8>],
    successor: &MaturityConstructorProjection,
    diagnostics: &mut MaturityProjectionDiagnostics,
) -> ProjectionResult<()> {
    let program = MaturityByteComparison::new(
        transaction.outputs()[0].program(),
        successor.output_program(),
    );
    diagnostics.output_program = Some(program.clone());
    if !program.agrees() {
        return Err(Refusal::OutputProgram(Box::new(diagnostics.clone())));
    }
    let parity = MaturityByteComparison::new(&stack[0], &[prefix(successor.parity())]);
    diagnostics.successor_prefix = Some(parity.clone());
    if !parity.agrees() {
        return Err(Refusal::SuccessorPrefix(Box::new(diagnostics.clone())));
    }
    Ok(())
}

fn static_comparison(
    projections: [&MaturityConstructorProjection; 2],
    leastness: [&MaturityNonceLeastness; 2],
    static_root: MaturityByteComparison,
    control_block: MaturityByteComparison,
) -> MaturityStaticComparison {
    let [before, after] = projections;
    let [old_least, new_least] = leastness;
    MaturityStaticComparison {
        static_root,
        control_block,
        descriptor_result: state_bundle_continuity(before.static_subtree(), after.static_subtree())
            .map_err(Box::new),
        constructor_result: old_least
            .host_least_constructor()
            .continuity(new_least.host_least_constructor())
            .map_err(Box::new),
    }
}

/// Identities of a submitted transaction under witness-stripped and exact-byte hashing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturitySubmittedIdentities {
    identity: Txid,
    witness_identity: Txid,
}

impl MaturitySubmittedIdentities {
    /// The double SHA-256 identity of the witness-stripped transaction.
    #[must_use]
    pub const fn identity(&self) -> Txid {
        self.identity
    }

    /// The double SHA-256 identity of the submitted bytes as sent.
    #[must_use]
    pub const fn witness_identity(&self) -> Txid {
        self.witness_identity
    }
}

/// Compute both identities used to check a stated accepted readback.
///
/// The witness identity hashes the submitted bytes as sent because the target hashes what it receives, rather than a re-encoding. `the_reference_txid_is_the_first_party_witness_stripped_digest` is the reference oracle test for this construction.
#[must_use]
pub fn submitted_transaction_identities(
    transaction: &TargetTransaction,
    submitted_bytes: &[u8],
) -> MaturitySubmittedIdentities {
    MaturitySubmittedIdentities {
        identity: Txid::from_internal(sha256(&sha256(&transaction.encode_without_witness()))),
        witness_identity: Txid::from_internal(sha256(&sha256(submitted_bytes))),
    }
}

fn verify_acceptance_claim(
    claim: &MaturityAcceptanceObligation,
    schedule: StateWitnessSchedule,
    transaction: &TargetTransaction,
    submitted_bytes: &[u8],
) -> ProjectionResult<()> {
    if let MaturityAcceptanceObligation::Established {
        schedule: claimed_schedule,
        identity: claimed_identity,
        readback,
    } = claim
    {
        if *claimed_schedule != schedule {
            return Err(Refusal::AcceptanceClaim(Box::new(
                MaturityAcceptanceClaimMismatch::Schedule {
                    claimed: *claimed_schedule,
                    retained: schedule,
                },
            )));
        }
        if readback.bytes() != submitted_bytes {
            return Err(Refusal::AcceptanceClaim(Box::new(
                MaturityAcceptanceClaimMismatch::ReadbackBytes {
                    claimed: readback.bytes().to_vec(),
                    submitted: submitted_bytes.to_vec(),
                },
            )));
        }
        let identities = submitted_transaction_identities(transaction, submitted_bytes);
        for claimed in [*claimed_identity, readback.identity()] {
            if claimed != identities.identity() {
                return Err(Refusal::AcceptanceClaim(Box::new(
                    MaturityAcceptanceClaimMismatch::Identity {
                        claimed,
                        recomputed: identities.identity(),
                    },
                )));
            }
        }
        if readback.witness_identity() != identities.witness_identity() {
            return Err(Refusal::AcceptanceClaim(Box::new(
                MaturityAcceptanceClaimMismatch::WitnessIdentity {
                    claimed: readback.witness_identity(),
                    recomputed: identities.witness_identity(),
                },
            )));
        }
    }
    Ok(())
}

/// Project both constructor sides from exact submitted bytes and the supplied spent output under one retained recipe.
///
/// Checks are ordered: decode and witness shape; spent outpoint and canonical metadata; retained-context coherence; predecessor reconstruction and leastness; root, leaf, control and predecessor prefix; realization transition; successor reconstruction, output and prefix; successor leastness; separate comparisons. Funding asset and amount remain supplied facts because the input contains neither. The source label and branch context are stated provenance, not authenticated provenance.
///
/// # Errors
/// Returns the first failed check as [`MaturityContinuityRefusal`]. Binding failures retain only comparisons already reached; in particular a static-root refusal issues no semantic verdict, and an output refusal names the expected successor without attributing the mismatch to a semantic or static change.
#[must_use = "projection can refuse the submitted bytes"]
pub fn project_maturity_continuity(
    input: MaturityProjectionInput<'_>,
) -> Result<ValidatedMaturityContinuity, MaturityContinuityRefusal> {
    project_maturity_continuity_with_acceptance(
        input,
        &MaturityAcceptanceObligation::Outstanding {
            routes: [
                MaturityAcceptanceRoute::RelayWitnessRestructure,
                MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
            ],
        },
    )
}

/// Project exact submitted bytes and verify a stated acceptance against the retained schedule and those bytes.
///
/// An outstanding obligation passes through because a projection without an accepted transcript has no acceptance to verify. An established obligation is checked after all constructor evidence is assembled, preserving the first-refusal order.
///
/// # Errors
/// Returns the first byte-binding refusal, or [`MaturityContinuityRefusal::AcceptanceClaim`] when an established claim disagrees with the projected transaction.
#[must_use = "acceptance verification can refuse the stated claim"]
pub fn project_maturity_continuity_with_acceptance(
    input: MaturityProjectionInput<'_>,
    claim: &MaturityAcceptanceObligation,
) -> Result<ValidatedMaturityContinuity, MaturityContinuityRefusal> {
    let transaction = TargetTransaction::decode(input.submitted_bytes)
        .map_err(|error| Refusal::Decode(Box::new(error)))?;
    let schedule = input.bundle.record().schedule();
    let stack = witness(&transaction, schedule)?;
    let decoded = transaction.inputs()[0].outpoint();
    if decoded != input.funded.outpoint {
        return Err(Refusal::SpentOutpoint {
            decoded,
            funded: input.funded.outpoint,
        });
    }
    let target = closure_target().map_err(|error| Refusal::Target(Box::new(error)))?;
    let metadata = decoded_witness_metadata(stack, schedule, &target)?;
    retained_context(&input, metadata)?;
    let predecessor = reconstruct(&target, metadata, input.bundle, TransactionSide::Input)?;
    predecessor_program(predecessor.output_program(), &input.funded.program)?;
    let predecessor_leastness = leastness(
        &target,
        &predecessor,
        input.bundle,
        TransactionSide::Input,
        input.bundle.policy().budget(),
    )?;
    let mut diagnostics = MaturityProjectionDiagnostics::empty();
    let (static_root, control_block) =
        predecessor_bindings(&target, stack, &predecessor, input.bundle, &mut diagnostics)?;
    let requested_cycle = Cycle::new(u64::from_be_bytes(fixed_bytes(&stack[2])));
    let bounds = input.bundle.deployment().lead_bounds().bounds();
    let expected = announce_maturity(&metadata.semantic, requested_cycle, bounds)
        .map_err(|error| Refusal::Transition(Box::new(error)))?;
    let mut semantic_comparison = transition_comparison(&predecessor, expected, requested_cycle);
    let successor_metadata = EncodedStateMetadata {
        semantic: expected,
        representation: StateRepresentationNonce::new(u32::from_be_bytes(fixed_bytes(&stack[1]))),
    };
    let successor = reconstruct(
        &target,
        successor_metadata,
        input.bundle,
        TransactionSide::Output,
    )?;
    let expected_fields = commitments(
        &encode_state_metadata(&expected, successor_metadata.representation),
        TransactionSide::Output,
    );
    semantic_comparison.reconstructed_against_expected = Some(field_comparisons(
        successor.field_commitments(),
        &expected_fields,
    ));
    diagnostics.semantic = Some(semantic_comparison.clone());
    successor_bindings(&transaction, stack, &successor, &mut diagnostics)?;
    let successor_leastness = leastness(
        &target,
        &successor,
        input.bundle,
        TransactionSide::Output,
        input.bundle.policy().budget(),
    )?;
    let static_comparison = static_comparison(
        [&predecessor, &successor],
        [&predecessor_leastness, &successor_leastness],
        static_root,
        control_block,
    );
    // Evidence assembly follows every existing byte-binding check, preserving the first-refusal order.
    let evidence = constructor_evidence(
        &target,
        [&predecessor, &successor],
        [&predecessor_leastness, &successor_leastness],
        input.bundle,
        &stack[8],
    )?;
    verify_acceptance_claim(claim, schedule, &transaction, input.submitted_bytes)?;
    Ok(ValidatedMaturityContinuity {
        source: input.source,
        submitted_bytes: input.submitted_bytes.to_vec(),
        byte_identity: sha256(input.submitted_bytes),
        transaction,
        funded: input.funded.clone(),
        branch: input.branch,
        identity: input.identity.clone(),
        bundle: input.bundle.clone(),
        bounds,
        requested_cycle,
        predecessor,
        successor,
        predecessor_leastness,
        successor_leastness,
        static_comparison,
        semantic_comparison,
        diagnostics,
        evidence,
        acceptance: Box::new(claim.clone()),
    })
}

/// Facts about one witnessed nonce, separate from the constructor a host search selects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityConstructorProjection {
    encoded_metadata: EncodedStateMetadata,
    metadata_bytes: Vec<u8>,
    static_subtree: StateStaticSubtree,
    merkle_root: [u8; 32],
    tweak_hash: [u8; 32],
    output_key: [u8; 32],
    parity: bool,
    output_program: Vec<u8>,
    control_recipe: StateControlRecipe,
    field_commitments: Vec<StateFieldCommitment>,
}

impl MaturityConstructorProjection {
    /// The decoded predecessor tuple or the expected successor with its witnessed nonce.
    #[must_use]
    pub const fn encoded_metadata(&self) -> &EncodedStateMetadata {
        &self.encoded_metadata
    }
    /// The representation nonce named by the bytes, not by a search.
    #[must_use]
    pub const fn nonce(&self) -> StateRepresentationNonce {
        self.encoded_metadata.representation
    }
    /// The exact canonical metadata encoding at this nonce.
    #[must_use]
    pub fn metadata_bytes(&self) -> &[u8] {
        &self.metadata_bytes
    }
    /// The supplied retained static descriptor used for reconstruction.
    #[must_use]
    pub const fn static_subtree(&self) -> &StateStaticSubtree {
        &self.static_subtree
    }
    /// The metadata/static outer root at this nonce.
    #[must_use]
    pub const fn merkle_root(&self) -> &[u8; 32] {
        &self.merkle_root
    }
    /// The tagged tweak digest at this nonce.
    #[must_use]
    pub const fn tweak_hash(&self) -> &[u8; 32] {
        &self.tweak_hash
    }
    /// The reconstructed x-only output key.
    #[must_use]
    pub const fn output_key(&self) -> &[u8; 32] {
        &self.output_key
    }
    /// Whether the reconstructed output point has odd y.
    #[must_use]
    pub const fn parity(&self) -> bool {
        self.parity
    }
    /// The program returned by the public fixed-nonce constructor entry.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }
    /// Reconstructed announcement control; the successor recipe is not an observed spend.
    #[must_use]
    pub const fn control_recipe(&self) -> &StateControlRecipe {
        &self.control_recipe
    }
    /// The six canonical semantic ranges for this transaction side.
    #[must_use]
    pub fn field_commitments(&self) -> &[StateFieldCommitment] {
        &self.field_commitments
    }
}

/// Host search evidence beside, never substituted for, the witnessed representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityNonceLeastness {
    witnessed_nonce: StateRepresentationNonce,
    host_least_constructor: CandidateStateConstructor,
    reconstructed_facts_match: Option<bool>,
}

impl MaturityNonceLeastness {
    /// The nonce read from the submitted witness.
    #[must_use]
    pub const fn witnessed_nonce(&self) -> StateRepresentationNonce {
        self.witnessed_nonce
    }
    /// The constructor selected by a separate scan from zero.
    #[must_use]
    pub const fn host_least_constructor(&self) -> &CandidateStateConstructor {
        &self.host_least_constructor
    }
    /// Every rejected lower nonce and the selected nonce.
    #[must_use]
    pub const fn evidence(&self) -> &StateNonceEvidence {
        self.host_least_constructor.evidence()
    }
    /// Whether the witnessed nonce equals the host's selected nonce.
    #[must_use]
    pub fn is_host_least(&self) -> bool {
        self.witnessed_nonce == self.host_least_constructor.nonce()
    }
    /// Equality of all recomputed facts with constructor readers when the nonces agree.
    ///
    /// `None` means different nonces, not target invalidity. The compared facts include the tuple, bytes, tree, root, tweak, key, parity, program, control recipe and all six field commitments.
    #[must_use]
    pub const fn reconstructed_facts_match(&self) -> Option<bool> {
        self.reconstructed_facts_match
    }
}

/// Byte bindings and retained recipe equalities, each kept separately.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityStaticComparison {
    static_root: MaturityByteComparison,
    control_block: MaturityByteComparison,
    descriptor_result: Result<(), Box<StateLinkRefusal>>,
    constructor_result: Result<(), Box<StateConstructorRefusal>>,
}

impl MaturityStaticComparison {
    /// Witnessed static root versus the retained descriptor's root.
    #[must_use]
    pub const fn static_root(&self) -> &MaturityByteComparison {
        &self.static_root
    }
    /// Witnessed predecessor control path versus the retained descriptor's reconstructed path.
    #[must_use]
    pub const fn control_block(&self) -> &MaturityByteComparison {
        &self.control_block
    }
    /// Whole predecessor descriptor versus the descriptor used for the successor.
    ///
    /// This projector uses the same retained tree for both; equality states that premise and is not a recovered hidden-subtree description.
    #[must_use]
    pub const fn descriptor_result(&self) -> &Result<(), Box<StateLinkRefusal>> {
        &self.descriptor_result
    }
    /// Root, internal-key and leaf-version continuity of the separately derived constructors.
    #[must_use]
    pub const fn constructor_result(&self) -> &Result<(), Box<StateConstructorRefusal>> {
        &self.constructor_result
    }
}

/// The attribution limit of an output commitment comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturitySuccessorAttribution {
    /// Another static tree can cause `OutputProgram`; the bytes alone do not identify that cause.
    CommitmentMismatchWithoutCauseAttribution,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckedMaterial {
    before: MaturityCheckedPrefix,
    after: MaturityCheckedPrefix,
    old_tweak: MaturityTweakEvidence,
    new_tweak: MaturityTweakEvidence,
    paths: MaturityControlPaths,
}

fn constructor_evidence(
    target: &ReviewedElementsTapscriptDefinition,
    projections: [&MaturityConstructorProjection; 2],
    leastness: [&MaturityNonceLeastness; 2],
    bundle: &CandidateLinkedMaturityBundle,
    witnessed: &[u8],
) -> ProjectionResult<CheckedMaterial> {
    let [before, after] = projections;
    let [old_least, new_least] = leastness;
    Ok(CheckedMaterial {
        before: checked_prefix(target, before, old_least, bundle, TransactionSide::Input)?,
        after: checked_prefix(target, after, new_least, bundle, TransactionSide::Output)?,
        old_tweak: tweak_evidence(target, before, bundle, TransactionSide::Input)?,
        new_tweak: tweak_evidence(target, after, bundle, TransactionSide::Output)?,
        paths: control_paths(target, before, after, bundle, witnessed)?,
    })
}

/// A privately constructed first-party projection, carrying no accepted-byte standing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedMaturityContinuity {
    source: MaturityByteSource,
    submitted_bytes: Vec<u8>,
    byte_identity: [u8; 32],
    transaction: TargetTransaction,
    funded: MaturityFundedPredecessor,
    branch: BranchContext,
    identity: CandidateDeploymentIdentity,
    bundle: CandidateLinkedMaturityBundle,
    bounds: AnnouncementLeadBounds,
    requested_cycle: Cycle,
    predecessor: MaturityConstructorProjection,
    successor: MaturityConstructorProjection,
    predecessor_leastness: MaturityNonceLeastness,
    successor_leastness: MaturityNonceLeastness,
    static_comparison: MaturityStaticComparison,
    semantic_comparison: MaturitySemanticComparison,
    diagnostics: MaturityProjectionDiagnostics,
    evidence: CheckedMaterial,
    acceptance: Box<MaturityAcceptanceObligation>,
}

impl ValidatedMaturityContinuity {
    /// Independently evaluated predecessor prefix and witnessed representation.
    #[must_use]
    pub const fn checked_predecessor(&self) -> &MaturityCheckedPrefix {
        &self.evidence.before
    }
    /// Independently evaluated successor prefix, retaining later-nonce admissibility.
    #[must_use]
    pub const fn checked_successor(&self) -> &MaturityCheckedPrefix {
        &self.evidence.after
    }
    /// Hash, point and parity comparisons for the predecessor.
    #[must_use]
    pub const fn predecessor_tweak(&self) -> &MaturityTweakEvidence {
        &self.evidence.old_tweak
    }
    /// Hash, point and parity comparisons for the successor.
    #[must_use]
    pub const fn successor_tweak(&self) -> &MaturityTweakEvidence {
        &self.evidence.new_tweak
    }
    /// Observed predecessor and derived, unobserved successor control evidence.
    #[must_use]
    pub const fn controls(&self) -> &MaturityControlPaths {
        &self.evidence.paths
    }
    /// The source class supplied with these bytes.
    #[must_use]
    pub const fn source(&self) -> &MaturityByteSource {
        &self.source
    }
    /// Exact submitted bytes, including witness and operator-signature bytes.
    #[must_use]
    pub fn submitted_bytes(&self) -> &[u8] {
        &self.submitted_bytes
    }
    /// SHA-256 of those exact bytes, not an accepted transaction identity.
    #[must_use]
    pub const fn byte_identity(&self) -> &[u8; 32] {
        &self.byte_identity
    }
    /// All decoded transaction facts, including output asset, value, nonce and program.
    #[must_use]
    pub const fn transaction(&self) -> &TargetTransaction {
        &self.transaction
    }
    /// Funding facts retained as caller-supplied evidence.
    #[must_use]
    pub const fn funded(&self) -> &MaturityFundedPredecessor {
        &self.funded
    }
    /// Caller-stated branch and checkpoint, with no freshness guarantee.
    #[must_use]
    pub const fn branch(&self) -> BranchContext {
        self.branch
    }
    /// Deployment identity compared with the recipe's identity.
    #[must_use]
    pub const fn identity(&self) -> &CandidateDeploymentIdentity {
        &self.identity
    }
    /// Retained recipe context, immutable through this record.
    #[must_use]
    pub const fn bundle(&self) -> &CandidateLinkedMaturityBundle {
        &self.bundle
    }
    /// The witness transport retained by the source's constructor recipe.
    #[must_use]
    pub const fn schedule(&self) -> StateWitnessSchedule {
        self.bundle.record().schedule()
    }
    /// The recipe's lead bounds used by the realization.
    #[must_use]
    pub const fn bounds(&self) -> AnnouncementLeadBounds {
        self.bounds
    }
    /// The cycle read from witness item two.
    #[must_use]
    pub const fn requested_cycle(&self) -> Cycle {
        self.requested_cycle
    }
    /// The predecessor reconstructed at its witnessed nonce.
    #[must_use]
    pub const fn predecessor(&self) -> &MaturityConstructorProjection {
        &self.predecessor
    }
    /// The expected successor reconstructed at its witnessed nonce.
    #[must_use]
    pub const fn successor(&self) -> &MaturityConstructorProjection {
        &self.successor
    }
    /// The realization's semantic result for the decoded predecessor and witnessed request.
    #[must_use]
    pub const fn expected_successor(&self) -> &StateMetadata {
        self.semantic_comparison.expected_successor()
    }
    /// The independent predecessor search and its equality facts.
    #[must_use]
    pub const fn predecessor_leastness(&self) -> &MaturityNonceLeastness {
        &self.predecessor_leastness
    }
    /// The independent successor search and its equality facts.
    #[must_use]
    pub const fn successor_leastness(&self) -> &MaturityNonceLeastness {
        &self.successor_leastness
    }
    /// Static byte bindings, descriptor equality and constructor continuity.
    #[must_use]
    pub const fn static_comparison(&self) -> &MaturityStaticComparison {
        &self.static_comparison
    }
    /// Transition field comparisons and reconstruction consistency, distinct from static facts.
    #[must_use]
    pub const fn semantic_comparison(&self) -> &MaturitySemanticComparison {
        &self.semantic_comparison
    }
    /// All witnessed binding operands reached by this successful projection.
    #[must_use]
    pub const fn diagnostics(&self) -> &MaturityProjectionDiagnostics {
        &self.diagnostics
    }
    /// An intended transition; no caller can supply a poison marker through this API.
    #[must_use]
    pub const fn observation_class(&self) -> MaturityObservationClass {
        MaturityObservationClass::IntendedTransition
    }
    /// The two admitted observation classes, conditional on no class-three observation.
    #[must_use]
    pub fn quantifier(&self) -> MaturityGuaranteeQuantifier {
        MaturityEvidenceCensus::default().quantifier()
    }
    /// The verified or passed-through obligation, outstanding when none was stated.
    #[must_use]
    pub fn acceptance_obligation(&self) -> MaturityAcceptanceObligation {
        self.acceptance.as_ref().clone()
    }
    /// An output mismatch cannot distinguish another static tree from another semantic commitment.
    #[must_use]
    pub const fn successor_attribution(&self) -> MaturitySuccessorAttribution {
        MaturitySuccessorAttribution::CommitmentMismatchWithoutCauseAttribution
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of};
    use crate::matrix::EvidenceBoundary;
    use crate::maturity_closure::{MaturityDeployment, maturity_sources};
    use crate::maturity_continuity_report::{
        MaturityContinuityReportEntry, assemble_maturity_continuity_report,
        render_maturity_continuity_report, validate_maturity_continuity_report,
    };
    use crate::maturity_corpus::{
        MATURITY_VARIABLE_RUN_ADDRESS, maturity_run_of_record, maturity_variable_run_of_record,
    };
    use crate::maturity_evidence::{
        MaturityConstructorMaterial, MaturityConstructorMaterialAbsence,
        MaturityExecutorProvenanceExpectation, derive_maturity_evidence_plan_with,
    };
    use crate::maturity_native::{MaturityAcceptedReadback, MaturityAnnouncementPlanner};
    use crate::subject::{ExperimentalSubject, SubjectStanding};
    use realization::ProtocolAmount;
    use std::fmt::Write as _;
    use std::io::Write as _;
    use std::sync::LazyLock;
    use target_elements_conformance::constructor::tagged::{
        TAP_BRANCH_TAG, TAP_LEAF_TAG, TAP_TWEAK_TAG, tagged_hash as constructor_tagged_hash,
    };
    use target_elements_conformance::executor::{OperationStep, TargetOperationPlanner};
    use target_elements_conformance::protocol::{
        FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, NativeResourceObservation,
        ObservedOutcomeLayer, OperationSubject, WireOutpoint,
    };
    use transaction::bytes::{AssetField, ValueField};
    use transaction::bytes::{InputWitness, TargetOutput, Txid};
    use transaction::state_abi::derive_maturity_announcement_abi;
    use transaction::state_construct::construct_maturity_announcement;
    use transaction::state_finalize::finalize_maturity_announcement;
    use transaction::state_view::{MaturityViewStatement, PublicMaturityStateView};
    use transaction::{MaturityAnnouncementRequest, RequestedForm, SponsorChangeRequest};

    #[derive(Clone)]
    #[expect(
        clippy::redundant_pub_crate,
        reason = "Shared with sibling history tests."
    )]
    pub(crate) struct Source {
        origin: MaturityByteSource,
        bytes: Vec<u8>,
        funded: MaturityFundedPredecessor,
        pub(crate) bundle: CandidateLinkedMaturityBundle,
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        acceptance: MaturityAcceptanceObligation,
        planner_successor: CandidateStateConstructor,
    }

    impl Source {
        pub(crate) fn input(&self) -> MaturityProjectionInput<'_> {
            MaturityProjectionInput {
                source: self.origin.clone(),
                submitted_bytes: &self.bytes,
                funded: &self.funded,
                branch: self.branch,
                bundle: &self.bundle,
                identity: &self.identity,
            }
        }

        fn project(&self) -> ProjectionResult<ValidatedMaturityContinuity> {
            project_maturity_continuity(self.input())
        }

        fn project_claimed(&self) -> ProjectionResult<ValidatedMaturityContinuity> {
            project_maturity_continuity_with_acceptance(self.input(), &self.acceptance)
        }
    }

    fn outstanding_acceptance() -> MaturityAcceptanceObligation {
        MaturityAcceptanceObligation::Outstanding {
            routes: [
                MaturityAcceptanceRoute::RelayWitnessRestructure,
                MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
            ],
        }
    }

    fn coin(funded: &FundedOutput) -> MaturityFundedPredecessor {
        MaturityFundedPredecessor {
            outpoint: outpoint_of(&funded.outpoint).expect("funded outpoint"),
            asset: asset_of(&funded.asset).expect("funded asset"),
            amount: funded.amount_satoshis,
            program: decode_hex(&funded.script).expect("funded program"),
        }
    }

    #[expect(
        clippy::redundant_pub_crate,
        reason = "Shared with sibling history tests."
    )]
    pub(crate) fn archived() -> &'static Source {
        static SOURCE: LazyLock<Source> = LazyLock::new(|| {
            let corpus = maturity_run_of_record().expect("validated archive");
            let identity = corpus.evidence().identity().clone();
            let branch = corpus.evidence().branch();
            let mut planner = MaturityAnnouncementPlanner::new(
                identity.clone(),
                branch,
                crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
            )
            .expect("planner");
            let mut next = planner.next_step(None).expect("issue");
            for (step, response) in &corpus.exchanges()[..2] {
                assert_eq!(next.as_ref(), Some(step));
                next = planner
                    .next_step(Some((step.case(), response)))
                    .expect("replay funding");
            }
            assert_eq!(next.as_ref(), Some(&corpus.exchanges()[2].0));
            let OperationSubject::Submission(submission) = corpus.exchanges()[2].0.subject() else {
                panic!("submission")
            };
            assert_eq!(
                planner.submission_bytes(),
                Some(submission.transaction_bytes.as_slice())
            );
            let [funded] = corpus.exchanges()[1].1.funded_outputs.as_slice() else {
                panic!("one funded coin")
            };
            Source {
                origin: MaturityByteSource::ArchivedSubmission {
                    run_address: corpus.report().run_address().to_owned(),
                },
                bytes: submission.transaction_bytes.clone(),
                funded: coin(funded),
                bundle: planner.bundle().clone(),
                identity,
                branch,
                acceptance: outstanding_acceptance(),
                planner_successor: planner
                    .announcement()
                    .expect("announcement")
                    .construction()
                    .successor_constructor()
                    .clone(),
            }
        });
        &SOURCE
    }

    #[expect(
        clippy::redundant_pub_crate,
        reason = "Shared with sibling history tests."
    )]
    pub(crate) fn variable_archived() -> &'static Source {
        static SOURCE: LazyLock<Source> = LazyLock::new(|| {
            let corpus = maturity_variable_run_of_record().expect("validated accepted archive");
            let identity = corpus.evidence().identity().clone();
            let branch = corpus.evidence().branch();
            let mut planner = MaturityAnnouncementPlanner::new(
                identity.clone(),
                branch,
                crate::maturity_closure::MaturityWitnessSelection::Retained(
                    StateWitnessSchedule::VariableMetadata,
                ),
            )
            .expect("variable planner");
            let mut next = planner.next_step(None).expect("issue");
            for (step, response) in &corpus.exchanges()[..2] {
                assert_eq!(next.as_ref(), Some(step));
                next = planner
                    .next_step(Some((step.case(), response)))
                    .expect("replay accepted funding");
            }
            assert_eq!(next.as_ref(), Some(&corpus.exchanges()[2].0));
            let OperationSubject::Submission(submission) = corpus.exchanges()[2].0.subject() else {
                panic!("accepted submission")
            };
            assert_eq!(
                planner.submission_bytes(),
                Some(submission.transaction_bytes.as_slice())
            );
            let [funded] = corpus.exchanges()[1].1.funded_outputs.as_slice() else {
                panic!("one accepted funded coin")
            };
            Source {
                origin: MaturityByteSource::ArchivedSubmission {
                    run_address: MATURITY_VARIABLE_RUN_ADDRESS.to_owned(),
                },
                bytes: submission.transaction_bytes.clone(),
                funded: coin(funded),
                bundle: planner.bundle().clone(),
                identity,
                branch,
                acceptance: corpus.evidence().acceptance_obligation().clone(),
                planner_successor: planner
                    .announcement()
                    .expect("variable announcement")
                    .construction()
                    .successor_constructor()
                    .clone(),
            }
        });
        &SOURCE
    }

    fn hex(bytes: &[u8]) -> String {
        let mut text = String::new();
        for byte in bytes {
            write!(text, "{byte:02x}").expect("String write");
        }
        text
    }

    fn scripted_response(step: &OperationStep) -> NativeOperationResponse {
        let OperationSubject::Funding(subject) = step.subject() else {
            panic!("funding subject")
        };
        let asset = format!("01{}fe", "55".repeat(30));
        NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: subject.issue_asset.then(|| asset.clone()),
            funded_outputs: vec![FundedOutput {
                outpoint: WireOutpoint {
                    txid: format!("02{}fd", "66".repeat(30)),
                    vout: if subject.issue_asset { 3 } else { 7 },
                },
                asset,
                amount_satoshis: subject.amount_per_output,
                script: hex(&subject.output_program),
            }],
            confidential_funded_outputs: Vec::new(),
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        }
    }

    fn node_free_with_schedule(schedule: StateWitnessSchedule) -> Source {
        let mut genesis = [0x22; 32];
        genesis[0] = 0x01;
        genesis[31] = 0xfe;
        let identity = CandidateDeploymentIdentity::new([0x17; 32], genesis).expect("identity");
        let branch = BranchContext::new([0x41; 32], 7).expect("branch");
        let mut planner = MaturityAnnouncementPlanner::new(
            identity.clone(),
            branch,
            crate::maturity_closure::MaturityWitnessSelection::Retained(schedule),
        )
        .expect("planner");
        let issue = planner.next_step(None).expect("issue").expect("step");
        let issued = scripted_response(&issue);
        issued.validate_shape().expect("issuance shape");
        let funding = planner
            .next_step(Some((issue.case(), &issued)))
            .expect("funding")
            .expect("step");
        let response = scripted_response(&funding);
        response.validate_shape().expect("funding shape");
        let submission = planner
            .next_step(Some((funding.case(), &response)))
            .expect("submission")
            .expect("step");
        let OperationSubject::Submission(subject) = submission.subject() else {
            panic!("submission")
        };
        assert_eq!(
            planner.submission_bytes(),
            Some(subject.transaction_bytes.as_slice())
        );
        Source {
            origin: MaturityByteSource::NodeFreeSubmitReady,
            bytes: subject.transaction_bytes.clone(),
            funded: coin(&response.funded_outputs[0]),
            bundle: planner.bundle().clone(),
            identity,
            branch,
            acceptance: outstanding_acceptance(),
            planner_successor: planner
                .announcement()
                .expect("announcement")
                .construction()
                .successor_constructor()
                .clone(),
        }
    }

    #[expect(
        clippy::redundant_pub_crate,
        reason = "Shared with sibling history tests."
    )]
    pub(crate) fn node_free() -> &'static Source {
        static SOURCE: LazyLock<Source> =
            LazyLock::new(|| node_free_with_schedule(StateWitnessSchedule::WholeMetadata));
        &SOURCE
    }

    fn variable_node_free() -> &'static Source {
        static SOURCE: LazyLock<Source> =
            LazyLock::new(|| node_free_with_schedule(StateWitnessSchedule::VariableMetadata));
        &SOURCE
    }

    fn positive(source: &Source) -> ValidatedMaturityContinuity {
        let record = source.project().expect("byte projection");
        let target = closure_target().expect("target");
        let item = &record.transaction().witnesses()[0].stack()[4];
        let canonical = match record.schedule() {
            StateWitnessSchedule::WholeMetadata => item.clone(),
            StateWitnessSchedule::VariableMetadata => {
                rebuild_state_metadata(&fixed_bytes::<STATE_METADATA_VARIABLE_BYTES>(item)).to_vec()
            }
        };
        let encoded = decode_state_metadata(&canonical).expect("metadata");
        assert_eq!(record.predecessor().encoded_metadata(), &encoded);
        assert_eq!(encoded.semantic.cycle, Cycle::new(5));
        assert_eq!(
            (record.bounds().minimum(), record.bounds().maximum()),
            (Cycle::new(4), Cycle::new(6))
        );
        assert_eq!(record.requested_cycle(), Cycle::new(10));
        let expected =
            announce_maturity(&encoded.semantic, record.requested_cycle(), record.bounds())
                .expect("transition");
        assert_eq!(record.expected_successor(), &expected);
        assert_eq!(
            record.successor().encoded_metadata(),
            source.planner_successor.encoded_metadata()
        );
        for (projection, least, side) in [
            (
                record.predecessor(),
                record.predecessor_leastness(),
                TransactionSide::Input,
            ),
            (
                record.successor(),
                record.successor_leastness(),
                TransactionSide::Output,
            ),
        ] {
            assert_eq!(
                projection.output_program(),
                state_output_program_at_nonce(
                    &target,
                    projection.encoded_metadata(),
                    source.bundle.static_subtree(),
                    source.bundle.policy().internal_key(),
                    &OracleStateCurve
                )
                .expect("fixed nonce")
            );
            let derived = CandidateStateConstructor::derive(
                &target,
                &projection.encoded_metadata().semantic,
                source.bundle.static_subtree(),
                source.bundle.policy().internal_key(),
                source.bundle.policy().budget(),
                &OracleStateCurve,
            )
            .expect("derive");
            assert_eq!(least.host_least_constructor(), &derived);
            assert!(least.is_host_least());
            assert_eq!(least.reconstructed_facts_match(), Some(true));
            assert!(facts_match(projection, &derived, side).expect("all facts"));
        }
        assert!(record.static_comparison().static_root().agrees());
        assert!(record.static_comparison().control_block().agrees());
        assert_eq!(record.static_comparison().descriptor_result(), &Ok(()));
        assert_eq!(record.static_comparison().constructor_result(), &Ok(()));
        assert!(record.semantic_comparison().transition_agrees());
        assert_eq!(
            record.semantic_comparison().reconstruction_agrees(),
            Some(true)
        );
        assert_eq!(record.byte_identity(), &sha256(&source.bytes));
        assert_eq!(record.funded(), &source.funded);
        record
    }

    fn report_observed(source: &str, record: &ValidatedMaturityContinuity) {
        let widths: Vec<_> = record.transaction().witnesses()[0]
            .stack()
            .iter()
            .map(Vec::len)
            .collect();
        writeln!(std::io::stdout().lock(), "\nRUN-REPORT projection source={source} predecessor_nonce={} successor_nonce={} request={} widths={widths:?} predecessor_rejected={:?} successor_rejected={:?} leastness=({},{}) facts=({:?},{:?}) static=(true,true) descriptors=ok constructor=ok semantic=(true,true)",
            record.predecessor().nonce().get(), record.successor().nonce().get(), record.requested_cycle().get(),
            record.predecessor_leastness().evidence().rejected, record.successor_leastness().evidence().rejected,
            record.predecessor_leastness().is_host_least(), record.successor_leastness().is_host_least(),
            record.predecessor_leastness().reconstructed_facts_match(), record.successor_leastness().reconstructed_facts_match()).expect("test observation output");
    }

    struct ExperimentalBytes {
        bytes: ExperimentalSubject<Vec<u8>>,
        original_boundary: EvidenceBoundary,
        actual_carrier: &'static str,
    }

    impl ExperimentalBytes {
        fn new(bytes: Vec<u8>, actual_carrier: &'static str) -> Self {
            Self {
                bytes: ExperimentalSubject::observe(bytes),
                original_boundary: EvidenceBoundary::RelayPolicyRejection,
                actual_carrier,
            }
        }

        fn project(&self, source: &Source) -> ProjectionResult<ValidatedMaturityContinuity> {
            assert_eq!(self.bytes.standing(), SubjectStanding::Experimental);
            assert_eq!(
                self.original_boundary,
                EvidenceBoundary::RelayPolicyRejection
            );
            assert_ne!(self.actual_carrier.len(), 0);
            let mut input = source.input();
            input.submitted_bytes = self.bytes.subject();
            project_maturity_continuity(input)
        }
    }

    fn changed_stack(
        source: &Source,
        carrier: &'static str,
        change: impl FnOnce(&mut Vec<Vec<u8>>),
    ) -> ExperimentalBytes {
        let transaction = TargetTransaction::decode(&source.bytes).expect("decode");
        let mut stack = transaction.witnesses()[0].stack().to_vec();
        change(&mut stack);
        ExperimentalBytes::new(
            TargetTransaction::with_output_witnesses(
                transaction.version(),
                transaction.inputs().to_vec(),
                transaction.outputs().to_vec(),
                transaction.lock_time(),
                vec![InputWitness::new(stack)],
                transaction.output_witnesses().to_vec(),
            )
            .expect("mutant encoding")
            .encode(),
            carrier,
        )
    }

    fn changed_program(source: &Source, program: Vec<u8>) -> ExperimentalBytes {
        let transaction = TargetTransaction::decode(&source.bytes).expect("decode");
        let output = &transaction.outputs()[0];
        let replaced = TargetOutput::new(output.asset(), output.value(), output.nonce(), program);
        ExperimentalBytes::new(
            TargetTransaction::with_output_witnesses(
                transaction.version(),
                transaction.inputs().to_vec(),
                vec![replaced],
                transaction.lock_time(),
                transaction.witnesses().to_vec(),
                transaction.output_witnesses().to_vec(),
            )
            .expect("mutant encoding")
            .encode(),
            "output zero program",
        )
    }

    fn coherent_source(deployment: MaturityDeployment) -> Source {
        let target = closure_target().expect("target");
        let bundle = maturity_sources(
            deployment,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("sources")
        .link(&OracleStateCurve)
        .expect("link");
        let predecessor = &bundle.instances()[0];
        let metadata = *predecessor.metadata();
        let identity = bundle.deployment().identity().clone();
        let branch = archived().branch;
        let funded = MaturityFundedPredecessor {
            outpoint: archived().funded.outpoint,
            asset: AssetId::from_internal(
                *deployment.parameters().expect("parameters").singleton(),
            ),
            amount: 1,
            program: predecessor.constructor().output_program(),
        };
        let view = PublicMaturityStateView::new([
            MaturityViewStatement::CurrentStateOutpoint(funded.outpoint),
            MaturityViewStatement::AssetAndAmount(
                AssetField::Explicit(funded.asset),
                ValueField::Explicit(funded.amount),
            ),
            MaturityViewStatement::PredecessorMetadata(metadata.semantic),
            MaturityViewStatement::PredecessorRepresentationNonce(metadata.representation),
            MaturityViewStatement::CurrentRootBinding(branch),
            MaturityViewStatement::PredecessorProgram(funded.program.clone()),
            MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
        ])
        .expect("view")
        .validate(&target, &OracleStateCurve)
        .expect("validated view");
        let abi = derive_maturity_announcement_abi(&target, &view).expect("ABI");
        let request = MaturityAnnouncementRequest::new(
            Cycle::new(10),
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
        )
        .expect("request");
        let construction =
            construct_maturity_announcement(&target, &abi, &view, &request, &OracleStateCurve)
                .expect("construction");
        let finalized = finalize_maturity_announcement(construction);
        let successor = finalized.construction().successor_constructor().clone();
        let bytes = placeholder_submission(&finalized, &bundle);
        Source {
            origin: MaturityByteSource::NodeFreeSubmitReady,
            bytes,
            funded,
            bundle,
            identity,
            branch,
            acceptance: outstanding_acceptance(),
            planner_successor: successor,
        }
    }

    fn placeholder_submission(
        finalized: &transaction::state_finalize::FinalizedMaturityAnnouncement,
        bundle: &CandidateLinkedMaturityBundle,
    ) -> Vec<u8> {
        let target = closure_target().expect("target");
        let predecessor = &bundle.instances()[0];
        let metadata = predecessor.metadata();
        let successor = finalized.construction().successor_constructor();
        let request = finalized.construction().request();
        // A designated placeholder is coherent for this signature-blind projector; it establishes neither authorization nor target acceptance. Second has no signer material.
        let stack = vec![
            vec![prefix(successor.parity())],
            successor.nonce().get().to_be_bytes().to_vec(),
            request.announced_cycle().get().to_be_bytes().to_vec(),
            bundle.static_subtree().root().to_vec(),
            encode_state_metadata(&metadata.semantic, metadata.representation),
            vec![prefix(predecessor.constructor().parity())],
            vec![0; 64],
            linked_announcement_bytes(bundle, &target).expect("leaf"),
            predecessor
                .constructor()
                .control_recipe(StateLeafRole::Announcement)
                .expect("recipe")
                .control_bytes()
                .expect("control"),
        ];
        let tx = finalized.protected();
        TargetTransaction::with_output_witnesses(
            tx.version(),
            tx.inputs().to_vec(),
            tx.outputs().to_vec(),
            tx.lock_time(),
            vec![InputWitness::new(stack)],
            tx.output_witnesses().to_vec(),
        )
        .expect("witness assembly")
        .encode()
    }

    fn mutation_context(
        section: MaturitySafetySection,
        name: &str,
        layer: MutationLayer,
        description: &str,
        items: &[usize],
        output: Option<usize>,
    ) -> MaturityMutationContext {
        MaturityMutationContext::new(
            section,
            name,
            layer,
            MaturityMutationCarrier::new(
                description.to_owned(),
                items.to_vec(),
                output,
                "explicit retained bundle; deployment pairing named in description".to_owned(),
                MaturitySignatureDisposition::PlaceholderNoAuthorization,
            ),
        )
        .expect("matrix row")
    }

    fn negative(
        source: &Source,
        context: MaturityMutationContext,
    ) -> ExperimentalSubject<MaturityContinuityMutant> {
        let result =
            MaturityContinuityMutant::observe(context, source.input()).expect("observed refusal");
        assert_eq!(result.standing(), SubjectStanding::Experimental);
        let subject = result.subject();
        assert_eq!(subject.submitted(), source.bytes);
        assert_eq!(subject.digest(), &sha256(&source.bytes));
        let context = subject.context();
        assert_eq!(context.declared_boundary(), context.row().boundary());
        assert_eq!(context.declared_layer(), context.row().mutation());
        assert_eq!(context.refusing_layer(), context.row().refusing_layer());
        assert_eq!(context.intended_carrier(), context.row().carrier());
        writeln!(std::io::stdout().lock(), "\nRUN-REPORT negative section={:?} row={} boundary={:?} declared_layer={:?} applied_layer={:?} carrier={:?} refusal={}", context.row().section(), context.row().name(), context.declared_boundary(), context.declared_layer(), context.applied_layer(), context.actual(), refusal_name(subject.failure())).expect("negative observation");
        result
    }

    fn refusal_name(refusal: &Refusal) -> &'static str {
        match refusal {
            Refusal::RetainedContext { .. } => "RetainedContext",
            Refusal::OutputProgram(_) => "OutputProgram",
            Refusal::ControlBlock(_) => "ControlBlock",
            Refusal::PredecessorPrefix(_) => "PredecessorPrefix",
            Refusal::SuccessorPrefix(_) => "SuccessorPrefix",
            _ => "other (asserted by test)",
        }
    }

    fn substituted_successor(
        source: &Source,
        semantic: StateMetadata,
        other: &CandidateLinkedMaturityBundle,
    ) -> Source {
        let target = closure_target().expect("target");
        let honest = source.planner_successor.encoded_metadata().semantic;
        let (expected, supplied) = (0..4096)
            .find_map(|value| {
                let nonce = StateRepresentationNonce::new(value);
                let expected = fixed_projection(
                    &target,
                    EncodedStateMetadata {
                        semantic: honest,
                        representation: nonce,
                    },
                    &source.bundle,
                    TransactionSide::Output,
                )
                .ok()?;
                let supplied = fixed_projection(
                    &target,
                    EncodedStateMetadata {
                        semantic,
                        representation: nonce,
                    },
                    other,
                    TransactionSide::Output,
                )
                .ok()?;
                Some((expected, supplied))
            })
            .expect("common admissible representation");
        let rebuild = |projection: &MaturityConstructorProjection| {
            let mut result = source.clone();
            result.bytes = changed_program(source, projection.output_program().to_vec())
                .bytes
                .into_subject();
            result.bytes = changed_stack(&result, "coherent output nonce and parity", |stack| {
                stack[0] = vec![prefix(projection.parity())];
                stack[1] = projection.nonce().get().to_be_bytes().to_vec();
            })
            .bytes
            .into_subject();
            result
        };
        rebuild(&expected)
            .project()
            .expect("honest common-nonce control");
        rebuild(&supplied)
    }

    fn assert_output_negative(subject: &MaturityContinuityMutant, source: &Source) {
        let Refusal::OutputProgram(diagnostics) = subject.failure() else {
            panic!("observed OutputProgram")
        };
        assert!(diagnostics.static_root().expect("root").agrees());
        assert!(diagnostics.control_block().expect("control").agrees());
        let semantic = diagnostics.semantic().expect("expected successor");
        assert!(semantic.transition_agrees());
        assert_eq!(semantic.reconstruction_agrees(), Some(true));
        assert_eq!(
            semantic.expected_successor(),
            &source.planner_successor.encoded_metadata().semantic
        );
        let comparison = diagnostics.output_program().expect("output operands");
        assert!(!comparison.agrees());
        let tx = TargetTransaction::decode(subject.submitted()).expect("decode mutant");
        let nonce = StateRepresentationNonce::new(u32::from_be_bytes(fixed_bytes(
            &tx.witnesses()[0].stack()[1],
        )));
        let expected = state_output_program_at_nonce(
            &closure_target().expect("target"),
            &EncodedStateMetadata {
                semantic: *semantic.expected_successor(),
                representation: nonce,
            },
            source.bundle.static_subtree(),
            source.bundle.policy().internal_key(),
            &OracleStateCurve,
        )
        .expect("expected program");
        assert_eq!(comparison.reconstructed(), expected);
        assert_eq!(comparison.witnessed(), tx.outputs()[0].program());
    }

    #[test]
    fn accepted_archive_claim_survives_projection() {
        let source = variable_archived();
        let record = source
            .project_claimed()
            .expect("accepted archive projection");
        let corpus = maturity_variable_run_of_record().expect("accepted archive");
        assert_eq!(
            record.acceptance_obligation(),
            corpus.evidence().acceptance_obligation().clone()
        );
        let MaturityAcceptanceObligation::Established {
            schedule,
            identity,
            readback,
        } = record.acceptance_obligation()
        else {
            panic!("established acceptance")
        };
        assert_eq!(schedule, StateWitnessSchedule::VariableMetadata);
        assert_eq!(identity, readback.identity());
        assert_eq!(readback.bytes(), source.bytes.as_slice());
    }

    #[test]
    fn a_claim_under_another_schedule_refuses() {
        let mut source = archived().clone();
        source.acceptance = variable_archived().acceptance.clone();
        let Refusal::AcceptanceClaim(mismatch) =
            source.project_claimed().expect_err("schedule disagreement")
        else {
            panic!("acceptance claim refusal")
        };
        assert_eq!(
            *mismatch,
            MaturityAcceptanceClaimMismatch::Schedule {
                claimed: StateWitnessSchedule::VariableMetadata,
                retained: StateWitnessSchedule::WholeMetadata,
            }
        );
    }

    #[test]
    fn a_claim_whose_readback_bytes_are_not_the_submitted_bytes_refuses() {
        let mut source = variable_node_free().clone();
        source.acceptance = variable_archived().acceptance.clone();
        let Refusal::AcceptanceClaim(mismatch) = source
            .project_claimed()
            .expect_err("readback bytes disagreement")
        else {
            panic!("acceptance claim refusal")
        };
        let MaturityAcceptanceClaimMismatch::ReadbackBytes { claimed, submitted } = *mismatch
        else {
            panic!("readback bytes mismatch")
        };
        let MaturityAcceptanceObligation::Established { readback, .. } = &source.acceptance else {
            panic!("established acceptance")
        };
        assert_ne!(readback.bytes(), source.bytes.as_slice());
        assert_eq!(claimed.as_slice(), readback.bytes());
        assert_eq!(submitted.as_slice(), source.bytes.as_slice());
    }

    #[test]
    fn a_claim_naming_another_identity_refuses() {
        let mut source = variable_archived().clone();
        let MaturityAcceptanceObligation::Established {
            identity, readback, ..
        } = &mut source.acceptance
        else {
            panic!("established acceptance")
        };
        let recomputed = readback.identity();
        let claimed = Txid::from_internal([0x7d; 32]);
        assert_ne!(claimed, recomputed);
        *identity = claimed;
        let Refusal::AcceptanceClaim(mismatch) =
            source.project_claimed().expect_err("identity disagreement")
        else {
            panic!("acceptance claim refusal")
        };
        assert_eq!(
            *mismatch,
            MaturityAcceptanceClaimMismatch::Identity {
                claimed,
                recomputed,
            }
        );
    }

    #[test]
    fn a_claim_whose_readback_witness_identity_is_wrong_refuses() {
        let mut source = variable_archived().clone();
        let MaturityAcceptanceObligation::Established { readback, .. } = &mut source.acceptance
        else {
            panic!("established acceptance")
        };
        let original = readback.clone();
        let recomputed = original.witness_identity();
        let claimed = Txid::from_internal([0x6d; 32]);
        assert_ne!(claimed, recomputed);
        *readback = MaturityAcceptedReadback::from_parts(
            original.identity(),
            claimed,
            *original.block_hash(),
            original.block_height(),
            original.bytes().to_vec(),
        );
        let Refusal::AcceptanceClaim(mismatch) = source
            .project_claimed()
            .expect_err("witness identity disagreement")
        else {
            panic!("acceptance claim refusal")
        };
        assert_eq!(
            *mismatch,
            MaturityAcceptanceClaimMismatch::WitnessIdentity {
                claimed,
                recomputed,
            }
        );
    }

    #[test]
    fn an_outstanding_claim_leaves_every_projection_outstanding() {
        let claim = outstanding_acceptance();
        for source in [
            archived(),
            node_free(),
            variable_node_free(),
            variable_archived(),
        ] {
            let stated = project_maturity_continuity_with_acceptance(source.input(), &claim)
                .expect("stated outstanding projection");
            let unstated = source.project().expect("unstated projection");
            assert_eq!(
                stated.acceptance_obligation(),
                unstated.acceptance_obligation()
            );
            assert!(matches!(
                stated.acceptance_obligation(),
                MaturityAcceptanceObligation::Outstanding { .. }
            ));
        }
    }

    #[test]
    fn the_published_identities_agree_with_the_accepted_readback() {
        let source = variable_archived();
        let transaction = TargetTransaction::decode(&source.bytes).expect("accepted transaction");
        let identities = submitted_transaction_identities(&transaction, &source.bytes);
        let MaturityAcceptanceObligation::Established { readback, .. } = &source.acceptance else {
            panic!("established acceptance")
        };
        assert_eq!(identities.identity(), readback.identity());
        assert_eq!(identities.witness_identity(), readback.witness_identity());
    }

    #[test]
    fn archived_prefix_is_checked_hash_by_hash() {
        let record = archived().project().expect("projection");
        let before = record.checked_predecessor();
        assert_eq!(before.side(), TransactionSide::Input);
        assert_eq!(before.rejected().len(), 1);
        let (lower, cause) = &before.rejected()[0];
        assert_eq!(lower.candidate(), StateRepresentationNonce::ZERO);
        assert_eq!(
            *cause,
            StateConstructorRefusal::CanonicalBranchSideNotSatisfied
        );
        assert!(lower.leaf_digest() > lower.retained_root());
        assert_eq!(lower.curve_outcome(), None);
        assert!(before.selected().branch_admissible());
        assert_eq!(before.selected().candidate().get(), 1);
        assert_eq!(before.witnessed(), before.selected());
        assert!(before.is_host_least());
        assert_eq!(before.residual().text(), StateNonceEvidence::RESIDUAL);
        assert_eq!(record.checked_successor().rejected().len(), 0);
        writeln!(
            std::io::stdout().lock(),
            "\nRUN-REPORT checked-prefix predecessor={before:?} successor={:?}",
            record.checked_successor()
        )
        .expect("prefix observation");
    }

    #[test]
    fn prefix_disagreement_is_refused() {
        let source = archived();
        let record = source.project().expect("projection");
        let mut projection = record.predecessor().clone();
        projection.metadata_bytes[25] ^= 1;
        assert_eq!(
            checked_prefix(
                &closure_target().expect("target"),
                &projection,
                record.predecessor_leastness(),
                &source.bundle,
                TransactionSide::Input
            ),
            Err(Refusal::PrefixEvidence {
                side: TransactionSide::Input,
                comparison: MaturityEvidenceMismatch::Metadata
            })
        );
        let mut least = record.predecessor_leastness().clone();
        least.witnessed_nonce = StateRepresentationNonce::ZERO;
        assert_eq!(
            checked_prefix(
                &closure_target().expect("target"),
                record.predecessor(),
                &least,
                &source.bundle,
                TransactionSide::Input
            ),
            Err(Refusal::PrefixEvidence {
                side: TransactionSide::Input,
                comparison: MaturityEvidenceMismatch::NonceOrder
            })
        );
    }

    #[test]
    fn tweak_evidence_recomputes_both_sides() {
        use MaturityCommitmentAssumption as A;

        for source in [archived(), node_free()] {
            let record = source.project().expect("projection");
            for evidence in [record.predecessor_tweak(), record.successor_tweak()] {
                for comparison in [
                    evidence.internal(),
                    evidence.metadata(),
                    evidence.branch(),
                    evidence.digest(),
                    evidence.key(),
                    evidence.oddness(),
                    evidence.program(),
                ] {
                    assert!(comparison.agrees());
                }
                assert_eq!(
                    evidence.premises(),
                    &[
                        A::LeafCollisionResistance,
                        A::LeafSecondPreimageResistance,
                        A::BranchCollisionResistance,
                        A::BranchSecondPreimageResistance,
                        A::TweakCollisionResistance,
                        A::TweakSecondPreimageResistance,
                        A::NumsDiscreteLogAndPreimageResistance,
                    ]
                );
                assert_eq!(
                    evidence.premises()[6].domain_or_residual(),
                    tapscript::StateInternalKeyPolicy::RESIDUAL
                );
                writeln!(std::io::stdout().lock(), "\nRUN-REPORT tweak source={:?} side={:?} nums={} leaf={} branch={} digest={} key={} parity={:?} comparisons=all-agree", source.origin, evidence.side(), hex(evidence.internal().reconstructed()), hex(evidence.metadata().reconstructed()), hex(evidence.branch().reconstructed()), hex(evidence.digest().reconstructed()), hex(evidence.key().reconstructed()), evidence.oddness().reconstructed()).expect("tweak observation");
            }
        }
    }

    #[test]
    fn commitment_premises_name_constructor_hash_domains() {
        use MaturityCommitmentAssumption as A;

        for (premise, expected) in [
            (A::LeafCollisionResistance, TAP_LEAF_TAG),
            (A::LeafSecondPreimageResistance, TAP_LEAF_TAG),
            (A::BranchCollisionResistance, TAP_BRANCH_TAG),
            (A::BranchSecondPreimageResistance, TAP_BRANCH_TAG),
            (A::TweakCollisionResistance, TAP_TWEAK_TAG),
            (A::TweakSecondPreimageResistance, TAP_TWEAK_TAG),
            (
                A::NumsDiscreteLogAndPreimageResistance,
                tapscript::StateInternalKeyPolicy::RESIDUAL,
            ),
        ] {
            assert_eq!(
                premise.domain_or_residual(),
                expected,
                "wrong tagged-hash domain for {premise:?}"
            );
        }

        assert_ne!(TAP_LEAF_TAG, TAP_BRANCH_TAG);
        assert_ne!(TAP_LEAF_TAG, TAP_TWEAK_TAG);
        assert_ne!(TAP_BRANCH_TAG, TAP_TWEAK_TAG);

        let message = b"commitment premise";
        let leaf = constructor_tagged_hash(TAP_LEAF_TAG, message);
        let branch = constructor_tagged_hash(TAP_BRANCH_TAG, message);
        let tweak = constructor_tagged_hash(TAP_TWEAK_TAG, message);
        assert_ne!(leaf, branch);
        assert_ne!(leaf, tweak);
        assert_ne!(branch, tweak);
    }

    #[test]
    fn tweak_disagreement_is_refused() {
        let source = archived();
        let record = source.project().expect("projection");
        let mut projection = record.successor().clone();
        projection.tweak_hash[0] ^= 1;
        assert_eq!(
            tweak_evidence(
                &closure_target().expect("target"),
                &projection,
                &source.bundle,
                TransactionSide::Output
            ),
            Err(Refusal::TweakEvidence {
                side: TransactionSide::Output,
                comparison: MaturityEvidenceMismatch::Digest
            })
        );
        projection = record.successor().clone();
        projection.parity = !projection.parity;
        assert_eq!(
            tweak_evidence(
                &closure_target().expect("target"),
                &projection,
                &source.bundle,
                TransactionSide::Output
            ),
            Err(Refusal::TweakEvidence {
                side: TransactionSide::Output,
                comparison: MaturityEvidenceMismatch::Parity
            })
        );
    }

    #[test]
    fn controls_keep_observed_and_derived_separate() {
        let record = archived().project().expect("projection");
        let controls = record.controls();
        assert_eq!(
            controls.observations(),
            [
                MaturityControlObservation::ObservedPredecessorSpend,
                MaturityControlObservation::DerivedSuccessorUnobserved
            ]
        );
        assert!(controls.observed().agrees());
        assert_eq!(
            controls.observed().witnessed(),
            record.transaction().witnesses()[0].stack()[8]
        );
        assert_eq!(controls.before(), record.predecessor().control_recipe());
        assert_eq!(controls.after(), record.successor().control_recipe());
        assert_eq!(
            controls.derived(),
            controls.after().control_bytes().expect("derived bytes")
        );
        assert_eq!(controls.lengths(), (65, 65));
        assert!(controls.inner().agrees());
        assert_eq!(controls.inner().witnessed().len(), 0);
        assert!(controls.outer().iter().all(MaturityByteComparison::agrees));
        let (actual, expected) = controls.first_byte_relation();
        assert_eq!(actual, expected);
        writeln!(std::io::stdout().lock(), "\nRUN-REPORT controls lengths={:?} internal_siblings=0 outer=both-metadata first_xor={actual} parity_xor={expected} predecessor=observed successor=derived-unobserved", controls.lengths()).expect("control observation");
    }

    #[test]
    fn control_relations_and_disagreement_are_measured() {
        let source = archived();
        let record = source.project().expect("projection");
        let target = closure_target().expect("target");
        let mut before = record.predecessor().clone();
        before.control_recipe.parity = !before.control_recipe.parity;
        assert_eq!(
            control_paths(
                &target,
                &before,
                record.successor(),
                &source.bundle,
                &record.transaction().witnesses()[0].stack()[8]
            ),
            Err(Refusal::ControlEvidence {
                side: TransactionSide::Input,
                comparison: MaturityEvidenceMismatch::Recipe
            })
        );
        let mut seen = [false; 2];
        for value in 0..256 {
            let Ok(after) = fixed_projection(
                &target,
                EncodedStateMetadata {
                    semantic: *record.expected_successor(),
                    representation: StateRepresentationNonce::new(value),
                },
                &source.bundle,
                TransactionSide::Output,
            ) else {
                continue;
            };
            let controls = control_paths(
                &target,
                record.predecessor(),
                &after,
                &source.bundle,
                &record.transaction().witnesses()[0].stack()[8],
            )
            .expect("independent paths");
            let (actual, expected) = controls.first_byte_relation();
            assert_eq!(actual, expected);
            assert!(controls.inner().agrees());
            assert!(controls.outer().iter().all(MaturityByteComparison::agrees));
            seen[usize::from(expected)] = true;
            if seen == [true, true] {
                break;
            }
        }
        assert_eq!(seen, [true, true]);
    }

    #[test]
    fn mutant_context_refuses_absent_rows() {
        let context = mutation_context(
            MaturitySafetySection::PredecessorConstructorFault,
            "wrong-control-block",
            MutationLayer::WitnessProof,
            "placeholder control subject",
            &[8],
            None,
        );
        assert_eq!(
            MaturityMutationContext::new(
                MaturitySafetySection::Positive,
                "wrong-control-block",
                MutationLayer::WitnessProof,
                context.actual().clone()
            ),
            Err(MaturityMutantAssemblyRefusal::UnknownRow)
        );
    }

    #[test]
    fn observed_mutant_retains_its_source_and_branch() {
        for original in [archived(), node_free()] {
            let mut source = original.clone();
            source.bytes.push(0);
            source.branch = BranchContext::new([0x73; 32], 19).expect("distinct branch");
            let context = mutation_context(
                MaturitySafetySection::PredecessorConstructorFault,
                "wrong-control-block",
                MutationLayer::WitnessProof,
                "submitted serialization with trailing byte",
                &[],
                None,
            );
            let observed = MaturityContinuityMutant::observe(context, source.input())
                .expect("framing refusal");
            assert_eq!(observed.subject().source(), &source.origin);
            assert_eq!(observed.subject().branch(), source.branch);
            assert_ne!(observed.subject().branch(), original.branch);
        }
    }

    #[test]
    fn negative_record_refuses_successful_projection() {
        // The designated placeholder establishes neither authorization nor target acceptance.
        let source = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let context = mutation_context(
            MaturitySafetySection::SuccessorConstructorFault,
            "later-admissible-nonce-instead-of-first",
            MutationLayer::LinkedConstructorProgram,
            "honest public-chain control; placeholder signature",
            &[],
            None,
        );
        assert_eq!(
            MaturityContinuityMutant::observe(context, source.input()),
            Err(MaturityMutantAssemblyRefusal::ProjectionSucceeded)
        );
    }

    #[test]
    fn whole_sources_under_other_deployments_refuse_retained_context() {
        // Both public-chain controls have designated placeholders: Second has no signer material; neither establishes authorization or target acceptance.
        let second = coherent_source(MaturityDeployment::Second);
        let published = coherent_source(MaturityDeployment::PublishedSignerHeld);
        assert_eq!(
            second
                .bundle
                .deployment()
                .lead_bounds()
                .bounds()
                .window(Cycle::new(5))
                .expect("window"),
            (Cycle::new(8), Cycle::new(10))
        );
        assert_eq!(
            second.bundle.instances()[0].metadata().semantic,
            published.bundle.instances()[0].metadata().semantic
        );
        for (honest, retained, description) in [
            (
                &second,
                &published,
                "Second bytes and funding / PublishedSignerHeld retained context; placeholder",
            ),
            (
                &published,
                &second,
                "PublishedSignerHeld bytes and funding / Second retained context; placeholder",
            ),
        ] {
            let control = honest.project().expect("independent honest control");
            assert!(control.static_comparison().control_block().agrees());
            assert!(control.semantic_comparison().transition_agrees());
            let mut mutant = honest.clone();
            mutant.bundle = retained.bundle.clone();
            mutant.identity = retained.identity.clone();
            let context = mutation_context(
                MaturitySafetySection::PredecessorConstructorFault,
                "stale-constructor-from-another-bundle",
                MutationLayer::LinkedConstructorProgram,
                description,
                &[],
                None,
            );
            let observed = negative(&mutant, context.clone());
            assert!(matches!(
                observed.subject().failure(),
                Refusal::RetainedContext { .. }
            ));
            assert_eq!(observed.subject().failure().diagnostics(), None);
            // These are facts of the separately successful control, not diagnostics reached by the refused projection.
            let paired = ExperimentalSubject::observe((context, control, observed.into_subject()));
            assert_eq!(paired.standing(), SubjectStanding::Experimental);
            assert_ne!(
                honest.bundle.static_subtree().root(),
                retained.bundle.static_subtree().root()
            );
        }
    }

    #[test]
    fn cross_bundle_successors_refuse_output_with_valid_control() {
        // Public-chain controls use designated placeholders, coherent for the projector only; neither authorization nor target acceptance is established.
        let second = coherent_source(MaturityDeployment::Second);
        let published = coherent_source(MaturityDeployment::PublishedSignerHeld);
        for (source, other, description) in [
            (
                &second,
                &published,
                "Second predecessor / PublishedSignerHeld successor; output nonce and prefix rebuilt; placeholder",
            ),
            (
                &published,
                &second,
                "PublishedSignerHeld predecessor / Second successor; output nonce and prefix rebuilt; placeholder",
            ),
        ] {
            let semantic = source.planner_successor.encoded_metadata().semantic;
            let mutant = substituted_successor(source, semantic, &other.bundle);
            let context = mutation_context(
                MaturitySafetySection::SuccessorConstructorFault,
                "successor-under-another-static-subtree",
                MutationLayer::LinkedConstructorProgram,
                description,
                &[0, 1],
                Some(0),
            );
            let observed = negative(&mutant, context);
            assert_output_negative(observed.subject(), source);
        }
    }

    #[test]
    fn wrong_semantic_successors_refuse_output_with_valid_control() {
        // Public-chain placeholder subjects establish neither authorization nor target acceptance; the signature is not an axis checked here.
        let source = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let expected = source.planner_successor.encoded_metadata().semantic;
        let cases = [
            (
                "successor-from-wrong-semantic-metadata",
                StateMetadata {
                    omega: ProtocolAmount::new(expected.omega.get() + 1).expect("amount"),
                    ..expected
                },
            ),
            (
                "wrong-announcement-cycle",
                StateMetadata {
                    maturity: Maturity::Announced {
                        cycle: Cycle::new(9),
                    },
                    ..expected
                },
            ),
        ];
        for (row, semantic) in cases {
            let mutant = substituted_successor(&source, semantic, &source.bundle);
            let context = mutation_context(
                MaturitySafetySection::SuccessorConstructorFault,
                row,
                MutationLayer::SemanticFact,
                "PublishedSignerHeld wrong semantic output at common admissible nonce; request remains 10; placeholder",
                &[0, 1],
                Some(0),
            );
            let observed = negative(&mutant, context);
            assert_output_negative(observed.subject(), &source);
            let tx = TargetTransaction::decode(observed.subject().submitted()).expect("decode");
            assert_eq!(tx.witnesses()[0].stack()[2], 10_u64.to_be_bytes());
        }
    }

    fn later_predecessor(source: &Source) -> MaturityConstructorProjection {
        let retained = source.bundle.instances()[0].metadata();
        (retained.representation.get() + 1..4096)
            .find_map(|value| {
                fixed_projection(
                    &closure_target().expect("target"),
                    EncodedStateMetadata {
                        semantic: retained.semantic,
                        representation: StateRepresentationNonce::new(value),
                    },
                    &source.bundle,
                    TransactionSide::Input,
                )
                .ok()
            })
            .expect("later admissible predecessor")
    }

    #[test]
    fn admissible_predecessor_substitution_refuses_retained_context() {
        // The designated public-chain placeholder establishes neither authorization nor target acceptance.
        let mut source = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let later = later_predecessor(&source);
        source.bytes = changed_stack(&source, "admissible predecessor encoding only", |stack| {
            stack[4] = later.metadata_bytes().to_vec();
        })
        .bytes
        .into_subject();
        let context = mutation_context(
            MaturitySafetySection::PredecessorConstructorFault,
            "predecessor-metadata-reconstructs-another-program",
            MutationLayer::WitnessProof,
            "PublishedSignerHeld item 4 re-encoded at another admissible nonce; retained instance unchanged; placeholder",
            &[4],
            None,
        );
        let observed = negative(&source, context);
        assert!(matches!(
            observed.subject().failure(),
            Refusal::RetainedContext { .. }
        ));
        assert_eq!(observed.subject().failure().diagnostics(), None);
    }

    #[test]
    fn constructed_funded_program_refuses_retained_context() {
        // The designated public-chain placeholder establishes neither authorization nor target acceptance.
        let mut source = coherent_source(MaturityDeployment::PublishedSignerHeld);
        source.funded.program = later_predecessor(&source).output_program().to_vec();
        let context = mutation_context(
            MaturitySafetySection::PredecessorConstructorFault,
            "wrong-predecessor-program",
            MutationLayer::LinkedConstructorProgram,
            "PublishedSignerHeld supplied funding program replaced by another admissible instance; placeholder",
            &[],
            None,
        );
        let observed = negative(&source, context);
        assert!(matches!(
            observed.subject().failure(),
            Refusal::RetainedContext { .. }
        ));
        assert_eq!(observed.subject().failure().diagnostics(), None);
    }

    #[test]
    fn descriptor_comparison_exposes_projector_attribution_limit() {
        // This public-chain placeholder establishes neither authorization nor target acceptance. The independent descriptor is not an altered bundle offered to the projector: that subject has no public construction API.
        let source = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let target = closure_target().expect("target");
        let original = source.bundle.static_subtree();
        let rebuilt = StateStaticSubtree::new(
            &target,
            Some(tapscript::StateStaticNode::Leaf {
                identity: 1,
                leaf: original.leaves()[0].leaf.clone(),
            }),
        )
        .expect("independent descriptor");
        assert_ne!(original, &rebuilt);
        assert_eq!(original.root(), rebuilt.root());
        assert_eq!(
            state_bundle_continuity(original, &rebuilt),
            Err(StateLinkRefusal::StaticSubtreeDiscontinuity {
                predecessor: *original.root(),
                successor: *rebuilt.root()
            })
        );
        let constructed = CandidateStateConstructor::derive(
            &target,
            &source.bundle.instances()[0].metadata().semantic,
            &rebuilt,
            source.bundle.policy().internal_key(),
            source.bundle.policy().budget(),
            &OracleStateCurve,
        )
        .expect("real arithmetic");
        assert_eq!(constructed.output_program(), source.funded.program);
        assert_eq!(
            source.bundle.instances()[0]
                .constructor()
                .continuity(&constructed),
            Ok(())
        );
        let context = mutation_context(
            MaturitySafetySection::PredecessorConstructorFault,
            "wrong-static-subtree",
            MutationLayer::StaticConstructorSchema,
            "independent descriptor identity 1 beside retained identity 0; unchanged bytes; placeholder",
            &[],
            None,
        );
        let observed = ExperimentalSubject::observe((
            context,
            source.project().expect("unchanged submitted subject"),
            rebuilt,
        ));
        let (_, projection, _) = observed.subject();
        // The projector compares its single retained tree with itself; the equality cannot authenticate an independently described hidden subtree.
        assert_eq!(projection.static_comparison().descriptor_result(), &Ok(()));
        assert!(projection.static_comparison().static_root().agrees());
        assert_eq!(
            projection.successor_attribution(),
            MaturitySuccessorAttribution::CommitmentMismatchWithoutCauseAttribution
        );
        writeln!(std::io::stdout().lock(), "\nRUN-REPORT descriptor same-root=true independent-descriptor=refused-equal-roots projector=success single-retained-tree=true attribution=CommitmentMismatchWithoutCauseAttribution").expect("descriptor observation");
    }

    #[test]
    fn archived_source_projects() {
        let record = positive(archived());
        assert_eq!(
            (
                record.predecessor().nonce().get(),
                record.successor().nonce().get()
            ),
            (1, 0)
        );
        assert_eq!(record.submitted_bytes().len(), 1694);
        assert_eq!(record.transaction().encode().len(), 1694);
        assert_eq!(record.transaction().encode_without_witness().len(), 130);
        assert_eq!(record.transaction().weight(), 2084);
        let widths: Vec<_> = record.transaction().witnesses()[0]
            .stack()
            .iter()
            .map(Vec::len)
            .collect();
        assert_eq!(widths, [1, 4, 8, 32, 86, 1, 64, 1286, 65]);
        writeln!(
            std::io::stdout().lock(),
            "\nRUN-REPORT decoder stripped={} encoded={} weight={} widths={widths:?}",
            record.transaction().encode_without_witness().len(),
            record.transaction().encode().len(),
            record.transaction().weight()
        )
        .expect("decoder observation");
        report_observed("archived", &record);
    }

    #[test]
    fn accepted_variable_archived_bytes_project_with_complete_byte_identity() {
        let source = variable_archived();
        let record = positive(source);
        assert_eq!(record.schedule(), StateWitnessSchedule::VariableMetadata);
        assert_eq!(
            record.source(),
            &MaturityByteSource::ArchivedSubmission {
                run_address: MATURITY_VARIABLE_RUN_ADDRESS.to_owned()
            }
        );
        assert_eq!(record.submitted_bytes(), source.bytes.as_slice());
        assert_eq!(record.byte_identity(), &sha256(&source.bytes));
        assert!(record.static_comparison().static_root().agrees());
        assert!(record.static_comparison().control_block().agrees());
        assert!(record.semantic_comparison().transition_agrees());
        assert_eq!(
            record.semantic_comparison().reconstruction_agrees(),
            Some(true)
        );
        report_observed("accepted-variable-archived", &record);
    }

    #[test]
    fn historical_and_variable_archives_validate_one_continuity_report() {
        let historical = positive(archived());
        let variable = positive(variable_archived());
        let sources = [&historical, &variable];
        let provenance = MaturityExecutorProvenanceExpectation::NotStatedByTheOperator;
        let binding = derive_maturity_evidence_plan_with(
            provenance.clone(),
            MaturityConstructorMaterial::Absent(
                MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
            ),
        )
        .expect("independent target binding")
        .binding()
        .clone();
        let report =
            assemble_maturity_continuity_report(&sources, binding.clone(), provenance.clone())
                .expect("two archives agree on acceptance");
        let validated =
            validate_maturity_continuity_report(&report, &sources, &binding, &provenance)
                .expect("two-schedule report validation");
        let schedules: Vec<_> = report
            .entries()
            .iter()
            .map(MaturityContinuityReportEntry::schedule)
            .collect();
        assert_eq!(
            schedules,
            [
                StateWitnessSchedule::WholeMetadata,
                StateWitnessSchedule::VariableMetadata
            ]
        );
        assert_eq!(report.census().sources(), 2);
        let rendered = render_maturity_continuity_report(&validated);
        assert!(rendered.contains("schedule whole-metadata\n"));
        assert!(rendered.contains("schedule variable-metadata\n"));
    }

    #[test]
    fn node_free_source_projects() {
        let record = positive(node_free());
        assert_eq!(record.source(), &MaturityByteSource::NodeFreeSubmitReady);
        report_observed("node-free", &record);
    }

    #[test]
    fn variable_node_free_source_projects_with_rebuilt_metadata() {
        let source = variable_node_free();
        let record = positive(source);
        assert_eq!(record.schedule(), StateWitnessSchedule::VariableMetadata);
        assert_eq!(record.source(), &MaturityByteSource::NodeFreeSubmitReady);
        assert_eq!(
            record.transaction().witnesses()[0].stack()[4].len(),
            STATE_METADATA_VARIABLE_BYTES
        );
        assert!(record.static_comparison().static_root().agrees());
        assert!(record.static_comparison().control_block().agrees());
        assert!(record.semantic_comparison().transition_agrees());
        assert_eq!(
            record.semantic_comparison().reconstruction_agrees(),
            Some(true)
        );
        report_observed("variable-node-free", &record);
    }

    #[test]
    fn whole_item_under_variable_bundle_refuses_witness_width() {
        let mut source = node_free().clone();
        source.bundle = variable_node_free().bundle.clone();
        assert_eq!(
            source.project(),
            Err(Refusal::WitnessWidth {
                index: 4,
                expected: STATE_METADATA_VARIABLE_BYTES,
                actual: STATE_METADATA_BYTES,
            })
        );
    }

    #[test]
    fn variable_item_under_whole_bundle_refuses_witness_width() {
        let mut source = variable_node_free().clone();
        source.bundle = node_free().bundle.clone();
        assert_eq!(
            source.project(),
            Err(Refusal::WitnessWidth {
                index: 4,
                expected: STATE_METADATA_BYTES,
                actual: STATE_METADATA_VARIABLE_BYTES,
            })
        );
    }

    #[test]
    fn carried_width_agrees_with_legalization_for_both_schedules() {
        let target = closure_target().expect("target");
        let bound = target
            .definition()
            .resources()
            .policy()
            .bounds()
            .get(&ResourceDimension::InitialWitnessItemBytes)
            .copied()
            .expect("initial item bound");
        for source in [node_free(), variable_node_free()] {
            let schedule = source.bundle.record().schedule();
            let legalization =
                legalize_state_witness_schedule(schedule, &STATE_METADATA_LAYOUT, bound)
                    .expect("reviewed legalization");
            let observed = TargetTransaction::decode(&source.bytes)
                .expect("transaction")
                .witnesses()[0]
                .stack()[4]
                .len();
            assert_eq!(carried_width(&legalization), declared_widths(schedule)[4]);
            assert_eq!(observed, carried_width(&legalization));
        }
    }

    #[test]
    fn variable_rebuild_matches_retained_constructor_and_round_trips() {
        let source = variable_node_free();
        let transaction = TargetTransaction::decode(&source.bytes).expect("transaction");
        let carried =
            fixed_bytes::<STATE_METADATA_VARIABLE_BYTES>(&transaction.witnesses()[0].stack()[4]);
        let rebuilt = rebuild_state_metadata(&carried);
        assert_eq!(
            rebuilt.as_slice(),
            source.bundle.instances()[0].constructor().metadata_bytes()
        );
        assert_eq!(
            realization::state_metadata_variable_region(&rebuilt),
            carried
        );
        assert_eq!(
            decode_state_metadata(&rebuilt).expect("strict metadata"),
            *source.bundle.instances()[0].metadata()
        );
    }

    #[test]
    fn invalid_variable_maturity_tag_refuses_metadata_decode() {
        let source = variable_node_free();
        let mutant = changed_stack(source, "variable metadata maturity tag", |stack| {
            let offset = STATE_METADATA_LAYOUT[7].range.start
                - realization::STATE_METADATA_VARIABLE_RANGE.start;
            stack[4][offset] = u8::MAX;
        });
        assert_eq!(
            mutant.project(source),
            Err(Refusal::MetadataDecode(Box::new(
                StateMetadataRefusal::UnknownMaturityDiscriminant
            )))
        );
    }

    #[test]
    fn variable_leaf_and_control_bindings_refuse_mutations() {
        let source = variable_node_free();
        let record = positive(source);
        assert!(record.diagnostics().leaf_script().expect("leaf").agrees());
        assert!(
            record
                .diagnostics()
                .control_block()
                .expect("control")
                .agrees()
        );
        let leaf = changed_stack(source, "variable announcement leaf", |stack| {
            stack[7][0] ^= 1;
        });
        assert!(matches!(leaf.project(source), Err(Refusal::LeafScript(_))));
        let control = changed_stack(source, "variable announcement control", |stack| {
            stack[8][0] ^= 1;
        });
        assert!(matches!(
            control.project(source),
            Err(Refusal::ControlBlock(_))
        ));
    }

    #[test]
    fn submitted_bytes_round_trip_exactly() {
        for source in [archived(), node_free()] {
            let transaction = TargetTransaction::decode(&source.bytes).expect("decode");
            assert_eq!(transaction.encode(), source.bytes);
            assert_ne!(transaction.encode_without_witness(), source.bytes);
        }
    }

    #[test]
    fn decode_refusal_is_preserved() {
        let source = archived();
        let mut bytes = source.bytes.clone();
        bytes.push(0);
        let inner = TargetTransaction::decode(&bytes).expect_err("trailing bytes");
        assert_eq!(
            ExperimentalBytes::new(bytes, "transaction framing").project(source),
            Err(Refusal::Decode(Box::new(inner)))
        );
    }

    #[test]
    fn transaction_shape_is_refused() {
        let source = archived();
        let transaction = TargetTransaction::decode(&source.bytes).expect("decode");
        let two_inputs = TargetTransaction::new(
            transaction.version(),
            vec![transaction.inputs()[0].clone(); 2],
            transaction.outputs().to_vec(),
            transaction.lock_time(),
            vec![transaction.witnesses()[0].clone(); 2],
        )
        .expect("two inputs");
        assert_eq!(
            ExperimentalBytes::new(two_inputs.encode(), "input census").project(source),
            Err(Refusal::InputCount { actual: 2 })
        );
        let two_outputs = TargetTransaction::new(
            transaction.version(),
            transaction.inputs().to_vec(),
            vec![transaction.outputs()[0].clone(); 2],
            transaction.lock_time(),
            transaction.witnesses().to_vec(),
        )
        .expect("two outputs");
        assert_eq!(
            ExperimentalBytes::new(two_outputs.encode(), "output census").project(source),
            Err(Refusal::OutputCount { actual: 2 })
        );
        let truncated = changed_stack(source, "witness item census", |stack| {
            stack.pop();
        });
        assert_eq!(
            truncated.project(source),
            Err(Refusal::WitnessItemCount { actual: 8 })
        );
    }

    #[test]
    fn all_seven_witness_widths_are_checked() {
        let source = archived();
        for (index, expected) in declared_widths(source.bundle.record().schedule())
            .into_iter()
            .enumerate()
        {
            let mutant = changed_stack(source, "fixed-width witness item", |stack| {
                stack[index].pop();
            });
            assert_eq!(
                mutant.project(source),
                Err(Refusal::WitnessWidth {
                    index,
                    expected,
                    actual: expected - 1
                })
            );
        }
    }

    #[test]
    fn wrong_funded_outpoint_is_refused() {
        let mut source = archived().clone();
        let decoded = source.funded.outpoint;
        source.funded.outpoint =
            Outpoint::new(Txid::from_internal([0x91; 32]), 2).expect("other outpoint");
        assert_eq!(
            source.project(),
            Err(Refusal::SpentOutpoint {
                decoded,
                funded: source.funded.outpoint
            })
        );
    }

    #[test]
    fn metadata_codec_refusal_is_preserved() {
        let source = archived();
        let mutant = changed_stack(source, "predecessor metadata domain", |stack| {
            stack[4][0] ^= 1;
        });
        assert_eq!(
            mutant.project(source),
            Err(Refusal::MetadataDecode(Box::new(
                StateMetadataRefusal::WrongDomain
            )))
        );
    }

    #[test]
    fn retained_metadata_disagreement_is_refused_first() {
        let source = archived();
        let mutant = changed_stack(source, "predecessor representation nonce", |stack| {
            stack[4][77] ^= 1;
            stack[3][0] ^= 1;
        });
        let Refusal::RetainedContext {
            witnessed,
            retained,
            ..
        } = mutant
            .project(source)
            .expect_err("retained disagreement first")
        else {
            panic!("retained context")
        };
        assert_ne!(Some(witnessed.as_ref()), retained.as_deref());
        let other = crate::maturity_closure::maturity_sources(
            crate::maturity_closure::MaturityDeployment::Second,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("sources")
        .link(&OracleStateCurve)
        .expect("other bundle");
        let mut input = source.input();
        input.bundle = &other;
        assert!(matches!(
            project_maturity_continuity(input),
            Err(Refusal::RetainedContext { .. })
        ));
    }

    #[test]
    fn retained_funded_program_disagreement_is_refused_first() {
        let mut source = archived().clone();
        source.funded.program[2] ^= 1;
        let Refusal::RetainedContext {
            funded_program,
            retained_program,
            ..
        } = source.project().expect_err("retained funding")
        else {
            panic!("retained context")
        };
        assert_eq!(funded_program, source.funded.program);
        assert_ne!(Some(funded_program), retained_program);
    }

    #[test]
    fn predecessor_reconstruction_refusal_is_preserved() {
        // The public entry refuses this altered tuple earlier. This tests the reconstruction stage itself under real arithmetic.
        let source = archived();
        let mut metadata = *source.bundle.instances()[0].metadata();
        metadata.representation = StateRepresentationNonce::ZERO;
        let refusal = reconstruct(
            &closure_target().expect("target"),
            metadata,
            &source.bundle,
            TransactionSide::Input,
        )
        .expect_err("nonce zero branch side");
        assert_eq!(
            refusal,
            Refusal::PredecessorReconstruction(Box::new(
                StateConstructorRefusal::CanonicalBranchSideNotSatisfied
            ))
        );
    }

    #[test]
    fn predecessor_program_comparison_names_both_operands() {
        // A stage test: the public entry detects an altered funded program at retained-context comparison first.
        assert_eq!(
            predecessor_program(&[1, 2], &[3, 4]),
            Err(Refusal::PredecessorProgram {
                reconstructed: vec![1, 2],
                funded: vec![3, 4]
            })
        );
    }

    #[test]
    fn predecessor_leastness_retains_the_rejection_prefix() {
        let source = archived();
        let record = source.project().expect("project");
        assert_eq!(
            record.predecessor_leastness().evidence().rejected,
            [(
                StateRepresentationNonce::ZERO,
                StateConstructorRefusal::CanonicalBranchSideNotSatisfied
            )]
        );
        assert_eq!(record.successor_leastness().evidence().rejected.len(), 0);
        let target = closure_target().expect("target");
        let budget = tapscript::StateNonceBudget::new(1).expect("one attempt");
        for side in [TransactionSide::Input, TransactionSide::Output] {
            let error = leastness(&target, record.predecessor(), &source.bundle, side, budget)
                .expect_err("one short");
            let expected = Box::new(StateConstructorRefusal::RepresentationSearchExhausted);
            assert_eq!(
                error,
                match side {
                    TransactionSide::Input => Refusal::PredecessorSearch(expected),
                    TransactionSide::Output => Refusal::SuccessorSearch(expected),
                }
            );
        }
    }

    #[test]
    fn static_root_mutation_stops_before_transition() {
        let source = archived();
        let mutant = changed_stack(source, "static-root witness and invalid request", |stack| {
            stack[3][0] ^= 1;
            stack[2] = 0_u64.to_be_bytes().to_vec();
        });
        let refusal = mutant.project(source).expect_err("root first");
        assert!(matches!(refusal, Refusal::StaticRoot(_)));
        assert_eq!(refusal.diagnostics().expect("diagnostics").semantic(), None);
    }

    #[test]
    fn replaced_leaf_script_is_refused() {
        let source = archived();
        let mutant = changed_stack(source, "executing leaf script", |stack| {
            stack[7][0] ^= 1;
        });
        let refusal = mutant.project(source).expect_err("leaf");
        assert!(matches!(refusal, Refusal::LeafScript(_)));
        let diagnostics = refusal.diagnostics().expect("diagnostics");
        assert!(diagnostics.static_root().expect("root").agrees());
        assert!(!diagnostics.leaf_script().expect("leaf").agrees());
        assert_eq!(diagnostics.semantic(), None);
    }

    #[test]
    fn replaced_control_block_is_refused() {
        // The public-chain placeholder establishes neither authorization nor target acceptance.
        let owned = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let source = &owned;
        for index in [0, 33] {
            let mutant = changed_stack(source, "predecessor control parity or sibling", |stack| {
                stack[8][index] ^= 1;
            });
            let refusal = mutant.project(source).expect_err("control");
            assert!(matches!(refusal, Refusal::ControlBlock(_)));
            let diagnostics = refusal.diagnostics().expect("diagnostics");
            assert!(diagnostics.leaf_script().expect("leaf").agrees());
            assert!(!diagnostics.control_block().expect("control").agrees());
            assert_eq!(diagnostics.semantic(), None);
            let mut substituted = source.clone();
            substituted.bytes = mutant.bytes.into_subject();
            let context = mutation_context(
                MaturitySafetySection::PredecessorConstructorFault,
                "wrong-control-block",
                MutationLayer::WitnessProof,
                if index == 0 {
                    "PublishedSignerHeld item 8 parity bit changed; placeholder"
                } else {
                    "PublishedSignerHeld item 8 outer sibling byte changed; placeholder"
                },
                &[8],
                None,
            );
            let observed = negative(&substituted, context);
            assert_eq!(observed.subject().failure(), &refusal);
        }
        // The foreign path comes from another public-chain control; its placeholder likewise establishes neither authorization nor target acceptance.
        let other = coherent_source(MaturityDeployment::Second);
        let control = other.project().expect("independent Second control");
        assert!(control.controls().observed().agrees());
        let mutant = changed_stack(source, "foreign coherent control block", |stack| {
            stack[8] = control.controls().observed().witnessed().to_vec();
        });
        let mut substituted = source.clone();
        substituted.bytes = mutant.bytes.into_subject();
        let context = mutation_context(
            MaturitySafetySection::AbiLinkerFault,
            "control-block-from-another-program",
            MutationLayer::WitnessProof,
            "PublishedSignerHeld item 8 replaced by Second's honest control; placeholder",
            &[8],
            None,
        );
        let observed = negative(&substituted, context);
        assert!(matches!(
            observed.subject().failure(),
            Refusal::ControlBlock(_)
        ));
        let diagnostics = observed
            .subject()
            .failure()
            .diagnostics()
            .expect("reached comparisons");
        assert!(diagnostics.static_root().expect("root").agrees());
        assert!(diagnostics.leaf_script().expect("leaf").agrees());
        assert!(!diagnostics.control_block().expect("foreign path").agrees());
        assert_eq!(diagnostics.semantic(), None);
    }

    #[test]
    fn replaced_predecessor_prefix_is_refused() {
        // The public-chain placeholder establishes neither authorization nor target acceptance.
        let owned = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let source = &owned;
        let mutant = changed_stack(source, "predecessor key prefix", |stack| {
            stack[5][0] ^= 1;
        });
        let refusal = mutant.project(source).expect_err("prefix");
        assert!(matches!(refusal, Refusal::PredecessorPrefix(_)));
        assert!(
            !refusal
                .diagnostics()
                .expect("diagnostics")
                .predecessor_prefix()
                .expect("prefix")
                .agrees()
        );
        let mut substituted = source.clone();
        substituted.bytes = mutant.bytes.into_subject();
        let context = mutation_context(
            MaturitySafetySection::PredecessorConstructorFault,
            "wrong-control-block",
            MutationLayer::WitnessProof,
            "PublishedSignerHeld item 5 parity prefix changed; item 8 remains honest; placeholder",
            &[5],
            None,
        );
        let observed = negative(&substituted, context);
        assert_eq!(observed.subject().failure(), &refusal);
    }

    #[test]
    fn transition_refusal_is_preserved() {
        let source = archived();
        for (cycle, expected) in [
            (8_u64, MaturityTransitionRefusal::AnnouncementBelowMinimum),
            (12, MaturityTransitionRefusal::AnnouncementAboveMaximum),
        ] {
            let mutant = changed_stack(source, "requested cycle", |stack| {
                stack[2] = cycle.to_be_bytes().to_vec();
            });
            assert_eq!(
                mutant.project(source),
                Err(Refusal::Transition(Box::new(expected)))
            );
        }
    }

    #[test]
    fn successor_reconstruction_refusal_is_preserved() {
        let source = archived();
        let target = closure_target().expect("target");
        let semantic = source.planner_successor.encoded_metadata().semantic;
        let (nonce, inner) = (0..256)
            .find_map(|value| {
                let encoded = EncodedStateMetadata {
                    semantic,
                    representation: StateRepresentationNonce::new(value),
                };
                state_output_program_at_nonce(
                    &target,
                    &encoded,
                    source.bundle.static_subtree(),
                    source.bundle.policy().internal_key(),
                    &OracleStateCurve,
                )
                .err()
                .map(|error| (value, error))
            })
            .expect("a refused fixed nonce");
        let mutant = changed_stack(source, "successor representation nonce", |stack| {
            stack[1] = nonce.to_be_bytes().to_vec();
        });
        assert_eq!(
            mutant.project(source),
            Err(Refusal::SuccessorReconstruction(Box::new(inner)))
        );
    }

    #[test]
    fn replaced_output_program_is_refused() {
        let source = archived();
        let mut program = source.planner_successor.output_program();
        program[2] ^= 1;
        let mutant = changed_program(source, program.clone());
        let refusal = mutant.project(source).expect_err("program");
        assert!(matches!(refusal, Refusal::OutputProgram(_)));
        let diagnostics = refusal.diagnostics().expect("diagnostics");
        assert_eq!(
            diagnostics
                .output_program()
                .expect("comparison")
                .witnessed(),
            program
        );
        assert_eq!(
            diagnostics
                .output_program()
                .expect("comparison")
                .reconstructed(),
            source.planner_successor.output_program()
        );
        assert!(
            diagnostics
                .semantic()
                .expect("semantic diagnostic")
                .transition_agrees()
        );
    }

    #[test]
    fn replaced_successor_prefix_is_refused() {
        // The public-chain placeholder establishes neither authorization nor target acceptance.
        let owned = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let source = &owned;
        let mutant = changed_stack(source, "successor key prefix", |stack| {
            stack[0][0] ^= 1;
        });
        let refusal = mutant.project(source).expect_err("prefix");
        assert!(matches!(refusal, Refusal::SuccessorPrefix(_)));
        let diagnostics = refusal.diagnostics().expect("diagnostics");
        assert!(diagnostics.output_program().expect("program").agrees());
        assert!(!diagnostics.successor_prefix().expect("prefix").agrees());
        assert!(
            diagnostics
                .semantic()
                .expect("semantic diagnostic")
                .transition_agrees()
        );
        let mut substituted = source.clone();
        substituted.bytes = mutant.bytes.into_subject();
        let context = mutation_context(
            MaturitySafetySection::SuccessorConstructorFault,
            "wrong-parity",
            MutationLayer::WitnessProof,
            "PublishedSignerHeld item 0 prefix changed; placeholder",
            &[0],
            None,
        );
        let observed = negative(&substituted, context);
        assert_eq!(observed.subject().failure(), &refusal);
    }

    #[test]
    fn later_admissible_successor_nonce_is_not_target_invalid() {
        // The public-chain placeholder establishes neither authorization nor target acceptance.
        let owned = coherent_source(MaturityDeployment::PublishedSignerHeld);
        let source = &owned;
        let target = closure_target().expect("target");
        let semantic = source.planner_successor.encoded_metadata().semantic;
        let later = (source.planner_successor.nonce().get() + 1..256)
            .find_map(|value| {
                fixed_projection(
                    &target,
                    EncodedStateMetadata {
                        semantic,
                        representation: StateRepresentationNonce::new(value),
                    },
                    &source.bundle,
                    TransactionSide::Output,
                )
                .ok()
            })
            .expect("later admissible nonce");
        let replaced = changed_program(source, later.output_program().to_vec());
        let mut context = source.clone();
        context.bytes = replaced.bytes.into_subject();
        let mutant = changed_stack(&context, "successor program, nonce and prefix", |stack| {
            stack[1] = later.nonce().get().to_be_bytes().to_vec();
            stack[0] = vec![prefix(later.parity())];
        });
        let record = mutant.project(source).expect("later nonce projects");
        assert!(!record.successor_leastness().is_host_least());
        assert_eq!(
            record.successor_leastness().reconstructed_facts_match(),
            None
        );
        assert_eq!(record.successor().nonce(), later.nonce());
        assert_eq!(record.successor().control_recipe(), later.control_recipe());
        assert_eq!(
            record.semantic_comparison().reconstruction_agrees(),
            Some(true)
        );
        assert!(!record.checked_successor().is_host_least());
        assert_eq!(
            record.checked_successor().residual().text(),
            StateNonceEvidence::RESIDUAL
        );
        let context = mutation_context(
            MaturitySafetySection::SuccessorConstructorFault,
            "later-admissible-nonce-instead-of-first",
            MutationLayer::LinkedConstructorProgram,
            "PublishedSignerHeld later admissible output program, nonce and parity rebuilt; placeholder",
            &[0, 1],
            Some(0),
        );
        let mut supplied = source.clone();
        supplied.bytes = mutant.bytes.into_subject();
        assert_eq!(
            MaturityContinuityMutant::observe(context.clone(), supplied.input()),
            Err(MaturityMutantAssemblyRefusal::ProjectionSucceeded)
        );
        let observed = ExperimentalSubject::observe((context, record));
        writeln!(std::io::stdout().lock(), "\nRUN-REPORT accepted-experiment row={} boundary={:?} declared_layer={:?} carrier={:?} host_least=false residual={:?}", observed.subject().0.row().name(), observed.subject().0.declared_boundary(), observed.subject().0.declared_layer(), observed.subject().0.actual(), observed.subject().1.checked_successor().residual()).expect("later nonce observation");
    }

    #[test]
    fn static_root_failure_retains_operands_without_semantic_verdict() {
        let source = archived();
        let mutant = changed_stack(source, "static-root witness item only", |stack| {
            stack[3][0] ^= 1;
        });
        let Refusal::StaticRoot(diagnostics) = mutant.project(source).expect_err("root") else {
            panic!("static root")
        };
        let comparison = diagnostics.static_root().expect("root operands");
        assert_ne!(comparison.witnessed(), comparison.reconstructed());
        assert_eq!(
            comparison.reconstructed(),
            source.bundle.static_subtree().root()
        );
        assert_eq!(diagnostics.semantic(), None);
        assert_eq!(diagnostics.leaf_script(), None);
    }

    fn common_nonce_control(
        source: &Source,
        request: Cycle,
    ) -> (Source, StateMetadata, Vec<u8>, StateRepresentationNonce) {
        let target = closure_target().expect("target");
        let predecessor = source.bundle.instances()[0].metadata().semantic;
        let bounds = source.bundle.deployment().lead_bounds().bounds();
        let expected = announce_maturity(&predecessor, request, bounds).expect("admissible cycle");
        let (original, program) = (0..256)
            .find_map(|value| {
                let nonce = StateRepresentationNonce::new(value);
                let original = fixed_projection(
                    &target,
                    EncodedStateMetadata {
                        semantic: source.planner_successor.encoded_metadata().semantic,
                        representation: nonce,
                    },
                    &source.bundle,
                    TransactionSide::Output,
                )
                .ok()?;
                let encoded = EncodedStateMetadata {
                    semantic: expected,
                    representation: nonce,
                };
                let program = state_output_program_at_nonce(
                    &target,
                    &encoded,
                    source.bundle.static_subtree(),
                    source.bundle.policy().internal_key(),
                    &OracleStateCurve,
                )
                .ok()?;
                Some((original, program))
            })
            .expect("common admissible nonce for the experimental control and its request mutant");
        // The archived nonce refuses both alternate requests before output binding. Rebuild an experimental control first, then change only its request item.
        let mut control_source = source.clone();
        control_source.bytes = changed_program(source, original.output_program().to_vec())
            .bytes
            .into_subject();
        let control = changed_stack(
            &control_source,
            "experimental control: successor program, nonce and prefix",
            |stack| {
                stack[1] = original.nonce().get().to_be_bytes().to_vec();
                stack[0] = vec![prefix(original.parity())];
            },
        );
        control
            .project(source)
            .expect("coherent experimental control");
        control_source.bytes = control.bytes.into_subject();
        (control_source, expected, program, original.nonce())
    }

    #[test]
    fn request_mutation_retains_expected_successor_on_output_refusal() {
        let source = archived();
        let request = Cycle::new(9);
        let (control, expected, program, nonce) = common_nonce_control(source, request);
        let mutant = changed_stack(&control, "requested-cycle witness item only", |stack| {
            stack[2] = request.get().to_be_bytes().to_vec();
        });
        let Refusal::OutputProgram(diagnostics) =
            mutant.project(source).expect_err("unchanged output")
        else {
            panic!("output program")
        };
        let semantic = diagnostics.semantic().expect("transition diagnostic");
        assert_eq!(semantic.expected_successor(), &expected);
        assert_eq!(semantic.requested_cycle(), request);
        assert!(semantic.transition_agrees());
        assert_eq!(semantic.reconstruction_agrees(), Some(true));
        let comparison = diagnostics.output_program().expect("program operands");
        assert_eq!(comparison.reconstructed(), program);
        assert_eq!(
            comparison.witnessed(),
            TargetTransaction::decode(&control.bytes)
                .expect("control")
                .outputs()[0]
                .program()
        );
        assert!(!comparison.agrees());
        assert!(diagnostics.static_root().expect("root").agrees());
        assert!(diagnostics.control_block().expect("control").agrees());
        writeln!(std::io::stdout().lock(), "\nRUN-REPORT mutant=request-item baseline_nonce={} baseline_request=10 request={} refusal=OutputProgram transition=true static_root=true control=true", nonce.get(), request.get()).expect("test observation output");
    }

    #[test]
    fn readers_preserve_context_and_outstanding_acceptance() {
        let source = archived();
        let record = source.project().expect("project");
        assert_eq!(record.source(), &source.origin);
        assert_eq!(record.branch(), source.branch);
        assert_eq!(record.identity(), &source.identity);
        assert_eq!(record.bundle(), &source.bundle);
        assert_eq!(
            record.observation_class(),
            MaturityObservationClass::IntendedTransition
        );
        assert_eq!(
            record.quantifier().admitted(),
            [
                MaturityObservationClass::IntendedTransition,
                MaturityObservationClass::ModeledUnintendedEnvironmentTransition
            ]
        );
        assert_eq!(
            record.quantifier().excluded(),
            MaturityObservationClass::ModelFalsifyingWithNoPreimage
        );
        assert_eq!(
            record.acceptance_obligation(),
            MaturityAcceptanceObligation::Outstanding {
                routes: [
                    MaturityAcceptanceRoute::RelayWitnessRestructure,
                    MaturityAcceptanceRoute::BlockLayerSubmissionSubject
                ]
            }
        );
        assert_eq!(
            record.successor_attribution(),
            MaturitySuccessorAttribution::CommitmentMismatchWithoutCauseAttribution
        );
        let mut owned_copy = record.funded().clone();
        owned_copy.program.clear();
        assert_eq!(record.funded(), &source.funded);
        let mut signature = record.transaction().witnesses()[0].stack()[6].clone();
        signature[0] ^= 1;
        let mutant = changed_stack(
            source,
            "operator signature only; no verification",
            |stack| {
                stack[6] = signature;
            },
        );
        let projected = mutant.project(source).expect("signature is not verified");
        assert_ne!(projected.byte_identity(), record.byte_identity());
        assert_eq!(projected.predecessor(), record.predecessor());
        assert_eq!(projected.successor(), record.successor());
    }

    fn reachability(refusal: &Refusal) -> &'static str {
        match refusal {
            Refusal::PrefixEvidence { .. } => {
                "prefix_disagreement_is_refused (private operand corruption; immutable production operands are rebuilt from the same validated inputs)"
            }
            Refusal::TweakEvidence { .. } => {
                "tweak_disagreement_is_refused (private operand corruption; immutable production projections already reconstruct these inputs)"
            }
            Refusal::ControlEvidence { .. } => {
                "control_relations_and_disagreement_are_measured (private operand corruption; immutable production recipes and witnessed bytes already passed binding)"
            }
            Refusal::Decode(_) => "decode_refusal_is_preserved",
            Refusal::InputCount { .. }
            | Refusal::OutputCount { .. }
            | Refusal::WitnessItemCount { .. } => "transaction_shape_is_refused",
            Refusal::WitnessWidth { .. } => "all_seven_witness_widths_are_checked",
            Refusal::WitnessLowering(_) => {
                "unreachable: the reviewed target's initial-item bound admits both retained schedules"
            }
            Refusal::SpentOutpoint { .. } => "wrong_funded_outpoint_is_refused",
            Refusal::MetadataDecode(_) => "metadata_codec_refusal_is_preserved",
            Refusal::RetainedContext { .. } => {
                "retained_metadata_disagreement_is_refused_first; retained_funded_program_disagreement_is_refused_first"
            }
            Refusal::PredecessorReconstruction(_) => {
                "predecessor_reconstruction_refusal_is_preserved (private reconstruction stage; retained-context gate rejects the byte mutant first)"
            }
            Refusal::PredecessorProgram { .. } => {
                "predecessor_program_comparison_names_both_operands (private binding stage; retained-context gate rejects altered funding first)"
            }
            Refusal::PredecessorSearch(_) | Refusal::SuccessorSearch(_) => {
                "predecessor_leastness_retains_the_rejection_prefix (private search stage with a shorter budget)"
            }
            Refusal::StaticRoot(_) => {
                "static_root_failure_retains_operands_without_semantic_verdict"
            }
            Refusal::Target(_) => {
                "unreachable: the reviewed target is fixed and no caller input reaches its validation"
            }
            Refusal::LeafReconstruction(_) => {
                "unreachable: every publicly linked maturity bundle contains its announcement program"
            }
            Refusal::LeafScript(_) => "replaced_leaf_script_is_refused",
            Refusal::ControlRecipe(_) => {
                "unreachable: fixed-nonce reconstruction validates static paths below the control-depth bound before appending the metadata sibling"
            }
            Refusal::ControlBlock(_) => "replaced_control_block_is_refused",
            Refusal::PredecessorPrefix(_) => "replaced_predecessor_prefix_is_refused",
            Refusal::Transition(_) => "transition_refusal_is_preserved",
            Refusal::SuccessorReconstruction(_) => "successor_reconstruction_refusal_is_preserved",
            Refusal::OutputProgram(_) => {
                "replaced_output_program_is_refused; request_mutation_retains_expected_successor_on_output_refusal"
            }
            Refusal::SuccessorPrefix(_) => "replaced_successor_prefix_is_refused",
            Refusal::AcceptanceClaim(_) => "a_claim_under_another_schedule_refuses",
        }
    }

    #[test]
    fn refusal_root_walk_is_exhaustive() {
        // Constructed members check vocabulary coverage; only the named tests above establish runtime reachability.
        let source = archived();
        let constructor = Box::new(StateConstructorRefusal::RepresentationSearchExhausted);
        let diagnostics = Box::new(MaturityProjectionDiagnostics::empty());
        let refusals = [
            Refusal::PrefixEvidence {
                side: TransactionSide::Input,
                comparison: MaturityEvidenceMismatch::NonceOrder,
            },
            Refusal::TweakEvidence {
                side: TransactionSide::Output,
                comparison: MaturityEvidenceMismatch::Digest,
            },
            Refusal::ControlEvidence {
                side: TransactionSide::Input,
                comparison: MaturityEvidenceMismatch::Recipe,
            },
            Refusal::Decode(Box::new(
                TargetTransaction::decode(&[]).expect_err("empty bytes"),
            )),
            Refusal::InputCount { actual: 2 },
            Refusal::OutputCount { actual: 2 },
            Refusal::WitnessItemCount { actual: 8 },
            Refusal::WitnessWidth {
                index: 0,
                expected: 1,
                actual: 0,
            },
            Refusal::WitnessLowering(Box::new(
                StateWitnessLoweringRefusal::VariableRowsNotContiguous,
            )),
            Refusal::SpentOutpoint {
                decoded: source.funded.outpoint,
                funded: source.funded.outpoint,
            },
            Refusal::MetadataDecode(Box::new(StateMetadataRefusal::WrongDomain)),
            Refusal::RetainedContext {
                witnessed: Box::new(*source.bundle.instances()[0].metadata()),
                retained: None,
                funded_program: source.funded.program.clone(),
                retained_program: None,
                identities: Box::new((source.identity.clone(), source.identity.clone())),
            },
            Refusal::PredecessorReconstruction(constructor.clone()),
            Refusal::PredecessorProgram {
                reconstructed: vec![1],
                funded: vec![2],
            },
            Refusal::PredecessorSearch(constructor.clone()),
            Refusal::StaticRoot(diagnostics.clone()),
            Refusal::Target(Box::new(MaturityClosureRefusal::ReviewedTargetInvalid)),
            Refusal::LeafReconstruction(Box::new(
                MaturityClosureRefusal::AnnouncementProgramAbsent,
            )),
            Refusal::LeafScript(diagnostics.clone()),
            Refusal::ControlRecipe(constructor.clone()),
            Refusal::ControlBlock(diagnostics.clone()),
            Refusal::PredecessorPrefix(diagnostics.clone()),
            Refusal::Transition(Box::new(
                MaturityTransitionRefusal::AnnouncementBelowMinimum,
            )),
            Refusal::SuccessorReconstruction(constructor.clone()),
            Refusal::OutputProgram(diagnostics.clone()),
            Refusal::SuccessorPrefix(diagnostics),
            Refusal::SuccessorSearch(constructor),
            Refusal::AcceptanceClaim(Box::new(MaturityAcceptanceClaimMismatch::Schedule {
                claimed: StateWitnessSchedule::VariableMetadata,
                retained: StateWitnessSchedule::WholeMetadata,
            })),
        ];
        assert_eq!(refusals.len(), 28);
        for refusal in refusals {
            assert_ne!(reachability(&refusal).len(), 0);
        }
    }
}
