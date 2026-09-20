//! Construction of the maturity announcement (§12.5), and the first of
//! §12.7's five states.
//!
//! # A construction is built, and it is not finalized
//!
//! [`MaturityConstruction`] is a candidate transaction that exists:
//! version, one input, one output, a lock time, and a null witness per
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
//! 7. optional sponsorship is fixed here, by there being none.
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
//! # The sponsored arm, and the condition under which it is refused
//!
//! Only the sponsorless candidate is built. A sponsored request is
//! representable and is refused by name, because the form has no
//! carrier: the reduced announcement leaf carries no sponsor check at
//! all, and the realization's sponsor relations have no region until a
//! later refit gives them one, so bytes emitted for the sponsored form
//! this generation would be constrained by no leaf and by no model. The
//! refusal names the absent carrier rather than pretending the request
//! was malformed; the request is well formed, and what it asks for does
//! not exist yet.

use linker::{LinkedArtifactStatus, StateLinkSymbol, StateSymbolValue};
use realization::{StateMetadata, announce_maturity};
use tapscript::{CandidateStateConstructor, StateCurveCapability};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::bytes::{
    AssetField, AssetId, InputWitness, NonceField, TargetInput, TargetOutput, TargetTransaction,
    ValueField,
};
use crate::error::TransactionRefusal;
use crate::state_abi::CandidateMaturityAnnouncementAbi;
use crate::state_request::MaturityAnnouncementRequest;
use crate::state_view::ValidatedMaturityStateView;

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
}

impl MaturityConstruction {
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
/// [`TransactionRefusal::SponsoredMaturityFormHasNoCarrier`] for a
/// request asking for the sponsored form,
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
    // Step 1: validate the semantic request. The sponsorless form with a
    // sponsor-change output is already refused by the request's own
    // constructor, so what is left here is the pair of facts a request
    // cannot settle alone — whether the form has a carrier at all, and
    // whether the artifacts handed in describe one deployment.
    if request.requested_form().sponsored() {
        return Err(TransactionRefusal::SponsoredMaturityFormHasNoCarrier);
    }
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
    // zero and the successor at output zero, and with one input and one
    // output there is no other position for either to occupy, so the
    // layout is read rather than restated as a second constraint.
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

    // Step 7: fix optional sponsorship, of which there is none. The
    // sponsorless form has no sponsor region, and the reviewed verdict
    // the ABI publishes fixes no fee output for the announcement form,
    // so the candidate is exactly one input and one output. That is
    // consensus-valid on the target without a fee output because the
    // singleton amount is conserved: the spent output and the successor
    // carry the same asset and the same amount, so the per-asset balance
    // has no difference for a fee to make up. It is not relayable under
    // default policy, and for a reason no construction choice here could
    // answer: the announcement's metadata witness item is wider than the
    // stack-item limit the relay path applies, which is the refusal the
    // ABI publishes together with direct submission to a producer as the
    // route.
    let transaction = TargetTransaction::new(
        abi.version().version(),
        vec![input],
        vec![output],
        MATURITY_ANNOUNCEMENT_LOCK_TIME,
        vec![InputWitness::default()],
    )?;

    Ok(MaturityConstruction {
        transaction,
        successor_constructor,
        successor_metadata,
        request: *request,
        validated: validated.clone(),
        abi: abi.clone(),
    })
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
