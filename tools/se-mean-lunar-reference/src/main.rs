//! Emits a Swiss Ephemeris reference corpus for the mean lunar node
//! (`SE_MEAN_NODE`) and mean lunar apogee (`SE_MEAN_APOG`, "mean Lilith") to
//! STDOUT as CSV.
//!
//! Frame: true ecliptic of date, nutation on (SE default — no SEFLG_NONUT,
//! no SEFLG_J2000). Ephemeris flag: Moshier (SEFLG_MOSEPH) — no data files
//! needed; both bodies are analytic mean elements in Swiss Ephemeris.
//! Same deterministic grid as `tools/se-true-node-reference`.
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- cargo run --release --manifest-path tools/se-mean-lunar-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SE_MEAN_NODE: c_int = 10;
const SE_MEAN_APOG: c_int = 12;
const SEFLG_MOSEPH: c_int = 4;

const JD_START_TT: f64 = 2_415_020.5; // 1900-01-01
const JD_END_TT: f64 = 2_488_070.0; //   ~2100-01-01
const STEP_DAYS: f64 = 23.0;

fn se_point(jd_tt: f64, body: c_int, name: &str) -> (f64, f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            body,
            SEFLG_MOSEPH,
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
    let (lon, lat, dist) = (xx[0], xx[1], xx[2]);
    assert!(
        lon.is_finite() && lat.is_finite() && dist.is_finite(),
        "non-finite SE {name} result at jd_tt={jd_tt}"
    );
    (lon.rem_euclid(360.0), lat, dist)
}

fn main() {
    println!(
        "# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc SE_MEAN_NODE=10 and SE_MEAN_APOG=12,"
    );
    println!(
        "# iflag=SEFLG_MOSEPH (Moshier, no data files). Frame: true ecliptic of date, nutation on."
    );
    println!(
        "# Columns: of-date true ecliptic longitude/latitude (deg) and geocentric distance (AU), node then apogee."
    );
    println!("jd_tt,se_mean_node_lon_deg,se_mean_node_lat_deg,se_mean_node_dist_au,se_mean_apog_lon_deg,se_mean_apog_lat_deg,se_mean_apog_dist_au");
    let mut jd = JD_START_TT;
    while jd <= JD_END_TT {
        let (nl, nb, nd) = se_point(jd, SE_MEAN_NODE, "SE_MEAN_NODE");
        let (al, ab, ad) = se_point(jd, SE_MEAN_APOG, "SE_MEAN_APOG");
        println!("{jd:.1},{nl:.9},{nb:.9},{nd:.12},{al:.9},{ab:.9},{ad:.12}");
        jd += STEP_DAYS;
    }
}
