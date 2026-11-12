//! External public consumer. Replay is reproducibility, never native acceptance.
mod fixture;

use fixture::{Packet, Snapshot};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct ChildReport {
    pid: u32,
    snapshot: Snapshot,
}

#[test]
fn source_and_merge_reproduce_from_public_inputs_in_a_fresh_process() {
    for members in [0, 2, 253] {
        let packet = fixture::case(members);
        let (expected, commitment) = fixture::rebuild(&packet, true);
        check_oracle(&packet, &commitment);
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("public-inputs.json");
        let output = directory.path().join("reproduced.json");
        std::fs::write(&input, serde_json::to_vec(&packet).unwrap()).unwrap();
        let start = std::time::Instant::now();
        let child = std::process::Command::new("/usr/bin/timeout")
            .args(["--kill-after=5", "120"])
            .arg(std::env::current_exe().unwrap())
            .env_clear()
            .env("TRIPOD_KEYED_REPRODUCTION_INPUT", &input)
            .env("TRIPOD_KEYED_REPRODUCTION_OUTPUT", &output)
            .args([
                "--ignored",
                "--exact",
                "keyed_reproduction::public_input_reproduction_child",
                "--nocapture",
            ])
            .output()
            .unwrap();
        writeln!(
            std::io::stdout().lock(),
            "keyed reproduction members={members} child wall={:.3}s timeout=120s exit={}",
            start.elapsed().as_secs_f64(),
            child.status
        )
        .unwrap();
        let stdout = String::from_utf8_lossy(&child.stdout);
        assert!(
            child.status.success(),
            "{stdout}\n{}",
            String::from_utf8_lossy(&child.stderr)
        );
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
        let result: ChildReport = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
        assert_ne!(result.pid, std::process::id());
        assert_eq!(
            result.snapshot, expected,
            "every named byte/projection coordinate"
        );
        assert_eq!(result.snapshot.witnesses.last(), Some(&packet.witness));
        assert_eq!(result.snapshot.witnesses.len(), members + 1);
        for witness in &result.snapshot.witnesses[..members] {
            assert_eq!(
                witness,
                &[commitment.leaf().to_vec(), commitment.control().to_vec()]
            );
        }
    }
}

#[test]
#[ignore = "launched explicitly by the bounded fresh-process parent"]
fn public_input_reproduction_child() {
    let input = std::env::var_os("TRIPOD_KEYED_REPRODUCTION_INPUT").unwrap();
    let packet: Packet = serde_json::from_slice(&std::fs::read(input).unwrap()).unwrap();
    // No fixture::case call and no signing: use the recorded ordinary witness.
    let (snapshot, commitment) = fixture::rebuild(&packet, true);
    check_oracle(&packet, &commitment);
    let output = std::env::var_os("TRIPOD_KEYED_REPRODUCTION_OUTPUT").unwrap();
    std::fs::write(
        output,
        serde_json::to_vec(&ChildReport {
            pid: std::process::id(),
            snapshot,
        })
        .unwrap(),
    )
    .unwrap();
}

fn check_oracle(packet: &Packet, commitment: &transaction::keyed_taproot::KeyedCommitment) {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let oracle = vectors::keyed_constructor::construct_keyed(
        &target,
        tapscript::keyed_constructor::KeyedSemanticKey::new(packet.key),
        &vectors::keyed_constructor::PublicKeyedCurve,
    )
    .unwrap();
    assert_eq!(
        vectors::keyed_constructor::reconstruct_keyed(
            &target,
            &oracle,
            &vectors::keyed_constructor::PublicKeyedCurve
        ),
        Ok(oracle.clone())
    );
    assert_eq!(commitment.leaf(), oracle.leaf_bytes);
    assert_eq!(commitment.leaf_version(), oracle.leaf_version);
    assert_eq!(commitment.internal_key(), &oracle.internal_key);
    assert_eq!(commitment.root(), &oracle.merkle_root);
    assert_eq!(
        commitment.tweak(),
        &target_elements_conformance::constructor::tweak(&oracle.internal_key, &oracle.merkle_root)
    );
    assert_eq!(commitment.output_key(), &oracle.output_key);
    assert_eq!(commitment.parity(), oracle.control.parity);
    assert_eq!(commitment.output_program(), oracle.output_program);
    assert_eq!(commitment.control().as_slice(), oracle.control.bytes);
    // Independent literal known answer, not a value supplied by either hasher.
    let generator = target_elements_conformance::constructor::curve::generator();
    let compressed: Vec<_> = std::iter::once(2).chain(generator.x_only_bytes()).collect();
    let hash = elements::hashes::hash160::Hash::hash(&compressed).to_byte_array();
    assert_eq!(
        hash,
        [
            0x75, 0x1e, 0x76, 0xe8, 0x19, 0x91, 0x96, 0xd4, 0x54, 0x94, 0x1c, 0x45, 0xd1, 0xb3,
            0xa3, 0x23, 0xf1, 0x43, 0x3b, 0xd6
        ]
    );
}

#[test]
fn public_reproduction_refuses_one_wrong_member_beside_its_unchanged_twin() {
    use transaction::keyed_inputs::{KeyedInputOffer, KeyedInputRefusal, KeyedInputRole};
    use transaction::keyed_request::{KeyedRequestRefusal, keyed_merge_request};
    use transaction::keyed_taproot::{KeyedAuthenticationRefusal, commit_keyed_bundle};
    let packet = fixture::case(2);
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let own = commit_keyed_bundle(&target, &fixture::linked(packet.key, false)).unwrap();
    let other = commit_keyed_bundle(&target, &fixture::linked([43; 32], false)).unwrap();
    let foreign = fixture::parent(other.output_program(), 99);
    let mut inputs: Vec<_> = packet
        .inputs
        .iter()
        .enumerate()
        .map(|(index, row)| KeyedInputOffer {
            input_index: u32::try_from(index).unwrap(),
            parent: &row.parent,
            outpoint: transaction::Outpoint::new(
                transaction::Txid::from_internal(row.txid),
                row.vout,
            )
            .unwrap(),
            sequence: row.sequence,
            flags: 0,
            role: if row.member {
                KeyedInputRole::Member
            } else {
                KeyedInputRole::Collateral
            },
            capability: None,
        })
        .collect();
    assert!(keyed_merge_request(&target, packet.key, &own, &inputs).is_ok());
    inputs[1].parent = &foreign;
    inputs[1].outpoint =
        transaction::Outpoint::new(transaction::Txid::from_internal(fixture::txid(&foreign)), 0)
            .unwrap();
    assert!(keyed_merge_request(&target, packet.key, &own, &inputs[..1]).is_ok());
    assert_eq!(
        keyed_merge_request(&target, packet.key, &own, &inputs).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 1,
            refusal: KeyedInputRefusal::Authentication(KeyedAuthenticationRefusal::WrongProgram)
        }
    );
    inputs[1].parent = &packet.inputs[1].parent;
    inputs[1].outpoint =
        transaction::Outpoint::new(transaction::Txid::from_internal(packet.inputs[1].txid), 1)
            .unwrap();
    assert_eq!(
        keyed_merge_request(&target, packet.key, &own, &inputs).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 1,
            refusal: KeyedInputRefusal::LaterBornMember(1)
        }
    );
    inputs[1].outpoint =
        transaction::Outpoint::new(transaction::Txid::from_internal(packet.inputs[1].txid), 0)
            .unwrap();
    assert!(keyed_merge_request(&target, packet.key, &own, &inputs).is_ok());
}
