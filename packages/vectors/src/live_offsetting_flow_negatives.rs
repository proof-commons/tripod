//! The offsetting-flow negative ceremony: a balanced three-in three-out
//! candidate whose extra input-and-output pair passes consensus intact
//! and is refused by the coordinator's own cardinality clause.
//!
//! # The row this ceremony drives
//!
//! `second-offsetting-u-flow`. Its fault is an ADDED flow — one more
//! source and one more destination, of equal value in the closed asset —
//! carried alongside the flow the operation declares. Because the pair
//! offsets exactly, the per-asset sum still balances and consensus has
//! nothing to refuse; what refuses is the covenant's live cardinality
//! fragment, which inspects the input and output counts and requires
//! each to equal the number the operation committed to.
//!
//! # Why the separator is the SHAPE
//!
//! The cardinality clause answers with a plain equality failure, the
//! same words any count mismatch would draw, so the verdict is
//! program-generic and cannot separate this row from another count
//! fault. The transaction SHAPE can: a `(3, 3)` candidate is distinct
//! from every shape already driven — `(2, 1)`, `(2, 3)`, `(1, 2)` — and
//! from the two-in two-out control it is offered against.
//!
//! # Why a ceremony of its own
//!
//! The owner-signing negative ceremony funds exactly two receipts and
//! its leaf-arrangement mutants are two positions wide by construction.
//! Funding a third coin there would rebuild its successor and move the
//! control digest it has already recorded, so the third flow needs a
//! ceremony that funds three coins from the start.
