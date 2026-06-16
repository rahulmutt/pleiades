# Status 1 — Current Execution Frontier

## Frontier

Phase 1, production reference backend and corpus, is the active frontier.

The reference corpus is now a real, broad, de440-sourced product committed under
`crates/pleiades-jpl/data/corpus/` (~25,659 data rows across boundary, interior,
fast-cluster, hold-out, and independent fixture-golden slices), sampled per-body
at each body's own cadence rather than every body on the Moon grid, with real
non-zero checksums and the pinned kernel SHA-256 in `manifest.txt`. The de440
kernel itself (114 MB, public domain) is not committed; only its SHA-256 is
pinned (in `corpus_spec::KERNEL_SHA256` and `docs/spk-kernel-sourcing.md`). The
fail-closed `validate-corpus` gate is live (no longer ignored) and passes over
the real committed corpus, failing closed on missing bodies/roles, schema drift,
checksum drift, malformed/non-finite rows, a placeholder kernel SHA, and
fixture-golden cross-check breaches. A kernel-gated `corpus_regen` test
reproduces all four backend slices from de440 within 1 km, so a clean checkout
can verify kernel-free and reproduce with the kernel. The fixture-golden slice is
independent (Horizons-sourced, not de440), giving a real cross-check; the giant
planets resolve to de440 system barycenters (de440 lacks planet-center IDs for
them), so their cross-check tolerance is 600 km, covering the
astrologically-negligible (<0.1") barycenter-vs-center offset, documented in the
gate. The JPL source-corpus contract remains explicit about reference/hold-out
provenance, source windows, source revisions, exact J2000 fixture evidence, and
boundary-request corpora, and the release-grade body-claims and
body/date/channel posture assemble from structured body lists and validated
corpus evidence. The checked-in JPL-style snapshots also expose reusable
pure-Rust CSV parsing entry points plus combined, split-source, and path-backed
split-source corpus loaders for arbitrary JPL-style CSV text. Separately, the
workspace audit checks the pinned `mise.toml` rust toolchain against the
workspace `rust-version` and requires the `rustfmt` and `clippy` components, so
tool-version provenance is part of the release gate. The remaining Phase 1 gap is
a broad public-data reader for arbitrary external JPL-style data products (beyond
the pinned de440 kernel and checked-in fixtures) and asteroid-kernel adoption.

## Why this comes first

The specification requires the 1600-2600 CE compressed data product to be
reproducible from public inputs and validated against measured error envelopes.
The current compressed artifact cannot be promoted until its source and hold-out
inputs are broad enough to support release claims.

## Current blockers

1. **Broad public-data reader** — the reproducible de440 generation pipeline and
   the live `validate-corpus` gate now produce and verify a broad checked corpus,
   but a reader for arbitrary external public JPL-style data products (beyond the
   pinned de440 kernel and checked-in fixtures) is still open, as is
   asteroid-kernel adoption.
2. **Artifact accuracy** — keep the packaged artifact draft-grade until Phase 2
   draft→production compressed-artifact thresholds pass.
3. **Compatibility evidence** — avoid widening house, ayanamsa, asteroid, Pluto,
   or lunar claims before supporting audits and validation are complete.

## Recommended next slice

The reproducible generation pipeline, kernel-free verification, and the
fail-closed corpus gate are in place over the real de440-sourced corpus. The
remaining Phase 1 work is the broad public-data reader:

- add a reader for arbitrary external public JPL-style data products on top of
  the existing split-source/path-backed loaders, and record any adopted
  small-body asteroid kernel provenance;
- then turn to Phase 2 readiness: rebase artifact generation on the validated
  Phase 1 corpus and define the draft→production threshold posture per body
  class and channel.

## Parallel-safe work

- Improve compressed-artifact fitting behind draft labels.
- Audit Pluto, lunar theory, and selected-asteroid body-claim boundaries.
- Audit house and ayanamsa descriptor entries against source/provenance evidence.
- Harden release gates that check existing evidence without broadening claims.
