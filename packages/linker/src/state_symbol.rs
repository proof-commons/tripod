//! The maturity link's closed symbol census, resolved in two passes.
//!
//! # What the two passes settle
//!
//! Pass one collects one definition per key from the typed sources the
//! link was given, records which layer is answerable for each, and
//! refuses a second claim on one key. Pass two compares that census
//! against the consumers the composed record and the constructor
//! actually have: every consumer has a definition, every definition is
//! the type its key declares, and every definition has a consumer.
//!
//! The split is not cosmetic. A duplicate is a fact about the definition
//! side alone, so it can be caught while collecting and the collection
//! never continues past it. Missing, unused and mistyped are facts about
//! the two sides together, and none of them can be stated until both
//! sides are complete — a link that checked them while collecting would
//! be comparing a finished consumer census against a census still being
//! built, and would report whichever key happened to be collected last.
//!
//! # Typed sources, never a value map
//!
//! No definition arrives as an entry in a map of names to values. A map
//! erases where a value came from: two byte strings of the right width
//! are indistinguishable inside it, a fixture magnitude reads exactly
//! like a calibrated one, and the check that admitted a value happened
//! somewhere the map cannot name, so the map itself becomes the
//! authority. Every definition here names the layer answerable for it
//! instead, and the value arrives as the type that validated it.
//!
//! # Width and domain are the sources' own
//!
//! Pass one re-checks neither. The singleton's identifier is a
//! thirty-two-byte array, so its width is its type; the lead window is a
//! [`Cycle`] pair the realization refused to build with a zero minimum
//! or an inverted order; the operator key was checked against the
//! reviewed approved encoding when the binding was established; the
//! declared amount is a positive magnitude inside the protocol-amount
//! domain by the type that resolved the asset declaration. A second copy
//! of any of those checks here would be a second authority on one
//! question, free to drift from the first and with no way to tell which
//! had drifted.
//!
//! # Why the shared asset and amount are one key each
//!
//! The structural recognition of input zero and the semantic
//! reconstruction of output zero push the same asset and the same
//! amount, and the composed record already names each of them once by
//! its structural identity. The link's key is derived from the family
//! enums rather than copied from that decision, so both family spellings
//! canonicalize onto one key: they name one deployment fact, and a
//! census with two keys for it could define it twice and inconsistently
//! — a program comparing input zero against one asset and output zero
//! against another would be a covenant nobody wrote.
//!
//! # Why a constructor reference is a key without its value
//!
//! [`StateConstructorReference`] embeds its value in the variant, so it
//! cannot be a key: `MetadataSchema(1)` and `MetadataSchema(2)` would be
//! two different names for one dependency, and a census keyed that way
//! could not say that a schema was defined twice. The kind is the stable
//! identity and the value is stored beside it, which is what lets pass
//! two ask whether one key has one definition.
//!
//! # What is deliberately not a key
//!
//! No owner role and no sighash profile. The reduced announcement leaf
//! reads no owner and selects no profile, so either would be a
//! definition nothing consumes — which pass two would refuse, and
//! rightly: a reserved key is a promise that some later program will
//! read it, made by a layer with no way to keep it.
//!
//! No sponsor-side key either. The leaf carries no change program, no
//! change version, no fee digest and no count bound, so there is nothing
//! for such a key to resolve; a link that offered one would be
//! specializing a deployment to a shape the contract never fixed.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::upstream::{Cycle, StateSingletonDeclaration};
use tapscript::{
    CandidateStateConstructor, StackItem, StateAnnouncementProgram, StateAnnouncementSymbol,
    StateBranchSide, StateConstructorReference, StateInternalKeyPolicy, StateNonceBudget,
    StateOperatorSymbol, StatePatternSymbol, StateProgramSymbol,
};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::state_deployment::StateLinkDeploymentParameters;
use crate::state_error::StateLinkRefusal;

// --- The symbol vocabulary --------------------------------------------

/// One typed key of a maturity announcement link.
///
/// Thirteen: the six the composed program pushes, derived from the three
/// family enums with the shared asset and amount canonicalized, and the
/// seven kinds of constructor reference, whose values are stored beside
/// the key rather than inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateLinkSymbol {
    /// The singleton's asset identifier, compared at input and output
    /// zero.
    StateAsset,
    /// The singleton's explicit amount, compared at input and output
    /// zero.
    StateAmount,
    /// The x-only internal key the metadata tweak is verified against.
    InternalKey,
    /// The minimum announcement lead, in cycles.
    MaturityLeadMin,
    /// The maximum announcement lead, in cycles.
    MaturityLeadMax,
    /// The committed operator public key the leaf authorizes against.
    CommittedOperatorKey,
    /// The canonical metadata schema revision.
    MetadataSchema,
    /// The exact static subtree root.
    StaticSubtreeRoot,
    /// The leaf version every committed leaf carries.
    LeafVersion,
    /// The admitted internal key and its residual policy.
    InternalKeyPolicy,
    /// The fixed outer branch ordering.
    BranchSide,
    /// The host's representation search bound.
    NonceBudget,
    /// The reviewed target contract revision.
    TargetPolicy,
}

impl StateLinkSymbol {
    /// Every key, in declaration order, which is also key order.
    ///
    /// An array rather than a slice, because the census's size is a fact
    /// this type states: a key added without a definition and a consumer
    /// changes this length, and the tests read the length from here
    /// rather than restating it.
    pub const ALL: [Self; 13] = [
        Self::StateAsset,
        Self::StateAmount,
        Self::InternalKey,
        Self::MaturityLeadMin,
        Self::MaturityLeadMax,
        Self::CommittedOperatorKey,
        Self::MetadataSchema,
        Self::StaticSubtreeRoot,
        Self::LeafVersion,
        Self::InternalKeyPolicy,
        Self::BranchSide,
        Self::NonceBudget,
        Self::TargetPolicy,
    ];

    /// The key one composed-program consumer resolves against.
    ///
    /// Total over the family enums and written without a wildcard, so a
    /// symbol added to any of the three families fails to compile here
    /// instead of reaching a key nobody chose. Both spellings of the
    /// shared asset and both of the shared amount land on one key.
    #[must_use]
    pub const fn from_program(symbol: StateProgramSymbol) -> Self {
        match symbol {
            StateProgramSymbol::Structural(StatePatternSymbol::StateAsset)
            | StateProgramSymbol::Semantic(StateAnnouncementSymbol::StateAsset) => Self::StateAsset,
            StateProgramSymbol::Structural(StatePatternSymbol::StateAmount)
            | StateProgramSymbol::Semantic(StateAnnouncementSymbol::StateAmount) => {
                Self::StateAmount
            }
            StateProgramSymbol::Semantic(StateAnnouncementSymbol::InternalKey) => Self::InternalKey,
            StateProgramSymbol::Semantic(StateAnnouncementSymbol::MaturityLeadMin) => {
                Self::MaturityLeadMin
            }
            StateProgramSymbol::Semantic(StateAnnouncementSymbol::MaturityLeadMax) => {
                Self::MaturityLeadMax
            }
            StateProgramSymbol::Operator(StateOperatorSymbol::CommittedOperatorKey) => {
                Self::CommittedOperatorKey
            }
        }
    }

    /// The key one constructor reference resolves against, discarding
    /// the value the variant embeds.
    #[must_use]
    pub const fn from_reference(reference: &StateConstructorReference) -> Self {
        match reference {
            StateConstructorReference::MetadataSchema(_) => Self::MetadataSchema,
            StateConstructorReference::StaticSubtreeRoot(_) => Self::StaticSubtreeRoot,
            StateConstructorReference::LeafVersion(_) => Self::LeafVersion,
            StateConstructorReference::InternalKeyPolicy(_) => Self::InternalKeyPolicy,
            StateConstructorReference::BranchSide(_) => Self::BranchSide,
            StateConstructorReference::NonceBudget(_) => Self::NonceBudget,
            StateConstructorReference::TargetPolicy(_) => Self::TargetPolicy,
        }
    }

    /// Whether some instruction of the composed program pushes this key.
    ///
    /// The six that do are the reduced census the composed record
    /// carries; the seven that do not are constructor policy, consumed
    /// by a field rather than by a push.
    #[must_use]
    pub const fn is_program_symbol(self) -> bool {
        matches!(
            self,
            Self::StateAsset
                | Self::StateAmount
                | Self::InternalKey
                | Self::MaturityLeadMin
                | Self::MaturityLeadMax
                | Self::CommittedOperatorKey
        )
    }
}

// --- Typed values ------------------------------------------------------

/// What kind of thing one key resolves to.
///
/// Not a width. The asset, the internal key and the operator key are all
/// thirty-two bytes under the reviewed contract, so width alone cannot
/// tell an asset supplied as a key from one supplied as an asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateSymbolType {
    /// An asset identifier compared with an introspected asset field.
    Asset,
    /// An explicit amount compared with an introspected value field.
    ExplicitAmount,
    /// An x-only public point.
    XOnlyPublicKey,
    /// An announcement lead, in cycles.
    LeadBound,
    /// A canonical metadata schema revision.
    MetadataSchema,
    /// A static subtree root.
    StaticRoot,
    /// The leaf version every committed leaf carries.
    LeafVersion,
    /// An internal key policy and its residual.
    InternalKeyPolicy,
    /// An outer branch ordering.
    BranchSide,
    /// A representation search bound.
    NonceBudget,
    /// A reviewed target contract revision.
    TargetRevision,
}

/// The type one key's role requires of its definition.
#[must_use]
pub const fn state_declared_type(symbol: StateLinkSymbol) -> StateSymbolType {
    match symbol {
        StateLinkSymbol::StateAsset => StateSymbolType::Asset,
        StateLinkSymbol::StateAmount => StateSymbolType::ExplicitAmount,
        StateLinkSymbol::InternalKey | StateLinkSymbol::CommittedOperatorKey => {
            StateSymbolType::XOnlyPublicKey
        }
        StateLinkSymbol::MaturityLeadMin | StateLinkSymbol::MaturityLeadMax => {
            StateSymbolType::LeadBound
        }
        StateLinkSymbol::MetadataSchema => StateSymbolType::MetadataSchema,
        StateLinkSymbol::StaticSubtreeRoot => StateSymbolType::StaticRoot,
        StateLinkSymbol::LeafVersion => StateSymbolType::LeafVersion,
        StateLinkSymbol::InternalKeyPolicy => StateSymbolType::InternalKeyPolicy,
        StateLinkSymbol::BranchSide => StateSymbolType::BranchSide,
        StateLinkSymbol::NonceBudget => StateSymbolType::NonceBudget,
        StateLinkSymbol::TargetPolicy => StateSymbolType::TargetRevision,
    }
}

/// One key's resolved value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateSymbolValue {
    /// An asset identifier.
    Asset(StackItem),
    /// An explicit amount.
    ExplicitAmount(StackItem),
    /// An x-only public point.
    XOnlyPublicKey(StackItem),
    /// An announcement lead.
    LeadBound(Cycle),
    /// A metadata schema revision.
    MetadataSchema(u32),
    /// A static subtree root.
    StaticRoot([u8; 32]),
    /// A leaf version.
    LeafVersion(LeafVersion),
    /// An internal key policy.
    InternalKeyPolicy(StateInternalKeyPolicy),
    /// An outer branch ordering.
    BranchSide(StateBranchSide),
    /// A representation search bound.
    NonceBudget(StateNonceBudget),
    /// A reviewed target contract revision.
    TargetRevision(TargetContractVersion),
}

impl StateSymbolValue {
    /// The kind this value is.
    #[must_use]
    pub const fn symbol_type(&self) -> StateSymbolType {
        match self {
            Self::Asset(_) => StateSymbolType::Asset,
            Self::ExplicitAmount(_) => StateSymbolType::ExplicitAmount,
            Self::XOnlyPublicKey(_) => StateSymbolType::XOnlyPublicKey,
            Self::LeadBound(_) => StateSymbolType::LeadBound,
            Self::MetadataSchema(_) => StateSymbolType::MetadataSchema,
            Self::StaticRoot(_) => StateSymbolType::StaticRoot,
            Self::LeafVersion(_) => StateSymbolType::LeafVersion,
            Self::InternalKeyPolicy(_) => StateSymbolType::InternalKeyPolicy,
            Self::BranchSide(_) => StateSymbolType::BranchSide,
            Self::NonceBudget(_) => StateSymbolType::NonceBudget,
            Self::TargetRevision(_) => StateSymbolType::TargetRevision,
        }
    }

    /// The literal a push site for this value carries, where a site
    /// exists.
    ///
    /// `None` for the seven constructor kinds, because no instruction
    /// pushes them: they are consumed by a constructor field, and a
    /// value returned here would invite a relocation for a site that
    /// does not exist. The lead bound is built rather than stored,
    /// because the window travels as a validated pair and the leaf reads
    /// each endpoint as an unsigned eight-byte little-endian item.
    #[must_use]
    pub fn push_item(&self, target: &ReviewedElementsTapscriptDefinition) -> Option<StackItem> {
        match self {
            Self::Asset(item) | Self::ExplicitAmount(item) | Self::XOnlyPublicKey(item) => {
                Some(item.clone())
            }
            Self::LeadBound(cycle) => Some(StackItem::unsigned_le64(target, cycle.get())),
            Self::MetadataSchema(_)
            | Self::StaticRoot(_)
            | Self::LeafVersion(_)
            | Self::InternalKeyPolicy(_)
            | Self::BranchSide(_)
            | Self::NonceBudget(_)
            | Self::TargetRevision(_) => None,
        }
    }
}

// --- The definition census ---------------------------------------------

/// Which layer is answerable for one definition.
///
/// Five, because five layers settle things here, and merging any two
/// would make the census say something false about who answers for a
/// value. A deployment's choice, the constructor's own recipe, the
/// reviewed contract, the architecture's lead bounds and the
/// architecture's asset declaration are five different kinds of claim,
/// and the last two are separate because a lead bound requires
/// deployment calibration while a declared issuance is fixed by the
/// declaration itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateDefinitionOrigin {
    /// The deployment: values known only once the object exists.
    Deployment,
    /// The candidate constructor's own typed declarations.
    Constructor,
    /// The reviewed target contract this link is bound to.
    ReviewedTarget,
    /// The architecture-keyed announcement lead bounds, resolved and
    /// validated by the realization.
    ArchitectureBounds,
    /// The architecture's asset declaration for the singleton.
    ArchitectureAsset,
}

/// The issued singleton's asset identifier.
///
/// Deployment data: which asset identifier the issuance produced is not
/// derivable from the architecture, which names the asset but cannot
/// know the identifier a chain assigned it. The width is the type, so
/// there is no width refusal to reach — and a test's bytes here are
/// public, meaningless material standing for no issued asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateSingletonAsset([u8; 32]);

impl StateSingletonAsset {
    /// State the issued identifier.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The exact identifier bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// One definition of one key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateSymbolDefinition {
    symbol: StateLinkSymbol,
    value: StateSymbolValue,
    origin: StateDefinitionOrigin,
}

impl StateSymbolDefinition {
    /// The key this defines.
    #[must_use]
    pub const fn symbol(&self) -> StateLinkSymbol {
        self.symbol
    }

    /// The resolved value.
    #[must_use]
    pub const fn value(&self) -> &StateSymbolValue {
        &self.value
    }

    /// Which layer is answerable for it.
    #[must_use]
    pub const fn origin(&self) -> StateDefinitionOrigin {
        self.origin
    }
}

/// The complete definition census, sorted by stable key.
///
/// A map, so the canonical order pass one calls for is the container's
/// own invariant rather than a step that could be skipped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateDefinitionCensus {
    definitions: BTreeMap<StateLinkSymbol, StateSymbolDefinition>,
}

impl StateDefinitionCensus {
    /// Every definition, in canonical order.
    #[must_use]
    pub const fn definitions(&self) -> &BTreeMap<StateLinkSymbol, StateSymbolDefinition> {
        &self.definitions
    }

    /// One key's definition.
    #[must_use]
    pub fn definition(&self, symbol: StateLinkSymbol) -> Option<&StateSymbolDefinition> {
        self.definitions.get(&symbol)
    }

    /// How many keys the census holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Whether the census is empty, which a resolved one never is.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Every key one origin settled, in canonical order.
    pub fn from_origin(
        &self,
        origin: StateDefinitionOrigin,
    ) -> impl Iterator<Item = StateLinkSymbol> {
        self.definitions
            .values()
            .filter(move |definition| definition.origin == origin)
            .map(StateSymbolDefinition::symbol)
    }

    /// Record one definition, refusing a second claim on one key.
    ///
    /// The type is not checked here. Pass two compares every definition
    /// against the type its key declares, and doing it twice would put
    /// the same judgement in two places, where a later change to one is
    /// invisible to the other.
    ///
    /// # Errors
    ///
    /// [`StateLinkRefusal::DuplicateDefinition`] when the key is already
    /// claimed.
    pub fn define(
        &mut self,
        symbol: StateLinkSymbol,
        value: StateSymbolValue,
        origin: StateDefinitionOrigin,
    ) -> Result<(), StateLinkRefusal> {
        if self.definitions.contains_key(&symbol) {
            return Err(StateLinkRefusal::DuplicateDefinition(symbol));
        }

        self.definitions.insert(
            symbol,
            StateSymbolDefinition {
                symbol,
                value,
                origin,
            },
        );

        Ok(())
    }
}

/// One definition's bytes as the literal the reviewed contract admits.
///
/// The bound is the target's, read through its own constructor rather
/// than restated here.
fn definition_item(
    target: &ReviewedElementsTapscriptDefinition,
    symbol: StateLinkSymbol,
    bytes: &[u8],
) -> Result<StackItem, StateLinkRefusal> {
    StackItem::new(target, bytes.to_vec())
        .map_err(|cause| StateLinkRefusal::InvalidDefinitionItem { symbol, cause })
}

/// Pass one: collect one definition per key from the typed sources.
///
/// Thirteen definitions over five origins. Two of them are not read off
/// a source but checked between two of them first: the declaration must
/// name the asset the validated plan recognizes STATE by, and the
/// constructor's declared revision must be the reviewed one this link is
/// bound to. Each is a fact two sources carry, and agreement is what
/// makes the definition single-valued rather than a preference for
/// whichever source was consulted first.
///
/// The lead bounds' origin says which layer is answerable for the
/// magnitudes — the architecture's bounds, resolved and validated by the
/// realization — and is a different question from whether those
/// magnitudes describe a deployment, which the window's own
/// [`crate::StateLeadBoundOrigin`] answers.
///
/// The internal key is defined twice over, deliberately and under two
/// keys: once as the constructor's policy, which no instruction pushes,
/// and once as the x-only item the semantic components push against the
/// metadata tweak. They are the same fact in two forms, and the census
/// needs both because one is consumed by a field and the other by four
/// push sites.
///
/// # Errors
///
/// [`StateLinkRefusal::SingletonDeclarationMismatch`] when the
/// declaration names another asset,
/// [`StateLinkRefusal::TargetRevisionDisagreement`] when the
/// constructor's revision is not the reviewed one,
/// [`StateLinkRefusal::DuplicateDefinition`] when one key is claimed
/// twice, and [`StateLinkRefusal::InvalidDefinitionItem`] when a value's
/// bytes are not a literal the reviewed contract admits.
pub fn collect_state_definitions(
    target: &ReviewedElementsTapscriptDefinition,
    deployment: &StateLinkDeploymentParameters,
    constructor: &CandidateStateConstructor,
    singleton: &StateSingletonAsset,
    declaration: &StateSingletonDeclaration,
) -> Result<StateDefinitionCensus, StateLinkRefusal> {
    // Two identifiers the architecture owns, compared without naming
    // their type: this crate depends on neither the architecture nor the
    // realization, and comparing two values it was handed needs no name.
    if declaration.asset() != *deployment.plan().state().asset() {
        return Err(StateLinkRefusal::SingletonDeclarationMismatch);
    }

    let mut census = StateDefinitionCensus::default();

    census.define(
        StateLinkSymbol::StateAsset,
        StateSymbolValue::Asset(definition_item(
            target,
            StateLinkSymbol::StateAsset,
            singleton.bytes(),
        )?),
        StateDefinitionOrigin::Deployment,
    )?;

    // The declared issuance as the eight little-endian bytes the leaf
    // compares. The signed reading of those bytes is exact for every
    // magnitude the protocol-amount domain admits, which stops below
    // 2^51, and it preserves the bytes for any value whatever, so there
    // is no conversion refusal to carry.
    census.define(
        StateLinkSymbol::StateAmount,
        StateSymbolValue::ExplicitAmount(StackItem::signed_le64(
            target,
            declaration.fixed_amount().get().cast_signed(),
        )),
        StateDefinitionOrigin::ArchitectureAsset,
    )?;

    census.define(
        StateLinkSymbol::CommittedOperatorKey,
        StateSymbolValue::XOnlyPublicKey(definition_item(
            target,
            StateLinkSymbol::CommittedOperatorKey,
            deployment.operator().key().bytes(),
        )?),
        StateDefinitionOrigin::Deployment,
    )?;

    let bounds = deployment.lead_bounds().bounds();
    census.define(
        StateLinkSymbol::MaturityLeadMin,
        StateSymbolValue::LeadBound(bounds.minimum()),
        StateDefinitionOrigin::ArchitectureBounds,
    )?;
    census.define(
        StateLinkSymbol::MaturityLeadMax,
        StateSymbolValue::LeadBound(bounds.maximum()),
        StateDefinitionOrigin::ArchitectureBounds,
    )?;

    define_constructor_references(&mut census, target, deployment, constructor)?;

    Ok(census)
}

/// Record the constructor's seven declarations and the key they carry.
///
/// Separate from the collector because it is the one source whose
/// definitions are enumerated from the source itself rather than named
/// one by one: the constructor states its dependencies, and this maps
/// each to its key while keeping the value the variant embeds.
///
/// # Errors
///
/// [`StateLinkRefusal::TargetRevisionDisagreement`] when the declared
/// revision is not the reviewed one,
/// [`StateLinkRefusal::DuplicateDefinition`] when a kind is declared
/// twice, and [`StateLinkRefusal::InvalidDefinitionItem`] when the
/// internal key's bytes are not a literal the reviewed contract admits.
fn define_constructor_references(
    census: &mut StateDefinitionCensus,
    target: &ReviewedElementsTapscriptDefinition,
    deployment: &StateLinkDeploymentParameters,
    constructor: &CandidateStateConstructor,
) -> Result<(), StateLinkRefusal> {
    for declared in constructor.reference_declarations() {
        let symbol = StateLinkSymbol::from_reference(&declared.reference);
        match declared.reference {
            StateConstructorReference::MetadataSchema(schema) => census.define(
                symbol,
                StateSymbolValue::MetadataSchema(schema),
                StateDefinitionOrigin::Constructor,
            )?,
            StateConstructorReference::StaticSubtreeRoot(root) => census.define(
                symbol,
                StateSymbolValue::StaticRoot(root),
                StateDefinitionOrigin::Constructor,
            )?,
            StateConstructorReference::LeafVersion(version) => census.define(
                symbol,
                StateSymbolValue::LeafVersion(version),
                StateDefinitionOrigin::Constructor,
            )?,
            StateConstructorReference::InternalKeyPolicy(policy) => {
                census.define(
                    symbol,
                    StateSymbolValue::InternalKeyPolicy(policy),
                    StateDefinitionOrigin::Constructor,
                )?;
                census.define(
                    StateLinkSymbol::InternalKey,
                    StateSymbolValue::XOnlyPublicKey(definition_item(
                        target,
                        StateLinkSymbol::InternalKey,
                        policy.key(),
                    )?),
                    StateDefinitionOrigin::Constructor,
                )?;
            }
            StateConstructorReference::BranchSide(side) => census.define(
                symbol,
                StateSymbolValue::BranchSide(side),
                StateDefinitionOrigin::Constructor,
            )?,
            StateConstructorReference::NonceBudget(budget) => census.define(
                symbol,
                StateSymbolValue::NonceBudget(budget),
                StateDefinitionOrigin::Constructor,
            )?,
            StateConstructorReference::TargetPolicy(revision) => {
                let reviewed = deployment.revision();
                if revision != reviewed {
                    return Err(StateLinkRefusal::TargetRevisionDisagreement {
                        constructor: revision,
                        reviewed,
                    });
                }
                census.define(
                    symbol,
                    StateSymbolValue::TargetRevision(reviewed),
                    StateDefinitionOrigin::ReviewedTarget,
                )?;
            }
        }
    }

    Ok(())
}

// --- The consumer census ----------------------------------------------

/// Where one key is consumed.
///
/// Both halves, because a key consumed by a constructor field and a key
/// consumed by a push site are consumed in different senses and a census
/// that reported only sites would call the seven constructor kinds
/// unused. A constructor-policy consumer counts even though it produces
/// no script push.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateConsumerSites {
    record_sites: BTreeSet<usize>,
    constructor_field: bool,
}

impl StateConsumerSites {
    /// State where one key is consumed.
    #[must_use]
    pub const fn new(record_sites: BTreeSet<usize>, constructor_field: bool) -> Self {
        Self {
            record_sites,
            constructor_field,
        }
    }

    /// Every instruction of the composed program that pushes this key.
    #[must_use]
    pub const fn record_sites(&self) -> &BTreeSet<usize> {
        &self.record_sites
    }

    /// Whether a constructor field consumes this key.
    #[must_use]
    pub const fn constructor_field(&self) -> bool {
        self.constructor_field
    }
}

/// Every key some source consumes, sorted by stable key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateConsumerCensus {
    consumers: BTreeMap<StateLinkSymbol, StateConsumerSites>,
}

impl StateConsumerCensus {
    /// Read the consumers off the composed record and the constructor.
    ///
    /// The record's own census already names the shared asset and amount
    /// once each, by their structural identity. This reads it through
    /// [`StateLinkSymbol::from_program`] and unions the sites anyway, so
    /// the link's key does not depend on that upstream choice: were the
    /// record to name both spellings, one key would still carry every
    /// site, and a change there would move a site count rather than
    /// silently split a key in two.
    #[must_use]
    pub fn from_sources(
        record: &StateAnnouncementProgram,
        constructor: &CandidateStateConstructor,
    ) -> Self {
        let mut consumers: BTreeMap<StateLinkSymbol, StateConsumerSites> = BTreeMap::new();

        for (symbol, consumer) in record.consumers() {
            consumers
                .entry(StateLinkSymbol::from_program(*symbol))
                .or_default()
                .record_sites
                .extend(&consumer.sites);
        }

        for declared in constructor.reference_declarations() {
            consumers
                .entry(StateLinkSymbol::from_reference(&declared.reference))
                .or_default()
                .constructor_field = true;
        }

        Self { consumers }
    }

    /// State a consumer census directly.
    #[must_use]
    pub const fn new(consumers: BTreeMap<StateLinkSymbol, StateConsumerSites>) -> Self {
        Self { consumers }
    }

    /// Every consumed key, in canonical order.
    #[must_use]
    pub const fn consumers(&self) -> &BTreeMap<StateLinkSymbol, StateConsumerSites> {
        &self.consumers
    }

    /// How many instructions of the composed program push a consumed
    /// key.
    #[must_use]
    pub fn push_site_count(&self) -> usize {
        self.consumers
            .values()
            .map(|sites| sites.record_sites.len())
            .sum()
    }
}

// --- Pass two ----------------------------------------------------------

/// One key's definition together with where it is consumed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateResolvedEntry {
    definition: StateSymbolDefinition,
    sites: StateConsumerSites,
}

impl StateResolvedEntry {
    /// The definition.
    #[must_use]
    pub const fn definition(&self) -> &StateSymbolDefinition {
        &self.definition
    }

    /// Where the key is consumed.
    #[must_use]
    pub const fn sites(&self) -> &StateConsumerSites {
        &self.sites
    }
}

/// The resolved census: every key, its definition and its consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateResolvedCensus {
    entries: BTreeMap<StateLinkSymbol, StateResolvedEntry>,
}

impl StateResolvedCensus {
    /// Every resolved key, in canonical order.
    #[must_use]
    pub const fn entries(&self) -> &BTreeMap<StateLinkSymbol, StateResolvedEntry> {
        &self.entries
    }

    /// How many instructions of the composed program push a resolved
    /// key.
    #[must_use]
    pub fn push_site_count(&self) -> usize {
        self.entries
            .values()
            .map(|entry| entry.sites.record_sites.len())
            .sum()
    }

    /// Every resolved key some instruction pushes.
    #[must_use]
    pub fn program_keys(&self) -> BTreeSet<StateLinkSymbol> {
        self.entries
            .keys()
            .copied()
            .filter(|symbol| symbol.is_program_symbol())
            .collect()
    }

    /// Every resolved key a constructor field consumes.
    #[must_use]
    pub fn constructor_keys(&self) -> BTreeSet<StateLinkSymbol> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.sites.constructor_field)
            .map(|(symbol, _)| *symbol)
            .collect()
    }
}

/// Pass two: resolve the census against what actually consumes it.
///
/// Three checks in a fixed order, because each is only meaningful once
/// the one before it holds. A consumer without a definition is checked
/// first: nothing else can be said about a key whose value is absent. A
/// definition of the wrong type is checked next, over definitions that
/// exist. A definition without a consumer is checked last, and it is the
/// census's closure statement — an enum variant does not disappear when
/// a push site does, so a record that stopped consuming a key is exactly
/// what this catches, where the two earlier checks would pass.
///
/// # Errors
///
/// [`StateLinkRefusal::MissingDefinition`] for a consumed key nothing
/// defines, [`StateLinkRefusal::IncompatibleType`] for a definition that
/// is not the kind its key declares, and
/// [`StateLinkRefusal::UnusedDefinition`] for a definition nothing
/// consumes.
pub fn resolve_state_census(
    census: &StateDefinitionCensus,
    consumers: &StateConsumerCensus,
) -> Result<StateResolvedCensus, StateLinkRefusal> {
    let mut entries: BTreeMap<StateLinkSymbol, StateResolvedEntry> = BTreeMap::new();

    for (&symbol, sites) in consumers.consumers() {
        let definition = census
            .definition(symbol)
            .ok_or(StateLinkRefusal::MissingDefinition(symbol))?;

        let declared = state_declared_type(symbol);
        let offered = definition.value.symbol_type();
        if declared != offered {
            return Err(StateLinkRefusal::IncompatibleType {
                symbol,
                declared,
                offered,
            });
        }

        entries.insert(
            symbol,
            StateResolvedEntry {
                definition: definition.clone(),
                sites: sites.clone(),
            },
        );
    }

    for &symbol in census.definitions().keys() {
        if !consumers.consumers().contains_key(&symbol) {
            return Err(StateLinkRefusal::UnusedDefinition(symbol));
        }
    }

    Ok(StateResolvedCensus { entries })
}
