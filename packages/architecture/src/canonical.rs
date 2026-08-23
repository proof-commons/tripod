//! Canonical semantic serialization and hashing.
//!
//! The semantic hash is computed over a deliberately canonicalized JSON
//! representation of the export body (not the published envelope, which
//! contains the hash itself). The encoding is intentionally insensitive
//! to comments, formatting, rustfmt, and declaration order in the Rust
//! source: object keys are sorted, and all semantically set-like arrays
//! are sorted during export conversion.
//!
//! TOML and pretty-printed JSON are presentation encodings and are
//! never hash inputs. This format is named
//! `tripod canonical manifest JSON v3` (v2 lifted the version
//! fields out of the hashed body and into the publication envelope; v3
//! prefixes the hashed input with a domain separator and changes no
//! byte of the encoding itself); it is not a claim of RFC 8785/JCS
//! compliance.
//!
//! Two hashes are computed over the same canonical encoding: the full
//! semantic hash (the whole export body) and the behavioural hash
//! (the behavioural arrays only). Each applies its own
//! domain-separation prefix, so the two digests cannot collide across
//! recipes even over coincidentally identical bytes. The versioning
//! gate `(´[RZ-pin:pins:denotation]´)` keys on the latter.
//!
//! Every public identity function takes a
//! [`ValidatedDraftArchitecture`], never a raw `Architecture`
//! (R2-N03). The adopted discipline puts validation before identity
//! `(´[ADR021-rule:identity:admission-order]´)`, and rehashing is
//! explicitly not revalidation, so an architecture with duplicate
//! declarations or a missing root must not be able to acquire a
//! semantic hash through any public path. The unchecked projections
//! remain crate-private for the mutation tests, which deliberately
//! build invalid architectures.

use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::export::{
    AmountLimitExport, ArchitectureExport, AssetExport, BoundExport, ClauseExport, ObjectExport,
    OperationExport, QuantityExport, RootExport, TagExport, WitnessExport,
};
use crate::spec::Architecture;
use crate::validate::ValidatedDraftArchitecture;

pub const SEMANTIC_HASH_ALGORITHM: &str = "sha256-canonical-json-v3";

/// Retired semantic algorithm identifiers.
///
/// `…-v2` hashed the canonical body with no domain separator, carrying
/// the algorithm identifier beside the digest in the envelope rather
/// than inside the hashed input. It was a reviewed exception to the
/// domain-separated form until the adopted adjudication discipline
/// required domain separation for every semantic identity; `…-v3`
/// prefixes the same canonical bytes. See
/// `(´[ADR021-rule:identity:separation-migration]´)`.
/// The projection and encoding are untouched, so the meaning
/// identified is unchanged and only the measurement moved. The retired
/// pinned value is recorded with the migration in ADR-021.
pub const RETIRED_SEMANTIC_HASH_ALGORITHMS: &[&str] = &["sha256-canonical-json-v2"];

pub const BEHAVIOURAL_HASH_ALGORITHM: &str = "sha256-canonical-json-behavioural-v3";

/// Retired behavioural algorithm identifiers.
///
/// `…-v1` hashed the full `BoundExport` rows, so a calibrated bound's
/// *draft default* moved the denotation hash — contradicting the
/// Denotation Law, which places `requires_deployment_calibration =
/// true` magnitudes outside the abstract denotation. v2 projected
/// those defaults out but still hashed full witness and clause rows,
/// so a *document label* rename (a witness `semantic_tag` or the
/// clause registry's frozen citation label) moved the hash —
/// contradicting the same law, which lists labels as presentation. v3
/// projects the document labels out; witness semantic identity (code
/// and id) and clause identity (code) remain hash inputs. Each retired
/// identifier's pinned release value is recorded in
/// `versioning_gate_tests` for the historical migration.
pub const RETIRED_BEHAVIOURAL_HASH_ALGORITHMS: &[&str] = &[
    "sha256-canonical-json-behavioural-v1",
    "sha256-canonical-json-behavioural-v2",
];

/// Domain-separation prefix for the behavioural hash input, so a
/// behavioural digest can never be confused with a full-manifest
/// digest over coincidentally identical bytes.
const BEHAVIOURAL_DOMAIN_PREFIX: &[u8] = b"tripod behavioural JSON v3\n";

/// Domain-separation prefix for the full-manifest semantic hash input.
///
/// Each prefix folds the domain separator and the recipe identifier of
/// `(´[ADR021-rule:identity:recipes]´)` into one string, as the behavioural
/// and deployment-profile prefixes already do.
const MANIFEST_DOMAIN_PREFIX: &[u8] = b"tripod canonical manifest JSON v3\n";

/// Canonical specification anchor-set hash algorithm identifier.
pub const ANCHOR_SET_HASH_ALGORITHM: &str = "sha256-anchor-set-v2";

/// Retired anchor-set algorithm identifiers.
///
/// `…-v1` retroactively names the original recipe, which hashed the
/// newline-joined sorted names with no domain separator and published
/// no identifier at all. It is named here so the migration record can
/// refer to it; no manifest ever carried the string. The name set and
/// its ordering are untouched by the migration, so the dependency set
/// identified is unchanged and only the measurement moved.
pub const RETIRED_ANCHOR_SET_HASH_ALGORITHMS: &[&str] = &["sha256-anchor-set-v1"];

/// Domain-separation prefix for the specification anchor-set hash input (a frozen recipe string).
const ANCHOR_SET_DOMAIN_PREFIX: &[u8] = b"tripod layer-0 anchor set v2\n";

/// Behavioural projection of a bound: identity, cardinality use, and
/// the frozen calibration classification. A calibrated bound's draft
/// `default_value` is deployment-profile material outside the abstract
/// denotation `(´[RZ-def:versioning:denotation-law]´)`, so it is projected
/// out of the hash input; a fixed (uncalibrated) bound's value is part
/// of the denotation and stays.
#[derive(Serialize)]
struct BehaviouralBoundProjection {
    code: u16,
    id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<u64>,

    requires_deployment_calibration: bool,
}

impl BehaviouralBoundProjection {
    fn from_export(bound: &BoundExport) -> Self {
        Self {
            code: bound.code,
            id: bound.id.clone(),
            default_value: if bound.requires_deployment_calibration {
                None
            } else {
                bound.default_value
            },
            requires_deployment_calibration: bound.requires_deployment_calibration,
        }
    }
}

/// Behavioural projection of a witness: stable code plus semantic
/// identity. The exported `semantic_tag` is the realization document's
/// citation label — presentation under the Denotation Law, guarded by
/// the sixteen-string weld rather than the hash — so it is projected
/// out: renaming a document label leaves the denotation and its
/// behavioural hash unchanged.
#[derive(Serialize)]
struct BehaviouralWitnessProjection {
    code: u16,
    id: String,
}

impl BehaviouralWitnessProjection {
    fn from_export(witness: &WitnessExport) -> Self {
        Self {
            code: witness.code,
            id: witness.id.clone(),
        }
    }
}

/// Behavioural projection of an invariant clause: the stable code
/// only. The exported `id` is the document's frozen clause label —
/// presentation, welded to the document separately — so it is
/// projected out for the same reason as witness tags.
#[derive(Serialize)]
struct BehaviouralClauseProjection {
    code: u16,
}

impl BehaviouralClauseProjection {
    fn from_export(clause: &ClauseExport) -> Self {
        Self { code: clause.code }
    }
}

/// The behavioural arrays only — the finite presentation of the
/// abstract system's denotation. Dependencies, decisions, evidence
/// tables, and every envelope field are deliberately excluded: they
/// are normative-descriptive or presentational, so changing them
/// leaves the denotation and its behavioural hash unchanged. Bounds
/// enter through
/// [`BehaviouralBoundProjection`] (calibrated draft defaults dropped);
/// witnesses and clauses enter through their projections (document
/// citation labels dropped). The versioning gate keys on this hash.
#[derive(Serialize)]
struct BehaviouralBody<'a> {
    assets: &'a [AssetExport],
    roots: &'a [RootExport],
    objects: &'a [ObjectExport],
    operations: &'a [OperationExport],
    quantities: &'a [QuantityExport],
    witnesses: Vec<BehaviouralWitnessProjection>,
    clauses: Vec<BehaviouralClauseProjection>,
    bounds: Vec<BehaviouralBoundProjection>,
    amount_limits: &'a [AmountLimitExport],
    tags: &'a [TagExport],
}

/// Hash over the behavioural arrays and relations only.
///
/// Covers assets, roots, objects, operations, quantities, witnesses,
/// clauses, bounds, `amount_limits`, and tags — dependencies,
/// decisions, and envelope excluded. The versioning gate keys on
/// this: if this hash moves undeclared,
/// the build fails.
pub fn behavioural_hash(
    validated: &ValidatedDraftArchitecture<'_>,
) -> Result<[u8; 32], serde_json::Error> {
    unchecked_behavioural_hash(validated.architecture())
}

pub fn behavioural_hash_hex(
    validated: &ValidatedDraftArchitecture<'_>,
) -> Result<String, serde_json::Error> {
    Ok(hex(&behavioural_hash(validated)?))
}

/// Behavioural hash over an architecture that has *not* been validated.
///
/// Crate-private on purpose (R2-N03): mutation tests deliberately build
/// invalid architectures and still need the projection, but no public
/// path may mint an identity for one.
pub(crate) fn unchecked_behavioural_hash(
    architecture: &Architecture,
) -> Result<[u8; 32], serde_json::Error> {
    let export = ArchitectureExport::from_architecture(architecture);

    export_behavioural_hash(&export)
}

pub(crate) fn export_behavioural_hash(
    export: &ArchitectureExport,
) -> Result<[u8; 32], serde_json::Error> {
    let body = BehaviouralBody {
        assets: &export.assets,
        roots: &export.roots,
        objects: &export.objects,
        operations: &export.operations,
        quantities: &export.quantities,
        witnesses: export
            .witnesses
            .iter()
            .map(BehaviouralWitnessProjection::from_export)
            .collect(),
        clauses: export
            .clauses
            .iter()
            .map(BehaviouralClauseProjection::from_export)
            .collect(),
        bounds: export
            .bounds
            .iter()
            .map(BehaviouralBoundProjection::from_export)
            .collect(),
        amount_limits: &export.amount_limits,
        tags: &export.tags,
    };

    let canonical = canonicalize_json(serde_json::to_value(&body)?);
    let bytes = serde_json::to_vec(&canonical)?;

    let mut hasher = Sha256::new();
    hasher.update(BEHAVIOURAL_DOMAIN_PREFIX);
    hasher.update(&bytes);

    let mut result = [0_u8; 32];
    result.copy_from_slice(&hasher.finalize());

    Ok(result)
}

pub(crate) fn export_behavioural_hash_hex(
    export: &ArchitectureExport,
) -> Result<String, serde_json::Error> {
    Ok(hex(&export_behavioural_hash(export)?))
}

pub fn canonical_json_bytes(
    validated: &ValidatedDraftArchitecture<'_>,
) -> Result<Vec<u8>, serde_json::Error> {
    unchecked_canonical_json_bytes(validated.architecture())
}

pub fn semantic_hash(
    validated: &ValidatedDraftArchitecture<'_>,
) -> Result<[u8; 32], serde_json::Error> {
    unchecked_semantic_hash(validated.architecture())
}

pub fn semantic_hash_hex(
    validated: &ValidatedDraftArchitecture<'_>,
) -> Result<String, serde_json::Error> {
    Ok(hex(&semantic_hash(validated)?))
}

/// Canonical projection of an architecture that has *not* been
/// validated; crate-private for the same reason as
/// [`unchecked_behavioural_hash`].
pub(crate) fn unchecked_canonical_json_bytes(
    architecture: &Architecture,
) -> Result<Vec<u8>, serde_json::Error> {
    let export = ArchitectureExport::from_architecture(architecture);

    export_body_canonical_bytes(&export)
}

/// Semantic hash over an architecture that has *not* been validated;
/// crate-private for the same reason as [`unchecked_behavioural_hash`].
pub(crate) fn unchecked_semantic_hash(
    architecture: &Architecture,
) -> Result<[u8; 32], serde_json::Error> {
    let bytes = unchecked_canonical_json_bytes(architecture)?;

    Ok(manifest_digest(&bytes))
}

/// The full-manifest semantic digest: the domain prefix over the
/// canonical body bytes. Single definition on purpose — the envelope
/// producer and the standalone identity function must never drift into
/// two recipes wearing one identifier.
fn manifest_digest(canonical_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(MANIFEST_DOMAIN_PREFIX);
    hasher.update(canonical_bytes);

    let mut result = [0_u8; 32];
    result.copy_from_slice(&hasher.finalize());

    result
}

/// Why one offered string is not an anchor name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AnchorNameDefect {
    /// The name holds the byte the anchor-set framing joins with.
    ///
    /// Named separately from the general character rule because it is
    /// the defect that would make the framing ambiguous rather than
    /// merely admit an unpronounceable name.
    HoldsTheSetSeparator,
    /// The name is not the three-part `kind:area:name` form.
    NotThreePart,
    /// One of the three segments is empty.
    SegmentEmpty,
    /// A segment holds a byte outside lowercase letters, digits, and
    /// the hyphen.
    SegmentCharacterNotAdmitted,
    /// A segment opens or closes with a hyphen.
    SegmentEdgeHyphen,
    /// A segment holds two hyphens in a row.
    SegmentDoubleHyphen,
    /// The kind segment is hyphenated, which the kind registry admits
    /// for no member.
    KindHyphenated,
}

impl fmt::Display for AnchorNameDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::HoldsTheSetSeparator => {
                "the anchor name holds the newline the anchor-set framing joins with"
            }
            Self::NotThreePart => "the anchor name is not written kind:area:name",
            Self::SegmentEmpty => "a segment of the anchor name is empty",
            Self::SegmentCharacterNotAdmitted => {
                "a segment holds a byte outside lowercase letters, digits, and the hyphen"
            }
            Self::SegmentEdgeHyphen => "a segment opens or closes with a hyphen",
            Self::SegmentDoubleHyphen => "a segment holds two hyphens in a row",
            Self::KindHyphenated => "the kind segment is hyphenated",
        };
        formatter.write_str(text)
    }
}

/// One specification anchor name, in the label grammar.
///
/// An anchor name is a specification label the realization document cites,
/// without the consumer prefix: the three-part `kind:area:name` form,
/// each segment written in lowercase letters, digits, and interior
/// hyphens, with an unhyphenated kind.
///
/// # Why this is a type
///
/// [`anchor_set_hash`] frames its members by newline join, so a name
/// holding a newline would make two different sets share one preimage.
/// The grammar excludes the separator, but a function taking plain
/// strings does not *say* so, and the safety of the identity then rests
/// on caller discipline rather than on the API. Validation before
/// identity is the repository's rule
/// `(´[ADR021-rule:identity:admission-order]´)`, and this type is how the
/// anchor-set identity keeps it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnchorName(String);

impl AnchorName {
    /// One anchor name, or why the text is not one.
    ///
    /// # Errors
    ///
    /// [`AnchorNameDefect`], naming the first rule the text breaks. The
    /// separator is reported before the general character rule, so a
    /// name holding a newline says what is actually wrong with it.
    pub fn new(text: &str) -> Result<Self, AnchorNameDefect> {
        if text.contains('\n') {
            return Err(AnchorNameDefect::HoldsTheSetSeparator);
        }

        let segments = text.split(':').collect::<Vec<_>>();
        if segments.len() != ANCHOR_NAME_SEGMENTS {
            return Err(AnchorNameDefect::NotThreePart);
        }

        for segment in &segments {
            if segment.is_empty() {
                return Err(AnchorNameDefect::SegmentEmpty);
            }
            if !segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(AnchorNameDefect::SegmentCharacterNotAdmitted);
            }
            if segment.starts_with('-') || segment.ends_with('-') {
                return Err(AnchorNameDefect::SegmentEdgeHyphen);
            }
            if segment.contains("--") {
                return Err(AnchorNameDefect::SegmentDoubleHyphen);
            }
        }

        // The kind ranges over a registry of words, and a registry of
        // words admits no hyphenated member.
        if segments[0].contains('-') {
            return Err(AnchorNameDefect::KindHyphenated);
        }

        Ok(Self(text.to_owned()))
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AnchorName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The number of colon-separated segments in an anchor name.
const ANCHOR_NAME_SEGMENTS: usize = 3;

/// A specification anchor set every member of which is an anchor name.
///
/// The set is the identity input: sorted and deduplicated, because a
/// set is what the recipe hashes and an occurrence order is not part of
/// it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidatedAnchorSet(BTreeSet<AnchorName>);

impl ValidatedAnchorSet {
    /// One anchor set, from names in any order with repeats.
    ///
    /// # Errors
    ///
    /// [`AnchorNameDefect`] for the first offered string that is not an
    /// anchor name. The set is all-or-nothing: a set holding one name
    /// that is not an anchor name is not an anchor set, and hashing the
    /// rest of it would mint an identity for a census nobody stated.
    pub fn new<'a>(
        anchor_names: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, AnchorNameDefect> {
        anchor_names
            .into_iter()
            .map(AnchorName::new)
            .collect::<Result<BTreeSet<AnchorName>, AnchorNameDefect>>()
            .map(Self)
    }

    /// The distinct names, in canonical order.
    pub fn iter(&self) -> impl Iterator<Item = &AnchorName> {
        self.0.iter()
    }

    /// How many distinct names the set holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the set holds no name at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The specification anchor-set hash recipe ([`ANCHOR_SET_HASH_ALGORITHM`]):
/// `sha256( domain_prefix || join("\n", sorted(distinct anchor names)) )`.
///
/// Anchor names are the specification labels the realization document cites,
/// without the `A-` consumer prefix. Sorting and deduplication are
/// [`ValidatedAnchorSet`]'s, so the names offered to it may be in any
/// occurrence order with repeats. The result is the value pinned as the
/// manifest's `SpecificationBinding::anchor_set_hash`; release validation
/// refuses a manifest that leaves it unset.
///
/// # The recipe is unchanged
///
/// Requiring a [`ValidatedAnchorSet`] is input validation, not a recipe
/// migration. The domain prefix is the same prefix, the join is the
/// same newline join over the same sorted distinct names, and every
/// anchor set that could honestly be offered before hashes to exactly
/// what it hashed before — so the pinned manifest value and
/// [`ANCHOR_SET_HASH_ALGORITHM`] both stand. What changed is which
/// inputs can be offered at all: a name holding the separator gave two
/// different sets one preimage, and that name is now not an anchor
/// name.
///
/// The prefix is the only difference from the retired `…-v1` recipe:
/// the identified dependency set and its canonical ordering are
/// unchanged.
#[must_use]
pub fn anchor_set_hash(anchors: &ValidatedAnchorSet) -> [u8; 32] {
    let joined = anchors
        .iter()
        .map(AnchorName::as_str)
        .collect::<Vec<_>>()
        .join("\n");

    let mut hasher = Sha256::new();
    hasher.update(ANCHOR_SET_DOMAIN_PREFIX);
    hasher.update(joined.as_bytes());

    let mut result = [0_u8; 32];
    result.copy_from_slice(&hasher.finalize());

    result
}

pub(crate) fn export_body_hash_hex(
    export: &ArchitectureExport,
) -> Result<String, serde_json::Error> {
    let bytes = export_body_canonical_bytes(export)?;

    Ok(hex(&manifest_digest(&bytes)))
}

fn export_body_canonical_bytes(export: &ArchitectureExport) -> Result<Vec<u8>, serde_json::Error> {
    let value = serde_json::to_value(export)?;

    let canonical = canonicalize_json(value);

    serde_json::to_vec(&canonical)
}

pub(crate) fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object
                .into_iter()
                .map(|(key, value)| (key, canonicalize_json(value)))
                .collect::<Vec<_>>();

            entries.sort_by(|left, right| left.0.cmp(&right.0));

            let mut result = Map::new();

            for (key, value) in entries {
                result.insert(key, value);
            }

            Value::Object(result)
        }

        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),

        other => other,
    }
}

/// Lowercase hex rendering used for every published digest.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";

    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }

    output
}
