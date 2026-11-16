//! Staged Burn/Clear bodies. These are not declaration dispatch or engine keys.
//!
//! Local identities are deliberately separate from the public relation enums;
//! bite 2.3b installs those enums and their exhaustive consumers atomically.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, DeltaKind, ObjectId, OperationId, TagId};

use crate::burn_clear_observation::{family, read_amount};
use crate::burn_clear_projection::{
    burn_event_shape_holds, clear_outcome, clear_transition, state_succession_holds,
};
use crate::{
    BoundBurnClearObservation, BurnClearPayload, BurnClearRefusal, ExpectedCanonicalDelta, ExprId,
    ExpressionNode, FactId, FactValues, ObservedDestructionLeg, ObservedSide, OperationObservation,
    ProofKind, ProtocolAmount, RealizationError, Relation, RelationDeclaration, RelationId,
    RelationKind, RelationStatus, RelationSubject, RepresentationMode, SemanticValue, StateField,
    TransactionSide,
};

/// Operation-local staged identity, never an engine key or public hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BurnClearLawKey {
    /// The operation's fixed or residual-conditional delta family policy.
    CanonicalDeltaPolicy,
    /// Admission for live receipts on the selected transaction side.
    SideRepresentation(TransactionSide),
    /// Typed data-output structure and association.
    DataOutputs,
    /// Fresh ASH, owner routing and event predicates.
    BurnBinding,
    /// Checked Clear arithmetic and positive progress.
    ClearClamp,
    /// Assignment of one member of the existing STATE field census.
    ClearAssignment(StateField),
    /// Independent nonzero, nonincreasing supply and unchanged backing.
    ClearFloor,
    /// Exact destruction and conditional residual.
    ClearFlow,
    /// STATE succession, flow and Clear event predicates.
    ClearBinding,
}

/// Complete local identity, including its operation owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BurnClearLawId {
    /// Architecture operation that owns this law.
    pub operation: OperationId,
    /// Operation-local law subject.
    pub key: BurnClearLawKey,
}

/// Closed staged bodies. Copy operands use existing STATE facts and Equal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BurnClearLaw {
    /// Existing delta values, selected by the operation's residual condition.
    CanonicalDeltaPolicy,
    /// Only the selected side is admitted by this body.
    SideRepresentation {
        /// Side whose objects are admitted.
        side: TransactionSide,
        /// Object family whose modes are constrained.
        object: ObjectId,
        /// Exact permitted semantic representations.
        allowed: BTreeSet<RepresentationMode>,
    },
    /// Architecture-owned schema/count/tag/asset/condition, with bound carriers.
    DataOutputs {
        /// Canonical rows checked by identity validation before evaluation.
        expected: Vec<architecture::DataOutputSpec>,
    },
    /// Bound Burn class anchor, owner routing and canonical flow.
    BurnBinding,
    /// No expression-valued X or caller-supplied aggregate.
    ClearClamp,
    /// YL has no expression; the other fields require their canonical Equal.
    ClearAssignment {
        /// Existing STATE field assigned by this law.
        field: StateField,
        /// Canonical Equal node for a copy, absent exactly for YL.
        expression: Option<ExpressionNode>,
    },
    /// Independently check successor supply and backing.
    ClearFloor,
    /// Join the derived X and residual to the exact canonical flow.
    ClearFlow,
    /// Join the exact STATE edge, flow and event presence.
    ClearBinding,
}

/// One staged law and its independently checkable local identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnClearLawDeclaration {
    /// Complete local identity, validated against the body.
    pub id: BurnClearLawId,
    /// Decidable semantic body.
    pub body: BurnClearLaw,
}

/// Staging errors remain separate from the unchanged public evaluator errors.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BurnClearLawError {
    /// Owner, subject or canonical body differs.
    #[error("staged law identity or body differs from its canonical declaration")]
    Identity,
    /// The offered declaration collection is not the exact census.
    #[error("staged law census is missing, duplicated or contains another owner")]
    Census,
    /// A required operand or transition cannot be evaluated after binding.
    #[error("law operand or transition refused: {0}")]
    Operand(#[from] BurnClearRefusal),
    /// Existing expression or checked arithmetic evaluation failed.
    #[error("semantic evaluation refused: {0}")]
    Evaluation(#[from] RealizationError),
}

impl BurnClearLawError {
    /// Stable payload-free refusal name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Census => "census",
            Self::Operand(_) => "operand",
            Self::Evaluation(_) => "evaluation",
        }
    }
}

fn state_fact(side: TransactionSide, field: StateField) -> FactId {
    FactId::StateField {
        operation: OperationId::Clear,
        side,
        field,
    }
}

fn copy_expression(field: StateField) -> ExpressionNode {
    ExpressionNode::Equal {
        left: ExprId::fact(state_fact(TransactionSide::Input, field)),
        right: ExprId::fact(state_fact(TransactionSide::Output, field)),
    }
}

/// Exact staged census: five Burn laws and twelve Clear laws.
///
/// Burn includes four new laws plus its existing delta policy.
/// Composed checks reuse their existing semantic owners.
/// An unsupported operation has no staged declarations.
pub fn burn_clear_laws(operation: OperationId) -> Vec<BurnClearLawDeclaration> {
    use BurnClearLaw as L;
    let mut bodies = match operation {
        OperationId::Burn => vec![
            L::CanonicalDeltaPolicy,
            L::SideRepresentation {
                side: TransactionSide::Input,
                object: ObjectId::ReceiptLive,
                allowed: BTreeSet::from([RepresentationMode::Explicit]),
            },
            L::SideRepresentation {
                side: TransactionSide::Output,
                object: ObjectId::ReceiptLive,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PrivateCommitted,
                ]),
            },
            L::BurnBinding,
        ],
        OperationId::Clear => vec![
            L::CanonicalDeltaPolicy,
            L::ClearClamp,
            L::ClearFloor,
            L::ClearFlow,
            L::ClearBinding,
        ],
        _ => return Vec::new(),
    };
    if let Some(spec) = architecture::ARCHITECTURE.operation(operation) {
        bodies.push(L::DataOutputs {
            expected: spec.data_outputs.to_vec(),
        });
    }
    if operation == OperationId::Clear {
        bodies.extend(StateField::ALL.iter().map(|field| L::ClearAssignment {
            field: *field,
            expression: (*field != StateField::YL).then(|| copy_expression(*field)),
        }));
    }
    let mut declarations: Vec<_> = bodies
        .into_iter()
        .map(|body| BurnClearLawDeclaration {
            id: BurnClearLawId {
                operation,
                key: body.key(),
            },
            body,
        })
        .collect();
    declarations.sort_by_key(|declaration| declaration.id);
    declarations
}

impl BurnClearLaw {
    /// Derive the local subject from the body; no ordinal identity is used.
    pub const fn key(&self) -> BurnClearLawKey {
        use BurnClearLawKey as K;
        match self {
            Self::CanonicalDeltaPolicy => K::CanonicalDeltaPolicy,
            Self::SideRepresentation { side, .. } => K::SideRepresentation(*side),
            Self::DataOutputs { .. } => K::DataOutputs,
            Self::BurnBinding => K::BurnBinding,
            Self::ClearClamp => K::ClearClamp,
            Self::ClearAssignment { field, .. } => K::ClearAssignment(*field),
            Self::ClearFloor => K::ClearFloor,
            Self::ClearFlow => K::ClearFlow,
            Self::ClearBinding => K::ClearBinding,
        }
    }

    /// The frozen proof alternative; representation is a mode constraint.
    pub const fn proof_kind(&self) -> Option<ProofKind> {
        match self {
            Self::SideRepresentation { .. } => None,
            Self::ClearClamp
            | Self::ClearAssignment { .. }
            | Self::ClearFloor
            | Self::ClearFlow => Some(ProofKind::PublicArithmetic),
            Self::CanonicalDeltaPolicy
            | Self::DataOutputs { .. }
            | Self::BurnBinding
            | Self::ClearBinding => Some(ProofKind::ManifestShape),
        }
    }
}

/// Exact multiplicity and canonical body check, independent of list ordering.
pub fn validate_burn_clear_law_census(
    operation: OperationId,
    declarations: &[BurnClearLawDeclaration],
) -> Result<(), BurnClearLawError> {
    let expected = burn_clear_laws(operation);
    let mut actual = declarations.to_vec();
    actual.sort_by_key(|declaration| declaration.id);
    if expected.is_empty() || actual != expected {
        return Err(BurnClearLawError::Census);
    }
    Ok(())
}

impl BurnClearLawDeclaration {
    /// Decide a staged body from an already bound primitive extension.
    /// `false` is a law disagreement; missing facts remain typed errors.
    /// This establishes neither a complete declaration nor model conformance.
    pub fn evaluate(&self, bound: &BoundBurnClearObservation) -> Result<bool, BurnClearLawError> {
        let common = &bound.observation.context.common;
        if self.id.operation != common.operation
            || !burn_clear_laws(self.id.operation).contains(self)
        {
            return Err(BurnClearLawError::Identity);
        }
        match &self.body {
            BurnClearLaw::CanonicalDeltaPolicy => canonical_delta(bound),
            BurnClearLaw::SideRepresentation {
                side,
                object,
                allowed,
            } => {
                let side = match side {
                    TransactionSide::Input => ObservedSide::Input,
                    TransactionSide::Output => ObservedSide::Output,
                };
                Ok(common
                    .declared_objects(side, *object)
                    .all(|object| allowed.contains(&object.representation)))
            }
            // Binding already enforces exact schema, count, tag and association.
            // Clear additionally joins its data quantity to the derived clamp.
            BurnClearLaw::DataOutputs { .. } => match &bound.observation.payload {
                BurnClearPayload::Burn { .. } => Ok(true),
                BurnClearPayload::Clear { destruction, .. } => {
                    Ok(destruction.amount == clear_transition(bound)?.clear_amount)
                }
                BurnClearPayload::Compact => Err(BurnClearRefusal::ProjectionRole.into()),
            },
            BurnClearLaw::BurnBinding => burn_binding(bound),
            BurnClearLaw::ClearClamp => match clear_outcome(bound)? {
                Ok(_) => Ok(true),
                Err(
                    crate::ClearTransitionRefusal::Sealed
                    | crate::ClearTransitionRefusal::ZeroProgress,
                ) => Ok(false),
                Err(refusal) => Err(BurnClearRefusal::from(refusal).into()),
            },
            BurnClearLaw::ClearAssignment { field, expression } => {
                assignment(bound, *field, expression.as_ref())
            }
            BurnClearLaw::ClearFloor => floor(bound),
            BurnClearLaw::ClearFlow => clear_flow(bound),
            BurnClearLaw::ClearBinding => clear_binding(bound),
        }
    }
}

/// Use the existing evaluator, including its canonical membership owner.
/// The singleton graph contains the actual operation-qualified legacy identity.
fn legacy_relation(
    common: &OperationObservation,
    kind: RelationKind,
    subject: RelationSubject,
    relation: Relation,
) -> Result<bool, BurnClearLawError> {
    let declaration = RelationDeclaration {
        id: RelationId::new(common.operation, kind, subject),
        relation,
        proof_alternatives: BTreeSet::new(),
    };
    let (graph, nodes, order) = crate::relation::build_relation_graph([declaration], [])?;
    let report = crate::evaluate::evaluate_operation(
        &graph,
        &nodes,
        &order,
        &petgraph::graph::DiGraph::new(),
        &BTreeMap::new(),
        &[],
        common,
        None,
    )?;
    Ok(report.verdicts.len() == 1 && report.verdicts[0].status == RelationStatus::Passed)
}

fn canonical_delta(bound: &BoundBurnClearObservation) -> Result<bool, BurnClearLawError> {
    let common = &bound.observation.context.common;
    let mut expected = BTreeSet::new();
    match &bound.observation.payload {
        BurnClearPayload::Burn { .. } => {
            expected.insert(ExpectedCanonicalDelta {
                asset: AssetId::U,
                kind: DeltaKind::Lateral,
                destruction_tag: None,
            });
        }
        BurnClearPayload::Clear { .. } => {
            let transition = clear_transition(bound)?;
            expected.insert(ExpectedCanonicalDelta {
                asset: AssetId::U,
                kind: DeltaKind::Destruction,
                destruction_tag: Some(TagId::Recon),
            });
            if !transition.residual.is_zero() {
                expected.insert(ExpectedCanonicalDelta {
                    asset: AssetId::U,
                    kind: DeltaKind::OwnerlessLateral,
                    destruction_tag: None,
                });
            }
        }
        BurnClearPayload::Compact => return Err(BurnClearRefusal::ProjectionRole.into()),
    }
    legacy_relation(
        common,
        RelationKind::CanonicalDeltaPolicy,
        RelationSubject::Operation,
        Relation::CanonicalDeltaPolicy { expected },
    )
}

fn assignment(
    bound: &BoundBurnClearObservation,
    field: StateField,
    expression: Option<&ExpressionNode>,
) -> Result<bool, BurnClearLawError> {
    let output = state_fact(TransactionSide::Output, field);
    let actual = bound
        .resolve_state_fact(&output)
        .ok_or_else(|| RealizationError::MissingFactValue(output.clone()))?;
    if field == StateField::YL {
        return Ok(actual == SemanticValue::Amount(clear_transition(bound)?.successor.y_l));
    }
    let input = state_fact(TransactionSide::Input, field);
    let predecessor = bound
        .resolve_state_fact(&input)
        .ok_or_else(|| RealizationError::MissingFactValue(input.clone()))?;
    let values = BTreeMap::from([
        (ExprId::fact(input), predecessor),
        (ExprId::fact(output), actual),
    ]);
    let expression = expression.ok_or(BurnClearLawError::Identity)?;
    Ok(
        crate::expression::evaluate_node(expression, &FactValues::default(), &values)?
            == SemanticValue::Bool(true),
    )
}

fn floor(bound: &BoundBurnClearObservation) -> Result<bool, BurnClearLawError> {
    let BurnClearPayload::Clear {
        predecessor,
        successor,
        ..
    } = &bound.observation.payload
    else {
        return Err(BurnClearRefusal::ProjectionRole.into());
    };
    let before = predecessor
        .metadata
        .y_l
        .checked_add(predecessor.metadata.y_t)?;
    let after = successor.metadata.y_l.checked_add(successor.metadata.y_t)?;
    Ok(after >= ProtocolAmount::ONE
        && after <= before
        && successor.metadata.omega == predecessor.metadata.omega)
}

fn clear_flow(bound: &BoundBurnClearObservation) -> Result<bool, BurnClearLawError> {
    let BurnClearPayload::Clear {
        ash_sources,
        destruction,
        ..
    } = &bound.observation.payload
    else {
        return Err(BurnClearRefusal::ProjectionRole.into());
    };
    let transition = clear_transition(bound)?;
    let common = &bound.observation.context.common;
    let partition = &common.region.canonical_partition;
    if !partition.issuances.is_empty() || partition.flows.len() != 1 {
        return Ok(false);
    }
    let flow = &partition.flows[0];
    let outputs = family(common, ObservedSide::Output, ObjectId::Ash);
    let residual_holds = if transition.residual.is_zero() {
        outputs.is_empty() && flow.destinations.is_empty() && flow.movement_kind.is_none()
    } else {
        outputs.len() == 1
            && flow.destinations == outputs
            && read_amount(common, outputs[0], ObjectId::Ash)? == transition.residual
            && flow.movement_kind == Some(DeltaKind::OwnerlessLateral)
    };
    Ok(flow.asset == AssetId::U
        && flow.sources == *ash_sources
        && flow.destructions
            == [ObservedDestructionLeg {
                tag: TagId::Recon,
                amount: transition.clear_amount,
            }]
        && destruction.flow == Some(0)
        && destruction.amount == transition.clear_amount
        && residual_holds
        && canonical_delta(bound)?)
}

fn burn_binding(bound: &BoundBurnClearObservation) -> Result<bool, BurnClearLawError> {
    let BurnClearPayload::Burn { fresh_ash, .. } = &bound.observation.payload else {
        return Err(BurnClearRefusal::ProjectionRole.into());
    };
    let common = &bound.observation.context.common;
    if !burn_event_shape_holds(common, *fresh_ash) {
        return Ok(false);
    }
    let owners = legacy_relation(
        common,
        RelationKind::Authorization,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
        Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        },
    )?;
    let conserved = legacy_relation(
        common,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
        Relation::AmountConservation {
            asset: AssetId::U,
            input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            output_objects: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::Ash]),
        },
    )?;
    Ok(owners && conserved && canonical_delta(bound)?)
}

fn clear_binding(bound: &BoundBurnClearObservation) -> Result<bool, BurnClearLawError> {
    let common = &bound.observation.context.common;
    Ok(state_succession_holds(bound)?
        && common
            .projections
            .contains(&architecture::ProjectionId::ClearEvent)
        && !common
            .projections
            .contains(&architecture::ProjectionId::BurnEvent)
        && !common
            .projections
            .contains(&architecture::ProjectionId::DistributionResidue)
        && clear_flow(bound)?)
}
