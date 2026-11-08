//! Admission of the key-free continuation archive without executing a node.
//!
//! Complete protocol responses come from the capture's wire JSON journal. The
//! shared birth-corpus reader checks framing, manifest and outcome projections;
//! its archive-specific Debug decoder is never called here. Admission establishes
//! these four subjects, not key-dependent measurements or ruling 161 acceptance.

use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::NativeOperationResponse;

pub use crate::keyed_birth_corpus::BirthAdmissionRefusal as ContinuationAdmissionRefusal;
use crate::keyed_birth_corpus::{
    BirthPolicyFact, bind_packet, capture_payload, capture_record, decode_operation_projections,
    hex, manifest_text, policy_facts, require, response_detail,
};
use crate::keyed_continuation::{ContinuationCampaignManifest, ContinuationSubjectId};
use crate::keyed_continuation_native::{
    ContinuationCampaignPlanner, ContinuationObservationClass, classify_continuation_subject,
    replay_continuation_capture,
};

const CAPTURE_NAME: &str = "0b54dac7e4b8.report.capture";
const TIMING_NAME: &str = "0b54dac7e4b8.report.timing";
const CAPTURE_TIMING_NAME: &str = "0b54dac7e4b8.report.capture.timing";
const CAPTURE: &str =
    include_str!("../fixtures/keyed-continuation-run-of-record/0b54dac7e4b8.report.capture");
const TIMING: &str =
    include_str!("../fixtures/keyed-continuation-run-of-record/0b54dac7e4b8.report.timing");
const CAPTURE_TIMING: &str =
    include_str!("../fixtures/keyed-continuation-run-of-record/0b54dac7e4b8.report.capture.timing");
const REPORT: &str = include_str!("../fixtures/keyed-continuation-run-of-record/RUN-REPORT");
const MANIFEST: &str = include_str!("../fixtures/keyed-continuation-run-of-record/MANIFEST.sha256");
const PINS: [&str; 5] = [
    "11bb8cab35d335c6b6114ccacb1696a7eead807b8c95a251ef0dcf264680f082",
    "006fc51855192829073ffcab51bbca8ba692e898cf4ede99b5d992523a851ff9",
    "88fc195b97991d3099513822e690cf5dda7172019bc15f6a61b7c19115efdbb2",
    "35f505c909d7cfcb66062c95f3597fd9b82b571ff23021f9d554c6e4fe872280",
    "53cf0c361765fb0b9c79f21174283c54d5b8e061390ecfeac669ab5f8a512f93",
];
type Result<T> = std::result::Result<T, ContinuationAdmissionRefusal>;

/// Five retained files, read without changing their capture-stage standing.
#[derive(Clone, Copy)]
pub struct ContinuationArchive<'a> {
    /// Schema-two capture containing the complete wire journal.
    pub capture: &'a str,
    /// Outer timing.
    pub timing: &'a str,
    /// Ceremony timing sidecar.
    pub capture_timing: &'a str,
    /// Summary report; never a source of executor responses.
    pub report: &'a str,
    /// Original four-entry manifest.
    pub manifest: &'a str,
}

/// Rebuilt packet, subject census and replayed observations admitted together.
pub struct AdmittedContinuationCampaign {
    manifest: ContinuationCampaignManifest,
    planner: ContinuationCampaignPlanner,
    policy_facts: Vec<BirthPolicyFact>,
}

impl AdmittedContinuationCampaign {
    /// Target packet and the four key-free subjects.
    #[must_use]
    pub const fn manifest(&self) -> &ContinuationCampaignManifest {
        &self.manifest
    }

    /// Every replayed request and complete response.
    #[must_use]
    pub const fn planner(&self) -> &ContinuationCampaignPlanner {
        &self.planner
    }

    /// All six policy facts, with observed or declared origins.
    #[must_use]
    pub fn policy_facts(&self) -> &[BirthPolicyFact] {
        &self.policy_facts
    }

    /// Coordinate-isolated classification, subject to the shared equality site limit.
    #[must_use]
    pub fn classification(&self, subject: ContinuationSubjectId) -> ContinuationObservationClass {
        classify_continuation_subject(&self.planner, subject)
    }
}

/// Admit the immutable run after checking all five independent node-side digests.
/// # Errors
/// Refuses a changed digest, binding, journal, projection or subject classification.
pub fn admit_run_of_record() -> Result<AdmittedContinuationCampaign> {
    for (bytes, pin) in [CAPTURE, TIMING, CAPTURE_TIMING, REPORT, MANIFEST]
        .into_iter()
        .zip(PINS)
    {
        require(
            hex(&sha256(bytes.as_bytes())) == pin,
            ContinuationAdmissionRefusal::Digest,
        )?;
    }
    admit_continuation_archive(ContinuationArchive {
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
pub fn admit_continuation_archive(
    archive: ContinuationArchive<'_>,
) -> Result<AdmittedContinuationCampaign> {
    let _span = tracing::info_span!("admit_keyfree_continuation_archive").entered();
    let result = admit(archive);
    tracing::info!(admitted = result.is_ok(), refusal = ?result.as_ref().err(), "continuation admission complete");
    result
}

fn admit(archive: ContinuationArchive<'_>) -> Result<AdmittedContinuationCampaign> {
    require(
        archive.manifest == archive_manifest(archive)
            && archive.timing == TIMING
            && archive.capture_timing == CAPTURE_TIMING
            && archive.report == REPORT,
        ContinuationAdmissionRefusal::Digest,
    )?;
    let (header, projections, payload) = capture_record(archive.capture, CAPTURE)?;
    require(
        payload["keyed_continuation_capture_schema"] == 1
            && payload["eligible_for_review"] == true
            && payload["admitted"] == false
            && payload["construction_refusal"] == "None"
            && payload["replay_refusal"] == "None",
        ContinuationAdmissionRefusal::Dimension,
    )?;
    let reference = capture_payload(CAPTURE)?;
    let journal = &payload["wire_journal"];
    // The outer header independently fixes deployment, activation and capabilities.
    // Bind the redundant wire handshake/environment to the same archived facts.
    for field in ["handshake", "environment"] {
        require(
            !journal[field].is_null() && journal[field] == reference["wire_journal"][field],
            ContinuationAdmissionRefusal::Binding,
        )?;
    }
    let target =
        reviewed_elements_tapscript().map_err(|_| ContinuationAdmissionRefusal::Binding)?;
    let planner = replay_continuation_capture(&target, journal)
        .map_err(|_| ContinuationAdmissionRefusal::Replay)?;
    let responses: Vec<_> = planner
        .exchanges()
        .iter()
        .map(|(step, response)| (step.case().step.clone(), response.clone()))
        .collect();
    require(
        decode_operation_projections(projections, &responses)? == planner.exchanges(),
        ContinuationAdmissionRefusal::Replay,
    )?;
    let reference_response: NativeOperationResponse =
        serde_json::from_value(reference["wire_journal"]["operations"][0]["response"].clone())
            .map_err(|_| ContinuationAdmissionRefusal::Dimension)?;
    let reference_policy = response_detail(&reference_response)?;
    let packet = bind_packet(
        header,
        &payload,
        planner.exchanges(),
        &reference,
        &reference_policy["policy"],
    )?;
    let manifest = planner
        .manifest(packet)
        .map_err(|_| ContinuationAdmissionRefusal::Binding)?;
    let rows = payload["rows"]
        .as_array()
        .ok_or(ContinuationAdmissionRefusal::Dimension)?;
    require(rows.len() == 4, ContinuationAdmissionRefusal::Dimension)?;
    for (row, record) in rows.iter().zip(planner.records()) {
        let expected = expected_class(record.subject.id);
        require(
            classify_continuation_subject(&planner, record.subject.id) == expected
                && row["record"] == record.capture_json()
                && row["class"] == format!("{expected:?}"),
            ContinuationAdmissionRefusal::Subject,
        )?;
    }
    let policy_facts = policy_facts(manifest.packet().policy());
    Ok(AdmittedContinuationCampaign {
        manifest,
        planner,
        policy_facts,
    })
}

fn archive_manifest(archive: ContinuationArchive<'_>) -> String {
    manifest_text(&[
        (CAPTURE_NAME, archive.capture),
        (TIMING_NAME, archive.timing),
        ("RUN-REPORT", archive.report),
        (CAPTURE_TIMING_NAME, archive.capture_timing),
    ])
}

const fn expected_class(id: ContinuationSubjectId) -> ContinuationObservationClass {
    match id {
        ContinuationSubjectId::Control => ContinuationObservationClass::MinedControl,
        ContinuationSubjectId::WrongVersion => ContinuationObservationClass::VersionEqualityRefusal,
        ContinuationSubjectId::WrongProgram | ContinuationSubjectId::DisplacedSuccessor => {
            ContinuationObservationClass::ProgramEqualityRefusal
        }
    }
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
        fn view(&self) -> ContinuationArchive<'_> {
            ContinuationArchive {
                capture: &self.capture,
                timing: TIMING,
                capture_timing: CAPTURE_TIMING,
                report: REPORT,
                manifest: &self.manifest,
            }
        }
        fn admit(&self) -> Result<AdmittedContinuationCampaign> {
            admit_continuation_archive(self.view())
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
    fn run_of_record_admits_exact_four_subjects() {
        let admitted = admit_run_of_record().unwrap_or_else(|error| panic!("admission: {error:?}"));
        assert_eq!(admitted.manifest().subjects().len(), 4);
        for id in ContinuationSubjectId::ALL {
            assert_eq!(admitted.classification(id), expected_class(id));
        }
        assert_eq!(admitted.planner().exchanges().len(), 11);
        let heights: Vec<_> = admitted
            .planner()
            .exchanges()
            .iter()
            .filter_map(|(_, r)| r.mined_readback.as_ref().map(|m| m.block_height))
            .collect();
        assert_eq!(heights, [3, 4]);
        let packet = admitted.manifest().packet();
        assert_eq!(packet.executor().protocol_schema, 8);
        assert_eq!(packet.executor().adapter_version, "2.1.0");
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
        assert_eq!(packet.policy().relay.satoshis_per_kvb, 100);
    }

    #[test]
    fn changed_capture_byte_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.capture = archive
            .capture
            .replacen("protocol-schema 8", "protocol-schema 7", 1);
        assert_eq!(
            archive.admit().err(),
            Some(ContinuationAdmissionRefusal::Digest)
        );
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
            Some(ContinuationAdmissionRefusal::Binding)
        );
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            p["executable_provenance"]["executable_sha256"] = Value::String("00".repeat(32));
        });
        assert_eq!(
            archive.admit().err(),
            Some(ContinuationAdmissionRefusal::Binding)
        );
        let mut archive = Archive::accepted();
        archive.payload(|p| p["wire_journal"]["handshake"]["schema"] = Value::from(7));
        assert_eq!(
            archive.admit().err(),
            Some(ContinuationAdmissionRefusal::Binding)
        );
    }

    #[test]
    fn changed_control_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            let byte = &mut p["wire_journal"]["operations"][10]["response"]["mined_readback"]["raw_transaction"][0];
            *byte = Value::from(byte.as_u64().expect("byte") ^ 1);
        });
        assert_eq!(
            archive.admit().err(),
            Some(ContinuationAdmissionRefusal::Subject)
        );
    }

    #[test]
    fn negative_attributed_to_another_class_refuses() {
        let mut archive = Archive::accepted();
        archive.payload(|p| p["rows"][0]["class"] = Value::String("VersionEqualityRefusal".into()));
        assert_eq!(
            archive.admit().err(),
            Some(ContinuationAdmissionRefusal::Subject)
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
        assert_eq!(
            archive.admit().err(),
            Some(ContinuationAdmissionRefusal::Replay)
        );
    }

    #[test]
    fn digest_mismatch_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.manifest.replace_range(..1, "0");
        assert_eq!(
            archive.admit().err(),
            Some(ContinuationAdmissionRefusal::Digest)
        );
    }
}
