//! Candidate ceremony vocabulary, carried beside the reviewed target
//! facts rather than transcribed from them.
//!
//! # Why a first-party word lives in a target-facts crate
//!
//! Every other module here states something a reviewed reading of the
//! target established. This one does not, and the widening is explicit
//! rather than silent: the crate contract is amended in the same change
//! that adds the module, in the README and in the package contract, so
//! that a reader who trusts "target facts only" is never quietly wrong.
//!
//! The reason is a dependency fact and not a convenience. The
//! reproducibility contract is named by the wire record in
//! `target-elements-conformance`, by the materializer profiles in
//! `transaction`, and by the evidence record; those two packages share
//! no library edge in either direction, and the only package both
//! already depend on is this one. One enum in each package, held equal
//! by a census test, would be two authored spellings of one closed
//! vocabulary — the defect the typed-source rule exists to prevent — so
//! the vocabulary is stated once, here, and both sides name the same
//! value.
//!
//! Nothing in this module is a target fact, an architecture operation, a
//! phase, a release identity, or a digest. It is candidate vocabulary
//! and no version number is minted for it.
//!
//! # What this crate still does not do
//!
//! It does not serialize. The enum below derives no serialization and
//! this crate gains no dependency to provide one: a package that speaks
//! the enum on a wire owns its own encoding of it, and the compiler
//! holds that encoding total over these variants.

use core::fmt;

/// Which reproducibility contract one ceremony runs under.
///
/// # Neither contract is the revision of the other
///
/// The selection is per ceremony and typed. One evidence schema carries
/// the selected contract as a field rather than two parallel schemas
/// existing; every validated report names the contract its run was
/// under; a report under one contract can never claim the other's
/// guarantees; and no run mixes contracts silently.
///
/// The enum is closed at two members and carries no catch-all. A third
/// contract would be a third variant every peer must advertise, not a
/// value an existing member could be stretched to cover.
///
/// It is deliberately not `non_exhaustive`: a consumer's match over the
/// two contracts is meant to stop compiling when a third is added,
/// because the thing a new contract changes is exactly what every such
/// match decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReproducibilityContract {
    /// Equal inputs produce equal bytes.
    ///
    /// The reference contract. It is what deterministic central public
    /// fixtures exist to serve, and it is the contract that satisfies
    /// the deterministic-public-fixture-openings requirement. A run that
    /// selected it and did not achieve it has failed; it has not
    /// changed contracts.
    ByteIdentity,
    /// Openings are produced by the run and retained, and comparison is
    /// semantic rather than byte-for-byte.
    ///
    /// Everything the reference contract fixes is preserved except the
    /// opening source and cross-run byte equality: the fixture
    /// semantics, the handle, the digest check, the output order, every
    /// construction check, and the exact target form are identical.
    RecordedRandomness,
}

impl ReproducibilityContract {
    /// Every contract, in the order the vocabulary states them.
    ///
    /// The array is what a census walks. A variant added without being
    /// placed here is caught by the exhaustive match in
    /// [`Self::code`] rather than silently omitted from every census
    /// that reads this constant.
    pub const ALL: [Self; 2] = [Self::ByteIdentity, Self::RecordedRandomness];

    /// The reference contract.
    ///
    /// Named once so that a lane wanting "the contract the proven rerun
    /// demonstrates" states it by this constant rather than by picking a
    /// variant and hoping it is the same one.
    pub const REFERENCE: Self = Self::ByteIdentity;

    /// The contract's stable code.
    ///
    /// One spelling, so that a package encoding this on a wire and a
    /// package writing it into a transcript preimage cannot disagree
    /// about the word. The match is exhaustive on purpose.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ByteIdentity => "byte_identity",
            Self::RecordedRandomness => "recorded_randomness",
        }
    }

    /// The contract one code names, where it names one.
    ///
    /// `None` rather than a default: a code this vocabulary does not
    /// hold is unknown, and answering it with the reference contract
    /// would move a run onto guarantees nobody selected.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.code() == code)
    }

    /// Whether wallet- or adapter-held randomness is an admissible
    /// opening source under this contract.
    ///
    /// A statement about which comparison a run may claim, and never
    /// about who holds the openings for a given ceremony. The custody
    /// model is settled elsewhere and this predicate does not reopen it.
    #[must_use]
    pub const fn admits_run_produced_openings(self) -> bool {
        match self {
            Self::ByteIdentity => false,
            Self::RecordedRandomness => true,
        }
    }

    /// Whether this contract may claim byte equality across runs.
    #[must_use]
    pub const fn claims_cross_run_byte_equality(self) -> bool {
        match self {
            Self::ByteIdentity => true,
            Self::RecordedRandomness => false,
        }
    }
}

impl fmt::Display for ReproducibilityContract {
    /// The code, and nothing beside it.
    ///
    /// A `Display` that rendered a contract differently from
    /// [`Self::code`] would be a second authored spelling of one word,
    /// harmless while a reader is human and not harmless at all the
    /// moment anything compares the two.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}
