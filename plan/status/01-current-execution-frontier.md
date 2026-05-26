# Status 1 — Current Execution Frontier

## Frontier

Phase 1, production reference backend and corpus, is the active frontier.

The repository has strong scaffolding around corpus summaries, backend matrices,
comparison reports, release bundle rehearsal, and CLI/validation inspection.
The JPL source-corpus contract is now explicit about reference/hold-out
provenance, source windows, source revisions, exact J2000 fixture evidence, and
boundary-request corpora, and the exact J2000 slice now also exposes an
explicit major-body/selected-asteroid class split in the source-corpus report.
The source-corpus summary now also surfaces the merged production-generation
body-class coverage and cadence split, and the production-generation source
summary now carries explicit source-density floors alongside the exact J2000
fixture-evidence payload, the source window payload, and an explicit source-class
breakdown across reference, hold-out, boundary, and provenance-only rows. Recent
source-corpus cleanup also corrected the selected-asteroid Apophis rows at J2000
and the early-2001 boundary samples, and regression tests now pin those Horizons
values directly. The production-generation manifest summary continues to validate
the derived source, coverage, and boundary request corpus records directly. The
checked-in JPL-style snapshots now also expose reusable pure-Rust CSV parsing
entry points for their manifest and row data, plus combined and split-source
corpus loaders for arbitrary JPL-style CSV text. The release-grade body-claims and
body/date/channel posture now assemble from structured body lists and validated
corpus evidence instead of a single hand-written prose string. The remaining
blocker is still the underlying production-grade reference input strategy:
current JPL evidence is a checked-in snapshot/hold-out fixture set, not a broad
public-data reader or production corpus provider.

## Why this comes first

The specification requires the 1500-2500 CE compressed data product to be
reproducible from public inputs and validated against measured error envelopes.
The current compressed artifact cannot be promoted until its source and hold-out
inputs are broad enough to support release claims.

## Current blockers

1. **Production source strategy** — broaden the exposed pure-Rust CSV reader
   into broader public-data inputs or a documented reproducible corpus-generation
   pipeline; split manifest/row corpus ingestion now exists as a step toward that.
2. **Corpus breadth** — cover all release-claimed bodies, channels, frames, and
   epoch classes with enough density for fitting and hold-out validation.
3. **Artifact accuracy** — keep the packaged artifact draft-grade until Phase 1
   inputs exist and Phase 2 thresholds pass.
4. **Compatibility evidence** — avoid widening house, ayanamsa, asteroid, Pluto,
   or lunar claims before supporting audits and validation are complete.

## Recommended next slice

Continue broadening the production source-corpus evidence in data first:

- add or generate representative source coverage to satisfy the contract;
- validate corpus shape, checksums, source revisions, and provenance from the
  data itself;
- update backend matrices and release profiles to consume the validated claim
  record.

## Parallel-safe work

- Improve compressed-artifact fitting behind draft labels.
- Audit Pluto, lunar theory, and selected-asteroid body-claim boundaries.
- Audit house and ayanamsa descriptor entries against source/provenance evidence.
- Harden release gates that check existing evidence without broadening claims.
