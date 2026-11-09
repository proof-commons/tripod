//! Admission of measurement (d), with relay policy distinct from consensus limits.
//!
//! Complete responses are decoded from wire JSON. The retained block-script
//! diagnostic is not promoted to a consensus boundary: the calls list is empty,
//! while the explicit relay reason is witness nonstandardness. Original archive
//! bytes and their unadmitted capture-time status remain unchanged.

use crate::keyed_birth::Wave1TargetPacket;
pub use crate::keyed_birth_corpus::BirthAdmissionRefusal as CostAdmissionRefusal;
use crate::keyed_birth_corpus::{
    BirthPolicyFact, bind_packet, capture_payload, capture_record, decode_operation_projections,
    hex, manifest_text, policy_facts, require, response_detail,
};
use crate::keyed_cost::{KeyedCostPacket, carrier_cost, cost_packet};
use crate::keyed_cost_native::{
    CostCampaignPlanner, CostSubjectId, cost_control_mined, oversized_policy_refused,
    oversized_resource_refused, replay_cost_capture,
};
use serde_json::{Value, json};
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::{NativeOperationResponse, ObservedOutcomeLayer};

const CAPTURE_NAME: &str = "da02067f499d.report.capture";
const TIMING_NAME: &str = "da02067f499d.report.timing";
const CAPTURE_TIMING_NAME: &str = "da02067f499d.report.capture.timing";
const CAPTURE: &str =
    include_str!("../fixtures/keyed-cost-run-of-record/da02067f499d.report.capture");
const TIMING: &str =
    include_str!("../fixtures/keyed-cost-run-of-record/da02067f499d.report.timing");
const CAPTURE_TIMING: &str =
    include_str!("../fixtures/keyed-cost-run-of-record/da02067f499d.report.capture.timing");
const REPORT: &str = include_str!("../fixtures/keyed-cost-run-of-record/RUN-REPORT");
const MANIFEST: &str = include_str!("../fixtures/keyed-cost-run-of-record/MANIFEST.sha256");
const PINS: [&str; 5] = [
    "ee10c4a011e5c22136d3477d72f2967b012bf5b2f2b9944c80badf4176b90f9e",
    "954ce6bbde5c04039c7b608948a4dfbdc3d50b5d720b2d66099eea1c58530e30",
    "88fc195b97991d3099513822e690cf5dda7172019bc15f6a61b7c19115efdbb2",
    "216d4d9e911df38a51cd8dd387aa4f9384b1c33afc05993df55e888f088c7444",
    "49e2ee9097acbc031900c83a0f01c1bbbfb9108a6fac710c1cb1858f7ef8d917",
];
type Result<T> = std::result::Result<T, CostAdmissionRefusal>;

pub use crate::keyed_continuation_corpus::ContinuationArchive as CostArchive;

/// Exact admitted classes; neither is a consensus push-size claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostAdmissionClass {
    /// Exact submitted transaction mined and read back.
    MinedControl,
    /// Explicit relay witness-standardness refusal beside its mined control.
    WitnessStandardnessRefusal,
}

/// Bound focused campaign and the completed comparison packet.
pub struct AdmittedCostCampaign {
    packet: Wave1TargetPacket,
    planner: CostCampaignPlanner,
    policy_facts: Vec<BirthPolicyFact>,
    comparison: KeyedCostPacket,
}
impl AdmittedCostCampaign {
    /// Reviewed target, deployment, executable, executor and policy binding.
    #[must_use]
    pub const fn packet(&self) -> &Wave1TargetPacket {
        &self.packet
    }
    /// Complete replayed journal, including parents and signing echoes.
    #[must_use]
    pub const fn planner(&self) -> &CostCampaignPlanner {
        &self.planner
    }
    /// Policy facts retain their observed or declared origins.
    #[must_use]
    pub fn policy_facts(&self) -> &[BirthPolicyFact] {
        &self.policy_facts
    }
    /// Earlier dimensions plus the three admitted focused carriers and binding.
    #[must_use]
    pub const fn comparison(&self) -> &KeyedCostPacket {
        &self.comparison
    }
    /// Every admitted subject has its expected class; no open subject is admitted.
    #[must_use]
    pub const fn classification(&self, id: CostSubjectId) -> CostAdmissionClass {
        match id {
            CostSubjectId::OversizedWitness => CostAdmissionClass::WitnessStandardnessRefusal,
            CostSubjectId::IsolatedContinuation | CostSubjectId::ResourceControl => {
                CostAdmissionClass::MinedControl
            }
        }
    }
}

/// Admit the immutable run after checking all five independent node-side digests.
/// # Errors
/// Refuses a changed digest, binding, journal, projection or subject classification.
pub fn admit_run_of_record() -> Result<AdmittedCostCampaign> {
    for (bytes, pin) in [CAPTURE, TIMING, CAPTURE_TIMING, REPORT, MANIFEST]
        .into_iter()
        .zip(PINS)
    {
        require(
            hex(&sha256(bytes.as_bytes())) == pin,
            CostAdmissionRefusal::Digest,
        )?;
    }
    admit_cost_archive(CostArchive {
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
pub fn admit_cost_archive(archive: CostArchive<'_>) -> Result<AdmittedCostCampaign> {
    let _span = tracing::info_span!("admit_cost_archive").entered();
    let result = admit(archive);
    tracing::info!(admitted = result.is_ok(), refusal = ?result.as_ref().err(), "cost admission complete");
    result
}

fn admit(archive: CostArchive<'_>) -> Result<AdmittedCostCampaign> {
    require(
        archive.manifest == archive_manifest(archive)
            && archive.timing == TIMING
            && archive.capture_timing == CAPTURE_TIMING
            && archive.report == REPORT,
        CostAdmissionRefusal::Digest,
    )?;
    let (header, projections, payload) = capture_record(archive.capture, CAPTURE)?;
    require(
        payload["keyed_cost_capture_schema"] == 1
            && payload["eligible_for_review"] == true
            && payload["admitted"] == false
            && payload["construction_refusal"] == "None"
            && payload["replay_refusal"] == "None",
        CostAdmissionRefusal::Dimension,
    )?;
    let reference = capture_payload(CAPTURE)?;
    let journal = &payload["wire_journal"];
    for field in ["handshake", "environment"] {
        require(
            !journal[field].is_null() && journal[field] == reference["wire_journal"][field],
            CostAdmissionRefusal::Binding,
        )?;
    }
    let target = reviewed_elements_tapscript().map_err(|_| CostAdmissionRefusal::Binding)?;
    let planner =
        replay_cost_capture(&target, journal).map_err(|_| CostAdmissionRefusal::Replay)?;
    let responses: Vec<_> = planner
        .exchanges()
        .iter()
        .map(|(step, response)| (step.case().step.clone(), response.clone()))
        .collect();
    require(
        decode_operation_projections(projections, &responses)? == planner.exchanges(),
        CostAdmissionRefusal::Replay,
    )?;
    let reference_response: NativeOperationResponse =
        serde_json::from_value(reference["wire_journal"]["operations"][0]["response"].clone())
            .map_err(|_| CostAdmissionRefusal::Dimension)?;
    let reference_policy = response_detail(&reference_response)?;
    let packet = bind_packet(
        header,
        &payload,
        planner.exchanges(),
        &reference,
        &reference_policy["policy"],
    )?;
    validate_subjects(&planner, &payload)?;
    let mut comparison = cost_packet().map_err(|_| CostAdmissionRefusal::Binding)?;
    comparison.bindings.push(packet.clone());
    for id in SUBJECTS {
        let (step, response) = planner.subject(id).ok_or(CostAdmissionRefusal::Dimension)?;
        comparison.carriers.push(
            carrier_cost("keyed-cost-run-of-record", step, response)
                .map_err(|_| CostAdmissionRefusal::Dimension)?,
        );
    }
    comparison
        .absent
        .retain(|s| s != "isolated continuation node-read transaction weight (staged)");
    comparison
        .absent
        .push("consensus push-size boundary".into());
    let policy_facts = policy_facts(packet.policy());
    Ok(AdmittedCostCampaign {
        packet,
        planner,
        policy_facts,
        comparison,
    })
}

const SUBJECTS: [CostSubjectId; 3] = [
    CostSubjectId::IsolatedContinuation,
    CostSubjectId::OversizedWitness,
    CostSubjectId::ResourceControl,
];

fn validate_subjects(planner: &CostCampaignPlanner, payload: &Value) -> Result<()> {
    require(
        cost_control_mined(planner, CostSubjectId::IsolatedContinuation)
            && cost_control_mined(planner, CostSubjectId::ResourceControl)
            && oversized_policy_refused(planner)
            && !oversized_resource_refused(planner),
        CostAdmissionRefusal::Subject,
    )?;
    let rows = payload["rows"]
        .as_array()
        .ok_or(CostAdmissionRefusal::Dimension)?;
    require(
        rows.len() == SUBJECTS.len(),
        CostAdmissionRefusal::Dimension,
    )?;
    for (row, id) in rows.iter().zip(SUBJECTS) {
        let (step, response) = planner.subject(id).ok_or(CostAdmissionRefusal::Dimension)?;
        let carrier = carrier_cost("focused-cost-capture", step, response)
            .map_err(|_| CostAdmissionRefusal::Dimension)?;
        require(
            response.resources.transaction_weight.is_some(),
            CostAdmissionRefusal::Dimension,
        )?;
        let expected = json!({"subject":format!("{id:?}"), "request":step.subject(), "response":response, "carrier":carrier, "mined_control":cost_control_mined(planner,id), "size_refusal":false, "witness_policy_refusal":id == CostSubjectId::OversizedWitness});
        require(*row == expected, CostAdmissionRefusal::Subject)?;
        if id == CostSubjectId::OversizedWitness {
            let detail = response_detail(response)?;
            require(
                response.observed_layer == ObservedOutcomeLayer::ScriptPathRejection
                    && detail["detail"]["calls"] == json!([]),
                CostAdmissionRefusal::Subject,
            )?;
            // Preserve the diagnostic, but do not infer an executed consensus path
            // from it or from the absence of call records.
            require(
                detail["detail"]["block_script_error"] == "Push value size limit exceeded"
                    && detail["detail"]["verdict_reason"] == "mandatory-script-verify-flag-failed",
                CostAdmissionRefusal::Subject,
            )?;
        }
    }
    Ok(())
}

fn archive_manifest(archive: CostArchive<'_>) -> String {
    manifest_text(&[
        (CAPTURE_NAME, archive.capture),
        (TIMING_NAME, archive.timing),
        ("RUN-REPORT", archive.report),
        (CAPTURE_TIMING_NAME, archive.capture_timing),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyed_birth::PolicyFactOrigin;
    use crate::keyed_birth_corpus::single_field;

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
        fn view(&self) -> CostArchive<'_> {
            CostArchive {
                capture: &self.capture,
                timing: TIMING,
                capture_timing: CAPTURE_TIMING,
                report: REPORT,
                manifest: &self.manifest,
            }
        }
        fn admit(&self) -> Result<AdmittedCostCampaign> {
            admit_cost_archive(self.view())
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
    fn run_of_record_admits_policy_and_two_mined_controls() {
        let admitted = admit_run_of_record().expect("cost admission");
        assert_eq!(admitted.planner().exchanges().len(), 12);
        assert_eq!(
            admitted.classification(CostSubjectId::OversizedWitness),
            CostAdmissionClass::WitnessStandardnessRefusal
        );
        for id in [
            CostSubjectId::IsolatedContinuation,
            CostSubjectId::ResourceControl,
        ] {
            assert_eq!(
                admitted.classification(id),
                CostAdmissionClass::MinedControl
            );
        }
        let heights: Vec<_> = admitted
            .planner()
            .exchanges()
            .iter()
            .filter_map(|(_, r)| r.mined_readback.as_ref().map(|m| m.block_height))
            .collect();
        assert_eq!(heights, [3, 4, 6, 7]);
        let comparison = admitted.comparison();
        assert_eq!(comparison.bindings.len(), 4);
        assert_eq!(comparison.carriers.len(), 19);
        assert_eq!(
            comparison.carriers[16..]
                .iter()
                .map(|r| r.node_weight)
                .collect::<Vec<_>>(),
            [Some(1288), Some(1806), Some(1284)]
        );
        assert!(
            comparison
                .carriers
                .iter()
                .all(|r| r.weight_matches == Some(true))
        );
        assert!(
            !comparison
                .absent
                .iter()
                .any(|s| s.contains("isolated continuation"))
        );
        assert!(
            comparison
                .absent
                .iter()
                .any(|s| s == "consensus push-size boundary")
        );
        assert_eq!(admitted.packet().executor().protocol_schema, 8);
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
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Digest));
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
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Binding));
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            p["executable_provenance"]["executable_sha256"] = Value::String("00".repeat(32));
        });
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Binding));
        let mut archive = Archive::accepted();
        archive.payload(|p| p["wire_journal"]["handshake"]["schema"] = Value::from(7));
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Binding));
    }

    #[test]
    fn changed_control_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            let byte = &mut p["wire_journal"]["operations"][4]["response"]["mined_readback"]["raw_transaction"][0];
            *byte = Value::from(byte.as_u64().expect("byte") ^ 1);
        });
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Subject));
    }

    #[test]
    fn negative_attributed_to_another_class_refuses() {
        let mut archive = Archive::accepted();
        archive.payload(|p| p["rows"][1]["size_refusal"] = Value::Bool(true));
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Subject));
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
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Replay));
    }

    #[test]
    fn digest_mismatch_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.manifest.replace_range(..1, "0");
        assert_eq!(archive.admit().err(), Some(CostAdmissionRefusal::Digest));
    }
}
