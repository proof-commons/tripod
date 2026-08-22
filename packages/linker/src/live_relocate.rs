//! Structured live-transfer relocation (§11.1, §13.4).
//!
//! # Substitution before serialization, all the way down
//!
//! [`crate::relocate`]'s discipline, unchanged, because §11.1 says the
//! relocation model does not move: nothing here touches a byte. Each
//! leaf's program is *rebuilt* from the backend's own program builders
//! over the resolved symbol values, which is substitution into the typed
//! program in the strongest available sense — the same code that produced
//! the pre-link program produces the linked one, from a different symbol
//! set.
//!
//! Byte patching could not have worked here either. The sponsor-change
//! witness program is a bounded-width payload, so a resolution can be a
//! different length from the value the bundle was laid out against, and
//! every push after it moves.
//!
//! # The owner does not move, and that is checked rather than assumed
//!
//! The owner key is the one relocated symbol a link does *not* resolve:
//! §7.6 gives a request no parameter through which another key could
//! arrive, so the constructor's committed owner is the value before and
//! after. That makes its sites the sharpest check in the file — the
//! recorded owner sites must carry the constructor's own key in the
//! linked program, so an owner substituted anywhere along the way is a
//! refusal rather than a quietly different program.
//!
//! # Four checks, because rebuilding is not the same as relocating
//!
//! - every recorded site now carries the resolved value, by exact typed
//!   comparison against the census rather than by observing that
//!   something changed;
//! - every instruction that changed is covered by a recorded site, so
//!   the link mutated nothing it did not declare;
//! - the linked program survives the §13.2 round trip, because that
//!   obligation is about the bytes a program serializes to and
//!   substitution changes those bytes;
//! - the linked program still schedules from §10.2's precondition and
//!   still satisfies §10.9, because a substitution that changed a
//!   program's stack behaviour would have produced a leaf the emitter
//!   would never have published.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::upstream::LiveTransferRepresentationPlan;
use tapscript::{
    AbstractLimits, CandidateRelocatableLiveTransferBundle, LiveBundleSymbol, LiveRelocationSite,
    LiveTransferLeafRole, LiveTransferSymbols, StackItem, TapscriptInstruction, TapscriptProgram,
    final_stack_defects, live_coordinator_program, live_member_program, live_program_precondition,
    resource_projection, validate_program,
};
use target_elements::{ResourceDimension, ReviewedElementsTapscriptDefinition};

use crate::error::LinkRefusal;
use crate::live_symbol::{LiveDefinitionCensus, LiveSymbolValue, OwnerParameter, link_symbol};

/// One live-transfer leaf after substitution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedLiveLeafProgram {
    leaf: LiveTransferLeafRole,
    program: TapscriptProgram,
    dimensions: BTreeMap<ResourceDimension, u64>,
    substituted: BTreeSet<LiveBundleSymbol>,
}

impl LinkedLiveLeafProgram {
    /// The leaf's identity.
    #[must_use]
    pub const fn leaf(&self) -> LiveTransferLeafRole {
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
    pub const fn substituted(&self) -> &BTreeSet<LiveBundleSymbol> {
        &self.substituted
    }
}

/// Substitute every resolved symbol into every live leaf (§11.1).
///
/// # Errors
///
/// [`LinkRefusal::UnresolvedLiveRelocation`] when a relocation names a
/// symbol with no definition, [`LinkRefusal::InvalidLinkedLiveProgram`]
/// when a rebuild fails, [`LinkRefusal::LiveRelocationNotApplied`] when a
/// recorded site does not carry the resolved value,
/// [`LinkRefusal::UntrackedLiveProgramMutation`] when an instruction no
/// relocation covers changed,
/// [`LinkRefusal::LiveRoundTripMismatch`] when a linked program does not
/// decode back to itself, and
/// [`LinkRefusal::LinkedLiveProgramDoesNotSchedule`] when a linked
/// program stops satisfying §10.2 or §10.9.
pub fn substitute_live(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateRelocatableLiveTransferBundle,
    census: &LiveDefinitionCensus,
    resolved: &LiveTransferSymbols,
) -> Result<BTreeMap<LiveTransferLeafRole, LinkedLiveLeafProgram>, LinkRefusal> {
    let sites = program_sites(bundle);
    let representation = bundle.representation();
    let owner = OwnerParameter::new(bundle.constructor().owner().clone());
    let mut linked = BTreeMap::new();

    for (leaf, pre_link) in bundle.leaves() {
        let program = rebuild(target, resolved, bundle, *leaf)?;
        let leaf_sites = sites.get(leaf).cloned().unwrap_or_default();

        check_sites(
            target,
            representation,
            &owner,
            *leaf,
            &program,
            &leaf_sites,
            census,
        )?;
        check_untracked(*leaf, pre_link.program(), &program, &leaf_sites)?;
        check_round_trip(target, *leaf, &program)?;
        check_schedule(target, *leaf, &program)?;

        let mut dimensions = resource_projection(target, &program);
        // The witness role is the pre-link one by construction: the
        // builders are the same and only pushed payloads changed, so a
        // program that scheduled from §10.2's one-item precondition
        // still does. The figure is carried across rather than
        // recomputed so the linked and pre-link witness statements
        // cannot drift apart.
        if let Some(initial) = pre_link
            .resources()
            .get(&ResourceDimension::InitialStackItems)
        {
            dimensions.insert(ResourceDimension::InitialStackItems, *initial);
        }

        linked.insert(
            *leaf,
            LinkedLiveLeafProgram {
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
    bundle: &CandidateRelocatableLiveTransferBundle,
) -> BTreeMap<LiveTransferLeafRole, BTreeMap<LiveBundleSymbol, BTreeSet<usize>>> {
    let mut sites: BTreeMap<LiveTransferLeafRole, BTreeMap<LiveBundleSymbol, BTreeSet<usize>>> =
        BTreeMap::new();
    for relocation in bundle.relocations() {
        if let LiveRelocationSite::ProgramInstructions { leaf, indices } = relocation.site() {
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
///
/// The constructor is the bundle's own, unchanged. That is what keeps the
/// owner out of a link's reach: the builders take the owner from the
/// constructor, and this link has no constructor but that one.
fn rebuild(
    target: &ReviewedElementsTapscriptDefinition,
    resolved: &LiveTransferSymbols,
    bundle: &CandidateRelocatableLiveTransferBundle,
    leaf: LiveTransferLeafRole,
) -> Result<TapscriptProgram, LinkRefusal> {
    let constructor = bundle.constructor();
    let built = match leaf {
        LiveTransferLeafRole::Coordinator { shape, .. } => {
            live_coordinator_program(target, resolved, constructor, shape)
        }
        LiveTransferLeafRole::Member { receipt_inputs, .. } => {
            live_member_program(target, resolved, constructor, receipt_inputs)
        }
    };

    built.map_err(|cause| LinkRefusal::InvalidLinkedLiveProgram {
        leaf,
        cause: Box::new(cause),
    })
}

/// Require every recorded site to carry its symbol's resolved value.
fn check_sites(
    target: &ReviewedElementsTapscriptDefinition,
    representation: LiveTransferRepresentationPlan,
    owner: &OwnerParameter,
    leaf: LiveTransferLeafRole,
    program: &TapscriptProgram,
    sites: &BTreeMap<LiveBundleSymbol, BTreeSet<usize>>,
    census: &LiveDefinitionCensus,
) -> Result<(), LinkRefusal> {
    for (symbol, indices) in sites {
        // The same mapping the census was built with, called rather than
        // restated: a second copy of it here would be a second answer to
        // which link symbol a relocation resolves against.
        let key = link_symbol(*symbol, representation, owner);
        let definition = census
            .definition(&key)
            .ok_or_else(|| LinkRefusal::UnresolvedLiveRelocation(Box::new(key.clone())))?;
        let expected = pushed_item(target, definition.value())
            .ok_or_else(|| LinkRefusal::UnresolvedLiveRelocation(Box::new(key)))?;

        for index in indices {
            let carried = match program.instructions().get(*index) {
                Some(TapscriptInstruction::Push(item)) => item.clone(),
                _ => {
                    return Err(LinkRefusal::LiveRelocationNotApplied {
                        leaf,
                        symbol: *symbol,
                    });
                }
            };
            if carried != expected {
                return Err(LinkRefusal::LiveRelocationNotApplied {
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
    value: &LiveSymbolValue,
) -> Option<StackItem> {
    match value {
        LiveSymbolValue::Asset(item)
        | LiveSymbolValue::WitnessProgram(item)
        | LiveSymbolValue::ProgramDigest(item)
        | LiveSymbolValue::XOnlyPublicKey(item) => Some(item.clone()),
        LiveSymbolValue::ScriptNumber(value) => StackItem::script_number(target, *value).ok(),
        LiveSymbolValue::OwnerKeyEncoding(_)
        | LiveSymbolValue::LeafVersion(_)
        | LiveSymbolValue::ShapeBound(_)
        | LiveSymbolValue::LeafScript(_)
        | LiveSymbolValue::RepresentationPlan(_)
        | LiveSymbolValue::SighashProfile(_)
        | LiveSymbolValue::LinkedConstructor(_) => None,
    }
}

/// Require every changed instruction to be covered by a recorded site.
fn check_untracked(
    leaf: LiveTransferLeafRole,
    pre_link: &TapscriptProgram,
    linked: &TapscriptProgram,
    sites: &BTreeMap<LiveBundleSymbol, BTreeSet<usize>>,
) -> Result<(), LinkRefusal> {
    let covered: BTreeSet<usize> = sites.values().flatten().copied().collect();

    // An instruction count that changed is itself an untracked
    // mutation, and it is reported against the first index the two
    // programs stop agreeing at rather than swallowed.
    if pre_link.len() != linked.len() {
        return Err(LinkRefusal::UntrackedLiveProgramMutation {
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
            return Err(LinkRefusal::UntrackedLiveProgramMutation { leaf, index });
        }
    }

    Ok(())
}

/// Require the linked program to survive §13.2's round trip.
fn check_round_trip(
    target: &ReviewedElementsTapscriptDefinition,
    leaf: LiveTransferLeafRole,
    program: &TapscriptProgram,
) -> Result<(), LinkRefusal> {
    let bytes = program.encode(target);
    let decoded = TapscriptProgram::decode(target, &bytes)
        .map_err(|_| LinkRefusal::LiveRoundTripMismatch(leaf))?;
    if &decoded != program {
        return Err(LinkRefusal::LiveRoundTripMismatch(leaf));
    }
    Ok(())
}

/// Require the linked program to still schedule and still satisfy §10.9.
///
/// The emitter held every leaf to both before publishing it, and a link
/// that changed pushed payloads has no business changing either — but
/// "has no business" is not a check, and a leaf that stopped satisfying
/// §10.9 after substitution would be exactly the artifact §10.9 exists
/// to refuse, published by a layer that never looked.
fn check_schedule(
    target: &ReviewedElementsTapscriptDefinition,
    leaf: LiveTransferLeafRole,
    program: &TapscriptProgram,
) -> Result<(), LinkRefusal> {
    let initial = live_program_precondition(target);
    validate_program(
        target,
        program,
        &initial,
        AbstractLimits::for_target(target),
    )
    .map_err(|_| LinkRefusal::LinkedLiveProgramDoesNotSchedule { leaf })?;

    let defects = final_stack_defects(target, program, &initial)
        .map_err(|_| LinkRefusal::LinkedLiveProgramDoesNotSchedule { leaf })?;
    if !defects.is_empty() {
        return Err(LinkRefusal::LinkedLiveProgramFailsTheFinalStackRule { leaf, defects });
    }
    Ok(())
}
