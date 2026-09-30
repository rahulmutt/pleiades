# Apparent place double-counts annual aberration (issue #93) — design

**Status:** design approved · **Opened:** 2026-09-30 · **Issue:** #93 ·
**Crates:** `pleiades-apparent` (the fix), `pleiades-validate` (gate
re-baselining), `pleiades-core` / `pleiades-events` / `pleiades-eclipse`
(documentation only) · **Follows:** #91 / PR #92 (apparent-place speed), which
surfaced this while measuring.

## Context

`apparent_position` (`crates/pleiades-apparent/src/apparent.rs`) builds a
body's apparent ecliptic place of date in four steps: light-time iteration
that re-queries the body's **geocentric** position at the retarded instant
`t − τ`, precession J2000 → mean equinox of date, nutation in longitude, and a
separate annual-aberration term (Meeus 23.2).

A geocentric position at `t − τ` is `body(t − τ) − Earth(t − τ)`. Retarding
the Earth by `τ` displaces the direction by `V_Earth · τ = Δ · (V_Earth / c)`,
which is the first-order annual aberration. The light-time re-query therefore
already carries aberration, and the separate term counts it a second time,
about 20″ in longitude for every body on that path.

FU-1 (resolved 2026-06-30) found and fixed exactly this for the Sun by adding
`apparent_sun_position`, which performs no light-time re-query and applies
aberration once. Its record and the surrounding doc comments state that "for
the planets, light-time and stellar aberration are genuinely distinct", which
is true of the effects but not of the code path: the geocentric re-query
contains both.

### Evidence (from issue #93, probe on `main` at 9453c5d, 2026-09-30)

Chart apparent longitudes over the packaged backend against the Horizons
goldens in `crates/pleiades-validate/data/apparent-goldens.csv`, with the
aberration term the pipeline applied read from
`BodyPlacement.apparent.aberration_longitude_arcsec`:

| Body | JD (TT) | Residual vs golden | Term applied | Residual without the term |
|---|---|---|---|---|
| Mercury | 2415025.5 | −19.59″ | −19.64″ | +0.05″ |
| Venus | 2415025.5 | −18.48″ | −18.51″ | +0.03″ |
| Mars | 2415025.5 | −20.92″ | −20.82″ | −0.10″ |
| Mars | 2488065.5 | +6.57″ | +7.56″ | −0.99″ |
| Jupiter | 2469807.5 | +19.93″ | +19.47″ | +0.46″ |
| Saturn | 2469807.5 | −20.21″ | −19.95″ | −0.26″ |
| Uranus | 2433282.5 | +20.87″ | +20.67″ | +0.21″ |
| Neptune | 2415025.5 | +19.52″ | +19.57″ | −0.06″ |
| Pluto | 2451545.0 | −18.61″ | −18.56″ | −0.05″ |

Across all 45 planet rows the residual tracks the applied term. With it
removed, Jupiter through Pluto are within 0.5″, Mars within 2.1″, Mercury and
Venus within 4.6″. The goldens header attributed the 15–25″ planet residuals
to the polynomial-fit ephemeris, which is what hid this.

The Moon does not clean up the same way: its residuals (−9″ to −43″) become
−39″ to +1″ with the term removed, so it has a second problem on top of this
one.

### Who is on the affected path

- `pleiades-core` charts: every release-grade body except the Sun and the
  osculating apsides/node, on the default (apparent) path. The apparent-place
  speed from PR #92 differences the apparent-minus-mean correction and follows
  the longitude automatically.
- `pleiades-events`: `geocentric_apparent_ecliptic` (`src/ephemeris.rs`) uses
  the same routine for every non-Sun body, so longitude crossings, `pheno`,
  rise/set/transit and occultations carry the offset (about 40 s of time for
  the Moon at 20″).
- **Not affected:** `pleiades-eclipse` (`src/ephemeris.rs`) has its own
  light-time-retarded Moon sampling with no separate aberration term and
  takes the Sun from `apparent_sun_position`; `pleiades-events::fixstar`
  applies `annual_aberration` to catalogue positions with no re-query, which
  is correct for a star.

## Decision

Take the light-time re-query as the carrier of annual aberration for every
body on the generic path, and stop adding the separate term. Retain the Sun
and apsis special cases unchanged.

### Approaches considered

1. **Drop the aberration term from the generic path (chosen).** One function
   changes. The issue's probe shows planet residuals collapse to the
   ephemeris-fit floor.
2. **Retard only the body, keep the Earth at `t`, apply aberration once.**
   Equivalent to first order, but needs barycentric Earth and body positions
   that the backend trait does not expose. Rejected: an API change for no
   accuracy gain.
3. **Route the Sun through the generic path too** (re-query the geocentric Sun
   at `t − τ`, no term) and delete `apparent_sun_position`. Rejected as an
   opportunistic refactor: the Sun rows are gated at 5″ and would move by
   second-order amounts for no benefit.

## Design

### 1. The code change (`pleiades-apparent`)

In `apparent_position`:

- After `apparent_via_light_time`, apply precession and nutation only. Pass
  `0.0` for both aberration arguments of `combine_apparent`, as the apsis path
  does.
- Keep calling `annual_aberration(lambda, beta, sun_true_longitude_of_date_deg,
  jd_tt)` but only to fill the provenance field: it is the Meeus 23.2 estimate
  of the aberration component the retarded re-query already contains. The
  `sun_true_longitude_of_date_deg` parameter therefore stays in the signature;
  the public API does not change.
- Provenance: `corrections.annual_aberration` stays `true` (the place includes
  annual aberration), `corrections.light_time` stays `true`, and
  `aberration_longitude_arcsec` carries the estimate. The field docs in
  `provenance.rs` state that on the light-time path this value is the
  estimated component included through the retarded geocentric query, not a
  term added on top, and that on the Sun path it is the term actually added.
- Module doc and the `apparent_sun_position` doc are rewritten: the order of
  operations is light-time re-query (which carries aberration, since the
  Earth is retarded with the body), precession, nutation. The sentence
  claiming the planet path needs both is replaced by the retarded-Earth
  argument above.

The Sun path (`apparent_sun_position`) and apsis path
(`apparent_apsis_position`) are untouched.

### 2. Documentation to correct

The "distinct effects, so the standard `apparent_position` is correct for
them" claim appears in:

- `crates/pleiades-apparent/src/apparent.rs` (module doc,
  `apparent_sun_position` doc);
- `crates/pleiades-eclipse/src/ephemeris.rs` ("Why aberration is applied only
  once" section of `apparent_sun_longitude_deg`);
- `crates/pleiades-core/src/chart/mod.rs` comment near the Sun arm of the
  apparent dispatch;
- the tolerance-rationale headers of `apparent-goldens.csv`,
  `equatorial-goldens.csv` and `topocentric-goldens.csv` in
  `crates/pleiades-validate/data/` (planet lines attribute the residual to the
  fit; Moon line attributes it to "the planetary formula applied to a body
  sharing Earth's orbit").

Each gets the corrected statement. `docs/follow-ups.md` FU-1 gets a dated
note that the planet half of the finding was fixed under #93, with a pointer
to this spec.

### 3. Gate re-baselining (`pleiades-validate`)

Every gate that consumes the generic apparent path is re-measured on the fixed
code and its ceiling re-pinned by that gate's own existing convention. All of
these live in `pleiades-validate`, which the blocking tier excludes; they run
in `release-gate` and nightly.

| Gate | Ceiling today | Convention | Expected direction |
|---|---|---|---|
| `validate-apparent`, planet rows | 26″ | max observed + 2″, per body | drops toward 1–5″ |
| `validate-apparent`, Moon rows | 45″ | max observed + 2″ | re-measured (section 4) |
| `validate-equatorial`, planet rows | per header | per header (max + margin) | drops |
| `validate-topocentric`, Sun/planet rows | per header | per header | planets drop; Sun unchanged |
| `validate-crossings` Tier-2 `GEO_PLANET` / `GEO_MOON` / `PLUTO` | 30″ / 31″ / 17″ | ceil(1.4 × group max) | drops |
| `validate-crossings` Tier-2 `HELIO` | 50″ | ceil(1.4 × group max) | re-measured; may not move |
| `validate-pheno` elongation / phase angle | 30″ / 85″ | ~1.4 × measured, rounded up | elongation drops (measured max was 20.97″, Uranus) |
| `validate-occultations` contact seconds | 65 s | ~1.4 × measured | expected to drop (20″ of Moon ≈ 40 s) |
| `validate-rise-trans` rise/set tight | 5 s | ~1.4 × measured | small; tighten only if the measured max moves |

Rules:

- A ceiling is only tightened, never loosened, in this change. A gate whose
  measured maximum gets worse is a finding to diagnose, not a number to
  raise.
- Each re-pinned constant or CSV tolerance carries the measured maximum, the
  body/row it came from, and the date, following the existing threshold-file
  comments.
- Goldens values are not regenerated; only the tolerance column and the
  header rationale change. The Horizons rows stand.
- The plan records the before/after measured maxima per gate.

### 4. The Moon

The fix is body-agnostic, so the Moon's apparent place changes with the
planets'. Its residual against Horizons is not expected to reach the planet
floor. This item measures, does not diagnose:

- Record the Moon's per-row residual against the Horizons apparent goldens
  after the fix.
- Record the Moon rows' Tier-2 residual in `validate-crossings` (against Swiss
  Ephemeris) after the fix.
- If the two references disagree by roughly the double-count amount, the
  goldens (or what Horizons' geocentric "apparent" quantity includes for the
  Moon) are the suspect; if they agree, the packaged Moon's fit against its
  source is. Either way, set the Moon golden tolerance from the measurement
  and open `FU-14` in `docs/follow-ups.md` with both numbers, the two
  hypotheses, and this spec as origin. The Moon tolerance follows the
  section 3 rule: tightened to max observed + 2″ when the measured maximum
  drops, left at 45″ otherwise.

Diagnosing and fixing the Moon's second problem is out of scope here.

### 5. Tests

- **Unit, `pleiades-apparent`** (`apparent/tests.rs`): a synthetic geocentric
  query whose body direction is a known function of the instant, with a
  reference that applies precession and nutation to the retarded direction and
  nothing else. Asserts the returned longitude and latitude match the
  reference to well under 0.1″, so a re-introduced 20″ term fails.
- **Regression, `pleiades-apparent`**: a retarded-Earth reference for a
  planet-like case (body on a circular orbit, Earth on its own circular orbit,
  light-time from the geocentric distance): compute `body(t − τ) − Earth(t)`,
  apply first-order aberration once, and check `apparent_position` over a
  geocentric query of `body(t′) − Earth(t′)` agrees to under 0.1″. This pins
  the physical claim, not just the absence of the term.
- **Provenance**: `aberration_longitude_arcsec` on the light-time path is
  non-zero and equals the Meeus estimate; `corrections.annual_aberration` is
  `true`; the Sun path's values are unchanged.
- **Existing**: the PR #92 apparent-motion tests run unchanged; the FU-1 Sun
  tests run unchanged; the eclipse gates are unaffected and confirm it.
- **Gates**: run every gate in section 3 before and after the fix; the
  numbers go into the plan and the pinned constants.

## Out of scope

- The Moon's remaining residual (section 4 files it).
- Gravitational light deflection (still omitted, sub-arcsecond away from the
  solar limb).
- Unifying the Sun path with the generic path.
- The `HELIO` crossings ceiling beyond re-measuring it.
