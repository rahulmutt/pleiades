//! Emits a Swiss Ephemeris geometric sidereal position corpus to STDOUT as
//! CSV: longitude, latitude and longitude speed of the Sun, the Moon and
//! Mercury–Pluto from
//! `swe_calc(jd_tt, body, SEFLG_MOSEPH | SEFLG_SIDEREAL | SEFLG_TRUEPOS |
//! SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_SPEED)` after `swe_set_sid_mode`
//! (Swiss Ephemeris' "ET" argument is TT).
//!
//! Place: the geometric geocentric place (no light-time, no aberration, no
//! deflection) on the mean ecliptic and equinox of date, less the ayanamsa.
//! Swiss Ephemeris drops nutation from a sidereal position. This is the place
//! a `pleiades-core` mean chart reports in a sidereal zodiac (issue #164).
//!
//! The geometric flags also keep a star-anchored ayanamsa (True Citra,
//! Galactic Center) the mean one: under apparent flags Swiss Ephemeris takes
//! the anchoring star's apparent place and folds its annual aberration, up to
//! about 20″, into the ayanamsa. `check_decomposition` asserts on every row
//! that the sidereal longitude is the mean-equinox longitude minus the mean
//! ayanamsa.
//!
//! After those rows come apparent rows (issue #164 (c)), marked by a 7th
//! column `apparent`: the Sun, the Moon and Mars under True Citra and
//! Galactic Center from `swe_calc(jd_tt, body, SEFLG_MOSEPH | SEFLG_SIDEREAL
//! | SEFLG_SPEED)`, Swiss Ephemeris' default, whose star-anchored ayanamsa
//! is read from the anchor star's apparent place.
//!
//! Ayanamsas: one per `pleiades-ayanamsa` computation class, the four the
//! crossings corpus uses. Grid: every 1087 days from 1901-01-01, 67 epochs
//! across the packaged 1900–2100 window; the step shares no period with the
//! year or the month. Ephemeris: Moshier (SEFLG_MOSEPH), no data files.
//!
//! Two build/run caveats: under devenv's gcc the build needs
//! `CFLAGS=-std=gnu17` (libswisseph-sys otherwise fails with a conflicting
//! `getenv` declaration), and `devenv shell` prints a banner line to stdout,
//! so build inside devenv and run the built binary directly:
//!
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release \
//!    --manifest-path tools/se-sidereal-position-reference/Cargo.toml`
//! `tools/se-sidereal-position-reference/target/release/se-sidereal-position-reference \
//!    > crates/pleiades-validate/data/sidereal-position-corpus/sidereal-position.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_get_ayanamsa_ex, swe_set_sid_mode};

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_SIDEREAL: c_int = 64 * 1024;

/// Geometric place: no light-time, aberration or deflection.
const GEOMETRIC: c_int = SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL;

/// (Swiss Ephemeris body id, name as written to the CSV).
const BODIES: [(c_int, &str); 10] = [
    (0, "Sun"),
    (1, "Moon"),
    (2, "Mercury"),
    (3, "Venus"),
    (4, "Mars"),
    (5, "Jupiter"),
    (6, "Saturn"),
    (7, "Uranus"),
    (8, "Neptune"),
    (9, "Pluto"),
];

/// (name as written to the CSV, SE_SIDM id): OffsetDefined, TrueStar,
/// Galactic, FittedOffset.
const AYANAMSAS: [(&str, c_int); 4] = [
    ("Lahiri", 1),
    ("TrueCitra", 27),
    ("GalacticCenter", 17),
    ("DeLuce", 2),
];

const JD_FIRST_TT: f64 = 2_415_385.5; // 1901-01-01
const JD_END_TT: f64 = 2_488_069.5; //   pleiades-events WINDOW_END_JD
const STEP_DAYS: f64 = 1087.0;

fn serr_string(serr: &[c_char]) -> String {
    unsafe { CStr::from_ptr(serr.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn se_state(jd_tt: f64, body: c_int, name: &str, iflag: c_int) -> [f64; 6] {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            body,
            iflag,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_calc({name}, iflag={iflag}) failed at jd_tt={jd_tt}: {}",
            serr_string(&serr)
        );
    }
    assert!(
        xx.iter().all(|v| v.is_finite()),
        "non-finite SE {name} result at jd_tt={jd_tt}"
    );
    xx[0] = xx[0].rem_euclid(360.0);
    xx
}

/// Mean ayanamsa (degrees) of the sidereal mode last set with
/// `swe_set_sid_mode`.
fn mean_ayanamsa(jd_tt: f64) -> f64 {
    let mut daya = 0.0_f64;
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_get_ayanamsa_ex(
            jd_tt,
            SEFLG_MOSEPH | SEFLG_NONUT | GEOMETRIC,
            &mut daya,
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_get_ayanamsa_ex failed at jd_tt={jd_tt}: {}",
            serr_string(&serr)
        );
    }
    daya
}

/// Asserts that the sidereal longitude about to be written is the geometric
/// mean-equinox longitude minus the mean ayanamsa.
fn check_decomposition(jd_tt: f64, body: c_int, name: &str, ayanamsa: &str, sidereal_lon: f64) {
    let tropical = se_state(jd_tt, body, name, SEFLG_MOSEPH | GEOMETRIC | SEFLG_NONUT)[0];
    let decomposed = (tropical - mean_ayanamsa(jd_tt)).rem_euclid(360.0);
    let diff = ((sidereal_lon - decomposed + 180.0).rem_euclid(360.0) - 180.0).abs();
    assert!(
        diff < 1e-6,
        "{ayanamsa} {name} at jd_tt={jd_tt}: sidereal {sidereal_lon} vs decomposed {decomposed} differ by {diff} deg"
    );
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc bodies 0..=9 (Sun..Pluto),");
    println!("# iflag=SEFLG_MOSEPH|SEFLG_SIDEREAL|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_SPEED after swe_set_sid_mode.");
    println!("# Place: geometric geocentric, mean ecliptic and equinox of date, less the mean ayanamsa (nutation-free).");
    println!("# Each row is checked against the geometric mean-equinox longitude minus swe_get_ayanamsa_ex. jd_tt is TT.");
    println!("# Columns: sidereal longitude and latitude (deg), longitude speed (deg/day).");
    println!("jd_tt,ayanamsa,body,lon_deg,lat_deg,lon_speed_deg_per_day");
    let place = SEFLG_MOSEPH | SEFLG_SIDEREAL | GEOMETRIC | SEFLG_SPEED;
    for (ayanamsa, sid_mode) in AYANAMSAS {
        unsafe { swe_set_sid_mode(sid_mode, 0.0, 0.0) };
        let mut jd = JD_FIRST_TT;
        while jd < JD_END_TT {
            for (body, name) in BODIES {
                let state = se_state(jd, body, name, place);
                check_decomposition(jd, body, name, ayanamsa, state[0]);
                println!(
                    "{jd:.1},{ayanamsa},{name},{:.9},{:.9},{:.12}",
                    state[0], state[1], state[3]
                );
            }
            jd += STEP_DAYS;
        }
    }
    // Apparent rows (issue #164 (c)): Swiss Ephemeris's default SEFLG_SIDEREAL,
    // whose ayanamsa is read from the anchor star's apparent place. The 7th
    // column marks them.
    println!("# Apparent rows: iflag=SEFLG_MOSEPH|SEFLG_SIDEREAL|SEFLG_SPEED (apparent place, nutation-free; star-anchored ayanamsa from the anchor star's apparent place). 7th column = apparent.");
    let apparent = SEFLG_MOSEPH | SEFLG_SIDEREAL | SEFLG_SPEED;
    for (ayanamsa, sid_mode) in [("TrueCitra", 27), ("GalacticCenter", 17)] {
        unsafe { swe_set_sid_mode(sid_mode, 0.0, 0.0) };
        let mut jd = JD_FIRST_TT;
        while jd < JD_END_TT {
            for (body, name) in [(0, "Sun"), (1, "Moon"), (4, "Mars")] {
                let state = se_state(jd, body, name, apparent);
                println!(
                    "{jd:.1},{ayanamsa},{name},{:.9},{:.9},{:.12},apparent",
                    state[0], state[1], state[3]
                );
            }
            jd += STEP_DAYS;
        }
    }
}
