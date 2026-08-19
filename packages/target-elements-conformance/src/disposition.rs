//! What Guide 11 decided about each confidential-to-public candidate.
//!
//! Guide 11 §9–§12 states four candidate mechanisms and asks each wave to
//! reach an explicit disposition rather than to leave one implied by
//! silence. This module is where those dispositions are recorded, in the
//! guide's own vocabulary and with the evidence each one rests on named.
//!
//! # Why a disposition lives here rather than in the target package
//!
//! Wave 5 put [`OpeningFeasibility`] in `target-elements`, and correctly:
//! whether the reviewed language *can* carry an opening is a target fact,
//! established by reading the target's own source. A disposition is not a
//! target fact. It is this project's decision about a candidate, taken in
//! the light of target facts, and Guide 11 §6.2 puts candidate work in
//! this package for exactly that reason. The two are kept apart so that a
//! later reader can change a decision without appearing to have changed
//! what the target does.
//!
//! Because the split is drawn there, the blockers a disposition names are
//! Wave 5's own [`OpeningBlocker`] values rather than a restatement of
//! them. A register that spelled its blockers out again could drift from
//! the review, and the drift would read as a second opinion.
//!
//! # A deferral is not a rejection, and one candidate is both
//!
//! Guide 11 §11.4 rejects any candidate that loses parity, and two of the
//! three reviewed blockers are parity facts. That refuses the §11.3 proof
//! outline as instantiated on the reviewed primitives — a specific shape,
//! rejected against a specific criterion. It does not refuse the
//! candidate *class*: a pattern that proved the normalization or negation
//! §11.4 asks for would not be caught by it, and no such pattern was
//! found rather than shown impossible.
//!
//! So [`Candidate::DirectAuthenticatedOpening`] carries both, and the
//! record says which is which. Collapsing them either way would be a
//! false statement: "rejected" would claim an impossibility the review
//! never established, and "deferred" alone would drop a shape §11.4
//! actually refuses.
//!
//! [`OpeningBlocker`]: target_elements::confidential::OpeningBlocker
//! [`OpeningFeasibility`]: target_elements::confidential::OpeningFeasibility

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use target_elements::confidential::OpeningBlocker;

/// One mechanism Guide 11 §9–§12 states and a wave must dispose of.
///
/// # Explicit-only is deliberately absent
///
/// Guide 11 §9's explicit-only boundary is not a fourth entry here. It is
/// a *policy over* the mechanisms rather than a mechanism: §20.3 makes
/// its acceptance conditional on how the others were disposed of, and
/// §24's final matrix is what selects it. Listing it beside them would
/// invite a wave to select the policy while disposing of a candidate,
/// which is the one order the guide forbids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Candidate {
    /// Guide 11 §10 — owner-authorized normalization of a private value
    /// into a public representation.
    OwnerAuthorizedNormalization,
    /// Guide 11 §11 — a target program that authenticates the input
    /// amount during the same transition.
    DirectAuthenticatedOpening,
    /// Guide 11 §9.1 and §10.2 — an output that keeps commitment algebra
    /// and publishes an authenticated amount and opening.
    PublicCommittedRepresentation,
    /// Guide 11 §12 — the durable record that carries an opening to a
    /// party that did not participate in creation.
    PublicOpeningCapsule,
}

impl Candidate {
    /// The complete census of candidates this register disposes of.
    pub const ALL: &'static [Self] = &[
        Self::OwnerAuthorizedNormalization,
        Self::DirectAuthenticatedOpening,
        Self::PublicCommittedRepresentation,
        Self::PublicOpeningCapsule,
    ];
}

/// Where a candidate stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DispositionState {
    /// Prototyped, executed against the target, and reported on.
    ///
    /// This state claims evidence exists, not that a policy selected the
    /// candidate: selection is Guide 11 §24's.
    Prototyped,
    /// Stated and not built, against blockers named one at a time.
    ///
    /// The named blockers are what separates this from a schedule. A
    /// deferral whose reason is "no wave reached it" is an absence
    /// wearing a decision's clothes, and Guide 11 §20.3 will not take
    /// one: it requires typed reasons.
    DeferredWithNamedBlocker,
    /// The question does not arise while what it serves is deferred.
    ///
    /// Distinct from a deferral of its own. Nothing about this candidate
    /// was examined and found wanting; what it exists to carry is not
    /// there.
    NotApplicableWhileDeferred,
}

/// A criterion under which one candidate shape is refused.
///
/// These are the guide's own rejection rules, cited rather than
/// paraphrased, so that a reader can check a refusal against the text
/// that authorizes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RejectionCriterion {
    /// Guide 11 §11.4 — a candidate that loses parity is rejected.
    ///
    /// An x-only primitive that implicitly selects one y cannot verify a
    /// general compressed commitment unless the pattern proves the
    /// normalization or negation that makes the two relations equivalent.
    ParityLost,
    /// Guide 11 §21 — a candidate whose opening only a host library
    /// verifies is rejected.
    ///
    /// The refusal is not that host verification is wrong; it is that a
    /// host-verified opening is not an *on-script* opening, and a
    /// candidate claiming the second while delivering the first would
    /// misreport where the authentication happened.
    OnlyHostLibraryVerifiesOpening,
}

/// One candidate shape refused against one criterion.
///
/// A rejected shape is narrower than a rejected candidate, and the
/// difference carries the whole content of the §11 disposition: naming
/// the shape says what was refused, and naming the criterion says under
/// which rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedShape {
    /// The shape refused, as the guide names it.
    pub shape: String,
    /// The rule it is refused under.
    pub criterion: RejectionCriterion,
}

/// One piece of evidence a disposition rests on.
///
/// Typed rather than prose so that a reader can tell a source review from
/// a target run from an oracle result without reading a sentence, and so
/// that a disposition citing nothing is visibly citing nothing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
#[non_exhaustive]
pub enum DispositionEvidence {
    /// A recorded project finding, by its identifier.
    Finding {
        /// The finding's identifier in the project backlog.
        id: String,
        /// What the finding established, in one sentence.
        note: String,
    },
    /// A row of the Guide 11 §8.4 conservation matrix.
    ConservationRow {
        /// The row's position in the guide's own table.
        ordinal: u32,
        /// What the row established.
        note: String,
    },
    /// A fact the Wave 5 source review established by reading the target.
    ReviewedTargetFact {
        /// What the review found.
        note: String,
    },
}

/// One candidate's disposition, with what it rests on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDisposition {
    /// Which candidate.
    pub candidate: Candidate,
    /// Where it stands.
    pub state: DispositionState,
    /// The reviewed blockers this disposition names, where it names any.
    ///
    /// Wave 5's own values. A deferral with an empty set would be a
    /// deferral without a named blocker, which
    /// [`DispositionState::DeferredWithNamedBlocker`] does not admit and
    /// which this module's tests refuse.
    ///
    /// Serialized through the wire spellings in [`crate::vocabulary`],
    /// because the owning crate is standard-library-only and derives no
    /// serialization of its own.
    #[serde(with = "blocker_spellings")]
    pub blockers: BTreeSet<OpeningBlocker>,
    /// Shapes refused outright, with the rule each is refused under.
    pub rejected_shapes: Vec<RejectedShape>,
    /// What this candidate waits on, where its state is a dependency.
    pub depends_on: Option<Candidate>,
    /// Why the disposition is what it is, in one paragraph.
    pub reasoning: String,
    /// What the reasoning rests on.
    pub evidence: Vec<DispositionEvidence>,
}

/// Serializes a blocker set as its reviewed wire spellings.
///
/// An unknown spelling is refused rather than skipped. A register that
/// dropped a blocker it could not name would report a narrower deferral
/// than the review established, which is the one direction this data must
/// not be able to drift in.
mod blocker_spellings {
    use std::collections::BTreeSet;

    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use target_elements::confidential::OpeningBlocker;

    use crate::vocabulary::{opening_blocker_from_name, opening_blocker_name};

    pub(super) fn serialize<S: Serializer>(
        blockers: &BTreeSet<OpeningBlocker>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let names: Vec<&'static str> = blockers
            .iter()
            .map(|blocker| opening_blocker_name(*blocker).unwrap_or("unspelled_opening_blocker"))
            .collect();
        names.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeSet<OpeningBlocker>, D::Error> {
        let names = Vec::<String>::deserialize(deserializer)?;
        names
            .into_iter()
            .map(|name| {
                opening_blocker_from_name(&name)
                    .ok_or_else(|| D::Error::custom(format!("no reviewed blocker is named {name}")))
            })
            .collect()
    }
}

/// The Guide-11 candidate disposition register.
///
/// # Every entry is a decision this project can be held to
///
/// The register is built rather than loaded, because it states
/// conclusions rather than data: a disposition read out of a file could
/// be edited without any of the reasoning moving with it, and the
/// reasoning is the part that has to survive.
#[must_use]
// One literal per candidate, each carrying its own reasoning and its own
// citations. Splitting this to satisfy a line count would scatter four
// decisions a reader checks against one guide across several functions,
// and the reasoning is most of the length.
#[allow(clippy::too_many_lines)]
pub fn candidate_dispositions() -> Vec<CandidateDisposition> {
    vec![
        CandidateDisposition {
            candidate: Candidate::OwnerAuthorizedNormalization,
            state: DispositionState::Prototyped,
            blockers: BTreeSet::new(),
            rejected_shapes: Vec::new(),
            depends_on: None,
            reasoning: "The private-to-explicit-with-private-change variant of \
                 Guide 11 §10.2 is constructible on this target and is prototyped \
                 here: every consumed owner signs the finalized output set, the \
                 amount, explicit asset, and owner are preserved, and the \
                 mutations of §10.4 are executed against a real node. The \
                 full-consumption variant is not part of the prototype, because \
                 the conservation matrix established that it cannot be built at \
                 all. Being prototyped is not being selected: the policy that \
                 selects a candidate is §24's."
                .to_owned(),
            evidence: vec![
                DispositionEvidence::ConservationRow {
                    ordinal: 4,
                    note: "A confidential input paying an explicit output and a \
                           private change output is accepted by the target."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-W7-03".to_owned(),
                    note: "Several confidential inputs to a single explicit output \
                           is not constructible: residual blinding has nowhere to \
                           go without a blinded output to absorb it. The blinded \
                           change variant is the constructible one."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-W10-01".to_owned(),
                    note: "The prototype is built and run: the unmutated claim is \
                           accepted by a real node, and every input is authorized \
                           by a taproot key-path signature carrying no sighash \
                           byte, which is the default all-outputs \
                           non-anyone-can-pay profile §10.3 requires."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-W10-02".to_owned(),
                    note: "All nine §10.4 mutations were executed against that \
                           node and every one met the layer expected of it before \
                           the run. Three are consensus-valid transactions the \
                           report layer alone refuses, which is why the closure \
                           check exists rather than being left to consensus."
                        .to_owned(),
                },
            ],
        },
        CandidateDisposition {
            candidate: Candidate::DirectAuthenticatedOpening,
            state: DispositionState::DeferredWithNamedBlocker,
            blockers: OpeningBlocker::ALL.iter().copied().collect(),
            rejected_shapes: vec![
                RejectedShape {
                    shape: "the §11.3 proof outline instantiated on the reviewed \
                            primitives, which would compare a commitment through an \
                            x-only primitive that implicitly selects one y"
                        .to_owned(),
                    criterion: RejectionCriterion::ParityLost,
                },
                RejectedShape {
                    shape: "an opening checked off-script and asserted on-script, \
                            which would present a host verdict as a target one"
                        .to_owned(),
                    criterion: RejectionCriterion::OnlyHostLibraryVerifiesOpening,
                },
            ],
            depends_on: None,
            reasoning: "No complete on-script form exists under the reviewed \
                 revision, and the review named three independent blockers rather \
                 than one general difficulty. Two of the three are parity facts, \
                 which is why the §11.3 outline as instantiated is refused outright \
                 under §11.4 rather than merely postponed. The candidate class is \
                 deferred rather than rejected: a pattern that proved the \
                 normalization or negation §11.4 asks for would not be caught by \
                 the criterion, and none was found rather than shown impossible. \
                 A wave that reaches one answers the blockers one at a time."
                .to_owned(),
            evidence: vec![
                DispositionEvidence::Finding {
                    id: "G11-C03".to_owned(),
                    note: "An authenticated public opening has no complete \
                           on-script form under the reviewed revision; three \
                           independent blockers are named and typed."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-C01".to_owned(),
                    note: "The confidential encodings record whether y is a \
                           quadratic residue and the curve primitives record \
                           whether y is odd, so no pattern carries from one \
                           convention to the other by analogy."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-O02".to_owned(),
                    note: "The upstream fixture named two_g encodes the negation \
                           of the doubled base point, so reading a fixture name as \
                           a claim would put a sign error into every later \
                           commitment."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-O03".to_owned(),
                    note: "Flipping a commitment's parity bit yields a well-formed \
                           encoding of the negated point, which is the \
                           encoding-level face of the unbound-parity blocker."
                        .to_owned(),
                },
                DispositionEvidence::ReviewedTargetFact {
                    note: "The reviewed language performs neither of the two curve \
                           maps nor the point addition an asset generator recipe \
                           needs, so a program cannot derive the generator it would \
                           have to open against."
                        .to_owned(),
                },
            ],
        },
        CandidateDisposition {
            candidate: Candidate::PublicCommittedRepresentation,
            state: DispositionState::DeferredWithNamedBlocker,
            blockers: OpeningBlocker::ALL.iter().copied().collect(),
            rejected_shapes: Vec::new(),
            depends_on: None,
            reasoning: "A public committed output is one whose published amount and \
                 opening a later relation can verify. On this target that \
                 verification has no on-script form, and the blockers are the same \
                 three: an output that published an opening nothing could check \
                 would be publishing unauthenticated metadata, which §21 refuses by \
                 name. The representation is therefore deferred rather than built, \
                 and the conservation matrix already carries the deferral as a \
                 typed row rather than as an absence."
                .to_owned(),
            evidence: vec![
                DispositionEvidence::ConservationRow {
                    ordinal: 3,
                    note: "The confidential-to-public-committed row is stated and \
                           deferred, because executing it would mean selecting a \
                           candidate none of which is selected."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-C03".to_owned(),
                    note: "The same three blockers that deny an on-script opening \
                           deny the verification a public committed output exists \
                           to offer."
                        .to_owned(),
                },
                DispositionEvidence::Finding {
                    id: "G11-W7-09".to_owned(),
                    note: "Both public-committed rows of §8.4 remain deferred \
                           against those blockers, and one of the two is executed \
                           only in its explicit form."
                        .to_owned(),
                },
            ],
        },
        CandidateDisposition {
            candidate: Candidate::PublicOpeningCapsule,
            state: DispositionState::NotApplicableWhileDeferred,
            blockers: BTreeSet::new(),
            rejected_shapes: Vec::new(),
            depends_on: Some(Candidate::PublicCommittedRepresentation),
            reasoning: "A capsule's contents are an opening and the fields that bind \
                 it to one output, and Guide 11 §12.1 admits only fields not \
                 authenticated elsewhere. With no public committed representation \
                 there is no opening to carry and no output to bind it to, so the \
                 canonical encoding, swap resistance, and signature commitment of \
                 §12.4 to §12.6 have no subject. Nothing about the capsule was \
                 examined and found wanting; what it exists to carry is not there. \
                 The normalization prototype does not supply one either: its public \
                 output is explicit, and an explicit amount is already public chain \
                 data that needs no capsule to be recovered."
                .to_owned(),
            evidence: vec![DispositionEvidence::Finding {
                id: "G11-C03".to_owned(),
                note: "The representation a capsule would serve is deferred against \
                       the three reviewed blockers."
                    .to_owned(),
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_candidate_is_disposed_of_exactly_once() {
        let register = candidate_dispositions();
        let named: BTreeSet<_> = register.iter().map(|entry| entry.candidate).collect();
        assert_eq!(named.len(), register.len());
        assert_eq!(named, Candidate::ALL.iter().copied().collect());
    }

    #[test]
    fn a_deferral_names_at_least_one_blocker() {
        // The whole difference between a deferral and a schedule. Guide 11
        // §20.3 takes typed reasons and nothing else, so a deferral with an
        // empty blocker set is a decision this register must not be able to
        // express by accident.
        for entry in candidate_dispositions() {
            if entry.state == DispositionState::DeferredWithNamedBlocker {
                assert!(!entry.blockers.is_empty(), "{:?}", entry.candidate);
            }
        }
    }

    #[test]
    fn a_not_applicable_disposition_names_what_it_waits_on() {
        for entry in candidate_dispositions() {
            if entry.state == DispositionState::NotApplicableWhileDeferred {
                let waited_on = entry.depends_on.expect("a dependency is named");
                // And what it waits on must itself be deferred, or the
                // dependency is stale rather than pending.
                let target = candidate_dispositions()
                    .into_iter()
                    .find(|other| other.candidate == waited_on)
                    .expect("the dependency is in the register");
                assert_eq!(target.state, DispositionState::DeferredWithNamedBlocker);
            }
        }
    }

    #[test]
    fn the_direct_opening_carries_both_a_deferral_and_a_refused_shape() {
        // Guide 11 §11.4 refuses the outline as instantiated; the class is
        // deferred. Collapsing either into the other would be a false
        // statement, so both have to be present at once.
        let entry = candidate_dispositions()
            .into_iter()
            .find(|entry| entry.candidate == Candidate::DirectAuthenticatedOpening)
            .expect("the direct opening is disposed of");
        assert_eq!(entry.state, DispositionState::DeferredWithNamedBlocker);
        assert_eq!(entry.blockers.len(), OpeningBlocker::ALL.len());
        assert!(
            entry
                .rejected_shapes
                .iter()
                .any(|shape| shape.criterion == RejectionCriterion::ParityLost)
        );
    }

    #[test]
    fn only_the_prototyped_candidate_claims_no_blocker() {
        let register = candidate_dispositions();
        let unblocked: Vec<_> = register
            .iter()
            .filter(|entry| entry.blockers.is_empty() && entry.depends_on.is_none())
            .map(|entry| entry.candidate)
            .collect();
        assert_eq!(unblocked, vec![Candidate::OwnerAuthorizedNormalization]);
    }

    #[test]
    fn a_prototyped_disposition_cites_the_run_that_makes_it_true() {
        // This register once recorded `Prototyped` while no prototype
        // existed, and the backlog had to carry the correction as a
        // separate row. The state claims evidence exists, so it has to
        // name the findings that hold it: a wave that deletes the
        // prototype now has to delete these citations too, which is
        // visible, rather than leaving a state that quietly overstates.
        for entry in candidate_dispositions() {
            if entry.state != DispositionState::Prototyped {
                continue;
            }
            let cited: BTreeSet<&str> = entry
                .evidence
                .iter()
                .filter_map(|item| match item {
                    DispositionEvidence::Finding { id, .. } => Some(id.as_str()),
                    _ => None,
                })
                .collect();
            assert!(
                cited.contains("G11-W10-01") && cited.contains("G11-W10-02"),
                "{:?} claims to be prototyped and cites {cited:?}",
                entry.candidate
            );
        }
    }

    #[test]
    fn every_disposition_cites_something() {
        for entry in candidate_dispositions() {
            assert!(!entry.evidence.is_empty(), "{:?}", entry.candidate);
            assert!(!entry.reasoning.is_empty(), "{:?}", entry.candidate);
        }
    }
}
