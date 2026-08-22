//! Statically specialized compact-ASH transaction shapes (Guide-12 §9).
//!
//! # Why a shape is a typed key rather than a bound
//!
//! The reviewed target has no general loop primitive, and Guide 12
//! assumes no general conditional dispatch. A backend therefore cannot
//! emit one program that handles "between two and eight ASH inputs": it
//! emits one program per shape, and the shape is what selects the
//! program. §9.1 makes that explicit rather than hiding a finite
//! unrolling behind target-independent prose, so the key that selects a
//! specialization is a value here, not a comment.
//!
//! # What a shape does not decide
//!
//! A shape fixes how many members of each region a transaction has. It
//! does not fix which inputs they are, which leaf spends them, which
//! output carries what, or any position: those belong to the candidate
//! ABI and the linker, further down. Nor does it name an amount of
//! anything — a sponsor count is a count of *members*, never a subtotal,
//! and no accessor here reports a value (§1.6).
//!
//! # Every invariant is a refusal, not an assertion
//!
//! [`CompactAshShape::new`] returns a typed refusal for each of the four
//! ways §9.1's validity condition can fail, so an inadmissible shape has
//! no value at all. A constructor that clamped a count into range would
//! be the weakening §8.3 forbids: it would answer a question about a
//! shape nobody asked for.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

/// The smallest ASH batch a compact-ASH operation can have.
///
/// Two, because the operation aggregates: one source is not a batch,
/// and the semantic contract has no unary form to fall back on. Stated
/// once here so the validity rule and the candidate-set audit read the
/// same figure.
pub const MINIMUM_ASH_INPUTS: u8 = 2;

/// Whether a shape carries the optional sponsor-change role.
///
/// A named pair rather than a Boolean: §10.5 recognizes the role by
/// declared role, canonical position, reserve asset, and admitted
/// program class, and never by comparing an amount with zero. A field
/// called `has_change: bool` invites exactly the amount test the rule
/// forbids, because a reader reaches for the cheapest way to compute a
/// Boolean.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SponsorChangePresence {
    /// The shape declares no sponsor-change role.
    Absent,
    /// The shape declares exactly one sponsor-change role.
    Present,
}

impl SponsorChangePresence {
    /// Both forms, in canonical order.
    pub const ALL: &'static [Self] = &[Self::Absent, Self::Present];
}

/// Why a proposed shape is not a valid candidate shape.
///
/// One variant per conjunct of §9.1's validity condition. They are
/// distinguished because they call for different corrections: a batch
/// below the minimum is a caller asking for an operation that does not
/// exist, a count above a bound is a caller asking for a
/// specialization that was not built, and change without a sponsor
/// input is a caller asking for a role with no region to sit in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShapeRejection {
    /// Fewer ASH inputs than the operation's own minimum.
    AshInputsBelowMinimum {
        /// The count offered.
        offered: u8,
        /// The minimum the operation requires.
        minimum: u8,
    },
    /// More ASH inputs than the candidate bound admits.
    AshInputsAboveBound {
        /// The count offered.
        offered: u8,
        /// The candidate ASH bound.
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
    /// none of them is not a narrow candidate but an absent one. It is
    /// refused here rather than downstream because every consumer would
    /// otherwise have to decide separately what an empty candidate
    /// means, and the honest answer is that there is nothing to decide.
    EmptyShapeSet,
    /// A set declaring a sparse count range whose counts are dense.
    ///
    /// The declaration is what §9.3 turns a gap from an omission into a
    /// reported limitation with, so it reports something. A set that
    /// claimed the limitation and then had none was reporting a
    /// restriction it does not carry, which is as inaccurate as the
    /// silent gap the rule refuses — in the other direction.
    DenseSetDeclaredSparse,
}

/// The finite bounds one candidate shape family is specialized over.
///
/// The bounds are a backend decision, not a semantic one: architecture
/// and realization own the cardinality relation, and this is the finite
/// window a particular candidate chose to emit programs for. A shape
/// outside it is not semantically invalid — it is unbuilt, and
/// [`ShapeRejection::AshInputsAboveBound`] says so in those terms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompactAshShapeBounds {
    ash_inputs: NonZeroU8,
    sponsor_inputs: u8,
}

impl CompactAshShapeBounds {
    /// Bounds admitting ASH batches up to `ash_inputs` and sponsor
    /// regions up to `sponsor_inputs`.
    ///
    /// # Errors
    ///
    /// [`ShapeRejection::AshInputsBelowMinimum`] when the ASH bound is
    /// itself below the operation's minimum, which would admit no shape
    /// at all.
    pub const fn new(ash_inputs: NonZeroU8, sponsor_inputs: u8) -> Result<Self, ShapeRejection> {
        if ash_inputs.get() < MINIMUM_ASH_INPUTS {
            return Err(ShapeRejection::AshInputsBelowMinimum {
                offered: ash_inputs.get(),
                minimum: MINIMUM_ASH_INPUTS,
            });
        }
        Ok(Self {
            ash_inputs,
            sponsor_inputs,
        })
    }

    /// The largest ASH batch these bounds admit.
    #[must_use]
    pub const fn ash_inputs(self) -> u8 {
        self.ash_inputs.get()
    }

    /// The largest sponsor region these bounds admit.
    #[must_use]
    pub const fn sponsor_inputs(self) -> u8 {
        self.sponsor_inputs
    }
}

/// One statically specialized transaction shape (§9.1).
///
/// The fields are private and the constructor checks §9.1's condition,
/// so an invalid shape does not exist to be emitted for. The accessors
/// report counts of members and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompactAshShape {
    ash_inputs: NonZeroU8,
    sponsor_inputs: u8,
    sponsor_change: SponsorChangePresence,
}

impl CompactAshShape {
    /// The shape with these counts, if §9.1 admits it.
    ///
    /// # Errors
    ///
    /// [`ShapeRejection::AshInputsBelowMinimum`],
    /// [`ShapeRejection::AshInputsAboveBound`],
    /// [`ShapeRejection::SponsorInputsAboveBound`], or
    /// [`ShapeRejection::SponsorChangeWithoutSponsorInput`], one per
    /// conjunct of the validity condition.
    pub const fn new(
        bounds: CompactAshShapeBounds,
        ash_inputs: NonZeroU8,
        sponsor_inputs: u8,
        sponsor_change: SponsorChangePresence,
    ) -> Result<Self, ShapeRejection> {
        if ash_inputs.get() < MINIMUM_ASH_INPUTS {
            return Err(ShapeRejection::AshInputsBelowMinimum {
                offered: ash_inputs.get(),
                minimum: MINIMUM_ASH_INPUTS,
            });
        }
        if ash_inputs.get() > bounds.ash_inputs() {
            return Err(ShapeRejection::AshInputsAboveBound {
                offered: ash_inputs.get(),
                bound: bounds.ash_inputs(),
            });
        }
        if sponsor_inputs > bounds.sponsor_inputs() {
            return Err(ShapeRejection::SponsorInputsAboveBound {
                offered: sponsor_inputs,
                bound: bounds.sponsor_inputs(),
            });
        }
        if matches!(sponsor_change, SponsorChangePresence::Present) && sponsor_inputs == 0 {
            return Err(ShapeRejection::SponsorChangeWithoutSponsorInput);
        }
        Ok(Self {
            ash_inputs,
            sponsor_inputs,
            sponsor_change,
        })
    }

    /// How many ASH inputs this shape has.
    #[must_use]
    pub const fn ash_inputs(self) -> u8 {
        self.ash_inputs.get()
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
    /// Region membership, which §1.6 admits, and never an amount.
    #[must_use]
    pub const fn sponsored(self) -> bool {
        self.sponsor_inputs > 0
    }

    /// The total input count this shape fixes.
    ///
    /// The figure §12.3 requires the coordinator to authenticate
    /// against the target's own input count. It is exact: the candidate
    /// layout of §10.1 is the ASH range followed by the sponsor suffix
    /// and nothing else, so the sum is the whole transaction's input
    /// count rather than a lower bound on it.
    #[must_use]
    pub const fn inputs(self) -> u16 {
        self.ash_inputs.get() as u16 + self.sponsor_inputs as u16
    }

    /// The total output count this shape fixes.
    ///
    /// Output 0 is the successor ASH; a sponsor-change role follows it
    /// where the shape declares one; and the target fee role follows
    /// that where the selected form carries one. A sponsorless shape
    /// pays no fee and therefore carries no fee output at all, because
    /// the reviewed target represents a zero fee by the *absence* of the
    /// output and refuses a zero-valued one
    /// (`(´[PLAN-rule:guide12-exec:fee-role]´)`, and the reviewed
    /// contract's own refused spelling).
    #[must_use]
    pub const fn outputs(self) -> u16 {
        let change = match self.sponsor_change {
            SponsorChangePresence::Absent => 0,
            SponsorChangePresence::Present => 1,
        };
        let fee = if self.sponsored() { 1 } else { 0 };
        1 + change + fee
    }

    /// The half-open ASH input range, as exact indices.
    ///
    /// `0..ash_inputs`: §10.1 puts the ASH family first, and input 0 is
    /// the canonical coordinator. The indices are input positions, so
    /// they are in the domain [`Self::sponsor_range`] describes.
    #[must_use]
    pub const fn ash_range(self) -> (u16, u16) {
        (0, self.ash_inputs.get() as u16)
    }

    /// The half-open sponsor suffix, as exact indices.
    ///
    /// Empty for a sponsorless shape, where both ends are the ASH
    /// count — which is the exact suffix start and the exact suffix
    /// length §10.4 has the coordinator authenticate even when there is
    /// nothing in it.
    ///
    /// # Why an index is a `u16` where a count is a `u8`
    ///
    /// The two are different domains, and the difference is not
    /// cosmetic. A count is a bound this candidate chose, and §9.2
    /// keeps those small on purpose. An index is a position in the
    /// target's own transaction, and every consumer already reads one
    /// as a `u16`: [`Self::inputs`] reports the total that way, the
    /// concrete layout places every region that way, and the candidate
    /// ABI names both ranges that way. Deriving the suffix end in the
    /// count's domain instead made the widest admissible shape wrap its
    /// own last index to zero, which is the one piece of arithmetic a
    /// range accessor may not perform.
    ///
    /// Here the sum of two `u8` counts is exact for every shape
    /// [`Self::new`] admits — the largest is 510, which the domain
    /// holds — so the accessor is total: it neither wraps nor panics,
    /// in any build profile.
    #[must_use]
    pub const fn sponsor_range(self) -> (u16, u16) {
        (
            self.ash_inputs.get() as u16,
            self.ash_inputs.get() as u16 + self.sponsor_inputs as u16,
        )
    }
}

/// Which §9.3 condition a candidate shape set does or does not meet.
///
/// A candidate that supports only the minimum batch is useful as a
/// first wave and insufficient for Phase-4 exit, so the audit reports
/// each condition separately rather than answering "useful" with one
/// Boolean that could not say what was missing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UsefulCandidateCondition {
    /// At least one batch larger than the minimum.
    BatchAboveMinimum,
    /// At least one sponsored shape.
    SponsoredShape,
    /// Sponsor change present in some admitted shape.
    SponsorChangePresent,
    /// Sponsor change absent in some admitted shape.
    SponsorChangeAbsent,
    /// Every count from the minimum through the ASH bound.
    DenseAshCounts,
}

impl UsefulCandidateCondition {
    /// The complete census of §9.3 conditions, in canonical order.
    pub const ALL: &'static [Self] = &[
        Self::BatchAboveMinimum,
        Self::SponsoredShape,
        Self::SponsorChangePresent,
        Self::SponsorChangeAbsent,
        Self::DenseAshCounts,
    ];
}

/// The shapes one candidate emits programs for, and what they cover.
///
/// The set is exact and finite. It is not a bound with a rule for
/// deriving members: §9.2 requires the unrolling to be visible, and a
/// derived membership test would let a shape be believed supported
/// because it satisfies an inequality rather than because a program
/// exists for it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CandidateShapeSet {
    bounds: CompactAshShapeBounds,
    shapes: BTreeSet<CompactAshShape>,
    sparse_counts_declared: bool,
}

impl CandidateShapeSet {
    /// A candidate set over `bounds` containing exactly `shapes`.
    ///
    /// `sparse_counts_declared` states, up front, whether the candidate
    /// intends to support only some of the counts in range. §9.3 admits
    /// a sparse set only where the candidate says so and reports the
    /// limitation; a set that quietly skipped a count while claiming
    /// density would be the silent gap the rule exists to refuse, so the
    /// declaration is an input and [`Self::unmet_conditions`] reads it.
    ///
    /// # What the bounds mean once a member disagrees with them
    ///
    /// A member outside the bounds is not a wider candidate. The two
    /// statements are read by different consumers — a linker resolves
    /// the advertised bound from [`Self::bounds`] while an emitter
    /// iterates [`Self::shapes`] — so a set holding both would advertise
    /// one window and emit programs for another, and nothing between
    /// them ever compares the two. Each member is therefore checked
    /// against this set's own bounds here, and not merely against
    /// whichever bounds it happened to be built under.
    ///
    /// # Errors
    ///
    /// [`ShapeRejection::AshInputsAboveBound`] or
    /// [`ShapeRejection::SponsorInputsAboveBound`] for a member outside
    /// this set's window; [`ShapeRejection::EmptyShapeSet`] for a
    /// candidate with nothing to emit; and
    /// [`ShapeRejection::DenseSetDeclaredSparse`] for a declaration that
    /// reports a limitation the members do not carry.
    pub fn new(
        bounds: CompactAshShapeBounds,
        shapes: BTreeSet<CompactAshShape>,
        sparse_counts_declared: bool,
    ) -> Result<Self, ShapeRejection> {
        if shapes.is_empty() {
            return Err(ShapeRejection::EmptyShapeSet);
        }
        for shape in &shapes {
            if shape.ash_inputs() > bounds.ash_inputs() {
                return Err(ShapeRejection::AshInputsAboveBound {
                    offered: shape.ash_inputs(),
                    bound: bounds.ash_inputs(),
                });
            }
            if shape.sponsor_inputs() > bounds.sponsor_inputs() {
                return Err(ShapeRejection::SponsorInputsAboveBound {
                    offered: shape.sponsor_inputs(),
                    bound: bounds.sponsor_inputs(),
                });
            }
        }
        if sparse_counts_declared && !has_count_gap(&shapes, bounds) {
            return Err(ShapeRejection::DenseSetDeclaredSparse);
        }
        Ok(Self {
            bounds,
            shapes,
            sparse_counts_declared,
        })
    }

    /// The bounds this set specializes over.
    #[must_use]
    pub const fn bounds(&self) -> CompactAshShapeBounds {
        self.bounds
    }

    /// Every admitted shape, in canonical order.
    pub fn shapes(&self) -> impl Iterator<Item = CompactAshShape> + '_ {
        self.shapes.iter().copied()
    }

    /// Whether the candidate declared a sparse supported-count set.
    #[must_use]
    pub const fn sparse_counts_declared(&self) -> bool {
        self.sparse_counts_declared
    }

    /// Whether this exact shape has a program in this candidate.
    #[must_use]
    pub fn admits(&self, shape: CompactAshShape) -> bool {
        self.shapes.contains(&shape)
    }

    /// The §9.3 conditions this set does not meet, in canonical order.
    ///
    /// Empty for a set that meets all five. The dense-count condition
    /// is reported as met when the candidate declared its count set
    /// sparse, because §9.3 admits that case explicitly — the
    /// declaration is what turns a gap from an omission into a reported
    /// limitation, and [`Self::sparse_counts_declared`] keeps it
    /// readable rather than hidden inside this answer.
    #[must_use]
    pub fn unmet_conditions(&self) -> BTreeSet<UsefulCandidateCondition> {
        let mut unmet = BTreeSet::new();

        if !self
            .shapes
            .iter()
            .any(|shape| shape.ash_inputs() > MINIMUM_ASH_INPUTS)
        {
            unmet.insert(UsefulCandidateCondition::BatchAboveMinimum);
        }
        if !self.shapes.iter().any(|shape| shape.sponsored()) {
            unmet.insert(UsefulCandidateCondition::SponsoredShape);
        }
        for (presence, condition) in [
            (
                SponsorChangePresence::Present,
                UsefulCandidateCondition::SponsorChangePresent,
            ),
            (
                SponsorChangePresence::Absent,
                UsefulCandidateCondition::SponsorChangeAbsent,
            ),
        ] {
            if !self
                .shapes
                .iter()
                .any(|shape| shape.sponsor_change() == presence)
            {
                unmet.insert(condition);
            }
        }

        if !self.sparse_counts_declared && has_count_gap(&self.shapes, self.bounds) {
            unmet.insert(UsefulCandidateCondition::DenseAshCounts);
        }

        unmet
    }
}

/// Whether some ASH count the bounds admit has no shape carrying it.
///
/// Stated once and read twice, by the two places §9.3 gives a gap a
/// meaning: [`CandidateShapeSet::new`] refuses a *sparsity declaration*
/// made over no gap at all, and [`CandidateShapeSet::unmet_conditions`]
/// reports an *undeclared* gap as the unmet density condition. Written
/// out separately the two could drift into disagreeing about what a gap
/// is, and then a set could be refused for having none while the audit
/// reported one.
fn has_count_gap(shapes: &BTreeSet<CompactAshShape>, bounds: CompactAshShapeBounds) -> bool {
    let counts = shapes
        .iter()
        .map(|shape| shape.ash_inputs())
        .collect::<BTreeSet<_>>();
    (MINIMUM_ASH_INPUTS..=bounds.ash_inputs()).any(|count| !counts.contains(&count))
}

/// The complete unrolling of one candidate bound assignment.
///
/// Every shape §9.1 admits under `bounds`, and nothing else: each ASH
/// count from [`MINIMUM_ASH_INPUTS`] through the ASH bound, each
/// sponsor count from none through the sponsor bound, and both change
/// presences wherever a sponsor region exists to carry one. The result
/// declares itself dense, because it is.
///
/// This is the one authored unrolling (§1.12). A study that measured
/// bound assignments by rebuilding the shape set itself would be
/// comparing sets that two different loops had produced, and a
/// disagreement between the loops would read as a measurement.
///
/// # Panics
///
/// If the set this loop builds is one [`CandidateShapeSet::new`]
/// refuses, which is a property of the loop rather than of `bounds`:
/// every member is built against these same bounds and so is inside
/// them, the ASH bound is at least [`MINIMUM_ASH_INPUTS`] and so at
/// least one member exists, and the result declares itself dense, which
/// it is. A panic here would mean this loop had stopped agreeing with
/// the validity rule, and it must fail loudly rather than hand back a
/// set that disagrees with its own bounds.
#[must_use]
pub fn dense_shape_set(bounds: CompactAshShapeBounds) -> CandidateShapeSet {
    let mut shapes = BTreeSet::new();

    for ash in MINIMUM_ASH_INPUTS..=bounds.ash_inputs() {
        for sponsors in 0..=bounds.sponsor_inputs() {
            for change in SponsorChangePresence::ALL {
                // The one combination §9.1 refuses inside these bounds:
                // a change role with no sponsor region to sit in. Asking
                // and discarding the refusal keeps the validity rule in
                // one place rather than restating it as a loop guard.
                if let Ok(shape) = CompactAshShape::new(bounds, nonzero(ash), sponsors, *change) {
                    shapes.insert(shape);
                }
            }
        }
    }

    CandidateShapeSet::new(bounds, shapes, false)
        .expect("the dense unrolling of a bound assignment is admissible under it")
}

/// The Phase-4 demonstration candidate shape set.
///
/// Batches of two through four ASH inputs; sponsorless and one-sponsor
/// forms of each; and, for the sponsored forms, both change presences.
/// The bounds are deliberately small: §9.2 lists what shape
/// specialization costs in leaves, bytes, depth, control, witness, and
/// weight, and a wider candidate would be claiming a size nobody had
/// sized. Widening it is a bound change and a measurement, not a
/// semantic change, which is the study §20 asks for.
///
/// # Panics
///
/// If the ASH bound written here ever stops satisfying §9.1's validity
/// condition. It is a constant, so that is a property of this
/// function's own source rather than of any input, and a panic would
/// mean the demonstration bounds had been edited below the minimum
/// batch — which must fail loudly rather than yield an empty set.
#[must_use]
pub fn demonstration_shape_set() -> CandidateShapeSet {
    let bounds = CompactAshShapeBounds::new(nonzero(4), 1).expect("four is above the minimum");
    dense_shape_set(bounds)
}

/// A nonzero count, for the constants above.
///
/// Every caller passes a literal the surrounding code has already
/// bounded below by [`MINIMUM_ASH_INPUTS`], so the zero case is
/// unreachable. It nonetheless yields the smallest nonzero count
/// rather than panicking, and that count is *below* the minimum batch,
/// so a caller who somehow reached it is refused by
/// [`CompactAshShape::new`] instead of quietly admitted.
const fn nonzero(value: u8) -> NonZeroU8 {
    match NonZeroU8::new(value) {
        Some(count) => count,
        None => NonZeroU8::MIN,
    }
}
