//! Provenance-bearing identities from committed native run records.
//!
//! [`RecordedAcceptance`] says exactly one thing: the carried identity
//! is the identity a committed run-of-record source records for an
//! accepted transaction. It does not say the native ceremony has been
//! reproduced. Reproduction is a separate executable binding, such as
//! the private-restart gate in `guide13_live_native`.

use std::fmt;

use transaction::{TransactionIdentityParseError, Txid};

/// A transaction identity minted by the run-of-record module that owns it.
///
/// There is no public constructor. Public minting functions live beside
/// the committed constants they cite and take no caller-provided identity,
/// so syntactically valid text invented at a closeout or role boundary
/// cannot become this token.
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

    /// Parse one committed display identity and bind it to its source.
    ///
    /// # Errors
    ///
    /// [`TransactionIdentityParseError`] if the committed identity is
    /// not exactly 64 ASCII hexadecimal digits.
    pub(crate) fn from_run_of_record(
        accepted_identity: &'static str,
        citation: &'static str,
    ) -> Result<Self, TransactionIdentityParseError> {
        Txid::from_target_display(accepted_identity).map(|accepted_identity| Self {
            accepted_identity,
            citation,
        })
    }

    /// The target-computed transaction identity the record carries.
    #[must_use]
    pub const fn accepted_identity(self) -> Txid {
        self.accepted_identity
    }

    /// The committed run-of-record source that owns the identity.
    #[must_use]
    pub const fn citation(self) -> &'static str {
        self.citation
    }
}

// Role renderings historically debug-printed the identity string. Keep
// those bytes stable while the typed token retains its citation through
// the public accessor above.
impl fmt::Debug for RecordedAcceptance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\"")?;
        fmt::Display::fmt(&self.accepted_identity, formatter)?;
        formatter.write_str("\"")
    }
}

macro_rules! mint_recorded_acceptance {
    ($function:ident, $identity:ident) => {
        /// Mint the acceptance recorded by this function's run-of-record
        /// constant.
        ///
        /// # Errors
        ///
        /// [`transaction::TransactionIdentityParseError`] if the committed
        /// identity is not exactly 64 ASCII hexadecimal digits.
        pub fn $function() -> Result<
            $crate::recorded_acceptance::RecordedAcceptance,
            transaction::TransactionIdentityParseError,
        > {
            $crate::recorded_acceptance::RecordedAcceptance::from_run_of_record(
                $identity,
                concat!(module_path!(), "::", stringify!($identity)),
            )
        }
    };
}

pub(crate) use mint_recorded_acceptance;
