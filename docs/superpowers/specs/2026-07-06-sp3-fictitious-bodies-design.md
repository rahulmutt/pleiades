# SP-3 — Fictitious (Hypothetical) Bodies

Phase: Event-engine track (SP series), slice SP-3 — the final scoped slice of the
three-sub-project arc that closes the engine-layer gaps for which Swiss Ephemeris
*ships a function*.

## Summary

Add the full default `seorbel.txt` fictitious-body set — the Uranian/Hamburg-school
planets, Transpluto, Vulcan, and the other hypothetical/historical bodies Swiss
Ephemeris computes through `swe_calc()` with body numbers ≥ `SE_FICT_OFFSET` (40) —
as first-class `CelestialBody` variants served by a new `FictitiousBackend`. Because
they enter through the normal backend boundary, they flow through the existing chart
pipeline (apparent place, topocentric, sidereal/ayanamsa, houses) exactly like the
major planets, with no new corrections written.

These bodies do not physically exist; they have no astronomy use and no ground truth.
"Correct" means **reproduces SE's `seorbel.txt`-driven definition to parity**. Each is
an *unperturbed* Kepler orbit propagated from committed osculating elements — the same
model SE uses — so parity is limited only by the accuracy of the shared Sun/Earth
position the transform reuses.

## Why this is SP-3

The SP arc (recorded in `docs/superpowers/specs/2026-07-01-sp1-angles-and-sidereal-time-design.md`)
scoped SP-3 as **new body sources**: fixed stars (`swe_fixstar`), hypothetical/fictitious
bodies from orbital elements, and optionally `swe_pheno` / planetary `swe_nod_aps`.

The fixed-star headline was already delivered by SP-2b: `pleiades-events::fixed_star_apparent`
ships a 35-star curated catalog carrying full astrometry (proper motion, parallax, radial
velocity) with apparent place matched to SE's `sefstars.txt`. The remaining SP-3 content is
therefore the *other* SE-ships-a-function gaps. Of those, **fictitious bodies are the one
that is astrology-or-nothing**: they exist only because astrologers use them (Uranian
astrology / cosmobiology, esoteric Vulcan, Transpluto), and supplying them is still an
*engine* primitive (positions from orbital elements), so it fits the library's stated
"astronomy engine only" posture. `swe_pheno` and `swe_nod_aps` are astronomy-flavored with
thin astrological payoff and are deferred to later slices (see Non-goals).

## SE functions targeted

- `swe_calc()` / `swe_calc_ut()` for the fictitious body numbers ≥ `SE_FICT_OFFSET` (40),
  i.e. the bodies defined in SE's default `seorbel.txt`.

Bodies (SE numbers 40–58, ~19 total):

| SE # | Name | Center | Notes |
|---|---|---|---|
| 40 | Cupido | helio | Uranian/Hamburg (Witte) |
| 41 | Hades | helio | Uranian/Hamburg (Witte) |
| 42 | Zeus | helio | Uranian/Hamburg (Sieggrün) |
| 43 | Kronos | helio | Uranian/Hamburg (Sieggrün) |
| 44 | Apollon | helio | Uranian/Hamburg (Sieggrün) |
| 45 | Admetos | helio | Uranian/Hamburg (Sieggrün) |
| 46 | Vulkanus | helio | Uranian/Hamburg (Sieggrün) |
| 47 | Poseidon | helio | Uranian/Hamburg (Sieggrün) |
| 48 | Isis (Transpluto) | helio | |
| 49 | Nibiru | helio | |
| 50 | Harrington | helio | |
| 51 | Neptune (Leverrier) | helio | historical pre-discovery prediction |
| 52 | Neptune (Adams) | helio | historical pre-discovery prediction |
| 53 | Pluto (Lowell) | helio | historical pre-discovery prediction |
| 54 | Pluto (Pickering) | helio | historical pre-discovery prediction |
| 55 | Vulcan | helio | intramercurial; SE applies a special-form element set |
| 56 | White Moon (Selena) | **geo** | geocentric orbit around Earth |
| 57 | Proserpina | helio | |
| 58 | Waldemath | **geo** | hypothetical second Earth moon; geocentric orbit |

The exact center (heliocentric vs geocentric), reference frame, and element coefficients
for each body are transcribed from SE's `seorbel.txt`; the table above records the two known
geocentric bodies as the cases that exercise the second computation path. Any transcription
error is caught by the `validate-fictitious` SE-parity gate.

## Design

### 1. Crate & structure

New crate **`pleiades-fict`**, mirroring `pleiades-elp` (pure math + `backend.rs`):

- **Element table** — parsed once at build time from a committed CSV
  (`crates/pleiades-fict/data/fictitious-elements.csv`), the same pattern as
  `pleiades-events/data/fixstars-catalog.csv`.
- **Kepler core** — solve Kepler's equation `E − e·sinE = M` by Newton iteration,
  → true anomaly → position in the orbital plane → rotate by argument of perihelion ω,
  inclination i, and longitude of ascending node Ω → Cartesian position in the elements'
  reference frame. Analytic velocity is computed alongside position from the same orbital
  state (see Motion).
- **`FictitiousBackend<S: EphemerisBackend>`** — holds a Sun-source backend `S`, implements
  the existing `EphemerisBackend` trait (`metadata` / `supports_body` / `position`), and
  routes into the standard provider chain. It obtains Earth's heliocentric position by
  negating the Sun's geocentric position from `S` — the "reuse the existing backend" pattern
  SP-2c used (`next_local_eclipse` reused the global eclipse walk). No new ephemeris
  dependency is introduced.

Crate dependencies: `pleiades-types`, `pleiades-backend` (for the trait + result types), and
whatever math/frame helpers already exist for precession/frame rotation (reused from
`pleiades-apparent` where applicable). `pleiades-fict` carries **no** ephemeris data of its own.

### 2. Body model

- New `CelestialBodyClass::Fictitious` (the enum is `#[non_exhaustive]`, so this is additive).
- ~19 new `CelestialBody` variants named to match SE (Cupido … Waldemath). `CelestialBody` is
  `#[non_exhaustive]`, so adding variants is **non-breaking**; the API-stability profile is
  unchanged.
- `supports_body` on `FictitiousBackend` returns true exactly for the `Fictitious`-class
  variants; all other backends return false for them, so routing is unambiguous.

### 3. Computation & frame

- **Time-dependent elements.** SE's `seorbel.txt` format allows each element to be a
  polynomial in time (`c₀ + c₁·t + c₂·t²`, `t` in the body's own epoch units): the Uranian
  planets propagate mean anomaly linearly with mean motion, while some bodies precess their
  node/perihelion. The committed table encodes these coefficients so we match SE across the
  window, not merely at a static epoch.
- **Two centering paths.**
  - *Heliocentric bodies* (most): Kepler → heliocentric ecliptic Cartesian, then
    `geocentric = body_helio − earth_helio`, where `earth_helio = − sun_geocentric` from the
    Sun-source backend.
  - *Geocentric-orbit bodies* (White Moon/Selena, Waldemath): Kepler directly around Earth;
    the result is already geocentric, so the Earth-position transform is skipped.
- **Frame.** Each body's elements carry a reference-frame tag (J2000 / B1950 / of-date, per
  `seorbel.txt`). The Cartesian result is rotated to the **J2000 mean ecliptic** so the
  backend boundary stays mean, geometric, and geocentric in J2000 — identical to every other
  first-party backend — and passes the `validate-frame-consistency` keystone gate. Apparent
  place, topocentric correction, sidereal/ayanamsa conversion, and house placement are then
  all supplied by the existing chart layer, unchanged.
- **Motion.** Analytic Kepler velocity is derived from the same orbital state at negligible
  cost and is exact for an unperturbed orbit, so `position()` returns `Motion = Derived`
  (ecliptic longitude/latitude/radial speed) rather than leaving motion unavailable. This
  keeps fictitious bodies consistent with the packaged backend's motion contract.

### 4. Claim tier

Fictitious bodies have **no physical ground truth**. They are exposed as **`ReleaseGrade`**
with `AccuracyClass::Exact` and an evidence string that states the claim is *definitional*:

> "Definitional: unperturbed Kepler orbit from committed `seorbel.txt` elements; SE
> `swe_calc` parity via `validate-fictitious` (bodies ≥ 40)."

This reuses the existing three-tier claim model (`ReleaseGrade` / `Constrained` /
`Approximate`) and its `compat-claims-audit` machinery rather than introducing a new
`Definitional` tier. The evidence wording carries the honesty: the release-grade claim is
about *reproducing the definition to parity*, not about physical accuracy against nature.
The overclaim-audit claim↔evidence↔profile↔prose mapping is extended to include the new
bodies with this framing.

### 5. Elements data

A **native CSV table** (`crates/pleiades-fict/data/fictitious-elements.csv`) transcribes
`seorbel.txt`'s coefficients: per body — SE number, name, epoch T₀, and for each of the six
orbital elements (a, e, i, Ω, ω, and mean anomaly / mean longitude) the polynomial
coefficients `c₀, c₁, c₂`, plus the reference-frame tag and the center (helio/geo). Each row
carries a provenance comment citing SE `seorbel.txt` as the source. The CSV is parsed once
into a static table (build-time `include_str!`, parsed on first use), matching the
`fixstars-catalog.csv` precedent.

Rationale over vendoring SE's `seorbel.txt` verbatim plus a bespoke parser: the CSV parse is
trivial and uniform with the existing fixed-star catalog, and the `validate-fictitious` SE
parity gate is the authoritative check on transcription correctness — a faithful parser buys
no additional safety the gate does not already provide.

### 6. Validation — `validate-fictitious`

Two-tier, matching `validate-crossings` / `validate-eclipses-local`:

- **Tier 1 — self-consistency golden.** Determinism (byte-stable output for fixed inputs) and
  a Kepler round-trip check (propagate → recover mean anomaly), fnv1a64 checksum-guarded
  against a committed golden.
- **Tier 2 — SE parity.** A committed corpus generated by an SE-based tool that calls
  `swe_calc` for each fictitious body over the 1900–2100 TDB window (per-body sampled dates),
  with per-body arcsecond ecliptic-longitude/latitude ceilings set from measured residuals.
  Residuals are bounded by the shared Sun/Earth position error (the Kepler math itself is
  exact), so ceilings should be tight (arcsecond-class). fnv1a64 checksum-guarded.

The gate is wired into `run_all_numeric_gates` and the `release-gate` / `release-smoke`
batteries. CLI aliases: `fictitious` (compute a fictitious body position),
`validate-fictitious`, and `fictitious-gate`. A reference-corpus generator lives under
`tools/` alongside the other SE generators.

### 7. Versioning, docs, scope

- **Compatibility profile:** bump **0.7.8 → 0.7.9** (new public body surface).
- **API-stability profile:** **unchanged at 0.2.2** — purely additive to the
  `#[non_exhaustive]` `CelestialBody` / `CelestialBodyClass` enums and new crate exports.
- **Docs & claims:** update `crates/pleiades-fict` README, the root `README.md`, `PLAN.md`,
  `plan/status/01-*.md`, `plan/status/02-*.md`, the plan stage/track docs, and the
  overclaim-audit claim↔evidence mapping. **Mark SP-3 done** and record the remaining
  event-engine follow-ups (below) as separate future slices.

## Non-goals (recorded for sequencing)

- **`swe_pheno`** — planetary phenomena (phase angle, illuminated fraction, elongation,
  apparent diameter, apparent magnitude). Astronomy-flavored; one astrological hook
  (elongation → combustion/cazimi). Deferred to a later slice.
- **`swe_nod_aps`** — planetary/lunar nodes & apsides. The heavily-used lunar nodes are
  already exposed as bodies; the general planetary case has niche astrological use. Deferred.
- **User-supplied custom elements** — registering arbitrary orbital-element sets via the
  existing `Custom(CustomBodyId)` mechanism. Possible future extension; SP-3 ships the fixed
  default `seorbel.txt` set only.
- **Fixed-star catalog expansion / `swe_fixstar_mag`** — the curated 35-star catalog and
  apparent place already shipped in SP-2b; magnitude and a larger catalog are a separate
  fixed-star follow-up.
- **Occultations** (`swe_lun_occult_when_loc` / `swe_lun_occult_how`) and **central-path
  cartography** — distinct subsystems already recorded as deferred in the SP-2c design.
- **Perturbations** — out by definition. SE's fictitious bodies are unperturbed Kepler
  orbits; matching SE *requires* the unperturbed model, not a refinement of it.

## Accuracy expectation

Because the Kepler propagation is exact and deterministic and SE computes the identical orbit
from the identical elements, parity is expected to be essentially exact, limited only by the
accuracy of the shared Sun/Earth position used in the heliocentric→geocentric transform
(sub-arcsecond for the packaged backend). Final ceilings are set from measured residuals when
the corpus lands, consistent with the other numeric gates.
