//! Byte-faithful import of the `0.6.52-dev` rerun-day archive.
//!
//! The archive is evidence about the suite that produced its bytes. Its identity is
//! artifact-internal: baseline commit `0.6.52-dev`, serialized execution, 40 of 40 tests, and
//! 420.7 seconds. Nothing in this module infers that identity from, or relabels it to, a later
//! tree.
//!
//! All 79 supplied files remain embedded and manifest-checked. The 39 timing sidecars and the
//! confidential-predecessor setup transcript are retained for audit only. The other 39 files
//! are separate test ceremonies and canonical evidence inputs; no parser path merges them into
//! a fictional aggregate run.
//!
//! The only successful parse result is typed incompleteness. Thirty-eight ceremonies lack exact
//! request bytes, while the key-path probe has its exact requests but lacks complete executor
//! provenance. There is deliberately no complete-carrier variant, row identifier, standing
//! type, or mutable accessor here, so importing this module cannot retype a standing row.

use std::collections::HashMap;
use std::fmt::Write as _;

use target_elements_conformance::constructor::tagged;

/// Number of opaque files retained from rerun day.
pub const ARCHIVE_FILE_COUNT: usize = 79;
/// Total bytes across the 79 opaque files.
pub const ARCHIVE_TOTAL_BYTES: usize = 66_133;
/// Number of canonical ceremony transcripts.
pub const CANONICAL_TRANSCRIPT_COUNT: usize = 39;
/// Number of audit-only timing sidecars.
pub const TIMING_SIDECAR_COUNT: usize = 39;
/// SHA-256 of the sorted `sha256sum`-format manifest for all 79 files.
pub const ARCHIVE_MANIFEST_SHA256: &str =
    "a42961b402cae902fc63b167e359873675b477634c358153b16628dbac26639a";

/// Artifact-internal identity carried by the rerun-day archive.
pub const ARCHIVE_SUITE_IDENTITY: ArchiveSuiteIdentity = ArchiveSuiteIdentity {
    baseline_commit: "0.6.52-dev",
    execution: SuiteExecution::Serialized,
    passed: 40,
    total: 40,
    wall_milliseconds: 420_700,
};

/// Whether the archived suite permitted concurrent ceremonies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuiteExecution {
    /// The ceremonies ran serially.
    Serialized,
}

/// Suite identity stated by the archived artifact itself.
///
/// This value is historical archive data. A later checkout cannot update it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveSuiteIdentity {
    /// Full baseline commit recorded for the rerun.
    pub baseline_commit: &'static str,
    /// Execution ordering recorded for the rerun.
    pub execution: SuiteExecution,
    /// Passing ceremony count.
    pub passed: usize,
    /// Total ceremony count.
    pub total: usize,
    /// Suite wall time, represented exactly in milliseconds.
    pub wall_milliseconds: u64,
}

/// Evidentiary role of an opaque archive file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveFileRole {
    /// One canonical test-ceremony transcript.
    CanonicalTranscript,
    /// Process timing retained only for audit.
    TimingSidecar,
    /// Setup transcript retained only for audit.
    PredecessorSetup,
}

/// One byte-embedded archive file and its mechanically generated digest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveFile {
    name: &'static str,
    bytes: &'static [u8],
    expected_sha256: &'static str,
    role: ArchiveFileRole,
}

impl ArchiveFile {
    /// Archive basename.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Exact archived bytes.
    #[must_use]
    pub const fn bytes(self) -> &'static [u8] {
        self.bytes
    }

    /// Lower-case SHA-256 from the generated manifest.
    #[must_use]
    pub const fn expected_sha256(self) -> &'static str {
        self.expected_sha256
    }

    /// Evidentiary role of this file.
    #[must_use]
    pub const fn role(self) -> ArchiveFileRole {
        self.role
    }
}

/// The two carrier defects the archived ceremonies can report.
///
/// No completeness or bound-standing variant exists. That omission is the type-system gate that
/// prevents this archive-only lane from manufacturing a standing change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveIncompleteness {
    /// The transcript displays request lengths but not the exact sent bytes.
    MissingRequestBytes,
    /// Exact requests exist, but the complete executor handshake does not.
    MissingExecutorProvenance,
}

/// One parsed ceremony. Every value is necessarily incomplete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchivedCeremony {
    name: &'static str,
    incompleteness: ArchiveIncompleteness,
}

impl ArchivedCeremony {
    /// Archive basename that uniquely identifies this ceremony.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Carrier defect that refused this ceremony before any standing change.
    #[must_use]
    pub const fn incompleteness(self) -> ArchiveIncompleteness {
        self.incompleteness
    }
}

/// Parsed rerun-day corpus containing one entry per canonical ceremony.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RerunDayCorpus {
    ceremonies: Vec<ArchivedCeremony>,
}

impl RerunDayCorpus {
    /// Separate, deterministically ordered ceremonies.
    #[must_use]
    pub fn ceremonies(&self) -> &[ArchivedCeremony] {
        &self.ceremonies
    }

    /// Cross-foot of the only two representable carrier defects.
    #[must_use]
    pub fn incompleteness_census(&self) -> ArchiveIncompletenessCensus {
        self.ceremonies.iter().fold(
            ArchiveIncompletenessCensus::default(),
            |mut census, ceremony| {
                match ceremony.incompleteness {
                    ArchiveIncompleteness::MissingRequestBytes => {
                        census.missing_request_bytes += 1;
                    }
                    ArchiveIncompleteness::MissingExecutorProvenance => {
                        census.missing_executor_provenance += 1;
                    }
                }
                census
            },
        )
    }
}

/// Census of typed archive incompleteness.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArchiveIncompletenessCensus {
    /// Ceremonies missing exact request bytes.
    pub missing_request_bytes: usize,
    /// Ceremonies missing complete executor provenance.
    pub missing_executor_provenance: usize,
}

/// A single strictly parsed transcript, still carrying only incompleteness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedRerunDayTranscript<'a> {
    name: &'a str,
    incompleteness: ArchiveIncompleteness,
}

impl<'a> ParsedRerunDayTranscript<'a> {
    /// Parsed archive basename.
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.name
    }

    /// Carrier defect found before any standing could change.
    #[must_use]
    pub const fn incompleteness(self) -> ArchiveIncompleteness {
        self.incompleteness
    }
}

/// Strict archive or transcript refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RerunDayCorpusRefusal {
    /// The manifest did not contain exactly 79 entries.
    ArchiveFileCount { expected: usize, actual: usize },
    /// The manifest entries were not strictly sorted and unique.
    ManifestOrder { name: String },
    /// A generated manifest digest was not 32-byte hexadecimal.
    InvalidManifestDigest { name: String },
    /// Exact bytes no longer match their generated manifest entry.
    ManifestDigestMismatch { name: String },
    /// Rendering the deterministic manifest unexpectedly failed.
    ManifestRendering,
    /// The whole rendered manifest hash changed.
    ManifestHashMismatch,
    /// The archived byte total changed.
    ArchiveByteCount { expected: usize, actual: usize },
    /// The caller named a non-canonical or unknown transcript.
    UnknownTranscript { name: String },
    /// UTF-8 decoding failed.
    InvalidUtf8 { name: String },
    /// A carriage return would make line-ending normalization ambiguous.
    CarriageReturn { name: String },
    /// The transcript lacked its final LF byte.
    MissingTerminalLf { name: String },
    /// A line was empty or did not use the exact `field value` shape.
    MalformedLine { name: String, line: usize },
    /// A field was not part of the transcript's closed schema.
    UnknownField { name: String, field: String },
    /// A singleton field appeared more than once.
    DuplicateField { name: String, field: String },
    /// A required field was absent.
    MissingField { name: String, field: String },
    /// More than one ceremony boundary was expressible in one file.
    AmbiguousRunBoundary { name: String },
    /// A displayed integer was not canonical unsigned decimal.
    InvalidNumber { name: String, field: String },
    /// A displayed count disagreed with the records it counts.
    CountMismatch {
        name: String,
        field: String,
        displayed: usize,
        observed: usize,
    },
    /// A hexadecimal carrier contained a non-hex digit or odd nibble.
    MalformedHex { name: String, field: String },
    /// A fixed-width carrier decoded to the wrong number of bytes.
    WrongByteLength {
        name: String,
        field: String,
        expected: usize,
        actual: usize,
    },
    /// A request's displayed byte count disagreed with its decoded bytes.
    DisplayedByteCountMismatch {
        name: String,
        request: String,
        displayed: usize,
        decoded: usize,
    },
}

/// All 79 byte-embedded archive files in manifest order.
#[must_use]
pub const fn archive_files() -> &'static [ArchiveFile] {
    &ARCHIVE_FILES
}

/// Verify every file digest, total byte count, and the whole generated manifest hash.
///
/// # Errors
///
/// Returns [`RerunDayCorpusRefusal`] at the first file-count, order, digest, byte-count, or
/// whole-manifest mismatch.
pub fn validate_archive_manifest() -> Result<(), RerunDayCorpusRefusal> {
    let inputs = ARCHIVE_FILES
        .iter()
        .map(|file| ManifestInput {
            name: file.name,
            bytes: file.bytes,
            expected_sha256: file.expected_sha256,
        })
        .collect::<Vec<_>>();
    validate_manifest_inputs(&inputs)
}

/// Strictly parse the canonical archive and return only its typed incompleteness.
///
/// # Errors
///
/// Returns [`RerunDayCorpusRefusal`] if the manifest or any canonical transcript fails its
/// closed schema.
pub fn parse_rerun_day_archive() -> Result<RerunDayCorpus, RerunDayCorpusRefusal> {
    validate_archive_manifest()?;
    let ceremonies = ARCHIVE_FILES
        .iter()
        .filter(|file| file.role == ArchiveFileRole::CanonicalTranscript)
        .map(|file| {
            parse_rerun_day_transcript(file.name, file.bytes).map(|parsed| ArchivedCeremony {
                name: file.name,
                incompleteness: parsed.incompleteness,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if ceremonies.len() != CANONICAL_TRANSCRIPT_COUNT {
        return Err(RerunDayCorpusRefusal::ArchiveFileCount {
            expected: CANONICAL_TRANSCRIPT_COUNT,
            actual: ceremonies.len(),
        });
    }
    Ok(RerunDayCorpus { ceremonies })
}

/// Parse one canonical transcript under its exact closed field schema.
///
/// # Errors
///
/// Returns [`RerunDayCorpusRefusal`] if `name` is not a canonical transcript or if `bytes`
/// violate its byte, field, count, hexadecimal, or run-boundary requirements.
pub fn parse_rerun_day_transcript<'a>(
    name: &'a str,
    bytes: &'a [u8],
) -> Result<ParsedRerunDayTranscript<'a>, RerunDayCorpusRefusal> {
    let schema = schema_for(name).ok_or_else(|| RerunDayCorpusRefusal::UnknownTranscript {
        name: name.to_owned(),
    })?;
    let parsed = parse_lines(name, bytes, schema)?;
    validate_numeric_fields(&parsed)?;
    validate_hex_fields(&parsed)?;
    validate_schema_counts(&parsed, schema.kind)?;
    if schema.kind == SchemaKind::Keypath {
        validate_keypath(&parsed)?;
    }

    Ok(ParsedRerunDayTranscript {
        name,
        incompleteness: if schema.kind == SchemaKind::Keypath {
            ArchiveIncompleteness::MissingExecutorProvenance
        } else {
            ArchiveIncompleteness::MissingRequestBytes
        },
    })
}

#[derive(Clone, Copy)]
struct ManifestInput<'a> {
    name: &'a str,
    bytes: &'a [u8],
    expected_sha256: &'a str,
}

fn validate_manifest_inputs(inputs: &[ManifestInput<'_>]) -> Result<(), RerunDayCorpusRefusal> {
    if inputs.len() != ARCHIVE_FILE_COUNT {
        return Err(RerunDayCorpusRefusal::ArchiveFileCount {
            expected: ARCHIVE_FILE_COUNT,
            actual: inputs.len(),
        });
    }

    let mut previous = None;
    let mut total_bytes = 0;
    let mut rendered = String::new();
    for input in inputs {
        if previous.is_some_and(|name| name >= input.name) {
            return Err(RerunDayCorpusRefusal::ManifestOrder {
                name: input.name.to_owned(),
            });
        }
        previous = Some(input.name);
        total_bytes += input.bytes.len();

        let expected = decode_hex(input.expected_sha256).map_err(|()| {
            RerunDayCorpusRefusal::InvalidManifestDigest {
                name: input.name.to_owned(),
            }
        })?;
        let expected: [u8; 32] =
            expected
                .try_into()
                .map_err(|_| RerunDayCorpusRefusal::InvalidManifestDigest {
                    name: input.name.to_owned(),
                })?;
        if tagged::sha256(input.bytes) != expected {
            return Err(RerunDayCorpusRefusal::ManifestDigestMismatch {
                name: input.name.to_owned(),
            });
        }
        writeln!(&mut rendered, "{}  {}", input.expected_sha256, input.name)
            .map_err(|_| RerunDayCorpusRefusal::ManifestRendering)?;
    }

    if total_bytes != ARCHIVE_TOTAL_BYTES {
        return Err(RerunDayCorpusRefusal::ArchiveByteCount {
            expected: ARCHIVE_TOTAL_BYTES,
            actual: total_bytes,
        });
    }
    let expected_manifest: [u8; 32] = decode_hex(ARCHIVE_MANIFEST_SHA256)
        .map_err(|()| RerunDayCorpusRefusal::ManifestHashMismatch)?
        .try_into()
        .map_err(|_| RerunDayCorpusRefusal::ManifestHashMismatch)?;
    if tagged::sha256(rendered.as_bytes()) != expected_manifest {
        return Err(RerunDayCorpusRefusal::ManifestHashMismatch);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SchemaKind {
    Explicit,
    Multi,
    Sponsor,
    SponsorPrivate,
    Conservation,
    Keypath,
    OwnerObservation,
    OwnerSigning,
    Pairs,
    PrivateRestart,
    ProofBearing,
    Report,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransactionIdField {
    Present,
    Absent,
}

#[derive(Clone, Copy)]
struct TranscriptSchema {
    kind: SchemaKind,
    boundary: &'static str,
    allowed: &'static [&'static str],
    repeated: &'static [&'static str],
    required: &'static [&'static str],
}

struct ParsedLines<'a> {
    name: &'a str,
    lines: Vec<(&'a str, &'a str)>,
    counts: HashMap<&'a str, usize>,
    singletons: HashMap<&'a str, &'a str>,
}

const BOUNDARY_FIELDS: &[&str] = &["case", "explicit_shape", "role", "run", "sponsor_shape"];

fn parse_lines<'a>(
    name: &'a str,
    bytes: &'a [u8],
    schema: TranscriptSchema,
) -> Result<ParsedLines<'a>, RerunDayCorpusRefusal> {
    if bytes.contains(&b'\r') {
        return Err(RerunDayCorpusRefusal::CarriageReturn {
            name: name.to_owned(),
        });
    }
    let text = std::str::from_utf8(bytes).map_err(|_| RerunDayCorpusRefusal::InvalidUtf8 {
        name: name.to_owned(),
    })?;
    let body = text
        .strip_suffix('\n')
        .ok_or_else(|| RerunDayCorpusRefusal::MissingTerminalLf {
            name: name.to_owned(),
        })?;

    let mut lines = Vec::new();
    let mut counts = HashMap::new();
    let mut singletons = HashMap::new();
    for (offset, line) in body.split('\n').enumerate() {
        let line_number = offset + 1;
        if line.is_empty()
            || line.starts_with(' ')
            || line.ends_with(' ')
            || line.as_bytes().contains(&b'\t')
        {
            return Err(RerunDayCorpusRefusal::MalformedLine {
                name: name.to_owned(),
                line: line_number,
            });
        }
        let (field, value) =
            line.split_once(' ')
                .ok_or_else(|| RerunDayCorpusRefusal::MalformedLine {
                    name: name.to_owned(),
                    line: line_number,
                })?;
        if value.is_empty() {
            return Err(RerunDayCorpusRefusal::MalformedLine {
                name: name.to_owned(),
                line: line_number,
            });
        }
        if BOUNDARY_FIELDS.contains(&field) && field != schema.boundary {
            return Err(RerunDayCorpusRefusal::AmbiguousRunBoundary {
                name: name.to_owned(),
            });
        }
        if !schema.allowed.contains(&field) {
            return Err(RerunDayCorpusRefusal::UnknownField {
                name: name.to_owned(),
                field: field.to_owned(),
            });
        }

        let count = counts.entry(field).or_insert(0);
        *count += 1;
        if *count > 1 && !schema.repeated.contains(&field) {
            if field == schema.boundary {
                return Err(RerunDayCorpusRefusal::AmbiguousRunBoundary {
                    name: name.to_owned(),
                });
            }
            return Err(RerunDayCorpusRefusal::DuplicateField {
                name: name.to_owned(),
                field: field.to_owned(),
            });
        }
        if *count == 1 {
            singletons.insert(field, value);
        }
        lines.push((field, value));
    }

    for required in schema.required {
        if !counts.contains_key(required) {
            return Err(RerunDayCorpusRefusal::MissingField {
                name: name.to_owned(),
                field: (*required).to_owned(),
            });
        }
    }
    Ok(ParsedLines {
        name,
        lines,
        counts,
        singletons,
    })
}

fn validate_numeric_fields(parsed: &ParsedLines<'_>) -> Result<(), RerunDayCorpusRefusal> {
    const NUMERIC_FIELDS: &[&str] = &[
        "control_submitted_bytes",
        "expected_output_count",
        "finalized_bytes",
        "funded_coins",
        "input_count",
        "observed_weight",
        "output_count",
        "output_witness_vector_length",
        "readback_block_height",
        "receipt_leaves",
        "sponsor_witness_items",
        "submitted_bytes",
        "target_reported_weight",
        "target_weight",
    ];
    for field in NUMERIC_FIELDS {
        if let Some(value) = parsed.singletons.get(field) {
            parse_usize(parsed.name, field, value)?;
        }
    }
    for (field, value) in &parsed.lines {
        if matches!(*field, "consensus_mutant" | "leaf_arrangement" | "mutant")
            && let Some(range) = token_after(value, "declared_range")
        {
            validate_range(parsed.name, field, range)?;
        }
    }
    Ok(())
}

fn validate_schema_counts(
    parsed: &ParsedLines<'_>,
    kind: SchemaKind,
) -> Result<(), RerunDayCorpusRefusal> {
    match kind {
        SchemaKind::Explicit => {
            match_count(parsed, "funded_coins", "coin")?;
            match_count(parsed, "input_count", "input")?;
            match_count(parsed, "output_count", "destination")?;
        }
        SchemaKind::Multi => {
            let displayed = singleton_usize(parsed, "output_count")?;
            let observed = parse_number_list(
                parsed.name,
                "output_witness_proof_bytes",
                singleton(parsed, "output_witness_proof_bytes")?,
            )?
            .len();
            refuse_count_mismatch(parsed.name, "output_count", displayed, observed)?;
        }
        SchemaKind::Sponsor => {
            match_count(
                parsed,
                "sponsor_witness_items",
                "sponsor_witness_item_bytes",
            )?;
        }
        SchemaKind::ProofBearing => {
            match_count(parsed, "output_witness_vector_length", "output_witness")?;
        }
        SchemaKind::SponsorPrivate
        | SchemaKind::Conservation
        | SchemaKind::Keypath
        | SchemaKind::OwnerObservation
        | SchemaKind::OwnerSigning
        | SchemaKind::Pairs
        | SchemaKind::PrivateRestart
        | SchemaKind::Report => {}
    }
    Ok(())
}

fn validate_hex_fields(parsed: &ParsedLines<'_>) -> Result<(), RerunDayCorpusRefusal> {
    for (field, value) in &parsed.lines {
        match *field {
            "accepted_txid" => {
                parse_transaction_id_field(parsed.name, field, value)?;
            }
            "committed_sponsor_funding_txid"
            | "control_accepted_txid"
            | "issued_asset"
            | "predecessor_digest"
            | "predecessor_fixture_digest"
            | "readback_txid"
            | "readback_witness_txid"
            | "sponsor_funding_txid"
            | "successor_digest" => validate_fixed_hex(parsed.name, field, value, 32)?,
            "consumed_commitment_prefix" => {
                validate_prefixed_hex(parsed.name, field, value, 1)?;
            }
            "input" | "leaf_arrangement" | "mutant" => {
                validate_token_hex(parsed.name, field, value, "message", 32)?;
            }
            "message" => {
                let digest = value.split_whitespace().last().unwrap_or_default();
                validate_fixed_hex(parsed.name, field, digest, 32)?;
            }
            "observed" => {
                validate_optional_transaction_id(parsed.name, field, value, "accepted_txid")?;
                validate_optional_transaction_id(parsed.name, field, value, "txid")?;
                if let Some(txid) = value
                    .split_whitespace()
                    .find_map(|token| token.strip_prefix("txid="))
                {
                    parse_transaction_id_field(parsed.name, field, txid)?;
                }
            }
            "control" => {
                validate_token_hex(parsed.name, field, value, "message", 32)?;
                validate_optional_transaction_id(parsed.name, field, value, "txid")?;
            }
            "control_observed" | "negative" => {
                validate_optional_transaction_id(parsed.name, field, value, "accepted_txid")?;
            }
            "provenance" => validate_provenance_hex(parsed.name, field, value)?,
            "binding" => validate_binding_hex(parsed.name, field, value)?,
            "coin" => validate_coin_hex(parsed.name, field, value)?,
            "reverification" => validate_reverification_hex(parsed.name, field, value)?,
            "spent_value_prefix" => {
                let prefix = value.split_whitespace().last().unwrap_or_default();
                validate_prefixed_hex(parsed.name, field, prefix, 1)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_keypath(parsed: &ParsedLines<'_>) -> Result<(), RerunDayCorpusRefusal> {
    validate_subkeys(
        parsed,
        "provenance",
        &[
            "declared_source_tip",
            "executor_trust",
            "genesis_id",
            "network_id",
            "target_version",
        ],
    )?;
    validate_subkeys(
        parsed,
        "binding",
        &[
            "internal_key",
            "internal_key_is_the_published_point",
            "merkle_root",
            "output_key",
            "program",
        ],
    )?;
    validate_subkeys(
        parsed,
        "attempt",
        &[
            "candidate_key_path_message",
            "signing_key_is_the_output_key",
            "signing_public_key",
            "spent_outpoint_index",
            "submitted",
            "submitted_bytes",
            "witness_item",
            "witness_items",
        ],
    )?;
    validate_subkeys(
        parsed,
        "observed",
        &["accepted_txid", "detail", "is_target_verdict", "layer"],
    )?;
    validate_subkeys(
        parsed,
        "control",
        &[
            "shares_the_attempts_witnessless_bytes",
            "submitted",
            "submitted_bytes",
            "witness_items",
        ],
    )?;
    validate_subkeys(
        parsed,
        "control_observed",
        &["accepted_txid", "detail", "is_target_verdict", "layer"],
    )?;

    validate_request_bytes(parsed, "attempt")?;
    validate_request_bytes(parsed, "control")?;
    let displayed_items = contextual_usize(parsed, "attempt", "witness_items")?;
    let observed_items = parsed
        .lines
        .iter()
        .filter(|(field, value)| *field == "attempt" && value.starts_with("witness_item "))
        .count();
    refuse_count_mismatch(
        parsed.name,
        "attempt witness_items",
        displayed_items,
        observed_items,
    )?;

    for (field, value) in &parsed.lines {
        if *field != "attempt" {
            continue;
        }
        if let Some(hex) = value.strip_prefix("witness_item 0 ") {
            validate_fixed_hex(parsed.name, "attempt witness_item", hex, 64)?;
        } else if let Some(hex) = value.strip_prefix("candidate_key_path_message ") {
            validate_fixed_hex(parsed.name, "attempt candidate_key_path_message", hex, 32)?;
        } else if let Some(hex) = value.strip_prefix("signing_public_key ") {
            validate_fixed_hex(parsed.name, "attempt signing_public_key", hex, 32)?;
        }
    }
    Ok(())
}

fn validate_request_bytes(
    parsed: &ParsedLines<'_>,
    request: &str,
) -> Result<(), RerunDayCorpusRefusal> {
    let displayed = contextual_usize(parsed, request, "submitted_bytes")?;
    let encoded = unique_contextual(parsed, request, "submitted")?;
    let decoded = decode_hex(encoded).map_err(|()| RerunDayCorpusRefusal::MalformedHex {
        name: parsed.name.to_owned(),
        field: format!("{request} submitted"),
    })?;
    if displayed != decoded.len() {
        return Err(RerunDayCorpusRefusal::DisplayedByteCountMismatch {
            name: parsed.name.to_owned(),
            request: request.to_owned(),
            displayed,
            decoded: decoded.len(),
        });
    }
    Ok(())
}

fn validate_subkeys(
    parsed: &ParsedLines<'_>,
    field: &str,
    allowed: &[&str],
) -> Result<(), RerunDayCorpusRefusal> {
    let mut seen = HashMap::new();
    for (_, value) in parsed
        .lines
        .iter()
        .filter(|(candidate, _)| *candidate == field)
    {
        let subkey = value.split_whitespace().next().unwrap_or_default();
        if !allowed.contains(&subkey) {
            return Err(RerunDayCorpusRefusal::UnknownField {
                name: parsed.name.to_owned(),
                field: format!("{field} {subkey}"),
            });
        }
        if subkey != "witness_item" {
            let count = seen.entry(subkey).or_insert(0);
            *count += 1;
            if *count > 1 {
                return Err(RerunDayCorpusRefusal::DuplicateField {
                    name: parsed.name.to_owned(),
                    field: format!("{field} {subkey}"),
                });
            }
        }
    }
    Ok(())
}

fn validate_provenance_hex(
    name: &str,
    field: &str,
    value: &str,
) -> Result<(), RerunDayCorpusRefusal> {
    if let Some(hex) = value.strip_prefix("network_id ") {
        validate_fixed_hex(name, field, hex, 32)?;
    } else if let Some(hex) = value.strip_prefix("genesis_id ") {
        validate_fixed_hex(name, field, hex, 32)?;
    } else if let Some(hex) = value.strip_prefix("declared_source_tip ") {
        validate_fixed_hex(name, field, hex, 20)?;
    }
    Ok(())
}

fn validate_binding_hex(name: &str, field: &str, value: &str) -> Result<(), RerunDayCorpusRefusal> {
    if let Some(hex) = value.strip_prefix("program ") {
        validate_fixed_hex(name, field, hex, 34)?;
    } else if let Some(hex) = value.strip_prefix("internal_key ") {
        validate_fixed_hex(name, field, hex, 32)?;
    } else if let Some(hex) = value.strip_prefix("output_key ") {
        validate_fixed_hex(name, field, hex, 32)?;
    } else if let Some(hex) = value.strip_prefix("merkle_root ") {
        validate_fixed_hex(name, field, hex, 32)?;
    }
    Ok(())
}

fn validate_coin_hex(name: &str, field: &str, value: &str) -> Result<(), RerunDayCorpusRefusal> {
    let tokens = value.split_whitespace().collect::<Vec<_>>();
    if let Some(hex) = tokens
        .windows(3)
        .find_map(|parts| (parts[0] == "asset" && parts[1] == "explicit").then_some(parts[2]))
    {
        validate_fixed_hex(name, field, hex, 32)?;
    }
    if let Some(hex) = tokens
        .windows(3)
        .find_map(|parts| (parts[0] == "value" && parts[1] == "commitment").then_some(parts[2]))
    {
        validate_fixed_hex(name, field, hex, 33)?;
    }
    validate_token_hex(name, field, value, "program", 34)
}

fn validate_reverification_hex(
    name: &str,
    field: &str,
    value: &str,
) -> Result<(), RerunDayCorpusRefusal> {
    let Some((subkey, hex)) = value.split_once(' ') else {
        return Ok(());
    };
    match subkey {
        "accepted_txid" => {
            parse_transaction_id_field(name, field, hex)?;
        }
        "recomputed_message" | "witness_txid" => validate_fixed_hex(name, field, hex, 32)?,
        "signature_from_readback" => validate_fixed_hex(name, field, hex, 64)?,
        _ => {}
    }
    Ok(())
}

fn validate_token_hex(
    name: &str,
    field: &str,
    value: &str,
    key: &str,
    bytes: usize,
) -> Result<(), RerunDayCorpusRefusal> {
    if let Some(hex) = token_after(value, key) {
        validate_fixed_hex(name, field, hex, bytes)?;
    }
    Ok(())
}

fn parse_transaction_id_field(
    name: &str,
    field: &str,
    value: &str,
) -> Result<TransactionIdField, RerunDayCorpusRefusal> {
    match value {
        "none" => Ok(TransactionIdField::Absent),
        hex => {
            validate_fixed_hex(name, field, hex, 32)?;
            Ok(TransactionIdField::Present)
        }
    }
}

fn validate_optional_transaction_id(
    name: &str,
    field: &str,
    value: &str,
    key: &str,
) -> Result<(), RerunDayCorpusRefusal> {
    if let Some(txid) = token_after(value, key) {
        parse_transaction_id_field(name, field, txid)?;
    }
    Ok(())
}

fn token_after<'a>(value: &'a str, key: &str) -> Option<&'a str> {
    let tokens = value.split_whitespace().collect::<Vec<_>>();
    tokens
        .windows(2)
        .find_map(|pair| (pair[0] == key).then_some(pair[1]))
}

fn validate_fixed_hex(
    name: &str,
    field: &str,
    value: &str,
    expected: usize,
) -> Result<(), RerunDayCorpusRefusal> {
    let decoded = decode_hex(value).map_err(|()| RerunDayCorpusRefusal::MalformedHex {
        name: name.to_owned(),
        field: field.to_owned(),
    })?;
    if decoded.len() != expected {
        return Err(RerunDayCorpusRefusal::WrongByteLength {
            name: name.to_owned(),
            field: field.to_owned(),
            expected,
            actual: decoded.len(),
        });
    }
    Ok(())
}

fn validate_prefixed_hex(
    name: &str,
    field: &str,
    value: &str,
    expected: usize,
) -> Result<(), RerunDayCorpusRefusal> {
    let hex = value
        .strip_prefix("0x")
        .ok_or_else(|| RerunDayCorpusRefusal::MalformedHex {
            name: name.to_owned(),
            field: field.to_owned(),
        })?;
    validate_fixed_hex(name, field, hex, expected)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, ()> {
    if !value.len().is_multiple_of(2) || value.is_empty() {
        return Err(());
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = hex_nibble(pair[0]).ok_or(())?;
            let low = hex_nibble(pair[1]).ok_or(())?;
            Ok((high << 4) | low)
        })
        .collect()
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn singleton<'a>(
    parsed: &'a ParsedLines<'_>,
    field: &str,
) -> Result<&'a str, RerunDayCorpusRefusal> {
    parsed
        .singletons
        .get(field)
        .copied()
        .ok_or_else(|| RerunDayCorpusRefusal::MissingField {
            name: parsed.name.to_owned(),
            field: field.to_owned(),
        })
}

fn singleton_usize(parsed: &ParsedLines<'_>, field: &str) -> Result<usize, RerunDayCorpusRefusal> {
    parse_usize(parsed.name, field, singleton(parsed, field)?)
}

fn contextual_usize(
    parsed: &ParsedLines<'_>,
    field: &str,
    subkey: &str,
) -> Result<usize, RerunDayCorpusRefusal> {
    parse_usize(
        parsed.name,
        &format!("{field} {subkey}"),
        unique_contextual(parsed, field, subkey)?,
    )
}

fn unique_contextual<'a>(
    parsed: &'a ParsedLines<'_>,
    field: &str,
    subkey: &str,
) -> Result<&'a str, RerunDayCorpusRefusal> {
    let prefix = format!("{subkey} ");
    let mut matches = parsed
        .lines
        .iter()
        .filter(|(candidate, value)| *candidate == field && value.starts_with(&prefix))
        .map(|(_, value)| &value[prefix.len()..]);
    let first = matches
        .next()
        .ok_or_else(|| RerunDayCorpusRefusal::MissingField {
            name: parsed.name.to_owned(),
            field: format!("{field} {subkey}"),
        })?;
    if matches.next().is_some() {
        return Err(RerunDayCorpusRefusal::DuplicateField {
            name: parsed.name.to_owned(),
            field: format!("{field} {subkey}"),
        });
    }
    Ok(first)
}

fn match_count(
    parsed: &ParsedLines<'_>,
    displayed_field: &str,
    observed_field: &str,
) -> Result<(), RerunDayCorpusRefusal> {
    let displayed = singleton_usize(parsed, displayed_field)?;
    let observed = parsed.counts.get(observed_field).copied().unwrap_or(0);
    refuse_count_mismatch(parsed.name, displayed_field, displayed, observed)
}

fn refuse_count_mismatch(
    name: &str,
    field: &str,
    displayed: usize,
    observed: usize,
) -> Result<(), RerunDayCorpusRefusal> {
    if displayed != observed {
        return Err(RerunDayCorpusRefusal::CountMismatch {
            name: name.to_owned(),
            field: field.to_owned(),
            displayed,
            observed,
        });
    }
    Ok(())
}

fn parse_usize(name: &str, field: &str, value: &str) -> Result<usize, RerunDayCorpusRefusal> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(RerunDayCorpusRefusal::InvalidNumber {
            name: name.to_owned(),
            field: field.to_owned(),
        });
    }
    value
        .parse()
        .map_err(|_| RerunDayCorpusRefusal::InvalidNumber {
            name: name.to_owned(),
            field: field.to_owned(),
        })
}

fn validate_range(name: &str, field: &str, value: &str) -> Result<(), RerunDayCorpusRefusal> {
    let (start, end) =
        value
            .split_once("..")
            .ok_or_else(|| RerunDayCorpusRefusal::InvalidNumber {
                name: name.to_owned(),
                field: field.to_owned(),
            })?;
    let start = parse_usize(name, field, start)?;
    let end = parse_usize(name, field, end)?;
    if start >= end {
        return Err(RerunDayCorpusRefusal::InvalidNumber {
            name: name.to_owned(),
            field: field.to_owned(),
        });
    }
    Ok(())
}

fn parse_number_list(
    name: &str,
    field: &str,
    value: &str,
) -> Result<Vec<usize>, RerunDayCorpusRefusal> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| RerunDayCorpusRefusal::InvalidNumber {
            name: name.to_owned(),
            field: field.to_owned(),
        })?;
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(", ")
        .map(|number| parse_usize(name, field, number))
        .collect()
}

fn schema_for(name: &str) -> Option<TranscriptSchema> {
    let suffix = name.strip_prefix("rerun-0.6.52-dev.")?;
    shape_schema(suffix).or_else(|| special_schema(suffix))
}

fn shape_schema(suffix: &str) -> Option<TranscriptSchema> {
    let schema = match suffix {
        "explicit-witness-negatives" => TranscriptSchema {
            kind: SchemaKind::Explicit,
            boundary: "explicit_shape",
            allowed: EXPLICIT_NEGATIVE_FIELDS,
            repeated: EXPLICIT_REPEATED_FIELDS,
            required: EXPLICIT_REQUIRED_FIELDS,
        },
        "explicit-boundary-values"
        | "explicit-maximum-inputs"
        | "explicit-maximum-outputs"
        | "explicit-merge"
        | "explicit-normalization"
        | "explicit-one-destination-owner"
        | "explicit-one-to-one"
        | "explicit-repeated-owner"
        | "explicit-self-paid-fee"
        | "explicit-several-destination-owners"
        | "explicit-several-owners"
        | "explicit-several-to-several"
        | "explicit-split"
        | "explicit-sponsorless" => TranscriptSchema {
            kind: SchemaKind::Explicit,
            boundary: "explicit_shape",
            allowed: EXPLICIT_FIELDS,
            repeated: EXPLICIT_REPEATED_FIELDS,
            required: EXPLICIT_REQUIRED_FIELDS,
        },
        "multi-entry-crossing"
        | "multi-exit-crossing"
        | "multi-many-to-many"
        | "multi-one-to-one-with-fee"
        | "multi-private-merge"
        | "multi-pure-split"
        | "multi-several-owners"
        | "multi-split"
        | "multi-strict-one-to-one" => TranscriptSchema {
            kind: SchemaKind::Multi,
            boundary: "run",
            allowed: MULTI_FIELDS,
            repeated: &["coin"],
            required: MULTI_REQUIRED_FIELDS,
        },
        "sponsored-change-absent"
        | "sponsored-change-present"
        | "sponsored-committed-value"
        | "sponsored-missing-authorization" => TranscriptSchema {
            kind: SchemaKind::Sponsor,
            boundary: "sponsor_shape",
            allowed: SPONSOR_FIELDS,
            repeated: SPONSOR_REPEATED_FIELDS,
            required: SPONSOR_REQUIRED_FIELDS,
        },
        "sponsored-private-explicit-no-change" | "sponsored-private-with-change" => {
            TranscriptSchema {
                kind: SchemaKind::SponsorPrivate,
                boundary: "case",
                allowed: SPONSOR_PRIVATE_FIELDS,
                repeated: &["sponsor_check"],
                required: SPONSOR_PRIVATE_REQUIRED_FIELDS,
            }
        }
        _ => return None,
    };
    Some(schema)
}

fn special_schema(suffix: &str) -> Option<TranscriptSchema> {
    let schema = match suffix {
        "conservation-negatives" => TranscriptSchema {
            kind: SchemaKind::Conservation,
            boundary: "run",
            allowed: CONSERVATION_FIELDS,
            repeated: &["mutant"],
            required: CONSERVATION_REQUIRED_FIELDS,
        },
        "keypath-probe" => TranscriptSchema {
            kind: SchemaKind::Keypath,
            boundary: "role",
            allowed: KEYPATH_FIELDS,
            repeated: KEYPATH_REPEATED_FIELDS,
            required: KEYPATH_REQUIRED_FIELDS,
        },
        "owner-observation" => TranscriptSchema {
            kind: SchemaKind::OwnerObservation,
            boundary: "role",
            allowed: OWNER_OBSERVATION_FIELDS,
            repeated: OWNER_OBSERVATION_REPEATED_FIELDS,
            required: OWNER_OBSERVATION_REQUIRED_FIELDS,
        },
        "owner-signing-negatives" => TranscriptSchema {
            kind: SchemaKind::OwnerSigning,
            boundary: "role",
            allowed: OWNER_SIGNING_FIELDS,
            repeated: OWNER_SIGNING_REPEATED_FIELDS,
            required: OWNER_SIGNING_REQUIRED_FIELDS,
        },
        "pairs-arc" => TranscriptSchema {
            kind: SchemaKind::Pairs,
            boundary: "run",
            allowed: PAIRS_FIELDS,
            repeated: PAIRS_REPEATED_FIELDS,
            required: PAIRS_REQUIRED_FIELDS,
        },
        "private-restart-control" | "private-restart-parity" => TranscriptSchema {
            kind: SchemaKind::PrivateRestart,
            boundary: "run",
            allowed: PRIVATE_RESTART_FIELDS,
            repeated: &["coin"],
            required: PRIVATE_RESTART_REQUIRED_FIELDS,
        },
        "proof-bearing-observation" => TranscriptSchema {
            kind: SchemaKind::ProofBearing,
            boundary: "role",
            allowed: PROOF_BEARING_FIELDS,
            repeated: PROOF_BEARING_REPEATED_FIELDS,
            required: PROOF_BEARING_REQUIRED_FIELDS,
        },
        "report" => TranscriptSchema {
            kind: SchemaKind::Report,
            boundary: "role",
            allowed: REPORT_FIELDS,
            repeated: &["materialized", "observed"],
            required: REPORT_REQUIRED_FIELDS,
        },
        _ => return None,
    };
    Some(schema)
}

const EXPLICIT_FIELDS: &[&str] = &[
    "accepted_txid",
    "builds_no_sponsor_region",
    "coin",
    "construction_refusal",
    "destination",
    "does_not_establish",
    "every_input_verified",
    "evidences_no_negative_case",
    "explicit_shape",
    "funded_coins",
    "input",
    "input_count",
    "input_owner",
    "issued_asset",
    "observed_detail",
    "observed_layer",
    "observed_weight",
    "offered_order_differs",
    "output_count",
    "readback_block_height",
    "readback_matches_submission",
    "readback_txid",
    "readback_witness_txid",
    "relinked",
    "row_name",
    "submitted_bytes",
];

const EXPLICIT_NEGATIVE_FIELDS: &[&str] = &[
    "accepted_txid",
    "builds_no_sponsor_region",
    "coin",
    "construction_refusal",
    "destination",
    "does_not_establish",
    "every_input_verified",
    "evidences_no_negative_case",
    "explicit_shape",
    "funded_coins",
    "input",
    "input_count",
    "input_owner",
    "issued_asset",
    "negative",
    "observed_detail",
    "observed_layer",
    "observed_weight",
    "offered_order_differs",
    "output_count",
    "readback_block_height",
    "readback_matches_submission",
    "readback_txid",
    "readback_witness_txid",
    "relinked",
    "row_name",
    "submitted_bytes",
];

const EXPLICIT_REPEATED_FIELDS: &[&str] = &[
    "coin",
    "destination",
    "does_not_establish",
    "input",
    "input_owner",
    "negative",
];

const EXPLICIT_REQUIRED_FIELDS: &[&str] = &[
    "accepted_txid",
    "explicit_shape",
    "funded_coins",
    "input_count",
    "issued_asset",
    "observed_layer",
    "output_count",
    "row_name",
    "submitted_bytes",
];

const MULTI_FIELDS: &[&str] = &[
    "accepted_txid",
    "coin",
    "construction_refusal",
    "evidences_no_minimality_relation",
    "evidences_no_negative_case",
    "forced_blinder",
    "input_count",
    "issued_asset",
    "moves_the_sponsor_row",
    "observed_detail",
    "observed_layer",
    "output_count",
    "output_witness_proof_bytes",
    "produced_an_accepted_control",
    "receipt_leaves",
    "reverification",
    "run",
    "submitted_bytes",
    "successor_digest",
    "target_reported_weight",
];

const MULTI_REQUIRED_FIELDS: &[&str] = &[
    "accepted_txid",
    "input_count",
    "issued_asset",
    "observed_layer",
    "output_count",
    "output_witness_proof_bytes",
    "run",
    "submitted_bytes",
];

const SPONSOR_FIELDS: &[&str] = &[
    "accepted_txid",
    "adapter_echo_matches",
    "committed_sponsor_check",
    "committed_sponsor_every_check_held",
    "committed_sponsor_funding_txid",
    "committed_sponsor_funding_weight",
    "construction_refusal",
    "does_not_establish",
    "every_owner_verified",
    "evidences_no_negative_case",
    "expected_output_count",
    "finalized_bytes",
    "input",
    "issued_asset",
    "mutated_binding_refusal",
    "negative",
    "observed_detail",
    "observed_layer",
    "offered_change",
    "offered_fee",
    "readback_block_height",
    "readback_matches_submission",
    "readback_txid",
    "receipt_coins",
    "relinked",
    "replay_changed_the_control",
    "row_name",
    "sponsor_change_output",
    "sponsor_funded",
    "sponsor_input_position",
    "sponsor_shape",
    "sponsor_value_form",
    "sponsor_witness_in_readback",
    "sponsor_witness_item_bytes",
    "sponsor_witness_items",
    "submitted_bytes",
    "target_weight",
    "witness_reached_the_control",
];

const SPONSOR_REPEATED_FIELDS: &[&str] = &[
    "committed_sponsor_check",
    "does_not_establish",
    "input",
    "sponsor_witness_item_bytes",
];

const SPONSOR_REQUIRED_FIELDS: &[&str] = &[
    "accepted_txid",
    "finalized_bytes",
    "issued_asset",
    "observed_layer",
    "sponsor_shape",
    "sponsor_witness_items",
    "submitted_bytes",
];

const SPONSOR_PRIVATE_FIELDS: &[&str] = &[
    "accepted_txid",
    "case",
    "issued_asset",
    "observed_layer",
    "owner_signature_verified",
    "predecessor_coins",
    "readback_matches_submission",
    "sponsor_change_located",
    "sponsor_check",
    "sponsor_echo_matches_what_was_sent",
    "sponsor_funding_txid",
    "sponsor_input_position",
    "sponsor_witness_items",
    "submitted_bytes",
    "target_weight",
];

const SPONSOR_PRIVATE_REQUIRED_FIELDS: &[&str] = &[
    "accepted_txid",
    "case",
    "issued_asset",
    "observed_layer",
    "sponsor_witness_items",
    "submitted_bytes",
];

const CONSERVATION_FIELDS: &[&str] = &[
    "construction_refusal",
    "consumed_commitment_prefix",
    "control_accepted_txid",
    "control_observed_layer",
    "control_reverification",
    "control_submitted_bytes",
    "issued_asset",
    "moves_the_sponsor_row",
    "mutant",
    "predecessor_digest",
    "run",
    "successor_digest",
];

const CONSERVATION_REQUIRED_FIELDS: &[&str] = &[
    "control_accepted_txid",
    "control_observed_layer",
    "control_submitted_bytes",
    "issued_asset",
    "mutant",
    "run",
];

const KEYPATH_FIELDS: &[&str] = &[
    "answers_matrix_row",
    "attempt",
    "binding",
    "coin",
    "control",
    "control_observed",
    "discharges_no_residual",
    "issued_asset",
    "non_claim",
    "observed",
    "provenance",
    "relinked",
    "residual_internal_key_unspendability_stands",
    "role",
];

const KEYPATH_REPEATED_FIELDS: &[&str] = &[
    "attempt",
    "binding",
    "control",
    "control_observed",
    "non_claim",
    "observed",
    "provenance",
];

const KEYPATH_REQUIRED_FIELDS: &[&str] = &[
    "attempt",
    "binding",
    "control",
    "control_observed",
    "issued_asset",
    "observed",
    "provenance",
    "role",
];

const OWNER_OBSERVATION_FIELDS: &[&str] = &[
    "clears_sighash_profile_unreviewed",
    "coin",
    "construction_refusal",
    "discharges_no_matrix_row",
    "establishes_the_proof_bearing_lane",
    "issued_asset",
    "message",
    "non_claim",
    "observed",
    "observed_acceptance",
    "relinked",
    "reverification",
    "role",
];

const OWNER_OBSERVATION_REPEATED_FIELDS: &[&str] = &[
    "coin",
    "construction_refusal",
    "message",
    "non_claim",
    "observed",
    "reverification",
];

const OWNER_OBSERVATION_REQUIRED_FIELDS: &[&str] = &["issued_asset", "message", "observed", "role"];

const OWNER_SIGNING_FIELDS: &[&str] = &[
    "coin",
    "consensus_mutant",
    "control",
    "control_reverification",
    "each_row_by_its_own_mutant",
    "issued_asset",
    "leaf_arrangement",
    "messages_differ",
    "mutant",
    "non_claim",
    "relinked",
    "role",
];

const OWNER_SIGNING_REPEATED_FIELDS: &[&str] =
    &["coin", "consensus_mutant", "leaf_arrangement", "non_claim"];

const OWNER_SIGNING_REQUIRED_FIELDS: &[&str] = &[
    "consensus_mutant",
    "control",
    "issued_asset",
    "leaf_arrangement",
    "mutant",
    "role",
];

const PAIRS_FIELDS: &[&str] = &[
    "builds_no_sponsor_region",
    "entry_condition_cites",
    "evidences_no_negative_case",
    "fixture_amount",
    "fixture_conserves",
    "issued_asset",
    "member",
    "observed",
    "projection",
    "projections_are_equal",
    "refusal",
    "run",
    "supports_the_projection_equality_row",
    "term",
    "terms_withheld_by_the_private_member",
];

const PAIRS_REPEATED_FIELDS: &[&str] = &["member", "observed", "projection", "term"];

const PAIRS_REQUIRED_FIELDS: &[&str] = &[
    "issued_asset",
    "member",
    "observed",
    "projection",
    "run",
    "term",
];

const PRIVATE_RESTART_FIELDS: &[&str] = &[
    "accepted_txid",
    "coin",
    "construction_refusal",
    "consumed_commitment_prefix",
    "consumed_receipt",
    "evidences_no_conservation_claim",
    "evidences_no_minimality_relation",
    "evidences_no_negative_case",
    "issued_asset",
    "moves_the_sponsor_row",
    "observed_detail",
    "observed_layer",
    "output_witness_proof_bytes",
    "predecessor_digest",
    "produced_an_accepted_control",
    "receipt_leaves",
    "reverification",
    "run",
    "submitted_bytes",
    "successor_digest",
];

const PRIVATE_RESTART_REQUIRED_FIELDS: &[&str] = &[
    "accepted_txid",
    "issued_asset",
    "observed_layer",
    "run",
    "submitted_bytes",
];

const PROOF_BEARING_FIELDS: &[&str] = &[
    "coin",
    "construction_control",
    "discharges_no_matrix_row",
    "evidences_the_other_guides_blinding",
    "evidences_the_other_guides_funding",
    "evidences_the_other_guides_materialization",
    "evidences_the_receipt_covenant",
    "issued_asset",
    "message",
    "non_claim",
    "observed",
    "observed_acceptance",
    "output_witness",
    "output_witness_vector_length",
    "predecessor_fixture_digest",
    "reverification",
    "role",
    "run_of_record_projection",
    "run_of_record_v2",
    "spent_value_prefix",
];

const PROOF_BEARING_REPEATED_FIELDS: &[&str] = &[
    "coin",
    "construction_control",
    "message",
    "non_claim",
    "observed",
    "output_witness",
    "reverification",
    "spent_value_prefix",
];

const PROOF_BEARING_REQUIRED_FIELDS: &[&str] = &[
    "issued_asset",
    "message",
    "observed",
    "output_witness",
    "output_witness_vector_length",
    "role",
];

const REPORT_FIELDS: &[&str] = &[
    "discharges_no_matrix_row",
    "issued_asset",
    "materialized",
    "not_submitted",
    "observed",
    "predicted",
    "relinked",
    "role",
];

const REPORT_REQUIRED_FIELDS: &[&str] = &["issued_asset", "observed", "predicted", "role"];

macro_rules! archive_file {
    ($suffix:literal, $digest:literal, $role:expr) => {
        ArchiveFile {
            name: concat!("rerun-0.6.52-dev.", $suffix),
            bytes: include_bytes!(concat!("../fixtures/rerun-day/rerun-0.6.52-dev.", $suffix)),
            expected_sha256: $digest,
            role: $role,
        }
    };
}

macro_rules! canonical {
    ($suffix:literal, $digest:literal) => {
        archive_file!($suffix, $digest, ArchiveFileRole::CanonicalTranscript)
    };
}

macro_rules! timing {
    ($suffix:literal, $digest:literal) => {
        archive_file!($suffix, $digest, ArchiveFileRole::TimingSidecar)
    };
}

static ARCHIVE_FILES: [ArchiveFile; ARCHIVE_FILE_COUNT] = [
    archive_file!(
        "confidential-predecessor",
        "2f6597eace505a9048f06067197e03b0b597cae59239fa85f1ca37ca20d2283c",
        ArchiveFileRole::PredecessorSetup
    ),
    canonical!(
        "conservation-negatives",
        "ce572482fbe6aeba6de03ffbaedc4af5c4932142183ac55c56d163037dbaf5ad"
    ),
    timing!(
        "conservation-negatives.timing",
        "dbe0fcbd1918729707a753f61d23cead55d5cc80e004f50e310e8b3e4905cd8e"
    ),
    canonical!(
        "explicit-boundary-values",
        "d7606623f6bf6f24b5938ca4be57860b3b78701e5557a74bb16143244189dc37"
    ),
    timing!(
        "explicit-boundary-values.timing",
        "1736eb0c7992bf4210e1a1eee0891675a7e948250ae681d1e344001889ca94af"
    ),
    canonical!(
        "explicit-maximum-inputs",
        "81c75e7d5c56408f01dc8bab0187f47afc07f47e580b5567fd9bdfed3535d2c4"
    ),
    timing!(
        "explicit-maximum-inputs.timing",
        "6edccb14b73c050d4dff59e1c1e8015b431bd7814343440297ff04fc243c4e32"
    ),
    canonical!(
        "explicit-maximum-outputs",
        "817551f24abd8d59197480837c4a484bddb3efb10470d5ed53e2b4ec1fd5d30e"
    ),
    timing!(
        "explicit-maximum-outputs.timing",
        "14efd0637b1263ff35745599b1af20fe9402b1d03ce91abdc6c8bcf71d9c57d2"
    ),
    canonical!(
        "explicit-merge",
        "d1a73be943993363e90ec8895ca2f6f4473bda2108c8d93e81064245ebd3d539"
    ),
    timing!(
        "explicit-merge.timing",
        "e42f711e042016a611e8992517459a53f6a095919dd5f556d872a85841a056e8"
    ),
    canonical!(
        "explicit-normalization",
        "cc1d0d8d38c81fab59650484dc8505069fff92b336f7f364ff849f16b16a98be"
    ),
    timing!(
        "explicit-normalization.timing",
        "e42f711e042016a611e8992517459a53f6a095919dd5f556d872a85841a056e8"
    ),
    canonical!(
        "explicit-one-destination-owner",
        "76b45b7a607d80bbf2df2825aaa90ed2635b85230bc38502f2b54a8237b70132"
    ),
    timing!(
        "explicit-one-destination-owner.timing",
        "19e9fcc629618ac571c422bb5e91e10a842fc23f9716bd38a171a652b2c2eb80"
    ),
    canonical!(
        "explicit-one-to-one",
        "402433da1750def667863353f3f17716eafec8aba839c0c25a9751c55d190594"
    ),
    timing!(
        "explicit-one-to-one.timing",
        "6f43d9e35ec37523272886f88cc16145bceaf30c1df9bd68056678827a509551"
    ),
    canonical!(
        "explicit-repeated-owner",
        "793eb036611bfd373fa6140abe7322104a06f2ddb9561f528f295831b100d028"
    ),
    timing!(
        "explicit-repeated-owner.timing",
        "e7bb865794b493ccc3edb84b5452e635521e4734c9e32f242365a5f2589bf6b5"
    ),
    canonical!(
        "explicit-self-paid-fee",
        "b2ed7e4b673e71ada9197fb8860d670e1302158ab3c7562d2c5fb6a6a7d1c319"
    ),
    timing!(
        "explicit-self-paid-fee.timing",
        "d0e033575f4c48440e5782bbf66c0bce8777ff2167ca18739395af3be542f954"
    ),
    canonical!(
        "explicit-several-destination-owners",
        "932c3f483f2f9f09d3eaf10be48dfda5d5569e284f90513c68607dad71f1bbff"
    ),
    timing!(
        "explicit-several-destination-owners.timing",
        "1736eb0c7992bf4210e1a1eee0891675a7e948250ae681d1e344001889ca94af"
    ),
    canonical!(
        "explicit-several-owners",
        "3a65d7d7eb9ba41f8083e7eeb291a3bf936e0c3c0cf22e1b1016c42d67c9c0fb"
    ),
    timing!(
        "explicit-several-owners.timing",
        "d70329f3f46783d4d3175cd2329b2d0e8a226142587d9e29f69d8730116824f9"
    ),
    canonical!(
        "explicit-several-to-several",
        "afb6f7073b5cd6d2273ba1a733f85aa8f46e2f4e9cbc303730b26a2b254f8b93"
    ),
    timing!(
        "explicit-several-to-several.timing",
        "19e9fcc629618ac571c422bb5e91e10a842fc23f9716bd38a171a652b2c2eb80"
    ),
    canonical!(
        "explicit-split",
        "1b2a7bd89491fd6a187ebcfc24e83e3b6c75af00d88d331a4af5e7d9ee9e99f0"
    ),
    timing!(
        "explicit-split.timing",
        "1736eb0c7992bf4210e1a1eee0891675a7e948250ae681d1e344001889ca94af"
    ),
    canonical!(
        "explicit-sponsorless",
        "7f03abd2163190d8dc71aaec27e1732b9eac30f1c35892f3f6b33b90c36c7994"
    ),
    timing!(
        "explicit-sponsorless.timing",
        "315f0a5e77a19d78c819cb8a3d8cffed205a1d3b356e4662b211f8b6518e30ea"
    ),
    canonical!(
        "explicit-witness-negatives",
        "3c12f68bb5b9fc439936410144c5cffb00fb26e68b3d5635af4153223b816b8f"
    ),
    timing!(
        "explicit-witness-negatives.timing",
        "eec6ed64a51b0612ed9ff7245e1eadfa4f89ef059d38720efd293ddbcc983e77"
    ),
    canonical!(
        "keypath-probe",
        "1ea6c67cb3040feceb1a83ef9ede91a63b49c885990f7fcabef15e2e29b50857"
    ),
    timing!(
        "keypath-probe.timing",
        "51e4a2edb0e05b03ad34abb60e9849f85d9deac04fe0c28b026c13db918d6fbf"
    ),
    canonical!(
        "multi-entry-crossing",
        "ed4e5a0481830b06eccbea9d746c65b85a326e7f005d299851e43df7ccd820c5"
    ),
    timing!(
        "multi-entry-crossing.timing",
        "c07d0530f18ee2a366618ea78b147c97e7bc823c8bd4bee9c7567e983b135dd2"
    ),
    canonical!(
        "multi-exit-crossing",
        "946c78d8695bf6ecb12865b46bcfac0fc4c9e89a133822f118a4a0ec59208c11"
    ),
    timing!(
        "multi-exit-crossing.timing",
        "ab9299fbdffcfb80174ac4240bf6a0d318076fad1e3f0fb5ae971b1267bd2267"
    ),
    canonical!(
        "multi-many-to-many",
        "9e74b8de9d111da66716c5420a9d0441cdd205e46829af395b5760fea7ebbb36"
    ),
    timing!(
        "multi-many-to-many.timing",
        "aa67cb1cac72aec41674409a8cfc50361c22792159be0198a3eb611bf5598d94"
    ),
    canonical!(
        "multi-one-to-one-with-fee",
        "fa82fed6e59ac8431181335a9fb15d5af84c2868dc1bfe2d0d0f38badb9f70be"
    ),
    timing!(
        "multi-one-to-one-with-fee.timing",
        "a45fb0ba36c2067e4d0e9c52b068bf8af0de8a23e66e4c75206b36e3118d2008"
    ),
    canonical!(
        "multi-private-merge",
        "09e4c2c15ffb42bc890e48aee9d0998ad7fb8348774a10a0cce38df316583ba4"
    ),
    timing!(
        "multi-private-merge.timing",
        "bcdabbfe21d42e2cda5f8a9762aa9fc65d25fd86de51d3fa4bb29d4918da3c47"
    ),
    canonical!(
        "multi-pure-split",
        "762bface563ae27ad15f3ff8cb67b10ed7a0664326888dc9579c56995e8084d3"
    ),
    timing!(
        "multi-pure-split.timing",
        "3b1b111b25734a3a897341d3dd1c468c9d2677a7998600f6e1b8ef47dbc565bd"
    ),
    canonical!(
        "multi-several-owners",
        "b9f7dc1db95bfbfb58c29bf5f5c541e0aae6ee91068df430cf98840f9ad69de0"
    ),
    timing!(
        "multi-several-owners.timing",
        "66f9f22d400d3d76da7e72c8a2652fa8cd708bf159a71b72bd0cce783cbfddd3"
    ),
    canonical!(
        "multi-split",
        "dfa3cbae7c6dead3b7cf1770a398ad6a96160248107bc3f219868490bcaa13c6"
    ),
    timing!(
        "multi-split.timing",
        "fb0a747a1ab15753d1cf8a3af735c768616eeb287dca84cee32e7f0266d1775e"
    ),
    canonical!(
        "multi-strict-one-to-one",
        "0e4920a2557dc6e7a80bbc55866778e5623ed6766de7aa15cc9ff954bafe3a30"
    ),
    timing!(
        "multi-strict-one-to-one.timing",
        "7057daa05726f130acac95d941d079f60c7b8038e0e2269d9f556871df32992c"
    ),
    canonical!(
        "owner-observation",
        "4199f49b77d0b275952f3b16d5251fd3d8816008bb7f1d22e7a104a4a3163bdf"
    ),
    timing!(
        "owner-observation.timing",
        "50c8a6b3c83e793e59c1b3fe791b0fb05378cccd5d57a96420bfa0ca794eb37d"
    ),
    canonical!(
        "owner-signing-negatives",
        "e219a32c5b20922fdd8b54717303574dcf1f1f3ce7555b368a79fcbb06d9298e"
    ),
    timing!(
        "owner-signing-negatives.timing",
        "0b594ef70d6d9f6e59cdb4fbd3623c4cb300e4c1c2c0f574215e9ee1a4eaff47"
    ),
    canonical!(
        "pairs-arc",
        "d14258ad485720c456928b1c4957417c5b1b71c7f36889e70acc14ab1733c5e8"
    ),
    timing!(
        "pairs-arc.timing",
        "d5eea63a68c8590fac99f7f0fc7a87654f52edddf9d8bf0acf6ec0e0b3269544"
    ),
    canonical!(
        "private-restart-control",
        "e382a69b546593652be0fdd1de81b5360bde259da3a7e0b320820e5a3ee175ae"
    ),
    timing!(
        "private-restart-control.timing",
        "9ede09b028446e1418c19719b2231ffffb35cc7aece391c6fdf3825286312726"
    ),
    canonical!(
        "private-restart-parity",
        "f9dbfc85b8a25db21af77ce45e9923c0f5a6764c21747e3bc247def733d52863"
    ),
    timing!(
        "private-restart-parity.timing",
        "37ccdf6d68713c4a8426d52cade6fffc742e4923ecba1b05c8c3457a71dce312"
    ),
    canonical!(
        "proof-bearing-observation",
        "9a864f812bf57afc293a7a0f91122e4144ec02c9a67b3dec8d24c48fb3bd080a"
    ),
    timing!(
        "proof-bearing-observation.timing",
        "7cca6d49f67acda93d2b38b48d0d87583f8862061347779926b3d413d9c7c48c"
    ),
    canonical!(
        "report",
        "2353019633b29feb56c74701005fb26acee71e2743608d831984aa7e0694e0b1"
    ),
    timing!(
        "report.timing",
        "240b7f57e03bc4b30a944e2f9aa2998ca18baba28e9cba636abff13bfc04b3db"
    ),
    canonical!(
        "sponsored-change-absent",
        "399c464537c56d283a0cf2f3535c8f4e1484307a310581999fc9a44c0546a68d"
    ),
    timing!(
        "sponsored-change-absent.timing",
        "1736eb0c7992bf4210e1a1eee0891675a7e948250ae681d1e344001889ca94af"
    ),
    canonical!(
        "sponsored-change-present",
        "0689d2c62f1315f94e6e1514db16f41ef9c28460d405c0c4cb5d4b84019764bb"
    ),
    timing!(
        "sponsored-change-present.timing",
        "1736eb0c7992bf4210e1a1eee0891675a7e948250ae681d1e344001889ca94af"
    ),
    canonical!(
        "sponsored-committed-value",
        "91c020c84bfd8968a7dd9f418adb07fbadfd02008d0bf3eca1183efbc2619200"
    ),
    timing!(
        "sponsored-committed-value.timing",
        "cd7919f5435a83358189be746d2ea559b4d5bb219aa1864e48435d75e6a5c0ce"
    ),
    canonical!(
        "sponsored-missing-authorization",
        "b7d8db01131e8709e7ac2cf980a80f3b2e5f9d02789441b03bc47a55dc25faf6"
    ),
    timing!(
        "sponsored-missing-authorization.timing",
        "1736eb0c7992bf4210e1a1eee0891675a7e948250ae681d1e344001889ca94af"
    ),
    canonical!(
        "sponsored-private-explicit-no-change",
        "252835989188555aef34474785a92dbc967a6ecc9067cdb3db507ce50d2f3e1f"
    ),
    timing!(
        "sponsored-private-explicit-no-change.timing",
        "14c971463351e089f8383ce872d85d110be51285f05c419b0f68a6bfbca76841"
    ),
    canonical!(
        "sponsored-private-with-change",
        "67d39bbf8f38dcabfefbacbf0fea42f2c21a114d1c16100dc9b59914e8c52fe5"
    ),
    timing!(
        "sponsored-private-with-change.timing",
        "fc03e4eaa86bd4c15a406a3b5f73eeb9c689e4e124584000eb1dc4e5737badf9"
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_file(suffix: &str) -> ArchiveFile {
        let name = format!("rerun-0.6.52-dev.{suffix}");
        ARCHIVE_FILES
            .iter()
            .find(|file| file.name == name.as_str())
            .copied()
            .expect("the test names a canonical archive file")
    }

    fn transcript_text(suffix: &str) -> (&'static str, &'static str) {
        let file = canonical_file(suffix);
        (
            file.name,
            std::str::from_utf8(file.bytes).expect("canonical transcripts are UTF-8"),
        )
    }

    #[test]
    fn the_manifest_covers_every_byte_faithfully() {
        validate_archive_manifest().expect("the checked-in archive matches its manifest");
        assert_eq!(ARCHIVE_FILES.len(), ARCHIVE_FILE_COUNT);
        assert_eq!(
            ARCHIVE_FILES
                .iter()
                .map(|file| file.bytes.len())
                .sum::<usize>(),
            ARCHIVE_TOTAL_BYTES
        );
        assert_eq!(
            ARCHIVE_FILES
                .iter()
                .filter(|file| file.role == ArchiveFileRole::CanonicalTranscript)
                .count(),
            CANONICAL_TRANSCRIPT_COUNT
        );
        assert_eq!(
            ARCHIVE_FILES
                .iter()
                .filter(|file| file.role == ArchiveFileRole::TimingSidecar)
                .count(),
            TIMING_SIDECAR_COUNT
        );
        assert_eq!(
            ARCHIVE_FILES
                .iter()
                .filter(|file| file.role == ArchiveFileRole::PredecessorSetup)
                .count(),
            1
        );
    }

    #[test]
    fn one_flipped_byte_fails_the_manifest() {
        let mut inputs = ARCHIVE_FILES
            .iter()
            .map(|file| ManifestInput {
                name: file.name,
                bytes: file.bytes,
                expected_sha256: file.expected_sha256,
            })
            .collect::<Vec<_>>();
        let mut flipped = inputs[1].bytes.to_vec();
        flipped[0] ^= 1;
        inputs[1].bytes = &flipped;

        assert!(matches!(
            validate_manifest_inputs(&inputs),
            Err(RerunDayCorpusRefusal::ManifestDigestMismatch { .. })
        ));
    }

    #[test]
    fn crlf_normalization_is_not_archive_fidelity() {
        let (name, text) = transcript_text("explicit-one-to-one");
        let mut crlf = Vec::with_capacity(text.len() + text.lines().count());
        for byte in text.bytes() {
            if byte == b'\n' {
                crlf.push(b'\r');
            }
            crlf.push(byte);
        }

        assert_eq!(
            parse_rerun_day_transcript(name, &crlf),
            Err(RerunDayCorpusRefusal::CarriageReturn {
                name: name.to_owned()
            })
        );
    }

    #[test]
    fn duplicate_missing_and_unknown_fields_are_refused() {
        let (name, text) = transcript_text("explicit-one-to-one");
        let issued_asset = text
            .lines()
            .find(|line| line.starts_with("issued_asset "))
            .expect("the canonical transcript carries the required field");
        let duplicate = text.replacen(issued_asset, &format!("{issued_asset}\n{issued_asset}"), 1);
        assert!(matches!(
            parse_rerun_day_transcript(name, duplicate.as_bytes()),
            Err(RerunDayCorpusRefusal::DuplicateField { .. })
        ));

        let missing = text.replacen(&format!("{issued_asset}\n"), "", 1);
        assert!(matches!(
            parse_rerun_day_transcript(name, missing.as_bytes()),
            Err(RerunDayCorpusRefusal::MissingField { .. })
        ));

        let unknown = format!("invented_field no\n{text}");
        assert!(matches!(
            parse_rerun_day_transcript(name, unknown.as_bytes()),
            Err(RerunDayCorpusRefusal::UnknownField { .. })
        ));
    }

    #[test]
    fn malformed_and_wrong_width_hex_are_refused_separately() {
        let (name, text) = transcript_text("explicit-one-to-one");
        let malformed = text.replacen("issued_asset d", "issued_asset z", 1);
        assert!(matches!(
            parse_rerun_day_transcript(name, malformed.as_bytes()),
            Err(RerunDayCorpusRefusal::MalformedHex { .. })
        ));

        let issued_asset = text
            .lines()
            .find(|line| line.starts_with("issued_asset "))
            .expect("the canonical transcript carries the required field");
        let wrong_width = text.replacen(issued_asset, "issued_asset 00", 1);
        assert!(matches!(
            parse_rerun_day_transcript(name, wrong_width.as_bytes()),
            Err(RerunDayCorpusRefusal::WrongByteLength { .. })
        ));
    }

    #[test]
    fn absent_accepted_transaction_ids_are_typed_and_exactly_spelled() {
        let (name, text) = transcript_text("sponsored-committed-value");
        assert_eq!(
            parse_transaction_id_field(name, "accepted_txid", "none"),
            Ok(TransactionIdField::Absent)
        );
        parse_rerun_day_transcript(name, text.as_bytes())
            .expect("the rejected positive attempt has no accepted transaction");

        let unsupported = text.replacen("accepted_txid none", "accepted_txid none-recorded", 1);
        assert_eq!(
            parse_rerun_day_transcript(name, unsupported.as_bytes()),
            Err(RerunDayCorpusRefusal::MalformedHex {
                name: name.to_owned(),
                field: "accepted_txid".to_owned(),
            })
        );
    }

    #[test]
    fn displayed_counts_and_run_boundaries_are_strict() {
        let (explicit_name, explicit) = transcript_text("explicit-one-to-one");
        let bad_count = explicit.replacen("input_count 1", "input_count 2", 1);
        assert!(matches!(
            parse_rerun_day_transcript(explicit_name, bad_count.as_bytes()),
            Err(RerunDayCorpusRefusal::CountMismatch { .. })
        ));

        let (multi_name, multi) = transcript_text("multi-many-to-many");
        let boundary = multi
            .lines()
            .next()
            .expect("the canonical transcript has a run boundary");
        let ambiguous = multi.replacen(boundary, &format!("{boundary}\n{boundary}"), 1);
        assert_eq!(
            parse_rerun_day_transcript(multi_name, ambiguous.as_bytes()),
            Err(RerunDayCorpusRefusal::AmbiguousRunBoundary {
                name: multi_name.to_owned()
            })
        );
    }

    #[test]
    fn displayed_request_bytes_must_equal_decoded_request_bytes() {
        let (name, keypath) = transcript_text("keypath-probe");
        let mismatch = keypath.replacen(
            "attempt submitted_bytes 281",
            "attempt submitted_bytes 280",
            1,
        );
        assert_eq!(
            parse_rerun_day_transcript(name, mismatch.as_bytes()),
            Err(RerunDayCorpusRefusal::DisplayedByteCountMismatch {
                name: name.to_owned(),
                request: "attempt".to_owned(),
                displayed: 280,
                decoded: 281,
            })
        );
    }

    #[test]
    fn the_present_archive_has_exactly_the_38_plus_1_refusal_census() {
        let corpus = parse_rerun_day_archive().expect("the canonical archive parses strictly");
        assert_eq!(corpus.ceremonies().len(), CANONICAL_TRANSCRIPT_COUNT);
        assert_eq!(
            corpus.incompleteness_census(),
            ArchiveIncompletenessCensus {
                missing_request_bytes: 38,
                missing_executor_provenance: 1,
            }
        );
        assert_eq!(
            corpus
                .ceremonies()
                .iter()
                .find(|ceremony| ceremony.name() == "rerun-0.6.52-dev.keypath-probe")
                .map(|ceremony| ceremony.incompleteness()),
            Some(ArchiveIncompleteness::MissingExecutorProvenance)
        );
        assert!(corpus.ceremonies().iter().all(|ceremony| {
            ceremony.name() == "rerun-0.6.52-dev.keypath-probe"
                || ceremony.incompleteness() == ArchiveIncompleteness::MissingRequestBytes
        }));
    }

    #[test]
    fn the_archive_exposes_no_standing_mutation_surface() {
        let accessor: fn(ArchivedCeremony) -> ArchiveIncompleteness =
            ArchivedCeremony::incompleteness;
        let corpus = parse_rerun_day_archive().expect("the canonical archive parses strictly");
        assert!(corpus.ceremonies().iter().all(|ceremony| matches!(
            accessor(*ceremony),
            ArchiveIncompleteness::MissingRequestBytes
                | ArchiveIncompleteness::MissingExecutorProvenance
        )));
    }

    #[test]
    fn suite_identity_is_artifact_internal_data() {
        assert_eq!(
            ARCHIVE_SUITE_IDENTITY,
            ArchiveSuiteIdentity {
                baseline_commit: "0.6.52-dev",
                execution: SuiteExecution::Serialized,
                passed: 40,
                total: 40,
                wall_milliseconds: 420_700,
            }
        );
    }
}
