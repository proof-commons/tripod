//! Why a STATE maturity link refused, in a root of its own.
//!
//! # Closed, and what the closure buys
//!
//! This enum carries no `non_exhaustive`, so a test can walk it. That is
//! the whole reason it is closed: "every refusal is exercised by a test
//! or declared unreachable with its reason" is a property of the census
//! only if the census of refusals can be enumerated, and an open root
//! makes that sentence unfalsifiable — a variant added tomorrow would be
//! neither exercised nor declared, and nothing would say so.
//!
//! [`crate::error::LinkRefusal`] stays open for the opposite reason. It
//! is the vocabulary of the engines every generation shares — the
//! reference graph, relocation, the tree, carrier closure, resources —
//! where the tree engine's STATE refusals also live, and a generation
//! added to those engines adds refusals to that root. Closing it would
//! make each new generation a breaking change to every consumer of the
//! shared one; leaving this root open would give up the only check that
//! makes the reachability table worth reading.
//!
//! # A refusal here is a defect in what the link was asked to resolve
//!
//! None of these is a statement about what a target node would do with
//! the result, and none returns a partial census: a refused link yields
//! no definitions, because a census missing one key is not a smaller
//! census but a different claim about which keys exist.

use std::collections::BTreeSet;

use tapscript::{StateAnnouncementId, StateLeafRole, StateProgramWitness, TapscriptError};
use target_elements::TargetContractVersion;

use crate::state_constructor_graph::StateReferenceGraphRefusal;
use crate::state_graph::{StateBindingTime, StateGraphNode, StateResidualComponent};
use crate::state_symbol::{StateLinkSymbol, StateSymbolType};

/// Why a STATE maturity link refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateLinkRefusal {
    /// Two definitions claim one typed key.
    ///
    /// Pass one refuses the second claim rather than overwriting the
    /// first: which source settles a key is a fact about the link, and a
    /// silent overwrite would make it a fact about collection order.
    DuplicateDefinition(StateLinkSymbol),

    /// A consumer's key has no definition.
    ///
    /// Either a push site in the composed record or a constructor field
    /// consumes this key, and nothing the link was given resolves it, so
    /// the program would carry a fixture where a deployment value
    /// belongs.
    MissingDefinition(StateLinkSymbol),

    /// A definition's key is consumed by nothing.
    ///
    /// The census's closure statement. An enum variant does not
    /// disappear when a push site does, so a record that stopped
    /// consuming a symbol leaves a definition nobody reads — and a
    /// definition nobody reads is a deployment value the link accepted
    /// responsibility for and then did not use.
    UnusedDefinition(StateLinkSymbol),

    /// A definition is not the type its key declares.
    ///
    /// Width alone cannot separate these: the asset, the internal key
    /// and the operator key are all thirty-two bytes under the reviewed
    /// contract, so an asset offered as a key has the right shape and
    /// the wrong meaning.
    IncompatibleType {
        /// The key whose definition was offered.
        symbol: StateLinkSymbol,
        /// What the key's role requires.
        declared: StateSymbolType,
        /// What the definition carries.
        offered: StateSymbolType,
    },

    /// The constructor's declared revision is not the reviewed one the
    /// link is bound to.
    ///
    /// Two typed sources carry this fact, and agreement is what makes
    /// the definition single-valued. A link that preferred one silently
    /// would be resolving against a contract the other source was never
    /// checked under.
    TargetRevisionDisagreement {
        /// The revision the constructor declares.
        constructor: TargetContractVersion,
        /// The revision the reviewed target fixes for this link.
        reviewed: TargetContractVersion,
    },

    /// The singleton declaration names an asset other than the planned
    /// STATE object's.
    ///
    /// No payload: both identifiers are the architecture's own, and this
    /// crate does not name that type. Carrying a rendering of them would
    /// put a display string where an identity belongs, and carrying
    /// neither is honest about what this layer can say.
    SingletonDeclarationMismatch,

    /// A definition's bytes are not a literal the reviewed contract
    /// admits.
    ///
    /// The item constructor's contract belongs to the reviewed target,
    /// not to this module, so the refusal carries what the constructor
    /// said rather than a bound restated here. Every value built in this
    /// census is eight or thirty-two bytes and the reviewed literal
    /// bound is far above both, which is why no test reaches this; a
    /// narrower reviewed contract would reach it, and assuming the
    /// current bound instead would make this module wrong on that day.
    InvalidDefinitionItem {
        /// The key whose value could not be built.
        symbol: StateLinkSymbol,
        /// What the item constructor refused.
        cause: TapscriptError,
    },

    // --- The authenticated graph ---------------------------------------
    /// The constructor's own declarations could not be frozen.
    ///
    /// The bound on distinct references belongs to the frozen graph
    /// rather than to this module, so the refusal travels exactly as
    /// that graph stated it instead of as a figure restated here and
    /// free to drift from the one actually enforced.
    FrozenGraph(StateReferenceGraphRefusal),

    /// The offered graph carries more distinct nodes than the bound.
    ///
    /// The figure is the constructor's local census bound applied to the
    /// whole typed graph, because a graph the constructor could not have
    /// declared is not one this link can resolve, and a bound enforced
    /// on one side only would be a bound on nothing.
    GraphReferenceLimitExceeded {
        /// The maximum number of distinct nodes.
        limit: usize,
    },

    /// A program would carry the root of the tree committing to it.
    ///
    /// Not a cycle that better evidence could cut. The root is a
    /// function of the program's bytes, so a literal for it inside those
    /// bytes is a fixed point, and the only way to reach for one is to
    /// hash until the bytes stop changing. Witnessing the root and
    /// authenticating it against the program is the resolution, and it
    /// is a different binding time rather than the same edge with a
    /// stronger claim attached to it.
    LiteralStaticRootBeneathItself {
        /// The leaf whose bytes would have to contain their own root.
        program: StateLeafRole,
    },

    /// The constructor-kind nodes are not the frozen graph's own.
    ///
    /// Both sets travel, because either side may be the one that moved:
    /// a kind the constructor declares and this graph omits, and a kind
    /// this graph carries that the constructor never declared, are
    /// different defects, and a single missing name could not tell them
    /// apart.
    ConstructorProjectionMismatch {
        /// The kinds the constructor's frozen graph declares.
        frozen: BTreeSet<StateLinkSymbol>,
        /// The kinds the offered graph carries.
        graph: BTreeSet<StateLinkSymbol>,
    },

    /// A binding time's required evidence is not in the record.
    ///
    /// The edge travels with the gap. A cut is a claim about one
    /// dependency, so a refusal naming only the absent role would leave
    /// a reader to guess which of the edges bound that way the record
    /// could not support.
    UnvalidatedCut {
        /// The dependent node of the edge that was not validated.
        referrer: StateGraphNode,
        /// The node it depends on.
        referent: StateGraphNode,
        /// The binding time whose evidence was demanded.
        binding: StateBindingTime,
        /// Required witness roles the record does not declare.
        missing_witnesses: Vec<StateProgramWitness>,
        /// Required components the record does not walk.
        missing_components: BTreeSet<StateAnnouncementId>,
    },

    /// A cycle survives every validated cut.
    ///
    /// The removed cuts travel with the component, because the finding
    /// is about what they did not reach: a component still cyclic after
    /// every authenticated removal is a cycle all of whose edges are
    /// settled before the leaf runs, and naming the removals is what
    /// shows the cut to have been insufficient rather than absent.
    ResidualCycle {
        /// The component still carrying a cycle.
        component: StateResidualComponent,
        /// Every cut edge removed before the residual was computed.
        cuts_removed: BTreeSet<usize>,
    },
}
