# Pluto periodic-term fit for the algorithmic backends (issue #129) — design

**Status:** approved (2026-10-04) · **Opened:** 2026-10-04 · **Issue:** #129 ·
**Crates:** `pleiades-vsop87` (code, data, tests), `pleiades-validate`
(comparison tolerance, docs), `pleiades-core` (compatibility profile)

## Context

Since #119, `Vsop87Backend` serves Pluto from fixed mean orbital elements
minus the VSOP87B Earth. The method is right for what it is and wrong by a
systematic 0.4–0.6°: over a monthly 1972–2099 sweep against Swiss Ephemeris
the issue measured max |Δλ| 35.7′, |Δβ| 10.8′, |Δr| 0.116 AU, with Δλ of one
sign throughout. Every other body the composite serves is within 11″. The
artifact-free `CompositeBackend<Vsop87Backend, ElpBackend>` is what a
consumer gets without the packaged data, so this is its Pluto by default.
`profiles.rs` already names the gap: the mean-element bucket stays "until a
Pluto-specific source path is selected".

Routed charts are unaffected: `PackagedDataBackend` serves Pluto (about 1″
from Swiss Ephemeris) ahead of the composite inside 1900–2100.

## Goal

Serve Pluto from `Vsop87Backend` at arcsecond class over the fit's validity
window, gated like the VSOP87B bodies, without changing routing, the backend
boundary frame, or any other body.

## Decision: Meeus Table 37.A, inside `pleiades-vsop87`

**The fit.** Meeus, *Astronomical Algorithms* (2nd ed.), chapter 37,
Table 37.A: 43 periodic terms in the arguments J, S, P (mean longitudes of
Jupiter, Saturn and Pluto) giving heliocentric ecliptic λ, β, r referred to
the dynamical equinox and ecliptic J2000. Meeus credits the series to
J. Chapront's fit to DE200 and quotes 0.6″ in λ, 0.2″ in β over 1885–2099,
with accuracy falling off quickly outside it.

Measured before writing this spec (throwaway probe, 7303 samples every
10 days over 1900–2099), geocentric J2000 = Table 37.A heliocentric + the
`Vsop87Backend` geocentric Sun, against the packaged DE440-fitted geometric
Pluto:

| | current mean elements | Table 37.A |
|---|---|---|
| max \|Δλ\| | 35.7′ | 3.29″ |
| max \|Δβ\| | (10.8′ per the issue) | 0.34″ |
| max \|Δr\| | (0.116 AU per the issue) | 3.3e-4 AU |

The probe also reproduces Meeus's worked Example 37.a (1992-10-13 0h TD:
λ 232.74071°, β 14.58782°, r 29.711111 AU) to the printed digits.

**Why this fit.** It is the published fit whose window covers the issue's
1900–2100 request almost completely, it is small enough to hand-check
(43 rows × 6 coefficients), and the measured error is well under the 11″
the other composite bodies reach. Longer-window fits (Goffin's 1983 series,
or the Chapront–Francou 1995 multi-interval fit) would extend coverage at
the cost of much larger tables and no improvement inside 1900–2100; they are
out of scope.

**Why inside `pleiades-vsop87`.** Pluto already has a slot there in four
places (claim, source kind, catalog entry, the fallback arm of
`geocentric_coordinates`), the composite routes Pluto to `Vsop87Backend`
because it lists Pluto among its supported bodies, and the geocentric
reduction needs the VSOP87B Earth that crate already evaluates. A sibling
crate would need a new composite layer or a routing change for no benefit.

**Data provenance.** The coefficients are transcribed from Table 37.A. Two
independent open implementations of the same table (one MIT-licensed, one
LGPL-licensed) were used only as transcription checks: all 43 rows agree with
each other to the last printed digit. No code is taken from either. The
table lives in its own data module, kept whole, with the source cited in the
module docs, like the VSOP87B tables cite IMCCE.

## 1. Code

- `src/tables/pluto_meeus.rs` (new, data module kept whole): the 43-row
  `const` table and `pluto_lbr(jd_tt) -> SphericalLbr`, evaluating
  λ = 238.958116° + 144.96° T + Σ(A sin α + B cos α),
  β = −3.908239° + Σ…, r = 40.7241346 + Σ… with α = iJ + jS + kP
  (J = 34.35° + 3034.9057° T, S = 50.08° + 1222.1138° T,
  P = 238.96° + 144.96° T, T in Julian centuries of TT from J2000). A small
  table, so a hand-written `const` rather than the `.bin` blob and generator
  the VSOP87B files use; it stays out of `source_manifest()` so the
  regenerator ignores it.
- `backend.rs`, `geocentric_coordinates`: inside the window, Pluto is
  `pluto_lbr` converted to Cartesian minus the VSOP87B heliocentric Earth,
  exactly as the table-backed planets are. Outside the window, the existing
  mean-element path is kept as the fallback.
- **Window:** JD_TT 2_409_542.5 (1885-01-01 0h) inclusive to 2_488_069.5
  (2100-01-01 0h) exclusive, i.e. the whole of 1885–2099, as named constants
  with the Meeus citation. The issue's 1900–2100 request ends at that
  exclusive bound.
- **Fallback outside the window:** the mean-element path, with the result's
  `QualityAnnotation::Approximate` as today. Inside the window the result
  carries the same quality annotation the VSOP87B planets carry. Rejecting
  out-of-window instants instead would break consumers of an algorithmic
  backend whose nominal range is unbounded, so the fallback stays. At the
  window edges Pluto therefore jumps by up to about 0.6°; this is documented
  on the window constant and in the crate README. A root finder spanning an
  edge sees the jump; within 1885–2099 nothing changes.
- **Speeds:** unchanged mechanism (the backend's ±0.5-day central difference
  of the geocentric place), now differencing the fit. A sample whose ±0.5-day
  neighbours straddle a window edge differences across the jump; the
  neighbours are evaluated with the same path as the centre instant (chosen
  by the centre instant), so the speed stays smooth up to the edge.

## 2. Catalog and claims

- `Vsop87BodySourceKind` gains `PeriodicTermFit` ("published periodic-term
  fit"), and the enum becomes `#[non_exhaustive]`, so that the next source
  family does not break downstream matches again. Both are breaking for an
  exhaustive external `match`, so the change ships as `feat(vsop87)!`
  (pre-1.0 minor bump via release-plz).
- The Pluto catalog entry becomes `PeriodicTermFit`, provenance "Meeus
  Astronomical Algorithms Table 37.A periodic-term fit (Chapront, DE200),
  valid 1885–2099; mean-element fallback outside", source fidelity
  `AccuracyClass::Exact` (the full published table). It keeps
  `canonical_sample: None`: canonical samples feed the J2000 geocentric
  evidence summaries of the VSOP87B files, and Example 37.a is a
  heliocentric check that lives in its own unit test instead.
- Inside the window a Pluto result carries `QualityAnnotation::Exact`, as the
  VSOP87B planets do; outside it carries `Approximate`.
- `vsop87_body_claims()`: Pluto moves from `BodyClaim::approximate` to
  `BodyClaim::constrained(Pluto, Moderate, AlgorithmicModel)`, like every
  other body the backend serves.
- **Buckets (amended 2026-10-04, while planning):** "source-backed" keeps
  meaning *backed by a vendored VSOP87B source file*, because the
  regenerator, the source manifest and the source-documentation summary all
  assume it. A new `Vsop87BodySourceKind::is_vsop87b()` decides the split:
  `source_backed_body_profiles()` is the VSOP87B kinds (Sun–Neptune,
  unchanged) and `fallback_body_profiles()` is every other kind, which is
  still exactly Pluto, now with kind `PeriodicTermFit`. Its docs say "bodies
  outside the VSOP87B files". Summary counts and body lists are unchanged;
  only the kind label and provenance strings move. The `MeanOrbitalElements`
  variant stays (it still describes the out-of-window path).

## 3. Validation

- **Example 37.a unit test** (blocking): heliocentric λ, β, r at
  JD 2448908.5 match the printed values to 1e-5°, 1e-5°, 1e-6 AU.
- **Table integrity test** (blocking): row count 43, and an FNV-1a checksum
  of the table's rendered coefficients pinned, so a stray edit is caught.
- **Packaged-reference sweep** (blocking, in `pleiades-vsop87` with
  `pleiades-data` as a dev-dependency; `pleiades-data` does not depend on
  `pleiades-vsop87`, so there is no cycle): `Vsop87Backend` Pluto against
  `PackagedDataBackend` Pluto every 30 days over 1900-01-01 – 2099-12-31
  (≈ 2430 samples). Ceilings = ceil(1.5 × measured max) from the first full
  run, recorded with the maxima in the source comment: expected ≈ 5″ λ,
  0.6″ β, 5e-4 AU. The packaged series is itself gated at about 1″ against
  Swiss Ephemeris (`validate-helio-position`), so this is a Swiss Ephemeris
  bound to within that.
- **Existing Swiss Ephemeris points** in `src/tests/pluto.rs` tighten from
  1°/0.5 AU to the sweep ceilings.
- **Window edges** (blocking): just inside each edge the fit path is used,
  just outside the fallback, and both yield finite results with a speed.
- **`pleiades-validate` comparison:** the `ComparisonToleranceScope::Pluto`
  thresholds (45° λ, 1° β, 0.25 AU) drop to the outer-planet thresholds the
  other bodies use, since Pluto now meets them against the JPL snapshot
  rows (all within 1885–2099).
- **Not regenerated:** the crossings, helio-position and apparent goldens all
  run on the packaged backend; nothing there moves.

## 4. Documentation and bookkeeping

- Crate README and workspace README backend row: Pluto is a periodic-term fit
  at arcsecond class over 1885–2099, mean elements outside.
- `docs/follow-ups.md`: a resolved entry for #129 with the measured sweep.
- `crates/pleiades-validate/src/crossings_validation.rs:48-52`: correct the
  comment that says the gate's backend serves Pluto from a mean-element
  fallback (that gate uses the packaged backend).
- Compatibility profile: summary sentence and additions entry, profile id
  bump, content checksum update.
- `plan/status/02-next-slice-candidates.md`: mark "Resolve Pluto" done.

## Error handling

| Condition | Behaviour |
|---|---|
| Instant inside 1885–2099 | periodic-term fit |
| Instant outside the window | mean-element fallback, `Approximate` quality |
| Non-finite instant | existing request validation, unchanged |

## Success criteria

1. The packaged-reference sweep passes in the blocking tier at arcsecond
   ceilings over 1900–2099.
2. Example 37.a reproduces to the printed digits.
3. Pluto's claim is `Constrained`/`Moderate`/`AlgorithmicModel`.
4. Every other body's output is bit-identical (VSOP87B paths untouched).
5. `mise run ci` green; nightly green on the branch.

## Resolved review questions

Spec approved 2026-10-04 with the defaults:

1. Outside 1885–2099 Pluto keeps the mean-element fallback (no fail-closed).
2. `Vsop87BodySourceKind` gains the variant and becomes `#[non_exhaustive]`
   in the same breaking change.
