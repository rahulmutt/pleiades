# SP-6 — Lunar Occultations (`swe_lun_occult_*` analogue)

Status: **draft — 2026-07-08**. Design proposed and reviewed in brainstorming;
awaiting written-spec review before handing to writing-plans.

Phase: Event-engine track (SP series), slice SP-6 — the first of the two
subsystem-level candidates queued after SP-5 (the other being central-path
cartography for solar eclipses). Occultations are the Moon-occults-a-target
sibling of the eclipse subsystem: a solar eclipse *is* the Moon occulting the
Sun, and SP-6 generalizes that local geometry to arbitrary occulted bodies
(planets as small discs, fixed stars as points).

## Summary

Add a Swiss-Ephemeris `swe_lun_occult_*` analogue: for the Moon occulting a
planet or a catalogued fixed star, compute the **local circumstances** at a given
time (`how`), search **forward/backward for the next locally-visible occultation**
at an observer (`when_loc`), and search for the **next occultation anywhere on
Earth** with its sub-lunar greatest-occultation point (`when_glob`).

The surface is an **engine function** on `EventEngine`, not new `CelestialBody`
variants, matching the rest of the SP series (crossings, rise/set, nodes/apsides,
pheno). Implementation is **Approach A**: a new `occult` module (plus internal
`conjunction` search and `contacts` root helpers) inside `pleiades-events`,
reusing the crate's existing `fixstar` (star apparent place), `semidiameter`
(Moon + planet discs), `horizontal` (az/alt), and `root` (bracket-then-bisect)
machinery, plus `pleiades-apparent`'s `topocentric_position` and `refraction`.
No new crate dependency is added.

## SE functions targeted

| SE function | This design method | Role |
| --- | --- | --- |
| `swe_lun_occult_how` | `EventEngine::occultation` | Local circumstances of a known occultation at a caller-supplied instant + observer. |
| `swe_lun_occult_when_loc` | `EventEngine::next_occultation` / `previous_occultation` | Forward/backward search for the next occultation *visible at a given place*. |
| `swe_lun_occult_when_glob` | `EventEngine::next_global_occultation` | Forward search for the next occultation *anywhere on Earth*, with the sub-lunar point where it is central/greatest. |

SE identifies the target by either a planet number (`ipl`) or a star name
(`starname`); this design's `OccultTarget` enum carries the same either/or.

## Decisions captured

| Topic | Decision |
| --- | --- |
| Operation scope | **Full trio: `how` + `when_loc` + `when_glob`.** |
| Target scope | **Planets Mercury–Pluto (small discs) + curated fixed stars (points).** Sun rejected (Moon-occults-Sun is a solar eclipse, SP-2c); Moon is the occulter. |
| Architecture | **Approach A — extend `pleiades-events` / `EventEngine` in place** (mirrors SP-2b/SP-4/SP-5). New `occult` module (+ internal `conjunction`/`contacts` helpers); reuse the crate's `fixstar`/`semidiameter`/`horizontal`/`root` and `pleiades-apparent`'s topocentric/refraction. |
| `when_loc` strategy | **Refine the shared conjunction walk per observer.** Walk Moon–target conjunctions, run the topocentric `how` at each, return the first with `any_phase_visible == true` — a strict refinement of the walk, no independent search to diverge. |
| `when_glob` strategy | Geocentric conjunction walk; an occultation is visible *somewhere* iff geocentric `min sep < s_moon + s_tgt + π_moon` (Moon horizontal parallax ~0.95°). Report the instant and the **sub-lunar point** of greatest occultation (analogous to the eclipse engine's sub-solar greatest-eclipse point). **Not** the full path polygon. |
| Contact geometry | Two-circle tangency: exterior C1/C4 at `sep = s_moon + s_tgt`, interior C2/C3 at `sep = s_moon − s_tgt` (present only for a fully-covered planet disc; a point star has C1/C4 only). |
| Time base | Native **TDB**, matching the SP-2a/2b/2c and eclipse engines. Same documented ΔT/UT1 caveat; no time-scale policy change. |
| Coverage window | Same hard **1900–2100 TDB** clamp inherited from `EventError`/`WINDOW_*`. |
| Refraction input | Explicit `Atmosphere` parameter (reuse `pleiades-apparent::Atmosphere`, SP-2b/2c); sets the horizon-visibility threshold. |
| Surfacing | `pleiades-events` stays the standalone event hub users depend on. New `occult` CLI alias through `pleiades-validate`'s render layer (matching `eclipse-local`/`rise-trans`). |
| Validation | New isolated `tools/se-occult-reference`; committed `occultations-corpus` + manifest (fnv1a64-guarded); fail-closed two-tier **`validate-occultations`** gate wired into `run_all_numeric_gates`; claim tier ↔ evidence through the overclaim audit. |
| Versioning | New public surface, **no rename** → bump compatibility profile `0.7.11 → 0.7.12`; **API-stability profile unchanged at `0.2.2`** (purely additive to `EventEngine`). |

## Scope & boundaries

**In:**

- **`how`** (`swe_lun_occult_how`): local circumstances of a lunar occultation of
  a target for one observer at a caller-supplied instant — contact times C1–C4 +
  local maximum, magnitude (diameter fraction) and obscuration (area fraction),
  target azimuth/altitude + visibility at each contact/max, and the observer's
  `occultation_type` (`Total`/`Grazing`/`Miss`). Full circumstances even when the
  target is below the horizon; only the *search* filters on visibility.
- **`when_loc`** (`swe_lun_occult_when_loc`): `next_occultation` /
  `previous_occultation` — forward/backward search returning the first occultation
  locally visible at the observer.
- **`when_glob`** (`swe_lun_occult_when_glob`): `next_global_occultation` — the
  next occultation anywhere on Earth, its greatest-occultation instant, the
  **sub-lunar point** where it is central/greatest, and whether a central
  occultation exists.
- Targets: planets Mercury–Pluto (as small discs) and the curated fixed-star
  catalog (`fixstar.rs`, as points).
- Fail-closed `validate-occultations` SE-parity gate + committed corpus + a new
  `tools/se-occult-reference` generator; `occult` CLI alias.

**Out (explicitly):**

- **Central-path cartography** — rendering the global occultation path polygon
  across Earth; SP-6 reports only the per-observer circumstances and the single
  sub-lunar greatest-occultation point. The path renderer is the separate
  deferred candidate.
- **Grazing-occultation limb profiles** — modelling the lunar limb's topographic
  profile for precise graze timing; SE does not model this precisely either and
  we have no limb DEM. Grazes are classified (`Grazing`) but not resolved to
  limb-mountain contact events.
- **Sun as target** — that is a solar eclipse, covered by SP-2c.
- **Non-lunar occultations** — planet–planet, planet–star, asteroid occultations;
  SE's `swe_lun_occult_*` API is Moon-only, and so is this slice.
- **ΔT/UT1/time-scale policy** changes — results stay TDB-native with the same
  documented caveat as SP-2b/2c.
- **Sidereal-zodiac (ayanamsa) variants** — outputs are tropical/of-date; a caller
  converts before/after.
- Any change to the **eclipse** subsystem, its gates, corpora, or window clamp.

**Key physics distinction the design commits to:** occultation contact instants
are genuinely **observer-dependent** (the Moon's diurnal parallax shifts it up to
~1° between geocentric and topocentric, moving contacts by minutes and changing
whether an occultation happens at all), so local contacts are found by
topocentric two-circle root-finding per observer. The **global** search works in
geocentric coordinates and reports the sub-lunar point where parallax pulls the
Moon most toward the target — it does not fabricate a per-observer result for the
whole Earth.

## Data model

New public types in `pleiades-events` (`occult` module), exported from `lib.rs`.
The local structure is deliberately parallel to SP-2c's
`LocalSolarCircumstances` (C1–C4 with the interior pair optional).

```rust
/// What the Moon is occulting.
pub enum OccultTarget {
    /// A planet, Mercury..=Pluto. Sun and Moon are rejected (Sun ⇒ solar eclipse;
    /// Moon is the occulter).
    Body(CelestialBody),
    /// A curated fixed-star catalog name (see `fixstar.rs`).
    Star(&'static str),
}

/// What this observer actually sees.
pub enum OccultationType {
    /// Target fully covered at maximum (point star hidden, or planet disc fully
    /// behind the Moon's limb).
    Total,
    /// The Moon's limb crosses the target but never fully covers it.
    Grazing,
    /// No contact for this observer (topocentric separation never small enough).
    Miss,
}

/// One observer-local contact event: its instant plus the target's horizontal
/// position and visibility there. A contact below the horizon is still timed
/// (instant present) but flagged `visible == false`, matching SE and SP-2c.
pub struct OccultationContact {
    pub instant: Instant,          // TDB
    pub altitude_degrees: f64,     // apparent (refracted) altitude of the target
    pub azimuth_degrees: f64,      // from SOUTH increasing WESTWARD, [0,360) — matches Horizontal / swe_azalt
    pub visible: bool,             // target above the horizon at this instant
}

/// Local circumstances of a lunar occultation of one target for one observer
/// (`how` / `when_loc`).
pub struct LocalOccultation {
    pub target: OccultTarget,
    pub occultation_type: OccultationType,           // what THIS observer sees
    pub maximum: OccultationContact,                 // instant of least topocentric separation
    pub magnitude: f64,                              // covered fraction of the target's DIAMETER at max (SE attr[0])
    pub obscuration: f64,                            // covered fraction of the target's disc AREA at max (SE attr[2]); star = 0/1
    pub first_contact: OccultationContact,           // C1  disappearance (exterior ingress)
    pub second_contact: Option<OccultationContact>,  // C2  fully hidden (planet disc only; None for a point star / graze)
    pub third_contact: Option<OccultationContact>,   // C3  begins to reappear (planet disc only)
    pub fourth_contact: OccultationContact,          // C4  reappearance (exterior egress)
    pub any_phase_visible: bool,                     // target above the horizon during any part of the event
}

/// Global circumstances (`when_glob`): time + the sub-lunar point where the
/// occultation is central/greatest. NOT the full path polygon.
pub struct GlobalOccultation {
    pub target: OccultTarget,
    pub maximum: Instant,              // TDB, instant of greatest global occultation
    pub sublunar_latitude: Latitude,   // sub-lunar point of maximum occultation
    pub sublunar_longitude: Longitude,
    pub central: bool,                 // a central occultation exists somewhere on Earth
    pub occultation_type: OccultationType, // best-case type over the globe
}
```

Design notes:

- **C1–C4 mirror SP-2c.** For a **point star**: C1 = disappearance, C4 =
  reappearance, C2/C3 = `None`. For a **planet disc**: all four present (C2 = disc
  fully behind the limb, C3 = disc starts to emerge). A **graze** fills only
  C1/C4 with `occultation_type = Grazing` (`Total` never reached).
- **Contacts below the horizon are still timed**, flagged `visible: false`;
  `any_phase_visible` is the "was any of it observable" predicate the `when_loc`
  search filters on — identical to SP-2c.
- **`magnitude`/`obscuration`** map to SE `swe_lun_occult_how`'s `attr[0]`/`attr[2]`
  (diameter fraction / area fraction); for a point star both are 0/1
  (unhidden/hidden).
- **All instants TDB**, 1900–2100 window, same documented ΔT caveat as SP-2b/2c.
- Reuses the existing `Instant`/`Latitude`/`Longitude`/`CelestialBody`; the five
  `Occult*` types are the only new value types.

## Architecture

### Crate `pleiades-events` (extended, Approach A)

A new `occult` module (plus internal `conjunction` search + `contacts` root
helpers) joins `crossings`/`rise_trans`/`horizontal`/`nod_aps`/`pheno`.
`EventEngine` gains occultation methods alongside its existing ones; existing
methods and types are untouched.

```rust
impl<B: EphemerisBackend> EventEngine<B> {
    // --- existing methods unchanged: crossings, rise/set, horizontal,
    //     nod_aps, pheno ---

    // --- SP-6: "how" — local circumstances for a target + observer at an instant ---
    pub fn occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,        // pleiades-apparent::Atmosphere (SP-2b)
        at: Instant,
    ) -> Result<LocalOccultation, EventError>;

    // --- SP-6: "when_loc" — next/previous occultation locally visible at observer ---
    pub fn next_occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        after: Instant,
    ) -> Result<Option<LocalOccultation>, EventError>;

    pub fn previous_occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        before: Instant,
    ) -> Result<Option<LocalOccultation>, EventError>;

    // --- SP-6: "when_glob" — next occultation anywhere on Earth ---
    pub fn next_global_occultation(
        &self,
        target: OccultTarget,
        after: Instant,
    ) -> Result<Option<GlobalOccultation>, EventError>;
}
```

Decisions:

- **`when_loc` refines the shared conjunction walk.** `next_occultation` walks
  Moon–target conjunctions forward, runs the topocentric `how` at each, and
  returns the first with `any_phase_visible == true`; `previous_occultation` walks
  backward. Local results are a strict refinement of the walk — no separate search
  to diverge (the Approach-A payoff). The window clamp and `EventError` are
  inherited unchanged. A bounded internal cap on how many conjunctions to inspect
  before yielding `None` is a logged (not silent) backstop.
- **`occultation` returns full circumstances even when not visible**; only the
  *search* filters on visibility.
- **`Atmosphere` is an explicit parameter**, reusing SP-2b/2c's
  `pleiades-apparent::Atmosphere`; it drives refraction, which sets the
  horizon-visibility threshold.
- **Backward compatibility:** purely additive to `EventEngine` and the crate's
  public exports; no rename. API-stability profile is unaffected.

### Reused machinery (no new corrections invented)

- `pleiades-apparent::topocentric_position` — diurnal parallax over WGS84; the
  term that makes occultation contacts observer-specific.
- `pleiades-apparent::refraction::{Atmosphere, apparent_from_true}` +
  `pleiades-events::horizontal` — the refracted-horizon threshold and the
  equatorial → horizontal (az/alt) rotation for visibility. `pleiades-events`
  already depends on `pleiades-apparent`, so **no new crate dependency is added**.
- `pleiades-events::fixstar` — star apparent place (occultation targets).
- `pleiades-events::semidiameter` — Moon + planet apparent semidiameters (the
  discs in the tangency geometry).
- `pleiades-events::root` — bracket-then-bisect root-finder (~0.5 s), backing
  contact/maximum roots.

The only genuinely new math is the **two-circle tangency roots** (planet
interior/exterior contacts) and the **sub-lunar-point projection**, both
closed-form given the existing samples; plus the two-circle **lens-area
obscuration** (shared in form with SP-2c).

## Algorithm

### Contact geometry (shared)

For a target and the Moon, let `sep(t)` be the center-to-center angular
separation, `s_moon(t)` the Moon's semidiameter, and `s_tgt(t)` the target's
semidiameter (0 for a star, a small disc for a planet):

- **Exterior contacts C1/C4:** roots of `sep − (s_moon + s_tgt) = 0` — the Moon's
  limb touches the target's edge (for a star, `sep = s_moon`: the
  disappearance/reappearance).
- **Interior contacts C2/C3:** roots of `sep − (s_moon − s_tgt) = 0`, present
  **only if** `min sep < s_moon − s_tgt` (planet fully behind the limb). For a
  point star `s_tgt = 0` so interior ≡ exterior → C2/C3 are `None`.
- **`occultation_type`:** `Total` if `min sep < s_moon − s_tgt`; `Grazing` if
  `s_moon − s_tgt ≤ min sep < s_moon + s_tgt`; `Miss` otherwise.
- **`magnitude`** = covered diameter fraction at max; **`obscuration`** = the
  two-circle lens-area fraction of the target's disc (planets), 0/1 for a star.

All roots use the `root.rs` bracket-then-bisect primitive to ~0.5 s, matching the
crossings/rise-set/eclipse precedent.

### Conjunction finder (the search core)

Occultations occur only near Moon–target longitude conjunctions (~monthly). The
walk steps time coarsely to bracket each local **minimum of `sep(t)`**, then
refines the minimum (root of `d/dt sep`, seeded from the conjunction). The step
size is bounded so the Moon (≈0.5°/h) cannot skip a conjunction — a few hours,
mirroring the eclipse syzygy walk. Each minimum is then tested against the contact
threshold.

### `occultation` (how)

Caller supplies target + observer + atmosphere + instant. Compute the
**topocentric** `sep`/`s_moon`/`s_tgt` around that instant, solve C1–C4 and the
maximum, and evaluate the target's topocentric az/alt + `visible` (via
`horizontal` + `apparent_from_true` refraction) at each contact/max. Returns full
circumstances even below the horizon; `Miss` when no contact.

### `next_occultation` / `previous_occultation` (when_loc)

Walk conjunctions forward/backward from the reference instant; at each, run the
topocentric `how`; return the first with `any_phase_visible == true`. A strict
per-observer refinement of the conjunction walk — no separate search to diverge.

### `next_global_occultation` (when_glob)

At each conjunction compute the **geocentric** `min sep`. An occultation is
visible *somewhere on Earth* iff `min sep < s_moon + s_tgt + π_moon`, where
`π_moon` (~0.95°) is the Moon's horizontal parallax — the maximum amount
topocentric shift can pull the Moon toward the target. The **sub-lunar point** at
the global-max instant (geographic lat/long directly beneath the Moon) is where
parallax pulls hardest → the central/greatest-occultation location, exactly
analogous to the sub-solar greatest-eclipse point the eclipse engine already
computes. `central = true` iff the topocentric `sep` at that sub-lunar point drops
below `s_moon − s_tgt`.

### Termination & error handling

- **Un-occultable stars terminate fast.** A star with `|ecliptic latitude| >
  ~6.6°` (Moon's max latitude ~5.3° + SD + parallax) can *never* be occulted
  (e.g. Sirius at −39°). A static pre-check returns `None` (`when_*`) / `Miss`
  (`how`) immediately, backed by a bounded conjunction cap (logged, not silent) as
  the backstop — so the search always terminates.
- **Rejections:** Sun or Moon as `OccultTarget::Body` → `EventError` (Sun ⇒ use
  the solar-eclipse path; Moon is the occulter); unknown star name → reuse
  `fixstar_entry`'s error; instants outside 1900–2100 → the inherited `WINDOW_*`
  clamp.
- **Fail-closed numerics:** `asin` domain clamps and non-finite guards per the
  crate convention (never NaN).

### Why this is correct and bounded

Occultation contacts are observer-dependent, so local contacts use topocentric
root-finding; the global search works geocentrically and reports the single
sub-lunar greatest-occultation point rather than fabricating a per-observer result
for the whole Earth. All work stays inside the 1900–2100 window and reuses
validated topocentric/refraction/star-apparent/semidiameter/root code — the only
new math is the two-circle tangency roots, the lens-area obscuration, and the
sub-lunar-point projection, all closed-form.

## Validation

### Reference tool

New isolated `tools/se-occult-reference` (mirrors
`tools/se-eclipse-local-reference`): a small Swiss-Ephemeris harness calling
`swe_lun_occult_when_loc` + `swe_lun_occult_how` + `swe_lun_occult_when_glob` over
a curated target × observer × epoch set, emitting a committed CSV corpus +
`MANIFEST.md` with a pinned SE version and an fnv1a64 checksum (the drift-guard
pattern of every other `se-*` corpus). SE `_ut` rows are converted to TDB once at
generation, matching the SP-2b/2c corpus convention.

### Corpus

Committed under `crates/pleiades-validate/data/occultations-corpus/`. A curated
set (~50–70 rows, on the order of the rise-trans/eclipse-local corpora) chosen to
exercise the branches:

- **Bright-star occultations** — Aldebaran, Regulus, Spica, Antares — from
  observers where the star is above vs below the horizon (C1/C4 only, C2/C3
  `None`).
- **Planetary occultations** — e.g. Venus/Jupiter/Saturn — with all four contacts
  (`Total`) and a `Grazing` disc case.
- **A non-occultable star** (Sirius, −39° ecliptic latitude) → asserts the
  fast-reject `Miss`/`None` termination path.
- **Global rows** exercising the sub-lunar point and `central` both true and
  false.

### Gate `validate-occultations`

Fail-closed, wired into `run_all_numeric_gates` alongside the other event gates,
with `occultations` / `occult-gate` aliases. Two-tier like the SP-2a/2b/2c gates;
ceilings live in a new `occult_thresholds.rs`, each set from *measured* residuals
(~1.4× measured maxima, the SP-2b convention), not guessed:

- **Tier 1 — self-consistency:** `C1 < max < C4`; `C2/C3` bracketed and present
  iff `Total` disc; `C2/C3 == None` for stars; magnitude/obscuration ∈ [0, 1] and
  mutually consistent (obscuration > 0 iff magnitude > 0); a committed engine
  golden column guards against silent drift.
- **Tier 2 — SE parity**, per-row ceilings roughly: contact/max instants ≤ a few
  seconds for well-conditioned geometry, widening toward graze/central-limit rows;
  magnitude ≤ ~0.01; obscuration ≤ ~0.01; azimuth/altitude reusing SP-2b's
  horizontal ceilings; `occultation_type` and `visible` exact (documented
  allowlist only if a knife-edge graze proves irreducible, mirroring the eclipse
  gates' single carve-out); global sub-lunar point ≤ a measured arcmin/km ceiling,
  `central` exact.

Measured residuals during implementation set the final published ceilings; the
values above are the design's expected envelope, to be tightened to evidence.

### Beyond the gate

- Integration invariants over the routing chain (mirroring the SP-5
  pheno-invariants test): the engine produces a consistent `LocalOccultation` for
  every catalogued backend path.
- Property tests: a star at high ecliptic latitude always yields `Miss`; ingress
  and egress contacts are symmetric about the maximum; `magnitude` monotone in
  `min sep`.

## CLI, versioning & docs

- **CLI.** New `occult` alias (plus `validate-occultations` / `occultations` /
  `occult-gate`) routed through `pleiades-validate`'s render layer, exactly like
  `eclipse-local`/`rise-trans`/`pheno`.
- **Versioning.** New public surface, no rename → **bump compatibility profile
  `0.7.11 → 0.7.12`**; **API-stability profile unchanged at `0.2.2`** (purely
  additive to `EventEngine` and the crate exports).
- **Docs & claims.** Extend the `pleiades-events` `lib.rs` scope block, update the
  crate README, add the new surface to the overclaim-audit claim tier ↔ evidence
  mapping, and update `README.md`, `PLAN.md`, and the `plan/status/*` files to
  mark **SP-6 done** and leave **central-path cartography** and **custom
  fictitious-body orbital elements** as the remaining event-engine candidates.

## Follow-up sub-projects (out of scope here, recorded for sequencing)

- **Central-path cartography** — rendering the global path of totality/annularity
  (solar eclipse) and the occultation path polygon across Earth; a global renderer
  complementary to SP-6's per-observer + single-sub-lunar-point slice.
- **Custom fictitious-body orbital elements** — user-supplied osculating elements
  beyond the committed `seorbel.txt` set (extends SP-3's `FictitiousBackend`).
