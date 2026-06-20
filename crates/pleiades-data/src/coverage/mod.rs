use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use pleiades_backend::EphemerisBackend;
use pleiades_backend::{
    Angle, Apparentness, CelestialBody, CoordinateFrame, EphemerisError, EphemerisRequest,
    EphemerisResult, Instant, JulianDay, TimeRange, TimeScale, ZodiacMode,
};
use pleiades_compression::{
    join_display, ArtifactOutput, ArtifactOutputSupport, ArtifactProfile,
    ArtifactProfileCoverageSummary, ArtifactResidualBodyCoverageSummary, ChannelKind,
    CompressedArtifact, EndianPolicy, PolynomialChannel, Segment, SpeedPolicy,
};
use pleiades_jpl::{
    comparison_snapshot_body_class_coverage_summary, format_reference_snapshot_summary,
    independent_holdout_snapshot_body_class_coverage_summary, production_generation_source_summary,
    production_generation_source_summary_for_report, production_reference_corpus,
    reference_snapshot_summary, selected_asteroid_source_request_corpus_summary,
    JplSnapshotBackend, ProductionGenerationSourceSummary, ReferenceSnapshotSummary,
    SnapshotCorpusBackend,
};

use crate::data::packaged_artifact;
use crate::lookup::{
    packaged_artifact_storage_summary_details, packaged_backend,
    packaged_frame_treatment_summary_details, packaged_lookup_epoch_policy_summary_details,
    packaged_request_policy_summary_details, PackagedArtifactStorageSummary,
    PackagedFrameTreatmentSummary, PackagedLookupEpochPolicy, PackagedRequestPolicySummary,
};
use crate::regenerate::{
    artifact_time_range, packaged_artifact_segment_validation_fractions_for_body,
    polynomial_channel_from_samples, PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS,
};
use crate::{
    packaged_artifact_generation_policy_note_text, packaged_artifact_source_text, packaged_bodies,
    ARTIFACT_LABEL, ARTIFACT_PROFILE_ID,
};

pub(crate) mod body;
pub(crate) mod fit;
pub(crate) mod generation;
pub(crate) mod generation_spec;
pub(crate) mod lsq;
pub(crate) mod profile;
pub(crate) mod regen;
pub(crate) mod target;
pub(crate) mod threshold;

pub use body::*;
pub use fit::*;
pub use generation::*;
pub use generation_spec::*;
pub use lsq::*;
pub use profile::*;
pub use regen::*;
pub use target::*;
pub use threshold::*;

// Re-export pub(crate) items needed by sibling submodules via `use super::*`.
// Items only used within their own submodule or accessible via the submodule
// path do not need to be listed here.
pub(crate) use fit::{
    packaged_artifact_fit_threshold_violations_from_envelope_and_thresholds,
    PackagedArtifactFitChannelFamilyAccumulator, PackagedArtifactFitSample,
    PackagedArtifactFitSegmentFamilyKey, PACKAGED_ARTIFACT_FIT_THRESHOLD_SUMMARY,
};
pub(crate) use profile::fnv1a64;
pub(crate) use regen::{
    format_scope_bodies, packaged_artifact_body_cadence, packaged_artifact_encoded_bytes,
    packaged_artifact_target_threshold_scope_envelope_summary_details, PackagedArtifactBodyCadence,
};
pub(crate) use target::PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES;
#[cfg(test)]
pub(crate) use threshold::packaged_artifact_fit_sample_fractions;
pub(crate) use threshold::{
    channel_from_dense_fit_samples_with_control_points,
    channel_from_fit_samples_with_control_points, distance_channel_from_dense_fit_samples,
    distance_channel_from_fit_samples, distance_channel_from_four_point_control_points,
    distance_channel_from_samples, packaged_artifact_body_scope,
    packaged_artifact_fit_channel_delta, packaged_artifact_fit_envelope_summary_from_samples,
    packaged_artifact_fit_expected_sample_count_with_filter,
    packaged_artifact_fit_outlier_samples_for_current_artifact,
    packaged_artifact_fit_samples_for_current_artifact,
};
