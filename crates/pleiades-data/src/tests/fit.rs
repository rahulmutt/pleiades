use crate::test_support::*;
use crate::*;

// ---------------------------------------------------------------------------
// Per-body fixed-ecliptic backend for heliocentric-frame tests
// ---------------------------------------------------------------------------

/// A test backend that returns a fixed geocentric ecliptic position for each
/// registered body, regardless of the queried instant.
struct FixedEclipticBackend {
    coords: std::collections::HashMap<CelestialBody, (f64, f64, f64)>,
}

impl FixedEclipticBackend {
    fn new() -> Self {
        Self {
            coords: std::collections::HashMap::new(),
        }
    }

    fn with(mut self, body: CelestialBody, lon: f64, lat: f64, au: f64) -> Self {
        self.coords.insert(body, (lon, lat, au));
        self
    }
}

impl pleiades_backend::EphemerisBackend for FixedEclipticBackend {
    fn metadata(&self) -> pleiades_backend::BackendMetadata {
        unimplemented!()
    }

    fn supports_body(&self, _body: pleiades_backend::CelestialBody) -> bool {
        true
    }

    fn position(
        &self,
        req: &pleiades_backend::EphemerisRequest,
    ) -> Result<pleiades_backend::EphemerisResult, pleiades_backend::EphemerisError> {
        let (lon, lat, dist) = *self.coords.get(&req.body).unwrap_or_else(|| {
            panic!(
                "FixedEclipticBackend: no coordinates registered for body {:?}",
                req.body
            )
        });
        let mut r = pleiades_backend::EphemerisResult::new(
            pleiades_backend::BackendId::new("fixed"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        r.ecliptic = Some(ecliptic(lon, lat, dist));
        Ok(r)
    }
}

/// Build an [`EclipticCoordinates`] from plain degree/AU values (local helper).
fn ecliptic(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
    EclipticCoordinates::new(
        pleiades_backend::Longitude::from_degrees(lon),
        pleiades_backend::Latitude::from_degrees(lat),
        Some(dist),
    )
}

#[test]
fn packaged_artifact_fit_outlier_sample_fractions_track_the_validation_lattice() {
    let artifact = packaged_artifact();
    let moon_segment = artifact
        .bodies
        .iter()
        .find(|body| body.body == CelestialBody::Moon)
        .and_then(|body| {
            body.segments
                .iter()
                .find(|segment| segment.start.julian_day.days() != segment.end.julian_day.days())
                .map(|segment| (&body.body, segment))
        })
        .expect("packaged artifact should include at least one multi-day Moon segment");
    let mercury_segment = artifact
        .bodies
        .iter()
        .find(|body| body.body == CelestialBody::Mercury)
        .and_then(|body| {
            body.segments
                .iter()
                .find(|segment| segment.start.julian_day.days() != segment.end.julian_day.days())
                .map(|segment| (&body.body, segment))
        })
        .expect("packaged artifact should include at least one multi-day Mercury segment");
    let saturn_segment = artifact
        .bodies
        .iter()
        .find(|body| body.body == CelestialBody::Saturn)
        .and_then(|body| {
            body.segments
                .iter()
                .find(|segment| segment.start.julian_day.days() != segment.end.julian_day.days())
                .map(|segment| (&body.body, segment))
        })
        .expect("packaged artifact should include at least one multi-day Saturn segment");
    let lunar_point_body = CelestialBody::MeanNode;
    let custom_segment = artifact
        .bodies
        .iter()
        .find(|body| matches!(body.body, CelestialBody::Custom(_)))
        .and_then(|body| {
            body.segments
                .iter()
                .find(|segment| segment.start.julian_day.days() != segment.end.julian_day.days())
                .map(|segment| (&body.body, segment))
        })
        .expect("packaged artifact should include at least one multi-day custom-body segment");

    assert_eq!(
        packaged_artifact_fit_sample_fractions(moon_segment.1),
        &[0.25, 0.5, 0.75]
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(moon_segment.0, moon_segment.1),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(moon_segment.0, moon_segment.1),
        packaged_artifact_fit_outlier_sample_fractions(moon_segment.0, moon_segment.1)
    );
    assert_eq!(
        packaged_artifact_fit_outlier_sample_fractions(moon_segment.0, moon_segment.1),
        &[0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]
    );
    assert_eq!(
        packaged_artifact_segment_validation_fractions_for_body(mercury_segment.0),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(mercury_segment.0, mercury_segment.1),
        PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_outlier_sample_fractions(mercury_segment.0, mercury_segment.1),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(saturn_segment.0, saturn_segment.1),
        PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_segment_validation_fractions_for_body(saturn_segment.0),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_outlier_sample_fractions(saturn_segment.0, saturn_segment.1),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_segment_validation_fractions_for_body(&lunar_point_body),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_segment_validation_fractions_for_body(&CelestialBody::Pluto),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(&lunar_point_body, moon_segment.1),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(&lunar_point_body, moon_segment.1),
        packaged_artifact_fit_outlier_sample_fractions(&lunar_point_body, moon_segment.1)
    );
    assert_eq!(
        packaged_artifact_fit_outlier_sample_fractions(&lunar_point_body, moon_segment.1),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    for lunar_point in [
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::TrueApogee,
        CelestialBody::MeanPerigee,
        CelestialBody::TruePerigee,
    ] {
        assert_eq!(
            packaged_artifact_segment_validation_fractions_for_body(&lunar_point),
            PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
        );
    }
    assert_eq!(
        packaged_artifact_segment_validation_fractions_for_body(custom_segment.0),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(custom_segment.0, custom_segment.1),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_fit_sample_fractions_for_body(custom_segment.0, custom_segment.1),
        packaged_artifact_fit_outlier_sample_fractions(custom_segment.0, custom_segment.1)
    );
    assert_eq!(
        packaged_artifact_fit_outlier_sample_fractions(custom_segment.0, custom_segment.1),
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    );
}

#[test]
fn packaged_artifact_outer_planets_use_dense_distance_validation() {
    let sample_segment = Segment::new(instant_tt(2_451_545.0), instant_tt(2_451_555.0), Vec::new());

    for body in [
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
    ] {
        assert!(packaged_artifact_body_cadence(&body).uses_dense_validation_sampling());
        assert_eq!(
            packaged_artifact_fit_sample_fractions_for_body(&body, &sample_segment),
            PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS
        );
        assert_eq!(
            packaged_artifact_fit_outlier_sample_fractions(&body, &sample_segment),
            PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
        );
        assert_eq!(
            packaged_artifact_segment_validation_fractions_for_body(&body),
            PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
        );
    }
}

#[test]
fn packaged_artifact_body_cadence_distinguishes_custom_asteroid_and_custom_body_catalogs() {
    let custom_asteroid = CelestialBody::Custom(CustomBodyId::new("ASTEROID", "99942-Apophis"));
    let custom_comet = CelestialBody::Custom(CustomBodyId::new("comet", "1P-Halley"));

    assert!(matches!(
        packaged_artifact_body_cadence(&custom_asteroid),
        PackagedArtifactBodyCadence::SelectedAsteroids
    ));
    assert_eq!(body_segment_span_limit(&custom_asteroid), 256.0);
    assert!(matches!(
        packaged_artifact_body_cadence(&custom_comet),
        PackagedArtifactBodyCadence::CustomBodies
    ));
    assert_eq!(body_segment_span_limit(&custom_comet), 512.0);
}

#[test]
fn planet_segment_is_fit_in_heliocentric_frame() {
    use pleiades_compression::heliocentric_from_geocentric;

    // Synthetic backend: Jupiter at fixed geocentric ecliptic, Sun at fixed geocentric ecliptic.
    let backend = FixedEclipticBackend::new()
        .with(
            CelestialBody::Jupiter,
            /*lon*/ 200.0,
            /*lat*/ 1.2,
            /*au*/ 5.4,
        )
        .with(CelestialBody::Sun, 95.0, 0.0, 1.0);

    let seg = crate::regenerate::fit_segment_within_span(
        &CelestialBody::Jupiter,
        2_451_545.0,
        2_451_545.0 + 30.0,
        &backend,
    )
    .expect("segment should fit");

    // Stored longitude (degree-0/constant for a constant source) must equal the
    // HELIOCENTRIC longitude, not the geocentric 200.0.
    let stored_lon = seg
        .channels
        .iter()
        .find(|c| c.kind == pleiades_compression::ChannelKind::Longitude)
        .unwrap()
        .coefficients[0];

    let expected =
        heliocentric_from_geocentric(&ecliptic(200.0, 1.2, 5.4), &ecliptic(95.0, 0.0, 1.0))
            .unwrap();
    assert!((stored_lon - expected.longitude.degrees()).abs() < 1e-6);
    assert!(
        (stored_lon - 200.0).abs() > 1.0,
        "must not store geocentric longitude"
    );

    // Sun and Moon stay geocentric — the reframe predicate must exclude them.
    assert!(!crate::regenerate::body_uses_heliocentric_frame(
        &CelestialBody::Sun
    ));
    assert!(!crate::regenerate::body_uses_heliocentric_frame(
        &CelestialBody::Moon
    ));
}
