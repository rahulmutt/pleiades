use crate::tables::series::{
    truncate_tables, SeriesFidelity, TRUNCATION_THRESHOLD, TRUNCATION_T_MAX_MILLENNIA,
};
use crate::tables::vsop87b_earth::{
    earth_lbr, evaluate, parse_generated_vsop87b_tables, SphericalLbr, Vsop87SeriesTables,
};
use crate::tables::vsop87b_jupiter::jupiter_lbr;
use crate::tables::vsop87b_mars::mars_lbr;
use crate::tables::vsop87b_mercury::mercury_lbr;
use crate::tables::vsop87b_neptune::neptune_lbr;
use crate::tables::vsop87b_saturn::saturn_lbr;
use crate::tables::vsop87b_uranus::uranus_lbr;
use crate::tables::vsop87b_venus::venus_lbr;

type LbrFn = fn(f64, SeriesFidelity) -> SphericalLbr;

struct Body {
    name: &'static str,
    bytes: &'static [u8],
    lbr: LbrFn,
    truncated_terms: usize,
}

fn bodies() -> [Body; 8] {
    [
        Body {
            name: "Earth",
            bytes: include_bytes!("../../data/VSOP87B.ear.bin"),
            lbr: earth_lbr,
            truncated_terms: 578,
        },
        Body {
            name: "Mercury",
            bytes: include_bytes!("../../data/VSOP87B.mer.bin"),
            lbr: mercury_lbr,
            truncated_terms: 672,
        },
        Body {
            name: "Venus",
            bytes: include_bytes!("../../data/VSOP87B.ven.bin"),
            lbr: venus_lbr,
            truncated_terms: 419,
        },
        Body {
            name: "Mars",
            bytes: include_bytes!("../../data/VSOP87B.mar.bin"),
            lbr: mars_lbr,
            truncated_terms: 1390,
        },
        Body {
            name: "Jupiter",
            bytes: include_bytes!("../../data/VSOP87B.jup.bin"),
            lbr: jupiter_lbr,
            truncated_terms: 1654,
        },
        Body {
            name: "Saturn",
            bytes: include_bytes!("../../data/VSOP87B.sat.bin"),
            lbr: saturn_lbr,
            truncated_terms: 3115,
        },
        Body {
            name: "Uranus",
            bytes: include_bytes!("../../data/VSOP87B.ura.bin"),
            lbr: uranus_lbr,
            truncated_terms: 3159,
        },
        Body {
            name: "Neptune",
            bytes: include_bytes!("../../data/VSOP87B.nep.bin"),
            lbr: neptune_lbr,
            truncated_terms: 1278,
        },
    ]
}

fn term_count(tables: &Vsop87SeriesTables) -> usize {
    [&tables.longitude, &tables.latitude, &tables.radius]
        .into_iter()
        .flatten()
        .map(Vec::len)
        .sum()
}

#[test]
fn truncated_tables_keep_the_expected_term_counts() {
    for body in bodies() {
        let full = parse_generated_vsop87b_tables(body.bytes);
        let truncated = truncate_tables(&full, TRUNCATION_THRESHOLD, TRUNCATION_T_MAX_MILLENNIA);
        assert_eq!(
            term_count(&truncated),
            body.truncated_terms,
            "{}",
            body.name
        );
    }
}

#[test]
fn truncate_tables_keeps_power_alignment() {
    for body in bodies() {
        let full = parse_generated_vsop87b_tables(body.bytes);
        let emptied = truncate_tables(&full, f64::INFINITY, TRUNCATION_T_MAX_MILLENNIA);
        assert_eq!(term_count(&emptied), 0, "{}", body.name);
        assert_eq!(
            emptied.longitude.len(),
            full.longitude.len(),
            "{}",
            body.name
        );
        assert_eq!(emptied.latitude.len(), full.latitude.len(), "{}", body.name);
        assert_eq!(emptied.radius.len(), full.radius.len(), "{}", body.name);
    }
}

#[test]
fn full_fidelity_is_bit_identical_to_the_generated_tables() {
    for body in bodies() {
        let tables = parse_generated_vsop87b_tables(body.bytes);
        for jd in [2_451_545.0, 2_305_447.5, 2_670_690.5] {
            let t = (jd - 2_451_545.0) / 365_250.0;
            let actual = (body.lbr)(jd, SeriesFidelity::Full);
            let longitude = evaluate(tables.longitude.iter().map(Vec::as_slice), t)
                .rem_euclid(core::f64::consts::TAU);
            let latitude = evaluate(tables.latitude.iter().map(Vec::as_slice), t);
            let radius = evaluate(tables.radius.iter().map(Vec::as_slice), t);
            assert!(
                actual.longitude_rad == longitude,
                "{} lon at {jd}",
                body.name
            );
            assert!(actual.latitude_rad == latitude, "{} lat at {jd}", body.name);
            assert!(actual.radius_au == radius, "{} rad at {jd}", body.name);
        }
    }
}

#[test]
fn truncated_fidelity_differs_but_stays_close_at_j2000() {
    let full = mars_lbr(2_451_545.0, SeriesFidelity::Full);
    let truncated = mars_lbr(2_451_545.0, SeriesFidelity::Truncated);
    assert!(
        full.longitude_rad != truncated.longitude_rad
            || full.latitude_rad != truncated.latitude_rad
            || full.radius_au != truncated.radius_au
    );
    let delta_arcsec = (full.longitude_rad - truncated.longitude_rad)
        .to_degrees()
        .abs()
        * 3600.0;
    assert!(delta_arcsec < 0.30, "longitude delta {delta_arcsec} arcsec");
}

// ---------------------------------------------------------------------------
// The default backend's windowed truncation against the full-series backend.
// ---------------------------------------------------------------------------

use crate::backend::Vsop87Backend;
use crate::pluto::PlutoPath;
use crate::tables::series::{TRUNCATED_WINDOW_END_JD, TRUNCATED_WINDOW_START_JD};
use crate::transforms::signed_longitude_delta_degrees;
use crate::Vsop87FullSeriesBackend;
use pleiades_backend::{
    Apparentness, EphemerisBackend, EphemerisRequest, EphemerisResult, QualityAnnotation,
};
use pleiades_types::{CelestialBody, EclipticCoordinates, Instant, JulianDay, TimeScale};

/// One body's drift ceilings and where its dense-scan maxima sit.
struct DriftCeiling {
    body: CelestialBody,
    /// Ceiling on |Δλ|, arcseconds.
    longitude_arcsec: f64,
    /// Ceiling on |Δβ|, arcseconds.
    latitude_arcsec: f64,
    /// Julian day (TT) of the dense-scan |Δλ| maximum.
    longitude_peak_jd: f64,
    /// Julian day (TT) of the dense-scan |Δβ| maximum.
    latitude_peak_jd: f64,
}

const fn ceiling(
    body: CelestialBody,
    longitude_arcsec: f64,
    latitude_arcsec: f64,
    longitude_peak_jd: f64,
    latitude_peak_jd: f64,
) -> DriftCeiling {
    DriftCeiling {
        body,
        longitude_arcsec,
        latitude_arcsec,
        longitude_peak_jd,
        latitude_peak_jd,
    }
}

/// Per-body ceilings on |Δλ| and |Δβ| (arcseconds, mean geometric TT place)
/// between the default and the full-series backend over 1600–2600 CE:
/// `min(0.30, ⌈1.25 × max⌉₀.₀₁)` of the dense maxima.
///
/// The maxima were measured on 2026-10-19 on 200,000 evenly spaced instants
/// over the window, then refined over ±15 days at 0.005-day steps around each
/// body's longitude and latitude argmax (the refinement added under 0.001″).
/// The peaks are spread over the window, not at its 2600 edge: the largest
/// sit near 2412 (Sun, Mercury) and 2508 (Venus, Mars). Mars longitude is
/// capped at the 0.30″ spec limit (1.25 × 0.2625″ would be 0.33″), which
/// leaves it 14% headroom.
const TRUNCATION_DRIFT_CEILINGS: [DriftCeiling; 9] = [
    // Dense maxima (lon, lat): 0.0504″ at 2412.0, 0.0334″ at 2459.9.
    ceiling(CelestialBody::Sun, 0.07, 0.05, 2_602_027.420, 2_619_508.285),
    // 0.0751″ at 2413.9, 0.0470″ at 2490.7.
    ceiling(
        CelestialBody::Mercury,
        0.10,
        0.06,
        2_602_705.947,
        2_630_760.707,
    ),
    // 0.1738″ at 2508.0, 0.1420″ at 2540.0.
    ceiling(
        CelestialBody::Venus,
        0.22,
        0.18,
        2_637_091.463,
        2_648_782.912,
    ),
    // 0.2625″ at 2507.6 (capped at 0.30″), 0.1329″ at 1956.7.
    ceiling(
        CelestialBody::Mars,
        0.30,
        0.17,
        2_636_940.599,
        2_435_729.736,
    ),
    // 0.0592″ at 1675.4, 0.0389″ at 2500.0.
    ceiling(
        CelestialBody::Jupiter,
        0.08,
        0.05,
        2_332_976.768,
        2_634_176.689,
    ),
    // 0.0883″ at 2357.0, 0.0369″ at 2004.9.
    ceiling(
        CelestialBody::Saturn,
        0.12,
        0.05,
        2_581_940.792,
        2_453_348.517,
    ),
    // 0.0668″ at 2304.7, 0.0347″ at 1661.6.
    ceiling(
        CelestialBody::Uranus,
        0.09,
        0.05,
        2_562_827.295,
        2_327_948.978,
    ),
    // 0.0369″ at 1847.7, 0.0191″ at 1733.8.
    ceiling(
        CelestialBody::Neptune,
        0.05,
        0.03,
        2_395_929.090,
        2_354_323.900,
    ),
    // 0.0012″ at 2475.8, 0.0011″ at 2491.8: only the VSOP87B Earth subtracted
    // from Pluto is truncated.
    ceiling(
        CelestialBody::Pluto,
        0.01,
        0.01,
        2_625_328.243,
        2_631_165.236,
    ),
];

/// Bound on the step at either window edge, in both axes (arcseconds). The
/// measured steps are at most 0.085″ (Venus longitude at 1600-01-01) and
/// 0.013″ in longitude (Saturn) at 2600-01-01; see
/// `edge_step_is_within_the_drift_ceiling`.
const EDGE_STEP_LIMIT_ARCSEC: f64 = 0.10;

fn tt_request(body: CelestialBody, jd: f64) -> EphemerisRequest {
    let mut request =
        EphemerisRequest::new(body, Instant::new(JulianDay::from_days(jd), TimeScale::Tt));
    request.apparent = Apparentness::Mean;
    request
}

fn mean_place(
    backend: &impl EphemerisBackend,
    body: CelestialBody,
    jd: f64,
) -> EclipticCoordinates {
    backend
        .position_without_motion(&tt_request(body, jd))
        .expect("mean TT position")
        .ecliptic
        .expect("ecliptic channel")
}

fn full_result(backend: &impl EphemerisBackend, body: CelestialBody, jd: f64) -> EphemerisResult {
    backend
        .position(&tt_request(body, jd))
        .expect("mean TT position with motion")
}

fn longitude_delta_arcsec(a: &EclipticCoordinates, b: &EclipticCoordinates) -> f64 {
    signed_longitude_delta_degrees(a.longitude.degrees(), b.longitude.degrees()).abs() * 3600.0
}

fn latitude_delta_arcsec(a: &EclipticCoordinates, b: &EclipticCoordinates) -> f64 {
    (a.latitude.degrees() - b.latitude.degrees()).abs() * 3600.0
}

/// Largest |Δλ| and |Δβ| (arcseconds) between the default and the
/// full-series backend over `instants`.
fn max_drift(body: &CelestialBody, instants: impl Iterator<Item = f64>) -> (f64, f64) {
    let default = Vsop87Backend::new();
    let full = Vsop87FullSeriesBackend::new();
    instants.fold((0.0_f64, 0.0_f64), |(max_lon, max_lat), jd| {
        let truncated = mean_place(&default, body.clone(), jd);
        let reference = mean_place(&full, body.clone(), jd);
        (
            max_lon.max(longitude_delta_arcsec(&truncated, &reference)),
            max_lat.max(latitude_delta_arcsec(&truncated, &reference)),
        )
    })
}

/// `samples` evenly spaced instants over the window, at the centres of
/// `samples` equal cells.
fn uniform_grid(samples: usize) -> impl Iterator<Item = f64> {
    let span = TRUNCATED_WINDOW_END_JD - TRUNCATED_WINDOW_START_JD;
    (0..samples).map(move |i| TRUNCATED_WINDOW_START_JD + (i as f64 + 0.5) * span / samples as f64)
}

/// ±15 days at 0.05-day steps (601 instants) around `centre`.
fn peak_neighbourhood(centre: f64) -> impl Iterator<Item = f64> {
    (-300..=300).map(move |i| centre + f64::from(i) * 0.05)
}

fn assert_within_ceiling(ceiling: &DriftCeiling, (max_lon, max_lat): (f64, f64)) {
    let body = &ceiling.body;
    println!("{body:?}: max |dlon| = {max_lon:.4}\", max |dlat| = {max_lat:.4}\"");
    assert!(
        max_lon <= ceiling.longitude_arcsec,
        "{body:?} longitude drift {max_lon}\" over ceiling {}\"",
        ceiling.longitude_arcsec
    );
    assert!(
        max_lat <= ceiling.latitude_arcsec,
        "{body:?} latitude drift {max_lat}\" over ceiling {}\"",
        ceiling.latitude_arcsec
    );
}

/// Blocking tier: a 2,000-instant uniform grid plus the neighbourhood of each
/// body's dense-scan longitude and latitude peaks, so the gate reaches the
/// dense maxima the sparse grid would miss.
#[test]
fn truncation_drift_stays_within_ceilings_over_1600_2600() {
    for ceiling in &TRUNCATION_DRIFT_CEILINGS {
        let instants = uniform_grid(2_000)
            .chain(peak_neighbourhood(ceiling.longitude_peak_jd))
            .chain(peak_neighbourhood(ceiling.latitude_peak_jd));
        assert_within_ceiling(ceiling, max_drift(&ceiling.body, instants));
    }
}

/// Slow tier: the 200,000-instant scan the ceilings were derived from, so a
/// change that moves a peak away from its pinned neighbourhood is still caught.
/// One thread per body, so the scan is not a single-core long pole in
/// `test-full`.
#[test]
#[ignore = "slow: 3.6 million backend evaluations; run with --ignored"]
fn truncation_drift_stays_within_ceilings_on_the_dense_scan() {
    let ceilings = TRUNCATION_DRIFT_CEILINGS;
    let maxima: Vec<(f64, f64)> = std::thread::scope(|scope| {
        let scans: Vec<_> = ceilings
            .iter()
            .map(|ceiling| scope.spawn(move || max_drift(&ceiling.body, uniform_grid(200_000))))
            .collect();
        scans
            .into_iter()
            .map(|scan| scan.join().expect("dense scan thread"))
            .collect()
    });
    for (ceiling, maximum) in ceilings.iter().zip(maxima) {
        assert_within_ceiling(ceiling, maximum);
    }
}

#[test]
fn default_backend_is_truncated_from_the_window_start() {
    let default = Vsop87Backend::new();
    let full = Vsop87FullSeriesBackend::new();
    for jd in [TRUNCATED_WINDOW_START_JD, TRUNCATED_WINDOW_END_JD - 1e-3] {
        assert_ne!(
            mean_place(&default, CelestialBody::Mars, jd),
            mean_place(&full, CelestialBody::Mars, jd),
            "{jd}"
        );
    }
}

#[test]
fn default_backend_is_full_series_outside_the_window() {
    let default = Vsop87Backend::new();
    let full = Vsop87FullSeriesBackend::new();
    for jd in [
        TRUNCATED_WINDOW_START_JD - 1e-3,
        TRUNCATED_WINDOW_END_JD,
        625_673.5,
        3_547_272.5,
    ] {
        for body in Vsop87Backend::supported_bodies() {
            let a = full_result(&default, body.clone(), jd);
            let b = full_result(&full, body.clone(), jd);
            assert_eq!(a.ecliptic, b.ecliptic, "{body:?} ecliptic at {jd}");
            assert_eq!(a.equatorial, b.equatorial, "{body:?} equatorial at {jd}");
            assert_eq!(a.motion, b.motion, "{body:?} motion at {jd}");
        }
    }
}

/// The step at each window edge is the truncated-minus-full difference at the
/// first instant inside the window, since the default backend is bit-identical
/// to the full series just outside it
/// (`default_backend_is_full_series_outside_the_window`).
#[test]
fn edge_step_is_within_the_drift_ceiling() {
    let default = Vsop87Backend::new();
    let full = Vsop87FullSeriesBackend::new();
    for ceiling in &TRUNCATION_DRIFT_CEILINGS {
        let body = &ceiling.body;
        for inside in [TRUNCATED_WINDOW_START_JD, TRUNCATED_WINDOW_END_JD - 1e-6] {
            let truncated = mean_place(&default, body.clone(), inside);
            let reference = mean_place(&full, body.clone(), inside);
            let longitude_step = longitude_delta_arcsec(&truncated, &reference);
            let latitude_step = latitude_delta_arcsec(&truncated, &reference);
            println!("{body:?} at {inside}: step {longitude_step:.4}\", {latitude_step:.4}\"");
            assert!(
                longitude_step <= ceiling.longitude_arcsec.min(EDGE_STEP_LIMIT_ARCSEC),
                "{body:?} longitude edge step {longitude_step}\" at {inside}"
            );
            assert!(
                latitude_step <= ceiling.latitude_arcsec.min(EDGE_STEP_LIMIT_ARCSEC),
                "{body:?} latitude edge step {latitude_step}\" at {inside}"
            );
        }
    }
}

/// Both stencil points take the request's fidelity, including where one of
/// them falls outside the window: at START+0.001 the −0.005-day point is
/// before the window, at END−0.001 the +0.005-day point is after it.
#[test]
fn motion_uses_the_request_fidelity_on_both_stencil_points() {
    const HALF_SPAN_DAYS: f64 = 0.005;
    for jd in [
        TRUNCATED_WINDOW_START_JD + 0.001,
        TRUNCATED_WINDOW_END_JD - 0.001,
    ] {
        let days = jd - 2_451_545.0;
        let pluto_path = PlutoPath::for_julian_day(jd);
        let sample = |offset: f64| {
            Vsop87Backend::to_ecliptic(
                Vsop87Backend::geocentric_coordinates_on(
                    CelestialBody::Mars,
                    days + offset,
                    pluto_path,
                    SeriesFidelity::Truncated,
                )
                .expect("Mars geocentric"),
            )
        };
        let expected = signed_longitude_delta_degrees(
            sample(-HALF_SPAN_DAYS).longitude.degrees(),
            sample(HALF_SPAN_DAYS).longitude.degrees(),
        ) / (2.0 * HALF_SPAN_DAYS);

        let motion = Vsop87Backend::motion(
            CelestialBody::Mars,
            days,
            pluto_path,
            SeriesFidelity::Truncated,
        )
        .expect("Mars motion");
        let default_speed = full_result(&Vsop87Backend::new(), CelestialBody::Mars, jd)
            .motion
            .expect("default motion")
            .longitude_deg_per_day
            .expect("longitude speed");
        assert_eq!(motion.longitude_deg_per_day, Some(default_speed), "{jd}");
        assert!(
            (default_speed - expected).abs() <= 1e-9,
            "default speed {default_speed} vs truncated stencil {expected} at {jd}"
        );
    }
}

#[test]
fn full_series_backend_reports_exact_and_default_reports_approximate_inside_the_window() {
    let quality = |backend: &dyn EphemerisBackend, jd: f64| {
        backend
            .position_without_motion(&tt_request(CelestialBody::Mars, jd))
            .expect("Mars position")
            .quality
    };
    let default = Vsop87Backend::new();
    let full = Vsop87FullSeriesBackend::new();
    assert_eq!(
        quality(&default, 2_451_545.0),
        QualityAnnotation::Approximate
    );
    assert_eq!(quality(&full, 2_451_545.0), QualityAnnotation::Exact);
    assert_eq!(quality(&default, 625_673.5), QualityAnnotation::Exact);
    assert_eq!(quality(&full, 625_673.5), QualityAnnotation::Exact);
}

#[test]
fn full_series_backend_metadata_matches_the_default_except_the_truncation_note() {
    const NOTE: &str = " Inside 1600-01-01..2600-01-01 (TT or TDB Julian day) the VSOP87B series are truncated (terms with |A|·0.6^α < 1e-8 dropped, measured within 0.3″ of the full series); outside it the full series is evaluated.";
    let mut default = Vsop87Backend::new().metadata();
    let full = Vsop87FullSeriesBackend::new().metadata();
    assert_eq!(
        default.provenance.summary,
        format!("{}{NOTE}", full.provenance.summary)
    );
    default.provenance.summary = full.provenance.summary.clone();
    assert_eq!(default, full);
}
