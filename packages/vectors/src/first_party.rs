//! What discharges a negative requirement no target can answer.
//!
//! Twenty of the plan's seventy-four negative requirements have first-party
//! boundaries, with eighteen observed comparisons and two formal dispositions.
//! Their boundary is the compiler's own analysis or the emitted structure
//! and no transaction exists at either. §19.2's conditions — a complete mutated
//! target transaction, an executed carrier, an observed target rejection
//! — cannot be met by any of them; explicit closure uses their own boundaries.
//!
//! # The policy, and why it needs six things rather than one
//!
//! §4.2 states the condition explicitly, and every clause of it removes
//! a way of appearing to have evidence. A canonical malformed input, so
//! the subject is a focused change of something the workspace really
//! publishes rather than an invention. The *exact owning* validator, so
//! a refusal from a neighbouring layer cannot stand in — which is the
//! trap the archaeology found, where every test asserting a compiler
//! error asserted the realization twin instead. A typed refusal naming
//! the intended class, so the requirement's own semantic mutation is
//! what came back and not merely some failure. A focused positive
//! control, so the refusal is attributable to the malformation and not
//! to the input being unacceptable all along. And a validated report
//! whose constructor is private, so a caller cannot state the conclusion
//! without passing through the two runs that establish it.
//!
//! # The same argument the negative half already makes
//!
//! This is [`crate::mutation`]'s un-mutated sibling, moved to a boundary
//! that has no bytes. There, a refusal means something because the
//! accepted transaction and the mutated one differ by exactly one thing.
//! Here, a compiler refusal means something because the published
//! dependency and the malformed one differ by exactly one thing, and the
//! published one is accepted by the same validator in the same call.

use std::collections::BTreeSet;

use architecture::{ARCHITECTURE, ObjectId, OperationId};
use compiler::operation_plan::{
    CoverageRequirementId, EvidenceRole, RelationMutation, TargetCoverageObligation,
    ValidatedTargetOperationPlan,
};
use compiler::{CompileError, validate_required_dependency};
use realization::{AvailabilityClass, ConstructibilityNodeId, RealizationScope, derive};

#[cfg(test)]
mod closure_destinations {
    use std::fmt::Write;
    use tapscript::{TapscriptInstruction as I, TapscriptProgram};
    use target_elements::OpcodeId as O;
    use transaction::bytes::{
        AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
        TargetTransaction, Txid, ValueField,
    };

    // This evaluator checks the linked instruction stream against serialized output
    // bytes. Authorization is a stated successful premise, never native evidence.
    #[derive(Clone)]
    struct Subject {
        protocol: [u8; 32],
        reserve: [u8; 32],
        change: Vec<u8>,
        private: bool,
        live: bool,
        protocol_inputs: u8,
        protocol_program: Vec<u8>,
    }

    impl Subject {
        fn outputs(&self) -> Vec<TargetOutput> {
            let value = if self.private {
                let mut commitment = [0x44; 33];
                commitment[0] = 8;
                ValueField::Commitment(commitment)
            } else {
                ValueField::Explicit(10 * u64::from(self.protocol_inputs))
            };
            [
                (self.protocol, value, self.protocol_program.clone()),
                (self.reserve, ValueField::Explicit(5), self.change.clone()),
                (self.reserve, ValueField::Explicit(1), Vec::new()),
            ]
            .into_iter()
            .map(|(asset, value, program)| {
                TargetOutput::new(
                    AssetField::Explicit(AssetId::from_internal(asset)),
                    value,
                    NonceField::Null,
                    program,
                )
            })
            .collect()
        }

        fn bytes(&self, outputs: Vec<TargetOutput>) -> Vec<u8> {
            let count = usize::from(self.protocol_inputs) + 1;
            let inputs = (0..=u32::from(self.protocol_inputs))
                .map(|index| {
                    TargetInput::new(
                        Outpoint::new(Txid::from_internal([0x22; 32]), index).expect("index"),
                        u32::MAX,
                    )
                })
                .collect();
            TargetTransaction::new(
                2,
                inputs,
                outputs,
                0,
                vec![InputWitness::new(Vec::new()); count],
            )
            .expect("concrete transaction")
            .encode()
        }

        #[allow(clippy::too_many_lines)]
        fn accepts(&self, program: &TapscriptProgram, bytes: &[u8]) -> bool {
            let tx = TargetTransaction::decode(bytes).expect("serialized subject");
            let mut stack: Vec<Vec<u8>> = if self.live {
                vec![vec![0x55; 64]]
            } else {
                Vec::new()
            };
            let pop = |stack: &mut Vec<Vec<u8>>| stack.pop().expect("linked stack discipline");
            let number = |bytes: &[u8]| {
                bytes
                    .iter()
                    .enumerate()
                    .fold(0_usize, |n, (i, b)| n | (usize::from(*b) << (8 * i)))
            };
            for instruction in program.instructions() {
                let opcode = match instruction {
                    I::Push(item) => {
                        stack.push(item.bytes().to_vec());
                        continue;
                    }
                    I::Opcode(opcode) => *opcode,
                };
                match opcode {
                    O::PushCurrentInputIndex => stack.push(Vec::new()),
                    O::InspectNumInputs => {
                        stack.push(vec![u8::try_from(tx.inputs().len()).expect("small census")]);
                    }
                    O::InspectNumOutputs => {
                        stack.push(vec![
                            u8::try_from(tx.outputs().len()).expect("small census"),
                        ]);
                    }
                    O::InspectInputIssuance => {
                        pop(&mut stack);
                        stack.push(Vec::new());
                    }
                    O::InspectInputAsset | O::InspectOutputAsset => {
                        let index = number(&pop(&mut stack));
                        let asset = if opcode == O::InspectInputAsset {
                            if index < usize::from(self.protocol_inputs) {
                                self.protocol
                            } else {
                                self.reserve
                            }
                        } else {
                            let Some(output) = tx.outputs().get(index) else {
                                return false;
                            };
                            let AssetField::Explicit(asset) = output.asset() else {
                                return false;
                            };
                            *asset.internal()
                        };
                        stack.push(asset.to_vec());
                        stack.push(vec![1]);
                    }
                    O::InspectInputValue | O::InspectOutputValue => {
                        let index = number(&pop(&mut stack));
                        let value = if opcode == O::InspectInputValue {
                            if self.private && index < usize::from(self.protocol_inputs) {
                                self.outputs()[0].value()
                            } else {
                                ValueField::Explicit(10)
                            }
                        } else {
                            let Some(output) = tx.outputs().get(index) else {
                                return false;
                            };
                            output.value()
                        };
                        match value {
                            ValueField::Explicit(value) => {
                                stack.push(value.to_le_bytes().to_vec());
                                stack.push(vec![1]);
                            }
                            ValueField::Commitment(value) => {
                                stack.push(value[1..].to_vec());
                                stack.push(vec![value[0]]);
                            }
                            _ => panic!("unsupported fixture value"),
                        }
                    }
                    O::InspectInputScriptPubKey | O::InspectOutputScriptPubKey => {
                        let index = number(&pop(&mut stack));
                        let script = if opcode == O::InspectInputScriptPubKey {
                            self.outputs()[0].program().to_vec()
                        } else {
                            let Some(output) = tx.outputs().get(index) else {
                                return false;
                            };
                            output.program().to_vec()
                        };
                        if script.is_empty() {
                            stack.push(crate::bundle::fee_program_digest().to_vec());
                            stack.push(vec![0x81]);
                        } else {
                            assert_eq!(usize::from(script[1]), script.len() - 2);
                            stack.push(script[2..].to_vec());
                            stack.push(if script[0] == 0 {
                                Vec::new()
                            } else {
                                vec![script[0] - 0x50]
                            });
                        }
                    }
                    O::EqualVerify => {
                        let right = pop(&mut stack);
                        if pop(&mut stack) != right {
                            return false;
                        }
                    }
                    O::Verify => {
                        if pop(&mut stack).iter().all(|b| *b == 0) {
                            return false;
                        }
                    }
                    O::Drop => {
                        pop(&mut stack);
                    }
                    O::Duplicate => {
                        stack.push(stack.last().expect("operand").clone());
                    }
                    O::Rotate => {
                        let value = stack.remove(stack.len() - 3);
                        stack.push(value);
                    }
                    O::Substring => {
                        let size = number(&pop(&mut stack));
                        let start = number(&pop(&mut stack));
                        let value = pop(&mut stack);
                        stack.push(value[start..start + size].to_vec());
                    }
                    O::GreaterThan64 | O::LessThan64 => {
                        let right =
                            i64::from_le_bytes(pop(&mut stack).try_into().expect("wide operand"));
                        let left =
                            i64::from_le_bytes(pop(&mut stack).try_into().expect("wide operand"));
                        stack.push(vec![u8::from(if opcode == O::GreaterThan64 {
                            left > right
                        } else {
                            left < right
                        })]);
                    }
                    O::Add64 => {
                        let right = i64::from_le_bytes(pop(&mut stack).try_into().expect("wide"));
                        let left = i64::from_le_bytes(pop(&mut stack).try_into().expect("wide"));
                        stack.push(
                            left.checked_add(right)
                                .expect("small amounts")
                                .to_le_bytes()
                                .to_vec(),
                        );
                        stack.push(vec![1]);
                    }
                    O::BitwiseAnd => {
                        let right = pop(&mut stack);
                        let left = pop(&mut stack);
                        stack.push(left.iter().zip(right).map(|(a, b)| a & b).collect());
                    }
                    O::CheckSigVerify => {
                        assert_eq!(pop(&mut stack).len(), 32);
                        assert_eq!(pop(&mut stack), vec![0x55; 64]);
                    }
                    other => panic!("unimplemented linked instruction {other:?}"),
                }
            }
            stack == vec![vec![1]]
        }
    }

    fn pair(program: &TapscriptProgram, subject: &Subject) {
        let control = subject.outputs();
        assert!(subject.accepts(program, &subject.bytes(control.clone())));
        for change in [vec![0, 32], vec![0x51, 32]] {
            let alternate = Subject {
                change: [change, vec![0x66; 32]].concat(),
                ..subject.clone()
            };
            // Before the removal, both an alternate payload and an ordinary taproot
            // output class are refused by the linked destination equality.
            assert!(alternate.accepts(program, &alternate.bytes(alternate.outputs())));
        }
        let mut wrong_asset = control.clone();
        wrong_asset[1] = TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal(subject.protocol)),
            ValueField::Explicit(5),
            NonceField::Null,
            subject.change.clone(),
        );
        assert!(!subject.accepts(program, &subject.bytes(wrong_asset)));
        let mut missing = control.clone();
        missing.remove(1);
        assert!(!subject.accepts(program, &subject.bytes(missing)));
        let mut extra = control.clone();
        extra.push(control[1].clone());
        assert!(!subject.accepts(program, &subject.bytes(extra)));
    }

    #[test]
    fn compact_linked_change_destination_pair() {
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let (leaf, program) = bundle.linked().programs().iter().find(|(leaf, _)| {
            matches!(leaf, tapscript::LeafRole::Coordinator { shape } if shape.ash_inputs() == 2 && shape.sponsor_change() == tapscript::SponsorChangePresence::Present)
        }).expect("fixed-batch change coordinator");
        let _ = leaf;
        let symbols = bundle.abi().symbols();
        pair(
            program.program(),
            &Subject {
                protocol: bundle.closed_asset(),
                reserve: bundle.reserve_asset(),
                private: false,
                live: false,
                protocol_inputs: 2,
                protocol_program: bundle
                    .pin()
                    .output_script(bundle.target())
                    .expect("constructor"),
                change: transaction::taproot::witness_program_script(
                    bundle.target(),
                    symbols.sponsor_change_version(),
                    symbols.sponsor_change_program(),
                )
                .expect("default"),
            },
        );
    }

    #[test]
    fn live_linked_change_destination_pairs() {
        use crate::live_plan::{PROTOCOL_ASSET, RESERVE_ASSET};
        let target = target_elements::reviewed_elements_tapscript().expect("target");
        let fee = crate::bundle::fee_program_digest();
        let linked =
            crate::live_plan::link_live_bundle_for_asset(PROTOCOL_ASSET, RESERVE_ASSET, fee)
                .expect("bundle");
        let abi = crate::live_plan::live_abi_for_asset(PROTOCOL_ASSET, RESERVE_ASSET, fee)
            .expect("linked constructors");
        for constructor in linked.constructors().values() {
            let program = constructor
                .programs()
                .iter()
                .find_map(|(leaf, program)| match leaf {
                    tapscript::LiveTransferLeafRole::Coordinator { shape, .. }
                        if shape.receipt_inputs() == 1
                            && shape.receipt_outputs() == 1
                            && shape.sponsor_change()
                                == tapscript::SponsorChangePresence::Present =>
                    {
                        Some(program)
                    }
                    _ => None,
                })
                .expect("one-to-one change coordinator");
            let deployment =
                crate::live_plan::live_deployment_for_asset(PROTOCOL_ASSET, RESERVE_ASSET, fee)
                    .expect("deployment");
            let symbols = deployment.resolved();
            pair(
                program.program(),
                &Subject {
                    protocol: PROTOCOL_ASSET,
                    reserve: RESERVE_ASSET,
                    private: constructor.representation()
                        == compiler::live_transfer_plan::LiveTransferRepresentationPlan::PrivateCommitted,
                    live: true,
                    protocol_inputs: 1,
                    protocol_program: abi
                        .destinations()
                        .get(constructor.owner(), constructor.representation())
                        .expect("destination")
                        .instance()
                        .program()
                        .to_vec(),
                    change: transaction::taproot::witness_program_script(
                        &target,
                        u8::try_from(symbols.sponsor_change_version()).expect("version"),
                        symbols.sponsor_change_program().bytes(),
                    )
                    .expect("default"),
                },
            );
        }
    }

    #[test]
    fn closure_leaf_bytes_and_roots_match_the_fixture() {
        for bundle in crate::live_plan::relocatable_live_bundles().expect("bundles") {
            // Before destination elision: 42 symbols and 218 relocation records.
            assert_eq!(
                (bundle.symbols().len(), bundle.relocations().len()),
                (40, 200)
            );
        }
        let hex = |bytes: &[u8]| {
            let mut text = String::new();
            for byte in bytes {
                write!(text, "{byte:02x}").expect("string write");
            }
            text
        };
        let compact = crate::bundle::fixture_bundle().expect("compact");
        let tree = compact.abi().tree();
        let mut snapshot = format!("compact root {}\n", hex(&tree.merkle_root()));
        for (leaf, bytes) in tree.leaf_programs() {
            writeln!(snapshot, "compact {leaf:?} {}", hex(bytes)).expect("string write");
        }
        let live = crate::live_plan::demonstration_live_bundle().expect("live");
        for (key, constructor) in live.constructors() {
            let tree = transaction::live_taproot::commit_live_tree(compact.target(), constructor)
                .expect("live tree");
            writeln!(snapshot, "live {key:?} root {}", hex(&tree.merkle_root()))
                .expect("string write");
            for (leaf, bytes) in tree.leaf_programs() {
                writeln!(snapshot, "live {key:?} {leaf:?} {}", hex(bytes)).expect("string write");
            }
        }
        assert_eq!(
            snapshot,
            include_str!("../fixtures/closure-compact-live.golden")
        );
    }
}

/// Which first-party validator owns one negative requirement.
///
/// A name for the exact entry point a discharge drove, carried into the
/// evidence so a reader of a coverage row can see *what* refused. There
/// is deliberately no arm for "the compiler", because the compiler
/// refuses many things and a row answered by one of them is not answered
/// by the others.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FirstPartyValidator {
    /// The compiler's check that an operation's authorization case
    /// discharges the availability class of a required dependency.
    ///
    /// `compiler::validate_required_dependency`, which is the sole site
    /// constructing either constructibility refusal and is the same call
    /// the scoped analysis makes for every required ancestor.
    CompilerRequiredDependency,
}

/// One first-party refusal, as a coverage row records it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FirstPartyRefusal {
    validator: FirstPartyValidator,
    class: RelationMutation,
}

impl FirstPartyRefusal {
    /// Which validator refused.
    #[must_use]
    pub const fn validator(self) -> FirstPartyValidator {
        self.validator
    }

    /// The semantic mutation class its refusal named.
    #[must_use]
    pub const fn class(self) -> RelationMutation {
        self.class
    }

    /// The observed compiler refusal class, without subject data.
    const fn tracing_observed_kind(self) -> &'static str {
        match self.class {
            RelationMutation::CardinalityBelowMinimum => "cardinality_below_minimum",
            RelationMutation::CardinalityAboveMaximum { .. } => "cardinality_above_maximum",
            RelationMutation::UndeclaredObjectFamily => "undeclared_object_family",
            RelationMutation::WrongRecognizedAsset => "wrong_recognized_asset",
            RelationMutation::WrongRecognizedObject => "wrong_recognized_object",
            RelationMutation::AmountMismatch => "amount_mismatch",
            RelationMutation::MissingRequiredOwner => "missing_required_owner",
            RelationMutation::UnexpectedProtocolSecret => "unexpected_protocol_secret",
            RelationMutation::SponsorProtocolOverlap => "sponsor_protocol_overlap",
            RelationMutation::SponsorMemberUnclaimed => "sponsor_member_unclaimed",
            RelationMutation::MissingSponsorAuthorization => "missing_sponsor_authorization",
            RelationMutation::SponsorEnvelopeMultiplicityExceeded => {
                "sponsor_envelope_multiplicity_exceeded"
            }
            RelationMutation::WrongRootEffect => "wrong_root_effect",
            RelationMutation::MissingRequiredProjection => "missing_required_projection",
            RelationMutation::ForbiddenProjectionPresent => "forbidden_projection_present",
            RelationMutation::MissingCanonicalDeltaFamily => "missing_canonical_delta_family",
            RelationMutation::UnexpectedCanonicalDeltaFamily => "unexpected_canonical_delta_family",
            RelationMutation::DuplicateCanonicalSourceOrDestination => {
                "duplicate_canonical_source_or_destination"
            }
            RelationMutation::UndeclaredOpenFlow => "undeclared_open_flow",
            RelationMutation::ConstructibilityWitnessUnavailable => {
                "constructibility_witness_unavailable"
            }
            RelationMutation::PermissionlessPrivateDependency => {
                "permissionless_private_dependency"
            }
            RelationMutation::UnsupportedRepresentation => "unsupported_representation",
            RelationMutation::UnauthenticatedRepresentation => "unauthenticated_representation",
            RelationMutation::RequiredLifecycleExitMissing => "required_lifecycle_exit_missing",
            RelationMutation::ExpressionPredicateFalse => "expression_predicate_false",
            RelationMutation::ExternalEvidenceMissing => "external_evidence_missing",
            RelationMutation::ExternalEvidenceFailed => "external_evidence_failed",
            RelationMutation::ExternalEvidenceIdentityMismatch => {
                "external_evidence_identity_mismatch"
            }
        }
    }
}

/// One canonical malformed typed input, and its control.
///
/// The malformation is a single field, and the control is the same value
/// without it. Stating the pair as one value rather than two is what
/// keeps a caller from offering a control that is not the malformed
/// input's sibling.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FirstPartyNegativeCase {
    /// A required dependency whose availability class was changed from
    /// the one the realization publishes to a private one.
    ///
    /// The published node is the control. The malformed input is that
    /// node with one field replaced, and the authorization offered to
    /// both is the operation's own — looked up rather than authored, so
    /// the case cannot be made easier by choosing a weaker one.
    PrivateRequiredDependency {
        /// The operation whose authorization case is offered.
        operation: OperationId,
        /// The node exactly as the realization publishes it.
        published: ConstructibilityNodeId,
        /// The one changed field.
        availability: AvailabilityClass,
    },
}

/// The canonical compact-ASH private-dependency case.
///
/// Both the validator tests and the plan derivation use this one case:
/// the published compact-ASH public required fact, changed to an
/// ASH-owner-private availability class.
pub(crate) fn compact_ash_permissionless_private_dependency_case()
-> Result<FirstPartyNegativeCase, FirstPartyEvidenceRefusal> {
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .map_err(|_| FirstPartyEvidenceRefusal::RealizationUnavailable)?;
    let published = realization
        .project()
        .constructibility
        .nodes
        .into_iter()
        .map(|node| node.id)
        .find(|id| {
            matches!(
                id,
                ConstructibilityNodeId::Fact {
                    operation: OperationId::CompactAsh,
                    availability: AvailabilityClass::Public,
                    ..
                }
            )
        })
        .ok_or(FirstPartyEvidenceRefusal::ControlIsNotPublished)?;
    Ok(FirstPartyNegativeCase::PrivateRequiredDependency {
        operation: OperationId::CompactAsh,
        published,
        availability: AvailabilityClass::InputOwners {
            object: ObjectId::Ash,
        },
    })
}

/// Why one offered case does not discharge its requirement.
///
/// Every arm is a way the offered evidence falls short of §4.2, and none
/// of them is a defect in the validator: a validator that refused the
/// control has said something true about the control, and what fails is
/// the claim that the refusal was about the malformation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FirstPartyEvidenceRefusal {
    /// The plan publishes no such requirement.
    RequirementNotPublished,
    /// The requirement is not one first-party code answers.
    ///
    /// A runtime or external row reaching here would be answered by a
    /// target or by a report, and a first-party refusal is not either.
    NotAFirstPartyRequirement,
    /// A staged authorization case belongs to another operation.
    CaseNamesAnotherOperation,
    /// The realization the malformed input is checked against could not
    /// be derived.
    RealizationUnavailable,
    /// The control is not a node the realization publishes.
    ///
    /// Then the pair is two inventions rather than a published value and
    /// a focused change of it, and a refusal of the second says nothing
    /// about the workspace.
    ControlIsNotPublished,
    /// The control carries no availability class to change.
    ///
    /// An operation node has no such field, so there is no single change
    /// to make to it and no pair to compare.
    ControlHasNoAvailabilityClass,
    /// The operation publishes several authorization cases.
    ///
    /// Then which one the malformed input is offered to decides what
    /// comes back, and a case that did not say would be answered by
    /// whichever the lookup reached first. Refused rather than resolved,
    /// for the reason every underdetermined link in this crate is.
    AuthorizationCaseUnderdetermined,
    /// The malformation left the input unchanged.
    ///
    /// The whole argument is that the two inputs differ by exactly one
    /// thing; two identical inputs differ by nothing, and the validator
    /// would have to answer them the same way.
    MalformationChangedNothing,
    /// The validator accepted the malformed input.
    MalformedInputWasAccepted,
    /// The validator refused, naming a class other than the
    /// requirement's.
    RefusalNamesAnotherClass(RelationMutation),
    /// The validator refused, naming no semantic mutation class at all.
    ///
    /// The guard rather than an observed state: the validator raises two
    /// errors and both name a class. It is here because the mapping
    /// returns an option, and a validator that grew a third error would
    /// otherwise have its refusal filed under whichever class the match
    /// reached first.
    RefusalNamesNoClass,
    /// The validator refused the control too.
    ///
    /// Then the refusal is not attributable to the malformation: the
    /// input was unacceptable before it was malformed.
    ControlWasRefused,
}

impl FirstPartyEvidenceRefusal {
    const fn tracing_refusal_kind(&self) -> &'static str {
        match self {
            Self::RequirementNotPublished => "requirement_not_published",
            Self::NotAFirstPartyRequirement => "not_a_first_party_requirement",
            Self::CaseNamesAnotherOperation => "case_names_another_operation",
            Self::RealizationUnavailable => "realization_unavailable",
            Self::ControlIsNotPublished => "control_is_not_published",
            Self::ControlHasNoAvailabilityClass => "control_has_no_availability_class",
            Self::AuthorizationCaseUnderdetermined => "authorization_case_underdetermined",
            Self::MalformationChangedNothing => "malformation_changed_nothing",
            Self::MalformedInputWasAccepted => "malformed_input_was_accepted",
            Self::RefusalNamesAnotherClass(_) => "refusal_names_another_class",
            Self::RefusalNamesNoClass => "refusal_names_no_class",
            Self::ControlWasRefused => "control_was_refused",
        }
    }
}

/// One first-party negative requirement, discharged.
///
/// # What holding one of these establishes
///
/// That the exact validator the requirement's boundary belongs to was
/// run twice — once on a canonical malformed input and once on the
/// published value that input is a single change of — that it refused
/// the first naming the requirement's own semantic mutation class, and
/// that it accepted the second. All of that is recomputed by
/// [`validate_first_party_negative`], which is the only constructor;
/// every field here is a conclusion of that run and none of them is a
/// value a caller offered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedFirstPartyNegativeEvidence {
    requirement: CoverageRequirementId,
    refusal: FirstPartyRefusal,
    control: ConstructibilityNodeId,
    malformed: ConstructibilityNodeId,
}

impl ValidatedFirstPartyNegativeEvidence {
    /// The requirement this evidence answers.
    #[must_use]
    pub const fn requirement(&self) -> &CoverageRequirementId {
        &self.requirement
    }

    /// What refused, and what class it named.
    #[must_use]
    pub const fn refusal(&self) -> FirstPartyRefusal {
        self.refusal
    }

    /// The published value the validator accepted.
    #[must_use]
    pub const fn control(&self) -> &ConstructibilityNodeId {
        &self.control
    }

    /// The malformed value it refused.
    #[must_use]
    pub const fn malformed(&self) -> &ConstructibilityNodeId {
        &self.malformed
    }
}

/// Discharge one first-party negative requirement, per §4.2.
///
/// # Errors
///
/// [`FirstPartyEvidenceRefusal`], naming which of §4.2's conditions the
/// offered case does not meet.
pub fn validate_first_party_negative(
    plan: &ValidatedTargetOperationPlan,
    requirement: &CoverageRequirementId,
    case: &FirstPartyNegativeCase,
) -> Result<ValidatedFirstPartyNegativeEvidence, FirstPartyEvidenceRefusal> {
    let span = tracing::debug_span!(
        target: "vectors::validation",
        "validate_mutant",
        family = "compact",
        boundary = "compiler_required_dependency"
    );
    let _entered = span.enter();
    let result = validate_first_party_negative_core(plan, requirement, case);
    match &result {
        Ok(evidence) => tracing::debug!(
            target: "vectors::validation",
            family = "compact",
            boundary = "compiler_required_dependency",
            outcome = "completed",
            standing = "refused",
            observed_refusal_kind = evidence.refusal().tracing_observed_kind(),
            "mutant validated"
        ),
        Err(refusal) => tracing::debug!(
            target: "vectors::validation",
            family = "compact",
            boundary = "compiler_required_dependency",
            outcome = "refused",
            refusal_kind = refusal.tracing_refusal_kind(),
            "mutant validated"
        ),
    }
    result
}

fn validate_first_party_negative_core(
    plan: &ValidatedTargetOperationPlan,
    requirement: &CoverageRequirementId,
    case: &FirstPartyNegativeCase,
) -> Result<ValidatedFirstPartyNegativeEvidence, FirstPartyEvidenceRefusal> {
    let published = plan
        .coverage_requirement(requirement)
        .ok_or(FirstPartyEvidenceRefusal::RequirementNotPublished)?;
    let TargetCoverageObligation::Negative(negative) = &published.obligation else {
        return Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement);
    };
    if !matches!(
        published.role,
        EvidenceRole::CompilerAnalysisResult | EvidenceRole::EmittedStructure
    ) {
        return Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement);
    }

    let FirstPartyNegativeCase::PrivateRequiredDependency {
        operation,
        published: control,
        availability,
    } = case;

    if *operation != requirement.relation.operation() {
        return Err(FirstPartyEvidenceRefusal::CaseNamesAnotherOperation);
    }

    // The control has to be a value the workspace really publishes, or
    // the pair is two inventions rather than a published value and one
    // focused change of it.
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .map_err(|_| FirstPartyEvidenceRefusal::RealizationUnavailable)?;
    let nodes: BTreeSet<ConstructibilityNodeId> = realization
        .project()
        .constructibility
        .nodes
        .into_iter()
        .map(|node| node.id)
        .collect();
    if !nodes.contains(control) {
        return Err(FirstPartyEvidenceRefusal::ControlIsNotPublished);
    }

    let malformed = with_availability(control, *availability)
        .ok_or(FirstPartyEvidenceRefusal::ControlHasNoAvailabilityClass)?;
    if malformed == *control {
        return Err(FirstPartyEvidenceRefusal::MalformationChangedNothing);
    }

    // The authorization is the operation's own, looked up rather than
    // authored: a case offering an authorization the operation does not
    // have would be refusing something the analysis never asks.
    let cases = realization
        .constructibility_authorizations(*operation)
        .map_err(|_| FirstPartyEvidenceRefusal::RealizationUnavailable)?;
    let [authorization] = cases else {
        return Err(FirstPartyEvidenceRefusal::AuthorizationCaseUnderdetermined);
    };

    // The negative half. Nothing below reads a list: the class comes
    // back out of the validator's own error. The dependency walk is
    // empty because this case states a dependency rather than building a
    // graph to reach it through; the walk is diagnostic detail on the
    // error and no part of the class it names.
    let Err(error) = validate_required_dependency(*operation, authorization, &malformed, Vec::new)
    else {
        return Err(FirstPartyEvidenceRefusal::MalformedInputWasAccepted);
    };
    let class = refused_class(&error).ok_or(FirstPartyEvidenceRefusal::RefusalNamesNoClass)?;
    if class != negative.mutation {
        return Err(FirstPartyEvidenceRefusal::RefusalNamesAnotherClass(class));
    }

    // The positive control, through the same call with the same
    // authorization. A validator that refused this one refused the
    // input rather than the malformation.
    if validate_required_dependency(*operation, authorization, control, Vec::new).is_err() {
        return Err(FirstPartyEvidenceRefusal::ControlWasRefused);
    }

    Ok(ValidatedFirstPartyNegativeEvidence {
        requirement: requirement.clone(),
        refusal: FirstPartyRefusal {
            validator: FirstPartyValidator::CompilerRequiredDependency,
            class,
        },
        control: control.clone(),
        malformed,
    })
}

/// The same node under another availability class.
///
/// `None` for an operation node, which carries no availability at all
/// and therefore has no such field to change.
fn with_availability(
    node: &ConstructibilityNodeId,
    availability: AvailabilityClass,
) -> Option<ConstructibilityNodeId> {
    match node {
        ConstructibilityNodeId::Operation(_) => None,
        ConstructibilityNodeId::Fact {
            operation, fact, ..
        } => Some(ConstructibilityNodeId::Fact {
            operation: *operation,
            fact: fact.clone(),
            availability,
        }),
        ConstructibilityNodeId::Witness {
            operation, role, ..
        } => Some(ConstructibilityNodeId::Witness {
            operation: *operation,
            role: *role,
            availability,
        }),
    }
}

/// The semantic mutation class one compiler refusal names.
///
/// `None` where the refusal names none, which is every other compiler
/// error: an error outside this validator's two is a failure of
/// something else and answers no coverage requirement here.
const fn refused_class(error: &CompileError) -> Option<RelationMutation> {
    match error {
        CompileError::PermissionlessPrivateDependency { .. } => {
            Some(RelationMutation::PermissionlessPrivateDependency)
        }
        CompileError::ConstructibilityWitnessUnavailable { .. } => {
            Some(RelationMutation::ConstructibilityWitnessUnavailable)
        }
        _ => None,
    }
}

/// Explicit test staging for the compact dependency-analysis boundary.
/// Retains the published node and only replaces its availability field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactDependencyTestStaging {
    case: FirstPartyNegativeCase,
}

impl CompactDependencyTestStaging {
    /// Prepare the canonical public/private pair without changing compiler input validation.
    ///
    /// # Errors
    /// Returns the canonical subject lookup failure.
    pub fn new() -> Result<Self, FirstPartyEvidenceRefusal> {
        Ok(Self {
            case: compact_ash_permissionless_private_dependency_case()?,
        })
    }

    /// Drive the exact dependency owner with both retained subjects.
    ///
    /// # Errors
    /// Returns the same class, identity and control checks as the evidence validator.
    pub fn validate(
        &self,
        plan: &ValidatedTargetOperationPlan,
        id: &CoverageRequirementId,
    ) -> Result<ValidatedFirstPartyNegativeEvidence, FirstPartyEvidenceRefusal> {
        validate_first_party_negative(plan, id, &self.case)
    }
}

/// Retained comparison subject at an unanswered first-party boundary.
/// Retention does not assert that a malformed sibling can reach that boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompactFirstPartySubject {
    Analysis(Box<ValidatedTargetOperationPlan>),
    Emitted(Box<linker::CandidateLinkedBundle>),
}

/// Explicit closure distinguishes exact observations from checked arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompactFirstPartyDisposition {
    Discharged(Box<ValidatedFirstPartyNegativeEvidence>),
    /// A newly supplied exact comparison, including declaration comparisons.
    Compared(Box<CompactFirstPartyComparisonEvidence>),
    /// Candidate-specific unreachability with its supporting comparison.
    FormallyClosed(Box<CompactFirstPartyFormalEvidence>),
    Retained {
        owner: crate::compact_negative_manifest::OwnerEntryMissing,
        subject: CompactFirstPartySubject,
    },
}

impl CompactFirstPartyDisposition {
    /// Closure includes checked formal arguments without calling them observations.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        !matches!(self, Self::Retained { .. })
    }

    /// Only exact comparisons count as observations.
    #[must_use]
    pub const fn is_observed(&self) -> bool {
        matches!(self, Self::Discharged(_) | Self::Compared(_))
    }
}

/// The claim established by an explicitly admitted compact reading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactClosureKind {
    ObservedRefusal,
    ObservedDeclarationRefusal,
    Formal,
}

/// Exact errors returned by the landed comparison entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompactObservedRefusal {
    Compiler(CompileError),
    Multiplicity(compiler::SponsorEnvelopeMultiplicityError),
    Secret(tapscript::CompactSecretFreePathRefusal),
    Representation(tapscript::CompactEmittedRepresentationRefusal),
    Lifecycle(tapscript::CompactEmittedLifecycleRefusal),
    Substrate(crate::report::CompactSubstrateReportRefusal),
}

/// Checked candidate-specific argument; its supporting comparison is opaque.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompactFormalReason {
    UnavailableWitnessUnderPermissionlessAuthorization {
        operation: OperationId,
        authorization: realization::ConstructibilityAuthorization,
        requested: RelationMutation,
        supporting: Box<CompactFirstPartyComparisonEvidence>,
        ruling: (&'static str, u16),
    },
}

/// Admission failures never stand in for an observed row refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompactClosureRefusal {
    MissingReading(CoverageRequirementId),
    DuplicateReading(CoverageRequirementId),
    SurplusReading(CoverageRequirementId),
    WrongKey(CoverageRequirementId),
    SubjectEqualsControl(CoverageRequirementId),
    PairMismatch(CoverageRequirementId),
    ControlRefused(CoverageRequirementId),
    SubjectAccepted(CoverageRequirementId),
    WrongRefusal(CoverageRequirementId),
    /// Any kind or reason disagreement with the canonical reading.
    FormalPremiseMismatch(CoverageRequirementId),
    EvidenceBindingMismatch(CoverageRequirementId),
    /// Failure to construct the retained comparison context.
    Construction(Box<crate::error::FixtureBundleRefusal>),
    /// The manifest could not preserve its keys.
    Manifest(crate::compact_negative_manifest::ManifestRefusal),
    /// A report operand did not reach the comparison boundary.
    Preparation(crate::report::CompactSubstrateReportPreparationRefusal),
    /// Dependency preparation failed before a comparison could be made.
    Dependency(FirstPartyEvidenceRefusal),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EmittedOffer {
    leaves: std::collections::BTreeMap<tapscript::LeafRole, tapscript::TapscriptProgram>,
    declarations: BTreeSet<compiler::operation_plan::LifecycleRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CompactOperand {
    Dependency {
        operation: OperationId,
        authorization: realization::ConstructibilityAuthorization,
        node: ConstructibilityNodeId,
    },
    Selection(realization::RepresentationMode),
    Projection(Box<realization::LifecycleGraphProjection>),
    Envelopes(Box<realization::OperationObservation>),
    Secret(tapscript::TapscriptProgram),
    Recognition(tapscript::TapscriptProgram),
    Emitted(Box<EmittedOffer>),
    Report(Option<Box<crate::report::CompactSubstrateReport>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompactRecord {
    requirement: CoverageRequirementId,
    control: CompactOperand,
    subject: CompactOperand,
    refusal: CompactObservedRefusal,
    kind: CompactClosureKind,
    reason: Option<CompactFormalReason>,
}

/// An exact accepted-control/refused-subject comparison, constructed privately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactFirstPartyComparisonEvidence {
    record: CompactRecord,
}

impl CompactFirstPartyComparisonEvidence {
    /// Observed refusal or observed declaration refusal, kept distinct.
    #[must_use]
    pub const fn kind(&self) -> CompactClosureKind {
        self.record.kind
    }

    /// Complete requirement answered by this comparison.
    #[must_use]
    pub const fn requirement(&self) -> &CoverageRequirementId {
        &self.record.requirement
    }

    /// Exact returned refusal, without a semantic-class relabelling.
    #[must_use]
    pub const fn refusal(&self) -> &CompactObservedRefusal {
        &self.record.refusal
    }
}

/// A checked argument with supporting evidence, never an observed requested class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactFirstPartyFormalEvidence {
    requirement: CoverageRequirementId,
    reason: CompactFormalReason,
}

impl CompactFirstPartyFormalEvidence {
    /// Complete requirement formally resolved.
    #[must_use]
    pub const fn requirement(&self) -> &CoverageRequirementId {
        &self.requirement
    }

    /// Candidate premise and the exact supporting comparison.
    #[must_use]
    pub const fn reason(&self) -> &CompactFormalReason {
        &self.reason
    }
}

/// A staged reading bound to immutable evidence sources; not yet a complete join.
#[derive(Clone, Debug)]
pub struct CompactClosureReading<'e> {
    requirement: CoverageRequirementId,
    evidence: CompactRecord,
    bundle: &'e crate::bundle::FixtureBundle,
    corpus: &'e crate::compact_corpus::CompactCorpus,
}

impl CompactClosureReading<'_> {
    /// The complete envelope key.
    #[must_use]
    pub const fn requirement(&self) -> &CoverageRequirementId {
        &self.requirement
    }

    /// Observed comparison or formal disposition, kept distinct.
    #[must_use]
    pub const fn kind(&self) -> CompactClosureKind {
        self.evidence.kind
    }

    /// Supporting refusal; formal rows do not credit it as their requested class.
    #[must_use]
    pub const fn refusal(&self) -> &CompactObservedRefusal {
        &self.evidence.refusal
    }

    /// Checked formal argument, present only for the unavailable-witness pair.
    #[must_use]
    pub const fn formal_reason(&self) -> Option<&CompactFormalReason> {
        self.evidence.reason.as_ref()
    }
}

/// Complete explicit admission of the nonnative compact negative population.
#[derive(Debug)]
pub struct AdmittedCompactClosure<'e> {
    bundle: &'e crate::bundle::FixtureBundle,
    corpus: &'e crate::compact_corpus::CompactCorpus,
    readings: std::collections::BTreeMap<CoverageRequirementId, CompactClosureReading<'e>>,
}

impl<'e> AdmittedCompactClosure<'e> {
    /// Immutable records, each bound to its complete requirement.
    #[must_use]
    pub const fn records(
        &self,
    ) -> &std::collections::BTreeMap<CoverageRequirementId, CompactClosureReading<'e>> {
        &self.readings
    }

    /// The retained fixture; admission never mutates its plan.
    #[must_use]
    pub const fn bundle(&self) -> &'e crate::bundle::FixtureBundle {
        self.bundle
    }

    /// The retained corpus; admission adds no archive predicate.
    #[must_use]
    pub const fn corpus(&self) -> &'e crate::compact_corpus::CompactCorpus {
        self.corpus
    }
}

struct CompactContext<'a> {
    bundle: &'a crate::bundle::FixtureBundle,
    input: compiler::BoundCompilerInput,
    emitted: tapscript::CandidateRelocatableTapscriptBundle,
}

fn compact_limits() -> compiler::operation_plan::PlacementSearchLimits {
    use std::num::NonZeroU64;
    compiler::operation_plan::PlacementSearchLimits::new(
        NonZeroU64::new(10_000_000).unwrap_or(NonZeroU64::MIN),
        NonZeroU64::new(1_000_000).unwrap_or(NonZeroU64::MIN),
    )
}

fn closure_construction(cause: crate::error::FixtureBundleRefusal) -> CompactClosureRefusal {
    CompactClosureRefusal::Construction(Box::new(cause))
}

fn compact_context(
    bundle: &crate::bundle::FixtureBundle,
) -> Result<CompactContext<'_>, CompactClosureRefusal> {
    use crate::bundle::{INTERNAL_KEY, SPONSOR_CHANGE_PROGRAM, fee_program_digest};
    use crate::error::FixtureBundleRefusal as F;
    use tapscript::CompactAshSymbols;
    let input = crate::bundle::bind_compact_ash_input().map_err(closure_construction)?;
    let target = bundle.target();
    let placeholders = CompactAshSymbols::new(
        target,
        vec![0x11; 32],
        vec![0x22; 32],
        vec![0x44; 20],
        0,
        vec![0x55; 32],
    )
    .map_err(|e| closure_construction(F::Symbols(e)))?;
    let emitted = tapscript::emit_candidate_bundle(
        target,
        bundle.plan(),
        tapscript::demonstration_policy(),
        placeholders,
    )
    .map_err(|e| closure_construction(F::Emission(e)))?;
    let resolved = CompactAshSymbols::new(
        target,
        bundle.closed_asset().to_vec(),
        bundle.reserve_asset().to_vec(),
        SPONSOR_CHANGE_PROGRAM.to_vec(),
        0,
        fee_program_digest().to_vec(),
    )
    .map_err(|e| closure_construction(F::Symbols(e)))?;
    let deployment = linker::LinkDeploymentParameters::new(
        target,
        resolved,
        INTERNAL_KEY.to_vec(),
        linker::SelfCommitmentStrategy::IdentityIntrospection,
        std::num::NonZeroU32::new(8).unwrap_or(std::num::NonZeroU32::MIN),
    )
    .map_err(|e| closure_construction(F::Link(e)))?;
    let linked = linker::link_candidate(target, &emitted, &deployment)
        .map_err(|e| closure_construction(F::Link(e)))?;
    if &linked != bundle.linked() {
        return Err(closure_construction(F::LinkedBundleMismatch));
    }
    Ok(CompactContext {
        bundle,
        input,
        emitted,
    })
}

fn compact_rows(
    bundle: &crate::bundle::FixtureBundle,
) -> Result<Vec<crate::compact_negative_manifest::CompactNegativeRow>, CompactClosureRefusal> {
    use crate::compact_negative_manifest::{NegativeClass, compact_negative_manifest};
    Ok(compact_negative_manifest(bundle)
        .map_err(CompactClosureRefusal::Manifest)?
        .into_values()
        .filter(|row| row.class != NegativeClass::TargetExecutable)
        .collect())
}

fn compact_coordinator(
    context: &CompactContext<'_>,
    id: &CoverageRequirementId,
) -> Result<tapscript::LeafRole, CompactClosureRefusal> {
    use compiler::operation_plan::SponsorCase;
    context
        .emitted
        .constructor()
        .leaves()
        .keys()
        .find(|key| {
            matches!(key, tapscript::LeafRole::Coordinator { shape }
            if shape.ash_inputs() == 2
                && shape.sponsor_inputs() == u8::from(id.case.sponsor != SponsorCase::Absent)
                && shape.sponsor_change() == tapscript::SponsorChangePresence::Absent)
        })
        .copied()
        .ok_or_else(|| CompactClosureRefusal::EvidenceBindingMismatch(id.clone()))
}

fn emitted_offer(context: &CompactContext<'_>) -> EmittedOffer {
    EmittedOffer {
        leaves: context
            .emitted
            .constructor()
            .leaves()
            .iter()
            .map(|(key, leaf)| (*key, leaf.program().clone()))
            .collect(),
        declarations: context
            .emitted
            .outstanding_lifecycle()
            .requirements()
            .cloned()
            .collect(),
    }
}

fn dependency_operands(
    context: &CompactContext<'_>,
) -> Result<(CompactOperand, CompactOperand), CompactClosureRefusal> {
    let FirstPartyNegativeCase::PrivateRequiredDependency {
        operation,
        published,
        availability,
    } = compact_ash_permissionless_private_dependency_case()
        .map_err(CompactClosureRefusal::Dependency)?;
    let cases = context
        .input
        .realization()
        .constructibility_authorizations(operation)
        .map_err(|_| {
            CompactClosureRefusal::Dependency(FirstPartyEvidenceRefusal::RealizationUnavailable)
        })?;
    let [authorization] = cases else {
        return Err(CompactClosureRefusal::Dependency(
            FirstPartyEvidenceRefusal::AuthorizationCaseUnderdetermined,
        ));
    };
    let malformed = with_availability(&published, availability).ok_or(
        CompactClosureRefusal::Dependency(FirstPartyEvidenceRefusal::ControlHasNoAvailabilityClass),
    )?;
    Ok((
        CompactOperand::Dependency {
            operation,
            authorization: authorization.clone(),
            node: published,
        },
        CompactOperand::Dependency {
            operation,
            authorization: authorization.clone(),
            node: malformed,
        },
    ))
}

fn compact_envelope_control() -> realization::OperationObservation {
    use architecture::{AssetId, BoundId, DeltaKind, OpenFlowKind, ProjectionId};
    use realization::{
        Count, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition, ObservedObject,
        ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow, ObservedOperationRegion,
        ObservedSide, ObservedValue, OperationObservation, OwnerId, ProtocolAmount,
        RepresentationMode,
    };
    use std::collections::BTreeMap;
    let sponsor = OwnerId([3; 32]);
    let input0 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    let input1 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let output = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 0,
    };
    let sponsor_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 2,
    };
    let sponsor_output = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 1,
    };
    let mut objects: Vec<_> = [(input0, 40), (input1, 60), (output, 100)]
        .into_iter()
        .map(|(reference, amount)| ObservedObject {
            reference,
            kind: ObservedObjectKind::Declared(ObjectId::Ash),
            asset: ObservedAsset::Declared(AssetId::U),
            value: ObservedValue::Protocol(
                ProtocolAmount::new(amount).unwrap_or(ProtocolAmount::ZERO),
            ),
            owner: None,
            representation: RepresentationMode::Explicit,
        })
        .collect();
    objects.extend(
        [sponsor_input, sponsor_output]
            .into_iter()
            .map(|reference| ObservedObject {
                reference,
                kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
                asset: ObservedAsset::Declared(AssetId::Lbtc),
                value: ObservedValue::SponsorOpaque,
                owner: Some(sponsor),
                representation: RepresentationMode::Explicit,
            }),
    );
    OperationObservation {
        operation: OperationId::CompactAsh,
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::from([sponsor]),
        region: ObservedOperationRegion {
            members: objects.iter().map(|o| o.reference).collect(),
            canonical_partition: ObservedCanonicalPartition {
                issuances: Vec::new(),
                flows: vec![ObservedCanonicalFlow {
                    asset: AssetId::U,
                    sources: vec![input0, input1],
                    destinations: vec![output],
                    movement_kind: Some(DeltaKind::OwnerlessLateral),
                    destructions: Vec::new(),
                }],
            },
            open_flows: vec![ObservedOpenFlow {
                kind: OpenFlowKind::FeeSponsor,
                sources: vec![sponsor_input],
                destinations: vec![sponsor_output],
                fee: ProtocolAmount::new(3).unwrap_or(ProtocolAmount::ZERO),
            }],
        },
        other_regions: Vec::new(),
        objects,
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the exhaustive mutation table retains each exact pair beside its refusal"
)]
fn record_pair(
    context: &CompactContext<'_>,
    corpus: Option<&crate::compact_corpus::CompactCorpus>,
    row: &crate::compact_negative_manifest::CompactNegativeRow,
) -> Result<CompactRecord, CompactClosureRefusal> {
    use CompactObservedRefusal as R;
    use CompactOperand as O;
    use RelationMutation as M;
    use compiler::operation_plan::CoverageBoundary;
    use realization::{
        LifecycleDependencyProjection, LifecycleEdge, LifecycleNodeId, RepresentationMode,
    };
    let id = &row.requirement;
    let (control, subject, refusal) = match row.mutation {
        M::PermissionlessPrivateDependency | M::ConstructibilityWitnessUnavailable => {
            let (control, subject) = dependency_operands(context)?;
            let O::Dependency { node, .. } = &subject else {
                return Err(CompactClosureRefusal::PairMismatch(id.clone()));
            };
            (
                control,
                subject.clone(),
                R::Compiler(CompileError::PermissionlessPrivateDependency {
                    operation: OperationId::CompactAsh,
                    node: node.clone(),
                    path: Vec::new(),
                }),
            )
        }
        M::UnsupportedRepresentation => (
            O::Selection(RepresentationMode::Explicit),
            O::Selection(RepresentationMode::PrivateCommitted),
            R::Compiler(CompileError::TargetPlanRepresentationMismatch {
                object: ObjectId::Ash,
            }),
        ),
        M::RequiredLifecycleExitMissing if id.boundary == CoverageBoundary::CompilerStatic => {
            let realization::RelationSubject::LifecycleExit { exit, .. } = id.relation.subject()
            else {
                return Err(CompactClosureRefusal::PairMismatch(id.clone()));
            };
            let control = context.input.realization().project().lifecycle;
            let mut subject = control.clone();
            let edge = LifecycleDependencyProjection {
                source: LifecycleNodeId::Representation {
                    object: ObjectId::Ash,
                    mode: RepresentationMode::Explicit,
                },
                target: LifecycleNodeId::RequiredExit {
                    object: ObjectId::Ash,
                    operation: *exit,
                },
                edge: LifecycleEdge::RequiresExit,
            };
            let position = subject
                .edges
                .iter()
                .position(|e| e == &edge)
                .ok_or_else(|| CompactClosureRefusal::PairMismatch(id.clone()))?;
            subject.edges.remove(position);
            (
                O::Projection(Box::new(control)),
                O::Projection(Box::new(subject)),
                R::Compiler(CompileError::MissingLifecyclePath {
                    object: ObjectId::Ash,
                    representation: RepresentationMode::Explicit,
                    exit: *exit,
                }),
            )
        }
        M::SponsorEnvelopeMultiplicityExceeded => {
            let control = compact_envelope_control();
            let mut subject = control.clone();
            subject
                .region
                .open_flows
                .push(realization::ObservedOpenFlow {
                    kind: architecture::OpenFlowKind::FeeSponsor,
                    sources: Vec::new(),
                    destinations: Vec::new(),
                    fee: realization::ProtocolAmount::ZERO,
                });
            (
                O::Envelopes(Box::new(control)),
                O::Envelopes(Box::new(subject)),
                R::Multiplicity(compiler::SponsorEnvelopeMultiplicityError::Refused(
                    realization::RelationVerdict {
                        relation: id.relation.clone(),
                        status: realization::RelationStatus::Failed {
                            reason: realization::RelationFailure::SponsorEnvelopeMultiplicity,
                        },
                    },
                )),
            )
        }
        M::UnexpectedProtocolSecret => {
            let key = compact_coordinator(context, id)?;
            let control = context.emitted.constructor().leaves()[&key]
                .program()
                .clone();
            let mut instructions = control.instructions().to_vec();
            instructions.insert(
                0,
                tapscript::TapscriptInstruction::Opcode(target_elements::OpcodeId::CheckSig),
            );
            let subject = tapscript::TapscriptProgram::new(instructions).map_err(|e| {
                closure_construction(crate::error::FixtureBundleRefusal::ProgramConstruction(e))
            })?;
            (
                O::Secret(control),
                O::Secret(subject),
                R::Secret(tapscript::CompactSecretFreePathRefusal::UnexpectedProtocolSecret),
            )
        }
        M::UnauthenticatedRepresentation => {
            let control = tapscript::pattern::ash_input_recognition_fragment(
                context.bundle.target(),
                context.emitted.unresolved_symbols(),
                0,
            )
            .map_err(|e| closure_construction(crate::error::FixtureBundleRefusal::Symbols(e)))?;
            let mut instructions = control.instructions().to_vec();
            let index = instructions
                .iter()
                .position(|i| {
                    *i == tapscript::TapscriptInstruction::Opcode(
                        target_elements::OpcodeId::InspectInputValue,
                    )
                })
                .ok_or_else(|| CompactClosureRefusal::PairMismatch(id.clone()))?
                + 2;
            if instructions.get(index)
                != Some(&tapscript::TapscriptInstruction::Opcode(
                    target_elements::OpcodeId::EqualVerify,
                ))
            {
                return Err(CompactClosureRefusal::PairMismatch(id.clone()));
            }
            instructions.remove(index);
            let subject = tapscript::TapscriptProgram::new(instructions).map_err(|e| {
                closure_construction(crate::error::FixtureBundleRefusal::ProgramConstruction(e))
            })?;
            (O::Recognition(control), O::Recognition(subject),
                R::Representation(tapscript::CompactEmittedRepresentationRefusal::MissingExplicitValueAuthentication))
        }
        M::RequiredLifecycleExitMissing => {
            let control = emitted_offer(context);
            let mut subject = control.clone();
            let refusal = match id.relation.subject() {
                realization::RelationSubject::LifecycleExit {
                    exit: OperationId::CompactAsh,
                    ..
                } => {
                    let leaf = compact_coordinator(context, id)?;
                    subject.leaves.remove(&leaf);
                    tapscript::CompactEmittedLifecycleRefusal::MissingCompactExit { leaf }
                }
                realization::RelationSubject::LifecycleExit {
                    exit: OperationId::Clear,
                    ..
                } => {
                    let requirement = control
                        .declarations
                        .iter()
                        .find(|r| r.exit == OperationId::Clear)
                        .ok_or_else(|| CompactClosureRefusal::PairMismatch(id.clone()))?
                        .clone();
                    subject.declarations.remove(&requirement);
                    tapscript::CompactEmittedLifecycleRefusal::MissingClearDeclaration {
                        requirement,
                    }
                }
                _ => return Err(CompactClosureRefusal::PairMismatch(id.clone())),
            };
            (
                O::Emitted(Box::new(control)),
                O::Emitted(Box::new(subject)),
                R::Lifecycle(refusal),
            )
        }
        M::ExternalEvidenceMissing
        | M::ExternalEvidenceFailed
        | M::ExternalEvidenceIdentityMismatch => {
            let corpus =
                corpus.ok_or_else(|| CompactClosureRefusal::EvidenceBindingMismatch(id.clone()))?;
            let (control, subject, refusal) =
                crate::report::stage_compact_substrate_pair(corpus, id)?;
            (
                O::Report(Some(Box::new(control))),
                O::Report(subject.map(Box::new)),
                R::Substrate(refusal),
            )
        }
        _ => return Err(CompactClosureRefusal::PairMismatch(id.clone())),
    };
    let mut record = CompactRecord {
        requirement: id.clone(),
        control,
        subject,
        refusal,
        kind: CompactClosureKind::ObservedRefusal,
        reason: None,
    };
    if matches!(
        record.refusal,
        R::Lifecycle(tapscript::CompactEmittedLifecycleRefusal::MissingClearDeclaration { .. })
    ) {
        record.kind = CompactClosureKind::ObservedDeclarationRefusal;
    }
    if row.mutation == M::ConstructibilityWitnessUnavailable {
        let O::Dependency {
            operation,
            authorization,
            ..
        } = &record.control
        else {
            return Err(CompactClosureRefusal::FormalPremiseMismatch(id.clone()));
        };
        if !matches!(
            authorization,
            realization::ConstructibilityAuthorization::Permissionless
                | realization::ConstructibilityAuthorization::CadencePermissionless
        ) {
            return Err(CompactClosureRefusal::FormalPremiseMismatch(id.clone()));
        }
        let mut supporting = record.clone();
        supporting.requirement.purpose = compiler::operation_plan::CoveragePurpose::FocusedReject(
            M::PermissionlessPrivateDependency,
        );
        if context
            .bundle
            .plan()
            .coverage_requirement(&supporting.requirement)
            .is_none()
        {
            return Err(CompactClosureRefusal::FormalPremiseMismatch(id.clone()));
        }
        record.reason = Some(
            CompactFormalReason::UnavailableWitnessUnderPermissionlessAuthorization {
                operation: *operation,
                authorization: authorization.clone(),
                requested: row.mutation,
                supporting: Box::new(CompactFirstPartyComparisonEvidence { record: supporting }),
                ruling: ("rule:phase7:wave0-rulings", 137),
            },
        );
        record.kind = CompactClosureKind::Formal;
    }
    Ok(record)
}

fn read_compact_operand(
    context: &CompactContext<'_>,
    corpus: Option<&crate::compact_corpus::CompactCorpus>,
    id: &CoverageRequirementId,
    operand: &CompactOperand,
) -> Result<Result<(), CompactObservedRefusal>, CompactClosureRefusal> {
    use CompactObservedRefusal as R;
    use CompactOperand as O;
    Ok(match operand {
        O::Dependency {
            operation,
            authorization,
            node,
        } => validate_required_dependency(*operation, authorization, node, Vec::new)
            .map_err(R::Compiler),
        O::Selection(offered) => compiler::operation_plan::validate_compact_representation(
            &context.input,
            compact_limits(),
            context.bundle.plan(),
            *offered,
        )
        .map_err(R::Compiler),
        O::Projection(offered) => {
            compiler::validate_compact_lifecycle_projection(&context.input, offered)
                .map_err(R::Compiler)
        }
        O::Envelopes(offered) => {
            compiler::validate_sponsor_envelope_multiplicity(context.input.realization(), offered)
                .map_err(R::Multiplicity)
        }
        O::Secret(offered) => {
            tapscript::validate_compact_secret_free_path(offered).map_err(R::Secret)
        }
        O::Recognition(offered) => tapscript::validate_compact_emitted_representation(
            context.bundle.target(),
            &context.emitted,
            offered,
        )
        .map_err(R::Representation),
        O::Emitted(offered) => tapscript::validate_compact_emitted_lifecycle(
            &context.emitted,
            &offered.leaves,
            &offered.declarations,
        )
        .map_err(R::Lifecycle),
        O::Report(offered) => {
            let corpus =
                corpus.ok_or_else(|| CompactClosureRefusal::EvidenceBindingMismatch(id.clone()))?;
            let submission = crate::report::compact_substrate_submission(corpus, id, false)?;
            let comparison = crate::report::prepare_compact_substrate_report_comparison(
                submission,
                offered.as_deref(),
            )
            .map_err(CompactClosureRefusal::Preparation)?;
            crate::report::validate_compact_substrate_report(&comparison).map_err(R::Substrate)
        }
    })
}

fn check_compact_pair(
    context: &CompactContext<'_>,
    corpus: Option<&crate::compact_corpus::CompactCorpus>,
    record: &CompactRecord,
) -> Result<(), CompactClosureRefusal> {
    let id = &record.requirement;
    if record.control == record.subject {
        return Err(CompactClosureRefusal::SubjectEqualsControl(id.clone()));
    }
    if matches!(
        id.purpose,
        compiler::operation_plan::CoveragePurpose::FocusedReject(
            RelationMutation::PermissionlessPrivateDependency
        )
    ) {
        let case = compact_ash_permissionless_private_dependency_case()
            .map_err(CompactClosureRefusal::Dependency)?;
        let evidence = validate_first_party_negative(context.bundle.plan(), id, &case)
            .map_err(CompactClosureRefusal::Dependency)?;
        if let (
            CompactOperand::Dependency { node: control, .. },
            CompactOperand::Dependency { node: subject, .. },
        ) = (&record.control, &record.subject)
            && control == evidence.control()
            && subject == evidence.malformed()
            && record.refusal
                == CompactObservedRefusal::Compiler(CompileError::PermissionlessPrivateDependency {
                    operation: OperationId::CompactAsh,
                    node: subject.clone(),
                    path: Vec::new(),
                })
        {
            return Ok(());
        }
        return Err(CompactClosureRefusal::PairMismatch(id.clone()));
    }
    if read_compact_operand(context, corpus, id, &record.control)?.is_err() {
        return Err(CompactClosureRefusal::ControlRefused(id.clone()));
    }
    match read_compact_operand(context, corpus, id, &record.subject)? {
        Ok(()) => Err(CompactClosureRefusal::SubjectAccepted(id.clone())),
        Err(refusal) if refusal != record.refusal => {
            Err(CompactClosureRefusal::WrongRefusal(id.clone()))
        }
        Err(_) => Ok(()),
    }
}

/// Stage exact compact pairs without changing plan or archive standing.
///
/// # Errors
/// Returns construction, binding or exact-pair failures before any ledger exists.
pub fn stage_compact_closure_readings<'e>(
    bundle: &'e crate::bundle::FixtureBundle,
    corpus: &'e crate::compact_corpus::CompactCorpus,
) -> Result<Vec<CompactClosureReading<'e>>, CompactClosureRefusal> {
    let context = compact_context(bundle)?;
    compact_rows(bundle)?
        .iter()
        .map(|row| {
            let evidence = record_pair(&context, Some(corpus), row)?;
            check_compact_pair(&context, Some(corpus), &evidence)?;
            Ok(CompactClosureReading {
                requirement: row.requirement.clone(),
                evidence,
                bundle,
                corpus,
            })
        })
        .collect()
}

/// Admit precisely the nonnative negative keys through a bidirectional join.
///
/// Each canonical pair is rebuilt and executed at its declared entry. Formal
/// evidence retains its checked argument instead of claiming the requested error.
/// Neither the derived plan nor archive admission is changed.
///
/// # Errors
/// Returns distinct key, pair, premise and operand failures; no partial ledger.
pub fn admit_compact_closure_readings<'e>(
    bundle: &'e crate::bundle::FixtureBundle,
    corpus: &'e crate::compact_corpus::CompactCorpus,
    readings: Vec<CompactClosureReading<'e>>,
) -> Result<AdmittedCompactClosure<'e>, CompactClosureRefusal> {
    let context = compact_context(bundle)?;
    let expected: std::collections::BTreeMap<_, _> = compact_rows(bundle)?
        .into_iter()
        .map(|row| (row.requirement.clone(), row))
        .collect();
    let mut admitted = std::collections::BTreeMap::new();
    for reading in readings {
        let id = &reading.requirement;
        if id != &reading.evidence.requirement {
            return Err(CompactClosureRefusal::WrongKey(id.clone()));
        }
        if admitted.contains_key(id) {
            return Err(CompactClosureRefusal::DuplicateReading(id.clone()));
        }
        let row = expected
            .get(id)
            .ok_or_else(|| CompactClosureRefusal::SurplusReading(id.clone()))?;
        if !std::ptr::eq(bundle, reading.bundle) || !std::ptr::eq(corpus, reading.corpus) {
            return Err(CompactClosureRefusal::EvidenceBindingMismatch(id.clone()));
        }
        let canonical = record_pair(&context, Some(corpus), row)?;
        if reading.evidence.control == reading.evidence.subject {
            return Err(CompactClosureRefusal::SubjectEqualsControl(id.clone()));
        }
        if reading.evidence.reason != canonical.reason || reading.evidence.kind != canonical.kind {
            return Err(CompactClosureRefusal::FormalPremiseMismatch(id.clone()));
        }
        if reading.evidence.control != canonical.control
            || reading.evidence.subject != canonical.subject
        {
            return Err(CompactClosureRefusal::PairMismatch(id.clone()));
        }
        if reading.evidence.refusal != canonical.refusal {
            return Err(CompactClosureRefusal::WrongRefusal(id.clone()));
        }
        check_compact_pair(&context, Some(corpus), &reading.evidence)?;
        admitted.insert(id.clone(), reading);
    }
    for id in expected.keys() {
        if !admitted.contains_key(id) {
            return Err(CompactClosureRefusal::MissingReading(id.clone()));
        }
    }
    Ok(AdmittedCompactClosure {
        bundle,
        corpus,
        readings: admitted,
    })
}

/// Explicitly close twenty first-party keys, retaining observed and formal kinds.
///
/// # Errors
/// Returns manifest or evidence failures instead of silently omitting a row.
pub fn compact_first_party_dispositions(
    bundle: &crate::bundle::FixtureBundle,
) -> Result<
    std::collections::BTreeMap<CoverageRequirementId, CompactFirstPartyDisposition>,
    crate::VectorError,
> {
    use crate::compact_negative_manifest::NegativeClass;
    let wrap = |error| crate::VectorError::CompactClosure(Box::new(error));
    let context = compact_context(bundle).map_err(wrap)?;
    let manifest = compact_rows(bundle).map_err(wrap)?;
    let staging = CompactDependencyTestStaging::new()
        .map_err(|cause| crate::VectorError::FirstPartyEvidenceRefused { cause })?;
    let mut rows = std::collections::BTreeMap::new();
    for row in manifest {
        if row.class != NegativeClass::FirstParty {
            continue;
        }
        let id = row.requirement.clone();
        let disposition = if row.mutation == RelationMutation::PermissionlessPrivateDependency {
            CompactFirstPartyDisposition::Discharged(Box::new(
                staging
                    .validate(bundle.plan(), &id)
                    .map_err(|cause| crate::VectorError::FirstPartyEvidenceRefused { cause })?,
            ))
        } else {
            let record = record_pair(&context, None, &row).map_err(wrap)?;
            check_compact_pair(&context, None, &record).map_err(wrap)?;
            if let Some(reason) = record.reason {
                CompactFirstPartyDisposition::FormallyClosed(Box::new(
                    CompactFirstPartyFormalEvidence {
                        requirement: id.clone(),
                        reason,
                    },
                ))
            } else {
                CompactFirstPartyDisposition::Compared(Box::new(
                    CompactFirstPartyComparisonEvidence { record },
                ))
            }
        };
        rows.insert(id, disposition);
    }
    Ok(rows)
}

/// Stable display name derived from the exact first-party requirement key.
///
/// # Errors
/// Refuses a requirement outside the admitted ten first-party relation classes.
pub fn compact_first_party_name(
    id: &CoverageRequirementId,
) -> Result<String, FirstPartyEvidenceRefusal> {
    use compiler::operation_plan::{CoverageBoundary, CoveragePurpose, SponsorCase};
    use realization::RelationSubject;
    let (CoveragePurpose::FocusedReject(mutation) | CoveragePurpose::StructuralRejection(mutation)) =
        id.purpose
    else {
        return Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement);
    };
    let base = match mutation {
        RelationMutation::PermissionlessPrivateDependency => "permissionless-private-dependency",
        RelationMutation::ConstructibilityWitnessUnavailable => {
            "constructibility-witness-unavailable"
        }
        RelationMutation::UnsupportedRepresentation => "unsupported-representation",
        RelationMutation::UnexpectedProtocolSecret => "unexpected-protocol-secret",
        RelationMutation::UnauthenticatedRepresentation => "unauthenticated-representation",
        RelationMutation::SponsorEnvelopeMultiplicityExceeded => "sponsor-multiplicity-exceeded",
        RelationMutation::RequiredLifecycleExitMissing => {
            match (id.relation.subject(), id.boundary) {
                (
                    RelationSubject::LifecycleExit {
                        exit: OperationId::CompactAsh,
                        ..
                    },
                    CoverageBoundary::CompilerStatic,
                ) => "missing-compact-exit-analysis",
                (
                    RelationSubject::LifecycleExit {
                        exit: OperationId::Clear,
                        ..
                    },
                    CoverageBoundary::CompilerStatic,
                ) => "missing-clear-exit-analysis",
                (
                    RelationSubject::LifecycleExit {
                        exit: OperationId::CompactAsh,
                        ..
                    },
                    CoverageBoundary::BackendStructural,
                ) => "missing-compact-exit-emitted",
                (
                    RelationSubject::LifecycleExit {
                        exit: OperationId::Clear,
                        ..
                    },
                    CoverageBoundary::BackendStructural,
                ) => "missing-clear-exit-emitted",
                _ => return Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement),
            }
        }
        _ => return Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement),
    };
    let sponsor = match id.case.sponsor {
        SponsorCase::Absent => "sponsorless",
        SponsorCase::Present => "sponsored",
    };
    Ok(format!("{base}-{sponsor}"))
}

#[cfg(test)]
mod tests {
    use super::{
        FirstPartyEvidenceRefusal, FirstPartyNegativeCase, FirstPartyValidator,
        compact_ash_permissionless_private_dependency_case, validate_first_party_negative,
    };
    use crate::bundle::fixture_bundle;
    use architecture::{ObjectId, OperationId};
    use compiler::operation_plan::{
        CoverageRequirementId, EvidenceRole, RelationMutation, SponsorCase,
        TargetCoverageObligation,
    };
    use realization::{AvailabilityClass, ConstructibilityNodeId};

    #[test]
    fn tracing_variant_names_are_lower_snake_case() {
        fn names_after<'a>(source: &'a str, marker: &str) -> Vec<&'a str> {
            let body = source
                .split_once(marker)
                .expect("tracing name table")
                .1
                .split_once("\n    }\n")
                .expect("closed tracing name table")
                .0;
            body.split('"').skip(1).step_by(2).collect()
        }
        fn is_lower_snake_case(name: &str) -> bool {
            name.bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase())
                && name.split('_').all(|part| {
                    !part.is_empty()
                        && part
                            .bytes()
                            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                })
        }
        let mut names = Vec::new();
        for (source, marker) in [
            (include_str!("error.rs"), "fn tracing_refusal_kind"),
            (include_str!("first_party.rs"), "fn tracing_refusal_kind"),
            (
                include_str!("maturity_corpus.rs"),
                "fn tracing_refusal_kind",
            ),
            (include_str!("report.rs"), "fn tracing_refusal_kind"),
            (include_str!("first_party.rs"), "fn tracing_observed_kind"),
        ] {
            let table = names_after(source, marker);
            assert!(!table.is_empty(), "empty tracing name table: {marker}");
            names.extend(table);
        }
        for name in &names {
            assert!(is_lower_snake_case(name), "invalid tracing name: {name}");
        }
        assert!(names.contains(&"not_a_first_party_requirement"));
        for (class, expected) in [
            (
                RelationMutation::PermissionlessPrivateDependency,
                "permissionless_private_dependency",
            ),
            (
                RelationMutation::ConstructibilityWitnessUnavailable,
                "constructibility_witness_unavailable",
            ),
        ] {
            let refusal = super::FirstPartyRefusal {
                validator: FirstPartyValidator::CompilerRequiredDependency,
                class,
            };
            assert_eq!(refusal.tracing_observed_kind(), expected);
        }
    }

    #[test]
    fn compact_first_party_dispositions_retain_all_twenty_subjects() {
        use super::CompactFirstPartyDisposition as D;
        let bundle = fixture_bundle().expect("bundle");
        let rows = super::compact_first_party_dispositions(&bundle).expect("dispositions");
        // Before scope-only multiplicity: 18 rows, with 16 retained.
        // Before explicit closure admission: 2 discharged / 18 retained.
        assert_eq!(rows.len(), 20);
        assert_eq!(rows.values().filter(|r| r.is_closed()).count(), 20);
        assert_eq!(
            rows.values()
                .filter(|r| matches!(r, D::Discharged(_)))
                .count(),
            2
        );
        assert_eq!(
            rows.values()
                .filter(|r| matches!(r, D::Retained { .. }))
                .count(),
            0
        );
        let mut names = std::collections::BTreeSet::new();
        for (id, disposition) in &rows {
            assert!(names.insert(super::compact_first_party_name(id).expect("named")));
            match disposition {
                D::Discharged(evidence) => assert_eq!(evidence.requirement(), id),
                D::Compared(evidence) => assert_eq!(evidence.requirement(), id),
                D::FormallyClosed(evidence) => assert_eq!(evidence.requirement(), id),
                D::Retained { .. } => panic!("explicit staging retains no unanswered key"),
            }
        }
        assert_eq!(rows.values().filter(|row| row.is_observed()).count(), 18);
        assert_eq!(
            rows.values()
                .filter(|row| matches!(row, D::FormallyClosed(_)))
                .count(),
            2
        );
        assert_eq!(rows.values().filter(|row| matches!(row, D::Compared(e) if e.kind() == super::CompactClosureKind::ObservedDeclarationRefusal)).count(), 2);
        let manifest =
            crate::compact_negative_manifest::compact_negative_manifest(&bundle).expect("manifest");
        assert_eq!(
            rows.keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            manifest
                .into_values()
                .filter(
                    |row| row.class == crate::compact_negative_manifest::NegativeClass::FirstParty
                )
                .map(|row| row.requirement)
                .collect()
        );
        for suffix in ["sponsorless", "sponsored"] {
            for base in [
                "permissionless-private-dependency",
                "constructibility-witness-unavailable",
                "unsupported-representation",
                "unexpected-protocol-secret",
                "unauthenticated-representation",
                "sponsor-multiplicity-exceeded",
                "missing-compact-exit-analysis",
                "missing-clear-exit-analysis",
                "missing-compact-exit-emitted",
                "missing-clear-exit-emitted",
            ] {
                assert!(names.contains(&format!("{base}-{suffix}")));
            }
        }
    }

    #[test]
    fn compact_envelope_control_is_conformant() {
        let input = crate::bundle::bind_compact_ash_input().expect("input");
        assert!(
            input
                .realization()
                .evaluate_operation(&super::compact_envelope_control())
                .expect("evaluation")
                .is_conformant()
        );
    }

    fn ledger_sources() -> (
        &'static crate::bundle::FixtureBundle,
        &'static crate::compact_corpus::CompactCorpus,
    ) {
        static BUNDLE: std::sync::OnceLock<crate::bundle::FixtureBundle> =
            std::sync::OnceLock::new();
        (
            BUNDLE.get_or_init(|| fixture_bundle().expect("fixture")),
            crate::compact_corpus::joined_campaign().expect("admitted corpus"),
        )
    }

    fn ledger_readings() -> Vec<super::CompactClosureReading<'static>> {
        static READINGS: std::sync::OnceLock<Vec<super::CompactClosureReading<'static>>> =
            std::sync::OnceLock::new();
        READINGS
            .get_or_init(|| {
                let (bundle, corpus) = ledger_sources();
                super::stage_compact_closure_readings(bundle, corpus).expect("exact pairs")
            })
            .clone()
    }

    fn admit_readings(
        readings: Vec<super::CompactClosureReading<'static>>,
    ) -> Result<super::AdmittedCompactClosure<'static>, super::CompactClosureRefusal> {
        let (bundle, corpus) = ledger_sources();
        super::admit_compact_closure_readings(bundle, corpus, readings)
    }

    #[test]
    fn compact_closure_joins_all_twenty_six_keys_and_the_native_forty_eight() {
        use super::CompactClosureKind as K;
        let (bundle, corpus) = ledger_sources();
        let ledger = admit_readings(ledger_readings()).expect("ledger");
        assert!(std::ptr::eq(ledger.bundle(), bundle));
        assert!(std::ptr::eq(ledger.corpus(), corpus));
        assert_eq!(ledger.records().len(), 26);
        let count = |kind| {
            ledger
                .records()
                .values()
                .filter(|r| r.kind() == kind)
                .count()
        };
        assert_eq!(
            [
                count(K::ObservedRefusal),
                count(K::ObservedDeclarationRefusal),
                count(K::Formal)
            ],
            [22, 2, 2]
        );
        let expected: std::collections::BTreeSet<_> = super::compact_rows(bundle)
            .expect("manifest")
            .into_iter()
            .map(|row| row.requirement)
            .collect();
        assert_eq!(
            ledger
                .records()
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            expected
        );
        let native = corpus.rows();
        assert_eq!(native.len(), 48);
        assert!(native.keys().all(|key| !ledger.records().contains_key(key)));
        let mut keys = expected;
        keys.extend(native.keys().cloned());
        let manifest =
            crate::compact_negative_manifest::compact_negative_manifest(bundle).expect("manifest");
        assert_eq!(keys, manifest.keys().cloned().collect());
        let native_observed = native
            .values()
            .filter(|r| matches!(r, crate::compact_corpus::NativeDisposition::Observed { .. }))
            .count();
        let native_formal = native
            .values()
            .filter(|r| {
                matches!(
                    r,
                    crate::compact_corpus::NativeDisposition::FormallyClosed(_)
                )
            })
            .count();
        assert_eq!((native_observed, native_formal), (27, 21));
        let count_evidence = |kind, substrate| {
            ledger
                .records()
                .values()
                .filter(|r| {
                    r.kind() == kind
                        && matches!(r.refusal(), super::CompactObservedRefusal::Substrate(_))
                            == substrate
                })
                .count()
        };
        let report_observed = count_evidence(K::ObservedRefusal, true);
        let first_party_observed = count_evidence(K::ObservedRefusal, false)
            + count_evidence(K::ObservedDeclarationRefusal, false);
        let first_party_formal = count_evidence(K::Formal, false);
        assert_eq!(
            (first_party_observed, first_party_formal, report_observed),
            (18, 2, 6)
        );
        // 74 = 27 native observed + 21 native formal + 18 first-party observed
        // + 2 first-party formal + 6 report observed; zero retained or unresolved.
        assert_eq!(
            keys.len(),
            native_observed
                + native_formal
                + first_party_observed
                + first_party_formal
                + report_observed
        );
        assert_eq!(
            native_observed + count(K::ObservedRefusal) + count(K::ObservedDeclarationRefusal),
            51
        );
        assert_eq!(native_formal + count(K::Formal), 23);
        let mut permuted = ledger_readings();
        permuted.reverse();
        let permuted = admit_readings(permuted).expect("permutation");
        assert_eq!(
            ledger.records().keys().collect::<Vec<_>>(),
            permuted.records().keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn compact_closure_join_defects_have_four_distinct_key_classes() {
        use super::CompactClosureRefusal as R;
        let base = ledger_readings();
        let mut missing = base.clone();
        let absent = missing.pop().expect("last").requirement;
        assert_eq!(
            admit_readings(missing).err(),
            Some(R::MissingReading(absent))
        );
        let mut duplicate = base.clone();
        duplicate.push(base[0].clone());
        assert_eq!(
            admit_readings(duplicate).err(),
            Some(R::DuplicateReading(base[0].requirement.clone()))
        );
        let mut surplus = base.clone();
        let mut extra = base[0].clone();
        extra.requirement.case.operation = OperationId::Clear;
        extra.evidence.requirement = extra.requirement.clone();
        let extra_key = extra.requirement.clone();
        surplus.push(extra);
        assert_eq!(
            admit_readings(surplus).err(),
            Some(R::SurplusReading(extra_key))
        );
        let mut wrong = base;
        wrong[0].requirement.case.operation = OperationId::Clear;
        let wrong_key = wrong[0].requirement.clone();
        assert_eq!(admit_readings(wrong).err(), Some(R::WrongKey(wrong_key)));
    }

    #[test]
    fn compact_closure_pair_and_formal_defects_cannot_receive_row_credit() {
        use super::{CompactClosureKind as K, CompactClosureRefusal as R, CompactOperand as O};
        let base = ledger_readings();
        let mut same = base.clone();
        same[0].evidence.subject = same[0].evidence.control.clone();
        assert_eq!(
            admit_readings(same).err(),
            Some(R::SubjectEqualsControl(base[0].requirement.clone()))
        );
        let mut wrong = base.clone();
        wrong[0].evidence.refusal = super::CompactObservedRefusal::Substrate(
            crate::report::CompactSubstrateReportRefusal::ExternalEvidenceMissing,
        );
        assert_eq!(
            admit_readings(wrong).err(),
            Some(R::WrongRefusal(base[0].requirement.clone()))
        );
        let formal = base
            .iter()
            .position(|r| r.kind() == K::Formal)
            .expect("formal");
        let mut changed = base.clone();
        changed[formal].evidence.kind = K::ObservedRefusal;
        assert_eq!(
            admit_readings(changed).err(),
            Some(R::FormalPremiseMismatch(base[formal].requirement.clone()))
        );
        let mut changed = base.clone();
        let Some(super::CompactFormalReason::UnavailableWitnessUnderPermissionlessAuthorization {
            operation,
            ..
        }) = &mut changed[formal].evidence.reason
        else {
            panic!("argument")
        };
        *operation = OperationId::TransferLive;
        assert_eq!(
            admit_readings(changed).err(),
            Some(R::FormalPremiseMismatch(base[formal].requirement.clone()))
        );
        let mut changed = base.clone();
        let O::Dependency { operation, .. } = &mut changed[formal].evidence.subject else {
            panic!("dependency")
        };
        *operation = OperationId::TransferLive;
        assert_eq!(
            admit_readings(changed).err(),
            Some(R::PairMismatch(base[formal].requirement.clone()))
        );
        let (bundle, corpus) = ledger_sources();
        let other = bundle.clone();
        let mut changed = ledger_readings();
        changed[0].bundle = &other;
        assert_eq!(
            super::admit_compact_closure_readings(&other, corpus, changed).err(),
            Some(R::EvidenceBindingMismatch(base[1].requirement.clone()))
        );
    }

    #[test]
    fn compact_closure_formal_authorization_must_match_the_canonical_premise() {
        let mut readings = ledger_readings();
        let reading = readings
            .iter_mut()
            .find(|r| r.kind() == super::CompactClosureKind::Formal)
            .expect("formal");
        let key = reading.requirement.clone();
        let Some(super::CompactFormalReason::UnavailableWitnessUnderPermissionlessAuthorization {
            authorization,
            ..
        }) = &mut reading.evidence.reason
        else {
            panic!("argument")
        };
        *authorization = realization::ConstructibilityAuthorization::Operator;
        assert_eq!(
            admit_readings(readings).err(),
            Some(super::CompactClosureRefusal::FormalPremiseMismatch(key))
        );
    }

    #[test]
    fn compact_closure_declaration_kind_must_match_the_canonical_reading() {
        let mut readings = ledger_readings();
        let reading = readings
            .iter_mut()
            .find(|r| r.kind() == super::CompactClosureKind::ObservedDeclarationRefusal)
            .expect("declaration");
        let key = reading.requirement.clone();
        reading.evidence.kind = super::CompactClosureKind::ObservedRefusal;
        assert_eq!(
            admit_readings(readings).err(),
            Some(super::CompactClosureRefusal::FormalPremiseMismatch(key))
        );
    }

    #[test]
    fn compact_closure_neighbouring_emitted_omission_is_not_a_compact_exit() {
        use super::{CompactClosureRefusal as R, CompactOperand as O};
        let mut readings = ledger_readings();
        let index = readings
            .iter()
            .position(|r| {
                matches!(
                    r.evidence.refusal,
                    super::CompactObservedRefusal::Lifecycle(
                        tapscript::CompactEmittedLifecycleRefusal::MissingCompactExit { .. }
                    )
                )
            })
            .expect("compact exit");
        let O::Emitted(control) = &readings[index].evidence.control else {
            panic!("emitted")
        };
        let mut subject = control.clone();
        subject
            .leaves
            .remove(&tapscript::LeafRole::Member { ash_inputs: 2 });
        readings[index].evidence.subject = O::Emitted(subject);
        let id = readings[index].requirement.clone();
        assert_eq!(admit_readings(readings).err(), Some(R::PairMismatch(id)));
    }

    #[test]
    fn compact_closure_checks_control_acceptance_and_subject_refusal_separately() {
        use super::{CompactClosureRefusal as R, CompactOperand as O};
        let (bundle, corpus) = ledger_sources();
        let context = super::compact_context(bundle).expect("context");
        let readings = ledger_readings();
        let original = readings
            .iter()
            .find(|r| matches!(r.evidence.control, O::Selection(_)))
            .expect("selection");
        let mut record = original.evidence.clone();
        record.control = O::Selection(realization::RepresentationMode::PrivateCommitted);
        record.subject = O::Selection(realization::RepresentationMode::Explicit);
        assert_eq!(
            super::check_compact_pair(&context, Some(corpus), &record),
            Err(R::ControlRefused(record.requirement.clone()))
        );
        let original = readings
            .iter()
            .find(|r| matches!(r.evidence.control, O::Secret(_)))
            .expect("secret");
        let mut record = original.evidence.clone();
        let O::Secret(control) = &record.control else {
            panic!("program")
        };
        let mut instructions = control.instructions().to_vec();
        instructions.insert(
            0,
            tapscript::TapscriptInstruction::Opcode(target_elements::OpcodeId::Drop),
        );
        record.subject =
            O::Secret(tapscript::TapscriptProgram::new(instructions).expect("program"));
        assert_eq!(
            super::check_compact_pair(&context, Some(corpus), &record),
            Err(R::SubjectAccepted(record.requirement.clone()))
        );
    }

    #[test]
    fn compact_closure_every_pair_restores_exactly_one_coordinate_and_accepts() {
        let (bundle, corpus) = ledger_sources();
        let context = super::compact_context(bundle).expect("context");
        for reading in ledger_readings() {
            let record = &reading.evidence;
            let restored = restore_compact_coordinate(record);
            assert_eq!(restored, record.control);
            assert_ne!(record.subject, record.control);
            assert_eq!(
                super::read_compact_operand(&context, Some(corpus), &record.requirement, &restored),
                Ok(Ok(()))
            );
            assert_eq!(
                super::read_compact_operand(
                    &context,
                    Some(corpus),
                    &record.requirement,
                    &record.subject
                ),
                Ok(Err(record.refusal.clone()))
            );
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "each operand restores only its declared coordinate"
    )]
    fn restore_compact_coordinate(record: &super::CompactRecord) -> super::CompactOperand {
        use super::CompactOperand as O;
        let mut restored = record.subject.clone();
        match (&record.control, &mut restored) {
            (O::Dependency { node: control, .. }, O::Dependency { node, .. }) => {
                let availability = match control {
                    ConstructibilityNodeId::Fact { availability, .. }
                    | ConstructibilityNodeId::Witness { availability, .. } => *availability,
                    ConstructibilityNodeId::Operation(_) => panic!("availability"),
                };
                *node = super::with_availability(node, availability).expect("availability");
            }
            (O::Selection(control), O::Selection(subject)) => *subject = *control,
            (O::Projection(control), O::Projection(subject)) => {
                assert_eq!(control.nodes, subject.nodes);
                assert_eq!(control.edges.len(), subject.edges.len() + 1);
                let missing = control
                    .edges
                    .iter()
                    .find(|e| !subject.edges.contains(e))
                    .expect("one edge")
                    .clone();
                let index = control
                    .edges
                    .iter()
                    .position(|e| e == &missing)
                    .expect("index");
                subject.edges.insert(index, missing);
            }
            (O::Envelopes(control), O::Envelopes(subject)) => {
                assert_eq!(
                    subject.region.open_flows.len(),
                    control.region.open_flows.len() + 1
                );
                subject.region.open_flows.pop().expect("one envelope");
            }
            (O::Secret(control), O::Secret(subject)) => {
                let mut instructions = subject.instructions().to_vec();
                assert_eq!(
                    instructions.remove(0),
                    tapscript::TapscriptInstruction::Opcode(target_elements::OpcodeId::CheckSig)
                );
                *subject = tapscript::TapscriptProgram::new(instructions).expect("restored");
                assert_eq!(subject, control);
            }
            (O::Recognition(control), O::Recognition(subject)) => {
                let index = control
                    .instructions()
                    .iter()
                    .position(|i| {
                        *i == tapscript::TapscriptInstruction::Opcode(
                            target_elements::OpcodeId::InspectInputValue,
                        )
                    })
                    .expect("inspection")
                    + 2;
                let mut instructions = subject.instructions().to_vec();
                instructions.insert(index, control.instructions()[index].clone());
                *subject = tapscript::TapscriptProgram::new(instructions).expect("restored");
            }
            (O::Emitted(control), O::Emitted(subject)) => match &record.refusal {
                super::CompactObservedRefusal::Lifecycle(
                    tapscript::CompactEmittedLifecycleRefusal::MissingCompactExit { leaf },
                ) => {
                    assert_eq!(subject.declarations, control.declarations);
                    assert_eq!(control.leaves.len(), subject.leaves.len() + 1);
                    assert_eq!(
                        subject.leaves.insert(*leaf, control.leaves[leaf].clone()),
                        None
                    );
                }
                super::CompactObservedRefusal::Lifecycle(
                    tapscript::CompactEmittedLifecycleRefusal::MissingClearDeclaration {
                        requirement,
                    },
                ) => {
                    assert_eq!(subject.leaves, control.leaves);
                    assert_eq!(control.declarations.len(), subject.declarations.len() + 1);
                    assert!(subject.declarations.insert(requirement.clone()));
                }
                _ => panic!("exact lifecycle class"),
            },
            (O::Report(Some(control)), O::Report(subject)) => match &record.refusal {
                super::CompactObservedRefusal::Substrate(
                    crate::report::CompactSubstrateReportRefusal::ExternalEvidenceMissing,
                ) => {
                    assert_eq!(*subject, None);
                    *subject = Some(control.clone());
                }
                super::CompactObservedRefusal::Substrate(
                    crate::report::CompactSubstrateReportRefusal::ExternalEvidenceFailed,
                ) => subject.as_mut().expect("report").outcome = control.outcome,
                super::CompactObservedRefusal::Substrate(
                    crate::report::CompactSubstrateReportRefusal::ExternalEvidenceIdentityMismatch,
                ) => subject.as_mut().expect("report").transaction = control.transaction.clone(),
                _ => panic!("exact report class"),
            },
            _ => panic!("matching operand kinds"),
        }
        restored
    }

    /// The first-party requirement of one class, in one sponsor case.
    fn requirement(
        plan: &compiler::operation_plan::ValidatedTargetOperationPlan,
        wanted: RelationMutation,
        sponsor: SponsorCase,
    ) -> CoverageRequirementId {
        plan.coverage()
            .find(|candidate| {
                let TargetCoverageObligation::Negative(negative) = &candidate.obligation else {
                    return false;
                };
                candidate.role == EvidenceRole::CompilerAnalysisResult
                    && candidate.id.case.sponsor == sponsor
                    && negative.mutation == wanted
            })
            .expect("the class is published as a compiler-static negative")
            .id
            .clone()
    }

    /// The compact-ASH input-family count, exactly as the realization
    /// publishes it: a required dependency of this operation whose
    /// availability is public, which is why the real analysis passes.
    fn published_public_fact() -> ConstructibilityNodeId {
        let FirstPartyNegativeCase::PrivateRequiredDependency { published, .. } =
            private_dependency();
        published
    }

    /// The canonical malformed input: that fact under a private class.
    fn private_dependency() -> FirstPartyNegativeCase {
        compact_ash_permissionless_private_dependency_case()
            .expect("the canonical private-dependency case is available")
    }

    #[test]
    fn first_party_trace_separates_observed_refusal_from_operation_refusal() {
        if crate::tracing_test::isolate_capture_test(
            module_path!(),
            "first_party_trace_separates_observed_refusal_from_operation_refusal",
        ) {
            return;
        }
        let bundle = fixture_bundle().expect("fixture");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let control = private_dependency();
        let FirstPartyNegativeCase::PrivateRequiredDependency { published, .. } = &control;
        let unchanged = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: published.clone(),
            availability: AvailabilityClass::Public,
        };
        let plain_control = validate_first_party_negative(plan, &id, &control);
        let (traced_control, control_records) =
            crate::tracing_test::capture(|| validate_first_party_negative(plan, &id, &control));
        assert_eq!(traced_control, plain_control);
        let completion = control_records
            .iter()
            .find(|record| record["fields"]["message"] == "mutant validated")
            .expect("mutant completion");
        assert_eq!(completion["target"], "vectors::validation");
        assert_eq!(completion["level"], "DEBUG");
        assert_eq!(completion["span"]["name"], "validate_mutant");
        assert_eq!(completion["fields"]["outcome"], "completed");
        assert_eq!(completion["fields"]["standing"], "refused");
        assert_eq!(
            completion["fields"]["observed_refusal_kind"],
            "permissionless_private_dependency"
        );
        assert!(completion["fields"].get("refusal_kind").is_none());
        let plain_refusal = validate_first_party_negative(plan, &id, &unchanged);
        let (traced_refusal, refusal_records) =
            crate::tracing_test::capture(|| validate_first_party_negative(plan, &id, &unchanged));
        assert_eq!(traced_refusal, plain_refusal);
        let completion = refusal_records
            .iter()
            .find(|record| record["fields"]["message"] == "mutant validated")
            .expect("mutant refusal completion");
        assert_eq!(completion["level"], "DEBUG");
        assert_eq!(completion["fields"]["outcome"], "refused");
        assert_eq!(
            completion["fields"]["refusal_kind"],
            "malformation_changed_nothing"
        );
        assert!(completion["fields"].get("standing").is_none());
        assert!(
            refusal_records
                .iter()
                .all(|record| record["level"] != "WARN" && record["level"] != "ERROR")
        );
    }

    #[test]
    fn the_permissionless_private_dependency_row_is_discharged_end_to_end() {
        // The proving instance. Compact-ASH's own authorization case is
        // the permissionless one, and every required dependency it
        // publishes is public — which is exactly why the real analysis
        // passes and why one changed availability class is the whole
        // malformation.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
            let id = requirement(
                plan,
                RelationMutation::PermissionlessPrivateDependency,
                sponsor,
            );
            let evidence = validate_first_party_negative(plan, &id, &private_dependency())
                .expect("the case discharges the row");
            assert_eq!(evidence.requirement(), &id);
            assert_eq!(
                evidence.refusal().validator(),
                FirstPartyValidator::CompilerRequiredDependency,
            );
            assert_eq!(
                evidence.refusal().class(),
                RelationMutation::PermissionlessPrivateDependency,
            );
            // And the pair really is a pair: one field apart, and the
            // control is the published value.
            assert_ne!(evidence.control(), evidence.malformed());
            assert_eq!(evidence.control(), &published_public_fact());
        }
    }

    #[test]
    fn the_same_case_does_not_discharge_the_neighbouring_class() {
        // The sibling refusal `ConstructibilityWitnessUnavailable` is
        // raised by the same validator, and this case cannot reach it:
        // compact-ASH publishes only a permissionless authorization,
        // and that is the arm the error's own construction selects on.
        // This route supplies no requested-class observation. The separate
        // closure ledger checks the formal argument of ruling 137.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::ConstructibilityWitnessUnavailable,
            SponsorCase::Absent,
        );
        assert_eq!(
            validate_first_party_negative(plan, &id, &private_dependency()),
            Err(FirstPartyEvidenceRefusal::RefusalNamesAnotherClass(
                RelationMutation::PermissionlessPrivateDependency
            )),
        );
    }

    #[test]
    fn another_operations_authorization_cannot_answer_compact_witness_coverage() {
        let bundle = fixture_bundle().expect("bundle");
        let id = requirement(
            bundle.plan(),
            RelationMutation::ConstructibilityWitnessUnavailable,
            SponsorCase::Absent,
        );
        let FirstPartyNegativeCase::PrivateRequiredDependency {
            published,
            availability,
            ..
        } = private_dependency();
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::TransferLive,
            published,
            availability,
        };
        assert_eq!(
            validate_first_party_negative(bundle.plan(), &id, &case),
            Err(FirstPartyEvidenceRefusal::CaseNamesAnotherOperation)
        );
    }

    #[test]
    fn a_control_the_workspace_does_not_publish_is_refused() {
        // Without this, the pair could be two inventions and the second
        // one's refusal would say nothing about the workspace.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: ConstructibilityNodeId::Witness {
                operation: OperationId::CompactAsh,
                role: realization::WitnessRole::OperatorAuthorization,
                availability: AvailabilityClass::Public,
            },
            availability: AvailabilityClass::Operator,
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::ControlIsNotPublished),
        );
    }

    #[test]
    fn a_malformation_that_changes_nothing_is_refused() {
        // Two identical inputs differ by nothing, so the validator would
        // answer them the same way and the control would establish the
        // opposite of what it is for.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: published_public_fact(),
            availability: AvailabilityClass::Public,
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::MalformationChangedNothing),
        );
    }

    #[test]
    fn a_malformation_the_validator_accepts_discharges_nothing() {
        // A sponsor-local availability is discharged by every
        // authorization case, so the "malformed" input is not malformed
        // and the validator says so. Without this arm a case could
        // change a field, be accepted, and still be offered as evidence
        // of a refusal.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: published_public_fact(),
            availability: AvailabilityClass::SponsorLocal,
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::MalformedInputWasAccepted),
        );
    }

    #[test]
    fn a_control_the_validator_also_refuses_discharges_nothing() {
        // The attributability guard, staged against a node the
        // realization really publishes: the live-transfer owner witness
        // is private to that operation's owners, so compact-ASH's
        // permissionless case refuses it before anything is changed. A
        // refusal of the malformed sibling then establishes nothing the
        // refusal of the control did not already.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: ConstructibilityNodeId::Witness {
                operation: OperationId::TransferLive,
                role: realization::WitnessRole::ProtocolOwnerAuthorization,
                availability: AvailabilityClass::InputOwners {
                    object: ObjectId::ReceiptLive,
                },
            },
            availability: AvailabilityClass::InputOwners {
                object: ObjectId::Ash,
            },
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::ControlWasRefused),
        );
    }

    #[test]
    fn a_runtime_requirement_is_not_answerable_here() {
        // §4.2's policy is about the first-party half only. A row a
        // target must answer stays a target's, whatever a compiler
        // refuses.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let runtime = plan
            .coverage()
            .find(|candidate| {
                matches!(candidate.obligation, TargetCoverageObligation::Negative(_))
                    && candidate.role == EvidenceRole::TargetExecution
            })
            .expect("the plan publishes a runtime negative row")
            .id
            .clone();
        assert_eq!(
            validate_first_party_negative(plan, &runtime, &private_dependency()),
            Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement),
        );
    }
}
