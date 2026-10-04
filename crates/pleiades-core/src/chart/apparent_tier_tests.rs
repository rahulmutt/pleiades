//! Apparent place is a frame reduction, not an accuracy claim: every body a
//! backend serves is reduced to the true equinox of date under an `Apparent`
//! request, whatever its `BodyClaimTier` (issue #113). Before this, only
//! `ReleaseGrade` bodies were reduced and every other body was returned in the
//! backend's mean J2000 frame while the snapshot still reported `Apparent`.

use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, ClaimEvidence, CompositeBackend, CoordinateFrame,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    QualityAnnotation, ZodiacMode,
};
use pleiades_elp::ElpBackend;
use pleiades_types::{
    CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, TimeRange,
    TimeScale,
};
use pleiades_vsop87::Vsop87Backend;

use super::test_support::ConstrainedOnlyChartBackend;
use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// 2025-03-29 00:00 TT, the instant of the issue's reproduction.
const ISSUE_113_JD_TT: f64 = 2_460_763.5;

/// JPL Horizons DE441 apparent ecliptic longitude of date for Saturn at
/// [`ISSUE_113_JD_TT`], quoted in issue #113.
const HORIZONS_SATURN_APPARENT_LONGITUDE_DEG: f64 = 354.1280;

/// Accumulated J2000 -> 2025 precession is about 0.3526 deg; nutation and
/// aberration add at most about 25 arcseconds on top of it.
const PRECESSION_2025_DEG_RANGE: (f64, f64) = (0.30, 0.40);

fn composite_backend() -> CompositeBackend<Vsop87Backend, ElpBackend> {
    CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())
}

fn issue_instant() -> Instant {
    Instant::new(JulianDay::from_days(ISSUE_113_JD_TT), TimeScale::Tt)
}

fn issue_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Saturn,
    ]
}

fn mean_j2000_longitude_deg(
    backend: &impl EphemerisBackend,
    body: &CelestialBody,
    instant: Instant,
) -> f64 {
    let request = EphemerisRequest {
        body: body.clone(),
        instant,
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    backend
        .position(&request)
        .expect("backend serves the body")
        .ecliptic
        .expect("ecliptic coordinates")
        .longitude
        .degrees()
}

fn longitude_deg(snapshot: &ChartSnapshot, body: &CelestialBody) -> f64 {
    snapshot
        .placement_for(body)
        .expect("body is placed")
        .position
        .ecliptic
        .expect("ecliptic coordinates")
        .longitude
        .degrees()
}

fn signed_difference_deg(a: f64, b: f64) -> f64 {
    let mut d = (a - b).rem_euclid(360.0);
    if d > 180.0 {
        d -= 360.0;
    }
    d
}

#[test]
fn constrained_body_receives_apparent_place() {
    // The fixture claims the Moon `Constrained`; the apparent pipeline must
    // still run for it and attach provenance, instead of leaving the backend's
    // J2000 place untouched.
    let engine = ChartEngine::new(ConstrainedOnlyChartBackend);
    let request = ChartRequest::new(Instant::new(
        JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Moon]);

    let snapshot = engine
        .chart(&request)
        .expect("a constrained body is reduced to apparent place, not an error");
    let placement = snapshot.placement_for(&CelestialBody::Moon).unwrap();

    assert_eq!(placement.position.apparent, Apparentness::Apparent);
    let provenance = placement
        .apparent
        .as_ref()
        .expect("apparent provenance is attached to a constrained body");
    assert!(provenance.corrections.precession);
    assert!(provenance.corrections.nutation_longitude);
    assert!(provenance.corrections.light_time);
}

#[test]
fn issue_113_default_chart_on_vsop87_elp_composite_is_apparent_of_date() {
    let backend = composite_backend();
    let engine = ChartEngine::new(composite_backend());
    let request = ChartRequest::new(issue_instant()).with_bodies(issue_bodies());
    assert_eq!(request.apparentness, Apparentness::Apparent);

    let snapshot = engine.chart(&request).expect("default chart succeeds");
    assert_eq!(snapshot.apparentness, Apparentness::Apparent);

    for body in issue_bodies() {
        let placement = snapshot.placement_for(&body).unwrap();
        assert_eq!(
            placement.position.apparent,
            Apparentness::Apparent,
            "{body}: placement must be apparent, not mean J2000"
        );
        assert!(
            placement.apparent.is_some(),
            "{body}: apparent provenance must be attached"
        );

        let mean = mean_j2000_longitude_deg(&backend, &body, issue_instant());
        let delta = signed_difference_deg(longitude_deg(&snapshot, &body), mean);
        let (lo, hi) = PRECESSION_2025_DEG_RANGE;
        assert!(
            (lo..=hi).contains(&delta),
            "{body}: apparent minus mean J2000 is {delta:.4} deg; expected the \
             2025 precession of about 0.3526 deg plus nutation and aberration"
        );
    }

    // Issue #113 measured the engine's Saturn 0.3455 deg below Horizons; the
    // reduced place must land within the algorithmic theory's own error, which
    // the issue's probe put at 0.007 deg after precession alone.
    let saturn = longitude_deg(&snapshot, &CelestialBody::Saturn);
    let residual = signed_difference_deg(saturn, HORIZONS_SATURN_APPARENT_LONGITUDE_DEG);
    assert!(
        residual.abs() < 0.03,
        "Saturn apparent longitude {saturn:.4} deg is {residual:+.4} deg from Horizons"
    );
}

#[test]
fn issue_113_sidereal_chart_on_composite_subtracts_ayanamsa_from_apparent_place() {
    let engine = ChartEngine::new(composite_backend());
    let tropical = engine
        .chart(&ChartRequest::new(issue_instant()).with_bodies(vec![CelestialBody::Saturn]))
        .expect("tropical chart succeeds");
    let sidereal = engine
        .chart(
            &ChartRequest::new(issue_instant())
                .with_bodies(vec![CelestialBody::Saturn])
                .with_zodiac_mode(ZodiacMode::Sidereal {
                    ayanamsa: crate::Ayanamsa::Lahiri,
                }),
        )
        .expect("sidereal chart succeeds");

    let placement = sidereal.placement_for(&CelestialBody::Saturn).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Apparent);
    assert!(placement.apparent.is_some());

    // Lahiri in 2025 is about 24.2 deg; the sidereal longitude is the apparent
    // tropical longitude minus that offset, not the J2000 longitude minus it.
    let offset = signed_difference_deg(
        longitude_deg(&tropical, &CelestialBody::Saturn),
        longitude_deg(&sidereal, &CelestialBody::Saturn),
    );
    assert!(
        (24.1..=24.4).contains(&offset),
        "tropical minus sidereal Saturn is {offset:.4} deg; expected the Lahiri ayanamsa"
    );
}

/// Serves the Moon only: a backend that cannot supply the Sun the apparent
/// pipeline needs for its aberration term.
struct MoonOnlyChartBackend;

impl EphemerisBackend for MoonOnlyChartBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("moon-only-chart"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("moon-only chart backend"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![BodyClaim::constrained(
                CelestialBody::Moon,
                AccuracyClass::Approximate,
                ClaimEvidence::AlgorithmicModel,
            )],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Moon)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if request.body != CelestialBody::Moon {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "moon-only chart backend serves the Moon only",
            ));
        }
        let mut result = EphemerisResult::new(
            BackendId::new("moon-only-chart"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(45.0),
            Latitude::from_degrees(0.0),
            Some(0.0026),
        ));
        Ok(result)
    }
}

#[test]
fn apparent_chart_needs_no_sun_from_the_backend() {
    // The Sun's longitude feeds only the provenance's aberration estimate,
    // which the backend-free Meeus Sun now serves (issue #128), so a backend
    // without a Sun can still serve an apparent chart.
    let engine = ChartEngine::new(MoonOnlyChartBackend);
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);

    let snapshot = engine
        .chart(&ChartRequest::new(instant).with_bodies(vec![CelestialBody::Moon]))
        .expect("an apparent chart needs no Sun source");
    let placement = snapshot.placement_for(&CelestialBody::Moon).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Apparent);
    assert!(placement.apparent.is_some());

    // The explicit mean request still works and is reported as mean.
    let snapshot = engine
        .chart(
            &ChartRequest::new(instant)
                .with_bodies(vec![CelestialBody::Moon])
                .with_apparentness(Apparentness::Mean),
        )
        .expect("mean chart succeeds without a Sun source");
    assert_eq!(snapshot.apparentness, Apparentness::Mean);
    let placement = snapshot.placement_for(&CelestialBody::Moon).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Mean);
    assert!(placement.apparent.is_none());
}

/// Swiss Ephemeris 2.10 (Moshier) apparent ecliptic place of date for Pluto,
/// quoted in issue #119: `(JD TT, longitude deg, latitude deg, distance AU)`.
const SWISS_EPHEMERIS_PLUTO_APPARENT: [(f64, f64, f64, f64); 3] = [
    (2_451_545.0, 251.4547, 10.8552, 31.064),
    (2_460_763.5, 303.5051, -3.4414, 35.639),
    (2_444_405.5, 199.0216, 17.3892, 29.718),
];

#[test]
fn issue_119_pluto_apparent_place_on_composite_is_within_a_degree_of_swiss_ephemeris() {
    // Issue #119 measured the composite's Pluto 110-114 deg from Swiss
    // Ephemeris at every epoch tried, in the wrong sign. Mean Keplerian
    // elements deliver about a degree over the 20th-21st centuries, which is
    // what the backend's `Approximate` claim promises.
    let engine = ChartEngine::new(composite_backend());
    for (jd_tt, lon, lat, dist) in SWISS_EPHEMERIS_PLUTO_APPARENT {
        let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
        let snapshot = engine
            .chart(&ChartRequest::new(instant).with_bodies(vec![CelestialBody::Pluto]))
            .expect("apparent Pluto chart succeeds");
        let ecliptic = snapshot
            .placement_for(&CelestialBody::Pluto)
            .expect("Pluto is placed")
            .position
            .ecliptic
            .expect("ecliptic coordinates");
        let residual = signed_difference_deg(ecliptic.longitude.degrees(), lon);
        assert!(
            residual.abs() < 1.0,
            "JD {jd_tt}: Pluto apparent longitude {:.4} deg is {residual:+.4} deg from Swiss Ephemeris",
            ecliptic.longitude.degrees()
        );
        assert!(
            (ecliptic.latitude.degrees() - lat).abs() < 1.0,
            "JD {jd_tt}: Pluto apparent latitude {:.4} deg vs Swiss Ephemeris {lat:.4}",
            ecliptic.latitude.degrees()
        );
        let distance = ecliptic.distance_au.expect("Pluto carries a distance");
        assert!(
            (distance - dist).abs() < 0.5,
            "JD {jd_tt}: Pluto distance {distance:.3} AU vs Swiss Ephemeris {dist:.3}"
        );
    }
}
