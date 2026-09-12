//! Bidirectional architecture/realization validation.

use std::collections::BTreeSet;

use architecture::{
    Architecture, AssetId, BoundId, DeltaCondition, DeltaKind, InputAuthorization, MaxCount,
    ObjectId, OpenFlowKind, OperationId, OperationKind, PermissionClass, ProjectionId,
    ProjectionRule, RootId, RootUse, ValueFlowClass, WitnessId,
};

use crate::{
    ArchitectureMismatchField, ConstructibilityAuthorization, ConstructibilityNodeId,
    DisclosureNode, ExpressionNode, FactId, RealizationError, Relation, RepresentationMode,
    ScopedRealizationSpec, SemanticType, require_lifecycle_exit, validate_constructibility,
};

pub fn validate_scoped_realization(
    architecture: &Architecture,
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
    realization.scope.validate_against(architecture)?;

    let scoped = realization
        .scope
        .operations()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let declared = realization
        .operations
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();

    if scoped != declared {
        return Err(RealizationError::UnsupportedOperationDeclaration(
            scoped
                .symmetric_difference(&declared)
                .copied()
                .next()
                .unwrap_or(OperationId::CompactAsh),
        ));
    }

    for operation_id in realization.scope.operations() {
        let operation = architecture.operation(*operation_id).ok_or(
            RealizationError::MissingArchitectureOperation(*operation_id),
        )?;

        for authorization in constructibility_authorizations(operation)? {
            validate_constructibility(
                &realization.constructibility_graph,
                &realization.constructibility_node_by_id,
                *operation_id,
                &authorization,
            )?;
        }
    }

    validate_relation_identities(realization)?;

    validate_architecture_family_relations(architecture, realization)?;

    validate_lifecycle_relation_weld(realization)?;

    validate_pilot_lifecycle(realization)?;

    validate_sponsor_value_opacity(realization)?;

    Ok(())
}

/// Every declared relation's identity describes its own body.
fn validate_relation_identities(
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
    for declaration in realization.operations.values() {
        for relation in &declaration.relations {
            validate_relation_identity(relation)?;
        }
    }

    Ok(())
}

/// One relation's identity, derived from its body and compared.
///
/// # Why a helper being right is not this being right
///
/// Relations are declared through constructors that pair an identity
/// with a body, and those constructors are careful. But a constructor
/// is the author's own account of what it built, and the identity is
/// what every later stage files the relation under: the compiler keys
/// coverage by [`RelationId`](crate::RelationId) while behaviour
/// classification reads the body. A relation whose key says one family
/// and whose body implements another gives two answers to the question
/// of what it is, and each later stage picks whichever answer its own
/// data structure happens to hold.
///
/// So the derivation here is the owner's, not the author's: the kind
/// comes out of the body exhaustively, with no wildcard arm, so a new
/// `Relation` variant cannot be added without deciding what it is
/// `(´[PLAN-rule:guide10:relation-identity]´)`.
///
/// # One subject, derived and compared for equality
///
/// The subject is derived the same way, and by the same argument. It
/// was once a predicate asking whether a declared subject was *among*
/// those a body could plausibly be filed under, and several bodies
/// admitted several: a closure over two families could be subjected to
/// either family, a root or projection policy to the operation or to
/// any member it governed, and the canonical-delta and open-flow
/// policies — which name no projection at all — to any projection in
/// the vocabulary. Admitting alternatives means the key can move while
/// the body stays identical, which is exactly what a complete typed key
/// must not permit.
///
/// So [`expected_relation_subject`] returns one subject and this
/// compares it for equality. A body constraining a whole side is
/// subjected to that side, and a body fixing operation-wide policy to
/// the operation `(´[PLAN-rule:guide11-exec:relation-subject]´)`.
///
/// # Errors
///
/// [`RealizationError::RelationKindMismatch`] when the body belongs to
/// another relation family, and
/// [`RealizationError::RelationSubjectMismatch`] when it belongs to
/// this family but not to the declared subject.
pub fn validate_relation_identity(
    declaration: &crate::RelationDeclaration,
) -> Result<(), RealizationError> {
    let expected = expected_relation_kind(&declaration.relation);
    if declaration.id.kind() != expected {
        return Err(RealizationError::RelationKindMismatch {
            declared: declaration.id.clone(),
            expected,
        });
    }
    let expected_subject = expected_relation_subject(&declaration.relation);
    if *declaration.id.subject() != expected_subject {
        return Err(RealizationError::RelationSubjectMismatch {
            declared: declaration.id.clone(),
            expected: expected_subject,
        });
    }

    Ok(())
}

/// The relation family one body belongs to.
///
/// Exhaustive by construction: there is no wildcard arm, so a new body
/// variant is a compile error here until its family is decided.
const fn expected_relation_kind(relation: &Relation) -> crate::RelationKind {
    use crate::RelationKind as Kind;

    match relation {
        Relation::Cardinality { .. } => Kind::Cardinality,
        Relation::AllowedObjectFamilies { .. } => Kind::AllowedObjectFamilies,
        Relation::Recognition { .. } => Kind::Recognition,
        Relation::AmountConservation { .. } => Kind::Conservation,
        Relation::OwnerAuthorization { .. }
        | Relation::PermissionlessAuthorization
        | Relation::OperatorAuthorization => {
            Kind::Authorization
        }
        Relation::SponsorIsolation => Kind::SponsorIsolation,
        Relation::SponsorEnvelopeMultiplicity { .. } => Kind::SponsorEnvelopeMultiplicity,
        Relation::RootPolicy { .. } => Kind::RootPolicy,
        Relation::ProjectionPolicy { .. } => Kind::ProjectionPolicy,
        Relation::CanonicalDeltaPolicy { .. } => Kind::CanonicalDeltaPolicy,
        Relation::OpenFlowPolicy { .. } => Kind::OpenFlowPolicy,
        Relation::Constructibility { .. } => Kind::Constructibility,
        Relation::Representation { .. } => Kind::Representation,
        Relation::LifecycleExit { .. } => Kind::Lifecycle,
        Relation::ExpressionPredicate { .. } => Kind::ExpressionPredicate,
        Relation::SubstrateConservation { .. } => Kind::SubstrateConservation,
    }
}

/// The one subject a body constrains.
///
/// Exhaustive in the same way, and for the same reason. This is a
/// function of the body rather than a predicate over candidate
/// subjects: a predicate can admit several keys for one body, and then
/// the key is a presentation choice rather than an identity
/// `(´[PLAN-rule:guide11-exec:relation-subject]´)`.
fn expected_relation_subject(relation: &Relation) -> crate::RelationSubject {
    use crate::RelationSubject as Subject;

    let family = |side: crate::ObservedSide, object: ObjectId| Subject::ObjectFamily {
        side: transaction_side(side),
        object,
    };

    match relation {
        Relation::Cardinality { side, object, .. } | Relation::Recognition { side, object, .. } => {
            family(*side, *object)
        }
        // The closure constrains a whole side. Subjecting it to one of
        // the families it admits would file the closure under a member
        // of its own result, and any other member would do as well; the
        // side is what the body is actually about.
        Relation::AllowedObjectFamilies { side, .. } => Subject::TransactionSide {
            side: transaction_side(*side),
        },
        // Owners are committed by an input family, which is the side
        // the authorization is about.
        Relation::OwnerAuthorization { object } => family(crate::ObservedSide::Input, *object),
        Relation::AmountConservation { asset, .. } | Relation::SubstrateConservation { asset } => {
            Subject::Asset { asset: *asset }
        }
        Relation::SponsorIsolation | Relation::SponsorEnvelopeMultiplicity { .. } => {
            Subject::Sponsor
        }
        Relation::Representation { object, .. } => Subject::Representation { object: *object },
        Relation::LifecycleExit { object, exit } => Subject::LifecycleExit {
            object: *object,
            exit: *exit,
        },
        // Operation-wide policies. Each fixes what the whole operation
        // may do — which roots it may use, which projections it may
        // derive, which canonical deltas and open flows it admits — and
        // none singles out a root or projection to be subjected to. An
        // expression predicate is owned by its operation for the same
        // reason: the subject vocabulary has no expression member, and
        // inventing one for a relation nothing declares yet would be
        // minting identity ahead of need.
        Relation::PermissionlessAuthorization
        | Relation::OperatorAuthorization
        | Relation::Constructibility { .. }
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::ExpressionPredicate { .. } => Subject::Operation,
    }
}

/// The identity-side spelling of an observed transaction side.
const fn transaction_side(side: crate::ObservedSide) -> crate::TransactionSide {
    match side {
        crate::ObservedSide::Input => crate::TransactionSide::Input,
        crate::ObservedSide::Output => crate::TransactionSide::Output,
    }
}

/// Every architecture-declared object family carries its semantic
/// relations (Guide-6 §4.4).
///
/// The architecture owns each family's side, cardinality bounds, and
/// asset; realization is what exposes them as the relations compiler
/// analysis consumes. Without this rule a family could keep its
/// recognition relation and silently lose its cardinality relation —
/// exactly the compact-ASH sponsor asymmetry — leaving architecture
/// facts owned by no relation while every other check still passed.
///
/// The expected census is derived from the typed operation row, never
/// from the operation's name: each declared input and output family
/// yields one expected cardinality relation and one expected
/// recognition relation, and the declared relation must equal it
/// exactly. A transposed side, a wrong bound, or a wrong recognized
/// asset therefore fails here rather than reaching the compiler.
fn validate_architecture_family_relations(
    architecture: &Architecture,
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
    for operation_id in realization.scope.operations() {
        let operation = architecture.operation(*operation_id).ok_or(
            RealizationError::MissingArchitectureOperation(*operation_id),
        )?;
        let declaration = realization
            .operations
            .get(operation_id)
            .ok_or(RealizationError::OperationOutsideScope(*operation_id))?;

        let census = expected_family_relations(architecture, operation)?;

        for (id, expected) in &census {
            let declared = declaration
                .relations
                .iter()
                .find(|declared| declared.id == *id)
                .ok_or_else(|| RealizationError::MissingArchitectureRelation {
                    operation: *operation_id,
                    kind: id.kind(),
                    subject: id.subject().clone(),
                })?;

            if declared.relation != *expected {
                return Err(RealizationError::ArchitectureRelationMismatch {
                    relation: id.clone(),
                });
            }
        }

        // And the other direction. Every relation of a family kind must
        // be one the architecture called for, so a surplus row cannot
        // travel as architecture-owned semantics for a family the
        // architecture never declared.
        let expected_ids = census
            .iter()
            .map(|(id, _)| id.clone())
            .collect::<BTreeSet<_>>();
        for declared in &declaration.relations {
            if matches!(
                declared.id.kind(),
                crate::RelationKind::Cardinality | crate::RelationKind::Recognition
            ) && !expected_ids.contains(&declared.id)
            {
                return Err(RealizationError::SurplusArchitectureRelation {
                    relation: declared.id.clone(),
                });
            }
        }
    }

    Ok(())
}

/// The cardinality and recognition relations one architecture
/// operation row requires, in canonical order.
fn expected_family_relations(
    architecture: &Architecture,
    operation: &architecture::OperationSpec,
) -> Result<Vec<(crate::RelationId, Relation)>, RealizationError> {
    let inputs = operation.inputs.iter().map(|input| {
        (
            crate::TransactionSide::Input,
            input.object,
            input.minimum,
            input.maximum,
        )
    });
    let outputs = operation.outputs.iter().map(|output| {
        (
            crate::TransactionSide::Output,
            output.object,
            output.minimum,
            output.maximum,
        )
    });
    let mut expected = Vec::new();

    for (side, object, minimum, maximum) in inputs.chain(outputs) {
        let asset = architecture
            .object(object)
            .ok_or(RealizationError::MissingArchitectureObject(object))?
            .asset;
        let subject = crate::RelationSubject::ObjectFamily { side, object };
        let observed = match side {
            crate::TransactionSide::Input => crate::ObservedSide::Input,
            crate::TransactionSide::Output => crate::ObservedSide::Output,
        };

        expected.push((
            crate::RelationId::new(
                operation.id,
                crate::RelationKind::Cardinality,
                subject.clone(),
            ),
            Relation::Cardinality {
                side: observed,
                object,
                minimum: crate::Count::new(u64::from(minimum)),
                maximum: match maximum {
                    MaxCount::Exact(exact) => {
                        crate::CardinalityMaximum::Exact(crate::Count::new(u64::from(exact)))
                    }
                    MaxCount::Bound(bound) => crate::CardinalityMaximum::Bound(bound),
                },
            },
        ));
        expected.push((
            crate::RelationId::new(operation.id, crate::RelationKind::Recognition, subject),
            Relation::Recognition {
                side: observed,
                object,
                asset,
            },
        ));
    }

    Ok(expected)
}

/// Sponsor-value opacity (F2-006): an ordinary sponsor L-BTC
/// amount — individual or aggregate — is sponsor-local data, never a
/// protocol-readable fact.
///
/// The structural read-set guard: no expression, disclosure node,
/// declassification entry, or constructibility fact may name a
/// `PLAIN_LBTC` family amount, on either side. Sponsor safety is
/// discharged by asset authentication, family recognition, exact
/// membership, owner authorization, and isolation — a backend must not
/// add an amount read merely because its target exposes an
/// introspection primitive.
///
/// Conservation is deliberately absent from that list (S3). It is the
/// substrate's: Elements validates that a transaction's inputs and
/// outputs balance, so this layer neither re-derives it nor reads the
/// amounts it would need to. The guard covers four declaration paths;
/// the sponsor-isolation evaluator is the fifth surface that could
/// reach these amounts, and it is value-blind for the same reason —
/// see `flow_role_is_exact`. A guard that covered only the
/// declaration paths would advertise a boundary the evaluator did not
/// hold.
///
/// The guard is family-shaped while sponsorship itself is a flow role
/// (S2-01), and that gap is deliberate: `FactId::FamilyAmount` names an
/// operation, a side, and a family, so no declarable fact can name the
/// sponsor region more narrowly than "ordinary L-BTC". Forbidding the
/// whole family is the conservative reading — it can only refuse a
/// protocol-role amount some later operation wants, never admit a
/// sponsor one. When the fact vocabulary gains a flow-keyed amount,
/// this guard narrows to the fee-sponsor region and protocol-role
/// L-BTC amounts become declarable under their own flow.
fn validate_sponsor_value_opacity(
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
    fn is_sponsor_amount(fact: &FactId) -> bool {
        matches!(
            fact,
            FactId::FamilyAmount {
                object: ObjectId::PlainLbtc,
                ..
            }
        )
    }

    let expression_read = realization
        .expression_graph
        .node_weights()
        .any(|declaration| {
            matches!(&declaration.node, ExpressionNode::Fact(fact) if is_sponsor_amount(fact))
        });

    let disclosure_read = realization
        .disclosure_graph
        .node_weights()
        .any(|node| matches!(node, DisclosureNode::Fact { id, .. } if is_sponsor_amount(id)));

    let constructibility_read = realization.constructibility_node_by_id.keys().any(
        |node| matches!(node, ConstructibilityNodeId::Fact { fact, .. } if is_sponsor_amount(fact)),
    );

    let declassification = &realization.declassification;
    let declassification_read = declassification
        .required_public
        .keys()
        .chain(declassification.newly_disclosed.keys())
        .chain(declassification.retained_private.iter())
        .any(is_sponsor_amount);

    if expression_read || disclosure_read || constructibility_read || declassification_read {
        return Err(RealizationError::SponsorValueRead);
    }

    Ok(())
}

/// Derives the constructibility authorization cases from one typed
/// architecture operation row. The operation name is never inspected:
/// the primary `PermissionClass` selects the case shape, and owner/refund
/// cases read the operation's input authorizations. A cadence-band
/// operation yields both its operator and delayed-permissionless windows.
pub fn constructibility_authorizations(
    operation: &architecture::OperationSpec,
) -> Result<Vec<ConstructibilityAuthorization>, RealizationError> {
    let cases = match operation.authorization {
        PermissionClass::Permissionless => vec![ConstructibilityAuthorization::Permissionless],

        PermissionClass::ReceiptOwners => vec![ConstructibilityAuthorization::InputOwners {
            objects: input_owner_objects(operation),
        }],

        PermissionClass::ClientAuthorized => {
            vec![ConstructibilityAuthorization::ClientAuthorized {
                objects: input_owner_objects(operation),
            }]
        }

        PermissionClass::RefundKey => {
            let object = operation
                .inputs
                .iter()
                .find(|input| input.authorization == InputAuthorization::RefundKey)
                .map(|input| input.object)
                .ok_or(RealizationError::ConstructibilityAuthorizationMismatch(
                    operation.id,
                ))?;

            vec![ConstructibilityAuthorization::RefundKey { object }]
        }

        PermissionClass::Operator => vec![ConstructibilityAuthorization::Operator],

        PermissionClass::CadenceBand => vec![
            ConstructibilityAuthorization::CadenceOperator,
            ConstructibilityAuthorization::CadencePermissionless,
        ],
    };

    Ok(cases)
}

/// The set of objects whose consumed owners authorize an operation.
fn input_owner_objects(operation: &architecture::OperationSpec) -> BTreeSet<ObjectId> {
    operation
        .inputs
        .iter()
        .filter(|input| input.authorization == InputAuthorization::InputOwner)
        .map(|input| input.object)
        .collect()
}

/// The lifecycle declarations one relation census requires.
///
/// Keyed by object, because a lifecycle statement is always about one
/// object family: the representations it may take, and the operations
/// it is required to be able to exit through.
#[derive(Debug, Default)]
struct LifecycleCensus {
    representations: std::collections::BTreeMap<ObjectId, BTreeSet<RepresentationMode>>,
    exits: std::collections::BTreeMap<ObjectId, BTreeSet<OperationId>>,
}

/// Weld the lifecycle graph to the lifecycle relations in both
/// directions (SR2-03).
///
/// The realization states each object's lifecycle twice: as semantic
/// relations (`Representation`, `LifecycleExit`) that the compiler's
/// relation census and coverage analysis consume, and as a lifecycle
/// graph of representation and required-exit nodes. Nothing previously
/// tied the two together generically, so a *coherent* omission — drop a
/// `LifecycleExit` relation and its relation dependency, keep the graph
/// path — validated: relation ownership passed, graph construction
/// passed, the hard-coded pilot reachability check still found its
/// path, and the compiler simply never saw the missing obligation.
///
/// The expected census is derived from the relation declarations
/// themselves, never from an operation's name, mirroring the
/// architecture family-relation rule above. Exact closure is:
///
/// - every graph node is declared by some relation;
/// - every relation-declared representation and exit has its node;
/// - every declared exit is reachable from every allowed
///   representation of the same object.
///
/// Exact *edge* equality follows rather than needing its own check: a
/// graph edge is shape-validated to run representation -> required-exit
/// within one object, duplicates are rejected at construction, and node
/// equality pins both endpoint sets — so the edge set is bounded by the
/// complete per-object product this function requires to be present.
fn validate_lifecycle_relation_weld(
    realization: &ScopedRealizationSpec,
) -> Result<(), RealizationError> {
    let census = lifecycle_census(realization);

    // Direction one: no lifecycle node outlives the relation that
    // declares it.
    for node in realization.lifecycle_node_by_id.keys() {
        let declared = match node {
            crate::LifecycleNodeId::Representation { object, mode } => census
                .representations
                .get(object)
                .is_some_and(|modes| modes.contains(mode)),
            crate::LifecycleNodeId::RequiredExit { object, operation } => census
                .exits
                .get(object)
                .is_some_and(|exits| exits.contains(operation)),
        };

        if !declared {
            return Err(RealizationError::UndeclaredLifecycleNode(node.clone()));
        }
    }

    // Direction two: every relation-declared representation and exit
    // has its node, and every declared exit is reachable from every
    // allowed representation of that object.
    for (object, exits) in &census.exits {
        let modes = census.representations.get(object);

        for exit in exits {
            if !realization.lifecycle_node_by_id.contains_key(
                &crate::LifecycleNodeId::RequiredExit {
                    object: *object,
                    operation: *exit,
                },
            ) {
                return Err(RealizationError::MissingLifecycleExitNode {
                    object: *object,
                    exit: *exit,
                });
            }

            // A required exit nothing may reach is not a weaker
            // obligation, it is an incoherent one.
            let Some(modes) = modes.filter(|modes| !modes.is_empty()) else {
                return Err(RealizationError::LifecycleExitWithoutRepresentation {
                    object: *object,
                    exit: *exit,
                });
            };

            for mode in modes {
                require_lifecycle_exit(
                    &realization.lifecycle_graph,
                    &realization.lifecycle_node_by_id,
                    *object,
                    *mode,
                    *exit,
                )?;
            }
        }
    }

    for (object, modes) in &census.representations {
        for mode in modes {
            if !realization.lifecycle_node_by_id.contains_key(
                &crate::LifecycleNodeId::Representation {
                    object: *object,
                    mode: *mode,
                },
            ) {
                return Err(RealizationError::MissingLifecycleRepresentation {
                    object: *object,
                    mode: *mode,
                });
            }
        }
    }

    Ok(())
}

/// The lifecycle statements the scoped relation declarations make.
///
/// Two operations may each declare part of one object's lifecycle, so
/// the census unions their statements rather than assuming a single
/// declaring operation.
fn lifecycle_census(realization: &ScopedRealizationSpec) -> LifecycleCensus {
    let mut census = LifecycleCensus::default();

    for declaration in realization.operations.values() {
        for relation in &declaration.relations {
            match &relation.relation {
                Relation::Representation { object, allowed } => {
                    census
                        .representations
                        .entry(*object)
                        .or_default()
                        .extend(allowed.iter().copied());
                }
                Relation::LifecycleExit { object, exit } => {
                    census.exits.entry(*object).or_default().insert(*exit);
                }
                _ => {}
            }
        }
    }

    census
}

/// Pin the two Phase-2 pilots' own lifecycle content.
///
/// This is deliberately *not* subsumed by the generic weld above, and
/// is now narrowed to that one job. The weld proves the graph and the
/// relations agree; it cannot prove they agree on the right thing,
/// because a coherent removal of both a `LifecycleExit` relation and
/// its graph path leaves a smaller but perfectly closed lifecycle. This
/// assertion is what pins the expected pilot content, so ASH keeps its
/// `CLEAR` exit and a live receipt keeps its burn and redeem exits
/// under both of their admitted representations.
///
/// It probes through `require_lifecycle_exit` rather than reading the
/// relations directly so that it states the pilot obligation in
/// lifecycle terms; the weld is what makes a passing probe evidence
/// about the relation census too.
fn validate_pilot_lifecycle(realization: &ScopedRealizationSpec) -> Result<(), RealizationError> {
    if realization.scope.contains(OperationId::CompactAsh) {
        for mode in [
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ] {
            for exit in [OperationId::CompactAsh, OperationId::Clear] {
                require_lifecycle_exit(
                    &realization.lifecycle_graph,
                    &realization.lifecycle_node_by_id,
                    ObjectId::Ash,
                    mode,
                    exit,
                )?;
            }
        }
    }

    if realization.scope.contains(OperationId::TransferLive) {
        for mode in [
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
        ] {
            for exit in [
                OperationId::TransferLive,
                OperationId::Burn,
                OperationId::Redeem,
            ] {
                require_lifecycle_exit(
                    &realization.lifecycle_graph,
                    &realization.lifecycle_node_by_id,
                    ObjectId::ReceiptLive,
                    mode,
                    exit,
                )?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub fn validate_compact_ash_architecture(
    architecture: &Architecture,
) -> Result<(), RealizationError> {
    let operation = architecture.operation(OperationId::CompactAsh).ok_or(
        RealizationError::MissingArchitectureOperation(OperationId::CompactAsh),
    )?;

    if operation.kind != OperationKind::CovenantBranch {
        return mismatch_compact(ArchitectureMismatchField::OperationKind);
    }

    if operation.authorization != PermissionClass::Permissionless {
        return mismatch_compact(ArchitectureMismatchField::Authorization);
    }

    if !operation.issuances.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::Issuances);
    }

    if !operation.reads.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::Reads);
    }

    if !operation.writes.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::Writes);
    }

    let input_families = operation
        .inputs
        .iter()
        .map(|input| input.object)
        .collect::<BTreeSet<_>>();

    if input_families != BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]) {
        return mismatch_compact(ArchitectureMismatchField::InputFamilies);
    }

    let output_families = operation
        .outputs
        .iter()
        .map(|output| output.object)
        .collect::<BTreeSet<_>>();

    if output_families != BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]) {
        return mismatch_compact(ArchitectureMismatchField::OutputFamilies);
    }

    let ash_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::Ash)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::InputFamilies))?;

    if ash_input.minimum != 2
        || ash_input.maximum != MaxCount::Bound(BoundId::AshBatchMax)
        || ash_input.authorization != InputAuthorization::Permissionless
    {
        return mismatch_compact(ArchitectureMismatchField::AshInput);
    }

    let ash_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::Ash)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::OutputFamilies))?;

    if ash_output.minimum != 1 || ash_output.maximum != MaxCount::Exact(1) {
        return mismatch_compact(ArchitectureMismatchField::AshOutput);
    }

    let sponsor_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::PlainLbtc)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::SponsorInput))?;

    if sponsor_input.minimum != 0
        || sponsor_input.maximum != MaxCount::Bound(BoundId::FeeSponsorInputMax)
        || sponsor_input.authorization != InputAuthorization::SponsorOwner
    {
        return mismatch_compact(ArchitectureMismatchField::SponsorInput);
    }

    let sponsor_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::PlainLbtc)
        .ok_or_else(|| compact_error(ArchitectureMismatchField::SponsorOutput))?;

    if sponsor_output.minimum != 0 || sponsor_output.maximum != MaxCount::Exact(1) {
        return mismatch_compact(ArchitectureMismatchField::SponsorOutput);
    }

    if operation.bounds.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([BoundId::AshBatchMax, BoundId::FeeSponsorInputMax])
    {
        return mismatch_compact(ArchitectureMismatchField::Bounds);
    }

    if operation.canonical_deltas.len() != 1 {
        return mismatch_compact(ArchitectureMismatchField::CanonicalDelta);
    }

    let delta = operation.canonical_deltas[0];
    if delta.asset != architecture::AssetId::U
        || delta.kind != DeltaKind::OwnerlessLateral
        || delta.condition != DeltaCondition::Always
        || delta.destruction_tag.is_some()
    {
        return mismatch_compact(ArchitectureMismatchField::CanonicalDelta);
    }

    if operation
        .open_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([OpenFlowKind::FeeSponsor])
    {
        return mismatch_compact(ArchitectureMismatchField::OpenFlows);
    }

    if operation
        .value_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            ValueFlowClass::OwnerlessBoundMovement,
            ValueFlowClass::SponsorEnvelope,
        ])
    {
        return mismatch_compact(ArchitectureMismatchField::ValueFlows);
    }

    for root in RootId::ALL {
        if operation.root_use(*root) != RootUse::Forbidden {
            return mismatch_compact(ArchitectureMismatchField::RootPolicy);
        }
    }

    if operation.projection_rule(ProjectionId::TransitionCertificate) != ProjectionRule::Required {
        return mismatch_compact(ArchitectureMismatchField::ProjectionPolicy);
    }

    for projection in [
        ProjectionId::BurnEvent,
        ProjectionId::ClearEvent,
        ProjectionId::DistributionResidue,
    ] {
        if operation.projection_rule(projection) != ProjectionRule::Forbidden {
            return mismatch_compact(ArchitectureMismatchField::ProjectionPolicy);
        }
    }

    if !operation.data_outputs.is_empty() {
        return mismatch_compact(ArchitectureMismatchField::DataOutputs);
    }

    if operation.witnesses.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([
            WitnessId::CanonicalDelta,
            WitnessId::AshLineage,
            WitnessId::ValueFlowClosure,
            WitnessId::UtxoLifecycle,
        ])
    {
        return mismatch_compact(ArchitectureMismatchField::Witnesses);
    }

    Ok(())
}

fn compact_error(field: ArchitectureMismatchField) -> RealizationError {
    RealizationError::ArchitectureOperationMismatch {
        operation: OperationId::CompactAsh,
        field,
    }
}

fn mismatch_compact(field: ArchitectureMismatchField) -> Result<(), RealizationError> {
    Err(compact_error(field))
}

#[allow(clippy::too_many_lines)]
pub fn validate_live_transfer_architecture(
    architecture: &Architecture,
) -> Result<(), RealizationError> {
    let operation = architecture.operation(OperationId::TransferLive).ok_or(
        RealizationError::MissingArchitectureOperation(OperationId::TransferLive),
    )?;

    if operation.kind != OperationKind::CovenantBranch {
        return mismatch_live(ArchitectureMismatchField::OperationKind);
    }

    if operation.authorization != PermissionClass::ReceiptOwners {
        return mismatch_live(ArchitectureMismatchField::Authorization);
    }

    if !operation.issuances.is_empty() {
        return mismatch_live(ArchitectureMismatchField::Issuances);
    }

    if !operation.reads.is_empty() {
        return mismatch_live(ArchitectureMismatchField::Reads);
    }

    if !operation.writes.is_empty() {
        return mismatch_live(ArchitectureMismatchField::Writes);
    }

    let input_families = operation
        .inputs
        .iter()
        .map(|input| input.object)
        .collect::<BTreeSet<_>>();

    if input_families != BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]) {
        return mismatch_live(ArchitectureMismatchField::InputFamilies);
    }

    let output_families = operation
        .outputs
        .iter()
        .map(|output| output.object)
        .collect::<BTreeSet<_>>();

    if output_families != BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]) {
        return mismatch_live(ArchitectureMismatchField::OutputFamilies);
    }

    let receipt_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::ReceiptLive)
        .ok_or_else(|| live_error(ArchitectureMismatchField::ReceiptInput))?;

    if receipt_input.minimum != 1
        || receipt_input.maximum != MaxCount::Bound(BoundId::TransferInputMax)
        || receipt_input.authorization != InputAuthorization::InputOwner
    {
        return mismatch_live(ArchitectureMismatchField::ReceiptInput);
    }

    let sponsor_input = operation
        .inputs
        .iter()
        .find(|input| input.object == ObjectId::PlainLbtc)
        .ok_or_else(|| live_error(ArchitectureMismatchField::SponsorInput))?;

    if sponsor_input.minimum != 0
        || sponsor_input.maximum != MaxCount::Bound(BoundId::FeeSponsorInputMax)
        || sponsor_input.authorization != InputAuthorization::SponsorOwner
    {
        return mismatch_live(ArchitectureMismatchField::SponsorInput);
    }

    let receipt_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::ReceiptLive)
        .ok_or_else(|| live_error(ArchitectureMismatchField::ReceiptOutput))?;

    if receipt_output.minimum != 1
        || receipt_output.maximum != MaxCount::Bound(BoundId::TransferOutputMax)
    {
        return mismatch_live(ArchitectureMismatchField::ReceiptOutput);
    }

    let sponsor_output = operation
        .outputs
        .iter()
        .find(|output| output.object == ObjectId::PlainLbtc)
        .ok_or_else(|| live_error(ArchitectureMismatchField::SponsorOutput))?;

    if sponsor_output.minimum != 0 || sponsor_output.maximum != MaxCount::Exact(1) {
        return mismatch_live(ArchitectureMismatchField::SponsorOutput);
    }

    if operation.bounds.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([
            BoundId::TransferInputMax,
            BoundId::TransferOutputMax,
            BoundId::FeeSponsorInputMax,
        ])
    {
        return mismatch_live(ArchitectureMismatchField::Bounds);
    }

    if operation
        .open_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([OpenFlowKind::FeeSponsor])
    {
        return mismatch_live(ArchitectureMismatchField::OpenFlows);
    }

    if operation
        .value_flows
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            ValueFlowClass::OwnerConsented,
            ValueFlowClass::SponsorEnvelope,
        ])
    {
        return mismatch_live(ArchitectureMismatchField::ValueFlows);
    }

    for root in RootId::ALL {
        if operation.root_use(*root) != RootUse::Forbidden {
            return mismatch_live(ArchitectureMismatchField::RootPolicy);
        }
    }

    if operation.canonical_deltas.len() != 1 {
        return mismatch_live(ArchitectureMismatchField::CanonicalDeltas);
    }

    let delta = operation.canonical_deltas[0];
    if delta.asset != AssetId::U
        || delta.kind != DeltaKind::Lateral
        || delta.condition != DeltaCondition::Always
        || delta.destruction_tag.is_some()
    {
        return mismatch_live(ArchitectureMismatchField::CanonicalDeltas);
    }

    if !operation.data_outputs.is_empty() {
        return mismatch_live(ArchitectureMismatchField::DataOutputs);
    }

    if operation.projection_rule(ProjectionId::TransitionCertificate) != ProjectionRule::Required {
        return mismatch_live(ArchitectureMismatchField::ProjectionPolicy);
    }

    for projection in [
        ProjectionId::BurnEvent,
        ProjectionId::ClearEvent,
        ProjectionId::DistributionResidue,
    ] {
        if operation.projection_rule(projection) != ProjectionRule::Forbidden {
            return mismatch_live(ArchitectureMismatchField::ProjectionPolicy);
        }
    }

    if operation.witnesses.iter().copied().collect::<BTreeSet<_>>()
        != BTreeSet::from([
            WitnessId::CanonicalDelta,
            WitnessId::ReceiptOwnerRouting,
            WitnessId::ReceiptClassClosure,
            WitnessId::ValueFlowClosure,
        ])
    {
        return mismatch_live(ArchitectureMismatchField::Witnesses);
    }

    Ok(())
}

fn live_error(field: ArchitectureMismatchField) -> RealizationError {
    RealizationError::ArchitectureOperationMismatch {
        operation: OperationId::TransferLive,
        field,
    }
}

fn mismatch_live(field: ArchitectureMismatchField) -> Result<(), RealizationError> {
    Err(live_error(field))
}

/// Generic per-operation ownership validation (F3-006).
///
/// Runs before graph assembly so a mistyped declaration fails with a
/// focused ownership error rather than a graph-shaped one, and so no
/// invariant rests on helper-constructor correctness. Checked per
/// operation: the declaration's own identity, expression IDs and
/// operands, relation IDs, relation-dependency endpoints, proof
/// alternative bindings, constructibility nodes and edge endpoints,
/// and disclosure nodes, edge endpoints, and seeds.
///
/// Deliberate cross-operation meanings stay valid because they are
/// typed payloads, not owners: a lifecycle `RequiredExit` (and the
/// `LifecycleExit` relation subject) names its exit operation as
/// semantic content while the declaring relation remains owned by the
/// declaring operation, and an architecture-owned `BoundValue` fact
/// has no owning operation at all.
pub fn validate_operation_ownership(
    requested: OperationId,
    declaration: &crate::OperationRealization,
) -> Result<(), RealizationError> {
    if declaration.operation != requested {
        return Err(RealizationError::OperationDeclarationIdentityMismatch {
            requested,
            declared: declaration.operation,
        });
    }

    for expression in &declaration.expressions {
        ensure_expression_owner(requested, &expression.id)?;
        for operand in expression_operand_ids(&expression.node) {
            ensure_expression_owner(requested, operand)?;
        }
        if let ExpressionNode::Fact(fact) = &expression.node
            && fact_owner(fact).is_some_and(|owner| owner != requested)
        {
            return Err(RealizationError::ForeignExpressionOwnership {
                operation: requested,
                expression: crate::ExprId::fact(fact.clone()),
            });
        }
    }

    for relation in &declaration.relations {
        if relation.id.operation() != requested {
            return Err(RealizationError::ForeignRelationOwnership {
                operation: requested,
                relation: relation.id.clone(),
            });
        }
        for alternative in &relation.proof_alternatives {
            if alternative.relation() != &relation.id {
                return Err(RealizationError::ForeignProofAlternativeBinding {
                    relation: relation.id.clone(),
                    foreign: alternative.relation().clone(),
                });
            }
        }
    }

    validate_predicate_bindings(requested, declaration)?;

    for dependency in &declaration.relation_dependencies {
        for endpoint in [&dependency.prerequisite, &dependency.dependent] {
            if endpoint.operation() != requested {
                return Err(RealizationError::ForeignRelationDependency {
                    operation: requested,
                    relation: endpoint.clone(),
                });
            }
        }
    }

    for node in &declaration.constructibility_nodes {
        ensure_constructibility_owner(requested, &node.id)?;
    }
    for edge in &declaration.constructibility_edges {
        ensure_constructibility_owner(requested, &edge.source)?;
        ensure_constructibility_owner(requested, &edge.target)?;
    }

    for node in &declaration.disclosure_nodes {
        ensure_disclosure_owner(requested, &node.id())?;
    }
    for edge in &declaration.disclosure_edges {
        ensure_disclosure_owner(requested, &edge.source)?;
        ensure_disclosure_owner(requested, &edge.target)?;
    }
    for seed in &declaration.disclosure_seeds {
        ensure_disclosure_owner(requested, &seed.node)?;
        for relation in seed_reason_relations(&seed.reason) {
            if relation.operation() != requested {
                return Err(RealizationError::ForeignDisclosureOwnership {
                    operation: requested,
                    node: crate::DisclosureNodeId::Relation(relation.clone()),
                });
            }
        }
        if let crate::DisclosureReason::PermissionlessConstructibility { operation, .. } =
            &seed.reason
            && *operation != requested
        {
            return Err(RealizationError::ForeignDisclosureOwnership {
                operation: requested,
                node: seed.node.clone(),
            });
        }
    }

    Ok(())
}

/// Validate expression-predicate relation bindings (F4-001).
///
/// Each `ExpressionPredicate` relation must name a declared,
/// same-operation, boolean expression. Same-operation ownership makes the
/// operation's own declared expressions the complete registry, so the
/// binding is checked here during derivation rather than deferred to
/// evaluation. Foreign predicate expressions are reported through the
/// shared ownership error.
fn validate_predicate_bindings(
    requested: OperationId,
    declaration: &crate::OperationRealization,
) -> Result<(), RealizationError> {
    let expression_types = declaration
        .expressions
        .iter()
        .map(|expression| (expression.id.clone(), expression.ty))
        .collect::<std::collections::BTreeMap<crate::ExprId, SemanticType>>();

    for relation in &declaration.relations {
        let Relation::ExpressionPredicate { expression } = &relation.relation else {
            continue;
        };

        ensure_expression_owner(requested, expression)?;

        match expression_types.get(expression) {
            None => {
                return Err(RealizationError::UnknownPredicateExpression {
                    operation: requested,
                    relation: relation.id.clone(),
                    expression: expression.clone(),
                });
            }
            Some(ty) if *ty != SemanticType::Bool => {
                return Err(RealizationError::NonBooleanPredicateExpression {
                    relation: relation.id.clone(),
                    expression: expression.clone(),
                    actual: *ty,
                });
            }
            Some(_) => {}
        }
    }

    Ok(())
}

/// Operation owning one primitive fact, if any.
///
/// `BoundValue` is architecture-owned and belongs to no operation.
fn fact_owner(fact: &FactId) -> Option<OperationId> {
    match fact {
        FactId::FamilyCount { operation, .. }
        | FactId::FamilyAmount { operation, .. }
        | FactId::StateField { operation, .. }
        | FactId::RequestedAnnouncementCycle { operation }
        | FactId::AnnouncementLead { operation, .. }
        | FactId::InputOwners { operation, .. }
        | FactId::Signers { operation }
        | FactId::ProjectionPresent { operation, .. }
        | FactId::FamilyRecognized { operation, .. }
        | FactId::SponsorIsolated { operation }
        | FactId::ProtocolSecretUsed { operation } => Some(*operation),
        FactId::BoundValue { .. } => None,
    }
}

fn expression_owner(id: &crate::ExprId) -> Option<OperationId> {
    match id {
        crate::ExprId::Fact(fact) => fact_owner(fact),
        crate::ExprId::Relation { relation, .. } => Some(relation.operation()),
    }
}

fn ensure_expression_owner(
    requested: OperationId,
    id: &crate::ExprId,
) -> Result<(), RealizationError> {
    if expression_owner(id).is_some_and(|owner| owner != requested) {
        return Err(RealizationError::ForeignExpressionOwnership {
            operation: requested,
            expression: id.clone(),
        });
    }
    Ok(())
}

fn expression_operand_ids(node: &ExpressionNode) -> Vec<&crate::ExprId> {
    match node {
        ExpressionNode::Fact(_)
        | ExpressionNode::Bool(_)
        | ExpressionNode::Count(_)
        | ExpressionNode::Amount(_) => Vec::new(),
        ExpressionNode::CheckedSum { terms, .. } | ExpressionNode::All { terms } => {
            terms.iter().collect()
        }
        ExpressionNode::Equal { left, right } | ExpressionNode::LessOrEqual { left, right } => {
            vec![left, right]
        }
        ExpressionNode::OwnerSubset {
            required,
            presented,
        } => vec![required, presented],
    }
}

fn ensure_constructibility_owner(
    requested: OperationId,
    node: &ConstructibilityNodeId,
) -> Result<(), RealizationError> {
    let owner = match node {
        ConstructibilityNodeId::Operation(operation)
        | ConstructibilityNodeId::Fact { operation, .. }
        | ConstructibilityNodeId::Witness { operation, .. } => *operation,
    };
    if owner != requested {
        return Err(RealizationError::ForeignConstructibilityOwnership {
            operation: requested,
            node: node.clone(),
        });
    }
    // A constructibility fact payload must agree with its declared
    // operation envelope.
    if let ConstructibilityNodeId::Fact { fact, .. } = node
        && fact_owner(fact).is_some_and(|owner| owner != requested)
    {
        return Err(RealizationError::ForeignConstructibilityOwnership {
            operation: requested,
            node: node.clone(),
        });
    }
    Ok(())
}

fn ensure_disclosure_owner(
    requested: OperationId,
    node: &crate::DisclosureNodeId,
) -> Result<(), RealizationError> {
    let owner = match node {
        crate::DisclosureNodeId::Fact(fact) => fact_owner(fact),
        crate::DisclosureNodeId::Relation(relation) => Some(relation.operation()),
    };
    if owner.is_some_and(|owner| owner != requested) {
        return Err(RealizationError::ForeignDisclosureOwnership {
            operation: requested,
            node: node.clone(),
        });
    }
    Ok(())
}

/// Weld every relation named by the disclosure declarations to the
/// relation census (S2).
///
/// Ownership validation runs per operation and proves only that a
/// named relation *would* belong to the declaring operation. It cannot
/// prove the relation exists, because the two graphs are assembled
/// independently. A same-operation phantom relation therefore passed
/// ownership, formed a locally well-formed disclosure graph, and could
/// carry a fact to required-public through fixed-point analysis with no
/// semantic relation owning the requirement.
///
/// Every relation reachable from a disclosure node, an edge endpoint,
/// or a seed reason must resolve in the relation census.
pub fn validate_disclosure_relation_census(
    nodes: &[crate::DisclosureNode],
    edges: &[crate::DisclosureDependencyDeclaration],
    seeds: &[crate::DisclosureSeed],
    relation_node_by_id: &std::collections::BTreeMap<
        crate::RelationId,
        petgraph::graph::NodeIndex<u32>,
    >,
) -> Result<(), RealizationError> {
    let check = |id: &crate::DisclosureNodeId| -> Result<(), RealizationError> {
        let crate::DisclosureNodeId::Relation(relation) = id else {
            return Ok(());
        };
        if relation_node_by_id.contains_key(relation) {
            return Ok(());
        }
        Err(RealizationError::UnknownDisclosureRelation {
            relation: relation.clone(),
        })
    };

    for node in nodes {
        check(&node.id())?;
    }
    for edge in edges {
        check(&edge.source)?;
        check(&edge.target)?;
    }
    for seed in seeds {
        check(&seed.node)?;
        // A reason may name a relation the graph never mentions, so
        // reasons are checked in their own right rather than through
        // the seeded node.
        for relation in seed_reason_relations(&seed.reason) {
            if !relation_node_by_id.contains_key(relation) {
                return Err(RealizationError::UnknownDisclosureRelation {
                    relation: relation.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Relations named inside one disclosure-seed reason.
fn seed_reason_relations(reason: &crate::DisclosureReason) -> Vec<&crate::RelationId> {
    match reason {
        crate::DisclosureReason::PermissionlessConstructibility { relation, .. }
        | crate::DisclosureReason::TargetSafety { relation } => vec![relation],
        crate::DisclosureReason::PublicState
        | crate::DisclosureReason::PublicEvent
        | crate::DisclosureReason::PublicRequest
        | crate::DisclosureReason::PublicInterface
        | crate::DisclosureReason::DeploymentPolicy { .. } => Vec::new(),
    }
}
