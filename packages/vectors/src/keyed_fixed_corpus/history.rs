//! Rebuild retained funded histories with the unchanged S7 reader and model.

use super::{CaptureFamily, CaptureSubject, CorpusRefusal, Packet};
use elements::hashes::sha256d;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;
use transaction::TargetTransaction;
use vectors::keyed_fixed_capture::workload_shape;
use vectors::keyed_fixed_history_model as model;
use vectors::keyed_fixed_history_reader as reader;

fn address(transaction: &TargetTransaction) -> reader::Address {
    reader::Address(sha256d::Hash::hash(&transaction.encode_without_witness()).to_byte_array())
}

const fn model_triple(triple: reader::Triple) -> model::Triple {
    model::Triple {
        key: triple.key,
        asset: triple.asset,
        value: triple.value,
    }
}

const fn reader_handle(handle: model::Handle) -> reader::Handle {
    reader::Handle {
        triple: reader::Triple {
            key: handle.triple.key,
            asset: handle.triple.asset,
            value: handle.triple.value,
        },
        output: reader::Output {
            event: handle.output.event,
            position: handle.output.position,
        },
    }
}

impl Packet {
    /// Rebuild all six funded history shapes and the omitted-branch reading.
    /// # Errors
    /// Refuses a byte, graph, literal-source or retained summary disagreement.
    pub fn rebuild_histories(&self) -> Result<usize, CorpusRefusal> {
        if self.family != CaptureFamily::Histories {
            return Err(CorpusRefusal::Record);
        }
        let mut count = 0;
        for (events, branching) in [
            (4, false),
            (4, true),
            (16, false),
            (16, true),
            (64, false),
            (64, true),
        ] {
            self.rebuild_shape(events, branching)?;
            count += 1;
        }
        Ok(count)
    }

    #[allow(clippy::too_many_lines)] // One retained graph is compared with both unchanged S7 owners.
    fn rebuild_shape(&self, events: usize, branching: bool) -> Result<(), CorpusRefusal> {
        let started = Instant::now();
        let subject = if branching {
            CaptureSubject::HistoryBranch(u8::try_from(events).map_err(|_| CorpusRefusal::Record)?)
        } else {
            CaptureSubject::HistoryLinear(u8::try_from(events).map_err(|_| CorpusRefusal::Record)?)
        };
        let prefix = format!("{}/{}/subject-", self.run, self.name);
        let mut selected = Vec::new();
        for path in self.files.keys().filter(|path| {
            path.starts_with(&prefix)
                && std::path::Path::new(path).extension() == Some(std::ffi::OsStr::new("json"))
        }) {
            let ordinal = path
                .strip_prefix(&prefix)
                .and_then(|name| name.strip_suffix(".json"))
                .ok_or(CorpusRefusal::Record)?
                .parse::<usize>()
                .map_err(|_| CorpusRefusal::Record)?;
            let record = self.record(ordinal)?;
            if record.subject == subject {
                selected.push(record);
            }
        }
        if selected.len() != events {
            return Err(CorpusRefusal::Census);
        }
        let first = &selected[0];
        let triple = reader::Triple {
            key: first.key,
            asset: first.asset,
            value: first.amount,
        };
        let mut input = reader::Input {
            bytes: BTreeMap::new(),
            association: Vec::new(),
            triples: BTreeSet::from([triple]),
            retrieved: BTreeMap::new(),
            assertions: reader::Assertions::default(),
        };
        let mut known = BTreeMap::new();
        let mut declarations = Vec::new();
        let mut edges = 0usize;
        for (position, record) in selected.iter().enumerate() {
            if (record.key, record.asset, record.amount) != (triple.key, triple.asset, triple.value)
                || record.row.response.accepted_txid.is_none()
                || record.row.readback.as_ref() != Some(&record.row.submitted)
            {
                return Err(CorpusRefusal::Record);
            }
            let id = u32::try_from(position + 1).map_err(|_| CorpusRefusal::Bound)?;
            let transaction = TargetTransaction::decode(&record.row.submitted)
                .map_err(|_| CorpusRefusal::Record)?;
            let event_address = address(&transaction);
            let mut parents = Vec::new();
            for (spent, raw_parent) in transaction.inputs().iter().zip(&record.row.parent_bytes) {
                let parent =
                    TargetTransaction::decode(raw_parent).map_err(|_| CorpusRefusal::Record)?;
                let parent_address = address(&parent);
                if spent.outpoint().txid().internal() != &parent_address.0 {
                    return Err(CorpusRefusal::Record);
                }
                if let Some(&parent_id) = known.get(&parent_address) {
                    edges += 1;
                    let output = model::Output {
                        event: parent_id,
                        position: spent.outpoint().index() as usize,
                    };
                    parents.push(output);
                } else {
                    input.retrieved.insert(parent_address, raw_parent.clone());
                    input.assertions.inclusion.insert(parent_address);
                    input.assertions.retrieval.insert(parent_address);
                }
            }
            if parents.is_empty() {
                edges += 1;
            }
            known.insert(event_address, id);
            input.bytes.insert(id, record.row.submitted.clone());
            input.association.push((id, event_address));
            input.assertions.inclusion.insert(event_address);
            input.assertions.retrieval.insert(event_address);
            let mut outputs = vec![None; transaction.outputs().len()];
            if let Some(first) = outputs.first_mut() {
                *first = Some(model_triple(triple));
            }
            declarations.push(model::Event {
                id,
                inputs: parents,
                outputs,
            });
        }
        let shape = workload_shape(events, branching);
        if shape.events() != events || edges != shape.edges() {
            return Err(CorpusRefusal::Record);
        }
        let mut literal = Vec::<BTreeSet<reader::Handle>>::with_capacity(events);
        for (position, declaration) in declarations.iter().enumerate() {
            let mut actual = declaration
                .inputs
                .iter()
                .map(|output| {
                    usize::try_from(output.event.checked_sub(1).ok_or(CorpusRefusal::Record)?)
                        .map_err(|_| CorpusRefusal::Bound)
                })
                .collect::<Result<Vec<_>, _>>()?;
            actual.sort_unstable();
            let mut expected = shape.parents[position].clone();
            expected.sort_unstable();
            if actual != expected {
                return Err(CorpusRefusal::Record);
            }
            let id = u32::try_from(position + 1).map_err(|_| CorpusRefusal::Bound)?;
            let handle = reader::Handle {
                triple,
                output: reader::Output {
                    event: id,
                    position: 0,
                },
            };
            let mut source_set = BTreeSet::new();
            for parent in expected {
                source_set.extend(
                    literal
                        .get(parent)
                        .ok_or(CorpusRefusal::Record)?
                        .iter()
                        .copied(),
                );
            }
            if source_set.is_empty() {
                source_set.insert(handle);
            }
            literal.push(source_set);
        }
        let reading = reader::read(&input).map_err(|_| CorpusRefusal::Record)?;
        let graph = model::analyze(&declarations).map_err(|_| CorpusRefusal::Record)?;
        for (position, set) in literal.iter().enumerate() {
            let id = u32::try_from(position + 1).map_err(|_| CorpusRefusal::Bound)?;
            let handle = reader::Handle {
                triple,
                output: reader::Output {
                    event: id,
                    position: 0,
                },
            };
            if reading.query(handle)
                != (reader::Query::Handle {
                    handle,
                    membership: reader::Membership::Complete(set.clone()),
                })
            {
                return Err(CorpusRefusal::Record);
            }
            let oracle = model::Handle {
                triple: model_triple(triple),
                output: model::Output {
                    event: id,
                    position: 0,
                },
            };
            let model_set = graph.sets().get(&oracle).ok_or(CorpusRefusal::Record)?;
            if model_set
                .iter()
                .copied()
                .map(reader_handle)
                .collect::<BTreeSet<_>>()
                != *set
            {
                return Err(CorpusRefusal::Record);
            }
        }
        let filename = format!(
            "history-{}-{events}.json",
            if branching { "branch" } else { "linear" }
        );
        let summary = self.family_json(&filename)?;
        if summary["events"] != events
            || summary["edges"] != edges
            || summary["unique_bytes"] != reading.event_bytes()
            || summary["records"] != reading.memberships().len()
            || summary["cold_read_micros"].as_u64().is_none()
            || summary["warm_read_micros"]
                .as_array()
                .is_none_or(|samples| {
                    samples.len() != 5 || samples.iter().any(|sample| sample.as_u64().is_none())
                })
        {
            return Err(CorpusRefusal::Measurement);
        }
        if branching && events == 4 {
            input.bytes.remove(&2);
            let missing = reader::read(&input).map_err(|_| CorpusRefusal::Record)?;
            if !missing
                .memberships()
                .values()
                .any(|membership| matches!(membership, reader::Membership::Incomplete(_)))
            {
                return Err(CorpusRefusal::Record);
            }
            let marker = self.family_json("history-missing-branch.json")?;
            if marker["subject"] != "history_missing_branch"
                || marker["boundary"] != "incomplete_history"
            {
                return Err(CorpusRefusal::Record);
            }
        }
        tracing::info!(target: "vectors::keyed_fixed_corpus", events, branching, edges, unique_bytes = reading.event_bytes(), wall_ms = started.elapsed().as_millis(), outcome = "completed", "funded history rebuilt");
        Ok(())
    }
}
