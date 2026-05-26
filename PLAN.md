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
- the JPL snapshot crate now exposes reusable pure-Rust CSV parsing entry points
  for the checked-in snapshot fixtures and a broader manifest+row corpus loader
  for arbitrary JPL-style CSV text, including path-backed split-source loading
  for separate manifest/row files, while validation, CLI, audit, benchmark,
  report, and release-bundle rehearsal surfaces continue to fail closed on stale
  rendered sidecars; source-corpus summaries now carry the production-generation
  body-class coverage and cadence payloads alongside the source-window evidence,
  and the release-facing body/date/channel posture now derives from validated
  corpus evidence;
- unsupported advanced modes are represented in policy surfaces rather than
  silently accepted;
- the workspace audit now checks the pinned `mise.toml` rust toolchain against
  the workspace `rust-version` and requires the `rustfmt` and `clippy`
  components, so tool-version provenance is part of the release gate.

## Important current limits

These are the implementation gaps that still block a production release:

- `pleiades-jpl` is a checked-in Horizons snapshot/fixture backend, not yet a
  broad public-data reader or production reference corpus provider.
- The checked-in source corpus is useful regression evidence, but it is sparse
  relative to the 1500-2500 CE production-artifact and body-claim goals; recent
  cleanup corrected the selected-asteroid Apophis J2000 / early-2001 fixture
  rows to match Horizons and pins those values in regression tests, but the
  broader corpus breadth gap remains.
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
  claims.

## Active implementation phases

| Phase | Focus | Workable-state promise | Details |
| --- | --- | --- | --- |
| 1 | Production reference backend and corpus | Maintainers can regenerate or ingest broad public reference inputs for every release-claimed body, frame, channel, and epoch class. | [plan/stages/01-production-reference-corpus.md](plan/stages/01-production-reference-corpus.md) |
| 2 | Release-grade compressed ephemeris | The 1500-2500 CE packaged backend is generated from validated Phase 1 inputs and passes published accuracy, size, and speed thresholds. | [plan/stages/02-production-compressed-ephemeris.md](plan/stages/02-production-compressed-ephemeris.md) |
| 3 | Body/backend claim closure | Public body and backend claims are either validated, constrained, approximate, or unsupported with no ambiguous middle state. | [plan/stages/03-body-and-backend-claims.md](plan/stages/03-body-and-backend-claims.md) |
| 4 | Request-mode semantics | UTC/Delta-T, apparent, topocentric, native sidereal, and motion-output requests are implemented with evidence or rejected consistently. | [plan/stages/04-advanced-request-modes.md](plan/stages/04-advanced-request-modes.md) |
| 5 | Compatibility and release gates | House/ayanamsa compatibility evidence and release gates prevent stale artifacts, native-dependency drift, and overbroad claims. | [plan/stages/05-compatibility-and-release-readiness.md](plan/stages/05-compatibility-and-release-readiness.md) |

## Current priority

Start with **Phase 1**: production reference inputs. Phase 2 generator work can
continue in parallel, but the compressed artifact must remain draft-grade until
its source inputs and hold-out corpus are production-grade.

## Plan maintenance rules

- Keep this plan limited to remaining implementation work.
- Remove tasks when they are implemented instead of adding completion notes.
- Do not list individual CLI aliases, report wrappers, sidecar files, or cache
  optimizations unless they are the remaining blocker.
- Keep `README.md`, release profiles, generated reports, and this plan aligned
  when public behavior or release claims change.

Status: refreshed 2026-05-24 after reviewing `SPEC.md`, `spec/*.md`, README
status, current crates, CLI/report posture, and checked-in plan files.
