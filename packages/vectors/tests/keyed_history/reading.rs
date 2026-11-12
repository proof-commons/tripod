//! Literal reader-rule expectations, separate from the report derivation.
use super::{fixture, reader};
use std::collections::{BTreeMap, BTreeSet};
use target_elements_conformance::constructor::tagged::sha256;
use vectors::keyed_evidence as e;
use vectors::keyed_history_reader as r;

pub(super) const fn handle(event: u32) -> r::Handle {
    r::Handle {
        key: r::Key([42; 32]),
        output: r::Output { event, position: 0 },
    }
}
pub(super) fn request(_input: &r::Input, event: u32) -> e::Request {
    e::Request {
        handle: handle(event),
        scope: e::Scope::HandleClosure,
        family: e::ReadingFamily::Acceptance,
        completeness: e::Completeness::Required,
        rule: e::Rule::AllSourceRecords,
        observed_epoch: e::Epoch(1),
        provenance: e::Provenance {
            deployment: e::Deployment("hypothetical-fixture".into()),
            policy_version: e::PolicyVersion("fixture-policy-1".into()),
            anchor: e::HistoryAnchor("external hypothetical chain position".into()),
            history_source: e::EvidenceSource("supplied byte fixture".into()),
            policy_source: e::EvidenceSource(
                "fixture caller readings, not native observations".into(),
            ),
            policy_readings: BTreeMap::from([(
                "fee/source".into(),
                "externally supplied; not used by acceptance".into(),
            )]),
        },
        sources: e::PremiseSources {
            inclusion: e::EvidenceSource("hypothetical valid-history inclusion".into()),
            unspentness: e::EvidenceSource("external per-input observation still owed".into()),
            retrieval: e::EvidenceSource("hypothetical finite retrieval".into()),
            finality: e::EvidenceSource("external permanence still owed".into()),
        },
        finality: e::FinalityRequest::NotRequested,
        declaration: None,
    }
}
pub(super) fn declaration(request: &e::Request) -> e::Declaration {
    e::Declaration {
        acceptance_source: e::EvidenceSource("hypothetical accepted fixture declaration".into()),
        deployment: request.provenance.deployment.clone(),
        policy_version: request.provenance.policy_version.clone(),
        anchor: request.provenance.anchor.clone(),
        rule: e::FinalityRule::Replacement(
            "hypothetical no replacement beyond named anchor".into(),
        ),
        inclusion_source: request.sources.inclusion.clone(),
        retrieval_source: request.sources.retrieval.clone(),
        unspentness_scope:
            "specified inputs at fixture observation epoch; not all historical inputs forever"
                .into(),
    }
}
pub(super) fn rules() {
    let fixture = fixture::build(fixture::graph("multi"));
    let input = reader::input(fixture.reader_inputs());
    let mut request = request(&input, 20);
    let all = e::reading_report(&input, &request).unwrap();
    premise_identities(&input, &all);
    assert_eq!(
        accepted(&all).reading.membership,
        BTreeSet::from([handle(10), handle(11)])
    );
    assert_eq!(accepted(&all).accepted, accepted(&all).reading.membership);
    assert_eq!(
        accepted(&all).annotations[&handle(10)].value,
        e::Value::Explicit(e::Amount(1010))
    );
    assert_eq!(
        accepted(&all).annotations[&handle(11)].value,
        e::Value::Explicit(e::Amount(1011))
    );
    assert_eq!(
        accepted(&all).annotations[&handle(10)].attribution,
        e::Attribution::Unassigned
    );
    assert_eq!(
        accepted(&all).annotations[&handle(10)].creating_bytes,
        input.bytes[&10]
    );
    assert_ne!(
        accepted(&all).annotations[&handle(10)].creating_bytes,
        input.bytes[&20]
    );
    request.rule = e::Rule::Fixed {
        asset: e::Asset([3; 32]),
        threshold: e::Amount(1011),
    };
    let fixed = e::reading_report(&input, &request).unwrap();
    assert_eq!(accepted(&fixed).accepted, BTreeSet::from([handle(11)]));
    assert_eq!(
        fixed.header.quantities[&e::Dimension::AcceptedRecords].observed,
        e::Measurement::Present {
            value: 1,
            unit: e::Unit::Records
        }
    );
    request.rule = e::Rule::Changing {
        record: handle(11),
        enabled_epoch: e::Epoch(1),
        disabled_epoch: e::Epoch(2),
    };
    request.observed_epoch = e::Epoch(2);
    let disabled = e::reading_report(&input, &request).unwrap();
    assert_eq!(accepted(&disabled).accepted, BTreeSet::from([handle(10)]));
    assert_eq!(
        accepted(&disabled).reading.membership,
        accepted(&fixed).reading.membership
    );
    assert_eq!(
        accepted(&fixed).reading.membership,
        accepted(&all).reading.membership
    );
    request.observed_epoch = e::Epoch(1);
    let enabled = e::reading_report(&input, &request).unwrap();
    assert_eq!(body(&enabled).rule, body(&disabled).rule);
    assert_ne!(body(&enabled).epoch, body(&disabled).epoch);
    assert_eq!(accepted(&enabled).accepted, accepted(&all).accepted);
    assert!(
        !accepted(&enabled)
            .accepted
            .is_subset(&accepted(&disabled).accepted)
    );
    rule_refusals(&input, &request);
    growth();
}
fn rule_refusals(input: &r::Input, accepted: &e::Request) {
    for (rule, epoch, expected) in [
        (
            e::Rule::Changing {
                record: handle(11),
                enabled_epoch: e::Epoch(1),
                disabled_epoch: e::Epoch(1),
            },
            e::Epoch(1),
            e::Refusal::SameEpoch,
        ),
        (
            e::Rule::Changing {
                record: handle(11),
                enabled_epoch: e::Epoch(1),
                disabled_epoch: e::Epoch(2),
            },
            e::Epoch(3),
            e::Refusal::UnknownEpoch,
        ),
    ] {
        assert!(e::reading_report(input, accepted).is_ok());
        let mut subject = accepted.clone();
        subject.rule = rule;
        subject.observed_epoch = epoch;
        assert_eq!(e::reading_report(input, &subject), Err(expected));
    }
}
fn growth() {
    let fixture = fixture::build(fixture::graph("path"));
    let complete = reader::input(fixture.reader_inputs());
    let rules = [
        e::Rule::AllSourceRecords,
        e::Rule::Fixed {
            asset: e::Asset([3; 32]),
            threshold: e::Amount(1011),
        },
    ];
    for rule in rules {
        let mut prior = BTreeSet::new();
        for end in [10, 20, 30, 40] {
            let mut prefix = complete.clone();
            prefix.bytes.retain(|id, _| *id <= end);
            prefix.association.retain(|(id, _)| *id <= end);
            let mut request = request(&complete, end);
            request.rule = rule.clone();
            let report = e::reading_report(&prefix, &request).unwrap();
            assert!(prior.is_subset(&accepted(&report).accepted));
            let expected = match (&rule, end) {
                (e::Rule::AllSourceRecords, 10 | 20) => BTreeSet::from([handle(10)]),
                (e::Rule::AllSourceRecords, _) => BTreeSet::from([handle(10), handle(11)]),
                (_, 10 | 20) => BTreeSet::new(),
                _ => BTreeSet::from([handle(11)]),
            };
            assert_eq!(accepted(&report).accepted, expected);
            prior.clone_from(&accepted(&report).accepted);
        }
    }
    let mut request = request(&complete, 30);
    request.rule = e::Rule::Changing {
        record: handle(11),
        enabled_epoch: e::Epoch(1),
        disabled_epoch: e::Epoch(2),
    };
    let mut prefix = complete.clone();
    prefix.bytes.retain(|id, _| *id <= 30);
    prefix.association.retain(|(id, _)| *id <= 30);
    let enabled = e::reading_report(&prefix, &request).unwrap();
    assert!(
        accepted(&enabled).reading.membership.contains(&handle(11)),
        "test's toggled identity exists"
    );
    assert_eq!(
        accepted(&enabled).accepted,
        BTreeSet::from([handle(10), handle(11)])
    );
    request.handle = handle(40);
    request.observed_epoch = e::Epoch(2);
    let disabled = e::reading_report(&complete, &request).unwrap();
    assert_eq!(accepted(&disabled).accepted, BTreeSet::from([handle(10)]));
    assert_eq!(body(&enabled).rule, body(&disabled).rule);
    // The rule is also defined before its named record is born.
    prefix.bytes.retain(|id, _| *id <= 10);
    prefix.association.retain(|(id, _)| *id <= 10);
    request.handle = handle(10);
    assert_eq!(
        accepted(&e::reading_report(&prefix, &request).unwrap()).accepted,
        BTreeSet::from([handle(10)])
    );
}

pub(super) fn premises() {
    let fixture = fixture::build(fixture::graph("source"));
    let input = reader::input(fixture.reader_inputs());
    let request = request(&input, 10);
    let report = e::reading_report(&input, &request).unwrap();
    assert_eq!(accepted(&report).reading.finality, e::Finality::Nonfinal);
    premise_identities(&input, &report);
    finality(&input, &request);
    completeness(&input, &request);
}
fn premise_identities(input: &r::Input, report: &e::Report) {
    let assumptions = report.header.assumptions.as_ref().unwrap();
    let parents: BTreeSet<_> = assumptions
        .iter()
        .filter_map(|a| match a {
            e::Assumption::Parent { obligation, .. } => Some(obligation.clone()),
            _ => None,
        })
        .collect();
    let mut expected = BTreeSet::new();
    for (event, bytes) in &input.bytes {
        let decoded: elements::Transaction = elements::encode::deserialize(bytes).unwrap();
        for (index, input) in decoded.input.iter().enumerate() {
            for fact in [
                r::ParentFact::Inclusion,
                r::ParentFact::Unspentness,
                r::ParentFact::Retrieval,
                r::ParentFact::Finality,
            ] {
                expected.insert(r::Obligation {
                    event: *event,
                    input: index,
                    address: r::Address(input.previous_output.txid.to_byte_array()),
                    position: input.previous_output.vout,
                    fact,
                });
            }
        }
    }
    assert_eq!(parents, expected);
    assert!(assumptions.iter().any(|a| matches!(a, e::Assumption::NoUnsuppliedKeyHandle { supplied_keys, .. } if supplied_keys == &input.keys)));
}
fn finality(input: &r::Input, accepted: &e::Request) {
    let mut request = accepted.clone();
    request.finality = e::FinalityRequest::Required;
    assert_eq!(
        e::reading_report(input, &request),
        Err(e::Refusal::FinalityMissing)
    );
    let form = declaration(&request);
    request.declaration = Some(e::DeclarationOffer::Hypothetical(
        e::HypotheticalDeclaration(form.clone()),
    ));
    assert_eq!(
        e::reading_report(input, &request),
        Err(e::Refusal::FinalityHypothetical)
    );
    request.finality = e::FinalityRequest::NotRequested;
    let report = e::reading_report(input, &request).unwrap();
    assert_eq!(e::validate_report(input, &request, &report), Ok(()));
    assert_eq!(
        super::reading::accepted(&report).reading.finality,
        e::Finality::Hypothetical(e::HypotheticalDeclaration(form.clone()))
    );
    let mut depth = request.clone();
    let mut depth_form = form.clone();
    depth_form.rule = e::FinalityRule::Depth(0);
    depth.declaration = Some(e::DeclarationOffer::Hypothetical(
        e::HypotheticalDeclaration(depth_form),
    ));
    assert!(e::reading_report(input, &depth).is_ok());
    for (coordinate, expected) in [
        (0, e::Refusal::FinalityDeployment),
        (1, e::Refusal::FinalityAnchor),
        (2, e::Refusal::FinalityPolicy),
        (3, e::Refusal::FinalityInclusionSource),
        (4, e::Refusal::FinalityRetrievalSource),
        (5, e::Refusal::FinalityUnspentnessScope),
        (6, e::Refusal::FinalityRule),
        (7, e::Refusal::FinalityAcceptanceSource),
        (8, e::Refusal::FinalityInclusionMismatch),
        (9, e::Refusal::FinalityRetrievalMismatch),
    ] {
        let mut changed = form.clone();
        match coordinate {
            0 => changed.deployment.0.push('x'),
            1 => changed.anchor.0.push('x'),
            2 => changed.policy_version.0.push('x'),
            3 => changed.inclusion_source.0.clear(),
            4 => changed.retrieval_source.0.clear(),
            5 => changed.unspentness_scope.clear(),
            6 => changed.rule = e::FinalityRule::Replacement(String::new()),
            7 => changed.acceptance_source.0.clear(),
            8 => changed.inclusion_source.0.push('x'),
            9 => changed.retrieval_source.0.push('x'),
            _ => unreachable!(),
        }
        assert!(e::reading_report(input, &request).is_ok());
        let mut subject = request.clone();
        subject.declaration = Some(e::DeclarationOffer::Hypothetical(
            e::HypotheticalDeclaration(changed),
        ));
        assert_eq!(e::reading_report(input, &subject), Err(expected));
    }
}
fn completeness(input: &r::Input, request: &e::Request) {
    let mut missing = input.clone();
    missing.retrieved.clear();
    assert!(e::reading_report(input, request).is_ok());
    let r::Query::Handle {
        membership: r::Membership::Incomplete(expected),
        ..
    } = r::read(&missing).unwrap().query(handle(10))
    else {
        panic!("withheld parent")
    };
    assert_eq!(
        e::reading_report(&missing, request),
        Err(e::Refusal::Incomplete {
            handle: handle(10),
            missing: expected
        })
    );
    let extra = extra_event(input, false);
    assert!(e::reading_report(&extra, request).is_ok());
    let mut whole = request.clone();
    whole.scope = e::Scope::WholeHistory;
    let missing = BTreeSet::from([r::Missing::Parent {
        event: 99,
        input: 0,
        address: r::Address([98; 32]),
        position: 1,
    }]);
    let closure = e::reading_report(&extra, request).unwrap();
    premise_identities(&extra, &closure);
    assert_eq!(
        body(&closure).reading.unexamined,
        BTreeMap::from([(99, missing.clone())])
    );
    assert_eq!(e::validate_report(&extra, request, &closure), Ok(()));
    let mut concealed = closure;
    body_mut(&mut concealed).reading.unexamined.clear();
    assert_eq!(
        e::validate_report(&extra, request, &concealed),
        Err(e::Refusal::UnexaminedMismatch)
    );
    assert_eq!(
        e::reading_report(&extra, &whole),
        Err(e::Refusal::WholeHistory {
            handles: BTreeMap::new(),
            events: BTreeMap::from([(99, missing.clone())])
        })
    );
    assert!(e::reading_report(input, &whole).is_ok());
    let keyed = extra_event(input, true);
    let keyed_report = e::reading_report(&keyed, request).unwrap();
    assert_eq!(
        body(&keyed_report).reading.incomplete,
        BTreeMap::from([(handle(99), missing.clone())])
    );
    premise_identities(&keyed, &keyed_report);
    let mut concealed = keyed_report;
    body_mut(&mut concealed).reading.incomplete.clear();
    assert_eq!(
        e::validate_report(&keyed, request, &concealed),
        Err(e::Refusal::IncompleteMismatch)
    );
    assert_eq!(
        e::reading_report(&keyed, &whole),
        Err(e::Refusal::WholeHistory {
            handles: BTreeMap::from([(handle(99), missing)]),
            events: BTreeMap::new()
        })
    );
    let mut absent = request.clone();
    absent.handle = handle(777);
    assert_eq!(
        e::reading_report(input, &absent),
        Err(e::Refusal::Query(r::Query::MissingEvent(777)))
    );
}
pub(super) fn extra_event(input: &r::Input, keyed: bool) -> r::Input {
    let mut extra = input.clone();
    let mut bytes = transaction::TargetTransaction::decode(&input.bytes[&10]).unwrap();
    let position = usize::from(!keyed);
    let ordinary = transaction::TargetOutput::new(
        bytes.outputs()[position].asset(),
        bytes.outputs()[position].value(),
        transaction::NonceField::Null,
        bytes.outputs()[position].program().to_vec(),
    );
    let outpoint =
        transaction::Outpoint::new(transaction::Txid::from_internal([98; 32]), 1).unwrap();
    bytes = transaction::TargetTransaction::new(
        2,
        vec![transaction::TargetInput::new(outpoint, 77)],
        vec![ordinary],
        0,
        vec![transaction::InputWitness::new(vec![])],
    )
    .unwrap();
    let encoded = bytes.encode();
    let address = r::Address(sha256(&sha256(&bytes.encode_without_witness())));
    extra.bytes.insert(99, encoded);
    extra.association.push((99, address));
    extra.assertions.inclusion.insert(address);
    extra.assertions.retrieval.insert(address);
    extra
}

pub(super) fn conditional() {
    for count in [2, 253] {
        let fixture = fixture::existing(count);
        let input = reader::input(fixture.reader_inputs());
        let request = request(&input, 1000);
        let r::Query::Handle {
            membership: r::Membership::Incomplete(missing),
            ..
        } = r::read(&input).unwrap().query(handle(1000))
        else {
            panic!("conditional cut remains incomplete")
        };
        assert_eq!(missing.len(), count);
        assert_eq!(
            e::reading_report(&input, &request),
            Err(e::Refusal::Incomplete {
                handle: handle(1000),
                missing: missing.clone()
            })
        );
        let mut observing = request;
        observing.completeness = e::Completeness::Report;
        observing.family = e::ReadingFamily::Reading;
        let report = e::reading_report(&input, &observing).unwrap();
        assert_eq!(e::validate_report(&input, &observing, &report), Ok(()));
        let e::Body::Reading(reading) = &report.body else {
            panic!("reading body")
        };
        assert_eq!(
            reading.outcome,
            e::Outcome::Incomplete {
                handle: handle(1000),
                missing
            }
        );
        let mut forged = report.clone();
        let e::Body::Reading(reading) = &mut forged.body else {
            panic!("reading body")
        };
        reading.outcome = e::Outcome::Complete(e::CompleteReading {
            scope: e::Scope::HandleClosure,
            finality: e::Finality::Nonfinal,
            membership: BTreeSet::new(),
        });
        assert_eq!(
            e::validate_report(&input, &observing, &forged),
            Err(e::Refusal::OutcomeMismatch)
        );
    }
}

pub(super) fn body(report: &e::Report) -> &e::AcceptanceBody {
    let e::Body::Acceptance(body) = &report.body else {
        panic!("acceptance body")
    };
    body
}
pub(super) fn body_mut(report: &mut e::Report) -> &mut e::AcceptanceBody {
    let e::Body::Acceptance(body) = &mut report.body else {
        panic!("acceptance body")
    };
    body
}
pub(super) fn accepted(report: &e::Report) -> &e::AcceptedReading {
    let e::Outcome::Complete(value) = &body(report).reading.outcome else {
        panic!("complete acceptance")
    };
    value
}
pub(super) fn accepted_mut(report: &mut e::Report) -> &mut e::AcceptedReading {
    let e::Outcome::Complete(value) = &mut body_mut(report).reading.outcome else {
        panic!("complete acceptance")
    };
    value
}
