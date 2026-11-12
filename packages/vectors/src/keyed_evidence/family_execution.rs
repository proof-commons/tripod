//! Replay consistency between indexes, not independent semantic evidence.
//! The fact builder is retained separately: sharing it needs the reader's private Index,
//! not visibility alone. The graph/literal harness is the semantic second source.
use super::{FamilyCondition, FamilyContext, Schedule};
use crate::keyed_history_reader::{Address, EventId, Relation, RelationEvidence};
use crate::keyed_stack_core::{self as model, Facts, InputFact, ProgramFact, Trace};
use target_elements_conformance::constructor::tagged::sha256;

fn program(bytes: &[u8]) -> ProgramFact {
    if let [version, length, payload @ ..] = bytes
        && usize::from(*length) == payload.len()
        && (2..=40).contains(&payload.len())
        && (*version == 0 || (0x51..=0x60).contains(version))
    {
        return ProgramFact::Witness {
            version: if *version == 0 { 0 } else { version - 0x50 },
            payload: payload.to_vec(),
        };
    }
    ProgramFact::Other {
        sha256: sha256(bytes),
    }
}
pub(super) fn context(c: &FamilyContext, event: EventId) -> Result<Facts, FamilyCondition> {
    let tx = c
        .decoded
        .get(&event)
        .ok_or(FamilyCondition::MissingSubject(event))?;
    Ok(Facts {
        enabled: true,
        current: None,
        inputs: tx
            .inputs()
            .iter()
            .map(|row| {
                let point = row.outpoint();
                let previous =
                    c.parents
                        .get(&Address(*point.txid().internal()))
                        .and_then(|parent| {
                            usize::try_from(point.index())
                                .ok()
                                .and_then(|i| parent.outputs().get(i))
                        });
                InputFact {
                    txid: *point.txid().internal(),
                    index: point.index(),
                    flags: 0,
                    program: previous
                        .map_or(ProgramFact::Unavailable, |output| program(output.program())),
                }
            })
            .collect(),
        outputs: tx
            .outputs()
            .iter()
            .map(|output| program(output.program()))
            .collect(),
    })
}
pub(super) fn evaluate(
    c: &FamilyContext,
    facts: &Facts,
    row: &RelationEvidence,
    schedule: Schedule,
) -> Result<Trace, FamilyCondition> {
    let tx = c
        .decoded
        .get(&row.event)
        .ok_or(FamilyCondition::MissingSubject(row.event))?;
    let stack = tx
        .witnesses()
        .get(row.input)
        .ok_or(FamilyCondition::IndexDisagreement)?
        .stack();
    // The immutable reader result authenticated the complete recipe. Recheck exact
    // byte binding here before slicing, then replay and compare the retained trace. This adds consistency only.
    if stack.len() != 2 || stack[0] != row.leaf || stack[1] != row.control || stack[0].len() != 52 {
        return Err(FamilyCondition::IndexDisagreement);
    }
    let mut facts = facts.clone();
    facts.current = Some(u32::try_from(row.input).map_err(|_| FamilyCondition::Arithmetic)?);
    let result = match schedule {
        Schedule::Birth => model::fragment_untraced(&stack[0][34..44], &facts),
        Schedule::Continuation => model::fragment_untraced(&stack[0][44..51], &facts),
        Schedule::Leaf => model::leaf_untraced(&stack[0], &facts, &row.member.key.0),
    };
    let trace = result.map_err(|error| FamilyCondition::Model {
        event: row.event,
        input: row.input,
        schedule,
        kind: error.kind(),
    })?;
    let expected = match schedule {
        Schedule::Leaf => c
            .reading
            .leaves()
            .iter()
            .find(|r| r.event == row.event && r.input == row.input && r.member == row.member)
            .map(|r| (r.primitive_count, r.peak, r.maximum_width)),
        Schedule::Birth | Schedule::Continuation => {
            let relation = if schedule == Schedule::Birth {
                Relation::BornFirst
            } else {
                Relation::ContinueFirst
            };
            c.reading
                .relations()
                .iter()
                .find(|r| {
                    r.event == row.event
                        && r.input == row.input
                        && r.member == row.member
                        && r.relation == relation
                })
                .map(|r| (r.primitive_count, r.peak, r.maximum_width))
        }
    }
    .ok_or(FamilyCondition::IndexDisagreement)?;
    check_replay(
        (trace.primitive_count, trace.peak, trace.maximum_width),
        expected,
    )?;
    Ok(trace)
}
pub(super) fn check_replay(
    actual: (u64, usize, usize),
    expected: (u64, usize, usize),
) -> Result<(), FamilyCondition> {
    if actual != expected {
        return Err(FamilyCondition::ReplayDisagreement);
    }
    Ok(())
}
pub(super) fn relations(
    c: &FamilyContext,
    event: EventId,
    relation: Relation,
) -> Result<Vec<RelationEvidence>, FamilyCondition> {
    let facts = context(c, event)?;
    c.reading
        .relations()
        .iter()
        .filter(|row| row.event == event && row.relation == relation)
        .map(|row| {
            let schedule = match relation {
                Relation::BornFirst => Schedule::Birth,
                Relation::ContinueFirst => Schedule::Continuation,
            };
            evaluate(c, &facts, row, schedule)?;
            Ok(row.clone())
        })
        .collect()
}
