//! Independent literal grammar and first-party arithmetic, never the emitter.
use target_elements_conformance::constructor::{
    curve::generator, output_program, tagged::sha256, tree::control_block_of_version_byte,
    tree::leaf_hash_of_version_byte, tweak, tweaked_key,
};
use transaction::keyed_taproot::KeyedCommitment;
use vectors::keyed_final_manifest::{FreezeManifest, predicted_leaf, validate_manifest};

/// Compare public production material with the literal grammar and independent curve.
pub fn check(key: [u8; 32], actual: &KeyedCommitment) {
    let offered = FreezeManifest::offer(key);
    assert_eq!(validate_manifest(&offered, key), Ok(()));
    let leaf = predicted_leaf(key);
    assert_eq!(leaf.len(), 52);
    assert_eq!(leaf[0], 32);
    assert_eq!(&leaf[1..33], key);
    // Walk instruction boundaries: a 32-byte operand occurs exactly once.
    let mut offset = 0;
    let mut instructions = Vec::new();
    let mut pushes = Vec::new();
    while offset < leaf.len() {
        let op = leaf[offset];
        instructions.push((offset, op));
        offset += 1;
        if (1..=75).contains(&op) {
            pushes.push((offset, op));
            offset += usize::from(op);
        }
    }
    assert_eq!(offset, leaf.len());
    assert_eq!(instructions.len(), 16);
    assert_eq!(pushes, [(1, 32), (38, 4)]);
    assert_eq!(
        instructions
            .iter()
            .filter(|(_, op)| *op == 0x88)
            .map(|(offset, _)| *offset)
            .collect::<Vec<_>>(),
        [42, 49, 50]
    );
    assert_eq!(
        vectors::keyed_native::KEYED_EQUALVERIFY_OFFSETS,
        [42, 49, 50]
    );
    let internal = sha256(&generator().uncompressed_bytes());
    let root = leaf_hash_of_version_byte(196, &leaf);
    let scalar = tweak(&internal, &root);
    let (output, parity) = tweaked_key(&internal, &scalar).unwrap();
    let control = control_block_of_version_byte(196, parity, &internal, &[]);
    assert_eq!(actual.leaf(), leaf);
    assert_eq!(actual.leaf_version(), 196);
    assert_eq!(actual.internal_key(), &internal);
    assert_eq!(actual.root(), &root);
    assert_eq!(actual.tweak(), &scalar);
    assert_eq!(actual.output_key(), &output);
    assert_eq!(actual.parity(), parity == 1);
    assert_eq!(actual.control().as_slice(), control.as_slice());
    assert_eq!(actual.output_program(), output_program(&output));
    assert_eq!(offered.leaf_digest, sha256(actual.leaf()));
}
