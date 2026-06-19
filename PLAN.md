# Pleiades Development Plan

This is the active forward plan for `pleiades`. It tracks only work still required by
`SPEC.md` and `spec/*.md`; historical bootstrap tasks, completed command aliases,
summary wrappers, bundle sidecars, and cache/report hardening are intentionally not
listed.

## Current implementation baseline

The workspace is past the original foundation roadmap:

- all mandatory first-party `pleiades-*` crates exist and preserve the specified
  layering;
- shared types, backend traits, capability metadata, batch helpers, composite
  routing helpers, and the high-level chart facade are present;
- baseline house and ayanamsa catalogs are available, with broader descriptor
  catalogs and compatibility-profile reporting;
- VSOP87-style planetary, compact Meeus-style lunar/lunar-point, JPL snapshot,
  and packaged-data backend crates exist;
- the JPL crate ships a reproducible de440-sourced reference corpus committed
  under `crates/pleiades-jpl/data/corpus/` behind a live fail-closed
  `validate-corpus` gate, plus reusable pure-Rust CSV parsing entry points and
  combined, split-source, and path-backed split-source corpus loaders for
  arbitrary JPL-style CSV text, while validation, CLI, audit, benchmark, report,
  and release-bundle rehearsal surfaces continue to fail closed on stale
  rendered sidecars; source-corpus summaries carry the production-generation
  body-class coverage and cadence payloads alongside the source-window evidence,
  and the release-facing body/date/channel posture derives from validated
  corpus evidence;
- unsupported advanced modes are represented in policy surfaces rather than
  silently accepted;
- the workspace audit now checks the pinned `mise.toml` rust toolchain against
  the workspace `rust-version` and requires the `rustfmt` and `clippy`
  components, so tool-version provenance is part of the release gate.

## Important current limits

These are the implementation gaps that still block a production release:

- `pleiades-jpl` carries a reproducible generation pipeline that produces a
  broad, de440-sourced reference corpus (~15,331 data rows across 1900-2100 CE,
  per-body cadence) committed under `crates/pleiades-jpl/data/corpus/` with real
  checksums and a pinned kernel SHA, behind a live fail-closed `validate-corpus`
  gate; a clean checkout verifies it kernel-free and reproduces it from de440
  with `PLEIADES_DE_KERNEL`. It now also ingests arbitrary external JPL-style
  data products (Horizons vector-table text, Horizons API JSON, generic CSV)
  into the corpus types via `pleiades-jpl::ingest`, with optional live Horizons
  fetch behind the default-off `horizons-fetch` feature. A curated asteroid
  corpus is committed: a Tier A main-belt core (Ceres, Pallas, Juno, Vesta,
  Hygiea, Psyche, Iris) reproducible from the pinned `sb441-n16` kernel, plus a
  Tier B constrained set of 27 centaurs, personal asteroids, and TNOs sourced
  from Horizons, advertised over 1900-2100 and held to the constrained body
  class (not release-grade).
- `pleiades-data` ships a draft compressed artifact whose model-error envelope
  still exceeds production thresholds for many bodies and channels.
- Pluto remains approximate/fallback-backed in first-party algorithmic paths and
  is excluded from release-grade major-body claims.
- `pleiades-elp` is a compact Meeus-style lunar baseline, not a full ELP
  coefficient implementation.
- Baseline selected-asteroid evidence exists, but broad asteroid release claims
  are not yet supported.
- First-party body-position requests remain mean, geometric, geocentric, and
  tropical at the backend boundary. Built-in UTC/Delta-T conversion,
  apparent-place corrections, topocentric body positions, and native sidereal
  backend output remain unsupported unless future validated backends add them.
- Broad house and ayanamsa descriptor catalogs are present, but formula,
  provenance, and interoperability audits still gate stronger compatibility
  claims, and the full target catalog enumerated in
  `spec/compatibility-catalog.md` (remaining house systems beyond the baseline 11
  and the broader Swiss Ephemeris ayanamsa set) is not yet implemented to
  release grade; that completion is deferred end-state work tracked in Phase 6.

## Active implementation phases

| Phase | Focus | Workable-state promise | Details |
| --- | --- | --- | --- |
| 1 | Production reference backend and corpus | Maintainers can regenerate or ingest broad public reference inputs for every release-claimed body, frame, channel, and epoch class. | [plan/stages/01-production-reference-corpus.md](plan/stages/01-production-reference-corpus.md) |
| 2 | Release-grade compressed ephemeris | The packaged backend (1900-2100 by default, with the wider 1600-2600 CE window opt-in via `generate-artifact`) is generated from validated Phase 1 inputs and passes published accuracy, size, and speed thresholds. | [plan/stages/02-production-compressed-ephemeris.md](plan/stages/02-production-compressed-ephemeris.md) |
| 3 | Body/backend claim closure | Public body and backend claims are either validated, constrained, approximate, or unsupported with no ambiguous middle state. | [plan/stages/03-body-and-backend-claims.md](plan/stages/03-body-and-backend-claims.md) |
| 4 | Request-mode semantics | UTC/Delta-T, apparent, topocentric, native sidereal, and motion-output requests are implemented with evidence or rejected consistently. | [plan/stages/04-advanced-request-modes.md](plan/stages/04-advanced-request-modes.md) |
| 5 | Compatibility and release gates | House/ayanamsa compatibility evidence and release gates prevent stale artifacts, native-dependency drift, and overbroad claims. | [plan/stages/05-compatibility-and-release-readiness.md](plan/stages/05-compatibility-and-release-readiness.md) |
| 6 | Target catalog completion and expansion (end-state, post-first-release) | The full `compatibility-catalog.md` house/ayanamsa set and optional chart utilities are reachable without API redesign; remaining target entries are shipped or reported as known gaps. | [plan/stages/06-catalog-completion-and-expansion.md](plan/stages/06-catalog-completion-and-expansion.md) |

Phases 1-5 gate the first production release. **Phase 6** is end-state work the
spec commits to (`requirements.md` FR-4/FR-5, `compatibility-catalog.md`,
`roadmap.md` Phase 6) but does not require for the first release; it is tracked
here so the full target catalog is not silently treated as complete once Phase 5
audits pass.

## Current priority

Start with **Phase 1**: production reference inputs. Phase 2 generator work can
continue in parallel, but the compressed artifact must remain draft-grade until
its source inputs and hold-out corpus are production-grade. Phase 6 is deferred
end-state work and must not broaden public claims before its own evidence exists.

## Plan maintenance rules

- Keep this plan limited to remaining implementation work.
- Remove tasks when they are implemented instead of adding completion notes.
- Do not list individual CLI aliases, report wrappers, sidecar files, or cache
  optimizations unless they are the remaining blocker.
- Keep `README.md`, release profiles, generated reports, and this plan aligned
  when public behavior or release claims change.

Status: refreshed 2026-06-19 — **default coverage window narrowed to 1900–2100**.
The shipped packaged artifact now covers 1900–2100 (the default `CoverageWindow`);
wider windows are opt-in, generated by power users via the `generate-artifact`
CLI subcommand (dense de440 fit over any `CoverageWindow`). Major bodies are
fit densely from de440 within each per-body segment span (kernel-gated, same gate
as corpus_regen); the constrained asteroid (Eros) is sourced from its committed
reference corpus and is window-independent. ARTIFACT_VERSION 5.

Size / perf baseline (`sp1_draft_size_perf_baseline`, unoptimized build):

| Metric         | Before (1600–2600)      | After (1900–2100)       | Change        |
| -------------- | ----------------------- | ----------------------- | ------------- |
| Artifact size  | 49,780,387 B (~47.5 MB) | 10,491,287 B (~10.0 MB) | 4.75× smaller |
| Decode latency | 1315.9 ms               | 259.7 ms                | 5.1× faster   |
| Lookup latency | 16,691.4 µs             | 3,326.0 µs              | 5.0× faster   |

Accuracy baseline committed in `crates/pleiades-data/src/accuracy_baseline.rs`
(measured over the 1900–2100 hold-out): inner bodies + Sun + Moon sub-arcsec;
outer planets draft-level (Uranus ~192″, Neptune ~109″, Pluto ~62″, Saturn ~9.5″,
Jupiter ~1.5″). The packaged artifact remains draft-grade; SP2 (accuracy tuning)
and SP3 (thresholds/budgets) are next.
Prior refresh 2026-06-17 committed the curated asteroid corpus (Tier A from
`sb441-n16`, Tier B constrained from Horizons over 1900-2100). Prior refresh
2026-06-17 added the broad public-data reader (`pleiades-jpl::ingest`). Prior
refresh 2026-06-16 promoted the `pleiades-jpl` reference corpus to a real,
broad, de440-sourced, checksum-pinned product behind a live fail-closed
`validate-corpus` gate. Prior refresh 2026-06-13 added Phase 6 for deferred
end-state catalog completion. Prior refresh 2026-05-24 reviewed `SPEC.md`,
`spec/*.md`, README status, current crates, CLI/report posture, and checked-in
plan files.
