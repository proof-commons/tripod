//! Signer abstraction.
//!
//! Implements `(´def:verification:signer-set´)`.
//!
//! The model interprets signer membership as authorization of the
//! complete modeled transaction output set under an output-committing
//! sighash mode. Exact signature bytes and sighash flags remain a
//! compiler/deployment obligation.

use std::collections::BTreeSet;

use crate::guard::Guard;
use crate::scalar::OwnerKey;

// ´def:verification:signer-set´

/// Set of public abstract owner identities considered to have authorized
/// the complete modeled transaction.
///
/// This is model evidence only. It contains no signatures, private keys,
/// signing nonces, or wallet capabilities (ADR-015).
pub type SignerSet = BTreeSet<OwnerKey>;

pub fn require_signer(signers: &SignerSet, owner: OwnerKey) -> Result<(), Guard> {
    if signers.contains(&owner) {
        Ok(())
    } else {
        Err(Guard::BadSignature)
    }
}
