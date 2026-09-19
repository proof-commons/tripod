//! The maturity link's three-sided carrier closure and its outstanding
//! ABI contract.
//!
//! # Three sides, compared rather than believed
//!
//! The compiler states what each relation of the announcement requires
//! and at which boundary it is discharged. The backend states what
//! discharges it — an emitted component, a named fact about the
//! deployment, the realization's own outstanding claim, or nothing this
//! system enforces. The link states where that discharge actually sits
//! in the leaf a deployment would publish. This module puts one row per
//! relation, per representation and per execution case in front of all
//! three and refuses by name wherever they disagree.
//!
//! None of the three sides is taken on trust. The emitted table is
//! recomputed from the composed record rather than read from a field
//! somebody filled in; the boundaries and cases are read from the
//! validated plan the link's own bridge carries, so the comparison
//! cannot be made against a plan this leaf was not emitted for; and the
//! linked side is located in the linked program and checked against the
//! tree that commits it.
//!
//! # Why the closure is over the discharge table and not over placements
//!
//! The earlier generations close over placements: the backend records
//! which concrete sites carry which relation-case, and the closure asks
//! whether those sites are the ones an accepted alternative named and
//! whether each is still reachable in the committed tree. That question
//! has no subject here. The reduced announcement leaf carries no
//! positional sponsor carriers at all: the sponsor relations close by
//! substrate conservation and by the self-position pin rather than by
//! anything the bytes inspect, and ten of the twenty-six relations have
//! no carrier in the leaf to point at. A placement closure would have
//! nothing to compare for those ten and would have to omit them, which
//! is exactly the omission that makes a census uncheckable.
//!
//! The discharge table has a row for every relation whatever discharges
//! it, so the closure ranges over all twenty-six. The placement question
//! survives inside it rather than being dropped: where the compiler does
//! raise a runtime carrier obligation and the leaf does carry it, the
//! row records which of the alternatives the compiler offered the
//! announcement leaf answers to.
//!
//! # Vacuity and external requirements are preserved, never absorbed
//!
//! A relation the compiler marks vacuous in a case has no content in
//! that case, and a row that reported it carried would be claiming
//! enforcement nothing provides. Such a row therefore carries
//! [`StateLinkedCarrier::Vacuous`] and refuses an emitted component
//! claiming to enforce it.
//!
//! An external requirement is the exact premise a relation leaves open,
//! and a closure that quietly counted it as discharged would be
//! converting an open premise into a result. Every requirement a case
//! carries must therefore still be visible afterwards — as the emitted
//! carrier, as the linked one, or, for the one relation whose signature
//! the leaf verifies while its membership half stays outstanding, as an
//! obligation on the side that does not exist yet.
//!
//! # The fourth side is a contract, not a claim
//!
//! There is no ABI. What exists is the exact list of what one must
//! supply, keyed by the same relation and representation as the rows, so
//! that the wave which builds it answers row by row instead of starting
//! from prose. [`StateAbiObligation`] says so in its name and its
//! documentation, it is counted in no census of discharges, and nothing
//! here treats a listed obligation as met.
//!
//! # The model-scope rows stay visible
//!
//! Ten rows name a relation the realization evaluates over the whole
//! observed transaction while no leaf enforces it. They are published
//! rather than dropped because the gap is real: until the realization's
//! region-scoping refit re-scopes those relations to the region an
//! operation claims, a transaction composing several operations is
//! accepted on-chain and outside the model. A closure that omitted them
//! would read as though the model already covered them.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use tapscript::upstream::{
    CarrierAssignmentAlternative, CarrierRole, DischargeBoundary, ExecutionCaseId,
    ExternalEvidenceRequirement, MaturityAnnouncementRepresentationPlan as Representation,
    MaturityAnnouncementRepresentationProjection, RelationActivity, RelationId, TargetRelationCase,
    TargetRelationRequirement, ValidatedMaturityAnnouncementOperationPlan,
};
use tapscript::{
    MaturityCarrier, StateAnnouncementId, StateAnnouncementProgram, StateExternalEvidenceRole,
    StateLeafRole, StateOperatorPatternId, StateProgramComponent, StateProgramWitness,
    StateStaticSubtree, TapscriptProgram, project_maturity_carriers,
};

use crate::state_deployment::StateLinkDeploymentParameters;
use crate::state_error::StateLinkRefusal;
use crate::state_relocate::LinkedStateLeafProgram;
use crate::state_taptree::StateLinkedTaptree;

// --- The three sides ---------------------------------------------------

/// Which of the three sides a comparison found to differ.
///
/// Named rather than left implicit in a refusal's wording, because the
/// three sides have different owners: a compiler-side difference belongs
/// to the plan, an emitted-side one to the composed record, and a
/// linked-side one to this link. A reader told only that "the
/// representations disagree" would have to re-derive which of the three
/// to go and look at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateDischargeSide {
    /// The validated plan's requirement, case and boundaries.
    Compiler,
    /// The composed record's carrier for the relation.
    Emitted,
    /// Where the linked leaf actually discharges it.
    Linked,
}

/// The emitted table's four classes, without their payloads.
///
/// The payloads distinguish one component or one open premise from
/// another; the class is what a boundary comparison and a census are
/// about, and carrying the payload into either would make two rows
/// naming different components look like two different kinds of
/// discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateDischargeClass {
    /// A component of the composed announcement recipe.
    Emitted,
    /// A named fact about the deployment, checked by no instruction.
    Deployment,
    /// The realization's own outstanding requirement.
    External,
    /// Evaluated over the whole observed transaction, enforced by no leaf.
    ModelScope,
}

impl StateDischargeClass {
    /// The class of one carrier.
    #[must_use]
    pub const fn of(carrier: &MaturityCarrier) -> Self {
        match carrier {
            MaturityCarrier::Emitted(_) => Self::Emitted,
            MaturityCarrier::Deployment(_) => Self::Deployment,
            MaturityCarrier::External(_) => Self::External,
            MaturityCarrier::ModelScope => Self::ModelScope,
        }
    }
}

// --- One row -----------------------------------------------------------

/// Where one relation's discharge sits in the linked leaf.
///
/// Five answers, because the emitted table has four classes and a
/// vacuous case has no discharge to locate at all. A component is
/// located at its actual range; a deployment fact is one the link's
/// bridge recorded; an external requirement travels exactly as the plan
/// states it; and the last two carry nothing, which is the point of
/// each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateLinkedCarrier {
    /// An emitted component found at its range inside the committed and
    /// linked leaf.
    Component {
        /// The component the emitted table names.
        component: StateProgramComponent,
        /// The leaf the linked program is.
        leaf: StateLeafRole,
        /// Its instruction range in that program.
        range: Range<usize>,
    },
    /// A fact about the deployment that the link's bridge recorded.
    DeploymentFact(StateExternalEvidenceRole),
    /// An outstanding requirement the plan carries, unchanged.
    ExternalRequirement(ExternalEvidenceRequirement),
    /// The realization evaluates this relation over the whole observed
    /// transaction and no leaf enforces it.
    ModelScope,
    /// The relation has no content in this execution case.
    Vacuous,
}

/// One relation of the announcement, in one representation and one case,
/// with all three sides beside each other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateCarrierRow {
    relation: RelationId,
    representation: Representation,
    case: ExecutionCaseId,
    activity: RelationActivity,
    boundaries: BTreeSet<DischargeBoundary>,
    emitted: MaturityCarrier,
    class: StateDischargeClass,
    linked: StateLinkedCarrier,
    external_requirements: BTreeSet<ExternalEvidenceRequirement>,
    selected: Option<CarrierAssignmentAlternative>,
}

impl StateCarrierRow {
    /// The relation this row is about.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    /// The representation the row was built under.
    #[must_use]
    pub const fn representation(&self) -> Representation {
        self.representation
    }

    /// The execution case the row was built in.
    ///
    /// Always present, never optional: the plan's own relation-case
    /// census crosses every in-scope relation with every applicable
    /// case, so a relation with no entry for a declared case is a defect
    /// rather than a shape this row has to represent, and
    /// [`StateLinkRefusal::MissingCompilerRelation`] is what names it.
    #[must_use]
    pub const fn case(&self) -> &ExecutionCaseId {
        &self.case
    }

    /// Whether the relation is active or vacuous in that case.
    #[must_use]
    pub const fn activity(&self) -> RelationActivity {
        self.activity
    }

    /// Every boundary the compiler discharges the relation at.
    #[must_use]
    pub const fn boundaries(&self) -> &BTreeSet<DischargeBoundary> {
        &self.boundaries
    }

    /// What the composed record says discharges the relation.
    #[must_use]
    pub const fn emitted(&self) -> &MaturityCarrier {
        &self.emitted
    }

    /// That carrier's class.
    #[must_use]
    pub const fn class(&self) -> StateDischargeClass {
        self.class
    }

    /// Where the linked leaf discharges it.
    #[must_use]
    pub const fn linked(&self) -> &StateLinkedCarrier {
        &self.linked
    }

    /// Every premise the case leaves open.
    #[must_use]
    pub const fn external_requirements(&self) -> &BTreeSet<ExternalEvidenceRequirement> {
        &self.external_requirements
    }

    /// The carrier alternative the announcement leaf answers to.
    ///
    /// Present only where the compiler raises a runtime carrier
    /// obligation for an active case and the leaf carries it. It is
    /// absent on every other row on purpose: a relation discharged
    /// compiler-statically or structurally has no carrier obligation to
    /// select from, a deployment fact and an outstanding premise are not
    /// the leaf, and a model-scope row is precisely one where the
    /// compiler offers the announcement leaf and the leaf does not carry
    /// it.
    #[must_use]
    pub const fn selected(&self) -> Option<&CarrierAssignmentAlternative> {
        self.selected.as_ref()
    }
}

// --- The outstanding contract ------------------------------------------

/// One thing an ABI must supply, stated exactly and discharged by
/// nothing.
///
/// The name is the claim: this is an obligation on a side that does not
/// exist yet. It is derived from the record's own witness schedule, the
/// plan's state, sponsor and operator projections, and the facts and
/// premises the link's bridge and the plan carry — never invented here —
/// and it is keyed like a row so that the wave supplying the ABI can
/// answer one relation at a time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateAbiObligation {
    relation: RelationId,
    representation: Representation,
    requirement: StateAbiRequirement,
}

impl StateAbiObligation {
    /// The relation the obligation belongs to.
    #[must_use]
    pub const fn relation(&self) -> &RelationId {
        &self.relation
    }

    /// The representation it was derived under.
    #[must_use]
    pub const fn representation(&self) -> Representation {
        self.representation
    }

    /// What must be supplied.
    #[must_use]
    pub const fn requirement(&self) -> &StateAbiRequirement {
        &self.requirement
    }
}

/// What an ABI must supply for one relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateAbiRequirement {
    /// One item of the record's declared witness schedule.
    WitnessRole(StateProgramWitness),
    /// The STATE input slot the plan's recognition and count describe.
    InputSlot,
    /// The STATE successor slot the plan's recognition and count describe.
    SuccessorSlot,
    /// The operator's witness, whose class and node the plan names.
    OperatorWitness,
    /// A deployment fact the link's bridge recorded and verified not at
    /// all.
    DeploymentFact(StateExternalEvidenceRole),
    /// A premise the plan leaves open, carried exactly as it states it.
    ExternalRequirement(ExternalEvidenceRequirement),
    /// The optional sponsor envelope the plan's sponsor projection
    /// describes.
    SponsorRegion,
}

// --- The closure -------------------------------------------------------

/// The checked comparison of all three sides, with the fourth stated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateCarrierClosure {
    rows: Vec<StateCarrierRow>,
    obligations: Vec<StateAbiObligation>,
    census: BTreeMap<StateDischargeClass, usize>,
}

impl StateCarrierClosure {
    /// Every row, by representation, then relation, then case.
    #[must_use]
    pub fn rows(&self) -> &[StateCarrierRow] {
        &self.rows
    }

    /// One row by its complete key.
    #[must_use]
    pub fn row(
        &self,
        relation: &RelationId,
        representation: Representation,
        case: &ExecutionCaseId,
    ) -> Option<&StateCarrierRow> {
        self.rows.iter().find(|row| {
            row.relation == *relation && row.representation == representation && row.case == *case
        })
    }

    /// Everything the ABI must supply, in derivation order.
    ///
    /// A vector rather than a set, because the record's witness schedule
    /// is ordered by the record and that order is a statement of its
    /// own: reordering it would discard what the record said about the
    /// depth its own items sit at.
    #[must_use]
    pub fn obligations(&self) -> &[StateAbiObligation] {
        &self.obligations
    }

    /// How many relations each class discharges.
    ///
    /// Counted per relation, not per row: a relation's carrier does not
    /// change with the case or the representation, and counting rows
    /// would multiply one discharge by the size of the case census.
    #[must_use]
    pub const fn census(&self) -> &BTreeMap<StateDischargeClass, usize> {
        &self.census
    }

    /// Every representation the closure carries rows for.
    pub fn representations(&self) -> impl Iterator<Item = Representation> + '_ {
        self.rows
            .iter()
            .map(|row| row.representation)
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    /// Whether the three sides read the same in every representation.
    ///
    /// A reader's own recomputation of what the constructor enforced,
    /// not a stored verdict. The selected alternative is deliberately
    /// outside the comparison: an alternative's layout dependencies may
    /// name the very representation the case fixed, so demanding
    /// equality there would be comparing the representations rather than
    /// the closure.
    #[must_use]
    pub fn is_identical_across_representations(&self) -> bool {
        let mut blocks: BTreeMap<Representation, Vec<&StateCarrierRow>> = BTreeMap::new();
        for row in &self.rows {
            blocks.entry(row.representation).or_default().push(row);
        }
        let mut blocks = blocks.into_values();
        let Some(first) = blocks.next() else {
            return true;
        };
        blocks.all(|block| {
            block.len() == first.len()
                && first
                    .iter()
                    .zip(&block)
                    .all(|(left, right)| disagreeing_side(left, right).is_none())
        })
    }
}

// --- Closing -----------------------------------------------------------

/// Close the maturity link's carriers on three sides.
///
/// The plan is read from the bridge rather than supplied beside it, for
/// the reason the earlier generations read theirs from the bundle: one
/// fact wants one carrier, and a caller able to hand in a second plan
/// could compare this leaf against requirements it was never emitted
/// for.
///
/// # Errors
///
/// [`StateLinkRefusal::EmittedProjection`] wrapping whatever the record's
/// own carrier projection refuses, and then every refusal the
/// crate-private `close_state_carrier_parts` entry raises over the
/// result.
pub fn close_state_carriers(
    record: &StateAnnouncementProgram,
    deployment: &StateLinkDeploymentParameters,
    linked: &LinkedStateLeafProgram,
    taptree: &StateLinkedTaptree,
) -> Result<StateCarrierClosure, StateLinkRefusal> {
    let plan = deployment.plan();
    let projection =
        project_maturity_carriers(plan, record).map_err(StateLinkRefusal::EmittedProjection)?;

    let mut emitted = BTreeMap::new();
    for representation in plan.representations() {
        let mode = representation.plan();
        if let Some(table) = projection.projection(mode) {
            emitted.insert(mode, table.clone());
        }
    }

    let witness: Vec<StateProgramWitness> =
        record.witness().iter().map(|(role, _)| *role).collect();
    let sites = linked.relocations().sites();

    close_state_carrier_parts(&StateCarrierParts {
        plan,
        emitted: &emitted,
        witness: &witness,
        ranges: record.components(),
        pristine: record.program(),
        deployment,
        leaf: linked.leaf(),
        linked: linked.program(),
        sites: &sites,
        committed: taptree.subtree(),
    })
}

/// The pieces one closure is computed from.
///
/// Separated from the entry above for the reason the record's own
/// projection separates its parts: a test showing what an absent range,
/// a dropped row or a moved instruction is refused by would otherwise
/// have to counterfeit a composed record, and a counterfeit record would
/// let the refusal be demonstrated against an artifact no backend
/// produced.
pub(crate) struct StateCarrierParts<'a> {
    /// The compiler side, as the bridge validated it.
    pub plan: &'a ValidatedMaturityAnnouncementOperationPlan,
    /// The emitted table, one map per representation.
    pub emitted: &'a BTreeMap<Representation, BTreeMap<RelationId, MaturityCarrier>>,
    /// The record's declared witness roles, in its own order.
    pub witness: &'a [StateProgramWitness],
    /// Every component's instruction range in the composed program.
    pub ranges: &'a BTreeMap<StateProgramComponent, Range<usize>>,
    /// The composed program before substitution.
    pub pristine: &'a TapscriptProgram,
    /// The link's bound sources, for the facts it recorded.
    pub deployment: &'a StateLinkDeploymentParameters,
    /// The role of the leaf the link produced.
    pub leaf: StateLeafRole,
    /// The linked program itself.
    pub linked: &'a TapscriptProgram,
    /// Every instruction index the link substituted at.
    pub sites: &'a BTreeSet<usize>,
    /// The subtree the constructor committed.
    pub committed: &'a StateStaticSubtree,
}

/// Compare the three sides over one set of parts.
///
/// # Errors
///
/// [`StateLinkRefusal::ExtraEmittedRow`] and
/// [`StateLinkRefusal::MissingEmittedRow`] in either direction of the
/// relation census, [`StateLinkRefusal::MissingCompilerRelation`] for a
/// declared case a relation carries no requirement in,
/// [`StateLinkRefusal::DuplicateCarrierRow`] for one key twice,
/// [`StateLinkRefusal::DischargeBoundaryDisagreement`],
/// [`StateLinkRefusal::ExternalRequirementDropped`],
/// [`StateLinkRefusal::VacuityDisagreement`],
/// [`StateLinkRefusal::DeploymentFactUnrecorded`],
/// [`StateLinkRefusal::MissingComponentRange`],
/// [`StateLinkRefusal::ComponentRangeOutsideLeaf`],
/// [`StateLinkRefusal::CarrierLeafUncommitted`],
/// [`StateLinkRefusal::ComponentRangeMoved`],
/// [`StateLinkRefusal::SelectedAlternativeUnmatched`],
/// [`StateLinkRefusal::CensusNotTotal`] and
/// [`StateLinkRefusal::RepresentationDisagreement`].
pub(crate) fn close_state_carrier_parts(
    parts: &StateCarrierParts<'_>,
) -> Result<StateCarrierClosure, StateLinkRefusal> {
    let empty = BTreeMap::new();
    let committed = committed_leaf_is_the_linked_one(parts);
    let mut blocks: Vec<Vec<StateCarrierRow>> = Vec::new();

    for projection in parts.plan.representations() {
        let table = parts.emitted.get(&projection.plan()).unwrap_or(&empty);
        blocks.push(representation_rows(parts, projection, table, committed)?);
    }

    let census = class_census(blocks.iter().flatten())?;
    representations_agree(&blocks)?;

    Ok(StateCarrierClosure {
        rows: blocks.into_iter().flatten().collect(),
        obligations: abi_obligations(parts),
        census,
    })
}

/// Whether the tree commits, at the linked leaf's role, the very program
/// the link produced.
///
/// The question is about the linked bytes and not the composed ones. A
/// deployment publishes the tree that commits what a spend runs, so a
/// tree committing the record's pre-link program while the linked leaf
/// differs commits a program nobody spends.
fn committed_leaf_is_the_linked_one(parts: &StateCarrierParts<'_>) -> bool {
    parts
        .committed
        .leaves()
        .iter()
        .any(|entry| entry.leaf.role == parts.leaf && entry.leaf.program == *parts.linked)
}

/// Every row of one representation, both census directions first.
fn representation_rows(
    parts: &StateCarrierParts<'_>,
    projection: &MaturityAnnouncementRepresentationProjection,
    table: &BTreeMap<RelationId, MaturityCarrier>,
    committed: bool,
) -> Result<Vec<StateCarrierRow>, StateLinkRefusal> {
    let representation = projection.plan();

    for relation in table.keys() {
        if projection.relation(relation).is_none() {
            return Err(StateLinkRefusal::ExtraEmittedRow {
                relation: relation.clone(),
                representation,
            });
        }
    }

    let mut rows = Vec::new();
    let mut keys = BTreeSet::new();

    for requirement in projection.relations() {
        let carrier = table.get(&requirement.relation).ok_or_else(|| {
            StateLinkRefusal::MissingEmittedRow {
                relation: requirement.relation.clone(),
                representation,
            }
        })?;

        for declared in projection.cases() {
            let case = requirement.cases.get(&declared.id).ok_or_else(|| {
                StateLinkRefusal::MissingCompilerRelation {
                    relation: requirement.relation.clone(),
                    representation,
                    case: declared.id.clone(),
                }
            })?;

            if !keys.insert((requirement.relation.clone(), declared.id.clone())) {
                return Err(StateLinkRefusal::DuplicateCarrierRow {
                    relation: requirement.relation.clone(),
                    representation,
                    case: declared.id.clone(),
                });
            }

            rows.push(carrier_row(
                parts,
                projection,
                requirement,
                case,
                carrier,
                committed,
            )?);
        }
    }

    Ok(rows)
}

/// One row, with the three sides held to each other.
fn carrier_row(
    parts: &StateCarrierParts<'_>,
    projection: &MaturityAnnouncementRepresentationProjection,
    requirement: &TargetRelationRequirement,
    case: &TargetRelationCase,
    carrier: &MaturityCarrier,
    committed: bool,
) -> Result<StateCarrierRow, StateLinkRefusal> {
    let representation = projection.plan();
    let relation = &requirement.relation;
    let class = StateDischargeClass::of(carrier);
    let authorization = relation == parts.plan.operator().authorization();

    if !class_agrees(class, &case.boundaries, authorization) {
        return Err(StateLinkRefusal::DischargeBoundaryDisagreement {
            relation: relation.clone(),
            representation,
            case: case.key.case.clone(),
            boundaries: case.boundaries.clone(),
            emitted: class,
        });
    }

    let linked = linked_carrier(parts, representation, relation, case, carrier, committed)?;

    for open in &case.external_evidence {
        if !external_preserved(parts.plan, relation, carrier, &linked, open) {
            return Err(StateLinkRefusal::ExternalRequirementDropped {
                relation: relation.clone(),
                representation,
                case: case.key.case.clone(),
                requirement: open.clone(),
            });
        }
    }

    let selected = selected_alternative(projection, parts.plan, case, class)?;

    Ok(StateCarrierRow {
        relation: relation.clone(),
        representation,
        case: case.key.case.clone(),
        activity: case.activity,
        boundaries: case.boundaries.clone(),
        emitted: carrier.clone(),
        class,
        linked,
        external_requirements: case.external_evidence.clone(),
        selected,
    })
}

/// Whether one class is admissible at the boundaries the plan discharges
/// the relation at.
///
/// Three shapes admit an emitted component, and the reason differs in
/// each. A runtime boundary is a check in the leaf's own bytes. The
/// compiler-static and backend-structural pair is an obligation on the
/// emitted artifact, and the component is what makes it true of this
/// one: the successor's representation is encoded and authenticated by
/// the component that reconstructs it, and a required lifecycle exit is
/// retained by the component that checks it. The external boundary
/// admits a component for exactly one relation — the operator's
/// authorization, whose signature the leaf verifies in script while the
/// membership half of the same premise stays outstanding — and for no
/// other, because an external relation reaching a carrier anywhere else
/// would be a discharge nobody performed.
///
/// A deployment fact and a model-scope row both stand where a runtime
/// carrier was expected and neither is one, which is what makes naming
/// them worth doing rather than leaving the row blank.
fn class_agrees(
    class: StateDischargeClass,
    boundaries: &BTreeSet<DischargeBoundary>,
    authorization: bool,
) -> bool {
    let runtime = boundaries.contains(&DischargeBoundary::RuntimeCarrier);
    let structural = boundaries.contains(&DischargeBoundary::CompilerStatic)
        && boundaries.contains(&DischargeBoundary::BackendStructural);
    let external = boundaries.contains(&DischargeBoundary::ExternalEvidence);

    match class {
        StateDischargeClass::Emitted => runtime || structural || (authorization && external),
        StateDischargeClass::Deployment | StateDischargeClass::ModelScope => runtime,
        StateDischargeClass::External => external,
    }
}

/// Where the linked leaf discharges one row.
fn linked_carrier(
    parts: &StateCarrierParts<'_>,
    representation: Representation,
    relation: &RelationId,
    case: &TargetRelationCase,
    carrier: &MaturityCarrier,
    committed: bool,
) -> Result<StateLinkedCarrier, StateLinkRefusal> {
    if case.activity == RelationActivity::Vacuous {
        if matches!(carrier, MaturityCarrier::Emitted(_)) {
            return Err(StateLinkRefusal::VacuityDisagreement {
                relation: relation.clone(),
                representation,
                case: case.key.case.clone(),
            });
        }
        return Ok(StateLinkedCarrier::Vacuous);
    }

    match carrier {
        MaturityCarrier::Emitted(component) => {
            component_carrier(parts, representation, relation, *component, committed)
        }
        MaturityCarrier::Deployment(role) => {
            if parts.deployment.deployment_facts().contains(role) {
                Ok(StateLinkedCarrier::DeploymentFact(*role))
            } else {
                Err(StateLinkRefusal::DeploymentFactUnrecorded {
                    relation: relation.clone(),
                    representation,
                    role: *role,
                })
            }
        }
        MaturityCarrier::External(open) => {
            Ok(StateLinkedCarrier::ExternalRequirement(open.clone()))
        }
        MaturityCarrier::ModelScope => Ok(StateLinkedCarrier::ModelScope),
    }
}

/// Locate one emitted component inside the committed and linked leaf.
fn component_carrier(
    parts: &StateCarrierParts<'_>,
    representation: Representation,
    relation: &RelationId,
    component: StateProgramComponent,
    committed: bool,
) -> Result<StateLinkedCarrier, StateLinkRefusal> {
    let range =
        parts
            .ranges
            .get(&component)
            .ok_or_else(|| StateLinkRefusal::MissingComponentRange {
                relation: relation.clone(),
                representation,
                component,
            })?;

    if range.end > parts.linked.len() {
        return Err(StateLinkRefusal::ComponentRangeOutsideLeaf {
            relation: relation.clone(),
            representation,
            component,
            range: range.clone(),
            length: parts.linked.len(),
        });
    }

    // Local before global: first that the component in the linked leaf is
    // the component the record composed, and only then that the tree
    // publishes the leaf it sits in. A range whose instructions moved is
    // a defect of this link whichever tree is bound over it.
    let moved = parts
        .pristine
        .instructions()
        .iter()
        .zip(parts.linked.instructions())
        .enumerate()
        .skip(range.start)
        .take(range.len())
        .find(|&(site, (before, after))| before != after && !parts.sites.contains(&site))
        .map(|(site, _)| site);

    if let Some(site) = moved {
        return Err(StateLinkRefusal::ComponentRangeMoved {
            relation: relation.clone(),
            representation,
            component,
            site,
        });
    }

    if !committed {
        return Err(StateLinkRefusal::CarrierLeafUncommitted {
            relation: relation.clone(),
            representation,
            leaf: parts.leaf,
        });
    }

    Ok(StateLinkedCarrier::Component {
        component,
        leaf: parts.leaf,
        range: range.clone(),
    })
}

/// Whether one open premise is still visible after the comparison.
fn external_preserved(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    relation: &RelationId,
    carrier: &MaturityCarrier,
    linked: &StateLinkedCarrier,
    requirement: &ExternalEvidenceRequirement,
) -> bool {
    if let MaturityCarrier::External(emitted) = carrier
        && emitted == requirement
    {
        return true;
    }
    if let StateLinkedCarrier::ExternalRequirement(placed) = linked
        && placed == requirement
    {
        return true;
    }
    // The one relation whose in-script half is emitted while the same
    // premise's membership half stays outstanding. It survives as an
    // obligation on the side that does not exist yet rather than as a
    // carrier, which is why the third route is stated here instead of
    // being absorbed by the first two.
    relation == plan.operator().authorization() && requirement == plan.operator().evidence()
}

/// The alternative the announcement leaf answers to, where the compiler
/// raised a carrier obligation and the leaf carries it.
fn selected_alternative(
    projection: &MaturityAnnouncementRepresentationProjection,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    case: &TargetRelationCase,
    class: StateDischargeClass,
) -> Result<Option<CarrierAssignmentAlternative>, StateLinkRefusal> {
    let carried = class == StateDischargeClass::Emitted
        && case.activity == RelationActivity::Active
        && case.boundaries.contains(&DischargeBoundary::RuntimeCarrier);

    if !carried {
        return Ok(None);
    }

    projection
        .carriers()
        .find(|requirement| requirement.relation_case == case.key)
        .and_then(|requirement| {
            requirement.alternatives.iter().find(|alternative| {
                !alternative.carriers.is_empty()
                    && alternative
                        .carriers
                        .iter()
                        .all(|placed| announcement_leaf_role(plan, &placed.carrier))
            })
        })
        .cloned()
        .map(Some)
        .ok_or_else(|| StateLinkRefusal::SelectedAlternativeUnmatched {
            relation: case.key.relation.clone(),
            representation: projection.plan(),
            case: case.key.case.clone(),
        })
}

/// Whether one abstract carrier role is the announcement leaf.
///
/// The leaf is the operation's only production program and it executes
/// at input zero, which is the singleton family's only input member, so
/// it is at once the operation's global coordinator anchored in that
/// family, that family's input coordinator, and every member of it. A
/// role over any other family is not this leaf: the sponsor region
/// carries no program in this candidate at all. The two non-runtime
/// roles are not programs either, so neither is this one.
fn announcement_leaf_role(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    role: &CarrierRole,
) -> bool {
    match role {
        CarrierRole::OperationGlobal { operation, anchor } => {
            *operation == plan.operation() && anchor == plan.state().object()
        }
        CarrierRole::InputFamilyCoordinator { object }
        | CarrierRole::EveryInputFamilyMember { object } => object == plan.state().object(),
        CarrierRole::BackendStructural { .. } | CarrierRole::ExternalEvidence { .. } => false,
    }
}

/// How many relations each class discharges, refusing a relation with
/// two classes.
fn class_census<'a>(
    rows: impl Iterator<Item = &'a StateCarrierRow>,
) -> Result<BTreeMap<StateDischargeClass, usize>, StateLinkRefusal> {
    let mut classes: BTreeMap<&RelationId, StateDischargeClass> = BTreeMap::new();

    for row in rows {
        if let Some(first) = classes.insert(&row.relation, row.class)
            && first != row.class
        {
            return Err(StateLinkRefusal::CensusNotTotal {
                relation: row.relation.clone(),
                first,
                second: row.class,
            });
        }
    }

    let mut census = BTreeMap::new();
    for class in classes.into_values() {
        *census.entry(class).or_insert(0_usize) += 1;
    }
    Ok(census)
}

/// Hold every representation's rows to the first representation's.
fn representations_agree(blocks: &[Vec<StateCarrierRow>]) -> Result<(), StateLinkRefusal> {
    let Some((first, rest)) = blocks.split_first() else {
        return Ok(());
    };

    for block in rest {
        if block.len() != first.len() {
            let shorter = first.len().min(block.len());
            let longer = if block.len() > first.len() {
                block
            } else {
                first
            };
            if let Some(row) = longer.get(shorter) {
                return Err(StateLinkRefusal::RepresentationDisagreement {
                    relation: row.relation.clone(),
                    representation: row.representation,
                    case: row.case.clone(),
                    side: StateDischargeSide::Compiler,
                });
            }
        }

        for (left, right) in first.iter().zip(block) {
            if let Some(side) = disagreeing_side(left, right) {
                return Err(StateLinkRefusal::RepresentationDisagreement {
                    relation: right.relation.clone(),
                    representation: right.representation,
                    case: right.case.clone(),
                    side,
                });
            }
        }
    }

    Ok(())
}

/// Which side two rows of one relation differ on, if any.
fn disagreeing_side(left: &StateCarrierRow, right: &StateCarrierRow) -> Option<StateDischargeSide> {
    if left.relation != right.relation
        || left.activity != right.activity
        || left.boundaries != right.boundaries
        || left.external_requirements != right.external_requirements
    {
        return Some(StateDischargeSide::Compiler);
    }
    if left.emitted != right.emitted {
        return Some(StateDischargeSide::Emitted);
    }
    if left.linked != right.linked {
        return Some(StateDischargeSide::Linked);
    }
    None
}

// --- The fourth side ---------------------------------------------------

/// The component that reads one witness item.
///
/// Every item of the record's schedule is read by one component, and the
/// attribution is exhaustive so that a role added later cannot be
/// absorbed into a default. Five of the seven are roles the
/// authenticated graph's own evidence table already names, and each is
/// attributed here to a component that table names for a binding time
/// carrying it, so the two statements cannot drift apart. The other two
/// are the window's requested cycle, which is the operand the inclusive
/// window compares, and the operator's signature, which is the item the
/// committed-key fragment verifies; neither is a cut's evidence, so
/// neither appears in that table at all.
pub(crate) const fn state_witness_component(role: StateProgramWitness) -> StateProgramComponent {
    match role {
        StateProgramWitness::SuccessorOutputKeyPrefix | StateProgramWitness::SuccessorNonce => {
            StateProgramComponent::Semantic(StateAnnouncementId::SuccessorReconstruction)
        }
        StateProgramWitness::RequestedCycle => {
            StateProgramComponent::Semantic(StateAnnouncementId::LeadWindow)
        }
        StateProgramWitness::StaticSubtreeRoot
        | StateProgramWitness::PredecessorMetadata
        | StateProgramWitness::PredecessorOutputKeyPrefix => {
            StateProgramComponent::Semantic(StateAnnouncementId::MetadataAuthentication)
        }
        StateProgramWitness::OperatorSignature => {
            StateProgramComponent::Operator(StateOperatorPatternId::OperatorAuthorizationV1)
        }
    }
}

/// The relation one deployment fact is an obligation of.
///
/// Conservation and the issuance base are both about the output side:
/// with one unit consumed and output zero taking its exact amount,
/// conservation leaves none of it for any other output, and the issuance
/// that placed the whole amount under the constructor is the induction
/// base that argument runs from. The non-reissuable declaration is about
/// the input side, because what it establishes is that no unit exists
/// anywhere else to be spent. The report-layer freshness role is not a
/// fact about a deployment and the bridge does not record it.
const fn deployment_fact_relation(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    role: StateExternalEvidenceRole,
) -> Option<&RelationId> {
    match role {
        StateExternalEvidenceRole::SubstrateConservation
        | StateExternalEvidenceRole::SingletonIssuedUnderConstructor => {
            Some(plan.state().output_cardinality_relation())
        }
        StateExternalEvidenceRole::SingletonNonReissuable => {
            Some(plan.state().input_cardinality_relation())
        }
        StateExternalEvidenceRole::CurrentStateRootFreshness => None,
    }
}

/// Everything the ABI must supply, keyed like the rows.
fn abi_obligations(parts: &StateCarrierParts<'_>) -> Vec<StateAbiObligation> {
    let plan = parts.plan;
    let mut keyed: BTreeMap<(RelationId, Representation), Vec<StateAbiRequirement>> =
        BTreeMap::new();

    for (&representation, table) in parts.emitted {
        for &role in parts.witness {
            let component = state_witness_component(role);
            for (relation, carrier) in table {
                if matches!(carrier, MaturityCarrier::Emitted(named) if *named == component) {
                    let requirement = StateAbiRequirement::WitnessRole(role);
                    push_once(&mut keyed, relation, representation, requirement);
                }
            }
        }

        let state = plan.state();
        push_once(
            &mut keyed,
            state.input_recognition(),
            representation,
            StateAbiRequirement::InputSlot,
        );
        push_once(
            &mut keyed,
            state.output_recognition(),
            representation,
            StateAbiRequirement::SuccessorSlot,
        );
        push_once(
            &mut keyed,
            plan.operator().authorization(),
            representation,
            StateAbiRequirement::OperatorWitness,
        );

        for &role in parts.deployment.deployment_facts() {
            if let Some(relation) = deployment_fact_relation(plan, role) {
                let requirement = StateAbiRequirement::DeploymentFact(role);
                push_once(&mut keyed, relation, representation, requirement);
            }
        }

        if let Some(projection) = plan.projection(representation) {
            for requirement in projection.relations() {
                for open in &requirement.external_evidence {
                    let carried = StateAbiRequirement::ExternalRequirement(open.clone());
                    push_once(&mut keyed, &requirement.relation, representation, carried);
                }
            }
        }

        push_once(
            &mut keyed,
            plan.sponsor().isolation(),
            representation,
            StateAbiRequirement::SponsorRegion,
        );
    }

    flatten_obligations(keyed)
}

/// Record one obligation under its key, never twice.
fn push_once(
    keyed: &mut BTreeMap<(RelationId, Representation), Vec<StateAbiRequirement>>,
    relation: &RelationId,
    representation: Representation,
    requirement: StateAbiRequirement,
) {
    let slot = keyed.entry((relation.clone(), representation)).or_default();
    if !slot.contains(&requirement) {
        slot.push(requirement);
    }
}

/// The keyed obligations in derivation order: key order outside, and the
/// order each key's requirements were derived in within it.
fn flatten_obligations(
    keyed: BTreeMap<(RelationId, Representation), Vec<StateAbiRequirement>>,
) -> Vec<StateAbiObligation> {
    let mut obligations = Vec::new();
    for ((relation, representation), requirements) in keyed {
        for requirement in requirements {
            obligations.push(StateAbiObligation {
                relation: relation.clone(),
                representation,
                requirement,
            });
        }
    }
    obligations
}
