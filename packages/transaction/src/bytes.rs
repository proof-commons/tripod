//! The first-party explicit-field target transaction, and its bytes.
//!
//! # Why this exists at all
//!
//! The reviewed substrate decision selected first-party structures over
//! the third-party library it reviewed, and named the four capabilities
//! this workspace would then have to own. Two of them are here: encode
//! an explicit-field transaction to exact target bytes, and decode
//! target bytes back into validated typed values. Nothing in
//! `target-elements` does either — that crate reviews what a *program*
//! does and what a *form* must look like, not how a transaction
//! serializes.
//!
//! # Every constant is transcribed from the target's own serializer
//!
//! The provenance is recorded beside each rule rather than in one
//! paragraph at the top, because a serializer is a list of independent
//! decisions and a reader checking one of them should not have to
//! reconstruct the rest. The reviewed checkout's
//! `src/primitives/transaction.h`, `src/primitives/confidential.h`, and
//! `src/primitives/txwitness.h` are the whole source.
//!
//! # Explicit fields, and a typed refusal for everything else
//!
//! The Phase-4 representation is explicit, so the structures here carry
//! explicit assets and explicit values, plus the one commitment the
//! candidate genuinely admits: a sponsor input's own value may stay
//! confidential, and a sponsor change output may therefore carry a
//! value commitment. A field this crate cannot construct is a typed
//! refusal rather than a silently dropped one, and a decoded byte
//! string carrying an issuance, a peg-in, a range proof, or a
//! surjection proof is refused rather than truncated: §1.11 forbids
//! returning a partial result, and a decoder that ignored a field would
//! be returning one.

use std::fmt;
use std::fmt::Write as _;
use std::str::FromStr;

use crate::error::TransactionRefusal;

/// How many bytes a transaction identifier occupies.
pub const TXID_BYTES: usize = 32;

/// How many bytes an explicit asset identifier occupies.
pub const ASSET_ID_BYTES: usize = 32;

/// How many bytes a field commitment occupies.
pub const COMMITMENT_BYTES: usize = 33;

/// The prefix byte a null field carries.
///
/// Provenance: the `case 0` branch of
/// `CConfidentialCommitment::Unserialize`
/// (`src/primitives/confidential.h`), which clears the field and reads
/// nothing further.
pub const NULL_PREFIX: u8 = 0x00;

/// The prefix byte an explicit field carries.
///
/// Provenance: the `case 1` branch of the same function, which resizes
/// to the field's explicit width.
pub const EXPLICIT_PREFIX: u8 = 0x01;

/// The prefix bytes a value commitment may carry.
///
/// Provenance: `CConfidentialValue : CConfidentialCommitment<9, 8, 9>`
/// (`src/primitives/confidential.h`), whose two template prefixes are
/// these bytes.
pub const VALUE_COMMITMENT_PREFIXES: [u8; 2] = [0x08, 0x09];

/// The prefix bytes an asset commitment may carry.
///
/// Provenance: `CConfidentialAsset : CConfidentialCommitment<33, 10,
/// 11>`. Recorded for the decoder's benefit; this crate constructs no
/// asset commitment, because the sponsor profile forces every sponsor
/// asset explicit and the closed protocol asset is explicit by policy.
pub const ASSET_COMMITMENT_PREFIXES: [u8; 2] = [0x0a, 0x0b];

/// The prefix bytes a nonce commitment may carry.
///
/// Provenance: `CConfidentialNonce : CConfidentialCommitment<33, 2, 3>`.
pub const NONCE_COMMITMENT_PREFIXES: [u8; 2] = [0x02, 0x03];

/// The flag byte a transaction with no witness carries.
///
/// Provenance: `SerializeTransaction`
/// (`src/primitives/transaction.h`), whose `flags` is zero unless the
/// transaction has a witness. In the target's own transaction mode the
/// byte is written unconditionally, which is why a witnessless
/// transaction still carries one — unlike the upstream Bitcoin
/// encoding, where the marker is present only in the extended form.
pub const NO_WITNESS_FLAG: u8 = 0x00;

/// The flag byte a transaction carrying a witness carries.
pub const WITNESS_FLAG: u8 = 0x01;

/// The outpoint index bit marking an issuance.
///
/// Provenance: `COutPoint::OUTPOINT_ISSUANCE_FLAG`.
pub const OUTPOINT_ISSUANCE_FLAG: u32 = 1 << 31;

/// The outpoint index bit marking a peg-in.
///
/// Provenance: `COutPoint::OUTPOINT_PEGIN_FLAG`.
pub const OUTPOINT_PEGIN_FLAG: u32 = 1 << 30;

/// The bits of an outpoint index that mean the index.
///
/// Provenance: `COutPoint::OUTPOINT_INDEX_MASK`.
pub const OUTPOINT_INDEX_MASK: u32 = 0x3fff_ffff;

/// The factor a stripped transaction's bytes are scaled by.
///
/// Provenance: `WITNESS_SCALE_FACTOR` (`src/consensus/consensus.h`),
/// used by `GetTransactionWeight` (`src/consensus/validation.h`).
pub const WITNESS_SCALE_FACTOR: u64 = 4;

// --- Compact size -----------------------------------------------------

/// Encode the target's variable-width count prefix.
///
/// Provenance: `WriteCompactSize` (`src/serialize.h`). The four widths
/// are written out rather than derived, because the boundaries are the
/// part a reader checks.
#[must_use]
pub fn compact_size(value: u64) -> Vec<u8> {
    if value < 0xfd {
        return vec![u8::try_from(value).unwrap_or_default()];
    }
    if u16::try_from(value).is_ok() {
        let mut bytes = vec![0xfd];
        bytes.extend_from_slice(&u16::try_from(value).unwrap_or(u16::MAX).to_le_bytes());
        return bytes;
    }
    if u32::try_from(value).is_ok() {
        let mut bytes = vec![0xfe];
        bytes.extend_from_slice(&u32::try_from(value).unwrap_or(u32::MAX).to_le_bytes());
        return bytes;
    }
    let mut bytes = vec![0xff];
    bytes.extend_from_slice(&value.to_le_bytes());
    bytes
}

/// A cursor over target bytes that refuses rather than panicking.
struct Reader<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl<'bytes> Reader<'bytes> {
    const fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'bytes [u8], TransactionRefusal> {
        let end = self
            .at
            .checked_add(count)
            .ok_or(TransactionRefusal::TruncatedTargetBytes { at: self.at })?;
        if end > self.bytes.len() {
            return Err(TransactionRefusal::TruncatedTargetBytes { at: self.at });
        }
        let taken = &self.bytes[self.at..end];
        self.at = end;
        Ok(taken)
    }

    fn byte(&mut self) -> Result<u8, TransactionRefusal> {
        Ok(self.take(1)?[0])
    }

    fn u32_le(&mut self) -> Result<u32, TransactionRefusal> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u64_be(&mut self) -> Result<u64, TransactionRefusal> {
        let bytes = self.take(8)?;
        let mut value = [0_u8; 8];
        value.copy_from_slice(bytes);
        Ok(u64::from_be_bytes(value))
    }

    /// Read a compact size, refusing a non-minimal encoding.
    ///
    /// Provenance: `ReadCompactSize` (`src/serialize.h`) refuses a
    /// value encoded in a wider form than it needs. The refusal is kept
    /// rather than relaxed: a decoder that accepted both spellings
    /// would accept two byte strings for one transaction, and the
    /// candidate's determinism claim is a claim about bytes.
    fn compact_size(&mut self) -> Result<u64, TransactionRefusal> {
        let at = self.at;
        let first = self.byte()?;
        let (value, minimum) = match first {
            0xfd => {
                let bytes = self.take(2)?;
                (u64::from(u16::from_le_bytes([bytes[0], bytes[1]])), 0xfd)
            }
            0xfe => {
                let bytes = self.take(4)?;
                (
                    u64::from(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
                    0x1_0000,
                )
            }
            0xff => {
                let bytes = self.take(8)?;
                let mut value = [0_u8; 8];
                value.copy_from_slice(bytes);
                (u64::from_le_bytes(value), 0x1_0000_0000)
            }
            _ => (u64::from(first), 0),
        };
        if value < minimum {
            return Err(TransactionRefusal::NonMinimalCompactSize { at, value });
        }
        Ok(value)
    }

    fn length_prefixed(&mut self) -> Result<Vec<u8>, TransactionRefusal> {
        let length = self.compact_size()?;
        let length = usize::try_from(length)
            .map_err(|_| TransactionRefusal::TruncatedTargetBytes { at: self.at })?;
        Ok(self.take(length)?.to_vec())
    }

    const fn exhausted(&self) -> bool {
        self.at == self.bytes.len()
    }
}

// --- Fields -----------------------------------------------------------

/// An explicit asset identifier, in the target's internal byte order.
///
/// Internal order, not display order. The target serializes the 32
/// bytes as they are stored and reverses them only when rendering hex
/// for a human, so a constant transcribed from a block explorer is the
/// reverse of what belongs here. The distinction is invisible in the
/// bytes and is the single easiest way to build a transaction that is
/// well formed and refers to the wrong asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetId([u8; ASSET_ID_BYTES]);

impl AssetId {
    /// The identifier these internal-order bytes name.
    #[must_use]
    pub const fn from_internal(bytes: [u8; ASSET_ID_BYTES]) -> Self {
        Self(bytes)
    }

    /// The identifier these bytes name, if they are the right width.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MalformedAssetIdentifier`] for any other
    /// width.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, TransactionRefusal> {
        <[u8; ASSET_ID_BYTES]>::try_from(bytes).map_or(
            Err(TransactionRefusal::MalformedAssetIdentifier {
                offered: bytes.len(),
            }),
            |bytes| Ok(Self(bytes)),
        )
    }

    /// The internal-order bytes.
    #[must_use]
    pub const fn internal(&self) -> &[u8; ASSET_ID_BYTES] {
        &self.0
    }
}

/// A transaction identifier, in the target's internal byte order.
///
/// The same order caveat as [`AssetId`], for the same reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Txid([u8; TXID_BYTES]);

/// Why target-display text could not name a transaction identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionIdentityParseError {
    /// A target transaction identity is exactly 32 displayed bytes.
    WrongLength {
        /// How many bytes the caller offered.
        offered: usize,
    },
    /// One displayed byte is not an ASCII hexadecimal digit.
    NonHexDigit {
        /// The zero-based byte index of the invalid digit.
        index: usize,
    },
}

impl fmt::Display for TransactionIdentityParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { offered } => {
                write!(
                    formatter,
                    "transaction identity has {offered} bytes, expected 64"
                )
            }
            Self::NonHexDigit { index } => {
                write!(
                    formatter,
                    "transaction identity has a non-hex digit at byte {index}"
                )
            }
        }
    }
}

impl std::error::Error for TransactionIdentityParseError {}

impl Txid {
    /// The identifier these internal-order bytes name.
    #[must_use]
    pub const fn from_internal(bytes: [u8; TXID_BYTES]) -> Self {
        Self(bytes)
    }

    /// The internal-order bytes.
    #[must_use]
    pub const fn internal(&self) -> &[u8; TXID_BYTES] {
        &self.0
    }

    /// Parse the target's 64-digit transaction-identity display form.
    ///
    /// The target displays the internal bytes in reverse order. Either
    /// ASCII hexadecimal case is accepted; rendering is canonical
    /// lowercase.
    ///
    /// # Errors
    ///
    /// [`TransactionIdentityParseError::WrongLength`] unless `text` is
    /// exactly 64 bytes, or [`TransactionIdentityParseError::NonHexDigit`]
    /// at the first byte that is not an ASCII hexadecimal digit.
    pub fn from_target_display(text: &str) -> Result<Self, TransactionIdentityParseError> {
        if text.len() != TXID_BYTES * 2 {
            return Err(TransactionIdentityParseError::WrongLength {
                offered: text.len(),
            });
        }

        let displayed = text.as_bytes();
        let mut internal = [0_u8; TXID_BYTES];
        for (displayed_index, (internal_byte, pair)) in internal
            .iter_mut()
            .rev()
            .zip(displayed.chunks_exact(2))
            .enumerate()
        {
            let high_index = displayed_index * 2;
            let low_index = high_index + 1;
            let high = hex_nibble(pair[0])
                .ok_or(TransactionIdentityParseError::NonHexDigit { index: high_index })?;
            let low = hex_nibble(pair[1])
                .ok_or(TransactionIdentityParseError::NonHexDigit { index: low_index })?;
            *internal_byte = (high << 4) | low;
        }
        Ok(Self(internal))
    }

    /// Render the target's canonical lowercase transaction identity.
    #[must_use]
    pub fn to_target_display(&self) -> String {
        let mut displayed = String::with_capacity(TXID_BYTES * 2);
        for byte in self.0.iter().rev() {
            let _ = write!(displayed, "{byte:02x}");
        }
        displayed
    }
}

impl FromStr for Txid {
    type Err = TransactionIdentityParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::from_target_display(text)
    }
}

impl fmt::Display for Txid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter().rev() {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// One previous output an input spends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Outpoint {
    txid: Txid,
    index: u32,
}

impl Outpoint {
    /// The outpoint naming `index` of `txid`.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::OutpointIndexOutOfRange`] when the index
    /// occupies a bit the target reserves for the issuance or peg-in
    /// marker. Such an index is unreachable in practice — it would need
    /// a parent transaction with over a billion outputs — and refusing
    /// it here is what keeps the marker bits meaning what they mean.
    pub const fn new(txid: Txid, index: u32) -> Result<Self, TransactionRefusal> {
        if index & !OUTPOINT_INDEX_MASK != 0 {
            return Err(TransactionRefusal::OutpointIndexOutOfRange { offered: index });
        }
        Ok(Self { txid, index })
    }

    /// The transaction the previous output belongs to.
    #[must_use]
    pub const fn txid(&self) -> Txid {
        self.txid
    }

    /// The index of the previous output.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
}

/// An output's asset field.
///
/// The commitment variant exists for the decoder. This crate
/// constructs only the explicit one: the closed protocol asset is
/// explicit under the fixed Phase-4 representation, and the reviewed
/// sponsor profile forces a sponsor input's asset explicit so that the
/// coordinator can authenticate it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AssetField {
    /// An explicit asset identifier.
    Explicit(AssetId),
    /// A blinded asset commitment.
    Commitment([u8; COMMITMENT_BYTES]),
}

/// An output's value field.
///
/// Sponsor value opacity is what the commitment variant is for, and it
/// is the reason this is an enumeration rather than an amount: the
/// sponsor-erasure law forbids the protocol relation from depending on
/// a sponsor's individual amount, and a builder that could only
/// represent explicit values would have made confidential sponsor
/// change unrepresentable rather than merely unused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ValueField {
    /// An explicit amount.
    Explicit(u64),
    /// A blinded value commitment.
    Commitment([u8; COMMITMENT_BYTES]),
}

/// One asset field, serialized as the target writes it.
///
/// Extracted from the output encoder rather than written twice, because
/// the owner message's spent-asset-and-amount term hashes the asset and
/// value fields *alone* — without the nonce and script that follow them
/// in an output (`src/script/interpreter.cpp:2454-2462`) — so the two
/// callers need the same field encoding at two different granularities.
/// Two spellings of one encoding is one spelling too many.
pub(crate) fn encode_asset_field(bytes: &mut Vec<u8>, asset: AssetField) {
    match asset {
        AssetField::Explicit(asset) => {
            bytes.push(EXPLICIT_PREFIX);
            bytes.extend_from_slice(asset.internal());
        }
        AssetField::Commitment(commitment) => bytes.extend_from_slice(&commitment),
    }
}

/// One value field, serialized as the target writes it.
pub(crate) fn encode_value_field(bytes: &mut Vec<u8>, value: ValueField) {
    match value {
        // Big-endian, and only here. An explicit amount is stored
        // big-endian in the transaction field and pushed to a script's
        // stack little-endian, so one value has two orders and neither
        // is the crate's default.
        ValueField::Explicit(amount) => {
            bytes.push(EXPLICIT_PREFIX);
            bytes.extend_from_slice(&amount.to_be_bytes());
        }
        ValueField::Commitment(commitment) => bytes.extend_from_slice(&commitment),
    }
}

/// An output's nonce field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NonceField {
    /// No nonce, which is the whole field for an unblinded output.
    Null,
    /// An ephemeral key commitment.
    Commitment([u8; COMMITMENT_BYTES]),
}

// --- Inputs, outputs, witnesses ---------------------------------------

/// One input of a candidate transaction.
///
/// No issuance and no peg-in: both are outside the candidate pipeline,
/// and neither has a field here to be set. A byte string carrying one
/// is refused by the decoder rather than represented.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetInput {
    outpoint: Outpoint,
    script_sig: Vec<u8>,
    sequence: u32,
}

impl TargetInput {
    /// The input spending `outpoint` under `sequence`.
    ///
    /// The signature script is empty for every input the candidate
    /// constructs — an ASH input is a script-path taproot spend and a
    /// sponsor input is a native witness program, and neither carries
    /// one — so it is not a parameter. A field nobody can set is a
    /// field nobody can set wrongly.
    #[must_use]
    pub const fn new(outpoint: Outpoint, sequence: u32) -> Self {
        Self {
            outpoint,
            script_sig: Vec::new(),
            sequence,
        }
    }

    /// The previous output this input spends.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The signature script, which the candidate leaves empty.
    #[must_use]
    pub fn script_sig(&self) -> &[u8] {
        &self.script_sig
    }

    /// The sequence field.
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(self.outpoint.txid.internal());
        bytes.extend_from_slice(&self.outpoint.index.to_le_bytes());
        bytes.extend_from_slice(&compact_size(self.script_sig.len() as u64));
        bytes.extend_from_slice(&self.script_sig);
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
    }

    fn decode(reader: &mut Reader<'_>) -> Result<Self, TransactionRefusal> {
        let hash = reader.take(TXID_BYTES)?;
        let mut txid = [0_u8; TXID_BYTES];
        txid.copy_from_slice(hash);
        let raw = reader.u32_le()?;
        if raw & OUTPOINT_ISSUANCE_FLAG != 0 {
            return Err(TransactionRefusal::IssuanceInputRefused);
        }
        if raw & OUTPOINT_PEGIN_FLAG != 0 {
            return Err(TransactionRefusal::PeginInputRefused);
        }
        let outpoint = Outpoint::new(Txid::from_internal(txid), raw)?;
        let script_sig = reader.length_prefixed()?;
        let sequence = reader.u32_le()?;
        Ok(Self {
            outpoint,
            script_sig,
            sequence,
        })
    }
}

/// One output of a candidate transaction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetOutput {
    asset: AssetField,
    value: ValueField,
    nonce: NonceField,
    program: Vec<u8>,
}

impl TargetOutput {
    /// The output carrying these fields and this program.
    #[must_use]
    pub const fn new(
        asset: AssetField,
        value: ValueField,
        nonce: NonceField,
        program: Vec<u8>,
    ) -> Self {
        Self {
            asset,
            value,
            nonce,
            program,
        }
    }

    /// The asset field.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The value field.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The nonce field.
    #[must_use]
    pub const fn nonce(&self) -> NonceField {
        self.nonce
    }

    /// The output program.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }

    /// Whether this output satisfies the reviewed fee-role test.
    ///
    /// Every conjunct of `CTxOut::IsFee`, and not one of them is an
    /// amount comparison. The method exists so that a caller checking
    /// the role checks the target's own test rather than the position
    /// the ABI happens to put it in.
    #[must_use]
    pub const fn is_fee(&self) -> bool {
        self.program.is_empty()
            && matches!(self.value, ValueField::Explicit(_))
            && matches!(self.asset, AssetField::Explicit(_))
    }

    /// This output, serialized as the target writes it.
    ///
    /// Crate-visible because the owner message's outputs term hashes
    /// each output in exactly this form
    /// (`src/script/interpreter.cpp:2443-2450`), and a message
    /// construction that re-spelled the output encoding would be a
    /// second opinion about it, free to drift from the bytes this crate
    /// actually produces.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        encode_asset_field(bytes, self.asset);
        encode_value_field(bytes, self.value);
        match self.nonce {
            NonceField::Null => bytes.push(NULL_PREFIX),
            NonceField::Commitment(commitment) => bytes.extend_from_slice(&commitment),
        }
        bytes.extend_from_slice(&compact_size(self.program.len() as u64));
        bytes.extend_from_slice(&self.program);
    }

    fn decode(reader: &mut Reader<'_>) -> Result<Self, TransactionRefusal> {
        let asset = match reader.byte()? {
            EXPLICIT_PREFIX => {
                AssetField::Explicit(AssetId::from_slice(reader.take(ASSET_ID_BYTES)?)?)
            }
            prefix if ASSET_COMMITMENT_PREFIXES.contains(&prefix) => {
                AssetField::Commitment(commitment(prefix, reader)?)
            }
            prefix => return Err(TransactionRefusal::UnrecognizedFieldPrefix { prefix }),
        };
        let value = match reader.byte()? {
            EXPLICIT_PREFIX => ValueField::Explicit(reader.u64_be()?),
            prefix if VALUE_COMMITMENT_PREFIXES.contains(&prefix) => {
                ValueField::Commitment(commitment(prefix, reader)?)
            }
            prefix => return Err(TransactionRefusal::UnrecognizedFieldPrefix { prefix }),
        };
        let nonce = match reader.byte()? {
            NULL_PREFIX => NonceField::Null,
            prefix if NONCE_COMMITMENT_PREFIXES.contains(&prefix) => {
                NonceField::Commitment(commitment(prefix, reader)?)
            }
            prefix => return Err(TransactionRefusal::UnrecognizedFieldPrefix { prefix }),
        };
        let program = reader.length_prefixed()?;
        Ok(Self {
            asset,
            value,
            nonce,
            program,
        })
    }
}

/// Read one output's witness, refusing a proof its value form forbids
/// and a proof its value form requires and did not get.
///
/// The condition is the output's own value field and not a decoder
/// parameter, because the target's condition is the output's own value
/// field: `VerifyConfidentialPair` reaches the rangeproof check exactly
/// where the value is a commitment, and a byte string says which of the
/// two it carries. A decoder that took the form as an argument would be
/// able to be told the wrong one.
///
/// The two refusals are asymmetric, and deliberately so. The surjection
/// field is refused unconditionally, because the only confidential form
/// this workspace constructs pairs a committed value with an EXPLICIT
/// asset — the unblinded generator case, where the target requires the
/// surjection field empty — and the blinded-asset form that would carry
/// one is not a form any code here builds. Refusing it is therefore
/// already the form-conditional answer for every form reachable at this
/// tree, and widening it to admit a surjection proof would be admitting
/// a form nothing can construct.
fn output_witness(
    reader: &mut Reader<'_>,
    index: usize,
    value: ValueField,
) -> Result<OutputWitness, TransactionRefusal> {
    let surjection_proof = reader.length_prefixed()?;
    if !surjection_proof.is_empty() {
        return Err(TransactionRefusal::SurjectionProofRefused);
    }
    let range_proof = reader.length_prefixed()?;
    match value {
        ValueField::Explicit(_) => {
            if !range_proof.is_empty() {
                return Err(TransactionRefusal::RangeProofRefused);
            }
        }
        ValueField::Commitment(_) => {
            if range_proof.is_empty() {
                return Err(TransactionRefusal::RangeProofRequired { output: index });
            }
        }
    }
    Ok(OutputWitness::new(surjection_proof, range_proof))
}

/// Read the 32 bytes following a commitment's prefix.
fn commitment(
    prefix: u8,
    reader: &mut Reader<'_>,
) -> Result<[u8; COMMITMENT_BYTES], TransactionRefusal> {
    let mut bytes = [0_u8; COMMITMENT_BYTES];
    bytes[0] = prefix;
    bytes[1..].copy_from_slice(reader.take(COMMITMENT_BYTES - 1)?);
    Ok(bytes)
}

/// One input's witness stack.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputWitness {
    stack: Vec<Vec<u8>>,
}

impl InputWitness {
    /// The witness carrying this stack, bottom item first.
    #[must_use]
    pub const fn new(stack: Vec<Vec<u8>>) -> Self {
        Self { stack }
    }

    /// The stack, bottom item first.
    #[must_use]
    pub fn stack(&self) -> &[Vec<u8>] {
        &self.stack
    }

    /// Whether this witness carries nothing.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.stack.is_empty()
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        // The two issuance proofs and the peg-in stack are empty for
        // every input this crate builds, and each is still written as
        // its own empty length prefix: the target reads four fields per
        // input witness, so three of them being empty is three bytes
        // rather than none.
        bytes.push(NULL_PREFIX);
        bytes.push(NULL_PREFIX);
        bytes.extend_from_slice(&compact_size(self.stack.len() as u64));
        for item in &self.stack {
            bytes.extend_from_slice(&compact_size(item.len() as u64));
            bytes.extend_from_slice(item);
        }
        bytes.push(NULL_PREFIX);
    }

    fn decode(reader: &mut Reader<'_>) -> Result<Self, TransactionRefusal> {
        if !reader.length_prefixed()?.is_empty() {
            return Err(TransactionRefusal::IssuanceProofRefused);
        }
        if !reader.length_prefixed()?.is_empty() {
            return Err(TransactionRefusal::IssuanceProofRefused);
        }
        let count = reader.compact_size()?;
        let count = usize::try_from(count)
            .map_err(|_| TransactionRefusal::TruncatedTargetBytes { at: 0 })?;
        let mut stack = Vec::with_capacity(count.min(1024));
        for _ in 0..count {
            stack.push(reader.length_prefixed()?);
        }
        if !reader.length_prefixed()?.is_empty() {
            return Err(TransactionRefusal::PeginInputRefused);
        }
        Ok(Self { stack })
    }
}

/// One output's witness: a surjection proof and a range proof.
///
/// A transaction-level vector in the target's serialization rather than
/// a field of [`TargetOutput`], and modelled that way here for the same
/// reason: `CTxWitness` holds `vtxoutwit` beside `vtxinwit`, the
/// target's own signer hashes the vector at whatever length it happens
/// to have, and putting the proofs inside the output would misplace the
/// thing whose *length* is the hazard.
///
/// Both members are empty for every explicit-form output, and an
/// all-empty witness serializes to the two empty length prefixes the
/// encoder used to write unconditionally — so an explicit candidate's
/// bytes are unchanged by this type existing.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputWitness {
    surjection_proof: Vec<u8>,
    range_proof: Vec<u8>,
}

impl OutputWitness {
    /// The witness carrying these two proofs.
    #[must_use]
    pub const fn new(surjection_proof: Vec<u8>, range_proof: Vec<u8>) -> Self {
        Self {
            surjection_proof,
            range_proof,
        }
    }

    /// The witness carrying a range proof and an empty surjection
    /// proof, which is the hybrid form's whole shape.
    ///
    /// Named rather than left to [`Self::new`] with an empty vector,
    /// because the empty surjection field is a target requirement for a
    /// confidential value paired with an explicit asset and not a
    /// convenience: `VerifyRangeProof` reaches the value only where the
    /// asset generator is unblinded.
    #[must_use]
    pub const fn range_proof_only(range_proof: Vec<u8>) -> Self {
        Self {
            surjection_proof: Vec::new(),
            range_proof,
        }
    }

    /// The witness carrying nothing, which is every explicit output's.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            surjection_proof: Vec::new(),
            range_proof: Vec::new(),
        }
    }

    /// The surjection proof, empty for every form this crate builds.
    #[must_use]
    pub fn surjection_proof(&self) -> &[u8] {
        &self.surjection_proof
    }

    /// The range proof, nonempty for a confidential value.
    #[must_use]
    pub fn range_proof(&self) -> &[u8] {
        &self.range_proof
    }

    /// Whether this witness carries nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.surjection_proof.is_empty() && self.range_proof.is_empty()
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&compact_size(self.surjection_proof.len() as u64));
        bytes.extend_from_slice(&self.surjection_proof);
        bytes.extend_from_slice(&compact_size(self.range_proof.len() as u64));
        bytes.extend_from_slice(&self.range_proof);
    }
}

// --- The transaction --------------------------------------------------

/// One candidate target transaction.
///
/// The witness census is positional and total on both sides: one
/// [`InputWitness`] per input and one [`OutputWitness`] per output.
/// Both are written when the transaction carries any witness at all,
/// which is what the target's own serializer does after resizing the
/// two witness vectors to the input and output counts.
///
/// That resizing is exactly what the target's own wallet signer misses
/// `(´[PLAN-obs:upstream:eg-019]´)`: it precomputes a taproot digest
/// over the ungrown vectors and calls the result complete. This crate
/// writes the grown form, so the bytes it produces commit to the digest
/// consensus checks; a signer corrected upstream would agree with them
/// rather than needing to be worked around.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetTransaction {
    version: u32,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    lock_time: u32,
    witnesses: Vec<InputWitness>,
    output_witnesses: Vec<OutputWitness>,
}

impl TargetTransaction {
    /// The transaction carrying these roles.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::EmptyInputCensus`] or
    /// [`TransactionRefusal::EmptyOutputCensus`] for a transaction with
    /// no input or no output, and
    /// [`TransactionRefusal::WitnessCensusMismatch`] when the witness
    /// census is not one entry per input. The last is the check that
    /// keeps the positional witness encoding honest: a witness list of
    /// a different length would serialize a witness against the wrong
    /// input and still produce well-formed bytes.
    pub fn new(
        version: u32,
        inputs: Vec<TargetInput>,
        outputs: Vec<TargetOutput>,
        lock_time: u32,
        witnesses: Vec<InputWitness>,
    ) -> Result<Self, TransactionRefusal> {
        if inputs.is_empty() {
            return Err(TransactionRefusal::EmptyInputCensus);
        }
        if outputs.is_empty() {
            return Err(TransactionRefusal::EmptyOutputCensus);
        }
        let output_witnesses = vec![OutputWitness::empty(); outputs.len()];
        Self::with_output_witnesses(
            version,
            inputs,
            outputs,
            lock_time,
            witnesses,
            output_witnesses,
        )
    }

    /// The transaction carrying these roles and these output witnesses.
    ///
    /// Separate from [`Self::new`] rather than a sixth parameter on it,
    /// because every caller outside the confidential lane builds
    /// explicit outputs whose witnesses are empty, and a constructor
    /// that made all of them pass an all-empty vector would have made
    /// the proof-free case say something about proofs.
    ///
    /// # Errors
    ///
    /// Everything [`Self::new`] refuses, plus
    /// [`TransactionRefusal::OutputWitnessCensusMismatch`] when the
    /// output-witness census is not one entry per output. The reason is
    /// the input side's exactly: the encoding is positional, so a
    /// shorter or longer vector would bind a proof to the wrong output
    /// and still produce well-formed bytes.
    pub fn with_output_witnesses(
        version: u32,
        inputs: Vec<TargetInput>,
        outputs: Vec<TargetOutput>,
        lock_time: u32,
        witnesses: Vec<InputWitness>,
        output_witnesses: Vec<OutputWitness>,
    ) -> Result<Self, TransactionRefusal> {
        if inputs.is_empty() {
            return Err(TransactionRefusal::EmptyInputCensus);
        }
        if outputs.is_empty() {
            return Err(TransactionRefusal::EmptyOutputCensus);
        }
        if witnesses.len() != inputs.len() {
            return Err(TransactionRefusal::WitnessCensusMismatch {
                inputs: inputs.len(),
                witnesses: witnesses.len(),
            });
        }
        if output_witnesses.len() != outputs.len() {
            return Err(TransactionRefusal::OutputWitnessCensusMismatch {
                outputs: outputs.len(),
                output_witnesses: output_witnesses.len(),
            });
        }
        Ok(Self {
            version,
            inputs,
            outputs,
            lock_time,
            witnesses,
            output_witnesses,
        })
    }

    /// The transaction version.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Every input, in position order.
    #[must_use]
    pub fn inputs(&self) -> &[TargetInput] {
        &self.inputs
    }

    /// Every output, in position order.
    #[must_use]
    pub fn outputs(&self) -> &[TargetOutput] {
        &self.outputs
    }

    /// The lock-time field.
    #[must_use]
    pub const fn lock_time(&self) -> u32 {
        self.lock_time
    }

    /// Every input's witness, in position order.
    #[must_use]
    pub fn witnesses(&self) -> &[InputWitness] {
        &self.witnesses
    }

    /// Every output's witness, in position order.
    #[must_use]
    pub fn output_witnesses(&self) -> &[OutputWitness] {
        &self.output_witnesses
    }

    /// Whether any witness carries anything.
    ///
    /// Provenance: `CTxWitness::IsNull`, which the target's serializer
    /// consults through `HasWitness` before setting the flag byte. A
    /// transaction whose every witness is empty must be serialized
    /// *without* the witness section; writing an all-empty section is
    /// an error the target asserts on rather than tolerates.
    ///
    /// `IsNull` is a conjunction over BOTH vectors, and so is this. A
    /// confidential funding transaction may legitimately carry null
    /// input witnesses — nothing has signed it yet — and range proofs
    /// its outputs are invalid without; answering `false` there would
    /// serialize it without the section that carries its proofs, which
    /// is a different transaction rather than a smaller encoding of the
    /// same one.
    ///
    /// The section's absence is not free downstream: an issuance in a
    /// witnessless transaction is refused as a balance failure
    /// `(´[PLAN-obs:upstream:eg-021]´)`, so what this predicate answers
    /// decides whether such a transaction can be funded at all.
    #[must_use]
    pub fn has_witness(&self) -> bool {
        self.witnesses.iter().any(|witness| !witness.is_null())
            || self
                .output_witnesses
                .iter()
                .any(|witness| !witness.is_empty())
    }

    /// The exact target bytes, witness included.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        self.serialize(self.has_witness())
    }

    /// The exact target bytes with the witness section omitted.
    ///
    /// Not a truncation of [`Self::encode`]: the flag byte differs, so
    /// the two encodings diverge at their fifth byte and one is not a
    /// prefix of the other.
    #[must_use]
    pub fn encode_without_witness(&self) -> Vec<u8> {
        self.serialize(false)
    }

    fn serialize(&self, with_witness: bool) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.push(if with_witness {
            WITNESS_FLAG
        } else {
            NO_WITNESS_FLAG
        });
        bytes.extend_from_slice(&compact_size(self.inputs.len() as u64));
        for input in &self.inputs {
            input.encode(&mut bytes);
        }
        bytes.extend_from_slice(&compact_size(self.outputs.len() as u64));
        for output in &self.outputs {
            output.encode(&mut bytes);
        }
        bytes.extend_from_slice(&self.lock_time.to_le_bytes());
        if with_witness {
            for witness in &self.witnesses {
                witness.encode(&mut bytes);
            }
            bytes.extend_from_slice(&self.output_witness_bytes());
        }
        bytes
    }

    /// The serialized output-witness vector alone, one entry per
    /// output.
    ///
    /// The region the target's `SIGHASH_ALL` hashes and the witnessless
    /// serialization omits. Exposed because the private lane's
    /// protected preimage has to contain it: a signer over bytes that
    /// omit it binds to the empty-vector case the recorded diagnosis
    /// `(´[PLAN-obs:upstream:eg-019]´)` names, and the resulting
    /// signature is complete and invalid.
    ///
    /// Not itself a transaction encoding, and it is not offered as one.
    /// It is one contiguous region of [`Self::encode`], extracted so
    /// that a preimage can be assembled from named regions rather than
    /// by slicing an offset out of a byte string.
    #[must_use]
    pub fn output_witness_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for witness in &self.output_witnesses {
            witness.encode(&mut bytes);
        }
        bytes
    }

    /// The transaction these target bytes encode.
    ///
    /// # Errors
    ///
    /// Every refusal this module's readers raise, plus
    /// [`TransactionRefusal::TrailingTargetBytes`] when the byte string
    /// is longer than the transaction it holds. Trailing bytes are a
    /// refusal rather than a remainder: a decoder that ignored them
    /// would accept two byte strings as one transaction.
    ///
    /// [`TransactionRefusal::SuperfluousWitnessRecord`] for the same
    /// reason, when the witness flag stands over a section carrying
    /// nothing. Between them the two refusals give the decoder its
    /// round-trip law: a transaction this returns re-encodes to exactly
    /// the bytes it was read from.
    pub fn decode(bytes: &[u8]) -> Result<Self, TransactionRefusal> {
        let mut reader = Reader::new(bytes);
        let version = reader.u32_le()?;
        let flags = reader.byte()?;
        let with_witness = match flags {
            NO_WITNESS_FLAG => false,
            WITNESS_FLAG => true,
            offered => return Err(TransactionRefusal::UnrecognizedWitnessFlag { offered }),
        };

        let input_count = reader.compact_size()?;
        let input_count = usize::try_from(input_count)
            .map_err(|_| TransactionRefusal::TruncatedTargetBytes { at: 0 })?;
        let mut inputs = Vec::with_capacity(input_count.min(1024));
        for _ in 0..input_count {
            inputs.push(TargetInput::decode(&mut reader)?);
        }

        let output_count = reader.compact_size()?;
        let output_count = usize::try_from(output_count)
            .map_err(|_| TransactionRefusal::TruncatedTargetBytes { at: 0 })?;
        let mut outputs = Vec::with_capacity(output_count.min(1024));
        for _ in 0..output_count {
            outputs.push(TargetOutput::decode(&mut reader)?);
        }

        let lock_time = reader.u32_le()?;

        let mut witnesses = Vec::with_capacity(inputs.len());
        let mut output_witnesses = Vec::with_capacity(outputs.len());
        if with_witness {
            for _ in &inputs {
                witnesses.push(InputWitness::decode(&mut reader)?);
            }
            for (index, output) in outputs.iter().enumerate() {
                output_witnesses.push(output_witness(&mut reader, index, output.value())?);
            }
        } else {
            witnesses.resize(inputs.len(), InputWitness::default());
            output_witnesses.resize(outputs.len(), OutputWitness::empty());
        }

        // The flag stands over a section, and the section has to carry
        // something. `CTxWitness::IsNull` is a conjunction over both
        // vectors, so the flagged spelling is superfluous exactly when
        // neither vector carries anything — which is what the encoder
        // writes without a section at all. A decoder that accepted the
        // flagged spelling there would hold a value that re-encodes to
        // other bytes than it came from.
        //
        // The condition is a widening rather than a weakening: an
        // all-null input-witness section used to be superfluous
        // unconditionally, and now is superfluous unless some output
        // witness carries a proof. Every byte string the old condition
        // admitted, the new one admits.
        if with_witness
            && witnesses.iter().all(InputWitness::is_null)
            && output_witnesses.iter().all(OutputWitness::is_empty)
        {
            return Err(TransactionRefusal::SuperfluousWitnessRecord);
        }

        if !reader.exhausted() {
            return Err(TransactionRefusal::TrailingTargetBytes { at: reader.at });
        }

        Self::with_output_witnesses(
            version,
            inputs,
            outputs,
            lock_time,
            witnesses,
            output_witnesses,
        )
    }

    /// The serialized bytes of the witness section alone.
    ///
    /// Settles `WitnessBytes`. Computed as the difference between the
    /// two serializations rather than by adding up field widths, which
    /// is the same identity `GetTransactionWeight` relies on and cannot
    /// drift from the encoder.
    #[must_use]
    pub fn witness_bytes(&self) -> u64 {
        let full = self.encode().len() as u64;
        let stripped = self.encode_without_witness().len() as u64;
        full.saturating_sub(stripped)
    }

    /// The transaction's exact weight.
    ///
    /// Provenance: `GetTransactionWeight`
    /// (`src/consensus/validation.h`), which is the stripped size times
    /// three plus the total size. Settles `TransactionWeight`.
    #[must_use]
    pub fn weight(&self) -> u64 {
        let stripped = self.encode_without_witness().len() as u64;
        let total = self.encode().len() as u64;
        stripped
            .saturating_mul(WITNESS_SCALE_FACTOR.saturating_sub(1))
            .saturating_add(total)
    }

    /// The transaction's virtual size, rounding up.
    ///
    /// Provenance: `GetVirtualTransactionSize` (`src/policy/policy.cpp`),
    /// which divides the weight by the scale factor rounding up. Stated
    /// here because every package limit the relay path applies is a
    /// virtual-size limit rather than a weight one.
    #[must_use]
    pub fn virtual_size(&self) -> u64 {
        self.weight()
            .saturating_add(WITNESS_SCALE_FACTOR.saturating_sub(1))
            / WITNESS_SCALE_FACTOR
    }
}
