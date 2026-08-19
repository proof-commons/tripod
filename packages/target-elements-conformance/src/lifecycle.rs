//! The Guide-11 §13 fresh-process lifecycle vocabulary.
//!
//! # What §13 asks, and what this path answers it with
//!
//! §13.1 defines "public" as *recoverable from canonical public chain
//! data by a party that did not participate in creation*, and rules out
//! *still present in the creator's process*. §13.5 closes by requiring
//! an actual process boundary rather than two functions sharing memory.
//!
//! §13 was written with a capsule in mind, because the representation it
//! anticipated was `PublicCommitted`: a committed amount cannot be read
//! without an opening, so something has to carry that opening and be
//! bound to the output. This wave runs the path that exists. On the
//! normalization path the public output is EXPLICIT — its amount and its
//! asset are on the output itself — so there is no capsule, no opening
//! to carry, and no binding to check. `G11-W8-03` recorded that as the
//! capsule's state already: not-applicable rather than deferred.
//!
//! So each §13.5 capsule failure is stated in the form it takes here,
//! and [`LifecycleRow`] carries the mapping as data rather than as
//! prose. Nothing is dropped for being inconvenient: the rows that have
//! no explicit-path analogue at all are absent because their subject is,
//! not because they were not run.
//!
//! # The two halves of future use
//!
//! §13.4's step 6 has Process B construct a future spend. That step
//! assumed a permissionless object — one any party may take. A
//! normalized output is OWNED, and the owner's key is precisely the
//! creator-local state the destruction boundary destroys, so a Process B
//! that could spend it would be evidence of a leak rather than of
//! publicness. The proof therefore splits, and both halves are checked:
//! the object is READABLE from public data alone, and the reading
//! process is a working constructor that confirms a spend of its OWN
//! funds. The permissionless case belongs to the compact-ASH work and is
//! out of scope here rather than quietly claimed.

use serde::{Deserialize, Serialize};

/// The schema the §13 public record declares.
pub const HANDOFF_SCHEMA: &str = "tripod-fresh-process-handoff-1";

/// Substrings that name owner-private material in a field name.
///
/// The allow-list in [`PublicHandoff`] is what actually holds — serde
/// refuses an unknown field outright. This list is the rule that would
/// still fire if a later wave widened that struct without asking what it
/// was widening it for, and it is checked by
/// [`names_owner_private_material`].
pub const FORBIDDEN_FIELD_SUBSTRINGS: [&str; 11] = [
    "key",
    "blind",
    "seed",
    "secret",
    "priv",
    "mnemonic",
    "descriptor",
    "wallet",
    "xprv",
    "nonce",
    "datadir",
];

/// Whether a field name names owner-private material.
#[must_use]
pub fn names_owner_private_material(field: &str) -> bool {
    let lowered = field.to_ascii_lowercase();
    FORBIDDEN_FIELD_SUBSTRINGS
        .iter()
        .any(|banned| lowered.contains(banned))
}

/// The public record that crosses the process boundary, and all of it.
///
/// This type is the entire channel between the process that created the
/// object and the process that had no part in creating it. Every field
/// is canonical public chain data or the public encoding of an address:
/// there is no key, no blinding factor, no descriptor, no wallet, no
/// seed, and no data directory here, and `deny_unknown_fields` is what
/// stops one arriving unnoticed. A record read leniently would turn a
/// fresh-process proof into a proof conducted over a private side
/// channel, which is the one failure this whole section exists to
/// exclude.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicHandoff {
    /// The record's own schema, checked against [`HANDOFF_SCHEMA`].
    pub schema: String,
    /// The chain the record is about.
    pub chain_name: String,
    /// The declared development network identity of that chain.
    pub network_id: String,
    /// The genesis the record binds itself to.
    pub genesis_id: String,
    /// The transaction carrying the object.
    pub txid: String,
    /// Which output of it the object is.
    pub output_index: u32,
    /// The block the transaction is in.
    ///
    /// A locator rather than a transaction index: an index is a node
    /// configuration, and a block hash is chain data.
    pub block_hash: String,
    /// That block's height.
    pub block_height: u64,
    /// The transaction's bytes, as the record states them.
    ///
    /// Never used in place of the chain's copy. The reading process
    /// fetches the chain's bytes and compares, which is what catches a
    /// record built from a different transaction's data — the copy
    /// parses perfectly and disagrees byte for byte.
    pub raw_transaction: String,
    /// The amount the record claims the object carries.
    pub claimed_explicit_amount: u64,
    /// The asset it claims, displayed.
    pub claimed_explicit_asset: String,
    /// The owner, as a public address and nothing more.
    ///
    /// The reading process rebuilds the output script from this alone
    /// and byte-compares it against the chain's, which is what a future
    /// constructor citing this output would have to do.
    pub claimed_owner_address: String,
}

impl PublicHandoff {
    /// Whether the record declares the schema this type reads.
    #[must_use]
    pub fn schema_is_current(&self) -> bool {
        self.schema == HANDOFF_SCHEMA
    }

    /// Every field name this type admits.
    ///
    /// Stated so that the ban can be checked against the allow-list
    /// rather than against a reader's memory of it.
    #[must_use]
    pub const fn field_names() -> [&'static str; 12] {
        [
            "schema",
            "chain_name",
            "network_id",
            "genesis_id",
            "txid",
            "output_index",
            "block_hash",
            "block_height",
            "raw_transaction",
            "claimed_explicit_amount",
            "claimed_explicit_asset",
            "claimed_owner_address",
        ]
    }
}

/// One row of the §13.5 matrix, in its explicit-path form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LifecycleRow {
    /// The record as published. The only row expected to verify.
    Accepted,
    /// §13.5's missing capsule: the chain does not carry the transaction.
    MissingTransaction,
    /// §13.5's wrong output, where the index is not one the transaction has.
    WrongOutputIndexAbsent,
    /// §13.5's wrong output, where the index is the private change.
    ///
    /// The nastier half: the index exists, the transaction is right, and
    /// the output publishes nothing because it is blinded.
    WrongOutputIndexPrivateChange,
    /// §13.5's copied capsule: another real transaction's bytes.
    CopiedEvidence,
    /// §13.5's stale capsule: a later transaction consumed the object.
    StaleEvidence,
    /// §13.5's wrong chain context: a genesis this chain does not have.
    WrongChainContext,
    /// §13.5's owner-private dependency, by field name.
    OwnerPrivateField,
    /// A field the schema does not admit at all.
    UnknownField,
}

/// What the reading process did with one record.
///
/// The refusals are distinct on purpose. "The transaction is not there"
/// and "the transaction is there and its bytes are not the ones you
/// published" are different facts about a public record, and a matrix
/// that collapsed them could not tell a pruned chain from a forged
/// citation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LifecycleOutcome {
    /// Process A published a record.
    Constructed,
    /// Process B located, parsed, and checked the object, and built its own spend.
    Verified,
    /// The chain does not carry the transaction the record names.
    RefusedEvidenceAbsent,
    /// The chain's bytes are not the record's bytes.
    RefusedCopiedEvidence,
    /// The transaction has no such output.
    RefusedOutputAbsent,
    /// The output exists and publishes no amount.
    RefusedOutputNotExplicit,
    /// The output existed and a later transaction consumed it.
    RefusedOutputSpent,
    /// The record declares a chain this is not.
    RefusedWrongChainContext,
    /// The record was refused before it was read as a record at all.
    ExecutorInfrastructureFailure,
    /// The step could not be built, so nothing judged anything.
    FixtureConstructionFailure,
}

impl LifecycleOutcome {
    /// Whether this outcome establishes a fact about the lifecycle.
    ///
    /// A step that could not be built establishes nothing: no process
    /// ever asked the question. §8.3's vocabulary exists to keep that
    /// distinct from an answer, and `G11-W7-03` is the precedent for
    /// recording such a row rather than dropping it.
    #[must_use]
    pub const fn establishes_fact(self) -> bool {
        !matches!(self, Self::FixtureConstructionFailure)
    }
}

/// One row's expectation, written before the run and owned by source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleExpectation {
    /// The row.
    pub row: LifecycleRow,
    /// What it is expected to do.
    pub expected: LifecycleOutcome,
    /// Which §13.5 failure it is the explicit-path form of, and why.
    pub reasoning: String,
}

/// The canonical §13.5 matrix, in its explicit-path form.
///
/// Owned here and nowhere else, for the reason the §10.4 matrix is: a
/// runner that restated an expectation would be supplying the answer it
/// is checked against, and the run could then agree with itself.
#[must_use]
pub fn canonical_lifecycle_matrix() -> Vec<LifecycleExpectation> {
    let rows: [(LifecycleRow, LifecycleOutcome, &str); 9] = [
        (
            LifecycleRow::Accepted,
            LifecycleOutcome::Verified,
            "The published record. An unrelated process locates the \
             transaction by its block locator, parses the output with its \
             own deserializer, reads the explicit amount and asset off the \
             output, rebuilds the output script from the public address \
             alone, and confirms a spend of its own funds. This is the row \
             the section exists to obtain, and every other row is a way of \
             showing that it did not pass by accident.",
        ),
        (
            LifecycleRow::MissingTransaction,
            LifecycleOutcome::RefusedEvidenceAbsent,
            "The explicit-path form of a missing capsule. The identifier is \
             still 64 well-formed hex digits and still names nothing, so the \
             refusal is the chain's answer rather than a shape check.",
        ),
        (
            LifecycleRow::WrongOutputIndexAbsent,
            LifecycleOutcome::RefusedOutputAbsent,
            "§13.5's wrong output in its simplest form: the transaction is \
             the right one and has no such output.",
        ),
        (
            LifecycleRow::WrongOutputIndexPrivateChange,
            LifecycleOutcome::RefusedOutputNotExplicit,
            "§13.5's wrong output in the form that matters on this path. The \
             index exists and the output is the private change, which \
             publishes no amount at all — so a reader that took any output \
             of the right transaction would recover nothing and must say so \
             rather than report an absence as a zero.",
        ),
        (
            LifecycleRow::CopiedEvidence,
            LifecycleOutcome::RefusedCopiedEvidence,
            "The explicit-path form of a copied capsule: another REAL \
             normalization's bytes, published for this outpoint. Both \
             transactions exist, both parse, and only a byte comparison \
             against the chain's own copy separates them — which is why the \
             record's bytes are checked against the chain rather than used \
             in its place.",
        ),
        (
            LifecycleRow::StaleEvidence,
            LifecycleOutcome::RefusedOutputSpent,
            "The explicit-path form of a stale capsule. The transaction is \
             still in the chain and every byte still parses; what is gone is \
             the object, because a later transaction consumed it. The record \
             is true about the past and false about what can be used now.",
        ),
        (
            LifecycleRow::WrongChainContext,
            LifecycleOutcome::RefusedWrongChainContext,
            "Refused before any evidence is fetched, and that ordering is \
             the row. A record checked afterwards would have had its txid \
             resolved and its amount parsed against the wrong chain, and \
             every one of those readings would have been about something \
             else.",
        ),
        (
            LifecycleRow::OwnerPrivateField,
            LifecycleOutcome::ExecutorInfrastructureFailure,
            "§13.5's owner-private dependency, refused by the NAME of the \
             field rather than by what it holds. The value carried in this \
             row is deliberately not a real one: a matrix that had to supply \
             a genuine blinding factor to prove it refuses blinding factors \
             would be carrying owner-private material to make a point about \
             not carrying it.",
        ),
        (
            LifecycleRow::UnknownField,
            LifecycleOutcome::ExecutorInfrastructureFailure,
            "The allow-list, checked separately from the name ban. The field \
             this row adds is innocuous — it names no secret and holds \
             nothing — and it is still refused, because what makes the \
             boundary a boundary is that the record carries exactly the \
             fields the schema states and no others.",
        ),
    ];
    rows.into_iter()
        .map(|(row, expected, reasoning)| LifecycleExpectation {
            row,
            expected,
            reasoning: reasoning.to_owned(),
        })
        .collect()
}
