//! Emits a Swiss Ephemeris **local (per-observer) eclipse circumstances**
//! reference corpus for the pure-Rust `pleiades-eclipse` local layer.
//!
//! Two CSVs plus a manifest:
//!   * `sol-local.csv` — solar-eclipse local circumstances. For each curated
//!     `(eclipse seed, observer)` case: the topocentric contact instants
//!     C1..C4 + local maximum (`swe_sol_eclipse_when_loc`), the local
//!     magnitude / obscuration (`attr[]`), and the Sun's azimuth / altitude at
//!     the local maximum (`swe_azalt`). Solar contacts are genuinely
//!     observer-dependent (topocentric), so they come from `when_loc`.
//!   * `lun-local.csv` — lunar-eclipse local circumstances. Lunar shadow
//!     contacts P1/U1/U2/U3/U4/P4 are **global** instants (the Moon enters
//!     Earth's shadow at one time for everyone), taken from
//!     `swe_lun_eclipse_when`; the local layer adds the umbral / penumbral
//!     magnitude and the Moon's azimuth / altitude + horizon visibility at the
//!     maximum (`swe_lun_eclipse_how` + `swe_azalt`).
//!   * `manifest.txt` — per-CSV row counts + `fnv1a64` checksum of the CSV
//!     bytes + the SE version string + the conventions above.
//!
//! Anchoring & visibility. `swe_sol_eclipse_when_loc` / `swe_lun_eclipse_when_loc`
//! *auto-advance past eclipses that are not locally visible*, which would make
//! it impossible to record a below-horizon / Moon-down observer against the
//! intended eclipse. So each case is first anchored to a specific eclipse via a
//! GLOBAL search (`swe_sol_eclipse_when_glob` / `swe_lun_eclipse_when`) seeded a
//! few days before the event:
//!   * Solar: if `when_loc` lands on the same eclipse (local max within 2 days of
//!     the global max) the eclipse is locally visible → topocentric C1..C4 +
//!     `attr` from `when_loc`, `se_any_visible=1`. Otherwise the Sun is
//!     down / no eclipse there → magnitude + az/alt from `swe_sol_eclipse_how`
//!     at the global max, contact fields left empty, `se_any_visible=0`.
//!   * Lunar: global contacts from `swe_lun_eclipse_when` (identical for every
//!     observer — matching the engine's geocentric contact model), then
//!     `swe_lun_eclipse_how` at the global max gives per-observer umbral /
//!     penumbral magnitude and the Moon's apparent altitude;
//!     `se_any_visible = (apparent alt > 0)`.
//!
//! Azimuth convention: `swe_azalt` returns azimuth measured from SOUTH,
//! increasing WESTWARD, in [0, 360) (SE native; `xaz[0] = 360 - x[0]` after the
//! source's "azimuth from south to west" step). Our engine matches it. Altitude
//! columns are true altitude and apparent (refracted) altitude in degrees. The
//! az/alt are computed at the ENGINE-CONSUMED numeric Julian Day, i.e. the TT
//! instant J: the body's apparent position at TT=J (`swe_calc`, ET argument) and
//! the Earth rotation with that SAME numeric J fed to `swe_azalt` as its time
//! argument. This mirrors the engine's `local::body_horizontal`, which samples
//! Sun/Moon at J (as ephemeris/TDB time) yet computes sidereal time treating J
//! verbatim as UT1 — so both sides use the identical numeric J for rotation, and
//! the comparison is a fair MODEL parity, not a ΔT rotation offset.
//!
//! Times: all instants are TT (Terrestrial Time) Julian days, matching the
//! engine's DYNAMICAL eclipse instants and the sibling global corpus's
//! `greatest_eclipse_jd_tt` column. The SE eclipse functions return UT; each
//! instant is converted to TT via `TT = jd_ut + swe_deltat(jd_ut)` before it is
//! emitted (columns are named `_jd_tt`). This differs from
//! `tools/se-rise-trans-reference` (which stays in UT) because rise/set/transit
//! are sidereal/UT1-defined events, whereas an eclipse contact/max instant is
//! dynamical (TT/TDB) — the gate therefore compares the engine's native-TDB
//! instants DIRECTLY against these `_jd_tt` columns, with no ΔT crossing.
//! Every eclipse max is asserted inside the 1900–2100 window.
//!
//! Ephemeris: Moshier (`SEFLG_MOSEPH`) — no `.se1` files needed.
//!
//! Usage:
//!   se-eclipse-local-reference --dry-run          # print CSVs+manifest, no writes
//!   se-eclipse-local-reference --out <dir>        # write the three files
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::env;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{
    swe_azalt, swe_calc_ut, swe_deltat, swe_julday, swe_lun_eclipse_how, swe_lun_eclipse_when,
    swe_set_ephe_path, swe_set_topo, swe_sol_eclipse_how, swe_sol_eclipse_when_glob,
    swe_sol_eclipse_when_loc, swe_version,
};

const SEFLG_MOSEPH: c_int = 4; // Moshier ephemeris, no data files
const SEFLG_TOPOCTR: c_int = 32768; // topocentric position (applies diurnal parallax)
const SE_GREG_CAL: c_int = 1;
const SE_SUN: c_int = 0;
const SE_MOON: c_int = 1;

// swe_azalt coordinate mode: input is ecliptic-of-date (lon, lat, dist).
const SE_ECL2HOR: c_int = 0;

// Eclipse type bits (swephexp.h). Masks pick the classification out of the
// return flag (which also carries SE_ECL_VISIBLE / *_VISIBLE bits).
const SE_ECL_TOTAL: c_int = 4;
const SE_ECL_ANNULAR: c_int = 8;
const SE_ECL_PARTIAL: c_int = 16;
const SE_ECL_ANNULAR_TOTAL: c_int = 32;
const SE_ECL_PENUMBRAL: c_int = 64;
const SE_ECL_ALLTYPES_SOLAR: c_int =
    1 | 2 | SE_ECL_TOTAL | SE_ECL_ANNULAR | SE_ECL_PARTIAL | SE_ECL_ANNULAR_TOTAL;
const SE_ECL_ALLTYPES_LUNAR: c_int = SE_ECL_TOTAL | SE_ECL_PARTIAL | SE_ECL_PENUMBRAL;

// Any type, forward search.
const IFLTYPE_ANY: c_int = 0;
const SEARCH_FORWARD: c_int = 0;

// Standard atmosphere (hPa/mbar, °C).
const STD_PRESS: f64 = 1013.25;
const STD_TEMP: f64 = 15.0;

// If when_loc's local max is within this many days of the global max, it is the
// same eclipse (consecutive eclipses are weeks–months apart; a sunrise/sunset-
// trimmed local max stays within a day of the global max).
const SAME_ECLIPSE_TOL_DAYS: f64 = 2.0;

// 1900-2100 CE window (Julian days); every eclipse max lands inside.
const JD_WINDOW_LO: f64 = 2_415_020.5; // 1900-01-01
const JD_WINDOW_HI: f64 = 2_488_069.5; // 2100-01-01

/// FNV-1a 64-bit over the CSV bytes — identical to the sibling harnesses'
/// `fnv1a64`, so the manifest checksum matches the repo's scheme byte-for-byte.
fn fnv1a64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn serr_string(serr: &[c_char]) -> String {
    unsafe { CStr::from_ptr(serr.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn se_version() -> String {
    let mut buf = [0 as c_char; 256];
    unsafe {
        swe_version(buf.as_mut_ptr());
        CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned()
    }
}

/// Calendar (UT midnight) → Julian day, via SE, so seeds read as dates.
fn jd_ut(year: c_int, month: c_int, day: c_int) -> f64 {
    unsafe { swe_julday(year, month, day, 0.0, SE_GREG_CAL) }
}

/// Convert an SE UT Julian Day to TT: `TT = jd_ut + swe_deltat(jd_ut)`
/// (`swe_deltat` returns ΔT in days). This is the engine's dynamical time base
/// for eclipse instants (matching the global corpus's `greatest_eclipse_jd_tt`).
fn ut_to_tt(jd_ut: f64) -> f64 {
    jd_ut + unsafe { swe_deltat(jd_ut) }
}

/// UT contact → TT, preserving SE's `0.0` "absent contact" sentinel (so an
/// absent phase stays absent, i.e. an empty CSV cell after `fmt_contact`).
fn contact_tt(jd_ut: f64) -> f64 {
    if jd_ut == 0.0 {
        0.0
    } else {
        ut_to_tt(jd_ut)
    }
}

/// Format a TT instant, asserting it is inside the 1900–2100 window. A zero
/// (SE "absent contact") is rendered as an empty CSV field.
fn fmt_contact(jd: f64, ctx: &str) -> String {
    if jd == 0.0 {
        return String::new();
    }
    assert!(
        (JD_WINDOW_LO..=JD_WINDOW_HI).contains(&jd),
        "{ctx}: contact instant {jd} outside 1900–2100 window"
    );
    format!("{jd:.9}")
}

/// (az_from_south_deg, true_alt_deg, apparent_alt_deg) of body `ipl` at the TT
/// instant `jd_tt`, exactly as SE returns: azimuth measured from SOUTH,
/// increasing WESTWARD.
///
/// To match the engine's `local::body_horizontal` — which samples the body at
/// the numeric instant J (as ephemeris/TDB time) yet computes Earth rotation
/// treating that SAME numeric J verbatim as UT1 — this uses the identical
/// numeric `jd_tt` (= J) for BOTH steps:
///   * the apparent position is taken at TT=`jd_tt` via `swe_calc` (whose time
///     argument IS ET/TT, so no ΔT is added), and
///   * `swe_azalt` is fed `jd_tt` as its (nominally UT) time argument, so its
///     sidereal time is computed from J-as-UT1, exactly as the engine does.
/// The result is a fair MODEL-parity comparison; it is NOT a ΔT rotation offset.
fn body_azalt(ipl: c_int, lat: f64, lon: f64, elev: f64, jd_tt: f64) -> (f64, f64, f64) {
    // Apparent TOPOCENTRIC ecliptic-of-date lon/lat/dist at TT=jd_tt (Moshier).
    // `swe_azalt` itself applies NO diurnal parallax (it sets the input radius to
    // 1 and only rotates geocentric direction → horizontal), so the parallax must
    // be baked into the position here via SEFLG_TOPOCTR — otherwise the Moon
    // (~1° horizontal parallax) would disagree with the engine's topocentric
    // az/alt by up to a degree. swe_set_topo sets the observer for the parallax.
    unsafe { swe_set_topo(lon, lat, elev) };
    let mut xx = [0.0_f64; 6];
    let mut serr = [0 as c_char; 256];
    // `swe_calc_ut(J, TOPOCTR)`: SE forms the parallax observer's position at
    // `tjd_ut = J` (`swi_get_observer` uses `tjd_ut = tjd_et - ΔT`, and here
    // `tjd_et = J + ΔT`), so the diurnal parallax is rotated with J-as-UT1 — the
    // SAME sidereal phase the engine uses. (Feeding `swe_calc(J)` instead would
    // rotate parallax at J-ΔT, leaving a ~ΔT azimuthal-parallax error of ~0.3°.)
    let ret = unsafe {
        swe_calc_ut(
            jd_tt,
            ipl,
            SEFLG_MOSEPH | SEFLG_TOPOCTR,
            xx.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };
    if ret < 0 {
        panic!("swe_calc_ut(ipl={ipl}) at jd_tt={jd_tt}: {}", serr_string(&serr));
    }
    let mut geopos = [lon, lat, elev];
    let mut xin = [xx[0], xx[1], xx[2]]; // ecliptic lon, lat, distance
    let mut xaz = [0.0_f64; 3];
    unsafe {
        // Feed jd_tt (= J) verbatim as swe_azalt's time argument: its sidereal
        // time is then computed from J-as-UT1, matching the engine's convention.
        swe_azalt(
            jd_tt,
            SE_ECL2HOR,
            geopos.as_mut_ptr(),
            STD_PRESS,
            STD_TEMP,
            xin.as_mut_ptr(),
            xaz.as_mut_ptr(),
        );
    }
    assert!(
        xaz[0].is_finite() && xaz[1].is_finite() && xaz[2].is_finite(),
        "non-finite swe_azalt output for ipl={ipl} at jd_tt={jd_tt}"
    );
    (xaz[0], xaz[1], xaz[2]) // az (from south), true alt, apparent alt
}

/// (label, seed date, lat, lon, elev_m).
type Case = (&'static str, (c_int, c_int, c_int), f64, f64, f64);

// ---------------------------------------------------------------------------
// Solar
// ---------------------------------------------------------------------------

/// Emit one solar local-circumstances row.
fn emit_solar(out: &mut String, rows: &mut usize, case: &Case) {
    let (label, (y, mo, d), lat, lon, elev) = *case;
    let seed = jd_ut(y, mo, d);
    let mut geopos = [lon, lat, elev];
    let mut serr = [0 as c_char; 256];

    // 1) Anchor: the specific next global eclipse after the seed.
    let mut tret_g = [0.0_f64; 10];
    let retg = unsafe {
        swe_sol_eclipse_when_glob(
            seed,
            SEFLG_MOSEPH,
            IFLTYPE_ANY,
            tret_g.as_mut_ptr(),
            SEARCH_FORWARD,
            serr.as_mut_ptr(),
        )
    };
    if retg < 0 {
        panic!("swe_sol_eclipse_when_glob({label}): {}", serr_string(&serr));
    }
    let jd_max_glob = tret_g[0];
    assert!(
        (JD_WINDOW_LO..=JD_WINDOW_HI).contains(&jd_max_glob),
        "{label}: global solar max {jd_max_glob} outside window"
    );

    // 2) Local: next locally-visible eclipse after the seed.
    let mut tret_l = [0.0_f64; 10];
    let mut attr_l = [0.0_f64; 20];
    let retl = unsafe {
        swe_sol_eclipse_when_loc(
            seed,
            SEFLG_MOSEPH,
            geopos.as_mut_ptr(),
            tret_l.as_mut_ptr(),
            attr_l.as_mut_ptr(),
            SEARCH_FORWARD,
            serr.as_mut_ptr(),
        )
    };
    if retl < 0 {
        panic!("swe_sol_eclipse_when_loc({label}): {}", serr_string(&serr));
    }

    let locally_visible = (tret_l[0] - jd_max_glob).abs() <= SAME_ECLIPSE_TOL_DAYS;

    let (max_tt, c1, c2, c3, c4, local_type, magnitude, obscuration, az, talt, aalt, visible);
    if locally_visible {
        // Topocentric contacts + attributes from when_loc, converted UT → TT.
        // tret: [0]=local max, [1]=C1, [2]=C2, [3]=C3, [4]=C4.
        max_tt = ut_to_tt(tret_l[0]);
        c1 = fmt_contact(contact_tt(tret_l[1]), label);
        c2 = fmt_contact(contact_tt(tret_l[2]), label); // empty for partial-only
        c3 = fmt_contact(contact_tt(tret_l[3]), label); // empty for partial-only
        c4 = fmt_contact(contact_tt(tret_l[4]), label);
        // attr: [0]=magnitude (diameter fraction), [2]=obscuration (area fraction).
        magnitude = attr_l[0];
        obscuration = attr_l[2];
        local_type = retl & SE_ECL_ALLTYPES_SOLAR;
        let (a, t, ap) = body_azalt(SE_SUN, lat, lon, elev, max_tt);
        az = a;
        talt = t;
        aalt = ap;
        visible = 1;
    } else {
        // Not locally visible: characterise the anchored eclipse at the global
        // max with swe_sol_eclipse_how. No local contact instants exist.
        let mut attr_h = [0.0_f64; 20];
        let reth = unsafe {
            swe_sol_eclipse_how(
                jd_max_glob,
                SEFLG_MOSEPH,
                geopos.as_mut_ptr(),
                attr_h.as_mut_ptr(),
                serr.as_mut_ptr(),
            )
        };
        if reth < 0 {
            panic!("swe_sol_eclipse_how({label}): {}", serr_string(&serr));
        }
        max_tt = ut_to_tt(jd_max_glob);
        c1 = String::new();
        c2 = String::new();
        c3 = String::new();
        c4 = String::new();
        magnitude = attr_h[0];
        obscuration = attr_h[2];
        local_type = reth & SE_ECL_ALLTYPES_SOLAR; // 0 when no eclipse there
        let (a, t, ap) = body_azalt(SE_SUN, lat, lon, elev, max_tt);
        az = a;
        talt = t;
        aalt = ap;
        visible = 0;
    }

    out.push_str(&format!(
        "{label},{lat:.4},{lon:.4},{elev:.1},{max_tt:.9},{c1},{c2},{c3},{c4},{local_type},{magnitude:.6},{obscuration:.6},{az:.9},{talt:.9},{aalt:.9},{visible}\n"
    ));
    *rows += 1;
}

fn build_solar_csv(version: &str) -> (String, usize) {
    let mut out = String::new();
    let mut rows = 0usize;
    out.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), swe_sol_eclipse_when_glob/when_loc/how + swe_azalt, iflag=SEFLG_MOSEPH.\n"
    ));
    out.push_str("# Local (topocentric) solar-eclipse circumstances. Times are TT (Terrestrial Time) Julian days: eclipse instants are DYNAMICAL, matching the engine and the global corpus's greatest_eclipse_jd_tt (contrast rise-trans, which is UT). SE returns UT; converted TT = jd_ut + swe_deltat(jd_ut). Anchored via when_glob; when_loc gives topocentric contacts when locally visible.\n");
    out.push_str("# se_max_jd_tt: local maximum (when_loc) if visible, else global maximum (when_glob). se_c*_jd_tt: C1..C4 topocentric contacts; empty = absent (C2/C3 for partial-only, or all when not locally visible).\n");
    out.push_str("# se_local_type = SE return flag & SE_ECL_ALLTYPES_SOLAR (4=total,8=annular,16=partial,32=annular-total,0=none-here). se_magnitude=diameter fraction (attr[0]); se_obscuration=area fraction (attr[2]).\n");
    out.push_str("# Azimuth of Sun from SOUTH increasing WESTWARD [0,360) (swe_azalt native). se_max_true_alt/app_alt in degrees. Az/alt computed at the engine-consumed numeric JD (the TT instant: swe_calc position at TT, swe_azalt rotation with that same J as UT1). se_any_visible=1 if the eclipse is above the horizon locally, else 0 (Sun down / no eclipse there).\n");
    out.push_str(
        "label,lat_deg,lon_deg,elev_m,se_max_jd_tt,se_c1_jd_tt,se_c2_jd_tt,se_c3_jd_tt,se_c4_jd_tt,se_local_type,se_magnitude,se_obscuration,se_max_az_deg,se_max_true_alt_deg,se_max_app_alt_deg,se_any_visible\n",
    );

    // Curated solar cases: seed a few days before each eclipse. Observers span
    // in-path (total/annular), partial-visible, and below-horizon / no-eclipse.
    let cases: &[Case] = &[
        // 2017-08-21 Great American total eclipse (seed 2017-08-19).
        ("2017TSE_SalemOR", (2017, 8, 19), 44.94, -123.04, 60.0),
        ("2017TSE_MadrasOR", (2017, 8, 19), 44.63, -121.13, 683.0),
        ("2017TSE_IdahoFalls", (2017, 8, 19), 43.49, -112.04, 1432.0),
        ("2017TSE_KansasCity", (2017, 8, 19), 39.10, -94.58, 277.0),
        ("2017TSE_Nashville", (2017, 8, 19), 36.16, -86.78, 169.0),
        ("2017TSE_CharlestonSC", (2017, 8, 19), 32.78, -79.93, 6.0),
        ("2017TSE_NYC_partial", (2017, 8, 19), 40.71, -74.01, 10.0),
        ("2017TSE_LosAngeles_partial", (2017, 8, 19), 34.05, -118.24, 89.0),
        ("2017TSE_Miami_partial", (2017, 8, 19), 25.76, -80.19, 2.0),
        ("2017TSE_Tokyo_belowhoriz", (2017, 8, 19), 35.68, 139.69, 40.0),
        // 2024-04-08 total eclipse (seed 2024-04-06).
        ("2024TSE_Mazatlan", (2024, 4, 6), 23.22, -106.42, 10.0),
        ("2024TSE_Torreon", (2024, 4, 6), 25.54, -103.41, 1120.0),
        ("2024TSE_Dallas", (2024, 4, 6), 32.78, -96.80, 131.0),
        ("2024TSE_Indianapolis", (2024, 4, 6), 39.77, -86.16, 218.0),
        ("2024TSE_Cleveland", (2024, 4, 6), 41.50, -81.69, 199.0),
        ("2024TSE_Miami_partial", (2024, 4, 6), 25.76, -80.19, 2.0),
        ("2024TSE_NYC_partial", (2024, 4, 6), 40.71, -74.01, 10.0),
        ("2024TSE_Tokyo_belowhoriz", (2024, 4, 6), 35.68, 139.69, 40.0),
        // 2023-10-14 annular eclipse (seed 2023-10-12).
        ("2023ASE_SanAntonio", (2023, 10, 12), 29.42, -98.49, 198.0),
        ("2023ASE_Albuquerque", (2023, 10, 12), 35.08, -106.65, 1619.0),
        ("2023ASE_CorpusChristi", (2023, 10, 12), 27.80, -97.40, 3.0),
        ("2023ASE_Denver_partial", (2023, 10, 12), 39.74, -104.99, 1609.0),
        ("2023ASE_NYC_partial", (2023, 10, 12), 40.71, -74.01, 10.0),
        ("2023ASE_MexicoCity_partial", (2023, 10, 12), 19.43, -99.13, 2240.0),
        // 2013-11-03 hybrid eclipse (seed 2013-11-01).
        ("2013HSE_Libreville_partial", (2013, 11, 1), 0.39, 9.45, 10.0),
        ("2013HSE_MidAtlantic_partial", (2013, 11, 1), 10.0, -40.0, 0.0),
        ("2013HSE_Madrid_partial", (2013, 11, 1), 40.42, -3.70, 667.0),
        ("2013HSE_Lagos_partial", (2013, 11, 1), 6.52, 3.38, 41.0),
        ("2013HSE_Nairobi_partial", (2013, 11, 1), -1.29, 36.82, 1795.0),
    ];
    for c in cases {
        emit_solar(&mut out, &mut rows, c);
    }
    (out, rows)
}

// ---------------------------------------------------------------------------
// Lunar
// ---------------------------------------------------------------------------

/// Emit one lunar local-circumstances row. Lunar shadow contacts are global
/// (identical for every observer); the local layer is magnitude + Moon az/alt +
/// visibility at the global maximum.
fn emit_lunar(out: &mut String, rows: &mut usize, case: &Case) {
    let (label, (y, mo, d), lat, lon, elev) = *case;
    let seed = jd_ut(y, mo, d);
    let mut geopos = [lon, lat, elev];
    let mut serr = [0 as c_char; 256];

    // Global lunar eclipse contacts (observer-independent).
    // tret: [0]=max, [2]=U1(partial begin), [3]=U4(partial end),
    //       [4]=U2(totality begin), [5]=U3(totality end),
    //       [6]=P1(penumbral begin), [7]=P4(penumbral end).
    let mut tret_g = [0.0_f64; 10];
    let retw = unsafe {
        swe_lun_eclipse_when(
            seed,
            SEFLG_MOSEPH,
            IFLTYPE_ANY,
            tret_g.as_mut_ptr(),
            SEARCH_FORWARD,
            serr.as_mut_ptr(),
        )
    };
    if retw < 0 {
        panic!("swe_lun_eclipse_when({label}): {}", serr_string(&serr));
    }
    let jd_max = tret_g[0];
    assert!(
        (JD_WINDOW_LO..=JD_WINDOW_HI).contains(&jd_max),
        "{label}: lunar max {jd_max} outside window"
    );
    let se_type = retw & SE_ECL_ALLTYPES_LUNAR;

    // Per-observer magnitude + Moon apparent altitude at the maximum.
    // attr: [0]=umbral magnitude, [1]=penumbral magnitude, [6]=apparent altitude.
    let mut attr = [0.0_f64; 20];
    let reth = unsafe {
        swe_lun_eclipse_how(
            jd_max,
            SEFLG_MOSEPH,
            geopos.as_mut_ptr(),
            attr.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };
    if reth < 0 {
        panic!("swe_lun_eclipse_how({label}): {}", serr_string(&serr));
    }
    let umbral_mag = attr[0];
    let penumbral_mag = attr[1];

    // Global contacts + max converted UT → TT (engine's dynamical time base).
    let jd_max_tt = ut_to_tt(jd_max);

    // Moon az/alt at the TT maximum (from-south azimuth, matching the engine;
    // computed at the engine-consumed numeric JD — see `body_azalt`).
    let (az, talt, aalt) = body_azalt(SE_MOON, lat, lon, elev, jd_max_tt);
    let visible = if aalt > 0.0 { 1 } else { 0 };

    let p1 = fmt_contact(contact_tt(tret_g[6]), label);
    let u1 = fmt_contact(contact_tt(tret_g[2]), label);
    let u2 = fmt_contact(contact_tt(tret_g[4]), label);
    let u3 = fmt_contact(contact_tt(tret_g[5]), label);
    let u4 = fmt_contact(contact_tt(tret_g[3]), label);
    let p4 = fmt_contact(contact_tt(tret_g[7]), label);

    out.push_str(&format!(
        "{label},{lat:.4},{lon:.4},{elev:.1},{jd_max_tt:.9},{p1},{u1},{u2},{u3},{u4},{p4},{se_type},{umbral_mag:.6},{penumbral_mag:.6},{az:.9},{talt:.9},{aalt:.9},{visible}\n"
    ));
    *rows += 1;
}

fn build_lunar_csv(version: &str) -> (String, usize) {
    let mut out = String::new();
    let mut rows = 0usize;
    out.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), swe_lun_eclipse_when/how + swe_azalt, iflag=SEFLG_MOSEPH.\n"
    ));
    out.push_str("# Local lunar-eclipse circumstances. Times are TT (Terrestrial Time) Julian days: eclipse instants are DYNAMICAL, matching the engine and the global corpus's greatest_eclipse_jd_tt (contrast rise-trans, which is UT). SE returns UT; converted TT = jd_ut + swe_deltat(jd_ut). Shadow contacts P1/U1/U2/U3/U4/P4 are GLOBAL instants (swe_lun_eclipse_when), identical for every observer; empty = absent phase (U1..U4 for penumbral-only, U2/U3 for partial-only).\n");
    out.push_str("# se_type = SE return flag & SE_ECL_ALLTYPES_LUNAR (4=total,16=partial,64=penumbral). se_umbral_mag=attr[0], se_penumbral_mag=attr[1] (swe_lun_eclipse_how at maximum).\n");
    out.push_str("# Moon azimuth from SOUTH increasing WESTWARD [0,360) (swe_azalt native); true/apparent altitude in degrees at the maximum. Az/alt computed at the engine-consumed numeric JD (the TT instant: swe_calc position at TT, swe_azalt rotation with that same J as UT1). se_any_visible=1 if the Moon is above the horizon (apparent alt>0) at maximum, else 0.\n");
    out.push_str(
        "label,lat_deg,lon_deg,elev_m,se_max_jd_tt,se_p1_jd_tt,se_u1_jd_tt,se_u2_jd_tt,se_u3_jd_tt,se_u4_jd_tt,se_p4_jd_tt,se_type,se_umbral_mag,se_penumbral_mag,se_max_az_deg,se_max_true_alt_deg,se_max_app_alt_deg,se_any_visible\n",
    );

    // Curated lunar cases: seed a few days before each eclipse. Observers span
    // Moon-up (visible) and Moon-down (below horizon) at the maximum.
    let cases: &[Case] = &[
        // 2018-07-27 total lunar eclipse (seed 2018-07-25).
        ("2018TLE_CapeTown", (2018, 7, 25), -33.92, 18.42, 42.0),
        ("2018TLE_Rome", (2018, 7, 25), 41.90, 12.50, 21.0),
        ("2018TLE_Delhi", (2018, 7, 25), 28.61, 77.21, 216.0),
        ("2018TLE_Nairobi", (2018, 7, 25), -1.29, 36.82, 1795.0),
        ("2018TLE_Moscow", (2018, 7, 25), 55.75, 37.62, 156.0),
        ("2018TLE_Sydney", (2018, 7, 25), -33.87, 151.21, 58.0),
        ("2018TLE_LosAngeles_down", (2018, 7, 25), 34.05, -118.24, 89.0),
        ("2018TLE_NYC_down", (2018, 7, 25), 40.71, -74.01, 10.0),
        // 2019-07-16 partial lunar eclipse (seed 2019-07-14).
        ("2019PLE_Rome", (2019, 7, 14), 41.90, 12.50, 21.0),
        ("2019PLE_Cairo", (2019, 7, 14), 30.04, 31.24, 23.0),
        ("2019PLE_Johannesburg", (2019, 7, 14), -26.20, 28.05, 1753.0),
        ("2019PLE_London", (2019, 7, 14), 51.51, -0.13, 11.0),
        ("2019PLE_Delhi", (2019, 7, 14), 28.61, 77.21, 216.0),
        ("2019PLE_LosAngeles_down", (2019, 7, 14), 34.05, -118.24, 89.0),
        // 2020-01-10 penumbral lunar eclipse (seed 2020-01-08).
        ("2020NLE_Delhi", (2020, 1, 8), 28.61, 77.21, 216.0),
        ("2020NLE_Moscow", (2020, 1, 8), 55.75, 37.62, 156.0),
        ("2020NLE_Beijing", (2020, 1, 8), 39.90, 116.41, 44.0),
        ("2020NLE_Rome", (2020, 1, 8), 41.90, 12.50, 21.0),
        ("2020NLE_London", (2020, 1, 8), 51.51, -0.13, 11.0),
        ("2020NLE_NYC_down", (2020, 1, 8), 40.71, -74.01, 10.0),
    ];
    for c in cases {
        emit_lunar(&mut out, &mut rows, c);
    }
    (out, rows)
}

// ---------------------------------------------------------------------------
// Manifest / driver
// ---------------------------------------------------------------------------

fn build_manifest(
    version: &str,
    sol_csv: &str,
    sol_rows: usize,
    lun_csv: &str,
    lun_rows: usize,
) -> String {
    let mut m = String::new();
    m.push_str("corpus: sol-local+lun-local\n");
    m.push_str(&format!(
        "source: Swiss Ephemeris {version} (Moshier, SEFLG_MOSEPH)\n"
    ));
    m.push_str("generator: tools/se-eclipse-local-reference\n");
    m.push_str(&format!(
        "file: sol-local.csv rows={sol_rows} checksum={}\n",
        fnv1a64(sol_csv)
    ));
    m.push_str(&format!(
        "file: lun-local.csv rows={lun_rows} checksum={}\n",
        fnv1a64(lun_csv)
    ));
    m.push_str("solar: swe_sol_eclipse_when_glob anchor + swe_sol_eclipse_when_loc (topocentric contacts, visible) / swe_sol_eclipse_how (magnitude when not locally visible)\n");
    m.push_str("lunar: swe_lun_eclipse_when (global contacts) + swe_lun_eclipse_how (per-observer magnitude, Moon altitude, visibility)\n");
    m.push_str("azimuth: measured from SOUTH, increasing WESTWARD (swe_azalt native convention); az/alt at the engine-consumed numeric JD (TT instant): swe_calc position at TT, swe_azalt rotation with that same J as UT1\n");
    m.push_str("times: TT (Terrestrial Time) Julian days = jd_ut + swe_deltat(jd_ut); eclipse instants are dynamical, matching the engine and the global corpus greatest_eclipse_jd_tt (contrast rise-trans UT)\n");
    m.push_str("visibility: se_any_visible=1 if the eclipse is above the horizon locally at maximum, else 0\n");
    m.push_str("window: 1900-2100 CE (JD 2415020.5..=2488069.5)\n");
    m
}

struct Config {
    dry_run: bool,
    out_dir: String,
    ephe_dir: String,
}

fn parse_args() -> Config {
    let mut dry_run = false;
    let mut out_dir = ".".to_string();
    let mut ephe_dir = env::var("SE_EPHE_PATH").unwrap_or_else(|_| "/tmp".to_string());
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--dry-run" => dry_run = true,
            "--out" => out_dir = args.next().expect("--out needs a directory"),
            "--ephe" => ephe_dir = args.next().expect("--ephe needs a directory"),
            other if !other.starts_with("--") => out_dir = other.to_string(),
            other => panic!("unknown argument: {other}"),
        }
    }
    Config {
        dry_run,
        out_dir,
        ephe_dir,
    }
}

fn main() {
    let cfg = parse_args();

    let ephe = std::ffi::CString::new(cfg.ephe_dir.clone()).expect("ephe path has NUL");
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };

    let version = se_version();
    let (sol_csv, sol_rows) = build_solar_csv(&version);
    let (lun_csv, lun_rows) = build_lunar_csv(&version);
    let manifest = build_manifest(&version, &sol_csv, sol_rows, &lun_csv, lun_rows);

    if cfg.dry_run {
        print!("{sol_csv}");
        println!("# ---- lun-local.csv ----");
        print!("{lun_csv}");
        println!("# ---- manifest.txt ----");
        print!("{manifest}");
        eprintln!(
            "dry-run: sol-local rows={sol_rows}, lun-local rows={lun_rows}, SE {version} (ephe={})",
            cfg.ephe_dir
        );
        return;
    }

    let sol_path = format!("{}/sol-local.csv", cfg.out_dir);
    let lun_path = format!("{}/lun-local.csv", cfg.out_dir);
    let mf_path = format!("{}/manifest.txt", cfg.out_dir);
    std::fs::write(&sol_path, &sol_csv).unwrap_or_else(|e| panic!("write {sol_path}: {e}"));
    std::fs::write(&lun_path, &lun_csv).unwrap_or_else(|e| panic!("write {lun_path}: {e}"));
    std::fs::write(&mf_path, &manifest).unwrap_or_else(|e| panic!("write {mf_path}: {e}"));
    eprintln!(
        "wrote {sol_path} ({sol_rows} rows), {lun_path} ({lun_rows} rows), {mf_path}; SE {version}"
    );
}
