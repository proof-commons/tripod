//! Constant-source and report-coordinate controls, not new construction recipes.
use super::{families, fixture, reader, reading};
use vectors::{keyed_evidence as e, keyed_history_reader as r};

pub(super) fn constants() {
    for (length, prefix) in [(252, vec![252]), (253, vec![253, 253, 0])] {
        assert_eq!(transaction::compact_size(length), prefix);
        let witness =
            transaction::InputWitness::new(vec![vec![7; usize::try_from(length).unwrap()]]);
        let cost = vectors::keyed_cost::witness_cost(&witness);
        assert_eq!(
            cost.script_witness_bytes,
            1 + prefix.len() + usize::try_from(length).unwrap()
        );
        assert_eq!(cost.input_witness_bytes, cost.script_witness_bytes + 3);
    }
    assert_eq!(tapscript::keyed_resources::keyed_witness_bytes(1), Ok(88));
    assert!(tapscript::keyed_resources::keyed_witness_bytes(u64::MAX).is_err());
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    assert!(tapscript::keyed_resources::keyed_transaction_weight(&target, 1, u64::MAX, 0).is_err());
    for schedule in [
        tapscript::keyed_resources::KeyedSchedule::Birth,
        tapscript::keyed_resources::KeyedSchedule::Continuation,
        tapscript::keyed_resources::KeyedSchedule::Leaf,
    ] {
        let row = tapscript::keyed_resources::keyed_resource_projection(&target, schedule).unwrap();
        assert!(
            target_elements::ResourceDimension::ALL
                .iter()
                .all(|dimension| row.dimensions.contains_key(dimension))
        );
    }
    let fixture = fixture::build(fixture::graph("multi"));
    let input = reader::input(fixture.reader_inputs());
    let c = e::prepare_families(&input).unwrap();
    let q = reading::request(&input, 20);
    let good = e::family_report(&c, e::Family::Resources, &q).unwrap();
    let mut bad = good.clone();
    let e::Observation::OfflineResources { archives } = &mut bad.header.observation else {
        panic!()
    };
    assert_eq!(c.archive_rows().unwrap().len(), 16);
    if let Some(row) = archives.first_mut() {
        row.digest.0[0] ^= 1;
    } else {
        archives.push(e::ArchivedObservation {
            archive: "invented".into(),
            step: "invented".into(),
            digest: e::ArchiveDigest([0; 32]),
        });
    }
    families::refuse(
        &c,
        e::Family::Resources,
        &q,
        &bad,
        e::FamilyCondition::Framing(e::Refusal::ObservationMismatch),
    );
    let mut bad = good;
    let e::Body::Resources(body) = &mut bad.body else {
        panic!()
    };
    body.schedules[0].member.key = r::Key([9; 32]);
    families::refuse(
        &c,
        e::Family::Resources,
        &q,
        &bad,
        e::FamilyCondition::ScheduleIdentity,
    );
    archive_rows(&c);
    overrides(&c, &q);
    trace_rows(&c, &q);
    observed_refusals(&input);
    scriptsig_size(&input);
}
fn trace_rows(c: &e::FamilyContext, q: &e::Request) {
    let table = include_str!("../../../../plans/drafts/wave1-bite-1-5-plan.md");
    for family in [
        e::Family::Birth,
        e::Family::Continuation,
        e::Family::Recurrence,
        e::Family::Reading,
        e::Family::Acceptance,
        e::Family::Resources,
    ] {
        let report = e::family_report(c, family, q).unwrap();
        let trace = match &report.body {
            e::Body::Birth(b) => &b.trace,
            e::Body::Continuation(b) => &b.trace,
            e::Body::Recurrence(b) => &b.trace,
            e::Body::Reading(b) => &b.trace,
            e::Body::Acceptance(b) => &b.reading.trace,
            e::Body::Resources(b) => &b.trace,
            _ => panic!(),
        };
        let name = match family {
            e::Family::Birth => "birth",
            e::Family::Continuation => "continuation",
            e::Family::Recurrence => "recurrence",
            e::Family::Reading => "reading",
            e::Family::Acceptance => "acceptance",
            e::Family::Resources => "resources",
        };
        let expected: Vec<_> = (1..8)
            .filter(|n| {
                let row = table
                    .lines()
                    .find(|r| r.starts_with(&format!("| {n}. ")))
                    .unwrap();
                let column = row.split('|').nth(3).unwrap().to_lowercase();
                column.contains(name)
            })
            .map(e::RequirementId)
            .collect();
        assert_eq!(trace.requirements, expected);
        check_trace(table, trace);
        let header_trace = report.header.trace.as_ref().unwrap();
        assert_eq!(header_trace.requirements, vec![e::RequirementId(8)]);
        check_trace(table, header_trace);
        let mut bad = report.clone();
        let trace = match &mut bad.body {
            e::Body::Birth(b) => &mut b.trace,
            e::Body::Continuation(b) => &mut b.trace,
            e::Body::Recurrence(b) => &mut b.trace,
            e::Body::Reading(b) => &mut b.trace,
            e::Body::Acceptance(b) => &mut b.reading.trace,
            e::Body::Resources(b) => &mut b.trace,
            _ => panic!(),
        };
        trace.paper.push(e::PaperId("invented-relation".into()));
        let condition = if matches!(family, e::Family::Reading | e::Family::Acceptance) {
            e::FamilyCondition::Framing(e::Refusal::TraceMismatch)
        } else {
            e::FamilyCondition::Trace
        };
        families::refuse(c, family, q, &bad, condition);
    }
}
fn check_trace(table: &str, trace: &e::Trace) {
    let mut expected = std::collections::BTreeSet::new();
    for requirement in &trace.requirements {
        let row = table
            .lines()
            .find(|row| row.starts_with(&format!("| {}. ", requirement.0)))
            .unwrap();
        for part in row.split("[A-").skip(1) {
            expected.insert(part.split(']').next().unwrap().to_owned());
        }
    }
    assert_eq!(
        trace
            .paper
            .iter()
            .map(|p| p.0.clone())
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
    for label in expected {
        let source = if label == "sec:kpc:limitations" {
            include_str!("../../../../papers/lineage/sections/10_limitations.tex")
        } else if label.contains("kpc-interface") || label == "sec:kpc:interface" {
            include_str!("../../../../papers/lineage/sections/01_interface.tex")
        } else if label.contains("kpc-model") {
            include_str!("../../../../papers/lineage/sections/02_model.tex")
        } else if label.contains("kpc-operations") {
            include_str!("../../../../papers/lineage/sections/03_operations.tex")
        } else {
            include_str!("../../../../papers/lineage/sections/04_invariants.tex")
        };
        assert!(source.contains(&format!("\\label{{{label}}}")), "{label}");
    }
}

fn observed_refusals(input: &r::Input) {
    let mut bad = input.clone();
    let mut tx: elements::Transaction = elements::encode::deserialize(&bad.bytes[&20]).unwrap();
    let unchanged = tx.input[0].witness.clone();
    let mut stack = tx.input[1].witness.script_witness.to_vec();
    stack[0][1] ^= 1;
    tx.input[1].witness.script_witness = stack.into();
    assert_eq!(tx.input[0].witness, unchanged);
    bad.bytes.insert(20, elements::encode::serialize(&tx));
    preparation_refusal(
        &bad,
        r::Refusal::UncommittedLeaf {
            event: 20,
            input: 1,
        },
        r::Boundary::CommitmentAuthentication,
    );
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let other = vectors::keyed_constructor::construct_keyed(
        &target,
        tapscript::keyed_constructor::KeyedSemanticKey::new([43; 32]),
        &vectors::keyed_constructor::PublicKeyedCurve,
    )
    .unwrap();
    let mut bad = input.clone();
    bad.keys.insert(r::Key([43; 32]));
    let mut parent: elements::Transaction = elements::encode::deserialize(&bad.bytes[&11]).unwrap();
    parent.output[0].script_pubkey = elements::Script::from(other.output_program);
    replace(&mut bad, 11, &parent);
    let mut tx: elements::Transaction = elements::encode::deserialize(&bad.bytes[&20]).unwrap();
    tx.input[1].previous_output.txid = parent.txid();
    replace(&mut bad, 20, &tx);
    preparation_refusal(
        &bad,
        r::Refusal::WrongSuccessor {
            event: 20,
            input: 1,
        },
        r::Boundary::Continuation,
    );
    let fixture = fixture::existing(2);
    let mut bad = reader::input(fixture.reader_inputs());
    assert!(e::prepare_families(&bad).is_ok());
    let mut tx: elements::Transaction = elements::encode::deserialize(&bad.bytes[&1000]).unwrap();
    tx.input[1].previous_output.vout = 1;
    let address = r::Address(tx.input[1].previous_output.txid.to_byte_array());
    replace(&mut bad, 1000, &tx);
    preparation_refusal(
        &bad,
        r::Refusal::LaterBirth {
            event: 1000,
            input: 1,
            address,
            position: 1,
        },
        r::Boundary::BornFirst,
    );
}
fn preparation_refusal(input: &r::Input, expected: r::Refusal, boundary: r::Boundary) {
    let error = e::prepare_families(input).unwrap_err();
    let observed = error.observation_boundary();
    assert_eq!(error.family, None);
    assert_eq!(error.boundary(), e::FamilyBoundary::Preparation);
    assert_eq!(
        (error.condition, observed),
        (
            e::FamilyCondition::Reader(expected),
            Some(e::FamilyObservationBoundary::Reader(boundary))
        )
    );
}
fn replace(input: &mut r::Input, event: u32, tx: &elements::Transaction) {
    let row = input
        .association
        .iter_mut()
        .find(|(id, _)| *id == event)
        .unwrap();
    let old = row.1;
    row.1 = r::Address(tx.txid().to_byte_array());
    input.assertions.inclusion.remove(&old);
    input.assertions.retrieval.remove(&old);
    input.assertions.inclusion.insert(row.1);
    input.assertions.retrieval.insert(row.1);
    input.bytes.insert(event, elements::encode::serialize(tx));
}

pub(super) fn rules(c: &e::FamilyContext, q: &e::Request) {
    let mut request = q.clone();
    request.rule = e::Rule::Fixed {
        asset: e::Asset([3; 32]),
        threshold: e::Amount(1011),
    };
    let fixed = e::family_report(c, e::Family::Acceptance, &request).unwrap();
    e::validate_acceptance(c, &request, &fixed).unwrap();
    let e::Body::Acceptance(body) = &fixed.body else {
        panic!()
    };
    let e::Outcome::Complete(row) = &body.reading.outcome else {
        panic!()
    };
    assert_eq!(
        row.accepted,
        std::collections::BTreeSet::from([reading::handle(11)])
    );
    assert_eq!(
        row.reading.membership,
        std::collections::BTreeSet::from([reading::handle(10), reading::handle(11)])
    );
    request.rule = e::Rule::Changing {
        record: reading::handle(11),
        enabled_epoch: e::Epoch(1),
        disabled_epoch: e::Epoch(2),
    };
    for (epoch, expected) in [
        (e::Epoch(1), vec![reading::handle(10), reading::handle(11)]),
        (e::Epoch(2), vec![reading::handle(10)]),
    ] {
        request.observed_epoch = epoch;
        let report = e::family_report(c, e::Family::Acceptance, &request).unwrap();
        e::validate_acceptance(c, &request, &report).unwrap();
        let e::Body::Acceptance(body) = &report.body else {
            panic!()
        };
        let e::Outcome::Complete(row) = &body.reading.outcome else {
            panic!()
        };
        assert_eq!(row.accepted, expected.into_iter().collect());
        assert_eq!(
            row.reading.membership,
            std::collections::BTreeSet::from([reading::handle(10), reading::handle(11)])
        );
    }
}

fn scriptsig_size(input: &r::Input) {
    // Untrusted byte subjects: scriptSig is decidable before witness shape.
    assert!(r::read(input).is_ok());
    for wrong_shape in [false, true] {
        let mut bad = input.clone();
        let mut tx: elements::Transaction = elements::encode::deserialize(&bad.bytes[&20]).unwrap();
        let unchanged = tx.input[0].clone();
        tx.input[1].script_sig = elements::Script::from(vec![0]);
        if wrong_shape {
            tx.input[1].witness.script_witness = vec![vec![1]].into();
        }
        assert_eq!(tx.input[0], unchanged);
        replace(&mut bad, 20, &tx);
        let expected = r::Refusal::ScriptSig {
            event: 20,
            input: 1,
        };
        assert_eq!(r::read(&bad).unwrap_err(), expected);
        assert_eq!(expected.boundary(), r::Boundary::CommitmentAuthentication);
        let error = e::prepare_families(&bad).unwrap_err();
        assert_eq!(error.condition, e::FamilyCondition::Reader(expected));
        assert_eq!(error.family, None);
        assert_eq!(error.boundary(), e::FamilyBoundary::Preparation);
        assert_eq!(
            error.observation_boundary(),
            Some(e::FamilyObservationBoundary::Reader(
                r::Boundary::CommitmentAuthentication
            ))
        );
    }
}

fn archive_rows(c: &e::FamilyContext) {
    for row in c.archive_rows().unwrap() {
        assert_eq!(
            row.weight_matches,
            row.node_weight.map(|w| w == row.serialized_weight)
        );
        assert_eq!(row.reported.transaction_weight, row.node_weight);
        assert_eq!(row.reported.peak_stack_items, None);
        assert_eq!(row.reported.peak_altstack_items, None);
        assert_eq!(row.reported.maximum_element_bytes, None);
        assert_eq!(row.reported.validation_budget_used, None);
    }
}
fn overrides(c: &e::FamilyContext, q: &e::Request) {
    for family in [
        e::Family::Birth,
        e::Family::Continuation,
        e::Family::Resources,
    ] {
        let good = e::family_report(c, family, q).unwrap();
        let mut changed = q.clone();
        let mut declaration = reading::declaration(q);
        declaration.deployment = e::Deployment("unrelated".into());
        changed.declaration = Some(e::DeclarationOffer::Hypothetical(
            e::HypotheticalDeclaration(declaration),
        ));
        assert_eq!(e::family_report(c, family, &changed).unwrap(), good);
        assert_eq!(
            good.header.quantities[&e::Dimension::Records].observed,
            e::Measurement::Absent(e::Absence::NotObservedHere)
        );
    }
    let good = e::family_report(c, e::Family::Acceptance, q).unwrap();
    let e::Body::Acceptance(body) = &good.body else {
        panic!()
    };
    let e::Outcome::Complete(row) = &body.reading.outcome else {
        panic!()
    };
    for (event, value) in [(10, 1010), (11, 1011)] {
        let annotation = &row.annotations[&reading::handle(event)];
        assert_eq!(annotation.value, e::Value::Explicit(e::Amount(value)));
        assert_eq!(annotation.asset, e::AssetValue::Explicit(e::Asset([3; 32])));
        assert_eq!(annotation.attribution, e::Attribution::Unassigned);
        assert_eq!(annotation.creating_bytes, c.input().bytes[&event]);
    }
    assert_eq!(
        good.header.quantities[&e::Dimension::Records].observed,
        e::Measurement::Present {
            value: 2,
            unit: e::Unit::Records
        }
    );
}
