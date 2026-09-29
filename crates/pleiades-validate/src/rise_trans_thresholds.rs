//! Per-row parity ceilings for `validate-rise-trans`, mirroring the crossings/
//! angles thresholds modules. Time ceilings are seconds; angle ceilings
//! arcsec.
//!
//! All ceilings below are MEASURED from the committed `rise-trans-corpus`
//! (Task 15, 50 rise-trans rows + 20 azalt rows) AFTER the two Task 16 engine
//! fixes landed (see `crates/pleiades-events/src/rise_trans.rs`):
//!
//! 1. **Elevation dip removed.** `standard_altitude` no longer subtracts a
//!    height-based horizon dip (`1.76'*sqrt(elev_m)`). Reading the vendored
//!    Swiss Ephemeris source (`libswisseph-sys-0.1.2/libswisseph/swecl.c`)
//!    confirmed `swe_rise_trans()` always calls `swe_rise_trans_true_hor`
//!    with `horhgt = 0`, and the height-based dip (`calc_dip`) is only
//!    computed when `horhgt == -100` — a sentinel this corpus's generator
//!    never requests. So SE's plain `swe_rise_trans` applies no elevation
//!    dip, and neither does the engine now.
//! 2. **Rise/set search bounded to `RISE_SET_SEARCH_SPAN_DAYS` (see
//!    `rise_trans.rs`).** `next_rise_set`'s `Rise`/`Set` arm now matches SE's
//!    own short-horizon search contract instead of scanning the entire
//!    ~190-year packaged window; a body that is circumpolar right now and
//!    stays that way for the whole span reports `None`, matching the
//!    corpus's 4 `none`/`none` rows exactly.
//!
//! With both fixes applied, every row in the corpus now falls into one of
//! four well-separated categories (see `is_grazing_row` /
//! `is_refraction_floor_row` in `rise_trans_validation.rs` for the exact
//! classification), each ceiling set to ~1.3-1.5x its measured max:
//!
//! - Point-body / no-refraction-floor rise-set rows ("tight").
//! - Sun/Moon rise-set rows with refraction enabled and no custom horizon
//!   offset ("refraction floor" — see below, Task 17's scope).
//! - The lat-66.5N winter Sun/Aldebaran rows ("grazing" — oblique path).
//! - Meridian transits (never touch `standard_altitude` at all, so entirely
//!   unaffected by fix 1; fix 2 doesn't apply to transits either).
//!
//! ## Task 17 — below-horizon refraction branch pinned to SE
//!
//! `pleiades_apparent::apparent_from_true`/`true_from_apparent` (see that
//! module's doc) now hold Bennett/Saemundsson's own refraction value fixed at
//! true/apparent h=-1 deg and fade it linearly to zero by h=-10 deg, matching
//! every below-horizon row in the committed `azalt.csv` corpus (SE reports
//! `se_apparent_alt_deg == se_true_alt_deg` there) to within ~9 arcsec at the
//! shallowest tested row and a small fraction of an arcsec at every deeper
//! one — down from up to ~282 arcsec previously. `h >= -1 deg` (which
//! includes every actual rise/set crossing this corpus exercises) is
//! completely unchanged, so this fix is measured to have NO effect on the
//! rise/set "refraction floor" category below (its measured max is bit-for-
//! bit identical before and after: 21.9052 s). This is expected and
//! documented, not glossed over: the corpus has no below-horizon ground truth
//! in the narrow band ([-1, 0) deg) where Sun/Moon disc-edge crossings
//! actually happen, so there is nothing to fit that residual against without
//! guessing; see `pleiades_apparent::refraction::apparent_from_true_below_horizon`'s
//! doc for why reproducing SE's own (discontinuous) below-horizon model
//! exactly was tried and rejected (it regressed a different row via
//! bisection-on-a-jump). Effects on the two ceilings below:
//!
//! - `RISE_SET_SECONDS_REFRACTION_FLOOR`: UNCHANGED (measured max unchanged).
//! - `APPARENT_ALTITUDE_ARCSEC`: TIGHTENED, and now gates below-horizon azalt
//!   rows too (previously informational-only) — see its doc below.
//!
//! ## Issue #74 — TDB time base
//!
//! The engine now reads the `TimeScale` tag on its query instants, rotates
//! the sky with UT1, samples bodies in TDB, and returns genuine TDB instants,
//! so the gate compares against `se_jd_tdb` (see `rise_trans_validation`'s
//! time-base note). Every time ceiling was re-measured against that column;
//! all four CEILINGS are retained unchanged, since no measured max moved by
//! more than a few tenths of a second except the transits, which improved
//! markedly (the Moon had been sampled ΔT earlier than the sky it was placed
//! against — a systematic ~2 s term on moon transits that is now gone):
//!
//! | category          | pre-#74 max (vs `se_jd_ut`) | post-#74 max (vs `se_jd_tdb`) | ceiling |
//! |-------------------|-----------------------------|-------------------------------|---------|
//! | tight             | 3.4631 s                    | 3.039 s                       | 5.0 s   |
//! | refraction floor  | 21.9052 s                   | 21.615 s                      | 31.0 s  |
//! | grazing           | 110.8948 s                  | 110.989 s                     | 160.0 s |
//! | transit           | 2.8894 s                    | 1.001 s                       | 4.0 s   |
//!
//! The per-constant docs below keep their original measured basis (the
//! numbers the ceilings were derived from); the table under "Issues #80/#81"
//! is the current measurement.
//!
//! ## Issues #80/#81 — returned instants are settled
//!
//! The engine now returns the later end of its final refinement bracket
//! instead of the midpoint, so every returned instant trails its event by
//! less than the 0.5 s refinement tolerance and never precedes it (that is
//! what lets a search chained from a returned instant step past the event).
//! Relative to the midpoint, instants moved later by up to 0.22 s. Every
//! time ceiling was re-measured and is retained unchanged:
//!
//! | category          | post-#74 max | post-#80/#81 max | ceiling |
//! |-------------------|--------------|------------------|---------|
//! | tight             | 3.039 s      | 3.259 s          | 5.0 s   |
//! | refraction floor  | 21.615 s     | 21.446 s         | 31.0 s  |
//! | grazing           | 110.989 s    | 111.209 s        | 160.0 s |
//! | transit           | 1.001 s      | 0.781 s          | 4.0 s   |
//!
//! One Tier-1 check changed shape. The hour angle at a returned transit used
//! to be gated two-sided under `SELF_CONSISTENCY_ARCSEC`; it is now gated
//! one-sided under `TRANSIT_HOUR_ANGLE_ARCSEC` (past the meridian by no more
//! than the tolerance's worth of rotation, and not before it), which states
//! the engine's contract rather than one fixture's position in its bracket.

/// Rise/set time-parity ceiling (seconds) for a well-conditioned,
/// non-grazing, non-refraction-floor row (point-like body: star or
/// Mars-class planet; OR a Sun/Moon row with refraction disabled or a custom
/// horizon offset that moves the crossing away from the geometric horizon).
/// Measured max over this subset: 3.4631 s (Sun rise, lat 40, `horizon_plus5`
/// preset — refraction on, but the +5 deg custom horizon keeps the crossing
/// well clear of the near-horizon refraction floor). Ceiling = ceil(1.4 x
/// 3.4631) rounded to 5.0 s.
pub const RISE_SET_SECONDS_TIGHT: f64 = 5.0;

/// Rise/set time-parity ceiling (seconds) for Sun/Moon rows where refraction
/// is enabled and no custom horizon offset is given, so the event is defined
/// exactly at the geometric horizon. Measured max: 21.9052 s (Moon rise, lat
/// 40, elev 10m) — UNCHANGED by Task 17's below-horizon refraction fix,
/// because every such crossing's true altitude stays within [-1, 0) deg,
/// which that fix deliberately left untouched (see the module doc's "Task
/// 17" section and `pleiades_apparent::refraction::apparent_from_true_below_horizon`'s
/// doc for why). Ceiling = ceil(1.4 x 21.9052) rounded to 31.0 s — unchanged
/// from its pre-Task-17 value, since the measured max didn't move; this is
/// an honest, not-yet-closed gap (see `rise_trans_validation::is_refraction_floor_row`),
/// not an inflated ceiling.
pub const RISE_SET_SECONDS_REFRACTION_FLOOR: f64 = 31.0;

/// Loosened ceiling for genuinely ill-conditioned rows: near-circumpolar /
/// oblique-path geometry where `d(altitude)/dt -> 0` amplifies model
/// disagreement into a large time residual. The only rows classified this
/// way are the Sun/Aldebaran rise/set at lat 66.5N (near the Arctic Circle —
/// the winter Sun's rise/set path is extremely oblique to the horizon).
/// Measured max (elevation 0, so unaffected by the dip fix, and stacked with
/// the refraction floor for the Sun rows): 110.8948 s. Ceiling = ceil(1.4 x
/// 110.8948) rounded to 160.0 s.
pub const RISE_SET_SECONDS_GRAZING: f64 = 160.0;

/// Meridian-transit time-parity ceiling (seconds). Transits never call
/// `standard_altitude` (no disc/dip term, no horizon residual at all — they
/// root-find the hour-angle zero instead), so this ceiling is unaffected by
/// either Task 16 engine fix. Well-conditioned (hour angle advances at the
/// full sidereal rate through the crossing), but the Moon's ~0.55 deg/hr
/// motion still shows up: measured max over all 14 transit rows: 2.8894 s
/// (Moon). Ceiling = ceil(1.4 x 2.8894) rounded to 4.0 s.
pub const TRANSIT_SECONDS: f64 = 4.0;

/// Azimuth angle-parity ceiling (arcseconds), mod-360 wraparound. Measured
/// max: 0.1146". Ceiling = ceil(1.4 x 0.1146) rounded to 0.2".
pub const AZIMUTH_ARCSEC: f64 = 0.2;

/// True (unrefracted) altitude angle-parity ceiling (arcseconds). Measured
/// max: 0.0411". Ceiling = ceil(1.4 x 0.0411) rounded to 0.1".
pub const TRUE_ALTITUDE_ARCSEC: f64 = 0.1;

/// Apparent (refracted) altitude angle-parity ceiling (arcseconds), gated for
/// EVERY azalt row, on/above AND below the horizon (Task 17: previously
/// gated above-horizon rows only, with below-horizon tracked informationally
/// at up to ~282" — see the module doc's "Task 17" section for how the
/// below-horizon refraction fix brought that down). Measured max over
/// on/above-horizon rows: 7.6052" (lat 0, true altitude 9.964 deg) —
/// unaffected by Task 17 (that fix only touches `h < 0`). Measured max over
/// below-horizon rows: 9.1284" (lat 0, true altitude -9.964 deg — the
/// shallowest below-horizon row, right at the edge of the refraction fix's
/// hold-then-fade blend; see `pleiades_apparent::refraction`'s doc). Overall
/// measured max: 9.1284". Ceiling = ceil(1.4 x 9.1284) rounded to 13.0".
pub const APPARENT_ALTITUDE_ARCSEC: f64 = 13.0;

/// Self-consistency ceiling (arcseconds): azalt round-trip
/// (`horizontal_to_equatorial(horizontal(x)) ~ x`) and the refraction
/// round-trip
/// (`true_from_apparent(apparent_from_true(h)) ~ h`) evaluated at
/// representative non-grazing altitudes (30 deg, 60 deg, 85 deg — chosen away
/// from the horizon, where Bennett/Saemundsson's inherent forward/inverse
/// mismatch is a known, documented non-bug rather than an engine defect;
/// exact horizon behavior is exercised by the Tier-2 rise/set corpus rows
/// instead). Measured max across both checks: 1.81" (refraction
/// round-trip at 30 deg true altitude) — unaffected by either Task 16 engine
/// fix (neither touches these code paths). Ceiling = ceil(1.4 x 1.81) rounded
/// to 2.6". The meridian-transit hour-angle check was gated here too until
/// issues #80/#81; it now has its own bound, [`TRANSIT_HOUR_ANGLE_ARCSEC`].
pub const SELF_CONSISTENCY_ARCSEC: f64 = 2.6;

/// Ceiling (arcseconds) on the hour angle at a returned upper transit,
/// measured PAST the meridian. Since issues #80/#81 the engine returns the
/// settled end of its refinement bracket, so a returned transit instant
/// trails the transit by less than the 0.5 s refinement tolerance and never
/// precedes it. The hour angle there is therefore one-sided: between zero and
/// the tolerance times the sidereal rate, 0.5 s x 15.0411"/s = 7.52".
///
/// Unlike the other ceilings in this module this one is ANALYTIC, not 1.4 x a
/// measured max: the residual is the position of the transit inside a
/// bisection bracket, uniform over that bracket, so any tighter figure would
/// pin one fixture's luck rather than the engine's contract. Measured on the
/// gate's Aldebaran fixture: 4.065" (0.27 s past the transit). Before the
/// change the instant was the bracket's midpoint, the residual two-sided and
/// at most half as large, and the check shared `SELF_CONSISTENCY_ARCSEC`.
pub const TRANSIT_HOUR_ANGLE_ARCSEC: f64 = 7.6;

/// How far BEFORE the meridian (arcseconds) a returned transit may read
/// before the gate calls it early. The engine guarantees none at all; this
/// slack only absorbs rounding between the engine's UT1 re-expression of the
/// instant and the gate's own (`pleiades_apparent::ut1_instant`), at 15"/s
/// under a millisecond.
pub const TRANSIT_HOUR_ANGLE_EARLY_SLACK_ARCSEC: f64 = 0.01;
