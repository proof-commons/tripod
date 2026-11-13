//! Public-entry control for the measured one-sponsor/three-output envelope.
use crate::keyed_reproduction::fixture::{self as prior, Packet};
use std::cell::Cell;
use transaction::keyed_construct::{
    KeyedConstructionError, KeyedConstructionRequest, KeyedOutputOffer, KeyedPolicy, admit_keyed,
};
use transaction::keyed_inputs::{KeyedInputOffer, KeyedInputRole};
use transaction::{AssetField, AssetId, NonceField, Outpoint, TargetOutput, Txid, ValueField};

struct Replay<'a>(&'a Packet, Cell<usize>);
impl transaction::SponsorCapability for Replay<'_> {
    fn offer(&self) -> transaction::SponsorOffer {
        transaction::SponsorOffer::new(
            self.0
                .inputs
                .iter()
                .map(|i| Outpoint::new(Txid::from_internal(i.txid), i.vout).unwrap()),
            self.0.fee,
            Some(ValueField::Explicit(self.0.change)),
        )
        .unwrap()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, self.0.change_program[2..].to_vec()))
    }
    fn sign(
        &self,
        request: &transaction::SponsorSigningRequest,
    ) -> Option<transaction::SponsorSignature> {
        self.1.set(self.1.get() + 1);
        Some(transaction::SponsorSignature::new(
            request.transaction().to_vec(),
            self.0.witness.clone(),
        ))
    }
}

fn admit(packet: &Packet) -> Result<usize, KeyedConstructionError> {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let commitment =
        transaction::keyed_taproot::commit_keyed_bundle(&target, &prior::linked(packet.key, true))
            .unwrap();
    let cap = Replay(packet, Cell::new(0));
    let inputs: Vec<_> = packet
        .inputs
        .iter()
        .enumerate()
        .map(|(index, row)| KeyedInputOffer {
            input_index: u32::try_from(index).unwrap(),
            parent: &row.parent,
            outpoint: Outpoint::new(Txid::from_internal(row.txid), row.vout).unwrap(),
            sequence: row.sequence,
            flags: 0,
            role: KeyedInputRole::Collateral,
            capability: Some(&cap),
        })
        .collect();
    let request =
        transaction::keyed_request::keyed_source_request(&target, packet.key, &commitment, &inputs)
            .unwrap();
    let asset = AssetId::from_internal(packet.asset);
    let outputs = KeyedOutputOffer {
        policy_asset: asset,
        successor_value: packet.successor,
        additional_outputs: vec![TargetOutput::new(
            AssetField::Explicit(asset),
            ValueField::Explicit(packet.change),
            NonceField::Null,
            packet.change_program.clone(),
        )],
        fee: Some(packet.fee),
    };
    let policy = KeyedPolicy::new(
        packet.provenance.clone(),
        asset,
        packet.rates,
        packet.minimums.clone(),
        packet.ceilings,
        packet.package_before,
        packet.signature_bound,
    )
    .unwrap();
    let result = admit_keyed(
        &target,
        KeyedConstructionRequest::Source(&request),
        outputs,
        &policy,
    );
    assert_eq!(cap.1.get(), usize::from(result.is_ok()));
    result.map(|value| value.transaction().outputs().len())
}

/// A second checked collateral spend refuses before asking for any signature.
pub fn check() {
    let control = prior::case(0);
    let frozen = vectors::keyed_final_manifest::FreezeManifest::offer(control.key).envelope;
    assert_eq!(
        u64::try_from(control.inputs.len()).unwrap(),
        frozen.sponsors
    );
    assert_eq!(
        u64::try_from(admit(&control).unwrap()).unwrap(),
        frozen.outputs
    );
    let mut extra = control;
    let mut input = extra.inputs[0].clone();
    input.parent = prior::parent(&extra.change_program, 678);
    input.txid = prior::txid(&input.parent);
    extra.inputs.push(input);
    extra.change += 1_000_000; // Derived balance adjustment for the extra parent output.
    assert_eq!(admit(&extra), Err(KeyedConstructionError::Envelope));
}
