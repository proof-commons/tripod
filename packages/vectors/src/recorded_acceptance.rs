//! Provenance-bearing identities from the validated current corpus.
//!
//! [`RecordedAcceptance`] says exactly one thing: the carried identity
//! is the target-computed identity the native-v2/revision-7 corpus records
//! for an accepted transaction.

use std::fmt;

use transaction::Txid;

/// A transaction identity minted from the validated current corpus.
///
/// There is no public constructor, so syntactically valid text invented at a
/// role boundary cannot become this token.
///
/// ```compile_fail
/// use transaction::Txid;
/// use vectors::RecordedAcceptance;
///
/// let invented = Txid::from_target_display(&"00".repeat(32)).unwrap();
/// let _ = RecordedAcceptance {
///     accepted_identity: invented,
///     citation: "caller-authored",
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordedAcceptance {
    accepted_identity: Txid,
    citation: &'static str,
}

impl RecordedAcceptance {
    /// Bind an identity supplied by the validated native-v2/revision-7 corpus.
    #[cfg(test)]
    pub(crate) const fn from_validated_corpus(accepted_identity: Txid) -> Self {
        Self {
            accepted_identity,
            citation: "vectors::live_corpus_native_v2_r7",
        }
    }

    /// The target-computed transaction identity the record carries.
    #[must_use]
    pub const fn accepted_identity(self) -> Txid {
        self.accepted_identity
    }

    /// The validated corpus source that owns the identity.
    #[must_use]
    pub const fn citation(self) -> &'static str {
        self.citation
    }
}

// Role renderings debug-print the identity string. Keep those bytes stable
// while the typed token retains its citation through the public accessor.
impl fmt::Debug for RecordedAcceptance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\"")?;
        fmt::Display::fmt(&self.accepted_identity, formatter)?;
        formatter.write_str("\"")
    }
}
