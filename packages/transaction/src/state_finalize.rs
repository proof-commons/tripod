//! Finalization of the maturity announcement (§12.7's second state), and
//! the operator signing request it admits (§13.1, §13.2).
//!
//! # What finalization fixes, and what it leaves alone
//!
//! A construction is a candidate that exists; a finalization is that
//! candidate with its protected regions settled and its exact bytes
//! taken once. [`FinalizedMaturityAnnouncement`] has private fields, no
//! setter, no accessor handing out a mutable reference and no
//! constructor outside this module, so the states §12.7 forbids a
//! builder from re-entering are unrepresentable rather than refused.
//!
//! What it does not do is larger than what it does. No signature exists
//! here and none is computed: what the operator is handed is a request
//! over the exact candidate, and what comes back is the next state's
//! question. Nothing is submitted, nothing is relayed, and no verdict
//! about either is stated. The chain stops at this state: signing
//! started, full authorization and submission readiness are types this
//! module does not name, and the request built here is what the state
//! after it signs.
//!
//! # The nine terms, and where each is read from
//!
//! §13.2 requires nine terms to derive from one finalized candidate, so
//! that a caller cannot substitute a committed leaf of the same tree.
//! [`FinalizedMaturityAnnouncement::signing_request`] reads all nine off
//! this one value:
//!
//! - candidate structure: the protected transaction, handed to the
//!   freeze whole rather than rebuilt from parts;
//! - protected bytes: this value's own, computed once at finalization;
//! - spent-output census: the one spent-output record's three fields,
//!   turned into the census entry §13.1 binds;
//! - input index: that record's position;
//! - leaf hash, leaf version and control block: the predecessor
//!   constructor's control recipe for the announcement role, which is
//!   the executing leaf data §12.7 protects;
//! - code-separator position: the operator profile's, applied by the
//!   freeze rather than offered to it;
//! - deployment seed: the operator binding's own genesis identity, in
//!   the byte order the message uses.
//!
//! There is no parameter, no argument and no setter through which a
//! leaf, a byte string or a spent output could arrive from anywhere
//! else, which is a stronger statement than a check that such a thing
//! was not offered.
//!
//! # Why the executing leaf comes from the predecessor's tree
//!
//! The spend this candidate performs consumes the predecessor's STATE
//! output, so the leaf it executes is a leaf of the *predecessor's*
//! committed tree and the control block authenticates that tree's root
//! under the policy's internal key. The successor's constructor commits
//! the output this transaction creates and says nothing about what
//! authorizes the input; taking either from the successor would produce
//! a control block that authenticates a tree no spent output pays to.
//!
//! Within that tree, the leaf script is the static subtree entry's own
//! committed program. A link also retains a relocated program per leaf,
//! and the two need not be the same byte string; the tapleaf hash the
//! control block authenticates was taken over the committed one, so the
//! committed one is what a spend supplies.
//!
//! # The offered comparison is exact, and it is over the protected
//! encoding
//!
//! Structural impossibility inside this crate is not a claim that
//! nothing anywhere can mutate the bytes, so
//! [`FinalizedMaturityAnnouncement::check_offered`] looks. It names the
//! region that moved where it can — the input census, an output
//! position, the version, the lock time — and closes over the exact
//! protected encoding, which catches what those four comparisons do not
//! reach. In this form that is the input's sequence field: inputs are
//! compared by outpoint, and a sequence can move without the census
//! changing size.
//!
//! The comparison is over the witnessless encoding rather than the full
//! one. For this candidate the two are the same byte string — its one
//! input witness is null and its one output witness empty, so the
//! encoder writes no witness section at all, which is why the finalized
//! bytes are simultaneously what a signer echoes and what the strict
//! decoder round-trips. They stop being the same string as soon as the
//! next state inserts the operator witness, and a check that refused
//! the candidate it had just authorized would be refusing the one thing
//! §12.7 admits moving.

use std::collections::BTreeSet;

use tapscript::{CandidateStateConstructor, StateLeafRole};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::bytes::{AssetField, Outpoint, TargetOutput, TargetTransaction, ValueField};
use crate::error::TransactionRefusal;
use crate::live_taproot::LiveCurveCapability;
use crate::operator_signing::{OperatorSigningInput, OperatorSigningRequest};
use crate::script_path_signing::{LiveDeployment, SpentOutputCensusEntry};
use crate::state_construct::MaturityConstruction;
use crate::taproot::Digest32;

/// Where the predecessor STATE output is spent.
///
/// The ABI puts the coordinator at input zero, and the sponsorless form
/// has exactly one input, so there is no other position for it to
/// occupy. Named rather than written twice, because the spent-output
/// census and the signing request have to agree about it.
const STATE_INPUT_POSITION: u16 = 0;

/// Where the successor STATE output is created.
///
/// Output zero, for the input position's reason: the ABI pins it and
/// the form has one output.
const SUCCESSOR_OUTPUT_POSITION: u16 = 0;

/// One region §12.7 protects, as this finalization settles it.
///
/// A census rather than prose, so that a finalized value can say what
/// it fixed and a reader can check the list against the guide instead
/// of against a struct.
///
/// Eleven members for twelve regions. The twelfth is the executing leaf
/// data, and nothing in the transaction bytes settles it: the candidate
/// carries one null input witness, and the leaf a spend executes, its
/// version, its hash and its control block are facts about the
/// predecessor's committed tree rather than fields of this transaction.
/// They are fixed by the signing request, which derives them from that
/// tree — so they are readable here through
/// [`FinalizedMaturityAnnouncement::executing_leaf`] and are not
/// something these bytes could be said to have settled.
///
/// This census is not the one a post-signing mutation refusal names its
/// region by. That census is twelve members and belongs to the states
/// after this one; this is what a finalized value reports it fixed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaturityFinalizedFact {
    /// Every input is fixed.
    Inputs,
    /// The spent-output census is fixed.
    SpentOutputCensus,
    /// Every output is fixed.
    Outputs,
    /// The successor's semantic metadata is fixed.
    SuccessorMetadata,
    /// The successor's canonical representation nonce is fixed.
    SuccessorRepresentationNonce,
    /// The successor's output program is fixed.
    SuccessorProgram,
    /// Every output witness is fixed.
    OutputWitnesses,
    /// The transaction version is fixed.
    Version,
    /// The lock time is fixed.
    LockTime,
    /// The sponsor region is fixed as absent.
    SponsorRegionAbsent,
    /// The fee role is fixed as absent.
    FeeRoleAbsent,
}

impl MaturityFinalizedFact {
    /// The complete census, in §12.7's own order.
    pub const ALL: &'static [Self] = &[
        Self::Inputs,
        Self::SpentOutputCensus,
        Self::Outputs,
        Self::SuccessorMetadata,
        Self::SuccessorRepresentationNonce,
        Self::SuccessorProgram,
        Self::OutputWitnesses,
        Self::Version,
        Self::LockTime,
        Self::SponsorRegionAbsent,
        Self::FeeRoleAbsent,
    ];
}

/// The output census the finalized form fixes.
///
/// Held beside the transaction rather than read back out of it, because
/// §12.7's post-boundary check compares an *offered* transaction with
/// what was finalized, and a comparison reading both sides from the
/// same value could not fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityFinalizedOutputCensus {
    outputs: Vec<TargetOutput>,
    successor_position: u16,
    sponsor_change_position: Option<u16>,
    fee_position: Option<u16>,
}

impl MaturityFinalizedOutputCensus {
    /// Every fixed output, in position order.
    #[must_use]
    pub fn outputs(&self) -> &[TargetOutput] {
        &self.outputs
    }

    /// Where the successor STATE output sits.
    #[must_use]
    pub const fn successor_position(&self) -> u16 {
        self.successor_position
    }

    /// Where the sponsor-change role sits, when the form has one.
    ///
    /// Always absent for the form this module finalizes, and absent
    /// because the form has no sponsor region at all rather than
    /// because a caller declined one: the reduced announcement leaf
    /// carries no sponsor check, and the realization's sponsor
    /// relations hold no region of the transaction until a later refit
    /// gives them one.
    #[must_use]
    pub const fn sponsor_change_position(&self) -> Option<u16> {
        self.sponsor_change_position
    }

    /// Where the target fee role sits, when the form has one.
    ///
    /// Always absent here, and consensus-valid without one: the spent
    /// output and the successor carry the same asset and the same
    /// amount, so the per-asset balance has no difference for a fee to
    /// make up.
    #[must_use]
    pub const fn fee_position(&self) -> Option<u16> {
        self.fee_position
    }
}

/// The spent predecessor output, as the finalized form fixes it.
///
/// §13.1 binds exact spent-output fields, and the target's operator
/// message is taken over the spent asset field, the spent value field
/// and the spent program together. The record holds all three rather
/// than one of them, because a census assembled beside the form from
/// values fetched separately is exactly the route the finalized form
/// exists to prevent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturitySpentOutputRecord {
    position: u16,
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
}

impl MaturitySpentOutputRecord {
    /// Which input position spends the predecessor.
    #[must_use]
    pub const fn position(&self) -> u16 {
        self.position
    }

    /// The outpoint the predecessor STATE output is spent from.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }

    /// The spent output's asset field, as the public view stated it.
    #[must_use]
    pub const fn asset(&self) -> AssetField {
        self.asset
    }

    /// The spent output's value field, as the public view stated it.
    #[must_use]
    pub const fn value(&self) -> ValueField {
        self.value
    }

    /// The spent output's program, as the public view stated it.
    ///
    /// The taproot witness program the predecessor sits behind, which
    /// carries the tweaked output key. The leaf a spend executes is a
    /// different string entirely and is reached through
    /// [`FinalizedMaturityAnnouncement::executing_leaf`].
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }

    /// The census entry §13.1 binds, over these same three fields.
    ///
    /// Derived rather than stored beside them, so the entry the signing
    /// request carries and the record a reviewer reads cannot come to
    /// differ.
    #[must_use]
    pub fn census_entry(&self) -> SpentOutputCensusEntry {
        SpentOutputCensusEntry::new(self.asset, self.value, self.program.clone())
    }
}

/// The executing leaf data §12.7 protects and the bytes do not carry.
///
/// Derived from the predecessor's committed tree, never supplied. It is
/// its own type rather than four returns because the four travel
/// together into one signing input, and because §13.2 asks for the
/// control block to be checkable against the candidate it was built
/// from — which needs it to be readable on this side at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityExecutingLeaf {
    tapleaf_hash: Digest32,
    leaf_version: LeafVersion,
    leaf_script: Vec<u8>,
    control_block: Vec<u8>,
}

impl MaturityExecutingLeaf {
    /// The executing leaf's tapleaf hash.
    #[must_use]
    pub const fn tapleaf_hash(&self) -> &Digest32 {
        &self.tapleaf_hash
    }

    /// The leaf version the committed tree hashed that leaf under.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The exact committed program the spend executes.
    #[must_use]
    pub fn leaf_script(&self) -> &[u8] {
        &self.leaf_script
    }

    /// The control block authenticating that leaf.
    ///
    /// The version-and-parity byte, the internal key, then the path
    /// from the leaf upward: the siblings inside the static subtree
    /// first, and the predecessor's metadata leaf hash outermost,
    /// because the committed root is the branch of the metadata leaf
    /// with the static subtree's root.
    #[must_use]
    pub fn control_block(&self) -> &[u8] {
        &self.control_block
    }
}

/// One maturity announcement with §12.7's regions settled.
///
/// A caller cannot substitute the executing leaf:
///
/// ```compile_fail,E0599
/// use transaction::FinalizedMaturityAnnouncement;
/// fn choose_leaf(finalized: &FinalizedMaturityAnnouncement) {
///     finalized.with_leaf_script(vec![0x51]);
/// }
/// ```
///
/// cannot substitute the protected bytes:
///
/// ```compile_fail,E0599
/// use transaction::FinalizedMaturityAnnouncement;
/// fn choose_bytes(finalized: &FinalizedMaturityAnnouncement) {
///     finalized.with_protected_bytes(vec![0x00]);
/// }
/// ```
///
/// and cannot substitute the spent output:
///
/// ```compile_fail,E0599
/// use transaction::FinalizedMaturityAnnouncement;
/// fn choose_spent(finalized: &FinalizedMaturityAnnouncement) {
///     finalized.with_spent_program(vec![0x51, 0x20]);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalizedMaturityAnnouncement {
    construction: MaturityConstruction,
    protected_bytes: Vec<u8>,
    outputs: MaturityFinalizedOutputCensus,
    spent: MaturitySpentOutputRecord,
}

impl FinalizedMaturityAnnouncement {
    /// The construction this finalizes, whole.
    ///
    /// The successor constructor, the successor metadata, the typed
    /// request, the validated public view and the candidate ABI are all
    /// reached through it rather than copied out beside it, so no
    /// second statement of any of them exists to drift from the one the
    /// construction settled.
    #[must_use]
    pub const fn construction(&self) -> &MaturityConstruction {
        &self.construction
    }

    /// The settled transaction.
    ///
    /// Read through the construction rather than held again: this value
    /// owns the construction, and a second copy of one transaction
    /// inside one value would be a second place for it to be stated.
    #[must_use]
    pub const fn protected(&self) -> &TargetTransaction {
        self.construction.transaction()
    }

    /// The exact protected serialization the operator signature commits
    /// to.
    ///
    /// Taken once, here, and compared rather than recomputed wherever
    /// it is read back. For this candidate it is also the full
    /// encoding: the one input witness is null and the one output
    /// witness empty, so the encoder writes no witness section, which
    /// is why these same bytes are what the strict decoder round-trips.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        &self.protected_bytes
    }

    /// The fixed output census.
    #[must_use]
    pub const fn outputs(&self) -> &MaturityFinalizedOutputCensus {
        &self.outputs
    }

    /// The fixed spent-output record for the one STATE input.
    #[must_use]
    pub const fn spent_output(&self) -> &MaturitySpentOutputRecord {
        &self.spent
    }

    /// Which of §12.7's regions this form settled.
    ///
    /// All eleven, for every finalized form: a value of this type that
    /// settled ten would be the thing §12.7 forbids, so the method
    /// reports the census rather than a subset of it. The twelfth
    /// region is the executing leaf data, which these bytes do not
    /// carry and the signing request binds.
    #[must_use]
    pub fn settled(&self) -> BTreeSet<MaturityFinalizedFact> {
        MaturityFinalizedFact::ALL.iter().copied().collect()
    }

    /// The executing leaf data, derived from the predecessor's
    /// committed tree.
    ///
    /// # Panics
    ///
    /// Panics only in three cases a linked bundle cannot arrange. The
    /// first is a bundle retaining no constructor application, which a
    /// link cannot produce because it retains the application it ran
    /// over its own sources. The second is a control recipe the
    /// predecessor constructor refuses: an absent announcement role is
    /// impossible because a static subtree without one is refused when
    /// the subtree is validated, a repeated role is impossible because
    /// the constructor's derivation refuses one before any commitment
    /// is computed, and an overdeep control path is impossible because
    /// that same derivation refuses an internal path of a hundred and
    /// twenty-eight or more, leaving the outer metadata sibling to
    /// bring a path of at most a hundred and twenty-seven to the depth
    /// the recipe admits. The third is a committed leaf hash the
    /// subtree that produced it does not carry, which is a statement
    /// about a value read out of that subtree moments earlier.
    #[must_use]
    pub fn executing_leaf(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> MaturityExecutingLeaf {
        let predecessor = self.predecessor();
        let recipe = predecessor
            .control_recipe(StateLeafRole::Announcement)
            .expect("a derived constructor carries a control recipe for the announcement role");
        let control_block = recipe
            .control_bytes()
            .expect("a control recipe that was derived derives its bytes again");
        let leaf_script = predecessor
            .static_subtree()
            .leaves()
            .iter()
            .find(|entry| entry.hash == recipe.executing_leaf_hash)
            .expect("the subtree carries the leaf its own recipe named")
            .leaf
            .program
            .encode(target);

        MaturityExecutingLeaf {
            tapleaf_hash: recipe.executing_leaf_hash,
            leaf_version: recipe.leaf_version,
            leaf_script,
            control_block,
        }
    }

    /// The operator signing request over this one finalized candidate.
    ///
    /// Every one of §13.2's nine terms is read off this value, so a
    /// caller substituting another committed leaf of the same tree, a
    /// different byte string or a different spent output has nothing to
    /// substitute it through.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MaturityAnnouncementSigningRequestRefused`]
    /// carrying whatever the freeze refused, whole: the binding's
    /// revision, the deployment genesis, the curve's verdict on the
    /// committed operator key, the census clauses and the leaf
    /// commitment are distinct findings, and the layer that owns the
    /// freeze has the words for all of them.
    ///
    /// # Panics
    ///
    /// Everything [`Self::executing_leaf`] panics on, for its reasons.
    pub fn signing_request(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
        curve: &dyn LiveCurveCapability,
    ) -> Result<OperatorSigningRequest<'_>, TransactionRefusal> {
        let leaf = self.executing_leaf(target);
        let binding = self
            .construction
            .validated_view()
            .view()
            .accepted_linked_bundle()
            .deployment()
            .operator();

        // The binding retains the printed identity and the message uses
        // internal order, so the seed is the binding's genesis reversed.
        // Read from the binding the request is frozen against rather
        // than from the bundle's own identity statement of it, so the
        // request and the commitment it is checked against cannot
        // disagree by construction.
        let mut genesis = *binding.deployment().genesis_id();
        genesis.reverse();

        OperatorSigningRequest::freeze(
            target,
            binding,
            self.protected().clone(),
            vec![self.spent.census_entry()],
            LiveDeployment::new(genesis),
            OperatorSigningInput::new(
                u32::from(self.spent.position()),
                leaf.tapleaf_hash,
                leaf.leaf_version,
                leaf.leaf_script,
                leaf.control_block,
            ),
            curve,
        )
        .map_err(
            |refusal| TransactionRefusal::MaturityAnnouncementSigningRequestRefused {
                refusal: Box::new(refusal),
            },
        )
    }

    /// Whether an offered transaction is the one that was finalized
    /// (§12.7).
    ///
    /// Each rejection is named for the region that moved rather than
    /// for the comparison that caught it, and the exact-byte close
    /// carries what no earlier comparison reaches.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::InputExtendedAfterSigning`] for an
    /// offering whose input census differs in size or in any outpoint;
    /// [`TransactionRefusal::OutputOmittedAfterSigning`] for an
    /// offering with fewer outputs;
    /// [`TransactionRefusal::OutputMutatedAfterSigning`] for an
    /// offering with more outputs or with an output that is not the one
    /// fixed at its position;
    /// [`TransactionRefusal::MaturityVersionChangedAfterFinalization`]
    /// and
    /// [`TransactionRefusal::MaturityLockTimeChangedAfterFinalization`]
    /// for the two scalar regions, which §12.7 censuses as regions of
    /// their own and which a report naming an output position would
    /// therefore misname; and
    /// [`TransactionRefusal::MaturityBytesDifferAfterFinalization`]
    /// when the protected encodings differ anywhere the four
    /// comparisons above do not reach, which in this form is the
    /// input's sequence field.
    ///
    /// The omission arm is unreachable for the form this module
    /// finalizes and is kept anyway: the candidate has one output and
    /// the target admits no transaction with none, so an offering
    /// missing that output cannot be built at all. The check states a
    /// law about the census rather than about its size, and an arm
    /// dropped because today's census has one member would have to be
    /// rediscovered by whoever gives the form a second.
    pub fn check_offered(&self, offered: &TargetTransaction) -> Result<(), TransactionRefusal> {
        let protected = self.protected();
        let finalized_inputs = protected.inputs();
        if offered.inputs().len() != finalized_inputs.len()
            || offered
                .inputs()
                .iter()
                .zip(finalized_inputs)
                .any(|(offered, fixed)| offered.outpoint() != fixed.outpoint())
        {
            return Err(TransactionRefusal::InputExtendedAfterSigning {
                finalized: finalized_inputs.len(),
                offered: offered.inputs().len(),
            });
        }

        let fixed = self.outputs.outputs();
        if offered.outputs().len() < fixed.len() {
            return Err(TransactionRefusal::OutputOmittedAfterSigning {
                position: position_of(offered.outputs().len()),
            });
        }
        if offered.outputs().len() > fixed.len() {
            return Err(TransactionRefusal::OutputMutatedAfterSigning {
                position: position_of(fixed.len()),
            });
        }
        for (index, (offered, fixed)) in offered.outputs().iter().zip(fixed).enumerate() {
            if offered != fixed {
                return Err(TransactionRefusal::OutputMutatedAfterSigning {
                    position: position_of(index),
                });
            }
        }

        if offered.version() != protected.version() {
            return Err(
                TransactionRefusal::MaturityVersionChangedAfterFinalization {
                    finalized: protected.version(),
                    offered: offered.version(),
                },
            );
        }
        if offered.lock_time() != protected.lock_time() {
            return Err(
                TransactionRefusal::MaturityLockTimeChangedAfterFinalization {
                    finalized: protected.lock_time(),
                    offered: offered.lock_time(),
                },
            );
        }

        let encoding = offered.encode_without_witness();
        if encoding == self.protected_bytes {
            Ok(())
        } else {
            Err(TransactionRefusal::MaturityBytesDifferAfterFinalization {
                at: first_difference(&self.protected_bytes, &encoding),
            })
        }
    }

    /// The predecessor constructor the link itself retained.
    fn predecessor(&self) -> &CandidateStateConstructor {
        self.construction
            .validated_view()
            .view()
            .accepted_linked_bundle()
            .instances()
            .first()
            .expect("a linked bundle retains the constructor application its own link ran")
            .constructor()
    }
}

/// Finalize one constructed maturity announcement (§12.7's second
/// state).
///
/// Infallible, and it is not a lost refusal. Every value it settles is
/// a read off something the construction already fixed: the outputs and
/// the bytes are the candidate's, and the spent-output record's four
/// fields are the validated view's own statements about the
/// predecessor. What can still refuse is the signing request, which
/// refuses where the freeze does, and that belongs to the value this
/// returns rather than to its creation.
///
/// A free function rather than a method on the construction, following
/// the live generation's own finalization: the state a construction
/// moves into is named by the module that owns that state, so the two
/// states stay readable apart.
#[must_use]
pub fn finalize_maturity_announcement(
    construction: MaturityConstruction,
) -> FinalizedMaturityAnnouncement {
    let protected_bytes = construction.transaction().encode();
    let outputs = MaturityFinalizedOutputCensus {
        outputs: construction.transaction().outputs().to_vec(),
        successor_position: SUCCESSOR_OUTPUT_POSITION,
        sponsor_change_position: None,
        fee_position: None,
    };

    let view = construction.validated_view().view();
    let spent = MaturitySpentOutputRecord {
        position: STATE_INPUT_POSITION,
        outpoint: view.current_state_outpoint(),
        asset: view.asset(),
        value: view.value(),
        program: view.predecessor_program().to_vec(),
    };

    FinalizedMaturityAnnouncement {
        construction,
        protected_bytes,
        outputs,
        spent,
    }
}

/// One output index as a position, saturating rather than wrapping.
///
/// A transaction with more than `u16::MAX` outputs is not one this
/// crate's own construction can build, and reporting the last
/// representable position is a true statement about where the report
/// stopped being exact.
fn position_of(index: usize) -> u16 {
    u16::try_from(index).unwrap_or(u16::MAX)
}

/// Where two protected encodings first disagree.
///
/// The common prefix's length when one is a prefix of the other, which
/// is where the shorter stopped saying anything. An offset rather than
/// the bytes: the finding is that they differ, and a caller offering a
/// candidate already holds every byte of it.
fn first_difference(finalized: &[u8], offered: &[u8]) -> usize {
    finalized
        .iter()
        .zip(offered)
        .position(|(finalized, offered)| finalized != offered)
        .unwrap_or_else(|| finalized.len().min(offered.len()))
}
