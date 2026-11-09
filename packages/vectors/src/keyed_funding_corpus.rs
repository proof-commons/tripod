//! Admission of the keyed funding archive without executing a node.
//!
//! Complete protocol responses come from the capture's wire JSON journal. The
//! shared birth-corpus reader checks framing, manifest and packet binding;
//! funding projections retain the pre-script consensus layer. No Debug decoder
//! supplies responses here. Admission establishes
//! these 36 subjects, not a final-byte funding recipe.

use std::fmt::Write as _;
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationSubject,
};

use crate::keyed_birth::Wave1TargetPacket;
pub use crate::keyed_birth_corpus::BirthAdmissionRefusal as FundingAdmissionRefusal;
use crate::keyed_birth_corpus::{
    BirthPolicyFact, bind_packet, capture_payload, capture_record, hex, manifest_text,
    policy_facts, require, response_detail,
};
use crate::keyed_funding::{
    FEE_OFFSETS, FundingOutcome, FundingPlanner, FundingPoint, FundingRole, HANDLE_POINTS,
    replay_funding_capture,
};
use serde_json::{Value, json};

const CAPTURE_NAME: &str = "af32047e7ca3.report.capture";
const TIMING_NAME: &str = "af32047e7ca3.report.timing";
const CAPTURE_TIMING_NAME: &str = "af32047e7ca3.report.capture.timing";
const CAPTURE: &str =
    include_str!("../fixtures/keyed-funding-run-of-record/af32047e7ca3.report.capture");
const TIMING: &str =
    include_str!("../fixtures/keyed-funding-run-of-record/af32047e7ca3.report.timing");
const CAPTURE_TIMING: &str =
    include_str!("../fixtures/keyed-funding-run-of-record/af32047e7ca3.report.capture.timing");
const REPORT: &str = include_str!("../fixtures/keyed-funding-run-of-record/RUN-REPORT");
const MANIFEST: &str = include_str!("../fixtures/keyed-funding-run-of-record/MANIFEST.sha256");
const PINS: [&str; 5] = [
    "5a26bed499f478781d6866605bbafcd303097dca885e847bff2cef87e2dcb9d7",
    "ba4974b2368bba42ffbb16319bd976e55d4a9536707ca6eb38b35b62d73501f0",
    "88fc195b97991d3099513822e690cf5dda7172019bc15f6a61b7c19115efdbb2",
    "18f13b3cca7df832b0bedd88d55f6d66e26458628555bfed65d2fd4081eadbb8",
    "39a374d64557300f2b2e42069a573e117f9f99b79dac9b11f9e9f76814edde63",
];
type Result<T> = std::result::Result<T, FundingAdmissionRefusal>;

pub use crate::keyed_continuation_corpus::ContinuationArchive as FundingArchive;

/// Rebuilt packet, subject census and replayed observations admitted together.
pub struct AdmittedFundingCampaign {
    packet: Wave1TargetPacket,
    planner: FundingPlanner,
    policy_facts: Vec<BirthPolicyFact>,
}

impl AdmittedFundingCampaign {
    /// Rebuilt target, deployment, executable, executor and policy binding.
    #[must_use]
    pub const fn packet(&self) -> &Wave1TargetPacket {
        &self.packet
    }

    /// Every replayed request and complete response.
    #[must_use]
    pub const fn planner(&self) -> &FundingPlanner {
        &self.planner
    }

    /// All six policy facts, with observed or declared origins.
    #[must_use]
    pub fn policy_facts(&self) -> &[BirthPolicyFact] {
        &self.policy_facts
    }
}

/// Admit the immutable run after checking all five independent node-side digests.
/// # Errors
/// Refuses a changed digest, binding, journal, projection or subject classification.
pub fn admit_run_of_record() -> Result<AdmittedFundingCampaign> {
    for (bytes, pin) in [CAPTURE, TIMING, CAPTURE_TIMING, REPORT, MANIFEST]
        .into_iter()
        .zip(PINS)
    {
        require(
            hex(&sha256(bytes.as_bytes())) == pin,
            FundingAdmissionRefusal::Digest,
        )?;
    }
    admit_funding_archive(FundingArchive {
        capture: CAPTURE,
        timing: TIMING,
        capture_timing: CAPTURE_TIMING,
        report: REPORT,
        manifest: MANIFEST,
    })
}

/// Admit an archive against this run's exact bindings and complete wire journal.
///
/// Resealed semantic mutants can use this entry; select [`admit_run_of_record`]
/// when immutable acquisition identity is required. No debug transcript is parsed.
/// # Errors
/// Refuses missing dimensions, changed bindings, failed replay or any open subject.
pub fn admit_funding_archive(archive: FundingArchive<'_>) -> Result<AdmittedFundingCampaign> {
    let _span = tracing::info_span!("admit_funding_archive").entered();
    let result = admit(archive);
    tracing::info!(admitted = result.is_ok(), refusal = ?result.as_ref().err(), "funding admission complete");
    result
}

fn admit(archive: FundingArchive<'_>) -> Result<AdmittedFundingCampaign> {
    require(
        archive.manifest == archive_manifest(archive)
            && archive.timing == TIMING
            && archive.capture_timing == CAPTURE_TIMING
            && archive.report == REPORT,
        FundingAdmissionRefusal::Digest,
    )?;
    let (header, projections, payload) = capture_record(archive.capture, CAPTURE)?;
    require(
        payload["keyed_funding_capture_schema"] == 1
            && payload["eligible_for_review"] == true
            && payload["admitted"] == false
            && payload["construction_refusal"] == "None"
            && payload["replay_refusal"] == "None",
        FundingAdmissionRefusal::Dimension,
    )?;
    let reference = capture_payload(CAPTURE)?;
    let journal = &payload["wire_journal"];
    // The outer header independently fixes deployment, activation and capabilities.
    // Bind the redundant wire handshake/environment to the same archived facts.
    for field in ["handshake", "environment", "descriptor"] {
        require(
            !journal[field].is_null() && journal[field] == reference["wire_journal"][field],
            FundingAdmissionRefusal::Binding,
        )?;
    }
    let target = reviewed_elements_tapscript().map_err(|_| FundingAdmissionRefusal::Binding)?;
    let planner =
        replay_funding_capture(&target, journal).map_err(|_| FundingAdmissionRefusal::Replay)?;
    validate_projections(projections, &planner)?;
    let reference_response: NativeOperationResponse =
        serde_json::from_value(reference["wire_journal"]["operations"][0]["response"].clone())
            .map_err(|_| FundingAdmissionRefusal::Dimension)?;
    let reference_policy = response_detail(&reference_response)?;
    let packet = bind_packet(
        header,
        &payload,
        planner.exchanges(),
        &reference,
        &reference_policy["policy"],
    )?;
    validate_observations(&planner)?;
    require(
        payload["report"] == planner.report_json(),
        FundingAdmissionRefusal::Subject,
    )?;
    let policy_facts = policy_facts(packet.policy());
    Ok(AdmittedFundingCampaign {
        packet,
        planner,
        policy_facts,
    })
}

fn archive_manifest(archive: FundingArchive<'_>) -> String {
    manifest_text(&[
        (CAPTURE_NAME, archive.capture),
        (TIMING_NAME, archive.timing),
        ("RUN-REPORT", archive.report),
        (CAPTURE_TIMING_NAME, archive.capture_timing),
    ])
}

// The birth projection decoder deliberately accepts only accepted/script-path
// layers. Funding includes pre-script consensus refusals. Re-encode this run's
// auxiliary projections from its replayed wire responses without changing any
// layer or extending a shared reader's accepted vocabulary.
fn validate_projections(projections: &str, planner: &FundingPlanner) -> Result<()> {
    let mut expected = String::new();
    let encoded = |bytes: &[u8]| format!("{} {}", bytes.len(), hex(bytes));
    for (ordinal, (step, response)) in planner.exchanges().iter().enumerate() {
        if ordinal != 0 {
            writeln!(expected, "operation {ordinal} begin")
                .map_err(|_| FundingAdmissionRefusal::Dimension)?;
        }
        let bytes = match step.subject() {
            OperationSubject::Submission(s) => s.transaction_bytes.as_slice(),
            _ => &[],
        };
        let subject = serde_json::to_string(step.subject())
            .map_err(|_| FundingAdmissionRefusal::Dimension)?;
        let (verdict, layer) = match response.observed_layer {
            ObservedOutcomeLayer::Accepted => ("accepted", "accepted"),
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript => {
                ("refused", "consensus-rejection-before-script")
            }
            _ => return Err(FundingAdmissionRefusal::Subject),
        };
        let detail = response
            .observed_detail
            .as_deref()
            .ok_or(FundingAdmissionRefusal::Dimension)?;
        for (field, value) in [
            (
                "operation-id",
                encoded(format!("operation-{ordinal}").as_bytes()),
            ),
            (
                "request-id",
                encoded(format!("request-{ordinal}").as_bytes()),
            ),
            ("request-role", "auxiliary".into()),
            ("request-bytes", encoded(bytes)),
            ("request-subject", encoded(subject.as_bytes())),
            (
                "response-id",
                encoded(format!("response-{ordinal}").as_bytes()),
            ),
            (
                "response-request-id",
                encoded(format!("request-{ordinal}").as_bytes()),
            ),
            (
                "response-operation-id",
                encoded(format!("operation-{ordinal}").as_bytes()),
            ),
            ("response-verdict", verdict.into()),
            ("response-layer", layer.into()),
            (
                "response-target-identity",
                response.accepted_txid.as_deref().unwrap_or("none").into(),
            ),
            ("response-detail", encoded(detail.as_bytes())),
            ("attribution-control-request-id", "none".into()),
            ("attribution-control-identity", "none".into()),
            ("mutation-kind", "none".into()),
            ("mutation-locator", "none".into()),
            ("projection-input", "none".into()),
        ] {
            writeln!(expected, "{field} {value}")
                .map_err(|_| FundingAdmissionRefusal::Dimension)?;
        }
        writeln!(expected, "operation {ordinal} end")
            .map_err(|_| FundingAdmissionRefusal::Dimension)?;
    }
    require(expected == projections, FundingAdmissionRefusal::Replay)
}

const fn expected_outcome(point: FundingPoint) -> FundingOutcome {
    match point {
        FundingPoint::Handle(0) => FundingOutcome::ConsensusRefused,
        FundingPoint::Handle(1 | 2 | 10) | FundingPoint::FeeOffset(-2 | -1) => {
            FundingOutcome::RelayRefusedBlockMined
        }
        _ => FundingOutcome::RelayMined,
    }
}

fn validate_observations(planner: &FundingPlanner) -> Result<()> {
    let report = planner.report_json();
    require(
        report["anchors_source_merge"] == json!([27, 34]),
        FundingAdmissionRefusal::Subject,
    )?;
    for (index, record) in planner.records().iter().enumerate() {
        let expected = expected_outcome(record.subject.point);
        require(
            planner.outcome(index) == expected,
            FundingAdmissionRefusal::Subject,
        )?;
        let detail = response_detail(&record.submission.1)?;
        let reason = match record.subject.point {
            FundingPoint::Handle(0) => json!("bad-txns-in-ne-out"),
            FundingPoint::Handle(1 | 2 | 10) => json!("dust"),
            FundingPoint::FeeOffset(-2 | -1) => json!("min relay fee not met"),
            _ => Value::Null,
        };
        require(
            detail["relay_detail"] == reason
                && detail["relay_allowed"] == (expected == FundingOutcome::RelayMined),
            FundingAdmissionRefusal::Subject,
        )?;
        if expected == FundingOutcome::ConsensusRefused {
            require(
                detail["detail"]["verdict_reason"] == "bad-txns-in-ne-out"
                    && detail["detail"]["block_script_error"] == "unclassified",
                FundingAdmissionRefusal::Subject,
            )?;
        }
    }
    for role in [FundingRole::Source, FundingRole::Merge] {
        for fees in [false, true] {
            require(
                planner.boundary(role, fees) == expected_boundary(role, fees),
                FundingAdmissionRefusal::Subject,
            )?;
        }
    }
    Ok(())
}

fn expected_boundary(role: FundingRole, fees: bool) -> Value {
    let anchor: u64 = if role == FundingRole::Source { 27 } else { 34 };
    let points: Vec<_> = if fees {
        FEE_OFFSETS
            .into_iter()
            .map(|offset| {
                let refused = offset < 0;
                json!([
                    anchor.saturating_add_signed(offset),
                    if refused {
                        FundingOutcome::RelayRefusedBlockMined
                    } else {
                        FundingOutcome::RelayMined
                    },
                    refused
                ])
            })
            .collect()
    } else {
        HANDLE_POINTS
            .into_iter()
            .filter(|n| *n > 0)
            .map(|n| {
                let refused = n < 100;
                json!([
                    n,
                    if refused {
                        FundingOutcome::RelayRefusedBlockMined
                    } else {
                        FundingOutcome::RelayMined
                    },
                    refused
                ])
            })
            .collect()
    };
    json!({"role": role, "fee_points": fees, "points": points, "reading": "sampled refusal/admission bracket only", "last_refused": if fees {anchor - 1} else {10}, "first_accepted": if fees {anchor} else {100}, "adjacent_satoshis": fees, "no_interpolation": true})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyed_birth::PolicyFactOrigin;
    use crate::keyed_birth_corpus::single_field;
    use serde_json::Value;

    struct Archive {
        capture: String,
        manifest: String,
    }
    impl Archive {
        fn accepted() -> Self {
            let archive = Self {
                capture: CAPTURE.into(),
                manifest: MANIFEST.into(),
            };
            assert!(archive.admit().is_ok());
            archive
        }
        fn view(&self) -> FundingArchive<'_> {
            FundingArchive {
                capture: &self.capture,
                timing: TIMING,
                capture_timing: CAPTURE_TIMING,
                report: REPORT,
                manifest: &self.manifest,
            }
        }
        fn admit(&self) -> Result<AdmittedFundingCampaign> {
            admit_funding_archive(self.view())
        }
        fn seal(&mut self) {
            let content = self
                .capture
                .split_once("capture-content-sha256 ")
                .expect("footer")
                .0;
            self.capture = format!(
                "{content}capture-content-sha256 {}\nnative-capture-end report\n",
                hex(&sha256(content.as_bytes()))
            );
            self.manifest = archive_manifest(self.view());
        }
        fn payload(&mut self, edit: impl FnOnce(&mut Value)) {
            let old = single_field(&self.capture, "legacy-rendering")
                .expect("payload")
                .to_owned();
            let mut payload = capture_payload(&self.capture).expect("JSON record");
            edit(&mut payload);
            let new = payload.to_string();
            self.capture = self.capture.replace(
                &format!("legacy-rendering {old}"),
                &format!("legacy-rendering {} {}", new.len(), hex(new.as_bytes())),
            );
            self.seal();
        }
    }

    #[test]
    fn run_of_record_admits_exact_subjects_and_boundaries() {
        let admitted = admit_run_of_record().unwrap_or_else(|error| panic!("admission: {error:?}"));
        assert_eq!(admitted.planner().records().len(), 36);
        assert_eq!(admitted.planner().exchanges().len(), 146);
        for (index, record) in admitted.planner().records().iter().enumerate() {
            assert_eq!(
                admitted.planner().outcome(index),
                expected_outcome(record.subject.point)
            );
        }
        for role in [FundingRole::Source, FundingRole::Merge] {
            for fees in [false, true] {
                assert_eq!(
                    admitted.planner().boundary(role, fees),
                    expected_boundary(role, fees)
                );
            }
        }
        assert_eq!(admitted.packet().policy().relay.satoshis_per_kvb, 100);
        assert_eq!(admitted.packet().policy().mempool.satoshis_per_kvb, 100);
        assert_eq!(admitted.packet().policy().block.satoshis_per_kvb, 100);
        assert_eq!(
            admitted
                .policy_facts()
                .iter()
                .map(|f| f.origin)
                .collect::<Vec<_>>(),
            [
                PolicyFactOrigin::Declared,
                PolicyFactOrigin::Declared,
                PolicyFactOrigin::Observed,
                PolicyFactOrigin::Observed,
                PolicyFactOrigin::Observed,
                PolicyFactOrigin::Declared
            ]
        );
    }

    #[test]
    fn changed_capture_byte_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.capture = archive
            .capture
            .replacen("protocol-schema 8", "protocol-schema 7", 1);
        assert_eq!(archive.admit().err(), Some(FundingAdmissionRefusal::Digest));
    }

    #[test]
    fn changed_binding_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.capture = archive.capture.replacen(
            "handshake-protocol-schema 8",
            "handshake-protocol-schema 7",
            1,
        );
        archive.seal();
        assert_eq!(
            archive.admit().err(),
            Some(FundingAdmissionRefusal::Binding)
        );
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            p["executable_provenance"]["executable_sha256"] = Value::String("00".repeat(32));
        });
        assert_eq!(
            archive.admit().err(),
            Some(FundingAdmissionRefusal::Binding)
        );
        let mut archive = Archive::accepted();
        archive.payload(|p| p["wire_journal"]["descriptor"]["semantic_key"][0] = json!(0));
        assert_eq!(
            archive.admit().err(),
            Some(FundingAdmissionRefusal::Binding)
        );
    }

    #[test]
    fn changed_control_readback_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            let byte = &mut p["wire_journal"]["operations"][2]["response"]["mined_readback"]["raw_transaction"][0];
            *byte = Value::from(byte.as_u64().expect("byte") ^ 1);
        });
        assert_eq!(archive.admit().err(), Some(FundingAdmissionRefusal::Replay));
    }

    #[test]
    fn changed_outcome_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| p["report"]["rows"][2]["outcome"] = json!("RelayMined"));
        assert_eq!(
            archive.admit().err(),
            Some(FundingAdmissionRefusal::Subject)
        );
    }

    #[test]
    fn changed_outer_consensus_layer_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.capture = archive.capture.replacen(
            "response-layer consensus-rejection-before-script",
            "response-layer script-path-rejection",
            1,
        );
        archive.seal();
        assert_eq!(archive.admit().err(), Some(FundingAdmissionRefusal::Replay));
    }

    #[test]
    fn changed_boundary_reading_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| p["report"]["boundaries"][0]["first_accepted"] = json!(11));
        assert_eq!(
            archive.admit().err(),
            Some(FundingAdmissionRefusal::Subject)
        );
    }

    #[test]
    fn missing_wire_response_field_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            p["wire_journal"]["operations"][1]["response"]
                .as_object_mut()
                .expect("response")
                .remove("sponsor_witness");
        });
        assert_eq!(archive.admit().err(), Some(FundingAdmissionRefusal::Replay));
    }

    #[test]
    fn missing_or_changed_digest_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.manifest.clear();
        assert_eq!(archive.admit().err(), Some(FundingAdmissionRefusal::Digest));
        let mut archive = Archive::accepted();
        archive.manifest.replace_range(..1, "0");
        assert_eq!(archive.admit().err(), Some(FundingAdmissionRefusal::Digest));
    }
}
