//! The prototype's synthetic metadata object and its canonical
//! encoding.
//!
//! # Not an ABI
//!
//! Every width, order, and tag below is a prototype design choice made
//! to exercise the constructor, and none of it is protocol semantics
//! (Guide-10 `rule:guide10:prototype-metadata`). It is deliberately
//! shaped like a real object — several independently mutable fields, an
//! explicit domain, an explicit schema, a canonical field order, exact
//! widths, and a reserved field required to be zero — so that the
//! constructor faces the problems a real object would, and deliberately
//! named nothing in particular so that no reader can mistake it for the
//! object a later guide will specify.
//!
//! # Canonical means exactly one encoding
//!
//! Decoding refuses anything that is not the encoding this module would
//! have produced: a wrong width, or a reserved field that is not zero.
//! A tolerant decoder would let two byte strings stand for one object,
//! and the whole constructor rests on the metadata leaf deriving from
//! exactly the bytes the object encodes to.

/// The domain the prototype's metadata objects live in.
///
/// Sixteen bytes of ASCII, fixed. It names the prototype rather than
/// any protocol object, which is the point: a constructor fixture may
/// name a metadata transition and may not name a protocol role
/// (Guide-10 `rule:guide10:compound-fixture`).
pub const METADATA_DOMAIN: [u8; 16] = *b"prototype-object";

/// How many bytes a canonical encoding occupies.
///
/// Sixteen of domain, four of schema, four of object kind, eight of
/// counter, four of flags, and eight reserved.
pub const METADATA_BYTES: usize = 44;

/// How many bytes the reserved field occupies.
const RESERVED_BYTES: usize = 8;

/// The prototype's metadata object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrototypeMetadata {
    /// Which prototype schema the remaining fields follow.
    pub schema: u32,
    /// Which kind of object this is, within the schema.
    pub object_kind: u32,
    /// The transition counter, and the only field a step changes.
    pub counter: u64,
    /// The object's flags.
    pub flags: u32,
}

/// Why a byte string is not a canonical metadata encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataDefect {
    /// The encoding is not exactly [`METADATA_BYTES`] long.
    WrongWidth {
        /// How many bytes were offered.
        offered: usize,
    },
    /// The domain field is not the prototype's.
    ForeignDomain,
    /// The reserved field is not zero, so the bytes are not the
    /// canonical encoding of any object.
    ReservedFieldSet,
}

/// Why a transition cannot be taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionDefect {
    /// The counter is at its maximum, so there is no successor. The
    /// prototype refuses rather than wrapping: a wrapped counter would
    /// make two distinct states of one object indistinguishable.
    CounterExhausted,
}

impl PrototypeMetadata {
    /// The canonical encoding of this object.
    #[must_use]
    pub fn encode(&self) -> [u8; METADATA_BYTES] {
        let mut bytes = [0_u8; METADATA_BYTES];
        let mut at = 0_usize;
        let mut write = |field: &[u8]| {
            bytes[at..at + field.len()].copy_from_slice(field);
            at += field.len();
        };
        write(&METADATA_DOMAIN);
        write(&self.schema.to_le_bytes());
        write(&self.object_kind.to_le_bytes());
        write(&self.counter.to_le_bytes());
        write(&self.flags.to_le_bytes());
        write(&[0_u8; RESERVED_BYTES]);
        bytes
    }

    /// The object a canonical encoding stands for.
    ///
    /// # Errors
    ///
    /// [`MetadataDefect`] when the bytes are not exactly what
    /// [`Self::encode`] would have produced for some object.
    pub fn decode(bytes: &[u8]) -> Result<Self, MetadataDefect> {
        if bytes.len() != METADATA_BYTES {
            return Err(MetadataDefect::WrongWidth {
                offered: bytes.len(),
            });
        }
        if bytes[..16] != METADATA_DOMAIN {
            return Err(MetadataDefect::ForeignDomain);
        }
        if bytes[36..44] != [0_u8; RESERVED_BYTES] {
            return Err(MetadataDefect::ReservedFieldSet);
        }
        Ok(Self {
            schema: read_u32(bytes, 16),
            object_kind: read_u32(bytes, 20),
            counter: read_u64(bytes, 24),
            flags: read_u32(bytes, 32),
        })
    }

    /// The successor of this object.
    ///
    /// Exactly one field moves, and it moves by exactly one. Every
    /// other field is carried through unchanged, which is what the
    /// constructor's continuity claim is about
    /// (Guide-10 `rule:guide10:successor-metadata`).
    ///
    /// # Errors
    ///
    /// [`TransitionDefect::CounterExhausted`] at the counter's maximum.
    pub const fn successor(&self) -> Result<Self, TransitionDefect> {
        match self.counter.checked_add(1) {
            Some(counter) => Ok(Self { counter, ..*self }),
            None => Err(TransitionDefect::CounterExhausted),
        }
    }
}

/// A four-byte little-endian field.
fn read_u32(bytes: &[u8], at: usize) -> u32 {
    let mut field = [0_u8; 4];
    field.copy_from_slice(&bytes[at..at + 4]);
    u32::from_le_bytes(field)
}

/// An eight-byte little-endian field.
fn read_u64(bytes: &[u8], at: usize) -> u64 {
    let mut field = [0_u8; 8];
    field.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(field)
}
