# `tripod-target-elements`

`target_elements` states the typed, reviewed Elements tapscript target
compatibility contract. It states target facts only, and nothing about the attestation contract
.

The package contract is
[plans/packages/target-elements.md](../../plans/packages/target-elements.md).

## Dependencies

None — neither first-party nor third-party. Every declaration is a
standard-library type. The crate serializes nothing, hashes nothing, parses
nothing, and opens no file.

## Boundary

The package will own the tapscript execution domain and leaf version, reviewed
opcode identities with their stack and failure contracts, field-specific
encodings, sighash and relative-timelock dimensions, confidential-value and
issuance capability descriptions, consensus and policy resource interfaces, and
the registry of target evidence a future deployment must produce.

It owns no attestation-contract operation, object, relation, proof plan,
authorization policy, batch bound, or transaction layout. It does not know which
assets are protocol closed assets, it does not select a sighash profile, and it
does not choose a cadence band or a batch bound. Those are downstream decisions.

## Review provenance is not target identity

The typed facts here are transcribed from a reviewed reading of upstream
Elements interpreter source. The upstream repository, the revision consulted,
the source paths, the node version, and the review date are recorded in
[plans/reference/elements-tapscript.md](../../plans/reference/elements-tapscript.md)
as review provenance. They never enter this crate's types or its stable
projections: the contract identifies a typed compatibility surface, not one
implementation revision.

No package parses that reference. It is review support for a human reader; the
typed Rust source here is the authority.

## Identity

The crate mints no digest. There is no target-definition hash, no
deployment-instance hash, and no field reserved for one. Direct typed comparison
of validated values is the whole comparison mechanism, and the stable contract
version carries the one compatibility decision a consumer actually makes.

## State

Implemented:

- the crate boundary and its no-dependency rule;
- the typed error root;
- the target-contract version and its supported census;
- the tapscript execution domain and the validated leaf version;
- the reviewed primitive registry, with complete operand, result, failure,
  and resource contracts for every admitted primitive;
- the field-specific encoding registry, with asset and value as independent
  axes and no global byte order;
- the literal-push contract: every push form with its opcode span and width
  field, the ordered minimal-form rule, the maximum literal size, and which
  of those rules is consensus and which is relay policy;
- signature, sighash, and relative-timelock dimensions;
- confidential-value and issuance capability descriptions;
- separate consensus and policy resource interfaces;
- the capability registry with an acyclic prerequisite relation;
- the evidence-requirement registry;
- the target validator, which reports every defect rather than the first;
- the stable semantic projection.

- the development deployment binding, its validation, and its combination
  with the contract.

## A deployment instance is not the target contract

The contract describes a compatibility surface; a binding names one network the
project intends to exercise it against. A binding never mutates the contract, so
the contract does not change when the network does.

`DeploymentEnvironment::Production` is nameable so that validation can refuse
it. No function in this crate returns a validated production binding, and a
development binding cannot be upgraded into one. An `ActivationDeclaration` is
typed input stating what a caller intends to test against — not a report, and
not an observation.

The binding carries no endpoint, username, password, cookie path, bearer token,
key, or wallet path, and none may be added. A future runner that must talk to a
node needs its own security design.

Binding a contract to a deployment proves only that the static contract is
internally valid, that the declaration is internally valid, and that the two
agree. It does not prove that the execution domain is active anywhere, that any
node behaves as described, that the network exists, or that anything is ready to
deploy.

## Support is not a boolean

A capability is `Reviewed`, `Incomplete`, or `Unsupported`, and none of the
three means deployment-evidenced. `Reviewed` means the typed static contract was
checked against upstream source; it does not mean a node was ever asked.

Three capabilities are deliberately not `Reviewed`. The sighash dimensions are
`Incomplete` because the review reached the signature primitives but not the
sighash construction, and every sighash dimension is recorded as unreviewed
rather than guessed. Whole-transaction value conservation is `Incomplete`
because it is a claim about the target's own consensus rules that no script
primitive demonstrates. Authenticated value opening is `Unsupported`, and it
must stay that way until a complete tested pattern exists: the low-level curve
and hash primitives being present is not an opening proof.

## Failure behavior is part of every primitive contract

A primitive described only by what it does when it succeeds is an incomplete
contract, because the reviewed target does not fail uniformly. Some primitives
abort evaluation; the signature primitives consume their operands and push a
false when the offered signature is empty; and the fixed-width arithmetic
primitives leave their operands in place and push a false *above* them on
overflow, so the failing path leaves a deeper stack than the succeeding one. All
three shapes are typed separately and none may be collapsed into the others.

## What this package owns, and what it does not

This crate carries target evidence *requirements* and no mutable
evidence-completion status for them. A static contract that recorded whether a
run had happened would change every time one did.

Evidence is produced and recorded separately by
`tripod-target-elements-conformance`, which runs a caller-selected
external executor against the reviewed primitive fixtures; development native
evidence exists there. Production target evidence remains absent, and
production target support is not claimed.
