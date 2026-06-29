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
  corpus is committed: a Tier-A set of 25 bodies (Ceres, Pallas, Juno, Vesta,
  Hygiea, Psyche, Iris, Eunomia, Cybele, Astraea, Hebe, Flora, Metis, Fortuna,
  Sappho, Eros, plus TNOs Eris, Sedna, Haumea, Makemake, Quaoar, Orcus,
  Gonggong, Varuna, Ixion) reproducible from the pinned `sb441-n373s` kernel,
  plus a Tier-B constrained set of 11 centaurs and personal/minor main-belt
  bodies sourced from Horizons, advertised over 1900-2100 and held to the
  constrained body class (not release-grade).
- `pleiades-data` ships an ARTIFACT_VERSION 7 compressed artifact (SP2
  heliocentric-planet reframe complete, all bodies sub-arcsec; SP3 complete —
  published per-body-class accuracy ceilings enforced, hard size gate ≤ 12 MB
  active, latency tracked in `PACKAGED_BUDGETS`, motion output `Motion = Derived`
  via `SpeedPolicy::FittedDerivative` gated against speed ceilings, window
  1900–2100 CE).
- body/backend claims are now **per-backend**: Pluto, the Moon, and Eros are
  release-grade via the packaged-data artifact, while VSOP87's Pluto stays
  approximate and the compact ELP Moon stays constrained; the twenty-five
  `sb441-n373s` Tier-A asteroids/TNOs (Ceres, Pallas, Juno, Vesta, Hygiea,
  Psyche, Iris, Eunomia, Cybele, Astraea, Hebe, Flora, Metis, Fortuna, Sappho,
  Eros, plus TNOs Eris, Sedna, Haumea, Makemake, Quaoar, Orcus, Gonggong,
  Varuna, Ixion) are release-grade via the corpus-dependent JPL/SPK backend;
  True Apogee/Perigee remain unsupported.
- First-party body-position requests remain mean, geometric, and geocentric at
  the backend boundary. Chart-layer topocentric body positions are now supported
  as an opt-in correction (diurnal parallax + diurnal aberration, Phase 4
  topocentric sub-task complete); native-backend topocentric remains unsupported.
  Native sidereal backend output remains unsupported unless future validated
  backends add it. Apparent place of date is now the default chart-layer output
  (light-time + precession-to-date + annual aberration + nutation-in-longitude,
  implemented in `pleiades-apparent`; Phase 4 apparent-place sub-task complete).
  Built-in civil UTC/UT1 → TT/TDB conversion is implemented in `pleiades-time`
  (Phase 4 civil-time sub-task complete).
- House systems: 24 of 25 catalogued built-in house systems are release-grade
  (Phase 6 house-catalog promotion done as of 2026-06-24; all 12 target systems
  promoted via SE numeric gate; Albategnius is the sole built-in not in the
  Phase-6 target catalog, kept descriptor-only; no house-system known gaps
  remain). Ayanamsa:
  48 release-claimed modes pass the SE numeric gate (6 original + 17
  offset-defined (slice 1) + 13 fitted (slice 2, done 2026-06-25) + 12
  fitted-offset (slice 3, done 2026-06-25); the fitted family comprises 4
  true-star modes and 9 galactic modes; the fitted-offset family comprises 12
  modes with SE-anchored offsets). The remaining deferred set (11 modes) is:
  the 3 anchorless modes (Udayagiri, PVR Pushya-paksha, Sheoran),
  observational/topocentric/house Babylonians (Babylonian True Geoc, True Topc,
  True Obs, House, House Obs, Sissy), DhruvaGalacticCenterMula, and legacy
  GalacticEquator (no distinct SE code).

## Active implementation phases

| Phase | Focus | Workable-state promise | Details |
| --- | --- | --- | --- |
| 1 | Production reference backend and corpus | Maintainers can regenerate or ingest broad public reference inputs for every release-claimed body, frame, channel, and epoch class. | [plan/stages/01-production-reference-corpus.md](plan/stages/01-production-reference-corpus.md) |
| 2 | Release-grade compressed ephemeris **(done)** | The packaged backend (1900–2100 by default; 1600–2600 CE opt-in via `generate-artifact`) is generated from validated Phase 1 inputs and passes published accuracy ceilings, hard size gate (≤ 12 MB), and speed thresholds; latency tracked; motion derived (SP3 complete). | [plan/stages/02-production-compressed-ephemeris.md](plan/stages/02-production-compressed-ephemeris.md) |
| 3 | Body/backend claim closure **(done)** | Public body and backend claims are either validated, constrained, approximate, or unsupported with no ambiguous middle state. | [plan/stages/03-body-and-backend-claims.md](plan/stages/03-body-and-backend-claims.md) |
| 4 | Request-mode semantics | UTC/Delta-T, apparent, topocentric (chart layer, opt-in), and motion-output requests are implemented with evidence or rejected consistently; native sidereal backend output remains the only remaining Phase 4 item (deliberate non-goal). | [plan/stages/04-advanced-request-modes.md](plan/stages/04-advanced-request-modes.md) |
| 5 | Compatibility and release gates | House/ayanamsa compatibility evidence and release gates prevent stale artifacts, native-dependency drift, and overbroad claims. | [plan/stages/05-compatibility-and-release-readiness.md](plan/stages/05-compatibility-and-release-readiness.md) |
| 6 | Target catalog completion and expansion (end-state, post-first-release) | The full `compatibility-catalog.md` house/ayanamsa set and optional chart utilities are reachable without API redesign; remaining target entries are shipped or reported as known gaps. | [plan/stages/06-catalog-completion-and-expansion.md](plan/stages/06-catalog-completion-and-expansion.md) |

Phases 1-5 gate the first production release. **Phase 6** is end-state work the
spec commits to (`requirements.md` FR-4/FR-5, `compatibility-catalog.md`,
`roadmap.md` Phase 6) but does not require for the first release; it is tracked
here so the full target catalog is not silently treated as complete once Phase 5
audits pass.

## Current priority

Phases 1, 2, and 3 are complete. Phase 3 closed body/backend claim closure via
per-backend claim model enforced by the `claims-audit` gate. The active frontier
is **Phase 4**: request-mode semantics. Civil-time UTC/UT1 → TT/TDB conversion
is **done** (implemented in `pleiades-time`). Apparent place of date is **done**
(implemented in `pleiades-apparent` as the default chart-layer output).
Chart-layer topocentric body positions are **done** (opt-in, diurnal parallax +
diurnal aberration, Phase 4 topocentric sub-task complete). Remaining Phase 4
work: native sidereal backend output (deliberate non-goal; native-backend
topocentric stays unsupported at the backend boundary). Phase 6 is deferred
end-state work and must not broaden public claims before its own evidence exists.

**Phase 5 progress:** The compatibility-audit pair for the Phase 5 release gate
is now both done. House-system numeric gate is **done** (`validate-houses`,
60-row SE corpus, per-formula-family ceilings set from measured residuals; all 12
baseline house systems pass). Ayanamsa numeric gate is **done** (`validate-ayanamsa`,
60-row SE mean corpus, per-mode-class ceilings set from measured residuals; 48
release-claimed modes pass — the 6 original modes (Lahiri, Raman, Krishnamurti,
Fagan/Bradley; True Chitra, True Citra ≤ 1.0″) plus the 17 promoted
offset-defined modes (slice 1: J2000, J1900, B1950, Usha Shashi, Djwhal Khul,
Yukteshwar, JN Bhasin, Sassanian, Lahiri ICRC, Lahiri 1940, Aryabhata 522 CE,
Suryasiddhanta 499 CE, Suryasiddhanta 499 CE Mean Sun, Aryabhata 499 CE,
Aryabhata 499 CE Mean Sun, Suryasiddhanta Revati, Suryasiddhanta Citra) under
the `OffsetDefined` mode-class ceiling now raised to 3.0″ (was 2.0″), plus the
13 promoted fitted modes (slice 2, done 2026-06-25: 4 true-star + 9 galactic),
plus the 12 promoted fitted-offset modes (slice 3, done 2026-06-25: Krishnamurti
VP291, Lahiri VP285, Valens Moon, DeLuce, Babylonian Britton, Babylonian Kugler
1/2/3, Babylonian Huber, Babylonian Aldebaran, Hipparchus, Babylonian Eta
Piscium).
The remaining deferred set (11 modes) — the 3 anchorless modes (Udayagiri,
PVR Pushya-paksha, Sheoran), the observational/topocentric/house Babylonians
(Babylonian True Geoc, True Topc, True Obs, House, House Obs, Sissy),
DhruvaGalacticCenterMula, and legacy GalacticEquator (no distinct SE code) —
are kept descriptor-only with no claim broadening. Compatibility overclaim gate
is **done**: claim tiers now live on catalog descriptors (per-entry
`claim_tier`); `compat-claims-audit` enforces tier↔evidence↔profile↔prose
agreement bidirectionally (catalog tiers must match SE numeric-gate evidence,
the compatibility profile, and README prose); `release-smoke`/`release-gate`
now run the full numeric-gate set (house, ayanamsa, apparent, topocentric,
corpus) plus the overclaim audit. Phase 5 is complete.

**Phase 6 progress (house-catalog release-grade promotion):** The house half of
Phase 6 is **done** as of 2026-06-24. All 12 target house systems were promoted
to release-grade via SE numeric gate (`validate-houses`, 115-row corpus including
variable-length Gauquelin sector rows; per-formula-family ceilings). Horizon and
Gauquelin were promoted via algorithm corrections (Horizon: corrected SE `H`
azimuth convention; Gauquelin: rewrote the 36-sector computation as a
Placidus-family semi-arc division, replacing a linear longitude lerp), not left
as known gaps. **24 of 25 built-in house systems are release-grade; the sole
built-in not in the Phase-6 target catalog is Albategnius (descriptor-only).**
There are no house-system known gaps to record. The high-latitude
`SwissEphemerisFallback` policy substitutes Porphyry's 12 quadrant cusps, a
valid substitute only for 12-cusp systems; for the 36-sector Gauquelin system
(which has no validated high-latitude reference) the fallback now rejects cleanly
with `InvalidLatitude` rather than emitting a dimensionally-invalid snapshot.
**Phase 6 ayanamsa fitted-family promotion (slice 2) is done** as of 2026-06-25.
13 fitted modes (4 true-star + 9 galactic) were promoted to release-grade,
bringing the total release-claimed ayanamsa count to 36 (6 original + 17
offset-defined + 13 fitted). **Phase 6 ayanamsa fitted-offset promotion (slice
3) is done** as of 2026-06-25. 12 fitted-offset modes (Krishnamurti VP291,
Lahiri VP285, Valens Moon, DeLuce, Babylonian Britton, Babylonian Kugler 1/2/3,
Babylonian Huber, Babylonian Aldebaran, Hipparchus, Babylonian Eta Piscium)
were promoted to release-grade; all 12 candidates passed the SE numeric gate
with no deferrals by residual. Total release-claimed ayanamsa count is now 48
(6 original + 17 offset-defined + 13 fitted + 12 fitted-offset). The
still-deferred set (11 modes) is listed in "Important current limits" above.
**Phase 6 ayanamsa slice 4 (descriptor accuracy) is done** as of 2026-06-26.
The six custom-definition-only Babylonian descriptors (House, Sissy, True Geoc,
True Topc, True Obs, House Obs) no longer falsely claim a Swiss Ephemeris label;
compatibility profile bumped to 0.7.1. Catalogued counts unchanged (59
catalogued / 48 release-grade / 11 deferred); the six remain the validated
custom-definition-only category, not release-claimed.

## Plan maintenance rules

- Keep this plan limited to remaining implementation work.
- Remove tasks when they are implemented instead of adding completion notes.
- Do not list individual CLI aliases, report wrappers, sidecar files, or cache
  optimizations unless they are the remaining blocker.
- Keep `README.md`, release profiles, generated reports, and this plan aligned
  when public behavior or release claims change.

Status: refreshed 2026-06-29 — **SP3 complete; Phases 1–3 done; per-backend claim model enforced by the claims-audit gate; Phase 4 active — civil-time conversion done, apparent-place done, topocentric (chart layer) done; only native sidereal backend output remains (deliberate non-goal); Phase 5 complete — house gate done + ayanamsa gate done + overclaim gate done; release-gate now runs the full numeric-gate set (house, ayanamsa, apparent, topocentric, corpus) plus the overclaim audit; Phase 6 house-catalog release-grade promotion done — 24 of 25 built-in house systems release-grade (Albategnius the sole built-in not in the Phase-6 target catalog, kept descriptor-only); all 12 target systems promoted; no house-system known gaps; Phase 6 ayanamsa offset-defined promotion (slice 1) done — 23 release-claimed ayanamsa modes (6 original + 17 promoted offset-defined), OffsetDefined ceiling raised to 3.0″; Phase 6 ayanamsa fitted-family promotion (slice 2) done — 36 release-claimed ayanamsa modes total (6 original + 17 offset-defined + 13 fitted); Phase 6 ayanamsa fitted-offset promotion (slice 3) done — 48 release-claimed ayanamsa modes total (6 original + 17 offset-defined + 13 fitted + 12 fitted-offset); still-deferred (11 modes): 3 anchorless modes, observational/topocentric/house Babylonians, DhruvaGalacticCenterMula, legacy GalacticEquator; Phase 6 ayanamsa slice 4 (descriptor accuracy) done — six custom-definition-only Babylonian descriptors corrected, compatibility profile bumped to 0.7.1, counts unchanged (59 catalogued / 48 release-grade / 11 deferred); Phase 6 asteroid sb441-n16 Tier-A promotion (slice 1) done — Tier-A release-grade asteroid set grew from 7 to 9 by adding 15 Eunomia and 65 Cybele (both kernel-confirmed present in sb441-n16; astrological usage cited via Swiss Ephemeris asteroid name catalog and Martha Lang-Wescott, *Mechanics of the Future: Asteroids*); Hebe was evaluated but is absent from sb441-n16 and stays Tier-B/constrained; non-kernel Tier-B bodies (Chiron, Eros, …) remain constrained and are deferred to a follow-up slice; Tier-B count unchanged (27 bodies); Phase 6 asteroid slice 2 done — retired sb441-n16, pinned sb441-n373s; 16 bodies promoted to Tier-A (Astraea, Hebe, Flora, Metis, Fortuna, Sappho, Eros plus TNOs Eris, Sedna, Haumea, Makemake, Quaoar, Orcus, Gonggong, Varuna, Ixion); Tier-A count now 25 (original 9 + 16 promoted); Tier-B count now 11 (5 centaurs + 6 personal/minor main-belt; all 9 TNOs promoted)**.
Published per-body-class accuracy ceilings enforced (1900–2100 CE), hard size gate
active (≤ 12 MB), latency tracked, motion output `Motion = Derived`
(SpeedPolicy::FittedDerivative) gated. ARTIFACT_VERSION 7.

Size / perf baseline (`sp1_draft_size_perf_baseline`, unoptimized build):

| Metric         | Before (1600–2600)      | After (1900–2100)       | Change        |
| -------------- | ----------------------- | ----------------------- | ------------- |
| Artifact size  | 49,780,387 B (~47.5 MB) | 10,491,287 B (~10.0 MB) | 4.75× smaller |
| Decode latency | 1315.9 ms               | 259.7 ms                | 5.1× faster   |
| Lookup latency | 16,691.4 µs             | 3,326.0 µs              | 5.0× faster   |

Accuracy baseline (SP2, heliocentric-planet reframe) committed in
`crates/pleiades-data/src/accuracy_baseline.rs` (measured over the 1900–2100
hold-out): planets (Mercury–Pluto) stored heliocentrically; geocentric ecliptic
reconstructed at lookup via `P_geo = P_helio + S_geo`; Sun/Moon/Eros remain
geocentric. All bodies sub-arcsec (Uranus ~0.0036″, Neptune ~0.0020″,
Pluto ~0.0018″, Saturn ~0.0009″, Jupiter ~0.0004″; inner bodies + Sun + Moon
remain sub-arcsec). SP3 thresholds and budgets now enforce these ceilings.
Prior refresh 2026-06-19 narrowed default coverage window to 1900–2100. Prior
refresh 2026-06-17 committed the curated asteroid corpus (Tier A from
`sb441-n16`, Tier B constrained from Horizons over 1900-2100). Prior refresh
2026-06-17 added the broad public-data reader (`pleiades-jpl::ingest`). Prior
refresh 2026-06-16 promoted the `pleiades-jpl` reference corpus to a real,
broad, de440-sourced, checksum-pinned product behind a live fail-closed
`validate-corpus` gate. Prior refresh 2026-06-13 added Phase 6 for deferred
end-state catalog completion. Prior refresh 2026-05-24 reviewed `SPEC.md`,
`spec/*.md`, README status, current crates, CLI/report posture, and checked-in
plan files.
