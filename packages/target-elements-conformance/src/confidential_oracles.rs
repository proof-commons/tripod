//! Reference implementations of the readback decoder and the rangeproof
//! verifier.
//!
//! # What claim class these carry, stated before they are used
//!
//! Both are built on the libraries the target itself vendors. A
//! rangeproof generated or verified through them, and a transaction
//! decoded through them, is CONFORMANCE-TO-THE-TARGET'S-OWN-
//! IMPLEMENTATION evidence. It is not independent evidence, and no
//! report may describe it as independent. The independent claim in this
//! arc belongs to the commitment arithmetic, where the first-party
//! bignum oracle owns it, and nothing here widens that.
//!
//! # Why the decoder is here rather than where transaction bytes are
//! understood
//!
//! The wire's decoder is a trait precisely so it can be implemented
//! outside the package that declares it, and the first-party encoder is
//! the natural home. It cannot be that home yet: at this tree the
//! first-party decoder refuses any nonempty proof field, which is one of
//! the serialization changes a later wave owns and which this wave must
//! not pre-empt. So this wave supplies a reference decoder, marks its
//! claim class, and leaves the first-party implementation to the wave
//! that repairs the encoder.
//!
//! # Nothing here is an expectation
//!
//! The decoder reads the target's observation and the verifier answers a
//! question about it. Neither produces a value the record compares
//! against itself: the record's expectations come from the registered
//! fixture and the first-party oracle, which are different origins.

use elements::encode::{deserialize, serialize};

use secp256k1_zkp::{Generator, PedersenCommitment, RangeProof, Secp256k1, Tag};

use crate::confidential_funding::{
    ConfidentialReadbackDecoder, DecodedAssetField, DecodedFundingOutput,
    DecodedFundingTransaction, DecodedValueField, RangeproofVerifier, ReadbackDecodeRefused,
};

/// The width of an identifier the target prints.
const IDENTIFIER_BYTES: usize = 32;

/// The byte marking an explicit field.
const EXPLICIT_PREFIX: u8 = 1;

/// Lowercase hexadecimal, which is the rendering the target reports in.
fn render_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// One identifier, in the display order the target prints it in.
///
/// The target serializes an identifier in one order and prints it in the
/// reverse of that order. Both spellings are the same value, and the
/// conversion is stated once here rather than at every call.
fn printed_identifier(serialized: &[u8]) -> String {
    let mut reversed = serialized.to_vec();
    reversed.reverse();
    render_hex(&reversed)
}

/// One printed identifier, back in serialization order.
fn serialized_identifier(printed: &str) -> Option<[u8; IDENTIFIER_BYTES]> {
    if printed.len() != IDENTIFIER_BYTES * 2 {
        return None;
    }
    let mut bytes = [0_u8; IDENTIFIER_BYTES];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let pair = printed.get(index * 2..index * 2 + 2)?;
        *slot = u8::from_str_radix(pair, 16).ok()?;
    }
    bytes.reverse();
    Some(bytes)
}

/// Reads mined bytes back into fields, through the target's own library.
///
/// Its claim class is conformance to the target's own implementation.
/// What it establishes is that the bytes a target mined say what the
/// answer said they say, which is a reading of the chain and never an
/// expectation about it.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceReadbackDecoder;

impl ReferenceReadbackDecoder {
    /// One decoder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ConfidentialReadbackDecoder for ReferenceReadbackDecoder {
    fn decode(&self, raw: &[u8]) -> Result<DecodedFundingTransaction, ReadbackDecodeRefused> {
        let transaction: elements::Transaction =
            deserialize(raw).map_err(|_| ReadbackDecodeRefused)?;
        let mut outputs = Vec::with_capacity(transaction.output.len());
        for output in &transaction.output {
            let asset_bytes = serialize(&output.asset);
            let asset = match asset_bytes.split_first() {
                Some((&EXPLICIT_PREFIX, body)) if body.len() == IDENTIFIER_BYTES => {
                    DecodedAssetField::Explicit(printed_identifier(body))
                }
                Some((_, _)) => DecodedAssetField::Commitment(asset_bytes.clone()),
                None => return Err(ReadbackDecodeRefused),
            };
            let value_bytes = serialize(&output.value);
            let value = match value_bytes.split_first() {
                Some((&EXPLICIT_PREFIX, body)) if body.len() == 8 => {
                    let mut amount = [0_u8; 8];
                    amount.copy_from_slice(body);
                    DecodedValueField::Explicit(u64::from_be_bytes(amount))
                }
                Some((_, _)) => DecodedValueField::Commitment(value_bytes.clone()),
                None => return Err(ReadbackDecodeRefused),
            };
            let nonce_bytes = serialize(&output.nonce);
            // A null field serializes to its one prefix byte and carries
            // no point. Reporting it as an empty nonce rather than as a
            // one-byte one is what lets the binding refuse it by width.
            let nonce = if nonce_bytes.len() <= 1 {
                Vec::new()
            } else {
                nonce_bytes
            };
            outputs.push(DecodedFundingOutput {
                asset,
                value,
                nonce,
                program: output.script_pubkey.as_bytes().to_vec(),
                // The proof's own serialization, which is the bytes the
                // witness carries and not the length-prefixed
                // consensus framing of them: the wire member carries
                // the proof, so a reader of the record and a reader of
                // the raw bytes read the same thing.
                surjection_proof: output
                    .witness
                    .surjection_proof
                    .as_ref()
                    .map(|proof| proof.serialize())
                    .unwrap_or_default(),
                rangeproof: output
                    .witness
                    .rangeproof
                    .as_ref()
                    .map(|proof| proof.serialize())
                    .unwrap_or_default(),
            });
        }
        Ok(DecodedFundingTransaction {
            transaction_id: transaction.txid().to_string(),
            witness_transaction_id: transaction.wtxid().to_string(),
            outputs,
        })
    }
}

/// Verifies one rangeproof against what it must bind to.
///
/// The three things are this output's value commitment, the UNBLINDED
/// generator of the explicit asset, and this output's program. A
/// verifier that checked fewer of them would accept a proof built for
/// some other output, which is the whole reason the census member covers
/// binding as well as validity.
///
/// The generator is unblinded because the representation this arc fixes
/// pairs an explicit asset with a confidential value: the asset carries
/// no blinder, so the generator is the asset's own and no surjection
/// proof exists to relate it to anything.
#[derive(Debug)]
pub struct ReferenceRangeproofVerifier {
    context: Secp256k1<secp256k1_zkp::All>,
}

impl Default for ReferenceRangeproofVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceRangeproofVerifier {
    /// One verifier, with its own context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: Secp256k1::new(),
        }
    }
}

impl RangeproofVerifier for ReferenceRangeproofVerifier {
    fn verifies(
        &self,
        proof: &[u8],
        value_commitment: &[u8],
        explicit_asset: &str,
        output_program: &[u8],
    ) -> bool {
        let Some(asset) = serialized_identifier(explicit_asset) else {
            return false;
        };
        let Ok(commitment) = PedersenCommitment::from_slice(value_commitment) else {
            return false;
        };
        let Ok(parsed) = RangeProof::from_slice(proof) else {
            return false;
        };
        let generator = Generator::new_unblinded(&self.context, Tag::from(asset));
        parsed
            .verify(&self.context, commitment, output_program, generator)
            .is_ok()
    }
}
