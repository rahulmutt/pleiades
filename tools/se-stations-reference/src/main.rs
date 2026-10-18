//! Emits a Swiss Ephemeris planetary-station reference corpus to STDOUT as
//! CSV. Swiss Ephemeris has no station finder, so a station is located here
//! by scanning the longitude speed of `swe_calc(jd_tt, body, iflag | SEFLG_SPEED)`
//! on a fixed grid and bisecting each sign change to 1e-7 day (Swiss
//! Ephemeris' "ET" argument is TT).
//!
//! Groups:
//!   - `geo`:  geocentric apparent, tropical, true equinox of date (SE default
//!     flags). Mercury–Pluto over the pleiades-events window less five days at
//!     each end; the true node over 1990–2030.
//!   - `mean`: geocentric geometric place in the mean ecliptic and equinox of
//!     date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT).
//!     Mercury, Mars, Saturn over 1990–2030.
//!   - `sid`:  geocentric apparent, sidereal Lahiri (SEFLG_SIDEREAL after
//!     `swe_set_sid_mode(SE_SIDM_LAHIRI)`). Mercury, Mars, Saturn over 1990–2030.
//!
//! Grid: 0.25 day for the planets (their closest stations are 19 days
//! apart). 0.005 day for the true node, whose speed touches zero about every
//! two weeks and can cross it for only a few hours.
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH), no data files needed.
//!
//! Asteroid mode (`--asteroids --out <dir> [--ephe <dir>]`): `geo` series of
//! Ceres, Pallas, Juno and Vesta (SE ids 17–20) over the full span with the
//! `geo` flags but SEFLG_SWIEPH instead of SEFLG_MOSEPH, from the
//! `seas_18`/`sepl_18`/`semo_18` files pinned in `pins.rs` (verified before
//! anything is written). Writes `<dir>/asteroids.csv` and
//! `<dir>/asteroids-manifest.txt` itself, so no devenv banner reaches them:
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/se-stations-reference/Cargo.toml`, then
//! `tools/se-stations-reference/target/release/se-stations-reference --asteroids --out crates/pleiades-validate/data/stations-corpus`.
//! The ephe directory is `$SE_EPHE_PATH`, else `--ephe`, else this tool's `data/`.
//!
//! True-apogee mode (`--true-apogee --out <dir> [--ephe <dir>]`): the `geo`
//! osculating lunar apogee (SE_OSCU_APOG, id 13) over the full span with
//! SEFLG_SWIEPH and the pinned `sepl_18`/`semo_18` files. A station is a sign
//! change of the central difference `wrap180(lon(t + h) - lon(t - h)) / 2h` of
//! the longitudes (not SE's reported speed, which jumps at isolated instants),
//! on a 0.01-day grid offset 0.00137 day from the span start, h = 0.001 day.
//! A second scan with 1.3 h must agree: a station found by one scan only has
//! to be one of an R/D pair under 3 h apart (an isolated jump) and is dropped
//! and reported; anything else aborts. Writes `true-apogee.csv`,
//! `true-apogee-windows.csv` (graze windows around near-zero speed extrema,
//! with the margin), `true-apogee-extrema.csv` (every one-sided extremum with
//! |cd| < 0.1 deg/day) and the two manifests into `--out`:
//! `tools/se-stations-reference/target/release/se-stations-reference --true-apogee --out crates/pleiades-validate/data/stations-corpus`.
//!
//! Fictitious mode (`--fictitious --out <dir> [--ephe <dir>]`): the same
//! method for the 19 fictitious bodies (SE ids 40-58) in the `mean` frame
//! (the mean-of-date flags above), 0.25-day grid, h = 0.01 day, with
//! `seorbel.txt` pinned beside the planet and Moon files. Writes the
//! `fictitious*` files; run it as above with `--fictitious`.
//!
//! Non-finite speeds: Swiss Ephemeris Moshier returns a NaN true-node speed at
//! isolated instants (e.g. jd_tt 2451544.9). Such a grid sample is skipped (and
//! reported on STDERR) and the bracket widens to the neighbouring finite
//! samples. Everything else non-finite aborts: a non-finite first sample, two
//! consecutive non-finite grid samples, a non-finite bisection midpoint, or a
//! non-finite longitude.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
//!    --manifest-path tools/se-stations-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/stations-corpus/stations.csv`

mod cd;
mod pins;

use std::ffi::CStr;
use std::fmt::Write as _;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_set_ephe_path, swe_set_sid_mode};

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_SWIEPH: c_int = 2;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_SIDEREAL: c_int = 64 * 1024;

const SE_SIDM_LAHIRI: c_int = 1;
const SE_TRUE_NODE: c_int = 11;
const SE_OSCU_APOG: c_int = 13;

const BASE: c_int = SEFLG_MOSEPH | SEFLG_SPEED;
const BASE_SWIEPH: c_int = SEFLG_SWIEPH | SEFLG_SPEED;
const MEAN_OF_DATE: c_int = SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT;

/// (Swiss Ephemeris body id, name as written to the CSV).
const PLANETS: [(c_int, &str); 8] = [
    (2, "Mercury"),
    (3, "Venus"),
    (4, "Mars"),
    (5, "Jupiter"),
    (6, "Saturn"),
    (7, "Uranus"),
    (8, "Neptune"),
    (9, "Pluto"),
];
const ASTEROIDS: [(c_int, &str); 4] = [(17, "Ceres"), (18, "Pallas"), (19, "Juno"), (20, "Vesta")];
const DEFAULT_EPHE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
const SUBSET: [(c_int, &str); 3] = [(2, "Mercury"), (4, "Mars"), (6, "Saturn")];

/// The pleiades-events window (JD 2415020.5–2488069.5) less five days at each
/// end, so neither side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

const PLANET_GRID_DAYS: f64 = 0.25;
const TRUE_NODE_GRID_DAYS: f64 = 0.005;
const BISECT_TOLERANCE_DAYS: f64 = 1e-7;

const TRUE_APOGEE_GRID_DAYS: f64 = 0.01;
const TRUE_APOGEE_STEP_DAYS: f64 = 0.001;
const FICTITIOUS_GRID_DAYS: f64 = 0.25;
const FICTITIOUS_STEP_DAYS: f64 = 0.01;
/// Graze margins of the central-difference speed (deg/day): 3 times the
/// largest engine-minus-SE speed difference at the SE extrema below 0.1
/// deg/day. Provisional values; Task 2 of the plan measures and replaces them.
const TRUE_APOGEE_SPEED_MARGIN_DEG_PER_DAY: f64 = 2e-3;
const FICTITIOUS_SPEED_MARGIN_DEG_PER_DAY: f64 = 1e-4;
const FICTITIOUS: [(c_int, &str); 19] = [
    (40, "Cupido"),
    (41, "Hades"),
    (42, "Zeus"),
    (43, "Kronos"),
    (44, "Apollon"),
    (45, "Admetos"),
    (46, "Vulkanus"),
    (47, "Poseidon"),
    (48, "Transpluto"),
    (49, "Nibiru"),
    (50, "Harrington"),
    (51, "NeptuneLeverrier"),
    (52, "NeptuneAdams"),
    (53, "PlutoLowell"),
    (54, "PlutoPickering"),
    (55, "Vulcan"),
    (56, "WhiteMoon"),
    (57, "Proserpina"),
    (58, "Waldemath"),
];

/// `(longitude_deg in [0, 360), longitude_speed_deg_per_day)`. The speed is
/// returned as is and may be NaN; callers decide how to treat that.
fn state(jd_tt: f64, ipl: c_int, iflag: c_int) -> (f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            ipl,
            iflag,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc(ipl={ipl}, iflag={iflag}) failed at jd_tt={jd_tt}: {msg}");
    }
    if iflag & SEFLG_SWIEPH != 0 && ret & SEFLG_SWIEPH == 0 {
        panic!("swe_calc(ipl={ipl}) fell back from SWIEPH at jd_tt={jd_tt} (ret={ret})");
    }
    assert!(
        xx[0].is_finite(),
        "non-finite SE longitude for ipl={ipl} at jd_tt={jd_tt}"
    );
    (xx[0].rem_euclid(360.0), xx[3])
}

/// Emits one row per sign change of the longitude speed in `[lo, hi]`.
fn scan(
    out: &mut dyn FnMut(String),
    group: &str,
    name: &str,
    ipl: c_int,
    iflag: c_int,
    (lo, hi): (f64, f64),
    grid: f64,
) {
    let speed = |jd: f64| state(jd, ipl, iflag).1;
    let steps = ((hi - lo) / grid).floor() as u64;
    let mut prev_jd = lo;
    let mut prev = speed(lo);
    assert!(
        prev.is_finite(),
        "non-finite speed at scan start {group},{name} jd_tt={lo}"
    );
    let mut skipped_last = false;
    for k in 1..=steps {
        let jd = lo + k as f64 * grid;
        let cur = speed(jd);
        if !cur.is_finite() {
            assert!(
                !skipped_last,
                "two consecutive non-finite speeds at {group},{name} jd_tt={jd}"
            );
            eprintln!("skipped non-finite speed sample: {group},{name},{jd:.7}");
            skipped_last = true;
            continue;
        }
        skipped_last = false;
        if (prev <= 0.0) != (cur <= 0.0) {
            let (mut a, mut b, mut f_a) = (prev_jd, jd, prev);
            while b - a > BISECT_TOLERANCE_DAYS {
                let mid = 0.5 * (a + b);
                let f_mid = speed(mid);
                assert!(
                    f_mid.is_finite(),
                    "non-finite speed at bisection midpoint {group},{name} jd_tt={mid}"
                );
                if (f_a <= 0.0) == (f_mid <= 0.0) {
                    a = mid;
                    f_a = f_mid;
                } else {
                    b = mid;
                }
            }
            let root = 0.5 * (a + b);
            let (lon, _) = state(root, ipl, iflag);
            let kind = if cur > 0.0 { "D" } else { "R" };
            out(format!("{group},{name},{root:.7},{lon:.9},{kind}"));
        }
        prev_jd = jd;
        prev = cur;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--asteroids") {
        asteroids_main(&args);
        return;
    }
    if args.iter().any(|a| a == "--true-apogee") {
        true_apogee_main(&args);
        return;
    }
    if args.iter().any(|a| a == "--fictitious") {
        fictitious_main(&args);
        return;
    }
    let out = &mut |row| println!("{row}");
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), Moshier (SEFLG_MOSEPH, no data files).");
    println!("# A row is a sign change of the longitude speed of swe_calc(jd_tt, body, iflag|SEFLG_SPEED),");
    println!("# bracketed on a 0.25-day grid (0.005 day for TrueNode) and bisected to 1e-7 day. jd_tt is TT.");
    println!("# geo: apparent, tropical, true equinox of date (default flags); planets JD 2415025.5-2488064.5,");
    println!("#   TrueNode JD 2447892.5-2462502.5. mean: SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT,");
    println!(
        "#   JD 2447892.5-2462502.5. sid: SEFLG_SIDEREAL, SE_SIDM_LAHIRI, JD 2447892.5-2462502.5."
    );
    println!(
        "# kind: R = turns retrograde, D = turns direct. lon_deg is the longitude at the station."
    );
    println!("# Swiss Ephemeris returns a NaN true-node speed at isolated instants (e.g. jd_tt 2451544.9); such a grid sample is skipped and the bracket widened.");
    println!("group,body,jd_tt,lon_deg,kind");
    for (ipl, name) in PLANETS {
        scan(out, "geo", name, ipl, BASE, FULL_SPAN, PLANET_GRID_DAYS);
    }
    scan(
        out,
        "geo",
        "TrueNode",
        SE_TRUE_NODE,
        BASE,
        SHORT_SPAN,
        TRUE_NODE_GRID_DAYS,
    );
    for (ipl, name) in SUBSET {
        scan(
            out,
            "mean",
            name,
            ipl,
            BASE | MEAN_OF_DATE,
            SHORT_SPAN,
            PLANET_GRID_DAYS,
        );
    }
    unsafe { swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0) };
    for (ipl, name) in SUBSET {
        scan(
            out,
            "sid",
            name,
            ipl,
            BASE | SEFLG_SIDEREAL,
            SHORT_SPAN,
            PLANET_GRID_DAYS,
        );
    }
}

/// Reads `--out` and `--ephe` (`$SE_EPHE_PATH` wins over `--ephe`), verifies
/// the data files with `verify` (exiting on failure), points Swiss Ephemeris
/// at them and returns the output directory.
fn ephe_setup(args: &[String], mode: &str, verify: fn(&str) -> Result<(), String>) -> String {
    let value_of = |flag: &str| {
        args.iter().position(|a| a == flag).map(|i| {
            args.get(i + 1)
                .unwrap_or_else(|| panic!("{flag} needs a value"))
                .clone()
        })
    };
    let out_dir = value_of("--out").unwrap_or_else(|| panic!("{mode} needs --out <dir>"));
    let ephe_dir = std::env::var("SE_EPHE_PATH")
        .ok()
        .or_else(|| value_of("--ephe"))
        .unwrap_or_else(|| DEFAULT_EPHE_DIR.to_string());
    if let Err(e) = verify(&ephe_dir) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    let ephe = std::ffi::CString::new(ephe_dir).expect("ephe path has NUL");
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };
    out_dir
}

/// Longitude in [0, 360) of `swe_calc(jd_tt, ipl, iflag)`; `state` fails
/// closed on an error, a SWIEPH fallback or a non-finite longitude.
fn longitude(jd_tt: f64, ipl: c_int, iflag: c_int) -> f64 {
    let (lon, _) = state(jd_tt, ipl, iflag);
    lon
}

/// The corpus, windows and extrema texts of one central-difference mode, with
/// their row counts and the artefact pairs dropped per body.
#[derive(Default)]
struct Outputs {
    corpus: String,
    corpus_rows: usize,
    windows: String,
    windows_rows: usize,
    extrema: String,
    dropped: Vec<(String, usize)>,
}

impl Outputs {
    fn corpus(&mut self, row: String) {
        self.corpus_rows += 1;
        writeln!(self.corpus, "{row}").unwrap();
    }

    fn window(&mut self, row: String) {
        self.windows_rows += 1;
        writeln!(self.windows, "{row}").unwrap();
    }

    fn extremum(&mut self, row: String) {
        writeln!(self.extrema, "{row}").unwrap();
    }

    /// "Name n, ..." over the bodies with a dropped pair, or "none".
    fn dropped_summary(&self) -> String {
        let parts: Vec<String> = self
            .dropped
            .iter()
            .filter(|(_, n)| *n > 0)
            .map(|(name, n)| format!("{name} {n}"))
            .collect();
        if parts.is_empty() {
            "none".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Writes the five files of `stem`: corpus, windows and extrema CSVs with
    /// their headers, and the corpus and windows manifests.
    fn write(
        &self,
        out_dir: &str,
        stem: &str,
        corpus_head: &str,
        windows_head: &str,
        extrema_head: &str,
    ) {
        let corpus = format!(
            "{corpus_head}group,body,jd_tt,lon_deg,kind\n{}",
            self.corpus
        );
        let windows = format!(
            "{windows_head}group,body,turn_jd_tt,miss_deg_per_day,lo_jd_tt,hi_jd_tt,se_stations,lo_side,hi_side,margin_deg_per_day\n{}",
            self.windows
        );
        let extrema = format!(
            "{extrema_head}group,body,jd_tt,cd_deg_per_day\n{}",
            self.extrema
        );
        let corpus_manifest = format!(
            "slice stations-{stem} file={stem}.csv role=stations rows={} checksum={}\n",
            self.corpus_rows,
            pins::fnv1a64(&corpus)
        );
        let windows_manifest = format!(
            "slice stations-{stem}-windows file={stem}-windows.csv role=windows rows={} checksum={}\n",
            self.windows_rows,
            pins::fnv1a64(&windows)
        );
        std::fs::create_dir_all(out_dir).unwrap_or_else(|e| panic!("create {out_dir}: {e}"));
        for (file, text) in [
            (format!("{stem}.csv"), &corpus),
            (format!("{stem}-manifest.txt"), &corpus_manifest),
            (format!("{stem}-windows.csv"), &windows),
            (format!("{stem}-windows-manifest.txt"), &windows_manifest),
            (format!("{stem}-extrema.csv"), &extrema),
        ] {
            let path = format!("{out_dir}/{file}");
            std::fs::write(&path, text).unwrap_or_else(|e| panic!("write {path}: {e}"));
        }
        eprintln!(
            "wrote {stem}* to {out_dir} ({} stations, {} windows)",
            self.corpus_rows, self.windows_rows
        );
    }
}

/// The central-difference stations of one body: both scans, artefact
/// rejection, windows and extrema. Appends corpus, windows and extrema rows.
#[allow(clippy::too_many_arguments)]
fn cd_body(
    out: &mut Outputs,
    group: &str,
    name: &str,
    ipl: c_int,
    iflag: c_int,
    grid: f64,
    h: f64,
    margin: f64,
) {
    let lon = |jd: f64| longitude(jd, ipl, iflag);
    let primary = cd::scan(&lon, FULL_SPAN, grid, h);
    let check = cd::scan(&lon, FULL_SPAN, grid, cd::CHECK_STEP_RATIO * h);
    let (kept, dropped) = cd::reject_artefacts(&primary.stations, &check.stations, h)
        .unwrap_or_else(|e| panic!("{group},{name}: {e}"));
    for [a, b] in &dropped {
        eprintln!(
            "{group},{name}: dropped artefact pair {:.7} {} / {:.7} {}",
            a.jd,
            a.kind.letter(),
            b.jd,
            b.kind.letter()
        );
    }
    let speed = |jd: f64| cd::central_difference(&lon, jd, h);
    let windows = cd::graze_windows(&speed, &primary.extrema, &kept, FULL_SPAN, grid, margin)
        .unwrap_or_else(|e| panic!("{group},{name}: {e}"));
    for s in &kept {
        out.corpus(format!(
            "{group},{name},{:.7},{:.9},{}",
            s.jd,
            lon(s.jd),
            s.kind.letter()
        ));
    }
    for w in &windows {
        out.window(format!(
            "{group},{name},{:.7},{:.3e},{:.7},{:.7},{},{},{},{margin:.3e}",
            w.turn_jd, w.miss_deg_per_day, w.lo, w.hi, w.se_stations, w.lo_side, w.hi_side
        ));
    }
    for &(jd, v) in primary
        .extrema
        .iter()
        .filter(|(_, v)| v.abs() < cd::EXTREMUM_RECORD_DEG_PER_DAY)
    {
        out.extremum(format!("{group},{name},{jd:.7},{v:+.6e}"));
    }
    out.dropped.push((name.to_string(), dropped.len()));
    eprintln!(
        "{group},{name}: {} stations, {} windows, {} artefact pairs dropped",
        kept.len(),
        windows.len(),
        dropped.len()
    );
}

/// Headers shared by the two central-difference modes.
fn cd_headers(
    out: &Outputs,
    ephe_note: &str,
    grid: f64,
    h: f64,
    margin: f64,
    group_note: &str,
    margin_basis: &str,
) -> [String; 3] {
    let corpus = format!(
        "# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with sepl_18/semo_18 (SHA-256 pinned in tools/se-stations-reference/src/pins.rs){ephe_note}.\n\
         # A row is a sign change of the central difference wrap180(lon(t + h) - lon(t - h)) / 2h of swe_calc(jd_tt, body, iflag) longitudes,\n\
         # on a {grid}-day grid offset 0.00137 day from JD 2415025.5, h = {h} day, bisected to 1e-7 day. jd_tt is TT. Not SE's reported speed:\n\
         # SE's osculating-apogee speed jumps at isolated instants and its Proserpina speed is inconsistent with its own positions (issue #167 (d)).\n\
         # Each body is also scanned with 1.3 h; a station found by one scan only must be one of an R/D pair under 3 h apart (an isolated jump), and is dropped.\n\
         # Dropped artefact pairs: {}.\n\
         # {group_note}; JD 2415025.5-2488064.5.\n\
         # kind: R = turns retrograde, D = turns direct. lon_deg is the longitude at the station.\n",
        out.dropped_summary()
    );
    let windows = format!(
        "# Graze windows of the central difference cd (h = {h} day, {grid}-day grid) around every one-sided extremum of |cd| below the margin.\n\
         # {group_note}.\n\
         # A window runs from the first grid instant on each side of the extremum at which |cd| >= 2 x margin; overlapping windows merge and keep the closer miss.\n\
         # se_stations counts the stations of the corpus inside [lo, hi]; lo_side and hi_side are the signs of cd at the edges.\n\
         # Parity: an odd count needs opposite sides, an even count the same side; the gate is strict (no tolerance) when miss >= margin / 2.\n\
         # Margin basis: {margin_basis}\n\
         # margin_deg_per_day={margin:e}\n"
    );
    let extrema = format!(
        "# Every sampled extremum of the central difference cd (h = {h} day, {grid}-day grid) that stays on one side of zero with |cd| < {} deg/day.\n\
         # {group_note}.\n",
        cd::EXTREMUM_RECORD_DEG_PER_DAY
    );
    [corpus, windows, extrema]
}

fn true_apogee_main(args: &[String]) {
    let out_dir = ephe_setup(args, "--true-apogee", pins::verify_swieph_files);
    let mut out = Outputs::default();
    cd_body(
        &mut out,
        "geo",
        "TrueApogee",
        SE_OSCU_APOG,
        SEFLG_SWIEPH,
        TRUE_APOGEE_GRID_DAYS,
        TRUE_APOGEE_STEP_DAYS,
        TRUE_APOGEE_SPEED_MARGIN_DEG_PER_DAY,
    );
    let [corpus, windows, extrema] = cd_headers(
        &out,
        "",
        TRUE_APOGEE_GRID_DAYS,
        TRUE_APOGEE_STEP_DAYS,
        TRUE_APOGEE_SPEED_MARGIN_DEG_PER_DAY,
        "geo: osculating lunar apogee (SE_OSCU_APOG), apparent, tropical, true equinox of date (default flags)",
        "3 times the largest engine-minus-SE speed difference at the SE extrema below 0.1 deg/day (provisional until measured).",
    );
    out.write(&out_dir, "true-apogee", &corpus, &windows, &extrema);
}

fn fictitious_main(args: &[String]) {
    let out_dir = ephe_setup(args, "--fictitious", pins::verify_fictitious_files);
    let mut out = Outputs::default();
    for (ipl, name) in FICTITIOUS {
        cd_body(
            &mut out,
            "mean",
            name,
            ipl,
            SEFLG_SWIEPH | MEAN_OF_DATE,
            FICTITIOUS_GRID_DAYS,
            FICTITIOUS_STEP_DAYS,
            FICTITIOUS_SPEED_MARGIN_DEG_PER_DAY,
        );
    }
    let [corpus, windows, extrema] = cd_headers(
        &out,
        " and seorbel.txt",
        FICTITIOUS_GRID_DAYS,
        FICTITIOUS_STEP_DAYS,
        FICTITIOUS_SPEED_MARGIN_DEG_PER_DAY,
        "mean: geocentric geometric place in the mean ecliptic and equinox of date (SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT), Cupido..Waldemath (SE ids 40-58)",
        "3 times the largest engine-minus-SE speed difference at the SE extrema below 0.1 deg/day (provisional until measured).",
    );
    out.write(&out_dir, "fictitious", &corpus, &windows, &extrema);
}

fn asteroids_main(args: &[String]) {
    let out_dir = ephe_setup(args, "--asteroids", pins::verify_swieph_files);

    let mut csv = String::new();
    csv.push_str("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with seas_18/sepl_18/semo_18 (SHA-256 pinned in tools/se-stations-reference/src/pins.rs).\n");
    csv.push_str("# A row is a sign change of the longitude speed of swe_calc(jd_tt, body, iflag|SEFLG_SPEED),\n");
    csv.push_str("# bracketed on a 0.25-day grid and bisected to 1e-7 day. jd_tt is TT.\n");
    csv.push_str("# geo: apparent, tropical, true equinox of date (default flags); Ceres, Pallas, Juno, Vesta JD 2415025.5-2488064.5.\n");
    csv.push_str("# kind: R = turns retrograde, D = turns direct. lon_deg is the longitude at the station.\n");
    csv.push_str("group,body,jd_tt,lon_deg,kind\n");
    let mut rows = 0usize;
    for (ipl, name) in ASTEROIDS {
        scan(
            &mut |row| {
                rows += 1;
                writeln!(csv, "{row}").unwrap();
            },
            "geo",
            name,
            ipl,
            BASE_SWIEPH,
            FULL_SPAN,
            PLANET_GRID_DAYS,
        );
    }
    let manifest = format!(
        "slice stations-asteroids file=asteroids.csv role=stations rows={rows} checksum={}\n",
        pins::fnv1a64(&csv)
    );
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| panic!("create {out_dir}: {e}"));
    let csv_path = format!("{out_dir}/asteroids.csv");
    let manifest_path = format!("{out_dir}/asteroids-manifest.txt");
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&manifest_path, &manifest)
        .unwrap_or_else(|e| panic!("write {manifest_path}: {e}"));
    eprintln!("wrote {csv_path} ({rows} rows) and {manifest_path}");
}
