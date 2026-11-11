//! Public schema-1 arithmetic over the workspace's safe C-library wrapper.
//!
//! Agreement with the node is not independent evidence: both use the same
//! library family. First-party conformance arithmetic remains the oracle.
//! These capability methods are untraced cores beneath commitment boundaries.

use secp256k1_zkp::{Parity, Scalar, Secp256k1, XOnlyPublicKey};
use tapscript::keyed_constructor::{KeyedCurveCapability, KeyedTweakOutcome};

use crate::taproot::{TAP_TWEAK_TAG, tagged_hash};

/// Production public arithmetic; no signing capability or secret is accepted.
#[derive(Clone, Copy, Debug)]
pub struct KeyedCurve;

impl KeyedCurveCapability for KeyedCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        XOnlyPublicKey::from_slice(key).is_ok()
    }

    fn output_key(&self, internal_key: &[u8; 32], root: &[u8; 32]) -> KeyedTweakOutcome {
        apply_tweak(internal_key, tweak(internal_key, root))
    }
}

pub(crate) fn tweak(internal_key: &[u8; 32], root: &[u8; 32]) -> [u8; 32] {
    let mut preimage = [0; 64];
    preimage[..32].copy_from_slice(internal_key);
    preimage[32..].copy_from_slice(root);
    tagged_hash(TAP_TWEAK_TAG, &preimage)
}

// The byte-scalar core permits exact boundary tests without hash preimages.
// After point and scalar parsing, add_tweak's reachable failure is infinity.
// Its subsequent conversion/parity failures are unreachable for a valid finite
// output from the same library; no backend error is treated as success.
pub(crate) fn apply_tweak(internal_key: &[u8; 32], bytes: [u8; 32]) -> KeyedTweakOutcome {
    let Ok(point) = XOnlyPublicKey::from_slice(internal_key) else {
        return KeyedTweakOutcome::InternalKeyNotAPoint;
    };
    let Ok(scalar) = Scalar::from_be_bytes(bytes) else {
        return KeyedTweakOutcome::TweakAboveGroupOrder;
    };
    match point.add_tweak(&Secp256k1::verification_only(), &scalar) {
        Ok((key, parity)) => KeyedTweakOutcome::OutputKey {
            key: key.serialize(),
            parity: parity == Parity::Odd,
        },
        Err(_) => KeyedTweakOutcome::TweakedPointIsIdentity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_scalar_and_point_boundaries_have_accepted_twins() {
        let generator = tapscript::STATE_GENERATOR_X;
        let control = KeyedTweakOutcome::OutputKey {
            key: generator,
            parity: false,
        };
        assert_eq!(apply_tweak(&generator, [0; 32]), control);
        assert_eq!(
            apply_tweak(&[0xff; 32], [0; 32]),
            KeyedTweakOutcome::InternalKeyNotAPoint
        );
        let order = secp256k1_zkp::constants::CURVE_ORDER;
        let mut order_minus_one = order;
        order_minus_one[31] -= 1;
        let mut order_minus_two = order_minus_one;
        order_minus_two[31] -= 1;
        let odd_control = KeyedTweakOutcome::OutputKey {
            key: generator,
            parity: true,
        };
        assert_eq!(apply_tweak(&generator, order_minus_two), odd_control);
        assert_eq!(
            apply_tweak(&generator, order_minus_one),
            KeyedTweakOutcome::TweakedPointIsIdentity
        );
        assert_eq!(apply_tweak(&generator, order_minus_two), odd_control);
        assert_eq!(
            apply_tweak(&generator, order),
            KeyedTweakOutcome::TweakAboveGroupOrder
        );
        assert!(KeyedCurve.internal_key_is_a_point(&generator));
        assert!(!KeyedCurve.internal_key_is_a_point(&[0xff; 32]));
    }
}
