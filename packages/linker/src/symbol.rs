//! Pass one of §14.3: the typed definition census.
//!
//! # What a symbol is here
//!
//! The key is the backend's own [`BundleSymbol`], not a name. §14.3
//! states that display strings are not symbol identity, and the
//! cheapest way to honour that is to never mint a string in the first
//! place: the census is a map from a typed enumeration to a typed
//! value, and there is no parse, no lookup by text, and no rendering
//! step anywhere in the resolution path.
//!
//! # Why the type check is separate from the width check
//!
//! A resolution can be the right width and the wrong kind. The x-only
//! public key, the closed asset, the reserve asset, and the fee
//! program digest are all thirty-two bytes under the reviewed
//! contract, so width alone cannot tell an asset supplied as an
//! internal key from one supplied as an asset. [`SymbolType`] is what
//! separates them, and every definition is checked against the type its
//! symbol's role declares before anything is substituted anywhere.

use std::collections::BTreeMap;

use tapscript::{
    BundleSymbol, CandidateRelocatableTapscriptBundle, StackItem, SymbolBinding, TapscriptProgram,
};
use target_elements::LeafVersion;

use crate::deployment::LinkDeploymentParameters;
use crate::error::LinkRefusal;

/// What kind of thing one symbol resolves to.
///
/// Not a width. Several of these share a width under the reviewed
/// contract, which is exactly why the kind is typed separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolType {
    /// An asset identifier compared with an introspected asset field.
    Asset,
    /// The program bytes of a witness-program script.
    WitnessProgram,
    /// A witness-program version, carried as a script number.
    ScriptNumber,
    /// The digest standing in for a script that is not a witness
    /// program.
    ProgramDigest,
    /// An x-only public point bound as the taproot internal key.
    XOnlyPublicKey,
    /// The leaf version every committed leaf carries.
    LeafVersion,
    /// A bound of the candidate shape set.
    ShapeBound,
    /// A complete committed tapscript leaf program.
    LeafScript,
}

/// One symbol's resolved value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolValue {
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
    /// A leaf version.
    LeafVersion(LeafVersion),
    /// A shape bound.
    ShapeBound(u8),
    /// A committed leaf program.
    LeafScript(Box<TapscriptProgram>),
}

impl SymbolValue {
    /// The kind this value is.
    #[must_use]
    pub const fn symbol_type(&self) -> SymbolType {
        match self {
            Self::Asset(_) => SymbolType::Asset,
            Self::WitnessProgram(_) => SymbolType::WitnessProgram,
            Self::ScriptNumber(_) => SymbolType::ScriptNumber,
            Self::ProgramDigest(_) => SymbolType::ProgramDigest,
            Self::XOnlyPublicKey(_) => SymbolType::XOnlyPublicKey,
            Self::LeafVersion(_) => SymbolType::LeafVersion,
            Self::ShapeBound(_) => SymbolType::ShapeBound,
            Self::LeafScript(_) => SymbolType::LeafScript,
        }
    }
}

/// Which layer settled one definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefinitionOrigin {
    /// The caller's public deployment parameters.
    DeploymentParameters,
    /// The relocatable bundle itself.
    Bundle,
}

/// One definition of one symbol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolDefinition {
    symbol: BundleSymbol,
    value: SymbolValue,
    origin: DefinitionOrigin,
}

impl SymbolDefinition {
    /// The symbol this defines.
    #[must_use]
    pub const fn symbol(&self) -> BundleSymbol {
        self.symbol
    }

    /// The resolved value.
    #[must_use]
    pub const fn value(&self) -> &SymbolValue {
        &self.value
    }

    /// Which layer settled it.
    #[must_use]
    pub const fn origin(&self) -> DefinitionOrigin {
        self.origin
    }
}

/// The complete definition census, sorted by stable key.
///
/// A map, so the sort §14.3's first pass calls for is the container's
/// own invariant rather than a step that could be skipped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionCensus {
    definitions: BTreeMap<BundleSymbol, SymbolDefinition>,
}

impl DefinitionCensus {
    /// Every definition, in canonical order.
    #[must_use]
    pub const fn definitions(&self) -> &BTreeMap<BundleSymbol, SymbolDefinition> {
        &self.definitions
    }

    /// One symbol's definition.
    #[must_use]
    pub fn definition(&self, symbol: BundleSymbol) -> Option<&SymbolDefinition> {
        self.definitions.get(&symbol)
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
    pub fn from_origin(&self, origin: DefinitionOrigin) -> impl Iterator<Item = BundleSymbol> {
        self.definitions
            .values()
            .filter(move |definition| definition.origin == origin)
            .map(SymbolDefinition::symbol)
    }
}

/// The type one symbol's role requires of its definition.
#[must_use]
pub const fn declared_type(symbol: BundleSymbol) -> SymbolType {
    match symbol {
        BundleSymbol::ClosedAsset | BundleSymbol::ReserveAsset => SymbolType::Asset,
        BundleSymbol::AshConstructorProgram | BundleSymbol::SponsorChangeProgram => {
            SymbolType::WitnessProgram
        }
        BundleSymbol::SponsorChangeProgramVersion => SymbolType::ScriptNumber,
        BundleSymbol::TargetFeeRoleProgramDigest => SymbolType::ProgramDigest,
        BundleSymbol::UnspendableInternalKey => SymbolType::XOnlyPublicKey,
        BundleSymbol::TargetLeafVersion => SymbolType::LeafVersion,
        BundleSymbol::CandidateAshBound | BundleSymbol::CandidateSponsorBound => {
            SymbolType::ShapeBound
        }
        BundleSymbol::CoordinatorProgram { .. } | BundleSymbol::MemberProgram { .. } => {
            SymbolType::LeafScript
        }
    }
}

/// Pass one: collect every definition, uniquely and by type (§14.3).
///
/// Both sources are consulted for every symbol the bundle's table
/// declares, and a symbol both of them define is ambiguous rather than
/// silently taken from the preferred one. That is the point of the
/// pass: which layer settles a symbol is a fact the bundle states, and
/// a link that quietly overrode it would be resolving against a table
/// nobody wrote.
///
/// # Errors
///
/// [`LinkRefusal::AmbiguousSymbol`] when both layers define one symbol,
/// [`LinkRefusal::MissingSymbol`] when neither does, and
/// [`LinkRefusal::IncompatibleSymbolType`] when a definition is not the
/// kind the symbol's role declares.
pub fn collect_definitions(
    bundle: &CandidateRelocatableTapscriptBundle,
    deployment: &LinkDeploymentParameters,
) -> Result<DefinitionCensus, LinkRefusal> {
    let mut definitions: BTreeMap<BundleSymbol, SymbolDefinition> = BTreeMap::new();

    for (symbol, entry) in bundle.symbols() {
        let supplied = deployment_value(*symbol, deployment);
        let defined = bundle_value(*symbol, bundle);

        // A symbol the programs read from the target has no definition
        // to collect, and that is its whole content. A value offered
        // for one is refused rather than ignored: it would be a
        // link-time resolution for something no site consumes, and
        // accepting it silently is how a caller comes to believe the
        // commitment was settled here.
        if entry.binding() == SymbolBinding::ReadFromTargetAtSpendTime {
            if supplied.is_some() || defined.is_some() {
                return Err(LinkRefusal::AmbiguousSymbol(*symbol));
            }
            continue;
        }

        let (value, origin) = match (supplied, defined) {
            (Some(_), Some(_)) => return Err(LinkRefusal::AmbiguousSymbol(*symbol)),
            (Some(value), None) => (value, DefinitionOrigin::DeploymentParameters),
            (None, Some(value)) => (value, DefinitionOrigin::Bundle),
            (None, None) => return Err(LinkRefusal::MissingSymbol(*symbol)),
        };

        // The bundle's own binding and the origin the definition came
        // from must be the same statement about who settles this
        // symbol. A disagreement means the table and the deployment are
        // describing different link boundaries.
        let expected_origin = match entry.binding() {
            SymbolBinding::ResolvedAtLink => DefinitionOrigin::DeploymentParameters,
            SymbolBinding::DefinedByBundle | SymbolBinding::ReadFromTargetAtSpendTime => {
                DefinitionOrigin::Bundle
            }
        };
        if origin != expected_origin {
            return Err(LinkRefusal::AmbiguousSymbol(*symbol));
        }

        let expected = declared_type(*symbol);
        let actual = value.symbol_type();
        if expected != actual {
            return Err(LinkRefusal::IncompatibleSymbolType {
                symbol: *symbol,
                expected,
                actual,
            });
        }

        if definitions
            .insert(
                *symbol,
                SymbolDefinition {
                    symbol: *symbol,
                    value,
                    origin,
                },
            )
            .is_some()
        {
            return Err(LinkRefusal::DuplicateSymbolDefinition(*symbol));
        }
    }

    Ok(DefinitionCensus { definitions })
}

/// The value the deployment parameters settle for one symbol, if any.
fn deployment_value(
    symbol: BundleSymbol,
    deployment: &LinkDeploymentParameters,
) -> Option<SymbolValue> {
    let resolved = deployment.resolved();
    Some(match symbol {
        BundleSymbol::ClosedAsset => SymbolValue::Asset(resolved.closed_asset().clone()),
        BundleSymbol::ReserveAsset => SymbolValue::Asset(resolved.reserve_asset().clone()),
        BundleSymbol::SponsorChangeProgram => {
            SymbolValue::WitnessProgram(resolved.sponsor_change_program().clone())
        }
        BundleSymbol::SponsorChangeProgramVersion => {
            SymbolValue::ScriptNumber(resolved.sponsor_change_version())
        }
        BundleSymbol::TargetFeeRoleProgramDigest => {
            SymbolValue::ProgramDigest(resolved.fee_program_digest().clone())
        }
        BundleSymbol::UnspendableInternalKey => {
            SymbolValue::XOnlyPublicKey(deployment.internal_key().clone())
        }
        _ => return None,
    })
}

/// The value the bundle itself settles for one symbol, if any.
///
/// The leaf programs here are the pre-link ones. Pass one fixes the
/// leaf *set*; the substitution stage rewrites each leaf's program and
/// the linked programs replace these, which is why nothing downstream
/// reads a leaf script out of this census.
fn bundle_value(
    symbol: BundleSymbol,
    bundle: &CandidateRelocatableTapscriptBundle,
) -> Option<SymbolValue> {
    let constructor = bundle.constructor();
    Some(match symbol {
        BundleSymbol::TargetLeafVersion => SymbolValue::LeafVersion(constructor.leaf_version()),
        BundleSymbol::CandidateAshBound => {
            SymbolValue::ShapeBound(constructor.shapes().bounds().ash_inputs())
        }
        BundleSymbol::CandidateSponsorBound => {
            SymbolValue::ShapeBound(constructor.shapes().bounds().sponsor_inputs())
        }
        BundleSymbol::CoordinatorProgram { shape } => SymbolValue::LeafScript(Box::new(
            constructor
                .leaf(tapscript::LeafRole::Coordinator { shape })?
                .program()
                .clone(),
        )),
        BundleSymbol::MemberProgram { ash_inputs } => SymbolValue::LeafScript(Box::new(
            constructor
                .leaf(tapscript::LeafRole::Member { ash_inputs })?
                .program()
                .clone(),
        )),
        _ => return None,
    })
}
