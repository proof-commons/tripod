//! The consensus shape-possibility register for the confidential lane.
//!
//! What transaction shapes can a blinded transfer take, which of those
//! has this workspace actually run, and where a shape consensus admits
//! is refused here, what refuses it and what would remove the refusal.
//!
//! # Why the register exists
//!
//! Two very different facts had been collapsing into one word. A shape
//! this workspace does not build because the target would reject it,
//! and a shape it does not build because its own fixture registry
//! declines to express it, were both being described as
//! "unconstructible" — and the second kind is a convention this
//! repository chose and could unchoose, while the first is arithmetic
//! nobody can vote on. Recording them under one name makes a local
//! convention read as a law of the protocol, which is the error this
//! module exists to make impossible.
//!
//! So every entry carries TWO verdicts that are computed and cited
//! separately. [`ConsensusVerdict`] says what the target's balance rule
//! admits, in a three-member evidence vocabulary that never lets a
//! derivation pass as an observation. [`FirstPartyStatus`] says what
//! this workspace's own registry does about it, and where the two
//! disagree — consensus admits, the registry refuses — the entry
//! carries a [`Limitation`] naming the refusal, the convention behind
//! it, and the [`RemovalPath`] that would end it.
//!
//! # What this module does not do
//!
//! It observes nothing. Every OBSERVED-ACCEPTED verdict cites a
//! `run_of_record` identity some earlier wave produced; no verdict here
//! is produced by running anything, and a shape consensus admits but
//! nobody has submitted is recorded SOURCE-DERIVED and never "run".
//! Nothing here moves a matrix row, a blocker or a residual: this is a
//! register, and a register is not evidence.
//!
//! # The balance rule the consensus verdicts are derived from
//!
//! Elements checks value conservation as a Pedersen tally over
//! commitments, at the pinned tip `b7fc5d080a`:
//! `src/confidential_validation.cpp:73-81` runs
//! `secp256k1_pedersen_verify_tally` over every input commitment
//! against every output commitment, queued at `:363-366`. An explicit
//! value does not sit outside that sum — it is committed at `:345-349`
//! with an ALL-ZERO blinder and joins the same tally, which is the
//! whole reason the derivations below work.
//!
//! Write a commitment as `v*H + r*G` for value `v` under blinder `r`.
//! The tally holds exactly when both coordinates balance: the values
//! sum equal, AND the blinders sum equal. The second half is the one
//! that decides shape possibility, because an explicit output
//! contributes `r = 0` and can never absorb a blinder.
//!
//! So the whole consensus question reduces to one predicate, computed
//! here by [`BlindedShape::blinder_sum_is_absorbable`]: does the output
//! set contain at least one BLINDED output? If it does, that output's
//! blinder can be set to whatever closes the sum and the shape is
//! possible. If every output is explicit, the output blinder sum is
//! fixed at zero, and the shape is possible only if the input blinder
//! sum is zero too — which for a genuinely blinded input it is not.
//!
//! # Where the fee output enters
//!
//! A fee output is mandatorily EXPLICIT. `CTxOut::IsFee`
//! (`src/primitives/transaction.h:324-327`) holds only for an output
//! with an empty `scriptPubKey` and an explicit value AND asset. A fee
//! output therefore always contributes a zero blinder, and can never be
//! the output that absorbs the input blinder sum. That single fact is
//! what separates [`BlindedShape::OneToOneWithFee`], which stays
//! possible because its OTHER output is blinded, from
//! [`BlindedShape::FeeOnly`], which is the one impossible shape in the
//! enumeration.
//!
//! A fee output is not mandatory in the other direction: a transaction
//! with no fee output at all is consensus-acceptable, which is not a
//! derivation here but an observation — every accepted identity this
//! module cites was built sponsorless and carries no fee output.

use target_elements_conformance::confidential_fixture::RegistrationRefusal;

/// One transaction shape of the blinded confidential lane.
///
/// A shape is the pair (blinded inputs, outputs) together with whether
/// one of those outputs is the mandatorily explicit fee output. Nothing
/// else about a transaction changes either verdict, which is why the
/// axis is this narrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlindedShape {
    /// One blinded input to one blinded output, no fee output.
    OneToOne,
    /// One blinded input to one blinded output beside a fee output.
    OneToOneWithFee,
    /// One blinded input to two blinded outputs.
    OneToTwo,
    /// One blinded input to three blinded outputs.
    OneToThree,
    /// Two blinded inputs merged into one blinded output.
    TwoToOne,
    /// Two blinded inputs to two blinded outputs.
    TwoToTwo,
    /// Two blinded inputs to three blinded outputs.
    TwoToThree,
    /// One blinded input to a fee output and nothing else.
    FeeOnly,
}

impl BlindedShape {
    /// Every shape the register enumerates.
    ///
    /// # The closure rule
    ///
    /// The enumeration is the small-shape window: at least one blinded
    /// input, at most two of them, and one to three outputs, plus the
    /// two fee-bearing members that window does not otherwise reach.
    ///
    /// It is closed, and a reader can see nothing is missing, because
    /// of what [`Self::blinder_sum_is_absorbable`] shows: the consensus
    /// verdict depends on NOTHING but whether at least one output is
    /// blinded. It does not depend on the input count, and it does not
    /// depend on the output count beyond the difference between "some
    /// blinded output" and "none". Both of those cases already appear
    /// in the window — every member but [`Self::FeeOnly`] is the first,
    /// and `FeeOnly` is the second — so every shape OUTSIDE the window
    /// inherits the verdict of whichever case it falls into, and adding
    /// it would restate a row rather than add one.
    ///
    /// The window is therefore chosen for the first-party half, where
    /// the counts DO matter: the registry's own clauses are cardinality
    /// clauses, and every shape this lane has built or been refused
    /// falls inside it.
    pub const ALL: [Self; 8] = [
        Self::OneToOne,
        Self::OneToOneWithFee,
        Self::OneToTwo,
        Self::OneToThree,
        Self::TwoToOne,
        Self::TwoToTwo,
        Self::TwoToThree,
        Self::FeeOnly,
    ];

    /// How many blinded inputs the shape consumes.
    #[must_use]
    pub const fn blinded_inputs(self) -> usize {
        match self {
            Self::OneToOne
            | Self::OneToOneWithFee
            | Self::OneToTwo
            | Self::OneToThree
            | Self::FeeOnly => 1,
            Self::TwoToOne | Self::TwoToTwo | Self::TwoToThree => 2,
        }
    }

    /// How many outputs the shape creates, the fee output included.
    #[must_use]
    pub const fn outputs(self) -> usize {
        match self {
            Self::OneToOne | Self::TwoToOne | Self::FeeOnly => 1,
            Self::OneToOneWithFee | Self::OneToTwo | Self::TwoToTwo => 2,
            Self::OneToThree | Self::TwoToThree => 3,
        }
    }

    /// How many of those outputs are the explicit fee output.
    ///
    /// Zero or one; a transaction has no reason to carry two, and none
    /// of the enumerated shapes does.
    #[must_use]
    pub const fn fee_outputs(self) -> usize {
        match self {
            Self::OneToOneWithFee | Self::FeeOnly => 1,
            Self::OneToOne
            | Self::OneToTwo
            | Self::OneToThree
            | Self::TwoToOne
            | Self::TwoToTwo
            | Self::TwoToThree => 0,
        }
    }

    /// How many outputs carry a blinded value.
    ///
    /// Every non-fee output on this lane is blinded, so this is the
    /// output count less the fee output.
    #[must_use]
    pub const fn blinded_outputs(self) -> usize {
        self.outputs() - self.fee_outputs()
    }

    /// Whether the output set can absorb the input blinder sum.
    ///
    /// This is the whole consensus question, computed rather than
    /// looked up. The tally balances the blinder coordinate as well as
    /// the value coordinate; an explicit output contributes a zero
    /// blinder; so a nonzero input blinder sum needs at least one
    /// blinded output to land on, and every shape that has one is
    /// possible.
    ///
    /// The answer is derived from the shape alone and is compared
    /// against the entry's recorded [`ConsensusVerdict`] by
    /// `the_recorded_verdicts_agree_with_the_tally_predicate`, so the
    /// register cannot record a verdict its own arithmetic denies.
    #[must_use]
    pub const fn blinder_sum_is_absorbable(self) -> bool {
        self.blinded_outputs() > 0
    }

    /// The handle the register's prose section uses for the shape.
    #[must_use]
    pub const fn handle(self) -> &'static str {
        match self {
            Self::OneToOne => "one-to-one",
            Self::OneToOneWithFee => "one-to-one-with-fee",
            Self::OneToTwo => "one-to-two",
            Self::OneToThree => "one-to-three",
            Self::TwoToOne => "two-to-one-merge",
            Self::TwoToTwo => "two-to-two",
            Self::TwoToThree => "two-to-three",
            Self::FeeOnly => "fee-only",
        }
    }
}

/// What consensus says about a shape, and on what evidence.
///
/// The three members are three EVIDENCE CLASSES, not three degrees of
/// confidence. The distinction they keep is the one this register was
/// built for: a shape somebody submitted and a node accepted is a
/// different kind of fact from a shape the balance rule permits and
/// nobody has ever built, and the second must never be reported as the
/// first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConsensusVerdict {
    /// A node accepted a transaction of this shape into a block.
    ///
    /// The strongest class, and the only one carrying a target-computed
    /// identity. The identity is not a literal here: it is the
    /// `run_of_record` constant an earlier wave recorded, so a wave that
    /// re-ran and got different bytes would move this register too.
    ObservedAccepted {
        /// The identity the target computed for the accepted shape.
        identity: &'static str,
    },
    /// The balance rule admits the shape; nobody has submitted one.
    ///
    /// Derived from the tally arithmetic and the pinned source, and
    /// EXPLICITLY NOT an observation. A shape in this class has never
    /// been offered to a node by this workspace, and the register says
    /// so rather than letting a derivation age into a claim of having
    /// run.
    SourceDerivedPossible,
    /// The balance rule cannot admit the shape.
    ///
    /// Also derived rather than observed — no node has refused one of
    /// these either, because none was ever built. The ground is the
    /// tally, not a node's verdict.
    SourceDerivedImpossible,
}

/// What this workspace's own fixture registry does with a shape.
///
/// The second, independent verdict. Its three members are the three
/// ways a first-party position can stand against the consensus one, and
/// naming them apart is what stops a local convention being read as a
/// protocol rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FirstPartyStatus {
    /// The registry builds it and a run of record observed it accepted.
    ConstructibleAndObserved,
    /// Consensus admits it and the registry refuses it anyway.
    ///
    /// The interesting case, and the one carrying a removal path. The
    /// refusal is recomputed by driving the registry, never asserted.
    RefusedByConvention {
        /// The typed refusal the registry actually returns.
        refusal: RegistrationRefusal,
        /// What refuses it, why, and what would remove it.
        limitation: Limitation,
    },
    /// Consensus cannot admit it and the registry refuses it too.
    ///
    /// The refusal guards consensus — though see
    /// [`Limitation::guards_only_incidentally`], because guarding by
    /// accident and guarding by design are not the same thing.
    RefusalGuardsConsensus {
        /// The typed refusal the registry actually returns.
        refusal: RegistrationRefusal,
    },
    /// The registry refused it by convention, the convention was
    /// STRUCTURALLY REMOVED, and a run of record then observed the shape
    /// accepted.
    ///
    /// # Why this is not just [`Self::ConstructibleAndObserved`]
    ///
    /// It could have been. The shape is constructible and it was
    /// observed, and collapsing it into that member would lose nothing a
    /// verdict depends on.
    ///
    /// What it would lose is the HISTORY, and the history is the point.
    /// The ruling this register implements asks that a first-party
    /// limitation be labeled, pinned, explained, and eventually removed —
    /// four stages, of which a register recording only the last would be
    /// evidence of none. A row that says "constructible" says nothing
    /// about a wall having stood there, and a wall nobody remembers is
    /// one that gets rebuilt.
    ///
    /// So the removed limitation stays cited from the row it used to
    /// refuse, and the removal carries what changed and what proved it.
    ConstructibleAfterRemoval {
        /// The convention that used to refuse the shape.
        removed: Limitation,
        /// What ended it, and what proved that it had.
        removal: LimitationRemoval,
    },
    /// The registry expresses the shape, and something downstream of the
    /// registry stops it short of a node.
    ///
    /// # Why this is not a refusal, and not an observation either
    ///
    /// The registry does not refuse it: a manifest of this shape
    /// registers, derives and digests. So recording it under
    /// [`Self::RefusedByConvention`] would name a refusal that no longer
    /// happens.
    ///
    /// And nothing has run it, so recording it constructible-and-observed
    /// would be the one error this register exists to prevent — a
    /// vocabulary that CAN express a shape is not a chain that HAS
    /// accepted one.
    ///
    /// It is a third thing, and the register would rather carry a third
    /// member than round it to whichever of the other two is nearer.
    ExpressibleAndUnrun {
        /// The convention that used to make the shape inexpressible.
        removed: Limitation,
        /// What ended that convention.
        removal: LimitationRemoval,
        /// Where a run stops now, in the layer's own terms.
        stops_at: &'static str,
    },
}

/// How a first-party limitation was structurally removed.
///
/// The fourth stage of the ruling's arc, recorded so the whole arc reads
/// from one place. A removal names the row that took it, the structural
/// change it made, and the target-computed identity that proved the
/// shape really runs — because a removal nobody ran is a claim about a
/// registry rather than about a chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LimitationRemoval {
    /// The backlog row that took the removal.
    pub row: &'static str,
    /// What structurally changed, in the registry's own terms.
    pub change: &'static str,
    /// The run-of-record identity of the first shape it unlocked, where
    /// one has run.
    ///
    /// `None` is a removal that is REAL at the registry and that nothing
    /// has yet carried to a node. The register keeps the two apart on
    /// purpose: a vocabulary that can express a shape and a chain that has
    /// accepted one are different facts, and this is the register whose
    /// whole reason for existing is not collapsing facts of different
    /// kinds into one word.
    pub proven_by: Option<&'static str>,
}

/// The two-output floor's removal, recorded once.
///
/// One constant rather than two literals, because the census row that
/// cites it and the limitation that reports it must not be able to
/// disagree about what happened. A test holds them equal; naming the
/// value makes the test a statement about wiring rather than about
/// somebody having copied a paragraph correctly.
const TWO_OUTPUT_FLOOR_REMOVAL: LimitationRemoval = LimitationRemoval {
    row: "T5-041",
    change: "The cardinality clause stopped counting outputs and started asking whether a short \
             manifest DECLARES the single-output fully-solved balancing form, which a new \
             `SoleBalancing` role states. The floor still refuses a lone output that does not \
             declare it, so nothing was relaxed; the zero-blinder degeneracy is answered by the \
             registry's existing `DegenerateBalancingScalar` refusal, left standing and now \
             load-bearing.",
    proven_by: Some(crate::live_multi_shapes::run_of_record::STRICT_ONE_TO_ONE_ACCEPTED_TXID),
};

/// The absent fee role's removal, recorded once.
///
/// Its `proven_by` is `None`, and that absence is the honest half of this
/// record. The fixture vocabulary really does express a fee output now —
/// explicit-valued, held out of the solve at a zero blinder, and required
/// to carry an empty program — and no node has been offered one, because
/// the layers between the registry and a chain have not learned the role.
const ABSENT_FEE_ROLE_REMOVAL: LimitationRemoval = LimitationRemoval {
    row: "T5-042",
    change: "A `Fee` member joined the fixture output role vocabulary, and the empty-program \
             clause moved from every output alike onto the role: a fee output is REQUIRED to \
             carry an empty program rather than excused from carrying one, and every other role \
             still needs a program. The fee is held out of the blinder solve at a zero blinder, \
             its opening is absent rather than zero-filled, and the admitted-prefix rule reads on \
             the outputs that have commitments instead of on the output count.",
    proven_by: None,
};

/// A first-party convention that refuses a shape consensus admits.
///
/// Each member names a rule of this repository's own fixture registry,
/// the model that rule came out of, and the removal path that would end
/// it. This is the "explicitly labeled, pinned and explained" half of
/// the ruling the register implements; the removal is filed and NOT
/// taken here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Limitation {
    /// The registry refuses any manifest of fewer than two outputs.
    ///
    /// Argued at `(´[PLAN-rule:shapes:two-output-floor]´)`, which
    /// carries the revision surface this row cannot.
    TwoOutputFloor,
    /// The fixture vocabulary has no fee output role.
    ///
    /// Argued at `(´[PLAN-rule:shapes:absent-fee-role]´)`.
    AbsentFeeRole,
    /// This lane funds only a predecessor whose blinders cancel.
    ///
    /// The limitation the two-output floor's removal UNCOVERED, and the
    /// clearest evidence that removing a wall does not always reveal open
    /// ground behind it. The registry now admits a merge; this ceremony
    /// still cannot build one, because the only coins it can offer a
    /// merge are the two halves of an inverse pair.
    ///
    /// It is a first-party limitation like the others and it is a
    /// different KIND of one: not a rule the registry states, but a
    /// predecessor the ceremony happens to fund. Argued at
    /// `(´[PLAN-rule:shapes:canceling-predecessor]´)`.
    CancelingPredecessorOnly,
}

impl Limitation {
    /// The source row that refuses the shape.
    #[must_use]
    pub const fn refused_at(self) -> &'static str {
        match self {
            Self::TwoOutputFloor => {
                "packages/target-elements-conformance/src/confidential_fixture.rs, the \
                 `outputs.len() < 2` clause of `register_with_source`"
            }
            Self::AbsentFeeRole => {
                "packages/target-elements-conformance/src/confidential_fixture.rs, the \
                 `output_program.is_empty()` clause of `register_with_source`"
            }
            Self::CancelingPredecessorOnly => {
                "packages/target-elements-conformance/src/confidential_fixture.rs, the \
                 zero-solution clause of `derive_at_counter`, reached because \
                 packages/vectors/src/live_multi_shapes.rs funds one predecessor whose two \
                 output blinders are ordered additive inverses"
            }
        }
    }

    /// Why the convention exists, stated as the model it came out of.
    ///
    /// Recording this matters as much as recording the refusal. A
    /// convention whose reason is written down can be argued with; one
    /// whose reason was never written down gets defended as though it
    /// were consensus.
    #[must_use]
    pub const fn convention(self) -> &'static str {
        match self {
            Self::TwoOutputFloor => {
                "The balancing-output model. A manifest names exactly one balancing output whose \
                 blinder is SOLVED to close the tally, and the registry's parity discipline \
                 searches over the freely chosen blinders of the others. With one output there \
                 is no other, so the model has nothing to search — which is a fact about the \
                 model and not about the arithmetic, since a lone output's blinder is fully \
                 determined by the input blinder sum and determining it is exactly what solving \
                 means."
            }
            Self::AbsentFeeRole => {
                "The absent fee role. Every fixture output must carry a nonempty output program, \
                 and a fee output carries an empty one by the target's own definition of a fee. \
                 The vocabulary has no member for an output that is explicit, unspendable and \
                 outside the blinder solve, so the shape is inexpressible rather than rejected."
            }
            Self::CancelingPredecessorOnly => {
                "The single funded predecessor. This ceremony funds ONE confidential predecessor \
                 from an EXPLICIT input, so that predecessor's own input blinder sum is zero and \
                 its two output blinders come out ordered additive inverses. Every two-input \
                 merge the ceremony could offer therefore consumes both halves of an inverse pair \
                 and presents a ZERO input blinder sum, which forces the lone output's blinder to \
                 zero — a commitment of exactly the value times the value generator, hiding \
                 nothing while the tally still balances. The registry refuses it, correctly. The \
                 limitation is the ceremony's, not the registry's: a merge of coins whose \
                 blinders do not cancel registers today."
            }
        }
    }

    /// The named path that would structurally remove the limitation.
    #[must_use]
    pub const fn removal_path(self) -> RemovalPath {
        match self {
            Self::TwoOutputFloor => RemovalPath::SingleOutputSolvedBalancingForm,
            Self::AbsentFeeRole => RemovalPath::FeeOutputRole,
            Self::CancelingPredecessorOnly => RemovalPath::NonCancelingPrecursor,
        }
    }

    /// How this limitation was structurally removed, where it has been.
    ///
    /// `None` is the honest answer for a limitation still standing, and
    /// the register is careful not to let a filed removal path read as a
    /// taken one: [`Self::removal_path`] says what WOULD end it, and this
    /// says what DID.
    #[must_use]
    pub const fn removal(self) -> Option<LimitationRemoval> {
        match self {
            Self::TwoOutputFloor => Some(TWO_OUTPUT_FLOOR_REMOVAL),
            Self::AbsentFeeRole => Some(ABSENT_FEE_ROLE_REMOVAL),
            Self::CancelingPredecessorOnly => None,
        }
    }

    /// Whether the refusal guards consensus only by accident.
    ///
    /// True where a refusal that happens to block a consensus-impossible
    /// shape is not reasoning about consensus at all. The two-output
    /// floor is such a refusal: it counts outputs, and would refuse the
    /// impossible fee-only shape and the perfectly possible merge with
    /// the same message and the same indifference. A reader who saw only
    /// the refusal would learn nothing about which of the two consensus
    /// forbids, so the register says which.
    #[must_use]
    pub const fn guards_only_incidentally(self) -> bool {
        match self {
            Self::TwoOutputFloor => true,
            Self::AbsentFeeRole | Self::CancelingPredecessorOnly => false,
        }
    }
}

/// A named structural removal for a limitation.
///
/// FILED, not implemented. Each member is a design this register commits
/// to naming and to nothing else; the work sits in the feature-request
/// register, and no part of it is taken by this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemovalPath {
    /// A manifest form whose single output is fully solved.
    SingleOutputSolvedBalancingForm,
    /// A fee member of the fixture output role vocabulary.
    FeeOutputRole,
    /// A precursor transaction whose outputs do not cancel.
    NonCancelingPrecursor,
}

impl RemovalPath {
    /// What the removal would have to build.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::SingleOutputSolvedBalancingForm => {
                "Extend the registry with a single-output fully-solved balancing form: a manifest \
                 whose one output is balancing and carries NO freely chosen blinder, taking the \
                 input blinder sum directly. The parity search has nothing to search and \
                 degenerates to a well-formedness check, which the code should say plainly rather \
                 than claim a discriminating power it would not have. This admits both the strict \
                 one-to-one and the private merge, and the two-output case stays bit-for-bit as \
                 it is."
            }
            Self::FeeOutputRole => {
                "Add a fee member to the fixture output role vocabulary: explicit-valued, held \
                 OUT of the blinder solve at a zero blinder, and REQUIRED to carry an empty \
                 output program rather than merely permitted one, so that the role is checked and \
                 not just excused from the nonempty-program clause. The clause then reads on the \
                 role instead of on every output alike."
            }
            Self::NonCancelingPrecursor => {
                "Chain a PRECURSOR submission whose outputs do not cancel, and merge two of \
                 those. A three-output precursor's blinders sum to the coin it consumed, so any \
                 TWO of them sum to that total less the third — nonzero for no reason anybody has \
                 to arrange. The ceremony already mines each acceptance rather than leaving it in \
                 the mempool, precisely so the coins it creates are visible to a later step, so \
                 what is missing is a second submission stage and not a capability. Nothing in \
                 the registry changes: it admits the merge already."
            }
        }
    }

    /// The degeneracy a single-output form has to name.
    ///
    /// Returns `Some` only for the merge form, and this is the reason
    /// the form is filed with a warning attached rather than filed
    /// plainly.
    #[must_use]
    pub const fn degeneracy(self) -> Option<&'static str> {
        match self {
            Self::SingleOutputSolvedBalancingForm => Some(
                "The forced blinder can be ZERO, and then it hides nothing. A merge's single \
                 output takes the SUM of the consumed coins' blinders, so merging the two halves \
                 of this repository's inverse-pair dual-parity predecessor — whose blinders \
                 cancel by construction, which is what makes it an inverse pair — forces that \
                 sum to zero. The output commitment is then exactly `v*H`: a point anyone can \
                 recompute from a guessed value, carrying a blinded output's form and none of its \
                 hiding. The form is still sound, and the tally still balances; what fails is \
                 confidentiality, silently. A predecessor whose blinders do NOT cancel avoids it, \
                 so the removal must either require a non-canceling predecessor or refuse a \
                 solved zero blinder outright.",
            ),
            Self::FeeOutputRole | Self::NonCancelingPrecursor => None,
        }
    }
}

/// One row of the register: a shape and both its verdicts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeCensusEntry {
    /// The shape this row is about.
    pub shape: BlindedShape,
    /// What consensus admits, and on what evidence.
    pub consensus: ConsensusVerdict,
    /// What this workspace's registry does about it.
    pub first_party: FirstPartyStatus,
}

/// The register: every enumerated shape, with both verdicts.
///
/// Total over [`BlindedShape::ALL`] by construction — the match has no
/// catch-all, so a shape added to the enumeration fails to compile here
/// until it is censused, which is the property that makes the register
/// a register rather than a list somebody keeps up to date by hand.
///
/// The strict one-to-one and the merge share an arm because they share
/// every fact this register holds: both are one-output shapes, both are
/// possible on the tally, and both draw the same cardinality refusal.
/// What separates them is the merge's zero-blinder degeneracy, which is
/// a property of the removal path rather than of the row.
#[must_use]
pub const fn census_entry(shape: BlindedShape) -> ShapeCensusEntry {
    let (consensus, first_party) = match shape {
        // The strict one-to-one, which the two-output floor refused until
        // the floor was removed and which a target has now accepted. Its
        // row keeps citing the removed limitation, because a row that said
        // only "constructible" would have lost the wall's history.
        BlindedShape::OneToOne => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::STRICT_ONE_TO_ONE_ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAfterRemoval {
                removed: Limitation::TwoOutputFloor,
                removal: TWO_OUTPUT_FLOOR_REMOVAL,
            },
        ),
        // The merge, which the SAME removal did not free. The floor no
        // longer refuses it and the registry admits it; what refuses it
        // now is the zero blinder its only available inputs would force,
        // which is a fact about this ceremony's predecessor rather than
        // about the registry's rules. The wall moved from a cardinality
        // accident to the confidentiality property that actually matters,
        // and the row says which wall it is standing at.
        BlindedShape::TwoToOne => (
            ConsensusVerdict::SourceDerivedPossible,
            FirstPartyStatus::RefusedByConvention {
                refusal: RegistrationRefusal::Derivation {
                    refusal: target_elements_conformance::confidential_fixture::FixtureDerivationRefusal::DegenerateBalancingScalar,
                },
                limitation: Limitation::CancelingPredecessorOnly,
            },
        ),
        // The fee-bearing shape. The vocabulary now expresses it and no
        // node has been offered one, which is two facts this register
        // refuses to round into either "refused" or "observed".
        BlindedShape::OneToOneWithFee => (
            ConsensusVerdict::SourceDerivedPossible,
            FirstPartyStatus::ExpressibleAndUnrun {
                removed: Limitation::AbsentFeeRole,
                removal: ABSENT_FEE_ROLE_REMOVAL,
                stops_at: "packages/vectors/src/live_proof_bearing_observation.rs, the fixture \
                           projection, which refuses `FeeRoleNotProjectable`: the materializer's \
                           own output-role vocabulary has no fee member, its per-output stage \
                           would compute a commitment and a range proof for an output that must \
                           carry an explicit value and no witness, and the executor adapter's \
                           fixture catalogue and parity search read every output as a committed \
                           one. The projection refuses rather than mapping a fee onto the \
                           balancing role, which would have produced a blinded fee output — not a \
                           fee at the target, and a silently wrong transaction rather than an \
                           honest stop.",
            },
        ),
        BlindedShape::OneToTwo => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_private_restart::run_of_record::ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAndObserved,
        ),
        BlindedShape::OneToThree => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::SPLIT_ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAndObserved,
        ),
        BlindedShape::TwoToTwo => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::SEVERAL_OWNERS_ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAndObserved,
        ),
        BlindedShape::TwoToThree => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::MANY_TO_MANY_ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAndObserved,
        ),
        BlindedShape::FeeOnly => (
            ConsensusVerdict::SourceDerivedImpossible,
            FirstPartyStatus::RefusalGuardsConsensus {
                refusal: RegistrationRefusal::OutputSetTooSmall { found: 1 },
            },
        ),
    };
    ShapeCensusEntry {
        shape,
        consensus,
        first_party,
    }
}

#[cfg(test)]
mod tests {
    use target_elements_conformance::confidential_fixture::{
        ConfidentialFixtureOutput, FixtureOutputRole, RegistrationRefusal,
    };

    use super::{
        BlindedShape, ConsensusVerdict, FirstPartyStatus, Limitation, RemovalPath, census_entry,
    };
    use crate::live_proof_bearing_observation::registry_refusal_for;

    /// A disposable asset for the registry drives below.
    ///
    /// Public test material under ADR-015; it names no chain.
    const CENSUS_ASSET: [u8; 32] = [0x3b; 32];

    /// An input blinder sum the registry admits as a scalar.
    ///
    /// # It stopped being a placeholder
    ///
    /// It used to be one. Every drive below reached a cardinality or role
    /// clause that runs BEFORE any blinder arithmetic, so the value never
    /// mattered and this said so.
    ///
    /// For the MERGE row it now matters, and it is the right value rather
    /// than a convenient one. This ceremony funds one predecessor from an
    /// explicit input, so that predecessor's two output blinders are
    /// ordered additive inverses and a merge consuming both presents
    /// exactly this sum: zero. The merge's recomputed refusal is therefore
    /// a statement about the coins this lane really has, not about an
    /// arbitrary scalar.
    const CENSUS_BLINDER_SUM: [u8; 32] = [0_u8; 32];

    /// Drives the registry with a manifest of the shape's output arity.
    ///
    /// The point of the register's refused rows: the refusal is
    /// RECOMPUTED against the live registry rather than copied out of a
    /// comment, so a registry that changed its mind would fail this
    /// module rather than leave it quietly stale.
    ///
    /// It lives in the test module because the crate's registry drivers
    /// do — the register itself is a statement about the registry, and
    /// only its verification needs to run one.
    ///
    /// Amounts and programs are the shape's own. A fee-bearing shape
    /// gets an EMPTY program for its fee output, because an empty
    /// `scriptPubKey` is what makes an output a fee at the target
    /// (`src/primitives/transaction.h:324-327`) — exactly the property
    /// the fixture vocabulary has no room for.
    ///
    /// The fee output goes LAST, where the shared manifest builder casts
    /// the final output as the BALANCING one. That is not a modelling
    /// choice made here but a second face of the same absent role: the
    /// builder has no way to say "explicit, outside the solve", so the
    /// one output that must never balance arrives cast as the output
    /// that does. The refusal reached is the empty-program clause either
    /// way.
    fn recomputed_registry_refusal(shape: BlindedShape) -> Option<RegistrationRefusal> {
        let count = shape.outputs();
        let fee_at = (shape.fee_outputs() == 1).then(|| count - 1);
        let outputs: Vec<ConfidentialFixtureOutput> = (0..count)
            .map(|index| {
                let is_fee = Some(index) == fee_at;
                ConfidentialFixtureOutput {
                    // The role is STATED rather than read off the output
                    // order, the shared builder having stopped assigning
                    // it by position. What is stated is what this helper
                    // used to be handed implicitly — the last output
                    // balances — so the refusals below are recomputed
                    // against the same manifests as before and no census
                    // row moves because a builder changed.
                    //
                    // The fee output is cast as balancing here, and that
                    // is not a modelling choice: the vocabulary has no fee
                    // member to cast it as, which is the absent role this
                    // register records. It is the second face of the same
                    // absence the positional builder wore.
                    // The role is STATED, the shared builder having
                    // stopped assigning it by position.
                    //
                    // A shape of ONE output declares the single-output
                    // fully-solved form, because that is what such a shape
                    // IS and a drive that withheld the declaration would be
                    // recomputing the refusal for a manifest nobody would
                    // write. The test is the manifest's whole output count
                    // and not its blinded count: the form is a statement
                    // about the manifest, so a lone blinded output sitting
                    // beside a fee output is not it.
                    //
                    // The fee output states the FEE role, which the
                    // vocabulary now has. It used to be cast as balancing
                    // — the one output that must never balance, wearing
                    // the role of the output that does — because there was
                    // nothing else to cast it as.
                    //
                    // The balancing output is therefore the last NON-fee
                    // output rather than the last output, which is the
                    // same correction said a second way.
                    role: if is_fee {
                        FixtureOutputRole::Fee
                    } else if count == 1 {
                        FixtureOutputRole::SoleBalancing
                    } else if index + 1 == count - shape.fee_outputs() {
                        FixtureOutputRole::Balancing
                    } else {
                        FixtureOutputRole::Primary
                    },
                    semantic_amount: 100_000_000_u64,
                    output_program: if is_fee { Vec::new() } else { vec![0x51_u8] },
                }
            })
            .collect();
        registry_refusal_for(
            &format!("ctf-v1/census-{}", shape.handle()),
            CENSUS_ASSET,
            CENSUS_BLINDER_SUM,
            outputs,
        )
    }

    /// The register is total over its enumeration, with no shape
    /// censused twice.
    ///
    /// Totality is what lets a reader trust the closure rule: the
    /// enumeration states what the space is, and this says every member
    /// of it has a row.
    #[test]
    fn the_register_is_total_over_the_enumeration() {
        assert_eq!(BlindedShape::ALL.len(), 8);
        let mut handles: Vec<&str> = BlindedShape::ALL
            .iter()
            .map(|shape| {
                let entry = census_entry(*shape);
                assert_eq!(
                    entry.shape, *shape,
                    "a row censuses the shape it is filed under"
                );
                shape.handle()
            })
            .collect();
        handles.sort_unstable();
        let censused = handles.len();
        handles.dedup();
        assert_eq!(handles.len(), censused, "no shape is censused twice");
    }

    /// The recorded consensus verdicts agree with the tally arithmetic.
    ///
    /// The register does not get to state a verdict its own derivation
    /// denies. The predicate is computed from the shape alone; a row
    /// claiming possibility for a shape with no blinded output, or
    /// impossibility for one that has one, fails here.
    #[test]
    fn the_recorded_verdicts_agree_with_the_tally_predicate() {
        for shape in BlindedShape::ALL {
            let absorbable = shape.blinder_sum_is_absorbable();
            match census_entry(shape).consensus {
                ConsensusVerdict::ObservedAccepted { .. }
                | ConsensusVerdict::SourceDerivedPossible => assert!(
                    absorbable,
                    "{} is recorded possible, so some output must absorb the blinder sum",
                    shape.handle(),
                ),
                ConsensusVerdict::SourceDerivedImpossible => assert!(
                    !absorbable,
                    "{} is recorded impossible, so no output may absorb the blinder sum",
                    shape.handle(),
                ),
            }
        }
    }

    /// Exactly one shape is impossible, and it is the fee-only one.
    ///
    /// Stated as its own fact because it is the register's sharpest
    /// claim: everything else the small-shape window holds, consensus
    /// admits.
    #[test]
    fn the_fee_only_shape_is_the_only_impossible_one() {
        let impossible: Vec<BlindedShape> = BlindedShape::ALL
            .into_iter()
            .filter(|shape| {
                matches!(
                    census_entry(*shape).consensus,
                    ConsensusVerdict::SourceDerivedImpossible
                )
            })
            .collect();
        assert_eq!(impossible, vec![BlindedShape::FeeOnly]);
        assert_eq!(BlindedShape::FeeOnly.blinded_outputs(), 0);
        assert_eq!(BlindedShape::FeeOnly.fee_outputs(), 1);
    }

    /// Every refused row's refusal is the one the registry really
    /// returns.
    ///
    /// The register's central claim, RECOMPUTED. Each refused shape's
    /// manifest is built and handed to the live registry, and the
    /// refusal that comes back must be the censused one. Nothing here
    /// asserts a remembered value.
    #[test]
    fn every_refused_row_recomputes_its_refusal_against_the_registry() {
        let mut refused = 0_usize;
        for shape in BlindedShape::ALL {
            let censused = match census_entry(shape).first_party {
                FirstPartyStatus::ConstructibleAndObserved
                | FirstPartyStatus::ConstructibleAfterRemoval { .. }
                | FirstPartyStatus::ExpressibleAndUnrun { .. } => continue,
                FirstPartyStatus::RefusedByConvention { refusal, .. }
                | FirstPartyStatus::RefusalGuardsConsensus { refusal } => refusal,
            };
            let recomputed = recomputed_registry_refusal(shape)
                .unwrap_or_else(|| panic!("the registry refuses {}", shape.handle()));
            assert_eq!(
                recomputed,
                censused,
                "the censused refusal for {} is the one the registry returns",
                shape.handle(),
            );
            refused += 1;
        }
        assert_eq!(
            refused, 2,
            "two of the eight shapes are refused: this lane's merge, and the impossible one",
        );
    }

    /// Every constructible row cites a run-of-record identity.
    ///
    /// Cited rather than copied: the expected values below are the
    /// `run_of_record` constants themselves, so this compares the
    /// register against the evidence rather than against a literal
    /// somebody transcribed.
    #[test]
    fn every_observed_row_cites_its_run_of_record_identity() {
        let expected = [
            (
                BlindedShape::OneToOne,
                crate::live_multi_shapes::run_of_record::STRICT_ONE_TO_ONE_ACCEPTED_TXID,
            ),
            (
                BlindedShape::OneToTwo,
                crate::live_private_restart::run_of_record::ACCEPTED_TXID,
            ),
            (
                BlindedShape::OneToThree,
                crate::live_multi_shapes::run_of_record::SPLIT_ACCEPTED_TXID,
            ),
            (
                BlindedShape::TwoToTwo,
                crate::live_multi_shapes::run_of_record::SEVERAL_OWNERS_ACCEPTED_TXID,
            ),
            (
                BlindedShape::TwoToThree,
                crate::live_multi_shapes::run_of_record::MANY_TO_MANY_ACCEPTED_TXID,
            ),
        ];
        for (shape, identity) in expected {
            let entry = census_entry(shape);
            assert_eq!(
                entry.consensus,
                ConsensusVerdict::ObservedAccepted { identity },
                "{} cites its own run of record",
                shape.handle(),
            );
            assert!(
                matches!(
                    entry.first_party,
                    FirstPartyStatus::ConstructibleAndObserved
                        | FirstPartyStatus::ConstructibleAfterRemoval { .. }
                ),
                "{} is observed, so its first-party status is a constructible one",
                shape.handle(),
            );
        }
        assert_eq!(expected.len(), 5, "five of the eight shapes have been run");
    }

    /// The observed rows' cardinalities match the ceremonies' own
    /// recorded counts.
    ///
    /// The multi-shape ceremony writes down how many receipts each
    /// shape consumed and how many outputs it created. If the register
    /// described a different shape than the run it cites, these would
    /// disagree.
    #[test]
    fn the_observed_cardinalities_match_the_recorded_run_counts() {
        use crate::live_multi_shapes::run_of_record::{OUTPUT_COUNTS, RECEIPT_LEAVES};

        let ordered = [
            BlindedShape::OneToThree,
            BlindedShape::TwoToThree,
            BlindedShape::TwoToTwo,
            BlindedShape::OneToOne,
        ];
        for (index, shape) in ordered.into_iter().enumerate() {
            assert_eq!(
                shape.blinded_inputs(),
                RECEIPT_LEAVES[index],
                "{} consumed the recorded number of receipts",
                shape.handle(),
            );
            assert_eq!(
                shape.outputs(),
                OUTPUT_COUNTS[index],
                "{} created the recorded number of outputs",
                shape.handle(),
            );
        }
        assert_eq!(
            crate::live_private_restart::run_of_record::RECEIPT_LEAVES,
            BlindedShape::OneToTwo.blinded_inputs(),
            "the one-to-two control consumed the recorded number of receipts",
        );
    }

    /// Every consensus-possible refused row carries a removal path.
    ///
    /// The ruling's second half, held structurally: a shape consensus
    /// admits and this workspace refuses may not sit in the register
    /// with the refusal recorded and nothing said about ending it.
    #[test]
    fn every_consensus_possible_refusal_names_a_removal_path() {
        let mut paths = Vec::new();
        for shape in BlindedShape::ALL {
            let entry = census_entry(shape);
            let FirstPartyStatus::RefusedByConvention { limitation, .. } = entry.first_party else {
                continue;
            };
            assert_ne!(
                entry.consensus,
                ConsensusVerdict::SourceDerivedImpossible,
                "{} is refused by convention, so consensus must admit it",
                shape.handle(),
            );
            assert_ne!(limitation.refused_at(), "", "the refusing row is named");
            assert_ne!(limitation.convention(), "", "the convention is explained");
            assert_ne!(
                limitation.removal_path().description(),
                "",
                "the removal path is described",
            );
            paths.push(limitation.removal_path());
        }
        paths.sort_unstable();
        paths.dedup();
        assert_eq!(
            paths,
            vec![RemovalPath::NonCancelingPrecursor],
            "one limitation still refuses a consensus-possible shape, and it files a path",
        );
    }

    /// A removed limitation stays cited from the row it used to refuse,
    /// and carries what ended it.
    ///
    /// The ruling's arc — labeled, pinned, explained, REMOVED — held
    /// structurally rather than left to prose. A row whose limitation was
    /// removed must still name that limitation, must still be able to say
    /// what the convention was, and must carry a removal naming the row
    /// that took it and the target-computed identity that proved the
    /// shape really runs.
    #[test]
    fn a_removed_limitation_keeps_its_history_and_names_what_ended_it() {
        let mut removed = Vec::new();
        for shape in BlindedShape::ALL {
            let FirstPartyStatus::ConstructibleAfterRemoval {
                removed: limitation,
                removal,
            } = census_entry(shape).first_party
            else {
                continue;
            };

            // The three earlier stages of the arc survive the fourth.
            assert_ne!(
                limitation.refused_at(),
                "",
                "the refusing row is still named"
            );
            assert_ne!(
                limitation.convention(),
                "",
                "the convention is still explained"
            );

            // And the fourth stage is recorded rather than implied.
            assert_eq!(
                limitation.removal(),
                Some(removal),
                "the row's removal is the limitation's own",
            );
            assert_ne!(removal.change, "", "the structural change is stated");
            assert_eq!(
                removal.proven_by.map(str::len),
                Some(64),
                "a removal recorded on an observed row is proven by a target-computed identity",
            );

            // The consensus half must have moved with it. A removal that
            // did not end in an acceptance is a claim about a registry.
            assert!(
                matches!(
                    census_entry(shape).consensus,
                    ConsensusVerdict::ObservedAccepted { .. }
                ),
                "{} records a removal, so a node must have accepted it",
                shape.handle(),
            );
            removed.push(limitation);
        }
        assert_eq!(
            removed,
            vec![Limitation::TwoOutputFloor],
            "exactly the two-output floor has been removed and run",
        );

        // A limitation still standing does NOT claim a removal, so a
        // filed path cannot read as a taken one.
        assert_eq!(Limitation::CancelingPredecessorOnly.removal(), None);
    }

    /// A removal that nothing has run says so, and says where a run
    /// stops.
    ///
    /// The register's sharpest discipline, applied to its own work. The
    /// fee role was really added and a fee-bearing manifest really
    /// registers — but a vocabulary that CAN express a shape is not a
    /// chain that HAS accepted one, and the whole reason this register
    /// exists is that those two had been collapsing into one word.
    ///
    /// So the row carries no identity, its removal carries no identity,
    /// and the place a run stops is named in the stopping layer's own
    /// terms rather than left as "not yet".
    #[test]
    fn an_unrun_removal_claims_no_acceptance_and_names_where_it_stops() {
        let mut expressible = Vec::new();
        for shape in BlindedShape::ALL {
            let FirstPartyStatus::ExpressibleAndUnrun {
                removed,
                removal,
                stops_at,
            } = census_entry(shape).first_party
            else {
                continue;
            };

            assert_eq!(
                removal.proven_by,
                None,
                "{} has not run, so its removal proves nothing about a chain",
                shape.handle(),
            );
            assert_ne!(stops_at, "", "the stopping layer is named");
            assert_eq!(removed.removal(), Some(removal));

            // And the consensus half must NOT have moved. An expressible
            // shape nobody has submitted is source-derived, and a register
            // that let this age into an observation would be the failure
            // it was built to prevent.
            assert_eq!(
                census_entry(shape).consensus,
                ConsensusVerdict::SourceDerivedPossible,
                "{} is expressible and unrun, so its evidence class is a derivation",
                shape.handle(),
            );
            expressible.push(shape);
        }
        assert_eq!(expressible, vec![BlindedShape::OneToOneWithFee]);
    }

    /// Removing the floor did not free the merge, and the register says
    /// which wall it is standing at now.
    ///
    /// # The observation this replaces
    ///
    /// The register's sharpest observation used to be that the two-output
    /// floor returned the SAME refusal for the impossible fee-only shape
    /// and the perfectly possible merge, so it guarded consensus by
    /// accident and a reader taking the refusal as a verdict would be
    /// wrong about one of the two.
    ///
    /// That coincidence has ENDED, and ending it is most of what the
    /// removal was worth. The merge no longer meets a cardinality wall at
    /// all: it meets the zero blinder its only available inputs would
    /// force, which is the confidentiality property that actually
    /// separates a merge worth building from one that hides nothing. The
    /// two shapes now draw different refusals, and each refusal is about
    /// its own shape.
    #[test]
    fn the_merge_and_the_fee_only_shape_no_longer_share_a_refusal() {
        let fee_only = recomputed_registry_refusal(BlindedShape::FeeOnly)
            .expect("the registry refuses the fee-only shape");
        let merge = recomputed_registry_refusal(BlindedShape::TwoToOne)
            .expect("the registry refuses this lane's merge");
        assert_ne!(
            fee_only, merge,
            "the accident the register recorded has ended: the two draw different refusals",
        );
        assert_eq!(
            fee_only,
            RegistrationRefusal::OutputSetTooSmall { found: 1 },
            "a lone FEE output can never declare the solved form, so the floor still holds it",
        );
        assert!(
            matches!(merge, RegistrationRefusal::Derivation { .. }),
            "the merge's wall is now its blinders and no longer its output count",
        );
    }

    /// The merge removal path names its zero-blinder degeneracy, and
    /// the fee one has none to name.
    ///
    /// The degeneracy is the reason the single-output form cannot be
    /// filed as a plain convenience: for the inverse-pair predecessor
    /// the forced blinder is zero and the "blinded" output hides
    /// nothing. A removal path that did not carry the warning would be
    /// filing a confidentiality hole as a feature.
    #[test]
    fn the_single_output_removal_path_names_the_zero_blinder_degeneracy() {
        let degeneracy = RemovalPath::SingleOutputSolvedBalancingForm
            .degeneracy()
            .expect("the single-output form names its degeneracy");
        assert!(degeneracy.contains("ZERO"));
        assert!(degeneracy.contains("inverse-pair"));
        assert!(
            RemovalPath::FeeOutputRole.degeneracy().is_none(),
            "an explicit fee output has no blinder to degenerate",
        );
    }

    /// The two-output floor guards the impossible shape only by
    /// accident, and the register says so.
    ///
    /// It counts outputs. It refuses the impossible fee-only shape and
    /// the perfectly possible merge with the same message, so a reader
    /// who took the refusal as a consensus verdict would draw the wrong
    /// conclusion about one of them — which is precisely what this
    /// register exists to prevent.
    #[test]
    fn the_cardinality_floor_guards_consensus_only_incidentally() {
        assert!(Limitation::TwoOutputFloor.guards_only_incidentally());
        assert!(!Limitation::AbsentFeeRole.guards_only_incidentally());
        assert!(!Limitation::CancelingPredecessorOnly.guards_only_incidentally());

        // The floor still refuses the one consensus-IMPOSSIBLE shape, and
        // still for a reason that has nothing to do with consensus: it
        // counts outputs. What has changed is that it no longer refuses a
        // possible shape with the same message, which is held next door by
        // `the_merge_and_the_fee_only_shape_no_longer_share_a_refusal`.
        let fee_only = recomputed_registry_refusal(BlindedShape::FeeOnly)
            .expect("the registry refuses the fee-only shape");
        assert_eq!(
            fee_only,
            RegistrationRefusal::OutputSetTooSmall { found: 1 }
        );
    }
}
