//! The finalization boundary and the owner signing requests it admits
//! (§12.6, §1.7).
//!
//! # The boundary is a type, and crossing it back is not spelled
//!
//! §12.6 fixes ten things before owner signing begins, and §12.7 lists
//! three ways a builder could unfix them afterwards. The crate's
//! standing preference is to make a thing unrepresentable rather than to
//! refuse it, so [`FinalizedLiveTransfer`] has private fields, no
//! setter, no accessor handing out a mutable reference, and no
//! constructor outside this module. A value of it is a transfer whose
//! ten items are settled, and there is no operation on it that settles
//! them differently.
//!
//! [`LiveSigningRequest`]'s own constructor is `pub(crate)` for the same reason §15.8
//! made the sponsor's request so: §1.7 admits no protocol-owner signing
//! request until all protected data is fixed, and a request nobody can
//! build except from a finalized form cannot exist before that.
//!
//! # The three post-boundary rejections are still executable
//!
//! Structural impossibility inside this crate is not the same claim as
//! nothing anywhere being able to mutate the bytes. A surgeon with the
//! candidate serialization can add an input or drop an output, and §12.7
//! requires those to be *rejected*, which means something has to be
//! looking. [`FinalizedLiveTransfer::check_offered`] is that check: it
//! compares an offered transaction against the finalized form and names
//! which of the three happened. So the boundary is enforced twice, and
//! the two enforcements answer different questions — the type answers
//! "can this crate do it", the check answers "did somebody".
//!
//! # No digest is minted here
//!
//! A signing request carries the exact protected serialization and not a
//! sighash. The digest a signature commits to is the target's own
//! construction over a transaction only a target-native run has, and
//! §1.7 requires the profile to be *observed* from the finalized witness
//! or recomputed from the exact request rather than asserted by the
//! builder. Handing an owner 32 bytes this crate hashed would be
//! asserting it. What the owner is handed is the message's preimage and
//! the census of what the profile must commit to; what comes back says
//! which bytes it was bound to.

use std::collections::BTreeSet;

use linker::OwnerParameter;
use linker::live_backend::{
    LiveTransferLeafRole, LiveTransferRepresentationPlan, LiveTransferShape, ProtectedDatum,
};
use target_elements::SighashDimension;

use crate::bytes::{AssetField, Outpoint, TargetOutput, TargetTransaction, ValueField};
use crate::error::TransactionRefusal;
use crate::live_abi::LiveTransactionForm;

/// One of the ten things §12.6 fixes before owner signing begins.
///
/// A census rather than prose, so that the finalized form can say which
/// of them it settled and a reader can check the list against the guide
/// rather than against a struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FinalizedFact {
    /// Every input is fixed.
    EveryInput,
    /// Every destination is fixed.
    EveryDestination,
    /// Every owner is fixed.
    EveryOwner,
    /// Every semantic value is fixed.
    EverySemanticValue,
    /// Every explicit value or commitment is fixed.
    EveryExplicitValueOrCommitment,
    /// Every output constructor is fixed.
    EveryOutputConstructor,
    /// Sponsor change is fixed, present or absent.
    SponsorChange,
    /// The fee role is fixed, present or absent.
    FeeRole,
    /// The target proofs the sighash covers are fixed.
    CoveredTargetProofs,
    /// Version and locktime are fixed.
    VersionAndLockTime,
}

impl FinalizedFact {
    /// The complete census, in §12.6's own order.
    pub const ALL: &'static [Self] = &[
        Self::EveryInput,
        Self::EveryDestination,
        Self::EveryOwner,
        Self::EverySemanticValue,
        Self::EveryExplicitValueOrCommitment,
        Self::EveryOutputConstructor,
        Self::SponsorChange,
        Self::FeeRole,
        Self::CoveredTargetProofs,
        Self::VersionAndLockTime,
    ];
}

/// One consumed receipt, as the finalized form fixes it.
///
/// The owner is *recognized* rather than selected (§12.3): the request
/// named an outpoint, the public view gave its program, and exactly one
/// linked constructor produces that program. What the record holds is
/// the conclusion, together with the leaf and control block a spend of
/// it supplies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptInputRecord {
    position: u16,
    outpoint: Outpoint,
    owner: OwnerParameter,
    leaf: LiveTransferLeafRole,
    leaf_script: Vec<u8>,
    control_block: Vec<u8>,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
}

impl ReceiptInputRecord {
    /// One fixed receipt input.
    ///
    /// The three spent-output fields are the ones the public view
    /// already stated for this outpoint, retained rather than read and
    /// dropped. The record used to keep the value alone, which was
    /// enough while the only question asked of it was conservation; the
    /// target's owner message is taken over the spent asset field, the
    /// spent value field and the spent program together, so a form that
    /// kept one of the three would leave a caller to fetch the other two
    /// from beside it — and a census assembled beside the form is
    /// exactly the route the finalized form exists to prevent.
    #[expect(
        clippy::too_many_arguments,
        reason = "one record per consumed receipt, and the arguments are \
                  its members; a parts struct used at the single call site \
                  would be a type whose only purpose is to lower a count"
    )]
    pub(crate) const fn new(
        position: u16,
        outpoint: Outpoint,
        owner: OwnerParameter,
        leaf: LiveTransferLeafRole,
        leaf_script: Vec<u8>,
        control_block: Vec<u8>,
        asset: AssetField,
        value: ValueField,
        program: Vec<u8>,
    ) -> Self {
        Self {
            position,
            outpoint,
            owner,
            leaf,
            leaf_script,
            control_block,
            asset,
            value,
            program,
        }
    }

    /// Which input position the receipt occupies.
    #[must_use]
    pub const fn position(&self) -> u16 {
        self.position
    }

    /// The outpoint the receipt is spent from.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The owner the input authenticates.
    #[must_use]
    pub const fn owner(&self) -> &OwnerParameter {
        &self.owner
    }

    /// The leaf the spend executes.
    #[must_use]
    pub const fn leaf(&self) -> LiveTransferLeafRole {
        self.leaf
    }

    /// The executing leaf's exact committed program.
    #[must_use]
    pub fn leaf_script(&self) -> &[u8] {
        &self.leaf_script
    }

    /// The control block authenticating that leaf.
    #[must_use]
    pub fn control_block(&self) -> &[u8] {
        &self.control_block
    }

    /// The spent output's asset field, as the public view stated it.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The spent output's value field, as the public view stated it.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The spent output's program, as the public view stated it.
    ///
    /// The taproot witness program the receipt sits behind, and not the
    /// leaf: the message's spent-scripts term is taken over each spent
    /// output's `scriptPubKey`, which carries the tweaked output key.
    /// [`Self::leaf_script`] is the program the spend executes and is a
    /// different string entirely.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }
}

/// The output census the finalized form fixes.
///
/// Held beside the transaction rather than read back out of it, because
/// §12.7's post-boundary checks compare an *offered* transaction against
/// what was finalized, and a comparison that read both sides from the
/// same value could not fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalizedOutputCensus {
    outputs: Vec<TargetOutput>,
    destination_range: (u16, u16),
    sponsor_change_position: Option<u16>,
    fee_position: Option<u16>,
}

impl FinalizedOutputCensus {
    /// The census over one settled output list.
    pub(crate) const fn new(
        outputs: Vec<TargetOutput>,
        destination_range: (u16, u16),
        sponsor_change_position: Option<u16>,
        fee_position: Option<u16>,
    ) -> Self {
        Self {
            outputs,
            destination_range,
            sponsor_change_position,
            fee_position,
        }
    }

    /// Every fixed output, in position order.
    #[must_use]
    pub fn outputs(&self) -> &[TargetOutput] {
        &self.outputs
    }

    /// The half-open run of destination positions.
    #[must_use]
    pub const fn destination_range(&self) -> (u16, u16) {
        self.destination_range
    }

    /// Where the sponsor-change role sits, when the form has one.
    #[must_use]
    pub const fn sponsor_change_position(&self) -> Option<u16> {
        self.sponsor_change_position
    }

    /// Where the target fee role sits, when the form has one.
    #[must_use]
    pub const fn fee_position(&self) -> Option<u16> {
        self.fee_position
    }
}

/// One live transfer with §12.6's ten items settled.
///
/// Every field is private, nothing hands out a mutable reference, and
/// the only constructor is `pub(crate)`. After this boundary the only
/// thing that moves is the witness, which the selected profile excludes
/// from the message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalizedLiveTransfer {
    shape: LiveTransferShape,
    representation: LiveTransferRepresentationPlan,
    form: LiveTransactionForm,
    protected: TargetTransaction,
    protected_bytes: Vec<u8>,
    outputs: FinalizedOutputCensus,
    receipts: Vec<ReceiptInputRecord>,
    sponsor_inputs: Vec<Outpoint>,
    required_dimensions: BTreeSet<SighashDimension>,
    protected_data: BTreeSet<ProtectedDatum>,
}

/// Everything one finalization settles, in one value.
///
/// A parameter object rather than twelve arguments, and the twelve are
/// not an accident: §12.6 fixes ten items and a form has to say which
/// shape and which representation it is. Naming them at the call site is
/// also what keeps two `(u16, u16)` ranges from being passed the wrong
/// way round.
pub(crate) struct FinalizedParts {
    pub(crate) shape: LiveTransferShape,
    pub(crate) representation: LiveTransferRepresentationPlan,
    pub(crate) form: LiveTransactionForm,
    pub(crate) protected: TargetTransaction,
    pub(crate) outputs: FinalizedOutputCensus,
    pub(crate) receipts: Vec<ReceiptInputRecord>,
    pub(crate) sponsor_inputs: Vec<Outpoint>,
    pub(crate) required_dimensions: BTreeSet<SighashDimension>,
    pub(crate) protected_data: BTreeSet<ProtectedDatum>,
}

/// The exact preimage every owner of `protected` is asked to bind to,
/// under `representation`.
///
/// # Why the private lane's answer is not the witnessless serialization
///
/// The target's `SIGHASH_ALL` incorporates the serialized output set AND
/// the hash of the output-witness vector, and it hashes that vector at
/// whatever length the vector happens to have. A transaction carrying no
/// witness deserializes with an EMPTY output-witness vector and its
/// signer hashes the empty string there, while serializing a transaction
/// that has any witness grows the vector to one entry per output and
/// consensus then hashes one entry per output. The two digests differ,
/// the signature is reported complete, and the target refuses the spend
/// — which is the recorded, runnable diagnosis behind
/// `(´[PLAN-obs:upstream:eg-019]´)`.
///
/// `encode_without_witness()` is exactly the empty-vector case. For an
/// explicit candidate that costs nothing, because there are no proofs to
/// omit and the empty vector is what the wire carries anyway. For a
/// proof-bearing candidate it is the difference between a declared
/// commitment and an actual one: an owner would be asked to bind to a
/// preimage that does not contain the range proofs the target's digest
/// covers.
///
/// So the private lane's preimage is the witnessless serialization
/// followed by the output-witness vector, and changing one proof byte
/// changes it. The scope is deliberate. The explicit lane's preimage is
/// unchanged, byte for byte, because its output-witness vector is empty
/// under both readings and widening a settled lane's bytes to say
/// nothing new would be a change for symmetry's sake.
///
/// # This is a preimage and not a digest
///
/// The concatenation is a named region census and is not offered as a
/// target serialization of anything: no target ever parses it. Which
/// message the owner-sighash work computes from these bytes, and how, is
/// that work's own and is not decided here.
pub(crate) fn protected_preimage(
    protected: &TargetTransaction,
    representation: LiveTransferRepresentationPlan,
) -> Vec<u8> {
    let mut bytes = protected.encode_without_witness();
    if matches!(
        representation,
        LiveTransferRepresentationPlan::PrivateCommitted
    ) {
        bytes.extend_from_slice(&protected.output_witness_bytes());
    }
    bytes
}

impl FinalizedLiveTransfer {
    /// The finalized form over one settled transaction.
    pub(crate) fn new(parts: FinalizedParts) -> Self {
        let protected_bytes = protected_preimage(&parts.protected, parts.representation);
        Self {
            shape: parts.shape,
            representation: parts.representation,
            form: parts.form,
            protected: parts.protected,
            protected_bytes,
            outputs: parts.outputs,
            receipts: parts.receipts,
            sponsor_inputs: parts.sponsor_inputs,
            required_dimensions: parts.required_dimensions,
            protected_data: parts.protected_data,
        }
    }

    /// The shape the transfer realizes.
    #[must_use]
    pub const fn shape(&self) -> LiveTransferShape {
        self.shape
    }

    /// The representation plan the transfer was built under.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// Which transaction form was built.
    #[must_use]
    pub const fn form(&self) -> LiveTransactionForm {
        self.form
    }

    /// The settled transaction, with no witness.
    #[must_use]
    pub const fn protected(&self) -> &TargetTransaction {
        &self.protected
    }

    /// The exact protected serialization every owner is asked to bind
    /// to.
    ///
    /// The preimage, not a digest. §1.7 leaves the digest to the target,
    /// and a builder that supplied one would be asserting the profile it
    /// intended rather than letting the profile be recomputed from the
    /// request.
    ///
    /// Which regions the preimage covers depends on the representation.
    /// The explicit lane's preimage is the witnessless serialization; the
    /// private lane's is that serialization followed by the
    /// output-witness vector, so an owner binds to bytes containing the
    /// range proofs the target's digest covers. The two are not
    /// symmetric on purpose: the explicit lane's vector is empty under
    /// both readings, so its bytes are unchanged byte for byte, and
    /// widening a settled lane's preimage to say nothing new would move
    /// a signature's preimage under every existing owner for nothing.
    ///
    /// The full argument, with the recorded upstream diagnosis it turns
    /// on, is at the crate-private `protected_preimage`, which is what
    /// computes these bytes. It is named rather than linked because a
    /// public item may not link a private one, and it is summarized
    /// rather than merely named so that a reader with only the public
    /// documentation still gets the answer instead of a pointer to
    /// something they cannot open.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        &self.protected_bytes
    }

    /// The fixed output census.
    #[must_use]
    pub const fn outputs(&self) -> &FinalizedOutputCensus {
        &self.outputs
    }

    /// Every consumed receipt, in input order.
    #[must_use]
    pub fn receipts(&self) -> &[ReceiptInputRecord] {
        &self.receipts
    }

    /// The sponsor suffix's outpoints, in canonical order.
    #[must_use]
    pub fn sponsor_inputs(&self) -> &[Outpoint] {
        &self.sponsor_inputs
    }

    /// Which of §12.6's ten items this form settled.
    ///
    /// All ten, for every finalized form: a value of this type that
    /// settled nine would be the thing §12.6 forbids, so the method
    /// reports the census rather than a subset of it. It exists so that
    /// the census can be read from the value a signing request came
    /// from, which is where a reviewer will look for it.
    #[must_use]
    pub fn settled(&self) -> BTreeSet<FinalizedFact> {
        FinalizedFact::ALL.iter().copied().collect()
    }

    /// The dimensions the selected profile requires a signature to
    /// commit to.
    #[must_use]
    pub const fn required_dimensions(&self) -> &BTreeSet<SighashDimension> {
        &self.required_dimensions
    }

    /// The protected data every owner signature commits to (§1.7).
    #[must_use]
    pub const fn protected_data(&self) -> &BTreeSet<ProtectedDatum> {
        &self.protected_data
    }

    /// The signing request for one consumed receipt.
    ///
    /// One per receipt input rather than one per owner: §1.6 keeps
    /// semantic authorization and target realization distinct, and the
    /// target's message is input-specific, so one owner holding three
    /// receipts answers three requests.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::ReceiptPositionOutsideFamily`] for a
    /// position no receipt occupies.
    pub fn signing_request(&self, input: u16) -> Result<LiveSigningRequest, TransactionRefusal> {
        let record = self
            .receipts
            .iter()
            .find(|receipt| receipt.position == input)
            .ok_or(TransactionRefusal::ReceiptPositionOutsideFamily { position: input })?;

        Ok(LiveSigningRequest::new(
            input,
            record.owner.clone(),
            self.representation,
            self.protected_bytes.clone(),
            record.leaf,
            self.required_dimensions.clone(),
            self.protected_data.clone(),
        ))
    }

    /// Every signing request, in input order.
    #[must_use]
    pub fn signing_requests(&self) -> Vec<LiveSigningRequest> {
        self.receipts
            .iter()
            .map(|record| {
                LiveSigningRequest::new(
                    record.position,
                    record.owner.clone(),
                    self.representation,
                    self.protected_bytes.clone(),
                    record.leaf,
                    self.required_dimensions.clone(),
                    self.protected_data.clone(),
                )
            })
            .collect()
    }

    /// Whether an offered transaction is the one that was finalized
    /// (§12.7).
    ///
    /// The three post-boundary rejections, each named for what happened
    /// rather than for the comparison that caught it. The witness is not
    /// compared: the selected profile excludes it from the message, and
    /// §12.6 admits exactly those fields moving.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::InputExtendedAfterSigning`] for an offering
    /// with a different input census;
    /// [`TransactionRefusal::OutputOmittedAfterSigning`] for an offering
    /// with fewer outputs; and
    /// [`TransactionRefusal::OutputMutatedAfterSigning`] for an offering
    /// whose output at some position is not the one that was fixed.
    /// Version and locktime changes are reported as a mutation of the
    /// output census's first position, because they are protected data
    /// the same signature covers and no narrower report would be true.
    pub fn check_offered(&self, offered: &TargetTransaction) -> Result<(), TransactionRefusal> {
        let finalized_inputs = self.protected.inputs();
        if offered.inputs().len() != finalized_inputs.len()
            || offered
                .inputs()
                .iter()
                .zip(finalized_inputs)
                .any(|(offered, fixed)| offered.outpoint() != fixed.outpoint())
        {
            return Err(TransactionRefusal::InputExtendedAfterSigning {
                finalized: finalized_inputs.len(),
                offered: offered.inputs().len(),
            });
        }

        let fixed = self.outputs.outputs();
        if offered.outputs().len() < fixed.len() {
            return Err(TransactionRefusal::OutputOmittedAfterSigning {
                position: position_of(offered.outputs().len()),
            });
        }
        if offered.outputs().len() > fixed.len() {
            return Err(TransactionRefusal::OutputMutatedAfterSigning {
                position: position_of(fixed.len()),
            });
        }
        for (index, (offered, fixed)) in offered.outputs().iter().zip(fixed).enumerate() {
            if offered != fixed {
                return Err(TransactionRefusal::OutputMutatedAfterSigning {
                    position: position_of(index),
                });
            }
        }

        if offered.version() != self.protected.version()
            || offered.lock_time() != self.protected.lock_time()
        {
            return Err(TransactionRefusal::OutputMutatedAfterSigning { position: 0 });
        }

        Ok(())
    }
}

/// One protocol owner's signing request (§1.7).
///
/// No constructor outside this crate, and inside it only the finalized
/// form calls one. §1.7's rule is that no request exists until all
/// protected data is fixed, and the way to have that rule hold is for
/// the only route to a request to run through a value that fixed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveSigningRequest {
    input: u16,
    owner: OwnerParameter,
    representation: LiveTransferRepresentationPlan,
    protected_bytes: Vec<u8>,
    leaf: LiveTransferLeafRole,
    required_dimensions: BTreeSet<SighashDimension>,
    protected_data: BTreeSet<ProtectedDatum>,
}

impl LiveSigningRequest {
    /// The request for one fixed receipt input.
    pub(crate) const fn new(
        input: u16,
        owner: OwnerParameter,
        representation: LiveTransferRepresentationPlan,
        protected_bytes: Vec<u8>,
        leaf: LiveTransferLeafRole,
        required_dimensions: BTreeSet<SighashDimension>,
        protected_data: BTreeSet<ProtectedDatum>,
    ) -> Self {
        Self {
            input,
            owner,
            representation,
            protected_bytes,
            leaf,
            required_dimensions,
            protected_data,
        }
    }

    /// Which input this request authorizes.
    #[must_use]
    pub const fn input(&self) -> u16 {
        self.input
    }

    /// The owner this request asks to authorize it.
    #[must_use]
    pub const fn owner(&self) -> &OwnerParameter {
        &self.owner
    }

    /// The representation plan the transfer was built under.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// The exact protected serialization to bind to.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        &self.protected_bytes
    }

    /// The leaf the spend of this input executes.
    #[must_use]
    pub const fn leaf(&self) -> LiveTransferLeafRole {
        self.leaf
    }

    /// The dimensions a conforming signature commits to.
    #[must_use]
    pub const fn required_dimensions(&self) -> &BTreeSet<SighashDimension> {
        &self.required_dimensions
    }

    /// The protected data those dimensions carry (§1.7).
    #[must_use]
    pub const fn protected_data(&self) -> &BTreeSet<ProtectedDatum> {
        &self.protected_data
    }
}

/// One output index as a position, saturating rather than wrapping.
///
/// A transaction with more than `u16::MAX` outputs is not one this
/// crate's own construction can build, and reporting the last
/// representable position is a true statement about where the report
/// stopped being exact.
fn position_of(index: usize) -> u16 {
    u16::try_from(index).unwrap_or(u16::MAX)
}
