//! Review controls use literal claims and synthetic untrusted evidence, not new histories.
use super::{fixture, reader, reading};
use std::collections::{BTreeMap, BTreeSet};
use vectors::{keyed_evidence as e, keyed_history_reader as r};

pub(super) fn controls() {
    let fixture = fixture::build(fixture::graph("source"));
    let input = reader::input(fixture.reader_inputs());
    let request = reading::request(&input, 10);
    literals(&input, &request);
    precedence(&input, &request);
    non_handle(&input, &request);
    let mut corrupt = input;
    corrupt.bytes.get_mut(&10).unwrap()[0] ^= 1;
    let error = r::read(&corrupt).unwrap_err();
    let mut reporting = request;
    reporting.completeness = e::Completeness::Report;
    assert_eq!(
        e::reading_report(&corrupt, &reporting),
        Err(e::Refusal::Reader(error))
    );
}
fn literals(input: &r::Input, request: &e::Request) {
    let mut input = input.clone();
    input.boundary_premises.insert(10);
    let report = e::reading_report(&input, request).unwrap();
    // Independent literal frame from fixture inputs, never EvidenceReferences::from.
    let expected = e::EvidenceReferences {
        events: BTreeMap::from([(10, input.bytes[&10].clone())]),
        association: vec![(
            10,
            input
                .association
                .iter()
                .find(|(id, _)| *id == 10)
                .unwrap()
                .1,
        )],
        retrieved: input.retrieved.clone(),
        assertions: input.assertions.clone(),
        boundary_premises: BTreeSet::from([10]),
        keys: BTreeSet::from([r::Key([42; 32])]),
    };
    assert_eq!(report.header.evidence, Some(expected));
    let decoded: elements::Transaction = elements::encode::deserialize(&input.bytes[&10]).unwrap();
    assert_eq!(decoded.input.len(), 1);
    let parent = decoded.input[0].previous_output;
    let mut expected = Vec::new();
    for (fact, source) in [
        (
            r::ParentFact::Inclusion,
            "hypothetical valid-history inclusion",
        ),
        (
            r::ParentFact::Unspentness,
            "external per-input observation still owed",
        ),
        (r::ParentFact::Retrieval, "hypothetical finite retrieval"),
        (r::ParentFact::Finality, "external permanence still owed"),
    ] {
        expected.push(e::Assumption::Parent {
            obligation: r::Obligation {
                event: 10,
                input: 0,
                address: r::Address(parent.txid.to_byte_array()),
                position: parent.vout,
                fact,
            },
            source: e::EvidenceSource(source.into()),
        });
    }
    expected.push(e::Assumption::NoUnsuppliedKeyHandle {
        supplied_keys: BTreeSet::from([r::Key([42; 32])]),
        inclusion_source: e::EvidenceSource("hypothetical valid-history inclusion".into()),
    });
    expected.push(e::Assumption::Finality(e::Finality::Nonfinal));
    assert_eq!(report.header.assumptions, Some(expected));
}
fn non_handle(input: &r::Input, request: &e::Request) {
    let mut observing = request.clone();
    observing.completeness = e::Completeness::Report;
    observing.family = e::ReadingFamily::Reading;
    observing.handle.output.position = 1;
    let report = e::reading_report(input, &observing).unwrap();
    let e::Body::Reading(body) = &report.body else {
        panic!("reading body")
    };
    assert_eq!(body.outcome, e::Outcome::NotHandle(observing.handle));
    assert_eq!(
        report.header.quantities[&e::Dimension::Records].observed,
        e::Measurement::Absent(e::Absence::NotObservedHere)
    );
    assert_eq!(e::validate_report(input, &observing, &report), Ok(()));
    observing.completeness = e::Completeness::Required;
    assert_eq!(
        e::reading_report(input, &observing),
        Err(e::Refusal::Query(r::Query::NotHandle))
    );
}
fn precedence(input: &r::Input, request: &e::Request) {
    let mut missing = input.clone();
    missing.retrieved.clear();
    let error = e::reading_report(&missing, request).unwrap_err();
    assert!(matches!(error, e::Refusal::Incomplete { .. }));
    let e::Refusal::Incomplete { missing: own, .. } = &error else {
        panic!("queried closure is incomplete")
    };
    let mut whole_cut = request.clone();
    whole_cut.scope = e::Scope::WholeHistory;
    assert!(e::reading_report(input, &whole_cut).is_ok());
    assert_eq!(
        e::reading_report(&missing, &whole_cut),
        Err(e::Refusal::WholeHistory {
            handles: BTreeMap::from([(reading::handle(10), own.clone())]),
            events: BTreeMap::new(),
        })
    );
    let mut required = request.clone();
    required.finality = e::FinalityRequest::Required;
    assert_eq!(
        e::reading_report(input, &required),
        Err(e::Refusal::FinalityMissing)
    );
    assert_eq!(e::reading_report(&missing, &required), Err(error.clone()));
    let mut final_report = required.clone();
    final_report.completeness = e::Completeness::Report;
    assert_eq!(
        e::reading_report(&missing, &final_report),
        Err(error.clone())
    );
    let mut form = reading::declaration(request);
    form.deployment.0.push('x');
    required.declaration = Some(e::DeclarationOffer::Hypothetical(
        e::HypotheticalDeclaration(form),
    ));
    assert_eq!(
        e::reading_report(input, &required),
        Err(e::Refusal::FinalityHypothetical)
    );
    assert_eq!(e::reading_report(&missing, &required), Err(error));
    required.finality = e::FinalityRequest::NotRequested;
    assert_eq!(
        e::reading_report(input, &required),
        Err(e::Refusal::FinalityDeployment)
    );
    whole(input, request);
    let mut bad_rule = request.clone();
    bad_rule.rule = e::Rule::Changing {
        record: reading::handle(10),
        enabled_epoch: e::Epoch(1),
        disabled_epoch: e::Epoch(1),
    };
    assert_eq!(
        e::reading_report(input, &bad_rule),
        Err(e::Refusal::SameEpoch)
    );
    bad_rule.finality = e::FinalityRequest::Required;
    assert_eq!(
        e::reading_report(input, &bad_rule),
        Err(e::Refusal::SameEpoch)
    );
}
fn whole(input: &r::Input, request: &e::Request) {
    let keyed = reading::extra_event(input, true);
    let ordinary = reading::extra_event(input, false);
    let mut both = keyed.clone();
    both.bytes.insert(100, ordinary.bytes[&99].clone());
    both.association.push((
        100,
        ordinary
            .association
            .iter()
            .find(|(id, _)| *id == 99)
            .unwrap()
            .1,
    ));
    both.assertions
        .inclusion
        .extend(&ordinary.assertions.inclusion);
    both.assertions
        .retrieval
        .extend(&ordinary.assertions.retrieval);
    let mut whole = request.clone();
    whole.scope = e::Scope::WholeHistory;
    let handles = BTreeMap::from([(
        reading::handle(99),
        BTreeSet::from([r::Missing::Parent {
            event: 99,
            input: 0,
            address: r::Address([98; 32]),
            position: 1,
        }]),
    )]);
    // Both would otherwise double-spend the missing outpoint: distinct missing parent
    // is required for an evidence-completeness subject rather than consistency failure.
    let mut decoded = transaction::TargetTransaction::decode(&both.bytes[&100]).unwrap();
    decoded = transaction::TargetTransaction::new(
        2,
        vec![transaction::TargetInput::new(
            transaction::Outpoint::new(transaction::Txid::from_internal([97; 32]), 1).unwrap(),
            77,
        )],
        decoded.outputs().to_vec(),
        0,
        vec![transaction::InputWitness::new(vec![])],
    )
    .unwrap();
    let bytes = decoded.encode();
    let address = r::Address(target_elements_conformance::constructor::tagged::sha256(
        &target_elements_conformance::constructor::tagged::sha256(
            &decoded.encode_without_witness(),
        ),
    ));
    both.bytes.insert(100, bytes);
    both.association.retain(|(id, _)| *id != 100);
    both.association.push((100, address));
    both.assertions.inclusion.insert(address);
    both.assertions.retrieval.insert(address);
    let events = BTreeMap::from([(
        100,
        BTreeSet::from([r::Missing::Parent {
            event: 100,
            input: 0,
            address: r::Address([97; 32]),
            position: 1,
        }]),
    )]);
    assert!(e::reading_report(input, &whole).is_ok());
    assert_eq!(
        e::reading_report(&keyed, &whole),
        Err(e::Refusal::WholeHistory {
            handles: handles.clone(),
            events: BTreeMap::new()
        })
    );
    let mut just_ordinary = both.clone();
    just_ordinary.bytes.remove(&99);
    just_ordinary.association.retain(|(id, _)| *id != 99);
    assert_eq!(
        e::reading_report(&just_ordinary, &whole),
        Err(e::Refusal::WholeHistory {
            handles: BTreeMap::new(),
            events: events.clone()
        })
    );
    let mut queried_missing = whole.clone();
    queried_missing.handle = reading::handle(99);
    assert_eq!(
        e::reading_report(&both, &queried_missing),
        Err(e::Refusal::WholeHistory {
            handles: handles.clone(),
            events: events.clone(),
        })
    );
    assert_eq!(
        e::reading_report(&both, &whole),
        Err(e::Refusal::WholeHistory { handles, events })
    );
    // An unused association is not a supplied event.
    let mut association_only = input.clone();
    association_only
        .association
        .push((700, r::Address([70; 32])));
    assert!(e::reading_report(&association_only, &whole).is_ok());
}
