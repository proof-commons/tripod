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
//! `run_of_record` identity a ceremony produced; no verdict here
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
    /// No blinded input at all, to two blinded outputs.
    ///
    /// Blinding on ENTRY: explicit receipts spent into confidential
    /// ones. It sits at the domain's edge rather than inside it — every
    /// other member consumes at least one blinded input and this member
    /// consumes none — which is why admitting it widened the closure
    /// rule rather than adding a row under it.
    EntryCrossing,
    /// Two blinded inputs to two EXPLICIT outputs and one blinded
    /// absorber.
    ///
    /// Unblinding on EXIT. The absorber is an ordinary blinded
    /// destination at a declared position, and it is the whole reason
    /// the shape is possible: the consumed blinder sum is nonzero and
    /// explicit outputs contribute zero to it, so without the absorber
    /// there is nothing for it to land on.
    ExitCrossing,
    /// Two blinded inputs to nothing but EXPLICIT outputs.
    ///
    /// The exit crossing with its absorber removed, and the corner where
    /// consensus and this workspace's registry refuse the same shape for
    /// DIFFERENT reasons — which is the case this register exists to
    /// keep apart. It is filed and never attempted.
    FullyUnblinding,
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
    /// verdict depends on NOTHING but whether the consumed blinder sum
    /// has somewhere to land. Two facts decide that and no third does —
    /// whether any input is blinded, and whether any output is. Every
    /// combination of those two appears in the window, so every shape
    /// OUTSIDE it inherits the verdict of whichever case it falls into,
    /// and adding one would restate a row rather than add one.
    ///
    /// # The domain the crossing wave widened
    ///
    /// The rule used to open "at least one blinded input", and that was
    /// not a simplification: it was the whole reason the register could
    /// derive [`Self::blinded_outputs`] from the output count instead of
    /// stating it. A lane on which every input is blinded and every
    /// non-fee output with it has exactly two kinds of output, so the
    /// fee count determines the rest.
    ///
    /// Representation crossing breaks BOTH halves of that, in opposite
    /// directions, and the register had to learn both. An ENTRY crossing
    /// consumes no blinded input at all, which is outside the old domain
    /// rather than unlisted within it. An EXIT crossing carries explicit
    /// NON-fee outputs, which the derived blinded-output count would
    /// have counted as blinded — and it would then have computed
    /// `absorbable` for a shape by counting outputs that absorb nothing.
    /// That is why [`Self::blinded_outputs`] is now an axis each member
    /// states rather than a subtraction, and why the tally predicate
    /// asks about the INPUT side too.
    ///
    /// The window is otherwise chosen for the first-party half, where
    /// the counts DO matter: the registry's own clauses are cardinality
    /// clauses, and every shape this lane has built or been refused
    /// falls inside it.
    pub const ALL: [Self; 11] = [
        Self::OneToOne,
        Self::OneToOneWithFee,
        Self::OneToTwo,
        Self::OneToThree,
        Self::TwoToOne,
        Self::TwoToTwo,
        Self::TwoToThree,
        Self::FeeOnly,
        Self::EntryCrossing,
        Self::ExitCrossing,
        Self::FullyUnblinding,
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
            Self::TwoToOne
            | Self::TwoToTwo
            | Self::TwoToThree
            | Self::ExitCrossing
            | Self::FullyUnblinding => 2,
            // The one member that consumes none, and the reason the
            // domain has an edge rather than a floor.
            Self::EntryCrossing => 0,
        }
    }

    /// How many outputs the shape creates, the fee output included.
    #[must_use]
    pub const fn outputs(self) -> usize {
        match self {
            Self::OneToOne | Self::TwoToOne | Self::FeeOnly => 1,
            Self::OneToOneWithFee
            | Self::OneToTwo
            | Self::TwoToTwo
            | Self::EntryCrossing
            | Self::FullyUnblinding => 2,
            Self::OneToThree | Self::TwoToThree | Self::ExitCrossing => 3,
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
            | Self::TwoToThree
            | Self::EntryCrossing
            | Self::ExitCrossing
            | Self::FullyUnblinding => 0,
        }
    }

    /// How many outputs carry a blinded value.
    ///
    /// STATED per member, not derived. It used to be the output count
    /// less the fee output, on the premise that every non-fee output on
    /// this lane is blinded — true of every homogeneous shape and false
    /// of an exit crossing, whose whole point is explicit non-fee
    /// outputs. Left derived, the subtraction would have counted those
    /// as blinded and [`Self::blinder_sum_is_absorbable`] would have
    /// reported a shape absorbable by outputs that absorb nothing.
    ///
    /// A stated axis cannot make that mistake, and it cannot be made
    /// silently either: [`Self::explicit_destinations`] is the leftover,
    /// and a test requires the three counts to partition the output set,
    /// so a member whose axes disagreed with its own output count fails
    /// rather than computing a wrong verdict.
    #[must_use]
    pub const fn blinded_outputs(self) -> usize {
        match self {
            Self::OneToOne | Self::OneToOneWithFee | Self::TwoToOne | Self::ExitCrossing => 1,
            Self::OneToTwo | Self::TwoToTwo | Self::EntryCrossing => 2,
            Self::OneToThree | Self::TwoToThree => 3,
            Self::FeeOnly | Self::FullyUnblinding => 0,
        }
    }

    /// How many outputs are EXPLICIT receipt destinations.
    ///
    /// The outputs that are neither blinded nor the fee: real programs,
    /// real owners, public amounts. Zero for every shape but the two
    /// exit-side ones, and derived as the leftover rather than stated,
    /// because it is the count that must close the partition rather than
    /// a third independent fact.
    #[must_use]
    pub const fn explicit_destinations(self) -> usize {
        self.outputs() - self.fee_outputs() - self.blinded_outputs()
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
    /// # The second way a sum lands
    ///
    /// A shape with NO blinded input presents a sum that is already
    /// zero, and zero is absorbed by an all-explicit output set without
    /// anything having to hold it. So the correct statement of the rule
    /// is not "a blinded input forces a blinded output" but "the input
    /// blinder sum must equal the output blinder sum, and explicit
    /// outputs contribute zero" — and the entry crossing is the member
    /// that makes the difference between those two readings visible.
    ///
    /// The answer is derived from the shape alone and is compared
    /// against the entry's recorded [`ConsensusVerdict`] by
    /// `the_recorded_verdicts_agree_with_the_tally_predicate`, so the
    /// register cannot record a verdict its own arithmetic denies.
    #[must_use]
    pub const fn blinder_sum_is_absorbable(self) -> bool {
        self.blinded_inputs() == 0 || self.blinded_outputs() > 0
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
            Self::EntryCrossing => "entry-crossing",
            Self::ExitCrossing => "exit-crossing",
            Self::FullyUnblinding => "fully-unblinding",
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
    /// `run_of_record` constant a ceremony recorded, so a wave that
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
/// The second, independent verdict. Its members are the distinct ways a
/// first-party position can stand against the consensus one, and naming
/// them apart is what stops a local convention being read as a protocol
/// rule.
///
/// Two of them exist because limitations get REMOVED, and a removal is
/// not one event but two facts that arrive separately: a vocabulary
/// learns to express a shape, and a chain accepts one. The register
/// carries a member for each rather than rounding the first up to the
/// second.
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
    /// The shape was BUILT, OFFERED to a node, and refused by it.
    ///
    /// The sixth status, and it exists because the fifth could not say
    /// this. Expressible-and-unrun is a shape nobody has offered; this is
    /// a shape somebody offered and a target turned away, which is more
    /// than the one and less than an acceptance.
    ///
    /// The refusal is a TARGET verdict and the limitation behind it is
    /// this workspace's own, and holding both in one member is the point:
    /// a reader who saw only the target's string would conclude the
    /// protocol forbids the shape, and a reader who saw only the
    /// limitation would not know a node had ever been asked.
    SubmittedAndRefused {
        /// The convention that used to make the shape unbuildable.
        removed: Limitation,
        /// What ended that convention.
        removal: LimitationRemoval,
        /// The convention that refuses it now.
        limitation: Limitation,
        /// The target's own verdict, verbatim and unmapped.
        observed_detail: &'static str,
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

/// The canceling predecessor's removal, recorded once.
///
/// Its `proven_by` is an identity, and that is the whole difference
/// between this removal and the fee role's. The ceremony now funds a
/// three-output predecessor whose coins cancel in no pair, a merge of two
/// of them was built, and a node accepted it.
const CANCELING_PREDECESSOR_REMOVAL: LimitationRemoval = LimitationRemoval {
    row: "T5-045",
    change: "The ceremony gained a SECOND predecessor: three outputs, funded exactly as the \
             dual-parity one is -- one explicit input, the same zero input blinder sum, the same \
             profiles -- and different in its output count alone. Three blinders summing to zero \
             cancel in no pair, any two of them summing to the negation of the third, and no \
             blinder is ever zero, so a merge of two of its coins has a forced blinder that is \
             nonzero for a reason that can be stated. The consumed sum is now SUMMED over the \
             coins the shape names rather than stated from one predecessor's structure, and the \
             ceremony writes the forced blinder's nonzero-ness into its own transcript.",
    proven_by: Some(crate::live_multi_shapes::run_of_record::MERGE_ACCEPTED_TXID),
};

/// The absent fee projection's removal, recorded once.
///
/// Its `proven_by` is `None`, and the reason is not the reason the fee
/// role's own removal had one. The layers between the registry and a
/// chain HAVE learned the role now: a fee-bearing candidate was built
/// with a real fee output and offered to a node. The node refused it, at
/// a wall further on, so no identity exists to cite -- which is a
/// different sentence from nobody having tried.
const FEE_ROLE_PROJECTION_REMOVAL: LimitationRemoval = LimitationRemoval {
    row: "T5-045",
    change: "The materializer's own output-role vocabulary gained a `Fee` member, its per-output \
             stage gained a fee stage that emits an explicit value, an explicit asset, a null \
             nonce, an empty program and an empty witness entry and then asks the built output \
             whether it IS a fee by the target's own three-conjunct predicate, and both \
             projections state the role instead of sweeping it into a catch-all that would have \
             solved a blinder for it. The view's three opening scalars became optional, because a \
             fee output's opening is ABSENT rather than zero.",
    proven_by: None,
};

/// The sponsorless fee-bearing shape's removal, recorded once.
///
/// `proven_by` carries an identity because the shape RAN. The vocabulary
/// gained the member, a candidate was built against it, and a node
/// accepted and mined the candidate -- which is the only thing that
/// turns a filed path into a taken one.
const SPONSORLESS_FEE_REMOVAL: LimitationRemoval = LimitationRemoval {
    row: "T5-047",
    change: "The reviewed live-transfer shape vocabulary gained a fee axis of its own, carried on \
             the BOUNDS so the demonstration candidate unrolls exactly as before and every \
             recorded digest re-derives. A sponsorless form may now declare the target fee role, \
             so a fee destination is no longer counted as a receipt output. Four coupled readings \
             moved together: the output count, the family-range census, the isolation fragment's \
             fee clause, and the pattern census deciding whether that fragment is emitted at all. \
             Shape selection counts the declared fee positions out of the destinations before \
             matching, reading them off the OPENINGS' roles, which is where the lane already \
             decides a destination is a fee. The fee clause takes its asset from who funded the \
             fee -- the protocol asset for a self-paying form, Elements balancing per asset and a \
             sponsorless shape having no reserve-asset input -- and the explicit conservation \
             relation gained the fee as a term rather than an allowance. The fee-bearing \
             deployment is welded to the digest an empty program actually hashes to, the \
             demonstration keeping its fixture constant and its identities untouched.",
    proven_by: Some(crate::live_multi_shapes::run_of_record::FEE_BEARING_SUCCESSOR_IDENTITY),
};

/// The homogeneous-representation limitation's removal, recorded once.
///
/// `proven_by` carries an identity because a shape it freed RAN: a real
/// node accepted and mined the EXIT crossing. The removal is therefore
/// TAKEN and not merely filed, which is the only thing that turns a
/// named path into a taken one.
///
/// It proves the exit direction and it does not prove the entry one.
/// The two rows say so separately, and that separation is the register
/// working: one removal can free two shapes and be carried to a chain by
/// only one of them, and a record that reported the removal alone would
/// imply both had run.
const PER_SIDE_REPRESENTATION_REMOVAL: LimitationRemoval = LimitationRemoval {
    row: "T5-054",
    change: "A composition pairs one admitted representation plan to each SIDE of a transfer,              taking §6.5's own \"unless separately admitted\" clause rather than widening the              guide, and leaving the plan census at the two members §6.1 states exhaustively. The              constructor carries the composition and derives its representation from the CONSUMED              side, so a crossing deployment seats its crossing constructor at exactly the key a              coin is recognized under and no destination table widens. The coordinator's value              obligation dispatches on the composition rather than on one plan, which is what the              obligation was always about -- the side a transfer CREATES -- and the exit direction              gains a POSITIONAL value-form fragment requiring the explicit form at every              destination but the declared absorber and the confidential form at that one. The              absorber is a declared destination position inside the destination range, so it adds              no output family and the §10.4 closure argument is untouched. The registry gained an              explicit receipt destination role at a new transcript code, the opposite corner of              the three predicates from the fee, and the materializer builds one through its own              stage that asks the target for the OPPOSITE answer the fee stage asks for. Every              recorded digest re-derives bit-for-bit through all of it.",
    proven_by: Some(crate::live_multi_shapes::run_of_record::EXIT_CROSSING_ACCEPTED_TXID),
};

/// A first-party convention that refuses a shape consensus admits.
///
/// Each member names a convention of this repository's own, the model it
/// came out of, and the removal path that would end it. Most are rules
/// the fixture registry states; [`Self::CancelingPredecessorOnly`] is
/// not, and the difference is worth keeping — it is a coin the ceremony
/// happens to fund rather than a rule anybody wrote, which is a wall of a
/// different kind and one no registry change would move.
///
/// This is the "explicitly labeled, pinned and explained" half of the
/// ruling the register implements. [`Self::removal`] carries the fourth
/// stage where it has been reached, and returns `None` where it has not,
/// so a filed path can never read as a taken one.
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
    /// Nothing between the registry and a chain could carry a fee role.
    ///
    /// Minted here as a limitation because it STOOD as one. It was
    /// recorded before as the place a fee-bearing shape stopped, which
    /// said where the wall was without saying that it was a wall of this
    /// workspace's own making. Argued at
    /// `(´[PLAN-rule:shapes:absent-fee-role]´)`.
    AbsentFeeProjection,
    /// The reviewed live-transfer shape vocabulary has no sponsorless
    /// fee-bearing member.
    ///
    /// The THIRD layer, uncovered by the second removal exactly as the
    /// second was uncovered by the first. A sponsorless shape is defined
    /// in that vocabulary as one that pays no fee at all, so a
    /// two-destination sponsorless request selects a shape of TWO RECEIPT
    /// OUTPUTS and the receipt covenant requires a receipt program at the
    /// fee's position. The target refuses the candidate at script
    /// verification, which is a target verdict on a covenant this
    /// workspace wrote. Argued at
    /// `(´[PLAN-rule:shapes:absent-fee-role]´)`.
    SponsorlessShapeHasNoFeeMember,
    /// A live transfer's representation is ONE variable for the whole
    /// transaction, so no transfer may cross.
    ///
    /// Not a registry rule and not a target rule: a rule of this
    /// workspace's own construction vocabulary, and the last of the
    /// three kinds this census keeps apart. The guide admitted mixed
    /// representation "unless separately admitted" from the beginning,
    /// and nothing was ever separately admitted, so the compiler carried
    /// one plan per transfer, the constructor was built for one plan,
    /// the coordinator's value obligation was dispatched on one plan,
    /// and the construction lane read one plan for both the receipts it
    /// recognized and the destinations it paid.
    ///
    /// It refused BOTH crossing shapes and it refused them before any
    /// arithmetic: not because a blinder failed to solve or a proof
    /// failed to build, but because no vocabulary existed in which the
    /// candidate could be stated. That is what makes it a limitation of
    /// this repository rather than an observation about consensus, which
    /// admits both directions and is upstream-tested doing so.
    HomogeneousRepresentationOnly,
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
            Self::AbsentFeeProjection => {
                "packages/vectors/src/live_proof_bearing_observation.rs, the fixture projection, \
                 whose role match had no arm for a fee output and refused by name rather than \
                 mapping one onto the balancing role"
            }
            Self::SponsorlessShapeHasNoFeeMember => {
                "packages/tapscript/src/live_shape.rs, whose `LiveTransferShape` counts receipt \
                 outputs and a sponsor region and nothing else, reached through \
                 packages/transaction/src/live_construct.rs `select_shape`, which matches a \
                 shape on `receipt_outputs() == destinations` and therefore reads a fee \
                 destination as a receipt output"
            }
            Self::HomogeneousRepresentationOnly => {
                "packages/compiler/src/live_transfer_plan.rs, whose                  `LiveTransferRepresentationPlan` is ONE value per transfer, read at every site                  that decides a form: packages/tapscript/src/live_pattern.rs, whose recognition                  fragment pins the spent value's form to that one plan and whose coordinator                  dispatches the destinations' obligation on it, and                  packages/transaction/src/live_construct.rs, which reads it for both the receipts                  it recognizes and the destinations it pays"
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
            Self::AbsentFeeProjection => {
                "The committed-output materializer. Every stage between the registry and a \
                 candidate was written for an output with a commitment: an independent \
                 recomputation to compare against, a nonce to derive, a range to prove, a witness \
                 entry to carry the proof. A fee output has none of them, so the role was \
                 withheld from the materializer's vocabulary rather than added ahead of the \
                 stages -- a member without the stages would have mapped a fee onto the \
                 committed path and produced a BLINDED fee output, which the target does not \
                 recognize as a fee at all."
            }
            Self::SponsorlessShapeHasNoFeeMember => {
                "Fees are the sponsor's job. The reviewed live-transfer shape vocabulary reads a \
                 sponsorless form as one that pays no fee at all, on the reviewed target's own \
                 representation of a zero fee by the ABSENCE of the output. So the vocabulary has \
                 no member for a sponsorless shape that pays its own fee, a fee destination is \
                 counted as a receipt output, and the receipt covenant constrains it as one. \
                 Nothing here decided against the shape; no decision was recorded because none \
                 was made."
            }
            Self::HomogeneousRepresentationOnly => {
                "One transfer, one representation. Guide §6.5 states the initial scope as                  homogeneous explicit REQUIRED and homogeneous private committed REQUIRED, with                  mixed 'unsupported unless separately admitted' -- so mixed was never forbidden                  and never built, and the vocabulary took the scope literally. The clause that                  kept it that way is the guide's next sentence rather than the scope: an ad hoc                  mixed transaction accepted by the target does not widen the ABI, so no run could                  ever have produced the admission and only a ruling could. None was made, so the                  single variable stood."
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
            Self::AbsentFeeProjection => RemovalPath::FeeRoleProjection,
            Self::SponsorlessShapeHasNoFeeMember => RemovalPath::SponsorlessFeeBearingShape,
            Self::HomogeneousRepresentationOnly => RemovalPath::PerSideRepresentationPlan,
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
            Self::CancelingPredecessorOnly => Some(CANCELING_PREDECESSOR_REMOVAL),
            Self::AbsentFeeProjection => Some(FEE_ROLE_PROJECTION_REMOVAL),
            Self::SponsorlessShapeHasNoFeeMember => Some(SPONSORLESS_FEE_REMOVAL),
            Self::HomogeneousRepresentationOnly => Some(PER_SIDE_REPRESENTATION_REMOVAL),
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
            Self::AbsentFeeRole
            | Self::CancelingPredecessorOnly
            | Self::AbsentFeeProjection
            | Self::SponsorlessShapeHasNoFeeMember
            | Self::HomogeneousRepresentationOnly => false,
        }
    }
}

/// A named structural removal for a limitation.
///
/// A path is a DESIGN this register commits to naming. Naming one says
/// nothing about whether it has been taken: [`Limitation::removal`] is
/// the only place that answers that, and it answers `None` by default.
///
/// Two of the paths below have since been taken and their descriptions
/// are left exactly as they were written, because a path's description is
/// what was proposed and the record of what was done belongs beside it
/// rather than on top of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemovalPath {
    /// A manifest form whose single output is fully solved.
    SingleOutputSolvedBalancingForm,
    /// A fee member of the fixture output role vocabulary.
    FeeOutputRole,
    /// A precursor transaction whose outputs do not cancel.
    NonCancelingPrecursor,
    /// A fee role carried from the registry to a candidate.
    FeeRoleProjection,
    /// A sponsorless shape that pays its own fee.
    SponsorlessFeeBearingShape,
    /// Pair an admitted representation plan to each SIDE of a transfer.
    ///
    /// The guide's own escape clause taken rather than a widening of the
    /// guide: §6.5 admits mixed representation "unless separately
    /// admitted", so what a crossing needs is an admission, and what an
    /// admission needs is a vocabulary in which the admitted thing can
    /// be said. The vocabulary is a PAIRING of the two plans §6.1
    /// already states exhaustively — one for the side a transfer
    /// consumes and one for the side it creates — which is why the plan
    /// census stays at two members and crossing introduces no third
    /// representation.
    ///
    /// A per-REFERENCE variable would be the other thing, and it stays
    /// deferred on a ground this path leaves literally true: the
    /// representation decision is one variable per object family, and a
    /// side is not a reference.
    PerSideRepresentationPlan,
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
            Self::FeeRoleProjection => {
                "Carry the fee role from the registry to a candidate. The materializer's own \
                 output-role vocabulary gains a fee member; its per-output stage gains a branch \
                 that emits an explicit value, an explicit asset, a null nonce, an empty program \
                 and an empty witness entry, and asks the built output whether it IS a fee by the \
                 target's own predicate rather than trusting that it built one; the projection's \
                 view carries the fee's ABSENT opening rather than a zero-filled one; and every \
                 role match states the fee arm instead of letting a catch-all solve a blinder for \
                 it."
            }
            Self::PerSideRepresentationPlan => {
                "Admit a PAIRING of the two representation plans, one per side of a transfer, \
                 rather than a third representation or a per-reference variable. §6.5 already \
                 wrote the escape clause -- mixed is unsupported UNLESS SEPARATELY ADMITTED -- so \
                 what the path needs is a ruling and a vocabulary, not a guide amendment, and the \
                 guide's next sentence is why no run could ever have supplied the admission: an \
                 ad hoc mixed transaction accepted by the target does not widen the ABI. The \
                 pairing threads to four decisions that each used to read the single variable: \
                 which value form the recognition fragment pins a spent receipt to, which \
                 obligation the coordinator emits over the destinations, which constructor a \
                 destination is paid to, and which key a deployment seats a constructor at. The \
                 exit direction additionally needs a POSITIONAL value-form leaf, because its \
                 created side is per-position heterogeneous -- explicit everywhere but the one \
                 declared absorber -- and no fragment that speaks about a whole range can say \
                 that."
            }
            Self::SponsorlessFeeBearingShape => {
                "Give the reviewed live-transfer shape vocabulary a sponsorless member that pays \
                 its own fee, so a fee destination is not counted as a receipt output and the \
                 receipt covenant does not demand a receipt program at the fee's position. The \
                 covenant already owns the discriminator it would need: the sponsored isolation \
                 fragment recognizes a fee output BY FORM, at a negative version marker against \
                 the digest of the empty program, and never by amount. What is missing is a shape \
                 that says a sponsorless form may carry one. This is a guide-level reading rather \
                 than a registry clause, so it stood filed until the owner's fee-matrix ruling at \
                 gate commit 0.5.7-dev made changing a reviewed reading a decision rather than an \
                 edit. It was then TAKEN, and the taking found more than the filing had \
                 anticipated. Three further readings decide WHERE the fee sits and had to move \
                 with the output count -- the family-range census, the isolation fragment's own \
                 emission, and the pattern census deciding whether that fragment is emitted at \
                 all, which had been a pure alias of the sponsor question. Selection had to count \
                 declared fee positions out of the destinations before matching. The fee clause \
                 had to read its asset off who funded the fee, a sponsorless form having no \
                 reserve-asset input while Elements balances per asset. The explicit conservation \
                 relation had to gain the fee as a term. And the deployment had to be welded to \
                 the digest an empty program actually hashes to, the fixture constant it carried \
                 hashing to no program at all."
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
            Self::FeeOutputRole
            | Self::NonCancelingPrecursor
            | Self::FeeRoleProjection
            | Self::SponsorlessFeeBearingShape => None,
            // The entry direction has one, and it is the SAME degeneracy
            // the single-output form carries, met from the other side.
            // An entry crossing consumes explicit coins, so its input
            // blinder sum is zero; a single blinded output would have to
            // declare the sole-balancing form, the solve would return
            // that zero unchanged, and the commitment would be exactly
            // `v*H` -- a blinded output's form with none of its hiding.
            // TWO blinded outputs avoid it, the balancing one solving to
            // the negation of a primary blinder that is searched and
            // refused if zero. So the floor is the registry's own
            // arithmetic rather than a preference, and Elements' wallet
            // refuses the same shape for the same reason.
            Self::PerSideRepresentationPlan => Some(
                "An entry crossing spends EXPLICIT coins, whose blinders are zero, so the \
                 input blinder sum a single blinded output would be forced to is zero and the \
                 commitment hides nothing. The removal must therefore require TWO OR MORE \
                 blinded outputs on entry, where the balancing blinder is the negation of a \
                 searched non-zero primary. This is the same degeneracy the single-output form \
                 carries, reached from the opposite side, and the registry's existing \
                 `DegenerateBalancingScalar` refusal is what catches it either way.",
            ),
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
        // The private merge, which met two walls and is past both. The row
        // cites the SECOND one, because that is the wall the shape was
        // standing at when this removal reached it; the first is cited by
        // the strict one-to-one's row and by the prose register, which
        // carries the arc neither row can.
        BlindedShape::TwoToOne => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::MERGE_ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAfterRemoval {
                removed: Limitation::CancelingPredecessorOnly,
                removal: CANCELING_PREDECESSOR_REMOVAL,
            },
        ),
        // The fee-bearing shape, which was BUILT and OFFERED and refused
        // three times and is now ACCEPTED. Its fee output really is a fee
        // -- the run's own output-witness census reads one range proof
        // and one EMPTY entry -- and every refusal on the way was a
        // covenant this workspace wrote rather than any rule about fees.
        BlindedShape::OneToOneWithFee => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::FEE_BEARING_SUCCESSOR_IDENTITY,
            },
            FirstPartyStatus::ConstructibleAfterRemoval {
                removed: Limitation::SponsorlessShapeHasNoFeeMember,
                removal: SPONSORLESS_FEE_REMOVAL,
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
        // BLINDING ON ENTRY, expressible and unrun. Consensus admits it
        // for a reason the other rows never needed: with no blinded
        // input the sum to absorb is already zero, so nothing has to
        // hold it and the two blinded outputs are this workspace's
        // convention rather than the target's rule. The floor of two is
        // the registry's own arithmetic -- a single blinded output would
        // have to declare the sole-balancing form, the solve would
        // return the zero input sum unchanged, and
        // `DegenerateBalancingScalar` would fire -- and Elements' own
        // wallet refuses the same thing for the same reason. Both are
        // conventions about confidentiality; neither is a protocol rule
        // and this row claims neither as one.
        // RUN. The shape this workspace has performed every ceremony as
        // a FUNDING step, performed for the first time as a
        // covenant-governed transfer -- the coin it spent sat at a
        // receipt constructor's program, which is the whole difference.
        BlindedShape::EntryCrossing => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::ENTRY_CROSSING_ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAfterRemoval {
                removed: Limitation::HomogeneousRepresentationOnly,
                removal: PER_SIDE_REPRESENTATION_REMOVAL,
            },
        ),
        // UNBLINDING ON EXIT, expressible and unrun. The absorber is
        // what makes it possible and it is an ordinary blinded
        // destination at a declared position, so the solve is the one
        // the registry already performs -- with nothing derived to
        // subtract it returns the input blinder sum ITSELF, which is
        // recomputed at the registry rather than predicted here.
        // RUN, and the row moved on the acceptance rather than on the
        // vocabulary. A real node took two blinded receipts into two
        // explicit destinations beside one blinded absorber, and the
        // proof census in the mined bytes is a vector no homogeneous
        // shape of this arity can produce: two entries empty and one
        // carrying the transaction's only range proof.
        BlindedShape::ExitCrossing => (
            ConsensusVerdict::ObservedAccepted {
                identity: crate::live_multi_shapes::run_of_record::EXIT_CROSSING_ACCEPTED_TXID,
            },
            FirstPartyStatus::ConstructibleAfterRemoval {
                removed: Limitation::HomogeneousRepresentationOnly,
                removal: PER_SIDE_REPRESENTATION_REMOVAL,
            },
        ),
        // THE CORNER THIS REGISTER EXISTS TO KEEP APART, and the one
        // shape here refused by BOTH sides for DIFFERENT reasons.
        // Consensus refuses it because a nonzero consumed blinder sum
        // has nowhere to land once the absorber is gone -- the same
        // arithmetic that makes the fee-only shape impossible. The
        // registry refuses it independently, and not on cardinality:
        // a manifest with no solving role has no output to solve, which
        // is a first-party rule about manifests rather than a reading of
        // the tally. Two grounds, one shape, and the register would be
        // worth less if it recorded either one alone.
        //
        // The escape is real and is the enabling condition for another
        // row rather than a way to take this one: a predecessor whose
        // consumed blinders CANCEL presents a zero sum, and over such a
        // set consensus admits full unblinding. That is a property of
        // the coins spent and not of the shape, so it does not move this
        // verdict. It is filed and never attempted.
        BlindedShape::FullyUnblinding => (
            ConsensusVerdict::SourceDerivedImpossible,
            FirstPartyStatus::RefusalGuardsConsensus {
                refusal: RegistrationRefusal::BalancingRoleNotUnique { found: 0 },
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
    use std::collections::BTreeSet;

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
                    //
                    // The EXPLICIT destinations of a crossing row come
                    // first and the blinded absorber last, which is the
                    // order the covenant's own positional leaf declares
                    // rather than a convenience: the absorber is the
                    // LAST destination, so a drive that put it anywhere
                    // else would be recomputing the refusal for a
                    // manifest no candidate would present.
                    role: if is_fee {
                        FixtureOutputRole::Fee
                    } else if index < shape.explicit_destinations() {
                        FixtureOutputRole::ExplicitDestination
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
        assert_eq!(BlindedShape::ALL.len(), 11);
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

    /// The three output axes partition the output set.
    ///
    /// The guard that makes a STATED blinded-output count safe. While
    /// the count was derived it could not disagree with the output
    /// count; stated, it can, and a member whose axes did not add up
    /// would compute a wrong consensus verdict rather than fail. This
    /// is what makes that impossible.
    #[test]
    fn every_shape_partitions_its_outputs_into_blinded_explicit_and_fee() {
        for shape in BlindedShape::ALL {
            assert_eq!(
                shape.blinded_outputs() + shape.explicit_destinations() + shape.fee_outputs(),
                shape.outputs(),
                "{} states axes that do not add up to its own output count",
                shape.handle(),
            );
            assert!(
                shape.fee_outputs() <= 1,
                "{} carries more than one fee output",
                shape.handle(),
            );
        }

        // And the axis earns its keep: exactly the two exit-side shapes
        // carry an explicit NON-fee output, which is the premise the
        // derivation used to assume away for every member.
        let explicit: Vec<&str> = BlindedShape::ALL
            .iter()
            .filter(|shape| shape.explicit_destinations() > 0)
            .map(|shape| shape.handle())
            .collect();
        assert_eq!(
            explicit,
            vec!["exit-crossing", "fully-unblinding"],
            "only the exit side creates explicit receipt destinations",
        );
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

    /// The impossible shapes are exactly those that consume a blinded
    /// input and create no blinded output.
    ///
    /// The register's sharpest claim, and it is now stated as the
    /// PROPERTY rather than as a member. It used to name the fee-only
    /// shape, and while that was the only member with no blinded output
    /// the two readings were the same sentence. The crossing wave added
    /// a second such member -- full unblinding -- and naming a member
    /// would have made this test a list to be updated rather than a
    /// claim to be checked.
    ///
    /// The property is the whole of the tally: a consumed blinder sum
    /// must land somewhere, explicit outputs contribute zero, so a shape
    /// that consumes a nonzero sum and blinds nothing has nowhere to put
    /// it. Both impossible members fail on exactly that, and they differ
    /// only in WHAT the outputs are -- a fee in one case, explicit
    /// receipt destinations in the other -- which is a difference the
    /// tally does not see.
    #[test]
    fn the_impossible_shapes_are_those_that_blind_an_input_and_no_output() {
        let impossible: Vec<BlindedShape> = BlindedShape::ALL
            .into_iter()
            .filter(|shape| {
                matches!(
                    census_entry(*shape).consensus,
                    ConsensusVerdict::SourceDerivedImpossible
                )
            })
            .collect();
        assert_eq!(
            impossible,
            vec![BlindedShape::FeeOnly, BlindedShape::FullyUnblinding],
        );

        for shape in impossible {
            assert_eq!(
                shape.blinded_outputs(),
                0,
                "{} is impossible because it blinds no output",
                shape.handle(),
            );
            assert!(
                shape.blinded_inputs() > 0,
                "{} is impossible because it has a sum to place at all",
                shape.handle(),
            );
        }

        // The two differ in what their outputs ARE, which is exactly
        // what the tally does not see -- and it is why the two rows
        // record DIFFERENT first-party refusals for the same consensus
        // verdict.
        assert_eq!(BlindedShape::FeeOnly.fee_outputs(), 1);
        assert_eq!(BlindedShape::FeeOnly.explicit_destinations(), 0);
        assert_eq!(BlindedShape::FullyUnblinding.fee_outputs(), 0);
        assert_eq!(BlindedShape::FullyUnblinding.explicit_destinations(), 2);
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
                | FirstPartyStatus::ExpressibleAndUnrun { .. }
                // A shape a TARGET refused carries no registry refusal to
                // recompute, and this test is about the registry. Its
                // refusal is a target's verdict, recorded verbatim on the
                // row and recomputed by nothing, because recomputing it
                // would mean asking the target again.
                | FirstPartyStatus::SubmittedAndRefused { .. } => continue,
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
            "two of the eleven shapes are refused by the registry, and both are the impossible \
             ones. They are refused for DIFFERENT reasons and that is the point: the fee-only \
             shape dies on cardinality, having one output, and the fully-unblinding shape has \
             three and dies because none of them SOLVES. The merge used to be a third and is now \
             accepted",
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
                BlindedShape::OneToOneWithFee,
                crate::live_multi_shapes::run_of_record::FEE_BEARING_SUCCESSOR_IDENTITY,
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
            (
                BlindedShape::TwoToOne,
                crate::live_multi_shapes::run_of_record::MERGE_ACCEPTED_TXID,
            ),
            (
                BlindedShape::ExitCrossing,
                crate::live_multi_shapes::run_of_record::EXIT_CROSSING_ACCEPTED_TXID,
            ),
            (
                BlindedShape::EntryCrossing,
                crate::live_multi_shapes::run_of_record::ENTRY_CROSSING_ACCEPTED_TXID,
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
        assert_eq!(
            expected.len(),
            9,
            "nine of the eleven shapes have been run: the seven homogeneous ones and BOTH \
             crossing directions. The two that have not are the two the tally forbids",
        );

        // The converse, which this test used to leave unchecked. The list
        // above says every shape in it is observed; without this, a shape
        // that BECAME observed and was never added to the list would pass
        // unnoticed, and a register whose observed set can grow quietly is
        // the one thing this module exists to prevent.
        let observed: Vec<BlindedShape> = BlindedShape::ALL
            .into_iter()
            .filter(|shape| {
                matches!(
                    census_entry(*shape).consensus,
                    ConsensusVerdict::ObservedAccepted { .. }
                )
            })
            .collect();
        let mut listed: Vec<BlindedShape> = expected.into_iter().map(|(shape, _)| shape).collect();
        listed.sort_unstable_by_key(|shape| shape.handle());
        let mut observed = observed;
        observed.sort_unstable_by_key(|shape| shape.handle());
        assert_eq!(
            observed, listed,
            "exactly the listed shapes are observed, and no others",
        );
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

        // The fee-bearing shape is in this list even though it was never
        // accepted, because what the list checks is that the register
        // describes the same CARDINALITIES the ceremony ran — a question
        // a refusal answers exactly as well as an acceptance does.
        let ordered = [
            BlindedShape::OneToThree,
            BlindedShape::TwoToThree,
            BlindedShape::TwoToTwo,
            BlindedShape::OneToOne,
            BlindedShape::OneToOneWithFee,
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
        assert!(
            paths.is_empty(),
            "no REGISTRY convention refuses a consensus-possible shape any more: {paths:?}",
        );

        // The discipline does not end with the registry, and this half is
        // what keeps the emptiness above from reading as completion. A
        // shape a target refused still owes a named path, and it owes one
        // for the same reason -- a refusal recorded without a way out
        // gets defended later as though it were consensus.
        let mut submitted = Vec::new();
        for shape in BlindedShape::ALL {
            let FirstPartyStatus::SubmittedAndRefused { limitation, .. } =
                census_entry(shape).first_party
            else {
                continue;
            };
            assert_ne!(limitation.refused_at(), "", "the refusing row is named");
            assert_ne!(limitation.convention(), "", "the convention is explained");
            assert_ne!(
                limitation.removal_path().description(),
                "",
                "the removal path is described",
            );
            assert_eq!(
                limitation.removal(),
                None,
                "{} still stands, so it may not claim a removal",
                shape.handle(),
            );
            submitted.push(limitation.removal_path());
        }
        // EMPTY, and the emptiness is the wave's result rather than a
        // loosening. The one shape that sat here was offered to a node,
        // refused, given the vocabulary member it was missing, and
        // accepted -- so it moved to a constructible status and took its
        // filed path with it as a TAKEN one. The loop above is kept
        // because the status is still reachable and still documented, and
        // the next shape to be refused by a target will be held to
        // exactly these four conditions.
        assert!(
            submitted.is_empty(),
            "no shape stands offered-and-refused any more: {submitted:?}",
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
        // The ROWS, in order, and the crossing limitation appears TWICE
        // because two rows cite it. That repetition is a fact rather
        // than a duplicate to collapse: one removal freed two shapes and
        // BOTH of them ran, which had not happened before -- every
        // earlier removal freed at most one.
        assert_eq!(
            removed,
            vec![
                Limitation::TwoOutputFloor,
                Limitation::SponsorlessShapeHasNoFeeMember,
                Limitation::CancelingPredecessorOnly,
                Limitation::HomogeneousRepresentationOnly,
                Limitation::HomogeneousRepresentationOnly,
            ],
            "five observed rows cite a removal, over FOUR distinct limitations: the floor, the \
             shape vocabulary with no sponsorless fee-bearing member, the canceling \
             predecessor, and the one representation per transfer -- which two rows cite \
             because it freed both crossing directions",
        );
        let distinct: BTreeSet<Limitation> = removed.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            4,
            "four distinct limitations are removed and run"
        );

        // Every DISTINCT removal is proven by its own identity, which is
        // what stops one acceptance being cited for work it did not do.
        // Counted over the distinct limitations rather than the rows,
        // because two rows citing one removal share its proof by
        // construction and that is not a collision.
        let identities: BTreeSet<&str> = distinct
            .iter()
            .filter_map(|limitation| limitation.removal())
            .filter_map(|removal| removal.proven_by)
            .collect();
        assert_eq!(
            identities.len(),
            distinct.len(),
            "each removal is proven by its own run, not by a shared one",
        );

        // And the shared removal's proof is ONE of the two acceptances
        // rather than both or neither. The register records which shape
        // carried a removal to a chain, and a removal freeing two shapes
        // does not thereby acquire two proofs.
        assert_eq!(
            Limitation::HomogeneousRepresentationOnly
                .removal()
                .and_then(|removal| removal.proven_by),
            Some(crate::live_multi_shapes::run_of_record::EXIT_CROSSING_ACCEPTED_TXID),
            "the crossing removal names the first shape that carried it to a chain",
        );
    }

    /// A removal that nothing has run says so, and says where a run
    /// stops.
    ///
    /// The register's sharpest discipline, applied to its own work. A
    /// vocabulary that CAN express a shape is not a chain that HAS
    /// accepted one, and the whole reason this register exists is that
    /// those two had been collapsing into one word.
    ///
    /// So the row carries no identity, its removal carries no identity,
    /// and the place a run stops is named in the stopping layer's own
    /// terms rather than left as "not yet".
    ///
    /// # The status is occupied again, and by two
    ///
    /// It stood EMPTY between the fee-bearing shape's acceptance and the
    /// crossing wave, and the emptiness was a result rather than a
    /// loosening. Both crossing directions now sit here: their
    /// limitation is really removed -- a composition pairs a plan to
    /// each side, the covenant dispatches on it, the registry has an
    /// explicit destination role and the materializer builds one -- and
    /// NO node has been offered either shape. That is precisely the
    /// distinction this status was minted to carry, and a wave that
    /// recorded its own unrun vocabulary as an observation would be the
    /// failure the register was built to prevent.
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

            // NOT `removal.proven_by == None`, and the change is a
            // reading rather than a loosening. That assertion encoded an
            // assumption that has now been refuted by running: that a
            // removal frees exactly one shape. The crossing removal
            // freed TWO, the exit direction ran and the entry direction
            // did not, and `proven_by` names -- by its own field doc --
            // the first shape a removal unlocked, not every shape it
            // could.
            //
            // What the discipline actually forbids is a ROW claiming an
            // acceptance it does not have, and that is asserted below
            // against this shape's own consensus verdict. A removal's
            // proof and a row's evidence are different facts, and this
            // is the register whose reason for existing is not
            // collapsing facts of different kinds into one word.
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
        // OCCUPIED, and by exactly the two the crossing wave added. The
        // list is pinned rather than counted, so a third shape arriving
        // here -- or one of these two leaving without its acceptance
        // being recorded -- is a visible test change and not a number
        // that quietly moved.
        // EMPTY again, and the emptiness is a result rather than a
        // loosening. Both crossings sat here when the vocabulary landed
        // and both then left by the only honest exit, an acceptance of
        // their own shape. The status is kept for the reason the others
        // are kept: a vocabulary member that nothing currently reaches
        // is not thereby wrong, and the next removal nobody has run must
        // be able to say so. What is checked above is that anything
        // sitting here would still owe an unproven acceptance and a
        // named stopping layer.
        assert!(
            expressible.is_empty(),
            "no shape is expressible-and-unrun any more: {expressible:?}",
        );

        // The fee-bearing shape used to be the sole member here, and it
        // has now left this status too -- by being ACCEPTED, which is the
        // second and last way out. The sixth status was minted to carry
        // the distinction between having been offered and having been
        // taken, and the shape has now been both in turn.
        //
        // The status is UNOCCUPIED and is kept, on the same reasoning
        // that keeps ConstructibleAfterRemoval alive: a vocabulary member
        // that can no longer be reached is not thereby wrong, and the
        // next shape a target refuses must be able to say so. What is
        // checked here is therefore that the status remains COHERENT --
        // anything sitting in it would still owe an unproven removal, a
        // carried verdict and an unmoved consensus half -- rather than
        // that something currently sits in it.
        let mut submitted = Vec::new();
        for shape in BlindedShape::ALL {
            let FirstPartyStatus::SubmittedAndRefused {
                removed,
                removal,
                observed_detail,
                ..
            } = census_entry(shape).first_party
            else {
                continue;
            };
            assert_eq!(
                removal.proven_by,
                None,
                "{} was refused, so its removal proves nothing about a chain",
                shape.handle(),
            );
            assert_eq!(removed.removal(), Some(removal));
            assert_ne!(observed_detail, "", "the target's own verdict is carried");
            assert_eq!(
                census_entry(shape).consensus,
                ConsensusVerdict::SourceDerivedPossible,
                "{} was refused by a first-party covenant, so consensus is still a derivation",
                shape.handle(),
            );
            submitted.push(shape);
        }
        assert!(
            submitted.is_empty(),
            "no shape stands offered-and-refused any more: {submitted:?}",
        );
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
