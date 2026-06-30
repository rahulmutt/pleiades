//! Emits a Swiss Ephemeris apparent equatorial-of-date (RA/Dec) reference corpus
//! for the major bodies to STDOUT as CSV.
//!
//! Frame: true equator/equinox of date, nutation + aberration on (SE default —
//! no SEFLG_NONUT/NOABERR/J2000), equatorial output (SEFLG_EQUATORIAL).
//! Ephemeris: Moshier (SEFLG_MOSEPH) — no data files needed.
//!
//! Usage: `cargo run --release > .../equatorial-se-corpus/equatorial-se.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_EQUATORIAL: c_int = 2048;
const IFLAG: c_int = SEFLG_MOSEPH | SEFLG_EQUATORIAL;

// (label, SE body number): Sun=0, Moon=1, Mercury=2 … Pluto=9.
const BODIES: &[(&str, c_int)] = &[
    ("Sun", 0), ("Moon", 1), ("Mercury", 2), ("Venus", 3), ("Mars", 4),
    ("Jupiter", 5), ("Saturn", 6), ("Uranus", 7), ("Neptune", 8), ("Pluto", 9),
];

const JD_START_TT: f64 = 2_415_025.5; // 1900-01-06 (inside coverage; see apparent gate)
const JD_END_TT: f64 = 2_488_065.5;   // 2099-12-26
const STEP_DAYS: f64 = 365.25 * 5.0;  // ~5-year cadence → ~40 epochs × 10 bodies

fn se_radec(jd_tt: f64, body: c_int) -> (f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(jd_tt, body, IFLAG, xx.as_mut_ptr(), serr.as_mut_ptr() as *mut c_char)
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy().into_owned();
        panic!("swe_calc(body={body}) failed at jd_tt={jd_tt}: {msg}");
    }
    let (ra, dec) = (xx[0], xx[1]);
    assert!(ra.is_finite() && dec.is_finite(), "non-finite SE RA/Dec at jd_tt={jd_tt}");
    (ra.rem_euclid(360.0), dec)
}

fn main() {
    println!("# Source: Swiss Ephemeris (libswisseph-sys 0.1.2), swe_calc, iflag=SEFLG_MOSEPH|SEFLG_EQUATORIAL.");
    println!("# Frame: apparent equatorial of date (nutation + aberration on). Columns: jd_tt,body,ra_deg,dec_deg.");
    println!("# Convention-parity reference (Moshier); ceilings are deliberately loose vs the Horizons accuracy gate.");
    println!("jd_tt,body,se_ra_deg,se_dec_deg");
    let mut jd = JD_START_TT;
    while jd <= JD_END_TT {
        for (label, num) in BODIES {
            let (ra, dec) = se_radec(jd, *num);
            println!("{jd:.1},{label},{ra:.9},{dec:.9}");
        }
        jd += STEP_DAYS;
    }
}
