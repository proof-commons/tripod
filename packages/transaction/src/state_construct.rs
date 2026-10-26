//! Construction of the maturity announcement (§12.5), and the first of
//! §12.7's five states.
//!
//! # A construction is built, and it is not finalized
//!
//! [`MaturityConstruction`] is a candidate transaction that exists:
//! version, inputs, outputs, a lock time, and a null witness per
//! input. Nothing in it is frozen, no protected-region census has been
//! taken over it, and no witness role is populated — the announcement's
//! seven roles are declared by the ABI together with the stage that owes
//! each item, and this stage owes none of them. What it does own is the
//! successor: the semantic metadata the transition derived, the
//! constructor the bundle's search settled on, and the output program
//! that constructor commits to.
//!
//! # The nine steps, and where each of them lives
//!
//! §12.5 lists nine steps and [`construct_maturity_announcement`] runs
//! the first seven, in order, with a stage comment per step:
//!
//! 1. the semantic request is validated here;
//! 2. the successor semantic metadata is derived by the realization's
//!    one maturity transition, over the deployment's own lead window;
//! 3. the canonical representation nonce is searched from zero, and
//! 4. the successor constructor derived, and
//! 5. the result compared with the linked constructor policy — which is
//!    one call to the linked bundle, for the reason given at that stage;
//! 6. output zero is fixed here;
//! 7. optional sponsorship is fixed from the construction context.
//!
//! Steps 8 and 9 — freezing the protected bytes and creating the
//! operator signing request — belong to the next state,
//! `FinalizedMaturityAnnouncement`, and are not performed here. A
//! construction is what that state finalizes, so performing either step
//! here would put the finalization boundary in two places and leave a
//! reader unable to say which side of it a given value was fixed on.
//!
//! # The prohibition is structural
//!
//! §12.5 says a caller cannot provide a nonce result or an arbitrary
//! output program. That is not enforced by a check, because a check
//! implies a field: there is no parameter, no constructor argument and
//! no setter through which either could arrive, and the type's own
//! documentation carries the compile failures that say so. The nonce is
//! the search's, and the program is the constructor's over the searched
//! nonce — which is also why the two travel together inside one
//! constructor here rather than as two values that could be combined
//! from different derivations.
//!
//! # The sponsored arm
//!
//! The offer fixes the suffix, fee and optional change. Sponsor spent
//! outputs remain available to the complete signing census. Region membership
//! belongs to the realization; the STATE leaf gains no sponsor checks.

use linker::{LinkedArtifactStatus, StateLinkSymbol, StateSymbolValue};
use realization::{StateMetadata, announce_maturity};
use tapscript::{CandidateStateConstructor, StateCurveCapability};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, ValueField,
};
use crate::error::TransactionRefusal;
use crate::sponsor::{SponsorCapability, SponsorOffer};
use crate::state_abi::CandidateMaturityAnnouncementAbi;
use crate::state_request::MaturityAnnouncementRequest;
use crate::state_view::ValidatedMaturityStateView;
use crate::taproot::witness_program_script;
use crate::view::{PublicConstructionView, PublicOutputView};

/// Construction facts for the sponsor suffix, separate from the semantic request.
pub struct MaturitySponsorContext<'a> {
    /// The declared reserve asset, independent of the offered coins.
    pub reserve_asset: AssetId,
    /// Public spent-output fields for the offered coins.
    pub view: &'a PublicConstructionView,
    /// The capability supplies funding, change destination and per-input answers.
    pub capability: &'a dyn SponsorCapability,
}

/// The lock time every maturity announcement is built at.
///
/// Zero, the value that imposes no height and no time constraint. No
/// announcement leaf reads the lock-time field, and the ABI's sequence
/// constraint puts the final sequence on every input, which leaves the
/// field inert in the target's own evaluation; a nonzero value would
/// therefore be a constraint nobody asked for, carried by bytes the
/// operator signs.
pub const MATURITY_ANNOUNCEMENT_LOCK_TIME: u32 = 0;

/// One constructed maturity announcement (§12.7's first state).
///
/// A caller cannot name the successor's representation nonce:
///
/// ```compile_fail,E0599
/// use transaction::MaturityConstruction;
/// fn choose_nonce(construction: &MaturityConstruction) {
///     construction.with_nonce(0);
/// }
/// ```
///
/// and cannot name its output program either:
///
/// ```compile_fail,E0599
/// use transaction::MaturityConstruction;
/// fn choose_program(construction: &MaturityConstruction) {
///     construction.with_output_program(vec![0x51, 0x20]);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityConstruction {
    transaction: TargetTransaction,
    successor_constructor: CandidateStateConstructor,
    successor_metadata: StateMetadata,
    request: MaturityAnnouncementRequest,
    validated: ValidatedMaturityStateView,
    abi: CandidateMaturityAnnouncementAbi,
    sponsor_spent: Vec<PublicOutputView>,
    sponsor_change_position: Option<u16>,
    fee_position: Option<u16>,
}

impl MaturityConstruction {
    /// Sponsor spent outputs in canonical suffix order.
    #[must_use]
    pub fn sponsor_spent(&self) -> &[PublicOutputView] {
        &self.sponsor_spent
    }

    /// The actual sponsor-change position after zero change is omitted.
    #[must_use]
    pub const fn sponsor_change_position(&self) -> Option<u16> {
        self.sponsor_change_position
    }

    /// The actual fee position.
    #[must_use]
    pub const fn fee_position(&self) -> Option<u16> {
        self.fee_position
    }
    /// The typed candidate transaction.
    #[must_use]
    pub const fn transaction(&self) -> &TargetTransaction {
        &self.transaction
    }

    /// The exact target bytes.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.transaction.encode()
    }

    /// The successor constructor the search settled on.
    ///
    /// The nonce, the parity, the output key, the output program and the
    /// search's own leastness evidence — with the residual that evidence
    /// states — are all read through it rather than copied out beside
    /// it, so no second statement of any of them exists to drift from
    /// the constructor that produced them.
    #[must_use]
    pub const fn successor_constructor(&self) -> &CandidateStateConstructor {
        &self.successor_constructor
    }

    /// The successor semantic metadata the transition derived.
    #[must_use]
    pub const fn successor_metadata(&self) -> StateMetadata {
        self.successor_metadata
    }

    /// The typed request this was built for.
    #[must_use]
    pub const fn request(&self) -> MaturityAnnouncementRequest {
        self.request
    }

    /// The validated public current-STATE view this was built over.
    ///
    /// The accepted linked bundle, the deployment's resolved symbols and
    /// the view's own residual are reached through it, which is what
    /// keeps the construction's account of the predecessor identical to
    /// the account the checks ran against.
    #[must_use]
    pub const fn validated_view(&self) -> &ValidatedMaturityStateView {
        &self.validated
    }

    /// The candidate ABI this was built against.
    #[must_use]
    pub const fn abi(&self) -> &CandidateMaturityAnnouncementAbi {
        &self.abi
    }
}

/// Construct the maturity announcement (§12.5).
///
/// # Errors
///
/// [`TransactionRefusal::SponsorRequestedWithoutCapability`] for a
/// sponsored request without its construction context,
/// [`TransactionRefusal::BundleIsNotACandidate`] for a bundle claiming
/// more than a prototype link,
/// [`TransactionRefusal::ContractRevisionMismatch`] when the ABI and the
/// target disagree about the reviewed revision,
/// [`TransactionRefusal::MaturityDeploymentAssetDisagreesWithView`] and
/// [`TransactionRefusal::MaturityDeploymentAmountDisagreesWithView`]
/// when the view's stated asset or amount is not the deployment's own,
/// [`TransactionRefusal::MalformedAssetIdentifier`] when the defined
/// asset is not an identifier width,
/// [`TransactionRefusal::MaturitySuccessorTransitionRefused`] for a
/// refused semantic transition,
/// [`TransactionRefusal::MaturitySuccessorSearchRefused`] for a refused
/// constructor application,
/// [`TransactionRefusal::MaturitySuccessorContinuityRefused`] when the
/// predecessor and the successor do not share their fixed construction
/// parameters, and whatever
/// [`TargetTransaction::new`] refuses about the assembled roles.
///
/// # Panics
///
/// Panics only if the accepted linked bundle retains no constructor
/// application, which a link cannot arrange: the link retains the
/// application it ran over its own sources, so a bundle without one
/// would be a bundle that broke its own construction contract before it
/// existed.
pub fn construct_maturity_announcement(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateMaturityAnnouncementAbi,
    validated: &ValidatedMaturityStateView,
    request: &MaturityAnnouncementRequest,
    curve: &impl StateCurveCapability,
) -> Result<MaturityConstruction, TransactionRefusal> {
    construct_maturity_announcement_with_sponsor(target, abi, validated, request, curve, None)
}

/// Construct the selected form using explicit sponsor construction facts.
///
/// # Errors
/// Returns the successor-construction refusals, or
/// [`TransactionRefusal::SponsorRequestedWithoutCapability`],
/// [`TransactionRefusal::SponsorCapabilityWithoutRequest`],
/// [`TransactionRefusal::EmptySponsorOffer`],
/// [`TransactionRefusal::SponsorChangeRequestedWithoutDestination`],
/// [`TransactionRefusal::SponsorChangeOfferedWithoutRequest`],
/// [`TransactionRefusal::OverlappingOutpoint`],
/// [`TransactionRefusal::MissingPublicSponsorView`],
/// [`TransactionRefusal::SponsorInputCarriesForeignAsset`],
/// [`TransactionRefusal::MaturitySponsorValueRequiresProofFinalization`],
/// [`TransactionRefusal::MaturitySponsorReserveAliasesStateAsset`] or
/// [`TransactionRefusal::MaturitySponsorInputPositionOutOfRange`]
/// before a signing request exists. The change destination can also return
/// [`TransactionRefusal::MalformedPinnedProgram`] or
/// [`TransactionRefusal::MalformedDeploymentSymbol`].
///
/// # Panics
/// Panics only if a linked bundle lost its retained constructor, which a link cannot arrange.
pub fn construct_maturity_announcement_with_sponsor(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateMaturityAnnouncementAbi,
    validated: &ValidatedMaturityStateView,
    request: &MaturityAnnouncementRequest,
    curve: &impl StateCurveCapability,
    sponsor: Option<&MaturitySponsorContext<'_>>,
) -> Result<MaturityConstruction, TransactionRefusal> {
    // Step 1: hold the form and offer together before constructing outputs.
    let offer = sponsor_offer(request, sponsor)?;
    let bundle = validated.view().accepted_linked_bundle();
    if bundle.status() != LinkedArtifactStatus::Prototype {
        return Err(TransactionRefusal::BundleIsNotACandidate);
    }
    if abi.contract() != target.definition().version() {
        return Err(TransactionRefusal::ContractRevisionMismatch);
    }
    let (asset, value) = successor_output_fields(target, validated)?;

    // Step 2: derive the successor semantic metadata. The transition has
    // one owner in the realization, and its refusal travels whole: an
    // already-announced predecessor, a complete one, a cycle below the
    // window, a cycle above it and a window that could not be derived
    // are five findings, and only the layer that derives the window has
    // the words for all five.
    let predecessor = validated.view().predecessor_metadata();
    let successor_metadata = announce_maturity(
        &predecessor,
        request.announced_cycle(),
        bundle.deployment().lead_bounds().bounds(),
    )
    .map_err(|refusal| TransactionRefusal::MaturitySuccessorTransitionRefused { refusal })?;

    // Steps 3, 4 and 5: search the nonce from zero, derive the successor
    // constructor, and compare the result with the linked constructor
    // policy. One call, because the recipe the bundle applies is that
    // policy: it searches under the policy's own budget, over the
    // policy's static subtree and internal key, so the comparison step
    // asks a question the application has already answered by
    // construction. A second scan here would give the leastness policy
    // and its measured budget a second place to drift. The refusal is
    // carried whole rather than flattened, so that exhaustion stays
    // distinguishable from the two faults that are not exhaustion.
    let successor_constructor = bundle
        .apply_constructor(target, &successor_metadata, curve)
        .map_err(
            |refusal| TransactionRefusal::MaturitySuccessorSearchRefused {
                refusal: Box::new(refusal),
            },
        )?;

    // What the comparison leaves is the continuity equality, held
    // between the application the link itself retained and the one just
    // derived. The retained predecessor is the left-hand side and the
    // successor the right, though the equality is symmetric in content —
    // three equalities over the static root, the internal key and the
    // leaf version — so nothing should be read into the argument order.
    let retained = bundle
        .instances()
        .first()
        .expect("a linked bundle retains the constructor application its own link ran")
        .constructor();
    hold_continuity(retained, &successor_constructor)?;

    // Step 6: fix output zero. The asset and the amount are the
    // deployment's own definitions rather than the caller's statement of
    // them, the program is the successor constructor's, and the nonce
    // field is the null one an unblinded explicit output carries. The
    // input's outpoint is the view's and its sequence is the ABI's; the
    // version is the ABI's too. The ABI puts the coordinator at input
    // zero and the successor at output zero; sponsor roles are appended
    // afterwards without moving either protected protocol position.
    let input = TargetInput::new(
        validated.view().current_state_outpoint(),
        abi.sequence().sequence(),
    );
    let output = TargetOutput::new(
        asset,
        value,
        NonceField::Null,
        successor_constructor.output_program(),
    );

    // Step 7: append the canonical sponsor suffix and its offered output roles.
    // Conservation and target authorization remain independent obligations.
    let mut inputs = vec![input];
    let mut outputs = vec![output];
    let mut sponsor_spent = Vec::new();
    let mut sponsor_change_position = None;
    let mut fee_position = None;
    if let (Some(context), Some(offer)) = (sponsor, offer) {
        sponsor_spent = sponsor_spent_outputs(
            context,
            &offer,
            validated.view().current_state_outpoint(),
            abi.inputs().sponsor_suffix().first(),
            asset,
        )?;
        inputs.extend(
            sponsor_spent
                .iter()
                .map(|spent| TargetInput::new(spent.outpoint(), abi.sequence().sequence())),
        );
        let next_output = abi.outputs().successor_position() + 1;
        if let Some(change) = sponsor_change_output(target, context, &offer)? {
            sponsor_change_position = Some(next_output);
            outputs.push(change);
        }
        fee_position = Some(next_output + u16::from(sponsor_change_position.is_some()));
        outputs.push(TargetOutput::new(
            AssetField::Explicit(context.reserve_asset),
            ValueField::Explicit(offer.fee()),
            NonceField::Null,
            Vec::new(),
        ));
    }
    let witnesses = vec![InputWitness::default(); inputs.len()];
    let transaction = TargetTransaction::new(
        abi.version().version(),
        inputs,
        outputs,
        MATURITY_ANNOUNCEMENT_LOCK_TIME,
        witnesses,
    )?;

    Ok(MaturityConstruction {
        transaction,
        successor_constructor,
        successor_metadata,
        request: *request,
        validated: validated.clone(),
        abi: abi.clone(),
        sponsor_spent,
        sponsor_change_position,
        fee_position,
    })
}

fn sponsor_spent_outputs(
    context: &MaturitySponsorContext<'_>,
    offer: &SponsorOffer,
    coordinator: Outpoint,
    first: u16,
    state_asset: AssetField,
) -> Result<Vec<PublicOutputView>, TransactionRefusal> {
    if AssetField::Explicit(context.reserve_asset) == state_asset {
        return Err(TransactionRefusal::MaturitySponsorReserveAliasesStateAsset);
    }
    if offer.inputs().len() > usize::from(u16::MAX) - usize::from(first) + 1 {
        return Err(TransactionRefusal::MaturitySponsorInputPositionOutOfRange);
    }
    offer
        .inputs()
        .iter()
        .map(|outpoint| {
            if *outpoint == coordinator {
                return Err(TransactionRefusal::OverlappingOutpoint(*outpoint));
            }
            let spent = context
                .view
                .get(*outpoint)
                .ok_or(TransactionRefusal::MissingPublicSponsorView(*outpoint))?;
            if spent.asset() != AssetField::Explicit(context.reserve_asset) {
                return Err(TransactionRefusal::SponsorInputCarriesForeignAsset(
                    *outpoint,
                ));
            }
            if !matches!(spent.value(), ValueField::Explicit(_)) {
                return Err(TransactionRefusal::MaturitySponsorValueRequiresProofFinalization);
            }
            Ok(spent.clone())
        })
        .collect()
}

fn sponsor_change_output(
    target: &ReviewedElementsTapscriptDefinition,
    context: &MaturitySponsorContext<'_>,
    offer: &SponsorOffer,
) -> Result<Option<TargetOutput>, TransactionRefusal> {
    let Some(value) = offer.change() else {
        return Ok(None);
    };
    let ValueField::Explicit(amount) = value else {
        return Err(TransactionRefusal::MaturitySponsorValueRequiresProofFinalization);
    };
    if amount == 0 {
        return Ok(None);
    }
    let (version, payload) = context
        .capability
        .change_destination()
        .ok_or(TransactionRefusal::SponsorChangeRequestedWithoutDestination)?;
    let program = witness_program_script(target, version, &payload)?;
    Ok(Some(TargetOutput::new(
        AssetField::Explicit(context.reserve_asset),
        value,
        NonceField::Null,
        program,
    )))
}

fn sponsor_offer(
    request: &MaturityAnnouncementRequest,
    sponsor: Option<&MaturitySponsorContext<'_>>,
) -> Result<Option<SponsorOffer>, TransactionRefusal> {
    match (request.requested_form().sponsored(), sponsor) {
        (true, None) => Err(TransactionRefusal::SponsorRequestedWithoutCapability),
        (false, Some(_)) => Err(TransactionRefusal::SponsorCapabilityWithoutRequest),
        (false, None) => Ok(None),
        (true, Some(context)) => {
            let offer = context.capability.offer();
            if offer.inputs().is_empty() {
                return Err(TransactionRefusal::EmptySponsorOffer);
            }
            match (request.sponsor_change().requested(), offer.change()) {
                (true, None) => Err(TransactionRefusal::SponsorChangeRequestedWithoutDestination),
                (false, Some(_)) => Err(TransactionRefusal::SponsorChangeOfferedWithoutRequest),
                _ => Ok(Some(offer)),
            }
        }
    }
}

/// The asset and amount output zero carries.
///
/// Read from the deployment's own resolved definitions and then held
/// against the view's statement of the same two facts, rather than taken
/// from the view: the leaf's structural patterns pin the singleton asset
/// and the declared issuance, so those definitions are what the
/// successor output has to carry for the spend to be the one the leaf
/// describes, and a caller's statement of them is a claim to be checked
/// against the deployment rather than a source to build from.
///
/// # Errors
///
/// [`TransactionRefusal::MaturityDeploymentAssetDisagreesWithView`] and
/// [`TransactionRefusal::MaturityDeploymentAmountDisagreesWithView`] as
/// documented on the variants, and
/// [`TransactionRefusal::MalformedAssetIdentifier`] for a defined asset
/// that is not an identifier width.
fn successor_output_fields(
    target: &ReviewedElementsTapscriptDefinition,
    validated: &ValidatedMaturityStateView,
) -> Result<(AssetField, ValueField), TransactionRefusal> {
    let entries = validated.view().deployment_symbols().entries();

    let defined = entries
        .get(&StateLinkSymbol::StateAsset)
        .map(|entry| entry.definition().value());
    let asset = match defined {
        Some(StateSymbolValue::Asset(item)) => {
            AssetField::Explicit(AssetId::from_slice(item.bytes())?)
        }
        _ => return Err(TransactionRefusal::MaturityDeploymentAssetDisagreesWithView),
    };
    if asset != validated.view().asset() {
        return Err(TransactionRefusal::MaturityDeploymentAssetDisagreesWithView);
    }

    let defined = entries
        .get(&StateLinkSymbol::StateAmount)
        .map(|entry| entry.definition().value());
    let value = match defined {
        Some(StateSymbolValue::ExplicitAmount(item)) => item
            .signed_le64_value(target)
            .map(i64::cast_unsigned)
            .map(ValueField::Explicit),
        _ => None,
    };
    let Some(value) = value else {
        return Err(TransactionRefusal::MaturityDeploymentAmountDisagreesWithView);
    };
    if value != validated.view().value() {
        return Err(TransactionRefusal::MaturityDeploymentAmountDisagreesWithView);
    }

    Ok((asset, value))
}

/// Hold the continuity equality between two adjacent constructors.
///
/// Its own function because the equality is the assertion a constructor
/// migration would replace, and because the refusal it carries is the
/// tapscript layer's own: which of the three fixed parameters differs is
/// a fact that layer has the word for, and flattening it into a
/// construction failure would discard the only part of the finding worth
/// reading.
///
/// # Errors
///
/// [`TransactionRefusal::MaturitySuccessorContinuityRefused`] when the
/// two constructors do not share their static root, internal key and
/// leaf version.
pub(crate) fn hold_continuity(
    predecessor: &CandidateStateConstructor,
    successor: &CandidateStateConstructor,
) -> Result<(), TransactionRefusal> {
    predecessor
        .continuity(successor)
        .map_err(|refusal| TransactionRefusal::MaturitySuccessorContinuityRefused { refusal })
}
