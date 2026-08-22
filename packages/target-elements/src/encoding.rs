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

use crate::capability::census_enum;
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

census_enum! {
    /// A stable key naming one field-specific target encoding.
    ///
    /// These are target field encodings, not attestation-contract
    /// representations. The package does not know which assets are
    /// protocol closed assets, and nothing here implies that a
    /// confidential form is or is not permitted by any protocol policy.
    ///
    /// The census below is stated as data. A consumer that must
    /// enumerate encodings reads it rather than assuming the registry
    /// is complete.
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
        /// The wider script-number encoding a lock-time operand is read at.
        ///
        /// The script language's ordinary number is bounded to four bytes,
        /// which is what makes arithmetic on it fit a thirty-two bit range.
        /// A lock-time operand is compared against an *unsigned* thirty-two
        /// bit transaction field, and the whole of that field — together
        /// with the flag bit above it that disables the check — needs a
        /// fifth byte to be stated as a signed number at all. The target
        /// reads that operand, and only that operand, at five.
        ///
        /// Stating it as its own class rather than widening
        /// [`Self::ScriptNumber`] keeps the two facts apart: an arithmetic
        /// or introspection operand five bytes wide is still malformed, and
        /// a contract that widened the shared class would have said the
        /// target accepts it everywhere.
        ///
        /// The one-operand exception is an upstream friction
        /// `(´[PLAN-obs:upstream:eg-005]´)`, easy to mis-transcribe as a
        /// general width. This class is the transcription that cannot be
        /// mistaken.
        LockTimeScriptNumber,
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
}

impl EncodingClass {
    /// What the class's payload bytes mean.
    ///
    /// Exhaustive by construction: a new encoding class cannot be
    /// added without deciding, here, whether its bytes carry a number.
    /// That is the point — the previous design let the decision be
    /// made accidentally, by whether a caller happened to pass a byte
    /// order.
    #[must_use]
    pub const fn interpretation(self) -> PayloadInterpretation {
        use PayloadInterpretation as I;

        match self {
            // Amounts are unsigned, and the explicit form reaches the
            // stack little-endian even though the transaction field
            // stores it the other way round.
            Self::ExplicitValue
            | Self::OutPointIndex
            | Self::Sequence
            | Self::UnsignedLittleEndian32
            | Self::UnsignedLittleEndian64 => I::UnsignedInteger,
            // The script language's number is signed, and so is the
            // fixed-width form the arithmetic primitives operate on.
            Self::ScriptNumber | Self::LockTimeScriptNumber | Self::SignedLittleEndian64 => {
                I::SignedInteger
            }
            // Commitments, digests, keys, signatures, scalars,
            // programs, and the absent forms are byte strings. An
            // order over them would claim an arithmetic meaning the
            // target never gives them.
            Self::ExplicitAsset
            | Self::ConfidentialAsset
            | Self::ConfidentialValue
            | Self::NullValue
            | Self::ExplicitNonce
            | Self::ConfidentialNonce
            | Self::NullNonce
            | Self::WitnessProgram
            | Self::ScriptPubKeySha256
            | Self::OutPointTxid
            | Self::OutPointFlags
            | Self::IssuanceEntropy
            | Self::IssuanceBlindingNonce
            | Self::Sha256Context
            | Self::Sha256Digest
            | Self::XOnlyPublicKey
            | Self::CompressedPublicKey
            | Self::SchnorrSignature
            | Self::EcScalar
            | Self::TaprootTweak => I::Opaque,
        }
    }

    /// The shape this class is fixed to under contract revision V1.
    ///
    /// Stated independently of the reviewed registry rather than read
    /// back out of it, so that a transcription mistake in the registry
    /// is a disagreement between two statements rather than a value
    /// agreeing with itself.
    #[must_use]
    #[expect(
        clippy::match_same_arms,
        reason = "one arm per encoding class, so a new class must be decided"
    )]
    pub const fn v1_shape(self) -> V1EncodingShape {
        use ByteOrder::LittleEndian as LE;
        use CanonicalEncodingRule as K;
        use EncodingDomain as D;

        let (domain, payload, canonicality, byte_order) = match self {
            Self::ExplicitAsset => (D::Asset, exact(32), K::PrefixDiscriminated, None),
            Self::ConfidentialAsset => (D::Asset, exact(32), K::PrefixDiscriminated, None),
            Self::ExplicitValue => (D::Value, exact(8), K::FixedWidth, Some(LE)),
            Self::ConfidentialValue => (D::Value, exact(32), K::PrefixDiscriminated, None),
            Self::NullValue => (D::Value, PayloadWidth::Absent, K::Unique, None),
            Self::ExplicitNonce => (D::Nonce, exact(32), K::PrefixDiscriminated, None),
            Self::ConfidentialNonce => (D::Nonce, exact(32), K::PrefixDiscriminated, None),
            Self::NullNonce => (D::Nonce, PayloadWidth::Absent, K::Unique, None),
            Self::WitnessProgram => (D::Program, bounded(2, 40), K::Unique, None),
            Self::ScriptPubKeySha256 => (D::Program, exact(32), K::Unique, None),
            Self::OutPointTxid => (D::OutPoint, exact(32), K::Unique, None),
            Self::OutPointIndex => (D::OutPoint, exact(4), K::FixedWidth, Some(LE)),
            Self::OutPointFlags => (D::OutPoint, exact(1), K::Unique, None),
            Self::Sequence => (D::Sequence, exact(4), K::FixedWidth, Some(LE)),
            Self::IssuanceEntropy => (D::Issuance, exact(32), K::Unique, None),
            Self::IssuanceBlindingNonce => (D::Issuance, exact(32), K::Unique, None),
            Self::ScriptNumber => (D::Number, bounded(0, 4), K::Minimal, Some(LE)),
            Self::LockTimeScriptNumber => (D::Number, bounded(0, 5), K::Minimal, Some(LE)),
            Self::SignedLittleEndian64 => (D::Number, exact(8), K::FixedWidth, Some(LE)),
            Self::UnsignedLittleEndian32 => (D::Number, exact(4), K::FixedWidth, Some(LE)),
            Self::UnsignedLittleEndian64 => (D::Number, exact(8), K::FixedWidth, Some(LE)),
            Self::Sha256Context => (D::Hash, bounded(40, 103), K::Unique, None),
            Self::Sha256Digest => (D::Hash, exact(32), K::Unique, None),
            Self::XOnlyPublicKey => (D::Key, exact(32), K::Unique, None),
            Self::CompressedPublicKey => (D::Key, exact(32), K::PrefixDiscriminated, None),
            Self::SchnorrSignature => (D::Signature, exact(64), K::Unique, None),
            Self::EcScalar => (D::Scalar, exact(32), K::Unique, None),
            Self::TaprootTweak => (D::Scalar, exact(32), K::Unique, None),
        };

        V1EncodingShape {
            domain,
            payload,
            canonicality,
            byte_order,
        }
    }
}

/// What the bytes of an encoded payload mean.
///
/// # Why this is not inferred from the byte order
///
/// Numericity used to be derived from whether a byte order happened to
/// be supplied, which made the byte-order checks tautological: an
/// encoding without an order was opaque *by definition*, so a signed
/// sixty-four bit integer offered with no order validated cleanly as
/// an opaque blob. The meaning of a field is a property of the field,
/// so it is decided here by an exhaustive match the caller cannot
/// reach into, and the offered byte order is then checked against it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PayloadInterpretation {
    /// The bytes carry no number, so no order applies to them.
    Opaque,
    /// The bytes carry a signed integer.
    SignedInteger,
    /// The bytes carry an unsigned integer.
    UnsignedInteger,
}

impl PayloadInterpretation {
    /// Whether the payload carries a number and so needs an order.
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        !matches!(self, Self::Opaque)
    }
}

/// The V1 shape of one encoding class, owned by this package.
///
/// A caller supplies prefixes and evidence links; it does not get to
/// decide what an explicit amount is or how wide an outpoint index is.
/// Those are fixed by the contract revision, and a definition that
/// disagrees with them under V1 is describing a different target
/// rather than configuring this one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct V1EncodingShape {
    domain: EncodingDomain,
    payload: PayloadWidth,
    canonicality: CanonicalEncodingRule,
    byte_order: Option<ByteOrder>,
}

impl V1EncodingShape {
    /// The field group the encoding belongs to.
    #[must_use]
    pub const fn domain(self) -> EncodingDomain {
        self.domain
    }

    /// The admissible payload width.
    #[must_use]
    pub const fn payload(self) -> PayloadWidth {
        self.payload
    }

    /// How a decoder recognizes the canonical form.
    #[must_use]
    pub const fn canonicality(self) -> CanonicalEncodingRule {
        self.canonicality
    }

    /// The order of the payload's bytes, for a numeric payload.
    #[must_use]
    pub const fn byte_order(self) -> Option<ByteOrder> {
        self.byte_order
    }
}

/// The complete typed contract of one field encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodingSpec {
    class: EncodingClass,
    domain: EncodingDomain,
    prefixes: BTreeSet<u8>,
    payload: PayloadWidth,
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

    /// What the payload's bytes mean.
    ///
    /// Derived from the encoding class, never from what the caller
    /// supplied alongside it.
    #[must_use]
    pub const fn interpretation(&self) -> PayloadInterpretation {
        self.class.interpretation()
    }

    /// Whether the payload's bytes carry a number, and so need an
    /// order.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        self.class.interpretation().is_numeric()
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
            C::LockTimeScriptNumber,
            D::Number,
            &[],
            bounded(0, 5),
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
