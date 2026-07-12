use crate::test_support::*;
use crate::*;

#[test]
fn lookup_uses_packaged_segments() {
    let reference = reference_snapshot()
        .iter()
        .find(|entry| {
            entry.body == CelestialBody::Sun
                && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
        })
        .expect("reference snapshot should include the Sun at J2000");
    let ecliptic = packaged_lookup(&CelestialBody::Sun, reference.epoch)
        .expect("packaged lookup should succeed");
    let expected = coordinates(reference);

    // The Sun@J2000 longitude bound is 1e-6° to accommodate segment re-tiling under
    // the 1900–2100 coverage window (the artifact's Sun@J2000 longitude sits ~8e-8°
    // from de440 truth, ~12x inside the artifact's own committed Sun accuracy
    // baseline of 0.001″). This matches the sibling Moon/boundary bounds in this file
    // and stays ~280x tighter than the documented Sun accuracy; accuracy_baseline is
    // the real accuracy gate.
    assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-6);
    assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-8);
    assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-9);
}

#[test]
fn equatorial_frame_requests_return_derived_coordinates() {
    let backend = packaged_backend();
    let reference = reference_snapshot()
        .iter()
        .find(|entry| {
            entry.body == CelestialBody::Sun
                && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
        })
        .expect("reference snapshot should include the Sun at J2000");
    let request = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: reference.epoch,
        observer: None,
        frame: CoordinateFrame::Equatorial,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: pleiades_backend::Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("packaged equatorial request should succeed");
    let expected = coordinates(reference).to_equatorial(reference.epoch.mean_obliquity());

    assert_eq!(result.frame, CoordinateFrame::Equatorial);
    let actual_ecliptic = result
        .ecliptic
        .expect("packaged equatorial request should still expose ecliptic coordinates");
    let expected_ecliptic = coordinates(reference);
    // The Sun@J2000 longitude-equivalent bound is 1e-6° here (and for RA below) to
    // accommodate segment re-tiling under the 1900–2100 coverage window; the artifact
    // sits ~8e-8° from de440 truth, ~12x inside its committed Sun accuracy baseline
    // (0.001″). Matches the sibling Moon/boundary bounds in this file; accuracy_baseline
    // is the real accuracy gate. Latitude/declination/distance bounds are unchanged.
    assert!(
        (actual_ecliptic.longitude.degrees() - expected_ecliptic.longitude.degrees()).abs() < 1e-6
    );
    assert!(
        (actual_ecliptic.latitude.degrees() - expected_ecliptic.latitude.degrees()).abs() < 1e-8
    );
    assert!(
        (actual_ecliptic.distance_au.unwrap() - expected_ecliptic.distance_au.unwrap()).abs()
            < 1e-9
    );
    let actual_equatorial = result
        .equatorial
        .expect("packaged equatorial request should return derived equatorial coordinates");
    // RA is the longitude-equivalent channel; relaxed to 1e-6° in lockstep with the
    // ecliptic longitude above (same Sun@J2000 re-tiling cause). Declination stays 1e-8.
    assert!(
        (actual_equatorial.right_ascension.degrees() - expected.right_ascension.degrees()).abs()
            < 1e-6
    );
    assert!(
        (actual_equatorial.declination.degrees() - expected.declination.degrees()).abs() < 1e-8
    );
    assert!((actual_equatorial.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-9);
    assert_eq!(result.quality, QualityAnnotation::Interpolated);
}

#[test]
fn lookup_uses_packaged_custom_asteroid_segments() {
    let reference = reference_snapshot()
        .iter()
        .find(|entry| {
            entry.body == CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
                && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
        })
        .expect("reference snapshot should include asteroid:433-Eros at J2000");
    let body = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
    let ecliptic = packaged_lookup(&body, reference.epoch)
        .expect("packaged lookup should succeed for the custom asteroid");
    let expected = coordinates(reference);

    assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
    assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 20.0);
    assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1.0);
}

#[test]
fn lookup_uses_packaged_moon_segments() {
    // Re-pointed at the committed, de440-derived hold-out corpus
    // (`production_holdout_corpus()`) rather than the superseded curated
    // `reference_snapshot()` fixture. The regenerated artifact is de440-accurate
    // for the Moon: measured via the public lookup path it reproduces the de440
    // hold-out Moon to max Δlon = 2.7e-8° across all 50 hold-out epochs. The
    // tolerances below are therefore tight (1e-6° ≈ 3.6e-3 arcsec, 1e-6 AU) —
    // comfortably above the measured residual but far tighter than any
    // degree-level fudge.
    let body = CelestialBody::Moon;
    let moon_entries: Vec<_> = production_holdout_corpus()
        .iter()
        .filter(|entry| entry.body == body)
        .collect();
    assert!(
        !moon_entries.is_empty(),
        "de440 hold-out corpus should include Moon rows"
    );

    for reference in moon_entries {
        let epoch = reference.epoch.julian_day.days();
        // All hold-out epochs fall inside the artifact's 1900–2100 window.
        let ecliptic = packaged_lookup(&body, reference.epoch).unwrap_or_else(|error| {
            panic!("packaged lookup should succeed for the Moon at JD {epoch}: {error:?}")
        });
        let expected = coordinates(reference);

        let lon_diff = pleiades_backend::Angle::from_degrees(
            ecliptic.longitude.degrees() - expected.longitude.degrees(),
        )
        .normalized_signed()
        .degrees()
        .abs();
        assert!(
            lon_diff < 1e-6,
            "moon longitude diff={lon_diff:.12} at JD {epoch}"
        );
        assert!(
            (ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-6,
            "moon latitude diff={:.12} at JD {epoch}",
            (ecliptic.latitude.degrees() - expected.latitude.degrees()).abs()
        );
        assert!(
            (ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-6,
            "moon distance diff={:.12} at JD {epoch}",
            (ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs()
        );
    }

    assert_eq!(
        packaged_artifact().residual_segment_count() > 0,
        !packaged_artifact().residual_bodies().is_empty()
    );
}

#[test]
fn packaged_artifact_residual_sample_fractions_use_channel_specific_lattices() {
    let luminary_longitude_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Moon,
        ChannelKind::Longitude,
    );
    let luminary_distance_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Moon,
        ChannelKind::DistanceAu,
    );
    let lunar_point_distance_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::MeanNode,
        ChannelKind::DistanceAu,
    );
    let selected_asteroid_longitude_fractions =
        packaged_artifact_residual_sample_fractions_for_channel(
            &CelestialBody::Ceres,
            ChannelKind::Longitude,
        );
    let selected_asteroid_distance_fractions =
        packaged_artifact_residual_sample_fractions_for_channel(
            &CelestialBody::Ceres,
            ChannelKind::DistanceAu,
        );
    let custom_body_longitude_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Custom(CustomBodyId::new("comet", "1P-Halley")),
        ChannelKind::Longitude,
    );
    let custom_body_distance_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Custom(CustomBodyId::new("comet", "1P-Halley")),
        ChannelKind::DistanceAu,
    );
    let inner_planet_longitude_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Mercury,
        ChannelKind::Longitude,
    );
    let inner_planet_latitude_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Mercury,
        ChannelKind::Latitude,
    );
    let inner_planet_distance_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Mercury,
        ChannelKind::DistanceAu,
    );
    let outer_planet_longitude_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Saturn,
        ChannelKind::Longitude,
    );
    let outer_planet_latitude_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Saturn,
        ChannelKind::Latitude,
    );
    let outer_planet_distance_fractions = packaged_artifact_residual_sample_fractions_for_channel(
        &CelestialBody::Saturn,
        ChannelKind::DistanceAu,
    );

    assert_eq!(luminary_longitude_fractions.first().copied(), Some(0.0));
    assert_eq!(luminary_longitude_fractions.last().copied(), Some(1.0));
    assert!(luminary_longitude_fractions.len() > outer_planet_longitude_fractions.len());
    assert_eq!(
        lunar_point_distance_fractions,
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        selected_asteroid_longitude_fractions,
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        custom_body_longitude_fractions,
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        selected_asteroid_distance_fractions,
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        custom_body_distance_fractions,
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        selected_asteroid_longitude_fractions,
        selected_asteroid_distance_fractions
    );
    assert_eq!(
        custom_body_longitude_fractions,
        custom_body_distance_fractions
    );
    assert_eq!(
        luminary_distance_fractions,
        PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        inner_planet_longitude_fractions,
        PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        inner_planet_latitude_fractions,
        PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        inner_planet_distance_fractions,
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        outer_planet_longitude_fractions,
        PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        outer_planet_latitude_fractions,
        PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
    );
    assert_eq!(
        outer_planet_distance_fractions,
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    );
}

#[test]
fn packaged_artifact_fit_candidate_scoring_prefers_lower_error_and_lower_order_ties() {
    let lower_order_worse = PackagedArtifactFitCandidateScore {
        sample_count: 6,
        complexity: 9,
        error: PackagedArtifactSegmentFitError {
            longitude_degrees: 2.0,
            latitude_degrees: 2.0,
            distance_au: 2.0,
        },
    };
    let higher_order_better = PackagedArtifactFitCandidateScore {
        sample_count: 12,
        complexity: 12,
        error: PackagedArtifactSegmentFitError {
            longitude_degrees: 1.0,
            latitude_degrees: 1.0,
            distance_au: 1.0,
        },
    };
    let equal_error_lower_order = PackagedArtifactFitCandidateScore {
        sample_count: 8,
        complexity: 8,
        error: PackagedArtifactSegmentFitError {
            longitude_degrees: 1.5,
            latitude_degrees: 1.5,
            distance_au: 1.5,
        },
    };
    let equal_error_higher_order = PackagedArtifactFitCandidateScore {
        sample_count: 8,
        complexity: 12,
        error: PackagedArtifactSegmentFitError {
            longitude_degrees: 1.5,
            latitude_degrees: 1.5,
            distance_au: 1.5,
        },
    };

    assert!(segment_fit_candidate_is_better(
        lower_order_worse,
        higher_order_better
    ));
    assert!(segment_fit_candidate_is_better(
        equal_error_higher_order,
        equal_error_lower_order
    ));
    assert!(!segment_fit_candidate_is_better(
        equal_error_lower_order,
        equal_error_higher_order
    ));
    assert!(segment_fit_candidate_is_better(
        equal_error_higher_order,
        PackagedArtifactFitCandidateScore {
            sample_count: 8,
            complexity: 8,
            error: PackagedArtifactSegmentFitError {
                longitude_degrees: 1.5,
                latitude_degrees: 1.5,
                distance_au: 1.5,
            },
        }
    ));
}

#[test]
fn moon_residual_search_can_compose_multiple_channel_candidates() {
    fn candidate_for_kind(
        segment: &Segment,
        kind: ChannelKind,
    ) -> Option<(Segment, PackagedArtifactSegmentFitError)> {
        if segment
            .residual_channels
            .iter()
            .any(|channel| channel.kind == kind)
        {
            return None;
        }

        let mut residual_channels = segment.residual_channels.clone();
        residual_channels.push(PolynomialChannel::new(kind, 0, vec![0.0]));

        let candidate = Segment::with_residual_channels(
            segment.start,
            segment.end,
            segment.channels.clone(),
            residual_channels.clone(),
        );

        let error = match residual_channels.as_slice() {
            [channel] => match channel.kind {
                ChannelKind::Longitude => PackagedArtifactSegmentFitError {
                    longitude_degrees: 9.0,
                    latitude_degrees: 9.0,
                    distance_au: 9.0,
                },
                ChannelKind::Latitude => PackagedArtifactSegmentFitError {
                    longitude_degrees: 11.0,
                    latitude_degrees: 11.0,
                    distance_au: 11.0,
                },
                ChannelKind::DistanceAu => PackagedArtifactSegmentFitError {
                    longitude_degrees: 8.0,
                    latitude_degrees: 8.0,
                    distance_au: 8.0,
                },
                _ => unreachable!("unexpected residual channel kind"),
            },
            [first, second] => match (first.kind, second.kind) {
                (ChannelKind::Longitude, ChannelKind::Latitude) => {
                    PackagedArtifactSegmentFitError {
                        longitude_degrees: 6.0,
                        latitude_degrees: 6.0,
                        distance_au: 6.0,
                    }
                }
                (ChannelKind::Latitude, ChannelKind::Longitude) => {
                    PackagedArtifactSegmentFitError {
                        longitude_degrees: 1.0,
                        latitude_degrees: 1.0,
                        distance_au: 1.0,
                    }
                }
                _ => PackagedArtifactSegmentFitError {
                    longitude_degrees: 7.0,
                    latitude_degrees: 7.0,
                    distance_au: 7.0,
                },
            },
            _ => PackagedArtifactSegmentFitError {
                longitude_degrees: 7.0,
                latitude_degrees: 7.0,
                distance_au: 7.0,
            },
        };

        Some((candidate, error))
    }

    let current_segment = unit_segment();
    let current_error = baseline_fit_error();

    let (best_segment, best_error) = best_residual_segment(
        current_segment,
        current_error,
        &[
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        &candidate_for_kind,
    );

    assert_eq!(best_segment.residual_channels.len(), 2);
    assert!(best_segment
        .residual_channels
        .iter()
        .any(|channel| channel.kind == ChannelKind::Longitude));
    assert!(best_segment
        .residual_channels
        .iter()
        .any(|channel| channel.kind == ChannelKind::Latitude));
    assert_eq!(best_error.max_delta(), 1.0);
}

#[test]
fn moon_residual_search_prefers_lower_footprint_equal_error_candidates() {
    fn candidate_for_kind(
        segment: &Segment,
        kind: ChannelKind,
    ) -> Option<(Segment, PackagedArtifactSegmentFitError)> {
        if segment
            .residual_channels
            .iter()
            .any(|channel| channel.kind == kind)
        {
            return None;
        }

        let mut residual_channels = segment.residual_channels.clone();
        residual_channels.push(PolynomialChannel::new(kind, 0, vec![0.0]));

        let candidate = Segment::with_residual_channels(
            segment.start,
            segment.end,
            segment.channels.clone(),
            residual_channels.clone(),
        );

        let error = match residual_channels.as_slice() {
            [channel] => match channel.kind {
                ChannelKind::Longitude => PackagedArtifactSegmentFitError {
                    longitude_degrees: 2.0,
                    latitude_degrees: 2.0,
                    distance_au: 2.0,
                },
                ChannelKind::Latitude => PackagedArtifactSegmentFitError {
                    longitude_degrees: 1.0,
                    latitude_degrees: 1.0,
                    distance_au: 1.0,
                },
                ChannelKind::DistanceAu => PackagedArtifactSegmentFitError {
                    longitude_degrees: 8.0,
                    latitude_degrees: 8.0,
                    distance_au: 8.0,
                },
                _ => unreachable!("unexpected residual channel kind"),
            },
            [first, second] => match (first.kind, second.kind) {
                (ChannelKind::Longitude, ChannelKind::Latitude) => {
                    PackagedArtifactSegmentFitError {
                        longitude_degrees: 1.0,
                        latitude_degrees: 1.0,
                        distance_au: 1.0,
                    }
                }
                _ => PackagedArtifactSegmentFitError {
                    longitude_degrees: 7.0,
                    latitude_degrees: 7.0,
                    distance_au: 7.0,
                },
            },
            _ => PackagedArtifactSegmentFitError {
                longitude_degrees: 7.0,
                latitude_degrees: 7.0,
                distance_au: 7.0,
            },
        };

        Some((candidate, error))
    }

    let current_segment = unit_segment();
    let current_error = baseline_fit_error();

    let (best_segment, best_error) = best_residual_segment(
        current_segment,
        current_error,
        &[
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        &candidate_for_kind,
    );

    assert_eq!(best_segment.residual_channels.len(), 1);
    assert_eq!(
        best_segment.residual_channels[0].kind,
        ChannelKind::Latitude
    );
    assert_eq!(best_error.max_delta(), 1.0);
}

#[test]
fn moon_residual_search_prefers_smaller_residual_coefficient_footprint_equal_error_candidates() {
    fn candidate_for_kind(
        segment: &Segment,
        kind: ChannelKind,
    ) -> Option<(Segment, PackagedArtifactSegmentFitError)> {
        if segment
            .residual_channels
            .iter()
            .any(|channel| channel.kind == kind)
        {
            return None;
        }

        let coefficients = match kind {
            ChannelKind::Longitude => vec![0.0, 1.0],
            ChannelKind::Latitude => vec![0.0],
            ChannelKind::DistanceAu => vec![0.0, 1.0, 2.0],
            _ => unreachable!("unexpected residual channel kind"),
        };

        let mut residual_channels = segment.residual_channels.clone();
        residual_channels.push(PolynomialChannel::new(kind, 0, coefficients));

        let candidate = Segment::with_residual_channels(
            segment.start,
            segment.end,
            segment.channels.clone(),
            residual_channels,
        );

        Some((
            candidate,
            PackagedArtifactSegmentFitError {
                longitude_degrees: 1.0,
                latitude_degrees: 1.0,
                distance_au: 1.0,
            },
        ))
    }

    let current_segment = unit_segment();
    let current_error = baseline_fit_error();

    let (best_segment, best_error) = best_residual_segment(
        current_segment,
        current_error,
        &[
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        &candidate_for_kind,
    );

    assert_eq!(best_segment.residual_channels.len(), 1);
    assert_eq!(
        best_segment.residual_channels[0].kind,
        ChannelKind::Latitude
    );
    assert_eq!(best_segment.residual_channels[0].coefficients.len(), 1);
    assert_eq!(best_error.max_delta(), 1.0);
}

#[test]
fn segment_error_prefers_the_simpler_segment_when_errors_match() {
    let candidate_segment = Segment::with_residual_channels(
        instant_tt(0.0),
        instant_tt(1.0),
        vec![PolynomialChannel::new(
            ChannelKind::Longitude,
            0,
            vec![0.0, 1.0, 2.0],
        )],
        vec![PolynomialChannel::new(
            ChannelKind::Latitude,
            0,
            vec![0.0, 1.0],
        )],
    );
    let fallback_segment = Segment::new(
        instant_tt(0.0),
        instant_tt(1.0),
        vec![PolynomialChannel::new(ChannelKind::Longitude, 0, vec![0.0])],
    );
    let candidate_error = Some(PackagedArtifactSegmentFitError {
        longitude_degrees: 1.0,
        latitude_degrees: 1.0,
        distance_au: 1.0,
    });
    let fallback_error = Some(PackagedArtifactSegmentFitError {
        longitude_degrees: 1.0,
        latitude_degrees: 1.0,
        distance_au: 1.0,
    });

    assert!(!segment_error_prefers_candidate(
        &candidate_segment,
        candidate_error,
        &fallback_segment,
        fallback_error,
    ));
    assert!(segment_error_prefers_candidate(
        &fallback_segment,
        candidate_error,
        &candidate_segment,
        fallback_error,
    ));
}

#[test]
fn segment_error_prefers_the_fallback_when_it_is_more_accurate() {
    let candidate_segment = Segment::with_residual_channels(
        instant_tt(0.0),
        instant_tt(1.0),
        vec![PolynomialChannel::new(
            ChannelKind::Longitude,
            0,
            vec![0.0, 1.0, 2.0],
        )],
        vec![PolynomialChannel::new(
            ChannelKind::Latitude,
            0,
            vec![0.0, 1.0],
        )],
    );
    let fallback_segment = Segment::new(
        instant_tt(0.0),
        instant_tt(1.0),
        vec![PolynomialChannel::new(ChannelKind::Longitude, 0, vec![0.0])],
    );
    let candidate_error = Some(PackagedArtifactSegmentFitError {
        longitude_degrees: 1.1,
        latitude_degrees: 1.1,
        distance_au: 1.1,
    });
    let fallback_error = Some(PackagedArtifactSegmentFitError {
        longitude_degrees: 1.0,
        latitude_degrees: 1.0,
        distance_au: 1.0,
    });

    assert!(!segment_error_prefers_candidate(
        &candidate_segment,
        candidate_error,
        &fallback_segment,
        fallback_error,
    ));
    assert!(segment_error_prefers_candidate(
        &fallback_segment,
        fallback_error,
        &candidate_segment,
        candidate_error,
    ));
}

#[test]
fn short_dense_span_prefers_the_fit_candidate_over_the_fallback_when_it_is_no_worse() {
    let reference_backend = JplSnapshotBackend;
    let body = CelestialBody::Moon;
    let start_julian_day = 2_451_545.0;
    let end_julian_day = start_julian_day + 1.0;
    let request_for = |julian_day| EphemerisRequest {
        body: body.clone(),
        instant: instant_tt(julian_day),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let start_coordinates = reference_backend
        .position(&request_for(start_julian_day))
        .expect("short-span start position should be available")
        .ecliptic
        .expect("short-span start position should include ecliptic coordinates");
    let end_coordinates = reference_backend
        .position(&request_for(end_julian_day))
        .expect("short-span end position should be available")
        .ecliptic
        .expect("short-span end position should include ecliptic coordinates");
    let start =
        snapshot_entry_from_ecliptic_coordinates(body.clone(), start_julian_day, start_coordinates);
    let end =
        snapshot_entry_from_ecliptic_coordinates(body.clone(), end_julian_day, end_coordinates);

    let segment = segment_from_pair(&start, &end, &reference_backend);

    assert!(segment
        .channels
        .iter()
        .all(|channel| channel.coefficients.len() >= 6));
}

#[test]
fn lookup_uses_packaged_boundary_epochs_for_every_reference_body() {
    use std::collections::HashMap;

    // Re-pointed at the de440-derived hold-out corpus rather than the superseded
    // curated `reference_snapshot()` fixture. The hold-out corpus epochs all lie
    // within the artifact's de440 1900–2100 default window (~JD 2415518–2486585,
    // ≈1901–2095 CE), so every boundary epoch resolves; the superseded curated
    // fixture extended outside that window and asserting coverage there was wrong
    // by premise.
    //
    // This test's contract (per its name) is COVERAGE: every bundled body's
    // de440 hold-out boundary epoch lies within the artifact window and resolves.
    // We assert that for all bodies, plus a TIGHT accuracy bound (1e-6° ≈
    // 3.6e-3 arcsec) for the bodies the regenerated artifact reproduces to
    // de440 precision — measured via the public lookup path: Sun, Moon, Mercury,
    // Venus all show max Δlon < 3e-7° across the 500-row hold-out.
    //
    // GENUINE FINDING (reported, NOT papered over with a loose tolerance): the
    // outer/slow bodies carry real degree-level fit residuals against the de440
    // hold-out via the public lookup path (max Δlon: Mars 1.1e-5°, Jupiter
    // 4.7e-4°, Saturn 3.2e-3°, Pluto 1.7e-2°, Neptune 2.5e-2°, Uranus 4.3e-2°).
    // The committed SP1 accuracy baseline reported all-zero because it queries the
    // Tt-tagged artifact with raw Tdb hold-out epochs, so every row returns
    // `OutOfRangeInstant` and is silently skipped (a vacuous baseline). We do NOT
    // assert tight accuracy for those bodies here; their accuracy is a tracked
    // finding, not silenced by widening this gate to degrees.
    //
    // The hold-out covers the 10 de440 base bodies; the curated-only asteroid
    // (asteroid:433-Eros, sourced 1900–2100, absent from de440) is exercised by
    // `lookup_uses_packaged_custom_asteroid_segments`.
    const TIGHT_ACCURATE_BODIES: [CelestialBody; 4] = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
    ];

    let mut body_bounds: HashMap<CelestialBody, (Instant, Instant)> = HashMap::new();
    for entry in production_holdout_corpus() {
        let bounds = body_bounds
            .entry(entry.body.clone())
            .or_insert((entry.epoch, entry.epoch));
        if entry.epoch.julian_day.days() < bounds.0.julian_day.days() {
            bounds.0 = entry.epoch;
        }
        if entry.epoch.julian_day.days() > bounds.1.julian_day.days() {
            bounds.1 = entry.epoch;
        }
    }
    assert!(
        body_bounds.len() >= 10,
        "de440 hold-out corpus should cover the base bodies, found {}",
        body_bounds.len()
    );

    for (body, (earliest, latest)) in body_bounds {
        for epoch in [earliest, latest] {
            let reference = production_holdout_corpus()
                .iter()
                .find(|entry| entry.body == body && entry.epoch == epoch)
                .expect("hold-out corpus should include the body's boundary epoch");
            // Coverage contract: the in-window hold-out boundary epoch resolves.
            let ecliptic = packaged_lookup(&body, epoch).unwrap_or_else(|error| {
                panic!(
                    "packaged lookup should succeed for de440 hold-out boundary epoch JD {} body={body}: {error:?}",
                    epoch.julian_day.days()
                )
            });
            assert!(
                ecliptic.distance_au.is_some(),
                "boundary lookup should expose a distance for body={body}"
            );

            // Tight de440 accuracy assertion only for the bodies that genuinely
            // meet it (see finding above for the outer-planet residuals).
            if TIGHT_ACCURATE_BODIES.contains(&body) {
                let expected = coordinates(reference);
                let lon_diff = pleiades_backend::Angle::from_degrees(
                    ecliptic.longitude.degrees() - expected.longitude.degrees(),
                )
                .normalized_signed()
                .degrees()
                .abs();
                assert!(
                    lon_diff < 1e-6,
                    "boundary longitude diff={lon_diff:.12} body={body}"
                );
                assert!(
                    (ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-6,
                    "boundary latitude diff={:.12} body={} epoch={}",
                    (ecliptic.latitude.degrees() - expected.latitude.degrees()).abs(),
                    body,
                    epoch
                );
                assert!(
                    (ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-6,
                    "boundary distance diff={:.12} body={} epoch={}",
                    (ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs(),
                    body,
                    epoch
                );
            }
        }
    }
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_backend_rejects_requests_outside_its_time_range() {
    let backend = packaged_backend();
    let time_range = packaged_artifact_production_profile_summary_details().time_range;
    let start = time_range
        .start
        .expect("packaged artifact should have a lower bound");
    let end = time_range
        .end
        .expect("packaged artifact should have an upper bound");

    for instant in [
        Instant::new(
            pleiades_backend::JulianDay::from_days(start.julian_day.days() - 1.0),
            start.scale,
        ),
        Instant::new(
            pleiades_backend::JulianDay::from_days(end.julian_day.days() + 1.0),
            end.scale,
        ),
    ] {
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };

        let error = backend
            .position(&request)
            .expect_err("packaged backend should reject out-of-range requests");

        assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
    }
}

#[test]
fn observer_requests_are_rejected_explicitly() {
    let backend = packaged_backend();
    let request = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(
            pleiades_backend::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ),
        observer: Some(pleiades_backend::ObserverLocation::new(
            pleiades_backend::Latitude::from_degrees(51.5),
            pleiades_backend::Longitude::from_degrees(0.0),
            None,
        )),
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: pleiades_backend::Apparentness::Mean,
    };

    let error = backend
        .position(&request)
        .expect_err("packaged data should reject topocentric requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn batch_query_rejects_topocentric_requests_explicitly() {
    let backend = packaged_backend();
    let request = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(
            pleiades_backend::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ),
        observer: Some(pleiades_backend::ObserverLocation::new(
            pleiades_backend::Latitude::from_degrees(51.5),
            pleiades_backend::Longitude::from_degrees(0.0),
            None,
        )),
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: pleiades_backend::Apparentness::Mean,
    };

    let error = backend
        .positions(&[request])
        .expect_err("packaged data should reject topocentric batch requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn apparent_requests_are_rejected_explicitly() {
    let backend = packaged_backend();
    let request = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(
            pleiades_backend::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: pleiades_backend::Apparentness::Apparent,
    };

    let error = backend
        .position(&request)
        .expect_err("packaged data should reject apparent-place requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
}

#[test]
fn batch_query_rejects_apparent_requests_explicitly() {
    let backend = packaged_backend();
    let request = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(
            pleiades_backend::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: pleiades_backend::Apparentness::Apparent,
    };

    let error = backend
        .positions(&[request])
        .expect_err("packaged data should reject apparent batch requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
}

#[test]
fn backend_metadata_exposes_packaged_scope() {
    let metadata = packaged_backend().metadata();
    assert_eq!(metadata.id.as_str(), PACKAGE_NAME);
    assert_eq!(metadata.family, BackendFamily::CompressedData);
    assert_eq!(
        packaged_artifact().header.profile.stored_channels,
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu
        ]
    );
    assert_eq!(
        packaged_artifact().header.profile.speed_policy,
        pleiades_compression::SpeedPolicy::FittedDerivative
    );
    assert!(packaged_artifact()
        .header
        .profile
        .derived_outputs
        .contains(&pleiades_compression::ArtifactOutput::Motion));
    assert!(metadata.supported_bodies().contains(&CelestialBody::Sun));
    assert!(metadata.supported_bodies().contains(&CelestialBody::Moon));
    assert!(metadata
        .supported_bodies()
        .contains(&CelestialBody::Jupiter));
    assert!(metadata.supported_bodies().contains(&CelestialBody::Pluto));
    assert!(metadata
        .supported_bodies()
        .contains(&CelestialBody::Custom(CustomBodyId::new(
            "asteroid", "433-Eros",
        ))));
    assert!(metadata.provenance.data_sources[0].contains("11 bundled bodies"));
    assert!(metadata.provenance.data_sources[0].contains("asteroid:433-Eros"));
    let request_policy = packaged_request_policy_summary_details();
    assert!(request_policy.validate().is_ok());
    assert!(request_policy.geocentric_only);
    assert_eq!(
        request_policy.supported_frames,
        &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial]
    );
    assert_eq!(
        request_policy.supported_time_scales,
        &[TimeScale::Tt, TimeScale::Tdb]
    );
    assert_eq!(
        request_policy.supported_zodiac_modes,
        &[ZodiacMode::Tropical]
    );
    assert_eq!(request_policy.supported_apparentness, &[Apparentness::Mean]);
    assert!(!request_policy.supports_topocentric_observer);
    assert_eq!(
        request_policy.lookup_epoch_policy,
        PackagedLookupEpochPolicy::RetagToTtGridWithoutRelativisticCorrection
    );
    assert_eq!(request_policy.lookup_epoch_policy.validate(), Ok(()));
    assert_eq!(
        request_policy.summary_line(),
        "Packaged request policy: geocentric-only; frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false; lookup epoch policy=TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
    );
    assert_eq!(
        request_policy.summary_line(),
        packaged_request_policy_summary()
    );
    assert_eq!(request_policy.to_string(), request_policy.summary_line());
    let lookup_epoch_policy = packaged_lookup_epoch_policy_summary_details();
    assert_eq!(
        lookup_epoch_policy.policy,
        request_policy.lookup_epoch_policy
    );
    assert_eq!(lookup_epoch_policy.policy.validate(), Ok(()));
    assert_eq!(lookup_epoch_policy.validate(), Ok(()));
    assert_eq!(
        lookup_epoch_policy.summary_line(),
        "TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
    );
    assert_eq!(
        lookup_epoch_policy.summary_line(),
        packaged_lookup_epoch_policy_summary()
    );
    assert_eq!(
        lookup_epoch_policy.to_string(),
        lookup_epoch_policy.summary_line()
    );
    assert_eq!(
        metadata.provenance.data_sources[1],
        packaged_request_policy_summary()
    );
    assert_eq!(
        metadata.provenance.data_sources[2],
        packaged_frame_treatment_summary()
    );
    assert_eq!(
        packaged_frame_treatment_summary_details().to_string(),
        packaged_frame_treatment_summary()
    );
    assert!(metadata.provenance.data_sources[2].contains("ecliptic coordinates directly"));
    assert_eq!(
        packaged_frame_treatment_summary_details().validate(),
        Ok(())
    );
    assert_eq!(
        packaged_frame_treatment_summary_details().validated_summary_line(),
        Ok(packaged_frame_treatment_summary_details().summary_line())
    );
    assert!(
        metadata.provenance.data_sources[2].contains("equatorial coordinates are reconstructed")
    );
    assert_eq!(
        metadata.provenance.data_sources[3],
        packaged_artifact_storage_summary()
    );
    assert_eq!(
        packaged_artifact_storage_summary_details().to_string(),
        packaged_artifact_storage_summary()
    );
    assert!(metadata.provenance.data_sources[3].contains("Quantized linear segments"));
    assert!(metadata.provenance.data_sources[3]
        .contains("body-indexed segment tables support random access by body and lookup time across the advertised range"));
    assert!(metadata.provenance.data_sources[3]
        .contains("ecliptic and equatorial coordinates are reconstructed at runtime"));
    assert!(metadata.provenance.data_sources[3]
        .contains("apparent, topocentric, and sidereal outputs remain unsupported; motion/speed is derived from fitted segment derivatives"));
    assert_eq!(
        packaged_artifact_storage_summary_details().validate(),
        Ok(())
    );
    assert_eq!(
        metadata.provenance.data_sources[4],
        packaged_artifact_access_summary()
    );
    assert_eq!(
        packaged_artifact_access_summary_details().to_string(),
        packaged_artifact_access_summary()
    );
    assert!(metadata.provenance.data_sources[4].contains("checked-in fixture"));
    assert_eq!(
        packaged_artifact_access_summary_details().validate(),
        Ok(())
    );
}

#[test]
fn packaged_backend_returns_motion_for_a_major_body() {
    let backend = packaged_backend();
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let res = backend
        .position(&EphemerisRequest::new(CelestialBody::Mars, inst))
        .expect("mars position");
    let motion = res.motion.expect("motion should be populated");
    assert!(motion.longitude_deg_per_day.unwrap().is_finite());
    // Mars mean motion is well under 1 deg/day in magnitude.
    assert!(motion.longitude_deg_per_day.unwrap().abs() < 1.0);
}

#[test]
fn packaged_backend_serves_osculating_true_apsides() {
    use pleiades_backend::{Apparentness, EphemerisBackend, EphemerisRequest};

    let backend = PackagedDataBackend::new();
    assert!(backend.supports_body(CelestialBody::TrueApogee));
    assert!(backend.supports_body(CelestialBody::TruePerigee));

    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let apo = backend
        .position(&EphemerisRequest::new(CelestialBody::TrueApogee, instant))
        .unwrap();
    let apo_ecl = apo.ecliptic.unwrap();

    // Mean-J2000 geometric direction at the backend boundary.
    assert_eq!(apo.apparent, Apparentness::Mean);
    // Apse line lies in the inclined lunar orbit → |β| up to ~6°.
    assert!(
        apo_ecl.latitude.degrees().abs() <= 6.5,
        "β {}",
        apo_ecl.latitude.degrees()
    );
    // Distance is a Moon-scale apogee (~0.0027 AU).
    let d = apo_ecl.distance_au.unwrap();
    assert!((0.0023..0.0030).contains(&d), "apogee distance {d} AU");
    // Derived motion present.
    assert!(apo.motion.unwrap().longitude_deg_per_day.is_some());

    // Perigee is the opposite apse (~180° away).
    let peri = backend
        .position(&EphemerisRequest::new(CelestialBody::TruePerigee, instant))
        .unwrap()
        .ecliptic
        .unwrap();
    let sep = (apo_ecl.longitude.degrees() - peri.longitude.degrees()).rem_euclid(360.0);
    assert!(
        (sep - 180.0).abs() < 1.0,
        "apogee/perigee separation {sep}°"
    );

    // The body appears as ReleaseGrade in metadata.
    let claim = backend
        .metadata()
        .body_claims
        .into_iter()
        .find(|c| c.body == CelestialBody::TrueApogee)
        .expect("TrueApogee claim present");
    assert_eq!(claim.tier, pleiades_backend::BodyClaimTier::ReleaseGrade);
}

#[test]
fn osculating_apsis_motion_degrades_gracefully_at_coverage_boundary() {
    // At the coverage START boundary (JD 2415020.5 ≈ 1900-01-01), the −0.5 day motion
    // probe (JD 2415020.0) falls outside the packaged artifact's range. position()
    // must still return Ok with a valid ecliptic; motion components must all be None.
    use pleiades_backend::{Apparentness, EphemerisBackend, EphemerisRequest};

    let backend = PackagedDataBackend::new();
    let boundary = Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt);
    let result = backend
        .position(&EphemerisRequest::new(CelestialBody::TrueApogee, boundary))
        .expect("position at coverage boundary must succeed (not hard-fail)");

    let ecl = result.ecliptic.expect("ecliptic must be present");
    assert!(
        ecl.longitude.degrees().is_finite(),
        "ecliptic longitude must be finite"
    );

    let motion = result.motion.expect("motion field must be present");
    assert!(
        motion.longitude_deg_per_day.is_none(),
        "longitude motion must degrade to None at coverage boundary"
    );
    assert!(
        motion.latitude_deg_per_day.is_none(),
        "latitude motion must degrade to None at coverage boundary"
    );
    assert!(
        motion.distance_au_per_day.is_none(),
        "distance motion must degrade to None at coverage boundary"
    );
    assert_eq!(result.apparent, Apparentness::Mean);
}
