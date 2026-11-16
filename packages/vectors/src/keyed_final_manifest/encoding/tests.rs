use super::*;

// Former Debug rendering is a compatibility oracle, never the recipe definition.
fn former(manifest: &FreezeManifest) -> String {
    format!("keyed-capture-freeze-v1:{manifest:?}")
}

#[test]
fn every_manifest_coordinate_is_encoded() {
    let control = FreezeManifest::offer([42; 32]);
    let encoded = render(&control).unwrap();
    assert_eq!(encoded, former(&control), "former Debug rendering drifted");
    let mutations: &[fn(&mut FreezeManifest)] = &[
        |m| {
            m.frozen.remove(&FrozenItem::Declarations);
        },
        |m| m.frozen.get_mut(&FrozenItem::Declarations).unwrap().clear(),
        |m| {
            m.frozen.get_mut(&FrozenItem::Declarations).unwrap()[0]
                .path
                .push('x');
        },
        |m| m.frozen.get_mut(&FrozenItem::Declarations).unwrap()[0].digest[0] ^= 1,
        |m| m.computed[0] = ComputedItem::Root,
        |m| m.key[0] ^= 1,
        |m| m.leaf_digest[0] ^= 1,
        |m| m.recipe_schema += 1,
        |m| m.target_version += 1,
        |m| m.envelope.handles += 1,
        |m| m.envelope.signature_bytes += 1,
        |m| m.envelope.weight_units += 1,
        |m| m.envelope.sponsors += 1,
        |m| m.envelope.outputs += 1,
    ];
    for mutate in mutations {
        let mut subject = control.clone();
        mutate(&mut subject);
        let changed = render(&subject).unwrap();
        assert_ne!(changed, encoded);
        assert_eq!(changed, former(&subject), "former Debug rendering drifted");
    }
}

#[test]
fn stored_sequence_order_counts() {
    let control = FreezeManifest::offer([42; 32]);
    let mut subject = control.clone();
    subject.computed.swap(0, 1);
    assert_ne!(render(&control).unwrap(), render(&subject).unwrap());
    assert_eq!(
        render(&subject).unwrap(),
        former(&subject),
        "former Debug rendering drifted"
    );
    subject = control.clone();
    subject
        .frozen
        .get_mut(&FrozenItem::Declarations)
        .unwrap()
        .swap(0, 1);
    assert_ne!(render(&control).unwrap(), render(&subject).unwrap());
    assert_eq!(
        render(&subject).unwrap(),
        former(&subject),
        "former Debug rendering drifted"
    );
}

#[test]
fn literal_empty_and_primitive_grammar() {
    let mut subject = FreezeManifest::offer([0; 32]);
    subject.frozen.clear();
    subject.computed.clear();
    subject.recipe_schema = u32::MAX;
    subject.target_version = 0;
    subject.envelope.handles = u64::MAX;
    let encoded = render(&subject).unwrap();
    assert!(encoded.starts_with(
        "keyed-capture-freeze-v1:FreezeManifest { frozen: {}, computed: [], key: [0, 0, "
    ));
    assert!(encoded.ends_with(", recipe_schema: 4294967295, target_version: 0, envelope: Envelope { handles: 18446744073709551615, signature_bytes: 72, weight_units: 400000, sponsors: 1, outputs: 3 } }"));
    assert_eq!(encoded, former(&subject), "former Debug rendering drifted");
    let mut out = String::new();
    bytes(&mut out, &[0, 9, 10, 99, 100, 255]);
    assert_eq!(out, "[0, 9, 10, 99, 100, 255]");
}

#[test]
fn path_alphabet_control_and_one_scalar_refusals() {
    use crate::keyed_final_manifest::{FreezeBoundary, validate_manifest};

    let mut out = String::new();
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789/._-";
    string(&mut out, alphabet).unwrap();
    assert_eq!(out, format!("\"{alphabet}\""));
    let control = FreezeManifest::offer([42; 32]);
    assert_eq!(validate_manifest(&control, control.key), Ok(()));
    assert!(control.capture_encoding().is_ok());
    for scalar in [' ', '"', '\\', 'é', '@', '[', '`', '{', ':', ',', '\u{7f}'] {
        let mut subject = control.clone();
        subject.frozen.get_mut(&FrozenItem::Declarations).unwrap()[0]
            .path
            .replace_range(..1, &scalar.to_string());
        assert_eq!(
            subject.capture_encoding(),
            Err(FreezeRefusal::FrozenInventory)
        );
        assert_eq!(
            validate_manifest(&subject, subject.key),
            Err(FreezeRefusal::FrozenInventory)
        );
    }
    assert_eq!(
        FreezeRefusal::FrozenInventory.boundary(),
        FreezeBoundary::SourceFreeze
    );
    assert_eq!(
        FreezeRefusal::FrozenInventory.refusal_kind(),
        "frozen_inventory"
    );
}

#[test]
fn one_coordinate_mutant_has_literal_bytes() {
    let control = FreezeManifest {
        frozen: std::collections::BTreeMap::default(),
        computed: Vec::new(),
        key: [0; 32],
        leaf_digest: [0; 32],
        recipe_schema: 0,
        target_version: 0,
        envelope: Envelope {
            handles: 0,
            signature_bytes: 0,
            weight_units: 0,
            sponsors: 0,
            outputs: 0,
        },
    };
    let mut subject = control.clone();
    subject.target_version = 1;
    let expected = concat!(
        "keyed-capture-freeze-v1:FreezeManifest { frozen: {}, computed: [], ",
        "key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], ",
        "leaf_digest: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], ",
        "recipe_schema: 0, target_version: 1, envelope: Envelope { handles: 0, signature_bytes: 0, weight_units: 0, sponsors: 0, outputs: 0 } }"
    );
    assert_ne!(control.capture_encoding().unwrap(), expected);
    assert_eq!(subject.capture_encoding().unwrap(), expected);
}
