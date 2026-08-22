//! The candidate linked bundle, and the link that produces one (§14.7).
//!
//! # A candidate, held there by three separate facts
//!
//! §1.9 keeps the candidate and final states distinct, and this type
//! stays on the candidate side without relying on anyone reading a
//! comment:
//!
//! - [`LinkedArtifactStatus`] is read, never written. There is no
//!   field, argument, or setter through which a caller could claim
//!   anything but [`LinkedArtifactStatus::Prototype`], and the
//!   evidence a promotion would rest on does not exist yet, so no field
//!   reserves a place for it (§1.10);
//! - the bundle's outstanding lifecycle is carried through from the
//!   relocatable bundle, where it is already structurally non-empty;
//! - [`OutstandingLinkObligations`] is a second, link-specific
//!   non-empty set. A linked bundle always owes at least the taproot
//!   output key, because this wave computes no hash and no tweak.
//!
//! # No digest, and no room for one
//!
//! §1.10 admits a digest only once a real consumer of one appears, and
//! forbids a candidate type reserving a field for a future one. There
//! is no bundle hash here, no leaf hash, no tree hash, and no field
//! that would hold one.
//!
//! # What a link actually does
//!
//! The stages are §14.2's list, in order, and each one refuses rather
//! than degrades. Nothing is returned when a stage refuses: there is no
//! partially linked value, which is §1.11 applied to a pipeline rather
//! than to a search.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU64, NonZeroUsize};

use tapscript::upstream::{LifecycleRequirement, RelationCaseKey, ValidatedTargetOperationPlan};
use tapscript::{
    AshRepresentationSelection, BackendArtifactStatus, BundleSymbol,
    CandidateRelocatableTapscriptBundle, CandidateShapeSet, CompactAshShape, ConcreteLayout,
    ConcreteRelationPlacement, ConstructorAssumption, ExactTargetProjection, ExplicitValuePolicy,
    InternalKeyPolicy, KeyPathPolicy, LeafRole, ProgramRole, RelocationSite, ResourceModel,
    ResourceObligation, StackItem, WitnessRole, fit_shape_model,
};
use target_elements::{
    LeafVersion, ResourceDimension, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    TargetEvidenceRequirementId,
};

use crate::carrier::{CarrierClosure, close};
use crate::deployment::{LinkDeploymentParameters, SelfCommitmentStrategy};
use crate::error::LinkRefusal;
use crate::graph::{FrozenReferenceGraph, apply_cycle_policy, resolve_references};
use crate::relocate::{LinkedLeafProgram, substitute};
use crate::symbol::{DefinitionCensus, collect_definitions};
use crate::taptree::{DeterministicTaptree, TapLeafInput, TaptreeInput, TreeObjective, assemble};

/// The status vocabulary linked artifacts are distinguished by (§1.9).
///
/// The same three states the backend's own vocabulary uses, because a
/// linked artifact cannot be further along than the backend artifact it
/// was linked from, and two different vocabularies would let it look
/// like it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LinkedArtifactStatus {
    /// Linked and internally checked; no operation evidence.
    Prototype,
    /// Full operation evidence passed for exactly these programs.
    CandidateOperationProven,
    /// Approved for production deployment.
    ProductionApproved,
}

/// One obligation a link creates and does not discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LinkObligation {
    /// The taptree's merkle root, the internal-key tweak, and the
    /// resulting taproot output key are not computed here.
    ///
    /// §14.5 states the tree input and the comparison against
    /// exhaustive enumeration, both of which are structure; the hashes
    /// and the curve arithmetic that turn a tree into an output key
    /// belong to the layer that builds a transaction against a real
    /// deployment. Computing them here would mint an identity before a
    /// consumer of one exists (§1.10).
    TaprootOutputKeyUndischarged,
    /// The supplied ASH constructor program was accepted on the
    /// caller's authority and not checked against the tree it commits
    /// to.
    ///
    /// Carried only under
    /// [`SelfCommitmentStrategy::ExternallyAuthenticatedCommitment`],
    /// and it is the whole content of that strategy: the equality
    /// between the supplied program and the taproot output over these
    /// leaves is exactly what nothing in this wave establishes.
    SelfCommitmentEqualityUndischarged,
    /// The resolved internal key was not verified unspendable from
    /// public data.
    ///
    /// The constructor states
    /// [`ConstructorAssumption::InternalKeyUnspendabilityVerifiableFromPublicData`]
    /// and names the linker as the layer that owes it. Discharging it
    /// needs curve arithmetic over a real key, so it is recorded rather
    /// than claimed.
    InternalKeyUnspendabilityUnverified,
}

/// The outstanding link obligations, which are never none.
///
/// Structurally non-empty for the same reason the backend's
/// outstanding lifecycle is: the least obligation is a field of its
/// own, so a linked bundle owing nothing has no representation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutstandingLinkObligations {
    least: LinkObligation,
    rest: BTreeSet<LinkObligation>,
}

impl OutstandingLinkObligations {
    /// Every outstanding obligation, in canonical order.
    pub fn obligations(&self) -> impl Iterator<Item = &LinkObligation> {
        std::iter::once(&self.least).chain(self.rest.iter())
    }

    /// How many are outstanding, which is never zero.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::MIN.saturating_add(self.rest.len())
    }

    /// Whether one obligation is outstanding.
    #[must_use]
    pub fn holds(&self, obligation: LinkObligation) -> bool {
        self.least == obligation || self.rest.contains(&obligation)
    }
}

/// The linked static constructor (§14.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedConstructor {
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    representation: AshRepresentationSelection,
    value_policy: ExplicitValuePolicy,
    internal_key: StackItem,
    internal_key_policy: InternalKeyPolicy,
    key_path: KeyPathPolicy,
    assumptions: BTreeSet<ConstructorAssumption>,
}

impl LinkedConstructor {
    /// The reviewed contract revision the constructor is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The selected ASH representation.
    ///
    /// Carried across rather than recomputed, and carried at all
    /// because §15.4 requires the transaction ABI to state the explicit
    /// ASH representation and §1.12 admits one authored source for it.
    /// A consumer that had to reach past the linked bundle for it would
    /// be reading the pre-link constructor the link may have changed.
    #[must_use]
    pub const fn representation(&self) -> AshRepresentationSelection {
        self.representation
    }

    /// The explicit value policy every value field is held to.
    #[must_use]
    pub const fn value_policy(&self) -> ExplicitValuePolicy {
        self.value_policy
    }

    /// The resolved unspendable internal key.
    #[must_use]
    pub const fn internal_key(&self) -> &StackItem {
        &self.internal_key
    }

    /// The internal-key policy, which admits one value.
    #[must_use]
    pub const fn internal_key_policy(&self) -> InternalKeyPolicy {
        self.internal_key_policy
    }

    /// The key-path policy, which admits one value.
    #[must_use]
    pub const fn key_path(&self) -> KeyPathPolicy {
        self.key_path
    }

    /// Every assumption the constructor rests on.
    #[must_use]
    pub const fn assumptions(&self) -> &BTreeSet<ConstructorAssumption> {
        &self.assumptions
    }
}

/// One program role's linked resource behaviour across the shape set.
///
/// Refitted rather than carried across. Substitution changes the
/// pushed literals' widths, so every linked program's exact encoded
/// byte length differs from the pre-link one and the pre-link
/// coefficients describe a program that no longer exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedResourceFormula {
    role: ProgramRole,
    dimension: ResourceDimension,
    model: ResourceModel,
    measurements: BTreeMap<CompactAshShape, u64>,
}

impl LinkedResourceFormula {
    /// The program role this describes.
    #[must_use]
    pub const fn role(&self) -> ProgramRole {
        self.role
    }

    /// The dimension this describes.
    #[must_use]
    pub const fn dimension(&self) -> ResourceDimension {
        self.dimension
    }

    /// The refitted model.
    #[must_use]
    pub const fn model(&self) -> ResourceModel {
        self.model
    }

    /// The exact linked measurement at every shape.
    #[must_use]
    pub const fn measurements(&self) -> &BTreeMap<CompactAshShape, u64> {
        &self.measurements
    }
}

/// The candidate linked bundle (§14.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateLinkedBundle {
    plan: ValidatedTargetOperationPlan,
    projection: ExactTargetProjection,
    shapes: CandidateShapeSet,
    constructor: LinkedConstructor,
    programs: BTreeMap<LeafRole, LinkedLeafProgram>,
    taptree: DeterministicTaptree,
    graph: FrozenReferenceGraph,
    definitions: DefinitionCensus,
    placements: BTreeMap<RelationCaseKey, ConcreteRelationPlacement>,
    closure: CarrierClosure,
    layouts: BTreeMap<CompactAshShape, ConcreteLayout>,
    witness: BTreeMap<LeafRole, WitnessRole>,
    formulas: BTreeMap<ProgramRole, BTreeMap<ResourceDimension, LinkedResourceFormula>>,
    outstanding_dimensions: BTreeMap<ResourceDimension, ResourceObligation>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
    lifecycle: BTreeSet<LifecycleRequirement>,
    obligations: OutstandingLinkObligations,
    self_commitment: SelfCommitmentStrategy,
}

impl CandidateLinkedBundle {
    /// The validated operation plan the linked programs serve.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedTargetOperationPlan {
        &self.plan
    }

    /// The exact reviewed target projection.
    #[must_use]
    pub const fn target_projection(&self) -> &ExactTargetProjection {
        &self.projection
    }

    /// The candidate shape and bound assignment.
    #[must_use]
    pub const fn shapes(&self) -> &CandidateShapeSet {
        &self.shapes
    }

    /// The linked constructor.
    #[must_use]
    pub const fn constructor(&self) -> &LinkedConstructor {
        &self.constructor
    }

    /// Every linked program, in canonical order.
    #[must_use]
    pub const fn programs(&self) -> &BTreeMap<LeafRole, LinkedLeafProgram> {
        &self.programs
    }

    /// One linked program by leaf.
    #[must_use]
    pub fn program(&self, leaf: LeafRole) -> Option<&LinkedLeafProgram> {
        self.programs.get(&leaf)
    }

    /// The committed taptree and its control recipes.
    #[must_use]
    pub const fn taptree(&self) -> &DeterministicTaptree {
        &self.taptree
    }

    /// The frozen reference graph pass two built.
    #[must_use]
    pub const fn reference_graph(&self) -> &FrozenReferenceGraph {
        &self.graph
    }

    /// The definition census pass one collected.
    #[must_use]
    pub const fn definitions(&self) -> &DefinitionCensus {
        &self.definitions
    }

    /// Every relation-case's concrete placement.
    #[must_use]
    pub const fn placements(&self) -> &BTreeMap<RelationCaseKey, ConcreteRelationPlacement> {
        &self.placements
    }

    /// The linked carrier closure.
    #[must_use]
    pub const fn carrier_closure(&self) -> &CarrierClosure {
        &self.closure
    }

    /// Every shape's exact transaction layout.
    #[must_use]
    pub const fn layouts(&self) -> &BTreeMap<CompactAshShape, ConcreteLayout> {
        &self.layouts
    }

    /// Every leaf's witness handoff.
    #[must_use]
    pub const fn witness_roles(&self) -> &BTreeMap<LeafRole, WitnessRole> {
        &self.witness
    }

    /// The refitted linked resource formulas.
    #[must_use]
    pub const fn formulas(
        &self,
    ) -> &BTreeMap<ProgramRole, BTreeMap<ResourceDimension, LinkedResourceFormula>> {
        &self.formulas
    }

    /// The dimensions still outstanding, and who owes each.
    ///
    /// One shorter than the relocatable bundle's list: `ControlPathDepth`
    /// was owed by the linker and the committed tree settles it, so it
    /// is a charged figure here rather than an obligation.
    #[must_use]
    pub const fn outstanding_dimensions(&self) -> &BTreeMap<ResourceDimension, ResourceObligation> {
        &self.outstanding_dimensions
    }

    /// The exact control-path depth the committed tree settles.
    #[must_use]
    pub const fn control_path_depth(&self) -> u32 {
        self.taptree.depth()
    }

    /// Every target evidence requirement still unresolved.
    #[must_use]
    pub const fn unresolved_target_evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// The outstanding clear lifecycle, carried from the backend.
    #[must_use]
    pub const fn outstanding_lifecycle(&self) -> &BTreeSet<LifecycleRequirement> {
        &self.lifecycle
    }

    /// The obligations this link created and did not discharge.
    #[must_use]
    pub const fn outstanding_obligations(&self) -> &OutstandingLinkObligations {
        &self.obligations
    }

    /// The self-commitment strategy this bundle was linked under.
    #[must_use]
    pub const fn self_commitment(&self) -> SelfCommitmentStrategy {
        self.self_commitment
    }

    /// This artifact's status (§1.9).
    ///
    /// Always [`LinkedArtifactStatus::Prototype`], and read-only. The
    /// evidence a promotion rests on is Wave 11's, does not exist, and
    /// has no field reserved for it here.
    #[must_use]
    pub const fn status(&self) -> LinkedArtifactStatus {
        LinkedArtifactStatus::Prototype
    }

    /// The exact total linked program bytes of every committed leaf.
    #[must_use]
    pub fn total_script_bytes(&self) -> u64 {
        self.programs
            .values()
            .filter_map(|program| program.charged(ResourceDimension::ScriptBytes))
            .fold(0, u64::saturating_add)
    }
}

/// Link one candidate relocatable bundle (§14).
///
/// The stages run in §14.2's order and each refuses rather than
/// degrades: pass one collects the definition census, pass two resolves
/// every reference and freezes the graph, the cycle policy of §14.4 is
/// applied to that graph, the resolved symbols are substituted before
/// serialization, the committed tree is assembled and checked against
/// the exact oracle, and the carrier census is closed against it.
///
/// # Errors
///
/// [`LinkRefusal::BundleIsNotACandidate`] for a bundle already claiming
/// more than a prototype, and any refusal the stages raise. No partial
/// bundle is returned by any path.
pub fn link_candidate(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateRelocatableTapscriptBundle,
    deployment: &LinkDeploymentParameters,
) -> Result<CandidateLinkedBundle, LinkRefusal> {
    if bundle.status() != BackendArtifactStatus::Prototype {
        return Err(LinkRefusal::BundleIsNotACandidate);
    }

    let definitions = collect_definitions(bundle, deployment)?;

    let graph = resolve_references(bundle, deployment.self_commitment())?;
    apply_cycle_policy(&graph)?;
    check_strategy_consistency(bundle, deployment)?;

    let programs = substitute(target, bundle, &definitions, deployment.resolved())?;

    let leaf_version = bundle.constructor().leaf_version();
    let tree_input = TaptreeInput::new(
        programs
            .keys()
            .map(|leaf| TapLeafInput::new(*leaf, EQUAL_LEAF_WEIGHT)),
        leaf_version,
        TreeObjective::MinimumTotalWeightedDepth,
        deployment.maximum_control_path_depth(),
    )?;
    let taptree = assemble(&tree_input)?;

    let committed: BTreeSet<LeafRole> = taptree.recipes().keys().copied().collect();
    let closure = close(bundle, &committed)?;

    let formulas = refit(bundle, &programs);
    let obligations = obligations(deployment.self_commitment());

    Ok(CandidateLinkedBundle {
        plan: bundle.plan().clone(),
        projection: bundle.target_projection().clone(),
        shapes: bundle.shapes().clone(),
        constructor: LinkedConstructor {
            contract: bundle.constructor().contract(),
            leaf_version,
            representation: bundle.constructor().representation(),
            value_policy: bundle.constructor().value_policy(),
            internal_key: deployment.internal_key().clone(),
            internal_key_policy: bundle.constructor().internal_key(),
            key_path: bundle.constructor().key_path(),
            assumptions: bundle.constructor().assumptions().clone(),
        },
        programs,
        taptree,
        graph,
        definitions,
        placements: bundle.placements().clone(),
        closure,
        layouts: bundle.layouts().clone(),
        witness: bundle
            .constructor()
            .leaves()
            .iter()
            .map(|(leaf, program)| (*leaf, program.witness().clone()))
            .collect(),
        formulas,
        outstanding_dimensions: bundle
            .outstanding_dimensions()
            .iter()
            .filter(|(dimension, _)| **dimension != ResourceDimension::ControlPathDepth)
            .map(|(dimension, obligation)| (*dimension, *obligation))
            .collect(),
        evidence: bundle.target_evidence().clone(),
        lifecycle: bundle
            .outstanding_lifecycle()
            .requirements()
            .cloned()
            .collect(),
        obligations,
        self_commitment: deployment.self_commitment(),
    })
}

/// Every leaf carries the same weight, because no execution-frequency
/// data exists.
///
/// §14.5 says so in as many words: if no execution-frequency data
/// exists, use equal weights. Inventing a frequency here would be
/// inventing a fact about how the operation gets used.
const EQUAL_LEAF_WEIGHT: NonZeroU64 = NonZeroU64::MIN;

/// Require a declared strategy to be consistent with what was emitted.
///
/// [`SelfCommitmentStrategy::IdentityIntrospection`] says the referring
/// programs read the value from the target rather than carrying a
/// literal for it. That is a checkable claim, and it is checked: a leaf
/// whose recorded relocations still push the symbol contradicts it.
fn check_strategy_consistency(
    bundle: &CandidateRelocatableTapscriptBundle,
    deployment: &LinkDeploymentParameters,
) -> Result<(), LinkRefusal> {
    if deployment.self_commitment() != SelfCommitmentStrategy::IdentityIntrospection {
        return Ok(());
    }

    no_literal_reaches_a_leaf(bundle.relocations_for(BundleSymbol::AshConstructorProgram))
}

/// Refuse if any of these relocations writes the symbol into a program.
///
/// Split from its caller so the refusal is reachable from a relocation
/// set stated directly. The bundles this crate links no longer emit
/// such a site, and a check whose failing branch could only be reasoned
/// about is a check nobody has run.
pub(crate) fn no_literal_reaches_a_leaf<'relocations>(
    relocations: impl Iterator<Item = &'relocations tapscript::Relocation>,
) -> Result<(), LinkRefusal> {
    for relocation in relocations {
        if let RelocationSite::ProgramInstructions { leaf, .. } = relocation.site() {
            return Err(LinkRefusal::CycleStrategyContradictedByRelocation {
                symbol: relocation.symbol(),
                leaf: *leaf,
            });
        }
    }

    Ok(())
}

/// The obligations a link under one strategy leaves outstanding.
fn obligations(strategy: SelfCommitmentStrategy) -> OutstandingLinkObligations {
    let mut rest = BTreeSet::from([LinkObligation::InternalKeyUnspendabilityUnverified]);
    if strategy == SelfCommitmentStrategy::ExternallyAuthenticatedCommitment {
        rest.insert(LinkObligation::SelfCommitmentEqualityUndischarged);
    }

    OutstandingLinkObligations {
        least: LinkObligation::TaprootOutputKeyUndischarged,
        rest,
    }
}

/// Refit every role's resource behaviour over the linked programs.
fn refit(
    bundle: &CandidateRelocatableTapscriptBundle,
    programs: &BTreeMap<LeafRole, LinkedLeafProgram>,
) -> BTreeMap<ProgramRole, BTreeMap<ResourceDimension, LinkedResourceFormula>> {
    let mut formulas: BTreeMap<ProgramRole, BTreeMap<ResourceDimension, LinkedResourceFormula>> =
        BTreeMap::new();

    let mut dimensions = BTreeSet::new();
    for program in programs.values() {
        dimensions.extend(program.dimensions().keys().copied());
    }

    for role in [ProgramRole::Coordinator, ProgramRole::Member] {
        for dimension in &dimensions {
            let mut measurements = BTreeMap::new();
            for shape in bundle.shapes().shapes() {
                let key = match role {
                    ProgramRole::Coordinator => LeafRole::Coordinator { shape },
                    ProgramRole::Member => LeafRole::Member {
                        ash_inputs: shape.ash_inputs(),
                    },
                };
                if let Some(measure) = programs
                    .get(&key)
                    .and_then(|program| program.charged(*dimension))
                {
                    measurements.insert(shape, measure);
                }
            }
            if measurements.is_empty() {
                continue;
            }
            let model = fit_shape_model(&measurements);
            formulas.entry(role).or_default().insert(
                *dimension,
                LinkedResourceFormula {
                    role,
                    dimension: *dimension,
                    model,
                    measurements,
                },
            );
        }
    }

    formulas
}
