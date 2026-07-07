//! Emits a Swiss Ephemeris **planetary nodes and apsides** reference corpus
//! for the upcoming `validate-nod-aps` gate (SP-4: planetary nodes and
//! apsides, a `swe_nod_aps` analogue).
//!
//! One CSV plus a manifest:
//!   * `nod-aps.csv` — mean, osculating, and osculating-barycentric
//!     ascending/descending nodes and perihelion/aphelion points (and their
//!     focal-point variants) for the classical planets (Sun..Pluto, with the
//!     mean set dropping Pluto, whose mean elements SE does not tabulate),
//!     computed via `swe_nod_aps`. 184 data rows total (72 mean + 6 mean
//!     focal-point + 80 osculating + 20 barycentric + 6 osculating
//!     focal-point).
//!   * `manifest.txt` — row count + `fnv1a64` checksum of the CSV bytes + the
//!     SE version string + the flags/method conventions below.
//!
//! CONTROLLER-MANDATED SCOPE (binding, decided after
//! `.superpowers/sdd/task-7-brief.md` was written):
//!   * Small bodies (Chiron/Pholus/Ceres/Pallas/Juno/Vesta, SE 15-20) are
//!     DROPPED: the offline backend chain cannot serve them.
//!   * Fictitious bodies (SE 40-58) are DROPPED: this SE build's
//!     `swe_nod_aps` rejects them unconditionally ("nodes/apsides for planet
//!     NN are not implemented") — the enabling condition is commented out in
//!     upstream swecl.c, so no reference is possible by design.
//!   * The brief's row arithmetic ("80/88/325") was wrong even before these
//!     cuts; the enumerations in `build_csv` below are authoritative: 184.
//!
//! Flags (SE default output minus gravitational deflection; see the SP-4 plan
//! §R3): `SEFLG_MOSEPH | SEFLG_SPEED | SEFLG_NOGDEFL` = `4 | 256 | 512` for
//! every category EXCEPT method-4 (barycentric) rows.
//!
//! **Method-4 rows are SWIEPH-based (DE431-derived), not MOSEPH**: for bodies
//! beyond ~6 AU heliocentric distance, `swe_nod_aps` with
//! `SE_NODBIT_OSCU_BAR` internally requires `SEFLG_BARYCTR` positions, and
//! Swiss Ephemeris hard-rejects `SEFLG_BARYCTR|SEFLG_MOSEPH` ("barycentric
//! Moshier positions are not supported" — Moshier has no barycenter). So
//! method-4 rows use `SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_NOGDEFL` =
//! `2 | 256 | 512` and need the compressed Swiss Ephemeris data files
//! `sepl_18.se1` (planets, 1800-2400 CE) and `semo_18.se1` (Moon) in the ephe
//! dir. Those files are NOT committed (see `.gitignore`); their SHA-256
//! digests are pinned below and verified fail-closed before any method-4 row
//! is emitted. Download at generation time:
//!   curl -fLo data/sepl_18.se1 https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/sepl_18.se1
//!   curl -fLo data/semo_18.se1 https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/semo_18.se1
//!
//! Methods (`swe_nod_aps` bit flags, see `swephexp.h`):
//!   `SE_NODBIT_MEAN` = 1, `SE_NODBIT_OSCU` = 2, `SE_NODBIT_OSCU_BAR` = 4,
//!   `SE_NODBIT_FOPOINT` = 256 (aphelion point replaced by the orbit's focal
//!   point; OR'd onto the base method).
//!
//! Fail-closed: every `swe_nod_aps` return code / `serr` is checked; any SE
//! error aborts the run, the two SWIEPH data files are existence- and
//! SHA-256-verified before the barycentric category, and every output
//! component (6 doubles — lon/lat/dist/speed-lon/speed-lat/speed-dist — for
//! each of the 4 points asc/dsc/peri/apo, 24 values total) is asserted
//! finite before the row is appended. No partial rows are ever emitted.
//!
//! Usage:
//!   se-nodaps-reference --dry-run          # print CSV+manifest, no writes
//!   se-nodaps-reference --out <dir>        # write the two files
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::env;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_nod_aps, swe_set_ephe_path, swe_version};

// SE default output minus gravitational deflection (see plan §R3):
// SEFLG_MOSEPH=4, SEFLG_SPEED=256, SEFLG_NOGDEFL=512.
const IFLAG_MOSEPH: c_int = 4 | 256 | 512;
// Method-4 (barycentric) rows only: SEFLG_SWIEPH=2 instead of MOSEPH, because
// Moshier cannot produce the barycentric positions SE_NODBIT_OSCU_BAR needs
// for bodies beyond ~6 AU (see module doc).
const IFLAG_SWIEPH: c_int = 2 | 256 | 512;

// swe_nod_aps `method` bit flags (see swephexp.h).
const SE_NODBIT_MEAN: c_int = 1;
const SE_NODBIT_OSCU: c_int = 2;
const SE_NODBIT_OSCU_BAR: c_int = 4;
const SE_NODBIT_FOPOINT: c_int = 256;

// Pinned SHA-256 digests of the SWIEPH data files needed by the method-4
// (barycentric) rows. Computed with `sha256sum` over the files fetched from
// https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/ at
// generation time. The files themselves are gitignored; these constants are
// the committed provenance.
const SEPL_18_SHA256: &str = "ca1393ceab3a44fbc895887cf789c68819ae6a1cbc9b22225872dbe4ccd99a66";
const SEMO_18_SHA256: &str = "1ca07bd67c24374d77226180c20a4f9996cba013697894810518e7eb582ca4f7";

// Epochs spanning 1900-2100, >= 2 days inside the packaged window.
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
const EPOCHS_SHORT: [f64; 5] = [
    2_415_100.5,
    2_433_282.5,
    2_451_545.0,
    2_466_154.5,
    2_488_021.5,
];

/// (label, SE body number), SE 0-9 classical planets. Small bodies (SE 15-20)
/// and fictitious bodies (SE 40-58) are out of scope (see module doc).
const PLANETS: [(&str, c_int); 10] = [
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

/// Minimal embedded SHA-256 (FIPS 180-4), public-domain-style, no deps.
/// Used only to pin the two SWIEPH data files; unit-tested below against the
/// canonical b"abc" digest.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (hi, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *hi = hi.wrapping_add(v);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

/// Fail-closed guard for the method-4 (barycentric) rows: both SWIEPH data
/// files must exist in `ephe_dir` and match their pinned SHA-256 digests.
fn verify_swieph_files(ephe_dir: &str) {
    for (name, want) in [
        ("sepl_18.se1", SEPL_18_SHA256),
        ("semo_18.se1", SEMO_18_SHA256),
    ] {
        let path = format!("{ephe_dir}/{name}");
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "cannot read {path}: {e}\n\
                 The barycentric (method=4) rows need the SWIEPH data files. Download:\n\
                 curl -fLo {ephe_dir}/{name} \
                 https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/{name}"
            )
        });
        let got = sha256_hex(&bytes);
        assert_eq!(
            got, want,
            "SHA-256 mismatch for {path}: got {got}, pinned {want}. \
             Re-download from https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/{name}"
        );
    }
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

/// Compute one `swe_nod_aps` row and append it to `csv`. Fail-closed: any SE
/// error or non-finite output panics before anything is appended — no
/// partial rows.
#[allow(clippy::too_many_arguments)]
fn emit_row(
    csv: &mut String,
    rows: &mut usize,
    label: &str,
    se_body: c_int,
    iflag: c_int,
    method: c_int,
    fopoint: bool,
    jd_tt: f64,
) {
    let mut xnasc = [0.0_f64; 6];
    let mut xndsc = [0.0_f64; 6];
    let mut xperi = [0.0_f64; 6];
    let mut xaphe = [0.0_f64; 6];
    let mut serr = [0 as c_char; 256];
    let m = method | if fopoint { SE_NODBIT_FOPOINT } else { 0 };
    let ret = unsafe {
        swe_nod_aps(
            jd_tt,
            se_body,
            iflag,
            m,
            xnasc.as_mut_ptr(),
            xndsc.as_mut_ptr(),
            xperi.as_mut_ptr(),
            xaphe.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };
    if ret < 0 {
        panic!(
            "swe_nod_aps(ipl={se_body} label={label} iflag={iflag} method={m}) at jd_tt={jd_tt}: {}",
            serr_string(&serr)
        );
    }
    for v in xnasc.iter().chain(&xndsc).chain(&xperi).chain(&xaphe) {
        assert!(
            v.is_finite(),
            "non-finite swe_nod_aps output for {label} (se={se_body}) method={m} at jd_tt={jd_tt}"
        );
    }
    csv.push_str(&format!(
        "{label},{se_body},{method},{},{jd_tt:.9}",
        u8::from(fopoint)
    ));
    for pt in [&xnasc, &xndsc, &xperi, &xaphe] {
        for v in pt.iter() {
            csv.push_str(&format!(",{v:.9}"));
        }
    }
    csv.push('\n');
    *rows += 1;
}

fn build_csv(version: &str, ephe_dir: &str) -> (String, usize) {
    let mut csv = String::new();
    let mut rows = 0usize;
    csv.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), swe_nod_aps, iflag={IFLAG_MOSEPH} \
         (SEFLG_MOSEPH|SEFLG_SPEED|SEFLG_NOGDEFL); method=4 rows only use iflag={IFLAG_SWIEPH} \
         (SEFLG_SWIEPH|SEFLG_SPEED|SEFLG_NOGDEFL, DE431-derived sepl_18.se1/semo_18.se1, SHA-256 \
         pinned in the generator) because Moshier cannot produce barycentric positions.\n"
    ));
    csv.push_str(
        "# Nodes/apsides for the classical planets (SE 0-9; mean set drops Pluto). method bit \
         flags: 1=mean, 2=osculating, 4=osculating-barycentric; fopoint=1 means \
         SE_NODBIT_FOPOINT was OR'd onto method (aphelion point replaced by the orbit's focal \
         point). Small bodies (SE 15-20) and fictitious bodies (SE 40-58) are OUT OF SCOPE \
         (offline backend chain cannot serve small bodies; this SE build's swe_nod_aps does not \
         implement fictitious bodies). Times are TT (Terrestrial Time) Julian days.\n",
    );
    csv.push_str(
        "label,se_body,method,fopoint,jd_tt,\
         asc_lon,asc_lat,asc_dist,asc_dlon,asc_dlat,asc_ddist,\
         dsc_lon,dsc_lat,dsc_dist,dsc_dlon,dsc_dlat,dsc_ddist,\
         peri_lon,peri_lat,peri_dist,peri_dlon,peri_dlat,peri_ddist,\
         apo_lon,apo_lat,apo_dist,apo_dlon,apo_dlat,apo_ddist\n",
    );

    // 1. Mean rows: (Sun, Moon, Mercury..Neptune) x EPOCHS x method=mean,
    //    fopoint=false -> 9 x 8 = 72 rows. Pluto is dropped: SE does not
    //    tabulate mean elements for it.
    for &(label, se_body) in &PLANETS[..9] {
        for &jd_tt in &EPOCHS {
            emit_row(
                &mut csv,
                &mut rows,
                label,
                se_body,
                IFLAG_MOSEPH,
                SE_NODBIT_MEAN,
                false,
                jd_tt,
            );
        }
    }

    // 2. Mean focal-point rows: (Moon, Mercury) x EPOCHS_SHORT[..3] x
    //    method=mean, fopoint=true -> 2 x 3 = 6 rows.
    for &(label, se_body) in &[PLANETS[1], PLANETS[2]] {
        for &jd_tt in &EPOCHS_SHORT[..3] {
            emit_row(
                &mut csv,
                &mut rows,
                label,
                se_body,
                IFLAG_MOSEPH,
                SE_NODBIT_MEAN,
                true,
                jd_tt,
            );
        }
    }

    // 3. Osculating rows: (Sun, Moon, Mercury..Pluto) x EPOCHS x
    //    method=osculating, fopoint=false -> 10 x 8 = 80 rows.
    for &(label, se_body) in &PLANETS {
        for &jd_tt in &EPOCHS {
            emit_row(
                &mut csv,
                &mut rows,
                label,
                se_body,
                IFLAG_MOSEPH,
                SE_NODBIT_OSCU,
                false,
                jd_tt,
            );
        }
    }

    // 4. Barycentric rows: (Jupiter, Saturn, Uranus, Neptune, Pluto) x
    //    EPOCHS_SHORT[..4] x method=osculating-barycentric -> 5 x 4 = 20 rows
    //    (Jupiter pins the <= 6 AU heliocentric fallback). SWIEPH-based:
    //    verify the pinned data files fail-closed before emitting anything.
    verify_swieph_files(ephe_dir);
    for &(label, se_body) in &PLANETS[5..10] {
        for &jd_tt in &EPOCHS_SHORT[..4] {
            emit_row(
                &mut csv,
                &mut rows,
                label,
                se_body,
                IFLAG_SWIEPH,
                SE_NODBIT_OSCU_BAR,
                false,
                jd_tt,
            );
        }
    }

    // 5. Osculating focal-point rows: (Mars, Neptune) x EPOCHS_SHORT[..3] x
    //    method=osculating, fopoint=true -> 2 x 3 = 6 rows.
    for &(label, se_body) in &[PLANETS[4], PLANETS[8]] {
        for &jd_tt in &EPOCHS_SHORT[..3] {
            emit_row(
                &mut csv,
                &mut rows,
                label,
                se_body,
                IFLAG_MOSEPH,
                SE_NODBIT_OSCU,
                true,
                jd_tt,
            );
        }
    }

    (csv, rows)
}

fn build_manifest(version: &str, csv: &str, rows: usize) -> String {
    let mut m = String::new();
    m.push_str("corpus: nod-aps\n");
    m.push_str(&format!(
        "source: Swiss Ephemeris {version} (Moshier, SEFLG_MOSEPH; method=4 rows SWIEPH/DE431-derived)\n"
    ));
    m.push_str("generator: tools/se-nodaps-reference\n");
    m.push_str(&format!(
        "file: nod-aps.csv rows={rows} checksum={}\n",
        fnv1a64(csv)
    ));
    m.push_str(&format!(
        "iflag: {IFLAG_MOSEPH} (SEFLG_MOSEPH|SEFLG_SPEED|SEFLG_NOGDEFL); method=4 rows: \
         {IFLAG_SWIEPH} (SEFLG_SWIEPH|SEFLG_SPEED|SEFLG_NOGDEFL, sepl_18.se1/semo_18.se1 \
         SHA-256-pinned in the generator)\n"
    ));
    m.push_str(
        "methods: 1=SE_NODBIT_MEAN, 2=SE_NODBIT_OSCU, 4=SE_NODBIT_OSCU_BAR; \
         fopoint OR's SE_NODBIT_FOPOINT (256) onto method\n",
    );
    m.push_str(
        "bodies: SE 0-9 classical planets (mean set drops Pluto, SE 9). Small bodies (SE \
         15-20) and fictitious bodies (SE 40-58) are OUT OF SCOPE (controller-mandated scope \
         cut; this SE build's swe_nod_aps does not implement fictitious bodies at all — see \
         src/main.rs doc header).\n",
    );
    m.push_str("times: TT (Terrestrial Time) Julian days\n");
    m.push_str("window: 1900-2100 CE (EPOCHS 8-point / EPOCHS_SHORT 5-point grids)\n");
    m
}

struct Config {
    dry_run: bool,
    out_dir: String,
    ephe_dir: String,
}

/// Default ephemeris directory: this tool's own `data/` folder (containing the
/// gitignored SWIEPH files needed by the method-4 rows), resolved at COMPILE
/// time from the manifest directory so it is correct regardless of the
/// process's current directory.
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
    let (csv, rows) = build_csv(&version, &cfg.ephe_dir);
    let manifest = build_manifest(&version, &csv, rows);

    if cfg.dry_run {
        print!("{csv}");
        println!("# ---- manifest.txt ----");
        print!("{manifest}");
        eprintln!(
            "dry-run: nod-aps rows={rows} (expected 184), SE {version} (ephe={})",
            cfg.ephe_dir
        );
        return;
    }

    std::fs::create_dir_all(&cfg.out_dir)
        .unwrap_or_else(|e| panic!("create_dir_all {}: {e}", cfg.out_dir));
    let csv_path = format!("{}/nod-aps.csv", cfg.out_dir);
    let mf_path = format!("{}/manifest.txt", cfg.out_dir);
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&mf_path, &manifest).unwrap_or_else(|e| panic!("write {mf_path}: {e}"));
    eprintln!("wrote {csv_path} ({rows} rows), {mf_path}; SE {version}");
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn sha256_abc_matches_fips_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_empty_matches_fips_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
