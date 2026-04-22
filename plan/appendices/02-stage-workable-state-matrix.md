# Appendix 2 — Stage Workable-State Matrix

This appendix makes the plan's core promise concrete: **after every stage, the repository should still be usable by someone other than the person who just finished the work**.

Use this matrix when reviewing sequencing changes or deciding whether a stage has been split at the right level.

## Stage matrix

| Stage | Minimum usable state | Primary user of that state | Typical proof that the stage is really workable |
| --- | --- | --- | --- |
| 1. Workspace bootstrap | The workspace structure exists, toolchain setup is reproducible, and standard checks run locally and in CI | contributor/maintainer | A new contributor can clone the repo and run format, lint, and test commands successfully |
| 2. Domain types and backend contract | Shared types and backend contracts compile cleanly and can support at least a toy backend | backend author/application integrator | A mock backend can answer typed requests and rustdoc/examples show the minimal query flow |
| 3. Chart MVP and algorithmic baseline | A real chart workflow exists using pure-Rust algorithmic components plus baseline house/ayanamsa support | end user/application developer | A sample chart can be generated through `pleiades-core` or `pleiades-cli` with current limits documented |
| 4. Reference backend and validation | A source-backed backend and repeatable validation commands exist alongside the MVP workflow | maintainer/validator | Comparison reports and benchmark outputs can be regenerated from documented inputs |
| 5. Compression and packaged data | A packaged backend for 1500-2500 exists with reproducible artifacts and measured error bounds | application packager/deployer | Artifact generation, checksum verification, and packaged lookups all work on representative inputs |
| 6. Compatibility expansion and release hardening | Releases can be prepared with explicit compatibility coverage, validation evidence, and reproducible artifacts | release maintainer/downstream consumer | A release bundle can be assembled with compatibility profile, capability matrices, validation outputs, and documented commands |

## How to use this matrix

### When splitting work

If a proposed milestone does not leave one of the usable states above intact, split the milestone into smaller slices.

### When reviewing plan changes

Ask three questions:

1. does the stage still end in a usable state,
2. is that usable state simpler than the next stage's state,
3. does the transition preserve the architecture and purity constraints from the spec.

### When the repository is behind the plan

Prefer restoring the previous workable state before moving on. For example:

- if Stage 3 work exists but the contract from Stage 2 is still unstable, stabilize the contract first,
- if Stage 5 artifacts exist but there is no reproducible Stage 4 validation path, rebuild the validation story before expanding distribution,
- if Stage 6 compatibility claims outpace the published profile, update the compatibility profile before adding more breadth.

## Practical interpretation

A stage is not workable merely because it compiles. It is workable when:

- the intended primary user for that stage can do something useful,
- the workflow is documented enough for another maintainer to continue,
- known constraints are explicit,
- the next stage can treat the current one as a dependable base rather than a cleanup target.
