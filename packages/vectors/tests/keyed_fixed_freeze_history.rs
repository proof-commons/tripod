//! Initial fixed-form freeze and a synthetic stale-inventory subject.
//! Source bytes are checked against literal pins; grammar is rebuilt separately.
use elements::hashes::sha256;
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use vectors::keyed_fixed_manifest::{FreezeManifest, FreezeRefusal};

fn parameters() -> KeyedFixedParameters {
    KeyedFixedParameters {
        key: [0; 32],
        asset: [0x23; 32],
        value: KeyedFixedValue::new(1).unwrap(),
    }
}

// A separate route writes field groups as independently specified fixed-width
// records. It never invokes the producer's encoder or hashes its output.
fn independent_encoding(m: &FreezeManifest) -> Vec<u8> {
    let mut groups = vec![b"tripod-keyed-fixed-capture-freeze-v1\0".to_vec()];
    groups.push(
        u32::try_from(m.sources.len())
            .unwrap()
            .to_be_bytes()
            .to_vec(),
    );
    for s in &m.sources {
        groups.push(u32::try_from(s.path.len()).unwrap().to_be_bytes().to_vec());
        groups.push(s.path.as_bytes().to_vec());
        groups.push(s.digest.to_vec());
    }
    groups.push(vec![0, 0, 0, 11]);
    groups.push(vec![
        0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0, 9, 0, 10, 0, 11,
    ]);
    let d = &m.descriptor;
    groups.extend([
        d.parameters.key.to_vec(),
        d.parameters.asset.to_vec(),
        d.parameters.value.get().to_be_bytes().to_vec(),
        d.transcript.to_vec(),
        d.pair_digest.to_vec(),
    ]);
    groups.extend([
        vec![0, 0, 0, 104],
        d.leaf.clone(),
        vec![0xc4],
        d.internal_key.to_vec(),
        d.root.to_vec(),
        d.tweak.to_vec(),
        d.output_key.to_vec(),
        vec![u8::from(d.parity)],
        vec![0, 0, 0, 34],
        d.output_program.clone(),
        vec![0, 0, 0, 33],
        d.control.clone(),
    ]);
    groups.push(vec![0, 0, 0, 1, 0, 0, 0, 2, 0]);
    groups.extend([104_u64, 140, 143, 0, 0].map(|n| n.to_be_bytes().to_vec()));
    groups.push(vec![0, 0, 0, 1]);
    groups.concat()
}

#[test]
fn initial_history_independent_encoding_and_named_validation_boundaries() {
    let q = parameters();
    let control = FreezeManifest::offer(q).unwrap();
    let validated = control.clone().validate(q).unwrap();
    // Initial S5 snapshot, no historical capture. Expected bytes were assembled
    // outside these crates from the record: Python hashlib for source/transcript
    // and tagged hashes, affine secp256k1 addition in JavaScript for the point,
    // then the explicitly ordered binary grammar. No producer output was read.
    // Current recipe identity: 5eec00be6a14319f447138dcf59bbf621fc53031918a11d4a82312b84a3d930c
    assert_eq!(validated.encoding().len(), 4249);
    assert_eq!(
        validated.identity(),
        [
            94, 236, 0, 190, 106, 20, 49, 159, 68, 113, 56, 220, 245, 155, 191, 98, 31, 197, 48,
            49, 145, 138, 17, 212, 168, 35, 18, 184, 74, 61, 147, 12,
        ]
    );
    let independent = independent_encoding(&control);
    assert_eq!(validated.encoding(), independent);
    assert_eq!(
        validated.identity(),
        sha256::Hash::hash(&independent).to_byte_array()
    );
    assert!(
        control
            .sources
            .windows(2)
            .all(|rows| rows[0].path < rows[1].path)
    );
    assert!(control.envelope.maximum_members.is_none());
    let mut subject = control.clone();
    subject.sources.pop();
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Inventory));
    let mut subject = control.clone();
    subject.sources.swap(0, 1);
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Inventory));
    let mut subject = control.clone();
    subject.computed.reverse();
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Inventory));
    let mut subject = control.clone();
    subject.descriptor.parameters.key[0] ^= 1;
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Triple));
    let mut subject = control.clone();
    subject.descriptor.output_program[2] ^= 1;
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Descriptor));
    let mut subject = control.clone();
    subject.recipe_schema += 1;
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Recipe));
    let mut subject = control.clone();
    subject.target_version += 1;
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Target));
    let mut subject = control;
    subject.envelope.maximum_members = Some(1564);
    assert_eq!(subject.validate(q), Err(FreezeRefusal::Envelope));
    let legacy = vectors::keyed_final_manifest::FreezeManifest::offer(q.key)
        .capture_encoding()
        .unwrap();
    assert_ne!(validated.encoding(), legacy.as_bytes());
    assert_ne!(
        validated.identity(),
        sha256::Hash::hash(legacy.as_bytes()).to_byte_array()
    );
}

#[test]
fn synthetic_stale_source_inventory_is_refused() {
    let q = parameters();
    let validated = FreezeManifest::offer(q).unwrap().validate(q).unwrap();
    // This is a test-owned stale subject, not a historical landed freeze.
    let mut stale = validated.manifest().clone();
    stale.sources[0].digest[0] ^= 1;
    let stale_identity = sha256::Hash::hash(&independent_encoding(&stale)).to_byte_array();
    assert_ne!(stale_identity, validated.identity());
    assert_eq!(stale.validate(q), Err(FreezeRefusal::Inventory));
}
