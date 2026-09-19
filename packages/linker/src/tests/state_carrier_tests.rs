//! The maturity link's carrier closure, over the real announcement.
//!
//! Every positive test below closes the plan, the record and the linked
//! leaf the rest of this suite uses, so the figures it pins are the
//! announcement's own rather than a fixture's. The refusals are reached
//! through the parts entry where a counterfeit record would otherwise be
//! needed, and through the public one wherever a second real deployment
//! can reach them, which is the honest demonstration of each.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use architecture::{ObjectId, OperationId};
use compiler::maturity_announcement_plan::ValidatedMaturityAnnouncementOperationPlan;
use realization::{RelationId, RelationKind, RelationSubject, TransactionSide};
use tapscript::upstream::{
    CarrierRole, DischargeBoundary, ExecutionCaseId, ExternalEvidenceRequirement,
    MaturityAnnouncementRepresentationPlan as Representation, RelationActivity,
};
use tapscript::{
    MaturityCarrier, MaturityCarrierRefusal, MaturityCarrierRefusalReason, StackItem,
    StateAnnouncementId, StateExternalEvidenceRole, StateLeafRole, StateOperatorPatternId,
    StatePatternId, StateProgramComponent, StateProgramWitness, StateStaticLeaf, StateStaticNode,
    StateStaticSubtree, TapscriptInstruction, TapscriptProgram, project_maturity_carriers,
};
use target_elements::LeafVersion;

use crate::state_carrier::{StateCarrierParts, close_state_carrier_parts, state_witness_component};
use crate::tests::state_relocate_tests::{second_bridge, second_resolved_census};
use crate::tests::{
    bridge, compact_ash_plan, linked_leaf, linked_taptree, record, reviewed_target,
};
use crate::{
    StateAbiRequirement, StateBindingTime, StateCarrierClosure, StateCarrierRow,
    StateDischargeClass, StateDischargeSide, StateLinkDeploymentParameters, StateLinkRefusal,
    StateLinkedCarrier, close_state_carriers, state_required_evidence, substitute_state,
};

// --- Fixtures ----------------------------------------------------------

/// The demonstration closure's own inputs, owned so a test can move one
/// of them and ask what the comparison says.
struct Fixture {
    plan: ValidatedMaturityAnnouncementOperationPlan,
    emitted: BTreeMap<Representation, BTreeMap<RelationId, MaturityCarrier>>,
    witness: Vec<StateProgramWitness>,
    ranges: BTreeMap<StateProgramComponent, Range<usize>>,
    pristine: TapscriptProgram,
    deployment: StateLinkDeploymentParameters,
    leaf: StateLeafRole,
    linked: TapscriptProgram,
    sites: BTreeSet<usize>,
    committed: StateStaticSubtree,
}

impl Fixture {
    fn new() -> Self {
        let record = record();
        let deployment = bridge();
        let leaf = linked_leaf();
        let taptree = linked_taptree(&leaf);
        let plan = deployment.plan().clone();
        let projection =
            project_maturity_carriers(&plan, &record).expect("the record projects its carriers");
        let emitted = plan
            .representations()
            .map(|representation| {
                let mode = representation.plan();
                let table = projection
                    .projection(mode)
                    .expect("every admitted mode is projected");
                (mode, table.clone())
            })
            .collect();

        Self {
            plan,
            emitted,
            witness: record.witness().iter().map(|(role, _)| *role).collect(),
            ranges: record.components().clone(),
            pristine: record.program().clone(),
            deployment,
            leaf: leaf.leaf(),
            linked: leaf.program().clone(),
            sites: leaf.relocations().sites(),
            committed: taptree.subtree().clone(),
        }
    }

    fn parts(&self) -> StateCarrierParts<'_> {
        StateCarrierParts {
            plan: &self.plan,
            emitted: &self.emitted,
            witness: &self.witness,
            ranges: &self.ranges,
            pristine: &self.pristine,
            deployment: &self.deployment,
            leaf: self.leaf,
            linked: &self.linked,
            sites: &self.sites,
            committed: &self.committed,
        }
    }

    fn close(&self) -> Result<StateCarrierClosure, StateLinkRefusal> {
        close_state_carrier_parts(&self.parts())
    }

    fn refuse(&self) -> StateLinkRefusal {
        self.close().expect_err("the moved fixture is refused")
    }

    /// Replace one relation's carrier in every representation.
    fn set(&mut self, relation: &RelationId, carrier: &MaturityCarrier) {
        for table in self.emitted.values_mut() {
            table.insert(relation.clone(), carrier.clone());
        }
    }

    /// Replace one relation's carrier in one representation only.
    fn set_one(&mut self, mode: Representation, relation: &RelationId, carrier: MaturityCarrier) {
        self.emitted
            .get_mut(&mode)
            .expect("the mode is projected")
            .insert(relation.clone(), carrier);
    }
}

/// The demonstration closure over the real plan, record and linked leaf.
fn closure() -> StateCarrierClosure {
    Fixture::new().close().expect("the demonstration closes")
}

/// A static subtree committing exactly one program at one role.
fn committing(program: &TapscriptProgram, role: StateLeafRole) -> StateStaticSubtree {
    StateStaticSubtree::new(
        &reviewed_target(),
        Some(StateStaticNode::Leaf {
            identity: 0,
            leaf: StateStaticLeaf {
                role,
                program: program.clone(),
                version: LeafVersion::TAPSCRIPT.get(),
            },
        }),
    )
    .expect("one leaf is a complete static subtree")
}

/// The one relation whose carrier is exactly this one.
fn relation_carrying(closed: &StateCarrierClosure, wanted: &MaturityCarrier) -> RelationId {
    let named: BTreeSet<&RelationId> = closed
        .rows()
        .iter()
        .filter(|row| row.emitted() == wanted)
        .map(StateCarrierRow::relation)
        .collect();
    named
        .into_iter()
        .next()
        .expect("the table carries this carrier")
        .clone()
}

/// A relation the plan marks vacuous in at least one case.
fn vacuous_relation(closed: &StateCarrierClosure) -> RelationId {
    closed
        .rows()
        .iter()
        .find(|row| row.activity() == RelationActivity::Vacuous)
        .expect("the sponsorless case leaves sponsor relations vacuous")
        .relation()
        .clone()
}

/// A relation that is active in every case and carried by no leaf.
fn unconditional_model_scope(closed: &StateCarrierClosure) -> RelationId {
    let vacuous: BTreeSet<&RelationId> = closed
        .rows()
        .iter()
        .filter(|row| row.activity() == RelationActivity::Vacuous)
        .map(StateCarrierRow::relation)
        .collect();
    closed
        .rows()
        .iter()
        .find(|row| {
            row.class() == StateDischargeClass::ModelScope && !vacuous.contains(row.relation())
        })
        .expect("the transaction-wide policies are active in every case")
        .relation()
        .clone()
}

/// One announcement relation identity, built the way the declaration
/// builds it.
fn announcement(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::AnnounceMaturity, kind, subject)
}

/// Every relation the realization evaluates over the whole observed
/// transaction while no leaf enforces it.
fn model_scope_relations() -> BTreeSet<RelationId> {
    let mut rows = BTreeSet::new();
    for side in [TransactionSide::Input, TransactionSide::Output] {
        for kind in [RelationKind::Cardinality, RelationKind::Recognition] {
            rows.insert(announcement(
                kind,
                RelationSubject::ObjectFamily {
                    side,
                    object: ObjectId::PlainLbtc,
                },
            ));
        }
        rows.insert(announcement(
            RelationKind::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
        ));
    }
    for kind in [
        RelationKind::SponsorIsolation,
        RelationKind::SponsorEnvelopeMultiplicity,
    ] {
        rows.insert(announcement(kind, RelationSubject::Sponsor));
    }
    for kind in [
        RelationKind::OpenFlowPolicy,
        RelationKind::CanonicalDeltaPolicy,
    ] {
        rows.insert(announcement(kind, RelationSubject::Operation));
    }
    rows
}

/// The relations the staged region-scoping refit re-scopes or retires.
///
/// Five relations, six identities: the family closure is declared once
/// per transaction side.
fn refit_relations() -> BTreeSet<RelationId> {
    let mut rows = BTreeSet::new();
    for side in [TransactionSide::Input, TransactionSide::Output] {
        rows.insert(announcement(
            RelationKind::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
        ));
    }
    for kind in [
        RelationKind::SponsorIsolation,
        RelationKind::SponsorEnvelopeMultiplicity,
    ] {
        rows.insert(announcement(kind, RelationSubject::Sponsor));
    }
    for kind in [
        RelationKind::OpenFlowPolicy,
        RelationKind::CanonicalDeltaPolicy,
    ] {
        rows.insert(announcement(kind, RelationSubject::Operation));
    }
    rows
}

/// Whether one abstract carrier role is the announcement leaf, derived
/// here from the architecture's own identifiers rather than from the
/// module under test.
fn announcement_leaf(role: &CarrierRole) -> bool {
    match role {
        CarrierRole::OperationGlobal { operation, anchor } => {
            *operation == OperationId::AnnounceMaturity && *anchor == ObjectId::State
        }
        CarrierRole::InputFamilyCoordinator { object }
        | CarrierRole::EveryInputFamilyMember { object } => *object == ObjectId::State,
        CarrierRole::BackendStructural { .. } | CarrierRole::ExternalEvidence { .. } => false,
    }
}

// --- (a) The real closure ----------------------------------------------

#[test]
fn the_closure_has_one_row_per_relation_representation_and_case() {
    let closed = closure();
    // Twenty-six relations, two representations, two execution cases:
    // the sponsorless case leaves the four sponsor-family relations
    // vacuous and the sponsored case activates them.
    assert_eq!(closed.rows().len(), 104);
    let active = closed
        .rows()
        .iter()
        .filter(|row| row.activity() == RelationActivity::Active)
        .count();
    assert_eq!(active, 96);
    assert_eq!(closed.rows().len() - active, 8);

    let relations: BTreeSet<&RelationId> = closed
        .rows()
        .iter()
        .map(StateCarrierRow::relation)
        .collect();
    assert_eq!(relations.len(), 26);
    let cases: BTreeSet<&ExecutionCaseId> =
        closed.rows().iter().map(StateCarrierRow::case).collect();
    assert_eq!(cases.len(), 4);

    assert_eq!(
        closed.representations().collect::<Vec<_>>(),
        [Representation::Explicit, Representation::PublicCommitted]
    );
    assert!(closed.is_identical_across_representations());
}

#[test]
fn every_row_is_reachable_by_its_complete_key() {
    let closed = closure();
    for row in closed.rows() {
        let found = closed
            .row(row.relation(), row.representation(), row.case())
            .expect("a row is found under its own key");
        assert_eq!(found, row);
    }
}

#[test]
fn the_class_census_is_the_records_own_discharge_table() {
    let closed = closure();
    let expected = BTreeMap::from([
        (StateDischargeClass::Emitted, 13),
        (StateDischargeClass::Deployment, 1),
        (StateDischargeClass::ModelScope, 10),
        (StateDischargeClass::External, 2),
    ]);
    assert_eq!(closed.census(), &expected);
    assert_eq!(closed.census().values().sum::<usize>(), 26);

    // The same figures read from the record's own projection, so the
    // closure is checked against the table rather than restating it.
    let projection = project_maturity_carriers(bridge().plan(), &record())
        .expect("the record projects its carriers");
    for mode in [Representation::Explicit, Representation::PublicCommitted] {
        let table = projection.projection(mode).expect("the mode is projected");
        let mut counted: BTreeMap<StateDischargeClass, usize> = BTreeMap::new();
        for carrier in table.values() {
            *counted.entry(StateDischargeClass::of(carrier)).or_insert(0) += 1;
        }
        assert_eq!(counted, expected);
    }
}

#[test]
fn every_emitted_row_sits_at_its_range_in_the_committed_linked_leaf() {
    let closed = closure();
    let leaf = linked_leaf();
    let ranges = record().components().clone();
    let mut located = 0;

    for row in closed.rows() {
        let StateLinkedCarrier::Component {
            component,
            leaf: role,
            range,
        } = row.linked()
        else {
            continue;
        };
        assert_eq!(row.class(), StateDischargeClass::Emitted);
        assert_eq!(*role, StateLeafRole::Announcement);
        assert_eq!(row.emitted(), &MaturityCarrier::Emitted(*component));
        assert_eq!(range, &ranges[component]);
        assert!(range.end <= leaf.program().len());
        located += 1;
    }

    // Thirteen emitted relations, in both cases of both representations.
    assert_eq!(located, 52);

    // The tree the closure was handed commits exactly those bytes.
    let committed = linked_taptree(&leaf);
    let entry = committed
        .subtree()
        .leaves()
        .iter()
        .find(|entry| entry.leaf.role == leaf.leaf())
        .expect("the committed subtree carries the announcement leaf");
    assert_eq!(&entry.leaf.program, leaf.program());
}

#[test]
fn the_model_scope_rows_are_the_ten_no_leaf_carries() {
    let closed = closure();
    let named: BTreeSet<RelationId> = closed
        .rows()
        .iter()
        .filter(|row| row.class() == StateDischargeClass::ModelScope)
        .map(|row| row.relation().clone())
        .collect();

    assert_eq!(named, model_scope_relations());
    assert_eq!(named.len(), 10);
    // The staged refit re-scopes or retires five of them, declared over
    // six identities; the other four lost their carrier with the
    // fragment that had claimed every position.
    let refit = refit_relations();
    assert_eq!(refit.len(), 6);
    assert!(refit.is_subset(&named));
    assert_eq!(named.difference(&refit).count(), 4);
}

// --- (b) Vacuity --------------------------------------------------------

#[test]
fn a_vacuous_case_carries_nothing_and_claims_nothing() {
    let closed = closure();
    let mut vacuous = 0;

    for row in closed.rows() {
        if row.activity() != RelationActivity::Vacuous {
            continue;
        }
        assert_eq!(row.linked(), &StateLinkedCarrier::Vacuous);
        assert_eq!(row.class(), StateDischargeClass::ModelScope);
        assert_eq!(row.selected(), None);
        assert_eq!(
            row.external_requirements(),
            &BTreeSet::<ExternalEvidenceRequirement>::new()
        );
        vacuous += 1;
    }

    // Four sponsor-family relations, in the sponsorless case of each
    // representation.
    assert_eq!(vacuous, 8);
    let relations: BTreeSet<RelationId> = closed
        .rows()
        .iter()
        .filter(|row| row.activity() == RelationActivity::Vacuous)
        .map(|row| row.relation().clone())
        .collect();
    assert_eq!(relations.len(), 4);
    assert!(relations.is_subset(&model_scope_relations()));
}

// --- (c) Both directions, and the leaf --------------------------------

#[test]
fn a_relation_the_record_answers_for_nowhere_is_refused() {
    let mut fixture = Fixture::new();
    let relation = relation_carrying(
        &closure(),
        &MaturityCarrier::Emitted(StateProgramComponent::Semantic(
            StateAnnouncementId::CopyThrough,
        )),
    );
    for table in fixture.emitted.values_mut() {
        table.remove(&relation);
    }

    assert_eq!(
        fixture.refuse(),
        StateLinkRefusal::MissingEmittedRow {
            relation,
            representation: Representation::Explicit,
        }
    );
}

#[test]
fn a_carrier_for_a_relation_the_plan_does_not_carry_is_refused() {
    let mut fixture = Fixture::new();
    // A real relation of another operation's plan: the emitted table is
    // keyed by relation identity, so a foreign key is exactly what an
    // extra row looks like.
    let foreign = compact_ash_plan()
        .relations()
        .next()
        .expect("the compact plan carries relations")
        .relation
        .clone();
    fixture.set(&foreign, &MaturityCarrier::ModelScope);

    assert_eq!(
        fixture.refuse(),
        StateLinkRefusal::ExtraEmittedRow {
            relation: foreign,
            representation: Representation::Explicit,
        }
    );
}

#[test]
fn a_class_the_boundaries_do_not_admit_is_refused() {
    let mut fixture = Fixture::new();
    // A lifecycle exit is discharged compiler-statically and
    // structurally, and the component that retains it is what makes that
    // true; nothing at all is a different claim.
    let relation = relation_carrying(
        &closure(),
        &MaturityCarrier::Emitted(StateProgramComponent::Semantic(
            StateAnnouncementId::MaturityPredecessor,
        )),
    );
    fixture.set(&relation, &MaturityCarrier::ModelScope);

    let refusal = fixture.refuse();
    let StateLinkRefusal::DischargeBoundaryDisagreement {
        relation: named,
        boundaries,
        emitted,
        ..
    } = refusal
    else {
        panic!("the boundaries admit no such class: {refusal:?}");
    };
    assert_eq!(named, relation);
    assert_eq!(emitted, StateDischargeClass::ModelScope);
    assert_eq!(
        boundaries,
        BTreeSet::from([
            DischargeBoundary::CompilerStatic,
            DischargeBoundary::BackendStructural
        ])
    );
}

#[test]
fn an_open_premise_that_disappears_is_refused() {
    let mut fixture = Fixture::new();
    let operator = fixture.plan.operator().evidence().clone();
    // The substrate's conservation premise, replaced by the operator's:
    // the class and the boundary still agree, and what is gone is the
    // premise the relation actually left open.
    let closed = closure();
    let relation = closed
        .rows()
        .iter()
        .find(|row| {
            row.class() == StateDischargeClass::External
                && row.emitted() != &MaturityCarrier::External(operator.clone())
        })
        .expect("one external row is not the operator's")
        .relation()
        .clone();
    let dropped = closed
        .rows()
        .iter()
        .find(|row| row.relation() == &relation)
        .expect("the relation has rows")
        .external_requirements()
        .clone();
    fixture.set(&relation, &MaturityCarrier::External(operator));

    let refusal = fixture.refuse();
    let StateLinkRefusal::ExternalRequirementDropped {
        relation: named,
        requirement,
        ..
    } = refusal
    else {
        panic!("the premise is gone from every side: {refusal:?}");
    };
    assert_eq!(named, relation);
    assert!(dropped.contains(&requirement));
}

#[test]
fn a_component_claiming_a_vacuous_case_is_refused() {
    let mut fixture = Fixture::new();
    let relation = vacuous_relation(&closure());
    fixture.set(
        &relation,
        &MaturityCarrier::Emitted(StateProgramComponent::Structural(
            StatePatternId::StateCoordinatorRoleV1,
        )),
    );

    let refusal = fixture.refuse();
    let StateLinkRefusal::VacuityDisagreement {
        relation: named,
        case,
        ..
    } = refusal
    else {
        panic!("a vacuous case carries nothing: {refusal:?}");
    };
    assert_eq!(named, relation);
    assert!(
        fixture
            .plan
            .projection(Representation::Explicit)
            .expect("the mode is projected")
            .case(&case)
            .is_some()
    );
}

#[test]
fn a_deployment_fact_the_bridge_did_not_record_is_refused() {
    let mut fixture = Fixture::new();
    let relation = relation_carrying(
        &closure(),
        &MaturityCarrier::Deployment(StateExternalEvidenceRole::SubstrateConservation),
    );
    // Chain-context freshness is a report-layer fact rather than one the
    // bridge records against a deployment, so nothing recorded it.
    fixture.set(
        &relation,
        &MaturityCarrier::Deployment(StateExternalEvidenceRole::CurrentStateRootFreshness),
    );

    assert_eq!(
        fixture.refuse(),
        StateLinkRefusal::DeploymentFactUnrecorded {
            relation,
            representation: Representation::Explicit,
            role: StateExternalEvidenceRole::CurrentStateRootFreshness,
        }
    );
}

#[test]
fn a_component_with_no_range_is_refused() {
    let mut fixture = Fixture::new();
    let component = StateProgramComponent::Structural(StatePatternId::StateCoordinatorRoleV1);
    let relation = relation_carrying(&closure(), &MaturityCarrier::Emitted(component));
    fixture.ranges.remove(&component);

    assert_eq!(
        fixture.refuse(),
        StateLinkRefusal::MissingComponentRange {
            relation,
            representation: Representation::Explicit,
            component,
        }
    );
}

#[test]
fn a_range_past_the_end_of_the_linked_program_is_refused() {
    let mut fixture = Fixture::new();
    let head = fixture
        .linked
        .instructions()
        .first()
        .expect("the linked program is not empty")
        .clone();
    fixture.linked = TapscriptProgram::new(vec![head]).expect("one instruction is a program");

    let refusal = fixture.refuse();
    let StateLinkRefusal::ComponentRangeOutsideLeaf { range, length, .. } = refusal else {
        panic!("a range outside the program is refused: {refusal:?}");
    };
    assert_eq!(length, 1);
    assert!(range.end > length);
}

#[test]
fn an_instruction_that_moved_outside_a_relocation_is_refused() {
    let mut fixture = Fixture::new();
    let component = StateProgramComponent::Structural(StatePatternId::StateCoordinatorRoleV1);
    let relation = relation_carrying(&closure(), &MaturityCarrier::Emitted(component));
    let (start, end) = {
        let range = &fixture.ranges[&component];
        (range.start, range.end)
    };
    let site = (start..end)
        .find(|index| !fixture.sites.contains(index))
        .expect("the coordinator range holds an index the link does not substitute at");

    let target = reviewed_target();
    let replacement = TapscriptInstruction::Push(
        StackItem::script_number(&target, 7).expect("seven is a script number"),
    );
    let mut instructions = fixture.linked.instructions().to_vec();
    assert_ne!(instructions[site], replacement);
    instructions[site] = replacement;
    fixture.linked = TapscriptProgram::new(instructions).expect("the moved program is admitted");
    // The tree is bound over the moved program too, so what the
    // comparison finds is the instruction and not the commitment.
    fixture.committed = committing(&fixture.linked, fixture.leaf);

    assert_eq!(
        fixture.refuse(),
        StateLinkRefusal::ComponentRangeMoved {
            relation,
            representation: Representation::Explicit,
            component,
            site,
        }
    );
}

#[test]
fn a_tree_committing_another_program_leaves_the_carrier_leaf_uncommitted() {
    // A second real deployment's leaf, closed against the first
    // deployment's committed tree: the tree publishes bytes nobody would
    // spend, which is the whole content of the refusal.
    let second = substitute_state(&reviewed_target(), &record(), &second_resolved_census())
        .expect("the second deployment links");
    let refusal = close_state_carriers(
        &record(),
        &second_bridge(),
        &second,
        &linked_taptree(&linked_leaf()),
    )
    .expect_err("a tree over another program does not commit this leaf");

    let StateLinkRefusal::CarrierLeafUncommitted { leaf, .. } = refusal else {
        panic!("the committed tree is not this leaf's: {refusal:?}");
    };
    assert_eq!(leaf, StateLeafRole::Announcement);
}

#[test]
fn the_second_deployment_closes_against_its_own_committed_tree() {
    let second = substitute_state(&reviewed_target(), &record(), &second_resolved_census())
        .expect("the second deployment links");
    let closed = close_state_carriers(
        &record(),
        &second_bridge(),
        &second,
        &linked_taptree(&second),
    )
    .expect("the second deployment closes against its own tree");

    assert_eq!(closed.rows().len(), 104);
    assert_eq!(closed.census(), closure().census());
    assert!(closed.is_identical_across_representations());
    assert_eq!(closed.obligations(), closure().obligations());
}

#[test]
fn one_relation_with_two_classes_is_refused() {
    let mut fixture = Fixture::new();
    let relation = unconditional_model_scope(&closure());
    // A class the boundaries still admit, so what the comparison finds
    // is the census and not the boundary.
    fixture.set_one(
        Representation::PublicCommitted,
        &relation,
        MaturityCarrier::Deployment(StateExternalEvidenceRole::SubstrateConservation),
    );

    assert_eq!(
        fixture.refuse(),
        StateLinkRefusal::CensusNotTotal {
            relation,
            first: StateDischargeClass::ModelScope,
            second: StateDischargeClass::Deployment,
        }
    );
}

#[test]
fn representations_that_read_differently_are_refused() {
    let mut fixture = Fixture::new();
    let relation = relation_carrying(
        &closure(),
        &MaturityCarrier::Emitted(StateProgramComponent::Semantic(
            StateAnnouncementId::MetadataAuthentication,
        )),
    );
    // The same class in both, so the census agrees and the difference is
    // the record's own answer for the relation.
    fixture.set_one(
        Representation::PublicCommitted,
        &relation,
        MaturityCarrier::Emitted(StateProgramComponent::Semantic(
            StateAnnouncementId::CopyThrough,
        )),
    );

    let refusal = fixture.refuse();
    let StateLinkRefusal::RepresentationDisagreement {
        relation: named,
        representation,
        side,
        ..
    } = refusal
    else {
        panic!("the representations read differently: {refusal:?}");
    };
    assert_eq!(named, relation);
    assert_eq!(representation, Representation::PublicCommitted);
    assert_eq!(side, StateDischargeSide::Emitted);
}

// --- (d) Every open premise stays visible -------------------------------

#[test]
fn every_open_premise_of_the_plan_is_visible_on_some_row() {
    let closed = closure();
    let plan = bridge().plan().clone();
    let mut open: BTreeSet<ExternalEvidenceRequirement> = BTreeSet::new();

    for projection in plan.representations() {
        for requirement in projection.relations() {
            open.extend(requirement.external_evidence.iter().cloned());
        }
    }
    // The substrate's conservation of the plain substrate, and the
    // operator's authorization of this operation.
    assert_eq!(open.len(), 2);

    for requirement in &open {
        let carried = closed.rows().iter().any(|row| {
            row.emitted() == &MaturityCarrier::External(requirement.clone())
                || row.linked() == &StateLinkedCarrier::ExternalRequirement(requirement.clone())
        });
        let outstanding = closed.obligations().iter().any(|obligation| {
            obligation.requirement()
                == &StateAbiRequirement::ExternalRequirement(requirement.clone())
        });
        assert!(carried && outstanding);
    }

    let on_rows: BTreeSet<ExternalEvidenceRequirement> = closed
        .rows()
        .iter()
        .flat_map(|row| row.external_requirements().iter().cloned())
        .collect();
    assert_eq!(on_rows, open);
}

// --- (e) The outstanding contract ---------------------------------------

#[test]
fn the_outstanding_contract_is_derived_keyed_and_equal_across_representations() {
    let closed = closure();
    let obligations = closed.obligations();
    let none: &[crate::StateAbiObligation] = &[];
    assert_ne!(obligations, none);
    assert_eq!(obligations.len(), 38);

    let relations: BTreeSet<&RelationId> = closed
        .rows()
        .iter()
        .map(StateCarrierRow::relation)
        .collect();
    for obligation in obligations {
        assert!(relations.contains(obligation.relation()));
    }

    // No element equals a later one, which is the property a set would
    // have given up the record's own witness order to obtain.
    for (index, obligation) in obligations.iter().enumerate() {
        assert!(!obligations[index + 1..].contains(obligation));
    }

    let mut by_mode: BTreeMap<Representation, Vec<(&RelationId, &StateAbiRequirement)>> =
        BTreeMap::new();
    for obligation in obligations {
        by_mode
            .entry(obligation.representation())
            .or_default()
            .push((obligation.relation(), obligation.requirement()));
    }
    assert_eq!(by_mode.len(), 2);
    let explicit = &by_mode[&Representation::Explicit];
    assert_eq!(explicit.len(), 19);
    assert_eq!(explicit, &by_mode[&Representation::PublicCommitted]);
}

#[test]
fn the_contract_names_every_witness_role_and_every_recorded_fact() {
    let closed = closure();
    // A vector rather than a set, because the requirement vocabulary
    // embeds the record's own witness roles and those carry no ordering.
    let carried: Vec<&StateAbiRequirement> = closed
        .obligations()
        .iter()
        .map(crate::StateAbiObligation::requirement)
        .collect();

    for (role, _) in record().witness() {
        assert!(carried.contains(&&StateAbiRequirement::WitnessRole(*role)));
    }
    for role in bridge().deployment_facts() {
        assert!(carried.contains(&&StateAbiRequirement::DeploymentFact(*role)));
    }
    for requirement in [
        StateAbiRequirement::InputSlot,
        StateAbiRequirement::SuccessorSlot,
        StateAbiRequirement::OperatorWitness,
        StateAbiRequirement::SponsorRegion,
    ] {
        assert!(carried.contains(&&requirement));
    }
}

#[test]
fn the_witness_attribution_agrees_with_the_authenticated_graphs_evidence_table() {
    // Pairs rather than a map, because the record's witness roles carry
    // no ordering and the table is walked once either way.
    let mut named: Vec<(StateProgramWitness, BTreeSet<StateProgramComponent>)> = Vec::new();
    for binding in [
        StateBindingTime::LinkTimeConstant,
        StateBindingTime::ConstructorPolicy,
        StateBindingTime::SpendTimeIntrospection,
        StateBindingTime::WitnessedThenAuthenticated,
        StateBindingTime::InProgramReconstruction,
    ] {
        let Some((roles, components)) = state_required_evidence(binding) else {
            continue;
        };
        for role in roles {
            let carried = components
                .iter()
                .map(|id| StateProgramComponent::Semantic(*id));
            match named.iter_mut().find(|(named, _)| named == role) {
                Some((_, slot)) => slot.extend(carried),
                None => named.push((*role, carried.collect())),
            }
        }
    }

    assert_eq!(named.len(), 5);
    for (role, components) in &named {
        assert!(components.contains(&state_witness_component(*role)));
    }

    // Neither of the other two is a cut's evidence, so neither appears in
    // that table and each is attributed here on its own terms.
    let absent = |role: StateProgramWitness| named.iter().all(|(named, _)| *named != role);
    assert!(absent(StateProgramWitness::RequestedCycle));
    assert!(absent(StateProgramWitness::OperatorSignature));
    assert_eq!(
        state_witness_component(StateProgramWitness::RequestedCycle),
        StateProgramComponent::Semantic(StateAnnouncementId::LeadWindow)
    );
    assert_eq!(
        state_witness_component(StateProgramWitness::OperatorSignature),
        StateProgramComponent::Operator(StateOperatorPatternId::OperatorAuthorizationV1)
    );
}

// --- The premises the unreachable refusals rest on ----------------------

#[test]
fn every_relation_carries_every_declared_case() {
    let plan = bridge().plan().clone();
    for projection in plan.representations() {
        let declared: BTreeSet<&ExecutionCaseId> =
            projection.cases().map(|case| &case.id).collect();
        assert_eq!(declared.len(), 2);
        assert_eq!(projection.relations().count(), 26);
        for requirement in projection.relations() {
            assert_eq!(requirement.cases.len(), 2);
            for case in &declared {
                assert!(requirement.cases.contains_key(case));
            }
        }
    }
}

#[test]
fn every_component_the_table_names_has_a_range_in_the_record() {
    let fixture = Fixture::new();
    let mut named = 0;
    for table in fixture.emitted.values() {
        for carrier in table.values() {
            if let MaturityCarrier::Emitted(component) = carrier {
                assert!(fixture.ranges.contains_key(component));
                named += 1;
            }
        }
    }
    assert_eq!(named, 26);
}

#[test]
fn every_carrier_obligation_offers_the_announcement_leaf() {
    let plan = bridge().plan().clone();
    let mut offered = 0;
    for projection in plan.representations() {
        for requirement in projection.carriers() {
            assert!(requirement.alternatives.iter().any(|alternative| {
                !alternative.carriers.is_empty()
                    && alternative
                        .carriers
                        .iter()
                        .all(|placed| announcement_leaf(&placed.carrier))
            }));
            offered += 1;
        }
    }
    assert_eq!(offered, 56);
}

#[test]
fn the_selected_alternative_is_the_leafs_where_the_plan_raises_one() {
    let closed = closure();
    let mut selected = 0;
    for row in closed.rows() {
        let Some(alternative) = row.selected() else {
            continue;
        };
        assert_eq!(row.class(), StateDischargeClass::Emitted);
        assert_eq!(row.activity(), RelationActivity::Active);
        assert!(
            row.boundaries()
                .contains(&DischargeBoundary::RuntimeCarrier)
        );
        assert!(
            alternative
                .carriers
                .iter()
                .all(|placed| announcement_leaf(&placed.carrier))
        );
        selected += 1;
    }
    // Five emitted relations are discharged at the runtime boundary, in
    // both cases of both representations.
    assert_eq!(selected, 20);
}

// --- (f) The closed root, for the carrier half --------------------------

/// Which test reaches one carrier refusal, or why nothing can.
///
/// Called from the census bite's walk over the whole closed root, which
/// names these variants one by one, so a variant added to the root still
/// fails to compile there until somebody accounts for it here.
pub(super) fn carrier_reachability(refusal: &StateLinkRefusal) -> &'static str {
    match refusal {
        StateLinkRefusal::MissingEmittedRow { .. } => {
            "a_relation_the_record_answers_for_nowhere_is_refused"
        }
        StateLinkRefusal::ExtraEmittedRow { .. } => {
            "a_carrier_for_a_relation_the_plan_does_not_carry_is_refused"
        }
        StateLinkRefusal::DischargeBoundaryDisagreement { .. } => {
            "a_class_the_boundaries_do_not_admit_is_refused"
        }
        StateLinkRefusal::ExternalRequirementDropped { .. } => {
            "an_open_premise_that_disappears_is_refused"
        }
        StateLinkRefusal::VacuityDisagreement { .. } => {
            "a_component_claiming_a_vacuous_case_is_refused"
        }
        StateLinkRefusal::DeploymentFactUnrecorded { .. } => {
            "a_deployment_fact_the_bridge_did_not_record_is_refused"
        }
        StateLinkRefusal::MissingComponentRange { .. } => "a_component_with_no_range_is_refused",
        StateLinkRefusal::ComponentRangeOutsideLeaf { .. } => {
            "a_range_past_the_end_of_the_linked_program_is_refused"
        }
        StateLinkRefusal::ComponentRangeMoved { .. } => {
            "an_instruction_that_moved_outside_a_relocation_is_refused"
        }
        StateLinkRefusal::CarrierLeafUncommitted { .. } => {
            "a_tree_committing_another_program_leaves_the_carrier_leaf_uncommitted"
        }
        StateLinkRefusal::CensusNotTotal { .. } => "one_relation_with_two_classes_is_refused",
        StateLinkRefusal::RepresentationDisagreement { .. } => {
            "representations_that_read_differently_are_refused"
        }
        StateLinkRefusal::EmittedProjection(_) => {
            "unreachable: the only admitted record is one exact recipe equality produced, so every \
             component its table names has a range in it"
        }
        StateLinkRefusal::MissingCompilerRelation { .. } => {
            "unreachable: the plan's own census crosses every relation with every declared case, \
             and the plan is constructible only through the entry that validates it"
        }
        StateLinkRefusal::DuplicateCarrierRow { .. } => {
            "unreachable: both censuses are maps keyed by relation and the case census is a map \
             keyed by case, so one key cannot reach two rows"
        }
        StateLinkRefusal::SelectedAlternativeUnmatched { .. } => {
            "unreachable: every candidate set for an active runtime relation-case of this \
             announcement includes the operation's coordinator anchored in the singleton family, \
             which is this leaf"
        }
        _ => "accounted for elsewhere in the closed root",
    }
}

/// A sample of every carrier refusal, for the walk over the root.
fn carrier_refusals() -> Vec<StateLinkRefusal> {
    let plan = bridge().plan().clone();
    let relation = plan.operator().authorization().clone();
    let case = plan
        .projection(Representation::Explicit)
        .expect("the mode is projected")
        .cases()
        .next()
        .expect("the plan declares cases")
        .id
        .clone();
    let representation = Representation::Explicit;
    let component = StateProgramComponent::Structural(StatePatternId::StateCoordinatorRoleV1);
    let requirement = plan.operator().evidence().clone();

    vec![
        StateLinkRefusal::EmittedProjection(MaturityCarrierRefusal {
            relation: relation.clone(),
            reason: MaturityCarrierRefusalReason::Unmapped,
        }),
        StateLinkRefusal::MissingCompilerRelation {
            relation: relation.clone(),
            representation,
            case: case.clone(),
        },
        StateLinkRefusal::MissingEmittedRow {
            relation: relation.clone(),
            representation,
        },
        StateLinkRefusal::ExtraEmittedRow {
            relation: relation.clone(),
            representation,
        },
        StateLinkRefusal::DuplicateCarrierRow {
            relation: relation.clone(),
            representation,
            case: case.clone(),
        },
        StateLinkRefusal::DischargeBoundaryDisagreement {
            relation: relation.clone(),
            representation,
            case: case.clone(),
            boundaries: BTreeSet::from([DischargeBoundary::RuntimeCarrier]),
            emitted: StateDischargeClass::ModelScope,
        },
        StateLinkRefusal::ExternalRequirementDropped {
            relation: relation.clone(),
            representation,
            case: case.clone(),
            requirement,
        },
        StateLinkRefusal::VacuityDisagreement {
            relation: relation.clone(),
            representation,
            case: case.clone(),
        },
        StateLinkRefusal::DeploymentFactUnrecorded {
            relation: relation.clone(),
            representation,
            role: StateExternalEvidenceRole::CurrentStateRootFreshness,
        },
        StateLinkRefusal::MissingComponentRange {
            relation: relation.clone(),
            representation,
            component,
        },
        StateLinkRefusal::ComponentRangeOutsideLeaf {
            relation: relation.clone(),
            representation,
            component,
            range: 0..1,
            length: 0,
        },
        StateLinkRefusal::CarrierLeafUncommitted {
            relation: relation.clone(),
            representation,
            leaf: StateLeafRole::Announcement,
        },
        StateLinkRefusal::ComponentRangeMoved {
            relation: relation.clone(),
            representation,
            component,
            site: 0,
        },
        StateLinkRefusal::SelectedAlternativeUnmatched {
            relation: relation.clone(),
            representation,
            case: case.clone(),
        },
        StateLinkRefusal::CensusNotTotal {
            relation: relation.clone(),
            first: StateDischargeClass::Emitted,
            second: StateDischargeClass::ModelScope,
        },
        StateLinkRefusal::RepresentationDisagreement {
            relation,
            representation,
            case,
            side: StateDischargeSide::Emitted,
        },
    ]
}

#[test]
fn every_carrier_refusal_is_reached_or_declared() {
    let refusals = carrier_refusals();
    let accounts: BTreeSet<&str> = refusals.iter().map(carrier_reachability).collect();

    assert_eq!(refusals.len(), 16);
    assert_eq!(accounts.len(), refusals.len());
    assert!(
        !accounts.contains("accounted for elsewhere in the closed root"),
        "a carrier refusal fell through to another half"
    );
    assert_eq!(
        refusals
            .iter()
            .filter(|refusal| carrier_reachability(refusal).starts_with("unreachable"))
            .count(),
        4
    );
}
