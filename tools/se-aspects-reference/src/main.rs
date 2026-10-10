//! Emits a Swiss Ephemeris exact-aspect reference corpus to STDOUT as CSV.
//! Swiss Ephemeris has no aspect finder, so an exact aspect is located here
//! from `swe_calc(jd_tt, body, iflag | SEFLG_SPEED)` for the two bodies: the
//! signed separation `wrap180(lon(first) − lon(second))` is scanned on a
//! fixed grid, each grid step is split at the zero of the relative longitude
//! speed (the separation's turning point), and each sign change of
//! `separation − level` is bisected to 1e-7 day. The levels of an angle are
//! `+angle` and `−angle` (one level for 0 and 180). Swiss Ephemeris' "ET"
//! argument is TT.
//!
//! Groups:
//!   - `geo`:   geocentric apparent, tropical, true equinox of date (SE
//!     default flags). Seven planet pairs over the pleiades-events window
//!     less five days at each end; Sun–Moon over 1990–2030.
//!   - `mean`:  geocentric geometric place in the mean ecliptic and equinox
//!     of date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT).
//!     Mercury–Venus and Mars–Saturn over 1990–2030.
//!   - `helio`: geometric heliocentric place (SEFLG_HELCTR | SEFLG_TRUEPOS),
//!     the place `se-helio-reference` uses. Mars–Jupiter over the full span.
//!
//! Grid: 0.05 day; 0.01 day for a pair with the Moon.
//!
//! Grazes: the corpus must hold no model-dependent event. If any turning
//! point of a pair's separation comes within 30 arcseconds of one of its
//! levels, on either side, the tool panics and names the pair, angle and
//! instant, and no corpus is written. Counts per pair and angle go to STDERR.
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH), no data files needed.
//!
//! Asteroid mode (`--asteroids --out <dir> [--ephe <dir>]`): `geo` aspects of
//! the Sun with Ceres, Pallas, Juno and Vesta (SE ids 17-20) over the full
//! span, and of the Moon with Ceres over 1990-2030, with the `geo` flags but
//! SEFLG_SWIEPH instead of SEFLG_MOSEPH, from the `seas_18`/`sepl_18`/`semo_18`
//! files pinned in `pins.rs` (verified before anything is written). The same
//! 30-arcsecond graze refusal applies. Writes `<dir>/asteroids.csv` and
//! `<dir>/asteroids-manifest.txt` itself, so no devenv banner reaches them:
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/se-aspects-reference/Cargo.toml`, then
//! `tools/se-aspects-reference/target/release/se-aspects-reference --asteroids --out crates/pleiades-validate/data/aspects-corpus`.
//! The ephe directory is `$SE_EPHE_PATH`, else `--ephe`, else this tool's `data/`.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
//!    --manifest-path tools/se-aspects-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/aspects-corpus/aspects.csv`

mod pins;

use std::ffi::CStr;
use std::fmt::Write as _;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_set_ephe_path};

const SEFLG_SWIEPH: c_int = 2;
const SEFLG_MOSEPH: c_int = 4;
const SEFLG_HELCTR: c_int = 8;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration

const GEO: c_int = SEFLG_MOSEPH | SEFLG_SPEED;
const MEAN: c_int = GEO | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT;
const HELIO: c_int = GEO | SEFLG_HELCTR | SEFLG_TRUEPOS;
/// `GEO` with the SWIEPH data files in place of Moshier.
const GEO_SWIEPH: c_int = (GEO & !SEFLG_MOSEPH) | SEFLG_SWIEPH;
const DEFAULT_EPHE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");

/// (Swiss Ephemeris body id, name as written to the CSV).
type Body = (c_int, &'static str);
const SUN: Body = (0, "Sun");
const MOON: Body = (1, "Moon");
const MERCURY: Body = (2, "Mercury");
const VENUS: Body = (3, "Venus");
const MARS: Body = (4, "Mars");
const JUPITER: Body = (5, "Jupiter");
const SATURN: Body = (6, "Saturn");
const PLUTO: Body = (9, "Pluto");

/// The pleiades-events window (JD 2415020.5–2488069.5) less five days at each
/// end, so neither side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

const GRID_DAYS: f64 = 0.05;
const MOON_GRID_DAYS: f64 = 0.01;
const BISECT_TOLERANCE_DAYS: f64 = 1e-7;
/// 30 arcseconds: several times the largest difference between the Swiss
/// Ephemeris (Moshier) and pleiades separations.
const GRAZE_MARGIN_DEG: f64 = 30.0 / 3600.0;
const ANGLES: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

/// `(longitude_deg in [0, 360), longitude_speed_deg_per_day)`.
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
        xx[0].is_finite() && xx[3].is_finite(),
        "non-finite SE longitude or speed for ipl={ipl} at jd_tt={jd_tt}"
    );
    (xx[0].rem_euclid(360.0), xx[3])
}

fn wrap180(d: f64) -> f64 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// The pair at one instant.
#[derive(Clone, Copy)]
struct Sample {
    jd: f64,
    /// `wrap180(first_lon − second_lon)`.
    separation: f64,
    /// Longitude speed of the first body less that of the second.
    rel_speed: f64,
    first_lon: f64,
    second_lon: f64,
}

fn sample(jd: f64, first: Body, second: Body, iflag: c_int) -> Sample {
    let (first_lon, first_speed) = state(jd, first.0, iflag);
    let (second_lon, second_speed) = state(jd, second.0, iflag);
    Sample {
        jd,
        separation: wrap180(first_lon - second_lon),
        rel_speed: first_speed - second_speed,
        first_lon,
        second_lon,
    }
}

/// The values of the signed separation at which an angle is exact.
fn levels(angle: f64) -> Vec<f64> {
    if angle == 0.0 || angle == 180.0 {
        vec![angle]
    } else {
        vec![angle, -angle]
    }
}

/// Bisects a sign change of `f` over `[a, b]` and returns the midpoint of
/// the final bracket.
fn bisect(f: impl Fn(f64) -> f64, mut a: f64, mut f_a: f64, mut b: f64) -> f64 {
    while b - a > BISECT_TOLERANCE_DAYS {
        let mid = 0.5 * (a + b);
        let f_mid = f(mid);
        if (f_a <= 0.0) == (f_mid <= 0.0) {
            a = mid;
            f_a = f_mid;
        } else {
            b = mid;
        }
    }
    0.5 * (a + b)
}

/// Hands every exact aspect of the pair in `[lo, hi]`, for each angle, to `out`
/// as a CSV row.
fn scan(
    out: &mut dyn FnMut(String),
    group: &str,
    first: Body,
    second: Body,
    iflag: c_int,
    (lo, hi): (f64, f64),
    grid: f64,
) {
    let at = |jd: f64| sample(jd, first, second, iflag);
    let mut events: Vec<Vec<Sample>> = vec![Vec::new(); ANGLES.len()];
    let steps = ((hi - lo) / grid).floor() as u64;
    let mut prev = at(lo);
    for k in 1..=steps {
        let cur = at(lo + k as f64 * grid);
        // Split the step at the separation's turning point, so that each
        // bracket is monotone and holds at most one event per level.
        let mut points = vec![prev];
        if (prev.rel_speed <= 0.0) != (cur.rel_speed <= 0.0) {
            let turn = at(bisect(
                |jd| at(jd).rel_speed,
                prev.jd,
                prev.rel_speed,
                cur.jd,
            ));
            for angle in ANGLES {
                for level in levels(angle) {
                    let miss = wrap180(turn.separation - level).abs();
                    assert!(
                        miss >= GRAZE_MARGIN_DEG,
                        "graze: {group},{},{} turns {:.2} arcsec from the {angle} degree level \
                         at jd_tt={:.5}; an event here would depend on the ephemeris",
                        first.1,
                        second.1,
                        miss * 3600.0,
                        turn.jd
                    );
                }
            }
            points.push(turn);
        }
        points.push(cur);
        for bracket in points.windows(2) {
            let (a, b) = (bracket[0], bracket[1]);
            for (index, angle) in ANGLES.iter().enumerate() {
                for level in levels(*angle) {
                    let f_a = wrap180(a.separation - level);
                    let f_b = wrap180(b.separation - level);
                    // The second test rejects the jump across the +-180 seam.
                    if (f_a <= 0.0) != (f_b <= 0.0) && (f_a - f_b).abs() < 180.0 {
                        let root = bisect(|jd| wrap180(at(jd).separation - level), a.jd, f_a, b.jd);
                        events[index].push(at(root));
                    }
                }
            }
        }
        prev = cur;
    }
    for (index, angle) in ANGLES.iter().enumerate() {
        events[index].sort_by(|x, y| x.jd.total_cmp(&y.jd));
        eprintln!(
            "{group},{},{},{angle:.0}: {} events",
            first.1,
            second.1,
            events[index].len()
        );
        for event in &events[index] {
            out(format!(
                "{group},{},{},{angle:.0},{:.7},{:.9},{:.9},{:.9}",
                first.1, second.1, event.jd, event.first_lon, event.second_lon, event.rel_speed
            ));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--asteroids") {
        asteroids_main(&args);
        return;
    }
    let out = &mut |row: String| println!("{row}");
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), Moshier (SEFLG_MOSEPH, no data files).");
    println!("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)");
    println!("# equals +angle or -angle, scanned on a 0.05-day grid (0.01 day with the Moon), each step split at the zero");
    println!("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.");
    println!("# geo: apparent, tropical, true equinox of date (default flags); JD 2415025.5-2488064.5, Sun-Moon JD 2447892.5-2462502.5.");
    println!(
        "# mean: SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT, JD 2447892.5-2462502.5."
    );
    println!("# helio: SEFLG_HELCTR|SEFLG_TRUEPOS, JD 2415025.5-2488064.5.");
    println!("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.");
    println!("# No turning point of any pair's separation is within 30 arcsec of one of its levels (the tool fails otherwise).");
    println!(
        "group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day"
    );
    scan(out, "geo", SUN, MOON, GEO, SHORT_SPAN, MOON_GRID_DAYS);
    for (first, second) in [
        (SUN, MERCURY),
        (MERCURY, VENUS),
        (VENUS, MARS),
        (MARS, JUPITER),
        (MARS, SATURN),
        (JUPITER, SATURN),
        (SATURN, PLUTO),
    ] {
        scan(out, "geo", first, second, GEO, FULL_SPAN, GRID_DAYS);
    }
    for (first, second) in [(MERCURY, VENUS), (MARS, SATURN)] {
        scan(out, "mean", first, second, MEAN, SHORT_SPAN, GRID_DAYS);
    }
    scan(out, "helio", MARS, JUPITER, HELIO, FULL_SPAN, GRID_DAYS);
}

fn asteroids_main(args: &[String]) {
    let value_of = |flag: &str| {
        args.iter().position(|a| a == flag).map(|i| {
            args.get(i + 1)
                .unwrap_or_else(|| panic!("{flag} needs a value"))
                .clone()
        })
    };
    let out_dir = value_of("--out").expect("--asteroids needs --out <dir>");
    let ephe_dir = std::env::var("SE_EPHE_PATH")
        .ok()
        .or_else(|| value_of("--ephe"))
        .unwrap_or_else(|| DEFAULT_EPHE_DIR.to_string());
    if let Err(e) = pins::verify_swieph_files(&ephe_dir) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    let ephe = std::ffi::CString::new(ephe_dir).expect("ephe path has NUL");
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };

    let mut csv = String::new();
    csv.push_str("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with seas_18/sepl_18/semo_18 (SHA-256 pinned in tools/se-aspects-reference/src/pins.rs).\n");
    csv.push_str("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)\n");
    csv.push_str("# equals +angle or -angle, scanned on a 0.05-day grid (0.01 day with the Moon), each step split at the zero\n");
    csv.push_str("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.\n");
    csv.push_str("# geo: apparent, tropical, true equinox of date (default flags) with SEFLG_SWIEPH; Sun with Ceres, Pallas, Juno, Vesta JD 2415025.5-2488064.5, Moon-Ceres JD 2447892.5-2462502.5.\n");
    csv.push_str("# Angles 0, 60, 90, 120, 180.\n");
    csv.push_str("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.\n");
    csv.push_str("# No turning point of any pair's separation is within 30 arcsec of one of its levels (the tool fails otherwise).\n");
    csv.push_str("group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day\n");
    let mut rows = 0usize;
    {
        let mut push = |row: String| {
            rows += 1;
            writeln!(csv, "{row}").unwrap();
        };
        const ASTEROIDS: [Body; 4] = [(17, "Ceres"), (18, "Pallas"), (19, "Juno"), (20, "Vesta")];
        for asteroid in ASTEROIDS {
            scan(&mut push, "geo", SUN, asteroid, GEO_SWIEPH, FULL_SPAN, GRID_DAYS);
        }
        scan(&mut push, "geo", MOON, (17, "Ceres"), GEO_SWIEPH, SHORT_SPAN, MOON_GRID_DAYS);
    }
    let manifest = format!(
        "slice aspects-asteroids file=asteroids.csv role=aspects rows={rows} checksum={}\n",
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
