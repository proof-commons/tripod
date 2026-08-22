//! The live-transfer capabilities, wired to the independent oracle
//! (Guide-13 §12.6, §12.8, §1.8).
//!
//! # Why the wiring lives here and not on either side of it
//!
//! The transaction layer needs two answers it must not compute:
//! whether an offered owner key names a point of the target's curve, and
//! what output key a committed tree determines. Both are curve
//! arithmetic, and the workspace's implementation of that arithmetic
//! belongs to the conformance package — which owns it precisely so that
//! an expectation about a builder's output is produced by something
//! other than the builder.
//!
//! Neither package may reach for the other. The transaction crate's own
//! dependency contract keeps the oracle out, because a builder that
//! computed its expectation by calling the oracle would be producing one
//! opinion and counting it twice; and the oracle does not depend on the
//! builder, because an independent oracle that did would not be
//! independent.
//!
//! This package depends on both, and its whole role is to hold evidence
//! against artifacts it did not produce. So the adapter belongs here:
//! the builder asks, the oracle answers, and the two never learn each
//! other's names.
//!
//! # What this does not make true
//!
//! A destination program derived through this adapter is the program the
//! deployment's constructor determines. It is not evidence that a target
//! accepts a spend committed under it — that is a run (§1.11), and every
//! obligation about it stays where it was.
//!
//! # No secret material crosses either boundary
//!
//! Both methods take public inputs and return public values. There is no
//! scalar in either signature, and the confidential adapter derives its
//! blinding from randomness the caller has already published.

use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::commitment_oracle::commitment::blinded_commitment;
use target_elements_conformance::constructor::tagged::{Digest32, tagged_hash};
use target_elements_conformance::live_receipt_key::live_receipt_output_key;
use target_elements_conformance::owner_key_oracle::offered_key_point;
use transaction::bytes::{AssetId, COMMITMENT_BYTES};
use transaction::live_private::PrivateValueCapability;
use transaction::live_request::{ProtocolValue, PublicTestRandomness};
use transaction::live_taproot::{LiveCurveCapability, TweakedOutputKey};
use transaction::taproot::OutputKeyParity;

/// The curve capability, answered by the independent host oracle.
///
/// Holds the reviewed contract because the owner-key question is asked
/// against the approved encoding the contract fixes, rather than against
/// a width written down here.
pub struct OracleLiveCurve {
    target: ReviewedElementsTapscriptDefinition,
}

impl OracleLiveCurve {
    /// The capability over one reviewed contract.
    #[must_use]
    pub const fn new(target: ReviewedElementsTapscriptDefinition) -> Self {
        Self { target }
    }
}

impl LiveCurveCapability for OracleLiveCurve {
    fn owner_key_is_a_curve_point(&self, owner: &[u8]) -> bool {
        offered_key_point(&self.target, owner).is_ok()
    }

    fn output_key(&self, internal_key: &[u8], merkle_root: &Digest32) -> Option<TweakedOutputKey> {
        let internal = <[u8; 32]>::try_from(internal_key).ok()?;
        let derived = live_receipt_output_key(&internal, merkle_root).ok()?;
        let parity = if derived.parity() == 0 {
            OutputKeyParity::Even
        } else {
            OutputKeyParity::Odd
        };
        Some(TweakedOutputKey::new(*derived.key(), parity))
    }
}

/// The tag the central public-fixture materializer derives a blinding
/// factor under.
///
/// A tag of this workspace's own, and deliberately not one of the
/// target's: the value it produces is a blinding factor for a test
/// fixture, and a tag borrowed from the target would suggest the
/// derivation was one the target specifies. It is not; §12.8's central
/// public-fixture model only requires that every opening be reproducible
/// from inputs the caller published, and this is what makes it so.
const FIXTURE_BLINDING_TAG: &str = "Guide13/central-public-fixture-blinding";

/// The confidential value capability, answered by the commitment
/// oracle.
///
/// §12.8's central public-fixture construction, exactly: one party holds
/// every opening, and every opening is a hash of values the caller
/// published. There is no randomness generated here, nothing is stored,
/// and no method hands an opening back.
pub struct OracleFixtureValues;

impl PrivateValueCapability for OracleFixtureValues {
    fn value_commitment(
        &self,
        asset: AssetId,
        value: ProtocolValue,
        randomness: &PublicTestRandomness,
        position: u16,
    ) -> Option<[u8; COMMITMENT_BYTES]> {
        // The blinding factor is a function of the published randomness
        // and the output's own position, so two destinations of equal
        // value commit to different points and the whole construction is
        // reproducible from the request.
        let mut preimage = Vec::with_capacity(34);
        preimage.extend_from_slice(randomness.bytes());
        preimage.extend_from_slice(&position.to_be_bytes());
        let value_blinding = tagged_hash(FIXTURE_BLINDING_TAG, &preimage);

        // The asset blinder is zero, and that is the representation
        // rather than a shortcut: §6.3 keeps the protocol asset explicit
        // under both plans and blinds only the value, so the generator
        // this commitment is taken against is the asset's own unblinded
        // one. The oracle admits a zero scalar for exactly this reason.
        let asset_blinding = [0_u8; 32];

        blinded_commitment(
            asset.internal(),
            value.amount(),
            &asset_blinding,
            &value_blinding,
        )
        .ok()
    }
}
