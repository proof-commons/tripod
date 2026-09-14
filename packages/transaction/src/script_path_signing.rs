//! Shared census validation and message construction for script-path signing.
//!
//! Role adapters compute their own protected preimage and supply their constants.
//! No constructor here publishes a route around the owner's finalized selection.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::bytes::{
    AssetField, InputWitness, OutputWitness, TargetTransaction, ValueField, compact_size,
    encode_asset_field, encode_value_field,
};
use crate::live_taproot::LiveCurveCapability;
use crate::taproot::{
    CONTROL_BASE_BYTES, DIGEST_BYTES, Digest32, TAPROOT_LEAF_MASK, TAPROOT_WITNESS_VERSION,
    branch_hash, tagged_hash, witness_program_script,
};

const MAXIMUM_CONTROL_PATH_DEPTH: usize = 128;

/// Constants selected by one role's established script-path profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScriptPathProfile {
    pub(crate) type_byte: u8,
    pub(crate) spend_type_byte: u8,
    pub(crate) signature_bytes: usize,
    pub(crate) key_version_byte: u8,
    pub(crate) codeseparator_position: u32,
}

/// Whether a signing input's witness will carry an annex.
///
/// Two members and not a boolean, because the profile refuses one of
/// them and a refusal needs something to refuse. The selected profile
/// admits [`Self::Absent`] only, which is what fixes
/// [`crate::live_census::OWNER_SPEND_TYPE_BYTE`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnnexDisposition {
    /// No annex, which is every witness this arc builds.
    Absent,
    /// An annex, which the profile refuses.
    ///
    /// A profile permitting one would have to say what it may contain,
    /// an annex is non-standard for relay, and the spend-type byte would
    /// stop being a constant.
    Present,
}

/// Whether a signing input bears an issuance.
///
/// The source review left this question on the census wave's desk by
/// name (rule:sighash-review:census-consequence). The accepted result
/// carries the output-witness vector and does *not* carry the
/// input-witness vector, and that asymmetry rests on a condition rather
/// than on a symmetry: term 10 of the message is the issuance
/// rangeproofs of every entry the input-witness vector happens to hold,
/// and it is length-dependent in exactly the way term 12 is.
///
/// While no input bears an issuance, every input-witness entry is
/// default-constructed, its two issuance rangeproofs serialize to one
/// zero byte each, and the whole term is a function of the input count —
/// which the protected bytes already carry. When an input does bear one,
/// the entries carry real rangeproof bytes the preimage does not contain
/// in any encoded form, and the term becomes unrecoverable in exactly
/// the way the proof-bearing lane's output-witness term is.
///
/// The selected profile requires the issuance dimension, so an
/// issuance-bearing candidate is admissible under the profile even
/// though no shape this arc builds today has one. The review gave the
/// census two ways out — a field for the input-witness issuance proofs,
/// or a typed refusal saying the census does not admit the shape — and
/// this is the second. Declaring [`Self::Bearing`] is
/// [`crate::live_census::OwnerCensusRefusal::IssuanceBearingInputRefused`], so the census's
/// silence about the input side is a claim with a checked precondition
/// rather than an omission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IssuanceDisposition {
    /// No issuance, which is the precondition the census's silence about
    /// the input-witness vector rests on.
    Absent,
    /// An issuance, which this census does not carry the field for.
    Bearing,
}

// --- The deployment ----------------------------------------------------

/// The deployment a census is bound to.
///
/// One field, and it is the one the message cannot do without. The
/// hasher is seeded at `src/script/interpreter.cpp:2671-2673` with the
/// tagged hash and then the genesis block hash *twice*, so two
/// candidates identical to the last byte, signed against two different
/// regtest chains, have different messages. A result bound to protected
/// bytes alone would be bound to a value that does not determine the
/// message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveDeployment {
    genesis_block_hash: Digest32,
}

impl LiveDeployment {
    /// The deployment whose chain starts at this block.
    #[must_use]
    pub const fn new(genesis_block_hash: Digest32) -> Self {
        Self { genesis_block_hash }
    }

    /// The genesis block hash, in the internal byte order the target
    /// writes into the message.
    #[must_use]
    pub const fn genesis_block_hash(&self) -> &Digest32 {
        &self.genesis_block_hash
    }
}

// --- The spent-output census ------------------------------------------

/// One spent output, as three message terms read it.
///
/// The asset and value fields feed term 6 and the script feeds term 7,
/// both taken at `src/script/interpreter.cpp:2454-2472` over the
/// *precomputed spent-output set* rather than over the transaction. That
/// is why a component holding only the transaction cannot form the
/// message, and why this census entry exists at all.
///
/// Observed target data. No opening for the value, whatever form the
/// value field takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpentOutputCensusEntry {
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
}

impl SpentOutputCensusEntry {
    /// The spent output carrying these observed fields.
    #[must_use]
    pub const fn new(asset: AssetField, value: ValueField, program: Vec<u8>) -> Self {
        Self {
            asset,
            value,
            program,
        }
    }

    /// The asset field, as observed.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The value field, as observed.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The script, as observed.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }
}

/// One signing input, as the census retains it.
///
/// Exactly the five per-input members the accepted option-B result
/// names: the input index, the tapleaf hash, the leaf version, the
/// codeseparator position, and the annex disposition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptPathSigningInputCensus {
    input_index: u32,
    tapleaf_hash: Digest32,
    leaf_version: LeafVersion,
    codeseparator_position: u32,
    annex: AnnexDisposition,
}

impl ScriptPathSigningInputCensus {
    /// Which input is being authorized — message term 14, written at
    /// `src/script/interpreter.cpp:2768`.
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.input_index
    }

    /// The executing leaf's hash — message term 16, written at `:2795`,
    /// and the same value the control-block check already computed at
    /// `:3287`.
    #[must_use]
    pub const fn tapleaf_hash(&self) -> &Digest32 {
        &self.tapleaf_hash
    }

    /// The executing leaf's version.
    ///
    /// Not itself a message term — term 17 is the *key* version, which
    /// the target fixes — but the value the tapleaf hash was taken over,
    /// carried so that a reader can recompute the leaf hash rather than
    /// trust it.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// Where in the leaf's execution the check occurred — message term
    /// 18, written at `:2798`.
    #[must_use]
    pub const fn codeseparator_position(&self) -> u32 {
        self.codeseparator_position
    }

    /// Whether the witness carries an annex, which is what the
    /// spend-type byte at term 13 reports.
    #[must_use]
    pub const fn annex(&self) -> AnnexDisposition {
        self.annex
    }
}

/// Why shared script-path census validation refused a candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptPathCensusRefusal {
    /// The spent output cardinality mismatch check failed.
    SpentOutputCardinalityMismatch {
        /// The inputs.
        inputs: usize,
        /// The spent outputs.
        spent_outputs: usize,
    },
    /// The output witness length mismatch check failed.
    OutputWitnessLengthMismatch {
        /// The outputs.
        outputs: usize,
        /// The output witnesses.
        output_witnesses: usize,
    },
    /// The annex disagreement check failed.
    AnnexDisagreement {
        /// The input index.
        input_index: u32,
        /// The declared.
        declared: AnnexDisposition,
        /// The recomputed spend type.
        recomputed_spend_type: u8,
    },
    /// The codeseparator position outside profile check failed.
    CodeseparatorPositionOutsideProfile {
        /// The input index.
        input_index: u32,
        /// The offered.
        offered: u32,
    },
    /// The deployment mismatch check failed.
    DeploymentMismatch {
        /// The expected.
        expected: Digest32,
        /// The offered.
        offered: Digest32,
    },
    /// The leaf hash does not commit check failed.
    LeafHashDoesNotCommit {
        /// The input index.
        input_index: u32,
    },
    /// The control block malformed check failed.
    ControlBlockMalformed {
        /// The input index.
        input_index: u32,
        /// The offered.
        offered: usize,
    },
    /// The leaf version disagrees with the control block check failed.
    LeafVersionDisagreesWithTheControlBlock {
        /// The input index.
        input_index: u32,
        /// The declared.
        declared: u8,
        /// The control byte.
        control_byte: u8,
    },
    /// The control block internal key is not a point check failed.
    ControlBlockInternalKeyIsNotAPoint {
        /// The input index.
        input_index: u32,
    },
    /// The type byte outside profile check failed.
    TypeByteOutsideProfile {
        /// The offered.
        offered: u8,
    },
    /// The signature width outside profile check failed.
    SignatureWidthOutsideProfile {
        /// The offered.
        offered: usize,
    },
    /// The issuance bearing input refused check failed.
    IssuanceBearingInputRefused {
        /// The input index.
        input_index: u32,
    },
    /// The signing input out of range check failed.
    SigningInputOutOfRange {
        /// The input index.
        input_index: u32,
        /// The inputs.
        inputs: usize,
    },
    /// The duplicate signing input check failed.
    DuplicateSigningInput {
        /// The input index.
        input_index: u32,
    },
    /// No input was requested.
    NoSigningInputRequested,
    /// The offered protected bytes differ from the expected bytes.
    ProtectedBytesAreNotTheCandidates,
}

/// An adapter's borrowed per-input validation facts.
pub(crate) struct ScriptPathInputRequest<'a> {
    pub(crate) input_index: u32,
    pub(crate) tapleaf_hash: &'a Digest32,
    pub(crate) leaf_version: LeafVersion,
    pub(crate) codeseparator_position: u32,
    pub(crate) annex: AnnexDisposition,
    pub(crate) issuance: IssuanceDisposition,
    pub(crate) control_block: &'a [u8],
}

impl ScriptPathInputRequest<'_> {
    /// Which input is being authorized.
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.input_index
    }

    /// The executing leaf's hash.
    #[must_use]
    pub const fn tapleaf_hash(&self) -> &Digest32 {
        self.tapleaf_hash
    }

    /// The executing leaf's version.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// Where in the leaf's execution the check occurs.
    #[must_use]
    pub const fn codeseparator_position(&self) -> u32 {
        self.codeseparator_position
    }

    /// Whether the witness will carry an annex.
    #[must_use]
    pub const fn annex(&self) -> AnnexDisposition {
        self.annex
    }

    /// Whether the input bears an issuance.
    #[must_use]
    pub const fn issuance(&self) -> IssuanceDisposition {
        self.issuance
    }

    /// The control block the witness will carry.
    #[must_use]
    pub const fn control_block(&self) -> &[u8] {
        self.control_block
    }
}

/// Candidate-wide values supplied together by a role adapter.
pub(crate) struct ScriptPathCensusParts {
    pub(crate) candidate: TargetTransaction,
    pub(crate) protected_bytes: Vec<u8>,
    pub(crate) output_witnesses: Vec<OutputWitness>,
    pub(crate) spent_outputs: Vec<SpentOutputCensusEntry>,
    pub(crate) deployment: LiveDeployment,
    pub(crate) profile: ScriptPathProfile,
}

/// Validated message inputs for one frozen candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptPathSigningCensus {
    candidate: TargetTransaction,
    protected_bytes: Vec<u8>,
    output_witnesses: Vec<OutputWitness>,
    spent_outputs: Vec<SpentOutputCensusEntry>,
    genesis_block_hash: Digest32,
    signing_inputs: Vec<ScriptPathSigningInputCensus>,
    profile: ScriptPathProfile,
}

impl ScriptPathSigningCensus {
    pub(crate) fn assemble<E: From<ScriptPathCensusRefusal>>(
        target: &ReviewedElementsTapscriptDefinition,
        parts: ScriptPathCensusParts,
        expected_protected_bytes: &[u8],
        requests: &[ScriptPathInputRequest<'_>],
        curve: &dyn LiveCurveCapability,
        mut check_selected: impl FnMut(&ScriptPathInputRequest<'_>) -> Result<(), E>,
    ) -> Result<Self, E> {
        check_protected_bytes(expected_protected_bytes, &parts.protected_bytes)?;
        check_cardinalities(&parts)?;
        if requests.is_empty() {
            return Err(ScriptPathCensusRefusal::NoSigningInputRequested.into());
        }
        let mut seen = BTreeSet::new();
        let mut signing_inputs = Vec::with_capacity(requests.len());
        for request in requests {
            let position = check_input(request, &parts, &mut seen)?;
            check_leaf_commits(target, request, &parts.spent_outputs[position], curve)?;
            check_selected(request)?;
            signing_inputs.push(ScriptPathSigningInputCensus {
                input_index: request.input_index(),
                tapleaf_hash: *request.tapleaf_hash(),
                leaf_version: request.leaf_version(),
                codeseparator_position: request.codeseparator_position(),
                annex: request.annex(),
            });
        }
        Ok(Self {
            candidate: parts.candidate,
            protected_bytes: parts.protected_bytes,
            output_witnesses: parts.output_witnesses,
            spent_outputs: parts.spent_outputs,
            genesis_block_hash: *parts.deployment.genesis_block_hash(),
            signing_inputs,
            profile: parts.profile,
        })
    }
    /// The candidate the census is about.
    ///
    /// Candidate structure, which the accepted result admits alongside
    /// observed target data. The message's version, locktime, input and
    /// output terms are read from here; the protected bytes cannot serve
    /// that purpose, because the private lane's preimage is the
    /// witnessless serialization followed by the output-witness vector
    /// and is not itself a decodable transaction.
    #[must_use]
    pub const fn candidate(&self) -> &TargetTransaction {
        &self.candidate
    }

    /// The exact protected bytes every owner binds to.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        &self.protected_bytes
    }

    /// The output-witness vector at its consensus length.
    #[must_use]
    pub fn output_witnesses(&self) -> &[OutputWitness] {
        &self.output_witnesses
    }

    /// The spent-output census, in input order.
    #[must_use]
    pub fn spent_outputs(&self) -> &[SpentOutputCensusEntry] {
        &self.spent_outputs
    }

    /// The deployment's genesis block hash.
    #[must_use]
    pub const fn genesis_block_hash(&self) -> &Digest32 {
        &self.genesis_block_hash
    }

    /// Every signing input, in request order.
    #[must_use]
    pub fn signing_inputs(&self) -> &[ScriptPathSigningInputCensus] {
        &self.signing_inputs
    }
}

fn check_cardinalities(parts: &ScriptPathCensusParts) -> Result<(), ScriptPathCensusRefusal> {
    let inputs = parts.candidate.inputs().len();
    let outputs = parts.candidate.outputs().len();
    if parts.spent_outputs.len() != inputs {
        return Err(ScriptPathCensusRefusal::SpentOutputCardinalityMismatch {
            inputs,
            spent_outputs: parts.spent_outputs.len(),
        });
    }
    if parts.output_witnesses.len() != outputs
        || parts.candidate.output_witnesses() != parts.output_witnesses.as_slice()
    {
        return Err(ScriptPathCensusRefusal::OutputWitnessLengthMismatch {
            outputs,
            output_witnesses: parts.output_witnesses.len(),
        });
    }
    Ok(())
}

fn check_input(
    request: &ScriptPathInputRequest<'_>,
    parts: &ScriptPathCensusParts,
    seen: &mut BTreeSet<u32>,
) -> Result<usize, ScriptPathCensusRefusal> {
    let inputs = parts.candidate.inputs().len();
    let index = request.input_index();
    let position = usize::try_from(index)
        .ok()
        .filter(|position| *position < inputs)
        .ok_or(ScriptPathCensusRefusal::SigningInputOutOfRange {
            input_index: index,
            inputs,
        })?;
    if !seen.insert(index) {
        return Err(ScriptPathCensusRefusal::DuplicateSigningInput { input_index: index });
    }
    if request.issuance() == IssuanceDisposition::Bearing {
        return Err(ScriptPathCensusRefusal::IssuanceBearingInputRefused { input_index: index });
    }
    if request.codeseparator_position() != parts.profile.codeseparator_position {
        return Err(
            ScriptPathCensusRefusal::CodeseparatorPositionOutsideProfile {
                input_index: index,
                offered: request.codeseparator_position(),
            },
        );
    }
    let recomputed = spend_type_byte(request.annex());
    if recomputed != parts.profile.spend_type_byte {
        return Err(ScriptPathCensusRefusal::AnnexDisagreement {
            input_index: index,
            declared: request.annex(),
            recomputed_spend_type: recomputed,
        });
    }
    Ok(position)
}

pub(crate) fn check_protected_bytes(
    expected: &[u8],
    offered: &[u8],
) -> Result<(), ScriptPathCensusRefusal> {
    if offered == expected {
        Ok(())
    } else {
        Err(ScriptPathCensusRefusal::ProtectedBytesAreNotTheCandidates)
    }
}

pub(crate) fn check_deployment(
    expected: Digest32,
    run: LiveDeployment,
) -> Result<(), ScriptPathCensusRefusal> {
    let offered = *run.genesis_block_hash();
    if offered == expected {
        Ok(())
    } else {
        Err(ScriptPathCensusRefusal::DeploymentMismatch { expected, offered })
    }
}

pub(crate) const fn check_type_byte(
    profile: ScriptPathProfile,
    offered: u8,
) -> Result<(), ScriptPathCensusRefusal> {
    if offered == profile.type_byte {
        Ok(())
    } else {
        Err(ScriptPathCensusRefusal::TypeByteOutsideProfile { offered })
    }
}

pub(crate) const fn check_signature_width(
    profile: ScriptPathProfile,
    offered: usize,
) -> Result<(), ScriptPathCensusRefusal> {
    if offered == profile.signature_bytes {
        Ok(())
    } else {
        Err(ScriptPathCensusRefusal::SignatureWidthOutsideProfile { offered })
    }
}

/// The spend-type byte one annex disposition recomputes to.
///
/// `src/script/interpreter.cpp:2748`: the extension flag shifted left by
/// one, plus the annex bit. The extension flag is 1 for every spend this
/// profile admits, because the profile is script path only.
#[must_use]
pub(crate) const fn spend_type_byte(annex: AnnexDisposition) -> u8 {
    const SCRIPT_PATH_EXTENSION_FLAG: u8 = 1;

    let annex_bit = match annex {
        AnnexDisposition::Absent => 0,
        AnnexDisposition::Present => 1,
    };

    (SCRIPT_PATH_EXTENSION_FLAG << 1_u8) | annex_bit
}

/// `VerifyTaprootCommitment`, recomputed on this side.
fn check_leaf_commits(
    target: &ReviewedElementsTapscriptDefinition,
    request: &ScriptPathInputRequest<'_>,
    spent: &SpentOutputCensusEntry,
    curve: &dyn LiveCurveCapability,
) -> Result<(), ScriptPathCensusRefusal> {
    let index = request.input_index();
    let block = request.control_block();

    if block.len() < CONTROL_BASE_BYTES
        || !(block.len() - CONTROL_BASE_BYTES).is_multiple_of(DIGEST_BYTES)
        || (block.len() - CONTROL_BASE_BYTES) / DIGEST_BYTES > MAXIMUM_CONTROL_PATH_DEPTH
    {
        return Err(ScriptPathCensusRefusal::ControlBlockMalformed {
            input_index: index,
            offered: block.len(),
        });
    }

    let control_byte = block[0];
    if control_byte & TAPROOT_LEAF_MASK != request.leaf_version().get() & TAPROOT_LEAF_MASK {
        return Err(
            ScriptPathCensusRefusal::LeafVersionDisagreesWithTheControlBlock {
                input_index: index,
                declared: request.leaf_version().get(),
                control_byte,
            },
        );
    }

    let internal_key = &block[1..CONTROL_BASE_BYTES];

    // The merkle root, folded from the executing leaf upward through the
    // path the control block carries. `branch_hash` orders each pair
    // lexicographically, which is what lets the path be a bare list with
    // no side bits — the same recursion the committed tree was built by,
    // run in the other direction.
    let mut root = *request.tapleaf_hash();
    for sibling in block[CONTROL_BASE_BYTES..].as_chunks::<DIGEST_BYTES>().0 {
        root = branch_hash(root, *sibling);
    }

    let output_key = curve.output_key(internal_key, &root).ok_or(
        ScriptPathCensusRefusal::ControlBlockInternalKeyIsNotAPoint { input_index: index },
    )?;

    if output_key.parity().bit() != control_byte & !TAPROOT_LEAF_MASK {
        return Err(ScriptPathCensusRefusal::LeafHashDoesNotCommit { input_index: index });
    }

    // Rebuilt through the crate's own reviewed witness-program grammar
    // rather than parsed out of the spent script. Parsing would have
    // meant a second opinion about which opcode a version-one program
    // starts with, and the two could come to differ.
    let expected = witness_program_script(target, TAPROOT_WITNESS_VERSION, output_key.key())
        .map_err(|_| ScriptPathCensusRefusal::LeafHashDoesNotCommit { input_index: index })?;

    if expected != spent.program() {
        return Err(ScriptPathCensusRefusal::LeafHashDoesNotCommit { input_index: index });
    }

    Ok(())
}

/// The tag the target seeds its taproot message hasher with.
///
/// `src/script/interpreter.cpp:552`. It is the Elements tag and not
/// BIP-341's: transcribing the Bitcoin value here would produce a
/// construction that hashes cleanly and describes the wrong message.
pub const TAP_SIGHASH_TAG: &str = "TapSighash/elements";

/// How a candidate's two witness vectors are treated when forming the
/// message.
///
/// Four members rather than two, and the four are the recorded twin
/// diagnosis's own candidate names. Two would have confounded the terms:
/// the message has two length-dependent terms, one over each witness
/// vector, and a run computing only "grown" and "emptied" cannot say
/// which of them moved.
///
/// The two length-dependent terms are the review's terms 10 and 12.
/// `GetIssuanceRangeproofsSHA256` at `src/script/interpreter.cpp:2431-2440`
/// iterates the input-witness vector and hashes each entry's two
/// issuance rangeproofs; `GetOutputWitnessesSHA256` at `:2418-2425`
/// iterates the output-witness vector and hashes each entry. Neither
/// takes a position and neither reads the transaction's own cardinality,
/// so both hash whatever the vector happens to hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessVectorTreatment {
    /// Both vectors at the length the wire form carries, which is one
    /// entry per input and one per output.
    ///
    /// What consensus hashes. `UnserializeTransaction` at
    /// `src/primitives/transaction.h:394-398` resizes both vectors
    /// together whenever the witness flag is set, and
    /// `SerializeTransaction` at `:460`, `:473-474` sets that flag and
    /// resizes both whenever the transaction has any witness at all.
    BothGrown,
    /// The output-witness vector emptied, the input side left grown.
    ///
    /// What a signer working from witnessless bytes produces, and what
    /// the target's own wallet produces: `CWallet::SignTransaction`
    /// resizes the input-witness vector to the input count at
    /// `src/wallet/wallet.cpp:2315` before any signing begins, and
    /// nothing resizes the output-witness vector until serialization.
    /// This is the recorded `G11-W11-06` hazard, and the twin diagnosis
    /// found the explicit row's signature verifying against this
    /// candidate and against no other.
    OutputsEmptied,
    /// The input-witness vector emptied, the output side left grown.
    ///
    /// The structural twin. The source symmetry is exact, so the
    /// candidate is well posed; the review confirmed the term is
    /// length-dependent and refuted the hazard as an observed defect of
    /// this target's signer.
    InputsEmptied,
    /// Both vectors emptied.
    ///
    /// What a signer forming the message from a witnessless
    /// serialization alone would hash, with nothing resized on either
    /// side.
    BothEmptied,
}

impl WitnessVectorTreatment {
    /// Whether the output-witness vector is hashed at its length.
    const fn output_side_grown(self) -> bool {
        matches!(self, Self::BothGrown | Self::InputsEmptied)
    }

    /// Whether the input-witness vector is hashed at its length.
    const fn input_side_grown(self) -> bool {
        matches!(self, Self::BothGrown | Self::OutputsEmptied)
    }
}

/// The two candidate messages the recorded diagnosis compares.
///
/// The diagnosis's method, reproduced on this side: for one candidate
/// and one signing input it computes the message with the output-witness
/// vector grown and the message with it empty, and reports which one a
/// produced signature actually verifies against. This type is the pair
/// itself, without the verdict — asking a target which one it accepts is
/// a later wave's work, and a pair carrying an answer nobody observed
/// would be the assertion this arc refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CandidateMessagePair {
    with_vector_grown: Digest32,
    with_vector_empty: Digest32,
}

impl CandidateMessagePair {
    /// The message with the output-witness vector at its consensus
    /// length — the diagnosis's `digest_with_vector_grown`.
    #[must_use]
    pub const fn with_vector_grown(&self) -> &Digest32 {
        &self.with_vector_grown
    }

    /// The message with the output-witness vector empty — the
    /// diagnosis's `digest_with_vector_empty`.
    #[must_use]
    pub const fn with_vector_empty(&self) -> &Digest32 {
        &self.with_vector_empty
    }

    /// Whether the two candidates are distinct.
    ///
    /// The property the whole hazard rests on, exposed as a value rather
    /// than assumed. Two candidates that coincided would make any verdict
    /// about which one a signature verifies against a coincidence, which
    /// is precisely what the recorded diagnosis checks before reporting.
    #[must_use]
    pub fn candidates_are_distinct(&self) -> bool {
        self.with_vector_grown != self.with_vector_empty
    }
}

/// The message the target forms for one signing input of one census.
///
/// Total rather than fallible: a signing input obtained from a census is
/// one the census already validated, so there is no clause left for this
/// function to fail. The refusals live where the census is assembled,
/// which is the only place a caller can supply something wrong.
#[must_use]
pub fn script_path_message(
    census: &ScriptPathSigningCensus,
    input: &ScriptPathSigningInputCensus,
    treatment: WitnessVectorTreatment,
) -> Digest32 {
    let mut stream = whole_transaction_stream(census, treatment);

    // Term 13, computed at `:2748` and written at `:2749`. Script path
    // and no annex, so it is a constant.
    stream.push(census.profile.spend_type_byte);

    // Term 14, written at `:2768`. Term 15, the annex hash, is absent
    // because the profile refuses the annex — which is what made term 13
    // a constant.
    stream.extend_from_slice(&input.input_index().to_le_bytes());

    // Terms 16, 17 and 18, the tapscript additions, written at
    // `:2793-2798`. The key version is the target's constant; the
    // codeseparator position is carried by the census rather than
    // assumed, because its constancy is a condition on the leaf
    // vocabulary rather than a property of the target.
    stream.extend_from_slice(input.tapleaf_hash());
    stream.push(census.profile.key_version_byte);
    stream.extend_from_slice(&input.codeseparator_position().to_le_bytes());

    // One SHA256 over the whole stream, at `:2801`.
    tagged_hash(TAP_SIGHASH_TAG, &stream)
}

/// The spend-type byte a key-path spend with no annex writes.
///
/// BIP-341 composes the byte as twice the extension flag plus the annex
/// bit, and the target follows it at `:2748`. A script path with no
/// annex is [`crate::live_census::OWNER_SPEND_TYPE_BYTE`]; the key path's extension flag is
/// zero, so the byte is zero, and the three tapscript terms that follow
/// the input index in the script-path stream are not written at all.
pub const KEY_PATH_SPEND_TYPE_BYTE: u8 = 0x00;

/// A candidate message for a KEY-PATH spend of the same candidate.
///
/// # This is not a reviewed construction, and the distinction matters
///
/// [`script_path_message`] is written from the Wave-1 source
/// review's term table, term by term, at the lines the review cites. No
/// review in this repository covers the key path, because no constructor
/// here is meant to be spent by one. So this function is the script-path
/// stream with its tapscript tail replaced by what the source's own
/// composition rule says a key-path spend writes, and it is
/// candidate-scoped in the strict sense: nothing has observed that a
/// target forms this message, and nothing here claims it does.
///
/// It shares its whole-transaction prefix with the reviewed
/// construction — one private function writes terms 0 to 12 for both —
/// rather than respelling twelve terms, so the two messages differ in
/// their tail and in nothing else. A caller comparing
/// them is comparing the spend type and the tapscript additions, which
/// is the only comparison this function supports.
///
/// # What it is for
///
/// One probe, which offers a key-path witness to a target and records
/// what the target says. The probe's finding is the target's answer; the
/// message this function returns is a datum about what was submitted,
/// not a claim about what was verified against.
#[must_use]
pub fn key_path_message(
    census: &ScriptPathSigningCensus,
    input_index: u32,
    treatment: WitnessVectorTreatment,
) -> Digest32 {
    let mut stream = whole_transaction_stream(census, treatment);

    // The key path's spend type, then the input index — and then
    // nothing. The tapleaf hash, the key version and the codeseparator
    // position are the extension the script path writes, and a key-path
    // stream that carried them would be the script-path stream with a
    // different first byte rather than a different spend.
    stream.push(KEY_PATH_SPEND_TYPE_BYTE);
    stream.extend_from_slice(&input_index.to_le_bytes());

    tagged_hash(TAP_SIGHASH_TAG, &stream)
}

/// Terms 0 to 12: everything both spend paths write, in order.
///
/// Extracted rather than duplicated. The two spend paths share every
/// whole-transaction term and diverge only at the spend-type byte, and a
/// second spelling of the shared prefix would be a second place for the
/// reviewed construction to drift from the one the probe submits under.
fn whole_transaction_stream(
    census: &ScriptPathSigningCensus,
    treatment: WitnessVectorTreatment,
) -> Vec<u8> {
    let candidate = census.candidate();
    let mut stream = Vec::new();

    // Term 0 is not in the stream: it is the hasher's seed. The tagged
    // hash at `:552`, then the genesis block hash twice at `:2671-2673`.
    // BIP-341's epoch byte is deliberately absent and the source keeps
    // the comment saying so at `:2714-2716`.
    stream.extend_from_slice(census.genesis_block_hash());
    stream.extend_from_slice(census.genesis_block_hash());

    // Term 1, the hash type, written at `:2722`. The profile is
    // default-only, so this is a constant and not an input.
    stream.push(census.profile.type_byte);

    // Terms 2 and 3, written directly and unconditionally at `:2725-2726`.
    stream.extend_from_slice(&candidate.version().to_le_bytes());
    stream.extend_from_slice(&candidate.lock_time().to_le_bytes());

    // Terms 4 to 10, the whole-transaction hashes, written at
    // `:2727-2739`. All seven are gated on the input type not being the
    // permitted-extension one, which this profile refuses.
    stream.extend_from_slice(&outpoint_flags_hash(candidate));
    stream.extend_from_slice(&prevouts_hash(candidate));
    stream.extend_from_slice(&spent_assets_amounts_hash(census.spent_outputs()));
    stream.extend_from_slice(&spent_scripts_hash(census.spent_outputs()));
    stream.extend_from_slice(&sequences_hash(candidate));
    stream.extend_from_slice(&issuances_hash(candidate));
    stream.extend_from_slice(&issuance_rangeproofs_hash(
        candidate.inputs().len(),
        treatment.input_side_grown(),
    ));

    // Terms 11 and 12, written at `:2742-2743` and gated on the output
    // type being all-outputs, which the default type resolves to at
    // `:2719`.
    stream.extend_from_slice(&outputs_hash(candidate));
    stream.extend_from_slice(&output_witnesses_hash(
        census.output_witnesses(),
        treatment.output_side_grown(),
    ));

    stream
}

/// The recorded diagnosis's two candidate messages for one signing
/// input.
///
/// The output side varies and the input side is held at its consensus
/// length in both, which is the recorded diagnosis's own shape: it
/// deep-copies the transaction, clears `vtxoutwit`, and hashes again,
/// leaving `vtxinwit` alone.
#[must_use]
pub fn message_pair(
    census: &ScriptPathSigningCensus,
    input: &ScriptPathSigningInputCensus,
) -> CandidateMessagePair {
    CandidateMessagePair {
        with_vector_grown: script_path_message(census, input, WitnessVectorTreatment::BothGrown),
        with_vector_empty: script_path_message(
            census,
            input,
            WitnessVectorTreatment::OutputsEmptied,
        ),
    }
}

// --- The whole-transaction hashes -------------------------------------

/// One SHA256 over a stream, which is what every helper below finishes
/// with.
fn single_sha256(bytes: &[u8]) -> Digest32 {
    Sha256::digest(bytes).into()
}

/// Term 4, `GetOutpointFlagsSHA256` at `:2368-2375`.
///
/// One byte per input, and `GetOutpointFlag` at `:137-143` composes it
/// from the issuance and peg-in markers alone. This crate constructs
/// neither and its decoder refuses both, so the byte is zero for every
/// input — computed from the input list rather than substituted, so a
/// transaction that somehow carried one would not be hashed as though it
/// did not.
fn outpoint_flags_hash(candidate: &TargetTransaction) -> Digest32 {
    let flags = vec![0_u8; candidate.inputs().len()];
    single_sha256(&flags)
}

/// Term 5, `GetPrevoutsSHA256` at `:2379-2386`.
///
/// `ss << txin.prevout` serializes a `COutPoint` as its 32-byte hash
/// followed by the index as four little-endian bytes.
fn prevouts_hash(candidate: &TargetTransaction) -> Digest32 {
    let mut stream = Vec::new();
    for input in candidate.inputs() {
        let outpoint = input.outpoint();
        stream.extend_from_slice(outpoint.txid().internal());
        stream.extend_from_slice(&outpoint.index().to_le_bytes());
    }
    single_sha256(&stream)
}

/// Term 6, `GetSpentAssetsAmountsSHA256` at `:2454-2462`.
///
/// The asset field then the value field of every spent output, and
/// neither the nonce nor the script — which is why the field encoders
/// are shared with the output encoder rather than this term reusing a
/// whole-output serialization that would write two fields too many.
fn spent_assets_amounts_hash(spent: &[SpentOutputCensusEntry]) -> Digest32 {
    let mut stream = Vec::new();
    for entry in spent {
        encode_asset_field(&mut stream, entry.asset());
        encode_value_field(&mut stream, entry.value());
    }
    single_sha256(&stream)
}

/// Term 7, `GetSpentScriptsSHA256` at `:2465-2472`.
///
/// `ss << txout.scriptPubKey` serializes a `CScript` as a compact-size
/// length followed by the bytes.
fn spent_scripts_hash(spent: &[SpentOutputCensusEntry]) -> Digest32 {
    let mut stream = Vec::new();
    for entry in spent {
        stream.extend_from_slice(&compact_size(entry.program().len() as u64));
        stream.extend_from_slice(entry.program());
    }
    single_sha256(&stream)
}

/// Term 8, `GetSequencesSHA256` at `:2390-2397`.
fn sequences_hash(candidate: &TargetTransaction) -> Digest32 {
    let mut stream = Vec::new();
    for input in candidate.inputs() {
        stream.extend_from_slice(&input.sequence().to_le_bytes());
    }
    single_sha256(&stream)
}

/// Term 9, `GetIssuanceSHA256` at `:2402-2412`.
///
/// One zero byte where the issuance is null, and the whole issuance
/// otherwise. Every input this crate represents has a null issuance —
/// the type carries no field for one and the decoder refuses the marker
/// bit — so the stream is one zero byte per input.
fn issuances_hash(candidate: &TargetTransaction) -> Digest32 {
    let zeros = vec![0_u8; candidate.inputs().len()];
    single_sha256(&zeros)
}

/// Term 10, `GetIssuanceRangeproofsSHA256` at `:2431-2440`.
///
/// Two length-prefixed byte vectors per entry the input-witness vector
/// happens to hold. Every entry is default-constructed while no input
/// bears an issuance, which the census refuses to admit otherwise, so
/// each entry contributes the two bytes `00 00` — the value the twin
/// diagnosis measured directly.
fn issuance_rangeproofs_hash(inputs: usize, grown: bool) -> Digest32 {
    let entries = if grown { inputs } else { 0 };
    let stream = vec![0_u8; entries.saturating_mul(2)];
    single_sha256(&stream)
}

/// Term 11, `GetOutputsSHA256` at `:2443-2450`.
fn outputs_hash(candidate: &TargetTransaction) -> Digest32 {
    let mut stream = Vec::new();
    for output in candidate.outputs() {
        output.encode(&mut stream);
    }
    single_sha256(&stream)
}

/// Term 12, `GetOutputWitnessesSHA256` at `:2418-2425`.
///
/// Every entry the vector happens to hold, and an entry is the
/// surjection proof then the range proof, each length-prefixed
/// (`src/primitives/txwitness.h:45-63`). An emptied vector hashes the
/// empty string, which is the whole content of the recorded hazard: a
/// signer working from witnessless bytes hashes nothing here while
/// consensus hashes one entry per output.
fn output_witnesses_hash(witnesses: &[OutputWitness], grown: bool) -> Digest32 {
    let mut stream = Vec::new();
    if grown {
        for witness in witnesses {
            stream.extend_from_slice(&compact_size(witness.surjection_proof().len() as u64));
            stream.extend_from_slice(witness.surjection_proof());
            stream.extend_from_slice(&compact_size(witness.range_proof().len() as u64));
            stream.extend_from_slice(witness.range_proof());
        }
    }
    single_sha256(&stream)
}

/// Why a response set does not answer each requested input exactly once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResponseCoverageRefusal {
    Unexpected { input_index: u32 },
    Duplicate { input_index: u32 },
    Missing { input_index: u32 },
}

pub(crate) fn check_response_index(
    inputs: &[ScriptPathSigningInputCensus],
    input_index: u32,
) -> Result<(), ResponseCoverageRefusal> {
    if inputs
        .iter()
        .any(|input| input.input_index() == input_index)
    {
        Ok(())
    } else {
        Err(ResponseCoverageRefusal::Unexpected { input_index })
    }
}

pub(crate) const fn check_response_duplicate(
    already_present: bool,
    input_index: u32,
) -> Result<(), ResponseCoverageRefusal> {
    if already_present {
        Err(ResponseCoverageRefusal::Duplicate { input_index })
    } else {
        Ok(())
    }
}

pub(crate) const fn check_response_present(
    present: bool,
    input_index: u32,
) -> Result<(), ResponseCoverageRefusal> {
    if present {
        Ok(())
    } else {
        Err(ResponseCoverageRefusal::Missing { input_index })
    }
}

/// The script-path witness ABI shared by both signing roles.
pub(crate) fn script_path_witness(
    signature: Vec<u8>,
    leaf_script: &[u8],
    control_block: &[u8],
) -> InputWitness {
    InputWitness::new(vec![
        signature,
        leaf_script.to_vec(),
        control_block.to_vec(),
    ])
}
