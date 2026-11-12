//! Offline history controls. All child processes are bounded and timed.
mod families;
mod family_controls;
mod family_trace;
mod fixture;
#[path = "../../src/keyed_history_model.rs"]
mod model;
mod reader;
mod reader_boundary;
mod reader_review;
mod reading;
mod reading_reports;
mod reading_review;
mod reading_trace;
// Let cargo fmt visit the independently compiled harness without linking it.
#[cfg(any())]
mod model_checks;

use std::io::Write;
use std::process::{Command, Output};

fn bounded(command: &mut Command, description: &str) -> Output {
    let start = std::time::Instant::now();
    let result = command.output().unwrap();
    writeln!(
        std::io::stdout().lock(),
        "keyed history {description} wall={:.6}s exit={}",
        start.elapsed().as_secs_f64(),
        result.status
    )
    .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    // Preserve the child transcript without making its nested harness markers
    // look like this process's own exactly-one-test evidence.
    for line in String::from_utf8_lossy(&result.stdout).lines() {
        writeln!(std::io::stdout().lock(), "history child: {line}").unwrap();
    }
    result
}

fn child(name: &str, seconds: &str) -> Command {
    let mut command = Command::new("/usr/bin/timeout");
    command
        .args(["--kill-after=1", seconds])
        .arg(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", name, "--nocapture"]);
    command
}

fn one_test(result: &Output) {
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| *line == "running 1 test")
            .count(),
        1
    );
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.starts_with("test result: ok. 1 passed; 0 failed;"))
            .count(),
        1
    );
}

#[test]
fn keyed_histories_have_bounded_public_subjects() {
    one_test(&bounded(
        &mut child("keyed_history::history_family_child", "60"),
        "family timeout=60s",
    ));
}

#[test]
#[ignore = "bounded family parent launches this exact child"]
fn history_family_child() {
    for subject in [
        "source",
        "single",
        "ordinary-edge",
        "multi",
        "path",
        "two-keys",
        "wide",
        "later",
        "existing-two",
        "existing-253",
        "association",
        "reader-negatives",
        "reader-review",
        "reading-review",
        "families-constants",
        "families-source",
        "families-single",
        "families-ordinary-edge",
        "families-path",
        "families-two-keys",
        "families-later",
        "families-multi",
        "families-wide",
        "families-cut-two",
        "families-cut-253",
        "reading-rules",
        "reading-premises",
        "reading-reports",
        "reading-conditional",
    ] {
        let mut command = child("keyed_history::history_subject_child", "10");
        command.env("TRIPOD_KEYED_HISTORY_SUBJECT", subject);
        one_test(&bounded(
            &mut command,
            &format!("subject={subject} timeout=10s"),
        ));
    }
}

#[test]
#[ignore = "bounded subject parent launches this exact child"]
fn history_subject_child() {
    let subject = std::env::var("TRIPOD_KEYED_HISTORY_SUBJECT").unwrap();
    if subject == "families-constants" {
        family_controls::constants();
        return;
    }
    if let Some(name) = subject.strip_prefix("families-") {
        families::subject(name);
        return;
    }
    match subject.as_str() {
        "reading-review" => {
            reading_review::controls();
            return;
        }
        "reading-rules" => {
            reading::rules();
            return;
        }
        "reading-premises" => {
            reading::premises();
            return;
        }
        "reading-reports" => {
            reading_reports::controls();
            return;
        }
        "reading-conditional" => {
            reading::conditional();
            return;
        }
        _ => (),
    }
    if subject == "reader-review" {
        reader_review::controls();
        return;
    }
    if subject == "reader-negatives" {
        reader::negatives();
        return;
    }
    if subject.starts_with("existing-") {
        let count = if subject == "existing-two" { 2 } else { 253 };
        let fixture = fixture::existing(count);
        assert_eq!(
            fixture::check_association(&fixture, &fixture.association),
            Ok(())
        );
        let literal: std::collections::BTreeSet<_> = (0..count)
            .map(|i| fixture::handle(42, u32::try_from(i).unwrap()))
            .collect();
        assert_eq!(
            fixture.expected.records(fixture::handle(42, 1000)),
            Some(&literal)
        );
        assert_invariants(&fixture);
        for id in 0..u32::try_from(count).unwrap() {
            assert_eq!(
                fixture.expected.query(model::Handle {
                    key: [42; 32],
                    output: model::Output {
                        event: id,
                        position: 1
                    }
                }),
                model::OutputStatus::NotHandle
            );
        }
        reader::compare(&fixture);
        let reader = fixture.reader_inputs();
        assert_eq!(reader.boundary_premises.len(), count);
        assert_eq!(reader.bytes.len(), count + 1);
        return;
    }
    let name = if subject == "association" {
        "multi"
    } else {
        &subject
    };
    let fixture = fixture::build(fixture::graph(name));
    assert_eq!(
        fixture::check_association(&fixture, &fixture.association),
        Ok(())
    );
    reader::compare(&fixture);
    assert_expected(name, &fixture);
    assert_invariants(&fixture);
    assert_eq!(fixture.reader_inputs().bytes, fixture.bytes);
    if subject == "association" {
        association_subjects(fixture);
    }
}

fn assert_expected(name: &str, fixture: &fixture::Fixture) {
    use std::collections::BTreeSet;
    let h = fixture::handle;
    let expected = &fixture.expected;
    let singleton = BTreeSet::from([h(42, 10)]);
    assert_eq!(expected.records(h(42, 10)), Some(&singleton));
    match name {
        "source" | "later" => assert_eq!(expected.live(), &BTreeSet::from([h(42, 10)])),
        "single" => {
            assert_eq!(expected.records(h(42, 20)), Some(&singleton));
            assert_eq!(expected.successor(h(42, 10)), Some(h(42, 20)));
        }
        "multi" | "path" => {
            let end = if name == "multi" { 20 } else { 40 };
            let union = BTreeSet::from([h(42, 10), h(42, 11)]);
            assert_eq!(expected.records(h(42, end)), Some(&union));
            assert!(singleton.is_subset(expected.records(h(42, end)).unwrap()));
            assert!(!expected.records(h(42, end)).unwrap().contains(&h(42, end)));
            if name == "path" {
                assert_eq!(expected.records(h(42, 20)), Some(&singleton));
                assert_eq!(expected.records(h(42, 30)), Some(&union));
                for (from, to) in [(10, 20), (20, 30), (11, 30), (30, 40)] {
                    assert_eq!(expected.successor(h(42, from)), Some(h(42, to)));
                }
            }
            assert_eq!(expected.live(), &BTreeSet::from([h(42, end)]));
        }
        "ordinary-edge" => {
            assert_eq!(
                expected.records(h(42, 20)),
                Some(&BTreeSet::from([h(42, 20)]))
            );
            assert_eq!(expected.live(), &BTreeSet::from([h(42, 10), h(42, 20)]));
            let decoded = transaction::TargetTransaction::decode(&fixture.bytes[&20]).unwrap();
            assert_eq!(decoded.inputs()[0].outpoint().index(), 1);
            assert_eq!(
                *decoded.inputs()[0].outpoint().txid().internal(),
                fixture::replay::txid(&fixture.bytes[&10])
            );
        }
        "two-keys" => {
            assert_eq!(
                expected.records(h(43, 11)),
                Some(&BTreeSet::from([h(43, 11)]))
            );
            assert_eq!(expected.live(), &BTreeSet::from([h(42, 20), h(43, 21)]));
            assert_eq!(expected.records(h(42, 20)), Some(&singleton));
            assert_eq!(
                expected.records(h(43, 21)),
                Some(&BTreeSet::from([h(43, 11)]))
            );
            assert!(
                expected
                    .records(h(42, 20))
                    .unwrap()
                    .is_disjoint(expected.records(h(43, 21)).unwrap())
            );
        }
        "wide" => {
            let records = (10..263).map(|id| h(42, id)).collect::<BTreeSet<_>>();
            assert_eq!(expected.records(h(42, 1000)), Some(&records));
            assert_eq!(expected.live(), &BTreeSet::from([h(42, 1000)]));
        }
        _ => panic!("known history"),
    }
    if name == "later" {
        let later = model::Handle {
            key: [42; 32],
            output: model::Output {
                event: 10,
                position: 1,
            },
        };
        assert_eq!(expected.records(later), None);
        let decoded = transaction::TargetTransaction::decode(&fixture.bytes[&10]).unwrap();
        assert_eq!(
            decoded.outputs()[0].program(),
            decoded.outputs()[1].program()
        );
        let mut spent = fixture.graph.clone();
        spent.push(model::Event {
            id: 20,
            inputs: vec![later.output],
            outputs: vec![Some([42; 32])],
        });
        let refusal = model::analyze(&spent).unwrap_err();
        assert_eq!(refusal, model::Refusal::LaterOutputConsumed(later.output));
        assert_eq!(refusal.boundary(), model::Boundary::BornFirst);
    }
    let mut permutation = fixture.graph.clone();
    permutation.reverse();
    assert_eq!(model::analyze(&permutation), Ok(fixture.expected.clone()));
}

fn assert_invariants(fixture: &fixture::Fixture) {
    use std::collections::{BTreeMap, BTreeSet};
    let index: BTreeMap<_, _> = fixture
        .graph
        .iter()
        .map(|event| (event.id, event))
        .collect();
    let mut sources = BTreeSet::new();
    for event in &fixture.graph {
        let Some(Some(key)) = event.outputs.first() else {
            continue;
        };
        let handle = model::Handle {
            key: *key,
            output: model::Output {
                event: event.id,
                position: 0,
            },
        };
        let members: Vec<_> = event
            .inputs
            .iter()
            .filter_map(|input| {
                let parent = index[&input.event];
                if input.position != 0 {
                    return None;
                }
                parent.outputs[0].map(|key| model::Handle {
                    key,
                    output: *input,
                })
            })
            .collect();
        if members.is_empty() {
            sources.insert(handle);
        }
        for member in members {
            assert_eq!(fixture.expected.successor(member), Some(handle));
            assert!(
                fixture
                    .expected
                    .records(member)
                    .unwrap()
                    .is_subset(fixture.expected.records(handle).unwrap())
            );
        }
    }
    let mut union = BTreeSet::new();
    for handle in fixture.expected.live() {
        let records = fixture.expected.records(*handle).unwrap();
        assert!(union.is_disjoint(records));
        union.extend(records);
    }
    assert_eq!(union, sources);
}

fn association_subjects(mut fixture: fixture::Fixture) {
    use fixture::{AssociationRefusal as R, check_association};
    let accepted = fixture.association.clone();
    assert_eq!(check_association(&fixture, &accepted), Ok(()));
    let mut subject = accepted.clone();
    subject.push(subject[0]);
    assert_eq!(
        check_association(&fixture, &subject),
        Err(R::DuplicateName(10))
    );
    subject.clone_from(&accepted);
    subject[1].1 = subject[0].1;
    assert_eq!(
        check_association(&fixture, &subject),
        Err(R::DuplicateAddress)
    );
    subject.clone_from(&accepted);
    subject[0].1[0] ^= 1;
    assert_eq!(
        check_association(&fixture, &subject),
        Err(R::AddressMismatch(10))
    );
    subject.clone_from(&accepted);
    subject[0].0 = 99;
    assert_eq!(
        check_association(&fixture, &subject),
        Err(R::UnknownName(99))
    );
    assert_eq!(
        check_association(&fixture, &accepted[1..]),
        Err(R::MissingName(10))
    );

    fixture.graph[2].inputs.reverse();
    assert_eq!(
        check_association(&fixture, &accepted),
        Err(R::InputEdges(20))
    );
    fixture.graph[2].inputs.reverse();
    fixture.graph[0].outputs[1] = Some([42; 32]);
    assert_eq!(
        check_association(&fixture, &accepted),
        Err(R::OutputKeys(10))
    );
    fixture.graph[0].outputs[1] = None;
    let original = fixture.bytes[&10].clone();
    fixture.bytes.insert(10, vec![0]);
    assert_eq!(
        check_association(&fixture, &accepted),
        Err(R::InvalidBytes(10))
    );
    fixture.bytes.remove(&10);
    assert_eq!(
        check_association(&fixture, &accepted),
        Err(R::MissingBytes(10))
    );
    let mut nonminimal = original.clone();
    assert_eq!(nonminimal[5], 1);
    nonminimal.splice(5..6, [253, 1, 0]);
    assert!(transaction::TargetTransaction::decode(&nonminimal).is_err());
    fixture.bytes.insert(10, nonminimal);
    assert_eq!(
        check_association(&fixture, &accepted),
        Err(R::InvalidBytes(10))
    );
    fixture.bytes.insert(10, original);
    let mut permuted = accepted;
    permuted.reverse();
    assert_eq!(check_association(&fixture, &permuted), Ok(()));
    let reader = fixture.reader_inputs();
    for packet in fixture.packets.values_mut() {
        packet.source = !packet.source;
        for input in &mut packet.inputs {
            input.member = !input.member;
        }
    }
    assert_eq!(fixture.reader_inputs(), reader);
    let declaration = include_str!("fixture.rs")
        .split("struct ReaderFixture {")
        .nth(1)
        .unwrap()
        .split("\n}")
        .next()
        .unwrap();
    for forbidden in ["expected", "graph", "source", "member", "Packet", "History"] {
        assert!(!declaration.contains(forbidden));
    }
    let mut ordinary = fixture::build(fixture::graph("ordinary-edge"));
    assert_eq!(check_association(&ordinary, &ordinary.association), Ok(()));
    ordinary.graph[1].inputs[0].position = 0;
    assert_eq!(
        check_association(&ordinary, &ordinary.association),
        Err(R::InputEdges(20))
    );
}

fn permitted_model_source(source: &str) -> bool {
    // Whole-line comments only. Reject literals and inline comments rather than
    // allowing a literal to hide the rest of a line from this lexical guard.
    let code = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    if code.contains('"') || code.contains("//") || code.contains("/*") {
        return false;
    }
    !code
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|token| {
            matches!(
                token,
                "cfg"
                    | "cfg_attr"
                    | "mod"
                    | "path"
                    | "architecture"
                    | "realization"
                    | "compiler"
                    | "linker"
                    | "transaction"
                    | "tapscript"
                    | "vectors"
                    | "target_elements"
                    | "target_elements_conformance"
                    | "reader"
                    | "builder"
                    | "crate"
                    | "super"
                    | "extern"
                    | "include"
                    | "include_str"
                    | "include_bytes"
                    | "macro_rules"
            )
        })
}

#[test]
fn keyed_history_oracle_import_boundary() {
    let source = include_str!("../../src/keyed_history_model.rs");
    assert!(permitted_model_source(source));
    for mutant in [
        "use transaction::TargetTransaction;",
        "use compiler as alias;",
        "type Alias = linker::keyed_bundle::LinkedKeyedBundle;",
        "fn f() { realization::evaluate(); }",
        "use super::Unknown;",
        "use tapscript as hidden;",
        "#[cfg(test)] use elements::Transaction;",
        "use architecture as hidden;",
        "let marker = \"//\"; use elements::Transaction;",
        "#[cfg_attr(test, derive(Clone))] struct Extra;",
        "mod extra {}",
        "#[path(0)] struct Extra;",
    ] {
        assert!(
            !permitted_model_source(&format!("{source}\n{mutant}")),
            "{mutant}"
        );
    }
    let directory = tempfile::tempdir().unwrap();
    let executable = directory.path().join("model-alone");
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/keyed_history/model_checks.rs");
    let mut compiler = Command::new("/usr/bin/timeout");
    compiler
        .args([
            "--kill-after=1",
            "60",
            "rustc",
            "--edition=2024",
            "-Dwarnings",
        ])
        .arg(source)
        .arg("-o")
        .arg(&executable);
    bounded(
        &mut compiler,
        "std-only compile timeout=60s no --extern or dependency search path",
    );
    let mut run = Command::new("/usr/bin/timeout");
    run.args(["--kill-after=1", "10"]).arg(executable);
    bounded(&mut run, "std-only model controls timeout=10s");
}
