//! Structured relocation, and the round trip it does not inherit.
//!
//! # Substitution before serialization, all the way down
//!
//! §13.4 prefers structured substitution and prohibits variable-width
//! byte patching. This module takes the preference to its conclusion:
//! nothing here touches a byte. Each leaf's program is *rebuilt* from
//! the backend's own program builders over the resolved symbol values,
//! which is substitution into the typed program in the strongest
//! available sense — the same code that produced the pre-link program
//! produces the linked one, from a different symbol set.
//!
//! Byte-level patching could not have worked in any case. Two of the
//! resolved symbols are value-determined widths: a witness program is a
//! bounded-width payload rather than a fixed one, so a resolution can
//! be a different length from the value the bundle was laid out
//! against, and every push after it moves. That is exactly the
//! situation §13.4 prohibits patching in.
//!
//! # Three checks, because rebuilding is not the same as relocating
//!
//! Rebuilding could silently do more or less than the recorded
//! relocations describe, so each linked program is compared with the
//! pre-link one instruction by instruction:
//!
//! - every recorded site now carries the resolved value, by exact
//!   typed comparison against the definition rather than by observing
//!   that something changed;
//! - every instruction that changed is covered by a recorded site, so
//!   the link mutated nothing it did not declare;
//! - the linked program survives the §13.2 round trip, because that
//!   obligation is about the bytes a program serializes to, and
//!   substitution changes those bytes.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::{
    BundleSymbol, CandidateRelocatableTapscriptBundle, CompactAshSymbols, LeafRole,
    ProgramResources, RelocationSite, StackItem, TapscriptInstruction, TapscriptProgram,
    coordinator_program, member_program, resource_projection,
};
use target_elements::{ResourceDimension, ReviewedElementsTapscriptDefinition};

use crate::error::LinkRefusal;
use crate::symbol::{DefinitionCensus, SymbolValue};

/// One leaf after substitution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedLeafProgram {
    leaf: LeafRole,
    program: TapscriptProgram,
    dimensions: BTreeMap<ResourceDimension, u64>,
    substituted: BTreeSet<BundleSymbol>,
}

impl LinkedLeafProgram {
    /// The leaf's identity.
    #[must_use]
    pub const fn leaf(&self) -> LeafRole {
        self.leaf
    }

    /// The linked typed program.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }

    /// The exact re-derived resource projection, in canonical order.
    #[must_use]
    pub const fn dimensions(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.dimensions
    }

    /// One dimension's linked figure.
    #[must_use]
    pub fn charged(&self, dimension: ResourceDimension) -> Option<u64> {
        self.dimensions.get(&dimension).copied()
    }

    /// Every symbol substituted into this leaf, in canonical order.
    #[must_use]
    pub const fn substituted(&self) -> &BTreeSet<BundleSymbol> {
        &self.substituted
    }
}

/// Substitute every resolved symbol into every leaf, and re-serialize.
///
/// # Errors
///
/// [`LinkRefusal::UnresolvedRelocation`] when a relocation names a
/// symbol with no definition, [`LinkRefusal::InvalidLinkedProgram`]
/// when a rebuild fails, [`LinkRefusal::RelocationNotApplied`] when a
/// recorded site does not carry the resolved value,
/// [`LinkRefusal::UntrackedProgramMutation`] when an instruction no
/// relocation covers changed, and [`LinkRefusal::RoundTripMismatch`]
/// when a linked program does not decode back to itself.
pub fn substitute(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateRelocatableTapscriptBundle,
    census: &DefinitionCensus,
    resolved: &CompactAshSymbols,
) -> Result<BTreeMap<LeafRole, LinkedLeafProgram>, LinkRefusal> {
    let sites = program_sites(bundle);
    let mut linked = BTreeMap::new();

    for (leaf, pre_link) in bundle.constructor().leaves() {
        let program = rebuild(target, resolved, *leaf, bundle)?;
        let leaf_sites = sites.get(leaf).cloned().unwrap_or_default();

        check_sites(target, *leaf, &program, &leaf_sites, census)?;
        check_untracked(*leaf, pre_link.program(), &program, &leaf_sites)?;
        check_round_trip(target, *leaf, &program)?;

        let mut dimensions = resource_projection(target, &program);
        // The witness role is the pre-link one by construction: the
        // builders are the same and only pushed payloads changed, so a
        // program that scheduled from the empty stack still does. The
        // figure is carried across rather than recomputed so the linked
        // and pre-link witness statements cannot drift apart.
        if let Some(initial) = pre_link
            .resources()
            .charged(ResourceDimension::InitialStackItems)
        {
            dimensions.insert(ResourceDimension::InitialStackItems, initial);
        }

        linked.insert(
            *leaf,
            LinkedLeafProgram {
                leaf: *leaf,
                program,
                dimensions,
                substituted: leaf_sites.keys().copied().collect(),
            },
        );
    }

    Ok(linked)
}

/// Every leaf's recorded relocation sites, by symbol.
fn program_sites(
    bundle: &CandidateRelocatableTapscriptBundle,
) -> BTreeMap<LeafRole, BTreeMap<BundleSymbol, BTreeSet<usize>>> {
    let mut sites: BTreeMap<LeafRole, BTreeMap<BundleSymbol, BTreeSet<usize>>> = BTreeMap::new();
    for relocation in bundle.relocations() {
        if let RelocationSite::ProgramInstructions { leaf, indices } = relocation.site() {
            sites
                .entry(*leaf)
                .or_default()
                .entry(relocation.symbol())
                .or_default()
                .extend(indices.iter().copied());
        }
    }
    sites
}

/// One leaf's program, rebuilt over the resolved symbols.
fn rebuild(
    target: &ReviewedElementsTapscriptDefinition,
    resolved: &CompactAshSymbols,
    leaf: LeafRole,
    bundle: &CandidateRelocatableTapscriptBundle,
) -> Result<TapscriptProgram, LinkRefusal> {
    let program = match leaf {
        LeafRole::Coordinator { shape } => coordinator_program(target, resolved, shape),
        LeafRole::Member { ash_inputs } => {
            // A member program is a function of the batch size alone,
            // so any admitted shape with that batch size rebuilds it.
            // The bundle's own sharing proof is what makes that true,
            // and it was established before this link began.
            let shape = bundle
                .shapes()
                .shapes()
                .find(|shape| shape.ash_inputs() == ash_inputs);
            match shape {
                Some(shape) => member_program(target, resolved, shape),
                None => {
                    return Err(LinkRefusal::RelocationNotApplied {
                        leaf,
                        symbol: BundleSymbol::MemberProgram { ash_inputs },
                    });
                }
            }
        }
    };

    program.map_err(|cause| LinkRefusal::InvalidLinkedProgram { leaf, cause })
}

/// Require every recorded site to carry its symbol's resolved value.
fn check_sites(
    target: &ReviewedElementsTapscriptDefinition,
    leaf: LeafRole,
    program: &TapscriptProgram,
    sites: &BTreeMap<BundleSymbol, BTreeSet<usize>>,
    census: &DefinitionCensus,
) -> Result<(), LinkRefusal> {
    for (symbol, indices) in sites {
        let definition = census
            .definition(*symbol)
            .ok_or(LinkRefusal::UnresolvedRelocation(*symbol))?;
        let expected = pushed_item(target, definition.value())
            .ok_or(LinkRefusal::UnresolvedRelocation(*symbol))?;

        for index in indices {
            let carried = match program.instructions().get(*index) {
                Some(TapscriptInstruction::Push(item)) => item.clone(),
                _ => {
                    return Err(LinkRefusal::RelocationNotApplied {
                        leaf,
                        symbol: *symbol,
                    });
                }
            };
            if carried != expected {
                return Err(LinkRefusal::RelocationNotApplied {
                    leaf,
                    symbol: *symbol,
                });
            }
        }
    }

    Ok(())
}

/// The exact item a resolved value is pushed as, where it is pushed.
fn pushed_item(
    target: &ReviewedElementsTapscriptDefinition,
    value: &SymbolValue,
) -> Option<StackItem> {
    match value {
        SymbolValue::Asset(item)
        | SymbolValue::WitnessProgram(item)
        | SymbolValue::ProgramDigest(item)
        | SymbolValue::XOnlyPublicKey(item) => Some(item.clone()),
        SymbolValue::ScriptNumber(value) => StackItem::script_number(target, *value).ok(),
        SymbolValue::LeafVersion(_) | SymbolValue::ShapeBound(_) | SymbolValue::LeafScript(_) => {
            None
        }
    }
}

/// Require every changed instruction to be covered by a recorded site.
fn check_untracked(
    leaf: LeafRole,
    pre_link: &TapscriptProgram,
    linked: &TapscriptProgram,
    sites: &BTreeMap<BundleSymbol, BTreeSet<usize>>,
) -> Result<(), LinkRefusal> {
    let covered: BTreeSet<usize> = sites.values().flatten().copied().collect();

    // An instruction count that changed is itself an untracked
    // mutation, and it is reported against the first index the two
    // programs stop agreeing at rather than swallowed.
    if pre_link.len() != linked.len() {
        return Err(LinkRefusal::UntrackedProgramMutation {
            leaf,
            index: pre_link.len().min(linked.len()),
        });
    }

    for (index, (before, after)) in pre_link
        .instructions()
        .iter()
        .zip(linked.instructions())
        .enumerate()
    {
        if before != after && !covered.contains(&index) {
            return Err(LinkRefusal::UntrackedProgramMutation { leaf, index });
        }
    }

    Ok(())
}

/// Require the linked program to survive §13.2's round trip.
fn check_round_trip(
    target: &ReviewedElementsTapscriptDefinition,
    leaf: LeafRole,
    program: &TapscriptProgram,
) -> Result<(), LinkRefusal> {
    let bytes = program.encode(target);
    let decoded = TapscriptProgram::decode(target, &bytes)
        .map_err(|_| LinkRefusal::RoundTripMismatch(leaf))?;
    if &decoded != program {
        return Err(LinkRefusal::RoundTripMismatch(leaf));
    }
    Ok(())
}

/// The pre-link resources of one leaf, for comparison reporting.
#[must_use]
pub fn pre_link_dimensions(resources: &ProgramResources) -> BTreeMap<ResourceDimension, u64> {
    resources.dimensions().clone()
}
