//! The typed live-transfer symbol census (§11.2).
//!
//! # What a live link symbol is
//!
//! The same rule [`crate::symbol`] states, over Guide 13's vocabulary: a
//! symbol is a typed key, never a name, and there is no parse, no lookup
//! by text, and no rendering step anywhere in the resolution path.
//!
//! # Why the link needs a key the bundle does not have
//!
//! [`tapscript::LiveBundleSymbol`] is the *bundle's* symbol table, and a
//! bundle holds one representation, so `CoordinatorProgram { shape }`
//! identifies a program unambiguously inside it. A link does not hold
//! one representation. §11.6's linked candidate carries both plans'
//! programs, and §11.2 names the explicit and the private coordinator as
//! two roles rather than one, so the link's key is the bundle's key
//! qualified by the representation whose bundle named it. Reusing the
//! bundle's key here would give the explicit and the private coordinator
//! of one shape the same identity, which is the collision §11.3 exists
//! to prevent, arriving through a map rather than through a script.
//!
//! # The owner-parameterized constructor
//!
//! §7.6 makes each output constructor a function of the destination
//! owner, and §11.2 names the live-receipt constructor as a symbol. The
//! two together mean one constructor symbol per (owner, representation),
//! and [`OwnerParameter`] is how that is said: a typed newtype over the
//! canonical [`OwnerKey`] the constructor committed, carried into the
//! symbol key itself. Nothing here concatenates an owner into a name, and
//! nothing here indexes owners by position — a rendering and an ordinal
//! are both ways of losing which owner a symbol belongs to, and both are
//! what §14.3's rule about display strings is really about.
//!
//! # Four origins, because four layers settle things
//!
//! [`crate::symbol::DefinitionOrigin`] has two, which was right for a
//! link that resolved deployment values into a bundle. Guide 13's link
//! settles two more kinds of thing, and merging them into the existing
//! pair would have made the census say something false about who is
//! answerable for a value:
//!
//! - [`LiveDefinitionOrigin::ReviewedContract`] for the selected sighash
//!   profile, which is neither a deployment's choice nor the bundle's
//!   invention but the reviewed target's own classification, read here
//!   and required to agree with the disposition the bundle recorded;
//! - [`LiveDefinitionOrigin::Link`] for the live-receipt constructor
//!   itself, which is the §10.4 induction's link-time end: no leaf fixed
//!   at construction can carry a literal for an owner-parameterized
//!   program, and the tree over that owner's leaves does not exist until
//!   this layer builds it.
//!
//! # No digest stands in for the constructor
//!
//! §1.13 mints no `LiveReceiptConstructorHash`, so
//! [`LinkedConstructorPlacement`] is not a summary of the constructor and
//! must not be read as one. It says which owner and representation a
//! committed leaf set belongs to; the constructor's identity remains its
//! typed value, which is the linked bundle's own programs and tree. A
//! field holding a stand-in identity here would be the reserved field
//! §1.10 refuses.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::upstream::LiveTransferRepresentationPlan;
use tapscript::{
    CandidateRelocatableLiveTransferBundle, LiveBundleSymbol, LiveTransferLeafRole,
    LiveTransferShape, OwnerKey, OwnerKeyEncodingClosure, OwnerProfileDisposition,
    OwnerSighashProfile, StackItem, SymbolBinding, TapscriptProgram,
};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::error::LinkRefusal;
use crate::live_deployment::{LiveLinkDeploymentParameters, owner_stack_item};

// --- The owner parameter ----------------------------------------------

/// The owner one constructor symbol is parameterized by (§7.6, §11.2).
///
/// A newtype over the canonical owner metadata rather than a name, an
/// index, or a digest of one. The three rejected alternatives fail for
/// three different reasons and it is worth naming them: a name is a
/// rendering, and §14.3 says a rendering is not identity; an index is a
/// position in some list, so two links over different owner sets would
/// reuse one key; and a digest would be an identity §1.13 mints none of.
///
/// The key is therefore the owner metadata itself, which is public by
/// construction — [`OwnerKey`] admits only a public key at the reviewed
/// contract's approved encoding, and §1.10 gives it no way to hold
/// anything else.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnerParameter {
    key: OwnerKey,
}

impl OwnerParameter {
    /// The parameter one constructor's committed owner names.
    #[must_use]
    pub const fn new(key: OwnerKey) -> Self {
        Self { key }
    }

    /// The canonical owner metadata.
    #[must_use]
    pub const fn key(&self) -> &OwnerKey {
        &self.key
    }
}

// --- The symbol vocabulary --------------------------------------------

/// One typed link-time role of a live-transfer link (§11.2).
///
/// §11.2's list, with the roles it leaves implicit made explicit. The
/// guide says "expected typed roles include", so the census is a lower
/// bound rather than a closed list, and the four members below that
/// §11.2 does not name — the reserve asset, the destination program
/// version, and the two sponsor-change roles — are here because the
/// emitted programs push literals for them and a relocation naming a
/// symbol the census does not describe would have nothing to resolve
/// against.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveLinkSymbol {
    /// The explicit protocol asset every receipt carries.
    ProtocolAsset,
    /// The explicit reserve asset every sponsor and fee role carries.
    ReserveAsset,
    /// The version a destination's program is read at.
    DestinationProgramVersion,
    /// The sponsor-change role's witness program.
    SponsorChangeProgram,
    /// The version the sponsor-change program is read at.
    SponsorChangeProgramVersion,
    /// The target fee role's program digest.
    TargetFeeRole,
    /// The canonical owner-metadata encoding the contract approves.
    ///
    /// §11.2's *owner metadata schema*: the encoding class and its
    /// canonicality rule, not any owner's bytes. A candidate that
    /// resolved an owner without settling the schema would have no way
    /// to say that two byte strings are the same owner.
    OwnerMetadataSchema,
    /// One committed owner's public key.
    OwnerPublicKey {
        /// The owner whose key this is.
        owner: OwnerParameter,
    },
    /// The unspendable taproot internal key (§7.5).
    UnspendableInternalKey,
    /// The reviewed tapscript leaf version.
    TargetLeafVersion,
    /// The selected owner sighash profile (§1.7, §9.2, §11.2).
    SelectedSighashProfile,
    /// The candidate receipt-input bound.
    CandidateReceiptInputBound,
    /// The candidate receipt-output bound.
    CandidateReceiptOutputBound,
    /// The candidate sponsor-region bound.
    CandidateSponsorBound,
    /// One representation plan this link carries programs for.
    RepresentationPlan {
        /// The plan.
        representation: LiveTransferRepresentationPlan,
    },
    /// One owner's live-receipt constructor under one representation.
    ///
    /// Owner-parameterized, which is the whole content of the role:
    /// §7.6 derives each output constructor from the destination owner,
    /// so there is one of these per (owner, representation) and no
    /// constructor symbol that is not somebody's.
    LiveReceiptConstructor {
        /// The committed owner.
        owner: OwnerParameter,
        /// The representation whose leaves the constructor holds.
        representation: LiveTransferRepresentationPlan,
    },
    /// One representation's coordinator program for one shape.
    CoordinatorProgram {
        /// The representation whose obligations the program carries.
        representation: LiveTransferRepresentationPlan,
        /// The shape whose counts the program authenticates.
        shape: LiveTransferShape,
    },
    /// One representation's member program for one receipt-input count.
    MemberProgram {
        /// The representation whose obligations the program carries.
        representation: LiveTransferRepresentationPlan,
        /// The receipt-input count whose member range it bounds.
        receipt_inputs: u8,
    },
}

impl LiveLinkSymbol {
    /// The bundle symbol this link symbol qualifies, where there is one.
    ///
    /// `None` for the four roles no bundle names — the owner metadata
    /// schema, the representation plan, and the live-receipt constructor
    /// are settled outside any one bundle's table, which is why the link
    /// needs a key of its own for them.
    #[must_use]
    pub const fn bundle_symbol(&self) -> Option<LiveBundleSymbol> {
        Some(match self {
            Self::ProtocolAsset => LiveBundleSymbol::ProtocolAsset,
            Self::ReserveAsset => LiveBundleSymbol::ReserveAsset,
            Self::DestinationProgramVersion => LiveBundleSymbol::DestinationProgramVersion,
            Self::SponsorChangeProgram => LiveBundleSymbol::SponsorChangeProgram,
            Self::SponsorChangeProgramVersion => LiveBundleSymbol::SponsorChangeProgramVersion,
            Self::TargetFeeRole => LiveBundleSymbol::TargetFeeRoleProgramDigest,
            Self::OwnerPublicKey { .. } => LiveBundleSymbol::OwnerPublicKey,
            Self::UnspendableInternalKey => LiveBundleSymbol::UnspendableInternalKey,
            Self::TargetLeafVersion => LiveBundleSymbol::TargetLeafVersion,
            Self::SelectedSighashProfile => LiveBundleSymbol::SelectedSighashProfile,
            Self::CandidateReceiptInputBound => LiveBundleSymbol::CandidateReceiptInputBound,
            Self::CandidateReceiptOutputBound => LiveBundleSymbol::CandidateReceiptOutputBound,
            Self::CandidateSponsorBound => LiveBundleSymbol::CandidateSponsorBound,
            Self::CoordinatorProgram { shape, .. } => {
                LiveBundleSymbol::CoordinatorProgram { shape: *shape }
            }
            Self::MemberProgram { receipt_inputs, .. } => LiveBundleSymbol::MemberProgram {
                receipt_inputs: *receipt_inputs,
            },
            Self::OwnerMetadataSchema
            | Self::RepresentationPlan { .. }
            | Self::LiveReceiptConstructor { .. } => return None,
        })
    }

    /// The §11.2 role this symbol fills.
    #[must_use]
    pub const fn role(&self) -> LiveLinkRole {
        match self {
            Self::ProtocolAsset => LiveLinkRole::ProtocolAsset,
            Self::ReserveAsset => LiveLinkRole::ReserveAsset,
            Self::DestinationProgramVersion => LiveLinkRole::DestinationProgramVersion,
            Self::SponsorChangeProgram => LiveLinkRole::SponsorChangeProgram,
            Self::SponsorChangeProgramVersion => LiveLinkRole::SponsorChangeProgramVersion,
            Self::TargetFeeRole => LiveLinkRole::TargetFeeRole,
            Self::OwnerMetadataSchema => LiveLinkRole::OwnerMetadataSchema,
            Self::OwnerPublicKey { .. } => LiveLinkRole::OwnerPublicKey,
            Self::UnspendableInternalKey => LiveLinkRole::UnspendableInternalKey,
            Self::TargetLeafVersion => LiveLinkRole::TargetLeafVersion,
            Self::SelectedSighashProfile => LiveLinkRole::SelectedSighashProfile,
            Self::CandidateReceiptInputBound => LiveLinkRole::CandidateInputBound,
            Self::CandidateReceiptOutputBound => LiveLinkRole::CandidateOutputBound,
            Self::CandidateSponsorBound => LiveLinkRole::CandidateSponsorBound,
            Self::RepresentationPlan { .. } => LiveLinkRole::RepresentationPlan,
            Self::LiveReceiptConstructor { .. } => LiveLinkRole::LiveReceiptConstructor,
            Self::CoordinatorProgram { representation, .. } => match representation {
                LiveTransferRepresentationPlan::Explicit => {
                    LiveLinkRole::ExplicitCoordinatorProgram
                }
                LiveTransferRepresentationPlan::PrivateCommitted => {
                    LiveLinkRole::PrivateCoordinatorProgram
                }
            },
            Self::MemberProgram { representation, .. } => match representation {
                LiveTransferRepresentationPlan::Explicit => LiveLinkRole::ExplicitMemberProgram,
                LiveTransferRepresentationPlan::PrivateCommitted => {
                    LiveLinkRole::PrivateMemberProgram
                }
            },
        }
    }
}

/// One role of §11.2's list.
///
/// The guide's own fifteen, with the four the emitted programs add. The
/// split between an explicit and a private coordinator is §11.2's, and
/// it is why the role is derived from the symbol rather than declared:
/// a coordinator's role follows from the representation it carries, so a
/// private program filling the explicit role has no spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveLinkRole {
    /// §11.2's `U asset`.
    ProtocolAsset,
    /// §11.2's `owner metadata schema`.
    OwnerMetadataSchema,
    /// §11.2's `live receipt constructor`.
    LiveReceiptConstructor,
    /// §11.2's `explicit coordinator program`.
    ExplicitCoordinatorProgram,
    /// §11.2's `explicit member program`.
    ExplicitMemberProgram,
    /// §11.2's `private coordinator program`.
    PrivateCoordinatorProgram,
    /// §11.2's `private member program`.
    PrivateMemberProgram,
    /// §11.2's `representation plan`.
    RepresentationPlan,
    /// §11.2's `target leaf version`.
    TargetLeafVersion,
    /// §11.2's `unspendable internal key`.
    UnspendableInternalKey,
    /// §11.2's `candidate input bound`.
    CandidateInputBound,
    /// §11.2's `candidate output bound`.
    CandidateOutputBound,
    /// §11.2's `candidate sponsor bound`.
    CandidateSponsorBound,
    /// §11.2's `selected sighash profile`.
    SelectedSighashProfile,
    /// §11.2's `target fee role`.
    TargetFeeRole,
    /// The committed owner's public key.
    OwnerPublicKey,
    /// The explicit reserve asset.
    ReserveAsset,
    /// The version a destination's program is read at.
    DestinationProgramVersion,
    /// The sponsor-change role's witness program.
    SponsorChangeProgram,
    /// The version the sponsor-change program is read at.
    SponsorChangeProgramVersion,
}

impl LiveLinkRole {
    /// The representation this role belongs to, where it belongs to one.
    ///
    /// `None` for every role that is not a program: the protocol asset,
    /// the owner schema, the bounds, the leaf version, and the sighash
    /// profile are one link's regardless of how many representations it
    /// carries. Only §11.2's four program roles are per-plan, which is
    /// the same split §11.3 draws through the leaf sets.
    #[must_use]
    pub const fn representation(self) -> Option<LiveTransferRepresentationPlan> {
        match self {
            Self::ExplicitCoordinatorProgram | Self::ExplicitMemberProgram => {
                Some(LiveTransferRepresentationPlan::Explicit)
            }
            Self::PrivateCoordinatorProgram | Self::PrivateMemberProgram => {
                Some(LiveTransferRepresentationPlan::PrivateCommitted)
            }
            _ => None,
        }
    }

    /// Every role, in census order.
    ///
    /// Read by [`link_role_defects`] rather than restated there, so a
    /// role added to this list is uncovered until some symbol fills it.
    pub const ALL: &'static [Self] = &[
        Self::ProtocolAsset,
        Self::OwnerMetadataSchema,
        Self::LiveReceiptConstructor,
        Self::ExplicitCoordinatorProgram,
        Self::ExplicitMemberProgram,
        Self::PrivateCoordinatorProgram,
        Self::PrivateMemberProgram,
        Self::RepresentationPlan,
        Self::TargetLeafVersion,
        Self::UnspendableInternalKey,
        Self::CandidateInputBound,
        Self::CandidateOutputBound,
        Self::CandidateSponsorBound,
        Self::SelectedSighashProfile,
        Self::TargetFeeRole,
        Self::OwnerPublicKey,
        Self::ReserveAsset,
        Self::DestinationProgramVersion,
        Self::SponsorChangeProgram,
        Self::SponsorChangeProgramVersion,
    ];

    /// The fifteen roles §11.2 lists by name, in the guide's own order.
    ///
    /// Separate from [`Self::ALL`] because the two answer different
    /// questions. A census that filled every role it invented and none
    /// the guide named would pass a completeness check against itself,
    /// so the guide's list is kept as its own constant and checked
    /// against.
    pub const GUIDE_LISTED: &'static [Self] = &[
        Self::ProtocolAsset,
        Self::OwnerMetadataSchema,
        Self::LiveReceiptConstructor,
        Self::ExplicitCoordinatorProgram,
        Self::ExplicitMemberProgram,
        Self::PrivateCoordinatorProgram,
        Self::PrivateMemberProgram,
        Self::RepresentationPlan,
        Self::TargetLeafVersion,
        Self::UnspendableInternalKey,
        Self::CandidateInputBound,
        Self::CandidateOutputBound,
        Self::CandidateSponsorBound,
        Self::SelectedSighashProfile,
        Self::TargetFeeRole,
    ];
}

// --- Typed values ------------------------------------------------------

/// What kind of thing one live symbol resolves to.
///
/// Not a width, for the reason [`crate::symbol::SymbolType`] is not one:
/// the closed asset, the reserve asset, an x-only key, and a program
/// digest are all thirty-two bytes under the reviewed contract, so width
/// alone cannot tell an asset supplied as a key from one supplied as an
/// asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveSymbolType {
    /// An asset identifier compared with an introspected asset field.
    Asset,
    /// The program bytes of a witness-program script.
    WitnessProgram,
    /// A witness-program version, carried as a script number.
    ScriptNumber,
    /// The digest standing in for a script that is not a witness program.
    ProgramDigest,
    /// An x-only public point.
    XOnlyPublicKey,
    /// The owner-metadata encoding closure.
    OwnerKeyEncoding,
    /// The leaf version every committed leaf carries.
    LeafVersion,
    /// A bound of the candidate shape set.
    ShapeBound,
    /// A complete committed tapscript leaf program.
    LeafScript,
    /// A representation plan.
    RepresentationPlan,
    /// The selected sighash profile and what the review says about it.
    SighashProfile,
    /// One owner's constructor, placed over a committed leaf set.
    LinkedConstructor,
}

/// Which owner and representation one committed leaf set belongs to.
///
/// Not a constructor identity and not a digest of one. §1.13 mints no
/// `LiveReceiptConstructorHash`, and §7.6 leaves a constructor's identity
/// as its typed value, so what this states is the *placement*: these
/// leaves, committed by this link, are the constructor for this owner
/// under this representation. The programs and the tree live in the
/// linked bundle, where a consumer compares them by value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LinkedConstructorPlacement {
    owner: OwnerParameter,
    representation: LiveTransferRepresentationPlan,
    committed_leaves: BTreeSet<LiveTransferLeafRole>,
}

impl LinkedConstructorPlacement {
    /// State one constructor's placement over a committed leaf set.
    #[must_use]
    pub const fn new(
        owner: OwnerParameter,
        representation: LiveTransferRepresentationPlan,
        committed_leaves: BTreeSet<LiveTransferLeafRole>,
    ) -> Self {
        Self {
            owner,
            representation,
            committed_leaves,
        }
    }

    /// The committed owner.
    #[must_use]
    pub const fn owner(&self) -> &OwnerParameter {
        &self.owner
    }

    /// The representation whose leaves the constructor holds.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// Every leaf the committed tree holds for this constructor.
    #[must_use]
    pub const fn committed_leaves(&self) -> &BTreeSet<LiveTransferLeafRole> {
        &self.committed_leaves
    }
}

/// The selected sighash profile, with what the review establishes.
///
/// Both halves, because §1.7's argument and the target's review are two
/// different statements. The profile is the protocol's requirement —
/// which transaction dimensions an owner's signature must commit to —
/// and the disposition is what the reviewed contract establishes about
/// those dimensions, which today is every one of them.
///
/// Carrying the second half is worth as much now as it was when the
/// answer was that the contract established none. The gap it named was
/// [`tapscript::RecognitionResidual::SighashProfileUnreviewed`], and it
/// travelled with this value rather than beside it, so a consumer
/// holding the profile held the reason it could not yet be relied on. A
/// consumer holding one now holds the assessment that says it can, from
/// the same field, and would hold the gap again the moment the reviewed
/// contract lost a required dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedSighashProfile {
    profile: OwnerSighashProfile,
    disposition: OwnerProfileDisposition,
}

impl SelectedSighashProfile {
    /// State the selection and its assessment together.
    #[must_use]
    pub const fn new(profile: OwnerSighashProfile, disposition: OwnerProfileDisposition) -> Self {
        Self {
            profile,
            disposition,
        }
    }

    /// The dimensions the signature must commit to, and the refused ones.
    #[must_use]
    pub const fn profile(&self) -> &OwnerSighashProfile {
        &self.profile
    }

    /// What the reviewed contract establishes about those dimensions.
    #[must_use]
    pub const fn disposition(&self) -> &OwnerProfileDisposition {
        &self.disposition
    }

    /// Whether the review establishes every dimension the profile needs.
    ///
    /// False for this candidate, and a consumer reading a verified
    /// signature as authorization over §1.7's protected data while this
    /// is false is reading past the residual rather than through it.
    #[must_use]
    pub const fn is_established(&self) -> bool {
        matches!(self.disposition, OwnerProfileDisposition::Established)
    }
}

/// One live symbol's resolved value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveSymbolValue {
    /// An asset identifier.
    Asset(StackItem),
    /// A witness program's bytes.
    WitnessProgram(StackItem),
    /// A script-number literal.
    ScriptNumber(i64),
    /// A program digest.
    ProgramDigest(StackItem),
    /// An x-only public point.
    XOnlyPublicKey(StackItem),
    /// The approved owner-metadata encoding.
    OwnerKeyEncoding(Box<OwnerKeyEncodingClosure>),
    /// A leaf version.
    LeafVersion(LeafVersion),
    /// A shape bound.
    ShapeBound(u8),
    /// A committed leaf program.
    LeafScript(Box<TapscriptProgram>),
    /// A representation plan.
    RepresentationPlan(LiveTransferRepresentationPlan),
    /// The selected sighash profile.
    SighashProfile(Box<SelectedSighashProfile>),
    /// One owner's constructor over a committed leaf set.
    LinkedConstructor(Box<LinkedConstructorPlacement>),
}

impl LiveSymbolValue {
    /// The kind this value is.
    #[must_use]
    pub const fn symbol_type(&self) -> LiveSymbolType {
        match self {
            Self::Asset(_) => LiveSymbolType::Asset,
            Self::WitnessProgram(_) => LiveSymbolType::WitnessProgram,
            Self::ScriptNumber(_) => LiveSymbolType::ScriptNumber,
            Self::ProgramDigest(_) => LiveSymbolType::ProgramDigest,
            Self::XOnlyPublicKey(_) => LiveSymbolType::XOnlyPublicKey,
            Self::OwnerKeyEncoding(_) => LiveSymbolType::OwnerKeyEncoding,
            Self::LeafVersion(_) => LiveSymbolType::LeafVersion,
            Self::ShapeBound(_) => LiveSymbolType::ShapeBound,
            Self::LeafScript(_) => LiveSymbolType::LeafScript,
            Self::RepresentationPlan(_) => LiveSymbolType::RepresentationPlan,
            Self::SighashProfile(_) => LiveSymbolType::SighashProfile,
            Self::LinkedConstructor(_) => LiveSymbolType::LinkedConstructor,
        }
    }
}

/// The type one live symbol's role requires of its definition.
#[must_use]
pub const fn live_declared_type(symbol: &LiveLinkSymbol) -> LiveSymbolType {
    match symbol {
        LiveLinkSymbol::ProtocolAsset | LiveLinkSymbol::ReserveAsset => LiveSymbolType::Asset,
        LiveLinkSymbol::SponsorChangeProgram => LiveSymbolType::WitnessProgram,
        LiveLinkSymbol::DestinationProgramVersion | LiveLinkSymbol::SponsorChangeProgramVersion => {
            LiveSymbolType::ScriptNumber
        }
        LiveLinkSymbol::TargetFeeRole => LiveSymbolType::ProgramDigest,
        LiveLinkSymbol::OwnerPublicKey { .. } | LiveLinkSymbol::UnspendableInternalKey => {
            LiveSymbolType::XOnlyPublicKey
        }
        LiveLinkSymbol::OwnerMetadataSchema => LiveSymbolType::OwnerKeyEncoding,
        LiveLinkSymbol::TargetLeafVersion => LiveSymbolType::LeafVersion,
        LiveLinkSymbol::SelectedSighashProfile => LiveSymbolType::SighashProfile,
        LiveLinkSymbol::CandidateReceiptInputBound
        | LiveLinkSymbol::CandidateReceiptOutputBound
        | LiveLinkSymbol::CandidateSponsorBound => LiveSymbolType::ShapeBound,
        LiveLinkSymbol::RepresentationPlan { .. } => LiveSymbolType::RepresentationPlan,
        LiveLinkSymbol::LiveReceiptConstructor { .. } => LiveSymbolType::LinkedConstructor,
        LiveLinkSymbol::CoordinatorProgram { .. } | LiveLinkSymbol::MemberProgram { .. } => {
            LiveSymbolType::LeafScript
        }
    }
}

// --- The census --------------------------------------------------------

/// Which layer settled one live definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveDefinitionOrigin {
    /// The caller's public deployment parameters.
    DeploymentParameters,
    /// The relocatable bundle itself.
    Bundle,
    /// The reviewed target contract.
    ///
    /// Not a deployment's choice and not the bundle's invention. The
    /// sighash profile is the candidate's protocol argument assessed
    /// against the reviewed contract's own sighash capability, so the
    /// layer answerable for it is the review.
    ReviewedContract,
    /// This link.
    ///
    /// The §10.4 induction's link-time end: an owner-parameterized
    /// constructor's leaves cannot carry a literal for the constructor,
    /// and the tree over them does not exist until a link builds it.
    Link,
}

/// One definition of one live symbol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveSymbolDefinition {
    symbol: LiveLinkSymbol,
    value: LiveSymbolValue,
    origin: LiveDefinitionOrigin,
}

impl LiveSymbolDefinition {
    /// The symbol this defines.
    #[must_use]
    pub const fn symbol(&self) -> &LiveLinkSymbol {
        &self.symbol
    }

    /// The resolved value.
    #[must_use]
    pub const fn value(&self) -> &LiveSymbolValue {
        &self.value
    }

    /// Which layer settled it.
    #[must_use]
    pub const fn origin(&self) -> LiveDefinitionOrigin {
        self.origin
    }
}

/// The complete live definition census, sorted by stable key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveDefinitionCensus {
    definitions: BTreeMap<LiveLinkSymbol, LiveSymbolDefinition>,
}

impl LiveDefinitionCensus {
    /// Every definition, in canonical order.
    #[must_use]
    pub const fn definitions(&self) -> &BTreeMap<LiveLinkSymbol, LiveSymbolDefinition> {
        &self.definitions
    }

    /// One symbol's definition.
    #[must_use]
    pub fn definition(&self, symbol: &LiveLinkSymbol) -> Option<&LiveSymbolDefinition> {
        self.definitions.get(symbol)
    }

    /// How many symbols the census holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Whether the census is empty, which a validated one never is.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Every symbol one origin settled, in canonical order.
    pub fn from_origin(
        &self,
        origin: LiveDefinitionOrigin,
    ) -> impl Iterator<Item = &LiveLinkSymbol> {
        self.definitions
            .values()
            .filter(move |definition| definition.origin == origin)
            .map(LiveSymbolDefinition::symbol)
    }

    /// Every §11.2 role some definition fills, in census order.
    #[must_use]
    pub fn filled_roles(&self) -> BTreeSet<LiveLinkRole> {
        self.definitions.keys().map(LiveLinkSymbol::role).collect()
    }

    /// Add one definition, refusing a second claim on one key.
    ///
    /// # Errors
    ///
    /// [`LinkRefusal::DuplicateLiveSymbolDefinition`] when the key is
    /// already claimed, and
    /// [`LinkRefusal::IncompatibleLiveSymbolType`] when the value is not
    /// the kind the symbol's role declares.
    pub fn define(
        &mut self,
        symbol: LiveLinkSymbol,
        value: LiveSymbolValue,
        origin: LiveDefinitionOrigin,
    ) -> Result<(), LinkRefusal> {
        let expected = live_declared_type(&symbol);
        let actual = value.symbol_type();
        if expected != actual {
            return Err(LinkRefusal::IncompatibleLiveSymbolType {
                symbol: Box::new(symbol),
                expected,
                actual,
            });
        }
        if self.definitions.contains_key(&symbol) {
            return Err(LinkRefusal::DuplicateLiveSymbolDefinition(Box::new(symbol)));
        }
        self.definitions.insert(
            symbol.clone(),
            LiveSymbolDefinition {
                symbol,
                value,
                origin,
            },
        );
        Ok(())
    }
}

/// Every §11.2 role no census in one link fills, in census order.
///
/// A property of the *link* rather than of any one census, and that is
/// the whole reason it takes a map. §11.3 keeps the two representations'
/// leaf sets disjoint and each emitted bundle carries one representation,
/// so no single census can fill both the explicit and the private
/// program roles — a per-census check would report every link as
/// incomplete, and a per-census check that quietly excused the absence
/// would stop noticing a census that had really lost a role.
///
/// A role belonging to a representation the link does not carry is out of
/// scope rather than unfilled. A link over the explicit plan alone is a
/// legitimate artifact — §11.6 has the candidate state its plan status
/// precisely so that a reader can tell which plans it speaks for — and it
/// is not required to fill the private plan's roles. What it must not do
/// is carry a plan and leave one of that plan's roles unresolved.
///
/// Empty for a link that stands up. A function rather than a test
/// assertion for the reason tapscript's own census checks are functions:
/// a consumer relying on the census being complete is entitled to confirm
/// it over the value it holds.
#[must_use]
pub fn link_role_defects<'census>(
    censuses: impl IntoIterator<
        Item = (
            LiveTransferRepresentationPlan,
            &'census LiveDefinitionCensus,
        ),
    >,
) -> Vec<LiveLinkRole> {
    let mut carried: BTreeSet<LiveTransferRepresentationPlan> = BTreeSet::new();
    let mut filled: BTreeSet<LiveLinkRole> = BTreeSet::new();
    for (plan, census) in censuses {
        carried.insert(plan);
        filled.extend(census.filled_roles());
    }

    LiveLinkRole::ALL
        .iter()
        // A role belonging to a representation this link does not carry
        // is out of scope; every other unfilled role is a defect.
        .filter(|role| {
            !filled.contains(role)
                && role
                    .representation()
                    .is_none_or(|plan| carried.contains(&plan))
        })
        .copied()
        .collect()
}

/// Collect one bundle's half of the live definition census (§11.2).
///
/// Both sources are consulted for every symbol the bundle's table
/// declares, and a symbol both of them define is ambiguous rather than
/// silently taken from the preferred one — which is [`crate::symbol`]'s
/// rule, unchanged, because §11.1 says the symbol model does not move.
///
/// The link's own half — the constructor placements — is added after the
/// tree exists, which is why this returns a census a caller extends
/// rather than a finished one.
///
/// # Errors
///
/// [`LinkRefusal::AmbiguousLiveSymbol`] when both layers define one
/// symbol or when a symbol the programs read from the target is offered a
/// value, [`LinkRefusal::MissingLiveSymbol`] when neither layer defines
/// one, [`LinkRefusal::IncompatibleLiveSymbolType`] when a definition is
/// not the kind the role declares, and
/// [`LinkRefusal::SighashProfileDisagreement`] when the bundle's recorded
/// disposition is not the one this link derives from the reviewed
/// contract.
pub fn collect_live_definitions(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateRelocatableLiveTransferBundle,
    deployment: &LiveLinkDeploymentParameters,
) -> Result<LiveDefinitionCensus, LinkRefusal> {
    let representation = bundle.representation();
    let owner = OwnerParameter::new(bundle.constructor().owner().clone());
    let mut census = LiveDefinitionCensus::default();

    for (symbol, entry) in bundle.symbols() {
        let key = link_symbol(*symbol, representation, &owner);
        let supplied = deployment_value(*symbol, deployment);
        let defined = bundle_value(target, *symbol, bundle, deployment)?;

        // A symbol the programs read from the target has no definition
        // to collect, and a value offered for one is refused rather than
        // ignored: accepting it silently is how a caller comes to
        // believe a commitment was settled here.
        if entry.binding() == SymbolBinding::ReadFromTargetAtSpendTime {
            if supplied.is_some() || defined.is_some() {
                return Err(LinkRefusal::AmbiguousLiveSymbol(Box::new(key)));
            }
            continue;
        }

        let (value, origin) = match (supplied, defined) {
            (Some(_), Some(_)) => return Err(LinkRefusal::AmbiguousLiveSymbol(Box::new(key))),
            (Some(value), None) => (value, LiveDefinitionOrigin::DeploymentParameters),
            (None, Some(value)) => (value, bundle_origin(*symbol)),
            (None, None) => return Err(LinkRefusal::MissingLiveSymbol(Box::new(key))),
        };

        // The bundle's own binding and the origin the definition came
        // from must be the same statement about who settles this symbol.
        let expected_origin = match entry.binding() {
            SymbolBinding::ResolvedAtLink => LiveDefinitionOrigin::DeploymentParameters,
            SymbolBinding::DefinedByBundle | SymbolBinding::ReadFromTargetAtSpendTime => {
                bundle_origin(*symbol)
            }
        };
        if origin != expected_origin {
            return Err(LinkRefusal::AmbiguousLiveSymbol(Box::new(key)));
        }

        census.define(key, value, origin)?;
    }

    // The two roles no bundle table names and no tree is needed for.
    census.define(
        LiveLinkSymbol::OwnerMetadataSchema,
        LiveSymbolValue::OwnerKeyEncoding(Box::new(bundle.constructor().owner_encoding().clone())),
        LiveDefinitionOrigin::Bundle,
    )?;
    census.define(
        LiveLinkSymbol::RepresentationPlan { representation },
        LiveSymbolValue::RepresentationPlan(representation),
        LiveDefinitionOrigin::Bundle,
    )?;

    Ok(census)
}

/// The link key one bundle symbol takes under one representation.
///
/// Total, and crate-visible because the relocation pass needs the same
/// mapping: a second copy of it there would be a second answer to which
/// link symbol a relocation's bundle symbol resolves against.
pub(crate) fn link_symbol(
    symbol: LiveBundleSymbol,
    representation: LiveTransferRepresentationPlan,
    owner: &OwnerParameter,
) -> LiveLinkSymbol {
    match symbol {
        LiveBundleSymbol::ProtocolAsset => LiveLinkSymbol::ProtocolAsset,
        LiveBundleSymbol::ReserveAsset => LiveLinkSymbol::ReserveAsset,
        LiveBundleSymbol::DestinationProgramVersion => LiveLinkSymbol::DestinationProgramVersion,
        LiveBundleSymbol::SponsorChangeProgram => LiveLinkSymbol::SponsorChangeProgram,
        LiveBundleSymbol::SponsorChangeProgramVersion => {
            LiveLinkSymbol::SponsorChangeProgramVersion
        }
        LiveBundleSymbol::TargetFeeRoleProgramDigest => LiveLinkSymbol::TargetFeeRole,
        LiveBundleSymbol::OwnerPublicKey => LiveLinkSymbol::OwnerPublicKey {
            owner: owner.clone(),
        },
        LiveBundleSymbol::UnspendableInternalKey => LiveLinkSymbol::UnspendableInternalKey,
        LiveBundleSymbol::TargetLeafVersion => LiveLinkSymbol::TargetLeafVersion,
        LiveBundleSymbol::SelectedSighashProfile => LiveLinkSymbol::SelectedSighashProfile,
        LiveBundleSymbol::CandidateReceiptInputBound => LiveLinkSymbol::CandidateReceiptInputBound,
        LiveBundleSymbol::CandidateReceiptOutputBound => {
            LiveLinkSymbol::CandidateReceiptOutputBound
        }
        LiveBundleSymbol::CandidateSponsorBound => LiveLinkSymbol::CandidateSponsorBound,
        LiveBundleSymbol::CoordinatorProgram { shape } => LiveLinkSymbol::CoordinatorProgram {
            representation,
            shape,
        },
        LiveBundleSymbol::MemberProgram { receipt_inputs } => LiveLinkSymbol::MemberProgram {
            representation,
            receipt_inputs,
        },
    }
}

/// The origin a bundle-settled symbol's value really comes from.
///
/// Every bundle-settled symbol is the bundle's own except the sighash
/// profile, which is the reviewed contract's classification. The bundle
/// records what it read; the link reads the same source and compares.
const fn bundle_origin(symbol: LiveBundleSymbol) -> LiveDefinitionOrigin {
    match symbol {
        LiveBundleSymbol::SelectedSighashProfile => LiveDefinitionOrigin::ReviewedContract,
        _ => LiveDefinitionOrigin::Bundle,
    }
}

/// The value the deployment parameters settle for one symbol, if any.
fn deployment_value(
    symbol: LiveBundleSymbol,
    deployment: &LiveLinkDeploymentParameters,
) -> Option<LiveSymbolValue> {
    let resolved = deployment.resolved();
    Some(match symbol {
        LiveBundleSymbol::ProtocolAsset => {
            LiveSymbolValue::Asset(resolved.protocol_asset().clone())
        }
        LiveBundleSymbol::ReserveAsset => LiveSymbolValue::Asset(resolved.reserve_asset().clone()),
        LiveBundleSymbol::DestinationProgramVersion => {
            LiveSymbolValue::ScriptNumber(resolved.destination_program_version())
        }
        LiveBundleSymbol::SponsorChangeProgram => {
            LiveSymbolValue::WitnessProgram(resolved.sponsor_change_program().clone())
        }
        LiveBundleSymbol::SponsorChangeProgramVersion => {
            LiveSymbolValue::ScriptNumber(resolved.sponsor_change_version())
        }
        LiveBundleSymbol::TargetFeeRoleProgramDigest => {
            LiveSymbolValue::ProgramDigest(resolved.fee_program_digest().clone())
        }
        LiveBundleSymbol::UnspendableInternalKey => {
            LiveSymbolValue::XOnlyPublicKey(deployment.internal_key().clone())
        }
        _ => return None,
    })
}

/// The value the bundle itself settles for one symbol, if any.
///
/// The leaf programs here are the pre-link ones: this pass fixes the
/// leaf *set*, and the substitution stage rewrites each program, so
/// nothing downstream reads a committed leaf script out of this census.
///
/// # Errors
///
/// [`LinkRefusal::SighashProfileDisagreement`] when the disposition the
/// bundle recorded is not the one this link derives from the reviewed
/// contract. The two are read from the same source, so a disagreement
/// means the bundle and the link are looking at different targets.
fn bundle_value(
    target: &ReviewedElementsTapscriptDefinition,
    symbol: LiveBundleSymbol,
    bundle: &CandidateRelocatableLiveTransferBundle,
    deployment: &LiveLinkDeploymentParameters,
) -> Result<Option<LiveSymbolValue>, LinkRefusal> {
    let constructor = bundle.constructor();
    let bounds = constructor.shapes().bounds();
    Ok(Some(match symbol {
        LiveBundleSymbol::OwnerPublicKey => {
            LiveSymbolValue::XOnlyPublicKey(owner_stack_item(target, constructor.owner())?)
        }
        LiveBundleSymbol::TargetLeafVersion => {
            LiveSymbolValue::LeafVersion(constructor.leaf_version())
        }
        LiveBundleSymbol::SelectedSighashProfile => {
            let selected = deployment.sighash_profile();
            if selected.disposition() != bundle.sighash_profile() {
                return Err(LinkRefusal::SighashProfileDisagreement {
                    bundle: Box::new(bundle.sighash_profile().clone()),
                    link: Box::new(selected.disposition().clone()),
                });
            }
            LiveSymbolValue::SighashProfile(Box::new(selected.clone()))
        }
        LiveBundleSymbol::CandidateReceiptInputBound => {
            LiveSymbolValue::ShapeBound(bounds.receipt_inputs())
        }
        LiveBundleSymbol::CandidateReceiptOutputBound => {
            LiveSymbolValue::ShapeBound(bounds.receipt_outputs())
        }
        LiveBundleSymbol::CandidateSponsorBound => {
            LiveSymbolValue::ShapeBound(bounds.sponsor_inputs())
        }
        LiveBundleSymbol::CoordinatorProgram { shape } => LiveSymbolValue::LeafScript(Box::new(
            bundle
                .leaf(LiveTransferLeafRole::Coordinator {
                    representation: bundle.representation(),
                    shape,
                })
                .ok_or_else(|| {
                    LinkRefusal::MissingLiveSymbol(Box::new(LiveLinkSymbol::CoordinatorProgram {
                        representation: bundle.representation(),
                        shape,
                    }))
                })?
                .program()
                .clone(),
        )),
        LiveBundleSymbol::MemberProgram { receipt_inputs } => {
            LiveSymbolValue::LeafScript(Box::new(
                bundle
                    .leaf(LiveTransferLeafRole::Member {
                        representation: bundle.representation(),
                        receipt_inputs,
                    })
                    .ok_or_else(|| {
                        LinkRefusal::MissingLiveSymbol(Box::new(LiveLinkSymbol::MemberProgram {
                            representation: bundle.representation(),
                            receipt_inputs,
                        }))
                    })?
                    .program()
                    .clone(),
            ))
        }
        _ => return Ok(None),
    }))
}
