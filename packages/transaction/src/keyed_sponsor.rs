//! Narrow Elements SegWit-v0 `SIGHASH_ALL` verification, never signing.
//!
//! Field order follows `SegwitV0SignatureMsg` in the pinned Elements source's
//! `test/functional/test_framework/script.py` and the version-zero branch of
//! `src/script/interpreter.cpp`. Only explicit, non-issuance, non-peg-in forms
//! are admitted. The shared decoder enforces the latter two conditions.
//! Signature production and every other signing profile remain delegated.

use crate::bytes::{compact_size, encode_value_field};
use crate::sponsor::{WITNESS_V0_KEYHASH_PROGRAM_BYTES, WITNESS_V0_KEYHASH_STACK_ITEMS};
use crate::trace_support::TraceRefusal;
use crate::{
    AssetField, NonceField, TargetOutput, TargetTransaction, TransactionRefusal, ValueField,
};
use bitcoin_hashes::{Hash, hash160};
use secp256k1_zkp::{Message, PublicKey, Secp256k1, ecdsa::Signature};
use sha2::{Digest, Sha256};

/// Verification refusal, not a covenant execution verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedSponsorRefusal {
    /// Canonical frozen transaction decoding failed.
    Decode(TransactionRefusal),
    /// Bytes include a witness section or fail stripped canonical round-trip.
    NonCanonicalFrozenBytes,
    /// Explicit/null-nonce output or empty scriptSig requirements failed.
    UnsupportedForm,
    /// Selected input is outside the frozen census.
    InputIndex,
    /// Bound spent program is not witness-v0 key-hash.
    Program,
    /// Witness does not contain exactly signature and public key.
    WitnessShape,
    /// Signature including hash byte is empty or exceeds 72 bytes.
    SignatureLength,
    /// Signature hash byte is not `SIGHASH_ALL` (1).
    HashType,
    /// Backend parsing or exact DER re-encoding failed.
    StrictDer,
    /// Signature is not normalized to low S.
    HighS,
    /// Key is not a valid compressed 33-byte curve point.
    CompressedPublicKey,
    /// Key HASH160 differs from the bound spent program.
    PublicKeyHash,
    /// ECDSA verification over the frozen digest failed.
    InvalidSignature,
}
impl KeyedSponsorRefusal {
    /// Static diagnostic class; no key or amount is logged.
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        self.tracing_kind()
    }
}
impl crate::trace_support::TraceRefusal for KeyedSponsorRefusal {
    fn tracing_kind(&self) -> &'static str {
        match self {
            Self::Decode(_) => "decode",
            Self::NonCanonicalFrozenBytes => "non_canonical_frozen_bytes",
            Self::UnsupportedForm => "unsupported_form",
            Self::InputIndex => "input_index",
            Self::Program => "program",
            Self::WitnessShape => "witness_shape",
            Self::SignatureLength => "signature_length",
            Self::HashType => "hash_type",
            Self::StrictDer => "strict_der",
            Self::HighS => "high_s",
            Self::CompressedPublicKey => "compressed_public_key",
            Self::PublicKeyHash => "public_key_hash",
            Self::InvalidSignature => "invalid_signature",
        }
    }
}
fn hash256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(Sha256::digest(bytes)).into()
}
fn explicit(output: &TargetOutput) -> bool {
    matches!(output.asset(), AssetField::Explicit(_))
        && matches!(output.value(), ValueField::Explicit(_))
        && output.nonce() == NonceField::Null
}

/// Recompute the one supported digest from canonical frozen non-witness bytes.
///
/// The supplied spent output must come from the caller's bound parent; this
/// function alone authenticates neither an outpoint nor chain membership.
/// # Errors
/// Refuses unsupported fields, noncanonical/witness-bearing bytes, index or program.
pub fn keyed_sponsor_digest(
    frozen: &[u8],
    index: usize,
    spent: &TargetOutput,
) -> Result<[u8; 32], KeyedSponsorRefusal> {
    let _span = tracing::debug_span!(target: "transaction::keyed_sponsor", "keyed_sponsor_digest", family = "keyed").entered();
    let result = digest_core(frozen, index, spent);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "transaction::keyed_sponsor", family = "keyed", outcome = "completed", "keyed sponsor digest completed");
        }
        Err(e) => {
            tracing::debug!(target: "transaction::keyed_sponsor", family = "keyed", outcome = "refused", refusal_kind = e.refusal_kind(), "keyed sponsor digest refused");
        }
    }
    result
}
pub(crate) fn digest_core(
    frozen: &[u8],
    index: usize,
    spent: &TargetOutput,
) -> Result<[u8; 32], KeyedSponsorRefusal> {
    let tx = TargetTransaction::decode_core(frozen).map_err(KeyedSponsorRefusal::Decode)?;
    if tx.encode_without_witness() != frozen {
        return Err(KeyedSponsorRefusal::NonCanonicalFrozenBytes);
    }
    if !explicit(spent)
        || tx.outputs().iter().any(|o| !explicit(o))
        || tx.inputs().iter().any(|i| !i.script_sig().is_empty())
    {
        return Err(KeyedSponsorRefusal::UnsupportedForm);
    }
    let input = tx
        .inputs()
        .get(index)
        .ok_or(KeyedSponsorRefusal::InputIndex)?;
    let program = spent.program();
    if program.len() != 2 + WITNESS_V0_KEYHASH_PROGRAM_BYTES
        || program[0] != 0
        || usize::from(program[1]) != WITNESS_V0_KEYHASH_PROGRAM_BYTES
    {
        return Err(KeyedSponsorRefusal::Program);
    }
    let mut prevouts = Vec::new();
    let mut sequences = Vec::new();
    for i in tx.inputs() {
        prevouts.extend_from_slice(i.outpoint().txid().internal());
        prevouts.extend_from_slice(&i.outpoint().index().to_le_bytes());
        sequences.extend_from_slice(&i.sequence().to_le_bytes());
    }
    let mut outputs = Vec::new();
    for output in tx.outputs() {
        output.encode(&mut outputs);
    }
    let mut code = vec![0x76, 0xa9, program[1]]; // P2PKH scriptCode from the bound witness program.
    code.extend_from_slice(&program[2..]);
    code.extend_from_slice(&[0x88, 0xac]);
    let mut message = Vec::new();
    message.extend_from_slice(&tx.version().to_le_bytes()); // frozen version
    message.extend_from_slice(&hash256(&prevouts)); // every frozen outpoint, ordered
    message.extend_from_slice(&hash256(&sequences)); // every frozen sequence, ordered
    message.extend_from_slice(&hash256(&vec![0; tx.inputs().len()])); // one null issuance byte per input
    message.extend_from_slice(input.outpoint().txid().internal()); // selected frozen outpoint
    message.extend_from_slice(&input.outpoint().index().to_le_bytes());
    message.extend_from_slice(&compact_size(code.len() as u64)); // framed scriptCode
    message.extend_from_slice(&code);
    encode_value_field(&mut message, spent.value()); // authenticated parent's explicit amount
    message.extend_from_slice(&input.sequence().to_le_bytes()); // selected frozen sequence
    // No selected issuance fields: decoder forbids issuance on every input.
    message.extend_from_slice(&hash256(&outputs)); // every frozen serialized output
    message.extend_from_slice(&tx.lock_time().to_le_bytes());
    message.extend_from_slice(&1_u32.to_le_bytes()); // SIGHASH_ALL, no extra flags
    Ok(hash256(&message))
}

/// Verify the admitted compressed-key, strict-DER low-S P2WPKH witness.
/// No secret is accepted, and no signing operation is called.
/// # Errors
/// Preserves the digest refusal or reports the exact witness/cryptographic boundary.
pub fn verify_keyed_sponsor(
    frozen: &[u8],
    index: usize,
    spent: &TargetOutput,
    witness: &[Vec<u8>],
) -> Result<(), KeyedSponsorRefusal> {
    let _span = tracing::debug_span!(target: "transaction::keyed_sponsor", "verify_keyed_sponsor", family = "keyed").entered();
    let result = verify_core(frozen, index, spent, witness);
    match &result {
        Ok(()) => {
            tracing::debug!(target: "transaction::keyed_sponsor", family = "keyed", outcome = "completed", "keyed sponsor verification completed");
        }
        Err(e) => {
            tracing::debug!(target: "transaction::keyed_sponsor", family = "keyed", outcome = "refused", refusal_kind = e.refusal_kind(), "keyed sponsor verification refused");
        }
    }
    result
}
pub(crate) fn verify_core(
    frozen: &[u8],
    index: usize,
    spent: &TargetOutput,
    witness: &[Vec<u8>],
) -> Result<(), KeyedSponsorRefusal> {
    let digest = digest_core(frozen, index, spent)?;
    if witness.len() != WITNESS_V0_KEYHASH_STACK_ITEMS {
        return Err(KeyedSponsorRefusal::WitnessShape);
    }
    let [signature, key] = witness else {
        return Err(KeyedSponsorRefusal::WitnessShape);
    };
    if signature.is_empty() || signature.len() > 72 {
        return Err(KeyedSponsorRefusal::SignatureLength);
    }
    if signature.last() != Some(&1) {
        return Err(KeyedSponsorRefusal::HashType);
    }
    let der = &signature[..signature.len() - 1];
    let sig = Signature::from_der(der).map_err(|_| KeyedSponsorRefusal::StrictDer)?;
    if sig.serialize_der().as_ref() != der {
        return Err(KeyedSponsorRefusal::StrictDer);
    }
    let mut normalized = sig;
    normalized.normalize_s();
    if normalized != sig {
        return Err(KeyedSponsorRefusal::HighS);
    }
    if key.len() != 33 || !matches!(key[0], 2 | 3) {
        return Err(KeyedSponsorRefusal::CompressedPublicKey);
    }
    let public =
        PublicKey::from_slice(key).map_err(|_| KeyedSponsorRefusal::CompressedPublicKey)?;
    if hash160::Hash::hash(key).to_byte_array().as_slice() != &spent.program()[2..] {
        return Err(KeyedSponsorRefusal::PublicKeyHash);
    }
    Secp256k1::verification_only()
        .verify_ecdsa(&Message::from_digest(digest), &sig, &public)
        .map_err(|_| KeyedSponsorRefusal::InvalidSignature)
}
