//! The split-commitment negative ceremony: a confidential one-in
//! three-out successor whose mutant copies one output's value commitment
//! onto another, so the duplicated value breaks the per-asset sum at a
//! locator no other row occupies.
//!
//! # The row this ceremony drives
//!
//! `copied-commitment`. Its fault is a value commitment that appears
//! twice: the output side then commits one amount two times while the
//! input side carries it once, so the closed asset's in-equals-out sum
//! breaks by exactly the copied value and the target refuses the
//! candidate in its amount check before any script runs.
//!
//! # Why a ceremony of its own, and why a THREE-output successor
//!
//! The verdict this row draws is the words `private-ct-imbalance`
//! already drew, so the verdict cannot be what separates the two rows —
//! the locator has to be. `private-ct-imbalance` mutates the value
//! commitment at output one of a one-in two-out successor. A successor
//! with a THIRD confidential output gives this row an output-index-two
//! value-commitment field on a `(1, 3)` shape: both members of the
//! `(range, shape)` pair differ, which is the same three-part separator
//! the §15.5 consensus rows rest on.
//!
//! That third output is why this is a new ceremony rather than a mutant
//! added to `conservation-negatives`, which builds a one-in two-out
//! successor. Widening that ceremony's successor would move the control
//! digest it has already recorded, and a recorded control that moves is
//! the one thing the corpus's stability property forbids.
