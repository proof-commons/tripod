//! Multi-owner response collection and its ten rejections (§12.7,
//! §1.6).
//!
//! # All owners sign the same finalized protected transaction
//!
//! One request per consumed receipt, one response per request, and every
//! response bound to the same bytes. §12.7 names ten ways that can fail
//! and this module names ten refusals; the correspondence is one to one
//! and is the module's whole contract. Seven of them are answered while
//! the responses are collected and three afterwards, against an offered
//! transaction, because the last three are things a builder does to the
//! bytes rather than things a signer does to a response.
//!
//! # Three levels, kept apart
//!
//! §1.6 distinguishes distinct semantic owners, concrete receipt inputs,
//! and concrete owner signatures, and warns that three signatures from
//! one owner are not three independent owners.
//! [`OwnerAuthorizationLevels`] reports all three separately, and the
//! owner level is a set while the other two are counts — which is the
//! whole of the distinction, made in the types rather than in a comment
//! beside them.
//!
//! # No partial owner set proceeds
//!
//! [`authorize_live_transfer`] returns a value or a refusal and never a
//! partially authorized transfer. There is no field on
//! [`AuthorizedLiveTransfer`] holding an incomplete response set and no
//! constructor that would build one, so a caller cannot reach target
//! submission with a subset however it handles the error.
//!
//! # What this module does not check
//!
//! Whether a signature verifies. This crate has no curve arithmetic
//! (§7.5's boundary, restated in [`crate::live_taproot`]), and the
//! signature is carried as opaque bytes bound to a preimage. What is
//! checked here is *binding*: which input, which owner, which profile,
//! which bytes. Verification belongs to the package that owns the curve,
//! and the digest a target actually forms belongs to a target-native
//! run.

use std::collections::{BTreeMap, BTreeSet};

use linker::OwnerParameter;
use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::SighashDimension;

use crate::bytes::{InputWitness, TargetTransaction};
use crate::error::TransactionRefusal;
use crate::live_finalize::{FinalizedLiveTransfer, LiveSigningRequest};

/// One protocol owner's answer to one signing request (§12.7).
///
/// The signature is opaque bytes. What makes the response checkable is
/// everything around it: which input it answers, which owner it claims
/// to be, which dimensions it says it committed to, and which exact
/// bytes it was taken over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveOwnerResponse {
    input: u16,
    owner: OwnerParameter,
    representation: LiveTransferRepresentationPlan,
    committed: BTreeSet<SighashDimension>,
    bound_to: Vec<u8>,
    signature: Vec<u8>,
}

impl LiveOwnerResponse {
    /// The response answering one request exactly.
    ///
    /// Every field except the signature is copied from the request, so
    /// a conforming response cannot drift from what was asked. The ten
    /// rejections are still reachable, through
    /// [`Self::from_parts`] — which exists so that a negative case can
    /// state exactly one disagreement rather than being unable to state
    /// any.
    #[must_use]
    pub fn to(request: &LiveSigningRequest, signature: Vec<u8>) -> Self {
        Self {
            input: request.input(),
            owner: request.owner().clone(),
            representation: request.representation(),
            committed: request.required_dimensions().clone(),
            bound_to: request.protected_bytes().to_vec(),
            signature,
        }
    }

    /// The response one signer offers, part by part.
    ///
    /// The constructor a disagreeing response needs. A negative case is
    /// a response that differs from its request in exactly one place,
    /// and a type that could only copy a request would make every §12.7
    /// rejection unreachable and therefore untested.
    #[must_use]
    pub const fn from_parts(
        input: u16,
        owner: OwnerParameter,
        representation: LiveTransferRepresentationPlan,
        committed: BTreeSet<SighashDimension>,
        bound_to: Vec<u8>,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            input,
            owner,
            representation,
            committed,
            bound_to,
            signature,
        }
    }

    /// Which input the response says it answers.
    #[must_use]
    pub const fn input(&self) -> u16 {
        self.input
    }

    /// Which owner the response claims to be.
    #[must_use]
    pub const fn owner(&self) -> &OwnerParameter {
        &self.owner
    }

    /// Which representation plan the response was taken under.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// The dimensions the response says its signature commits to.
    #[must_use]
    pub const fn committed_dimensions(&self) -> &BTreeSet<SighashDimension> {
        &self.committed
    }

    /// The exact bytes the response was taken over.
    #[must_use]
    pub fn bound_to(&self) -> &[u8] {
        &self.bound_to
    }

    /// The signature, as opaque bytes.
    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

/// The three levels §1.6 keeps distinct.
///
/// One owner may control several receipt inputs. The semantic owner set
/// contains that owner once, while the target realization requires one
/// signature per input, because the target message is input-specific.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerAuthorizationLevels {
    distinct_semantic_owners: BTreeSet<OwnerParameter>,
    receipt_inputs: usize,
    owner_signatures: usize,
}

impl OwnerAuthorizationLevels {
    /// Every distinct semantic owner represented, in canonical order.
    #[must_use]
    pub const fn distinct_semantic_owners(&self) -> &BTreeSet<OwnerParameter> {
        &self.distinct_semantic_owners
    }

    /// How many concrete receipt inputs are consumed.
    #[must_use]
    pub const fn receipt_inputs(&self) -> usize {
        self.receipt_inputs
    }

    /// How many concrete owner signatures were collected.
    #[must_use]
    pub const fn owner_signatures(&self) -> usize {
        self.owner_signatures
    }

    /// Whether some owner authorized more than one input.
    ///
    /// The condition §1.6 warns a report against collapsing. False for a
    /// transfer whose owners hold one receipt each, and true exactly
    /// when the owner set is smaller than the input census.
    #[must_use]
    pub fn some_owner_holds_several_inputs(&self) -> bool {
        self.distinct_semantic_owners.len() < self.receipt_inputs
    }
}

/// One finalized transfer every required owner authorized.
///
/// There is no variant of this type holding a partial response set:
/// [`authorize_live_transfer`] returns one of these or a refusal, so a
/// caller holding one is holding a complete owner set (§12.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedLiveTransfer {
    finalized: FinalizedLiveTransfer,
    levels: OwnerAuthorizationLevels,
    witnesses: BTreeMap<u16, InputWitness>,
}

impl AuthorizedLiveTransfer {
    /// The finalized form every response is bound to.
    #[must_use]
    pub const fn finalized(&self) -> &FinalizedLiveTransfer {
        &self.finalized
    }

    /// The three §1.6 levels.
    #[must_use]
    pub const fn levels(&self) -> &OwnerAuthorizationLevels {
        &self.levels
    }

    /// Every receipt input's assembled witness, in position order.
    #[must_use]
    pub const fn witnesses(&self) -> &BTreeMap<u16, InputWitness> {
        &self.witnesses
    }

    /// Whether an offered transaction is the one that was authorized
    /// (§12.7).
    ///
    /// # Errors
    ///
    /// The three post-boundary refusals of
    /// [`FinalizedLiveTransfer::check_offered`].
    pub fn check_offered(&self, offered: &TargetTransaction) -> Result<(), TransactionRefusal> {
        self.finalized.check_offered(offered)
    }
}

/// Collect owner responses against one finalized transfer (§12.7).
///
/// Responses are offered as `(position, response)` pairs rather than as
/// a map, so that a response offered twice for one position is a case
/// this function can see. A map would have collapsed the duplicate
/// before it arrived, which is exactly the rejection §12.7 asks for.
///
/// # Errors
///
/// [`TransactionRefusal::OwnerResponseDuplicated`] for a position
/// answered twice; [`TransactionRefusal::UnexpectedSigner`] for a
/// position that is not a receipt input;
/// [`TransactionRefusal::ResponseForWrongInput`] when a response answers
/// a different request than the position it is offered at;
/// [`TransactionRefusal::ResponseFromWrongOwner`] when it claims an
/// owner the input does not authenticate;
/// [`TransactionRefusal::ResponseUnderWrongSighashProfile`] when its
/// committed dimensions are not the selected profile's;
/// [`TransactionRefusal::ResponseBoundToDifferentBytes`] when it was
/// taken over bytes other than the finalized serialization; and
/// [`TransactionRefusal::OwnerResponseMissing`] for a receipt input no
/// response answers.
pub fn authorize_live_transfer(
    finalized: FinalizedLiveTransfer,
    responses: impl IntoIterator<Item = (u16, LiveOwnerResponse)>,
) -> Result<AuthorizedLiveTransfer, TransactionRefusal> {
    let records: BTreeMap<_, _> = finalized
        .receipts()
        .iter()
        .map(|record| (record.position(), record))
        .collect();

    let mut collected: BTreeMap<u16, LiveOwnerResponse> = BTreeMap::new();
    for (position, response) in responses {
        if collected.contains_key(&position) {
            return Err(TransactionRefusal::OwnerResponseDuplicated { input: position });
        }
        let record = records
            .get(&position)
            .ok_or(TransactionRefusal::UnexpectedSigner { input: position })?;

        if response.input != position {
            return Err(TransactionRefusal::ResponseForWrongInput { input: position });
        }
        if &response.owner != record.owner() {
            return Err(TransactionRefusal::ResponseFromWrongOwner { input: position });
        }
        if &response.committed != finalized.required_dimensions() {
            return Err(TransactionRefusal::ResponseUnderWrongSighashProfile { input: position });
        }
        if response.bound_to != finalized.protected_bytes() {
            return Err(TransactionRefusal::ResponseBoundToDifferentBytes { input: position });
        }

        collected.insert(position, response);
    }

    for position in records.keys() {
        if !collected.contains_key(position) {
            return Err(TransactionRefusal::OwnerResponseMissing { input: *position });
        }
    }

    let witnesses = records
        .iter()
        .map(|(position, record)| {
            let response = collected
                .get(position)
                .ok_or(TransactionRefusal::OwnerResponseMissing { input: *position })?;
            // The ABI's witness order: the owner signature the leaf
            // checks, then the leaf, then the control block that
            // authenticates it.
            Ok((
                *position,
                InputWitness::new(vec![
                    response.signature.clone(),
                    record.leaf_script().to_vec(),
                    record.control_block().to_vec(),
                ]),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, TransactionRefusal>>()?;

    let levels = OwnerAuthorizationLevels {
        distinct_semantic_owners: records
            .values()
            .map(|record| record.owner().clone())
            .collect(),
        receipt_inputs: records.len(),
        owner_signatures: collected.len(),
    };

    Ok(AuthorizedLiveTransfer {
        finalized,
        levels,
        witnesses,
    })
}
