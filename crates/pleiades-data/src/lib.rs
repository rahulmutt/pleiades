//! Packaged compressed ephemeris backend for the default 1900-2100 range.
//!
//! Wider coverage is available as an opt-in: regenerate the artifact over a
//! custom window with the `generate-artifact <kernel> --out <path>
//! [--start --end]` CLI subcommand.
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

mod accuracy_baseline;
mod backend;
mod coverage;
mod data;
mod lookup;
mod regenerate;
pub mod thresholds;

pub use accuracy_baseline::{
    accuracy_baseline_against, eros_self_consistency_max_longitude_arcsec,
    packaged_artifact_accuracy_baseline, packaged_artifact_accuracy_baseline_summary_for_report,
    BodyChannelError,
};
pub use thresholds::packaged_artifact_thresholds_summary_for_report;
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
    production_generation_source_summary_for_report, production_holdout_corpus, reference_snapshot,
    JplSnapshotBackend,
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
            "Quantized adjacent same-body quadratic windows with longitude-unwrapped planetary fits, with the comparison-body planetary set densely fit from the JPL de440 kernel over the default 1900-2100 coverage window and the constrained asteroid:433-Eros sourced from its committed reference corpus, with point segments only for single-epoch bodies and recursively subdivided quadratic spans for multi-epoch bodies using body-class span caps and measured-fit comparison against the fallback, {}.",
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
mod test_support;
#[cfg(test)]
mod tests;
