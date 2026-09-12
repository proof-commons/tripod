//! Canonical representation of semantic STATE metadata.
//!
//! The encoding fixes its domain separator, schema revision, field
//! order, field widths, integer byte order, maturity discriminants,
//! representation-nonce position, reserved-field behavior, and
//! rejection of trailing bytes. Integers use big-endian byte order.
//!
//! The representation nonce is not semantic STATE: semantic projection
//! erases it. `UnexpectedSemanticFieldChange`,
//! `NoncanonicalRepresentationNonce`, and `RepresentationSearchExhausted`
//! are therefore structurally unreachable here as decode refusals and
//! belong to the constructor wave's own closed sum.

use thiserror::Error;

use crate::{Cycle, Maturity, ProtocolAmount, StateMetadata};

/// Domain separator for canonical STATE metadata bytes.
pub const STATE_METADATA_DOMAIN: &[u8] = b"tripod/state-metadata";

/// Schema revision for canonical STATE metadata bytes.
pub const STATE_METADATA_SCHEMA: u32 = 1;

/// Exact width of one canonical STATE metadata encoding.
pub const STATE_METADATA_BYTES: usize = 86;

/// Non-semantic nonce retained by the STATE metadata representation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateRepresentationNonce(u32);

impl StateRepresentationNonce {
    /// The first representation nonce.
    pub const ZERO: Self = Self(0);

    /// Construct a representation nonce.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Return the underlying representation nonce.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Advance deterministically, returning `None` at the domain maximum.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

/// Decoded semantic metadata together with its representation nonce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EncodedStateMetadata {
    /// Semantic STATE metadata.
    pub semantic: StateMetadata,

    /// Non-semantic representation nonce.
    pub representation: StateRepresentationNonce,
}

/// The closed reason canonical STATE metadata bytes are refused.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq, Hash)]
pub enum StateMetadataRefusal {
    /// The domain separator does not identify STATE metadata.
    #[error("the STATE metadata domain separator is wrong")]
    WrongDomain,

    /// The schema revision is not supported.
    #[error("the STATE metadata schema revision is unsupported")]
    UnsupportedSchema,

    /// The input ends before a fixed-width field is complete.
    #[error("the STATE metadata encoding has the wrong length")]
    WrongLength,

    /// A decoded protocol amount is outside its semantic domain.
    #[error("a STATE metadata amount is outside the protocol domain")]
    AmountOutOfDomain,

    /// The maturity discriminant is not defined by this schema.
    #[error("the STATE metadata maturity discriminant is unknown")]
    UnknownMaturityDiscriminant,

    /// The maturity payload is nonzero when its discriminant has no payload.
    #[error("the STATE metadata maturity payload is malformed")]
    MaturityPayloadMalformed,

    /// At least one reserved byte is nonzero.
    #[error("a reserved STATE metadata byte is nonzero")]
    ReservedFieldNonzero,

    /// Bytes remain after the fixed-width encoding.
    #[error("the STATE metadata encoding has trailing bytes")]
    TrailingBytes,
}

impl StateMetadataRefusal {
    /// Every refusal, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::WrongDomain,
        Self::UnsupportedSchema,
        Self::WrongLength,
        Self::AmountOutOfDomain,
        Self::UnknownMaturityDiscriminant,
        Self::MaturityPayloadMalformed,
        Self::ReservedFieldNonzero,
        Self::TrailingBytes,
    ];

    /// Return the stable kebab-case name of this refusal.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WrongDomain => "wrong-domain",
            Self::UnsupportedSchema => "unsupported-schema",
            Self::WrongLength => "wrong-length",
            Self::AmountOutOfDomain => "amount-out-of-domain",
            Self::UnknownMaturityDiscriminant => "unknown-maturity-discriminant",
            Self::MaturityPayloadMalformed => "maturity-payload-malformed",
            Self::ReservedFieldNonzero => "reserved-field-nonzero",
            Self::TrailingBytes => "trailing-bytes",
        }
    }
}

/// Encode semantic STATE metadata and its representation nonce canonically.
#[must_use]
pub fn encode_state_metadata(
    semantic: &StateMetadata,
    representation: StateRepresentationNonce,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(STATE_METADATA_BYTES);

    output.extend_from_slice(STATE_METADATA_DOMAIN);
    output.extend_from_slice(&STATE_METADATA_SCHEMA.to_be_bytes());
    output.extend_from_slice(&semantic.omega.get().to_be_bytes());
    output.extend_from_slice(&semantic.y_l.get().to_be_bytes());
    output.extend_from_slice(&semantic.y_t.get().to_be_bytes());
    output.extend_from_slice(&semantic.q.get().to_be_bytes());
    output.extend_from_slice(&semantic.cycle.get().to_be_bytes());

    let (maturity_tag, announced_cycle) = match semantic.maturity {
        Maturity::Unannounced => (0_u8, Cycle::ZERO),
        Maturity::Announced { cycle } => (1_u8, cycle),
        Maturity::Complete => (2_u8, Cycle::ZERO),
    };

    output.push(maturity_tag);
    output.extend_from_slice(&announced_cycle.get().to_be_bytes());
    output.extend_from_slice(&representation.get().to_be_bytes());
    output.extend_from_slice(&[0_u8; 8]);

    output
}

fn take<const N: usize>(input: &[u8], cursor: &mut usize) -> Result<[u8; N], StateMetadataRefusal> {
    let end = cursor
        .checked_add(N)
        .ok_or(StateMetadataRefusal::WrongLength)?;
    let slice = input
        .get(*cursor..end)
        .ok_or(StateMetadataRefusal::WrongLength)?;
    let mut field = [0_u8; N];
    field.copy_from_slice(slice);
    *cursor = end;
    Ok(field)
}

/// Decode one canonical STATE metadata representation strictly.
pub fn decode_state_metadata(bytes: &[u8]) -> Result<EncodedStateMetadata, StateMetadataRefusal> {
    if bytes.len() < STATE_METADATA_DOMAIN.len() {
        return Err(StateMetadataRefusal::WrongLength);
    }

    if &bytes[..STATE_METADATA_DOMAIN.len()] != STATE_METADATA_DOMAIN {
        return Err(StateMetadataRefusal::WrongDomain);
    }

    let mut cursor = STATE_METADATA_DOMAIN.len();

    let schema = u32::from_be_bytes(take::<4>(bytes, &mut cursor)?);

    if schema != STATE_METADATA_SCHEMA {
        return Err(StateMetadataRefusal::UnsupportedSchema);
    }

    let omega = u64::from_be_bytes(take::<8>(bytes, &mut cursor)?);
    let y_l = u64::from_be_bytes(take::<8>(bytes, &mut cursor)?);
    let y_t = u64::from_be_bytes(take::<8>(bytes, &mut cursor)?);
    let q = u64::from_be_bytes(take::<8>(bytes, &mut cursor)?);
    let cycle = u64::from_be_bytes(take::<8>(bytes, &mut cursor)?);
    let maturity_tag = take::<1>(bytes, &mut cursor)?[0];
    let announced_cycle = u64::from_be_bytes(take::<8>(bytes, &mut cursor)?);
    let representation = u32::from_be_bytes(take::<4>(bytes, &mut cursor)?);
    let reserved = take::<8>(bytes, &mut cursor)?;

    let omega = ProtocolAmount::new(omega).map_err(|_| StateMetadataRefusal::AmountOutOfDomain)?;
    let y_l = ProtocolAmount::new(y_l).map_err(|_| StateMetadataRefusal::AmountOutOfDomain)?;
    let y_t = ProtocolAmount::new(y_t).map_err(|_| StateMetadataRefusal::AmountOutOfDomain)?;
    let q = ProtocolAmount::new(q).map_err(|_| StateMetadataRefusal::AmountOutOfDomain)?;

    let maturity = match maturity_tag {
        0 if announced_cycle == 0 => Maturity::Unannounced,
        1 => Maturity::Announced {
            cycle: Cycle::new(announced_cycle),
        },
        2 if announced_cycle == 0 => Maturity::Complete,
        0 | 2 => return Err(StateMetadataRefusal::MaturityPayloadMalformed),
        _ => return Err(StateMetadataRefusal::UnknownMaturityDiscriminant),
    };

    if reserved.iter().any(|byte| *byte != 0) {
        return Err(StateMetadataRefusal::ReservedFieldNonzero);
    }

    if cursor != bytes.len() {
        return Err(StateMetadataRefusal::TrailingBytes);
    }

    Ok(EncodedStateMetadata {
        semantic: StateMetadata {
            omega,
            y_l,
            y_t,
            q,
            cycle: Cycle::new(cycle),
            maturity,
        },
        representation: StateRepresentationNonce::new(representation),
    })
}
