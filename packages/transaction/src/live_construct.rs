//! Live-transfer construction, in stages, each refusing rather than
//! degrading (§12.1, §12.2, §12.5, §12.8).
//!
//! # Three entry points, because the boundary is in the middle
//!
//! §12.6 puts a boundary between building and signing, so a single
//! `construct` would have to invent the owners' answers. The three
//! stages are [`finalize_live_transfer`], which settles §12.6's ten
//! items and hands back a form nothing can unsettle;
//! [`crate::live_signing::authorize_live_transfer`], which collects the
//! answers; and [`complete_live_transfer`], which attaches the witnesses
//! the profile excludes from the message and produces candidate bytes.
//!
//! # Form exactness is checked as an equivalence, not as a default
//!
//! §12.5 states a five-way iff and a five-step implication chain, and
//! every arrow of both is a refusal here rather than a fallback. An
//! empty sponsor capability in particular does not downgrade a sponsored
//! request: the two forms have different input censuses, different
//! output censuses, and different target versions, and building the
//! other one silently would be handing back a transaction the caller did
//! not ask for.
//!
//! # The predecessor owner is recognized, never selected
//!
//! §12.3 refuses a request that names the predecessor owner, so
//! construction finds it: the request names an outpoint, the public view
//! gives that outpoint's program, and exactly one linked constructor
//! produces that program. An outpoint whose program no constructor
//! produces is not a live receipt of this deployment, and that is a
//! refusal rather than a receipt with an unknown owner.
//!
//! # Candidate bytes, and nothing further
//!
//! What comes out is a complete target transaction and its
//! serialization. There is no digest, no submission, and no claim about
//! what a target would do with it (§1.11, §12.9).

use std::collections::{BTreeMap, BTreeSet};

use linker::OwnerParameter;
use linker::live_backend::{LiveTransferRepresentationPlan, LiveTransferShape};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::abi::TargetTransactionVersion;
use crate::bytes::{
    AssetField, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput, TargetTransaction,
    ValueField,
};
use crate::construct::check_weight;
use crate::error::TransactionRefusal;
use crate::live_abi::{CandidateLiveTransferAbi, LiveShapeAbi, LiveTransactionForm};
use crate::live_finalize::{
    FinalizedLiveTransfer, FinalizedOutputCensus, FinalizedParts, ReceiptInputRecord,
    SpentSponsorOutput,
};
use crate::live_materialize::{
    ConfidentialConstructionIntent, ConfidentialDestinationIntent, ConfidentialInputIntent,
    ConfidentialMaterializationProfiles, ConfidentialOutputRole, ConfidentialProofMaterializer,
    FixtureOpeningReference, FrozenConfidentialFixtureView, IndependentCommitmentCheck,
    MaterializedConfidentialCandidate, NonProtocolFundingRegion, SCALAR_BYTES,
    materialize_confidential_candidate,
};
use crate::live_private::{
    ConfidentialConstructionModel, PrivateValueCapability, SelectedConstructionModel,
};
use crate::live_request::{LiveTransferRequest, RequestedForm};
use crate::live_signing::AuthorizedLiveTransfer;
use crate::sponsor::{SighashProfile, SignerRole, SponsorCapability, SponsorSigningRequest};
use crate::taproot::witness_program_script;
use crate::view::PublicConstructionView;

/// The sequence number every live-transfer input carries.
///
/// The final value, which is what an input that is not opting into
/// relative timelocks or replaceability carries. §12.3 refuses a request
/// that selects it and §7.3 makes the live class structural, so it is a
/// constant rather than a parameter.
pub const LIVE_TRANSFER_SEQUENCE: u32 = 0xffff_ffff;

/// The owner census one live transfer realizes (§1.6).
///
/// The set and the count are separate fields because they answer
/// separate questions, and a report that carried one of them would be
/// the collapse §1.6 warns against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveOwnerCensus {
    distinct_semantic_owners: BTreeSet<OwnerParameter>,
    receipt_inputs: usize,
}

impl LiveOwnerCensus {
    /// Every distinct semantic owner whose receipt is consumed.
    #[must_use]
    pub const fn distinct_semantic_owners(&self) -> &BTreeSet<OwnerParameter> {
        &self.distinct_semantic_owners
    }

    /// How many concrete receipt inputs are consumed.
    #[must_use]
    pub const fn receipt_inputs(&self) -> usize {
        self.receipt_inputs
    }
}

/// What one live-transfer construction settled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveConstructionReport {
    shape: LiveTransferShape,
    representation: LiveTransferRepresentationPlan,
    form: LiveTransactionForm,
    version: TargetTransactionVersion,
    owners: LiveOwnerCensus,
    destinations: usize,
    created_total: u64,
    consumed_total: Option<u64>,
    model: Option<SelectedConstructionModel>,
}

impl LiveConstructionReport {
    /// The shape the transfer realized.
    #[must_use]
    pub const fn shape(&self) -> LiveTransferShape {
        self.shape
    }

    /// The representation plan it was built under.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// Which transaction form was built.
    #[must_use]
    pub const fn form(&self) -> LiveTransactionForm {
        self.form
    }

    /// The target transaction version.
    #[must_use]
    pub const fn version(&self) -> TargetTransactionVersion {
        self.version
    }

    /// The §1.6 owner census.
    #[must_use]
    pub const fn owners(&self) -> &LiveOwnerCensus {
        &self.owners
    }

    /// How many destinations were created.
    #[must_use]
    pub const fn destinations(&self) -> usize {
        self.destinations
    }

    /// The semantic total the destinations carry.
    #[must_use]
    pub const fn created_total(&self) -> u64 {
        self.created_total
    }

    /// The explicit total the consumed receipts carried.
    ///
    /// `None` under the private representation, and `None` there because
    /// the amounts are commitments rather than because the sum was not
    /// computed. §6.4 refuses a public subtotal of private amounts, and
    /// a report carrying one would be exactly that.
    #[must_use]
    pub const fn consumed_total(&self) -> Option<u64> {
        self.consumed_total
    }

    /// The confidential construction model, where one was recorded.
    #[must_use]
    pub const fn construction_model(&self) -> Option<&SelectedConstructionModel> {
        self.model.as_ref()
    }
}

/// One finalized transfer and the report of how it was built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveFinalization {
    finalized: FinalizedLiveTransfer,
    report: LiveConstructionReport,
}

impl LiveFinalization {
    /// The finalized form.
    #[must_use]
    pub const fn finalized(&self) -> &FinalizedLiveTransfer {
        &self.finalized
    }

    /// The construction report.
    #[must_use]
    pub const fn report(&self) -> &LiveConstructionReport {
        &self.report
    }

    /// The finalized form, taken out for signing.
    #[must_use]
    pub fn into_finalized(self) -> FinalizedLiveTransfer {
        self.finalized
    }
}

/// One complete candidate live-transfer transaction (§12.9).
///
/// Candidate bytes and nothing further: no digest, no submission, and no
/// verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateLiveTransferTransaction {
    transaction: TargetTransaction,
    report: LiveConstructionReport,
    authorized: AuthorizedLiveTransfer,
}

impl CandidateLiveTransferTransaction {
    /// The complete transaction, witnesses attached.
    #[must_use]
    pub const fn transaction(&self) -> &TargetTransaction {
        &self.transaction
    }

    /// The candidate serialization.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.transaction.encode()
    }

    /// The construction report.
    #[must_use]
    pub const fn report(&self) -> &LiveConstructionReport {
        &self.report
    }

    /// The authorized finalized form the witnesses answer.
    #[must_use]
    pub const fn authorized(&self) -> &AuthorizedLiveTransfer {
        &self.authorized
    }
}

/// Settle §12.6's ten items for one request.
///
/// # Errors
///
/// [`TransactionRefusal::RepresentationNotLinked`] when the ABI carries
/// no constructor for the plan the request selected; every §12.5
/// form-exactness refusal; the three §12.1 pre-sort
/// rejections that reach construction
/// ([`TransactionRefusal::SponsorOverlapsReceiptFamily`],
/// [`TransactionRefusal::MissingPublicReceiptView`], and the duplicate
/// the request already refused);
/// [`TransactionRefusal::UnsupportedLiveShape`] when no admitted shape
/// realizes the counts;
/// [`TransactionRefusal::ReceiptInputIsNotALiveReceipt`],
/// [`TransactionRefusal::ReceiptInputCarriesForeignAsset`] and
/// [`TransactionRefusal::ReceiptInputValueFormRefused`] for a selected
/// outpoint the deployment does not recognize;
/// [`TransactionRefusal::MissingPublicSponsorView`] and
/// [`TransactionRefusal::LiveSponsorInputCarriesForeignAsset`] for an
/// offered sponsor input the caller cannot show, or shows holding an
/// asset that is not the deployment's reserve;
/// [`TransactionRefusal::DestinationOwnerHasNoConstructor`] for a
/// destination owner nothing was linked for;
/// [`TransactionRefusal::DestinationTotalOutOfRange`] and
/// [`TransactionRefusal::LiveConservationFailed`] for arithmetic that
/// does not close; and
/// [`TransactionRefusal::PrivateValueCapabilityAbsent`] for a private
/// request with no confidential capability.
pub fn finalize_live_transfer(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    view: &PublicConstructionView,
    sponsor: Option<&dyn SponsorCapability>,
    private: Option<&dyn PrivateValueCapability>,
) -> Result<LiveFinalization, TransactionRefusal> {
    // Before §12.6's ten items: the ABI has to carry the plan the
    // request selected. A linked candidate legitimately carries one
    // representation — an explicit-only link is a link, not a defect —
    // and the request selects its plan without ever seeing the ABI, so
    // the two can disagree about which plans exist.
    //
    // Refusing here is about accuracy rather than safety. Without it the
    // absent plan is met four stages later, where receipt recognition
    // searches the constructors of a plan that was never linked, finds
    // none, and answers `ReceiptInputIsNotALiveReceipt` — a true
    // sentence about a receipt that is fine, naming the wrong cause.
    // The owner-specific refusal below stays for the case it is about:
    // the plan is linked and one destination owner has no constructor
    // in it.
    if !abi.representations().contains(&request.representation()) {
        return Err(TransactionRefusal::RepresentationNotLinked);
    }

    // Stage 1: §12.5's equivalence, both directions, before anything is
    // built from either side of it.
    let offer = check_form_exactness(request, sponsor)?;

    // Stage 2: the representation's own capability requirement. The
    // request already settled that a private form carries published
    // randomness; this settles that something can turn it into a field.
    let model = match (request.representation(), private) {
        (LiveTransferRepresentationPlan::Explicit, Some(_)) => {
            return Err(TransactionRefusal::PrivateValueCapabilityWithoutPrivateForm);
        }
        (LiveTransferRepresentationPlan::PrivateCommitted, None) => {
            return Err(TransactionRefusal::PrivateValueCapabilityAbsent);
        }
        (LiveTransferRepresentationPlan::PrivateCommitted, Some(_)) => Some(
            SelectedConstructionModel::record(ConfidentialConstructionModel::EXPECTED)?,
        ),
        (LiveTransferRepresentationPlan::Explicit, None) => None,
    };

    // Stage 3: protocol and sponsor regions must be disjoint, and the
    // rejection is before the sort rather than after it (§12.1).
    let sponsor_inputs: Vec<Outpoint> = offer
        .as_ref()
        .map(|offer| offer.inputs().iter().copied().collect())
        .unwrap_or_default();
    let sponsor_spent = recognize_sponsors(abi, request, view, &sponsor_inputs)?;

    // Stage 4: the shape, chosen by every count at once.
    let shape = select_shape(abi, request, sponsor_inputs.len())?;

    // Stage 5: recognize each consumed receipt.
    let recognized = recognize_receipts(abi, request, view)?;

    // Stage 6: the destinations' arithmetic, and the conservation the
    // explicit plan closes locally.
    let created_total = request
        .destination_total()
        .ok_or(TransactionRefusal::DestinationTotalOutOfRange)?;
    let consumed_total = explicit_total(&recognized)?;
    if let Some(consumed) = consumed_total
        && consumed != created_total
    {
        return Err(TransactionRefusal::LiveConservationFailed {
            consumed,
            created: created_total,
        });
    }

    // Stage 7: inputs, receipts first in canonical order and the sponsor
    // suffix after, each run already sorted by the set it came from.
    let inputs: Vec<TargetInput> = request
        .receipts()
        .iter()
        .chain(sponsor_inputs.iter())
        .map(|outpoint| TargetInput::new(*outpoint, LIVE_TRANSFER_SEQUENCE))
        .collect();

    // Stage 8: outputs, destinations in typed request order and the two
    // optional roles after them.
    let outputs = assemble_outputs(
        target,
        abi,
        shape,
        request,
        offer.as_ref(),
        sponsor,
        private,
    )?;

    // Stage 9: the protected transaction. Null witnesses, because the
    // witness is what §12.6 admits moving afterwards.
    let protected = TargetTransaction::new(
        shape.version().version(),
        inputs,
        outputs.clone(),
        abi.lock_time(),
        vec![InputWitness::default(); request.receipts().len() + sponsor_inputs.len()],
    )?;

    // Stage 10: the receipt records, each carrying the leaf its own
    // position executes.
    let receipts = receipt_records(abi, request, shape, &recognized)?;

    let owners = LiveOwnerCensus {
        distinct_semantic_owners: receipts
            .iter()
            .map(|record| record.owner().clone())
            .collect(),
        receipt_inputs: receipts.len(),
    };

    let report = LiveConstructionReport {
        shape: shape.shape(),
        representation: request.representation(),
        form: shape.form(),
        version: shape.version(),
        owners,
        destinations: request.destinations().len(),
        created_total,
        consumed_total,
        model,
    };

    let finalized = FinalizedLiveTransfer::new(FinalizedParts {
        shape: shape.shape(),
        representation: request.representation(),
        form: shape.form(),
        protected,
        outputs: FinalizedOutputCensus::new(
            outputs,
            shape.destination_range(),
            shape.sponsor_change_position(),
            shape.fee_position(),
        ),
        receipts,
        sponsor_inputs,
        sponsor_spent,
        required_dimensions: abi.sighash_profile().profile().required().collect(),
        protected_data: abi.protected_data().clone(),
    });

    Ok(LiveFinalization { finalized, report })
}

/// Attach the witnesses and produce candidate bytes.
///
/// # Errors
///
/// [`TransactionRefusal::SponsorSignatureMissing`] and
/// [`TransactionRefusal::SponsorSignatureBindingMismatch`] for a sponsor
/// that does not answer or answers about other bytes; and any refusal of
/// [`check_weight`], which is the same preflight the compact-ASH
/// pipeline runs.
pub fn complete_live_transfer(
    target: &ReviewedElementsTapscriptDefinition,
    authorized: AuthorizedLiveTransfer,
    report: LiveConstructionReport,
    sponsor: Option<&dyn SponsorCapability>,
) -> Result<CandidateLiveTransferTransaction, TransactionRefusal> {
    let finalized = authorized.finalized();
    let protected = finalized.protected();

    let mut witnesses = Vec::with_capacity(protected.inputs().len());
    for (index, input) in protected.inputs().iter().enumerate() {
        let position = u16::try_from(index).unwrap_or(u16::MAX);
        if let Some(witness) = authorized.witnesses().get(&position) {
            witnesses.push(witness.clone());
            continue;
        }

        // A sponsor input. §1.9 keeps the sponsor's own authorization
        // outside protocol data, so it is collected here rather than
        // through the owner responses.
        let capability =
            sponsor.ok_or(TransactionRefusal::LiveSponsorRequestedWithoutCapability)?;
        let request = SponsorSigningRequest::new(
            finalized.protected_bytes().to_vec(),
            position,
            SignerRole::SponsorSuffixMember,
            SighashProfile::AllInputsAllOutputs,
            finalized.outputs().outputs().to_vec(),
        );
        let outpoint = input.outpoint();
        let signature = capability
            .sign(&request)
            .ok_or(TransactionRefusal::SponsorSignatureMissing(outpoint))?;
        if signature.bound_to() != finalized.protected_bytes() {
            return Err(TransactionRefusal::SponsorSignatureBindingMismatch(
                outpoint,
            ));
        }
        witnesses.push(InputWitness::new(signature.stack().to_vec()));
    }

    let transaction = TargetTransaction::new(
        protected.version(),
        protected.inputs().to_vec(),
        protected.outputs().to_vec(),
        protected.lock_time(),
        witnesses,
    )?;
    check_weight(target, &transaction)?;

    Ok(CandidateLiveTransferTransaction {
        transaction,
        report,
        authorized,
    })
}

/// §12.5's equivalence and implication chain, both directions.
///
/// Returns the sponsor's offer for a sponsored request and nothing for a
/// sponsorless one, which is the fourth and fifth arrows of the chain
/// made structural: a sponsorless build has no offer to read a change
/// value or a fee from.
fn check_form_exactness(
    request: &LiveTransferRequest,
    sponsor: Option<&dyn SponsorCapability>,
) -> Result<Option<crate::sponsor::SponsorOffer>, TransactionRefusal> {
    match (request.form(), sponsor) {
        (RequestedForm::Sponsored, None) => {
            Err(TransactionRefusal::LiveSponsorRequestedWithoutCapability)
        }
        (RequestedForm::Sponsorless, Some(_)) => {
            Err(TransactionRefusal::LiveSponsorCapabilityWithoutRequest)
        }
        (RequestedForm::Sponsorless, None) => Ok(None),
        (RequestedForm::Sponsored, Some(capability)) => {
            let offer = capability.offer();
            // The third term. An empty capability funds no fee, and
            // accepting it would build the sponsorless form for a
            // request that asked for the sponsored one.
            if offer.inputs().is_empty() {
                return Err(TransactionRefusal::EmptyLiveSponsorOffer);
            }
            match (request.sponsor_change().requested(), offer.change()) {
                (true, None) => Err(TransactionRefusal::SponsorChangeRequestedWithoutDestination),
                (false, Some(_)) => Err(TransactionRefusal::SponsorChangeOfferedWithoutRequest),
                _ => Ok(Some(offer)),
            }
        }
    }
}

/// The one admitted shape realizing every requested count.
///
/// Five conjuncts, and the form is one of them rather than a consequence
/// of the others: §12.5 reports the form as its own term, and a shape
/// selected by counts alone could satisfy them under the other form.
fn select_shape<'abi>(
    abi: &'abi CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    sponsor_inputs: usize,
) -> Result<&'abi LiveShapeAbi, TransactionRefusal> {
    let receipt_inputs = request.receipts().len();
    let destinations = request.destinations().len();
    let sponsor_change = request.sponsor_change().requested();
    let wanted = if request.form().sponsored() {
        LiveTransactionForm::Sponsored
    } else {
        LiveTransactionForm::Sponsorless
    };

    abi.shapes()
        .values()
        .find(|candidate| {
            let shape = candidate.shape();
            usize::from(shape.receipt_inputs()) == receipt_inputs
                && usize::from(shape.receipt_outputs()) == destinations
                && usize::from(shape.sponsor_inputs()) == sponsor_inputs
                && candidate.sponsor_change_position().is_some() == sponsor_change
                && candidate.form() == wanted
        })
        .ok_or(TransactionRefusal::UnsupportedLiveShape {
            receipt_inputs,
            destinations,
            sponsor_inputs,
            sponsor_change,
        })
}

/// One recognized receipt: its owner and the three spent-output fields
/// the view stated.
///
/// The asset and the program are carried rather than checked and
/// dropped. Both are read here already — the asset to refuse a foreign
/// one, the program to recognize the owner — and both are terms of the
/// target's owner message, so the form that comes out of this
/// construction can carry what the message needs instead of leaving a
/// later caller to fetch it from somewhere else.
struct RecognizedReceipt {
    outpoint: Outpoint,
    owner: OwnerParameter,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
}

/// Check every offered sponsor input, before the sort (§12.1, §10.7).
///
/// Two rejections, in the order a caller can act on them. The regions
/// must be disjoint, and each offered input must be a coin the caller
/// can show holding the deployment's reserve asset.
///
/// # Why the asset is checked here rather than left to the target
///
/// §10.7 isolates the sponsor and fee roles in the reserve asset, and
/// the fee output this build is about to write names that asset as a
/// literal the deployment welded in. A sponsor input carrying anything
/// else funds that output in an asset it does not hold, which is a
/// transaction that cannot balance.
///
/// The compact-ASH lane has refused exactly this since it had a sponsor
/// region, and the absence here was never a decision:
/// [`TransactionRefusal::LiveSponsorInputCarriesForeignAsset`] was
/// already minted for this check and nothing had ever raised it.
/// Without it the mismatch stays invisible until a node reads the
/// transaction — and a construction defect reported by a target is a
/// defect reported at the wrong layer.
fn recognize_sponsors(
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    view: &PublicConstructionView,
    sponsor_inputs: &[Outpoint],
) -> Result<Vec<SpentSponsorOutput>, TransactionRefusal> {
    let mut spent = Vec::with_capacity(sponsor_inputs.len());
    for outpoint in sponsor_inputs {
        if request.receipts().contains(outpoint) {
            return Err(TransactionRefusal::SponsorOverlapsReceiptFamily(*outpoint));
        }
        let stated = view
            .get(*outpoint)
            .ok_or(TransactionRefusal::MissingPublicSponsorView(*outpoint))?;
        if stated.asset() != AssetField::Explicit(abi.symbols().reserve_asset()) {
            return Err(TransactionRefusal::LiveSponsorInputCarriesForeignAsset(
                *outpoint,
            ));
        }

        // Kept, not merely checked. The owner's signature commits to
        // every spent output, and this is the last place that holds a
        // view of the sponsor region.
        spent.push(SpentSponsorOutput::new(
            stated.asset(),
            stated.value(),
            stated.program().to_vec(),
        ));
    }
    Ok(spent)
}

/// Recognize every selected outpoint as some owner's live receipt.
fn recognize_receipts(
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    view: &PublicConstructionView,
) -> Result<Vec<RecognizedReceipt>, TransactionRefusal> {
    request
        .receipts()
        .iter()
        .map(|outpoint| {
            let stated = view
                .get(*outpoint)
                .ok_or(TransactionRefusal::MissingPublicReceiptView(*outpoint))?;

            if stated.asset() != AssetField::Explicit(abi.symbols().protocol_asset()) {
                return Err(TransactionRefusal::ReceiptInputCarriesForeignAsset(
                    *outpoint,
                ));
            }

            // §6.3 keeps the asset explicit under both plans and the
            // value confidential under only one, so the value field's
            // form is what the representation decides.
            let admitted = matches!(
                (request.representation(), stated.value()),
                (
                    LiveTransferRepresentationPlan::Explicit,
                    ValueField::Explicit(_)
                ) | (
                    LiveTransferRepresentationPlan::PrivateCommitted,
                    ValueField::Commitment(_),
                )
            );
            if !admitted {
                return Err(TransactionRefusal::ReceiptInputValueFormRefused(*outpoint));
            }

            let owner = abi
                .destinations()
                .owner_of_program(request.representation(), stated.program())
                .ok_or(TransactionRefusal::ReceiptInputIsNotALiveReceipt(*outpoint))?;

            Ok(RecognizedReceipt {
                outpoint: *outpoint,
                owner: owner.clone(),
                asset: stated.asset(),
                value: stated.value(),
                program: stated.program().to_vec(),
            })
        })
        .collect()
}

/// The explicit total the consumed receipts carry, where they carry one.
fn explicit_total(recognized: &[RecognizedReceipt]) -> Result<Option<u64>, TransactionRefusal> {
    let mut total = 0_u64;
    for receipt in recognized {
        match receipt.value {
            ValueField::Explicit(amount) => {
                total = total
                    .checked_add(amount)
                    .ok_or(TransactionRefusal::DestinationTotalOutOfRange)?;
            }
            // A commitment has no explicit amount to add, and §6.4
            // refuses a public subtotal of private ones, so the whole
            // total is absent rather than partial.
            ValueField::Commitment(_) => return Ok(None),
        }
    }
    Ok(Some(total))
}

/// The destinations, then the sponsor change, then the fee role.
fn assemble_outputs(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    shape: &LiveShapeAbi,
    request: &LiveTransferRequest,
    offer: Option<&crate::sponsor::SponsorOffer>,
    sponsor: Option<&dyn SponsorCapability>,
    private: Option<&dyn PrivateValueCapability>,
) -> Result<Vec<TargetOutput>, TransactionRefusal> {
    let mut placed: BTreeMap<u16, TargetOutput> = BTreeMap::new();
    let (first, _) = shape.destination_range();

    for (index, destination) in request.destinations().iter().enumerate() {
        let position = first.saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        let constructor = abi
            .destinations()
            .get(destination.owner(), request.representation())
            .ok_or_else(|| TransactionRefusal::DestinationOwnerHasNoConstructor {
                owner: destination.owner().clone(),
            })?;

        // The private ordering never depends on the amount, the opening,
        // or the blinding material (§12.2): the position is the request
        // index, and the field is built from it rather than the other
        // way round.
        let value = match private {
            None => ValueField::Explicit(destination.value().amount()),
            Some(capability) => {
                let randomness = request
                    .public_test_randomness()
                    .ok_or(TransactionRefusal::PrivateFormWithoutPublicTestRandomness)?;
                ValueField::Commitment(
                    capability
                        .value_commitment(
                            abi.symbols().protocol_asset(),
                            destination.value(),
                            &randomness,
                            position,
                        )
                        .ok_or(TransactionRefusal::DestinationValueCommitmentUndetermined {
                            position,
                        })?,
                )
            }
        };

        placed.insert(
            position,
            TargetOutput::new(
                AssetField::Explicit(abi.symbols().protocol_asset()),
                value,
                NonceField::Null,
                constructor.instance().program().to_vec(),
            ),
        );
    }

    if let (Some(position), Some(offer)) = (shape.sponsor_change_position(), offer) {
        let value = offer
            .change()
            .ok_or(TransactionRefusal::SponsorChangeRequestedWithoutDestination)?;
        let destination = sponsor.and_then(SponsorCapability::change_destination);
        let (version, payload) = destination.unwrap_or_else(|| {
            (
                abi.symbols().sponsor_change_version(),
                abi.symbols().sponsor_change_program().to_vec(),
            )
        });
        if version != abi.symbols().sponsor_change_version()
            || payload != abi.symbols().sponsor_change_program()
        {
            return Err(TransactionRefusal::MalformedLiveDeploymentSymbol {
                symbol: "sponsor change program",
            });
        }
        placed.insert(
            position,
            TargetOutput::new(
                AssetField::Explicit(abi.symbols().reserve_asset()),
                value,
                NonceField::Null,
                witness_program_script(target, version, &payload)?,
            ),
        );
    }

    if let (Some(position), Some(offer)) = (shape.fee_position(), offer) {
        placed.insert(
            position,
            TargetOutput::new(
                AssetField::Explicit(abi.symbols().reserve_asset()),
                ValueField::Explicit(offer.fee()),
                NonceField::Null,
                // The empty program: the fee role's whole identity is
                // target-structural, and this is the structure.
                Vec::new(),
            ),
        );
    }

    Ok(placed.into_values().collect())
}

/// One record per receipt input, with the leaf its position executes.
fn receipt_records(
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    shape: &LiveShapeAbi,
    recognized: &[RecognizedReceipt],
) -> Result<Vec<ReceiptInputRecord>, TransactionRefusal> {
    let (first, _) = shape.receipt_input_range();

    recognized
        .iter()
        .enumerate()
        .map(|(index, receipt)| {
            let position = first.saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
            let leaf = shape.receipt_leaf(request.representation(), position)?;
            let constructor = abi
                .destinations()
                .get(&receipt.owner, request.representation())
                .ok_or_else(|| TransactionRefusal::DestinationOwnerHasNoConstructor {
                    owner: receipt.owner.clone(),
                })?;
            let script = constructor
                .instance()
                .tree()
                .leaf_programs()
                .get(&leaf)
                .cloned()
                .ok_or(TransactionRefusal::MissingLiveLeaf(leaf))?;
            let control = constructor.instance().control_block(leaf)?;

            Ok(ReceiptInputRecord::new(
                position,
                receipt.outpoint,
                receipt.owner.clone(),
                leaf,
                script,
                control,
                receipt.asset,
                receipt.value,
                receipt.program.clone(),
            ))
        })
        .collect()
}

// -------------------------------------------------------------------------
// The private lane's own finalization (rule:guide-ctf-exec:per-output-retirement)
// -------------------------------------------------------------------------

/// What the request cannot say, stated by the caller instead.
///
/// # Why this is a parameter and not a field of the request
///
/// Three of these facts are opening material and one is an election, and
/// the request vocabulary can carry neither.
///
/// The opening material — which registered fixture each input and each
/// destination belongs to, and the explicit amount behind a spent
/// commitment — is exactly what a request is forbidden to carry: a
/// request travels on a wire whose canonical form is a handle and a
/// digest, and a request that carried an amount would put a private
/// amount in a schema whose exclusions say it holds none
/// (rule:guide-ctf-exec:exclusions).
///
/// The election is the balancing designation. A transaction-wide
/// materializer has to be told which destination absorbs the blinder
/// residue, and the receipt-destination vocabulary has no way to say it,
/// because for an explicit transfer there is nothing to absorb. Electing
/// it here, out loud, is better than electing it by position and calling
/// the convention obvious.
///
/// # It is public disposable test material and nothing else
///
/// Every scalar reachable through this type is ADR-015 test material.
/// Nothing here is, becomes, or stands in for production custody, and
/// the record built downstream carries that as a non-claim rather than
/// as a sentence in this comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateLiveOpenings {
    inputs: Vec<PrivateInputOpening>,
    destinations: Vec<PrivateDestinationOpening>,
    non_protocol_region: NonProtocolFundingRegion,
    profiles: ConfidentialMaterializationProfiles,
}

/// One spent predecessor output's opening facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateInputOpening {
    /// Which registered fixture output this input is.
    pub opening: FixtureOpeningReference,
    /// The explicit amount behind the spent commitment.
    pub explicit_amount: u64,
    /// The asset blinder, which the guide's representation fixes at
    /// zero and which is carried rather than assumed so that a
    /// representation that stopped fixing it would be a changed value
    /// here and not a changed constant somewhere else.
    pub zero_asset_blinder: [u8; SCALAR_BYTES],
}

/// One destination's opening facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateDestinationOpening {
    /// Which registered fixture output this destination is.
    pub fixture: FixtureOpeningReference,
    /// Whether this destination is the primary one or the one that
    /// absorbs the blinder residue.
    pub role: ConfidentialOutputRole,
}

impl PrivateLiveOpenings {
    /// The openings for one private transfer.
    #[must_use]
    pub const fn new(
        inputs: Vec<PrivateInputOpening>,
        destinations: Vec<PrivateDestinationOpening>,
        non_protocol_region: NonProtocolFundingRegion,
        profiles: ConfidentialMaterializationProfiles,
    ) -> Self {
        Self {
            inputs,
            destinations,
            non_protocol_region,
            profiles,
        }
    }

    /// The per-input openings, in the request's receipt order.
    #[must_use]
    pub fn inputs(&self) -> &[PrivateInputOpening] {
        &self.inputs
    }

    /// The per-destination openings, in the request's destination order.
    #[must_use]
    pub fn destinations(&self) -> &[PrivateDestinationOpening] {
        &self.destinations
    }
}

/// A private transfer, finalized transaction-wide.
///
/// # Why this is not a `LiveFinalization`
///
/// Because a `LiveFinalization` carries a [`FinalizedLiveTransfer`],
/// whose protected bytes are the witnessless serialization, and the
/// proof-finalized candidate's protected bytes are not. Holding one
/// inside the other would leave a proof-free preimage authoritative,
/// which is the exact loss the materializer's own freeze exists to
/// prevent — so the private lane returns a different type rather than a
/// wrapped one.
///
/// What it does carry beside the candidate is the receipt records, and
/// those are the reason this function exists rather than a direct call
/// to the materializer: each record holds the leaf its own input
/// position executes and the control block that authenticates it, which
/// is what makes the spend a *live-receipt covenant* spend rather than a
/// spend of some program the caller chose.
#[derive(Clone, Debug)]
pub struct PrivateLiveFinalization {
    materialized: MaterializedConfidentialCandidate,
    receipts: Vec<ReceiptInputRecord>,
    report: LiveConstructionReport,
}

impl PrivateLiveFinalization {
    /// The proof-finalized candidate.
    #[must_use]
    pub const fn materialized(&self) -> &MaterializedConfidentialCandidate {
        &self.materialized
    }

    /// The receipt records, each carrying its own leaf and control
    /// block.
    #[must_use]
    pub fn receipts(&self) -> &[ReceiptInputRecord] {
        &self.receipts
    }

    /// The construction report.
    #[must_use]
    pub const fn report(&self) -> &LiveConstructionReport {
        &self.report
    }

    /// The candidate, taken out.
    #[must_use]
    pub fn into_materialized(self) -> MaterializedConfidentialCandidate {
        self.materialized
    }
}

/// The confidential destinations, each with the program its role
/// determines.
///
/// Split out of the finalization because it is the one place where a
/// destination's program is decided, and because a fee destination
/// decides it differently — see the comment inside.
///
/// # Errors
///
/// [`TransactionRefusal::DestinationOwnerHasNoConstructor`] where a
/// non-fee destination names an owner the ABI does not carry a
/// constructor for.
fn private_destination_intents(
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    openings: &PrivateLiveOpenings,
) -> Result<Vec<ConfidentialDestinationIntent>, TransactionRefusal> {
    let mut destinations = Vec::with_capacity(request.destinations().len());
    for (destination, opening) in request.destinations().iter().zip(openings.destinations()) {
        let program = if opening.role == ConfidentialOutputRole::Fee {
            Vec::new()
        } else {
            abi.destinations()
                .get(destination.owner(), request.representation())
                .ok_or_else(|| TransactionRefusal::DestinationOwnerHasNoConstructor {
                    owner: destination.owner().clone(),
                })?
                .instance()
                .program()
                .to_vec()
        };
        destinations.push(ConfidentialDestinationIntent::new(
            destination.value().amount(),
            abi.symbols().protocol_asset(),
            program,
            opening.fixture.clone(),
            opening.role,
        ));
    }
    Ok(destinations)
}

/// Finalize a private live transfer transaction-wide.
///
/// The private lane's entry point, and the one §8.9 names when it says
/// the private branch takes the transaction-wide path. The per-output
/// role is untouched and still serves the per-output path it always
/// served; what moves here is the private COMPLETE-TRANSACTION claim,
/// which now rests on a materializer that sees the whole transaction —
/// every opening, every destination, and the balance across them — and
/// therefore produces a candidate carrying real range proofs and a
/// blinder sum that closes.
///
/// The stages it shares with the explicit lane are shared rather than
/// copied: the shape is selected by the same function over the same
/// counts, the receipts are recognized against the same view, and the
/// receipt records come out of the same assembly. Only the outputs and
/// the protected transaction are the materializer's instead of this
/// module's, which is the whole of the difference and is why the two
/// lanes cannot drift about what a live receipt is.
///
/// # Errors
///
/// [`TransactionRefusal::PrivateFinalizationIsNotTheExplicitLane`] for a
/// request that is not private;
/// [`TransactionRefusal::PrivateFinalizationIsSponsorless`] for a
/// sponsored one; [`TransactionRefusal::RepresentationNotLinked`] where
/// the ABI carries no private plan;
/// [`TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest`] where the
/// openings and the request disagree about how many things there are;
/// [`TransactionRefusal::PrivateMaterializationRefused`] carrying the
/// materializer's own refusal; and any refusal of shape selection or of
/// receipt recognition, both of which are the explicit lane's own and
/// are called rather than reimplemented here.
pub fn finalize_private_live_transfer(
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    view: &PublicConstructionView,
    openings: &PrivateLiveOpenings,
    fixtures: &FrozenConfidentialFixtureView,
    crypto: &dyn ConfidentialProofMaterializer,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<PrivateLiveFinalization, TransactionRefusal> {
    if request.representation() != LiveTransferRepresentationPlan::PrivateCommitted {
        return Err(
            TransactionRefusal::PrivateFinalizationIsNotTheExplicitLane {
                representation: request.representation(),
            },
        );
    }
    if request.form() != RequestedForm::Sponsorless {
        return Err(TransactionRefusal::PrivateFinalizationIsSponsorless);
    }
    if !abi.representations().contains(&request.representation()) {
        return Err(TransactionRefusal::RepresentationNotLinked);
    }
    if openings.inputs().len() != request.receipts().len() {
        return Err(TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest {
            offered: openings.inputs().len(),
            required: request.receipts().len(),
        });
    }
    if openings.destinations().len() != request.destinations().len() {
        return Err(TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest {
            offered: openings.destinations().len(),
            required: request.destinations().len(),
        });
    }

    // The explicit lane's own stages, called and not reimplemented.
    let shape = select_shape(abi, request, 0)?;
    let recognized = recognize_receipts(abi, request, view)?;
    let created_total = request
        .destination_total()
        .ok_or(TransactionRefusal::DestinationTotalOutOfRange)?;

    // The inputs the materializer sees are the ones the node reported,
    // carried through recognition unchanged, plus the opening each one
    // needs and the per-output role cannot reach.
    let inputs: Vec<ConfidentialInputIntent> = recognized
        .iter()
        .zip(openings.inputs())
        .map(|(receipt, opening)| {
            ConfidentialInputIntent::new(
                receipt.outpoint,
                receipt.asset,
                receipt.value,
                receipt.program.clone(),
                LIVE_TRANSFER_SEQUENCE,
                opening.opening.clone(),
                opening.explicit_amount,
                opening.zero_asset_blinder,
            )
        })
        .collect();

    // Each destination pays to the private constructor the ABI
    // determines for that owner. Taking the program from anywhere else
    // is what would make the candidate a private transfer that is not a
    // live-receipt transfer.
    //
    // A FEE destination is the exception, and it is an exception with a
    // reason rather than a special case: a fee output's program is EMPTY
    // by the target's own definition of a fee, so there is no owner to
    // resolve and no constructor to look up. The role decides, which is
    // where every other role already comes from — the opening — and the
    // request's owner parameter at a fee position is not consulted at
    // all. The private request vocabulary is the explicit lane's and has
    // no fee destination member to state instead; that is a gap in the
    // request vocabulary and it is named here rather than papered over by
    // resolving an owner whose program would then be discarded.
    let destinations = private_destination_intents(abi, request, openings)?;

    let intent = ConfidentialConstructionIntent::new(
        inputs,
        destinations,
        openings.non_protocol_region.clone(),
        openings.profiles,
        shape.version().version(),
        abi.lock_time(),
    );

    let materialized = materialize_confidential_candidate(&intent, fixtures, crypto, checker)
        .map_err(|refusal| TransactionRefusal::PrivateMaterializationRefused(Box::new(refusal)))?;

    // Stage 10 of the explicit lane, unchanged: the leaf each position
    // executes and the control block that authenticates it.
    let receipts = receipt_records(abi, request, shape, &recognized)?;

    let owners = LiveOwnerCensus {
        distinct_semantic_owners: receipts
            .iter()
            .map(|record| record.owner().clone())
            .collect(),
        receipt_inputs: receipts.len(),
    };

    let report = LiveConstructionReport {
        shape: shape.shape(),
        representation: request.representation(),
        form: shape.form(),
        version: shape.version(),
        owners,
        destinations: request.destinations().len(),
        created_total,
        // Absent, and absent for a reason the explicit lane also
        // records: a public subtotal of private amounts is refused, and
        // the conservation this candidate satisfies is the target's
        // commitment balance rather than an arithmetic this module can
        // close.
        consumed_total: None,
        // The proof-bearing constructor, which is what retires
        // NoRangeProofIsProducedOrChecked — and retires it by scope,
        // for a lane where a proof is genuinely produced and checked,
        // rather than by edit.
        model: Some(SelectedConstructionModel::record_proof_bearing(
            ConfidentialConstructionModel::EXPECTED,
        )?),
    };

    Ok(PrivateLiveFinalization {
        materialized,
        receipts,
        report,
    })
}
