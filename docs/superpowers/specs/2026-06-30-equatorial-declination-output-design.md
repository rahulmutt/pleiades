# Apparent equatorial (RA/Dec) of date for release-grade bodies

**Date:** 2026-06-30
**Status:** Design approved, pending spec review
**Origin:** Follow-up sub-project recorded at the end of
`docs/superpowers/specs/2026-06-29-eclipse-subsystem-design.md` ("Equatorial /
declination output"), reiterated as the next-queued item in
`docs/follow-ups.md` (FU-2 "Next queued") and the `PLAN.md` status line
("equatorial/declination follow-up is the next queued sub-project").

## Problem

The `EquatorialCoordinates` type and the `EclipticCoordinates::to_equatorial` /
`EquatorialCoordinates::to_ecliptic` rotations already exist and are
round-trip-stable and unit-tested
(`crates/pleiades-types/src/coordinates.rs`). The `EphemerisResult.equatorial`
field exists (`crates/pleiades-backend/src/result.rs:60`). First-party backends
already populate it — e.g. `pleiades-data` computes
`ecliptic.to_equatorial(req.instant.mean_obliquity())`
(`crates/pleiades-data/src/backend.rs:93,318`).

The gap is at the **chart layer**. The chart apparent path rewrites
`position.ecliptic` to apparent-of-date (precession + nutation-in-longitude,
optionally topocentric) in `crates/pleiades-core/src/chart/mod.rs`, but it
**never touches `position.equatorial`**. So on an apparent-mode release-grade
row, the equatorial field is the backend's **mean-obliquity, J2000-ish** RA/Dec,
inconsistent with the apparent ecliptic sitting beside it. There is also no
validation gate over the equatorial channel.

This sub-project closes that gap: produce an **apparent equatorial of date
(RA/Dec)** at the chart layer, consistent with the final apparent ecliptic,
using **true obliquity (mean obliquity + nutation-in-obliquity Δε)** — and lock
it in behind a fail-closed gate.

## Goals

1. For every **release-grade body** (the bodies that already receive the
   apparent-ecliptic transform), populate `position.equatorial` with the
   **apparent equatorial of date**, derived from the final apparent ecliptic of
   date using true obliquity.
2. Keep the change a single source of truth: derive equatorial from the already
   apparent-corrected (and optionally topocentric) **tropical** ecliptic, so it
   inherits precession, nutation, annual aberration, and the topocentric
   correction with no duplicated correction logic.
3. Preserve the standing architecture invariant: first-party backends remain
   **mean-only and J2000 at the backend boundary**. Of-date equatorial lives at
   the chart layer, exactly like the apparent ecliptic.
4. Lock the result in behind a fail-closed numeric gate, validated against
   **two** authorities: JPL Horizons apparent RA/Dec (accuracy) and Swiss
   Ephemeris `SEFLG_EQUATORIAL` (convention/sign/units parity).
5. Keep the of-date rotation in an isolated, independently unit-testable pure
   helper, reusable by the eclipse crate and future RA/Dec consumers.

## Non-goals (named so they are not silently assumed done)

- **Declination-parallel aspects** and **out-of-bounds (OOB) detection** — this
  slice populates the field that *enables* them; the chart features are
  downstream.
- **Native equatorial backend output** — backends stay mean/J2000; this is a
  chart-layer derivation.
- **Equatorial motion / speed channels** — position only.
- **Right-ascension-based house systems or any RA-driven chart math** — out of
  scope here.

## Frame definition

The chart `equatorial` field reports the **apparent equatorial of date**:

- True obliquity `ε_true = mean_obliquity_degrees(jd_tt) + Δε(jd_tt)/3600`,
  where `Δε` is nutation-in-obliquity from `pleiades_apparent::nutation::nutation`.
- RA/Dec are the rotation of the **tropical apparent ecliptic of date** by
  `ε_true`, via the existing `EclipticCoordinates::to_equatorial`.
- The equatorial frame of a row therefore **matches the ecliptic frame of that
  row**: apparent-of-date for apparent rows, mean for mean-fallback rows (see
  edge cases).

RA/Dec are an equatorial **frame** and are independent of the zodiac (ayanamsa).
The derivation is performed on the **tropical** ecliptic, strictly *before* the
sidereal ayanamsa longitude-shift, so the equatorial output is identical for the
same instant whether the chart is tropical or sidereal.

## Architecture & data flow

### New helper (`pleiades-apparent`)

```rust
/// Apparent equatorial of date from the tropical apparent ecliptic of date.
pub fn apparent_equatorial_of_date(
    tropical_apparent_ecliptic: EclipticCoordinates,
    jd_tt: f64,
) -> Result<EquatorialCoordinates, ApparentPlaceError>;
```

Computes `ε_true` (mean obliquity + Δε) and returns
`tropical_apparent_ecliptic.to_equatorial(Angle::from_degrees(ε_true))`. Pure,
no I/O, independently unit-testable. Lives in `pleiades-apparent` because that
crate owns nutation/obliquity; exported so the eclipse crate and any future
RA/Dec consumer reuse the exact same of-date definition.

### Chart-layer wiring (`pleiades-core/src/chart/mod.rs`)

Within the per-body assembly closure, the existing order is:

1. apparent block sets `*ecliptic = outcome.ecliptic` (tropical apparent of date);
2. topocentric block (opt-in) sets `*ecliptic = topo.ecliptic` (tropical
   apparent topocentric);
3. sidereal re-apply shifts `ecliptic.longitude` by the ayanamsa;
4. sign re-derive; `BodyPlacement` constructed.

Insert the equatorial derivation **after step 2 and before step 3**, guarded by
`apparent.is_some()`: call `apparent_equatorial_of_date` on the current
(tropical, of-date) `position.ecliptic` and write the result into
`position.equatorial`. The `jd_tt` and `ε_true` already computed in the
topocentric branch are hoisted so both branches share a single obliquity
computation.

**Per-body data flow:** backend mean/J2000 ecliptic → apparent ecliptic of date
(precession + nutation + aberration) → [optional topocentric] → **rotate
tropical ecliptic to equatorial of date** → sidereal longitude shift touches only
`ecliptic.longitude` (equatorial untouched).

No backend changes. No change to `EquatorialCoordinates` or the rotation math.

## Edge cases

- **Tropical vs sidereal:** equatorial derived from the tropical ecliptic before
  the ayanamsa shift; identical for both modes (asserted by a parity test).
- **Mean-fallback bodies** (`apparent.is_none()`): not recomputed. The backend's
  mean-obliquity equatorial stays, consistent with the mean ecliptic on that row.
  Documented: the row's equatorial frame matches its ecliptic frame.
- **Apsides** (`TrueApogee` / `TruePerigee`): already finalized to apparent-of-date
  via `apparent_apsis_position`; the same rotation applies uniformly — no
  special-casing.
- **Pole conditioning:** the forward rotation is well-conditioned; only the gate
  comparison needs care at high `|Dec|` (see Validation).
- **Distance channel:** preserved through the rotation (already handled by
  `to_equatorial`).

## Validation (two authorities)

Both arms fail-closed: a missing or short corpus floors the validated-row count
(the pattern used by the `validate-lilith` fail-closed integrity fix), so an
absent reference can never silently pass.

### Accuracy gate — JPL Horizons (primary, release-grade)

Extend the **existing** `apparent-goldens` corpus
(`crates/pleiades-validate/data/apparent-goldens.csv`): re-run the regen script
(`crates/pleiades-validate/scripts/regen-apparent-goldens.sh`) adding Horizons
`QUANTITIES='2'` (apparent RA & DEC, referred to the true equator and equinox of
date, with light-time, deflection, and stellar aberration) alongside the current
`ObsEcLon` columns — same epochs, same bodies, one authority consistent with the
ecliptic we derive from. A `validate-equatorial` check asserts per-body RA/Dec
residuals under model-limit ceilings (documented inline in the CSV header, the
way the ecliptic ceilings are). Wired into the release-gate numeric-gate set.

### Convention-parity cross-check — Swiss Ephemeris (secondary)

New `tools/se-equatorial-reference` using `SEFLG_EQUATORIAL` with default
apparent flags (nutation + aberration on), generating a committed CSV consumed
via `include_str!` (mirrors `tools/se-lilith-reference`). It carries the same
build-env note: building the tool requires `libclang-dev` + `LIBCLANG_PATH`, but
the gate never rebuilds it — the committed CSV is read directly. Its ceilings are
**deliberately looser**: its purpose is to catch convention/sign/units mistakes
(RA in degrees vs hours, equinox handling, Dec sign), not to re-assert
sub-arcsec accuracy against the less-accurate SE Moshier model. Enforced by a
`validate-equatorial-se` arm (or a second arm of the same gate).

### Pole conditioning in comparisons

RA residuals are compared as `ΔRA·cos(Dec)` (equivalently, angular separation on
the sphere) so near-pole rows do not false-fail; Dec is compared signed directly.

## Error handling

The helper returns `Result` (its only fallible step is `nutation`; mean
obliquity is an infallible polynomial). If `nutation` fails (e.g. out-of-window
`jd_tt`), the chart **degrades gracefully** — `position.equatorial` is left as
the backend's mean value rather than failing the whole chart, mirroring the
existing apparent mean-fallback philosophy. No new hard-failure path is added to
chart building.

## Testing

- **Unit (`pleiades-apparent`):** helper round-trips against `to_ecliptic` with
  the same obliquity; a known-epoch fixture (Meeus worked example) checks
  absolute RA/Dec.
- **Chart (`pleiades-core`):** equatorial populated and apparent-of-date for a
  release-grade body; tropical-vs-sidereal equatorial **parity**; a topocentric
  row's equatorial reflects the topocentric ecliptic; a mean-fallback row keeps
  the mean equatorial.
- **Gate (`pleiades-validate`):** both corpora parse; residuals under ceilings;
  fail-closed on a truncated corpus.
- New corpus-backed tests follow the repo's test-speed tiering conventions.

## Claims, compatibility & docs impact

This promotes a release claim, so:

- **Compatibility profile bump** (from `0.7.1`): the metadata stating equatorial
  is "derived via mean-obliquity transforms when supported" becomes "apparent
  equatorial of date (true obliquity) for release-grade bodies."
- **README** "Important current limits" and the request-metadata text (e.g.
  `crates/pleiades-backend/src/request_tests.rs:802`) updated to describe
  apparent-of-date equatorial.
- **PLAN.md** status line: drop "equatorial/declination follow-up is the next
  queued sub-project"; record it done.
- **`docs/follow-ups.md`:** new resolved entry; FU-2's "Next queued" pointer
  marked done.
