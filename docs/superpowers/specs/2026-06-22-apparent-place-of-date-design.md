# Apparent Place of Date — Design (Phase 4, supersedes apparent-place-corrections)

**Status:** Approved design — 2026-06-22
**Supersedes:** `docs/superpowers/specs/2026-06-22-apparent-place-corrections-design.md` and its plan `docs/superpowers/plans/2026-06-22-apparent-place-corrections.md` (parked at Task 1).

## Goal

Make the pleiades engine produce **apparent ecliptic positions referred to the true equinox of date** as the **default** chart output for release-grade bodies, by composing four corrections — **light-time, precession-to-date, nutation-in-longitude, and annual aberration** — in a single pure crate (`pleiades-apparent`), and validate the result end-to-end against a regenerated JPL Horizons apparent-of-date corpus to arcsecond tolerance.

## Problem & context

The packaged-data backend returns geocentric ecliptic positions referred to the **J2000 / ICRF equinox**, not the equinox of date. There is **no precession step anywhere in the workspace**: `icrf_to_ecliptic()` (`crates/pleiades-jpl/src/spk/chain.rs:148`) only tilts a fixed ICRF/J2000 vector by the mean obliquity, and the validation corpus CSV headers literally read "geocentric ecliptic J2000". Consequently the engine's "tropical" longitudes are J2000-framed and off from true of-date tropical by accumulated precession — up to ~1.4° at the 1900/2100 window edges (≈50.3″/yr).

This was discovered when the parked apparent-place plan hit its Task 1 guard test (`sun_longitude_is_of_date_not_j2000`): the backend returned the Sun at 281.22° for JD 2433283.0 (1950), where the of-date value is ~280.5° (the +0.70° gap is exactly 50 years of precession). That plan applied nutation + aberration "to true equinox of date" but **omitted precession**, so it could not be correct as written.

Verified during design: JPL Horizons **is** reachable from this environment at `https://ssd.jpl.nasa.gov/api/horizons.api` (the host the project's own `crates/pleiades-jpl/src/ingest/fetch.rs:44` uses; the earlier "unreachable" reading was a wrong-hostname test against `ssd.api.jpl.nasa.gov`). Horizons quantity 31 (ObsEcLon, apparent ecliptic of date) returns 280.3689° for the Sun at J2000 — confirming we can source apparent-of-date reference values directly.

## Decisions (with rationale)

1. **Scope is merged, not layered.** Precession-to-date is implemented together with nutation + aberration + light-time as one apparent-place capability, rather than as a separate "foundation" pass. Rationale: Horizons' natural of-date output (Q31) is *apparent* (it bakes in aberration + nutation), so a precession-only intermediate cannot be validated tightly against Horizons; building the full reduction matches Horizons to arcsec in one pass and is scientifically cleanest.
2. **Apparent of-date is the default output** for release-grade bodies; this is what almanacs / Swiss Ephemeris / astrologers expect and satisfies the goal that all charts are of-date. Mean (J2000) remains available as an explicit **diagnostic** mode and keeps validating against the existing J2000 corpus. Non-release-grade bodies fall back to mean with provenance noting it (not an error).
3. **Backend stays J2000 mean-only** (Approach 1). All frame/correction math lives in the pure `pleiades-apparent` crate; the chart layer composes it. Rationale: preserves the strong existing J2000 corpus as a geometric-core gate, keeps backends simple, and puts precession alongside the corrections it combines with.
4. **Validation regenerates the corpus from Horizons apparent-of-date (Q31)** and validates the chart-layer apparent output end-to-end to arcsec. The existing J2000 corpus is retained as the geometric-core gate.
5. **Precession model: Meeus ch. 21 ecliptic precession** (IAU polynomial; sub-arcsec over 1900–2100; pure, no data file). The arcsec end-to-end Horizons gate is the arbiter; upgrade to IAU 2006 only if it misses.

## Architecture

```
backend (pleiades-data)          pleiades-apparent  (pure; only dep: pleiades-types)
  geocentric ECLIPTIC J2000        lighttime.rs   — retarded-epoch iterator
  MEAN position (λ,β,dist)         precession.rs  — NEW: J2000 ecliptic → ecliptic of date (Meeus 21)
        │  (re-queried at           nutation.rs    — Δψ, Δε, mean obliquity (checksum-pinned IAU-1980)
        │   retarded epochs)        aberration.rs  — annual aberration (Meeus 23)
        ▼                          apparent.rs    — orchestrator (composes the four)
  pleiades-core chart engine ◄──── provenance.rs / error.rs / policy.rs
    default: apparent of-date (release-grade)
    diagnostic: mean (J2000)
        ▼
  pleiades-cli (default apparent; --mean diagnostic; provenance line)
        ▼
  pleiades-validate
    EXISTING J2000 corpus      → gates backend geometric core (unchanged, relabeled)
    NEW apparent-of-date corpus (Horizons Q31) → gates chart apparent output, arcsec
```

`pleiades-apparent` is the crate the parked plan already designed (FNV-1a checksums byte-identical to `pleiades_time::fnv1a64`; `summary_line()` + `Display` on every public error/summary type; edition 2021; only dep `pleiades-types` + optional `serde`). This spec adds `precession.rs` and threads a precession stage through the orchestrator and provenance.

## The of-date apparent reduction pipeline

For each release-grade body at TT instant *t*, the orchestrator runs in order:

1. **Light-time** — iterate the retarded epoch *t − τ*, τ = distance × `LIGHT_TIME_DAYS_PER_AU` (0.005_775_518_3 d/AU), re-querying the backend (mean J2000) until convergence. Yields the geometric **J2000** ecliptic position of where the body is seen. *(existing `lighttime.rs`)*
2. **Precession (NEW)** — rotate that geometric direction from the **J2000 ecliptic/equinox to the mean ecliptic/equinox of date**, transforming both λ and β (ecliptic→ecliptic precession, Meeus ch. 21). Pure polynomial, no data file.
3. **Nutation** — add **Δψ** to longitude (mean → true equinox of date); Δψ leaves β unchanged. *(existing `nutation.rs`)*
4. **Aberration** — add annual aberration Δλ, Δβ (apparent displacement from Earth's velocity). *(existing `aberration.rs`)*

Result: `λ_apparent = precess_λ(λ_J2000, β_J2000, t) + Δψ + Δλ_aberr`; `β_apparent = precess_β(λ_J2000, β_J2000, t) + Δβ_aberr` — apparent, true equinox of date, matching Horizons Q31.

The **Sun's true longitude ⊙** needed by the aberration term is queried mean and **precessed to of-date** before use, for frame consistency.

**Precession interface (new in `pleiades-apparent`):**
- `precession::precess_ecliptic_j2000_to_date(lon_deg, lat_deg, jd_tt) -> (lon_deg, lat_deg)` (Meeus ch. 21).
- Non-finite output → `ApparentPlaceError::NonFiniteCorrection { stage: "precession" }`.

**Provenance additions (`ApparentProvenance` / `CorrectionSet`):**
- `CorrectionSet.precession: bool`; `ApparentProvenance.precession_longitude_arcsec: f64`; `model_sources` notes the precession model.

## Chart-layer integration & default mode (`pleiades-core`)

- Default apparentness becomes **Apparent**; an explicit **mean (diagnostic)** mode returns the raw backend J2000 position. The backend only ever receives `Mean`; "apparent" is a chart-layer composition.
- Per body under the default:
  - *release-grade* → query backend mean (J2000) → apparent pipeline → apparent-of-date placement; `BodyPlacement.apparent = Some(provenance)`; `position.apparent = Apparentness::Apparent`; **sign re-derived from the apparent longitude**.
  - *non-release-grade* → graceful fallback to mean J2000; `apparent = None`; diagnostic notes "apparent unavailable (not release-grade)". Not an error.
- `pleiades-core` re-exports `ApparentProvenance` / `CorrectionSet`.

## CLI (`pleiades-cli`)

- Default chart output is apparent of-date; add a `--mean` diagnostic flag (raw J2000); emit an apparent-provenance line per release-grade body; update help/usage text.

## Validation / corpus regeneration (`pleiades-validate`)

- **New corpus** `crates/pleiades-validate/data/apparent-goldens.csv` — columns `body, jd_tt, apparent_longitude_deg, tolerance_arcsec` — covering release-grade bodies (Sun, Moon, Mercury–Pluto, Eros) across epochs spanning 1900–2100. Regenerated from Horizons Q31 (OBSERVER, geocentric `500@399`, ecliptic of date, `ANG_FORMAT=DEG`, `extra_prec=YES`) via `https://ssd.jpl.nasa.gov/api/horizons.api`. Header records the exact query recipe; FNV-1a checksum pinned.
- **Regeneration seam** behind the existing `horizons-fetch` feature (a small binary/function). Network is used only at regeneration time; normal `cargo test` reads the committed CSV offline (matching the project's existing reference-data pattern).
- **Fail-closed gate** `validate_apparent_goldens()`: build an apparent chart per row via the packaged backend + `ChartEngine`, assert within per-row tolerance; fail closed on malformed rows, non-release-grade body, chart error, or tolerance exceedance. Per-body tolerances tightened toward arcsec; **Moon looser** (geocentric apparent Moon to arcsec is theory-limited; documented).
- **Existing J2000 corpus retained** as the geometric-core gate (validates `backend.position` mean J2000), relabeled to that role.

## Migration (behavior change)

The default chart output shifts by precession (~1.4° max at window edges) + corrections (~tens of arcsec). Required:
- Regenerate **all** snapshot goldens and pinned chart longitudes across the workspace (chart snapshot tests, CLI chart tests, any pinned-longitude assertions, doc examples).
- Update policy/docs: `pleiades-backend` apparentness + unsupported-modes text (apparent is now the chart-layer default; backends remain mean-only), README, `docs/time-observer-policy.md`, `PLAN.md`, `plan/stages/04-advanced-request-modes.md`.
- Reintroduce the Task 1 frame guard as a **regression test that now passes** (Sun apparent ~280.4° at 1950).

This golden migration is the largest single cost. The work is one coherent feature; if the migration proves too large for one plan, the implementation plan may split it into its own task cluster.

## Error handling

- Apparent pipeline stays fail-closed via `ApparentPlaceError` (`NonConvergentLightTime`, `MissingDistance`, `NonFiniteCorrection { stage }`, `StaleModelData { kind }`) and `ApparentLightTimeError<E>` (`Query(E)` / `Apparent(...)`).
- Precession non-finite → `NonFiniteCorrection { stage: "precession" }`.
- Stale nutation checksum → `StaleModelData { kind: "nutation" }` (fail-closed before use).
- Non-release-grade under default → graceful mean fallback (not an error).

## Testing

- **Unit:** precession vs a Meeus ch. 21 worked example (pinned tolerance, mirroring the nutation 0.03″ pin); nutation / aberration / light-time / orchestrator-combine (from the parked plan).
- **Integration:** apparent-of-date corpus vs Horizons Q31 to arcsec; existing J2000 core gate unchanged; frame-guard regression.
- **Workspace gate:** `cargo test --workspace`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo fmt --all --check`.

## Relationship to the parked plan

This spec supersedes the apparent-place-corrections plan and reuses most of its task content (crate scaffold + `fnv1a64`; `error`, `nutation`, `aberration`, `lighttime`, `provenance`, `policy` modules; CLI provenance line). It **adds**: the `precession` module, the precession stage in the orchestrator + provenance, the default-mode flip to apparent, Horizons-sourced corpus regeneration (replacing the placeholder goldens), and the workspace-wide golden migration.

## Out of scope

Gravitational light-deflection and the full untruncated nutation series remain omitted (absorbed by validation tolerance), as in the parked plan. Topocentric and native-sidereal modes remain future Phase-4 work.
