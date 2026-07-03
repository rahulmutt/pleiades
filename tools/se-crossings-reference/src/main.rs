//! Emits a Swiss Ephemeris crossing reference corpus to STDOUT as CSV.
//!
//! Frame `geo`: geocentric apparent tropical longitude of date (SE default).
//!   - Sun / Moon crossings use SE's dedicated `swe_solcross_ut` /
//!     `swe_mooncross_ut` (UT in/out), converted to TDB via `swe_deltat`.
//!   - Mars crossings: SE exposes NO geocentric planet-crossing function
//!     (only Sun and Moon have `swe_*cross`; general bodies only have the
//!     *heliocentric* `swe_helio_cross`). So the geocentric Mars crossings are
//!     located by a deterministic bisection on SE's own `swe_calc` geocentric
//!     longitudes (real SE positions, no hand-entered values). This is required
//!     to exercise the retrograde triple-crossing, which is a geocentric
//!     phenomenon only.
//! Frame `helio`: heliocentric (SEFLG_HELCTR) via `swe_helio_cross` (ET/TDB).
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH) so no data files are needed.
//! All start and crossing JDs are TDB and fall inside the 1900-2100 window.
//! Every row is a FORWARD "next crossing after start" (direction=fwd).
//!
//! Usage: `cargo run --release > .../crossings-corpus/crossings.csv`
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_deltat, swe_helio_cross, swe_mooncross_ut, swe_solcross_ut};

const SEFLG_MOSEPH: c_int = 4; // Moshier ephemeris, no data files
const SEFLG_HELCTR: c_int = 8; // heliocentric position

// SE ipl body numbers.
const SE_SUN: c_int = 0;
const SE_MOON: c_int = 1;
const SE_MARS: c_int = 4;
const SE_JUPITER: c_int = 5;
const SE_SATURN: c_int = 6;

// 1900-2100 CE packaged window (TDB Julian days).
const JD_WINDOW_LO: f64 = 2_415_020.5; // 1900-01-01
const JD_WINDOW_HI: f64 = 2_488_069.5; // 2100-01-01

fn serr_string(serr: &[c_char]) -> String {
    unsafe { CStr::from_ptr(serr.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// Geocentric apparent tropical longitude of `ipl` (degrees, [0,360)) at TDB.
fn geo_longitude(jd_tdb: f64, ipl: c_int) -> f64 {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tdb,
            ipl,
            SEFLG_MOSEPH,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_calc(ipl={ipl}) failed at jd_tdb={jd_tdb}: {}",
            serr_string(&serr)
        );
    }
    assert!(xx[0].is_finite(), "non-finite longitude at jd_tdb={jd_tdb}");
    xx[0].rem_euclid(360.0)
}

/// Geocentric Sun crossing of `target_deg`, next after `start_tdb`.
/// Uses SE `swe_solcross_ut` (UT) with a swe_deltat TDB<->UT conversion.
fn geo_sun_cross_tdb(target_deg: f64, start_tdb: f64) -> f64 {
    let jd_ut = start_tdb - unsafe { swe_deltat(start_tdb) };
    let mut serr = [0_i8; 256];
    let crossing_ut = unsafe {
        swe_solcross_ut(target_deg, jd_ut, SEFLG_MOSEPH, serr.as_mut_ptr() as *mut c_char)
    };
    if crossing_ut < jd_ut {
        panic!(
            "swe_solcross_ut(target={target_deg}) failed at jd_ut={jd_ut}: {}",
            serr_string(&serr)
        );
    }
    crossing_ut + unsafe { swe_deltat(crossing_ut) }
}

/// Geocentric Moon crossing of `target_deg`, next after `start_tdb`.
fn geo_moon_cross_tdb(target_deg: f64, start_tdb: f64) -> f64 {
    let jd_ut = start_tdb - unsafe { swe_deltat(start_tdb) };
    let mut serr = [0_i8; 256];
    let crossing_ut = unsafe {
        swe_mooncross_ut(target_deg, jd_ut, SEFLG_MOSEPH, serr.as_mut_ptr() as *mut c_char)
    };
    if crossing_ut < jd_ut {
        panic!(
            "swe_mooncross_ut(target={target_deg}) failed at jd_ut={jd_ut}: {}",
            serr_string(&serr)
        );
    }
    crossing_ut + unsafe { swe_deltat(crossing_ut) }
}

/// Heliocentric crossing of `ipl` over `target_deg`, next after `start_tdb`.
/// `swe_helio_cross` takes/returns ET(=TDB) directly via the out-param.
fn helio_cross_tdb(ipl: c_int, target_deg: f64, start_tdb: f64) -> f64 {
    let mut jd_cross = 0.0_f64;
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_helio_cross(
            ipl,
            target_deg,
            start_tdb,
            SEFLG_MOSEPH | SEFLG_HELCTR,
            1, // dir = +1 (forward)
            &mut jd_cross,
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_helio_cross(ipl={ipl}, target={target_deg}) failed at jd_et={start_tdb}: {}",
            serr_string(&serr)
        );
    }
    jd_cross
}

/// Signed shortest angular distance lon-target, in (-180, 180].
fn signed_delta(lon: f64, target: f64) -> f64 {
    ((lon - target + 180.0).rem_euclid(360.0)) - 180.0
}

/// Next geocentric crossing of `ipl` over `target_deg` strictly after
/// `start_tdb`, by deterministic scan + bisection on SE `swe_calc` longitudes.
/// Used for Mars (no SE geocentric planet-crossing function exists).
fn geo_planet_cross_tdb(ipl: c_int, target_deg: f64, start_tdb: f64) -> f64 {
    const STEP: f64 = 0.25; // days; << time to move 180deg, so no wrap aliasing
    let mut a = start_tdb;
    let mut fa = signed_delta(geo_longitude(a, ipl), target_deg);
    let mut jd = start_tdb + STEP;
    while jd <= JD_WINDOW_HI {
        let fb = signed_delta(geo_longitude(jd, ipl), target_deg);
        if fa == 0.0 || (fa < 0.0) != (fb < 0.0) {
            // bracket [a, jd]: bisect
            let (mut lo, mut hi) = (a, jd);
            for _ in 0..64 {
                let mid = 0.5 * (lo + hi);
                let fm = signed_delta(geo_longitude(mid, ipl), target_deg);
                if (fa < 0.0) != (fm < 0.0) {
                    hi = mid;
                } else {
                    lo = mid;
                    fa = fm;
                }
            }
            return 0.5 * (lo + hi);
        }
        a = jd;
        fa = fb;
        jd += STEP;
    }
    panic!("no geocentric crossing of {target_deg} for ipl={ipl} after {start_tdb} within window");
}

fn in_window(jd: f64) -> bool {
    (JD_WINDOW_LO..=JD_WINDOW_HI).contains(&jd)
}

fn emit(frame: &str, body: &str, target: f64, start_tdb: f64, crossing_tdb: f64) {
    assert!(
        in_window(start_tdb),
        "start out of window: {body} {target} start={start_tdb}"
    );
    assert!(
        in_window(crossing_tdb),
        "crossing out of window: {body} {target} crossing={crossing_tdb}"
    );
    assert!(
        crossing_tdb > start_tdb,
        "non-forward crossing: {body} {target} start={start_tdb} crossing={crossing_tdb}"
    );
    // direction is fwd for every row: the downstream gate does forward-only
    // "next crossing after start" and ignores the direction column.
    println!("{frame},{body},{target:.6},{start_tdb:.6},fwd,{crossing_tdb:.9}");
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2).");
    println!("# geo Sun/Moon: swe_solcross_ut / swe_mooncross_ut (UT), TDB via swe_deltat.");
    println!("# geo Mars: bisection on swe_calc geocentric longitude (no SE geo planet-cross fn).");
    println!("# helio: swe_helio_cross (ET/TDB), iflag=SEFLG_MOSEPH|SEFLG_HELCTR, dir=+1.");
    println!("# All rows forward (next crossing after start); times TDB within 1900-2100.");
    println!("frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb");

    // --- geo Sun: cardinal points + one arbitrary longitude, starts spread. ---
    let sun_targets = [0.0_f64, 90.0, 180.0, 270.0, 137.5];
    let sun_starts = [2_416_000.5_f64, 2_440_000.5, 2_470_000.5]; // ~1902 / ~1968 / ~2050
    for &start in &sun_starts {
        for &t in &sun_targets {
            let c = geo_sun_cross_tdb(t, start);
            emit("geo", "Sun", t, start, c);
        }
    }

    // --- geo Moon: cardinal points + one arbitrary longitude, starts spread. ---
    let moon_targets = [0.0_f64, 90.0, 180.0, 270.0, 45.0];
    let moon_starts = [2_420_000.5_f64, 2_450_000.5, 2_480_000.5]; // ~1913 / ~1995 / ~2077
    for &start in &moon_starts {
        for &t in &moon_targets {
            let c = geo_moon_cross_tdb(t, start);
            emit("geo", "Moon", t, start, c);
        }
    }

    // --- geo Mars retrograde triple-crossing (2003 opposition loop). ---
    // Target 337.0 deg is crossed three times: direct, retrograde, direct.
    // Each start sits just after the previous crossing so forward
    // "next crossing after start" yields crossings 1, 2, then 3 in turn.
    let mars_target = 337.0_f64;
    let mars_starts = [
        2_452_791.5_f64, // 2003-06-01, before crossing 1
        2_452_832.0,     // ~2003-07-12, after crossing 1, before crossing 2
        2_452_878.0,     // ~2003-08-27, after crossing 2, before crossing 3
    ];
    let mut mars_prev = f64::NEG_INFINITY;
    for &start in &mars_starts {
        let c = geo_planet_cross_tdb(SE_MARS, mars_target, start);
        assert!(
            c > mars_prev + 1.0,
            "Mars crossings not distinct: {c} vs prev {mars_prev}"
        );
        mars_prev = c;
        emit("geo", "Mars", mars_target, start, c);
    }

    // --- helio Jupiter & Saturn crossing 0 deg (and 180 deg). ---
    let helio_targets = [0.0_f64, 180.0];
    let helio_starts = [2_430_000.5_f64, 2_460_000.5]; // ~1941 / ~2023
    for &(ipl, name) in &[(SE_JUPITER, "Jupiter"), (SE_SATURN, "Saturn")] {
        for &start in &helio_starts {
            for &t in &helio_targets {
                let c = helio_cross_tdb(ipl, t, start);
                emit("helio", name, t, start, c);
            }
        }
    }

    let _ = SE_SUN;
    let _ = SE_MOON;
}
