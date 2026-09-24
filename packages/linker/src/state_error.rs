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
use std::ops::Range;

use tapscript::upstream::{
    DischargeBoundary, EncodedStateMetadata, ExecutionCaseId, ExternalEvidenceRequirement,
    MaturityAnnouncementRepresentationPlan as Representation, RelationId,
};
use tapscript::{
    FinalStackDefect, MaturityCarrierRefusal, StateAnnouncementId, StateConstructorRefusal,
    StateExternalEvidenceRole, StateLeafRole, StateProgramComponent, StateProgramWitness,
    TapscriptError,
};
use target_elements::{ResourceDimension, TargetContractVersion};

use crate::state_carrier::{StateDischargeClass, StateDischargeSide};
use crate::state_constructor_graph::StateReferenceGraphRefusal;
use crate::state_graph::{StateBindingTime, StateGraphNode, StateResidualComponent};
use crate::state_resource::{InitialArgumentBound, InitialArgumentWidth};
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

    /// Layout constants do not fill the prefix before the variable span.
    MetadataHeaderWidthMismatch {
        /// Bytes the constant rows supplied.
        width: usize,
        /// Expected width fixed by the variable span's start.
        variable_start: usize,
    },

    /// The codec's constant schema bytes disagree with the constructor.
    MetadataSchemaDisagreement {
        /// Schema bytes fixed by the layout.
        layout: Vec<u8>,
        /// Schema revision declared by the constructor.
        constructor: u32,
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

    // --- Relocation ---
    /// A key some instruction pushes resolves to no pushable item.
    ///
    /// The census says an instruction carries this key and the
    /// definition has no literal form, so there is nothing to place.
    /// Refusing beats skipping the site: a skipped site would leave the
    /// composition's fixture in the linked program with nothing saying
    /// so.
    UnresolvedRelocation(StateLinkSymbol),

    /// A recorded site is not a push in the pristine program.
    ///
    /// Sites are absolute instruction indices, so a site naming a
    /// primitive or naming nothing at all is a census that does not
    /// describe this program. Substituting there would overwrite an
    /// instruction the composition chose.
    SiteIsNotAPush {
        /// The key whose site it is.
        symbol: StateLinkSymbol,
        /// The instruction index.
        site: usize,
    },

    /// A recorded site lies in no component range.
    ///
    /// Every relocation is attributed to the component that emitted it,
    /// because a record naming no component could not be checked against
    /// the component census the carrier comparison closes over. An index
    /// inside the program and outside every range means the composition
    /// lost track of instructions it emitted.
    SiteOutsideEveryComponent {
        /// The key whose site it is.
        symbol: StateLinkSymbol,
        /// The instruction index.
        site: usize,
    },

    /// The witnessed static root resolved to a pushable literal.
    ///
    /// The root is bound at spend time by the tweak equation, and a
    /// literal beneath the program committing to it would be a
    /// self-commitment with no authenticated cut. A relocation for it is
    /// therefore refused rather than placed.
    LiteralStaticRootRelocation,

    /// One key's sites and the instructions its substitution moves
    /// disagree.
    ///
    /// The cross-check that keeps discovery honest: the census is read
    /// off the record's consumers, and rebuilding the program with one
    /// key's value alone must move exactly the instructions the census
    /// claims and no others. A census pointing a key at a push that
    /// carries something else is caught here, where a byte search would
    /// have called the coincidence a site.
    RelocationCensusDisagreement {
        /// The key whose sites were checked.
        symbol: StateLinkSymbol,
        /// The indices the census claims move.
        expected: BTreeSet<usize>,
        /// The indices the rebuild actually moves.
        observed: BTreeSet<usize>,
    },

    /// A relocated site does not carry its linked value.
    ///
    /// Checked by exact typed comparison against the record rather than
    /// by observing that something changed, because a site that happens
    /// to differ from its fixture is not thereby the value the link
    /// resolved.
    RelocationNotApplied {
        /// The key whose site it is.
        symbol: StateLinkSymbol,
        /// The instruction index.
        site: usize,
    },

    /// An instruction no relocation covers changed.
    ///
    /// The link declares what it moves, and everything else in the
    /// linked program must be the composition's own. A differing
    /// instruction count is reported the same way, against the first
    /// index at which the two programs can no longer be compared.
    UntrackedProgramMutation {
        /// The instruction index.
        site: usize,
    },

    /// The linked program does not decode back to itself.
    ///
    /// Substitution changes the bytes a program serializes to, so the
    /// round trip is a property of the linked artifact rather than one
    /// inherited from the pristine one.
    RoundTripMismatch,

    /// The linked program no longer schedules from the record's own
    /// precondition.
    ///
    /// The declared witness is the composition's, unchanged by a link
    /// that moves only pushed payloads, so the walk runs against the
    /// record's precondition rather than one this module invents.
    LinkedProgramDoesNotSchedule {
        /// What the walk refused.
        cause: TapscriptError,
    },

    /// The linked program's abstract execution is not the record's.
    ///
    /// Every substitution preserves width and the abstract walk types a
    /// literal by its width alone, so an execution that moved means the
    /// program's shape moved — which is a defect in the substitution and
    /// not a property of the deployment.
    AbstractExecutionMoved,

    /// One relocation's width difference is not representable.
    ///
    /// The delta is the linked push's exact encoded width minus the
    /// pristine one's, and it is computed rather than assumed so that a
    /// link which changed a width could be seen to have done so.
    ResourceDeltaOverflow {
        /// The key whose site it is.
        symbol: StateLinkSymbol,
        /// The instruction index.
        site: usize,
    },

    // --- Resources ---
    /// One dimension's checked total does not fit its unit.
    ///
    /// Checked rather than saturating: a saturated total is visibly
    /// pinned but establishes nothing, because a program whose cost
    /// overflowed reports the same pinned figure as one that did not.
    ResourceTotalOverflow {
        /// The dimension whose sum overflowed.
        dimension: ResourceDimension,
    },

    /// An emitted schedule has an initial argument whose relay admission
    /// is not established by its exact width and the reviewed policy.
    ///
    /// The first refused position is named so a link cannot hide a
    /// later refusal behind an earlier one. Replay-only schedules keep
    /// this observation without changing their linked bytes.
    InitialArgumentNotAdmitted {
        /// The argument's deepest-first position.
        position: usize,
        /// The role declared at that position.
        role: StateProgramWitness,
        /// Its exact width or its non-exact declaration.
        width: InitialArgumentWidth,
        /// The stated policy bound or its explicit absence.
        bound: InitialArgumentBound,
    },

    /// The linked program leaves the final stack in a state the rule
    /// refuses.
    ///
    /// The walk is run over the linked program rather than inherited
    /// from the composition, because the obligation is about the artifact
    /// a deployment would publish.
    LinkedProgramFailsTheFinalStackRule {
        /// Every defect the walk found, in canonical order.
        defects: Vec<FinalStackDefect>,
    },

    /// A checked total and the record's diagnostic projection disagree.
    ///
    /// The projection saturates by design, so a pinned figure is admitted
    /// beside a checked one; an unpinned difference means the two are
    /// measuring different programs. A projected dimension the totals do not measure is refused with no checked figure, because not measured is not equal.
    ResourceProjectionDisagreement {
        /// The dimension they disagree on.
        dimension: ResourceDimension,
        /// What the record's projection carries.
        diagnostic: u64,
        /// What the checked arithmetic yields, or nothing where this module measures no total for the dimension.
        checked: Option<u64>,
    },

    // --- Carrier closure ---
    /// The composed record's own carrier projection refused.
    ///
    /// Wrapped rather than restated, because the projection's reasons
    /// are about the record and this root's are about the comparison: a
    /// refusal renamed here would lose which of the two layers found the
    /// defect. No test reaches it through the public entry, and the
    /// reason is recomputed rather than assumed: the only admitted
    /// record is one exact recipe equality produced, so its component
    /// set is the complete recipe and every relation the table maps to a
    /// component finds it there, while the plan is constructible only
    /// through the entry that validates its census.
    EmittedProjection(MaturityCarrierRefusal),

    /// A relation carries no requirement in a case the plan declares.
    ///
    /// The condition a row's optional case would have represented,
    /// refused instead of represented: a row whose case were absent
    /// could state no boundaries and no activity, so it would compare
    /// nothing. No test reaches it, because the plan's own relation-case
    /// census crosses every in-scope relation with every applicable case
    /// exactly once and refuses a relation that disappears for being
    /// vacuous, structural or externally evidenced.
    MissingCompilerRelation {
        /// The relation whose case is absent.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The declared case it carries no requirement in.
        case: ExecutionCaseId,
    },

    /// A relation the plan requires has no emitted carrier.
    ///
    /// The compiler-to-emitted direction of the census. A relation the
    /// analysis raised and the record answers for nowhere is a relation
    /// whose discharge nobody stated, which is the gap the table exists
    /// to make impossible.
    MissingEmittedRow {
        /// The relation the plan requires.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
    },

    /// An emitted carrier names a relation the plan does not carry.
    ///
    /// The other direction, and the one that matters for the leaf: a
    /// discharge claimed for a relation this operation's analysis never
    /// raised is a claim about something else, and admitting it would
    /// let a table grow rows the plan could never check.
    ExtraEmittedRow {
        /// The relation the emitted table names.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
    },

    /// One relation, representation and case key two rows.
    ///
    /// The census's single-valuedness, refused rather than assumed. No
    /// test reaches it: both censuses are maps keyed by relation and the
    /// case census is a map keyed by case, so the key is unique by
    /// construction — which is a property of the containers the
    /// comparison is built from, and this variant is what would name it
    /// if one of them stopped being a map.
    DuplicateCarrierRow {
        /// The relation keyed twice.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The case keyed twice.
        case: ExecutionCaseId,
    },

    /// The emitted class is not admissible at the boundaries the plan
    /// discharges the relation at.
    ///
    /// Both sides travel, because either may be the one that moved: a
    /// class naming a component where the analysis discharges nothing at
    /// runtime or structurally, and a boundary set that no longer admits
    /// the class the record publishes, are different defects and a
    /// single name could not tell them apart.
    DischargeBoundaryDisagreement {
        /// The relation whose sides disagree.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The case the boundaries were read from.
        case: ExecutionCaseId,
        /// Every boundary the plan discharges the relation at.
        boundaries: BTreeSet<DischargeBoundary>,
        /// The class the record publishes.
        emitted: StateDischargeClass,
    },

    /// A premise the plan leaves open is visible on no side afterwards.
    ///
    /// An open premise that disappears into a comparison has been
    /// converted into a result, which is the one thing a closure over
    /// external evidence must not do. It survives as the emitted
    /// carrier, as the linked one, or as an obligation on the side that
    /// does not exist yet; anything else is this refusal.
    ExternalRequirementDropped {
        /// The relation that left the premise open.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The case that carries it.
        case: ExecutionCaseId,
        /// The premise itself, exactly as the plan states it.
        requirement: ExternalEvidenceRequirement,
    },

    /// An emitted component claims to enforce a relation with no content
    /// in this case.
    ///
    /// Vacuity is a disposition the plan states, not an omission, and a
    /// row reporting such a relation carried would claim enforcement
    /// nothing provides in the case it is claimed for.
    VacuityDisagreement {
        /// The relation the record claims to carry.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The case it is vacuous in.
        case: ExecutionCaseId,
    },

    /// The record rests a relation on a deployment fact the link's
    /// bridge did not record.
    ///
    /// The linked side of such a row says a named deployment record
    /// exists to go to. Publishing that without the bridge having
    /// recorded the role would be the unchecked claim this comparison
    /// exists to prevent, and the fact is not one this process can
    /// observe for itself.
    DeploymentFactUnrecorded {
        /// The relation resting on the fact.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The role the record named.
        role: StateExternalEvidenceRole,
    },

    /// An emitted component occupies no range in the composed program.
    ///
    /// A component with no range cannot be located in the linked leaf at
    /// all, so a row naming one would assert a discharge nobody could
    /// point at.
    MissingComponentRange {
        /// The relation the component carries.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The component with no range.
        component: StateProgramComponent,
    },

    /// A component's range runs past the end of the linked program.
    ///
    /// The range is the composition's statement about where the
    /// component sits, and the linked program is what a spend runs. A
    /// range outside it describes a program this link did not produce.
    ComponentRangeOutsideLeaf {
        /// The relation the component carries.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The component whose range it is.
        component: StateProgramComponent,
        /// The range the composition recorded.
        range: Range<usize>,
        /// The linked program's instruction count.
        length: usize,
    },

    /// The committed tree does not commit the linked leaf.
    ///
    /// A deployment publishes the tree that commits the bytes a spend
    /// runs. A tree committing some other program at this role commits a
    /// program nobody spends, so a component located in the linked leaf
    /// would be located in a leaf no control path reaches.
    CarrierLeafUncommitted {
        /// The relation whose component is in that leaf.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The leaf the link produced.
        leaf: StateLeafRole,
    },

    /// An instruction inside a component's range moved without a
    /// relocation.
    ///
    /// The link declares what it substitutes, and the tie back to the
    /// record is that everything else inside a component's range is the
    /// composition's own. An instruction that moved elsewhere means the
    /// component in the linked leaf is not the component the emitted
    /// table named.
    ComponentRangeMoved {
        /// The relation the component carries.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The component whose range it is.
        component: StateProgramComponent,
        /// The instruction index that moved.
        site: usize,
    },

    /// No alternative the compiler offered is discharged by the
    /// announcement leaf.
    ///
    /// The placement question, kept inside the discharge table: where
    /// the analysis does raise a runtime carrier obligation and the
    /// record claims the leaf carries it, one of the alternatives the
    /// analysis accepted must be a role this leaf actually plays. No
    /// test reaches it, and the reason is a property of this operation's
    /// plan rather than an assumption: every candidate set for an active
    /// runtime relation-case of this announcement includes the
    /// operation's global coordinator anchored in the singleton family,
    /// and that role is the announcement leaf.
    SelectedAlternativeUnmatched {
        /// The relation whose carrier obligation it is.
        relation: RelationId,
        /// The representation being compared.
        representation: Representation,
        /// The case the obligation is active in.
        case: ExecutionCaseId,
    },

    /// One relation resolves to two classes.
    ///
    /// The census counts relations rather than rows, which is only
    /// meaningful if a relation has one class across every case and
    /// every representation. Two classes for one relation would make the
    /// published figures a sum over something nobody stated.
    CensusNotTotal {
        /// The relation with two classes.
        relation: RelationId,
        /// The class first seen.
        first: StateDischargeClass,
        /// The class seen afterwards.
        second: StateDischargeClass,
    },

    /// The representations do not read the same.
    ///
    /// The side travels with the finding, because the three have
    /// different owners: a compiler-side difference belongs to the plan,
    /// an emitted one to the composed record, and a linked one to this
    /// link, and a reader told only that they disagree would have to
    /// re-derive which to go and look at.
    RepresentationDisagreement {
        /// The relation that reads differently.
        relation: RelationId,
        /// The representation whose row differs from the first.
        representation: Representation,
        /// The case it differs in.
        case: ExecutionCaseId,
        /// Which side differs.
        side: StateDischargeSide,
    },

    // --- The candidate bundle ---
    /// The supplied constructor commits a program other than the
    /// record's.
    ///
    /// The census is collected against that constructor, so a
    /// constructor whose subtree commits some other program would resolve
    /// this record's keys against a recipe built for different bytes. The
    /// refusal carries no payload: both programs are typed values this
    /// root would have to render to name, and a rendering of a program is
    /// not the program.
    SuppliedConstructorCommitsAnotherProgram,

    /// Applying the constructor over the linked subtree refused.
    ///
    /// Wrapped rather than restated, for the reason the emitted
    /// projection is: the construction's reasons are about metadata, a
    /// nonce budget and a curve, and this root's are about the link. A
    /// refusal renamed here would lose which of the two layers found the
    /// defect.
    ConstructorApplication(StateConstructorRefusal),

    /// One metadata-independent reference did not survive the
    /// application.
    ///
    /// The reference half of the fixed point. The static root must be the
    /// linked subtree's, because that is the subtree the application was
    /// handed; the metadata schema and the branch side must be the
    /// supplied constructor's, because they are constants of the recipe
    /// rather than functions of a subtree. A reference that moved would
    /// mean the applied constructor is not the supplied one over new
    /// bytes but a different recipe.
    AppliedReferenceDisagreement {
        /// The reference kind that did not agree.
        symbol: StateLinkSymbol,
    },

    /// A pushed key's resolved entry moved when the constructor was
    /// applied.
    ///
    /// The census half of the fixed point, and the whole reason the
    /// application can be run after substitution. The linked leaf was
    /// built from the census collected against the supplied constructor;
    /// if re-collecting it against the applied one moved a key some
    /// instruction pushes, the leaf those bytes belong to is not the leaf
    /// the subtree commits, and there would be no order in which the two
    /// could be run.
    CensusMovedUnderApplication {
        /// The pushed key whose resolved entry differs.
        symbol: StateLinkSymbol,
    },

    /// A predecessor and a successor static subtree differ.
    ///
    /// The bundle's continuity equality, refused rather than migrated:
    /// no migration between static subtrees is implemented, so the two
    /// roots travel and nothing is reconciled. Two subtrees whose roots
    /// read alike here differ in something the root does not commit, the
    /// caller-supplied leaf identity among it.
    StaticSubtreeDiscontinuity {
        /// The predecessor subtree's root.
        predecessor: [u8; 32],
        /// The successor subtree's root.
        successor: [u8; 32],
    },

    /// One semantic metadata already has a retained instance.
    ///
    /// Retention is keyed by the semantic metadata, because the same
    /// metadata over one subtree under one policy is one constructor and
    /// a second copy of it would be the same artifact carried twice. The
    /// refusal names the instance already held rather than the metadata
    /// just offered, which the caller has: a nonce would not identify it,
    /// since two different metadata values may both select the first
    /// nonce the budget admits.
    InstanceAlreadyRetained {
        /// The exact metadata, nonce included, of the instance held.
        metadata: EncodedStateMetadata,
    },

    /// The constructor states no value for one of its policy kinds.
    ///
    /// The constructor exposes its metadata-independent policy only
    /// through its reference declarations, so the policy is read from
    /// there rather than from accessors that do not exist. No test
    /// reaches this, and the reason is recomputed rather than assumed:
    /// those declarations are a fixed array with one entry per reference
    /// kind, built by the constructor from its own fields, so no kind can
    /// be absent — this variant is what would name it on the day that
    /// array stopped being total.
    ConstructorPolicyIncomplete {
        /// The policy kind the declarations did not carry.
        missing: StateLinkSymbol,
    },
}
