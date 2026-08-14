//! The adapter's typed error root.
//!
//! # Only failures that can happen
//!
//! Four variants, because four failures are reachable. The set-level
//! assessment builds one map per published census, and the two ways
//! either map can disagree with the census it was built from — a member
//! assessed twice, and a key set that is not the census — are checks
//! that run on every call and that a test drives to failure. The
//! capability census and the external-evidence-role census get their
//! own variants rather than a shared one carrying a kind: a reader of a
//! failure should not have to decode which census broke.
//!
//! Several failure classes a reader might expect are deliberately
//! absent, because no input reaches them:
//!
//! - a missing target primitive is an *assessment*, not an error. It is
//!   a fact about the target that the caller must see next to every
//!   other capability's disposition, not an exception that discards the
//!   rest of the census.
//! - a missing target evidence requirement is unconstructible. The
//!   target validator refuses a contract whose evidence registry is not
//!   the complete census, so no validated definition exists that omits
//!   one. A variant for it would read as a check that is running when
//!   nothing can trigger it.
//! - an unsupported target contract revision is unconstructible for the
//!   same reason: the revision type has no unchecked constructor.
//! - an unreviewed target definition cannot arrive here, because the
//!   public assessment entry points accept only the reviewed Elements
//!   contract, and the target package alone constructs that wrapper.

use std::fmt;

use compiler::target::{ExternalEvidenceRole, RequiredCapability};

/// A failure of the set-level capability assessment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TapscriptError {
    /// One compiler capability was assessed more than once.
    ///
    /// Two assessments of one capability are two answers to one
    /// question, and nothing downstream could choose between them.
    DuplicateCapabilityAssessment(RequiredCapability),

    /// The assessed capabilities are not exactly the required ones.
    ///
    /// Both directions matter. A missing assessment silently drops a
    /// requirement the analysis published, which is the weakening this
    /// adapter exists to prevent; an unexpected one answers a question
    /// no analysis asked, which would let an assessment set look
    /// broader than the analysis behind it.
    CapabilityAssessmentCensusMismatch {
        /// Required capabilities with no assessment, in census order.
        missing: Vec<RequiredCapability>,
        /// Assessed capabilities nothing required, in census order.
        unexpected: Vec<RequiredCapability>,
    },

    /// One compiler evidence role was assessed more than once.
    ///
    /// As for a capability: two assessments of one role are two answers
    /// to one question, and nothing downstream could choose between
    /// them.
    DuplicateEvidenceAssessment(ExternalEvidenceRole),

    /// The assessed evidence roles are not exactly the required ones.
    ///
    /// Both directions matter, for the reason the capability census
    /// gives. A missing role assessment is precisely the silent drop
    /// this census was added to prevent: the compiler's evidence
    /// boundary would reach the adapter and stop there.
    EvidenceAssessmentCensusMismatch {
        /// Required roles with no assessment, in census order.
        missing: Vec<ExternalEvidenceRole>,
        /// Assessed roles nothing required, in census order.
        unexpected: Vec<ExternalEvidenceRole>,
    },
}

impl fmt::Display for TapscriptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCapabilityAssessment(capability) => {
                write!(formatter, "capability {capability:?} was assessed twice")
            }
            Self::CapabilityAssessmentCensusMismatch {
                missing,
                unexpected,
            } => write!(
                formatter,
                "capability assessment census mismatch: {} missing, {} unexpected",
                missing.len(),
                unexpected.len(),
            ),
            Self::DuplicateEvidenceAssessment(role) => {
                write!(formatter, "evidence role {role:?} was assessed twice")
            }
            Self::EvidenceAssessmentCensusMismatch {
                missing,
                unexpected,
            } => write!(
                formatter,
                "evidence assessment census mismatch: {} missing, {} unexpected",
                missing.len(),
                unexpected.len(),
            ),
        }
    }
}

impl std::error::Error for TapscriptError {}
