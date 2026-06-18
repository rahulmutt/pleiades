use std::collections::HashMap;
use std::sync::OnceLock;
use std::{cmp::Ordering, fmt};

use pleiades_backend::{
    Angle, Apparentness, CelestialBody, CoordinateFrame, CustomBodyId, EclipticCoordinates,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, Instant, JulianDay,
    TimeRange, TimeScale, ZodiacMode,
};
use pleiades_compression::{
    join_display, ArtifactHeader, BodyArtifact, ChannelKind, CompressedArtifact, PolynomialChannel,
    Segment,
};
use pleiades_jpl::{
    production_generation_source_summary, reference_snapshot, reference_snapshot_summary,
    JplSnapshotBackend, SnapshotEntry,
};

use crate::coverage::{
    channel_from_dense_fit_samples_with_control_points,
    channel_from_fit_samples_with_control_points, distance_channel_from_dense_fit_samples,
    distance_channel_from_fit_samples, distance_channel_from_four_point_control_points,
    distance_channel_from_samples, packaged_artifact_body_cadence, PackagedArtifactBodyCadence,
};
use crate::data::{packaged_artifact_bytes, packaged_artifact_from_bytes};
use crate::{packaged_artifact_source_text, packaged_bodies, ARTIFACT_LABEL, AU_IN_KM};

pub(crate) fn build_packaged_artifact() -> CompressedArtifact {
    packaged_artifact_from_bytes(packaged_artifact_bytes())
        .expect("checked-in packaged artifact fixture should decode and validate")
}

pub(crate) fn validate_packaged_artifact_phase1_source_inputs(
) -> Result<(), pleiades_compression::CompressionError> {
    production_generation_source_summary().validate().map_err(|error| {
        pleiades_compression::CompressionError::new(
            pleiades_compression::CompressionErrorKind::InvalidFormat,
            format!(
                "packaged artifact regeneration phase-1 source inputs production-generation source summary is invalid: {error}"
            ),
        )
    })?;

    let reference_snapshot_summary = reference_snapshot_summary().ok_or_else(|| {
        pleiades_compression::CompressionError::new(
            pleiades_compression::CompressionErrorKind::InvalidFormat,
            "packaged artifact regeneration phase-1 source inputs are missing reference snapshot coverage",
        )
    })?;
    reference_snapshot_summary.validate().map_err(|error| {
        pleiades_compression::CompressionError::new(
            pleiades_compression::CompressionErrorKind::InvalidFormat,
            format!(
                "packaged artifact regeneration phase-1 source inputs reference snapshot summary is invalid: {error}"
            ),
        )
    })?;

    Ok(())
}

fn validate_packaged_artifact_reference_snapshot_inputs(
    snapshot: &[SnapshotEntry],
) -> Result<(), pleiades_compression::CompressionError> {
    let reference_snapshot = reference_snapshot();
    if snapshot.len() != reference_snapshot.len() {
        return Err(pleiades_compression::CompressionError::new(
            pleiades_compression::CompressionErrorKind::InvalidFormat,
            format!(
                "packaged artifact regeneration snapshot input length {} does not match the checked-in reference snapshot length {}",
                snapshot.len(),
                reference_snapshot.len()
            ),
        ));
    }

    for (index, (actual, expected)) in snapshot.iter().zip(reference_snapshot).enumerate() {
        if actual != expected {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration snapshot input at index {index} does not match the checked-in reference snapshot: expected {expected:?}; found {actual:?}",
                ),
            ));
        }
    }

    Ok(())
}

/// Rebuilds the packaged artifact from validated JPL reference-snapshot inputs.
///
/// This helper is deterministic and pure Rust so maintainers can regenerate the
/// checked-in fixture without relying on platform-specific tooling. Callers can
/// supply the checked-in reference snapshot slice to make the generation inputs
/// explicit while preserving the same bundled artifact layout.
pub fn try_regenerate_packaged_artifact_from_snapshot(
    snapshot: &[SnapshotEntry],
) -> Result<CompressedArtifact, pleiades_compression::CompressionError> {
    validate_packaged_artifact_phase1_source_inputs()?;
    validate_packaged_artifact_reference_snapshot_inputs(snapshot)?;

    let mut artifact = CompressedArtifact::new(
        ArtifactHeader::new(ARTIFACT_LABEL, packaged_artifact_source_text()),
        packaged_body_artifacts_from_snapshot(snapshot),
    );
    artifact.checksum = artifact
        .checksum()
        .expect("packaged artifact checksum should be reproducible");
    artifact
        .validate()
        .expect("packaged artifact should validate before encoding");
    Ok(artifact)
}

/// Rebuilds the packaged artifact from validated JPL reference-snapshot inputs.
///
/// This helper is deterministic and pure Rust so maintainers can regenerate the
/// checked-in fixture without relying on platform-specific tooling. Callers can
/// supply the checked-in reference snapshot slice to make the generation inputs
/// explicit while preserving the same bundled artifact layout.
pub fn regenerate_packaged_artifact_from_snapshot(
    snapshot: &[SnapshotEntry],
) -> CompressedArtifact {
    try_regenerate_packaged_artifact_from_snapshot(snapshot)
        .expect("checked-in reference snapshot inputs should validate")
}

/// Rebuilds the packaged artifact from the checked-in JPL reference snapshot.
///
/// The regenerated artifact is cached in-process so repeated validation and report paths
/// can reuse the same deterministic reconstruction without paying the full rebuild cost
/// on every call.
pub fn regenerate_packaged_artifact() -> CompressedArtifact {
    static ARTIFACT: OnceLock<CompressedArtifact> = OnceLock::new();

    ARTIFACT
        .get_or_init(|| regenerate_packaged_artifact_from_snapshot(reference_snapshot()))
        .clone()
}

/// Returns the encoded bytes for the regenerated packaged artifact.
///
/// This caches the encoded payload separately so regeneration commands can reuse the same
/// deterministic bytes across repeated sidecar writes without re-encoding the artifact.
pub fn regenerate_packaged_artifact_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();

    BYTES
        .get_or_init(|| {
            regenerate_packaged_artifact()
                .encode()
                .expect("packaged artifact should encode deterministically")
        })
        .as_slice()
}

fn packaged_body_artifacts_from_snapshot(snapshot: &[SnapshotEntry]) -> Vec<BodyArtifact> {
    let mut entries_by_body: HashMap<CelestialBody, Vec<&SnapshotEntry>> = HashMap::new();

    for entry in snapshot {
        entries_by_body
            .entry(entry.body.clone())
            .or_default()
            .push(entry);
    }

    let mut artifacts = Vec::new();
    std::thread::scope(|scope| {
        let mut handles = Vec::new();

        for (body_index, body) in packaged_bodies().iter().cloned().enumerate() {
            let Some(mut entries) = entries_by_body.remove(&body) else {
                continue;
            };

            handles.push(scope.spawn(move || {
                entries.sort_by(|left, right| {
                    left.epoch
                        .julian_day
                        .days()
                        .partial_cmp(&right.epoch.julian_day.days())
                        .unwrap_or(Ordering::Equal)
                });

                let reference_backend = JplSnapshotBackend;
                let segments = body_segments_from_entries(&entries, &reference_backend);

                (body_index, BodyArtifact::new(body, segments))
            }));
        }

        for handle in handles {
            artifacts.push(
                handle
                    .join()
                    .expect("packaged artifact body reconstruction should not panic"),
            );
        }
    });

    artifacts.sort_by_key(|(body_index, _)| *body_index);
    artifacts
        .into_iter()
        .map(|(_, artifact)| artifact)
        .collect()
}

fn body_segments_from_entries(
    entries: &[&SnapshotEntry],
    reference_backend: &JplSnapshotBackend,
) -> Vec<Segment> {
    match entries.len() {
        0 => Vec::new(),
        1 => vec![segment_from_single_entry(entries[0])],
        _ => entries
            .windows(2)
            .flat_map(|window| {
                body_segment_windows_for_interval(window[0], window[1], reference_backend)
            })
            .collect(),
    }
}

pub(crate) fn body_segment_span_limit(body: &CelestialBody) -> f64 {
    match packaged_artifact_body_cadence(body) {
        PackagedArtifactBodyCadence::Luminaries => 256.0,
        PackagedArtifactBodyCadence::InnerPlanets => 384.0,
        PackagedArtifactBodyCadence::OuterPlanets => 768.0,
        PackagedArtifactBodyCadence::Pluto => 1_536.0,
        PackagedArtifactBodyCadence::LunarPoints => 256.0,
        PackagedArtifactBodyCadence::SelectedAsteroids => 256.0,
        PackagedArtifactBodyCadence::CustomBodies => 512.0,
    }
}

const PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO: f64 = 1.1;
const PACKAGED_ARTIFACT_EXTREME_SPLIT_RATIO: f64 = 4.0;
const PACKAGED_ARTIFACT_EXTREME_SPLIT_MIN_SPAN_RATIO: f64 = 3.0;
pub(crate) const PACKAGED_ARTIFACT_LEFT_BIASED_SPLIT_FRACTION: f64 = 0.4;
pub(crate) const PACKAGED_ARTIFACT_RIGHT_BIASED_SPLIT_FRACTION: f64 = 0.6;
pub(crate) const PACKAGED_ARTIFACT_LEFT_EXTREME_SPLIT_FRACTION: f64 = 0.25;
pub(crate) const PACKAGED_ARTIFACT_RIGHT_EXTREME_SPLIT_FRACTION: f64 = 0.75;
pub(crate) const PACKAGED_ARTIFACT_ONE_FIFTH_SPLIT_FRACTION: f64 = 1.0 / 5.0;
pub(crate) const PACKAGED_ARTIFACT_FOUR_FIFTHS_SPLIT_FRACTION: f64 = 4.0 / 5.0;
pub(crate) const PACKAGED_ARTIFACT_ONE_SEVENTH_SPLIT_FRACTION: f64 = 1.0 / 7.0;
pub(crate) const PACKAGED_ARTIFACT_SIX_SEVENTHS_SPLIT_FRACTION: f64 = 6.0 / 7.0;
pub(crate) const PACKAGED_ARTIFACT_ONE_NINTH_SPLIT_FRACTION: f64 = 1.0 / 9.0;
pub(crate) const PACKAGED_ARTIFACT_EIGHT_NINTHS_SPLIT_FRACTION: f64 = 8.0 / 9.0;
pub(crate) const PACKAGED_ARTIFACT_ONE_EIGHTH_SPLIT_FRACTION: f64 = 1.0 / 8.0;
pub(crate) const PACKAGED_ARTIFACT_SEVEN_EIGHTHS_SPLIT_FRACTION: f64 = 7.0 / 8.0;
const PACKAGED_ARTIFACT_SUPER_EXTREME_DENSE_SPLIT_SPAN_RATIO: f64 = 32.0;
const PACKAGED_ARTIFACT_EXTREME_DENSE_SPLIT_SPAN_RATIO: f64 = 16.0;
pub(crate) const PACKAGED_ARTIFACT_ONE_THIRD_SPLIT_FRACTION: f64 = 1.0 / 3.0;
pub(crate) const PACKAGED_ARTIFACT_TWO_THIRD_SPLIT_FRACTION: f64 = 2.0 / 3.0;
pub(crate) const PACKAGED_ARTIFACT_ONE_SIXTH_SPLIT_FRACTION: f64 = 1.0 / 6.0;
const PACKAGED_ARTIFACT_VERY_LONG_DENSE_SPLIT_SPAN_RATIO: f64 = 4.0;
const PACKAGED_ARTIFACT_LONGEST_DENSE_SPLIT_SPAN_RATIO: f64 = 8.0;

#[derive(Clone, Copy)]
pub(crate) struct PackagedArtifactSplitCurvature<'a> {
    pub(crate) start_coordinates: &'a EclipticCoordinates,
    pub(crate) quarter_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) one_fifth_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) one_sixth_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) one_seventh_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) six_sevenths_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) one_ninth_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) eight_ninths_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) one_eighth_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) seven_eighths_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) one_third_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) midpoint_coordinates: &'a EclipticCoordinates,
    pub(crate) two_third_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) four_fifth_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) five_sixth_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) three_quarter_coordinates: Option<&'a EclipticCoordinates>,
    pub(crate) end_coordinates: &'a EclipticCoordinates,
}

fn packaged_artifact_coordinate_step_delta(
    lhs: &EclipticCoordinates,
    rhs: &EclipticCoordinates,
) -> f64 {
    let longitude_delta = Angle::from_degrees(rhs.longitude.degrees() - lhs.longitude.degrees())
        .normalized_signed()
        .degrees()
        .abs();
    let latitude_delta = (rhs.latitude.degrees() - lhs.latitude.degrees()).abs();
    let distance_delta = match (lhs.distance_au, rhs.distance_au) {
        (Some(lhs_distance), Some(rhs_distance)) => (rhs_distance - lhs_distance).abs(),
        _ => 0.0,
    };

    longitude_delta.max(latitude_delta).max(distance_delta)
}

fn packaged_artifact_segment_transition_curvature(
    start: &EclipticCoordinates,
    middle: &EclipticCoordinates,
    end: &EclipticCoordinates,
) -> f64 {
    packaged_artifact_coordinate_step_delta(start, middle)
        .max(packaged_artifact_coordinate_step_delta(middle, end))
}

pub(crate) fn packaged_artifact_split_fraction_for_interval(
    body: &CelestialBody,
    span_days: f64,
    span_limit: f64,
    curvature: PackagedArtifactSplitCurvature<'_>,
) -> f64 {
    if !packaged_artifact_body_cadence(body).uses_dense_sampling() || span_days <= span_limit * 2.0
    {
        return 0.5;
    }

    let (Some(quarter_coordinates), Some(three_quarter_coordinates)) = (
        curvature.quarter_coordinates,
        curvature.three_quarter_coordinates,
    ) else {
        return 0.5;
    };

    let left_curvature = packaged_artifact_segment_transition_curvature(
        curvature.start_coordinates,
        quarter_coordinates,
        curvature.midpoint_coordinates,
    );
    let right_curvature = packaged_artifact_segment_transition_curvature(
        curvature.midpoint_coordinates,
        three_quarter_coordinates,
        curvature.end_coordinates,
    );

    if span_days > span_limit * PACKAGED_ARTIFACT_EXTREME_SPLIT_MIN_SPAN_RATIO {
        if left_curvature > right_curvature * PACKAGED_ARTIFACT_EXTREME_SPLIT_RATIO {
            return PACKAGED_ARTIFACT_LEFT_EXTREME_SPLIT_FRACTION;
        }
        if right_curvature > left_curvature * PACKAGED_ARTIFACT_EXTREME_SPLIT_RATIO {
            return PACKAGED_ARTIFACT_RIGHT_EXTREME_SPLIT_FRACTION;
        }
        if left_curvature > right_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO {
            return PACKAGED_ARTIFACT_LEFT_BIASED_SPLIT_FRACTION;
        }
        if right_curvature > left_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO {
            return PACKAGED_ARTIFACT_RIGHT_BIASED_SPLIT_FRACTION;
        }
    } else {
        if left_curvature > right_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO {
            return PACKAGED_ARTIFACT_LEFT_BIASED_SPLIT_FRACTION;
        }
        if right_curvature > left_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO {
            return PACKAGED_ARTIFACT_RIGHT_BIASED_SPLIT_FRACTION;
        }
    }

    if span_days > span_limit * PACKAGED_ARTIFACT_VERY_LONG_DENSE_SPLIT_SPAN_RATIO {
        if let (Some(one_sixth_coordinates), Some(five_sixth_coordinates)) = (
            curvature.one_sixth_coordinates,
            curvature.five_sixth_coordinates,
        ) {
            let left_sixth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.start_coordinates,
                one_sixth_coordinates,
                curvature.midpoint_coordinates,
            );
            let right_sixth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.midpoint_coordinates,
                five_sixth_coordinates,
                curvature.end_coordinates,
            );

            if left_sixth_curvature > right_sixth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_ONE_SIXTH_SPLIT_FRACTION;
            }
            if right_sixth_curvature > left_sixth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return 5.0 / 6.0;
            }
        }
    }

    let (Some(one_third_coordinates), Some(two_third_coordinates)) = (
        curvature.one_third_coordinates,
        curvature.two_third_coordinates,
    ) else {
        return 0.5;
    };

    let left_third_curvature = packaged_artifact_segment_transition_curvature(
        curvature.start_coordinates,
        one_third_coordinates,
        curvature.midpoint_coordinates,
    );
    let right_third_curvature = packaged_artifact_segment_transition_curvature(
        curvature.midpoint_coordinates,
        two_third_coordinates,
        curvature.end_coordinates,
    );

    if left_third_curvature > right_third_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO {
        return PACKAGED_ARTIFACT_ONE_THIRD_SPLIT_FRACTION;
    }
    if right_third_curvature > left_third_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO {
        return PACKAGED_ARTIFACT_TWO_THIRD_SPLIT_FRACTION;
    }

    if span_days > span_limit * PACKAGED_ARTIFACT_SUPER_EXTREME_DENSE_SPLIT_SPAN_RATIO {
        if let (Some(one_ninth_coordinates), Some(eight_ninths_coordinates)) = (
            curvature.one_ninth_coordinates,
            curvature.eight_ninths_coordinates,
        ) {
            let left_ninth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.start_coordinates,
                one_ninth_coordinates,
                curvature.midpoint_coordinates,
            );
            let right_ninth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.midpoint_coordinates,
                eight_ninths_coordinates,
                curvature.end_coordinates,
            );

            if left_ninth_curvature > right_ninth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_ONE_NINTH_SPLIT_FRACTION;
            }
            if right_ninth_curvature > left_ninth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_EIGHT_NINTHS_SPLIT_FRACTION;
            }
        }

        if let (Some(one_eighth_coordinates), Some(seven_eighths_coordinates)) = (
            curvature.one_eighth_coordinates,
            curvature.seven_eighths_coordinates,
        ) {
            let left_eighth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.start_coordinates,
                one_eighth_coordinates,
                curvature.midpoint_coordinates,
            );
            let right_eighth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.midpoint_coordinates,
                seven_eighths_coordinates,
                curvature.end_coordinates,
            );

            if left_eighth_curvature
                > right_eighth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_ONE_EIGHTH_SPLIT_FRACTION;
            }
            if right_eighth_curvature
                > left_eighth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_SEVEN_EIGHTHS_SPLIT_FRACTION;
            }
        }
    }

    if span_days > span_limit * PACKAGED_ARTIFACT_EXTREME_DENSE_SPLIT_SPAN_RATIO {
        if let (Some(one_seventh_coordinates), Some(six_sevenths_coordinates)) = (
            curvature.one_seventh_coordinates,
            curvature.six_sevenths_coordinates,
        ) {
            let left_seventh_curvature = packaged_artifact_segment_transition_curvature(
                curvature.start_coordinates,
                one_seventh_coordinates,
                curvature.midpoint_coordinates,
            );
            let right_seventh_curvature = packaged_artifact_segment_transition_curvature(
                curvature.midpoint_coordinates,
                six_sevenths_coordinates,
                curvature.end_coordinates,
            );

            if left_seventh_curvature
                > right_seventh_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_ONE_SEVENTH_SPLIT_FRACTION;
            }
            if right_seventh_curvature
                > left_seventh_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_SIX_SEVENTHS_SPLIT_FRACTION;
            }
        }
    }

    if span_days > span_limit * PACKAGED_ARTIFACT_LONGEST_DENSE_SPLIT_SPAN_RATIO {
        if let (Some(one_fifth_coordinates), Some(four_fifth_coordinates)) = (
            curvature.one_fifth_coordinates,
            curvature.four_fifth_coordinates,
        ) {
            let left_fifth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.start_coordinates,
                one_fifth_coordinates,
                curvature.midpoint_coordinates,
            );
            let right_fifth_curvature = packaged_artifact_segment_transition_curvature(
                curvature.midpoint_coordinates,
                four_fifth_coordinates,
                curvature.end_coordinates,
            );

            if left_fifth_curvature > right_fifth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_ONE_FIFTH_SPLIT_FRACTION;
            }
            if right_fifth_curvature > left_fifth_curvature * PACKAGED_ARTIFACT_SPLIT_BALANCE_RATIO
            {
                return PACKAGED_ARTIFACT_FOUR_FIFTHS_SPLIT_FRACTION;
            }
        }
    }

    0.5
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PackagedArtifactSegmentFitError {
    pub(crate) longitude_degrees: f64,
    pub(crate) latitude_degrees: f64,
    pub(crate) distance_au: f64,
}

impl PackagedArtifactSegmentFitError {
    pub(crate) fn max_delta(self) -> f64 {
        self.longitude_degrees
            .max(self.latitude_degrees)
            .max(self.distance_au)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PackagedArtifactFitCandidateScore {
    pub(crate) sample_count: usize,
    pub(crate) complexity: usize,
    pub(crate) error: PackagedArtifactSegmentFitError,
}

impl PackagedArtifactFitCandidateScore {
    pub(crate) fn max_delta(self) -> f64 {
        self.error.max_delta()
    }
}

pub(crate) fn segment_fit_candidate_is_better(
    existing: PackagedArtifactFitCandidateScore,
    candidate: PackagedArtifactFitCandidateScore,
) -> bool {
    match candidate.max_delta().total_cmp(&existing.max_delta()) {
        Ordering::Less => true,
        Ordering::Greater => false,
        Ordering::Equal => match candidate.sample_count.cmp(&existing.sample_count) {
            Ordering::Less => true,
            Ordering::Greater => false,
            Ordering::Equal => candidate.complexity < existing.complexity,
        },
    }
}

// Keep candidate-versus-fallback selection accuracy-first: only prefer a candidate
// when its measured fit is no worse than the fallback reconstruction.
const PACKAGED_ARTIFACT_SEGMENT_FIT_ACCEPTANCE_RATIO: f64 = 1.0;

fn segment_complexity(segment: &Segment) -> usize {
    segment
        .channels
        .iter()
        .chain(segment.residual_channels.iter())
        .map(|channel| channel.coefficients.len())
        .sum()
}

pub(crate) fn segment_error_prefers_candidate(
    candidate_segment: &Segment,
    candidate_error: Option<PackagedArtifactSegmentFitError>,
    fallback_segment: &Segment,
    fallback_error: Option<PackagedArtifactSegmentFitError>,
) -> bool {
    candidate_error.is_some()
        && match (candidate_error, fallback_error) {
            (Some(candidate_error), Some(fallback_error)) => match candidate_error
                .max_delta()
                .total_cmp(&fallback_error.max_delta())
            {
                Ordering::Less => true,
                Ordering::Equal => {
                    segment_complexity(candidate_segment) <= segment_complexity(fallback_segment)
                }
                Ordering::Greater => {
                    candidate_error.max_delta()
                        <= fallback_error.max_delta()
                            * PACKAGED_ARTIFACT_SEGMENT_FIT_ACCEPTANCE_RATIO
                }
            },
            (Some(_), None) => true,
            (None, _) => false,
        }
}

pub(crate) fn packaged_artifact_segment_validation_fractions_for_body(
    body: &CelestialBody,
) -> &'static [f64] {
    if packaged_artifact_body_cadence(body).uses_dense_validation_sampling() {
        PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS
    } else {
        PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS
    }
}

fn packaged_artifact_segment_fit_error(
    body: &CelestialBody,
    segment: &Segment,
    reference_backend: &JplSnapshotBackend,
) -> Option<PackagedArtifactSegmentFitError> {
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new(ARTIFACT_LABEL, packaged_artifact_source_text()),
        vec![BodyArtifact::new(body.clone(), vec![segment.clone()])],
    );
    let span_days = segment.end.julian_day.days() - segment.start.julian_day.days();
    let mut saw_sample = false;
    let mut longitude_degrees: f64 = 0.0;
    let mut latitude_degrees: f64 = 0.0;
    let mut distance_au: f64 = 0.0;

    for fraction in packaged_artifact_segment_validation_fractions_for_body(body) {
        let sample_jd = segment.start.julian_day.days() + span_days * fraction;
        let request = EphemerisRequest {
            body: body.clone(),
            instant: Instant::new(JulianDay::from_days(sample_jd), TimeScale::Tt),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let expected = reference_backend.position(&request).ok()?.ecliptic?;
        let actual = artifact.lookup_ecliptic(body, request.instant).ok()?;
        longitude_degrees = longitude_degrees.max(
            Angle::from_degrees(actual.longitude.degrees() - expected.longitude.degrees())
                .normalized_signed()
                .degrees()
                .abs(),
        );
        latitude_degrees =
            latitude_degrees.max((actual.latitude.degrees() - expected.latitude.degrees()).abs());
        distance_au = distance_au.max((actual.distance_au? - expected.distance_au?).abs());
        saw_sample = true;
    }

    saw_sample.then_some(PackagedArtifactSegmentFitError {
        longitude_degrees,
        latitude_degrees,
        distance_au,
    })
}

fn packaged_artifact_body_class_span_cap_entries() -> Vec<(&'static str, f64)> {
    vec![
        ("luminaries", body_segment_span_limit(&CelestialBody::Sun)),
        (
            "inner planets",
            body_segment_span_limit(&CelestialBody::Mercury),
        ),
        (
            "outer planets",
            body_segment_span_limit(&CelestialBody::Jupiter),
        ),
        ("pluto", body_segment_span_limit(&CelestialBody::Pluto)),
        (
            "lunar points",
            body_segment_span_limit(&CelestialBody::MeanNode),
        ),
        (
            "selected asteroids",
            body_segment_span_limit(&CelestialBody::Ceres),
        ),
        (
            "custom bodies",
            body_segment_span_limit(&CelestialBody::Custom(CustomBodyId::new(
                "catalog",
                "designation",
            ))),
        ),
    ]
}

/// Structured summary for the packaged-artifact body-class span caps.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactBodyClassSpanCapSummary {
    /// Body-class span cap entries in release-facing order.
    pub entries: Vec<(&'static str, f64)>,
}

/// Validation error for a packaged-artifact body-class span cap summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactBodyClassSpanCapSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactBodyClassSpanCapSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact body-class span cap summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactBodyClassSpanCapSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactBodyClassSpanCapSummaryValidationError {}

impl PackagedArtifactBodyClassSpanCapSummary {
    /// Returns the body-class span cap summary as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!("body-class span caps: {}", self.entries_summary_line())
    }

    fn entries_summary_line(&self) -> String {
        let entries = self
            .entries
            .iter()
            .map(|(label, days)| format!("{label}={days:.0} days"))
            .collect::<Vec<_>>();

        join_display(&entries)
    }

    /// Returns the validated body-class span cap summary as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactBodyClassSpanCapSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactBodyClassSpanCapSummaryValidationError> {
        if self.entries != packaged_artifact_body_class_span_cap_entries() {
            return Err(
                PackagedArtifactBodyClassSpanCapSummaryValidationError::FieldOutOfSync {
                    field: "entries",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for PackagedArtifactBodyClassSpanCapSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact body-class span caps summary record.
pub fn packaged_artifact_body_class_span_cap_summary_details(
) -> PackagedArtifactBodyClassSpanCapSummary {
    let summary = PackagedArtifactBodyClassSpanCapSummary {
        entries: packaged_artifact_body_class_span_cap_entries(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact body-class span caps after validating the structured posture.
pub fn packaged_artifact_body_class_span_cap_summary_for_report() -> String {
    let summary = packaged_artifact_body_class_span_cap_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body-class span caps: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact body-class span-cap entries after validating the structured posture.
pub fn packaged_artifact_body_class_span_cap_entries_for_report() -> String {
    let summary = packaged_artifact_body_class_span_cap_summary_details();
    match summary.validated_summary_line() {
        Ok(_) => summary.entries_summary_line(),
        Err(error) => format!("unavailable ({error})"),
    }
}

fn packaged_artifact_body_cadence_counts() -> [(&'static str, usize); 7] {
    let mut counts = [0usize; 7];

    for body in packaged_bodies() {
        match packaged_artifact_body_cadence(body) {
            PackagedArtifactBodyCadence::Luminaries => counts[0] += 1,
            PackagedArtifactBodyCadence::InnerPlanets => counts[1] += 1,
            PackagedArtifactBodyCadence::OuterPlanets => counts[2] += 1,
            PackagedArtifactBodyCadence::Pluto => counts[3] += 1,
            PackagedArtifactBodyCadence::LunarPoints => counts[4] += 1,
            PackagedArtifactBodyCadence::SelectedAsteroids => counts[5] += 1,
            PackagedArtifactBodyCadence::CustomBodies => counts[6] += 1,
        }
    }

    [
        ("luminaries", counts[0]),
        ("inner planets", counts[1]),
        ("outer planets", counts[2]),
        ("pluto", counts[3]),
        ("lunar points", counts[4]),
        ("selected asteroids", counts[5]),
        ("custom bodies", counts[6]),
    ]
}

/// Structured summary for the packaged-artifact body cadence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactBodyCadenceSummary {
    /// Body-cadence entries in release-facing order.
    pub entries: Vec<(&'static str, usize)>,
}

/// Validation error for a packaged-artifact body cadence summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactBodyCadenceSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactBodyCadenceSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact body cadence summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactBodyCadenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactBodyCadenceSummaryValidationError {}

impl PackagedArtifactBodyCadenceSummary {
    /// Returns the body cadence summary as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        let entries = self
            .entries
            .iter()
            .map(|(label, count)| {
                format!(
                    "{label}={count} {}",
                    if *count == 1 { "body" } else { "bodies" }
                )
            })
            .collect::<Vec<_>>();

        format!("body cadence: {}", join_display(&entries))
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactBodyCadenceSummaryValidationError> {
        if self.entries != packaged_artifact_body_cadence_counts().to_vec() {
            return Err(
                PackagedArtifactBodyCadenceSummaryValidationError::FieldOutOfSync {
                    field: "entries",
                },
            );
        }

        Ok(())
    }

    /// Returns the summary line after validating the structured posture.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactBodyCadenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactBodyCadenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact body cadence summary record.
pub fn packaged_artifact_body_cadence_summary_details() -> PackagedArtifactBodyCadenceSummary {
    let summary = PackagedArtifactBodyCadenceSummary {
        entries: packaged_artifact_body_cadence_counts().to_vec(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

fn render_packaged_artifact_body_cadence_summary(
    summary: &PackagedArtifactBodyCadenceSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body cadence: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact body cadence as a compact human-readable line.
pub fn packaged_artifact_body_cadence_summary_for_report() -> String {
    render_packaged_artifact_body_cadence_summary(&packaged_artifact_body_cadence_summary_details())
}

fn body_segment_windows_for_interval(
    start: &SnapshotEntry,
    end: &SnapshotEntry,
    reference_backend: &JplSnapshotBackend,
) -> Vec<Segment> {
    let span_days = end.epoch.julian_day.days() - start.epoch.julian_day.days();
    let span_limit = body_segment_span_limit(&start.body);
    let start_coordinates = coordinates(start);
    let end_coordinates = coordinates(end);
    let start_longitude = start_coordinates.longitude.degrees();
    let end_longitude =
        unwrap_longitude_degrees(start_longitude, end_coordinates.longitude.degrees());
    let start_instant = Instant::new(start.epoch.julian_day, TimeScale::Tt);
    let end_instant = Instant::new(end.epoch.julian_day, TimeScale::Tt);
    let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
        let sample_jd = start.epoch.julian_day.days()
            + (end.epoch.julian_day.days() - start.epoch.julian_day.days()) * fraction;
        let request = EphemerisRequest {
            body: start.body.clone(),
            instant: Instant::new(JulianDay::from_days(sample_jd), TimeScale::Tt),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        reference_backend
            .position(&request)
            .ok()
            .and_then(|result| result.ecliptic)
    };
    let finalize =
        |segment| segment_with_optional_residual_channels(&start.body, segment, reference_backend);
    let candidate = segment_from_pair(start, end, reference_backend);
    let candidate_error = if segment_fits_quantization(&candidate) {
        packaged_artifact_segment_fit_error(&start.body, &candidate, reference_backend)
    } else {
        None
    };

    if span_days <= 1.0 {
        let fallback = segment_from_pair_fallback(
            start_instant,
            end_instant,
            start_longitude,
            end_longitude,
            &start_coordinates,
            &end_coordinates,
            Some(span_days),
            Some(span_limit),
            &sample_fraction,
        );
        let fallback_error = if segment_fits_quantization(&fallback) {
            packaged_artifact_segment_fit_error(&start.body, &fallback, reference_backend)
        } else {
            None
        };

        return if segment_error_prefers_candidate(
            &candidate,
            candidate_error,
            &fallback,
            fallback_error,
        ) {
            vec![finalize(candidate)]
        } else {
            vec![finalize(fallback)]
        };
    }

    if span_days <= span_limit {
        let fallback = segment_from_pair_fallback(
            start_instant,
            end_instant,
            start_longitude,
            end_longitude,
            &start_coordinates,
            &end_coordinates,
            Some(span_days),
            Some(span_limit),
            &sample_fraction,
        );
        let fallback_error = if segment_fits_quantization(&fallback) {
            packaged_artifact_segment_fit_error(&start.body, &fallback, reference_backend)
        } else {
            None
        };

        if segment_error_prefers_candidate(&candidate, candidate_error, &fallback, fallback_error) {
            return vec![finalize(candidate)];
        }
    }

    let midpoint_jd = (start.epoch.julian_day.days() + end.epoch.julian_day.days()) / 2.0;
    if midpoint_jd <= start.epoch.julian_day.days() || midpoint_jd >= end.epoch.julian_day.days() {
        return vec![finalize(segment_from_pair_fallback(
            start_instant,
            end_instant,
            start_longitude,
            end_longitude,
            &start_coordinates,
            &end_coordinates,
            Some(span_days),
            Some(span_limit),
            &sample_fraction,
        ))];
    }
    let Some(midpoint_coordinates) = sample_fraction(0.5) else {
        return vec![finalize(segment_from_pair_fallback(
            start_instant,
            end_instant,
            start_longitude,
            end_longitude,
            &start_coordinates,
            &end_coordinates,
            Some(span_days),
            Some(span_limit),
            &sample_fraction,
        ))];
    };
    let use_curvature_bias = packaged_artifact_body_cadence(&start.body).uses_dense_sampling()
        && span_days > span_limit * 2.0;
    let quarter_coordinates = if use_curvature_bias {
        sample_fraction(0.25)
    } else {
        None
    };
    let one_fifth_coordinates = if use_curvature_bias && span_days > span_limit * 8.0 {
        sample_fraction(1.0 / 5.0)
    } else {
        None
    };
    let one_sixth_coordinates = if use_curvature_bias {
        sample_fraction(1.0 / 6.0)
    } else {
        None
    };
    let one_seventh_coordinates = if use_curvature_bias && span_days > span_limit * 16.0 {
        sample_fraction(1.0 / 7.0)
    } else {
        None
    };
    let six_sevenths_coordinates = if use_curvature_bias && span_days > span_limit * 16.0 {
        sample_fraction(6.0 / 7.0)
    } else {
        None
    };
    let three_quarter_coordinates = if use_curvature_bias {
        sample_fraction(0.75)
    } else {
        None
    };
    let five_sixth_coordinates = if use_curvature_bias {
        sample_fraction(5.0 / 6.0)
    } else {
        None
    };
    let one_ninth_coordinates = if use_curvature_bias && span_days > span_limit * 32.0 {
        sample_fraction(1.0 / 9.0)
    } else {
        None
    };
    let eight_ninths_coordinates = if use_curvature_bias && span_days > span_limit * 32.0 {
        sample_fraction(8.0 / 9.0)
    } else {
        None
    };
    let one_eighth_coordinates = if use_curvature_bias && span_days > span_limit * 128.0 {
        sample_fraction(1.0 / 8.0)
    } else {
        None
    };
    let seven_eighths_coordinates = if use_curvature_bias && span_days > span_limit * 128.0 {
        sample_fraction(7.0 / 8.0)
    } else {
        None
    };
    let one_third_coordinates = if use_curvature_bias {
        sample_fraction(1.0 / 3.0)
    } else {
        None
    };
    let two_third_coordinates = if use_curvature_bias {
        sample_fraction(2.0 / 3.0)
    } else {
        None
    };
    let four_fifth_coordinates = if use_curvature_bias && span_days > span_limit * 8.0 {
        sample_fraction(4.0 / 5.0)
    } else {
        None
    };
    let split_fraction = packaged_artifact_split_fraction_for_interval(
        &start.body,
        span_days,
        span_limit,
        PackagedArtifactSplitCurvature {
            start_coordinates: &start_coordinates,
            quarter_coordinates: quarter_coordinates.as_ref(),
            one_fifth_coordinates: one_fifth_coordinates.as_ref(),
            one_sixth_coordinates: one_sixth_coordinates.as_ref(),
            one_seventh_coordinates: one_seventh_coordinates.as_ref(),
            six_sevenths_coordinates: six_sevenths_coordinates.as_ref(),
            one_ninth_coordinates: one_ninth_coordinates.as_ref(),
            eight_ninths_coordinates: eight_ninths_coordinates.as_ref(),
            one_eighth_coordinates: one_eighth_coordinates.as_ref(),
            seven_eighths_coordinates: seven_eighths_coordinates.as_ref(),
            one_third_coordinates: one_third_coordinates.as_ref(),
            midpoint_coordinates: &midpoint_coordinates,
            two_third_coordinates: two_third_coordinates.as_ref(),
            four_fifth_coordinates: four_fifth_coordinates.as_ref(),
            five_sixth_coordinates: five_sixth_coordinates.as_ref(),
            three_quarter_coordinates: three_quarter_coordinates.as_ref(),
            end_coordinates: &end_coordinates,
        },
    );
    let split_jd = start.epoch.julian_day.days() + span_days * split_fraction;
    if split_jd <= start.epoch.julian_day.days() || split_jd >= end.epoch.julian_day.days() {
        return vec![finalize(segment_from_pair_fallback(
            start_instant,
            end_instant,
            start_longitude,
            end_longitude,
            &start_coordinates,
            &end_coordinates,
            Some(span_days),
            Some(span_limit),
            &sample_fraction,
        ))];
    }
    let split_coordinates = if (split_fraction - 0.5).abs() < f64::EPSILON {
        midpoint_coordinates
    } else {
        let Some(split_coordinates) = sample_fraction(split_fraction) else {
            return vec![finalize(segment_from_pair_fallback(
                start_instant,
                end_instant,
                start_longitude,
                end_longitude,
                &start_coordinates,
                &end_coordinates,
                Some(span_days),
                Some(span_limit),
                &sample_fraction,
            ))];
        };
        split_coordinates
    };

    let split_entry =
        snapshot_entry_from_ecliptic_coordinates(start.body.clone(), split_jd, split_coordinates);

    let mut segments = body_segment_windows_for_interval(start, &split_entry, reference_backend);
    segments.extend(body_segment_windows_for_interval(
        &split_entry,
        end,
        reference_backend,
    ));
    segments.into_iter().map(finalize).collect()
}

fn segment_fits_quantization(segment: &Segment) -> bool {
    segment
        .channels
        .iter()
        .chain(segment.residual_channels.iter())
        .all(|channel| {
            channel_coefficients_fit_quantization(channel.scale_exponent, &channel.coefficients)
        })
}

pub(crate) fn snapshot_entry_from_ecliptic_coordinates(
    body: CelestialBody,
    julian_day: f64,
    coordinates: EclipticCoordinates,
) -> SnapshotEntry {
    let radius_km = coordinates.distance_au.unwrap_or_default() * AU_IN_KM;
    let longitude_radians = coordinates.longitude.degrees().to_radians();
    let latitude_radians = coordinates.latitude.degrees().to_radians();
    let cos_latitude = latitude_radians.cos();
    SnapshotEntry {
        body,
        epoch: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tt),
        x_km: radius_km * cos_latitude * longitude_radians.cos(),
        y_km: radius_km * cos_latitude * longitude_radians.sin(),
        z_km: radius_km * latitude_radians.sin(),
    }
}

fn unwrap_longitude_degrees(reference_degrees: f64, candidate_degrees: f64) -> f64 {
    reference_degrees
        + Angle::from_degrees(candidate_degrees - reference_degrees)
            .normalized_signed()
            .degrees()
}

fn segment_from_single_entry(entry: &SnapshotEntry) -> Segment {
    let coordinates = coordinates(entry);
    Segment::new(
        Instant::new(entry.epoch.julian_day, TimeScale::Tt),
        Instant::new(entry.epoch.julian_day, TimeScale::Tt),
        vec![
            PolynomialChannel::linear(
                ChannelKind::Longitude,
                9,
                coordinates.longitude.degrees(),
                coordinates.longitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::Latitude,
                9,
                coordinates.latitude.degrees(),
                coordinates.latitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::DistanceAu,
                10,
                coordinates.distance_au.unwrap_or_default(),
                coordinates.distance_au.unwrap_or_default(),
            ),
        ],
    )
}

pub(crate) fn segment_from_pair(
    start: &SnapshotEntry,
    end: &SnapshotEntry,
    reference_backend: &JplSnapshotBackend,
) -> Segment {
    let span_days = end.epoch.julian_day.days() - start.epoch.julian_day.days();
    let span_limit = body_segment_span_limit(&start.body);
    let start_coordinates = coordinates(start);
    let end_coordinates = coordinates(end);
    let start_longitude = start_coordinates.longitude.degrees();
    let end_longitude =
        unwrap_longitude_degrees(start_longitude, end_coordinates.longitude.degrees());
    let start_instant = Instant::new(start.epoch.julian_day, TimeScale::Tt);
    let end_instant = Instant::new(end.epoch.julian_day, TimeScale::Tt);
    let sample_fraction = |fraction: f64| -> Option<EclipticCoordinates> {
        let sample_jd = start.epoch.julian_day.days()
            + (end.epoch.julian_day.days() - start.epoch.julian_day.days()) * fraction;
        let request = EphemerisRequest {
            body: start.body.clone(),
            instant: Instant::new(JulianDay::from_days(sample_jd), TimeScale::Tt),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        reference_backend
            .position(&request)
            .ok()
            .and_then(|result| result.ecliptic)
    };

    let finalize =
        |segment| segment_with_optional_residual_channels(&start.body, segment, reference_backend);

    let mut best_candidate: Option<(Segment, PackagedArtifactFitCandidateScore)> = None;
    for sample_count in packaged_artifact_fit_sample_counts_for_body(&start.body) {
        if let Some((segment, error)) = segment_from_pair_fit_attempt(
            start_instant,
            end_instant,
            &start.body,
            &start_coordinates,
            &end_coordinates,
            &sample_fraction,
            reference_backend,
            *sample_count,
        ) {
            let score = PackagedArtifactFitCandidateScore {
                sample_count: *sample_count,
                complexity: segment_complexity(&segment),
                error,
            };
            let should_replace = best_candidate
                .as_ref()
                .map(|(_, existing_score)| segment_fit_candidate_is_better(*existing_score, score))
                .unwrap_or(true);
            if should_replace {
                best_candidate = Some((segment, score));
            }
        }
    }

    let fallback = segment_from_pair_fallback(
        start_instant,
        end_instant,
        start_longitude,
        end_longitude,
        &start_coordinates,
        &end_coordinates,
        Some(span_days),
        Some(span_limit),
        &sample_fraction,
    );
    let fallback_error = if segment_fits_quantization(&fallback) {
        packaged_artifact_segment_fit_error(&start.body, &fallback, reference_backend)
    } else {
        None
    };

    if let Some((candidate, score)) = &best_candidate {
        if segment_error_prefers_candidate(candidate, Some(score.error), &fallback, fallback_error)
        {
            return finalize(candidate.clone());
        }
    }

    finalize(fallback)
}

#[allow(clippy::too_many_arguments)]
fn segment_from_pair_fit_attempt<F>(
    start_instant: Instant,
    end_instant: Instant,
    body: &CelestialBody,
    start_coordinates: &EclipticCoordinates,
    end_coordinates: &EclipticCoordinates,
    sample_fraction: &F,
    reference_backend: &JplSnapshotBackend,
    sample_count: usize,
) -> Option<(Segment, PackagedArtifactSegmentFitError)>
where
    F: Fn(f64) -> Option<EclipticCoordinates>,
{
    let fit_sample_fractions = chebyshev_lobatto_fractions(sample_count);
    let fit_sample_coordinates = fit_sample_fractions
        .iter()
        .map(|fraction| sample_fraction(*fraction))
        .collect::<Option<Vec<_>>>()?;

    let longitude_samples = unwrap_longitude_samples(
        &fit_sample_coordinates
            .iter()
            .map(|coordinates| coordinates.longitude.degrees())
            .collect::<Vec<_>>(),
    );

    let fit_samples = fit_sample_fractions
        .iter()
        .copied()
        .zip(fit_sample_coordinates.iter())
        .collect::<Vec<_>>();

    let longitude_fit_samples = fit_samples
        .iter()
        .enumerate()
        .map(|(index, (fraction, _))| (*fraction, longitude_samples[index]))
        .collect::<Vec<_>>();
    let latitude_fit_samples = fit_samples
        .iter()
        .map(|(fraction, coordinates)| (*fraction, coordinates.latitude.degrees()))
        .collect::<Vec<_>>();

    let (Some(longitude_channel), Some(latitude_channel)) = (
        channel_from_fit_samples_with_control_points(
            ChannelKind::Longitude,
            9,
            &longitude_fit_samples,
        ),
        channel_from_fit_samples_with_control_points(
            ChannelKind::Latitude,
            9,
            &latitude_fit_samples,
        ),
    ) else {
        return None;
    };

    let midpoint_distance_au = sample_fraction(0.5).and_then(|coordinates| coordinates.distance_au);
    let distance_samples = fit_samples
        .iter()
        .filter_map(|(fraction, coordinates)| {
            coordinates
                .distance_au
                .map(|distance| (*fraction, distance))
        })
        .collect::<Vec<_>>();
    let segment = Segment::new(
        start_instant,
        end_instant,
        vec![
            longitude_channel,
            latitude_channel,
            distance_channel_from_fit_samples(
                &distance_samples,
                start_coordinates.distance_au.unwrap_or_default(),
                midpoint_distance_au,
                end_coordinates.distance_au.unwrap_or_default(),
            ),
        ],
    );
    let error = packaged_artifact_segment_fit_error(body, &segment, reference_backend)?;
    Some((segment, error))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn segment_from_pair_fallback(
    start_instant: Instant,
    end_instant: Instant,
    start_longitude: f64,
    end_longitude: f64,
    start_coordinates: &EclipticCoordinates,
    end_coordinates: &EclipticCoordinates,
    span_days: Option<f64>,
    span_limit: Option<f64>,
    sample_fraction: &dyn Fn(f64) -> Option<EclipticCoordinates>,
) -> Segment {
    let Some(midpoint_coordinates) = sample_fraction(0.5) else {
        return Segment::new(
            start_instant,
            end_instant,
            vec![
                PolynomialChannel::linear(
                    ChannelKind::Longitude,
                    9,
                    start_longitude,
                    end_longitude,
                ),
                PolynomialChannel::linear(
                    ChannelKind::Latitude,
                    9,
                    start_coordinates.latitude.degrees(),
                    end_coordinates.latitude.degrees(),
                ),
                distance_channel_from_samples(
                    start_coordinates.distance_au.unwrap_or_default(),
                    None,
                    end_coordinates.distance_au.unwrap_or_default(),
                ),
            ],
        );
    };
    let midpoint_distance_au = midpoint_coordinates.distance_au;
    let distance_start = start_coordinates.distance_au.unwrap_or_default();
    let distance_end = end_coordinates.distance_au.unwrap_or_default();
    let midpoint_longitude =
        unwrap_longitude_degrees(start_longitude, midpoint_coordinates.longitude.degrees());

    let quarter_coordinates = sample_fraction(0.25);
    let three_quarter_coordinates = sample_fraction(0.75);
    if let (Some(quarter_coordinates), Some(three_quarter_coordinates)) = (
        quarter_coordinates.as_ref(),
        three_quarter_coordinates.as_ref(),
    ) {
        if let (
            Some(quarter_distance_au),
            Some(midpoint_distance_au),
            Some(three_quarter_distance_au),
        ) = (
            quarter_coordinates.distance_au,
            midpoint_distance_au,
            three_quarter_coordinates.distance_au,
        ) {
            let longitude_samples = unwrap_longitude_samples(&[
                start_longitude,
                quarter_coordinates.longitude.degrees(),
                midpoint_longitude,
                three_quarter_coordinates.longitude.degrees(),
                end_longitude,
            ]);

            if let (Some(longitude_channel), Some(latitude_channel)) = (
                channel_from_dense_fit_samples_with_control_points(
                    ChannelKind::Longitude,
                    9,
                    &[
                        (0.0, longitude_samples[0]),
                        (0.25, longitude_samples[1]),
                        (0.5, longitude_samples[2]),
                        (0.75, longitude_samples[3]),
                        (1.0, longitude_samples[4]),
                    ],
                ),
                channel_from_dense_fit_samples_with_control_points(
                    ChannelKind::Latitude,
                    9,
                    &[
                        (0.0, start_coordinates.latitude.degrees()),
                        (0.25, quarter_coordinates.latitude.degrees()),
                        (0.5, midpoint_coordinates.latitude.degrees()),
                        (0.75, three_quarter_coordinates.latitude.degrees()),
                        (1.0, end_coordinates.latitude.degrees()),
                    ],
                ),
            ) {
                let distance_channel = distance_channel_from_dense_fit_samples(
                    &[
                        (0.0, distance_start),
                        (0.25, quarter_distance_au),
                        (0.5, midpoint_distance_au),
                        (0.75, three_quarter_distance_au),
                        (1.0, distance_end),
                    ],
                    distance_start,
                    Some(midpoint_distance_au),
                    distance_end,
                );

                return Segment::new(
                    start_instant,
                    end_instant,
                    vec![longitude_channel, latitude_channel, distance_channel],
                );
            }
        }
    }

    if let (Some(span_days), Some(span_limit)) = (span_days, span_limit) {
        if span_days > span_limit * PACKAGED_ARTIFACT_LONGEST_DENSE_SPLIT_SPAN_RATIO {
            let one_fifth_coordinates = sample_fraction(1.0 / 5.0);
            let two_fifth_coordinates = sample_fraction(2.0 / 5.0);
            let three_fifth_coordinates = sample_fraction(3.0 / 5.0);
            let four_fifth_coordinates = sample_fraction(4.0 / 5.0);
            if let (
                Some(one_fifth_coordinates),
                Some(two_fifth_coordinates),
                Some(three_fifth_coordinates),
                Some(four_fifth_coordinates),
            ) = (
                one_fifth_coordinates.as_ref(),
                two_fifth_coordinates.as_ref(),
                three_fifth_coordinates.as_ref(),
                four_fifth_coordinates.as_ref(),
            ) {
                let longitude_samples = unwrap_longitude_samples(&[
                    start_longitude,
                    one_fifth_coordinates.longitude.degrees(),
                    two_fifth_coordinates.longitude.degrees(),
                    three_fifth_coordinates.longitude.degrees(),
                    four_fifth_coordinates.longitude.degrees(),
                    end_longitude,
                ]);

                if let (Some(longitude_channel), Some(latitude_channel)) = (
                    channel_from_fit_samples_with_control_points(
                        ChannelKind::Longitude,
                        9,
                        &[
                            (0.0, longitude_samples[0]),
                            (1.0 / 5.0, longitude_samples[1]),
                            (2.0 / 5.0, longitude_samples[2]),
                            (3.0 / 5.0, longitude_samples[3]),
                            (4.0 / 5.0, longitude_samples[4]),
                            (1.0, longitude_samples[5]),
                        ],
                    ),
                    channel_from_fit_samples_with_control_points(
                        ChannelKind::Latitude,
                        9,
                        &[
                            (0.0, start_coordinates.latitude.degrees()),
                            (1.0 / 5.0, one_fifth_coordinates.latitude.degrees()),
                            (2.0 / 5.0, two_fifth_coordinates.latitude.degrees()),
                            (3.0 / 5.0, three_fifth_coordinates.latitude.degrees()),
                            (4.0 / 5.0, four_fifth_coordinates.latitude.degrees()),
                            (1.0, end_coordinates.latitude.degrees()),
                        ],
                    ),
                ) {
                    if let (
                        Some(one_fifth_distance_au),
                        Some(two_fifth_distance_au),
                        Some(three_fifth_distance_au),
                        Some(four_fifth_distance_au),
                    ) = (
                        one_fifth_coordinates.distance_au,
                        two_fifth_coordinates.distance_au,
                        three_fifth_coordinates.distance_au,
                        four_fifth_coordinates.distance_au,
                    ) {
                        let distance_channel = distance_channel_from_fit_samples(
                            &[
                                (0.0, start_coordinates.distance_au.unwrap_or_default()),
                                (1.0 / 5.0, one_fifth_distance_au),
                                (2.0 / 5.0, two_fifth_distance_au),
                                (3.0 / 5.0, three_fifth_distance_au),
                                (4.0 / 5.0, four_fifth_distance_au),
                                (1.0, end_coordinates.distance_au.unwrap_or_default()),
                            ],
                            start_coordinates.distance_au.unwrap_or_default(),
                            midpoint_coordinates.distance_au,
                            end_coordinates.distance_au.unwrap_or_default(),
                        );

                        return Segment::new(
                            start_instant,
                            end_instant,
                            vec![longitude_channel, latitude_channel, distance_channel],
                        );
                    }
                }
            }
        }

        if span_days > span_limit * PACKAGED_ARTIFACT_SUPER_EXTREME_DENSE_SPLIT_SPAN_RATIO {
            let one_seventh_coordinates = sample_fraction(1.0 / 7.0);
            let two_seventh_coordinates = sample_fraction(2.0 / 7.0);
            let three_seventh_coordinates = sample_fraction(3.0 / 7.0);
            let four_seventh_coordinates = sample_fraction(4.0 / 7.0);
            let five_seventh_coordinates = sample_fraction(5.0 / 7.0);
            let six_seventh_coordinates = sample_fraction(6.0 / 7.0);
            if let (
                Some(one_seventh_coordinates),
                Some(two_seventh_coordinates),
                Some(three_seventh_coordinates),
                Some(four_seventh_coordinates),
                Some(five_seventh_coordinates),
                Some(six_seventh_coordinates),
            ) = (
                one_seventh_coordinates.as_ref(),
                two_seventh_coordinates.as_ref(),
                three_seventh_coordinates.as_ref(),
                four_seventh_coordinates.as_ref(),
                five_seventh_coordinates.as_ref(),
                six_seventh_coordinates.as_ref(),
            ) {
                let longitude_samples = unwrap_longitude_samples(&[
                    start_longitude,
                    one_seventh_coordinates.longitude.degrees(),
                    two_seventh_coordinates.longitude.degrees(),
                    three_seventh_coordinates.longitude.degrees(),
                    four_seventh_coordinates.longitude.degrees(),
                    five_seventh_coordinates.longitude.degrees(),
                    six_seventh_coordinates.longitude.degrees(),
                    end_longitude,
                ]);

                if let (Some(longitude_channel), Some(latitude_channel)) = (
                    channel_from_dense_fit_samples_with_control_points(
                        ChannelKind::Longitude,
                        9,
                        &[
                            (0.0, longitude_samples[0]),
                            (1.0 / 7.0, longitude_samples[1]),
                            (2.0 / 7.0, longitude_samples[2]),
                            (3.0 / 7.0, longitude_samples[3]),
                            (4.0 / 7.0, longitude_samples[4]),
                            (5.0 / 7.0, longitude_samples[5]),
                            (6.0 / 7.0, longitude_samples[6]),
                            (1.0, longitude_samples[7]),
                        ],
                    ),
                    channel_from_dense_fit_samples_with_control_points(
                        ChannelKind::Latitude,
                        9,
                        &[
                            (0.0, start_coordinates.latitude.degrees()),
                            (1.0 / 7.0, one_seventh_coordinates.latitude.degrees()),
                            (2.0 / 7.0, two_seventh_coordinates.latitude.degrees()),
                            (3.0 / 7.0, three_seventh_coordinates.latitude.degrees()),
                            (4.0 / 7.0, four_seventh_coordinates.latitude.degrees()),
                            (5.0 / 7.0, five_seventh_coordinates.latitude.degrees()),
                            (6.0 / 7.0, six_seventh_coordinates.latitude.degrees()),
                            (1.0, end_coordinates.latitude.degrees()),
                        ],
                    ),
                ) {
                    if let (
                        Some(one_seventh_distance_au),
                        Some(two_seventh_distance_au),
                        Some(three_seventh_distance_au),
                        Some(four_seventh_distance_au),
                        Some(five_seventh_distance_au),
                        Some(six_seventh_distance_au),
                    ) = (
                        one_seventh_coordinates.distance_au,
                        two_seventh_coordinates.distance_au,
                        three_seventh_coordinates.distance_au,
                        four_seventh_coordinates.distance_au,
                        five_seventh_coordinates.distance_au,
                        six_seventh_coordinates.distance_au,
                    ) {
                        let distance_channel = distance_channel_from_dense_fit_samples(
                            &[
                                (0.0, start_coordinates.distance_au.unwrap_or_default()),
                                (1.0 / 7.0, one_seventh_distance_au),
                                (2.0 / 7.0, two_seventh_distance_au),
                                (3.0 / 7.0, three_seventh_distance_au),
                                (4.0 / 7.0, four_seventh_distance_au),
                                (5.0 / 7.0, five_seventh_distance_au),
                                (6.0 / 7.0, six_seventh_distance_au),
                                (1.0, end_coordinates.distance_au.unwrap_or_default()),
                            ],
                            start_coordinates.distance_au.unwrap_or_default(),
                            midpoint_coordinates.distance_au,
                            end_coordinates.distance_au.unwrap_or_default(),
                        );

                        return Segment::new(
                            start_instant,
                            end_instant,
                            vec![longitude_channel, latitude_channel, distance_channel],
                        );
                    }
                }
            }
        }
    }

    let Some(first_third_coordinates) = sample_fraction(1.0 / 3.0) else {
        let midpoint_longitude =
            unwrap_longitude_degrees(start_longitude, midpoint_coordinates.longitude.degrees());
        return Segment::new(
            start_instant,
            end_instant,
            vec![
                PolynomialChannel::quadratic(
                    ChannelKind::Longitude,
                    9,
                    start_longitude,
                    midpoint_longitude,
                    end_longitude,
                    0.5,
                ),
                PolynomialChannel::quadratic(
                    ChannelKind::Latitude,
                    9,
                    start_coordinates.latitude.degrees(),
                    midpoint_coordinates.latitude.degrees(),
                    end_coordinates.latitude.degrees(),
                    0.5,
                ),
                distance_channel_from_samples(
                    distance_start,
                    midpoint_coordinates.distance_au,
                    distance_end,
                ),
            ],
        );
    };

    let Some(second_third_coordinates) = sample_fraction(2.0 / 3.0) else {
        let midpoint_longitude =
            unwrap_longitude_degrees(start_longitude, midpoint_coordinates.longitude.degrees());
        return Segment::new(
            start_instant,
            end_instant,
            vec![
                PolynomialChannel::quadratic(
                    ChannelKind::Longitude,
                    9,
                    start_longitude,
                    midpoint_longitude,
                    end_longitude,
                    0.5,
                ),
                PolynomialChannel::quadratic(
                    ChannelKind::Latitude,
                    9,
                    start_coordinates.latitude.degrees(),
                    midpoint_coordinates.latitude.degrees(),
                    end_coordinates.latitude.degrees(),
                    0.5,
                ),
                distance_channel_from_samples(
                    distance_start,
                    midpoint_coordinates.distance_au,
                    distance_end,
                ),
            ],
        );
    };

    let longitude_samples = unwrap_longitude_samples(&[
        start_longitude,
        first_third_coordinates.longitude.degrees(),
        second_third_coordinates.longitude.degrees(),
        end_longitude,
    ]);

    if let (Some(longitude_channel), Some(latitude_channel)) = (
        polynomial_channel_from_samples(
            ChannelKind::Longitude,
            9,
            &[
                (0.0, longitude_samples[0]),
                (1.0 / 3.0, longitude_samples[1]),
                (2.0 / 3.0, longitude_samples[2]),
                (1.0, longitude_samples[3]),
            ],
        ),
        polynomial_channel_from_samples(
            ChannelKind::Latitude,
            9,
            &[
                (0.0, start_coordinates.latitude.degrees()),
                (1.0 / 3.0, first_third_coordinates.latitude.degrees()),
                (2.0 / 3.0, second_third_coordinates.latitude.degrees()),
                (1.0, end_coordinates.latitude.degrees()),
            ],
        ),
    ) {
        let distance_channel = if let (Some(first_third_distance), Some(second_third_distance)) = (
            first_third_coordinates.distance_au,
            second_third_coordinates.distance_au,
        ) {
            distance_channel_from_four_point_control_points(
                distance_start,
                first_third_distance,
                second_third_distance,
                distance_end,
            )
            .unwrap_or_else(|| {
                distance_channel_from_samples(
                    distance_start,
                    midpoint_coordinates.distance_au,
                    distance_end,
                )
            })
        } else {
            distance_channel_from_samples(
                distance_start,
                midpoint_coordinates.distance_au,
                distance_end,
            )
        };

        return Segment::new(
            start_instant,
            end_instant,
            vec![longitude_channel, latitude_channel, distance_channel],
        );
    }

    let midpoint_longitude =
        unwrap_longitude_degrees(start_longitude, midpoint_coordinates.longitude.degrees());

    Segment::new(
        start_instant,
        end_instant,
        vec![
            PolynomialChannel::quadratic(
                ChannelKind::Longitude,
                9,
                start_longitude,
                midpoint_longitude,
                end_longitude,
                0.5,
            ),
            PolynomialChannel::quadratic(
                ChannelKind::Latitude,
                9,
                start_coordinates.latitude.degrees(),
                midpoint_coordinates.latitude.degrees(),
                end_coordinates.latitude.degrees(),
                0.5,
            ),
            distance_channel_from_samples(
                start_coordinates.distance_au.unwrap_or_default(),
                midpoint_distance_au,
                end_coordinates.distance_au.unwrap_or_default(),
            ),
        ],
    )
}

fn segment_with_optional_residual_channels(
    body: &CelestialBody,
    segment: Segment,
    reference_backend: &JplSnapshotBackend,
) -> Segment {
    let Some(base_error) = packaged_artifact_segment_fit_error(body, &segment, reference_backend)
    else {
        return segment;
    };

    let candidate_for_kind = |segment: &Segment,
                              channel_kind: ChannelKind|
     -> Option<(Segment, PackagedArtifactSegmentFitError)> {
        let candidate = residual_segment(body, segment, reference_backend, channel_kind)?;
        let candidate_error =
            packaged_artifact_segment_fit_error(body, &candidate, reference_backend)?;
        Some((candidate, candidate_error))
    };

    let (best_segment, _) = best_residual_segment(
        segment,
        base_error,
        &[
            ChannelKind::Longitude,
            ChannelKind::Latitude,
            ChannelKind::DistanceAu,
        ],
        &candidate_for_kind,
    );

    best_segment
}

fn residual_segment_is_better(
    candidate_segment: &Segment,
    candidate_error: PackagedArtifactSegmentFitError,
    existing_segment: &Segment,
    existing_error: PackagedArtifactSegmentFitError,
) -> bool {
    match candidate_error
        .max_delta()
        .total_cmp(&existing_error.max_delta())
    {
        Ordering::Less => true,
        Ordering::Greater => false,
        Ordering::Equal => match candidate_segment
            .residual_channels
            .len()
            .cmp(&existing_segment.residual_channels.len())
        {
            Ordering::Less => true,
            Ordering::Greater => false,
            Ordering::Equal => {
                let candidate_residual_coefficients = candidate_segment
                    .residual_channels
                    .iter()
                    .map(|channel| channel.coefficients.len())
                    .sum::<usize>();
                let existing_residual_coefficients = existing_segment
                    .residual_channels
                    .iter()
                    .map(|channel| channel.coefficients.len())
                    .sum::<usize>();

                candidate_residual_coefficients < existing_residual_coefficients
            }
        },
    }
}

pub(crate) fn best_residual_segment<F>(
    current_segment: Segment,
    current_error: PackagedArtifactSegmentFitError,
    remaining_kinds: &[ChannelKind],
    candidate_for_kind: &F,
) -> (Segment, PackagedArtifactSegmentFitError)
where
    F: Fn(&Segment, ChannelKind) -> Option<(Segment, PackagedArtifactSegmentFitError)>,
{
    let mut best_segment = current_segment.clone();
    let mut best_error = current_error;

    for kind in remaining_kinds.iter().copied() {
        let Some((candidate_segment, candidate_error)) = candidate_for_kind(&current_segment, kind)
        else {
            continue;
        };

        let next_remaining_kinds = remaining_kinds
            .iter()
            .copied()
            .filter(|candidate_kind| *candidate_kind != kind)
            .collect::<Vec<_>>();

        let (recursive_segment, recursive_error) = best_residual_segment(
            candidate_segment,
            candidate_error,
            &next_remaining_kinds,
            candidate_for_kind,
        );

        if residual_segment_is_better(
            &recursive_segment,
            recursive_error,
            &best_segment,
            best_error,
        ) {
            best_segment = recursive_segment;
            best_error = recursive_error;
        }
    }

    (best_segment, best_error)
}

fn residual_segment(
    body: &CelestialBody,
    segment: &Segment,
    reference_backend: &JplSnapshotBackend,
    kind: ChannelKind,
) -> Option<Segment> {
    if segment
        .residual_channels
        .iter()
        .any(|channel| channel.kind == kind)
    {
        return None;
    }

    let channel = segment
        .channels
        .iter()
        .find(|channel| channel.kind == kind)?;
    let span_days = segment.end.julian_day.days() - segment.start.julian_day.days();
    let residual_samples = packaged_artifact_residual_sample_fractions_for_channel(body, kind)
        .iter()
        .copied()
        .map(|fraction| {
            let sample_jd = segment.start.julian_day.days() + span_days * fraction;
            let request = EphemerisRequest {
                body: body.clone(),
                instant: Instant::new(JulianDay::from_days(sample_jd), TimeScale::Tt),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            };

            let expected = reference_backend.position(&request).ok()?.ecliptic?;
            let x = if span_days == 0.0 {
                0.0
            } else {
                (sample_jd - segment.start.julian_day.days()) / span_days
            };
            let current_value = segment_channel_value(segment, kind, x)?;
            let residual = match kind {
                ChannelKind::Longitude => {
                    Angle::from_degrees(expected.longitude.degrees() - current_value)
                        .normalized_signed()
                        .degrees()
                }
                ChannelKind::Latitude => expected.latitude.degrees() - current_value,
                ChannelKind::DistanceAu => expected.distance_au? - current_value,
                _ => unreachable!("unsupported packaged-artifact channel kind"),
            };
            Some((fraction, residual))
        })
        .collect::<Option<Vec<_>>>()?;

    let residual_channel =
        polynomial_channel_from_samples(kind, channel.scale_exponent, &residual_samples)?;

    let mut residual_channels = segment.residual_channels.clone();
    residual_channels.push(residual_channel);
    residual_channels.sort_by_key(|channel| channel.kind as u8);

    let residual_segment = Segment::with_residual_channels(
        segment.start,
        segment.end,
        segment.channels.clone(),
        residual_channels,
    );

    if segment_fits_quantization(&residual_segment) {
        Some(residual_segment)
    } else {
        None
    }
}

pub(crate) const PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS: &[f64] = &[0.0, 0.25, 0.5, 0.75, 1.0];
pub(crate) const PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS: &[f64] =
    &[0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0];
pub(crate) const PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS: &[f64] =
    &[0.125, 0.25, 0.5, 0.75, 0.875];
pub(crate) const PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS: &[f64] =
    &[0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875];
pub(crate) const PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS: &[usize] = &[6, 8, 10, 12, 14];
pub(crate) const PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS: &[usize] =
    &[6, 8, 10, 12, 14, 16, 18, 20];

pub(crate) fn packaged_artifact_fit_sample_counts_for_body(
    body: &CelestialBody,
) -> &'static [usize] {
    if packaged_artifact_body_cadence(body).uses_dense_sampling() {
        PACKAGED_ARTIFACT_DENSE_FIT_SAMPLE_COUNTS
    } else {
        PACKAGED_ARTIFACT_MEDIUM_FIT_SAMPLE_COUNTS
    }
}

pub(crate) fn packaged_artifact_residual_sample_fractions_for_channel(
    body: &CelestialBody,
    kind: ChannelKind,
) -> &'static [f64] {
    if packaged_artifact_body_cadence(body).uses_dense_residual_sample_lattice(kind) {
        PACKAGED_ARTIFACT_DENSE_RESIDUAL_SAMPLE_FRACTIONS
    } else {
        PACKAGED_ARTIFACT_RESIDUAL_SAMPLE_FRACTIONS
    }
}

pub(crate) fn segment_channel_value(segment: &Segment, kind: ChannelKind, x: f64) -> Option<f64> {
    let base = segment
        .channels
        .iter()
        .find(|channel| channel.kind == kind)?;
    let residual = segment
        .residual_channels
        .iter()
        .find(|channel| channel.kind == kind)
        .map(|channel| evaluate_polynomial_channel(channel, x))
        .unwrap_or(0.0);

    Some(evaluate_polynomial_channel(base, x) + residual)
}

pub(crate) fn evaluate_polynomial_channel(channel: &PolynomialChannel, x: f64) -> f64 {
    let mut result = 0.0;
    let mut power = 1.0;
    for coefficient in &channel.coefficients {
        result += coefficient * power;
        power *= x;
    }
    result
}

pub(crate) fn chebyshev_lobatto_fractions(sample_count: usize) -> Vec<f64> {
    match sample_count {
        0 => Vec::new(),
        1 => vec![0.0],
        _ => (0..sample_count)
            .map(|index| {
                let theta = std::f64::consts::PI * index as f64 / (sample_count - 1) as f64;
                (1.0 - theta.cos()) / 2.0
            })
            .collect(),
    }
}

fn unwrap_longitude_samples(samples: &[f64]) -> Vec<f64> {
    let mut unwrapped = Vec::with_capacity(samples.len());

    for &sample in samples {
        if let Some(&previous) = unwrapped.last() {
            unwrapped.push(unwrap_longitude_degrees(previous, sample));
        } else {
            unwrapped.push(sample);
        }
    }

    unwrapped
}

#[allow(clippy::needless_range_loop)]
fn fit_polynomial_coefficients(samples: &[(f64, f64)]) -> Option<Vec<f64>> {
    let order = samples.len();
    if order == 0 {
        return None;
    }

    let mut matrix = vec![vec![0.0; order + 1]; order];
    for (row, (x, y)) in samples.iter().enumerate() {
        let mut power = 1.0;
        for column in 0..order {
            matrix[row][column] = power;
            power *= *x;
        }
        matrix[row][order] = *y;
    }

    for pivot_index in 0..order {
        let mut best_row = pivot_index;
        let mut best_value = matrix[pivot_index][pivot_index].abs();
        for row in (pivot_index + 1)..order {
            let candidate = matrix[row][pivot_index].abs();
            if candidate > best_value {
                best_value = candidate;
                best_row = row;
            }
        }

        if best_value == 0.0 {
            return None;
        }

        if best_row != pivot_index {
            matrix.swap(pivot_index, best_row);
        }

        let pivot = matrix[pivot_index][pivot_index];
        for column in pivot_index..=order {
            matrix[pivot_index][column] /= pivot;
        }

        for row in 0..order {
            if row == pivot_index {
                continue;
            }

            let factor = matrix[row][pivot_index];
            if factor == 0.0 {
                continue;
            }

            for column in pivot_index..=order {
                matrix[row][column] -= factor * matrix[pivot_index][column];
            }
        }
    }

    Some(matrix.into_iter().map(|row| row[order]).collect())
}

fn channel_coefficients_fit_quantization(scale_exponent: u8, coefficients: &[f64]) -> bool {
    let scale = 10f64.powi(scale_exponent as i32);
    coefficients.iter().all(|coefficient| {
        let scaled = coefficient * scale;
        scaled.is_finite() && scaled.round() >= i64::MIN as f64 && scaled.round() <= i64::MAX as f64
    })
}

pub(crate) fn polynomial_channel_from_samples(
    kind: ChannelKind,
    scale_exponent: u8,
    samples: &[(f64, f64)],
) -> Option<PolynomialChannel> {
    let coefficients = fit_polynomial_coefficients(samples)?;
    if !channel_coefficients_fit_quantization(scale_exponent, &coefficients) {
        return None;
    }

    Some(PolynomialChannel::new(kind, scale_exponent, coefficients))
}

pub(crate) fn coordinates(entry: &SnapshotEntry) -> EclipticCoordinates {
    let radius_km =
        (entry.x_km * entry.x_km + entry.y_km * entry.y_km + entry.z_km * entry.z_km).sqrt();
    let longitude = entry.y_km.atan2(entry.x_km).to_degrees();
    let latitude = (entry.z_km / radius_km)
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();
    EclipticCoordinates::new(
        pleiades_backend::Longitude::from_degrees(longitude),
        pleiades_backend::Latitude::from_degrees(latitude),
        Some(radius_km / AU_IN_KM),
    )
}

pub(crate) fn artifact_time_range(artifact: &CompressedArtifact) -> TimeRange {
    let mut start: Option<Instant> = None;
    let mut end: Option<Instant> = None;
    for body in &artifact.bodies {
        for segment in &body.segments {
            start = Some(match start {
                Some(current) => {
                    if segment.start.julian_day.days() < current.julian_day.days() {
                        segment.start
                    } else {
                        current
                    }
                }
                None => segment.start,
            });
            end = Some(match end {
                Some(current) => {
                    if segment.end.julian_day.days() > current.julian_day.days() {
                        segment.end
                    } else {
                        current
                    }
                }
                None => segment.end,
            });
        }
    }
    TimeRange::new(start, end)
}

pub(crate) fn normalize_lookup_instant(instant: Instant) -> Instant {
    match instant.scale {
        TimeScale::Tt => instant,
        TimeScale::Tdb => Instant::new(instant.julian_day, TimeScale::Tt),
        _ => instant,
    }
}

pub(crate) fn map_artifact_error(error: pleiades_compression::CompressionError) -> EphemerisError {
    let kind = match error.kind {
        pleiades_compression::CompressionErrorKind::MissingBody => {
            EphemerisErrorKind::UnsupportedBody
        }
        pleiades_compression::CompressionErrorKind::OutOfRangeInstant => {
            EphemerisErrorKind::OutOfRangeInstant
        }
        pleiades_compression::CompressionErrorKind::UnsupportedTimeScale => {
            EphemerisErrorKind::UnsupportedTimeScale
        }
        pleiades_compression::CompressionErrorKind::MissingChannel => {
            EphemerisErrorKind::MissingDataset
        }
        pleiades_compression::CompressionErrorKind::QuantizationOverflow
        | pleiades_compression::CompressionErrorKind::InvalidFormat
        | pleiades_compression::CompressionErrorKind::UnsupportedEndianPolicy
        | pleiades_compression::CompressionErrorKind::InvalidMagic
        | pleiades_compression::CompressionErrorKind::UnsupportedVersion
        | pleiades_compression::CompressionErrorKind::ChecksumMismatch
        | pleiades_compression::CompressionErrorKind::Truncated
        | _ => EphemerisErrorKind::NumericalFailure,
    };

    EphemerisError::new(kind, error.message)
}

/// Fits one segment over `[t0_jd, t1_jd]` by sampling `reference` (de440 or a
/// test backend) at the body's within-span sample count and least-squares
/// fitting Longitude/Latitude/DistanceAu channels over the normalized interval.
///
/// The x-domain for the polynomial fit matches the decoder: `x = (t - t0) / span`
/// in `[0, 1]`, consistent with `CompressedArtifact::lookup_ecliptic` (artifact.rs).
///
/// Scale exponents match the existing generation pipeline: Longitude=9, Latitude=9,
/// DistanceAu=10 (see `regenerate.rs` segment_from_single_entry and threshold.rs).
#[allow(dead_code)]
pub(crate) fn fit_segment_within_span(
    body: &CelestialBody,
    t0_jd: f64,
    t1_jd: f64,
    reference: &dyn EphemerisBackend,
) -> Option<Segment> {
    use crate::coverage::{fit_polynomial_lsq, fitting_degree, fitting_within_span_sample_count};

    let n = fitting_within_span_sample_count(body).max(fitting_degree(body) + 1);
    let span = t1_jd - t0_jd;
    if span <= 0.0 {
        return None;
    }
    if n < 2 {
        return None;
    }

    let mut xs = Vec::with_capacity(n);
    let mut lon_deg = Vec::with_capacity(n);
    let mut lat = Vec::with_capacity(n);
    let mut dist = Vec::with_capacity(n);
    for i in 0..n {
        let frac = i as f64 / (n as f64 - 1.0);
        let jd = t0_jd + frac * span;
        let inst = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let res = reference
            .position(&EphemerisRequest::new(body.clone(), inst))
            .ok()?;
        let ec = res.ecliptic?;
        xs.push(frac);
        lon_deg.push(ec.longitude.degrees());
        lat.push(ec.latitude.degrees());
        dist.push(ec.distance_au?);
    }

    // Unwrap longitude to a continuous series before fitting (reuse existing helper).
    let lon_unwrapped = unwrap_longitude_samples(&lon_deg);

    let degree = fitting_degree(body);
    let to_samples =
        |ys: &[f64]| -> Vec<(f64, f64)> { xs.iter().copied().zip(ys.iter().copied()).collect() };

    let lon_coeffs = fit_polynomial_lsq(&to_samples(&lon_unwrapped), degree)?;
    let lat_coeffs = fit_polynomial_lsq(&to_samples(&lat), degree)?;
    let dist_coeffs = fit_polynomial_lsq(&to_samples(&dist), degree)?;

    // Channels must be ordered by ChannelKind discriminant: Longitude=0, Latitude=1, DistanceAu=2.
    // Scale exponents match the existing generation pipeline (Longitude=9, Latitude=9, DistanceAu=10).
    let channels = vec![
        PolynomialChannel::new(ChannelKind::Longitude, 9, lon_coeffs),
        PolynomialChannel::new(ChannelKind::Latitude, 9, lat_coeffs),
        PolynomialChannel::new(ChannelKind::DistanceAu, 10, dist_coeffs),
    ];

    // Validate each channel's coefficients are finite (fail-closed).
    for channel in &channels {
        channel.validate().ok()?;
    }

    let seg = Segment::new(
        Instant::new(JulianDay::from_days(t0_jd), TimeScale::Tdb),
        Instant::new(JulianDay::from_days(t1_jd), TimeScale::Tdb),
        channels,
    );
    Some(seg)
}
