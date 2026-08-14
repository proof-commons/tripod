# tripod-target-elements-conformance

The secretless target-native conformance harness for the reviewed
Elements tapscript primitives.

## Purpose

Secretless execution of generic target primitive fixtures through an
external Elements executor, and the typed report of what that executor
observed.

## Inputs

- the reviewed static target contract;
- the development deployment binding;
- public generic fixtures — script bytes, initial stack, transaction
  context, and expected outcome;
- an explicit external executor capability (a program path the caller
  selects);
- explicit report and stamp destinations.

## Outputs

- a typed native-conformance report;
- optionally, that report as an explicit build asset plus its success
  stamp (ADR-010, ADR-014).

## Not claimed

- executor authenticity — the handshake is provenance, not identity, and
  an executor may misdescribe itself;
- implementation independence;
- production activation;
- backend correctness;
- release evidence identity — no report digest is minted, and none may
  be added here.

## Secrets

None. No interface accepts an RPC username, password, bearer token,
cookie path, private key, signing nonce, production blinding factor,
private opening, wallet path, or production endpoint. An executor that
must authenticate to a node owns that boundary outside this process
(ADR-015).

Disposable test-network material — regtest keys and a cookie confined to
a throwaway data directory — is public fixture data under
(`[ADR015-rule:security:test-material]`), and it remains the *executor's*
material: the first-party interface neither accepts it nor reads it.

## State

Implemented: the package boundary, the wire vocabulary, the generic
fixture language, the secretless executor protocol, the external
executor driver with its explicit typed timeout, the typed
native-conformance report, the Guide-9 evidence plan, the comparison,
the gate, and the ADR-010 checker command with its non-default Meson
lane.

Also implemented: the canonical primitive census. It covers every
reviewed primitive, and every required evidence row has cases bearing on
it.

What a case can establish is bounded by what a validating node reports.
It answers whether a spend was valid and, coarsely, why not; it exposes
no interpreter stack, and it reports one reason for several reviewed
causes. Expectations are shaped accordingly: a verdict, the failure
classes the contract admits, and the exact stacks as static statements
that are compared only when an executor reports one.

The reviewed domain requires evaluation to finish with exactly one true
item, and the reviewed primitive census has no equality, drop, or verify
primitive to reduce a deeper stack with. Several primitives therefore
have no reachable accepting case, and their cases establish the number
of items the primitive pushed instead.

Deliberately not covered, and recorded as residuals rather than filled
in with cases that cannot run: a signature over a transaction sighash,
blinded assets, amounts, and nonces, issuing inputs, an absent
introspection context, and any execution domain other than the reviewed
one.

## The native lane is not part of ordinary CI

The Meson target `target-elements-native-check` is non-default and is
defined only when `-Dtarget_native_executor=` names an executor. With no
executor configured the target does not exist, nothing runs it, and
ordinary CI claims no target-native evidence — a skipped native lane
leaves the Guide-9 evidence incomplete even when every other lane is
green.
