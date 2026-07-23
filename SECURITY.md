# Security Policy

## Current status

Tripod is a contract closure compiler under construction, together with the exemplar contract that proves it: an abstract specification (the *Attestation* paper), a typed architecture manifest, an executable reference model, a target-independent realization foundation, and the supporting build and documentation toolchain.

This repository does not currently contain a production target backend,
transaction signer, wallet, deployment release, or production key-management
component. It has **not reached a v1 release and is not deployed**. No current
commit should be treated as a production deployment release merely because the
architecture or model tests are final or green.

## Security model

Current first-party packages process public data only. They do not legitimately
accept private keys, passwords, tokens, signing nonces, blinding factors,
private openings, wallet credentials, or production deployment authority.

Repository source and build definitions are executable. Code from an untrusted
contribution must be run in an isolated environment without credentials,
production authority, repository write authority, or release authority. The
checkout does not sandbox itself.

`execwrap` is a process launcher and byte router, not a sandbox. Child output
relayed by it is not sanitized.

The detailed policy is [ADR-015](adr/015-public-data-and-execution-trust.md).

## What to report

Please report any defect that could cause one or more of the following:

- acceptance of an invalid semantic transition;
- unauthorized issuance, destruction, value transfer, recipient change, or
  state transition;
- disagreement among the specification, realization, typed architecture, and executable model;
- forged event, query, accounting, provenance, or deployment evidence;
- generated or release artifact substitution accepted as valid;
- bypass of architecture, behavioural-version, bundle, ABI, target, or
  deployment-profile identity checks;
- arbitrary file access or overwrite outside an explicit command destination;
- accidental exposure of an actual credential by a current first-party
  diagnostic path;
- a build or release path that silently weakens a required security relation.

Do not include private keys, credentials, wallet seeds, production blinders, or
other live secrets in a report. Revoke or rotate an exposed credential before
sending diagnostic material.

## How to report (pre-v1)

Because this project has not reached v1 and nothing is deployed, there are no
funds, signing authority, or release integrity at risk, and there is no
coordinated-disclosure window to protect. **Report security issues in the
open**, the same way you would report any other correctness defect — through
the ordinary public issue or discussion mechanism of wherever this repository
is hosted. A public report is appropriate at this stage precisely because the
threat is to specification and model *correctness*, not to a live system.

This will change as the project approaches a deployed release. When a
production target, signer, or release path exists, this policy will be revised
to establish a monitored private reporting channel and a coordinated-disclosure
process before any such component is accepted (see the future-secret and
production-authority rules of ADR-015).

## Out of scope for current packages

The following are not current application-level secret-handling defects:

- a caller deliberately placing a secret in a public path or argument;
- malicious code stealing credentials from an environment that exposed them;
- a core dump containing ambient environment values;
- compromise of an operating system, kernel, hypervisor, compiler, or hardware;
- production key custody, because no current package implements it.

These may still be important environment, supply-chain, or future deployment
risks. They are not claims that the current public-data packages manage those
secrets.

## Disclosure and response

Until v1, security-relevant defects are triaged in the open alongside ordinary
correctness work: the maintainers establish the affected scope, avoid
reproducing any real secret value that a reporter mistakenly includes, and
remediate according to severity. A private, coordinated process is established
before, not after, the project gains a deployed component that could put funds
or authority at risk.
