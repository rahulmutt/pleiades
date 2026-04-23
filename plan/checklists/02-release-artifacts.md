# Checklist 2 — Release Artifacts and Maintainer Outputs

This checklist captures the non-code outputs that should exist as the project matures.

It complements the stage documents by making release expectations explicit.

## Always expected once the relevant subsystem exists

### Developer environment and workflow

- repository-managed tool definitions (`mise.toml`, and `devenv.nix` only when justified)
- documented build, lint, test, and validation commands
- CI configuration aligned with documented local workflows

### Public-facing compatibility and capability documentation

- release compatibility profile listing built-in house systems, ayanamsas, aliases, validation reference points, intentional custom-definition labels, and compatibility caveats with those categories kept distinct
- release notes summarizing the current coverage additions and remaining limitations
- backend capability matrices listing body coverage, ranges, modes, and expected accuracy class
- user-visible notes about unsupported features or documented failure modes

### Validation outputs

- benchmark summaries for representative workloads
- comparison reports for implemented backends
- archived regression cases for previously fixed numerical or interoperability issues
- API stability summaries for quick inspection of the current façade posture
- compact summaries for other mature release artifacts when they materially improve maintainer auditability
- evidence for published accuracy claims

## Additional outputs once packaged data exists

- artifact format version identifier
- artifact checksums
- source provenance and generation metadata
- measured error summaries against generation sources
- reproducible generation command or pipeline documentation
- verification that staged bundles reject missing provenance fields, unexpected extra files, and tampered artifact contents

## Suggested ownership by stage

| Stage | New artifacts that should begin to exist |
| --- | --- |
| 1 | toolchain docs, CI workflows, crate responsibility docs |
| 2 | rustdoc examples, public type semantics, mock-backend examples |
| 3 | compatibility profile v0, chart examples, CLI snapshots or sample reports |
| 4 | backend capability matrices, comparison reports, benchmark baselines |
| 5 | artifact metadata, checksums, error envelopes, regeneration docs |
| 6 | release-grade compatibility profile, archived validation bundles, stable release checklist |

## Minimum release bundle for a mature release

A mature release should be accompanied by:

- source tag or versioned release commit
- compatibility profile for that release
- release notes derived from the published profile and release-specific coverage
- release checklist or similar maintainer-facing release gate summary
- capability matrix for each shipped backend
- API stability summary alongside the full API stability posture
- validation report bundle or links to archived reports
- packaged artifact metadata, checksums, and provenance when artifacts are shipped
- changelog or release notes describing new coverage and known limitations
- compact summary views for the compatibility profile, backend matrix, API stability posture, validation report, and packaged artifact status when those mature reports are part of the shipped release surface

## Maintenance rule

If the code adds a new backend, catalog entry family, validation command, or distributable artifact, update the corresponding release-artifact expectations in this checklist or the relevant stage/track doc during the same change.
