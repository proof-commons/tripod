//! The candidate resource study of Guide-12 §20.
//!
//! §20.2 asks for a finite candidate enumeration and §20.3 for a
//! per-candidate measurement, so this module enumerates bound
//! assignments and measures what each one costs. Every figure here is
//! recomputed from bytes this process emitted; none is transcribed from
//! a previous wave's notes.
//!
//! # Emission is measured for every candidate, and linking is not
//!
//! The study has two tiers because the pipeline has two tiers. Emission
//! takes a shape and answers with a program, so every assignment in the
//! enumeration can be emitted and measured. Linking additionally builds
//! one taptree over the whole leaf set, and the linker's exact-cost
//! oracle refuses a set above its own budget — so most of the
//! enumeration has no linked measurement at all, and saying which ones
//! is one of this study's results rather than an omission from it.
//!
//! [`EmittedMeasurement`] is therefore the tier every candidate has,
//! and it says so in its own name: these are the bytes emission
//! produces, before relocation substitutes any deployment literal.
//! §13.3's figures move under substitution, so a linked candidate's
//! bytes are read off the linked bundle and never predicted from here.
//!
//! # What this module does not decide
//!
//! Nothing here selects a bound. §20.6 admits a statement that some
//! assignment fits a tested bundle and forbids recording a production
//! value, so this module reports measurements and the conditions of
//! §9.3, and no function in it returns a chosen candidate.

use std::collections::BTreeMap;
use std::num::NonZeroU8;

use tapscript::{
    CompactAshShape, CompactAshShapeBounds, CompactAshSymbols, MINIMUM_ASH_INPUTS, ResourceModel,
    TapscriptError, TapscriptInstruction, TapscriptProgram, coordinator_program, dense_shape_set,
    fit_shape_model, member_program, resource_projection,
};
use target_elements::{ResourceDimension, ReviewedElementsTapscriptDefinition};

/// The §20.2 research ASH bounds, in the order §20.2 states them.
///
/// Research inputs, not accepted values. §20.2 says so in those words,
/// and the name keeps the distinction readable at every use site.
pub const RESEARCH_ASH_BOUNDS: [u8; 6] = [2, 4, 8, 16, 32, 64];

/// The §20.2 research sponsor bounds, in the order §20.2 states them.
pub const RESEARCH_SPONSOR_BOUNDS: [u8; 6] = [0, 1, 2, 4, 8, 16];

/// Every §20.2 candidate bound assignment, in canonical order.
///
/// The product of the two research lists, ASH bound major. §20.2
/// requires the enumeration to be deterministic and finite, and it is
/// both: the lists are constants and the order is their nesting, so two
/// runs produce the same assignments in the same order.
///
/// # Panics
///
/// Never in practice, and not by suppression: every ASH bound in
/// [`RESEARCH_ASH_BOUNDS`] is at or above [`MINIMUM_ASH_INPUTS`], so
/// the bounds constructor cannot refuse. A panic would mean the
/// research list had been edited below the minimum batch, which must
/// fail loudly rather than silently drop an assignment the study
/// claims to have covered.
#[must_use]
pub fn research_assignments() -> Vec<CompactAshShapeBounds> {
    let mut assignments = Vec::new();
    for ash in RESEARCH_ASH_BOUNDS {
        for sponsors in RESEARCH_SPONSOR_BOUNDS {
            let bound = NonZeroU8::new(ash).unwrap_or(NonZeroU8::MIN);
            assignments.push(
                CompactAshShapeBounds::new(bound, sponsors)
                    .expect("every research ASH bound is at or above the minimum batch"),
            );
        }
    }
    assignments
}

/// How many taptree leaves one assignment's dense unrolling needs.
///
/// One coordinator leaf per shape, plus one member leaf per distinct
/// ASH count — §11.2's sharing rule, which makes a member leaf a
/// function of the batch size alone. Stated as its own function
/// because the linker's oracle budget is a bound on exactly this
/// figure, so a study that wants to know which assignments can link at
/// all needs it before it builds anything.
#[must_use]
pub fn leaf_count(bounds: CompactAshShapeBounds) -> u64 {
    let counts = u64::from(bounds.ash_inputs() - MINIMUM_ASH_INPUTS + 1);
    let per_count = 1 + 2 * u64::from(bounds.sponsor_inputs());
    counts * per_count + counts
}

/// One program role's byte behaviour across an assignment's shapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleBytes {
    largest: u64,
    largest_at: Option<CompactAshShape>,
    total: u64,
    push: u64,
    model: ResourceModel,
}

impl RoleBytes {
    /// The largest encoded program this role reached.
    #[must_use]
    pub const fn largest(&self) -> u64 {
        self.largest
    }

    /// The shape the largest program was emitted for.
    ///
    /// Absent for a role that emitted nothing, which no assignment in
    /// the enumeration produces. It is an absence rather than a
    /// stand-in shape because §20.4 reads this field to say which
    /// candidate maximizes an objective, and a stand-in would name a
    /// shape that never won anything.
    #[must_use]
    pub const fn largest_at(&self) -> Option<CompactAshShape> {
        self.largest_at
    }

    /// Every distinct leaf of this role, summed.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.total
    }

    /// How many of [`Self::total`] are push bytes.
    ///
    /// The figure `G12-R11` was about. A push costs its opcode, its
    /// length prefix where the form carries one, and its payload, and
    /// counting only the opcode understated every emitted program.
    #[must_use]
    pub const fn push(&self) -> u64 {
        self.push
    }

    /// The affine model of this role's encoded length, where one fits.
    #[must_use]
    pub const fn model(&self) -> ResourceModel {
        self.model
    }
}

/// What one candidate bound assignment costs at emission.
///
/// Every field is recomputed from programs this measurement emitted.
/// The figures are pre-link by construction and the type's name says
/// so, because §13.3's substitution changes literal widths and a
/// pre-link byte count does not describe a linked program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmittedMeasurement {
    bounds: CompactAshShapeBounds,
    shapes: u64,
    leaves: u64,
    coordinator: RoleBytes,
    member: RoleBytes,
    dimensions: BTreeMap<ResourceDimension, u64>,
}

impl EmittedMeasurement {
    /// The assignment measured.
    #[must_use]
    pub const fn bounds(&self) -> CompactAshShapeBounds {
        self.bounds
    }

    /// How many shapes the dense unrolling admits.
    #[must_use]
    pub const fn shapes(&self) -> u64 {
        self.shapes
    }

    /// How many distinct leaves those shapes need.
    #[must_use]
    pub const fn leaves(&self) -> u64 {
        self.leaves
    }

    /// The coordinator role's byte behaviour.
    #[must_use]
    pub const fn coordinator(&self) -> &RoleBytes {
        &self.coordinator
    }

    /// The member role's byte behaviour.
    #[must_use]
    pub const fn member(&self) -> &RoleBytes {
        &self.member
    }

    /// Every distinct leaf of both roles, summed.
    #[must_use]
    pub const fn total_script_bytes(&self) -> u64 {
        self.coordinator.total + self.member.total
    }

    /// Every push byte in the whole leaf set.
    #[must_use]
    pub const fn total_push_bytes(&self) -> u64 {
        self.coordinator.push + self.member.push
    }

    /// The greatest charge this assignment reached, per dimension.
    ///
    /// Only the dimensions emission establishes: `ScriptBytes`,
    /// `OperationCost`, and `ValidationBudget`. The peaks, the widest
    /// element, and every whole-transaction figure are owed by later
    /// layers and are absent here rather than defaulted to zero, which
    /// would report "not measured" as a measurement.
    #[must_use]
    pub const fn dimensions(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.dimensions
    }
}

/// Measure one candidate bound assignment at emission.
///
/// Emits every leaf the assignment's dense unrolling needs and reads
/// the figures off those programs. Nothing is cached between calls: two
/// assignments sharing a shape emit its program twice, because a shared
/// cache would make the second answer a function of the first call's
/// order.
///
/// # Errors
///
/// The [`TapscriptError`] the emission refused with. A refusal is a
/// real finding about the assignment — an instruction bound reached, or
/// a literal the reviewed contract will not push — so it is returned
/// rather than skipped.
pub fn measure_emitted(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    bounds: CompactAshShapeBounds,
) -> Result<EmittedMeasurement, TapscriptError> {
    let shapes = dense_shape_set(bounds);

    let mut coordinator = RoleAccumulator::default();
    let mut member = RoleAccumulator::default();
    let mut member_bytes: BTreeMap<u8, u64> = BTreeMap::new();
    let mut shape_count = 0_u64;

    for shape in shapes.shapes() {
        shape_count += 1;
        let program = coordinator_program(target, symbols, shape)?;
        coordinator.observe(target, shape, &program);

        // §11.2: a member leaf is a function of the batch size alone.
        // It is therefore emitted once per distinct ASH count and its
        // bytes are summed once — but the measurement table is keyed by
        // shape, exactly as the backend's own formula fit is, because a
        // table holding only the shapes that introduced a new count
        // varies along no sponsor axis and no affine model fits it.
        if let Some(bytes) = member_bytes.get(&shape.ash_inputs()) {
            member.observe_shared(shape, *bytes);
        } else {
            let program = member_program(target, symbols, shape)?;
            let bytes = member.observe(target, shape, &program);
            member_bytes.insert(shape.ash_inputs(), bytes);
        }
    }

    let dimensions = coordinator
        .dimensions
        .iter()
        .map(|(dimension, charge)| {
            let other = member.dimensions.get(dimension).copied().unwrap_or(0);
            (*dimension, (*charge).max(other))
        })
        .collect();

    Ok(EmittedMeasurement {
        bounds,
        shapes: shape_count,
        leaves: shape_count + member_bytes.len() as u64,
        coordinator: coordinator.settle(),
        member: member.settle(),
        dimensions,
    })
}

/// What one candidate bound assignment answered when measured.
///
/// A refusal is a result, not a gap. §20.2 forbids assuming
/// monotonicity, and an assignment the backend cannot emit for at all
/// is exactly the kind of non-monotone answer the enumeration exists to
/// find — so it is carried beside the measurements rather than dropped
/// from the matrix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateOutcome {
    /// The assignment emitted, and these are its figures.
    Emitted(Box<EmittedMeasurement>),
    /// The assignment did not emit, for this reason.
    Refused(TapscriptError),
}

impl CandidateOutcome {
    /// The measurement, where the assignment emitted one.
    #[must_use]
    pub const fn measurement(&self) -> Option<&EmittedMeasurement> {
        match self {
            Self::Emitted(measurement) => Some(measurement),
            Self::Refused(_) => None,
        }
    }
}

/// Measure every §20.2 assignment, in the enumeration's own order.
///
/// One entry per assignment, and the entries stay aligned with
/// [`research_assignments`] whatever any of them answered — which is
/// what lets the matrix be read as a table rather than as a list of
/// the assignments that happened to work.
#[must_use]
pub fn emitted_matrix(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
) -> Vec<(CompactAshShapeBounds, CandidateOutcome)> {
    research_assignments()
        .into_iter()
        .map(|bounds| {
            let outcome = match measure_emitted(target, symbols, bounds) {
                Ok(measurement) => CandidateOutcome::Emitted(Box::new(measurement)),
                Err(refusal) => CandidateOutcome::Refused(refusal),
            };
            (bounds, outcome)
        })
        .collect()
}

/// The §20.3 table, as whitespace-separated rows.
///
/// One header line and one row per enumerated assignment, in the
/// enumeration's order. A refused assignment carries its refusal in
/// place of its figures rather than being dropped, so the row count is
/// the assignment count whatever any of them answered.
///
/// The rendering is deliberately plain text with no aligned columns:
/// alignment would make the bytes a function of the widest figure in
/// the table, and a report whose bytes move when an unrelated row grows
/// is not one two runs can be compared byte for byte.
#[must_use]
pub fn render_matrix(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
) -> String {
    use core::fmt::Write as _;

    let mut out = String::from(
        "ash sponsors shapes leaves coordinator_largest coordinator_total \
         member_largest member_total push_bytes total_script_bytes\n",
    );

    for (bounds, outcome) in emitted_matrix(target, symbols) {
        match outcome {
            CandidateOutcome::Emitted(measurement) => {
                // Writing into a String cannot fail, and the result is
                // discarded rather than unwrapped so a formatting
                // change can never panic a study run.
                let _ = writeln!(
                    out,
                    "{} {} {} {} {} {} {} {} {} {}",
                    bounds.ash_inputs(),
                    bounds.sponsor_inputs(),
                    measurement.shapes(),
                    measurement.leaves(),
                    measurement.coordinator().largest(),
                    measurement.coordinator().total(),
                    measurement.member().largest(),
                    measurement.member().total(),
                    measurement.total_push_bytes(),
                    measurement.total_script_bytes(),
                );
            }
            CandidateOutcome::Refused(refusal) => {
                let _ = writeln!(
                    out,
                    "{} {} refused {refusal}",
                    bounds.ash_inputs(),
                    bounds.sponsor_inputs(),
                );
            }
        }
    }

    out
}

/// The running figures of one role, while its programs are emitted.
#[derive(Default)]
struct RoleAccumulator {
    largest: u64,
    largest_at: Option<CompactAshShape>,
    total: u64,
    push: u64,
    measurements: BTreeMap<CompactAshShape, u64>,
    dimensions: BTreeMap<ResourceDimension, u64>,
}

impl RoleAccumulator {
    /// Fold one newly emitted leaf into the running figures.
    ///
    /// Returns the leaf's encoded length, so a later shape sharing this
    /// leaf can be tabled against it without emitting it again.
    fn observe(
        &mut self,
        target: &ReviewedElementsTapscriptDefinition,
        shape: CompactAshShape,
        program: &TapscriptProgram,
    ) -> u64 {
        let bytes = program.encoded_length(target);
        self.total = self.total.saturating_add(bytes);
        self.push = self.push.saturating_add(push_bytes(target, program));

        for (dimension, charge) in resource_projection(target, program) {
            let entry = self.dimensions.entry(dimension).or_insert(0);
            *entry = (*entry).max(charge);
        }

        self.observe_shared(shape, bytes);
        bytes
    }

    /// Table one shape against a leaf already counted.
    ///
    /// The measurement table drives the affine fit and is keyed by
    /// shape, so a shared leaf appears once per shape that selects it.
    /// The byte totals are untouched here: the leaf's bytes were
    /// counted when it was emitted, and counting them again would
    /// multiply one leaf by the shapes sharing it.
    fn observe_shared(&mut self, shape: CompactAshShape, bytes: u64) {
        if bytes > self.largest || self.largest_at.is_none() {
            self.largest = bytes;
            self.largest_at = Some(shape);
        }
        self.measurements.insert(shape, bytes);
    }

    /// The settled figures, with the model fitted over what was seen.
    fn settle(self) -> RoleBytes {
        RoleBytes {
            largest: self.largest,
            largest_at: self.largest_at,
            total: self.total,
            push: self.push,
            model: fit_shape_model(&self.measurements),
        }
    }
}

/// How many of a program's encoded bytes belong to its pushes.
///
/// Each push is encoded alone by the program encoder and the result
/// measured, so the count is the encoder's own answer rather than a
/// second account of what a push form costs — which is the shape of
/// mistake `G12-R11` was.
fn push_bytes(target: &ReviewedElementsTapscriptDefinition, program: &TapscriptProgram) -> u64 {
    let mut total = 0_u64;
    for instruction in program.instructions() {
        let TapscriptInstruction::Push(_) = instruction else {
            continue;
        };
        if let Ok(single) = TapscriptProgram::new(vec![instruction.clone()]) {
            total = total.saturating_add(single.encoded_length(target));
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::{
        RESEARCH_ASH_BOUNDS, RESEARCH_SPONSOR_BOUNDS, leaf_count, measure_emitted, render_matrix,
        research_assignments,
    };
    use crate::bundle::{
        CLOSED_ASSET, RESERVE_ASSET, SPONSOR_CHANGE_PROGRAM, fee_program_digest, fixture_bundle,
    };
    use std::collections::BTreeSet;
    use std::num::NonZeroU8;
    use tapscript::{CompactAshShapeBounds, CompactAshSymbols, ProgramRole};
    use target_elements::{ResourceDimension, ReviewedElementsTapscriptDefinition};

    /// The reviewed target, for the measurements below.
    fn target() -> ReviewedElementsTapscriptDefinition {
        target_elements::reviewed_elements_tapscript().expect("the reviewed target binds")
    }

    /// The resolved symbols a deployment links against.
    ///
    /// The study measures with these rather than with the emission
    /// placeholders, because a placeholder's width is chosen to differ
    /// from the resolved one and a byte study of widths nobody deploys
    /// would answer a question nobody asked.
    fn symbols(target: &ReviewedElementsTapscriptDefinition) -> CompactAshSymbols {
        CompactAshSymbols::new(
            target,
            CLOSED_ASSET.to_vec(),
            RESERVE_ASSET.to_vec(),
            SPONSOR_CHANGE_PROGRAM.to_vec(),
            0,
            fee_program_digest().to_vec(),
        )
        .expect("the resolved symbols bind")
    }

    /// Bounds from a pair, for the fixtures.
    fn bounds(ash: u8, sponsors: u8) -> CompactAshShapeBounds {
        CompactAshShapeBounds::new(
            NonZeroU8::new(ash).expect("the fixture bounds are nonzero"),
            sponsors,
        )
        .expect("the fixture bounds are above the minimum")
    }

    #[test]
    fn the_enumeration_is_the_finite_deterministic_product_section_twenty_two_states() {
        // §20.2 states two research lists and requires the enumeration
        // over them to be deterministic and finite. The count is
        // recomputed from the lists rather than written as 36, so
        // editing a list moves the expectation with it.
        let assignments = research_assignments();

        assert_eq!(
            assignments.len(),
            RESEARCH_ASH_BOUNDS.len() * RESEARCH_SPONSOR_BOUNDS.len(),
        );
        assert_eq!(assignments, research_assignments());

        let distinct: BTreeSet<_> = assignments.iter().copied().collect();
        assert_eq!(distinct.len(), assignments.len());

        for bounds in &assignments {
            assert!(RESEARCH_ASH_BOUNDS.contains(&bounds.ash_inputs()));
            assert!(RESEARCH_SPONSOR_BOUNDS.contains(&bounds.sponsor_inputs()));
        }
    }

    #[test]
    fn the_leaf_count_is_the_one_the_emission_actually_produces() {
        // The linker's oracle budget is a bound on the leaf count, so
        // the study needs that figure before it builds anything. A
        // closed form that disagreed with the emission would let the
        // study report an assignment linkable when it is not, so the
        // two are compared on every assignment small enough to emit
        // cheaply.
        let target = target();
        let symbols = symbols(&target);

        for (ash, sponsors) in [(2, 0), (2, 1), (2, 4), (4, 0), (4, 1), (4, 2), (8, 0)] {
            let bounds = bounds(ash, sponsors);
            let measured = measure_emitted(&target, &symbols, bounds).expect("the emission holds");

            assert_eq!(
                measured.leaves(),
                leaf_count(bounds),
                "the closed form and the emission disagree at {ash} ASH and {sponsors} sponsors",
            );
        }
    }

    #[test]
    fn emitting_at_the_resolved_symbols_reproduces_the_linked_bundle_bytes() {
        // The cross-check that makes this study's figures comparable
        // with the pipeline's own. The linker reaches its programs by
        // substituting resolved symbols into placeholder-emitted ones;
        // this module reaches them by emitting at the resolved symbols
        // directly. The two routes must agree byte for byte, and a
        // disagreement would be a real finding about substitution
        // rather than a tolerance to widen.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let measured = measure_emitted(
            bundle.target(),
            &symbols(bundle.target()),
            bundle.linked().shapes().bounds(),
        )
        .expect("the demonstration assignment emits");

        assert_eq!(
            measured.total_script_bytes(),
            bundle.linked().total_script_bytes(),
        );
        assert_eq!(measured.leaves(), bundle.linked().programs().len() as u64);

        for (role, bytes) in [
            (ProgramRole::Coordinator, measured.coordinator()),
            (ProgramRole::Member, measured.member()),
        ] {
            let linked = bundle
                .linked()
                .formulas()
                .get(&role)
                .and_then(|dimensions| dimensions.get(&ResourceDimension::ScriptBytes))
                .expect("every role has a linked script-byte formula");

            assert_eq!(
                bytes.model(),
                linked.model(),
                "the {role:?} byte model differs between emission and linking",
            );
        }
    }

    #[test]
    fn script_bytes_include_the_push_bytes_they_are_made_of() {
        // `G12-R11`: the projection once counted opcode bytes and
        // omitted every push opcode, length prefix, and payload. The
        // programs are mostly pushed literals, so a push total of zero
        // — or one equal to the whole program — is the shape that
        // defect had, and both are refused here.
        let target = target();
        let symbols = symbols(&target);
        let measured =
            measure_emitted(&target, &symbols, bounds(4, 1)).expect("the emission holds");

        assert!(measured.total_push_bytes() > 0);
        assert!(measured.total_push_bytes() < measured.total_script_bytes());
        assert_eq!(
            measured.dimensions().get(&ResourceDimension::ScriptBytes),
            Some(&measured.coordinator().largest()),
            "the largest charged script-byte figure is the largest program",
        );
    }

    #[test]
    fn the_rendered_matrix_carries_one_row_per_assignment() {
        // §20.3's table is the study's own deliverable, so the
        // rendering is checked rather than eyeballed: one header and
        // one row per enumerated assignment, and the demonstration
        // candidate's row carrying the figures the linked bundle
        // independently reports.
        let target = target();
        let rendered = render_matrix(&target, &symbols(&target));
        let lines: Vec<_> = rendered.lines().collect();

        assert_eq!(lines.len(), research_assignments().len() + 1);
        assert!(lines[0].starts_with("ash sponsors"));
        assert!(
            lines.iter().any(|line| line.starts_with("4 1 9 12 ")),
            "the demonstration assignment is nine shapes over twelve leaves",
        );
    }

    #[test]
    fn emission_establishes_three_dimensions_and_leaves_the_rest_absent() {
        // §20.3 names eighteen measures and emission answers a few of
        // them. The ones it does not answer are absent rather than
        // zero: a zero peak stack would report "not measured" as a
        // measurement, which is the reading the null-carrying
        // observation schema exists to prevent.
        let target = target();
        let symbols = symbols(&target);
        let measured =
            measure_emitted(&target, &symbols, bounds(4, 1)).expect("the emission holds");

        let established: BTreeSet<_> = measured.dimensions().keys().copied().collect();
        assert_eq!(
            established,
            BTreeSet::from([
                ResourceDimension::ScriptBytes,
                ResourceDimension::OperationCost,
                ResourceDimension::ValidationBudget,
            ]),
        );

        for owed in [
            ResourceDimension::PeakStackItems,
            ResourceDimension::StackElementBytes,
            ResourceDimension::TransactionWeight,
            ResourceDimension::WitnessBytes,
            ResourceDimension::ControlPathDepth,
        ] {
            assert!(
                !measured.dimensions().contains_key(&owed),
                "{owed:?} is owed by a later layer and must not be answered here",
            );
        }
    }
}
