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

/// Why a dimension the target names is carried by no message term.
///
/// One member, because the source review found exactly one such
/// dimension and a vocabulary with room for unexamined others would
/// invite a later dimension to be filed here instead of read. A second
/// member is an added variant with its own argument, not a default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutsideMessageGround {
    /// The message fixes a value the dimension's own value is composed
    /// with, and the composition is completed by a consensus check the
    /// message does not cover.
    ///
    /// This is the internal key's ground, and it is the source review's
    /// argument rather than a summary of it
    /// (`rule:sighash-review:internal-key`). What the spent-scripts term
    /// commits, at `src/script/interpreter.cpp:2465-2472` written at
    /// `:2736`, is each spent output's `scriptPubKey`; for a taproot
    /// output the 32 bytes inside that program are the **tweaked output
    /// key**, not the internal key, and the two are related by a tweak
    /// the message never mentions.
    ///
    /// The internal key is bound to that output key by
    /// `VerifyTaprootCommitment` at `:3217-3229`, which reads the
    /// internal key from the control block at `:3222`, the output key
    /// from the witness program at `:3224`, computes the merkle root
    /// from the executing leaf and the supplied path at `:3226`, and
    /// requires at `:3228` that the output key be the internal key
    /// tweaked by that root — a check the script-path branch runs at
    /// `:3288-3290`, refusing with a witness-program mismatch before the
    /// leaf executes.
    ///
    /// So the relation is a composition and not a commitment, for two
    /// reasons the review refuses to blur. The binding lives in the
    /// witness check and the witness is not covered by the message:
    /// nothing in the stream at `:2712-2801` reads the control block.
    /// And the tweak is not injective in the message's view — the
    /// message is identical for any two internal-key-and-path pairs that
    /// tweak to the same output key under the same executing leaf,
    /// because the message carries the output key and the tapleaf hash
    /// and nothing about the path. That no such pair is easy to find is
    /// a hardness argument, not a commitment.
    ComposedThroughTheControlBlockCheck,
}

/// What one profile requires of one target dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DimensionRole {
    /// The signature must commit to this dimension.
    Required,
    /// The signature must not be taken under this dimension.
    Refused(DimensionRefusal),
    /// The message carries no term for this dimension, and the
    /// protection the profile wanted from it is recorded as carried by
    /// another dimension instead.
    ///
    /// Three roles and not two, because the two the profile started with
    /// could not say this. Requiring a dimension no message term carries
    /// makes [`OwnerSighashProfile::assess`] permanently
    /// [`OwnerProfileDisposition::ReviewIncomplete`] — no reading of the
    /// message and no recomputation of it can ever move a dimension the
    /// message does not contain — and refusing it would have claimed the
    /// profile declines a protection it in fact has. Neither is true, so
    /// the vocabulary grew a third answer rather than one of the two
    /// being stretched.
    ///
    /// The variant carries its own reason so that the argument travels
    /// with the dimension. A reader who asks why the required set is
    /// seven rather than eight finds
    /// [`OutsideMessageGround`] and its citations at the point of use,
    /// not a dimension that quietly stopped being mentioned.
    NotCarriedByTheMessage {
        /// The dimension whose term fixes the value this one composes
        /// with.
        carried_by: SighashDimension,
        /// Why the message itself carries nothing.
        ground: OutsideMessageGround,
    },
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
    coverage: BTreeMap<ProtectedDatum, BTreeSet<SighashDimension>>,
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
                DimensionRole::Required | DimensionRole::NotCarriedByTheMessage { .. } => None,
            })
    }

    /// Every dimension no message term carries, with the dimension that
    /// carries its protection instead and the ground for saying so.
    ///
    /// Published rather than left inside [`Self::role`] because the set
    /// is the answer to a question a reader of the required set will
    /// ask: the profile names ten dimensions and requires seven, and the
    /// three that are not required split into two refusals with grounds
    /// and one composition with a citation. An iterator makes the third
    /// group as walkable as the second instead of reachable only by
    /// asking about a dimension one already suspected.
    pub fn not_carried_by_the_message(
        &self,
    ) -> impl Iterator<Item = (SighashDimension, SighashDimension, OutsideMessageGround)> + '_ {
        self.roles
            .iter()
            .filter_map(|(dimension, role)| match role {
                DimensionRole::NotCarriedByTheMessage { carried_by, ground } => {
                    Some((*dimension, *carried_by, *ground))
                }
                DimensionRole::Required | DimensionRole::Refused(_) => None,
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

    /// Every dimension that carries one protected datum.
    ///
    /// Set-valued rather than single-valued, and the widening is a type
    /// change the confidential-funding guide's protected-bytes repair
    /// asked for by name (§9.3 part 2, §3.2). A datum can be protected
    /// by more than one term of the target's message, and a signature
    /// answering with one dimension had to choose which of them to
    /// publish — which is how the proof fields came to declare the input
    /// side's anchoring alone while the target's own message covers the
    /// created outputs' proofs as well. Declaring less protection than
    /// the target gives is not a safe error: a later narrowing of the
    /// profile would be argued against the declaration rather than
    /// against the rule.
    ///
    /// # Panics
    ///
    /// Never for a member of [`ProtectedDatum::ALL`]: the constructor
    /// refuses to build a profile whose coverage map is not total, so a
    /// missing entry is unreachable rather than handled.
    #[must_use]
    pub fn carrier(&self, datum: ProtectedDatum) -> &BTreeSet<SighashDimension> {
        self.coverage
            .get(&datum)
            .expect("a selected profile covers every protected datum")
    }

    /// Every protected datum paired with each dimension that carries it.
    ///
    /// Flattened, so a datum with two carriers appears twice. The shape
    /// is what lets a reader ask whether a carrier set *includes* a
    /// dimension without the iterator having decided for them which of
    /// several is the one worth reporting.
    pub fn coverage(&self) -> impl Iterator<Item = (ProtectedDatum, SighashDimension)> + '_ {
        self.coverage.iter().flat_map(|(datum, dimensions)| {
            dimensions.iter().map(move |dimension| (*datum, *dimension))
        })
    }

    /// What one reviewed sighash capability establishes about this
    /// profile.
    ///
    /// The capability is taken rather than assumed, so the answer moves
    /// when the review does: a contract that reviewed the dimensions
    /// would make this [`OwnerProfileDisposition::Established`] with no
    /// edit here, and a contract that lost one would take it back out.
    ///
    /// # Nothing is handed in
    ///
    /// The disposition is recomputed here from two independently
    /// maintained facts — which dimensions this profile requires, and
    /// which the reviewed contract establishes — and there is no
    /// argument, field, or constructor anywhere that lets a caller state
    /// the answer instead. That is the whole reason the review verdict
    /// is a population of the capability rather than a disposition
    /// written down: the wave that reviewed the dimensions could not
    /// declare the profile established even if it wanted to, and the
    /// dimension it failed to exercise names itself in the result.
    #[must_use]
    pub fn assess(&self, capability: &SighashCapability) -> OwnerProfileDisposition {
        let unreviewed = self
            .required()
            .filter(|dimension| !capability.is_reviewed(*dimension))
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
/// The map is set-valued and the totality it argues is set-valued with
/// it: every protected datum lands on a *non-empty* set of dimensions,
/// and every dimension in every such set is one this profile requires.
/// Fourteen of the fifteen data have one carrier and the proof fields
/// have two, which is why totality can no longer be read off a
/// single-valued lookup — a datum whose carrier set had gone empty, or
/// had come to name a refused or a not-message-carried dimension, would
/// be a coverage claim with nothing behind it, and
/// [`profile_coverage_lands_only_on_required_dimensions`] is the check
/// that says so.
///
/// [`profile_coverage_lands_only_on_required_dimensions`]: crate::authorization::profile_coverage_lands_only_on_required_dimensions
///
/// The two refusals are therefore not spare caution. They are the two
/// dimensions the target offers that would each, on their own, undo one
/// of the two totals the argument rests on.
///
/// # Why the required set is seven and not eight
///
/// The tenth dimension the target names,
/// [`SighashDimension::InternalKey`], is neither required nor refused.
/// The source review read the message term by term and found no
/// internal-key term in it at all
/// (`rule:sighash-review:internal-key`), so the dimension is re-typed
/// rather than kept: its protection is recorded as carried by
/// [`SighashDimension::SpentOutputs`], where the output key actually is,
/// and the argument for saying so travels with the dimension at
/// [`OutsideMessageGround`] rather than being deleted along with the
/// requirement.
///
/// The coverage map is what makes that re-typing free of consequence
/// rather than a loss: it assigns no protected datum to the internal
/// key, and it did not before the re-typing either. So no protected
/// datum moves, no datum lands on a dimension the profile does not
/// require, and the coverage argument below is unchanged in content.
/// The requirement that left had been protecting nothing the argument
/// itself names — the signature of a requirement stated by analogy to
/// BIP-341 rather than derived from this target's message.
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
                Dimension::InternalKey => DimensionRole::NotCarriedByTheMessage {
                    carried_by: Dimension::SpentOutputs,
                    ground: OutsideMessageGround::ComposedThroughTheControlBlockCheck,
                },
                Dimension::AllOutputs
                | Dimension::AllInputs
                | Dimension::Issuance
                | Dimension::Version
                | Dimension::LockTime
                | Dimension::TapleafHash
                | Dimension::SpentOutputs => DimensionRole::Required,
                _ => DimensionRole::Refused(DimensionRefusal::Unclassified),
            };

            (*dimension, role)
        })
        .collect();

    let coverage = ProtectedDatum::ALL
        .iter()
        .map(|datum| {
            let dimensions: BTreeSet<SighashDimension> = match datum {
                // Which receipts are consumed is the input set itself,
                // and what each of them was worth is a property of the
                // outputs being spent rather than of this transaction's
                // own fields.
                ProtectedDatum::ReceiptInputs | ProtectedDatum::SponsorInputs => {
                    BTreeSet::from([Dimension::AllInputs])
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
                | ProtectedDatum::RepresentationPlan => BTreeSet::from([Dimension::AllOutputs]),
                // Two carriers, and the second is the confidential-funding
                // guide's protected-bytes repair (§9.3 part 2).
                //
                // The spent outputs carry the input side's own value and
                // asset fields, which is where the proofs a confidential
                // transfer relies on are anchored. That half is unchanged
                // and is owed afterwards exactly as it was met before.
                //
                // The created outputs carry the other half, and the
                // target's own message is why: under this profile's
                // all-outputs type the stream writes both the serialized
                // output list and the hash of the output-witness vector,
                // and the output-witness vector is where a created
                // output's range proof and surjection proof live. So a
                // transfer's own created proofs are data the owner's
                // signature commits to, and a coverage map naming the
                // spent outputs alone declared less protection than the
                // target actually gives.
                ProtectedDatum::ProofFields => {
                    BTreeSet::from([Dimension::SpentOutputs, Dimension::AllOutputs])
                }
                ProtectedDatum::TransactionVersion => BTreeSet::from([Dimension::Version]),
                ProtectedDatum::LockTime => BTreeSet::from([Dimension::LockTime]),
                ProtectedDatum::IssuanceFields => BTreeSet::from([Dimension::Issuance]),
                ProtectedDatum::ScriptPathFields => BTreeSet::from([Dimension::TapleafHash]),
            };

            (*datum, dimensions)
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

/// Whether every protected datum's carriers are dimensions the profile
/// requires.
///
/// The coverage argument's totality, recomputed rather than asserted.
/// Single-valued coverage made half of this true by construction: a
/// lookup either answered or panicked, so the only way to be wrong was
/// to name a dimension the profile did not require. Set-valued coverage
/// adds a second way — an empty set — and both are checked here.
///
/// A function rather than a test for the same reason its sibling is one:
/// a consumer deciding whether to rely on the profile is entitled to ask
/// whether the profile's own argument holds, and the honest answer to
/// "does this coverage map still argue what it claims" is a value rather
/// than a build step somebody else ran.
#[must_use]
pub fn profile_coverage_lands_only_on_required_dimensions(profile: &OwnerSighashProfile) -> bool {
    ProtectedDatum::ALL.iter().all(|datum| {
        let carriers = profile.carrier(*datum);
        !carriers.is_empty()
            && carriers
                .iter()
                .all(|dimension| profile.role(*dimension) == Some(DimensionRole::Required))
    })
}
