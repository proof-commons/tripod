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

## The compound-proof primitive census

Guide 10 required a primitive-needs census before any prototype code: every
need a compound proof has, decided against what the reviewed target actually
offers, with no row left assumed. This is that table. "Constructor" is the
dynamic-metadata-leaf candidate; "wide floor" is the derived-limb candidate.

| Need | Constructor | Wide floor | Before V2 | Decision |
|---|---|---|---|---|
| duplicate top item | yes | yes | absent | admitted, `OP_DUP` 0x76 |
| duplicate top pair | yes | yes | absent | admitted, `OP_2DUP` 0x6e |
| copy second item | yes | yes | absent | admitted, `OP_OVER` 0x78 |
| swap two items | yes | yes | absent | admitted, `OP_SWAP` 0x7c |
| rotate short frame | yes | yes | absent | admitted, `OP_ROT` 0x7b |
| remove second item | yes | yes | absent | admitted, `OP_NIP` 0x77 |
| copy top below second | yes | yes | absent | admitted, `OP_TUCK` 0x7d |
| remove item | yes | yes | absent | admitted, `OP_DROP` 0x75 |
| remove two items | yes | yes | absent | admitted, `OP_2DROP` 0x6d |
| equality | yes | yes | absent | admitted, `OP_EQUAL` 0x87 |
| equality-and-abort | yes | yes | absent | admitted, `OP_EQUALVERIFY` 0x88 |
| verify Boolean | yes | yes | absent | admitted, `OP_VERIFY` 0x69 |
| byte concatenation | yes | no | absent | admitted, `OP_CAT` 0x7e |
| byte split/slice | yes | no | absent | admitted, `OP_SUBSTR` 0x7f |
| byte width | yes | no | absent | admitted, `OP_SIZE` 0x82 |
| bitwise selection | yes | no | absent | admitted, `OP_AND` 0x84 and `OP_XOR` 0x86 |
| byte lexicographic order | yes | no | absent | **unavailable as a primitive**; see below |
| conditional branch | no | no | absent | not admitted; see below |
| alternate stack | no | no | absent | not admitted; see below |
| indexed copy | no | no | absent | not admitted; see below |
| read below the third item | yes | no | absent | **decided against**: no primitive offers it; see below |
| streaming hash | yes | no | reviewed | unchanged |
| signed fixed-width arithmetic | yes | yes | reviewed | unchanged |
| signed comparison | no | yes | reviewed | unchanged |
| script-number conversion | yes | yes | reviewed | unchanged |
| input/output program inspection | yes | no | reviewed | unchanged |
| tweak verification | yes | no | reviewed | operand corrected in V2; see below |

Concatenation, slicing, and the bitwise operations are admitted because Elements
re-enables them: the disable list that carries them upstream has them commented
out, so they execute in tapscript rather than being refused. That is a target
fact and not an assumption from another script language, which is the whole
reason the census exists.

### Byte-lexicographic ordering is not a primitive

No reviewed primitive orders two byte strings. The fixed-width comparisons take
eight-byte signed integers and refuse anything else, and the script-number
ordering reads a number; neither orders a thirty-two byte digest. The capability
is named and carries `Unsupported` rather than being left unmentioned, so a
construction that needs canonical ordering has to confront the status.

It does not follow that the constructor candidate is rejected. Ordering is
constructible from primitives that do exist: a four-byte chunk read unsigned and
compared, applied per chunk, with the per-chunk results combined arithmetically
so the first differing chunk decides. Having the capability and being able to
build it are different claims, and only the second one holds.

### Three needs are decided as not admitted

Conditional branching is not admitted. No schedule step requires it — the
selection a canonical ordering needs is arithmetic on values in `{0,1}` and
branches nowhere — and the abstract stack validator is a linear fold with no
control stack, so admitting a branch would put a primitive in the registry whose
contract nothing could check.

The alternate stack is not admitted. The success algebra states main-stack
effects only, and no scheduled step needs the scratch space, so admitting a
mover would mean extending the algebra for a convenience.

Indexed copying is not admitted. Its operand count is chosen at run time, and a
contract whose operand list is fixed cannot state that. The schedules close
without it.

### The reach bound is three items, and every schedule is shaped by it

The three refusals above have one consequence between them, and it governs
every compound proof this crate can express. With no indexed copy, no indexed
move, and no alternate-stack transfer, the deepest item any reviewed primitive
can read is the third: no primitive in the registry declares a fourth operand,
and no successful form consumes more than three. So a program may hold at most
two computed values and still reach the next witness beneath them.

This is a property of the reviewed contracts rather than a convention, and it is
machine-checked over the whole registry rather than asserted about the
primitives somebody happened to look at. It decides whether a compound proof can
be scheduled at all: the constructor continuity proof has to keep one static
subtree root alive across an entire second constructor derivation, and it fits
only because the root is the deepest of the three values the first half retains,
every later witness lies beneath it in consumption order, and the second half
consumes it last.

The bound also refuses things. The metadata transition proof and the continuity
proof each schedule on their own, and composing the two into one program is not
currently expressible: the transition needs both metadata objects adjacent, the
continuity proof needs the static root between them, and no reviewed primitive
reaches past the third item to reorder them. That is an open finding rather than
a settled decision, and it is recorded as one.

### The tweak operand is a width, not an encoding

The tweak position of `OP_TWEAKVERIFY` was declared in the V2 registry as one
exact encoding class. That was wrong about the target in the refusing direction.
Upstream checks `vchTweak.size() != 32` and nothing else, and decides what the
thirty-two bytes mean afterwards inside `CheckPayToContract`; a streaming-hash
digest, which is exactly what a constructor program derives there, was refused
by a rule the target does not have.

The operand algebra now carries a position admitted on width alone, naming the
class the target reads an item as without making that class an admission
condition. A wrong width is still refused, and an item whose width the abstract
state has not settled still satisfies nothing. What the correction does not do
is promise the derived tweak is a valid scalar: the rare instance that is not
fails inside the curve arithmetic, which is where the target fails it, and that
residual is unchanged.

## Review provenance for the compound-proof primitives

Every primitive above was read in the upstream interpreter one at a time, in the
tapscript execution path. The repository, revision, source paths, and review
date belong in
[plans/reference/elements-tapscript.md](../../plans/reference/elements-tapscript.md)
with the rest of the review provenance, which now records the revision-2 census,
the width-only tweak correction and its interpreter lines, and the unsupported
ordering capability.

## Identity

The crate mints no digest. There is no target-definition hash, no
deployment-instance hash, and no field reserved for one. Direct typed comparison
of validated values is the whole comparison mechanism, and the stable contract
version carries the one compatibility decision a consumer actually makes.

## State

Implemented:

- the crate boundary and its no-dependency rule;
- the typed error root;
- the target-contract version and its supported census, now two revisions:
  V1 remains the historical Guide-9 contract and V2 carries the compound-proof
  primitive census together with the widened operand and success algebra;
- the tapscript execution domain and the validated leaf version;
- the reviewed primitive registry, with complete operand, result, failure,
  and resource contracts for every admitted primitive, including the
  compound-proof substrate: the ordinary stack operations, byte equality and its
  verifying form, Boolean verification, concatenation, width, slicing, and the
  bitwise combinators;
- an operand position that constrains nothing and a successful form that carries
  a declared operand through by index, which is what lets a polymorphic stack
  operation be described without inventing types for the caller's items;
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

Four capabilities are deliberately not `Reviewed`. The sighash dimensions are
`Incomplete` because the review reached the signature primitives but not the
sighash construction, and every sighash dimension is recorded as unreviewed
rather than guessed. Whole-transaction value conservation is `Incomplete`
because it is a claim about the target's own consensus rules that no script
primitive demonstrates. Authenticated value opening is `Unsupported`, and it
must stay that way until a complete tested pattern exists: the low-level curve
and hash primitives being present is not an opening proof. Canonical byte
ordering is `Unsupported` because no reviewed primitive performs it at all, as
the census above records.

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
