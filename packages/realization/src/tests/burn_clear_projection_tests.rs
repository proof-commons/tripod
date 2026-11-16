use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId, RootId, TagId,
};

use crate::*;

type Mutation = (BurnClearRefusal, fn(&mut BurnClearObservation));

fn literal_clear_accounting(common: &OperationObservation) -> AccountingProjection {
    AccountingProjection {
        context: context(common),
        values: AccountingValues::Clear {
            aggregate: amount(10),
            clear_amount: amount(6),
            state_decrement: amount(6),
            destruction: Destruction {
                asset: AssetId::U,
                amount: amount(6),
                tag: TagId::Recon,
                flow: Some(0),
            },
            residual: amount(4),
        },
        event: Some(ProjectionId::ClearEvent),
        distribution_residue: false,
    }
}

#[test]
fn foreign_payload_references_refuse_in_a_well_formed_snapshot() {
    let (mut common, mut extension) = burn(&[10]);
    let foreign = object(ObservedSide::Output, 7, ObjectId::Ash, 10);
    common.other_regions.push(ObservedOperationRegion {
        members: BTreeSet::from([foreign.reference]),
        ..Default::default()
    });
    common.objects.push(foreign);
    extension.context = context(&common);
    bound(&common, &extension);
    if let BurnClearPayload::Burn { fresh_ash, records } = &mut extension.payload {
        *fresh_ash = reference(ObservedSide::Output, 7);
        records[0].fresh_ash = reference(ObservedSide::Output, 7);
    }
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(BurnClearRefusal::AshBinding)
    );
    let (mut common, mut extension) = clear();
    let foreign = object(ObservedSide::Input, 9, ObjectId::State, 1);
    common.other_regions.push(ObservedOperationRegion {
        members: BTreeSet::from([foreign.reference]),
        ..Default::default()
    });
    common.objects.push(foreign);
    extension.context = context(&common);
    bound(&common, &extension);
    if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
        predecessor.reference = reference(ObservedSide::Input, 9);
    }
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(BurnClearRefusal::StateBinding)
    );
}

#[test]
fn state_succession_reaches_effect_and_assignment_checks() {
    for assignment in [false, true] {
        let (mut common, mut extension) = clear();
        let mut edge = StateSuccessionProjection {
            context: context(&common),
            predecessor: literal_state(ObservedSide::Input, 6),
            successor: literal_state(ObservedSide::Output, 0),
        };
        validate_state_succession(&bound(&common, &extension), &edge).unwrap();
        if assignment {
            if let BurnClearPayload::Clear { successor, .. } = &mut extension.payload {
                successor.metadata.q = amount(3);
            }
            edge.successor.metadata.q = amount(3);
            let mut event = literal_clear_event(&common);
            event.successor.metadata.q = amount(3);
            let checked = bound(&common, &extension);
            validate_clear_event(&checked, &event).unwrap();
            validate_accounting(&checked, &literal_clear_accounting(&common)).unwrap();
        } else {
            common.root_effects[0].effect = ObservedRootEffectKind::Termination;
            extension.context = context(&common);
            edge.context = context(&common);
        }
        assert_eq!(
            validate_state_succession(&bound(&common, &extension), &edge),
            Err(BurnClearRefusal::StateSuccession)
        );
    }
}

#[test]
fn accounting_refuses_independent_totals_residue_and_actual_decrement() {
    let (common, extension) = burn(&[10]);
    let checked = bound(&common, &extension);
    let accounting = AccountingProjection {
        context: context(&common),
        values: AccountingValues::Burn {
            record_total: 10,
            fresh_amount: amount(10),
        },
        event: Some(ProjectionId::BurnEvent),
        distribution_residue: false,
    };
    validate_accounting(&checked, &accounting).unwrap();
    for residue in [false, true] {
        let mut subject = accounting.clone();
        if residue {
            subject.distribution_residue = true;
        } else if let AccountingValues::Burn { record_total, .. } = &mut subject.values {
            *record_total = 11;
        }
        validate_burn_event(&checked, &literal_burn_event(&common)).unwrap();
        assert_eq!(
            validate_accounting(&checked, &subject),
            Err(BurnClearRefusal::Accounting)
        );
    }
    let (common, mut extension) = clear();
    let accounting = literal_clear_accounting(&common);
    validate_accounting(&bound(&common, &extension), &accounting).unwrap();
    if let BurnClearPayload::Clear { successor, .. } = &mut extension.payload {
        successor.metadata.y_l = amount(1);
    }
    assert_eq!(
        validate_accounting(&bound(&common, &extension), &accounting),
        Err(BurnClearRefusal::Accounting)
    );
}

#[test]
fn event_carried_fields_are_compared_with_independent_literals() {
    let (common, extension) = burn(&[10]);
    let checked = bound(&common, &extension);
    let mut event = literal_burn_event(&common);
    validate_burn_event(&checked, &event).unwrap();
    event.records[0].address = BurnAddress([8; 32]);
    assert_eq!(
        validate_burn_event(&checked, &event),
        Err(BurnClearRefusal::BurnEvent)
    );
    let (common, extension) = clear();
    let checked = bound(&common, &extension);
    let mut event = literal_clear_event(&common);
    validate_clear_event(&checked, &event).unwrap();
    event.ash_sources.push(reference(ObservedSide::Input, 9));
    assert_eq!(
        validate_clear_event(&checked, &event),
        Err(BurnClearRefusal::ClearEvent)
    );
    assert_eq!(
        checked.resolve_state_fact(&FactId::StateField {
            operation: OperationId::Clear,
            side: TransactionSide::Input,
            field: StateField::YL
        }),
        Some(SemanticValue::Amount(amount(6)))
    );
    assert_eq!(
        checked.resolve_state_fact(&FactId::StateField {
            operation: OperationId::Clear,
            side: TransactionSide::Output,
            field: StateField::YL
        }),
        Some(SemanticValue::Amount(ProtocolAmount::ZERO))
    );
}

#[test]
fn compact_refuses_value_loss_state_and_destruction() {
    for subject in 0..3 {
        let mut common = common(
            OperationId::CompactAsh,
            vec![
                object(ObservedSide::Input, 0, ObjectId::Ash, 10),
                object(ObservedSide::Output, 0, ObjectId::Ash, 10),
            ],
        );
        let mut extension = BurnClearObservation {
            context: context(&common),
            payload: BurnClearPayload::Compact,
        };
        let accounting = AccountingProjection {
            context: context(&common),
            values: AccountingValues::Compact {
                input: amount(10),
                output: amount(10),
            },
            event: None,
            distribution_residue: false,
        };
        let checked = bound(&common, &extension);
        validate_accounting(&checked, &accounting).unwrap();
        validate_compact_silence(
            &checked,
            &CompactSilenceProjection {
                context: context(&common),
            },
        )
        .unwrap();
        match subject {
            0 => common.objects[1].value = ObservedValue::Protocol(amount(9)),
            1 => common.claim_object(object(ObservedSide::Output, 1, ObjectId::State, 1)),
            _ => common
                .region
                .canonical_partition
                .flows
                .push(ObservedCanonicalFlow {
                    asset: AssetId::U,
                    sources: vec![],
                    destinations: vec![],
                    movement_kind: None,
                    destructions: vec![ObservedDestructionLeg {
                        tag: TagId::Recon,
                        amount: amount(1),
                    }],
                }),
        }
        extension.context = context(&common);
        let checked = bound(&common, &extension);
        if subject == 0 {
            let offered = AccountingProjection {
                context: context(&common),
                values: AccountingValues::Compact {
                    input: amount(10),
                    output: amount(9),
                },
                event: None,
                distribution_residue: false,
            };
            assert_eq!(
                validate_accounting(&checked, &offered),
                Err(BurnClearRefusal::Accounting)
            );
        } else {
            assert_eq!(
                validate_compact_silence(
                    &checked,
                    &CompactSilenceProjection {
                        context: context(&common)
                    }
                ),
                Err(BurnClearRefusal::CompactSilence)
            );
        }
    }
}

#[test]
fn clear_event_pins_both_offered_x_and_actual_destruction() {
    let (mut common, mut extension) = clear();
    let checked = bound(&common, &extension);
    let mut event = literal_clear_event(&common);
    validate_clear_event(&checked, &event).unwrap();
    event.clear_amount = amount(5);
    assert_eq!(
        validate_clear_event(&checked, &event),
        Err(BurnClearRefusal::ClearEvent)
    );
    common.region.canonical_partition.flows[0].destructions[0].amount = amount(5);
    if let BurnClearPayload::Clear { destruction, .. } = &mut extension.payload {
        destruction.amount = amount(5);
    }
    extension.context = context(&common);
    let mut event = literal_clear_event(&common);
    event.destruction.amount = amount(5);
    assert_eq!(
        validate_clear_event(&bound(&common, &extension), &event),
        Err(BurnClearRefusal::ClearEvent)
    );
}

#[test]
fn bound_observation_tracing_emits_one_outcome_and_preserves_results() {
    use super::tracing_tests::{assert_no_warning_or_error, capture, in_capture_child};
    if !in_capture_child(
        "tests::burn_clear_projection_tests::bound_observation_tracing_emits_one_outcome_and_preserves_results",
    ) {
        return;
    }
    let (common, extension) = burn(&[4, 6]);
    let expected = context(&common);
    for (subject, outcome) in [(Some(&extension), "completed"), (None, "refused")] {
        let plain = validate_burn_clear_observation(&common, &expected, subject);
        let (captured, records) =
            capture(|| validate_burn_clear_observation(&common, &expected, subject));
        assert_eq!(plain, captured);
        assert_no_warning_or_error(&records);
        assert_eq!(records.len(), 2);
        let span = &records[0];
        assert_eq!(span.name, "validate_burn_clear_observation");
        assert_eq!(span.target, "realization::burn_clear_observation");
        assert_eq!(span.level, tracing::Level::DEBUG);
        assert_eq!(
            span.fields["operation_code"],
            OperationId::Burn.code().to_string()
        );
        assert_eq!(span.types.get("operation_code"), Some(&"u64"));
        assert_eq!(span.fields.len(), 1);
        let event = &records[1];
        assert_eq!(event.target, "realization::burn_clear_observation");
        assert_eq!(event.level, tracing::Level::DEBUG);
        assert_eq!(event.fields["message"], "burn clear observation validated");
        assert_eq!(event.fields["outcome"], outcome);
        assert_eq!(event.types.get("outcome"), Some(&"str"));
        if outcome == "refused" {
            assert_eq!(event.fields["refusal_kind"], "missing-extension");
            assert_eq!(event.types.get("refusal_kind"), Some(&"str"));
            assert_eq!(event.fields.len(), 3);
        } else {
            assert_eq!(event.fields.len(), 2);
        }
    }
}

#[test]
fn burn_clear_refusal_names_are_distinct_and_follow_their_grammar() {
    let names: BTreeSet<_> = BurnClearRefusal::ALL
        .iter()
        .map(|refusal| refusal.name())
        .collect();
    let expected = BTreeSet::from([
        "missing-extension",
        "malformed-observation",
        "common-observation",
        "transaction",
        "certificate",
        "operation",
        "ash-binding",
        "state-binding",
        "record-bound",
        "record-ordinal",
        "record-structure",
        "data-association",
        "destruction-binding",
        "burn-event",
        "clear-event",
        "accounting",
        "state-succession",
        "compact-silence",
        "arithmetic",
        "clear-transition",
        "projection-role",
    ]);
    assert_eq!(names, expected);
    assert_eq!(names.len(), BurnClearRefusal::ALL.len());
    assert!(
        names
            .iter()
            .all(|name| name.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
    );
    for refusal in BurnClearRefusal::ALL {
        assert_eq!(refusal.to_string(), refusal.name());
    }
}

fn literal_burn_event(common: &OperationObservation) -> BurnEventProjection {
    BurnEventProjection {
        context: context(common),
        fresh_ash: reference(ObservedSide::Output, 0),
        fresh_amount: amount(10),
        records: vec![BurnRecord {
            ordinal: 0,
            address: BurnAddress([9; 32]),
            amount: amount(10),
            tag: TagId::Burn,
            fresh_ash: reference(ObservedSide::Output, 0),
        }],
    }
}

fn literal_state(side: ObservedSide, y_l: u64) -> ObservedState {
    ObservedState {
        reference: reference(side, 0),
        metadata: StateMetadata {
            omega: amount(20),
            y_l: amount(y_l),
            y_t: amount(3),
            q: amount(2),
            cycle: Cycle::new(7),
            maturity: Maturity::Unannounced,
        },
    }
}

fn literal_clear_event(common: &OperationObservation) -> ClearEventProjection {
    ClearEventProjection {
        context: context(common),
        predecessor: literal_state(ObservedSide::Input, 6),
        successor: literal_state(ObservedSide::Output, 0),
        ash_sources: vec![reference(ObservedSide::Input, 1)],
        aggregate: amount(10),
        clear_amount: amount(6),
        destruction: Destruction {
            asset: AssetId::U,
            amount: amount(6),
            tag: TagId::Recon,
            flow: Some(0),
        },
        residual: vec![(reference(ObservedSide::Output, 1), amount(4))],
        omega: amount(20),
        y: amount(3),
    }
}

#[test]
fn review_clear_transition_refusals_are_not_arithmetic() {
    for subject in 0..3 {
        let (mut common, mut extension) = clear();
        let mut edge = StateSuccessionProjection {
            context: context(&common),
            predecessor: literal_state(ObservedSide::Input, 6),
            successor: literal_state(ObservedSide::Output, 0),
        };
        validate_state_succession(&bound(&common, &extension), &edge).unwrap();
        match subject {
            0 => {
                if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
                    predecessor.metadata.y_l = amount(0);
                }
                edge.predecessor.metadata.y_l = amount(0);
            }
            1 => {
                common.bounds.insert(BoundId::AshBatchMax, Count::ZERO);
            }
            _ => {
                common.bounds.remove(&BoundId::AshBatchMax);
            }
        }
        extension.context = context(&common);
        edge.context = context(&common);
        let refusal = validate_state_succession(&bound(&common, &extension), &edge).unwrap_err();
        assert_eq!(refusal.name(), "clear-transition");
    }
}

#[test]
fn sealed_and_supply_domain_projection_refusals_stay_distinct() {
    let (mut common, mut extension) = clear();
    // Independent control: Y=6, B=10, hence X=5, successor Y=1 and R=5.
    common.objects[3].value = ObservedValue::Protocol(amount(5));
    common.region.canonical_partition.flows[0].destructions[0].amount = amount(5);
    if let BurnClearPayload::Clear {
        predecessor,
        successor,
        destruction,
        ..
    } = &mut extension.payload
    {
        predecessor.metadata.y_t = amount(0);
        successor.metadata.y_l = amount(1);
        successor.metadata.y_t = amount(0);
        destruction.amount = amount(5);
    }
    extension.context = context(&common);
    let mut edge = StateSuccessionProjection {
        context: context(&common),
        predecessor: ObservedState {
            metadata: StateMetadata {
                y_t: amount(0),
                ..literal_state(ObservedSide::Input, 6).metadata
            },
            ..literal_state(ObservedSide::Input, 6)
        },
        successor: ObservedState {
            metadata: StateMetadata {
                y_t: amount(0),
                ..literal_state(ObservedSide::Output, 1).metadata
            },
            ..literal_state(ObservedSide::Output, 1)
        },
    };
    validate_state_succession(&bound(&common, &extension), &edge).unwrap();
    if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
        predecessor.metadata.y_l = amount(0);
    }
    edge.predecessor.metadata.y_l = amount(0);
    assert_eq!(
        validate_state_succession(&bound(&common, &extension), &edge),
        Err(BurnClearRefusal::ClearTransition)
    );

    let (common, mut extension) = clear();
    let mut edge = StateSuccessionProjection {
        context: context(&common),
        predecessor: literal_state(ObservedSide::Input, 6),
        successor: literal_state(ObservedSide::Output, 0),
    };
    validate_state_succession(&bound(&common, &extension), &edge).unwrap();
    if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
        predecessor.metadata.y_l = amount(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1);
    }
    edge.predecessor.metadata.y_l = amount(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1);
    assert_eq!(
        validate_state_succession(&bound(&common, &extension), &edge),
        Err(BurnClearRefusal::Arithmetic)
    );
}

#[test]
fn review_clear_binding_rejects_second_leg_and_issuance() {
    for issuance in [false, true] {
        let (mut common, mut extension) = clear();
        bound(&common, &extension);
        if issuance {
            common
                .region
                .canonical_partition
                .issuances
                .push(ObservedIssuance {
                    asset: AssetId::U,
                    authority: AssetId::Pid,
                    authority_input: reference(ObservedSide::Input, 0),
                    amount: amount(1),
                    destinations: vec![],
                });
        } else {
            common
                .region
                .canonical_partition
                .flows
                .push(ObservedCanonicalFlow {
                    asset: AssetId::U,
                    sources: vec![],
                    destinations: vec![],
                    movement_kind: None,
                    destructions: vec![ObservedDestructionLeg {
                        tag: TagId::Recon,
                        amount: amount(1),
                    }],
                });
            // Preserve the association when normalization moves the existing flow.
            if let BurnClearPayload::Clear { destruction, .. } = &mut extension.payload {
                destruction.flow = Some(1);
            }
        }
        extension.context = context(&common);
        assert_eq!(
            validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
            Err(BurnClearRefusal::DestructionBinding)
        );
    }
}

#[test]
fn review_burn_event_requires_live_inputs_and_live_change() {
    for input in [true, false] {
        let (mut common, mut extension) = burn(&[10]);
        if !input {
            common.claim_object(object(ObservedSide::Output, 1, ObjectId::ReceiptLive, 1));
        }
        extension.context = context(&common);
        validate_burn_event(&bound(&common, &extension), &literal_burn_event(&common)).unwrap();
        if input {
            common.objects[0].kind = ObservedObjectKind::Declared(ObjectId::Ash);
        } else {
            common.objects[2].kind = ObservedObjectKind::Unrecognized;
        }
        extension.context = context(&common);
        assert_eq!(
            validate_burn_event(&bound(&common, &extension), &literal_burn_event(&common)),
            Err(BurnClearRefusal::BurnEvent)
        );
    }
}

#[test]
fn review_clear_event_refuses_foreign_roots() {
    let (mut common, mut extension) = clear();
    validate_clear_event(&bound(&common, &extension), &literal_clear_event(&common)).unwrap();
    common.root_effects.push(ObservedRootEffect {
        root: RootId::Resv,
        effect: ObservedRootEffectKind::Succession,
    });
    extension.context = context(&common);
    assert_eq!(
        validate_clear_event(&bound(&common, &extension), &literal_clear_event(&common)),
        Err(BurnClearRefusal::ClearEvent)
    );
}

#[test]
fn address_bytes_do_not_change_structural_admission() {
    for bytes in [
        [0; 32],
        [255; 32],
        std::array::from_fn(|i| u8::try_from(i).unwrap()),
    ] {
        let (common, mut extension) = burn(&[10]);
        if let BurnClearPayload::Burn { records, .. } = &mut extension.payload {
            records[0].address = BurnAddress(bytes);
        }
        let checked = bound(&common, &extension);
        validate_burn_event(&checked, &burn_event(&extension)).unwrap();
        assert_eq!(
            evaluate_record_credit(&checked).unwrap(),
            RecordCredit::Credited { total: 10 }
        );
    }
}

#[test]
fn valid_foreign_region_selection_cannot_reuse_an_extension() {
    let (mut common, mut extension) = burn(&[10]);
    let foreign = object(ObservedSide::Output, 7, ObjectId::Ash, 10);
    common.other_regions.push(ObservedOperationRegion {
        members: BTreeSet::from([foreign.reference]),
        ..Default::default()
    });
    common.objects.push(foreign);
    extension.context = context(&common);
    let expected = context(&common);
    bound(&common, &extension);
    std::mem::swap(&mut common.region, &mut common.other_regions[0]);
    // Both region selections are structurally normalizable. This subject
    // therefore reaches snapshot binding rather than malformed references.
    common.clone().validate_and_normalize().unwrap();
    assert_eq!(
        validate_burn_clear_observation(&common, &expected, Some(&extension)),
        Err(BurnClearRefusal::CommonObservation)
    );
}

fn amount(value: u64) -> ProtocolAmount {
    ProtocolAmount::new(value).unwrap()
}
fn reference(side: ObservedSide, ordinal: u32) -> ObservedObjectRef {
    ObservedObjectRef { side, ordinal }
}
fn object(side: ObservedSide, ordinal: u32, kind: ObjectId, value: u64) -> ObservedObject {
    ObservedObject {
        reference: reference(side, ordinal),
        kind: ObservedObjectKind::Declared(kind),
        asset: ObservedAsset::Declared(if kind == ObjectId::State {
            AssetId::Pid
        } else {
            AssetId::U
        }),
        value: ObservedValue::Protocol(amount(value)),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}

fn common(operation: OperationId, objects: Vec<ObservedObject>) -> OperationObservation {
    OperationObservation {
        operation,
        region: ObservedOperationRegion {
            members: objects.iter().map(|object| object.reference).collect(),
            ..Default::default()
        },
        objects,
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        other_regions: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::BurnRecordMax, Count::new(3)),
            (BoundId::AshBatchMax, Count::new(3)),
        ]),
    }
}

fn context(common: &OperationObservation) -> BurnClearContext {
    BurnClearContext::bind_observation(common.clone(), [1; 32], [2; 32]).unwrap()
}

fn burn(claims: &[u64]) -> (OperationObservation, BurnClearObservation) {
    let fresh_ash = reference(ObservedSide::Output, 0);
    let mut common = common(
        OperationId::Burn,
        vec![
            object(ObservedSide::Input, 0, ObjectId::ReceiptLive, 10),
            object(ObservedSide::Output, 0, ObjectId::Ash, 10),
        ],
    );
    common.projections.insert(ProjectionId::BurnEvent);
    common
        .region
        .canonical_partition
        .flows
        .push(ObservedCanonicalFlow {
            asset: AssetId::U,
            sources: vec![reference(ObservedSide::Input, 0)],
            destinations: vec![fresh_ash],
            movement_kind: Some(DeltaKind::Lateral),
            destructions: vec![],
        });
    let records = claims
        .iter()
        .enumerate()
        .map(|(ordinal, value)| BurnRecord {
            ordinal: u32::try_from(ordinal).unwrap(),
            address: BurnAddress([9; 32]),
            amount: amount(*value),
            tag: TagId::Burn,
            fresh_ash,
        })
        .collect();
    let extension = BurnClearObservation {
        context: context(&common),
        payload: BurnClearPayload::Burn { fresh_ash, records },
    };
    (common, extension)
}

fn clear() -> (OperationObservation, BurnClearObservation) {
    let mut common = common(
        OperationId::Clear,
        vec![
            object(ObservedSide::Input, 0, ObjectId::State, 1),
            object(ObservedSide::Input, 1, ObjectId::Ash, 10),
            object(ObservedSide::Output, 0, ObjectId::State, 1),
            object(ObservedSide::Output, 1, ObjectId::Ash, 4),
        ],
    );
    common.projections.insert(ProjectionId::ClearEvent);
    common.root_effects.push(ObservedRootEffect {
        root: RootId::State,
        effect: ObservedRootEffectKind::Succession,
    });
    common
        .region
        .canonical_partition
        .flows
        .push(ObservedCanonicalFlow {
            asset: AssetId::U,
            sources: vec![reference(ObservedSide::Input, 1)],
            destinations: vec![reference(ObservedSide::Output, 1)],
            movement_kind: Some(DeltaKind::OwnerlessLateral),
            destructions: vec![ObservedDestructionLeg {
                tag: TagId::Recon,
                amount: amount(6),
            }],
        });
    // Independent arithmetic: B=10, y_l=6, Y-1=8, hence X=6 and R=4.
    let predecessor = ObservedState {
        reference: reference(ObservedSide::Input, 0),
        metadata: StateMetadata {
            omega: amount(20),
            y_l: amount(6),
            y_t: amount(3),
            q: amount(2),
            cycle: Cycle::new(7),
            maturity: Maturity::Unannounced,
        },
    };
    let successor = ObservedState {
        reference: reference(ObservedSide::Output, 0),
        metadata: StateMetadata {
            y_l: amount(0),
            ..predecessor.metadata
        },
    };
    let extension = BurnClearObservation {
        context: context(&common),
        payload: BurnClearPayload::Clear {
            predecessor,
            successor,
            ash_sources: vec![reference(ObservedSide::Input, 1)],
            destruction: Destruction {
                asset: AssetId::U,
                amount: amount(6),
                tag: TagId::Recon,
                flow: Some(0),
            },
        },
    };
    (common, extension)
}

fn bound(
    common: &OperationObservation,
    extension: &BurnClearObservation,
) -> BoundBurnClearObservation {
    validate_burn_clear_observation(common, &context(common), Some(extension)).unwrap()
}

fn burn_event(extension: &BurnClearObservation) -> BurnEventProjection {
    let BurnClearPayload::Burn { fresh_ash, records } = &extension.payload else {
        panic!("Burn fixture")
    };
    BurnEventProjection {
        context: extension.context.clone(),
        fresh_ash: *fresh_ash,
        fresh_amount: amount(10),
        records: records.clone(),
    }
}

#[test]
fn credit_controls_preserve_burn_event_and_structure() {
    for (claims, total, credited) in [
        (vec![], 0_u128, true),
        (vec![3, 4], 7, true),
        (vec![4, 6], 10, true),
        (vec![6, 6], 12, false),
        (
            vec![PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1, 2],
            u128::from(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE) + 1,
            false,
        ),
    ] {
        let (common, extension) = burn(&claims);
        let bound = bound(&common, &extension);
        validate_burn_event(&bound, &burn_event(&extension)).unwrap();
        assert_eq!(
            evaluate_record_credit(&bound).unwrap(),
            if credited {
                RecordCredit::Credited { total }
            } else {
                RecordCredit::Overclaim {
                    total,
                    fresh_amount: amount(10),
                }
            }
        );
        validate_accounting(
            &bound,
            &AccountingProjection {
                context: context(&common),
                values: AccountingValues::Burn {
                    record_total: total,
                    fresh_amount: amount(10),
                },
                event: Some(ProjectionId::BurnEvent),
                distribution_residue: false,
            },
        )
        .unwrap();
    }
}

#[test]
fn clear_event_does_not_pre_sum_a_sealed_family() {
    for (y_t, refusal) in [
        (0, BurnClearRefusal::ClearTransition),
        (3, BurnClearRefusal::Arithmetic),
    ] {
        let (mut common, mut extension) = clear();
        common.objects[1].value =
            ObservedValue::Protocol(amount(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1));
        common.claim_object(object(ObservedSide::Input, 2, ObjectId::Ash, 2));
        common.region.canonical_partition.flows[0]
            .sources
            .push(reference(ObservedSide::Input, 2));
        if let BurnClearPayload::Clear {
            predecessor,
            ash_sources,
            ..
        } = &mut extension.payload
        {
            predecessor.metadata.y_l = amount(0);
            predecessor.metadata.y_t = amount(y_t);
            ash_sources.push(reference(ObservedSide::Input, 2));
        }
        extension.context = context(&common);
        let checked = bound(&common, &extension);
        assert_eq!(
            validate_clear_event(&checked, &literal_clear_event(&common)),
            Err(refusal)
        );
    }
}

#[test]
fn records_accept_repeated_addresses_maximum_and_contiguous_ordinals() {
    let (common, extension) = burn(&[2, 3, 5]);
    let bound = bound(&common, &extension);
    assert_eq!(
        evaluate_record_credit(&bound).unwrap(),
        RecordCredit::Credited { total: 10 }
    );
}

#[test]
fn structural_refusals_have_one_coordinate_controls() {
    use BurnClearRefusal as R;
    let mutations: [Mutation; 7] = [
        (R::RecordOrdinal, |e| {
            if let BurnClearPayload::Burn { records, .. } = &mut e.payload {
                records[0].ordinal = 1;
            }
        }),
        (R::RecordOrdinal, |e| {
            if let BurnClearPayload::Burn { records, .. } = &mut e.payload {
                records[1].ordinal = 0;
            }
        }),
        (R::RecordStructure, |e| {
            if let BurnClearPayload::Burn { records, .. } = &mut e.payload {
                records[0].amount = amount(0);
            }
        }),
        (R::RecordStructure, |e| {
            if let BurnClearPayload::Burn { records, .. } = &mut e.payload {
                records[0].tag = TagId::Recon;
            }
        }),
        (R::DataAssociation, |e| {
            if let BurnClearPayload::Burn { records, .. } = &mut e.payload {
                records[0].fresh_ash.ordinal = 1;
            }
        }),
        (R::AshBinding, |e| {
            if let BurnClearPayload::Burn { fresh_ash, .. } = &mut e.payload {
                fresh_ash.ordinal = 1;
            }
        }),
        (R::Operation, |e| e.payload = BurnClearPayload::Compact),
    ];
    for (refusal, mutate) in mutations {
        let (common, mut extension) = burn(&[4, 6]);
        bound(&common, &extension);
        mutate(&mut extension);
        assert_eq!(
            validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
            Err(refusal)
        );
    }
    let (mut common, mut extension) = burn(&[2, 3, 5]);
    bound(&common, &extension);
    common.bounds.insert(BoundId::BurnRecordMax, Count::new(2));
    extension.context = context(&common);
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(R::RecordBound)
    );
    common.bounds.remove(&BoundId::BurnRecordMax);
    extension.context = context(&common);
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(R::RecordBound)
    );
}

#[test]
fn binding_refuses_wrong_region_certificate_transaction_and_snapshot() {
    use BurnClearRefusal as R;
    let (common, extension) = burn(&[10]);
    let expected = context(&common);
    bound(&common, &extension);
    assert_eq!(
        validate_burn_clear_observation(&common, &expected, None),
        Err(R::MissingExtension)
    );
    for (transaction, certificate, refusal) in [
        ([3; 32], [2; 32], R::Transaction),
        ([1; 32], [3; 32], R::Certificate),
    ] {
        let mut subject = extension.clone();
        subject.context =
            BurnClearContext::bind_observation(common.clone(), transaction, certificate).unwrap();
        assert_eq!(
            validate_burn_clear_observation(&common, &expected, Some(&subject)),
            Err(refusal)
        );
    }
    let mut foreign = common.clone();
    let fresh = reference(ObservedSide::Output, 0);
    foreign.region.members.remove(&fresh);
    foreign.other_regions.push(ObservedOperationRegion {
        members: BTreeSet::from([fresh]),
        ..Default::default()
    });
    assert_eq!(
        validate_burn_clear_observation(&foreign, &expected, Some(&extension)),
        Err(R::MalformedObservation)
    );
    let mut changed = common.clone();
    changed.objects[1].value = ObservedValue::Protocol(amount(11));
    assert_eq!(
        validate_burn_clear_observation(&changed, &expected, Some(&extension)),
        Err(R::CommonObservation)
    );
    let mut missing_certificate = common;
    missing_certificate
        .projections
        .remove(&ProjectionId::TransitionCertificate);
    let mut subject = extension;
    subject.context = context(&missing_certificate);
    assert_eq!(
        validate_burn_clear_observation(
            &missing_certificate,
            &context(&missing_certificate),
            Some(&subject)
        ),
        Err(R::Certificate)
    );
}

#[test]
fn normalization_preserves_binding_and_refusal_precedence_is_explicit() {
    use BurnClearRefusal as R;
    let (mut common, mut extension) = burn(&[6, 6]);
    common.objects.reverse();
    bound(&common, &extension);
    // Record content wins over ordinal, matching the model; both precede credit.
    if let BurnClearPayload::Burn { records, .. } = &mut extension.payload {
        records[0].ordinal = 2;
        records[0].amount = amount(0);
    }
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(R::RecordStructure)
    );
    extension.context =
        BurnClearContext::bind_observation(common.clone(), [3; 32], [4; 32]).unwrap();
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(R::Transaction)
    );
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), None),
        Err(R::MissingExtension)
    );
}

#[test]
fn clear_carrier_refusals_reach_independent_boundaries() {
    use BurnClearRefusal as R;
    let mutations: [Mutation; 6] = [
        (R::StateBinding, |e| {
            if let BurnClearPayload::Clear { predecessor, .. } = &mut e.payload {
                predecessor.reference.ordinal = 1;
            }
        }),
        (R::StateBinding, |e| {
            if let BurnClearPayload::Clear { successor, .. } = &mut e.payload {
                successor.reference.side = ObservedSide::Input;
            }
        }),
        (R::AshBinding, |e| {
            if let BurnClearPayload::Clear { ash_sources, .. } = &mut e.payload {
                ash_sources[0].ordinal = 0;
            }
        }),
        (R::DataAssociation, |e| {
            if let BurnClearPayload::Clear { destruction, .. } = &mut e.payload {
                destruction.flow = None;
            }
        }),
        (R::DestructionBinding, |e| {
            if let BurnClearPayload::Clear { destruction, .. } = &mut e.payload {
                destruction.amount = amount(5);
            }
        }),
        (R::DestructionBinding, |e| {
            if let BurnClearPayload::Clear { destruction, .. } = &mut e.payload {
                destruction.tag = TagId::Burn;
            }
        }),
    ];
    for (refusal, mutate) in mutations {
        let (common, mut extension) = clear();
        bound(&common, &extension);
        mutate(&mut extension);
        assert_eq!(
            validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
            Err(refusal)
        );
    }
}

#[test]
fn clear_event_accounting_and_succession_are_separate_subjects() {
    let (common, extension) = clear();
    let bound = bound(&common, &extension);
    let mut event = literal_clear_event(&common);
    let mut edge = StateSuccessionProjection {
        context: context(&common),
        predecessor: literal_state(ObservedSide::Input, 6),
        successor: literal_state(ObservedSide::Output, 0),
    };
    let mut accounting = AccountingProjection {
        context: context(&common),
        values: AccountingValues::Clear {
            aggregate: amount(10),
            clear_amount: amount(6),
            state_decrement: amount(6),
            destruction: event.destruction,
            residual: amount(4),
        },
        event: Some(ProjectionId::ClearEvent),
        distribution_residue: false,
    };
    validate_clear_event(&bound, &event).unwrap();
    validate_state_succession(&bound, &edge).unwrap();
    validate_accounting(&bound, &accounting).unwrap();
    event.y = amount(4);
    assert_eq!(
        validate_clear_event(&bound, &event),
        Err(BurnClearRefusal::ClearEvent)
    );
    validate_accounting(&bound, &accounting).unwrap();
    edge.predecessor.reference.ordinal = 1;
    assert_eq!(
        validate_state_succession(&bound, &edge),
        Err(BurnClearRefusal::StateSuccession)
    );
    if let AccountingValues::Clear { residual, .. } = &mut accounting.values {
        *residual = amount(5);
    }
    assert_eq!(
        validate_accounting(&bound, &accounting),
        Err(BurnClearRefusal::Accounting)
    );
    assert_eq!(
        evaluate_record_credit(&bound),
        Err(BurnClearRefusal::ProjectionRole)
    );
}

#[test]
fn burn_event_corruption_does_not_supply_accounting_or_credit() {
    let (common, extension) = burn(&[10]);
    let bound = bound(&common, &extension);
    let mut event = literal_burn_event(&common);
    validate_burn_event(&bound, &event).unwrap();
    event.fresh_amount = amount(9);
    assert_eq!(
        validate_burn_event(&bound, &event),
        Err(BurnClearRefusal::BurnEvent)
    );
    assert_eq!(
        evaluate_record_credit(&bound).unwrap(),
        RecordCredit::Credited { total: 10 }
    );
}

#[test]
fn compact_accounting_and_silence_do_not_create_fresh_credit() {
    let mut common = common(
        OperationId::CompactAsh,
        vec![
            object(ObservedSide::Input, 0, ObjectId::Ash, 10),
            object(ObservedSide::Output, 0, ObjectId::Ash, 10),
        ],
    );
    let mut extension = BurnClearObservation {
        context: context(&common),
        payload: BurnClearPayload::Compact,
    };
    let checked = bound(&common, &extension);
    let silence = CompactSilenceProjection {
        context: context(&common),
    };
    validate_compact_silence(&checked, &silence).unwrap();
    let accounting = AccountingProjection {
        context: context(&common),
        values: AccountingValues::Compact {
            input: amount(10),
            output: amount(10),
        },
        event: None,
        distribution_residue: false,
    };
    validate_accounting(&checked, &accounting).unwrap();
    assert_eq!(
        evaluate_record_credit(&checked),
        Err(BurnClearRefusal::ProjectionRole)
    );
    common.projections.insert(ProjectionId::BurnEvent);
    extension.context = context(&common);
    let checked = bound(&common, &extension);
    assert_eq!(
        validate_compact_silence(
            &checked,
            &CompactSilenceProjection {
                context: context(&common)
            }
        ),
        Err(BurnClearRefusal::CompactSilence)
    );
    assert_eq!(
        validate_accounting(&checked, &accounting),
        Err(BurnClearRefusal::Accounting)
    );
}

#[test]
fn state_fields_are_exhaustive_operation_qualified_and_never_defaulted() {
    let (common, extension) = clear();
    let checked = bound(&common, &extension);
    let expected = [
        SemanticValue::Amount(amount(20)),
        SemanticValue::Amount(amount(6)),
        SemanticValue::Amount(amount(3)),
        SemanticValue::Amount(amount(2)),
        SemanticValue::Cycle(Cycle::new(7)),
        SemanticValue::Maturity(Maturity::Unannounced),
    ];
    assert_eq!(StateField::ALL.len(), expected.len());
    for (field, value) in StateField::ALL.iter().zip(expected) {
        let fact = FactId::StateField {
            operation: OperationId::Clear,
            side: TransactionSide::Input,
            field: *field,
        };
        assert_eq!(checked.resolve_state_fact(&fact), Some(value));
        let foreign = FactId::StateField {
            operation: OperationId::AnnounceMaturity,
            side: TransactionSide::Input,
            field: *field,
        };
        assert_eq!(checked.resolve_state_fact(&foreign), None);
    }
    let (burn_common, burn_extension) = burn(&[]);
    assert_eq!(
        bound(&burn_common, &burn_extension).resolve_state_fact(&FactId::StateField {
            operation: OperationId::Clear,
            side: TransactionSide::Input,
            field: StateField::YL
        }),
        None
    );
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), None),
        Err(BurnClearRefusal::MissingExtension)
    );
    // The production initializer remains outside this packet. Reuse its existing
    // maturity seam control without changing its constructor or evaluator.
    let maturity = super::announce_maturity_tests::observation();
    let spec = super::announce_maturity_tests::realization();
    let report = spec.evaluate_operation(&maturity).unwrap();
    assert_eq!(report.verdicts.len(), 26);
    assert!(report.is_conformant());
    assert!(!report.is_evidence_complete());
}

#[test]
fn arithmetic_refusal_has_a_valid_clear_control() {
    let (common, mut extension) = clear();
    let checked = bound(&common, &extension);
    validate_clear_event(&checked, &literal_clear_event(&common)).unwrap();
    if let BurnClearPayload::Clear { successor, .. } = &mut extension.payload {
        successor.metadata.y_l = amount(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1);
    }
    let checked = bound(&common, &extension);
    assert_eq!(
        validate_clear_event(&checked, &literal_clear_event(&common)),
        Err(BurnClearRefusal::Arithmetic)
    );
}
