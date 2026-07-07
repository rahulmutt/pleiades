//! Emits a Swiss Ephemeris **phase, phase angle, elongation, apparent
//! diameter, and magnitude** reference corpus for the upcoming
//! `validate-pheno` gate (SP-5: phase, phase angle & magnitude, a
//! `swe_pheno` analogue).
//!
//! One CSV plus a manifest:
//!   * `pheno.csv` — for each of the ten classical planets (Sun..Pluto, SE
//!     numbers 0-9) at 8 epochs spanning 1900-2100, the five `swe_pheno`
//!     attributes computed via `swe_pheno`. 80 data rows total (10 bodies x
//!     8 epochs).
//!   * `manifest.txt` — row count + `fnv1a64` checksum of the CSV bytes + the
//!     SE version string + the flags/window conventions below.
//!
//! Flags (SE default apparent output minus gravitational deflection; see the
//! SP-5 plan §E8): `SEFLG_MOSEPH | SEFLG_NOGDEFL` = `4 | 512`. Moshier is
//! KERNEL-FREE — no `.se1` data files, no network fetch, unlike SP-4's
//! nod-aps tool (which needed barycentric SWIEPH files for its method-4
//! rows). This tool needs nothing beyond the Moshier analytic ephemeris
//! built into `libswisseph-sys`.
//!
//! `swe_pheno` attr layout (see swephexp.h): `attr[0]` = phase angle
//! (degrees), `attr[1]` = phase (illuminated fraction), `attr[2]` =
//! elongation (degrees), `attr[3]` = apparent diameter (DEGREES — see plan
//! §E1, not arcseconds), `attr[4]` = apparent visual magnitude. For the Sun,
//! SE skips the phase/elongation blocks entirely and leaves them at `0` (see
//! plan §E2) — the generated corpus is expected to show `phase=0` and
//! `elongation=0` for every Sun row.
//!
//! Fail-closed: every `swe_pheno` return code / `serr` is checked; any SE
//! error aborts the run, and every one of the five attr values is asserted
//! finite before the row is appended. No partial rows are ever emitted.
//!
//! Usage:
//!   se-pheno-reference --dry-run          # print CSV+manifest, no writes
//!   se-pheno-reference --out <dir>        # write the two files
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::env;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_pheno, swe_set_ephe_path, swe_version};

// SE default apparent minus gravitational deflection (plan §E8):
// SEFLG_MOSEPH=4, SEFLG_NOGDEFL=512.
const IFLAG: c_int = 4 | 512;

// Interior 1900-2100 epochs (same set as the SP-4 nod-aps corpus): span crosses
// Neptune's 2451544.5 magnitude break and a range of Saturn ring openings and
// inner-planet phase angles.
const EPOCHS: [f64; 8] = [
    2_415_100.5,
    2_433_282.5,
    2_441_683.5,
    2_451_545.0,
    2_459_000.5,
    2_466_154.5,
    2_477_476.5,
    2_488_021.5,
];

/// (label, SE body number): the ten majors.
const BODIES: [(&str, c_int); 10] = [
    ("Sun", 0),
    ("Moon", 1),
    ("Mercury", 2),
    ("Venus", 3),
    ("Mars", 4),
    ("Jupiter", 5),
    ("Saturn", 6),
    ("Uranus", 7),
    ("Neptune", 8),
    ("Pluto", 9),
];

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

/// Compute one `swe_pheno` row and append it to `csv`. Fail-closed: any SE
/// error or non-finite output panics before anything is appended — no
/// partial rows.
///
/// attr[0]=phase angle, [1]=phase, [2]=elongation, [3]=diameter(deg),
/// [4]=magnitude.
fn emit_row(csv: &mut String, rows: &mut usize, label: &str, se_body: c_int, jd_tt: f64) {
    let mut attr = [0.0_f64; 20];
    let mut serr = [0 as c_char; 256];
    let ret = unsafe { swe_pheno(jd_tt, se_body, IFLAG, attr.as_mut_ptr(), serr.as_mut_ptr()) };
    if ret < 0 {
        panic!(
            "swe_pheno(ipl={se_body} label={label} iflag={IFLAG}) at jd_tt={jd_tt}: {}",
            serr_string(&serr)
        );
    }
    for i in 0..5 {
        assert!(
            attr[i].is_finite(),
            "non-finite attr[{i}] for {label} (se={se_body}) at jd_tt={jd_tt}"
        );
    }
    csv.push_str(&format!(
        "{label},{se_body},{jd_tt:.9},{:.9},{:.9},{:.9},{:.9},{:.9}\n",
        attr[0], attr[1], attr[2], attr[3], attr[4]
    ));
    *rows += 1;
}

fn build_csv(version: &str) -> (String, usize) {
    let mut csv = String::new();
    let mut rows = 0usize;
    csv.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), swe_pheno, iflag={IFLAG} \
         (SEFLG_MOSEPH|SEFLG_NOGDEFL).\n"
    ));
    csv.push_str(
        "# Phase/phase-angle/elongation/diameter/magnitude for the classical planets (SE 0-9). \
         attr[0]=phase_angle (deg), attr[1]=phase (illuminated fraction), attr[2]=elongation \
         (deg), attr[3]=diameter (DEGREES, not arcsec), attr[4]=magnitude. For the Sun, SE skips \
         the phase/elongation blocks and leaves them at 0. Times are TT (Terrestrial Time) \
         Julian days.\n",
    );
    csv.push_str("label,se_body,jd_tt,phase_angle,phase,elongation,diameter_deg,magnitude\n");

    // BODIES x EPOCHS in order = 10 x 8 = 80 rows.
    for &(label, se_body) in &BODIES {
        for &jd_tt in &EPOCHS {
            emit_row(&mut csv, &mut rows, label, se_body, jd_tt);
        }
    }

    (csv, rows)
}

fn build_manifest(version: &str, csv: &str, rows: usize) -> String {
    let mut m = String::new();
    m.push_str("corpus: pheno\n");
    m.push_str(&format!(
        "source: Swiss Ephemeris {version} (Moshier, SEFLG_MOSEPH)\n"
    ));
    m.push_str("generator: tools/se-pheno-reference\n");
    m.push_str(&format!(
        "file: pheno.csv rows={rows} checksum={}\n",
        fnv1a64(csv)
    ));
    m.push_str(&format!("iflag: {IFLAG} (SEFLG_MOSEPH|SEFLG_NOGDEFL)\n"));
    m.push_str(
        "attrs: attr[0]=phase_angle (deg), attr[1]=phase (illuminated fraction), \
         attr[2]=elongation (deg), attr[3]=diameter_deg (DEGREES, not arcsec), \
         attr[4]=magnitude\n",
    );
    m.push_str("bodies: SE 0-9 classical planets (Sun..Pluto)\n");
    m.push_str("times: TT (Terrestrial Time) Julian days\n");
    m.push_str("window: 1900-2100 CE (EPOCHS 8-point grid)\n");
    m
}

struct Config {
    dry_run: bool,
    out_dir: String,
    ephe_dir: String,
}

/// Default ephemeris directory: this tool's own `data/` folder (unused by
/// Moshier, kept for parity with the sibling tools' `--ephe` override),
/// resolved at COMPILE time from the manifest directory so it is correct
/// regardless of the process's current directory.
const DEFAULT_EPHE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");

fn parse_args() -> Config {
    let mut dry_run = false;
    let mut out_dir = ".".to_string();
    let mut ephe_dir = env::var("SE_EPHE_PATH").unwrap_or_else(|_| DEFAULT_EPHE_DIR.to_string());
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
            "dry-run: pheno rows={rows} (expected 80), SE {version} (ephe={})",
            cfg.ephe_dir
        );
        return;
    }

    std::fs::create_dir_all(&cfg.out_dir)
        .unwrap_or_else(|e| panic!("create_dir_all {}: {e}", cfg.out_dir));
    let csv_path = format!("{}/pheno.csv", cfg.out_dir);
    let mf_path = format!("{}/manifest.txt", cfg.out_dir);
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&mf_path, &manifest).unwrap_or_else(|e| panic!("write {mf_path}: {e}"));
    eprintln!("wrote {csv_path} ({rows} rows), {mf_path}; SE {version}");
}
