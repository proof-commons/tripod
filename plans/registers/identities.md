# Identity and Hash Register · `reg:identities:ownership`

This register identifies the owner and boundary of each cross-package identity.
It is a planning aid, not a substitute for typed identity definitions.

| Identity | Owner | Binds | Does not replace |
|---|---|---|---|
| anchor-set hash | architecture/weld | imported anchor set | architecture semantics |
| architecture semantic hash | architecture | complete canonical architecture body | authenticity by itself |
| architecture behavioural hash | architecture | denotation projection | complete publication bytes |
| realization identity | realization | target-independent semantic graph | architecture identity |
| compiler configuration identity | compiler | analysis and policy choices | realization identity |
| analyzed-program identity | compiler | one normalized relation analysis | linked bundle |
| target-definition identity | target package | typed compatibility contract | node implementation revision |
| deployment-instance identity | target/release | network, genesis, activation/configuration | target-definition identity |
| backend configuration identity | backend | proof patterns and lowering policy | target identity |
| relocatable bundle identity | backend | unlinked target programs and relocations | linked deployment bytes |
| linked-bundle identity | linker | final programs, constructors, constants, bounds | deployment profile |
| transaction ABI identity | transaction | layouts, witnesses, metadata, bundle binding | semantic realization |
| vector-set identity | vectors | fixtures, mutations, coverage policy | execution report |
| evidence-report identity | report owner | exact claim, artifacts, tool, results | another evidence class |
| deployment-profile hash | architecture/release | deployment evidence profile | architecture hash |
| release identity | release | final manifest, assets, profile, policy | protocol denotation |

Implementation revisions remain review or test provenance unless a typed
compatibility contract explicitly makes another fact identity-relevant.