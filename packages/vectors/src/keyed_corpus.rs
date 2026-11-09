//! Admission of the keyed constructor and continuation archive without executing a node.
//!
//! Complete protocol responses come from the capture's wire JSON journal. The
//! shared birth-corpus reader checks framing, manifest and outcome projections;
//! its archive-specific Debug decoder is never called here. Admission establishes
//! these eight subjects, not ruling 161 acceptance.

use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::NativeOperationResponse;

use crate::keyed_birth::Wave1TargetPacket;
pub use crate::keyed_birth_corpus::BirthAdmissionRefusal as KeyedAdmissionRefusal;
use crate::keyed_birth_corpus::{
    BirthPolicyFact, bind_packet, capture_payload, capture_record, decode_operation_projections,
    hex, manifest_text, policy_facts, require, response_detail,
};
use crate::keyed_native::KeyedSubjectId;
use crate::keyed_native::{
    KeyedCampaignPlanner, KeyedObservationClass, classify_keyed_subject, replay_keyed_capture,
};

const CAPTURE_NAME: &str = "417c1d19b184.report.capture";
const TIMING_NAME: &str = "417c1d19b184.report.timing";
const CAPTURE_TIMING_NAME: &str = "417c1d19b184.report.capture.timing";
const CAPTURE: &str = include_str!("../fixtures/keyed-run-of-record/417c1d19b184.report.capture");
const TIMING: &str = include_str!("../fixtures/keyed-run-of-record/417c1d19b184.report.timing");
const CAPTURE_TIMING: &str =
    include_str!("../fixtures/keyed-run-of-record/417c1d19b184.report.capture.timing");
const REPORT: &str = include_str!("../fixtures/keyed-run-of-record/RUN-REPORT");
const MANIFEST: &str = include_str!("../fixtures/keyed-run-of-record/MANIFEST.sha256");
const PINS: [&str; 5] = [
    "1dbc7b4ac1c223e83b81732f2717a079c62c136dc8815aa139d6675970434a18",
    "026738058372aed97a6b1e4784a89ba21c790b9b046196429a9a37d4c300f434",
    "88fc195b97991d3099513822e690cf5dda7172019bc15f6a61b7c19115efdbb2",
    "b8b756300056a457dd5d957f281c1f2bd47257bd11b1ff0e8232e523c17bbd6e",
    "8fd34e930a654c6ed42ec605faef201da0b61b1a000ba9cca88c7481638494ed",
];
type Result<T> = std::result::Result<T, KeyedAdmissionRefusal>;

pub use crate::keyed_continuation_corpus::ContinuationArchive as KeyedArchive;

/// Rebuilt packet, subject census and replayed observations admitted together.
pub struct AdmittedKeyedCampaign {
    packet: Wave1TargetPacket,
    planner: KeyedCampaignPlanner,
    policy_facts: Vec<BirthPolicyFact>,
}

impl AdmittedKeyedCampaign {
    /// Rebuilt target, deployment, executable, executor and policy binding.
    #[must_use]
    pub const fn packet(&self) -> &Wave1TargetPacket {
        &self.packet
    }

    /// Every replayed request and complete response.
    #[must_use]
    pub const fn planner(&self) -> &KeyedCampaignPlanner {
        &self.planner
    }

    /// All six policy facts, with observed or declared origins.
    #[must_use]
    pub fn policy_facts(&self) -> &[BirthPolicyFact] {
        &self.policy_facts
    }

    /// Coordinate-isolated classification, subject to the shared equality site limit.
    #[must_use]
    pub fn classification(&self, subject: KeyedSubjectId) -> KeyedObservationClass {
        classify_keyed_subject(&self.planner, subject)
    }
}

/// Admit the immutable run after checking all five independent node-side digests.
/// # Errors
/// Refuses a changed digest, binding, journal, projection or subject classification.
pub fn admit_run_of_record() -> Result<AdmittedKeyedCampaign> {
    for (bytes, pin) in [CAPTURE, TIMING, CAPTURE_TIMING, REPORT, MANIFEST]
        .into_iter()
        .zip(PINS)
    {
        require(
            hex(&sha256(bytes.as_bytes())) == pin,
            KeyedAdmissionRefusal::Digest,
        )?;
    }
    admit_keyed_archive(KeyedArchive {
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
pub fn admit_keyed_archive(archive: KeyedArchive<'_>) -> Result<AdmittedKeyedCampaign> {
    let _span = tracing::info_span!("admit_keyed_archive").entered();
    let result = admit(archive);
    tracing::info!(admitted = result.is_ok(), refusal = ?result.as_ref().err(), "keyed admission complete");
    result
}

fn admit(archive: KeyedArchive<'_>) -> Result<AdmittedKeyedCampaign> {
    require(
        archive.manifest == archive_manifest(archive)
            && archive.timing == TIMING
            && archive.capture_timing == CAPTURE_TIMING
            && archive.report == REPORT,
        KeyedAdmissionRefusal::Digest,
    )?;
    let (header, projections, payload) = capture_record(archive.capture, CAPTURE)?;
    require(
        payload["keyed_native_capture_schema"] == 1
            && payload["eligible_for_review"] == true
            && payload["admitted"] == false
            && payload["construction_refusal"] == "None"
            && payload["replay_refusal"] == "None",
        KeyedAdmissionRefusal::Dimension,
    )?;
    let reference = capture_payload(CAPTURE)?;
    let journal = &payload["wire_journal"];
    // The outer header independently fixes deployment, activation and capabilities.
    // Bind the redundant wire handshake/environment to the same archived facts.
    for field in ["handshake", "environment", "descriptors"] {
        require(
            !journal[field].is_null() && journal[field] == reference["wire_journal"][field],
            KeyedAdmissionRefusal::Binding,
        )?;
    }
    let target = reviewed_elements_tapscript().map_err(|_| KeyedAdmissionRefusal::Binding)?;
    let planner =
        replay_keyed_capture(&target, journal).map_err(|_| KeyedAdmissionRefusal::Replay)?;
    let responses: Vec<_> = planner
        .exchanges()
        .iter()
        .map(|(step, response)| (step.case().step.clone(), response.clone()))
        .collect();
    require(
        decode_operation_projections(projections, &responses)? == planner.exchanges(),
        KeyedAdmissionRefusal::Replay,
    )?;
    let reference_response: NativeOperationResponse =
        serde_json::from_value(reference["wire_journal"]["operations"][0]["response"].clone())
            .map_err(|_| KeyedAdmissionRefusal::Dimension)?;
    let reference_policy = response_detail(&reference_response)?;
    let packet = bind_packet(
        header,
        &payload,
        planner.exchanges(),
        &reference,
        &reference_policy["policy"],
    )?;
    let rows = payload["rows"]
        .as_array()
        .ok_or(KeyedAdmissionRefusal::Dimension)?;
    require(rows.len() == 8, KeyedAdmissionRefusal::Dimension)?;
    for (row, record) in rows.iter().zip(planner.records()) {
        let expected = expected_class(record.id);
        require(
            classify_keyed_subject(&planner, record.id) == expected
                && row["record"] == record.capture_json()
                && row["class"] == format!("{expected:?}"),
            KeyedAdmissionRefusal::Subject,
        )?;
    }
    let policy_facts = policy_facts(packet.policy());
    Ok(AdmittedKeyedCampaign {
        packet,
        planner,
        policy_facts,
    })
}

fn archive_manifest(archive: KeyedArchive<'_>) -> String {
    manifest_text(&[
        (CAPTURE_NAME, archive.capture),
        (TIMING_NAME, archive.timing),
        ("RUN-REPORT", archive.report),
        (CAPTURE_TIMING_NAME, archive.capture_timing),
    ])
}

const fn expected_class(id: KeyedSubjectId) -> KeyedObservationClass {
    match id {
        KeyedSubjectId::ControlK
        | KeyedSubjectId::SameKeyControl
        | KeyedSubjectId::CoexistenceL => KeyedObservationClass::MinedControl,
        KeyedSubjectId::WrongControlParity | KeyedSubjectId::UncommittedLeaf => {
            KeyedObservationClass::CommitmentRefusal
        }
        KeyedSubjectId::KeyPathAttempt => KeyedObservationClass::KeyPathSignatureRefusal,
        KeyedSubjectId::WrongKey => KeyedObservationClass::ProgramEqualityRefusal,
        KeyedSubjectId::MixedKeys => KeyedObservationClass::MixedKeyEqualityRefusal,
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
        fn view(&self) -> KeyedArchive<'_> {
            KeyedArchive {
                capture: &self.capture,
                timing: TIMING,
                capture_timing: CAPTURE_TIMING,
                report: REPORT,
                manifest: &self.manifest,
            }
        }
        fn admit(&self) -> Result<AdmittedKeyedCampaign> {
            admit_keyed_archive(self.view())
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
    fn run_of_record_admits_exact_eight_subjects() {
        let admitted = admit_run_of_record().unwrap_or_else(|error| panic!("admission: {error:?}"));
        assert_eq!(admitted.planner().records().len(), 8);
        assert_ne!(
            admitted.planner().recipes().descriptors()[0].output_program,
            admitted.planner().recipes().descriptors()[1].output_program
        );
        for id in KeyedSubjectId::ALL {
            assert_eq!(admitted.classification(id), expected_class(id));
        }
        assert_eq!(admitted.planner().exchanges().len(), 25);
        let heights: Vec<_> = admitted
            .planner()
            .exchanges()
            .iter()
            .filter_map(|(_, r)| r.mined_readback.as_ref().map(|m| m.block_height))
            .collect();
        assert_eq!(heights, [3, 4, 5, 6, 7, 8, 9]);
        let packet = admitted.packet();
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
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Digest));
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
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Binding));
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            p["executable_provenance"]["executable_sha256"] = Value::String("00".repeat(32));
        });
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Binding));
        let mut archive = Archive::accepted();
        archive.payload(|p| p["wire_journal"]["handshake"]["schema"] = Value::from(7));
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Binding));
    }

    #[test]
    fn changed_descriptor_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            let byte = &mut p["wire_journal"]["descriptors"][0]["control"]["bytes"][0];
            *byte = Value::from(byte.as_u64().expect("byte") ^ 1);
        });
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Binding));
    }

    #[test]
    fn changed_control_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            let byte = &mut p["wire_journal"]["operations"][18]["response"]["mined_readback"]["raw_transaction"][0];
            *byte = Value::from(byte.as_u64().expect("byte") ^ 1);
        });
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Subject));
    }

    #[test]
    fn negative_attributed_to_another_class_refuses() {
        let mut archive = Archive::accepted();
        archive.payload(|p| p["rows"][0]["class"] = Value::String("ProgramEqualityRefusal".into()));
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Subject));
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
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Replay));
    }

    #[test]
    fn digest_mismatch_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.manifest.replace_range(..1, "0");
        assert_eq!(archive.admit().err(), Some(KeyedAdmissionRefusal::Digest));
    }
}
