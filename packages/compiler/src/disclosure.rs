//! Plan-specific disclosure analysis (Guide-3 Tranche F).
//!
//! The realization's declassification analysis is inherited without
//! change of meaning; the compiler only *adds* disclosure, and every
//! addition carries a typed reason naming the proof or representation
//! decision that caused it. The sponsor amount is structurally
//! rejected everywhere — inherited, added, or retained-private:
//! sponsor erasure means the fact is absent, not merely secret.

// One item-level allowance remains: `ProofRequirement` is a declared
// disclosure reason the two pilots never produce, because no pilot
// proof publishes a fact its representation had kept private.

use std::collections::{BTreeMap, BTreeSet};

use realization::{FactId, RepresentationMode};

use crate::{CompileError, lifecycle::RepresentationChoiceId};

/// Why the compiler added one disclosure.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompilerDisclosureReason {
    #[allow(dead_code)]
    ProofRequirement {
        relation: realization::RelationId,
        proof: realization::ProofAlternativeId,
    },

    RepresentationSelection {
        operation: architecture::OperationId,
        object: architecture::ObjectId,
        representation: RepresentationMode,
    },
}

/// One plan candidate's complete disclosure analysis.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompilerDisclosureAnalysis {
    pub inherited_required_public: BTreeMap<FactId, BTreeSet<realization::DisclosureReason>>,
    pub added_required_public: BTreeMap<FactId, BTreeSet<CompilerDisclosureReason>>,
    pub retained_private: BTreeSet<FactId>,
}

/// True for the erased sponsor family's amount fact.
#[must_use]
pub const fn is_sponsor_amount(fact: &FactId) -> bool {
    matches!(
        fact,
        FactId::FamilyAmount {
            object: architecture::ObjectId::PlainLbtc,
            ..
        }
    )
}

/// Derive one candidate's disclosure from the inherited analysis and
/// the candidate's representation selections.
///
/// An explicit (or public-committed) representation selection makes
/// that object's amount facts public, with the selection recorded as
/// the typed reason; a private-committed selection retains them
/// private. Facts the realization already requires public stay
/// inherited, never re-added.
pub fn derive_disclosure(
    inherited: &realization::DeclassificationAnalysis,
    representations: &BTreeMap<RepresentationChoiceId, RepresentationMode>,
) -> Result<CompilerDisclosureAnalysis, CompileError> {
    for fact in inherited
        .required_public
        .keys()
        .chain(inherited.newly_disclosed.keys())
        .chain(inherited.retained_private.iter())
    {
        if is_sponsor_amount(fact) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    let mut analysis = CompilerDisclosureAnalysis {
        inherited_required_public: inherited.required_public.clone(),
        added_required_public: BTreeMap::new(),
        retained_private: inherited.retained_private.clone(),
    };

    for (choice, mode) in representations {
        let public = matches!(
            mode,
            RepresentationMode::Explicit | RepresentationMode::PublicCommitted
        );
        let reason = CompilerDisclosureReason::RepresentationSelection {
            operation: choice.operation,
            object: choice.object,
            representation: *mode,
        };

        // The affected facts are the object's amount facts retained
        // private by the realization for this operation.
        let affected = analysis
            .retained_private
            .iter()
            .filter(|fact| {
                matches!(
                    fact,
                    FactId::FamilyAmount {
                        operation,
                        object,
                        ..
                    } if *operation == choice.operation && *object == choice.object
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        if public {
            for fact in affected {
                analysis.retained_private.remove(&fact);
                analysis
                    .added_required_public
                    .entry(fact)
                    .or_default()
                    .insert(reason.clone());
            }
        }
    }

    for fact in analysis.added_required_public.keys() {
        if is_sponsor_amount(fact) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    Ok(analysis)
}

/// Validate one candidate's disclosure consistency (§10.5).
///
/// Every added disclosure must carry at least one typed reason, and no
/// sponsor amount may appear anywhere.
pub fn validate_disclosure(analysis: &CompilerDisclosureAnalysis) -> Result<(), CompileError> {
    for (fact, reasons) in &analysis.added_required_public {
        if reasons.is_empty() {
            return Err(CompileError::MissingDisclosureReason { fact: fact.clone() });
        }
    }

    for fact in analysis
        .inherited_required_public
        .keys()
        .chain(analysis.added_required_public.keys())
        .chain(analysis.retained_private.iter())
    {
        if is_sponsor_amount(fact) {
            return Err(CompileError::SponsorValueRead);
        }
    }

    Ok(())
}
