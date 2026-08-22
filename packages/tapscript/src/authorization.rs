//! The selected owner sighash profile, and the owner-key encoding
//! closure it depends on (Guide-13 §1.7, §1.8, §9.2).
//!
//! # Why the selection lives here and not in the target package
//!
//! `target_elements::authorization` states what the target *offers* and
//! says in as many words that it selects no profile. That is the right
//! boundary: a target that named the profile its consumers should use
//! would be answering a protocol question with a target fact. This
//! module is where the answer belongs, because choosing a profile is an
//! act of *assessment* — it needs the protocol's list of protected data
//! on one side and the target's dimension vocabulary on the other, and
//! the adapter is the only place holding both.
//!
//! # A selection is not an availability claim
//!
//! Nothing here says the profile can be used. [`OwnerSighashProfile`]
//! states which commitments the protocol requires and which target
//! dimensions carry them; [`OwnerSighashProfile::assess`] then reports
//! what the *reviewed* contract establishes about those dimensions, and
//! the honest answer today is that it establishes none of them. The
//! review reached the signature primitives and stopped short of the
//! sighash type enumeration and the message construction, so a profile
//! that reported itself available would be reporting a review that did
//! not happen.
//!
//! Keeping the two apart is what makes the selection worth writing down
//! before the review exists. The argument for the profile — that these
//! protected data need those commitments — is a protocol argument that
//! does not become truer when a reviewer reads the target's source. What
//! the review changes is only whether the target can be relied on to
//! honour it.
//!
//! # Why the profile refuses dimensions rather than ignoring them
//!
//! Every dimension the target offers is classified, required or
//! refused, for the same reason
//! [`target_elements::SighashCapability`] classifies every dimension
//! reviewed or unreviewed: a dimension nobody mentioned is
//! indistinguishable from a dimension somebody forgot. The two refused
//! dimensions are the narrowing ones, and refusing them is the whole
//! content of §1.7's "all-inputs, all-outputs profile unless a narrower
//! profile is separately proved to preserve every required commitment".
//!
//! # What this module does not do
//!
//! It builds no message, hashes nothing, and holds no key. A profile is
//! a statement about which transaction dimensions a signature must
//! commit to; the digest that realizes it is the target's to compute
//! and a target-native run's to observe. §1.7 requires the selected
//! profile to be read back from the finalized witness or recomputed
//! from the exact signing request, and neither is a thing a static
//! assessment can do.

use std::collections::{BTreeMap, BTreeSet};

use target_elements::{
    AuthorizationContract, EncodingClass, SighashCapability, SighashDimension,
    UnknownPublicKeyTypeRule,
};

use crate::capability::census_enum;

census_enum! {
    /// One item of protected data a signing request must have fixed
    /// (§1.7).
    ///
    /// The protocol's own list, in the guide's order. It is the reason
    /// the profile below requires what it requires: each member names
    /// something whose alteration after signing would change what the
    /// owner authorized, and a profile is exactly a claim that no such
    /// alteration survives.
    ///
    /// Sponsor data appears twice and neither entry is an amount. §1.9
    /// keeps individual sponsor values out of every published field, and
    /// what the profile commits to is the sponsor region's *shape* —
    /// which inputs are in it and whether a change output exists — not
    /// what any of them is worth. A profile that committed to the shape
    /// and a field that published the value would be two different
    /// mistakes, and this census makes only the first one sayable.
    pub enum ProtectedDatum {
        /// The exact receipt inputs being consumed.
        ReceiptInputs,
        /// The destination entries the transfer creates.
        DestinationEntries,
        /// The owner each destination is created for.
        DestinationOwners,
        /// The semantic value each destination carries.
        DestinationSemanticValues,
        /// The explicit value or value commitment field of each output.
        ExplicitValuesOrCommitments,
        /// The constructor each created receipt is built under.
        ReceiptConstructors,
        /// Which representation plan the transfer was built under.
        RepresentationPlan,
        /// The sponsor inputs, where the profile commits them.
        SponsorInputs,
        /// The sponsor change output, present or absent.
        SponsorChange,
        /// The output carrying the target's fee role.
        TargetFeeRole,
        /// The rangeproof and surjection-proof fields the digest covers.
        ProofFields,
        /// The transaction version field.
        TransactionVersion,
        /// The transaction locktime field.
        LockTime,
        /// The issuance fields, including their absence.
        IssuanceFields,
        /// The taproot script-path fields the spend depends on.
        ScriptPathFields,
    }
}

/// Why the profile refuses one dimension.
///
/// A refusal is a claim with content, not a shrug. Each ground names
/// what the dimension would stop protecting, so a later wave proposing
/// the narrower profile §1.7 permits knows exactly which commitment it
/// owes a separate proof for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DimensionRefusal {
    /// Committing to one output leaves every other output free to
    /// change after signing.
    ///
    /// Under this profile the owner authorizes a whole transfer — a
    /// destination multiset and an aggregate conservation — and a
    /// single-output commitment authorizes one entry of it. The rest of
    /// the transfer would be whatever the builder pleased.
    LeavesOtherOutputsFree,
    /// Permitting later inputs leaves the consumed receipt set free to
    /// grow after signing.
    ///
    /// An added input is an added source of the conserved asset, so a
    /// transfer the owner signed as balanced becomes one that moves
    /// value the owner never saw.
    LeavesInputSetOpen,
    /// The target offers a dimension this profile has not considered.
    ///
    /// [`target_elements::SighashDimension`] is non-exhaustive, so a
    /// dimension added to it cannot be made a compile failure here the
    /// way a census inside one crate can. Refusing it is the
    /// fail-closed answer: an unconsidered dimension is one nobody has
    /// argued preserves the protected data, and a profile that admitted
    /// it by default would widen its own commitment claim without
    /// anybody deciding to.
    ///
    /// [`profile_classifies_every_offered_dimension`] is the loud
    /// signal the compiler cannot give.
    ///
    /// [`profile_classifies_every_offered_dimension`]: crate::authorization::profile_classifies_every_offered_dimension
    Unclassified,
}

/// What one profile requires of one target dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DimensionRole {
    /// The signature must commit to this dimension.
    Required,
    /// The signature must not be taken under this dimension.
    Refused(DimensionRefusal),
}

/// What the reviewed contract establishes about a selected profile.
///
/// Two members and not a boolean, because "unavailable" would merge the
/// target refusing a dimension with the review never having read it.
/// Only the second is true today, and only the second is repaired by
/// reviewing rather than by choosing a different target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerProfileDisposition {
    /// Every dimension the profile requires is established by the
    /// review, and no refused dimension is forced.
    Established,
    /// The target offers the dimensions; this review did not establish
    /// them.
    ///
    /// A backend must treat every named dimension as unavailable and
    /// say so, exactly as
    /// [`target_elements::SighashCapability`] requires of any consumer
    /// of an unreviewed dimension.
    ReviewIncomplete {
        /// The required dimensions the review did not reach, in census
        /// order.
        unreviewed: BTreeSet<SighashDimension>,
    },
}

/// The owner sighash profile this candidate selects (§1.7, §9.2).
///
/// Read-only, with private fields and no public constructor: the sole
/// route to a value is [`selected_owner_profile`], which classifies
/// every dimension the target offers and proves that every protected
/// datum lands on a required one. A profile assembled from arbitrary
/// sets would be a preference rather than an argument, and nothing
/// downstream could tell the two apart once they shared a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerSighashProfile {
    roles: BTreeMap<SighashDimension, DimensionRole>,
    coverage: BTreeMap<ProtectedDatum, SighashDimension>,
}

impl OwnerSighashProfile {
    /// Every dimension the signature must commit to, in census order.
    pub fn required(&self) -> impl Iterator<Item = SighashDimension> + '_ {
        self.roles
            .iter()
            .filter(|(_, role)| **role == DimensionRole::Required)
            .map(|(dimension, _)| *dimension)
    }

    /// Every dimension the profile refuses, with its ground.
    pub fn refused(&self) -> impl Iterator<Item = (SighashDimension, DimensionRefusal)> + '_ {
        self.roles
            .iter()
            .filter_map(|(dimension, role)| match role {
                DimensionRole::Refused(ground) => Some((*dimension, *ground)),
                DimensionRole::Required => None,
            })
    }

    /// The role this profile gives one dimension.
    ///
    /// Total over the target's dimension census: every dimension has a
    /// role, so `None` means the argument is not a dimension this
    /// target offers rather than one the profile forgot.
    #[must_use]
    pub fn role(&self, dimension: SighashDimension) -> Option<DimensionRole> {
        self.roles.get(&dimension).copied()
    }

    /// The dimension that carries one protected datum.
    ///
    /// # Panics
    ///
    /// Never for a member of [`ProtectedDatum::ALL`]: the constructor
    /// refuses to build a profile whose coverage map is not total, so a
    /// missing entry is unreachable rather than handled.
    #[must_use]
    pub fn carrier(&self, datum: ProtectedDatum) -> SighashDimension {
        *self
            .coverage
            .get(&datum)
            .expect("a selected profile covers every protected datum")
    }

    /// Every protected datum with the dimension that carries it.
    pub fn coverage(&self) -> impl Iterator<Item = (ProtectedDatum, SighashDimension)> + '_ {
        self.coverage
            .iter()
            .map(|(datum, dimension)| (*datum, *dimension))
    }

    /// What one reviewed sighash capability establishes about this
    /// profile.
    ///
    /// The capability is taken rather than assumed, so the answer moves
    /// when the review does: a contract that reviewed the dimensions
    /// would make this [`OwnerProfileDisposition::Established`] with no
    /// edit here, and a contract that lost one would take it back out.
    #[must_use]
    pub fn assess(&self, capability: &SighashCapability) -> OwnerProfileDisposition {
        let unreviewed = self
            .required()
            .filter(|dimension| !capability.reviewed().contains(dimension))
            .collect::<BTreeSet<_>>();

        if unreviewed.is_empty() {
            OwnerProfileDisposition::Established
        } else {
            OwnerProfileDisposition::ReviewIncomplete { unreviewed }
        }
    }
}

/// The profile Guide 13 selects for owner authorization (§1.7).
///
/// # The selection argument
///
/// §1.7 fixes the conclusion — an all-inputs, all-outputs profile
/// unless a narrower one is separately proved to preserve every
/// required commitment — and the coverage map below is the argument for
/// it, one protected datum at a time. Nothing narrower survives it:
/// eleven of the fifteen protected data are output-side or input-side
/// facts about the *whole* transaction, so a profile that dropped
/// either total commitment would leave a majority of the list
/// unprotected, and the remaining four are single fields the target
/// commits to in any case.
///
/// The two refusals are therefore not spare caution. They are the two
/// dimensions the target offers that would each, on their own, undo one
/// of the two totals the argument rests on.
#[must_use]
pub fn selected_owner_profile() -> OwnerSighashProfile {
    use SighashDimension as Dimension;

    let roles = SighashDimension::ALL
        .iter()
        .map(|dimension| {
            let role = match dimension {
                Dimension::SingleOutput => {
                    DimensionRole::Refused(DimensionRefusal::LeavesOtherOutputsFree)
                }
                Dimension::InputExtensionPermitted => {
                    DimensionRole::Refused(DimensionRefusal::LeavesInputSetOpen)
                }
                Dimension::AllOutputs
                | Dimension::AllInputs
                | Dimension::Issuance
                | Dimension::Version
                | Dimension::LockTime
                | Dimension::TapleafHash
                | Dimension::InternalKey
                | Dimension::SpentOutputs => DimensionRole::Required,
                _ => DimensionRole::Refused(DimensionRefusal::Unclassified),
            };

            (*dimension, role)
        })
        .collect();

    let coverage = ProtectedDatum::ALL
        .iter()
        .map(|datum| {
            let dimension = match datum {
                // Which receipts are consumed is the input set itself,
                // and what each of them was worth is a property of the
                // outputs being spent rather than of this transaction's
                // own fields.
                ProtectedDatum::ReceiptInputs | ProtectedDatum::SponsorInputs => {
                    Dimension::AllInputs
                }
                // Everything a destination *is* — its existence, its
                // owner's program, its value field, and the constructor
                // that program encodes — is carried in the output list,
                // and a commitment to the whole list carries all four at
                // once. Sponsor change and the fee role are outputs on
                // the same list; committing to their presence is what
                // keeps a balanced-theft rearrangement from surviving.
                //
                // The representation plan joins them because it is not
                // a field of its own. It is visible as the *form* the
                // output value fields take, explicit or committed, so
                // what commits to the plan is the commitment to those
                // fields: a transaction whose outputs were rebuilt
                // under the other plan is a different output list.
                ProtectedDatum::DestinationEntries
                | ProtectedDatum::DestinationOwners
                | ProtectedDatum::DestinationSemanticValues
                | ProtectedDatum::ExplicitValuesOrCommitments
                | ProtectedDatum::ReceiptConstructors
                | ProtectedDatum::SponsorChange
                | ProtectedDatum::TargetFeeRole
                | ProtectedDatum::RepresentationPlan => Dimension::AllOutputs,
                // The spent outputs carry the input side's own value and
                // asset fields, which is where the proofs a confidential
                // transfer relies on are anchored.
                ProtectedDatum::ProofFields => Dimension::SpentOutputs,
                ProtectedDatum::TransactionVersion => Dimension::Version,
                ProtectedDatum::LockTime => Dimension::LockTime,
                ProtectedDatum::IssuanceFields => Dimension::Issuance,
                ProtectedDatum::ScriptPathFields => Dimension::TapleafHash,
            };

            (*datum, dimension)
        })
        .collect();

    OwnerSighashProfile { roles, coverage }
}

/// What an owner check owes beyond the signature primitive's answer
/// (§1.8).
///
/// Derived from the target's own unknown-key rule rather than fixed, so
/// the obligation tracks the target: a target that refused unrecognized
/// keys would move this to
/// [`Self::SignatureResultSuffices`] without an edit here, and a target
/// that stopped refusing them would move it back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerKeyObligation {
    /// The pattern must authenticate the key's encoding itself, before
    /// relying on the signature result.
    ///
    /// The reviewed primitive reports success *without verifying* for
    /// an unrecognized nonempty key, so a program that read the
    /// signature result alone would accept a witness that proved
    /// nothing. An accepted owner check is the conjunction §1.8 states:
    /// the key has the approved encoding, and the signature verifies
    /// against that key. A forward-compatibility success path is not
    /// authorization.
    AuthenticateEncodingIndependently,
    /// The target refuses an unrecognized key, so its own answer
    /// already carries the encoding.
    SignatureResultSuffices,
}

census_enum! {
    /// One negative case the owner-key closure requires (§1.8).
    ///
    /// Six cases and not one, because they fail for six different
    /// reasons and a harness that ran one of them would learn nothing
    /// about the other five. The first three probe the encoding gate;
    /// the last three probe the signature itself against a key or a
    /// transaction the owner did not authorize.
    pub enum OwnerKeyNegative {
        /// The offered key is empty.
        EmptyKey,
        /// The offered key is nonempty and of a type the target does
        /// not recognize — the case the forward-compatibility path
        /// would otherwise let through.
        UnknownNonemptyKeyType,
        /// The offered key claims the approved encoding and is not a
        /// well-formed member of it.
        MalformedApprovedKey,
        /// The offered key is well-formed and approved, and belongs to
        /// somebody other than the required owner.
        ApprovedKeyOfAnotherOwner,
        /// The signature verifies, against a key the check did not
        /// require.
        ValidSignatureAgainstAnotherKey,
        /// The signature verifies, over a transaction other than the
        /// one being authorized.
        ValidSignatureOverAnotherTransaction,
    }
}

/// The owner-key encoding closure this target obliges (§1.8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerKeyEncodingClosure {
    approved: EncodingClass,
    unknown_key_rule: UnknownPublicKeyTypeRule,
    obligation: OwnerKeyObligation,
}

impl OwnerKeyEncodingClosure {
    /// The encoding an owner key must carry.
    #[must_use]
    pub const fn approved(&self) -> EncodingClass {
        self.approved
    }

    /// What the target does with a key encoding it does not recognize.
    #[must_use]
    pub const fn unknown_key_rule(&self) -> UnknownPublicKeyTypeRule {
        self.unknown_key_rule
    }

    /// What a pattern must do about it.
    #[must_use]
    pub const fn obligation(&self) -> OwnerKeyObligation {
        self.obligation
    }

    /// Every negative case the closure requires, in census order.
    ///
    /// The census is complete whichever obligation holds. A target that
    /// refused unknown keys would still owe every one of these: what
    /// its refusal changes is which component establishes the first
    /// three, not whether a run has to show them failing.
    pub fn negatives(&self) -> impl Iterator<Item = OwnerKeyNegative> {
        OwnerKeyNegative::ALL.iter().copied()
    }
}

/// The closure one reviewed authorization contract obliges.
#[must_use]
pub const fn owner_key_encoding_closure(
    contract: &AuthorizationContract,
) -> OwnerKeyEncodingClosure {
    let signature = contract.signature();
    let unknown_key_rule = signature.unknown_public_key_type();

    OwnerKeyEncodingClosure {
        approved: signature.public_key_encoding(),
        unknown_key_rule,
        obligation: match unknown_key_rule {
            UnknownPublicKeyTypeRule::Rejected => OwnerKeyObligation::SignatureResultSuffices,
            // Fail-closed, and the default arm exists only because the
            // target's rule is non-exhaustive. Every rule other than an
            // outright refusal leaves *some* key the primitive answers
            // for without verifying, and a pattern that authenticated
            // the encoding it did not need to has lost nothing but a
            // few bytes. The reverse mistake loses the authorization.
            UnknownPublicKeyTypeRule::SucceedsWithoutVerification | _ => {
                OwnerKeyObligation::AuthenticateEncodingIndependently
            }
        },
    }
}

/// Whether the selected profile classifies every dimension the target
/// offers.
///
/// The signal [`DimensionRefusal::Unclassified`] exists to raise. It is
/// a function rather than a test assertion because a consumer assessing
/// this target is entitled to ask the same question before relying on
/// the profile, and a dimension the target grew after this crate was
/// written is exactly the kind of thing a consumer would rather be told
/// about than infer.
#[must_use]
pub fn profile_classifies_every_offered_dimension(profile: &OwnerSighashProfile) -> bool {
    profile
        .refused()
        .all(|(_, ground)| ground != DimensionRefusal::Unclassified)
}
