# Phase 1 — Production Reference Backend and Corpus

## Goal

Move from checked-in regression fixtures to production-grade public reference
inputs that can validate release claims and generate the 1600-2600 CE compressed
artifact.

## Current baseline

- JPL/Horizons snapshot and hold-out CSV fixtures are checked in and validated.
- Source, frame, time-scale, schema, checksum, redistribution posture, and
  exact J2000 fixture-exactness evidence are reported through CLI, validation,
  backend-matrix, and bundle surfaces; the release-facing body/date/channel
  posture now derives from validated corpus evidence rather than narrative
  prose, and the checked-in JPL-style snapshots now have reusable pure-Rust CSV
  parsing entry points for their manifest and row data plus split-source,
  path-backed split-source, and combined corpus loaders for arbitrary JPL-style
  CSV text.
- The reference corpus is now a real, broad, de440-sourced product committed
  under `crates/pleiades-jpl/data/corpus/` (boundary, interior, fast-cluster,
  hold-out, and independent fixture-golden slices, ~25,659 data rows), sampled
  per-body at each body's own cadence, with real non-zero checksums and the
  pinned kernel SHA-256 in `manifest.txt`. The de440 kernel itself is not
  committed; only its SHA-256 is pinned (in `corpus_spec::KERNEL_SHA256` and
  `docs/spk-kernel-sourcing.md`).

## Remaining implementation work

- Add a broad public-data reader for arbitrary external JPL-style data products,
  beyond the pinned de440 kernel and checked-in fixtures, on top of the exposed
  pure-Rust CSV parsing and split-source loading paths. Keep separate manifest
  and row sources first-class so corpus-generation tooling can work from split
  public inputs as well as combined fixture text.
- Adopt a small-body asteroid SPK kernel to broaden selected-asteroid source
  coverage (record its provenance in `docs/spk-kernel-sourcing.md` when added).
- Keep fitting/reference, independent hold-out, boundary-overlay,
  fixture-exactness, and provenance-only evidence separate in data and reports.

## Exit criteria

- A clean checkout can verify the reference inputs kernel-free
  (`pleiades-validate validate-corpus`) and reproduce all backend slices from
  de440 with `PLEIADES_DE_KERNEL` set (the gated `corpus_regen` test
  reproduces every slice within 1 km). **Met** for the de440 kernel and
  checked-in fixtures; a reader for arbitrary external public data products is
  still open.
- Corpus validation fails closed on missing bodies, epochs, channels, roles,
  schema drift, checksum/source-revision drift, malformed/non-finite rows,
  placeholder kernel SHA, and fixture-golden cross-check breaches. **Met** — the
  embedded `validate-corpus` gate is live (no longer ignored) over the real
  committed corpus.
- Release-claimed body/channel/frame/date coverage is broad enough to feed the
  production artifact generator and hold-out comparisons. **Met** across
  1600-2600 CE; Pluto stays constrained.
