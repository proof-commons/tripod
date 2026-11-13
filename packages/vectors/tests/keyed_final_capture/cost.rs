//! Exact-byte resource comparison. Model counters are never native measurements.
use super::retained::SessionArchive;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tapscript::keyed_resources::{KeyedSchedule, keyed_resource_projection};
use target_elements_conformance::protocol::{ObservedOutcomeLayer, OperationSubject};
use vectors::keyed_history_reader as r;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One byte-bound clause or whole-leaf model observation and grammar prediction.
pub struct MemberCost {
    pub subject: [u8; 32],
    pub input: usize,
    pub phase: String,
    pub primitive_unit: String,
    pub predicted_primitives: u64,
    pub model_primitives: u64,
    pub model_peak_items: usize,
    pub model_maximum_element_bytes: usize,
    pub native_primitives: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Direct complete-byte size and independently compared transaction weights.
pub struct SubjectCost {
    pub digest: [u8; 32],
    pub direct_complete_bytes: usize,
    pub local_weight_units: u64,
    pub elements_weight_units: u64,
    pub native_weight_units: Option<u64>,
    pub script_bytes_per_member: usize,
    pub control_bytes_per_member: usize,
    pub execution_argument_items: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Available resource comparisons, with uninstrumented native dimensions absent.
pub struct Costs {
    pub members: Vec<MemberCost>,
    pub subjects: Vec<SubjectCost>,
    pub native_clause_wall_seconds: Option<u64>,
    pub native_clause_cpu_seconds: Option<u64>,
    pub native_clause_stalls_seconds: Option<u64>,
    pub native_validation_charge: Option<u64>,
}
#[allow(
    clippy::too_many_lines,
    reason = "One indexed byte pass keeps the three independent resource sources adjacent."
)]
/// Derive comparisons from retained bytes using the reviewed reader and projector.
/// # Panics
/// Panics if a supplied control is unreadable, its history refuses or its resource comparisons disagree.
pub fn derive(sessions: &[SessionArchive]) -> Costs {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let prediction = [
        KeyedSchedule::Birth,
        KeyedSchedule::Continuation,
        KeyedSchedule::Leaf,
    ]
    .map(|s| keyed_resource_projection(&target, s).unwrap());
    let tapscript::keyed_resources::KeyedResourceTerm::Exact(script_bytes) =
        prediction[2].dimensions[&target_elements::ResourceDimension::ScriptBytes]
    else {
        panic!("reviewed leaf has an exact script size");
    };
    assert_eq!(script_bytes, 52);
    let mut result = Costs {
        members: vec![],
        subjects: vec![],
        native_clause_wall_seconds: None,
        native_clause_cpu_seconds: None,
        native_clause_stalls_seconds: None,
        native_validation_charge: None,
    };
    for session in sessions {
        let commitment = transaction::keyed_taproot::commit_keyed_bundle(
            &target,
            &crate::keyed_reproduction::fixture::linked(session.key, false),
        )
        .unwrap();
        let control_bytes = commitment.control().len();
        assert_eq!(control_bytes, 33);
        let mut input = r::Input {
            bytes: BTreeMap::new(),
            association: vec![],
            keys: [r::Key(session.key)].into(),
            retrieved: BTreeMap::new(),
            boundary_premises: BTreeSet::new(),
            assertions: r::Assertions {
                inclusion: BTreeSet::new(),
                retrieval: BTreeSet::new(),
            },
        };
        let mut subjects = BTreeMap::new();
        for (request, response) in &session.exchanges {
            if response.observed_layer != ObservedOutcomeLayer::Accepted {
                continue;
            }
            let Some(readback) = &response.mined_readback else {
                continue;
            };
            let address = r::Address(crate::keyed_reproduction::fixture::txid(
                &readback.raw_transaction,
            ));
            input.assertions.inclusion.insert(address);
            input.assertions.retrieval.insert(address);
            match &request.subject {
                OperationSubject::SponsorFunding(_) => {
                    input
                        .retrieved
                        .insert(address, readback.raw_transaction.clone());
                }
                OperationSubject::Submission(submitted) => {
                    let event = u32::try_from(input.bytes.len()).unwrap();
                    let raw = &submitted.transaction_bytes;
                    let tx = transaction::TargetTransaction::decode(raw).unwrap();
                    let external: elements::Transaction =
                        elements::encode::deserialize(raw).unwrap();
                    assert_eq!(tx.weight(), u64::try_from(external.weight()).unwrap());
                    if let Some(weight) = response.resources.transaction_weight {
                        assert_eq!(weight, tx.weight());
                    }
                    let digest = target_elements_conformance::constructor::tagged::sha256(raw);
                    let witnesses: Vec<_> = tx
                        .witnesses()
                        .iter()
                        .filter(|w| {
                            w.stack().first().is_some_and(|script| {
                                script.len() == usize::try_from(script_bytes).unwrap()
                            })
                        })
                        .collect();
                    for witness in &witnesses {
                        assert_eq!(witness.stack().len(), 2);
                        assert_eq!(witness.stack()[1].len(), control_bytes);
                    }
                    result.subjects.push(SubjectCost {
                        digest,
                        direct_complete_bytes: raw.len(),
                        local_weight_units: tx.weight(),
                        elements_weight_units: u64::try_from(external.weight()).unwrap(),
                        native_weight_units: response.resources.transaction_weight,
                        script_bytes_per_member: usize::try_from(script_bytes).unwrap(),
                        control_bytes_per_member: control_bytes,
                        execution_argument_items: 0,
                    });
                    subjects.insert(event, digest);
                    input.bytes.insert(event, raw.clone());
                    input.association.push((event, address));
                }
                _ => {}
            }
        }
        // Assertions state only the retained readback premise. The reader still
        // carries inclusion, retrieval, unspentness and finality as obligations.
        let reading = r::read(&input).unwrap();
        for row in reading.relations() {
            let (index, phase) = match row.relation {
                r::Relation::BornFirst => (0, "birth"),
                r::Relation::ContinueFirst => (1, "continuation"),
            };
            assert_eq!(row.primitive_count, prediction[index].primitive_count);
            result.members.push(MemberCost {
                subject: subjects[&row.event],
                input: row.input,
                phase: phase.into(),
                primitive_unit: "primitive calls".into(),
                predicted_primitives: prediction[index].primitive_count,
                model_primitives: row.primitive_count,
                model_peak_items: row.peak,
                model_maximum_element_bytes: row.maximum_width,
                native_primitives: None,
            });
        }
        for row in reading.leaves() {
            assert_eq!(row.primitive_count, prediction[2].primitive_count);
            result.members.push(MemberCost {
                subject: subjects[&row.event],
                input: row.input,
                phase: "composition".into(),
                primitive_unit: "primitive calls".into(),
                predicted_primitives: prediction[2].primitive_count,
                model_primitives: row.primitive_count,
                model_peak_items: row.peak,
                model_maximum_element_bytes: row.maximum_width,
                native_primitives: None,
            });
        }
    }
    result
}
