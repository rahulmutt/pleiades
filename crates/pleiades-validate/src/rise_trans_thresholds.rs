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
//! ## Remaining known finding — below/near-horizon refraction-model floor
//! ## (Task 17's stated scope, NOT fixed here)
//!
//! Even with the dip removed, Sun/Moon (large angular disc, ~0.25-0.27 deg
//! semidiameter) rise/set events with refraction enabled and no custom
//! horizon show a 15-22s time residual against `se_jd_ut`, while point
//! bodies (Mars, stars) under the same conditions show < 3.5s. The azalt
//! corpus shows the identical signature directly: azimuth/true-altitude are
//! sub-arcsecond everywhere, but apparent (refracted) altitude diverges
//! specifically for BELOW-horizon points (true altitude < 0 deg): up to
//! ~282" there, vs. a clean <= 7.61" for on/above-horizon points. This is
//! consistent with the engine's Bennett-forward refraction formula
//! disagreeing with SE's own refraction algorithm by an amount that grows
//! sharply near and below the horizon (amplified for Sun/Moon rise/set
//! because their disc-edge crossing sits exactly there). This is squarely
//! the plan's own anticipated Task 17 scope ("pin the below-horizon
//! refraction branch"); it is NOT papered over with an inflated ceiling here:
//! the rise/set "refraction floor" category gets its own honestly-measured
//! ceiling (still far tighter than the ~800s an all-rows-uniform ceiling
//! would require), and azalt's apparent-altitude check is gated only for
//! on/above-horizon rows (`se_true_alt_deg >= 0`) — below-horizon apparent
//! altitude is computed and reported (informational,
//! `RiseTransReport::max_below_horizon_apparent_alt_residual_arcsec`) but not
//! gated, with an explicit comment flagging it for Task 17 to tighten.

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
/// exactly at the geometric horizon — squarely in the below-horizon
/// refraction-model floor described above (Task 17's scope). Measured max:
/// 21.9052 s (Moon rise, lat 40, elev 10m). Ceiling = ceil(1.4 x 21.9052)
/// rounded to 31.0 s. NOT further tightened here; Task 17 owns closing this
/// gap (see the module doc and `rise_trans_validation::is_refraction_floor_row`).
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

/// Apparent (refracted) altitude angle-parity ceiling (arcseconds), gated
/// ONLY for rows at or above the horizon (`se_true_alt_deg >= 0.0`); see the
/// module doc for why below-horizon rows are excluded (Task 17's scope) and
/// tracked informationally instead. Measured max over the on/above-horizon
/// subset: 7.6052" (lat 0, true altitude 9.964 deg). Ceiling = ceil(1.4 x
/// 7.6052) rounded to 11.0". Below-horizon rows range up to ~282" and are
/// NOT gated by this constant.
pub const APPARENT_ALTITUDE_ARCSEC: f64 = 11.0;

/// Self-consistency ceiling (arcseconds): azalt round-trip
/// (`horizontal_to_equatorial(horizontal(x)) ~ x`), the meridian-transit
/// hour-angle-zero check, and the refraction round-trip
/// (`true_from_apparent(apparent_from_true(h)) ~ h`) evaluated at
/// representative non-grazing altitudes (30 deg, 60 deg, 85 deg — chosen away
/// from the horizon, where Bennett/Saemundsson's inherent forward/inverse
/// mismatch is a known, documented non-bug rather than an engine defect;
/// exact horizon behavior is exercised by the Tier-2 rise/set corpus rows
/// instead). Measured max across all three checks: 1.81" (refraction
/// round-trip at 30 deg true altitude) — unaffected by either Task 16 engine
/// fix (neither touches these code paths). Ceiling = ceil(1.4 x 1.81) rounded
/// to 2.6".
pub const SELF_CONSISTENCY_ARCSEC: f64 = 2.6;
