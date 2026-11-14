//! Full final histories: byte reader versus graph fold and literal set rules.
use super::{decoded_manifest as manifest, hex};
use crate::{keyed_evidence as e, keyed_history_reader as r};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use target_elements_conformance::constructor::tagged::sha256;
#[allow(
    dead_code,
    reason = "The unchanged graph model is shared with its exhaustive existing test suite."
)]
#[path = "../keyed_history_model.rs"]
mod model;

fn address(raw: &[u8]) -> r::Address {
    let tx: elements::Transaction = elements::encode::deserialize(raw).unwrap();
    r::Address(tx.txid().to_byte_array())
}
const fn handle(key: [u8; 32], event: u32, position: usize) -> r::Handle {
    r::Handle {
        key: r::Key(key),
        output: r::Output { event, position },
    }
}
const fn from_model(h: model::Handle) -> r::Handle {
    handle(h.key, h.output.event, h.output.position)
}

pub(super) fn input(
    manifest: &crate::keyed_final_capture::CaptureManifest,
    session: &Value,
) -> r::Input {
    let rows: BTreeMap<_, _> = manifest
        .rows
        .iter()
        .map(|row| (hex(&row.digest), row))
        .collect();
    let bytes: BTreeMap<_, _> = session["observations"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, row)| row["layer"] == "accepted")
        .map(|(index, row)| {
            (
                u32::try_from(index + 1).unwrap(),
                rows[row["digest"].as_str().unwrap()].submitted.clone(),
            )
        })
        .collect();
    let association: Vec<_> = bytes.iter().map(|(id, raw)| (*id, address(raw))).collect();
    let retrieved: BTreeMap<_, _> = session["parents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|parent| {
            let raw = super::bytes(&parent["raw_transaction"]).unwrap();
            let tx: elements::Transaction = elements::encode::deserialize(&raw).unwrap();
            assert_eq!(parent["transaction_id"], tx.txid().to_string());
            assert_eq!(parent["witness_transaction_id"], tx.wtxid().to_string());
            (address(&raw), raw)
        })
        .collect();
    let assertions: BTreeSet<_> = association
        .iter()
        .map(|(_, address)| *address)
        .chain(retrieved.keys().copied())
        .collect();
    r::Input {
        bytes,
        association,
        retrieved,
        keys: BTreeSet::from([
            r::Key([42; 32]),
            r::Key(crate::keyed_final_manifest::SECOND_KEY),
        ]),
        boundary_premises: BTreeSet::new(),
        assertions: r::Assertions {
            inclusion: assertions.clone(),
            retrieval: assertions,
        },
    }
}

fn graph_events(input: &r::Input) -> Vec<model::Event> {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let programs: BTreeMap<_, _> = input
        .keys
        .iter()
        .map(|key| {
            let recipe = crate::keyed_constructor::construct_keyed(
                &target,
                tapscript::keyed_constructor::KeyedSemanticKey::new(key.0),
                &crate::keyed_constructor::PublicKeyedCurve,
            )
            .unwrap();
            (recipe.output_program, key.0)
        })
        .collect();
    let names: BTreeMap<_, _> = input
        .association
        .iter()
        .map(|(id, address)| (*address, *id))
        .collect();
    input
        .bytes
        .iter()
        .map(|(id, raw)| {
            let tx: elements::Transaction = elements::encode::deserialize(raw).unwrap();
            model::Event {
                id: *id,
                inputs: tx
                    .input
                    .iter()
                    .filter_map(|i| {
                        names
                            .get(&r::Address(i.previous_output.txid.to_byte_array()))
                            .map(|event| model::Output {
                                event: *event,
                                position: i.previous_output.vout as usize,
                            })
                    })
                    .collect(),
                outputs: tx
                    .output
                    .iter()
                    .map(|o| programs.get(o.script_pubkey.as_bytes()).copied())
                    .collect(),
            }
        })
        .collect()
}

fn compare_graph(input: &r::Input, reading: &r::Reading) {
    let graph = graph_events(input);
    let expected = model::analyze(&graph).unwrap();
    assert_eq!(
        reading.live(),
        &expected.live().iter().copied().map(from_model).collect()
    );
    for event in &graph {
        for (position, key) in event.outputs.iter().enumerate() {
            if let Some(key) = key {
                let h = handle(*key, event.id, position);
                let mh = model::Handle {
                    key: *key,
                    output: model::Output {
                        event: event.id,
                        position,
                    },
                };
                if position == 0 {
                    let literal: BTreeSet<_> = expected
                        .records(mh)
                        .unwrap()
                        .iter()
                        .copied()
                        .map(from_model)
                        .collect();
                    assert!(!literal.is_empty());
                    assert_eq!(reading.memberships()[&h], r::Membership::Complete(literal));
                } else {
                    assert_eq!(reading.query(h), r::Query::NotHandle);
                    assert_eq!(expected.query(mh), model::OutputStatus::NotHandle);
                    assert_eq!(expected.records(mh), None);
                }
            }
        }
    }
}

struct PreparedHistory {
    case: Value,
    manifest: crate::keyed_final_capture::CaptureManifest,
    input: r::Input,
    native: crate::final_native::NativeHistory,
    last: r::Handle,
    key_index: usize,
}
fn prepare_history(case: Value, key_index: usize) -> PreparedHistory {
    let _span = tracing::info_span!("final_history", key_index).entered();
    let session = &case["sessions"][key_index];
    let manifest = manifest(&case).unwrap();
    let input = input(&manifest, session);
    finish_history(case, manifest, input, key_index)
}
fn finish_history(
    case: Value,
    manifest: crate::keyed_final_capture::CaptureManifest,
    input: r::Input,
    key_index: usize,
) -> PreparedHistory {
    let native_history = crate::final_native::NativeHistory::prepare(input.clone()).unwrap();
    let reading = native_history.reading();
    let last = *reading
        .memberships()
        .keys()
        .max_by_key(|h| h.output.event)
        .unwrap();
    PreparedHistory {
        case,
        manifest,
        input,
        native: native_history,
        last,
        key_index,
    }
}
pub(super) fn check(case: &Value, key_index: usize) {
    let prepared = prepare_history(case.clone(), key_index);
    check_model(&prepared);
    check_joins(&prepared, 0, usize::MAX);
    check_controls(&prepared);
    let context = e::prepare_families(&prepared.input).unwrap();
    for live in prepared.native.reading().live() {
        let request = request(*live);
        let validators: [(e::Family, Validator); 6] = [
            (e::Family::Birth, e::validate_birth),
            (e::Family::Continuation, e::validate_continuation),
            (e::Family::Recurrence, e::validate_recurrence),
            (e::Family::Reading, e::validate_reading),
            (e::Family::Acceptance, e::validate_acceptance),
            (e::Family::Resources, e::validate_resources),
        ];
        for (family, validate) in validators {
            let report = e::family_report(&context, family, &request).unwrap();
            validate(&context, &request, &report).unwrap();
        }
    }
}
pub(super) fn check_wide(case_index: usize) {
    for key in 0..2 {
        let (case, manifest, input) =
            timed_input(case_index, key, "load-history", "decode-history");
        let prepared = Arc::new(super::semantic_worker(
            case_index,
            key,
            "prepare-history",
            0,
            move || finish_history(case, manifest, input, key),
        ));
        let model = Arc::clone(&prepared);
        super::semantic_worker(case_index, key, "model", 0, move || check_model(&model));
        for chunk in 0..prepared.input.bytes.len().div_ceil(128) {
            let joined = Arc::clone(&prepared);
            super::semantic_worker(case_index, key, "join", chunk, move || {
                check_joins(&joined, chunk * 128, 128);
            });
        }
        if prepared.input.bytes.len() < 20 {
            super::semantic_worker(case_index, key, "controls", 0, move || {
                check_controls(&prepared);
            });
        }
    }
}
fn timed_input(
    case_index: usize,
    key: usize,
    load_phase: &'static str,
    decode_phase: &'static str,
) -> (Value, crate::keyed_final_capture::CaptureManifest, r::Input) {
    let case = super::semantic_worker(case_index, key, load_phase, 0, move || {
        super::load(5, case_index).unwrap()
    });
    super::semantic_worker(case_index, key, decode_phase, 0, move || {
        let manifest = manifest(&case).unwrap();
        let input = input(&manifest, &case["sessions"][key]);
        (case, manifest, input)
    })
}
fn check_model(prepared: &PreparedHistory) {
    let case = &prepared.case;
    let session = &case["sessions"][prepared.key_index];
    let reading = prepared.native.reading();
    let last = prepared.last;
    compare_graph(&prepared.input, reading);
    if let Some(count) = case["manifest"]["target"]["scaling"].as_u64() {
        let count = u32::try_from(count).unwrap();
        let key: [u8; 32] = serde_json::from_value(session["key"].clone()).unwrap();
        let sources = count.max(1);
        for event in 1..=sources {
            let h = handle(key, event, 0);
            assert_eq!(
                reading.memberships()[&h],
                r::Membership::Complete(BTreeSet::from([h]))
            );
        }
        if count > 0 {
            assert_eq!(
                reading.memberships()[&last],
                r::Membership::Complete((1..=count).map(|event| handle(key, event, 0)).collect())
            );
            assert_eq!(reading.relations().len(), 2 * count as usize);
        }
        assert_eq!(reading.live(), &BTreeSet::from([last]));
    }
}
fn check_joins(prepared: &PreparedHistory, first: usize, count: usize) {
    let input = &prepared.input;
    let session = &prepared.case["sessions"][prepared.key_index];
    let reading = prepared.native.reading();
    // Exact event/input/member/fragment joins, including structural accepted rows.
    let decoded: BTreeMap<_, _> = input
        .bytes
        .iter()
        .skip(first)
        .take(count)
        .map(|(event, raw)| (*event, transaction::TargetTransaction::decode(raw).unwrap()))
        .collect();
    for relation in reading
        .relations()
        .iter()
        .filter(|relation| decoded.contains_key(&relation.event))
    {
        let tx = &decoded[&relation.event];
        assert_eq!(
            relation.member.address.0,
            *tx.inputs()[relation.input].outpoint().txid().internal()
        );
        assert_eq!(
            relation.member.position,
            tx.inputs()[relation.input].outpoint().index()
        );
        assert_eq!(relation.leaf, tx.witnesses()[relation.input].stack()[0]);
        assert_eq!(relation.control, tx.witnesses()[relation.input].stack()[1]);
    }
    let rows: BTreeMap<_, _> = prepared
        .manifest
        .rows
        .iter()
        .map(|r| (r.digest, r))
        .collect();
    let observations: BTreeMap<_, _> = session["observations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| (o["digest"].as_str().unwrap(), o))
        .collect();
    for (event, raw) in input.bytes.iter().skip(first).take(count) {
        let digest = sha256(raw);
        let row = rows[&digest];
        let observation = observations[hex(&digest).as_str()];
        let joined = prepared
            .native
            .join(
                *event,
                row,
                observation,
                session["diagnostic"].as_str().unwrap(),
            )
            .unwrap();
        assert_eq!(joined.subject, digest);
        assert_eq!(
            joined.relations.len(),
            reading
                .relations()
                .iter()
                .filter(|r| r.event == *event)
                .count()
        );
    }
}
fn check_controls(prepared: &PreparedHistory) {
    if prepared.input.bytes.len() < 20 {
        missing(&prepared.input, prepared.last);
        refused_prefixes(
            &prepared.input,
            &prepared.case["sessions"][prepared.key_index],
            &prepared.manifest,
        );
    }
}

fn refused_prefixes(
    input: &r::Input,
    session: &Value,
    manifest: &crate::keyed_final_capture::CaptureManifest,
) {
    use crate::keyed_final_capture::CaptureSchema as S;
    for (index, observation) in session["observations"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        if observation["layer"] == "accepted" {
            continue;
        }
        let row = manifest
            .rows
            .iter()
            .find(|row| observation["digest"] == hex(&row.digest))
            .unwrap();
        let event = u32::try_from(index + 1).unwrap();
        let mut prefix = input.clone();
        prefix.bytes.retain(|id, _| *id < event);
        prefix.association.retain(|(id, _)| *id < event);
        // The flagged transaction is deliberately outside the canonical decoder's domain.
        let id = if row.schema == S::Flagged {
            r::Address([0; 32])
        } else {
            address(&row.submitted)
        };
        prefix.bytes.insert(event, row.submitted.clone());
        prefix.association.push((event, id));
        prefix.assertions.inclusion.insert(id);
        prefix.assertions.retrieval.insert(id);
        let result = r::read(&prefix);
        match row.schema {
            S::LaterBorn => assert!(matches!(result, Err(r::Refusal::LaterBirth { .. }))),
            S::Flagged => assert!(matches!(result, Err(r::Refusal::UnsupportedForm { .. }))),
            S::WrongKey | S::WrongScript | S::ChangedMember => {
                assert!(
                    matches!(result, Err(r::Refusal::WrongSuccessor { .. })),
                    "{:?}: {result:?}",
                    row.schema
                );
            }
            S::WrongVersion | S::DisplacedSuccessor => {
                assert!(matches!(result, Err(r::Refusal::WrongVersion { .. })));
            }
            S::WrongParity => assert!(matches!(result, Err(r::Refusal::WrongParity { .. }))),
            S::UncommittedLeaf => {
                assert!(matches!(result, Err(r::Refusal::UncommittedLeaf { .. })));
            }
            S::KeyPathAttempt => assert!(matches!(result, Err(r::Refusal::WitnessShape { .. }))),
            S::FundingPoint(_) => {
                result.unwrap();
            } // Native sponsor/balance checks are not reader clauses.
            _ => panic!("reviewed refused schema"),
        }
    }
}

fn request(h: r::Handle) -> e::Request {
    e::Request {
        handle: h,
        scope: e::Scope::HandleClosure,
        family: e::ReadingFamily::Acceptance,
        completeness: e::Completeness::Required,
        rule: e::Rule::AllSourceRecords,
        observed_epoch: e::Epoch(1),
        finality: e::FinalityRequest::NotRequested,
        declaration: None,
        provenance: e::Provenance {
            deployment: e::Deployment(
                "retained isolated regtest observation; no deployment activation".into(),
            ),
            policy_version: e::PolicyVersion("final capture policy per exact subject".into()),
            anchor: e::HistoryAnchor("sealed case and session prefix".into()),
            history_source: e::EvidenceSource("sealed submitted bytes and mined readbacks".into()),
            policy_source: e::EvidenceSource(
                "sealed per-subject native policy observations".into(),
            ),
            policy_readings: BTreeMap::new(),
        },
        sources: e::PremiseSources {
            inclusion: e::EvidenceSource(
                "retained native mined readbacks; conditional validity".into(),
            ),
            unspentness: e::EvidenceSource(
                "observed successful spend at its session prefix only".into(),
            ),
            retrieval: e::EvidenceSource(
                "retained public parent bytes; completeness remains a premise".into(),
            ),
            finality: e::EvidenceSource("absent deployment finality".into()),
        },
    }
}
type Validator = fn(&e::FamilyContext, &e::Request, &e::Report) -> Result<(), e::FamilyRefusal>;
fn graph_expectation(input: &r::Input) -> (r::Handle, BTreeSet<r::Handle>, u64) {
    let graph = graph_events(input);
    let expected = model::analyze(&graph).unwrap();
    assert_eq!(expected.live().len(), 1);
    let last = *expected.live().first().unwrap();
    let records = expected
        .records(last)
        .unwrap()
        .iter()
        .copied()
        .map(from_model)
        .collect();
    let members = graph
        .iter()
        .find(|event| event.id == last.output.event)
        .unwrap()
        .inputs
        .iter()
        .filter(|output| {
            expected
                .records(model::Handle {
                    key: last.key,
                    output: **output,
                })
                .is_some()
        })
        .count() as u64;
    (from_model(last), records, members)
}
pub(super) fn check_families(case_index: usize, key: usize, rule_group: bool) {
    let phase = if rule_group { "rule" } else { "family" };
    let prepare_phase = if rule_group {
        "prepare-rules"
    } else {
        "prepare-families"
    };
    let (load_phase, decode_phase, graph_phase) = if rule_group {
        ("load-rules", "decode-rules", "graph-rules")
    } else {
        ("load-families", "decode-families", "graph-families")
    };
    let (_, _, input) = timed_input(case_index, key, load_phase, decode_phase);
    let (input, last, records, members) =
        super::semantic_worker(case_index, key, graph_phase, 0, move || {
            let (last, records, members) = graph_expectation(&input);
            (input, last, Arc::new(records), members)
        });
    let context = super::semantic_worker(case_index, key, prepare_phase, 0, move || {
        Arc::new(e::prepare_families(&input).unwrap())
    });
    let count = if rule_group { 4 } else { 6 };
    for part in 0..count {
        let context = Arc::clone(&context);
        let records = Arc::clone(&records);
        super::semantic_worker(case_index, key, phase, part, move || {
            if rule_group {
                rules(&context, &request(last), &records, part);
            } else {
                families(&context, last, part, members);
            }
        });
    }
}
#[allow(
    clippy::too_many_lines,
    reason = "Keep the independent one-coordinate report controls beside their common accepted control."
)]
fn families(context: &e::FamilyContext, last: r::Handle, index: usize, members: u64) {
    let pairs: [(e::Family, Validator); 6] = [
        (e::Family::Birth, e::validate_birth),
        (e::Family::Continuation, e::validate_continuation),
        (e::Family::Recurrence, e::validate_recurrence),
        (e::Family::Reading, e::validate_reading),
        (e::Family::Acceptance, e::validate_acceptance),
        (e::Family::Resources, e::validate_resources),
    ];
    let input = context.input();
    let request = request(last);
    let (family, validate) = pairs[index];
    let report = e::family_report(context, family, &request).unwrap();
    validate(context, &request, &report).unwrap();
    if let e::Body::Resources(body) = &report.body {
        assert_eq!(body.members, members);
        assert_eq!(body.members_unit, e::Unit::Items);
        for (quantity, value, unit) in [
            (&body.script_stack_bytes, 88 * members, e::Unit::Bytes),
            (&body.member_witness_bytes, 91 * members, e::Unit::Bytes),
            (&body.member_weight, 255 * members, e::Unit::WeightUnits),
        ] {
            assert_eq!(quantity.predicted, e::Measurement::Present { value, unit });
            assert_eq!(quantity.direct, e::Measurement::Present { value, unit });
        }
    }
    if input.bytes.len() < 20 {
        let mut wrong = report.clone();
        wrong
            .header
            .assumptions
            .as_mut()
            .unwrap()
            .retain(|a| !matches!(a, e::Assumption::Cryptographic { .. }));
        assert_eq!(
            validate(context, &request, &wrong).unwrap_err().condition,
            e::FamilyCondition::Framing(e::Refusal::AssumptionsMismatch)
        );
        let mut wrong = report.clone();
        for assumption in wrong.header.assumptions.as_mut().unwrap() {
            if let e::Assumption::Cryptographic { conditions, .. } = assumption {
                conditions.remove(&e::CryptographicCondition::NumsAndDiscreteLog);
            }
        }
        assert_eq!(
            validate(context, &request, &wrong).unwrap_err().condition,
            e::FamilyCondition::Framing(e::Refusal::AssumptionsMismatch)
        );
        let mut wrong = report.clone();
        wrong
            .header
            .quantities
            .get_mut(&e::Dimension::SubjectBytes)
            .unwrap()
            .observed = e::Measurement::Present {
            value: input.bytes[&last.output.event].len() as u64,
            unit: e::Unit::Records,
        };
        assert_eq!(
            validate(context, &request, &wrong).unwrap_err().condition,
            e::FamilyCondition::Framing(e::Refusal::QuantityUnit(e::Dimension::SubjectBytes))
        );
        let mut wrong = report.clone();
        for assumption in wrong.header.assumptions.as_mut().unwrap() {
            if let e::Assumption::Cryptographic { supplied_keys, .. } = assumption {
                supplied_keys.insert(r::Key([99; 32]));
            }
        }
        assert_eq!(
            validate(context, &request, &wrong).unwrap_err().condition,
            e::FamilyCondition::Framing(e::Refusal::AssumptionsMismatch)
        );
        for member in [false, true] {
            let mut wrong = report.clone();
            let relations = match &mut wrong.body {
                e::Body::Birth(body) => &mut body.relations,
                e::Body::Continuation(body) => &mut body.relations,
                _ => continue,
            };
            let Some(first) = relations.first_mut() else {
                continue;
            };
            if member {
                first.member.key.0[0] ^= 1;
            } else {
                first.relation = match first.relation {
                    r::Relation::BornFirst => r::Relation::ContinueFirst,
                    r::Relation::ContinueFirst => r::Relation::BornFirst,
                };
            }
            assert_eq!(
                validate(context, &request, &wrong).unwrap_err().condition,
                if member {
                    e::FamilyCondition::RelationMember
                } else {
                    e::FamilyCondition::RelationKind
                }
            );
        }
    }
}
fn rules(
    context: &e::FamilyContext,
    request: &e::Request,
    records: &BTreeSet<r::Handle>,
    selected: usize,
) {
    let record = *records.first().unwrap();
    let tx = transaction::TargetTransaction::decode(&context.input().bytes[&record.output.event])
        .unwrap();
    let transaction::AssetField::Explicit(asset) = tx.outputs()[0].asset() else {
        panic!("explicit source")
    };
    for (index, (rule, epoch, expected)) in [
        (e::Rule::AllSourceRecords, e::Epoch(1), records.clone()),
        (
            e::Rule::Fixed {
                asset: e::Asset(*asset.internal()),
                threshold: e::Amount(u64::MAX),
            },
            e::Epoch(1),
            BTreeSet::new(),
        ),
        (
            e::Rule::Changing {
                record,
                enabled_epoch: e::Epoch(1),
                disabled_epoch: e::Epoch(2),
            },
            e::Epoch(1),
            records.clone(),
        ),
        (
            e::Rule::Changing {
                record,
                enabled_epoch: e::Epoch(1),
                disabled_epoch: e::Epoch(2),
            },
            e::Epoch(2),
            records.iter().copied().filter(|h| *h != record).collect(),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if index != selected {
            continue;
        }
        let mut request = request.clone();
        request.rule = rule;
        request.observed_epoch = epoch;
        let report = e::family_report(context, e::Family::Acceptance, &request).unwrap();
        e::validate_acceptance(context, &request, &report).unwrap();
        let e::Body::Acceptance(body) = report.body else {
            panic!("acceptance")
        };
        let e::Outcome::Complete(outcome) = body.reading.outcome else {
            panic!("complete")
        };
        assert_eq!(&outcome.reading.membership, records);
        assert_eq!(outcome.accepted, expected);
    }
}
fn missing(input: &r::Input, last: r::Handle) {
    let control = r::read(input).unwrap();
    assert!(matches!(
        control.query(last),
        r::Query::Handle {
            membership: r::Membership::Complete(_),
            ..
        }
    ));
    let (source, index, first, position) = control.memberships().iter().find_map(|(h, membership)| {
        if !matches!(membership, r::Membership::Complete(records) if records.len() == 1 && records.contains(h)) {
            return None;
        }
        let tx: elements::Transaction = elements::encode::deserialize(&input.bytes[&h.output.event]).unwrap();
        tx.input.iter().enumerate().find_map(|(index, member)| {
            let parent = r::Address(member.previous_output.txid.to_byte_array());
            input.retrieved.contains_key(&parent).then_some((*h, index, parent, member.previous_output.vout))
        })
    }).unwrap();
    let event = source.output.event;
    let mut missing = input.clone();
    // Removing this SOURCE's funding ancestor leaves its assertions unchanged.
    missing.retrieved.remove(&first);
    let reading = r::read(&missing).unwrap();
    let r::Membership::Incomplete(causes) = &reading.memberships()[&source] else {
        panic!("removed funding parent leaves its spending SOURCE incomplete");
    };
    assert!(causes.contains(&r::Missing::Parent {
        event,
        input: index,
        address: first,
        position
    }));
    let mut premise = input.clone();
    let last_address = input
        .association
        .iter()
        .find(|(event, _)| *event == last.output.event)
        .unwrap()
        .1;
    premise.assertions.inclusion.remove(&last_address);
    let reading = r::read(&premise).unwrap();
    assert!(matches!(
        reading.query(last),
        r::Query::Handle {
            membership: r::Membership::Incomplete(causes),
            ..
        } if causes.contains(&r::Missing::Inclusion(last_address))
    ));
}

pub(super) fn further_key() {
    use transaction::{
        AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
        TargetTransaction, Txid, ValueField,
    };
    let key = [17; 32];
    assert_ne!(key, [42; 32]);
    assert_ne!(key, crate::keyed_final_manifest::SECOND_KEY);
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let recipe = crate::keyed_constructor::construct_keyed(
        &target,
        tapscript::keyed_constructor::KeyedSemanticKey::new(key),
        &crate::keyed_constructor::PublicKeyedCurve,
    )
    .unwrap();
    let output = |program| {
        TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal([3; 32])),
            ValueField::Explicit(1000),
            NonceField::Null,
            program,
        )
    };
    let parent = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([9; 32]), 0).unwrap(),
            1,
        )],
        vec![output(vec![0, 20].into_iter().chain([1; 20]).collect())],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode();
    let parent_address = address(&parent);
    let source = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal(parent_address.0), 0).unwrap(),
            1,
        )],
        vec![output(recipe.output_program)],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode();
    let source_address = address(&source);
    let input = r::Input {
        bytes: BTreeMap::from([(1, source)]),
        association: vec![(1, source_address)],
        keys: BTreeSet::from([r::Key(key)]),
        retrieved: BTreeMap::from([(parent_address, parent)]),
        boundary_premises: BTreeSet::new(),
        assertions: r::Assertions {
            inclusion: BTreeSet::from([source_address, parent_address]),
            retrieval: BTreeSet::from([source_address, parent_address]),
        },
    };
    let reading = r::read(&input).unwrap();
    let h = handle(key, 1, 0);
    assert_eq!(
        reading.memberships()[&h],
        r::Membership::Complete(BTreeSet::from([h]))
    );
    assert!(matches!(
        reading.query(handle([42; 32], 1, 0)),
        r::Query::WrongKey { .. }
    ));
    let mut absent = input.clone();
    absent.keys.clear();
    assert!(matches!(
        r::read(&absent).unwrap().query(h),
        r::Query::UnresolvedKey(_)
    ));
    families(&e::prepare_families(&input).unwrap(), h, 4, 0);
}
