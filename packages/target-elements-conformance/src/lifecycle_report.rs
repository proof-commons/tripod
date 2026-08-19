//! The typed `FreshProcessLifecycle` report of Guide 11 §14.
//!
//! # The role is its own type, and that is §14.1's rule rather than a preference
//!
//! §14.1 lists five report roles and says a report of one role cannot
//! satisfy another. [`ConservationReportRole`] is shared between the
//! §8.4 and §10.4 reports, which is a decision those waves recorded and
//! this one does not reopen; what it does not do is stretch to a third
//! question. A lifecycle report answers "is this recoverable by a party
//! that did not participate in creation", and a safety report answers
//! "does this path survive mutation". Neither answer is evidence for the
//! other, so a value of one type must not be constructible where the
//! other is wanted.
//!
//! The role has exactly one variant, for the reason
//! [`crate::normalization_report`] gives: this wave establishes what the
//! target does, not that the candidate is selected. §24's matrix decides
//! that, and a canonical role belongs to whichever wave earns it.
//!
//! # The boundary is a finding, not an assumption
//!
//! Everything else in this report is worthless if Process A was still
//! running, or if its wallet was still on disk, while Process B read the
//! chain. So [`DestructionRecord`] is part of the report rather than
//! part of the runner's prose, [`FreshProcessLifecycleReport::boundary_holds`]
//! is a check a reader can apply, and the record states what was
//! destroyed by name and size and never by content — copying a wallet
//! database into an evidence record would defeat the boundary in the act
//! of documenting it.
//!
//! # The expectations are rebuilt from source
//!
//! Every row's expectation comes from [`canonical_lifecycle_matrix`],
//! not from the run record. A run that supplied the answer it is checked
//! against agrees with itself, and on this lane in particular the
//! temptation is acute: the runner already knows which record it
//! corrupted.

use serde::{Deserialize, Serialize};

use crate::conservation_report::RowVerdict;
use crate::lifecycle::{
    LifecycleOutcome, LifecycleRow, PublicHandoff, canonical_lifecycle_matrix,
    names_owner_private_material,
};
use crate::normalization::AuthorizationProfile;

/// What a `FreshProcessLifecycle` report may claim.
///
/// One variant, and a separate type from every other role's. See the
/// module documentation: §14.1's rule is that a report of one role
/// cannot satisfy another, and sharing a type is how that stops being
/// true.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LifecycleReportRole {
    /// Evidence about the target, and no claim that the path is selected.
    Experimental,
}

/// One file that existed before the boundary and does not after.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestroyedFile {
    /// Its path, relative to the wallet directory.
    pub path: String,
    /// Its size. Never its content.
    pub bytes: u64,
}

/// What the boundary destroyed, and what it deliberately did not.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestructionRecord {
    /// The scoping, stated rather than implied.
    ///
    /// The node is not creator state. Chain data IS the canonical public
    /// record, so a boundary that destroyed it would destroy the
    /// evidence the section exists to find; a boundary that kept the
    /// wallet would not be a boundary at all. Getting this backwards
    /// yields a test that passes for the wrong reason in one direction
    /// and fails for the wrong reason in the other.
    pub scope: String,
    /// The creator's wallet directory, relative to the chain directory.
    pub wallet_directory: String,
    /// What was in it.
    pub destroyed_files: Vec<DestroyedFile>,
    /// How much, in total.
    pub destroyed_bytes: u64,
    /// Whether the directory survived. It must not have.
    pub wallet_directory_present_after: bool,
    /// The creating process's exit status, where it had one.
    pub process_a_exit_status: Option<i32>,
    /// Whether the creating process was still running. It must not have been.
    pub process_a_running_after: bool,
}

impl DestructionRecord {
    /// Whether the boundary is real.
    ///
    /// Both halves are required and neither substitutes for the other: a
    /// deleted wallet with the creator still running leaves the keys in
    /// that process's memory, and an exited creator with its wallet on
    /// disk leaves them for anything that opens the directory.
    #[must_use]
    pub const fn holds(&self) -> bool {
        !self.wallet_directory_present_after
            && !self.process_a_running_after
            && !self.destroyed_files.is_empty()
    }
}

/// One thing the reading process looked for, and what it found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckOutcome {
    /// What was checked.
    pub check: String,
    /// What the public record claimed, rendered.
    pub expected: String,
    /// What the chain carried, rendered.
    pub observed: String,
    /// Whether they met.
    pub agrees: bool,
}

/// What one §13.5 row did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleRowOutcome {
    /// The row.
    pub row: LifecycleRow,
    /// What source expected of it, before the run.
    pub expected: LifecycleOutcome,
    /// What the reading process actually did.
    pub observed: LifecycleOutcome,
    /// How expectation and observation met.
    pub verdict: RowVerdict,
    /// Every check the reading process recorded on the way.
    pub checks: Vec<CheckOutcome>,
}

impl LifecycleRowOutcome {
    /// Builds one row outcome, comparing against source's expectation.
    #[must_use]
    pub fn new(
        row: LifecycleRow,
        expected: LifecycleOutcome,
        observed: LifecycleOutcome,
        checks: Vec<CheckOutcome>,
    ) -> Self {
        let verdict = if !observed.establishes_fact() {
            RowVerdict::NotTargetEvidence
        } else if expected == observed {
            RowVerdict::Agrees
        } else {
            RowVerdict::Disagrees
        };
        Self {
            row,
            expected,
            observed,
            verdict,
            checks,
        }
    }
}

/// One complete pass of the reading process over every row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingPass {
    /// Which pass this is.
    pub attempt: u32,
    /// The operating-system process that made it.
    ///
    /// Recorded because §13.5's closing sentence is about processes, and
    /// a report whose passes shared a process identifier would be
    /// reporting two function calls.
    pub pid: u32,
    /// The wallet that process created for itself.
    pub wallet_name: String,
    /// What it did with each row.
    pub rows: Vec<LifecycleRowOutcome>,
}

/// The §14.1 `FreshProcessLifecycle` report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshProcessLifecycleReport {
    /// What this report may claim.
    pub role: LifecycleReportRole,
    /// The public record that crossed the boundary.
    pub handoff: PublicHandoff,
    /// The profile the creator's authorization actually used.
    pub authorization_profile: Option<AuthorizationProfile>,
    /// What the boundary destroyed.
    pub destruction: DestructionRecord,
    /// Every pass the reading processes made.
    pub passes: Vec<ReadingPass>,
    /// Rows the run could not build, with the target's own words.
    ///
    /// Present rather than absent. A row that reached no verdict
    /// establishes nothing, and dropping it would make the matrix look
    /// complete — `G11-W7-03`'s precedent, and the reason its finding
    /// was worth recording at all.
    pub unbuilt_rows: Vec<UnbuiltRow>,
    /// The genesis the run was bound to, as the node reported it.
    pub observed_genesis: String,
    /// The declared development network identity.
    pub observed_network: String,
    /// The integration tip the operator declared (ADR-018).
    pub declared_tip: Option<String>,
    /// The revision the node binary reported about itself.
    pub binary_reported_revision: Option<String>,
}

/// A row the run could not build, and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnbuiltRow {
    /// The row.
    pub row: LifecycleRow,
    /// What the target or the adapter said, verbatim and unmapped.
    pub reason: String,
}

impl FreshProcessLifecycleReport {
    /// Whether the boundary between the processes is real.
    #[must_use]
    pub fn boundary_holds(&self) -> bool {
        self.destruction.holds() && self.passes_ran_in_distinct_processes()
    }

    /// Whether every pass was made by a different process from the others.
    ///
    /// §13.5's closing sentence, checked rather than asserted.
    #[must_use]
    pub fn passes_ran_in_distinct_processes(&self) -> bool {
        let mut seen: Vec<u32> = self.passes.iter().map(|pass| pass.pid).collect();
        let total = seen.len();
        seen.sort_unstable();
        seen.dedup();
        seen.len() == total && total > 0
    }

    /// Whether the published record verified.
    ///
    /// The one row the section exists to obtain. Every other row shows
    /// it did not pass by accident.
    #[must_use]
    pub fn published_record_verifies(&self) -> bool {
        !self.passes.is_empty()
            && self.passes.iter().all(|pass| {
                pass.rows.iter().any(|row| {
                    row.row == LifecycleRow::Accepted && row.observed == LifecycleOutcome::Verified
                })
            })
    }

    /// Every row whose observation contradicts what source expected.
    #[must_use]
    pub fn disagreements(&self) -> Vec<(u32, LifecycleRow)> {
        self.passes
            .iter()
            .flat_map(|pass| {
                pass.rows
                    .iter()
                    .filter(|row| row.verdict == RowVerdict::Disagrees)
                    .map(move |row| (pass.attempt, row.row))
            })
            .collect()
    }

    /// Whether every pass reached the same verdict on every row.
    ///
    /// §13.5's hidden-cache dependency. Two passes that disagreed would
    /// mean the first one's result came partly from state the second
    /// one did not have.
    #[must_use]
    pub fn passes_agree(&self) -> bool {
        let Some(first) = self.passes.first() else {
            return false;
        };
        let shape = |pass: &ReadingPass| -> Vec<(LifecycleRow, LifecycleOutcome)> {
            pass.rows
                .iter()
                .map(|row| (row.row, row.observed))
                .collect()
        };
        let reference = shape(first);
        self.passes.len() > 1 && self.passes.iter().all(|pass| shape(pass) == reference)
    }

    /// Whether the record carries any field naming owner-private material.
    ///
    /// A tautology against [`PublicHandoff`]'s own field list, and
    /// deliberately so: what it checks is that the SCHEMA admits no such
    /// field, so widening the struct without thinking breaks this rather
    /// than passing quietly.
    #[must_use]
    pub fn record_names_no_owner_private_material() -> bool {
        !PublicHandoff::field_names()
            .iter()
            .any(|field| names_owner_private_material(field))
    }

    /// Whether every row source stated was either run or recorded unbuilt.
    ///
    /// A matrix missing a row is not a matrix that passed it.
    #[must_use]
    pub fn matrix_is_complete(&self) -> bool {
        let Some(first) = self.passes.first() else {
            return false;
        };
        canonical_lifecycle_matrix().into_iter().all(|expectation| {
            first.rows.iter().any(|row| row.row == expectation.row)
                || self
                    .unbuilt_rows
                    .iter()
                    .any(|row| row.row == expectation.row)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn destruction() -> DestructionRecord {
        DestructionRecord {
            scope: "the creator's wallet and process".to_owned(),
            wallet_directory: "elementsregtest/wallets/a".to_owned(),
            destroyed_files: vec![DestroyedFile {
                path: "wallet.dat".to_owned(),
                bytes: 98_304,
            }],
            destroyed_bytes: 98_304,
            wallet_directory_present_after: false,
            process_a_exit_status: Some(0),
            process_a_running_after: false,
        }
    }

    fn handoff() -> PublicHandoff {
        PublicHandoff {
            schema: crate::lifecycle::HANDOFF_SCHEMA.to_owned(),
            chain_name: "elementsregtest".to_owned(),
            network_id: "11".repeat(32),
            genesis_id: "ab".repeat(32),
            txid: "cd".repeat(32),
            output_index: 0,
            block_hash: "ef".repeat(32),
            block_height: 5,
            raw_transaction: "0200".to_owned(),
            claimed_explicit_amount: 5_000_000,
            claimed_explicit_asset: "b2".repeat(32),
            claimed_owner_address: "ert1p...".to_owned(),
        }
    }

    fn pass(attempt: u32, pid: u32, accepted: LifecycleOutcome) -> ReadingPass {
        ReadingPass {
            attempt,
            pid,
            wallet_name: format!("b{attempt}"),
            rows: vec![LifecycleRowOutcome::new(
                LifecycleRow::Accepted,
                LifecycleOutcome::Verified,
                accepted,
                Vec::new(),
            )],
        }
    }

    fn report(passes: Vec<ReadingPass>) -> FreshProcessLifecycleReport {
        FreshProcessLifecycleReport {
            role: LifecycleReportRole::Experimental,
            handoff: handoff(),
            authorization_profile: Some(AuthorizationProfile::SighashDefault),
            destruction: destruction(),
            passes,
            unbuilt_rows: Vec::new(),
            observed_genesis: "ab".repeat(32),
            observed_network: "11".repeat(32),
            declared_tip: None,
            binary_reported_revision: None,
        }
    }

    #[test]
    fn the_public_record_admits_no_owner_private_field() {
        assert!(FreshProcessLifecycleReport::record_names_no_owner_private_material());
    }

    #[test]
    fn a_record_with_an_extra_field_does_not_parse() {
        // The allow-list is the rule that actually holds, and it is
        // serde's rather than a hand-written check that could be
        // forgotten.
        let mut value = serde_json::to_value(handoff()).expect("a record serializes");
        value["owner_blinding_key"] = serde_json::json!("00");
        assert!(serde_json::from_value::<PublicHandoff>(value).is_err());
    }

    #[test]
    fn an_innocuous_unknown_field_is_refused_too() {
        let mut value = serde_json::to_value(handoff()).expect("a record serializes");
        value["note"] = serde_json::json!("harmless");
        assert!(serde_json::from_value::<PublicHandoff>(value).is_err());
    }

    #[test]
    fn the_name_ban_catches_what_a_widened_allow_list_would_admit() {
        assert!(names_owner_private_material("owner_blinding_key"));
        assert!(names_owner_private_material("WALLET_PATH"));
        assert!(names_owner_private_material("output_descriptor"));
        assert!(!names_owner_private_material("txid"));
        assert!(!names_owner_private_material("block_height"));
    }

    #[test]
    fn a_boundary_needs_both_halves() {
        let mut record = destruction();
        assert!(record.holds());
        // The creator exited and its wallet is still on disk: the keys
        // are there for anything that opens the directory.
        record.wallet_directory_present_after = true;
        assert!(!record.holds());
        record.wallet_directory_present_after = false;
        // The wallet is gone and the creator is still running: the keys
        // are in that process's memory.
        record.process_a_running_after = true;
        assert!(!record.holds());
    }

    #[test]
    fn two_passes_in_one_process_are_not_two_processes() {
        let shared = report(vec![
            pass(1, 4242, LifecycleOutcome::Verified),
            pass(2, 4242, LifecycleOutcome::Verified),
        ]);
        assert!(!shared.passes_ran_in_distinct_processes());
        assert!(!shared.boundary_holds());

        let separate = report(vec![
            pass(1, 4242, LifecycleOutcome::Verified),
            pass(2, 4243, LifecycleOutcome::Verified),
        ]);
        assert!(separate.passes_ran_in_distinct_processes());
        assert!(separate.boundary_holds());
    }

    #[test]
    fn passes_that_disagree_are_a_cache_dependency() {
        let document = report(vec![
            pass(1, 1, LifecycleOutcome::Verified),
            pass(2, 2, LifecycleOutcome::RefusedOutputSpent),
        ]);
        assert!(!document.passes_agree());
        assert!(!document.published_record_verifies());

        let agreeing = report(vec![
            pass(1, 1, LifecycleOutcome::Verified),
            pass(2, 2, LifecycleOutcome::Verified),
        ]);
        assert!(agreeing.passes_agree());
        assert!(agreeing.published_record_verifies());
    }

    #[test]
    fn one_pass_cannot_establish_cache_independence() {
        // A single reading proves the record is readable and says
        // nothing about whether the reading was cached.
        let single = report(vec![pass(1, 1, LifecycleOutcome::Verified)]);
        assert!(!single.passes_agree());
        assert!(single.published_record_verifies());
    }

    #[test]
    fn an_unexpected_outcome_is_reported_rather_than_absorbed() {
        let outcome = LifecycleRowOutcome::new(
            LifecycleRow::CopiedEvidence,
            LifecycleOutcome::RefusedCopiedEvidence,
            LifecycleOutcome::Verified,
            Vec::new(),
        );
        assert_eq!(outcome.verdict, RowVerdict::Disagrees);
    }

    #[test]
    fn an_unbuilt_row_establishes_nothing() {
        let outcome = LifecycleRowOutcome::new(
            LifecycleRow::StaleEvidence,
            LifecycleOutcome::RefusedOutputSpent,
            LifecycleOutcome::FixtureConstructionFailure,
            Vec::new(),
        );
        assert_eq!(outcome.verdict, RowVerdict::NotTargetEvidence);
        assert!(!outcome.observed.establishes_fact());
    }

    #[test]
    fn a_missing_row_leaves_the_matrix_incomplete() {
        let mut document = report(vec![pass(1, 1, LifecycleOutcome::Verified)]);
        assert!(!document.matrix_is_complete());
        // Recorded as unbuilt, the row is accounted for rather than absent.
        for expectation in canonical_lifecycle_matrix() {
            if expectation.row == LifecycleRow::Accepted {
                continue;
            }
            document.unbuilt_rows.push(UnbuiltRow {
                row: expectation.row,
                reason: "not run in this fixture".to_owned(),
            });
        }
        assert!(document.matrix_is_complete());
    }

    /// `G12-R06`: completeness is decided by the first pass alone.
    ///
    /// A Guide-12 preflight reproduction. It asserts the current
    /// behaviour, not the wanted one, and the wave that repairs the row
    /// flips it.
    ///
    /// The check reads `passes.first()` and asks, for each expectation,
    /// whether *some* row of that pass names it. Two consequences
    /// follow, and both are shown here: a later pass may answer fewer
    /// rows than the matrix states and the report still calls the matrix
    /// complete, and a pass may answer one row twice and nothing
    /// notices, because presence is asked and census equality is not.
    ///
    /// The emitting binary gates on `boundary_holds` and
    /// `matrix_is_complete` and never calls `passes_agree`, so the
    /// disagreement these passes would show is not consulted; and
    /// `passes_ran_in_distinct_processes` is satisfied by a single pass,
    /// so nothing requires the two complete distinct-process passes the
    /// cache-independence claim rests on.
    #[test]
    fn matrix_completeness_reads_only_the_first_pass() {
        let mut complete = report(vec![pass(1, 1, LifecycleOutcome::Verified)]);
        for expectation in canonical_lifecycle_matrix() {
            if expectation.row == LifecycleRow::Accepted {
                continue;
            }
            complete.unbuilt_rows.push(UnbuiltRow {
                row: expectation.row,
                reason: "not run in this fixture".to_owned(),
            });
        }
        assert!(complete.matrix_is_complete());

        // A second pass that answered nothing at all. The census the
        // first pass carried is never asked of it.
        let mut later_pass_is_empty = complete.clone();
        later_pass_is_empty.passes.push(ReadingPass {
            attempt: 2,
            pid: 2,
            wallet_name: "b2".to_owned(),
            rows: Vec::new(),
        });
        assert!(
            later_pass_is_empty.matrix_is_complete(),
            "G12-R06: a later pass is expected to go unchecked while the row is open",
        );
        assert!(later_pass_is_empty.boundary_holds());

        // The first pass answering one row twice. Presence holds, so the
        // duplicate is invisible to the completeness check.
        let mut duplicated = complete.clone();
        let repeated = duplicated.passes[0].rows[0].clone();
        duplicated.passes[0].rows.push(repeated);
        assert_eq!(duplicated.passes[0].rows.len(), 2);
        assert!(
            duplicated.matrix_is_complete(),
            "G12-R06: a duplicated row is expected to go unchecked while the row is open",
        );

        // And one pass alone satisfies the process boundary, so nothing
        // asks for the second complete pass at all.
        assert!(complete.passes_ran_in_distinct_processes());
        assert!(!complete.passes_agree());
    }

    #[test]
    fn the_role_admits_no_canonical_claim() {
        assert_eq!(report(Vec::new()).role, LifecycleReportRole::Experimental);
    }

    #[test]
    fn every_row_of_the_matrix_states_its_reasoning() {
        let matrix = canonical_lifecycle_matrix();
        assert_eq!(matrix.len(), 9);
        for expectation in &matrix {
            assert!(
                expectation.reasoning.len() > 80,
                "{:?} states no reasoning",
                expectation.row
            );
        }
        // Exactly one row is expected to verify. A matrix with two would
        // be claiming a corrupted record is as good as the published one.
        assert_eq!(
            matrix
                .iter()
                .filter(|row| row.expected == LifecycleOutcome::Verified)
                .count(),
            1
        );
    }
}
