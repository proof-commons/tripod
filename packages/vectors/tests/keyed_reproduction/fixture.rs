//! Test-only public-input packet, not a production descriptor or wire format.
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use elements::bitcoin::hashes::Hash;
use linker::{
    keyed_bundle::{LinkedKeyedBundle, link_keyed_bundle},
    keyed_reference::keyed_parameter_offers,
};
use serde::{Deserialize, Serialize};
use tapscript::{
    keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle},
    keyed_constructor::KeyedLeafRecipe,
};
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::constructor::tagged::sha256;
use transaction::keyed_construct::{
    KeyedConstructionRequest, KeyedOutputOffer, KeyedPolicy, admit_keyed, construct_keyed,
};
use transaction::keyed_inputs::{KeyedInputOffer, KeyedInputRole};
use transaction::keyed_request::{keyed_merge_request, keyed_source_request};
use transaction::keyed_taproot::{KeyedCommitment, commit_keyed_bundle};
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub parent: Vec<u8>,
    pub txid: [u8; 32],
    pub vout: u32,
    pub sequence: u32,
    pub member: bool,
}

/// Every variable construction input is carried, including fee and census.
/// Only public bytes cross the process boundary; no secret field exists.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Packet {
    pub key: [u8; 32],
    pub source: bool,
    pub inputs: Vec<Input>,
    pub asset: [u8; 32],
    pub successor: u64,
    pub change: u64,
    pub fee: u64,
    pub change_program: Vec<u8>,
    pub provenance: String,
    pub rates: [u64; 3],
    pub minimums: Vec<u64>,
    pub ceilings: [u64; 3],
    pub package_before: u64,
    pub signature_bound: usize,
    pub witness: Vec<Vec<u8>>,
}

/// Bytes and diagnostic projections, never a stable serialization contract.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub linked_bytes: Vec<u8>,
    pub linked_projection: String,
    pub closure: String,
    pub commitment: Vec<Vec<u8>>,
    pub receipt: String,
    pub coverage: String,
    pub evidence: String,
    pub stripped: Vec<u8>,
    pub complete: Vec<u8>,
    pub witnesses: Vec<Vec<Vec<u8>>>,
    pub obligations: String,
}

pub fn linked(key: [u8; 32], source: bool) -> LinkedKeyedBundle {
    let target = reviewed_elements_tapscript().unwrap();
    let manifest = KeyedManifest::contract().unwrap();
    let declaration = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let limit = std::num::NonZeroU64::new(100_000).unwrap();
    let input = bind_input(
        &manifest,
        declaration,
        vec![O::Keyed(if source { K::Source } else { K::Merge })],
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .unwrap();
    let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
    let projection = plan.project();
    let selection = KeyedBundleSelection {
        binding: projection.binding.clone(),
        scope: projection.scope.clone(),
        proof: 0,
        placements: projection.plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    // Independently offered from the packet key, not read back from emission.
    let recipe = KeyedLeafRecipe::new(key);
    let emission = emit_keyed_bundle(&target, &plan, selection, recipe).unwrap();
    link_keyed_bundle(&target, &emission, &keyed_parameter_offers(&target, recipe)).unwrap()
}

const fn output(asset: [u8; 32], value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(asset)),
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}

pub fn parent(program: &[u8], salt: u32) -> Vec<u8> {
    TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(
                Txid::from_internal(std::array::from_fn(|i| u8::try_from(i).unwrap())),
                salt,
            )
            .unwrap(),
            31,
        )],
        vec![output([3; 32], 1_000_000, program.to_vec()); 2],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode()
}

pub fn txid(bytes: &[u8]) -> [u8; 32] {
    sha256(&sha256(
        &TargetTransaction::decode(bytes)
            .unwrap()
            .encode_without_witness(),
    ))
}

struct Replay<'a> {
    packet: &'a Packet,
    offers: std::cell::Cell<usize>,
    signs: std::cell::Cell<usize>,
}
impl transaction::SponsorCapability for Replay<'_> {
    fn offer(&self) -> transaction::SponsorOffer {
        self.offers.set(self.offers.get() + 1);
        transaction::SponsorOffer::new(
            self.packet
                .inputs
                .iter()
                .filter(|i| !i.member)
                .map(|i| Outpoint::new(Txid::from_internal(i.txid), i.vout).unwrap()),
            self.packet.fee,
            Some(ValueField::Explicit(self.packet.change)),
        )
        .unwrap()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, self.packet.change_program[2..].to_vec()))
    }
    fn sign(
        &self,
        request: &transaction::SponsorSigningRequest,
    ) -> Option<transaction::SponsorSignature> {
        self.signs.set(self.signs.get() + 1);
        assert_eq!(request.role(), transaction::SignerRole::KeyedCollateral);
        assert_eq!(usize::from(request.input()), self.packet.inputs.len() - 1);
        Some(transaction::SponsorSignature::new(
            request.transaction().to_vec(),
            self.packet.witness.clone(),
        ))
    }
}

pub fn reference_digest(bytes: &[u8], index: usize, spent: &TargetOutput) -> [u8; 32] {
    let reference: elements::Transaction = elements::encode::deserialize(bytes).unwrap();
    let script = elements::Script::new_p2pkh(
        &elements::PubkeyHash::from_slice(&spent.program()[2..]).unwrap(),
    );
    let ValueField::Explicit(value) = spent.value() else {
        panic!("explicit fixture")
    };
    elements::sighash::SighashCache::new(&reference)
        .segwitv0_sighash(
            index,
            &script,
            elements::confidential::Value::Explicit(value),
            elements::EcdsaSighashType::All,
        )
        .to_byte_array()
}

pub fn rebuild(packet: &Packet, admitted: bool) -> (Snapshot, KeyedCommitment) {
    let target = reviewed_elements_tapscript().unwrap();
    let linked = linked(packet.key, packet.source);
    let commitment = commit_keyed_bundle(&target, &linked).unwrap();
    rebuild_with_commitment(packet, admitted, &commitment)
}

/// Reuses only immutable public construction material, never an expectation.
///
/// # Panics
/// Panics if the packet and supplied commitment do not form a supported fixture.
pub fn rebuild_with_commitment(
    packet: &Packet,
    admitted: bool,
    commitment: &KeyedCommitment,
) -> (Snapshot, KeyedCommitment) {
    let target = reviewed_elements_tapscript().unwrap();
    let replay = Replay {
        packet,
        offers: std::cell::Cell::new(0),
        signs: std::cell::Cell::new(0),
    };
    let inputs = input_offers(packet, &replay);
    let source;
    let merge;
    let (request, receipt) = if packet.source {
        source = keyed_source_request(&target, packet.key, commitment, &inputs).unwrap();
        (KeyedConstructionRequest::Source(&source), source.members())
    } else {
        merge = keyed_merge_request(&target, packet.key, commitment, &inputs).unwrap();
        (KeyedConstructionRequest::Merge(&merge), merge.members())
    };
    finish(packet, admitted, &replay, commitment, request, receipt)
}

fn input_offers<'a>(packet: &'a Packet, replay: &'a Replay<'_>) -> Vec<KeyedInputOffer<'a, 'a>> {
    packet
        .inputs
        .iter()
        .enumerate()
        .map(|(index, row)| KeyedInputOffer {
            input_index: u32::try_from(index).unwrap(),
            parent: &row.parent,
            outpoint: Outpoint::new(Txid::from_internal(row.txid), row.vout).unwrap(),
            sequence: row.sequence,
            flags: 0,
            role: if row.member {
                KeyedInputRole::Member
            } else {
                KeyedInputRole::Collateral
            },
            capability: if row.member {
                None
            } else {
                Some(replay as &dyn transaction::SponsorCapability)
            },
        })
        .collect()
}

fn finish(
    packet: &Packet,
    admitted: bool,
    replay: &Replay<'_>,
    commitment: &KeyedCommitment,
    request: KeyedConstructionRequest<'_, '_>,
    receipt: &tapscript::keyed_members::KeyedMemberReceipt,
) -> (Snapshot, KeyedCommitment) {
    let target = reviewed_elements_tapscript().unwrap();
    let linked = commitment.linked();
    let member_count = packet.inputs.iter().filter(|row| row.member).count();
    assert_eq!(receipt.members().len(), member_count);
    assert_eq!(receipt.evidence().len(), if packet.source { 4 } else { 3 });
    assert_eq!(receipt.coverage().len(), if packet.source { 0 } else { 2 });
    assert!(
        receipt
            .coverage()
            .values()
            .all(|members| members.len() == member_count)
    );
    let offered = KeyedOutputOffer {
        policy_asset: AssetId::from_internal(packet.asset),
        successor_value: packet.successor,
        additional_outputs: vec![output(
            packet.asset,
            packet.change,
            packet.change_program.clone(),
        )],
        fee: Some(packet.fee),
    };
    let policy = KeyedPolicy::new(
        packet.provenance.clone(),
        AssetId::from_internal(packet.asset),
        packet.rates,
        packet.minimums.clone(),
        packet.ceilings,
        packet.package_before,
        packet.signature_bound,
    )
    .unwrap();
    let (transaction, obligations) = if admitted {
        let result = admit_keyed(&target, request, offered, &policy).unwrap();
        assert_eq!(replay.signs.get(), 1);
        (
            result.transaction().clone(),
            format!("{:?}", result.obligations()),
        )
    } else {
        let result = construct_keyed(&target, request, offered).unwrap();
        assert_eq!(replay.signs.get(), 0);
        (
            result.transaction().clone(),
            format!("{:?}", result.obligations()),
        )
    };
    assert_eq!(
        replay.offers.get(),
        1,
        "request alone reads capability offer"
    );
    let index = packet.inputs.len() - 1;
    let parent = TargetTransaction::decode(&packet.inputs[index].parent).unwrap();
    let spent = &parent.outputs()[usize::try_from(packet.inputs[index].vout).unwrap()];
    assert_eq!(
        transaction::keyed_sponsor::keyed_sponsor_digest(
            &transaction.encode_without_witness(),
            index,
            spent
        )
        .unwrap(),
        reference_digest(&transaction.encode_without_witness(), index, spent)
    );
    let snapshot = Snapshot {
        linked_bytes: linked.bytes().to_vec(),
        linked_projection: format!("{:?}", linked.project()),
        closure: format!("{:?}", linked.closure()),
        commitment: vec![
            commitment.internal_key().to_vec(),
            commitment.root().to_vec(),
            commitment.tweak().to_vec(),
            commitment.output_key().to_vec(),
            commitment.output_program().to_vec(),
            commitment.control().to_vec(),
            vec![commitment.leaf_version(), u8::from(commitment.parity())],
        ],
        receipt: format!("{receipt:?}"),
        coverage: format!("{:?}", receipt.coverage()),
        evidence: format!("{:?}", receipt.evidence()),
        stripped: transaction.encode_without_witness(),
        complete: transaction.encode(),
        witnesses: transaction
            .witnesses()
            .iter()
            .map(|w| w.stack().to_vec())
            .collect(),
        obligations,
    };
    (snapshot, commitment.clone())
}

/// Disposable fixture setup signs, including history-subject setup children;
/// replay and the fresh reproduction child accept only recorded bytes.
pub fn case(members: usize) -> Packet {
    use elements::secp256k1_zkp::{PublicKey, Secp256k1, SecretKey};
    let secret = SecretKey::from_slice(&[1; 32]).unwrap();
    let secp = Secp256k1::new();
    let public = PublicKey::from_secret_key(&secp, &secret).serialize();
    let program: Vec<_> = [0, 20]
        .into_iter()
        .chain(elements::hashes::hash160::Hash::hash(&public).to_byte_array())
        .collect();
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], members == 0)).unwrap();
    let inputs = (0..=members)
        .map(|index| {
            let member = index < members;
            let parent = parent(
                if member {
                    commitment.output_program()
                } else {
                    &program
                },
                u32::try_from(index).unwrap(),
            );
            Input {
                txid: txid(&parent),
                parent,
                vout: u32::from(!member),
                sequence: 100 + u32::try_from(index).unwrap(),
                member,
            }
        })
        .collect();
    let mut packet = Packet {
        key: [42; 32],
        source: members == 0,
        inputs,
        asset: [3; 32],
        successor: 1000,
        change: u64::try_from(members + 1).unwrap() * 1_000_000 - 11_000,
        fee: 10_000,
        change_program: program,
        provenance: "offline test readings; final-byte node observation outstanding".into(),
        rates: [100; 3],
        minimums: vec![100, 100, 1],
        ceilings: [400_000; 3],
        package_before: 0,
        signature_bound: 72,
        witness: vec![],
    };
    record_signature(&mut packet);
    packet
}

/// Disposable funding setup only; reconstruction consumes the resulting public
/// witness without calling this helper or accepting a secret.
///
/// # Panics
/// Panics if the test packet is not the fixed sponsor's supported explicit form.
pub fn record_signature(packet: &mut Packet) {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked(packet.key, packet.source)).unwrap();
    record_signature_with_commitment(packet, &commitment);
}

/// Signs disposable funding using cached public material; no covenant secret.
///
/// # Panics
/// Panics if the explicit packet cannot be constructed with this material.
pub fn record_signature_with_commitment(packet: &mut Packet, commitment: &KeyedCommitment) {
    use elements::secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey};
    let secret = SecretKey::from_slice(&[1; 32]).unwrap();
    let secp = Secp256k1::new();
    let public = PublicKey::from_secret_key(&secp, &secret).serialize();
    let (unsigned, _) = rebuild_with_commitment(packet, false, commitment);
    let index = packet.inputs.len() - 1;
    let parent = TargetTransaction::decode(&packet.inputs[index].parent).unwrap();
    let digest = reference_digest(
        &unsigned.stripped,
        index,
        &parent.outputs()[usize::try_from(packet.inputs[index].vout).unwrap()],
    );
    let mut signature = secp
        .sign_ecdsa(&Message::from_digest(digest), &secret)
        .serialize_der()
        .to_vec();
    signature.push(1);
    packet.witness = vec![signature, public.to_vec()];
}
