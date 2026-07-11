use super::*;

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
fn supported_body_j2000_ecliptic_batch_parity_summary_has_a_displayable_summary_line() {
    let summary =
        supported_body_j2000_ecliptic_batch_parity_summary().expect("batch summary should exist");

    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
}

#[test]
fn supported_body_j2000_equatorial_batch_parity_summary_has_a_displayable_summary_line() {
    let summary =
        supported_body_j2000_equatorial_batch_parity_summary().expect("batch summary should exist");

    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Equatorial);
}

#[test]
fn supported_body_j1900_ecliptic_batch_parity_summary_has_a_displayable_summary_line() {
    let summary =
        supported_body_j1900_ecliptic_batch_parity_summary().expect("batch summary should exist");

    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
}

#[test]
fn supported_body_j1900_equatorial_batch_parity_summary_has_a_displayable_summary_line() {
    let summary =
        supported_body_j1900_equatorial_batch_parity_summary().expect("batch summary should exist");

    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(summary.frame, CoordinateFrame::Equatorial);
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
fn vsop87_claims_majors_constrained_pluto_approximate() {
    use pleiades_backend::{BodyClaimTier, CelestialBody, EphemerisBackend};
    let meta = Vsop87Backend::new().metadata();
    assert_eq!(
        meta.claim_for(&CelestialBody::Mars).map(|c| c.tier),
        Some(BodyClaimTier::Constrained)
    );
    assert_eq!(
        meta.claim_for(&CelestialBody::Pluto).map(|c| c.tier),
        Some(BodyClaimTier::Approximate)
    );
    assert!(meta.release_grade_bodies().is_empty());
}
