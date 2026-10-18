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

/// Per-body ceilings on |Δλ| and |Δβ| (arcseconds) between the default and
/// the full-series backend over 1600–2600 CE: `min(0.30, ⌈1.25 × max⌉₀.₀₁)`
/// of the maxima measured on the 2,000-instant grid below.
const TRUNCATION_DRIFT_CEILINGS_ARCSEC: [(CelestialBody, f64, f64); 9] = [
    // Measured maxima (lon, lat): 0.0366, 0.0237.
    (CelestialBody::Sun, 0.05, 0.03),
    // 0.0542, 0.0389.
    (CelestialBody::Mercury, 0.07, 0.05),
    // 0.1126, 0.0942.
    (CelestialBody::Venus, 0.15, 0.12),
    // 0.1886, 0.0997.
    (CelestialBody::Mars, 0.24, 0.13),
    // 0.0571, 0.0318.
    (CelestialBody::Jupiter, 0.08, 0.04),
    // 0.0693, 0.0363.
    (CelestialBody::Saturn, 0.09, 0.05),
    // 0.0619, 0.0344.
    (CelestialBody::Uranus, 0.08, 0.05),
    // 0.0350, 0.0180.
    (CelestialBody::Neptune, 0.05, 0.03),
    // 0.0010, 0.0009: only the VSOP87B Earth subtracted from Pluto is truncated.
    (CelestialBody::Pluto, 0.01, 0.01),
];

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

#[test]
fn truncation_drift_stays_within_ceilings_over_1600_2600() {
    const SAMPLES: usize = 2_000;
    let span = TRUNCATED_WINDOW_END_JD - TRUNCATED_WINDOW_START_JD;
    let default = Vsop87Backend::new();
    let full = Vsop87FullSeriesBackend::new();
    for (body, longitude_ceiling, latitude_ceiling) in TRUNCATION_DRIFT_CEILINGS_ARCSEC {
        let (mut max_lon, mut max_lat) = (0.0_f64, 0.0_f64);
        for i in 0..SAMPLES {
            let jitter = (i as f64 * 0.618_034).fract();
            let jd = TRUNCATED_WINDOW_START_JD + (i as f64 + jitter) * span / SAMPLES as f64;
            let truncated = mean_place(&default, body.clone(), jd);
            let reference = mean_place(&full, body.clone(), jd);
            max_lon = max_lon.max(longitude_delta_arcsec(&truncated, &reference));
            max_lat = max_lat.max(latitude_delta_arcsec(&truncated, &reference));
        }
        println!("{body:?}: max |dlon| = {max_lon:.4}\", max |dlat| = {max_lat:.4}\"");
        assert!(
            max_lon <= longitude_ceiling,
            "{body:?} longitude drift {max_lon}\" over ceiling {longitude_ceiling}\""
        );
        assert!(
            max_lat <= latitude_ceiling,
            "{body:?} latitude drift {max_lat}\" over ceiling {latitude_ceiling}\""
        );
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

#[test]
fn edge_step_is_within_the_drift_ceiling() {
    let default = Vsop87Backend::new();
    for (body, longitude_ceiling, _) in TRUNCATION_DRIFT_CEILINGS_ARCSEC {
        for (outside, inside) in [
            (TRUNCATED_WINDOW_START_JD - 1e-6, TRUNCATED_WINDOW_START_JD),
            (TRUNCATED_WINDOW_END_JD, TRUNCATED_WINDOW_END_JD - 1e-6),
        ] {
            let step = longitude_delta_arcsec(
                &mean_place(&default, body.clone(), outside),
                &mean_place(&default, body.clone(), inside),
            );
            assert!(
                step <= longitude_ceiling + 1e-4,
                "{body:?} edge step {step}\" at {inside}"
            );
        }
    }
}

#[test]
fn motion_uses_the_request_fidelity_on_both_stencil_points() {
    const HALF_SPAN_DAYS: f64 = 0.005;
    let jd = TRUNCATED_WINDOW_START_JD + 0.001;
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
    assert_eq!(motion.longitude_deg_per_day, Some(default_speed));
    assert!(
        (default_speed - expected).abs() <= 1e-9,
        "default speed {default_speed} vs truncated stencil {expected}"
    );
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
    const NOTE: &str = " Inside 1600-01-01..2600-01-01 TT the VSOP87B series are truncated (terms with |A|·0.6^α < 1e-8 dropped, at most 0.3″ from the full series); outside it the full series is evaluated.";
    let mut default = Vsop87Backend::new().metadata();
    let full = Vsop87FullSeriesBackend::new().metadata();
    assert_eq!(
        default.provenance.summary,
        format!("{}{NOTE}", full.provenance.summary)
    );
    default.provenance.summary = full.provenance.summary.clone();
    assert_eq!(default, full);
}
