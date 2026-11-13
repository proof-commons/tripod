//! Public-byte controls for explicit funding readback binding.
use crate::protocol::{FundedOutput, MinedFundingReadback, WireOutpoint};
use crate::sponsor_parent::{
    MAXIMUM_SPONSOR_PARENT_BYTES, SponsorParentRefusal as R, bind_sponsor_parent,
};
use transaction::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};

/// Canonical public parent and independently named funded output for wire controls.
///
/// # Panics
/// Panics only if the literal test transaction cannot be encoded, which callers
/// cannot arrange because this helper takes no inputs.
pub fn control() -> (MinedFundingReadback, FundedOutput) {
    let asset = AssetId::from_internal(std::array::from_fn(|i| u8::try_from(i).unwrap()));
    let tx = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([7; 32]), 3).unwrap(),
            19,
        )],
        vec![
            TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Explicit(1),
                NonceField::Null,
                vec![0x52],
            ),
            TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Explicit(99_999),
                NonceField::Null,
                vec![0x51],
            ),
        ],
        5,
        vec![InputWitness::new(vec![])],
    )
    .unwrap();
    let raw = tx.encode();
    let digest = |bytes: &[u8]| {
        use crate::constructor::tagged::sha256;
        Txid::from_internal(sha256(&sha256(bytes))).to_target_display()
    };
    let txid = digest(&tx.encode_without_witness());
    (
        MinedFundingReadback {
            transaction_id: txid.clone(),
            witness_transaction_id: digest(&raw),
            block_hash: "07".repeat(32),
            block_height: 11,
            raw_transaction: raw,
        },
        FundedOutput {
            outpoint: WireOutpoint { txid, vout: 1 },
            asset: "1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100".into(),
            amount_satoshis: 99_999,
            script: "51".into(),
        },
    )
}

#[test]
fn sponsor_parent_binds_each_public_fact() {
    let (readback, funded) = control();
    assert_eq!(
        bind_sponsor_parent(&readback, &funded),
        Ok(readback.raw_transaction.as_slice())
    );
    let mut subject = readback.clone();
    subject.raw_transaction = vec![0; MAXIMUM_SPONSOR_PARENT_BYTES + 1];
    assert_eq!(bind_sponsor_parent(&subject, &funded), Err(R::Size));
    let mut subject = readback.clone();
    subject.raw_transaction.pop();
    assert_eq!(bind_sponsor_parent(&subject, &funded), Err(R::Decode));
    let mut subject = readback.clone();
    subject.transaction_id.replace_range(..2, "01");
    assert_eq!(
        bind_sponsor_parent(&subject, &funded),
        Err(R::ReadbackIdentity)
    );
    let mut subject = readback.clone();
    subject.witness_transaction_id.replace_range(..2, "02");
    assert_eq!(
        bind_sponsor_parent(&subject, &funded),
        Err(R::WitnessIdentity)
    );
    let mut subject = funded.clone();
    subject.outpoint.txid.replace_range(..2, "04");
    assert_eq!(
        bind_sponsor_parent(&readback, &subject),
        Err(R::OutpointIdentity)
    );
    let mut subject = funded.clone();
    subject.outpoint.vout = 2;
    assert_eq!(
        bind_sponsor_parent(&readback, &subject),
        Err(R::OutputIndex)
    );
    let mut subject = funded.clone();
    subject.asset.replace_range(..2, "04");
    assert_eq!(bind_sponsor_parent(&readback, &subject), Err(R::Asset));
    let mut subject = funded.clone();
    subject.amount_satoshis += 1;
    assert_eq!(bind_sponsor_parent(&readback, &subject), Err(R::Amount));
    let mut subject = funded.clone();
    subject.script = "52".into();
    assert_eq!(bind_sponsor_parent(&readback, &subject), Err(R::Script));
    // A nonminimal CompactSize input count is rejected by the canonical decoder.
    let mut subject = readback;
    subject.raw_transaction.splice(5..6, [0xfd, 1, 0]);
    assert_eq!(bind_sponsor_parent(&subject, &funded), Err(R::Decode));
}

#[test]
fn funding_readback_trace_has_one_terminal_per_return() {
    use std::io::Write as _;
    const TEST: &str =
        "tests::sponsor_parent_tests::funding_readback_trace_has_one_terminal_per_return";
    if std::env::var("TRIPOD_SPONSOR_PARENT_TRACE").as_deref() != Ok("child") {
        let start = std::time::Instant::now();
        let output = std::process::Command::new("/usr/bin/timeout")
            .args(["--kill-after=1", "40"])
            .arg(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env("TRIPOD_SPONSOR_PARENT_TRACE", "child")
            .output()
            .unwrap();
        writeln!(
            std::io::stdout().lock(),
            "sponsor-parent trace wall={:.6}s exit={}",
            start.elapsed().as_secs_f64(),
            output.status
        )
        .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
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
        return;
    }
    let (readback, funded) = control();
    let ((), trace) = super::support::capture_trace(|| {
        assert!(bind_sponsor_parent(&readback, &funded).is_ok());
        let mut wrong = funded.clone();
        wrong.script = "52".into();
        assert_eq!(bind_sponsor_parent(&readback, &wrong), Err(R::Script));
    });
    assert_eq!(
        trace
            .lines()
            .filter(|line| line.contains("sponsor parent binding completed")
                && line.contains("INFO")
                && line.contains("target_elements_conformance::sponsor_parent"))
            .count(),
        1
    );
    assert_eq!(
        trace
            .lines()
            .filter(|line| line.contains("sponsor parent binding refused")
                && line.contains("DEBUG")
                && line.contains("refusal_kind=\"script\""))
            .count(),
        1
    );
    assert!(!trace.contains("WARN"));
    assert!(!trace.contains("ERROR"));
}

#[test]
fn sponsor_parent_budget_fits_the_wire_and_matches_the_executor() {
    use crate::protocol::ProtocolLimits;
    use crate::sponsor_parent::MAXIMUM_SPONSOR_PARENT_BYTES;
    assert_eq!(MAXIMUM_SPONSOR_PARENT_BYTES, 512 * 1024);
    // JSON's worst array spelling is four bytes per raw byte, leaving envelope room.
    const {
        assert!(
            MAXIMUM_SPONSOR_PARENT_BYTES * 4 + 65_536
                < ProtocolLimits::DEFAULT.maximum_response_bytes
        );
    }
    let source = include_str!("../../../../scripts/elements-native-executor.py");
    assert!(source.contains("MAXIMUM_SPONSOR_PARENT_BYTES = 512 * 1024"));
}

fn rebound(
    mut readback: MinedFundingReadback,
    mut funded: FundedOutput,
    tx: &elements::Transaction,
) -> (MinedFundingReadback, FundedOutput) {
    readback.raw_transaction = elements::encode::serialize(tx);
    readback.transaction_id = tx.txid().to_string();
    readback.witness_transaction_id = tx.wtxid().to_string();
    funded.outpoint.txid.clone_from(&readback.transaction_id);
    (readback, funded)
}

#[test]
fn non_explicit_sponsor_fields_have_their_own_refusals() {
    let (readback, funded) = control();
    assert!(bind_sponsor_parent(&readback, &funded).is_ok());
    let tx: elements::Transaction =
        elements::encode::deserialize(&readback.raw_transaction).unwrap();
    let context = secp256k1_zkp::Secp256k1::signing_only();
    let generator =
        secp256k1_zkp::Generator::new_unblinded(&context, secp256k1_zkp::Tag::from([3; 32]));
    let mut asset = tx.clone();
    asset.output[1].asset = elements::confidential::Asset::Confidential(generator);
    let (a, coin) = rebound(readback.clone(), funded.clone(), &asset);
    assert_eq!(bind_sponsor_parent(&a, &coin), Err(R::NonExplicitAsset));
    let mut value = tx;
    value.output[1].value =
        elements::confidential::Value::Confidential(secp256k1_zkp::PedersenCommitment::new(
            &context,
            99_999,
            secp256k1_zkp::Tweak::from_inner([1; 32]).unwrap(),
            generator,
        ));
    let (v, coin) = rebound(readback, funded, &value);
    assert_eq!(bind_sponsor_parent(&v, &coin), Err(R::NonExplicitValue));
}

#[test]
fn witness_identity_is_not_the_stripped_identity() {
    let (readback, funded) = control();
    let mut tx: elements::Transaction =
        elements::encode::deserialize(&readback.raw_transaction).unwrap();
    tx.input[0].witness.script_witness = vec![vec![1, 2, 3]].into();
    let (mut readback, funded) = rebound(readback, funded, &tx);
    assert_ne!(readback.transaction_id, readback.witness_transaction_id);
    assert_eq!(
        bind_sponsor_parent(&readback, &funded),
        Ok(readback.raw_transaction.as_slice())
    );
    readback
        .witness_transaction_id
        .clone_from(&readback.transaction_id);
    assert_eq!(
        bind_sponsor_parent(&readback, &funded),
        Err(R::WitnessIdentity)
    );
}

#[test]
fn superfluous_witness_flag_is_refused_by_the_pinned_decoder() {
    let (mut readback, funded) = control();
    assert!(bind_sponsor_parent(&readback, &funded).is_ok());
    assert_eq!(readback.raw_transaction[4], 0);
    readback.raw_transaction[4] = 1;
    // One input has four empty witness vectors; two outputs have two each.
    readback.raw_transaction.extend_from_slice(&[0; 8]);
    assert_eq!(bind_sponsor_parent(&readback, &funded), Err(R::Decode));
}
