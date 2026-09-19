//! The maturity leaf relocated from its resolved census, one record per
//! occurrence.
//!
//! # A record per occurrence is what makes the census checkable
//!
//! Every mandatory occurrence of a resolved key carries exactly one
//! relocation record, and the two failures that statement excludes are
//! different failures. An occurrence without a record is a fixture that
//! survives the link: the composition's own placeholder reaches a
//! deployment inside a program that claims to be linked. A record
//! without an occurrence is the mirror image: a value the link claims to
//! have placed and did not, counted in the census, absent from the
//! bytes. Only a one-to-one census can say that neither happened, which
//! is why the size of this census is a fact the tests read rather than a
//! total the module reports about itself.
//!
//! # Discovered from the census, cross-checked by one-symbol rebuilding
//!
//! Discovery reads the resolved census's absolute sites; it never
//! searches the program for bytes. A search would find coincidences — a
//! thirty-two-byte value can match bytes that are not a site at all, and
//! nothing in the bytes says which of the matches was a consumer.
//!
//! Rebuilding alone proves the opposite half and no more. A program
//! rebuilt from one key's value shows which instructions that value
//! moves, and says nothing about whether the census named them: a census
//! that omitted a site and a census that invented one both survive a
//! rebuild nobody compared against them.
//!
//! So both run, and each catches what the other misses. For every key,
//! the pristine program is rebuilt with that key's linked value alone,
//! and the indices that move must be exactly the census's sites for it
//! when the linked value differs from what the record says those sites
//! carried, and none at all when it does not. The record's own consumer
//! census is the reference, because it is the record's statement of what
//! each site carried; a census pointing a key at a push carrying
//! something else is refused here rather than linked.
//!
//! # Push sites, because a field occurrence has no instruction
//!
//! The seven constructor keys are consumed by a constructor field and no
//! instruction pushes them, so they carry no relocation record: a record
//! for one would name a site that does not exist, and the census's size
//! would stop meaning what it means for the six keys the program does
//! push.
//!
//! # One rebuild, with every value together, then reparsed and
//! revalidated
//!
//! The linked program is built once from the pristine instructions with
//! every linked value placed at its site. It is not patched, and it is
//! not built in passes: a program patched in place is a program nobody
//! stated, and a program assembled by repeated partial substitution
//! passes through intermediate forms that no census describes. What is
//! then checked is the finished artifact — every site carries its linked
//! value, nothing outside the sites moved, the bytes decode back to the
//! same program, and the walk from the record's own precondition reaches
//! the record's own execution.
//!
//! The last of those is an equality rather than a fresh judgement, and
//! it can be, because every substitution here preserves width and the
//! abstract walk types a literal by its width alone. An execution that
//! moved therefore means the program's shape moved, which is a defect in
//! the substitution rather than a property of the deployment.
//!
//! # A shared component alias counts a physical site once
//!
//! A site is attributed to the component whose range holds it. Where
//! more than one range holds it, the record names the first in component
//! order and keeps the rest beside it, and the site is still one
//! relocation: the census counts instructions, not memberships, and a
//! site counted twice would make the leaf look as though it pushed a
//! value it pushes once.
//!
//! # The witnessed static root has no relocation at all
//!
//! The root is bound at spend time by the tweak equation the semantic
//! authentication verifies, which is why the constructor's key for it
//! resolves to no pushable item. A literal beneath the program
//! committing to that root would be a self-commitment with no
//! authenticated cut — the program's bytes would have to contain the
//! hash of the tree those same bytes are committed in — so this module
//! refuses such a relocation rather than searching for the fixed point
//! it would need.

use std::collections::BTreeSet;

use tapscript::{
    AbstractExecutionResult, AbstractLimits, StackItem, StateAnnouncementProgram, StateLeafRole,
    StateProgramComponent, TapscriptInstruction, TapscriptProgram, validate_program,
};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::state_error::StateLinkRefusal;
use crate::state_symbol::{
    StateLinkSymbol, StateResolvedCensus, StateSymbolType, state_declared_type,
};

// --- One relocation ----------------------------------------------------

/// What one substitution costs, in the one dimension it can move.
///
/// Script bytes alone. A push's cost in the other dimensions is not a
/// function of its width under the reviewed contract — the operation
/// budget charges nothing for any primitive and the validation budget
/// charges only signature and curve checks — so a delta stated for one
/// of them would be a claim with no way to be false. The width is
/// nonetheless measured rather than assumed equal, because a link that
/// changed a width is exactly the link this record would have to show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateResourceDelta {
    script_bytes: i64,
}

impl StateResourceDelta {
    /// The linked push's exact encoded width minus the pristine one's.
    #[must_use]
    pub const fn script_bytes(self) -> i64 {
        self.script_bytes
    }
}

/// One mandatory occurrence of one resolved key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRelocation {
    symbol: StateLinkSymbol,
    expected: StateSymbolType,
    component: StateProgramComponent,
    aliases: BTreeSet<StateProgramComponent>,
    leaf: StateLeafRole,
    site: usize,
    pre_value: StackItem,
    linked_value: StackItem,
    delta: StateResourceDelta,
}

impl StateRelocation {
    /// The key this occurrence resolves against.
    #[must_use]
    pub const fn symbol(&self) -> StateLinkSymbol {
        self.symbol
    }

    /// The type that key's role requires.
    #[must_use]
    pub const fn expected(&self) -> StateSymbolType {
        self.expected
    }

    /// The component whose range holds the site.
    #[must_use]
    pub const fn component(&self) -> StateProgramComponent {
        self.component
    }

    /// Every further component whose range also holds it.
    ///
    /// Empty where the ranges partition the program, which is what the
    /// composition produces today; the field exists because the
    /// attribution is the composition's to change and a shared range
    /// must still count the site once.
    #[must_use]
    pub const fn aliases(&self) -> &BTreeSet<StateProgramComponent> {
        &self.aliases
    }

    /// The leaf the occurrence belongs to.
    #[must_use]
    pub const fn leaf(&self) -> StateLeafRole {
        self.leaf
    }

    /// The absolute instruction index in the composed program.
    #[must_use]
    pub const fn site(&self) -> usize {
        self.site
    }

    /// The literal the pristine program pushes there.
    #[must_use]
    pub const fn pre_value(&self) -> &StackItem {
        &self.pre_value
    }

    /// The literal the link places there.
    #[must_use]
    pub const fn linked_value(&self) -> &StackItem {
        &self.linked_value
    }

    /// What the substitution costs.
    #[must_use]
    pub const fn delta(&self) -> StateResourceDelta {
        self.delta
    }
}

/// Every relocation of one leaf, in site order.
///
/// A vector rather than a map, because two keys may not share a site but
/// the order that matters is the program's: a reader checking a census
/// against a program walks the program forwards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRelocationCensus {
    relocations: Vec<StateRelocation>,
}

impl StateRelocationCensus {
    /// Every relocation, in site order.
    #[must_use]
    pub fn relocations(&self) -> &[StateRelocation] {
        &self.relocations
    }

    /// How many occurrences the census holds.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.relocations.len()
    }

    /// Whether the census holds none, which a resolved leaf's never
    /// does.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.relocations.is_empty()
    }

    /// One key's occurrences, in site order.
    pub fn by_symbol(&self, symbol: StateLinkSymbol) -> impl Iterator<Item = &StateRelocation> {
        self.relocations
            .iter()
            .filter(move |relocation| relocation.symbol == symbol)
    }

    /// Every relocated instruction index.
    #[must_use]
    pub fn sites(&self) -> BTreeSet<usize> {
        self.relocations
            .iter()
            .map(|relocation| relocation.site)
            .collect()
    }

    /// Every key some occurrence names, in key order.
    #[must_use]
    pub fn symbols(&self) -> BTreeSet<StateLinkSymbol> {
        self.relocations
            .iter()
            .map(|relocation| relocation.symbol)
            .collect()
    }
}

// --- Discovery ---------------------------------------------------------

/// Read every relocation off the resolved census and cross-check it.
///
/// The seven keys a constructor field consumes are skipped rather than
/// refused: they have no record site, so there is no occurrence for a
/// record to describe.
///
/// # Errors
///
/// [`StateLinkRefusal::LiteralStaticRootRelocation`] when the witnessed
/// root resolves to a pushable item,
/// [`StateLinkRefusal::UnresolvedRelocation`] when a pushed key resolves
/// to none, [`StateLinkRefusal::SiteIsNotAPush`] when a recorded site is
/// not a push in the pristine program,
/// [`StateLinkRefusal::SiteOutsideEveryComponent`] when it lies in no
/// component range, [`StateLinkRefusal::ResourceDeltaOverflow`] when the
/// two widths cannot be subtracted, and
/// [`StateLinkRefusal::RelocationCensusDisagreement`] when one key's
/// sites and the instructions its substitution moves disagree.
pub fn discover_state_relocations(
    target: &ReviewedElementsTapscriptDefinition,
    record: &StateAnnouncementProgram,
    resolved: &StateResolvedCensus,
) -> Result<StateRelocationCensus, StateLinkRefusal> {
    let pristine = record.program().instructions();
    let mut relocations: Vec<StateRelocation> = Vec::new();

    for (&symbol, entry) in resolved.entries() {
        let sites = entry.sites().record_sites();
        if sites.is_empty() {
            continue;
        }

        let linked = entry.definition().value().push_item(target);
        if symbol == StateLinkSymbol::StaticSubtreeRoot && linked.is_some() {
            return Err(StateLinkRefusal::LiteralStaticRootRelocation);
        }
        let linked = linked.ok_or(StateLinkRefusal::UnresolvedRelocation(symbol))?;

        for &site in sites {
            let Some(TapscriptInstruction::Push(pre_value)) = pristine.get(site) else {
                return Err(StateLinkRefusal::SiteIsNotAPush { symbol, site });
            };
            let (component, aliases) = attribution(record, site)
                .ok_or(StateLinkRefusal::SiteOutsideEveryComponent { symbol, site })?;

            relocations.push(StateRelocation {
                symbol,
                expected: state_declared_type(symbol),
                component,
                aliases,
                leaf: StateLeafRole::Announcement,
                site,
                pre_value: pre_value.clone(),
                linked_value: linked.clone(),
                delta: delta(target, symbol, site, pre_value, &linked)?,
            });
        }

        cross_check(record, symbol, sites, &linked)?;
    }

    relocations.sort_by_key(|relocation| relocation.site);
    Ok(StateRelocationCensus { relocations })
}

/// The component whose range holds one site, and every further one that
/// also does.
fn attribution(
    record: &StateAnnouncementProgram,
    site: usize,
) -> Option<(StateProgramComponent, BTreeSet<StateProgramComponent>)> {
    let mut holders = record
        .components()
        .iter()
        .filter(|(_, range)| range.contains(&site))
        .map(|(component, _)| *component);

    let first = holders.next()?;
    Some((first, holders.collect()))
}

/// The record's own statement of what one key's sites carried.
///
/// The composed record names each link key once, because both family
/// spellings of the shared asset and amount canonicalize onto one key
/// and the record's own census already agrees on one item per consumer.
fn reference_item(
    record: &StateAnnouncementProgram,
    symbol: StateLinkSymbol,
) -> Option<&StackItem> {
    record
        .consumers()
        .iter()
        .find(|(consumed, _)| StateLinkSymbol::from_program(**consumed) == symbol)
        .map(|(_, consumer)| &consumer.item)
}

/// Rebuild the pristine program with one key's value alone and compare.
///
/// A key whose sites the record's consumer census does not carry is a
/// disagreement rather than a case needing a second reference: those
/// sites were supposed to have come from that census, so a key absent
/// from it claims occurrences the record never recorded.
fn cross_check(
    record: &StateAnnouncementProgram,
    symbol: StateLinkSymbol,
    sites: &BTreeSet<usize>,
    linked: &StackItem,
) -> Result<(), StateLinkRefusal> {
    let pristine = record.program().instructions();
    let Some(reference) = reference_item(record, symbol) else {
        return Err(StateLinkRefusal::RelocationCensusDisagreement {
            symbol,
            expected: sites.clone(),
            observed: BTreeSet::new(),
        });
    };

    let expected = if reference == linked {
        BTreeSet::new()
    } else {
        sites.clone()
    };

    let mut rebuilt = pristine.to_vec();
    for &site in sites {
        if let Some(slot) = rebuilt.get_mut(site) {
            *slot = TapscriptInstruction::Push(linked.clone());
        }
    }

    let observed: BTreeSet<usize> = pristine
        .iter()
        .zip(&rebuilt)
        .enumerate()
        .filter(|(_, (before, after))| before != after)
        .map(|(index, _)| index)
        .collect();

    if observed == expected {
        return Ok(());
    }
    Err(StateLinkRefusal::RelocationCensusDisagreement {
        symbol,
        expected,
        observed,
    })
}

/// The checked width difference of one substitution.
fn delta(
    target: &ReviewedElementsTapscriptDefinition,
    symbol: StateLinkSymbol,
    site: usize,
    pre_value: &StackItem,
    linked: &StackItem,
) -> Result<StateResourceDelta, StateLinkRefusal> {
    let overflow = StateLinkRefusal::ResourceDeltaOverflow { symbol, site };
    let before = i64::try_from(push_width(target, pre_value)).map_err(|_| overflow.clone())?;
    let after = i64::try_from(push_width(target, linked)).map_err(|_| overflow.clone())?;

    after
        .checked_sub(before)
        .map(|script_bytes| StateResourceDelta { script_bytes })
        .ok_or(overflow)
}

/// The exact target bytes one literal occupies where it is pushed.
///
/// Measured through the encoder over a one-instruction program rather
/// than restated here, so the figure is the encoding's own and cannot
/// drift from the bytes the linked program writes.
///
/// # Panics
///
/// Panics only if one instruction exceeds the program's instruction
/// limit, which a limit of ten thousand cannot arrange.
fn push_width(target: &ReviewedElementsTapscriptDefinition, item: &StackItem) -> u64 {
    TapscriptProgram::new(vec![TapscriptInstruction::Push(item.clone())])
        .expect("one instruction is within the program's instruction limit")
        .encoded_length(target)
}

// --- The linked leaf ---------------------------------------------------

/// One announcement leaf after substitution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedStateLeafProgram {
    leaf: StateLeafRole,
    program: TapscriptProgram,
    relocations: StateRelocationCensus,
    execution: AbstractExecutionResult,
}

impl LinkedStateLeafProgram {
    /// The leaf's role.
    #[must_use]
    pub const fn leaf(&self) -> StateLeafRole {
        self.leaf
    }

    /// The linked typed program.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }

    /// Every occurrence the link placed.
    #[must_use]
    pub const fn relocations(&self) -> &StateRelocationCensus {
        &self.relocations
    }

    /// The walk the linked program was revalidated by.
    #[must_use]
    pub const fn execution(&self) -> &AbstractExecutionResult {
        &self.execution
    }
}

/// Link one announcement leaf from its resolved census.
///
/// # Panics
///
/// Panics only if the rebuild exceeds the program's instruction limit,
/// which it cannot: it carries the pristine program's instruction count,
/// and that program was admitted under the same limit.
///
/// # Errors
///
/// Whatever [`discover_state_relocations`] raises, and then whatever
/// [`check_linked_state_program`] raises over the rebuilt program.
pub fn substitute_state(
    target: &ReviewedElementsTapscriptDefinition,
    record: &StateAnnouncementProgram,
    resolved: &StateResolvedCensus,
) -> Result<LinkedStateLeafProgram, StateLinkRefusal> {
    let relocations = discover_state_relocations(target, record, resolved)?;

    // Once, from the pristine instructions, with every value together.
    let mut instructions = record.program().instructions().to_vec();
    for relocation in relocations.relocations() {
        if let Some(slot) = instructions.get_mut(relocation.site) {
            *slot = TapscriptInstruction::Push(relocation.linked_value.clone());
        }
    }
    let program = TapscriptProgram::new(instructions)
        .expect("the rebuild carries the pristine program's instruction count");

    let execution = check_linked_state_program(target, record, &program, &relocations)?;

    Ok(LinkedStateLeafProgram {
        leaf: StateLeafRole::Announcement,
        program,
        relocations,
        execution,
    })
}

/// Hold one program to the census that claims to describe it.
///
/// Public because the checks are the claim: a caller holding a program
/// this module did not build can put it to exactly the same four
/// questions, and a test can show what each of them refuses.
///
/// # Errors
///
/// [`StateLinkRefusal::RelocationNotApplied`] when a site does not carry
/// its linked value, [`StateLinkRefusal::UntrackedProgramMutation`] when
/// an instruction no relocation covers changed,
/// [`StateLinkRefusal::RoundTripMismatch`] when the bytes do not decode
/// back to the same program,
/// [`StateLinkRefusal::LinkedProgramDoesNotSchedule`] when the walk from
/// the record's precondition refuses it, and
/// [`StateLinkRefusal::AbstractExecutionMoved`] when that walk reaches a
/// different execution from the record's.
pub fn check_linked_state_program(
    target: &ReviewedElementsTapscriptDefinition,
    record: &StateAnnouncementProgram,
    program: &TapscriptProgram,
    relocations: &StateRelocationCensus,
) -> Result<AbstractExecutionResult, StateLinkRefusal> {
    check_sites(program, relocations)?;
    check_untracked(record.program(), program, relocations)?;
    check_round_trip(target, program)?;
    revalidate(target, record, program)
}

/// Require every site to carry its linked value.
fn check_sites(
    program: &TapscriptProgram,
    relocations: &StateRelocationCensus,
) -> Result<(), StateLinkRefusal> {
    for relocation in relocations.relocations() {
        match program.instructions().get(relocation.site) {
            Some(TapscriptInstruction::Push(item)) if *item == relocation.linked_value => {}
            _ => {
                return Err(StateLinkRefusal::RelocationNotApplied {
                    symbol: relocation.symbol,
                    site: relocation.site,
                });
            }
        }
    }
    Ok(())
}

/// Require every instruction the census does not cover to be the
/// composition's own.
fn check_untracked(
    pristine: &TapscriptProgram,
    linked: &TapscriptProgram,
    relocations: &StateRelocationCensus,
) -> Result<(), StateLinkRefusal> {
    // A changed instruction count is itself an untracked mutation, and
    // it is reported against the first index at which the two programs
    // can no longer be compared rather than swallowed.
    if pristine.len() != linked.len() {
        return Err(StateLinkRefusal::UntrackedProgramMutation {
            site: pristine.len().min(linked.len()),
        });
    }

    let covered = relocations.sites();
    for (site, (before, after)) in pristine
        .instructions()
        .iter()
        .zip(linked.instructions())
        .enumerate()
    {
        if before != after && !covered.contains(&site) {
            return Err(StateLinkRefusal::UntrackedProgramMutation { site });
        }
    }
    Ok(())
}

/// Require the linked program to decode back to itself.
fn check_round_trip(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> Result<(), StateLinkRefusal> {
    let bytes = program.encode(target);
    let decoded = TapscriptProgram::decode(target, &bytes)
        .map_err(|_| StateLinkRefusal::RoundTripMismatch)?;
    if decoded == *program {
        return Ok(());
    }
    Err(StateLinkRefusal::RoundTripMismatch)
}

/// Walk the linked program from the record's own precondition.
fn revalidate(
    target: &ReviewedElementsTapscriptDefinition,
    record: &StateAnnouncementProgram,
    program: &TapscriptProgram,
) -> Result<AbstractExecutionResult, StateLinkRefusal> {
    let execution = validate_program(
        target,
        program,
        record.precondition(),
        AbstractLimits::for_target(target),
    )
    .map_err(|cause| StateLinkRefusal::LinkedProgramDoesNotSchedule { cause })?;

    if execution == *record.execution() {
        return Ok(execution);
    }
    Err(StateLinkRefusal::AbstractExecutionMoved)
}
