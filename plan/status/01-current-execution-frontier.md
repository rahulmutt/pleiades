# Status 1 — Current Execution Frontier

## Frontier

Phase 1, production reference backend and corpus, is the active frontier.

The repository has strong scaffolding around corpus summaries, backend matrices,
comparison reports, release bundle rehearsal, and CLI/validation inspection.
The JPL source-corpus contract is now explicit about reference/hold-out
provenance, source windows, source revisions, exact J2000 fixture evidence, and
boundary-request corpora, and the production-generation source summary now
carries the exact J2000 fixture-evidence payload directly alongside the source
window payload. The production-generation manifest summary continues to
validate the derived source, coverage, and boundary request corpus records
directly. The release-grade body-claims and body/date/channel posture now
assembles from structured body lists instead of a single hand-written prose
string. The remaining blocker is still the underlying production-grade
reference input strategy: current JPL evidence is a checked-in snapshot/
hold-out fixture set, not a broad public-data reader or production corpus
provider.

## Why this comes first

The specification requires the 1500-2500 CE compressed data product to be
reproducible from public inputs and validated against measured error envelopes.
The current compressed artifact cannot be promoted until its source and hold-out
inputs are broad enough to support release claims.

## Current blockers

1. **Production source strategy** — implement a pure-Rust public-data reader or a
   documented reproducible corpus-generation pipeline.
2. **Corpus breadth** — cover all release-claimed bodies, channels, frames, and
   epoch classes with enough density for fitting and hold-out validation.
3. **Claim derivation** — derive body/date/channel release claims from validated
   corpus evidence rather than narrative summaries.
4. **Artifact accuracy** — keep the packaged artifact draft-grade until Phase 1
   inputs exist and Phase 2 thresholds pass.
5. **Compatibility evidence** — avoid widening house, ayanamsa, asteroid, Pluto,
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
