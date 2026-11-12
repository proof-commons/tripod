//! Grammar versus replay counters and decoded-field sizes under canonical round trip.
//! Whole weight evaluates the projector formula on measured terms; only the harness
//! elements implementation is an independent weight comparison.
use super::{
    Absence, Family, FamilyCondition, FamilyContext, Measurement, ResourceComparison,
    ResourcesBody, Schedule, ScheduleResources, Unit, families::trace, family_types::present,
};
use crate::keyed_history_reader::{EventId, Relation};
use tapscript::keyed_resources::{
    KeyedResourceTerm, KeyedSchedule, keyed_resource_projection, keyed_transaction_weight,
    keyed_witness_bytes,
};
use target_elements::ResourceDimension;

const fn absent() -> Measurement {
    Measurement::Absent(Absence::NotObservedHere)
}
fn comparison(
    predicted: u64,
    model: Option<u64>,
    direct: Option<u64>,
    unit: Unit,
) -> ResourceComparison {
    ResourceComparison {
        predicted: present(predicted, unit),
        model: model.map_or_else(absent, |v| present(v, unit)),
        direct: direct.map_or_else(absent, |v| present(v, unit)),
    }
}
fn width(n: usize) -> u64 {
    transaction::compact_size(n as u64).len() as u64
}
fn witness(items: &[Vec<u8>]) -> Result<u64, FamilyCondition> {
    let mut size = 3u64
        .checked_add(width(items.len()))
        .ok_or(FamilyCondition::Arithmetic)?;
    for item in items {
        size = size
            .checked_add(width(item.len()))
            .and_then(|v| v.checked_add(item.len() as u64))
            .ok_or(FamilyCondition::Arithmetic)?;
    }
    Ok(size)
}
fn sum(
    mut values: impl Iterator<Item = Result<u64, FamilyCondition>>,
) -> Result<u64, FamilyCondition> {
    values.try_fold(0u64, |a, b| {
        a.checked_add(b?).ok_or(FamilyCondition::Arithmetic)
    })
}
fn member_base(
    tx: &transaction::TargetTransaction,
    relations: &[&crate::keyed_history_reader::RelationEvidence],
) -> Result<u64, FamilyCondition> {
    sum(relations.iter().map(|row| {
        let script = tx.inputs()[row.input].script_sig();
        40u64
            .checked_add(width(script.len()))
            .and_then(|v| v.checked_add(script.len() as u64))
            .ok_or(FamilyCondition::Arithmetic)
    }))
}
pub(super) fn member_prediction(
    members: u64,
    measured_base: u64,
    scale: u64,
    witness: u64,
) -> Result<u64, FamilyCondition> {
    // Schema-one member input: outpoint 36, empty scriptSig prefix 1, sequence 4.
    let recipe_base = 36u64
        .checked_add(width(0))
        .and_then(|v| v.checked_add(4))
        .and_then(|v| v.checked_mul(members))
        .ok_or(FamilyCondition::Arithmetic)?;
    if recipe_base != measured_base {
        return Err(FamilyCondition::MemberFormDisagreement);
    }
    recipe_base
        .checked_mul(scale)
        .and_then(|v| v.checked_add(witness))
        .ok_or(FamilyCondition::Arithmetic)
}
fn decoded_witness(
    tx: &transaction::TargetTransaction,
    bytes: &[u8],
) -> Result<u64, FamilyCondition> {
    // Recomputed from decoded fields under the reader's canonical round trip.
    // Three empty input fields are fixed by the decoder envelope, not measured.
    let size = if bytes.get(4) == Some(&1) {
        let inputs = sum(tx.witnesses().iter().map(|w| witness(w.stack())))?;
        let outputs = sum(tx.output_witnesses().iter().map(|w| {
            sum([w.surjection_proof(), w.range_proof()]
                .into_iter()
                .map(|bytes| {
                    width(bytes.len())
                        .checked_add(bytes.len() as u64)
                        .ok_or(FamilyCondition::Arithmetic)
                }))
        }))?;
        inputs
            .checked_add(outputs)
            .ok_or(FamilyCondition::Arithmetic)?
    } else {
        0
    };
    Ok(size)
}
pub(super) fn derive(c: &FamilyContext, event: EventId) -> Result<ResourcesBody, FamilyCondition> {
    let tx = c
        .decoded
        .get(&event)
        .ok_or(FamilyCondition::MissingSubject(event))?;
    let bytes = &c.input.bytes[&event];
    let target = target_elements::reviewed_elements_tapscript().map_err(FamilyCondition::Target)?;
    let relations: Vec<_> = c
        .reading
        .relations()
        .iter()
        .filter(|r| r.event == event && r.relation == Relation::BornFirst)
        .collect();
    let schedules = schedules(c, event, &relations, &target)?;
    let members = relations.len() as u64;
    let member_base = member_base(tx, &relations)?;
    let member_witness = sum(relations
        .iter()
        .map(|r| witness(tx.witnesses()[r.input].stack())))?;
    let all_witness = decoded_witness(tx, bytes)?;
    let base = (bytes.len() as u64)
        .checked_sub(all_witness)
        .ok_or(FamilyCondition::Arithmetic)?;
    let other = all_witness
        .checked_sub(member_witness)
        .ok_or(FamilyCondition::Arithmetic)?;
    let script_stack = keyed_witness_bytes(members).map_err(|_| FamilyCondition::Arithmetic)?;
    let empty_fields = 3u64
        .checked_mul(members)
        .ok_or(FamilyCondition::Arithmetic)?;
    let predicted_witness = script_stack
        .checked_add(empty_fields)
        .ok_or(FamilyCondition::Arithmetic)?;
    if member_witness != predicted_witness {
        return Err(FamilyCondition::PredictionDisagreement);
    }
    let scale = target
        .definition()
        .resources()
        .consensus()
        .witness_scale_factor();
    let predicted = keyed_transaction_weight(&target, members, base, other)
        .map_err(|_| FamilyCondition::Arithmetic)?;
    let direct = scale
        .checked_mul(base)
        .and_then(|v| v.checked_add(all_witness))
        .ok_or(FamilyCondition::Arithmetic)?;
    if predicted != direct {
        return Err(FamilyCondition::PredictionDisagreement);
    }
    let predicted_member = member_prediction(members, member_base, scale, predicted_witness)?;
    let measured_member = member_base
        .checked_mul(scale)
        .and_then(|v| v.checked_add(member_witness))
        .ok_or(FamilyCondition::Arithmetic)?;
    Ok(ResourcesBody {
        trace: trace(Family::Resources),
        schedules,
        members,
        members_unit: Unit::Items,
        script_stack_bytes: comparison(
            script_stack,
            None,
            Some(
                member_witness
                    .checked_sub(empty_fields)
                    .ok_or(FamilyCondition::Arithmetic)?,
            ),
            Unit::Bytes,
        ),
        member_witness_bytes: comparison(
            predicted_witness,
            None,
            Some(member_witness),
            Unit::Bytes,
        ),
        // The reader refuses nonempty member scriptSig before this defensive
        // recipe-versus-decoded-base comparison (41 base bytes).
        member_weight: comparison(
            predicted_member,
            None,
            Some(measured_member),
            Unit::WeightUnits,
        ),
        base_bytes: base,
        base_bytes_unit: Unit::Bytes,
        non_member_witness_bytes: other,
        non_member_witness_bytes_unit: Unit::Bytes,
        transaction_weight: comparison(predicted, None, Some(direct), Unit::WeightUnits),
    })
}
fn schedules(
    c: &FamilyContext,
    event: EventId,
    relations: &[&crate::keyed_history_reader::RelationEvidence],
    target: &target_elements::ReviewedElementsTapscriptDefinition,
) -> Result<Vec<ScheduleResources>, FamilyCondition> {
    let projections = [
        KeyedSchedule::Birth,
        KeyedSchedule::Continuation,
        KeyedSchedule::Leaf,
    ]
    .map(|schedule| {
        keyed_resource_projection(target, schedule).map_err(FamilyCondition::Projection)
    });
    let [birth, continuation, leaf_projection] = projections;
    let projections = [birth?, continuation?, leaf_projection?];
    let grammar = [
        tapscript::born_first_fragment(target)
            .map_err(FamilyCondition::Projection)?
            .encode(target),
        tapscript::keyed_pattern::continue_first_fragment(target)
            .map_err(FamilyCondition::Projection)?
            .encode(target),
    ];
    let facts = super::family_execution::context(c, event)?;
    let mut schedules = Vec::new();
    for row in relations {
        if row.leaf[34..44] != grammar[0] || row.leaf[44..51] != grammar[1] {
            return Err(FamilyCondition::GrammarDisagreement);
        }
        let arguments = c.decoded[&event].witnesses()[row.input]
            .stack()
            .len()
            .checked_sub(2)
            .ok_or(FamilyCondition::IndexDisagreement)?;
        let birth = super::family_execution::evaluate(c, &facts, row, Schedule::Birth)?;
        let continuing = super::family_execution::evaluate(c, &facts, row, Schedule::Continuation)?;
        let leaf = super::family_execution::evaluate(c, &facts, row, Schedule::Leaf)?;
        for (schedule, fragment, operations, peak, maximum_width) in [
            (
                Schedule::Birth,
                &row.leaf[34..44],
                birth.primitive_count,
                birth.peak,
                birth.maximum_width,
            ),
            (
                Schedule::Continuation,
                &row.leaf[44..51],
                continuing.primitive_count,
                continuing.peak,
                continuing.maximum_width,
            ),
            (
                Schedule::Leaf,
                row.leaf.as_slice(),
                leaf.primitive_count,
                leaf.peak,
                leaf.maximum_width,
            ),
        ] {
            let projection = &projections[match schedule {
                Schedule::Birth => 0,
                Schedule::Continuation => 1,
                Schedule::Leaf => 2,
            }];
            schedules.push(schedule_row(
                row,
                schedule,
                fragment,
                (operations, peak, maximum_width),
                projection,
                arguments,
            )?);
        }
    }
    Ok(schedules)
}

fn schedule_row(
    row: &crate::keyed_history_reader::RelationEvidence,
    schedule: Schedule,
    fragment: &[u8],
    counters: (u64, usize, usize),
    projection: &tapscript::keyed_resources::KeyedResourceProjection,
    arguments: usize,
) -> Result<ScheduleResources, FamilyCondition> {
    let (operations, peak, maximum_width) = counters;
    let event = row.event;
    let KeyedResourceTerm::Exact(script_bytes) =
        projection.dimensions[&ResourceDimension::ScriptBytes]
    else {
        return Err(FamilyCondition::ProjectionShape);
    };
    let KeyedResourceTerm::Exact(stack) = projection.dimensions[&ResourceDimension::PeakStackItems]
    else {
        return Err(FamilyCondition::ProjectionShape);
    };
    let KeyedResourceTerm::ElementWidthBound(bound) =
        projection.dimensions[&ResourceDimension::StackElementBytes]
    else {
        return Err(FamilyCondition::ProjectionShape);
    };
    if script_bytes != fragment.len() as u64
        || projection.primitive_count != operations
        || stack != peak as u64
        || maximum_width as u64 > bound
    {
        return Err(FamilyCondition::PredictionDisagreement);
    }
    Ok(ScheduleResources {
        event,
        input: row.input,
        member: row.member,
        schedule,
        script: fragment.to_vec(),
        control: row.control.clone(),
        execution_arguments: arguments,
        execution_arguments_unit: Unit::Items,
        dimensions: projection.dimensions.clone(),
        script_bytes: comparison(script_bytes, None, Some(fragment.len() as u64), Unit::Bytes),
        operations: comparison(
            projection.primitive_count,
            Some(operations),
            None,
            Unit::Operations,
        ),
        peak: comparison(stack, Some(peak as u64), None, Unit::Items),
        element_width: ResourceComparison {
            predicted: present(bound, Unit::Bound),
            model: present(maximum_width as u64, Unit::Bytes),
            direct: absent(),
        },
    })
}

fn measurement(a: &Measurement, b: &Measurement) -> Result<(), FamilyCondition> {
    match (a, b) {
        (
            Measurement::Present { value: a, unit: ua },
            Measurement::Present { value: b, unit: ub },
        ) => {
            if ua != ub {
                Err(FamilyCondition::ResourceUnit)
            } else if a != b {
                Err(FamilyCondition::ResourceCount)
            } else {
                Ok(())
            }
        }
        (Measurement::Absent(a), Measurement::Absent(b)) if a == b => Ok(()),
        (Measurement::Present { .. } | Measurement::Absent(_), _) => {
            Err(FamilyCondition::ResourceAvailability)
        }
    }
}
fn quantities(a: &ResourceComparison, b: &ResourceComparison) -> Result<(), FamilyCondition> {
    measurement(&a.predicted, &b.predicted)?;
    measurement(&a.model, &b.model)?;
    measurement(&a.direct, &b.direct)
}
pub(super) fn compare(a: &ResourcesBody, b: &ResourcesBody) -> Result<(), FamilyCondition> {
    if a.trace != b.trace {
        return Err(FamilyCondition::Trace);
    }
    if a.members_unit != b.members_unit
        || a.base_bytes_unit != b.base_bytes_unit
        || a.non_member_witness_bytes_unit != b.non_member_witness_bytes_unit
    {
        return Err(FamilyCondition::ResourceUnit);
    }
    if a.members != b.members
        || a.base_bytes != b.base_bytes
        || a.non_member_witness_bytes != b.non_member_witness_bytes
    {
        return Err(FamilyCondition::ResourceTerms);
    }
    if a.schedules.len() != b.schedules.len() {
        return Err(FamilyCondition::ScheduleIdentity);
    }
    for (a, b) in a.schedules.iter().zip(&b.schedules) {
        if (a.event, a.input, a.member, a.schedule) != (b.event, b.input, b.member, b.schedule) {
            return Err(FamilyCondition::ScheduleIdentity);
        }
        if a.dimensions != b.dimensions {
            return Err(FamilyCondition::ProjectionTerms);
        }
        if a.execution_arguments_unit != b.execution_arguments_unit {
            return Err(FamilyCondition::ResourceUnit);
        }
        if a.execution_arguments != b.execution_arguments {
            return Err(FamilyCondition::ExecutionArguments);
        }
        if a.control != b.control {
            return Err(FamilyCondition::Control);
        }
        if a.script != b.script {
            return Err(FamilyCondition::Script);
        }
        for (a, b) in [
            (&a.script_bytes, &b.script_bytes),
            (&a.operations, &b.operations),
            (&a.peak, &b.peak),
            (&a.element_width, &b.element_width),
        ] {
            quantities(a, b)?;
        }
    }
    for (a, b) in [
        (&a.script_stack_bytes, &b.script_stack_bytes),
        (&a.member_witness_bytes, &b.member_witness_bytes),
        (&a.member_weight, &b.member_weight),
        (&a.transaction_weight, &b.transaction_weight),
    ] {
        quantities(a, b)?;
    }
    Ok(())
}

/// Header resource quantities summarize whole-leaf calls/peak only (not the three schedules summed).
pub(super) fn header(
    header: &mut super::Header,
    body: &ResourcesBody,
) -> Result<(), FamilyCondition> {
    use super::{Dimension as D, Quantity};
    let bytes = header
        .subject
        .as_ref()
        .ok_or(FamilyCondition::IndexDisagreement)?
        .bytes
        .len();
    let mut rows =
        super::report::quantities(None, None, bytes).map_err(FamilyCondition::Observation)?;
    rows.insert(
        D::Weight,
        Quantity {
            observed: body.transaction_weight.direct.clone(),
            predicted: body.transaction_weight.predicted.clone(),
        },
    );
    let leaves: Vec<_> = body
        .schedules
        .iter()
        .filter(|s| s.schedule == Schedule::Leaf)
        .collect();
    if !leaves.is_empty() {
        let total = |predicted: bool| {
            sum(leaves.iter().map(|s| {
                match if predicted {
                    &s.operations.predicted
                } else {
                    &s.operations.model
                } {
                    Measurement::Present {
                        value,
                        unit: Unit::Operations,
                    } => Ok(*value),
                    _ => Err(FamilyCondition::ProjectionShape),
                }
            }))
        };
        let peak = |predicted: bool| -> Result<u64, FamilyCondition> {
            leaves
                .iter()
                .map(|s| {
                    match if predicted {
                        &s.peak.predicted
                    } else {
                        &s.peak.model
                    } {
                        Measurement::Present {
                            value,
                            unit: Unit::Items,
                        } => Ok(*value),
                        _ => Err(FamilyCondition::ProjectionShape),
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|v| v.into_iter().max().unwrap_or(0))
        };
        rows.insert(
            D::Operations,
            Quantity {
                observed: present(total(false)?, Unit::Operations),
                predicted: present(total(true)?, Unit::Operations),
            },
        );
        rows.insert(
            D::StackItems,
            Quantity {
                observed: present(peak(false)?, Unit::Items),
                predicted: present(peak(true)?, Unit::Items),
            },
        );
    }
    header.quantities = rows;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn family_witness_formula_covers_compact_size_transition() {
        for (n, width) in [(252, 1), (253, 3)] {
            assert_eq!(witness(&[vec![7; n]]), Ok(3 + 1 + width + n as u64));
        }
        assert_eq!(member_prediction(1, 41, 4, 91), Ok(255));
        assert_eq!(
            member_prediction(1, 42, 4, 91),
            Err(FamilyCondition::MemberFormDisagreement)
        );
        assert_eq!(
            member_prediction(u64::MAX, 41, 4, 91),
            Err(FamilyCondition::Arithmetic)
        );
        assert_eq!(
            super::super::family_execution::check_replay((5, 3, 32), (5, 3, 32)),
            Ok(())
        );
        assert_eq!(
            super::super::family_execution::check_replay((6, 3, 32), (5, 3, 32)),
            Err(FamilyCondition::ReplayDisagreement)
        );
    }
}
