# Apparent-star ayanamsa: Swiss Ephemeris default sidereal parity (issue #164 (c))

Status: approved design, 2026-10-07. Closes the last open item of #164.

## Problem

`pleiades` subtracts the **mean** ayanamsa from every sidereal place, in every
frame. Swiss Ephemeris, under its default `SEFLG_SIDEREAL` with apparent
flags, computes a star-anchored ayanamsa from the anchoring star's
**apparent** place: the star's annual aberration and gravitational light
deflection enter the ayanamsa. A user comparing a pleiades sidereal chart with
Swiss Ephemeris (or software built on it) sees a difference of up to about
22″ for those ayanamsas. That is about 8 minutes of crossing time for the Sun
and hours for a slow planet.

## Measurement (2026-10-07)

Swiss Ephemeris (Moshier), `swe_get_ayanamsa_ex`, every 7.3 days over
1900–2100. Columns: apparent (`MOSEPH|NONUT`) minus geometric
(`… |TRUEPOS|NOABERR|NOGDEFL`), split into aberration (`… |NOGDEFL` minus
geometric) and deflection (the rest). The last column is the largest residual
after fitting `A·cos λ☉ + B·sin λ☉ + C` to the total. All in arcseconds.

| Mode (SE id) | Total, max | Aberration, max | Deflection, max | Residual of a pure annual fit |
|---|---|---|---|---|
| True Citra / Chitra (27) | 20.685 | 20.583 | 0.114 | 0.504 |
| True Revati (28) | 21.541 | 20.547 | 1.091 | 1.094 |
| True Pushya (29) | 22.676 | 20.812 | 2.768 | 2.760 |
| True Mula (35) | 21.448 | 21.448 | 0.018 | 0.516 |
| True Sheoran (39) | 22.676 | 20.812 | 2.768 | 2.760 |
| Galactic Center (17), Rgilbrand (30), Cochrane (40) | 20.949 | 20.932 | 0.043 | 0.505 |
| Galactic Center Mula/Wilhelm (36) | 21.880 | 21.862 | 0.046 | 0.550 |
| Galactic Equator IAU1958, True, Mula, Fiorenza (31, 32, 33, 41); Galactic Center Mardyks (34) | 0 | 0 | 0 | 0 |
| Lahiri (1), Fagan/Bradley (0) (controls) | 0 | 0 | 0 | 0 |

Findings:

- **Nine SE modes (ten catalog entries) are affected, not fifteen.** #164
  supposed that every ayanamsa of the `TrueStar` and `Galactic` computation
  classes is affected. The four galactic-equator modes and Mardyks are not.
- **Annual aberration is nearly all of it.** A pure annual term leaves about
  0.5″, mostly the eccentricity term of Earth's orbit, so parity needs the full
  aberration formula, not `κ cos(λ☉ − λ★)` alone.
- **Light deflection matters for the near-ecliptic anchor stars:** up to
  1.091″ for ζ Psc (Revati) and 2.768″ for δ Cnc (Pushya, Sheoran), when the
  Sun passes near them. It is at most 0.114″ for the others.

## Decisions

1. **An opt-in, request-level convention option**, `SiderealStarPlace { Mean,
   Apparent }`, default `Mean`. Chosen over apparent twins of the ayanamsas in
   the catalog (nine more entries that mix a convention into an ayanamsa's
   identity, and would have to stay mean in a mean chart against their names)
   and over a new `ZodiacMode` variant (additive, but every internal match on
   `ZodiacMode::Sidereal` must learn it and a missed arm silently falls back;
   it also puts a Swiss Ephemeris convention into the shared type crate).
2. **The default does not change.** Every chart, event and golden is
   bit-identical under `Mean`.
3. **Swiss Ephemeris ties the star place to the body's flags**, and so does
   pleiades: the correction applies only to a placement that is itself reduced
   to the apparent place. A mean chart, the mean fallback inside an apparent
   chart, the mean-of-date and heliocentric event frames, and the tropical
   zodiac keep the mean ayanamsa.
4. **The correction is computed in closed form**, without backend reads: the
   anchor star's annual aberration (Meeus 23.2, with the eccentricity terms)
   plus its gravitational light deflection, both from the Meeus Sun that
   `pleiades-apparent` already carries for the aberration argument.

## Components

### `pleiades-types`

No change.

### `pleiades-ayanamsa` (depends on `pleiades-types` only)

Pure data, additive:

- `SiderealStarPlace { Mean, Apparent }`: `#[non_exhaustive]`, `Default =
  Mean`, `Copy`, optional serde. Defined here because both `pleiades-core` and
  `pleiades-events` depend on this crate; `pleiades-events` has `pleiades-core`
  only as a dev-dependency. Re-exported from both.
- `StarAnchor { sidereal_longitude: Angle, ecliptic_latitude: Angle }` and
  `fn star_anchor(&Ayanamsa) -> Option<StarAnchor>`. `Some` for exactly True
  Citra, True Chitra, True Revati, True Pushya, True Mula, True Sheoran,
  Galactic Center, Galactic Center (Rgilbrand), Galactic Center (Mula/Wilhelm)
  and Galactic Center (Cochrane). `None` for every other mode, including
  custom ayanamsas.
- The anchor values are measured from Swiss Ephemeris (Planning task 1), not
  typed from memory. The anchor star's mean place of date is
  `λ★ = ayanamsa_mean(t) + sidereal_longitude` and `β★ = ecliptic_latitude`.

### `pleiades-apparent`

Pure functions, additive:

- `gravitational_deflection(lambda_deg, beta_deg, jd_tt) -> DeflectionOffset`,
  a new type shaped like `AberrationOffset` (`d_lambda_arcsec`,
  `d_beta_arcsec`): the deflection of a source at infinity by the Sun,
  `Δ = (2GM☉ / c² r) · cot(ψ/2)` along the great circle away from the Sun,
  with ψ the elongation and r the Sun's distance, from the Meeus Sun. Near
  conjunction it follows the rule Swiss Ephemeris uses, measured in Planning
  task 1.
- `apparent_star_correction(lambda_deg, beta_deg, jd_tt) -> f64`: the
  ecliptic-longitude sum of `annual_aberration` and the deflection, in
  degrees, and its time derivative for speeds. The one place the arithmetic
  lives.

### `pleiades-core`

- `ChartRequest::with_sidereal_star_place(SiderealStarPlace)`; the request
  field defaults to `Mean`.
- `chart/sidereal.rs`: a placement's offset is
  `ayanamsa_mean + apparent_star_correction(λ★, β★, t)` when the request says
  `Apparent`, the placement is reduced to the apparent place, and
  `star_anchor` is `Some`. Otherwise it is the mean ayanamsa, as today.
- The placement's longitude speed subtracts the correction's derivative as
  well as the ayanamsa's rate.
- House cusps and the four angles follow what `swe_houses_ex` does under
  `SEFLG_SIDEREAL` (Planning task 1). If Swiss Ephemeris uses the apparent
  ayanamsa there, cusps follow the placements; if not, they keep the mean
  ayanamsa and the docs say so.
- The derived lunar points, which the chart reduces as geometric directions
  without aberration, keep the mean ayanamsa unless Planning task 1 shows Swiss
  Ephemeris does otherwise.
- Sidereal provenance records the star place used when the correction is
  applied, and `ChartSnapshot`'s `Display` shows it.

### `pleiades-events`

- `CrossingReference` gains `star_place: SiderealStarPlace` (default `Mean`)
  and `with_star_place`. The crate is unpublished, so the new public field
  costs no release.
- `reference.rs` applies the correction only in
  `CrossingFrame::GeocentricApparentOfDate`. The mean-of-date and heliocentric
  frames use geometric flags in Swiss Ephemeris and keep the mean ayanamsa.
- Speeds, and therefore stations and aspect turning points, include the
  correction's derivative.

### `pleiades-cli`

`--star-place mean|apparent` (default `mean`) on every command that takes
`--ayanamsa`.

## Behaviour

| Case | Result |
|---|---|
| `Mean` (default) | Bit-identical to today everywhere. |
| `Apparent`, tropical zodiac | No effect; not an error. |
| `Apparent`, ayanamsa without an anchor (Lahiri, galactic equator, Mardyks, custom, …) | No effect, matching Swiss Ephemeris's zero difference; documented and tested, not an error. |
| `Apparent`, mean chart or a placement on the mean fallback | Mean ayanamsa. |
| `Apparent`, topocentric chart | The same annual correction; no diurnal aberration of the anchor, as in Swiss Ephemeris. |
| `Apparent`, events in `GeocentricMeanOfDate` or `Heliocentric` | Mean ayanamsa. |

No new error variants: the correction is closed-form and its inputs are
already finite.

## Accuracy

- Apparent-minus-mean ayanamsa against Swiss Ephemeris, 1900–2100: target
  **0.1″** for every anchored mode away from solar conjunction of the anchor
  star. The remaining terms after Meeus 23.2 (Earth–Moon barycentre motion,
  planetary perturbation of Earth's velocity) are of order 0.01–0.02″, and the
  Meeus Sun's error contributes about 0.004″. If aberration alone cannot reach
  0.1″, the spec records the measured floor instead of adding Earth-velocity
  series.
- Near conjunction, the ceiling is the measured maximum × 1.4, as in the other
  gates.
- The anchor latitude is held constant at its J2000 value. It moves about
  0.5″ a century, and its effect on Δλ is `κ · tan β★ · sec β★ · δβ`, under
  0.001″ even for the Galactic Center (β ≈ −5.6°). Planning task 1 confirms
  this.

## Validation

### Planning task 1: measurements that fix the open rules

In `tools/se-ayanamsa-reference`, a new `apparent` mode measures and records,
as amendments to this spec:

1. Each anchored mode's sidereal anchor longitude and ecliptic latitude.
2. Swiss Ephemeris's deflection near conjunction (its taper or cut-off).
3. Whether `swe_houses_ex` with `SEFLG_SIDEREAL` uses the apparent ayanamsa.
4. Whether the true node and apsides carry the mean or the apparent ayanamsa
   under plain `SEFLG_SIDEREAL`.

### Gate `validate-ayanamsa-apparent`

Fail-closed and in the release battery, checksum-guarded (fnv1a64) and pinned
by row count:

- Corpus: Swiss Ephemeris apparent-minus-mean ayanamsa (Moshier, `NONUT`) for
  the nine anchored SE modes, about 100 instants each. About 60 % are uniform
  over 1900–2100; about 40 % lie within ±3 days of the anchor star's
  conjunction with the Sun, so deflection is held where it matters.
- Zero-difference rows for the four galactic-equator modes and Mardyks.
- Ceilings per class (aberration-dominated, deflection-dominated), from
  measured maxima × 1.4, with 0.1″ the target for the first.

### End-to-end parity

`validate-sidereal-position` gains rows from Swiss Ephemeris's plain
(apparent) `SEFLG_SIDEREAL` for the Sun, the Moon and Mars × True Citra and
Galactic Center, checked against a chart built with
`SiderealStarPlace::Apparent`. Its existing mean rows do not change.

### Unit and integration tests

- `pleiades-apparent`: deflection against hand-evaluated values at chosen
  elongations, including the near-conjunction rule; the correction's
  derivative against a central difference.
- `pleiades-ayanamsa`: `star_anchor` is `Some` for exactly the ten catalog
  entries above; True Chitra equals True Citra.
- `pleiades-core`, `pleiades-events`: default options are bit-identical;
  `Apparent` moves only anchored, apparent placements; mean charts, mean
  frames and unanchored ayanamsas are unaffected; speeds include the
  derivative; a crossing found under `Apparent` lies at the apparent-ayanamsa
  target longitude.
- `pleiades-cli`: `--star-place apparent` round-trip.

## Documentation

- Rustdoc on every new public item, with an example on
  `ChartRequest::with_sidereal_star_place`.
- The `pleiades-core`, `pleiades-events` and `pleiades-ayanamsa` READMEs;
  `docs/cli.md`.
- `CURRENT_COMPATIBILITY_PROFILE_SUMMARY`: a new entry, the profile id bumped
  and the content checksum re-measured.
- `spec/api-and-ergonomics.md`: a line on convention options that default to
  the existing behaviour.
- #164: post the measurement and correct "every `TrueStar` and `Galactic`
  ayanamsa" to the nine SE modes.

## Delivery

One PR from the worktree branch `worktree-issue-164c-apparent-star-ayanamsa`.
If the plan outgrows one reviewable diff, it splits into (1) the measurements,
the anchors, the correction and the new gate, and (2) the opt-in on core,
events and CLI with the end-to-end rows.

## Amendments (Planning task 1, measured 2026-10-07)

Measured with throwaway probes against Swiss Ephemeris 2.10.03 (libswisseph-sys
0.1.2, Moshier) and read from its source (`sweph.c` `swi_get_ayanamsa_ex`,
`swi_deflect_light`, `meff`; `swehouse.c`). They replace the corresponding
parts of the design above.

1. **Anchors are five stars, not ten per-mode rows.** Swiss Ephemeris
   hard-codes each star-anchored mode as a star plus a sidereal longitude:
   Spica 180° (True Citra), ζ Psc 359.8333333333° (True Revati), δ Cnc 106°
   (True Pushya) and 103.49264221625° (True Sheoran), λ Sco 240° (True Mula),
   Sgr A* 240° (Galactic Center), 270° (Cochrane) and 210 + 90 × 0.3819660113°
   (Rgilbrand). A star's mean place of date is therefore its **primary**
   mode's mean ayanamsa plus that mode's anchor; SE's own geometric
   `swe_fixstar` longitude equals that sum to 0.000000″ for all five stars over
   1900–2100. `pleiades-ayanamsa` holds `AnchorStar` (five stars), each mode's
   `StarAnchor { star, projection }`, and the star's mean place.
2. **Latitude has a linear term.** The stars' latitudes move 18–100″ over
   1900–2100. A line `β = b0 + b1·T` (T in Julian centuries from J2000) fits
   SE within 0.11″ for every star, which is negligible even for δ Cnc's
   deflection; a constant would not be (δ Cnc passes 278″ from the ecliptic).
   Fitted values: Spica −2.054501717°, −0.007550067°/cy; ζ Psc −0.213417908°,
   +0.002565390°/cy; δ Cnc +0.077157475°, +0.003140560°/cy; λ Sco
   −13.788460720°, −0.013920718°/cy; Sgr A* −5.607682523°, −0.013203301°/cy.
3. **Galactic Center (Mula/Wilhelm) is projected.** SE takes Sgr A*'s right
   ascension, projects it onto the ecliptic with `swi_armc_to_mc` and the mean
   obliquity, and subtracts 246.6666666667°. Its correction is the projected
   apparent place minus the projected mean place
   (`AnchorProjection::PolarRightAscension`); every other mode's is the
   longitude difference (`AnchorProjection::EclipticLongitude`).
4. **Deflection follows `swi_deflect_light`.** The vector formula
   `u + g1/(1 + u·e) · (e − (u·e) u)` with `g1 = 2GM☉·meff/(c²·AU·r)`, `e` the
   Sun-to-Earth unit vector and `r` the Sun's distance, tapered inside the
   solar disc by SE's 101-row effective-mass table `meff` (Stix's solar
   model), keyed on `sin(elongation) / (959.63″ / r)`. Deflection is applied
   before aberration, as in SE. The Meeus Sun gains its radius vector (Meeus
   25.5) for `r`.
5. **House cusps take the correction.** `swe_houses_ex` passes the caller's
   flags to `swe_get_ayanamsa_ex`; a plain `SEFLG_SIDEREAL` cusp uses the
   apparent ayanamsa including nutation (residual ≤ 0.002″ over 1900–2100).
   So in an apparent chart with `SiderealStarPlace::Apparent`, cusps, angles
   and `ascmc` points take the same correction as placements. A mean chart's
   cusps do not.
6. **Lunar points take the correction.** Under plain `SEFLG_SIDEREAL` Swiss
   Ephemeris applies the apparent ayanamsa to every body, including the mean
   and true node and the mean and osculating apogee (residual 0.000″): the
   star place follows the request's flags, not the body's physics. Every
   placement of an apparent chart therefore takes the correction except a
   mean-fallback placement, and every body in `GeocentricApparentOfDate`
   events.
7. **`SiderealStarPlace` lives in `pleiades-types`**, beside `ZodiacMode`.
   `pleiades-ayanamsa` has no serde support and `CrossingReference` derives
   serde; a new enum in the type crate is additive. `pleiades-core` and
   `pleiades-events` re-export it.
8. **Speeds need no analytic derivative.** Both crates already difference the
   corrected place centrally (`correction_sample` in core, `sampled_place` in
   events), so the correction's rate enters every speed automatically.
9. **Semver.** `ChartRequest` and `ChartSnapshot` are not `#[non_exhaustive]`,
   so their new `sidereal_star_place` field is a breaking change to
   `pleiades-core` (decided 2026-10-07: accept it, as `topocentric` was added,
   and commit it as `feat(core)!:` so release-plz cuts a new minor).
   `CrossingReference` is already `#[non_exhaustive]`; its new field is
   additive, with `serde(default)` so older serialized references read as
   `Mean`.
10. **One composition, held equal across crates.** `pleiades-core` exposes
    `apparent_star_ayanamsa_correction(&Ayanamsa, Instant) -> Option<Angle>`.
    `pleiades-events` cannot depend on `pleiades-core`, so it keeps a private
    twin; a cross-crate test in `pleiades-events/tests/reference.rs` holds the
    two equal.
11. **Prototype accuracy** (Meeus 23.2 aberration, the deflection above,
    Meeus Sun), maximum |SE − model| over 1900–2100 every 0.731 day: True Citra
    0.012″, True Mula 0.012″, Galactic Center 0.011″, Galactic Center
    (Mula/Wilhelm) 0.012″, True Revati 0.056″, True Pushya 0.496″. The last is
    δ Cnc passing behind the solar disc, where the Meeus Sun's ~0.01° error is
    large against the star's 278″ closest approach; that row class gets the
    measured ceiling the design allows.

## Out of scope

- Changing the default convention.
- An apparent-place variant of any ayanamsa without a star anchor.
- Diurnal aberration of the anchor star.
- Earth-velocity series beyond Meeus 23.2 unless the measurement shows 0.1″ is
  unreachable without them, in which case the floor is recorded instead.
