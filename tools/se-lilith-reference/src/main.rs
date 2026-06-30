//! Emits a Swiss Ephemeris reference corpus for the osculating true lunar
//! apogee (`SE_OSCU_APOG`, "True Black Moon Lilith") to STDOUT as CSV.
//!
//! Frame: true ecliptic of date, nutation on (SE default — no SEFLG_NONUT,
//! no SEFLG_J2000). Ephemeris: Moshier (SEFLG_MOSEPH) — no data files needed.
//! The perigee is the opposite apse and is not emitted (SE has no separate
//! perigee body); the gate checks the apogee against SE and the perigee via
//! internal symmetry.
//!
//! Usage: `cargo run --release > .../lilith-corpus/lilith.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SE_OSCU_APOG: c_int = 13;
const SEFLG_MOSEPH: c_int = 4;

// Deterministic sampling grid across the 1900–2100 packaged window. Step is
// coprime-ish with the ~206-day anomalistic period so successive samples land
// on different orbit phases.
const JD_START_TT: f64 = 2_415_020.5; // 1900-01-01
const JD_END_TT: f64 = 2_488_070.0; //   ~2100-01-01
const STEP_DAYS: f64 = 23.0;

fn se_true_apogee(jd_tt: f64) -> (f64, f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            SE_OSCU_APOG,
            SEFLG_MOSEPH,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc(SE_OSCU_APOG) failed at jd_tt={jd_tt}: {msg}");
    }
    let (lon, lat, dist) = (xx[0], xx[1], xx[2]);
    assert!(
        lon.is_finite() && lat.is_finite() && dist.is_finite(),
        "non-finite SE result at jd_tt={jd_tt}"
    );
    (lon.rem_euclid(360.0), lat, dist)
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc SE_OSCU_APOG=13,");
    println!("# iflag=SEFLG_MOSEPH (Moshier, no data files). Frame: true ecliptic of date, nutation on.");
    println!("# Columns: of-date true ecliptic longitude/latitude (deg) and geocentric distance (AU).");
    println!("# Accuracy note: Moshier Moon vs the DE440-sourced packaged Moon is part of the gate budget;");
    println!("# upgrade path is SEFLG_JPLEPH against DE440 if the band is too loose.");
    println!("jd_tt,se_oscu_apogee_lon_deg,se_oscu_apogee_lat_deg,se_oscu_apogee_dist_au");
    let mut jd = JD_START_TT;
    while jd <= JD_END_TT {
        let (lon, lat, dist) = se_true_apogee(jd);
        println!("{jd:.1},{lon:.9},{lat:.9},{dist:.12}");
        jd += STEP_DAYS;
    }
}
