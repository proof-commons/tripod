//! Exact linked resource formulas for the live shape set (§11.1, §11.6).
//!
//! # Why compact ASH's model could not be reused
//!
//! [`tapscript::ResourceModel`] is an exact affine function of compact
//! ASH's *three* axes. A live-transfer shape has four — receipt inputs,
//! receipt outputs, sponsor inputs, and the sponsor-change presence —
//! because §5.3 admits split and merge, so the count of receipts created
//! is free of the count consumed. Fitting a live table with the
//! three-axis model would have had to drop one axis, and dropping an axis
//! is not a coarser model: it is a model that is wrong wherever that axis
//! moves.
//!
//! §11.1 names *explicit and private resource formulas* among the five
//! concrete needs the linker may extend for, and this is that extension:
//! the same fitting discipline over the axes a live shape actually has.
//!
//! # The discipline is the one compact ASH established
//!
//! Coefficients are exact unit differences taken *inside* the measured
//! table, never a regression; the model is then recomputed at every
//! measured shape and discarded whole if it misses one. There is no
//! best-fit here. A model right about most shapes is the truncated
//! result §1.11 refuses, and [`LiveResourceModel::ExactTableOnly`] is
//! what a table with no exact model reports — with the exact per-shape
//! figures still standing beside it, because those are measurements and
//! remain true whether or not a formula summarizes them.
//!
//! # Refitted, never carried across
//!
//! Substitution changes the pushed literals' widths, so every linked
//! program's exact encoded byte length differs from the pre-link one and
//! the pre-link coefficients describe a program that no longer exists.

use std::collections::BTreeMap;

use tapscript::upstream::LiveTransferRepresentationPlan;
use tapscript::{LiveProgramRole, LiveTransferShape, SponsorChangePresence};
use target_elements::ResourceDimension;

/// An exact model of one dimension over the live shape set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveResourceModel {
    /// An exact affine function of the shape's four counts.
    ///
    /// Valid over exactly the measured shape set and nowhere else: the
    /// coefficients are read off differences within that set and then
    /// recomputed against every member of it. A shape outside the set has
    /// no program, so it has no figure for this to predict.
    Affine {
        /// The constant term.
        base: i64,
        /// The cost of one further receipt input.
        per_receipt_input: i64,
        /// The cost of one further receipt output.
        per_receipt_output: i64,
        /// The cost of the first sponsor input and the fee role it
        /// forces.
        per_sponsor_input: i64,
        /// The cost of the optional sponsor-change role.
        per_sponsor_change: i64,
    },
    /// No affine model reproduces every measurement exactly.
    ///
    /// Never a partial fit: the exact per-shape figures stand alone
    /// rather than a model that is right about most of them (§1.11).
    ExactTableOnly,
}

/// One representation and role's linked resource behaviour (§11.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedLiveResourceFormula {
    representation: LiveTransferRepresentationPlan,
    role: LiveProgramRole,
    dimension: ResourceDimension,
    model: LiveResourceModel,
    measurements: BTreeMap<LiveTransferShape, u64>,
}

impl LinkedLiveResourceFormula {
    /// The representation this formula describes.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// The program role this formula describes.
    #[must_use]
    pub const fn role(&self) -> LiveProgramRole {
        self.role
    }

    /// The dimension this formula describes.
    #[must_use]
    pub const fn dimension(&self) -> ResourceDimension {
        self.dimension
    }

    /// The refitted model.
    #[must_use]
    pub const fn model(&self) -> LiveResourceModel {
        self.model
    }

    /// The exact linked measurement at every shape this role serves.
    #[must_use]
    pub const fn measurements(&self) -> &BTreeMap<LiveTransferShape, u64> {
        &self.measurements
    }
}

/// State one role's formula over its exact linked measurements.
///
/// Crate-visible rather than public because the measurements have to be
/// the linked ones: a formula built over figures somebody else supplied
/// would describe programs this link did not produce.
pub(crate) fn linked_formula(
    representation: LiveTransferRepresentationPlan,
    role: LiveProgramRole,
    dimension: ResourceDimension,
    measurements: BTreeMap<LiveTransferShape, u64>,
) -> LinkedLiveResourceFormula {
    let model = fit_live_model(&measurements);
    LinkedLiveResourceFormula {
        representation,
        role,
        dimension,
        model,
        measurements,
    }
}

/// How many axes a live shape presents to the model.
const AXES: usize = 4;

/// The four axes one live shape presents to the model, in census order.
///
/// An array rather than four named fields, because every use of it is a
/// walk: the coefficients are fitted one axis at a time and predicted by
/// a dot product, and naming the positions would mean writing both loops
/// out four times.
const fn axis_values(shape: LiveTransferShape) -> [i64; AXES] {
    [
        shape.receipt_inputs() as i64,
        shape.receipt_outputs() as i64,
        shape.sponsor_inputs() as i64,
        match shape.sponsor_change() {
            SponsorChangePresence::Present => 1,
            SponsorChangePresence::Absent => 0,
        },
    ]
}

/// The coefficients of one fitted model, in the same census order.
const fn coefficients(model: LiveResourceModel) -> Option<(i64, [i64; AXES])> {
    match model {
        LiveResourceModel::Affine {
            base,
            per_receipt_input,
            per_receipt_output,
            per_sponsor_input,
            per_sponsor_change,
        } => Some((
            base,
            [
                per_receipt_input,
                per_receipt_output,
                per_sponsor_input,
                per_sponsor_change,
            ],
        )),
        LiveResourceModel::ExactTableOnly => None,
    }
}

/// The exact affine model of one measurement table, or none.
///
/// The coefficients are exact differences taken inside the table, and the
/// model is then recomputed at every measured shape. A model that misses
/// one shape is discarded whole.
#[must_use]
pub fn fit_live_model(measurements: &BTreeMap<LiveTransferShape, u64>) -> LiveResourceModel {
    let Some(per_receipt_input) = unit_difference(measurements, 0) else {
        return LiveResourceModel::ExactTableOnly;
    };
    let Some(per_receipt_output) = unit_difference(measurements, 1) else {
        return LiveResourceModel::ExactTableOnly;
    };
    let Some(per_sponsor_input) = unit_difference(measurements, 2) else {
        return LiveResourceModel::ExactTableOnly;
    };
    let Some(per_sponsor_change) = unit_difference(measurements, 3) else {
        return LiveResourceModel::ExactTableOnly;
    };

    let Some((shape, measure)) = measurements.iter().next() else {
        return LiveResourceModel::ExactTableOnly;
    };
    let Ok(measure) = i64::try_from(*measure) else {
        return LiveResourceModel::ExactTableOnly;
    };
    let terms = [
        per_receipt_input,
        per_receipt_output,
        per_sponsor_input,
        per_sponsor_change,
    ];
    let Some(base) =
        weighted(terms, axis_values(*shape)).and_then(|term| measure.checked_sub(term))
    else {
        return LiveResourceModel::ExactTableOnly;
    };

    let model = LiveResourceModel::Affine {
        base,
        per_receipt_input,
        per_receipt_output,
        per_sponsor_input,
        per_sponsor_change,
    };

    // Recomputed at every measured shape. A coefficient set derived from
    // a few differences is a hypothesis until it reproduces the table.
    for (shape, measure) in measurements {
        let Ok(measure) = i64::try_from(*measure) else {
            return LiveResourceModel::ExactTableOnly;
        };
        if predict_live_model(model, *shape) != Some(measure) {
            return LiveResourceModel::ExactTableOnly;
        }
    }

    model
}

/// What one fitted model predicts at one shape, where a model exists.
///
/// The companion of [`fit_live_model`], public for the same reason: a
/// refitted model has to be checkable by whoever refitted it.
#[must_use]
pub fn predict_live_model(model: LiveResourceModel, shape: LiveTransferShape) -> Option<i64> {
    let (base, terms) = coefficients(model)?;
    weighted(terms, axis_values(shape)).and_then(|term| base.checked_add(term))
}

/// The weighted sum of one shape's axes under four coefficients.
///
/// Checked throughout: a saturated term would make the recomputation
/// above agree with a model that does not describe the table.
fn weighted(terms: [i64; AXES], axes: [i64; AXES]) -> Option<i64> {
    terms
        .iter()
        .zip(axes)
        .try_fold(0_i64, |total, (term, axis)| {
            total.checked_add(term.checked_mul(axis)?)
        })
}

/// The exact cost of one unit step along one axis, where the table fixes
/// one.
///
/// A pair of measured shapes differing by exactly one on the chosen axis
/// and equal on the other three. Every such pair must give the same
/// difference; a table where two pairs disagree has no affine model, and
/// a table with no such pair leaves the coefficient unfixed — which is
/// reported as zero only when the axis is constant across the whole
/// table, and as no model otherwise. An unfixed coefficient guessed at
/// would be a model asserting something the measurements never showed.
fn unit_difference(measurements: &BTreeMap<LiveTransferShape, u64>, axis: usize) -> Option<i64> {
    let axis_of = |shape: LiveTransferShape| -> i64 { axis_values(shape)[axis.min(AXES - 1)] };
    let others = |shape: LiveTransferShape| -> Vec<i64> {
        axis_values(shape)
            .into_iter()
            .enumerate()
            .filter(|(index, _)| *index != axis)
            .map(|(_, value)| value)
            .collect()
    };

    let mut difference: Option<i64> = None;
    let mut moved = false;
    for (low, low_measure) in measurements {
        for (high, high_measure) in measurements {
            if others(*low) != others(*high) || axis_of(*high) != axis_of(*low) + 1 {
                continue;
            }
            let step = i64::try_from(*high_measure)
                .ok()?
                .checked_sub(i64::try_from(*low_measure).ok()?)?;
            match difference {
                Some(fixed) if fixed != step => return None,
                Some(_) => {}
                None => difference = Some(step),
            }
            moved = true;
        }
    }

    if moved {
        return difference;
    }
    // No adjacent pair on this axis. The coefficient is fixed at zero
    // only where the axis never varies in the table at all, because then
    // no measurement depends on it; otherwise the table has gaps this
    // model cannot bridge and there is no model.
    let values: Vec<i64> = measurements.keys().map(|shape| axis_of(*shape)).collect();
    let first = values.first()?;
    values.iter().all(|value| value == first).then_some(0)
}
