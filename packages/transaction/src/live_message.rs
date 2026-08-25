//! An independently written construction of the target's owner message.
//!
//! # Where this construction's expectation comes from
//!
//! Every term below was written from the Wave-1 source review's own term
//! table (tab:sighash-review:terms in
//! `plans/reference/owner-sighash-review.md`) at the pinned Elements tip
//! `b7fc5d080a`, and each is cited at the line the review cites. Nothing
//! here was derived from the code that produces the census it consumes,
//! and nothing here consults a target.
//!
//! That separation is the whole value of the module, and the arc's rule
//! says why: one computation is never both observation and expectation.
//! A message construction whose expectation came from the same code path
//! as the value it checks would agree with itself and report the
//! agreement as evidence.
//!
//! # What is reused, and why that is not the same code path
//!
//! Two things are deliberately taken from this crate rather than
//! respelled here: the tagged-hash primitive, and the *serialization* of
//! an output, an output witness, and an asset or value field.
//!
//! The thing under check is the message — which terms are written, in
//! what order, over what. The serialization of a target output is a
//! different claim, evidenced elsewhere, and this crate exists to
//! produce exactly those bytes. Respelling it here would create a second
//! opinion about output encoding that is free to drift from the bytes
//! the crate actually emits, which would make a disagreement between
//! them look like a message finding when it was an encoding typo. So the
//! encodings are shared and the *stream* is independent, and this
//! paragraph is the boundary stated rather than left to be inferred.
//!
//! # What this module does not claim
//!
//! No digest here is authoritative. The wave that wrote this module
//! produced a candidate recomputation and nothing else:
//! `SighashProfileUnreviewed` stood, `OwnerSighashNotComputable` stood,
//! no dimension moved to reviewed, and no target had been asked what it
//! thought of any value this module returns. A recomputation matching a
//! target's digest establishes that the model is right about the
//! message, not that a node accepts a spend built from it.
//!
//! Both of those residuals have since been cleared, and neither by
//! anything in this module — the digest blocker by a later wave's
//! observed acceptance, and the profile residual by the review verdict
//! and the re-typing. What is unchanged is the non-claim
//! itself: this module still asserts no digest, and a value it returns
//! is still a first-party recomputation rather than a target's answer.

use sha2::{Digest, Sha256};

use crate::bytes::{
    OutputWitness, TargetTransaction, compact_size, encode_asset_field, encode_value_field,
};
use crate::live_census::{
    OWNER_KEY_VERSION_BYTE, OWNER_SIGHASH_TYPE_BYTE, OWNER_SPEND_TYPE_BYTE, OwnerSigningCensus,
    OwnerSigningInputCensus, SpentOutputCensusEntry,
};
use crate::taproot::{Digest32, tagged_hash};

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
pub fn candidate_owner_message(
    census: &OwnerSigningCensus,
    input: &OwnerSigningInputCensus,
    treatment: WitnessVectorTreatment,
) -> Digest32 {
    let mut stream = whole_transaction_stream(census, treatment);

    // Term 13, computed at `:2748` and written at `:2749`. Script path
    // and no annex, so it is a constant.
    stream.push(OWNER_SPEND_TYPE_BYTE);

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
    stream.push(OWNER_KEY_VERSION_BYTE);
    stream.extend_from_slice(&input.codeseparator_position().to_le_bytes());

    // One SHA256 over the whole stream, at `:2801`.
    tagged_hash(TAP_SIGHASH_TAG, &stream)
}

/// The spend-type byte a key-path spend with no annex writes.
///
/// BIP-341 composes the byte as twice the extension flag plus the annex
/// bit, and the target follows it at `:2748`. A script path with no
/// annex is [`OWNER_SPEND_TYPE_BYTE`]; the key path's extension flag is
/// zero, so the byte is zero, and the three tapscript terms that follow
/// the input index in the script-path stream are not written at all.
pub const KEY_PATH_SPEND_TYPE_BYTE: u8 = 0x00;

/// A candidate message for a KEY-PATH spend of the same candidate.
///
/// # This is not a reviewed construction, and the distinction matters
///
/// [`candidate_owner_message`] is written from the Wave-1 source
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
pub fn candidate_key_path_message(
    census: &OwnerSigningCensus,
    input: &OwnerSigningInputCensus,
    treatment: WitnessVectorTreatment,
) -> Digest32 {
    let mut stream = whole_transaction_stream(census, treatment);

    // The key path's spend type, then the input index — and then
    // nothing. The tapleaf hash, the key version and the codeseparator
    // position are the extension the script path writes, and a key-path
    // stream that carried them would be the script-path stream with a
    // different first byte rather than a different spend.
    stream.push(KEY_PATH_SPEND_TYPE_BYTE);
    stream.extend_from_slice(&input.input_index().to_le_bytes());

    tagged_hash(TAP_SIGHASH_TAG, &stream)
}

/// Terms 0 to 12: everything both spend paths write, in order.
///
/// Extracted rather than duplicated. The two spend paths share every
/// whole-transaction term and diverge only at the spend-type byte, and a
/// second spelling of the shared prefix would be a second place for the
/// reviewed construction to drift from the one the probe submits under.
fn whole_transaction_stream(
    census: &OwnerSigningCensus,
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
    stream.push(OWNER_SIGHASH_TYPE_BYTE);

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
pub fn candidate_message_pair(
    census: &OwnerSigningCensus,
    input: &OwnerSigningInputCensus,
) -> CandidateMessagePair {
    CandidateMessagePair {
        with_vector_grown: candidate_owner_message(
            census,
            input,
            WitnessVectorTreatment::BothGrown,
        ),
        with_vector_empty: candidate_owner_message(
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
