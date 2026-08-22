//! The live-receipt output key, and the obligation it discharges
//! (Guide-13 §7.5, §12.6).
//!
//! # The obligation this closes
//!
//! Every layer between the constructor and the transaction ABI carried a
//! `TaprootOutputKeyUndischarged` obligation, and each carried it for the
//! same reason: computing a taproot output key needs point arithmetic,
//! and the backend, the linker, and the transaction builder deliberately
//! have none. This package does, and it has it because the target does —
//! the whole of [`crate::constructor`] exists so that an expectation
//! about a constructor's exact output is produced by something other
//! than the thing under test.
//!
//! So the obligation ends here rather than being handed on again. What
//! this module adds to [`crate::constructor::tree`] is not arithmetic;
//! it is a name. A live receipt's output key is the tweak of the
//! constructor's committed root under its inherited internal key, and
//! saying so in one function is what lets a caller ask for it without
//! reassembling the two steps and getting the preimage order wrong.
//!
//! # What it does not discharge
//!
//! That the key is unspendable through its key path (§7.5's internal-key
//! policy), and that a target accepts a spend committed under it. The
//! first is an argument about where the internal key came from, and the
//! second is a run. Neither is a computation this module could perform,
//! and neither is claimed.
//!
//! # No secret material, at any point
//!
//! The same rule the rest of [`crate::constructor`] holds: the internal
//! key is a published point with no known private scalar, the tweak is a
//! hash of public data, and the arithmetic is public point arithmetic.
//! Nothing here generates, accepts, holds, or derives a private scalar.

use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::constructor::tagged::Digest32;
use crate::constructor::tree::{TweakDefect, output_program, tweak, tweaked_key};

/// Everything one live receipt's committed tree and internal key
/// determine.
///
/// Four values rather than one, because a caller checking a builder's
/// destination output needs the program and a caller checking its
/// control block needs the parity, and returning the key alone would
/// make each of them recompute one of the other three.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveReceiptOutputKey {
    tweak: Digest32,
    key: [u8; FIELD_ELEMENT_BYTES],
    parity: u8,
    program: Vec<u8>,
}

impl LiveReceiptOutputKey {
    /// The tweak the internal key and the committed root determined.
    #[must_use]
    pub const fn tweak(&self) -> &Digest32 {
        &self.tweak
    }

    /// The x-only output key.
    #[must_use]
    pub const fn key(&self) -> &[u8; FIELD_ELEMENT_BYTES] {
        &self.key
    }

    /// The output key's parity, as a control block's low bit carries it.
    #[must_use]
    pub const fn parity(&self) -> u8 {
        self.parity
    }

    /// The witness-version-one program an output carrying this receipt
    /// pays to.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }
}

/// The output key one live-receipt constructor's tree determines.
///
/// The independent answer to what a transaction builder's destination
/// output should be paying to: the builder assembles the tree from the
/// linked leaf programs and hashes it, and this recomputes the key from
/// that root under the deployment's internal key using this package's
/// own curve arithmetic. Two computations of the same value, and the one
/// that is not the builder's is this one.
///
/// # Errors
///
/// [`TweakDefect::InternalKeyNotOnCurve`] when the internal key is not
/// an x coordinate any point has; [`TweakDefect::TweakNotAScalar`] when
/// the tweak is at or above the group order; and
/// [`TweakDefect::TweakedKeyIsIdentity`] when the sum is the identity,
/// which has no x-only encoding and therefore no output.
pub fn live_receipt_output_key(
    internal_key: &[u8; FIELD_ELEMENT_BYTES],
    merkle_root: &Digest32,
) -> Result<LiveReceiptOutputKey, TweakDefect> {
    let tweak = tweak(internal_key, merkle_root);
    let (key, parity) = tweaked_key(internal_key, &tweak)?;

    Ok(LiveReceiptOutputKey {
        tweak,
        key,
        parity,
        program: output_program(&key),
    })
}
