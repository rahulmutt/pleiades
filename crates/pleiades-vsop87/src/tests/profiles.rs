use super::*;

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
