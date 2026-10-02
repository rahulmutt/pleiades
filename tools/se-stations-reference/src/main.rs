//! Emits a Swiss Ephemeris planetary-station reference corpus to STDOUT as
//! CSV. Swiss Ephemeris has no station finder, so a station is located here
//! by scanning the longitude speed of `swe_calc(jd_tt, body, iflag | SEFLG_SPEED)`
//! on a fixed grid and bisecting each sign change to 1e-7 day (Swiss
//! Ephemeris' "ET" argument is TT).
//!
//! Groups:
//!   - `geo`:  geocentric apparent, tropical, true equinox of date (SE default
//!     flags). Mercury–Pluto over the pleiades-events window less five days at
//!     each end; the true node over 1990–2030.
//!   - `mean`: geocentric geometric place in the mean ecliptic and equinox of
//!     date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT).
//!     Mercury, Mars, Saturn over 1990–2030.
//!   - `sid`:  geocentric apparent, sidereal Lahiri (SEFLG_SIDEREAL after
//!     `swe_set_sid_mode(SE_SIDM_LAHIRI)`). Mercury, Mars, Saturn over 1990–2030.
//!
//! Grid: 0.25 day for the planets (their closest stations are 19 days
//! apart). 0.005 day for the true node, whose speed touches zero about every
//! two weeks and can cross it for only a few hours.
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH), no data files needed.
//!
//! Non-finite speeds: Swiss Ephemeris Moshier returns a NaN true-node speed at
//! isolated instants (e.g. jd_tt 2451544.9). Such a grid sample is skipped (and
//! reported on STDERR) and the bracket widens to the neighbouring finite
//! samples. Everything else non-finite aborts: a non-finite first sample, two
//! consecutive non-finite grid samples, a non-finite bisection midpoint, or a
//! non-finite longitude.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
//!    --manifest-path tools/se-stations-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/stations-corpus/stations.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_set_sid_mode};

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_SIDEREAL: c_int = 64 * 1024;

const SE_SIDM_LAHIRI: c_int = 1;
const SE_TRUE_NODE: c_int = 11;

const BASE: c_int = SEFLG_MOSEPH | SEFLG_SPEED;
const MEAN_OF_DATE: c_int = SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT;

/// (Swiss Ephemeris body id, name as written to the CSV).
const PLANETS: [(c_int, &str); 8] = [
    (2, "Mercury"),
    (3, "Venus"),
    (4, "Mars"),
    (5, "Jupiter"),
    (6, "Saturn"),
    (7, "Uranus"),
    (8, "Neptune"),
    (9, "Pluto"),
];
const SUBSET: [(c_int, &str); 3] = [(2, "Mercury"), (4, "Mars"), (6, "Saturn")];

/// The pleiades-events window (JD 2415020.5–2488069.5) less five days at each
/// end, so neither side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

const PLANET_GRID_DAYS: f64 = 0.25;
const TRUE_NODE_GRID_DAYS: f64 = 0.005;
const BISECT_TOLERANCE_DAYS: f64 = 1e-7;

/// `(longitude_deg in [0, 360), longitude_speed_deg_per_day)`. The speed is
/// returned as is and may be NaN; callers decide how to treat that.
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
        xx[0].is_finite(),
        "non-finite SE longitude for ipl={ipl} at jd_tt={jd_tt}"
    );
    (xx[0].rem_euclid(360.0), xx[3])
}

/// Prints one row per sign change of the longitude speed in `[lo, hi]`.
fn scan(group: &str, name: &str, ipl: c_int, iflag: c_int, (lo, hi): (f64, f64), grid: f64) {
    let speed = |jd: f64| state(jd, ipl, iflag).1;
    let steps = ((hi - lo) / grid).floor() as u64;
    let mut prev_jd = lo;
    let mut prev = speed(lo);
    assert!(
        prev.is_finite(),
        "non-finite speed at scan start {group},{name} jd_tt={lo}"
    );
    let mut skipped_last = false;
    for k in 1..=steps {
        let jd = lo + k as f64 * grid;
        let cur = speed(jd);
        if !cur.is_finite() {
            assert!(
                !skipped_last,
                "two consecutive non-finite speeds at {group},{name} jd_tt={jd}"
            );
            eprintln!("skipped non-finite speed sample: {group},{name},{jd:.7}");
            skipped_last = true;
            continue;
        }
        skipped_last = false;
        if (prev <= 0.0) != (cur <= 0.0) {
            let (mut a, mut b, mut f_a) = (prev_jd, jd, prev);
            while b - a > BISECT_TOLERANCE_DAYS {
                let mid = 0.5 * (a + b);
                let f_mid = speed(mid);
                assert!(
                    f_mid.is_finite(),
                    "non-finite speed at bisection midpoint {group},{name} jd_tt={mid}"
                );
                if (f_a <= 0.0) == (f_mid <= 0.0) {
                    a = mid;
                    f_a = f_mid;
                } else {
                    b = mid;
                }
            }
            let root = 0.5 * (a + b);
            let (lon, _) = state(root, ipl, iflag);
            let kind = if cur > 0.0 { "D" } else { "R" };
            println!("{group},{name},{root:.7},{lon:.9},{kind}");
        }
        prev_jd = jd;
        prev = cur;
    }
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), Moshier (SEFLG_MOSEPH, no data files).");
    println!("# A row is a sign change of the longitude speed of swe_calc(jd_tt, body, iflag|SEFLG_SPEED),");
    println!("# bracketed on a 0.25-day grid (0.005 day for TrueNode) and bisected to 1e-7 day. jd_tt is TT.");
    println!("# geo: apparent, tropical, true equinox of date (default flags); planets JD 2415025.5-2488064.5,");
    println!("#   TrueNode JD 2447892.5-2462502.5. mean: SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT,");
    println!(
        "#   JD 2447892.5-2462502.5. sid: SEFLG_SIDEREAL, SE_SIDM_LAHIRI, JD 2447892.5-2462502.5."
    );
    println!(
        "# kind: R = turns retrograde, D = turns direct. lon_deg is the longitude at the station."
    );
    println!("# Swiss Ephemeris returns a NaN true-node speed at isolated instants (e.g. jd_tt 2451544.9); such a grid sample is skipped and the bracket widened.");
    println!("group,body,jd_tt,lon_deg,kind");
    for (ipl, name) in PLANETS {
        scan("geo", name, ipl, BASE, FULL_SPAN, PLANET_GRID_DAYS);
    }
    scan(
        "geo",
        "TrueNode",
        SE_TRUE_NODE,
        BASE,
        SHORT_SPAN,
        TRUE_NODE_GRID_DAYS,
    );
    for (ipl, name) in SUBSET {
        scan(
            "mean",
            name,
            ipl,
            BASE | MEAN_OF_DATE,
            SHORT_SPAN,
            PLANET_GRID_DAYS,
        );
    }
    unsafe { swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0) };
    for (ipl, name) in SUBSET {
        scan(
            "sid",
            name,
            ipl,
            BASE | SEFLG_SIDEREAL,
            SHORT_SPAN,
            PLANET_GRID_DAYS,
        );
    }
}
