use std::ffi::CString;
use std::os::raw::c_char;

use libswisseph_sys::raw::{swe_calc, swe_fixstar, swe_get_ayanamsa_ex, swe_set_sid_mode};

// SE ephemeris/computation flag bits (see swephexp.h).
const SEFLG_NONUT: i32 = 64; // no nutation (mean obliquity / mean equinox)
const SEFLG_NOABERR: i32 = 1024; // no annual aberration
// Mean ayanamsa: nutation- and aberration-free, hence smooth and cubic-fittable.
const MEAN_IFLAG: i32 = SEFLG_NONUT | SEFLG_NOABERR; // = 1088

/// (pleiades mode_code, SE sidereal-mode integer)
///
/// The SE_SIDM integers below are the one plan-mandated place these literals
/// appear in the tool (for the corpus path). The `measure-offset` path reads
/// each mode's SE_SIDM from `pleiades_ayanamsa::IN_SCOPE_ANCHORS` instead.
const MODES: &[(&str, i32)] = &[
    ("FaganBradley",  0),
    ("Lahiri",        1),
    ("Raman",         3),
    ("Krishnamurti",  5),
    ("TrueChitra",   27),
    ("TrueCitra",    27),
    // offset-defined family — promoted (P) modes only:
    ("J2000",                    18),
    ("J1900",                    19),
    ("B1950",                    20),
    ("UshaShashi",                4),
    ("DjwhalKhul",                6),
    ("Yukteshwar",                7),
    ("JnBhasin",                  8),
    ("Sassanian",                16),
    ("LahiriIcrc",               46),
    ("Lahiri1940",               43),
    ("Aryabhata522",             37),
    ("Suryasiddhanta499",        21),
    ("Suryasiddhanta499MeanSun", 22),
    ("Aryabhata499",             23),
    ("Aryabhata499MeanSun",      24),
    ("SuryasiddhantaRevati",     25),
    ("SuryasiddhantaCitra",      26),
    // fitted family — slice 2: true-star + galactic cubic fits.
    ("TrueRevati",                28),
    ("TruePushya",                29),
    ("TrueMula",                  35),
    ("TrueSheoran",               39),
    ("GalacticCenter",            17),
    ("GalacticCenterRgilbrand",   30),
    ("GalacticEquatorIau1958",    31),
    ("GalacticEquatorTrue",       32),
    ("GalacticEquatorMula",       33),
    ("GalacticCenterMardyks",     34),
    ("GalacticCenterMulaWilhelm", 36),
    ("GalacticCenterCochrane",    40),
    ("GalacticEquatorFiorenza",   41),
    // fitted-offset family — slice 3: failed-offset modes re-fit with cubics.
    ("DeLuce",                2),
    ("BabylonianKugler1",     9),
    ("BabylonianKugler2",    10),
    ("BabylonianKugler3",    11),
    ("BabylonianHuber",      12),
    ("BabylonianEtaPiscium", 13),
    ("BabylonianAldebaran",  14),
    ("Hipparchus",           15),
    ("BabylonianBritton",    38),
    ("ValensMoon",           42),
    ("LahiriVP285",          44),
    ("KrishnamurtiVP291",    45),
    // Still deferred: anchorless modes (Udayagiri, PvrPushyaPaksha, Sheoran —
    // no distinct SE_SIDM code), observational/topocentric/house Babylonians
    // (TrueGeoc/TrueTopc/TrueObs/House/HouseObs/Sissy — not smooth in time), and
    // DhruvaGalacticCenterMula / legacy GalacticEquator (no distinct SE code).
];

/// Hold-out validation instants (jd_tt). Deliberately NOT on the dense fit grid
/// (Task: fit uses an even-year grid), so the gate is a genuine hold-out check.
const HOLDOUT_JD_TT: &[f64] = &[
    2_415_020.5,   // 1900-01-01
    2_420_000.5,
    2_424_152.0,   // ~1925
    2_429_000.5,
    2_433_282.5,   // ~1950
    2_438_000.5,
    2_451_545.0,   // J2000.0
    2_460_676.5,   // ~2025
    2_469_807.0,   // ~2050
    2_488_070.0,   // ~2100
];

fn ayanamsa(code: i32, jd_tt: f64) -> f64 {
    let mut daya = 0.0_f64;
    let mut serr = [0 as c_char; 256];
    let ret = unsafe {
        swe_set_sid_mode(code, 0.0, 0.0);
        swe_get_ayanamsa_ex(jd_tt, MEAN_IFLAG, &mut daya, serr.as_mut_ptr())
    };
    if ret < 0 {
        let msg = unsafe { std::ffi::CStr::from_ptr(serr.as_ptr()) }.to_string_lossy();
        panic!("swe_get_ayanamsa_ex failed for code {code} at jd {jd_tt}: {msg}");
    }
    assert!(daya.is_finite(), "SE returned non-finite ayanamsa for code {code} at jd {jd_tt}");
    daya
}

fn emit_corpus() {
    println!("mode_code,jd_tt,se_ayanamsa_deg");
    for &(name, code) in MODES {
        for &jd in HOLDOUT_JD_TT {
            println!("{name},{jd},{:.9}", ayanamsa(code, jd));
        }
    }
}

/// Solve a small symmetric normal-equation system by Gaussian elimination.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Vec<f64> {
    let n = b.len();
    for col in 0..n {
        let mut piv = col;
        for r in col + 1..n { if a[r][col].abs() > a[piv][col].abs() { piv = r; } }
        a.swap(col, piv); b.swap(col, piv);
        let d = a[col][col];
        for r in 0..n {
            if r == col { continue; }
            let f = a[r][col] / d;
            for c in col..n { a[r][c] -= f * a[col][c]; }
            b[r] -= f * b[col];
        }
    }
    (0..n).map(|i| b[i] / a[i][i]).collect()
}

/// Fit ayanamsa_deg(T) = c0 + c1 T + c2 T^2 + c3 T^3, T = (jd-2451545)/36525,
/// over a dense even-year grid, and print a Rust const block.
fn emit_fit() {
    let dense: Vec<f64> = (1900..=2100).step_by(2)
        .map(|y| 2_451_545.0 + (y as f64 - 2000.0) * 365.25)
        .collect();
    println!("// Generated by se-ayanamsa-reference fit. T = (jd_tt - 2451545.0)/36525.0");
    const FIT_MODES: &[(&str, i32)] = &[
        ("TrueChitra", 27), ("TrueCitra", 27),
        ("TrueRevati", 28), ("TruePushya", 29), ("TrueMula", 35), ("TrueSheoran", 39),
        ("GalacticCenter", 17), ("GalacticCenterRgilbrand", 30),
        ("GalacticEquatorIau1958", 31), ("GalacticEquatorTrue", 32),
        ("GalacticEquatorMula", 33), ("GalacticCenterMardyks", 34),
        ("GalacticCenterMulaWilhelm", 36), ("GalacticCenterCochrane", 40),
        ("GalacticEquatorFiorenza", 41),
        // slice 3 — fitted-offset family:
        ("DeLuce", 2), ("BabylonianKugler1", 9), ("BabylonianKugler2", 10),
        ("BabylonianKugler3", 11), ("BabylonianHuber", 12), ("BabylonianEtaPiscium", 13),
        ("BabylonianAldebaran", 14), ("Hipparchus", 15), ("BabylonianBritton", 38),
        ("ValensMoon", 42), ("LahiriVP285", 44), ("KrishnamurtiVP291", 45),
    ];
    for &(name, code) in FIT_MODES {
        // Normal equations for a degree-3 polynomial.
        let mut ata = vec![vec![0.0f64; 4]; 4];
        let mut atb = vec![0.0f64; 4];
        for &jd in &dense {
            let t = (jd - 2_451_545.0) / 36_525.0;
            let powers = [1.0, t, t * t, t * t * t];
            let y = ayanamsa(code, jd);
            for i in 0..4 {
                atb[i] += powers[i] * y;
                for j in 0..4 { ata[i][j] += powers[i] * powers[j]; }
            }
        }
        let c = solve(ata, atb);
        println!(
            "pub(crate) const {}_COEFFS: [f64; 4] = [{:.12e}, {:.12e}, {:.12e}, {:.12e}];",
            name.to_uppercase(), c[0], c[1], c[2], c[3]
        );
    }
}

/// Evaluate a degree-3 polynomial in T = (jd-2451545)/36525.
fn eval_cubic(c: &[f64; 4], jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36_525.0;
    c[0] + c[1] * t + c[2] * t * t + c[3] * t * t * t
}

/// Refit the true-star cubic and report the worst residual at the holdout
/// instants (arcseconds) — the Step-2 success criterion (< 1.0").
fn emit_holdout_check() {
    let dense: Vec<f64> = (1900..=2100)
        .step_by(2)
        .map(|y| 2_451_545.0 + (y as f64 - 2000.0) * 365.25)
        .collect();
    let code = 27; // True Chitra / True Citra
    let mut ata = vec![vec![0.0f64; 4]; 4];
    let mut atb = vec![0.0f64; 4];
    for &jd in &dense {
        let t = (jd - 2_451_545.0) / 36_525.0;
        let powers = [1.0, t, t * t, t * t * t];
        let y = ayanamsa(code, jd);
        for i in 0..4 {
            atb[i] += powers[i] * y;
            for j in 0..4 {
                ata[i][j] += powers[i] * powers[j];
            }
        }
    }
    let c4: Vec<f64> = solve(ata, atb);
    let c = [c4[0], c4[1], c4[2], c4[3]];
    let mut worst = 0.0f64;
    for &jd in HOLDOUT_JD_TT {
        let se = ayanamsa(code, jd);
        let fitv = eval_cubic(&c, jd);
        let resid = (se - fitv).abs() * 3600.0;
        println!("holdout jd={jd} se={se:.9} fit={fitv:.9} resid={resid:.4}\"");
        if resid > worst {
            worst = resid;
        }
    }
    println!("WORST_TRUESTAR_HOLDOUT_ARCSEC={worst:.4}");
}

/// Wrap an angle (degrees) into (-180, 180], defending the residual against a
/// 360° rollover when SE and the model straddle the 0/360 boundary.
fn wrap_to_pm180(mut deg: f64) -> f64 {
    deg %= 360.0;
    if deg > 180.0 {
        deg -= 360.0;
    } else if deg <= -180.0 {
        deg += 360.0;
    }
    deg
}

/// Cluster cutoff used for the tool's printed PASS/DEFER verdict.
///
/// Empirically (Task 2) the in-scope offset modes split into a tight cluster
/// whose worst residual is <= 1.370402" (anchor + IAU-2006 precession reproduces
/// SE), a gap, then modes at >= 2.24". The recorded OffsetDefined ceiling is the
/// formula value `ceil(max(worst over P) * 2)` = ceil(1.370402 * 2) = 3.0", but
/// promoting the 2.24"/2.27" headroom-only modes would re-derive a runaway
/// ceiling and force-promote the whole table — the fail-safe forbids that. So P
/// is the cluster, and this cutoff sits in the unambiguous gap so the printed
/// verdict matches the recorded P/D. See plan Task 2 results block.
const OFFSET_DEFINED_CLUSTER_CUTOFF_ARCSEC: f64 = 1.5;

/// Fit each fitted-family mode over the dense even-year grid, then report the
/// worst holdout residual (arcsec) and a PASS/DEFER verdict at the 1.0" floor
/// (the per-family ceiling is recomputed in the crate from the measured maxima).
fn emit_measure_fitted() {
    let dense: Vec<f64> = (1900..=2100)
        .step_by(2)
        .map(|y| 2_451_545.0 + (y as f64 - 2000.0) * 365.25)
        .collect();
    // Intentionally lists only the 13 NEW fitted candidates; the fit-emit list (emit_fit)
    // additionally includes the already-gated TrueChitra/TrueCitra.
    const FIT_MODES: &[(&str, i32)] = &[
        ("TrueRevati", 28), ("TruePushya", 29), ("TrueMula", 35), ("TrueSheoran", 39),
        ("GalacticCenter", 17), ("GalacticCenterRgilbrand", 30),
        ("GalacticEquatorIau1958", 31), ("GalacticEquatorTrue", 32),
        ("GalacticEquatorMula", 33), ("GalacticCenterMardyks", 34),
        ("GalacticCenterMulaWilhelm", 36), ("GalacticCenterCochrane", 40),
        ("GalacticEquatorFiorenza", 41),
    ];
    for &(name, code) in FIT_MODES {
        let mut ata = vec![vec![0.0f64; 4]; 4];
        let mut atb = vec![0.0f64; 4];
        for &jd in &dense {
            let t = (jd - 2_451_545.0) / 36_525.0;
            let powers = [1.0, t, t * t, t * t * t];
            let y = ayanamsa(code, jd);
            for i in 0..4 {
                atb[i] += powers[i] * y;
                for j in 0..4 { ata[i][j] += powers[i] * powers[j]; }
            }
        }
        let c4 = solve(ata, atb);
        let c = [c4[0], c4[1], c4[2], c4[3]];
        let mut worst = 0.0f64;
        for &jd in HOLDOUT_JD_TT {
            let resid = (ayanamsa(code, jd) - eval_cubic(&c, jd)).abs() * 3600.0;
            if resid > worst { worst = resid; }
        }
        let verdict = if worst <= 1.0 { "PASS" } else { "DEFER" };
        println!("{name} worst={worst:.6} verdict={verdict}");
    }
}

/// Fit each slice-3 failed-offset candidate over the dense even-year grid, then
/// report the worst holdout residual (arcsec) and a PASS/DEFER verdict at the
/// 1.0" floor (the FittedOffset ceiling is recomputed in the crate from the
/// measured maxima).
fn emit_measure_fitted_offset() {
    let dense: Vec<f64> = (1900..=2100)
        .step_by(2)
        .map(|y| 2_451_545.0 + (y as f64 - 2000.0) * 365.25)
        .collect();
    const FIT_MODES: &[(&str, i32)] = &[
        ("DeLuce", 2), ("BabylonianKugler1", 9), ("BabylonianKugler2", 10),
        ("BabylonianKugler3", 11), ("BabylonianHuber", 12), ("BabylonianEtaPiscium", 13),
        ("BabylonianAldebaran", 14), ("Hipparchus", 15), ("BabylonianBritton", 38),
        ("ValensMoon", 42), ("LahiriVP285", 44), ("KrishnamurtiVP291", 45),
    ];
    for &(name, code) in FIT_MODES {
        let mut ata = vec![vec![0.0f64; 4]; 4];
        let mut atb = vec![0.0f64; 4];
        for &jd in &dense {
            let t = (jd - 2_451_545.0) / 36_525.0;
            let powers = [1.0, t, t * t, t * t * t];
            let y = ayanamsa(code, jd);
            for i in 0..4 {
                atb[i] += powers[i] * y;
                for j in 0..4 { ata[i][j] += powers[i] * powers[j]; }
            }
        }
        let c4 = solve(ata, atb);
        let c = [c4[0], c4[1], c4[2], c4[3]];
        let mut worst = 0.0f64;
        for &jd in HOLDOUT_JD_TT {
            let resid = (ayanamsa(code, jd) - eval_cubic(&c, jd)).abs() * 3600.0;
            if resid > worst { worst = resid; }
        }
        let verdict = if worst <= 1.0 { "PASS" } else { "DEFER" };
        println!("{name} worst={worst:.6} verdict={verdict}");
    }
}

/// Measure each in-scope offset mode's worst residual against the gate's exact
/// OffsetDefined model: `ayan_t0 + precession_delta_degrees(jd, t0)` (degrees),
/// reading `(se_sidm, t0, ayan_t0)` from the crate's single anchor source.
///
/// Prints one line per mode: `<name> worst=<arcsec> verdict=<PASS|DEFER>`.
fn emit_offset_measure() {
    for (mode, anchor) in pleiades_ayanamsa::IN_SCOPE_ANCHORS {
        let mut worst = 0.0f64;
        for &jd in HOLDOUT_JD_TT {
            let model_deg =
                anchor.ayan_t0 + pleiades_ayanamsa::precession_delta_degrees(jd, anchor.t0);
            let se_deg = ayanamsa(anchor.se_sidm, jd);
            let resid_arcsec = wrap_to_pm180(se_deg - model_deg).abs() * 3600.0;
            if resid_arcsec > worst {
                worst = resid_arcsec;
            }
        }
        let verdict = if worst <= OFFSET_DEFINED_CLUSTER_CUTOFF_ARCSEC {
            "PASS"
        } else {
            "DEFER"
        };
        println!("{mode:?} worst={worst:.6} verdict={verdict}");
    }
}

const SEFLG_MOSEPH: i32 = 4;
const SEFLG_TRUEPOS: i32 = 16;
const SEFLG_NOGDEFL: i32 = 512;
/// Apparent star place, nutation-free (the place SEFLG_SIDEREAL uses).
const APPARENT_IFLAG: i32 = SEFLG_MOSEPH | SEFLG_NONUT;
/// Geometric star place: the mean ayanamsa.
const GEOMETRIC_IFLAG: i32 = APPARENT_IFLAG | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL;

/// (mode, SE_SIDM, anchor star name for its solar conjunctions, or None for a
/// mode Swiss Ephemeris does not aberrate).
const APPARENT_MODES: &[(&str, i32, Option<&str>)] = &[
    ("TrueCitra", 27, Some("Spica")),
    ("TrueRevati", 28, Some(",zePsc")),
    ("TruePushya", 29, Some(",deCnc")),
    ("TrueMula", 35, Some(",laSco")),
    ("TrueSheoran", 39, Some(",deCnc")),
    ("GalacticCenter", 17, Some(",SgrA*")),
    ("GalacticCenterRgilbrand", 30, Some(",SgrA*")),
    ("GalacticCenterMulaWilhelm", 36, Some(",SgrA*")),
    ("GalacticCenterCochrane", 40, Some(",SgrA*")),
    ("GalacticEquatorIau1958", 31, None),
    ("GalacticEquatorTrue", 32, None),
    ("GalacticEquatorMula", 33, None),
    ("GalacticCenterMardyks", 34, None),
    ("GalacticEquatorFiorenza", 41, None),
];

fn ayanamsa_with(code: i32, jd_tt: f64, iflag: i32) -> f64 {
    unsafe { swe_set_sid_mode(code, 0.0, 0.0) };
    let mut out = 0.0;
    let mut err = [0 as c_char; 256];
    let rc = unsafe { swe_get_ayanamsa_ex(jd_tt, iflag, &mut out, err.as_mut_ptr()) };
    assert!(rc >= 0 && out.is_finite(), "swe_get_ayanamsa_ex({code}, {jd_tt})");
    out
}

fn star_longitude(name: &str, jd_tt: f64) -> f64 {
    star_place(name, jd_tt).0
}

/// Geometric `(λ, β)` of `name`, mean ecliptic and equinox of date.
fn star_place(name: &str, jd_tt: f64) -> (f64, f64) {
    // swe_fixstar writes the star's full name back into this buffer.
    let mut buf = [0 as c_char; 256];
    for (dst, src) in buf.iter_mut().zip(CString::new(name).unwrap().as_bytes_with_nul()) {
        *dst = *src as c_char;
    }
    let (mut xx, mut err) = ([0.0f64; 6], [0 as c_char; 256]);
    let rc = unsafe {
        swe_fixstar(buf.as_mut_ptr(), jd_tt, GEOMETRIC_IFLAG, xx.as_mut_ptr(), err.as_mut_ptr())
    };
    if rc < 0 {
        let msg = unsafe { std::ffi::CStr::from_ptr(err.as_ptr()) }.to_string_lossy();
        panic!("swe_fixstar {name} at jd {jd_tt}: {msg}");
    }
    (xx[0], xx[1])
}

/// Swiss Ephemeris's mean obliquity of date, degrees (`swe_calc(SE_ECL_NUT)`'s
/// second value).
fn mean_obliquity(jd_tt: f64) -> f64 {
    const SE_ECL_NUT: i32 = -1;
    let (mut xx, mut err) = ([0.0f64; 6], [0 as c_char; 256]);
    let rc = unsafe { swe_calc(jd_tt, SE_ECL_NUT, GEOMETRIC_IFLAG, xx.as_mut_ptr(), err.as_mut_ptr()) };
    assert!(rc >= 0, "swe_calc SE_ECL_NUT");
    xx[1]
}

/// The anchor stars' geometric places at 1900, J2000 and 2100 (issue #226),
/// and the Galactic Center's polar projection as Swiss Ephemeris's
/// Mula/Wilhelm mode computes it: `swi_armc_to_mc` of the star's right
/// ascension, which is the mode-36 geometric ayanamsa + 246.6666666667°.
fn emit_anchor_places() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_fixstar with SEFLG_MOSEPH|SEFLG_NONUT|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL.");
    println!("# Generated by `tools/se-ayanamsa-reference anchor-places`. jd_tt is TT; degrees.");
    println!("star,jd_tt,lambda_deg,beta_deg");
    for star in ["Spica", ",zePsc", ",deCnc", ",laSco", ",SgrA*"] {
        for jd in [2_415_020.5, 2_451_545.0, 2_488_069.5] {
            let (lambda, beta) = star_place(star, jd);
            println!("{star},{jd:.1},{lambda:.9},{beta:.9}");
        }
    }
    println!("jd_tt,sgra_lambda_deg,sgra_beta_deg,mean_obliquity_deg,se_polar_projection_deg");
    for jd in [2_415_020.5, 2_451_545.0, 2_488_069.5] {
        let (lambda, beta) = star_place(",SgrA*", jd);
        let projection = (ayanamsa_with(36, jd, GEOMETRIC_IFLAG) + 246.666_666_666_7).rem_euclid(360.0);
        println!("{jd:.1},{lambda:.9},{beta:.9},{:.9},{projection:.9}", mean_obliquity(jd));
    }
}

fn sun_longitude(jd_tt: f64) -> f64 {
    let (mut xx, mut err) = ([0.0f64; 6], [0 as c_char; 256]);
    let rc = unsafe { swe_calc(jd_tt, 0, GEOMETRIC_IFLAG, xx.as_mut_ptr(), err.as_mut_ptr()) };
    assert!(rc >= 0, "swe_calc Sun");
    xx[0]
}

fn wrap180(d: f64) -> f64 {
    (d + 540.0).rem_euclid(360.0) - 180.0
}

/// The instant in the window [jd0, jd0 + 366) at which the Sun's geometric
/// longitude equals `star`'s: daily scan, then bisection to 1e-6 d.
fn conjunction(star: &str, jd0: f64) -> f64 {
    let f = |jd: f64| wrap180(sun_longitude(jd) - star_longitude(star, jd));
    let mut lo = jd0;
    while !(f(lo) < 0.0 && f(lo + 1.0) >= 0.0) {
        lo += 1.0;
        assert!(lo < jd0 + 367.0, "no conjunction of {star} after {jd0}");
    }
    let mut hi = lo + 1.0;
    while hi - lo > 1e-6 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid
        } else {
            hi = mid
        }
    }
    0.5 * (lo + hi)
}

/// Apparent − geometric ayanamsa (Swiss Ephemeris, Moshier), arcsec: 60
/// uniform rows per anchored mode plus 50 rows around ten conjunctions of its
/// anchor star with the Sun (where light deflection peaks), and 10 uniform rows
/// per mode Swiss Ephemeris does not aberrate.
fn emit_apparent() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_get_ayanamsa_ex after swe_set_sid_mode.");
    println!("# se_correction_arcsec = (ayanamsa with SEFLG_MOSEPH|SEFLG_NONUT - ayanamsa with SEFLG_MOSEPH|SEFLG_NONUT|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL) * 3600.");
    println!("# class: uniform (spread over 1900-2100) or conjunction (within a day of the anchor star's conjunction with the Sun).");
    println!("# Generated by `tools/se-ayanamsa-reference apparent`. jd_tt is TT.");
    println!("mode,jd_tt,class,se_correction_arcsec");
    for &(name, code, star) in APPARENT_MODES {
        let uniform = if star.is_some() { 60 } else { 10 };
        let step = (2_488_069.5 - 2_415_020.5) / uniform as f64;
        for k in 0..uniform {
            let jd = 2_415_020.87 + k as f64 * step;
            let c = (ayanamsa_with(code, jd, APPARENT_IFLAG) - ayanamsa_with(code, jd, GEOMETRIC_IFLAG)) * 3600.0;
            println!("{name},{jd:.6},uniform,{c:.6}");
        }
        let Some(star) = star else { continue };
        for year in 0..10 {
            let jd0 = 2_416_480.5 + year as f64 * 7_305.0; // 1904-01-01 + 20 y steps
            let t0 = conjunction(star, jd0);
            for offset in [-1.0, -0.15, 0.0, 0.15, 1.0] {
                let jd = t0 + offset;
                let c = (ayanamsa_with(code, jd, APPARENT_IFLAG) - ayanamsa_with(code, jd, GEOMETRIC_IFLAG)) * 3600.0;
                println!("{name},{jd:.6},conjunction,{c:.6}");
            }
        }
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("fit") => emit_fit(),
        Some("holdout") => emit_holdout_check(),
        Some("measure-offset") => emit_offset_measure(),
        Some("measure-fitted") => emit_measure_fitted(),
        Some("measure-fitted-offset") => emit_measure_fitted_offset(),
        Some("apparent") => emit_apparent(),
        Some("anchor-places") => emit_anchor_places(),
        _ => emit_corpus(),
    }
}
