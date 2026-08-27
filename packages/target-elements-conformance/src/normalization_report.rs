//! The typed normalization-safety report of Guide 11 §14.
//!
//! # This report is experimental, and the type says so
//!
//! §14 asks for a safety report over a candidate path: what the path
//! makes public and why, what it preserves, which mutations it survives,
//! and on what it was observed. What it does not authorize is a claim
//! that the candidate is *selected* — Guide 11 §24's matrix does that,
//! and this wave does not reach it. So the role has one variant, exactly
//! as [`crate::conservation_report::ConservationReportRole`] does, and a
//! canonical role must be added by whichever wave earns it rather than
//! chosen by a caller who would like one.
//!
//! # The refusal layer is derived here, from data the adapter did not
//! classify
//!
//! Three of the §10.4 rows are transactions the target accepts and the
//! claim must still refuse. The adapter reports the layer the node
//! answered at, the outputs the claim named, and the outputs the target
//! carries; [`derive_refusal`] turns those into a refusal layer. The
//! division matters: an adapter that decided the layer could report a
//! report-layer refusal for a transaction it never compared, and Wave 7
//! recorded what that costs (`G11-W7-06`).
//!
//! # A target refusal preempts a report refusal, and that ordering is
//! not a preference
//!
//! Where the target refused, the refusal is the target's — even when the
//! claim also disagrees with the transaction. A transaction the target
//! will not have never becomes a record, so a report cannot be what
//! refused it. The three post-signing rows are the case in point: their
//! output sets also break closure, and reporting them as report-layer
//! refusals would credit the report with work the signature did.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::conservation_report::{ConservationReportRole, RowVerdict};
use crate::declassification::Declassification;
use crate::normalization::{
    AuthorizationProfile, ClosureFinding, NormalizationClaim, NormalizationMutation,
    PreservationFinding, RefusalLayer, canonical_mutation_matrix, closure_finding,
    preservation_finding,
};
use crate::protocol::{NativeNormalizationResponse, ObservedOutcomeLayer, ResponseShapeDefect};

/// What one §10.4 row did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationOutcome {
    /// The mutation applied.
    pub mutation: NormalizationMutation,
    /// Where the row's author expected the refusal to come from.
    pub expected: RefusalLayer,
    /// Where it actually came from.
    pub observed: RefusalLayer,
    /// The layer the target itself answered at, where it answered.
    pub observed_target_layer: Option<ObservedOutcomeLayer>,
    /// What the target or the adapter said, verbatim and unmapped.
    pub observed_detail: Option<String>,
    /// Whether the observed output set is exactly the claimed one.
    pub closure: ClosureFinding,
    /// Whether §10.1's properties survived.
    pub preservation: PreservationFinding,
    /// The profile the owner's signature actually used.
    pub authorization_profile: Option<AuthorizationProfile>,
    /// How expectation and observation met.
    pub verdict: RowVerdict,
    /// The transaction the run built, where it built one.
    pub materialized_transaction: Option<String>,
}

impl MutationOutcome {
    /// Whether this row establishes anything at all.
    ///
    /// A row whose fixture could not be built and one whose environment
    /// failed establish nothing: neither the target nor the report ever
    /// judged the transaction. Counting them is the mistake §8.3's
    /// vocabulary exists to prevent.
    #[must_use]
    pub fn establishes_fact(&self) -> bool {
        self.observed_target_layer
            .is_some_and(|layer| layer.is_target_verdict())
    }
}

/// Turns one adapter observation into the layer that refused it.
///
/// # The order is target first, and it is load-bearing
///
/// Where the target refused, that is the refusal: a transaction the
/// target will not accept never reaches a record for a report to check.
/// Only where the target accepted does the claim get asked, and then the
/// two checks are the whole content of a report-layer refusal.
///
/// The two non-verdict layers return no refusal at all. They are not
/// "not refused" — nothing judged the transaction — and the caller
/// distinguishes them by [`MutationOutcome::establishes_fact`] rather
/// than by reading a layer that would be a fiction.
#[must_use]
pub const fn derive_refusal(
    target_layer: ObservedOutcomeLayer,
    closure: &ClosureFinding,
    preservation: &PreservationFinding,
) -> Option<RefusalLayer> {
    match target_layer {
        ObservedOutcomeLayer::FixtureConstructionFailure
        | ObservedOutcomeLayer::ExecutorInfrastructureFailure => None,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript => Some(RefusalLayer::TargetConsensus),
        // A signature failure reaches the mempool with the mandatory
        // script prefix, which is the layer the adapter reports as a
        // script-path rejection. On this path the script being run is the
        // owner's authorization, so the two names describe one event.
        //
        // A key-path refusal joins it, and shares the arm rather than
        // taking one of its own because the two really are one answer at
        // THIS boundary: both are the target judging an offered signature
        // and finding it invalid, and the key path reaches that verdict
        // without a leaf at all. The distinction the two layers carry is
        // about which path ran, which is a fact the observed layer keeps
        // and this projection does not need. The member is named rather
        // than swept into a wildcard, so the next layer added to the
        // vocabulary fails to compile here instead of being absorbed.
        ObservedOutcomeLayer::ScriptPathRejection | ObservedOutcomeLayer::KeyPathRejection => {
            Some(RefusalLayer::TargetSignature)
        }
        // A transaction the target would not relay is not a refusal of
        // the claim, and is reported as its own thing rather than folded
        // into either neighbour: no row expects it, so it surfaces as a
        // disagreement instead of passing as a consensus refusal.
        ObservedOutcomeLayer::RelayPolicyRejection => Some(RefusalLayer::TargetRelayPolicy),
        ObservedOutcomeLayer::Accepted => {
            if closure.holds() && preservation.holds() {
                Some(RefusalLayer::NotRefused)
            } else {
                Some(RefusalLayer::ReportLayer)
            }
        }
    }
}

/// Builds one row's outcome from what the adapter reported.
///
/// The closure and preservation findings are computed here, over the
/// adapter's two output lists, so that the comparison happens in the
/// package that owns the claim rather than in the process that built the
/// transaction.
#[must_use]
pub fn outcome_of(
    mutation: NormalizationMutation,
    expected: RefusalLayer,
    response: &NativeNormalizationResponse,
) -> MutationOutcome {
    let closure = closure_finding(&response.claimed_outputs, &response.observed_outputs);
    let preservation = preservation_finding(&response.claimed_outputs, &response.observed_outputs);
    let refusal = derive_refusal(response.observed_layer, &closure, &preservation);

    let (observed, verdict) = match refusal {
        None => (RefusalLayer::NotRefused, RowVerdict::NotTargetEvidence),
        Some(layer) if layer == expected => (layer, RowVerdict::Agrees),
        Some(layer) => (layer, RowVerdict::Disagrees),
    };

    MutationOutcome {
        mutation,
        expected,
        observed,
        observed_target_layer: Some(response.observed_layer),
        observed_detail: response.observed_detail.clone(),
        closure,
        preservation,
        authorization_profile: response.authorization_profile,
        verdict,
        materialized_transaction: response
            .transaction_bytes
            .as_ref()
            .map(|bytes| hex_of(bytes)),
    }
}

fn hex_of(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// Why one run record's responses are not a census of the matrix.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NormalizationIngestionDefect {
    /// A response contradicts itself under its own shape rules.
    SelfContradictoryResponse {
        /// The row the response answered.
        row: String,
        /// What the shape rules refused.
        defect: ResponseShapeDefect,
    },
    /// Two responses answered the same row.
    ///
    /// Reported rather than resolved: a run that answered one row twice
    /// has not said which answer is the row's, and keeping either one is
    /// the harness choosing evidence on the run's behalf.
    DuplicateResponse {
        /// The row answered more than once.
        row: String,
    },
    /// A response answered a row the canonical matrix does not carry.
    UnexpectedRow {
        /// The row the run named.
        row: String,
    },
    /// The canonical matrix carries a row the run did not answer.
    UnansweredRow {
        /// The row nobody answered.
        row: String,
    },
}

impl fmt::Display for NormalizationIngestionDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfContradictoryResponse { row, defect } => {
                write!(
                    formatter,
                    "the response for {row} contradicts itself: {defect:?}"
                )
            }
            Self::DuplicateResponse { row } => {
                write!(formatter, "the run answered {row} more than once")
            }
            Self::UnexpectedRow { row } => write!(
                formatter,
                "the run answered {row}, which the canonical mutation matrix does not carry",
            ),
            Self::UnansweredRow { row } => {
                write!(formatter, "the run answered no row for {row}")
            }
        }
    }
}

/// The wire spelling of one mutation, as a run record writes it.
#[must_use]
pub fn mutation_wire_spelling(mutation: NormalizationMutation) -> String {
    serde_json::to_value(mutation)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_default()
}

/// One run's responses, indexed by the row each answers.
///
/// # Why the census is exact in both directions
///
/// The report is a statement about the canonical matrix, so the run's
/// answers and the matrix's rows must be the same set. Indexing alone
/// is not that: a second answer for one row used to replace the first
/// silently, and a response naming a row the matrix does not carry was
/// dropped without a word, because the report loop read the matrix and
/// never the census. Either way the report described a run that did not
/// happen — one whose duplicate was resolved by arrival order, or one
/// whose extra answer was never mentioned.
///
/// So both directions are checked here and neither is repaired: an
/// unanswered row, an unexpected row, and a duplicated row are each a
/// refusal, and the caller emits no report at all.
///
/// # Errors
///
/// [`NormalizationIngestionDefect`], naming the first row that breaks
/// one of the rules, in the order the rules are written.
pub fn ingest_normalization_responses(
    responses: impl IntoIterator<Item = NativeNormalizationResponse>,
) -> Result<BTreeMap<String, NativeNormalizationResponse>, NormalizationIngestionDefect> {
    let mut answered: BTreeMap<String, NativeNormalizationResponse> = BTreeMap::new();

    for response in responses {
        let row = response.case.normalization.clone();
        response.validate_shape().map_err(|defect| {
            NormalizationIngestionDefect::SelfContradictoryResponse {
                row: row.clone(),
                defect,
            }
        })?;
        if answered.insert(row.clone(), response).is_some() {
            return Err(NormalizationIngestionDefect::DuplicateResponse { row });
        }
    }

    let expected: BTreeSet<String> = canonical_mutation_matrix()
        .into_iter()
        .map(|row| mutation_wire_spelling(row.mutation))
        .collect();

    // Answered-but-unexpected first: a run naming a row nobody asked
    // for is describing some other matrix, and saying so is more useful
    // than reporting the rows of this one it happens to be missing.
    if let Some(row) = answered.keys().find(|row| !expected.contains(*row)) {
        return Err(NormalizationIngestionDefect::UnexpectedRow { row: row.clone() });
    }
    if let Some(row) = expected.iter().find(|row| !answered.contains_key(*row)) {
        return Err(NormalizationIngestionDefect::UnansweredRow { row: row.clone() });
    }

    Ok(answered)
}

/// What one normalization run established.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationReport {
    /// What this report claims to be.
    pub role: ConservationReportRole,
    /// The claim the prototype made.
    pub claim: NormalizationClaim,
    /// What the path made public, and why each fact is public.
    ///
    /// Guide 11 §14.4's requirement, carried as the typed list rather
    /// than as prose: a report whose disclosures were sentences could
    /// not be checked for the semantic-necessity claim §14.4 forbids.
    pub disclosures: Vec<Declassification>,
    /// The profile the owner's authorization used.
    pub authorization_profile: Option<AuthorizationProfile>,
    /// The chain the run observed.
    pub observed_genesis: String,
    /// The network identity that chain was bound to.
    pub observed_network: String,
    /// The integration tip the operator declared they ran.
    pub declared_tip: Option<String>,
    /// The revision the node binary reported about itself.
    pub binary_reported_revision: Option<String>,
    /// Every §10.4 row, in matrix order.
    pub rows: Vec<MutationOutcome>,
}

impl NormalizationReport {
    /// Whether every row that reached a verdict met its expectation.
    ///
    /// Not a gate and not "the run passed": a run whose rows mostly
    /// failed to build would satisfy it, which is why
    /// [`Self::rows_establishing_facts`] is reported beside it and
    /// neither is useful alone.
    #[must_use]
    pub fn every_verdict_agrees(&self) -> bool {
        self.rows
            .iter()
            .filter(|row| row.establishes_fact())
            .all(|row| row.verdict == RowVerdict::Agrees)
    }

    /// How many rows actually reached a verdict.
    #[must_use]
    pub fn rows_establishing_facts(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.establishes_fact())
            .count()
    }

    /// Whether the prototype's own row stands.
    ///
    /// # The one row whose success is the candidate's success
    ///
    /// Eight rows establish that a mutation is refused. Exactly one
    /// establishes that the unmutated claim is *accepted*, and without it
    /// the other eight would describe a path that refuses everything —
    /// which is trivially safe and useless. A disposition resting on this
    /// report has to ask this question specifically.
    #[must_use]
    pub fn unmutated_claim_stands(&self) -> bool {
        self.rows.iter().any(|row| {
            row.mutation == NormalizationMutation::None
                && row.observed == RefusalLayer::NotRefused
                && row.verdict == RowVerdict::Agrees
        })
    }

    /// Whether the authorization committed to the finalized output set.
    ///
    /// §10.3's prerequisite, asked of the profile the run actually
    /// observed rather than of the one it intended.
    #[must_use]
    pub fn authorization_commits_all_outputs(&self) -> bool {
        self.authorization_profile
            .is_some_and(|profile| profile.commits_all_outputs())
    }

    /// The rows the report layer alone refused.
    ///
    /// The wave's substantive finding, available as data: these are
    /// transactions the target accepted and the claim did not.
    #[must_use]
    pub fn report_layer_refusals(&self) -> Vec<NormalizationMutation> {
        self.rows
            .iter()
            .filter(|row| row.observed == RefusalLayer::ReportLayer)
            .map(|row| row.mutation)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalization::{ClaimedOutput, ObservedOutput, OutputRole, PreservedProperty};

    fn accepted_response(
        claimed: Vec<ClaimedOutput>,
        observed: Vec<ObservedOutput>,
    ) -> NativeNormalizationResponse {
        NativeNormalizationResponse {
            schema: crate::protocol::NATIVE_PROTOCOL_SCHEMA,
            case: crate::protocol::NormalizationCaseId {
                normalization: "probe".to_owned(),
            },
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            claimed_outputs: claimed,
            observed_outputs: observed,
            authorization_profile: Some(AuthorizationProfile::SighashDefault),
            observed_witness_sizes: vec![vec![64]],
            transaction_bytes: Some(vec![0xab, 0xcd]),
        }
    }

    fn claimed(script: u8, amount: Option<u64>) -> ClaimedOutput {
        ClaimedOutput {
            role: OutputRole::Normalized,
            script_pubkey: vec![script; 4],
            explicit_amount: amount,
            explicit_asset: amount.map(|_| [7_u8; 32]),
        }
    }

    fn observed(script: u8, amount: Option<u64>) -> ObservedOutput {
        ObservedOutput {
            script_pubkey: vec![script; 4],
            explicit_amount: amount,
            explicit_asset: amount.map(|_| [7_u8; 32]),
            is_fee: false,
        }
    }

    #[test]
    fn an_accepted_and_agreeing_transaction_is_not_refused() {
        let response =
            accepted_response(vec![claimed(0xaa, Some(5))], vec![observed(0xaa, Some(5))]);
        let outcome = outcome_of(
            NormalizationMutation::None,
            RefusalLayer::NotRefused,
            &response,
        );
        assert_eq!(outcome.observed, RefusalLayer::NotRefused);
        assert_eq!(outcome.verdict, RowVerdict::Agrees);
    }

    #[test]
    fn an_accepted_transaction_the_claim_disowns_is_a_report_layer_refusal() {
        // The wave's substantive prediction: the target has nothing to
        // say, and the claim refuses.
        let response =
            accepted_response(vec![claimed(0xaa, Some(5))], vec![observed(0xcc, Some(5))]);
        let outcome = outcome_of(
            NormalizationMutation::OwnerChanged,
            RefusalLayer::ReportLayer,
            &response,
        );
        assert_eq!(outcome.observed, RefusalLayer::ReportLayer);
        assert_eq!(outcome.verdict, RowVerdict::Agrees);
        assert!(!outcome.closure.holds());
        assert!(!outcome.preservation.holds());
    }

    #[test]
    fn a_hidden_output_is_refused_by_closure_while_preservation_holds() {
        // Consensus accepts it and the normalized output is untouched, so
        // closure alone carries the refusal. If preservation were what
        // caught this, the report would be claiming the owner was paid
        // wrongly when they were paid exactly right.
        let response = accepted_response(
            vec![claimed(0xaa, Some(5))],
            vec![observed(0xaa, Some(5)), observed(0xcc, None)],
        );
        let outcome = outcome_of(
            NormalizationMutation::HiddenPrivateOutput,
            RefusalLayer::ReportLayer,
            &response,
        );
        assert_eq!(outcome.observed, RefusalLayer::ReportLayer);
        assert!(!outcome.closure.holds());
        assert!(outcome.preservation.holds());
    }

    #[test]
    fn a_target_refusal_preempts_a_report_refusal() {
        // The three post-signing rows also break closure. Reporting them
        // as report-layer refusals would credit the report with work the
        // signature did.
        let mut response =
            accepted_response(vec![claimed(0xaa, Some(5))], vec![observed(0xcc, Some(5))]);
        response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
        let outcome = outcome_of(
            NormalizationMutation::OutputMutatedAfterSigning,
            RefusalLayer::TargetSignature,
            &response,
        );
        assert_eq!(outcome.observed, RefusalLayer::TargetSignature);
        assert!(!outcome.closure.holds());
        assert_eq!(outcome.verdict, RowVerdict::Agrees);
    }

    #[test]
    fn a_construction_failure_establishes_nothing() {
        let mut response = accepted_response(Vec::new(), Vec::new());
        response.observed_layer = ObservedOutcomeLayer::FixtureConstructionFailure;
        let outcome = outcome_of(
            NormalizationMutation::None,
            RefusalLayer::NotRefused,
            &response,
        );
        assert!(!outcome.establishes_fact());
        assert_eq!(outcome.verdict, RowVerdict::NotTargetEvidence);
    }

    #[test]
    fn a_disagreement_is_reported_rather_than_absorbed() {
        // The target accepted a mutation expected to be refused by
        // consensus. Nothing here rewrites the expectation.
        let response =
            accepted_response(vec![claimed(0xaa, Some(5))], vec![observed(0xaa, Some(5))]);
        let outcome = outcome_of(
            NormalizationMutation::AssetChanged,
            RefusalLayer::TargetConsensus,
            &response,
        );
        assert_eq!(outcome.verdict, RowVerdict::Disagrees);
        assert_eq!(outcome.observed, RefusalLayer::NotRefused);
    }

    fn report(rows: Vec<MutationOutcome>) -> NormalizationReport {
        NormalizationReport {
            role: ConservationReportRole::Experimental,
            claim: NormalizationClaim::canonical(),
            disclosures: crate::declassification::normalization_declassifications(),
            authorization_profile: Some(AuthorizationProfile::SighashDefault),
            observed_genesis: "00".to_owned(),
            observed_network: "11".to_owned(),
            declared_tip: None,
            binary_reported_revision: None,
            rows,
        }
    }

    fn row(
        mutation: NormalizationMutation,
        observed: RefusalLayer,
        verdict: RowVerdict,
    ) -> MutationOutcome {
        MutationOutcome {
            mutation,
            expected: observed,
            observed,
            observed_target_layer: Some(ObservedOutcomeLayer::Accepted),
            observed_detail: None,
            closure: ClosureFinding::Holds,
            preservation: PreservationFinding::Holds,
            authorization_profile: Some(AuthorizationProfile::SighashDefault),
            verdict,
            materialized_transaction: None,
        }
    }

    #[test]
    fn the_unmutated_row_is_what_makes_the_report_a_prototype() {
        let standing = report(vec![row(
            NormalizationMutation::None,
            RefusalLayer::NotRefused,
            RowVerdict::Agrees,
        )]);
        assert!(standing.unmutated_claim_stands());

        // A report of eight refusals and no accepted claim describes a
        // path that refuses everything, which is not a prototype.
        let refusing = report(vec![row(
            NormalizationMutation::AmountChanged,
            RefusalLayer::ReportLayer,
            RowVerdict::Agrees,
        )]);
        assert!(!refusing.unmutated_claim_stands());
    }

    #[test]
    fn the_report_states_the_profile_it_observed() {
        assert!(report(Vec::new()).authorization_commits_all_outputs());
        let mut unsigned = report(Vec::new());
        unsigned.authorization_profile = None;
        assert!(!unsigned.authorization_commits_all_outputs());
    }

    #[test]
    fn the_disclosures_are_the_normalization_list() {
        let document = report(Vec::new());
        assert_eq!(document.disclosures.len(), 3);
        for entry in &document.disclosures {
            assert!(!entry.reason.is_semantic_necessity(), "{}", entry.fact);
        }
    }

    #[test]
    fn the_role_admits_no_canonical_claim() {
        assert_eq!(
            report(Vec::new()).role,
            ConservationReportRole::Experimental
        );
    }

    #[test]
    fn preservation_names_the_property_that_broke() {
        let response =
            accepted_response(vec![claimed(0xaa, Some(5))], vec![observed(0xaa, Some(6))]);
        let outcome = outcome_of(
            NormalizationMutation::AmountChanged,
            RefusalLayer::ReportLayer,
            &response,
        );
        let PreservationFinding::Violated { properties } = &outcome.preservation else {
            panic!("a changed amount breaks preservation");
        };
        assert_eq!(properties, &vec![PreservedProperty::SemanticAmount]);
    }
}
