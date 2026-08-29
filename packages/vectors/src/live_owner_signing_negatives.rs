//! The owner-signing negative ceremony: a script-path §15 negative signed
//! over its own mutated bytes, so its refusal is the leaf's own clause
//! rather than the signature gate.
//!
//! # What this ceremony is, and why the route it runs on exists
//!
//! Every §15 row the negative-half register carries under
//! [`crate::live_negative_half::NegativeHalfGap::OwnerSigningOverForeignBytesAbsent`]
//! declares a SCRIPT-PATH refusal. Reaching one needs the target to RUN
//! the leaf; the leaf checks an owner signature first; and byte surgery
//! after signing invalidates that signature — so a mutant carrying a stale
//! signature dies at the signature gate and the row's own class is never
//! what refused. This ceremony breaks that deadlock: it does the surgery,
//! then builds a census over the MUTATED bytes through
//! [`transaction::live_census::OwnerSigningCensus::over_foreign_bytes_for_negative_evidence`],
//! forms the owner message from it, and re-signs afresh. The re-signed
//! mutant passes `OP_CHECKSIGVERIFY` and reaches the leaf's own clause.
//!
//! # The script-path row this ceremony drives
//!
//! `vault-control-entitlement-or-bare-u-output`. The coordinator leaf's
//! `InspectOutputScriptPubKey` clause constrains the WITNESS VERSION of
//! every destination output — and only the version, the 32-byte payload
//! being dropped. Mutating one destination's program to a different-version
//! ("bare-u") program, while keeping its asset and its explicit value,
//! leaves the per-asset consensus tally intact, so the mutant passes
//! `bad-txns-in-ne-out` and reaches the version clause, which refuses it.
//!
//! That is the CLEAN shape the governing per-row fact names: a re-signed
//! mutant authorizes its own outputs, so a row is drivable to its
//! script-path class only where a covenant clause constrains the mutated
//! field INDEPENDENT of the signature AND the consensus balance rule does
//! not fire first. An asset or a single-output value surgery breaks the
//! per-asset sum and is refused `bad-txns-in-ne-out` before the leaf runs.
//!
//! # The consensus rows this ceremony drives
//!
//! That same consensus verdict is itself an attributable observation when
//! it separates by field, which is the `private-ct-imbalance` precedent.
//! So beside the bare-u script mutant this ceremony stages SEVEN
//! consensus-conservation mutants on the same signed control — one per §15
//! row whose fault breaks the explicit per-asset sum: a wrong explicit
//! asset, a blinded asset commitment with no surjection proof, a value one
//! below and one above the input total, an omitted destination, an added
//! hidden output, and an omitted source input. Each is cut from the signed
//! control's OWN bytes and is NOT re-signed — the consensus balance check
//! is queued before script verification, so a broken tally refuses the
//! mutant before its stale signature is examined, the fact the conservation
//! ceremony's proof-negatives already rest on. The separating fact is the
//! declared byte RANGE together with the transaction SHAPE, and it takes
//! both: the four field surgeries — the two asset fields and the two value
//! fields — keep the control's 2-in-2-out shape and separate by four
//! distinct ranges, while the three structural surgeries change the shape,
//! and two of those three declare the SAME range because `changed_range`
//! cannot localize an insertion or a deletion past the output-count varint.
//! `private-output-omitted` and `hidden-private-u-output` both declare
//! `88..245` and separate by their shapes, `(2, 1)` against `(2, 3)`. The
//! `(range, shape)` tuple is distinct across all seven, so no two rows rest
//! on one observation.
//!
//! # The leaf-arrangement rows this ceremony drives
//!
//! The four leaf-arrangement rows form TWO collision pairs, each drawing
//! ONE verdict: the coordinator index check (`PushCurrentInputIndex; 0;
//! EqualVerify`) and the member bound check (`... 1; GreaterThanOrEqual64;
//! Verify`). So beside the field mutants this ceremony stages TWO
//! leaf-arrangement mutants — one per pair, the arrangement whose only
//! failing input is the leaf at the forbidden position. `two-coordinators`
//! reveals the coordinator leaf at both inputs, so the coordinator running
//! at input one fails the index `EqualVerify`; `no-coordinator` reveals a
//! member leaf at both inputs, so the member running at input zero fails
//! the bound Verify. The other half of each pair (`wrong-coordinator`,
//! `member-coordinator-leaf-exchange`) stays typed because its own
//! arrangement carries a SECOND failing input, so it has no separating
//! fact of its own against the pair-partner already driven. Which of the
//! two failures a target would report for such a candidate is NOT claimed:
//! abort selection across a multi-input candidate is the target's, and no
//! in-repo source settles it.
//!
//! These mutants move NO taptree. Both funded receipts are paid to one
//! program, so both spent outputs commit to one taptree holding both the
//! coordinator leaf and the member leaf; a leaf-arrangement mutant reveals
//! an already-committed leaf at a different input, and the census's own
//! recomputed `VerifyTaprootCommitment` accepts the rearranged control
//! block because it commits against the same taproot output. Nothing but
//! the witness changes, so the witnessless serialization stays the
//! control's and the separating fact is the revealed-leaf ARRANGEMENT. Each
//! input is re-signed over the rearranged census, so the candidate passes
//! the signature gate and reaches the leaf's own index or bound clause.
//!
//! # Attribution is by mutated field, against a control on the same chain
//!
//! The mutant is offered FIRST and the unmutated control LAST, to one node
//! on one chain — a refused mutant spends nothing, so the control's coins
//! stay unspent for its acceptance. The two candidates differ in exactly
//! one field of the witnessless serialization: the mutated destination's
//! program. Every other output field is held fixed, so the `EQUALVERIFY`
//! the leaf fails is the one the destination's scriptPubKey-version clause
//! carries and no other. The signatures differ too, and by design: the
//! whole point of the route is that the mutant is re-signed over its own
//! mutated bytes. The declared field range is therefore measured over the
//! WITNESSLESS serialization, where the re-signing does not reach.
//!
//! # Every key here is published test material
//!
//! The signing scalars are the BIP-340 appendix secret keys, admitted
//! under ADR-015's test-material rule `(´[ADR015-rule:security:test-material]´)`.
//! They authorize nothing on any network anyone uses, the chain this
//! ceremony runs against is created and destroyed by the run, and nothing
//! here is custody of anything.

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::LeafVersion;
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    FundedOutput, MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, OutputWitness, TargetOutput,
    TargetTransaction, ValueField,
};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{
    AnnexDisposition, IssuanceDisposition, LiveDeployment, OWNER_CODESEPARATOR_POSITION,
    OwnerCensusRefusal, OwnerSigningCensus, OwnerSigningInputRequest,
};
use transaction::live_construct::finalize_live_transfer;
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::taproot::{CONTROL_BASE_BYTES, DIGEST_BYTES, Digest32, leaf_hash};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{
    ObservedFundedCoin, asset_of, decode_hex, outpoint_of, printed,
};
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, RESERVE_ASSET, SECOND_SCALAR, THIRD_SCALAR,
    demonstration_live_abi, live_abi_for_asset, published_owner, reviewed_target, signing_material,
};
use crate::live_report::LiveMutationLocator;

/// What each funded receipt holds.
///
/// Equal to the owner-observation ceremony's figure deliberately: the
/// control this ceremony accepts is the same candidate that ceremony
/// accepts, and a different amount would make the two runs incomparable.
const RECEIPT_AMOUNT: u64 = 5_000;

/// How many receipts the ceremony funds and then consumes.
const RECEIPT_COUNT: u8 = 2;

/// The auxiliary value every signature here is taken with.
///
/// A published constant, not randomness: BIP-340 masks the scalar with it
/// before deriving the nonce, so fixing it makes every signature
/// reproducible from written-down inputs.
const SIGNING_AUXILIARY: [u8; FIELD_ELEMENT_BYTES] = [0x33; FIELD_ELEMENT_BYTES];

/// The output the mutant rewrites the program of.
///
/// The first destination, which the coordinator leaf at input zero
/// inspects along with every other destination. Any destination would
/// serve; the point is that ONE output's program is rewritten and the
/// asset and the value are left alone.
const MUTATED_OUTPUT: usize = 0;

/// A different-version program the mutant pays the first destination to.
///
/// A version-zero witness program (the P2WPKH shape) where the receipt
/// constructor's is version one. `InspectOutputScriptPubKey` pushes the
/// version, the coordinator leaf checks it against the destination version
/// one, and a version-zero program fails that `EQUALVERIFY` — which is the
/// "bare-u output" the row names. It is a standard output, so the node
/// reaches the leaf rather than refusing it as nonstandard, and it carries
/// the SAME asset and the SAME explicit value, so the per-asset tally is
/// undisturbed.
const BARE_U_PROGRAM: [u8; 22] = [
    0x00, 0x14, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
];

/// The ceremony's own name for the mutant submission.
pub const MUTANT_STEP: &str = "bare-u-output-mutant";

/// The ceremony's own name for the control submission.
pub const CONTROL_STEP: &str = "vault-control-entitlement-control";

/// The value the added hidden output carries, in the protocol asset.
///
/// Any positive amount serves: it raises the protocol-asset output sum
/// above the input sum, which is the conservation break the row names.
const HIDDEN_OUTPUT_AMOUNT: u64 = 1_000;

/// The output the two value-total surgeries and the two asset surgeries
/// place their mutations at, chosen so each row earns a DISTINCT declared
/// field range on the one explicit control.
///
/// The two asset surgeries sit at the two receipt outputs' asset fields,
/// the two value surgeries at the two receipts' value fields — four
/// distinct field regions on one transaction, the `private-ct-imbalance`
/// discipline applied across the explicit successor. The structural
/// surgeries change the shape and separate by that.
const FIRST_RECEIPT: usize = 0;
const SECOND_RECEIPT: usize = 1;

/// The input the omitted-source surgery drops.
const DROPPED_INPUT: usize = 1;

/// The explicit value the out-of-domain surgery writes.
///
/// Two to the fifty-first, the exclusive upper limit of the protocol's
/// own amount domain and above the reviewed target's money ceiling, so
/// `CheckTransaction` refuses the output as too large before the balance
/// rule is reached and before any script runs. It is written ABSOLUTELY
/// rather than as a delta: a value one below or above the input total is
/// a different fault at the same field, and only an absolute write puts
/// the field outside the domain regardless of what the control held.
const OUT_OF_DOMAIN_VALUE: u64 = 1 << 51;

/// The input whose control block the witness surgery malforms.
const MALFORMED_CONTROL_INPUT: usize = 0;

/// The witness-stack position the control block occupies.
///
/// The ceremony assembles every script-path witness as `[signature,
/// leaf_script, control_block]`, so the control block is the third item.
const CONTROL_BLOCK_ITEM: usize = 2;

/// The ceremony's own name for the malformed control-path submission,
/// which is also the §15 row it drives.
///
/// The two coincide here where they do not for the other two families:
/// the consensus and leaf-arrangement steps each stage several rows and
/// take a prefix to keep their siblings apart, while this surgery stages
/// exactly one row and has no sibling to be separated from.
pub const MALFORMED_CONTROL_PATH_STEP: &str = "malformed-control-path";

/// The deepest merkle path a control block may carry.
///
/// Stated here because the census that enforces it keeps it private; the
/// head and path-entry sizes are the published ones and are taken from
/// the taproot module rather than restated, so this surgery cannot drift
/// away from the geometry the census checks.
const CONTROL_BLOCK_MAX_PATH_ENTRIES: usize = 128;

/// The byte the witness surgery appends to the control block.
///
/// Its VALUE is immaterial and deliberately so: the size test fires
/// before any byte of the path is read, so a zero states that the
/// surgery is about the LENGTH and nothing else. A byte chosen to mean
/// something would invite the mutant to be read as a path claim.
const CONTROL_BLOCK_PAD_BYTE: u8 = 0x00;

/// One consensus-conservation surgery this ceremony stages on the signed
/// explicit control, each breaking the explicit per-asset sum in its own
/// way so the target answers `bad-txns-in-ne-out` (or a surjection
/// verdict) at [`ObservedOutcomeLayer::ConsensusRejectionBeforeScript`]
/// before any covenant clause runs.
///
/// The mutants carry the control's OWN signatures unchanged: the surgery
/// is post-signature and the signature is not re-taken, because the
/// consensus balance check is queued before script verification, so the
/// tally is what refuses and a stale signature never gets examined — the
/// same fact the conservation ceremony's proof-negatives rest on. Each is
/// attributed by the byte range its mutation confined itself to in the
/// WITNESSLESS serialization, measured rather than asserted, disjoint or
/// at least distinct from every sibling's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsensusSurgery {
    /// Rewrite the first receipt's asset to a different explicit asset:
    /// the protocol-asset output sum falls one receipt short.
    WrongExplicitAsset,
    /// Replace the second receipt's explicit asset with a blinded asset
    /// commitment carrying no surjection proof: surjection fails.
    ConfidentialAssetCommitment,
    /// Lower the first receipt's explicit value by one: outputs fall one
    /// below inputs.
    OutputTotalOneBelowInput,
    /// Raise the second receipt's explicit value by one: outputs rise one
    /// above inputs.
    OutputTotalOneAboveInput,
    /// Delete the second receipt output entirely: the output sum drops.
    PrivateOutputOmitted,
    /// Append an undeclared protocol-asset output: the output sum rises.
    HiddenPrivateUOutput,
    /// Delete the second receipt input: the input sum drops.
    OmittedSource,
    /// Write the first receipt's explicit value outside the protocol's
    /// amount domain: the field itself is refused before the sum is
    /// taken.
    AmountOutsideSemanticDomain,
}

/// Every consensus surgery, in the order the ceremony submits them.
const CONSENSUS_SURGERIES: [ConsensusSurgery; 8] = [
    ConsensusSurgery::WrongExplicitAsset,
    ConsensusSurgery::ConfidentialAssetCommitment,
    ConsensusSurgery::OutputTotalOneBelowInput,
    ConsensusSurgery::OutputTotalOneAboveInput,
    ConsensusSurgery::PrivateOutputOmitted,
    ConsensusSurgery::HiddenPrivateUOutput,
    ConsensusSurgery::OmittedSource,
    ConsensusSurgery::AmountOutsideSemanticDomain,
];

impl ConsensusSurgery {
    /// The §15 row this surgery drives, spelled as the safety matrix names
    /// it.
    const fn row(self) -> &'static str {
        match self {
            Self::WrongExplicitAsset => "wrong-explicit-asset",
            Self::ConfidentialAssetCommitment => "confidential-asset-commitment",
            Self::OutputTotalOneBelowInput => "output-total-one-below-input",
            Self::OutputTotalOneAboveInput => "output-total-one-above-input",
            Self::PrivateOutputOmitted => "private-output-omitted",
            Self::HiddenPrivateUOutput => "hidden-private-u-output",
            Self::OmittedSource => "omitted-source",
            Self::AmountOutsideSemanticDomain => "amount-outside-semantic-domain",
        }
    }

    /// The ceremony's own name for this surgery's submission step.
    fn step_name(self) -> String {
        format!("consensus-{}", self.row())
    }

    /// Apply this surgery to the signed control, returning the mutant.
    fn apply(
        self,
        control: &TargetTransaction,
    ) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
        match self {
            Self::WrongExplicitAsset => rewrite_output_asset(
                control,
                FIRST_RECEIPT,
                AssetField::Explicit(AssetId::from_internal(RESERVE_ASSET)),
            ),
            Self::ConfidentialAssetCommitment => {
                let asset = control
                    .outputs()
                    .get(SECOND_RECEIPT)
                    .map(TargetOutput::asset)
                    .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
                rewrite_output_asset(control, SECOND_RECEIPT, blinded_asset_of(asset))
            }
            Self::OutputTotalOneBelowInput => adjust_output_value(control, FIRST_RECEIPT, -1),
            Self::OutputTotalOneAboveInput => adjust_output_value(control, SECOND_RECEIPT, 1),
            Self::PrivateOutputOmitted => remove_output(control, SECOND_RECEIPT),
            Self::HiddenPrivateUOutput => append_hidden_output(control),
            Self::OmittedSource => remove_input(control, DROPPED_INPUT),
            // The SAME field the one-below surgery lowers, written
            // absolutely instead of shifted. The two separate by the
            // measured range rather than by the field: a delta of one
            // moves the low-order bytes of the amount, while a write of
            // two to the fifty-first moves the high-order bytes and
            // leaves the low ones as they were, so the bounded diff
            // reports two different `(start, end)` pairs on one field.
            Self::AmountOutsideSemanticDomain => {
                set_output_value(control, FIRST_RECEIPT, OUT_OF_DOMAIN_VALUE)
            }
        }
    }
}

/// One leaf-arrangement surgery: which committed leaf each input reveals.
///
/// The control reveals the coordinator leaf at input zero and the member
/// leaf at input one; each mutant collapses that arrangement to ONE role,
/// so a leaf lands at a position its own covenant clause forbids. The
/// mutation lives entirely in the WITNESS — the outputs, inputs, assets
/// and values are the control's, so the witnessless serialization is
/// byte-identical to the control's and the separating fact is the
/// revealed-leaf arrangement itself, not a byte range.
///
/// Both funded receipts are paid to ONE program, so both spent outputs
/// commit to ONE taptree that holds both the coordinator leaf and the
/// member leaf. Revealing either leaf at either input therefore commits
/// against the same taproot output — the census's own recomputed
/// `VerifyTaprootCommitment` accepts the rearranged control block — so no
/// leaf-arrangement mutant needs a taptree of its own and none moves the
/// demonstration taptree's digest. Each input is re-signed over the
/// rearranged census through the seam, so the candidate passes the
/// signature gate and reaches the leaf's own index or bound clause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeafArrangement {
    /// Reveal the COORDINATOR leaf at both inputs. Input zero's
    /// coordinator is the control's and passes; input one's coordinator
    /// leaf runs at index one and fails the coordinator index check
    /// (`PushCurrentInputIndex; 0; EqualVerify`), the one input that
    /// fails, so the verdict is unambiguously that clause's.
    TwoCoordinators,
    /// Reveal a MEMBER leaf at both inputs. Input one's member is the
    /// control's and passes; input zero's member leaf runs at index zero
    /// and fails the member bound's lower check (`... 1;
    /// GreaterThanOrEqual64; Verify`), the one input that fails.
    NoCoordinator,
    /// EXCHANGE the two roles rather than collapsing them: input zero
    /// reveals receipt one's MEMBER leaf and input one reveals receipt
    /// zero's COORDINATOR leaf. Both inputs then run a leaf its own
    /// covenant clause forbids at that position — the member fails the
    /// bound's lower check at index zero and the coordinator fails the
    /// index `EqualVerify` at index one — so this arrangement carries
    /// TWO failing inputs where the other two carry one. That is why no
    /// verdict is predicted for it: which of two failures a target
    /// reports across a multi-input candidate is the target's own abort
    /// selection, and no source in this workspace settles it. What the
    /// row rests on instead is the ARRANGEMENT, which is `[1, 0]` and
    /// distinct from the control's and from both siblings'.
    MemberCoordinatorExchange,
}

/// Every leaf-arrangement surgery, in the order the ceremony submits them.
const LEAF_ARRANGEMENTS: [LeafArrangement; 3] = [
    LeafArrangement::TwoCoordinators,
    LeafArrangement::NoCoordinator,
    LeafArrangement::MemberCoordinatorExchange,
];

impl LeafArrangement {
    /// The §15 row this surgery drives, spelled as the safety matrix names
    /// it.
    const fn row(self) -> &'static str {
        match self {
            Self::TwoCoordinators => "two-coordinators",
            Self::NoCoordinator => "no-coordinator",
            Self::MemberCoordinatorExchange => "member-coordinator-leaf-exchange",
        }
    }

    /// The pair-partner this drive leaves without a separating fact, or
    /// `"none"` where the pair carries no such partner any more.
    ///
    /// Both collision pairs are now closed and neither closes by being
    /// left typed. `wrong-coordinator` stands adjudicated: its own
    /// arrangements are exhausted by faults already registered, so it is
    /// not an undriven partner waiting on a run. And
    /// `member-coordinator-leaf-exchange` is driven HERE, by the
    /// exchanged arrangement below, which is a distinct candidate from
    /// either collapse and rests on that arrangement rather than on a
    /// verdict. So every arrangement reports `"none"`, and the line is
    /// kept rather than dropped because that is the fact the drive
    /// establishes: this ceremony leaves no leaf-arrangement row behind
    /// it.
    const fn typed_partner(self) -> &'static str {
        match self {
            Self::TwoCoordinators | Self::NoCoordinator | Self::MemberCoordinatorExchange => "none",
        }
    }

    /// The receipt POSITION whose leaf each input reveals. The control's
    /// arrangement is `[0, 1]`; the two collapsing mutants reduce it to
    /// one role and the exchanging mutant swaps the two. The ceremony
    /// funds exactly two receipts, so the arrangement is two positions
    /// wide.
    const fn sources(self) -> [u16; RECEIPT_COUNT as usize] {
        match self {
            Self::TwoCoordinators => [0, 0],
            Self::NoCoordinator => [1, 1],
            Self::MemberCoordinatorExchange => [1, 0],
        }
    }

    /// The ceremony's own name for this surgery's submission step.
    fn step_name(self) -> String {
        format!("leaf-arrangement-{}", self.row())
    }
}

/// One leaf-arrangement mutant, as this ceremony built, submitted and
/// observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeafArrangementObservation {
    row: &'static str,
    typed_partner: &'static str,
    revealed_arrangement: Vec<u16>,
    mutant_bytes: Vec<u8>,
    submitted_bytes: usize,
    message: Digest32,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl LeafArrangementObservation {
    /// The §15 row this mutant drives.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The pair-partner this drive leaves typed with the non-separation.
    #[must_use]
    pub const fn typed_partner(&self) -> &'static str {
        self.typed_partner
    }

    /// The revealed-leaf arrangement: the source receipt position each
    /// input reveals the leaf of. The separating fact, since the
    /// witnessless serialization is the control's — a mutant's arrangement
    /// is distinct from the control's `[0, 1]` and from every sibling's.
    #[must_use]
    pub fn revealed_arrangement(&self) -> &[u16] {
        &self.revealed_arrangement
    }

    /// How many bytes this mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The message the rearranged input's signature was taken over.
    #[must_use]
    pub const fn message(&self) -> &Digest32 {
        &self.message
    }

    /// The layer the target refused this mutant at, where it was observed.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }
}

/// The malformed control-path mutant, as this ceremony built, submitted
/// and observed it.
///
/// The mutation is a WITNESS mutation and nothing else: one item of one
/// input's stack changes length and every other byte of the candidate,
/// witnessless serialization included, is the control's. That is why the
/// signature is not re-taken. The tapscript message commits to the
/// tapleaf hash — the leaf version and the leaf script — and not to the
/// control block's path bytes, so a control block of a different size
/// leaves the signature valid and the target reaches the size check
/// rather than the signature gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessSurgeryObservation {
    row: &'static str,
    input_index: usize,
    item_index: usize,
    control_item_bytes: usize,
    mutant_item_bytes: usize,
    mutant_bytes: Vec<u8>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl WitnessSurgeryObservation {
    /// The §15 row this mutant drives.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The input whose witness carries the malformed item.
    #[must_use]
    pub const fn input_index(&self) -> usize {
        self.input_index
    }

    /// The witness-stack position the malformed item occupies.
    #[must_use]
    pub const fn item_index(&self) -> usize {
        self.item_index
    }

    /// How long the control's item at that position was, and how long
    /// the mutant's is. A valid control block is thirty-three bytes plus
    /// a whole number of thirty-two-byte path entries, so the two
    /// lengths differing by one is exactly what makes the mutant's size
    /// invalid while its path bytes stay the control's.
    #[must_use]
    pub const fn item_lengths(&self) -> (usize, usize) {
        (self.control_item_bytes, self.mutant_item_bytes)
    }

    /// How many bytes this mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the target refused this mutant at, where it was observed.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }
}

/// One consensus mutant, as this ceremony built, submitted and observed
/// it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusMutantObservation {
    row: &'static str,
    declared_field_range: (usize, usize),
    input_count: usize,
    output_count: usize,
    mutant_bytes: Vec<u8>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl ConsensusMutantObservation {
    /// The §15 row this mutant drives.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The half-open witnessless byte range the mutation confined itself
    /// to, in the control's coordinates.
    #[must_use]
    pub const fn declared_field_range(&self) -> (usize, usize) {
        self.declared_field_range
    }

    /// The mutant's transaction shape: its input and output counts. The
    /// field surgeries keep the control's shape and separate by their byte
    /// range; the structural surgeries change the shape, and it is the
    /// shape that separates them where their byte ranges cannot be
    /// localized past the output-count varint.
    #[must_use]
    pub const fn shape(&self) -> (usize, usize) {
        (self.input_count, self.output_count)
    }

    /// The separating fact this mutant declares: its byte range together
    /// with its shape. Distinct across every driven row, so no two rows
    /// rest on one observation.
    #[must_use]
    pub const fn separator(&self) -> ((usize, usize), (usize, usize)) {
        (self.declared_field_range, self.shape())
    }

    /// How many bytes this mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the target refused this mutant at, where it was observed.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }
}

/// What this ceremony refuses, before any node is asked.
///
/// Every member is a construction or infrastructure fact and none is a
/// target verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerSigningNegativeRefusal {
    /// The reviewed target, the demonstration link, or a published owner
    /// was unavailable.
    SubstrateUnavailable,
    /// The issuance step named no asset to link against.
    IssuanceNamedNoAsset,
    /// The funding step created no coin to spend.
    FundingCreatedNoPredecessor,
    /// A funded coin's outpoint, asset, or program did not decode.
    MalformedFundedOutput,
    /// Re-linking the deployment against the issued asset was refused.
    RelinkRefused,
    /// The candidate did not construct or finalize.
    CandidateNotConstructible,
    /// The census refused the finalized candidate or its mutant.
    CensusRefused(OwnerCensusRefusal),
    /// The signing material refused to produce a signature.
    SigningRefused,
    /// The mutated program disturbed a field outside the declared one.
    MutationNotConfined {
        /// The half-open witnessless byte range the diff actually touched.
        touched: (usize, usize),
        /// The half-open range the mutation declared it would touch.
        declared: (usize, usize),
    },
}

/// The two-origin check on the accepted control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlReverification {
    readback_matches_submission: bool,
}

impl ControlReverification {
    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }
}

/// The mutant, as this ceremony submitted and observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutantObservation {
    declared_field_range: (usize, usize),
    submitted_bytes: usize,
    message: Digest32,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl MutantObservation {
    /// The half-open witnessless byte range the mutation stayed within:
    /// the mutated destination's program.
    #[must_use]
    pub const fn declared_field_range(&self) -> (usize, usize) {
        self.declared_field_range
    }

    /// How many bytes the mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The message the mutant's signature was taken over.
    #[must_use]
    pub const fn message(&self) -> &Digest32 {
        &self.message
    }

    /// The layer the target refused the mutant at, where it was observed.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }
}

/// The control, as this ceremony submitted and observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlObservation {
    submitted_bytes: usize,
    message: Digest32,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    reverification: Option<ControlReverification>,
}

impl ControlObservation {
    /// How many bytes the control handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The message the control's signature was taken over.
    #[must_use]
    pub const fn message(&self) -> &Digest32 {
        &self.message
    }

    /// The layer the target answered the control at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The two-origin check, where an acceptance was observed.
    #[must_use]
    pub const fn reverification(&self) -> Option<&ControlReverification> {
        self.reverification.as_ref()
    }
}

/// Everything the ceremony recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnerSigningNegativeRecord {
    issued_asset: Option<String>,
    relinked: bool,
    coins: Vec<ObservedFundedCoin>,
    mutant: Option<MutantObservation>,
    consensus_mutants: Vec<ConsensusMutantObservation>,
    leaf_arrangements: Vec<LeafArrangementObservation>,
    witness_surgery: Option<WitnessSurgeryObservation>,
    control: Option<ControlObservation>,
    refusal: Option<OwnerSigningNegativeRefusal>,
}

impl OwnerSigningNegativeRecord {
    /// The consensus-conservation mutants, in the order they ran.
    #[must_use]
    pub fn consensus_mutants(&self) -> &[ConsensusMutantObservation] {
        &self.consensus_mutants
    }

    /// The malformed control-path mutant, where it was built.
    #[must_use]
    pub const fn witness_surgery(&self) -> Option<&WitnessSurgeryObservation> {
        self.witness_surgery.as_ref()
    }

    /// The leaf-arrangement mutants, in the order they ran.
    #[must_use]
    pub fn leaf_arrangements(&self) -> &[LeafArrangementObservation] {
        &self.leaf_arrangements
    }

    /// The exact locator staged for one submitted mutant.
    ///
    /// This accessor returns only the planner's typed mutation metadata.
    /// Exact request bytes and target responses remain owned by the
    /// executor journal.
    #[must_use]
    pub fn capture_locator(&self, step: &str) -> Option<LiveMutationLocator> {
        if step == "bare-u-output-mutant" {
            let (start, end) = self.mutant.as_ref()?.declared_field_range();
            return Some(LiveMutationLocator::WitnesslessRange { start, end });
        }
        if step == MALFORMED_CONTROL_PATH_STEP {
            let surgery = self.witness_surgery.as_ref()?;
            return Some(LiveMutationLocator::WitnessItem {
                input_index: surgery.input_index(),
                item_index: surgery.item_index(),
            });
        }
        if let Some(row) = step.strip_prefix("consensus-") {
            let mutant = self
                .consensus_mutants
                .iter()
                .find(|mutant| mutant.row() == row)?;
            let (mutant_inputs, mutant_outputs) = mutant.shape();
            if mutant_inputs != usize::from(RECEIPT_COUNT)
                || mutant_outputs != usize::from(RECEIPT_COUNT)
            {
                return Some(LiveMutationLocator::TransactionShape {
                    control_inputs: usize::from(RECEIPT_COUNT),
                    mutant_inputs,
                    control_outputs: usize::from(RECEIPT_COUNT),
                    mutant_outputs,
                });
            }
            let (start, end) = mutant.declared_field_range();
            return Some(LiveMutationLocator::WitnesslessRange { start, end });
        }
        let row = step.strip_prefix("leaf-arrangement-")?;
        let mutant = self
            .leaf_arrangements
            .iter()
            .find(|observation| observation.row() == row)?;
        let coordinator = self
            .leaf_arrangements
            .iter()
            .find(|observation| observation.row() == "two-coordinators")
            .and_then(|observation| revealed_leaf_programs(&observation.mutant_bytes))?
            .first()
            .cloned()?;
        let member = self
            .leaf_arrangements
            .iter()
            .find(|observation| observation.row() == "no-coordinator")
            .and_then(|observation| revealed_leaf_programs(&observation.mutant_bytes))?
            .first()
            .cloned()?;
        let mutant_programs = revealed_leaf_programs(&mutant.mutant_bytes)?;
        let mutant_coordinator_leaf_indices = mutant_programs
            .iter()
            .enumerate()
            .filter_map(|(index, program)| (program == &coordinator).then_some(index))
            .collect();
        Some(LiveMutationLocator::CommittedLeafArrangement {
            input_indices: (0..mutant_programs.len()).collect(),
            control_coordinator_leaf_indices: vec![0],
            mutant_coordinator_leaf_indices,
            control_committed_leaf_programs: vec![coordinator, member],
            mutant_committed_leaf_programs: mutant_programs,
        })
    }

    /// The asset identity the target chose.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// Whether the deployment was linked against that asset before
    /// anything was funded.
    #[must_use]
    pub const fn relinked(&self) -> bool {
        self.relinked
    }

    /// Every coin the funding step created, as the node reported it.
    #[must_use]
    pub fn coins(&self) -> &[ObservedFundedCoin] {
        &self.coins
    }

    /// The mutant's observation, where it was built and submitted.
    #[must_use]
    pub const fn mutant(&self) -> Option<&MutantObservation> {
        self.mutant.as_ref()
    }

    /// The control's observation, where it was built and submitted.
    #[must_use]
    pub const fn control(&self) -> Option<&ControlObservation> {
        self.control.as_ref()
    }

    /// Why the ceremony stopped, where it did.
    #[must_use]
    pub const fn refusal(&self) -> Option<&OwnerSigningNegativeRefusal> {
        self.refusal.as_ref()
    }

    /// What this ceremony does not claim, whatever it observed.
    #[must_use]
    pub fn non_claims() -> Vec<&'static str> {
        vec![
            "establishes nothing about the proof-bearing lane: every candidate here is an \
             explicit one and the covenant it reaches is the explicit destination's",
            "attributes no consensus mutant to a script clause: a mutant refused \
             bad-txns-in-ne-out reached no leaf and is recorded as a consensus verdict, not a \
             script-path one — the script-path verdicts here are the bare-u program surgery and \
             the two leaf-arrangement mutants",
            "discharges each row by its OWN mutant: the bare-u program surgery is the script-path \
             row's, and each consensus surgery breaks conservation in its own field so its refusal \
             separates by a distinct declared range and transaction shape rather than sharing one \
             observation",
            "drives every leaf-arrangement row it stages and moves no taptree: two-coordinators \
             and no-coordinator each reveal a committed leaf at a forbidden position and are \
             refused at the covenant's own index or bound clause, while \
             member-coordinator-leaf-exchange exchanges the two roles and rests on its ARRANGEMENT \
             rather than on a verdict, because both of its inputs fail and which failure a target \
             reports across a multi-input candidate is the target's own abort selection",
            "predicts no verdict for the malformed control path beyond the layer: the control \
             block is refused for its SIZE during taproot script verification, and the exact words \
             a target gives for a wrong-sized control block are the target's",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

fn revealed_leaf_programs(bytes: &[u8]) -> Option<Vec<Vec<u8>>> {
    let transaction = TargetTransaction::decode(bytes).ok()?;
    transaction
        .witnesses()
        .iter()
        .map(|witness| {
            witness
                .stack()
                .len()
                .checked_sub(2)
                .and_then(|index| witness.stack().get(index))
                .cloned()
        })
        .collect()
}

/// What the plan is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset the deployment is then linked against.
    Issue,
    /// Pay the issued asset to the re-linked explicit constructor.
    Fund,
    /// Submit the bare-u script mutant, first, so the control's coins stay
    /// unspent.
    Mutant,
    /// Submit the consensus-conservation mutant at this index, before the
    /// control so its coins stay unspent for the acceptance.
    ConsensusMutant(usize),
    /// Submit the leaf-arrangement mutant at this index, before the control
    /// so its coins stay unspent for the acceptance.
    LeafArrangement(usize),
    /// Submit the malformed control-path mutant, before the control so
    /// its coins stay unspent for the acceptance.
    WitnessSurgery,
    /// Submit the unmutated control, last, which is what consumes them.
    Control,
    /// Nothing further.
    Done,
}

/// One submitted candidate, held until its answer arrives.
struct PendingSubmission {
    bytes: Vec<u8>,
}

/// The owner-signing negative ceremony.
pub struct OwnerSigningNegativePlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    explicit_program: Vec<u8>,
    genesis_block_hash: Digest32,
    pending: Option<PendingSubmission>,
    record: OwnerSigningNegativeRecord,
}

impl OwnerSigningNegativePlanner {
    /// The ceremony bound to one deployment's genesis block hash.
    ///
    /// The genesis hash is a constructor argument rather than something a
    /// candidate carries: the target seeds its message hasher with it
    /// twice, so two identical candidates on two chains have different
    /// messages, and it arrives from the run's own deployment binding in
    /// the same PRINTED order [`crate::live_owner_observation`] reverses to
    /// the seed.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI or
    /// the explicit destination constructor is unavailable.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        let explicit_program = explicit_destination_program(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            abi,
            explicit_program,
            genesis_block_hash: crate::live_owner_observation::printed_order(
                printed_genesis_identity,
            ),
            pending: None,
            record: OwnerSigningNegativeRecord::default(),
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &OwnerSigningNegativeRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: OwnerSigningNegativeRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The funding step for the explicit constructor's program.
    fn funding_step(&self, name: &str, issue: bool) -> OperationStep {
        OperationStep::new(
            name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.record.issued_asset.clone()
                },
                output_program: self.explicit_program.clone(),
                outputs: RECEIPT_COUNT,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        )
    }

    /// Link the deployment against the asset the target issued.
    fn relink(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), OwnerSigningNegativeRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(OwnerSigningNegativeRefusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(OwnerSigningNegativeRefusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(*identity.internal(), RESERVE_ASSET, FEE_PROGRAM_DIGEST)
            .map_err(|_| OwnerSigningNegativeRefusal::RelinkRefused)?;
        self.explicit_program = explicit_destination_program(&abi)
            .map_err(|_| OwnerSigningNegativeRefusal::RelinkRefused)?;
        self.record.issued_asset = Some(asset);
        self.record.relinked = true;
        self.abi = abi;
        Ok(())
    }

    /// Take the funded coins from the node's own report of them.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), OwnerSigningNegativeRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(OwnerSigningNegativeRefusal::FundingCreatedNoPredecessor);
        }
        let expected_asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(OwnerSigningNegativeRefusal::IssuanceNamedNoAsset)?;

        let mut coins = Vec::with_capacity(response.funded_outputs.len());
        for funded in &response.funded_outputs {
            coins.push(self.observed_coin(funded, expected_asset)?);
        }
        self.record.coins = coins;
        Ok(())
    }

    /// One funded coin, as reported and as expected.
    fn observed_coin(
        &self,
        funded: &FundedOutput,
        expected_asset: AssetId,
    ) -> Result<ObservedFundedCoin, OwnerSigningNegativeRefusal> {
        let outpoint = outpoint_of(&funded.outpoint)
            .ok_or(OwnerSigningNegativeRefusal::MalformedFundedOutput)?;
        let asset =
            asset_of(&funded.asset).ok_or(OwnerSigningNegativeRefusal::MalformedFundedOutput)?;
        let program =
            decode_hex(&funded.script).ok_or(OwnerSigningNegativeRefusal::MalformedFundedOutput)?;

        let matches_expectation = asset == expected_asset
            && funded.amount_satoshis == RECEIPT_AMOUNT
            && program == self.explicit_program;

        Ok(ObservedFundedCoin::observed(
            outpoint,
            AssetField::Explicit(asset),
            ValueField::Explicit(funded.amount_satoshis),
            program,
            matches_expectation,
        ))
    }

    /// One finalized explicit candidate over the funded coins, paying the
    /// two destinations this ceremony's successor has always paid.
    fn finalize(&self) -> Result<FinalizedLiveTransfer, OwnerSigningNegativeRefusal> {
        finalize_explicit(&self.abi, &self.record.coins, usize::from(RECEIPT_COUNT))
    }

    /// The per-input signing requests for a finalized candidate.
    fn requests(finalized: &FinalizedLiveTransfer) -> Vec<OwnerSigningInputRequest> {
        signing_requests(finalized)
    }
}

/// The published owners an explicit successor pays, in output order.
///
/// The first two are the pair the sponsorless two-output successor has
/// always paid, in that order, and a wider successor APPENDS rather than
/// inserts. That is what lets the destination count become a parameter
/// without moving the two-output candidate's bytes: widening the array
/// leaves the shorter prefix exactly as it was, so the ceremony whose
/// control digest is already recorded still builds the candidate it
/// recorded.
const DESTINATION_SCALARS: [[u8; FIELD_ELEMENT_BYTES]; 3] =
    [SECOND_SCALAR, FIRST_SCALAR, THIRD_SCALAR];

/// The public view a ceremony constructs against, over its funded coins.
///
/// # Errors
///
/// [`OwnerSigningNegativeRefusal::CandidateNotConstructible`] where the
/// coins do not form a construction view.
pub(crate) fn explicit_view(
    coins: &[ObservedFundedCoin],
) -> Result<PublicConstructionView, OwnerSigningNegativeRefusal> {
    PublicConstructionView::new(coins.iter().map(|coin| {
        PublicOutputView::new(
            coin.outpoint(),
            coin.asset(),
            coin.value(),
            coin.program().to_vec(),
        )
    }))
    .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)
}

/// One finalized explicit sponsorless candidate spending every funded coin
/// and paying `destinations` published owners.
///
/// The consumed total is divided evenly and the LAST destination takes the
/// remainder, so the created side sums to the consumed side exactly and the
/// candidate balances whatever the counts are. At two destinations this is
/// the same arithmetic the two-output successor always did — an even half
/// and the rest — so its bytes do not move.
///
/// # Errors
///
/// [`OwnerSigningNegativeRefusal::CandidateNotConstructible`] where the
/// coins do not view, the count is zero or wider than the published owners
/// available, the totals do not compute, or the request does not finalize;
/// [`OwnerSigningNegativeRefusal::SubstrateUnavailable`] where the reviewed
/// target or a published owner is unavailable.
pub(crate) fn finalize_explicit(
    abi: &CandidateLiveTransferAbi,
    coins: &[ObservedFundedCoin],
    destinations: usize,
) -> Result<FinalizedLiveTransfer, OwnerSigningNegativeRefusal> {
    let view = explicit_view(coins)?;
    let points: Vec<Outpoint> = coins.iter().map(ObservedFundedCoin::outpoint).collect();
    let total = RECEIPT_AMOUNT
        .checked_mul(u64::try_from(points.len()).unwrap_or(0))
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let scalars = DESTINATION_SCALARS
        .get(..destinations)
        .filter(|scalars| !scalars.is_empty())
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

    let width = u64::try_from(destinations).unwrap_or(0);
    let share = total
        .checked_div(width)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let head = share
        .checked_mul(width.saturating_sub(1))
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let remainder = total
        .checked_sub(head)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

    let mut receipts = Vec::with_capacity(destinations);
    for (index, scalar) in scalars.iter().enumerate() {
        let amount = if index + 1 == destinations {
            remainder
        } else {
            share
        };
        let owner = published_owner(scalar)
            .map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
        let value = ProtocolValue::new(amount)
            .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
        receipts.push(LiveReceiptDestination::new(
            linker::OwnerParameter::new(owner),
            value,
        ));
    }

    let request = LiveTransferRequest::new(
        points,
        receipts,
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

    let target =
        reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
    let finalization = finalize_live_transfer(&target, abi, &request, &view, None, None)
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    Ok(finalization.into_finalized())
}

/// The per-input signing requests for a finalized candidate, one per
/// receipt, each naming the leaf that input executes.
pub(crate) fn signing_requests(finalized: &FinalizedLiveTransfer) -> Vec<OwnerSigningInputRequest> {
    finalized
        .receipts()
        .iter()
        .map(|record| {
            OwnerSigningInputRequest::new(
                u32::from(record.position()),
                leaf_hash(LeafVersion::TAPSCRIPT, record.leaf_script()),
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                record.control_block().to_vec(),
            )
        })
        .collect()
}

/// The census of one candidate's parts, over the negative-evidence route.
pub(crate) fn negative_census(
    candidate: TargetTransaction,
    spent_outputs: Vec<transaction::live_census::SpentOutputCensusEntry>,
    genesis: Digest32,
    requests: &[OwnerSigningInputRequest],
) -> Result<OwnerSigningCensus, OwnerSigningNegativeRefusal> {
    let target =
        reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
    let curve = OracleLiveCurve::new(
        reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?,
    );
    let protected_bytes = candidate.encode_without_witness();
    let output_witnesses = candidate.output_witnesses().to_vec();
    OwnerSigningCensus::over_foreign_bytes_for_negative_evidence(
        &target,
        candidate,
        protected_bytes,
        output_witnesses,
        spent_outputs,
        LiveDeployment::new(genesis),
        requests,
        &curve,
    )
    .map_err(OwnerSigningNegativeRefusal::CensusRefused)
}

/// The spent-output census the finalized explicit form carries, read
/// through the production route so its entries are exactly the ones that
/// route would sign over.
pub(crate) fn explicit_spent_outputs(
    finalized: &FinalizedLiveTransfer,
    genesis: Digest32,
) -> Result<Vec<transaction::live_census::SpentOutputCensusEntry>, OwnerSigningNegativeRefusal> {
    let target =
        reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
    let curve = OracleLiveCurve::new(
        reviewed_target().map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?,
    );
    let census = OwnerSigningCensus::from_explicit_finalized(
        &target,
        finalized,
        LiveDeployment::new(genesis),
        &curve,
    )
    .map_err(OwnerSigningNegativeRefusal::CensusRefused)?;
    Ok(census.spent_outputs().to_vec())
}

impl OwnerSigningNegativePlanner {
    /// The spent-output census for this ceremony's own deployment.
    fn spent_outputs(
        &self,
        finalized: &FinalizedLiveTransfer,
    ) -> Result<Vec<transaction::live_census::SpentOutputCensusEntry>, OwnerSigningNegativeRefusal>
    {
        explicit_spent_outputs(finalized, self.genesis_block_hash)
    }

    /// One candidate's submittable bytes and its first input's message.
    ///
    /// Both the control and the mutant are built here, differing only in
    /// whether the mutated destination's program is rewritten. Each input
    /// is re-signed over the candidate's own census and its witness stack
    /// assembled `[signature, leaf_script, control_block]`, which is the
    /// route's whole point: the mutant is signed over its own mutated
    /// bytes, so it passes the signature gate the leaf checks first.
    fn build(
        &self,
        finalized: &FinalizedLiveTransfer,
        spent_outputs: &[transaction::live_census::SpentOutputCensusEntry],
        mutate: bool,
    ) -> Result<(Vec<u8>, Digest32), OwnerSigningNegativeRefusal> {
        let candidate = finalized.protected().clone();
        let candidate = if mutate {
            rewrite_output_program(&candidate, MUTATED_OUTPUT, BARE_U_PROGRAM.to_vec())?
        } else {
            candidate
        };

        let requests = Self::requests(finalized);
        let census = negative_census(
            candidate.clone(),
            spent_outputs.to_vec(),
            self.genesis_block_hash,
            &requests,
        )?;

        // Every funded receipt is paid to the FIRST owner's explicit
        // destination program, so every input's leaf checks that one
        // owner's key, and every input is signed by that one scalar. A
        // per-position scalar would sign an input's leaf with a key it does
        // not authenticate, which the target refuses as an invalid
        // signature before any output clause runs.
        let material = signing_material(&FIRST_SCALAR)
            .map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
        let mut witnesses = candidate.witnesses().to_vec();
        let mut first_message = None;
        for record in finalized.receipts() {
            let position = usize::from(record.position());
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(record.position()))
                .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            if first_message.is_none() {
                first_message = Some(message);
            }
            let signature = material
                .sign(&message, &SIGNING_AUXILIARY)
                .map_err(|_| OwnerSigningNegativeRefusal::SigningRefused)?
                .to_vec();
            let witness = InputWitness::new(vec![
                signature,
                record.leaf_script().to_vec(),
                record.control_block().to_vec(),
            ]);
            *witnesses
                .get_mut(position)
                .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)? = witness;
        }

        let assembled = TargetTransaction::with_output_witnesses(
            candidate.version(),
            candidate.inputs().to_vec(),
            candidate.outputs().to_vec(),
            candidate.lock_time(),
            witnesses,
            candidate.output_witnesses().to_vec(),
        )
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

        let message =
            first_message.ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
        Ok((assembled.encode(), message))
    }

    /// Build the mutant and the control, and stage the mutant for
    /// submission. The declared field range is measured over the two
    /// candidates' WITNESSLESS serializations, where the re-signing does
    /// not reach, and checked to be the mutated output's program alone.
    fn stage_mutant(&mut self) -> Result<Vec<u8>, OwnerSigningNegativeRefusal> {
        let finalized = self.finalize()?;
        let spent_outputs = self.spent_outputs(&finalized)?;

        let (control_bytes, control_message) = self.build(&finalized, &spent_outputs, false)?;
        let (mutant_bytes, mutant_message) = self.build(&finalized, &spent_outputs, true)?;

        // The declared field: the mutated output's program, measured over
        // the witnessless serialization by substituting a sentinel program
        // of the mutant's own length and diffing. The witness bytes differ
        // between control and mutant because the mutant is re-signed, which
        // is the route's purpose and not a disturbance the declaration
        // covers.
        let control_candidate = finalized.protected().clone();
        let mutant_candidate =
            rewrite_output_program(&control_candidate, MUTATED_OUTPUT, BARE_U_PROGRAM.to_vec())?;
        let declared = changed_range(
            &control_candidate.encode_without_witness(),
            &mutant_candidate.encode_without_witness(),
        );

        // The whole witnessless diff between the two submitted candidates
        // must be that same range: nothing outside the mutated program
        // moved in the serialization the message is taken over.
        let touched = changed_range(
            &decode_witnessless(&control_bytes),
            &decode_witnessless(&mutant_bytes),
        );
        if touched != declared {
            return Err(OwnerSigningNegativeRefusal::MutationNotConfined { touched, declared });
        }

        self.record.mutant = Some(MutantObservation {
            declared_field_range: declared,
            submitted_bytes: mutant_bytes.len(),
            message: mutant_message,
            observed_layer: None,
            observed_detail: None,
        });
        self.record.control = Some(ControlObservation {
            submitted_bytes: control_bytes.len(),
            message: control_message,
            observed_layer: None,
            observed_detail: None,
            accepted_txid: None,
            reverification: None,
        });
        // The consensus-conservation mutants are cut from the SIGNED
        // control's own bytes and NOT re-signed: the consensus balance
        // check is queued before script verification, so a broken tally
        // refuses the mutant before its stale signature is examined. Each
        // is confined to its own witnessless byte range, measured here.
        self.record.consensus_mutants = build_consensus_mutants(&control_bytes)?;

        // The leaf-arrangement mutants rearrange which committed leaf each
        // input reveals and re-sign over the rearranged census, so each
        // passes the signature gate and is refused at the covenant's own
        // index or bound clause. Both spent outputs commit to one taptree
        // holding both leaves, so the rearrangement reuses committed leaves
        // and moves no digest.
        self.record.leaf_arrangements = self.build_leaf_arrangements(&finalized, &spent_outputs)?;

        // The malformed control-path mutant is cut from the same signed
        // control and is NOT re-signed either, for a different reason
        // than the consensus mutants: the tapscript message commits to
        // the tapleaf hash rather than to the control block's path
        // bytes, so resizing the control block leaves the signature
        // valid and the size check is what the candidate reaches.
        self.record.witness_surgery = Some(build_witness_surgery(&control_bytes)?);

        // The control's bytes are stashed on the control record's message
        // check; the bytes themselves are rebuilt for the control step so
        // the ceremony holds one pending submission at a time.
        Ok(mutant_bytes)
    }

    /// Record what the target did with the bare-u mutant.
    fn settle_mutant(&mut self, response: &NativeOperationResponse) {
        if let Some(mutant) = self.record.mutant.as_mut() {
            mutant.observed_layer = Some(response.observed_layer);
            mutant.observed_detail.clone_from(&response.observed_detail);
        }
    }

    /// Record what the target did with one consensus mutant.
    fn settle_consensus_mutant(&mut self, index: usize, response: &NativeOperationResponse) {
        if let Some(mutant) = self.record.consensus_mutants.get_mut(index) {
            mutant.observed_layer = Some(response.observed_layer);
            mutant.observed_detail.clone_from(&response.observed_detail);
        }
    }

    /// The submission step for one consensus mutant.
    fn consensus_mutant_step(&self, index: usize) -> Option<OperationStep> {
        let mutant = self.record.consensus_mutants.get(index)?;
        Some(OperationStep::new(
            &CONSENSUS_SURGERIES[index].step_name(),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: mutant.mutant_bytes.clone(),
            })),
        ))
    }

    /// Build every leaf-arrangement mutant from the finalized candidate.
    ///
    /// Each mutant reuses the finalized receipts' own committed leaf
    /// scripts and control blocks, rearranged across the inputs, and is
    /// re-signed over the rearranged census. Nothing but the witness
    /// changes, so the witnessless serialization stays the control's and
    /// the separating fact is the arrangement.
    fn build_leaf_arrangements(
        &self,
        finalized: &FinalizedLiveTransfer,
        spent_outputs: &[transaction::live_census::SpentOutputCensusEntry],
    ) -> Result<Vec<LeafArrangementObservation>, OwnerSigningNegativeRefusal> {
        let mut mutants = Vec::with_capacity(LEAF_ARRANGEMENTS.len());
        for arrangement in LEAF_ARRANGEMENTS {
            let (mutant_bytes, message) =
                self.build_leaf_arrangement(finalized, spent_outputs, arrangement)?;
            mutants.push(LeafArrangementObservation {
                row: arrangement.row(),
                typed_partner: arrangement.typed_partner(),
                revealed_arrangement: arrangement.sources().to_vec(),
                submitted_bytes: mutant_bytes.len(),
                mutant_bytes,
                message,
                observed_layer: None,
                observed_detail: None,
            });
        }
        Ok(mutants)
    }

    /// One leaf-arrangement mutant's submittable bytes and the message its
    /// REARRANGED input's signature was taken over.
    ///
    /// Every input's witness is rebuilt: its revealed leaf is the leaf of
    /// the receipt the arrangement names for that position, its control
    /// block that receipt's, and its signature re-taken over the rearranged
    /// census so the candidate passes the signature gate. The message
    /// recorded is the rearranged input's, so it differs from the control's
    /// at that input and the comparison is not vacuous.
    fn build_leaf_arrangement(
        &self,
        finalized: &FinalizedLiveTransfer,
        spent_outputs: &[transaction::live_census::SpentOutputCensusEntry],
        arrangement: LeafArrangement,
    ) -> Result<(Vec<u8>, Digest32), OwnerSigningNegativeRefusal> {
        let candidate = finalized.protected().clone();
        let sources = arrangement.sources();

        let mut requests = Vec::with_capacity(finalized.receipts().len());
        for record in finalized.receipts() {
            let revealed = revealed_leaf(finalized, &sources, record.position())?;
            requests.push(OwnerSigningInputRequest::new(
                u32::from(record.position()),
                leaf_hash(LeafVersion::TAPSCRIPT, revealed.leaf_script()),
                LeafVersion::TAPSCRIPT,
                OWNER_CODESEPARATOR_POSITION,
                AnnexDisposition::Absent,
                IssuanceDisposition::Absent,
                revealed.control_block().to_vec(),
            ));
        }

        let census = negative_census(
            candidate.clone(),
            spent_outputs.to_vec(),
            self.genesis_block_hash,
            &requests,
        )?;

        let material = signing_material(&FIRST_SCALAR)
            .map_err(|_| OwnerSigningNegativeRefusal::SubstrateUnavailable)?;
        let mut witnesses = candidate.witnesses().to_vec();
        let mut rearranged_message = None;
        for record in finalized.receipts() {
            let position = usize::from(record.position());
            let revealed = revealed_leaf(finalized, &sources, record.position())?;
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(record.position()))
                .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            // The rearranged input is the one whose revealed leaf is not its
            // own position's: its message differs from the control's, which
            // is what makes the mutant a distinct candidate.
            if rearranged_message.is_none()
                && sources.get(position).copied() != Some(record.position())
            {
                rearranged_message = Some(message);
            }
            let signature = material
                .sign(&message, &SIGNING_AUXILIARY)
                .map_err(|_| OwnerSigningNegativeRefusal::SigningRefused)?
                .to_vec();
            let witness = InputWitness::new(vec![
                signature,
                revealed.leaf_script().to_vec(),
                revealed.control_block().to_vec(),
            ]);
            *witnesses
                .get_mut(position)
                .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)? = witness;
        }

        let assembled = TargetTransaction::with_output_witnesses(
            candidate.version(),
            candidate.inputs().to_vec(),
            candidate.outputs().to_vec(),
            candidate.lock_time(),
            witnesses,
            candidate.output_witnesses().to_vec(),
        )
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

        let message =
            rearranged_message.ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
        Ok((assembled.encode(), message))
    }

    /// Record what the target did with one leaf-arrangement mutant.
    fn settle_leaf_arrangement(&mut self, index: usize, response: &NativeOperationResponse) {
        if let Some(mutant) = self.record.leaf_arrangements.get_mut(index) {
            mutant.observed_layer = Some(response.observed_layer);
            mutant.observed_detail.clone_from(&response.observed_detail);
        }
    }

    /// The submission step for one leaf-arrangement mutant.
    fn leaf_arrangement_step(&self, index: usize) -> Option<OperationStep> {
        let mutant = self.record.leaf_arrangements.get(index)?;
        Some(OperationStep::new(
            &LEAF_ARRANGEMENTS[index].step_name(),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: mutant.mutant_bytes.clone(),
            })),
        ))
    }

    /// Record what the target did with the malformed control-path mutant.
    fn settle_witness_surgery(&mut self, response: &NativeOperationResponse) {
        if let Some(surgery) = self.record.witness_surgery.as_mut() {
            surgery.observed_layer = Some(response.observed_layer);
            surgery
                .observed_detail
                .clone_from(&response.observed_detail);
        }
    }

    /// The submission step for the malformed control-path mutant.
    fn witness_surgery_step(&self) -> Option<OperationStep> {
        let surgery = self.record.witness_surgery.as_ref()?;
        Some(OperationStep::new(
            MALFORMED_CONTROL_PATH_STEP,
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: surgery.mutant_bytes.clone(),
            })),
        ))
    }

    /// Rebuild and stage the control for submission.
    fn stage_control(&self) -> Result<Vec<u8>, OwnerSigningNegativeRefusal> {
        let finalized = self.finalize()?;
        let spent_outputs = self.spent_outputs(&finalized)?;
        let (control_bytes, _message) = self.build(&finalized, &spent_outputs, false)?;
        Ok(control_bytes)
    }

    /// Record what the target did with the control, and the readback where
    /// it accepted.
    fn settle_control(&mut self, submitted: &[u8], response: &NativeOperationResponse) {
        if let Some(control) = self.record.control.as_mut() {
            control.observed_layer = Some(response.observed_layer);
            control
                .observed_detail
                .clone_from(&response.observed_detail);
            control.accepted_txid.clone_from(&response.accepted_txid);
            if response.observed_layer == ObservedOutcomeLayer::Accepted {
                control.reverification = reverify(response.mined_readback.as_ref(), submitted);
            }
        }
    }
}

impl TargetOperationPlanner for OwnerSigningNegativePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.relink(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund;
                }
                Stage::Fund => {
                    if let Err(refusal) = self.settle_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Mutant;
                }
                Stage::Mutant => {
                    self.settle_mutant(response);
                    self.stage = Stage::ConsensusMutant(0);
                }
                Stage::ConsensusMutant(index) => {
                    self.settle_consensus_mutant(index, response);
                    self.stage = if index + 1 < self.record.consensus_mutants.len() {
                        Stage::ConsensusMutant(index + 1)
                    } else {
                        Stage::LeafArrangement(0)
                    };
                }
                Stage::LeafArrangement(index) => {
                    self.settle_leaf_arrangement(index, response);
                    self.stage = if index + 1 < self.record.leaf_arrangements.len() {
                        Stage::LeafArrangement(index + 1)
                    } else {
                        Stage::WitnessSurgery
                    };
                }
                Stage::WitnessSurgery => {
                    self.settle_witness_surgery(response);
                    self.stage = Stage::Control;
                }
                Stage::Control => {
                    let submitted = self
                        .pending
                        .take()
                        .map(|pending| pending.bytes)
                        .unwrap_or_default();
                    self.settle_control(&submitted, response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(self.funding_step("issue-protocol-asset", true))),
            Stage::Fund => Ok(Some(self.funding_step("fund-explicit-constructor", false))),
            Stage::Mutant => match self.stage_mutant() {
                Ok(bytes) => {
                    self.pending = Some(PendingSubmission {
                        bytes: bytes.clone(),
                    });
                    Ok(Some(OperationStep::new(
                        MUTANT_STEP,
                        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                            transaction_bytes: bytes,
                        })),
                    )))
                }
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::ConsensusMutant(index) => self.consensus_mutant_step(index).map_or_else(
                || Err(self.refuse(OwnerSigningNegativeRefusal::CandidateNotConstructible)),
                |step| Ok(Some(step)),
            ),
            Stage::LeafArrangement(index) => self.leaf_arrangement_step(index).map_or_else(
                || Err(self.refuse(OwnerSigningNegativeRefusal::CandidateNotConstructible)),
                |step| Ok(Some(step)),
            ),
            Stage::WitnessSurgery => self.witness_surgery_step().map_or_else(
                || Err(self.refuse(OwnerSigningNegativeRefusal::CandidateNotConstructible)),
                |step| Ok(Some(step)),
            ),
            Stage::Control => match self.stage_control() {
                Ok(bytes) => {
                    self.pending = Some(PendingSubmission {
                        bytes: bytes.clone(),
                    });
                    Ok(Some(OperationStep::new(
                        CONTROL_STEP,
                        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                            transaction_bytes: bytes,
                        })),
                    )))
                }
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Done => Ok(None),
        }
    }
}

/// The explicit destination program of the first published owner.
fn explicit_destination_program(abi: &CandidateLiveTransferAbi) -> Result<Vec<u8>, VectorError> {
    Ok(abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(published_owner(&FIRST_SCALAR)?),
            LiveTransferRepresentationPlan::Explicit,
        )
        .ok_or(VectorError::LiveSubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

/// The receipt whose committed leaf a leaf-arrangement input reveals.
///
/// Looked up by the source POSITION the arrangement names rather than by
/// list index, so the mapping does not rest on the order the receipts
/// happen to arrive in. Both funded receipts commit to one taptree holding
/// both leaves, so any receipt's leaf is spendable at any input against the
/// shared taproot output.
fn revealed_leaf<'a>(
    finalized: &'a FinalizedLiveTransfer,
    sources: &[u16],
    position: u16,
) -> Result<&'a transaction::ReceiptInputRecord, OwnerSigningNegativeRefusal> {
    let source = *sources
        .get(usize::from(position))
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    finalized
        .receipts()
        .iter()
        .find(|record| record.position() == source)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)
}

/// One candidate with a single output's program rewritten, asset, value
/// and nonce held fixed.
fn rewrite_output_program(
    candidate: &TargetTransaction,
    output: usize,
    program: Vec<u8>,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let mut outputs = candidate.outputs().to_vec();
    let target = outputs
        .get_mut(output)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    *target = TargetOutput::new(target.asset(), target.value(), target.nonce(), program);
    TargetTransaction::with_output_witnesses(
        candidate.version(),
        candidate.inputs().to_vec(),
        outputs,
        candidate.lock_time(),
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
    .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)
}

/// One candidate with a single output's asset rewritten, value, nonce and
/// program held fixed. The output witness is left as it was: an explicit
/// output carries none, and a blinded asset with no surjection proof is
/// exactly the surjection break `confidential-asset-commitment` names.
fn rewrite_output_asset(
    candidate: &TargetTransaction,
    output: usize,
    asset: AssetField,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let mut outputs = candidate.outputs().to_vec();
    let target = outputs
        .get_mut(output)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    *target = TargetOutput::new(
        asset,
        target.value(),
        target.nonce(),
        target.program().to_vec(),
    );
    rebuild(
        candidate,
        candidate.inputs().to_vec(),
        outputs,
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
}

/// One candidate with a single explicit output's value shifted by a signed
/// delta, asset, nonce and program held fixed.
fn adjust_output_value(
    candidate: &TargetTransaction,
    output: usize,
    delta: i64,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let mut outputs = candidate.outputs().to_vec();
    let target = outputs
        .get_mut(output)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let ValueField::Explicit(amount) = target.value() else {
        return Err(OwnerSigningNegativeRefusal::CandidateNotConstructible);
    };
    let shifted = amount
        .checked_add_signed(delta)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    *target = TargetOutput::new(
        target.asset(),
        ValueField::Explicit(shifted),
        target.nonce(),
        target.program().to_vec(),
    );
    rebuild(
        candidate,
        candidate.inputs().to_vec(),
        outputs,
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
}

/// One candidate with a single explicit output's value REPLACED, asset,
/// nonce and program held fixed.
///
/// The absolute sibling of [`adjust_output_value`]. A delta cannot reach
/// the out-of-domain range from an arbitrary control amount without
/// arithmetic that depends on what the control held; an absolute write
/// states the field the candidate is to carry and leaves the dependence
/// out of the mutation.
fn set_output_value(
    candidate: &TargetTransaction,
    output: usize,
    amount: u64,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let mut outputs = candidate.outputs().to_vec();
    let target = outputs
        .get_mut(output)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let ValueField::Explicit(_) = target.value() else {
        return Err(OwnerSigningNegativeRefusal::CandidateNotConstructible);
    };
    *target = TargetOutput::new(
        target.asset(),
        ValueField::Explicit(amount),
        target.nonce(),
        target.program().to_vec(),
    );
    rebuild(
        candidate,
        candidate.inputs().to_vec(),
        outputs,
        candidate.witnesses().to_vec(),
        candidate.output_witnesses().to_vec(),
    )
}

/// One candidate with a single output, and its parallel output witness,
/// deleted.
fn remove_output(
    candidate: &TargetTransaction,
    output: usize,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let mut outputs = candidate.outputs().to_vec();
    let mut output_witnesses = candidate.output_witnesses().to_vec();
    if output >= outputs.len() {
        return Err(OwnerSigningNegativeRefusal::CandidateNotConstructible);
    }
    outputs.remove(output);
    if output < output_witnesses.len() {
        output_witnesses.remove(output);
    }
    rebuild(
        candidate,
        candidate.inputs().to_vec(),
        outputs,
        candidate.witnesses().to_vec(),
        output_witnesses,
    )
}

/// One candidate with an undeclared protocol-asset output appended, taking
/// the first receipt's asset and program so it is a well-formed explicit
/// output whose only fault is that it raises the output sum.
fn append_hidden_output(
    candidate: &TargetTransaction,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let template = candidate
        .outputs()
        .get(FIRST_RECEIPT)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let hidden = TargetOutput::new(
        template.asset(),
        ValueField::Explicit(HIDDEN_OUTPUT_AMOUNT),
        NonceField::Null,
        template.program().to_vec(),
    );
    let mut outputs = candidate.outputs().to_vec();
    outputs.push(hidden);
    let mut output_witnesses = candidate.output_witnesses().to_vec();
    output_witnesses.push(OutputWitness::new(Vec::new(), Vec::new()));
    rebuild(
        candidate,
        candidate.inputs().to_vec(),
        outputs,
        candidate.witnesses().to_vec(),
        output_witnesses,
    )
}

/// One candidate with a single input, and its parallel input witness,
/// deleted.
fn remove_input(
    candidate: &TargetTransaction,
    input: usize,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    let mut inputs = candidate.inputs().to_vec();
    let mut witnesses = candidate.witnesses().to_vec();
    if input >= inputs.len() {
        return Err(OwnerSigningNegativeRefusal::CandidateNotConstructible);
    }
    inputs.remove(input);
    if input < witnesses.len() {
        witnesses.remove(input);
    }
    rebuild(
        candidate,
        inputs,
        candidate.outputs().to_vec(),
        witnesses,
        candidate.output_witnesses().to_vec(),
    )
}

/// A blinded asset commitment carrying the explicit asset's bytes under a
/// blinded prefix, so the field is a well-formed 33-byte commitment with
/// no surjection proof behind it.
fn blinded_asset_of(asset: AssetField) -> AssetField {
    let mut commitment = [0x0a_u8; transaction::bytes::COMMITMENT_BYTES];
    if let AssetField::Explicit(id) = asset {
        commitment[1..].copy_from_slice(id.internal());
    }
    AssetField::Commitment(commitment)
}

/// Rebuild a candidate from its version and lock time with new inputs,
/// outputs, and witnesses.
fn rebuild(
    candidate: &TargetTransaction,
    inputs: Vec<transaction::bytes::TargetInput>,
    outputs: Vec<TargetOutput>,
    witnesses: Vec<InputWitness>,
    output_witnesses: Vec<OutputWitness>,
) -> Result<TargetTransaction, OwnerSigningNegativeRefusal> {
    TargetTransaction::with_output_witnesses(
        candidate.version(),
        inputs,
        outputs,
        candidate.lock_time(),
        witnesses,
        output_witnesses,
    )
    .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)
}

/// Whether a control block of this many bytes is one the reviewed target
/// will parse.
///
/// It reads a leaf version and parity byte, a thirty-two-byte internal
/// key, and then a whole number of thirty-two-byte merkle path entries up
/// to a bounded depth, and refuses any other length outright — before it
/// looks at what the path spells.
const fn control_block_size_is_valid(bytes: usize) -> bool {
    bytes >= CONTROL_BASE_BYTES
        && bytes <= CONTROL_BASE_BYTES + CONTROL_BLOCK_MAX_PATH_ENTRIES * DIGEST_BYTES
        && (bytes - CONTROL_BASE_BYTES).is_multiple_of(DIGEST_BYTES)
}

/// Build the malformed control-path mutant from the signed control's own
/// bytes.
///
/// The surgery APPENDS one byte to input zero's control block. A parsable
/// control block is [`CONTROL_BASE_BYTES`] plus a whole number of
/// [`DIGEST_BYTES`] path entries, so a length one above
/// a parsable one is never itself parsable — one is not a multiple of
/// thirty-two — while every byte the control block already held stays
/// exactly where it was. Appending rather than truncating is what keeps
/// that second half true: a truncation would drop a path byte, and the
/// mutant would then be arguing about the path as well as the size.
///
/// The signature is NOT re-taken, and unlike the consensus surgeries the
/// reason is not that the refusal comes first. The tapscript message is
/// taken over the tapleaf hash — the leaf version and the leaf script —
/// and not over the control block, so the control's own signature is
/// still the correct signature for these bytes and the candidate reaches
/// the size check rather than stopping at a signature gate.
///
/// # Errors
///
/// [`OwnerSigningNegativeRefusal::CandidateNotConstructible`] where the
/// control does not decode, carries no input at the surgery's index, has
/// no item at the control-block position, or carries a control block
/// whose size is ALREADY unparsable — that last one because a mutant cut
/// from a malformed control would be refused for the control's fault
/// rather than for the surgery's.
/// [`OwnerSigningNegativeRefusal::MutationNotConfined`] where the
/// witnessless serialization moved, which would mean the mutation was
/// not confined to the witness.
fn build_witness_surgery(
    control_bytes: &[u8],
) -> Result<WitnessSurgeryObservation, OwnerSigningNegativeRefusal> {
    let control = TargetTransaction::decode(control_bytes)
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;

    let mut witnesses = control.witnesses().to_vec();
    let witness = witnesses
        .get_mut(MALFORMED_CONTROL_INPUT)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let mut stack = witness.stack().to_vec();
    let item = stack
        .get_mut(CONTROL_BLOCK_ITEM)
        .ok_or(OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let control_item_bytes = item.len();
    if !control_block_size_is_valid(control_item_bytes) {
        return Err(OwnerSigningNegativeRefusal::CandidateNotConstructible);
    }
    item.push(CONTROL_BLOCK_PAD_BYTE);
    let mutant_item_bytes = item.len();
    *witness = InputWitness::new(stack);

    let mutant = rebuild(
        &control,
        control.inputs().to_vec(),
        control.outputs().to_vec(),
        witnesses,
        control.output_witnesses().to_vec(),
    )?;

    // The two witnessless serializations must be the SAME bytes, which
    // `changed_range` states as the empty range at their common end. This
    // is the witness-only claim made checkable: every consensus surgery
    // declares a range it moved, and this one declares that it moved
    // nothing there at all.
    let control_witnessless = control.encode_without_witness();
    let declared = (control_witnessless.len(), control_witnessless.len());
    let touched = changed_range(&control_witnessless, &mutant.encode_without_witness());
    if touched != declared {
        return Err(OwnerSigningNegativeRefusal::MutationNotConfined { touched, declared });
    }

    let mutant_bytes = mutant.encode();
    Ok(WitnessSurgeryObservation {
        row: MALFORMED_CONTROL_PATH_STEP,
        input_index: MALFORMED_CONTROL_INPUT,
        item_index: CONTROL_BLOCK_ITEM,
        control_item_bytes,
        mutant_item_bytes,
        submitted_bytes: mutant_bytes.len(),
        mutant_bytes,
        observed_layer: None,
        observed_detail: None,
    })
}

/// Build every consensus-conservation mutant from the signed control's
/// own bytes.
///
/// Each surgery is applied to the decoded signed control, the mutant
/// encoded WITH its stale witness for submission, and its declared field
/// range measured over the witnessless serialization — where the surgery
/// lives and the stale signature does not — by diffing against the
/// control's own witnessless bytes. The signatures are deliberately not
/// re-taken: the target's consensus balance check refuses each mutant
/// before any script or signature runs, so a valid signature would add
/// nothing and re-signing would obscure that the tally is what refused.
fn build_consensus_mutants(
    control_bytes: &[u8],
) -> Result<Vec<ConsensusMutantObservation>, OwnerSigningNegativeRefusal> {
    let control = TargetTransaction::decode(control_bytes)
        .map_err(|_| OwnerSigningNegativeRefusal::CandidateNotConstructible)?;
    let control_witnessless = control.encode_without_witness();

    let mut mutants = Vec::with_capacity(CONSENSUS_SURGERIES.len());
    for surgery in CONSENSUS_SURGERIES {
        let mutant = surgery.apply(&control)?;
        let mutant_bytes = mutant.encode();
        let declared = changed_range(&control_witnessless, &mutant.encode_without_witness());
        mutants.push(ConsensusMutantObservation {
            row: surgery.row(),
            declared_field_range: declared,
            input_count: mutant.inputs().len(),
            output_count: mutant.outputs().len(),
            submitted_bytes: mutant_bytes.len(),
            mutant_bytes,
            observed_layer: None,
            observed_detail: None,
        });
    }
    Ok(mutants)
}

/// The witnessless serialization of a candidate that decoded from wire
/// bytes, for confining the mutation to the serialization the message is
/// taken over.
fn decode_witnessless(bytes: &[u8]) -> Vec<u8> {
    TargetTransaction::decode(bytes)
        .map_or_else(|_| bytes.to_vec(), |tx| tx.encode_without_witness())
}

/// The half-open range, in the left string's coordinates, over which two
/// byte strings differ, bounded from both ends so an insertion or deletion
/// is a change inside a range rather than a change to everything after it.
fn changed_range(left: &[u8], right: &[u8]) -> (usize, usize) {
    let prefix = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(left.len() - prefix)
        .min(right.len().saturating_sub(prefix));
    (prefix, left.len() - suffix)
}

/// The two-origin readback check for an accepted control.
fn reverify(
    readback: Option<&MinedFundingReadback>,
    submitted: &[u8],
) -> Option<ControlReverification> {
    let readback = readback?;
    Some(ControlReverification {
        readback_matches_submission: readback.raw_transaction == submitted,
    })
}

/// One run's transcript, one fact per line.
#[must_use]
pub fn render_owner_signing_negatives(record: &OwnerSigningNegativeRecord) -> String {
    let mut lines = vec!["role owner-signing-negative-run".to_owned()];
    lines.push(format!(
        "issued_asset {}",
        record.issued_asset().unwrap_or("none")
    ));
    lines.push(format!("relinked {}", record.relinked()));

    for (index, coin) in record.coins().iter().enumerate() {
        lines.push(format!(
            "coin {index} program_bytes {} node_fields_match_expectation {}",
            coin.program().len(),
            coin.matches_expectation(),
        ));
    }

    push_mutant_lines(&mut lines, record);

    if let Some(control) = record.control() {
        lines.push(format!(
            "control submitted_bytes {} message {} layer {} txid {} detail {}",
            control.submitted_bytes(),
            printed(control.message().as_slice()),
            control
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            control.accepted_txid().unwrap_or("none"),
            control.observed_detail().unwrap_or("none"),
        ));
        if let Some(check) = control.reverification() {
            lines.push(format!(
                "control_reverification readback_matches_submission {}",
                check.readback_matches_submission(),
            ));
        }
    } else {
        lines.push("control none".to_owned());
    }

    // Whether the two candidates' messages differ, stated as its own line:
    // a mutant whose message coincided with the control's would be signed
    // over the same bytes and the whole comparison would be vacuous.
    let distinct_messages = matches!(
        (record.mutant(), record.control()),
        (Some(mutant), Some(control)) if mutant.message() != control.message()
    );
    lines.push(format!("messages_differ {distinct_messages}"));

    if let Some(refusal) = record.refusal() {
        lines.push(format!("ceremony_refused {refusal:?}"));
    }

    for claim in OwnerSigningNegativeRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }
    lines.push("each_row_by_its_own_mutant true".to_owned());

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// The four mutant families this ceremony stages, one fact per line.
///
/// Split from [`render_owner_signing_negatives`] rather than allowed past
/// the line bound: the four families are one subject — every mutant the
/// run built and what the target did with it — while what remains in the
/// caller is the run's frame, its coins, its control and its non-claims.
/// Splitting on that seam keeps each half about one thing.
fn push_mutant_lines(lines: &mut Vec<String>, record: &OwnerSigningNegativeRecord) {
    if let Some(mutant) = record.mutant() {
        lines.push(format!(
            "mutant row vault-control-entitlement-or-bare-u-output declared_range {}..{} submitted_bytes {} message {} layer {} detail {}",
            mutant.declared_field_range().0,
            mutant.declared_field_range().1,
            mutant.submitted_bytes(),
            printed(mutant.message().as_slice()),
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        ));
    } else {
        lines.push("mutant none".to_owned());
    }

    for mutant in record.consensus_mutants() {
        lines.push(format!(
            "consensus_mutant row {} declared_range {}..{} shape {}in-{}out submitted_bytes {} layer {} detail {}",
            mutant.row(),
            mutant.declared_field_range().0,
            mutant.declared_field_range().1,
            mutant.shape().0,
            mutant.shape().1,
            mutant.submitted_bytes(),
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        ));
    }

    for mutant in record.leaf_arrangements() {
        lines.push(format!(
            "leaf_arrangement row {} arrangement {:?} typed_partner {} submitted_bytes {} message {} layer {} detail {}",
            mutant.row(),
            mutant.revealed_arrangement(),
            mutant.typed_partner(),
            mutant.submitted_bytes(),
            printed(mutant.message().as_slice()),
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        ));
    }

    if let Some(surgery) = record.witness_surgery() {
        let (control_item, mutant_item) = surgery.item_lengths();
        lines.push(format!(
            "witness_surgery row {} input {} item {} control_item_bytes {control_item} mutant_item_bytes {mutant_item} submitted_bytes {} layer {} detail {}",
            surgery.row(),
            surgery.input_index(),
            surgery.item_index(),
            surgery.submitted_bytes(),
            surgery
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            surgery.observed_detail().unwrap_or("none"),
        ));
    } else {
        lines.push("witness_surgery none".to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BARE_U_PROGRAM, CONTROL_BASE_BYTES, CONTROL_BLOCK_MAX_PATH_ENTRIES, DIGEST_BYTES,
        changed_range, control_block_size_is_valid,
    };

    #[test]
    fn changed_range_bounds_a_mutation_from_both_ends() {
        assert_eq!(changed_range(&[0, 1, 2, 3, 4], &[0, 1, 9, 3, 4]), (2, 3));
        assert_eq!(changed_range(&[0, 1, 2, 3, 4], &[0, 1, 3, 4]), (2, 3));
        assert_eq!(changed_range(&[7, 7], &[7, 7]), (2, 2));
    }

    #[test]
    fn the_leaf_arrangements_declare_distinct_reveal_orders() {
        // Each driven row's arrangement is distinct from the control's and
        // from every sibling's, which is what makes the three
        // leaf-arrangement mutants distinct candidates when their verdicts
        // read as a plain OP_EQUALVERIFY or OP_VERIFY. The control reveals
        // coordinator then member; two-coordinators collapses to
        // coordinator at both, no-coordinator to member at both, and the
        // member/coordinator exchange keeps one of each but swaps which
        // input carries which. The exchange is the case that makes this
        // check load-bearing rather than decorative: it holds the same
        // MULTISET of leaves as the control and separates from it only by
        // order, so an arrangement compared as a set would not tell the
        // two apart.
        let arrangements = [
            [0, 1],
            super::LeafArrangement::TwoCoordinators.sources(),
            super::LeafArrangement::NoCoordinator.sources(),
            super::LeafArrangement::MemberCoordinatorExchange.sources(),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for arrangement in arrangements {
            assert!(
                seen.insert(arrangement),
                "two leaf arrangements share the reveal order {arrangement:?}",
            );
        }
    }

    #[test]
    fn appending_one_byte_makes_every_parsable_control_block_size_unparsable() {
        // The malformed control-path surgery rests on one arithmetic fact
        // and this is it: a parsable control block is the base plus a whole
        // number of path entries, so adding a single byte leaves a
        // remainder of one against a modulus of thirty-two and can never
        // land back on a parsable length. Checked across the whole
        // admissible depth rather than at one example, because the surgery
        // does not get to choose how deep the ceremony's taptree is.
        for entries in 0..=CONTROL_BLOCK_MAX_PATH_ENTRIES {
            let parsable = CONTROL_BASE_BYTES + entries * DIGEST_BYTES;
            assert!(
                control_block_size_is_valid(parsable),
                "a base plus {entries} whole path entries was rejected as unparsable",
            );
            assert!(
                !control_block_size_is_valid(parsable + 1),
                "one byte past a parsable size was still parsable at {entries} entries",
            );
        }
        // And the two ends are refused for their own reasons: one byte
        // short of the base has no room for the internal key, and one
        // entry past the bound is deeper than the target will read.
        assert!(!control_block_size_is_valid(CONTROL_BASE_BYTES - 1));
        assert!(!control_block_size_is_valid(
            CONTROL_BASE_BYTES + (CONTROL_BLOCK_MAX_PATH_ENTRIES + 1) * DIGEST_BYTES
        ));
    }

    #[test]
    fn the_bare_u_program_is_a_version_zero_witness_program() {
        // Version zero where the receipt constructor's is version one, and
        // a whole 20-byte program body, which is what makes the coordinator
        // leaf's version clause the one that refuses it.
        assert_eq!(BARE_U_PROGRAM[0], 0x00, "the version byte is not zero");
        assert_eq!(BARE_U_PROGRAM[1], 0x14, "the push is not twenty bytes");
        assert_eq!(BARE_U_PROGRAM.len(), 22);
    }
}
