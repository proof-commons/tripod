//! Constructor projections from exact submitted maturity-announcement bytes.
//!
//! The input outpoint binds supplied funding evidence; the transaction does not contain the spent asset, amount or program. The witnessed predecessor encoding must agree with the retained recipe before either constructor is reconstructed. The successor is the realization's transition over that decoded predecessor and witnessed request, tested against the observed output commitment.
//!
//! Static byte bindings, whole retained-descriptor equality and semantic comparisons are separate results. Reusing one retained tree on both sides states the descriptor equality's premise; a matching root alone establishes neither descriptor identity nor hidden-subtree contents. The successor field comparison checks reconstruction consistency, not an independently observed semantic tuple. A different successor commitment is refused without attributing its cause to a semantic change or another static tree.
//!
//! Fixed-nonce reconstruction and host leastness are separate evidence. These projections verify no signature, authenticate no branch freshness and establish no accepted readback. Acceptance remains an outstanding obligation, and experimental byte mutations answer no matrix row.

use linker::{
    CandidateDeploymentIdentity, CandidateLinkedMaturityBundle, StateLinkRefusal,
    state_bundle_continuity,
};
use realization::{
    AnnouncementLeadBounds, Cycle, EncodedStateMetadata, Maturity, MaturityTransitionRefusal,
    StateField, StateMetadata, StateMetadataRefusal, StateRepresentationNonce, TransactionSide,
    announce_maturity, decode_state_metadata, encode_state_metadata,
};
use tapscript::{
    CandidateStateConstructor, StateConstructorRefusal, StateControlRecipe, StateCurveCapability,
    StateFieldCommitment, StateLeafRole, StateNonceEvidence, StateStaticSubtree, StateTweakOutcome,
    state_metadata_leaf_program, state_output_program_at_nonce,
};
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::constructor::tagged::sha256;
use transaction::bytes::{AssetId, Outpoint, TargetTransaction};
use transaction::error::TransactionRefusal;
use transaction::operator_right::BranchContext;
use transaction::taproot::{branch_hash, leaf_hash, tagged_hash};

use crate::maturity_closure::{
    MaturityClosureRefusal, OracleStateCurve, closure_target, linked_announcement_bytes,
};
use crate::maturity_evidence::{
    MaturityEvidenceCensus, MaturityGuaranteeQuantifier, MaturityObservationClass,
};
use crate::maturity_native::{MaturityAcceptanceObligation, MaturityAcceptanceRoute};

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

    /// Bytes read from the submitted witness or output.
    #[must_use]
    pub fn witnessed(&self) -> &[u8] {
        &self.witnessed
    }

    /// Bytes reconstructed from the retained recipe and decoded metadata.
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

/// The first failed projection check, preserving inner roots and the operands already compared.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MaturityContinuityRefusal {
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

const WIDTHS: [usize; 7] = [1, 4, 8, 32, 86, 1, 64];

fn witness(transaction: &TargetTransaction) -> ProjectionResult<&[Vec<u8>]> {
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
    for (index, (item, expected)) in stack.iter().zip(WIDTHS).enumerate() {
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

fn fixed_bytes<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut result = [0; N];
    result.copy_from_slice(bytes);
    result
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

const fn prefix(odd: bool) -> u8 {
    if odd { 0x03 } else { 0x02 }
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

/// Project both constructor sides from exact submitted bytes and the supplied spent output under one retained recipe.
///
/// Checks are ordered: decode and witness shape; spent outpoint and canonical metadata; retained-context coherence; predecessor reconstruction and leastness; root, leaf, control and predecessor prefix; realization transition; successor reconstruction, output and prefix; successor leastness; separate comparisons. Funding asset and amount remain supplied facts because the input contains neither. The source label and branch context are stated provenance, not authenticated provenance.
///
/// # Errors
/// Returns the first failed check as [`MaturityContinuityRefusal`]. Binding failures retain only comparisons already reached; in particular a static-root refusal issues no semantic verdict, and an output refusal names the expected successor without attributing the mismatch to a semantic or static change.
pub fn project_maturity_continuity(
    input: MaturityProjectionInput<'_>,
) -> Result<ValidatedMaturityContinuity, MaturityContinuityRefusal> {
    let transaction = TargetTransaction::decode(input.submitted_bytes)
        .map_err(|error| Refusal::Decode(Box::new(error)))?;
    let stack = witness(&transaction)?;
    let decoded = transaction.inputs()[0].outpoint();
    if decoded != input.funded.outpoint {
        return Err(Refusal::SpentOutpoint {
            decoded,
            funded: input.funded.outpoint,
        });
    }
    let metadata = decode_state_metadata(&stack[4])
        .map_err(|error| Refusal::MetadataDecode(Box::new(error)))?;
    retained_context(&input, metadata)?;
    let target = closure_target().map_err(|error| Refusal::Target(Box::new(error)))?;
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
    let static_comparison = MaturityStaticComparison {
        static_root,
        control_block,
        descriptor_result: state_bundle_continuity(
            predecessor.static_subtree(),
            successor.static_subtree(),
        )
        .map_err(Box::new),
        constructor_result: predecessor_leastness
            .host_least_constructor()
            .continuity(successor_leastness.host_least_constructor())
            .map_err(Box::new),
    };
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
}

impl ValidatedMaturityContinuity {
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
    /// The unresolved accepted-step obligation with both implementation routes.
    #[must_use]
    pub const fn acceptance_obligation(&self) -> MaturityAcceptanceObligation {
        MaturityAcceptanceObligation::Outstanding {
            routes: [
                MaturityAcceptanceRoute::RelayWitnessRestructure,
                MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
            ],
        }
    }
    /// An output mismatch cannot distinguish another static tree from another semantic commitment.
    #[must_use]
    pub const fn successor_attribution(&self) -> MaturitySuccessorAttribution {
        MaturitySuccessorAttribution::CommitmentMismatchWithoutCauseAttribution
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of};
    use crate::matrix::EvidenceBoundary;
    use crate::maturity_corpus::maturity_run_of_record;
    use crate::maturity_native::MaturityAnnouncementPlanner;
    use crate::subject::{ExperimentalSubject, SubjectStanding};
    use std::fmt::Write as _;
    use std::io::Write as _;
    use std::sync::LazyLock;
    use target_elements_conformance::executor::{OperationStep, TargetOperationPlanner};
    use target_elements_conformance::protocol::{
        FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, NativeResourceObservation,
        ObservedOutcomeLayer, OperationSubject, WireOutpoint,
    };
    use transaction::bytes::{InputWitness, TargetOutput, Txid};

    #[derive(Clone)]
    struct Source {
        origin: MaturityByteSource,
        bytes: Vec<u8>,
        funded: MaturityFundedPredecessor,
        bundle: CandidateLinkedMaturityBundle,
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        planner_successor: CandidateStateConstructor,
    }

    impl Source {
        fn input(&self) -> MaturityProjectionInput<'_> {
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
    }

    fn coin(funded: &FundedOutput) -> MaturityFundedPredecessor {
        MaturityFundedPredecessor {
            outpoint: outpoint_of(&funded.outpoint).expect("funded outpoint"),
            asset: asset_of(&funded.asset).expect("funded asset"),
            amount: funded.amount_satoshis,
            program: decode_hex(&funded.script).expect("funded program"),
        }
    }

    fn archived() -> &'static Source {
        static SOURCE: LazyLock<Source> = LazyLock::new(|| {
            let corpus = maturity_run_of_record().expect("validated archive");
            let identity = corpus.evidence().identity().clone();
            let branch = corpus.evidence().branch();
            let mut planner =
                MaturityAnnouncementPlanner::new(identity.clone(), branch).expect("planner");
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

    fn node_free() -> &'static Source {
        static SOURCE: LazyLock<Source> = LazyLock::new(|| {
            let mut genesis = [0x22; 32];
            genesis[0] = 0x01;
            genesis[31] = 0xfe;
            let identity = CandidateDeploymentIdentity::new([0x17; 32], genesis).expect("identity");
            let branch = BranchContext::new([0x41; 32], 7).expect("branch");
            let mut planner =
                MaturityAnnouncementPlanner::new(identity.clone(), branch).expect("planner");
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

    fn positive(source: &Source) -> ValidatedMaturityContinuity {
        let record = source.project().expect("byte projection");
        let target = closure_target().expect("target");
        let encoded = decode_state_metadata(&record.transaction().witnesses()[0].stack()[4])
            .expect("metadata");
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
        assert_eq!(record.transaction().encode_without_witness().len(), 130);
        assert_eq!(record.transaction().weight(), 2084);
        let widths: Vec<_> = record.transaction().witnesses()[0]
            .stack()
            .iter()
            .map(Vec::len)
            .collect();
        assert_eq!(widths, [1, 4, 8, 32, 86, 1, 64, 1286, 65]);
        report_observed("archived", &record);
    }

    #[test]
    fn node_free_source_projects() {
        let record = positive(node_free());
        assert_eq!(record.source(), &MaturityByteSource::NodeFreeSubmitReady);
        report_observed("node-free", &record);
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
        for (index, expected) in WIDTHS.into_iter().enumerate() {
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
        let source = archived();
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
        }
    }

    #[test]
    fn replaced_predecessor_prefix_is_refused() {
        let source = archived();
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
        let source = archived();
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
    }

    #[test]
    fn later_admissible_successor_nonce_is_not_target_invalid() {
        let source = archived();
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
            Refusal::Decode(_) => "decode_refusal_is_preserved",
            Refusal::InputCount { .. }
            | Refusal::OutputCount { .. }
            | Refusal::WitnessItemCount { .. } => "transaction_shape_is_refused",
            Refusal::WitnessWidth { .. } => "all_seven_witness_widths_are_checked",
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
        }
    }

    #[test]
    fn refusal_root_walk_is_exhaustive() {
        // Constructed members check vocabulary coverage; only the named tests above establish runtime reachability.
        let source = archived();
        let constructor = Box::new(StateConstructorRefusal::RepresentationSearchExhausted);
        let diagnostics = Box::new(MaturityProjectionDiagnostics::empty());
        let refusals = [
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
        ];
        assert_eq!(refusals.len(), 23);
        for refusal in refusals {
            assert_ne!(reachability(&refusal).len(), 0);
        }
    }
}
