//! The candidate linked maturity bundle (§11.6), and the link that
//! produces one.
//!
//! # Every artifact the link produced, carried once
//!
//! §11.6's linked candidate is the whole of what the link established:
//! the validated plan and the reviewed revision, the bound deployment
//! and its operator binding, the constructor policy and the exact static
//! subtree, the linked record and its programs, the placements and the
//! control recipes, the resolved census, the authenticated graph and its
//! cut proof, the relocations, the checked resources, the carrier
//! closure and its outstanding ABI contract, the evidence and lifecycle
//! obligations, and the candidate status.
//!
//! Carried once is a rule and not an aspiration, so it is stated here
//! and applied everywhere below: a value another field already carries
//! is reached by delegation, and only a value that would otherwise have
//! to be rebuilt is a field of its own. The plan, the reviewed revision
//! and the lifecycle closure are the bound deployment's and are read
//! through it; the static subtree is the committed tree's; the
//! relocations are the linked program's; the ABI obligations are the
//! closure's; the definitions are a projection of the resolved census,
//! which embeds each of them. Two fields for one fact would let a reader
//! compare a bundle against itself and find a disagreement, which is
//! precisely what a carried-once aggregate exists to make impossible.
//!
//! # No content digest, and no field for one
//!
//! Nothing here hashes the bundle, the leaf set, the tree or the
//! constructor, and no field would hold such a figure. An identity
//! minted before a consumer of one exists is an identity this generation
//! refuses to mint: it would be published, compared and depended upon by
//! nobody, and the first real consumer would arrive to find a name
//! already chosen for reasons no requirement recorded. The hashes that
//! do appear — the static root, the metadata leaf hash, the merkle root
//! — are the constructor's own, computed over bytes a spend runs, and
//! they are carried rather than minted.
//!
//! # The constructor is applied over the linked subtree, never the
//! pre-link one
//!
//! A deployment publishes the tree that commits the bytes a spend runs.
//! The composed record's program still carries the composition's
//! fixtures at every push site the census resolves, so a constructor
//! derived over it commits a program nobody spends — which the carrier
//! closure already refuses from the other side, by locating each emitted
//! component inside a leaf the committed tree actually commits. So the
//! link substitutes first and derives afterwards, over a static subtree
//! built from the linked leaf alone.
//!
//! That order is sound because of a fixed point, and the fixed point is
//! asserted rather than assumed. Of the six keys the leaf pushes, five
//! resolve from the deployment, the architecture's declarations and the
//! reviewed target, none of which the constructor touches. The sixth,
//! the internal key, does resolve from the constructor — from its
//! internal-key policy — but that policy is an *input* to the derivation
//! rather than a function of the subtree, so it is the same value before
//! and after. The static root is the one definition that does move, and
//! no instruction pushes it: it is witnessed at spend time and
//! authenticated against the program, which is why the census gives it
//! no push site and the relocation stage refuses a literal one. The
//! linked leaf therefore does not change when the constructor is
//! re-derived over the subtree that commits it, and
//! [`state_application_fixed_point`] and
//! [`state_applied_references_agree`] are where the link says so instead
//! of leaving a reader to work it out.
//!
//! # A concrete constructor is an application, not a field
//!
//! The same static subtree under two semantic metadata values is two
//! constructors: two metadata leaves, two merkle roots, two output keys,
//! and in general two representation nonces. A bundle with a
//! constructor field would therefore have to pick one metadata value and
//! carry the result as though it were a property of the link, and a
//! reader could not tell the retained instance from the recipe. So the
//! aggregate exposes [`CandidateLinkedMaturityBundle::apply_constructor`]
//! — fallible, over the fixed linked subtree, under the bundle's own
//! policy and budget — and retains a supplied instance only through
//! [`CandidateLinkedMaturityBundle::retain`], which keeps the exact
//! encoded metadata and nonce beside it. Retaining a constructor without
//! its exact metadata would retain a claim rather than an artifact,
//! because nothing in the constructor's bytes says which metadata a
//! caller believed it was applying.
//!
//! # The continuity equality is one named function
//!
//! §17.3 refuses constructor migration, and no migration between static
//! subtrees is implemented. [`state_bundle_continuity`] is the whole of
//! that rule in this crate: one function, exact subtree equality, one
//! refusal. It is written once rather than spread across the places a
//! subtree is compared because it is the single assertion a migration
//! relation would replace, and scattered equalities would leave a later
//! generation nothing to point at.
//!
//! # The outstanding set is never empty
//!
//! [`OutstandingStateLinkObligations`] holds its least member in a field
//! of its own, so a linked maturity bundle owing nothing has no
//! representation — the shape the earlier generations' outstanding sets
//! already take, and for the same reason. What this link does not do is
//! not a gap to be discovered by reading it: it validates no current
//! state, searches no nonce against a real predecessor, finalizes
//! nothing, populates no witness, supplies no ABI and enforces none of
//! the relations the realization evaluates over a whole observed
//! transaction. Each of those is a named member of the set, with the
//! layer that owes it named in prose beside it.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU32, NonZeroUsize};

use tapscript::upstream::{
    EncodedStateMetadata, ExternalEvidenceRequirement, MaturityAnnouncementLifecycleClosure,
    MaturityAnnouncementRepresentationProjection, StateMetadata, StateSingletonDeclaration,
    ValidatedMaturityAnnouncementOperationPlan,
};
use tapscript::{
    CandidateStateConstructor, StateAnnouncementProgram, StateConstructorGeneration,
    StateConstructorReference, StateCurveCapability, StateInternalKeyPolicy, StateLeafRole,
    StateNonceBudget, StateStaticLeaf, StateStaticNode, StateStaticSubtree,
};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::bundle::LinkedArtifactStatus;
use crate::error::LinkRefusal;
use crate::state_carrier::{StateAbiObligation, StateCarrierClosure, close_state_carriers};
use crate::state_deployment::StateLinkDeploymentParameters;
use crate::state_error::StateLinkRefusal;
use crate::state_graph::{StateAuthenticatedGraph, state_graph_from_sources};
use crate::state_relocate::{LinkedStateLeafProgram, StateRelocationCensus, substitute_state};
use crate::state_resource::{StateLinkedResources, measure_state_resources};
use crate::state_symbol::{
    StateConsumerCensus, StateLinkSymbol, StateResolvedCensus, StateResolvedEntry,
    StateSingletonAsset, StateSymbolDefinition, collect_state_definitions, resolve_state_census,
};
use crate::state_taptree::{StateLinkedTaptree, assemble_state_static, state_static_taptree_input};
use crate::taptree::ControlPathRecipe;

// --- The outstanding set ------------------------------------------------

/// One obligation a maturity link creates and does not discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StateLinkObligation {
    /// The current state was not validated against an observed
    /// predecessor.
    ///
    /// This link is handed semantic metadata and applies a constructor
    /// to it. Whether that metadata is the state a real predecessor
    /// output actually carries is a question about a chain, answered by
    /// the wave that builds a transaction against one; a link claiming
    /// it would be claiming an observation it never made.
    CurrentStateValidationUndischarged,
    /// No representation nonce was searched for a successor of an
    /// observed predecessor.
    ///
    /// The nonce a retained instance carries is the one the application
    /// selected for the metadata the caller supplied. A successor's
    /// metadata is derived from a predecessor's by the transition, so the
    /// search that matters is over a state this link has not been shown,
    /// and it belongs to the same wave as the validation above.
    SuccessorNonceSearchUndischarged,
    /// Nothing here is finalized and no witness is populated.
    ///
    /// The bundle carries what a spend needs to be built from — the
    /// linked bytes, the committed tree, the control recipes and the
    /// exact ABI contract — and builds no spend. Finalization and
    /// witness population are the transaction-building wave's, and a
    /// candidate that performed either would have stopped being a
    /// candidate.
    FinalizationAndWitnessPopulationUndischarged,
    /// The fourth side of the carrier closure is not supplied.
    ///
    /// The closure compares three sides and states the fourth as an
    /// exact contract rather than a claim: one obligation per witness
    /// role, slot, deployment fact and open premise, keyed like a row so
    /// the wave that builds the ABI answers one relation at a time.
    /// Nothing here counts a listed obligation as met.
    AbiUnsupplied,
    /// The relations the realization evaluates over a whole observed
    /// transaction are enforced by no leaf.
    ///
    /// Five of the announcement's relations quantify over the whole
    /// observed transaction rather than over the positions the operation
    /// claims. The reduced leaf carries no check for any of them, and
    /// the closure publishes them as model-scope rows rather than
    /// dropping them, because until the realization re-scopes those
    /// relations to the region an operation claims, a transaction
    /// composing several operations is accepted on-chain and outside the
    /// model. The refit is the realization's, not this link's.
    ModelScopeRelationsUnenforced,
}

/// The outstanding maturity link obligations, which are never none.
///
/// Structurally non-empty: the least obligation is a field of its own,
/// so a linked bundle owing nothing has no representation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutstandingStateLinkObligations {
    least: StateLinkObligation,
    rest: BTreeSet<StateLinkObligation>,
}

impl OutstandingStateLinkObligations {
    /// Every outstanding obligation, in canonical order.
    pub fn obligations(&self) -> impl Iterator<Item = &StateLinkObligation> {
        std::iter::once(&self.least).chain(self.rest.iter())
    }

    /// How many are outstanding, which is never zero.
    ///
    /// Saturation is unreachable: `rest` is a set of the remaining
    /// [`StateLinkObligation`] variants while `least` is held
    /// separately, so the count is bounded by the enum's finite variant
    /// set.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::MIN.saturating_add(self.rest.len())
    }

    /// Whether one obligation is outstanding.
    #[must_use]
    pub fn holds(&self, obligation: StateLinkObligation) -> bool {
        self.least == obligation || self.rest.contains(&obligation)
    }
}

/// The obligations every maturity link leaves outstanding.
///
/// A constant set rather than a computed one, and that is the honest
/// shape: not one of the five depends on what was linked. Three are the
/// transaction-building wave's, one is the ABI's and one is the
/// realization's refit, and no deployment parameter, record or
/// constructor discharges any of them.
fn outstanding() -> OutstandingStateLinkObligations {
    OutstandingStateLinkObligations {
        least: StateLinkObligation::CurrentStateValidationUndischarged,
        rest: BTreeSet::from([
            StateLinkObligation::SuccessorNonceSearchUndischarged,
            StateLinkObligation::FinalizationAndWitnessPopulationUndischarged,
            StateLinkObligation::AbiUnsupplied,
            StateLinkObligation::ModelScopeRelationsUnenforced,
        ]),
    }
}

// --- The constructor's metadata-independent policy ----------------------

/// What a constructor fixes independently of any metadata value.
///
/// The part of a recipe that survives a change of semantic metadata, and
/// therefore the part an application can be run under. It is read from
/// the constructor the link was resolved against rather than restated,
/// and it is read through that constructor's reference declarations
/// because those are where the constructor states these values: it
/// exposes no accessor for its internal-key policy, leaf version, target
/// revision or nonce budget.
///
/// Two further metadata-independent references exist — the canonical
/// metadata schema revision and the fixed outer branch side — and
/// neither is a field here. Both are constants of the recipe rather than
/// choices a deployment makes, and [`StateConstructorGeneration`] is the
/// identity of the recipe they belong to; carrying them beside it would
/// be naming one thing twice. They are held to the same invariance the
/// fields are, by [`state_applied_references_agree`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateConstructorPolicy {
    generation: StateConstructorGeneration,
    internal_key: StateInternalKeyPolicy,
    leaf_version: LeafVersion,
    target_policy: TargetContractVersion,
    budget: StateNonceBudget,
}

impl StateConstructorPolicy {
    /// Read one constructor's metadata-independent policy.
    ///
    /// # Errors
    ///
    /// [`StateLinkRefusal::ConstructorPolicyIncomplete`] when the
    /// declarations carry no value for one of the four kinds, which the
    /// constructor's own fixed declaration array rules out.
    pub fn of(constructor: &CandidateStateConstructor) -> Result<Self, StateLinkRefusal> {
        let mut internal_key = None;
        let mut leaf_version = None;
        let mut target_policy = None;
        let mut budget = None;

        for declared in constructor.reference_declarations() {
            match declared.reference {
                StateConstructorReference::InternalKeyPolicy(policy) => {
                    internal_key = Some(policy);
                }
                StateConstructorReference::LeafVersion(version) => leaf_version = Some(version),
                StateConstructorReference::TargetPolicy(revision) => target_policy = Some(revision),
                StateConstructorReference::NonceBudget(attempts) => budget = Some(attempts),
                StateConstructorReference::MetadataSchema(_)
                | StateConstructorReference::StaticSubtreeRoot(_)
                | StateConstructorReference::BranchSide(_) => {}
            }
        }

        let missing = |symbol| StateLinkRefusal::ConstructorPolicyIncomplete { missing: symbol };
        Ok(Self {
            generation: constructor.generation(),
            internal_key: internal_key
                .ok_or_else(|| missing(StateLinkSymbol::InternalKeyPolicy))?,
            leaf_version: leaf_version.ok_or_else(|| missing(StateLinkSymbol::LeafVersion))?,
            target_policy: target_policy.ok_or_else(|| missing(StateLinkSymbol::TargetPolicy))?,
            budget: budget.ok_or_else(|| missing(StateLinkSymbol::NonceBudget))?,
        })
    }

    /// The closed recipe generation identity.
    #[must_use]
    pub const fn generation(&self) -> StateConstructorGeneration {
        self.generation
    }

    /// The admitted internal key and its residual policy.
    #[must_use]
    pub const fn internal_key(&self) -> StateInternalKeyPolicy {
        self.internal_key
    }

    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The reviewed target contract revision.
    #[must_use]
    pub const fn target_policy(&self) -> TargetContractVersion {
        self.target_policy
    }

    /// The host's representation search bound.
    #[must_use]
    pub const fn budget(&self) -> StateNonceBudget {
        self.budget
    }
}

// --- One retained instance ----------------------------------------------

/// One supplied concrete constructor, with the exact metadata it is of.
///
/// Both halves travel because neither states the other: the constructor
/// commits an encoding of the metadata rather than the metadata, and the
/// metadata alone does not say which nonce the search settled on. A
/// retained instance missing either would be a claim about an artifact
/// instead of the artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRetainedConstructor {
    metadata: EncodedStateMetadata,
    constructor: CandidateStateConstructor,
}

impl StateRetainedConstructor {
    /// The exact semantic metadata and selected representation nonce.
    #[must_use]
    pub const fn metadata(&self) -> &EncodedStateMetadata {
        &self.metadata
    }

    /// The concrete constructor derived from it.
    #[must_use]
    pub const fn constructor(&self) -> &CandidateStateConstructor {
        &self.constructor
    }
}

// --- The continuity equality --------------------------------------------

/// The §17.3 continuity equality: one predecessor subtree, one
/// successor, and no migration between them.
///
/// This is the no-migration rule, written once. §17.3 implements no
/// constructor migration, so a successor whose static subtree is not the
/// predecessor's is rejected even where the semantic metadata would
/// otherwise match — and the rejection is the whole of the rule, because
/// there is no reconciliation for it to fall back on. It is one named
/// function rather than an equality repeated wherever two subtrees meet
/// because it is the single assertion a migration relation would
/// replace: a later generation that implements migration has exactly one
/// place to point at, and a reader asking what forbids it today has
/// exactly one place to read. Nothing about how such a generation would
/// be shaped is decided here.
///
/// The comparison is the whole subtree — the root and every leaf entry —
/// rather than the root alone, because the root does not commit the
/// caller-supplied leaf identities, so two subtrees can share a root and
/// still be different values.
///
/// # Errors
///
/// [`StateLinkRefusal::StaticSubtreeDiscontinuity`], carrying both
/// roots.
pub fn state_bundle_continuity(
    predecessor: &StateStaticSubtree,
    successor: &StateStaticSubtree,
) -> Result<(), StateLinkRefusal> {
    if predecessor == successor {
        return Ok(());
    }
    Err(StateLinkRefusal::StaticSubtreeDiscontinuity {
        predecessor: *predecessor.root(),
        successor: *successor.root(),
    })
}

// --- The application's fixed point --------------------------------------

/// Hold the census collected before an application to the one collected
/// after it.
///
/// The census half of the fixed point that makes applying a constructor
/// *after* substitution sound. Every key some instruction pushes must
/// resolve to the same entry under both constructors, in both
/// directions: a key that gained an entry and a key that lost one are
/// each a move. The keys no instruction pushes are deliberately outside
/// the comparison — the static root is exactly the definition the
/// application is expected to change.
///
/// # Errors
///
/// [`StateLinkRefusal::CensusMovedUnderApplication`], naming the first
/// pushed key that differs.
pub fn state_application_fixed_point(
    before: &StateResolvedCensus,
    after: &StateResolvedCensus,
) -> Result<(), StateLinkRefusal> {
    let moved = |left: &StateResolvedCensus, right: &StateResolvedCensus| {
        left.entries()
            .iter()
            .filter(|(symbol, _)| symbol.is_program_symbol())
            .find(|(symbol, entry)| right.entries().get(symbol) != Some(*entry))
            .map(|(symbol, _)| *symbol)
    };

    if let Some(symbol) = moved(before, after).or_else(|| moved(after, before)) {
        return Err(StateLinkRefusal::CensusMovedUnderApplication { symbol });
    }
    Ok(())
}

/// Hold an applied constructor's metadata-independent references to what
/// the application was supposed to fix.
///
/// The reference half of the same fixed point. The static root must be
/// the subtree the application was handed, which is what makes the
/// applied constructor a constructor *of the linked leaf*; the metadata
/// schema and the branch side must be the supplied constructor's, which
/// is what makes it the same recipe rather than another one. The four
/// kinds [`StateConstructorPolicy`] carries as fields are already held
/// by being read from the supplied constructor and passed to the
/// application; these two are the remainder.
///
/// # Errors
///
/// [`StateLinkRefusal::AppliedReferenceDisagreement`], naming the
/// reference kind that did not agree.
pub fn state_applied_references_agree(
    applied: &CandidateStateConstructor,
    supplied: &CandidateStateConstructor,
    subtree: &StateStaticSubtree,
) -> Result<(), StateLinkRefusal> {
    let root = StateLinkSymbol::StaticSubtreeRoot;
    if reference(applied, root)
        != Some(StateConstructorReference::StaticSubtreeRoot(
            *subtree.root(),
        ))
    {
        return Err(StateLinkRefusal::AppliedReferenceDisagreement { symbol: root });
    }

    for symbol in [StateLinkSymbol::MetadataSchema, StateLinkSymbol::BranchSide] {
        if reference(applied, symbol) != reference(supplied, symbol) {
            return Err(StateLinkRefusal::AppliedReferenceDisagreement { symbol });
        }
    }
    Ok(())
}

/// One constructor's declaration of one reference kind.
fn reference(
    constructor: &CandidateStateConstructor,
    symbol: StateLinkSymbol,
) -> Option<StateConstructorReference> {
    constructor
        .reference_declarations()
        .into_iter()
        .map(|declared| declared.reference)
        .find(|declared| StateLinkSymbol::from_reference(declared) == symbol)
}

// --- The bundle ---------------------------------------------------------

/// The candidate linked maturity bundle (§11.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateLinkedMaturityBundle {
    deployment: StateLinkDeploymentParameters,
    policy: StateConstructorPolicy,
    record: StateAnnouncementProgram,
    programs: BTreeMap<StateLeafRole, LinkedStateLeafProgram>,
    taptree: StateLinkedTaptree,
    resolved: StateResolvedCensus,
    graph: StateAuthenticatedGraph,
    resources: StateLinkedResources,
    closure: StateCarrierClosure,
    evidence: BTreeSet<ExternalEvidenceRequirement>,
    instances: Vec<StateRetainedConstructor>,
    obligations: OutstandingStateLinkObligations,
}

impl CandidateLinkedMaturityBundle {
    /// The validated announcement plan the linked leaf serves.
    ///
    /// Read through the bound deployment, which carries it: the plan a
    /// link resolves against is the plan its sources were bound under,
    /// and a second copy here could differ from that one.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedMaturityAnnouncementOperationPlan {
        self.deployment.plan()
    }

    /// The reviewed contract revision this link is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.deployment.revision()
    }

    /// The bound sources, with the validated operator binding, the lead
    /// window and its origin, the candidate deployment identity, the
    /// depth cap and the recorded deployment facts.
    #[must_use]
    pub const fn deployment(&self) -> &StateLinkDeploymentParameters {
        &self.deployment
    }

    /// The constructor policy the link was resolved against.
    #[must_use]
    pub const fn policy(&self) -> &StateConstructorPolicy {
        &self.policy
    }

    /// The exact linked static subtree every constructor here is applied
    /// over.
    ///
    /// The committed tree's own, because that is the subtree it was
    /// bound to: a second copy could disagree with the tree that commits
    /// it.
    #[must_use]
    pub const fn static_subtree(&self) -> &StateStaticSubtree {
        self.taptree.subtree()
    }

    /// The composed announcement record, before substitution.
    #[must_use]
    pub const fn record(&self) -> &StateAnnouncementProgram {
        &self.record
    }

    /// Every linked program, in canonical leaf order.
    #[must_use]
    pub const fn programs(&self) -> &BTreeMap<StateLeafRole, LinkedStateLeafProgram> {
        &self.programs
    }

    /// One linked program by leaf.
    #[must_use]
    pub fn program(&self, leaf: StateLeafRole) -> Option<&LinkedStateLeafProgram> {
        self.programs.get(&leaf)
    }

    /// The committed tree, with its two roots, its per-leaf depths and
    /// its exact cost evidence.
    #[must_use]
    pub const fn taptree(&self) -> &StateLinkedTaptree {
        &self.taptree
    }

    /// The control recipes of the committed static tree (§11.4).
    #[must_use]
    pub const fn control_recipes(
        &self,
    ) -> &BTreeMap<StateLeafRole, ControlPathRecipe<StateLeafRole>> {
        self.taptree.tree().recipes()
    }

    /// Every definition, in canonical key order.
    ///
    /// A projection of the resolved census rather than a second census:
    /// pass two refuses a definition no consumer reads, so every
    /// definition the link collected is embedded in a resolved entry,
    /// and a definition census carried beside it would be one fact held
    /// twice.
    pub fn definitions(&self) -> impl Iterator<Item = &StateSymbolDefinition> {
        self.resolved
            .entries()
            .values()
            .map(StateResolvedEntry::definition)
    }

    /// The resolved census: every key, its definition and its consumers.
    #[must_use]
    pub const fn resolved(&self) -> &StateResolvedCensus {
        &self.resolved
    }

    /// The authenticated graph, its validated cuts and the residual
    /// proof.
    #[must_use]
    pub const fn graph(&self) -> &StateAuthenticatedGraph {
        &self.graph
    }

    /// Every occurrence the link placed in the announcement leaf.
    ///
    /// The linked program's own census. `None` where no announcement
    /// program is carried, which a linked bundle does not arrange and
    /// the type does not forbid.
    #[must_use]
    pub fn relocations(&self) -> Option<&StateRelocationCensus> {
        self.program(StateLeafRole::Announcement)
            .map(LinkedStateLeafProgram::relocations)
    }

    /// The checked linked resource evidence.
    #[must_use]
    pub const fn resources(&self) -> &StateLinkedResources {
        &self.resources
    }

    /// The checked three-sided carrier closure (§11.5).
    #[must_use]
    pub const fn carrier_closure(&self) -> &StateCarrierClosure {
        &self.closure
    }

    /// Everything an ABI must supply, in derivation order.
    ///
    /// The closure's own contract, read through it. Nothing here counts
    /// a listed obligation as met.
    #[must_use]
    pub fn abi_obligations(&self) -> &[StateAbiObligation] {
        self.closure.obligations()
    }

    /// The implemented and outstanding exit sets the plan closes over.
    ///
    /// The plan's own closure, shared by both representations: the plan
    /// validates that the two modes' exit closures agree, so one value
    /// is the whole statement and a per-representation copy would be
    /// that one fact written twice.
    #[must_use]
    pub const fn lifecycle(&self) -> &MaturityAnnouncementLifecycleClosure {
        self.deployment.plan().lifecycle()
    }

    /// Every external premise the plan's relations leave open.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<ExternalEvidenceRequirement> {
        &self.evidence
    }

    /// Every retained concrete constructor, in retention order.
    ///
    /// The first is the application the link itself ran, over the
    /// metadata its sources supplied.
    #[must_use]
    pub fn instances(&self) -> &[StateRetainedConstructor] {
        &self.instances
    }

    /// The obligations this link created and did not discharge.
    #[must_use]
    pub const fn obligations(&self) -> &OutstandingStateLinkObligations {
        &self.obligations
    }

    /// This artifact's status (§1.12).
    ///
    /// Always [`LinkedArtifactStatus::Prototype`], and read-only. There
    /// is no field, argument or setter through which a caller could
    /// claim anything else: the evidence a promotion rests on is a later
    /// wave's, does not exist, and has no room reserved for it here.
    #[must_use]
    pub const fn status(&self) -> LinkedArtifactStatus {
        LinkedArtifactStatus::Prototype
    }

    /// Apply this bundle's constructor recipe to one semantic metadata
    /// value.
    ///
    /// Over the fixed linked subtree, under this bundle's own policy and
    /// nonce budget, and fallible because a construction can fail:
    /// the encoding can be refused, the fixed outer side can go
    /// unsatisfied for every nonce the budget admits, and the curve can
    /// refuse a tweak. Nothing is retained by this call.
    ///
    /// # Errors
    ///
    /// [`StateLinkRefusal::ConstructorApplication`], wrapping exactly
    /// what the construction refused.
    pub fn apply_constructor(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        metadata: &StateMetadata,
        curve: &impl StateCurveCapability,
    ) -> Result<CandidateStateConstructor, StateLinkRefusal> {
        apply(target, self.static_subtree(), &self.policy, metadata, curve)
    }

    /// Apply the recipe and retain the result beside its exact metadata.
    ///
    /// The retained instance is held to the continuity equality before
    /// it is kept, so a constructor over any other static subtree cannot
    /// enter — and to the metadata it is of, so one semantic metadata
    /// has one instance. A second retention of the same metadata is
    /// refused rather than appended: it would be the same artifact
    /// carried twice.
    ///
    /// # Errors
    ///
    /// [`StateLinkRefusal::ConstructorApplication`] for a refused
    /// construction, [`StateLinkRefusal::StaticSubtreeDiscontinuity`]
    /// for a constructor over another subtree, and
    /// [`StateLinkRefusal::InstanceAlreadyRetained`] for a semantic
    /// metadata already held.
    ///
    /// # Panics
    ///
    /// Panics only if the vector pushed to immediately above were empty,
    /// which no push can arrange.
    pub fn retain(
        &mut self,
        target: &ReviewedElementsTapscriptDefinition,
        metadata: &StateMetadata,
        curve: &impl StateCurveCapability,
    ) -> Result<&StateRetainedConstructor, StateLinkRefusal> {
        let constructor = self.apply_constructor(target, metadata, curve)?;
        state_bundle_continuity(constructor.static_subtree(), self.static_subtree())?;

        let encoded = *constructor.encoded_metadata();
        if let Some(held) = self
            .instances
            .iter()
            .find(|held| held.metadata.semantic == encoded.semantic)
        {
            return Err(StateLinkRefusal::InstanceAlreadyRetained {
                metadata: held.metadata,
            });
        }

        self.instances.push(StateRetainedConstructor {
            metadata: encoded,
            constructor,
        });
        Ok(self
            .instances
            .last()
            .expect("the instance pushed above is this vector's last"))
    }
}

// --- The link -----------------------------------------------------------

/// The typed sources one candidate maturity link is given.
///
/// A sources struct rather than a parameter list, because seven
/// positional references of which five are borrowed typed values is a
/// signature a caller can get wrong silently.
///
/// No derive: a curve capability is a behaviour rather than a value, so
/// an equality or a rendering of these sources would be an equality or a
/// rendering of an implementation.
pub struct StateLinkSources<'a, C: StateCurveCapability> {
    record: &'a StateAnnouncementProgram,
    deployment: &'a StateLinkDeploymentParameters,
    constructor: &'a CandidateStateConstructor,
    singleton: &'a StateSingletonAsset,
    declaration: &'a StateSingletonDeclaration,
    metadata: &'a StateMetadata,
    curve: &'a C,
}

impl<'a, C: StateCurveCapability> StateLinkSources<'a, C> {
    /// State one link's sources, each as the type that validated it.
    #[must_use]
    pub const fn new(
        record: &'a StateAnnouncementProgram,
        deployment: &'a StateLinkDeploymentParameters,
        constructor: &'a CandidateStateConstructor,
        singleton: &'a StateSingletonAsset,
        declaration: &'a StateSingletonDeclaration,
        metadata: &'a StateMetadata,
        curve: &'a C,
    ) -> Self {
        Self {
            record,
            deployment,
            constructor,
            singleton,
            declaration,
            metadata,
            curve,
        }
    }

    /// The composed announcement record.
    #[must_use]
    pub const fn record(&self) -> &'a StateAnnouncementProgram {
        self.record
    }

    /// The bound deployment sources.
    #[must_use]
    pub const fn deployment(&self) -> &'a StateLinkDeploymentParameters {
        self.deployment
    }

    /// The constructor the census is collected against.
    #[must_use]
    pub const fn constructor(&self) -> &'a CandidateStateConstructor {
        self.constructor
    }

    /// The issued singleton's asset identifier.
    #[must_use]
    pub const fn singleton(&self) -> &'a StateSingletonAsset {
        self.singleton
    }

    /// The architecture's declaration of that singleton.
    #[must_use]
    pub const fn declaration(&self) -> &'a StateSingletonDeclaration {
        self.declaration
    }

    /// The semantic metadata the link's own application is of.
    #[must_use]
    pub const fn metadata(&self) -> &'a StateMetadata {
        self.metadata
    }

    /// The curve capability the application is run against.
    #[must_use]
    pub const fn curve(&self) -> &'a C {
        self.curve
    }
}

/// Link the candidate maturity bundle (§11.6).
///
/// The stages run in the order their inputs require, and each refuses
/// rather than degrades. The supplied constructor is checked to commit
/// the record's own program before anything is collected against it;
/// the census resolves; the leaf is substituted and revalidated by the
/// relocation stage itself; the linked static subtree is built from that
/// leaf alone and the constructor is applied over it, with both halves
/// of the fixed point asserted; the graph, the tree, the resources and
/// the carrier closure then run over the applied constructor and the
/// linked bytes; the continuity equality is put; and the applied
/// instance is retained as the bundle's first.
///
/// # Errors
///
/// [`LinkRefusal::StateLink`] wrapping every refusal the census,
/// relocation, application, graph, resource and carrier stages raise,
/// and the tree stage's own refusals unwrapped, because the committed
/// tree is assembled by the shared engine and speaks the shared root.
///
/// No tree refusal is reachable through this entry, and the reason is
/// recomputed rather than assumed: exactly one leaf is declared, at the
/// announcement role and the reviewed leaf version; the subtree bound
/// against is the applied constructor's own, so the leaf sets, the
/// versions, the depths and the outer pair are each compared with
/// themselves; and a one-leaf complete tree is one level deep, which no
/// positive cap is exceeded by. The tree stage's refusals are exercised
/// against the shared root where that stage lives.
pub fn link_state_candidate<C: StateCurveCapability>(
    target: &ReviewedElementsTapscriptDefinition,
    sources: &StateLinkSources<'_, C>,
) -> Result<CandidateLinkedMaturityBundle, LinkRefusal> {
    let record = sources.record;
    let deployment = sources.deployment;
    let supplied = sources.constructor;

    require_recorded_program(supplied, record)?;
    let supplied_census = resolve(target, sources, supplied)?;

    let linked = substitute_state(target, record, &supplied_census)?;

    let subtree = linked_subtree(target, &linked)?;
    let policy = StateConstructorPolicy::of(supplied)?;
    let applied = apply(target, &subtree, &policy, sources.metadata, sources.curve)?;
    let resolved = resolve(target, sources, &applied)?;
    state_application_fixed_point(&supplied_census, &resolved)?;
    state_applied_references_agree(&applied, supplied, &subtree)?;

    let graph = state_graph_from_sources(record, &applied, &resolved)?;
    let taptree = committed_tree(target, deployment, &linked, &applied)?;
    let resources = measure_state_resources(target, record, &linked)?;
    let closure = close_state_carriers(record, deployment, &linked, &taptree)?;

    // The rule against itself, at the point the bundle is assembled. It
    // holds by construction here — the applied constructor was derived
    // over this very subtree — and it is put regardless, because the
    // one named place the no-migration rule lives has to be a step of
    // every link rather than a function nobody calls. Its non-trivial
    // use is between two bundles.
    state_bundle_continuity(applied.static_subtree(), &subtree)?;

    Ok(CandidateLinkedMaturityBundle {
        deployment: deployment.clone(),
        policy,
        record: record.clone(),
        programs: BTreeMap::from([(linked.leaf(), linked)]),
        taptree,
        resolved,
        graph,
        resources,
        closure,
        evidence: open_premises(deployment.plan()),
        instances: vec![StateRetainedConstructor {
            metadata: *applied.encoded_metadata(),
            constructor: applied,
        }],
        obligations: outstanding(),
    })
}

/// Require the supplied constructor to commit the record's own program.
///
/// The census is collected against that constructor, and its static root
/// becomes a definition of this link. A constructor whose subtree
/// commits some other program would therefore resolve this record's keys
/// against a recipe built for different bytes, and every figure
/// downstream would be computed over two artifacts that were never one.
fn require_recorded_program(
    constructor: &CandidateStateConstructor,
    record: &StateAnnouncementProgram,
) -> Result<(), StateLinkRefusal> {
    let committed = constructor.static_subtree().leaves().iter().any(|entry| {
        entry.leaf.role == StateLeafRole::Announcement && entry.leaf.program == *record.program()
    });
    if committed {
        return Ok(());
    }
    Err(StateLinkRefusal::SuppliedConstructorCommitsAnotherProgram)
}

/// Both census passes over one constructor.
fn resolve<C: StateCurveCapability>(
    target: &ReviewedElementsTapscriptDefinition,
    sources: &StateLinkSources<'_, C>,
    constructor: &CandidateStateConstructor,
) -> Result<StateResolvedCensus, StateLinkRefusal> {
    let definitions = collect_state_definitions(
        target,
        sources.deployment,
        constructor,
        sources.singleton,
        sources.declaration,
    )?;
    resolve_state_census(
        &definitions,
        &StateConsumerCensus::from_sources(sources.record, constructor),
    )
}

/// The static subtree of the linked leaf alone.
fn linked_subtree(
    target: &ReviewedElementsTapscriptDefinition,
    linked: &LinkedStateLeafProgram,
) -> Result<StateStaticSubtree, StateLinkRefusal> {
    StateStaticSubtree::new(
        target,
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role: linked.leaf(),
                program: linked.program().clone(),
                version: target.definition().leaf_version().get(),
            },
        }),
    )
    .map_err(StateLinkRefusal::ConstructorApplication)
}

/// One application of one recipe to one metadata value.
fn apply(
    target: &ReviewedElementsTapscriptDefinition,
    subtree: &StateStaticSubtree,
    policy: &StateConstructorPolicy,
    metadata: &StateMetadata,
    curve: &impl StateCurveCapability,
) -> Result<CandidateStateConstructor, StateLinkRefusal> {
    CandidateStateConstructor::derive(
        target,
        metadata,
        subtree,
        policy.internal_key,
        policy.budget,
        curve,
    )
    .map_err(StateLinkRefusal::ConstructorApplication)
}

/// The committed tree over the linked leaf, under the deployment's cap.
///
/// The cap handed to the static input is the deployment's own less one
/// for the outer pair, floored at one, and the deployment's own figure
/// is enforced over the complete tree where the binding is made.
fn committed_tree(
    target: &ReviewedElementsTapscriptDefinition,
    deployment: &StateLinkDeploymentParameters,
    linked: &LinkedStateLeafProgram,
    applied: &CandidateStateConstructor,
) -> Result<StateLinkedTaptree, LinkRefusal> {
    let complete = deployment.maximum_control_path_depth();
    let static_depth = NonZeroU32::new(complete.get().saturating_sub(1)).unwrap_or(NonZeroU32::MIN);
    let input = state_static_taptree_input(
        [linked.leaf()],
        target.definition().leaf_version(),
        static_depth,
    )?;
    StateLinkedTaptree::bind(target, assemble_state_static(&input)?, applied, complete)
}

/// Every external premise the plan's relations leave open.
///
/// Read from the plan rather than from the closure that compares it, so
/// the set is the plan's statement about its own relations and not a
/// function of how the rows were built.
fn open_premises(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> BTreeSet<ExternalEvidenceRequirement> {
    plan.representations()
        .flat_map(MaturityAnnouncementRepresentationProjection::relations)
        .flat_map(|requirement| requirement.external_evidence.iter().cloned())
        .collect()
}
