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
- the encoding and evidence-requirement keys those primitives name;
- the target validator, which reports every defect rather than the first;
- the stable semantic projection.

Not implemented, and not claimed:

- the encoding specification registry;
- authorization, timelock, confidential-value, and issuance contracts;
- consensus and policy resource interfaces;
- capability contracts and evidence-requirement contents;
- the development deployment binding.

## Failure behavior is part of every primitive contract

A primitive described only by what it does when it succeeds is an incomplete
contract, because the reviewed target does not fail uniformly. Some primitives
abort evaluation; the signature primitives consume their operands and push a
false when the offered signature is empty; and the fixed-width arithmetic
primitives leave their operands in place and push a false *above* them on
overflow, so the failing path leaves a deeper stack than the succeeding one. All
three shapes are typed separately and none may be collapsed into the others.

Target-native deployment evidence has not been produced, and production target
support is not claimed.
