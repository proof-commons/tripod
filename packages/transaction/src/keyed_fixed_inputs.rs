//! Parent-bound input facts. Collateral authorization is deferred, not asserted.
use crate::keyed_fixed_taproot::{
    AuthenticatedKeyedFixedPredecessor, KeyedFixedAuthenticationRefusal, KeyedFixedCommitment,
    authenticate_keyed_fixed_predecessor_core,
};
use crate::{
    AssetField, AssetId, Outpoint, SponsorCapability, SponsorOffer, TargetOutput,
    TargetTransaction, TransactionRefusal, Txid, ValueField,
};
use sha2::{Digest, Sha256};

/// A role offer, checked against the decoded output before sponsor exclusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedInputRole {
    Member,
    Collateral,
}

/// Untrusted request input. Position is its actual proposed transaction index.
pub struct KeyedFixedInputOffer<'parent, 'capability> {
    pub input_index: u32,
    pub parent: &'parent [u8],
    pub outpoint: Outpoint,
    pub sequence: u32,
    /// Offered spending-input flags (`GetOutpointFlag`); this packet accepts only zero.
    pub flags: u8,
    pub role: KeyedFixedInputRole,
    pub capability: Option<&'capability dyn SponsorCapability>,
}

/// Names the shared decoder's refusal, not a proven parent form or covenant invalidity.
///
/// `Issuance` also covers the coinbase null-prevout spelling, which trips the
/// issuance-bit check first. `PegIn` includes a nonempty peg-in witness on an
/// unflagged input. Issuance proofs on an unflagged input remain
/// `KeyedFixedInputRefusal::Decode(TransactionRefusal::IssuanceProofRefused)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnsupportedParentForm {
    Issuance,
    PegIn,
}

/// Parent, role and capability-offer refusals retain their own boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedInputRefusal {
    UnsupportedForm(UnsupportedParentForm),
    Decode(TransactionRefusal),
    NonCanonicalParent,
    ParentHash { offered: Txid, computed: Txid },
    MissingOutput(u32),
    Authentication(KeyedFixedAuthenticationRefusal),
    FalseCollateralRole,
    CapabilityDoesNotNameOutpoint(Outpoint),
    MemberCapability,
    InvalidOutpointFlags(u8),
    UnsupportedSpendingInputForm(u8),
    LaterBornMember(u32),
    WrongMemberAsset,
    WrongMemberValue,
    WrongMemberForm,
}
impl KeyedFixedInputRefusal {
    /// Stable tracing classification; details remain in the typed refusal.
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        crate::trace_support::TraceRefusal::tracing_kind(self)
    }
}
impl crate::trace_support::TraceRefusal for KeyedFixedInputRefusal {
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
            Self::WrongMemberAsset => "wrong_member_asset",
            Self::WrongMemberValue => "wrong_member_value",
            Self::WrongMemberForm => "wrong_member_form",
        }
    }
}

/// Canonical parent bytes and locally reproduced selected-output facts.
/// No inclusion, unspentness, retrieval or finality is established.
/// ```compile_fail,E0451
/// use transaction::{Outpoint, TargetOutput};
/// use transaction::keyed_fixed_inputs::BoundParentOutput;
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
/// use transaction::keyed_fixed_inputs::{BoundKeyedFixedInput, BoundParentOutput};
/// fn forge(parent: BoundParentOutput) -> BoundKeyedFixedInput<'static> {
///     BoundKeyedFixedInput { input_index: 0, sequence: 0, flags: 0, parent,
///         predecessor: None, capability: None, capability_offer: None }
/// }
/// ```
pub struct BoundKeyedFixedInput<'a> {
    input_index: u32,
    sequence: u32,
    flags: u8,
    parent: BoundParentOutput,
    predecessor: Option<AuthenticatedKeyedFixedPredecessor<'a>>,
    capability: Option<&'a dyn SponsorCapability>,
    capability_offer: Option<SponsorOffer>,
}
impl std::fmt::Debug for BoundKeyedFixedInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundKeyedFixedInput")
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
impl<'a> BoundKeyedFixedInput<'a> {
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
    pub const fn predecessor(&self) -> Option<&AuthenticatedKeyedFixedPredecessor<'a>> {
        self.predecessor.as_ref()
    }
    /// Retained callback only. Neither availability nor authority to sign is proved.
    #[must_use]
    pub const fn capability(&self) -> Option<&'a dyn SponsorCapability> {
        self.capability
    }
    #[must_use]
    pub const fn role(&self) -> KeyedFixedInputRole {
        if self.predecessor.is_some() {
            KeyedFixedInputRole::Member
        } else {
            KeyedFixedInputRole::Collateral
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
    commitment: &'a KeyedFixedCommitment,
    offer: &KeyedFixedInputOffer<'_, 'a>,
) -> Result<BoundKeyedFixedInput<'a>, KeyedFixedInputRefusal> {
    use KeyedFixedInputRefusal as E;
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
        KeyedFixedInputRole::Member => {
            // Mint only after canonical decoding, content addressing and vout lookup.
            let receipt = authenticate_keyed_fixed_predecessor_core(commitment, output.program())
                .map_err(E::Authentication)?;
            if offer.capability.is_some() {
                return Err(E::MemberCapability);
            }
            if offer.outpoint.index() != 0 {
                return Err(E::LaterBornMember(offer.outpoint.index()));
            }
            let parameters = commitment.recipe().parameters();
            match output.asset() {
                AssetField::Explicit(asset)
                    if asset == AssetId::from_internal(parameters.asset) => {}
                AssetField::Explicit(_) => return Err(E::WrongMemberAsset),
                _ => return Err(E::WrongMemberForm),
            }
            match output.value() {
                ValueField::Explicit(value) if value == parameters.value.get() => {}
                ValueField::Explicit(_) => return Err(E::WrongMemberValue),
                _ => return Err(E::WrongMemberForm),
            }
            Some(receipt)
        }
        KeyedFixedInputRole::Collateral => {
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
    Ok(BoundKeyedFixedInput {
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
