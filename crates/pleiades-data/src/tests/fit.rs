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
fn polynomial_channel_from_samples_supports_chebyshev_lobatto_fits() {
    let fractions = chebyshev_lobatto_fractions(6);
    assert_eq!(fractions.len(), 6);
    assert_eq!(fractions.first().copied(), Some(0.0));
    assert_eq!(fractions.last().copied(), Some(1.0));
    let expected_fractions = [
        0.0,
        0.095_491_502_812_526_27,
        0.345_491_502_812_526_3,
        0.654_508_497_187_473_7,
        0.904_508_497_187_473_7,
        1.0,
    ];
    for (actual, expected) in fractions.iter().zip(expected_fractions) {
        assert!((actual - expected).abs() < 1e-12);
    }

    let samples = fractions
        .iter()
        .copied()
        .map(|fraction| {
            let value = 1.0 + 2.0 * fraction - 3.0 * fraction.powi(2) + 4.0 * fraction.powi(3)
                - 5.0 * fraction.powi(4)
                + 6.0 * fraction.powi(5);
            (fraction, value)
        })
        .collect::<Vec<_>>();
    let channel = polynomial_channel_from_samples(ChannelKind::Longitude, 9, &samples)
        .expect("six-point fit should succeed");

    assert_eq!(channel.coefficients.len(), 6);
    let expected_coefficients = [1.0, 2.0, -3.0, 4.0, -5.0, 6.0];
    for (actual, expected) in channel.coefficients.iter().zip(expected_coefficients) {
        assert!((actual - expected).abs() < 1e-9);
    }
}

#[test]
fn distance_channel_from_samples_uses_midpoint_quadratic_reconstruction() {
    let channel = distance_channel_from_samples(1.0, Some(2.0), 3.0);
    assert_eq!(channel.coefficients.len(), 3);
    assert!((evaluate_polynomial_channel(&channel, 0.0) - 1.0).abs() < 1e-12);
    assert!((evaluate_polynomial_channel(&channel, 0.5) - 2.0).abs() < 1e-12);
    assert!((evaluate_polynomial_channel(&channel, 1.0) - 3.0).abs() < 1e-12);

    let linear = distance_channel_from_samples(1.0, None, 3.0);
    assert_eq!(linear.coefficients.len(), 2);
    assert!((evaluate_polynomial_channel(&linear, 0.5) - 2.0).abs() < 1e-12);
}

#[test]
fn distance_channel_from_fit_samples_supports_cubic_reconstruction() {
    let samples = [0.0_f64, 1.0_f64 / 3.0_f64, 2.0_f64 / 3.0_f64, 1.0_f64]
        .iter()
        .copied()
        .map(|fraction| {
            let value = 1.0 + 2.0 * fraction - 3.0 * fraction.powi(2) + 4.0 * fraction.powi(3);
            (fraction, value)
        })
        .collect::<Vec<_>>();
    let channel = distance_channel_from_fit_samples(&samples, 1.0, Some(2.0), 3.0);

    assert_eq!(channel.coefficients.len(), 4);
    let expected_coefficients = [1.0, 2.0, -3.0, 4.0];
    for (actual, expected) in channel.coefficients.iter().zip(expected_coefficients) {
        assert!((actual - expected).abs() < 1e-9);
    }
}

#[test]
fn distance_channel_from_fit_samples_prefers_four_point_control_points_when_needed() {
    let cubic =
        |fraction: f64| 1.0 + 2.0 * fraction - 3.0 * fraction.powi(2) + 4.0 * fraction.powi(3);
    let samples = [
        (0.0, cubic(0.0)),
        (0.1, 1.0e20),
        (0.3, cubic(0.3)),
        (0.7, cubic(0.7)),
        (0.9, -1.0e20),
        (1.0, cubic(1.0)),
    ];
    let channel =
        distance_channel_from_fit_samples(&samples, cubic(0.0), Some(cubic(0.5)), cubic(1.0));

    assert_eq!(channel.coefficients.len(), 4);
    let expected_coefficients = [1.0, 2.0, -3.0, 4.0];
    for (actual, expected) in channel.coefficients.iter().zip(expected_coefficients) {
        assert!((actual - expected).abs() < 1e-9);
    }
}

#[test]
fn segment_from_pair_fallback_can_use_dense_quarter_point_samples() {
    let longitude =
        |fraction: f64| 10.0 + 2.0 * fraction - 3.0 * fraction.powi(2) + 4.0 * fraction.powi(3);
    let latitude = |fraction: f64| -5.0 + fraction + 2.0 * fraction.powi(2) - fraction.powi(3);
    let distance =
        |fraction: f64| 1.0 + 0.5 * fraction - 0.25 * fraction.powi(2) + 0.125 * fraction.powi(3);
    let start_coordinates = ecl(longitude(0.0), latitude(0.0), distance(0.0));
    let end_coordinates = ecl(longitude(1.0), latitude(1.0), distance(1.0));
    let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
        if (fraction - 0.5).abs() < f64::EPSILON {
            return Some(ecl(1.0e20, -1.0e20, 1.0e20));
        }

        Some(ecl(
            longitude(fraction),
            latitude(fraction),
            distance(fraction),
        ))
    };
    let segment = segment_from_pair_fallback(
        instant_tt(0.0),
        instant_tt(1.0),
        longitude(0.0),
        longitude(1.0),
        &start_coordinates,
        &end_coordinates,
        Some(1.0),
        Some(1.0),
        &sample_fraction,
    );

    for fraction in [0.25, 0.5, 0.75] {
        let actual_longitude = segment_channel_value(&segment, ChannelKind::Longitude, fraction)
            .expect("longitude channel should evaluate");
        let actual_latitude = segment_channel_value(&segment, ChannelKind::Latitude, fraction)
            .expect("latitude channel should evaluate");
        let actual_distance = segment_channel_value(&segment, ChannelKind::DistanceAu, fraction)
            .expect("distance channel should evaluate");

        assert!(
            (actual_longitude - longitude(fraction)).abs() < 1e-9,
            "longitude mismatch at fraction {fraction}: {actual_longitude} vs {}",
            longitude(fraction)
        );
        assert!(
            (actual_latitude - latitude(fraction)).abs() < 1e-9,
            "latitude mismatch at fraction {fraction}: {actual_latitude} vs {}",
            latitude(fraction)
        );
        assert!(
            (actual_distance - distance(fraction)).abs() < 1e-9,
            "distance mismatch at fraction {fraction}: {actual_distance} vs {}",
            distance(fraction)
        );
    }
}

#[test]
fn segment_from_pair_fallback_can_use_dense_five_point_samples_on_long_spans() {
    let longitude = |fraction: f64| {
        15.0 - 3.0 * fraction + 2.0 * fraction.powi(2) - fraction.powi(3) + 0.5 * fraction.powi(4)
            - 0.25 * fraction.powi(5)
    };
    let latitude = |fraction: f64| {
        -2.0 + 4.0 * fraction - 1.5 * fraction.powi(2) + 0.75 * fraction.powi(3)
            - 0.5 * fraction.powi(4)
            + 0.125 * fraction.powi(5)
    };
    let distance = |fraction: f64| {
        3.0 + 0.25 * fraction + 0.5 * fraction.powi(2) - 0.125 * fraction.powi(3)
            + 0.0625 * fraction.powi(4)
            - 0.03125 * fraction.powi(5)
    };
    let start_coordinates = ecl(longitude(0.0), latitude(0.0), distance(0.0));
    let end_coordinates = ecl(longitude(1.0), latitude(1.0), distance(1.0));
    let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
        if (fraction - 0.25).abs() < f64::EPSILON || (fraction - 0.75).abs() < f64::EPSILON {
            return None;
        }

        Some(ecl(
            longitude(fraction),
            latitude(fraction),
            distance(fraction),
        ))
    };
    let segment = segment_from_pair_fallback(
        instant_tt(0.0),
        instant_tt(13_000.0),
        longitude(0.0),
        longitude(1.0),
        &start_coordinates,
        &end_coordinates,
        Some(13_000.0),
        Some(1_536.0),
        &sample_fraction,
    );

    for fraction in [0.2, 0.4, 0.5, 0.6, 0.8] {
        let actual_longitude = segment_channel_value(&segment, ChannelKind::Longitude, fraction)
            .expect("longitude channel should evaluate");
        let actual_latitude = segment_channel_value(&segment, ChannelKind::Latitude, fraction)
            .expect("latitude channel should evaluate");
        let actual_distance = segment_channel_value(&segment, ChannelKind::DistanceAu, fraction)
            .expect("distance channel should evaluate");

        assert!(
            (actual_longitude - longitude(fraction)).abs() < 1e-8,
            "longitude mismatch at fraction {fraction}: {actual_longitude} vs {}",
            longitude(fraction)
        );
        assert!(
            (actual_latitude - latitude(fraction)).abs() < 1e-8,
            "latitude mismatch at fraction {fraction}: {actual_latitude} vs {}",
            latitude(fraction)
        );
        assert!(
            (actual_distance - distance(fraction)).abs() < 1e-8,
            "distance mismatch at fraction {fraction}: {actual_distance} vs {}",
            distance(fraction)
        );
    }
}

#[test]
fn segment_from_pair_fallback_can_use_dense_seven_point_samples_on_super_extreme_spans() {
    let longitude =
        |fraction: f64| 8.0 + 1.25 * fraction - 0.5 * fraction.powi(2) + 0.125 * fraction.powi(3);
    let latitude = |fraction: f64| {
        -3.0 + 0.75 * fraction + 0.25 * fraction.powi(2) - 0.0625 * fraction.powi(3)
    };
    let distance = |fraction: f64| {
        2.0 + 0.5 * fraction - 0.125 * fraction.powi(2) + 0.03125 * fraction.powi(3)
    };
    let start_coordinates = ecl(longitude(0.0), latitude(0.0), distance(0.0));
    let end_coordinates = ecl(longitude(1.0), latitude(1.0), distance(1.0));
    let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
        if (fraction - 0.2).abs() < f64::EPSILON
            || (fraction - 0.4).abs() < f64::EPSILON
            || (fraction - 0.6).abs() < f64::EPSILON
            || (fraction - 0.8).abs() < f64::EPSILON
        {
            return None;
        }

        Some(ecl(
            longitude(fraction),
            latitude(fraction),
            distance(fraction),
        ))
    };
    let segment = segment_from_pair_fallback(
        instant_tt(0.0),
        instant_tt(50_000.0),
        longitude(0.0),
        longitude(1.0),
        &start_coordinates,
        &end_coordinates,
        Some(50_000.0),
        Some(1_536.0),
        &sample_fraction,
    );

    for fraction in [
        1.0 / 7.0,
        2.0 / 7.0,
        3.0 / 7.0,
        4.0 / 7.0,
        5.0 / 7.0,
        6.0 / 7.0,
    ] {
        let actual_longitude = segment_channel_value(&segment, ChannelKind::Longitude, fraction)
            .expect("longitude channel should evaluate");
        let actual_latitude = segment_channel_value(&segment, ChannelKind::Latitude, fraction)
            .expect("latitude channel should evaluate");
        let actual_distance = segment_channel_value(&segment, ChannelKind::DistanceAu, fraction)
            .expect("distance channel should evaluate");

        assert!(
            (actual_longitude - longitude(fraction)).abs() < 1e-8,
            "longitude mismatch at fraction {fraction}: {actual_longitude} vs {}",
            longitude(fraction)
        );
        assert!(
            (actual_latitude - latitude(fraction)).abs() < 1e-8,
            "latitude mismatch at fraction {fraction}: {actual_latitude} vs {}",
            latitude(fraction)
        );
        assert!(
            (actual_distance - distance(fraction)).abs() < 1e-8,
            "distance mismatch at fraction {fraction}: {actual_distance} vs {}",
            distance(fraction)
        );
    }
}

#[test]
fn channel_from_fit_samples_with_control_points_falls_back_when_higher_order_fit_overflows() {
    let samples = [
        (0.0, 0.0),
        (0.2, 1.0e20),
        (0.4, 0.0),
        (0.6, 0.0),
        (0.8, 1.0e20),
        (1.0, 0.0),
    ];
    let channel = channel_from_fit_samples_with_control_points(ChannelKind::Latitude, 0, &samples)
        .expect("control-point fallback should succeed");

    assert_eq!(channel.coefficients.len(), 4);
    for coefficient in &channel.coefficients {
        assert!(coefficient.abs() < 1e-12);
    }
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
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(moon_segment.0),
        PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(&CelestialBody::Pluto),
        PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(&CelestialBody::Ceres),
        PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(&lunar_point_body),
        PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(custom_segment.0),
        PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS
    );
    for lunar_point in [
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::TrueApogee,
        CelestialBody::MeanPerigee,
        CelestialBody::TruePerigee,
    ] {
        assert!(packaged_artifact_body_cadence(&lunar_point).uses_dense_sampling());
        assert_eq!(
            packaged_artifact_fit_sample_counts_for_body(&lunar_point),
            PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS
        );
        assert_eq!(
            packaged_artifact_segment_validation_fractions_for_body(&lunar_point),
            PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
        );
        assert_eq!(
            packaged_artifact_residual_sample_fractions_for_channel(
                &lunar_point,
                ChannelKind::Longitude,
            ),
            PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
        );
        assert_eq!(
            packaged_artifact_residual_sample_fractions_for_channel(
                &lunar_point,
                ChannelKind::DistanceAu,
            ),
            PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
        );
    }
    assert!(packaged_artifact_body_cadence(moon_segment.0).uses_dense_sampling());
    assert!(packaged_artifact_body_cadence(&CelestialBody::Pluto).uses_dense_sampling());
    assert!(packaged_artifact_body_cadence(&CelestialBody::Ceres).uses_dense_sampling());
    assert!(!packaged_artifact_body_cadence(mercury_segment.0).uses_dense_sampling());
    assert!(!packaged_artifact_body_cadence(saturn_segment.0).uses_dense_sampling());
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(mercury_segment.0),
        PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(&CelestialBody::Venus),
        PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(&CelestialBody::Jupiter),
        PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        packaged_artifact_fit_sample_counts_for_body(saturn_segment.0),
        PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS
    );
    assert_eq!(
        PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS,
        &[6, 8, 10, 12, 14]
    );
    assert_eq!(
        PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS.last().copied(),
        Some(20)
    );
    assert_eq!(
        packaged_artifact_residual_sample_fractions_for_channel(
            &lunar_point_body,
            ChannelKind::Longitude,
        ),
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_residual_sample_fractions_for_channel(
            &lunar_point_body,
            ChannelKind::DistanceAu,
        ),
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_residual_sample_fractions_for_channel(
            &CelestialBody::Ceres,
            ChannelKind::Longitude,
        ),
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_residual_sample_fractions_for_channel(
            &CelestialBody::Ceres,
            ChannelKind::DistanceAu,
        ),
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_residual_sample_fractions_for_channel(
            custom_segment.0,
            ChannelKind::Latitude,
        ),
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        packaged_artifact_residual_sample_fractions_for_channel(
            custom_segment.0,
            ChannelKind::DistanceAu,
        ),
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
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
fn packaged_artifact_outer_planets_use_medium_fit_sampling_and_dense_distance_validation() {
    let sample_segment = Segment::new(instant_tt(2_451_545.0), instant_tt(2_451_555.0), Vec::new());

    for body in [
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
    ] {
        assert!(!packaged_artifact_body_cadence(&body).uses_dense_sampling());
        assert!(packaged_artifact_body_cadence(&body).uses_dense_validation_sampling());
        assert_eq!(
            packaged_artifact_fit_sample_counts_for_body(&body),
            PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS
        );
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
        assert_eq!(
            packaged_artifact_residual_sample_fractions_for_channel(&body, ChannelKind::Longitude),
            PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
        );
        assert_eq!(
            packaged_artifact_residual_sample_fractions_for_channel(&body, ChannelKind::Latitude),
            PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
        );
        assert_eq!(
            packaged_artifact_residual_sample_fractions_for_channel(&body, ChannelKind::DistanceAu),
            PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
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
fn packaged_artifact_split_fraction_prefers_dense_body_curvature_bias() {
    let moderate_left_start = ecl(0.0, 0.0, 1.0);
    let moderate_left_quarter = ecl(1.0, 0.4, 1.01);
    let moderate_left_midpoint = ecl(1.8, 0.7, 1.02);
    let moderate_left_three_quarter = ecl(2.6, 1.0, 1.03);
    let moderate_left_end = ecl(3.4, 1.3, 1.04);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            3_200.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &moderate_left_start,
                quarter_coordinates: Some(&moderate_left_quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: None,
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &moderate_left_midpoint,
                two_third_coordinates: None,
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&moderate_left_three_quarter),
                end_coordinates: &moderate_left_end,
            },
        ),
        PACKAGED_ARTIFACT_LEFT_BIASED_SPLIT_FRACTION
    );

    let moderate_right_start = ecl(0.0, 0.0, 1.0);
    let moderate_right_quarter = ecl(0.8, 0.3, 1.01);
    let moderate_right_midpoint = ecl(1.0, 0.4, 1.02);
    let moderate_right_three_quarter = ecl(1.8, 0.7, 1.03);
    let moderate_right_end = ecl(3.0, 1.1, 1.04);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            3_200.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &moderate_right_start,
                quarter_coordinates: Some(&moderate_right_quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: None,
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &moderate_right_midpoint,
                two_third_coordinates: None,
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&moderate_right_three_quarter),
                end_coordinates: &moderate_right_end,
            },
        ),
        PACKAGED_ARTIFACT_RIGHT_BIASED_SPLIT_FRACTION
    );

    let extreme_left_start = ecl(0.0, 0.0, 1.0);
    let extreme_left_quarter = ecl(8.0, 4.0, 1.1);
    let extreme_left_midpoint = ecl(14.0, 7.0, 1.2);
    let extreme_left_three_quarter = ecl(15.0, 7.2, 1.22);
    let extreme_left_end = ecl(16.0, 7.4, 1.24);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            5_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &extreme_left_start,
                quarter_coordinates: Some(&extreme_left_quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: None,
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &extreme_left_midpoint,
                two_third_coordinates: None,
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&extreme_left_three_quarter),
                end_coordinates: &extreme_left_end,
            },
        ),
        PACKAGED_ARTIFACT_LEFT_EXTREME_SPLIT_FRACTION
    );

    let extreme_right_start = ecl(0.0, 0.0, 1.0);
    let extreme_right_quarter = ecl(1.0, 0.5, 1.01);
    let extreme_right_midpoint = ecl(2.0, 1.0, 1.02);
    let extreme_right_three_quarter = ecl(10.0, 5.0, 1.08);
    let extreme_right_end = ecl(16.0, 8.0, 1.12);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            5_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &extreme_right_start,
                quarter_coordinates: Some(&extreme_right_quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: None,
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &extreme_right_midpoint,
                two_third_coordinates: None,
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&extreme_right_three_quarter),
                end_coordinates: &extreme_right_end,
            },
        ),
        PACKAGED_ARTIFACT_RIGHT_EXTREME_SPLIT_FRACTION
    );

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Saturn,
            3_200.0,
            body_segment_span_limit(&CelestialBody::Saturn),
            PackagedArtifactSplitCurvature {
                start_coordinates: &moderate_left_start,
                quarter_coordinates: Some(&moderate_left_quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: None,
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &moderate_left_midpoint,
                two_third_coordinates: None,
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&moderate_left_three_quarter),
                end_coordinates: &moderate_left_end,
            },
        ),
        0.5
    );
}

#[test]
fn packaged_artifact_split_fraction_uses_dense_third_point_bias_when_quarter_curvature_is_balanced()
{
    let start = ecl(0.0, 0.0, 1.0);
    let quarter = ecl(1.0, 0.4, 1.02);
    let one_third = ecl(5.0, 2.0, 1.08);
    let midpoint = ecl(2.0, 0.8, 1.04);
    let two_third = ecl(2.1, 0.85, 1.05);
    let three_quarter = ecl(3.0, 1.2, 1.06);
    let end = ecl(4.0, 1.6, 1.08);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            3_200.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &start,
                quarter_coordinates: Some(&quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: Some(&one_third),
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &midpoint,
                two_third_coordinates: Some(&two_third),
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&three_quarter),
                end_coordinates: &end,
            },
        ),
        PACKAGED_ARTIFACT_ONE_THIRD_SPLIT_FRACTION
    );
}

#[test]
fn packaged_artifact_split_fraction_uses_dense_sixth_point_bias_on_very_long_spans() {
    let start = ecl(0.0, 0.0, 1.0);
    let one_sixth = ecl(0.9, 0.35, 1.01);
    let quarter = ecl(1.0, 0.4, 1.02);
    let one_third = ecl(1.5, 0.6, 1.03);
    let midpoint = ecl(2.0, 0.8, 1.04);
    let two_third = ecl(2.4, 0.9, 1.05);
    let three_quarter = ecl(3.0, 1.2, 1.06);
    let five_sixth = ecl(8.0, 4.0, 1.1);
    let end = ecl(3.4, 1.3, 1.07);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            7_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &start,
                quarter_coordinates: Some(&quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: Some(&one_sixth),
                one_third_coordinates: Some(&one_third),
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &midpoint,
                two_third_coordinates: Some(&two_third),
                four_fifth_coordinates: None,
                five_sixth_coordinates: Some(&five_sixth),
                three_quarter_coordinates: Some(&three_quarter),
                end_coordinates: &end,
            },
        ),
        5.0 / 6.0
    );
}

#[test]
fn packaged_artifact_split_fraction_falls_back_to_dense_third_point_bias_when_sixth_points_are_unavailable(
) {
    let start = ecl(0.0, 0.0, 1.0);
    let quarter = ecl(1.0, 0.4, 1.01);
    let one_third = ecl(5.0, 2.0, 2.0);
    let midpoint = ecl(2.0, 0.8, 1.02);
    let two_third = ecl(2.1, 0.85, 1.05);
    let three_quarter = ecl(3.0, 1.2, 1.03);
    let end = ecl(4.0, 1.6, 1.04);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            7_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &start,
                quarter_coordinates: Some(&quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: Some(&one_third),
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &midpoint,
                two_third_coordinates: Some(&two_third),
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&three_quarter),
                end_coordinates: &end,
            },
        ),
        PACKAGED_ARTIFACT_ONE_THIRD_SPLIT_FRACTION
    );
}

#[test]
fn packaged_artifact_split_fraction_falls_back_to_midpoint_when_third_points_are_unavailable() {
    let start = ecl(0.0, 0.0, 1.0);
    let quarter = ecl(1.0, 0.4, 1.01);
    let midpoint = ecl(2.0, 0.8, 1.02);
    let three_quarter = ecl(3.0, 1.2, 1.03);
    let end = ecl(4.0, 1.6, 1.04);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            7_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &start,
                quarter_coordinates: Some(&quarter),
                one_fifth_coordinates: None,
                one_sixth_coordinates: None,
                one_third_coordinates: None,
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &midpoint,
                two_third_coordinates: None,
                four_fifth_coordinates: None,
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&three_quarter),
                end_coordinates: &end,
            },
        ),
        0.5
    );
}

#[test]
fn packaged_artifact_split_fraction_uses_dense_fifth_point_bias_on_very_long_spans() {
    let start = ecl(0.0, 0.0, 1.0);
    let quarter = ecl(1.0, 0.4, 1.01);
    let one_fifth = ecl(6.0, 3.0, 1.06);
    let one_sixth = ecl(0.8, 0.32, 1.008);
    let one_third = ecl(1.5, 0.6, 1.015);
    let midpoint = ecl(2.0, 0.8, 1.02);
    let two_third = ecl(2.5, 1.0, 1.03);
    let four_fifth = ecl(2.2, 0.9, 1.025);
    let five_sixth = ecl(3.2, 1.28, 1.036);
    let three_quarter = ecl(3.0, 1.2, 1.04);
    let end = ecl(4.0, 1.6, 1.08);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            13_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &start,
                quarter_coordinates: Some(&quarter),
                one_fifth_coordinates: Some(&one_fifth),
                one_sixth_coordinates: Some(&one_sixth),
                one_third_coordinates: Some(&one_third),
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &midpoint,
                two_third_coordinates: Some(&two_third),
                four_fifth_coordinates: Some(&four_fifth),
                five_sixth_coordinates: Some(&five_sixth),
                three_quarter_coordinates: Some(&three_quarter),
                end_coordinates: &end,
            },
        ),
        PACKAGED_ARTIFACT_ONE_FIFTH_SPLIT_FRACTION
    );
}

#[test]
fn packaged_artifact_split_fraction_uses_dense_four_fifth_point_bias_on_very_long_spans() {
    let start = ecl(0.0, 0.0, 1.0);
    let quarter = ecl(1.0, 0.4, 1.01);
    let one_fifth = ecl(1.1, 0.44, 1.011);
    let one_sixth = ecl(0.8, 0.32, 1.008);
    let one_third = ecl(1.5, 0.6, 1.015);
    let midpoint = ecl(2.0, 0.8, 1.02);
    let two_third = ecl(2.5, 1.0, 1.03);
    let four_fifth = ecl(7.0, 3.5, 1.07);
    let five_sixth = ecl(3.2, 1.28, 1.036);
    let three_quarter = ecl(3.0, 1.2, 1.04);
    let end = ecl(4.0, 1.6, 1.08);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            13_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &start,
                quarter_coordinates: Some(&quarter),
                one_fifth_coordinates: Some(&one_fifth),
                one_sixth_coordinates: Some(&one_sixth),
                one_third_coordinates: Some(&one_third),
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &midpoint,
                two_third_coordinates: Some(&two_third),
                four_fifth_coordinates: Some(&four_fifth),
                five_sixth_coordinates: Some(&five_sixth),
                three_quarter_coordinates: Some(&three_quarter),
                end_coordinates: &end,
            },
        ),
        PACKAGED_ARTIFACT_FOUR_FIFTHS_SPLIT_FRACTION
    );
}

#[test]
fn packaged_artifact_split_fraction_uses_dense_ninth_and_eighth_point_bias_on_super_extreme_spans()
{
    let point = |longitude: f64, latitude: f64| ecl(longitude, latitude, 1.0);

    let baseline = point(0.0, 0.0);
    let one_ninth = point(16.0, 6.4);
    let one_eighth = point(14.0, 5.6);
    let seven_eighths = point(14.0, 5.6);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            300_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &baseline,
                quarter_coordinates: Some(&baseline),
                one_fifth_coordinates: None,
                one_sixth_coordinates: Some(&baseline),
                one_seventh_coordinates: Some(&baseline),
                six_sevenths_coordinates: Some(&baseline),
                one_ninth_coordinates: Some(&one_ninth),
                eight_ninths_coordinates: Some(&baseline),
                one_eighth_coordinates: Some(&baseline),
                seven_eighths_coordinates: Some(&baseline),
                one_third_coordinates: Some(&baseline),
                midpoint_coordinates: &baseline,
                two_third_coordinates: Some(&baseline),
                four_fifth_coordinates: Some(&baseline),
                five_sixth_coordinates: Some(&baseline),
                three_quarter_coordinates: Some(&baseline),
                end_coordinates: &baseline,
            },
        ),
        PACKAGED_ARTIFACT_ONE_NINTH_SPLIT_FRACTION
    );

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            300_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &baseline,
                quarter_coordinates: Some(&baseline),
                one_fifth_coordinates: None,
                one_sixth_coordinates: Some(&baseline),
                one_seventh_coordinates: Some(&baseline),
                six_sevenths_coordinates: Some(&baseline),
                one_ninth_coordinates: Some(&baseline),
                eight_ninths_coordinates: Some(&baseline),
                one_eighth_coordinates: Some(&one_eighth),
                seven_eighths_coordinates: Some(&baseline),
                one_third_coordinates: Some(&baseline),
                midpoint_coordinates: &baseline,
                two_third_coordinates: Some(&baseline),
                four_fifth_coordinates: Some(&baseline),
                five_sixth_coordinates: Some(&baseline),
                three_quarter_coordinates: Some(&baseline),
                end_coordinates: &baseline,
            },
        ),
        PACKAGED_ARTIFACT_ONE_EIGHTH_SPLIT_FRACTION
    );

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            300_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &baseline,
                quarter_coordinates: Some(&baseline),
                one_fifth_coordinates: None,
                one_sixth_coordinates: Some(&baseline),
                one_seventh_coordinates: Some(&baseline),
                six_sevenths_coordinates: Some(&baseline),
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: Some(&baseline),
                seven_eighths_coordinates: Some(&seven_eighths),
                one_third_coordinates: Some(&baseline),
                midpoint_coordinates: &baseline,
                two_third_coordinates: Some(&baseline),
                four_fifth_coordinates: Some(&baseline),
                five_sixth_coordinates: Some(&baseline),
                three_quarter_coordinates: Some(&baseline),
                end_coordinates: &baseline,
            },
        ),
        PACKAGED_ARTIFACT_SEVEN_EIGHTHS_SPLIT_FRACTION
    );
}

#[test]
fn packaged_artifact_split_fraction_ignores_fifth_point_bias_before_the_longest_span_threshold() {
    let start = ecl(0.0, 0.0, 1.0);
    let quarter = ecl(1.0, 0.4, 1.01);
    let one_fifth = ecl(6.2, 3.1, 1.062);
    let one_third = ecl(1.5, 0.6, 1.015);
    let midpoint = ecl(2.0, 0.8, 1.02);
    let two_third = ecl(2.5, 1.0, 1.03);
    let four_fifth = ecl(6.0, 3.0, 1.06);
    let three_quarter = ecl(3.0, 1.2, 1.04);
    let end = ecl(4.0, 1.6, 1.08);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            7_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &start,
                quarter_coordinates: Some(&quarter),
                one_fifth_coordinates: Some(&one_fifth),
                one_sixth_coordinates: None,
                one_third_coordinates: Some(&one_third),
                one_seventh_coordinates: None,
                six_sevenths_coordinates: None,
                one_ninth_coordinates: None,
                eight_ninths_coordinates: None,
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                midpoint_coordinates: &midpoint,
                two_third_coordinates: Some(&two_third),
                four_fifth_coordinates: Some(&four_fifth),
                five_sixth_coordinates: None,
                three_quarter_coordinates: Some(&three_quarter),
                end_coordinates: &end,
            },
        ),
        0.5
    );
}

#[test]
fn packaged_artifact_split_fraction_uses_dense_seventh_point_bias_on_extreme_spans() {
    let point = |longitude: f64, latitude: f64| ecl(longitude, latitude, 1.0);

    let baseline = point(0.0, 0.0);
    let one_seventh = point(12.0, 4.8);
    let six_sevenths = point(12.0, 4.8);

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            30_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &baseline,
                quarter_coordinates: Some(&baseline),
                one_fifth_coordinates: None,
                one_sixth_coordinates: Some(&baseline),
                one_seventh_coordinates: Some(&one_seventh),
                six_sevenths_coordinates: Some(&baseline),
                one_ninth_coordinates: Some(&baseline),
                eight_ninths_coordinates: Some(&baseline),
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                one_third_coordinates: Some(&baseline),
                midpoint_coordinates: &baseline,
                two_third_coordinates: Some(&baseline),
                four_fifth_coordinates: Some(&baseline),
                five_sixth_coordinates: Some(&baseline),
                three_quarter_coordinates: Some(&baseline),
                end_coordinates: &baseline,
            },
        ),
        PACKAGED_ARTIFACT_ONE_SEVENTH_SPLIT_FRACTION
    );

    assert_eq!(
        packaged_artifact_split_fraction_for_interval(
            &CelestialBody::Pluto,
            30_000.0,
            body_segment_span_limit(&CelestialBody::Pluto),
            PackagedArtifactSplitCurvature {
                start_coordinates: &baseline,
                quarter_coordinates: Some(&baseline),
                one_fifth_coordinates: None,
                one_sixth_coordinates: Some(&baseline),
                one_seventh_coordinates: Some(&baseline),
                six_sevenths_coordinates: Some(&six_sevenths),
                one_ninth_coordinates: Some(&baseline),
                eight_ninths_coordinates: Some(&baseline),
                one_eighth_coordinates: None,
                seven_eighths_coordinates: None,
                one_third_coordinates: Some(&baseline),
                midpoint_coordinates: &baseline,
                two_third_coordinates: Some(&baseline),
                four_fifth_coordinates: Some(&baseline),
                five_sixth_coordinates: Some(&baseline),
                three_quarter_coordinates: Some(&baseline),
                end_coordinates: &baseline,
            },
        ),
        PACKAGED_ARTIFACT_SIX_SEVENTHS_SPLIT_FRACTION
    );
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
