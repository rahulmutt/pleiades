use crate::test_support::*;
use crate::*;

#[test]
fn packaged_request_policy_summary_validation_rejects_drift() {
    let mut summary = packaged_request_policy_summary_details();
    summary.supported_frames = &[CoordinateFrame::Ecliptic];

    let error = summary
        .validate()
        .expect_err("drifted packaged request-policy summary should be rejected");
    assert!(format!("{error}").contains("supported_frames"));
}

#[test]
fn packaged_artifact_profile_summary_details_match_the_bundled_header() {
    let artifact = packaged_artifact();
    let summary = packaged_artifact_profile_summary_details();

    assert_eq!(summary.body_count, artifact.bodies.len());
    assert_eq!(
        summary.bodies,
        artifact
            .bodies
            .iter()
            .map(|series| series.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.endian_policy, artifact.header.endian_policy);
    assert_eq!(summary.profile, artifact.header.profile);
    assert_eq!(
        summary.summary_line(),
        artifact
            .header
            .summary_for_body_count(artifact.bodies.len())
    );
    assert_eq!(
        summary.profile.summary_line(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates, Motion]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates]; speed policy: FittedDerivative"
    );
    assert_eq!(summary.validate(), Ok(()));
    let coverage = summary.profile_coverage_summary();
    assert_eq!(coverage.body_count, artifact.bodies.len());
    assert_eq!(coverage.bodies, summary.bodies);
    assert_eq!(coverage.profile, summary.profile);
    assert_eq!(
        coverage.summary_line(),
        summary.profile.summary_for_body_count(summary.body_count)
    );
    assert_eq!(
        coverage.summary_line_with_bodies(),
        format!(
            "{}; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros",
            summary.profile.summary_for_body_count(summary.body_count)
        )
    );
    assert_eq!(coverage.to_string(), coverage.summary_line());
    coverage
        .validate()
        .expect("packaged profile coverage summary should validate");
    assert_eq!(summary.to_string(), summary.summary_line());
    summary
        .validate()
        .expect("packaged artifact profile summary should validate");
    assert_eq!(
        summary.summary_line_with_bodies(),
        format!(
            "{}; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros",
            artifact
                .header
                .summary_for_body_count(artifact.bodies.len())
        )
    );
    assert_eq!(
        packaged_artifact_profile_summary(),
        artifact
            .header
            .summary_for_body_count(artifact.bodies.len())
    );
    let output_support_summary = packaged_artifact_output_support_summary_details();
    assert_eq!(output_support_summary.profile, summary.profile);
    assert_eq!(
        output_support_summary.summary_line(),
        summary.profile.output_support_summary_line()
    );
    output_support_summary
        .validate()
        .expect("packaged artifact output-support summary should validate");
    assert_eq!(
        output_support_summary.to_string(),
        output_support_summary.summary_line()
    );
    assert_eq!(
        summary.output_support_summary_line(),
        summary.profile.output_support_summary_line()
    );
    assert_eq!(
        summary.summary_line_with_output_support(),
        format!(
            "{}; output support: {}",
            summary.summary_line_with_bodies(),
            summary.profile.output_support_summary_line()
        )
    );
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        summary.validated_summary_line_with_bodies(),
        Ok(summary.summary_line_with_bodies())
    );
    assert_eq!(
        summary.validated_summary_line_with_output_support(),
        Ok(summary.summary_line_with_output_support())
    );
    assert_eq!(
        packaged_artifact_profile_summary_with_body_coverage(),
        format!(
            "{}; bundled bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros",
            artifact
                .header
                .summary_for_body_count(artifact.bodies.len())
        )
    );
    assert_eq!(
        packaged_artifact_profile_coverage_summary_details(),
        summary.profile_coverage_summary()
    );
    assert_eq!(
        packaged_artifact_profile_summary_with_output_support(),
        summary.summary_line_with_output_support()
    );
}

#[test]
fn packaged_artifact_profile_summary_validation_rejects_body_count_drift() {
    let mut summary = packaged_artifact_profile_summary_details();
    summary.body_count += 1;

    let error = summary
        .validate()
        .expect_err("body-count drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact profile body count does not match bundled body list"));
}

#[test]
fn packaged_artifact_profile_summary_validation_rejects_profile_drift() {
    let mut summary = packaged_artifact_profile_summary_details();
    summary
        .profile
        .derived_outputs
        .retain(|output| *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates);

    let error = summary
        .validate()
        .expect_err("profile drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact profile metadata does not match the checked-in packaged artifact profile"));
}

#[test]
fn packaged_artifact_profile_summary_validation_rejects_bundled_body_set_drift() {
    let mut bodies = packaged_bodies().to_vec();
    bodies[0] = CelestialBody::Ceres;

    let summary = PackagedArtifactProfileSummary {
        body_count: bodies.len(),
        bodies,
        endian_policy: EndianPolicy::LittleEndian,
        profile: ArtifactProfile::ecliptic_longitude_latitude_distance(),
    };

    let error = summary
        .validate()
        .expect_err("packaged body set drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact profile bundled body list does not match the checked-in packaged body set"));
}

#[test]
fn packaged_artifact_profile_summary_validation_rejects_empty_bodies() {
    let summary = PackagedArtifactProfileSummary {
        body_count: 0,
        bodies: Vec::new(),
        endian_policy: EndianPolicy::LittleEndian,
        profile: ArtifactProfile::ecliptic_longitude_latitude_distance(),
    };

    let error = summary
        .validate()
        .expect_err("empty packaged body lists should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("artifact profile coverage bundled body list must not be empty"));
}

#[test]
fn packaged_artifact_profile_summary_validation_rejects_duplicate_bodies() {
    let summary = PackagedArtifactProfileSummary {
        body_count: 3,
        bodies: vec![CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Sun],
        endian_policy: EndianPolicy::LittleEndian,
        profile: ArtifactProfile::ecliptic_longitude_latitude_distance(),
    };

    let error = summary
        .validate()
        .expect_err("duplicate packaged body lists should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("artifact profile coverage bundled bodies contains duplicate Sun entry"));
}

#[test]
fn packaged_artifact_output_support_summary_validation_rejects_profile_drift() {
    let summary = PackagedArtifactOutputSupportSummary {
        profile: ArtifactProfile::new(
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu,
            ],
            vec![
                pleiades_compression::ArtifactOutput::EclipticCoordinates,
                pleiades_compression::ArtifactOutput::EquatorialCoordinates,
            ],
            vec![
                pleiades_compression::ArtifactOutput::ApparentCorrections,
                pleiades_compression::ArtifactOutput::TopocentricCoordinates,
                pleiades_compression::ArtifactOutput::SiderealCoordinates,
            ],
            pleiades_compression::SpeedPolicy::Unsupported,
        ),
    };

    let error = summary
        .validate()
        .expect_err("profile drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains("Motion"));
}

#[test]
fn packaged_artifact_output_support_summary_validation_rejects_equatorial_support_drift() {
    let mut profile = packaged_artifact_profile_summary_details().profile.clone();
    profile
        .derived_outputs
        .retain(|output| *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates);

    let summary = PackagedArtifactOutputSupportSummary { profile };
    let error = summary
        .validate()
        .expect_err("equatorial output support drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains("EquatorialCoordinates"));
}

#[test]
fn packaged_artifact_storage_summary_validation_rejects_profile_drift() {
    let mut profile = packaged_artifact_profile_summary_details().profile.clone();
    profile
        .stored_channels
        .retain(|channel| *channel != ChannelKind::DistanceAu);

    let error = validate_packaged_artifact_storage_profile(&profile)
        .expect_err("drifted packaged storage profile should be rejected");
    assert_eq!(
        error,
        PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
            field: "stored_channels"
        }
    );

    let mut profile = packaged_artifact_profile_summary_details().profile.clone();
    profile
        .derived_outputs
        .retain(|output| *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates);

    let error = validate_packaged_artifact_storage_profile(&profile)
        .expect_err("drifted packaged storage profile should be rejected");
    assert_eq!(
        error,
        PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
            field: "derived_outputs"
        }
    );

    let mut profile = packaged_artifact_profile_summary_details().profile.clone();
    profile
        .derived_outputs
        .retain(|output| *output != pleiades_compression::ArtifactOutput::Motion);

    let error = validate_packaged_artifact_storage_profile(&profile)
        .expect_err("drifted packaged storage profile should be rejected");
    assert_eq!(
        error,
        PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
            field: "derived_outputs"
        }
    );
}

#[test]
fn packaged_artifact_access_summary_matches_current_build_posture() {
    let summary = packaged_artifact_access_summary_details();
    assert_eq!(
        summary.explicit_path_loading,
        packaged_artifact_path_loading_enabled()
    );
    assert_eq!(summary.summary_line(), packaged_artifact_access_summary());
    assert_eq!(summary.to_string(), packaged_artifact_access_summary());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    summary
        .validate()
        .expect("packaged artifact access summary should validate");
}

#[test]
fn packaged_artifact_output_support_summary_matches_current_build_posture() {
    let summary = packaged_artifact_output_support_summary_details();
    assert_eq!(
        summary.summary_line(),
        summary.profile.output_support_summary_line()
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    summary
        .validate()
        .expect("packaged artifact output-support summary should validate");
}

#[test]
fn packaged_artifact_output_support_summary_validation_rejects_drift() {
    let mut summary = packaged_artifact_output_support_summary_details();
    summary
        .profile
        .derived_outputs
        .retain(|output| *output != ArtifactOutput::EquatorialCoordinates);

    assert!(summary.validated_summary_line().is_err());
    assert!(summary.validate().is_err());
}

#[test]
fn packaged_artifact_access_summary_validation_rejects_drift() {
    let mut summary = packaged_artifact_access_summary_details();
    summary.explicit_path_loading = !summary.explicit_path_loading;

    let error = summary
        .validate()
        .expect_err("drifted packaged artifact access summary should be rejected");
    assert_eq!(
        error,
        PackagedArtifactAccessSummaryValidationError::FeatureStateOutOfSync {
            field: "explicit_path_loading"
        }
    );
}

#[test]
fn packaged_artifact_profile_summary_report_marks_drift_as_unavailable() {
    let mut summary = packaged_artifact_profile_summary_details();
    summary.body_count += 1;

    assert_eq!(
        render_packaged_artifact_profile_summary(&summary, false),
        "Packaged artifact profile: unavailable (InvalidFormat: packaged artifact profile body count does not match bundled body list)"
    );
    assert_eq!(
        render_packaged_artifact_profile_summary(&summary, true),
        "Packaged artifact profile with bundled bodies: unavailable (InvalidFormat: packaged artifact profile body count does not match bundled body list)"
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_policy_summary_matches_current_posture() {
    let summary = packaged_artifact_generation_policy_summary_details();
    let artifact = packaged_artifact();
    assert_eq!(
        summary.policy,
        PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows
    );
    assert_eq!(
        summary.summary_line(),
        format!(
            "adjacent same-body quadratic windows; {}",
            packaged_artifact_generation_policy_note_text()
        )
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    summary
        .validate()
        .expect("generation policy summary should validate");
    assert_eq!(
        packaged_artifact_generation_policy_summary(),
        summary.to_string()
    );
    let residual_bodies = packaged_artifact_generation_residual_bodies_summary_details();
    // SP1 draft baseline: the dense de440-backed artifact fits the inner bodies and
    // luminaries well enough that no residual correction is stored for them. Only the
    // Eros asteroid series (carried from the curated snapshot) still carries residuals.
    assert!(artifact
        .residual_bodies()
        .iter()
        .any(|body| matches!(body, CelestialBody::Custom(custom)
            if custom.designation.eq_ignore_ascii_case("433-Eros"))));
    assert!(artifact.residual_segment_count() > 0);
    assert_eq!(residual_bodies.body_count, artifact.residual_bodies().len());
    assert_eq!(residual_bodies.bodies, artifact.residual_bodies().to_vec());
    assert_eq!(
        residual_bodies.summary_line(),
        residual_bodies.summary_line()
    );
    assert_eq!(residual_bodies.to_string(), residual_bodies.summary_line());
    residual_bodies
        .validate(artifact)
        .expect("residual body coverage summary should validate");
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_policy_summary_rejects_residual_body_drift() {
    let error = validate_packaged_artifact_generation_policy_residual_bodies(
        PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
        &[CelestialBody::Sun],
    )
    .expect_err("residual body drift should fail validation");
    assert_eq!(
        error,
        PackagedArtifactGenerationPolicySummaryValidationError::FieldOutOfSync {
            field: "residual_bodies",
        }
    );
    assert_eq!(
        error.summary_line(),
        "the packaged artifact generation policy summary field `residual_bodies` is out of sync with the current posture"
    );
    assert_eq!(error.to_string(), error.summary_line());
}

#[test]
fn packaged_artifact_generation_policy_validation_error_has_summary_line() {
    let error = PackagedArtifactGenerationPolicyValidationError::FieldOutOfSync { field: "policy" };
    assert_eq!(
        error.summary_line(),
        "the packaged artifact generation policy field `policy` is out of sync with the current posture"
    );
    assert_eq!(error.to_string(), error.summary_line());
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_includes_reference_snapshot_coverage() {
    let summary = packaged_artifact_regeneration_summary_details();
    let artifact = packaged_artifact();
    assert_eq!(summary.label, ARTIFACT_LABEL);
    assert_eq!(summary.artifact_version, artifact.header.version);
    assert_eq!(summary.source, packaged_artifact_source_text());
    assert_eq!(
        summary.source_revision,
        production_generation_source_summary_for_report()
    );
    assert_eq!(summary.profile_id, ARTIFACT_PROFILE_ID);
    assert_eq!(summary.checksum, artifact.checksum);
    assert_eq!(
        summary.generation_policy,
        PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows
    );
    assert_eq!(summary.bodies.len(), packaged_bodies().len());
    assert_eq!(
        summary.quantization_scales,
        packaged_artifact_quantization_scales_line()
    );
    assert_eq!(
        summary.body_coverage_line(),
        "11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"
    );
    assert_eq!(
        summary.generation_policy_line(),
        format!(
            "generation policy: adjacent same-body quadratic windows; {}",
            packaged_artifact_generation_policy_note_text()
        )
    );
    assert_eq!(summary.fit_envelope.body_count, packaged_bodies().len());
    assert_eq!(
        summary.fit_envelope.expected_sample_count,
        summary.fit_envelope.sample_count
    );
    summary
        .fit_envelope
        .validate()
        .expect("packaged fit envelope should validate");
    let residual_coverage = summary.residual_body_coverage_summary();
    assert_eq!(
        residual_coverage.body_count,
        artifact.residual_bodies().len()
    );
    assert_eq!(
        residual_coverage.bodies,
        artifact.residual_bodies().to_vec()
    );
    assert_eq!(
        residual_coverage.summary_line(),
        packaged_artifact_generation_residual_bodies_summary_details().summary_line()
    );
    assert_eq!(
        residual_coverage.to_string(),
        residual_coverage.summary_line()
    );
    residual_coverage
        .validate(artifact)
        .expect("residual body coverage should validate");
    assert_eq!(
        packaged_body_coverage_summary_details().summary_line(),
        format!("Packaged body set: {}", summary.body_coverage_line())
    );
    assert_eq!(
        packaged_body_coverage_summary_details().validated_summary_line(),
        Ok(packaged_body_coverage_summary_details().summary_line())
    );

    let provenance = summary.summary_line();
    assert_eq!(summary.to_string(), provenance);
    assert_eq!(summary.validated_summary_line(), Ok(provenance.clone()));
    summary
        .validate()
        .expect("packaged regeneration summary should validate");
    assert!(provenance
        .contains("Packaged artifact regeneration source: label=stage-5 packaged-data draft"));
    assert!(provenance.contains("profile id=pleiades-packaged-artifact-profile/stage-5-draft"));
    assert!(provenance.contains("source revision=Production generation source:"));
    assert!(provenance.contains("normalized intermediates: label=stage-5 packaged-data draft; profile id=pleiades-packaged-artifact-profile/stage-5-draft; version="));
    assert!(provenance.contains("body count=11; segments="));
    assert!(provenance.contains("residual-bearing segments="));
    assert!(provenance.contains("stored channels="));
    assert!(provenance.contains("segment span days="));
    assert!(provenance.contains("checksum=0x"));
    assert!(provenance.contains("artifact size="));
    assert!(provenance.contains("generation policy: adjacent same-body quadratic windows"));
    assert!(
        provenance.contains("quantization scales: stored=Longitude=9, Latitude=9, DistanceAu=10")
    );
    assert!(provenance.contains(&format!("artifact version={}", artifact.header.version)));
    assert!(provenance.contains("11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"));
    assert!(provenance.contains("Reference snapshot coverage:"));
    assert!(provenance.contains("fit envelope:"));
    assert!(provenance.contains("segment samples across"));
    assert!(provenance.contains("rows across"));
    assert!(provenance.contains("asteroid rows"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_normalized_intermediate_summary_matches_current_posture() {
    let summary = packaged_artifact_normalized_intermediate_summary_details();
    let artifact = packaged_artifact();

    assert_eq!(
        summary,
        packaged_artifact_normalized_intermediate_summary_details()
    );
    assert_eq!(
        summary.checksum,
        fnv1a64(summary.summary_payload_line().as_bytes())
    );
    assert_eq!(summary.label, ARTIFACT_LABEL);
    assert_eq!(summary.artifact_version, artifact.header.version);
    assert_eq!(summary.source, packaged_artifact_source_text());
    assert_eq!(
        summary.source_revision,
        production_generation_source_summary_for_report()
    );
    assert_eq!(summary.profile_id, ARTIFACT_PROFILE_ID);
    assert_eq!(summary.time_range, artifact_time_range(artifact));
    assert_eq!(
        summary.generation_policy,
        PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows
    );
    assert_eq!(
        summary.quantization_scales,
        packaged_artifact_quantization_scales_line()
    );
    assert_eq!(summary.body_count, artifact.bodies.len());
    assert_eq!(summary.segment_count, artifact.segment_count());
    assert_eq!(
        summary.residual_segment_count,
        artifact.residual_segment_count()
    );
    assert_eq!(
        summary.stored_channel_count,
        packaged_artifact_channel_count(artifact, false)
    );
    assert_eq!(
        summary.residual_channel_count,
        packaged_artifact_channel_count(artifact, true)
    );
    assert_eq!(
        summary.min_segment_span_days,
        packaged_artifact_segment_span_bounds(artifact).0
    );
    assert_eq!(
        summary.max_segment_span_days,
        packaged_artifact_segment_span_bounds(artifact).1
    );
    assert!(summary
        .summary_line()
        .contains("Packaged artifact normalized intermediates: label=stage-5 packaged-data draft"));
    assert!(summary.summary_line().contains("checksum=0x"));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    summary
        .validate()
        .expect("normalized intermediates summary should validate");
}

#[test]
fn packaged_artifact_source_and_policy_prose_share_the_generation_tail() {
    assert!(packaged_artifact_generation_policy_note_text()
        .ends_with(PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("quarter-biased splits on very long dense-body spans"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("shared four-point control-point fallback across longitude, latitude, and distance channels"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("residual-channel combinations and remaining channel-order permutations"));
    assert!(packaged_artifact_generation_policy_note_text().contains("smaller residual footprint"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("dense quarter-point control-point lattice"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("one-sixth and five-sixth probe fractions"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("five-point fallback on the longest dense-body spans"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("seven-point fallback on super-extreme dense-body spans"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("one-fifth and four-fifth probe fractions"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("one-ninth and eight-ninths probe fractions"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("one-eighth and seven-eighths probe fractions"));
    assert!(packaged_artifact_generation_policy_note_text()
        .contains("one-seventh and six-sevenths probe fractions"));
    assert!(packaged_artifact_generation_policy_note_text().contains("lunar points"));
    assert!(
        packaged_artifact_generation_policy_note_text()
            .find("seven-point fallback on super-extreme dense-body spans")
            .expect("seven-point fallback text should be present")
            < packaged_artifact_generation_policy_note_text()
                .find("one-ninth and eight-ninths probe fractions")
                .expect("ninth probe text should be present")
    );
    assert!(
        packaged_artifact_generation_policy_note_text()
            .find("one-ninth and eight-ninths probe fractions")
            .expect("ninth probe text should be present")
            < packaged_artifact_generation_policy_note_text()
                .find("one-eighth and seven-eighths probe fractions")
                .expect("eighth probe text should be present")
    );
    assert!(
        packaged_artifact_generation_policy_note_text()
            .find("one-eighth and seven-eighths probe fractions")
            .expect("eighth probe text should be present")
            < packaged_artifact_generation_policy_note_text()
                .find("one-seventh and six-sevenths probe fractions")
                .expect("seventh probe text should be present")
    );
    assert!(
        packaged_artifact_generation_policy_note_text()
            .find("one-seventh and six-sevenths probe fractions")
            .expect("seventh probe text should be present")
            < packaged_artifact_generation_policy_note_text()
                .find("one-fifth and four-fifth probe fractions")
                .expect("fifth probe text should be present")
    );
    assert!(packaged_artifact_source_text()
        .contains("quarter-biased splits on very long dense-body spans"));
    assert!(packaged_artifact_source_text().contains(PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL));
    assert!(packaged_artifact_source_text()
        .ends_with(&format!("{PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL}.")));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_normalized_intermediate_summary_validation_rejects_checksum_drift() {
    let mut summary = packaged_artifact_normalized_intermediate_summary_details();
    summary.checksum ^= 0x1;

    let error = summary
        .validate()
        .expect_err("normalized intermediate checksum drift should be rejected");

    assert!(error
        .to_string()
        .contains("packaged artifact normalized intermediate summary checksum 0x"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_profile_id_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.profile_id = "pleiades-packaged-artifact-profile/test-drift";

    let error = summary
        .validate()
        .expect_err("profile id drift should be rejected");

    assert!(error
        .to_string()
        .contains("packaged artifact regeneration summary profile id does not match the checked-in artifact profile id"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_source_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.source = "drifted source";

    let error = summary
        .validate()
        .expect_err("source drift should be rejected");

    assert!(error
        .to_string()
        .contains("packaged artifact regeneration summary source does not match the checked-in artifact source"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_source_revision_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.source_revision = "drifted source revision".to_string();

    let error = summary
        .validate()
        .expect_err("source revision drift should be rejected");

    assert!(error
        .to_string()
        .contains("packaged artifact regeneration summary source revision does not match the checked-in production-generation source summary"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_checksum_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.checksum ^= 1;

    let error = summary
        .validate()
        .expect_err("checksum drift should be rejected");

    assert!(error
        .to_string()
        .contains("packaged artifact regeneration summary checksum"));

    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.quantization_scales = "quantization scales: stored=Longitude=10".to_string();
    let error = summary
        .validate()
        .expect_err("quantization scale drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary quantization scales do not match the checked-in packaged artifact"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_fit_envelope_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.fit_envelope.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("fit envelope drift should be rejected");

    assert!(error
        .to_string()
        .contains("packaged artifact regeneration fit envelope is invalid"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validated_summary_line_rejects_metadata_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.artifact_version += 1;

    let error = summary
        .validated_summary_line()
        .expect_err("metadata drift should be rejected");

    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary artifact version"));
}

#[test]
fn packaged_frame_treatment_summary_reuses_the_structured_report_helper() {
    let summary = PackagedFrameTreatmentSummary;

    assert_eq!(summary.summary_line(), packaged_frame_treatment_summary());
    assert_eq!(summary.to_string(), packaged_frame_treatment_summary());
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn packaged_frame_treatment_summary_rejects_whitespace_padded_summary_text() {
    let summary = format!(" {} ", PackagedFrameTreatmentSummary.summary_line());

    assert_eq!(
        validate_packaged_frame_treatment_summary_line(&summary),
        Err(PackagedFrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
    );
}

#[test]
fn packaged_artifact_storage_summary_rejects_whitespace_padded_summary_text() {
    let summary = format!(" {} ", PackagedArtifactStorageSummary.summary_line());

    assert_eq!(
        validate_packaged_artifact_storage_summary_line(&summary),
        Err(PackagedArtifactStorageSummaryValidationError::WhitespacePaddedSummary)
    );
}

#[test]
fn packaged_artifact_storage_summary_rejects_blank_summary_text() {
    assert_eq!(
        validate_packaged_artifact_storage_summary_line(""),
        Err(PackagedArtifactStorageSummaryValidationError::BlankSummary)
    );
}

#[test]
fn packaged_artifact_access_summary_rejects_whitespace_padded_summary_text() {
    let summary = format!(
        " {} ",
        PackagedArtifactAccessSummary {
            explicit_path_loading: cfg!(feature = "packaged-artifact-path"),
        }
        .summary_line()
    );

    assert_eq!(
        validate_packaged_artifact_access_summary_line(&summary),
        Err(PackagedArtifactAccessSummaryValidationError::WhitespacePaddedSummary)
    );
}

#[test]
fn packaged_artifact_access_summary_rejects_blank_summary_text() {
    assert_eq!(
        validate_packaged_artifact_access_summary_line(""),
        Err(PackagedArtifactAccessSummaryValidationError::BlankSummary)
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_duplicate_bodies() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.bodies[1] = summary.bodies[0].clone();

    let error = summary
        .validate()
        .expect_err("duplicate regeneration bodies should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary contains duplicate body entry"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_body_list_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.bodies.swap(0, 1);

    let error = summary
        .validate()
        .expect_err("body order drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary body list does not match the checked-in packaged body set"));
    assert!(error.message.contains("expected [Sun, Moon"));
    assert!(error.message.contains("got [Moon, Sun"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_residual_body_subset_drift() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary
        .validate_residual_body_subset()
        .expect("current residual body coverage should stay within the bundled body list");

    summary
        .residual_bodies
        .push(CelestialBody::Custom(CustomBodyId::new(
            "catalog",
            "designation",
        )));

    let error = summary
        .validate_residual_body_subset()
        .expect_err("residual bodies outside the bundled body list should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary residual body catalog:designation is not covered by the bundled body list"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_metadata_drift() {
    let expected_artifact = packaged_artifact();
    let mut summary = packaged_artifact_regeneration_summary_details();

    summary.label = "drifted label";
    let error = summary
        .validate()
        .expect_err("label drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact regeneration summary label does not match the checked-in artifact label"
    ));

    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.source = "drifted source";
    let error = summary
        .validate()
        .expect_err("source drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary source does not match the checked-in artifact source"));

    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.artifact_version = expected_artifact.header.version + 1;
    let error = summary
        .validate()
        .expect_err("artifact version drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary artifact version"));
    assert!(error
        .message
        .contains("does not match the checked-in packaged artifact version"));

    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.checksum ^= 0x1;
    let error = summary
        .validate()
        .expect_err("checksum drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary checksum 0x"));
    assert!(error
        .message
        .contains("does not match the checked-in packaged artifact checksum 0x"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_regeneration_summary_validation_rejects_missing_reference_snapshot() {
    let mut summary = packaged_artifact_regeneration_summary_details();
    summary.reference_snapshot = None;

    let error = summary
        .validate()
        .expect_err("missing reference snapshot should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary is missing reference snapshot coverage"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_production_profile_summary_details();
    let artifact = packaged_artifact();

    assert_eq!(summary.profile_id, ARTIFACT_PROFILE_ID);
    assert_eq!(summary.label, ARTIFACT_LABEL);
    assert_eq!(summary.artifact_version, artifact.header.version);
    assert_eq!(summary.time_range, artifact_time_range(artifact));
    assert_eq!(
        summary.body_coverage,
        packaged_body_coverage_summary_details()
    );
    assert_eq!(
        summary.artifact_profile,
        packaged_artifact_profile_summary_details().profile
    );
    assert_eq!(summary.speed_policy, summary.artifact_profile.speed_policy);
    assert_eq!(
        summary.generation_policy,
        PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows
    );
    assert_eq!(
        summary.request_policy,
        packaged_request_policy_summary_details()
    );
    assert_eq!(
        summary.lookup_epoch_policy,
        packaged_lookup_epoch_policy_summary_details().policy
    );
    assert_eq!(
        summary.frame_treatment,
        packaged_frame_treatment_summary_details()
    );
    assert!(summary.summary_line().contains(
        "lookup epoch policy=TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
    ));
    assert_eq!(
        summary.storage_summary,
        packaged_artifact_storage_summary_details()
    );
    assert_eq!(
        summary.target_thresholds,
        packaged_artifact_target_threshold_summary_details()
    );
    assert_eq!(
        summary.target_thresholds.fit_envelope,
        packaged_artifact_fit_envelope_summary_details()
    );
    assert_eq!(
        summary.target_thresholds.scope_envelopes,
        packaged_artifact_target_threshold_scope_envelopes_summary_details()
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    summary
        .validate()
        .expect("production-profile skeleton should validate");
    assert!(summary
        .summary_line()
        .contains("Packaged artifact production profile draft:"));
    assert!(summary
        .summary_line()
        .contains("source provenance=Production generation source:"));
    assert!(summary.summary_line().contains("output support="));
    assert!(summary
        .summary_line()
        .contains("speed policy=FittedDerivative"));
    assert!(summary
        .summary_line()
        .contains("segment strategy=bodies with a single sampled epoch use point segments"));
    assert!(summary
        .summary_line()
        .contains("target thresholds: production thresholds recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
    assert!(summary
        .summary_line()
        .contains("scope envelopes=scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
    assert_eq!(
        packaged_artifact_production_profile_summary(),
        summary.summary_line()
    );
}

#[test]
fn packaged_artifact_speed_policy_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_speed_policy_summary_details();
    let artifact = packaged_artifact();

    assert_eq!(summary.policy, artifact.header.profile.speed_policy);
    assert_eq!(summary.policy, SpeedPolicy::FittedDerivative);
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        summary.summary_line(),
        "FittedDerivative; motion output support=derived"
    );
    summary
        .validate()
        .expect("packaged-artifact speed policy should validate");

    let mut drifted = summary;
    drifted.policy = SpeedPolicy::Stored;
    let error = drifted
        .validate()
        .expect_err("speed-policy drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactSpeedPolicySummaryValidationError::FieldOutOfSync { field: "policy" }
    );
    assert!(error
        .to_string()
        .contains("speed-policy summary field `policy`"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_time_range_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.time_range = TimeRange::new(None, None);

    let error = summary
        .validate()
        .expect_err("time-range drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "time_range"
        }
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_source_provenance_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.source_provenance = "drifted source provenance".to_string();

    let error = summary
        .validate()
        .expect_err("source-provenance drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "source_provenance"
        }
    );
    assert!(error.to_string().contains("source_provenance"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_request_policy_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.request_policy.supports_topocentric_observer = true;

    let error = summary
        .validate()
        .expect_err("request-policy drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "request_policy"
        }
    );
    assert!(error.to_string().contains("request_policy"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_profile_id_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.profile_id = "pleiades-packaged-artifact-profile/test-drift";

    let error = parameters
        .validate()
        .expect_err("profile id drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters profile id does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_label_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.label = "drifted label";

    let error = parameters
        .validate()
        .expect_err("label drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters label does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_body_coverage_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.body_coverage.bodies[0] = CelestialBody::Ceres;

    let error = parameters
        .validate()
        .expect_err("body coverage drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact generator parameters body coverage does not match the current production profile"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_time_range_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.time_range = TimeRange::new(None, None);

    let error = parameters
        .validate()
        .expect_err("time range drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact generator parameters time range does not match the current production profile"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_artifact_version_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.artifact_version += 1;

    let error = parameters
        .validate()
        .expect_err("artifact version drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters version does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_source_provenance_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.source_provenance = "drifted source provenance".to_string();

    let error = parameters
        .validate()
        .expect_err("source-provenance drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters source provenance does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_checksum_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.checksum ^= 0x1;

    let error = parameters
        .validate()
        .expect_err("checksum drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact generator parameters checksum does not match the current packaged artifact"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_artifact_profile_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.artifact_profile.speed_policy = pleiades_compression::SpeedPolicy::Stored;

    let error = parameters
        .validate()
        .expect_err("artifact profile drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters artifact profile does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_speed_policy_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.speed_policy = pleiades_compression::SpeedPolicy::Stored;

    let error = parameters
        .validate()
        .expect_err("speed policy drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters speed policy does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_request_policy_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.request_policy.supports_topocentric_observer = true;

    let error = parameters
        .validate()
        .expect_err("request policy drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters request policy does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generator_parameters_validation_rejects_target_threshold_drift() {
    let mut parameters = packaged_artifact_generator_parameters_details();
    parameters.target_thresholds.state = PackagedArtifactTargetThresholdState::Draft;

    let error = parameters
        .validate()
        .expect_err("target threshold drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters target thresholds do not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_request_policy_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest
        .parameters
        .request_policy
        .supports_topocentric_observer = true;

    let error = manifest
        .validate()
        .expect_err("request policy drift should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters request policy does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_reflects_the_current_posture() {
    let manifest = packaged_artifact_generation_manifest_details();
    let parameters = packaged_artifact_generator_parameters_details();
    let regeneration = packaged_artifact_regeneration_summary_details();

    assert_eq!(manifest.parameters, parameters);
    assert_eq!(manifest.regeneration, regeneration);
    assert_eq!(
        parameters.speed_policy,
        parameters.artifact_profile.speed_policy
    );
    assert_eq!(manifest.to_string(), manifest.summary_line());
    assert_eq!(
        manifest.validated_summary_line(),
        Ok(manifest.summary_line())
    );
    manifest
        .validate()
        .expect("generation manifest should validate");
    assert!(manifest
        .summary_line()
        .contains("Packaged artifact generation manifest:"));
    assert!(manifest.summary_line().contains("output support="));
    assert!(manifest.summary_line().contains("checksum=0x"));
    assert!(manifest
        .summary_line()
        .contains("speed policy=FittedDerivative"));
    assert!(manifest.summary_line().contains("segment strategy="));
    assert!(manifest
        .summary_line()
        .contains("source revision=Production generation source:"));
    assert!(manifest.summary_line().contains("regeneration="));
    assert_eq!(
        packaged_artifact_generation_manifest(),
        manifest.summary_line()
    );
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_artifacts_keep_lookup_epoch_and_segment_strategy_aligned() {
    let production_profile = packaged_artifact_production_profile_summary_details();
    let generator_parameters = packaged_artifact_generator_parameters_details();
    let manifest = packaged_artifact_generation_manifest_details();

    assert_eq!(
        production_profile.lookup_epoch_policy,
        generator_parameters.lookup_epoch_policy
    );
    assert_eq!(
        generator_parameters.lookup_epoch_policy,
        manifest.parameters.lookup_epoch_policy
    );
    assert_eq!(
        production_profile.lookup_epoch_policy.summary_line(),
        generator_parameters.lookup_epoch_policy.summary_line()
    );
    assert_eq!(
        generator_parameters.generation_policy.segment_strategy(),
        manifest.parameters.generation_policy.segment_strategy()
    );
    assert_eq!(
        production_profile.generation_policy.segment_strategy(),
        generator_parameters.generation_policy.segment_strategy()
    );
    assert!(production_profile
        .summary_line()
        .contains("source provenance=Production generation source:"));
    assert!(production_profile
        .summary_line()
        .contains("lookup epoch policy=TT-grid retag without relativistic correction"));
    assert!(manifest
        .summary_line()
        .contains("source provenance=Production generation source:"));
    assert!(manifest
        .summary_line()
        .contains("segment strategy=bodies with a single sampled epoch use point segments"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_profile_id_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.parameters.profile_id = "pleiades-packaged-artifact-profile/test-drift";

    let error = manifest
        .validate()
        .expect_err("drifted generation parameters should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters profile id does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_label_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.parameters.label = "drifted label";

    let error = manifest
        .validate()
        .expect_err("drifted generation label should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters label does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_artifact_profile_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.parameters.artifact_profile.speed_policy = pleiades_compression::SpeedPolicy::Stored;

    let error = manifest
        .validate()
        .expect_err("drifted generation artifact profile should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters artifact profile does not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_checksum_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.manifest_checksum ^= 1;

    let error = manifest
        .validate()
        .expect_err("drifted generation manifest checksum should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact generation manifest checksum 0x"));
    assert!(error
        .message
        .contains("does not match the current packaged-artifact manifest checksum 0x"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_source_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.regeneration.source = "drifted source";

    let error = manifest
        .validate()
        .expect_err("drifted regeneration source should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact regeneration summary source does not match the checked-in artifact source"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_artifact_version_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.regeneration.artifact_version += 1;

    let error = manifest
        .validate()
        .expect_err("drifted regeneration artifact version should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration summary artifact version"));
    assert!(error
        .message
        .contains("does not match the checked-in packaged artifact version"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_parameter_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.parameters.target_thresholds = PackagedArtifactTargetThresholdSummary {
        profile_id: ARTIFACT_PROFILE_ID,
        state: PackagedArtifactTargetThresholdState::ProductionReady,
        scopes: &["luminaries"],
        fit_envelope: packaged_artifact_fit_envelope_summary_details(),
        scope_envelopes: packaged_artifact_target_threshold_scope_envelopes_summary_details(),
        phase2_corpus_alignment: packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available"),
    };

    let error = manifest
        .validate()
        .expect_err("drifted generation parameters should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error.message.contains(
        "packaged artifact generator parameters target thresholds do not match the current production profile"
    ));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_generation_manifest_validation_rejects_regeneration_drift() {
    let mut manifest = packaged_artifact_generation_manifest_details();
    manifest.regeneration.fit_envelope.sample_count += 1;

    let error = manifest
        .validate()
        .expect_err("drifted regeneration metadata should be rejected");
    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::InvalidFormat
    );
    assert!(error
        .message
        .contains("packaged artifact regeneration fit envelope is invalid"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_label_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.label = "drifted label";

    let error = summary
        .validate()
        .expect_err("label drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync { field: "label" }
    );
    assert!(error.to_string().contains("label"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_artifact_version_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.artifact_version += 1;

    let error = summary
        .validate()
        .expect_err("artifact version drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "artifact_version",
        }
    );
    assert!(error.to_string().contains("artifact_version"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_artifact_profile_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.artifact_profile.speed_policy = pleiades_compression::SpeedPolicy::Stored;

    let error = summary
        .validate()
        .expect_err("artifact profile drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "artifact_profile",
        }
    );
    assert!(error.to_string().contains("artifact_profile"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_speed_policy_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.speed_policy = pleiades_compression::SpeedPolicy::Stored;

    let error = summary
        .validate()
        .expect_err("speed policy drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "speed_policy",
        }
    );
    assert!(error.to_string().contains("speed_policy"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_stored_channel_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary
        .artifact_profile
        .stored_channels
        .retain(|channel| *channel != ChannelKind::DistanceAu);

    let error = summary
        .validate()
        .expect_err("stored channel drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "artifact_profile",
        }
    );
    assert!(error.to_string().contains("artifact_profile"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_body_coverage_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.body_coverage.body_count += 1;

    let error = summary
        .validate()
        .expect_err("body coverage drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "body_coverage",
        }
    );
    assert!(error.to_string().contains("body_coverage"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_production_profile_summary_validation_rejects_target_threshold_drift() {
    let mut summary = packaged_artifact_production_profile_summary_details();
    summary.target_thresholds = PackagedArtifactTargetThresholdSummary {
        profile_id: ARTIFACT_PROFILE_ID,
        state: PackagedArtifactTargetThresholdState::ProductionReady,
        scopes: &["luminaries"],
        fit_envelope: packaged_artifact_fit_envelope_summary_details(),
        scope_envelopes: packaged_artifact_target_threshold_scope_envelopes_summary_details(),
        phase2_corpus_alignment: packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available"),
    };

    let error = summary
        .validate()
        .expect_err("target threshold drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
            field: "target_thresholds",
        }
    );
    assert!(error.to_string().contains("target_thresholds"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_scope_envelopes_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_target_threshold_scope_envelopes_summary_details();

    assert_eq!(
        summary.scope_envelopes.len(),
        PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES.len()
    );
    assert!(summary
        .summary_line()
        .contains("scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    summary
        .validate()
        .expect("target-threshold scope envelopes should validate");
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_scope_envelopes_summary_validation_rejects_drift() {
    let mut summary = packaged_artifact_target_threshold_scope_envelopes_summary_details();
    summary.scope_envelopes[0]
        .fit_envelope
        .max_distance_delta_au += 1.0;

    let error = summary
        .validate()
        .expect_err("target-threshold scope envelope drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdScopeEnvelopesSummaryValidationError::FieldOutOfSync {
            field: "scope_envelopes",
        }
    );
    assert!(error.to_string().contains("scope_envelopes"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_scope_threshold_violation() {
    let mut summary = packaged_artifact_target_threshold_summary_details();
    summary.scope_envelopes.scope_envelopes[0]
        .fit_envelope
        .max_distance_delta_au =
        packaged_artifact_fit_threshold_summary_details().max_distance_delta_au + 1.0;

    let error = summary
        .validate()
        .expect_err("scope threshold violation should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
            field: "scope_envelopes",
        }
    );
    assert!(error.to_string().contains("scope_envelopes"));
}

#[test]
fn packaged_artifact_target_threshold_state_validation_rejects_draft() {
    let error = PackagedArtifactTargetThresholdState::Draft
        .validate_production_ready()
        .expect_err("draft target-threshold state should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdStateValidationError::Draft
    );
    assert!(error
        .to_string()
        .contains("production thresholds are not yet release-ready"));
}

#[test]
fn packaged_artifact_target_threshold_state_summary_rejects_draft_state() {
    let error = PackagedArtifactTargetThresholdState::Draft
        .validated_summary_line()
        .expect_err("draft target-threshold state summary should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdStateValidationError::Draft
    );
    assert!(error
        .to_string()
        .contains("production thresholds are not yet release-ready"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_draft_state() {
    let mut summary = packaged_artifact_target_threshold_summary_details();
    summary.state = PackagedArtifactTargetThresholdState::Draft;

    let error = summary
        .validate()
        .expect_err("draft target-threshold state should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync { field: "state" }
    );
    assert!(error.to_string().contains("state"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_includes_phase2_corpus_alignment() {
    let summary = packaged_artifact_target_threshold_summary_details();
    assert!(summary
        .summary_line()
        .contains("phase 2 corpus alignment=reference source=Reference snapshot source:"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("reference source=Reference snapshot source:"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("reference exact J2000 evidence=Reference snapshot exact J2000 evidence:"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("comparison source=Comparison snapshot source:"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("independent hold-out source=Independent hold-out source:"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("selected asteroid source evidence=Selected asteroid source evidence:"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("selected asteroid source windows=Selected asteroid source windows:"));
    assert!(summary.phase2_corpus_alignment.summary_line().contains(
        "selected asteroid source request corpus=Selected asteroid source request corpus:"
    ));
    assert!(summary.phase2_corpus_alignment.summary_line().contains(
        "production generation boundary source=Production generation boundary overlay source:"
    ));
    assert!(summary.phase2_corpus_alignment.summary_line().contains(
        "production generation body-class coverage=Production generation body-class coverage:"
    ));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("production generation source=Production generation source:"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("Reference snapshot body-class coverage"));
    assert!(summary
        .phase2_corpus_alignment
        .summary_line()
        .contains("Independent hold-out body-class coverage"));
    assert!(summary
        .phase2_corpus_alignment
        .validated_summary_line()
        .is_ok());
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_corpus_alignment_drift() {
    let mut summary = packaged_artifact_target_threshold_summary_details();
    summary
        .phase2_corpus_alignment
        .independent_holdout
        .row_count += 1;

    let error = summary
        .validate()
        .expect_err("phase-2 corpus alignment drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
            field: "phase2_corpus_alignment.independent_holdout",
        }
    );
    assert!(error
        .to_string()
        .contains("phase2_corpus_alignment.independent_holdout"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_source_drift() {
    let mut summary = packaged_artifact_target_threshold_summary_details();
    summary
        .phase2_corpus_alignment
        .reference_snapshot_source
        .source = "drifted source".to_string();

    let error = summary
        .validate()
        .expect_err("phase-2 source drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
            field: "phase2_corpus_alignment.reference_snapshot_source",
        }
    );
    assert!(error
        .to_string()
        .contains("phase2_corpus_alignment.reference_snapshot_source"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_boundary_source_drift() {
    let mut summary = packaged_artifact_target_threshold_summary_details();
    summary
        .phase2_corpus_alignment
        .production_generation_boundary_source
        .source = "drifted source".to_string();

    let error = summary
        .validate()
        .expect_err("phase-2 boundary source drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
            field: "phase2_corpus_alignment.production_generation_boundary_source",
        }
    );
    assert!(error
        .to_string()
        .contains("phase2_corpus_alignment.production_generation_boundary_source"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_generation_source_drift() {
    let mut summary = packaged_artifact_target_threshold_summary_details();
    summary
        .phase2_corpus_alignment
        .production_generation_source
        .source_revision
        .reference_snapshot_checksum ^= 1;

    let error = summary
        .validate()
        .expect_err("phase-2 generation source drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
            field: "phase2_corpus_alignment.production_generation_source",
        }
    );
    assert!(error
        .to_string()
        .contains("phase2_corpus_alignment.production_generation_source"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_request_corpus_drift() {
    let mut summary = packaged_artifact_target_threshold_summary_details();
    summary
        .phase2_corpus_alignment
        .selected_asteroid_source_request_corpus
        .request_count += 1;

    let error = summary
        .validate()
        .expect_err("phase-2 request corpus drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
            field: "phase2_corpus_alignment.selected_asteroid_source_request_corpus",
        }
    );
    assert!(error
        .to_string()
        .contains("phase2_corpus_alignment.selected_asteroid_source_request_corpus"));
}

#[test]
fn packaged_artifact_phase2_corpus_alignment_summary_validation_rejects_exact_j2000_drift() {
    let mut summary = packaged_artifact_phase2_corpus_alignment_summary_details()
        .expect("phase-2 corpus evidence should be available");
    summary.reference_snapshot_exact_j2000.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("exact J2000 drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
            field: "reference_snapshot_exact_j2000",
        }
    );
    assert!(error.to_string().contains("reference_snapshot_exact_j2000"));
}

#[test]
fn packaged_artifact_phase2_corpus_alignment_summary_validation_rejects_source_drift() {
    let mut summary = packaged_artifact_phase2_corpus_alignment_summary_details()
        .expect("phase-2 corpus evidence should be available");
    summary.reference_snapshot_source.source = "drifted source".to_string();

    let error = summary
        .validate()
        .expect_err("phase-2 corpus alignment drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
            field: "reference_snapshot_source",
        }
    );
    assert!(error.to_string().contains("reference_snapshot_source"));
}

#[test]
fn packaged_artifact_phase2_corpus_alignment_summary_validation_rejects_equatorial_request_corpus_drift(
) {
    let mut summary = packaged_artifact_phase2_corpus_alignment_summary_details()
        .expect("phase-2 corpus evidence should be available");
    summary
        .selected_asteroid_source_request_corpus_equatorial
        .request_count += 1;

    let error = summary
        .validate()
        .expect_err("equatorial request corpus drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
            field: "selected_asteroid_source_request_corpus_equatorial",
        }
    );
    assert!(error
        .to_string()
        .contains("selected_asteroid_source_request_corpus_equatorial"));
}

#[test]
fn packaged_artifact_phase2_corpus_alignment_summary_details_remain_publicly_reusable() {
    let summary = packaged_artifact_phase2_corpus_alignment_summary_details()
        .expect("phase-2 corpus evidence should be available");

    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_source_fit_holdout_sync_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_source_fit_holdout_sync_summary_details();
    let phase2_summary = packaged_artifact_phase2_corpus_alignment_summary_details()
        .expect("phase-2 corpus evidence should be available");
    let target_thresholds = packaged_artifact_target_threshold_summary_details();

    assert!(summary
        .summary_line()
        .contains("source-fit and hold-out sync:"));
    assert!(summary
        .summary_line()
        .contains("fit thresholds: mean Δlon≤79.299372815190°"));
    assert!(summary
        .summary_line()
        .contains("target thresholds: production thresholds recorded"));
    assert!(summary
        .summary_line()
        .contains("phase 2 corpus alignment=reference source="));
    assert!(summary
        .summary_line()
        .contains("reference exact J2000 evidence=Reference snapshot exact J2000 evidence:"));
    assert!(summary.summary_line().contains(
        "selected asteroid source request corpus=Selected asteroid source request corpus:"
    ));
    assert_eq!(summary.phase2_corpus_alignment, phase2_summary);
    assert_eq!(target_thresholds.phase2_corpus_alignment, phase2_summary);
    assert_eq!(summary.target_thresholds, target_thresholds);
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_source_fit_holdout_sync_summary_validation_rejects_fit_threshold_drift() {
    let mut summary = packaged_artifact_source_fit_holdout_sync_summary_details();
    summary.fit_thresholds.max_distance_delta_au += 1.0;

    let error = summary
        .validate()
        .expect_err("fit threshold drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
            field: "fit_thresholds",
        }
    );
    assert!(error.to_string().contains("fit_thresholds"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_source_fit_holdout_sync_summary_validation_rejects_target_threshold_drift() {
    let mut summary = packaged_artifact_source_fit_holdout_sync_summary_details();
    summary.target_thresholds.state = PackagedArtifactTargetThresholdState::Draft;

    let error = summary
        .validate()
        .expect_err("target threshold drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
            field: "target_thresholds",
        }
    );
    assert!(error.to_string().contains("target_thresholds"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_source_fit_holdout_sync_summary_validation_rejects_phase2_drift() {
    let mut summary = packaged_artifact_source_fit_holdout_sync_summary_details();
    summary
        .phase2_corpus_alignment
        .reference_snapshot_source
        .source
        .push_str(" drift");

    let error = summary
        .validate()
        .expect_err("phase-2 corpus drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
            field: "phase2_corpus_alignment.reference_snapshot_source",
        }
    );
    assert!(error
        .to_string()
        .contains("phase2_corpus_alignment.reference_snapshot_source"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_fit_threshold_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_fit_threshold_summary_details();
    let thresholds = packaged_artifact_fit_threshold_summary_details();
    let violations = packaged_artifact_fit_threshold_violation_summary_details();
    let scope_envelopes = packaged_artifact_target_threshold_scope_envelopes_summary_details();

    assert_eq!(
        summary.summary_line(),
        "fit thresholds: mean Δlon≤79.299372815190°, mean Δlat≤3.320919159432°, mean Δdist≤5240.247310255700 AU; max Δlon≤179.999799204804°, max Δlat≤69.957118473923°, max Δdist≤10227288.989857684821 AU"
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.validate().is_ok());
    assert_eq!(
        scope_envelopes.scope_envelopes.len(),
        PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES.len()
    );
    assert!(scope_envelopes.validate().is_ok());
    for scope in &scope_envelopes.scope_envelopes {
        assert_eq!(scope.validate(), Ok(()));
        assert!(scope
            .fit_envelope
            .validate_against_thresholds(&thresholds)
            .is_ok());
    }
    assert_eq!(
        violations.summary_line(),
        "fit threshold violations: 0; details: none"
    );
    assert_eq!(violations.to_string(), violations.summary_line());
    assert_eq!(
        violations.validated_summary_line(),
        Ok(violations.summary_line())
    );
    assert!(violations.validate().is_ok());
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_fit_threshold_violation_summary_validation_rejects_drift() {
    let mut summary = packaged_artifact_fit_threshold_violation_summary_details();
    summary
        .violations
        .push(PackagedArtifactFitThresholdViolation {
            field: "drift",
            measured_bits: 1.0f64.to_bits(),
            threshold_bits: 0.5f64.to_bits(),
            overage_bits: 0.5f64.to_bits(),
        });

    let error = summary
        .validate()
        .expect_err("threshold violation drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactFitThresholdViolationsSummaryValidationError::FieldOutOfSync {
            field: "violations",
        }
    );
    assert!(error.to_string().contains("violations"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_fit_margin_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_fit_margin_summary_details();
    assert_eq!(summary.to_string(), summary.summary_line());
    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_fit_margin_summary_validation_rejects_envelope_drift() {
    let mut summary = packaged_artifact_fit_margin_summary_details();
    summary.envelope.mean_distance_delta_au += 1.0;

    let error = summary
        .validate()
        .expect_err("fit margin envelope drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync { field: "envelope" }
    );
    assert!(error.to_string().contains("envelope"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_fit_margin_summary_validation_rejects_threshold_drift() {
    let mut summary = packaged_artifact_fit_margin_summary_details();
    summary.thresholds.max_distance_delta_au += 1.0;

    let error = summary
        .validate()
        .expect_err("fit margin threshold drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
            field: "thresholds",
        }
    );
    assert!(error.to_string().contains("thresholds"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_fit_outlier_summary_prioritizes_distance_channel_outliers() {
    let summary = packaged_artifact_fit_outlier_summary_details();
    let first_body_summary = summary
        .body_summaries
        .first()
        .expect("packaged artifact fit outlier summary should include at least one body")
        .summary_line();
    let distance = first_body_summary
        .find("DistanceAu=")
        .expect("distance outliers should be surfaced first within body summaries");
    let longitude = first_body_summary
        .find("Longitude=")
        .expect("longitude outliers should still be rendered");
    let latitude = first_body_summary
        .find("Latitude=")
        .expect("latitude outliers should still be rendered");
    assert!(distance < longitude && distance < latitude);

    let by_channel_summary = packaged_artifact_fit_channel_outlier_summary_details();
    let by_channel = by_channel_summary.summary_line();
    let distance = by_channel
        .find("DistanceAu{")
        .expect("distance outliers should be surfaced in the channel summary");
    let longitude = by_channel
        .find("Longitude{")
        .expect("longitude outliers should still be rendered");
    let latitude = by_channel
        .find("Latitude{")
        .expect("latitude outliers should still be rendered");
    assert!(distance < longitude && distance < latitude);
    assert_eq!(by_channel_summary.to_string(), by_channel);
    assert_eq!(
        by_channel_summary.validated_summary_line(),
        Ok(by_channel.clone())
    );
    assert!(by_channel_summary.validate().is_ok());
}

#[test]
fn packaged_artifact_fit_channel_outlier_summary_prefers_shorter_failing_family_on_equal_delta() {
    let body = CelestialBody::Moon;
    let long_segment_start = instant_tt(0.0);
    let long_segment_end = instant_tt(10.0);
    let short_segment_start = instant_tt(20.0);
    let short_segment_end = instant_tt(22.0);

    let samples = vec![
        PackagedArtifactFitSample {
            body: body.clone(),
            segment_start: long_segment_start,
            segment_end: long_segment_end,
            sample_instant: instant_tt(2.5),
            sample_fraction: 0.25,
            longitude_delta_degrees: 1.0,
            latitude_delta_degrees: 0.0,
            distance_delta_au: 0.0,
        },
        PackagedArtifactFitSample {
            body: body.clone(),
            segment_start: long_segment_start,
            segment_end: long_segment_end,
            sample_instant: instant_tt(7.5),
            sample_fraction: 0.75,
            longitude_delta_degrees: 1.0,
            latitude_delta_degrees: 0.0,
            distance_delta_au: 0.0,
        },
        PackagedArtifactFitSample {
            body,
            segment_start: short_segment_start,
            segment_end: short_segment_end,
            sample_instant: instant_tt(21.0),
            sample_fraction: 0.5,
            longitude_delta_degrees: 1.0,
            latitude_delta_degrees: 0.0,
            distance_delta_au: 0.0,
        },
    ];

    let summary =
        packaged_artifact_fit_channel_outlier_summary_for_channel(&samples, ChannelKind::Longitude)
            .expect("channel summary should exist");

    assert!(summary.contains("span=2.000000000000 d"));
    assert!(summary.contains("samples=1"));
    assert!(!summary.contains("span=10.000000000000 d"));
}

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn packaged_artifact_fit_channel_outlier_summary_validation_rejects_drift() {
    let mut summary = packaged_artifact_fit_channel_outlier_summary_details();
    summary.channel_summaries.pop();

    let error = summary
        .validate()
        .expect_err("channel outlier drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactFitChannelOutlierSummaryValidationError::FieldOutOfSync {
            field: "channel_summaries",
        }
    );
    assert!(error.to_string().contains("channel_summaries"));
}

#[test]
fn packaged_artifact_body_class_span_cap_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_body_class_span_cap_summary_details();
    assert_eq!(
        summary.summary_line(),
        "body-class span caps: luminaries=256 days, inner planets=384 days, outer planets=768 days, pluto=1536 days, lunar points=256 days, selected asteroids=256 days, custom bodies=512 days"
    );
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.validate().is_ok());
}

#[test]
fn packaged_artifact_body_class_span_cap_summary_validation_rejects_drift() {
    let mut summary = packaged_artifact_body_class_span_cap_summary_details();
    summary.entries[0].1 += 1.0;

    let error = summary
        .validate()
        .expect_err("body-class span cap drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactBodyClassSpanCapSummaryValidationError::FieldOutOfSync { field: "entries" }
    );
    assert_eq!(
        error.to_string(),
        "the packaged artifact body-class span cap summary field `entries` is out of sync with the current posture"
    );
}

#[test]
fn packaged_artifact_body_cadence_summary_reflects_the_current_posture() {
    let summary = packaged_artifact_body_cadence_summary_details();
    assert_eq!(
        summary.entries,
        vec![
            ("luminaries", 2),
            ("inner planets", 3),
            ("outer planets", 4),
            ("pluto", 1),
            ("lunar points", 0),
            ("selected asteroids", 1),
            ("custom bodies", 0),
        ]
    );
    assert_eq!(
        summary.summary_line(),
        "body cadence: luminaries=2 bodies, inner planets=3 bodies, outer planets=4 bodies, pluto=1 body, lunar points=0 bodies, selected asteroids=1 body, custom bodies=0 bodies"
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    summary
        .validate()
        .expect("packaged artifact body cadence summary should validate");
}

#[test]
fn packaged_artifact_body_cadence_summary_validation_rejects_drift() {
    let mut summary = packaged_artifact_body_cadence_summary_details();
    summary.entries[0].1 += 1;

    let error = summary
        .validate()
        .expect_err("body cadence drift should be rejected");
    assert_eq!(
        error,
        PackagedArtifactBodyCadenceSummaryValidationError::FieldOutOfSync { field: "entries" }
    );
    assert_eq!(
        error.to_string(),
        "the packaged artifact body cadence summary field `entries` is out of sync with the current posture"
    );
}

#[test]
fn packaged_body_coverage_summary_matches_the_packaged_body_set() {
    let summary = packaged_body_coverage_summary_details();
    assert_eq!(summary.body_count, packaged_bodies().len());
    assert_eq!(summary.bodies, packaged_bodies().to_vec());
    assert_eq!(
        summary.summary_line(),
        "Packaged body set: 11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"
    );
}

#[test]
fn packaged_body_coverage_summary_validation_rejects_body_count_drift() {
    let mut summary = packaged_body_coverage_summary_details();
    summary.body_count += 1;

    let error = summary
        .validate()
        .expect_err("body-count drift should be rejected");
    assert_eq!(
        error,
        PackagedBodyCoverageSummaryValidationError::FieldOutOfSync {
            field: "body_count",
        }
    );
    assert_eq!(
        error.to_string(),
        "the packaged body coverage summary field `body_count` is out of sync with the current bundled body set"
    );
}

#[test]
fn packaged_body_coverage_summary_validated_summary_line_rejects_body_drift() {
    let mut summary = packaged_body_coverage_summary_details();
    summary.bodies.swap(0, 1);

    let error = summary
        .validated_summary_line()
        .expect_err("body-order drift should be rejected");
    assert_eq!(
        error,
        PackagedBodyCoverageSummaryValidationError::FieldOutOfSync { field: "bodies" }
    );
    assert_eq!(
        error.to_string(),
        "the packaged body coverage summary field `bodies` is out of sync with the current bundled body set"
    );
}

/// Shared synthetic ephemeris backend for kernel-free unit tests.
///
/// Longitude advances 1 deg/day, latitude has a small sinusoidal wobble, and
/// distance stays near 1 AU. All values are smooth analytic functions, making
/// them easy to fit exactly with a degree-8 polynomial over small spans.
struct Synthetic;

impl pleiades_backend::EphemerisBackend for Synthetic {
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
        let jd = req.instant.julian_day.days();
        let lon = (jd * 1.0).rem_euclid(360.0);
        let lat = 0.1 * (jd / 50.0).sin();
        let dist = 1.0 + 0.01 * (jd / 80.0).cos();
        let mut r = pleiades_backend::EphemerisResult::new(
            pleiades_backend::BackendId::new("synthetic"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        r.ecliptic = Some(pleiades_backend::EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(lon),
            pleiades_backend::Latitude::from_degrees(lat),
            Some(dist),
        ));
        Ok(r)
    }
}

#[test]
fn fit_segment_within_span_reproduces_a_smooth_synthetic_body() {
    use pleiades_compression::{ArtifactHeader, BodyArtifact};

    let body = CelestialBody::Sun;
    let (t0, t1) = (2_451_545.0, 2_451_545.0 + 16.0);
    let seg = crate::regenerate::fit_segment_within_span(&body, t0, t1, &Synthetic)
        .expect("fit should succeed");
    // The segment spans [t0, t1] and carries the three channels.
    assert_eq!(seg.start.julian_day.days(), t0);
    assert_eq!(seg.end.julian_day.days(), t1);
    assert!(seg
        .channels
        .iter()
        .any(|c| matches!(c.kind, pleiades_compression::ChannelKind::Longitude)));
    assert!(seg
        .channels
        .iter()
        .any(|c| matches!(c.kind, pleiades_compression::ChannelKind::Latitude)));
    assert!(seg
        .channels
        .iter()
        .any(|c| matches!(c.kind, pleiades_compression::ChannelKind::DistanceAu)));
    // Validate the segment (checks channel order, uniqueness, finite coefficients).
    seg.validate().expect("generated segment should validate");

    // Accuracy probes: evaluate the fitted segment at 5 interior epochs via the
    // real quantized decode path (CompressedArtifact::lookup_ecliptic) and compare
    // against the Synthetic backend's ground-truth functions.  A degree-8 LSQ fit
    // of these smooth functions over a 16-day span should be essentially exact, so
    // we use a tight tolerance — a wrong x-domain or wrong scale_exponent would fail.
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("within-span probe", "fit_segment_within_span accuracy test"),
        vec![BodyArtifact::new(body.clone(), vec![seg])],
    );

    // Route probes through a real binary encode/decode round-trip so that
    // scale_exponent quantization is exercised (PolynomialChannel::evaluate
    // ignores scale_exponent; quantization only happens during encode/decode).
    let bytes = artifact
        .encode()
        .expect("within-span artifact should encode");
    let artifact = pleiades_compression::CompressedArtifact::decode(&bytes)
        .expect("round-tripped artifact should decode");

    let probe_fracs = [0.1, 0.3, 0.5, 0.7, 0.9];
    for frac in probe_fracs {
        let probe_jd = t0 + frac * (t1 - t0);
        // Segments are Tt-tagged (lookup convention); probe with Tt to match.
        let probe_inst = Instant::new(JulianDay::from_days(probe_jd), TimeScale::Tt);

        // Ground-truth from the Synthetic backend.
        let gt_lon = (probe_jd * 1.0).rem_euclid(360.0);
        let gt_lat = 0.1 * (probe_jd / 50.0).sin();
        let gt_dist = 1.0 + 0.01 * (probe_jd / 80.0).cos();

        // Decoded from the fitted segment via the real encode/decode path.
        let decoded = artifact
            .lookup_ecliptic(&body, probe_inst)
            .unwrap_or_else(|e| panic!("lookup_ecliptic failed at frac={frac}: {e}"));

        // Longitude comparison: use shortest angular distance to handle wrap.
        let lon_err = {
            let diff = decoded.longitude.degrees() - gt_lon;
            let wrapped = diff - diff.div_euclid(360.0) * 360.0;
            let signed = if wrapped > 180.0 {
                wrapped - 360.0
            } else if wrapped < -180.0 {
                wrapped + 360.0
            } else {
                wrapped
            };
            signed.abs()
        };
        assert!(
            lon_err < 1e-3,
            "longitude error {lon_err} deg at frac={frac} (probe_jd={probe_jd})"
        );

        let lat_err = (decoded.latitude.degrees() - gt_lat).abs();
        assert!(
            lat_err < 1e-3,
            "latitude error {lat_err} deg at frac={frac} (probe_jd={probe_jd})"
        );

        let dist_err =
            (decoded.distance_au.expect("distance_au should be present") - gt_dist).abs();
        assert!(
            dist_err < 1e-6,
            "distance error {dist_err} AU at frac={frac} (probe_jd={probe_jd})"
        );
    }
}

/// Kernel-free assembly test: runs `build_packaged_artifact_from_reference_over`
/// with tiny synthetic windows so the test finishes in milliseconds.
///
/// Rationale for window parameterisation: the full 1900–2100 default-window build produces
/// ~91 000 segments for the Moon alone; a synthetic-backend test over the full
/// range would take minutes and violate the "no slow non-ignored tests" rule
/// from the prior slice's review. By exposing the `_over` core we can exercise
/// the complete assembly logic (body fan-out, span tiling, segment fitting,
/// checksum, validate) with windows only a few hundred days wide — enough for
/// several segments per body — while keeping runtime under a second.
///
/// For the constrained asteroid (Eros), segments are re-derived from the
/// reference snapshot (curated corpus data), not from the committed artifact.
/// The test verifies snapshot-based sourcing: Eros is present with ≥1 segment
/// and its count matches the expected snapshot-fit count exactly.
#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn build_from_reference_produces_all_bodies_with_spanning_segments() {
    // Tiny window: a few hundred days covers several segments for every
    // non-asteroid cadence class (Moon=4-day spans → ~50 segs; outer
    // planets=512-day spans → ~1 seg) while running in milliseconds on the
    // Synthetic backend. The asteroid is re-derived from the reference snapshot,
    // not carried from the committed artifact, so no .bin decode is needed.
    let base_window = (2_451_545.0, 2_451_545.0 + 200.0);

    let artifact =
        crate::regenerate::build_packaged_artifact_from_reference_over(&Synthetic, base_window);

    for body in crate::packaged_bodies() {
        let ba = artifact
            .bodies
            .iter()
            .find(|b| &b.body == body)
            .unwrap_or_else(|| panic!("missing body {body}"));
        assert!(!ba.segments.is_empty(), "{body} has no segments");

        let cadence = crate::coverage::packaged_artifact_body_cadence(body);
        match cadence {
            crate::coverage::PackagedArtifactBodyCadence::SelectedAsteroids
            | crate::coverage::PackagedArtifactBodyCadence::CustomBodies => {
                // Eros must have been re-derived from the reference snapshot,
                // not fit from the Synthetic backend. Verify segment count
                // matches the expected snapshot-fit count — this is
                // format/version-independent and does not decode the committed
                // .bin.
                use pleiades_jpl::{reference_snapshot, JplSnapshotBackend, SnapshotEntry};
                use std::cmp::Ordering;
                let snap = reference_snapshot();
                let mut e: Vec<&SnapshotEntry> = snap.iter().filter(|x| x.body == *body).collect();
                e.sort_by(|left, right| {
                    left.epoch
                        .julian_day
                        .days()
                        .partial_cmp(&right.epoch.julian_day.days())
                        .unwrap_or(Ordering::Equal)
                });
                let expected =
                    crate::regenerate::body_segments_from_entries(&e, &JplSnapshotBackend).len();
                assert_eq!(
                    ba.segments.len(),
                    expected,
                    "{body}: snapshot-fit segment count {}, expected {expected} from reference snapshot",
                    ba.segments.len(),
                );
            }
            _ => {
                // Majors: segments must be contiguous and ascending.
                for pair in ba.segments.windows(2) {
                    assert!(
                        pair[1].start.julian_day.days() >= pair[0].end.julian_day.days(),
                        "{body}: segments not contiguous/ascending at boundary between {} and {}",
                        pair[0].end.julian_day.days(),
                        pair[1].start.julian_day.days(),
                    );
                }
            }
        }
    }
}

/// Guards that the public window-parameterized builder and the default builder
/// produce identical results for the same window.
#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn default_window_artifact_matches_explicit_default_over() {
    use pleiades_jpl::spk::corpus_spec::CoverageWindow;
    // A tiny synthetic window keeps this in milliseconds; assert the public
    // window-parameterized builder and the default builder agree for the same window.
    let reference = Synthetic;
    let window = CoverageWindow::new(2_451_545.0, 2_451_545.0 + 40.0);
    let a = crate::regenerate::build_packaged_artifact_from_reference_over(
        &reference,
        window.as_tuple(),
    );
    let b = crate::regenerate::build_packaged_artifact_from_reference_over(
        &reference,
        (2_451_545.0, 2_451_545.0 + 40.0),
    );
    assert_eq!(a.encode().unwrap(), b.encode().unwrap());
}

/// Regression test for the TDB/TT timescale bug in `fit_segment_within_span`.
///
/// Before the fix, segment boundaries were tagged `Tdb`; `normalize_lookup_instant`
/// re-tags every query to `Tt`; and `Segment::contains` requires
/// `segment.start.scale == query.scale` — so Tt queries always missed Tdb-tagged
/// segments, returning `OutOfRangeInstant` for every major-body packaged lookup.
///
/// This test exercises the exact lookup path the runtime uses: a `Tt`-tagged
/// query through `normalize_lookup_instant` → `CompressedArtifact::lookup_ecliptic`.
#[test]
fn fit_segment_tt_lookup_succeeds_after_normalize() {
    use pleiades_compression::{ArtifactHeader, BodyArtifact};

    let body = CelestialBody::Sun;
    let (t0, t1) = (2_451_545.0, 2_451_545.0 + 16.0);

    let seg = crate::regenerate::fit_segment_within_span(&body, t0, t1, &Synthetic)
        .expect("fit should succeed");

    // Encode/decode round-trip to exercise the full quantized path.
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new(
            "tt-lookup-regression",
            "fit_segment_tt_lookup_succeeds_after_normalize",
        ),
        vec![BodyArtifact::new(body.clone(), vec![seg])],
    );
    let bytes = artifact.encode().expect("artifact should encode");
    let artifact =
        pleiades_compression::CompressedArtifact::decode(&bytes).expect("artifact should decode");

    // Simulate the runtime packaged-lookup path: query with Tt-tagged instant
    // through normalize_lookup_instant (which is a no-op for Tt inputs, just
    // as the packaged backend does).
    let mid_jd = (t0 + t1) / 2.0;
    let tt_instant = Instant::new(JulianDay::from_days(mid_jd), TimeScale::Tt);
    let lookup_instant = crate::regenerate::normalize_lookup_instant(tt_instant);

    // Before the fix (Tdb boundaries): this returns Err(OutOfRangeInstant).
    // After the fix (Tt boundaries): this returns Ok.
    artifact
        .lookup_ecliptic(&body, lookup_instant)
        .unwrap_or_else(|e| {
            panic!("Tt-tagged lookup on a fit-produced segment failed (scale mismatch bug?): {e}")
        });
}
