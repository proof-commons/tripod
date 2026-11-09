//! Finite attestation proof tables and lossless candidate projection.
use crate::{
    CompileError, engine_proof as core,
    proof::{LocalProofRejection, ProofPlanCandidate, RelationObligation, RelationObligationClass},
    relation::CompilerRelationAnalysis,
    source::{derive_source_requirements, proof_capabilities, relation_operands},
};
use realization::{
    ProofAlternativeId, Relation, RelationDeclaration, RelationId, RepresentationMode,
};
use std::collections::{BTreeMap, BTreeSet};
/// Classify every relation in the analysis exactly once.
pub fn classify_obligations(
    relations: &CompilerRelationAnalysis,
) -> Result<Vec<RelationObligation>, CompileError> {
    let mut obligations = Vec::new();

    for node in relations.graph.node_weights() {
        let declaration = &node.source;
        let operands = relation_operands(declaration)?;

        let class = match &declaration.relation {
            // A representation relation constrains which mode the
            // compiler may select; the mode itself remains a decision
            // variable derived from the relation's allowed set, and no
            // arithmetic proof variable is created for the constraint.
            Relation::LifecycleExit { .. } | Relation::Representation { .. } => {
                RelationObligationClass::StaticallyValidated
            }

            Relation::OperatorAuthorization
            | Relation::Constructibility {
                class: realization::ConstructibilityClass::Operator,
            } => operator_obligation(declaration)?,

            Relation::SubstrateConservation { asset } => {
                // The realization approves exactly one proof class for
                // this relation. The compiler validates that invariant
                // rather than manufacturing the class or taking an
                // arbitrary first element.
                let approved = ProofAlternativeId::new(
                    declaration.id.clone(),
                    realization::ProofKind::SubstrateConservation,
                );

                if declaration.proof_alternatives.len() != 1
                    || !declaration.proof_alternatives.contains(&approved)
                {
                    return Err(CompileError::InvalidExternalEvidenceProofAlternatives {
                        relation: declaration.id.clone(),
                    });
                }

                RelationObligationClass::ExternalEvidence {
                    requirement: realization::ExternalEvidenceRequirement::SubstrateConservation {
                        operation: declaration.id.operation(),
                        asset: *asset,
                    },
                    required_capabilities: proof_capabilities(declaration, approved.proof()),
                    source_requirements: derive_source_requirements(declaration, approved.proof())?,
                    proof: approved,
                }
            }

            _ => {
                let mut alternatives = declaration
                    .proof_alternatives
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                alternatives.sort();

                if alternatives.is_empty() {
                    return Err(CompileError::MissingProofAlternative {
                        relation: declaration.id.clone(),
                    });
                }

                RelationObligationClass::ProofRequired { alternatives }
            }
        };

        obligations.push(RelationObligation {
            relation: declaration.id.clone(),
            class,
            operands,
        });
    }

    obligations.sort_by(|left, right| left.relation.cmp(&right.relation));
    Ok(obligations)
}

fn operator_obligation(
    declaration: &RelationDeclaration,
) -> Result<RelationObligationClass, CompileError> {
    // Both operator relations approve exactly ManifestShape. That class
    // fixes their requirements but cannot discharge operator evidence.
    let approved = ProofAlternativeId::new(
        declaration.id.clone(),
        realization::ProofKind::ManifestShape,
    );

    if declaration.proof_alternatives.len() != 1
        || !declaration.proof_alternatives.contains(&approved)
    {
        return Err(CompileError::InvalidExternalEvidenceProofAlternatives {
            relation: declaration.id.clone(),
        });
    }

    Ok(RelationObligationClass::ExternalEvidence {
        requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
            operation: declaration.id.operation(),
        },
        required_capabilities: proof_capabilities(declaration, approved.proof()),
        source_requirements: derive_source_requirements(declaration, approved.proof())?,
        proof: approved,
    })
}

pub struct Lowered {
    pub records: core::Records,
    relations: BTreeMap<crate::engine_records::RelationKey, RelationId>,
    proofs: BTreeMap<core::AlternativeKey, ProofAlternativeId>,
    choices: BTreeMap<core::ChoiceKey, crate::lifecycle::RepresentationChoiceId>,
    modes: BTreeMap<crate::engine_lifecycle::ModeKey, RepresentationMode>,
    sources: BTreeMap<crate::engine_source::SourceRow, crate::source::SourceRequirement>,
    evidence: BTreeMap<crate::engine_source::Evidence, realization::ExternalEvidenceRequirement>,
    lifecycle: BTreeMap<
        crate::engine_lifecycle::LifecycleRequirementRecord,
        crate::lifecycle::LifecycleRequirement,
    >,
    facts: BTreeMap<core::FactKey, realization::FactId>,
    inherited: BTreeMap<realization::FactId, BTreeSet<realization::DisclosureReason>>,
}

pub fn lower(input: &crate::input::BoundCompilerInput) -> Result<Lowered, CompileError> {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_proofs", scope_count = crate::instrumentation::count(input.scope().operations().len()));
    let _entered = span.enter();
    let relations = crate::relation::build_relation_analysis(input)?;
    let constructibility = crate::constructibility::build_constructibility_analysis(input)?;
    let lifecycle = crate::lifecycle::build_lifecycle_analysis(input, &relations)?;
    let obligations = classify_obligations(&relations)?;
    let declarations = relations
        .graph
        .node_weights()
        .map(|node| (node.source.id.clone(), &node.source))
        .collect::<BTreeMap<_, _>>();
    let mut lowered = Lowered {
        records: core::Records {
            obligations: BTreeMap::new(),
            choices: lifecycle
                .choices
                .iter()
                .map(|choice| {
                    (
                        choice_key(&choice.id),
                        choice.candidates.iter().copied().map(mode_key).collect(),
                    )
                })
                .collect(),
            lifecycle: lifecycle.requirements.iter().map(lifecycle_key).collect(),
            disclosure: disclosure_records(
                input.realization().declassification(),
                &lifecycle.choices,
            ),
        },
        relations: declarations
            .keys()
            .map(|id| (super::relation_key(id), id.clone()))
            .collect(),
        proofs: BTreeMap::new(),
        sources: BTreeMap::new(),
        evidence: BTreeMap::new(),
        choices: lifecycle
            .choices
            .iter()
            .map(|choice| (choice_key(&choice.id), choice.id.clone()))
            .collect(),
        modes: [
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
            RepresentationMode::PublicCommitted,
        ]
        .into_iter()
        .map(|mode| (mode_key(mode), mode))
        .collect(),
        lifecycle: lifecycle
            .requirements
            .iter()
            .map(|row| (lifecycle_key(row), row.clone()))
            .collect(),
        facts: input
            .realization()
            .declassification()
            .retained_private
            .iter()
            .map(|fact| (fact_key(fact), fact.clone()))
            .collect(),
        inherited: input
            .realization()
            .declassification()
            .required_public
            .clone(),
    };
    lowered.lower_obligations(
        obligations,
        &declarations,
        &constructibility,
        &lifecycle.choices,
    )?;
    Ok(lowered)
}

impl Lowered {
    fn lower_obligations(
        &mut self,
        obligations: Vec<RelationObligation>,
        declarations: &BTreeMap<RelationId, &RelationDeclaration>,
        constructibility: &crate::constructibility::CompilerConstructibilityAnalysis,
        choices: &[crate::lifecycle::RepresentationChoice],
    ) -> Result<(), CompileError> {
        for obligation in obligations {
            let declaration = declarations[&obligation.relation];
            let class = match obligation.class {
                RelationObligationClass::StaticallyValidated => core::Obligation::Static,
                RelationObligationClass::ProofRequired { alternatives } => {
                    core::Obligation::Required(
                        alternatives
                            .iter()
                            .map(|alternative| {
                                self.alternative(
                                    declaration,
                                    alternative,
                                    constructibility,
                                    choices,
                                )
                            })
                            .collect(),
                    )
                }
                RelationObligationClass::ExternalEvidence {
                    requirement,
                    proof,
                    source_requirements,
                    ..
                } => {
                    crate::constructibility::validate_source_constructibility(
                        constructibility,
                        &obligation.relation,
                        &source_requirements,
                    )?;
                    let mut alternative =
                        self.alternative(declaration, &proof, constructibility, choices);
                    let key = evidence_key(&requirement);
                    self.evidence.insert(key.clone(), requirement);
                    alternative.evidence.insert(key);
                    core::Obligation::Evidence(alternative)
                }
            };
            self.records
                .obligations
                .insert(super::relation_key(&obligation.relation), class);
        }
        Ok(())
    }
    fn alternative(
        &mut self,
        declaration: &RelationDeclaration,
        proof: &ProofAlternativeId,
        constructibility: &crate::constructibility::CompilerConstructibilityAnalysis,
        choices: &[crate::lifecycle::RepresentationChoice],
    ) -> core::Alternative {
        let id = core::AlternativeKey {
            relation: super::relation_key(&declaration.id),
            namespace: super::ATTESTATION,
            kind: proof.proof() as u16,
        };
        self.proofs.insert(id.clone(), proof.clone());
        let rows = derive_source_requirements(declaration, proof.proof());
        let constructible = crate::proof::locally_feasible(
            declaration,
            proof,
            &crate::capability::CapabilityView::Unconstrained,
            constructibility,
        )
        .is_ok();
        let sources = rows
            .map(|rows| {
                let table = super::source::table(declaration, proof.proof(), &rows);
                for (key, row) in table.rows.iter().zip(rows) {
                    self.sources.insert(key.clone(), row);
                }
                table.rows
            })
            .map_err(|_| LocalProofRejection::SourceDerivation);
        let evidence = crate::source::proof_external_evidence(declaration, proof.proof())
            .into_iter()
            .map(|requirement| {
                let key = evidence_key(&requirement);
                self.evidence.insert(key.clone(), requirement);
                key
            })
            .collect();
        let incompatible = choices
            .iter()
            .flat_map(|choice| {
                choice.candidates.iter().filter_map(|mode| {
                    let touches = match &declaration.relation {
                        Relation::AmountConservation {
                            input_objects,
                            output_objects,
                            ..
                        } => {
                            input_objects.contains(&choice.id.object)
                                || output_objects.contains(&choice.id.object)
                        }
                        _ => false,
                    };
                    (declaration.id.operation() == choice.id.operation
                        && touches
                        && !crate::lifecycle::proof_supports_representation(proof.proof(), *mode))
                    .then_some((choice_key(&choice.id), mode_key(*mode)))
                })
            })
            .collect();
        core::Alternative {
            id,
            capabilities: proof_capabilities(declaration, proof.proof()),
            sources,
            constructible,
            evidence,
            incompatible,
        }
    }

    pub fn enumerate(
        &self,
        view: &crate::capability::CapabilityView,
        limits: crate::input::ProofSearchLimits,
        report: &mut crate::proof::ProofSearchReport,
    ) -> Result<Vec<ProofPlanCandidate>, CompileError> {
        let result =
            core::enumerate(&self.records, view, limits, report).map_err(|error| match error {
                core::Error::NoFeasiblePlan(relations) => CompileError::NoFeasibleProofPlan {
                    blocked_relations: relations
                        .iter()
                        .map(|key| self.relations[key].clone())
                        .collect(),
                },
                core::Error::StateLimit(maximum) => {
                    CompileError::ProofSearchStateLimitExceeded { maximum }
                }
                core::Error::CandidateLimit(maximum) => {
                    CompileError::ProofCandidateLimitExceeded { maximum }
                }
                core::Error::FixedSource(_) => CompileError::InvalidRealization,
            })?;
        let mut candidates = result
            .into_iter()
            .map(|candidate| self.candidate(candidate))
            .collect::<Vec<_>>();
        candidates.sort();
        candidates.dedup();
        Ok(candidates)
    }

    pub fn candidate(&self, candidate: core::Candidate) -> ProofPlanCandidate {
        let mut source_requirements = candidate
            .sources
            .iter()
            .map(|row| self.sources[row].clone())
            .collect::<Vec<_>>();
        source_requirements.sort();
        source_requirements.dedup();
        ProofPlanCandidate {
            proofs: candidate.proofs.into_iter().map(|(relation, alternative)| (self.relations[&relation].clone(), self.proofs[&alternative].clone())).collect(),
            representations: candidate.representations.into_iter().map(|(choice, mode)| (self.choices[&choice].clone(), self.modes[&mode])).collect(),
            required_capabilities: candidate.capabilities, source_requirements,
            external_evidence: candidate.evidence.iter().map(|key| self.evidence[key].clone()).collect(),
            lifecycle: candidate.lifecycle.iter().map(|row| self.lifecycle[row].clone()).collect(),
            disclosure: crate::disclosure::CompilerDisclosureAnalysis {
                inherited_required_public: self.inherited.clone(),
                added_required_public: candidate.disclosed.into_iter().map(|(fact, reasons)| (self.facts[&fact].clone(), reasons.into_iter().map(|(choice, mode)| { let id = &self.choices[&choice]; crate::disclosure::CompilerDisclosureReason::RepresentationSelection { operation: id.operation, object: id.object, representation: self.modes[&mode] } }).collect())).collect(),
                retained_private: candidate.retained.iter().map(|key| self.facts[key].clone()).collect(),
            },
        }
    }
}

fn disclosure_records(
    inherited: &realization::DeclassificationAnalysis,
    choices: &[crate::lifecycle::RepresentationChoice],
) -> core::DisclosureRecords {
    let valid = !inherited
        .required_public
        .keys()
        .chain(inherited.newly_disclosed.keys())
        .chain(inherited.retained_private.iter())
        .any(crate::disclosure::is_sponsor_amount);
    let mut public_effects = BTreeMap::new();
    for choice in choices {
        for mode in &choice.candidates {
            if matches!(
                mode,
                RepresentationMode::Explicit | RepresentationMode::PublicCommitted
            ) {
                let affected = inherited.retained_private.iter().filter(|fact| matches!(fact, realization::FactId::FamilyAmount { operation, object, .. } if *operation == choice.id.operation && *object == choice.id.object)).map(fact_key).collect();
                public_effects.insert((choice_key(&choice.id), mode_key(*mode)), affected);
            }
        }
    }
    core::DisclosureRecords {
        valid,
        retained: inherited.retained_private.iter().map(fact_key).collect(),
        public_effects,
    }
}

pub fn derive_disclosure(
    inherited: &realization::DeclassificationAnalysis,
    representations: &BTreeMap<crate::lifecycle::RepresentationChoiceId, RepresentationMode>,
) -> Result<crate::disclosure::CompilerDisclosureAnalysis, CompileError> {
    let choices = representations
        .iter()
        .map(|(id, mode)| crate::lifecycle::RepresentationChoice {
            id: id.clone(),
            candidates: vec![*mode],
        })
        .collect::<Vec<_>>();
    let records = disclosure_records(inherited, &choices);
    let selection = representations
        .iter()
        .map(|(choice, mode)| (choice_key(choice), mode_key(*mode)))
        .collect();
    let result = core::disclose(&records, &selection).ok_or(CompileError::SponsorValueRead)?;
    let facts = inherited
        .retained_private
        .iter()
        .map(|fact| (fact_key(fact), fact))
        .collect::<BTreeMap<_, _>>();
    let reasons = representations
        .iter()
        .map(|(choice, mode)| {
            (
                (choice_key(choice), mode_key(*mode)),
                crate::disclosure::CompilerDisclosureReason::RepresentationSelection {
                    operation: choice.operation,
                    object: choice.object,
                    representation: *mode,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    Ok(crate::disclosure::CompilerDisclosureAnalysis {
        inherited_required_public: inherited.required_public.clone(),
        added_required_public: result
            .disclosed
            .into_iter()
            .map(|(fact, keys)| {
                (
                    facts[&fact].clone(),
                    keys.iter().map(|key| reasons[key].clone()).collect(),
                )
            })
            .collect(),
        retained_private: result
            .retained
            .iter()
            .map(|key| facts[key].clone())
            .collect(),
    })
}
fn fact_key(fact: &realization::FactId) -> core::FactKey {
    core::FactKey {
        namespace: super::ATTESTATION,
        subject: super::constructibility::fact_key(fact),
    }
}
const fn object_key(object: architecture::ObjectId) -> crate::engine_lifecycle::ObjectKey {
    crate::engine_lifecycle::ObjectKey {
        namespace: super::ATTESTATION,
        code: object.code(),
    }
}
pub const fn mode_key(mode: RepresentationMode) -> crate::engine_lifecycle::ModeKey {
    crate::engine_lifecycle::ModeKey {
        namespace: super::ATTESTATION,
        code: mode as u16,
    }
}
pub const fn choice_key(choice: &crate::lifecycle::RepresentationChoiceId) -> core::ChoiceKey {
    core::ChoiceKey {
        operation: super::operation_key(choice.operation),
        object: object_key(choice.object),
    }
}
pub const fn lifecycle_key(
    row: &crate::lifecycle::LifecycleRequirement,
) -> crate::engine_lifecycle::LifecycleRequirementRecord {
    crate::engine_lifecycle::LifecycleRequirementRecord {
        object: object_key(row.object),
        representation: mode_key(row.representation),
        exit: super::operation_key(row.exit),
        status: row.status,
    }
}
pub const fn evidence_key(
    requirement: &realization::ExternalEvidenceRequirement,
) -> crate::engine_source::Evidence {
    use crate::engine_source::Evidence as E;
    match requirement {
        realization::ExternalEvidenceRequirement::OperatorAuthorization { operation } => {
            E::Operator {
                operation: super::operation_key(*operation),
            }
        }
        realization::ExternalEvidenceRequirement::ConfidentialValueConservation {
            operation,
            asset,
        } => E::ConfidentialValue {
            operation: super::operation_key(*operation),
            asset: asset.code(),
        },
        realization::ExternalEvidenceRequirement::SubstrateConservation { operation, asset } => {
            E::SubstrateValue {
                operation: super::operation_key(*operation),
                asset: asset.code(),
            }
        }
    }
}
