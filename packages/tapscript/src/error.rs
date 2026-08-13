//! The adapter's typed error root.
//!
//! # Only failures that can happen
//!
//! Two variants, because two failures are reachable. The set-level
//! assessment builds one map from one census, and the two ways that map
//! can disagree with the census it was built from — a capability
//! assessed twice, and a key set that is not the census — are checks
//! that run on every call and that a test drives to failure.
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
//!   the complete census, so no [`ElementsTarget`] exists that omits
//!   one. A variant for it would read as a check that is running when
//!   nothing can trigger it.
//! - an unsupported target contract revision is unconstructible for the
//!   same reason: the revision type has no unchecked constructor.
//! - a rejected target definition cannot arrive here, because the
//!   adapter accepts only a validated target.
//!
//! [`ElementsTarget`]: target_elements::ElementsTarget

use std::fmt;

use compiler::target::RequiredCapability;

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
                "assessment census mismatch: {} missing, {} unexpected",
                missing.len(),
                unexpected.len(),
            ),
        }
    }
}

impl std::error::Error for TapscriptError {}
