//! Field-specific target encodings.
//!
//! At this stage the module owns the stable encoding *keys* and the
//! width and byte-order vocabulary that opcode stack contracts refer
//! to. The specification registry that gives each key its prefixes,
//! width, canonicality rule, and unknown-prefix rule follows with the
//! encoding registry.
//!
//! # Asset and value are independent axes
//!
//! Asset confidentiality and value confidentiality are separate
//! classes here, and neither is inferred from the other. A transaction
//! may carry an explicit asset with a confidential value, or the
//! reverse, and the target contract must be able to say so.
//!
//! # Byte order is field-specific
//!
//! There is no global target byte order, and this crate deliberately
//! offers no way to set one. The reviewed target mixes orders within a
//! single value: an explicit amount is stored big-endian in the
//! transaction field but is pushed to the stack little-endian, and the
//! outpoint index is little-endian throughout. Each encoding carries
//! its own order.

use std::num::NonZeroUsize;

/// The order in which a fixed-width field's bytes are laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ByteOrder {
    /// Least significant byte first.
    LittleEndian,
    /// Most significant byte first.
    BigEndian,
}

/// How wide an encoded payload may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PayloadWidth {
    /// One exact width in bytes.
    Exact(NonZeroUsize),
    /// An inclusive range of admissible widths.
    Bounded {
        /// The smallest admissible width, which may be zero.
        minimum: usize,
        /// The largest admissible width.
        maximum: NonZeroUsize,
    },
}

impl PayloadWidth {
    /// Whether the width is internally coherent.
    ///
    /// A bounded width whose minimum exceeds its maximum describes no
    /// admissible encoding at all, and a validator must reject it
    /// rather than silently normalize the bounds.
    #[must_use]
    pub const fn is_coherent(self) -> bool {
        match self {
            Self::Exact(_) => true,
            Self::Bounded { minimum, maximum } => minimum <= maximum.get(),
        }
    }

    /// Whether the width is a single fixed size, and so requires an
    /// explicit byte order when its bytes are ordered.
    #[must_use]
    pub const fn is_fixed(self) -> bool {
        matches!(self, Self::Exact(_))
    }
}

/// A stable key naming one field-specific target encoding.
///
/// These are target field encodings, not attestation-contract
/// representations. The package does not know which assets are
/// protocol closed assets, and nothing here implies that a
/// confidential form is or is not permitted by any protocol policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EncodingClass {
    /// An asset carried in the clear.
    ExplicitAsset,
    /// An asset carried as a blinded commitment.
    ConfidentialAsset,
    /// An amount carried in the clear.
    ExplicitValue,
    /// An amount carried as a blinded commitment.
    ConfidentialValue,
    /// The absent form of a value field.
    NullValue,
    /// A nonce carried in the clear.
    ExplicitNonce,
    /// A nonce carried as a blinded commitment.
    ConfidentialNonce,
    /// The absent form of a nonce field.
    NullNonce,

    /// The program bytes of a witness-program script.
    WitnessProgram,
    /// The digest standing in for a script that is not a witness
    /// program.
    ScriptPubKeySha256,

    /// The transaction identifier half of an outpoint.
    OutPointTxid,
    /// The output index half of an outpoint.
    OutPointIndex,
    /// The issuance and peg-in flag byte accompanying an outpoint.
    OutPointFlags,
    /// An input sequence field.
    Sequence,

    /// The entropy field of an asset issuance.
    IssuanceEntropy,
    /// The blinding-nonce field of an asset issuance, whose zero value
    /// distinguishes an issuance from a reissuance.
    IssuanceBlindingNonce,

    /// The variable-width signed number encoding used by the script
    /// language.
    ScriptNumber,
    /// A signed 64-bit fixed-width integer.
    SignedLittleEndian64,
    /// An unsigned 32-bit fixed-width integer.
    UnsignedLittleEndian32,
    /// An unsigned 64-bit fixed-width integer.
    UnsignedLittleEndian64,

    /// A serialized streaming hash state.
    Sha256Context,
    /// A finalized hash output.
    Sha256Digest,

    /// A public key carried without its parity byte.
    XOnlyPublicKey,
    /// A public key carried with its parity byte.
    CompressedPublicKey,
    /// A signature over the target's script-path sighash.
    SchnorrSignature,
    /// A scalar operand of an elliptic-curve check.
    EcScalar,
    /// A tweak operand of a pay-to-contract check.
    TaprootTweak,
}

impl EncodingClass {
    /// The complete census of encoding keys.
    ///
    /// Stated here as data. A consumer that must enumerate encodings
    /// reads this rather than assuming the registry is complete.
    pub const ALL: &'static [Self] = &[
        Self::ExplicitAsset,
        Self::ConfidentialAsset,
        Self::ExplicitValue,
        Self::ConfidentialValue,
        Self::NullValue,
        Self::ExplicitNonce,
        Self::ConfidentialNonce,
        Self::NullNonce,
        Self::WitnessProgram,
        Self::ScriptPubKeySha256,
        Self::OutPointTxid,
        Self::OutPointIndex,
        Self::OutPointFlags,
        Self::Sequence,
        Self::IssuanceEntropy,
        Self::IssuanceBlindingNonce,
        Self::ScriptNumber,
        Self::SignedLittleEndian64,
        Self::UnsignedLittleEndian32,
        Self::UnsignedLittleEndian64,
        Self::Sha256Context,
        Self::Sha256Digest,
        Self::XOnlyPublicKey,
        Self::CompressedPublicKey,
        Self::SchnorrSignature,
        Self::EcScalar,
        Self::TaprootTweak,
    ];
}
