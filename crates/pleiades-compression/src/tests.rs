use super::*;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

use crate::codec::{
    encode_artifact_profile, encode_celestial_body, encode_endian_policy, fnv1a64, write_string,
    write_u16, write_u32, write_u64, write_u8, Cursor,
};

#[test]
fn encode_decode_roundtrip_preserves_structure() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "unit test fixture"),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, -1.0, 1.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 1.0, 2.0),
                ],
            )],
        )],
    );

    let encoded = artifact.encode().expect("artifact should encode");
    let decoded = CompressedArtifact::decode(&encoded).expect("artifact should decode");
    assert_eq!(decoded.header.version, ARTIFACT_VERSION);
    assert_eq!(decoded.header.generation_label, "demo");
    assert_eq!(decoded.header.source, "unit test fixture");
    assert_eq!(decoded.header.endian_policy, EndianPolicy::LittleEndian);
    assert_eq!(
        decoded.header.profile,
        ArtifactProfile::packaged_ecliptic_longitude_latitude_distance_with_derived_equatorial()
    );
    assert_eq!(
        decoded.header.profile.stored_channels,
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu
        ]
    );
    assert_eq!(
        decoded.header.profile.speed_policy,
        SpeedPolicy::Unsupported
    );
    assert!(decoded
        .header
        .profile
        .derived_outputs
        .contains(&ArtifactOutput::EclipticCoordinates));
    assert!(decoded
        .header
        .profile
        .unsupported_outputs
        .contains(&ArtifactOutput::Motion));
    assert_eq!(decoded.bodies.len(), 1);
    assert_eq!(decoded.bodies[0].body, CelestialBody::Sun);
    assert_eq!(
        decoded.checksum,
        artifact.checksum().expect("checksum should compute")
    );
}

#[test]
fn decode_rejects_checksum_corruption() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "tamper check fixture"),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![PolynomialChannel::linear(
                    ChannelKind::Longitude,
                    9,
                    10.0,
                    11.0,
                )],
            )],
        )],
    );

    let mut encoded = artifact.encode().expect("artifact should encode");
    let last_index = encoded.len() - 1;
    encoded[last_index] ^= 0x01;

    let error = CompressedArtifact::decode(&encoded).expect_err("tampered artifact should fail");
    assert_eq!(error.kind, CompressionErrorKind::ChecksumMismatch);
}

#[test]
fn polynomial_channel_quadratic_interpolates_start_midpoint_and_end() {
    let channel = PolynomialChannel::quadratic(ChannelKind::Longitude, 9, 10.0, 16.0, 20.0, 0.5);

    assert!((channel.evaluate(0.0) - 10.0).abs() < 1e-12);
    assert!((channel.evaluate(0.5) - 16.0).abs() < 1e-12);
    assert!((channel.evaluate(1.0) - 20.0).abs() < 1e-12);
}

#[test]
fn explicit_profile_roundtrip_preserves_stored_derived_and_unsupported_outputs() {
    let profile = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        vec![ArtifactOutput::EclipticCoordinates, ArtifactOutput::Motion],
        vec![
            ArtifactOutput::EquatorialCoordinates,
            ArtifactOutput::ApparentCorrections,
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutput::SiderealCoordinates,
        ],
        SpeedPolicy::FittedDerivative,
    );
    let artifact = CompressedArtifact::new(
        ArtifactHeader::with_profile("profile demo", "unit test profile", profile.clone()),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            )],
        )],
    );

    let decoded = CompressedArtifact::decode(
        &artifact
            .encode()
            .expect("artifact should encode with profile"),
    )
    .expect("artifact should decode with profile");

    assert_eq!(decoded.header.profile, profile);
    assert_eq!(decoded.header.endian_policy, EndianPolicy::LittleEndian);
}

#[test]
fn artifact_profile_reports_output_support_statuses() {
    let profile = ArtifactProfile::ecliptic_longitude_latitude_distance();
    let explicit_profile =
        ArtifactProfile::packaged_ecliptic_longitude_latitude_distance_with_derived_equatorial();

    assert_eq!(profile, explicit_profile);
    assert_eq!(profile.summary_line(), explicit_profile.summary_line());
    assert_eq!(
        ArtifactProfile::packaged_ecliptic_longitude_latitude_distance_with_derived_equatorial(),
        explicit_profile
    );

    assert_eq!(
        profile.output_support(ArtifactOutput::EclipticCoordinates),
        ArtifactOutputSupport::Derived
    );
    assert_eq!(
        profile.output_support(ArtifactOutput::EquatorialCoordinates),
        ArtifactOutputSupport::Derived
    );
    assert_eq!(
        profile.output_support(ArtifactOutput::Motion),
        ArtifactOutputSupport::Unsupported
    );
    assert_eq!(
        profile.output_support(ArtifactOutput::SiderealCoordinates),
        ArtifactOutputSupport::Unsupported
    );
    assert_eq!(
        profile.speed_policy.motion_output_support(),
        ArtifactOutputSupport::Unsupported
    );
    assert_eq!(
        profile.motion_output_support(),
        ArtifactOutputSupport::Unsupported
    );
    assert!(profile.supports_output(ArtifactOutput::EclipticCoordinates));
    assert!(profile.supports_output(ArtifactOutput::EquatorialCoordinates));
    assert!(!profile.is_unsupported_output(ArtifactOutput::EquatorialCoordinates));
    assert!(profile.is_unsupported_output(ArtifactOutput::SiderealCoordinates));

    let unlisted_summary_profile = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        vec![ArtifactOutput::EclipticCoordinates],
        vec![
            ArtifactOutput::ApparentCorrections,
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutput::SiderealCoordinates,
            ArtifactOutput::Motion,
        ],
        SpeedPolicy::Unsupported,
    );
    let error = unlisted_summary_profile
        .validate()
        .expect_err("unlisted profile output should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert_eq!(
        error.message,
        "artifact profile output EquatorialCoordinates must be explicitly listed as stored, derived, approximated, or unsupported"
    );

    assert_eq!(
        SpeedPolicy::Stored.motion_output_support(),
        ArtifactOutputSupport::Stored
    );
    assert_eq!(
        SpeedPolicy::FittedDerivative.motion_output_support(),
        ArtifactOutputSupport::Derived
    );
    assert_eq!(
        SpeedPolicy::NumericalDifference.motion_output_support(),
        ArtifactOutputSupport::Approximated
    );

    let numerical_difference_profile = ArtifactProfile::new(
        vec![ChannelKind::Longitude],
        Vec::new(),
        Vec::new(),
        SpeedPolicy::NumericalDifference,
    );
    assert_eq!(
        numerical_difference_profile.output_support(ArtifactOutput::Motion),
        ArtifactOutputSupport::Approximated
    );
    assert_eq!(
        numerical_difference_profile.motion_output_support(),
        ArtifactOutputSupport::Approximated
    );
    assert!(numerical_difference_profile.supports_output(ArtifactOutput::Motion));
    assert!(!numerical_difference_profile.is_unsupported_output(ArtifactOutput::Motion));

    let unlisted_profile = ArtifactProfile::new(
        vec![ChannelKind::Longitude],
        Vec::new(),
        Vec::new(),
        SpeedPolicy::Unsupported,
    );
    assert_eq!(
        unlisted_profile.output_support(ArtifactOutput::Motion),
        ArtifactOutputSupport::Unsupported
    );
    assert_eq!(ArtifactOutputSupport::Approximated.label(), "approximated");
    assert_eq!(
        ArtifactOutputSupport::Approximated.to_string(),
        "approximated"
    );
    assert!(!unlisted_profile.supports_output(ArtifactOutput::Motion));
    assert!(unlisted_profile.is_unsupported_output(ArtifactOutput::Motion));
}

#[test]
fn artifact_residual_body_coverage_summary_tracks_artifact_residual_bodies() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("residual coverage demo", "unit test residual coverage"),
        vec![
            BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        10.0,
                        11.0,
                    )],
                )],
            ),
            BodyArtifact::new(
                CelestialBody::Moon,
                vec![Segment::with_residual_channels(
                    Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        20.0,
                        21.0,
                    )],
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        0.1,
                        0.2,
                    )],
                )],
            ),
        ],
    );

    let summary = artifact.residual_body_coverage_summary();
    assert_eq!(summary.body_count, 1);
    assert_eq!(summary.bodies, vec![CelestialBody::Moon]);
    assert_eq!(summary.summary_line(), "residual bodies: Moon");
    assert_eq!(
        summary
            .validated_summary_line(&artifact)
            .expect("residual body coverage summary should validate"),
        "residual bodies: Moon"
    );
    assert_eq!(
        summary.summary_line_with_body_count(),
        "residual bodies: Moon; applies to 1 bundled body"
    );
    assert_eq!(
        summary
            .validated_summary_line_with_body_count(&artifact)
            .expect("residual body coverage summary should validate"),
        "residual bodies: Moon; applies to 1 bundled body"
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    summary
        .validate(&artifact)
        .expect("residual body coverage should match the artifact");

    let mut drifted = summary.clone();
    drifted.body_count += 1;
    let count_error = drifted
        .validate(&artifact)
        .expect_err("drifted residual body coverage count should be rejected");
    assert_eq!(count_error.kind, CompressionErrorKind::InvalidFormat);
    assert!(format!("{count_error}").contains("body count does not match the body list"));
    let validated_error = drifted
        .validated_summary_line(&artifact)
        .expect_err("drifted residual body coverage validated line should be rejected");
    assert_eq!(validated_error.kind, CompressionErrorKind::InvalidFormat);

    let mut drifted = summary.clone();
    drifted.bodies = vec![CelestialBody::Sun];
    let error = drifted
        .validate(&artifact)
        .expect_err("drifted residual body coverage should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(format!("{error}").contains("residual-body coverage body list"));

    let duplicate_residual_segment = Segment::with_residual_channels(
        Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
        vec![PolynomialChannel::linear(
            ChannelKind::Longitude,
            9,
            20.0,
            21.0,
        )],
        vec![PolynomialChannel::linear(
            ChannelKind::Longitude,
            9,
            0.1,
            0.2,
        )],
    );
    let duplicate_residual_artifact = CompressedArtifact::new(
        ArtifactHeader::new(
            "duplicate residual coverage demo",
            "unit test duplicate residual coverage",
        ),
        vec![
            BodyArtifact::new(
                CelestialBody::Moon,
                vec![duplicate_residual_segment.clone()],
            ),
            BodyArtifact::new(CelestialBody::Moon, vec![duplicate_residual_segment]),
        ],
    );
    let duplicate_summary = duplicate_residual_artifact.residual_body_coverage_summary();
    let duplicate_error = duplicate_summary
        .validate(&duplicate_residual_artifact)
        .expect_err("duplicate residual body coverage should be rejected");
    assert_eq!(duplicate_error.kind, CompressionErrorKind::InvalidFormat);
    assert!(format!("{duplicate_error}")
        .contains("artifact residual-body coverage bodies contains duplicate Moon entry"));
}

#[test]
fn artifact_validation_helpers_reject_invalid_profiles_and_segments() {
    let invalid_profile = ArtifactProfile::new(
        vec![ChannelKind::Longitude, ChannelKind::Longitude],
        vec![ArtifactOutput::EclipticCoordinates],
        vec![ArtifactOutput::EquatorialCoordinates],
        SpeedPolicy::Unsupported,
    );
    let profile_error = invalid_profile
        .validate()
        .expect_err("duplicate profile entries should be rejected");
    assert_eq!(profile_error.kind, CompressionErrorKind::InvalidFormat);

    let out_of_order_stored_profile = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::DistanceAu,
            ChannelKind::Latitude,
        ],
        vec![ArtifactOutput::EclipticCoordinates],
        vec![ArtifactOutput::Motion],
        SpeedPolicy::Unsupported,
    );
    let out_of_order_stored_error = out_of_order_stored_profile
        .validate()
        .expect_err("stored channels should be ordered canonically");
    assert_eq!(
        out_of_order_stored_error.kind,
        CompressionErrorKind::InvalidFormat
    );
    assert!(format!("{out_of_order_stored_error}")
        .contains("artifact profile stored channels must be ordered by channel kind"));

    let out_of_order_output_profile = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        vec![
            ArtifactOutput::EquatorialCoordinates,
            ArtifactOutput::EclipticCoordinates,
        ],
        vec![ArtifactOutput::Motion],
        SpeedPolicy::Unsupported,
    );
    let out_of_order_output_error = out_of_order_output_profile
        .validate()
        .expect_err("derived outputs should be ordered canonically");
    assert_eq!(
        out_of_order_output_error.kind,
        CompressionErrorKind::InvalidFormat
    );
    assert!(format!("{out_of_order_output_error}")
        .contains("artifact profile derived outputs must be ordered by artifact output kind"));

    let stored_motion_profile = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        vec![ArtifactOutput::EclipticCoordinates],
        vec![
            ArtifactOutput::EquatorialCoordinates,
            ArtifactOutput::ApparentCorrections,
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutput::SiderealCoordinates,
        ],
        SpeedPolicy::Stored,
    );
    assert_eq!(
        stored_motion_profile.output_support(ArtifactOutput::Motion),
        ArtifactOutputSupport::Stored
    );
    assert!(stored_motion_profile.supports_output(ArtifactOutput::Motion));
    assert_eq!(stored_motion_profile.validate(), Ok(()));

    let motion_policy_mismatch = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        vec![ArtifactOutput::EclipticCoordinates, ArtifactOutput::Motion],
        Vec::new(),
        SpeedPolicy::Stored,
    );
    let motion_policy_error = motion_policy_mismatch
        .validate()
        .expect_err("stored motion support should keep Motion out of derived outputs");
    assert_eq!(
        motion_policy_error.kind,
        CompressionErrorKind::InvalidFormat
    );

    let unsupported_motion_mismatch = ArtifactProfile::new(
        vec![ChannelKind::Longitude],
        vec![ArtifactOutput::EclipticCoordinates],
        vec![
            ArtifactOutput::EquatorialCoordinates,
            ArtifactOutput::ApparentCorrections,
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutput::SiderealCoordinates,
        ],
        SpeedPolicy::Unsupported,
    );
    let unsupported_motion_error = unsupported_motion_mismatch
        .validate()
        .expect_err("unsupported motion policy should require Motion in unsupported outputs");
    assert_eq!(
        unsupported_motion_error.kind,
        CompressionErrorKind::InvalidFormat
    );

    let out_of_order_unsupported_profile = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        vec![ArtifactOutput::EclipticCoordinates],
        vec![ArtifactOutput::Motion, ArtifactOutput::SiderealCoordinates],
        SpeedPolicy::Unsupported,
    );
    let out_of_order_unsupported_error = out_of_order_unsupported_profile
        .validate()
        .expect_err("unsupported outputs should be ordered canonically");
    assert_eq!(
        out_of_order_unsupported_error.kind,
        CompressionErrorKind::InvalidFormat
    );
    assert!(format!("{out_of_order_unsupported_error}")
        .contains("artifact profile unsupported outputs must be ordered by artifact output kind"));

    let derived_coordinate_channel_mismatch = ArtifactProfile::new(
        vec![ChannelKind::Longitude, ChannelKind::Latitude],
        vec![
            ArtifactOutput::EclipticCoordinates,
            ArtifactOutput::EquatorialCoordinates,
        ],
        vec![
            ArtifactOutput::ApparentCorrections,
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutput::SiderealCoordinates,
            ArtifactOutput::Motion,
        ],
        SpeedPolicy::Unsupported,
    );
    let derived_coordinate_channel_error =
        derived_coordinate_channel_mismatch.validate().expect_err(
            "derived coordinate outputs should require longitude, latitude, and distance channels",
        );
    assert_eq!(
        derived_coordinate_channel_error.kind,
        CompressionErrorKind::InvalidFormat
    );
    assert!(format!("{derived_coordinate_channel_error}").contains(
        "derived coordinate outputs require Longitude, Latitude, and DistanceAu in stored channels"
    ));

    let derived_coordinate_channel_missing_latitude = ArtifactProfile::new(
        vec![ChannelKind::Longitude, ChannelKind::DistanceAu],
        vec![
            ArtifactOutput::EclipticCoordinates,
            ArtifactOutput::EquatorialCoordinates,
        ],
        vec![
            ArtifactOutput::ApparentCorrections,
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutput::SiderealCoordinates,
            ArtifactOutput::Motion,
        ],
        SpeedPolicy::Unsupported,
    );
    let derived_coordinate_channel_missing_latitude_error =
        derived_coordinate_channel_missing_latitude
            .validate()
            .expect_err("derived coordinate outputs should require latitude as well");
    assert_eq!(
        derived_coordinate_channel_missing_latitude_error.kind,
        CompressionErrorKind::InvalidFormat
    );
    assert!(format!("{derived_coordinate_channel_missing_latitude_error}").contains(
        "derived coordinate outputs require Longitude, Latitude, and DistanceAu in stored channels"
    ));

    let invalid_segment_body = BodyArtifact::new(
        CelestialBody::Moon,
        vec![Segment::new(
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 1.0),
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 2.0, 3.0),
                PolynomialChannel::linear(ChannelKind::Latitude, 9, 4.0, 5.0),
            ],
        )],
    );
    let segment_error = invalid_segment_body
        .validate()
        .expect_err("duplicate segment channels should be rejected");
    assert_eq!(segment_error.kind, CompressionErrorKind::InvalidFormat);

    let invalid_artifact = CompressedArtifact::new(
        ArtifactHeader::new("duplicate body demo", "unit test duplicate body validation"),
        vec![
            BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        10.0,
                        11.0,
                    )],
                )],
            ),
            BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        12.0,
                        13.0,
                    )],
                )],
            ),
        ],
    );
    let artifact_error = invalid_artifact
        .validate()
        .expect_err("duplicate bodies should be rejected");
    assert_eq!(artifact_error.kind, CompressionErrorKind::InvalidFormat);

    let blank_header = ArtifactHeader::with_profile(
        "  ",
        "unit test blank header",
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
    );
    let blank_header_error = blank_header
        .validate()
        .expect_err("blank generation labels should be rejected");
    assert_eq!(blank_header_error.kind, CompressionErrorKind::InvalidFormat);

    let blank_source_header = ArtifactHeader::with_profile(
        "blank source demo",
        "",
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
    );
    let blank_source_error = blank_source_header
        .validate()
        .expect_err("blank sources should be rejected");
    assert_eq!(blank_source_error.kind, CompressionErrorKind::InvalidFormat);

    let padded_header = ArtifactHeader::with_profile(
        " padded demo ",
        "unit test padded header",
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
    );
    let padded_header_error = padded_header
        .validate()
        .expect_err("padded generation labels should be rejected");
    assert_eq!(
        padded_header_error.kind,
        CompressionErrorKind::InvalidFormat
    );

    let padded_source_header = ArtifactHeader::with_profile(
        "padded source demo",
        " unit test padded source ",
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
    );
    let padded_source_error = padded_source_header
        .validate()
        .expect_err("padded sources should be rejected");
    assert_eq!(
        padded_source_error.kind,
        CompressionErrorKind::InvalidFormat
    );

    let blank_header_artifact = CompressedArtifact::new(
        ArtifactHeader::with_profile(
            " ",
            "unit test blank header artifact",
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        ),
        Vec::new(),
    );
    let blank_header_artifact_error = blank_header_artifact
        .validate()
        .expect_err("artifact validation should reject blank header metadata");
    assert_eq!(
        blank_header_artifact_error.kind,
        CompressionErrorKind::InvalidFormat
    );

    let blank_header_encode_error = CompressedArtifact::new(
        ArtifactHeader::with_profile(
            " ",
            "unit test blank header encode",
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        ),
        Vec::new(),
    )
    .encode()
    .expect_err("artifact encoding should reject blank header metadata");
    assert_eq!(
        blank_header_encode_error.kind,
        CompressionErrorKind::InvalidFormat
    );
}

#[test]
fn artifact_encoding_rejects_duplicate_profile_entries() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::with_profile(
            "duplicate profile demo",
            "unit test duplicate profile entries",
            ArtifactProfile::new(
                vec![ChannelKind::Longitude, ChannelKind::Longitude],
                vec![ArtifactOutput::EclipticCoordinates],
                vec![ArtifactOutput::EquatorialCoordinates],
                SpeedPolicy::Unsupported,
            ),
        ),
        Vec::new(),
    );

    let error = artifact
        .encode()
        .expect_err("duplicate profile entries should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
}

#[test]
fn artifact_encoding_rejects_duplicate_segment_channels() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new(
            "duplicate segment demo",
            "unit test duplicate segment channels",
        ),
        vec![BodyArtifact::new(
            CelestialBody::Moon,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 1.0),
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 2.0, 3.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 4.0, 5.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            )],
        )],
    );

    let error = artifact
        .encode()
        .expect_err("duplicate segment channels should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
}

#[test]
fn artifact_encoding_rejects_mismatched_segment_scales() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new(
            "mismatched segment scales demo",
            "unit test mismatched segment scales",
        ),
        vec![BodyArtifact::new(
            CelestialBody::Moon,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 1.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 2.0, 3.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            )],
        )],
    );

    let error = artifact
        .encode()
        .expect_err("mismatched segment scales should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
}

#[test]
fn artifact_decoding_rejects_duplicate_body_entries() {
    let profile = ArtifactProfile::ecliptic_longitude_latitude_distance();
    let mut payload = Vec::new();
    write_string(&mut payload, "duplicate body decode demo");
    write_string(&mut payload, "unit test duplicate body decode");
    write_u8(
        &mut payload,
        encode_endian_policy(EndianPolicy::LittleEndian),
    );
    encode_artifact_profile(&mut payload, &profile).expect("profile should encode");
    write_u16(&mut payload, 2);
    encode_celestial_body(&mut payload, &CelestialBody::Sun).expect("Sun should encode");
    write_u32(&mut payload, 0);
    encode_celestial_body(&mut payload, &CelestialBody::Sun).expect("Sun should encode");
    write_u32(&mut payload, 0);

    let checksum = fnv1a64(&payload);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&ARTIFACT_MAGIC);
    write_u16(&mut bytes, ARTIFACT_VERSION);
    write_u64(&mut bytes, checksum);
    bytes.extend_from_slice(&payload);

    let error =
        CompressedArtifact::decode(&bytes).expect_err("duplicate body entries should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
}

#[test]
fn residual_channels_are_applied_during_lookup() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("residual demo", "unit test residual channels"),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::with_residual_channels(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0),
                    PolynomialChannel::new(ChannelKind::Latitude, 9, vec![1.0]),
                    PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![2.0]),
                ],
                vec![
                    PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.25]),
                    PolynomialChannel::new(ChannelKind::Latitude, 9, vec![0.0]),
                    PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![-0.1]),
                ],
            )],
        )],
    );

    let lookup = artifact
        .lookup_ecliptic(
            &CelestialBody::Sun,
            Instant::new(JulianDay::from_days(0.5), TimeScale::Tt),
        )
        .expect("residual-corrected lookup should succeed");

    assert!((lookup.longitude.degrees() - 10.75).abs() < 1e-12);
    assert!((lookup.latitude.degrees() - 1.0).abs() < 1e-12);
    assert!((lookup.distance_au.unwrap() - 1.9).abs() < 1e-12);
}

#[test]
fn residual_channels_roundtrip_through_the_codec() {
    let segment = Segment::with_residual_channels(
        Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
        vec![
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![-0.25]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![0.125]),
        ],
    );
    segment
        .validate()
        .expect("segment metadata should validate");
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("residual roundtrip demo", "unit test residual roundtrip"),
        vec![BodyArtifact::new(CelestialBody::Sun, vec![segment.clone()])],
    );

    let decoded = CompressedArtifact::decode(
        &artifact
            .encode()
            .expect("artifact with residual channels should encode"),
    )
    .expect("artifact with residual channels should decode");

    assert_eq!(decoded.bodies[0].segments[0], segment);
}

#[test]
fn compressed_artifact_summary_line_reports_residual_segments() {
    let residual_segment = Segment::with_residual_channels(
        Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
        vec![
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![-0.25]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![0.125]),
        ],
    );
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("residual summary demo", "unit test residual summary"),
        vec![
            BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(
                        ChannelKind::Longitude,
                        9,
                        1.0,
                        2.0,
                    )],
                )],
            ),
            BodyArtifact::new(CelestialBody::Moon, vec![residual_segment]),
        ],
    );

    assert_eq!(artifact.segment_count(), 2);
    assert_eq!(artifact.residual_segment_count(), 1);
    assert_eq!(artifact.residual_bodies(), vec![CelestialBody::Moon]);
    assert_eq!(
        artifact.summary_line(),
        "bodies: 2; segments: 2; residual-bearing segments: 1; residual-bearing bodies: Moon"
    );
    assert_eq!(artifact.to_string(), artifact.summary_line());
}

#[test]
fn segment_summary_line_reports_stored_and_residual_channels() {
    let segment = Segment::with_residual_channels(
        Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
        vec![
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![-0.25]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![0.125]),
        ],
    );

    let expected = format!(
        "start: {}; end: {}; stored channels: [Longitude, Latitude, DistanceAu]; residual channels: [Longitude, Latitude, DistanceAu]",
        segment.start, segment.end
    );
    assert_eq!(segment.summary_line(), expected);
    assert_eq!(segment.to_string(), expected);
}

#[test]
fn body_artifact_summary_line_reports_body_and_residual_segments() {
    let body_artifact = BodyArtifact::new(
        CelestialBody::Sun,
        vec![
            Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0),
                    PolynomialChannel::new(ChannelKind::Latitude, 9, vec![1.0]),
                    PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![2.0]),
                ],
            ),
            Segment::with_residual_channels(
                Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
                    PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
                    PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
                ],
                vec![PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5])],
            ),
        ],
    );

    let expected = "body: Sun; segments: 2; residual-bearing segments: 1";
    assert_eq!(body_artifact.summary_line(), expected);
    assert_eq!(body_artifact.to_string(), expected);
}

#[test]
fn segment_validate_rejects_duplicate_residual_channels() {
    let segment = Segment::with_residual_channels(
        Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
        vec![
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![-0.25]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![0.125]),
        ],
    );

    let error = segment
        .validate()
        .expect_err("duplicate residual channels should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error.to_string().contains("segment residual channels"));
}

#[test]
fn segment_validate_rejects_unsorted_stored_channels() {
    let segment = Segment::new(
        Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
    );

    let error = segment
        .validate()
        .expect_err("stored channels should be ordered canonically by channel kind");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .to_string()
        .contains("segment stored channels must be ordered by channel kind"));
}

#[test]
fn segment_validate_rejects_residual_channels_without_matching_stored_channels() {
    let segment = Segment::with_residual_channels(
        Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
        vec![
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.5]),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![-0.25]),
        ],
    );

    let error = segment
        .validate()
        .expect_err("residual channels should require matching stored channels");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .to_string()
        .contains("segment residual channels require a matching stored channel"));
}

#[test]
fn segment_validate_rejects_non_finite_bounds() {
    let segment = Segment::new(
        Instant::new(JulianDay::from_days(f64::INFINITY), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![3.0]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
    );

    let error = segment
        .validate()
        .expect_err("non-finite segment bounds should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error.message.contains("segment start must be finite"));
}

#[test]
fn segment_validate_rejects_non_finite_channel_coefficients() {
    let segment = Segment::new(
        Instant::new(JulianDay::from_days(10.0), TimeScale::Tt),
        Instant::new(JulianDay::from_days(11.0), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 22.0),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![f64::NAN]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 12, vec![4.0]),
        ],
    );

    let error = segment
        .validate()
        .expect_err("non-finite channel coefficients should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .message
        .contains("polynomial channel Latitude contains a non-finite coefficient at index 0"));
}

#[test]
fn body_validate_rejects_overlapping_segments_for_the_same_scale() {
    let body = BodyArtifact::new(
        CelestialBody::Moon,
        vec![
            Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 20.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            ),
            Segment::new(
                Instant::new(JulianDay::from_days(1.5), TimeScale::Tt),
                Instant::new(JulianDay::from_days(3.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 30.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.3, 0.4),
                ],
            ),
        ],
    );

    let error = body
        .validate()
        .expect_err("overlapping same-scale segments should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error.message.contains("ordered and non-overlapping"));
}

#[test]
fn body_validate_allows_shared_boundary_segments_for_the_same_scale() {
    let body = BodyArtifact::new(
        CelestialBody::Moon,
        vec![
            Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 10.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            ),
            Segment::new(
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 30.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.3, 0.4),
                ],
            ),
        ],
    );

    assert!(body.validate().is_ok());
}

#[test]
fn explicit_endian_policy_roundtrip_preserves_header_metadata() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::with_profile_and_endian(
            "endian demo",
            "unit test endian policy",
            EndianPolicy::LittleEndian,
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        ),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            )],
        )],
    );

    let decoded = CompressedArtifact::decode(
        &artifact
            .encode()
            .expect("artifact should encode with explicit endian policy"),
    )
    .expect("artifact should decode with explicit endian policy");

    assert_eq!(decoded.header.endian_policy, EndianPolicy::LittleEndian);
    assert_eq!(decoded.header.generation_label, "endian demo");
}

#[test]
fn decode_rejects_unsupported_endian_policy() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::with_profile_and_endian(
            "endian demo",
            "unit test endian policy",
            EndianPolicy::LittleEndian,
            ArtifactProfile::ecliptic_longitude_latitude_distance(),
        ),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            )],
        )],
    );

    let mut bytes = artifact
        .encode()
        .expect("artifact should encode with explicit endian policy");
    let mut cursor = Cursor::new(&bytes);
    assert_eq!(
        cursor.read_array::<8>().expect("magic should decode"),
        ARTIFACT_MAGIC
    );
    assert_eq!(
        cursor.read_u16().expect("version should decode"),
        ARTIFACT_VERSION
    );
    let _ = cursor.read_u64().expect("checksum should decode");
    let _ = cursor.read_string().expect("label should decode");
    let _ = cursor.read_string().expect("source should decode");
    let endian_index = cursor.offset;
    bytes[endian_index] = 1;
    let payload_offset =
        ARTIFACT_MAGIC.len() + std::mem::size_of::<u16>() + std::mem::size_of::<u64>();
    let checksum = fnv1a64(&bytes[payload_offset..]);
    let checksum_offset = ARTIFACT_MAGIC.len() + std::mem::size_of::<u16>();
    bytes[checksum_offset..checksum_offset + std::mem::size_of::<u64>()]
        .copy_from_slice(&checksum.to_le_bytes());

    let error = CompressedArtifact::decode(&bytes)
        .expect_err("artifact should reject unsupported endian policy");
    assert_eq!(error.kind, CompressionErrorKind::UnsupportedEndianPolicy);
    assert_eq!(
        error.message,
        "artifact byte-order policy 1 is not supported"
    );
}

#[test]
fn artifact_profile_summary_lists_capability_fields() {
    let profile = ArtifactProfile::new(
        vec![
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        vec![
            ArtifactOutput::EclipticCoordinates,
            ArtifactOutput::EquatorialCoordinates,
        ],
        vec![
            ArtifactOutput::ApparentCorrections,
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutput::SiderealCoordinates,
            ArtifactOutput::Motion,
        ],
        SpeedPolicy::Unsupported,
    );
    let coverage = ArtifactProfileCoverageSummary::new(
        profile.clone(),
        vec![CelestialBody::Sun, CelestialBody::Moon],
    );

    assert_eq!(
        profile.summary(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
    );
    assert_eq!(
        profile.summary_line(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
    );
    assert_eq!(
        profile.summary_for_body_count(11),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies"
    );
    assert_eq!(
        profile.summary_line_with_body_count(11),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies"
    );
    assert_eq!(
        profile.to_string(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
    );
    assert_eq!(
        profile.output_support_entries_summary_line(),
        "EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported"
    );
    assert_eq!(
        profile.output_support_summary_line(),
        "EclipticCoordinates=derived, EquatorialCoordinates=derived, ApparentCorrections=unsupported, TopocentricCoordinates=unsupported, SiderealCoordinates=unsupported, Motion=unsupported; unlisted outputs: []; support counts: stored=0, derived=2, approximated=0, unsupported=4, unlisted=0"
    );
    assert_eq!(
        profile
            .validated_summary_line()
            .expect("profile summary should validate"),
        profile.summary_line()
    );
    assert_eq!(
        profile
            .validated_output_support_entries_summary_line()
            .expect("output-support entries should validate"),
        profile.output_support_entries_summary_line()
    );
    assert_eq!(
        profile
            .validated_output_support_summary_line()
            .expect("output-support summary should validate"),
        profile.output_support_summary_line()
    );
    assert_eq!(coverage.body_count, 2);
    assert_eq!(
        coverage.bodies,
        vec![CelestialBody::Sun, CelestialBody::Moon]
    );
    assert_eq!(coverage.profile, profile);
    assert_eq!(
        coverage.summary_line(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies"
    );
    assert_eq!(
        coverage
            .validated_summary_line()
            .expect("coverage summary should validate"),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies"
    );
    assert_eq!(
        coverage.summary_line_with_bodies(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies; bundled bodies: Sun, Moon"
    );
    assert_eq!(
        coverage
            .validated_summary_line_with_bodies()
            .expect("coverage summary should validate"),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies; bundled bodies: Sun, Moon"
    );
    assert_eq!(
        coverage.to_string(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies"
    );
    coverage
        .validate()
        .expect("coverage summary should validate");

    let header = ArtifactHeader::with_profile_and_endian(
        "demo",
        "source",
        EndianPolicy::LittleEndian,
        profile,
    );
    assert_eq!(
        header.summary(),
        "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
    );
    assert_eq!(
        header.summary_line(),
        "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
    );
    assert_eq!(
        header.summary_for_body_count(11),
        "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 11 bundled bodies"
    );
    assert_eq!(
        header.to_string(),
        "byte order: little-endian; stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
    );
}

#[test]
fn artifact_profile_coverage_validation_rejects_body_count_drift() {
    let mut coverage = ArtifactProfileCoverageSummary::new(
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
        vec![CelestialBody::Sun, CelestialBody::Moon],
    );
    coverage.body_count += 1;

    let error = coverage
        .validate()
        .expect_err("body-count drift should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .message
        .contains("artifact profile coverage body count does not match bundled body list"));
}

#[test]
fn artifact_profile_coverage_summary_line_uses_bundled_body_list() {
    let mut coverage = ArtifactProfileCoverageSummary::new(
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
        vec![CelestialBody::Sun, CelestialBody::Moon],
    );
    coverage.body_count += 1;

    assert_eq!(
        coverage.summary_line(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies"
    );
    assert_eq!(
        coverage.summary_line_with_bodies(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies; bundled bodies: Sun, Moon"
    );
    assert_eq!(
        coverage.to_string(),
        "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported; applies to 2 bundled bodies"
    );
}

#[test]
fn artifact_profile_coverage_validation_rejects_empty_bodies() {
    let coverage = ArtifactProfileCoverageSummary::new(
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
        Vec::new(),
    );

    let error = coverage
        .validate()
        .expect_err("empty bundled-body lists should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .message
        .contains("artifact profile coverage bundled body list must not be empty"));
}

#[test]
fn artifact_profile_coverage_validated_summary_line_rejects_drift() {
    let mut coverage = ArtifactProfileCoverageSummary::new(
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
        vec![CelestialBody::Sun, CelestialBody::Moon],
    );
    coverage.body_count += 1;

    let error = coverage
        .validated_summary_line_with_bodies()
        .expect_err("drifted coverage summaries should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .message
        .contains("artifact profile coverage body count does not match bundled body list"));
}

#[test]
fn artifact_profile_coverage_validation_rejects_duplicate_bodies() {
    let coverage = ArtifactProfileCoverageSummary::new(
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
        vec![CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Sun],
    );

    let error = coverage
        .validate()
        .expect_err("duplicate bundled bodies should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .message
        .contains("artifact profile coverage bundled bodies contains duplicate Sun entry"));
}

#[test]
fn artifact_profile_coverage_validation_rejects_out_of_order_bodies() {
    let coverage = ArtifactProfileCoverageSummary::new(
        ArtifactProfile::ecliptic_longitude_latitude_distance(),
        vec![CelestialBody::Moon, CelestialBody::Sun],
    );

    let error = coverage
        .validate()
        .expect_err("out-of-order bundled bodies should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error.message.contains(
        "artifact profile coverage bundled bodies must be ordered canonically; found Moon before Sun"
    ));
}

#[test]
fn artifact_profile_labels_are_stable() {
    assert_eq!(ChannelKind::Longitude.label(), "Longitude");
    assert_eq!(ChannelKind::Latitude.label(), "Latitude");
    assert_eq!(ChannelKind::DistanceAu.label(), "DistanceAu");

    assert_eq!(
        ArtifactOutput::EclipticCoordinates.label(),
        "EclipticCoordinates"
    );
    assert_eq!(
        ArtifactOutput::EquatorialCoordinates.label(),
        "EquatorialCoordinates"
    );
    assert_eq!(
        ArtifactOutput::ApparentCorrections.label(),
        "ApparentCorrections"
    );
    assert_eq!(
        ArtifactOutput::TopocentricCoordinates.label(),
        "TopocentricCoordinates"
    );
    assert_eq!(
        ArtifactOutput::SiderealCoordinates.label(),
        "SiderealCoordinates"
    );
    assert_eq!(ArtifactOutput::Motion.label(), "Motion");

    assert_eq!(SpeedPolicy::Unsupported.label(), "Unsupported");
    assert_eq!(SpeedPolicy::Stored.label(), "Stored");
    assert_eq!(SpeedPolicy::FittedDerivative.label(), "FittedDerivative");
    assert_eq!(
        SpeedPolicy::NumericalDifference.label(),
        "NumericalDifference"
    );

    assert_eq!(ChannelKind::Longitude.to_string(), "Longitude");
    assert_eq!(ArtifactOutput::Motion.to_string(), "Motion");
    assert_eq!(SpeedPolicy::Stored.to_string(), "Stored");
    assert_eq!(
        join_display(&[CelestialBody::Sun, CelestialBody::Moon]),
        "Sun, Moon"
    );
}

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip_preserves_artifacts() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("serde demo", "json roundtrip fixture"),
        vec![BodyArtifact::new(
            CelestialBody::Moon,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![PolynomialChannel::linear(
                    ChannelKind::Longitude,
                    9,
                    15.0,
                    30.0,
                )],
            )],
        )],
    );

    let decoded: CompressedArtifact =
        serde_json::from_value(serde_json::to_value(&artifact).expect("artifact should serialize"))
            .expect("artifact should deserialize");
    assert_eq!(decoded, artifact);
}

#[test]
fn lookup_interpolates_segment_channels() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "lookup fixture"),
        vec![BodyArtifact::new(
            CelestialBody::Moon,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 20.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                ],
            )],
        )],
    );

    let result = artifact
        .lookup_ecliptic(
            &CelestialBody::Moon,
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
        )
        .expect("lookup should work");

    assert_eq!(result.longitude.degrees(), 10.0);
    assert_eq!(result.latitude.degrees(), 2.0);
    assert_eq!(result.distance_au, Some(0.15000000000000002));
}

#[test]
fn lookup_rejects_missing_body() {
    let artifact = CompressedArtifact::new(ArtifactHeader::new("demo", "x"), Vec::new());
    let error = artifact
        .lookup_ecliptic(
            &CelestialBody::Sun,
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
        )
        .expect_err("missing bodies should error");
    assert_eq!(error.kind, CompressionErrorKind::MissingBody);
}

#[test]
fn compressed_artifact_profile_coverage_summary_tracks_bundled_bodies() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "coverage fixture"),
        vec![
            BodyArtifact::new(CelestialBody::Sun, Vec::new()),
            BodyArtifact::new(CelestialBody::Moon, Vec::new()),
        ],
    );

    let coverage = artifact.profile_coverage_summary();
    assert_eq!(coverage.body_count, 2);
    assert_eq!(
        coverage.bodies,
        vec![CelestialBody::Sun, CelestialBody::Moon]
    );
    assert_eq!(coverage.profile, artifact.header.profile);
    assert_eq!(coverage.validate(), Ok(()));
    assert_eq!(
        coverage.summary_line_with_bodies(),
        format!(
            "{}; bundled bodies: Sun, Moon",
            artifact.header.profile.summary_for_body_count(2)
        )
    );
}

#[test]
fn compressed_artifact_validate_rejects_empty_body_set() {
    let artifact = CompressedArtifact::new(ArtifactHeader::new("demo", "empty"), Vec::new());
    let error = artifact
        .validate()
        .expect_err("empty body sets should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .to_string()
        .contains("artifact profile coverage bundled body list must not be empty"));
}

#[test]
fn compressed_artifact_validate_rejects_noncanonical_body_order() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "unordered bodies"),
        vec![
            BodyArtifact::new(CelestialBody::Moon, Vec::new()),
            BodyArtifact::new(CelestialBody::Sun, Vec::new()),
        ],
    );

    let error = artifact
        .validate()
        .expect_err("non-canonical body ordering should be rejected");
    assert_eq!(error.kind, CompressionErrorKind::InvalidFormat);
    assert!(error
        .to_string()
        .contains("compressed artifact body entries must be ordered canonically"));
}

#[test]
fn random_access_helpers_return_body_and_segment_matches() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "segment access fixture"),
        vec![BodyArtifact::new(
            CelestialBody::Moon,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![PolynomialChannel::linear(
                    ChannelKind::Longitude,
                    9,
                    0.0,
                    20.0,
                )],
            )],
        )],
    );

    let body = artifact
        .body_artifact(&CelestialBody::Moon)
        .expect("body lookup should work");
    assert_eq!(body.body, CelestialBody::Moon);
    assert_eq!(body.segments.len(), 1);

    let segment = artifact
        .segment_for(
            &CelestialBody::Moon,
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
        )
        .expect("segment lookup should work");
    assert_eq!(segment.start.julian_day.days(), 0.0);
    assert_eq!(segment.end.julian_day.days(), 2.0);
    assert!(body
        .segment_at(Instant::new(JulianDay::from_days(0.0), TimeScale::Tt))
        .is_some());
    assert!(body
        .segment_at(Instant::new(JulianDay::from_days(2.0), TimeScale::Tt))
        .is_some());
    assert!(body
        .segment_at(Instant::new(JulianDay::from_days(2.1), TimeScale::Tt))
        .is_none());

    let error = artifact
        .segment_for(
            &CelestialBody::Moon,
            Instant::new(JulianDay::from_days(2.1), TimeScale::Tt),
        )
        .expect_err("out-of-range instant should error");
    assert_eq!(error.kind, CompressionErrorKind::OutOfRangeInstant);
}

#[test]
fn random_access_helpers_prefer_the_later_segment_on_shared_boundaries() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "boundary fixture"),
        vec![BodyArtifact::new(
            CelestialBody::Moon,
            vec![
                Segment::new(
                    Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 10.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 2.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.1, 0.2),
                    ],
                ),
                Segment::new(
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                    vec![
                        PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 30.0),
                        PolynomialChannel::linear(ChannelKind::Latitude, 9, 3.0, 4.0),
                        PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.3, 0.4),
                    ],
                ),
            ],
        )],
    );

    let shared_boundary = Instant::new(JulianDay::from_days(1.0), TimeScale::Tt);
    let segment = artifact
        .segment_for(&CelestialBody::Moon, shared_boundary)
        .expect("shared boundary should resolve to the later segment");
    assert_eq!(segment.start.julian_day.days(), 1.0);
    assert_eq!(segment.end.julian_day.days(), 2.0);

    let ecliptic = artifact
        .lookup_ecliptic(&CelestialBody::Moon, shared_boundary)
        .expect("boundary lookup should succeed");
    assert_eq!(ecliptic.longitude.degrees(), 20.0);
    assert_eq!(ecliptic.latitude.degrees(), 3.0);
    assert_eq!(ecliptic.distance_au, Some(0.3));
}

#[test]
fn encode_decode_roundtrip_preserves_lunar_apogee_and_perigee_bodies() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "lunar point fixture"),
        vec![
            BodyArtifact::new(CelestialBody::MeanApogee, Vec::new()),
            BodyArtifact::new(CelestialBody::TrueApogee, Vec::new()),
            BodyArtifact::new(CelestialBody::MeanPerigee, Vec::new()),
            BodyArtifact::new(CelestialBody::TruePerigee, Vec::new()),
        ],
    );

    let encoded = artifact.encode().expect("artifact should encode");
    let decoded = CompressedArtifact::decode(&encoded).expect("artifact should decode");

    assert_eq!(decoded.bodies.len(), 4);
    assert_eq!(decoded.bodies[0].body, CelestialBody::MeanApogee);
    assert_eq!(decoded.bodies[1].body, CelestialBody::TrueApogee);
    assert_eq!(decoded.bodies[2].body, CelestialBody::MeanPerigee);
    assert_eq!(decoded.bodies[3].body, CelestialBody::TruePerigee);
}

#[test]
fn lookup_ecliptic_requires_the_profile_to_advertise_it() {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::with_profile(
            "demo",
            "ecliptic lookup fixture",
            ArtifactProfile::new(
                vec![
                    ChannelKind::Longitude,
                    ChannelKind::Latitude,
                    ChannelKind::DistanceAu,
                ],
                vec![ArtifactOutput::EquatorialCoordinates],
                vec![
                    ArtifactOutput::ApparentCorrections,
                    ArtifactOutput::TopocentricCoordinates,
                    ArtifactOutput::SiderealCoordinates,
                    ArtifactOutput::Motion,
                ],
                SpeedPolicy::Unsupported,
            ),
        ),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.5, 0.75),
                ],
            )],
        )],
    );

    let instant = Instant::new(JulianDay::from_days(1.0), TimeScale::Tt);
    let error = artifact
        .lookup_ecliptic(&CelestialBody::Sun, instant)
        .expect_err("ecliptic lookup should respect the advertised profile");

    assert_eq!(
        error.message,
        "artifact profile does not support EclipticCoordinates"
    );
}

#[test]
fn lookup_equatorial_requires_the_profile_to_advertise_it() {
    use pleiades_types::Angle;

    let artifact = CompressedArtifact::new(
        ArtifactHeader::with_profile(
            "demo",
            "equatorial lookup fixture",
            ArtifactProfile::new(
                vec![
                    ChannelKind::Longitude,
                    ChannelKind::Latitude,
                    ChannelKind::DistanceAu,
                ],
                vec![ArtifactOutput::EclipticCoordinates],
                vec![
                    ArtifactOutput::ApparentCorrections,
                    ArtifactOutput::TopocentricCoordinates,
                    ArtifactOutput::SiderealCoordinates,
                    ArtifactOutput::Motion,
                ],
                SpeedPolicy::Unsupported,
            ),
        ),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.5, 0.75),
                ],
            )],
        )],
    );

    let instant = Instant::new(JulianDay::from_days(1.0), TimeScale::Tt);
    let obliquity = Angle::from_degrees(23.439_291_11);
    let error = artifact
        .lookup_equatorial(&CelestialBody::Sun, instant, obliquity)
        .expect_err("equatorial lookup should respect the advertised profile");

    assert_eq!(
        error.message,
        "artifact profile does not support EquatorialCoordinates"
    );
}

#[test]
fn lookup_equatorial_reconstructs_derived_coordinates() {
    use pleiades_types::Angle;

    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("demo", "equatorial lookup fixture"),
        vec![BodyArtifact::new(
            CelestialBody::Sun,
            vec![Segment::new(
                Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                Instant::new(JulianDay::from_days(2.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 20.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 1.0, 3.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 0.5, 0.75),
                ],
            )],
        )],
    );

    let instant = Instant::new(JulianDay::from_days(1.0), TimeScale::Tt);
    let obliquity = Angle::from_degrees(23.439_291_11);
    let ecliptic = artifact
        .lookup_ecliptic(&CelestialBody::Sun, instant)
        .expect("ecliptic lookup should succeed");
    let equatorial = artifact
        .lookup_equatorial(&CelestialBody::Sun, instant, obliquity)
        .expect("equatorial lookup should succeed");

    assert_eq!(equatorial, ecliptic.to_equatorial(obliquity));
    assert_eq!(equatorial.distance_au, Some(0.625));
}

#[test]
fn segment_count_exceeding_u16_max_round_trips_correctly() {
    // Regression test: per-body segment count was encoded as u16 (max 65,535).
    // The Moon in a dense de440 artifact can have ~91,311 segments, which
    // truncated to a wrong value and corrupted the byte stream on decode.
    // This test builds a body with 70,000 non-overlapping segments to prove
    // the u32 fix works. Each segment spans 1 day; segments are consecutive.
    const SEGMENT_COUNT: usize = 70_000;
    let segments = (0..SEGMENT_COUNT)
        .map(|i| {
            let i = i as f64;
            Segment::new(
                Instant::new(JulianDay::from_days(i), TimeScale::Tt),
                Instant::new(JulianDay::from_days(i + 1.0), TimeScale::Tt),
                vec![
                    PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 1.0),
                    PolynomialChannel::linear(ChannelKind::Latitude, 9, 0.0, 1.0),
                    PolynomialChannel::linear(ChannelKind::DistanceAu, 12, 1.0, 2.0),
                ],
            )
        })
        .collect::<Vec<_>>();
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("dense-regression", "u32 segment count regression"),
        vec![BodyArtifact::new(CelestialBody::Moon, segments)],
    );
    let encoded = artifact
        .encode()
        .expect("artifact with >65,535 segments should encode");
    let decoded = CompressedArtifact::decode(&encoded)
        .expect("artifact with >65,535 segments should decode without stream misalignment");
    assert_eq!(
        decoded.bodies[0].segments.len(),
        SEGMENT_COUNT,
        "decoded segment count must equal original (u16 truncation would give a wrong value)"
    );
}
