//! Literal framing and field-sensitivity controls for the owned grammar.
use super::*;
use realization::keyed_fixed::KeyedFixedValue;

fn offer() -> FreezeManifest {
    FreezeManifest::offer(KeyedFixedParameters {
        key: [0; 32],
        asset: [0x23; 32],
        value: KeyedFixedValue::new(1).unwrap(),
    })
    .unwrap()
}

#[test]
fn count_framing_has_an_exact_width_and_refuses_overflow() {
    let mut bytes = Vec::new();
    count(&mut bytes, 0).unwrap();
    assert_eq!(bytes, [0, 0, 0, 0]);
    if let Ok(maximum) = usize::try_from(u32::MAX) {
        bytes.clear();
        count(&mut bytes, maximum).unwrap();
        assert_eq!(bytes, [255, 255, 255, 255]);
    }
    if let Ok(too_large) = usize::try_from(u64::from(u32::MAX) + 1) {
        bytes.clear();
        assert_eq!(count(&mut bytes, too_large), Err(FreezeRefusal::Framing));
        assert_eq!(bytes, [] as [u8; 0]);
    }
}

#[test]
fn alphabet_framing_domain_and_field_sensitivity() {
    let mut control = offer();
    control.sources = vec![SourceIdentity {
        path: "a/Z_0.-".into(),
        digest: [3; 32],
    }];
    control.computed = vec![ComputedItem::Triple];
    let encoded = encode(&control).unwrap();
    let prefix = [
        b"tripod-keyed-fixed-capture-freeze-v1\0".as_slice(),
        &[0, 0, 0, 1],
        &[0, 0, 0, 7],
        b"a/Z_0.-",
        &[3; 32],
        &[0, 0, 0, 1, 0, 1],
    ]
    .concat();
    assert!(encoded.starts_with(&prefix));
    assert!(!encoded.starts_with(b"keyed-capture-freeze-v1:"));
    for path in ["a b", "a\n", "a\"", "a\\b", "é", "a\0b"] {
        let mut subject = control.clone();
        subject.sources[0].path = path.into();
        assert_eq!(encode(&subject), Err(FreezeRefusal::Alphabet));
    }
    let mutations: &[fn(&mut FreezeManifest)] = &[
        |m| m.sources[0].path.push('x'),
        |m| m.sources[0].digest[0] ^= 1,
        |m| m.sources.push(m.sources[0].clone()),
        |m| m.computed.push(ComputedItem::Leaf),
        |m| m.descriptor.parameters.key[0] ^= 1,
        |m| m.descriptor.parameters.asset[0] ^= 1,
        |m| m.descriptor.parameters.value = KeyedFixedValue::new(2).unwrap(),
        |m| m.descriptor.transcript[0] ^= 1,
        |m| m.descriptor.pair_digest[0] ^= 1,
        |m| m.descriptor.leaf.push(0),
        |m| m.descriptor.leaf_version ^= 1,
        |m| m.descriptor.internal_key[0] ^= 1,
        |m| m.descriptor.root[0] ^= 1,
        |m| m.descriptor.tweak[0] ^= 1,
        |m| m.descriptor.output_key[0] ^= 1,
        |m| m.descriptor.parity = !m.descriptor.parity,
        |m| m.descriptor.output_program.push(0),
        |m| m.descriptor.control.push(0),
        |m| m.recipe_schema += 1,
        |m| m.target_version += 1,
        |m| m.envelope.maximum_members = Some(0),
        |m| m.envelope.script_bytes += 1,
        |m| m.envelope.script_witness_bytes += 1,
        |m| m.envelope.input_witness_bytes += 1,
        |m| m.envelope.control_depth += 1,
        |m| m.envelope.argument_items += 1,
        |m| m.envelope.annex = true,
        |m| m.envelope.issuance = true,
        |m| m.envelope.peg_in = true,
        |m| m.envelope.empty_nonce = false,
    ];
    for mutate in mutations {
        let mut subject = control.clone();
        mutate(&mut subject);
        assert_ne!(encode(&subject).unwrap(), encoded);
    }
}
