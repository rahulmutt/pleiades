//! Emits a Swiss Ephemeris **fictitious (hypothetical) body** reference corpus
//! for the pure-Rust `pleiades-fict` backend.
//!
//! One CSV plus a manifest:
//!   * `fictitious.csv` — for each of SE's 19 fictitious bodies (SE numbers
//!     40..=58: Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus,
//!     Poseidon, Transpluto (Isis-Transpluto), Nibiru, Harrington, Neptune
//!     (Leverrier), Neptune (Adams), Pluto (Lowell), Pluto (Pickering),
//!     Vulcan, White Moon (Selena), Proserpina, Waldemath), a per-body date
//!     grid sampled over 1900-2100 TDB: geocentric J2000-ecliptic
//!     longitude/latitude/distance from `swe_calc`.
//!   * `manifest.txt` — row count + `fnv1a64` checksum of the CSV bytes + the
//!     SE version string + the flags/window conventions above.
//!
//! Flags (GEOMETRIC, not apparent): `SEFLG_MOSEPH | SEFLG_J2000 |
//! SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL` = `4 | 32 | 16 | 1024 | 512`.
//! `SEFLG_J2000` alone would emit *apparent* positions (light-time +
//! aberration + gravitational deflection), which would NOT match
//! `FictitiousBackend::position`'s geometric geocentric-J2000 boundary:
//! `SEFLG_TRUEPOS` drops light-time, `SEFLG_NOABERR` drops annual aberration,
//! `SEFLG_NOGDEFL` drops gravitational light deflection.
//!
//! Elements file: SE's built-in fallback element set only covers bodies
//! 40-54; bodies 55-58 (Vulcan, White Moon, Proserpina, Waldemath) require the
//! `seorbel.txt` element file (`SE_FICT_OFFSET_1`-based bodies beyond the
//! built-in table return `SE_NFICT_ELEM` errors otherwise). This tool commits
//! the verbatim upstream `seorbel.txt` under `data/` and points
//! `swe_set_ephe_path` at that directory.
//!
//! Fail-closed: every `swe_calc` return code / `serr` is checked; any SE error
//! aborts the run (so a missing/unreadable `seorbel.txt` cannot silently
//! produce a short corpus), and every lon/lat/dist triple is asserted finite.
//!
//! Window: 1900-2100 CE (JD 2415020.5..=2488069.5 TT), matching the sibling
//! `se-eclipse-local-reference` / `se-eclipse-global-reference` window.
//!
//! Usage:
//!   se-fictitious-reference --dry-run          # print CSV+manifest, no writes
//!   se-fictitious-reference --out <dir>        # write the two files
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::env;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_set_ephe_path, swe_version};

// Geometric (not apparent) flags — see module doc.
const SEFLG_MOSEPH: c_int = 4; // Moshier ephemeris, no data files
const SEFLG_J2000: c_int = 32; // J2000 equinox/ecliptic, no precession
const SEFLG_TRUEPOS: c_int = 16; // true (geometric) position, no light-time
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_NOGDEFL: c_int = 512; // no gravitational light deflection
const IFLAG: c_int =
    SEFLG_MOSEPH | SEFLG_J2000 | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL;

// SE fictitious-body numbers: SE_FICT_OFFSET_1 (39) + 1..=19.
const SE_FICT_FIRST: c_int = 40;
const SE_FICT_LAST: c_int = 58;

// 1900-2100 CE window (Julian days, TT) — matches se-eclipse-local-reference.
const JD_WINDOW_LO: f64 = 2_415_020.5; // 1900-01-01
const JD_WINDOW_HI: f64 = 2_488_069.5; // 2100-01-01

// Samples per body, evenly spaced (inclusive) across the window.
const SAMPLES_PER_BODY: usize = 30;

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

/// `n` Julian Days evenly spaced (inclusive) across `[JD_WINDOW_LO,
/// JD_WINDOW_HI]`.
fn sample_dates(n: usize) -> Vec<f64> {
    assert!(n >= 2, "need at least 2 samples to span the window");
    let step = (JD_WINDOW_HI - JD_WINDOW_LO) / (n - 1) as f64;
    (0..n).map(|i| JD_WINDOW_LO + step * i as f64).collect()
}

/// SE body number (40..=58) → label. Matches the `CelestialBody` enum variant
/// names used by `pleiades-fict`'s `data/fictitious-elements.csv` (same order:
/// SE fictitious sets 1..=19), so the two corpora join on `label` directly.
fn body_label(se_body: c_int) -> &'static str {
    match se_body {
        40 => "Cupido",
        41 => "Hades",
        42 => "Zeus",
        43 => "Kronos",
        44 => "Apollon",
        45 => "Admetos",
        46 => "Vulkanus",
        47 => "Poseidon",
        48 => "Transpluto",
        49 => "Nibiru",
        50 => "Harrington",
        51 => "NeptuneLeverrier",
        52 => "NeptuneAdams",
        53 => "PlutoLowell",
        54 => "PlutoPickering",
        55 => "Vulcan",
        56 => "WhiteMoon",
        57 => "Proserpina",
        58 => "Waldemath",
        other => panic!("SE body {other} outside the fictitious range 40..=58"),
    }
}

/// Emit one body's sampled rows: `label,se_body,jd_tt,lon_deg,lat_deg,dist_au`.
fn emit_body(out: &mut String, rows: &mut usize, se_body: c_int, dates: &[f64]) {
    let label = body_label(se_body);
    for &jd_tt in dates {
        let mut xx = [0.0_f64; 6];
        let mut serr = [0 as c_char; 256];
        let ret = unsafe { swe_calc(jd_tt, se_body, IFLAG, xx.as_mut_ptr(), serr.as_mut_ptr()) };
        if ret < 0 {
            panic!(
                "swe_calc(ipl={se_body} label={label}) at jd_tt={jd_tt}: {}",
                serr_string(&serr)
            );
        }
        let (lon, lat, dist) = (xx[0], xx[1], xx[2]);
        assert!(
            lon.is_finite() && lat.is_finite() && dist.is_finite(),
            "non-finite swe_calc output for {label} (se={se_body}) at jd_tt={jd_tt}: \
             lon={lon} lat={lat} dist={dist}"
        );
        out.push_str(&format!(
            "{label},{se_body},{jd_tt:.9},{lon:.9},{lat:.9},{dist:.9}\n"
        ));
        *rows += 1;
    }
}

fn build_csv(version: &str) -> (String, usize) {
    let mut out = String::new();
    let mut rows = 0usize;
    out.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), swe_calc, iflag={IFLAG} \
         (SEFLG_MOSEPH|SEFLG_J2000|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL).\n"
    ));
    out.push_str(
        "# Fictitious (hypothetical) bodies, SE numbers 40..=58 (SE_FICT_OFFSET_1 + 1..=19). \
         Elements read from the committed data/seorbel.txt (SE's built-in fallback only covers \
         40-54; 55-58 require this file). Geocentric J2000-ecliptic lon/lat (degrees) and \
         distance (AU), GEOMETRIC (no light-time, no aberration, no gravitational deflection) — \
         matches FictitiousBackend::position's geocentric-J2000 boundary. Times are TT \
         (Terrestrial Time) Julian days, evenly sampled per body across the 1900-2100 CE window \
         (JD 2415020.5..=2488069.5).\n",
    );
    out.push_str("label,se_body,jd_tt,lon_deg,lat_deg,dist_au\n");

    let dates = sample_dates(SAMPLES_PER_BODY);
    for se_body in SE_FICT_FIRST..=SE_FICT_LAST {
        emit_body(&mut out, &mut rows, se_body, &dates);
    }
    (out, rows)
}

fn build_manifest(version: &str, csv: &str, rows: usize) -> String {
    let mut m = String::new();
    m.push_str("corpus: fictitious\n");
    m.push_str(&format!(
        "source: Swiss Ephemeris {version} (Moshier, SEFLG_MOSEPH)\n"
    ));
    m.push_str("generator: tools/se-fictitious-reference\n");
    m.push_str(&format!(
        "file: fictitious.csv rows={rows} checksum={}\n",
        fnv1a64(csv)
    ));
    m.push_str(&format!(
        "iflag: {IFLAG} (SEFLG_MOSEPH|SEFLG_J2000|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL) — geometric geocentric J2000\n"
    ));
    m.push_str("bodies: SE 40..=58 (19 fictitious bodies; elements from committed data/seorbel.txt)\n");
    m.push_str(&format!(
        "sampling: {SAMPLES_PER_BODY} dates per body, evenly spaced across the window\n"
    ));
    m.push_str("times: TT (Terrestrial Time) Julian days\n");
    m.push_str("window: 1900-2100 CE (JD 2415020.5..=2488069.5)\n");
    m
}

struct Config {
    dry_run: bool,
    out_dir: String,
    ephe_dir: String,
}

/// Default ephemeris directory: this tool's own `data/` folder (containing the
/// committed `seorbel.txt`), resolved at COMPILE time from the manifest
/// directory so it is correct regardless of the process's current directory
/// (e.g. `cargo run --manifest-path tools/se-fictitious-reference/Cargo.toml`
/// invoked from the repo root).
const DEFAULT_EPHE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");

fn parse_args() -> Config {
    let mut dry_run = false;
    let mut out_dir = ".".to_string();
    let mut ephe_dir =
        env::var("SE_EPHE_PATH").unwrap_or_else(|_| DEFAULT_EPHE_DIR.to_string());
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
    let (csv, rows) = build_csv(&version);
    let manifest = build_manifest(&version, &csv, rows);

    if cfg.dry_run {
        print!("{csv}");
        println!("# ---- manifest.txt ----");
        print!("{manifest}");
        eprintln!(
            "dry-run: fictitious rows={rows} ({} bodies x {SAMPLES_PER_BODY} samples), SE {version} (ephe={})",
            SE_FICT_LAST - SE_FICT_FIRST + 1,
            cfg.ephe_dir
        );
        return;
    }

    let csv_path = format!("{}/fictitious.csv", cfg.out_dir);
    let mf_path = format!("{}/manifest.txt", cfg.out_dir);
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&mf_path, &manifest).unwrap_or_else(|e| panic!("write {mf_path}: {e}"));
    eprintln!("wrote {csv_path} ({rows} rows), {mf_path}; SE {version}");
}
