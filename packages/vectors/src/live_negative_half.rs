//! The negative half's work register: every §15 row a run has still not
//! answered, each carrying the NAMED GAP that keeps it waiting.
//!
//! # Why a register and not a number
//!
//! [`crate::live_evidence`] classifies every row of the §15 matrix and
//! counts how many stand at [`LiveRowStanding::NativeRunRequired`]. A
//! count is enough to know the half is unfinished and not enough to work
//! on: it says HOW MANY rows are waiting and never says WHICH, so a wave
//! reading it has to rediscover the list by re-deriving the classifier's
//! fall-through every time, and two waves can disagree about the list
//! while agreeing about the number. The count is left to the census
//! deliberately — a number repeated in prose here would be a second
//! place for it to drift.
//!
//! This register is that list, held against the classifier by
//! [`every_outstanding_row_is_registered`]. The test is set equality in
//! BOTH directions, which is what makes the register load-bearing rather
//! than documentation: a row that moves to an observed standing and is
//! left here fails the test, and a row that stays required without being
//! listed here fails it too. Neither can happen silently.
//!
//! # Why every entry carries a gap
//!
//! [`LiveRowStanding::NativeRunRequired`] means "a run would answer
//! this". For most of the rows below that is FALSE, and the falseness is
//! this register's main finding: no candidate staging the row's own
//! mutation can be built at all, or none whose refusal could name the
//! row's class. A standing that says a run is owed, for a row no run can
//! reach, misdirects every wave that reads it towards chartering the
//! run.
//!
//! So each entry names its own [`NegativeHalfGap`]. The vocabulary has
//! no member meaning "not yet looked at": an entry without a gap does
//! not compile, which is the same discipline the census-forms register
//! was built under one level down. What the gaps buy is triage a reader
//! can act on — [`NegativeHalfGap::MutantBuilderOwed`] is work somebody
//! can simply do, and [`NegativeHalfGap::RefusalIsProgramGeneric`] is a
//! wall no amount of work moves.
//!
//! # What this register is not
//!
//! It is not evidence, and it moves no row. A row leaves this list by
//! being ANSWERED at its own site — an observed refusal recorded in
//! [`crate::live_evidence`] against an accepted control — and never by
//! being deleted from here. The register follows the classifier; the
//! classifier never follows the register.
//!
//! Nor is a gap a verdict on the row. Several entries below record that
//! a row's own TYPING is in question, and naming that is not the same as
//! settling it: settling belongs elsewhere, and this register's job is
//! to make sure the question is asked rather than lost.

use crate::error::VectorError;
use crate::live_evidence::{LiveRowStanding, derive_live_evidence_plan};

/// Why one §15 row is still waiting.
///
/// The members are ordered from "work somebody can do" to "wall nothing
/// moves", because that is the order a wave planning the next bite reads
/// them in.
///
/// There is deliberately no member meaning `unassessed`. A row whose gap
/// nobody has established does not get an entry, and a row without an
/// entry fails [`every_outstanding_row_is_registered`], so the absence
/// of that member is what makes the register's coverage total rather
/// than aspirational.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NegativeHalfGap {
    /// The mutant is constructible today and nobody has written it.
    ///
    /// The one member that is WORK rather than an obstacle, and it is
    /// kept separate for exactly that reason: a reader planning a wave
    /// should be able to see the cheap wins without reading every note.
    /// A row here needs no new capability, no ruling and no target
    /// change — only a case added to a ceremony that already runs.
    MutantBuilderOwed,
    /// The safe successor this ceremony emits does not carry the structure
    /// the row mutates, so reaching the fault needs a structurally
    /// different transaction.
    ///
    /// Work rather than a wall, but work larger than a case added to a
    /// ceremony that already runs: the row's fault lives in a shape the
    /// demonstration successor has none of — a facet the safe constructor
    /// never emits (an issuance on an input, a destruction, a root effect,
    /// a specialized-event projection, a transition certificate to omit),
    /// the sponsored form this sponsorless ceremony does not build, or an
    /// added offsetting input-and-output pair the fixed two-in two-out
    /// shape does not carry. The owner-signing route re-signs whatever
    /// bytes it is handed, so it would carry such a candidate past the
    /// signature gate — but the candidate has to be BUILT first, and
    /// building it is a different ceremony rather than a mutation of this
    /// one.
    FacetNeedsADifferentTransaction,
    /// Re-signing over the mutated bytes does not carry the row to its
    /// script class. HISTORICAL, and now VACATED.
    ///
    /// The name once said a component was absent — a way to produce a valid
    /// owner signature over rebuilt bytes. That component is built and
    /// used: `OwnerSigningCensus::over_foreign_bytes_for_negative_evidence`
    /// signs a mutant over its own census, and
    /// `vault-control-entitlement-or-bare-u-output` left through it on the
    /// coordinator leaf's `InspectOutputScriptPubKey` version clause. Every
    /// OTHER row that carried this member has since had its native
    /// determination made and been retyped to its true gap: the route
    /// rescues no further row to a NOVEL script class, because the only
    /// covenant clause a conservation-preserving surgery reaches is the
    /// version clause that row already took — the destination closure
    /// checks the output's asset and program VERSION and then DROPS the
    /// payload, so a conservation-preserving payload substitute is accepted
    /// rather than refused. The rows moved to
    /// [`Self::ConsensusAnswersBeforeScript`],
    /// [`Self::FacetNeedsADifferentTransaction`],
    /// [`Self::NoIndependentCovenantClause`],
    /// [`Self::NoAdmittedShapeCarriesTheFault`] and
    /// [`Self::NoAdmittedRepresentationCarriesTheField`]. No row carries
    /// this member now, and it is kept only as provenance.
    OwnerSigningOverForeignBytesAbsent,
    /// Consensus answers the mutant before the covenant's own clause runs.
    ///
    /// The row's mutation is stageable on the explicit successor as a byte
    /// surgery, and the re-signed mutant passes the signature gate — but
    /// the mutation breaks the explicit per-asset sum (an asset changed, a
    /// value moved off its total, an output added or removed, an input
    /// removed), and the target checks that balance at CONSENSUS before it
    /// runs any script. So the refusal a run produces is `bad-txns-in-ne-out`
    /// or a proof verdict, not the row's declared SCRIPT class, and the
    /// covenant's own clause for the same field — the destination closure's
    /// asset check, the explicit conservation fragment — never gets to run.
    /// This is the wall for the SCRIPT class specifically: a drive to the
    /// CONSENSUS verdict is possible where a distinct output earns a
    /// separating field range, which is the `private-ct-imbalance`
    /// precedent.
    ConsensusAnswersBeforeScript,
    /// The explicit lane's witness mutation reaches item zero only.
    ///
    /// The staged mutation replaces the signature payload and nothing
    /// else; the leaf script and the control block are written after it,
    /// and the attributability check is a width bound around a
    /// signature. Reaching the rest of the stack needs that stage
    /// generalized to a declared byte range, which is a prerequisite
    /// rather than a refinement.
    WitnessSurgeryStageAbsent,
    /// No admitted shape can carry the fault.
    ///
    /// Not a missing builder but a missing SUBJECT: the candidate the
    /// row describes is outside what this workspace's own bounds admit,
    /// so there is nothing to mutate.
    NoAdmittedShapeCarriesTheFault,
    /// The field the row mutates is empty in every candidate built.
    ///
    /// Distinct from the member above because the shape is admitted and
    /// the FIELD is the thing that is not there. "Malformed" presupposes
    /// a well-formed original, and there is none to malform.
    NoAdmittedRepresentationCarriesTheField,
    /// The mutated field is derived from the shape, with no covenant clause
    /// constraining it independent of the owner signature.
    ///
    /// Distinct from the two structural-absence members above: the field is
    /// present and the shape is admitted, but the covenant carries no
    /// clause that reads THIS field and refuses it on its own — a
    /// position's role (a fee-or-sponsor u, an unclassified u) is fixed by
    /// where it sits in the shape, not by a value a clause inspects. A
    /// re-signed mutant authorizes its own outputs, so with no independent
    /// clause to refuse the field the row's own observation is not
    /// producible on this deployment at all.
    NoIndependentCovenantClause,
    /// The row's own typing is in question and a ruling is owed.
    ///
    /// The row may be mis-typed in polarity, in boundary, or in the
    /// relation it links to. Driving it before the ruling would produce
    /// evidence filed under a claim the row may not be making — the
    /// failure the time-locked row's retyping was the repair for.
    RowTypingInQuestion,
    /// Several published classes fit the row equally well.
    ///
    /// The matrix itself says so. A refusal cannot name the row's class
    /// while the row has no single class to name, so narrowing it is
    /// owed before any run.
    RowClassUnderdetermined,
    /// The observed-layer vocabulary has no member for what happened.
    ///
    /// The refusal exists and was recorded; what is missing is a name to
    /// file it under, so filing it would read the vocabulary's poverty
    /// as evidence.
    ObservedLayerVocabularyAbsent,
    /// One identical verdict covers rows that must be told apart.
    ///
    /// Each row's own mutant can be driven and each draws the SAME
    /// words, so nothing in the record separates them. This is not the
    /// same as two rows sharing a verdict where each drove its own
    /// mutant and a declared FIELD separates them — that is answerable,
    /// and two §15.5 rows were answered that way. It is the case where
    /// no such separating fact exists.
    TargetVerdictDoesNotSeparateTheRows,
    /// The refusal is program-generic and can never name the class.
    ///
    /// The wall nothing moves. A leaf runs only from a taptree its spent
    /// program commits to, so any candidate whose program or revealed
    /// leaf is not the committed one is refused by the COMMITMENT rule
    /// before a single opcode executes — the identical verdict every
    /// foreign taptree draws. An observation of it establishes the
    /// commitment rule and says nothing about the row. This is the
    /// ground the time-locked row was retyped first-party on.
    RefusalIsProgramGeneric,
    /// The pair the row compares has never been submitted.
    ///
    /// The one positive row here. Two obstacles stand: the accepted
    /// transactions on hand are independent ceremonies rather than one
    /// fixture materialized twice, and no standing can hold a relation
    /// over two identities.
    PairSubmissionCapabilityAbsent,
}

/// One row, its gap, and the ground for the gap.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NegativeHalfEntry {
    row: &'static str,
    gap: NegativeHalfGap,
    ground: &'static str,
}

impl NegativeHalfEntry {
    /// The §15 row this entry is about.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// Why it is still waiting.
    #[must_use]
    pub const fn gap(&self) -> NegativeHalfGap {
        self.gap
    }

    /// The specific fact the gap rests on for this row.
    ///
    /// One sentence, and about THIS row rather than about the gap: the
    /// gap's own meaning is on its member, and repeating it here would
    /// make the register say the same thing twice.
    #[must_use]
    pub const fn ground(&self) -> &'static str {
        self.ground
    }
}

/// One entry, spelled once.
const fn entry(row: &'static str, gap: NegativeHalfGap, ground: &'static str) -> NegativeHalfEntry {
    NegativeHalfEntry { row, gap, ground }
}

use NegativeHalfGap as G;

/// Every §15 row still standing at
/// [`LiveRowStanding::NativeRunRequired`], in matrix order.
///
/// Derived from the classifier's own fall-through rather than kept by
/// hand: a row reaches [`LiveRowStanding::NativeRunRequired`] only after
/// every earlier branch has declined it — it is not vocabulary-closed,
/// not report-layer, not first-party, not blocked on a named component,
/// and no acceptance, refusal or determinism observation of its own
/// shape exists. What is written here is the result of that subtraction,
/// and the test below is what keeps the writing and the subtraction the
/// same list.
pub const STILL_REQUIRED: &[NegativeHalfEntry] = &[
    // §15.2 — the one positive row no run has answered. It is here for
    // the same reason the negatives are: a row waiting on a run is
    // waiting on a run whatever its polarity, and a register that held
    // only negatives would report the positive half complete.
    entry(
        "projection-equality-with-paired-explicit",
        G::PairSubmissionCapabilityAbsent,
        "the pair registry carries no accepted member and the pairs lane submits nothing",
    ),
    // §15.4 — the class, asset and constructor faults still waiting.
    //
    // The row that shared this section's head with a fifth
    // recognition-class sibling, `wrong-owner-metadata`, has LEFT: its
    // honest third-owner program is now buildable, a third owner threaded
    // through the bundle link, and it is discharged first-party beside
    // its three siblings on the input recognition's own refusal.
    entry(
        "wrong-explicit-asset",
        G::ConsensusAnswersBeforeScript,
        "rewriting a destination's asset unbalances the explicit per-asset sum, refused bad-txns-in-ne-out before the destination closure's asset clause runs",
    ),
    entry(
        "confidential-asset-commitment",
        G::ConsensusAnswersBeforeScript,
        "replacing an explicit asset with a commitment fails surjection at consensus before the covenant's explicit-asset clause is reached",
    ),
    entry(
        "unclassified-u",
        G::NoIndependentCovenantClause,
        "a position's role is fixed by where it sits in the shape, and no covenant clause reads a classification field to refuse an unclassified one",
    ),
    entry(
        "sponsor-or-fee-role-carrying-u",
        G::NoIndependentCovenantClause,
        "the fee position is fixed by the shape and no covenant clause constrains a u's fee-or-sponsor role independent of the signature",
    ),
    entry(
        "key-path-escape",
        G::ObservedLayerVocabularyAbsent,
        "the probe ran and its refusal is filed under a script-path name for want of a key-path one",
    ),
    entry(
        "malformed-control-path",
        G::WitnessSurgeryStageAbsent,
        "the control block is witness item two and the staged mutation reaches item zero only",
    ),
    // §15.5 — the value and partition faults still waiting. Two rows of
    // this section have LEFT this register: `malformed-rangeproof` and
    // `wrong-private-blinding-balance` are answered by the conservation
    // ceremony's own run, each on its own driven mutant, and they are
    // recorded there rather than here.
    entry(
        "output-total-one-below-input",
        G::ConsensusAnswersBeforeScript,
        "a destination value one below its total leaves inputs exceeding outputs, refused bad-txns-in-ne-out before the explicit conservation fragment",
    ),
    entry(
        "output-total-one-above-input",
        G::ConsensusAnswersBeforeScript,
        "a destination value one above its total leaves outputs exceeding inputs, refused bad-txns-in-ne-out before that same fragment",
    ),
    entry(
        "amount-outside-semantic-domain",
        G::NoAdmittedShapeCarriesTheFault,
        "conservation forces the total to equal the consumed one and no chain mints such a coin",
    ),
    entry(
        "malformed-surjection-proof",
        G::NoAdmittedRepresentationCarriesTheField,
        "the surjection field is empty in every form built, an explicit asset requiring it so",
    ),
    entry(
        "copied-commitment",
        G::TargetVerdictDoesNotSeparateTheRows,
        "its mutant draws bad-txns-in-ne-out at the change output's value commitment, the same words and field range private-ct-imbalance already drove, and the successor has no third confidential output whose distinct range would separate them",
    ),
    entry(
        "private-output-omitted",
        G::ConsensusAnswersBeforeScript,
        "deleting a created destination drops the output sum, refused bad-txns-in-ne-out before the covenant's cardinality clause runs",
    ),
    entry(
        "hidden-private-u-output",
        G::ConsensusAnswersBeforeScript,
        "adding an undeclared protocol-asset output raises the output sum, refused bad-txns-in-ne-out before the cardinality clause",
    ),
    entry(
        "omitted-source",
        G::ConsensusAnswersBeforeScript,
        "deleting a receipt input drops the input sum, refused bad-txns-in-ne-out before the covenant reads a single field",
    ),
    entry(
        "output-claimed-through-two-flows",
        G::NoAdmittedRepresentationCarriesTheField,
        "a per-flow assignment is a request concept with no wire field, so no built candidate carries an output claimed through two flows",
    ),
    entry(
        "issuance",
        G::FacetNeedsADifferentTransaction,
        "the covenant carries an issuance-absence clause, but the successor emits no issuance and staging one is a different transaction",
    ),
    entry(
        "destruction",
        G::FacetNeedsADifferentTransaction,
        "a destruction facet the safe constructor never emits, so there is no successor output to mutate into one",
    ),
    entry(
        "value-routed-into-ash-or-time-locked-receipt",
        G::NoAdmittedShapeCarriesTheFault,
        "no ash or time-locked destination is admitted, and the destination closure drops a payload, so a same-version substitute is accepted rather than refused",
    ),
    entry(
        "second-offsetting-u-flow",
        G::FacetNeedsADifferentTransaction,
        "an offsetting flow is an added balanced input-and-output pair the fixed two-in two-out successor does not carry",
    ),
    // §15.6 — the sponsor faults still waiting. The sponsor table's
    // other five are answered: two are report-layer, two are pre-target
    // first-party, and `missing-sponsor-authorization` is one of the
    // observed refusals this register's successors are modelled on.
    entry(
        "sponsor-protocol-overlap",
        G::FacetNeedsADifferentTransaction,
        "the sponsor region belongs to a sponsored successor this sponsorless ceremony does not build",
    ),
    entry(
        "two-sponsor-envelopes",
        G::NoAdmittedShapeCarriesTheFault,
        "the shape bounds admit at most one sponsor input, refused before any program is emitted",
    ),
    entry(
        "foreign-sponsor-asset",
        G::NoAdmittedShapeCarriesTheFault,
        "sponsor recognition refuses a foreign-asset sponsor coin first-party, so no such candidate is admitted to mutate",
    ),
    entry(
        "sponsor-change-in-protocol-range",
        G::FacetNeedsADifferentTransaction,
        "the sponsor-change position belongs to a sponsored successor this ceremony does not build",
    ),
    entry(
        "sponsor-member-unclassified",
        G::NoAdmittedShapeCarriesTheFault,
        "every emitted position is classified by construction, so no unclassified member exists",
    ),
    entry(
        "confidential-sponsor-values",
        G::RowClassUnderdetermined,
        "committed sponsor values are an accepted positive shape and the negative is unstated",
    ),
    // §15.7 — the root, event, ABI and linker faults still waiting. One
    // row of this section is deliberately NOT here and never was:
    // `raw-transaction-bypassing-safe-construction` stood at
    // `InfrastructureBlocked`, which is a different state from waiting
    // on a run, and listing it here would have reported a blocked row as
    // a runnable one. It is now ANSWERED at
    // `LiveRowStanding::FirstPartyFactObserved` — its own site says what
    // it asks is whether a raw path exists at all, and one does — so it
    // is absent from this register for the second of two reasons in a
    // row, neither of them that a run is owed.
    entry(
        "any-root-input-or-output",
        G::FacetNeedsADifferentTransaction,
        "a root effect the safe constructor never emits, needing a transaction shape this ceremony does not build",
    ),
    entry(
        "burn-record-or-specialized-event",
        G::FacetNeedsADifferentTransaction,
        "a specialized-event projection the safe constructor never emits, needing a different transaction",
    ),
    entry(
        "omitted-transition-certificate",
        G::FacetNeedsADifferentTransaction,
        "the successor emits no transition certificate, so there is none to omit without building a certificate-bearing transaction first",
    ),
    // The four leaf-arrangement rows. The covenant DOES introspect its
    // own input index, so these are not commitment-generic — and that is
    // what makes the collision the finding rather than a guess: the
    // coordinator fragment and the member fragment abort at different
    // opcodes, giving two verdicts for four rows.
    entry(
        "wrong-coordinator",
        G::TargetVerdictDoesNotSeparateTheRows,
        "aborts at the coordinator index check, in the same words two-coordinators draws",
    ),
    entry(
        "two-coordinators",
        G::TargetVerdictDoesNotSeparateTheRows,
        "aborts at the coordinator index check, in the same words wrong-coordinator draws",
    ),
    entry(
        "no-coordinator",
        G::TargetVerdictDoesNotSeparateTheRows,
        "aborts at the member bound check, in the same words the leaf exchange draws",
    ),
    entry(
        "member-coordinator-leaf-exchange",
        G::TargetVerdictDoesNotSeparateTheRows,
        "aborts at the member bound check, in the same words no-coordinator draws",
    ),
    entry(
        "receipt-sponsor-range-exchange",
        G::FacetNeedsADifferentTransaction,
        "the receipt and sponsor ranges belong to a sponsored successor this ceremony does not build",
    ),
    // A TYPED STOP, and the one row of the seven the retyping ruling
    // does not reach. Its target-side wall stands as written. What the
    // retyping would need beside it is a first-party validator that
    // refuses a reordered stack, and there is none: `check_offered`
    // compares inputs, outputs, version and locktime and states in its
    // own doc that the witness is excluded because the selected profile
    // excludes it from the message, and `live_signing` writes the stack
    // unconditionally from `LiveWitnessItem::ORDER` with no field a
    // caller could use to reorder it. So the order is not a degree of
    // freedom, which is a structural protection rather than a refusal —
    // nothing REFUSES a reorder because nothing can express one.
    //
    // The nearest available observation is refused deliberately. A
    // signing census can be handed a declared leaf hash taken over the
    // item a reorder would move into the leaf position, which draws
    // `LeafHashDoesNotCommit` — but that is a MODEL of the row rather
    // than the row's own mutation, and it is the same class
    // `control-block-from-another-program` draws on its own mutant. Two
    // rows sharing one class where only one drove its own change is the
    // reading-one-observation-onto-two-rows the register exists to
    // prevent. Escalated rather than answered.
    entry(
        "witness-reorder",
        G::RefusalIsProgramGeneric,
        "no validator refuses a reorder because the stack order is written from a fixed constant",
    ),
    entry(
        "target-bytes-changed-after-abi-validation",
        G::RowTypingInQuestion,
        "the live lane has no ABI-validation entry point, so the row's after has no referent yet",
    ),
];

/// The rows the classifier itself leaves waiting on a run, recomputed.
///
/// The register's counterpart, and the reason the register can be
/// trusted: this reads the standing of every row from
/// [`derive_live_evidence_plan`] rather than from any list, so the two
/// can be compared instead of one being asserted.
///
/// # Errors
///
/// Whatever [`derive_live_evidence_plan`] returns when a source artifact
/// does not build.
pub fn outstanding_rows() -> Result<Vec<&'static str>, VectorError> {
    let plan = derive_live_evidence_plan()?;
    Ok(plan
        .rows()
        .iter()
        .filter(|row| matches!(row.standing(), LiveRowStanding::NativeRunRequired(_)))
        .map(|row| row.row().name())
        .collect())
}

/// How many rows carry each gap, recomputed from the register.
#[must_use]
pub fn gap_census() -> std::collections::BTreeMap<NegativeHalfGap, usize> {
    let mut census = std::collections::BTreeMap::new();
    for entry in STILL_REQUIRED {
        *census.entry(entry.gap()).or_insert(0) += 1;
    }
    census
}

#[cfg(test)]
mod tests {
    use super::{NegativeHalfEntry, NegativeHalfGap, STILL_REQUIRED, gap_census, outstanding_rows};
    use std::collections::BTreeSet;

    /// The register and the classifier name the same rows.
    ///
    /// Set equality in both directions, reported as the two differences
    /// rather than as one boolean, so a failure says which rows drifted
    /// and in which direction instead of only that something did.
    #[test]
    fn every_outstanding_row_is_registered() {
        let computed: BTreeSet<&str> = outstanding_rows()
            .expect("the evidence plan derives")
            .into_iter()
            .collect();
        let registered: BTreeSet<&str> =
            STILL_REQUIRED.iter().map(NegativeHalfEntry::row).collect();

        let unregistered: Vec<&&str> = computed.difference(&registered).collect();
        assert!(
            unregistered.is_empty(),
            "rows still waiting on a run and absent from the register: {unregistered:?}",
        );

        let stale: Vec<&&str> = registered.difference(&computed).collect();
        assert!(
            stale.is_empty(),
            "rows registered as waiting that the classifier has answered: {stale:?}",
        );
    }

    /// The register lists each row once.
    ///
    /// A duplicated entry would still satisfy set equality above while
    /// making the register's own length disagree with what it holds.
    #[test]
    fn the_register_holds_no_duplicate() {
        let unique: BTreeSet<&str> = STILL_REQUIRED.iter().map(NegativeHalfEntry::row).collect();
        assert_eq!(
            unique.len(),
            STILL_REQUIRED.len(),
            "the register lists a row more than once",
        );
    }

    /// Every entry states a ground, and states it about its own row.
    ///
    /// A gap without a ground would be the `unassessed` member the
    /// vocabulary deliberately lacks, reintroduced as an empty string.
    #[test]
    fn every_entry_grounds_its_gap() {
        for entry in STILL_REQUIRED {
            assert!(
                !entry.ground().is_empty(),
                "{} names a gap without grounding it",
                entry.row(),
            );
        }
    }

    /// The gap census covers the register and nothing else.
    ///
    /// Recomputed rather than asserted per member: what matters is that
    /// no row escapes a gap, which is the sum agreeing with the length.
    #[test]
    fn the_gap_census_accounts_for_every_row() {
        let census = gap_census();
        assert_eq!(
            census.values().sum::<usize>(),
            STILL_REQUIRED.len(),
            "the gap census and the register disagree about how many rows there are",
        );
    }

    /// The rows a wave could simply do are findable without reading
    /// every note.
    ///
    /// The register's practical purpose, asserted so it keeps working: a
    /// reader planning the next bite asks for this member and gets the
    /// cheap wins. It is allowed to become empty — that would mean the
    /// easy half is finished, not that the test is stale.
    #[test]
    fn the_workable_rows_are_separable_from_the_walls() {
        let workable = STILL_REQUIRED
            .iter()
            .filter(|entry| entry.gap() == NegativeHalfGap::MutantBuilderOwed)
            .count();
        assert!(
            workable < STILL_REQUIRED.len(),
            "every row cannot be merely unwritten work",
        );
    }
}
