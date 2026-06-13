use super::*;
use crate::source_docs::{
    build_generated_binary_audits_with_lookup,
    format_validated_canonical_epoch_evidence_summary_for_report,
    format_validated_canonical_j1900_batch_parity_summary_for_report,
    format_validated_canonical_j2000_batch_parity_summary_for_report,
    format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report,
    format_validated_generated_binary_audit_summary_for_report,
    format_validated_source_audit_summary_for_report,
    format_validated_source_body_class_evidence_summary_for_report,
    format_validated_source_documentation_health_summary_for_report,
    format_validated_source_documentation_summary_for_report,
    format_validated_supported_body_canonical_batch_parity_summary_for_report,
    format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report,
    format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report,
    format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report,
    format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report,
    source_documentation_health_issues, CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
    CANONICAL_EVIDENCE_SUMMARY_LABEL,
};

#[test]
fn package_name_is_stable() {
    assert_eq!(PACKAGE_NAME, "pleiades-vsop87");
}

#[test]
fn backend_reports_major_planets() {
    let backend = Vsop87Backend::new();
    assert!(backend.supports_body(CelestialBody::Sun));
    assert!(backend.supports_body(CelestialBody::Mars));
    assert!(!backend.supports_body(CelestialBody::Moon));
}

#[test]
fn j2000_sun_position_uses_vendored_vsop87b_earth_file() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Sun);
    let result = backend.position(&request).expect("sun query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Earth file evaluated
    // at J2000 and converted to geometric geocentric solar coordinates.
    assert_degrees_close(ecliptic.longitude.degrees(), 280.377_843_416_648_5, 0.001);
    assert_degrees_close(
        ecliptic.latitude.degrees(),
        0.000_227_210_514_369_001,
        0.000_01,
    );
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        0.983_327_682_322_294_2,
        0.000_01,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn j2000_mercury_position_uses_vendored_vsop87b_mercury_file() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Mercury);
    let result = backend
        .position(&request)
        .expect("Mercury query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Mercury and Earth
    // files evaluated at J2000 and reduced to geometric geocentric ecliptic
    // coordinates.
    assert_degrees_close(
        ecliptic.longitude.degrees(),
        271.904_744_694_147_67,
        0.000_000_001,
    );
    assert_degrees_close(
        ecliptic.latitude.degrees(),
        -0.995_553_498_474_437_4,
        0.000_000_001,
    );
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        1.415_524_982_482_968,
        0.000_000_000_001,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn j2000_venus_position_uses_vendored_vsop87b_venus_file() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Venus);
    let result = backend.position(&request).expect("Venus query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Venus and Earth
    // files evaluated at J2000 and reduced to geometric geocentric ecliptic
    // coordinates.
    assert_degrees_close(ecliptic.longitude.degrees(), 241.576_729_276_029_5, 0.001);
    assert_degrees_close(ecliptic.latitude.degrees(), 2.066_187_460_260_189, 0.000_1);
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        1.137_689_108_663_588,
        0.000_01,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn j2000_mars_position_uses_generated_vsop87b_mars_table() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Mars);
    let result = backend.position(&request).expect("Mars query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Mars and Earth
    // files evaluated at J2000 and reduced to geometric geocentric ecliptic
    // coordinates. The runtime path now reaches them through the generated
    // binary Mars table derived from the vendored Mars source file.
    assert_degrees_close(
        ecliptic.longitude.degrees(),
        327.974_906_233_385_87,
        0.000_000_001,
    );
    assert_degrees_close(
        ecliptic.latitude.degrees(),
        -1.067_660_978_531_137_7,
        0.000_000_001,
    );
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        1.849_603_891_293_057_7,
        0.000_000_000_001,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn j2000_jupiter_position_uses_full_vsop87b_jupiter_file() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Jupiter);
    let result = backend
        .position(&request)
        .expect("Jupiter query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Jupiter and Earth
    // files evaluated at J2000 and reduced to geometric geocentric ecliptic
    // coordinates.
    assert_degrees_close(ecliptic.longitude.degrees(), 25.258_084_319_944_018, 0.004);
    assert_degrees_close(
        ecliptic.latitude.degrees(),
        -1.262_035_369_214_697_3,
        0.000_2,
    );
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        4.621_126_218_764_805,
        0.000_1,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn j2000_saturn_position_uses_full_vsop87b_saturn_file() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Saturn);
    let result = backend
        .position(&request)
        .expect("Saturn query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Saturn and Earth
    // files evaluated at J2000 and reduced to geometric geocentric ecliptic
    // coordinates.
    assert_degrees_close(ecliptic.longitude.degrees(), 40.398_572_276_886_384, 0.004);
    assert_degrees_close(
        ecliptic.latitude.degrees(),
        -2.444_625_745_599_142_3,
        0.000_2,
    );
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        8.652_748_862_003_302,
        0.000_5,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn j2000_uranus_position_uses_full_vsop87b_uranus_file() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Uranus);
    let result = backend
        .position(&request)
        .expect("Uranus query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Uranus and Earth
    // files evaluated at J2000 and reduced to geometric geocentric ecliptic
    // coordinates.
    assert_degrees_close(ecliptic.longitude.degrees(), 314.819_126_206_595_1, 0.006);
    assert_degrees_close(
        ecliptic.latitude.degrees(),
        -0.658_295_956_624_516_5,
        0.000_1,
    );
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        20.727_185_531_715_136,
        0.000_1,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn j2000_neptune_position_uses_full_vsop87b_neptune_file() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Neptune);
    let result = backend
        .position(&request)
        .expect("Neptune query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");

    // Golden values are the full public IMCCE VSOP87B Neptune and Earth
    // files evaluated at J2000 and reduced to geometric geocentric ecliptic
    // coordinates.
    assert_degrees_close(ecliptic.longitude.degrees(), 303.203_423_517_050_34, 0.001);
    assert_degrees_close(
        ecliptic.latitude.degrees(),
        0.234_955_476_702_893_77,
        0.000_1,
    );
    assert_close(
        ecliptic.distance_au.expect("distance should exist"),
        31.024_432_860_406_91,
        0.000_1,
    );
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn batch_query_covers_all_source_backed_vsop87_paths() {
    let backend = Vsop87Backend::new();
    let samples = canonical_epoch_samples();
    let requests = samples
        .iter()
        .map(|sample| mean_request(sample.body.clone()))
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch query should work for every source-backed body");

    assert_eq!(results.len(), samples.len());
    for (sample, result) in samples.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert_degrees_close(
            ecliptic.longitude.degrees(),
            sample.expected_longitude_deg,
            sample.max_longitude_delta_deg,
        );
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            sample.expected_latitude_deg,
            sample.max_latitude_delta_deg,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            sample.expected_distance_au,
            sample.max_distance_delta_au,
        );
        let expected_quality = QualityAnnotation::Exact;
        assert_eq!(result.quality, expected_quality);
    }
}

#[test]
fn batch_query_covers_all_supported_vsop87_paths() {
    let backend = Vsop87Backend::new();
    let requests = Vsop87Backend::supported_bodies()
        .iter()
        .cloned()
        .map(mean_request)
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch query should work for every supported body");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        match result.body {
            CelestialBody::Pluto => {
                assert_eq!(result.quality, QualityAnnotation::Approximate);
            }
            _ => {
                assert_eq!(result.quality, QualityAnnotation::Exact);
            }
        }

        let single = backend
            .position(request)
            .expect("single query should work for every supported body");
        assert_eq!(single.body, result.body);
        assert_eq!(single.quality, result.quality);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        let single_ecliptic = single
            .ecliptic
            .as_ref()
            .expect("single-query ecliptic result should exist");
        assert_eq!(
            ecliptic.longitude.degrees(),
            single_ecliptic.longitude.degrees()
        );
        assert_eq!(
            ecliptic.latitude.degrees(),
            single_ecliptic.latitude.degrees()
        );
        assert_eq!(
            ecliptic.distance_au.expect("distance should exist"),
            single_ecliptic
                .distance_au
                .expect("single-query distance should exist")
        );

        if let Some(sample) = canonical_epoch_samples()
            .into_iter()
            .find(|sample| sample.body == result.body)
        {
            assert_degrees_close(
                ecliptic.longitude.degrees(),
                sample.expected_longitude_deg,
                sample.max_longitude_delta_deg,
            );
            assert_degrees_close(
                ecliptic.latitude.degrees(),
                sample.expected_latitude_deg,
                sample.max_latitude_delta_deg,
            );
            assert_close(
                ecliptic.distance_au.expect("distance should exist"),
                sample.expected_distance_au,
                sample.max_distance_delta_au,
            );
        }
    }
}

#[test]
fn batch_query_preserves_supported_vsop87_paths_for_tdb_requests() {
    let backend = Vsop87Backend::new();
    let requests = Vsop87Backend::supported_bodies()
        .iter()
        .cloned()
        .map(|body| {
            let mut request = mean_request(body);
            request.instant.scale = TimeScale::Tdb;
            request
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch TDB query should work for every supported body");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant.scale, TimeScale::Tdb);
        match result.body {
            CelestialBody::Pluto => {
                assert_eq!(result.quality, QualityAnnotation::Approximate);
            }
            _ => {
                assert_eq!(result.quality, QualityAnnotation::Exact);
            }
        }

        let single = backend
            .position(request)
            .expect("single TDB query should work for every supported body");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant.scale, TimeScale::Tdb);
        assert_eq!(single.quality, result.quality);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        let single_ecliptic = single
            .ecliptic
            .as_ref()
            .expect("single-query ecliptic result should exist");
        assert_eq!(
            ecliptic.longitude.degrees(),
            single_ecliptic.longitude.degrees()
        );
        assert_eq!(
            ecliptic.latitude.degrees(),
            single_ecliptic.latitude.degrees()
        );
        assert_eq!(
            ecliptic.distance_au.expect("distance should exist"),
            single_ecliptic
                .distance_au
                .expect("single-query distance should exist")
        );

        if let Some(sample) = canonical_epoch_samples()
            .into_iter()
            .find(|sample| sample.body == result.body)
        {
            assert_degrees_close(
                ecliptic.longitude.degrees(),
                sample.expected_longitude_deg,
                sample.max_longitude_delta_deg,
            );
            assert_degrees_close(
                ecliptic.latitude.degrees(),
                sample.expected_latitude_deg,
                sample.max_latitude_delta_deg,
            );
            assert_close(
                ecliptic.distance_au.expect("distance should exist"),
                sample.expected_distance_au,
                sample.max_distance_delta_au,
            );
        }
    }
}

#[test]
fn batch_query_preserves_supported_vsop87_paths_for_mixed_time_scales() {
    let backend = Vsop87Backend::new();
    let requests = Vsop87Backend::supported_bodies()
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, body)| {
            let mut request = mean_request(body);
            request.instant.scale = if index % 2 == 0 {
                TimeScale::Tt
            } else {
                TimeScale::Tdb
            };
            request
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch mixed-scale query should work for every supported body");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant.scale, request.instant.scale);
        let single = backend
            .position(request)
            .expect("single mixed-scale query should work for every supported body");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant.scale, request.instant.scale);
        assert_eq!(single.quality, result.quality);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        let single_ecliptic = single
            .ecliptic
            .as_ref()
            .expect("single-query ecliptic result should exist");
        assert_eq!(
            ecliptic.longitude.degrees(),
            single_ecliptic.longitude.degrees()
        );
        assert_eq!(
            ecliptic.latitude.degrees(),
            single_ecliptic.latitude.degrees()
        );
        assert_eq!(
            ecliptic.distance_au.expect("distance should exist"),
            single_ecliptic
                .distance_au
                .expect("single-query distance should exist")
        );
    }
}

#[test]
fn batch_query_preserves_canonical_sample_order_for_source_backed_paths() {
    let backend = Vsop87Backend::new();
    let mut requests = canonical_epoch_requests();
    let mut samples = canonical_epoch_samples();
    requests.reverse();
    samples.reverse();

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve input order for every source-backed body");

    assert_eq!(results.len(), samples.len());
    for (sample, result) in samples.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert_degrees_close(
            ecliptic.longitude.degrees(),
            sample.expected_longitude_deg,
            sample.max_longitude_delta_deg,
        );
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            sample.expected_latitude_deg,
            sample.max_latitude_delta_deg,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            sample.expected_distance_au,
            sample.max_distance_delta_au,
        );
    }
}

#[test]
fn batch_query_preserves_equatorial_frame_and_values() {
    let backend = Vsop87Backend::new();
    let mut requests = canonical_epoch_requests();
    let mut samples = canonical_epoch_samples();
    requests.reverse();
    samples.reverse();
    for request in &mut requests {
        request.frame = CoordinateFrame::Equatorial;
    }

    let results = backend
        .positions(&requests)
        .expect("batch equatorial query should preserve the canonical sample order");

    assert_eq!(results.len(), samples.len());
    for (sample, result) in samples.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert_degrees_close(
            ecliptic.longitude.degrees(),
            sample.expected_longitude_deg,
            sample.max_longitude_delta_deg,
        );
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            sample.expected_latitude_deg,
            sample.max_latitude_delta_deg,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            sample.expected_distance_au,
            sample.max_distance_delta_au,
        );

        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .as_ref()
            .expect("equatorial result should exist");

        assert_eq!(equatorial, &expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn batch_query_preserves_supported_vsop87_paths_at_the_j1900_reference_epoch() {
    let backend = Vsop87Backend::new();
    let requests = requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb),
        CoordinateFrame::Equatorial,
    );

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve the supported planetary set at J1900");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        match result.body {
            CelestialBody::Pluto => {
                assert_eq!(result.quality, QualityAnnotation::Approximate);
            }
            _ => {
                assert_eq!(result.quality, QualityAnnotation::Exact);
            }
        }

        let single = backend
            .position(request)
            .expect("single query should match the J1900 batch path");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant, result.instant);
        assert_eq!(single.frame, result.frame);
        assert_eq!(single.quality, result.quality);
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.equatorial, result.equatorial);
        assert_eq!(single.motion, result.motion);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());

        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .as_ref()
            .expect("equatorial result should exist");

        assert_eq!(equatorial, &expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn batch_query_preserves_supported_vsop87_paths_at_the_j1900_ecliptic_reference_epoch() {
    let backend = Vsop87Backend::new();
    let requests = requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb),
        CoordinateFrame::Ecliptic,
    );

    let results = backend.positions(&requests).expect(
        "batch query should preserve the supported planetary set at J1900 in ecliptic frame",
    );

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, CoordinateFrame::Ecliptic);
        match result.body {
            CelestialBody::Pluto => {
                assert_eq!(result.quality, QualityAnnotation::Approximate);
            }
            _ => {
                assert_eq!(result.quality, QualityAnnotation::Exact);
            }
        }

        let single = backend
            .position(request)
            .expect("single query should match the J1900 ecliptic batch path");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant, result.instant);
        assert_eq!(single.frame, result.frame);
        assert_eq!(single.quality, result.quality);
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.equatorial, result.equatorial);
        assert_eq!(single.motion, result.motion);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());

        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .as_ref()
            .expect("equatorial result should exist");

        assert_eq!(equatorial, &expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn batch_query_preserves_supported_vsop87_paths_at_the_j2000_reference_epoch() {
    let backend = Vsop87Backend::new();
    let requests = requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
        CoordinateFrame::Equatorial,
    );

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve the supported planetary set at J2000");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        match result.body {
            CelestialBody::Pluto => {
                assert_eq!(result.quality, QualityAnnotation::Approximate);
            }
            _ => {
                assert_eq!(result.quality, QualityAnnotation::Exact);
            }
        }

        let single = backend
            .position(request)
            .expect("single query should match the J2000 batch path");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant, result.instant);
        assert_eq!(single.frame, result.frame);
        assert_eq!(single.quality, result.quality);
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.equatorial, result.equatorial);
        assert_eq!(single.motion, result.motion);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());

        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .as_ref()
            .expect("equatorial result should exist");

        assert_eq!(equatorial, &expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn batch_query_preserves_supported_vsop87_paths_at_the_j2000_reference_epoch_in_equatorial_frame() {
    let backend = Vsop87Backend::new();
    let requests = requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
        CoordinateFrame::Equatorial,
    );

    let results = backend.positions(&requests).expect(
        "batch query should preserve the supported planetary set at J2000 in equatorial frame",
    );

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        match result.body {
            CelestialBody::Pluto => {
                assert_eq!(result.quality, QualityAnnotation::Approximate);
            }
            _ => {
                assert_eq!(result.quality, QualityAnnotation::Exact);
            }
        }

        let single = backend
            .position(request)
            .expect("single query should match the J2000 equatorial batch path");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant, result.instant);
        assert_eq!(single.frame, result.frame);
        assert_eq!(single.quality, result.quality);
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.equatorial, result.equatorial);
        assert_eq!(single.motion, result.motion);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());

        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .as_ref()
            .expect("equatorial result should exist");

        assert_eq!(equatorial, &expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn supported_body_j2000_ecliptic_batch_parity_report_matches_the_backend_formatter() {
    let summary =
        supported_body_j2000_ecliptic_batch_parity_summary().expect("batch summary should exist");
    let rendered = supported_body_j2000_ecliptic_batch_parity_summary_for_report();

    assert_eq!(summary.summary_line(), rendered);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert!(rendered.contains("VSOP87 supported-body J2000 ecliptic batch parity:"));
    assert!(rendered.contains("batch/single parity preserved"));
    assert!(rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto"));
}

#[test]
fn supported_body_j2000_ecliptic_batch_parity_report_surfaces_validation_errors() {
    let mut summary =
        supported_body_j2000_ecliptic_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Equatorial;

    assert_eq!(
        format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report(
            &summary
        ),
        "VSOP87 supported-body J2000 ecliptic batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn supported_body_j2000_equatorial_batch_parity_report_matches_the_backend_formatter() {
    let summary =
        supported_body_j2000_equatorial_batch_parity_summary().expect("batch summary should exist");
    let rendered = supported_body_j2000_equatorial_batch_parity_summary_for_report();

    assert_eq!(summary.summary_line(), rendered);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Equatorial);
    assert!(rendered.contains("VSOP87 supported-body J2000 equatorial batch parity:"));
    assert!(rendered.contains("batch/single parity preserved"));
    assert!(rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto"));
}

#[test]
fn supported_body_j2000_equatorial_batch_parity_report_surfaces_validation_errors() {
    let mut summary =
        supported_body_j2000_equatorial_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Ecliptic;

    assert_eq!(
        format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report(
            &summary
        ),
        "VSOP87 supported-body J2000 equatorial batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn supported_body_j1900_ecliptic_batch_parity_report_matches_the_backend_formatter() {
    let summary =
        supported_body_j1900_ecliptic_batch_parity_summary().expect("batch summary should exist");
    let rendered = supported_body_j1900_ecliptic_batch_parity_summary_for_report();

    assert_eq!(summary.summary_line(), rendered);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert!(rendered.contains("VSOP87 supported-body J1900 ecliptic batch parity:"));
    assert!(rendered.contains("batch/single parity preserved"));
    assert!(rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto"));
}

#[test]
fn supported_body_j1900_ecliptic_batch_parity_report_surfaces_validation_errors() {
    let mut summary =
        supported_body_j1900_ecliptic_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Equatorial;

    assert_eq!(
        format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report(
            &summary
        ),
        "VSOP87 supported-body J1900 ecliptic batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn supported_body_j1900_equatorial_batch_parity_report_matches_the_backend_formatter() {
    let summary =
        supported_body_j1900_equatorial_batch_parity_summary().expect("batch summary should exist");
    let rendered = supported_body_j1900_equatorial_batch_parity_summary_for_report();

    assert_eq!(summary.summary_line(), rendered);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Equatorial);
    assert!(rendered.contains("VSOP87 supported-body J1900 equatorial batch parity:"));
    assert!(rendered.contains("batch/single parity preserved"));
    assert!(rendered.contains("Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto"));
}

#[test]
fn supported_body_j1900_equatorial_batch_parity_report_surfaces_validation_errors() {
    let mut summary =
        supported_body_j1900_equatorial_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Ecliptic;

    assert_eq!(
        format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report(
            &summary
        ),
        "VSOP87 supported-body J1900 equatorial batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn batch_query_preserves_supported_vsop87_paths_at_the_j2000_reference_epoch_in_ecliptic_frame() {
    let backend = Vsop87Backend::new();
    let requests = requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
        CoordinateFrame::Ecliptic,
    );

    let results = backend.positions(&requests).expect(
        "batch query should preserve the supported planetary set at J2000 in ecliptic frame",
    );

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, CoordinateFrame::Ecliptic);
        match result.body {
            CelestialBody::Pluto => {
                assert_eq!(result.quality, QualityAnnotation::Approximate);
            }
            _ => {
                assert_eq!(result.quality, QualityAnnotation::Exact);
            }
        }

        let single = backend
            .position(request)
            .expect("single query should match the J2000 ecliptic batch path");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant, result.instant);
        assert_eq!(single.frame, result.frame);
        assert_eq!(single.quality, result.quality);
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.equatorial, result.equatorial);
        assert_eq!(single.motion, result.motion);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());

        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .as_ref()
            .expect("equatorial result should exist");

        assert_eq!(equatorial, &expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn batch_query_preserves_mixed_frame_requests_and_values() {
    let backend = Vsop87Backend::new();
    let requests = canonical_epoch_requests()
        .into_iter()
        .enumerate()
        .map(|(index, mut request)| {
            request.frame = if index % 2 == 0 {
                CoordinateFrame::Ecliptic
            } else {
                CoordinateFrame::Equatorial
            };
            request
        })
        .collect::<Vec<_>>();
    let samples = canonical_epoch_samples();

    let results = backend
        .positions(&requests)
        .expect("mixed frame batch query should preserve the canonical sample order");

    assert_eq!(results.len(), samples.len());
    for ((request, result), sample) in requests.iter().zip(results.iter()).zip(samples.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, request.frame);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert_degrees_close(
            ecliptic.longitude.degrees(),
            sample.expected_longitude_deg,
            sample.max_longitude_delta_deg,
        );
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            sample.expected_latitude_deg,
            sample.max_latitude_delta_deg,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            sample.expected_distance_au,
            sample.max_distance_delta_au,
        );

        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .as_ref()
            .expect("equatorial result should exist");

        assert_eq!(equatorial, &expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn finite_difference_motion_is_reported_for_supported_bodies() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Mars);
    let result = backend.position(&request).expect("Mars query should work");
    let motion = result.motion.expect("motion should be populated");

    assert!(motion
        .longitude_deg_per_day
        .expect("longitude speed should exist")
        .is_finite());
    assert!(motion
        .latitude_deg_per_day
        .expect("latitude speed should exist")
        .is_finite());
    assert!(motion
        .distance_au_per_day
        .expect("distance speed should exist")
        .is_finite());
}

#[test]
fn topocentric_requests_are_rejected_explicitly() {
    let backend = Vsop87Backend::new();
    let mut request = mean_request(CelestialBody::Mars);
    request.observer = Some(pleiades_types::ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(0.0),
        None,
    ));

    let error = backend
        .position(&request)
        .expect_err("topocentric requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn batch_query_rejects_topocentric_requests_explicitly() {
    let backend = Vsop87Backend::new();
    let mut request = mean_request(CelestialBody::Mars);
    request.observer = Some(pleiades_types::ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(0.0),
        None,
    ));

    let error = backend
        .positions(&[request])
        .expect_err("topocentric batch requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn apparent_requests_are_rejected_explicitly() {
    let backend = Vsop87Backend::new();
    let mut request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    );
    request.apparent = Apparentness::Apparent;

    let error = backend
        .position(&request)
        .expect_err("apparent requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(BACKEND_LABEL));
}

#[test]
fn batch_query_rejects_apparent_requests_explicitly() {
    let backend = Vsop87Backend::new();
    let mut request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    );
    request.apparent = Apparentness::Apparent;

    let error = backend
        .positions(&[request])
        .expect_err("apparent batch requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(BACKEND_LABEL));
}

#[test]
fn unsupported_time_scales_are_rejected_explicitly() {
    let backend = Vsop87Backend::new();
    let request = EphemerisRequest::new(
        CelestialBody::Mars,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Utc),
    );

    let error = backend
        .position(&request)
        .expect_err("UTC requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
}

#[test]
fn batch_query_rejects_unsupported_time_scales_explicitly() {
    let backend = Vsop87Backend::new();
    let request = EphemerisRequest::new(
        CelestialBody::Mars,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Utc),
    );

    let error = backend
        .positions(&[request])
        .expect_err("UTC batch requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
}

#[test]
fn unsupported_bodies_report_the_current_backend_label() {
    let backend = Vsop87Backend::new();
    let request = mean_request(CelestialBody::Moon);

    let error = backend
        .position(&request)
        .expect_err("moon requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert!(error.message.contains(BACKEND_LABEL));
}

#[test]
fn tdb_requests_are_accepted_like_tt_requests() {
    let backend = Vsop87Backend::new();
    let tt_request = mean_request(CelestialBody::Mars);
    let tdb_request = EphemerisRequest::new(
        CelestialBody::Mars,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
    );

    let tt_result = backend
        .position(&tt_request)
        .expect("TT request should be supported");
    let tdb_result = backend
        .position(&tdb_request)
        .expect("TDB request should be supported");

    assert_eq!(tt_result.body, tdb_result.body);
    assert_eq!(tt_result.instant.scale, TimeScale::Tt);
    assert_eq!(tdb_result.instant.scale, TimeScale::Tdb);
    assert_eq!(tt_result.ecliptic, tdb_result.ecliptic);
    assert_eq!(tt_result.equatorial, tdb_result.equatorial);
    assert_eq!(tt_result.motion, tdb_result.motion);
}

#[test]
fn batch_query_preserves_mixed_time_scales_and_values() {
    let backend = Vsop87Backend::new();
    let requests = canonical_epoch_requests()
        .into_iter()
        .enumerate()
        .map(|(index, mut request)| {
            request.instant.scale = if index % 2 == 0 {
                TimeScale::Tt
            } else {
                TimeScale::Tdb
            };
            request
        })
        .collect::<Vec<_>>();
    let samples = canonical_epoch_samples();

    let results = backend
        .positions(&requests)
        .expect("mixed-scale batch query should preserve the canonical sample order");

    assert_eq!(results.len(), samples.len());
    for ((request, result), sample) in requests.iter().zip(results.iter()).zip(samples.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.instant.scale, request.instant.scale);

        let single = backend
            .position(request)
            .expect("single mixed-scale query should preserve the canonical sample order");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant, result.instant);
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.equatorial, result.equatorial);
        assert_eq!(single.motion, result.motion);

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("ecliptic result should exist");
        assert_degrees_close(
            ecliptic.longitude.degrees(),
            sample.expected_longitude_deg,
            sample.max_longitude_delta_deg,
        );
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            sample.expected_latitude_deg,
            sample.max_latitude_delta_deg,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            sample.expected_distance_au,
            sample.max_distance_delta_au,
        );
    }
}

#[test]
fn body_source_profiles_validate_the_current_catalog_pairings() {
    let profiles = body_source_profiles();
    assert!(profiles.iter().all(|profile| profile.validate().is_ok()));

    let profile = profiles
        .iter()
        .find(|profile| profile.body == CelestialBody::Pluto)
        .expect("Pluto profile should exist");

    let mut kind_drift = profile.clone();
    kind_drift.kind = match kind_drift.kind {
        Vsop87BodySourceKind::MeanOrbitalElements => Vsop87BodySourceKind::GeneratedBinaryVsop87b,
        _ => Vsop87BodySourceKind::MeanOrbitalElements,
    };
    assert!(matches!(
        kind_drift.validate(),
        Err(Vsop87BodySourceValidationError::SourceKindMismatch { .. })
    ));

    let mut provenance_drift = profile.clone();
    provenance_drift.provenance = " drifted provenance ";
    assert!(matches!(
        provenance_drift.validate(),
        Err(Vsop87BodySourceValidationError::WhitespacePaddedProvenance { .. })
    ));
}

#[test]
fn metadata_identifies_source_backed_planet_vsop87b_paths() {
    let metadata = Vsop87Backend::new().metadata();
    assert!(metadata
        .provenance
        .summary
        .contains("8 generated binary VSOP87B body paths"));
    assert!(!metadata
        .provenance
        .summary
        .contains("vendored full-file VSOP87B body paths"));
    assert!(metadata
        .provenance
        .summary
        .contains("1 fallback mean-element body path"));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Sun: IMCCE/CELMECH VSOP87B VSOP87B.ear")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("mean-obliquity transform")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("outside the source-backed VSOP87 coefficient tables")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Mercury: IMCCE/CELMECH VSOP87B VSOP87B.mer")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Venus: IMCCE/CELMECH VSOP87B VSOP87B.ven")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Mars: IMCCE/CELMECH VSOP87B VSOP87B.mar")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Jupiter: IMCCE/CELMECH VSOP87B VSOP87B.jup")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Saturn: IMCCE/CELMECH VSOP87B VSOP87B.sat")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Uranus: IMCCE/CELMECH VSOP87B VSOP87B.ura")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Neptune: IMCCE/CELMECH VSOP87B VSOP87B.nep")));
    assert_eq!(
        metadata.supported_time_scales,
        vec![TimeScale::Tt, TimeScale::Tdb]
    );
}

#[test]
fn body_source_profiles_identify_generated_binary_and_full_file_paths() {
    let profiles = body_source_profiles();
    assert_eq!(profiles.len(), Vsop87Backend::supported_bodies().len());

    for body in [
        CelestialBody::Sun,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
    ] {
        let profile = profiles
            .iter()
            .find(|profile| profile.body == body)
            .expect("source profile should exist");
        assert_eq!(profile.kind, Vsop87BodySourceKind::GeneratedBinaryVsop87b);
        assert_eq!(profile.accuracy, AccuracyClass::Exact);
        assert!(profile
            .provenance
            .contains("vendored full IMCCE/CELMECH VSOP87B"));
        assert_eq!(profile.summary_line(), profile.to_string());
    }

    let sun = profiles
        .iter()
        .find(|profile| profile.body == CelestialBody::Sun)
        .expect("Sun profile should exist");
    assert!(sun
        .summary_line()
        .starts_with("Sun: kind=generated binary VSOP87B, accuracy=Exact"));
    assert!(sun
        .summary_line()
        .contains("vendored full IMCCE/CELMECH VSOP87B"));

    let pluto = profiles
        .iter()
        .find(|profile| profile.body == CelestialBody::Pluto)
        .expect("Pluto profile should exist");
    assert_eq!(pluto.kind, Vsop87BodySourceKind::MeanOrbitalElements);
    assert!(pluto.provenance.contains("fallback"));
    assert_eq!(pluto.summary_line(), pluto.to_string());
    assert!(pluto
        .summary_line()
        .starts_with("Pluto: kind=mean orbital elements fallback, accuracy=Approximate"));
}

#[test]
fn canonical_epoch_samples_cover_source_backed_paths() {
    let samples = canonical_epoch_samples();
    assert_eq!(samples.len(), 8);
    assert!(samples
        .iter()
        .any(|sample| sample.body == CelestialBody::Sun));
    assert!(samples
        .iter()
        .any(|sample| sample.body == CelestialBody::Mercury));
    assert!(samples
        .iter()
        .any(|sample| sample.body == CelestialBody::Neptune));
    assert!(samples
        .iter()
        .all(|sample| sample.max_longitude_delta_deg > 0.0));
    assert!(samples
        .iter()
        .all(|sample| sample.max_latitude_delta_deg > 0.0));
    assert!(samples
        .iter()
        .all(|sample| sample.max_distance_delta_au > 0.0));
}

#[test]
fn canonical_epoch_error_envelope_matches_the_public_sample_catalog() {
    let samples = canonical_epoch_samples();
    let body_evidence = canonical_epoch_body_evidence().expect("evidence should exist");
    let summary = canonical_epoch_evidence_summary().expect("summary should exist");

    assert_eq!(body_evidence.len(), samples.len());
    assert_eq!(summary.sample_count, samples.len());
    assert_eq!(
        summary.sample_bodies,
        samples
            .iter()
            .map(|sample| sample.body.clone())
            .collect::<Vec<_>>()
    );
    assert!(summary.within_interim_limits);
    assert!(body_evidence
        .iter()
        .all(|evidence| evidence.within_interim_limits));
    assert!(body_evidence
        .iter()
        .any(|evidence| evidence.source_kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b));
    assert!(summary.max_longitude_delta_deg > 0.0);
    assert!(summary.max_latitude_delta_deg > 0.0);
    assert!(summary.max_distance_delta_au > 0.0);
    assert!(summary.mean_longitude_delta_deg > 0.0);
    assert!(summary.median_longitude_delta_deg > 0.0);
    assert!(summary.rms_longitude_delta_deg > 0.0);
    assert!(summary.percentile_longitude_delta_deg > 0.0);
    assert!(summary.mean_latitude_delta_deg > 0.0);
    assert!(summary.median_latitude_delta_deg > 0.0);
    assert!(summary.percentile_latitude_delta_deg > 0.0);
    assert!(summary.rms_latitude_delta_deg > 0.0);
    assert!(summary.mean_distance_delta_au > 0.0);
    assert!(summary.median_distance_delta_au > 0.0);
    assert!(summary.percentile_distance_delta_au > 0.0);
    assert!(summary.rms_distance_delta_au > 0.0);
    assert_eq!(summary.out_of_limit_count, 0);
    assert!(body_evidence
        .iter()
        .any(|evidence| evidence.body == summary.max_longitude_delta_body));
    assert!(body_evidence
        .iter()
        .any(|evidence| evidence.body == summary.max_latitude_delta_body));
    assert!(body_evidence
        .iter()
        .any(|evidence| evidence.body == summary.max_distance_delta_body));
    let max_longitude = body_evidence
        .iter()
        .find(|evidence| evidence.body == summary.max_longitude_delta_body)
        .expect("max longitude body should exist");
    let max_latitude = body_evidence
        .iter()
        .find(|evidence| evidence.body == summary.max_latitude_delta_body)
        .expect("max latitude body should exist");
    let max_distance = body_evidence
        .iter()
        .find(|evidence| evidence.body == summary.max_distance_delta_body)
        .expect("max distance body should exist");
    assert_eq!(
        summary.max_longitude_delta_source_kind,
        max_longitude.source_kind
    );
    assert_eq!(
        summary.max_longitude_delta_source_file,
        max_longitude.source_file
    );
    assert_eq!(
        summary.max_latitude_delta_source_kind,
        max_latitude.source_kind
    );
    assert_eq!(
        summary.max_latitude_delta_source_file,
        max_latitude.source_file
    );
    assert_eq!(
        summary.max_distance_delta_source_kind,
        max_distance.source_kind
    );
    assert_eq!(
        summary.max_distance_delta_source_file,
        max_distance.source_file
    );
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn canonical_epoch_error_envelope_validation_rejects_out_of_limit_count_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.out_of_limit_count = 1;

    let error = summary
        .validate()
        .expect_err("drifted canonical evidence summaries should fail validation");

    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed evidence summary field `out_of_limit_count` is out of sync with the current canonical evidence"
    );
    assert_eq!(
        format_validated_canonical_epoch_evidence_summary_for_report(&summary),
        "VSOP87 canonical J2000 source-backed evidence: unavailable (the VSOP87 canonical J2000 source-backed evidence summary field `out_of_limit_count` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn source_specifications_document_variant_frames_units_and_range() {
    let specs = source_specifications();
    assert_eq!(specs.len(), 8);
    assert!(validate_source_specifications(&specs).is_ok());
    assert!(specs.iter().all(|spec| spec.variant == "VSOP87B"));
    assert!(specs
        .iter()
        .all(|spec| spec.frame == "J2000 ecliptic/equinox"));
    assert!(specs
        .iter()
        .all(|spec| spec.units == "degrees and astronomical units"));
    assert!(specs
        .iter()
        .any(|spec| spec.reduction.contains("solar reduction")));
    assert!(specs
        .iter()
        .all(|spec| spec.reduction.contains("geocentric")));
    assert!(specs.iter().all(|spec| {
        spec.truncation_policy
            == "generated binary coefficient table derived from vendored full source file"
    }));
    assert!(!specs
        .iter()
        .any(|spec| spec.truncation_policy == "vendored full source file"));
    assert!(specs.iter().all(|spec| spec
        .date_range
        .contains("full public source file; J2000 canonical reference sample")));
    assert!(specs
        .iter()
        .all(|spec| spec.transform_note.contains("mean-obliquity transform")));
    assert!(specs.iter().any(|spec| spec.source_file == "VSOP87B.nep"));
}

#[test]
fn source_specification_validation_rejects_blank_metadata() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    spec.frame = "   ";

    let error = spec
        .validate()
        .expect_err("blank source-specification fields should fail validation");

    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {} has a blank `frame` field",
            spec.body
        )
    );
}

#[test]
fn source_specification_validation_rejects_blank_date_range() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.date_range = "\t";

    let error = spec
        .validate()
        .expect_err("blank source-specification fields should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::BlankField {
            body: body.clone(),
            field: "date_range",
        }
    );
    assert_eq!(
        error.to_string(),
        format!("the VSOP87 source specification for {body} has a blank `date_range` field")
    );
}

#[test]
fn source_specification_validation_rejects_canonical_metadata_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.frame = "J2000 equatorial";

    let error = spec
        .validate()
        .expect_err("canonical source-specification drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "frame",
            expected: "J2000 ecliptic/equinox",
            found: "J2000 equatorial",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `frame` = `J2000 equatorial`, but expected `J2000 ecliptic/equinox`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_variant_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.variant = "VSOP87C";

    let error = spec
        .validate()
        .expect_err("variant drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "variant",
            expected: "VSOP87B",
            found: "VSOP87C",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `variant` = `VSOP87C`, but expected `VSOP87B`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_coordinate_family_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.coordinate_family = "heliocentric rectangular variables";

    let error = spec
        .validate()
        .expect_err("coordinate-family drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "coordinate_family",
            expected: "heliocentric spherical variables",
            found: "heliocentric rectangular variables",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `coordinate_family` = `heliocentric rectangular variables`, but expected `heliocentric spherical variables`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_units_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.units = "radians and astronomical units";

    let error = spec
        .validate()
        .expect_err("units drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "units",
            expected: "degrees and astronomical units",
            found: "radians and astronomical units",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `units` = `radians and astronomical units`, but expected `degrees and astronomical units`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_reduction_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.reduction = "geocentric experimental reduction";

    let error = spec
        .validate()
        .expect_err("reduction drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "reduction",
            expected: "geocentric solar reduction from Earth coefficients",
            found: "geocentric experimental reduction",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `reduction` = `geocentric experimental reduction`, but expected `geocentric solar reduction from Earth coefficients`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_transform_note_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.transform_note = "J2000 equatorial inputs; equatorial coordinates are derived with a mean-obliquity transform";

    let error = spec
        .validate()
        .expect_err("transform note drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "transform_note",
            expected:
                "J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform",
            found:
                "J2000 equatorial inputs; equatorial coordinates are derived with a mean-obliquity transform",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `transform_note` = `J2000 equatorial inputs; equatorial coordinates are derived with a mean-obliquity transform`, but expected `J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_truncation_policy_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.truncation_policy = "vendored full source file";

    let error = spec
        .validate()
        .expect_err("truncation policy drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "truncation_policy",
            expected: "generated binary coefficient table derived from vendored full source file",
            found: "vendored full source file",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `truncation_policy` = `vendored full source file`, but expected `generated binary coefficient table derived from vendored full source file`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_date_range_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.date_range = "full public source file; J2000 reference sample";

    let error = spec
        .validate()
        .expect_err("date range drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "date_range",
            expected: "full public source file; J2000 canonical reference sample",
            found: "full public source file; J2000 reference sample",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `date_range` = `full public source file; J2000 reference sample`, but expected `full public source file; J2000 canonical reference sample`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_unknown_public_source_file() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.source_file = "VSOP87B.synthetic";

    let error = spec
        .validate()
        .expect_err("unknown public source files should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::UnknownSourceFile {
            body: body.clone(),
            source_file: "VSOP87B.synthetic",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} references unknown public source file `VSOP87B.synthetic`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_body_source_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.source_file = "VSOP87B.mer";

    let error = spec
        .validate()
        .expect_err("source-file drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "source_file",
            expected: "VSOP87B.ear",
            found: "VSOP87B.mer",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `source_file` = `VSOP87B.mer`, but expected `VSOP87B.ear`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_unknown_body() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    spec.body = CelestialBody::Pluto;

    let error = spec
        .validate()
        .expect_err("unknown source-backed bodies should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::UnknownBody {
            body: CelestialBody::Pluto,
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 source specification for Pluto is no longer backed by the current source catalog"
    );
}

#[test]
fn source_specification_catalog_rejects_duplicate_public_source_files() {
    let mut specs = source_specifications();
    let duplicated_source_file = specs[0].source_file;
    specs[1].source_file = duplicated_source_file;

    let error = validate_source_specifications(&specs)
        .expect_err("duplicate public source files should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::DuplicateSourceFile {
            source_file: duplicated_source_file,
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification catalog lists public source file `{duplicated_source_file}` more than once"
        )
    );
}

#[test]
fn source_specification_catalog_rejects_whitespace_padded_duplicate_public_source_files() {
    let mut specs = source_specifications();
    let duplicated_source_file = specs[0].source_file;
    specs[1].source_file = "  VSOP87B.ear  ";

    let error = validate_source_specifications(&specs)
        .expect_err("whitespace-padded public source files should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::DuplicateSourceFile {
            source_file: duplicated_source_file,
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification catalog lists public source file `{duplicated_source_file}` more than once"
        )
    );
}

#[test]
fn source_audit_manifest_tracks_all_vendored_inputs() {
    let audits = source_audits();
    let summary = source_audit_summary();

    assert_eq!(audits.len(), 8);
    assert!(audits.iter().all(|audit| audit.validate().is_ok()));
    assert_eq!(audits[0].summary_line(), audits[0].to_string());
    assert_eq!(summary.source_count, 8);
    assert_eq!(summary.source_bodies, source_backed_body_order());
    assert_eq!(
        summary.source_files,
        source_specifications()
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.vendored_full_file_count, 8);
    assert_eq!(summary.fingerprint_count, 8);
    assert!(summary.total_term_count > 0);
    assert!(summary.max_byte_length > 0);
    assert!(summary.max_line_count > 0);

    let mut fingerprints = audits
        .iter()
        .map(|audit| audit.fingerprint)
        .collect::<Vec<_>>();
    fingerprints.sort_unstable();
    fingerprints.dedup();
    assert_eq!(fingerprints.len(), audits.len());

    let earth = audits
        .iter()
        .find(|audit| audit.body == CelestialBody::Sun)
        .expect("Sun audit should exist");
    assert_eq!(earth.source_file, "VSOP87B.ear");
    assert_eq!(earth.term_count, 2_564);
}

#[test]
fn source_audit_validation_rejects_drifted_fields() {
    let mut audit = source_audits()[0].clone();
    audit.term_count += 1;

    let error = audit
        .validate()
        .expect_err("drifted source audit records should fail validation");
    assert_eq!(
        error.to_string(),
        "source audit record #1 for Sun and source file `VSOP87B.ear` has a stale `term_count` field"
    );
    assert_eq!(
        validate_source_audits(&[audit]),
        Err(Vsop87SourceAuditValidationError::FieldOutOfSync {
            position: 1,
            body: CelestialBody::Sun,
            source_file: "VSOP87B.ear",
            field: "term_count",
        })
    );
}

#[test]
fn source_audit_report_matches_the_backend_formatter() {
    let summary = source_audit_summary();
    assert_eq!(
        source_audit_summary_for_report(),
        "VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"
    );
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    let rendered = summary.summary_line();
    assert_eq!(summary.validated_summary_line(), Ok(rendered));
    assert_eq!(
        source_audit_summary_for_report(),
        format_source_audit_summary(&summary)
    );
}

#[test]
fn source_audit_summary_validate_rejects_drifted_fields() {
    let mut summary = source_audit_summary();
    summary.fingerprint_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted source audit summaries should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source audit summary field `fingerprint_count` is out of sync with the current manifest"
    );
    assert_eq!(
        summary.validated_summary_line(),
        Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
            field: "fingerprint_count"
        })
    );
    assert_eq!(
        format_validated_source_audit_summary_for_report(&summary),
        "VSOP87 source audit: unavailable (the VSOP87 source audit summary field `fingerprint_count` is out of sync with the current manifest)"
    );
}

#[test]
fn generated_binary_audit_manifest_tracks_all_checked_in_blobs() {
    let audits = generated_binary_audits();
    let summary = generated_binary_audit_summary();

    assert_eq!(audits.len(), 8);
    assert_eq!(summary.blob_count, 8);
    assert_eq!(summary.source_file_count, 8);
    assert_eq!(summary.fingerprint_count, 8);
    assert_eq!(summary.source_bodies, source_backed_body_order());
    assert_eq!(
        summary.source_files,
        source_specifications()
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>()
    );
    assert!(summary.total_byte_length > 0);
    assert!(summary.max_byte_length > 0);
    assert_eq!(
        audits
            .iter()
            .map(|audit| audit.source_file)
            .collect::<Vec<_>>(),
        summary.source_files
    );
    for audit in audits.iter() {
        assert_eq!(audit.validate(), Ok(()));
        assert_eq!(
            validate_generated_binary_audits(std::slice::from_ref(audit)),
            Ok(())
        );
    }

    let mut fingerprints = audits
        .iter()
        .map(|audit| audit.fingerprint)
        .collect::<Vec<_>>();
    fingerprints.sort_unstable();
    fingerprints.dedup();
    assert_eq!(fingerprints.len(), audits.len());

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    let rendered = summary.summary_line();
    assert_eq!(summary.validated_summary_line(), Ok(rendered));
    assert_eq!(
        generated_binary_audit_summary_for_report(),
        format_generated_binary_audit_summary(&summary)
    );
    assert!(generated_binary_audit_summary_for_report().contains(
        "VSOP87 generated binary audit: 8 checked-in blobs across 8 source files (bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep)"
    ));
}

#[test]
fn generated_binary_audit_summary_validate_rejects_drifted_fields() {
    let mut summary = generated_binary_audit_summary();
    summary.source_file_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted generated blob summaries should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 generated binary audit summary field `source_file_count` is out of sync with the current manifest"
    );
    assert_eq!(
        summary.validated_summary_line(),
        Err(
            Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                field: "source_file_count"
            }
        )
    );
    assert_eq!(
        format_validated_generated_binary_audit_summary_for_report(&summary),
        "VSOP87 generated binary audit: unavailable (the VSOP87 generated binary audit summary field `source_file_count` is out of sync with the current manifest)"
    );
}

#[test]
fn generated_binary_audit_validation_rejects_body_source_mismatches() {
    let mut audit = generated_binary_audits()[0].clone();
    audit.body = CelestialBody::Mercury;

    let error = audit
        .validate()
        .expect_err("mismatched generated blob audits should fail validation");
    assert_eq!(
        error.to_string(),
        "generated binary audit record #1 uses source file `VSOP87B.ear`, which belongs to Sun rather than Mercury"
    );
    assert_eq!(
        validate_generated_binary_audits(&[audit]),
        Err(
            Vsop87GeneratedBlobAuditValidationError::BodySourceMismatch {
                position: 1,
                body: CelestialBody::Mercury,
                source_file: "VSOP87B.ear",
                expected_body: CelestialBody::Sun,
            }
        )
    );
}

#[test]
fn generated_binary_audit_builder_rejects_missing_checked_in_blob() {
    let error = build_generated_binary_audits_with_lookup(|source_file| {
        if source_file == "VSOP87B.ear" {
            None
        } else {
            Some(&[])
        }
    })
    .expect_err("missing generated blobs should fail manifest construction");

    assert_eq!(
        error,
        Vsop87GeneratedBlobAuditValidationError::MissingGeneratedBlob {
            position: 1,
            body: CelestialBody::Sun,
            source_file: "VSOP87B.ear",
        }
    );
    assert_eq!(
        error.to_string(),
        "generated binary audit record #1 is missing the checked-in blob for Sun at source file `VSOP87B.ear`"
    );
}

#[test]
fn source_documentation_summary_tracks_catalog_counts() {
    let summary = source_documentation_summary();

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.summary_line()
    );
    assert_eq!(summary.source_specification_count, 8);
    assert_eq!(summary.source_backed_profile_count, 8);
    assert_eq!(
        summary.source_backed_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(
        summary.source_files,
        vec![
            "VSOP87B.ear",
            "VSOP87B.mer",
            "VSOP87B.ven",
            "VSOP87B.mar",
            "VSOP87B.jup",
            "VSOP87B.sat",
            "VSOP87B.ura",
            "VSOP87B.nep",
        ]
    );
    assert_eq!(
        summary.generated_binary_bodies,
        summary.source_backed_bodies
    );
    assert_eq!(
        summary.generated_binary_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(
        summary.generated_binary_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert!(summary.vendored_full_file_bodies.is_empty());
    assert!(summary.truncated_bodies.is_empty());
    assert_eq!(summary.generated_binary_profile_count, 8);
    assert_eq!(summary.vendored_full_file_profile_count, 0);
    assert_eq!(summary.truncated_profile_count, 0);
    assert_eq!(summary.fallback_profile_count, 1);
    assert_eq!(summary.fallback_bodies, vec![CelestialBody::Pluto]);
    assert_eq!(
        summary.date_ranges,
        vec!["full public source file; J2000 canonical reference sample"]
    );
}

#[test]
fn source_specification_summary_is_typed_and_reusable() {
    let specs = source_specifications();
    let first = &specs[0];
    let expected_joined = specs
        .iter()
        .map(format_source_specification)
        .collect::<Vec<_>>()
        .join(", ");

    assert_eq!(first.summary_line(), first.to_string());
    assert_eq!(
        first.validated_summary_line().unwrap(),
        first.summary_line()
    );
    assert_eq!(format_source_specification(first), first.summary_line());
    assert!(first.summary_line().contains("body=Sun"));
    assert!(first.summary_line().contains("file=VSOP87B.ear"));
    assert!(first.summary_line().contains("variant=VSOP87B"));
    assert!(first
        .summary_line()
        .contains("date range=full public source file; J2000 canonical reference sample"));
    assert_eq!(format_source_specifications(&specs), expected_joined);
    assert_eq!(source_specifications_for_report(), expected_joined);
    assert!(source_specifications_for_report().contains("body=Neptune"));
}

#[test]
fn source_specification_summary_rejects_drifted_metadata() {
    let spec = Vsop87SourceSpecification {
        body: CelestialBody::Sun,
        source_file: "VSOP87B.synthetic",
        variant: "VSOP87B",
        coordinate_family: "heliocentric spherical variables",
        frame: "J2000 ecliptic/equinox",
        units: "degrees and astronomical units",
        reduction: "geocentric solar reduction from Earth coefficients",
        transform_note:
            "J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform",
        truncation_policy: "generated binary coefficient table derived from vendored full source file",
        date_range: "full public source file; J2000 canonical reference sample",
    };

    let error = spec
        .validated_summary_line()
        .expect_err("unknown source files should be rejected");
    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::UnknownSourceFile {
            body: CelestialBody::Sun,
            source_file: "VSOP87B.synthetic",
        }
    );
    assert!(format_source_specification(&spec).starts_with(
        "VSOP87 source specification unavailable (the VSOP87 source specification for Sun references unknown public source file `VSOP87B.synthetic`)"
    ));
}

#[test]
fn source_documentation_health_summary_confirms_catalog_partitioning() {
    let documentation_summary = source_documentation_summary();
    let summary = source_documentation_health_summary();

    assert!(summary.consistent);
    assert!(summary.documentation_consistent);
    assert!(summary.issues.is_empty());
    assert_eq!(summary.source_specification_count, 8);
    assert_eq!(summary.source_file_count, 8);
    assert_eq!(
        summary.source_files,
        vec![
            "VSOP87B.ear",
            "VSOP87B.mer",
            "VSOP87B.ven",
            "VSOP87B.mar",
            "VSOP87B.jup",
            "VSOP87B.sat",
            "VSOP87B.ura",
            "VSOP87B.nep",
        ]
    );
    assert_eq!(summary.source_backed_profile_count, 8);
    assert_eq!(summary.body_profile_count, 9);
    assert_eq!(
        summary.generated_binary_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert!(summary.vendored_full_file_bodies.is_empty());
    assert!(summary.truncated_bodies.is_empty());
    assert_eq!(summary.generated_binary_profile_count, 8);
    assert_eq!(summary.vendored_full_file_profile_count, 0);
    assert_eq!(summary.truncated_profile_count, 0);
    assert_eq!(summary.fallback_profile_count, 1);
    assert!(summary.validate().is_ok());
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        source_documentation_health_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        summary.source_backed_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(
        summary.source_backed_partition_bodies,
        summary.source_backed_bodies
    );
    assert_eq!(
        source_documentation_partition_bodies(&documentation_summary),
        documentation_summary.source_backed_bodies
    );
    assert_eq!(summary.fallback_bodies, vec![CelestialBody::Pluto]);
    assert_eq!(
        format_source_documentation_health_summary(&summary),
        "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
    );
    assert_eq!(
        source_documentation_health_summary_for_report(),
        format_source_documentation_health_summary(&summary)
    );
}

#[test]
fn source_documentation_health_summary_lists_issues_when_inconsistent() {
    let summary = Vsop87SourceDocumentationHealthSummary {
        consistent: false,
        documentation_consistent: false,
        issues: vec![
            Vsop87SourceDocumentationHealthIssue::SourceSpecificationFileCountMismatch,
            Vsop87SourceDocumentationHealthIssue::DocumentedFieldMismatch,
        ],
        source_specification_count: 1,
        source_file_count: 2,
        source_files: vec!["VSOP87B.ear"],
        source_backed_profile_count: 1,
        source_backed_bodies: vec![CelestialBody::Sun],
        source_backed_partition_bodies: vec![CelestialBody::Sun],
        generated_binary_bodies: vec![CelestialBody::Sun],
        vendored_full_file_bodies: vec![],
        truncated_bodies: vec![],
        body_profile_count: 2,
        generated_binary_profile_count: 1,
        vendored_full_file_profile_count: 0,
        truncated_profile_count: 0,
        fallback_profile_count: 1,
        fallback_bodies: vec![CelestialBody::Pluto],
    };

    assert_eq!(
        format_source_documentation_health_summary(&summary),
        "VSOP87 source documentation health: needs attention (1 source specs, 2 source files, 1 source-backed profiles, 2 body profiles; 1 generated binary profiles (Sun), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear; source-backed order: Sun; source-backed partition order: Sun; fallback order: Pluto; documented fields: needs attention); issues: source specification/file count mismatch, documented field mismatch"
    );
    let error = summary
        .validate()
        .expect_err("inconsistent summary should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert_eq!(error.to_string(), summary.summary_line());
    assert_eq!(
        format_validated_source_documentation_health_summary_for_report(&summary),
        "VSOP87 source documentation health: unavailable (VSOP87 source documentation health: needs attention (1 source specs, 2 source files, 1 source-backed profiles, 2 body profiles; 1 generated binary profiles (Sun), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 approximate fallback profiles (Pluto); source files: VSOP87B.ear; source-backed order: Sun; source-backed partition order: Sun; fallback order: Pluto; documented fields: needs attention); issues: source specification/file count mismatch, documented field mismatch)"
    );
}

#[test]
fn source_documentation_health_summary_rejects_partition_order_drift() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.source_backed_partition_bodies.reverse();

    let error = summary
        .validate()
        .expect_err("partition order drift should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
}

#[test]
fn source_documentation_health_summary_rejects_profile_count_drift() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.generated_binary_profile_count += 1;

    let error = summary
        .validate()
        .expect_err("profile count drift should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
}

#[test]
fn source_documentation_health_summary_rejects_source_backed_body_duplicates() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.generated_binary_bodies[1] = CelestialBody::Sun;
    summary.source_backed_partition_bodies = summary
        .generated_binary_bodies
        .iter()
        .chain(summary.vendored_full_file_bodies.iter())
        .chain(summary.truncated_bodies.iter())
        .cloned()
        .collect();
    summary.source_backed_bodies = summary.source_backed_partition_bodies.clone();
    summary.consistent = false;
    summary.issues = vec![Vsop87SourceDocumentationHealthIssue::SourceBackedBodyDuplicate];

    let error = summary
        .validate()
        .expect_err("duplicate source-backed bodies should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert!(error.to_string().contains("source-backed body duplicate"));
}

#[test]
fn source_documentation_health_summary_rejects_source_backed_fallback_overlap() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.fallback_bodies = vec![CelestialBody::Sun];
    summary.consistent = false;
    summary.issues = vec![Vsop87SourceDocumentationHealthIssue::SourceBackedFallbackBodyOverlap];

    let error = summary
        .validate()
        .expect_err("source-backed/fallback overlap should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert!(error
        .to_string()
        .contains("source-backed/fallback body overlap"));
}

#[test]
fn source_documentation_health_summary_rejects_fallback_body_duplicates() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.fallback_bodies = vec![CelestialBody::Pluto, CelestialBody::Pluto];
    summary.fallback_profile_count = summary.fallback_bodies.len();
    summary.body_profile_count =
        summary.source_backed_profile_count + summary.fallback_profile_count;
    summary.consistent = false;
    summary.issues = vec![Vsop87SourceDocumentationHealthIssue::FallbackBodyDuplicate];

    let error = summary
        .validate()
        .expect_err("duplicate fallback bodies should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert!(error.to_string().contains("fallback body duplicate"));
}

#[test]
fn source_documentation_health_issue_labels_are_stable() {
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::SourceSpecificationFileCountMismatch.to_string(),
        "source specification/file count mismatch"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::SourceBackedBodyDuplicate.to_string(),
        "source-backed body duplicate"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::FallbackBodyDuplicate.to_string(),
        "fallback body duplicate"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::SourceBackedFallbackBodyOverlap.to_string(),
        "source-backed/fallback body overlap"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::DocumentedFieldMismatch.to_string(),
        "documented field mismatch"
    );
}

#[test]
fn source_documentation_health_issues_detect_partition_order_drift() {
    let mut summary = source_documentation_summary();
    summary.source_backed_bodies.reverse();
    let source_specs = source_specifications();

    let issues = source_documentation_health_issues(
        &summary,
        &source_specs,
        body_catalog_entries().len(),
        summary.source_files.len(),
    );

    assert!(issues.contains(&Vsop87SourceDocumentationHealthIssue::SourceBackedBodyOrderMismatch));
}

#[test]
fn request_policy_summary_tracks_the_public_backend_posture() {
    let policy = vsop87_request_policy();

    assert_eq!(policy.to_string(), policy.summary_line());
    assert_eq!(
        policy.summary_line(),
        "frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
    );
    assert_eq!(
        policy.supported_frames,
        &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial]
    );
    assert_eq!(
        policy.supported_time_scales,
        &[TimeScale::Tt, TimeScale::Tdb]
    );
    assert_eq!(policy.supported_zodiac_modes, &[ZodiacMode::Tropical]);
    assert_eq!(policy.supported_apparentness, &[Apparentness::Mean]);
    assert!(!policy.supports_topocentric_observer);
    assert!(policy.validate().is_ok());
    assert_eq!(
        vsop87_request_policy_summary_for_report(),
        policy.summary_line()
    );
}

#[test]
fn request_policy_summary_validation_rejects_stale_posture() {
    let mut policy = vsop87_request_policy();
    policy.supports_topocentric_observer = true;

    let error = policy
        .validate()
        .expect_err("drifted VSOP87 request-policy summaries should fail validation");

    assert_eq!(
        error,
        Vsop87RequestPolicyValidationError::FieldOutOfSync {
            field: "supports_topocentric_observer"
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 request-policy summary field `supports_topocentric_observer` is out of sync with the current posture"
    );
}

#[test]
fn source_kind_display_labels_match_the_release_facing_labels() {
    let cases = [
        (
            Vsop87BodySourceKind::TruncatedVsop87b,
            "truncated VSOP87B slice",
        ),
        (
            Vsop87BodySourceKind::VendoredVsop87b,
            "vendored full-file VSOP87B",
        ),
        (
            Vsop87BodySourceKind::GeneratedBinaryVsop87b,
            "generated binary VSOP87B",
        ),
        (
            Vsop87BodySourceKind::MeanOrbitalElements,
            "mean orbital elements fallback",
        ),
    ];

    for (kind, expected) in cases {
        assert_eq!(kind.label(), expected);
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn source_documentation_report_matches_the_backend_formatter() {
    let summary = source_documentation_summary();
    let rendered = source_documentation_summary_for_report();
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line().unwrap(), rendered);
    assert!(source_documentation_health_summary().validate().is_ok());
    assert_eq!(rendered, format_source_documentation_summary(&summary));
    assert_eq!(summary.summary_line(), rendered);
    assert_eq!(summary.to_string(), rendered);
    assert!(rendered.contains("source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
    assert!(rendered.contains("source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"));
}

#[test]
fn source_documentation_report_marks_summary_drift_as_unavailable() {
    let mut summary = source_documentation_summary();
    summary.source_specification_count += 1;

    assert_eq!(
        format_validated_source_documentation_summary_for_report(&summary),
        "VSOP87 source documentation: unavailable (the VSOP87 source documentation summary field `source_specification_count` is out of sync with the current source catalog)"
    );
}

#[test]
fn source_documentation_summary_validation_rejects_source_file_drift() {
    let mut summary = source_documentation_summary();
    summary.source_specification_count += 1;

    let error = summary
        .validate()
        .expect_err("source specification count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source documentation summary field `source_specification_count` is out of sync with the current source catalog"
    );
}

#[test]
fn source_body_evidence_summary_matches_the_canonical_body_evidence() {
    let evidence = canonical_epoch_body_evidence().expect("evidence should exist");
    let summary = source_body_evidence_summary().expect("summary should exist");

    assert_eq!(summary.sample_count, evidence.len());
    assert_eq!(
        summary.sample_bodies,
        evidence
            .iter()
            .map(|row| row.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.within_interim_limits_count, evidence.len());
    assert_eq!(summary.vendored_full_file_count, 0);
    assert_eq!(summary.generated_binary_count, evidence.len());
    assert_eq!(summary.truncated_count, 0);
    assert_eq!(summary.outside_interim_limit_count, 0);
    assert!(summary.outside_interim_limit_bodies.is_empty());
    assert!(evidence.iter().all(|row| row.within_interim_limits));
    assert!(format_source_body_evidence_summary(&summary).contains(
        "source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
    ));
}

#[test]
fn source_body_evidence_report_matches_the_backend_formatter() {
    let summary = source_body_evidence_summary().expect("summary should exist");
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        source_body_evidence_summary_for_report(),
        format_source_body_evidence_summary(&summary)
    );
    assert_eq!(
        summary.summary_line(),
        source_body_evidence_summary_for_report()
    );
    assert_eq!(
        summary.to_string(),
        source_body_evidence_summary_for_report()
    );
}

#[test]
fn source_body_evidence_validated_summary_line_rejects_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validated_summary_line()
        .expect_err("drifted body evidence summary should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_report_matches_the_backend_formatter() {
    let summary = source_body_class_evidence_summary().expect("summary should exist");
    assert_eq!(summary.len(), 2);
    assert_eq!(summary[0].class, Vsop87SourceBodyClass::Luminary);
    assert_eq!(summary[0].sample_count, 1);
    assert_eq!(summary[0].sample_bodies, vec![CelestialBody::Sun]);
    assert_eq!(summary[0].validate(), Ok(()));
    assert_eq!(summary[1].class, Vsop87SourceBodyClass::MajorPlanet);
    assert_eq!(summary[1].sample_count, 7);
    assert_eq!(
        summary[1].sample_bodies,
        vec![
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(summary[1].validate(), Ok(()));
    let rendered = source_body_class_evidence_summary_for_report();
    assert_eq!(
        rendered,
        format_source_body_class_evidence_summary(&summary)
    );
    assert_eq!(summary[0].summary_line(), summary[0].to_string());
    assert_eq!(summary[1].summary_line(), summary[1].to_string());
    assert_eq!(
        summary[0].validated_summary_line(),
        Ok(summary[0].summary_line())
    );
    assert_eq!(
        summary[1].validated_summary_line(),
        Ok(summary[1].summary_line())
    );
    assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("median Δlat="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("median Δdist="));
    assert!(rendered.contains("p95 Δdist="));
    assert!(rendered.contains(
        "Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
    ));
}

#[test]
fn source_body_class_evidence_report_marks_drift_as_unavailable() {
    let mut summary = source_body_class_evidence_summary().expect("summary should exist");
    summary[0].sample_count += 1;

    assert_eq!(
        format_validated_source_body_class_evidence_summary_for_report(&summary),
        "VSOP87 source-backed body-class envelopes: unavailable (the VSOP87 source-backed body-class evidence summary field `sample_count` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_count_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_duplicate_sample_bodies() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    let duplicated_body = summary.sample_bodies[0].clone();
    summary.sample_bodies.push(duplicated_body);
    summary.sample_count += 1;
    summary.within_interim_limits_count += 1;
    summary.generated_binary_count += 1;

    let error = summary
        .validate()
        .expect_err("duplicate sample bodies should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_sample_order_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.sample_bodies.reverse();

    let error = summary
        .validate()
        .expect_err("sample bodies must preserve canonical order");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_within_limit_count_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.within_interim_limits_count += 1;

    let error = summary
        .validate()
        .expect_err("within-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `within_interim_limits_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_outside_limit_count_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.outside_interim_limit_count += 1;
    summary.outside_interim_limit_bodies = vec![CelestialBody::Moon];

    let error = summary
        .validate()
        .expect_err("outside-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `outside_interim_limit_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_empty_summary() {
    let summary = Vsop87SourceBodyEvidenceSummary {
        sample_count: 0,
        sample_bodies: Vec::new(),
        within_interim_limits_count: 0,
        vendored_full_file_count: 0,
        generated_binary_count: 0,
        truncated_count: 0,
        outside_interim_limit_count: 0,
        outside_interim_limit_bodies: Vec::new(),
    };

    let error = summary
        .validate()
        .expect_err("empty evidence summaries should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_count_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .next()
        .expect("at least one class summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_duplicate_sample_bodies() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.sample_bodies[1] = summary.sample_bodies[0].clone();

    let error = summary
        .validate()
        .expect_err("duplicate sample bodies should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_sample_order_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.sample_bodies.reverse();

    let error = summary
        .validate()
        .expect_err("sample bodies must preserve canonical order");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_within_limit_count_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.within_interim_limits_count += 1;

    let error = summary
        .validate()
        .expect_err("within-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `within_interim_limits_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_outside_limit_count_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.outside_interim_limit_count += 1;
    summary.outside_interim_limit_bodies = vec![CelestialBody::Moon];

    let error = summary
        .validate()
        .expect_err("outside-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `outside_interim_limit_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_blank_peak_source_file() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_longitude_delta_source_file = "";

    let error = summary
        .validate()
        .expect_err("blank peak source file should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `max_longitude_delta_source_file` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_source_kind_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_longitude_delta_source_kind = Vsop87BodySourceKind::VendoredVsop87b;

    let error = summary
        .validate()
        .expect_err("source-kind drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `max_longitude_delta_source_kind` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_source_file_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_latitude_delta_source_file = "VSOP87B.ear";

    let error = summary
        .validate()
        .expect_err("source-file drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `max_latitude_delta_source_file` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_non_finite_metric() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.mean_distance_delta_au = f64::NAN;

    let error = summary
        .validate()
        .expect_err("non-finite metrics should fail validation");
    assert_eq!(
        error,
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
            field: "mean_distance_delta_au",
        }
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_metric_order_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.median_longitude_delta_deg = summary.percentile_longitude_delta_deg + 1e-9;

    let error = summary
        .validate()
        .expect_err("metric ordering should fail validation");
    assert_eq!(
        error,
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
            field: "median_longitude_delta_deg",
        }
    );
}

#[test]
fn canonical_equatorial_body_class_evidence_report_matches_the_backend_formatter() {
    let summary =
        canonical_epoch_equatorial_body_class_evidence_summary().expect("summary should exist");
    assert_eq!(summary.len(), 2);
    assert_eq!(summary[0].class, Vsop87SourceBodyClass::Luminary);
    assert_eq!(summary[0].sample_count, 1);
    assert_eq!(summary[0].sample_bodies, vec![CelestialBody::Sun]);
    assert_eq!(summary[0].validate(), Ok(()));
    assert_eq!(summary[1].class, Vsop87SourceBodyClass::MajorPlanet);
    assert_eq!(summary[1].sample_count, 7);
    assert_eq!(
        summary[1].sample_bodies,
        vec![
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(summary[1].validate(), Ok(()));
    let rendered = canonical_epoch_equatorial_body_class_evidence_summary_for_report();
    assert_eq!(
        rendered,
        format_canonical_equatorial_body_class_evidence_summary(&summary)
    );
    assert_eq!(summary[0].summary_line(), summary[0].to_string());
    assert_eq!(summary[1].summary_line(), summary[1].to_string());
    assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
    assert!(rendered.contains("median Δra="));
    assert!(rendered.contains("p95 Δra="));
    assert!(rendered.contains("median Δdec="));
    assert!(rendered.contains("p95 Δdec="));
    assert!(rendered.contains("median Δdist="));
    assert!(rendered.contains("p95 Δdist="));
    assert!(rendered.contains(
        "Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
    ));
}

#[test]
fn canonical_equatorial_body_class_evidence_summary_validation_rejects_count_drift() {
    let mut summary = canonical_epoch_equatorial_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .next()
        .expect("at least one class summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial body-class evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn canonical_equatorial_body_class_evidence_summary_validation_rejects_blank_peak_source_file() {
    let mut summary = canonical_epoch_equatorial_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_distance_delta_source_file = "";

    let error = summary
        .validate()
        .expect_err("blank peak source file should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial body-class evidence summary field `max_distance_delta_source_file` is blank"
    );
}

#[test]
fn canonical_evidence_report_matches_the_backend_formatter() {
    let summary = canonical_epoch_evidence_summary().expect("summary should exist");
    let rendered = canonical_epoch_evidence_summary_for_report();
    assert_eq!(rendered, format_canonical_epoch_evidence_summary(&summary));
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(rendered, summary.summary_line());
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("p95 Δdist="));
}

#[test]
fn canonical_body_evidence_row_validation_and_summary_line_are_stable() {
    let row = canonical_epoch_body_evidence()
        .expect("evidence should exist")
        .into_iter()
        .next()
        .expect("at least one evidence row should exist");

    assert_eq!(row.summary_line(), row.to_string());
    assert!(row.summary_line().contains("kind="));
    assert!(row.summary_line().contains("source=VSOP87B.ear"));
    assert!(row.summary_line().contains("provenance="));
    assert!(row.summary_line().contains("status within interim limits"));
    assert_eq!(row.validate(), Ok(()));
}

#[test]
fn canonical_body_evidence_validation_rejects_source_file_drift() {
    let mut row = canonical_epoch_body_evidence()
        .expect("evidence should exist")
        .into_iter()
        .next()
        .expect("at least one evidence row should exist");
    row.source_file = "VSOP87B.synthetic";
    let body = row.body.clone();

    let error = row
        .validate()
        .expect_err("source file drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalBodyEvidenceValidationError::SourceFileMismatch {
            body,
            expected: "VSOP87B.ear",
            found: "VSOP87B.synthetic",
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed body evidence row for Sun expects source file `VSOP87B.ear` but found `VSOP87B.synthetic`"
    );
}

#[test]
fn canonical_body_evidence_validation_rejects_interim_limit_status_drift() {
    let mut row = canonical_epoch_body_evidence()
        .expect("evidence should exist")
        .into_iter()
        .next()
        .expect("at least one evidence row should exist");
    row.within_interim_limits = !row.within_interim_limits;
    let body = row.body.clone();

    let error = row
        .validate()
        .expect_err("interim-limit status drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalBodyEvidenceValidationError::InterimLimitStatusMismatch { body }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed body evidence row for Sun has a mismatched interim-limit status"
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_duplicate_bodies() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    let duplicated_body = summary.sample_bodies[0].clone();
    summary.sample_bodies[1] = duplicated_body.clone();

    let error = summary
        .validate()
        .expect_err("duplicate bodies should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::DuplicateBody {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            body: duplicated_body,
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed evidence summary lists body `Sun` more than once"
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_peak_source_file_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.max_longitude_delta_source_file = "VSOP87B.synthetic";

    let error = summary
        .validate()
        .expect_err("peak source file drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "max_longitude_delta_source_file",
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed evidence summary field `max_longitude_delta_source_file` is out of sync with the current canonical evidence"
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_non_finite_metric() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.mean_distance_delta_au = f64::INFINITY;

    let error = summary
        .validate()
        .expect_err("non-finite metrics should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "mean_distance_delta_au",
        }
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_metric_order_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.median_longitude_delta_deg = summary.percentile_longitude_delta_deg + 1e-9;

    let error = summary
        .validate()
        .expect_err("metric ordering should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "median_longitude_delta_deg",
        }
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_body_evidence_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.mean_distance_delta_au += 1e-12;

    let error = summary
        .validate()
        .expect_err("body-evidence drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "mean_distance_delta_au",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_non_finite_metric() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.rms_distance_delta_au = f64::NAN;

    let error = summary
        .validate()
        .expect_err("non-finite metrics should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "rms_distance_delta_au",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_metric_order_drift() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.percentile_declination_delta_deg = summary.max_declination_delta_deg + 1e-9;

    let error = summary
        .validate()
        .expect_err("metric ordering should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "percentile_declination_delta_deg",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_body_evidence_drift() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.mean_right_ascension_delta_deg += 1e-12;

    let error = summary
        .validate()
        .expect_err("body-evidence drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "mean_right_ascension_delta_deg",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_peak_source_kind_drift() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.max_right_ascension_delta_source_kind = Vsop87BodySourceKind::MeanOrbitalElements;

    let error = summary
        .validate()
        .expect_err("peak source kind drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "max_right_ascension_delta_source_kind",
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial companion evidence summary field `max_right_ascension_delta_source_kind` is out of sync with the current canonical evidence"
    );
}

#[test]
fn canonical_j2000_batch_parity_report_matches_the_backend_formatter() {
    let summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    let rendered = canonical_j2000_batch_parity_summary_for_report();
    assert_eq!(rendered, summary.summary_line());
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.sample_count, canonical_epoch_samples().len());
    assert_eq!(
        summary.sample_bodies,
        canonical_epoch_samples()
            .iter()
            .map(|sample| sample.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.reference_epoch.julian_day.days(), J2000);
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tt);
    assert!(rendered.contains("quality counts: Exact="));
    assert!(rendered.contains("batch/single parity preserved"));
}

#[test]
fn canonical_j2000_batch_parity_requests_preserve_the_source_backed_batch_slice() {
    let requests = canonical_j2000_batch_parity_requests();

    assert_eq!(requests.len(), canonical_epoch_samples().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        source_backed_body_order()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tt
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn canonical_epoch_requests_remain_a_compatibility_alias() {
    assert_eq!(
        canonical_epoch_requests(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_epoch_request_corpus_remains_a_compatibility_alias() {
    assert_eq!(canonical_epoch_request_corpus(), canonical_epoch_requests());
}

#[test]
fn canonical_j2000_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_j2000_batch_parity_request_corpus(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_j2000_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_j2000_request_corpus(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_epoch_batch_parity_requests_remain_a_compatibility_alias() {
    assert_eq!(
        canonical_epoch_batch_parity_requests(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_epoch_batch_parity_request_corpus_remains_a_compatibility_alias() {
    assert_eq!(
        canonical_epoch_batch_parity_request_corpus(),
        canonical_epoch_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_batch_parity_requests(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_batch_parity_request_corpus(),
        source_backed_body_j2000_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        source_backed_body_j2000_request_corpus(),
        source_backed_body_j2000_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_ecliptic_batch_parity_requests_preserve_the_source_backed_body_order() {
    let requests = source_backed_body_j2000_ecliptic_batch_parity_request_corpus();

    assert_eq!(requests.len(), source_backed_body_order().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        source_backed_body_order()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tt
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn source_backed_body_j2000_ecliptic_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_ecliptic_batch_parity_requests(),
        source_backed_body_j2000_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j2000_ecliptic_batch_parity_requests(),
        source_backed_body_j2000_ecliptic_batch_parity_request_corpus()
    );
}

#[test]
fn source_backed_body_request_corpus_aliases_remain_the_frame_specific_canonical_slices() {
    assert_eq!(
        source_backed_body_j2000_ecliptic_request_corpus(),
        source_backed_body_j2000_ecliptic_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j2000_equatorial_request_corpus(),
        source_backed_body_j2000_equatorial_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j1900_ecliptic_request_corpus(),
        source_backed_body_j1900_ecliptic_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j1900_equatorial_request_corpus(),
        source_backed_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_equatorial_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        canonical_j1900_equatorial_batch_parity_requests(),
        canonical_j1900_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_j1900_equatorial_batch_parity_request_corpus(),
        canonical_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_j1900_batch_parity_request_corpus(),
        canonical_j1900_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_j1900_request_corpus(),
        canonical_j1900_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = canonical_j1900_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J1900
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Equatorial
    }));
}

#[test]
fn supported_body_j2000_equatorial_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j2000_equatorial_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Equatorial
    }));
}

#[test]
fn supported_body_j2000_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        supported_body_j2000_equatorial_batch_parity_request_corpus(),
        supported_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j2000_equatorial_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j2000_equatorial_request_corpus(),
        supported_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_equatorial_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_equatorial_batch_parity_requests(),
        supported_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_equatorial_batch_parity_request_corpus(),
        source_backed_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j2000_ecliptic_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j2000_ecliptic_batch_parity_request_corpus();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn supported_body_j2000_ecliptic_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        supported_body_j2000_ecliptic_batch_parity_requests(),
        supported_body_j2000_ecliptic_batch_parity_request_corpus()
    );
}

#[test]
fn supported_body_j2000_ecliptic_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j2000_ecliptic_request_corpus(),
        supported_body_j2000_ecliptic_batch_parity_request_corpus()
    );
}

#[test]
fn supported_body_request_corpus_remains_the_ecliptic_aliases() {
    assert_eq!(
        supported_body_j2000_request_corpus(),
        supported_body_j2000_ecliptic_request_corpus()
    );
    assert_eq!(
        supported_body_j1900_request_corpus(),
        supported_body_j1900_ecliptic_request_corpus()
    );
}

#[test]
fn supported_body_j1900_ecliptic_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j1900_ecliptic_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J1900
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn source_backed_body_j1900_ecliptic_batch_parity_requests_preserve_the_supported_body_order() {
    assert_eq!(
        source_backed_body_j1900_ecliptic_batch_parity_requests(),
        supported_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j1900_ecliptic_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j1900_ecliptic_batch_parity_request_corpus(),
        source_backed_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j1900_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        source_backed_body_j1900_request_corpus(),
        source_backed_body_j1900_ecliptic_request_corpus()
    );
}

#[test]
fn source_backed_body_j1900_equatorial_batch_parity_requests_preserve_the_supported_body_order() {
    assert_eq!(
        source_backed_body_j1900_equatorial_batch_parity_requests(),
        supported_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j1900_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j1900_equatorial_batch_parity_request_corpus(),
        source_backed_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_ecliptic_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        supported_body_j1900_ecliptic_batch_parity_request_corpus(),
        supported_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_ecliptic_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j1900_ecliptic_request_corpus(),
        supported_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_equatorial_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j1900_equatorial_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J1900
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Equatorial
    }));
}

#[test]
fn supported_body_j1900_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        supported_body_j1900_equatorial_batch_parity_request_corpus(),
        supported_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_equatorial_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j1900_equatorial_request_corpus(),
        supported_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_canonical_batch_parity_summary_matches_the_backend_helpers() {
    let summary = supported_body_canonical_batch_parity_summary()
        .expect("supported-body canonical batch matrix should exist");
    let rendered = supported_body_canonical_batch_parity_summary_for_report();
    let requests = supported_body_canonical_batch_parity_requests();

    assert_eq!(rendered, summary.summary_line());
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(
        summary.supported_body_count,
        Vsop87Backend::supported_bodies().len()
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j2000_equatorial.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j1900_ecliptic.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j1900_equatorial.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        summary.j2000_equatorial.sample_bodies
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        summary.j1900_ecliptic.sample_bodies
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        summary.j1900_equatorial.sample_bodies
    );
    assert_eq!(summary.j2000_ecliptic.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.j2000_equatorial.frame, CoordinateFrame::Equatorial);
    assert_eq!(summary.j1900_ecliptic.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.j1900_equatorial.frame, CoordinateFrame::Equatorial);
    assert_eq!(requests.len(), summary.supported_body_count * 4);
    assert_eq!(
        requests[..summary.supported_body_count]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j2000_ecliptic.sample_bodies
    );
    assert_eq!(
        requests[summary.supported_body_count..summary.supported_body_count * 2]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j2000_equatorial.sample_bodies
    );
    assert_eq!(
        requests[summary.supported_body_count * 2..summary.supported_body_count * 3]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j1900_ecliptic.sample_bodies
    );
    assert_eq!(
        requests[summary.supported_body_count * 3..]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j1900_equatorial.sample_bodies
    );
    assert!(rendered.contains("J2000 ecliptic"));
    assert!(rendered.contains("J2000 equatorial"));
    assert!(rendered.contains("J1900 ecliptic"));
    assert!(rendered.contains("J1900 equatorial"));
}

#[test]
fn supported_body_canonical_batch_parity_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_canonical_batch_parity_request_corpus(),
        supported_body_canonical_batch_parity_requests()
    );
}

#[test]
fn supported_body_canonical_batch_matrix_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_canonical_batch_matrix_request_corpus(),
        supported_body_canonical_batch_parity_request_corpus()
    );
}

#[test]
fn supported_body_canonical_batch_matrix_requests_remain_the_alias() {
    assert_eq!(
        supported_body_canonical_batch_matrix_requests(),
        supported_body_canonical_batch_matrix_request_corpus()
    );
}

#[test]
fn supported_body_canonical_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_canonical_request_corpus(),
        supported_body_canonical_batch_matrix_requests()
    );
}

#[test]
fn supported_body_canonical_batch_parity_report_surfaces_validation_errors() {
    let mut summary = supported_body_canonical_batch_parity_summary()
        .expect("supported-body canonical batch matrix should exist");
    summary.supported_body_count += 1;

    let rendered =
        format_validated_supported_body_canonical_batch_parity_summary_for_report(&summary);

    assert!(rendered.starts_with("VSOP87 supported-body canonical batch matrix: unavailable ("));
    assert!(rendered.contains("supported_body_count"));
}

#[test]
fn canonical_mixed_time_scale_batch_parity_requests_preserve_the_canonical_slice() {
    let requests = canonical_mixed_time_scale_batch_parity_requests();

    assert_eq!(requests.len(), canonical_epoch_samples().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        canonical_epoch_samples()
            .iter()
            .map(|sample| sample.body.clone())
            .collect::<Vec<_>>()
    );
    assert!(requests.iter().enumerate().all(|(index, request)| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale
                == if index % 2 == 0 {
                    TimeScale::Tt
                } else {
                    TimeScale::Tdb
                }
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn canonical_mixed_tt_tdb_batch_parity_report_matches_the_backend_formatter() {
    let summary = canonical_mixed_time_scale_batch_parity_summary()
        .expect("mixed batch summary should exist");
    let rendered = canonical_mixed_time_scale_batch_parity_summary_for_report();
    assert_eq!(rendered, summary.summary_line());
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.sample_count, canonical_epoch_samples().len());
    assert_eq!(
        summary.sample_bodies,
        canonical_epoch_samples()
            .iter()
            .map(|sample| sample.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.reference_epoch.julian_day.days(), J2000);
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tt);
    assert_eq!(summary.tt_request_count, summary.sample_count.div_ceil(2));
    assert_eq!(summary.tdb_request_count, summary.sample_count / 2);
    assert!(rendered.contains("TT/TDB mix"));
    assert!(rendered.contains("TT requests="));
    assert!(rendered.contains("TDB requests="));
}

#[test]
fn canonical_mixed_time_scale_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_mixed_time_scale_batch_parity_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_tt_tdb_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        canonical_mixed_tt_tdb_batch_parity_requests(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_tt_tdb_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_mixed_tt_tdb_batch_parity_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_time_scale_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_mixed_time_scale_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_tt_tdb_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_mixed_tt_tdb_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_tt_tdb_batch_parity_report_surfaces_validation_errors() {
    let mut summary = canonical_mixed_time_scale_batch_parity_summary()
        .expect("mixed batch summary should exist");
    summary.tt_request_count += 1;

    assert_eq!(
        format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report(&summary),
        "VSOP87 canonical mixed TT/TDB batch parity: unavailable (the VSOP87 canonical batch parity summary field `tt_request_count` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn canonical_j2000_batch_parity_report_surfaces_validation_errors() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.sample_count += 1;

    assert_eq!(
        format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
        "VSOP87 canonical J2000 batch parity: unavailable (the VSOP87 canonical batch parity summary field `sample_count` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_body_order_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.sample_bodies.reverse();

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_frame_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Equatorial;

    assert_eq!(
        summary.validate(),
        Err(Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" })
    );
}

#[test]
fn canonical_j2000_batch_parity_report_surfaces_body_order_validation_errors() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.sample_bodies.reverse();

    assert_eq!(
        format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
        "VSOP87 canonical J2000 batch parity: unavailable (the VSOP87 canonical batch parity summary field `sample_bodies` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn canonical_j2000_batch_parity_report_surfaces_frame_validation_errors() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Equatorial;

    assert_eq!(
        format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
        "VSOP87 canonical J2000 batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn canonical_j1900_batch_parity_report_matches_the_backend_formatter() {
    let summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    let rendered = canonical_j1900_batch_parity_summary_for_report();
    assert_eq!(rendered, summary.summary_line());
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.sample_count, summary.sample_bodies.len());
    assert_eq!(
        summary.sample_bodies,
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert_eq!(summary.frame, CoordinateFrame::Equatorial);
    assert_eq!(summary.reference_epoch.julian_day.days(), J1900);
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(
        summary.sample_count,
        summary.exact_count
            + summary.interpolated_count
            + summary.approximate_count
            + summary.unknown_count
    );
    assert!(rendered.contains("JD 2415020.0 (TDB)"));
    assert!(rendered.contains("quality counts: Exact="));
    assert!(rendered.contains("batch/single parity preserved"));
}

#[test]
fn canonical_j1900_batch_parity_report_surfaces_validation_errors() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Ecliptic;

    assert_eq!(
        format_validated_canonical_j1900_batch_parity_summary_for_report(&summary),
        "VSOP87 canonical J1900 batch parity: unavailable (the VSOP87 canonical batch parity summary field `frame` is out of sync with the current canonical evidence)"
    );
}

#[test]
fn canonical_batch_parity_summary_validation_rejects_count_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.sample_count += 1;

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        )
    );
}

#[test]
fn canonical_batch_parity_summary_validation_rejects_quality_count_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    if summary.exact_count > 0 {
        summary.exact_count -= 1;
        summary.unknown_count += 1;
    } else if summary.interpolated_count > 0 {
        summary.interpolated_count -= 1;
        summary.exact_count += 1;
    } else if summary.approximate_count > 0 {
        summary.approximate_count -= 1;
        summary.exact_count += 1;
    } else {
        summary.unknown_count -= 1;
        summary.exact_count += 1;
    }

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "quality_counts"
            }
        )
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_quality_count_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    if summary.exact_count > 0 {
        summary.exact_count -= 1;
        summary.unknown_count += 1;
    } else if summary.interpolated_count > 0 {
        summary.interpolated_count -= 1;
        summary.exact_count += 1;
    } else if summary.approximate_count > 0 {
        summary.approximate_count -= 1;
        summary.exact_count += 1;
    } else {
        summary.unknown_count -= 1;
        summary.exact_count += 1;
    }

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "quality_counts"
            }
        )
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_reference_epoch_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.reference_epoch = Instant::new(
        pleiades_types::JulianDay::from_days(J2000 + 1.0),
        TimeScale::Tt,
    );

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "reference_epoch"
            }
        )
    );
}

#[test]
fn canonical_j1900_batch_parity_summary_validation_rejects_reference_epoch_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    summary.reference_epoch = Instant::new(
        pleiades_types::JulianDay::from_days(J1900 + 1.0),
        TimeScale::Tdb,
    );

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "reference_epoch"
            }
        )
    );
}

#[test]
fn canonical_j1900_batch_parity_summary_validation_rejects_frame_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Ecliptic;

    assert_eq!(
        summary.validate(),
        Err(Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" })
    );
}

#[test]
fn canonical_j1900_batch_parity_summary_validation_rejects_body_order_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    summary.sample_bodies.reverse();

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    );
}

#[test]
fn canonical_evidence_outlier_note_reports_the_current_interim_status() {
    let summary = canonical_epoch_outlier_summary().expect("outlier summary should exist");

    assert_eq!(
        summary.summary_line(),
        "VSOP87 canonical J2000 interim outliers: none"
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        canonical_epoch_outlier_note_for_report(),
        summary.summary_line()
    );
}

#[test]
fn canonical_evidence_outlier_summary_validation_rejects_drift() {
    let mut summary = canonical_epoch_outlier_summary().expect("outlier summary should exist");
    summary.outlier_bodies.push(CelestialBody::Sun);

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalOutlierSummaryValidationError::FieldOutOfSync {
                field: "outlier_bodies"
            }
        )
    );
}

#[test]
fn frame_treatment_summary_has_a_displayable_summary_line() {
    let summary = frame_treatment_summary_details();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
    );
    assert_eq!(frame_treatment_summary(), summary.summary_line());
    assert_eq!(frame_treatment_summary_for_report(), summary.to_string());
    assert!(summary.summary_line().contains("mean-obliquity transform"));
}

#[test]
fn canonical_equatorial_evidence_report_matches_the_backend_formatter() {
    let summary =
        canonical_epoch_equatorial_evidence_summary().expect("equatorial summary should exist");
    let rendered = canonical_epoch_equatorial_evidence_summary_for_report();
    assert_eq!(
        rendered,
        format_canonical_equatorial_evidence_summary(&summary)
    );
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(rendered, summary.summary_line());
    assert!(rendered.contains("p95 Δra="));
    assert!(rendered.contains("p95 Δdec="));
    assert!(rendered.contains("p95 Δdist="));
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_peak_body_drift() {
    let mut summary =
        canonical_epoch_equatorial_evidence_summary().expect("equatorial summary should exist");
    summary.max_distance_delta_body = CelestialBody::Pluto;

    let error = summary
        .validate()
        .expect_err("peak body drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::PeakBodyNotInSamples {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "max_distance_delta_body",
            body: CelestialBody::Pluto,
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial companion evidence summary field `max_distance_delta_body` points at body `Pluto` which is absent from the sample body list"
    );
}

#[test]
fn canonical_evidence_report_lists_the_measured_bodies() {
    let rendered = canonical_epoch_evidence_summary_for_report();
    assert!(
        rendered.contains("bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune")
    );
}

#[test]
fn canonical_equatorial_evidence_report_lists_the_measured_bodies() {
    let rendered = canonical_epoch_equatorial_evidence_summary_for_report();
    assert!(
        rendered.contains("bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune")
    );
}

#[test]
fn source_manifest_pairs_bodies_with_source_files_in_release_order() {
    let manifest = source_manifest();
    let expected_manifest = source_specifications()
        .into_iter()
        .map(|spec| (spec.body, spec.source_file))
        .collect::<Vec<_>>();

    assert_eq!(manifest, expected_manifest);
    assert_eq!(
        supported_source_files(),
        expected_manifest
            .iter()
            .map(|(_, source_file)| *source_file)
            .collect::<Vec<_>>()
    );
    assert_eq!(validate_source_manifest(&manifest), Ok(()));
    for (body, source_file) in &manifest {
        assert!(
            checked_in_generated_vsop87b_table_bytes_for_source_file(source_file).is_some(),
            "supported source file {source_file} should have a checked-in generated blob for {body}"
        );
    }
}

#[test]
fn source_manifest_validation_rejects_entry_drift() {
    let mut manifest = source_manifest();
    manifest.swap(0, 1);

    let error = validate_source_manifest(&manifest)
        .expect_err("drifted source manifests should fail validation");

    assert_eq!(
        error.to_string(),
        "the VSOP87 source manifest entry 0 is out of sync with the current source catalog (expected Sun / VSOP87B.ear, got Mercury / VSOP87B.mer)"
    );
}

#[test]
fn source_manifest_validation_rejects_length_drift_with_manifest_details() {
    let mut manifest = source_manifest();
    manifest.pop();

    let error = validate_source_manifest(&manifest)
        .expect_err("truncated source manifests should fail validation");

    assert_eq!(
        error.to_string(),
        "the VSOP87 source manifest length is out of sync with the current source catalog (expected 8 entries [Sun / VSOP87B.ear, Mercury / VSOP87B.mer, Venus / VSOP87B.ven, Mars / VSOP87B.mar, Jupiter / VSOP87B.jup, Saturn / VSOP87B.sat, Uranus / VSOP87B.ura, Neptune / VSOP87B.nep], got 7 entries [Sun / VSOP87B.ear, Mercury / VSOP87B.mer, Venus / VSOP87B.ven, Mars / VSOP87B.mar, Jupiter / VSOP87B.jup, Saturn / VSOP87B.sat, Uranus / VSOP87B.ura])"
    );
}

#[test]
fn regenerated_binary_tables_match_the_checked_in_artifacts() {
    for spec in source_specifications() {
        let regenerated = generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .expect("source-backed tables should regenerate");
        let expected = checked_in_generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .expect("supported source files should have a checked-in generated blob");
        assert_eq!(
            regenerated.as_slice(),
            expected,
            "regenerated blob should match {}",
            spec.source_file
        );
    }
}

#[test]
fn checked_in_generated_tables_cover_the_supported_source_file_set() {
    for source_file in supported_source_files() {
        assert!(
            checked_in_generated_vsop87b_table_bytes_for_source_file(source_file).is_some(),
            "supported source file {source_file} should have a checked-in generated blob"
        );
    }
    assert!(checked_in_generated_vsop87b_table_bytes_for_source_file("VSOP87B.plu").is_none());
}

#[test]
fn supported_source_files_are_exposed_for_reproducibility_tooling() {
    assert_eq!(
        supported_source_files(),
        source_documentation_summary().source_files
    );
}

#[test]
fn request_corpus_helper_preserves_body_order_and_defaults() {
    let instant = Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb);
    let bodies = vec![
        CelestialBody::Mars,
        CelestialBody::Sun,
        CelestialBody::Neptune,
    ];
    let requests = requests_for_bodies_at(bodies.clone(), instant, CoordinateFrame::Equatorial);

    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        bodies
    );
    assert!(requests.iter().all(|request| {
        request.instant == instant
            && request.frame == CoordinateFrame::Equatorial
            && request.zodiac_mode == ZodiacMode::Tropical
            && request.apparent == Apparentness::Mean
            && request.observer.is_none()
    }));
}

#[test]
fn source_backed_and_fallback_body_profiles_are_exposed_for_reproducibility_tooling() {
    let source_backed_profiles = source_backed_body_profiles();
    let fallback_profiles = fallback_body_profiles();
    let summary = source_documentation_summary();

    assert_eq!(
        source_backed_profiles.len(),
        summary.source_backed_profile_count
    );
    assert_eq!(fallback_profiles.len(), summary.fallback_profile_count);
    assert_eq!(source_backed_body_order(), summary.source_backed_bodies);
    assert_eq!(
        source_backed_profiles
            .iter()
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>(),
        summary.source_backed_bodies
    );
    assert_eq!(
        fallback_profiles
            .iter()
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>(),
        summary.fallback_bodies
    );
    assert!(fallback_profiles
        .iter()
        .all(|profile| profile.kind == Vsop87BodySourceKind::MeanOrbitalElements));
    assert!(source_backed_profiles
        .iter()
        .all(|profile| profile.kind != Vsop87BodySourceKind::MeanOrbitalElements));
    assert_eq!(
        source_backed_profiles.len() + fallback_profiles.len(),
        body_source_profiles().len()
    );
}

#[test]
fn regeneration_helper_reports_unknown_source_files_explicitly() {
    let error = try_generated_vsop87b_table_bytes_for_source_file("VSOP87B.plu")
        .expect_err("unsupported source files should be rejected");

    assert_eq!(
        error,
        Vsop87TableGenerationError::UnknownSourceFile {
            source_file: "VSOP87B.plu".to_string(),
            supported_source_files: vec![
                "VSOP87B.ear",
                "VSOP87B.mer",
                "VSOP87B.ven",
                "VSOP87B.mar",
                "VSOP87B.jup",
                "VSOP87B.sat",
                "VSOP87B.ura",
                "VSOP87B.nep",
            ],
        }
    );
    assert!(error
        .to_string()
        .contains("no vendored VSOP87B source text found for VSOP87B.plu"));
}

#[test]
fn unified_body_catalog_keeps_profiles_specs_and_samples_aligned() {
    let catalog = body_catalog_entries();
    assert_eq!(catalog.len(), Vsop87Backend::supported_bodies().len());

    let source_backed = catalog
        .iter()
        .filter(|entry| {
            matches!(
                entry.source_profile.kind,
                Vsop87BodySourceKind::TruncatedVsop87b
                    | Vsop87BodySourceKind::VendoredVsop87b
                    | Vsop87BodySourceKind::GeneratedBinaryVsop87b
            )
        })
        .count();
    let fallback = catalog
        .iter()
        .filter(|entry| entry.source_profile.kind == Vsop87BodySourceKind::MeanOrbitalElements)
        .count();
    assert_eq!(source_backed, 8);
    assert_eq!(fallback, 1);

    let pluto = catalog
        .iter()
        .find(|entry| entry.source_profile.body == CelestialBody::Pluto)
        .expect("Pluto entry should exist");
    assert!(pluto.source_specification.is_none());
    assert!(pluto.canonical_sample.is_none());

    let sun = catalog
        .iter()
        .find(|entry| entry.source_profile.body == CelestialBody::Sun)
        .expect("Sun entry should exist");
    assert_eq!(
        sun.source_profile.kind,
        Vsop87BodySourceKind::GeneratedBinaryVsop87b
    );
    assert!(sun.source_specification.is_some());
    assert!(sun.canonical_sample.is_some());
}

#[test]
fn signed_longitude_delta_wraps_across_zero_aries() {
    assert_eq!(signed_longitude_delta_degrees(359.5, 0.5), 1.0);
    assert_eq!(signed_longitude_delta_degrees(0.5, 359.5), -1.0);
}

fn mean_request(body: CelestialBody) -> EphemerisRequest {
    mean_request_at(
        body,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    )
}

fn mean_request_at(body: CelestialBody, instant: Instant) -> EphemerisRequest {
    let mut request = EphemerisRequest::new(body, instant);
    request.apparent = Apparentness::Mean;
    request
}

fn assert_degrees_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = signed_longitude_delta_degrees(expected, actual).abs();
    assert!(
        delta <= tolerance,
        "expected {actual}° to be within {tolerance}° of {expected}°; delta was {delta}°"
    );
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}; delta was {delta}"
    );
}
