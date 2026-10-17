//! Emits a Swiss Ephemeris exact-aspect reference corpus to STDOUT as CSV.
//! Swiss Ephemeris has no aspect finder, so an exact aspect is located here
//! from `swe_calc(jd_tt, body, iflag | SEFLG_SPEED)` for the two bodies: the
//! signed separation `wrap180(lon(first) − lon(second))` is scanned on a
//! fixed grid, each grid step is split at the zero of the relative longitude
//! speed (the separation's turning point), and each sign change of
//! `separation − level` is bisected to 1e-7 day. The levels of an angle are
//! `+angle` and `−angle` (one level for 0 and 180). Swiss Ephemeris' "ET"
//! argument is TT.
//!
//! Groups:
//!   - `geo`:   geocentric apparent, tropical, true equinox of date (SE
//!     default flags). Seven planet pairs over the pleiades-events window
//!     less five days at each end; Sun–Moon over 1990–2030.
//!   - `mean`:  geocentric geometric place in the mean ecliptic and equinox
//!     of date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT).
//!     Mercury–Venus and Mars–Saturn over 1990–2030.
//!   - `helio`: geometric heliocentric place (SEFLG_HELCTR | SEFLG_TRUEPOS),
//!     the place `se-helio-reference` uses. Mars–Jupiter over the full span.
//!
//! Grid: 0.05 day; 0.01 day for a pair with the Moon.
//!
//! Grazes: the corpus must hold no model-dependent event. If any turning
//! point of a pair's separation comes within 30 arcseconds of one of its
//! levels, on either side, the tool panics and names the pair, angle and
//! instant, and no corpus is written. Counts per pair and angle, and each
//! pair's turning points and closest turn to a level, go to STDERR.
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH), no data files needed.
//!
//! Asteroid mode (`--asteroids --out <dir> [--ephe <dir>]`): `geo` aspects of
//! the Sun with Ceres, Pallas, Juno and Vesta (SE ids 17-20) and of Mars with
//! Vesta and Jupiter with Ceres (pairs whose separation turns) over the full
//! span, and of the Moon with Ceres over 1990-2030, with the `geo` flags but
//! SEFLG_SWIEPH instead of SEFLG_MOSEPH, from the `seas_18`/`sepl_18`/`semo_18`
//! files pinned in `pins.rs` (verified before anything is written). The same
//! 30-arcsecond graze refusal applies. Writes `<dir>/asteroids.csv` and
//! `<dir>/asteroids-manifest.txt` itself, so no devenv banner reaches them:
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/se-aspects-reference/Cargo.toml`, then
//! `tools/se-aspects-reference/target/release/se-aspects-reference --asteroids --out crates/pleiades-validate/data/aspects-corpus`.
//! The ephe directory is `$SE_EPHE_PATH`, else `--ephe`, else this tool's `data/`.
//!
//! Lunar-point mode (`--lunar-points --out <dir> [--ephe <dir>]`): `geo`
//! aspects of the true node (SE_TRUE_NODE, id 11) and the mean node (id 10)
//! with the Sun, Mars, Jupiter, Saturn, Uranus, Neptune and Pluto, and of the
//! osculating apogee (SE_OSCU_APOG, id 13) and the mean apogee (id 12) with the
//! Sun, Jupiter, Saturn and Pluto, over the full span, with the `geo` flags
//! but SEFLG_SWIEPH, from the `sepl_18`/`semo_18` files pinned in `pins.rs`.
//! The true node's speed oscillates with a period of about two weeks and
//! touches zero, so its separation from a slow planet turns thousands of
//! times; the grid is 0.005 day. Unlike the planet and asteroid modes, a turn
//! within the pair's graze margin of a level (3 arcseconds; 12 for the
//! osculating apogee) is not refused but recorded as a graze window on the
//! grid (issue #168 (c)). Writes `<dir>/lunar-points.csv`,
//! `<dir>/lunar-points-manifest.txt`, `<dir>/lunar-points-grazes.csv` and
//! `<dir>/lunar-points-grazes-manifest.txt`. Build with
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/se-aspects-reference/Cargo.toml`
//! or, outside devenv, `env CFLAGS=-std=gnu17 LIBCLANG_PATH=<libclang> cargo build --release --offline --manifest-path tools/se-aspects-reference/Cargo.toml`,
//! where `<libclang>` is the libclang `devenv.nix` provides (`LIBCLANG_PATH` =
//! `${pkgs.libclang.lib}/lib`), for example `/nix/store/…-clang-<ver>-lib/lib`;
//! run `tools/se-aspects-reference/target/release/se-aspects-reference --lunar-points --out crates/pleiades-validate/data/aspects-corpus`.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
//!    --manifest-path tools/se-aspects-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/aspects-corpus/aspects.csv`

mod pins;

use std::ffi::CStr;
use std::fmt::Write as _;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_set_ephe_path};

const SEFLG_SWIEPH: c_int = 2;
const SEFLG_MOSEPH: c_int = 4;
const SEFLG_HELCTR: c_int = 8;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration

const GEO: c_int = SEFLG_MOSEPH | SEFLG_SPEED;
const MEAN: c_int = GEO | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT;
const HELIO: c_int = GEO | SEFLG_HELCTR | SEFLG_TRUEPOS;
/// `GEO` with the SWIEPH data files in place of Moshier.
const GEO_SWIEPH: c_int = (GEO & !SEFLG_MOSEPH) | SEFLG_SWIEPH;
const DEFAULT_EPHE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");

/// (Swiss Ephemeris body id, name as written to the CSV).
type Body = (c_int, &'static str);
const SUN: Body = (0, "Sun");
const MOON: Body = (1, "Moon");
const MERCURY: Body = (2, "Mercury");
const VENUS: Body = (3, "Venus");
const MARS: Body = (4, "Mars");
const JUPITER: Body = (5, "Jupiter");
const SATURN: Body = (6, "Saturn");
const URANUS: Body = (7, "Uranus");
const NEPTUNE: Body = (8, "Neptune");
const PLUTO: Body = (9, "Pluto");
const CERES: Body = (17, "Ceres");
const PALLAS: Body = (18, "Pallas");
const JUNO: Body = (19, "Juno");
const VESTA: Body = (20, "Vesta");
/// SE_TRUE_NODE: the osculating (true) lunar ascending node.
const TRUE_NODE: Body = (11, "TrueNode");
/// SE_OSCU_APOG: the osculating lunar apogee ("true Lilith").
const TRUE_APOGEE: Body = (13, "TrueApogee");
/// SE_MEAN_NODE: the mean lunar ascending node (analytic).
const MEAN_NODE: Body = (10, "MeanNode");
/// SE_MEAN_APOG: the mean lunar apogee (analytic).
const MEAN_APOGEE: Body = (12, "MeanApogee");

/// The pleiades-events window (JD 2415020.5–2488069.5) less five days at each
/// end, so neither side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

const GRID_DAYS: f64 = 0.05;
const MOON_GRID_DAYS: f64 = 0.01;
const BISECT_TOLERANCE_DAYS: f64 = 1e-7;
/// 30 arcseconds: several times the largest difference between the Swiss
/// Ephemeris (Moshier or SWIEPH seas_18) and pleiades separations of the planet
/// and asteroid pairs.
const PLANET_GRAZE_MARGIN_DEG: f64 = 30.0 / 3600.0;
const ANGLES: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

/// The asteroid-mode pairs: `(first, second, span, grid)`, all `geo` with
/// `GEO_SWIEPH`. The separation of a Sun or Moon pair never turns; Mars–Vesta
/// and Jupiter–Ceres turn 188 and 315 times over the span (issue #253).
const ASTEROID_PAIRS: [(Body, Body, (f64, f64), f64); 7] = [
    (SUN, CERES, FULL_SPAN, GRID_DAYS),
    (SUN, PALLAS, FULL_SPAN, GRID_DAYS),
    (SUN, JUNO, FULL_SPAN, GRID_DAYS),
    (SUN, VESTA, FULL_SPAN, GRID_DAYS),
    (MOON, CERES, SHORT_SPAN, MOON_GRID_DAYS),
    (MARS, VESTA, FULL_SPAN, GRID_DAYS),
    (JUPITER, CERES, FULL_SPAN, GRID_DAYS),
];

/// The lunar-point grid: the true node's speed oscillates with a period of
/// about two weeks and touches zero, so its separation from a slow planet
/// turns thousands of times over the span (2392 for Jupiter, 6600 for Pluto).
const LUNAR_POINT_GRID_DAYS: f64 = 0.005;
/// 3 arcseconds, for the true node and the mean points: 3.6 times the largest
/// engine-minus-SE SWIEPH separation at a turning point measured for the true
/// node (0.82"). A turn this close to a level becomes a graze window, not an
/// error (issue #168 (c)).
const LUNAR_POINT_GRAZE_MARGIN_DEG: f64 = 3.0 / 3600.0;
/// 12 arcseconds, for the osculating apogee: 3.3 times the largest
/// engine-minus-SE SWIEPH separation at its turning points (3.6", spike
/// 2026-10-17, after the apsides mu fix of #271).
const TRUE_APOGEE_GRAZE_MARGIN_DEG: f64 = 12.0 / 3600.0;

/// The graze margin of a lunar-point pair, chosen by its first body.
fn lunar_point_margin_deg(first: Body) -> f64 {
    if first.0 == TRUE_APOGEE.0 {
        TRUE_APOGEE_GRAZE_MARGIN_DEG
    } else {
        LUNAR_POINT_GRAZE_MARGIN_DEG
    }
}

/// The lunar-point mode's pairs, all `geo` with `GEO_SWIEPH` over the full span.
const LUNAR_POINT_PAIRS: [(Body, Body); 22] = [
    (TRUE_NODE, SUN),
    (TRUE_NODE, MARS),
    (TRUE_NODE, JUPITER),
    (TRUE_NODE, SATURN),
    (TRUE_NODE, URANUS),
    (TRUE_NODE, NEPTUNE),
    (TRUE_NODE, PLUTO),
    (TRUE_APOGEE, SUN),
    (TRUE_APOGEE, JUPITER),
    (TRUE_APOGEE, SATURN),
    (TRUE_APOGEE, PLUTO),
    (MEAN_NODE, SUN),
    (MEAN_NODE, MARS),
    (MEAN_NODE, JUPITER),
    (MEAN_NODE, SATURN),
    (MEAN_NODE, URANUS),
    (MEAN_NODE, NEPTUNE),
    (MEAN_NODE, PLUTO),
    (MEAN_APOGEE, SUN),
    (MEAN_APOGEE, JUPITER),
    (MEAN_APOGEE, SATURN),
    (MEAN_APOGEE, PLUTO),
];

/// `(longitude_deg in [0, 360), longitude_speed_deg_per_day)`.
fn state(jd_tt: f64, ipl: c_int, iflag: c_int) -> (f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            ipl,
            iflag,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc(ipl={ipl}, iflag={iflag}) failed at jd_tt={jd_tt}: {msg}");
    }
    if iflag & SEFLG_SWIEPH != 0 && ret & SEFLG_SWIEPH == 0 {
        panic!("swe_calc(ipl={ipl}) fell back from SWIEPH at jd_tt={jd_tt} (ret={ret})");
    }
    assert!(
        xx[0].is_finite() && xx[3].is_finite(),
        "non-finite SE longitude or speed for ipl={ipl} at jd_tt={jd_tt}"
    );
    (xx[0].rem_euclid(360.0), xx[3])
}

fn wrap180(d: f64) -> f64 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// The pair at one instant.
#[derive(Clone, Copy)]
struct Sample {
    jd: f64,
    /// `wrap180(first_lon − second_lon)`.
    separation: f64,
    /// Longitude speed of the first body less that of the second.
    rel_speed: f64,
    first_lon: f64,
    second_lon: f64,
}

fn sample(jd: f64, first: Body, second: Body, iflag: c_int) -> Sample {
    let (first_lon, first_speed) = state(jd, first.0, iflag);
    let (second_lon, second_speed) = state(jd, second.0, iflag);
    Sample {
        jd,
        separation: wrap180(first_lon - second_lon),
        rel_speed: first_speed - second_speed,
        first_lon,
        second_lon,
    }
}

/// The values of the signed separation at which an angle is exact.
fn levels(angle: f64) -> Vec<f64> {
    if angle == 0.0 || angle == 180.0 {
        vec![angle]
    } else {
        vec![angle, -angle]
    }
}

/// Bisects a sign change of `f` over `[a, b]` and returns the midpoint of
/// the final bracket.
fn bisect(f: impl Fn(f64) -> f64, mut a: f64, mut f_a: f64, mut b: f64) -> f64 {
    while b - a > BISECT_TOLERANCE_DAYS {
        let mid = 0.5 * (a + b);
        let f_mid = f(mid);
        if (f_a <= 0.0) == (f_mid <= 0.0) {
            a = mid;
            f_a = f_mid;
        } else {
            b = mid;
        }
    }
    0.5 * (a + b)
}

/// What `scan` does with a turning point inside the margin of a level.
enum Graze<'a> {
    /// Abort the run: an event there would depend on the ephemeris
    /// (planet and asteroid modes).
    Refuse(f64),
    /// Record a graze window in `out` (lunar-point mode, issue #168 (c)).
    Window {
        margin_deg: f64,
        out: &'a mut dyn FnMut(String),
    },
}

impl Graze<'_> {
    fn margin_deg(&self) -> f64 {
        match self {
            Self::Refuse(margin) => *margin,
            Self::Window { margin_deg, .. } => *margin_deg,
        }
    }
}

/// Hands every exact aspect of the pair in `[lo, hi]`, for each angle, to `out`
/// as a CSV row.
// Mirrors write_graze_windows: the scan's inputs are all independent.
#[allow(clippy::too_many_arguments)]
fn scan(
    out: &mut dyn FnMut(String),
    group: &str,
    first: Body,
    second: Body,
    iflag: c_int,
    (lo, hi): (f64, f64),
    grid: f64,
    graze: Graze,
) {
    let margin_deg = graze.margin_deg();
    // (angle index, level, turn jd, miss deg) of each turn inside the margin.
    let mut close_turns: Vec<(usize, f64, f64, f64)> = Vec::new();
    let at = |jd: f64| sample(jd, first, second, iflag);
    let mut events: Vec<Vec<Sample>> = vec![Vec::new(); ANGLES.len()];
    let steps = ((hi - lo) / grid).floor() as u64;
    let mut turns = 0usize;
    let mut closest_miss = f64::INFINITY;
    let mut prev = at(lo);
    for k in 1..=steps {
        let cur = at(lo + k as f64 * grid);
        // Split the step at the separation's turning point, so that each
        // bracket is monotone and holds at most one event per level.
        let mut points = vec![prev];
        if (prev.rel_speed <= 0.0) != (cur.rel_speed <= 0.0) {
            let turn = at(bisect(
                |jd| at(jd).rel_speed,
                prev.jd,
                prev.rel_speed,
                cur.jd,
            ));
            turns += 1;
            for (index, angle) in ANGLES.iter().enumerate() {
                for level in levels(*angle) {
                    let miss = wrap180(turn.separation - level).abs();
                    closest_miss = closest_miss.min(miss);
                    if let Graze::Refuse(_) = graze {
                        assert!(
                            miss >= margin_deg,
                            "graze: {group},{},{} turns {:.2} arcsec from the {angle} degree level \
                             at jd_tt={:.5}, inside the {:.0} arcsec margin; an event here would \
                             depend on the ephemeris",
                            first.1,
                            second.1,
                            miss * 3600.0,
                            turn.jd,
                            margin_deg * 3600.0
                        );
                    } else if miss < margin_deg {
                        close_turns.push((index, level, turn.jd, miss));
                    }
                }
            }
            points.push(turn);
        }
        points.push(cur);
        for bracket in points.windows(2) {
            let (a, b) = (bracket[0], bracket[1]);
            for (index, angle) in ANGLES.iter().enumerate() {
                for level in levels(*angle) {
                    let f_a = wrap180(a.separation - level);
                    let f_b = wrap180(b.separation - level);
                    // The second test rejects the jump across the +-180 seam.
                    if (f_a <= 0.0) != (f_b <= 0.0) && (f_a - f_b).abs() < 180.0 {
                        let root = bisect(|jd| wrap180(at(jd).separation - level), a.jd, f_a, b.jd);
                        events[index].push(at(root));
                    }
                }
            }
        }
        prev = cur;
    }
    if turns == 0 {
        eprintln!("{group},{},{}: no turning points", first.1, second.1);
    } else {
        eprintln!(
            "{group},{},{}: {turns} turning points, closest {:.1} arcsec from a level",
            first.1,
            second.1,
            closest_miss * 3600.0
        );
    }
    for (index, angle) in ANGLES.iter().enumerate() {
        events[index].sort_by(|x, y| x.jd.total_cmp(&y.jd));
        eprintln!(
            "{group},{},{},{angle:.0}: {} events",
            first.1,
            second.1,
            events[index].len()
        );
        for event in &events[index] {
            out(format!(
                "{group},{},{},{angle:.0},{:.7},{:.9},{:.9},{:.9}",
                first.1, second.1, event.jd, event.first_lon, event.second_lon, event.rel_speed
            ));
        }
    }
    if let Graze::Window {
        out: windows_out, ..
    } = graze
    {
        write_graze_windows(
            windows_out,
            (group, first, second),
            &at,
            &events,
            &close_turns,
            (lo, hi),
            grid,
            margin_deg,
        );
    }
}

/// `(lo, hi, turn_jd, miss_deg, level)` of one graze window.
type Window = (f64, f64, f64, f64, f64);

/// Turns each close turn into a window `[lo, hi]` on the scan's grid: the first
/// grid instant on each side at which the separation is at least twice the
/// margin from the level. Merges overlapping windows of one angle, counts the
/// angle's events inside, and writes one grazes row per merged window. The
/// count's parity is set by the edges: even when both lie on one side of the
/// level, odd when they straddle it (a wiggle on a crossing trend). A count of
/// the other parity (a missed root), or a window reaching the span's edge, aborts.
#[allow(clippy::too_many_arguments)]
fn write_graze_windows(
    out: &mut dyn FnMut(String),
    (group, first, second): (&str, Body, Body),
    at: &dyn Fn(f64) -> Sample,
    events: &[Vec<Sample>],
    close_turns: &[(usize, f64, f64, f64)],
    (span_lo, span_hi): (f64, f64),
    grid: f64,
    margin_deg: f64,
) {
    let steps = ((span_hi - span_lo) / grid).floor() as i64;
    let instant = |k: i64| span_lo + k as f64 * grid;
    // (lo, hi, turn_jd, miss_deg, level) per angle. The two levels of one angle
    // are at least 120 degrees apart, so overlapping windows share their level.
    let mut windows: Vec<Vec<Window>> = vec![Vec::new(); ANGLES.len()];
    for &(index, level, turn_jd, miss) in close_turns {
        let far = |k: i64| wrap180(at(instant(k)).separation - level).abs() >= 2.0 * margin_deg;
        let mut k_lo = ((turn_jd - span_lo) / grid).floor() as i64;
        while k_lo > 0 && !far(k_lo) {
            k_lo -= 1;
        }
        let mut k_hi = k_lo + 1;
        while k_hi < steps && !far(k_hi) {
            k_hi += 1;
        }
        assert!(
            far(k_lo) && far(k_hi),
            "graze window of {group},{},{} at jd_tt={turn_jd:.5} reaches the span's edge",
            first.1,
            second.1
        );
        windows[index].push((instant(k_lo), instant(k_hi), turn_jd, miss, level));
    }
    for (index, angle) in ANGLES.iter().enumerate() {
        let mut list = std::mem::take(&mut windows[index]);
        list.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut merged: Vec<Window> = Vec::new();
        for w in list {
            match merged.last_mut() {
                Some(last) if w.0 <= last.1 => {
                    last.1 = last.1.max(w.1);
                    if w.3 < last.3 {
                        last.2 = w.2;
                        last.3 = w.3;
                    }
                }
                _ => merged.push(w),
            }
        }
        for (lo, hi, turn_jd, miss, level) in merged {
            let inside = events[index]
                .iter()
                .filter(|e| lo <= e.jd && e.jd <= hi)
                .count();
            let above = |jd: f64| wrap180(at(jd).separation - level) > 0.0;
            let straddles = above(lo) != above(hi);
            assert!(
                inside % 2 == usize::from(straddles),
                "graze window of {group},{},{},{angle:.0} [{lo:.7}, {hi:.7}] holds {inside} SE events, but its edges imply an {} count",
                first.1,
                second.1,
                if straddles { "odd" } else { "even" }
            );
            eprintln!(
                "{group},{},{},{angle:.0}: graze window [{lo:.5}, {hi:.5}], closest turn {:.2} arcsec at {turn_jd:.5}, {inside} SE events",
                first.1,
                second.1,
                miss * 3600.0
            );
            out(format!(
                "{group},{},{},{angle:.0},{turn_jd:.7},{:.3},{lo:.7},{hi:.7},{inside}",
                first.1,
                second.1,
                miss * 3600.0
            ));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--asteroids") {
        asteroids_main(&args);
        return;
    }
    if args.iter().any(|a| a == "--lunar-points") {
        lunar_points_main(&args);
        return;
    }
    let out = &mut |row: String| println!("{row}");
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), Moshier (SEFLG_MOSEPH, no data files).");
    println!("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)");
    println!("# equals +angle or -angle, scanned on a 0.05-day grid (0.01 day with the Moon), each step split at the zero");
    println!("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.");
    println!("# geo: apparent, tropical, true equinox of date (default flags); JD 2415025.5-2488064.5, Sun-Moon JD 2447892.5-2462502.5.");
    println!(
        "# mean: SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT, JD 2447892.5-2462502.5."
    );
    println!("# helio: SEFLG_HELCTR|SEFLG_TRUEPOS, JD 2415025.5-2488064.5.");
    println!("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.");
    println!("# No turning point of any pair's separation is within 30 arcsec of one of its levels (the tool fails otherwise).");
    println!(
        "group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day"
    );
    scan(
        out,
        "geo",
        SUN,
        MOON,
        GEO,
        SHORT_SPAN,
        MOON_GRID_DAYS,
        Graze::Refuse(PLANET_GRAZE_MARGIN_DEG),
    );
    for (first, second) in [
        (SUN, MERCURY),
        (MERCURY, VENUS),
        (VENUS, MARS),
        (MARS, JUPITER),
        (MARS, SATURN),
        (JUPITER, SATURN),
        (SATURN, PLUTO),
    ] {
        scan(
            out,
            "geo",
            first,
            second,
            GEO,
            FULL_SPAN,
            GRID_DAYS,
            Graze::Refuse(PLANET_GRAZE_MARGIN_DEG),
        );
    }
    for (first, second) in [(MERCURY, VENUS), (MARS, SATURN)] {
        scan(
            out,
            "mean",
            first,
            second,
            MEAN,
            SHORT_SPAN,
            GRID_DAYS,
            Graze::Refuse(PLANET_GRAZE_MARGIN_DEG),
        );
    }
    scan(
        out,
        "helio",
        MARS,
        JUPITER,
        HELIO,
        FULL_SPAN,
        GRID_DAYS,
        Graze::Refuse(PLANET_GRAZE_MARGIN_DEG),
    );
}

/// Reads `--out`, verifies the pinned SWIEPH files and points Swiss Ephemeris
/// at them; returns the output directory.
fn swieph_setup(args: &[String], mode: &str) -> String {
    let value_of = |flag: &str| {
        args.iter().position(|a| a == flag).map(|i| {
            args.get(i + 1)
                .unwrap_or_else(|| panic!("{flag} needs a value"))
                .clone()
        })
    };
    let out_dir = value_of("--out").unwrap_or_else(|| panic!("{mode} needs --out <dir>"));
    let ephe_dir = std::env::var("SE_EPHE_PATH")
        .ok()
        .or_else(|| value_of("--ephe"))
        .unwrap_or_else(|| DEFAULT_EPHE_DIR.to_string());
    if let Err(e) = pins::verify_swieph_files(&ephe_dir) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    let ephe = std::ffi::CString::new(ephe_dir).expect("ephe path has NUL");
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };
    out_dir
}

fn asteroids_main(args: &[String]) {
    let out_dir = swieph_setup(args, "--asteroids");
    let mut csv = String::new();
    csv.push_str("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with seas_18/sepl_18/semo_18 (SHA-256 pinned in tools/se-aspects-reference/src/pins.rs).\n");
    csv.push_str("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)\n");
    csv.push_str("# equals +angle or -angle, scanned on a 0.05-day grid (0.01 day with the Moon), each step split at the zero\n");
    csv.push_str("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.\n");
    csv.push_str("# geo: apparent, tropical, true equinox of date (default flags) with SEFLG_SWIEPH; Sun with Ceres, Pallas, Juno, Vesta, Mars-Vesta, Jupiter-Ceres JD 2415025.5-2488064.5, Moon-Ceres JD 2447892.5-2462502.5.\n");
    csv.push_str("# Angles 0, 60, 90, 120, 180.\n");
    csv.push_str("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.\n");
    csv.push_str("# No turning point of any pair's separation is within 30 arcsec of one of its levels (the tool fails otherwise).\n");
    csv.push_str(
        "group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day\n",
    );
    let mut rows = 0usize;
    {
        let mut push = |row: String| {
            rows += 1;
            writeln!(csv, "{row}").unwrap();
        };
        for (first, second, span, grid) in ASTEROID_PAIRS {
            scan(
                &mut push,
                "geo",
                first,
                second,
                GEO_SWIEPH,
                span,
                grid,
                Graze::Refuse(PLANET_GRAZE_MARGIN_DEG),
            );
        }
    }
    let manifest = format!(
        "slice aspects-asteroids file=asteroids.csv role=aspects rows={rows} checksum={}\n",
        pins::fnv1a64(&csv)
    );
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| panic!("create {out_dir}: {e}"));
    let csv_path = format!("{out_dir}/asteroids.csv");
    let manifest_path = format!("{out_dir}/asteroids-manifest.txt");
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&manifest_path, &manifest)
        .unwrap_or_else(|e| panic!("write {manifest_path}: {e}"));
    eprintln!("wrote {csv_path} ({rows} rows) and {manifest_path}");
}

fn lunar_points_main(args: &[String]) {
    let out_dir = swieph_setup(args, "--lunar-points");
    let mut csv = String::new();
    csv.push_str("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with sepl_18/semo_18 (SHA-256 pinned in tools/se-aspects-reference/src/pins.rs).\n");
    csv.push_str("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)\n");
    csv.push_str(
        "# equals +angle or -angle, scanned on a 0.005-day grid, each step split at the zero\n",
    );
    csv.push_str("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.\n");
    csv.push_str("# geo: apparent, tropical, true equinox of date (default flags) with SEFLG_SWIEPH; TrueNode (SE_TRUE_NODE) with Sun, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto; TrueApogee (SE_OSCU_APOG) with Sun, Jupiter, Saturn, Pluto; MeanNode (SE_MEAN_NODE) with the TrueNode planets; MeanApogee (SE_MEAN_APOG) with the TrueApogee planets; JD 2415025.5-2488064.5.\n");
    csv.push_str("# Angles 0, 60, 90, 120, 180.\n");
    csv.push_str("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.\n");
    csv.push_str("# A turning point within the pair's graze margin of a level (TrueApogee 12 arcsec, the other points 3 arcsec) is a graze window in lunar-points-grazes.csv; events inside a window may exist in one ephemeris only.\n");
    csv.push_str(
        "group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day\n",
    );
    let mut grazes = String::new();
    grazes.push_str("# Source: as lunar-points.csv (Swiss Ephemeris 2.10.03, SEFLG_SWIEPH, sepl_18/semo_18 SHA-256 pinned).\n");
    grazes.push_str("# One row per graze window of a lunar-point pair and angle: a turning point of the separation within the pair's\n");
    grazes.push_str("# margin of a level (TrueApogee 12 arcsec, TrueNode/MeanNode/MeanApogee 3 arcsec). lo_jd_tt and hi_jd_tt are the first\n");
    grazes.push_str("# 0.005-day grid instants on each side at which |separation - level| >= 2 x margin; overlapping windows are merged and\n");
    grazes.push_str("# turn_jd_tt/miss_arcsec are the closest turn. se_events counts lunar-points.csv rows of the pair and angle inside [lo, hi]\n");
    grazes.push_str("# (even when both edges lie on one side of the level, odd when they straddle it). Events inside a window may exist\n");
    grazes.push_str("# in one ephemeris only; the gate requires each side's count to have se_events' parity (issue #168 (c)).\n");
    grazes.push_str(
        "group,first,second,angle_deg,turn_jd_tt,miss_arcsec,lo_jd_tt,hi_jd_tt,se_events\n",
    );
    let mut rows = 0usize;
    let mut graze_rows = 0usize;
    {
        let mut push = |row: String| {
            rows += 1;
            writeln!(csv, "{row}").unwrap();
        };
        let mut push_graze = |row: String| {
            graze_rows += 1;
            writeln!(grazes, "{row}").unwrap();
        };
        for (first, second) in LUNAR_POINT_PAIRS {
            scan(
                &mut push,
                "geo",
                first,
                second,
                GEO_SWIEPH,
                FULL_SPAN,
                LUNAR_POINT_GRID_DAYS,
                Graze::Window {
                    margin_deg: lunar_point_margin_deg(first),
                    out: &mut push_graze,
                },
            );
        }
    }
    let manifest = format!(
        "slice aspects-lunar-points file=lunar-points.csv role=aspects rows={rows} checksum={}\n",
        pins::fnv1a64(&csv)
    );
    let grazes_manifest = format!(
        "slice aspects-lunar-points-grazes file=lunar-points-grazes.csv role=grazes rows={graze_rows} checksum={}\n",
        pins::fnv1a64(&grazes)
    );
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| panic!("create {out_dir}: {e}"));
    let csv_path = format!("{out_dir}/lunar-points.csv");
    let manifest_path = format!("{out_dir}/lunar-points-manifest.txt");
    let grazes_path = format!("{out_dir}/lunar-points-grazes.csv");
    let grazes_manifest_path = format!("{out_dir}/lunar-points-grazes-manifest.txt");
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&manifest_path, &manifest)
        .unwrap_or_else(|e| panic!("write {manifest_path}: {e}"));
    std::fs::write(&grazes_path, &grazes).unwrap_or_else(|e| panic!("write {grazes_path}: {e}"));
    std::fs::write(&grazes_manifest_path, &grazes_manifest)
        .unwrap_or_else(|e| panic!("write {grazes_manifest_path}: {e}"));
    eprintln!(
        "wrote {csv_path} ({rows} rows), {grazes_path} ({graze_rows} windows) and their manifests"
    );
}
