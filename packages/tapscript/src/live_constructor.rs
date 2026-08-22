//! The candidate live-receipt constructor (Guide-13 §7).
//!
//! # What this module binds, and what it refuses to be
//!
//! [`derive_live_receipt_constructor`] assembles one
//! [`StaticLiveReceiptConstructor`]: the reviewed contract it is built
//! against, the canonical owner metadata, the live class and owner
//! closures the compiler's validated plan published, the selected
//! representation plan, the static transfer leaf set, the inherited
//! internal-key and key-path policies, and the candidate lifecycle.
//! [`ConstructorBinding`] is the §7.1 list itself, as data, so the one
//! element this wave does not bind — the candidate ABI schema, which is
//! §12's — is visible rather than absent.
//!
//! §7.1's other sentence is a property of the field list: value stays in
//! the target value field and is not duplicated into metadata. No field
//! here is an amount, and none can be, because the only value-side thing
//! the constructor holds is the compiler's
//! [`LiveTransferValueProjection`], which publishes the conservation
//! relation's shape and holds no amount either.
//!
//! # The first secret-adjacent surface of the batch
//!
//! §7.2 says [`OwnerKey`] is a public authorization identity containing
//! no private key material, and §1.10 forbids Guide 13 introducing a
//! production interface for owner private keys at all. Neither is left
//! to a comment.
//!
//! The only bytes any function here accepts are an owner's *public* key,
//! and they are refused unless the encoding class naming them sits in
//! the target's own key domain. That check is not decoration:
//! [`EncodingClass::EcScalar`] — the shape a private key takes — is
//! exactly as wide as the approved public encoding and carries exactly
//! the same opaque payload interpretation, so width and byte-level
//! form cannot tell the two apart. The domain can, and
//! [`OwnerKeyRejection::NotAKeyEncoding`] is what it says when it does.
//!
//! # No key-path escape is a property, not a promise
//!
//! §7.5 admits no owner internal key, no operator key, no release key,
//! no generated-and-discarded key, no caller-selected key, and no
//! accepted key-path escape. Three separate facts hold that, and none of
//! them is a sentence in a comment:
//!
//! - the policies are the compact-ASH constructor's own
//!   [`InternalKeyPolicy`] and [`KeyPathPolicy`], inherited rather than
//!   re-minted as §7.5 directs, and each has one variant — so a
//!   spendable internal key and an accepted escape are unrepresentable;
//! - [`LiveSpendingRoute`] has one variant, and
//!   [`StaticLiveReceiptConstructor::spending_routes`] derives the whole
//!   route census from the leaf set, so a route that is not a script
//!   path has nothing to be built out of;
//! - construction refuses an empty transfer leaf set
//!   ([`LiveConstructorRefusal::KeyPathWouldBeTheOnlySpendingRoute`]).
//!   That is the one with teeth. A taproot output carrying no script
//!   leaf can be spent only through its key path, so an empty leaf set
//!   *is* the escape — and refusing it is why
//!   [`key_path_closure`] answers [`KeyPathClosure::Closed`] for every
//!   constructor that exists, rather than for every constructor
//!   somebody remembered to check.
//!
//! # The live class is structural (§7.3)
//!
//! §7.3 asks for four things, and three of them are properties of the
//! type list rather than of any check.
//!
//! The live and time-locked constructors are distinct because this type
//! exists and no other constructor does: there is no class field to set,
//! no constructor argument naming a family, and no way to build this
//! value for anything but the family the compiler's plan named. A
//! live-transfer leaf cannot spend a time-locked constructor because
//! [`LiveTransferLeafRole`] is not [`crate::bundle::LeafRole`] and is
//! not any other operation's leaf either — the leaf sets of two
//! operations are two types, so a leaf of one has no way of reaching the
//! other's constructor. And a time-locked output cannot satisfy a live
//! destination role because the class closure this constructor carries
//! forbids that family on both sides, which is a census the compiler
//! derived by subtraction rather than a list anybody transcribed.
//!
//! The fourth is a property with a test: class is not inferred from
//! target position, value representation, amount, or owner. Position and
//! amount are not inputs to [`derive_live_receipt_constructor`] at all —
//! there is no parameter that could carry one. The other two are inputs,
//! and varying either leaves the class exactly where the plan put it.
//!
//! # Where the facts come from
//!
//! The §5 contract is not restated here. The live class, the owner
//! family, the value relation, the admitted representations, and the
//! lifecycle exits arrive as the compiler's own validated projections
//! and are stored as those projections, so a reader asking what the
//! class is reads the answer from the component that derived and
//! validated it. What this module adds is the *joins* between them that
//! only a constructor needs: that the family whose owners authorize is
//! the family the constructor builds for, that the selected
//! representation is one the policy admits, and that the leaf set is
//! exactly the one the selected representation and the admitted shapes
//! call for.
//!
//! The authorization-facing choices are likewise cited rather than
//! rederived. The owner encoding is the one
//! [`owner_key_encoding_closure`] read off the reviewed contract, and
//! the closure is retained on the constructor because the obligation it
//! carries — whether a pattern must authenticate the key encoding
//! itself — is what the recognition fragments of §10.2 owe next.
//!
//! # No digest is minted
//!
//! §1.13 admits a digest only once a real consumer of one exists, and
//! names `LiveReceiptConstructorHash` among the identities Guide 13
//! mints none of. The constructor's identity is the typed value: two
//! constructors are the same one when their fields agree, which is what
//! makes §7.6's determinism checkable without hashing anything.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use compiler::live_transfer_plan::{
    LiveTransferClassProjection, LiveTransferLifecycleClosure, LiveTransferOwnerProjection,
    LiveTransferRepresentationPlan, LiveTransferValueProjection,
    ValidatedLiveTransferOperationPlan,
};
use target_elements::{
    CanonicalEncodingRule, EncodingClass, EncodingDomain, LeafVersion, PayloadWidth,
    ReviewedElementsTapscriptDefinition, TargetContractVersion,
};

use crate::authorization::{OwnerKeyEncodingClosure, OwnerKeyNegative, owner_key_encoding_closure};
use crate::bundle::{ConstructorAssumption, InternalKeyPolicy, KeyPathPolicy};
use crate::capability::census_enum;
use crate::live_shape::{LiveTransferShape, LiveTransferShapeSet};

// --- Canonical owner metadata (§7.2) ----------------------------------

/// Why offered owner metadata is not the canonical encoding (§7.2).
///
/// §7.2 lists seven things the encoding rejects, and they do not divide
/// evenly into seven variants, because three of them are the same
/// structural fact seen from different sides and one of them is not
/// reachable through this type at all.
///
/// - *omitted owner* and *wrong width* are separate variants, because
///   the target itself distinguishes an absent field from a short one:
///   [`PayloadWidth::Absent`] is a form in its own right, not a
///   zero-length payload;
/// - *unknown key type* and *alternate encoding of the same key* are
///   [`Self::NotAKeyEncoding`] and
///   [`Self::AlternateEncodingOfApprovedKey`], split by which of the two
///   mistakes was made — offering something that is not a public key,
///   or offering the right key under the wrong one of the target's two
///   key encodings;
/// - *malformed owner key* is split by what can actually be
///   established here. An offering that fails the encoding is refused;
///   an offering that passes it and is nonetheless not a curve point is
///   a [`OwnerKeyResidual::CurvePointMembership`], because this crate
///   holds no curve arithmetic and a refusal it could not compute would
///   be a claim rather than a check;
/// - *owner bytes not committed by the constructor* has no variant
///   because it has no value to refuse: the owner is a parameter of
///   [`derive_live_receipt_constructor`], so bytes that reach an output
///   any other way were never this constructor's owner. The mutation
///   case that reaches past the constructor to substitute them is
///   [`ConstructorMutationCaseId::OwnerBytesReplacedInTheLinkedOutput`],
///   and it carries residuals rather than a refusal;
/// - *key types admitted only through the forward-compatibility path*
///   likewise has no variant, and for the sharper reason that
///   [`EncodingClass`] is a census of *reviewed* classes: a type the
///   target would admit only by not recognizing it is not a member, so
///   it cannot be offered here. What it does need is for the program to
///   refuse it too, and the obligation to do that travels on the
///   retained [`OwnerKeyEncodingClosure`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerKeyRejection {
    /// No owner bytes were offered at all.
    OwnerOmitted,
    /// The offered bytes are not the approved encoding's exact width.
    WrongWidth {
        /// How many bytes were offered.
        offered: usize,
        /// How many the approved encoding fixes.
        required: usize,
    },
    /// The offered class is not a public-key encoding.
    ///
    /// The §1.10 guard. Every encoding the target names sits in one
    /// domain, and the domain is the only thing that separates a public
    /// key from the scalar a private key is: the two are the same width
    /// and the same opaque payload, so nothing about the bytes would
    /// distinguish them.
    NotAKeyEncoding {
        /// The class offered.
        offered: EncodingClass,
        /// The domain it belongs to.
        domain: EncodingDomain,
    },
    /// The offered class is a key encoding, and not the approved one.
    ///
    /// §7.2's *alternate encoding of the same key*: the target names
    /// more than one way to write a public key, and one owner written
    /// two ways would be two owners as far as any byte comparison is
    /// concerned.
    AlternateEncodingOfApprovedKey {
        /// The class offered.
        offered: EncodingClass,
        /// The class the reviewed contract approves.
        approved: EncodingClass,
    },
    /// The approved encoding does not admit exactly one byte string per
    /// key.
    ///
    /// A guard on the *target*, not on the caller: it fires only if the
    /// contract's approved owner-key encoding ever stops being uniquely
    /// canonical, at which point §7.2's "one canonical encoding" would
    /// no longer exist and the alternate-encoding rejection above would
    /// be reachable inside the approved class. Refusing then is the
    /// fail-closed answer.
    NonUniqueCanonicalForm {
        /// The approved class.
        approved: EncodingClass,
        /// The canonicality rule it carries.
        rule: CanonicalEncodingRule,
    },
    /// The approved encoding fixes no single width.
    ///
    /// The other target-side guard. "Wrong width" is only meaningful
    /// against an exact width, so an approved class that became absent
    /// or bounded would leave the width rejection with nothing to
    /// compare against rather than with a wider tolerance.
    UnfixedApprovedWidth {
        /// The approved class.
        approved: EncodingClass,
        /// The width it admits.
        width: PayloadWidth,
    },
}

/// Canonical owner metadata: a public authorization identity (§7.2).
///
/// # What a value of this type is
///
/// The bytes of one owner's public key, together with the target
/// encoding class they are written in — which is always the class the
/// reviewed contract approves, because [`Self::new`] admits no other.
///
/// # What a value of this type is not
///
/// It is not, and cannot be made to hold, private key material. There is
/// no constructor taking a scalar, a seed, a signing nonce, or an
/// opening; the single constructor takes bytes and the class naming
/// their encoding, and refuses every class outside the target's public
/// key domain. §1.10 defers any first-party production interface for
/// owner private keys to a separate ADR-015 design, and the way to keep
/// that promise in a type is to give the type no way to accept one.
///
/// It is also not a claim that the key is a curve point, that anybody
/// holds the corresponding secret, or that the key belongs to the party
/// the caller had in mind. [`OwnerKeyResidual`] names all three, and
/// each says which of Wave 3's owner-key negatives is where a run has to
/// show it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnerKey {
    encoding: EncodingClass,
    bytes: Vec<u8>,
}

impl OwnerKey {
    /// Canonical owner metadata from offered bytes, or the reason it is
    /// not canonical.
    ///
    /// The offered class is a parameter rather than a constant even
    /// though exactly one is approved, and that is deliberate: a caller
    /// who could not name the wrong class could not be refused for
    /// naming one, and §7.2's rejections would be code no request could
    /// reach. The approved class is not transcribed either — it is read
    /// from the closure Wave 3 derived from the reviewed contract, so a
    /// target that approved a different encoding moves this check
    /// without an edit here.
    ///
    /// # Errors
    ///
    /// [`OwnerKeyRejection::OwnerOmitted`] for empty bytes;
    /// [`OwnerKeyRejection::NotAKeyEncoding`] for a class outside the
    /// target's key domain;
    /// [`OwnerKeyRejection::AlternateEncodingOfApprovedKey`] for a key
    /// class other than the approved one;
    /// [`OwnerKeyRejection::NonUniqueCanonicalForm`] and
    /// [`OwnerKeyRejection::UnfixedApprovedWidth`] when the approved
    /// class itself no longer describes one canonical fixed-width form;
    /// and [`OwnerKeyRejection::WrongWidth`] for the wrong number of
    /// bytes.
    pub fn new(
        closure: &OwnerKeyEncodingClosure,
        encoding: EncodingClass,
        bytes: Vec<u8>,
    ) -> Result<Self, OwnerKeyRejection> {
        // Emptiness first. An empty offering is the *absent* form, which
        // §7.2 lists separately from a wrong width and which the target
        // itself represents separately; reporting it as a width mismatch
        // would merge "no owner" with "this owner, mistyped".
        if bytes.is_empty() {
            return Err(OwnerKeyRejection::OwnerOmitted);
        }

        let shape = encoding.v1_shape();
        // The domain gate before the class gate: every non-key class
        // also fails the class comparison below, and of the two answers
        // this is the one that names what went wrong. It is the only
        // check that separates the approved public key from the scalar
        // a private key is written as.
        if shape.domain() != EncodingDomain::Key {
            return Err(OwnerKeyRejection::NotAKeyEncoding {
                offered: encoding,
                domain: shape.domain(),
            });
        }

        let approved = closure.approved();
        if encoding != approved {
            return Err(OwnerKeyRejection::AlternateEncodingOfApprovedKey {
                offered: encoding,
                approved,
            });
        }

        let approved_shape = approved.v1_shape();
        if approved_shape.canonicality() != CanonicalEncodingRule::Unique {
            return Err(OwnerKeyRejection::NonUniqueCanonicalForm {
                approved,
                rule: approved_shape.canonicality(),
            });
        }

        let PayloadWidth::Exact(required) = approved_shape.payload() else {
            return Err(OwnerKeyRejection::UnfixedApprovedWidth {
                approved,
                width: approved_shape.payload(),
            });
        };
        if bytes.len() != required.get() {
            return Err(OwnerKeyRejection::WrongWidth {
                offered: bytes.len(),
                required: required.get(),
            });
        }

        Ok(Self { encoding, bytes })
    }

    /// The owner's public key bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The target encoding class the bytes are written in.
    #[must_use]
    pub const fn encoding(&self) -> EncodingClass {
        self.encoding
    }

    /// How many bytes the metadata occupies.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.bytes.len()
    }
}

census_enum! {
    /// What accepting owner metadata establishes (§7.2).
    ///
    /// Four facts, each one a comparison this crate actually performs
    /// against the reviewed contract's own encoding registry. The list
    /// is published beside [`OwnerKeyResidual`] so that what an accepted
    /// [`OwnerKey`] means is a census rather than an impression.
    pub enum OwnerKeyEstablishment {
        /// The class sits in the target's public-key domain.
        PublicKeyDomain,
        /// The class is the one the reviewed contract approves.
        ApprovedEncodingClass,
        /// That class admits exactly one byte string per key.
        UniqueCanonicalForm,
        /// The offering is that class's exact fixed width.
        ExactApprovedWidth,
    }
}

census_enum! {
    /// What accepting owner metadata does not establish (§7.2, §1.8).
    ///
    /// A residual is not a caveat. Each member names something a
    /// target-native run has to show, and [`Self::negative`] says which
    /// of Wave 3's owner-key negatives is the run that shows it — so a
    /// reader can go from "the encoding passed" to "and here is what
    /// still has to fail" without leaving the type.
    pub enum OwnerKeyResidual {
        /// The bytes are a point on the target's curve.
        ///
        /// Not computable here: the adapter holds no curve arithmetic
        /// and takes no dependency that would give it any, so a refusal
        /// would be a claim rather than a check.
        CurvePointMembership,
        /// The key is the one the intended party controls.
        KeyIsTheIntendedPartys,
        /// Somebody holds the secret this key corresponds to.
        ///
        /// Nothing about a public encoding could establish it, and
        /// nothing about this candidate needs to: what a spend has to
        /// show is a signature that verifies, which is a different
        /// statement made by a different component.
        OwnerHoldsTheCorrespondingSecret,
    }
}

impl OwnerKeyResidual {
    /// The owner-key negative whose run discharges this residual.
    ///
    /// `None` for [`Self::OwnerHoldsTheCorrespondingSecret`]: Wave 3's
    /// negatives probe the *encoding gate* and the *signature check*,
    /// and secret-holding is the thing the signature check is for rather
    /// than a case it enumerates. Returning some nearby negative would
    /// make the map total at the cost of making it wrong.
    #[must_use]
    pub const fn negative(self) -> Option<OwnerKeyNegative> {
        match self {
            Self::CurvePointMembership => Some(OwnerKeyNegative::MalformedApprovedKey),
            Self::KeyIsTheIntendedPartys => Some(OwnerKeyNegative::ApprovedKeyOfAnotherOwner),
            Self::OwnerHoldsTheCorrespondingSecret => None,
        }
    }
}

// --- The static transfer leaf set (§7.1, §10.3, §11.3) ----------------

/// Which typed role one live-transfer program plays (§10.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveProgramRole {
    /// The single program spending input 0, which performs the
    /// transaction-global checks.
    Coordinator,
    /// The program every nonzero receipt position spends.
    Member,
}

/// The identity of one leaf of the static transfer program set.
///
/// The identity is exactly what the program depends on, which is what
/// makes leaf sharing a checkable claim later rather than an assumption
/// now: a coordinator program is a function of the whole shape because
/// §10.3 has it authenticate every count, and a member program is a
/// function of the receipt-input count alone because that is the range
/// it bounds.
///
/// # Why the representation is part of the identity
///
/// §11.3 admits a shared explicit/private coordinator only under one
/// complete typed proof, and a shared member leaf only under four named
/// conditions including target-native vectors through the shared leaf.
/// None of those proofs exists yet. Keying the leaf by representation is
/// therefore the fail-closed default: two representations get two leaves
/// until somebody establishes they may get one, rather than sharing by
/// default and being narrowed if anybody notices. It also forecloses
/// §11.3's other sentence structurally — no attacker-selected in-script
/// dispatch can pick a semantic representation when a constructor holds
/// the leaves of one representation only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveTransferLeafRole {
    /// The coordinator leaf of one exact shape.
    Coordinator {
        /// The representation whose obligations the program carries.
        representation: LiveTransferRepresentationPlan,
        /// The shape whose counts the program authenticates.
        shape: LiveTransferShape,
    },
    /// The member leaf serving every shape of one receipt-input count.
    Member {
        /// The representation whose obligations the program carries.
        representation: LiveTransferRepresentationPlan,
        /// The receipt-input count whose member range the program
        /// bounds.
        receipt_inputs: u8,
    },
}

impl LiveTransferLeafRole {
    /// Which program role this leaf carries.
    #[must_use]
    pub const fn program_role(self) -> LiveProgramRole {
        match self {
            Self::Coordinator { .. } => LiveProgramRole::Coordinator,
            Self::Member { .. } => LiveProgramRole::Member,
        }
    }

    /// The representation this leaf belongs to.
    #[must_use]
    pub const fn representation(self) -> LiveTransferRepresentationPlan {
        match self {
            Self::Coordinator { representation, .. } | Self::Member { representation, .. } => {
                representation
            }
        }
    }
}

/// The leaf set one representation and one shape set call for (§7.1).
///
/// Every admitted shape contributes its coordinator, and every shape
/// with more than one receipt input contributes the member leaf of its
/// count.
///
/// # Why the one-to-one shape contributes no member leaf
///
/// §10.3 makes member leaves valid only at *nonzero* receipt positions,
/// and a one-input transfer has no nonzero receipt position: input 0 is
/// the coordinator and there is nothing after it. A member leaf built
/// for that shape would be a leaf no spend could ever reach — dead
/// weight in the taptree, extra control-block depth for every other
/// leaf, and one more thing for §11.4's duplicate-free tree input to
/// carry. Compact ASH never meets the case because its batch minimum is
/// two; the live transfer meets it in its very first shape.
#[must_use]
pub fn static_transfer_leaf_set(
    representation: LiveTransferRepresentationPlan,
    shapes: &LiveTransferShapeSet,
) -> BTreeSet<LiveTransferLeafRole> {
    let mut leaves = BTreeSet::new();

    for shape in shapes.shapes() {
        leaves.insert(LiveTransferLeafRole::Coordinator {
            representation,
            shape,
        });
        if shape.receipt_inputs() > 1 {
            leaves.insert(LiveTransferLeafRole::Member {
                representation,
                receipt_inputs: shape.receipt_inputs(),
            });
        }
    }

    leaves
}

// --- Spending routes and the key-path closure (§7.5) ------------------

/// One route by which this constructor's output can be spent.
///
/// One variant. A key-path route is not refused here, or filtered out
/// here, or documented as unsupported here: it has no value, so there is
/// nothing for a later edit to admit by relaxing a check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveSpendingRoute {
    /// A script-path spend of one leaf of the transfer program set.
    ScriptPath {
        /// The leaf spent.
        leaf: LiveTransferLeafRole,
    },
}

/// Whether a constructor leaves any key-path escape (§7.5).
///
/// Computed from the leaf set rather than declared, so it answers for
/// the constructor in hand rather than for the design. There is exactly
/// one way it can open, and construction refuses that one, which is what
/// makes the closed answer worth taking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyPathClosure {
    /// Every spend goes through one of the script paths, and there is
    /// at least one.
    Closed {
        /// How many script paths the constructor carries.
        script_paths: NonZeroUsize,
    },
    /// The constructor carries no script leaf, so its output could be
    /// spent only through the key path.
    OpenForLackOfScriptPath,
}

impl KeyPathClosure {
    /// Whether the closure holds.
    #[must_use]
    pub const fn is_closed(self) -> bool {
        matches!(self, Self::Closed { .. })
    }
}

/// The key-path closure of one constructor (§7.5).
///
/// A free function rather than a method, for the same reason
/// [`crate::authorization::profile_classifies_every_offered_dimension`]
/// is one: a consumer deciding whether to rely on this constructor is
/// entitled to ask the question itself, over a value it holds, rather
/// than to read that somebody asked it once.
#[must_use]
pub fn key_path_closure(constructor: &StaticLiveReceiptConstructor) -> KeyPathClosure {
    NonZeroUsize::new(constructor.leaves().count())
        .map_or(KeyPathClosure::OpenForLackOfScriptPath, |script_paths| {
            KeyPathClosure::Closed { script_paths }
        })
}

// --- The candidate lifecycle (§7.4, §17) ------------------------------

/// The lifecycle state of a Phase-5 live-receipt candidate (§7.4).
///
/// Structurally incomplete: [`Self::outstanding`] is a [`NonZeroUsize`],
/// so a candidate whose lifecycle is complete has no representation
/// here, and construction refuses a plan claiming one. That is what
/// makes "this is a candidate" a fact about the type rather than a
/// sentence a later edit could drop.
///
/// §7.4's other requirement — that the candidate contain only
/// implemented transfer leaves, and no spendable placeholder for burn,
/// redemption, or normalization — needs no field at all.
/// [`LiveTransferLeafRole`] has a coordinator and a member and nothing
/// else, so a burn leaf is not something this constructor refuses to
/// hold; it is something no value can express.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateTransferLifecycle {
    closure: LiveTransferLifecycleClosure,
    outstanding: NonZeroUsize,
}

impl CandidateTransferLifecycle {
    /// The compiler's own lifecycle-exit closure.
    ///
    /// Handed back whole rather than re-published field by field: the
    /// exits are the compiler's to name, and a consumer that wants to
    /// know which ones are outstanding asks the component that derived
    /// them.
    #[must_use]
    pub const fn closure(&self) -> &LiveTransferLifecycleClosure {
        &self.closure
    }

    /// How many lifecycle exits are outstanding, which is never zero.
    #[must_use]
    pub const fn outstanding(&self) -> NonZeroUsize {
        self.outstanding
    }
}

// --- What §7.1 binds --------------------------------------------------

census_enum! {
    /// One element §7.1 has the candidate constructor bind.
    ///
    /// The guide's own list, in its own order, so that the one element
    /// this wave does not bind is visible as a member with a status
    /// rather than as an omission a reader would have to notice.
    pub enum ConstructorBinding {
        /// The exact explicit protocol asset.
        ExplicitAsset,
        /// The live receipt object family.
        LiveReceiptFamily,
        /// Canonical owner metadata.
        CanonicalOwnerMetadata,
        /// The live class.
        LiveClass,
        /// The selected representation plan.
        SelectedRepresentationPlan,
        /// The static transfer program set.
        StaticTransferProgramSet,
        /// The target contract revision.
        TargetContractRevision,
        /// The leaf version.
        LeafVersion,
        /// The internal-key policy.
        InternalKeyPolicy,
        /// The candidate ABI schema.
        CandidateAbiSchema,
    }
}

/// Whether one §7.1 binding is carried by the constructor yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BindingStatus {
    /// The constructor carries it, and an accessor reports it.
    Bound,
    /// The binding waits on a component this wave does not build.
    Outstanding {
        /// What has to exist first.
        pending: PendingBinding,
    },
}

/// What an outstanding §7.1 binding is waiting for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PendingBinding {
    /// The candidate transaction ABI of §12, which is derived from the
    /// linked bundle and does not exist yet.
    ///
    /// Binding a schema here would mean inventing one, and §1.12 keeps
    /// `CandidateLiveTransferAbi` a thing Guide 13 constructs *later*
    /// rather than a field this constructor reserves.
    CandidateTransactionAbi,
}

impl ConstructorBinding {
    /// Whether this constructor binds the element yet.
    #[must_use]
    pub const fn status(self) -> BindingStatus {
        match self {
            Self::CandidateAbiSchema => BindingStatus::Outstanding {
                pending: PendingBinding::CandidateTransactionAbi,
            },
            Self::ExplicitAsset
            | Self::LiveReceiptFamily
            | Self::CanonicalOwnerMetadata
            | Self::LiveClass
            | Self::SelectedRepresentationPlan
            | Self::StaticTransferProgramSet
            | Self::TargetContractRevision
            | Self::LeafVersion
            | Self::InternalKeyPolicy => BindingStatus::Bound,
        }
    }
}

// --- Refusals ---------------------------------------------------------

/// Why a candidate live-receipt constructor was not derived.
///
/// Construction failure, never target rejection (§1.11): every variant
/// names something wrong with what this constructor was asked to build,
/// and none of them is a statement about what a target would do with the
/// result.
/// Owner metadata has no variant here, and its absence is deliberate:
/// the constructor takes an [`OwnerKey`], which is a value that already
/// passed §7.2's encoding, so a refusal for bad owner bytes would be one
/// nothing could produce. [`OwnerKeyRejection`] is where those refusals
/// live, and a refusal nobody can reach is exactly the decoration this
/// crate keeps out of its censuses.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LiveConstructorRefusal {
    /// The plan's lifecycle is complete, so it is not this type's
    /// subject.
    ///
    /// A release-complete plan needs the final constructor §1.12 says
    /// Guide 13 must not construct.
    PlanClaimsReleaseComplete,
    /// The plan implements no lifecycle exit, so there is no operation
    /// for this constructor to be the constructor of.
    PlanImplementsNoExit,
    /// One exit is both implemented and outstanding.
    ///
    /// The two censuses are meant to partition the required exits, and
    /// a constructor whose lifecycle claimed an exit twice would be
    /// reporting itself both complete and incomplete for it.
    LifecycleExitBothImplementedAndOutstanding,
    /// The plan's protocol family is not among the families it admits.
    ProtocolFamilyNotAdmitted,
    /// The plan forbids its own protocol family.
    ProtocolFamilyForbiddenOnItsOwnSides,
    /// The plan's protocol and sponsor families are the same one.
    ///
    /// §1.9's sponsor isolation rests on the two regions being
    /// distinguishable by declared family, and a plan in which they
    /// coincide has nothing to isolate.
    ProtocolAndSponsorFamiliesCoincide,
    /// The family whose owners must authorize is not the family the
    /// constructor builds for.
    ///
    /// A join between two projections this constructor reads, not a
    /// re-derivation of either: the constructor commits an owner, and
    /// the owner it commits has to be an owner of the thing it is
    /// building.
    OwnerFamilyIsNotTheProtocolFamily,
    /// The selected representation is not one the plan admits.
    RepresentationNotAdmitted {
        /// The representation selected.
        selected: LiveTransferRepresentationPlan,
    },
    /// The constructor would carry no script leaf at all.
    ///
    /// §7.5's key-path rule, enforced where it can bite. A taproot
    /// output with no script path can be spent only through its key
    /// path, so an empty transfer leaf set is not a narrow candidate —
    /// it is the accepted escape §7.5 admits none of, arrived at by
    /// omission instead of by declaration.
    KeyPathWouldBeTheOnlySpendingRoute,
    /// A leaf belongs to a representation other than the selected one.
    LeafOfAnotherRepresentation {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// The representation the constructor selected.
        selected: LiveTransferRepresentationPlan,
    },
    /// The leaf set is missing leaves the admitted shapes need.
    ///
    /// A shape whose coordinator or member leaf is absent is a shape the
    /// candidate advertises and cannot spend.
    LeafSetIncomplete {
        /// The leaves the shape set calls for and the offering lacks.
        missing: BTreeSet<LiveTransferLeafRole>,
    },
    /// The leaf set carries leaves no admitted shape reaches.
    ///
    /// An unreachable leaf is not spare capacity: it enlarges the
    /// taptree and every other leaf's control block, and it advertises a
    /// spending route the candidate does not claim to support.
    LeafServesNoAdmittedShape {
        /// The leaves no admitted shape calls for.
        orphans: BTreeSet<LiveTransferLeafRole>,
    },
}

// --- The constructor --------------------------------------------------

/// The static candidate live-receipt constructor (§7).
///
/// Every field is private and there is no public constructor, no
/// `Default`, and no builder: the sole route to a value is
/// [`derive_live_receipt_constructor`], which takes the reviewed
/// contract, the compiler's validated plan, one owner, one
/// representation, one shape set, and one leaf set, and checks the joins
/// between them. A constructor assembled from arbitrary fields would be
/// a request rather than a derivation, and nothing downstream could tell
/// the two apart once they shared a type.
///
/// # Its identity is the value
///
/// §1.13 mints no `LiveReceiptConstructorHash`, so equality is the
/// identity: two constructors are the same constructor when their fields
/// agree. That is what makes §7.6's determinism a property with a test
/// rather than a claim about bytes nobody has produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticLiveReceiptConstructor {
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    owner: OwnerKey,
    owner_encoding: OwnerKeyEncodingClosure,
    class: LiveTransferClassProjection,
    owner_family: LiveTransferOwnerProjection,
    value: LiveTransferValueProjection,
    representation: LiveTransferRepresentationPlan,
    shapes: LiveTransferShapeSet,
    leaves: BTreeSet<LiveTransferLeafRole>,
    internal_key: InternalKeyPolicy,
    key_path: KeyPathPolicy,
    assumptions: BTreeSet<ConstructorAssumption>,
    lifecycle: CandidateTransferLifecycle,
}

impl StaticLiveReceiptConstructor {
    /// The reviewed contract revision this constructor is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every emitted leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The canonical owner metadata this constructor commits.
    #[must_use]
    pub const fn owner(&self) -> &OwnerKey {
        &self.owner
    }

    /// The owner-key encoding closure the metadata was checked against.
    ///
    /// Retained rather than discarded because the obligation it carries
    /// is the one §10.2's recognition fragments owe next: a target that
    /// answers for an unrecognized key without verifying makes the
    /// encoding the program's business, and this is where a later wave
    /// reads that it is.
    #[must_use]
    pub const fn owner_encoding(&self) -> &OwnerKeyEncodingClosure {
        &self.owner_encoding
    }

    /// The live-class closure the compiler validated.
    #[must_use]
    pub const fn class(&self) -> &LiveTransferClassProjection {
        &self.class
    }

    /// The owner-authorization closure the compiler validated.
    #[must_use]
    pub const fn owner_family(&self) -> &LiveTransferOwnerProjection {
        &self.owner_family
    }

    /// The value and partition contract the compiler validated.
    ///
    /// The whole of §7.1's "value remains in the target value field":
    /// what the constructor holds is the conservation relation's shape,
    /// which names an asset and a flow and no amount at all.
    #[must_use]
    pub const fn value(&self) -> &LiveTransferValueProjection {
        &self.value
    }

    /// The representation plan this constructor is built for.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// The candidate shape set.
    #[must_use]
    pub const fn shapes(&self) -> &LiveTransferShapeSet {
        &self.shapes
    }

    /// Every leaf of the static transfer program set, in canonical
    /// order.
    pub fn leaves(&self) -> impl Iterator<Item = LiveTransferLeafRole> + '_ {
        self.leaves.iter().copied()
    }

    /// The two leaves one admitted shape spends through.
    ///
    /// The member leaf is `None` for the one-to-one shape, which has no
    /// nonzero receipt position for a member to be valid at, and `None`
    /// for a shape this candidate does not admit. The two are told apart
    /// by the coordinator: a shape with no coordinator is a shape this
    /// constructor cannot spend at all.
    #[must_use]
    pub fn shape_leaves(
        &self,
        shape: LiveTransferShape,
    ) -> Option<(LiveTransferLeafRole, Option<LiveTransferLeafRole>)> {
        let coordinator = LiveTransferLeafRole::Coordinator {
            representation: self.representation,
            shape,
        };
        if !self.leaves.contains(&coordinator) {
            return None;
        }

        let member = LiveTransferLeafRole::Member {
            representation: self.representation,
            receipt_inputs: shape.receipt_inputs(),
        };
        Some((coordinator, self.leaves.contains(&member).then_some(member)))
    }

    /// Every route by which this constructor's output can be spent.
    ///
    /// Derived from the leaf set, so the census is the leaf set: there
    /// is no separate list of routes for the two to disagree about, and
    /// no route that is not a script path.
    pub fn spending_routes(&self) -> impl Iterator<Item = LiveSpendingRoute> + '_ {
        self.leaves()
            .map(|leaf| LiveSpendingRoute::ScriptPath { leaf })
    }

    /// The inherited internal-key policy (§7.5).
    #[must_use]
    pub const fn internal_key(&self) -> InternalKeyPolicy {
        self.internal_key
    }

    /// The inherited key-path policy (§7.5).
    #[must_use]
    pub const fn key_path(&self) -> KeyPathPolicy {
        self.key_path
    }

    /// Every assumption the constructor rests on and does not establish.
    #[must_use]
    pub const fn assumptions(&self) -> &BTreeSet<ConstructorAssumption> {
        &self.assumptions
    }

    /// The candidate lifecycle state.
    #[must_use]
    pub const fn lifecycle(&self) -> &CandidateTransferLifecycle {
        &self.lifecycle
    }
}

/// Derive the candidate live-receipt constructor (§7.6).
///
/// Each output constructor derives from the destination owner, the live
/// class, the selected representation, the linked static transfer
/// program set, and the target constants — which is exactly this
/// parameter list, with the class and the target constants arriving
/// through the components that own them rather than as caller-supplied
/// values.
///
/// §7.6's other sentence is a property of the same list: the request
/// cannot supply arbitrary constructor bytes or control paths. The only
/// bytes any parameter carries are the owner's public key, and they are
/// admitted only at the approved encoding's exact width; every other
/// parameter is a typed value from a component that validated it. There
/// is no parameter through which a caller could name a program, a
/// control block, an internal key, or a spending route.
///
/// # The order of the checks
///
/// The empty leaf set is refused before the leaf set is compared with
/// the shapes, even though an empty set is also an incomplete one.
/// §7.5's key-path rule is the harder statement and
/// [`LiveConstructorRefusal::KeyPathWouldBeTheOnlySpendingRoute`] is the
/// one that says what actually went wrong; reporting it as a missing
/// leaf would bury the escape inside a bookkeeping complaint.
///
/// # Errors
///
/// The lifecycle, class, and owner-family refusals when the plan's own
/// projections do not hold together;
/// [`LiveConstructorRefusal::RepresentationNotAdmitted`] for a
/// representation the plan does not admit;
/// [`LiveConstructorRefusal::KeyPathWouldBeTheOnlySpendingRoute`] for an
/// empty leaf set; and
/// [`LiveConstructorRefusal::LeafOfAnotherRepresentation`],
/// [`LiveConstructorRefusal::LeafSetIncomplete`], or
/// [`LiveConstructorRefusal::LeafServesNoAdmittedShape`] for a leaf set
/// that is not the one the selected representation and admitted shapes
/// call for.
pub fn derive_live_receipt_constructor(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedLiveTransferOperationPlan,
    representation: LiveTransferRepresentationPlan,
    owner: OwnerKey,
    shapes: LiveTransferShapeSet,
    leaves: BTreeSet<LiveTransferLeafRole>,
) -> Result<StaticLiveReceiptConstructor, LiveConstructorRefusal> {
    let contract = target.definition();
    let owner_encoding = owner_key_encoding_closure(contract.authorization());

    let lifecycle = candidate_lifecycle(plan)?;
    check_class(plan)?;

    if !plan
        .representation()
        .admitted()
        .any(|admitted| admitted == representation)
    {
        return Err(LiveConstructorRefusal::RepresentationNotAdmitted {
            selected: representation,
        });
    }

    check_leaves(representation, &shapes, &leaves)?;

    Ok(StaticLiveReceiptConstructor {
        contract: contract.version(),
        leaf_version: contract.leaf_version(),
        owner,
        owner_encoding,
        class: plan.class().clone(),
        owner_family: plan.owner().clone(),
        value: plan.value().clone(),
        representation,
        shapes,
        leaves,
        // Inherited whole from the compact-ASH constructor, as §7.5
        // directs, and after the preflight rows it conditions the
        // inheritance on: the zero taproot tweak is admitted and the
        // tree-cost overflow returns a typed refusal, both closed in the
        // Wave-0 repair. Re-minting a policy here would have produced a
        // second one-variant enum saying the same thing, and two of them
        // is one more place for an escape to be admitted.
        internal_key: InternalKeyPolicy::UnspendableWithResidualDiscreteLogAssumption,
        key_path: KeyPathPolicy::NoAcceptedEscape,
        assumptions: BTreeSet::from([
            ConstructorAssumption::ResidualDiscreteLogOnUnspendableInternalKey,
            ConstructorAssumption::InternalKeyUnspendabilityVerifiableFromPublicData,
        ]),
        lifecycle,
    })
}

/// The candidate lifecycle of one validated plan (§7.4).
fn candidate_lifecycle(
    plan: &ValidatedLiveTransferOperationPlan,
) -> Result<CandidateTransferLifecycle, LiveConstructorRefusal> {
    let closure = plan.lifecycle();

    if closure.release_complete() {
        return Err(LiveConstructorRefusal::PlanClaimsReleaseComplete);
    }

    let implemented = closure.implemented().collect::<Vec<_>>();
    if implemented.is_empty() {
        return Err(LiveConstructorRefusal::PlanImplementsNoExit);
    }
    if closure
        .outstanding()
        .any(|exit| implemented.contains(&exit))
    {
        return Err(LiveConstructorRefusal::LifecycleExitBothImplementedAndOutstanding);
    }

    // Nonzero by the release-complete refusal above: an empty
    // outstanding set is exactly what makes a closure release-complete,
    // so the count cannot be zero here. Refusing again rather than
    // unwrapping keeps that reasoning out of the type's soundness.
    let outstanding = NonZeroUsize::new(closure.outstanding().count())
        .ok_or(LiveConstructorRefusal::PlanClaimsReleaseComplete)?;

    Ok(CandidateTransferLifecycle {
        closure: closure.clone(),
        outstanding,
    })
}

/// The class joins one constructor needs from a validated plan (§7.3).
///
/// Not a re-derivation of §5.4: the compiler derived the admitted and
/// forbidden censuses and validated them against the guide. What is
/// checked here is only that the projections this constructor is about
/// to read hold together — the family it builds for is admitted, is not
/// forbidden on its own sides, is distinguishable from the sponsor
/// family, and is the family whose owners the plan says must authorize.
fn check_class(plan: &ValidatedLiveTransferOperationPlan) -> Result<(), LiveConstructorRefusal> {
    let class = plan.class();
    let protocol = class.protocol();

    if !class.admitted().any(|object| object == protocol) {
        return Err(LiveConstructorRefusal::ProtocolFamilyNotAdmitted);
    }
    if class.forbids(protocol) {
        return Err(LiveConstructorRefusal::ProtocolFamilyForbiddenOnItsOwnSides);
    }
    if class.sponsor() == protocol {
        return Err(LiveConstructorRefusal::ProtocolAndSponsorFamiliesCoincide);
    }
    if plan.owner().object() != protocol {
        return Err(LiveConstructorRefusal::OwnerFamilyIsNotTheProtocolFamily);
    }

    Ok(())
}

/// The offered leaf set against the one the shapes call for (§7.1).
fn check_leaves(
    representation: LiveTransferRepresentationPlan,
    shapes: &LiveTransferShapeSet,
    leaves: &BTreeSet<LiveTransferLeafRole>,
) -> Result<(), LiveConstructorRefusal> {
    if leaves.is_empty() {
        return Err(LiveConstructorRefusal::KeyPathWouldBeTheOnlySpendingRoute);
    }
    if let Some(leaf) = leaves
        .iter()
        .find(|leaf| leaf.representation() != representation)
    {
        return Err(LiveConstructorRefusal::LeafOfAnotherRepresentation {
            leaf: *leaf,
            selected: representation,
        });
    }

    let required = static_transfer_leaf_set(representation, shapes);
    let missing = required
        .difference(leaves)
        .copied()
        .collect::<BTreeSet<_>>();
    if !missing.is_empty() {
        return Err(LiveConstructorRefusal::LeafSetIncomplete { missing });
    }
    let orphans = leaves
        .difference(&required)
        .copied()
        .collect::<BTreeSet<_>>();
    if !orphans.is_empty() {
        return Err(LiveConstructorRefusal::LeafServesNoAdmittedShape { orphans });
    }

    Ok(())
}

// --- Constructor mutation cases ---------------------------------------

census_enum! {
    /// What one constructor mutation case disturbs.
    ///
    /// The facets §7 gives the constructor, one per rule.
    /// [`mutation_census_defects`] requires every one of them to be
    /// disturbed by some case, because a facet nothing tries to break is
    /// a facet nobody has established anything about — and the coverage
    /// check reads [`Self::ALL`] rather than a list of its own, so a
    /// facet added later is uncovered until a case covers it.
    pub enum ConstructorFacet {
        /// The canonical owner metadata (§7.2).
        OwnerMetadata,
        /// The structural live class (§7.3).
        LiveClass,
        /// The explicit or private representation role (§6, §11.3).
        RepresentationRole,
        /// The static transfer leaf set (§7.1).
        TransferLeafSet,
        /// The absence of a key-path escape (§7.5).
        KeyPathClosure,
        /// The inherited internal-key policy (§7.5).
        InternalKeyPolicy,
        /// The candidate lifecycle (§7.4).
        CandidateLifecycle,
        /// The candidate shape set (§18.1).
        ShapeSet,
    }
}

census_enum! {
    /// What still stands between one mutation case and a target-native
    /// run.
    ///
    /// Every case that this constructor does *not* refuse carries at
    /// least one, and every one of those carries
    /// [`Self::TargetNativeRunRequired`]: §1.11 admits no target-negative
    /// claim without a complete target transaction and an observed
    /// target verdict, so a case whose residual list omitted it would be
    /// claiming an outcome nobody has seen.
    pub enum MutationResidual {
        /// The mutation is expressible only in the linked taptree, and
        /// §11's linker extension is not built.
        LinkerNotExtended,
        /// The mutation is expressible only in a complete transaction,
        /// and §12's candidate ABI is not derived.
        AbiNotDerived,
        /// The mutation needs a spent receipt built under a different
        /// constructor, and no predecessor exists to build one from.
        PredecessorConstructorAbsent,
        /// A complete target transaction and an observed target verdict
        /// (§1.11).
        TargetNativeRunRequired,
    }
}

/// What this constructor does about one mutation.
///
/// Two members, and neither is a verdict. A refusal is a fact about this
/// crate that a test can execute today; an expressible mutation is one
/// that reaches past the constructor into a component that does not
/// exist yet, and what it carries is what is missing rather than what a
/// target would say.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstructorDisposition {
    /// Construction refuses it, and the refusal names it.
    RefusedByTheConstructor,
    /// Nothing here refuses it, because nothing here can express it.
    ExpressibleByRawSurgery,
}

census_enum! {
    /// One constructor mutation case.
    ///
    /// Declaration order is the census order and ranks nothing: the
    /// cases this constructor refuses come first and the ones that reach
    /// past it follow, which is the order a reader wants — what the
    /// constructor does about a mutation is the first question, and the
    /// cases it cannot answer for read as a group rather than as
    /// scattered gaps.
    pub enum ConstructorMutationCaseId {
    /// The owner is left out of the metadata entirely.
    OwnerOmitted,
    /// The owner bytes are offered at some width other than the
    /// approved encoding's.
    OwnerKeyWrongWidth,
    /// The same owner is offered under the target's other public-key
    /// encoding.
    OwnerKeyAlternateEncoding,
    /// A scalar is offered where a public key belongs.
    ///
    /// The §1.10 case: the offering is the exact width and the exact
    /// opaque payload of the approved encoding, and is the shape a
    /// private key takes.
    OwnerKeyScalarMaterial,
    /// A leaf of the other representation is placed in the leaf set.
    LeafOfTheOtherRepresentation,
    /// An admitted shape's coordinator leaf is removed.
    CoordinatorLeafRemoved,
    /// An admitted shape's member leaf is removed.
    MemberLeafRemoved,
    /// A leaf is added for a shape the candidate does not admit.
    LeafForAnUnadmittedShape,
    /// Every leaf is removed.
    ///
    /// The key-path case: an output with no script path can be spent
    /// only through its key path.
    EveryTransferLeafRemoved,
    /// A shape outside the candidate bounds is offered.
    ShapeOutsideTheCandidateBounds,
    /// A sponsor-change role is declared with no sponsor region.
    SponsorChangeWithoutSponsorRegion,

    /// The owner bytes are replaced in the linked output after the
    /// constructor committed them.
    OwnerBytesReplacedInTheLinkedOutput,
    /// A time-locked predecessor is offered to a live transfer leaf.
    TimeLockedPredecessorOfferedToATransferLeaf,
    /// A key-path spend is attempted against the linked output.
    KeyPathSpendAttemptedOnTheLinkedOutput,
    /// The unspendable internal key is replaced with a spendable one.
    InternalKeyReplacedWithASpendableOne,
    /// A burn leaf is added to the linked taptree.
    BurnLeafAddedToTheTaptree,
    /// The other representation's leaves are substituted in the linked
    /// tree.
    RepresentationSwappedInTheLinkedTree,
    }
}

/// One constructor mutation case and what becomes of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructorMutationCase {
    id: ConstructorMutationCaseId,
    disturbs: ConstructorFacet,
    disposition: ConstructorDisposition,
    residuals: BTreeSet<MutationResidual>,
}

impl ConstructorMutationCase {
    /// The case's identity.
    #[must_use]
    pub const fn id(&self) -> ConstructorMutationCaseId {
        self.id
    }

    /// The constructor facet the mutation disturbs.
    #[must_use]
    pub const fn disturbs(&self) -> ConstructorFacet {
        self.disturbs
    }

    /// What this constructor does about the mutation.
    #[must_use]
    pub const fn disposition(&self) -> ConstructorDisposition {
        self.disposition
    }

    /// What still stands between the case and a target-native run.
    ///
    /// Empty exactly for the refused cases, whose whole content is a
    /// refusal a test executes here.
    pub fn residuals(&self) -> impl Iterator<Item = MutationResidual> + '_ {
        self.residuals.iter().copied()
    }
}

/// Why a mutation-case census does not stand up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationCensusDefect {
    /// The census names a case twice, or omits one.
    CensusMismatch {
        /// Named by [`ConstructorMutationCaseId::ALL`] and absent from
        /// the map.
        missing: BTreeSet<ConstructorMutationCaseId>,
        /// Present in the map and not named by the census constant.
        unexpected: BTreeSet<ConstructorMutationCaseId>,
    },
    /// A refused case carries a residual.
    ///
    /// A residual says a run is still owed. A case this constructor
    /// refuses is settled here, and carrying one would suggest the
    /// refusal was provisional.
    RefusedCaseCarriesResidual {
        /// The case.
        case: ConstructorMutationCaseId,
    },
    /// An expressible case carries no residual.
    ///
    /// It would be claiming to be runnable today, and none is.
    ExpressibleCaseWithoutResidual {
        /// The case.
        case: ConstructorMutationCaseId,
    },
    /// An expressible case does not require a target-native run.
    ExpressibleCaseWithoutTargetRun {
        /// The case.
        case: ConstructorMutationCaseId,
    },
    /// No case disturbs one constructor facet.
    FacetNotDisturbed {
        /// The facet nothing tries to break.
        facet: ConstructorFacet,
    },
}

/// The census of constructor mutation cases (§7).
///
/// Eleven cases this constructor refuses and six that reach past it.
/// The split is the useful part: it says exactly where the constructor's
/// authority ends, and every case on the far side of that line names the
/// component that has to exist before anybody can run it.
#[must_use]
pub fn constructor_mutation_cases() -> BTreeMap<ConstructorMutationCaseId, ConstructorMutationCase>
{
    use ConstructorDisposition as Disposition;
    use ConstructorFacet as Facet;
    use ConstructorMutationCaseId as Case;
    use MutationResidual as Residual;

    ConstructorMutationCaseId::ALL
        .iter()
        .map(|id| {
            let (disturbs, disposition, residuals): (_, _, &[Residual]) = match id {
                Case::OwnerOmitted
                | Case::OwnerKeyWrongWidth
                | Case::OwnerKeyAlternateEncoding
                | Case::OwnerKeyScalarMaterial => (
                    Facet::OwnerMetadata,
                    Disposition::RefusedByTheConstructor,
                    &[],
                ),
                Case::LeafOfTheOtherRepresentation => (
                    Facet::RepresentationRole,
                    Disposition::RefusedByTheConstructor,
                    &[],
                ),
                Case::CoordinatorLeafRemoved
                | Case::MemberLeafRemoved
                | Case::LeafForAnUnadmittedShape => (
                    Facet::TransferLeafSet,
                    Disposition::RefusedByTheConstructor,
                    &[],
                ),
                Case::EveryTransferLeafRemoved => (
                    Facet::KeyPathClosure,
                    Disposition::RefusedByTheConstructor,
                    &[],
                ),
                Case::ShapeOutsideTheCandidateBounds | Case::SponsorChangeWithoutSponsorRegion => {
                    (Facet::ShapeSet, Disposition::RefusedByTheConstructor, &[])
                }

                // Past the constructor. Each names what would have to
                // exist first: the owner bytes, the internal key, and
                // the tree's leaves are resolved at link, the class and
                // key-path cases need a transaction to put a predecessor
                // or a witness in, and all of them need a verdict nobody
                // can observe yet.
                Case::OwnerBytesReplacedInTheLinkedOutput => (
                    Facet::OwnerMetadata,
                    Disposition::ExpressibleByRawSurgery,
                    &[
                        Residual::LinkerNotExtended,
                        Residual::TargetNativeRunRequired,
                    ],
                ),
                Case::InternalKeyReplacedWithASpendableOne => (
                    Facet::InternalKeyPolicy,
                    Disposition::ExpressibleByRawSurgery,
                    &[
                        Residual::LinkerNotExtended,
                        Residual::TargetNativeRunRequired,
                    ],
                ),
                Case::BurnLeafAddedToTheTaptree => (
                    Facet::CandidateLifecycle,
                    Disposition::ExpressibleByRawSurgery,
                    &[
                        Residual::LinkerNotExtended,
                        Residual::TargetNativeRunRequired,
                    ],
                ),
                Case::RepresentationSwappedInTheLinkedTree => (
                    Facet::RepresentationRole,
                    Disposition::ExpressibleByRawSurgery,
                    &[
                        Residual::LinkerNotExtended,
                        Residual::TargetNativeRunRequired,
                    ],
                ),
                Case::TimeLockedPredecessorOfferedToATransferLeaf => (
                    Facet::LiveClass,
                    Disposition::ExpressibleByRawSurgery,
                    &[
                        Residual::PredecessorConstructorAbsent,
                        Residual::AbiNotDerived,
                        Residual::TargetNativeRunRequired,
                    ],
                ),
                Case::KeyPathSpendAttemptedOnTheLinkedOutput => (
                    Facet::KeyPathClosure,
                    Disposition::ExpressibleByRawSurgery,
                    &[
                        Residual::LinkerNotExtended,
                        Residual::AbiNotDerived,
                        Residual::TargetNativeRunRequired,
                    ],
                ),
            };

            (
                *id,
                ConstructorMutationCase {
                    id: *id,
                    disturbs,
                    disposition,
                    residuals: residuals.iter().copied().collect(),
                },
            )
        })
        .collect()
}

/// Every way one mutation-case census fails to stand up, in canonical
/// order.
///
/// Empty for a census that holds. A function rather than a test
/// assertion because the properties it checks are what make the census
/// worth reading — a reader relying on the refused/expressible split is
/// entitled to confirm the split is a partition and that every facet is
/// covered.
#[must_use]
pub fn mutation_census_defects(
    cases: &BTreeMap<ConstructorMutationCaseId, ConstructorMutationCase>,
) -> Vec<MutationCensusDefect> {
    let mut defects = Vec::new();

    let named = ConstructorMutationCaseId::ALL
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let present = cases.keys().copied().collect::<BTreeSet<_>>();
    let missing = named.difference(&present).copied().collect::<BTreeSet<_>>();
    let unexpected = present.difference(&named).copied().collect::<BTreeSet<_>>();
    if !missing.is_empty() || !unexpected.is_empty() {
        defects.push(MutationCensusDefect::CensusMismatch {
            missing,
            unexpected,
        });
    }

    for case in cases.values() {
        let has_residual = case.residuals().next().is_some();
        match case.disposition() {
            ConstructorDisposition::RefusedByTheConstructor => {
                if has_residual {
                    defects
                        .push(MutationCensusDefect::RefusedCaseCarriesResidual { case: case.id() });
                }
            }
            ConstructorDisposition::ExpressibleByRawSurgery => {
                if has_residual {
                    if !case
                        .residuals()
                        .any(|residual| residual == MutationResidual::TargetNativeRunRequired)
                    {
                        defects.push(MutationCensusDefect::ExpressibleCaseWithoutTargetRun {
                            case: case.id(),
                        });
                    }
                } else {
                    defects.push(MutationCensusDefect::ExpressibleCaseWithoutResidual {
                        case: case.id(),
                    });
                }
            }
        }
    }

    let disturbed = cases
        .values()
        .map(ConstructorMutationCase::disturbs)
        .collect::<BTreeSet<_>>();
    for facet in ConstructorFacet::ALL {
        if !disturbed.contains(facet) {
            defects.push(MutationCensusDefect::FacetNotDisturbed { facet: *facet });
        }
    }

    defects
}
