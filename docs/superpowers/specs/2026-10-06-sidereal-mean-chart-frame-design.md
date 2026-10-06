# Sidereal mean chart on the mean equinox of date

Date: 2026-10-06. Issue: #164 item (b). Status: design, awaiting review.

## Problem

A chart requested with `Apparentness::Mean` reports the backend's J2000 mean
place. In a sidereal zodiac the chart layer subtracts the ayanamsa of the
chart's date from that J2000 longitude
(`crates/pleiades-core/src/chart/mod.rs`, the pre-apparent block that calls
`sidereal_longitude`).

An ayanamsa is the distance from the sidereal zero point to the equinox of
date. Subtracting it from a longitude counted from the J2000 equinox leaves
the precession accumulated since J2000 in the result. Measured for the Sun
with Lahiri, against the mean-of-date sidereal longitude `pleiades-events`
reports (`measure_chart_sidereal_conventions`):

| JD (TT) | Year | Chart minus mean-of-date sidereal |
|---|---|---|
| 2420000.5 | 1913 | 4342.509″ |
| 2451545.0 | 2000 | 0.000″ |
| 2460000.5 | 2023 | −1164.290″ |
| 2480000.5 | 2077 | −3918.689″ |

That is 1.2° at the start of the supported window. The placement's speed is
derived from the same longitude, so it is wrong by the precession rate, about
0.14″ per day. The same mix reaches a body that an apparent sidereal chart
falls back to a mean place for (issue #170).

## Decision

A sidereal placement that the chart layer leaves mean is the place on the
**mean ecliptic and equinox of date, less the mean ayanamsa of date**.

This is the rule #88 gave `pleiades-events` for its
`GeocentricMeanOfDate` sidereal frame, and it is what Swiss Ephemeris returns
for `SEFLG_SIDEREAL | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL`. After
this change the workspace has one definition of a sidereal longitude.

Two alternatives were considered and rejected:

- **Subtract the ayanamsa at J2000.** Smallest change and self-consistent,
  but it matches neither Swiss Ephemeris nor `pleiades-events`, and it would
  leave two sidereal definitions in the workspace.
- **Put every mean chart on the mean equinox of date, tropical included.**
  More coherent, since bodies and cusps would share a frame, but it changes
  tropical mean chart output for every user. It is a separate decision and
  out of scope here.

## What changes

All of it is in `pleiades-core`'s chart assembly. No public type or function
signature changes.

1. **Place.** Where the chart layer applies a sidereal zodiac to a mean
   placement, it first precesses the backend's J2000 longitude and latitude
   to the mean ecliptic and equinox of date
   (`pleiades_apparent::precess_ecliptic_j2000_to_date`, the IAU 1976
   precession the apparent path and `pleiades-events` already use), then
   subtracts the ayanamsa. The placement reports the precessed latitude.
   Distance is unchanged.
2. **Speed.** The longitude and latitude speeds become the backend's J2000
   speeds plus the rate of the precession step, differenced centrally over
   ±0.5 day on the J2000 place extrapolated with its own speed. The ayanamsa
   rate then comes off the longitude speed as it does today. No extra backend
   read is made, so `query_count_tests` are unaffected. A placement with no
   speed keeps none.
3. **Scope of "mean placement".** Both a chart requested with
   `Apparentness::Mean` and the mean fallback of an apparent chart take the
   step, through the one code path they already share.
4. **Sign and house.** Both are already derived from the reported longitude.
   A sidereal mean body and the sidereal cusps (#157, #180) are now in the
   same zodiac, so the house of a body near a cusp can change. That is the
   intended result.

What does not change:

- A **tropical** mean chart still reports the J2000 place. The rustdoc and
  READMEs will say so explicitly; today they do not say which equinox a mean
  chart is on.
- An **apparent** sidereal chart, already on the mean equinox of date since
  #120.
- The **equatorial** coordinates of a mean placement: the backend's J2000
  right ascension and declination, which no zodiac moves.
- A backend with `native_sidereal` capability: the chart layer does not
  touch its sidereal output. No first-party backend sets it.
- `pleiades_core::sidereal_longitude`, the public helper. It subtracts an
  ayanamsa from a longitude it is told is on the mean equinox, and still
  does.

A failure of the precession step (a non-finite input) fails the chart with
the existing apparent-place error mapping. It is not swallowed.

## Validation

### Unit and cross-crate tests

In `crates/pleiades-core/src/chart/sidereal_tests.rs`:

- A sidereal mean longitude equals the precessed J2000 longitude less the
  ayanamsa, at epochs near both ends of the window and at J2000, for the Sun,
  the Moon, a planet and a lunar point.
- The reported latitude is the precessed latitude.
- `sidereal_mean_speed_drops_by_the_rate_of_the_ayanamsa` is replaced: the
  sidereal mean speed is the tropical J2000 speed plus the precession rate
  less the ayanamsa rate, checked against a difference of the chart's own
  longitudes a day apart.
- A mean fallback in an apparent sidereal chart reports the same longitude as
  a requested mean chart.

In `crates/pleiades-events/tests/reference.rs`, the ignored diagnostic
`measure_chart_sidereal_conventions` becomes an asserting test: the chart's
sidereal mean longitude equals `longitude_at` in the mean-of-date sidereal
reference to within floating-point noise at the four epochs above. It fails
today with the figures in the table.

### Swiss Ephemeris gate

A new gate, `validate-sidereal-position`, modelled on
`validate-helio-position`:

- **Tool** `tools/se-sidereal-position-reference`, outside the workspace like
  its siblings. It writes Swiss Ephemeris (Moshier) geometric sidereal places
  and speeds: `SEFLG_SIDEREAL | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL
  | SEFLG_SPEED`.
- **Corpus** `crates/pleiades-validate/data/sidereal-position-corpus/`: the
  Sun, the Moon and Mercury to Pluto; one ayanamsa per computation class
  (Lahiri, True Citra, Galactic Center, De Luce), the four the crossings
  corpus uses; epochs every three years across 1900–2100. About 2,700 rows.
- **Gate** compares `ChartEngine` on the packaged backend, mean and sidereal,
  with the corpus in longitude, latitude and longitude speed. Ceilings are
  measured per body class and set by the corpus rule, `ceil(1.4 × max)`. The
  crossings gate's mean-of-date sidereal groups measure 0.32″ (Sun), 1.2″
  (Moon) and 0.42″ (planets), so figures near 1″, 4″ and 1″ are expected.
- **Stop condition.** A group with a systematic offset above its class's
  ayanamsa-gate ceiling is investigated before any ceiling is set.
- The gate joins the release battery (`release-smoke`, blocking tier). It
  computes about 270 charts, small beside the batteries already there; the
  plan measures its cost before it is added.

### Existing suites

The issue expected a chart golden to regenerate. A search of the committed
goldens found none that holds a sidereal mean chart; the plan confirms that
by running the blocking tier (`mise run ci`) and updates any pinned value it
turns up. The crossings, stations and aspects gates do
not go through the chart layer and are not expected to move.

## Documentation

- Rustdoc on `ChartRequest::with_apparentness` and `with_zodiac_mode`: which
  equinox each combination is on.
- `crates/pleiades-core/README.md` and `docs/cli.md` (`--mean` with
  `--ayanamsa`).
- `docs/status.md`: a row for the new gate.
- `crates/pleiades-events/README.md`: its mean-of-date frame note says "this
  is not the J2000 longitude a `pleiades-core` mean chart reports". That
  stays true for the tropical zodiac and gets one sentence saying the
  sidereal mean chart now agrees.

## Compatibility

Sidereal mean chart longitudes move by the precession since J2000: nothing at
2000, about 20′ at 2025, 1.2° at 1913. Latitudes move with the ecliptic, by
up to about 40″ at the ends of the window, and speeds by about 0.14″ per day.
This is a correction of wrong
output, released as a `fix(core)` through release-plz. Tropical charts and
apparent charts are unchanged.

## Out of scope

- Item (c) of #164, an opt-in apparent-star ayanamsa.
- The frame of a tropical mean chart. One consequence is worth recording: a
  tropical mean chart assigns houses by comparing a J2000 body longitude with
  cusps on the equinox of date, which `mean_houses_follow_the_reported_longitude`
  pins. That is a candidate for a separate issue.

## Delivery

One pull request on `fix/sidereal-mean-chart-frame`, in two commits: the
chart fix with its tests, then the reference tool, corpus and gate. The gate
cannot pass before the fix, and the fix should not land without it.
