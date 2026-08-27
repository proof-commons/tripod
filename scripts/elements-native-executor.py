#!/usr/bin/env python3
"""Target-native executor adapter for the Guide-9 conformance protocol.

This program speaks the newline-delimited JSON executor protocol owned by
`tripod-target-elements-conformance` on its stdin and stdout, and
answers each case by executing the subject's exact script bytes through a real
Elements node: a disposable `elementsregtest` `elementsd` instance that this
process boots, drives over JSON-RPC, and destroys. Nothing here interprets a
script; the verdict is the node's.

The request carries no answer
-----------------------------
This adapter speaks protocol revision 4. It keeps the property this section
is about, which arrived in revision 3 and has not moved since -- a request
carries the execution subject and nothing about what the result should be:
no expected verdict, no expected failure class, no expected final stack, no
expected resource figure, no claim set, and no evidence class. Earlier
revisions sent
the whole fixture and required an executor to discard the expectation before
executing, and this adapter did; revision 3 makes the discipline unnecessary
by removing the field, and the two revisions are refused for each other at
the handshake rather than reconciled.

Three streams out, and nothing on stderr
---------------------------------------
The harness spawns the selected executor with two arguments and no others:
`--output`, naming the file this adapter writes its own diagnostics to, and
`--elements-output`, naming the file raw child text is quarantined in. Both
are mandatory, both name destinations the harness itself chose, and neither
is a credential or can become one -- Guide-9 section 17.6 bans `--rpc-user`,
`--rpc-password`, `--cookie`, `--token`, `--wallet`, and `--private-key`
from this boundary, and the name of a file to write diagnostics into is none
of them.

Everything else this script requires -- the `elementsd` and `elements-cli`
paths, the `--framework` path, the `--network-id` -- is the executor's own
configuration, established outside the first-party interface (ADR-015). The
`elements-native-executor.sh` launcher beside this file is that boundary: it
reads that configuration from the environment and appends the harness's own
two arguments to it.

So there are exactly three streams out of this process, and the third is
empty:

  stdout             protocol data, and nothing else.
  --output           this adapter's diagnostics, TYPED FACTS ONLY: a method
                     this adapter itself named, a child's exit status, a
                     phase, a class from the harness's own vocabulary, a
                     duration, a record number. No child byte reaches it.
  --elements-output  raw child text, quarantined: what `elements-cli` wrote
                     on its stderr, what the node answered, what this
                     adapter serialized. Every entry carries a header this
                     adapter wrote, numbering it so a typed line in
                     `--output` and the raw text it declined to quote name
                     the same record.
  stderr             nothing, ever.

Stderr used to be the diagnostic stream, on the ground that the harness
nulls it. That made the discipline depend on the reader: bytes the harness
happened not to read were treated as bytes that had not left, and an
operator running this adapter by hand got the raw child text on their
terminal all the same. A destination this adapter is *told* is the
destination does not depend on who is reading it. What can still reach
stderr is an interpreter traceback raised before the arguments have been
read; that is the interpreter's behaviour rather than this program's
statement, and it is stated here rather than fought.

The boundary runs in both directions, and the second direction is the one
that was missing. The harness does not read this process's stderr; this
process likewise does not read its OWN children's stderr into anything that
reaches a first-party record. The `elements-cli` child's stderr used to be
collapsed into an `AdapterError` note, and that note is written to protocol
records as `observed_detail` and `detail` -- so child bytes arrived in
first-party evidence by the back door while the contract above appeared to
forbid exactly that. A note now states the method and the client's exit
status, both of them fixed and typed, and says that the reason is omitted;
the reason itself goes to the elements output, behind a numbered header
(G12-R04).

For the same reason no note interpolates a configuration path. Those paths
are the operator's argv, not the target's answer, and a first-party record
is not where an operator's directory layout belongs.

Strict bounded request framing
------------------------------
The harness enforces a framing on this adapter's answers -- one nonempty
JSON object, one newline, at most `maximum + 1` bytes read per record,
blank records refused rather than skipped, unknown fields refused rather
than ignored. This adapter enforces the same framing on the requests, and
the symmetry is the point: a framing only one side applies is a framing
the exchange does not have.

Five endings are told apart, because they are five different things to
have gone wrong:

  clean end     the stream ended AT a record boundary. In the execution
                phase that is the exchange finishing and the only clean
                exit there is; in the handshake phase it is a harness
                that closed before saying anything.
  unterminated  the stream ended INSIDE a record. A harness that stopped
                writing part-way, which is not the same event as one
                that finished.
  oversized     the bound was reached with no newline in sight. At most
                `maximum + 1` bytes are ever taken, so this is a refusal
                this adapter states rather than an allocation the host
                eventually stops.
  blank         a record that says nothing. Skipped, once; a framing that
                skips them cannot tell "the harness said nothing here"
                from "the harness is finished".
  malformed     a well-framed record that is not one JSON object.

Each is a fixed spelling in the `--output` file and, except for the clean
end of the execution phase, a nonzero exit. The record itself is written
nowhere: a record refused for its framing is exactly the record whose
bytes have not been established as anything.

The bounds are the contract's rather than this file's. They are stated in
the harness's own `protocol.rs` and mirrored here as
`MAXIMUM_HANDSHAKE_REQUEST_BYTES` and `MAXIMUM_REQUEST_BYTES`, with a
cross-language test reading these lines out of this file and comparing
them; a bound only one side held would be the two-sided disagreement
protocol revision 4 was minted to end.

Disposable datadir, and who owns the cookie
-------------------------------------------
The node runs in a fresh `mkdtemp` directory that this process creates, owns,
and deletes when the exchange ends -- on end of stdin, on error, and on
SIGINT/SIGTERM. `elementsd` writes its RPC cookie inside that directory and
`elements-cli` resolves it from the same `-datadir`; the cookie is never read
by this script, never placed on a command line, never logged, and never
crosses the protocol boundary. It is disposable test-network material under
ADR-015's rule with the heading "Disposable test-network material": generated
for a regtest chain whose assets have no value, confined to disposable test
state, destroyed with its environment, and never derived from or reused as
production key material. The internal taproot key is the published BIP-341
NUMS constant, which is a public test constant with no known discrete log.

The harness supervises this whole tree as one process group and stops it
gracefully first, precisely so that the cleanup below is reachable on a
timeout: the SIGTERM handler is what removes the datadir and the cookie in it
(Guide-10 section 5.8). Two residuals stay honest about their limits. A
forceful kill -- the harness's own second step, once the bounded cleanup
interval has passed, or any kill -9 from elsewhere -- cannot be handled, so the
node and the datadir are then reclaimed by the group signal and by the host's
temporary-directory policy rather than by this script. And a datadir left by a
hard-killed run is not adopted by a later one: each run mkdtemps its own.

Provenance
----------
Five roles, kept apart, because one revision string cannot answer five
questions and a schema offering one invites whichever answer is easiest to
obtain (ADR-018).

  adapter_name/version       this script, which builds the transactions
  framework_revision         the upstream functional-test framework it
                             builds them with
  node_name/node_version     the first word, and the whole first line, of
                             `elementsd --version`
  binary_reported_revision   the revision the binary itself embeds in that
                             line, and nothing else
  intended_executed_tip      what the operator says they meant to run
  upstream_base              the upstream base that tip derives from
  included_local_topics      the local topic branches folded into it

A checkout's `git rev-parse HEAD` is never substituted for
`binary_reported_revision`. A checkout identifies intended source; it says
nothing about a binary built at some earlier time from some other state, and
the fallback that once filled the field this way was a misattribution rather
than a best effort. A binary embedding no revision reports `None`, and the
report records that this run establishes no workspace provenance.

The last three are explicit operator declarations, passed as arguments. They
are recorded as declarations and are never derived from a working tree.

Environment
-----------
The adapter states the environment it actually ran on, after the node is up:

  genesis_id    `getblockhash 0` from the node that executed, byte order as
                the node prints it
  chain_name    the chain the node was configured to run
  network_id    the identity the deployment binding names for that chain,
                passed as `--network-id` and restated only once the genesis
                above has been read from the booted node

The harness compares all of it with the validated binding before any case
executes, so a run cannot inherit a caller's label for a chain the adapter
never looked at. A dishonest adapter can still lie; what this removes is the
honest one labelling one chain with another chain's identity.

Execution model
---------------
For each case the adapter builds two real transactions on the disposable
chain:

  funding   spends an anyone-can-spend output and creates a taproot output
            whose single-leaf tree carries the fixture's exact script bytes
            at the fixture's leaf version;
  spending  spends that taproot output through the script path, with the
            witness stack set to the fixture's exact initial stack followed
            by the leaf script and the control block.

The funding transaction is broadcast and confirmed with `generateblock`, so
that the spending transaction is never subject to mempool ancestor policy --
an ancestor-limit rejection would otherwise be indistinguishable from a
script rejection. The spending transaction is offered to `testmempoolaccept`.
A policy-only rejection is retried at consensus level by asking
`generateblock` to include the raw transaction in a block, because the
contract is about consensus behaviour and standardness is not consensus.

`final_stack` and `final_altstack` are always null: `elementsd` exposes no
interpreter stack, and synthesising one would be fabricating evidence.

What the fixture states, and what this adapter supplies
-------------------------------------------------------
Several fixture fields are optional, and an absent one is the fixture saying
that only the executor can know the value: an outpoint names a funding output
this adapter created, the program of the input under validation commits to
the very script the fixture carries, and which asset a development network
issues is the network's own fact. Those are supplied here. A field that *is*
stated is a requirement, and every requirement this adapter cannot meet is
refused by name -- never approximated, because an approximated context makes
the target answer a question the fixture did not ask.

Input amounts are the one supplied value with a rule attached. The declared
outputs fix the total the inputs must carry, and no fee or change output may
be added, since an added output would move every output index a fixture
states. So the inputs the fixture leaves unstated share exactly what the
stated ones leave, and the materialised transaction conserves value with the
outputs the fixture asked for and no others.

Stated taproot constructions
----------------------------
A request may carry a complete taproot construction: an internal key, a
tree, the leaf the spend executes, an optional control block, the program the
consumed output must carry, and the outputs the transaction must carry by
role. The tree is then materialised exactly as stated, through the upstream
framework's own `taproot_construct`, and every stated value is checked
against what the framework built:

  internal key         used as given; the framework is never asked to pick
                       one, and the key it reports back is compared;
  leaves               every leaf script and leaf version byte is written as
                       stated, and the executing leaf's are compared against
                       what the framework hashed;
  shape                every branch is handed over as its own two-item list,
                       because the framework's helper splits a flat list down
                       the middle and would otherwise rebuild the tree in a
                       shape the fixture did not state. The executing leaf's
                       authenticated depth is compared against the stated
                       one, and the committed program against the stated
                       predecessor program, which together pin the tree;
  control block        derived from the tree that was built, and compared
                       against the stated one where the request states one.

Any disagreement is refused by name as an infrastructure error, never
approximated and never reported as a target verdict: a construction that was
not the stated one makes the target answer a question the fixture did not
ask. The capability is advertised only when the framework actually offers
the helper, since an adapter that claimed it and could not build a tree would
be sent work only it could refuse.

The spending transaction carries only the program-carrying outputs the
construction stated, and no change output. The explicit fee output is
neither requested nor avoidable: an Elements transaction accounts for its
fee in an output with no program, and one without it pays nothing and is
refused by relay for a reason that is not a script verdict.

Compound-prototype fixtures
---------------------------
A compound fixture is a record of its own rather than a primitive one with
extra fields: its case is a relation and a name, its outcome is a spend
verdict, and it always states a construction. The two are told apart by the
case identity, which is the one field whose shape differs, and a compound
request is answered only because this adapter advertises
`compound_prototype_fixtures` -- which it does only where the framework
offers the helper that builds a taproot commitment at all.

Both stated relations reach the same two steps. The stated tree is
materialised and checked against every value the fixture stated, exactly as
for a primitive case carrying a construction; then the predecessor output is
funded and spent through the stated executing leaf with the fixture's exact
witness. What differs is what the transaction is required to create:

  metadata constructor    one successor output carrying the stated program,
  continuity              at index zero, followed by the fee output. The
                          composed program reads the created output at that
                          stated role rather than searching for it;

  wide floor              no output of any role. The pattern reads no
                          transaction field, so the whole input value goes
                          to the fee output and the transaction carries no
                          program-carrying output at all -- which is what
                          the fixture stating none means.

Three fields a primitive fixture states are absent from a compound one and
are supplied here under a stated rule rather than guessed: the execution
domain, which a fixture stating a leaf and a control path already is; the
leaf version, which is stated once inside the executing leaf and checked
against the one version this adapter advertises; and the enforcement layer,
which is consensus, because a compound relation states that a spend stands
or does not and standardness is not that contract.

Resource observations
---------------------
`script_bytes` and `initial_stack_items` are restated from the fixture, so a
harness comparing them catches an executor that ran something else.
`transaction_weight` is read back from `decoderawtransaction`. The peak
depths, the largest element, and the validation budget are null: a validating
node exposes none of them, and a zero would report "not observed" as a
measurement.

Verdict mapping
---------------
`testmempoolaccept` reports an allowed result, or a `reject-reason`. Elements
formats a script failure as `mandatory-script-verify-flag-failed (TEXT)` at
consensus level and `non-mandatory-script-verify-flag (TEXT)` at policy
level, where TEXT is `ScriptErrorString` from `src/script/script_error.cpp`.

That wrapper does not by itself say a script ran. A taproot key-path
signature failure wears the same one, and a key-path spend offers no leaf
script and no control block, so no covenant clause executes. The wrapper
therefore opens the question and `observed_key_path_spend` settles it, from
the witness the bytes carry and the program each spent output pays: a
witness-version-one program with a one-item witness at every input is
reported as `key_path_rejection`, and anything else as
`script_path_rejection`. Both are read back from the node; nothing is
inferred from what the caller wanted.
Anything else -- a missing input, a fee-rate refusal, a value-conservation
failure -- is this adapter failing to build a transaction, not the target
reaching a verdict, and is reported as `infrastructure_error`.

TEXT is mapped to the harness's `ObservedFailureClass` by the explicit table
below. A TEXT with no entry, and a TEXT whose entry is `None` because the
node does not distinguish the classes the harness names, produce a rejection
with a null `observed_failure`: an unmapped class is reported as unmapped and
never as a plausible neighbour.

  Elements ScriptErrorString                             ObservedFailureClass
  ---------------------------------------------------- ----------------------
  Script evaluated without error but finished with a     evaluated_false
    false/empty top stack element
  Operation not valid with the current stack size        stack_underflow
  Operation not valid with the current altstack size     stack_underflow
  Arithmetic opcodes expect 8 bytes operands             invalid_operand_width
  Opcode missing or not understood                       unknown_opcode
  Attempted to use a disabled opcode                     unknown_opcode
  Data push larger than necessary                        malformed_push
  Negative locktime                                      negative_timelock
  Locktime requirement not satisfied                     unsatisfied_timelock
  Invalid Sha256 context object read                     hash_context_load
  Invalid Sha256 context object write                    hash_context_write
  Introspection opcode used without correct evaluation   introspection_context_unavailable
    context
  Introspection index out of bounds                      introspection_index_out_of_range
  EC scalar mult verify fail                             invalid_curve_relation
  Invalid Schnorr signature                              invalid_signature
  Signature must be zero for failed CHECK(MULTI)SIG      invalid_signature
    operation
  Public key is neither compressed or uncompressed       invalid_public_key_encoding
  Too much signature validation relative to witness      validation_budget_exhausted
    weight
  Taproot version reserved for soft-fork upgrades        leaf_version_rejected
  OP_CHECKMULTISIG(VERIFY) is not available in           unsupported_execution_domain
    tapscript
  Stack size must be exactly one after execution         non_singleton_final_stack
  Push value size limit exceeded                         malformed_push
  unknown error                                          malformed_script_number
  Invalid Schnorr signature size                         invalid_signature
  Script failed an OP_CHECKSIGVERIFY operation           empty_signature
  Arithmetic opcode error                                fixed_width_conversion_refused
  Script failed an OP_VERIFY operation                   false_verification
  Script failed an OP_EQUALVERIFY operation              unequal_operands
  Script failed an OP_NUMEQUALVERIFY operation           unequal_operands
  Stack size limit exceeded                              stack_size_limit_exceeded
  Script is too big                                      script_size_limit_exceeded
  Operation limit exceeded                               script_operation_limit_exceeded
  Invalid Schnorr signature hash type                    invalid_signature_hash_type
  Invalid Taproot control block size                     malformed_control_block

The three verify failures were unmapped until a compound-prototype run
reached them, and leaving them so was not the conservative choice it looked
like. The harness names both classes exactly -- one for a verifying
comparison whose operands were not equal, one for a verified operand that
was the target's false -- so reporting neither was reporting less than was
observed. It was also unanswerable: this adapter advertises failure-class
reporting, and the harness refuses a rejection that names no class from an
executor that said it distinguishes them. A composed proof rejects through
exactly these opcodes, so every refusing row of both prototype matrices
would have been refused as a malformed response rather than read as the
target verdict it is.

The last five were unmapped for the same reason and were mapped the same way:
the harness's vocabulary had no class for them, so the vocabulary gained one
rather than the adapter gaining a plausible neighbour. Each names an
observation the target makes and no existing class states -- three limits
refused on the script's own shape rather than on any value it computed, a
sighash type read and refused before verification is attempted, and a
control block whose width is not one the format defines. A table entry
naming an approximate class would have been worse than no entry, because a
report cannot tell an approximation from an observation; a table entry
naming an exact one is what makes a refusing row readable.

Four of the earlier entries are worth naming, because they were settled by reading
the interpreter rather than by guessing at a string:

  "unknown error" is what a script-number exception becomes. Minimal encoding
  is enforced by a relay flag and not at consensus, so the only script-number
  exception a consensus run can raise is an operand wider than the primitive
  reads -- which is exactly the harness's malformed-script-number class. In
  the relay lane a nonminimal operand raises the same exception, and the same
  class covers it.

  "Invalid Schnorr signature size" and "Invalid Schnorr signature" are two
  target codes under one contract cause: the reviewed failure census admits
  the offered signature being refused, and does not split refusal by width
  from refusal by verification.

  "Script failed an OP_CHECKSIGVERIFY operation" is reachable in tapscript
  only for an empty signature -- a non-empty signature that does not verify
  errors earlier with its own code -- so the generic string is precise.

  "Arithmetic opcode error" is genuinely coarse and is reported as such. It
  is raised for a fixed-width conversion whose operand is the wrong width and
  for one whose result will not fit a script number, and the reviewed
  contract names those as separate causes. The harness has a class that means
  exactly "the target refused a fixed-width conversion without saying which",
  and that is what is reported; a fixture whose contract cause is one of the
  two admits it alongside.

What consensus does not see
---------------------------
Two of the table's entries were measured to be unreachable in the consensus
lane, and are kept so that a fixture stating them at the relay layer -- where
they are real -- is classified rather than silently unmapped:

  malformed_push          a nonminimal push is standardness in tapscript and
                          is accepted at consensus. The oversized form is a
                          consensus rule and does reach this class;
  leaf_version_rejected   an unrecognised leaf version is unconditionally
                          valid at consensus under BIP-341, and is accepted.

`unknown_opcode` is reachable: an OP_SUCCESSx byte makes the script succeed
at consensus, but a byte that is neither defined nor OP_SUCCESSx is rejected
there, and so is a push whose payload runs off the end of the script -- the
target answers a malformed push and an undefined byte with one code.

Measured against `v28.99.0-6f43e3ffe730`.
"""

from __future__ import annotations

import argparse
import ctypes
from enum import Enum, auto
import hashlib
import io
import json
import os
import re
import shutil
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import time

COMMAND_NAME = "elements-native-executor"

# This adapter's own version, which is provenance for the transactions it
# builds and is not the node's version.
ADAPTER_VERSION = "2.1.0"

# The protocol revision this adapter speaks. It must match
# NATIVE_PROTOCOL_SCHEMA in the conformance package.
#
# Revision 4 is where the two sides began describing the same exchange.
# Under revision 3 this adapter wrote an observed_openings member the
# conformance package's conservation response did not declare while
# refusing undeclared members, so the harness could not read the answers
# this file produced; and the lifecycle step had no typed record over
# there at all. Revision 4 declares both. The revisions move together --
# a bump on one side alone would reproduce exactly the disagreement the
# bump exists to end (G12-R09).
#
# Revision 5 adds the confidential funding arm: a fifth operation subject
# and two response members that are NOT defaulted on the harness side. A
# revision-4 adapter therefore cannot answer a revision-5 exchange at
# all, which is the point -- the alternative is this adapter replying
# with silence in exactly the members a confidential answer lives in.
# Both implementations move together for the same recorded reason, and
# this file's constant is one half of that single change.
#
# Revision 6 widens the observed-layer vocabulary with
# `key_path_rejection`. No record shape moves; what moves is the set of
# values `observed_layer` may carry, and a revision-5 harness refuses a
# name it has never heard rather than reading it. That refusal would
# arrive as a transport failure instead of as the verdict the target
# actually reached, so the widening is numbered like every other break
# here, and both sides bump in one change.
NATIVE_PROTOCOL_SCHEMA = 6

# The reviewed tapscript leaf version.
TAPSCRIPT_LEAF_VERSION = 0xC4

# The width of an x-only key, which is also the width of one control-block
# path node.
XONLY_KEY_BYTES = 32

# The bit a control block's first byte carries the output key's parity in;
# every other bit of that byte is the leaf version.
CONTROL_PARITY_MASK = 0x01

# The published BIP-341 NUMS point, used as the taproot internal key so that
# the key path is unspendable. A public constant, not key material.
NUMS_INTERNAL_KEY_HEX = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"

# The anyone-can-spend program the disposable chain's free coins sit behind,
# and which this adapter reuses for its own change and block rewards.
ANYONE_CAN_SPEND_HEX = "51"

# The witness-carrying form of that same program: a version-0 witness
# script hash committing to it.
#
# # Why an issuance cannot be funded from the bare program
#
# The target only checks an issuance when the transaction carries a
# witness section. `VerifyAmounts` returns false outright when the
# issuing input has no witness entry -- `src/confidential_validation.cpp`,
# the `i >= tx.witness.vtxinwit.size()` guard ahead of the issuance's
# `VerifyIssuanceAmount` call -- so the issued asset's pseudo-input is
# never added to the input side of the balance at all. The caller sees
# that as `bad-txns-in-ne-out` from `Consensus::CheckTxInputs`, on a
# transaction whose explicit amounts balance in every asset.
#
# The section cannot simply be added empty: the target refuses to encode
# a witness whose every entry is empty (`Superfluous witness record`),
# an explicit issuance may not carry a range proof, and an explicit
# output may not carry a range or surjection proof. The only field left
# that may be filled is a spending input's witness stack, which the bare
# program never has. So the coin that funds the issuance is paid to this
# program instead, and the issuing input spends it by naming the
# committed program on its stack (T4-012).
ANYONE_CAN_SPEND_WITNESS_HEX = "0020" + hashlib.sha256(
    bytes.fromhex(ANYONE_CAN_SPEND_HEX)
).hexdigest()

# The stack that spends it: the committed program, and nothing else. No
# key, seed, or signature is involved `(ADR-015 rule test-material)`.
ANYONE_CAN_SPEND_WITNESS_STACK = (bytes.fromhex(ANYONE_CAN_SPEND_HEX),)

# Explicit-amount prefix for an Elements asset, value, or nonce field.
EXPLICIT_PREFIX = 0x01

# Satoshis committed to each case's taproot output, and the fee each
# adapter-built transaction pays. Both are far above any relay threshold on a
# chain configured with -minrelaytxfee=0.
CASE_FUNDING_SATOSHIS = 100_000
ADAPTER_FEE_SATOSHIS = 1_000

# How many whole units the Guide-12 section 16.2 funding ceremony issues
# its disposable asset in.
#
# One percent of the target's own money bound. The bound itself was the
# first choice and the target refused it: a transaction's outputs may not
# total more than the bound, and the chain's free coin already holds the
# whole of it, so an issuance of the full supply cannot share a
# transaction with the coin that pays its fee. `prepare` parks all but a
# working slice of that coin for the same reason.
#
# A hundredth of the bound is far above every amount the sponsorless
# fixture census asks for except one, and that one asks for more than the
# bound itself -- which is a fact about the fixture and is left to be
# refused by the target rather than papered over by issuing more.
#
# The issuance is a single transaction on a disposable chain and the
# units authorize nothing `(ADR-015 rule test-material)`.
ISSUED_ASSET_UNITS = 210_000

# The policy-asset slice the operation lane keeps spendable, in satoshis.
#
# Large enough to pay the fee of every transaction the lane builds, and
# small enough that it plus an issuance stays inside the target's money
# bound with room to spare.
OPERATION_WORKING_SATOSHIS = 1_000_000_000

# The secret this adapter authorizes sponsor coins with.
#
# # Why a fixed value is the right one here
#
# It is a published constant on a chain this process creates and
# destroys. It holds nothing anybody settles on, it is not derived from
# and does not derive any production material, and an authorization made
# with it is meaningful only against the disposable genesis this run
# pinned `(ADR-015 rule test-material)`. Writing it here rather than
# generating one keeps a run reproducible: the sponsor program is then
# the same program every time, so two runs of the same census produce
# the same bytes.
#
# # Why this adapter holds a key at all
#
# Because the packages that build a sponsored transaction deliberately
# do not. The candidate constructs the transaction and asks for an
# authorization of one input; a constructor able to produce that
# authorization itself could produce one nobody asked for. So the key
# lives on this side of the boundary, where its whole scope is a
# regtest chain.
#
# # Why it is derived rather than drawn
#
# The value used to be a pattern somebody chose, and a chosen constant
# asks a reader to accept that the roll was fair. So it is computed
# instead, from a string published beside it: the digest of that string,
# reduced into the group's scalar range. Nobody has to trust the choice,
# because anybody can repeat the computation, and the reduction is the
# obvious one that cannot land on zero. The internal key above is the
# same pattern, being the digest of the curve generator's encoding.
#
# The constant remains constant, so a run is as reproducible as it ever
# was. What changed with the derivation is the key itself, so the
# sponsored ceremonies already recorded in the corpus were authorized by
# the previous value and a fresh run will not reproduce their bytes.
# That costs nothing a reader can check: those recordings publish their
# transactions, and every identifier and signature hash rendered beside
# them is derived from or verified against those published bytes.

# The order of the secp256k1 group, from the curve's specification.
SECP256K1_GROUP_ORDER = (
    0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
)

SPONSOR_SECRET_DERIVATION = "tripod/native-executor/sponsor-secret/v1"
SPONSOR_SECRET_HEX = format(
    int.from_bytes(hashlib.sha256(SPONSOR_SECRET_DERIVATION.encode()).digest(), "big")
    % (SECP256K1_GROUP_ORDER - 1)
    + 1,
    "064x",
)

# The sighash type a sponsor authorization commits under.
#
# `SIGHASH_ALL`, which commits to every input and every output. The
# protocol names exactly this profile and no other: one committing to
# fewer outputs would let an authorization be replayed against a
# transaction whose protected outputs differ.
SPONSOR_SIGHASH_ALL = 1

# The boundary between a lock time counted in blocks and one counted in
# seconds, and the bit layout of a sequence field's relative lock. All four
# are the target's own constants, restated here because the adapter has to
# mature a chain far enough for the declared transaction to be final -- and a
# transaction refused as non-final is not a script verdict.
LOCKTIME_HEIGHT_THRESHOLD = 500_000_000
SEQUENCE_DISABLE_FLAG = 0x8000_0000
SEQUENCE_TIME_MODE_FLAG = 0x0040_0000
SEQUENCE_AGE_MASK = 0x0000_FFFF
SEQUENCE_TIME_GRANULARITY_BITS = 9

# The median a relative time lock is compared against is taken over this many
# blocks, so advancing it means mining that many at the later time.
MEDIAN_TIME_BLOCKS = 11

# The descriptor the maturity blocks pay to: the same anyone-can-spend
# program the chain's free coins sit behind, which keeps every coin this
# adapter creates spendable by the adapter and by nobody who cares.
MINING_DESCRIPTOR = "raw(%s)" % ANYONE_CAN_SPEND_HEX

# Every defect this adapter implements. A row naming anything else is
# refused rather than run undamaged.
KNOWN_CONSERVATION_DEFECTS = (
    "none",
    "one_unit_imbalance",
    "wrong_blinder_sum",
    "malformed_range_proof",
    "malformed_surjection_proof",
    "wrong_explicit_asset",
    "copied_commitment_from_other_asset",
    "hidden_confidential_output",
)

# An asset identifier the disposable chain never issued, used by the
# wrong-asset conservation row. A transaction paying it is constructible
# and cannot balance, which is exactly what that row states.
FOREIGN_ASSET_HEX = "5a" * 32

# Every mutation of the Guide-11 section 10.4 threat matrix this adapter
# implements. Refused by name rather than skipped, for the reason
# `G11-W7-07` records: a misspelled defect constant once ran a row
# undamaged and the target accepting a valid transaction was filed as the
# target accepting a broken one.
KNOWN_NORMALIZATION_MUTATIONS = (
    "none",
    "amount_changed",
    "owner_changed",
    "asset_changed",
    "wrong_blinding_balance",
    "hidden_private_output",
    "extra_output_after_signing",
    "output_mutated_after_signing",
    "unauthorized_representation_change",
)

# The address type every normalization coin and destination uses.
#
# Taproot, and not by preference. Guide 11 section 10.3 requires the
# owner's authorization to commit to the finalized output set, and the
# reviewed digest reaches that profile only on the taproot path
# (´[PLAN-tab:elements-ref:ct-sighash]´). A key-path spend of this
# address type
# is signed with a bare 64-byte Schnorr signature carrying no trailing
# sighash byte, and the reviewed digest reads a missing byte as the
# default all-outputs non-anyone-can-pay mode -- so the profile is
# observable in the witness rather than asserted by this adapter.
NORMALIZATION_ADDRESS_TYPE = "bech32m"

# The fee a normalization transaction declares.
#
# Deliberately larger than `ADAPTER_FEE_SATOSHIS`. The row that appends an
# output after signing funds it from the declared fee so that value still
# conserves -- and the remaining fee has to stay comfortably relayable, or
# the transaction would be refused for its fee rate and the signature
# evidence the row exists for would never be reached.
NORMALIZATION_FEE_SATOSHIS = 20_000

# What an output appended after signing takes from the declared fee.
NORMALIZATION_EXTRA_OUTPUT_SATOSHIS = 5_000

# How much an amount-changing mutation moves, and how much a hidden output
# absorbs. Both are stated by the fixture; these are the adapter's own
# refusal to guess when a fixture omits one.
NORMALIZATION_AMOUNT_DELTA = 1_000_000
NORMALIZATION_HIDDEN_AMOUNT = 1_000_000

# The confidential predecessor fixture recipe, carried on this side of
# the wire.
#
# # Why this adapter holds a fixture catalogue at all
#
# The confidential request carries a handle, a digest, ordered
# destination programs, and the profiles, and it carries no amount and no
# opening -- there is no member an amount could be written into. What
# each destination HOLDS is therefore a fact of the registered fixture,
# and a materializer that could not resolve the fixture could not
# materialize anything. So both sides hold the same public catalogue and
# the digest is what detects them drifting apart: this adapter recomputes
# the digest from its own catalogue and its own arithmetic, and answers a
# request whose digest differs by refusing it rather than by materializing
# something else.
#
# Every value below is public disposable test material on a chain this
# process creates and destroys `(ADR-015 rule test-material)`. Nothing
# here is a secret, nothing here is retained, and nothing here authorizes
# anything anywhere else.
CONFIDENTIAL_DERIVATION_TAG = b"tripod/guide-ctf/derive/v1"
CONFIDENTIAL_DIGEST_TAG = b"tripod/guide-ctf/fixture-digest/v1"

# The one digit in the handle grammar's whole spelling.
CONFIDENTIAL_GRAMMAR_VERSION = 1

# The bounded counters. Search moves upward from zero without wrapping,
# without skipping, without randomness, and without concurrency, and
# exhaustion is a refusal rather than a retry with a different source.
CONFIDENTIAL_MAX_PARITY_COUNTER = 4095
CONFIDENTIAL_MAX_SCALAR_COUNTER = 255

# The order of the group every opening scalar lives in, which is the
# curve's group order stated once above.
CONFIDENTIAL_GROUP_ORDER = SECP256K1_GROUP_ORDER

# The serialized value-commitment prefixes the target admits, in the
# order the slice requires them: one square y, then one non-square.
CONFIDENTIAL_VALUE_PREFIXES = (8, 9)

# The rangeproof shape every output of this slice carries.
#
# A zero exponent and fifty-two minimum bits, which is the widest range
# the semantic amount domain needs and the shape the target's own
# blinding path defaults to. Neither is a consensus requirement; both are
# fixed so that two runs of the same fixture produce the same bytes.
CONFIDENTIAL_RANGEPROOF_EXPONENT = 0
CONFIDENTIAL_RANGEPROOF_MINIMUM_BITS = 52

# The lower bound a proof over a SPENDABLE program must prove.
#
# One, and not zero, and this is a consensus rule rather than a
# preference: the target refuses a rangeproof whose proven minimum is
# zero unless the program it pays is unspendable, so a proof that is
# otherwise valid is rejected for that alone
# (`src/script/sigcache.cpp:157-161`). The target's own blinding path
# states the same rule from the other side, choosing zero exactly when
# the program is unspendable (`src/blind.cpp:264`). Every program this
# slice's fixtures pay is spendable, so the bound is one.
CONFIDENTIAL_RANGEPROOF_MINIMUM_VALUE = 1

# The widest proof the library can emit, which is the buffer this adapter
# offers it.
CONFIDENTIAL_RANGEPROOF_CAPACITY = 5134

# The derivation roles, by their transcript codes.
CONFIDENTIAL_ROLE_VALUE_BLINDER = 1
CONFIDENTIAL_ROLE_NONCE_SECRET = 2
CONFIDENTIAL_ROLE_RANGEPROOF_SEED = 3

# The output roles, by their transcript codes. Exactly one output of a
# transaction is balancing and every other is primary.
CONFIDENTIAL_OUTPUT_ROLES = {"primary": 1, "balancing": 2}

# The one derivation profile and the one material class this adapter
# reads, by their transcript codes.
CONFIDENTIAL_DERIVATION_PROFILE_CODE = 1
CONFIDENTIAL_MATERIAL_CLASS_CODE = 1

# Every registered case, by handle.
#
# The amounts and roles are the fixture's; the asset and the ordered
# programs arrive in the request. A handle this table does not hold is an
# unknown handle and is refused before any cryptographic work.
CONFIDENTIAL_FIXTURE_CATALOGUE = {
    "ctf-v1/predecessor-dual-parity": {
        "retry_limit": CONFIDENTIAL_MAX_PARITY_COUNTER,
        # The funding input is explicit, so it contributes a zero value
        # blinder. It is stated rather than assumed, because the
        # balancing solve is stated once, generally, and this is one
        # instance of it.
        "input_blinder_sum": bytes(32),
        "outputs": (
            {"role": "primary", "semantic_amount": 700_000_000},
            {"role": "balancing", "semantic_amount": 300_000_000},
        ),
    },
    # The three-output predecessor whose coins do NOT cancel in any pair.
    #
    # Funded exactly as the dual-parity one is -- one explicit input, the
    # same zero input blinder sum, the same profiles -- and different in
    # its output count alone. That is the whole of the arithmetic: two
    # blinders summing to zero are ordered additive inverses and cancel,
    # while three summing to zero cancel in no pair, any two of them
    # summing to the negation of the third.
    #
    # The amounts sum to the same total the dual-parity fixture's do, so
    # the issuing step funds what it funded before and the two are
    # alternatives rather than a larger ceremony.
    "ctf-v1/predecessor-triple-noncanceling": {
        "retry_limit": CONFIDENTIAL_MAX_PARITY_COUNTER,
        "input_blinder_sum": bytes(32),
        "outputs": (
            {"role": "primary", "semantic_amount": 700_000_000},
            {"role": "balancing", "semantic_amount": 200_000_000},
            {"role": "primary", "semantic_amount": 100_000_000},
        ),
    },
    # The sponsor's own reserve coins, whose VALUES are committed.
    #
    # # Why a sponsor coin needs a fixture at all
    #
    # Because nothing on the chain states its amount. An explicit sponsor
    # coin is read back from the node and the caller learns what it holds
    # by asking; a committed one is read back as a commitment, and the
    # amount behind it exists only where it was written down. This table
    # is where it is written down, and the digest is what binds the
    # caller's copy of that number to this one.
    #
    # # Why there are TWO outputs and not one
    #
    # The funding input is the adapter's own change coin, which is
    # explicit and therefore contributes a zero value blinder. A single
    # committed output would have to carry a zero blinder to close that
    # sum, and a commitment under a zero blinder is a point anybody
    # recomputes from a guessed amount -- the form of a blinded output
    # with none of the hiding. So the coin that gets SPENT is a primary
    # whose blinder is derived, and a second output solves the balance.
    #
    # # Why both amounts are the same number
    #
    # So that the pair witnesses the thing a commitment is for. The two
    # outputs carry equal semantic amounts under different blinders, so
    # their serialized commitments differ; a reader who could recover an
    # amount from a commitment would find these two identical, and they
    # are not.
    #
    # The primary's amount is the sponsor offer of the with-change shape
    # -- the fee it pays plus the change it takes back -- because that is
    # the coin the sponsored ceremony spends. The reserve sub-equation is
    # held on the caller's side, where the shape states it.
    "ctf-v1/sponsor-reserve-dual-parity": {
        "retry_limit": CONFIDENTIAL_MAX_PARITY_COUNTER,
        # Explicit input, so the same zero the predecessors state.
        "input_blinder_sum": bytes(32),
        "outputs": (
            {"role": "primary", "semantic_amount": 1_250},
            {"role": "balancing", "semantic_amount": 1_250},
        ),
    },
}


def confidential_prefixes_admitted(prefixes: tuple) -> bool:
    """Whether a case's serialized commitment prefixes satisfy the rule
    its arity states.

    Two outputs are held to the admitted pair IN FIXED ORDER, so the
    dual-parity predecessor carries one of each and exercises both
    admitted forms rather than one of them twice. Every other width is
    held to membership: each commitment carries one of the two admitted
    prefixes, which is the whole of what the reviewed target contract
    states about a third output or a lone one. There is no reviewed
    fixed-order convention past the pair, and inventing one here would be
    this adapter deciding a question the contract does not answer.

    This mirrors `prefixes_match` in the first-party fixture registry and
    the same rule on the record side, deliberately: the two sides derive
    independently and the digest is what detects them drifting apart, so
    a rule held on one side alone would surface as a digest mismatch
    rather than as the disagreement it is.
    """
    if len(prefixes) == 2:
        return prefixes == CONFIDENTIAL_VALUE_PREFIXES
    return all(prefix in CONFIDENTIAL_VALUE_PREFIXES for prefix in prefixes)


def confidential_tagged_hash(tag: bytes, message: bytes) -> bytes:
    """The published tagged-hash construction: the tag's digest twice, then
    the message.

    Prefixing with a fixed-width pair rather than the tag's own bytes is
    what makes the domain separation independent of the tag's length.
    """
    prefix = hashlib.sha256(tag).digest()
    return hashlib.sha256(prefix + prefix + message).digest()


def confidential_framed(parts: list, value: bytes) -> None:
    """One byte string, framed by its own width.

    Big-endian, four bytes, always present. Framing every member is what
    stops two transcripts colliding by concatenation.
    """
    parts.append(struct.pack(">I", len(value)))
    parts.append(value)


def confidential_derivation_preimage(
    handle: str, index: int, role: int, parity: int, scalar: int
) -> bytes:
    """The preimage one derived value is taken over.

    The case identity, the output index, the derivation role, the parity
    counter, and the scalar counter, each framed or fixed-width, so that
    no two preimages can collide.
    """
    parts = [struct.pack(">H", CONFIDENTIAL_GRAMMAR_VERSION)]
    confidential_framed(parts, handle.encode("utf-8"))
    parts.append(struct.pack(">I", index))
    parts.append(bytes([role]))
    parts.append(struct.pack(">H", parity))
    parts.append(bytes([scalar]))
    return b"".join(parts)


def confidential_derive(handle: str, index: int, role: int, parity: int, scalar: int) -> bytes:
    """One derived value, before it is read as anything."""
    return confidential_tagged_hash(
        CONFIDENTIAL_DERIVATION_TAG,
        confidential_derivation_preimage(handle, index, role, parity, scalar),
    )


def confidential_search_scalar(handle: str, index: int, role: int, parity: int) -> bytes:
    """One role's scalar for one output, searched upward from zero.

    Admitted means nonzero and below the group order. Exhaustion raises
    rather than falling back to a different source.
    """
    for scalar in range(CONFIDENTIAL_MAX_SCALAR_COUNTER + 1):
        raw = confidential_derive(handle, index, role, parity, scalar)
        value = int.from_bytes(raw, "big")
        if 0 < value < CONFIDENTIAL_GROUP_ORDER:
            return raw
    raise AdapterError(
        "the scalar search for role %d of output %d reached its bound" % (role, index)
    )


def confidential_digest_transcript(
    handle: str,
    profiles: dict,
    retry_limit: int,
    parity: int,
    asset: bytes,
    outputs: tuple,
    programs: list,
    openings: list,
) -> bytes:
    """The transcript the fixture digest is taken over.

    The contract tag sits INSIDE the transcript rather than beside it,
    which is what makes a semantic-only recorded-randomness digest
    impossible to mistake for a byte-identity one. The members that exist
    only under byte identity -- the counter, the amounts, the openings,
    and the resulting prefixes -- are written only when the openings are
    there to write.
    """
    parts = [struct.pack(">H", CONFIDENTIAL_GRAMMAR_VERSION)]
    confidential_framed(parts, handle.encode("utf-8"))
    confidential_framed(parts, profiles["reproducibility_contract"].encode("utf-8"))
    confidential_framed(parts, profiles["representation"].encode("utf-8"))
    confidential_framed(parts, profiles["custody"].encode("utf-8"))
    confidential_framed(parts, profiles["materializer"].encode("utf-8"))
    parts.append(bytes([CONFIDENTIAL_DERIVATION_PROFILE_CODE]))
    parts.append(bytes([CONFIDENTIAL_MATERIAL_CLASS_CODE]))
    parts.append(struct.pack(">H", retry_limit))
    if openings is None:
        parts.append(bytes([0]))
    else:
        parts.append(bytes([1]))
        parts.append(struct.pack(">H", parity))
    parts.append(struct.pack(">I", len(outputs)))
    for index, output in enumerate(outputs):
        parts.append(struct.pack(">I", index))
        parts.append(bytes([CONFIDENTIAL_OUTPUT_ROLES[output["role"]]]))
        confidential_framed(parts, asset)
        confidential_framed(parts, programs[index])
        if openings is not None:
            opening = openings[index]
            parts.append(struct.pack(">Q", output["semantic_amount"]))
            confidential_framed(parts, opening["value_blinder"])
            confidential_framed(parts, opening["nonce_input"])
            confidential_framed(parts, opening["rangeproof_seed"])
            parts.append(bytes([opening["value_commitment"][0]]))
    return b"".join(parts)


# The schema of the Guide-11 section 13 public handoff record.
#
# The whole point of naming it is that a record of another shape is
# refused rather than read leniently: the record is the entire channel
# between the creating process and the unrelated one, and a field this
# schema does not know is either a mistake or a covert channel.
FRESH_PROCESS_HANDOFF_SCHEMA = "tripod-fresh-process-handoff-1"

# Every field the handoff record may carry. All of it is canonical public
# chain data or the public encoding of an address. There is no key, no
# blinding factor, no descriptor, no wallet, no seed, and no data
# directory, and `FRESH_PROCESS_HANDOFF_BANNED_SUBSTRINGS` refuses one by
# name so that a future field cannot arrive unnoticed.
FRESH_PROCESS_HANDOFF_FIELDS = (
    "schema",
    "chain_name",
    "network_id",
    "genesis_id",
    "txid",
    "output_index",
    "block_hash",
    "block_height",
    "raw_transaction",
    "claimed_explicit_amount",
    "claimed_explicit_asset",
    "claimed_owner_address",
)

# A field whose NAME contains any of these is refused even if it were
# somehow admitted above. Redundant on purpose: the allow-list is the
# real rule, and this is the rule that would still fire if somebody
# widened the allow-list without thinking about what they were widening
# it for.
FRESH_PROCESS_HANDOFF_BANNED_SUBSTRINGS = (
    "key",
    "blind",
    "seed",
    "secret",
    "priv",
    "mnemonic",
    "descriptor",
    "wallet",
    "xprv",
    "nonce",
    "datadir",
)

# What the fresh process pays itself in its own future construction.
# Small: the transaction's value is that it exists and confirms.
LIFECYCLE_SPONSOR_SPEND_SATOSHIS = 100_000

# Prefixes Elements puts in front of a script error in a rejection reason.
CONSENSUS_SCRIPT_PREFIX = "mandatory-script-verify-flag-failed ("
POLICY_SCRIPT_PREFIX = "non-mandatory-script-verify-flag ("

# The two observed layers that are NOT verdicts of the target. A response
# at either may carry no target observation at all -- not an outpoint, not
# an identity, and not a resource figure (`G12-R14`). Stated here so the
# rule is applied from one list rather than restated at each writer.
NON_VERDICT_LAYERS = frozenset(
    {"fixture_construction_failure", "executor_infrastructure_failure"}
)

# The mapping documented in the module docstring. A key absent from this
# table, and a key whose value is None, both mean "this adapter did not
# classify the rejection".
FAILURE_CLASS_BY_SCRIPT_ERROR = {
    "Script evaluated without error but finished with a false/empty top stack element": "evaluated_false",
    "Operation not valid with the current stack size": "stack_underflow",
    "Operation not valid with the current altstack size": "stack_underflow",
    "Arithmetic opcodes expect 8 bytes operands": "invalid_operand_width",
    "Opcode missing or not understood": "unknown_opcode",
    "Attempted to use a disabled opcode": "unknown_opcode",
    "Data push larger than necessary": "malformed_push",
    "Negative locktime": "negative_timelock",
    "Locktime requirement not satisfied": "unsatisfied_timelock",
    "Invalid Sha256 context object read": "hash_context_load",
    "Invalid Sha256 context object write": "hash_context_write",
    "Introspection opcode used without correct evaluation context": "introspection_context_unavailable",
    "Introspection index out of bounds": "introspection_index_out_of_range",
    "EC scalar mult verify fail": "invalid_curve_relation",
    "Invalid Schnorr signature": "invalid_signature",
    "Signature must be zero for failed CHECK(MULTI)SIG operation": "invalid_signature",
    "Public key is neither compressed or uncompressed": "invalid_public_key_encoding",
    "Too much signature validation relative to witness weight": "validation_budget_exhausted",
    "Taproot version reserved for soft-fork upgrades": "leaf_version_rejected",
    "OP_CHECKMULTISIG(VERIFY) is not available in tapscript": "unsupported_execution_domain",
    # The final stack was not the single item the reviewed domain requires.
    "Stack size must be exactly one after execution": "non_singleton_final_stack",
    # A literal the target refuses outright, which is what the harness's
    # malformed-push class names.
    "Push value size limit exceeded": "malformed_push",
    # The generic code a script-number exception becomes. At consensus the
    # only script-number exception is an operand wider than the primitive
    # reads: minimal encoding is a relay rule and is not enforced there at
    # all, so this string has one meaning in the consensus lane and the
    # same one in the relay lane, where a nonminimal operand joins it.
    "unknown error": "malformed_script_number",
    # A signature of the wrong width, which the reviewed contract classes
    # with a signature that does not verify: both are the target refusing
    # the offered signature, and the contract's failure census draws no
    # line between them.
    "Invalid Schnorr signature size": "invalid_signature",
    # The verifying forms reach this only for an empty signature: a
    # non-empty signature that does not verify errors earlier with its own
    # code, so the generic verify-failure string is precise here.
    "Script failed an OP_CHECKSIGVERIFY operation": "empty_signature",
    # Deliberately coarse, and named as such. Elements answers a conversion
    # whose operand is the wrong width and one whose result will not fit a
    # script number with this one string, and the reviewed contract names
    # those as separate causes. Reporting either would name a cause this
    # adapter did not observe.
    "Arithmetic opcode error": "fixed_width_conversion_refused",
    # The two verifying comparisons, which the harness names exactly. A
    # composed proof rejects through these more often than through anything
    # else: every equality a schedule verifies is one of them.
    "Script failed an OP_EQUALVERIFY operation": "unequal_operands",
    "Script failed an OP_NUMEQUALVERIFY operation": "unequal_operands",
    # A verified operand that was the target's false, which is what an
    # unsatisfied arithmetic success flag becomes.
    "Script failed an OP_VERIFY operation": "false_verification",
    # The three limits the target refuses on the script's own shape rather
    # than on a value it computed: too many stack items across both
    # stacks, a script longer than it executes, and more operations than
    # it admits. None of them is a budget the execution spent, and none is
    # a result too wide, so each has its own class.
    "Stack size limit exceeded": "stack_size_limit_exceeded",
    "Script is too big": "script_size_limit_exceeded",
    "Operation limit exceeded": "script_operation_limit_exceeded",
    # The sighash type is read from the signature's trailing byte before
    # verification is attempted, so a byte outside the admitted set is not
    # a signature that failed to verify over a message the target
    # computed.
    "Invalid Schnorr signature hash type": "invalid_signature_hash_type",
    # A control block whose length is not one the format defines. The
    # target never reaches a leaf version to judge, so this is not the
    # refused-leaf-version class.
    "Invalid Taproot control block size": "malformed_control_block",
}


class ConstructionError(Exception):
    """The adapter could not build the stated transaction.

    Kept apart from `AdapterError` because Guide 11 section 8.3 requires a
    report to distinguish a fixture that could not be built from an
    environment that failed around a run, and both from a target verdict.
    A construction failure means the target was never asked -- reporting it
    as a rejection would manufacture a consensus fact out of a limitation
    of this materializer.
    """

    def __init__(self, note: str) -> None:
        super().__init__(note)
        self.note = note


class AdapterError(Exception):
    """A case this adapter could not run, with a note naming why.

    Every instance becomes one `infrastructure_error` response. It is never
    a target verdict: the two are kept apart on purpose (Guide-9 section 11.6).

    `client_detail` is what the `elements-cli` child wrote on its stderr,
    and it is deliberately NOT part of the note. The note is written to
    first-party protocol records, and a child's stderr is not first-party
    evidence (G12-R04). The detail exists because one caller has a
    legitimate need for those bytes -- the consensus judgement reads the
    target's script error out of them -- and that caller CLASSIFIES them
    into the typed failure vocabulary rather than propagating them. An
    unmapped message becomes no class at all; it never becomes a record's
    text by another route.
    """

    def __init__(self, note: str, client_detail: str = "") -> None:
        super().__init__(note)
        self.note = note
        self.client_detail = client_detail


class DiagnosticOutcome(Enum):
    """Every outcome the typed diagnostic stream admits."""

    RPC_CLIENT_FAILED = auto()
    FRAMEWORK_CONSTRUCTION_FAILED = auto()
    ADAPTER_TRANSACTION_REFUSED = auto()
    MEMPOOL_REASON_QUARANTINED = auto()
    TARGET_TRANSACTION_REFUSED = auto()
    ACCEPTED_TRANSACTION_NOT_CONFIRMABLE = auto()
    SYNTHETIC_PROOF_CORRUPTION_APPLIED = auto()
    UNCLASSIFIED_SCRIPT_ERROR = auto()
    FRAMEWORK_LOADED = auto()
    CONFIDENTIAL_MATERIALIZER_READY = auto()
    CONFIDENTIAL_MATERIALIZER_INITIALIZATION_FAILED = auto()
    CONFIDENTIAL_MATERIALIZER_UNAVAILABLE = auto()
    NODE_READY = auto()
    REQUEST_STREAM_ENDED = auto()
    CASE_ANSWERED = auto()
    CONSERVATION_ROW_ANSWERED = auto()
    NORMALIZATION_ROW_ANSWERED = auto()
    LIFECYCLE_STEP_ANSWERED = auto()
    FIXTURE_CONSTRUCTION_FAILED = auto()
    EXECUTOR_INFRASTRUCTURE_FAILED = auto()
    HANDSHAKE_FIELD_CENSUS_FAILED = auto()
    EXECUTION_REQUEST_FIELD_CENSUS_FAILED = auto()
    PROTOCOL_REVISION_REFUSED = auto()
    FRAMING_CLEAN_EOF = auto()
    FRAMING_BLANK_RECORD = auto()
    FRAMING_MALFORMED_RECORD = auto()
    FRAMING_OVERSIZED_RECORD = auto()
    FRAMING_UNTERMINATED_RECORD = auto()
    JSON_DECODE_FAILED = auto()
    FATAL_PROTOCOL_FAULT = auto()
    UNRECOGNIZED_DIAGNOSTIC_OUTCOME = auto()
    TYPED_DIAGNOSTIC_CONTENT_REJECTED = auto()


def diagnostic_message(outcome: DiagnosticOutcome, record: int | None = None) -> str:
    """Renders one known diagnostic outcome with its fixed spelling."""
    if outcome is DiagnosticOutcome.RPC_CLIENT_FAILED:
        return "rpc client failed; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.FRAMEWORK_CONSTRUCTION_FAILED:
        return "framework construction failed; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.ADAPTER_TRANSACTION_REFUSED:
        return "adapter-built transaction was refused; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.MEMPOOL_REASON_QUARANTINED:
        return "mempool refusal reason quarantined in elements-output record %d" % record
    if outcome is DiagnosticOutcome.TARGET_TRANSACTION_REFUSED:
        return (
            "target refused an adapter-built transaction; detail is "
            "elements-output record %d" % record
        )
    if outcome is DiagnosticOutcome.ACCEPTED_TRANSACTION_NOT_CONFIRMABLE:
        return (
            "accepted transaction was not confirmable; detail is "
            "elements-output record %d" % record
        )
    if outcome is DiagnosticOutcome.SYNTHETIC_PROOF_CORRUPTION_APPLIED:
        return "synthetic proof corruption applied; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.UNCLASSIFIED_SCRIPT_ERROR:
        return (
            "rejected with an unclassified script error; detail is "
            "elements-output record %d" % record
        )
    if outcome is DiagnosticOutcome.FRAMEWORK_LOADED:
        return "framework loaded"
    if outcome is DiagnosticOutcome.CONFIDENTIAL_MATERIALIZER_READY:
        return "confidential materializer ready"
    if outcome is DiagnosticOutcome.CONFIDENTIAL_MATERIALIZER_INITIALIZATION_FAILED:
        return "confidential materializer initialization failed"
    if outcome is DiagnosticOutcome.CONFIDENTIAL_MATERIALIZER_UNAVAILABLE:
        return "confidential materializer unavailable"
    if outcome is DiagnosticOutcome.NODE_READY:
        return "node ready; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.REQUEST_STREAM_ENDED:
        return "the request stream ended cleanly at a record boundary"
    if outcome is DiagnosticOutcome.CASE_ANSWERED:
        return "case answered; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.CONSERVATION_ROW_ANSWERED:
        return "conservation row answered; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.NORMALIZATION_ROW_ANSWERED:
        return "normalization row answered; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.LIFECYCLE_STEP_ANSWERED:
        return "lifecycle step answered; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.FIXTURE_CONSTRUCTION_FAILED:
        return "fixture construction failed; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.EXECUTOR_INFRASTRUCTURE_FAILED:
        return "executor infrastructure failed; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.HANDSHAKE_FIELD_CENSUS_FAILED:
        return (
            "fatal: handshake failed its field census; detail is "
            "elements-output record %d" % record
        )
    if outcome is DiagnosticOutcome.EXECUTION_REQUEST_FIELD_CENSUS_FAILED:
        return (
            "fatal: execution request failed its field census; detail is "
            "elements-output record %d" % record
        )
    if outcome is DiagnosticOutcome.PROTOCOL_REVISION_REFUSED:
        return "fatal: protocol revision refused"
    if outcome is DiagnosticOutcome.FRAMING_CLEAN_EOF:
        return "fatal: request framing failure clean_eof"
    if outcome is DiagnosticOutcome.FRAMING_BLANK_RECORD:
        return "fatal: request framing failure blank_record"
    if outcome is DiagnosticOutcome.FRAMING_MALFORMED_RECORD:
        return "fatal: request framing failure malformed_record"
    if outcome is DiagnosticOutcome.FRAMING_OVERSIZED_RECORD:
        return "fatal: request framing failure oversized_record"
    if outcome is DiagnosticOutcome.FRAMING_UNTERMINATED_RECORD:
        return "fatal: request framing failure unterminated_record"
    if outcome is DiagnosticOutcome.JSON_DECODE_FAILED:
        return "fatal: a JSON value this adapter read did not decode"
    if outcome is DiagnosticOutcome.FATAL_PROTOCOL_FAULT:
        return "fatal: protocol fault; detail is elements-output record %d" % record
    if outcome is DiagnosticOutcome.UNRECOGNIZED_DIAGNOSTIC_OUTCOME:
        return (
            "fatal: unrecognized diagnostic outcome; detail is "
            "elements-output record %d" % record
        )
    if outcome is DiagnosticOutcome.TYPED_DIAGNOSTIC_CONTENT_REJECTED:
        return "fatal: typed diagnostic content rejected"
    raise ValueError("unrecognized diagnostic outcome")


class FatalAdapterError(Exception):
    """A condition that ends the exchange rather than one case."""

    def __init__(
        self,
        detail: str = "",
        *,
        outcome: DiagnosticOutcome | None = None,
        record: int | None = None,
        diagnostic_written: bool = False,
    ) -> None:
        super().__init__(detail)
        self.outcome = outcome
        self.record = record
        self.diagnostic_written = diagnostic_written


class DiagnosticStreams:
    """The two destinations this adapter writes diagnostics to.

    Neither is stdout and neither is stderr. One holds typed facts and one
    holds raw child text, and the whole point of there being two is that
    the separation is a property of *where a byte is written* rather than
    of how carefully each call site phrased itself.

    `record` numbers every quarantined entry. The number is the only thing
    that crosses: a typed line may say that record 7 holds the text it
    declined to quote, and record 7's header in the elements output says
    which method and which exit status it belongs to. A reader can
    therefore correlate the two files without either file quoting the
    other's material.

    Both handles are flushed after every write. This process can be killed
    by the harness's group signal at any point, and a diagnostic that was
    still in a buffer when that happened would be a diagnostic about
    exactly the run that most needed one.
    """

    def __init__(self, output, elements_output) -> None:
        self.output = output
        self.elements_output = elements_output
        self.record = 0

    def _write_typed(self, message: str) -> None:
        """Writes one already-rendered fixed diagnostic line."""
        self.output.write("%s: %s\n" % (COMMAND_NAME, message))
        self.output.flush()

    def typed(self, outcome: DiagnosticOutcome, record: int | None = None) -> None:
        """Writes one closed-enum diagnostic outcome.

        CR/LF-bearing misuse is dropped and replaced with one fixed fatal
        line. Any other unrecognized form is quarantined and takes its own
        fixed fatal path; neither form is ever written through.
        """
        if isinstance(outcome, str) and ("\r" in outcome or "\n" in outcome):
            self._write_typed(
                diagnostic_message(DiagnosticOutcome.TYPED_DIAGNOSTIC_CONTENT_REJECTED)
            )
            raise FatalAdapterError(
                outcome=DiagnosticOutcome.TYPED_DIAGNOSTIC_CONTENT_REJECTED,
                diagnostic_written=True,
            )
        if not isinstance(outcome, DiagnosticOutcome):
            detail_record = self.quarantine("unrecognized diagnostic outcome", str(outcome))
            self._write_typed(
                diagnostic_message(
                    DiagnosticOutcome.UNRECOGNIZED_DIAGNOSTIC_OUTCOME,
                    detail_record,
                )
            )
            raise FatalAdapterError(
                outcome=DiagnosticOutcome.UNRECOGNIZED_DIAGNOSTIC_OUTCOME,
                record=detail_record,
                diagnostic_written=True,
            )
        try:
            message = diagnostic_message(outcome, record)
        except (TypeError, ValueError):
            detail_record = self.quarantine(
                "unrecognized diagnostic outcome",
                "%r with record %r" % (outcome, record),
            )
            self._write_typed(
                diagnostic_message(
                    DiagnosticOutcome.UNRECOGNIZED_DIAGNOSTIC_OUTCOME,
                    detail_record,
                )
            )
            raise FatalAdapterError(
                outcome=DiagnosticOutcome.UNRECOGNIZED_DIAGNOSTIC_OUTCOME,
                record=detail_record,
                diagnostic_written=True,
            ) from None
        if "\r" in message or "\n" in message:
            self._write_typed(
                diagnostic_message(DiagnosticOutcome.TYPED_DIAGNOSTIC_CONTENT_REJECTED)
            )
            raise FatalAdapterError(
                outcome=DiagnosticOutcome.TYPED_DIAGNOSTIC_CONTENT_REJECTED,
                diagnostic_written=True,
            )
        self._write_typed(message)

    def quarantine(self, subject: str, text: str) -> int:
        """Writes raw child text behind a header, and numbers it.

        The header is this adapter's own sentence -- a subject it composed
        from typed facts, a record number, and the byte count -- so a
        reader can find the entry's bounds without parsing the payload.
        The payload is written whole and unread: truncating it here would
        be this adapter deciding which part of a child's message mattered,
        which is the judgement the quarantine exists to avoid making.
        """
        self.record += 1
        number = self.record
        self.elements_output.write(
            "----- %s record %d: %s (%d characters) -----\n"
            % (COMMAND_NAME, number, subject, len(text))
        )
        self.elements_output.write(text)
        if not text.endswith("\n"):
            self.elements_output.write("\n")
        self.elements_output.write("----- end record %d -----\n" % number)
        self.elements_output.flush()
        return number


# The streams, established by `main` from the harness's two mandatory
# arguments and by nothing else.
#
# `None` before that, and `None` is not a fallback to stderr: an unconfigured
# adapter discards its diagnostics rather than writing them somewhere it was
# not told to write. The only way to reach a call site with the streams unset
# is to import this module and call one of its functions directly, which is
# what the node-free test lanes do; a run of this program as a program has
# parsed its arguments before any diagnostic exists to write.
STREAMS = None

# The status a run exits with when the two destinations could not be
# established at all -- named as a constant because it is the one refusal
# this program can make without being able to write a word about it, so the
# number is the whole of the message.
DIAGNOSTICS_UNAVAILABLE_STATUS = 2


def diagnostic_record(message: str) -> int | None:
    """Returns the quarantine record named at one known message's end."""
    match = re.search(r"elements-output record ([0-9]+)$", message)
    return int(match.group(1)) if match is not None else None


def diagnostic_classification(message: str):
    """Classifies one legacy call-site form into the closed outcome enum."""
    if message == "framework loaded":
        return DiagnosticOutcome.FRAMEWORK_LOADED, None, None
    if message == "the request stream ended cleanly at a record boundary":
        return DiagnosticOutcome.REQUEST_STREAM_ENDED, None, None
    if message.startswith("node ready in "):
        return DiagnosticOutcome.NODE_READY, None, "node startup detail"
    if message.startswith("case answered in "):
        return DiagnosticOutcome.CASE_ANSWERED, None, "case completion detail"
    if message.startswith("row ") and " answered in " in message:
        return DiagnosticOutcome.CONSERVATION_ROW_ANSWERED, None, "row completion detail"
    if message.startswith("normalization row ") and " answered in " in message:
        return DiagnosticOutcome.NORMALIZATION_ROW_ANSWERED, None, "normalization detail"
    if message.startswith("lifecycle step ") and " answered in " in message:
        return DiagnosticOutcome.LIFECYCLE_STEP_ANSWERED, None, "lifecycle detail"
    if message.startswith("corrupted "):
        return DiagnosticOutcome.SYNTHETIC_PROOF_CORRUPTION_APPLIED, None, "corruption detail"
    if message.startswith("infrastructure error: "):
        return DiagnosticOutcome.EXECUTOR_INFRASTRUCTURE_FAILED, None, "case failure"
    if message.startswith("fixture construction failure: "):
        return DiagnosticOutcome.FIXTURE_CONSTRUCTION_FAILED, None, "fixture failure"
    if message.startswith("executor infrastructure failure: "):
        return DiagnosticOutcome.EXECUTOR_INFRASTRUCTURE_FAILED, None, "executor failure"
    if message.startswith("the target refused the "):
        return DiagnosticOutcome.TARGET_TRANSACTION_REFUSED, None, "target refusal"
    if message.startswith("the accepted transaction was not confirmable: "):
        return (
            DiagnosticOutcome.ACCEPTED_TRANSACTION_NOT_CONFIRMABLE,
            None,
            "confirmation failure",
        )

    record = diagnostic_record(message)
    if record is None:
        return None
    if message.startswith("rpc "):
        return DiagnosticOutcome.RPC_CLIENT_FAILED, record, None
    if message.startswith("the framework raised "):
        return DiagnosticOutcome.FRAMEWORK_CONSTRUCTION_FAILED, record, None
    if message.startswith("the mempool's reason is "):
        return DiagnosticOutcome.MEMPOOL_REASON_QUARANTINED, record, None
    if message.startswith("rejected, with a script error "):
        return DiagnosticOutcome.UNCLASSIFIED_SCRIPT_ERROR, record, None
    if " transaction this adapter built was refused; " in message:
        return DiagnosticOutcome.ADAPTER_TRANSACTION_REFUSED, record, None
    return None


def log(outcome: DiagnosticOutcome | str, record: int | None = None) -> None:
    """Classifies and writes one closed-enum diagnostic outcome."""
    if STREAMS is None:
        return
    if isinstance(outcome, DiagnosticOutcome):
        STREAMS.typed(outcome, record)
        return
    if not isinstance(outcome, str):
        STREAMS.typed(outcome, record)
        return
    classification = diagnostic_classification(outcome)
    if classification is None:
        STREAMS.typed(outcome, record)
        return
    known, known_record, detail_subject = classification
    if detail_subject is not None:
        known_record = STREAMS.quarantine(detail_subject, outcome)
    STREAMS.typed(known, known_record)


def quarantined(subject: str, text: str) -> int:
    """Files raw child text in the `--elements-output` quarantine.

    Returns the record number, so the caller's typed line can name where
    the text it did not quote actually went. Returns 0 where no stream is
    configured, which is the "there is no such record" answer.
    """
    if STREAMS is None:
        return 0
    return STREAMS.quarantine(subject, text)


def log_quarantined(outcome: DiagnosticOutcome, subject: str, text: str) -> None:
    """Quarantines variable detail, then writes only its typed outcome."""
    record = quarantined(subject, text)
    log(outcome, record)


# --------------------------------------------------------------------------
# Strict wire decoding
# --------------------------------------------------------------------------


def require_object(value: object, path: str) -> dict:
    """Returns value as an object, or names the path that was not one."""
    if not isinstance(value, dict):
        raise AdapterError("field is not an object: %s" % path)
    return value


def require_keys(value: dict, allowed: tuple, path: str) -> None:
    """Refuses any field this adapter does not know at `path`.

    A parallel wave may extend the fixture schema. An unknown field must be
    loud here, so that a drift between the harness's fixture type and this
    adapter's reading of it surfaces as a named infrastructure error rather
    than as a silently ignored difference.
    """
    for key in value:
        if key not in allowed:
            raise AdapterError("unknown field: %s.%s" % (path, key))
    for key in allowed:
        if key not in value:
            raise AdapterError("missing field: %s.%s" % (path, key))


def require_int(value: object, path: str) -> int:
    """Returns value as an integer, or names the path that was not one."""
    if isinstance(value, bool) or not isinstance(value, int):
        raise AdapterError("field is not an integer: %s" % path)
    return value


def require_bytes(value: object, path: str) -> bytes:
    """Returns a serde byte sequence -- a JSON array of octets -- as bytes."""
    if not isinstance(value, list):
        raise AdapterError("field is not a byte array: %s" % path)
    out = bytearray()
    for index, item in enumerate(value):
        octet = require_int(item, "%s[%d]" % (path, index))
        if octet < 0 or octet > 0xFF:
            raise AdapterError("byte out of range: %s[%d]" % (path, index))
        out.append(octet)
    return bytes(out)


def require_byte_vectors(value: object, path: str) -> list:
    """Returns a JSON array of byte arrays as a list of bytes."""
    if not isinstance(value, list):
        raise AdapterError("field is not an array of byte arrays: %s" % path)
    return [require_bytes(item, "%s[%d]" % (path, index)) for index, item in enumerate(value)]


def require_optional_bytes(value: object, path: str):
    """Returns an optional serde byte sequence: bytes, or None where absent.

    An absent field is the fixture saying the executor supplies the value.
    It is never read as an empty one: empty bytes are a statement, and
    silence is not.
    """
    if value is None:
        return None
    return require_bytes(value, path)


def require_optional_int(value: object, path: str):
    """Returns an optional integer, or None where the fixture states none."""
    if value is None:
        return None
    return require_int(value, path)


def require_string(value: object, path: str) -> str:
    """Returns value as a string, or names the path that was not one."""
    if not isinstance(value, str):
        raise AdapterError("field is not a string: %s" % path)
    return value


def parse_subject(raw: object) -> dict:
    """Decodes one execution subject strictly, naming any field this adapter
    cannot read.

    Under protocol revision 3 there is no expectation here to discard. The
    request carries what to execute and nothing about what the result should
    be, so the discipline this adapter used to keep -- validate the expected
    outcome for shape and then look away from it -- is now a property of the
    message rather than of the adapter's restraint.

    Two members are validated and then discarded: `leaf_version_status` and
    `script_source` describe how the case came to be stated, which does not
    change what a node is asked. `enforcement_layer` is checked against the
    one layer this adapter answers at rather than ignored, since answering a
    relay question with a consensus verdict would be a different answer
    wearing the right shape.
    """
    subject = require_object(raw, "subject")
    require_keys(
        subject,
        (
            "case",
            "target_contract_version",
            "network_id",
            "genesis_id",
            "execution_domain",
            "leaf_version",
            "leaf_version_status",
            "enforcement_layer",
            "script_source",
            "script",
            "initial_stack",
            "context",
        ),
        "subject",
    )
    check_enumeration(
        subject["leaf_version_status"],
        ("reviewed", "unreviewed"),
        "subject.leaf_version_status",
    )
    check_enumeration(
        subject["script_source"],
        ("typed_program", "deliberately_malformed"),
        "subject.script_source",
    )
    layer = check_enumeration(
        subject["enforcement_layer"],
        ("consensus", "relay_policy"),
        "subject.enforcement_layer",
    )
    return {
        "enforcement_layer": layer,
        "execution_domain": subject["execution_domain"],
        "leaf_version": require_int(subject["leaf_version"], "subject.leaf_version"),
        "script": require_bytes(subject["script"], "subject.script"),
        "initial_stack": require_byte_vectors(subject["initial_stack"], "subject.initial_stack"),
        "context": parse_context(subject["context"]),
    }


def check_enumeration(raw: object, admitted: tuple, path: str) -> str:
    """Returns one of the admitted spellings, or names the one that was not."""
    text = require_string(raw, path)
    if text not in admitted:
        raise AdapterError("unknown value at %s: %s" % (path, text))
    return text


# Revision 3 sends no expectation of any kind, so the shape checks that
# used to validate `expected` and `expected_resources` without reading them
# are gone with the fields. A request that carries one now meets
# `require_keys`, which refuses an unknown member outright -- a stricter
# answer than looking away from it was.


def parse_context(raw: object):
    """Decodes the generic transaction context, or None where there is none."""
    if raw is None:
        return None
    context = require_object(raw, "subject.context")
    require_keys(
        context,
        ("version", "locktime", "current_input_index", "inputs", "outputs", "script_path"),
        "subject.context",
    )
    inputs = context["inputs"]
    outputs = context["outputs"]
    if not isinstance(inputs, list) or not isinstance(outputs, list):
        raise AdapterError("subject.context inputs and outputs must be arrays")
    return {
        "version": require_int(context["version"], "subject.context.version"),
        "locktime": require_int(context["locktime"], "subject.context.locktime"),
        "current_input_index": require_int(
            context["current_input_index"], "subject.context.current_input_index"
        ),
        "inputs": [
            parse_input(item, "subject.context.inputs[%d]" % index)
            for index, item in enumerate(inputs)
        ],
        "outputs": [
            parse_output(item, "subject.context.outputs[%d]" % index)
            for index, item in enumerate(outputs)
        ],
        "script_path": parse_script_path(context["script_path"]),
    }


def parse_input(raw: object, path: str) -> dict:
    """Decodes one fixture input."""
    value = require_object(raw, path)
    require_keys(
        value,
        (
            "outpoint_txid",
            "outpoint_index",
            "spent_asset",
            "spent_value",
            "spent_program",
            "sequence",
            "issuance",
            "witness",
        ),
        path,
    )
    issuance = value["issuance"]
    if issuance is not None:
        require_keys(
            require_object(issuance, path + ".issuance"),
            ("asset_amount", "inflation_keys_amount", "entropy", "blinding_nonce"),
            path + ".issuance",
        )
    return {
        "outpoint_txid": require_optional_bytes(
            value["outpoint_txid"], path + ".outpoint_txid"
        ),
        "outpoint_index": require_optional_int(
            value["outpoint_index"], path + ".outpoint_index"
        ),
        "spent_asset": require_optional_bytes(value["spent_asset"], path + ".spent_asset"),
        "spent_value": require_optional_bytes(value["spent_value"], path + ".spent_value"),
        "spent_program": require_optional_bytes(
            value["spent_program"], path + ".spent_program"
        ),
        "sequence": require_int(value["sequence"], path + ".sequence"),
        "issuance": issuance,
        "witness": require_byte_vectors(value["witness"], path + ".witness"),
        "path": path,
    }


def parse_output(raw: object, path: str) -> dict:
    """Decodes one fixture output."""
    value = require_object(raw, path)
    require_keys(value, ("asset", "value", "nonce", "program"), path)
    return {
        "asset": require_optional_bytes(value["asset"], path + ".asset"),
        "value": require_bytes(value["value"], path + ".value"),
        "nonce": require_bytes(value["nonce"], path + ".nonce"),
        "program": require_optional_bytes(value["program"], path + ".program"),
        "path": path,
    }


def parse_script_path(raw: object) -> dict:
    """Decodes the declared script path."""
    value = require_object(raw, "subject.context.script_path")
    require_keys(
        value, ("leaf_version", "script", "control"), "subject.context.script_path"
    )
    return {
        "leaf_version": require_int(
            value["leaf_version"], "subject.context.script_path.leaf_version"
        ),
        "script": require_bytes(value["script"], "subject.context.script_path.script"),
        "control": require_optional_bytes(
            value["control"], "subject.context.script_path.control"
        ),
    }


def parse_tap_tree(raw: object, path: str) -> dict:
    """Decodes one stated taproot tree node.

    The wire form is an externally tagged enum: a node is an object carrying
    exactly one of `leaf` or `branch`. A node carrying neither, or both,
    states no tree and is refused here rather than read as whichever variant
    happens to be recognised -- a tree read as something other than what was
    sent is exactly the substitution the stated tree exists to prevent.
    """
    node = require_object(raw, path)
    names = tuple(node)
    if len(names) != 1 or names[0] not in ("leaf", "branch"):
        raise AdapterError("field is not a taproot tree node: %s" % path)
    variant = names[0]
    inner = "%s.%s" % (path, variant)
    body = require_object(node[variant], inner)
    if variant == "leaf":
        require_keys(body, ("version", "script"), inner)
        version = require_int(body["version"], inner + ".version")
        if version < 0 or version > 0xFF:
            raise AdapterError("leaf version is not a byte: %s.version" % inner)
        return {
            "kind": "leaf",
            "version": version,
            "script": require_bytes(body["script"], inner + ".script"),
        }
    require_keys(body, ("left", "right"), inner)
    return {
        "kind": "branch",
        "left": parse_tap_tree(body["left"], inner + ".left"),
        "right": parse_tap_tree(body["right"], inner + ".right"),
    }


def parse_construction_output(raw: object, path: str) -> dict:
    """Decodes one output a construction requires, by role."""
    value = require_object(raw, path)
    require_keys(value, ("role", "program"), path)
    return {
        "role": check_enumeration(value["role"], ("successor",), path + ".role"),
        "program": require_bytes(value["program"], path + ".program"),
    }


def parse_construction(raw: object, path: str = "request.construction") -> dict:
    """Decodes one stated taproot construction strictly.

    Every field is a requirement rather than a hint, so an unreadable one is
    named here and the case is refused. Reading a construction loosely would
    let the adapter build a tree the request did not state and then report a
    target verdict about it.

    The path is a parameter because the same record reaches this adapter in
    two places: beside a primitive fixture, and inside a compound one. A
    refusal names where the field it could not read actually was.
    """
    value = require_object(raw, path)
    require_keys(
        value,
        (
            "internal_key",
            "tree",
            "executing_leaf",
            "control",
            "predecessor_program",
            "outputs",
        ),
        path,
    )
    internal_key = require_bytes(value["internal_key"], path + ".internal_key")
    if len(internal_key) != XONLY_KEY_BYTES:
        raise AdapterError(
            "the stated internal key is %d bytes, and an x-only key is %d: "
            "%s.internal_key" % (len(internal_key), XONLY_KEY_BYTES, path)
        )
    raw_outputs = value["outputs"]
    if not isinstance(raw_outputs, list):
        raise AdapterError("field is not an array: %s.outputs" % path)
    return {
        "internal_key": internal_key,
        "tree": parse_tap_tree(value["tree"], path + ".tree"),
        "executing_leaf": parse_tap_tree(value["executing_leaf"], path + ".executing_leaf"),
        "control": require_optional_bytes(value["control"], path + ".control"),
        "predecessor_program": require_bytes(
            value["predecessor_program"], path + ".predecessor_program"
        ),
        "outputs": [
            parse_construction_output(item, "%s.outputs[%d]" % (path, index))
            for index, item in enumerate(raw_outputs)
        ],
    }


def parse_prototype_fixture(raw: object) -> tuple:
    """Decodes one compound-prototype fixture strictly.

    Returns the same pair the primitive path produces -- the execution
    subject and the construction to materialise -- so that one execution
    routine serves both. What differs is what the record states and what it
    does not.

    Three fields a primitive fixture states are absent here, and each is
    supplied by this adapter under a stated rule rather than guessed:

      execution domain    a compound fixture is a taproot script-path spend
                          by construction, since it states a leaf, a tree,
                          and a control path. This adapter executes exactly
                          that domain and no other;
      leaf version        stated once, inside the executing leaf, rather
                          than twice where two copies could drift. It is
                          checked against the one version this adapter
                          advertises, and any other is refused;
      enforcement layer   consensus. A compound relation states that a
                          spend stands or does not, which is what block
                          validation decides; standardness is not the
                          contract, and answering a consensus question with
                          a mempool verdict would report an unrelayable but
                          perfectly valid spend as an invalid one.

    `target_contract_version` is validated and then discarded. The expected
    verdict, the expected resource figures, and the claim set are not
    discarded here: under revision 3 they never arrive, which is what makes
    the boundary a property of the protocol rather than of this adapter.
    """
    subject = require_object(raw, "subject")
    require_keys(
        subject,
        (
            "case",
            "target_contract_version",
            "script",
            "initial_stack",
            "construction",
        ),
        "subject",
    )
    require_int(subject["target_contract_version"], "subject.target_contract_version")
    check_prototype_case_shape(subject["case"])

    construction = parse_construction(subject["construction"], "subject.construction")
    executing = construction["executing_leaf"]
    if executing["kind"] != "leaf":
        raise AdapterError(
            "subject.construction.executing_leaf is a branch, and a spend "
            "executes a leaf"
        )
    if executing["version"] != TAPSCRIPT_LEAF_VERSION:
        raise AdapterError(
            "subject.construction.executing_leaf.version is %d, and this "
            "adapter advertised only leaf version %d"
            % (executing["version"], TAPSCRIPT_LEAF_VERSION)
        )
    return (
        {
            "enforcement_layer": "consensus",
            "execution_domain": "tapscript",
            "leaf_version": executing["version"],
            "script": require_bytes(subject["script"], "subject.script"),
            "initial_stack": require_byte_vectors(
                subject["initial_stack"], "subject.initial_stack"
            ),
            "context": None,
        },
        construction,
    )


def check_prototype_case_shape(raw: object) -> None:
    """Validates a compound case identity without reading it as a primitive one.

    The identity is echoed back verbatim rather than rebuilt, so nothing
    here is retained. What it establishes is that the record really is the
    one this adapter thinks it is: a case naming no relation would have been
    routed here by the relation key alone.
    """
    case = require_object(raw, "subject.case")
    require_keys(case, ("relation", "name"), "subject.case")
    check_enumeration(
        case["relation"],
        ("metadata_constructor_continuity", "wide_floor_relation"),
        "subject.case.relation",
    )
    require_string(case["name"], "subject.case.name")


# The claim set used to be validated for shape here and then discarded.
# Revision 3 does not send one, so there is nothing left to look away
# from: what a passing case would establish is the harness's accounting
# and never crossed the boundary usefully.


# --------------------------------------------------------------------------
# The disposable node
# --------------------------------------------------------------------------


def free_loopback_port() -> int:
    """Picks a loopback port the node can bind."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


class DisposableNode:
    """One `elementsd` regtest instance in a directory this process owns.

    The directory is created here, holds the node's cookie, and is deleted in
    `stop`. Nothing outside this class ever names it, and no credential
    leaves it: `elements-cli` is given the same `-datadir` and resolves the
    cookie itself.

    # One lane owns a directory this process did not create

    The Guide-11 section 13 fresh-process lane needs the CHAIN to outlive
    the process that built on it, because that is the entire question it
    asks: whether public evidence is recoverable by a party that did not
    participate in creation. A disposable directory answers that question
    by destroying the evidence along with the creator, which is not a
    proof of anything.

    So `datadir` names a directory the caller owns, and `retain` keeps it
    after `stop`. Both are off by default and every other lane keeps the
    directory it always had. What persists is chain data -- the canonical
    public record. Creator-local state is a wallet inside that directory,
    and destroying it is the lane's own step rather than a side effect of
    the node exiting.
    """

    def __init__(
        self,
        elementsd: str,
        elements_cli: str,
        chain: str,
        boot_timeout: float,
        enable_wallet: bool = False,
        datadir=None,
        retain: bool = False,
    ) -> None:
        self.elementsd = elementsd
        self.elements_cli = elements_cli
        self.chain = chain
        self.boot_timeout = boot_timeout
        # Off unless a lane needs it, so the primitive and compound lanes
        # boot exactly the node they have always booted. It is enabled only
        # for the conservation lane, which needs a wallet for one reason:
        # a confidential coin whose blinding factors are KNOWN. The node
        # reports those in `listunspent`, and no non-wallet interface does.
        #
        # No chain parameter changes with it. That is not a preference: the
        # genesis identifier is what the harness binds the run to, and a
        # chain parameter would move it, so the wallet is funded from the
        # same free coin the adapter already spends rather than by a block
        # subsidy or a connected genesis output.
        self.enable_wallet = enable_wallet
        if datadir is None:
            self.datadir = tempfile.mkdtemp(prefix="tripod-native-executor-")
            self.retain = retain
        else:
            os.makedirs(datadir, exist_ok=True)
            self.datadir = os.path.abspath(datadir)
            # A directory this process did not create is never deleted by
            # it, whatever the flag says. Deleting a caller's chain would
            # be destroying the public record the lane exists to read.
            self.retain = True
        self.rpc_port = free_loopback_port()
        self.process = None
        self.node_log = None

    def start(self) -> None:
        """Boots the node and waits for its RPC interface."""
        self.node_log = open(os.path.join(self.datadir, "node.out"), "wb")
        arguments = [
            self.elementsd,
            "-datadir=" + self.datadir,
            "-chain=" + self.chain,
            "-daemon=0",
            "-server=1",
            "-listen=0",
            "-discover=0",
            "-dnsseed=0",
            "-rpcbind=127.0.0.1",
            "-rpcallowip=127.0.0.1",
            "-rpcport=%d" % self.rpc_port,
            "-disablewallet=%d" % (0 if self.enable_wallet else 1),
            # Regtest validates pegins against a bitcoind this harness
            # does not run, so the lane could not boot with the upstream
            # default (´[PLAN-obs:upstream:regtest-pegin-validation]´).
            "-validatepegin=0",
            "-minrelaytxfee=0",
            "-blockmintxfee=0",
            "-initialfreecoins=2100000000000000",
            "-anyonecanspendaremine=1",
            "-printtoconsole=1",
        ]
        self.process = subprocess.Popen(
            arguments,
            stdin=subprocess.DEVNULL,
            stdout=self.node_log,
            stderr=subprocess.STDOUT,
        )
        deadline = time.monotonic() + self.boot_timeout
        while True:
            if self.process.poll() is not None:
                raise FatalAdapterError(
                    "the node exited during startup with status %d" % self.process.returncode
                )
            try:
                self.call("getblockchaininfo")
                return
            except AdapterError:
                if time.monotonic() >= deadline:
                    raise FatalAdapterError("the node did not answer RPC before the boot timeout")
                time.sleep(0.2)

    def call(self, method: str, *arguments: str, wallet=None):
        """Issues one JSON-RPC call through `elements-cli`.

        `wallet` routes the call at a named wallet. It is the wallet's
        NAME and never a credential: the node's cookie stays inside the
        disposable datadir and is resolved by `elements-cli` from the same
        `-datadir`, exactly as for every other call here.
        """
        command = [
            self.elements_cli,
            "-datadir=" + self.datadir,
            "-chain=" + self.chain,
            "-rpcport=%d" % self.rpc_port,
        ]
        if wallet is not None:
            command.append("-rpcwallet=" + wallet)
        command.append(method)
        command.extend(arguments)
        try:
            completed = subprocess.run(
                command, stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False
            )
        except OSError as error:
            raise AdapterError("could not run the node client: %s" % error.strerror)
        if completed.returncode != 0:
            # The client's stderr is NOT read into this note. The note
            # becomes observed_detail on a first-party protocol record,
            # so anything placed here is bytes from a child process
            # arriving in first-party evidence -- which is precisely
            # what the stream contract above says does not happen. That
            # contract used to be stated as "the harness nulls this
            # process's stderr"; it did not describe this path, and the
            # same bytes were reaching the same place by the back door
            # (G12-R04).
            #
            # What is left is fixed and typed: the method, which the
            # harness itself named, and the client's exit status. The
            # omission is stated rather than silent, because a
            # diagnostic that quietly dropped the reason would be less
            # honest than one that says where the reason went -- and
            # where it went is the elements output, which is named here
            # by record number rather than quoted.
            record = quarantined(
                "rpc %s exit status %d, client stderr"
                % (method, completed.returncode),
                completed.stderr,
            )
            log(
                "rpc %s failed with client exit status %d; the client's text "
                "is elements-output record %d"
                % (method, completed.returncode, record)
            )
            raise AdapterError(
                "rpc %s failed with client exit status %d; the client's stderr is "
                "omitted from first-party records by contract"
                % (method, completed.returncode),
                client_detail=one_line(completed.stderr),
            )
        text = completed.stdout.strip()
        if text == "":
            return None
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            return text

    def stop(self) -> None:
        """Terminates the node and deletes the directory it owned."""
        if self.process is not None and self.process.poll() is None:
            try:
                self.call("stop")
            except AdapterError:
                pass
            try:
                self.process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                self.process.terminate()
                try:
                    self.process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait(timeout=10)
        self.process = None
        if self.node_log is not None:
            self.node_log.close()
            self.node_log = None
        if not self.retain:
            shutil.rmtree(self.datadir, ignore_errors=True)


def one_line(text: str) -> str:
    """Collapses a client's message to one bounded line.

    Used only where the result stays inside this process -- the
    `client_detail` a consensus judgement classifies -- and never to
    prepare text for a typed diagnostic. Raw text is quarantined whole by
    `quarantined`; there is nothing left for a collapsing helper to make
    safe, because collapsing was never what made it safe.
    """
    return " ".join(text.split())[:400]


# --------------------------------------------------------------------------
# Transaction construction
# --------------------------------------------------------------------------


def load_framework(framework: str):
    """Imports the upstream functional-test framework from an explicit path.

    The path is supplied as `--framework`; it is never guessed, never read
    from the environment, and never defaulted. The framework provides the
    reference taproot tweak, control-block, and Elements transaction
    serialisation used below, so that this adapter does not reimplement any
    of them.
    """
    candidate = os.path.abspath(framework)
    if os.path.basename(candidate) == "test_framework":
        candidate = os.path.dirname(candidate)
    if not os.path.isdir(os.path.join(candidate, "test_framework")):
        raise FatalAdapterError("the framework path holds no test_framework package")
    sys.path.insert(0, candidate)
    try:
        from test_framework import messages, script
    except ImportError as error:
        raise FatalAdapterError("the framework did not import: %s" % error)
    # The key module is optional, and its absence is reported as an
    # absent capability rather than a fatal one: everything except the
    # sponsor lane runs perfectly well without it, and a framework
    # revision that does not ship it should still be usable for the rest.
    try:
        from test_framework import key as key_module
    except ImportError:
        key_module = None
    return candidate, messages, script, key_module


def txid_to_internal_int(txid_hex: str) -> int:
    """Converts a displayed txid into the integer an outpoint carries."""
    return int.from_bytes(bytes.fromhex(txid_hex), "big")


def tree_items(node: dict, path: str, leaves: list):
    """Maps one stated tree onto the item structure `taproot_construct` takes.

    The framework's own helper splits a *flat* list of items down the middle
    and descends, so a tree handed over flat would come back in whatever
    shape that split produced rather than in the shape the fixture stated.
    Every branch is therefore written as its own two-item list: the nesting
    carries the shape, and the helper is left nothing to choose.

    Each leaf is given a name unique to its position, so that a tree carrying
    the same script twice still yields two distinct entries in the
    framework's leaf table -- one name for two leaves would silently drop one
    of them.
    """
    if node["kind"] == "leaf":
        name = "leaf:%s" % (path or "root")
        leaves.append({"name": name, "node": node, "depth": len(path)})
        return (name, node["script"], node["version"])
    return [
        tree_items(node["left"], path + "l", leaves),
        tree_items(node["right"], path + "r", leaves),
    ]


def explicit_amount(field: bytes, path: str) -> int:
    """Reads an explicit Elements value field, or refuses a confidential one."""
    if len(field) == 9 and field[0] == EXPLICIT_PREFIX:
        return int.from_bytes(field[1:], "big")
    raise AdapterError(
        "confidential or malformed value field, which this adapter cannot "
        "materialise deterministically: %s" % path
    )


def explicit_value_field(amount: int) -> bytes:
    """The nine-byte explicit form of one value field.

    The inverse of `explicit_amount`, spelled once so the two forms of a
    value field are built in one place: a prefix byte and the amount in
    the target's own big-endian order.
    """
    return bytes([EXPLICIT_PREFIX]) + amount.to_bytes(8, "big")


class CaseExecutor:
    """Executes fixtures against one disposable node."""

    def __init__(self, node: DisposableNode, messages, script, key_module=None) -> None:
        self.node = node
        self.messages = messages
        self.script = script
        self.key_module = key_module
        self.internal_key = bytes.fromhex(NUMS_INTERNAL_KEY_HEX)
        self.anyone_can_spend = bytes.fromhex(ANYONE_CAN_SPEND_HEX)
        self.anyone_can_spend_witness = bytes.fromhex(ANYONE_CAN_SPEND_WITNESS_HEX)
        self.change = None
        self.policy_asset_field = None
        self.mining_descriptor = None
        self.mock_time = 0
        # Whether this adapter can genuinely materialise a stated tree. It is
        # the framework's helper that does the work, so the capability is the
        # helper's presence and not a claim written by hand: an adapter that
        # advertised a tree it could not build would be sent exactly the work
        # only it could refuse.
        self.tree_materialization = callable(getattr(script, "taproot_construct", None))
        # A compound-prototype fixture states a construction and nothing
        # else it could be answered from, so the ability to answer one is
        # exactly the ability to materialise the tree it states. It is
        # derived from the same helper rather than written by hand, because
        # an adapter advertising a capability whose machinery it lacks would
        # be sent precisely the work only it could refuse.
        self.prototype_fixtures = self.tree_materialization
        # Present only where the wallet the conservation lane needs was
        # enabled at boot. Derived from the node this adapter actually
        # started rather than written by hand, on the same reasoning as
        # every other capability here.
        self.conservation = None
        # The section 13 lane, present on the same condition and for the
        # same reason: it builds on the normalization claim, which builds
        # on the wallet.
        self.lifecycle = None
        # The Guide-12 section 16.2 operation lane. Present only where the
        # wallet was enabled, and the reason is narrow rather than
        # inherited: an issuance has to send the issued asset somewhere,
        # `rawissueasset` names that somewhere by address, and this
        # adapter has no way to spell an address without a wallet. The
        # submission half needs no wallet at all, but the two capabilities
        # are advertised together because a submission with nothing funded
        # to spend is a step that could only be refused.
        self.operations = None
        # Whether this adapter can act as a sponsor: hold reserve value at
        # a program of its own and authorize a spend of it. Derived from
        # the machinery actually present rather than written by hand, on
        # the same reasoning as every other capability here -- the two
        # things it needs are a key implementation and the segwit-v0
        # signature hash, and an adapter advertising a capability whose
        # machinery it lacks would be sent precisely the work only it
        # could refuse.
        self.sponsor_authorization = key_module is not None and callable(
            getattr(script, "SegwitV0SignatureHash", None)
        )

    def prime(self) -> None:
        """Locates a spendable free-coin output and confirms one block.

        On a chain this process just created, that output is the genesis
        free coin and the search ends there. On a chain a previous
        process already built on -- the Guide-11 section 13 lane's only
        difference -- the genesis coin is long spent, and taking it
        anyway would name an outpoint the chain no longer has. The
        spentness is asked of the node rather than assumed, and the
        replacement is found by scanning the public UTXO set for the same
        anyone-can-spend program, which is chain data and nobody's
        secret.
        """
        # The descriptor the maturity blocks are mined to, checksummed by the
        # node so that a hand-written checksum cannot drift from the script.
        self.mining_descriptor = self.node.call(
            "getdescriptorinfo", MINING_DESCRIPTOR
        )["descriptor"]
        self.node.call("generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, "[]")
        genesis_hash = self.node.call("getblockhash", "0")
        genesis = self.node.call("getblock", genesis_hash, "2")
        for transaction in genesis["tx"]:
            for output in transaction["vout"]:
                program = output["scriptPubKey"]["hex"]
                if program != ANYONE_CAN_SPEND_HEX:
                    continue
                amount = int(round(float(output["value"]) * 100_000_000))
                if amount <= 0:
                    continue
                asset = bytes.fromhex(output["asset"])[::-1]
                self.policy_asset_field = bytes([EXPLICIT_PREFIX]) + asset
                self.change = {
                    "txid": transaction["txid"],
                    "vout": output["n"],
                    "amount": amount,
                }
                self.mock_time = self.node.call("getblockchaininfo")["mediantime"]
                if self.node.call(
                    "gettxout", transaction["txid"], str(output["n"])
                ) is None:
                    self.change = self.rescan_free_coin()
                return
        raise FatalAdapterError("the chain carries no anyone-can-spend free-coin output")

    def rescan_free_coin(self) -> dict:
        """The largest unspent anyone-can-spend output the chain carries.

        Public data end to end: an unspent-output scan for a program
        whose spending condition is "anyone", which is what makes it a
        sponsor source a party with no relationship to any previous
        process can use.
        """
        answer = self.node.call(
            "scantxoutset", "start",
            json.dumps(["raw(%s)" % ANYONE_CAN_SPEND_HEX]),
        )
        found = None
        for entry in (answer or {}).get("unspents", []):
            amount = int(round(float(entry["amount"]) * 100_000_000))
            if amount <= 0:
                continue
            if found is None or amount > found["amount"]:
                found = {"txid": entry["txid"], "vout": entry["vout"], "amount": amount}
        if found is None:
            raise FatalAdapterError(
                "the chain carries no unspent anyone-can-spend output to fund from"
            )
        return found

    # -- chain maturity ---------------------------------------------------

    def mine(self, count: int) -> None:
        """Mines empty blocks to the adapter's own program."""
        if count > 0:
            self.node.call("generatetodescriptor", str(count), self.mining_descriptor)

    def mature(self, context: dict, funded: list) -> None:
        """Advances the chain until the declared transaction can be final.

        A fixture states a lock time and a sequence per input, and both are
        chain requirements as well as script data: a transaction whose lock
        time is above the tip, or whose input is younger than its own
        relative lock, is refused as non-final. That refusal is not a script
        verdict, and reporting it as one would make every introspection case
        look like a target rejection.

        So the chain is grown to meet what the fixture declared, rather than
        the declaration being trimmed to what an unmatured chain accepts. The
        rules restated here are the target's: an absolute lock time below the
        threshold counts blocks and above it counts seconds; a relative lock
        counts either, is measured from the block that funded the input, and
        is disabled outright by the flag bit.
        """
        required_height = 0
        required_time = 0

        # An absolute lock binds only while some input is non-final.
        if context["locktime"] != 0 and any(
            declared["sequence"] != 0xFFFF_FFFF for declared in context["inputs"]
        ):
            if context["locktime"] < LOCKTIME_HEIGHT_THRESHOLD:
                required_height = max(required_height, context["locktime"])
            else:
                required_time = max(required_time, context["locktime"] + 1)

        # Relative locks bind only from the version that introduced them.
        if context["version"] >= 2:
            for entry in funded:
                sequence = entry["sequence"]
                if sequence & SEQUENCE_DISABLE_FLAG:
                    continue
                age = sequence & SEQUENCE_AGE_MASK
                if sequence & SEQUENCE_TIME_MODE_FLAG:
                    required_time = max(
                        required_time,
                        entry["parent_median_time"] + (age << SEQUENCE_TIME_GRANULARITY_BITS) + 1,
                    )
                else:
                    required_height = max(required_height, entry["coin_height"] + age - 1)

        self.advance(required_height, required_time)

    def advance(self, required_height: int, required_time: int) -> None:
        """Grows the chain to one height and one median time."""
        info = self.node.call("getblockchaininfo")
        if required_time > info["mediantime"]:
            # The median is taken over a window, so the clock is moved once
            # and the window is refilled at the later time. The clock only
            # ever moves forward: a chain whose time went backwards would
            # refuse the very blocks meant to advance it.
            self.mock_time = max(self.mock_time + 1, required_time)
            self.node.call("setmocktime", str(self.mock_time))
            self.mine(MEDIAN_TIME_BLOCKS)
            info = self.node.call("getblockchaininfo")
            if info["mediantime"] < required_time:
                raise AdapterError(
                    "the chain's median time reached %d, and the declared "
                    "relative timelock needs %d"
                    % (info["mediantime"], required_time)
                )
        if required_height > info["blocks"]:
            self.mine(required_height - info["blocks"])

    # -- taproot ----------------------------------------------------------

    def taproot_for(self, script_bytes: bytes, leaf_version: int):
        """Builds the single-leaf taproot commitment to one script."""
        leaf_script = self.script.CScript(script_bytes)
        info = self.script.taproot_construct(
            self.internal_key, [("leaf", leaf_script, leaf_version)]
        )
        leaf = info.leaves["leaf"]
        control = bytes([leaf.version + info.negflag]) + info.internal_pubkey + leaf.merklebranch
        return info.scriptPubKey, bytes(leaf.script), control

    def materialise_construction(self, fixture: dict, construction: dict):
        """Builds exactly the taproot commitment the request states.

        Nothing here is chosen by this adapter: the internal key, every leaf
        script, every leaf version byte, and the tree's shape are the
        request's, and each is checked against what the framework actually
        built. A stated value this adapter cannot reproduce ends the case as
        a named refusal, because an approximated construction makes the
        target answer a question the fixture did not ask.

        The two strongest checks are the last two. The committed program
        pins the internal key and the whole tree at once -- no other tree
        yields it -- and the control block pins the path the spend will
        actually authenticate.
        """
        executing = construction["executing_leaf"]
        if executing["kind"] != "leaf":
            raise AdapterError(
                "request.construction.executing_leaf is a branch, and a spend "
                "executes a leaf"
            )
        if executing["script"] != fixture["script"]:
            raise AdapterError(
                "request.construction.executing_leaf.script disagrees with "
                "subject.script"
            )
        if executing["version"] != fixture["leaf_version"]:
            raise AdapterError(
                "request.construction.executing_leaf.version disagrees with "
                "subject.leaf_version"
            )

        leaves = []
        items = tree_items(construction["tree"], "", leaves)
        if not isinstance(items, list):
            # A tree that is one leaf: the framework's helper reads a
            # single-item list as that leaf.
            items = [items]
        for entry in leaves:
            version = entry["node"]["version"]
            if version & CONTROL_PARITY_MASK:
                raise AdapterError(
                    "the stated tree carries the odd leaf version %d, and a "
                    "taproot leaf version's low bit is the output key's "
                    "parity rather than part of the version" % version
                )

        try:
            info = self.script.taproot_construct(construction["internal_key"], items)
        except Exception as error:
            # The exception's TYPE, which is a fixed and bounded fact,
            # and not its message. A third-party library's message is
            # uncontrolled text that can carry a path out of this
            # operator's filesystem, and observed_detail is a
            # first-party record rather than a place for it. The full
            # message is quarantined with the child text, for the same
            # reason and in the same place (G12-R04): the framework runs
            # in this process, but its message is no more this adapter's
            # own sentence than a child's stderr is.
            record = quarantined(
                "framework raised %s building a taproot commitment"
                % type(error).__name__,
                str(error),
            )
            log(
                "the framework raised %s building a taproot commitment; its "
                "message is elements-output record %d"
                % (type(error).__name__, record)
            )
            raise AdapterError(
                "the framework built no taproot commitment for the stated "
                "tree, raising %s" % type(error).__name__
            )

        if bytes(info.internal_pubkey) != construction["internal_key"]:
            raise AdapterError(
                "the framework committed to an internal key other than the "
                "stated one"
            )

        matches = [
            entry
            for entry in leaves
            if entry["node"]["version"] == executing["version"]
            and entry["node"]["script"] == executing["script"]
        ]
        if len(matches) != 1:
            raise AdapterError(
                "the stated tree carries the executing leaf %d times, and a "
                "control path is determined only by exactly one" % len(matches)
            )
        entry = matches[0]
        leaf = info.leaves[entry["name"]]
        if bytes(leaf.script) != executing["script"] or leaf.version != executing["version"]:
            raise AdapterError(
                "the framework built the executing leaf from other bytes than "
                "the stated ones"
            )
        built_depth, remainder = divmod(len(leaf.merklebranch), XONLY_KEY_BYTES)
        if remainder or built_depth != entry["depth"]:
            raise AdapterError(
                "the framework authenticated the executing leaf under %d "
                "sibling bytes, and the stated tree places it at depth %d"
                % (len(leaf.merklebranch), entry["depth"])
            )

        program = bytes(info.scriptPubKey)
        control = (
            bytes([leaf.version + info.negflag])
            + bytes(info.internal_pubkey)
            + bytes(leaf.merklebranch)
        )
        if program != construction["predecessor_program"]:
            raise AdapterError(
                "the stated tree commits to a program of %d bytes that is not "
                "the stated predecessor program" % len(program)
            )
        stated_control = construction["control"]
        if stated_control is not None and stated_control != control:
            if len(stated_control) == len(control) and stated_control[1:] == control[1:]:
                raise AdapterError(
                    "the stated control block carries a first byte the stated "
                    "tree does not determine, which is the leaf version or the "
                    "output key's parity"
                )
            raise AdapterError(
                "the stated control block is not the one the stated tree "
                "determines"
            )
        return program, bytes(leaf.script), control

    # -- primitives -------------------------------------------------------

    def output(self, amount: int, program: bytes, asset_field=None, nonce_field=None):
        """Builds one explicit Elements output."""
        messages = self.messages
        asset = asset_field if asset_field is not None else self.policy_asset_field
        out = messages.CTxOut(
            nValue=messages.CTxOutValue(amount),
            scriptPubKey=program,
            nAsset=messages.CTxOutAsset(asset),
        )
        if nonce_field is not None:
            out.nNonce = messages.CTxOutNonce(nonce_field)
        return out

    def raw_output(self, declared: dict):
        """Builds one output from declared field bytes, verbatim.

        A field the fixture leaves unstated is the executor's to supply, and
        only two are: the asset, because which asset a development network
        issues is the network's fact, and the program, because a fixture that
        named one would be describing a deployment. Both are supplied with
        the chain's own -- the policy asset and the anyone-can-spend program
        -- and everything the fixture does state is written unchanged.
        """
        messages = self.messages
        asset_field = declared["asset"]
        if asset_field is None:
            asset_field = self.policy_asset_field
        program = declared["program"]
        if program is None:
            program = self.anyone_can_spend
        out = messages.CTxOut()
        out.nAsset = messages.CTxOutAsset(asset_field)
        out.nValue = messages.CTxOutValue()
        out.nValue.vchCommitment = declared["value"]
        out.nNonce = messages.CTxOutNonce(declared["nonce"])
        out.scriptPubKey = program
        return out

    def fund(self, program: bytes, amount: int) -> str:
        """Creates and confirms one output paying `amount` to `program`.

        The funding transaction is confirmed by mining it directly, rather
        than broadcast and left in the mempool. `generateblock` mines exactly
        the transactions it is handed and pulls nothing from the mempool, so
        an unconfirmed parent would be invisible both to the consensus retry
        in `judge_at_consensus` and to a block built later; and an
        ever-growing mempool chain would eventually meet the ancestor limit,
        whose rejection is not a script verdict but would arrive looking like
        one.
        """
        messages = self.messages
        source = self.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")
        remainder = source["amount"] - amount - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError("the adapter's change output cannot fund this case")
        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(source["txid"]), source["vout"]),
                nSequence=0xFFFFFFFE,
            )
        )
        transaction.vout.append(self.output(amount, program))
        transaction.vout.append(self.output(remainder, self.anyone_can_spend))
        transaction.vout.append(self.output(ADAPTER_FEE_SATOSHIS, b""))
        # The adaptation the docstring above explains, cited at the line
        # that makes it: mining the funding transaction directly is how
        # this adapter works around a miner that takes only what it is
        # handed (´[PLAN-obs:upstream:eg-007]´).
        self.node.call(
            "generateblock",
            "raw(%s)" % ANYONE_CAN_SPEND_HEX,
            json.dumps([transaction.serialize().hex()]),
        )
        txid = transaction.rehash()
        self.change = {"txid": txid, "vout": 1, "amount": remainder}
        return txid

    # -- one case ---------------------------------------------------------

    def execute(self, fixture: dict, construction=None) -> dict:
        """Runs one fixture and returns its verdict body."""
        if fixture["execution_domain"] != "tapscript":
            raise AdapterError(
                "unsupported execution domain: %s" % fixture["execution_domain"]
            )
        script_bytes = fixture["script"]
        leaf_version = fixture["leaf_version"]
        context = fixture["context"]
        if construction is not None:
            # One transaction shape per case. A request stating both a
            # construction and a generic context states two, and the second
            # would have to overwrite the first's outputs -- so the pair is
            # refused rather than resolved by precedence.
            if context is not None:
                raise AdapterError(
                    "the request states both a construction and a transaction "
                    "context, and this adapter materialises one transaction "
                    "shape rather than two"
                )
            program, leaf_script, control = self.materialise_construction(
                fixture, construction
            )
            transaction = self.build_construction_spend(
                program, leaf_script, control, fixture, construction
            )
            raw = transaction.serialize().hex()
            body = self.judge(raw, fixture["enforcement_layer"])
            body["transaction_weight"] = self.weight_of(raw)
            return body
        program, leaf_script, control = self.taproot_for(script_bytes, leaf_version)
        if context is None:
            transaction = self.build_default_spend(program, leaf_script, control, fixture)
        else:
            transaction = self.build_context_spend(
                program, leaf_script, control, fixture, context
            )
        raw = transaction.serialize().hex()
        body = self.judge(raw, fixture["enforcement_layer"])
        body["transaction_weight"] = self.weight_of(raw)
        return body

    def weight_of(self, raw: str):
        """The materialised transaction's weight, as the node computes it.

        The one resource figure a validating node can actually be asked for.
        It is read back from the node rather than computed here so that the
        observation is the target's own accounting; when a node reports none,
        the answer is that it was not observed, never a number this adapter
        derived.
        """
        try:
            decoded = self.node.call("decoderawtransaction", raw)
        except AdapterError:
            return None
        if not isinstance(decoded, dict):
            return None
        weight = decoded.get("weight")
        if isinstance(weight, bool) or not isinstance(weight, int):
            return None
        return weight

    def build_default_spend(self, program, leaf_script, control, fixture):
        """Builds the minimal transaction that exercises the leaf."""
        messages = self.messages
        funding_txid = self.fund(program, CASE_FUNDING_SATOSHIS)
        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(funding_txid), 0), nSequence=0xFFFFFFFE
            )
        )
        transaction.vout.append(
            self.output(CASE_FUNDING_SATOSHIS - ADAPTER_FEE_SATOSHIS, self.anyone_can_spend)
        )
        transaction.vout.append(self.output(ADAPTER_FEE_SATOSHIS, b""))
        witness = messages.CTxInWitness()
        witness.scriptWitness.stack = list(fixture["initial_stack"]) + [leaf_script, control]
        transaction.wit.vtxinwit.append(witness)
        return transaction

    def build_construction_spend(self, program, leaf_script, control, fixture, construction):
        """Spends the stated predecessor output into the stated outputs.

        Every program-carrying output the transaction has is one the
        construction named, and no change output is added: a second
        program-carrying output could satisfy an output requirement the
        request did not state.

        The explicit fee output is neither requested nor avoidable. An
        Elements transaction accounts for its fee in an output with no
        program at all, so a transaction without one pays nothing and is
        refused by relay for a reason that is not a script verdict.

        A construction stating no output at all is the ordinary case for a
        relation whose program reads no transaction field. The whole input
        value then goes to the fee output and the transaction carries no
        program-carrying output whatsoever -- which is what "no output of
        any role" means, and is a stronger statement than paying the
        remainder to an adapter-chosen program the fixture never asked for.
        """
        messages = self.messages
        outputs = construction["outputs"]
        successors = [entry for entry in outputs if entry["role"] == "successor"]
        if outputs and len(successors) != len(outputs):
            raise AdapterError(
                "the request states an output whose role this adapter does not "
                "materialise"
            )
        if len(successors) > 1:
            raise AdapterError(
                "the request states %d successor outputs, and a spend carries "
                "at most one" % len(successors)
            )
        successor_program = successors[0]["program"] if successors else None
        if successors and not successor_program:
            raise AdapterError(
                "the stated successor output carries no program, which on this "
                "chain is the shape of a fee output rather than of a successor"
            )
        funding_txid = self.fund(program, CASE_FUNDING_SATOSHIS)
        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(funding_txid), 0), nSequence=0xFFFFFFFE
            )
        )
        if successor_program is None:
            transaction.vout.append(self.output(CASE_FUNDING_SATOSHIS, b""))
        else:
            transaction.vout.append(
                self.output(CASE_FUNDING_SATOSHIS - ADAPTER_FEE_SATOSHIS, successor_program)
            )
            transaction.vout.append(self.output(ADAPTER_FEE_SATOSHIS, b""))
        witness = messages.CTxInWitness()
        witness.scriptWitness.stack = list(fixture["initial_stack"]) + [leaf_script, control]
        transaction.wit.vtxinwit.append(witness)
        return transaction

    def build_context_spend(self, program, leaf_script, control, fixture, context):
        """Materialises a declared generic transaction context.

        Only what a disposable regtest chain can be made to say is
        materialised. A declaration this adapter cannot honour is refused by
        name rather than approximated: an approximated context would make the
        target answer a question the fixture did not ask.
        """
        messages = self.messages
        inputs = context["inputs"]
        index = context["current_input_index"]
        if not inputs:
            raise AdapterError("subject.context declares no inputs")
        if index >= len(inputs):
            raise AdapterError(
                "subject.context.current_input_index names no declared input"
            )
        declared_path = context["script_path"]
        # Exact equality, in both directions. Tolerating an empty declared
        # path let a context that stated no leaf script through, and the
        # adapter would then have chosen the leaf the fixture failed to
        # state. Canonical construction cannot produce that shape, but the
        # protocol is what stands between this adapter and a sender.
        if declared_path["script"] != fixture["script"]:
            raise AdapterError(
                "subject.context.script_path.script disagrees with subject.script"
            )
        if declared_path["control"] is not None:
            raise AdapterError(
                "subject.context.script_path.control is stated, and this adapter "
                "derives the control block from the leaf it built"
            )

        # Every unmaterialisable declaration is named before any amount is
        # computed, so that a refusal reports the field it could not honour
        # rather than an arithmetic consequence of it.
        for declared in inputs:
            self.refuse_unmaterialisable_input(declared)
        amounts = self.input_amounts(context)
        funded = []
        for position, declared in enumerate(inputs):
            amount = amounts[position]
            spend_program = program if position == index else self.anyone_can_spend
            # Read before funding: the block that will carry the funding
            # output is the next one, so this tip is exactly the block a
            # relative time lock measures from.
            before = self.node.call("getblockchaininfo")
            funded.append(
                {
                    "declared": declared,
                    "sequence": declared["sequence"],
                    "txid": self.fund(spend_program, amount),
                    "coin_height": before["blocks"] + 1,
                    "parent_median_time": before["mediantime"],
                }
            )

        transaction = messages.CTransaction()
        transaction.version = context["version"]
        transaction.nLockTime = context["locktime"]
        for entry in funded:
            transaction.vin.append(
                messages.CTxIn(
                    messages.COutPoint(txid_to_internal_int(entry["txid"]), 0),
                    nSequence=entry["sequence"],
                )
            )
        for declared in context["outputs"]:
            transaction.vout.append(self.raw_output(declared))
        for position, entry in enumerate(funded):
            witness = messages.CTxInWitness()
            if position == index:
                witness.scriptWitness.stack = list(fixture["initial_stack"]) + [
                    leaf_script,
                    control,
                ]
            else:
                witness.scriptWitness.stack = list(entry["declared"]["witness"])
            transaction.wit.vtxinwit.append(witness)

        self.mature(context, funded)
        return transaction

    def input_amounts(self, context: dict) -> list:
        """How much each declared input carries.

        Value conservation is the chain's rule, not the fixture's. The
        declared outputs fix the total the inputs must carry, and an input
        whose value the fixture leaves unstated is one the executor funds --
        so the unstated inputs share whatever the stated ones leave. That is
        what lets the fixture's transaction be materialised exactly: an
        adapter that funded arbitrary amounts would have to add a change or
        fee output, which would move every output index the fixture states.
        """
        outputs = context["outputs"]
        inputs = context["inputs"]
        total = 0
        for declared in outputs:
            if declared["asset"] is not None and declared["asset"] != self.policy_asset_field:
                raise AdapterError(
                    "%s.asset names an asset this adapter cannot issue on the "
                    "disposable chain" % declared["path"]
                )
            total += explicit_amount(declared["value"], declared["path"] + ".value")

        stated = {}
        for position, declared in enumerate(inputs):
            if declared["spent_value"] is not None:
                stated[position] = explicit_amount(
                    declared["spent_value"], declared["path"] + ".spent_value"
                )
        unstated = [position for position in range(len(inputs)) if position not in stated]
        remainder = total - sum(stated.values())

        if not unstated:
            if remainder != 0:
                raise AdapterError(
                    "the declared inputs and outputs do not conserve value, and no "
                    "input is left unstated for the executor to fund"
                )
            return [stated[position] for position in range(len(inputs))]
        if remainder < len(unstated):
            raise AdapterError(
                "the declared outputs leave too little value for the %d input(s) "
                "the executor funds" % len(unstated)
            )
        share = remainder // len(unstated)
        amounts = dict(stated)
        for position in unstated[:-1]:
            amounts[position] = share
        amounts[unstated[-1]] = remainder - share * (len(unstated) - 1)
        return [amounts[position] for position in range(len(inputs))]

    def refuse_unmaterialisable_input(self, declared: dict) -> None:
        """Names every declared input field this adapter cannot honour.

        A field stated as absent is the fixture leaving the value to the
        executor, which is the case this adapter serves. A field stated with
        a value is a requirement, and every requirement it cannot meet is
        refused by name rather than approximated.
        """
        path = declared["path"]
        if declared["outpoint_txid"] is not None:
            raise AdapterError(
                "%s.outpoint_txid names a transaction this adapter cannot create; "
                "state it absent to have the executor supply the real outpoint" % path
            )
        if declared["outpoint_index"] is not None:
            raise AdapterError(
                "%s.outpoint_index is stated, and this adapter supplies the real "
                "outpoint" % path
            )
        if declared["issuance"] is not None:
            raise AdapterError("%s.issuance is not materialisable by this adapter" % path)
        if (
            declared["spent_asset"] is not None
            and declared["spent_asset"] != self.policy_asset_field
        ):
            raise AdapterError(
                "%s.spent_asset names an asset this adapter cannot issue on the "
                "disposable chain" % path
            )
        if declared["spent_program"] is not None:
            raise AdapterError(
                "%s.spent_program is stated, and this adapter supplies the programs "
                "it funds -- the taproot commitment to the fixture's leaf for the "
                "input under validation, and an anyone-can-spend program for the "
                "rest" % path
            )

    # -- verdict ----------------------------------------------------------

    def judge(self, raw: str, layer: str) -> dict:
        """Asks the node the question the fixture actually stated.

        The two layers are different rules and are asked in different ways.
        Consensus is what a block enforces, so a consensus fixture is judged
        by offering the transaction to `generateblock`; relay is what the
        mempool enforces, so a relay fixture is judged by
        `testmempoolaccept`. Answering either question with the other's
        machinery is how a harness reports an unrelayable but perfectly
        valid spend as an invalid one, or the reverse.

        The previous shape -- mempool first, retried as a block when the
        refusal was a policy script failure -- collapsed the two, and it
        could only retry a refusal that named a script. A policy rule that
        refuses a transaction *without* running the script, such as an
        oversized witness element, then looked like the adapter failing to
        build a transaction rather than like standardness.
        """
        if layer == "relay_policy":
            return self.judge_in_mempool(raw)
        return self.judge_in_block(raw)

    def judge_in_block(self, raw: str) -> dict:
        """Judges at consensus, which is what block validation enforces.

        The script error is read from the client's stderr, which is the
        only place block validation states it. Those bytes are read for
        CLASSIFICATION and go no further: what leaves this method is a
        member of the typed failure vocabulary or nothing at all, never
        the target's text (G12-R04).
        """
        try:
            self.node.call("generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw]))
        except AdapterError as error:
            message = script_error_in(error.client_detail, CONSENSUS_SCRIPT_PREFIX)
            if message is not None:
                return rejection(message)
            raise AdapterError(
                "the node refused the block for a reason that is not a script verdict",
                client_detail=error.client_detail,
            )
        return {"verdict": "accepted", "observed_failure": None}

    def judge_in_mempool(self, raw: str) -> dict:
        """Judges at relay, which is what the mempool enforces."""
        answer = self.node.call("testmempoolaccept", json.dumps([raw]))
        if not isinstance(answer, list) or len(answer) != 1:
            raise AdapterError("the node did not answer testmempoolaccept with one result")
        result = answer[0]
        if result.get("allowed") is True:
            return {"verdict": "accepted", "observed_failure": None}
        reason = result.get("reject-reason")
        if not isinstance(reason, str):
            raise AdapterError("the node rejected without naming a reason")
        for prefix in (POLICY_SCRIPT_PREFIX, CONSENSUS_SCRIPT_PREFIX):
            message = script_error_in(reason, prefix)
            if message is not None:
                return rejection(message)
        # The reject reason is the node's structured RPC answer rather
        # than a child's stderr, so it is the target speaking and may be
        # recorded.
        raise AdapterError(
            "the node refused the transaction for a reason that is not a script "
            "verdict: %s" % reason
        )


class ConfidentialMaterializer:
    """The deterministic materializer for the explicit-asset
    confidential-value representation.

    # Why this is first-party arithmetic over the target's own library

    No reviewed stock interface produces this representation: the common
    blinding path draws fresh blinders, blinds the asset, and generates a
    surjection proof, and mutating a blinded asset back to explicit after
    proof construction is a rejection condition rather than a fallback.
    So the commitments, the blinders, the nonces, and the proofs are
    built here, from the fixture, with no wallet involved.

    What the library supplies is the arithmetic, and the claim class
    follows from that: a proof this class generates is
    conformance-to-the-target's-own-implementation evidence and never
    independent evidence. The independent claim in this arc belongs to
    the first-party commitment oracle on the harness side, which
    recomputes every commitment from published constants and compares it
    against what the chain holds.

    # Deterministic, and never entropic

    Every scalar is derived, the parity search is bounded and moves
    upward from zero, and a proof failure raises rather than becoming a
    source of randomness. There is exactly one answer for a fixture, and
    if it does not work this refuses instead of searching until it does.
    """

    # The context capabilities the arithmetic needs: signing for proof
    # generation, verification for the commitment and generator parses.
    CONTEXT_FLAGS = 0x0301

    # The compressed public-key serialization flag.
    COMPRESSED = 0x0102

    # The widths the library's opaque records occupy.
    GENERATOR_BYTES = 64
    COMMITMENT_BYTES = 64
    PUBKEY_BYTES = 64

    def __init__(self, library_path: str) -> None:
        self.library_path = library_path
        self.library = ctypes.CDLL(library_path)
        self.library.secp256k1_context_create.restype = ctypes.c_void_p
        self.library.secp256k1_context_create.argtypes = [ctypes.c_uint]
        self.context = self.library.secp256k1_context_create(self.CONTEXT_FLAGS)
        if not self.context:
            raise AdapterError("the zero-knowledge library refused a context")

    def generator(self, asset: bytes):
        """The unblinded generator of one explicit asset."""
        out = ctypes.create_string_buffer(self.GENERATOR_BYTES)
        ok = self.library.secp256k1_generator_generate(
            ctypes.c_void_p(self.context), out, ctypes.c_char_p(asset)
        )
        if ok != 1:
            raise AdapterError("the library refused the asset generator")
        return out

    def commit(self, blinder: bytes, amount: int, generator):
        """The commitment point for one amount under one blinder."""
        out = ctypes.create_string_buffer(self.COMMITMENT_BYTES)
        ok = self.library.secp256k1_pedersen_commit(
            ctypes.c_void_p(self.context),
            out,
            ctypes.c_char_p(blinder),
            ctypes.c_uint64(amount),
            generator,
        )
        if ok != 1:
            raise AdapterError("the library refused the value commitment")
        return out

    def serialize_commitment(self, commitment) -> bytes:
        """The commitment, in the target's own thirty-three byte form."""
        out = ctypes.create_string_buffer(33)
        ok = self.library.secp256k1_pedersen_commitment_serialize(
            ctypes.c_void_p(self.context), out, commitment
        )
        if ok != 1:
            raise AdapterError("the library refused to serialize a commitment")
        return out.raw[:33]

    def nonce_field(self, secret: bytes) -> bytes:
        """The nonce field one nonce input produces.

        A point the target transports rather than one it commits to,
        written with the standard compressed pair, so its prefix records
        oddness rather than squareness. The library's own compressed
        serialization is that pair, which is why nothing here writes a
        prefix by hand.
        """
        pubkey = ctypes.create_string_buffer(self.PUBKEY_BYTES)
        ok = self.library.secp256k1_ec_pubkey_create(
            ctypes.c_void_p(self.context), pubkey, ctypes.c_char_p(secret)
        )
        if ok != 1:
            raise AdapterError("the library refused the nonce point")
        out = ctypes.create_string_buffer(33)
        length = ctypes.c_size_t(33)
        self.library.secp256k1_ec_pubkey_serialize(
            ctypes.c_void_p(self.context),
            out,
            ctypes.byref(length),
            pubkey,
            ctypes.c_uint(self.COMPRESSED),
        )
        return out.raw[: length.value]

    def rangeproof(
        self,
        commitment,
        blinder: bytes,
        seed: bytes,
        amount: int,
        program: bytes,
        generator,
    ) -> bytes:
        """One rangeproof, bound to this output's commitment, generator,
        and program.

        The program travels as the proof's additional commitment, which
        is what the target's own validation reads it as, and it is what
        stops a proof built for one output verifying against another.
        """
        proof = ctypes.create_string_buffer(CONFIDENTIAL_RANGEPROOF_CAPACITY)
        length = ctypes.c_size_t(CONFIDENTIAL_RANGEPROOF_CAPACITY)
        ok = self.library.secp256k1_rangeproof_sign(
            ctypes.c_void_p(self.context),
            proof,
            ctypes.byref(length),
            ctypes.c_uint64(CONFIDENTIAL_RANGEPROOF_MINIMUM_VALUE),
            commitment,
            ctypes.c_char_p(blinder),
            ctypes.c_char_p(seed),
            ctypes.c_int(CONFIDENTIAL_RANGEPROOF_EXPONENT),
            ctypes.c_int(CONFIDENTIAL_RANGEPROOF_MINIMUM_BITS),
            ctypes.c_uint64(amount),
            None,
            ctypes.c_size_t(0),
            ctypes.c_char_p(program) if program else None,
            ctypes.c_size_t(len(program)),
            generator,
        )
        if ok != 1:
            # No randomness is added and no retry follows. Under this
            # contract there is exactly one deterministic answer, and if
            # it does not work the ceremony refuses.
            raise AdapterError("proof generation refused, and no retry follows")
        return proof.raw[: length.value]

    def resolve(self, handle: str, asset: bytes, programs: list, profiles: dict) -> dict:
        """Derives one registered case completely, and returns it with its
        own digest.

        The parity search is the bounded deterministic one: upward from
        zero until the serialized commitment prefixes satisfy the rule
        their arity states, and a typed refusal at the bound. See
        `confidential_prefixes_admitted` for the rule.
        """
        entry = CONFIDENTIAL_FIXTURE_CATALOGUE[handle]
        outputs = entry["outputs"]
        if len(programs) != len(outputs):
            raise AdapterError(
                "the request states %d destinations and the fixture states %d outputs"
                % (len(programs), len(outputs))
            )
        input_sum = int.from_bytes(entry["input_blinder_sum"], "big")
        generator = self.generator(asset)
        bound = min(entry["retry_limit"], CONFIDENTIAL_MAX_PARITY_COUNTER)
        for parity in range(bound + 1):
            openings = self.derive_at(handle, outputs, programs, input_sum, generator, parity)
            prefixes = tuple(opening["value_commitment"][0] for opening in openings)
            if confidential_prefixes_admitted(prefixes):
                digest = confidential_tagged_hash(
                    CONFIDENTIAL_DIGEST_TAG,
                    confidential_digest_transcript(
                        handle,
                        profiles,
                        entry["retry_limit"],
                        parity,
                        asset,
                        outputs,
                        programs,
                        openings,
                    ),
                )
                return {
                    "parity_counter": parity,
                    "openings": openings,
                    "digest": digest,
                    "outputs": outputs,
                    "generator": generator,
                }
        raise AdapterError("the parity search reached its bound without the required pair")

    def derive_at(
        self, handle: str, outputs: tuple, programs: list, input_sum: int, generator, parity: int
    ) -> list:
        """Every opening of one case, at one parity counter."""
        blinders = []
        derived_sum = 0
        for index, output in enumerate(outputs):
            if output["role"] == "balancing":
                blinders.append(None)
                continue
            raw = confidential_search_scalar(
                handle, index, CONFIDENTIAL_ROLE_VALUE_BLINDER, parity
            )
            derived_sum = (derived_sum + int.from_bytes(raw, "big")) % CONFIDENTIAL_GROUP_ORDER
            blinders.append(raw)
        # The balancing blinder is solved and never derived: the input
        # blinder sum minus the others', in the group. For this
        # predecessor the input contributes zero, so the two come out
        # ordered additive inverses.
        balancing = (input_sum - derived_sum) % CONFIDENTIAL_GROUP_ORDER
        if balancing == 0:
            raise AdapterError("the solved balancing blinder is zero")
        balancing_bytes = balancing.to_bytes(32, "big")
        blinders = [balancing_bytes if entry is None else entry for entry in blinders]
        # The independent recheck: the solve produced the value, and this
        # reads every blinder back and adds them again.
        recheck = 0
        for raw in blinders:
            recheck = (recheck + int.from_bytes(raw, "big")) % CONFIDENTIAL_GROUP_ORDER
        if recheck != input_sum % CONFIDENTIAL_GROUP_ORDER:
            raise AdapterError("the recheck of the blinder sums disagreed")
        openings = []
        for index, output in enumerate(outputs):
            blinder = blinders[index]
            nonce_input = confidential_search_scalar(
                handle, index, CONFIDENTIAL_ROLE_NONCE_SECRET, parity
            )
            seed = confidential_derive(
                handle, index, CONFIDENTIAL_ROLE_RANGEPROOF_SEED, parity, 0
            )
            commitment = self.commit(blinder, output["semantic_amount"], generator)
            openings.append(
                {
                    "value_blinder": blinder,
                    "nonce_input": nonce_input,
                    "rangeproof_seed": seed,
                    "commitment": commitment,
                    "value_commitment": self.serialize_commitment(commitment),
                    "nonce_field": self.nonce_field(nonce_input),
                }
            )
        return openings


def locate_zero_knowledge_library(elementsd: str, stated) -> str:
    """The shared zero-knowledge library this adapter's materializer calls,
    or nothing.

    # Why it is located rather than configured

    The library is the node's own, built in the node's own tree, so the
    path follows from the binary the operator already named. An operator
    may state one explicitly; nothing here searches a system path, and
    nothing here installs anything.

    # Why a shared object is linked from the archive

    The node's build produces the library as a static archive, which
    ctypes cannot open. Linking a shared object from that same archive is
    a re-packaging of bytes the operator already built, not a second
    build of a second library: the code is the archive's, the compiler is
    the host's, and the result lives in this run's disposable directory
    and dies with it. Where no compiler or no archive is present the
    answer is nothing, and the adapter then advertises no confidential
    funding at all -- which is the honest answer rather than a broken
    capability.
    """
    if stated:
        return stated if os.path.exists(stated) else ""
    root = os.path.dirname(os.path.dirname(os.path.abspath(elementsd)))
    library = os.path.join(root, "src", "secp256k1", "lib", "libsecp256k1.so")
    if os.path.exists(library):
        return library
    archive = os.path.join(root, "src", "secp256k1", "lib", "libsecp256k1.a")
    if not os.path.exists(archive):
        return ""
    compiler = shutil.which("cc") or shutil.which("gcc")
    if compiler is None:
        return ""
    linked = os.path.join(tempfile.mkdtemp(prefix="tripod-zk-"), "libsecp256k1.so")
    completed = subprocess.run(
        [
            compiler,
            "-shared",
            "-o",
            linked,
            "-Wl,--whole-archive",
            archive,
            "-Wl,--no-whole-archive",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0 or not os.path.exists(linked):
        return ""
    return linked


class OperationExecutor:
    """Performs one Guide-12 section 16.2 operation step against the node.

    # What this class knows, and what it deliberately does not

    Two things: how to create spendable outputs at a witness program a
    caller named, and how to hand the node a complete transaction and
    report what it did with it. Neither is compact-ASH vocabulary. It
    does not know what an ASH is, which outputs an operation will spend,
    what a relation is, or what any answer is evidence of -- all of that
    lives with the caller, which is the whole content of the section 16.2
    boundary.

    # Why funding is raw rather than wallet-driven

    The wallet is used for exactly one thing: spelling an address, which
    `rawissueasset` requires and which cannot be produced without one.
    Every transaction this class builds is assembled here and confirmed
    by mining it directly, for the reason `CaseExecutor.fund` states: an
    unconfirmed parent is invisible to `generateblock`, which pulls
    nothing from the mempool, so a chain of unconfirmed funding would
    leave every later step spending coins the block it is judged in
    cannot see.

    No key, seed, or signature is involved. The funding source is the
    chain's anyone-can-spend free coin, which is public data and nobody's
    secret `(ADR-015 rule test-material)`.
    """

    def __init__(self, executor: "CaseExecutor", wallet_name=None) -> None:
        self.executor = executor
        self.wallet_name = wallet_name or "operations"
        self.prepared = False
        # One retained reserve output per issued asset, so that later
        # funding steps have something of that asset to spend. Public
        # chain data; the key is the target's own spelling of the asset.
        self.reserves = {}
        # What this adapter put in each sponsor coin it created, keyed by
        # the outpoint that holds it. A signing step needs the value the
        # coin carries -- the segwit-v0 signature hash commits to it --
        # and a coin this adapter never created is one it cannot
        # authorize, which is a refusal rather than a guess.
        self.sponsor_coins = {}
        self.sponsor_key_cache = None
        # The deterministic materializer, where the zero-knowledge
        # library the node's own build produced could be reached. None
        # is the honest answer where it could not, and the handshake
        # then advertises no confidential funding at all.
        self.materializer = None

    def prepare(self) -> None:
        """Creates the wallet, and parks all but a working slice of the
        free coin.

        # Why the free coin has to be split before anything is issued

        The chain's free coin holds the target's entire money bound, and
        the target refuses a transaction whose outputs total more than
        that bound. A transaction that spent the free coin *and* created
        an issued asset would therefore be refused for
        `bad-txns-txouttotal-toolarge` no matter how little of the asset
        it issued: the policy side alone is already the whole bound.

        So the free coin is split once, in a transaction that moves only
        the policy asset and is inside the bound by construction, into a
        small working coin this lane spends and a parked remainder it
        never touches. Every later transaction spends the working coin,
        so the policy side of an issuance is a rounding error rather than
        the whole supply.

        The parked output sits at the same anyone-can-spend program the
        free coin did. It is not hidden, reserved, or owned: it is public
        chain data any party could spend, and this lane simply does not.

        The working output sits at the witness-carrying form of that same
        program instead, because the issuance that spends it is only
        checked at all if its input can carry a witness stack -- see
        `ANYONE_CAN_SPEND_WITNESS_HEX`. It is anyone-can-spend on the
        same terms: the stack that spends it is the program itself, which
        is a published constant and nobody's secret.
        """
        if self.prepared:
            return
        executor = self.executor
        messages = executor.messages
        node = executor.node
        try:
            node.call("createwallet", self.wallet_name)
        except AdapterError:
            # Already present on a chain directory a previous process
            # built. Loading it is the same wallet either way.
            try:
                node.call("loadwallet", self.wallet_name)
            except AdapterError:
                pass

        source = executor.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")
        working = OPERATION_WORKING_SATOSHIS
        parked = source["amount"] - working - ADAPTER_FEE_SATOSHIS
        if parked < 0:
            raise AdapterError("the free coin cannot be split into a working slice")

        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(source["txid"]), source["vout"]),
                nSequence=0xFFFFFFFE,
            )
        )
        transaction.vout.append(
            executor.output(working, executor.anyone_can_spend_witness)
        )
        transaction.vout.append(executor.output(parked, executor.anyone_can_spend))
        transaction.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))
        txid = self.mine(transaction, "free-coin split")
        executor.change = {"txid": txid, "vout": 0, "amount": working}
        self.prepared = True

    # -- helpers ----------------------------------------------------------

    def asset_field(self, printed: str) -> bytes:
        """The explicit asset field for an identity the target printed.

        A target prints an asset identity in the reverse of the order it
        commits to it in, and a transaction carries the committed order.
        The reversal happens here, at the boundary.
        """
        return bytes([EXPLICIT_PREFIX]) + bytes.fromhex(printed)[::-1]

    def printed_asset(self, field: bytes) -> str:
        """The target's own spelling of an explicit asset field."""
        return field[1:][::-1].hex()

    def mine(self, transaction, note: str = "") -> str:
        """Confirms one transaction by mining exactly it.

        A refusal here is the adapter failing to build something the
        chain accepts, which is an infrastructure failure and never a
        target verdict. The transaction this adapter built, and the
        node's structured reason for refusing it, both go to the
        quarantine so the failure can be diagnosed; neither reaches a
        first-party record, and neither reaches the typed diagnostic
        stream either (G12-R04).

        The serialized transaction is this adapter's own material rather
        than a child's, and it is quarantined all the same. The
        quarantine is not only for bytes of untrusted origin: it is for
        unbounded text, and a diagnostic stream that admitted one
        transaction hex would have no ground on which to refuse the next
        arbitrary string.
        """
        raw = transaction.serialize().hex()
        try:
            self.executor.node.call(
                "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
            )
        except AdapterError:
            record = quarantined(
                "the %s transaction this adapter built, refused" % (note or "unnamed"),
                raw,
            )
            log(
                "the %s transaction this adapter built was refused; its bytes "
                "are elements-output record %d" % (note or "unnamed", record)
            )
            # The mempool's structured reason, which names the rule; the
            # block error names only the check that reported it.
            try:
                answer = self.executor.node.call("testmempoolaccept", json.dumps([raw]))
            except AdapterError:
                pass
            else:
                reason = quarantined(
                    "testmempoolaccept on the refused %s transaction"
                    % (note or "unnamed"),
                    json.dumps(answer),
                )
                log("the mempool's reason is elements-output record %d" % reason)
            raise
        return transaction.rehash()

    def created(self, txid: str, index: int) -> dict:
        """One funded output, read back from the node rather than echoed.

        The amount and the script are what the node stored, not what this
        adapter asked for. An adapter that echoed the request would be
        reporting its own intention as a chain fact, and the caller has
        no way to tell the two apart from the outside.
        """
        entry = self.executor.node.call("gettxout", txid, str(index))
        if entry is None:
            raise AdapterError(
                "the node does not hold the output this step just created"
            )
        return {
            "outpoint": {"txid": txid, "vout": index},
            "asset": entry["asset"],
            "amount_satoshis": int(round(float(entry["value"]) * 100_000_000)),
            "script": entry["scriptPubKey"]["hex"],
        }

    # -- funding ----------------------------------------------------------

    def issue(self, subject: dict) -> dict:
        """Issues the disposable asset and creates the asked-for output.

        The asset identity is the target's, and this is the step whose
        whole content is learning it: an Elements asset identifier is
        derived from the issuing input's outpoint and the contract hash,
        so no caller can choose one and no adapter can predict one.
        """
        executor = self.executor
        messages = executor.messages
        node = executor.node
        # Before the change output is read: preparing splits the free
        # coin and replaces it, so a source read first would name a coin
        # this transaction is no longer allowed to spend.
        self.prepare()
        source = executor.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")

        remainder = source["amount"] - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError("the adapter's change output cannot fund an issuance")

        base = messages.CTransaction()
        base.version = 2
        base.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(source["txid"]), source["vout"]),
                nSequence=0xFFFFFFFE,
            )
        )
        base.vout.append(executor.output(remainder, executor.anyone_can_spend))
        base.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))

        address = node.call("getnewaddress", wallet=self.wallet_name)
        answer = node.call(
            "rawissueasset",
            base.serialize().hex(),
            json.dumps(
                [
                    {
                        "asset_amount": ISSUED_ASSET_UNITS,
                        "asset_address": address,
                        "blind": False,
                    }
                ]
            ),
        )
        if not isinstance(answer, list) or len(answer) != 1:
            raise AdapterError("the node did not answer rawissueasset with one issuance")
        issuance = answer[0]
        printed = issuance.get("asset")
        if not isinstance(printed, str):
            raise AdapterError("the node issued an asset and named none")

        # The identity is fixed by the issuing input and the contract
        # hash, so the outputs may be rebuilt freely: what the caller
        # asked for at its own program, a reserve this adapter retains so
        # later steps have something of the asset to spend, and the
        # policy change and fee the base transaction already carried.
        issued = messages.CTransaction()
        issued.deserialize(io.BytesIO(bytes.fromhex(issuance["hex"])))
        field = self.asset_field(printed)
        total = ISSUED_ASSET_UNITS * 100_000_000
        wanted = subject["amount_per_output"] * subject["outputs"]
        if wanted > total:
            raise AdapterError(
                "the step asks for more of the asset than the issuance creates"
            )
        issued.vout = []
        for _ in range(subject["outputs"]):
            issued.vout.append(
                executor.output(
                    subject["amount_per_output"], subject["output_program"], field
                )
            )
        issued.vout.append(
            executor.output(total - wanted, executor.anyone_can_spend, field)
        )
        issued.vout.append(executor.output(remainder, executor.anyone_can_spend))
        issued.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))

        # The issuing input names the program its funding coin commits
        # to. That stack is what spends the coin, and it is also the only
        # thing that gives this transaction a witness section -- without
        # one the target never checks the issuance and reports the
        # balanced transaction as `bad-txns-in-ne-out` (T4-012).
        issued.wit.vtxinwit = [messages.CTxInWitness() for _ in issued.vin]
        issued.wit.vtxinwit[0].scriptWitness.stack = list(
            ANYONE_CAN_SPEND_WITNESS_STACK
        )

        txid = self.mine(issued, "issuance")
        reserve_index = subject["outputs"]
        self.reserves[printed] = {
            "txid": txid,
            "vout": reserve_index,
            "amount": total - wanted,
            "field": field,
        }
        executor.change = {
            "txid": txid,
            "vout": reserve_index + 1,
            "amount": remainder,
        }
        return {
            "issued_asset": printed,
            "funded_outputs": [
                self.created(txid, index) for index in range(subject["outputs"])
            ],
        }

    def pay(self, subject: dict) -> dict:
        """Creates outputs of an already-issued asset at a stated program."""
        executor = self.executor
        messages = executor.messages
        printed = subject["asset"]
        if printed is None:
            raise AdapterError("a non-issuing funding step named no asset")
        reserve = self.reserves.get(printed)
        if reserve is None:
            raise AdapterError("no earlier step of this run issued that asset")
        source = executor.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")

        wanted = subject["amount_per_output"] * subject["outputs"]
        if wanted > reserve["amount"]:
            # Named in satoshis, because whether this is a fixture asking
            # for more than the target's money bound or a run that simply
            # issued too little is the whole question, and a message that
            # states neither amount cannot tell the two apart.
            raise AdapterError(
                "the step asks for more of the asset than this run holds: "
                "wanted %d, reserve holds %d, this run issued %d"
                % (wanted, reserve["amount"], ISSUED_ASSET_UNITS * 100_000_000)
            )
        remainder = source["amount"] - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError("the adapter's change output cannot pay the fee")

        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(
                    txid_to_internal_int(reserve["txid"]), reserve["vout"]
                ),
                nSequence=0xFFFFFFFE,
            )
        )
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(source["txid"]), source["vout"]),
                nSequence=0xFFFFFFFE,
            )
        )
        field = reserve["field"]
        for _ in range(subject["outputs"]):
            transaction.vout.append(
                executor.output(
                    subject["amount_per_output"], subject["output_program"], field
                )
            )
        transaction.vout.append(
            executor.output(
                reserve["amount"] - wanted, executor.anyone_can_spend, field
            )
        )
        transaction.vout.append(executor.output(remainder, executor.anyone_can_spend))
        transaction.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))

        txid = self.mine(transaction, "asset payment")
        reserve_index = subject["outputs"]
        self.reserves[printed] = {
            "txid": txid,
            "vout": reserve_index,
            "amount": reserve["amount"] - wanted,
            "field": field,
        }
        executor.change = {
            "txid": txid,
            "vout": reserve_index + 1,
            "amount": remainder,
        }
        return {
            "issued_asset": None,
            "funded_outputs": [
                self.created(txid, index) for index in range(subject["outputs"])
            ],
        }

    # -- confidential funding ---------------------------------------------

    def exact_protocol_coin(self, printed: str, reserve: dict, total: int) -> dict:
        """Mines one coin holding exactly the protocol amount the fixture
        fixes.

        # Why a separate transaction exists at all

        The slice's shape is a requirement and not an example: one
        explicit protocol-asset input, and exactly two protocol outputs
        whose semantic values sum to that input's amount. A funding
        transaction that also returned protocol-asset change would carry
        a third member of the protocol asset, which is a member the
        region classifier refuses and a sum the balance equation would no
        longer close over.

        So the remainder is split off HERE, in an ordinary explicit
        transaction that is not the funding transaction and is not
        reported as one. What the funding transaction then spends is a
        coin whose amount is exactly the fixture's total.
        """
        executor = self.executor
        messages = executor.messages
        source = executor.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")
        if total > reserve["amount"]:
            raise AdapterError(
                "the fixture asks for more of the asset than this run holds: "
                "wanted %d, reserve holds %d" % (total, reserve["amount"])
            )
        remainder = source["amount"] - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError("the adapter's change output cannot pay the fee")

        transaction = messages.CTransaction()
        transaction.version = 2
        for coin in (reserve, source):
            transaction.vin.append(
                messages.CTxIn(
                    messages.COutPoint(txid_to_internal_int(coin["txid"]), coin["vout"]),
                    nSequence=0xFFFFFFFE,
                )
            )
        field = reserve["field"]
        transaction.vout.append(executor.output(total, executor.anyone_can_spend, field))
        transaction.vout.append(
            executor.output(reserve["amount"] - total, executor.anyone_can_spend, field)
        )
        transaction.vout.append(executor.output(remainder, executor.anyone_can_spend))
        transaction.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))
        txid = self.mine(transaction, "exact protocol coin")
        self.reserves[printed] = {
            "txid": txid,
            "vout": 1,
            "amount": reserve["amount"] - total,
            "field": field,
        }
        executor.change = {"txid": txid, "vout": 2, "amount": remainder}
        return {"txid": txid, "vout": 0, "amount": total, "field": field}

    def mine_and_read_back(self, transaction, note: str) -> dict:
        """Mines one transaction and reads the block and the raw bytes back
        from the node.

        Every fact here is the node's, read after the fact: the block the
        node says it made, the transactions that block holds, the height
        it holds them at, and the bytes the node reports for the
        transaction. Nothing is projected from the object this adapter
        just built.
        """
        executor = self.executor
        node = executor.node
        raw = transaction.serialize().hex()
        try:
            answer = node.call("generateblock", MINING_DESCRIPTOR, json.dumps([raw]))
        except AdapterError as error:
            log("the target refused the %s transaction: %s" % (note, error))
            raise
        block_hash = answer["hash"] if isinstance(answer, dict) else answer
        block = node.call("getblock", block_hash)
        txid = transaction.rehash()
        if txid not in block.get("tx", []):
            raise AdapterError("the block the node made does not hold the transaction")
        reported = node.call("getrawtransaction", txid, json.dumps(True), block_hash)
        return {
            "transaction_id": reported["txid"],
            "witness_transaction_id": reported["hash"],
            "block_hash": block_hash,
            "block_height": int(block["height"]),
            "raw_transaction": list(bytes.fromhex(reported["hex"])),
        }

    def read_back_confirmed(self, mined, txid: str) -> dict:
        """Reads one confirmed transaction back out of the node.

        The sibling of `mine_and_read_back` for the submission path,
        which differs in one way that matters: this adapter never held
        the transaction as an object. It was handed bytes, and every
        field below is read back from the node afterwards rather than
        projected from what arrived -- the block the node says it made,
        the transactions that block holds, the height, and the bytes the
        node reports for the identity it computed.

        That is the whole reason the member exists. A caller that had to
        re-derive the accepted witness from the bytes it submitted would
        be checking its own value against itself; what it needs is the
        target's own copy, and only the target has one.
        """
        node = self.executor.node
        block_hash = mined["hash"] if isinstance(mined, dict) else mined
        block = node.call("getblock", block_hash)
        if txid not in block.get("tx", []):
            raise AdapterError("the block the node made does not hold the submitted transaction")
        reported = node.call("getrawtransaction", txid, json.dumps(True), block_hash)
        return {
            "transaction_id": reported["txid"],
            "witness_transaction_id": reported["hash"],
            "block_hash": block_hash,
            "block_height": int(block["height"]),
            "raw_transaction": list(bytes.fromhex(reported["hex"])),
        }

    def fund_confidential(self, subject: dict) -> dict:
        """Materializes, submits, mines, and reads back one confidential
        predecessor.

        # The order is the shape's own

        The fixture is resolved before any cryptographic work, and an
        unknown handle or a drifted digest refuses there. Then the exact
        protocol coin is mined; then every commitment, nonce, and
        rangeproof is materialized from the registered openings; then the
        transaction is assembled with its proofs already final; and only
        then is it mined. Nothing is repaired afterwards, and no proof is
        regenerated.
        """
        executor = self.executor
        messages = executor.messages
        materializer = self.materializer
        if materializer is None:
            raise AdapterError("this adapter has no deterministic materializer")
        if subject["issue_asset"]:
            # The fixture binds the protocol asset, and a step that
            # issued the asset would be binding a value that does not
            # exist until the step has already run. The asset is issued
            # by an earlier explicit step, and named here.
            raise AdapterError(
                "this adapter materializes confidential funding only against an "
                "asset an earlier step issued"
            )
        printed = subject["asset"]
        reserve = self.reserves.get(printed)
        if reserve is None:
            raise AdapterError("no earlier step of this run issued that asset")

        binding = subject["binding"]
        handle = binding["fixture_handle"]
        if handle not in CONFIDENTIAL_FIXTURE_CATALOGUE:
            raise AdapterError("no fixture is registered under that handle")
        programs = [destination["output_program"] for destination in subject["destinations"]]
        asset = bytes.fromhex(printed)[::-1]
        resolved = materializer.resolve(handle, asset, programs, binding["profiles"])
        if bytes(binding["fixture_digest"]) != resolved["digest"]:
            raise AdapterError("the fixture registered under that handle carries another digest")

        outputs = resolved["outputs"]
        openings = resolved["openings"]
        total = sum(output["semantic_amount"] for output in outputs)
        coin = self.exact_protocol_coin(printed, reserve, total)
        source = executor.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")
        remainder = source["amount"] - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError("the adapter's change output cannot pay the fee")

        transaction = messages.CTransaction()
        transaction.version = 2
        for spent in (coin, source):
            transaction.vin.append(
                messages.CTxIn(
                    messages.COutPoint(txid_to_internal_int(spent["txid"]), spent["vout"]),
                    nSequence=0xFFFFFFFE,
                )
            )
        for index, output in enumerate(outputs):
            opening = openings[index]
            member = executor.output(
                output["semantic_amount"], programs[index], coin["field"]
            )
            # The value field carries the commitment rather than the
            # amount, and the asset field stays explicit. That pairing is
            # the whole representation: the generator is the asset's own
            # unblinded one, so a rangeproof is required and the
            # surjection-proof field is empty.
            member.nValue.vchCommitment = opening["value_commitment"]
            member.nNonce = messages.CTxOutNonce(opening["nonce_field"])
            transaction.vout.append(member)
        transaction.vout.append(executor.output(remainder, executor.anyone_can_spend))
        transaction.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))

        transaction.wit.vtxoutwit = [
            messages.CTxOutWitness() for _ in transaction.vout
        ]
        for index, output in enumerate(outputs):
            opening = openings[index]
            transaction.wit.vtxoutwit[index].vchRangeproof = materializer.rangeproof(
                opening["commitment"],
                opening["value_blinder"],
                opening["rangeproof_seed"],
                output["semantic_amount"],
                programs[index],
                resolved["generator"],
            )
            transaction.wit.vtxoutwit[index].vchSurjectionproof = b""

        readback = self.mine_and_read_back(transaction, "confidential funding")
        executor.change = {
            "txid": readback["transaction_id"],
            "vout": len(outputs),
            "amount": remainder,
        }
        reported = []
        for index, output in enumerate(outputs):
            opening = openings[index]
            reported.append(
                {
                    "outpoint": {"txid": readback["transaction_id"], "vout": index},
                    "explicit_asset": printed,
                    "value_commitment": list(opening["value_commitment"]),
                    "nonce": list(opening["nonce_field"]),
                    "script": programs[index].hex(),
                    "output_witness_index": index,
                    # Declared and empty, because an absent field cannot
                    # be observed to be empty.
                    "surjection_proof": [],
                    "rangeproof": list(transaction.wit.vtxoutwit[index].vchRangeproof),
                }
            )
        return {
            "issued_asset": None,
            "funded_outputs": [],
            "accepted_txid": None,
            "confidential_funded_outputs": reported,
            "mined_readback": readback,
            "transaction_weight": executor.weight_of(transaction.serialize().hex()),
        }

    # -- sponsorship ------------------------------------------------------

    def sponsor_key(self):
        """The key this adapter authorizes sponsor coins with.

        Built once from `SPONSOR_SECRET_HEX` and kept, so that the
        program below is the same program for the whole run.
        """
        if self.sponsor_key_cache is None:
            key_module = self.executor.key_module
            if key_module is None:
                raise AdapterError(
                    "this adapter's framework ships no key implementation, so "
                    "it can authorize nothing"
                )
            key = key_module.ECKey()
            key.set(bytes.fromhex(SPONSOR_SECRET_HEX), True)
            if not key.is_valid:
                raise AdapterError("the sponsor secret is not a valid key")
            self.sponsor_key_cache = key
        return self.sponsor_key_cache

    def sponsor_program(self) -> bytes:
        """The witness program sponsor coins are paid to.

        Version zero over the twenty-byte hash of this adapter's own
        public key, which is the one program class the candidate's
        reviewed sponsor profile admits. The class is not this adapter's
        choice -- the emitted coordinator checks it -- and building it
        from the framework's own hash helper keeps the two from drifting.
        """
        pubkey = self.sponsor_key().get_pubkey().get_bytes()
        return bytes([0x00, 0x14]) + self.executor.script.hash160(pubkey)

    def fund_sponsor(self, subject: dict) -> dict:
        """Creates sponsor coins out of this adapter's own reserve.

        # Why the reserve is the policy asset and not something issued

        The candidate's fee output carries the reserve asset, and a fee
        this target's mempool weighs is one paid in the chain's own
        policy asset. A reserve this lane issued would produce a
        transaction whose fee the relay layer does not recognize as a
        fee -- an honest answer, but a relay refusal rather than the
        acceptance the step is asked for. So the reserve is what the
        chain already uses as one, and the caller learns which asset
        that is from the answer rather than by stating it.

        The coins are unblinded, because they are built here rather than
        by a wallet. That is not a detail: the coordinator introspects
        the sponsor input's asset, and an introspection reads an
        explicit field. A confidential coin would be refused by the
        candidate's own program.
        """
        executor = self.executor
        messages = executor.messages
        source = executor.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")

        outputs = subject["sponsor_outputs"]
        amount = subject["amount_per_sponsor_output"]
        wanted = outputs * amount
        remainder = source["amount"] - wanted - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError(
                "the adapter's working coin cannot fund this sponsor step: "
                "wanted %d plus a fee of %d, and it holds %d"
                % (wanted, ADAPTER_FEE_SATOSHIS, source["amount"])
            )

        program = self.sponsor_program()
        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(source["txid"]), source["vout"]),
                nSequence=0xFFFFFFFE,
            )
        )
        for _ in range(outputs):
            transaction.vout.append(executor.output(amount, program))
        # The change goes back to the BARE anyone-can-spend program, not
        # the witness-carrying form. The two are not interchangeable:
        # the witness form has to be spent with its own program on the
        # stack, and every later step here spends this coin without
        # pushing one. Only the issuance needs the witness form, because
        # only an issuance is checked solely when its transaction
        # carries a witness section (T4-012), and `issue` has already
        # consumed it by the time this step runs.
        transaction.vout.append(executor.output(remainder, executor.anyone_can_spend))
        transaction.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))

        txid = self.mine(transaction, "sponsor funding")
        executor.change = {"txid": txid, "vout": outputs, "amount": remainder}

        created = []
        for index in range(outputs):
            entry = self.created(txid, index)
            # Retained so a later signing step knows what the coin holds
            # without asking the caller to restate it.
            #
            # The FIELD is retained and not the number. A signature hash
            # commits to the spent output's value field verbatim, and
            # that field has two forms -- an explicit one that carries
            # an amount and a committed one that carries a point. A
            # cache holding an integer can only ever reconstruct the
            # first, so a coin of the second kind would have nothing
            # here to be signed against. Writing the field itself is
            # what lets one signing step serve both.
            self.sponsor_coins["%s:%d" % (txid, index)] = {
                "value_field": explicit_value_field(entry["amount_satoshis"]),
                "asset": entry["asset"],
            }
            created.append(entry)
        return {"issued_asset": None, "funded_outputs": created, "accepted_txid": None}

    def fund_confidential_sponsor(self, subject: dict) -> dict:
        """Creates sponsor coins whose VALUES are committed and whose
        ASSET stays explicit.

        # The pairing, and why it is not a middle position

        The sponsor's asset is the chain's own reserve and it is written
        explicitly, because the candidate's covenant INTROSPECTS it: an
        introspection reads a field, and a committed asset is not a field
        anything reads an identity out of. The value carries a commitment
        because the value is the part a sponsor has a reason to keep, and
        nothing in the covenant reads it. So the two fields differ for
        two separate reasons rather than as a compromise between them.

        # Why this transaction has an output nothing will ever spend

        Its single input is this adapter's explicit change coin, which
        contributes a zero value blinder. One committed output would
        therefore have to be committed under a zero blinder to close the
        sum, and that commitment is recomputable from a guessed amount --
        it has the shape of a blinded output and hides nothing. The
        fixture states two outputs for that reason: the coin the sponsor
        will spend is a primary whose blinder is derived, and the second
        solves the balance. Only the first is offered onward.

        # What is checked before any commitment is built

        That the first destination is the program THIS adapter can
        authorize a spend of. A sponsor coin paid anywhere else is a coin
        this adapter cannot sign, and the run would discover that two
        steps later as a signing refusal whose cause looked like the
        candidate's.

        # Why the created coins are not read back the usual way

        `created` reads `gettxout`'s `value`, and a committed output does
        not carry one. The node is not asked for what it does not hold:
        the outputs are reported the way the confidential receipt path
        reports its own, as commitments beside the mined bytes, and the
        caller's own oracle is what recomputes them.
        """
        executor = self.executor
        messages = executor.messages
        materializer = self.materializer
        if materializer is None:
            raise AdapterError("this adapter has no deterministic materializer")

        binding = subject["binding"]
        handle = binding["fixture_handle"]
        if handle not in CONFIDENTIAL_FIXTURE_CATALOGUE:
            raise AdapterError("no fixture is registered under that handle")
        programs = [destination["output_program"] for destination in subject["destinations"]]
        authorizable = self.sponsor_program()
        if programs[0] != authorizable:
            raise AdapterError(
                "the step's first destination is not the program this adapter "
                "can authorize a spend of, so the coin it asks for would be "
                "unspendable by the run that asked for it"
            )

        field = executor.policy_asset_field
        if field is None:
            raise AdapterError("this adapter has not learned the chain's reserve asset")
        # The reserve travels in the target's internal order inside the
        # field and in the printed order everywhere a caller reads one.
        asset = field[1:]
        printed = asset[::-1].hex()

        resolved = materializer.resolve(handle, asset, programs, binding["profiles"])
        if bytes(binding["fixture_digest"]) != resolved["digest"]:
            raise AdapterError("the fixture registered under that handle carries another digest")

        outputs = resolved["outputs"]
        openings = resolved["openings"]
        total = sum(output["semantic_amount"] for output in outputs)
        source = executor.change
        if source is None:
            raise AdapterError("the adapter has no spendable change output")
        remainder = source["amount"] - total - ADAPTER_FEE_SATOSHIS
        if remainder < 0:
            raise AdapterError(
                "the adapter's working coin cannot fund this sponsor step: "
                "wanted %d plus a fee of %d, and it holds %d"
                % (total, ADAPTER_FEE_SATOSHIS, source["amount"])
            )

        transaction = messages.CTransaction()
        transaction.version = 2
        transaction.vin.append(
            messages.CTxIn(
                messages.COutPoint(txid_to_internal_int(source["txid"]), source["vout"]),
                nSequence=0xFFFFFFFE,
            )
        )
        for index, output in enumerate(outputs):
            opening = openings[index]
            member = executor.output(output["semantic_amount"], programs[index], field)
            member.nValue.vchCommitment = opening["value_commitment"]
            member.nNonce = messages.CTxOutNonce(opening["nonce_field"])
            transaction.vout.append(member)
        transaction.vout.append(executor.output(remainder, executor.anyone_can_spend))
        transaction.vout.append(executor.output(ADAPTER_FEE_SATOSHIS, b""))

        transaction.wit.vtxoutwit = [messages.CTxOutWitness() for _ in transaction.vout]
        for index, output in enumerate(outputs):
            opening = openings[index]
            transaction.wit.vtxoutwit[index].vchRangeproof = materializer.rangeproof(
                opening["commitment"],
                opening["value_blinder"],
                opening["rangeproof_seed"],
                output["semantic_amount"],
                programs[index],
                resolved["generator"],
            )
            transaction.wit.vtxoutwit[index].vchSurjectionproof = b""

        readback = self.mine_and_read_back(transaction, "confidential sponsor funding")
        txid = readback["transaction_id"]
        executor.change = {"txid": txid, "vout": len(outputs), "amount": remainder}

        reported = []
        for index, output in enumerate(outputs):
            opening = openings[index]
            # The signing step's own record of what this coin holds. The
            # field is the commitment, because that is what a spend of
            # this coin commits to.
            self.sponsor_coins["%s:%d" % (txid, index)] = {
                "value_field": opening["value_commitment"],
                "asset": printed,
            }
            reported.append(
                {
                    "outpoint": {"txid": txid, "vout": index},
                    "explicit_asset": printed,
                    "value_commitment": list(opening["value_commitment"]),
                    "nonce": list(opening["nonce_field"]),
                    "script": programs[index].hex(),
                    "output_witness_index": index,
                    # Declared and empty, because an absent field cannot
                    # be observed to be empty.
                    "surjection_proof": [],
                    "rangeproof": list(transaction.wit.vtxoutwit[index].vchRangeproof),
                }
            )
        return {
            "issued_asset": None,
            "funded_outputs": [],
            "accepted_txid": None,
            "confidential_funded_outputs": reported,
            "mined_readback": readback,
            "transaction_weight": executor.weight_of(transaction.serialize().hex()),
        }

    def sign_sponsor(self, subject: dict) -> dict:
        """Authorizes one input of a finalized transaction.

        # What is checked before anything is signed

        That the input at the stated index really spends the stated
        coin, and that the coin is one this adapter created. Signing an
        input the caller misidentified would produce an authorization
        over the wrong prevout -- which the target would refuse, but
        only after the run had recorded a refusal whose cause was this
        adapter's rather than the candidate's.

        # Why the nonce is forced deterministic

        The framework's ECDSA signer uses a random nonce unless asked
        otherwise, and a random nonce makes every run of the same census
        produce different bytes. Reproducibility is a property this lane
        is expected to have, so the nonce is derived from the key and
        the message instead (RFC 6979) and two runs agree to the byte.
        """
        executor = self.executor
        messages = executor.messages
        script = executor.script

        raw = subject["finalized_transaction"]
        transaction = messages.CTransaction()
        try:
            transaction.deserialize(io.BytesIO(raw))
        except Exception as error:
            raise AdapterError(
                "the bytes handed to this signing step are not a transaction "
                "this framework decodes: %s" % error
            )

        index = subject["sponsor_input_index"]
        if index >= len(transaction.vin):
            raise AdapterError(
                "the step named input %d of a transaction with %d inputs"
                % (index, len(transaction.vin))
            )
        stated = subject["sponsor_outpoint"]
        prevout = transaction.vin[index].prevout
        # The framework holds an outpoint's identity as an integer over
        # the displayed byte order, which is how `txid_to_internal_int`
        # produces one.
        if (
            prevout.hash != txid_to_internal_int(stated["txid"])
            or prevout.n != stated["vout"]
        ):
            raise AdapterError(
                "input %d does not spend the coin the step named" % index
            )

        held = self.sponsor_coins.get("%s:%d" % (stated["txid"], stated["vout"]))
        if held is None:
            raise AdapterError(
                "this adapter did not create the coin it was asked to "
                "authorize a spend of, so it holds nothing that could"
            )

        # The field the funding step retained, written through unchanged.
        # Whether it is an explicit amount or a value commitment is the
        # funding step's fact and not this one's: the framework below
        # serializes either form correctly, and a signing step that
        # rebuilt the field from a number would silently be able to
        # authorize only one of them.
        value = messages.CTxOutValue()
        value.vchCommitment = held["value_field"]

        key = self.sponsor_key()
        pubkey = key.get_pubkey().get_bytes()
        # The script the signature hash is taken over for a version-zero
        # key-hash program is the pay-to-public-key-hash script the
        # program's payload names, which is the target's own rule and
        # not this adapter's convention.
        script_code = script.CScript(
            [
                script.OP_DUP,
                script.OP_HASH160,
                script.hash160(pubkey),
                script.OP_EQUALVERIFY,
                script.OP_CHECKSIG,
            ]
        )
        digest = script.SegwitV0SignatureHash(
            script_code, transaction, index, SPONSOR_SIGHASH_ALL, value
        )
        signature = key.sign_ecdsa(digest, rfc6979=True)

        return {
            "issued_asset": None,
            "funded_outputs": [],
            "accepted_txid": None,
            # Bottom item first: the authorization, then the key it is
            # checked against, which is the stack the admitted program
            # class takes.
            #
            # Written as arrays of octets, which is how every other byte
            # field crosses this boundary -- `require_bytes` reads one
            # from the harness and the harness reads one back. Hex here
            # would be a second encoding for one kind of value, and the
            # harness would refuse the record rather than misread it.
            "sponsor_witness": [
                list(signature + bytes([SPONSOR_SIGHASH_ALL])),
                list(pubkey),
            ],
            # Echoed rather than asserted. The caller compares these with
            # what it sent, byte for byte.
            "signature_bound_to": list(raw),
        }

    # -- submission -------------------------------------------------------

    def submit(self, subject: dict) -> dict:
        """Hands the target a complete transaction and reports the layer.

        # Why the mempool is asked first and the block second

        The two questions are different rules, and which one refused is
        the answer this step exists to produce. A transaction the mempool
        allows is consensus-valid and relayable; one it refuses may still
        be consensus-valid, and the only way to tell standardness from a
        consensus rule is to ask consensus separately. So a mempool
        refusal that is not a script verdict is retried at consensus, and
        the retry is what separates `relay_policy_rejection` from
        `consensus_rejection_before_script`.

        A script verdict is neither: the script ran and failed, which is
        the same fact at either layer.

        # And a script verdict is not always a SCRIPT verdict

        The mandatory-script wrapper is also what a taproot KEY-PATH
        signature failure wears, and no script runs on that path at all.
        So the wrapper opens the question rather than settling it, and
        `observed_key_path_spend` settles it from the witness and the
        spent programs -- an observation about the bytes submitted, never
        an expectation the caller supplied.
        """
        raw = subject["transaction_bytes"].hex()
        answer = self.executor.node.call("testmempoolaccept", json.dumps([raw]))
        if not isinstance(answer, list) or len(answer) != 1:
            raise AdapterError("the node did not answer testmempoolaccept with one result")
        result = answer[0]

        if result.get("allowed") is True:
            # Confirmed rather than left in the mempool, so that an
            # acceptance is an acceptance by block validation too and the
            # coins it creates are visible to any later step.
            try:
                mined = self.executor.node.call(
                    "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
                )
            except AdapterError as error:
                return self.refused_at_consensus(error, raw=raw)
            txid = result.get("txid")
            return {
                "observed_layer": "accepted",
                "observed_detail": None,
                "accepted_txid": txid,
                "mined_readback": self.read_back_confirmed(mined, txid),
                "transaction_weight": self.executor.weight_of(raw),
            }

        reason = result.get("reject-reason")
        if not isinstance(reason, str):
            raise AdapterError("the node rejected without naming a reason")
        for prefix in (POLICY_SCRIPT_PREFIX, CONSENSUS_SCRIPT_PREFIX):
            if script_error_in(reason, prefix) is not None:
                key_path = observed_key_path_spend(self.executor.node, raw)
                return {
                    "observed_layer": (
                        "key_path_rejection" if key_path else "script_path_rejection"
                    ),
                    "observed_detail": reason,
                    "accepted_txid": None,
                    "transaction_weight": self.executor.weight_of(raw),
                }

        # Not a script verdict. Ask consensus directly: a transaction a
        # block accepts was refused by standardness, and one a block also
        # refuses was refused before any script ran.
        try:
            self.executor.node.call(
                "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
            )
        except AdapterError as error:
            return self.refused_at_consensus(error, mempool_reason=reason, raw=raw)
        return {
            "observed_layer": "relay_policy_rejection",
            "observed_detail": reason,
            "accepted_txid": None,
            "transaction_weight": self.executor.weight_of(raw),
        }

    def refused_at_consensus(self, error, mempool_reason=None, raw=None) -> dict:
        """Classifies a block-validation refusal.

        The client's stderr is read for CLASSIFICATION only. What leaves
        this method is either the mempool's own structured reason, which
        is the target speaking through an RPC answer, or nothing --
        never the child's text (G12-R04).

        # A mempool reason that is not a script error settles the layer

        Block validation runs the amount checks inside the SAME check
        queue as the script checks, so a one-unit imbalance, a malformed
        rangeproof and a broken surjection proof all reach the block
        layer wearing `mandatory-script-verify-flag-failed (unknown
        error)`. The mempool tells them apart and the block cannot: a
        conservation failure is `bad-txns-in-ne-out` there, and a real
        script failure carries the mandatory-script prefix.

        So when the mempool already named a reason and that reason is not
        a script verdict, it decides, and the block's text is not
        consulted. Letting the block override it attributes a
        conservation failure to an opening script that never ran -- the
        misattribution the conservation judgement documents as a defect
        of its own earlier revision, reached here by the other path.

        The block's text is still read when the mempool named nothing to
        contradict it, which is the transaction a relay accepted and a
        block did not.
        """
        detail = mempool_reason
        # The refused bytes still have a weight, and it is the node's
        # own figure for them rather than this adapter's arithmetic. A
        # refusal is a target verdict, so the observation belongs on it.
        weight = self.executor.weight_of(raw) if raw is not None else None
        if (
            mempool_reason is None
            and script_error_in(error.client_detail, CONSENSUS_SCRIPT_PREFIX) is not None
        ):
            # The same wrapper opens the same question here, and it is
            # settled the same way, from the bytes rather than from the
            # string. Reading it at one of the two sites only would leave
            # the misfiling alive on the other.
            key_path = raw is not None and observed_key_path_spend(self.executor.node, raw)
            return {
                "observed_layer": (
                    "key_path_rejection" if key_path else "script_path_rejection"
                ),
                "observed_detail": detail,
                "accepted_txid": None,
                "transaction_weight": weight,
            }
        return {
            "observed_layer": "consensus_rejection_before_script",
            "observed_detail": detail,
            "accepted_txid": None,
            "transaction_weight": weight,
        }


class ConservationExecutor:
    """Materializes and judges one Guide-11 section 8.4 conservation row.

    Confidential value on this target is produced by the node, not by the
    upstream Python framework: the framework offers no Pedersen
    commitment, range proof, or surjection proof, and the only interfaces
    that build one are `blindrawtransaction` and `rawblindrawtransaction`.
    Both draw every output blinding factor from `GetStrongRandBytes` and
    generate a fresh ephemeral nonce key, and neither takes a seed.

    Two consequences, both recorded rather than worked around:

      determinism   equal fixture inputs do NOT yield equal transaction
                    bytes. The fixture's own inputs are fully determined,
                    the produced bytes are reported per run, and
                    byte-level reproducibility is stated as unachievable
                    through this materializer;

      blinders      a confidential coin whose blinders are known has to
                    come from the wallet, which reports them in
                    `listunspent`. That wallet is disposable test-network
                    material under ADR-015's test-material rule: it is
                    created on a regtest chain whose assets have no value,
                    lives inside the datadir this process deletes, and is
                    never derived from or reused as production material.
                    Guide 11 section 1.7 bans a production wallet from the
                    command interface, and none appears there: this wallet
                    is created by the adapter and named by nobody.

    Nothing here changes a chain parameter. The wallet is funded from the
    same free coin the primitive lane already spends, because the genesis
    identifier is what the run is bound to and a chain parameter would
    move it.
    """

    WALLET = "guide11-conservation"

    def __init__(self, executor: "CaseExecutor", wallet_name=None) -> None:
        self.executor = executor
        self.node = executor.node
        self.messages = executor.messages
        self.ready = False
        # Shadows the class default so that two processes sharing one
        # chain directory cannot end up sharing one wallet -- which would
        # make a fresh-process boundary a fiction.
        if wallet_name is not None:
            self.WALLET = wallet_name

    # -- the disposable wallet -------------------------------------------

    # How much of the chain's free coin the wallet is given once, up front.
    # Generous on purpose: every row funds its own coins from it, and a
    # wallet that ran dry mid-matrix would turn later rows into
    # construction failures for a reason that is not the row's.
    WALLET_ENDOWMENT_SATOSHIS = 10_000_000_000

    def prepare(self) -> None:
        """Creates the disposable wallet and endows it, once per process.

        The endowment is one raw transaction spending the chain's free
        coin -- the same coin the primitive lane spends -- so no chain
        parameter, block subsidy, or connected genesis output is involved
        and the genesis identifier the run is bound to does not move.

        After this the wallet owns the money and manages its own change,
        which is what makes a confidential coin obtainable at all: the
        blinding key belongs to the wallet, so the node can report the
        blinding factors this matrix needs.
        """
        if self.ready:
            return
        try:
            self.node.call("createwallet", self.WALLET)
        except AdapterError as error:
            if "already exists" not in error.note:
                raise

        address = self.node.call("getnewaddress", wallet=self.WALLET)
        info = self.node.call("getaddressinfo", address, wallet=self.WALLET)
        program = bytes.fromhex(info["scriptPubKey"])
        self.executor.fund(program, self.WALLET_ENDOWMENT_SATOSHIS)
        self.ready = True

    def address(self, confidential: bool, address_type=None) -> str:
        """One wallet address, blinded or not.

        `address_type` is passed through to the node when given, and
        omitted otherwise so that the conservation rows keep the wallet's
        own default exactly as they had it. The normalization lane asks
        for taproot by name, because its signing profile depends on the
        address type rather than merely tolerating it.

        # Why the unconfidential form is taken explicitly

        `getnewaddress` returns a BLINDED address by default on this
        target: `-blindedaddresses` defaults to 1. Returning what it hands
        back for an explicit output would silently blind every row that
        said it was explicit, and the transactions would still be valid --
        so the matrix would report an explicit-to-explicit row that was
        confidential end to end, and nothing in the result would say so.

        Both forms are therefore taken from `getaddressinfo` by name
        rather than by default.
        """
        if address_type is None:
            fresh = self.node.call("getnewaddress", wallet=self.WALLET)
        else:
            fresh = self.node.call("getnewaddress", "", address_type, wallet=self.WALLET)
        info = self.node.call("getaddressinfo", fresh, wallet=self.WALLET)
        wanted = info.get("confidential") if confidential else info.get("unconfidential")
        if not isinstance(wanted, str) or wanted == "":
            raise AdapterError(
                "the node reported no %s form for a wallet address"
                % ("confidential" if confidential else "unconfidential")
            )
        return wanted

    # -- creating the stated input coins ---------------------------------

    def create_coin(self, amount: int, confidential: bool, address_type=None) -> dict:
        """Creates one coin the wallet owns, and returns what it knows.

        A confidential coin is one paid to a blinded address: the wallet
        blinds the transaction on the way out, and because the blinding
        key is the wallet's own, `listunspent` reports the amount and
        asset blinding factors. Those factors are what every confidential
        row needs and what no non-wallet interface offers.

        The coin is confirmed in a block of its own so no row's
        transaction is ever subject to mempool ancestor policy -- an
        ancestor-limit refusal is not a conservation verdict and would
        arrive looking like one.
        """
        destination = self.address(confidential, address_type)
        try:
            txid = self.node.call(
                "sendtoaddress", destination, satoshis_to_amount(amount),
                wallet=self.WALLET,
            )
        except AdapterError as error:
            raise ConstructionError(
                "the wallet paid no coin of %d: %s" % (amount, error.note)
            )
        self.node.call(
            "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([txid])
        )

        plain = self.plain_form(destination)
        for entry in self.node.call("listunspent", "1", "9999999", wallet=self.WALLET):
            if entry["txid"] == txid and entry.get("address") == plain:
                if confidential and entry.get("amountblinder", "00" * 32) == "00" * 32:
                    raise ConstructionError(
                        "the wallet created an unblinded coin for a confidential input"
                    )
                if not confidential and entry.get("amountblinder", "00" * 32) != "00" * 32:
                    raise ConstructionError(
                        "the wallet created a blinded coin for an explicit input"
                    )
                # Locked so the wallet's own coin selection cannot spend it
                # while funding a later coin of the same row. Without this
                # the row's transaction names an input the wallet has
                # already consumed, and the node answers
                # `bad-txns-inputs-missingorspent` -- which is this
                # adapter's bookkeeping failing, not a conservation verdict.
                self.node.call(
                    "lockunspent", "false",
                    json.dumps([{"txid": txid, "vout": entry["vout"]}]),
                    wallet=self.WALLET,
                )
                return entry
        raise AdapterError("the wallet did not report the coin this adapter just created")

    def plain_form(self, address: str) -> str:
        """The unconfidential form of an address, as the node states it."""
        info = self.node.call("getaddressinfo", address, wallet=self.WALLET)
        return info.get("unconfidential", address)

    # -- one row ----------------------------------------------------------

    def execute(self, case: dict, subject: dict) -> dict:
        """Materializes one row and reports the layer that answered it."""
        self.prepare()
        inputs = subject["inputs"]
        outputs = subject["outputs"]
        defect = subject["defect"]
        # Fail closed. A defect name this adapter does not implement used
        # to fall through every branch and produce an undamaged
        # transaction, which the target then accepted -- and the row was
        # recorded as the target accepting a broken proof. One misspelled
        # constant was all it took, so an unknown defect is now refused by
        # name rather than silently skipped.
        if defect not in KNOWN_CONSERVATION_DEFECTS:
            raise ConstructionError(
                "this adapter does not implement the defect %r" % defect
            )

        # The coins the row consumes. A confidential input is a blinded
        # coin whose blinders the wallet reports; an explicit one is not
        # blinded at all.
        coins = []
        for index, declared in enumerate(inputs):
            confidential = declared["value"]["representation"] == "confidential"
            try:
                coins.append(self.create_coin(declared["value"]["amount"], confidential))
            except ConstructionError:
                raise
            except AdapterError as error:
                raise ConstructionError(
                    "input %d could not be created: %s" % (index, error.note)
                )

        # A dedicated explicit coin pays the fee, so that the row's own
        # amounts state the conservation question and nothing else. An
        # explicit value joins the tally zero-blinded, so it perturbs no
        # blinding balance.
        fee_coin = self.create_coin(ADAPTER_FEE_SATOSHIS, False)
        coins.append(fee_coin)

        spend_inputs = [
            {"txid": coin["txid"], "vout": coin["vout"]} for coin in coins
        ]

        declared_outputs = []
        for declared in outputs:
            amount = declared["value"]["amount"]
            if defect == "one_unit_imbalance":
                amount += 1
            confidential = declared["value"]["representation"] == "confidential"
            if defect == "wrong_explicit_asset":
                # An asset the chain never issued. The transaction is
                # constructible and cannot balance, which is the row.
                declared_outputs.append({
                    self.address(False): satoshis_to_amount(amount),
                    "asset": FOREIGN_ASSET_HEX,
                })
            else:
                declared_outputs.append(
                    {self.address(confidential): satoshis_to_amount(amount)}
                )

        if defect == "hidden_confidential_output":
            # The unstated output that absorbs the value the stated set
            # does not account for. Consensus admits it; that is the row.
            hidden = sum(entry["value"]["amount"] for entry in inputs) - sum(
                entry["value"]["amount"] for entry in outputs
            )
            if hidden > 0:
                declared_outputs.append(
                    {self.address(True): satoshis_to_amount(hidden)}
                )

        declared_outputs.append({"fee": satoshis_to_amount(ADAPTER_FEE_SATOSHIS)})

        try:
            raw = self.node.call(
                "createrawtransaction",
                json.dumps(spend_inputs),
                json.dumps(declared_outputs),
            )
        except AdapterError as error:
            raise ConstructionError("the node built no raw transaction: %s" % error.note)

        # Blinding, where any coin or any stated output is confidential.
        needs_blinding = any(
            coin.get("amountblinder", "00" * 32) != "00" * 32 for coin in coins
        ) or any(entry["value"]["representation"] == "confidential" for entry in outputs)
        if needs_blinding:
            blinders = []
            for coin in coins:
                blinder = coin.get("amountblinder", "00" * 32)
                if defect == "wrong_blinder_sum" and blinder != "00" * 32:
                    # Declare a blinder the chain does not agree with. The
                    # amounts stay right and the blinding balance does not
                    # close, which is exactly the row.
                    blinder = "11" * 32
                blinders.append(blinder)
            try:
                raw = self.node.call(
                    "rawblindrawtransaction",
                    raw,
                    json.dumps(blinders),
                    json.dumps([satoshis_to_amount_float(coin["amount"]) for coin in coins]),
                    json.dumps([coin["asset"] for coin in coins]),
                    json.dumps([coin.get("assetblinder", "00" * 32) for coin in coins]),
                )
            except AdapterError as error:
                raise ConstructionError(
                    "the node blinded no transaction for this row: %s" % error.note
                )

        try:
            signed = self.node.call(
                "signrawtransactionwithwallet", raw, wallet=self.WALLET
            )
        except AdapterError as error:
            raise ConstructionError("the wallet signed no transaction: %s" % error.note)
        raw = signed.get("hex")
        if not isinstance(raw, str):
            raise ConstructionError("the wallet returned no signed transaction")
        if signed.get("complete") is not True:
            # An incompletely signed transaction fails script verification
            # for a reason that has nothing to do with the row. Reporting
            # it as a target verdict would file this adapter's inability
            # to sign as evidence about conservation.
            raise ConstructionError(
                "the wallet signed the row's transaction only partially: %s"
                % one_line(json.dumps(signed.get("errors", [])))[:200]
            )

        if defect in ("malformed_range_proof", "malformed_surjection_proof"):
            raw = self.corrupt_proof(raw, defect)
        if defect == "copied_commitment_from_other_asset":
            raw = self.copy_commitment(raw)

        layer, detail = self.judge(raw)
        value_commitments, asset_commitments = self.read_commitments(raw)
        return {
            "observed_layer": layer,
            "observed_detail": detail,
            "transaction_bytes": list(bytes.fromhex(raw)),
            "observed_value_commitments": value_commitments,
            "observed_asset_commitments": asset_commitments,
            "observed_openings": self.read_openings(raw) if layer == "accepted" else [],
        }

    def read_openings(self, raw: str):
        """The openings the node reports for the outputs it just created.

        # Why the target has to supply these

        The oracle predicts a commitment from an amount and two blinding
        factors. This materializer does not choose those factors -- the
        node draws them -- so without reading them back there is nothing
        for the oracle to predict, and the three-way comparison of section
        7.4 has no second point to meet at.

        The transaction is confirmed and the created coins are looked up,
        which is the only interface that reports them. What comes back is
        the target's own statement of what it committed to, and the
        comparison then runs one way: the oracle predicts bytes from these
        openings and the prediction is checked against the commitment the
        transaction actually carries. No expected value is rewritten to
        match an observation.
        """
        try:
            self.node.call(
                "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
            )
        except AdapterError as error:
            log("the accepted transaction was not confirmable: %s" % error.note)
            return []
        decoded = self.node.call("decoderawtransaction", raw)
        txid = decoded["txid"]
        openings = []
        for entry in self.node.call("listunspent", "0", "9999999", wallet=self.WALLET):
            if entry["txid"] != txid:
                continue
            blinder = entry.get("amountblinder", "00" * 32)
            if blinder == "00" * 32:
                continue
            openings.append({
                "vout": entry["vout"],
                "amount_satoshis": int(round(float(entry["amount"]) * 100_000_000)),
                "asset": entry["asset"],
                "amount_blinder": blinder,
                "asset_blinder": entry.get("assetblinder", "00" * 32),
            })
        return openings

    # -- deliberate defects -----------------------------------------------

    def corrupt_proof(self, raw: str, defect: str) -> str:
        """Flips one bit of the first proof of the named kind.

        The result is checked to actually differ from what went in. A
        mutation that a serialization round trip quietly discards would
        leave a perfectly valid transaction wearing a malformed row's
        name, and the target accepting it would be recorded as the target
        accepting a broken proof -- a false and alarming claim produced
        entirely by this adapter.
        """
        transaction = self.deserialize(raw)
        # The round trip itself must be faithful before any mutation is
        # read into it.
        if transaction.serialize().hex() != raw:
            raise ConstructionError(
                "this adapter's transaction round trip is not byte-faithful, so a "
                "corrupted proof cannot be attributed to the target"
            )
        # Every proof of the kind, not just the first. A transaction may
        # carry several, and corrupting one of them leaves the rest
        # verifying -- which is a weaker case than the row states and, if
        # the target accepts it, an acceptance the row cannot explain.
        corrupted_indices = []
        for index, witness in enumerate(transaction.wit.vtxoutwit):
            field = (
                witness.vchRangeproof
                if defect == "malformed_range_proof"
                else witness.vchSurjectionproof
            )
            if len(field) == 0:
                continue
            mutated = bytearray(bytes(field))
            mutated[len(mutated) // 2] ^= 0x01
            if defect == "malformed_range_proof":
                witness.vchRangeproof = bytes(mutated)
            else:
                witness.vchSurjectionproof = bytes(mutated)
            corrupted_indices.append(index)
        if not corrupted_indices:
            raise ConstructionError(
                "the materialized transaction carries no %s to corrupt" % defect
            )
        corrupted = transaction.serialize().hex()
        if corrupted == raw:
            raise ConstructionError(
                "the %s mutation did not survive serialization" % defect
            )
        log("corrupted %s on outputs %s" % (defect, corrupted_indices))
        return corrupted

    def copy_commitment(self, raw: str) -> str:
        """Replaces one output's value commitment with another output's.

        The donor is an output of a different asset, created here for the
        purpose. Copying a commitment leaves the amounts stated correctly
        and the tally unsatisfiable, which is the row.
        """
        transaction = self.deserialize(raw)
        donor = self.create_coin(ADAPTER_FEE_SATOSHIS, True)
        # `gettransaction` rather than `getrawtransaction`: the donor is a
        # wallet transaction already confirmed in a block, and the raw
        # interface needs a block hash or a transaction index the
        # disposable node does not build.
        donor_raw = self.node.call(
            "gettransaction", donor["txid"], wallet=self.WALLET
        )["hex"]
        donor_transaction = self.deserialize(donor_raw)
        replacement = None
        for out in donor_transaction.vout:
            commitment = bytes(out.nValue.vchCommitment)
            if len(commitment) == 33:
                replacement = commitment
                break
        if replacement is None:
            raise ConstructionError("no donor output carried a value commitment")
        for out in transaction.vout:
            if len(bytes(out.nValue.vchCommitment)) == 33:
                out.nValue.vchCommitment = replacement
                return transaction.serialize().hex()
        raise ConstructionError("this row's transaction carries no commitment to replace")

    def deserialize(self, raw: str):
        """One transaction, from its hex."""
        transaction = self.messages.CTransaction()
        transaction.deserialize(io.BytesIO(bytes.fromhex(raw)))
        return transaction

    # -- the layer that answered ------------------------------------------

    def judge(self, raw: str):
        """Reports WHICH layer refused, from what the node actually did.

        Guide 11 section 8.3's whole point. `testmempoolaccept` changes no
        state, so it is asked first, and it is also the only interface
        that names a CT failure precisely.

        # Why the block's answer must not be read for the layer

        Block validation runs the amount checks inside the SAME check
        queue as the script checks, and a failure of either surfaces as
        `mandatory-script-verify-flag-failed (unknown error)`. So a
        malformed rangeproof, a broken surjection proof, and a one-unit
        imbalance all arrive at the block layer wearing a script error's
        clothes. Classifying on that string attributes a conservation
        failure to an opening script that never ran -- which is exactly
        the misattribution section 8.3 exists to prevent, and which an
        earlier revision of this method committed.

        The mempool distinguishes them: a conservation failure is
        `bad-txns-in-ne-out`, and a genuine script failure carries the
        mandatory-script prefix. So the mempool reason decides the layer,
        and the block is asked only to separate a merely unrelayable
        transaction from one consensus will not have.

          relay accepts                      -> accepted
          relay names a mandatory script err  -> script-path rejection,
                                                 or key-path rejection
                                                 where the bytes are
                                                 observed to have run no
                                                 script at all
          relay names any other consensus
            reason, and a block also refuses  -> consensus rejection
                                                 before script
          relay refuses, a block takes it     -> relay-policy rejection

        The key-path arm is unreachable from a conservation row -- every
        candidate this lane materializes reveals a leaf and a control
        block -- and it is asked here anyway, because one rule spelled
        twice is two rules that can disagree, which is the lesson
        `script_error_in` was extracted for.

        Nothing here consults an expectation, because none was sent.
        """
        answer = self.node.call("testmempoolaccept", json.dumps([raw]))
        if not isinstance(answer, list) or len(answer) != 1:
            raise AdapterError("the node did not answer testmempoolaccept with one result")
        result = answer[0]
        if result.get("allowed") is True:
            return "accepted", None
        relay_reason = result.get("reject-reason")
        if not isinstance(relay_reason, str):
            raise AdapterError("the node rejected without naming a reason")

        if relay_reason.startswith(CONSENSUS_SCRIPT_PREFIX):
            if observed_key_path_spend(self.node, raw):
                return "key_path_rejection", relay_reason
            return "script_path_rejection", relay_reason

        # Whether the refusal is consensus or merely standardness is
        # settled by asking a block to take it, not by reading the string.
        try:
            self.node.call(
                "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
            )
        except AdapterError as error:
            return "consensus_rejection_before_script", relay_reason
        return "relay_policy_rejection", relay_reason

    def read_commitments(self, raw: str):
        """The output commitments the node reads back out of the bytes.

        The third leg of the section 7.4 comparison. Read from the node's
        own decoding rather than recomputed here, because a leg the
        harness computed would be comparing the oracle with itself.
        """
        values = []
        assets = []
        try:
            decoded = self.node.call("decoderawtransaction", raw)
        except AdapterError:
            return values, assets
        for out in decoded.get("vout", []):
            commitment = out.get("valuecommitment")
            if isinstance(commitment, str):
                values.append(list(bytes.fromhex(commitment)))
            commitment = out.get("assetcommitment")
            if isinstance(commitment, str):
                assets.append(list(bytes.fromhex(commitment)))
        return values, assets


class NormalizationExecutor:
    """Builds and judges one Guide-11 section 10.4 normalization row.

    The claim is owner-authorized normalization: a private coin the owner
    holds is consumed, half its value is republished in the clear, and a
    blinded change output absorbs the rest along with the residual
    blinding. Wave 7 established that the fully explicit variant is not
    constructible at all (`G11-W7-03`), so this is the shape that exists.

    # What this adapter reports, and what it must not decide

    Three of the nine rows are transactions the target ACCEPTS and the
    claim must still refuse. Their refusal is a disagreement between what
    the claim named and what the transaction carries, and neither side may
    be manufactured here:

      claimed    resolved from the claim BEFORE any mutation is applied,
                 so a mutation cannot quietly move the claim to wherever
                 the transaction ended up;

      observed   read back out of the node's own decoder, so the harness
                 is never comparing its intent with itself.

    The comparison itself is not made here. This adapter reports both
    sides and the layer the node answered at; which layer refused is
    decided by the typed report, from data it did not generate.

    # The signing profile is observed rather than asserted

    Section 10.3 requires authorization committing to the finalized output
    set. This adapter uses taproot addresses so the wallet produces a
    key-path signature, and reports the witness item sizes it actually
    found. A bare 64-byte item is a Schnorr signature with no trailing
    sighash byte, which the reviewed digest reads as the default
    all-outputs non-anyone-can-pay mode. A reader checks the profile
    against those sizes rather than against this docstring.
    """

    def __init__(self, conservation: "ConservationExecutor") -> None:
        self.conservation = conservation
        self.node = conservation.node
        self.messages = conservation.messages

    # -- resolving the claim ----------------------------------------------

    def script_of(self, address: str) -> bytes:
        """The scriptPubKey an address pays, as the node states it."""
        info = self.node.call(
            "getaddressinfo", address, wallet=self.conservation.WALLET
        )
        program = info.get("scriptPubKey")
        if not isinstance(program, str):
            raise AdapterError("the node reported no scriptPubKey for an address")
        return bytes.fromhex(program)

    def execute(self, case: dict, subject: dict) -> dict:
        """Builds one normalization row and reports what happened to it."""
        mutation = subject["mutation"]
        if mutation not in KNOWN_NORMALIZATION_MUTATIONS:
            raise ConstructionError(
                "this adapter does not implement the mutation %r" % mutation
            )
        claim = subject["claim"]
        consumed = claim["consumed"]["value"]["amount"]
        normalized = claim["normalized"]["value"]["amount"]
        change = claim["change"]["value"]["amount"]
        if consumed != normalized + change:
            raise ConstructionError(
                "the claim does not conserve the value it consumes: %d into %d and %d"
                % (consumed, normalized, change)
            )

        self.conservation.prepare()
        kind = NORMALIZATION_ADDRESS_TYPE

        # The owner's private coin, and a separate explicit coin for the
        # fee so the claim's own amounts state the question and nothing
        # else. An explicit value joins the tally zero-blinded.
        owner_coin = self.conservation.create_coin(consumed, True, kind)
        if owner_coin.get("amountblinder", "00" * 32) == "00" * 32:
            raise ConstructionError(
                "the owner's coin is not blinded, so there is no private value "
                "to normalize"
            )
        fee_coin = self.conservation.create_coin(NORMALIZATION_FEE_SATOSHIS, False, kind)
        coins = [owner_coin, fee_coin]

        # The claim's own destinations, resolved once and before any
        # mutation touches anything.
        normalized_address = self.conservation.address(False, kind)
        change_address = self.conservation.address(True, kind)
        normalized_script = self.script_of(self.conservation.plain_form(normalized_address))
        change_script = self.script_of(self.conservation.plain_form(change_address))
        policy_asset = owner_coin["asset"]
        claimed_outputs = [
            {
                "role": "normalized",
                "script_pubkey": list(normalized_script),
                "explicit_amount": normalized,
                "explicit_asset": list(bytes.fromhex(policy_asset)),
            },
            {
                "role": "private_change",
                "script_pubkey": list(change_script),
                # A blinded output publishes neither, and the claim does
                # not pretend otherwise: what it asserts about this output
                # is that it exists and is the owner's.
                "explicit_amount": None,
                "explicit_asset": None,
            },
        ]

        raw = self.build(
            claim, mutation, coins, normalized_address, change_address, policy_asset
        )
        raw = self.blind(raw, coins, mutation)
        raw = self.sign(raw)
        profile, witness_sizes = self.read_authorization(raw)
        raw = self.mutate_after_signing(raw, mutation)

        layer, detail = self.conservation.judge(raw)
        return {
            "observed_layer": layer,
            "observed_detail": detail,
            # The address the claim's normalized output pays, in its
            # canonical public encoding. Reported because the section 13
            # lane publishes it and for no other reason: it is the one
            # public projection of the owner, and a party with no
            # relationship to this process has to be able to rebuild the
            # output script from it rather than be handed the bytes.
            "normalized_address": normalized_address,
            "claimed_outputs": claimed_outputs,
            "observed_outputs": self.read_outputs(raw),
            "authorization_profile": profile,
            "observed_witness_sizes": witness_sizes,
            "transaction_bytes": list(bytes.fromhex(raw)),
        }

    # -- building ----------------------------------------------------------

    def build(self, claim, mutation, coins, normalized_address, change_address,
              policy_asset) -> str:
        """The unsigned transaction, with any pre-signing mutation applied."""
        normalized = claim["normalized"]["value"]["amount"]
        change = claim["change"]["value"]["amount"]

        if mutation == "amount_changed":
            # Compensated on purpose: the change absorbs exactly what the
            # normalized output gains, so the transaction still conserves
            # value and consensus has nothing to refuse. The uncompensated
            # form is already a consensus rejection (conservation row 6).
            normalized += NORMALIZATION_AMOUNT_DELTA
            change -= NORMALIZATION_AMOUNT_DELTA

        destination = normalized_address
        if mutation == "owner_changed":
            # A script the claim does not name. Every amount stays right,
            # so the transaction is consensus-valid and pays the wrong
            # party -- which no target check has any opinion about.
            destination = self.conservation.address(False, NORMALIZATION_ADDRESS_TYPE)

        first = {destination: satoshis_to_amount(normalized)}
        if mutation == "asset_changed":
            # An asset the chain never issued. Unlike the owner and the
            # amount, this one cannot be changed while conserving value.
            first["asset"] = FOREIGN_ASSET_HEX

        declared = [first]

        if mutation == "hidden_private_output":
            # The value comes out of the private change, which is blinded,
            # so nothing observable shrinks. Only the presence of an
            # output the claim does not name gives it away.
            change -= NORMALIZATION_HIDDEN_AMOUNT
            declared.append({change_address: satoshis_to_amount(change)})
            declared.append({
                self.conservation.address(True, NORMALIZATION_ADDRESS_TYPE):
                    satoshis_to_amount(NORMALIZATION_HIDDEN_AMOUNT)
            })
        else:
            declared.append({change_address: satoshis_to_amount(change)})

        declared.append({"fee": satoshis_to_amount(NORMALIZATION_FEE_SATOSHIS)})

        spend_inputs = [{"txid": c["txid"], "vout": c["vout"]} for c in coins]
        try:
            return self.node.call(
                "createrawtransaction",
                json.dumps(spend_inputs),
                json.dumps(declared),
            )
        except AdapterError as error:
            raise ConstructionError(
                "the node built no raw transaction for this row: %s" % error.note
            )

    def blind(self, raw: str, coins, mutation: str) -> str:
        """Blinds the transaction, honouring the wrong-balance mutation."""
        blinders = []
        for coin in coins:
            blinder = coin.get("amountblinder", "00" * 32)
            if mutation == "wrong_blinding_balance" and blinder != "00" * 32:
                # A blinder the chain does not agree with. The amounts stay
                # right and the blinding balance does not close.
                blinder = "11" * 32
            blinders.append(blinder)
        try:
            return self.node.call(
                "rawblindrawtransaction",
                raw,
                json.dumps(blinders),
                json.dumps([satoshis_to_amount_float(c["amount"]) for c in coins]),
                json.dumps([c["asset"] for c in coins]),
                json.dumps([c.get("assetblinder", "00" * 32) for c in coins]),
            )
        except AdapterError as error:
            raise ConstructionError(
                "the node blinded no transaction for this row: %s" % error.note
            )

    def sign(self, raw: str) -> str:
        """The owner's authorization over the finalized output set."""
        try:
            signed = self.node.call(
                "signrawtransactionwithwallet", raw, wallet=self.conservation.WALLET
            )
        except AdapterError as error:
            raise ConstructionError("the wallet signed no transaction: %s" % error.note)
        if signed.get("complete") is not True:
            # A partially signed transaction fails verification for a
            # reason that has nothing to do with the row, and reporting it
            # would file this adapter's inability to sign as evidence
            # about authorization.
            raise ConstructionError(
                "the wallet signed the row's transaction only partially: %s"
                % one_line(json.dumps(signed.get("errors", [])))[:200]
            )
        raw = signed.get("hex")
        if not isinstance(raw, str):
            raise ConstructionError("the wallet returned no signed transaction")
        return raw

    def read_authorization(self, raw: str):
        """The signing profile, read out of the witness rather than assumed.

        A taproot key-path spend carries exactly one witness item. Sixty-
        four bytes means no trailing sighash byte, and the reviewed digest
        reads its absence as the default all-outputs non-anyone-can-pay
        mode; sixty-five means the mode is the trailing byte's, and only
        an explicit all-outputs byte without anyone-can-pay qualifies.

        Anything else is refused rather than described. A row whose
        signature did not commit to the output set would answer the three
        post-signing mutations with a refusal that establishes nothing,
        and reporting the profile as unknown while still running them
        would produce exactly that.
        """
        transaction = self.conservation.deserialize(raw)
        sizes = []
        profiles = set()
        for witness in transaction.wit.vtxinwit:
            stack = [bytes(item) for item in witness.scriptWitness.stack]
            sizes.append([len(item) for item in stack])
            if len(stack) != 1:
                raise ConstructionError(
                    "an input's witness carries %d items, so this spend is not the "
                    "key-path taproot authorization section 10.3 requires"
                    % len(stack)
                )
            signature = stack[0]
            if len(signature) == 64:
                profiles.add("sighash_default")
            elif len(signature) == 65 and signature[64] == 0x01:
                profiles.add("sighash_all_no_anyone_can_pay")
            else:
                raise ConstructionError(
                    "an input is signed with a %d-byte witness item whose profile is "
                    "not one section 10.3 admits" % len(signature)
                )
        if len(profiles) != 1:
            raise ConstructionError(
                "the inputs are signed under %d different profiles, so the "
                "authorization has no single profile to report" % len(profiles)
            )
        return profiles.pop(), sizes

    # -- mutations applied after the owner signed --------------------------

    def mutate_after_signing(self, raw: str, mutation: str) -> str:
        """Applies a post-signing mutation, or returns the bytes unchanged."""
        if mutation not in (
            "extra_output_after_signing",
            "output_mutated_after_signing",
            "unauthorized_representation_change",
        ):
            return raw

        transaction = self.conservation.deserialize(raw)
        # The round trip must be faithful before any mutation is read into
        # it, on `G11-W7-07`'s reasoning: a mutation a serialization
        # quietly discards would leave a valid transaction wearing a
        # mutated row's name.
        if transaction.serialize().hex() != raw:
            raise ConstructionError(
                "this adapter's transaction round trip is not byte-faithful, so a "
                "post-signing mutation cannot be attributed to the target"
            )

        if mutation == "extra_output_after_signing":
            self.append_output(transaction)
        elif mutation == "output_mutated_after_signing":
            self.repoint_output(transaction)
        else:
            self.strip_owner_witness(transaction)

        mutated = transaction.serialize().hex()
        if mutated == raw:
            raise ConstructionError(
                "the %s mutation did not survive serialization" % mutation
            )
        return mutated

    def append_output(self, transaction) -> None:
        """Appends an output, funded from the declared fee.

        Funded from the fee so that value still conserves: a row that also
        unbalanced the transaction would be refused by the tally, and the
        signature -- the thing the row exists to test -- would never be
        reached. The remaining fee stays well above any relay threshold
        for the same reason.
        """
        messages = self.messages
        fee_index = None
        for index, out in enumerate(transaction.vout):
            if len(bytes(out.scriptPubKey)) == 0:
                fee_index = index
                break
        if fee_index is None:
            raise ConstructionError("this row's transaction declares no fee output")
        fee_out = transaction.vout[fee_index]
        fee_amount = explicit_amount(bytes(fee_out.nValue.vchCommitment), "the fee output")
        remaining = fee_amount - NORMALIZATION_EXTRA_OUTPUT_SATOSHIS
        if remaining <= 0:
            raise ConstructionError(
                "the declared fee cannot fund an appended output"
            )
        asset_field = bytes(fee_out.nAsset.vchCommitment)
        fee_out.nValue = messages.CTxOutValue(remaining)
        appended = messages.CTxOut(
            nValue=messages.CTxOutValue(NORMALIZATION_EXTRA_OUTPUT_SATOSHIS),
            scriptPubKey=bytes.fromhex(ANYONE_CAN_SPEND_HEX),
            nAsset=messages.CTxOutAsset(asset_field),
        )
        # Before the fee, because Elements expects the fee output last.
        transaction.vout.insert(fee_index, appended)

    def repoint_output(self, transaction) -> None:
        """Pays the normalized output to a different script, amount intact.

        No amount is touched, so conservation still closes exactly and the
        only thing wrong with the transaction is that the owner did not
        sign for this recipient.
        """
        for out in transaction.vout:
            program = bytes(out.scriptPubKey)
            if len(program) == 0:
                continue
            commitment = bytes(out.nValue.vchCommitment)
            if len(commitment) == 33:
                # Blinded: this is the change, not the normalized output.
                continue
            out.scriptPubKey = bytes.fromhex(ANYONE_CAN_SPEND_HEX)
            return
        raise ConstructionError(
            "this row's transaction carries no explicit output to repoint"
        )

    def strip_owner_witness(self, transaction) -> None:
        """Removes the owner's authorization entirely."""
        if not transaction.wit.vtxinwit:
            raise ConstructionError("this row's transaction carries no input witness")
        transaction.wit.vtxinwit[0].scriptWitness.stack = []

    # -- what the target says it carries -----------------------------------

    def read_outputs(self, raw: str):
        """The outputs the node's own decoder reports.

        Read from the target rather than remembered, because the checks
        these feed exist to catch a transaction that is not what the claim
        says -- and an observation this adapter generated from its own
        intent could never disagree with that intent.
        """
        try:
            decoded = self.node.call("decoderawtransaction", raw)
        except AdapterError as error:
            raise AdapterError(
                "the node did not decode the transaction this row built: %s"
                % error.note
            )
        outputs = []
        for out in decoded.get("vout", []):
            program = out.get("scriptPubKey", {}).get("hex", "")
            amount = out.get("value")
            asset = out.get("asset")
            outputs.append({
                "script_pubkey": list(bytes.fromhex(program)),
                "explicit_amount": (
                    None if amount is None
                    else int(round(float(amount) * 100_000_000))
                ),
                "explicit_asset": (
                    None if not isinstance(asset, str) else list(bytes.fromhex(asset))
                ),
                "is_fee": out.get("scriptPubKey", {}).get("type") == "fee"
                or program == "",
            })
        return outputs


class LifecycleExecutor:
    """Guide-11 section 13's fresh-process lifecycle, on the accepted path.

    # What is being proved, and what the accepted path changes about it

    Section 13 asks whether "public" means *recoverable from canonical
    public chain data by a party that did not participate in creation*,
    rather than *still present in the creator's process*. It was written
    with a capsule in mind, because the representation it anticipated was
    PublicCommitted: a committed amount is unreadable without an opening,
    so something has to carry the opening and be bound to the output.

    This wave runs the path that exists. On the normalization path the
    public output is EXPLICIT, so its amount and asset are on the output
    itself and there is no capsule, no opening to carry, and no binding
    to check. `G11-W8-03` already recorded that as the capsule's state --
    not-applicable rather than deferred -- and this lane inherits it. The
    reconstruction is therefore: locate the outpoint from chain data,
    parse the explicit output canonically, and check the amount, the
    asset, and the owner projection against what the public record
    claimed.

    # Why Process B does not spend the object, and why that is not a
    # weakened claim

    Section 13.4's step 6 has Process B construct a future spend. That
    step assumed a permissionless object -- something any party may take
    -- which is what a formula-bound payout would be. A normalized output
    is OWNED, and the owner's key is precisely the creator-local state
    the destruction boundary destroys. A Process B that could spend it
    would be evidence of a leak, not of publicness.

    So the future-use proof splits in two, and both halves run:

      readable    B rebuilds the expected output script from the public
                  address alone, by pure-Python bech32m decoding, and
                  byte-compares it against what the chain carries. That
                  is what a future constructor would have to do to cite
                  this output, and it is done from public data only;

      usable      B constructs and confirms a real spend of ITS OWN
                  sponsor funds, paying an address derived fresh inside
                  B. This shows B is a working constructor on this chain
                  rather than a reader that could not have built
                  anything.

    The permissionless-future-use case -- B spending the object itself --
    belongs to the compact ASH work and is out of scope here rather than
    quietly claimed.

    # The chain persists and the creator does not

    The node is not creator state. The chain data IS the canonical public
    record, so destroying it between A and B would destroy the evidence
    the test exists to find. What the boundary destroys is the creator's
    wallet -- the keys, the blinding factors, and the descriptors -- and
    the creator's process. That scoping is stated because getting it
    backwards yields a test that passes for the wrong reason in one
    direction and fails for the wrong reason in the other.
    """

    def __init__(self, normalization: "NormalizationExecutor", network_id: str) -> None:
        self.normalization = normalization
        self.conservation = normalization.conservation
        self.executor = normalization.conservation.executor
        self.node = normalization.node
        self.messages = normalization.messages
        self.network_id = network_id.lower()
        # Imported here rather than at module scope: the framework is
        # loaded from an explicit path at startup, and this is the module
        # that decodes an address without asking the node to.
        from test_framework import address as framework_address

        self.address_module = framework_address

    # -- Process A ---------------------------------------------------------

    def construct(self, subject: dict) -> dict:
        """Builds the accepted normalization and confirms it in a block.

        The normalization itself is the section 10.4 row with no
        mutation, run through the same executor the matrix runs, so this
        lane cannot be passing a transaction the matrix would not
        recognize. What is added here is confirmation: the matrix judges
        at the mempool and never mines, and a section 13 handoff has to
        name a block.
        """
        claim = subject["claim"]
        body = self.normalization.execute(
            {"normalization": "none"}, {"claim": claim, "mutation": "none"}
        )
        if body["observed_layer"] != "accepted":
            raise ConstructionError(
                "the unmutated normalization was not accepted, so there is no "
                "public record to publish: %s (%s)"
                % (body["observed_layer"], body["observed_detail"])
            )
        raw = bytes(body["transaction_bytes"]).hex()

        index = self.locate_normalized_output(body)
        block_hash = self.node.call(
            "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
        )["hash"]
        block = self.node.call("getblock", block_hash)
        transaction = self.conservation.deserialize(raw)
        txid = transaction.rehash()
        if txid not in block.get("tx", []):
            raise AdapterError(
                "the block this adapter mined does not carry the transaction it "
                "was given"
            )

        # Locked against this wallet's own coin selection. The published
        # object is ordinary wallet money, and a later construction
        # funding itself would spend it -- which showed up as the
        # published record verifying as SPENT, a false fact of exactly
        # the kind `G11-W7-07`'s second fault records. Locking is the
        # discipline `create_coin` already keeps, for the same reason.
        self.node.call(
            "lockunspent", "false",
            json.dumps([{"txid": txid, "vout": index}]),
            wallet=self.conservation.WALLET,
        )

        claimed = body["claimed_outputs"][index]
        superseded = None
        supersede_failure = None
        if subject.get("supersede"):
            # A failure here does not fail the step. What the caller
            # asked for is a public record, and it has one either way;
            # what it does NOT then have is a record whose object was
            # later spent. Reported as an unbuilt row with the target's
            # own words, on `G11-W7-03`'s precedent -- a row that reached
            # no verdict establishes nothing and is a finding about the
            # candidate rather than a defect of the row.
            try:
                superseded = self.supersede(txid, index, claimed["explicit_amount"])
            except (ConstructionError, AdapterError) as error:
                supersede_failure = error.note
        return {
            "superseded_by": superseded,
            "supersede_failure": supersede_failure,
            "handoff": {
                "schema": FRESH_PROCESS_HANDOFF_SCHEMA,
                "chain_name": self.node.chain,
                "network_id": self.network_id,
                "genesis_id": self.node.call("getblockhash", "0"),
                "txid": txid,
                "output_index": index,
                "block_hash": block_hash,
                "block_height": block["height"],
                "raw_transaction": raw,
                "claimed_explicit_amount": claimed["explicit_amount"],
                "claimed_explicit_asset": bytes(claimed["explicit_asset"]).hex(),
                "claimed_owner_address": body["normalized_address"],
            },
            "authorization_profile": body["authorization_profile"],
            "observed_witness_sizes": body["observed_witness_sizes"],
            "observed_outputs": body["observed_outputs"],
        }

    def supersede(self, txid: str, index: int, amount: int) -> str:
        """Spends the object just published, while its owner still exists.

        This is the explicit-path form of section 13.5's stale capsule: a
        public record that was true when it was written and names an
        output a later transaction has since consumed. It has to be built
        HERE, inside Process A, because spending an owned output needs
        the owner's key -- which is precisely what the boundary destroys.
        A stale record manufactured after the boundary would be a
        different row wearing this one's name.
        """
        # TWO confidential outputs, and the count is the whole point.
        #
        # This target's taproot digest commits the transaction's OUTPUT
        # WITNESS vector, and it hashes that vector at whatever length it
        # happens to have rather than at one entry per output. A
        # transaction that carries no witness at all therefore hashes its
        # output witnesses as the empty string while signing, and the
        # same bytes on the wire hash one empty witness per output --
        # because serialization grows the vector to match the outputs on
        # the way out and deserialization grows it again on the way in.
        # The wallet signs the first digest, consensus checks the second,
        # and the spend is refused as an invalid Schnorr signature after
        # the wallet has already reported the signing complete.
        #
        # So this spend has to reach the signer already carrying output
        # witnesses, which means it has to be genuinely blinded. One
        # confidential output is not enough to do that: with an explicit
        # input there is no input blinding factor to balance against, so
        # a lone blinded output would need a zero blinder, and the node
        # declines to blind at all. It declines QUIETLY -- the blinding
        # RPC's ignore-failure behaviour is on by default -- and hands
        # back a transaction that is still fully explicit, which is how
        # an earlier revision of this method paid to a confidential
        # address and still produced an unsignable spend.
        #
        # Two confidential outputs give the blinder sum something to
        # close against. Neither output is ever read again; what the row
        # needs from them is only that the transaction the wallet signs
        # is the transaction consensus verifies.
        first = self.conservation.address(True, NORMALIZATION_ADDRESS_TYPE)
        second = self.conservation.address(True, NORMALIZATION_ADDRESS_TYPE)
        fee = NORMALIZATION_FEE_SATOSHIS
        # Built by the node rather than by hand. Everything else in this
        # adapter constructs its own bytes on purpose -- a fixture states
        # what it means and the adapter must not improve on it -- but
        # this transaction is not a fixture. It exists only to make an
        # earlier output spent, and building it here would put this
        # adapter's serialization between the wallet and the sighash it
        # signs for no benefit the row cares about.
        unsigned = self.node.call(
            "createrawtransaction",
            json.dumps([{"txid": txid, "vout": index}]),
            # Written as JSON text rather than encoded from Python
            # numbers. An amount here is a decimal quantity of whole
            # units, and routing it through a float to get there is how a
            # transaction ends up off by a satoshi that nothing in the
            # code says it should be off by.
            '[{"%s":%s},{"%s":%s},{"fee":%s}]'
            % (
                first,
                satoshis_to_amount(amount - fee - (amount - fee) // 2),
                second,
                satoshis_to_amount((amount - fee) // 2),
                satoshis_to_amount(fee),
            ),
        )
        # Unlocked again, because this row exists to spend it. The lock
        # is against accidental consumption by coin selection, not
        # against the owner deciding to spend the object on purpose.
        self.node.call(
            "lockunspent", "true",
            json.dumps([{"txid": txid, "vout": index}]),
            wallet=self.conservation.WALLET,
        )
        spent = self.node.call("gettxout", txid, str(index))
        if spent is None:
            raise ConstructionError(
                "the object this row means to supersede is not in the "
                "unspent-output set"
            )
        # Blinding is asked for LOUDLY. The default is to hand back an
        # unblinded transaction when the blinding cannot be balanced,
        # which is silent, and a silent one is what produced a complete
        # signature no block would take. Refusing here means a future
        # change to this transaction's shape fails as a blinding error
        # naming its own cause rather than as an invalid signature three
        # steps later.
        blinded = self.node.call(
            "blindrawtransaction", unsigned, "false",
            wallet=self.conservation.WALLET,
        )
        # And checked, rather than trusted. The RPC could report success
        # while leaving the outputs explicit, and the whole reason this
        # spend is blinded at all is that the digest the wallet signs
        # depends on the output witnesses being there.
        decoded = self.node.call("decoderawtransaction", blinded)
        if not any(output.get("valuecommitment") for output in decoded["vout"]):
            raise ConstructionError(
                "the node reported a blinded transaction whose outputs are all "
                "explicit, so the spend would reach the signer with no output "
                "witnesses and produce a signature consensus refuses"
            )
        signed = self.node.call(
            "signrawtransactionwithwallet",
            blinded,
            wallet=self.conservation.WALLET,
        )
        if not signed.get("complete"):
            raise ConstructionError(
                "the owner's wallet did not finish the spend that supersedes "
                "the published object: %s" % json.dumps(signed.get("errors"))
            )
        relay = self.node.call("testmempoolaccept", json.dumps([signed["hex"]]))
        if relay[0].get("allowed") is not True:
            raise ConstructionError(
                "the owner's wallet produced a complete signature the target "
                "refuses: %s" % relay[0].get("reject-reason")
            )
        self.node.call(
            "generateblock",
            "raw(%s)" % ANYONE_CAN_SPEND_HEX,
            json.dumps([signed["hex"]]),
        )
        successor = self.conservation.deserialize(signed["hex"]).rehash()
        if self.node.call("gettxout", txid, str(index)) is not None:
            raise ConstructionError(
                "the published object is still unspent after the transaction "
                "meant to supersede it"
            )
        return successor

    def locate_normalized_output(self, body: dict) -> int:
        """Which output of the built transaction is the normalized one.

        Found by matching the claim's own script and amount against what
        the target's decoder reports, rather than by trusting the order
        the outputs were declared in: blinding is free to reorder, and a
        handoff naming the wrong index would be this adapter publishing a
        false record rather than the target doing anything.
        """
        wanted = None
        for entry in body["claimed_outputs"]:
            if entry["role"] == "normalized":
                wanted = entry
                break
        if wanted is None:
            raise AdapterError("the claim states no normalized output")
        matches = [
            position
            for position, observed in enumerate(body["observed_outputs"])
            if observed["script_pubkey"] == wanted["script_pubkey"]
            and observed["explicit_amount"] == wanted["explicit_amount"]
            and observed["explicit_asset"] == wanted["explicit_asset"]
        ]
        if len(matches) != 1:
            raise AdapterError(
                "the transaction carries %d outputs matching the normalized "
                "claim, and a public record may name exactly one" % len(matches)
            )
        return matches[0]

    # -- Process B ---------------------------------------------------------

    def verify(self, subject: dict) -> dict:
        """Reconstructs the public fact from the handoff and nothing else.

        Every check records what it looked for and what it found, and
        none of them decides whether the run passes: that judgement is
        the typed report's, built in the package that owns the claim from
        a record this process did not classify.
        """
        handoff = subject["handoff"]
        checks = []

        def record(name, expected, observed):
            # The agreement is decided over the values themselves and
            # the spelling happens afterwards, so nothing is decided by
            # how a value prints. Both sides are carried as text because
            # a member that were sometimes a number, sometimes a string,
            # and sometimes a flag is the untyped value tree revision 4
            # exists to remove (G12-R09).
            checks.append(
                {
                    "check": name,
                    "expected": str(expected),
                    "observed": str(observed),
                    "agrees": expected == observed,
                }
            )

        # The chain context is settled before any evidence is fetched. A
        # run that read one chain's transaction while declaring another's
        # identity would be a false record, and no later check repairs
        # it: the txid would resolve, the amount would parse, and every
        # one of them would be about the wrong chain.
        observed_genesis = self.node.call("getblockhash", "0")
        record("chain_context_genesis", handoff["genesis_id"], observed_genesis)
        if handoff["genesis_id"] != observed_genesis:
            return {
                "outcome": "refused_wrong_chain_context",
                "checks": checks,
                "spend": None,
            }

        # 1. locate the public evidence, by the block locator the record
        #    names. A block locator rather than a transaction index,
        #    because an index is a node configuration and the locator is
        #    chain data.
        try:
            chain_raw = self.node.call(
                "getrawtransaction", handoff["txid"], "false", handoff["block_hash"]
            )
        except AdapterError as error:
            record("evidence_located", handoff["txid"], None)
            return {
                "outcome": "refused_evidence_absent",
                "checks": checks,
                "spend": None,
                "detail": error.note,
            }
        record("evidence_located", handoff["txid"], handoff["txid"])

        # The record's own copy of the bytes is checked against the
        # chain's rather than used in its place. This is what catches a
        # record built from another transaction's data: the copy would
        # parse perfectly and disagree with the chain byte for byte.
        record("evidence_bytes_match_chain", chain_raw, handoff["raw_transaction"])
        if chain_raw != handoff["raw_transaction"]:
            return {
                "outcome": "refused_copied_evidence",
                "checks": checks,
                "spend": None,
            }

        # 2. parse it canonically -- with the framework's own
        #    deserializer, so the amount does not come from the node's
        #    opinion of its own transaction.
        parsed = self.conservation.deserialize(chain_raw)
        index = handoff["output_index"]
        if index < 0 or index >= len(parsed.vout):
            record("output_index_in_range", "0..%d" % len(parsed.vout), index)
            return {
                "outcome": "refused_output_absent",
                "checks": checks,
                "spend": None,
            }
        record("output_index_in_range", True, True)
        out = parsed.vout[index]

        # 3. bind it to the intended output, and 5. recover the public
        #    semantic amount. Both are the same read on this path: the
        #    amount and the asset are ON the output, which is what makes
        #    the explicit representation need no capsule.
        try:
            amount = explicit_amount(bytes(out.nValue.vchCommitment), "output.value")
            asset = bytes(out.nAsset.vchCommitment)
        except AdapterError:
            record("output_is_explicit", True, False)
            return {
                "outcome": "refused_output_not_explicit",
                "checks": checks,
                "spend": None,
            }
        record("output_is_explicit", True, True)
        record("explicit_amount", handoff["claimed_explicit_amount"], amount)
        # Stored least-significant-first behind the explicit prefix, and
        # displayed the other way round.
        record(
            "explicit_asset",
            handoff["claimed_explicit_asset"],
            asset[1:][::-1].hex() if len(asset) == 33 else asset.hex(),
        )

        # 4. verify the opening. There is none on this path, and the
        #    owner projection is checked in its place: the script the
        #    chain carries is rebuilt from the public address alone,
        #    without asking the node, and byte-compared.
        chain_script = bytes(out.scriptPubKey)
        rebuilt = bytes(
            self.address_module.address_to_scriptpubkey(
                handoff["claimed_owner_address"]
            )
        )
        record("owner_projection_bytes", chain_script.hex(), rebuilt.hex())

        # The unspent-output set is asked as well, because a record may
        # name a transaction that exists and an output that is gone.
        unspent = self.node.call("gettxout", handoff["txid"], str(index))
        record("output_unspent", True, unspent is not None)
        if unspent is None:
            # The transaction is still in the chain and every byte of it
            # still parses. What is gone is the object: a later
            # transaction consumed it, so a record naming it is true
            # about the past and false about what can be used now. This
            # is the explicit-path form of section 13.5's stale capsule.
            return {
                "outcome": "refused_output_spent",
                "checks": checks,
                "spend": None,
            }
        record(
            "utxo_set_amount",
            handoff["claimed_explicit_amount"],
            int(round(float(unspent["value"]) * 100_000_000)),
        )

        # No owner-private witness anywhere in this process. The wallet
        # this process created is asked whether it can spend the object,
        # and the answer has to be no: a Process B that could spend it
        # would be evidence that the boundary leaked, not that the object
        # is public.
        self.conservation.prepare()
        spendable = [
            entry
            for entry in self.node.call(
                "listunspent", "0", "9999999", wallet=self.conservation.WALLET
            )
            if entry["txid"] == handoff["txid"] and entry["vout"] == index
        ]
        record("owner_object_spendable_by_this_process", False, spendable != [])

        # 6-7. the future construction, of this process's OWN funds,
        #      paying an address derived fresh here.
        spend = self.spend_own_funds(handoff)
        return {"outcome": "verified", "checks": checks, "spend": spend}

    def spend_own_funds(self, handoff: dict) -> dict:
        """Builds, confirms, and reports one spend of this process's funds.

        The point is not the transaction. It is that the process which
        read the public record is a working constructor on this chain, so
        a failure to spend the normalized output would be a statement
        about ownership rather than about this process's ability to build
        anything at all.
        """
        destination = self.conservation.address(False, NORMALIZATION_ADDRESS_TYPE)
        program = self.normalization.script_of(destination)
        amount = LIFECYCLE_SPONSOR_SPEND_SATOSHIS
        # Captured before the spend, because the outpoint it consumes is
        # the whole claim being made: this process built a transaction
        # out of funds it located itself, and none of them is the
        # owner's object.
        source = dict(self.executor.change)
        txid = self.executor.fund(program, amount)
        if self.node.call("gettxout", txid, "0") is None:
            raise AdapterError("the spend this process built did not reach the chain")
        return {
            "txid": txid,
            "amount": amount,
            "destination": destination,
            "destination_script": program.hex(),
            "consumed_outpoint": {"txid": source["txid"], "vout": source["vout"]},
            "consumed_owner_object": (
                source["txid"] == handoff["txid"]
                and source["vout"] == handoff["output_index"]
            ),
        }


def satoshis_to_amount(satoshis: int) -> str:
    """One amount in the decimal form the node's RPC reads."""
    return "%d.%08d" % (satoshis // 100_000_000, satoshis % 100_000_000)


def satoshis_to_amount_float(amount) -> float:
    """One RPC-reported amount, passed back unchanged in value."""
    return float(amount)


def script_error_in(text: str, prefix: str):
    """The script error a wrapper carries, or None if it carries none.

    # Why this counts depth instead of finding a parenthesis

    The consensus judgement used to take the substring up to the FIRST
    `)` after the wrapper opened. Two of the messages the class table
    holds carry a parenthesis of their own --
    `Signature must be zero for failed CHECK(MULTI)SIG operation` and
    `OP_CHECKMULTISIG(VERIFY) is not available in tapscript` -- so each
    truncated to a string the table does not hold, the lookup missed,
    and a rejection the target had classified precisely arrived with no
    class at all. Nothing failed loudly: a lost class reads as a target
    that refused for an unclassified reason (G12-R07).

    So the wrapper is closed by matching the parenthesis it opened,
    which is what "the exact outer wrapper" means when the payload may
    contain parentheses. The relay judgement's whole-wrapper reading was
    already correct and is now expressed through this same function,
    because two spellings of one rule are two things that can disagree.

    Returns None where the wrapper is absent or never closes, so a
    caller distinguishes "not a script verdict" from "a script verdict
    naming nothing".
    """
    opening = prefix.rindex("(")
    head = prefix[:opening]
    position = text.find(head + "(")
    if position < 0:
        return None
    start = position + len(head) + 1
    depth = 1
    for index in range(start, len(text)):
        if text[index] == "(":
            depth += 1
        elif text[index] == ")":
            depth -= 1
            if depth == 0:
                return text[start:index]
    # An unbalanced wrapper is not a verdict this adapter can read. It
    # is reported as unreadable rather than repaired by guessing where
    # the message ended.
    return None


def observed_key_path_spend(node, raw: str) -> bool:
    """Whether EVERY input of these bytes took the taproot key path.

    # The wire fact this exists to stop misreporting

    Elements answers a taproot key-path signature failure with the same
    `mandatory-script-verify-flag-failed (...)` wrapper it answers a leaf
    failure with, so the reject reason alone cannot tell the two apart.
    Classifying on that string alone files a key-path refusal as
    `script_path_rejection`, which claims a covenant script was reached
    and refused when none was offered at all. The internal-key
    unspendability probe met exactly that and reported it rather than
    working around it; this function is the observation that repairs it.

    # It OBSERVES, and never infers from an expectation

    Nothing here is told what the caller intended. Two facts are read
    back from the node -- the witness the bytes actually carry, and the
    program each spent output actually pays -- and a key-path spend is
    the conjunction of them: a witness-version-one program, and a witness
    of exactly one item once an annex is set aside. A one-item witness
    alone is not enough, because at a version-zero P2WSH input that one
    item IS the script and a script does run.

    # Silence is not a claim

    Where a spent output cannot be read back, or a witness cannot be, the
    shape was not observed and this answers False. The caller then
    reports what it reported before, which understates rather than
    invents: an unobserved key path is not evidence of a script path, and
    the honest cost of that is carried by the caller's own record rather
    than by a guess made here.
    """
    try:
        decoded = node.call("decoderawtransaction", raw)
    except AdapterError:
        return False
    if not isinstance(decoded, dict):
        return False
    inputs = decoded.get("vin")
    if not isinstance(inputs, list) or not inputs:
        return False

    for entry in inputs:
        if not isinstance(entry, dict):
            return False
        witness = entry.get("txinwitness")
        if not isinstance(witness, list) or not witness:
            return False
        # An annex is the trailing item beginning 0x50, and it is set
        # aside before the count is read because it is not a stack item
        # the target executes. This workspace builds none; the rule is
        # written anyway so that a candidate carrying one is classified
        # by what it spends rather than by an item nobody ran.
        items = list(witness)
        if len(items) >= 2 and isinstance(items[-1], str) and items[-1][:2] == "50":
            items = items[:-1]
        if len(items) != 1:
            return False

        txid = entry.get("txid")
        index = entry.get("vout")
        if not isinstance(txid, str) or isinstance(index, bool) or not isinstance(index, int):
            return False
        try:
            spent = node.call("gettxout", txid, str(index))
        except AdapterError:
            return False
        if not isinstance(spent, dict):
            return False
        program = spent.get("scriptPubKey", {}).get("hex")
        # Witness version one, thirty-two byte program: the only shape
        # whose one-item witness is a key-path spend.
        if not isinstance(program, str) or len(program) != 68 or program[:4] != "5120":
            return False

    return True


def rejection(script_error: str) -> dict:
    """Builds a rejection body, naming any script error the table cannot map."""
    text = script_error.strip()
    failure = FAILURE_CLASS_BY_SCRIPT_ERROR.get(text)
    if failure is None:
        # The unclassified message is the target's own text, lifted out of
        # the client's stderr, and it is exactly the material the
        # quarantine is for: it is the reason a class is missing, so it
        # has to be recoverable, and it is uncontrolled text, so it may
        # not travel in a typed line. The response carries no class, which
        # is what "not classified" means on the wire; the record number is
        # how an operator finds the message that needs a table entry.
        record = quarantined("unclassified script error", text)
        log(
            "rejected, with a script error this adapter does not classify; "
            "the message is elements-output record %d" % record
        )
    return {"verdict": "rejected", "observed_failure": failure}


# --------------------------------------------------------------------------
# Protocol
# --------------------------------------------------------------------------


def write_message(value: dict) -> None:
    """Writes one NDJSON protocol line on stdout."""
    sys.stdout.write(json.dumps(value, separators=(",", ":")) + "\n")
    sys.stdout.flush()


# The request bounds, mirrored from the harness's own contract
# (`packages/target-elements-conformance/src/protocol.rs`, the
# MAXIMUM_HANDSHAKE_REQUEST_BYTES and MAXIMUM_REQUEST_BYTES constants).
#
# Mirrored rather than negotiated because the bound is the contract's and
# not either implementation's. The harness enforces it on the way out and
# this adapter enforces it on the way in; a figure that only one side held
# would be a framing only one side had, which is the two-sided
# disagreement protocol revision 4 was minted to end. A cross-language
# test reads these two lines out of this file and compares them with the
# Rust constants, so the two cannot drift in silence.
MAXIMUM_HANDSHAKE_REQUEST_BYTES = 64 * 1024
MAXIMUM_REQUEST_BYTES = 4 * 1024 * 1024

# The framing failures this reader tells apart, as fixed spellings.
#
# Five distinctions and not one "bad record", because they are five
# different things to have gone wrong and a caller reading the diagnostic
# stream has five different next steps. A framing that collapsed them
# would report a harness that stopped writing and a harness that wrote
# four megabytes without a newline as the same event.
FRAMING_CLEAN_EOF = "clean_eof"
FRAMING_BLANK_RECORD = "blank_record"
FRAMING_MALFORMED_RECORD = "malformed_record"
FRAMING_OVERSIZED_RECORD = "oversized_record"
FRAMING_UNTERMINATED_RECORD = "unterminated_record"


class ProtocolFramingError(FatalAdapterError):
    """A request this adapter refused to read, named by what was wrong.

    `failure` is one of the fixed spellings above and `phase` is the part
    of the exchange the reader was in, so the diagnostic that follows is
    two typed facts rather than a sentence assembled from whatever was in
    the buffer. Nothing from the record itself is carried: a record
    refused for its framing is exactly the record whose bytes have not
    been established as anything.
    """

    def __init__(self, failure: str, phase: str) -> None:
        super().__init__("%s in the %s phase" % (failure, phase))
        self.failure = failure
        self.phase = phase


class RequestReader:
    """Reads strict newline-delimited JSON requests, under an explicit bound.

    The same framing the harness enforces on this adapter's answers
    (`packages/target-elements-conformance/src/executor.rs`, `read_record`
    and `bounded_read`), applied to the requests: one nonempty JSON object,
    one newline, and nothing else.

    # Why bounded

    At most `maximum + 1` bytes are taken before a record is refused. A
    harness that wrote without ever emitting a newline would otherwise
    make this process allocate until the host stopped it, which turns a
    typed framing failure into a resource failure of the process that was
    supposed to report it. The bound is not protection from the harness --
    the harness already owns this process -- it is the difference between
    a refusal this adapter states and a refusal the kernel states for it.

    # Why the cases stay apart

    A stream that ends at a record boundary is the exchange finishing. A
    stream that ends part-way through a record is a harness that died
    mid-write. A record that reached the bound without a newline is a
    record too large to be one. A blank record is a harness that wrote
    something that says nothing -- and skipping it, which this adapter used
    to do, means a framing that cannot tell "the harness said nothing here"
    from "the harness is finished". Only the first of the four is a clean
    exit.

    The reader works on the byte stream rather than the decoded one. A
    bound is a count of bytes, and a text stream that has already applied
    universal newlines and its own decoding has already done the reading
    the bound exists to limit.
    """

    def __init__(self, stream) -> None:
        self.stream = stream

    def read(self, maximum: int, phase: str):
        """One record as text, or None at a clean end of stream.

        Raises `ProtocolFramingError` for each of the four framing
        failures. A record that is well framed but not JSON is the
        caller's to refuse, because "not JSON" is a fact about the
        record's content rather than about where it ended.
        """
        chunk = self.stream.readline(maximum + 1)
        if chunk == b"":
            return None
        if not chunk.endswith(b"\n"):
            # Either the bound was reached without a newline, or the
            # stream ended mid-record. The first is a record too large to
            # be one; the second is a harness that stopped writing in the
            # middle of one, and they are not the same fault.
            raise ProtocolFramingError(
                FRAMING_OVERSIZED_RECORD
                if len(chunk) > maximum
                else FRAMING_UNTERMINATED_RECORD,
                phase,
            )
        try:
            record = chunk[:-1].decode("utf-8")
        except UnicodeDecodeError:
            raise ProtocolFramingError(FRAMING_MALFORMED_RECORD, phase) from None
        if record.strip() == "":
            raise ProtocolFramingError(FRAMING_BLANK_RECORD, phase)
        return record

    def read_object(self, maximum: int, phase: str):
        """One record, decoded to a JSON object, or None at a clean end.

        The object-ness is part of the framing rather than of the
        request's meaning: every record this protocol defines is one JSON
        object, so an array or a bare number is a malformed record and not
        a request with a surprising shape.
        """
        record = self.read(maximum, phase)
        if record is None:
            return None
        try:
            value = json.loads(record)
        except json.JSONDecodeError:
            raise ProtocolFramingError(FRAMING_MALFORMED_RECORD, phase) from None
        if not isinstance(value, dict):
            raise ProtocolFramingError(FRAMING_MALFORMED_RECORD, phase)
        return value


def resources_for(fixture, weight) -> dict:
    """Reports the observations this executor can actually make.

    Two figures are the fixture's own and are restated exactly, so that a
    harness comparing them catches an executor that ran something other than
    what it was handed. The transaction's weight is a real observation, read
    back from the node.

    Everything else is null, and null is the point: a node exposes no
    interpreter stack, so the peak depths, the largest element, and the
    validation budget are not measured here. Reporting them as zero -- which
    is what this adapter had to do before the schema admitted absence --
    turned "not observed" into a measurement of nothing.
    """
    return {
        "script_bytes": len(fixture["script"]) if fixture else 0,
        "initial_stack_items": len(fixture["initial_stack"]) if fixture else 0,
        "peak_stack_items": None,
        "peak_altstack_items": None,
        "maximum_element_bytes": None,
        "validation_budget_used": None,
        "transaction_weight": weight,
    }


def node_provenance(elementsd: str):
    """Reads the node's own version line and the revision it embeds.

    The revision returned is the *binary's* own, or None. There is no
    checkout fallback: a working tree's HEAD identifies intended source, and
    reporting it here would attribute one program's identity to another.
    """
    completed = subprocess.run(
        [elementsd, "--version"], stdin=subprocess.DEVNULL, capture_output=True, text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise FatalAdapterError("the node binary did not report a version")
    first_line = completed.stdout.strip().splitlines()[0].strip()
    name = first_line.split(" ")[0] if first_line else "unknown"
    match = re.search(r"-([0-9a-f]{7,40})(-dirty)?$", first_line)
    return name, first_line, (match.group(1) if match else None)


def framework_revision(framework_path):
    """The revision of the transaction framework this adapter builds with.

    Read from the framework checkout, which is exactly what this field is
    about: the framework is source this adapter imports and runs in-process,
    so its checkout state *is* what executed. That is not true of the node
    binary, whose field is left alone.
    """
    if framework_path is None:
        return None
    completed = subprocess.run(
        ["git", "-C", str(framework_path), "rev-parse", "HEAD"],
        stdin=subprocess.DEVNULL, capture_output=True, text=True, check=False,
    )
    return completed.stdout.strip() if completed.returncode == 0 else None


def identifier(text: str, role: str) -> bytes:
    """One 32-byte public identifier, from 64 hex digits."""
    try:
        value = bytes.fromhex(text)
    except ValueError:
        raise FatalAdapterError("the %s identifier is not hex" % role) from None
    if len(value) != 32:
        raise FatalAdapterError("the %s identifier must be 32 bytes" % role)
    return value


def initialize_confidential_materializer(executor, library) -> None:
    """Initializes the optional materializer without retaining path text."""
    if library:
        try:
            materializer = ConfidentialMaterializer(library)
        except Exception:
            # The loader's exception text is discarded. It may carry the
            # operator-selected path, and neither diagnostic destination is
            # approved to retain that path.
            log(DiagnosticOutcome.CONFIDENTIAL_MATERIALIZER_INITIALIZATION_FAILED)
        else:
            executor.operations.materializer = materializer
            log(DiagnosticOutcome.CONFIDENTIAL_MATERIALIZER_READY)
    else:
        log(DiagnosticOutcome.CONFIDENTIAL_MATERIALIZER_UNAVAILABLE)


def serve(arguments) -> int:
    """Runs the whole exchange, and destroys the node whatever happens."""
    framework_path, messages, script, key_module = load_framework(arguments.framework)
    # That the framework loaded, and not where from. The path is the
    # operator's own argv, and this adapter's diagnostics are no more a
    # place for an operator's directory layout than a first-party record
    # is -- the two files travel together, and a path written in either is
    # a path that has left this host.
    log("framework loaded")
    name, version, revision = node_provenance(arguments.elementsd)
    network_id = identifier(arguments.network_id, "network")
    topics = [topic for topic in (arguments.included_local_topic or []) if topic]

    reader = RequestReader(sys.stdin.buffer)
    handshake = reader.read_object(MAXIMUM_HANDSHAKE_REQUEST_BYTES, "handshake")
    if handshake is None:
        # A clean end of stream here is still fatal, because the exchange
        # has not happened. The distinction the reader keeps is between a
        # stream that ended where a record may end and one that ended
        # inside a record; whether ending there was allowed is this
        # caller's question, and in the handshake phase it is not.
        raise ProtocolFramingError(FRAMING_CLEAN_EOF, "handshake")
    # An exact field census, mirroring the harness's own
    # `deny_unknown_fields` on `HandshakeRequest`. A revision-4 handshake
    # carries `schema` and nothing else, so a record with an extra member
    # is a harness this adapter does not agree with about what revision 4
    # is -- which is the very state the revision was minted to end, and is
    # therefore refused rather than read past.
    try:
        require_keys(handshake, ("schema",), "handshake")
    except AdapterError as error:
        # The field's NAME is the harness's own text and is bounded only
        # by the record bound, so it is quarantined like any other
        # unreviewed string rather than interpolated into a typed line.
        record = quarantined("handshake field census", error.note)
        raise FatalAdapterError(
            "the handshake failed its field census; the detail is "
            "elements-output record %d" % record
        ) from None
    if handshake["schema"] != NATIVE_PROTOCOL_SCHEMA:
        raise FatalAdapterError("the harness spoke a protocol revision this adapter does not")

    node = DisposableNode(
        arguments.elementsd, arguments.elements_cli, arguments.chain,
        arguments.boot_timeout_seconds, arguments.enable_wallet,
        arguments.datadir,
    )
    closing = {"done": False}

    def close(*_ignored):
        if not closing["done"]:
            closing["done"] = True
            node.stop()

    for received in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        try:
            signal.signal(received, lambda *_a: (close(), os._exit(1)))
        except (ValueError, OSError):
            pass

    try:
        started = time.monotonic()
        node.start()
        executor = CaseExecutor(node, messages, script, key_module)
        executor.prime()
        if arguments.enable_wallet:
            executor.conservation = ConservationExecutor(executor, arguments.wallet_name)
            executor.normalization = NormalizationExecutor(executor.conservation)
            executor.lifecycle = LifecycleExecutor(
                executor.normalization, arguments.network_id
            )
            # A wallet of its own, named apart from the conservation
            # lane's, so two lanes sharing one chain directory cannot
            # share one wallet.
            executor.operations = OperationExecutor(
                executor,
                None if arguments.wallet_name is None else arguments.wallet_name + "-ops",
            )
            # The deterministic materializer, where the node's own
            # zero-knowledge library can be reached. A failure to reach
            # it is recorded and leaves the materializer absent; it is
            # never fatal, because an adapter that cannot materialize
            # confidential funding is an adapter that advertises none.
            library = locate_zero_knowledge_library(
                arguments.elementsd, arguments.zk_library
            )
            initialize_confidential_materializer(executor, library)
        log("node ready in %.1fs" % (time.monotonic() - started))

        write_message(
            {
                "protocol_schema": NATIVE_PROTOCOL_SCHEMA,
                "adapter_name": COMMAND_NAME,
                "adapter_version": ADAPTER_VERSION,
                "framework_revision": framework_revision(framework_path),
                "node_name": name,
                "node_version": version,
                # The binary's own revision, or nothing. Never a checkout.
                "binary_reported_revision": revision,
                "intended_executed_tip": arguments.intended_executed_tip,
                "upstream_base": arguments.upstream_base,
                "included_local_topics": sorted(set(topics)),
                "supported_domains": ["tapscript"],
                "supported_leaf_versions": [TAPSCRIPT_LEAF_VERSION],
                # Both tree-bearing capabilities are advertised only where
                # the loaded framework actually offers the helper that
                # builds a taproot commitment, so the harness never sends a
                # request to an adapter that could only refuse it
                # (Guide-10 rule:guide10:schema-migration).
                #
                # They are two claims and not one. Materializing a stated
                # tree is what a primitive case with a construction needs;
                # reading a compound-prototype record is a further claim
                # about the record shape itself, and the harness requires
                # both before it sends one.
                "capabilities": [
                    "failure_class_reporting",
                    "resource_observation",
                    "transaction_context",
                ]
                + (["tree_materialization"] if executor.tree_materialization else [])
                + (
                    ["compound_prototype_fixtures"]
                    if executor.prototype_fixtures
                    else []
                )
                # Advertised only where the wallet the conservation lane
                # needs is actually enabled, so the harness never sends a
                # row to an adapter that could only refuse it.
                + (
                    ["confidential_conservation"]
                    if arguments.enable_wallet
                    else []
                )
                # The normalization claim rests on the same wallet, and is
                # a further claim on top of it: an executor that can build
                # a confidential transaction still has to resolve a claim,
                # sign under the section 10.3 profile, and read the output
                # set back from the target.
                + (
                    ["owner_authorized_normalization"]
                    if arguments.enable_wallet
                    else []
                )
                # The section 13 lane, on the same wallet again. It is a
                # further claim still: building the object is one thing,
                # and reading it back from chain data with no relationship
                # to the process that built it is the thing section 13
                # actually asks about.
                + (
                    ["fresh_process_lifecycle"]
                    if arguments.enable_wallet
                    else []
                )
                # The Guide-12 section 16.2 operation lane, and the one
                # place two capabilities are advertised on one condition.
                # They are still two claims: creating spendable outputs at
                # a caller's program and judging a caller's complete
                # transaction are different work. What ties them to one
                # condition is that this adapter's issuance needs an
                # address, an address needs a wallet, and a submission
                # step with nothing funded to spend could only be
                # refused -- so advertising the second without the first
                # would be advertising work this adapter cannot usefully
                # be asked for.
                + (
                    ["test_funding_ceremony", "target_transaction_submission"]
                    if executor.operations is not None
                    else []
                )
                # Acting as a sponsor: holding reserve value at a program
                # of this adapter's own and authorizing a spend of it. It
                # rests on the operation lane, because the coin is
                # created by the same machinery, and on the framework
                # actually shipping the key and signature-hash helpers,
                # because this adapter implements neither itself.
                #
                # The claim is about this disposable chain and nothing
                # else. The key it authorizes with is a published
                # constant belonging to a genesis this process created
                # `(ADR-015 rule test-material)`, so what it establishes
                # is that the candidate's transaction is one the target
                # accepts -- never anything about a sponsor's standing.
                + (
                    ["test_sponsor_authorization"]
                    if executor.operations is not None
                    and executor.sponsor_authorization
                    else []
                )
                # The confidential arm, advertised only where the
                # deterministic materializer is actually reachable. No
                # reviewed stock interface produces the explicit-asset /
                # confidential-value form, so what stands behind this
                # claim is this adapter's own materializer over the
                # node's own zero-knowledge library -- and where that
                # library could not be reached, the honest answer is no
                # capability and no advertisement rather than a
                # capability that could only refuse.
                #
                # The capability and the advertisement are written on ONE
                # condition on purpose. The harness refuses a peer that
                # states either without the other, so writing them apart
                # would be writing a contradiction this adapter could
                # produce.
                + (
                    ["confidential_value_test_funding"]
                    if executor.operations is not None
                    and executor.operations.materializer is not None
                    else []
                )
                # The join of the two above, advertised on the
                # conjunction of their conditions because it is a claim
                # about doing both to ONE coin: committing its value at
                # funding time and still being able to authorize a spend
                # of it afterwards. Neither neighbour implies it -- the
                # confidential arm creates coins it never spends, and the
                # sponsor arm spends coins whose value is a number -- and
                # what makes it true here is that the funding step
                # retains the value FIELD rather than an amount.
                + (
                    ["confidential_value_sponsor_authorization"]
                    if executor.operations is not None
                    and executor.sponsor_authorization
                    and executor.operations.materializer is not None
                    else []
                ),
                "confidential_funding": (
                    {
                        "representation_profiles": ["explicit_asset_confidential_value"],
                        "custody_profiles": ["central_public_fixtures"],
                        "materializer_profiles": ["guide_ctf_deterministic_v1"],
                        # Byte identity alone. The recorded-randomness
                        # contract is designed and typed on both sides,
                        # and this adapter implements no opening source
                        # for it, so advertising it would be advertising
                        # a comparison nothing here can make.
                        "reproducibility_contracts": ["byte_identity"],
                    }
                    if executor.operations is not None
                    and executor.operations.materializer is not None
                    else None
                ),
            }
        )

        # What this run actually executed on, read from the node that is
        # now up. The genesis is the node's answer; the chain name is what
        # it was configured to run; the network identity is the name the
        # binding gives that chain, restated only now that its genesis has
        # been observed.
        genesis = identifier(node.call("getblockhash", "0"), "genesis")
        write_message(
            {
                "schema": NATIVE_PROTOCOL_SCHEMA,
                "environment": "development",
                "chain_name": arguments.chain,
                "network_id": list(network_id),
                "genesis_id": list(genesis),
                "active_domains": ["tapscript"],
                "active_leaf_versions": [TAPSCRIPT_LEAF_VERSION],
            }
        )

        while True:
            request = reader.read_object(MAXIMUM_REQUEST_BYTES, "request")
            if request is None:
                # The one clean ending. The harness closed its side at a
                # record boundary, which is how the exchange finishes;
                # every other way a record can end is a framing failure
                # the reader has already raised.
                log("the request stream ended cleanly at a record boundary")
                break
            answer_case(executor, request)
    finally:
        close()
    return 0


def require_execution_request_fields(request: dict, allowed: tuple) -> None:
    """Refuses and quarantines one unknown execution-request field."""
    for key in request:
        if key not in allowed:
            record = quarantined(
                "execution request field census",
                "unknown field: request.%s" % key,
            )
            raise FatalAdapterError(
                outcome=DiagnosticOutcome.EXECUTION_REQUEST_FIELD_CENSUS_FAILED,
                record=record,
            ) from None


def answer_case(executor: CaseExecutor, request: dict) -> None:
    """Answers exactly one execution request, primitive or compound.

    The two records are told apart by the case identity they carry, which
    is the one field whose shape differs between them: a primitive case is
    a group and an ordinal, and a compound one is a relation and a name.
    Reading the fixture first and inferring the record from which fields
    parsed would mean deciding what was asked from what happened to be
    readable.

    The record arrives decoded. Framing -- where the record ended, whether
    it was JSON at all, whether it was an object -- is `RequestReader`'s,
    and settled before this function sees anything; what is left here is
    what the request MEANS.
    """
    case = request.get("case")
    # A conservation row is told apart the same way the other two records
    # are: by the one field whose shape differs. A primitive case is a
    # group and an ordinal, a compound one is a relation and a name, and a
    # conservation row is an ordinal and a name.
    if isinstance(case, dict) and "ordinal" in case and "name" in case:
        answer_conservation_row(executor, request, case)
        return
    # A normalization row is told apart the same way, by the one field
    # whose shape differs: it names a mutation and nothing else.
    if isinstance(case, dict) and "normalization" in case:
        answer_normalization_row(executor, request, case)
        return
    # A section 13 lifecycle step names its role and nothing else.
    if isinstance(case, dict) and "lifecycle" in case:
        answer_lifecycle_step(executor, request, case)
        return
    # A section 16.2 operation step names a kind and the caller's own name
    # for it. Told apart the same way as the rest: by the one field whose
    # shape differs.
    if isinstance(case, dict) and "operation" in case and "step" in case:
        answer_operation_step(executor, request, case)
        return
    # The case identity is echoed verbatim, so that the harness correlates
    # against exactly what it sent. Without one there is nothing to answer,
    # and answering the wrong case would be worse than not answering.
    if not isinstance(case, dict):
        raise FatalAdapterError("the harness sent a request naming no case")
    # A compound-prototype request carries no separate construction field:
    # the subject states its own. Admitting one here would admit a message
    # stating two constructions.
    compound = "relation" in case
    allowed = ("schema", "case", "subject") if compound else (
        "schema",
        "case",
        "subject",
        "construction",
    )
    require_execution_request_fields(request, allowed)
    fixture = None
    body = None
    try:
        if request.get("schema") != NATIVE_PROTOCOL_SCHEMA:
            raise AdapterError("the request carries a protocol revision this adapter does not")
        if compound:
            # A compound request sent to an adapter that advertised no
            # compound-prototype capability is the harness contradicting the
            # handshake it was given. The case is refused rather than
            # answered from machinery this adapter said it does not have.
            if not executor.prototype_fixtures:
                raise AdapterError(
                    "the request is a compound-prototype fixture, and this "
                    "adapter advertised no compound-prototype capability"
                )
            fixture, construction = parse_prototype_fixture(request.get("subject"))
        else:
            fixture = parse_subject(request.get("subject"))
            construction = request.get("construction")
            if construction is not None:
                # A construction sent to an adapter that advertised no tree
                # materialization is the same contradiction, for the same
                # reason: the case is refused rather than answered from a
                # tree this adapter would have had to invent.
                if not executor.tree_materialization:
                    raise AdapterError(
                        "the request states a taproot construction, and this "
                        "adapter advertised no tree materialization"
                    )
                construction = parse_construction(construction)
        started = time.monotonic()
        body = executor.execute(fixture, construction)
        log("case answered in %.2fs" % (time.monotonic() - started))
    except AdapterError as error:
        log("infrastructure error: %s" % error.note)
        body = {"verdict": "infrastructure_error", "observed_failure": None}
    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "verdict": body["verdict"],
            "final_stack": None,
            "final_altstack": None,
            "observed_failure": body["observed_failure"],
            "resources": resources_for(fixture, body.get("transaction_weight")),
        }
    )


def answer_conservation_row(executor: CaseExecutor, request: dict, case: dict) -> None:
    """Answers exactly one Guide-11 section 8.4 conservation row.

    The three failure layers that are not target verdicts are produced
    here and nowhere else, from what actually went wrong: a fixture this
    adapter could not build, an environment that failed around the run,
    and -- in `ConservationExecutor.judge` -- the layer the node itself
    answered at. None of them is chosen from an expectation, because the
    request carries none.
    """
    require_execution_request_fields(request, ("schema", "case", "subject"))

    body = None
    try:
        if request.get("schema") != NATIVE_PROTOCOL_SCHEMA:
            raise AdapterError("the request carries a protocol revision this adapter does not")
        if executor.conservation is None:
            raise AdapterError(
                "the request is a conservation row, and this adapter advertised no "
                "confidential-conservation capability"
            )
        subject = request.get("subject")
        if not isinstance(subject, dict):
            raise AdapterError("the conservation request states no subject")
        require_keys(subject, ("inputs", "outputs", "defect"), "request.subject")
        started = time.monotonic()
        body = executor.conservation.execute(case, subject)
        log("row %s answered in %.2fs as %s"
            % (case.get("name"), time.monotonic() - started, body["observed_layer"]))
    except ConstructionError as error:
        log("fixture construction failure: %s" % error.note)
        body = {
            "observed_layer": "fixture_construction_failure",
            "observed_detail": error.note,
            "transaction_bytes": None,
            "observed_value_commitments": [],
            "observed_asset_commitments": [],
            "observed_openings": [],
        }
    except AdapterError as error:
        log("executor infrastructure failure: %s" % error.note)
        body = {
            "observed_layer": "executor_infrastructure_failure",
            "observed_detail": error.note,
            "transaction_bytes": None,
            "observed_value_commitments": [],
            "observed_asset_commitments": [],
            "observed_openings": [],
        }

    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "observed_layer": body["observed_layer"],
            "observed_detail": body["observed_detail"],
            "transaction_bytes": body["transaction_bytes"],
            "observed_value_commitments": body["observed_value_commitments"],
            "observed_asset_commitments": body["observed_asset_commitments"],
            "observed_openings": body["observed_openings"],
        }
    )


def answer_normalization_row(executor: CaseExecutor, request: dict, case: dict) -> None:
    """Answers exactly one Guide-11 section 10.4 normalization row.

    The two layers that are not target verdicts are produced here and
    nowhere else, from what actually went wrong. Neither is chosen from
    an expectation, because the request carries none -- and this record
    in particular must not, since three of its rows are answered by the
    report layer and an adapter that knew which ones could report a
    disagreement it never observed.
    """
    require_execution_request_fields(request, ("schema", "case", "subject"))

    body = None
    try:
        if request.get("schema") != NATIVE_PROTOCOL_SCHEMA:
            raise AdapterError("the request carries a protocol revision this adapter does not")
        if getattr(executor, "normalization", None) is None:
            raise AdapterError(
                "the request is a normalization row, and this adapter advertised no "
                "owner-authorized-normalization capability"
            )
        subject = request.get("subject")
        if not isinstance(subject, dict):
            raise AdapterError("the normalization request states no subject")
        require_keys(subject, ("claim", "mutation"), "request.subject")
        started = time.monotonic()
        body = executor.normalization.execute(case, subject)
        log("normalization row %s answered in %.2fs as %s"
            % (case.get("normalization"), time.monotonic() - started,
               body["observed_layer"]))
    except ConstructionError as error:
        log("fixture construction failure: %s" % error.note)
        body = {
            "observed_layer": "fixture_construction_failure",
            "observed_detail": error.note,
            "claimed_outputs": [],
            "observed_outputs": [],
            "authorization_profile": None,
            "observed_witness_sizes": [],
            "transaction_bytes": None,
        }
    except AdapterError as error:
        log("executor infrastructure failure: %s" % error.note)
        body = {
            "observed_layer": "executor_infrastructure_failure",
            "observed_detail": error.note,
            "claimed_outputs": [],
            "observed_outputs": [],
            "authorization_profile": None,
            "observed_witness_sizes": [],
            "transaction_bytes": None,
        }

    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "observed_layer": body["observed_layer"],
            "observed_detail": body["observed_detail"],
            "claimed_outputs": body["claimed_outputs"],
            "observed_outputs": body["observed_outputs"],
            "authorization_profile": body["authorization_profile"],
            "observed_witness_sizes": body["observed_witness_sizes"],
            "transaction_bytes": body["transaction_bytes"],
        }
    )


def parse_handoff(raw: object) -> dict:
    """Reads the section 13 public record strictly, or refuses it.

    Strict because this record is the ENTIRE channel between the process
    that created the object and the process that had no part in it. A
    field this schema does not know is either a mistake or a covert
    channel, and neither may be read leniently: the whole result would be
    a fresh-process proof conducted over a private side channel.

    The two rules are an allow-list and a name ban, and the redundancy is
    deliberate. The allow-list is what actually holds; the ban is what
    would still fire if a later wave widened the allow-list without
    thinking about what it was widening it for.
    """
    handoff = require_object(raw, "request.subject.handoff")
    for key in handoff:
        lowered = key.lower()
        for banned in FRESH_PROCESS_HANDOFF_BANNED_SUBSTRINGS:
            if banned in lowered:
                raise AdapterError(
                    "the public record carries a field named %s, and a public "
                    "record carries no owner-private material" % key
                )
    require_keys(handoff, FRESH_PROCESS_HANDOFF_FIELDS, "request.subject.handoff")
    if handoff["schema"] != FRESH_PROCESS_HANDOFF_SCHEMA:
        raise AdapterError(
            "the public record states a schema this adapter does not read: %s"
            % handoff["schema"]
        )
    for field in (
        "chain_name",
        "network_id",
        "genesis_id",
        "txid",
        "block_hash",
        "raw_transaction",
        "claimed_explicit_asset",
        "claimed_owner_address",
    ):
        require_string(handoff[field], "request.subject.handoff.%s" % field)
    for field in ("output_index", "block_height", "claimed_explicit_amount"):
        require_int(handoff[field], "request.subject.handoff.%s" % field)
    return handoff


def answer_lifecycle_step(executor: CaseExecutor, request: dict, case: dict) -> None:
    """Answers exactly one Guide-11 section 13 lifecycle step.

    Neither step decides anything. Process A states what it published,
    Process B states what it looked for and what it found, and the
    comparison that turns those into a verdict lives in the typed report
    -- for the reason `G11-W7-06` recorded, and which this lane would be
    the easiest place in the project to forget.
    """
    require_execution_request_fields(request, ("schema", "case", "subject"))

    body = None
    outcome = None
    try:
        if request.get("schema") != NATIVE_PROTOCOL_SCHEMA:
            raise AdapterError("the request carries a protocol revision this adapter does not")
        if getattr(executor, "lifecycle", None) is None:
            raise AdapterError(
                "the request is a lifecycle step, and this adapter advertised no "
                "fresh-process-lifecycle capability"
            )
        subject = request.get("subject")
        if not isinstance(subject, dict):
            raise AdapterError("the lifecycle request states no subject")
        role = case.get("lifecycle")
        started = time.monotonic()
        if role == "construct":
            require_keys(subject, ("claim", "supersede"), "request.subject")
            body = executor.lifecycle.construct(subject)
            outcome = "constructed"
        elif role == "verify":
            require_keys(subject, ("handoff",), "request.subject")
            body = executor.lifecycle.verify({"handoff": parse_handoff(subject["handoff"])})
            outcome = body["outcome"]
        else:
            raise AdapterError("the lifecycle request names no role this adapter runs")
        log("lifecycle step %s answered in %.2fs as %s"
            % (role, time.monotonic() - started, outcome))
    except ConstructionError as error:
        log("fixture construction failure: %s" % error.note)
        body = {}
        outcome = "fixture_construction_failure"
    except AdapterError as error:
        log("executor infrastructure failure: %s" % error.note)
        body = {"detail": error.note}
        outcome = "executor_infrastructure_failure"

    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "outcome": outcome,
            "handoff": body.get("handoff"),
            "authorization_profile": body.get("authorization_profile"),
            "observed_witness_sizes": body.get("observed_witness_sizes", []),
            "observed_outputs": body.get("observed_outputs", []),
            "checks": body.get("checks", []),
            "spend": body.get("spend"),
            "superseded_by": body.get("superseded_by"),
            "supersede_failure": body.get("supersede_failure"),
            "detail": body.get("detail"),
        }
    )


def parse_confidential_funding_subject(subject: dict) -> dict:
    """Reads one confidential funding subject, strictly.

    # Why this parses a request it will only refuse

    Because refusing a record is not the same as failing to read one.
    This adapter advertises no confidential funding capability, so the
    harness never sends it one; a record that arrives anyway is answered
    with a typed refusal rather than an exception, and it is read first
    so that the refusal is about the work rather than about the framing.

    Nothing is defaulted, ignored, or repaired. An unknown member, an
    unknown profile tag, an unknown representation tag, and an empty
    destination set are all refusals here, before any construction.
    """
    require_keys(
        subject,
        ("issue_asset", "asset", "destinations", "binding"),
        "request.subject",
    )
    issue = subject.get("issue_asset")
    if not isinstance(issue, bool):
        raise FatalAdapterError("request.subject.issue_asset is not a boolean")
    asset = subject.get("asset")
    if asset is not None and not isinstance(asset, str):
        raise FatalAdapterError("request.subject.asset is not a string")
    # The same cross-member rule the explicit arm states, spelled once
    # per arm because it is one question about the protocol asset.
    if issue and asset is not None:
        raise FatalAdapterError("an issuing funding step also named an asset")
    if not issue and asset is None:
        raise FatalAdapterError("a non-issuing funding step named no asset")

    destinations = parse_confidential_destinations(subject.get("destinations"))
    binding = parse_confidential_binding(subject.get("binding"))
    return {
        "issue_asset": issue,
        "asset": asset,
        "destinations": destinations,
        "binding": binding,
    }


def parse_confidential_destinations(raw: object) -> list:
    """Reads the ordered destination programs of one confidential
    funding subject.

    Shared by the arms that name a protocol asset and the arm that does
    not, because the ordered programs mean the same thing in both: they
    are what the fixture's outputs are paid to, in the order the fixture
    states them, and they are inside the digest.
    """
    if not isinstance(raw, list) or not raw:
        raise FatalAdapterError(
            "request.subject.destinations is not a nonempty list of destinations"
        )
    destinations = []
    for entry in raw:
        destination = require_object(entry, "request.subject.destinations[]")
        require_keys(destination, ("output_program",), "request.subject.destinations[]")
        destinations.append(
            {
                "output_program": require_bytes(
                    destination.get("output_program"),
                    "request.subject.destinations[].output_program",
                )
            }
        )
    return destinations


def parse_confidential_binding(raw: object) -> dict:
    """Reads the fixture binding of one confidential funding subject."""
    binding = require_object(raw, "request.subject.binding")
    require_keys(
        binding,
        ("fixture_handle", "fixture_digest", "profiles"),
        "request.subject.binding",
    )
    handle = binding.get("fixture_handle")
    if not isinstance(handle, str):
        raise FatalAdapterError("request.subject.binding.fixture_handle is not a string")
    digest = require_bytes(
        binding.get("fixture_digest"), "request.subject.binding.fixture_digest"
    )
    if len(digest) != 32:
        raise FatalAdapterError(
            "request.subject.binding.fixture_digest is not a thirty-two byte digest"
        )
    profiles = require_object(
        binding.get("profiles"), "request.subject.binding.profiles"
    )
    require_keys(
        profiles,
        ("representation", "custody", "materializer", "reproducibility_contract"),
        "request.subject.binding.profiles",
    )
    # The assigned tags, stated here because a tag this adapter has never
    # heard of is unknown rather than the nearest one it knows.
    assigned = {
        "representation": ("explicit_asset_confidential_value",),
        "custody": ("central_public_fixtures",),
        "materializer": ("guide_ctf_deterministic_v1",),
        "reproducibility_contract": ("byte_identity", "recorded_randomness"),
    }
    for member, admitted in assigned.items():
        stated = profiles.get(member)
        if stated not in admitted:
            raise FatalAdapterError(
                "request.subject.binding.profiles.%s names a tag this adapter does "
                "not read: %s" % (member, stated)
            )
    return {
        "fixture_handle": handle,
        "fixture_digest": digest,
        "profiles": {member: profiles[member] for member in assigned},
    }


def parse_confidential_sponsor_subject(subject: dict) -> dict:
    """Reads one confidential SPONSOR funding subject, strictly.

    # Why this subject names no asset where its sibling does

    The sibling funds coins of the protocol asset, which an earlier step
    issued and the caller therefore knows. A sponsor coin carries the
    chain's own reserve, which no step of a run chooses: the adapter
    reads it off the chain it was pointed at, and a caller that stated
    one would be stating a fact it learned from this adapter in the
    first place. So the member is absent rather than optional, and a
    record carrying one is refused here.
    """
    require_keys(subject, ("destinations", "binding"), "request.subject")
    return {
        "destinations": parse_confidential_destinations(subject.get("destinations")),
        "binding": parse_confidential_binding(subject.get("binding")),
    }


def parse_operation_subject(raw: object, kind: str) -> dict:
    """Reads one operation subject, refusing anything it does not define.

    The subjects share no member, which is what lets the kind stated in
    the case identity decide which one is admitted: a record carrying
    another kind's members is a request whose two halves disagree, and
    is refused here rather than answered by whichever half parsed.
    """
    subject = require_object(raw, "request.subject")
    if kind == "fund_confidential":
        return parse_confidential_funding_subject(subject)

    if kind == "fund_confidential_sponsor":
        return parse_confidential_sponsor_subject(subject)

    if kind == "submit":
        require_keys(subject, ("transaction_bytes",), "request.subject")
        return {"transaction_bytes": require_bytes(
            subject.get("transaction_bytes"), "request.subject.transaction_bytes"
        )}

    if kind == "fund_sponsor":
        require_keys(
            subject,
            ("sponsor_outputs", "amount_per_sponsor_output"),
            "request.subject",
        )
        outputs = require_int(
            subject.get("sponsor_outputs"), "request.subject.sponsor_outputs"
        )
        if outputs < 1 or outputs > 255:
            raise FatalAdapterError(
                "request.subject.sponsor_outputs is not a byte count of outputs"
            )
        return {
            "sponsor_outputs": outputs,
            "amount_per_sponsor_output": require_int(
                subject.get("amount_per_sponsor_output"),
                "request.subject.amount_per_sponsor_output",
            ),
        }

    if kind == "sign_sponsor":
        require_keys(
            subject,
            (
                "finalized_transaction",
                "sponsor_input_index",
                "sponsor_outpoint",
                "sighash_profile",
            ),
            "request.subject",
        )
        profile = subject.get("sighash_profile")
        # The one profile the protocol names. An adapter that quietly
        # signed under another would produce an authorization protecting
        # a different output set than the caller asked it to.
        if profile != "all_inputs_all_outputs":
            raise FatalAdapterError(
                "request.subject.sighash_profile names a profile this adapter "
                "does not implement: %s" % profile
            )
        outpoint = require_object(
            subject.get("sponsor_outpoint"), "request.subject.sponsor_outpoint"
        )
        require_keys(outpoint, ("txid", "vout"), "request.subject.sponsor_outpoint")
        txid = outpoint.get("txid")
        if not isinstance(txid, str):
            raise FatalAdapterError(
                "request.subject.sponsor_outpoint.txid is not a string"
            )
        return {
            "finalized_transaction": require_bytes(
                subject.get("finalized_transaction"),
                "request.subject.finalized_transaction",
            ),
            "sponsor_input_index": require_int(
                subject.get("sponsor_input_index"),
                "request.subject.sponsor_input_index",
            ),
            "sponsor_outpoint": {
                "txid": txid,
                "vout": require_int(
                    outpoint.get("vout"), "request.subject.sponsor_outpoint.vout"
                ),
            },
        }

    require_keys(
        subject,
        ("issue_asset", "asset", "output_program", "outputs", "amount_per_output"),
        "request.subject",
    )
    issue = subject.get("issue_asset")
    if not isinstance(issue, bool):
        raise FatalAdapterError("request.subject.issue_asset is not a boolean")
    asset = subject.get("asset")
    if asset is not None and not isinstance(asset, str):
        raise FatalAdapterError("request.subject.asset is not a string")
    # The one cross-member rule the protocol states: a step that issues
    # cannot also name the asset, because the target has not chosen it.
    if issue and asset is not None:
        raise FatalAdapterError("an issuing funding step also named an asset")
    if not issue and asset is None:
        raise FatalAdapterError("a non-issuing funding step named no asset")
    outputs = require_int(subject.get("outputs"), "request.subject.outputs")
    if outputs < 1 or outputs > 255:
        raise FatalAdapterError("request.subject.outputs is not a byte count of outputs")
    return {
        "issue_asset": issue,
        "asset": asset,
        "output_program": require_bytes(
            subject.get("output_program"), "request.subject.output_program"
        ),
        "outputs": outputs,
        "amount_per_output": require_int(
            subject.get("amount_per_output"), "request.subject.amount_per_output"
        ),
    }


def answer_operation_step(executor: CaseExecutor, request: dict, case: dict) -> None:
    """Answers exactly one Guide-12 section 16.2 operation step.

    # The three answers, and why they are not the same answer

    A step either did what it was asked, or the target refused it, or it
    never happened. The third is an `executor_infrastructure_failure` and
    is NOT a target verdict of any kind (section 1.5): a node that could
    not be reached, a reserve this run does not hold, an adapter that was
    handed a step it never advertised. Reporting one of those as a
    rejection would manufacture a consensus fact out of a broken
    environment, which is the failure the six-layer vocabulary exists to
    prevent.

    A funding step reports what it created and never an accepted
    identity; a submission reports an identity and never created outputs.
    The harness checks that separation on the way in, and this function
    produces records that satisfy it by construction rather than by
    remembering to.
    """
    require_execution_request_fields(request, ("schema", "case", "subject"))
    kind = case.get("operation")
    if kind not in (
        "fund",
        "submit",
        "fund_sponsor",
        "sign_sponsor",
        "fund_confidential",
        "fund_confidential_sponsor",
    ):
        raise FatalAdapterError("the harness sent an operation step of an unknown kind")
    subject = parse_operation_subject(request.get("subject"), kind)

    operations = executor.operations
    if operations is None:
        # Normally unreachable: without a wallet this adapter advertises
        # neither operation capability, so the harness refuses the step
        # before sending it. The branch stays so a mis-sent record gets a
        # typed answer rather than an exception.
        note = (
            "this adapter was booted without a wallet and advertised no "
            "operation capability"
        )
        log("executor infrastructure failure: %s" % note)
        write_operation_failure(case, note)
        return

    try:
        if kind == "fund":
            body = (
                operations.issue(subject)
                if subject["issue_asset"]
                else operations.pay(subject)
            )
            body.setdefault("accepted_txid", None)
        elif kind == "fund_sponsor":
            body = operations.fund_sponsor(subject)
        elif kind == "fund_confidential_sponsor":
            # The confidential arm of sponsor funding, and never a
            # fallback for the explicit one: it answers with the
            # confidential members and leaves the explicit ones empty,
            # because a coin whose value is committed has no amount for
            # the explicit members to carry.
            body = operations.fund_confidential_sponsor(subject)
        elif kind == "sign_sponsor":
            body = operations.sign_sponsor(subject)
        elif kind == "fund_confidential":
            # An explicit arm and never a fallback for it: the answer
            # carries the confidential members and leaves the explicit
            # ones empty, so a harness reading it cannot mistake one for
            # the other.
            body = operations.fund_confidential(subject)
        else:
            body = operations.submit(subject)
            body.setdefault("issued_asset", None)
            body.setdefault("funded_outputs", [])
    except AdapterError as error:
        # The adapter could not perform the step. That is not a statement
        # about the transaction and must not be recorded as one.
        log("executor infrastructure failure: %s" % error)
        write_operation_failure(case, str(error))
        return

    body.setdefault("observed_layer", "accepted")
    body.setdefault("observed_detail", None)
    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "observed_layer": body["observed_layer"],
            "observed_detail": body["observed_detail"],
            "issued_asset": body["issued_asset"],
            "funded_outputs": body["funded_outputs"],
            # Declared on every operation answer, never defaulted. The
            # harness's record does not default them either, which is
            # what makes revision 5 a revision: an adapter that omitted
            # them would be answering a revision-5 exchange with a
            # revision-4 record.
            "confidential_funded_outputs": body.get(
                "confidential_funded_outputs", []
            ),
            "mined_readback": body.get("mined_readback"),
            "accepted_txid": body["accepted_txid"],
            # Written only by the step that was asked for one. The
            # harness refuses a response carrying an authorization it
            # never requested, so the defaults are empty rather than
            # omitted-and-hoped-for.
            "sponsor_witness": body.get("sponsor_witness", []),
            "signature_bound_to": body.get("signature_bound_to"),
            # No INTERPRETER observation is made for an operation step:
            # the node exposes no per-script stack, so the peaks, the
            # widest element, and the validation budget stay null, and
            # inventing them here would be this adapter reporting its own
            # arithmetic as the target's accounting.
            #
            # The transaction's weight is the exception, and it is not an
            # exception to that rule. It is read back from the node's own
            # `decoderawtransaction` rather than computed here, so it is
            # the target's accounting; it is what
            # (´[PLAN-rule:guide12-exec:resource-comparison]´) compares
            # a prediction against; and it is written only on a target
            # verdict, because a response that is not one may carry no
            # observation at all (`G12-R14`).
            "resources": {
                "script_bytes": 0,
                "initial_stack_items": 0,
                "peak_stack_items": None,
                "peak_altstack_items": None,
                "maximum_element_bytes": None,
                "validation_budget_used": None,
                "transaction_weight": (
                    body.get("transaction_weight")
                    if body["observed_layer"] not in NON_VERDICT_LAYERS
                    else None
                ),
            },
        }
    )


def write_operation_failure(case: dict, note: str) -> None:
    """Reports a step that did not happen, with every observation empty.

    Which is what the harness's own shape check requires of one: an
    outpoint is a coin the target created, an issued asset is an identity
    the target chose, a transaction identity is one the target computed
    over bytes it accepted, and an authorization is one this adapter
    produced. None exists here.
    """
    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "observed_layer": "executor_infrastructure_failure",
            "observed_detail": note,
            "issued_asset": None,
            "funded_outputs": [],
            "confidential_funded_outputs": [],
            "mined_readback": None,
            "accepted_txid": None,
            "sponsor_witness": [],
            "signature_bound_to": None,
            "resources": {
                "script_bytes": 0,
                "initial_stack_items": 0,
                "peak_stack_items": None,
                "peak_altstack_items": None,
                "maximum_element_bytes": None,
                "validation_budget_used": None,
                "transaction_weight": None,
            },
        }
    )


def parse_arguments(argv):
    """Reads this adapter's own explicit, credential-free configuration."""
    parser = argparse.ArgumentParser(
        prog=COMMAND_NAME,
        description=(
            "Execute primitive and compound-prototype fixtures through a "
            "disposable Elements regtest node, speaking the conformance "
            "executor protocol on stdio."
        ),
    )
    # The two the HARNESS supplies. Mandatory, because this program writes
    # nothing to stderr and a run with nowhere to write its diagnostics
    # would be a run whose failures are unexplainable -- which is the state
    # the stderr contract was quietly in whenever the harness nulled it.
    parser.add_argument(
        "--output",
        required=True,
        help="file this adapter writes its own diagnostics to. Typed facts "
        "only: a method, an exit status, a phase, a failure class, a "
        "duration, a record number. No byte of any child process reaches it",
    )
    parser.add_argument(
        "--elements-output",
        required=True,
        help="file raw child text is quarantined in: what elements-cli wrote "
        "on its stderr, what the node answered, what this adapter "
        "serialized. Every entry carries a numbered header, and the number "
        "is what a typed line in --output names instead of quoting the text",
    )
    parser.add_argument("--elementsd", required=True, help="path to the elementsd binary")
    parser.add_argument("--elements-cli", required=True, help="path to the elements-cli binary")
    parser.add_argument(
        "--zk-library",
        default=None,
        help="path to the shared zero-knowledge library the deterministic "
        "confidential materializer calls. Optional: where it is not stated "
        "the adapter derives it from the node binary's own build tree, and "
        "where it cannot be reached at all the adapter advertises no "
        "confidential funding rather than a capability it could only refuse",
    )
    parser.add_argument(
        "--framework",
        required=True,
        help="path to the upstream functional-test framework (its test_framework "
        "package, or the directory holding it)",
    )
    parser.add_argument(
        "--network-id",
        required=True,
        help="the public development network identity of the chain this adapter "
        "boots, as 64 hex digits; restated as an observation only once the "
        "chain's genesis has been read from the node",
    )
    parser.add_argument(
        "--intended-executed-tip",
        default=None,
        help="the integration tip the operator intended to run (ADR-018); a "
        "declaration, never derived from a working tree",
    )
    parser.add_argument(
        "--upstream-base",
        default=None,
        help="the upstream base that tip derives from (ADR-018)",
    )
    parser.add_argument(
        "--included-local-topic",
        action="append",
        default=None,
        help="one local topic branch folded into that tip; repeatable (ADR-018)",
    )
    parser.add_argument(
        "--enable-wallet",
        action="store_true",
        help="boot the node with its wallet enabled and advertise the "
        "confidential-conservation capability. The wallet is created by this "
        "adapter on a disposable regtest chain and is test-fixture material "
        "under ADR-015; no wallet, seed, or key crosses this interface",
    )
    parser.add_argument(
        "--datadir",
        default=None,
        help="a chain directory the CALLER owns, used instead of the disposable "
        "one this adapter would create and never deleted by it. For the "
        "Guide-11 section 13 fresh-process lane, whose question is whether "
        "public evidence outlives the process that created it; no credential "
        "crosses this boundary, and the node's cookie still lives and dies "
        "inside the directory",
    )
    parser.add_argument(
        "--wallet-name",
        default=None,
        help="the name of the disposable wallet this process creates, so that "
        "two processes sharing one chain directory cannot share one wallet. "
        "A name, never a credential",
    )
    parser.add_argument("--chain", default="elementsregtest", help="the disposable chain name")
    parser.add_argument(
        "--boot-timeout-seconds", type=float, default=120.0, help="how long to wait for RPC"
    )
    return parser.parse_args(argv)


def open_diagnostics(arguments) -> DiagnosticStreams:
    """Opens the two destinations the harness named.

    Refused where they are the same destination. Two roles sharing one
    file would put raw child text into the typed stream by the shortest
    route available -- writing it there -- and a separation that a
    configuration mistake can undo is not a separation. The comparison is
    lexical normalization rather than filesystem identity, for the reason
    ADR-017's path-role rule gives: the host may replace or remount a path
    at any moment, so a device-and-inode answer would establish a boundary
    the repository cannot hold.

    Opened for append rather than truncation. The files belong to the run
    the harness named them for, and a harness that reuses one path for two
    phases of a run should get both phases rather than the second only.
    """
    if os.path.abspath(arguments.output) == os.path.abspath(arguments.elements_output):
        raise FatalAdapterError(
            "--output and --elements-output name one destination, and the "
            "two streams are separate or they are not streams"
        )
    output = open(arguments.output, "a", encoding="utf-8", errors="replace")
    try:
        elements_output = open(
            arguments.elements_output, "a", encoding="utf-8", errors="replace"
        )
    except OSError:
        output.close()
        raise
    return DiagnosticStreams(output, elements_output)


def report_framing_error(error: ProtocolFramingError) -> None:
    """Writes one fixed outcome for a recognized request-framing failure."""
    if error.phase not in ("handshake", "request"):
        report_fatal(FatalAdapterError(str(error)))
        return
    if error.failure == FRAMING_CLEAN_EOF:
        log(DiagnosticOutcome.FRAMING_CLEAN_EOF)
        return
    if error.failure == FRAMING_BLANK_RECORD:
        log(DiagnosticOutcome.FRAMING_BLANK_RECORD)
        return
    if error.failure == FRAMING_MALFORMED_RECORD:
        log(DiagnosticOutcome.FRAMING_MALFORMED_RECORD)
        return
    if error.failure == FRAMING_OVERSIZED_RECORD:
        log(DiagnosticOutcome.FRAMING_OVERSIZED_RECORD)
        return
    if error.failure == FRAMING_UNTERMINATED_RECORD:
        log(DiagnosticOutcome.FRAMING_UNTERMINATED_RECORD)
        return
    report_fatal(FatalAdapterError(str(error)))


def report_fatal(error: FatalAdapterError) -> None:
    """Writes one fatal outcome without interpolating exception text."""
    if error.diagnostic_written:
        return
    if error.outcome is not None:
        log(error.outcome, error.record)
        return
    detail = str(error)
    if detail.startswith("the handshake failed its field census; "):
        record = diagnostic_record(detail)
        if record is not None:
            log(DiagnosticOutcome.HANDSHAKE_FIELD_CENSUS_FAILED, record)
            return
    if detail == "the harness spoke a protocol revision this adapter does not":
        log(DiagnosticOutcome.PROTOCOL_REVISION_REFUSED)
        return
    log_quarantined(
        DiagnosticOutcome.FATAL_PROTOCOL_FAULT,
        "fatal protocol fault detail",
        detail,
    )


def main(argv) -> int:
    """Entry point.

    Nothing here writes to stderr, including the failure paths: a fatal
    condition is a typed line in the `--output` file and a nonzero status,
    and the status is what the harness reads. An exception escaping before
    the streams are open still reaches stderr as an interpreter traceback,
    which is the interpreter's doing and is stated in this module's own
    documentation rather than caught and hidden.
    """
    global STREAMS
    arguments = parse_arguments(argv)
    try:
        STREAMS = open_diagnostics(arguments)
    except (FatalAdapterError, OSError):
        # There is no destination, so there is nowhere honest to say so.
        # The refusal is the exit status and nothing else: a status is a
        # typed fact the harness already reads, and stderr stays empty
        # even here.
        return DIAGNOSTICS_UNAVAILABLE_STATUS
    try:
        return serve(arguments)
    except ProtocolFramingError as error:
        # One closed outcome and nothing from the refused record itself.
        report_framing_error(error)
        return 1
    except FatalAdapterError as error:
        report_fatal(error)
        return 1
    except json.JSONDecodeError:
        # Retained for the JSON this adapter decodes outside the request
        # framing -- the node's own answers. A malformed REQUEST is a
        # framing failure and is raised as one.
        log(DiagnosticOutcome.JSON_DECODE_FAILED)
        return 1
    finally:
        streams, STREAMS = STREAMS, None
        streams.output.close()
        streams.elements_output.close()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
