# SP-2c · Local (Per-Observer) Eclipse Circumstances — Design

Status: **draft — 2026-07-05**. Design proposed and reviewed in brainstorming;
awaiting written-spec review before handing to writing-plans.

This is the third slice of **SP-2**, the event-finding sub-project opened by
SP-1 (angles & sidereal time, 2026-07-01) and continued by SP-2a (longitude
crossings, 2026-07-03) and SP-2b (rise/set/transit + horizontal coordinates,
2026-07-05). SP-2c closes the observer-local gap in the eclipse subsystem:

- **SP-2a (done):** longitude crossings — `swe_solcross`, `swe_mooncross`,
  general geocentric body crossings, and heliocentric `swe_helio_cross`.
- **SP-2b (done):** rise/set/meridian transit + horizontal coordinates —
  `swe_rise_trans`, `swe_azalt`/`swe_azalt_rev`.
- **SP-2c (this spec):** local eclipse circumstances — `swe_sol_eclipse_when_loc`
  / `swe_sol_eclipse_how` and `swe_lun_eclipse_when_loc` / `swe_lun_eclipse_how`.

SP-2c extends the existing **`pleiades-eclipse`** crate (global/geocentric only
today) with per-observer circumstances, reusing the topocentric-parallax,
atmospheric-refraction, and horizontal-coordinate machinery built in SP-2b
(`pleiades-apparent`), and the crate's own shadow-geometry and Sun/Moon sampling.

## Context

pleiades is an **astronomy engine only**: it supplies
positions/houses/ayanamsas/eclipses/angles/crossings/rise-set, and the
application builds the technique layer on top. "Where and when is this eclipse
visible for an observer at this location, and what does it look like from there"
is an engine-level question, not an astrological technique.

Current state (verified in code):

- **The global geocentric eclipse engine exists.** `pleiades-eclipse`
  (`EclipseEngine`) computes, per eclipse: type, instant of greatest eclipse,
  magnitude, gamma, Saros series, eclipsed longitude, and (solar) sub-solar
  greatest-eclipse point. Its `lib.rs` scope block explicitly states
  "**Coverage:** global / geocentric only. No per-observer local circumstances."
  — the line SP-2c flips.
- **Shadow geometry + Sun/Moon sampling exist.** `geometry.rs`
  (`classify_solar`/`classify_lunar`, umbral/penumbral shadow-radius model,
  `SHADOW_INFLATION`) and `ephemeris.rs` (`sample_sun_moon` →
  `SunMoonSample { sun/moon_longitude/latitude_deg, *_distance_au }`, apparent
  Sun via `pleiades-apparent`) back the global engine.
- **Topocentric parallax exists** (Phase 4 + SP-2b): `pleiades-apparent`'s
  `topocentric_position` / `TopocentricPosition` applies diurnal parallax over a
  WGS84 ellipsoid — the correction that makes *solar* contact timing genuinely
  observer-dependent (the Moon shifts up to ~1° between geocentric and
  topocentric).
- **Horizontal coordinates + refraction exist** (SP-2b): `pleiades-events`'
  `Horizontal`/`horizontal` transform and `pleiades-apparent`'s
  `refraction::{Atmosphere, apparent_from_true, true_from_apparent}` supply
  az/alt and the refracted-horizon threshold rise/set already relies on.
- **`ObserverLocation`** (`pleiades-types`) carries `latitude`, `longitude`,
  optional `elevation_m` — the observer input, already used by houses/rise-set.
- **The generic root-finder exists** (SP-2a): bracket-by-stepping + bisect to
  ~0.5 s, backing crossings and rise/set. Contact-time finding reuses this
  primitive.
- **The SE-parity gate pattern exists.** `tools/se-rise-trans-reference` (and the
  sibling `se-*-reference` tools) generate committed, checksum-guarded corpora;
  fail-closed two-tier gates (`validate-rise-trans`, `validate-crossings`) wire
  into `run_all_numeric_gates` in `pleiades-validate/src/render/cli.rs`.

So SP-2c is **a topocentric contact-time root-finder + a two-circle
tangency/obscuration geometry + a horizon-visibility layer + a local-search
wrapper over the existing global walk + an SE-parity gate**, almost all of it
standing on infrastructure that already exists.

## Decisions captured

| Topic | Decision |
| --- | --- |
| Operation scope | **Full parity: both `when_loc` (local search) and `how` (local snapshot).** |
| Kind scope | **Both solar and lunar** local circumstances. |
| SE functions targeted | `swe_sol_eclipse_when_loc`, `swe_sol_eclipse_how`, `swe_lun_eclipse_when_loc`, `swe_lun_eclipse_how`. |
| Architecture | **Approach A — extend `pleiades-eclipse` + `EclipseEngine` in place** (mirrors SP-2b extending `pleiades-events`). New `local` module (+ internal `contacts` root helper); reuse SP-2b's topocentric/refraction/horizontal machinery and the crate's own `geometry`/`ephemeris`. |
| `when_loc` strategy | **Reuse the validated global walk.** `next/previous_local_eclipse` iterate `next_eclipse`/`previous_eclipse` and return the first eclipse whose local circumstances are visible — a strict refinement of the global engine, no independent syzygy search to diverge. |
| Solar contacts | Genuinely observer-dependent: found by **topocentric** two-circle tangency root-finding (C1/C4 external, C2/C3 internal), local maximum by minimizing topocentric separation. |
| Lunar contacts | **Global instants** (the Moon enters Earth's shadow at one time for everyone); the local layer adds horizon **visibility + az/alt** at those instants. Matches SE `swe_lun_eclipse_when_loc`. |
| Time base | Native **TDB**, matching `Eclipse::greatest_eclipse` and the SP-2a/2b engines. Same documented ΔT caveat; no ΔT/UT1 policy change. |
| Coverage window | Same hard **1900–2100 TDB** clamp inherited from the global engine and `EclipseError`. |
| Refraction input | Explicit `Atmosphere` parameter (reuse `pleiades-apparent::Atmosphere`); it sets the horizon-visibility threshold. |
| Surfacing | `pleiades-eclipse` stays a standalone crate users depend on directly. New `eclipse-local` CLI alias through `pleiades-validate`'s render layer (matching `eclipses`/`rise-trans`). |
| Validation | New isolated `tools/se-eclipse-local-reference`; committed `eclipses-local-corpus` + manifest (fnv1a64-guarded); fail-closed two-tier **`validate-eclipses-local`** gate wired into `run_all_numeric_gates`; claim-tier ↔ evidence through the overclaim audit. |
| Versioning | New public surface, **no rename** → bump compatibility profile `0.7.7 → 0.7.8`; **API-stability profile unchanged at `0.2.2`** (purely additive). |

## Scope & boundaries

**In:**

- **Solar local** (`swe_sol_eclipse_when_loc` / `swe_sol_eclipse_how`): local
  contact times C1–C4 + local maximum; local magnitude (diameter fraction) and
  obscuration (area fraction); Sun azimuth/altitude at each contact/max;
  visibility (Sun above the horizon); central-path membership (what the observer
  actually sees: total/annular/hybrid vs partial-only).
- **Lunar local** (`swe_lun_eclipse_when_loc` / `swe_lun_eclipse_how`):
  penumbral P1/P4, umbral U1/U4, totality U2/U3 contact instants; umbral and
  penumbral magnitude; Moon azimuth/altitude; visibility (Moon above the horizon)
  at each contact.
- `when_loc` search: `next_local_eclipse` / `previous_local_eclipse` (forward and
  backward), reusing the global walk and returning the first locally-visible
  eclipse.
- `how` snapshot: `local_circumstances(eclipse, observer, atmosphere)` — full
  circumstances even when not visible; only the *search* filters on visibility.
- Fail-closed `validate-eclipses-local` SE-parity gate + committed corpus + a new
  `tools/se-eclipse-local-reference` generator; `eclipse-local` CLI alias.

**Out (explicitly):**

- Lunar/planetary/stellar **occultations** (`swe_lun_occult_*`) — not eclipses.
- Any change to the **global** geocentric engine, its `validate-eclipses` gate,
  the NASA-canon corpus, or the 1900–2100 TDB window clamp.
- Central-path **cartography** (rendering the path of totality across Earth) —
  this slice is per-observer, not a global path renderer.
- ΔT/UT1/time-scale **policy** changes — results stay TDB-native with the same
  documented caveat as SP-2b.
- Sidereal-zodiac (ayanamsa) variants — outputs are tropical/of-date; a caller
  converts before/after.
- Migrating other engines onto shared code beyond what already exists; the
  crossings/rise-set engines and their gates stay untouched.

**Key physics distinction the design commits to:** *solar* contact instants are
genuinely observer-dependent (lunar diurnal parallax shifts the Moon up to ~1°,
moving contacts by many minutes and changing central-path membership), so they
are found by topocentric root-finding per observer. *Lunar* umbral/penumbral
contact instants are essentially **global** (the Moon enters Earth's shadow at
one time for everyone; topocentric parallax perturbs them by only seconds and SE
treats them as geocentric); the "local" content there is **horizon visibility +
az/alt**, not distinct contact instants. The design encodes both correctly rather
than fabricating per-observer lunar contact instants that do not physically
differ.

## Data model

New public types in `pleiades-eclipse` (`local` module), exported from `lib.rs`:

```rust
/// One observer-local contact event: its instant plus the eclipsed body's
/// horizontal position there. A contact that happens below the horizon is still
/// timed (instant present) but flagged `visible == false`, matching SE.
pub struct LocalContact {
    pub instant: Instant,              // TDB
    pub altitude_degrees: f64,         // apparent (refracted) altitude of the body
    pub azimuth_degrees: f64,          // from SOUTH increasing WESTWARD, [0,360) — matches SP-2b Horizontal / swe_azalt
    pub visible: bool,                 // body above the horizon at this instant
}

/// Local circumstances of a solar eclipse for one observer.
pub struct LocalSolarCircumstances {
    pub local_type: SolarEclipseType,          // what THIS observer sees
    pub maximum: LocalContact,                 // instant of local greatest eclipse
    pub magnitude: f64,                        // covered fraction of the Sun's DIAMETER at local max
    pub obscuration: f64,                      // covered fraction of the Sun's AREA at local max
    pub first_contact: LocalContact,           // C1  partial begins
    pub second_contact: Option<LocalContact>,  // C2  total/annular begins (central path only)
    pub third_contact: Option<LocalContact>,   // C3  total/annular ends (central path only)
    pub fourth_contact: LocalContact,          // C4  partial ends
    pub any_phase_visible: bool,               // Sun above horizon during at least part of the eclipse
}

/// Local circumstances of a lunar eclipse for one observer.
/// Contact instants are global (shared by all observers); the local content is
/// horizon visibility + the Moon's az/alt at each contact.
pub struct LocalLunarCircumstances {
    pub eclipse_type: LunarEclipseType,        // Penumbral/Partial/Total (same as global)
    pub maximum: LocalContact,
    pub umbral_magnitude: f64,
    pub penumbral_magnitude: f64,
    pub penumbral_begin: LocalContact,         // P1
    pub partial_begin: Option<LocalContact>,   // U1 (absent for penumbral-only)
    pub total_begin: Option<LocalContact>,     // U2 (total only)
    pub total_end: Option<LocalContact>,       // U3 (total only)
    pub partial_end: Option<LocalContact>,     // U4 (absent for penumbral-only)
    pub penumbral_end: LocalContact,           // P4
    pub any_phase_visible: bool,               // Moon above horizon during at least part of the eclipse
}

/// Tagged local result returned by the search / how methods.
pub enum LocalCircumstances {
    Solar(LocalSolarCircumstances),
    Lunar(LocalLunarCircumstances),
}
```

Design notes:

- **`LocalContact` carries az/alt + visibility uniformly** so every event reports
  where in the sky it is and whether it is above the horizon — the whole point of
  "local."
- **Solar `local_type` may differ from the global `Eclipse::eclipse_type`**: a
  globally total eclipse can be partial-only for a given observer. This field is
  what the observer actually sees; `Partial` when the observer is outside the
  central path.
- **Lunar reuses the global `LunarEclipseType`** unchanged, because the shadow
  immersion class is global; only visibility is local.
- Contacts below the horizon are **still timed** but flagged `visible: false`;
  `any_phase_visible` is the quick "was any of it observable" answer and the
  predicate the search filters on.
- All instants are **TDB**, consistent with `Eclipse::greatest_eclipse` and the
  SP-2a/2b engines.
- The types reuse the existing `SolarEclipseType` / `LunarEclipseType` /
  `Instant`; `LocalContact`/`LocalSolarCircumstances`/`LocalLunarCircumstances`/
  `LocalCircumstances` are the only new value types.

## Architecture

### Crate `pleiades-eclipse` (extended, Approach A)

A new `local` module (plus an internal `contacts` root helper) joins
`engine`/`ephemeris`/`geometry`/`saros`/`syzygy`/`types`. `EclipseEngine` gains
local methods alongside its existing global ones; existing methods and types are
untouched.

```rust
impl<B: EphemerisBackend> EclipseEngine<B> {
    // --- existing global methods unchanged: next_eclipse, previous_eclipse,
    //     eclipses_in_range ---

    // --- SP-2c: "how" — local circumstances for a known eclipse + observer ---
    pub fn local_circumstances(
        &self,
        eclipse: &Eclipse,
        observer: &ObserverLocation,
        atmosphere: Atmosphere,        // pleiades-apparent::Atmosphere (SP-2b)
    ) -> Result<LocalCircumstances, EclipseError>;

    // --- SP-2c: "when_loc" — next/previous eclipse locally visible at observer ---
    pub fn next_local_eclipse(
        &self,
        after: Instant,
        observer: &ObserverLocation,
        filter: EclipseFilter,
        atmosphere: Atmosphere,
    ) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError>;

    pub fn previous_local_eclipse(
        &self,
        before: Instant,
        observer: &ObserverLocation,
        filter: EclipseFilter,
        atmosphere: Atmosphere,
    ) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError>;
}
```

Decisions:

- **`when_loc` reuses the global walk.** `next_local_eclipse` iterates
  `next_eclipse` and returns the first eclipse whose `local_circumstances` yields
  `any_phase_visible == true`; `previous_local_eclipse` walks backward. Local
  results are therefore a strict refinement of the validated global engine — no
  separate syzygy search to diverge (the Approach-A payoff). The window clamp and
  `EclipseError` are inherited unchanged. A bounded internal cap on how many
  global eclipses to inspect before yielding `None` guards against pathological
  loops and is logged, not silent (there is always a locally-visible eclipse well
  within the window, so the cap is a backstop).
- **`local_circumstances` returns full circumstances even when not visible**;
  only the *search* filters on visibility. A caller can ask "what were the local
  circumstances of the 2017 eclipse from London" (mostly below-horizon) and still
  get timed contacts with `visible: false`.
- **`Atmosphere` is an explicit parameter**, reusing SP-2b's
  `pleiades-apparent::Atmosphere`; it drives refraction, which sets the
  horizon-visibility threshold (`h0 = −refraction(0)` for the relevant limb).
  Callers pass the standard atmosphere for the common case.
- **Backward compatibility:** purely additive to `EclipseEngine` and the crate's
  public exports; no rename. API-stability profile is unaffected.

### Reused machinery (no new corrections invented)

- `pleiades-apparent::topocentric_position` — diurnal parallax over WGS84; the
  term that makes solar contacts observer-specific.
- `pleiades-apparent::{sidereal_time, true_obliquity_degrees}` +
  `refraction::{Atmosphere, apparent_from_true}` — local apparent sidereal time,
  obliquity of date, and the refracted-horizon threshold. `pleiades-eclipse`
  already depends on `pleiades-apparent`, so **no new crate dependency is added**;
  the small equatorial → horizontal (az/alt) rotation is inlined with the exact
  formula SP-2b uses (azimuth from south, westward), rather than taking a
  dependency on `pleiades-events` for its `Horizontal` type. This matches how
  SP-2b itself inlines that rotation in both `horizontal.rs` and `rise_trans.rs`.
- `pleiades-eclipse::geometry` shadow-radius model + `ephemeris::sample_sun_moon`
  — Sun/Moon apparent samples and umbral/penumbral radii.
- SP-2a bracket-then-bisect root-finder — contact/maximum roots to ~0.5 s.

The only genuinely new math is the **two-circle tangency roots** and the
**lens-area obscuration**, both closed-form given the existing samples.

## Algorithm

### Solar contacts (topocentric, observer-dependent)

For an observer, define the topocentric center-to-center separation `sep(t)`
between Sun and Moon and their topocentric semidiameters `s_sun(t)`, `s_moon(t)`
(both scale with topocentric distance; the Moon's is the term parallax moves
most), using `topocentric_position` on the existing Sun/Moon samples:

- **C1 / C4** are roots of `sep(t) − (s_sun + s_moon) = 0` (external tangency),
  bracketing the local maximum.
- **C2 / C3** are roots of `sep(t) − |s_moon − s_sun| = 0` (internal tangency) and
  **exist only if** `min sep < |s_moon − s_sun|` (observer inside the central
  path); otherwise `None`.
- **Local maximum** is the minimum of `sep(t)` — found as the root of
  `d/dt sep(t)` (or golden-section minimization) seeded from the global
  greatest-eclipse instant ± ~3 h.
- **Local magnitude** at max: `(s_sun + s_moon − sep) / (2·s_sun)` (diameter
  fraction). **Obscuration**: the standard two-circle lens-area fraction of the
  Sun's disk. **`local_type`**: `Total`/`Annular` from the sign of
  `s_moon − s_sun` at max when central (Hybrid preserved from the global type only
  where the observer straddles the total/annular limit), else `Partial`.
- **Az/alt + visibility** at each contact/max: topocentric horizontal of the Sun
  via the SP-2b transform + `apparent_from_true` refraction; `visible =
  apparent_altitude > 0`. `any_phase_visible` is true if the Sun is above the
  horizon over any sub-interval of `[C1, C4]`.

All roots use the bracket-then-bisect primitive to ~0.5 s, matching the
crossings/rise-set precedent.

### Lunar contacts (global instants, local visibility)

The six shadow contacts (P1, U1, U2, U3, U4, P4) are roots of the **geocentric**
Moon-center-to-shadow-axis distance crossing the penumbral/umbral radii —
reusing the existing `geometry` shadow-radius model, computed **once per
eclipse**, independent of observer. `U1/U2/U3/U4` are `None` for a
penumbral-only eclipse (and `U2/U3` for a partial-only umbral eclipse). The
**local** layer then evaluates the Moon's topocentric az/alt + `visible` flag at
each of those instants and sets `any_phase_visible`. This matches SE:
`swe_lun_eclipse_when_loc` returns the same contact instants regardless of
location; only visibility changes.

### Why this is correct and bounded

Solar needs topocentric root-finding because parallax reshapes the geometry per
observer; lunar does not, so the design does not fabricate per-observer lunar
contact instants. Both stay inside the 1900–2100 window and reuse validated
sampling / topocentric / refraction code — the only new math is the two-circle
tangency roots and the lens-area obscuration, both closed-form.

## Validation

### Reference tool

New isolated `tools/se-eclipse-local-reference` (mirrors
`tools/se-rise-trans-reference`): a small Swiss-Ephemeris harness calling
`swe_sol_eclipse_when_loc` + `swe_sol_eclipse_how` and `swe_lun_eclipse_when_loc`
+ `swe_lun_eclipse_how` over a curated observer × epoch set, emitting a committed
CSV corpus + `MANIFEST.md` with a pinned SE version and an fnv1a64 checksum
(same drift-guard pattern as every other `se-*` corpus). SE `_ut` rows are
converted to TDB once at generation, matching the SP-2b corpus convention.

### Corpus

Committed under `crates/pleiades-validate/data/eclipses-local-corpus/`. A curated
set (~50–70 rows, on the order of the rise-trans corpus, not the 900-row global
canon) spanning:

- Several well-known solar eclipses (e.g. 2017-08-21, 2024-04-08, an annular, a
  hybrid) from multiple observers: inside the central path (C2/C3 present),
  partial-only (C2/C3 absent), and below-horizon (all contacts `visible: false`).
- Lunar eclipses (total, partial, penumbral) from observers where the Moon is up
  vs down at contact.

Chosen to exercise C2/C3 present-vs-absent, visibility both ways, both
hemispheres, and both `local_type == Partial` (outside path) and total/annular
(inside path).

### Gate `validate-eclipses-local`

Fail-closed, wired into `run_all_numeric_gates` alongside `validate-rise-trans`,
with an `eclipses-local-gate` alias. Two-tier like the SP-2a/2b gates; ceilings
live in a new `eclipse_local_thresholds.rs` mirroring `rise_trans_thresholds.rs`,
each set from *measured* residuals (SP-2b precedent), not guessed:

- **Tier 1 — self-consistency:** contacts bracket the maximum
  (C1 < max < C4; P1 ≤ U1 ≤ U2 ≤ max ≤ U3 ≤ U4 ≤ P4 where present); solar
  magnitude/obscuration in `[0, 1]` and consistent (obscuration ≥ 0 iff
  magnitude > 0); a committed engine golden column guards against silent drift.
- **Tier 2 — SE parity**, per-row ceilings roughly: contact/max instants ≤ a few
  seconds for well-conditioned geometry, widening toward grazing / central-limit
  rows; magnitude ≤ ~0.01; obscuration ≤ ~0.01; azimuth/altitude reusing SP-2b's
  horizontal ceilings (azimuth ≤ ~0.2″-class, altitude ≤ ~0.1″-class); `visible`
  and solar `local_type` exact (with a documented allowlist only if a knife-edge
  central-limit row proves irreducible, mirroring the global gate's single
  hybrid/annular allowlist — time/magnitude/visibility still enforced there).

Measured residuals during implementation set the final published ceilings
(~1.4× measured maxima, matching the rise-trans convention); the values above are
the design's expected envelope, to be tightened to evidence.

## CLI, versioning & docs

- **CLI.** New `eclipse-local` alias (plus `validate-eclipses-local` /
  `eclipses-local-gate`) routed through `pleiades-validate`'s render layer,
  exactly like `rise-trans`/`eclipses`.
- **Versioning.** New public surface, no rename → **bump compatibility profile
  `0.7.7 → 0.7.8`**; **API-stability profile unchanged at `0.2.2`** (purely
  additive to `EclipseEngine` and the crate exports).
- **Docs & claims.** Flip the `pleiades-eclipse` `lib.rs` scope block ("No
  per-observer local circumstances" → the new local coverage), update the crate
  README, add the new surface to the overclaim-audit claim tier ↔ evidence
  mapping, and update `README.md`, `PLAN.md`, and the `plan/status/*` files to
  mark SP-2c done and leave **SP-3** as the remaining event-engine slice.

## Follow-up sub-projects (out of scope here, recorded for sequencing)

- **SP-3** — not yet scoped in detail; the final event-engine slice after SP-2c.
- **Occultations** (`swe_lun_occult_when_loc` / `swe_lun_occult_how`) — Moon–star
  and Moon–planet occultations; a distinct subsystem from eclipses, deferred.
- **Central-path cartography** — rendering the global path of totality/annularity
  across Earth (a global renderer, complementary to this per-observer slice).
