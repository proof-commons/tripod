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
`[ADR015-rule:security:test-material]`, and it remains the *executor's*
material: the first-party interface neither accepts it nor reads it.

## State

Implemented: the package boundary and the wire vocabulary that names
reviewed target identities in protocol and report data.

Not implemented: the canonical fixture census, the executor protocol
driver, and the report. No primitive fixture has been authored, so no
run can satisfy the Guide-9 evidence plan yet.
