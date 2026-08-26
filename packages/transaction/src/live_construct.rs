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
use linker::live_backend::{
    LiveTransferComposition, LiveTransferRepresentationPlan, LiveTransferShape,
};
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
    ConfidentialInputRegion, ConfidentialMaterializationProfiles, ConfidentialOutputRole,
    ConfidentialProofMaterializer, FixtureOpeningReference, FrozenConfidentialFixtureView,
    IndependentCommitmentCheck, MaterializedConfidentialCandidate, NonProtocolFundingRegion,
    SCALAR_BYTES, materialize_confidential_candidate,
};
use crate::live_private::{
    ConfidentialConstructionModel, PrivateValueCapability, SelectedConstructionModel,
};
use crate::live_request::{LiveReceiptDestination, LiveTransferRequest, RequestedForm};
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
    finalize_live_transfer_declaring(target, abi, request, view, sponsor, private, &[])
}

/// The explicit finalization, with each destination's ROLE declared.
///
/// [`finalize_live_transfer`] is this function with every destination
/// declared a receipt output, which is what an empty slice means here.
/// The two are one pipeline rather than two, so a self-paying candidate
/// is not a parallel construction path that could come to disagree with
/// the one every other explicit shape uses.
///
/// The roles are the caller's declaration, not an inference. Reading the
/// fee position off the amounts, the programs, or the shape would make
/// the lane guess at a fact the caller already knows, and a guess that
/// is right on every shape built today is exactly the kind that stops
/// being right without looking different.
///
/// # Errors
///
/// Every refusal [`finalize_live_transfer`] answers, and additionally:
/// [`TransactionRefusal::DeclaredRolesDoNotCoverDestinations`] when the
/// declaration is neither empty nor one role per destination entry;
/// [`TransactionRefusal::SelfPaidFeeUnderSponsoredForm`] when a
/// sponsored request declares a fee entry, its fee being the sponsor's
/// to place from the offer; and
/// [`TransactionRefusal::SelfPaidFeeDeclaredMoreThanOnce`] for a second
/// declared fee entry, the target admitting one fee position.
pub fn finalize_live_transfer_declaring(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    view: &PublicConstructionView,
    sponsor: Option<&dyn SponsorCapability>,
    private: Option<&dyn PrivateValueCapability>,
    declared: &[ExplicitDestinationRole],
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

    // Stage 4: the shape, chosen by every count at once. A SPONSORED
    // form's fee is funded from the sponsor region and construction
    // places it from the offer, so it declares nothing here. A
    // sponsorless form that pays its OWN fee funds the fee out of the
    // receipts, and says which destination entry is the fee.
    let roles = admit_declared_roles(request, declared)?;
    let shape = select_shape(abi, request, sponsor_inputs.len(), roles)?;

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
        &OutputDeclarations {
            offer: offer.as_ref(),
            sponsor,
            private,
            declared,
        },
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

/// The explicit lane's declared roles, censused once and checked.
///
/// An EMPTY declaration is every entry a receipt output, which is what
/// every explicit caller but the self-paying one says, and it is
/// admitted without further question so that those callers are
/// unaffected by this stage existing at all.
///
/// # Errors
///
/// [`TransactionRefusal::DeclaredRolesDoNotCoverDestinations`] for a
/// non-empty declaration that is not one role per destination entry;
/// [`TransactionRefusal::SelfPaidFeeUnderSponsoredForm`] where a
/// sponsored request declares a fee entry; and
/// [`TransactionRefusal::SelfPaidFeeDeclaredMoreThanOnce`] for a second
/// declared fee entry.
fn admit_declared_roles(
    request: &LiveTransferRequest,
    declared: &[ExplicitDestinationRole],
) -> Result<DeclaredDestinationRoles, TransactionRefusal> {
    if declared.is_empty() {
        return Ok(DeclaredDestinationRoles::NONE);
    }
    if declared.len() != request.destinations().len() {
        return Err(TransactionRefusal::DeclaredRolesDoNotCoverDestinations {
            declared: declared.len(),
            destinations: request.destinations().len(),
        });
    }
    let census = DeclaredDestinationRoles::declared_explicitly(declared);
    if census.fee > 0 && request.form().sponsored() {
        return Err(TransactionRefusal::SelfPaidFeeUnderSponsoredForm);
    }
    if census.fee > 1 {
        return Err(TransactionRefusal::SelfPaidFeeDeclaredMoreThanOnce {
            declared: census.fee,
        });
    }
    Ok(census)
}

/// What one EXPLICIT destination entry is for.
///
/// The explicit counterpart of [`PrivateDestinationOpening::role`], and
/// deliberately a separate two-member vocabulary rather than a reuse of
/// [`ConfidentialOutputRole`]. That enum's members are distinctions the
/// confidential materializer draws — which output derives a blinder and
/// which absorbs the residue — and an explicit destination draws none of
/// them. Reusing it here would oblige every explicit caller to answer a
/// question about blinding that the explicit lane does not ask.
///
/// The role is stated BESIDE the destination rather than inside it.
/// §12.4 fixes a destination at an owner and a value, and §12.3's
/// may-not-select list names the target fee role, so a request cannot
/// carry this and does not: the declaration is a construction-time
/// argument, exactly as the private lane's openings are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExplicitDestinationRole {
    /// A live receipt output, paying a destination owner's constructor.
    ReceiptOutput,
    /// The transaction's own fee, funded out of the consumed receipts.
    ///
    /// Its entry's OWNER is never consulted — a fee output carries no
    /// program at all, which is most of what makes it a fee — and its
    /// entry's VALUE is the fee. The entry still names an owner because
    /// §12.4 gives a destination no way not to.
    Fee,
}

/// How many declared destination positions are NOT live receipt outputs.
///
/// A census of the request's own destinations by the role their openings
/// carry, taken once and handed to shape selection, rather than two
/// counts recomputed at each caller.
///
/// # Why the two roles are counted apart
///
/// Because they are discounted for the same reason and consulted for
/// different ones. Both a fee position and a sponsor-change position
/// occupy a declared destination slot that is not a live receipt output,
/// so both are subtracted before the receipt-output comparison. Only the
/// fee decides whether the candidate bears a fee. Collapsing them into
/// one total is arithmetically equal on every shape this lane builds
/// today and wrong on the first sponsorless request that declares a
/// change position, which is precisely the kind of agreement that stops
/// holding without anything looking different.
///
/// The explicit lane declares NEITHER when a SPONSOR funds the fee: it
/// appends its sponsor change and its fee itself, from the offer, at the
/// positions the shape names, and its request's destinations are receipt
/// outputs to a one. So it passes [`Self::NONE`] and the discount is a
/// fact stated where the private lane states every other one — the
/// opening's role.
///
/// The explicit lane that pays its OWN fee declares the fee, and
/// declares it the same way the private lane does: the fee is a
/// destination ENTRY and its role is stated beside the entry rather than
/// inside it, because §12.4 fixes a destination at an owner and a value
/// and §12.3 does not let a request select the target fee role. What the
/// explicit lane lacks is openings to carry the role, so it carries an
/// [`ExplicitDestinationRole`] per entry instead — the same declaration
/// with the confidential half removed, rather than a second way of
/// deciding that a destination is a fee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeclaredDestinationRoles {
    /// Declared positions whose role is a fee.
    fee: usize,
    /// Declared positions whose role is the sponsor's change.
    sponsor_change: usize,
}

impl DeclaredDestinationRoles {
    /// A request whose declared destinations are all receipt outputs.
    const NONE: Self = Self {
        fee: 0,
        sponsor_change: 0,
    };

    /// Census the openings' roles.
    fn census(openings: &[PrivateDestinationOpening]) -> Self {
        let mut roles = Self::NONE;
        for opening in openings {
            match opening.role {
                ConfidentialOutputRole::Fee => roles.fee += 1,
                ConfidentialOutputRole::SponsorChange => roles.sponsor_change += 1,
                // An explicit destination is a RECEIPT OUTPUT like the
                // other two, and is counted with them rather than
                // against them. What this census subtracts is positions
                // that are not receipts, and an exit crossing's
                // destinations all are.
                ConfidentialOutputRole::Primary
                | ConfidentialOutputRole::Balancing
                | ConfidentialOutputRole::ExplicitDestination => (),
            }
        }
        roles
    }

    /// Census the explicit lane's declared roles.
    ///
    /// No sponsor-change arm, and the absence is the scope rather than an
    /// omission: an explicit sponsor's change is placed from the OFFER at
    /// the position the shape names, never declared as a destination
    /// entry, so nothing on this lane can raise that count.
    fn declared_explicitly(roles: &[ExplicitDestinationRole]) -> Self {
        let mut census = Self::NONE;
        for role in roles {
            match role {
                ExplicitDestinationRole::Fee => census.fee += 1,
                ExplicitDestinationRole::ReceiptOutput => (),
            }
        }
        census
    }

    /// How many declared positions are discounted altogether.
    const fn declared(self) -> usize {
        self.fee + self.sponsor_change
    }
}

/// The one admitted shape realizing every requested count.
///
/// Six conjuncts, and the form is one of them rather than a consequence
/// of the others: §12.5 reports the form as its own term, and a shape
/// selected by counts alone could satisfy them under the other form.
///
/// # Why the fee is counted out of the destinations
///
/// The private request vocabulary has no fee destination member, so a
/// candidate that pays its own fee states the fee as a destination and
/// the *opening's role* is what says which position it is — the same
/// place [`private_destination_intents`] already reads it, and the same
/// reason. A shape's `receipt_outputs` counts live receipts, and a fee
/// output is emphatically not one, so the two are compared only after
/// the declared fee positions are discounted. `fee_destinations` is
/// therefore a statement the caller makes about its own request rather
/// than something inferred here from an empty program or a zero owner.
///
/// This is exactly the conjunct that refused the sponsorless fee-bearing
/// candidate at a real node: two destinations were matched against a
/// two-receipt-output shape, so the receipt covenant demanded a receipt
/// constructor program where the fee's empty program sat.
fn select_shape<'abi>(
    abi: &'abi CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    sponsor_inputs: usize,
    roles: DeclaredDestinationRoles,
) -> Result<&'abi LiveShapeAbi, TransactionRefusal> {
    let receipt_inputs = request.receipts().len();
    let declared = request.destinations().len();
    let sponsor_change = request.sponsor_change().requested();
    let wanted = if request.form().sponsored() {
        LiveTransactionForm::Sponsored
    } else {
        LiveTransactionForm::Sponsorless
    };
    // A sponsored form's fee is funded by the sponsor region and never
    // appears among the destinations, so the two sources of a fee output
    // are disjoint and adding them would be double counting.
    //
    // The fee count is read on its own and NOT off the combined
    // non-receipt total, because the two discounts answer two different
    // questions. A declared sponsor-change position is discounted from
    // the receipt-output comparison exactly as a fee is, and it says
    // nothing whatever about whether the candidate bears a fee — a
    // request declaring a change position and no fee would otherwise be
    // matched against a fee-bearing shape on the strength of the
    // change.
    let bears_a_fee = roles.fee > 0 || request.form().sponsored();

    let refusal = TransactionRefusal::UnsupportedLiveShape {
        receipt_inputs,
        destinations: declared,
        sponsor_inputs,
        sponsor_change,
    };
    // Refused rather than saturated: a request declaring more non-receipt
    // positions than it has destinations describes no shape at all, and
    // clamping it would go looking for one.
    let Some(destinations) = declared.checked_sub(roles.declared()) else {
        return Err(refusal);
    };

    abi.shapes()
        .values()
        .find(|candidate| {
            let shape = candidate.shape();
            usize::from(shape.receipt_inputs()) == receipt_inputs
                && usize::from(shape.receipt_outputs()) == destinations
                && usize::from(shape.sponsor_inputs()) == sponsor_inputs
                && candidate.sponsor_change_position().is_some() == sponsor_change
                && candidate.fee_position().is_some() == bears_a_fee
                && candidate.form() == wanted
        })
        .ok_or(refusal)
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
///
/// # Why the VALUE form is not checked here, and it is not an oversight
///
/// [`recognize_receipts`] gates a receipt's value field against the
/// requested representation and refuses a form the plan does not read.
/// This function has no counterpart, and the asymmetry was read as an
/// unexplained gap once. It is not one, and trying to close it is what
/// established that: adding the mirror clause immediately refused the
/// disclosure-minimality pair registry's own SPONSOR pair under the
/// private plan, whose private member deliberately carries an EXPLICIT
/// sponsor value.
///
/// The reason is the one §1.9 states. A representation plan is about
/// the PROTOCOL region — which receipts are consumed and which
/// destinations are created, the family whose amounts a protocol claim
/// is about. The sponsor region is deliberately outside every protocol
/// claim: it carries the reserve asset, it sits outside both balance
/// equations, and §10.7's isolation fragment introspects no value field
/// in it AT ALL, a property checked on the emitted instructions rather
/// than argued. So the plan has nothing to say about the sponsor's value
/// form, and a clause here would not be enforcing §6.3 — it would be
/// extending it over a region it was written to exclude.
///
/// What follows is that this function guards neither form, and both are
/// built without complaint here. A private transfer sponsored by an
/// explicit coin is a registered pair member and is accepted.
///
/// # A reading recorded here was overturned by running
///
/// This comment used to add that an explicit transfer's sponsor could
/// carry a commitment, and named that combination as what the
/// `private-sponsor-values` row is about. The first half stands and is
/// the paragraph above: nothing here guards the value form, and such a
/// candidate is CONSTRUCTED. The second half is refuted by a target. The
/// candidate was built, owner-signed, sponsor-signed and offered to the
/// pinned node, which refused it at consensus before script with its
/// balance check.
///
/// The refusal is arithmetic and not policy, which is why no guard here
/// could have been the difference. The target balances per asset, so the
/// reserve sub-equation is the sponsor input against the fee and the
/// change. A committed sponsor value carries a blinder, both outputs the
/// explicit lane writes for that region are explicit and carry none, and
/// nothing in the transaction absorbs the difference — so the sum cannot
/// close whatever the amounts are. A fee is mandatorily explicit, so the
/// only term that could absorb it is the sponsor's CHANGE, and the lane
/// that writes committed change is the private one.
///
/// So `private-sponsor-values` is about a PRIVATE transfer sponsored by
/// a committed coin, and it is
/// [`finalize_private_live_transfer`] that builds it.
///
/// The ASSET is different and is checked above, because the covenant
/// reads it and an introspection reads an explicit field.
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

/// What construction was handed beyond the request and the shape.
///
/// One bundle rather than four parallel arguments, because all four are
/// the caller's optional declarations about the same output census and
/// a reader meeting them one at a time cannot see that.
struct OutputDeclarations<'a> {
    /// The sponsor's offer, where a sponsor funds this transaction.
    offer: Option<&'a crate::sponsor::SponsorOffer>,
    /// The sponsor's own capability, which names its change destination.
    sponsor: Option<&'a dyn SponsorCapability>,
    /// The confidential capability, where values are commitments.
    private: Option<&'a dyn PrivateValueCapability>,
    /// The explicit lane's per-entry roles; empty means all receipts.
    declared: &'a [ExplicitDestinationRole],
}

/// The self-paid fee output one declared destination entry becomes.
///
/// Placed at the position the SHAPE names rather than at the entry's own
/// index, and the two are then required to agree. Placing it by index
/// would work on every shape whose fee entry is written last and
/// silently misplace it on the first one that is not.
///
/// # Errors
///
/// [`TransactionRefusal::SelfPaidFeeHasNoShapePosition`] where the
/// selected shape carries no fee position, and
/// [`TransactionRefusal::SelfPaidFeePositionDisagreesWithShape`] where
/// it names a different one.
const fn self_paid_fee_output(
    abi: &CandidateLiveTransferAbi,
    shape: &LiveShapeAbi,
    destination: &LiveReceiptDestination,
    position: u16,
) -> Result<(u16, TargetOutput), TransactionRefusal> {
    let Some(fee_position) = shape.fee_position() else {
        return Err(TransactionRefusal::SelfPaidFeeHasNoShapePosition);
    };
    if fee_position != position {
        return Err(TransactionRefusal::SelfPaidFeePositionDisagreesWithShape {
            declared: position,
            shaped: fee_position,
        });
    }
    Ok((
        fee_position,
        TargetOutput::new(
            // The PROTOCOL asset, because this fee is funded out of the
            // receipts and Elements balances per asset: a reserve-asset
            // fee beside no reserve-asset input dies at the tally. The
            // sponsored fee reads the reserve for the mirror-image
            // reason, its fee having been funded from the sponsor region.
            AssetField::Explicit(abi.symbols().protocol_asset()),
            ValueField::Explicit(destination.value().amount()),
            NonceField::Null,
            // The empty program: the fee role's whole identity is
            // target-structural, and this is the structure.
            Vec::new(),
        ),
    ))
}

/// The destinations, then the sponsor change, then the fee role.
fn assemble_outputs(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    shape: &LiveShapeAbi,
    request: &LiveTransferRequest,
    declarations: &OutputDeclarations<'_>,
) -> Result<Vec<TargetOutput>, TransactionRefusal> {
    let OutputDeclarations {
        offer,
        sponsor,
        private,
        declared,
    } = *declarations;
    let mut placed: BTreeMap<u16, TargetOutput> = BTreeMap::new();
    let (first, _) = shape.destination_range();

    for (index, destination) in request.destinations().iter().enumerate() {
        let position = first.saturating_add(u16::try_from(index).unwrap_or(u16::MAX));

        if declared.get(index) == Some(&ExplicitDestinationRole::Fee) {
            let (fee_position, output) = self_paid_fee_output(abi, shape, destination, position)?;
            placed.insert(fee_position, output);
            continue;
        }

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
///
/// One type for both regions, because a sponsor's coin needs exactly
/// what a receipt needs: it is an output of its own funding fixture, so
/// it opens against the registry the same way and its blinder joins the
/// same transaction-wide sum. What differs is which equations it is
/// inside, and that is the [`Self::region`] field rather than a second
/// vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateInputOpening {
    /// Which region this input belongs to.
    ///
    /// Declared by the caller and checked against the position, never
    /// inferred from the asset — the materializer's own reason applies
    /// unchanged here, that a region read off the asset would make the
    /// protocol balance depend on a comparison the balance is trying to
    /// decide.
    pub region: ConfidentialInputRegion,
    /// Which registered fixture output this input is.
    ///
    /// `None` for a consumed receipt whose VALUE is explicit, which has
    /// no opening to name: its amount is public and the blinder it
    /// brings to the transaction-wide sum is the all-zero one every
    /// explicit value is committed with. An entry crossing's receipts
    /// are exactly that, and a reference here would be naming a fixture
    /// output that does not exist.
    pub opening: Option<FixtureOpeningReference>,
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
    sponsor_inputs: Vec<Outpoint>,
    sponsor_spent: Vec<SpentSponsorOutput>,
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

    /// The sponsor's consumed coins, in the suffix order they occupy.
    ///
    /// Empty for a sponsorless candidate, which is the same thing the
    /// explicit lane's finalization says with the same emptiness.
    #[must_use]
    pub fn sponsor_inputs(&self) -> &[Outpoint] {
        &self.sponsor_inputs
    }

    /// What the view stated each sponsor coin held.
    ///
    /// Carried rather than re-fetched, because an owner's signature
    /// commits to EVERY spent output and this is the last place holding
    /// a view of the sponsor region. A caller rebuilding the owner
    /// message from the receipts alone would sign a different
    /// transaction than the one it is about to submit.
    #[must_use]
    pub fn sponsor_spent(&self) -> &[SpentSponsorOutput] {
        &self.sponsor_spent
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
/// # Which asset each position carries
///
/// Read off the role, and off the form for the one role where the form
/// decides. §10.7 isolates the sponsor and the fee roles in the RESERVE
/// asset, so a sponsored candidate's fee and its sponsor change both
/// carry the reserve — the sponsor's coin brought that asset in, and an
/// output funded from it in any other asset is a transaction that cannot
/// balance. A SPONSORLESS candidate's fee is funded from the receipts
/// instead, and therefore carries the PROTOCOL asset. That is the one
/// place the form and not the role decides, and it is why the fee's
/// asset is not a constant beside the role.
///
/// Everything else is the protocol asset, which is what the destinations
/// of a live transfer are denominated in.
///
/// Writing one asset over every position, as this did while the lane was
/// sponsorless, was correct for exactly as long as no position could
/// carry another one.
///
/// # Errors
///
/// [`TransactionRefusal::DestinationOwnerHasNoConstructor`] where a
/// receipt destination names an owner the ABI does not carry a
/// constructor for; and
/// [`TransactionRefusal::MalformedLiveDeploymentSymbol`] where the
/// deployment's sponsor-change symbol does not form a witness program.
fn private_destination_intents(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    composition: LiveTransferComposition,
    openings: &PrivateLiveOpenings,
) -> Result<Vec<ConfidentialDestinationIntent>, TransactionRefusal> {
    let mut destinations = Vec::with_capacity(request.destinations().len());
    for (destination, opening) in request.destinations().iter().zip(openings.destinations()) {
        let (asset, program) = match opening.role {
            // The empty program: the fee role's whole identity is
            // target-structural, and this is the structure. Its owner
            // parameter is not consulted at all — see the caller.
            ConfidentialOutputRole::Fee => {
                let asset = if request.form().sponsored() {
                    abi.symbols().reserve_asset()
                } else {
                    abi.symbols().protocol_asset()
                };
                (asset, Vec::new())
            }
            // The sponsor's change pays the deployment's sponsor-change
            // program, exactly as the explicit lane's does and from the
            // same symbol. Taking it from the destination's owner
            // instead would return the sponsor's reserve to a live
            // receipt constructor, which is a receipt nobody can spend
            // and a sponsor who is not repaid.
            ConfidentialOutputRole::SponsorChange => {
                let version = abi.symbols().sponsor_change_version();
                let payload = abi.symbols().sponsor_change_program();
                (
                    abi.symbols().reserve_asset(),
                    witness_program_script(target, version, payload)?,
                )
            }
            // Three roles, one answer, and the same one: every RECEIPT
            // destination carries the protocol asset at its owner's
            // constructor program, whatever its value form. The form is
            // the materializer's business and the program is not.
            ConfidentialOutputRole::Primary
            | ConfidentialOutputRole::Balancing
            | ConfidentialOutputRole::ExplicitDestination => (
                abi.symbols().protocol_asset(),
                // The CREATED side's plan, which is the whole of what a
                // crossing changes here. A destination is a coin this
                // transfer mints, and the constructor it must be paid
                // to is the one that will RECOGNIZE it when somebody
                // spends it next -- which is the created side's, not
                // the side this transfer's own receipts were read
                // under. While both sides were one plan the two
                // questions had one answer.
                abi.destinations()
                    .get(destination.owner(), composition.created())
                    .ok_or_else(|| TransactionRefusal::DestinationOwnerHasNoConstructor {
                        owner: destination.owner().clone(),
                    })?
                    .instance()
                    .program()
                    .to_vec(),
            ),
        };
        destinations.push(ConfidentialDestinationIntent::new(
            destination.value().amount(),
            asset,
            program,
            opening.fixture.clone(),
            opening.role,
        ));
    }
    Ok(destinations)
}

/// The sponsor's witnesses for a finalized PRIVATE candidate, by
/// position.
///
/// The private lane's counterpart to the sponsor branch inside
/// [`complete_live_transfer`], and it exists here rather than in a
/// ceremony for the reason [`SponsorSigningRequest`] is not publicly
/// constructible: a request names the exact bytes an authorization is
/// produced against, and a caller that could mint one could ask a
/// sponsor to authorize bytes that are not the candidate's. The request
/// is therefore built where the candidate is, from the candidate.
///
/// # Why the private lane needs its own and cannot call the explicit one
///
/// What a sponsor signs over. [`complete_live_transfer`] hands the
/// capability the finalized form's protected bytes, which are the
/// WITNESSLESS serialization. A proof-finalized candidate's protected
/// bytes are the frozen ones the materializer produced, and they are not
/// the same bytes — so a sponsor answering the explicit lane's request
/// would be authorizing a transaction that is not this one.
///
/// # Why it returns witnesses instead of a transaction
///
/// Because the owner authorizations for this lane are produced outside
/// this crate, against messages this crate computes. Returning the
/// sponsor's half by position lets the caller merge the two without
/// either half being able to displace the other, and without this
/// function needing owner signing material it has no business holding.
///
/// # Errors
///
/// [`TransactionRefusal::LiveSponsorRequestedWithoutCapability`] where
/// the candidate carries a sponsor input and no capability was offered;
/// [`TransactionRefusal::SponsorSignatureMissing`] where the sponsor
/// declined; and
/// [`TransactionRefusal::SponsorSignatureBindingMismatch`] where it
/// answered about other bytes.
pub fn private_sponsor_witnesses(
    finalization: &PrivateLiveFinalization,
    sponsor: Option<&dyn SponsorCapability>,
) -> Result<BTreeMap<u16, InputWitness>, TransactionRefusal> {
    let mut witnesses = BTreeMap::new();
    if finalization.sponsor_inputs().is_empty() {
        return Ok(witnesses);
    }
    let capability = sponsor.ok_or(TransactionRefusal::LiveSponsorRequestedWithoutCapability)?;
    let frozen = finalization.materialized().proof_finalized();
    let protected = frozen.protected_bytes();
    let outputs = frozen.protected().outputs().to_vec();

    // The sponsor suffix, at the positions the input order puts it: the
    // receipts occupy the run before it, and this lane wrote both runs
    // in that order.
    let first = finalization.receipts().len();
    for (offset, outpoint) in finalization.sponsor_inputs().iter().enumerate() {
        let position = u16::try_from(first + offset).unwrap_or(u16::MAX);
        let request = SponsorSigningRequest::new(
            protected.to_vec(),
            position,
            SignerRole::SponsorSuffixMember,
            SighashProfile::AllInputsAllOutputs,
            outputs.clone(),
        );
        let signature = capability
            .sign(&request)
            .ok_or(TransactionRefusal::SponsorSignatureMissing(*outpoint))?;
        if signature.bound_to() != protected {
            return Err(TransactionRefusal::SponsorSignatureBindingMismatch(
                *outpoint,
            ));
        }
        witnesses.insert(position, InputWitness::new(signature.stack().to_vec()));
    }
    Ok(witnesses)
}

/// Hold each input opening to the region its position holds.
///
/// Receipts first and the sponsor suffix after, which is the canonical
/// input order the explicit lane sorts into and the order the
/// materializer reads its regions in.
///
/// Checked rather than re-sorted, and rather than derived from the
/// position and the declaration discarded. Re-sorting would move an
/// opening onto a coin it does not open, and openings carry blinders, so
/// the failure would surface as a sum that does not close rather than as
/// the caller error it is.
///
/// # Errors
///
/// [`TransactionRefusal::PrivateOpeningRegionDisagreesWithPosition`] at
/// the first position whose opening claims the other region.
fn check_opening_regions(
    openings: &PrivateLiveOpenings,
    receipts: usize,
) -> Result<(), TransactionRefusal> {
    for (position, opening) in openings.inputs().iter().enumerate() {
        let expected = if position < receipts {
            ConfidentialInputRegion::Receipt
        } else {
            ConfidentialInputRegion::SponsorReserve
        };
        if opening.region != expected {
            return Err(
                TransactionRefusal::PrivateOpeningRegionDisagreesWithPosition {
                    position,
                    stated: opening.region,
                    expected,
                },
            );
        }
    }
    Ok(())
}

/// Hold the declared sponsor-region outputs to the sponsor's own offer.
///
/// The private lane declares its fee and its sponsor change as
/// destination positions, so their amounts arrive from the REQUEST,
/// while the sponsor states the same two numbers in its OFFER. Two
/// sources for one number is a disagreement waiting to happen, and the
/// explicit lane never had it: there the offer is the only source,
/// because construction places both outputs itself.
///
/// Only a SPONSORED request is checked, and that is the scope rather
/// than a shortcut. A sponsorless fee-bearing candidate funds its fee
/// out of the receipts and has no offer to disagree with, so its fee
/// position has exactly one source already.
///
/// # Errors
///
/// [`TransactionRefusal::PrivateSponsorRegionAmountDisagreesWithOffer`]
/// at the first role whose declared amount is not the offered one.
fn check_sponsor_region_amounts(
    request: &LiveTransferRequest,
    openings: &PrivateLiveOpenings,
    offer: Option<&crate::sponsor::SponsorOffer>,
) -> Result<(), TransactionRefusal> {
    let Some(offer) = offer else {
        return Ok(());
    };
    for (destination, opening) in request.destinations().iter().zip(openings.destinations()) {
        let offered = match opening.role {
            ConfidentialOutputRole::Fee => offer.fee(),
            // A change position with no offered change was already
            // refused by form exactness, so the absent case here is the
            // impossible one rather than a second policy.
            //
            // An offer stating its change as a COMMITMENT is skipped
            // rather than refused, and the skip is the honest answer
            // instead of a comparison invented to have one: there is no
            // number in a commitment to compare a declared amount with.
            // Nothing is lost by passing here, because the materializer
            // closes the balance over both outputs either way; what this
            // check adds is attribution for the case where two plain
            // numbers disagree.
            ConfidentialOutputRole::SponsorChange => match offer.change() {
                Some(ValueField::Explicit(change)) => change,
                Some(_) => continue,
                None => return Err(TransactionRefusal::SponsorChangeRequestedWithoutDestination),
            },
            ConfidentialOutputRole::Primary
            | ConfidentialOutputRole::Balancing
            | ConfidentialOutputRole::ExplicitDestination => continue,
        };
        let declared = destination.value().amount();
        if declared != offered {
            return Err(
                TransactionRefusal::PrivateSponsorRegionAmountDisagreesWithOffer {
                    role: opening.role,
                    declared,
                    offered,
                },
            );
        }
    }
    Ok(())
}

/// Every consumed input the materializer sees, in canonical order.
///
/// The three fields of each spent output are the ones the node reported,
/// carried through recognition unchanged; the opening beside them is the
/// one the per-output role cannot reach. The receipts come from receipt
/// recognition and the sponsor's coins from sponsor recognition — two
/// functions because they answer two different questions about a coin,
/// and one order because the transaction has one.
///
/// Infallible: every disagreement this could meet was refused before it
/// was called, which is why the regions are checked up there rather than
/// here.
fn private_input_intents(
    recognized: &[RecognizedReceipt],
    sponsor_inputs: &[Outpoint],
    sponsor_spent: &[SpentSponsorOutput],
    openings: &PrivateLiveOpenings,
) -> Vec<ConfidentialInputIntent> {
    let receipts = recognized.iter().map(|receipt| {
        (
            receipt.outpoint,
            receipt.asset,
            receipt.value,
            receipt.program.as_slice(),
        )
    });
    let sponsors = sponsor_inputs
        .iter()
        .zip(sponsor_spent)
        .map(|(outpoint, spent)| (*outpoint, spent.asset(), spent.value(), spent.program()));
    receipts
        .chain(sponsors)
        .zip(openings.inputs())
        .map(|((outpoint, asset, value, program), opening)| {
            // The region the opening declares, already checked against
            // this position. Two constructors rather than a parameter,
            // on the materializer's own ground: an input becomes a
            // sponsor's only where somebody meant it to be one.
            let build = match opening.region {
                ConfidentialInputRegion::Receipt => ConfidentialInputIntent::new,
                ConfidentialInputRegion::SponsorReserve => ConfidentialInputIntent::sponsor,
            };
            // The opening-free constructors, chosen by the SAME declared
            // region the opened ones are. Four constructors rather than
            // a flag, on the materializer's own ground: an input becomes
            // a sponsor's only where somebody meant it to be one, and it
            // becomes opening-free only where somebody meant that too. A
            // coin that fell into the wrong one would be a wrong
            // transaction rather than a refused one.
            //
            // This branch used to build a RECEIPT whatever the region
            // said, which discarded the declaration on exactly the
            // inputs that had no opening to fall back on. A sponsor coin
            // with an explicit value then joined the protocol subtotal
            // that §1.9 holds it outside of, and the candidate was
            // refused for a semantic imbalance of exactly the sponsor's
            // own amount.
            let build_bare = match opening.region {
                ConfidentialInputRegion::Receipt => ConfidentialInputIntent::explicit_receipt,
                ConfidentialInputRegion::SponsorReserve => {
                    ConfidentialInputIntent::explicit_sponsor
                }
            };
            opening.opening.clone().map_or_else(
                || {
                    build_bare(
                        outpoint,
                        asset,
                        value,
                        program.to_vec(),
                        LIVE_TRANSFER_SEQUENCE,
                        opening.explicit_amount,
                    )
                },
                |reference| {
                    build(
                        outpoint,
                        asset,
                        value,
                        program.to_vec(),
                        LIVE_TRANSFER_SEQUENCE,
                        reference,
                        opening.explicit_amount,
                        opening.zero_asset_blinder,
                    )
                },
            )
        })
        .collect()
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
/// counts, the receipts are recognized against the same view, the
/// sponsor's coins are recognized against the same view by the same
/// function, the form and the offer are held to §12.5's equivalence by
/// the same check, and the receipt records come out of the same
/// assembly. Only the outputs and the protected transaction are the
/// materializer's instead of this module's, which is the whole of the
/// difference and is why the two lanes cannot drift about what a live
/// receipt is.
///
/// # Why this lane is the one that can be sponsored by a blinded coin
///
/// Arithmetic, established by a target and not by preference. The target
/// balances per asset, so a sponsored transaction's reserve sub-equation
/// is the sponsor input against the fee and the change. A committed
/// sponsor value carries a blinder; a fee is mandatorily explicit and
/// carries none; so the only term that can absorb that blinder is the
/// sponsor's CHANGE, and a candidate spending a committed sponsor coin
/// must return COMMITTED change. The explicit lane writes explicit
/// outputs by construction and therefore cannot, which a node said
/// plainly by refusing such a candidate at consensus before script. A
/// committed change output is the materializer's shape, so it is this
/// lane's.
///
/// The blinded sponsor coin is admitted and NOT required. §1.9 holds the
/// sponsor region outside every protocol claim, so the representation
/// plan has nothing to say about the sponsor's value form and a private
/// transfer sponsored by an EXPLICIT coin stays exactly as buildable as
/// it was — see [`recognize_sponsors`], which declines to mirror the
/// receipt clause for that reason.
///
/// # Errors
///
/// [`TransactionRefusal::PrivateFinalizationIsNotTheExplicitLane`] for a
/// request that is not private;
/// [`TransactionRefusal::RepresentationNotLinked`] where the ABI carries
/// no private plan;
/// [`TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest`] where the
/// openings and the request disagree about how many things there are;
/// [`TransactionRefusal::PrivateOpeningRegionDisagreesWithPosition`]
/// where an opening claims a region its position does not hold;
/// [`TransactionRefusal::PrivateMaterializationRefused`] carrying the
/// materializer's own refusal; and any refusal of form exactness, of
/// shape selection, or of receipt and sponsor recognition, all of which
/// are the explicit lane's own and are called rather than reimplemented
/// here.
// Nine parameters, and the count is the lane rather than an accretion.
// Six are the ones this entry point always took; the reviewed target and
// the sponsor capability are what admitting a sponsored request needs,
// because the sponsor change's program is built from a deployment symbol
// through the target's own push forms and because §12.5's equivalence
// cannot be checked against a capability nobody passed. Grouping them
// into a parameter struct would name a thing that is not one.
#[allow(clippy::too_many_arguments)]
pub fn finalize_private_live_transfer(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    view: &PublicConstructionView,
    sponsor: Option<&dyn SponsorCapability>,
    openings: &PrivateLiveOpenings,
    fixtures: &FrozenConfidentialFixtureView,
    crypto: &dyn ConfidentialProofMaterializer,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<PrivateLiveFinalization, TransactionRefusal> {
    finalize_private_live_transfer_composing(
        target,
        abi,
        request,
        LiveTransferComposition::HomogeneousPrivate,
        view,
        sponsor,
        openings,
        fixtures,
        crypto,
        checker,
    )
}

/// One private-lane candidate under a stated COMPOSITION.
///
/// The general form, of which [`finalize_private_live_transfer`] is the
/// wholly private case. The two are one pipeline rather than two, so a
/// crossing candidate is not a parallel construction path that could
/// drift from the one every homogeneous private shape uses.
///
/// # Why crossing is this lane's and not the explicit lane's
///
/// Because a lane is chosen by what it must BUILD, not by what it must
/// read. Every composition but the wholly explicit one either consumes a
/// commitment, whose blinder joins a sum somebody must close, or creates
/// one, which needs a blinder, a range proof and a commitment nobody but
/// this lane derives. The explicit lane builds none of those and would
/// have to grow all of them to serve a crossing; this lane already has
/// them and needs only to be told which side is which.
///
/// The request's own representation stays the CONSUMED side's plan, and
/// that is not a convention either: `recognize_receipts` reads it to
/// decide the value form a spent receipt must carry, which is a question
/// about the side being consumed. A request naming the other side would
/// be refusing its own inputs.
///
/// # Errors
///
/// As [`finalize_private_live_transfer`], and
/// [`TransactionRefusal::PrivateFinalizationIsNotTheExplicitLane`] for a
/// wholly explicit composition or for a request whose representation is
/// not the composition's consumed side.
#[allow(clippy::too_many_arguments)]
pub fn finalize_private_live_transfer_composing(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    composition: LiveTransferComposition,
    view: &PublicConstructionView,
    sponsor: Option<&dyn SponsorCapability>,
    openings: &PrivateLiveOpenings,
    fixtures: &FrozenConfidentialFixtureView,
    crypto: &dyn ConfidentialProofMaterializer,
    checker: &dyn IndependentCommitmentCheck,
) -> Result<PrivateLiveFinalization, TransactionRefusal> {
    // A wholly explicit transfer builds no commitment and belongs to the
    // other lane. Every other composition has at least one blinded field
    // and belongs here.
    if composition == LiveTransferComposition::HomogeneousExplicit {
        return Err(
            TransactionRefusal::PrivateFinalizationIsNotTheExplicitLane {
                representation: composition.created(),
            },
        );
    }
    // The request names the side its own receipts are read under, and a
    // request naming the other side would be refusing its own inputs.
    if request.representation() != composition.consumed() {
        return Err(
            TransactionRefusal::PrivateFinalizationIsNotTheExplicitLane {
                representation: request.representation(),
            },
        );
    }
    // BOTH sides must be linked into this deployment. The consumed side
    // is what recognizes the receipts and the created side is what the
    // destinations are paid to, so a deployment holding only one of them
    // could build a candidate whose outputs nobody can spend.
    for side in [composition.consumed(), composition.created()] {
        if !abi.representations().contains(&side) {
            return Err(TransactionRefusal::RepresentationNotLinked);
        }
    }

    // §12.5's equivalence, both directions, before anything is built
    // from either side of it — the explicit lane's stage 1, called here
    // for the same reason it is called there. This is what refuses a
    // sponsored request with no capability and a capability with no
    // sponsored request, and it is why this lane no longer needs a
    // refusal of its own for the form.
    let offer = check_form_exactness(request, sponsor)?;
    let sponsor_inputs: Vec<Outpoint> = offer
        .as_ref()
        .map(|offer| offer.inputs().iter().copied().collect())
        .unwrap_or_default();
    let sponsor_spent = recognize_sponsors(abi, request, view, &sponsor_inputs)?;

    // The openings cover the WHOLE input order and not the receipts
    // alone. A sponsor's coin is a spent predecessor output like any
    // other and needs the same opening: its blinder is one addend of the
    // transaction-wide input sum, and an input whose blinder the
    // materializer cannot reach is a sum that cannot be closed.
    let required_inputs = request.receipts().len() + sponsor_inputs.len();
    if openings.inputs().len() != required_inputs {
        return Err(TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest {
            offered: openings.inputs().len(),
            required: required_inputs,
        });
    }
    if openings.destinations().len() != request.destinations().len() {
        return Err(TransactionRefusal::PrivateOpeningsDoNotCoverTheRequest {
            offered: openings.destinations().len(),
            required: request.destinations().len(),
        });
    }

    check_opening_regions(openings, request.receipts().len())?;
    check_sponsor_region_amounts(request, openings, offer.as_ref())?;

    // The explicit lane's own stages, called and not reimplemented. The
    // non-receipt positions are counted off the openings' roles, which is
    // where this lane already decides that a destination is a fee: the
    // request vocabulary cannot say it, so the opening does, and the
    // count is read from the same field rather than a second one that
    // could disagree with it. The sponsor's change is counted from the
    // same field for the same reason.
    let roles = DeclaredDestinationRoles::census(openings.destinations());
    let shape = select_shape(abi, request, sponsor_inputs.len(), roles)?;
    let recognized = recognize_receipts(abi, request, view)?;
    let created_total = request
        .destination_total()
        .ok_or(TransactionRefusal::DestinationTotalOutOfRange)?;

    let inputs = private_input_intents(&recognized, &sponsor_inputs, &sponsor_spent, openings);

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
    //
    // The SPONSOR CHANGE is the second exception and has the same shape
    // of reason: its program is the deployment's sponsor-change symbol
    // rather than any owner's constructor, because the output repays the
    // sponsor and not a live receipt holder.
    let destinations = private_destination_intents(target, abi, request, composition, openings)?;

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
        sponsor_inputs,
        sponsor_spent,
        report,
    })
}
