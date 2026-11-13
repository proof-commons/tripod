//! Public pipeline freeze subjects; bounded offline child processes only.
mod census;
mod controls;
mod envelope;
mod fixture;
mod freeze_history;
mod inventory;
mod prediction;

use crate::keyed_reproduction::fixture as prior;
use std::io::Write;
use std::process::{Command, Output};
use vectors::keyed_final_manifest::{SECOND_KEY, SECOND_KEY_PHRASE};

fn run(command: &mut Command, name: &str) -> Output {
    let start = std::time::Instant::now();
    let output = command.output().unwrap();
    let wall = start.elapsed().as_secs_f64();
    writeln!(
        std::io::stdout().lock(),
        "keyed freeze {name} wall={wall:.6}s exit={} split_or_report={}",
        output.status,
        wall > if name == "family" { 75.0 } else { 10.0 }
    )
    .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout.lines().filter(|l| *l == "running 1 test").count(), 1);
    assert_eq!(
        stdout
            .lines()
            .filter(|l| l.starts_with("test result: ok. 1 passed; 0 failed;"))
            .count(),
        1
    );
    for line in stdout.lines() {
        writeln!(std::io::stdout().lock(), "freeze child: {line}").unwrap();
    }
    output
}

fn child(test: &str, timeout: &str) -> Command {
    let mut command = Command::new("/usr/bin/timeout");
    command
        .args(["--kill-after=1", timeout])
        .arg(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", test, "--nocapture"]);
    command
}

#[test]
fn supplied_keys_have_bounded_freeze_evidence() {
    run(&mut child("keyed_final::family_child", "300"), "family");
}

#[test]
#[ignore = "bounded parent selects exactly one family child"]
fn family_child() {
    for subject in [
        "manifest",
        "control-key",
        "second-key",
        "edge-keys",
        "zero-key",
        "ones-key",
        "high-bit-key",
        "bindings",
        "tracing",
    ] {
        run(
            child("keyed_final::subject_child", "40").env("TRIPOD_FREEZE_SUBJECT", subject),
            subject,
        );
    }
}

#[test]
#[ignore = "bounded family selects exactly one subject child"]
fn subject_child() {
    let started = std::time::Instant::now();
    match std::env::var("TRIPOD_FREEZE_SUBJECT").unwrap().as_str() {
        "manifest" => {
            controls::manifest();
            census::check();
            freeze_history::check();
        }
        "control-key" => reproduce([42; 32], started),
        "second-key" => {
            assert_eq!(
                target_elements_conformance::constructor::tagged::sha256(
                    SECOND_KEY_PHRASE.as_bytes()
                ),
                SECOND_KEY
            );
            reproduce(SECOND_KEY, started);
        }
        "edge-keys" => controls::edge_keys(),
        "zero-key" => reproduce([0; 32], started),
        "ones-key" => reproduce([255; 32], started),
        "high-bit-key" => reproduce([128; 32], started),
        "bindings" => controls::bindings(),
        "tracing" => controls::tracing(),
        _ => panic!("unknown subject"),
    }
}

fn reproduce(key: [u8; 32], started: std::time::Instant) {
    use std::collections::BTreeSet;
    use vectors::keyed_history_reader::{Handle, Key, Membership, Output, read};
    let (packets, snapshots) = fixture::graph(key);
    for (packet, snapshot) in packets.iter().zip(&snapshots) {
        inventory::check(packet, snapshot);
        let (_, commitment) = prior::rebuild(packet, true);
        prediction::check(key, &commitment);
    }
    let reading = read(&fixture::reader_input(key, &packets, &snapshots)).unwrap();
    let handle = |event| Handle {
        key: Key(key),
        output: Output { event, position: 0 },
    };
    for event in [10, 20] {
        assert_eq!(
            reading.memberships()[&handle(event)],
            Membership::Complete(BTreeSet::from([handle(event)]))
        );
    }
    assert_eq!(
        reading.memberships()[&handle(30)],
        Membership::Complete(BTreeSet::from([handle(10), handle(20)]))
    );
    assert_eq!(reading.live(), &BTreeSet::from([handle(30)]));
    assert_eq!(reading.relations().len(), 4);
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("public.json");
    let output = directory.path().join("result.json");
    std::fs::write(&input, serde_json::to_vec(&packets).unwrap()).unwrap();
    // Leave one second for reporting/cleanup inside the outer 40-second guard.
    let replay_guard = 39_u64
        .checked_sub(started.elapsed().as_secs())
        .unwrap()
        .min(30);
    assert!(replay_guard > 0, "subject exhausted before replay");
    run(
        child("keyed_final::reproduction_child", &replay_guard.to_string())
            .env_clear()
            .env("TRIPOD_FREEZE_INPUT", &input)
            .env("TRIPOD_FREEZE_OUTPUT", &output),
        "fresh-reproduction",
    );
    let (pid, actual): (u32, Vec<prior::Snapshot>) =
        serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_ne!(pid, std::process::id());
    assert_eq!(actual, snapshots);
}

#[test]
#[ignore = "bounded subject supplies only public packets and recorded signatures"]
fn reproduction_child() {
    let packets: Vec<prior::Packet> = serde_json::from_slice(
        &std::fs::read(std::env::var_os("TRIPOD_FREEZE_INPUT").unwrap()).unwrap(),
    )
    .unwrap();
    let snapshots: Vec<_> = packets
        .iter()
        .map(|p| {
            let (snapshot, commitment) = prior::rebuild(p, true);
            prediction::check(p.key, &commitment);
            snapshot
        })
        .collect();
    std::fs::write(
        std::env::var_os("TRIPOD_FREEZE_OUTPUT").unwrap(),
        serde_json::to_vec(&(std::process::id(), snapshots)).unwrap(),
    )
    .unwrap();
}
