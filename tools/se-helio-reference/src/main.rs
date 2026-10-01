//! Emits a Swiss Ephemeris heliocentric reference corpus (Mercury–Pluto) to
//! STDOUT as CSV: position and speed from
//! `swe_calc(jd_et, body, SEFLG_MOSEPH | SEFLG_HELCTR | SEFLG_SPEED)`.
//!
//! Frame: true ecliptic and equinox of date, nutation on (SE default — no
//! SEFLG_NONUT, no SEFLG_J2000). Heliocentric places are geometric: SE applies
//! no light-time or aberration under SEFLG_HELCTR. Ephemeris: Moshier
//! (SEFLG_MOSEPH), no data files needed.
//!
//! Grid: every 23 days from 1900-01-01 across the packaged window, plus the
//! window's last instant, so the corpus has rows on both window edges.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
//!    --manifest-path tools/se-helio-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/helio-position-corpus/helio-position.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_HELCTR: c_int = 8;
const SEFLG_SPEED: c_int = 256;

/// (Swiss Ephemeris body id, name as written to the CSV).
const BODIES: [(c_int, &str); 8] = [
    (2, "Mercury"),
    (3, "Venus"),
    (4, "Mars"),
    (5, "Jupiter"),
    (6, "Saturn"),
    (7, "Uranus"),
    (8, "Neptune"),
    (9, "Pluto"),
];

const JD_START_TT: f64 = 2_415_020.5; // 1900-01-01, pleiades-events WINDOW_START_JD
const JD_END_TT: f64 = 2_488_069.5; //   pleiades-events WINDOW_END_JD
const STEP_DAYS: f64 = 23.0;

fn se_state(jd_tt: f64, body: c_int, name: &str) -> [f64; 6] {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            body,
            SEFLG_MOSEPH | SEFLG_HELCTR | SEFLG_SPEED,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc({name}) failed at jd_tt={jd_tt}: {msg}");
    }
    assert!(
        xx.iter().all(|v| v.is_finite()),
        "non-finite SE {name} result at jd_tt={jd_tt}"
    );
    xx[0] = xx[0].rem_euclid(360.0);
    xx
}

fn emit(jd: f64) {
    for (body, name) in BODIES {
        let [lon, lat, dist, lon_speed, lat_speed, dist_speed] = se_state(jd, body, name);
        println!(
            "{jd:.1},{name},{lon:.9},{lat:.9},{dist:.12},{lon_speed:.12},{lat_speed:.12},{dist_speed:.14}"
        );
    }
}

fn main() {
    println!(
        "# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc bodies 2..=9 (Mercury..Pluto),"
    );
    println!(
        "# iflag=SEFLG_MOSEPH|SEFLG_HELCTR|SEFLG_SPEED (Moshier, no data files). Frame: heliocentric,"
    );
    println!(
        "# true ecliptic and equinox of date, nutation on, geometric (no light-time, no aberration)."
    );
    println!(
        "# Columns: longitude/latitude (deg), distance (AU), then their speeds (deg/day, deg/day, AU/day)."
    );
    println!(
        "jd_tt,body,lon_deg,lat_deg,dist_au,lon_speed_deg_per_day,lat_speed_deg_per_day,dist_speed_au_per_day"
    );
    let mut jd = JD_START_TT;
    while jd < JD_END_TT {
        emit(jd);
        jd += STEP_DAYS;
    }
    emit(JD_END_TT);
}
