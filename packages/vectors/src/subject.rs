//! Canonical and experimental evidence subjects.
//!
//! Guide-12 §16.4 ends with one sentence that decides this module's
//! shape: *ad hoc vectors produce experimental reports only.* A rule
//! phrased that way can be obeyed or forgotten, so it is spent here as a
//! type instead. A claim reaches evidence standing by passing through
//! the checked plan constructor, and there is no other route: the
//! canonical wrapper's field is private to this crate, and no
//! conversion, `From`, or promotion method takes an experimental
//! subject to a canonical one.
//!
//! The asymmetry is deliberate. Demoting a canonical subject to an
//! experimental one is always sound — it weakens a claim — and is
//! offered. The reverse would let a caller launder an unadmitted vector
//! into the census the exit gate counts, which is exactly the failure
//! §1.11 forbids when it refuses a truncated vector matrix published as
//! complete.

use core::fmt::{self, Display};

/// The standing a report's claim carries.
///
/// This is the observable half of the separation: a report prints its
/// standing, and a reader never has to infer from surrounding prose
/// whether a number belongs in the exit-gate census.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SubjectStanding {
    /// Admitted through the checked evidence plan; countable evidence.
    Canonical,
    /// Constructed ad hoc; informative only, never countable evidence.
    Experimental,
}

impl SubjectStanding {
    /// Every standing, for census recomputation.
    pub const ALL: &'static [Self] = &[Self::Canonical, Self::Experimental];

    /// Whether a claim of this standing counts toward the exit gate.
    #[must_use]
    pub const fn counts_as_evidence(self) -> bool {
        matches!(self, Self::Canonical)
    }
}

impl Display for SubjectStanding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Canonical => "canonical",
            Self::Experimental => "experimental",
        })
    }
}

/// A subject admitted through the checked evidence plan.
///
/// The constructor is crate-private on purpose. A downstream caller can
/// read a canonical subject and can weaken it, but cannot mint one,
/// because minting is what the plan's validation is for.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalSubject<T> {
    subject: T,
}

impl<T> CanonicalSubject<T> {
    /// Admit a subject. Reachable only from inside this crate, and only
    /// from a path that has already validated the plan it belongs to.
    //
    // The allow is scoped to this one function and is temporary: the
    // evidence plan is the caller, and it lands in the following commit
    // of this same wave. Widening it to the module would hide the next
    // unused item too, which is the opposite of what the census
    // discipline is for.
    #[allow(dead_code)]
    pub(crate) const fn admit(subject: T) -> Self {
        Self { subject }
    }

    /// The admitted subject.
    #[must_use]
    pub const fn subject(&self) -> &T {
        &self.subject
    }

    /// The standing this wrapper confers.
    #[must_use]
    pub const fn standing(&self) -> SubjectStanding {
        SubjectStanding::Canonical
    }

    /// Weaken to an experimental subject.
    ///
    /// Offered because weakening a claim is always sound. The reverse
    /// direction is deliberately absent.
    #[must_use]
    pub fn into_experimental(self) -> ExperimentalSubject<T> {
        ExperimentalSubject::observe(self.subject)
    }

    /// Consume the wrapper for the subject.
    #[must_use]
    pub fn into_subject(self) -> T {
        self.subject
    }
}

/// A subject constructed outside the checked evidence plan.
///
/// Anyone may build one. Nothing built this way is ever counted by a
/// census, and no method here returns a [`CanonicalSubject`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExperimentalSubject<T> {
    subject: T,
}

impl<T> ExperimentalSubject<T> {
    /// Record an ad hoc subject.
    pub const fn observe(subject: T) -> Self {
        Self { subject }
    }

    /// The observed subject.
    #[must_use]
    pub const fn subject(&self) -> &T {
        &self.subject
    }

    /// The standing this wrapper confers.
    #[must_use]
    pub const fn standing(&self) -> SubjectStanding {
        SubjectStanding::Experimental
    }

    /// Consume the wrapper for the subject.
    #[must_use]
    pub fn into_subject(self) -> T {
        self.subject
    }
}

#[cfg(test)]
mod tests {
    use super::{CanonicalSubject, ExperimentalSubject, SubjectStanding};

    #[test]
    fn standings_census_is_exact() {
        assert_eq!(SubjectStanding::ALL.len(), 2);
        assert_eq!(
            SubjectStanding::ALL
                .iter()
                .filter(|standing| standing.counts_as_evidence())
                .count(),
            1,
            "exactly one standing may be counted by the exit gate"
        );
    }

    #[test]
    fn an_admitted_subject_weakens_but_an_observed_one_does_not_strengthen() {
        let admitted = CanonicalSubject::admit(7_u8);
        assert_eq!(admitted.standing(), SubjectStanding::Canonical);

        let weakened = admitted.into_experimental();
        assert_eq!(weakened.standing(), SubjectStanding::Experimental);
        assert_eq!(*weakened.subject(), 7);

        // The reverse direction is absent by construction: there is no
        // method on `ExperimentalSubject` returning a canonical one, and
        // `CanonicalSubject::admit` is crate-private. This assertion
        // records the intent that the compiler enforces.
        let observed = ExperimentalSubject::observe(7_u8);
        assert!(!observed.standing().counts_as_evidence());
    }

    #[test]
    fn standings_print_stably() {
        assert_eq!(SubjectStanding::Canonical.to_string(), "canonical");
        assert_eq!(SubjectStanding::Experimental.to_string(), "experimental");
    }
}
