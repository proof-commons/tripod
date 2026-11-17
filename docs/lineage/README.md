# Keyed covenant realization

S5's implementation receivers are packages/tapscript/src/keyed_fixed_constructor.rs, packages/linker/src/keyed_fixed_bundle.rs and packages/vectors/src/keyed_fixed_constructor.rs. The independent byte/transcript/stack/commitment controls are packages/vectors/tests/keyed_fixed_template.rs and packages/tapscript/tests/keyed_fixed_bundle.rs. Source-freeze admission and S8's activation deadline are ([PLAN-rec:identities:keyed-fixed-capture-stop]); code and test existence does not establish a native result.

The [realization record](realization.md) specifies the public fixed-form recipe for the [version 2.1.0 paper](../../papers/lineage-fixed-form/main.tex), its conditional authentication, target envelope and derived floor. It distinguishes that contract from implemented and observed evidence, and preserves the legacy edition's scope.

These documents use the DOC owner and keyed-fixed area. The attestation realization and its RZ register remain separate. The recipe selects no deployment triple, capture availability or finality policy.
