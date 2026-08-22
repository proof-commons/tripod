//! Statically specialized live-transfer transaction shapes (Guide-13
//! §5.1, §12.1, §12.2, §18.1).
//!
//! # Why the live transfer needs its own shape and not the compact-ASH
//! one
//!
//! The two operations disagree about both of the things a shape fixes.
//! Compact ASH aggregates, so [`crate::shape::MINIMUM_ASH_INPUTS`] is
//! two and one source is not a batch; a live transfer admits the unary
//! form, because §5.1 bounds its input count below by one and the
//! one-to-one transfer is the first case §18.2 measures. And compact ASH
//! creates exactly one successor, so its output count is derived from
//! the sponsor roles alone; a live transfer creates \(m\) destinations
//! and §5.3 admits split, merge, and redistribution, so the output count
//! is a second free axis rather than a constant.
//!
//! A shape type carrying both operations would therefore have to make
//! the output count optional and the input minimum conditional, and
//! every consumer would have to know which operation it was holding
//! before it could read either field. The two are separate types here
//! for the same reason the leaf roles of the two operations are: a value
//! that cannot be handed to the wrong emitter is a stronger statement
//! than a comment saying it should not be.
//!
//! # What a shape does not decide
//!
//! The same boundary [`crate::shape`] draws. A shape fixes how many
//! members each region has. It fixes no position beyond the ranges
//! §12.1 and §12.2 already fix by counting, names no owner, selects no
//! representation, and reports no amount: a sponsor count is a count of
//! *members*, never a subtotal, and §1.9 keeps individual sponsor values
//! out of every accessor here.
//!
//! # The bounds are a candidate assignment, not a semantic limit
//!
//! §18.1 lists the values the resource study is to evaluate and calls
//! them research candidates. Nothing here fixes one: the bounds are an
//! argument, a shape outside them is *unbuilt* rather than invalid, and
//! [`LiveShapeRejection::ReceiptInputsAboveBound`] says so in those
//! terms.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

use crate::shape::SponsorChangePresence;

/// The smallest receipt-input count a live transfer can have.
///
/// One, from §5.1's \(1\le n\). The transfer relation is stated over a
/// multiset on each side and has a perfectly good unary form, so the
/// aggregation argument that puts [`crate::shape::MINIMUM_ASH_INPUTS`]
/// at two does not apply here. Stated once so the validity rule and the
/// density audit read the same figure.
pub const MINIMUM_TRANSFER_RECEIPT_INPUTS: u8 = 1;

/// The smallest destination count a live transfer can have.
///
/// One, from §5.1's \(1\le m\). A transfer that created no destination
/// would destroy the value it consumed, and §5.6 forbids destruction on
/// the canonical flow, so the zero case is not a narrow transfer but a
/// different operation.
pub const MINIMUM_TRANSFER_RECEIPT_OUTPUTS: u8 = 1;

/// Why a proposed live-transfer shape is not a valid candidate shape.
///
/// One variant per way §5.1's cardinality relation and §12's layout can
/// fail together. They are distinguished because they call for different
/// corrections: a count above a bound is a caller asking for a
/// specialization that was not built, and change without a sponsor input
/// is a caller asking for a role with no region to sit in.
///
/// The two minima have no variant, and their absence is the point:
/// [`NonZeroU8`] already carries them, so a shape below either minimum
/// has no value to refuse rather than a refusal to report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveShapeRejection {
    /// More receipt inputs than the candidate bound admits.
    ReceiptInputsAboveBound {
        /// The count offered.
        offered: u8,
        /// The candidate receipt-input bound.
        bound: u8,
    },
    /// More receipt outputs than the candidate bound admits.
    ReceiptOutputsAboveBound {
        /// The count offered.
        offered: u8,
        /// The candidate receipt-output bound.
        bound: u8,
    },
    /// More sponsor inputs than the candidate bound admits.
    SponsorInputsAboveBound {
        /// The count offered.
        offered: u8,
        /// The candidate sponsor bound.
        bound: u8,
    },
    /// A sponsor-change role with no sponsor region to belong to.
    SponsorChangeWithoutSponsorInput,
    /// A candidate set holding no shape at all.
    ///
    /// A candidate is the shapes it emits programs for, so a set with
    /// none of them is not a narrow candidate but an absent one.
    EmptyShapeSet,
    /// A set declaring a sparse count range whose counts are dense.
    ///
    /// The declaration is what turns a gap from an omission into a
    /// reported limitation, so it has to report something. A set that
    /// claimed the limitation and then carried none would be inaccurate
    /// in the other direction from the silent gap.
    DenseSetDeclaredSparse,
}

/// The finite bounds one live-transfer candidate is specialized over
/// (§18.1).
///
/// Three axes, because §18.1 enumerates three: `TRANSFER_INPUT_MAX`,
/// `TRANSFER_OUTPUT_MAX`, and `FEE_SPONSOR_INPUT_MAX`. Each is a
/// backend decision about what to emit programs for; the architecture
/// owns the cardinality relation itself, and the compiler's validated
/// plan publishes it.
///
/// Construction is total, unlike [`crate::shape::CompactAshShapeBounds`]:
/// the receipt minima are one, [`NonZeroU8`] carries that, and a bound
/// of one admits exactly the one-to-one shape rather than nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveTransferShapeBounds {
    receipt_inputs: NonZeroU8,
    receipt_outputs: NonZeroU8,
    sponsor_inputs: u8,
}

impl LiveTransferShapeBounds {
    /// Bounds admitting up to these receipt and sponsor counts.
    #[must_use]
    pub const fn new(
        receipt_inputs: NonZeroU8,
        receipt_outputs: NonZeroU8,
        sponsor_inputs: u8,
    ) -> Self {
        Self {
            receipt_inputs,
            receipt_outputs,
            sponsor_inputs,
        }
    }

    /// The largest receipt-input count these bounds admit.
    #[must_use]
    pub const fn receipt_inputs(self) -> u8 {
        self.receipt_inputs.get()
    }

    /// The largest destination count these bounds admit.
    #[must_use]
    pub const fn receipt_outputs(self) -> u8 {
        self.receipt_outputs.get()
    }

    /// The largest sponsor region these bounds admit.
    #[must_use]
    pub const fn sponsor_inputs(self) -> u8 {
        self.sponsor_inputs
    }
}

/// One statically specialized live-transfer shape (§5.1, §12).
///
/// The fields are private and the constructor checks the validity
/// condition, so an inadmissible shape does not exist to be emitted for.
/// The accessors report counts of members and index ranges, and nothing
/// else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveTransferShape {
    receipt_inputs: NonZeroU8,
    receipt_outputs: NonZeroU8,
    sponsor_inputs: u8,
    sponsor_change: SponsorChangePresence,
}

impl LiveTransferShape {
    /// The shape with these counts, if the candidate bounds admit it.
    ///
    /// # Errors
    ///
    /// [`LiveShapeRejection::ReceiptInputsAboveBound`],
    /// [`LiveShapeRejection::ReceiptOutputsAboveBound`],
    /// [`LiveShapeRejection::SponsorInputsAboveBound`], or
    /// [`LiveShapeRejection::SponsorChangeWithoutSponsorInput`], one per
    /// conjunct of the validity condition.
    pub const fn new(
        bounds: LiveTransferShapeBounds,
        receipt_inputs: NonZeroU8,
        receipt_outputs: NonZeroU8,
        sponsor_inputs: u8,
        sponsor_change: SponsorChangePresence,
    ) -> Result<Self, LiveShapeRejection> {
        if receipt_inputs.get() > bounds.receipt_inputs() {
            return Err(LiveShapeRejection::ReceiptInputsAboveBound {
                offered: receipt_inputs.get(),
                bound: bounds.receipt_inputs(),
            });
        }
        if receipt_outputs.get() > bounds.receipt_outputs() {
            return Err(LiveShapeRejection::ReceiptOutputsAboveBound {
                offered: receipt_outputs.get(),
                bound: bounds.receipt_outputs(),
            });
        }
        if sponsor_inputs > bounds.sponsor_inputs() {
            return Err(LiveShapeRejection::SponsorInputsAboveBound {
                offered: sponsor_inputs,
                bound: bounds.sponsor_inputs(),
            });
        }
        if matches!(sponsor_change, SponsorChangePresence::Present) && sponsor_inputs == 0 {
            return Err(LiveShapeRejection::SponsorChangeWithoutSponsorInput);
        }
        Ok(Self {
            receipt_inputs,
            receipt_outputs,
            sponsor_inputs,
            sponsor_change,
        })
    }

    /// How many live receipts this shape consumes.
    #[must_use]
    pub const fn receipt_inputs(self) -> u8 {
        self.receipt_inputs.get()
    }

    /// How many live receipts this shape creates.
    #[must_use]
    pub const fn receipt_outputs(self) -> u8 {
        self.receipt_outputs.get()
    }

    /// How many sponsor inputs this shape has.
    #[must_use]
    pub const fn sponsor_inputs(self) -> u8 {
        self.sponsor_inputs
    }

    /// Whether this shape declares the sponsor-change role.
    #[must_use]
    pub const fn sponsor_change(self) -> SponsorChangePresence {
        self.sponsor_change
    }

    /// Whether this shape carries a sponsor region at all.
    ///
    /// Region membership, and never an amount (§1.9).
    #[must_use]
    pub const fn sponsored(self) -> bool {
        self.sponsor_inputs > 0
    }

    /// The total input count this shape fixes.
    ///
    /// Exact rather than a lower bound: §12.1's layout is the receipt
    /// range followed by the sponsor suffix and nothing else, so the sum
    /// is the whole transaction's input count and the coordinator can
    /// authenticate the target's own count against it (§10.3).
    #[must_use]
    pub const fn inputs(self) -> u16 {
        self.receipt_inputs.get() as u16 + self.sponsor_inputs as u16
    }

    /// The total output count this shape fixes.
    ///
    /// Destinations first, then the sponsor-change role where the shape
    /// declares one, then the target fee role where the form carries
    /// one. A sponsorless shape pays no fee and therefore carries no fee
    /// output at all: the reviewed target represents a zero fee by the
    /// *absence* of the output and refuses a zero-valued one, which is
    /// the same reading [`crate::shape::CompactAshShape::outputs`] takes
    /// and for the same reason.
    #[must_use]
    pub const fn outputs(self) -> u16 {
        let change = match self.sponsor_change {
            SponsorChangePresence::Absent => 0,
            SponsorChangePresence::Present => 1,
        };
        let fee = if self.sponsored() { 1 } else { 0 };
        self.receipt_outputs.get() as u16 + change + fee
    }

    /// The half-open receipt-input range, as exact indices.
    ///
    /// `0..receipt_inputs`: §12.1 puts the receipt family first, and
    /// input 0 is the coordinator (§10.3).
    #[must_use]
    pub const fn receipt_input_range(self) -> (u16, u16) {
        (0, self.receipt_inputs.get() as u16)
    }

    /// The half-open sponsor suffix, as exact indices.
    ///
    /// Empty for a sponsorless shape, where both ends are the receipt
    /// count — the exact suffix start and the exact suffix length the
    /// coordinator authenticates even when there is nothing in it
    /// (§10.7).
    ///
    /// # Why an index is a `u16` where a count is a `u8`
    ///
    /// The two are different domains. A count is a bound this candidate
    /// chose, and §18.1's research candidates keep those small on
    /// purpose. An index is a position in the target's own transaction,
    /// and §18.1 requires the position domain to be one that cannot
    /// overflow an accepted shape. It cannot: the largest sum two `u8`
    /// counts can reach is 510, which `u16` holds with room, so this
    /// accessor neither wraps nor panics in any build profile.
    #[must_use]
    pub const fn sponsor_range(self) -> (u16, u16) {
        (
            self.receipt_inputs.get() as u16,
            self.receipt_inputs.get() as u16 + self.sponsor_inputs as u16,
        )
    }

    /// The half-open destination range, as exact indices.
    ///
    /// `0..receipt_outputs`, from §12.2. The destinations are in typed
    /// request order, which is presentation order and not semantic
    /// identity: the range says how many positions the family occupies,
    /// and says nothing about which destination is which.
    #[must_use]
    pub const fn destination_range(self) -> (u16, u16) {
        (0, self.receipt_outputs.get() as u16)
    }
}

/// The shapes one live-transfer candidate emits programs for.
///
/// The set is exact and finite, and is not a bound with a membership
/// rule: a derived test would let a shape be believed supported because
/// it satisfies an inequality rather than because a program exists for
/// it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiveTransferShapeSet {
    bounds: LiveTransferShapeBounds,
    shapes: BTreeSet<LiveTransferShape>,
    sparse_counts_declared: bool,
}

impl LiveTransferShapeSet {
    /// A candidate set over `bounds` containing exactly `shapes`.
    ///
    /// `sparse_counts_declared` states up front whether the candidate
    /// intends to support only some of the counts in range. A sparse set
    /// is admitted only where the candidate says so, and a set that
    /// quietly skipped a count while claiming density would be the
    /// silent gap the declaration exists to refuse — so the declaration
    /// is an input here rather than a conclusion drawn from the members.
    ///
    /// # What the bounds mean once a member disagrees with them
    ///
    /// A member outside the bounds is not a wider candidate. The two
    /// statements are read by different consumers — a linker resolves
    /// the advertised bound from [`Self::bounds`] while an emitter
    /// iterates [`Self::shapes`] — so a set holding both would advertise
    /// one window and emit programs for another. Each member is
    /// therefore checked against this set's own bounds, and not merely
    /// against whichever bounds it happened to be built under.
    ///
    /// # Errors
    ///
    /// [`LiveShapeRejection::ReceiptInputsAboveBound`],
    /// [`LiveShapeRejection::ReceiptOutputsAboveBound`], or
    /// [`LiveShapeRejection::SponsorInputsAboveBound`] for a member
    /// outside this set's window;
    /// [`LiveShapeRejection::EmptyShapeSet`] for a candidate with
    /// nothing to emit; and
    /// [`LiveShapeRejection::DenseSetDeclaredSparse`] for a declaration
    /// that reports a limitation the members do not carry.
    pub fn new(
        bounds: LiveTransferShapeBounds,
        shapes: BTreeSet<LiveTransferShape>,
        sparse_counts_declared: bool,
    ) -> Result<Self, LiveShapeRejection> {
        if shapes.is_empty() {
            return Err(LiveShapeRejection::EmptyShapeSet);
        }
        for shape in &shapes {
            if shape.receipt_inputs() > bounds.receipt_inputs() {
                return Err(LiveShapeRejection::ReceiptInputsAboveBound {
                    offered: shape.receipt_inputs(),
                    bound: bounds.receipt_inputs(),
                });
            }
            if shape.receipt_outputs() > bounds.receipt_outputs() {
                return Err(LiveShapeRejection::ReceiptOutputsAboveBound {
                    offered: shape.receipt_outputs(),
                    bound: bounds.receipt_outputs(),
                });
            }
            if shape.sponsor_inputs() > bounds.sponsor_inputs() {
                return Err(LiveShapeRejection::SponsorInputsAboveBound {
                    offered: shape.sponsor_inputs(),
                    bound: bounds.sponsor_inputs(),
                });
            }
        }
        if sparse_counts_declared && !has_count_gap(&shapes, bounds) {
            return Err(LiveShapeRejection::DenseSetDeclaredSparse);
        }
        Ok(Self {
            bounds,
            shapes,
            sparse_counts_declared,
        })
    }

    /// The bounds this set specializes over.
    #[must_use]
    pub const fn bounds(&self) -> LiveTransferShapeBounds {
        self.bounds
    }

    /// Every admitted shape, in canonical order.
    pub fn shapes(&self) -> impl Iterator<Item = LiveTransferShape> + '_ {
        self.shapes.iter().copied()
    }

    /// How many shapes this candidate emits programs for.
    #[must_use]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Whether the set is empty, which no constructed set is.
    ///
    /// Present because [`Self::len`] is, and answering it from the
    /// members rather than from a remembered flag keeps the two from
    /// disagreeing. [`Self::new`] refuses the empty set, so this is
    /// always false for a value that exists.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Whether the candidate declared a sparse supported-count set.
    #[must_use]
    pub const fn sparse_counts_declared(&self) -> bool {
        self.sparse_counts_declared
    }

    /// Whether this exact shape has a program in this candidate.
    #[must_use]
    pub fn admits(&self, shape: LiveTransferShape) -> bool {
        self.shapes.contains(&shape)
    }

    /// Every receipt-input count some admitted shape carries.
    ///
    /// The member leaf of §10.3 is a function of this count alone, so
    /// this is exactly the census of member leaves the candidate needs.
    #[must_use]
    pub fn receipt_input_counts(&self) -> BTreeSet<u8> {
        self.shapes
            .iter()
            .map(|shape| shape.receipt_inputs())
            .collect()
    }
}

/// Whether some receipt-input count the bounds admit has no shape
/// carrying it.
///
/// Stated once and read twice, by the two places a gap has a meaning:
/// [`LiveTransferShapeSet::new`] refuses a *sparsity declaration* made
/// over no gap at all, and a candidate audit reports an *undeclared*
/// gap. Written out separately the two could drift into disagreeing
/// about what a gap is, and then a set could be refused for having none
/// while the audit reported one.
fn has_count_gap(shapes: &BTreeSet<LiveTransferShape>, bounds: LiveTransferShapeBounds) -> bool {
    let inputs = shapes
        .iter()
        .map(|shape| shape.receipt_inputs())
        .collect::<BTreeSet<_>>();
    let outputs = shapes
        .iter()
        .map(|shape| shape.receipt_outputs())
        .collect::<BTreeSet<_>>();

    (MINIMUM_TRANSFER_RECEIPT_INPUTS..=bounds.receipt_inputs())
        .any(|count| !inputs.contains(&count))
        || (MINIMUM_TRANSFER_RECEIPT_OUTPUTS..=bounds.receipt_outputs())
            .any(|count| !outputs.contains(&count))
}

/// The complete unrolling of one live-transfer bound assignment.
///
/// Every shape the validity condition admits under `bounds` and nothing
/// else: each receipt-input count from
/// [`MINIMUM_TRANSFER_RECEIPT_INPUTS`] through the input bound, each
/// destination count from [`MINIMUM_TRANSFER_RECEIPT_OUTPUTS`] through
/// the output bound, each sponsor count from none through the sponsor
/// bound, and both change presences wherever a sponsor region exists to
/// carry one. The result declares itself dense, because it is.
///
/// This is the one authored unrolling. A resource study that measured
/// bound assignments by rebuilding the shape set itself would be
/// comparing sets two different loops had produced, and a disagreement
/// between the loops would read as a measurement (§18.4).
///
/// # Panics
///
/// If the set this loop builds is one [`LiveTransferShapeSet::new`]
/// refuses, which is a property of the loop rather than of `bounds`:
/// every member is built against these same bounds and so is inside
/// them, both receipt bounds are nonzero and so at least the one-to-one
/// shape exists, and the result declares itself dense, which it is. A
/// panic here would mean this loop had stopped agreeing with the
/// validity rule, and it must fail loudly rather than hand back a set
/// that disagrees with its own bounds.
#[must_use]
pub fn dense_live_shape_set(bounds: LiveTransferShapeBounds) -> LiveTransferShapeSet {
    let mut shapes = BTreeSet::new();

    for inputs in MINIMUM_TRANSFER_RECEIPT_INPUTS..=bounds.receipt_inputs() {
        for outputs in MINIMUM_TRANSFER_RECEIPT_OUTPUTS..=bounds.receipt_outputs() {
            for sponsors in 0..=bounds.sponsor_inputs() {
                for change in SponsorChangePresence::ALL {
                    // The one combination the validity condition refuses
                    // inside these bounds: a change role with no sponsor
                    // region to sit in. Asking and discarding the
                    // refusal keeps the rule in one place rather than
                    // restating it as a loop guard.
                    if let Ok(shape) = LiveTransferShape::new(
                        bounds,
                        nonzero(inputs),
                        nonzero(outputs),
                        sponsors,
                        *change,
                    ) {
                        shapes.insert(shape);
                    }
                }
            }
        }
    }

    LiveTransferShapeSet::new(bounds, shapes, false)
        .expect("the dense unrolling of a bound assignment is admissible under it")
}

/// The Phase-5 demonstration live-transfer shape set.
///
/// Up to three receipts consumed and three created, sponsorless and
/// one-sponsor forms of each, and both change presences for the
/// sponsored forms. The window is deliberately small: it already
/// contains every §5.3 composition — one-to-one, split, merge, and
/// many-to-many redistribution — which is the set §18.2 measures first,
/// and a wider candidate would be claiming a size nobody had sized.
/// Widening it is a bound change and a measurement, not a semantic
/// change, which is the study §18.1 asks for.
#[must_use]
pub fn demonstration_live_shape_set() -> LiveTransferShapeSet {
    dense_live_shape_set(LiveTransferShapeBounds::new(nonzero(3), nonzero(3), 1))
}

/// A nonzero count, for the loops and constants above.
///
/// Every caller passes a value the surrounding code has already bounded
/// below by one, so the zero case is unreachable. It nonetheless yields
/// the smallest nonzero count rather than panicking, which is the
/// one-to-one shape and the narrowest thing this module can mean.
const fn nonzero(value: u8) -> NonZeroU8 {
    match NonZeroU8::new(value) {
        Some(count) => count,
        None => NonZeroU8::MIN,
    }
}
