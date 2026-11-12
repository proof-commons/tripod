//! Finite paper model, compiled by the conformance tests, not a production reader.
//!
//! Identity and covenant membership are declared premises here, not derived from
//! hashes or programs. No byte decoder, builder, evaluator or curve is called.
use std::collections::{BTreeMap, BTreeSet};

/// A declared semantic key; equal local output names do not erase this component.
pub type Key = [u8; 32];

/// A symbolic event name, not a transaction id or a content digest.
pub type EventId = u32;

/// An output's creating event and immutable birth position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Output {
    /// Creating event in the declared graph.
    pub event: EventId,
    /// Zero-based position in that event.
    pub position: usize,
}

/// A handle or SOURCE record identity: key and output, never an amount.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle {
    /// Key whose covenant the output bears.
    pub key: Key,
    /// Immutable output identity.
    pub output: Output,
}

/// Immutable event facts supplied independently of construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// Unique symbolic name.
    pub id: EventId,
    /// All graph-local consumed outputs, including ordinary outputs if declared.
    pub inputs: Vec<Output>,
    /// Ordered outputs: `None` is ordinary; `Some(key)` bears that covenant.
    pub outputs: Vec<Option<Key>>,
}

/// The boundary at which a declared finite graph fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    /// Missing dependency or cyclic graph, before the ordered fold.
    Graph,
    /// Identity, single-consumption or output-domain premise.
    ValidHistory,
    /// A later covenant-shaped output cannot be spent.
    BornFirst,
    /// A consumed covenant requires the same key at successor output zero.
    ContinueFirst,
}

/// One condition per refusal; these are model outcomes, not node rejections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Repeated identical event declaration.
    DuplicateEvent(EventId),
    /// Same event name bound to different immutable contents.
    ConflictingEvent(EventId),
    /// A dependency's event is absent.
    MissingEvent(Output),
    /// A dependency names no output in its creating event.
    MissingOutput(Output),
    /// An output is spent twice, including twice by one event.
    DuplicateConsumption(Output),
    /// No dependency-ready event remains.
    Cycle,
    /// A covenant-bearing nonzero output is consumed.
    LaterOutputConsumed(Output),
    /// A spender's first output does not bear the consumed key.
    WrongSuccessor {
        /// Spent handle.
        input: Handle,
        /// Offending event.
        spender: EventId,
    },
}

impl Refusal {
    /// Names the premise or clause actually checked by the model.
    pub const fn boundary(&self) -> Boundary {
        match self {
            Self::MissingEvent(_) | Self::Cycle => Boundary::Graph,
            Self::DuplicateEvent(_)
            | Self::ConflictingEvent(_)
            | Self::MissingOutput(_)
            | Self::DuplicateConsumption(_) => Boundary::ValidHistory,
            Self::LaterOutputConsumed(_) => Boundary::BornFirst,
            Self::WrongSuccessor { .. } => Boundary::ContinueFirst,
        }
    }
}

/// Dependency-derived sets, including consumed handles, and the live frontier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    outputs: BTreeMap<EventId, Vec<Option<Key>>>,
    sets: BTreeMap<Handle, BTreeSet<Handle>>,
    live: BTreeSet<Handle>,
    successors: BTreeMap<Handle, Handle>,
    order: Vec<EventId>,
}

/// Precise query result; only a declared ordinary or later output is a non-handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputStatus {
    /// The creating event is not declared.
    MissingEvent,
    /// The event exists but this output position does not.
    MissingOutput,
    /// A declared ordinary output or a later covenant-bearing output.
    NotHandle,
    /// The position is a handle under another key.
    WrongKey(Key),
    /// This exact keyed handle exists.
    Handle(Handle),
}

impl History {
    /// Distinguishes missing evidence coordinates from declared non-membership.
    pub fn query(&self, handle: Handle) -> OutputStatus {
        let Some(outputs) = self.outputs.get(&handle.output.event) else {
            return OutputStatus::MissingEvent;
        };
        let Some(key) = outputs.get(handle.output.position) else {
            return OutputStatus::MissingOutput;
        };
        match key {
            _ if handle.output.position != 0 => OutputStatus::NotHandle,
            None => OutputStatus::NotHandle,
            Some(key) if *key != handle.key => OutputStatus::WrongKey(*key),
            Some(_) => OutputStatus::Handle(handle),
        }
    }
    /// Returns no set for a non-handle, including a later same-key output.
    pub fn records(&self, handle: Handle) -> Option<&BTreeSet<Handle>> {
        self.sets.get(&handle)
    }

    /// Unconsumed handles, whose record sets are pairwise disjoint in a history.
    pub const fn live(&self) -> &BTreeSet<Handle> {
        &self.live
    }

    /// Unique successor when the handle has been consumed.
    pub fn successor(&self, handle: Handle) -> Option<Handle> {
        self.successors.get(&handle).copied()
    }

    /// Deterministic topological order, independent of declaration order.
    pub fn order(&self) -> &[EventId] {
        &self.order
    }
}

/// Folds the declared graph in dependency order using only paper identities.
/// Ordinary external funding is a fixture premise, not inferred by this model.
/// Checks duplicate declarations by sorted id, then missing events, missing
/// outputs, duplicate consumption, complete topology, and finally clauses in
/// minimum-id topological order and declared input order.
///
/// # Panics
/// Panics only if a checked parent or its completed handle set is absent during
/// the fold, which immutable prevalidated declarations cannot arrange.
pub fn analyze(events: &[Event]) -> Result<History, Refusal> {
    let declared = declarations(events)?;
    let order = topology(&declared)?;
    let mut history = History {
        outputs: declared
            .iter()
            .map(|(id, event)| (*id, event.outputs.clone()))
            .collect(),
        sets: BTreeMap::new(),
        live: BTreeSet::new(),
        successors: BTreeMap::new(),
        order,
    };
    for id in &history.order {
        let ready = declared[id];
        let first = ready.outputs.first().copied().flatten();
        let mut records = BTreeSet::new();
        let mut members = Vec::new();
        for input in &ready.inputs {
            // All parents have completed. Their first-output sets carry membership;
            // other output positions retain their explicitly declared key below.
            let parent = declared[&input.event];
            if let Some(key) = parent.outputs[input.position] {
                if input.position != 0 {
                    return Err(Refusal::LaterOutputConsumed(*input));
                }
                let handle = Handle {
                    key,
                    output: *input,
                };
                if first != Some(key) {
                    return Err(Refusal::WrongSuccessor {
                        input: handle,
                        spender: ready.id,
                    });
                }
                // Topology completes every parent's first-output set before use.
                let invariant = std::any::type_name::<Handle>();
                let set = history.sets.get(&handle).expect(invariant);
                records.extend(set.iter().copied());
                members.push(handle);
            }
        }
        if let Some(key) = first {
            let handle = Handle {
                key,
                output: Output {
                    event: ready.id,
                    position: 0,
                },
            };
            if members.is_empty() {
                records.insert(handle);
            }
            for member in members {
                history.live.remove(&member);
                history.successors.insert(member, handle);
            }
            history.sets.insert(handle, records);
            history.live.insert(handle);
        }
    }
    Ok(history)
}

fn declarations(events: &[Event]) -> Result<BTreeMap<EventId, &Event>, Refusal> {
    let mut groups: BTreeMap<EventId, Vec<&Event>> = BTreeMap::new();
    for event in events {
        groups.entry(event.id).or_default().push(event);
    }
    let mut declared = BTreeMap::new();
    for (id, group) in groups {
        let first = group[0];
        if group.len() != 1 {
            return Err(if group.iter().all(|event| *event == first) {
                Refusal::DuplicateEvent(id)
            } else {
                Refusal::ConflictingEvent(id)
            });
        }
        declared.insert(id, first);
    }
    for event in declared.values() {
        for input in &event.inputs {
            if !declared.contains_key(&input.event) {
                return Err(Refusal::MissingEvent(*input));
            }
        }
    }
    for event in declared.values() {
        for input in &event.inputs {
            if input.position >= declared[&input.event].outputs.len() {
                return Err(Refusal::MissingOutput(*input));
            }
        }
    }
    let mut consumed = BTreeSet::new();
    for event in declared.values() {
        for input in &event.inputs {
            if !consumed.insert(*input) {
                return Err(Refusal::DuplicateConsumption(*input));
            }
        }
    }
    Ok(declared)
}

fn topology(declared: &BTreeMap<EventId, &Event>) -> Result<Vec<EventId>, Refusal> {
    let mut remaining = BTreeMap::new();
    let mut children: BTreeMap<EventId, BTreeSet<EventId>> = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for event in declared.values() {
        let dependencies: BTreeSet<_> = event.inputs.iter().map(|input| input.event).collect();
        remaining.insert(event.id, dependencies.len());
        if dependencies.is_empty() {
            ready.insert(event.id);
        }
        for parent in dependencies {
            children.entry(parent).or_default().insert(event.id);
        }
    }
    let mut order = Vec::new();
    while let Some(id) = ready.pop_first() {
        order.push(id);
        if let Some(next) = children.get(&id) {
            for child in next {
                let invariant = std::any::type_name::<Event>();
                let count = remaining.get_mut(child).expect(invariant);
                *count -= 1;
                if *count == 0 {
                    ready.insert(*child);
                }
            }
        }
    }
    if order.len() != declared.len() {
        return Err(Refusal::Cycle);
    }
    Ok(order)
}
