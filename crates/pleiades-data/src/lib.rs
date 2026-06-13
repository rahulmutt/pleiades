//! Packaged compressed ephemeris backend for the common 1500-2500 range.
//!
//! This crate now ships a small stage-5 draft artifact backed by the
//! `pleiades-compression` codec. The bundled data is regenerated from the
//! checked-in JPL reference snapshot and validated against a deterministic
//! binary fixture that covers the comparison-body planetary set plus the
//! source-backed custom asteroid `asteroid:433-Eros`, and the backend falls
//! back to other providers when callers request bodies outside that packaged
//! slice. The packaged artifact stores ecliptic coordinates directly,
//! reconstructs equatorial coordinates from the stored channels and
//! mean-obliquity transform when requested, and adds residual correction
//! channels on high-curvature spans when they improve the fit. A
//! maintainer-facing regeneration helper can rebuild the checked-in fixture
//! from the bundled JPL reference snapshot without introducing any native
//! tooling. When the `packaged-artifact-path` feature is
//! enabled, callers can also load an explicit artifact file for larger or
//! externally distributed packaged datasets. See `docs/time-observer-policy.md`
//! for the explicit packaged request/lookup-epoch policy, and
//! `spec/data-compression.md` for the stored-vs-derived artifact contract.
//!
//! # Examples
//!
//! ```
//! use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
//! use pleiades_data::{packaged_backend, packaged_body_coverage_summary, packaged_lookup};
//!
//! let _backend = packaged_backend();
//! let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
//! let sun = packaged_lookup(&CelestialBody::Sun, instant)
//!     .expect("Sun should be in the packaged artifact");
//!
//! assert!(sun.distance_au.is_some());
//! assert!(packaged_body_coverage_summary().contains("433-Eros"));
//! ```

#![forbid(unsafe_code)]

use std::sync::OnceLock;

use pleiades_backend::{CelestialBody, CustomBodyId};
use pleiades_jpl::SnapshotEntry;

mod backend;
mod coverage;
mod data;
mod lookup;
mod regenerate;

pub use backend::*;
pub use coverage::*;
pub use data::*;
pub use lookup::*;
pub use regenerate::*;

// Test-only re-exports: bring pub(crate) items into lib.rs scope so that
// `use super::*` in the tests module can pick them up.
#[cfg(test)]
pub(crate) use coverage::{
    channel_from_fit_samples_with_control_points, distance_channel_from_fit_samples,
    distance_channel_from_samples, packaged_artifact_body_cadence,
    packaged_artifact_fit_outlier_sample_fractions, packaged_artifact_fit_sample_fractions,
    packaged_artifact_fit_sample_fractions_for_body, PackagedArtifactBodyCadence,
};
#[cfg(test)]
pub(crate) use data::PACKAGED_ARTIFACT_FIXTURE;
#[cfg(test)]
pub(crate) use lookup::{
    validate_packaged_artifact_access_summary_line, validate_packaged_artifact_storage_profile,
    validate_packaged_artifact_storage_summary_line,
    validate_packaged_frame_treatment_summary_line,
};
#[cfg(test)]
pub(crate) use regenerate::{
    best_residual_segment,
    // other functions
    body_segment_span_limit,
    chebyshev_lobatto_fractions,
    coordinates,
    evaluate_polynomial_channel,
    packaged_artifact_fit_sample_counts_for_body,
    packaged_artifact_residual_sample_fractions_for_channel,
    packaged_artifact_segment_validation_fractions_for_body,
    packaged_artifact_split_fraction_for_interval,
    segment_channel_value,
    segment_error_prefers_candidate,
    segment_fit_candidate_is_better,
    segment_from_pair,
    segment_from_pair_fallback,
    snapshot_entry_from_ecliptic_coordinates,
    validate_packaged_artifact_phase1_source_inputs,
    PackagedArtifactFitCandidateScore,
    PackagedArtifactSegmentFitError,
    // structs
    PackagedArtifactSplitCurvature,
    PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS,
    PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS,
    PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS,
    PACKAGED_ARTIFACT_FOUR_FIFTHS_SPLIT_FRACTION,
    // split fraction constants
    PACKAGED_ARTIFACT_LEFT_BIASED_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_LEFT_EXTREME_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS,
    PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS,
    PACKAGED_ARTIFACT_ONE_EIGHTH_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_ONE_FIFTH_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_ONE_NINTH_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_ONE_SEVENTH_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_ONE_THIRD_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS,
    PACKAGED_ARTIFACT_RIGHT_BIASED_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_RIGHT_EXTREME_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_SEVEN_EIGHTHS_SPLIT_FRACTION,
    PACKAGED_ARTIFACT_SIX_SEVENTHS_SPLIT_FRACTION,
};
// External types needed by tests via `use super::*`
#[cfg(test)]
pub(crate) use pleiades_backend::{
    Apparentness, BackendFamily, CoordinateFrame, EclipticCoordinates, EphemerisBackend,
    EphemerisErrorKind, EphemerisRequest, Instant, JulianDay, QualityAnnotation, TimeRange,
    TimeScale, ZodiacMode,
};
#[cfg(test)]
pub(crate) use pleiades_compression::{
    ArtifactOutput, ArtifactProfile, ChannelKind, CompressedArtifact, EndianPolicy,
    PolynomialChannel, Segment, SpeedPolicy,
};
#[cfg(test)]
pub(crate) use pleiades_jpl::{
    production_generation_source_summary_for_report, reference_snapshot, JplSnapshotBackend,
};

const PACKAGE_NAME: &str = "pleiades-data";
const ARTIFACT_LABEL: &str = "stage-5 packaged-data draft";
const ARTIFACT_PROFILE_ID: &str = "pleiades-packaged-artifact-profile/stage-5-draft";
const PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL: &str = "with 8-point and 10-point Chebyshev-Lobatto baseline candidates before the dense body-specific ladders and 12-point and 14-point candidates for inner and outer planets before fallback, with 10-point, 12-point, 14-point, 16-point, 18-point, and 20-point options for luminaries, lunar points, Pluto, selected asteroids, and custom bodies, and the best dense candidate wins before fallback, with equal-error, equal-sample-count ties preferring the simpler segment, residual correction channels on high-curvature spans when they improve the fit, residual-channel combinations and remaining channel-order permutations when composing those channels, preferring the smaller residual footprint on equal-error ties, higher-order reconstruction from fit samples when it quantizes cleanly, shared four-point control-point fallback across longitude, latitude, and distance channels when the higher-order fit does not quantize cleanly, quarter-biased splits on very long dense-body spans when quarter-point curvature is strongly asymmetric, a dense quarter-point control-point lattice before exact-third fallback on irregular spans, one-sixth and five-sixth probe fractions on very long dense-body spans when quarter-point curvature stays balanced, one-third and two-thirds probe fractions on long dense-body spans when quarter-point curvature stays balanced, a dense five-point fallback on the longest dense-body spans when one-fifth through four-fifth samples fit cleanly, a dense seven-point fallback on super-extreme dense-body spans when one-seventh through six-sevenths samples fit cleanly, one-ninth and eight-ninths probe fractions on super-extreme dense-body spans when the finer probes stay balanced, one-eighth and seven-eighths probe fractions on super-extreme dense-body spans when the ninth-point probes stay balanced, one-seventh and six-sevenths probe fractions on extreme dense-body spans when the super-extreme probes stay balanced, one-fifth and four-fifth probe fractions on the longest dense-body spans when the coarser probes stay balanced, and quadratic fallback otherwise";

pub(crate) fn packaged_artifact_generation_policy_note_text() -> &'static str {
    static NOTE: OnceLock<String> = OnceLock::new();
    NOTE.get_or_init(|| {
        format!(
            "bodies with a single sampled epoch use point segments; bodies with two or more sampled epochs are recursively subdivided into quadratic windows using body-class span caps and measured-fit comparison against the fallback, {}",
            PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL
        )
    })
    .as_str()
}

pub(crate) fn packaged_artifact_source_text() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| {
        format!(
            "Quantized adjacent same-body quadratic windows with longitude-unwrapped planetary fits fitted to JPL Horizons reference epochs (1800, 2000, 2500 CE) for the comparison-body planetary set plus asteroid:433-Eros, with point segments only for single-epoch bodies and recursively subdivided quadratic spans for multi-epoch bodies using body-class span caps and measured-fit comparison against the fallback, {}.",
            PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL
        )
    })
    .as_str()
}

const PACKAGED_BASE_BODIES: [CelestialBody; 10] = [
    CelestialBody::Sun,
    CelestialBody::Moon,
    CelestialBody::Mercury,
    CelestialBody::Venus,
    CelestialBody::Mars,
    CelestialBody::Jupiter,
    CelestialBody::Saturn,
    CelestialBody::Uranus,
    CelestialBody::Neptune,
    CelestialBody::Pluto,
];

const PACKAGED_REFERENCE_EPOCH_JD: f64 = 2_451_545.0;

pub(crate) fn packaged_bodies() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES.get_or_init(|| {
        let mut bodies = PACKAGED_BASE_BODIES.to_vec();
        bodies.push(CelestialBody::Custom(CustomBodyId::new(
            "asteroid", "433-Eros",
        )));
        bodies
    })
}

pub(crate) fn packaged_reference_entry_for_body(
    snapshot: &[SnapshotEntry],
    body: &CelestialBody,
) -> Option<SnapshotEntry> {
    snapshot
        .iter()
        .find(|entry| {
            entry.body == *body
                && (entry.epoch.julian_day.days() - PACKAGED_REFERENCE_EPOCH_JD).abs()
                    < f64::EPSILON
        })
        .cloned()
        .or_else(|| snapshot.iter().find(|entry| entry.body == *body).cloned())
}

pub(crate) const AU_IN_KM: f64 = 149_597_870.7;

/// Returns the canonical package name for this crate.
pub const fn package_name() -> &'static str {
    PACKAGE_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_artifact_roundtrips_through_codec() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let fixture = CompressedArtifact::decode(PACKAGED_ARTIFACT_FIXTURE)
            .expect("packaged artifact fixture should decode");
        assert_eq!(
            fixture.header.generation_label,
            artifact.header.generation_label
        );
        assert_eq!(fixture.bodies, artifact.bodies);
        let decoded =
            CompressedArtifact::decode(&encoded).expect("packaged artifact should decode");
        assert_eq!(decoded.header.generation_label, ARTIFACT_LABEL);
        assert_eq!(decoded.bodies.len(), packaged_bodies().len());
        assert_eq!(decoded.checksum, artifact.checksum().unwrap());
    }

    #[test]
    fn packaged_backend_from_artifact_uses_supplied_metadata() {
        let mut artifact = packaged_artifact().clone();
        artifact.header.source = "external packaged artifact".to_string();

        let backend = PackagedDataBackend::from_artifact(artifact);
        let metadata = backend.metadata();

        assert_eq!(metadata.provenance.summary, "external packaged artifact");
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
        assert!(metadata
            .supported_frames
            .contains(&CoordinateFrame::Equatorial));
    }

    #[cfg(feature = "packaged-artifact-path")]
    #[test]
    fn packaged_backend_from_path_loads_a_file_artifact() {
        let path = std::env::temp_dir().join(format!(
            "pleiades-data-packaged-artifact-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos()
        ));
        std::fs::write(&path, PACKAGED_ARTIFACT_FIXTURE).expect("test artifact should be writable");

        let backend = PackagedDataBackend::from_path(&path)
            .expect("packaged artifact path should load successfully");
        let metadata = backend.metadata();

        assert_eq!(metadata.id.as_str(), PACKAGE_NAME);
        assert!(metadata.offline);
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));

        let _ = std::fs::remove_file(&path);
    }

    #[cfg(feature = "packaged-artifact-path")]
    #[test]
    fn packaged_artifact_from_path_rejects_corrupted_artifact() {
        let path = std::env::temp_dir().join(format!(
            "pleiades-data-packaged-artifact-corrupt-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos()
        ));
        std::fs::write(&path, b"not a valid packaged artifact")
            .expect("corrupt artifact should be writable");

        let error = packaged_artifact_from_path(&path)
            .expect_err("corrupted packaged artifact should fail to decode");
        let error_text = error.to_string();

        match error {
            PackagedArtifactLoadError::Decode(_) => {}
            other => panic!("expected decode failure, got {other}"),
        }
        assert!(error_text.contains("failed to decode packaged artifact"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn packaged_artifact_decode_rejects_checksum_corruption() {
        let mut encoded = PACKAGED_ARTIFACT_FIXTURE.to_vec();
        let last_index = encoded.len() - 1;
        encoded[last_index] ^= 0x01;

        let error = CompressedArtifact::decode(&encoded)
            .expect_err("tampered packaged artifact should fail to decode");

        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::ChecksumMismatch
        );
    }

    #[test]
    #[ignore = "full packaged-artifact regeneration is a slow release-validation check"]
    fn packaged_artifact_fixture_matches_reference_snapshot_generation() {
        let generated = regenerate_packaged_artifact();
        generated
            .validate()
            .expect("generated packaged artifact should validate");
        let encoded = generated
            .encode()
            .expect("generated packaged artifact should encode");
        assert_eq!(encoded, PACKAGED_ARTIFACT_FIXTURE);
        assert_eq!(
            generated.residual_segment_count() > 0,
            !generated.residual_bodies().is_empty()
        );
    }

    #[test]
    #[ignore = "full packaged-artifact regeneration is a slow release-validation check"]
    fn packaged_artifact_generation_from_supplied_snapshot_matches_the_default_fixture() {
        let snapshot = reference_snapshot();
        let generated_from_snapshot = regenerate_packaged_artifact_from_snapshot(snapshot);
        let generated_default = regenerate_packaged_artifact();

        assert_eq!(generated_from_snapshot, generated_default);
        assert_eq!(
            generated_from_snapshot.encode().unwrap(),
            PACKAGED_ARTIFACT_FIXTURE
        );
    }

    #[test]
    fn packaged_artifact_generation_rejects_tampered_reference_snapshot_inputs() {
        let mut snapshot = reference_snapshot().to_vec();
        snapshot[0].x_km += 1.0;

        let error = try_regenerate_packaged_artifact_from_snapshot(&snapshot)
            .expect_err("tampered reference snapshot inputs should be rejected");

        assert!(
            error
                .to_string()
                .contains("packaged artifact regeneration snapshot input at index 0 does not match the checked-in reference snapshot"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn packaged_artifact_generation_validates_phase1_source_inputs() {
        validate_packaged_artifact_phase1_source_inputs()
            .expect("phase-1 source inputs should validate before regeneration");
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
        let distance = |fraction: f64| {
            1.0 + 0.5 * fraction - 0.25 * fraction.powi(2) + 0.125 * fraction.powi(3)
        };
        let start_coordinates = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(longitude(0.0)),
            pleiades_backend::Latitude::from_degrees(latitude(0.0)),
            Some(distance(0.0)),
        );
        let end_coordinates = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(longitude(1.0)),
            pleiades_backend::Latitude::from_degrees(latitude(1.0)),
            Some(distance(1.0)),
        );
        let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
            if (fraction - 0.5).abs() < f64::EPSILON {
                return Some(EclipticCoordinates::new(
                    pleiades_backend::Longitude::from_degrees(1.0e20),
                    pleiades_backend::Latitude::from_degrees(-1.0e20),
                    Some(1.0e20),
                ));
            }

            Some(EclipticCoordinates::new(
                pleiades_backend::Longitude::from_degrees(longitude(fraction)),
                pleiades_backend::Latitude::from_degrees(latitude(fraction)),
                Some(distance(fraction)),
            ))
        };
        let segment = segment_from_pair_fallback(
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
            longitude(0.0),
            longitude(1.0),
            &start_coordinates,
            &end_coordinates,
            Some(1.0),
            Some(1.0),
            &sample_fraction,
        );

        for fraction in [0.25, 0.5, 0.75] {
            let actual_longitude =
                segment_channel_value(&segment, ChannelKind::Longitude, fraction)
                    .expect("longitude channel should evaluate");
            let actual_latitude = segment_channel_value(&segment, ChannelKind::Latitude, fraction)
                .expect("latitude channel should evaluate");
            let actual_distance =
                segment_channel_value(&segment, ChannelKind::DistanceAu, fraction)
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
            15.0 - 3.0 * fraction + 2.0 * fraction.powi(2) - fraction.powi(3)
                + 0.5 * fraction.powi(4)
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
        let start_coordinates = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(longitude(0.0)),
            pleiades_backend::Latitude::from_degrees(latitude(0.0)),
            Some(distance(0.0)),
        );
        let end_coordinates = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(longitude(1.0)),
            pleiades_backend::Latitude::from_degrees(latitude(1.0)),
            Some(distance(1.0)),
        );
        let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
            if (fraction - 0.25).abs() < f64::EPSILON || (fraction - 0.75).abs() < f64::EPSILON {
                return None;
            }

            Some(EclipticCoordinates::new(
                pleiades_backend::Longitude::from_degrees(longitude(fraction)),
                pleiades_backend::Latitude::from_degrees(latitude(fraction)),
                Some(distance(fraction)),
            ))
        };
        let segment = segment_from_pair_fallback(
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(13_000.0), TimeScale::Tt),
            longitude(0.0),
            longitude(1.0),
            &start_coordinates,
            &end_coordinates,
            Some(13_000.0),
            Some(1_536.0),
            &sample_fraction,
        );

        for fraction in [0.2, 0.4, 0.5, 0.6, 0.8] {
            let actual_longitude =
                segment_channel_value(&segment, ChannelKind::Longitude, fraction)
                    .expect("longitude channel should evaluate");
            let actual_latitude = segment_channel_value(&segment, ChannelKind::Latitude, fraction)
                .expect("latitude channel should evaluate");
            let actual_distance =
                segment_channel_value(&segment, ChannelKind::DistanceAu, fraction)
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
        let longitude = |fraction: f64| {
            8.0 + 1.25 * fraction - 0.5 * fraction.powi(2) + 0.125 * fraction.powi(3)
        };
        let latitude = |fraction: f64| {
            -3.0 + 0.75 * fraction + 0.25 * fraction.powi(2) - 0.0625 * fraction.powi(3)
        };
        let distance = |fraction: f64| {
            2.0 + 0.5 * fraction - 0.125 * fraction.powi(2) + 0.03125 * fraction.powi(3)
        };
        let start_coordinates = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(longitude(0.0)),
            pleiades_backend::Latitude::from_degrees(latitude(0.0)),
            Some(distance(0.0)),
        );
        let end_coordinates = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(longitude(1.0)),
            pleiades_backend::Latitude::from_degrees(latitude(1.0)),
            Some(distance(1.0)),
        );
        let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
            if (fraction - 0.2).abs() < f64::EPSILON
                || (fraction - 0.4).abs() < f64::EPSILON
                || (fraction - 0.6).abs() < f64::EPSILON
                || (fraction - 0.8).abs() < f64::EPSILON
            {
                return None;
            }

            Some(EclipticCoordinates::new(
                pleiades_backend::Longitude::from_degrees(longitude(fraction)),
                pleiades_backend::Latitude::from_degrees(latitude(fraction)),
                Some(distance(fraction)),
            ))
        };
        let segment = segment_from_pair_fallback(
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(50_000.0), TimeScale::Tt),
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
            let actual_longitude =
                segment_channel_value(&segment, ChannelKind::Longitude, fraction)
                    .expect("longitude channel should evaluate");
            let actual_latitude = segment_channel_value(&segment, ChannelKind::Latitude, fraction)
                .expect("latitude channel should evaluate");
            let actual_distance =
                segment_channel_value(&segment, ChannelKind::DistanceAu, fraction)
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
        let channel =
            channel_from_fit_samples_with_control_points(ChannelKind::Latitude, 0, &samples)
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
                    .find(|segment| {
                        segment.start.julian_day.days() != segment.end.julian_day.days()
                    })
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
                    .find(|segment| {
                        segment.start.julian_day.days() != segment.end.julian_day.days()
                    })
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
                    .find(|segment| {
                        segment.start.julian_day.days() != segment.end.julian_day.days()
                    })
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
                    .find(|segment| {
                        segment.start.julian_day.days() != segment.end.julian_day.days()
                    })
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
        let sample_segment = Segment::new(
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_451_555.0), TimeScale::Tt),
            Vec::new(),
        );

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
                packaged_artifact_residual_sample_fractions_for_channel(
                    &body,
                    ChannelKind::Longitude
                ),
                PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
            );
            assert_eq!(
                packaged_artifact_residual_sample_fractions_for_channel(
                    &body,
                    ChannelKind::Latitude
                ),
                PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
            );
            assert_eq!(
                packaged_artifact_residual_sample_fractions_for_channel(
                    &body,
                    ChannelKind::DistanceAu
                ),
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
        let moderate_left_start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let moderate_left_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.01),
        );
        let moderate_left_midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.8),
            pleiades_backend::Latitude::from_degrees(0.7),
            Some(1.02),
        );
        let moderate_left_three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.6),
            pleiades_backend::Latitude::from_degrees(1.0),
            Some(1.03),
        );
        let moderate_left_end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.4),
            pleiades_backend::Latitude::from_degrees(1.3),
            Some(1.04),
        );

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

        let moderate_right_start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let moderate_right_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.8),
            pleiades_backend::Latitude::from_degrees(0.3),
            Some(1.01),
        );
        let moderate_right_midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.02),
        );
        let moderate_right_three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.8),
            pleiades_backend::Latitude::from_degrees(0.7),
            Some(1.03),
        );
        let moderate_right_end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.1),
            Some(1.04),
        );

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

        let extreme_left_start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let extreme_left_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(8.0),
            pleiades_backend::Latitude::from_degrees(4.0),
            Some(1.1),
        );
        let extreme_left_midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(14.0),
            pleiades_backend::Latitude::from_degrees(7.0),
            Some(1.2),
        );
        let extreme_left_three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(15.0),
            pleiades_backend::Latitude::from_degrees(7.2),
            Some(1.22),
        );
        let extreme_left_end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(16.0),
            pleiades_backend::Latitude::from_degrees(7.4),
            Some(1.24),
        );

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

        let extreme_right_start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let extreme_right_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.5),
            Some(1.01),
        );
        let extreme_right_midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(1.0),
            Some(1.02),
        );
        let extreme_right_three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(10.0),
            pleiades_backend::Latitude::from_degrees(5.0),
            Some(1.08),
        );
        let extreme_right_end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(16.0),
            pleiades_backend::Latitude::from_degrees(8.0),
            Some(1.12),
        );

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
    fn packaged_artifact_split_fraction_uses_dense_third_point_bias_when_quarter_curvature_is_balanced(
    ) {
        let start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.02),
        );
        let one_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(5.0),
            pleiades_backend::Latitude::from_degrees(2.0),
            Some(1.08),
        );
        let midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(0.8),
            Some(1.04),
        );
        let two_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.1),
            pleiades_backend::Latitude::from_degrees(0.85),
            Some(1.05),
        );
        let three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.2),
            Some(1.06),
        );
        let end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(4.0),
            pleiades_backend::Latitude::from_degrees(1.6),
            Some(1.08),
        );

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
        let start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let one_sixth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.9),
            pleiades_backend::Latitude::from_degrees(0.35),
            Some(1.01),
        );
        let quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.02),
        );
        let one_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.5),
            pleiades_backend::Latitude::from_degrees(0.6),
            Some(1.03),
        );
        let midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(0.8),
            Some(1.04),
        );
        let two_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.4),
            pleiades_backend::Latitude::from_degrees(0.9),
            Some(1.05),
        );
        let three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.2),
            Some(1.06),
        );
        let five_sixth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(8.0),
            pleiades_backend::Latitude::from_degrees(4.0),
            Some(1.1),
        );
        let end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.4),
            pleiades_backend::Latitude::from_degrees(1.3),
            Some(1.07),
        );

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
        let start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.01),
        );
        let one_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(5.0),
            pleiades_backend::Latitude::from_degrees(2.0),
            Some(2.0),
        );
        let midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(0.8),
            Some(1.02),
        );
        let two_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.1),
            pleiades_backend::Latitude::from_degrees(0.85),
            Some(1.05),
        );
        let three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.2),
            Some(1.03),
        );
        let end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(4.0),
            pleiades_backend::Latitude::from_degrees(1.6),
            Some(1.04),
        );

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
        let start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.01),
        );
        let midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(0.8),
            Some(1.02),
        );
        let three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.2),
            Some(1.03),
        );
        let end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(4.0),
            pleiades_backend::Latitude::from_degrees(1.6),
            Some(1.04),
        );

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
        let start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.01),
        );
        let one_fifth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(6.0),
            pleiades_backend::Latitude::from_degrees(3.0),
            Some(1.06),
        );
        let one_sixth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.8),
            pleiades_backend::Latitude::from_degrees(0.32),
            Some(1.008),
        );
        let one_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.5),
            pleiades_backend::Latitude::from_degrees(0.6),
            Some(1.015),
        );
        let midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(0.8),
            Some(1.02),
        );
        let two_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.5),
            pleiades_backend::Latitude::from_degrees(1.0),
            Some(1.03),
        );
        let four_fifth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.2),
            pleiades_backend::Latitude::from_degrees(0.9),
            Some(1.025),
        );
        let five_sixth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.2),
            pleiades_backend::Latitude::from_degrees(1.28),
            Some(1.036),
        );
        let three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.2),
            Some(1.04),
        );
        let end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(4.0),
            pleiades_backend::Latitude::from_degrees(1.6),
            Some(1.08),
        );

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
        let start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.01),
        );
        let one_fifth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.1),
            pleiades_backend::Latitude::from_degrees(0.44),
            Some(1.011),
        );
        let one_sixth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.8),
            pleiades_backend::Latitude::from_degrees(0.32),
            Some(1.008),
        );
        let one_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.5),
            pleiades_backend::Latitude::from_degrees(0.6),
            Some(1.015),
        );
        let midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(0.8),
            Some(1.02),
        );
        let two_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.5),
            pleiades_backend::Latitude::from_degrees(1.0),
            Some(1.03),
        );
        let four_fifth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(7.0),
            pleiades_backend::Latitude::from_degrees(3.5),
            Some(1.07),
        );
        let five_sixth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.2),
            pleiades_backend::Latitude::from_degrees(1.28),
            Some(1.036),
        );
        let three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.2),
            Some(1.04),
        );
        let end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(4.0),
            pleiades_backend::Latitude::from_degrees(1.6),
            Some(1.08),
        );

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
    fn packaged_artifact_split_fraction_uses_dense_ninth_and_eighth_point_bias_on_super_extreme_spans(
    ) {
        let point = |longitude: f64, latitude: f64| {
            EclipticCoordinates::new(
                pleiades_backend::Longitude::from_degrees(longitude),
                pleiades_backend::Latitude::from_degrees(latitude),
                Some(1.0),
            )
        };

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
    fn packaged_artifact_split_fraction_ignores_fifth_point_bias_before_the_longest_span_threshold()
    {
        let start = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(0.0),
            pleiades_backend::Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.0),
            pleiades_backend::Latitude::from_degrees(0.4),
            Some(1.01),
        );
        let one_fifth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(6.2),
            pleiades_backend::Latitude::from_degrees(3.1),
            Some(1.062),
        );
        let one_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(1.5),
            pleiades_backend::Latitude::from_degrees(0.6),
            Some(1.015),
        );
        let midpoint = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.0),
            pleiades_backend::Latitude::from_degrees(0.8),
            Some(1.02),
        );
        let two_third = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(2.5),
            pleiades_backend::Latitude::from_degrees(1.0),
            Some(1.03),
        );
        let four_fifth = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(6.0),
            pleiades_backend::Latitude::from_degrees(3.0),
            Some(1.06),
        );
        let three_quarter = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(3.0),
            pleiades_backend::Latitude::from_degrees(1.2),
            Some(1.04),
        );
        let end = EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(4.0),
            pleiades_backend::Latitude::from_degrees(1.6),
            Some(1.08),
        );

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
        let point = |longitude: f64, latitude: f64| {
            EclipticCoordinates::new(
                pleiades_backend::Longitude::from_degrees(longitude),
                pleiades_backend::Latitude::from_degrees(latitude),
                Some(1.0),
            )
        };

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

        assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
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
        assert!(
            (actual_ecliptic.longitude.degrees() - expected_ecliptic.longitude.degrees()).abs()
                < 1e-8
        );
        assert!(
            (actual_ecliptic.latitude.degrees() - expected_ecliptic.latitude.degrees()).abs()
                < 1e-8
        );
        assert!(
            (actual_ecliptic.distance_au.unwrap() - expected_ecliptic.distance_au.unwrap()).abs()
                < 1e-9
        );
        let actual_equatorial = result
            .equatorial
            .expect("packaged equatorial request should return derived equatorial coordinates");
        assert!(
            (actual_equatorial.right_ascension.degrees() - expected.right_ascension.degrees())
                .abs()
                < 1e-8
        );
        assert!(
            (actual_equatorial.declination.degrees() - expected.declination.degrees()).abs() < 1e-8
        );
        assert!(
            (actual_equatorial.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-9
        );
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
        let body = CelestialBody::Moon;
        for epoch in [2_400_000.0, 2_500_000.0] {
            let reference = reference_snapshot()
                .iter()
                .find(|entry| {
                    entry.body == body
                        && (entry.epoch.julian_day.days() - epoch).abs() < f64::EPSILON
                })
                .expect("reference snapshot should include the Moon at the sampled epoch");
            let ecliptic = packaged_lookup(&body, reference.epoch)
                .expect("packaged lookup should succeed for the Moon");
            let expected = coordinates(reference);

            assert!(
                (ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-6,
                "moon longitude diff={:.12}",
                (ecliptic.longitude.degrees() - expected.longitude.degrees()).abs()
            );
            assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 20.0);
            assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-3);
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
        let lunar_point_distance_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
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
        let custom_body_longitude_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
                &CelestialBody::Custom(CustomBodyId::new("comet", "1P-Halley")),
                ChannelKind::Longitude,
            );
        let custom_body_distance_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
                &CelestialBody::Custom(CustomBodyId::new("comet", "1P-Halley")),
                ChannelKind::DistanceAu,
            );
        let inner_planet_longitude_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
                &CelestialBody::Mercury,
                ChannelKind::Longitude,
            );
        let inner_planet_latitude_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
                &CelestialBody::Mercury,
                ChannelKind::Latitude,
            );
        let inner_planet_distance_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
                &CelestialBody::Mercury,
                ChannelKind::DistanceAu,
            );
        let outer_planet_longitude_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
                &CelestialBody::Saturn,
                ChannelKind::Longitude,
            );
        let outer_planet_latitude_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
                &CelestialBody::Saturn,
                ChannelKind::Latitude,
            );
        let outer_planet_distance_fractions =
            packaged_artifact_residual_sample_fractions_for_channel(
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

        let current_segment = Segment::new(
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
            vec![PolynomialChannel::new(ChannelKind::Longitude, 0, vec![0.0])],
        );
        let current_error = PackagedArtifactSegmentFitError {
            longitude_degrees: 10.0,
            latitude_degrees: 10.0,
            distance_au: 10.0,
        };

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

        let current_segment = Segment::new(
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
            vec![PolynomialChannel::new(ChannelKind::Longitude, 0, vec![0.0])],
        );
        let current_error = PackagedArtifactSegmentFitError {
            longitude_degrees: 10.0,
            latitude_degrees: 10.0,
            distance_au: 10.0,
        };

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
    fn moon_residual_search_prefers_smaller_residual_coefficient_footprint_equal_error_candidates()
    {
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

        let current_segment = Segment::new(
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
            vec![PolynomialChannel::new(ChannelKind::Longitude, 0, vec![0.0])],
        );
        let current_error = PackagedArtifactSegmentFitError {
            longitude_degrees: 10.0,
            latitude_degrees: 10.0,
            distance_au: 10.0,
        };

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
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
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
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
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
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
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
            Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
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
            instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tt),
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
        let start = snapshot_entry_from_ecliptic_coordinates(
            body.clone(),
            start_julian_day,
            start_coordinates,
        );
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

        let mut body_bounds: HashMap<CelestialBody, (Instant, Instant)> = HashMap::new();
        for body in packaged_bodies() {
            let mut body_entries = reference_snapshot()
                .iter()
                .filter(|entry| entry.body == *body);
            let Some(first_entry) = body_entries.next() else {
                panic!("reference snapshot should include packaged body {body}");
            };
            let mut earliest = first_entry.epoch;
            let mut latest = first_entry.epoch;

            for entry in body_entries {
                if entry.epoch.julian_day.days() < earliest.julian_day.days() {
                    earliest = entry.epoch;
                }
                if entry.epoch.julian_day.days() > latest.julian_day.days() {
                    latest = entry.epoch;
                }
            }

            body_bounds.insert(body.clone(), (earliest, latest));
        }

        for (body, (earliest, latest)) in body_bounds {
            for epoch in [earliest, latest] {
                let reference = reference_snapshot()
                    .iter()
                    .find(|entry| entry.body == body && entry.epoch == epoch)
                    .expect("reference snapshot should include the body's boundary epoch");
                let ecliptic = packaged_lookup(&body, epoch)
                    .expect("packaged lookup should succeed for reference boundary epochs");
                let expected = coordinates(reference);
                let longitude_tolerance = if body == CelestialBody::Pluto {
                    1e-5
                } else {
                    3e-5
                };
                let latitude_tolerance = 1e-5;
                let distance_tolerance = 1e-5;

                assert!(
                    (ecliptic.longitude.degrees() - expected.longitude.degrees()).abs()
                        < longitude_tolerance,
                    "boundary longitude diff={:.12} body={}",
                    (ecliptic.longitude.degrees() - expected.longitude.degrees()).abs(),
                    body
                );
                assert!(
                    (ecliptic.latitude.degrees() - expected.latitude.degrees()).abs()
                        < latitude_tolerance,
                    "boundary latitude diff={:.12} body={} epoch={}",
                    (ecliptic.latitude.degrees() - expected.latitude.degrees()).abs(),
                    body,
                    epoch
                );
                assert!(
                    (ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs()
                        < distance_tolerance,
                    "boundary distance diff={:.12} body={} epoch={}",
                    (ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs(),
                    body,
                    epoch
                );
            }
        }
    }

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
            pleiades_compression::SpeedPolicy::Unsupported
        );
        assert!(packaged_artifact()
            .header
            .profile
            .unsupported_outputs
            .contains(&pleiades_compression::ArtifactOutput::Motion));
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
        assert!(metadata.body_coverage.contains(&CelestialBody::Moon));
        assert!(metadata.body_coverage.contains(&CelestialBody::Jupiter));
        assert!(metadata.body_coverage.contains(&CelestialBody::Pluto));
        assert!(metadata
            .body_coverage
            .contains(&CelestialBody::Custom(CustomBodyId::new(
                "asteroid", "433-Eros",
            ))));
        assert!(metadata.provenance.data_sources[0].contains("11 bundled bodies"));
        assert!(metadata.provenance.data_sources[0].contains("asteroid:433-Eros"));
        assert_eq!(
            packaged_body_coverage_summary(),
            metadata.provenance.data_sources[0]
        );
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
            packaged_request_policy_summary_for_report()
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
            packaged_lookup_epoch_policy_summary_for_report()
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
            packaged_request_policy_summary_for_report()
        );
        assert_eq!(
            metadata.provenance.data_sources[2],
            packaged_frame_treatment_summary_for_report()
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
        assert!(metadata.provenance.data_sources[2]
            .contains("equatorial coordinates are reconstructed"));
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
            .contains("apparent, topocentric, sidereal, and motion outputs remain unsupported"));
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
            "stored channels: [Longitude, Latitude, DistanceAu]; derived outputs: [EclipticCoordinates, EquatorialCoordinates]; unsupported outputs: [ApparentCorrections, TopocentricCoordinates, SiderealCoordinates, Motion]; speed policy: Unsupported"
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
            packaged_artifact_output_support_summary_for_report(),
            summary.profile.output_support_summary_line()
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
            packaged_artifact_profile_coverage_summary_for_report(),
            summary
                .profile_coverage_summary()
                .summary_line_with_bodies()
        );
        assert_eq!(
            packaged_artifact_profile_summary_with_output_support(),
            summary.summary_line_with_output_support()
        );
        assert_eq!(
            packaged_artifact_profile_summary_with_output_support_for_report(),
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
        summary.profile.derived_outputs.retain(|output| {
            *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates
        });

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
        profile.derived_outputs.retain(|output| {
            *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates
        });

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
        profile.derived_outputs.retain(|output| {
            *output != pleiades_compression::ArtifactOutput::EquatorialCoordinates
        });

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
            .unsupported_outputs
            .retain(|output| *output != pleiades_compression::ArtifactOutput::Motion);

        let error = validate_packaged_artifact_storage_profile(&profile)
            .expect_err("drifted packaged storage profile should be rejected");
        assert_eq!(
            error,
            PackagedArtifactStorageSummaryValidationError::ProfileOutOfSync {
                field: "unsupported_outputs"
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
        assert_eq!(
            packaged_artifact_access_summary_for_report(),
            summary.to_string()
        );
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
        assert_eq!(
            packaged_artifact_output_support_summary_for_report(),
            summary.summary_line()
        );
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
        assert!(artifact.residual_bodies().contains(&CelestialBody::Moon));
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
        assert_eq!(
            packaged_artifact_generation_residual_bodies_summary_for_report(),
            residual_bodies.summary_line_with_body_count()
        );
    }

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
        let error =
            PackagedArtifactGenerationPolicyValidationError::FieldOutOfSync { field: "policy" };
        assert_eq!(
            error.summary_line(),
            "the packaged artifact generation policy field `policy` is out of sync with the current posture"
        );
        assert_eq!(error.to_string(), error.summary_line());
    }

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
        assert_eq!(
            summary.residual_body_line(),
            packaged_artifact_generation_residual_bodies_summary_for_report()
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
        assert_eq!(
            packaged_body_coverage_summary(),
            packaged_body_coverage_summary_details().to_string()
        );

        let provenance = summary.summary_line();
        assert_eq!(summary.to_string(), provenance);
        assert_eq!(summary.validated_summary_line(), Ok(provenance.clone()));
        summary
            .validate()
            .expect("packaged regeneration summary should validate");
        assert_eq!(
            summary.normalized_intermediates.summary_line(),
            packaged_artifact_normalized_intermediate_summary_for_report()
        );
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
        assert!(provenance
            .contains("quantization scales: stored=Longitude=9, Latitude=9, DistanceAu=10"));
        assert!(
            provenance.contains(&packaged_artifact_generation_residual_bodies_summary_for_report())
        );
        assert!(provenance.contains(&format!("artifact version={}", artifact.header.version)));
        assert!(provenance.contains("11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)"));
        assert!(provenance.contains("Reference snapshot coverage:"));
        assert!(provenance.contains("fit envelope:"));
        assert!(provenance.contains("segment samples across"));
        assert!(provenance.contains("rows across"));
        assert!(provenance.contains("asteroid rows"));
    }

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
        assert!(summary.summary_line().contains(
            "Packaged artifact normalized intermediates: label=stage-5 packaged-data draft"
        ));
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
        assert!(
            packaged_artifact_generation_policy_note_text().contains("smaller residual footprint")
        );
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
        assert!(
            packaged_artifact_source_text().contains(PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL)
        );
        assert!(packaged_artifact_source_text()
            .ends_with(&format!("{PACKAGED_ARTIFACT_GENERATION_STRATEGY_TAIL}.")));
    }

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
        assert_eq!(
            packaged_frame_treatment_summary_for_report(),
            summary.to_string()
        );
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
        assert!(error
            .message
            .contains("packaged artifact regeneration summary label does not match the checked-in artifact label"));

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
        assert!(error.message.contains(
            "packaged artifact regeneration summary is missing reference snapshot coverage"
        ));
    }

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
        assert!(summary.summary_line().contains("speed policy=Unsupported"));
        assert!(summary
            .summary_line()
            .contains("segment strategy=bodies with a single sampled epoch use point segments"));
        assert!(summary
            .summary_line()
            .contains("target thresholds: production thresholds recorded; scopes=luminaries, major planets, pluto, lunar points, selected asteroids, custom bodies; fit envelope:"));
        assert!(summary
            .summary_line()
            .contains("scope envelopes=scope=luminaries; bodies=2 (Sun, Moon); fit envelope:"));
        assert!(
            packaged_artifact_target_threshold_scope_envelopes_for_report()
                .contains("scope=luminaries; bodies=2 (Sun, Moon); fit envelope:")
        );
        assert!(packaged_artifact_production_profile_summary_for_report()
            .contains("Packaged artifact production profile draft:"));
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
        assert_eq!(summary.policy, SpeedPolicy::Unsupported);
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Unsupported; motion output support=unsupported"
        );
        assert_eq!(
            packaged_artifact_speed_policy_summary_for_report(),
            summary.summary_line()
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
        assert!(manifest.summary_line().contains("speed policy=Unsupported"));
        assert!(manifest.summary_line().contains("segment strategy="));
        assert!(manifest
            .summary_line()
            .contains("source revision=Production generation source:"));
        assert!(manifest.summary_line().contains("regeneration="));
        assert!(packaged_artifact_generation_manifest_for_report()
            .contains("Packaged artifact generation manifest:"));
        assert_eq!(
            packaged_artifact_generation_manifest(),
            manifest.summary_line()
        );
    }

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

    #[test]
    fn packaged_artifact_generation_manifest_validation_rejects_artifact_profile_drift() {
        let mut manifest = packaged_artifact_generation_manifest_details();
        manifest.parameters.artifact_profile.speed_policy =
            pleiades_compression::SpeedPolicy::Stored;

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

    #[test]
    fn packaged_artifact_production_profile_summary_validation_rejects_label_drift() {
        let mut summary = packaged_artifact_production_profile_summary_details();
        summary.label = "drifted label";

        let error = summary
            .validate()
            .expect_err("label drift should be rejected");
        assert_eq!(
            error,
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "label",
            }
        );
        assert!(error.to_string().contains("label"));
    }

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

    #[test]
    fn packaged_artifact_target_threshold_summary_validation_rejects_draft_state() {
        let mut summary = packaged_artifact_target_threshold_summary_details();
        summary.state = PackagedArtifactTargetThresholdState::Draft;

        let error = summary
            .validate()
            .expect_err("draft target-threshold state should be rejected");
        assert_eq!(
            error,
            PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                field: "state",
            }
        );
        assert!(error.to_string().contains("state"));
    }

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

    #[test]
    fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_corpus_alignment_drift()
    {
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

    #[test]
    fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_boundary_source_drift()
    {
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

    #[test]
    fn packaged_artifact_target_threshold_summary_validation_rejects_phase2_generation_source_drift(
    ) {
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
    fn packaged_artifact_phase2_corpus_alignment_summary_for_report_is_validated() {
        let rendered = packaged_artifact_phase2_corpus_alignment_summary_for_report();
        assert!(rendered.contains("reference snapshot="));
        assert!(rendered.contains("reference exact J2000 evidence="));
        assert!(rendered.contains("comparison snapshot="));
        assert!(rendered.contains("independent hold-out="));
        assert!(rendered.contains(
            "production generation body-class coverage=Production generation body-class coverage:"
        ));
        assert!(rendered.contains("production generation source=Production generation source:"));
        assert!(rendered.contains("Reference snapshot body-class coverage"));
        assert!(rendered.contains("Reference snapshot exact J2000 evidence"));
        assert!(rendered.contains("Independent hold-out body-class coverage"));
        assert!(rendered.contains(
            "selected asteroid source request corpus=Selected asteroid source request corpus:"
        ));
        assert!(rendered.contains(
            "selected asteroid source request corpus equatorial=Selected asteroid source request corpus:"
        ));
        assert!(rendered.contains(
            "production generation boundary source=Production generation boundary overlay source:"
        ));
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
        assert_eq!(
            packaged_artifact_phase2_corpus_alignment_summary_for_report(),
            summary.summary_line()
        );
    }

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
            .contains("fit thresholds: mean Δlon≤39.066737306976°"));
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
        assert_eq!(
            packaged_artifact_source_fit_holdout_sync_summary_for_report(),
            summary.summary_line()
        );
    }

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

    #[test]
    fn packaged_artifact_source_fit_holdout_sync_summary_validation_rejects_target_threshold_drift()
    {
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

    #[test]
    fn packaged_artifact_fit_threshold_summary_reflects_the_current_posture() {
        let summary = packaged_artifact_fit_threshold_summary_details();
        let thresholds = packaged_artifact_fit_threshold_summary_details();
        let violations = packaged_artifact_fit_threshold_violation_summary_details();
        let scope_envelopes = packaged_artifact_target_threshold_scope_envelopes_summary_details();

        assert_eq!(
            summary.summary_line(),
            "fit thresholds: mean Δlon≤39.066737306976°, mean Δlat≤54.258413456361°, mean Δdist≤167525.454245761939 AU; max Δlon≤179.935747101401°, max Δlat≤5436.377507814662°, max Δdist≤67056450.790259867907 AU"
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
        assert!(packaged_artifact_fit_threshold_summary_for_report()
            .contains("fit thresholds: mean Δlon≤39.066737306976°"));
        assert_eq!(
            packaged_artifact_fit_threshold_violation_count_for_report(),
            "fit threshold violations: 0"
        );
        assert_eq!(
            packaged_artifact_fit_threshold_violation_summary_for_report(),
            "fit threshold violations: 0; details: none"
        );
    }

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

    #[test]
    fn packaged_artifact_fit_margin_summary_reflects_the_current_posture() {
        let summary = packaged_artifact_fit_margin_summary_details();
        assert_eq!(
            summary.summary_line(),
            packaged_artifact_fit_margin_summary_for_report()
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert!(summary.validate().is_ok());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    }

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
        assert_eq!(
            packaged_artifact_fit_channel_outlier_summary_for_report(),
            by_channel
        );
    }

    #[test]
    fn packaged_artifact_fit_channel_outlier_summary_prefers_shorter_failing_family_on_equal_delta()
    {
        let body = CelestialBody::Moon;
        let long_segment_start = Instant::new(JulianDay::from_days(0.0), TimeScale::Tt);
        let long_segment_end = Instant::new(JulianDay::from_days(10.0), TimeScale::Tt);
        let short_segment_start = Instant::new(JulianDay::from_days(20.0), TimeScale::Tt);
        let short_segment_end = Instant::new(JulianDay::from_days(22.0), TimeScale::Tt);

        let samples = vec![
            PackagedArtifactFitSample {
                body: body.clone(),
                segment_start: long_segment_start,
                segment_end: long_segment_end,
                sample_instant: Instant::new(JulianDay::from_days(2.5), TimeScale::Tt),
                sample_fraction: 0.25,
                longitude_delta_degrees: 1.0,
                latitude_delta_degrees: 0.0,
                distance_delta_au: 0.0,
            },
            PackagedArtifactFitSample {
                body: body.clone(),
                segment_start: long_segment_start,
                segment_end: long_segment_end,
                sample_instant: Instant::new(JulianDay::from_days(7.5), TimeScale::Tt),
                sample_fraction: 0.75,
                longitude_delta_degrees: 1.0,
                latitude_delta_degrees: 0.0,
                distance_delta_au: 0.0,
            },
            PackagedArtifactFitSample {
                body,
                segment_start: short_segment_start,
                segment_end: short_segment_end,
                sample_instant: Instant::new(JulianDay::from_days(21.0), TimeScale::Tt),
                sample_fraction: 0.5,
                longitude_delta_degrees: 1.0,
                latitude_delta_degrees: 0.0,
                distance_delta_au: 0.0,
            },
        ];

        let summary = packaged_artifact_fit_channel_outlier_summary_for_channel(
            &samples,
            ChannelKind::Longitude,
        )
        .expect("channel summary should exist");

        assert!(summary.contains("span=2.000000000000 d"));
        assert!(summary.contains("samples=1"));
        assert!(!summary.contains("span=10.000000000000 d"));
    }

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
        assert_eq!(
            packaged_artifact_body_class_span_cap_summary_for_report(),
            summary.to_string()
        );
        assert_eq!(
            packaged_artifact_body_class_span_cap_entries_for_report(),
            "luminaries=256 days, inner planets=384 days, outer planets=768 days, pluto=1536 days, lunar points=256 days, selected asteroids=256 days, custom bodies=512 days"
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
            PackagedArtifactBodyClassSpanCapSummaryValidationError::FieldOutOfSync {
                field: "entries",
            }
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
        assert_eq!(
            packaged_artifact_body_cadence_summary_for_report(),
            summary.summary_line()
        );
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
        assert_eq!(packaged_body_coverage_summary(), summary.to_string());
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

    #[test]
    fn packaged_body_coverage_summary_report_marks_drift_as_unavailable() {
        let mut summary = packaged_body_coverage_summary_details();
        summary.bodies.swap(0, 1);

        assert_eq!(
            format_validated_packaged_body_coverage_summary_for_report(&summary),
            "Packaged body set: unavailable (the packaged body coverage summary field `bodies` is out of sync with the current bundled body set)"
        );
    }
}
