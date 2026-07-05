//! Emits a Swiss Ephemeris rise / set / transit + horizontal-coordinate
//! reference corpus for the pure-Rust `pleiades-events` engine.
//!
//! Two CSVs plus a manifest:
//!   * `rise-trans.csv` — `swe_rise_trans` / `swe_rise_trans_true_hor` over a
//!     representative fixture matrix of (object, event, observer, flag preset).
//!     Bodies (Sun/Moon/Mars) go through the `ipl` path; fixed stars
//!     (Aldebaran, Regulus) through the `swe_fixstar` starname path. The UT JD
//!     SE returns is converted to TDB the SAME way the sibling harnesses do:
//!     `jd_tdb = jd_ut + swe_deltat(jd_ut)`. Circumpolar / no-rise rows (SE
//!     return code -2) are recorded with `none` so the gate can assert `None`.
//!   * `azalt.csv` — `swe_azalt` with `SE_ECL2HOR` over (ecliptic point,
//!     observer, epoch, atmosphere). Records azimuth + true altitude + apparent
//!     altitude EXACTLY as SE returns them. Azimuth convention: measured from
//!     SOUTH, increasing WESTWARD (swe_azalt native; our engine matches it).
//!   * `manifest.txt` — per-CSV row counts + `fnv1a64` checksum of the CSV
//!     bytes + the SE version string + the conventions above.
//!
//! Ecliptic-point rise/set (Option B): SE `swe_rise_trans` operates only on
//! planets and named stars, NOT arbitrary ecliptic points, so the (lon 90°,
//! lat 0°) ecliptic-point rise/set fixture is OMITTED from `rise-trans.csv`.
//! Ecliptic points ARE exercised against SE in `azalt.csv` (`SE_ECL2HOR`
//! genuinely supports them); the point's rise/set is left to the gate's Tier-1
//! self-consistency check. No SE reference is fabricated for what SE did not
//! compute.
//!
//! Ephemeris: Moshier (`SEFLG_MOSEPH`) — no `.se1` files. The fixed-star
//! fixtures need `sefstars.txt` in the ephemeris path (`--ephe <dir>` or the
//! `SE_EPHE_PATH` env var, default `/tmp`); that catalog is NOT committed.
//!
//! Usage:
//!   se-rise-trans-reference --dry-run            # print CSVs+manifest, no writes
//!   se-rise-trans-reference --out <dir>          # write the three files
//!   se-rise-trans-reference --out <dir> --ephe <dir-with-sefstars.txt>
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{
    swe_azalt, swe_deltat, swe_rise_trans, swe_rise_trans_true_hor, swe_set_ephe_path, swe_version,
};

const SEFLG_MOSEPH: c_int = 4; // Moshier ephemeris, no data files

// swe_rise_trans event selectors (rsmi).
const SE_CALC_RISE: c_int = 1;
const SE_CALC_SET: c_int = 2;
const SE_CALC_MTRANSIT: c_int = 4; // upper (meridian) transit
const SE_CALC_ITRANSIT: c_int = 8; // lower transit

// swe_rise_trans flag bits (or'ed into rsmi for rise/set).
const SE_BIT_DISC_CENTER: c_int = 256;
const SE_BIT_DISC_BOTTOM: c_int = 8192; // lower limb
const SE_BIT_GEOCTR_NO_ECL_LAT: c_int = 128;
const SE_BIT_NO_REFRACTION: c_int = 512;
const SE_BIT_FIXED_DISC_SIZE: c_int = 16384;
const SE_BIT_HINDU_RISING: c_int =
    SE_BIT_DISC_CENTER | SE_BIT_NO_REFRACTION | SE_BIT_GEOCTR_NO_ECL_LAT;

// swe_azalt coordinate mode.
const SE_ECL2HOR: c_int = 0;

// SE returns -2 (with tret[0] == 0) when no rise/set exists (circumpolar or
// never rises); 0 == OK, -1 == error.
const SE_RT_NO_EVENT: c_int = -2;

// 1900-2100 CE window (Julian days); every fixture start + result lands inside.
const JD_WINDOW_LO: f64 = 2_415_020.5; // 1900-01-01
const JD_WINDOW_HI: f64 = 2_488_069.5; // 2100-01-01

/// FNV-1a 64-bit over the CSV bytes — identical to `pleiades_time::fnv1a64`
/// / `pleiades_apparent::fnv1a64`, recomputed here so the manifest checksum
/// matches the crossings-manifest scheme byte-for-byte.
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

/// The object whose event is sought. Bodies use the SE `ipl` path; stars use
/// the `swe_fixstar` starname path of `swe_rise_trans`.
#[derive(Clone, Copy)]
enum Object {
    Body { name: &'static str, ipl: c_int },
    Star { name: &'static str },
}

impl Object {
    fn label(&self) -> &'static str {
        match self {
            Object::Body { name, .. } => name,
            Object::Star { name } => name,
        }
    }
}

const SUN: Object = Object::Body { name: "Sun", ipl: 0 };
const MOON: Object = Object::Body { name: "Moon", ipl: 1 };
const MARS: Object = Object::Body { name: "Mars", ipl: 4 };
const ALDEBARAN: Object = Object::Star { name: "Aldebaran" };
const REGULUS: Object = Object::Star { name: "Regulus" };

#[derive(Clone, Copy, PartialEq)]
enum Event {
    Rise,
    Set,
    UpperTransit,
    LowerTransit,
}

impl Event {
    fn label(&self) -> &'static str {
        match self {
            Event::Rise => "Rise",
            Event::Set => "Set",
            Event::UpperTransit => "UpperTransit",
            Event::LowerTransit => "LowerTransit",
        }
    }
    fn base_rsmi(&self) -> c_int {
        match self {
            Event::Rise => SE_CALC_RISE,
            Event::Set => SE_CALC_SET,
            Event::UpperTransit => SE_CALC_MTRANSIT,
            Event::LowerTransit => SE_CALC_ITRANSIT,
        }
    }
    fn is_transit(&self) -> bool {
        matches!(self, Event::UpperTransit | Event::LowerTransit)
    }
}

/// A `swe_rise_trans` flag preset, mapped to our engine's `RiseSetOptions`
/// component fields so the gate can reconstruct the call exactly.
#[derive(Clone, Copy)]
struct Preset {
    label: &'static str,
    disc: &'static str, // "upper" | "center" | "lower"
    refraction: bool,
    no_ecl_lat: bool,
    fixed_disc: bool,
    hindu: bool,
    horizon: Option<f64>, // Some(h) => swe_rise_trans_true_hor(horhgt = h)
    bits: c_int,          // rsmi bits or'ed for rise/set
}

const P_DEFAULT: Preset = Preset {
    label: "default",
    disc: "upper",
    refraction: true,
    no_ecl_lat: false,
    fixed_disc: false,
    hindu: false,
    horizon: None,
    bits: 0,
};
const P_CENTER_NOREFR: Preset = Preset {
    label: "center_norefr",
    disc: "center",
    refraction: false,
    no_ecl_lat: false,
    fixed_disc: false,
    hindu: false,
    horizon: None,
    bits: SE_BIT_DISC_CENTER | SE_BIT_NO_REFRACTION,
};
const P_LOWER_LIMB: Preset = Preset {
    label: "lower_limb",
    disc: "lower",
    refraction: true,
    no_ecl_lat: false,
    fixed_disc: false,
    hindu: false,
    horizon: None,
    bits: SE_BIT_DISC_BOTTOM,
};
const P_FIXED_DISC: Preset = Preset {
    label: "fixed_disc",
    disc: "upper",
    refraction: true,
    no_ecl_lat: false,
    fixed_disc: true,
    hindu: false,
    horizon: None,
    bits: SE_BIT_FIXED_DISC_SIZE,
};
const P_HINDU: Preset = Preset {
    label: "hindu",
    disc: "center",
    refraction: false,
    no_ecl_lat: true,
    fixed_disc: false,
    hindu: true,
    horizon: None,
    bits: SE_BIT_HINDU_RISING,
};
const P_HORIZON5: Preset = Preset {
    label: "horizon_plus5",
    disc: "upper",
    refraction: true,
    no_ecl_lat: false,
    fixed_disc: false,
    hindu: false,
    horizon: Some(5.0),
    bits: 0,
};

/// (label, lat, lon, elev_m).
type Observer = (&'static str, f64, f64, f64);
const OBS_NYC: Observer = ("NYC", 40.0, -74.0, 10.0);
const OBS_EQ: Observer = ("Equator", 0.0, 0.0, 0.0);
const OBS_HILAT: Observer = ("Rovaniemi", 66.5, 25.0, 0.0);
const OBS_SYD: Observer = ("Sydney", -33.0, 151.0, 50.0);

// Standard atmosphere (hPa/mbar, °C).
const STD_PRESS: f64 = 1013.25;
const STD_TEMP: f64 = 15.0;

// Fixture start epochs (UT Julian days), all in-window.
const START_J2000: f64 = 2_451_544.5; // 2000-01-01 00:00 UT
const START_SUMMER: f64 = 2_451_726.5; // 2000-07-01 (N-hemisphere summer)
// Near the 2024-2025 major lunar standstill: Moon declination exceeds +23.5°,
// so at 66.5°N it is circumpolar and swe_rise_trans returns "no event".
const START_MOON_STANDSTILL: f64 = 2_460_674.5; // 2025-01-05 00:00 UT

/// Convert an SE UT Julian day to TDB the same way the sibling harnesses do.
fn ut_to_tdb(jd_ut: f64) -> f64 {
    jd_ut + unsafe { swe_deltat(jd_ut) }
}

/// Call `swe_rise_trans` (or `swe_rise_trans_true_hor` when a custom horizon is
/// set). Returns (return_code, tret[0]). Panics only on a hard SE error (-1).
#[allow(clippy::too_many_arguments)]
fn call_rise_trans(
    obj: Object,
    rsmi: c_int,
    lat: f64,
    lon: f64,
    elev: f64,
    atpress: f64,
    attemp: f64,
    horizon: Option<f64>,
    start_ut: f64,
) -> (c_int, f64) {
    let mut geopos = [lon, lat, elev];
    let mut tret = [0.0_f64; 10];
    let mut serr = [0 as c_char; 256];

    // For stars, SE reads (and may rewrite) the starname buffer, so pass a
    // mutable, generously sized buffer seeded with the catalog name.
    let mut namebuf: Vec<c_char> = match obj {
        Object::Star { name } => {
            let cs = CString::new(name).unwrap();
            let mut b: Vec<c_char> = cs.as_bytes_with_nul().iter().map(|&x| x as c_char).collect();
            b.resize(256, 0);
            b
        }
        Object::Body { .. } => Vec::new(),
    };
    let (ipl, name_ptr): (c_int, *mut c_char) = match obj {
        Object::Body { ipl, .. } => (ipl, std::ptr::null_mut()),
        Object::Star { .. } => (0, namebuf.as_mut_ptr()),
    };

    let ret = unsafe {
        match horizon {
            Some(h) => swe_rise_trans_true_hor(
                start_ut,
                ipl,
                name_ptr,
                SEFLG_MOSEPH,
                rsmi,
                geopos.as_mut_ptr(),
                atpress,
                attemp,
                h,
                tret.as_mut_ptr(),
                serr.as_mut_ptr(),
            ),
            None => swe_rise_trans(
                start_ut,
                ipl,
                name_ptr,
                SEFLG_MOSEPH,
                rsmi,
                geopos.as_mut_ptr(),
                atpress,
                attemp,
                tret.as_mut_ptr(),
                serr.as_mut_ptr(),
            ),
        }
    };
    if ret == -1 {
        panic!(
            "swe_rise_trans({}, rsmi={rsmi}) hard error at start_ut={start_ut}: {}",
            obj.label(),
            serr_string(&serr)
        );
    }
    (ret, tret[0])
}

/// Emit one rise/set/transit row. For transit events the disc/refraction/etc.
/// flag columns are recorded but SE ignores them (a transit is a meridian
/// crossing); the gate treats transits as flag-independent.
#[allow(clippy::too_many_arguments)]
fn emit_rise_trans(
    out: &mut String,
    rows: &mut usize,
    obj: Object,
    ev: Event,
    obs: Observer,
    p: &Preset,
    atpress: f64,
    attemp: f64,
    start_ut: f64,
) {
    let (_, lat, lon, elev) = obs;
    // Transit is horizon-independent: no disc/refraction/horizon bits apply.
    let (rsmi, horizon, p_used) = if ev.is_transit() {
        (ev.base_rsmi(), None, &P_DEFAULT)
    } else {
        (ev.base_rsmi() | p.bits, p.horizon, p)
    };
    let (ret, jd_ut) =
        call_rise_trans(obj, rsmi, lat, lon, elev, atpress, attemp, horizon, start_ut);

    let (se_ut, se_tdb) = if ret == SE_RT_NO_EVENT || jd_ut == 0.0 {
        ("none".to_string(), "none".to_string())
    } else {
        assert!(
            (JD_WINDOW_LO..=JD_WINDOW_HI).contains(&jd_ut),
            "rise/set result out of window: {} {} start={start_ut} jd_ut={jd_ut}",
            obj.label(),
            ev.label()
        );
        (format!("{jd_ut:.9}"), format!("{:.9}", ut_to_tdb(jd_ut)))
    };
    let hor = p_used
        .horizon
        .map(|h| format!("{h:.3}"))
        .unwrap_or_else(|| "none".to_string());

    out.push_str(&format!(
        "{},{},{lat:.4},{lon:.4},{elev:.1},{},{},{},{},{},{},{},{atpress:.3},{attemp:.3},{start_ut:.6},{se_ut},{se_tdb}\n",
        obj.label(),
        ev.label(),
        p_used.label,
        p_used.disc,
        p_used.refraction as u8,
        p_used.no_ecl_lat as u8,
        p_used.fixed_disc as u8,
        p_used.hindu as u8,
        hor,
    ));
    *rows += 1;
}

fn build_rise_trans_csv(version: &str) -> (String, usize) {
    let mut out = String::new();
    let mut rows = 0usize;
    out.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), swe_rise_trans / swe_rise_trans_true_hor, iflag=SEFLG_MOSEPH.\n"
    ));
    out.push_str("# Bodies via ipl (Sun=0,Moon=1,Mars=4); stars via swe_fixstar starname path. UT->TDB: jd_tdb=jd_ut+swe_deltat(jd_ut).\n");
    out.push_str("# se_jd_ut/se_jd_tdb == 'none' means SE reported no rise/set (return -2: circumpolar or never rises).\n");
    out.push_str("# Disc: default=upper limb; center=SE_BIT_DISC_CENTER; lower=SE_BIT_DISC_BOTTOM. refraction=0 => SE_BIT_NO_REFRACTION. fixed_disc=SE_BIT_FIXED_DISC_SIZE. hindu=SE_BIT_HINDU_RISING. horizon_deg => swe_rise_trans_true_hor(horhgt). For transits SE ignores these flags.\n");
    out.push_str("# Ecliptic-point rise/set OMITTED (SE swe_rise_trans has no arbitrary-ecliptic-point support); ecliptic points are covered in azalt.csv.\n");
    out.push_str(
        "object,event,lat_deg,lon_deg,elev_m,preset,disc,refraction,no_ecl_lat,fixed_disc,hindu,horizon_deg,atpress_hpa,attemp_c,start_jd_ut,se_jd_ut,se_jd_tdb\n",
    );

    // Block A — full flag spread on the Sun at NYC (rise & set). Default preset
    // is covered by Block B, so A carries the five non-default presets.
    for ev in [Event::Rise, Event::Set] {
        for p in [
            &P_CENTER_NOREFR,
            &P_LOWER_LIMB,
            &P_FIXED_DISC,
            &P_HINDU,
            &P_HORIZON5,
        ] {
            emit_rise_trans(
                &mut out, &mut rows, SUN, ev, OBS_NYC, p, STD_PRESS, STD_TEMP, START_J2000,
            );
        }
    }

    // Block B — object coverage at NYC, default preset, all four events.
    for obj in [SUN, MOON, MARS, ALDEBARAN, REGULUS] {
        for ev in [
            Event::Rise,
            Event::Set,
            Event::UpperTransit,
            Event::LowerTransit,
        ] {
            emit_rise_trans(
                &mut out, &mut rows, obj, ev, OBS_NYC, &P_DEFAULT, STD_PRESS, STD_TEMP, START_J2000,
            );
        }
    }

    // Block C — observer coverage (Sun + a star), default preset, rise & set.
    for obs in [OBS_EQ, OBS_HILAT, OBS_SYD] {
        for obj in [SUN, ALDEBARAN] {
            for ev in [Event::Rise, Event::Set] {
                emit_rise_trans(
                    &mut out, &mut rows, obj, ev, obs, &P_DEFAULT, STD_PRESS, STD_TEMP, START_J2000,
                );
            }
        }
    }

    // Block D — transits at other observers (always defined, even high-lat).
    for obs in [OBS_EQ, OBS_SYD] {
        for ev in [Event::UpperTransit, Event::LowerTransit] {
            emit_rise_trans(
                &mut out, &mut rows, SUN, ev, obs, &P_DEFAULT, STD_PRESS, STD_TEMP, START_J2000,
            );
        }
    }

    // Block E — circumpolar / no-event: the Moon near a major lunar standstill
    // is circumpolar at 66.5°N, so rise & set both return SE "none". A summer
    // Sun at the same high latitude is a near-grazing case recorded as-is.
    for ev in [Event::Rise, Event::Set] {
        emit_rise_trans(
            &mut out,
            &mut rows,
            MOON,
            ev,
            OBS_HILAT,
            &P_DEFAULT,
            STD_PRESS,
            STD_TEMP,
            START_MOON_STANDSTILL,
        );
    }
    for ev in [Event::Rise, Event::Set] {
        emit_rise_trans(
            &mut out, &mut rows, SUN, ev, OBS_HILAT, &P_DEFAULT, STD_PRESS, STD_TEMP, START_SUMMER,
        );
    }

    (out, rows)
}

/// One `swe_azalt` (SE_ECL2HOR) call. Returns (azimuth, true_alt, apparent_alt),
/// azimuth measured from SOUTH increasing WESTWARD, exactly as SE returns.
#[allow(clippy::too_many_arguments)]
fn se_azalt_call(
    lon_ecl: f64,
    lat_ecl: f64,
    lat: f64,
    lon: f64,
    elev: f64,
    atpress: f64,
    attemp: f64,
    jd_ut: f64,
) -> (f64, f64, f64) {
    let mut geopos = [lon, lat, elev];
    let mut xin = [lon_ecl, lat_ecl, 1.0]; // ecliptic lon/lat + unit distance
    let mut xaz = [0.0_f64; 3];
    unsafe {
        swe_azalt(
            jd_ut,
            SE_ECL2HOR,
            geopos.as_mut_ptr(),
            atpress,
            attemp,
            xin.as_mut_ptr(),
            xaz.as_mut_ptr(),
        );
    }
    assert!(
        xaz[0].is_finite() && xaz[1].is_finite() && xaz[2].is_finite(),
        "non-finite swe_azalt output at jd_ut={jd_ut}"
    );
    (xaz[0], xaz[1], xaz[2])
}

#[allow(clippy::too_many_arguments)]
fn emit_azalt(
    out: &mut String,
    rows: &mut usize,
    lon_ecl: f64,
    lat_ecl: f64,
    obs: Observer,
    atpress: f64,
    attemp: f64,
    jd_ut: f64,
) {
    let (_, lat, lon, elev) = obs;
    let (az, true_alt, app_alt) =
        se_azalt_call(lon_ecl, lat_ecl, lat, lon, elev, atpress, attemp, jd_ut);
    out.push_str(&format!(
        "{lon_ecl:.4},{lat_ecl:.4},{lat:.4},{lon:.4},{elev:.1},{atpress:.3},{attemp:.3},{jd_ut:.6},{az:.9},{true_alt:.9},{app_alt:.9}\n"
    ));
    *rows += 1;
}

fn build_azalt_csv(version: &str) -> (String, usize) {
    let mut out = String::new();
    let mut rows = 0usize;
    out.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), swe_azalt, calc_flag=SE_ECL2HOR.\n"
    ));
    out.push_str("# Input: apparent ecliptic of date (lon,lat) degrees. Azimuth measured from SOUTH, increasing WESTWARD (SE native). Altitudes in degrees.\n");
    out.push_str(
        "lon_ecl_deg,lat_ecl_deg,lat_deg,lon_deg,elev_m,atpress_hpa,attemp_c,jd_ut,se_azimuth_deg,se_true_alt_deg,se_apparent_alt_deg\n",
    );

    // Ecliptic points including the (90,0) point omitted from rise-trans.csv.
    let points: [(f64, f64); 5] =
        [(0.0, 0.0), (90.0, 0.0), (180.0, 0.0), (270.0, 0.0), (45.0, 5.0)];

    // Block A — points across three observers at J2000.
    for &(le, be) in &points {
        for obs in [OBS_NYC, OBS_EQ, OBS_SYD] {
            emit_azalt(&mut out, &mut rows, le, be, obs, STD_PRESS, STD_TEMP, START_J2000);
        }
    }
    // Block B — high-latitude observer + a second epoch for a couple of points.
    for &(le, be) in &[(90.0, 0.0), (45.0, 5.0)] {
        for &jd in &[START_J2000, START_SUMMER] {
            emit_azalt(&mut out, &mut rows, le, be, OBS_HILAT, STD_PRESS, STD_TEMP, jd);
        }
    }
    // Block C — non-standard atmosphere (low pressure / cold) exercises the
    // apparent-altitude refraction term distinctly from the true altitude.
    emit_azalt(&mut out, &mut rows, 90.0, 0.0, OBS_NYC, 950.0, -5.0, START_J2000);

    (out, rows)
}

fn build_manifest(
    version: &str,
    rt_csv: &str,
    rt_rows: usize,
    az_csv: &str,
    az_rows: usize,
) -> String {
    let mut m = String::new();
    m.push_str("corpus: rise-trans+azalt\n");
    m.push_str(&format!("source: Swiss Ephemeris {version} (Moshier, SEFLG_MOSEPH)\n"));
    m.push_str("generator: tools/se-rise-trans-reference\n");
    m.push_str(&format!(
        "file: rise-trans.csv rows={rt_rows} checksum={}\n",
        fnv1a64(rt_csv)
    ));
    m.push_str(&format!(
        "file: azalt.csv rows={az_rows} checksum={}\n",
        fnv1a64(az_csv)
    ));
    m.push_str("rise-set: swe_rise_trans / swe_rise_trans_true_hor; ut_to_tdb: jd_tdb = jd_ut + swe_deltat(jd_ut)\n");
    m.push_str("no-event: se_jd_ut/se_jd_tdb == 'none' (SE return -2: circumpolar or never rises)\n");
    m.push_str("ecliptic-point rise/set: OMITTED (Option B) — SE swe_rise_trans has no arbitrary-ecliptic-point support; ecliptic points covered in azalt.csv (SE_ECL2HOR)\n");
    m.push_str("azimuth: measured from SOUTH, increasing WESTWARD (swe_azalt native convention)\n");
    m.push_str("window: 1900-2100 CE (JD 2415020.5..=2488069.5); rise/set/transit times in TDB\n");
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

    // Fixed-star fixtures resolve `sefstars.txt` from this directory.
    let ephe = CString::new(cfg.ephe_dir.clone()).expect("ephe path has NUL");
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };

    let version = se_version();
    let (rt_csv, rt_rows) = build_rise_trans_csv(&version);
    let (az_csv, az_rows) = build_azalt_csv(&version);
    let manifest = build_manifest(&version, &rt_csv, rt_rows, &az_csv, az_rows);

    if cfg.dry_run {
        print!("{rt_csv}");
        println!("# ---- azalt.csv ----");
        print!("{az_csv}");
        println!("# ---- manifest.txt ----");
        print!("{manifest}");
        eprintln!(
            "dry-run: rise-trans rows={rt_rows}, azalt rows={az_rows}, SE {version} (ephe={})",
            cfg.ephe_dir
        );
        return;
    }

    let rt_path = format!("{}/rise-trans.csv", cfg.out_dir);
    let az_path = format!("{}/azalt.csv", cfg.out_dir);
    let mf_path = format!("{}/manifest.txt", cfg.out_dir);
    std::fs::write(&rt_path, &rt_csv).unwrap_or_else(|e| panic!("write {rt_path}: {e}"));
    std::fs::write(&az_path, &az_csv).unwrap_or_else(|e| panic!("write {az_path}: {e}"));
    std::fs::write(&mf_path, &manifest).unwrap_or_else(|e| panic!("write {mf_path}: {e}"));
    eprintln!("wrote {rt_path} ({rt_rows} rows), {az_path} ({az_rows} rows), {mf_path}; SE {version}");
}
