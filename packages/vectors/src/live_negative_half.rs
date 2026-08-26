//! The negative half's work register: every §15 row a run has still not
//! answered, enumerated rather than left as a gap in a count.
//!
//! # Why a register and not a number
//!
//! [`crate::live_evidence`] classifies every row of the §15 matrix and
//! counts how many stand at [`LiveRowStanding::NativeRunRequired`]. A
//! count is enough to know the half is unfinished and not enough to work
//! on: it says HOW MANY rows are waiting and never says WHICH, so a
//! wave reading it has to rediscover the list by re-deriving the
//! classifier's fall-through every time, and two waves can disagree
//! about the list while agreeing about the number. The count is left to
//! the census deliberately — a number repeated in prose here would be a
//! second place for it to drift.
//!
//! This register is that list, written down once and held against the
//! classifier by [`every_outstanding_row_is_registered`]. The test is
//! set equality in BOTH directions, which is what makes the register
//! load-bearing rather than documentation: a row that moves to an
//! observed standing and is left here fails the test, and a row that
//! stays required without being listed here fails it too. Neither can
//! happen silently.
//!
//! # What this register is not
//!
//! It is not evidence, and it moves no row. A row leaves this list by
//! being ANSWERED at its own site — an observed refusal recorded in
//! [`crate::live_evidence`] against an accepted control — and never by
//! being deleted from here. The register follows the classifier; the
//! classifier never follows the register.

use crate::error::VectorError;
use crate::live_evidence::{LiveRowStanding, derive_live_evidence_plan};

/// Every §15 row still standing at
/// [`LiveRowStanding::NativeRunRequired`], in matrix order.
///
/// Derived from the classifier's own fall-through rather than kept by
/// hand: a row reaches [`LiveRowStanding::NativeRunRequired`] only after
/// every earlier branch has declined it — it is not vocabulary-closed,
/// not report-layer, not first-party, not blocked on a named component,
/// and no acceptance, refusal or determinism observation of its own
/// shape exists. What is written here is the result of that subtraction,
/// and the test below is what keeps the writing and the subtraction the
/// same list.
pub const STILL_REQUIRED: &[&str] = &[
    // §15.2 — the one positive row no run has answered. It is here for
    // the same reason the negatives are: a row waiting on a run is
    // waiting on a run whatever its polarity, and a register that held
    // only negatives would report the positive half complete.
    "projection-equality-with-paired-explicit",
    // §15.4 — the class, asset and constructor faults still waiting.
    "ash-input-or-output",
    "vault-control-entitlement-or-bare-u-output",
    "wrong-owner-metadata",
    "malformed-live-metadata",
    "stale-constructor",
    "wrong-explicit-asset",
    "confidential-asset-commitment",
    "unclassified-u",
    "sponsor-or-fee-role-carrying-u",
    "foreign-asset-under-receipt-shaped-program",
    "key-path-escape",
    "malformed-control-path",
    // §15.5 — the value and partition faults still waiting. Two rows
    // of this section have LEFT this register: `malformed-rangeproof`
    // and `wrong-private-blinding-balance` are answered by the
    // conservation ceremony's own run, each on its own mutant, and they
    // are recorded there rather than here. `private-ct-imbalance` stays
    // because no mutant of it was built.
    "output-total-one-below-input",
    "output-total-one-above-input",
    "amount-outside-semantic-domain",
    "private-ct-imbalance",
    "malformed-surjection-proof",
    "copied-commitment",
    "private-output-omitted",
    "hidden-private-u-output",
    "omitted-source",
    "duplicated-source",
    "duplicated-destination",
    "output-claimed-through-two-flows",
    "issuance",
    "destruction",
    "value-routed-into-ash-or-time-locked-receipt",
    "second-offsetting-u-flow",
    // §15.6 — the sponsor faults still waiting. The sponsor table's
    // other five are answered: two are report-layer, two are pre-target
    // first-party, and `missing-sponsor-authorization` is one of the
    // observed refusals this register's successors are modelled on.
    "sponsor-protocol-overlap",
    "two-sponsor-envelopes",
    "foreign-sponsor-asset",
    "sponsor-change-in-protocol-range",
    "sponsor-member-unclassified",
    "balanced-theft",
    "zero-valued-ordinary-sponsor-member",
    "confidential-sponsor-values",
    // §15.7 — the root, event, ABI and linker faults still waiting.
    // One unanswered row of this section is deliberately NOT here:
    // `raw-transaction-bypassing-safe-construction` stands at
    // `InfrastructureBlocked`, which is a different state from waiting
    // on a run, and listing it here would report a blocked row as a
    // runnable one.
    "any-root-input-or-output",
    "burn-record-or-specialized-event",
    "omitted-transition-certificate",
    "wrong-coordinator",
    "two-coordinators",
    "no-coordinator",
    "member-coordinator-leaf-exchange",
    "receipt-sponsor-range-exchange",
    "witness-reorder",
    "control-block-from-another-program",
    "target-bytes-changed-after-abi-validation",
];

/// The rows the classifier itself leaves waiting on a run, recomputed.
///
/// The register's counterpart, and the reason the register can be
/// trusted: this reads the standing of every row from
/// [`derive_live_evidence_plan`] rather than from any list, so the two
/// can be compared instead of one being asserted.
///
/// # Errors
///
/// Whatever [`derive_live_evidence_plan`] returns when a source artifact
/// does not build.
pub fn outstanding_rows() -> Result<Vec<&'static str>, VectorError> {
    let plan = derive_live_evidence_plan()?;
    Ok(plan
        .rows()
        .iter()
        .filter(|row| matches!(row.standing(), LiveRowStanding::NativeRunRequired(_)))
        .map(|row| row.row().name())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{STILL_REQUIRED, outstanding_rows};
    use std::collections::BTreeSet;

    /// The register and the classifier name the same rows.
    ///
    /// Set equality in both directions, reported as the two differences
    /// rather than as one boolean, so a failure says which rows drifted
    /// and in which direction instead of only that something did.
    #[test]
    fn every_outstanding_row_is_registered() {
        let computed: BTreeSet<&str> = outstanding_rows()
            .expect("the evidence plan derives")
            .into_iter()
            .collect();
        let registered: BTreeSet<&str> = STILL_REQUIRED.iter().copied().collect();

        let unregistered: Vec<&&str> = computed.difference(&registered).collect();
        assert!(
            unregistered.is_empty(),
            "rows still waiting on a run and absent from the register: {unregistered:?}",
        );

        let stale: Vec<&&str> = registered.difference(&computed).collect();
        assert!(
            stale.is_empty(),
            "rows registered as waiting that the classifier has answered: {stale:?}",
        );
    }

    /// The register lists each row once.
    ///
    /// A duplicated entry would still satisfy set equality above while
    /// making the register's own length disagree with what it holds.
    #[test]
    fn the_register_holds_no_duplicate() {
        let unique: BTreeSet<&str> = STILL_REQUIRED.iter().copied().collect();
        assert_eq!(
            unique.len(),
            STILL_REQUIRED.len(),
            "the register lists a row more than once",
        );
    }
}
