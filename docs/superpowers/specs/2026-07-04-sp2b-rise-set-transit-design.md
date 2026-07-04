# SP-2b · Rise/Set/Transit + Horizontal Coordinates — Design

Status: **draft — 2026-07-04**. Design proposed and reviewed in brainstorming;
awaiting written-spec review before handing to writing-plans.

This is the second slice of **SP-2**, the event-finding sub-project opened by
SP-1 (angles & sidereal time, 2026-07-01) and continued by SP-2a (longitude
crossings, 2026-07-03). SP-2b closes the observer-local event gaps where Swiss
Ephemeris ships a function:

- **SP-2a (done):** longitude crossings — `swe_solcross`, `swe_mooncross`,
  general geocentric body crossings, and heliocentric `swe_helio_cross`.
- **SP-2b (this spec):** rise/set/meridian transit + horizontal coordinates —
  `swe_rise_trans` (full-flag) and `swe_azalt`/`swe_azalt_rev`.
- **SP-2c (later spec):** local eclipse circumstances (observer-local
  visibility and contact timing).

SP-2c stays out of scope here. SP-2b extends the `pleiades-events` crate and
reuses the generic root-finder, the 1900–2100 TDB window clamp, and
`EventError` that SP-2a introduced.

## Context

pleiades is an **astronomy engine only**: it supplies
positions/houses/ayanamsas/eclipses/angles/crossings, and the application builds
the technique layer on top. Rise/set/transit and horizontal coordinates are
engine-level quantities ("when does body B rise for an observer?", "what is body
B's azimuth/altitude now?"), not astrological techniques.

Current state (verified in code):

- **The generic root-finder exists.** `pleiades-events/src/root.rs`
  (bracket-by-stepping + bisect to ~0.5 s) already backs SP-2a crossings.
  Rise/set is a root of `apparent_altitude(t) − h0`; transit is a root of the
  local hour angle. Both reuse this primitive unchanged.
- **Sidereal-time machinery exists** (SP-1): `greenwich_mean_sidereal_time_degrees`,
  `equation_of_equinoxes_degrees`, `sidereal_time(instant, observer_longitude)`
  (`crates/pleiades-apparent/src/sidereal.rs`) supply local apparent sidereal
  time → local hour angle `H = LST − RA`.
- **Apparent equatorial of-date exists:** `apparent_equatorial_of_date` and
  `true_obliquity_degrees` (`crates/pleiades-apparent/src/equatorial.rs`).
- **Topocentric corrections exist:** `topocentric_position` /
  `TopocentricPosition`, `ObserverGeocentric`, and diurnal parallax/aberration
  over a WGS84 ellipsoid (`crates/pleiades-apparent/src/topocentric.rs`,
  `parallax.rs`) — the horizontal parallax term rise/set needs, especially for
  the Moon.
- **`ObserverLocation`** (`crates/pleiades-types/src/observer.rs`) already
  carries `latitude`, `longitude`, optional `elevation_m` — the observer input.
- **Atmospheric refraction is explicitly *omitted* repo-wide.**
  `crates/pleiades-apparent/src/provenance.rs` and `lib.rs` document
  "atmospheric refraction omitted". This is the one genuinely new correction the
  slice must add, and flipping that provenance line is gated.
- **No horizontal (azimuth/altitude) transform exists yet.** `swe_azalt` has no
  analogue; this slice builds it.
- **No fixed-star position source exists.** `swe_rise_trans` accepts named stars;
  pleiades has no star catalog, so a bounded curated one is introduced here.

So SP-2b is **a horizontal-coordinate transform + a refraction correction + a
rise/set/transit root-finder + a curated fixed-star apparent-place path + a
parity gate**, most of it standing on infrastructure that already exists.

## Decisions captured

| Topic | Decision |
| --- | --- |
| SE functions targeted | `swe_rise_trans`/`swe_rise_trans_true_hor` (full-flag), `swe_azalt`, `swe_azalt_rev`. Star apparent place uses `swe_fixstar`-compatible default-flag semantics for the curated set. |
| Home crate | Extend **`pleiades-events`** (SP-2a's crate), generic over `B: EphemerisBackend`. New modules `horizontal`, `rise_trans`, `fixstar`; reuse `root.rs`, the 1900–2100 TDB window clamp, and `EventError`. |
| Refraction home | New **`refraction` module in `pleiades-apparent`** (a reusable correction, not an event). Both directions: Bennett (true→apparent) and Saemundsson (apparent→true), pressure/temperature scaled, pinned to SE's low-altitude branch behavior. Flips the "refraction omitted" provenance line. |
| Engine surface | Rename `CrossingEngine` → **`EventEngine`**; crossings, horizontal, and rise/set all become `EventEngine` methods. Keep a `#[deprecated]` `CrossingEngine` type alias for one cycle (one-time API-stability bump). |
| Feature scope | **Full `swe_rise_trans` flag parity:** events {rise, set, upper (meridian) transit, lower transit} × disc {center, upper-limb, lower-limb (`DISC_BOTTOM`), `FIXED_DISC_SIZE`} × refraction {on, off (`NO_REFRACTION`)} × `GEOCTR_NO_ECL_LAT` × `HINDU_RISING` (SE's composed flag) × configurable pressure/temperature × custom horizon altitude (`rise_trans_true_hor`). |
| Object scope | Release-grade bodies (Sun, Moon, Mercury–Pluto), an arbitrary supplied **ecliptic point** (the `GEOCTR_NO_ECL_LAT` "rising of a zodiac degree" case), and a **curated fixed-star set** (~30 astrologically-used stars). |
| Fixed-star source | Committed `fixstars-catalog` (J2000 ICRS RA/Dec + proper motion + parallax + radial velocity), values matching SE's `sefstars.txt` so parity is exact by construction. Apparent place via the existing precession/nutation/aberration pipeline. Byproduct: a small public `fixed_star_apparent(name, instant)`. |
| Time base | Native **TDB** results (matching the eclipse/crossings engines). SE `_ut` corpus rows converted once at generation. Civil/UT1 via the existing `pleiades-time` conversion. No ΔT/UT1 policy change. |
| Coverage window | Same hard **1900–2100 TDB** clamp. Out-of-window requests/scans fail closed. |
| Surfacing | `pleiades-events` stays a standalone crate users depend on directly (not re-exported through `pleiades-core`, matching eclipse/crossings precedent). New `rise-trans` and `azalt` CLI aliases routed through `pleiades-validate`'s render layer, exactly like the `crossings`/`eclipses` aliases. |
| Validation | New isolated `tools/se-rise-trans-reference`; committed `rise-trans-corpus` + manifest; fail-closed **two-tier `validate-rise-trans`** gate (self-consistency golden + SE parity), wired into `release-smoke`/`release-gate`; claim-tier ↔ evidence through the overclaim audit. |
| Versioning | New public surface + flipped refraction provenance → bump compatibility profile from `0.7.6`. `EventEngine` rename → bump API-stability profile from `0.2.1`. |

## Scope & boundaries

**In:**

- Horizontal coordinates: `swe_azalt` (ecliptic or equatorial input → azimuth +
  true altitude + apparent altitude) and `swe_azalt_rev` (horizontal → equatorial).
- A both-directions atmospheric **refraction** correction in `pleiades-apparent`.
- Rise / set / upper (meridian) transit / lower transit, full `swe_rise_trans`
  flag parity (disc center/upper-limb/lower-limb/fixed-size, refraction on/off,
  `GEOCTR_NO_ECL_LAT`, `HINDU_RISING`, custom horizon altitude,
  configurable pressure/temperature).
- Object targets: release-grade bodies, an arbitrary ecliptic point, and a
  curated fixed-star set with its committed catalog + apparent-place path.
- `EventEngine` rename (crossings preserved via deprecated alias).
- `rise-trans` / `azalt` CLI aliases through the validate render layer.
- Two-tier SE-parity gate `validate-rise-trans` over a committed corpus.

**Out (explicitly):**

- SP-2c material (local eclipse circumstances).
- A general fixed-star subsystem beyond the curated set (full `sefstars.txt`
  port is a later slice; the curated catalog is deliberately bounded).
- Sidereal-zodiac (ayanamsa) variants — targets are tropical/of-date like the
  rest of the engine; a caller converts before calling.
- Native heliocentric or non-topocentric "rise" notions — rise/set is inherently
  topocentric.
- Any change to the ΔT/UT1/time-scale policy beyond documenting the existing
  TDB-native behavior.
- Migrating other engines onto shared code beyond what already exists; the
  eclipse crate and `validate-eclipses` stay untouched.

## Architecture

### 1. Crate `pleiades-events` (extended)

Three new modules join `crossings`, all generic over the backend and reusing
`root.rs`, the window clamp, and `EventError`:

- `horizontal` — azalt / azalt_rev transforms.
- `rise_trans` — rise/set/transit root-finding.
- `fixstar` — curated fixed-star apparent place + committed catalog reader.

`CrossingEngine` is renamed `EventEngine`; a `#[deprecated] pub type
CrossingEngine<B> = EventEngine<B>;` alias preserves the SP-2a surface for one
cycle.

### 2. Refraction (`pleiades-apparent::refraction`)

```rust
/// True (geometric) altitude → apparent altitude (Bennett 1982), pressure/temp scaled.
pub fn apparent_from_true(true_alt_deg: f64, atmos: Atmosphere) -> f64;
/// Apparent altitude → true altitude (Saemundsson 1986 inverse), pressure/temp scaled.
pub fn true_from_apparent(apparent_alt_deg: f64, atmos: Atmosphere) -> f64;
```

- Bennett: `R = 1.02 / tan(h + 10.3/(h + 5.11))` arcmin, scaled by
  `pressure/1010 · 283/(273 + temperature)`.
- Saemundsson inverse for the true→apparent-consistent reverse.
- SE's low-altitude / below-horizon branch behavior (including `h < −5°`) is
  matched exactly and proven by the corpus — this is the one fiddly region.
- Flipping the "atmospheric refraction omitted" provenance line is gated by
  `validate-rise-trans`.

### 3. Horizontal coordinates (`pleiades-events::horizontal`)

```rust
pub struct Horizontal {
    pub azimuth: f64,           // degrees, SE convention (S=0 / configurable to match swe_azalt)
    pub true_altitude: f64,     // geometric
    pub apparent_altitude: f64, // refracted
}
pub enum HorizontalInput {
    Ecliptic(Longitude, Latitude),      // SE_ECL2HOR
    Equatorial(RightAscension, Declination), // SE_EQU2HOR
}

impl<B: EphemerisBackend> EventEngine<B> {
    pub fn horizontal(&self, input: HorizontalInput, observer: ObserverLocation,
                      atmos: Atmosphere, at: Instant) -> Result<Horizontal, EventError>;
    pub fn horizontal_to_equatorial(&self, azimuth_deg: f64, altitude_deg: f64, is_apparent: bool,
                      observer: ObserverLocation, atmos: Atmosphere, at: Instant)
                      -> Result<(RightAscension, Declination), EventError>; // swe_azalt_rev
}
```

- Path: input → apparent equatorial of-date (RA/Dec) via
  `apparent_equatorial_of_date` (with topocentric parallax for the observer) →
  local hour angle `H = LST(instant, observer.longitude) − RA` → az/alt by the
  standard spherical rotation with `observer.latitude` → apparent altitude via
  the refraction module. Azimuth convention pinned to `swe_azalt` during
  implementation.

### 4. Rise/set/transit (`pleiades-events::rise_trans`)

```rust
pub enum RiseSetEvent { Rise, Set, UpperTransit, LowerTransit }
pub enum DiscMode { Center, UpperLimb, LowerLimb }
pub enum RiseSetTarget {
    Body(CelestialBody),
    EclipticPoint(Longitude, Latitude), // GEOCTR_NO_ECL_LAT drives latitude handling
    FixedStar(&'static str),
}
pub struct Atmosphere { pub pressure_mbar: f64, pub temperature_c: f64 } // SE defaults 1013.25 / 15
impl Default for Atmosphere { /* 1013.25 mbar, 15 °C */ }

pub struct RiseSetOptions {
    pub disc: DiscMode,
    pub refraction: bool,          // NO_REFRACTION = false
    pub no_ecl_lat: bool,          // GEOCTR_NO_ECL_LAT
    pub hindu: bool,               // HINDU_RISING = DISC_CENTER | NO_REFRACTION | GEOCTR_NO_ECL_LAT
    pub fixed_disc_size: bool,     // FIXED_DISC_SIZE
    pub horizon_altitude_deg: Option<f64>, // rise_trans_true_hor
}

pub struct RiseSet { pub event: RiseSetEvent, pub instant: Instant /*TDB*/, pub target: RiseSetTarget }

impl<B: EphemerisBackend> EventEngine<B> {
    pub fn next_rise_set(&self, target: RiseSetTarget, event: RiseSetEvent,
        observer: ObserverLocation, atmos: Atmosphere, opts: RiseSetOptions, after: Instant)
        -> Result<Option<RiseSet>, EventError>;
    pub fn rise_sets_in_range(&self, target: RiseSetTarget, event: RiseSetEvent,
        observer: ObserverLocation, atmos: Atmosphere, opts: RiseSetOptions,
        start: Instant, end: Instant) -> Result<Vec<RiseSet>, EventError>;
}
```

**Standard altitude assembly** (the `h0` the root-finder solves
`apparent_altitude(target, t) = h0` for), matching SE's flag semantics:

```
h0 = − horizon_refraction        (dropped when refraction = false)
     ∓ semidiameter_term         (UpperLimb subtracts SD; LowerLimb adds; Center = 0)
     − horizon_dip(elevation_m)  (geometric dip from observer elevation)
     + horizon_altitude_deg      (custom raised horizon, when set)
```

- **Semidiameter:** Sun/Moon from apparent distance + physical radius; planets
  from SE's disc-radius table (small but nonzero for parity); stars = 0.
  `FIXED_DISC_SIZE` freezes SD at mean distance.
- **Parallax:** the Moon's horizontal parallax (~57′) dominates its `h0`;
  included for all bodies via `topocentric_position` / `ObserverGeocentric`.
- **`GEOCTR_NO_ECL_LAT`:** forces the target's ecliptic latitude to 0 (rising of
  the zodiac degree, not the body). Drives `EclipticPoint` and Hindu rising.
- **`HINDU_RISING`:** honored as SE's exact composition
  (`DISC_CENTER | NO_REFRACTION | GEOCTR_NO_ECL_LAT`).

**Root-finding:**

- **Rise/set:** step-scan `f(t) = apparent_altitude(target, t) − h0` over the day
  at a few-minute step (fine enough for the fast Moon and grazing events), bisect
  each sign change to ~0.5 s.
- **Transit:** root-find `H(t) = LST(t) − RA(t)` → 0 (upper) or ±12ʰ (lower)
  using the sidereal-time helpers.
- **Ill-conditioning:** near a grazing rise/set at high latitude,
  `d(alt)/dt → 0`, so the crossing time is sensitive (the retrograde-crossing
  story from SP-2a). Return the bracketed root, never NaN; the gate ceilings for
  such rows are documented looser in *time*.

### 5. Fixed-star apparent place (`pleiades-events::fixstar`)

- Committed `crates/pleiades-events/data/fixstars-catalog.csv` (~30 stars:
  Aldebaran, Regulus, Spica, Antares, Fomalhaut, Algol, Sirius, Betelgeuse,
  Rigel, Pollux, Deneb, Vega, Altair, Alcyone, Zuben Elgenubi/Eschamali,
  Bellatrix, Capella, Arcturus, Procyon, Alphard, Aselli/Praesepe, …), each row:
  SE-compatible name + J2000 ICRS RA/Dec + proper motion (mas/yr) + parallax
  (mas) + radial velocity (km/s). Values matched to SE's `sefstars.txt` so
  parity is exact by construction.
- Apparent place via the existing pipeline: space motion to epoch → precession
  (IAU-1976) → nutation (IAU-1980) → annual aberration → (topocentric diurnal
  parallax/aberration for azalt). No light-time / light-deflection for stars,
  matching `swe_fixstar` default flags.
- Unknown star name → `EventError::UnknownFixedStar` (fail closed, no placeholder).
- Byproduct: `pub fn fixed_star_apparent(name, instant) -> Result<...>` that a
  future full-catalog slice can grow.

### 6. Surfacing

- `pleiades-events` stays a standalone crate; users construct
  `EventEngine::new(backend)`. Not re-exported through `pleiades-core` (eclipse
  and crossings aren't either).
- New `rise-trans` and `azalt` CLI aliases routed through
  `pleiades-validate`'s render layer (`render/cli.rs`), mirroring the existing
  `crossings`/`eclipses` aliases — a render/report surface, not a core API.

### 7. SE reference extension (`tools/se-rise-trans-reference`)

New isolated, out-of-workspace crate (own `Cargo.lock`), like the existing SE
harnesses. Calls `swe_rise_trans`/`swe_rise_trans_true_hor`/`swe_azalt`/
`swe_azalt_rev` (and `swe_fixstar` for the curated stars) over a fixture set and
emits crossing/transit JD (converted to TDB for `_ut` rows) and az/alt values,
writing `rise-trans-corpus` + manifest (row counts, fnv1a64 checksums, SE
version 2.10.03). libclang is needed only to build the harness and regenerate
the corpus, never to run the gate or build the workspace (se-crossings /
se-lilith precedent).

### 8. Corpus (`crates/pleiades-validate/data/rise-trans-corpus`)

Committed CSV covering: objects {Sun, Moon, a planet, an ecliptic point, ≥2
stars} × events {rise, set, upper transit, lower transit} × a representative
flag spread {center / upper-limb / lower-limb, refraction on/off, hindu,
fixed-disc, custom horizon} × observer latitudes including mid-latitude, tropics,
and a **high-latitude circumpolar case that must return no event**. Manifest
records checksums + SE version. A clean checkout stays tool-free and validates
only committed values (de440/SE-corpus precedent).

### 9. The gate: `validate-rise-trans`

Fail-closed command in `pleiades-validate` (`rise_trans_validation.rs`), wired
into `release-smoke`/`release-gate` next to `validate-crossings`/
`validate-angles`. Two tiers, mirroring `validate-crossings`:

- **Tier 1 — self-consistency golden** (tool-free, always runs): azalt round-trip
  (`horizontal_to_equatorial(horizontal(x)) ≈ x`), rise/set symmetry, transit ⇒
  hour-angle-zero, refraction inverse round-trip. Sub-arcsec / sub-second
  ceilings.
- **Tier 2 — SE parity** over the committed corpus: **time residuals** for
  rise/set/transit (tight — target a few seconds; documented looser bands near
  grazing / high latitude and for the fast Moon) and **angle residuals** for
  azalt (arcsec). Ceilings live in a `thresholds` module mirroring the
  house/angles/crossings gates.
- Provenance: fnv1a64 checksum-drift + SE version pinned (SP-2a-FU precedent);
  claim-tier ↔ evidence carried through the overclaim audit.

## Data flow

```
fixtures (object, event, observer, flags, atmosphere, start_epoch)
  └─(offline, SE present)─> se-rise-trans-reference:
        swe_rise_trans / swe_rise_trans_true_hor / swe_azalt / swe_azalt_rev / swe_fixstar
        → event JD (→ TDB) and az/alt
        └─ write rise-trans-corpus + manifest
              └─> committed corpus (source of truth)
                     └─(runtime gate: validate-rise-trans)─>
                           Tier 1: pleiades self-consistency goldens
                           Tier 2: EventEngine::rise_sets_in_range / horizontal()
                                   compared vs SE within per-row ceilings
                                 └─> pass / fail (fail-closed)
```

## Error handling / fail-closed conditions

- **Circumpolar / never-rises:** `Ok(None)` — a real "no event in range" answer,
  distinct from `EventError`. The gate includes a case that must return `None`.
- **Ill-conditioned grazing events:** near-grazing rise/set at high latitude has
  `d(alt)/dt → 0`; return the bracketed root, not a NaN, with looser documented
  time ceilings for those rows.
- **Misuse / missing data:** out-of-window instant, unsupported body, missing
  `distance_au` for topocentric reconstruction, unknown fixed-star name, or a
  non-finite atmosphere → `EventError` (documented kind), never a placeholder.
- **Gate fails** on: missing slice, checksum/schema/provenance drift, any time
  or angle residual over its ceiling, a self-consistency round-trip breach, or a
  case that should be `None` returning an event (and vice versa).
- **Generation fails** only on SE-side problems (mirrors `validate-eclipses`/
  `validate-crossings`).

## Constraints

- **C1 — Pure-Rust workspace audit (hard).** The SE binding must never enter the
  published workspace lockfile (`workspace-audit` fails closed on `-sys`/`links`/
  `build.rs`). Satisfied by making `tools/se-rise-trans-reference` a new isolated
  crate with its own `Cargo.lock`, outside the workspace — no new in-workspace
  FFI. (`libclang-dev` + `LIBCLANG_PATH` needed only to build the harness and
  regenerate the corpus, never to run the gate or build the workspace.)
- **C2 — SE license.** Verification-only, non-shipping harness; reuse the
  existing `LICENSE-NOTES.md` posture from the other SE reference tools. The
  curated `fixstars-catalog` uses Hipparcos-derived astrometry (public source
  data), not SE-distributed files; provenance recorded in the catalog header.
- **C3 — Do not perturb passing gates.** The eclipse crate and
  `validate-eclipses`, plus `validate-crossings` and `validate-angles`, are left
  untouched and must still pass. The `CrossingEngine` rename ships with a
  deprecated alias so existing callers and doctests compile unchanged.
- **C4 — Refraction provenance flip is gated.** Turning on atmospheric refraction
  changes a documented "omitted" line; `validate-rise-trans` must prove the
  refraction path matches SE before the provenance/prose is updated.

## Compatibility / versioning

- New public surface (rise/set/transit + horizontal coords + curated fixed-stars
  + CLI aliases) and the flipped refraction provenance → bump the compatibility
  profile from `0.7.6`; note the rise/set/transit + horizontal-coordinate +
  curated fixed-star capability in README "current state". Add the entries to the
  compatibility profile with claim tiers tied to `validate-rise-trans` evidence,
  and let the overclaim audit enforce tier ↔ evidence ↔ profile ↔ prose
  agreement.
- The `CrossingEngine` → `EventEngine` rename is a deliberate one-cycle
  deprecation (aliased), so bump the API-stability profile from `0.2.1`. No other
  breaking change to existing public types.

## Testing

- Unit tests against inline goldens: azalt for a known body/observer/instant
  (true + apparent altitude); a Sun rise and set; upper and lower transit; a Moon
  rise (large parallax); an ecliptic-point rise (`GEOCTR_NO_ECL_LAT`); a fixed-star
  rise; a Hindu-rising case (composed flags); a lower-limb and a fixed-disc case.
- Self-consistency: azalt round-trip; rise/set symmetry; transit ⇒ hour-angle
  zero; refraction `apparent_from_true`/`true_from_apparent` round-trip.
- Fail-closed tests: out-of-window instant, unsupported body, missing
  `distance_au`, unknown fixed-star name, non-finite atmosphere → assert the
  documented error, not NaN; circumpolar case → assert `Ok(None)`.
- Doctest examples on every new public item (the crates doctest heavily).
- Integration: the `validate-rise-trans` gate over the committed corpus (both
  tiers); a manifest checksum-drift test.
- Regression guard: `validate-eclipses`, `validate-crossings`, and
  `validate-angles` still pass unchanged (proves the new modules + refraction
  addition + engine rename did not perturb existing engines/gates).

## Open items (confirm during planning/implementation)

1. **Azimuth convention** — pin `Horizontal.azimuth` to `swe_azalt`'s output
   convention (SE measures azimuth from south, westward) exactly, proven by the
   corpus.
2. **Refraction below-horizon branch** — match SE's exact behavior for
   `h < −5°` / deep-negative apparent altitudes (SE clamps/blends there); the
   corpus must include below-horizon rows to pin it.
3. **Semidiameter/disc-radius table** — reproduce SE's per-planet disc radii for
   limb events; confirm the values and the `FIXED_DISC_SIZE` mean-distance basis.
4. **Fixed-star astrometry values** — pin the curated catalog's RA/Dec/PM/parallax/
   RV to the same figures SE's `sefstars.txt` carries so `swe_fixstar` parity is
   exact; record source + epoch in the catalog header.
5. **Per-row time/angle ceilings** — set from measured SE-vs-pleiades residuals;
   tight for the general case, documented looser for grazing/high-latitude/fast-Moon
   rows, mirroring the two-tier crossings gate.
6. **Engine rename blast radius** — confirm the `CrossingEngine` → `EventEngine`
   rename + deprecated alias leaves all SP-2a callers, doctests, and the
   `crossings` CLI alias compiling unchanged.
7. **Corpus shape** — one `rise-trans.csv` with `object`/`event`/`flags` columns
   vs. per-event slices; pick to keep the schema clean and the manifest simple.
