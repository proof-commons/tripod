# ADR-015: Public-Data Tools and the Execution-Environment Trust Boundary

**Status:** Proposed
**Scope:** Current first-party libraries, binaries, tests, build definitions,
documentation tooling, generated publications, and release preparation
**Amends:** the interpretation of ADR-010 diagnostic classification and
ADR-011 build/reproducibility policy
**Does not establish:** production key custody, a sandbox, or deployment
security

---

## Context · `sec:security:context`

This repository currently contains specifications, typed architecture,
target-independent realization work, an executable reference model, generated
publications, documentation tooling, and build support.

Current first-party packages do not implement production signing, wallet
custody, confidential transaction key management, node credential management,
or production deployment authorization.

The repository nevertheless executes source-controlled code through Rust,
Cargo, Meson, shell, Python, TeX, Biber, latexmk, and selected child
executables. A checkout therefore cannot establish a security boundary around
itself: untrusted source executed with a developer or CI account normally has
that account's operating-system authority.

The repository must distinguish:

1. honest-tool output hygiene and source-tree correctness, which first-party
   code can govern; and
2. containment of malicious or untrusted executable code, which belongs to the
   execution environment.

## Public-data contract · `rule:security:public-data`

Current first-party package interfaces are public-data interfaces.

Their legitimate inputs include:

- repository source and documentation;
- public Git metadata;
- public paths, identifiers, hashes, and UUIDs;
- typed architecture and realization values;
- transparent model worlds and synthetic test identities;
- generated public artifacts;
- public child command configuration and deliberately relayed child output.

Their legitimate outputs include:

- JSON results and diagnostics;
- build reports and stamps;
- generated manifests and registers;
- flattened LaTeX and rendered publications;
- model and test results;
- public hashes, identities, and evidence metadata.

A current first-party command does not legitimately accept:

- credentials;
- private keys or seed material;
- signing nonces;
- passwords or bearer tokens;
- node or cloud credentials;
- production blinding factors;
- private commitment openings;
- wallet-private state;
- release-signing or deployment-authority secrets.

Supplying such a value through a public path, URL, argument, source field, model
identifier, or child command violates the interface contract.

## Public model state · `rule:security:model-data`

The executable model is a transparent public-data semantic model.

Model values such as `OwnerKey`, `SignerSet`, `AttestationAddress`, `World`,
`Utxo`, and `TransitionCertificate` are public semantic abstractions.

`OwnerKey` is not private key material. `SignerSet` records abstract
authorization membership; it contains neither signatures nor signing keys.

Callers must not place private key bytes, seed material, signing nonces,
blinding factors, private openings, credentials, or other secrets in model
identifiers or model state.

The executable model is evidence about abstract behavior. It is not a deployed
application, wallet, signer, or target program.

## Diagnostic boundary · `rule:security:diagnostics`

First-party result data, ordinary diagnostics, debug diagnostics, test output,
and panic records are public-data outputs under the current interface contract.

Diagnostic safety follows this order:

1. use safe typed public fields;
2. transform understood structures narrowly, such as removing URL userinfo or
   credential-bearing query fields;
3. omit arbitrary or classified values wholesale;
4. use best-effort free-form redaction only as defense in depth.

Current first-party diagnostics omit:

- raw argv;
- ambient environment values;
- fields explicitly classified as credentials or secret material;
- arbitrary stderr from an argument-supplied external executable;
- panic payloads by default.

Free-form redaction is not a completeness claim and is not a security
boundary.

Canonical results and generated publications are validated against their typed
schemas. They are not heuristically scrubbed, because redaction would change
their semantics.

Debug mode may emit additional public input and internal control-flow detail.
It does not make the command a safe transport for out-of-contract secret
material.

## Child execution · `rule:security:child-execution`

An executable path is an execution capability, not ordinary untrusted data.

A caller selecting:

- the Git executable;
- an `execwrap` child;
- a TeX tool;
- another build helper

is authorizing that executable to run with the authority provided by the
operating-system environment.

`execwrap` routes child bytes. It does not:

- authenticate the child;
- sandbox the child;
- classify child arguments;
- remove secrets from child memory;
- sanitize deliberately relayed child output;
- restrict filesystem or network access.

Raw child argv remains omitted from wrapper diagnostics. Deliberately relayed
child stdout or stderr remains unsanitized child result data.

## Untrusted-source execution · `rule:security:untrusted-source`

Repository source is executable through multiple build paths.

An external contribution may modify:

- Rust code and tests;
- Cargo manifests and dependency selection;
- Meson definitions;
- shell and Python scripts;
- LaTeX sources;
- `.latexmkrc`;
- build and test behavior.

Untrusted source must therefore be executed in an environment established
outside the untrusted checkout.

The environment, not a script supplied by the untrusted branch, owns:

- credential removal;
- filesystem and process isolation;
- network restrictions;
- read-only source mounts where desired;
- writable build and temporary directories;
- agent-socket exclusion;
- worker lifetime and destruction;
- core-dump and crash-artifact policy.

An untrusted contribution environment receives no production secrets,
production signing authority, deployment credentials, repository write
authority, or release authority.

## Build write boundary · `rule:security:build-writes`

Non-writing is a correctness and reproducibility property, not a malicious-code
sandbox.

Ordinary check mode does not repair or modify tracked source or committed
generated publications.

Builds may write declared build products, reports, stamps, caches, temporary
files, and explicitly configured publication mirrors.

Generation and publication are explicit side effects and write only their
declared or argument-supplied destinations.

A clean-tree check detects honest-tool source mutation. It does not prove that
malicious code lacked write access or did not restore a file before the final
check.

An execution environment may additionally enforce a read-only source mount.
That enforcement is outside the checkout.

## Crash artifacts · `rule:security:crash-artifacts`

Core dumps, minidumps, heap snapshots, debugger memory captures, sanitizer
memory reports, and crash bundles are not ordinary command diagnostics.

They may contain:

- argv and environment values;
- stack and heap contents;
- source being processed;
- formatting and parser buffers;
- child arguments and child output;
- any data read by the host process.

Current first-party packages have no legitimate production secret input, so
they require no model- or package-specific dump scrubbing.

Crash artifacts are never considered declassified by heuristic scanning or
post-hoc scrubbing. An environment that retains one treats it as sensitive
until reviewed and does not publish it automatically.

A future process that legitimately handles secret material must define and
enforce its own dump, debugger, memory, and incident-response policy.

## Dependency and platform trust · `rule:security:trusted-computing-base`

The build and release trusted computing base includes, as applicable:

- operating system and kernel;
- hypervisor or container runtime;
- hardware;
- Rust compiler and Cargo;
- Git, Meson, Ninja, Python, shell, TeX, Biber, and latexmk;
- first-party and third-party dependencies;
- CI and artifact infrastructure.

The repository does not claim to audit this entire stack.

Locked dependency resolution, dependency review, first-party unsafe-code
policy, deterministic builds, independent evidence, and reproducible artifacts
reduce the trust placed in any one component. They do not eliminate
supply-chain or platform compromise.

A release claim must name the remaining trusted computing base and residual
assumptions honestly.

## Production signing and deployment · `rule:security:production-authority`

Ordinary development, test, document, model, CI, and release-build processes
do not receive production private keys, production blinding material, node
credentials, or release-signing authority.

Release builders produce unsigned candidates.

Production signing or deployment authorization belongs to a separate
operational boundary that verifies the exact applicable:

- source revision;
- architecture and realization identity;
- target and deployment identity;
- linked bundle;
- transaction ABI;
- calibrated bounds;
- evidence reports;
- finalized transaction or release digest.

Private authority should remain behind an external signer, hardware-backed
signer, HSM, or another separately reviewed capability boundary where
practical.

## Future secret-bearing components · `rule:security:future-secrets`

A package or executable introducing a legitimate secret input requires a new
reviewed security design before the interface is accepted.

The design must state:

- secret type and owner;
- attacker and trust model;
- input channel;
- process and child-execution boundary;
- logging, panic, and error behavior;
- memory and core-dump policy;
- serialization and report exclusion;
- randomness and nonce policy;
- signing or external-authority boundary;
- incident response and rotation;
- focused positive and negative tests.

Secret values must not be introduced through command-line arguments.

No current public-data package implicitly becomes a secret-processing package.

## Secret scanning · `rule:security:secret-scanning`

A maintained external secret scanner may be used as a bounded detective control
for common accidentally committed credentials.

Such scanning:

- is not a first-party semantic package;
- does not prove arbitrary source or history is secret-free;
- does not protect ambient environment credentials;
- does not declassify core dumps or arbitrary binary artifacts;
- does not contain malicious executed code;
- does not replace credential rotation after exposure.

Scanner findings are reported without reproducing the matched secret.

## Security priorities · `rule:security:priorities`

For the current repository, security review prioritizes:

1. semantic and authorization correctness;
2. architecture/model/realization agreement;
3. target translation and relation coverage;
4. artifact provenance, identity, and reproducibility;
5. independent evidence and deployment-profile correctness;
6. build and diagnostic contract integrity;
7. incidental public-data output hygiene.

Current secret custody is not a repository capability and must not be described
as one.

## Non-claims · `sec:security:non-claims`

This policy does not claim that:

- a checkout sandboxes itself;
- `execwrap` contains a child process;
- redaction detects every possible secret;
- a clean-tree check contains malicious code;
- pinned dependency resolution proves dependency trustworthiness;
- one reproducible build proves an uncompromised toolchain;
- model authorization proves cryptographic signing;
- architecture finality implies deployment readiness;
- a core dump can be made public through heuristic scrubbing;
- current packages are suitable for production key custody.

## Verification · `gate:security:verification`

This policy is implemented for the current repository when:

- current first-party CLI schemas contain no legitimate secret-valued argument;
- package documentation identifies public abstract owner/signing values;
- raw argv and ambient environment values are absent from first-party
  diagnostics;
- argument-supplied external stderr is not logged as a first-party diagnostic;
- `execwrap` documents its non-sandbox and unsanitized-relay boundary;
- tests and model fixtures remain synthetic/public;
- untrusted CI receives no production credentials or release authority;
- crash artifacts are not automatically published by repository-controlled CI;
- ordinary checks remain non-writing with respect to tracked source;
- future secret-bearing interfaces cannot enter without a separate review;
- repository and build checks remain green and clean.
