//! The owner-authorization case census (Guide-13 §1.8, §9.2, §19.3).
//!
//! # The failure this census is shaped against
//!
//! A mutation case is a transaction the target must reject, and the
//! only thing that makes it one is that the signature commits to what
//! the mutation changed. Under a narrower sighash profile the very same
//! case is a transaction the target *accepts*: the output was changed,
//! the signature never covered it, and the run reports a pass. Nothing
//! about the case's own description would look different — same
//! fixture, same mutation, same name — and the evidence it produced
//! would be worth nothing.
//!
//! So every mutation case here names the protected datum it disturbs,
//! and [`validate_case_census`] requires that datum to be one something
//! commits to. The production path derives that set from the selected
//! profile's required dimensions
//! ([`committed_protected_data`]), so the coupling is real: a profile
//! narrowed later stops this census validating rather than quietly
//! turning its mutation cases into passes.
//!
//! # Nothing here is target evidence
//!
//! A case states what a target-native run must show, and each one
//! carries whatever still stands between it and a run (§1.11: a
//! target-negative claim requires a complete target transaction and an
//! observed target verdict). No case records a verdict, and the type has
//! no field one could occupy.
//!
//! That last sentence is what carries the non-claim, and it is worth
//! being exact about now that most cases carry no residual at all. The
//! unreviewed-profile residual stood on every case and was doing two
//! jobs: naming the profile gap, and standing in as the marker that
//! nothing here had been run. The review verdict and the owner's
//! re-typing ruling closed the first, so it is cleared — and the second
//! was never its to carry. An empty residual set means nothing in this
//! vocabulary still holds the case back. It does not mean the case has
//! been run, and none has: what forbids that claim is the absence of a
//! verdict field, which is a property of the type rather than of a set
//! that happened to be non-empty.
//!
//! # Why the three lists are one census
//!
//! §9.2 names eight target-native tests, §1.8 names six negatives for
//! the key encoding, and §19.3 names eleven authorization cases. They
//! overlap and none contains the others. Keeping three lists would have
//! meant three places for a case to be dropped from and two ways for
//! them to disagree about what a shared case is called, so they are
//! merged here and [`covers_every_required_list`] checks the merge
//! against each list separately.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::{OwnerKeyNegative, OwnerSighashProfile, ProtectedDatum, selected_owner_profile};

/// One owner-authorization case a target-native run must exhibit.
///
/// Declaration order is a stable census order and ranks nothing: the
/// accepting case is first because a census whose only positive is
/// buried reads as a census of failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerAuthorizationCaseId {
    /// The approved owner signs the finalized transaction, and the
    /// target accepts it.
    ValidOwnerSignature,

    /// No signature is offered for a required owner.
    MissingSignature,
    /// An empty signature is offered.
    ///
    /// Distinct from the missing one because the reviewed primitive
    /// treats it differently: an empty signature consumes its operands
    /// and pushes a false, which a program can branch on, where a
    /// missing witness item is a structural fault before any program
    /// runs.
    EmptySignature,
    /// A non-empty signature that does not verify is offered.
    InvalidSignature,
    /// A different approved owner's signature is offered.
    WrongOwner,
    /// An empty owner key is offered.
    ///
    /// §1.8's first negative, and distinct from every signature-side
    /// case: the reviewed primitive refuses the empty key outright,
    /// through a failure cause of its own, where a missing witness item
    /// is a structural fault before the key is read at all. Mapping
    /// §1.8's empty key onto [`Self::MissingSignature`] was the earlier
    /// reading, and it lost the one negative the target has a dedicated
    /// refusal for.
    EmptyOwnerKey,
    /// A key of a form the target does not recognize is offered.
    ///
    /// The forward-compatibility case §1.8 is written for: the reviewed
    /// primitive reports success for it without verifying anything.
    UnknownKeyForm,
    /// A key claiming the approved encoding that is not a well-formed
    /// member of it is offered.
    MalformedApprovedKey,
    /// A signature that verifies against some other key is offered.
    SignatureAgainstAnotherKey,
    /// A signature the owner took over a different transaction is
    /// offered.
    SignatureBoundToAnotherTransaction,
    /// The signature is taken under a sighash byte other than the
    /// selected profile's.
    WrongSighashByte,

    /// An output is changed after the owner signed.
    OutputChangedAfterSigning,
    /// An output is removed after the owner signed.
    OutputRemovedAfterSigning,
    /// An input is added after the owner signed.
    InputAddedAfterSigning,

    /// One of several required owners does not authorize.
    IncompleteOwnerSet,
    /// One owner holds several inputs and one of their concrete
    /// witnesses is omitted.
    ///
    /// The case a per-owner check passes and a per-input check fails.
    /// Collapsing it into [`Self::IncompleteOwnerSet`] would lose
    /// exactly the distinction §19.3 lists it for.
    RepeatedOwnerWithOneWitnessOmitted,
    /// The sponsor's own owner does not authorize.
    SponsorOwnerOmission,
}

impl OwnerAuthorizationCaseId {
    /// The complete census, in the type's own canonical order.
    pub const ALL: &'static [Self] = &[
        Self::ValidOwnerSignature,
        Self::MissingSignature,
        Self::EmptySignature,
        Self::InvalidSignature,
        Self::WrongOwner,
        Self::EmptyOwnerKey,
        Self::UnknownKeyForm,
        Self::MalformedApprovedKey,
        Self::SignatureAgainstAnotherKey,
        Self::SignatureBoundToAnotherTransaction,
        Self::WrongSighashByte,
        Self::OutputChangedAfterSigning,
        Self::OutputRemovedAfterSigning,
        Self::InputAddedAfterSigning,
        Self::IncompleteOwnerSet,
        Self::RepeatedOwnerWithOneWitnessOmitted,
        Self::SponsorOwnerOmission,
    ];
}

/// What a target-native run must observe for one case.
///
/// Two members, and neither is "passes". A target verdict is the
/// observation; this is what the case is *for*, stated before the run
/// so that a run cannot be read as having confirmed whatever it
/// happened to produce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerAuthorizationExpectation {
    /// The target accepts the transaction.
    Accepted,
    /// The target refuses the transaction.
    Refused,
}

/// What still stands between one case and a target-native run.
///
/// Every case carried one while the profile the whole census is stated
/// against was unreviewed: that was the honest common residual, and the
/// cases needing more said what more. The common one is cleared, so the
/// set is now empty for every case but the sponsor's.
///
/// An empty set is therefore not a case that could run today. It is a
/// case with nothing left *in this vocabulary*, which is a narrower
/// statement and the only one this type was ever able to make — §1.11's
/// bar is met by the module's own shape, where no case records a verdict
/// and there is no field one could occupy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CaseResidual {
    /// The selected sighash profile is not established by the review,
    /// so no run can yet say the signature committed to what the case
    /// assumes it committed to.
    ///
    /// # It is no longer carried, and the verdict that moved it
    ///
    /// This is the sharpest place the profile gap bit, because the
    /// failure this whole census is shaped against is exactly the one an
    /// unestablished profile leaves open: under a narrower profile a
    /// mutation case is a transaction the target *accepts*, and the
    /// evidence it produced would be worth nothing.
    ///
    /// The review verdict established six of the profile's seven
    /// required dimensions on one observed acceptance, and a
    /// post-verdict re-typing moved the seventh — the issuance
    /// dimension, which no candidate this arc builds can exercise
    /// because both of its message terms are formed from the input count
    /// alone — from required to refused, on the ground that the census
    /// refuses an issuance-bearing signing request and the decoder
    /// refuses issuance-bearing bytes. The required set is six, every
    /// member of it is established, and the recomputed disposition is
    /// established. So the residual is gone from every case.
    ///
    /// It cleared on a review verdict and never on a run, which is the
    /// order the two owner-sighash residuals were separated to keep: the
    /// digest blocker moved first, on an observed acceptance, and this
    /// moved afterwards, on a verdict. The two were never simultaneous
    /// and never rested on the same evidence.
    ///
    /// # What did not move with it
    ///
    /// No case. Not one case here is answered, discharged, or run, and
    /// clearing this residual could not have done any of those: a case
    /// is answered by an observed target verdict, this package records
    /// none, and the type still has no field one could occupy. What the
    /// clearing removes is a reason a case could not yet be *stated as
    /// evidence*, not a reason it had not been run.
    ///
    /// The mutation coupling did not move either, and it is the half a
    /// reader should check rather than trust: every mutation case still
    /// names the protected datum it disturbs, and the census still
    /// validates that datum against the profile's own committed set, so
    /// a profile narrowed later still stops this census validating.
    ///
    /// The word stays in this vocabulary because it is still the right
    /// name for the condition, and a census stated against a profile
    /// whose review had lapsed must be able to say so.
    ProfileUnreviewed,
    /// The case additionally needs a sponsor envelope with its own
    /// authorizing owner.
    ///
    /// §12's candidate ABI builds the sponsored form — a sponsor suffix,
    /// a sponsor-change role, and a fee role — but the sponsor's own
    /// authorization arrives through an adapter that hands back a
    /// witness stack, and §1.9 keeps that outside protocol data. A case
    /// about the sponsor's owner failing to authorize therefore still
    /// needs an envelope whose signer is modelled rather than supplied,
    /// and this residual says so rather than being flipped alongside the
    /// multi-owner one.
    SponsorEnvelope,
}

/// One owner-authorization case (§9.2, §19.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerAuthorizationCase {
    id: OwnerAuthorizationCaseId,
    expectation: OwnerAuthorizationExpectation,
    disturbed: Option<ProtectedDatum>,
    residuals: BTreeSet<CaseResidual>,
}

impl OwnerAuthorizationCase {
    /// The case's identity.
    #[must_use]
    pub const fn id(&self) -> OwnerAuthorizationCaseId {
        self.id
    }

    /// What a target-native run must observe.
    #[must_use]
    pub const fn expectation(&self) -> OwnerAuthorizationExpectation {
        self.expectation
    }

    /// The protected datum this case disturbs after signing, where it
    /// disturbs one.
    ///
    /// `None` for the cases that alter the *witness* rather than the
    /// signed transaction: an absent, empty, malformed, or foreign key
    /// changes what is offered, not what was covered, so there is no
    /// protected datum for the profile to have to carry.
    #[must_use]
    pub const fn disturbed(&self) -> Option<ProtectedDatum> {
        self.disturbed
    }

    /// What still stands between this case and a target-native run.
    pub fn residuals(&self) -> impl Iterator<Item = CaseResidual> + '_ {
        self.residuals.iter().copied()
    }
}

/// Why a case census does not stand up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseCensusDefect {
    /// The census names a case twice, or omits one.
    CensusMismatch {
        /// The cases the census constant names and the set does not.
        missing: BTreeSet<OwnerAuthorizationCaseId>,
        /// The cases the set names and the census constant does not.
        unexpected: BTreeSet<OwnerAuthorizationCaseId>,
    },
    /// A mutation case disturbs a datum the selected profile does not
    /// commit to.
    ///
    /// The failure this census exists to prevent: the case would be a
    /// transaction the target accepts, reported as one it rejects.
    UncommittedMutation {
        /// The case whose mutation the signature would not cover.
        case: OwnerAuthorizationCaseId,
        /// The datum it disturbs.
        datum: ProtectedDatum,
    },
    /// The census carries no accepting case.
    ///
    /// A census of refusals establishes that the pattern refuses
    /// things, which every pattern that refuses everything also does.
    NoAcceptingCase,
    /// A case claims it could run with nothing outstanding.
    MissingResidual {
        /// The case claiming to be runnable.
        case: OwnerAuthorizationCaseId,
    },
}

/// The census of owner-authorization cases (§9.2, §19.3).
#[must_use]
pub fn owner_authorization_cases() -> BTreeMap<OwnerAuthorizationCaseId, OwnerAuthorizationCase> {
    use CaseResidual as Residual;
    use OwnerAuthorizationCaseId as Case;
    use OwnerAuthorizationExpectation as Expect;

    OwnerAuthorizationCaseId::ALL
        .iter()
        .map(|id| {
            let (expectation, disturbed, extra) = match id {
                Case::ValidOwnerSignature => (Expect::Accepted, None, None),

                // Witness-side faults, and the wrong sighash byte with
                // them. Each changes what is *offered* rather than what
                // was covered, so no protected datum is disturbed and
                // the commitment has nothing to carry for them.
                //
                // The sighash byte belongs here rather than with the
                // mutations for a sharper reason: it changes which
                // dimensions the digest covers at all, so what it
                // disturbs is the profile rather than a datum inside
                // it. Naming a datum would claim it was an ordinary
                // mutation, which is the one thing it is not.
                Case::MissingSignature
                | Case::EmptySignature
                | Case::InvalidSignature
                | Case::WrongOwner
                | Case::EmptyOwnerKey
                | Case::UnknownKeyForm
                | Case::MalformedApprovedKey
                | Case::SignatureAgainstAnotherKey
                | Case::WrongSighashByte => (Expect::Refused, None, None),

                // Adding an input changes the consumed set, and a
                // signature taken over another transaction disturbs
                // every protected datum at once. The receipt inputs are
                // named for both because a transaction that consumed
                // different receipts is the least arguable way for two
                // transactions to differ.
                Case::SignatureBoundToAnotherTransaction | Case::InputAddedAfterSigning => {
                    (Expect::Refused, Some(ProtectedDatum::ReceiptInputs), None)
                }

                Case::OutputChangedAfterSigning => (
                    Expect::Refused,
                    Some(ProtectedDatum::DestinationSemanticValues),
                    None,
                ),
                Case::OutputRemovedAfterSigning => (
                    Expect::Refused,
                    Some(ProtectedDatum::DestinationEntries),
                    None,
                ),

                // §12's candidate ABI builds both of these transactions.
                // A transfer consuming two owners' receipts is what the
                // multi-owner case needs; a transfer consuming two
                // receipts of one owner is what the repeated-owner case
                // needs, and the ABI's own owner census reports it as
                // one semantic owner and two concrete signatures, which
                // is the distinction §1.6 lists the case for. The
                // `MultiOwnerTransaction` residual is therefore gone
                // rather than kept as a discharged marker.
                Case::IncompleteOwnerSet | Case::RepeatedOwnerWithOneWitnessOmitted => {
                    (Expect::Refused, None, None)
                }
                Case::SponsorOwnerOmission => {
                    (Expect::Refused, None, Some(Residual::SponsorEnvelope))
                }
            };

            // The unreviewed profile stood on every case here until the
            // review verdict and the re-typing cleared
            // it, so what remains is whatever the case needs beyond it —
            // which for all but the sponsor case is nothing.
            let residuals: BTreeSet<_> = extra.into_iter().collect();

            (
                *id,
                OwnerAuthorizationCase {
                    id: *id,
                    expectation,
                    disturbed,
                    residuals,
                },
            )
        })
        .collect()
}

/// The protected data one profile's required dimensions carry.
///
/// The seam between the profile and the census, and it is a seam rather
/// than a direct read for two reasons. A validator that took the
/// profile could only ever be exercised against the one profile that
/// exists, and the profile deliberately has no public constructor, so
/// the guard below could never be shown firing. And the guard is about
/// *coverage*, not about a profile: what a mutation case needs is that
/// something commits to what it changed, and this set is exactly that
/// question's answer.
///
/// The production path derives the set from the selected profile, so
/// the coupling is real; a test narrows the set, so the coupling is
/// checkable.
///
/// The carrier lookup is set-valued, and the question this asks of the
/// set is "any" rather than "all". A datum is committed as soon as one
/// dimension carrying it is required, because the paragraphs above state
/// the need exactly: what a mutation case needs is that *something*
/// commits to what it changed. Reading it as "all" would drop a datum
/// the signature demonstrably commits to out of the set the moment the
/// coverage map recorded a second, weaker carrier for it — punishing a
/// more complete declaration, which is the opposite of what widening the
/// map was for.
#[must_use]
pub fn committed_protected_data(profile: &OwnerSighashProfile) -> BTreeSet<ProtectedDatum> {
    let required = profile.required().collect::<BTreeSet<_>>();

    ProtectedDatum::ALL
        .iter()
        .copied()
        .filter(|datum| {
            profile
                .carrier(*datum)
                .iter()
                .any(|dimension| required.contains(dimension))
        })
        .collect()
}

/// Validate one case census against the data a signature commits to.
///
/// # Errors
///
/// [`CaseCensusDefect::CensusMismatch`] when the set is not exactly the
/// census constant; [`CaseCensusDefect::UncommittedMutation`] when a
/// case disturbs a datum nothing commits to;
/// [`CaseCensusDefect::NoAcceptingCase`] when nothing in the census is
/// expected to be accepted; [`CaseCensusDefect::MissingResidual`] when
/// a case claims nothing stands between it and a run.
pub fn validate_case_census(
    cases: &BTreeMap<OwnerAuthorizationCaseId, OwnerAuthorizationCase>,
    committed: &BTreeSet<ProtectedDatum>,
) -> Result<(), CaseCensusDefect> {
    let present = cases.keys().copied().collect::<BTreeSet<_>>();
    let census = OwnerAuthorizationCaseId::ALL
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    if present != census {
        return Err(CaseCensusDefect::CensusMismatch {
            missing: census.difference(&present).copied().collect(),
            unexpected: present.difference(&census).copied().collect(),
        });
    }

    for case in cases.values() {
        if let Some(datum) = case.disturbed
            && !committed.contains(&datum)
        {
            return Err(CaseCensusDefect::UncommittedMutation {
                case: case.id,
                datum,
            });
        }

        if case.residuals.is_empty() {
            return Err(CaseCensusDefect::MissingResidual { case: case.id });
        }
    }

    if !cases
        .values()
        .any(|case| case.expectation == OwnerAuthorizationExpectation::Accepted)
    {
        return Err(CaseCensusDefect::NoAcceptingCase);
    }

    Ok(())
}

/// The validated census, against the selected profile.
///
/// # Errors
///
/// Any failure of [`validate_case_census`].
pub fn validated_owner_authorization_cases()
-> Result<BTreeMap<OwnerAuthorizationCaseId, OwnerAuthorizationCase>, CaseCensusDefect> {
    let cases = owner_authorization_cases();

    validate_case_census(&cases, &committed_protected_data(&selected_owner_profile()))?;
    Ok(cases)
}

/// The case that answers one of §1.8's required key negatives.
///
/// Total over [`OwnerKeyNegative`], and exhaustive with no wildcard
/// arm: a negative added to that census stops this crate compiling
/// until the case answering it is named. That is the mechanism that
/// keeps the merged census from silently dropping one of the three
/// lists it merges.
#[must_use]
pub const fn case_for_key_negative(negative: OwnerKeyNegative) -> OwnerAuthorizationCaseId {
    match negative {
        OwnerKeyNegative::EmptyKey => OwnerAuthorizationCaseId::EmptyOwnerKey,
        OwnerKeyNegative::UnknownNonemptyKeyType => OwnerAuthorizationCaseId::UnknownKeyForm,
        OwnerKeyNegative::MalformedApprovedKey => OwnerAuthorizationCaseId::MalformedApprovedKey,
        OwnerKeyNegative::ApprovedKeyOfAnotherOwner => OwnerAuthorizationCaseId::WrongOwner,
        OwnerKeyNegative::ValidSignatureAgainstAnotherKey => {
            OwnerAuthorizationCaseId::SignatureAgainstAnotherKey
        }
        OwnerKeyNegative::ValidSignatureOverAnotherTransaction => {
            OwnerAuthorizationCaseId::SignatureBoundToAnotherTransaction
        }
    }
}

/// Whether the census answers every case each source list requires.
///
/// The three lists are checked separately rather than as their union: a
/// union check passes when one list's members happen to be a subset of
/// another's, which is exactly how a merged census loses a list.
#[must_use]
pub fn covers_every_required_list(
    cases: &BTreeMap<OwnerAuthorizationCaseId, OwnerAuthorizationCase>,
) -> bool {
    use OwnerAuthorizationCaseId as Case;

    // §9.2's target-native tests.
    let target_native = [
        Case::ValidOwnerSignature,
        Case::MissingSignature,
        Case::WrongOwner,
        Case::OutputChangedAfterSigning,
        Case::InputAddedAfterSigning,
        Case::OutputRemovedAfterSigning,
        Case::WrongSighashByte,
        Case::SignatureBoundToAnotherTransaction,
    ];
    // §19.3's authorization coverage.
    let coverage = [
        Case::ValidOwnerSignature,
        Case::MissingSignature,
        Case::WrongOwner,
        Case::InvalidSignature,
        Case::EmptySignature,
        Case::UnknownKeyForm,
        Case::OutputChangedAfterSigning,
        Case::InputAddedAfterSigning,
        Case::IncompleteOwnerSet,
        Case::RepeatedOwnerWithOneWitnessOmitted,
        Case::SponsorOwnerOmission,
    ];

    target_native.iter().all(|case| cases.contains_key(case))
        && coverage.iter().all(|case| cases.contains_key(case))
        && OwnerKeyNegative::ALL
            .iter()
            .all(|negative| cases.contains_key(&case_for_key_negative(*negative)))
}
