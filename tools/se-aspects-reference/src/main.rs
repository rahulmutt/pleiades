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
//! instant, and no corpus is written. Counts per pair and angle go to STDERR.
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH), no data files needed.
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

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

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

/// (Swiss Ephemeris body id, name as written to the CSV).
type Body = (c_int, &'static str);
const SUN: Body = (0, "Sun");
const MOON: Body = (1, "Moon");
const MERCURY: Body = (2, "Mercury");
const VENUS: Body = (3, "Venus");
const MARS: Body = (4, "Mars");
const JUPITER: Body = (5, "Jupiter");
const SATURN: Body = (6, "Saturn");
const PLUTO: Body = (9, "Pluto");

/// The pleiades-events window (JD 2415020.5–2488069.5) less five days at each
/// end, so neither side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

const GRID_DAYS: f64 = 0.05;
const MOON_GRID_DAYS: f64 = 0.01;
const BISECT_TOLERANCE_DAYS: f64 = 1e-7;
/// 30 arcseconds: several times the largest difference between the Swiss
/// Ephemeris (Moshier) and pleiades separations.
const GRAZE_MARGIN_DEG: f64 = 30.0 / 3600.0;
const ANGLES: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

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

/// Prints every exact aspect of the pair in `[lo, hi]`, for each angle.
fn scan(group: &str, first: Body, second: Body, iflag: c_int, (lo, hi): (f64, f64), grid: f64) {
    let at = |jd: f64| sample(jd, first, second, iflag);
    let mut events: Vec<Vec<Sample>> = vec![Vec::new(); ANGLES.len()];
    let steps = ((hi - lo) / grid).floor() as u64;
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
            for angle in ANGLES {
                for level in levels(angle) {
                    let miss = wrap180(turn.separation - level).abs();
                    assert!(
                        miss >= GRAZE_MARGIN_DEG,
                        "graze: {group},{},{} turns {:.2} arcsec from the {angle} degree level \
                         at jd_tt={:.5}; an event here would depend on the ephemeris",
                        first.1,
                        second.1,
                        miss * 3600.0,
                        turn.jd
                    );
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
    for (index, angle) in ANGLES.iter().enumerate() {
        events[index].sort_by(|x, y| x.jd.total_cmp(&y.jd));
        eprintln!(
            "{group},{},{},{angle:.0}: {} events",
            first.1,
            second.1,
            events[index].len()
        );
        for event in &events[index] {
            println!(
                "{group},{},{},{angle:.0},{:.7},{:.9},{:.9},{:.9}",
                first.1, second.1, event.jd, event.first_lon, event.second_lon, event.rel_speed
            );
        }
    }
}

fn main() {
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
    scan("geo", SUN, MOON, GEO, SHORT_SPAN, MOON_GRID_DAYS);
    for (first, second) in [
        (SUN, MERCURY),
        (MERCURY, VENUS),
        (VENUS, MARS),
        (MARS, JUPITER),
        (MARS, SATURN),
        (JUPITER, SATURN),
        (SATURN, PLUTO),
    ] {
        scan("geo", first, second, GEO, FULL_SPAN, GRID_DAYS);
    }
    for (first, second) in [(MERCURY, VENUS), (MARS, SATURN)] {
        scan("mean", first, second, MEAN, SHORT_SPAN, GRID_DAYS);
    }
    scan("helio", MARS, JUPITER, HELIO, FULL_SPAN, GRID_DAYS);
}
