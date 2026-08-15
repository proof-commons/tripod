//! The canonical primitive census.
//!
//! # What a case establishes, and what establishes it
//!
//! A case is one generic target execution and the outcome the reviewed
//! contract requires of it. The outcome is authored here from the
//! contract and from published vectors — never from an executor's
//! answer — and the executor's only job is to run the script and say
//! whether the spend was valid (Guide-9 §10.6).
//!
//! # The one rule that shapes every script
//!
//! The reviewed execution domain requires evaluation to finish with
//! exactly one item, and that item to be true. Most reviewed primitives
//! push more than one. Until the compound-proof substrate was reviewed
//! there was no equality, drop, or verify primitive to reduce them with,
//! and the groups written before it still state their outcomes the way
//! that constraint forced. So a case ends one of four ways, and which
//! one it ends is the observation:
//!
//! ```text
//! accepted                    one true item
//! completed false             one false item
//! left several items          the primitive succeeded and pushed more
//! aborted                     a primitive failed, in a stated class
//! ```
//!
//! Where a reduction exists — converting a primitive's success flag to
//! the fixed width and comparing it with the result — the successful
//! path collapses to one item and the retained-operand failure path to
//! two, so the two verdicts differ. Where none exists, the case states
//! the depth, which is still the contract's success alternative.
//!
//! The compound-proof group in [`compound`] is the first that does not
//! work under that limitation: it has equality and verification, so its
//! cases compare the target's result against an independently computed
//! expectation and end in one true item. Reworking the older groups to
//! assert their bytes the same way is available and deliberately not
//! done here — it would rewrite evidence this wave did not review.
//!
//! # What this census deliberately does not contain
//!
//! - a signature over a transaction sighash, which depends on the
//!   transaction the executor builds and so cannot be a static fixture;
//! - blinded assets, amounts, and nonces, and issuing inputs, which the
//!   reviewed executor cannot yet materialize;
//! - a case with no introspection context, which a node cannot produce;
//! - an execution-domain rejection, which needs a domain the reviewed
//!   contract deliberately does not describe;
//! - a relative timelock at the top of the sequence mask counted in
//!   *blocks*, which would need an input sixty-five thousand
//!   confirmations deep. The same boundary counted in intervals is
//!   stated instead, because the mask is the same sixteen bits either
//!   way and only one of the two is a chain a run can build.
//!
//! Each is recorded as a residual. A case that can only be answered with
//! infrastructure trouble establishes nothing, and a census padded with
//! them reports a broken environment as target evidence.

pub mod author;
pub mod compound;
pub mod context;
pub mod crypto;
pub mod encoding;
pub mod hashing;
pub mod introspection;
pub mod material;
pub mod numeric;
pub mod timelock;

use target_elements::{ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition};

use crate::error::NativeConformanceError;
use crate::fixture::PrimitiveFixtureSet;

use author::CensusAuthor;

/// The canonical census, stated against one contract and one binding.
///
/// # Errors
///
/// [`NativeConformanceError::FixtureNotExpressible`] when a case in this
/// repository's own source states something the reviewed contract does
/// not admit, and
/// [`NativeConformanceError::DuplicateFixtureCase`] when two cases claim
/// one identity.
pub fn canonical_census(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
) -> Result<PrimitiveFixtureSet, NativeConformanceError> {
    let mut author = CensusAuthor::new(target, binding);

    encoding::cases(&mut author);
    numeric::cases(&mut author);
    hashing::cases(&mut author);
    introspection::cases(&mut author);
    crypto::cases(&mut author);
    timelock::cases(&mut author);
    compound::cases(&mut author);

    PrimitiveFixtureSet::new(author.finish()?)
}
