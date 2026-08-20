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
This adapter speaks protocol revision 3, in which a request carries the
execution subject and nothing about what the result should be: no expected
verdict, no expected failure class, no expected final stack, no expected
resource figure, no claim set, and no evidence class. Earlier revisions sent
the whole fixture and required an executor to discard the expectation before
executing, and this adapter did; revision 3 makes the discipline unnecessary
by removing the field, and the two revisions are refused for each other at
the handshake rather than reconciled.

No-arguments-from-harness contract
----------------------------------
The conformance harness spawns the selected executor with no arguments at all
and never reads its stderr (Guide-9 section 11.1, section 11.2). This script
therefore has to be reached through a launcher that supplies its
configuration; `elements-native-executor.sh` beside it is that launcher. The
configuration this script requires -- the `elementsd` and `elements-cli`
paths, and the `--framework` path -- is the executor's own, established
outside the first-party interface, and carries no credential of any kind
(ADR-015, and Guide-9 section 17.6, which bans `--rpc-user`, `--rpc-password`,
`--cookie`, `--token`, `--wallet`, and `--private-key`).

Everything this script writes on stdout is protocol data. Diagnostics go to
stderr, which the harness nulls.

The boundary runs in both directions, and the second direction is the one
that was missing. The harness does not read this process's stderr; this
process likewise does not read its OWN children's stderr into anything that
reaches a first-party record. The `elements-cli` child's stderr used to be
collapsed into an `AdapterError` note, and that note is written to protocol
records as `observed_detail` and `detail` -- so child bytes arrived in
first-party evidence by the back door while the contract above appeared to
forbid exactly that. A note now states the method and the client's exit
status, both of them fixed and typed, and says that the reason is omitted;
the reason itself is logged here, on the stderr the harness nulls (G12-R04).

For the same reason no note interpolates a configuration path. Those paths
are the operator's argv, not the target's answer, and a first-party record
is not where an operator's directory layout belongs.

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
import io
import json
import os
import re
import shutil
import signal
import socket
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
NATIVE_PROTOCOL_SCHEMA = 4

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
# (`tab:elements-ref:ct-sighash`). A key-path spend of this address type
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


class FatalAdapterError(Exception):
    """A condition that ends the exchange rather than one case."""


def log(message: str) -> None:
    """Writes one diagnostic line to stderr, which the harness nulls."""
    print("%s: %s" % (COMMAND_NAME, message), file=sys.stderr, flush=True)


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
            # what the no-arguments-from-harness contract above says
            # does not happen. The contract described the harness
            # reading THIS process's stderr; it did not describe this
            # path, and the same bytes were reaching the same place by
            # the back door (G12-R04).
            #
            # What is left is fixed and typed: the method, which the
            # harness itself named, and the client's exit status. The
            # omission is stated rather than silent, because a
            # diagnostic that quietly dropped the reason would be less
            # honest than one that says where the reason went -- it is
            # on this adapter's own stderr, which the harness nulls.
            log(
                "rpc %s failed with status %d, and the client said: %s"
                % (method, completed.returncode, one_line(completed.stderr))
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
    """Collapses a client's message to one line for a stderr diagnostic."""
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
    return candidate, messages, script


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


class CaseExecutor:
    """Executes fixtures against one disposable node."""

    def __init__(self, node: DisposableNode, messages, script) -> None:
        self.node = node
        self.messages = messages
        self.script = script
        self.internal_key = bytes.fromhex(NUMS_INTERNAL_KEY_HEX)
        self.anyone_can_spend = bytes.fromhex(ANYONE_CAN_SPEND_HEX)
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
            # message goes to the stderr the harness nulls (G12-R04).
            log(
                "the framework raised %s building a taproot commitment: %s"
                % (type(error).__name__, one_line(str(error)))
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
        transaction.vout.append(executor.output(working, executor.anyone_can_spend))
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

    def with_witness_section(self, transaction) -> str:
        """Serializes a transaction with an explicit, empty witness.

        # Why an issuance cannot be serialized without one

        The framework writes a witness section only when some entry is
        non-null, and every entry of an explicit issuance IS null: no
        range proof, no inflation-keys proof, no script witness. So the
        transaction went to the target with no witness section at all,
        and the target does not count an issuance whose witness is
        absent -- the issued asset then has outputs and no inputs, which
        it reports as `bad-txns-in-ne-out` on a transaction whose every
        amount balances.

        That was observed rather than reasoned: the target refused its
        own `rawissueasset` output, unmodified, for the same rule.

        So the section is written by hand: the flag byte is set and one
        empty entry is emitted per input and per output, which is what a
        node's own serializer produces for the same transaction.
        """
        transaction.wit.vtxinwit = [messages.CTxInWitness() for _ in transaction.vin]
        transaction.wit.vtxoutwit = [messages.CTxOutWitness() for _ in transaction.vout]
        raw = transaction.version.to_bytes(4, "little")
        raw += b"\x01"
        raw += messages.ser_vector(transaction.vin)
        raw += messages.ser_vector(transaction.vout)
        raw += transaction.nLockTime.to_bytes(4, "little")
        raw += transaction.wit.serialize()
        return raw.hex()

    def mine(self, transaction, note: str = "", raw=None) -> str:
        """Confirms one transaction by mining exactly it.

        A refusal here is the adapter failing to build something the
        chain accepts, which is an infrastructure failure and never a
        target verdict. The transaction this adapter built is logged to
        stderr so the failure can be diagnosed; stderr is diagnostics the
        harness discards, and nothing from it reaches a first-party
        record (G12-R04).
        """
        raw = transaction.serialize().hex() if raw is None else raw
        try:
            self.executor.node.call(
                "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
            )
        except AdapterError:
            log("the %s transaction this adapter built was refused: %s" % (note, raw))
            # The mempool's structured reason, which names the rule; the
            # block error names only the check that reported it.
            try:
                log(
                    "the mempool says: %s"
                    % json.dumps(
                        self.executor.node.call("testmempoolaccept", json.dumps([raw]))
                    )
                )
            except AdapterError:
                pass
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
        base_hex = self.with_witness_section(base)

        # What the chain says the coin this transaction spends is worth,
        # rather than what this adapter believes. Diagnostics on stderr;
        # nothing from here reaches a first-party record.
        held = node.call("gettxout", source["txid"], str(source["vout"]))
        log(
            "issuance spends %s:%d which this adapter believes holds %d and the "
            "chain reports as %s"
            % (source["txid"], source["vout"], source["amount"], json.dumps(held))
        )
        address = node.call("getnewaddress", wallet=self.wallet_name)
        answer = node.call(
            "rawissueasset",
            base_hex,
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
        log(
            "unmodified issuance acceptance: %s"
            % json.dumps(
                node.call("testmempoolaccept", json.dumps([issuance["hex"]]))
            )
        )
        issued = messages.CTransaction()
        issued.deserialize(io.BytesIO(bytes.fromhex(issuance["hex"])))
        log(
            "round trip equal: %s"
            % (issued.serialize().hex() == issuance["hex"])
        )
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
        # An Elements transaction carries one witness entry per output as
        # well as per input, and this transaction was DESERIALIZED with a
        # different output count than it now has. The serializer writes
        # exactly as many entries as the list holds and the deserializer
        # reads exactly as many as there are outputs, so a stale list
        # writes a short witness section and the target reads the
        # following bytes as the missing entry -- which decodes into a
        # transaction that is not the one built here. The observed
        # symptom was `bad-txns-in-ne-out` on a transaction that balances
        # exactly, which is what a misparse looks like from the far side.
        issued.wit.vtxinwit = [messages.CTxInWitness() for _ in issued.vin]
        issued.wit.vtxoutwit = [messages.CTxOutWitness() for _ in issued.vout]

        txid = self.mine(issued, "issuance", self.with_witness_section(issued))
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
            raise AdapterError(
                "the step asks for more of the asset than this run holds"
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
                self.executor.node.call(
                    "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
                )
            except AdapterError as error:
                return self.refused_at_consensus(error)
            return {
                "observed_layer": "accepted",
                "observed_detail": None,
                "accepted_txid": result.get("txid"),
            }

        reason = result.get("reject-reason")
        if not isinstance(reason, str):
            raise AdapterError("the node rejected without naming a reason")
        for prefix in (POLICY_SCRIPT_PREFIX, CONSENSUS_SCRIPT_PREFIX):
            if script_error_in(reason, prefix) is not None:
                return {
                    "observed_layer": "script_path_rejection",
                    "observed_detail": reason,
                    "accepted_txid": None,
                }

        # Not a script verdict. Ask consensus directly: a transaction a
        # block accepts was refused by standardness, and one a block also
        # refuses was refused before any script ran.
        try:
            self.executor.node.call(
                "generateblock", "raw(%s)" % ANYONE_CAN_SPEND_HEX, json.dumps([raw])
            )
        except AdapterError as error:
            return self.refused_at_consensus(error, mempool_reason=reason)
        return {
            "observed_layer": "relay_policy_rejection",
            "observed_detail": reason,
            "accepted_txid": None,
        }

    def refused_at_consensus(self, error, mempool_reason=None) -> dict:
        """Classifies a block-validation refusal.

        The client's stderr is read for CLASSIFICATION only. What leaves
        this method is either the mempool's own structured reason, which
        is the target speaking through an RPC answer, or nothing --
        never the child's text (G12-R04).
        """
        detail = mempool_reason
        if script_error_in(error.client_detail, CONSENSUS_SCRIPT_PREFIX) is not None:
            return {
                "observed_layer": "script_path_rejection",
                "observed_detail": detail,
                "accepted_txid": None,
            }
        return {
            "observed_layer": "consensus_rejection_before_script",
            "observed_detail": detail,
            "accepted_txid": None,
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
          relay names a mandatory script err  -> script-path rejection
          relay names any other consensus
            reason, and a block also refuses  -> consensus rejection
                                                 before script
          relay refuses, a block takes it     -> relay-policy rejection

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


def rejection(script_error: str) -> dict:
    """Builds a rejection body, naming any script error the table cannot map."""
    text = script_error.strip()
    failure = FAILURE_CLASS_BY_SCRIPT_ERROR.get(text)
    if failure is None:
        log("rejected, with a script error this adapter does not classify: %s" % text)
    return {"verdict": "rejected", "observed_failure": failure}


# --------------------------------------------------------------------------
# Protocol
# --------------------------------------------------------------------------


def write_message(value: dict) -> None:
    """Writes one NDJSON protocol line on stdout."""
    sys.stdout.write(json.dumps(value, separators=(",", ":")) + "\n")
    sys.stdout.flush()


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


def serve(arguments) -> int:
    """Runs the whole exchange, and destroys the node whatever happens."""
    framework_path, messages, script = load_framework(arguments.framework)
    log("framework loaded from %s" % framework_path)
    name, version, revision = node_provenance(arguments.elementsd)
    network_id = identifier(arguments.network_id, "network")
    topics = [topic for topic in (arguments.included_local_topic or []) if topic]

    request_line = sys.stdin.readline()
    if request_line == "":
        raise FatalAdapterError("the harness closed stdin before the handshake")
    handshake = json.loads(request_line)
    if not isinstance(handshake, dict) or handshake.get("schema") != NATIVE_PROTOCOL_SCHEMA:
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
        executor = CaseExecutor(node, messages, script)
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

        for line in sys.stdin:
            if line.strip() == "":
                continue
            answer_case(executor, line)
    finally:
        close()
    return 0


def answer_case(executor: CaseExecutor, line: str) -> None:
    """Answers exactly one execution request, primitive or compound.

    The two records are told apart by the case identity they carry, which
    is the one field whose shape differs between them: a primitive case is
    a group and an ordinal, and a compound one is a relation and a name.
    Reading the fixture first and inferring the record from which fields
    parsed would mean deciding what was asked from what happened to be
    readable.
    """
    request = json.loads(line)
    if not isinstance(request, dict):
        raise FatalAdapterError("the harness sent a request that is not an object")
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
    for key in request:
        if key not in allowed:
            raise FatalAdapterError("the harness sent a request field named %s" % key)
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
    for key in request:
        if key not in ("schema", "case", "subject"):
            raise FatalAdapterError("the harness sent a request field named %s" % key)

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
    for key in request:
        if key not in ("schema", "case", "subject"):
            raise FatalAdapterError("the harness sent a request field named %s" % key)

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
    for key in request:
        if key not in ("schema", "case", "subject"):
            raise FatalAdapterError("the harness sent a request field named %s" % key)

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


def parse_operation_subject(raw: object, kind: str) -> dict:
    """Reads one operation subject, refusing anything it does not define.

    The two subjects share no member, which is what lets the kind stated
    in the case identity decide which one is admitted: a record carrying
    the other kind's members is a request whose two halves disagree, and
    is refused here rather than answered by whichever half parsed.
    """
    subject = require_object(raw, "request.subject")
    if kind == "submit":
        require_keys(subject, ("transaction_bytes",), "request.subject")
        return {"transaction_bytes": require_bytes(
            subject.get("transaction_bytes"), "request.subject.transaction_bytes"
        )}

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
    for key in request:
        if key not in ("schema", "case", "subject"):
            raise FatalAdapterError("the harness sent a request field named %s" % key)
    kind = case.get("operation")
    if kind not in ("fund", "submit"):
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
            "accepted_txid": body["accepted_txid"],
            # No interpreter observation is made for an operation step.
            # The node reports no per-script resource figures for a
            # transaction it validated as a whole, and inventing them
            # here would be this adapter reporting its own arithmetic as
            # the target's accounting.
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


def write_operation_failure(case: dict, note: str) -> None:
    """Reports a step that did not happen, with every observation empty.

    Which is what the harness's own shape check requires of one: an
    outpoint is a coin the target created, an issued asset is an identity
    the target chose, and a transaction identity is one the target
    computed over bytes it accepted. None exists here.
    """
    write_message(
        {
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": case,
            "observed_layer": "executor_infrastructure_failure",
            "observed_detail": note,
            "issued_asset": None,
            "funded_outputs": [],
            "accepted_txid": None,
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
    parser.add_argument("--elementsd", required=True, help="path to the elementsd binary")
    parser.add_argument("--elements-cli", required=True, help="path to the elements-cli binary")
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


def main(argv) -> int:
    """Entry point."""
    arguments = parse_arguments(argv)
    try:
        return serve(arguments)
    except FatalAdapterError as error:
        log("fatal: %s" % error)
        return 1
    except json.JSONDecodeError:
        log("fatal: the harness sent a line that is not JSON")
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
