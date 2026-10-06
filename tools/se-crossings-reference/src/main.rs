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
//! Frame `helio`: heliocentric geometric place (SEFLG_HELCTR | SEFLG_TRUEPOS)
//!   via `swe_helio_cross` (ET/TDB). Without SEFLG_TRUEPOS Swiss Ephemeris
//!   retards the planet by the heliocentric light-time r/c, which is 3 minutes
//!   for Mercury and four to six hours for Pluto; the engine's heliocentric
//!   frame is the place at the instant itself (issue #163).
//! Frame `geo-mean`: geocentric geometric longitude on the mean equinox of
//!   date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT),
//!   bisected on `swe_calc` like the geocentric planets.
//! Column `zodiac`: `tropical`, or an ayanamsa name for rows computed with
//!   SEFLG_SIDEREAL after `swe_set_sid_mode`. Swiss Ephemeris drops nutation
//!   from sidereal positions, so a sidereal longitude is the mean-equinox
//!   longitude minus the mean ayanamsa.
//!   Exception: `geo` rows of the star-anchored ayanamsas (TrueCitra,
//!   GalacticCenter) use the apparent mean-equinox longitude minus the mean
//!   ayanamsa (`swe_get_ayanamsa_ex` with TRUEPOS|NOABERR|NOGDEFL). Under plain
//!   SEFLG_SIDEREAL Swiss Ephemeris takes the anchoring star's apparent place and
//!   so folds the star's annual aberration (up to ~20") into the ayanamsa;
//!   pleiades uses the mean ayanamsa. `check_sidereal_decomposition` asserts the
//!   decomposition against SE's own geometric SEFLG_SIDEREAL longitude.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH) so no data files are needed.
//! All start and crossing JDs are TDB and fall inside the 1900-2100 window.
//! Every row is a FORWARD "next crossing after start" (direction=fwd).
//!
//! Usage: `cargo run --release > .../crossings-corpus/crossings.csv`
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{
    swe_calc, swe_deltat, swe_get_ayanamsa_ex, swe_helio_cross, swe_mooncross_ut, swe_set_sid_mode,
    swe_solcross_ut,
};

const SEFLG_MOSEPH: c_int = 4; // Moshier ephemeris, no data files
const SEFLG_HELCTR: c_int = 8; // heliocentric position
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_SIDEREAL: c_int = 64 * 1024;

/// Geometric place: the `geo-mean` frame before the equinox choice.
const GEOMETRIC: c_int = SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL;

const SE_SUN: c_int = 0;
const SE_MOON: c_int = 1;

// SE ipl body numbers.
const SE_MERCURY: c_int = 2;
const SE_VENUS: c_int = 3;
const SE_MARS: c_int = 4;
const SE_JUPITER: c_int = 5;
const SE_SATURN: c_int = 6;
const SE_URANUS: c_int = 7;
const SE_NEPTUNE: c_int = 8;
const SE_PLUTO: c_int = 9;

// 1900-2100 CE packaged window (TDB Julian days).
const JD_WINDOW_LO: f64 = 2_415_020.5; // 1900-01-01
const JD_WINDOW_HI: f64 = 2_488_069.5; // 2100-01-01

fn serr_string(serr: &[c_char]) -> String {
    unsafe { CStr::from_ptr(serr.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// Geocentric longitude of `ipl` (degrees, [0,360)) at TDB under `iflag`.
fn flagged_longitude(jd_tdb: f64, ipl: c_int, iflag: c_int) -> f64 {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tdb,
            ipl,
            iflag,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_calc(ipl={ipl}, iflag={iflag}) failed at jd_tdb={jd_tdb}: {}",
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
            SEFLG_MOSEPH | SEFLG_HELCTR | SEFLG_TRUEPOS,
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
/// Serves all geocentric planets (Mercury–Pluto): SE exposes no geocentric
/// planet-crossing function. The bracket-acceptance test guards against the
/// `signed_delta` antipode discontinuity (target±180°, where it wraps
/// +180→−180): a true zero-crossing has |fa−fb| ≪ 180° at STEP=0.25 d, whereas
/// the antipode seam gives |fa−fb| ≈ 360°, so the `< 180.0` guard rejects it.
fn bisect_cross_tdb(longitude: impl Fn(f64) -> f64, target_deg: f64, start_tdb: f64) -> f64 {
    const STEP: f64 = 0.25; // days; << time to move 180deg, so no wrap aliasing
    let mut a = start_tdb;
    let mut fa = signed_delta(longitude(a), target_deg);
    let mut jd = start_tdb + STEP;
    while jd <= JD_WINDOW_HI {
        let fb = signed_delta(longitude(jd), target_deg);
        if fa == 0.0 || ((fa < 0.0) != (fb < 0.0) && (fa - fb).abs() < 180.0) {
            // bracket [a, jd]: bisect
            let (mut lo, mut hi) = (a, jd);
            for _ in 0..64 {
                let mid = 0.5 * (lo + hi);
                let fm = signed_delta(longitude(mid), target_deg);
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
    panic!(
        "no geocentric crossing of {target_deg} after {start_tdb} within window"
    );
}

fn flagged_cross_tdb(ipl: c_int, iflag: c_int, target_deg: f64, start_tdb: f64) -> f64 {
    bisect_cross_tdb(|jd| flagged_longitude(jd, ipl, iflag), target_deg, start_tdb)
}

/// Mean ayanamsa (degrees) of the sidereal mode last set with `swe_set_sid_mode`.
/// Geometric flags keep a star-anchored mode from folding the anchoring star's
/// annual aberration into the offset.
fn mean_ayanamsa(jd_tdb: f64) -> f64 {
    let mut daya = 0.0_f64;
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_get_ayanamsa_ex(
            jd_tdb,
            SEFLG_MOSEPH | SEFLG_NONUT | GEOMETRIC,
            &mut daya,
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_get_ayanamsa_ex failed at jd_tdb={jd_tdb}: {}",
            serr_string(&serr)
        );
    }
    daya
}

/// Apparent mean-equinox-of-date longitude minus the mean ayanamsa: the `geo`
/// reference for star-anchored ayanamsas.
fn apparent_mean_sidereal_longitude(jd_tdb: f64, ipl: c_int) -> f64 {
    (flagged_longitude(jd_tdb, ipl, SEFLG_MOSEPH | SEFLG_NONUT) - mean_ayanamsa(jd_tdb))
        .rem_euclid(360.0)
}

fn apparent_mean_sidereal_cross_tdb(ipl: c_int, target_deg: f64, start_tdb: f64) -> f64 {
    bisect_cross_tdb(
        |jd| apparent_mean_sidereal_longitude(jd, ipl),
        target_deg,
        start_tdb,
    )
}

/// Asserts that, for each ayanamsa, SE's geometric `SEFLG_SIDEREAL` longitude
/// equals the geometric mean-equinox longitude minus the mean ayanamsa.
fn check_sidereal_decomposition(ayanamsas: &[(&str, c_int)]) {
    let jd = 2_450_000.5;
    for &(name, sid_mode) in ayanamsas {
        unsafe { swe_set_sid_mode(sid_mode, 0.0, 0.0) };
        let sidereal = flagged_longitude(jd, SE_SUN, SEFLG_MOSEPH | SEFLG_SIDEREAL | GEOMETRIC);
        let decomposed = (flagged_longitude(jd, SE_SUN, SEFLG_MOSEPH | GEOMETRIC | SEFLG_NONUT)
            - mean_ayanamsa(jd))
        .rem_euclid(360.0);
        let diff = signed_delta(sidereal, decomposed).abs();
        assert!(
            diff < 1e-6,
            "{name}: sidereal {sidereal} vs decomposed {decomposed} differ by {diff} deg"
        );
    }
}

fn geo_planet_cross_tdb(ipl: c_int, target_deg: f64, start_tdb: f64) -> f64 {
    flagged_cross_tdb(ipl, SEFLG_MOSEPH, target_deg, start_tdb)
}

fn in_window(jd: f64) -> bool {
    (JD_WINDOW_LO..=JD_WINDOW_HI).contains(&jd)
}

fn emit(frame: &str, body: &str, target: f64, start_tdb: f64, crossing_tdb: f64, zodiac: &str) {
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
    println!("{frame},{body},{target:.6},{start_tdb:.6},fwd,{crossing_tdb:.9},{zodiac}");
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2).");
    println!("# geo Sun/Moon: swe_solcross_ut / swe_mooncross_ut (UT), TDB via swe_deltat.");
    println!("# geo planets: bisection on swe_calc geocentric longitude (no SE geo planet-cross fn); Mars block retains the retrograde triple-crossing.");
    println!("# helio: swe_helio_cross (ET/TDB), iflag=SEFLG_MOSEPH|SEFLG_HELCTR|SEFLG_TRUEPOS (geometric, no light-time), dir=+1.");
    println!("# geo-mean: bisection on swe_calc with SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT.");
    println!("# zodiac != tropical: swe_set_sid_mode + SEFLG_SIDEREAL (nutation-free; mean ayanamsa).");
    println!("# geo rows of star-anchored ayanamsas (TrueCitra, GalacticCenter): apparent mean-equinox longitude minus the mean ayanamsa (swe_get_ayanamsa_ex, TRUEPOS|NOABERR|NOGDEFL), because under plain SEFLG_SIDEREAL SE takes the anchoring star's apparent place and folds the star's annual aberration (up to ~20\") into the ayanamsa; pleiades uses the mean ayanamsa.");
    println!("# All rows forward (next crossing after start); times TDB within 1900-2100.");
    println!("frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac");

    // --- geo Sun: cardinal points + one arbitrary longitude, starts spread. ---
    let sun_targets = [0.0_f64, 90.0, 180.0, 270.0, 137.5];
    let sun_starts = [2_416_000.5_f64, 2_440_000.5, 2_470_000.5]; // ~1902 / ~1968 / ~2050
    for &start in &sun_starts {
        for &t in &sun_targets {
            let c = geo_sun_cross_tdb(t, start);
            emit("geo", "Sun", t, start, c, "tropical");
        }
    }

    // --- geo Moon: cardinal points + one arbitrary longitude, starts spread. ---
    let moon_targets = [0.0_f64, 90.0, 180.0, 270.0, 45.0];
    let moon_starts = [2_420_000.5_f64, 2_450_000.5, 2_480_000.5]; // ~1913 / ~1995 / ~2077
    for &start in &moon_starts {
        for &t in &moon_targets {
            let c = geo_moon_cross_tdb(t, start);
            emit("geo", "Moon", t, start, c, "tropical");
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
        emit("geo", "Mars", mars_target, start, c, "tropical");
    }

    // --- geo planets Mercury–Pluto: cardinal-ish targets, in-window starts. ---
    // SE has no geocentric planet-crossing function, so each is bisected on
    // swe_calc geocentric longitude (see geo_planet_cross_tdb). Starts are chosen
    // to sit comfortably inside 1900–2100 so the forward crossing is in-window.
    let geo_planets: [(c_int, &str); 7] = [
        (SE_MERCURY, "Mercury"),
        (SE_VENUS, "Venus"),
        (SE_JUPITER, "Jupiter"),
        (SE_SATURN, "Saturn"),
        (SE_URANUS, "Uranus"),
        (SE_NEPTUNE, "Neptune"),
        (SE_PLUTO, "Pluto"),
    ];
    let geo_planet_targets = [0.0_f64, 120.0, 240.0];
    // ~1903, early in the window: even Pluto (~248 yr period, slowest) reaches all
    // three targets before 2100. A later start (e.g. ~1968) leaves Pluto past its
    // final in-window 120° passage, so that crossing would not exist in-window.
    let geo_planet_start = 2_416_000.5_f64;
    for &(ipl, name) in &geo_planets {
        for &t in &geo_planet_targets {
            let c = geo_planet_cross_tdb(ipl, t, geo_planet_start);
            emit("geo", name, t, geo_planet_start, c, "tropical");
        }
    }

    // --- helio planets Mercury–Pluto via swe_helio_cross. ---
    // Starts are ~1901 / ~1930 (both ≤1930) so even Neptune's worst-case
    // one-period reach (~165 yr) stays inside the 1900–2100 window. Pluto
    // (~248 yr period) still cannot fit both a 0° and 180° crossing in the
    // ~200 yr window, so out-of-window helio crossings are documented and
    // skipped below rather than emitted.
    let helio_targets = [0.0_f64, 180.0];
    let helio_starts = [2_415_400.5_f64, 2_426_000.5]; // ~1901 / ~1930
    let helio_planets: [(c_int, &str); 8] = [
        (SE_MERCURY, "Mercury"),
        (SE_VENUS, "Venus"),
        (SE_MARS, "Mars"),
        (SE_JUPITER, "Jupiter"),
        (SE_SATURN, "Saturn"),
        (SE_URANUS, "Uranus"),
        (SE_NEPTUNE, "Neptune"),
        (SE_PLUTO, "Pluto"),
    ];
    for &(ipl, name) in &helio_planets {
        for &start in &helio_starts {
            for &t in &helio_targets {
                let c = helio_cross_tdb(ipl, t, start);
                // Slow outer planets (Pluto) can have their next crossing fall
                // beyond the hard 1900–2100 window. Document and skip rather
                // than panic in emit's window assertion; keeps exclusions
                // explicit in the corpus.
                if !in_window(c) {
                    println!(
                        "# skipped: helio {} {} (next crossing JD {:.6} beyond 1900-2100 window)",
                        name, t, c
                    );
                    continue;
                }
                emit("helio", name, t, start, c, "tropical");
            }
        }
    }

    // --- geo-mean tropical: geometric place, mean equinox of date. ---
    let mean_tropical = SEFLG_MOSEPH | GEOMETRIC | SEFLG_NONUT;
    for &start in &sun_starts {
        for &t in &[0.0_f64, 90.0, 137.5] {
            emit("geo-mean", "Sun", t, start, flagged_cross_tdb(SE_SUN, mean_tropical, t, start), "tropical");
        }
    }
    for &start in &moon_starts {
        for &t in &[0.0_f64, 180.0, 45.0] {
            emit("geo-mean", "Moon", t, start, flagged_cross_tdb(SE_MOON, mean_tropical, t, start), "tropical");
        }
    }
    let mut mars_prev = f64::NEG_INFINITY;
    for &start in &mars_starts {
        let c = flagged_cross_tdb(SE_MARS, mean_tropical, mars_target, start);
        assert!(c > mars_prev + 1.0, "geo-mean Mars crossings not distinct: {c} vs {mars_prev}");
        mars_prev = c;
        emit("geo-mean", "Mars", mars_target, start, c, "tropical");
    }
    for &t in &geo_planet_targets {
        let c = flagged_cross_tdb(SE_JUPITER, mean_tropical, t, geo_planet_start);
        emit("geo-mean", "Jupiter", t, geo_planet_start, c, "tropical");
    }

    // --- sidereal: one ayanamsa per pleiades-ayanamsa computation class. ---
    // (name, SE_SIDM): OffsetDefined, TrueStar, Galactic, FittedOffset.
    let ayanamsas: [(&str, c_int); 4] = [
        ("Lahiri", 1),
        ("TrueCitra", 27),
        ("GalacticCenter", 17),
        ("DeLuce", 2),
    ];
    let places: [(&str, c_int); 2] = [
        ("geo", SEFLG_MOSEPH | SEFLG_SIDEREAL),
        ("geo-mean", SEFLG_MOSEPH | SEFLG_SIDEREAL | GEOMETRIC),
    ];
    check_sidereal_decomposition(&ayanamsas);
    for &(zodiac, sid_mode) in &ayanamsas {
        unsafe { swe_set_sid_mode(sid_mode, 0.0, 0.0) };
        let star_anchored = zodiac == "TrueCitra" || zodiac == "GalacticCenter";
        for &(frame, iflag) in &places {
            // Star-anchored `geo` rows: apparent longitude minus the mean ayanamsa.
            let cross = |ipl: c_int, t: f64, start: f64| {
                if star_anchored && frame == "geo" {
                    apparent_mean_sidereal_cross_tdb(ipl, t, start)
                } else {
                    flagged_cross_tdb(ipl, iflag, t, start)
                }
            };
            for &start in &[2_416_000.5_f64, 2_470_000.5] {
                for &t in &[0.0_f64, 137.5] {
                    emit(frame, "Sun", t, start, cross(SE_SUN, t, start), zodiac);
                }
            }
            for &start in &[2_420_000.5_f64, 2_480_000.5] {
                emit(frame, "Moon", 45.0, start, cross(SE_MOON, 45.0, start), zodiac);
            }
            let c = cross(SE_JUPITER, 120.0, geo_planet_start);
            emit(frame, "Jupiter", 120.0, geo_planet_start, c, zodiac);
        }
    }

    // --- sidereal Mars retrograde triple-crossing (Lahiri, apparent). ---
    // Tropical 337.0 deg is 313.1 deg in the Lahiri zodiac in 2003
    // (ayanamsa 23.9 deg), so the same three starts bracket the same loop.
    unsafe { swe_set_sid_mode(1, 0.0, 0.0) };
    let sidereal_mars_target = 313.1_f64;
    let mut mars_prev = f64::NEG_INFINITY;
    for &start in &mars_starts {
        let c = flagged_cross_tdb(SE_MARS, SEFLG_MOSEPH | SEFLG_SIDEREAL, sidereal_mars_target, start);
        assert!(c > mars_prev + 1.0, "sidereal Mars crossings not distinct: {c} vs {mars_prev}");
        mars_prev = c;
        emit("geo", "Mars", sidereal_mars_target, start, c, "Lahiri");
    }
}
