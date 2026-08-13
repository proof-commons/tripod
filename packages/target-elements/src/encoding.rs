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

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use crate::evidence::TargetEvidenceRequirementId;

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
    /// The field is absent and carries no bytes at all.
    ///
    /// This is a form in its own right, not a zero-width payload: the
    /// target uses the empty item both as a false and as an
    /// absent-field marker, and a decoder must be able to name it.
    Absent,
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
            Self::Absent | Self::Exact(_) => true,
            Self::Bounded { minimum, maximum } => minimum <= maximum.get(),
        }
    }
}

/// The group of fields within which a prefix byte must be
/// unambiguous.
///
/// A prefix byte only has to distinguish the forms of *one* field. The
/// same byte means "explicit" in the asset, value, and nonce domains
/// alike, and treating prefixes as globally unique would be a
/// misreading of the target rather than a stricter check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EncodingDomain {
    /// The asset field of an input or output.
    Asset,
    /// The value field of an input or output.
    Value,
    /// The nonce field of an output.
    Nonce,
    /// A locking program.
    Program,
    /// The parts of an outpoint.
    OutPoint,
    /// An input sequence field.
    Sequence,
    /// The fields of an asset issuance.
    Issuance,
    /// The target's numeric representations.
    Number,
    /// Hash states and outputs.
    Hash,
    /// Public keys.
    Key,
    /// Signatures.
    Signature,
    /// Curve scalars and tweaks.
    Scalar,
}

/// How a decoder decides that an encoded value is in canonical form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CanonicalEncodingRule {
    /// Exactly one byte string represents the value.
    Unique,
    /// The shortest representation is the only admissible one.
    Minimal,
    /// The value occupies one fixed width.
    FixedWidth,
    /// A leading byte selects among the admissible forms.
    PrefixDiscriminated,
}

/// What a decoder does with a prefix byte it does not recognize.
///
/// There is exactly one rule, and that is the point. An unknown prefix
/// is not preserved as an opaque but valid value on the theory that a
/// future target might define it: doing so would let an encoding this
/// package has never reviewed flow through a program as though it had
/// been. Forward compatibility is a decision to be taken explicitly at
/// each boundary, not a default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UnknownPrefixRule {
    /// The value is refused.
    Reject,
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

/// The complete typed contract of one field encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodingSpec {
    class: EncodingClass,
    domain: EncodingDomain,
    prefixes: BTreeSet<u8>,
    payload: PayloadWidth,
    numeric: bool,
    byte_order: Option<ByteOrder>,
    canonicality: CanonicalEncodingRule,
    unknown_prefix: UnknownPrefixRule,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl EncodingSpec {
    /// States the contract of one encoding.
    #[must_use]
    pub fn new(
        class: EncodingClass,
        domain: EncodingDomain,
        prefixes: impl IntoIterator<Item = u8>,
        payload: PayloadWidth,
        byte_order: Option<ByteOrder>,
        canonicality: CanonicalEncodingRule,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            class,
            domain,
            prefixes: prefixes.into_iter().collect(),
            payload,
            numeric: byte_order.is_some(),
            byte_order,
            canonicality,
            unknown_prefix: UnknownPrefixRule::Reject,
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The encoding's stable identity.
    #[must_use]
    pub const fn class(&self) -> EncodingClass {
        self.class
    }

    /// The field group its prefixes must be unambiguous within.
    #[must_use]
    pub const fn domain(&self) -> EncodingDomain {
        self.domain
    }

    /// The prefix bytes that select this form.
    #[must_use]
    pub const fn prefixes(&self) -> &BTreeSet<u8> {
        &self.prefixes
    }

    /// The admissible payload width.
    #[must_use]
    pub const fn payload(&self) -> PayloadWidth {
        self.payload
    }

    /// Whether the payload's bytes carry a number, and so need an
    /// order.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        self.numeric
    }

    /// The order of the payload's bytes, for a numeric payload.
    #[must_use]
    pub const fn byte_order(&self) -> Option<ByteOrder> {
        self.byte_order
    }

    /// How a decoder recognizes the canonical form.
    #[must_use]
    pub const fn canonicality(&self) -> CanonicalEncodingRule {
        self.canonicality
    }

    /// What a decoder does with an unrecognized prefix.
    #[must_use]
    pub const fn unknown_prefix(&self) -> UnknownPrefixRule {
        self.unknown_prefix
    }

    /// The evidence a deployment must produce for this encoding.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }
}

/// An exact payload width, for the registry declarations.
const fn exact(width: usize) -> PayloadWidth {
    PayloadWidth::Exact(NonZeroUsize::new(width).expect("a reviewed width is never zero"))
}

/// A bounded payload width, for the registry declarations.
const fn bounded(minimum: usize, maximum: usize) -> PayloadWidth {
    PayloadWidth::Bounded {
        minimum,
        maximum: NonZeroUsize::new(maximum).expect("a reviewed bound is never zero"),
    }
}

/// Builds the reviewed encoding registry.
///
/// Asset and value are independent axes here, as they are on the
/// target: a transaction may carry an explicit asset with a
/// confidential value or the reverse, and nothing in this registry
/// lets one be inferred from the other.
/// The evidence a shape claim needs.
const SHAPE_EVIDENCE: &[TargetEvidenceRequirementId] =
    &[TargetEvidenceRequirementId::EncodingSemantics];

/// The evidence an introspected field's shape claim needs.
const FIELD_EVIDENCE: &[TargetEvidenceRequirementId] = &[
    TargetEvidenceRequirementId::EncodingSemantics,
    TargetEvidenceRequirementId::InputIntrospectionSemantics,
];

/// The evidence an issuance field's shape claim needs.
const ISSUANCE_EVIDENCE: &[TargetEvidenceRequirementId] = &[
    TargetEvidenceRequirementId::EncodingSemantics,
    TargetEvidenceRequirementId::IssuanceIntrospection,
];

/// Builds one encoding specification, always rejecting unknown
/// prefixes.
fn spec(
    class: EncodingClass,
    domain: EncodingDomain,
    prefixes: &[u8],
    payload: PayloadWidth,
    byte_order: Option<ByteOrder>,
    canonicality: CanonicalEncodingRule,
    evidence: &[TargetEvidenceRequirementId],
) -> (EncodingClass, EncodingSpec) {
    (
        class,
        EncodingSpec::new(
            class,
            domain,
            prefixes.iter().copied(),
            payload,
            byte_order,
            canonicality,
            evidence.iter().copied(),
        ),
    )
}

/// Asset, value, and nonce field encodings.
fn value_field_encodings() -> Vec<(EncodingClass, EncodingSpec)> {
    use ByteOrder::LittleEndian as LE;
    use CanonicalEncodingRule as K;
    use EncodingClass as C;
    use EncodingDomain as D;

    let shape = SHAPE_EVIDENCE;
    let asset = FIELD_EVIDENCE;

    vec![
        // -- Asset ---------------------------------------------
        spec(
            C::ExplicitAsset,
            D::Asset,
            &[0x01],
            exact(32),
            None,
            K::PrefixDiscriminated,
            asset,
        ),
        spec(
            C::ConfidentialAsset,
            D::Asset,
            &[0x0a, 0x0b],
            exact(32),
            None,
            K::PrefixDiscriminated,
            asset,
        ),
        // -- Value ---------------------------------------------
        //
        // The explicit amount is the one encoding whose byte order
        // differs between the transaction field and the stack: the
        // field stores it most significant byte first and the
        // introspection primitive reverses it on the way out. The
        // order recorded here is the stack order, because that is the
        // order a program actually operates on.
        spec(
            C::ExplicitValue,
            D::Value,
            &[0x01],
            exact(8),
            Some(LE),
            K::FixedWidth,
            asset,
        ),
        spec(
            C::ConfidentialValue,
            D::Value,
            &[0x08, 0x09],
            exact(32),
            None,
            K::PrefixDiscriminated,
            asset,
        ),
        spec(
            C::NullValue,
            D::Value,
            &[],
            PayloadWidth::Absent,
            None,
            K::Unique,
            shape,
        ),
        // -- Nonce ---------------------------------------------
        spec(
            C::ExplicitNonce,
            D::Nonce,
            &[0x01],
            exact(32),
            None,
            K::PrefixDiscriminated,
            shape,
        ),
        spec(
            C::ConfidentialNonce,
            D::Nonce,
            &[0x02, 0x03],
            exact(32),
            None,
            K::PrefixDiscriminated,
            shape,
        ),
        spec(
            C::NullNonce,
            D::Nonce,
            &[],
            PayloadWidth::Absent,
            None,
            K::Unique,
            shape,
        ),
    ]
}

/// Program, outpoint, sequence, and issuance encodings.
fn structural_encodings() -> Vec<(EncodingClass, EncodingSpec)> {
    use ByteOrder::LittleEndian as LE;
    use CanonicalEncodingRule as K;
    use EncodingClass as C;
    use EncodingDomain as D;

    let shape = SHAPE_EVIDENCE;
    let issuance = ISSUANCE_EVIDENCE;

    vec![
        // -- Program -------------------------------------------
        spec(
            C::WitnessProgram,
            D::Program,
            &[],
            bounded(2, 40),
            None,
            K::Unique,
            shape,
        ),
        spec(
            C::ScriptPubKeySha256,
            D::Program,
            &[],
            exact(32),
            None,
            K::Unique,
            shape,
        ),
        // -- OutPoint ------------------------------------------
        spec(
            C::OutPointTxid,
            D::OutPoint,
            &[],
            exact(32),
            None,
            K::Unique,
            shape,
        ),
        spec(
            C::OutPointIndex,
            D::OutPoint,
            &[],
            exact(4),
            Some(LE),
            K::FixedWidth,
            shape,
        ),
        spec(
            C::OutPointFlags,
            D::OutPoint,
            &[],
            exact(1),
            None,
            K::Unique,
            issuance,
        ),
        // -- Sequence ------------------------------------------
        spec(
            C::Sequence,
            D::Sequence,
            &[],
            exact(4),
            Some(LE),
            K::FixedWidth,
            shape,
        ),
        // -- Issuance ------------------------------------------
        spec(
            C::IssuanceEntropy,
            D::Issuance,
            &[],
            exact(32),
            None,
            K::Unique,
            issuance,
        ),
        spec(
            C::IssuanceBlindingNonce,
            D::Issuance,
            &[],
            exact(32),
            None,
            K::Unique,
            issuance,
        ),
    ]
}

/// Numeric, hashing, key, signature, and scalar encodings.
fn primitive_encodings() -> Vec<(EncodingClass, EncodingSpec)> {
    use ByteOrder::LittleEndian as LE;
    use CanonicalEncodingRule as K;
    use EncodingClass as C;
    use EncodingDomain as D;

    let shape = SHAPE_EVIDENCE;

    vec![
        // -- Numbers -------------------------------------------
        spec(
            C::ScriptNumber,
            D::Number,
            &[],
            bounded(0, 4),
            Some(LE),
            K::Minimal,
            shape,
        ),
        spec(
            C::SignedLittleEndian64,
            D::Number,
            &[],
            exact(8),
            Some(LE),
            K::FixedWidth,
            shape,
        ),
        spec(
            C::UnsignedLittleEndian32,
            D::Number,
            &[],
            exact(4),
            Some(LE),
            K::FixedWidth,
            shape,
        ),
        spec(
            C::UnsignedLittleEndian64,
            D::Number,
            &[],
            exact(8),
            Some(LE),
            K::FixedWidth,
            shape,
        ),
    ]
}

/// Hash, key, signature, and scalar encodings.
fn cryptographic_encodings() -> Vec<(EncodingClass, EncodingSpec)> {
    use CanonicalEncodingRule as K;
    use EncodingClass as C;
    use EncodingDomain as D;

    let shape = SHAPE_EVIDENCE;

    vec![
        // -- Hashing -------------------------------------------
        //
        // A serialized hash state is forty bytes plus whatever partial
        // block it holds, so its width is genuinely variable.
        spec(
            C::Sha256Context,
            D::Hash,
            &[],
            bounded(40, 103),
            None,
            K::Unique,
            shape,
        ),
        spec(
            C::Sha256Digest,
            D::Hash,
            &[],
            exact(32),
            None,
            K::Unique,
            shape,
        ),
        // -- Keys, signatures, scalars -------------------------
        spec(
            C::XOnlyPublicKey,
            D::Key,
            &[],
            exact(32),
            None,
            K::Unique,
            shape,
        ),
        spec(
            C::CompressedPublicKey,
            D::Key,
            &[0x02, 0x03],
            exact(32),
            None,
            K::PrefixDiscriminated,
            shape,
        ),
        spec(
            C::SchnorrSignature,
            D::Signature,
            &[],
            exact(64),
            None,
            K::Unique,
            shape,
        ),
        spec(
            C::EcScalar,
            D::Scalar,
            &[],
            exact(32),
            None,
            K::Unique,
            shape,
        ),
        spec(
            C::TaprootTweak,
            D::Scalar,
            &[],
            exact(32),
            None,
            K::Unique,
            shape,
        ),
    ]
}

/// Builds the reviewed encoding registry.
///
/// Asset and value are independent axes here, as they are on the
/// target: a transaction may carry an explicit asset with a
/// confidential value or the reverse, and nothing in this registry
/// lets one be inferred from the other.
pub(crate) fn reviewed_encodings() -> BTreeMap<EncodingClass, EncodingSpec> {
    [
        value_field_encodings(),
        structural_encodings(),
        primitive_encodings(),
        cryptographic_encodings(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
