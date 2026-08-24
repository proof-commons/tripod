//! The owner signing-input census, and the profile constants it is
//! checked against.
//!
//! # What this module is for
//!
//! The owner-sighash concept's accepted result is option B: an owner is
//! handed protected bytes, a census of signing inputs, and takes back
//! opaque authorization bytes plus the hash-type byte used, bound to one
//! candidate identity. This module is the census half.
//!
//! The census exists because the target's message is not a function of
//! the candidate alone. The source review read the message term by term
//! at the pinned Elements tip and found nineteen terms, of which four
//! groups come from outside the candidate's own serialization: the
//! deployment's genesis block hash, which seeds the hasher twice; the
//! spent outputs' asset, value and script fields, which three separate
//! terms cover; the output-witness vector at its consensus length, which
//! one term covers; and the executing leaf's hash, key version and
//! codeseparator position, which the script path adds. A component
//! handed only [`crate::live_finalize::FinalizedLiveTransfer::protected_bytes`]
//! cannot form the message for any input, and the concept's own
//! correction to the consuming guide's §4.1 says so in five named
//! places.
//!
//! # What the census excludes, as a rule and not an omission
//!
//! No opening, no blinder, no nonce input, no key, and no proof input
//! enters the census under any lane. The exclusion is what keeps a
//! secret-bearing interface from being selected at all, so ADR-015's
//! design gate is never reached and the whole census is public test
//! material in the sense `(´[ADR015-rule:security:test-material]´)`
//! fixes: every value it touches — every key, chain, genesis hash and
//! census entry — is disposable regtest material authorizing nothing
//! anywhere else.
//!
//! The exclusion is enforced by shape rather than by review. There is no
//! field an opening could be written into, and the one place a key would
//! naturally appear — the control block, whose second field is the
//! taproot internal key — is a *constructor argument* that the leaf
//! commitment check consumes and drops. Nothing key-shaped is retained.
//!
//! # Why there is no public constructor
//!
//! [`OwnerSigningCensus`] is reachable from exactly one public route,
//! [`OwnerSigningCensus::from_proof_finalized`], and that route takes a
//! materialized confidential candidate. The pattern is the finalized
//! form's own: private fields, immutable accessors, and a constructor
//! whose argument cannot be obtained except by going through
//! finalization.
//!
//! The route takes the materialized value rather than the frozen
//! candidate inside it because the frozen candidate does not carry the
//! spent-output census, and a caller assembling one beside it would be
//! the route that skips finalization — a census whose spent outputs came
//! from somewhere other than the run that produced the candidate is
//! exactly the deployment-and-cardinality confusion the refusals below
//! exist to catch.
//!
//! # What this module does not do
//!
//! It computes no digest, and it asserts none. Forming a candidate
//! message from a census is [`crate::live_message`]'s, observing what a
//! target does with one is a later wave's, and neither
//! `SighashProfileUnreviewed` nor `OwnerSighashNotComputable` is
//! affected by anything here. A capability existing moves no standing.

use std::collections::BTreeSet;

use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::bytes::{AssetField, OutputWitness, TargetTransaction, ValueField};
use crate::live_materialize::MaterializedConfidentialCandidate;
use crate::live_taproot::LiveCurveCapability;
use crate::taproot::{
    CONTROL_BASE_BYTES, DIGEST_BYTES, Digest32, TAPROOT_LEAF_MASK, TAPROOT_WITNESS_VERSION,
    branch_hash, witness_program_script,
};

// --- The profile's constants ------------------------------------------

/// The hash-type byte the selected profile fixes.
///
/// The profile admits the default hash type only, so the byte is `0x00`
/// and a returned byte that disagrees is a refusal rather than a
/// variant. The two are not two spellings of one thing:
/// `src/script/interpreter.cpp:2722` writes the type byte into the
/// message, so `0x00` and `0x01` produce different messages, and
/// `:2958-2966` refuses a 65-byte signature whose trailing byte is
/// zero — which is why the default type is the one with no trailing
/// byte at all.
pub const OWNER_SIGHASH_TYPE_BYTE: u8 = 0x00;

/// The spend-type byte every candidate this arc produces carries.
///
/// `src/script/interpreter.cpp:2748` computes it as the extension flag
/// shifted left by one, with the annex bit added. The profile is script
/// path only, so the extension flag is 1 (`:2691-2706`), and the profile
/// refuses the annex, so the annex bit is 0. Two rulings together make
/// one byte a constant a test asserts rather than an input a builder
/// supplies.
pub const OWNER_SPEND_TYPE_BYTE: u8 = 0x02;

/// The width of an authorizing signature under the selected profile.
///
/// Sixty-four, because a 65-byte signature is one carrying a trailing
/// hash-type byte and the default type carries none. This is also the
/// width every already-measured weight in the workspace was taken at, so
/// the profile ruling leaves every recorded figure true.
pub const OWNER_SIGNATURE_BYTES: usize = 64;

/// The key version the message carries for every tapscript spend.
///
/// A constant fixed by the target rather than by the profile:
/// `src/script/interpreter.cpp:2702` sets it for every tapscript spend
/// and the comment there says an upgraded key version would arrive as a
/// new signature version rather than as a different byte here.
pub const OWNER_KEY_VERSION_BYTE: u8 = 0x00;

/// The codeseparator position every leaf this arc emits reports.
///
/// `src/script/interpreter.cpp:581` sets this value at the head of
/// evaluation and only `:1472` moves it, and no leaf in this workspace
/// emits the opcode that moves it. That makes the constancy a condition
/// on the leaf vocabulary rather than a property of the target, which is
/// why the census carries the field rather than assuming the condition —
/// this constant is the value a test compares against, not a value the
/// census substitutes.
pub const OWNER_CODESEPARATOR_POSITION: u32 = 0xffff_ffff;

/// The deepest control path a census admits.
///
/// The target's own bound on a taproot merkle path.
const MAXIMUM_CONTROL_PATH_DEPTH: usize = 128;

// --- Declared dispositions --------------------------------------------

/// Whether a signing input's witness will carry an annex.
///
/// Two members and not a boolean, because the profile refuses one of
/// them and a refusal needs something to refuse. The selected profile
/// admits [`Self::Absent`] only, which is what fixes
/// [`OWNER_SPEND_TYPE_BYTE`].
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
/// name (`rule:sighash-review:census-consequence`). The accepted result
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
/// [`OwnerCensusRefusal::IssuanceBearingInputRefused`], so the census's
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

// --- One signing input -------------------------------------------------

/// What a caller asks the census for, per signing input.
///
/// Distinct from [`OwnerSigningInputCensus`], which is what the census
/// *retains*, and the difference is the point. The control block is here
/// and not there: the leaf commitment check consumes it and drops it, so
/// the internal key inside it never becomes a census field. A census
/// that carried the control block would be carrying a key, and the
/// exclusion rule is enforced by there being no field rather than by a
/// reviewer noticing.
///
/// The issuance disposition is here and not there for a different
/// reason: only [`IssuanceDisposition::Absent`] ever survives
/// construction, so retaining it would be storing a constant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerSigningInputRequest {
    input_index: u32,
    tapleaf_hash: Digest32,
    leaf_version: LeafVersion,
    codeseparator_position: u32,
    annex: AnnexDisposition,
    issuance: IssuanceDisposition,
    control_block: Vec<u8>,
}

impl OwnerSigningInputRequest {
    /// The request to authorize one input under one leaf.
    #[must_use]
    pub const fn new(
        input_index: u32,
        tapleaf_hash: Digest32,
        leaf_version: LeafVersion,
        codeseparator_position: u32,
        annex: AnnexDisposition,
        issuance: IssuanceDisposition,
        control_block: Vec<u8>,
    ) -> Self {
        Self {
            input_index,
            tapleaf_hash,
            leaf_version,
            codeseparator_position,
            annex,
            issuance,
            control_block,
        }
    }

    /// Which input is being authorized.
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.input_index
    }

    /// The executing leaf's hash.
    #[must_use]
    pub const fn tapleaf_hash(&self) -> &Digest32 {
        &self.tapleaf_hash
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
    pub fn control_block(&self) -> &[u8] {
        &self.control_block
    }
}

/// One signing input, as the census retains it.
///
/// Exactly the five per-input members the accepted option-B result
/// names: the input index, the tapleaf hash, the leaf version, the
/// codeseparator position, and the annex disposition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerSigningInputCensus {
    input_index: u32,
    tapleaf_hash: Digest32,
    leaf_version: LeafVersion,
    codeseparator_position: u32,
    annex: AnnexDisposition,
}

impl OwnerSigningInputCensus {
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

// --- The refusals ------------------------------------------------------

/// Why a census could not be assembled.
///
/// Closed, with no catch-all. Every member is a *construction* refusal
/// and none is a target verdict: nothing here says what a node would do
/// with anything, and a run refused for one of these reasons is not
/// evidence about a chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerCensusRefusal {
    /// The spent-output census does not have one entry per input.
    ///
    /// Terms 6 and 7 are taken over the whole spent-output set, so a
    /// census short by one entry forms a message over a different set
    /// than the target will. Failure-matrix row 9.
    SpentOutputCardinalityMismatch {
        /// How many inputs the candidate has.
        inputs: usize,
        /// How many spent outputs the census offered.
        spent_outputs: usize,
    },
    /// The output-witness vector is not at its consensus length.
    ///
    /// The sharpest refusal in the list, and the one the recorded
    /// `G11-W11-06` diagnosis is about. `GetOutputWitnessesSHA256` at
    /// `src/script/interpreter.cpp:2418-2425` iterates the vector and
    /// hashes whatever entries are there; it takes no index and consults
    /// no cardinality. A vector at the wrong length is a different
    /// message, and the signature over it is complete and invalid.
    /// Failure-matrix row 2.
    OutputWitnessLengthMismatch {
        /// How many outputs the candidate has.
        outputs: usize,
        /// How many output-witness entries the census offered.
        output_witnesses: usize,
    },
    /// The declared annex disposition does not agree with the profile.
    ///
    /// The profile refuses the annex, which is what fixes the spend-type
    /// byte at [`OWNER_SPEND_TYPE_BYTE`]. A declared annex recomputes
    /// that byte to something else, and the refusal carries the
    /// recomputed value rather than merely naming the disagreement.
    /// Failure-matrix row 6.
    AnnexDisagreement {
        /// Which input declared it.
        input_index: u32,
        /// What the request declared.
        declared: AnnexDisposition,
        /// The spend-type byte that declaration recomputes to.
        recomputed_spend_type: u8,
    },
    /// The census is bound to a different deployment than the run.
    ///
    /// The hasher is seeded with the genesis block hash twice, so a
    /// signature produced against one deployment would be complete and
    /// refused on another. Failure-matrix row 4.
    DeploymentMismatch {
        /// The deployment record's genesis block hash.
        expected: Digest32,
        /// The genesis block hash the census offered.
        offered: Digest32,
    },
    /// The executing leaf does not commit under the control block.
    ///
    /// The merkle root folded from the declared tapleaf hash and the
    /// control block's path, tweaked onto the control block's internal
    /// key, is not the output key in the spent output's program. This is
    /// `VerifyTaprootCommitment` at `src/script/interpreter.cpp:3217-3229`,
    /// recomputed on this side: the target refuses such a witness with a
    /// witness-program mismatch at `:3288-3290`, before any signature is
    /// checked and before the leaf executes. Failure-matrix row 8.
    LeafHashDoesNotCommit {
        /// Which input's leaf failed to commit.
        input_index: u32,
    },
    /// The control block is not a version-and-parity byte, an x-only
    /// internal key, and a whole number of path elements within the
    /// target's depth bound.
    ControlBlockMalformed {
        /// Which input carried it.
        input_index: u32,
        /// How many bytes it carried.
        offered: usize,
    },
    /// The control block's leaf version does not match the declared one.
    ///
    /// The control block's first byte carries the leaf version in its
    /// top seven bits and the output key's parity in its lowest, so a
    /// declared version disagreeing with it means the tapleaf hash was
    /// taken over a version the witness does not claim.
    LeafVersionDisagreesWithTheControlBlock {
        /// Which input carried them.
        input_index: u32,
        /// The version the request declared.
        declared: u8,
        /// The version-and-parity byte the control block carried.
        control_byte: u8,
    },
    /// The internal key in the control block is not a point of the
    /// target's curve, or the tweak it names has no output key.
    ControlBlockInternalKeyIsNotAPoint {
        /// Which input carried it.
        input_index: u32,
    },
    /// A hash-type byte outside the selected profile.
    ///
    /// `src/script/interpreter.cpp:2721` admits `0x00..=0x03` and
    /// `0x81..=0x83` and refuses the rest as a hash-type script error,
    /// but the profile is narrower than the target: it admits
    /// [`OWNER_SIGHASH_TYPE_BYTE`] alone. A returned `0x01` under a
    /// default-only profile is a refusal and not a variant, because the
    /// two produce different messages and a report accepting both would
    /// be naming a profile it did not hold to. Failure-matrix row 5.
    TypeByteOutsideProfile {
        /// The byte offered.
        offered: u8,
    },
    /// A signature width outside the selected profile.
    ///
    /// Sixty-four bytes under the default type, and the target's own
    /// answers to the two mistakes differ: a wrong width is a signature
    /// size error, while a 65-byte signature with a trailing zero is a
    /// hash-type error at `:2958-2966`. Failure-matrix row 10.
    SignatureWidthOutsideProfile {
        /// The width offered.
        offered: usize,
    },
    /// A signing input declared an issuance the census carries no field
    /// for.
    ///
    /// See [`IssuanceDisposition`]: the census's silence about the
    /// input-witness vector is a claim resting on the precondition that
    /// no input bears an issuance, and this is the refusal that makes
    /// the precondition checked rather than assumed.
    IssuanceBearingInputRefused {
        /// Which input declared it.
        input_index: u32,
    },
    /// A signing request names an input the candidate does not have.
    SigningInputOutOfRange {
        /// The index requested.
        input_index: u32,
        /// How many inputs the candidate has.
        inputs: usize,
    },
    /// Two signing requests name the same input.
    ///
    /// One request per input, as the finalized form already builds: two
    /// would be two messages for one authorization, and the second would
    /// silently decide which leaf the first was taken under.
    DuplicateSigningInput {
        /// The index requested twice.
        input_index: u32,
    },
    /// No input was requested at all.
    ///
    /// A census with nothing to authorize is not a narrower census; it
    /// is a value whose only use would be to report that a handoff
    /// happened.
    NoSigningInputRequested,
    /// The protected bytes offered are not the candidate's own.
    ///
    /// The binding is exact-byte in the pattern
    /// [`crate::live_finalize::FinalizedLiveTransfer::check_offered`]
    /// already sets: an answer is checked by comparing bytes, not by
    /// trusting that the signer looked.
    ProtectedBytesAreNotTheCandidates,
}

// --- The census --------------------------------------------------------

/// Everything the target reads to form one candidate's owner messages,
/// and nothing else.
///
/// Private fields, immutable accessors, and no public constructor. The
/// one public route is [`Self::from_proof_finalized`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerSigningCensus {
    candidate: TargetTransaction,
    protected_bytes: Vec<u8>,
    output_witnesses: Vec<OutputWitness>,
    spent_outputs: Vec<SpentOutputCensusEntry>,
    genesis_block_hash: Digest32,
    signing_inputs: Vec<OwnerSigningInputCensus>,
}

impl OwnerSigningCensus {
    /// The census of one materialized confidential candidate.
    ///
    /// The only public route to a census, and it takes a value the
    /// materializer alone produces, so there is no route that skips
    /// finalization.
    ///
    /// # Errors
    ///
    /// [`OwnerCensusRefusal`], at the first clause the request fails.
    /// Every one is a construction refusal and none is a target verdict.
    pub fn from_proof_finalized(
        target: &ReviewedElementsTapscriptDefinition,
        materialized: &MaterializedConfidentialCandidate,
        deployment: LiveDeployment,
        requests: &[OwnerSigningInputRequest],
        curve: &dyn LiveCurveCapability,
    ) -> Result<Self, OwnerCensusRefusal> {
        let frozen = materialized.proof_finalized();
        let candidate = frozen.protected().clone();
        let output_witnesses = candidate.output_witnesses().to_vec();
        let spent_outputs = materialized
            .signer_inputs()
            .iter()
            .map(|input| {
                SpentOutputCensusEntry::new(
                    input.spent_asset(),
                    input.spent_value(),
                    input.spent_program().to_vec(),
                )
            })
            .collect::<Vec<_>>();

        Self::assemble(
            target,
            candidate,
            frozen.protected_bytes().to_vec(),
            output_witnesses,
            spent_outputs,
            deployment,
            requests,
            curve,
        )
    }

    /// The census of parts a test supplies directly.
    ///
    /// An authorized, bounded seam and not a second public route: it is
    /// crate-private *and* test-only — the attribute is what keeps it
    /// from being a route at all outside a test build — it runs the same
    /// clause list as the public route rather than a relaxed one, and
    /// its whole purpose is that the refusals above can be shown firing.
    /// Several of them are
    /// structurally unreachable from a materialized candidate — a frozen
    /// candidate's output-witness vector is one entry per output by the
    /// transaction type's own invariant, and its protected bytes are the
    /// bytes it computed — and a refusal that no test can reach is a
    /// refusal nobody has checked says what it says.
    ///
    /// # Errors
    ///
    /// [`OwnerCensusRefusal`], at the first clause the parts fail.
    #[cfg(test)]
    #[expect(
        clippy::too_many_arguments,
        reason = "the seam takes the assembled clause list's own arguments; \
                  bundling them into a parts struct would give the seam a \
                  shape the public route does not have"
    )]
    pub(crate) fn from_parts(
        target: &ReviewedElementsTapscriptDefinition,
        candidate: TargetTransaction,
        protected_bytes: Vec<u8>,
        output_witnesses: Vec<OutputWitness>,
        spent_outputs: Vec<SpentOutputCensusEntry>,
        deployment: LiveDeployment,
        requests: &[OwnerSigningInputRequest],
        curve: &dyn LiveCurveCapability,
    ) -> Result<Self, OwnerCensusRefusal> {
        Self::assemble(
            target,
            candidate,
            protected_bytes,
            output_witnesses,
            spent_outputs,
            deployment,
            requests,
            curve,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one clause list shared by both routes; splitting it would \
                  let the two routes come to check different things"
    )]
    fn assemble(
        target: &ReviewedElementsTapscriptDefinition,
        candidate: TargetTransaction,
        protected_bytes: Vec<u8>,
        output_witnesses: Vec<OutputWitness>,
        spent_outputs: Vec<SpentOutputCensusEntry>,
        deployment: LiveDeployment,
        requests: &[OwnerSigningInputRequest],
        curve: &dyn LiveCurveCapability,
    ) -> Result<Self, OwnerCensusRefusal> {
        let inputs = candidate.inputs().len();
        let outputs = candidate.outputs().len();

        if spent_outputs.len() != inputs {
            return Err(OwnerCensusRefusal::SpentOutputCardinalityMismatch {
                inputs,
                spent_outputs: spent_outputs.len(),
            });
        }

        if output_witnesses.len() != outputs {
            return Err(OwnerCensusRefusal::OutputWitnessLengthMismatch {
                outputs,
                output_witnesses: output_witnesses.len(),
            });
        }

        if candidate.output_witnesses() != output_witnesses.as_slice() {
            return Err(OwnerCensusRefusal::OutputWitnessLengthMismatch {
                outputs,
                output_witnesses: output_witnesses.len(),
            });
        }

        if requests.is_empty() {
            return Err(OwnerCensusRefusal::NoSigningInputRequested);
        }

        let mut seen = BTreeSet::new();
        let mut signing_inputs = Vec::with_capacity(requests.len());

        for request in requests {
            let index = request.input_index();
            let position = usize::try_from(index)
                .ok()
                .filter(|position| *position < inputs)
                .ok_or(OwnerCensusRefusal::SigningInputOutOfRange {
                    input_index: index,
                    inputs,
                })?;

            if !seen.insert(index) {
                return Err(OwnerCensusRefusal::DuplicateSigningInput { input_index: index });
            }

            if request.issuance() == IssuanceDisposition::Bearing {
                return Err(OwnerCensusRefusal::IssuanceBearingInputRefused { input_index: index });
            }

            let recomputed = spend_type_byte(request.annex());
            if recomputed != OWNER_SPEND_TYPE_BYTE {
                return Err(OwnerCensusRefusal::AnnexDisagreement {
                    input_index: index,
                    declared: request.annex(),
                    recomputed_spend_type: recomputed,
                });
            }

            check_leaf_commits(target, request, &spent_outputs[position], curve)?;

            signing_inputs.push(OwnerSigningInputCensus {
                input_index: index,
                tapleaf_hash: *request.tapleaf_hash(),
                leaf_version: request.leaf_version(),
                codeseparator_position: request.codeseparator_position(),
                annex: request.annex(),
            });
        }

        Ok(Self {
            candidate,
            protected_bytes,
            output_witnesses,
            spent_outputs,
            genesis_block_hash: *deployment.genesis_block_hash(),
            signing_inputs,
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
    pub fn signing_inputs(&self) -> &[OwnerSigningInputCensus] {
        &self.signing_inputs
    }

    /// Whether an offered candidate is the one this census is bound to.
    ///
    /// # Errors
    ///
    /// [`OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates`] when
    /// the offered candidate's own protected preimage is not the bytes
    /// this census carries. Exact bytes compared, in the pattern
    /// [`crate::live_finalize::FinalizedLiveTransfer::check_offered`]
    /// sets, rather than trusting that the signer looked.
    pub fn check_offered(&self, offered: &[u8]) -> Result<(), OwnerCensusRefusal> {
        if offered == self.protected_bytes {
            Ok(())
        } else {
            Err(OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates)
        }
    }

    /// Whether this census belongs to the deployment a run is against.
    ///
    /// Failure-matrix row 4, and the check has to be spelled because
    /// nothing about a candidate reveals which chain it was censused
    /// for. The hasher is seeded with the genesis block hash twice, so
    /// two candidates identical to the last byte have different messages
    /// on two chains: a census carried to the wrong run produces a
    /// signature that is complete, well formed, and refused on chain,
    /// with no earlier symptom at all.
    ///
    /// # Errors
    ///
    /// [`OwnerCensusRefusal::DeploymentMismatch`] when the run's
    /// deployment is not the one the census was built against.
    pub fn check_deployment(&self, run: LiveDeployment) -> Result<(), OwnerCensusRefusal> {
        let offered = *run.genesis_block_hash();

        if offered == self.genesis_block_hash {
            Ok(())
        } else {
            Err(OwnerCensusRefusal::DeploymentMismatch {
                expected: self.genesis_block_hash,
                offered,
            })
        }
    }
}

// --- Profile checks a returned answer must pass -----------------------

/// Whether a returned hash-type byte is the one the profile selected.
///
/// The accepted result's third constraint: the returned byte is
/// *checked* against the profile, not recorded. A returned `0x01` under
/// a default-only profile is a refusal and not a variant.
///
/// # Errors
///
/// [`OwnerCensusRefusal::TypeByteOutsideProfile`] for any other byte,
/// including the ones the target itself admits.
pub const fn check_type_byte(offered: u8) -> Result<(), OwnerCensusRefusal> {
    if offered == OWNER_SIGHASH_TYPE_BYTE {
        Ok(())
    } else {
        Err(OwnerCensusRefusal::TypeByteOutsideProfile { offered })
    }
}

/// Whether a returned authorization is the width the profile fixes.
///
/// # Errors
///
/// [`OwnerCensusRefusal::SignatureWidthOutsideProfile`] for any other
/// width, including the 65 a non-default type byte would carry.
pub const fn check_signature_width(offered: usize) -> Result<(), OwnerCensusRefusal> {
    if offered == OWNER_SIGNATURE_BYTES {
        Ok(())
    } else {
        Err(OwnerCensusRefusal::SignatureWidthOutsideProfile { offered })
    }
}

/// The spend-type byte one annex disposition recomputes to.
///
/// `src/script/interpreter.cpp:2748`: the extension flag shifted left by
/// one, plus the annex bit. The extension flag is 1 for every spend this
/// profile admits, because the profile is script path only.
#[must_use]
pub const fn spend_type_byte(annex: AnnexDisposition) -> u8 {
    const SCRIPT_PATH_EXTENSION_FLAG: u8 = 1;

    let annex_bit = match annex {
        AnnexDisposition::Absent => 0,
        AnnexDisposition::Present => 1,
    };

    (SCRIPT_PATH_EXTENSION_FLAG << 1_u8) | annex_bit
}

// --- The control-block check ------------------------------------------

/// `VerifyTaprootCommitment`, recomputed on this side.
fn check_leaf_commits(
    target: &ReviewedElementsTapscriptDefinition,
    request: &OwnerSigningInputRequest,
    spent: &SpentOutputCensusEntry,
    curve: &dyn LiveCurveCapability,
) -> Result<(), OwnerCensusRefusal> {
    let index = request.input_index();
    let block = request.control_block();

    if block.len() < CONTROL_BASE_BYTES
        || !(block.len() - CONTROL_BASE_BYTES).is_multiple_of(DIGEST_BYTES)
        || (block.len() - CONTROL_BASE_BYTES) / DIGEST_BYTES > MAXIMUM_CONTROL_PATH_DEPTH
    {
        return Err(OwnerCensusRefusal::ControlBlockMalformed {
            input_index: index,
            offered: block.len(),
        });
    }

    let control_byte = block[0];
    if control_byte & TAPROOT_LEAF_MASK != request.leaf_version().get() & TAPROOT_LEAF_MASK {
        return Err(
            OwnerCensusRefusal::LeafVersionDisagreesWithTheControlBlock {
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

    let output_key = curve
        .output_key(internal_key, &root)
        .ok_or(OwnerCensusRefusal::ControlBlockInternalKeyIsNotAPoint { input_index: index })?;

    if output_key.parity().bit() != control_byte & !TAPROOT_LEAF_MASK {
        return Err(OwnerCensusRefusal::LeafHashDoesNotCommit { input_index: index });
    }

    // Rebuilt through the crate's own reviewed witness-program grammar
    // rather than parsed out of the spent script. Parsing would have
    // meant a second opinion about which opcode a version-one program
    // starts with, and the two could come to differ.
    let expected = witness_program_script(target, TAPROOT_WITNESS_VERSION, output_key.key())
        .map_err(|_| OwnerCensusRefusal::LeafHashDoesNotCommit { input_index: index })?;

    if expected != spent.program() {
        return Err(OwnerCensusRefusal::LeafHashDoesNotCommit { input_index: index });
    }

    Ok(())
}
