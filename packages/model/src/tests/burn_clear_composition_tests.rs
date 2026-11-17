//! Joint evidence from executed Burn, compact and Clear model transitions.

use std::collections::{BTreeMap, BTreeSet};

use super::advanced_fixtures::give_live_receipt;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;
use architecture::{OperationId, ProjectionId};
use realization::{
    AccountingValues, BoundBurnClearObservation, BurnClearContext, BurnClearObservation,
    BurnClearPayload, BurnClearRefusal, ObservedObjectRef, ObservedSide, OperationObservation,
    RecordCredit,
};

fn scope(operations: impl IntoIterator<Item = OperationId>) -> realization::ScopedRealizationSpec {
    realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations(operations).unwrap(),
    )
    .unwrap()
}

fn bind(
    common: &OperationObservation,
    extension: &BurnClearObservation,
) -> Result<BoundBurnClearObservation, BurnClearRefusal> {
    realization::validate_burn_clear_observation(common, &extension.context, Some(extension))
}

fn burn(
    world: &World,
    owner: OwnerKey,
    amount: u64,
    claim: u64,
) -> ExecutedTransition<BurnReceipts> {
    let receipt = find_receipts(world, owner, ReceiptClass::Live)[0];
    let request = BurnReceipts {
        receipts: vec![receipt],
        signers: signers(&[owner]),
        ash_value: sat(amount),
        change: Vec::new(),
        records: vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(claim),
        }],
        fee_envelope: FeeEnvelope::default(),
    };
    execute_bound(world, request, next_order(world)).unwrap()
}

fn compact(world: &World) -> ExecutedTransition<CompactAsh> {
    execute_bound(
        world,
        CompactAsh {
            ash_inputs: find_ash(world),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(world),
    )
    .unwrap()
}

fn clear(world: &World, inputs: Vec<OutPoint>) -> ExecutedTransition<ClearAsh> {
    execute_bound(
        world,
        ClearAsh {
            ash_inputs: inputs,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(world),
    )
    .unwrap()
}

fn two_burn_world() -> (
    ExecutedTransition<BurnReceipts>,
    ExecutedTransition<BurnReceipts>,
) {
    let initial = test_fixtures::world();
    let first = give_live_receipt(&initial, ALICE, sat(50));
    let funded = give_live_receipt(&first, BOB, sat(50));
    let alice = burn(&funded, ALICE, 50, 50);
    let bob = burn(alice.after(), BOB, 50, 51);
    (alice, bob)
}

fn model_credit(execution: &ExecutedTransition<BurnReceipts>) -> bool {
    let indexer = ReferenceIndexer::from_assumed_kernel_history(
        &execution.after().history,
        &test_fixtures::chain_view_for_history(execution.after()),
        [0; 32],
    )
    .unwrap();
    indexer.burns()[&execution.certificate().txid]
        .records_accepted()
        .unwrap()
}

fn assert_operation(
    common: &ModelConformanceObservation,
    extension: &BurnClearObservation,
    spec: &realization::ScopedRealizationSpec,
) -> BoundBurnClearObservation {
    let bound = bind(common.observation(), extension).unwrap();
    let report = spec.evaluate_operation_with_burn_clear(&bound).unwrap();
    assert!(report.is_conformant(), "{:?}", report.verdicts);
    assert!(unresolved_model_evidence(&report, common).is_empty());
    bound
}

#[test]
fn compact_comparison_adapter_matches_execution() {
    let (alice, bob) = two_burn_world();
    assert!(model_credit(&alice));
    assert!(!model_credit(&bob));
    let execution = compact(bob.after());
    let projected = observe_compact_ash_with_burn_clear(&execution).unwrap();
    assert_eq!(projected.common, observe_compact_ash(&execution).unwrap());
    assert_eq!(
        projected.accounting.values,
        AccountingValues::Compact {
            input: realization::ProtocolAmount::new(100).unwrap(),
            output: realization::ProtocolAmount::new(100).unwrap(),
        }
    );
    assert!(execution.certificate().burn.is_none());
    assert!(execution.certificate().clear.is_none());
    assert!(execution.certificate().distribution_residue.is_none());
    assert_eq!(projected.accounting.event, None);
    assert!(!projected.accounting.distribution_residue);
    let bound = assert_operation(
        &projected.common,
        &projected.extension,
        &scope([OperationId::CompactAsh]),
    );
    realization::validate_compact_silence(&bound, &projected.silence).unwrap();
    realization::validate_accounting(&bound, &projected.accounting).unwrap();
    let mut changed = projected.accounting.clone();
    if let AccountingValues::Compact { output, .. } = &mut changed.values {
        *output = realization::ProtocolAmount::new(99).unwrap();
    }
    assert_eq!(
        realization::validate_accounting(&bound, &changed),
        Err(BurnClearRefusal::Accounting)
    );
    realization::validate_compact_silence(&bound, &projected.silence).unwrap();
}

#[test]
fn sequential_burn_compact_clear_preserves_credit_identity() {
    let (alice, bob) = two_burn_world();
    let mut refused = alice.request().clone();
    refused.ash_value = sat(51);
    assert!(execute_bound(alice.before(), refused, next_order(alice.before())).is_err());
    let original_records = [alice.certificate(), bob.certificate()].map(|certificate| {
        (
            certificate.txid,
            certificate.burn.as_ref().unwrap().records.clone(),
        )
    });
    assert_eq!([model_credit(&alice), model_credit(&bob)], [true, false]);
    let compact_execution = compact(bob.after());
    let clear_execution = clear(
        compact_execution.after(),
        find_ash(compact_execution.after()),
    );
    assert_eq!(clear_execution.certificate().burn, None);
    assert_eq!(compact_execution.certificate().burn, None);
    assert_eq!(find_ash(clear_execution.after()), [] as [OutPoint; 0]);
    let indexer = ReferenceIndexer::from_assumed_kernel_history(
        &clear_execution.after().history,
        &test_fixtures::chain_view_for_history(clear_execution.after()),
        [0; 32],
    )
    .unwrap();
    assert_eq!(indexer.burns().len(), 2);
    for (index, (txid, records)) in original_records.into_iter().enumerate() {
        assert_eq!(indexer.burns()[&txid].records, records);
        assert_eq!(
            indexer.burns()[&txid].records_accepted().unwrap(),
            index == 0
        );
    }
    let burn_spec = scope([OperationId::Burn]);
    for (execution, credited) in [(&alice, true), (&bob, false)] {
        let projected = observe_burn(execution).unwrap();
        let bound = assert_operation(&projected.common, &projected.extension, &burn_spec);
        assert_eq!(
            matches!(
                realization::evaluate_record_credit(&bound).unwrap(),
                RecordCredit::Credited { .. }
            ),
            credited
        );
    }
    let compact_projected = observe_compact_ash_with_burn_clear(&compact_execution).unwrap();
    assert_operation(
        &compact_projected.common,
        &compact_projected.extension,
        &scope([OperationId::CompactAsh]),
    );
    let clear_projected = observe_clear(&clear_execution).unwrap();
    assert_operation(
        &clear_projected.common,
        &clear_projected.extension,
        &scope([OperationId::Clear]),
    );
}

#[test]
fn union_scope_preserves_operation_verdicts() {
    let (alice, bob) = two_burn_world();
    let compact_execution = compact(bob.after());
    let clear_execution = clear(
        compact_execution.after(),
        find_ash(compact_execution.after()),
    );
    let burn = observe_burn(&alice).unwrap();
    let compact = observe_compact_ash_with_burn_clear(&compact_execution).unwrap();
    let clear = observe_clear(&clear_execution).unwrap();
    let cases = [
        (OperationId::Burn, &burn.common, &burn.extension, 32),
        (
            OperationId::CompactAsh,
            &compact.common,
            &compact.extension,
            23,
        ),
        (OperationId::Clear, &clear.common, &clear.extension, 44),
    ];
    let combined = scope([
        OperationId::Burn,
        OperationId::CompactAsh,
        OperationId::Clear,
        OperationId::TransferLive,
    ]);
    for (operation, common, extension, relation_count) in cases {
        let singleton = scope([operation]);
        let bound = bind(common.observation(), extension).unwrap();
        let single = singleton
            .evaluate_operation_with_burn_clear(&bound)
            .unwrap();
        let mixed = combined.evaluate_operation_with_burn_clear(&bound).unwrap();
        assert_eq!(mixed, single);
        let expected_premises = BTreeSet::from([
            realization::ExternalEvidenceRequirement::SubstrateConservation {
                operation,
                asset: architecture::AssetId::Lbtc,
            },
        ]);
        assert_eq!(
            mixed
                .required_external_evidence()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected_premises
        );
        assert_eq!(
            single
                .required_external_evidence()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected_premises
        );
        assert_eq!(mixed.verdicts.len(), relation_count);
        assert_eq!(
            singleton.operation(operation).unwrap().relations.len(),
            relation_count
        );
        assert_eq!(
            combined.operation(operation).unwrap().relations.len(),
            relation_count
        );
        assert_eq!(
            combined.operation(operation),
            singleton.operation(operation)
        );
        assert!(mixed.is_conformant());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Role {
    Passed,
    Refused(BurnClearRefusal),
    Credit(bool),
    OperationFailure {
        failed: BTreeSet<realization::RelationId>,
        blocked: BTreeMap<realization::RelationId, Vec<realization::RelationId>>,
    },
    NotApplicable,
    NotReached,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Roles {
    operation: Role,
    burn_event: Role,
    record: Role,
    compact_silence: Role,
    clear_event: Role,
    accounting: Role,
    state: Role,
}

fn role(result: Result<(), BurnClearRefusal>) -> Role {
    match result {
        Ok(()) => Role::Passed,
        Err(BurnClearRefusal::ProjectionRole) => Role::NotApplicable,
        Err(refusal) => Role::Refused(refusal),
    }
}

#[derive(Clone, Copy)]
struct Offers<'a> {
    burn_event: &'a realization::BurnEventProjection,
    clear_event: &'a realization::ClearEventProjection,
    silence: &'a realization::CompactSilenceProjection,
    accounting: &'a realization::AccountingProjection,
    state: &'a realization::StateSuccessionProjection,
}

fn operation_role(report: &realization::ConformanceReport) -> Role {
    if report.is_conformant() {
        return Role::Passed;
    }
    let mut failed = BTreeSet::new();
    let mut blocked = BTreeMap::new();
    for verdict in &report.verdicts {
        match &verdict.status {
            realization::RelationStatus::Failed { .. } => {
                failed.insert(verdict.relation.clone());
            }
            realization::RelationStatus::Blocked { prerequisites } => {
                blocked.insert(verdict.relation.clone(), prerequisites.clone());
            }
            realization::RelationStatus::Passed
            | realization::RelationStatus::StaticallyValidated
            | realization::RelationStatus::EvidenceRequired { .. } => {}
        }
    }
    Role::OperationFailure { failed, blocked }
}

fn not_reached_roles(operation: OperationId) -> Roles {
    let applicable = |role| {
        if role {
            Role::NotReached
        } else {
            Role::NotApplicable
        }
    };
    Roles {
        operation: Role::NotReached,
        burn_event: applicable(operation == OperationId::Burn),
        record: applicable(operation == OperationId::Burn),
        compact_silence: applicable(operation == OperationId::CompactAsh),
        clear_event: applicable(operation == OperationId::Clear),
        accounting: Role::NotReached,
        state: applicable(operation == OperationId::Clear),
    }
}

fn matrix_row(
    operation: OperationId,
    binding: Result<BoundBurnClearObservation, BurnClearRefusal>,
    offers: Offers<'_>,
    model_credit: Option<bool>,
) -> Roles {
    let Ok(bound) = binding else {
        return not_reached_roles(operation);
    };
    let report = scope([operation])
        .evaluate_operation_with_burn_clear(&bound)
        .unwrap();
    let record = match realization::evaluate_record_credit(&bound) {
        Ok(credit) => {
            let credited = matches!(credit, RecordCredit::Credited { .. });
            assert_eq!(Some(credited), model_credit);
            Role::Credit(credited)
        }
        Err(BurnClearRefusal::ProjectionRole) => Role::NotApplicable,
        Err(refusal) => Role::Refused(refusal),
    };
    Roles {
        operation: operation_role(&report),
        burn_event: role(realization::validate_burn_event(&bound, offers.burn_event)),
        record,
        compact_silence: role(realization::validate_compact_silence(
            &bound,
            offers.silence,
        )),
        clear_event: role(realization::validate_clear_event(
            &bound,
            offers.clear_event,
        )),
        accounting: role(realization::validate_accounting(&bound, offers.accounting)),
        state: role(realization::validate_state_succession(&bound, offers.state)),
    }
}

#[test]
fn seven_projection_roles_keep_separate_results() {
    let (alice, bob) = two_burn_world();
    let burn_control = observe_burn(&alice).unwrap();
    let overclaim = observe_burn(&bob).unwrap();
    let compact_execution = compact(bob.after());
    let compact_control = observe_compact_ash_with_burn_clear(&compact_execution).unwrap();
    let clear_execution = clear(
        compact_execution.after(),
        find_ash(compact_execution.after()),
    );
    let clear_control = observe_clear(&clear_execution).unwrap();
    let burn_offers = Offers {
        burn_event: &burn_control.event,
        clear_event: &clear_control.event,
        silence: &compact_control.silence,
        accounting: &burn_control.accounting,
        state: &clear_control.state_succession,
    };
    let compact_offers = Offers {
        accounting: &compact_control.accounting,
        ..burn_offers
    };
    let clear_offers = Offers {
        accounting: &clear_control.accounting,
        ..burn_offers
    };
    assert_eq!(
        matrix_row(
            OperationId::Burn,
            bind(burn_control.common.observation(), &burn_control.extension),
            burn_offers,
            Some(model_credit(&alice)),
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::Passed,
            record: Role::Credit(true),
            compact_silence: Role::NotApplicable,
            clear_event: Role::NotApplicable,
            accounting: Role::Passed,
            state: Role::NotApplicable,
        }
    );
    assert_eq!(
        matrix_row(
            OperationId::Burn,
            bind(overclaim.common.observation(), &overclaim.extension),
            Offers {
                burn_event: &overclaim.event,
                accounting: &overclaim.accounting,
                ..burn_offers
            },
            Some(model_credit(&bob)),
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::Passed,
            record: Role::Credit(false),
            compact_silence: Role::NotApplicable,
            clear_event: Role::NotApplicable,
            accounting: Role::Passed,
            state: Role::NotApplicable,
        }
    );
    assert_eq!(
        matrix_row(
            OperationId::CompactAsh,
            bind(
                compact_control.common.observation(),
                &compact_control.extension
            ),
            compact_offers,
            None,
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::NotApplicable,
            record: Role::NotApplicable,
            compact_silence: Role::Passed,
            clear_event: Role::NotApplicable,
            accounting: Role::Passed,
            state: Role::NotApplicable,
        }
    );
    assert_eq!(
        matrix_row(
            OperationId::Clear,
            bind(clear_control.common.observation(), &clear_control.extension),
            clear_offers,
            None,
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::NotApplicable,
            record: Role::NotApplicable,
            compact_silence: Role::NotApplicable,
            clear_event: Role::Passed,
            accounting: Role::Passed,
            state: Role::Passed,
        }
    );

    let mut changed_burn_event = burn_control.event.clone();
    changed_burn_event.fresh_amount = realization::ProtocolAmount::new(49).unwrap();
    assert_eq!(
        matrix_row(
            OperationId::Burn,
            bind(burn_control.common.observation(), &burn_control.extension),
            Offers {
                burn_event: &changed_burn_event,
                ..burn_offers
            },
            Some(model_credit(&alice)),
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::Refused(BurnClearRefusal::BurnEvent),
            record: Role::Credit(true),
            compact_silence: Role::NotApplicable,
            clear_event: Role::NotApplicable,
            accounting: Role::Passed,
            state: Role::NotApplicable,
        }
    );
    let mut duplicate = burn_control.extension.clone();
    if let BurnClearPayload::Burn { records, .. } = &mut duplicate.payload {
        records.push(records[0]);
    }
    let duplicate_binding = bind(burn_control.common.observation(), &duplicate);
    assert_eq!(duplicate_binding, Err(BurnClearRefusal::RecordOrdinal));
    assert_eq!(
        matrix_row(
            OperationId::Burn,
            duplicate_binding,
            burn_offers,
            Some(model_credit(&alice))
        ),
        Roles {
            operation: Role::NotReached,
            burn_event: Role::NotReached,
            record: Role::NotReached,
            compact_silence: Role::NotApplicable,
            clear_event: Role::NotApplicable,
            accounting: Role::NotReached,
            state: Role::NotApplicable,
        }
    );

    let mut changed_clear_event = clear_control.event.clone();
    changed_clear_event.aggregate = realization::ProtocolAmount::new(99).unwrap();
    assert_eq!(
        matrix_row(
            OperationId::Clear,
            bind(clear_control.common.observation(), &clear_control.extension),
            Offers {
                clear_event: &changed_clear_event,
                ..clear_offers
            },
            None,
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::NotApplicable,
            record: Role::NotApplicable,
            compact_silence: Role::NotApplicable,
            clear_event: Role::Refused(BurnClearRefusal::ClearEvent),
            accounting: Role::Passed,
            state: Role::Passed,
        }
    );
    let mut changed_accounting = clear_control.accounting.clone();
    if let AccountingValues::Clear { aggregate, .. } = &mut changed_accounting.values {
        *aggregate = realization::ProtocolAmount::new(99).unwrap();
    }
    assert_eq!(
        matrix_row(
            OperationId::Clear,
            bind(clear_control.common.observation(), &clear_control.extension),
            Offers {
                accounting: &changed_accounting,
                ..clear_offers
            },
            None,
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::NotApplicable,
            record: Role::NotApplicable,
            compact_silence: Role::NotApplicable,
            clear_event: Role::Passed,
            accounting: Role::Refused(BurnClearRefusal::Accounting),
            state: Role::Passed,
        }
    );
    let mut changed_state = clear_control.state_succession.clone();
    changed_state.predecessor.metadata.q = realization::ProtocolAmount::new(1).unwrap();
    assert_eq!(
        matrix_row(
            OperationId::Clear,
            bind(clear_control.common.observation(), &clear_control.extension),
            Offers {
                state: &changed_state,
                ..clear_offers
            },
            None,
        ),
        Roles {
            operation: Role::Passed,
            burn_event: Role::NotApplicable,
            record: Role::NotApplicable,
            compact_silence: Role::NotApplicable,
            clear_event: Role::Passed,
            accounting: Role::Passed,
            state: Role::Refused(BurnClearRefusal::StateSuccession),
        }
    );

    let mut changed_common = compact_control.common.observation().clone();
    changed_common.projections.insert(ProjectionId::ClearEvent);
    let compact_token = compact_execution.certificate().txid.0;
    let context =
        BurnClearContext::bind_observation(changed_common.clone(), compact_token, compact_token)
            .unwrap();
    let extension = BurnClearObservation {
        context: context.clone(),
        payload: BurnClearPayload::Compact,
    };
    let silence = realization::CompactSilenceProjection {
        context: context.clone(),
    };
    let mut accounting = compact_control.accounting.clone();
    accounting.context = context;
    let bound = bind(&changed_common, &extension).unwrap();
    let report = scope([OperationId::CompactAsh])
        .evaluate_operation_with_burn_clear(&bound)
        .unwrap();
    let projection = realization::RelationId::new(
        OperationId::CompactAsh,
        realization::RelationKind::ProjectionPolicy,
        realization::RelationSubject::Operation,
    );
    let constructibility = realization::RelationId::new(
        OperationId::CompactAsh,
        realization::RelationKind::Constructibility,
        realization::RelationSubject::Operation,
    );
    let representation = realization::RelationId::new(
        OperationId::CompactAsh,
        realization::RelationKind::Representation,
        realization::RelationSubject::Representation {
            object: architecture::ObjectId::Ash,
        },
    );
    let lifecycle = |exit| {
        realization::RelationId::new(
            OperationId::CompactAsh,
            realization::RelationKind::Lifecycle,
            realization::RelationSubject::LifecycleExit {
                object: architecture::ObjectId::Ash,
                exit,
            },
        )
    };
    let blocked = BTreeMap::from([
        (constructibility.clone(), vec![projection.clone()]),
        (representation.clone(), vec![constructibility]),
        (
            lifecycle(OperationId::CompactAsh),
            vec![representation.clone()],
        ),
        (lifecycle(OperationId::Clear), vec![representation]),
    ]);
    let failed = BTreeSet::from([projection]);
    let expected_failure = Role::OperationFailure {
        failed: failed.clone(),
        blocked: blocked.clone(),
    };
    assert_eq!(operation_role(&report), expected_failure);
    assert_eq!(
        report.failed_relations().cloned().collect::<BTreeSet<_>>(),
        failed.into_iter().chain(blocked.keys().cloned()).collect(),
    );
    assert_eq!(
        matrix_row(
            OperationId::CompactAsh,
            Ok(bound),
            Offers {
                silence: &silence,
                accounting: &accounting,
                ..compact_offers
            },
            None,
        ),
        Roles {
            operation: expected_failure,
            burn_event: Role::NotApplicable,
            record: Role::NotApplicable,
            compact_silence: Role::Refused(BurnClearRefusal::CompactSilence),
            clear_event: Role::NotApplicable,
            accounting: Role::Refused(BurnClearRefusal::Accounting),
            state: Role::NotApplicable,
        }
    );
}

#[test]
fn same_base_clear_alternatives_require_common_binding() {
    let first = give_live_receipt(&test_fixtures::world(), ALICE, sat(40));
    let funded = give_live_receipt(&first, BOB, sat(60));
    let first_burn = super::advanced_fixtures::burn_from_owner(&funded, ALICE, sat(40), Vec::new());
    let world = super::advanced_fixtures::burn_from_owner(&first_burn, BOB, sat(60), Vec::new());
    let ashes = find_ash(&world);
    assert_eq!(ashes.len(), 2);
    let first = clear(&world, vec![ashes[0]]);
    let second = clear(&world, vec![ashes[1]]);
    assert_eq!(first.certificate().txid, second.certificate().txid);
    assert_eq!(
        first.certificate().state_edge,
        second.certificate().state_edge
    );
    // Unequal ASH values make the two normalized common observations distinct.
    // Equal-valued single-ASH alternatives would have identical common values.
    let first = observe_clear(&first).unwrap();
    let second = observe_clear(&second).unwrap();
    let spec = scope([OperationId::Clear]);
    assert_operation(&first.common, &first.extension, &spec);
    assert_operation(&second.common, &second.extension, &spec);
    assert_ne!(first.common.observation(), second.common.observation());
    assert_eq!(
        realization::validate_burn_clear_observation(
            first.common.observation(),
            &first.extension.context,
            Some(&second.extension),
        ),
        Err(BurnClearRefusal::CommonObservation)
    );
    assert_eq!(
        realization::validate_burn_clear_observation(
            second.common.observation(),
            &second.extension.context,
            Some(&first.extension),
        ),
        Err(BurnClearRefusal::CommonObservation)
    );
}

#[test]
fn clear_residual_does_not_become_burn_or_residue() {
    let (world, _) = super::advanced_fixtures::sealing_world();
    let supply = world.state().unwrap().1.y().unwrap();
    let burned =
        super::advanced_fixtures::burn_from_owner(&world, GENESIS_OWNER, supply, Vec::new());
    let execution = clear(&burned, find_ash(&burned));
    let projected = observe_clear(&execution).unwrap();
    let residual = find_ash(execution.after());
    assert_eq!(residual.len(), 1);
    assert_eq!(execution.after().utxo(residual[0]).unwrap().meta, Meta::Ash);
    assert!(execution.certificate().burn.is_none());
    assert!(execution.certificate().distribution_residue.is_none());
    assert!(
        !projected
            .common
            .observation()
            .projections
            .contains(&ProjectionId::BurnEvent)
    );
    assert!(
        !projected
            .common
            .observation()
            .projections
            .contains(&ProjectionId::DistributionResidue)
    );
    let bound = assert_operation(
        &projected.common,
        &projected.extension,
        &scope([OperationId::Clear]),
    );
    assert_eq!(projected.event.residual.len(), 1);
    realization::validate_clear_event(&bound, &projected.event).unwrap();
    realization::validate_state_succession(&bound, &projected.state_succession).unwrap();
    realization::validate_accounting(&bound, &projected.accounting).unwrap();
    let mut changed = projected.accounting.clone();
    changed.distribution_residue = true;
    assert_eq!(
        realization::validate_accounting(&bound, &changed),
        Err(BurnClearRefusal::Accounting)
    );
    realization::validate_clear_event(&bound, &projected.event).unwrap();
    realization::validate_state_succession(&bound, &projected.state_succession).unwrap();
}

fn constructibility_status(report: &realization::ConformanceReport) -> realization::RelationStatus {
    let id = realization::RelationId::new(
        report.operation,
        realization::RelationKind::Constructibility,
        realization::RelationSubject::Operation,
    );
    report
        .verdicts
        .iter()
        .find(|verdict| verdict.relation == id)
        .unwrap()
        .status
        .clone()
}

fn authorization_status(report: &realization::ConformanceReport) -> realization::RelationStatus {
    let id = realization::RelationId::new(
        report.operation,
        realization::RelationKind::Authorization,
        realization::RelationSubject::Operation,
    );
    report
        .verdicts
        .iter()
        .find(|verdict| verdict.relation == id)
        .unwrap()
        .status
        .clone()
}

#[test]
fn public_compact_and_clear_refuse_protocol_signers() {
    let (alice, bob) = two_burn_world();
    let compact_execution = compact(bob.after());
    let clear_execution = clear(
        compact_execution.after(),
        find_ash(compact_execution.after()),
    );
    let compact = observe_compact_ash_with_burn_clear(&compact_execution).unwrap();
    let clear = observe_clear(&clear_execution).unwrap();
    let sponsor_base = super::ash_clear_tests::world_with_two_ash();
    let (sponsor_world, sponsor_input) = fund_lbtc(&sponsor_base, SPONSOR, sat(20));
    let sponsored_compact_execution = execute_bound(
        &sponsor_world,
        CompactAsh {
            ash_inputs: find_ash(&sponsor_world),
            fee_envelope: fee_envelope_exact(sponsor_input, SPONSOR, sat(20), sat(1)),
        },
        next_order(&sponsor_world),
    )
    .unwrap();
    let sponsored_clear_execution = execute_bound(
        &sponsor_world,
        ClearAsh {
            ash_inputs: find_ash(&sponsor_world),
            fee_envelope: fee_envelope_exact(sponsor_input, SPONSOR, sat(20), sat(1)),
        },
        next_order(&sponsor_world),
    )
    .unwrap();
    let sponsored_compact =
        observe_compact_ash_with_burn_clear(&sponsored_compact_execution).unwrap();
    let sponsored_clear = observe_clear(&sponsored_clear_execution).unwrap();
    let burn = observe_burn(&alice).unwrap();
    let burn_report = scope([OperationId::Burn])
        .evaluate_operation_with_burn_clear(
            &bind(burn.common.observation(), &burn.extension).unwrap(),
        )
        .unwrap();
    assert!(burn_report.is_conformant());
    assert!(
        burn.common
            .observation()
            .protocol_signers
            .contains(&realization::OwnerId(ALICE.0))
    );
    assert!(matches!(
        constructibility_status(&burn_report),
        realization::RelationStatus::Passed
    ));
    for (operation, common, extension, token, sponsored) in [
        (
            OperationId::CompactAsh,
            &compact.common,
            &compact.extension,
            compact_execution.certificate().txid.0,
            false,
        ),
        (
            OperationId::Clear,
            &clear.common,
            &clear.extension,
            clear_execution.certificate().txid.0,
            false,
        ),
        (
            OperationId::CompactAsh,
            &sponsored_compact.common,
            &sponsored_compact.extension,
            sponsored_compact_execution.certificate().txid.0,
            true,
        ),
        (
            OperationId::Clear,
            &sponsored_clear.common,
            &sponsored_clear.extension,
            sponsored_clear_execution.certificate().txid.0,
            true,
        ),
    ] {
        let spec = scope([operation]);
        let control = assert_operation(common, extension, &spec);
        let control_report = spec.evaluate_operation_with_burn_clear(&control).unwrap();
        assert_eq!(
            authorization_status(&control_report),
            realization::RelationStatus::Passed
        );
        assert_eq!(
            constructibility_status(&control_report),
            realization::RelationStatus::Passed
        );
        assert!(common.observation().protocol_signers.is_empty());
        if sponsored {
            assert_eq!(
                common.observation().sponsor_signers,
                BTreeSet::from([realization::OwnerId(SPONSOR.0)])
            );
        } else {
            assert!(common.observation().sponsor_signers.is_empty());
        }
        let mut changed = common.observation().clone();
        changed
            .protocol_signers
            .insert(realization::OwnerId(ALICE.0));
        let context = BurnClearContext::bind_observation(changed.clone(), token, token).unwrap();
        let offered = BurnClearObservation {
            context,
            payload: extension.payload.clone(),
        };
        let bound = bind(&changed, &offered).unwrap();
        let report = spec.evaluate_operation_with_burn_clear(&bound).unwrap();
        let authorization = realization::RelationId::new(
            operation,
            realization::RelationKind::Authorization,
            realization::RelationSubject::Operation,
        );
        assert!(matches!(
            authorization_status(&report),
            realization::RelationStatus::Failed {
                reason: realization::RelationFailure::UnexpectedProtocolAuthorization
            }
        ));
        assert_eq!(
            constructibility_status(&report),
            realization::RelationStatus::Blocked {
                prerequisites: vec![authorization.clone()]
            },
        );
        assert_eq!(
            report
                .verdicts
                .iter()
                .filter_map(|verdict| matches!(
                    verdict.status,
                    realization::RelationStatus::Failed { .. }
                )
                .then_some(verdict.relation.clone()))
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([authorization]),
        );
    }
}

type RefMap = BTreeMap<ObservedObjectRef, ObservedObjectRef>;

fn mapped(reference: &mut ObservedObjectRef, map: &RefMap) {
    *reference = map[reference];
}

fn remap_region(region: &mut realization::ObservedOperationRegion, map: &RefMap) {
    region.members = region
        .members
        .iter()
        .map(|reference| map[reference])
        .collect();
    for issuance in &mut region.canonical_partition.issuances {
        mapped(&mut issuance.authority_input, map);
        for destination in &mut issuance.destinations {
            mapped(destination, map);
        }
    }
    for flow in &mut region.canonical_partition.flows {
        for reference in flow.sources.iter_mut().chain(&mut flow.destinations) {
            mapped(reference, map);
        }
    }
    for flow in &mut region.open_flows {
        for reference in flow.sources.iter_mut().chain(&mut flow.destinations) {
            mapped(reference, map);
        }
    }
}

fn remap_common(source: &OperationObservation, base: u32) -> (OperationObservation, RefMap) {
    let mut map = RefMap::new();
    for object in &source.objects {
        let old = object.reference;
        let new = ObservedObjectRef {
            side: old.side,
            ordinal: old.ordinal.checked_add(base).unwrap(),
        };
        assert_eq!(map.insert(old, new), None);
    }
    assert_eq!(map.values().collect::<BTreeSet<_>>().len(), map.len());
    let mut result = source.clone();
    for object in &mut result.objects {
        mapped(&mut object.reference, &map);
    }
    remap_region(&mut result.region, &map);
    for region in &mut result.other_regions {
        remap_region(region, &map);
    }
    assert_eq!(result.protocol_signers, source.protocol_signers);
    assert_eq!(result.sponsor_signers, source.sponsor_signers);
    assert_eq!(result.bounds, source.bounds);
    assert_eq!(result.root_effects, source.root_effects);
    assert_eq!(result.projections, source.projections);
    for (original, changed) in source.objects.iter().zip(&result.objects) {
        assert_eq!(original.reference.side, changed.reference.side);
        assert_eq!(original.kind, changed.kind);
        assert_eq!(original.asset, changed.asset);
        assert_eq!(original.value, changed.value);
        assert_eq!(original.owner, changed.owner);
    }
    (result, map)
}

fn remap_extension(extension: &mut BurnClearObservation, map: &RefMap) {
    match &mut extension.payload {
        BurnClearPayload::Burn { fresh_ash, records } => {
            mapped(fresh_ash, map);
            for record in records {
                mapped(&mut record.fresh_ash, map);
            }
        }
        BurnClearPayload::Clear {
            predecessor,
            successor,
            ash_sources,
            ..
        } => {
            mapped(&mut predecessor.reference, map);
            mapped(&mut successor.reference, map);
            for source in ash_sources {
                mapped(source, map);
            }
        }
        BurnClearPayload::Compact => {}
    }
}

struct Catalog {
    commons: Vec<OperationObservation>,
    maps: Vec<RefMap>,
}

fn catalog(sources: &[OperationObservation]) -> Catalog {
    let mapped = sources
        .iter()
        .enumerate()
        .map(|(index, source)| remap_common(source, u32::try_from(index + 1).unwrap() * 100))
        .collect::<Vec<_>>();
    let all_objects = mapped
        .iter()
        .flat_map(|(common, _)| common.objects.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        all_objects
            .iter()
            .map(|object| object.reference)
            .collect::<BTreeSet<_>>()
            .len(),
        all_objects.len()
    );
    let regions = mapped
        .iter()
        .map(|(common, _)| common.region.clone())
        .collect::<Vec<_>>();
    let commons = mapped
        .iter()
        .enumerate()
        .map(|(index, (source, _))| {
            let mut common = source.clone();
            common.objects = all_objects.clone();
            common.other_regions = regions
                .iter()
                .enumerate()
                .filter_map(|(other, region)| (other != index).then_some(region.clone()))
                .collect();
            common.validate_and_normalize().unwrap()
        })
        .collect();
    Catalog {
        commons,
        maps: mapped.into_iter().map(|(_, map)| map).collect(),
    }
}

fn rebind(
    source: &BurnClearObservation,
    common: &OperationObservation,
    map: &RefMap,
    txid: TxId,
) -> BurnClearObservation {
    let mut result = source.clone();
    remap_extension(&mut result, map);
    result.context = BurnClearContext::bind_observation(common.clone(), txid.0, txid.0).unwrap();
    result
}

fn remap_burn_event(
    source: &realization::BurnEventProjection,
    extension: &BurnClearObservation,
    map: &RefMap,
) -> realization::BurnEventProjection {
    let mut event = source.clone();
    event.context = extension.context.clone();
    mapped(&mut event.fresh_ash, map);
    for record in &mut event.records {
        mapped(&mut record.fresh_ash, map);
    }
    event
}

fn remap_clear_event(
    source: &realization::ClearEventProjection,
    extension: &BurnClearObservation,
    map: &RefMap,
) -> realization::ClearEventProjection {
    let mut event = source.clone();
    event.context = extension.context.clone();
    mapped(&mut event.predecessor.reference, map);
    mapped(&mut event.successor.reference, map);
    for reference in &mut event.ash_sources {
        mapped(reference, map);
    }
    for (reference, _) in &mut event.residual {
        mapped(reference, map);
    }
    event
}

#[test]
fn disjoint_regions_preserve_sponsored_controls() {
    let initial = give_live_receipt(&test_fixtures::world(), ALICE, sat(50));
    let (sponsored_burn, burn_sponsor) = fund_lbtc(&initial, SPONSOR, sat(20));
    let alice = execute_bound(
        &sponsored_burn,
        BurnReceipts {
            receipts: vec![find_receipts(&sponsored_burn, ALICE, ReceiptClass::Live)[0]],
            signers: signers(&[ALICE]),
            ash_value: sat(50),
            change: Vec::new(),
            records: vec![BurnRecord {
                record_index: 0,
                address: ADDRESS_A,
                amount: sat(50),
            }],
            fee_envelope: fee_envelope_exact(burn_sponsor, SPONSOR, sat(20), sat(1)),
        },
        next_order(&sponsored_burn),
    )
    .unwrap();
    let (first, bob) = two_burn_world();
    assert!(model_credit(&first));
    let (sponsored, sponsor_input) = fund_lbtc(bob.after(), SPONSOR, sat(20));
    let compact_execution = execute_bound(
        &sponsored,
        CompactAsh {
            ash_inputs: find_ash(&sponsored),
            fee_envelope: fee_envelope_exact(sponsor_input, SPONSOR, sat(20), sat(1)),
        },
        next_order(&sponsored),
    )
    .unwrap();
    let clear_base = super::ash_clear_tests::world_with_two_ash();
    let (sponsored_clear, clear_sponsor) = fund_lbtc(&clear_base, SPONSOR, sat(20));
    let clear_execution = execute_bound(
        &sponsored_clear,
        ClearAsh {
            ash_inputs: find_ash(&sponsored_clear),
            fee_envelope: fee_envelope_exact(clear_sponsor, SPONSOR, sat(20), sat(1)),
        },
        next_order(&sponsored_clear),
    )
    .unwrap();
    let burn = observe_burn(&alice).unwrap();
    let compact = observe_compact_ash_with_burn_clear(&compact_execution).unwrap();
    let clear = observe_clear(&clear_execution).unwrap();
    for common in [&burn.common, &compact.common, &clear.common] {
        assert!(
            common
                .observation()
                .objects
                .iter()
                .any(|object| object.value == realization::ObservedValue::SponsorOpaque)
        );
    }
    let sources = [
        burn.common.observation().clone(),
        compact.common.observation().clone(),
        clear.common.observation().clone(),
    ];
    let catalog = catalog(&sources);
    let extensions = [
        rebind(
            &burn.extension,
            &catalog.commons[0],
            &catalog.maps[0],
            alice.certificate().txid,
        ),
        rebind(
            &compact.extension,
            &catalog.commons[1],
            &catalog.maps[1],
            compact_execution.certificate().txid,
        ),
        rebind(
            &clear.extension,
            &catalog.commons[2],
            &catalog.maps[2],
            clear_execution.certificate().txid,
        ),
    ];
    let original_extensions = [&burn.extension, &compact.extension, &clear.extension];
    let joint = scope([
        OperationId::Burn,
        OperationId::CompactAsh,
        OperationId::Clear,
        OperationId::TransferLive,
    ]);
    for (index, extension) in extensions.iter().enumerate() {
        let bound = bind(&catalog.commons[index], extension).unwrap();
        let report = joint.evaluate_operation_with_burn_clear(&bound).unwrap();
        assert!(report.is_conformant(), "{:?}", report.verdicts);
        let reverse = catalog.maps[index]
            .iter()
            .map(|(old, new)| (*new, *old))
            .collect::<RefMap>();
        let mut original = catalog.commons[index].clone();
        original
            .objects
            .retain(|object| reverse.contains_key(&object.reference));
        original.other_regions.clear();
        for object in &mut original.objects {
            mapped(&mut object.reference, &reverse);
        }
        remap_region(&mut original.region, &reverse);
        assert_eq!(original, sources[index]);
        let mut recovered = extension.clone();
        remap_extension(&mut recovered, &reverse);
        recovered.context = original_extensions[index].context.clone();
        assert_eq!(recovered, *original_extensions[index]);
        assert_eq!(
            catalog.commons[index]
                .region
                .canonical_partition
                .flows
                .len(),
            sources[index].region.canonical_partition.flows.len()
        );
    }
    let burn_bound = bind(&catalog.commons[0], &extensions[0]).unwrap();
    let burn_event = remap_burn_event(&burn.event, &extensions[0], &catalog.maps[0]);
    realization::validate_burn_event(&burn_bound, &burn_event).unwrap();
    assert_eq!(
        remap_burn_event(
            &burn_event,
            &burn.extension,
            &catalog.maps[0]
                .iter()
                .map(|(old, new)| (*new, *old))
                .collect()
        ),
        burn.event
    );
    let mut burn_accounting = burn.accounting.clone();
    burn_accounting.context = extensions[0].context.clone();
    realization::validate_accounting(&burn_bound, &burn_accounting).unwrap();
    burn_accounting.context = burn.accounting.context.clone();
    assert_eq!(burn_accounting, burn.accounting);
    let compact_bound = bind(&catalog.commons[1], &extensions[1]).unwrap();
    let mut accounting = compact.accounting.clone();
    accounting.context = extensions[1].context.clone();
    realization::validate_accounting(&compact_bound, &accounting).unwrap();
    accounting.context = compact.accounting.context.clone();
    assert_eq!(accounting, compact.accounting);
    let silence = realization::CompactSilenceProjection {
        context: extensions[1].context.clone(),
    };
    realization::validate_compact_silence(&compact_bound, &silence).unwrap();
    let mut recovered_silence = silence;
    recovered_silence.context = compact.silence.context.clone();
    assert_eq!(recovered_silence, compact.silence);
    let clear_bound = bind(&catalog.commons[2], &extensions[2]).unwrap();
    let clear_event = remap_clear_event(&clear.event, &extensions[2], &catalog.maps[2]);
    realization::validate_clear_event(&clear_bound, &clear_event).unwrap();
    assert_eq!(
        remap_clear_event(
            &clear_event,
            &clear.extension,
            &catalog.maps[2]
                .iter()
                .map(|(old, new)| (*new, *old))
                .collect()
        ),
        clear.event
    );
    let mut clear_state = clear.state_succession.clone();
    clear_state.context = extensions[2].context.clone();
    mapped(&mut clear_state.predecessor.reference, &catalog.maps[2]);
    mapped(&mut clear_state.successor.reference, &catalog.maps[2]);
    realization::validate_state_succession(&clear_bound, &clear_state).unwrap();
    clear_state.context = clear.state_succession.context.clone();
    let reverse_state = catalog.maps[2]
        .iter()
        .map(|(old, new)| (*new, *old))
        .collect();
    mapped(&mut clear_state.predecessor.reference, &reverse_state);
    mapped(&mut clear_state.successor.reference, &reverse_state);
    assert_eq!(clear_state, clear.state_succession);
    let mut clear_accounting = clear.accounting.clone();
    clear_accounting.context = extensions[2].context.clone();
    realization::validate_accounting(&clear_bound, &clear_accounting).unwrap();
    clear_accounting.context = clear.accounting.context.clone();
    assert_eq!(clear_accounting, clear.accounting);
    assert!(
        catalog.commons[1]
            .objects
            .iter()
            .any(|object| object.value == realization::ObservedValue::SponsorOpaque)
    );
}

#[test]
fn substituted_region_and_foreign_payload_refuse() {
    let (alice, bob) = two_burn_world();
    let compact_execution = compact(bob.after());
    let clear_execution = clear(
        compact_execution.after(),
        find_ash(compact_execution.after()),
    );
    let other_clear_world = super::ash_clear_tests::world_with_two_ash();
    let other_clear_execution = clear(&other_clear_world, vec![find_ash(&other_clear_world)[0]]);
    let burn = observe_burn(&alice).unwrap();
    let compact = observe_compact_ash_with_burn_clear(&compact_execution).unwrap();
    let clear = observe_clear(&clear_execution).unwrap();
    let other_clear = observe_clear(&other_clear_execution).unwrap();
    let catalog = catalog(&[
        burn.common.observation().clone(),
        compact.common.observation().clone(),
        clear.common.observation().clone(),
        other_clear.common.observation().clone(),
    ]);
    let burn_extension = rebind(
        &burn.extension,
        &catalog.commons[0],
        &catalog.maps[0],
        alice.certificate().txid,
    );
    let clear_extension = rebind(
        &clear.extension,
        &catalog.commons[2],
        &catalog.maps[2],
        clear_execution.certificate().txid,
    );
    bind(&catalog.commons[0], &burn_extension).unwrap();
    bind(&catalog.commons[2], &clear_extension).unwrap();
    let mut switched = catalog.commons[0].clone();
    std::mem::swap(&mut switched.region, &mut switched.other_regions[0]);
    switched.clone().validate_and_normalize().unwrap();
    assert_eq!(
        realization::validate_burn_clear_observation(
            &switched,
            &burn_extension.context,
            Some(&burn_extension)
        ),
        Err(BurnClearRefusal::CommonObservation)
    );
    let mut foreign_ash = burn_extension;
    if let BurnClearPayload::Burn { fresh_ash, records } = &mut foreign_ash.payload {
        let foreign_ash_reference = compact
            .common
            .observation()
            .objects
            .iter()
            .find(|object| {
                object.reference.side == ObservedSide::Output
                    && object.kind
                        == realization::ObservedObjectKind::Declared(architecture::ObjectId::Ash)
            })
            .unwrap()
            .reference;
        *fresh_ash = catalog.maps[1][&foreign_ash_reference];
        for record in records {
            record.fresh_ash = *fresh_ash;
        }
    }
    assert_eq!(
        bind(&catalog.commons[0], &foreign_ash),
        Err(BurnClearRefusal::AshBinding)
    );
    let mut foreign_state = clear_extension;
    if let BurnClearPayload::Clear { predecessor, .. } = &mut foreign_state.payload {
        let original = other_clear.state_succession.predecessor.reference;
        predecessor.reference = catalog.maps[3][&original];
    }
    assert_eq!(
        bind(&catalog.commons[2], &foreign_state),
        Err(BurnClearRefusal::StateBinding)
    );
}

#[test]
fn overlapping_region_members_refuse_normalization() {
    let (alice, bob) = two_burn_world();
    let compact_execution = compact(bob.after());
    let burn = observe_burn(&alice).unwrap();
    let compact = observe_compact_ash_with_burn_clear(&compact_execution).unwrap();
    let catalog = catalog(&[
        burn.common.observation().clone(),
        compact.common.observation().clone(),
    ]);
    let control = catalog.commons[0].clone();
    control.clone().validate_and_normalize().unwrap();
    let foreign = *control.other_regions[0].members.iter().next().unwrap();
    let mut subject = control;
    subject.region.members.insert(foreign);
    assert_eq!(
        subject.validate_and_normalize(),
        Err(realization::RealizationError::ObservedRegionOverlap(
            foreign
        ))
    );
}
