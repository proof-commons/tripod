//! Parent-bound input facts. Collateral authorization is deferred, not asserted.
use crate::keyed_taproot::{
    AuthenticatedKeyedPredecessor, KeyedAuthenticationRefusal, KeyedCommitment,
    authenticate_keyed_predecessor_core,
};
use crate::{
    Outpoint, SponsorCapability, SponsorOffer, TargetOutput, TargetTransaction, TransactionRefusal,
    Txid,
};
use sha2::{Digest, Sha256};

/// A role offer, checked against the decoded output before sponsor exclusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedInputRole {
    Member,
    Collateral,
}

/// Untrusted request input. Position is its actual proposed transaction index.
pub struct KeyedInputOffer<'parent, 'capability> {
    pub input_index: u32,
    pub parent: &'parent [u8],
    pub outpoint: Outpoint,
    pub sequence: u32,
    /// Offered spending-input flags (`GetOutpointFlag`); this packet accepts only zero.
    pub flags: u8,
    pub role: KeyedInputRole,
    pub capability: Option<&'capability dyn SponsorCapability>,
}

/// Names the shared decoder's refusal, not a proven parent form or covenant invalidity.
///
/// `Issuance` also covers the coinbase null-prevout spelling, which trips the
/// issuance-bit check first. `PegIn` includes a nonempty peg-in witness on an
/// unflagged input. Issuance proofs on an unflagged input remain
/// `KeyedInputRefusal::Decode(TransactionRefusal::IssuanceProofRefused)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnsupportedParentForm {
    Issuance,
    PegIn,
}

/// Parent, role and capability-offer refusals retain their own boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedInputRefusal {
    UnsupportedForm(UnsupportedParentForm),
    Decode(TransactionRefusal),
    NonCanonicalParent,
    ParentHash { offered: Txid, computed: Txid },
    MissingOutput(u32),
    Authentication(KeyedAuthenticationRefusal),
    FalseCollateralRole,
    CapabilityDoesNotNameOutpoint(Outpoint),
    MemberCapability,
    InvalidOutpointFlags(u8),
    UnsupportedSpendingInputForm(u8),
    LaterBornMember(u32),
}
impl KeyedInputRefusal {
    /// Stable tracing classification; details remain in the typed refusal.
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        crate::trace_support::TraceRefusal::tracing_kind(self)
    }
}
impl crate::trace_support::TraceRefusal for KeyedInputRefusal {
    fn tracing_kind(&self) -> &'static str {
        match self {
            Self::UnsupportedForm(_) => "unsupported_form",
            Self::Decode(_) => "decode",
            Self::NonCanonicalParent => "non_canonical_parent",
            Self::ParentHash { .. } => "parent_hash",
            Self::MissingOutput(_) => "missing_output",
            Self::Authentication(_) => "authentication",
            Self::FalseCollateralRole => "false_collateral_role",
            Self::CapabilityDoesNotNameOutpoint(_) => "capability_does_not_name_outpoint",
            Self::MemberCapability => "member_capability",
            Self::InvalidOutpointFlags(_) => "invalid_outpoint_flags",
            Self::UnsupportedSpendingInputForm(_) => "unsupported_spending_input_form",
            Self::LaterBornMember(_) => "later_born_member",
        }
    }
}

/// Canonical parent bytes and locally reproduced selected-output facts.
/// No inclusion, unspentness, retrieval or finality is established.
/// ```compile_fail,E0451
/// use transaction::{Outpoint, TargetOutput};
/// use transaction::keyed_inputs::BoundParentOutput;
/// fn forge(outpoint: Outpoint, output: TargetOutput) -> BoundParentOutput {
///     BoundParentOutput { parent: vec![], outpoint, output }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundParentOutput {
    parent: Vec<u8>,
    outpoint: Outpoint,
    output: TargetOutput,
}
impl BoundParentOutput {
    #[must_use]
    pub fn parent_bytes(&self) -> &[u8] {
        &self.parent
    }
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }
    #[must_use]
    pub const fn output(&self) -> &TargetOutput {
        &self.output
    }
}

/// Input receipt with parent provenance; absence of a keyed receipt means
/// collateral bound to a parent output, with authorization deferred.
///
/// Neither role establishes spending authority: a member has local commitment equality, birth
/// position zero and an accepted zero flag offer, not covenant execution. Collateral
/// authorization belongs to the admitted signing phase. Member execution evidence
/// belongs to the later construction/conformance steps.
/// ```compile_fail,E0451
/// use transaction::keyed_inputs::{BoundKeyedInput, BoundParentOutput};
/// fn forge(parent: BoundParentOutput) -> BoundKeyedInput<'static> {
///     BoundKeyedInput { input_index: 0, sequence: 0, flags: 0, parent,
///         predecessor: None, capability: None, capability_offer: None }
/// }
/// ```
pub struct BoundKeyedInput<'a> {
    input_index: u32,
    sequence: u32,
    flags: u8,
    parent: BoundParentOutput,
    predecessor: Option<AuthenticatedKeyedPredecessor<'a>>,
    capability: Option<&'a dyn SponsorCapability>,
    capability_offer: Option<SponsorOffer>,
}
impl std::fmt::Debug for BoundKeyedInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundKeyedInput")
            .field("input_index", &self.input_index)
            .field("sequence", &self.sequence)
            .field("flags", &self.flags)
            .field("parent", &self.parent)
            .field("predecessor", &self.predecessor)
            .field("capability_present", &self.capability.is_some())
            .field("capability_offer", &self.capability_offer)
            .finish()
    }
}
impl<'a> BoundKeyedInput<'a> {
    #[must_use]
    pub const fn input_index(&self) -> u32 {
        self.input_index
    }
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }
    #[must_use]
    pub const fn flags(&self) -> u8 {
        self.flags
    }
    #[must_use]
    pub const fn parent(&self) -> &BoundParentOutput {
        &self.parent
    }
    #[must_use]
    pub const fn predecessor(&self) -> Option<&AuthenticatedKeyedPredecessor<'a>> {
        self.predecessor.as_ref()
    }
    /// Retained callback only. Neither availability nor authority to sign is proved.
    #[must_use]
    pub const fn capability(&self) -> Option<&'a dyn SponsorCapability> {
        self.capability
    }
    #[must_use]
    pub const fn role(&self) -> KeyedInputRole {
        if self.predecessor.is_some() {
            KeyedInputRole::Member
        } else {
            KeyedInputRole::Collateral
        }
    }
    /// Snapshot actually checked at binding, including fee, change and input set.
    /// Step 4 must reconcile coverage and member overlap without recalling `offer()`.
    #[must_use]
    pub const fn capability_offer(&self) -> Option<&SponsorOffer> {
        self.capability_offer.as_ref()
    }
}

pub(crate) fn bind_input<'a>(
    commitment: &'a KeyedCommitment,
    offer: &KeyedInputOffer<'_, 'a>,
) -> Result<BoundKeyedInput<'a>, KeyedInputRefusal> {
    use KeyedInputRefusal as E;
    let parent = TargetTransaction::decode_core(offer.parent).map_err(|error| match error {
        TransactionRefusal::IssuanceInputRefused => {
            E::UnsupportedForm(UnsupportedParentForm::Issuance)
        }
        TransactionRefusal::PeginInputRefused => E::UnsupportedForm(UnsupportedParentForm::PegIn),
        other => E::Decode(other),
    })?;
    if parent.encode() != offer.parent {
        return Err(E::NonCanonicalParent);
    }
    let computed =
        Txid::from_internal(Sha256::digest(Sha256::digest(parent.encode_without_witness())).into());
    if computed != offer.outpoint.txid() {
        return Err(E::ParentHash {
            offered: offer.outpoint.txid(),
            computed,
        });
    }
    let output = parent
        .outputs()
        .get(offer.outpoint.index() as usize)
        .ok_or_else(|| E::MissingOutput(offer.outpoint.index()))?
        .clone();
    // These are offers for the not-yet-materialized spending input. The current
    // encoder supports neither flag; reserved bits are outside the target domain.
    if offer.flags & !0xc0 != 0 {
        return Err(E::InvalidOutpointFlags(offer.flags));
    }
    if offer.flags != 0 {
        return Err(E::UnsupportedSpendingInputForm(offer.flags));
    }
    let predecessor = match offer.role {
        KeyedInputRole::Member => {
            // Mint only after canonical decoding, content addressing and vout lookup.
            let receipt = authenticate_keyed_predecessor_core(commitment, output.program())
                .map_err(E::Authentication)?;
            if offer.capability.is_some() {
                return Err(E::MemberCapability);
            }
            if offer.outpoint.index() != 0 {
                return Err(E::LaterBornMember(offer.outpoint.index()));
            }
            Some(receipt)
        }
        KeyedInputRole::Collateral => {
            if output.program() == commitment.output_program() {
                return Err(E::FalseCollateralRole);
            }
            None
        }
    };
    let capability_offer = offer.capability.map(SponsorCapability::offer);
    if let Some(snapshot) = &capability_offer
        && !snapshot.inputs().contains(&offer.outpoint)
    {
        return Err(E::CapabilityDoesNotNameOutpoint(offer.outpoint));
    }
    Ok(BoundKeyedInput {
        input_index: offer.input_index,
        sequence: offer.sequence,
        flags: offer.flags,
        parent: BoundParentOutput {
            parent: offer.parent.to_vec(),
            outpoint: offer.outpoint,
            output,
        },
        predecessor,
        capability: offer.capability,
        capability_offer,
    })
}
