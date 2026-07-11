//! Lunar backend boundary based on a compact pure-Rust analytical model.
//!
//! The first release intentionally keeps a compact Moon-and-lunar-point
//! backend for chart workflows by combining a compact Meeus-style truncated
//! lunar position series with geocentric coordinate transforms, Meeus-style
//! mean node/perigee/apogee formulae, and finite-difference mean-motion
//! estimates. The backend accepts both TT and TDB requests as dynamical-time
//! inputs and still rejects UT-based requests explicitly.
//!
//! The current lunar-theory selection is intentionally explicit: the backend
//! exposes the Moon plus the mean/true node and mean apogee/perigee channels
//! through a small structured specification, and it also lists true apogee /
//! true perigee as unsupported bodies, so future source-backed ELP work can
//! attach provenance, supported channels, unsupported channels, and date-range
//! notes without changing the public API shape. Note: the osculating true
//! apogee/perigee (True Lilith) are now served release-grade by
//! `PackagedDataBackend` ahead of this backend in the routing chain, so the
//! ELP-local `Unsupported` claim for those bodies is no longer a global gap.
//!
//! The current catalog also has
//! typed lookup helpers by source identifier, model name, structured source
//! family, and family label so future source-backed lunar variants can slot
//! into the same resolution path.
//!
//! See `docs/lunar-theory-policy.md` for the current baseline, validation
//! scope, and source/provenance posture.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use core::fmt;

use pleiades_backend::Apparentness;
use pleiades_types::{CelestialBody, CoordinateFrame, Instant, TimeRange, TimeScale, ZodiacMode};

mod backend;
pub(crate) mod catalog;
pub(crate) mod data;
mod evidence;
mod request_policy;
mod series;
mod source;
mod specification;

pub use backend::ElpBackend;

// request_policy re-exports
pub use request_policy::{
    lunar_theory_request_policy, LunarTheoryRequestPolicy, LunarTheoryRequestPolicyValidationError,
};

// source re-exports
pub use source::{
    lunar_theory_source_family, lunar_theory_source_family_summary, lunar_theory_source_selection,
    lunar_theory_source_summary, LunarTheoryCatalogKey, LunarTheorySourceFamily,
    LunarTheorySourceFamilySummary, LunarTheorySourceFamilySummaryValidationError,
    LunarTheorySourceSelection, LunarTheorySourceSelectionValidationError,
    LunarTheorySourceSummary, LunarTheorySourceSummaryValidationError,
};

// specification re-exports
pub use specification::{
    lunar_theory_specification, LunarTheorySpecification, LunarTheorySpecificationValidationError,
};

// catalog re-exports
pub use catalog::{
    current_lunar_theory_catalog_entry, lunar_theory_capability_summary, lunar_theory_catalog,
    lunar_theory_catalog_entry_for_alias, lunar_theory_catalog_entry_for_current_selection,
    lunar_theory_catalog_entry_for_family_label, lunar_theory_catalog_entry_for_key,
    lunar_theory_catalog_entry_for_label, lunar_theory_catalog_entry_for_model_name,
    lunar_theory_catalog_entry_for_selection, lunar_theory_catalog_entry_for_source_family,
    lunar_theory_catalog_entry_for_source_identifier, lunar_theory_catalog_summary,
    lunar_theory_catalog_validation_summary, lunar_theory_limitations_summary,
    lunar_theory_supported_bodies, lunar_theory_unsupported_bodies, resolve_lunar_theory,
    resolve_lunar_theory_by_alias, resolve_lunar_theory_by_family, resolve_lunar_theory_by_key,
    resolve_lunar_theory_by_selection, validate_lunar_theory_catalog, LunarTheoryCapabilitySummary,
    LunarTheoryCapabilitySummaryValidationError, LunarTheoryCatalogEntry,
    LunarTheoryCatalogSummary, LunarTheoryCatalogSummaryValidationError,
    LunarTheoryCatalogValidationError, LunarTheoryCatalogValidationSummary,
    LunarTheoryLimitationsSummary, LunarTheoryLimitationsSummaryValidationError,
};

// Re-exports used only by tests via `use super::*`; gated to avoid unused-import warnings.
#[cfg(test)]
pub(crate) use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
#[cfg(test)]
pub(crate) use pleiades_types::{Angle, EquatorialCoordinates, Latitude};

#[cfg(test)]
pub(crate) use catalog::validate_lunar_theory_catalog_entries;

#[cfg(test)]
pub(crate) use specification::{LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS, LUNAR_THEORY_SPECIFICATION};

#[cfg(test)]
pub(crate) use series::signed_longitude_delta_degrees;

// evidence re-exports
pub use evidence::{
    format_lunar_equatorial_reference_evidence_envelope, format_lunar_reference_evidence_envelope,
    lunar_apparent_comparison_batch_parity_request_corpus,
    lunar_apparent_comparison_batch_parity_requests,
    lunar_apparent_comparison_equatorial_batch_parity_request_corpus,
    lunar_apparent_comparison_equatorial_batch_parity_requests,
    lunar_apparent_comparison_equatorial_request_corpus,
    lunar_apparent_comparison_equatorial_requests, lunar_apparent_comparison_evidence,
    lunar_apparent_comparison_request_corpus, lunar_apparent_comparison_requests,
    lunar_apparent_comparison_summary, lunar_equatorial_reference_batch_parity_request_corpus,
    lunar_equatorial_reference_batch_parity_requests,
    lunar_equatorial_reference_batch_parity_summary,
    lunar_equatorial_reference_batch_request_corpus, lunar_equatorial_reference_batch_requests,
    lunar_equatorial_reference_evidence, lunar_equatorial_reference_evidence_envelope,
    lunar_equatorial_reference_evidence_summary, lunar_equatorial_reference_request_corpus,
    lunar_high_curvature_continuity_batch_parity_request_corpus,
    lunar_high_curvature_continuity_batch_parity_requests,
    lunar_high_curvature_continuity_envelope, lunar_high_curvature_continuity_request_corpus,
    lunar_high_curvature_continuity_requests,
    lunar_high_curvature_equatorial_continuity_batch_parity_request_corpus,
    lunar_high_curvature_equatorial_continuity_batch_parity_requests,
    lunar_high_curvature_equatorial_continuity_envelope,
    lunar_high_curvature_equatorial_continuity_request_corpus,
    lunar_high_curvature_equatorial_continuity_requests,
    lunar_high_curvature_equatorial_request_corpus, lunar_high_curvature_request_corpus,
    lunar_reference_batch_parity_request_corpus, lunar_reference_batch_parity_requests,
    lunar_reference_batch_parity_summary, lunar_reference_batch_request_corpus,
    lunar_reference_batch_requests, lunar_reference_evidence, lunar_reference_evidence_envelope,
    lunar_reference_evidence_summary, lunar_reference_request_corpus,
    lunar_source_window_request_corpus, lunar_source_window_summary, LunarApparentComparisonSample,
    LunarApparentComparisonSummary, LunarEquatorialReferenceBatchParitySummary,
    LunarEquatorialReferenceBatchParitySummaryValidationError,
    LunarEquatorialReferenceEvidenceEnvelope, LunarEquatorialReferenceEvidenceSummary,
    LunarEvidenceEnvelopeValidationError, LunarHighCurvatureContinuityEnvelope,
    LunarHighCurvatureEquatorialContinuityEnvelope, LunarHighCurvatureEvidenceValidationError,
    LunarReferenceBatchParitySummary, LunarReferenceBatchParitySummaryValidationError,
    LunarReferenceEvidenceEnvelope, LunarReferenceEvidenceSummary, LunarReferenceSample,
    LunarSourceWindowSummary, LunarSourceWindowSummaryValidationError,
};

pub(crate) const PACKAGE_NAME: &str = "pleiades-elp";
pub(crate) const J2000: f64 = 2_451_545.0;

// Private formatting helpers shared across modules

pub(crate) fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_bodies(bodies: &[CelestialBody]) -> String {
    join_display(bodies)
}

pub(crate) fn format_frames(frames: &[CoordinateFrame]) -> String {
    join_display(frames)
}

pub(crate) fn format_time_scales(scales: &[TimeScale]) -> String {
    join_display(scales)
}

pub(crate) fn format_zodiac_modes(modes: &[ZodiacMode]) -> String {
    join_display(modes)
}

pub(crate) fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    join_display(modes)
}

pub(crate) fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

pub(crate) fn format_epoch_range(start: Instant, end: Instant) -> String {
    TimeRange::new(Some(start), Some(end)).summary_line()
}

pub(crate) fn mean_value(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    Some(values.iter().copied().sum::<f64>() / values.len() as f64)
}

pub(crate) fn median_value(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let midpoint = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[midpoint - 1] + values[midpoint]) / 2.0)
    } else {
        Some(values[midpoint])
    }
}

pub(crate) fn percentile_value(values: &mut [f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let percentile = percentile.clamp(0.0, 1.0);
    let position = percentile * (values.len().saturating_sub(1)) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        Some(values[lower_index])
    } else {
        let weight = position - lower_index as f64;
        Some(values[lower_index] * (1.0 - weight) + values[upper_index] * weight)
    }
}

// Additional specification-related public items

/// Formats the release-facing one-line summary for a lunar-theory specification.
///
/// Validation and release tooling use this helper so lunar provenance stays
/// owned by the backend crate rather than duplicated in reporting layers.
pub fn format_lunar_theory_specification(theory: &LunarTheorySpecification) -> String {
    let source = theory.source_selection();
    format!(
        "ELP lunar theory specification: {} [{}; family: {}; selected key: {}] ({} supported bodies: {}; {} unsupported bodies: {}); request policy: {}; citation: {}; provenance: {}; redistribution: {}; truncation: {}; units: {}; validation window: {}; date-range note: {}; frame treatment: {}; license: {}",
        theory.model_name,
        source.identifier,
        source.family,
        source.catalog_key(),
        theory.supported_bodies.len(),
        format_bodies(theory.supported_bodies),
        theory.unsupported_bodies.len(),
        format_bodies(theory.unsupported_bodies),
        theory.request_policy.summary_line(),
        source.citation,
        source.material,
        source.redistribution_note,
        theory.truncation_note,
        theory.unit_note,
        theory.validation_window,
        theory.date_range_note,
        theory.frame_note,
        source.license_note,
    )
}

/// Returns the current lunar-theory frame-treatment summary.
pub fn lunar_theory_frame_treatment_summary() -> &'static str {
    lunar_theory_frame_treatment_summary_details().summary_line()
}

/// Returns the structured lunar-theory frame-treatment summary.
pub fn lunar_theory_frame_treatment_summary_details() -> pleiades_backend::FrameTreatmentSummary {
    pleiades_backend::FrameTreatmentSummary::new(lunar_theory_specification().frame_note)
}

#[cfg(test)]
mod tests;
