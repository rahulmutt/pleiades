//! JPL Horizons reference fixture backend for validation, comparison, and
//! selected asteroid support.
//!
//! This crate provides a narrow, source-backed backend based on a checked-in
//! JPL Horizons vector fixture. The backend serves exact states at fixture
//! epochs and uses cubic interpolation on four-sample windows when it can,
//! falling back to quadratic interpolation on three-sample windows and linear
//! interpolation between adjacent samples when fewer fixture points are
//! available. The checked-in ecliptic fixture can also be rotated into a
//! mean-obliquity equatorial frame for chart requests that prefer equatorial
//! output. This intentionally small derivative format proves the pure-Rust
//! reader/interpolator path before larger public JPL-derived corpora are added.
//!
//! The checked-in fixture also includes a small set of named asteroids and a
//! custom `catalog:designation` example so the shared body taxonomy can exercise
//! source-backed asteroid support without changing the comparison corpus used by
//! validation reports.

#![forbid(unsafe_code)]

use core::fmt;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    CelestialBody, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, FrameTreatmentSummary, QualityAnnotation,
};
use pleiades_types::{
    Apparentness, CoordinateFrame, CustomBodyId, EclipticCoordinates, Instant, JulianDay, Latitude,
    Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};

const REFERENCE_EPOCH_JD: f64 = 2_451_545.0;
const AU_IN_KM: f64 = 149_597_870.7;

/// Canonical JPL Horizons snapshot instant used by the reference backend.
pub const fn reference_instant() -> Instant {
    Instant::new(JulianDay::from_days(REFERENCE_EPOCH_JD), TimeScale::Tdb)
}

/// The narrow body set covered by the checked-in reference snapshot.
pub fn reference_bodies() -> &'static [pleiades_backend::CelestialBody] {
    snapshot_bodies()
}

/// The instants covered by the checked-in reference snapshot.
pub fn reference_epochs() -> &'static [Instant] {
    snapshot_instants()
}

/// Returns the parsed reference fixture entries.
pub fn reference_snapshot() -> &'static [SnapshotEntry] {
    snapshot_entries().unwrap_or(&[])
}

/// Returns the reference-snapshot request corpus in the requested frame.
///
/// The requests preserve the checked-in row order and stored epochs from the
/// derivative CSV. Callers can reuse this corpus for exact batch checks or
/// retag the returned instants with a different time-scale policy if needed.
pub fn reference_snapshot_requests(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    snapshot_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`reference_snapshot_requests`].
#[doc(alias = "reference_snapshot_requests")]
pub fn reference_snapshot_request_corpus(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_requests(frame)
}

/// Returns the ecliptic reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_requests`].
#[doc(alias = "reference_snapshot_requests")]
pub fn reference_snapshot_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_requests(CoordinateFrame::Ecliptic)
}

/// Returns the ecliptic reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_ecliptic_request_corpus`].
#[doc(alias = "reference_snapshot_ecliptic_request_corpus")]
pub fn reference_snapshot_ecliptic_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_ecliptic_request_corpus()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_requests`].
#[doc(alias = "reference_snapshot_requests")]
pub fn reference_snapshot_equatorial_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_requests(CoordinateFrame::Equatorial)
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_parity_requests")]
pub fn reference_snapshot_equatorial_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_batch_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_batch_parity_requests")]
pub fn reference_snapshot_equatorial_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>>
{
    reference_snapshot_equatorial_batch_parity_requests()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_request_corpus`].
#[doc(alias = "reference_snapshot_equatorial_request_corpus")]
pub fn reference_snapshot_equatorial_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_request_corpus()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_parity_requests")]
pub fn reference_snapshot_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_equatorial_parity_requests`].
#[doc(alias = "reference_snapshot_equatorial_parity_requests")]
pub fn reference_snapshot_equatorial_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_equatorial_parity_requests()
}

/// Returns the mixed-frame reference-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order and alternate between ecliptic
/// and equatorial frames so downstream tooling can reuse the exact release-facing
/// batch shape without reconstructing it from snapshot metadata.
pub fn reference_snapshot_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    snapshot_entries().map(|entries| {
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame: if index % 2 == 0 {
                    CoordinateFrame::Ecliptic
                } else {
                    CoordinateFrame::Equatorial
                },
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the mixed-frame reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_snapshot_batch_parity_requests`].
#[doc(alias = "reference_snapshot_batch_parity_requests")]
pub fn reference_snapshot_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order, keep the ecliptic frame,
/// and alternate TT/TDB labels so downstream tooling can reuse the exact
/// release-facing batch shape without reconstructing it from snapshot metadata.
pub fn reference_snapshot_mixed_time_scale_batch_parity_requests() -> Option<Vec<EphemerisRequest>>
{
    snapshot_entries().map(|entries| {
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(
                    entry.epoch.julian_day,
                    if index % 2 == 0 {
                        TimeScale::Tt
                    } else {
                        TimeScale::Tdb
                    },
                ),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn reference_snapshot_mixed_time_scale_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn reference_snapshot_mixed_tt_tdb_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn reference_snapshot_mixed_tt_tdb_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>>
{
    reference_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_time_scale_request_corpus`].
#[doc(alias = "reference_snapshot_mixed_time_scale_request_corpus")]
pub fn reference_snapshot_mixed_tt_tdb_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB reference-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`reference_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "reference_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn reference_snapshot_mixed_time_scale_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_snapshot_mixed_time_scale_batch_parity_requests()
}

const PRODUCTION_GENERATION_BOUNDARY_COVERAGE: &str =
    "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000";

fn production_generation_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            independent_holdout_snapshot_entries()
                .into_iter()
                .flatten()
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

fn production_generation_snapshot_body_list() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = production_generation_snapshot_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

fn production_generation_boundary_body_list() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = production_generation_boundary_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

fn production_generation_snapshot_bodies() -> &'static [CelestialBody] {
    production_generation_snapshot_body_list()
}

fn extend_unique_snapshot_entries(merged: &mut Vec<SnapshotEntry>, entries: &[SnapshotEntry]) {
    let mut seen = merged
        .iter()
        .map(|entry| {
            (
                entry.body.to_string(),
                entry.epoch.julian_day.days().to_bits(),
            )
        })
        .collect::<BTreeSet<_>>();

    for entry in entries {
        let key = (
            entry.body.to_string(),
            entry.epoch.julian_day.days().to_bits(),
        );
        if seen.insert(key) {
            merged.push(entry.clone());
        }
    }
}

/// Returns the production-generation snapshot corpus used for boundary-coverage checks.
///
/// The merged corpus keeps the checked-in reference rows first, then appends the
/// boundary overlay while skipping overlapping body/epoch pairs that are already
/// covered by the reference snapshot.
pub fn production_generation_snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            let mut merged = Vec::new();
            if let Some(reference_entries) = snapshot_entries() {
                extend_unique_snapshot_entries(&mut merged, reference_entries);
            }
            if let Some(boundary_entries) = production_generation_boundary_entries() {
                extend_unique_snapshot_entries(&mut merged, boundary_entries);
            }
            merged
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Returns the production-generation request corpus in the requested frame.
pub fn production_generation_snapshot_requests(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_snapshot_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`production_generation_snapshot_requests`].
#[doc(alias = "production_generation_snapshot_requests")]
pub fn production_generation_snapshot_request_corpus(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_snapshot_requests(frame)
}

/// Returns the production-generation boundary-overlay request corpus in the requested frame.
pub fn production_generation_boundary_requests(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_boundary_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`production_generation_boundary_requests`].
#[doc(alias = "production_generation_boundary_requests")]
pub fn production_generation_boundary_request_corpus(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    production_generation_boundary_requests(frame)
}

/// A compact coverage summary for the production-generation boundary overlay.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProductionGenerationBoundarySummary {
    /// Total number of parsed boundary-overlay rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the boundary overlay.
    pub body_count: usize,
    /// Bodies covered by the boundary overlay in first-seen order.
    pub bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs covered by the boundary overlay.
    pub epoch_count: usize,
    /// Earliest epoch represented in the boundary overlay.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the boundary overlay.
    pub latest_epoch: Instant,
}

/// Structured validation errors for the production-generation boundary overlay summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationBoundarySummaryValidationError {
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary body order drifted from the checked-in boundary overlay.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl ProductionGenerationBoundarySummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ProductionGenerationBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match body list length {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(f, "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(f, "body order mismatch at index {index}: expected {expected}, found {found}"),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "epoch range {}..{} is invalid",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundarySummaryValidationError {}

impl ProductionGenerationBoundarySummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ProductionGenerationBoundarySummaryValidationError> {
        if self.body_count == 0 {
            return Err(ProductionGenerationBoundarySummaryValidationError::MissingBodies);
        }
        if self.bodies.is_empty() {
            return Err(ProductionGenerationBoundarySummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                ProductionGenerationBoundarySummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    ProductionGenerationBoundarySummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .unwrap(),
                        second_index: index,
                        body: body.to_string(),
                    },
                );
            }
        }

        let expected_bodies = production_generation_boundary_body_list();
        if self.bodies != expected_bodies {
            let mismatch_index = self
                .bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.bodies.len().min(expected_bodies.len()));
            return Err(
                ProductionGenerationBoundarySummaryValidationError::BodyOrderMismatch {
                    index: mismatch_index,
                    expected: expected_bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of boundary body list>".to_string()),
                    found: self
                        .bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of summary body list>".to_string()),
                },
            );
        }

        if self.epoch_count == 0 {
            return Err(ProductionGenerationBoundarySummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                ProductionGenerationBoundarySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        if production_generation_boundary_summary().as_ref() != Some(self) {
            return Err(ProductionGenerationBoundarySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Production generation boundary overlay: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(self.bodies),
        )
    }

    /// Returns a compact summary line after validating the boundary overlay summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a compact coverage summary for the production-generation boundary overlay.
pub fn production_generation_boundary_summary() -> Option<ProductionGenerationBoundarySummary> {
    static SUMMARY: OnceLock<ProductionGenerationBoundarySummary> = OnceLock::new();
    Some(*SUMMARY.get_or_init(|| {
        let entries = production_generation_boundary_entries()
            .expect("production generation boundary entries should exist");

        let mut earliest_epoch = entries[0].epoch;
        let mut latest_epoch = entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        ProductionGenerationBoundarySummary {
            row_count: entries.len(),
            body_count: production_generation_boundary_body_list().len(),
            bodies: production_generation_boundary_body_list(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        }
    }))
}

/// Formats the production-generation boundary overlay for release-facing reporting.
pub fn format_production_generation_boundary_summary(
    summary: &ProductionGenerationBoundarySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing production-generation boundary overlay summary string.
pub fn production_generation_boundary_summary_for_report() -> String {
    match production_generation_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Production generation boundary overlay: unavailable ({error})"),
        },
        None => "Production generation boundary overlay: unavailable".to_string(),
    }
}

/// Returns the provenance summary shared by the production-generation boundary overlay.
#[doc(alias = "independent_holdout_source_summary")]
pub fn production_generation_boundary_source_summary() -> IndependentHoldoutSourceSummary {
    independent_holdout_source_summary()
}

/// Formats the provenance summary for the production-generation boundary overlay.
pub fn format_production_generation_boundary_source_summary(
    summary: &IndependentHoldoutSourceSummary,
) -> String {
    format!(
        "Production generation boundary overlay source: {}; coverage={}; columns={}",
        summary.source, summary.coverage, summary.columns
    )
}

/// Returns the release-facing provenance summary string for the production-generation boundary overlay.
pub fn production_generation_boundary_source_summary_for_report() -> String {
    let summary = production_generation_boundary_source_summary();
    match summary.validate() {
        Ok(()) => format_production_generation_boundary_source_summary(&summary),
        Err(error) => {
            format!("Production generation boundary overlay source: unavailable ({error})")
        }
    }
}

/// A single body-window slice inside the production-generation boundary overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryWindow {
    /// The boundary-overlay body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ProductionGenerationBoundaryWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

/// Compact release-facing summary for the production-generation boundary windows.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryWindowSummary {
    /// Number of boundary-overlay samples in the expanded source slice.
    pub sample_count: usize,
    /// Bodies covered by the expanded source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the expanded source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the expanded source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the expanded source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ProductionGenerationBoundaryWindow>,
}

/// Structured validation errors for a production-generation boundary window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductionGenerationBoundaryWindowSummaryValidationError {
    /// The summary no longer matches the checked-in boundary window slice.
    DerivedSummaryMismatch,
}

impl fmt::Display for ProductionGenerationBoundaryWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DerivedSummaryMismatch => write!(
                f,
                "the production-generation boundary window summary is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundaryWindowSummaryValidationError {}

impl ProductionGenerationBoundaryWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ProductionGenerationBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Production generation boundary windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the summary still matches the checked-in boundary window slice.
    pub fn validate(&self) -> Result<(), ProductionGenerationBoundaryWindowSummaryValidationError> {
        let Some(expected) = production_generation_boundary_window_summary_details() else {
            return Err(
                ProductionGenerationBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        };

        if self != &expected {
            return Err(
                ProductionGenerationBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns the validated summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundaryWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundaryWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn production_generation_boundary_window_summary_details(
) -> Option<ProductionGenerationBoundaryWindowSummary> {
    let entries = production_generation_boundary_entries()?;
    let mut windows = Vec::new();
    for body in production_generation_boundary_body_list() {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(ProductionGenerationBoundaryWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("production generation boundary windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("production generation boundary windows should not be empty after collection");

    Some(ProductionGenerationBoundaryWindowSummary {
        sample_count: entries.len(),
        sample_bodies: production_generation_boundary_body_list().to_vec(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the production-generation boundary windows.
pub fn production_generation_boundary_window_summary(
) -> Option<ProductionGenerationBoundaryWindowSummary> {
    static SUMMARY: OnceLock<ProductionGenerationBoundaryWindowSummary> = OnceLock::new();
    Some(
        SUMMARY
            .get_or_init(|| {
                production_generation_boundary_window_summary_details()
                    .expect("production generation boundary windows should exist")
            })
            .clone(),
    )
}

/// Returns the release-facing production-generation boundary window summary string.
pub fn production_generation_boundary_window_summary_for_report() -> String {
    match production_generation_boundary_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Production generation boundary windows: unavailable ({error})"),
        },
        None => "Production generation boundary windows: unavailable".to_string(),
    }
}

/// A compact body-class coverage summary for the production-generation boundary overlay used by validation and generation tooling.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryBodyClassCoverageSummary {
    /// Number of rows in the boundary overlay.
    pub row_count: usize,
    /// Number of major-body rows in the boundary overlay.
    pub major_body_row_count: usize,
    /// Major bodies covered by the boundary overlay in first-seen order.
    pub major_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the major-body subset.
    pub major_epoch_count: usize,
    /// Per-body windows covered by the major-body subset in first-seen order.
    pub major_windows: Vec<ProductionGenerationBoundaryWindow>,
    /// Number of selected-asteroid rows in the boundary overlay.
    pub asteroid_row_count: usize,
    /// Selected asteroids covered by the boundary overlay in first-seen order.
    pub asteroid_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the selected-asteroid subset.
    pub asteroid_epoch_count: usize,
    /// Per-body windows covered by the selected-asteroid subset in first-seen order.
    pub asteroid_windows: Vec<ProductionGenerationBoundaryWindow>,
}

/// Validation error for a production-generation boundary body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in boundary body-class coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the production-generation boundary body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError {}

impl ProductionGenerationBoundaryBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let major_windows = self
            .major_windows
            .iter()
            .map(ProductionGenerationBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let asteroid_windows = self
            .asteroid_windows
            .iter()
            .map(ProductionGenerationBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Production generation boundary body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
            self.major_body_row_count,
            self.major_bodies.len(),
            self.major_epoch_count,
            major_windows,
            self.asteroid_row_count,
            self.asteroid_bodies.len(),
            self.asteroid_epoch_count,
            asteroid_windows,
        )
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError> {
        let Some(expected) = production_generation_boundary_body_class_coverage_summary_details()
        else {
            return Err(
                ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self != &expected {
            return Err(
                ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundaryBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundaryBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn production_generation_boundary_body_class_coverage_summary_details(
) -> Option<ProductionGenerationBoundaryBodyClassCoverageSummary> {
    let summary = production_generation_boundary_summary()?;
    let source_windows = production_generation_boundary_window_summary_details()?;
    let entries = production_generation_boundary_entries()?;

    let mut major_body_row_count = 0usize;
    let mut major_epochs = BTreeSet::new();
    let mut asteroid_row_count = 0usize;
    let mut asteroid_epochs = BTreeSet::new();

    for entry in entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        if is_comparison_body(&entry.body) {
            major_body_row_count += 1;
            major_epochs.insert(epoch_bits);
        }
        if is_reference_asteroid(&entry.body) {
            asteroid_row_count += 1;
            asteroid_epochs.insert(epoch_bits);
        }
    }

    Some(ProductionGenerationBoundaryBodyClassCoverageSummary {
        row_count: summary.row_count,
        major_body_row_count,
        major_bodies: summary
            .bodies
            .iter()
            .filter(|body| is_comparison_body(body))
            .cloned()
            .collect(),
        major_epoch_count: major_epochs.len(),
        major_windows: source_windows
            .windows
            .iter()
            .filter(|window| is_comparison_body(&window.body))
            .cloned()
            .collect(),
        asteroid_row_count,
        asteroid_bodies: summary
            .bodies
            .iter()
            .filter(|body| is_reference_asteroid(body))
            .cloned()
            .collect(),
        asteroid_epoch_count: asteroid_epochs.len(),
        asteroid_windows: source_windows
            .windows
            .iter()
            .filter(|window| is_reference_asteroid(&window.body))
            .cloned()
            .collect(),
    })
}

/// Returns the compact body-class coverage summary for the production-generation boundary overlay.
pub fn production_generation_boundary_body_class_coverage_summary(
) -> Option<ProductionGenerationBoundaryBodyClassCoverageSummary> {
    production_generation_boundary_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the production-generation boundary overlay.
pub fn production_generation_boundary_body_class_coverage_summary_for_report() -> String {
    match production_generation_boundary_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Production generation boundary body-class coverage: unavailable ({error})")
            }
        },
        None => "Production generation boundary body-class coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the production-generation boundary request corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationBoundaryRequestCorpusSummary {
    /// Total number of generated requests.
    pub request_count: usize,
    /// Number of distinct bodies covered by the request corpus.
    pub body_count: usize,
    /// Bodies covered by the request corpus in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the request corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the request corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the request corpus.
    pub latest_epoch: Instant,
    /// Coordinate frame requested by the corpus.
    pub frame: CoordinateFrame,
    /// Time scale requested by the corpus.
    pub time_scale: TimeScale,
    /// Zodiac mode requested by the corpus.
    pub zodiac_mode: ZodiacMode,
    /// Apparentness requested by the corpus.
    pub apparentness: Apparentness,
}

/// Structured validation errors for the production-generation boundary request corpus summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationBoundaryRequestCorpusSummaryValidationError {
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary body order drifted from the checked-in boundary request corpus.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl ProductionGenerationBoundaryRequestCorpusSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ProductionGenerationBoundaryRequestCorpusSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match body list length {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(f, "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(f, "body order mismatch at index {index}: expected {expected}, found {found}"),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "epoch range {}..{} is invalid",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ProductionGenerationBoundaryRequestCorpusSummaryValidationError {}

impl ProductionGenerationBoundaryRequestCorpusSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Production generation boundary request corpus: {} requests (frame={}; time scale={}; zodiac mode={}; apparentness={}; observerless) across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.request_count,
            self.frame,
            self.time_scale,
            self.zodiac_mode,
            self.apparentness,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current boundary request corpus.
    pub fn validate(
        &self,
    ) -> Result<(), ProductionGenerationBoundaryRequestCorpusSummaryValidationError> {
        let Some(expected) =
            production_generation_boundary_request_corpus_summary_details(self.frame)
        else {
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::DerivedSummaryMismatch);
        };

        if self.request_count != expected.request_count {
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::DerivedSummaryMismatch);
        }
        if self.body_count != expected.body_count {
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::BodyCountMismatch {
                body_count: self.body_count,
                bodies_len: self.bodies.len(),
            });
        }
        if self.bodies.as_slice() != expected.bodies.as_slice() {
            for (index, (expected, found)) in
                expected.bodies.iter().zip(self.bodies.iter()).enumerate()
            {
                if expected != found {
                    return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::BodyOrderMismatch {
                        index,
                        expected: expected.to_string(),
                        found: found.to_string(),
                    });
                }
            }
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::BodyCountMismatch {
                body_count: self.body_count,
                bodies_len: self.bodies.len(),
            });
        }
        if self.epoch_count != expected.epoch_count {
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::DerivedSummaryMismatch);
        }
        if self.earliest_epoch != expected.earliest_epoch
            || self.latest_epoch != expected.latest_epoch
        {
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::InvalidEpochRange {
                earliest_epoch: self.earliest_epoch,
                latest_epoch: self.latest_epoch,
            });
        }
        if self.time_scale != expected.time_scale
            || self.zodiac_mode != expected.zodiac_mode
            || self.apparentness != expected.apparentness
        {
            return Err(ProductionGenerationBoundaryRequestCorpusSummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current request corpus.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationBoundaryRequestCorpusSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationBoundaryRequestCorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn production_generation_boundary_request_corpus_summary_details(
    frame: CoordinateFrame,
) -> Option<ProductionGenerationBoundaryRequestCorpusSummary> {
    let entries = production_generation_boundary_entries()?;
    let requests = production_generation_boundary_requests(frame)?;
    if requests.is_empty() {
        return None;
    }

    let mut bodies = Vec::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = requests[0].instant;
    let mut latest_epoch = requests[0].instant;
    let time_scale = requests[0].instant.scale;

    for (request, entry) in requests.iter().zip(entries.iter()) {
        if request.body != entry.body
            || request.instant.julian_day.days() != entry.epoch.julian_day.days()
            || request.instant.scale != time_scale
            || request.frame != frame
            || request.zodiac_mode != ZodiacMode::Tropical
            || request.apparent != Apparentness::Mean
            || request.observer.is_some()
        {
            return None;
        }

        if !bodies.contains(&request.body) {
            bodies.push(request.body.clone());
        }
        epochs.insert(request.instant.julian_day.days().to_bits());
        if request.instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = request.instant;
        }
        if request.instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = request.instant;
        }
    }

    Some(ProductionGenerationBoundaryRequestCorpusSummary {
        request_count: requests.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
        frame,
        time_scale,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
    })
}

/// Returns the production-generation boundary request corpus summary in the requested frame.
pub fn production_generation_boundary_request_corpus_summary(
    frame: CoordinateFrame,
) -> Option<ProductionGenerationBoundaryRequestCorpusSummary> {
    static ECLIPTIC_SUMMARY: OnceLock<ProductionGenerationBoundaryRequestCorpusSummary> =
        OnceLock::new();
    if frame == CoordinateFrame::Ecliptic {
        Some(
            ECLIPTIC_SUMMARY
                .get_or_init(|| {
                    production_generation_boundary_request_corpus_summary_details(frame)
                        .expect("production generation boundary request corpus should exist")
                })
                .clone(),
        )
    } else {
        production_generation_boundary_request_corpus_summary_details(frame)
    }
}

/// Formats the production-generation boundary request corpus for release-facing reporting.
pub fn format_production_generation_boundary_request_corpus_summary(
    summary: &ProductionGenerationBoundaryRequestCorpusSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing production-generation boundary request corpus summary string.
pub fn production_generation_boundary_request_corpus_summary_for_report() -> String {
    match production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic) {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Production generation boundary request corpus: unavailable ({error})")
            }
        },
        None => "Production generation boundary request corpus: unavailable".to_string(),
    }
}

/// A compact coverage summary for the checked-in reference snapshot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceSnapshotSummary {
    /// Total number of parsed snapshot rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the snapshot.
    pub body_count: usize,
    /// Bodies covered by the snapshot in first-seen order.
    pub bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs covered by the snapshot.
    pub epoch_count: usize,
    /// Number of rows that belong to the reference asteroid subset.
    pub asteroid_row_count: usize,
    /// Earliest epoch represented in the snapshot.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the snapshot.
    pub latest_epoch: Instant,
}

/// Structured validation errors for a reference snapshot coverage summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotSummaryValidationError {
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary body order drifted from the checked-in reference snapshot.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The asteroid row count exceeded the total row count.
    AsteroidRowCountExceedsRowCount {
        asteroid_row_count: usize,
        row_count: usize,
    },
    /// The summary's epoch count drifted away from the checked-in derived evidence.
    EpochCountMismatch {
        epoch_count: usize,
        derived_epoch_count: usize,
    },
    /// The summary's asteroid row count drifted away from the checked-in derived evidence.
    AsteroidRowCountMismatch {
        asteroid_row_count: usize,
        derived_asteroid_row_count: usize,
    },
    /// The summary's epoch range drifted away from the checked-in derived evidence.
    EpochRangeMismatch {
        earliest_epoch: Instant,
        latest_epoch: Instant,
        derived_earliest_epoch: Instant,
        derived_latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl ReferenceSnapshotSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::AsteroidRowCountExceedsRowCount { .. } => "asteroid row count exceeds row count",
            Self::EpochCountMismatch { .. } => "epoch count mismatch",
            Self::AsteroidRowCountMismatch { .. } => "asteroid row count mismatch",
            Self::EpochRangeMismatch { .. } => "epoch range mismatch",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ReferenceSnapshotSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBodies => f.write_str(self.label()),
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(
                f,
                "body count {body_count} does not match body list length {bodies_len}"
            ),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(
                f,
                "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "body order mismatch at index {index}: expected '{expected}' but found '{found}'"
            ),
            Self::MissingEpochs => f.write_str(self.label()),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range: earliest {} is after latest {}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            Self::AsteroidRowCountExceedsRowCount {
                asteroid_row_count,
                row_count,
            } => write!(
                f,
                "asteroid row count {asteroid_row_count} exceeds row count {row_count}"
            ),
            Self::EpochCountMismatch {
                epoch_count,
                derived_epoch_count,
            } => write!(
                f,
                "epoch count {epoch_count} does not match derived epoch count {derived_epoch_count}"
            ),
            Self::AsteroidRowCountMismatch {
                asteroid_row_count,
                derived_asteroid_row_count,
            } => write!(
                f,
                "asteroid row count {asteroid_row_count} does not match derived asteroid row count {derived_asteroid_row_count}"
            ),
            Self::EpochRangeMismatch {
                earliest_epoch,
                latest_epoch,
                derived_earliest_epoch,
                derived_latest_epoch,
            } => write!(
                f,
                "epoch range {}..{} does not match derived range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
                format_instant(*derived_earliest_epoch),
                format_instant(*derived_latest_epoch),
            ),
            Self::DerivedSummaryMismatch => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ReferenceSnapshotSummaryValidationError {}

/// Returns a compact coverage summary for the checked-in reference snapshot.
pub fn reference_snapshot_summary() -> Option<ReferenceSnapshotSummary> {
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut asteroid_row_count = 0usize;
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;
    let reference_asteroids = reference_asteroids();

    for entry in entries {
        bodies.insert(entry.body.to_string());
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if reference_asteroids.contains(&entry.body) {
            asteroid_row_count += 1;
        }
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }
    }

    Some(ReferenceSnapshotSummary {
        row_count: entries.len(),
        body_count: bodies.len(),
        bodies: snapshot_bodies(),
        epoch_count: epochs.len(),
        asteroid_row_count,
        earliest_epoch,
        latest_epoch,
    })
}

/// A compact body-class coverage summary for the checked-in reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotBodyClassCoverageSummary {
    /// Number of major-body rows in the checked-in reference snapshot.
    pub major_body_row_count: usize,
    /// Major bodies covered by the checked-in reference snapshot in first-seen order.
    pub major_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the major-body subset.
    pub major_epoch_count: usize,
    /// Per-body windows covered by the major-body subset in first-seen order.
    pub major_windows: Vec<ReferenceSnapshotSourceWindow>,
    /// Number of selected-asteroid rows in the checked-in reference snapshot.
    pub asteroid_row_count: usize,
    /// Selected asteroids covered by the checked-in reference snapshot in first-seen order.
    pub asteroid_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the selected-asteroid subset.
    pub asteroid_epoch_count: usize,
    /// Per-body windows covered by the selected-asteroid subset in first-seen order.
    pub asteroid_windows: Vec<ReferenceSnapshotSourceWindow>,
}

/// Validation error for a reference snapshot body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceSnapshotBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in body-class coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceSnapshotBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference snapshot body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotBodyClassCoverageSummaryValidationError {}

impl ReferenceSnapshotBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let major_windows = self
            .major_windows
            .iter()
            .map(ReferenceSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let asteroid_windows = self
            .asteroid_windows
            .iter()
            .map(ReferenceSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Reference snapshot body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
            self.major_body_row_count,
            self.major_bodies.len(),
            self.major_epoch_count,
            major_windows,
            self.asteroid_row_count,
            self.asteroid_bodies.len(),
            self.asteroid_epoch_count,
            asteroid_windows,
        )
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotBodyClassCoverageSummaryValidationError> {
        let Some(expected) = reference_snapshot_body_class_coverage_summary_details() else {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_body_row_count",
                },
            );
        };

        if self.major_body_row_count != expected.major_body_row_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_body_row_count",
                },
            );
        }
        if self.major_bodies != expected.major_bodies {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_bodies",
                },
            );
        }
        if self.major_epoch_count != expected.major_epoch_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_epoch_count",
                },
            );
        }
        if self.major_windows != expected.major_windows {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_windows",
                },
            );
        }
        if self.asteroid_row_count != expected.asteroid_row_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_row_count",
                },
            );
        }
        if self.asteroid_bodies != expected.asteroid_bodies {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_bodies",
                },
            );
        }
        if self.asteroid_epoch_count != expected.asteroid_epoch_count {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_epoch_count",
                },
            );
        }
        if self.asteroid_windows != expected.asteroid_windows {
            return Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_body_class_coverage_summary_details(
) -> Option<ReferenceSnapshotBodyClassCoverageSummary> {
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut major_body_row_count = 0usize;
    let mut asteroid_row_count = 0usize;
    let mut major_epochs = BTreeSet::new();
    let mut asteroid_epochs = BTreeSet::new();

    for entry in entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        if is_comparison_body(&entry.body) {
            major_body_row_count += 1;
            major_epochs.insert(epoch_bits);
        }
        if is_reference_asteroid(&entry.body) {
            asteroid_row_count += 1;
            asteroid_epochs.insert(epoch_bits);
        }
    }

    let source_windows = reference_snapshot_source_window_summary_details()?;
    let major_windows = source_windows
        .windows
        .iter()
        .filter(|window| is_comparison_body(&window.body))
        .cloned()
        .collect::<Vec<_>>();
    let asteroid_windows = source_windows
        .windows
        .iter()
        .filter(|window| is_reference_asteroid(&window.body))
        .cloned()
        .collect::<Vec<_>>();

    Some(ReferenceSnapshotBodyClassCoverageSummary {
        major_body_row_count,
        major_bodies: snapshot_bodies()
            .iter()
            .filter(|body| is_comparison_body(body))
            .cloned()
            .collect(),
        major_epoch_count: major_epochs.len(),
        major_windows,
        asteroid_row_count,
        asteroid_bodies: reference_asteroids().to_vec(),
        asteroid_epoch_count: asteroid_epochs.len(),
        asteroid_windows,
    })
}

/// Returns a compact body-class coverage summary for the checked-in reference snapshot.
pub fn reference_snapshot_body_class_coverage_summary(
) -> Option<ReferenceSnapshotBodyClassCoverageSummary> {
    reference_snapshot_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the checked-in reference snapshot.
pub fn reference_snapshot_body_class_coverage_summary_for_report() -> String {
    match reference_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference snapshot body-class coverage: unavailable ({error})"),
        },
        None => "Reference snapshot body-class coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the checked-in reference snapshot in
/// equatorial-frame batch parity mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceSnapshotEquatorialParitySummary {
    /// Total number of parsed snapshot rows exercised through equatorial requests.
    pub row_count: usize,
    /// Number of distinct bodies covered by the snapshot.
    pub body_count: usize,
    /// Bodies covered by the snapshot in first-seen order.
    pub bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs covered by the snapshot.
    pub epoch_count: usize,
    /// Earliest epoch represented in the snapshot.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the snapshot.
    pub latest_epoch: Instant,
}

/// Returns a compact equatorial parity summary for the checked-in reference snapshot.
pub fn reference_snapshot_equatorial_parity_summary(
) -> Option<ReferenceSnapshotEquatorialParitySummary> {
    reference_snapshot_summary()
        .filter(|summary| summary.validate().is_ok())
        .map(|summary| ReferenceSnapshotEquatorialParitySummary {
            row_count: summary.row_count,
            body_count: summary.body_count,
            bodies: summary.bodies,
            epoch_count: summary.epoch_count,
            earliest_epoch: summary.earliest_epoch,
            latest_epoch: summary.latest_epoch,
        })
}

impl ReferenceSnapshotEquatorialParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL reference snapshot equatorial parity: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; mean-obliquity transform against the checked-in ecliptic fixture",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(self.bodies),
        )
    }
}

/// Structured validation errors for a reference snapshot equatorial parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotEquatorialParitySummaryValidationError {
    /// The nested reference snapshot summary failed validation.
    Snapshot(ReferenceSnapshotSummaryValidationError),
}

impl fmt::Display for ReferenceSnapshotEquatorialParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => {
                write!(f, "reference snapshot validation failed: {error}")
            }
        }
    }
}

impl std::error::Error for ReferenceSnapshotEquatorialParitySummaryValidationError {}

impl ReferenceSnapshotEquatorialParitySummary {
    /// Validates that the equatorial parity summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotEquatorialParitySummaryValidationError> {
        let snapshot = ReferenceSnapshotSummary {
            row_count: self.row_count,
            body_count: self.body_count,
            bodies: self.bodies,
            epoch_count: self.epoch_count,
            asteroid_row_count: reference_snapshot_summary()
                .ok_or(
                    ReferenceSnapshotEquatorialParitySummaryValidationError::Snapshot(
                        ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch,
                    ),
                )?
                .asteroid_row_count,
            earliest_epoch: self.earliest_epoch,
            latest_epoch: self.latest_epoch,
        };

        snapshot
            .validate()
            .map_err(ReferenceSnapshotEquatorialParitySummaryValidationError::Snapshot)
    }

    /// Returns a compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotEquatorialParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotEquatorialParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in reference snapshot equatorial parity summary for
/// release-facing reporting.
pub fn format_reference_snapshot_equatorial_parity_summary(
    summary: &ReferenceSnapshotEquatorialParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing reference snapshot equatorial parity summary string.
pub fn reference_snapshot_equatorial_parity_summary_for_report() -> String {
    match reference_snapshot_equatorial_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("JPL reference snapshot equatorial parity: unavailable ({error})")
            }
        },
        None => "JPL reference snapshot equatorial parity: unavailable".to_string(),
    }
}

/// A compact coverage summary for the checked-in reference snapshot in mixed-frame batch parity mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceSnapshotBatchParitySummary {
    /// Reference snapshot coverage exercised through the batch regression.
    pub snapshot: ReferenceSnapshotSummary,
    /// Number of ecliptic requests in the mixed-frame batch regression.
    pub ecliptic_request_count: usize,
    /// Number of equatorial requests in the mixed-frame batch regression.
    pub equatorial_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

/// Returns a compact mixed-frame batch parity summary for the checked-in reference snapshot.
pub fn reference_snapshot_batch_parity_summary() -> Option<ReferenceSnapshotBatchParitySummary> {
    let snapshot = reference_snapshot_summary()?;
    let backend = JplSnapshotBackend;
    let requests = reference_snapshot_batch_parity_requests()?;
    let results = backend.positions(&requests).ok()?;

    if results.len() != requests.len() {
        return None;
    }

    let mut ecliptic_request_count = 0usize;
    let mut equatorial_request_count = 0usize;
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;

    for ((request, result), entry) in requests
        .iter()
        .zip(results.iter())
        .zip(reference_snapshot())
    {
        let single = backend.position(request).ok()?;
        if single != *result {
            return None;
        }

        if result.body != entry.body
            || result.instant != entry.epoch
            || result.frame != request.frame
        {
            return None;
        }

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("reference snapshot batch parity rows should include ecliptic coordinates");
        if *ecliptic != entry.ecliptic() {
            return None;
        }

        if request.frame == CoordinateFrame::Equatorial {
            let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .as_ref()
                .expect("equatorial batch parity rows should include equatorial coordinates");
            if *equatorial != expected_equatorial {
                return None;
            }
        }

        match request.frame {
            CoordinateFrame::Ecliptic => ecliptic_request_count += 1,
            CoordinateFrame::Equatorial => equatorial_request_count += 1,
            _ => return None,
        }

        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some(ReferenceSnapshotBatchParitySummary {
        snapshot,
        ecliptic_request_count,
        equatorial_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Structured validation errors for a reference snapshot batch parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotBatchParitySummaryValidationError {
    /// The nested reference snapshot summary failed validation.
    Snapshot(ReferenceSnapshotSummaryValidationError),
    /// The number of mixed-frame requests does not match the row count.
    RequestCountMismatch {
        ecliptic_request_count: usize,
        equatorial_request_count: usize,
        row_count: usize,
    },
    /// The quality counts do not match the row count.
    QualityCountMismatch {
        exact_count: usize,
        interpolated_count: usize,
        approximate_count: usize,
        unknown_count: usize,
        row_count: usize,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl fmt::Display for ReferenceSnapshotBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(f, "reference snapshot validation failed: {error}"),
            Self::RequestCountMismatch {
                ecliptic_request_count,
                equatorial_request_count,
                row_count,
            } => write!(
                f,
                "request count {}+{} does not match row count {}",
                ecliptic_request_count, equatorial_request_count, row_count,
            ),
            Self::QualityCountMismatch {
                exact_count,
                interpolated_count,
                approximate_count,
                unknown_count,
                row_count,
            } => write!(
                f,
                "quality counts {}+{}+{}+{} do not match row count {}",
                exact_count, interpolated_count, approximate_count, unknown_count, row_count,
            ),
            Self::DerivedSummaryMismatch => f.write_str("derived summary mismatch"),
        }
    }
}

impl std::error::Error for ReferenceSnapshotBatchParitySummaryValidationError {}

impl ReferenceSnapshotBatchParitySummary {
    /// Validates that the batch parity summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotBatchParitySummaryValidationError> {
        self.snapshot
            .validate()
            .map_err(ReferenceSnapshotBatchParitySummaryValidationError::Snapshot)?;

        if self.ecliptic_request_count + self.equatorial_request_count != self.snapshot.row_count {
            return Err(
                ReferenceSnapshotBatchParitySummaryValidationError::RequestCountMismatch {
                    ecliptic_request_count: self.ecliptic_request_count,
                    equatorial_request_count: self.equatorial_request_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.snapshot.row_count
        {
            return Err(
                ReferenceSnapshotBatchParitySummaryValidationError::QualityCountMismatch {
                    exact_count: self.exact_count,
                    interpolated_count: self.interpolated_count,
                    approximate_count: self.approximate_count,
                    unknown_count: self.unknown_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if reference_snapshot_batch_parity_summary().as_ref() != Some(self) {
            return Err(ReferenceSnapshotBatchParitySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL reference snapshot batch parity: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; frame mix: {} ecliptic, {} equatorial; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.snapshot.row_count,
            self.snapshot.body_count,
            self.snapshot.epoch_count,
            format_instant(self.snapshot.earliest_epoch),
            format_instant(self.snapshot.latest_epoch),
            format_bodies(self.snapshot.bodies),
            self.ecliptic_request_count,
            self.equatorial_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns a compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in reference snapshot batch parity summary for release-facing reporting.
pub fn format_reference_snapshot_batch_parity_summary(
    summary: &ReferenceSnapshotBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing reference snapshot batch parity summary string.
pub fn reference_snapshot_batch_parity_summary_for_report() -> String {
    match reference_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL reference snapshot batch parity: unavailable ({error})"),
        },
        None => "JPL reference snapshot batch parity: unavailable".to_string(),
    }
}

impl ReferenceSnapshotSummary {
    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotSummaryValidationError> {
        if self.bodies.is_empty() {
            return Err(ReferenceSnapshotSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(ReferenceSnapshotSummaryValidationError::BodyCountMismatch {
                body_count: self.body_count,
                bodies_len: self.bodies.len(),
            });
        }
        let mut seen_bodies = BTreeMap::new();
        for (index, body) in self.bodies.iter().enumerate() {
            let body_label = body.to_string();
            if let Some(first_index) = seen_bodies.insert(body_label.clone(), index) {
                return Err(ReferenceSnapshotSummaryValidationError::DuplicateBody {
                    first_index,
                    second_index: index,
                    body: body_label,
                });
            }
        }

        let expected_bodies = reference_bodies();
        if self.bodies != expected_bodies {
            let mismatch_index = self
                .bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.bodies.len().min(expected_bodies.len()));
            return Err(ReferenceSnapshotSummaryValidationError::BodyOrderMismatch {
                index: mismatch_index,
                expected: expected_bodies
                    .get(mismatch_index)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "<end of reference body list>".to_string()),
                found: self
                    .bodies
                    .get(mismatch_index)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "<end of summary body list>".to_string()),
            });
        }

        let derived_summary = reference_snapshot_summary()
            .ok_or(ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch)?;
        if self.row_count != derived_summary.row_count {
            return Err(ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch);
        }
        if self.epoch_count == 0 {
            return Err(ReferenceSnapshotSummaryValidationError::MissingEpochs);
        }
        if self.epoch_count != derived_summary.epoch_count {
            return Err(
                ReferenceSnapshotSummaryValidationError::EpochCountMismatch {
                    epoch_count: self.epoch_count,
                    derived_epoch_count: derived_summary.epoch_count,
                },
            );
        }
        if self.asteroid_row_count > self.row_count {
            return Err(
                ReferenceSnapshotSummaryValidationError::AsteroidRowCountExceedsRowCount {
                    asteroid_row_count: self.asteroid_row_count,
                    row_count: self.row_count,
                },
            );
        }
        if self.asteroid_row_count != derived_summary.asteroid_row_count {
            return Err(
                ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                    asteroid_row_count: self.asteroid_row_count,
                    derived_asteroid_row_count: derived_summary.asteroid_row_count,
                },
            );
        }
        if self.earliest_epoch != derived_summary.earliest_epoch
            || self.latest_epoch != derived_summary.latest_epoch
        {
            return Err(
                ReferenceSnapshotSummaryValidationError::EpochRangeMismatch {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                    derived_earliest_epoch: derived_summary.earliest_epoch,
                    derived_latest_epoch: derived_summary.latest_epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot coverage: {} rows across {} bodies and {} epochs ({} asteroid rows; {}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            self.asteroid_row_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(self.bodies),
        )
    }

    /// Returns a compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in reference snapshot coverage for release-facing reporting.
pub fn format_reference_snapshot_summary(summary: &ReferenceSnapshotSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing reference snapshot coverage summary string.
pub fn reference_snapshot_summary_for_report() -> String {
    match reference_snapshot_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference snapshot coverage: unavailable ({error})"),
        },
        None => "Reference snapshot coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the production-generation corpus.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProductionGenerationSnapshotSummary {
    /// Total number of parsed snapshot rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the corpus.
    pub body_count: usize,
    /// Bodies covered by the corpus in first-seen order.
    pub bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs covered by the corpus.
    pub epoch_count: usize,
    /// Number of rows contributed by the boundary overlay.
    pub boundary_row_count: usize,
    /// Number of distinct bodies contributed by the boundary overlay.
    pub boundary_body_count: usize,
    /// Bodies contributed by the boundary overlay in first-seen order.
    pub boundary_bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs represented by the boundary overlay.
    pub boundary_epoch_count: usize,
    /// Earliest epoch represented in the corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the corpus.
    pub latest_epoch: Instant,
    /// Earliest epoch represented in the boundary overlay.
    pub boundary_earliest_epoch: Instant,
    /// Latest epoch represented in the boundary overlay.
    pub boundary_latest_epoch: Instant,
}

/// Structured validation errors for the production-generation coverage summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationSnapshotSummaryValidationError {
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary body order drifted from the checked-in production corpus.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl ProductionGenerationSnapshotSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ProductionGenerationSnapshotSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match body list length {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(f, "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(f, "body order mismatch at index {index}: expected {expected}, found {found}"),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "epoch range {}..{} is invalid",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ProductionGenerationSnapshotSummaryValidationError {}

impl ProductionGenerationSnapshotSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ProductionGenerationSnapshotSummaryValidationError> {
        if self.body_count == 0 {
            return Err(ProductionGenerationSnapshotSummaryValidationError::MissingBodies);
        }
        if self.bodies.is_empty() {
            return Err(ProductionGenerationSnapshotSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                ProductionGenerationSnapshotSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    ProductionGenerationSnapshotSummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .unwrap(),
                        second_index: index,
                        body: body.to_string(),
                    },
                );
            }
        }

        let expected_bodies = production_generation_snapshot_bodies();
        if self.bodies != expected_bodies {
            let mismatch_index = self
                .bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.bodies.len().min(expected_bodies.len()));
            return Err(
                ProductionGenerationSnapshotSummaryValidationError::BodyOrderMismatch {
                    index: mismatch_index,
                    expected: expected_bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of production body list>".to_string()),
                    found: self
                        .bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of summary body list>".to_string()),
                },
            );
        }

        if self.epoch_count == 0 {
            return Err(ProductionGenerationSnapshotSummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                ProductionGenerationSnapshotSummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        if production_generation_snapshot_summary().as_ref() != Some(self) {
            return Err(ProductionGenerationSnapshotSummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Production generation coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; boundary overlay ({PRODUCTION_GENERATION_BOUNDARY_COVERAGE}): {} rows across {} bodies and {} epochs ({}..{}); boundary bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(self.bodies),
            self.boundary_row_count,
            self.boundary_body_count,
            self.boundary_epoch_count,
            format_instant(self.boundary_earliest_epoch),
            format_instant(self.boundary_latest_epoch),
            format_bodies(self.boundary_bodies),
        )
    }

    /// Returns a compact summary line after validating the production-generation summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationSnapshotSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the production-generation coverage for release-facing reporting.
pub fn format_production_generation_snapshot_summary(
    summary: &ProductionGenerationSnapshotSummary,
) -> String {
    summary.summary_line()
}

/// Returns the production-generation coverage summary used in release-facing reporting.
pub fn production_generation_snapshot_summary() -> Option<ProductionGenerationSnapshotSummary> {
    static SUMMARY: OnceLock<ProductionGenerationSnapshotSummary> = OnceLock::new();
    Some(*SUMMARY.get_or_init(|| {
        let entries = production_generation_snapshot_entries()
            .expect("production generation snapshot entries should exist");
        let boundary_entries = production_generation_boundary_entries()
            .expect("production generation boundary entries should exist");

        let mut earliest_epoch = entries[0].epoch;
        let mut latest_epoch = entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        let mut boundary_earliest_epoch = boundary_entries[0].epoch;
        let mut boundary_latest_epoch = boundary_entries[0].epoch;
        let mut boundary_epochs = BTreeSet::new();
        for entry in boundary_entries {
            boundary_epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < boundary_earliest_epoch.julian_day.days() {
                boundary_earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > boundary_latest_epoch.julian_day.days() {
                boundary_latest_epoch = entry.epoch;
            }
        }

        ProductionGenerationSnapshotSummary {
            row_count: entries.len(),
            body_count: production_generation_snapshot_body_list().len(),
            bodies: production_generation_snapshot_body_list(),
            epoch_count: epochs.len(),
            boundary_row_count: boundary_entries.len(),
            boundary_body_count: production_generation_boundary_body_list().len(),
            boundary_bodies: production_generation_boundary_body_list(),
            boundary_epoch_count: boundary_epochs.len(),
            earliest_epoch,
            latest_epoch,
            boundary_earliest_epoch,
            boundary_latest_epoch,
        }
    }))
}

/// Returns the release-facing production-generation coverage summary string.
pub fn production_generation_snapshot_summary_for_report() -> String {
    match production_generation_snapshot_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Production generation coverage: unavailable ({error})"),
        },
        None => "Production generation coverage: unavailable".to_string(),
    }
}

/// Returns the combined source provenance for the production-generation corpus.
pub fn production_generation_source_summary_for_report() -> String {
    let reference_summary = reference_snapshot_source_summary();
    let boundary_summary = production_generation_boundary_source_summary();

    if let Err(error) = reference_summary.validate() {
        return format!("Production generation source: unavailable ({error})");
    }
    if let Err(error) = boundary_summary.validate() {
        return format!("Production generation source: unavailable ({error})");
    }

    format!(
        "Production generation source: {}; {}",
        reference_summary.summary_line(),
        format_production_generation_boundary_source_summary(&boundary_summary)
    )
}

/// A single body-window slice inside the production-generation coverage corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationSnapshotWindow {
    /// The body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of source-backed samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ProductionGenerationSnapshotWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

/// Compact release-facing summary for the production-generation source windows.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationSnapshotWindowSummary {
    /// Number of source-backed samples in the merged production-generation corpus.
    pub sample_count: usize,
    /// Bodies covered by the merged production-generation corpus in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the merged production-generation corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the merged production-generation corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the merged production-generation corpus.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ProductionGenerationSnapshotWindow>,
}

/// Structured validation errors for a production-generation source window summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationSnapshotWindowSummaryValidationError {
    /// The summary did not include any samples.
    MissingSamples,
    /// The summary did not include any bodies.
    MissingBodies,
    /// The declared body count did not match the number of listed bodies.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary included a blank body label.
    BlankBody { index: usize },
    /// The summary body order diverged from the checked-in merged corpus.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary did not include any epochs.
    MissingEpochs,
    /// The summary reported an invalid epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The summary diverged from the derived merged-corpus windows.
    DerivedSummaryMismatch,
}

impl ProductionGenerationSnapshotWindowSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingSamples => "missing samples",
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BlankBody { .. } => "blank body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for ProductionGenerationSnapshotWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match listed bodies {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(
                f,
                "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"
            ),
            Self::BlankBody { index } => write!(f, "blank body at index {index}"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "body order mismatch at index {index}: expected '{expected}', found '{found}'"
            ),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range: earliest {} is after latest {}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch)
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ProductionGenerationSnapshotWindowSummaryValidationError {}

impl fmt::Display for ProductionGenerationSnapshotWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl ProductionGenerationSnapshotWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Production generation source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            self.windows
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        )
    }

    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), ProductionGenerationSnapshotWindowSummaryValidationError> {
        if self.sample_count == 0 {
            return Err(ProductionGenerationSnapshotWindowSummaryValidationError::MissingSamples);
        }
        if self.sample_bodies.is_empty() {
            return Err(ProductionGenerationSnapshotWindowSummaryValidationError::MissingBodies);
        }
        if self.sample_bodies.len() != self.windows.len() {
            return Err(
                ProductionGenerationSnapshotWindowSummaryValidationError::BodyCountMismatch {
                    body_count: self.sample_bodies.len(),
                    bodies_len: self.windows.len(),
                },
            );
        }
        let mut seen_bodies = BTreeSet::new();
        for (index, body) in self.sample_bodies.iter().enumerate() {
            if body.to_string().trim().is_empty() {
                return Err(
                    ProductionGenerationSnapshotWindowSummaryValidationError::BlankBody { index },
                );
            }
            if !seen_bodies.insert(body.to_string()) {
                return Err(
                    ProductionGenerationSnapshotWindowSummaryValidationError::DuplicateBody {
                        first_index: self.sample_bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .unwrap(),
                        second_index: index,
                        body: body.to_string(),
                    },
                );
            }
        }

        let expected_bodies = production_generation_snapshot_bodies();
        if self.sample_bodies.as_slice() != expected_bodies {
            let mismatch_index = self
                .sample_bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.sample_bodies.len().min(expected_bodies.len()));
            return Err(
                ProductionGenerationSnapshotWindowSummaryValidationError::BodyOrderMismatch {
                    index: mismatch_index,
                    expected: expected_bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of production body list>".to_string()),
                    found: self
                        .sample_bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of summary body list>".to_string()),
                },
            );
        }

        if self.epoch_count == 0 {
            return Err(ProductionGenerationSnapshotWindowSummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                ProductionGenerationSnapshotWindowSummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        if production_generation_snapshot_window_summary().as_ref() != Some(self) {
            return Err(
                ProductionGenerationSnapshotWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns a compact summary line after validating the summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationSnapshotWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationSnapshotWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the production-generation source windows for release-facing reporting.
pub fn format_production_generation_snapshot_window_summary(
    summary: &ProductionGenerationSnapshotWindowSummary,
) -> String {
    summary.summary_line()
}

fn production_generation_snapshot_window_summary_details(
) -> Option<ProductionGenerationSnapshotWindowSummary> {
    let entries = production_generation_snapshot_entries()?;
    let mut windows = Vec::new();
    for body in production_generation_snapshot_bodies() {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(ProductionGenerationSnapshotWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("production generation source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("production generation source windows should not be empty after collection");

    Some(ProductionGenerationSnapshotWindowSummary {
        sample_count: entries.len(),
        sample_bodies: production_generation_snapshot_bodies().to_vec(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the merged production-generation source windows.
pub fn production_generation_snapshot_window_summary(
) -> Option<ProductionGenerationSnapshotWindowSummary> {
    static SUMMARY: OnceLock<ProductionGenerationSnapshotWindowSummary> = OnceLock::new();
    Some(
        SUMMARY
            .get_or_init(|| {
                production_generation_snapshot_window_summary_details()
                    .expect("production generation source windows should exist")
            })
            .clone(),
    )
}

/// Returns the release-facing production-generation source window summary string.
pub fn production_generation_snapshot_window_summary_for_report() -> String {
    match production_generation_snapshot_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Production generation source windows: unavailable ({error})"),
        },
        None => "Production generation source windows: unavailable".to_string(),
    }
}

/// A compact body-class coverage summary for the merged production-generation corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationSnapshotBodyClassCoverageSummary {
    /// Number of rows in the merged production-generation corpus.
    pub row_count: usize,
    /// Number of major-body rows in the merged production-generation corpus.
    pub major_body_row_count: usize,
    /// Major bodies covered by the merged production-generation corpus in first-seen order.
    pub major_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the major-body subset.
    pub major_epoch_count: usize,
    /// Per-body windows covered by the major-body subset in first-seen order.
    pub major_windows: Vec<ProductionGenerationSnapshotWindow>,
    /// Number of selected-asteroid rows in the merged production-generation corpus.
    pub asteroid_row_count: usize,
    /// Selected asteroids covered by the merged production-generation corpus in first-seen order.
    pub asteroid_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the selected-asteroid subset.
    pub asteroid_epoch_count: usize,
    /// Per-body windows covered by the selected-asteroid subset in first-seen order.
    pub asteroid_windows: Vec<ProductionGenerationSnapshotWindow>,
}

/// Validation error for a merged production-generation body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the current slice.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the production-generation body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError {}

impl ProductionGenerationSnapshotBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let major_windows = self
            .major_windows
            .iter()
            .map(ProductionGenerationSnapshotWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let asteroid_windows = self
            .asteroid_windows
            .iter()
            .map(ProductionGenerationSnapshotWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Production generation body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
            self.major_body_row_count,
            self.major_bodies.len(),
            self.major_epoch_count,
            major_windows,
            self.asteroid_row_count,
            self.asteroid_bodies.len(),
            self.asteroid_epoch_count,
            asteroid_windows,
        )
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError> {
        let Some(expected) = production_generation_snapshot_body_class_coverage_summary_details()
        else {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self != &expected {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ProductionGenerationSnapshotBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn production_generation_snapshot_body_class_coverage_summary_details(
) -> Option<ProductionGenerationSnapshotBodyClassCoverageSummary> {
    let summary = production_generation_snapshot_summary()?;
    let source_windows = production_generation_snapshot_window_summary_details()?;
    let entries = production_generation_snapshot_entries()?;

    let mut major_body_row_count = 0usize;
    let mut major_epochs = BTreeSet::new();
    let mut asteroid_row_count = 0usize;
    let mut asteroid_epochs = BTreeSet::new();

    for entry in entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        if is_comparison_body(&entry.body) {
            major_body_row_count += 1;
            major_epochs.insert(epoch_bits);
        }
        if is_reference_asteroid(&entry.body) {
            asteroid_row_count += 1;
            asteroid_epochs.insert(epoch_bits);
        }
    }

    Some(ProductionGenerationSnapshotBodyClassCoverageSummary {
        row_count: summary.row_count,
        major_body_row_count,
        major_bodies: summary
            .bodies
            .iter()
            .filter(|body| is_comparison_body(body))
            .cloned()
            .collect(),
        major_epoch_count: major_epochs.len(),
        major_windows: source_windows
            .windows
            .iter()
            .filter(|window| is_comparison_body(&window.body))
            .cloned()
            .collect(),
        asteroid_row_count,
        asteroid_bodies: summary
            .bodies
            .iter()
            .filter(|body| is_reference_asteroid(body))
            .cloned()
            .collect(),
        asteroid_epoch_count: asteroid_epochs.len(),
        asteroid_windows: source_windows
            .windows
            .iter()
            .filter(|window| is_reference_asteroid(&window.body))
            .cloned()
            .collect(),
    })
}

/// Returns the compact body-class coverage summary for the merged production-generation corpus.
pub fn production_generation_snapshot_body_class_coverage_summary(
) -> Option<ProductionGenerationSnapshotBodyClassCoverageSummary> {
    production_generation_snapshot_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the merged production-generation corpus.
pub fn production_generation_snapshot_body_class_coverage_summary_for_report() -> String {
    match production_generation_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Production generation body-class coverage: unavailable ({error})")
            }
        },
        None => "Production generation body-class coverage: unavailable".to_string(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotSummary {
    /// Total number of parsed snapshot rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the comparison corpus.
    pub body_count: usize,
    /// Bodies covered by the comparison corpus in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the comparison corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the comparison corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the comparison corpus.
    pub latest_epoch: Instant,
}

/// Structured validation errors for a comparison snapshot summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonSnapshotSummaryValidationError {
    /// The summary did not include any rows.
    MissingRows,
    /// The summary did not include any bodies.
    MissingBodies,
    /// The declared body count did not match the number of listed bodies.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary body order diverged from the checked-in comparison corpus.
    BodyOrderMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// The summary did not include any epochs.
    MissingEpochs,
    /// The summary reported an invalid epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
}

impl ComparisonSnapshotSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingRows => "missing rows",
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
        }
    }
}

impl fmt::Display for ComparisonSnapshotSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match listed bodies {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(
                f,
                "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "body order mismatch at index {index}: expected '{expected}', found '{found}'"
            ),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch)
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ComparisonSnapshotSummaryValidationError {}

/// Structured validation errors for an independent hold-out snapshot summary.
#[derive(Clone, Debug, PartialEq)]
pub enum IndependentHoldoutSnapshotSummaryValidationError {
    /// The summary did not include any rows.
    MissingRows,
    /// The summary did not include any bodies.
    MissingBodies,
    /// The declared body count did not match the number of listed bodies.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    /// The summary did not include any epochs.
    MissingEpochs,
    /// The summary reported an invalid epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl IndependentHoldoutSnapshotSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingRows => "missing rows",
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for IndependentHoldoutSnapshotSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match listed bodies {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(
                f,
                "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"
            ),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch)
            ),
            Self::DerivedSummaryMismatch => f.write_str(self.label()),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotSummaryValidationError {}

/// A compact coverage summary for the independent hold-out corpus used to
/// validate interpolation against rows that are not part of the main snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotSummary {
    /// Total number of parsed hold-out rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the hold-out corpus.
    pub body_count: usize,
    /// Bodies covered by the hold-out corpus in first-seen order.
    pub bodies: Vec<String>,
    /// Number of distinct epochs covered by the hold-out corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the hold-out corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the hold-out corpus.
    pub latest_epoch: Instant,
}

/// Returns a compact coverage summary for the independent hold-out corpus.
pub fn independent_holdout_snapshot_summary() -> Option<IndependentHoldoutSnapshotSummary> {
    let entries = independent_holdout_snapshot_entries()?;

    let mut bodies = Vec::new();
    let mut seen_bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;

    for entry in entries {
        let body = entry.body.to_string();
        if seen_bodies.insert(body.clone()) {
            bodies.push(body);
        }
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }
    }

    Some(IndependentHoldoutSnapshotSummary {
        row_count: entries.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
    })
}

impl IndependentHoldoutSnapshotSummary {
    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), IndependentHoldoutSnapshotSummaryValidationError> {
        if self.row_count == 0 {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::MissingRows);
        }
        if self.bodies.is_empty() {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                IndependentHoldoutSnapshotSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    IndependentHoldoutSnapshotSummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .unwrap(),
                        second_index: index,
                        body: body.clone(),
                    },
                );
            }
        }

        if self.epoch_count == 0 {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                IndependentHoldoutSnapshotSummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        if independent_holdout_snapshot_summary().as_ref() != Some(self) {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let bodies = if self.bodies.is_empty() {
            "none".to_string()
        } else {
            self.bodies.join(", ")
        };
        format!(
            "Independent hold-out coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            bodies,
        )
    }

    /// Returns a compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for IndependentHoldoutSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A single body-window slice inside the independent hold-out snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotSourceWindow {
    /// The hold-out body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl IndependentHoldoutSnapshotSourceWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

/// Compact release-facing summary for the independent hold-out snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotSourceWindowSummary {
    /// Number of hold-out samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the hold-out source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<IndependentHoldoutSnapshotSourceWindow>,
}

impl IndependentHoldoutSnapshotSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(IndependentHoldoutSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Independent hold-out source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the hold-out source windows still match the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotSourceWindowSummaryValidationError> {
        let Some(expected) = independent_holdout_source_window_summary_details() else {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated hold-out source window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Validation error for an independent hold-out source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndependentHoldoutSnapshotSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in hold-out source windows.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for IndependentHoldoutSnapshotSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the independent hold-out source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotSourceWindowSummaryValidationError {}

fn independent_holdout_source_window_summary_details(
) -> Option<IndependentHoldoutSnapshotSourceWindowSummary> {
    let entries = independent_holdout_snapshot_entries()?;
    let mut windows = Vec::new();
    for body in independent_holdout_bodies() {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(IndependentHoldoutSnapshotSourceWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("independent hold-out source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("independent hold-out source windows should not be empty after collection");

    Some(IndependentHoldoutSnapshotSourceWindowSummary {
        sample_count: entries.len(),
        sample_bodies: independent_holdout_bodies().to_vec(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the independent hold-out source coverage.
pub fn independent_holdout_snapshot_source_window_summary(
) -> Option<IndependentHoldoutSnapshotSourceWindowSummary> {
    independent_holdout_source_window_summary_details()
}

/// Formats the independent hold-out source windows for release-facing reporting.
pub fn format_independent_holdout_snapshot_source_window_summary(
    summary: &IndependentHoldoutSnapshotSourceWindowSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out source window summary string.
pub fn independent_holdout_snapshot_source_window_summary_for_report() -> String {
    match independent_holdout_snapshot_source_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Independent hold-out source windows: unavailable ({error})"),
        },
        None => "Independent hold-out source windows: unavailable".to_string(),
    }
}

/// Compact release-facing summary for any accidental overlap between the checked-in reference snapshot and the independent hold-out snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceHoldoutOverlapSummary {
    /// Number of body/epoch pairs shared by the reference snapshot and the hold-out snapshot.
    pub shared_sample_count: usize,
    /// Shared bodies in first-seen reference order.
    pub shared_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs shared by the reference snapshot and the hold-out snapshot.
    pub shared_epoch_count: usize,
}

/// Structured validation errors for a reference/hold-out overlap summary that drifted from the current snapshots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHoldoutOverlapSummaryValidationError {
    /// A summary field is out of sync with the current overlap posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceHoldoutOverlapSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference/hold-out overlap summary field `{field}` is out of sync with the current snapshots"
            ),
        }
    }
}

impl std::error::Error for ReferenceHoldoutOverlapSummaryValidationError {}

impl ReferenceHoldoutOverlapSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        if self.shared_sample_count == 0 {
            return "Reference/hold-out overlap: 0 shared body-epoch pairs (reference snapshot and independent hold-out remain disjoint)".to_string();
        }

        format!(
            "Reference/hold-out overlap: {} shared body-epoch pairs across {} bodies and {} epochs; bodies: {}",
            self.shared_sample_count,
            self.shared_bodies.len(),
            self.shared_epoch_count,
            format_bodies(&self.shared_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current overlap posture.
    pub fn validate(&self) -> Result<(), ReferenceHoldoutOverlapSummaryValidationError> {
        let Some(expected) = reference_holdout_overlap_summary_details() else {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_sample_count",
                },
            );
        };

        if self.shared_sample_count != expected.shared_sample_count {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_sample_count",
                },
            );
        }
        if self.shared_bodies != expected.shared_bodies {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_bodies",
                },
            );
        }
        if self.shared_epoch_count != expected.shared_epoch_count {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_epoch_count",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHoldoutOverlapSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHoldoutOverlapSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_holdout_overlap_summary_details() -> Option<ReferenceHoldoutOverlapSummary> {
    let reference_entries = reference_snapshot();
    let holdout_entries = independent_holdout_snapshot_entries()?;
    let mut holdout_pairs = BTreeSet::new();
    for entry in holdout_entries {
        holdout_pairs.insert((
            entry.body.to_string(),
            entry.epoch.julian_day.days().to_bits(),
        ));
    }

    let mut shared_bodies = Vec::new();
    let mut shared_body_seen = BTreeSet::new();
    let mut shared_epochs = BTreeSet::new();
    let mut shared_sample_count = 0usize;

    for body in reference_bodies() {
        let mut body_has_overlap = false;
        for entry in reference_entries.iter().filter(|entry| entry.body == *body) {
            let key = (
                entry.body.to_string(),
                entry.epoch.julian_day.days().to_bits(),
            );
            if holdout_pairs.contains(&key) {
                shared_sample_count += 1;
                body_has_overlap = true;
                shared_epochs.insert(entry.epoch.julian_day.days().to_bits());
            }
        }
        if body_has_overlap && shared_body_seen.insert(body.to_string()) {
            shared_bodies.push(body.clone());
        }
    }

    Some(ReferenceHoldoutOverlapSummary {
        shared_sample_count,
        shared_bodies,
        shared_epoch_count: shared_epochs.len(),
    })
}

/// Returns the compact typed summary for any overlap between the reference snapshot and the independent hold-out snapshot.
pub fn reference_holdout_overlap_summary() -> Option<ReferenceHoldoutOverlapSummary> {
    static SUMMARY: OnceLock<ReferenceHoldoutOverlapSummary> = OnceLock::new();
    Some(
        SUMMARY
            .get_or_init(|| {
                reference_holdout_overlap_summary_details()
                    .expect("reference/hold-out overlap summary should exist")
            })
            .clone(),
    )
}

/// Formats the reference/hold-out overlap summary for release-facing reporting.
pub fn format_reference_holdout_overlap_summary(
    summary: &ReferenceHoldoutOverlapSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing reference/hold-out overlap summary string.
pub fn reference_holdout_overlap_summary_for_report() -> String {
    match reference_holdout_overlap_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference/hold-out overlap: unavailable ({error})"),
        },
        None => "Reference/hold-out overlap: unavailable".to_string(),
    }
}

/// Formats the independent hold-out corpus coverage for release-facing reporting.
pub fn format_independent_holdout_snapshot_summary(
    summary: &IndependentHoldoutSnapshotSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out coverage summary string.
pub fn independent_holdout_snapshot_summary_for_report() -> String {
    match independent_holdout_snapshot_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Independent hold-out coverage: unavailable ({error})"),
        },
        None => match independent_holdout_snapshot_error() {
            Some(error) => format!("Independent hold-out coverage: unavailable ({error})"),
            None => "Independent hold-out coverage: unavailable".to_string(),
        },
    }
}

/// A compact body-class coverage summary for the independent hold-out snapshot used by validation.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotBodyClassCoverageSummary {
    /// Number of rows in the hold-out snapshot.
    pub row_count: usize,
    /// Bodies covered by the hold-out snapshot in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the hold-out snapshot.
    pub epoch_count: usize,
    /// Per-body windows covered by the hold-out snapshot in first-seen order.
    pub windows: Vec<IndependentHoldoutSnapshotSourceWindow>,
}

/// Validation error for an independent hold-out body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in hold-out body-class coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the independent hold-out body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError {}

impl IndependentHoldoutSnapshotBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let windows = self
            .windows
            .iter()
            .map(IndependentHoldoutSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Independent hold-out body-class coverage: {} rows across {} bodies and {} epochs; bodies: {}; windows: {}",
            self.row_count,
            self.bodies.len(),
            self.epoch_count,
            format_bodies(&self.bodies),
            windows,
        )
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError> {
        let Some(expected) = independent_holdout_snapshot_body_class_coverage_summary_details()
        else {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self.row_count != expected.row_count {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for IndependentHoldoutSnapshotBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn independent_holdout_snapshot_body_class_coverage_summary_details(
) -> Option<IndependentHoldoutSnapshotBodyClassCoverageSummary> {
    let summary = independent_holdout_snapshot_summary()?;
    let source_windows = independent_holdout_source_window_summary_details()?;

    Some(IndependentHoldoutSnapshotBodyClassCoverageSummary {
        row_count: summary.row_count,
        bodies: independent_holdout_bodies().to_vec(),
        epoch_count: summary.epoch_count,
        windows: source_windows.windows,
    })
}

/// Returns a compact body-class coverage summary for the independent hold-out snapshot used by validation.
pub fn independent_holdout_snapshot_body_class_coverage_summary(
) -> Option<IndependentHoldoutSnapshotBodyClassCoverageSummary> {
    independent_holdout_snapshot_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the independent hold-out snapshot.
pub fn independent_holdout_snapshot_body_class_coverage_summary_for_report() -> String {
    match independent_holdout_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Independent hold-out body-class coverage: unavailable ({error})")
            }
        },
        None => "Independent hold-out body-class coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the independent hold-out corpus in mixed-scale
/// batch parity mode.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotBatchParitySummary {
    /// Coverage summary for the checked-in hold-out corpus.
    pub snapshot: IndependentHoldoutSnapshotSummary,
    /// Number of TT requests in the batch regression.
    pub tt_request_count: usize,
    /// Number of TDB requests in the batch regression.
    pub tdb_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
    /// Whether the batch regression preserved request order and batch/single parity.
    pub parity_preserved: bool,
}

/// Returns a compact mixed-scale batch parity summary for the checked-in hold-out corpus.
pub fn independent_holdout_snapshot_batch_parity_summary(
) -> Option<IndependentHoldoutSnapshotBatchParitySummary> {
    let snapshot = independent_holdout_snapshot_summary()?;
    let entries = independent_holdout_snapshot_entries()?;
    let backend = JplSnapshotBackend;
    let requests = independent_holdout_snapshot_batch_parity_requests()?;
    let results = backend.positions(&requests).ok()?;

    if results.len() != requests.len() {
        return None;
    }

    let mut tt_request_count = 0usize;
    let mut tdb_request_count = 0usize;
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for ((request, result), entry) in requests.iter().zip(results.iter()).zip(entries) {
        let single = backend.position(request).ok();
        single_query_parity &= single.as_ref().is_some_and(|single| single == result);

        order_preserved &= result.body == entry.body
            && result.instant == request.instant
            && result.frame == request.frame
            && result.zodiac_mode == request.zodiac_mode
            && result.apparent == request.apparent;

        match request.instant.scale {
            TimeScale::Tt => tt_request_count += 1,
            TimeScale::Tdb => tdb_request_count += 1,
            _ => return None,
        }

        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some(IndependentHoldoutSnapshotBatchParitySummary {
        snapshot,
        tt_request_count,
        tdb_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
        parity_preserved: order_preserved && single_query_parity,
    })
}

/// Structured validation errors for an independent hold-out batch parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum IndependentHoldoutSnapshotBatchParitySummaryValidationError {
    /// The nested hold-out coverage summary failed validation.
    Snapshot(IndependentHoldoutSnapshotSummaryValidationError),
    /// The number of mixed-scale requests does not match the row count.
    RequestCountMismatch {
        tt_request_count: usize,
        tdb_request_count: usize,
        row_count: usize,
    },
    /// The mixed-scale batch parity slice collapsed to a single time scale.
    TimeScaleMixMissing {
        tt_request_count: usize,
        tdb_request_count: usize,
    },
    /// The quality counts do not match the row count.
    QualityCountMismatch {
        exact_count: usize,
        interpolated_count: usize,
        approximate_count: usize,
        unknown_count: usize,
        row_count: usize,
    },
    /// The batch regression did not preserve request order and single-query parity.
    ParityNotPreserved,
}

impl fmt::Display for IndependentHoldoutSnapshotBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(f, "independent hold-out validation failed: {error}"),
            Self::RequestCountMismatch {
                tt_request_count,
                tdb_request_count,
                row_count,
            } => write!(
                f,
                "request count {}+{} does not match row count {}",
                tt_request_count, tdb_request_count, row_count,
            ),
            Self::TimeScaleMixMissing {
                tt_request_count,
                tdb_request_count,
            } => write!(
                f,
                "time-scale mix must include both TT and TDB requests (TT={}, TDB={})",
                tt_request_count, tdb_request_count,
            ),
            Self::QualityCountMismatch {
                exact_count,
                interpolated_count,
                approximate_count,
                unknown_count,
                row_count,
            } => write!(
                f,
                "quality counts {}+{}+{}+{} do not match row count {}",
                exact_count, interpolated_count, approximate_count, unknown_count, row_count,
            ),
            Self::ParityNotPreserved => f.write_str("batch/single parity was not preserved"),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotBatchParitySummaryValidationError {}

impl IndependentHoldoutSnapshotBatchParitySummary {
    /// Validates that the batch parity summary remains internally consistent.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotBatchParitySummaryValidationError> {
        self.snapshot
            .validate()
            .map_err(IndependentHoldoutSnapshotBatchParitySummaryValidationError::Snapshot)?;

        if self.tt_request_count + self.tdb_request_count != self.snapshot.row_count {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::RequestCountMismatch {
                    tt_request_count: self.tt_request_count,
                    tdb_request_count: self.tdb_request_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if self.tt_request_count == 0 || self.tdb_request_count == 0 {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::TimeScaleMixMissing {
                    tt_request_count: self.tt_request_count,
                    tdb_request_count: self.tdb_request_count,
                },
            );
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.snapshot.row_count
        {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::QualityCountMismatch {
                    exact_count: self.exact_count,
                    interpolated_count: self.interpolated_count,
                    approximate_count: self.approximate_count,
                    unknown_count: self.unknown_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if !self.parity_preserved {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::ParityNotPreserved,
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let order = if self.parity_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        format!(
            "JPL independent hold-out batch parity: {} requests across {} bodies ({}) and {} epochs ({}..{}); TT requests={}, TDB requests={}; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; order={}, single-query parity={}",
            self.snapshot.row_count,
            self.snapshot.body_count,
            if self.snapshot.bodies.is_empty() {
                "none".to_string()
            } else {
                self.snapshot.bodies.join(", ")
            },
            self.snapshot.epoch_count,
            format_instant(self.snapshot.earliest_epoch),
            format_instant(self.snapshot.latest_epoch),
            self.tt_request_count,
            self.tdb_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
            order,
            order,
        )
    }
}

impl fmt::Display for IndependentHoldoutSnapshotBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the independent hold-out mixed-scale batch parity summary for release-facing reporting.
pub fn format_independent_holdout_snapshot_batch_parity_summary(
    summary: &IndependentHoldoutSnapshotBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out mixed-scale batch parity summary string.
pub fn independent_holdout_snapshot_batch_parity_summary_for_report() -> String {
    match independent_holdout_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL independent hold-out batch parity: unavailable ({error})"),
        },
        None => "JPL independent hold-out batch parity: unavailable".to_string(),
    }
}

/// A compact coverage summary for the independent hold-out corpus in
/// equatorial-frame batch parity mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotEquatorialParitySummary {
    /// Total number of parsed hold-out rows exercised through equatorial requests.
    pub row_count: usize,
    /// Number of distinct bodies covered by the hold-out corpus.
    pub body_count: usize,
    /// Number of distinct epochs covered by the hold-out corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the hold-out corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the hold-out corpus.
    pub latest_epoch: Instant,
}

/// Returns a compact equatorial parity summary for the checked-in hold-out corpus.
pub fn independent_holdout_snapshot_equatorial_parity_summary(
) -> Option<IndependentHoldoutSnapshotEquatorialParitySummary> {
    independent_holdout_snapshot_summary().map(|summary| {
        IndependentHoldoutSnapshotEquatorialParitySummary {
            row_count: summary.row_count,
            body_count: summary.body_count,
            epoch_count: summary.epoch_count,
            earliest_epoch: summary.earliest_epoch,
            latest_epoch: summary.latest_epoch,
        }
    })
}

impl IndependentHoldoutSnapshotEquatorialParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotEquatorialParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL independent hold-out equatorial parity: {} rows across {} bodies and {} epochs ({}..{}); mean-obliquity transform against the checked-in ecliptic fixture",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
        )
    }
}

/// Structured validation errors for an independent hold-out equatorial parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum IndependentHoldoutSnapshotEquatorialParitySummaryValidationError {
    /// The summary did not include any rows.
    MissingRows,
    /// The summary did not include any bodies.
    MissingBodies,
    /// The body count exceeds the row count.
    BodyCountExceedsRowCount { body_count: usize, row_count: usize },
    /// The summary did not include any epochs.
    MissingEpochs,
    /// The summary reported an invalid epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
}

impl fmt::Display for IndependentHoldoutSnapshotEquatorialParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRows => f.write_str("missing rows"),
            Self::MissingBodies => f.write_str("missing bodies"),
            Self::BodyCountExceedsRowCount {
                body_count,
                row_count,
            } => write!(f, "body count {body_count} exceeds row count {row_count}"),
            Self::MissingEpochs => f.write_str("missing epochs"),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch)
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotEquatorialParitySummaryValidationError {}

impl IndependentHoldoutSnapshotEquatorialParitySummary {
    /// Validates that the equatorial parity summary remains internally consistent.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotEquatorialParitySummaryValidationError> {
        if self.row_count == 0 {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::MissingRows,
            );
        }
        if self.body_count == 0 {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::MissingBodies,
            );
        }
        if self.body_count > self.row_count {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::BodyCountExceedsRowCount {
                    body_count: self.body_count,
                    row_count: self.row_count,
                },
            );
        }
        if self.epoch_count == 0 {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::MissingEpochs,
            );
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for IndependentHoldoutSnapshotEquatorialParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in independent hold-out equatorial parity summary for
/// release-facing reporting.
pub fn format_independent_holdout_snapshot_equatorial_parity_summary(
    summary: &IndependentHoldoutSnapshotEquatorialParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out equatorial parity summary string.
pub fn independent_holdout_snapshot_equatorial_parity_summary_for_report() -> String {
    match independent_holdout_snapshot_equatorial_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("JPL independent hold-out equatorial parity: unavailable ({error})")
            }
        },
        None => "JPL independent hold-out equatorial parity: unavailable".to_string(),
    }
}

/// Returns a compact coverage summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_summary() -> Option<ComparisonSnapshotSummary> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;

    for entry in entries {
        bodies.insert(entry.body.to_string());
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }
    }

    Some(ComparisonSnapshotSummary {
        row_count: entries.len(),
        body_count: bodies.len(),
        bodies: comparison_body_list().to_vec(),
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// A compact body-class coverage summary for the comparison snapshot used by validation.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotBodyClassCoverageSummary {
    /// Number of rows in the comparison snapshot.
    pub row_count: usize,
    /// Bodies covered by the comparison snapshot in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the comparison snapshot.
    pub epoch_count: usize,
    /// Per-body windows covered by the comparison snapshot in first-seen order.
    pub windows: Vec<ComparisonSnapshotSourceWindow>,
}

/// Validation error for a comparison snapshot body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComparisonSnapshotBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in body-class coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ComparisonSnapshotBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the comparison snapshot body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ComparisonSnapshotBodyClassCoverageSummaryValidationError {}

impl ComparisonSnapshotBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let windows = self
            .windows
            .iter()
            .map(ComparisonSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Comparison snapshot body-class coverage: {} rows across {} bodies and {} epochs; bodies: {}; windows: {}",
            self.row_count,
            self.body_count(),
            self.epoch_count,
            format_bodies(&self.bodies),
            windows,
        )
    }

    fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ComparisonSnapshotBodyClassCoverageSummaryValidationError> {
        let Some(expected) = comparison_snapshot_body_class_coverage_summary_details() else {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self.row_count != expected.row_count {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn comparison_snapshot_body_class_coverage_summary_details(
) -> Option<ComparisonSnapshotBodyClassCoverageSummary> {
    let summary = comparison_snapshot_summary()?;
    let source_windows = comparison_snapshot_source_window_summary_details()?;

    Some(ComparisonSnapshotBodyClassCoverageSummary {
        row_count: summary.row_count,
        bodies: summary.bodies.to_vec(),
        epoch_count: summary.epoch_count,
        windows: source_windows.windows,
    })
}

/// Returns the compact body-class coverage summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_body_class_coverage_summary(
) -> Option<ComparisonSnapshotBodyClassCoverageSummary> {
    comparison_snapshot_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the comparison snapshot.
pub fn comparison_snapshot_body_class_coverage_summary_for_report() -> String {
    match comparison_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Comparison snapshot body-class coverage: unavailable ({error})"),
        },
        None => "Comparison snapshot body-class coverage: unavailable".to_string(),
    }
}

/// Backend-owned provenance summary for the comparison snapshot used by validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComparisonSnapshotSourceSummary {
    /// Source attribution for the comparison snapshot.
    pub source: String,
    /// Coverage note for the comparison snapshot.
    pub coverage: String,
    /// CSV column layout for the comparison snapshot.
    pub columns: String,
}

/// Structured validation errors for a comparison snapshot provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonSnapshotSourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty coverage label.
    BlankCoverage,
    /// The summary did not include a non-empty columns label.
    BlankColumns,
    /// The summary carried surrounding whitespace in one of its labels.
    SurroundedByWhitespace { field: &'static str },
}

impl ComparisonSnapshotSourceSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankSource => "blank source",
            Self::BlankCoverage => "blank coverage",
            Self::BlankColumns => "blank columns",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
        }
    }
}

impl fmt::Display for ComparisonSnapshotSourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ComparisonSnapshotSourceSummaryValidationError {}

impl ComparisonSnapshotSourceSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotSourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::BlankSource);
        }
        if has_surrounding_whitespace(&self.source) {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                },
            );
        }
        if self.coverage.trim().is_empty() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::BlankCoverage);
        }
        if has_surrounding_whitespace(&self.coverage) {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                },
            );
        }
        if self.columns.trim().is_empty() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::BlankColumns);
        }
        if has_surrounding_whitespace(&self.columns) {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "columns",
                },
            );
        }
        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Comparison snapshot source: {}; coverage={}; columns={}",
            self.source, self.coverage, self.columns
        )
    }

    /// Returns a compact summary line after validating the comparison snapshot source summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotSourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the comparison snapshot.
pub fn comparison_snapshot_source_summary() -> ComparisonSnapshotSourceSummary {
    static SUMMARY: OnceLock<ComparisonSnapshotSourceSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = comparison_snapshot_manifest();
            ComparisonSnapshotSourceSummary {
                source: manifest
                    .source_or("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.")
                    .to_string(),
                coverage: manifest.coverage_or(
                    "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
                )
                .to_string(),
                columns: manifest.columns_summary(),
            }
        })
        .clone()
}

/// Formats the source/material summary for the comparison snapshot used by validation.
pub fn format_comparison_snapshot_source_summary(
    summary: &ComparisonSnapshotSourceSummary,
) -> String {
    summary.summary_line()
}

fn format_validated_comparison_snapshot_source_summary_for_report(
    summary: &ComparisonSnapshotSourceSummary,
    manifest: &SnapshotManifest,
) -> String {
    if let Err(error) = manifest.validate() {
        return format!("Comparison snapshot source: unavailable ({error})");
    }

    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Comparison snapshot source: unavailable ({error})"),
    }
}

/// Returns the source/material summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_source_summary_for_report() -> String {
    format_validated_comparison_snapshot_source_summary_for_report(
        &comparison_snapshot_source_summary(),
        comparison_snapshot_manifest(),
    )
}

/// A single body-window slice inside the comparison snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotSourceWindow {
    /// The snapshot body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ComparisonSnapshotSourceWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

/// Compact release-facing summary for the comparison snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotSourceWindowSummary {
    /// Number of comparison-snapshot samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the comparison snapshot source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ComparisonSnapshotSourceWindow>,
}

impl fmt::Display for ComparisonSnapshotSourceWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl ComparisonSnapshotSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ComparisonSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Comparison snapshot source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the comparison snapshot source windows still match the checked-in slice.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotSourceWindowSummaryValidationError> {
        let Some(expected) = comparison_snapshot_source_window_summary_details() else {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated comparison snapshot source window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotSourceWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a comparison snapshot source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonSnapshotSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in comparison snapshot source windows.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ComparisonSnapshotSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the comparison snapshot source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ComparisonSnapshotSourceWindowSummaryValidationError {}

fn comparison_snapshot_source_window_summary_details(
) -> Option<ComparisonSnapshotSourceWindowSummary> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut windows = Vec::new();
    for body in comparison_body_list() {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(ComparisonSnapshotSourceWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("comparison snapshot source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("comparison snapshot source windows should not be empty after collection");

    Some(ComparisonSnapshotSourceWindowSummary {
        sample_count: entries.len(),
        sample_bodies: comparison_body_list().to_vec(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the comparison snapshot source coverage.
pub fn comparison_snapshot_source_window_summary() -> Option<ComparisonSnapshotSourceWindowSummary>
{
    comparison_snapshot_source_window_summary_details()
}

/// Formats the comparison snapshot source windows for release-facing reporting.
pub fn format_comparison_snapshot_source_window_summary(
    summary: &ComparisonSnapshotSourceWindowSummary,
) -> String {
    summary.summary_line()
}

/// Returns the body-window summary for the comparison snapshot.
pub fn comparison_snapshot_source_window_summary_for_report() -> String {
    match comparison_snapshot_source_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Comparison snapshot source windows: unavailable ({error})"),
        },
        None => "Comparison snapshot source windows: unavailable".to_string(),
    }
}

#[cfg(test)]
fn format_validated_source_summary_for_report(
    label: &'static str,
    manifest: &SnapshotManifest,
    render: impl FnOnce() -> String,
) -> String {
    match manifest.validate() {
        Ok(()) => render(),
        Err(error) => format!("{label}: unavailable ({error})"),
    }
}

#[cfg(test)]
fn format_manifest_summary_for_report(label: &str, manifest: &SnapshotManifest) -> String {
    match manifest.validate() {
        Ok(()) => manifest.summary_line(label),
        Err(error) => format!("{label}: unavailable ({error})"),
    }
}

/// Returns the manifest summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_manifest_summary() -> SnapshotManifestSummary {
    SnapshotManifestSummary {
        label: "Comparison snapshot manifest",
        manifest: comparison_snapshot_manifest().clone(),
        source_fallback: "unknown",
        coverage_fallback:
            "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
    }
}

/// Returns the manifest summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_manifest_summary_for_report() -> String {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/j2000_snapshot.csv"
    ));
    if let Err(error) = validate_snapshot_manifest_header_structure(
        manifest_text,
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
        "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
        &["body", "x_km", "y_km", "z_km"],
    ) {
        return format!("Comparison snapshot manifest: unavailable ({error})");
    }

    let summary = comparison_snapshot_manifest_summary();
    match summary.validate_with_expected_metadata(
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
        "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
        &["body", "x_km", "y_km", "z_km"],
    ) {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("Comparison snapshot manifest: unavailable ({error})"),
    }
}

impl ComparisonSnapshotSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotSummaryValidationError> {
        if self.row_count == 0 {
            return Err(ComparisonSnapshotSummaryValidationError::MissingRows);
        }
        if self.bodies.is_empty() {
            return Err(ComparisonSnapshotSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                ComparisonSnapshotSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(ComparisonSnapshotSummaryValidationError::DuplicateBody {
                    first_index: self.bodies[..index]
                        .iter()
                        .position(|other| other == body)
                        .unwrap(),
                    second_index: index,
                    body: body.to_string(),
                });
            }
        }

        let expected_bodies = comparison_body_list();
        if self.bodies != expected_bodies {
            let mismatch_index = self
                .bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.bodies.len().min(expected_bodies.len()));
            return Err(
                ComparisonSnapshotSummaryValidationError::BodyOrderMismatch {
                    index: mismatch_index,
                    expected: expected_bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of comparison body list>".to_string()),
                    found: self
                        .bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of summary body list>".to_string()),
                },
            );
        }

        if self.epoch_count == 0 {
            return Err(ComparisonSnapshotSummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                ComparisonSnapshotSummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Comparison snapshot coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.bodies),
        )
    }

    /// Returns a compact summary line after validating the comparison snapshot summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the comparison snapshot coverage for release-facing reporting.
pub fn format_comparison_snapshot_summary(summary: &ComparisonSnapshotSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing comparison snapshot coverage summary string.
pub fn comparison_snapshot_summary_for_report() -> String {
    match comparison_snapshot_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Comparison snapshot coverage: unavailable ({error})"),
        },
        None => "Comparison snapshot coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the checked-in comparison snapshot in mixed-frame batch parity mode.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotBatchParitySummary {
    /// Comparison snapshot coverage exercised through the batch regression.
    pub snapshot: ComparisonSnapshotSummary,
    /// Number of ecliptic requests in the mixed-frame batch regression.
    pub ecliptic_request_count: usize,
    /// Number of equatorial requests in the mixed-frame batch regression.
    pub equatorial_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

/// Returns a compact mixed-frame batch parity summary for the checked-in comparison snapshot.
pub fn comparison_snapshot_batch_parity_summary() -> Option<ComparisonSnapshotBatchParitySummary> {
    let snapshot = comparison_snapshot_summary()?;
    let backend = JplSnapshotBackend;
    let requests = comparison_snapshot_batch_parity_requests()?;
    let results = backend.positions(&requests).ok()?;

    if results.len() != requests.len() {
        return None;
    }

    let mut ecliptic_request_count = 0usize;
    let mut equatorial_request_count = 0usize;
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;

    for ((request, result), entry) in requests
        .iter()
        .zip(results.iter())
        .zip(comparison_snapshot())
    {
        let single = backend.position(request).ok()?;
        if single != *result {
            return None;
        }

        if result.body != entry.body
            || result.instant.julian_day != entry.epoch.julian_day
            || result.frame != request.frame
        {
            return None;
        }

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("comparison snapshot batch parity rows should include ecliptic coordinates");
        if *ecliptic != entry.ecliptic() {
            return None;
        }

        if request.frame == CoordinateFrame::Equatorial {
            let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .as_ref()
                .expect("equatorial batch parity rows should include equatorial coordinates");
            if *equatorial != expected_equatorial {
                return None;
            }
        }

        match request.frame {
            CoordinateFrame::Ecliptic => ecliptic_request_count += 1,
            CoordinateFrame::Equatorial => equatorial_request_count += 1,
            _ => return None,
        }

        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some(ComparisonSnapshotBatchParitySummary {
        snapshot,
        ecliptic_request_count,
        equatorial_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Structured validation errors for a comparison snapshot batch parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonSnapshotBatchParitySummaryValidationError {
    /// The nested comparison snapshot summary failed validation.
    Snapshot(ComparisonSnapshotSummaryValidationError),
    /// The number of mixed-frame requests does not match the row count.
    RequestCountMismatch {
        ecliptic_request_count: usize,
        equatorial_request_count: usize,
        row_count: usize,
    },
    /// The quality counts do not match the row count.
    QualityCountMismatch {
        exact_count: usize,
        interpolated_count: usize,
        approximate_count: usize,
        unknown_count: usize,
        row_count: usize,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl fmt::Display for ComparisonSnapshotBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(f, "comparison snapshot validation failed: {error}"),
            Self::RequestCountMismatch {
                ecliptic_request_count,
                equatorial_request_count,
                row_count,
            } => write!(
                f,
                "request count {}+{} does not match row count {}",
                ecliptic_request_count, equatorial_request_count, row_count,
            ),
            Self::QualityCountMismatch {
                exact_count,
                interpolated_count,
                approximate_count,
                unknown_count,
                row_count,
            } => write!(
                f,
                "quality counts {}+{}+{}+{} do not match row count {}",
                exact_count, interpolated_count, approximate_count, unknown_count, row_count,
            ),
            Self::DerivedSummaryMismatch => f.write_str("derived summary mismatch"),
        }
    }
}

impl std::error::Error for ComparisonSnapshotBatchParitySummaryValidationError {}

impl ComparisonSnapshotBatchParitySummary {
    /// Validates that the batch parity summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotBatchParitySummaryValidationError> {
        self.snapshot
            .validate()
            .map_err(ComparisonSnapshotBatchParitySummaryValidationError::Snapshot)?;

        if self.ecliptic_request_count + self.equatorial_request_count != self.snapshot.row_count {
            return Err(
                ComparisonSnapshotBatchParitySummaryValidationError::RequestCountMismatch {
                    ecliptic_request_count: self.ecliptic_request_count,
                    equatorial_request_count: self.equatorial_request_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.snapshot.row_count
        {
            return Err(
                ComparisonSnapshotBatchParitySummaryValidationError::QualityCountMismatch {
                    exact_count: self.exact_count,
                    interpolated_count: self.interpolated_count,
                    approximate_count: self.approximate_count,
                    unknown_count: self.unknown_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if comparison_snapshot_batch_parity_summary().as_ref() != Some(self) {
            return Err(
                ComparisonSnapshotBatchParitySummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL comparison snapshot batch parity: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; frame mix: {} ecliptic, {} equatorial; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.snapshot.row_count,
            self.snapshot.body_count,
            self.snapshot.epoch_count,
            format_instant(self.snapshot.earliest_epoch),
            format_instant(self.snapshot.latest_epoch),
            format_bodies(&self.snapshot.bodies),
            self.ecliptic_request_count,
            self.equatorial_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns a compact summary line after validating the batch parity summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in comparison snapshot batch parity summary for release-facing reporting.
pub fn format_comparison_snapshot_batch_parity_summary(
    summary: &ComparisonSnapshotBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing comparison snapshot batch parity summary string.
pub fn comparison_snapshot_batch_parity_summary_for_report() -> String {
    match comparison_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL comparison snapshot batch parity: unavailable ({error})"),
        },
        None => "JPL comparison snapshot batch parity: unavailable".to_string(),
    }
}

/// Returns the source-backed asteroid subset present in the reference snapshot.
pub fn reference_asteroids() -> &'static [pleiades_backend::CelestialBody] {
    reference_asteroid_list()
}

/// Exact J2000 asteroid reference samples from the checked-in snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEvidence {
    /// Asteroid body covered by the exact snapshot row.
    pub body: pleiades_backend::CelestialBody,
    /// Exact epoch used for the reference sample.
    pub epoch: Instant,
    /// Ecliptic longitude in degrees.
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees.
    pub latitude_deg: f64,
    /// Geocentric distance in astronomical units.
    pub distance_au: f64,
}

/// Exact J2000 asteroid equatorial samples derived from the checked-in snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEquatorialEvidence {
    /// Asteroid body covered by the exact snapshot row.
    pub body: pleiades_backend::CelestialBody,
    /// Exact epoch used for the reference sample.
    pub epoch: Instant,
    /// Mean-obliquity equatorial coordinates derived from the ecliptic sample.
    pub equatorial: pleiades_types::EquatorialCoordinates,
}

/// Returns the exact J2000 asteroid evidence samples present in the reference snapshot.
pub fn reference_asteroid_evidence() -> &'static [ReferenceAsteroidEvidence] {
    reference_asteroid_evidence_list()
}

/// Returns the exact J2000 asteroid request corpus in the requested frame.
///
/// The requests preserve the checked-in asteroid order and stored J2000 epoch,
/// so downstream batch checks can reuse the exact selected-asteroid slice
/// without reconstructing it from the sample evidence.
pub fn reference_asteroid_requests(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests_with_frame_selector(|_| frame)
}

/// Returns the exact J2000 asteroid request corpus in the requested frame.
///
/// This is a compatibility alias for [`reference_asteroid_requests`].
#[doc(alias = "reference_asteroid_requests")]
pub fn reference_asteroid_request_corpus(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests(frame)
}

/// Returns the exact J2000 asteroid request corpus in the ecliptic frame.
///
/// This is a compatibility alias for [`reference_asteroid_request_corpus`].
#[doc(alias = "reference_asteroid_request_corpus")]
pub fn reference_asteroid_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests(CoordinateFrame::Ecliptic)
}

/// Returns the exact J2000 asteroid request corpus in the equatorial frame.
///
/// This is a compatibility alias for [`reference_asteroid_request_corpus`].
#[doc(alias = "reference_asteroid_request_corpus")]
pub fn reference_asteroid_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests(CoordinateFrame::Equatorial)
}

/// Returns the mixed-frame exact J2000 asteroid request corpus used by batch parity checks.
///
/// The requests preserve the checked-in asteroid order and alternate between
/// ecliptic and equatorial frames so downstream tooling can reuse the exact
/// selected-asteroid batch shape without reconstructing it from sample evidence.
pub fn reference_asteroid_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests_with_frame_selector(|index| {
        if index % 2 == 0 {
            CoordinateFrame::Ecliptic
        } else {
            CoordinateFrame::Equatorial
        }
    })
}

/// Returns the mixed-frame exact J2000 asteroid request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_asteroid_batch_parity_requests`].
#[doc(alias = "reference_asteroid_batch_parity_requests")]
pub fn reference_asteroid_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_batch_parity_requests()
}

/// Returns the exact J2000 asteroid equatorial evidence samples derived from the reference snapshot.
pub fn reference_asteroid_equatorial_evidence() -> &'static [ReferenceAsteroidEquatorialEvidence] {
    reference_asteroid_equatorial_evidence_list()
}

/// Compact release-facing summary for the exact asteroid evidence slice.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEvidenceSummary {
    /// Number of exact samples in the selected asteroid slice.
    pub sample_count: usize,
    /// Bodies covered by the exact asteroid evidence slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the asteroid evidence slice.
    pub epoch: Instant,
}

/// Validation errors for a reference asteroid evidence summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceAsteroidEvidenceSummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for ReferenceAsteroidEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceAsteroidEvidenceSummaryValidationError {}

impl ReferenceAsteroidEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Selected asteroid evidence: {} exact J2000 samples at {} ({})",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceAsteroidEvidenceSummaryValidationError> {
        let evidence = reference_asteroid_evidence();
        if evidence.is_empty() {
            return Err(ReferenceAsteroidEvidenceSummaryValidationError::Empty);
        }

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceAsteroidEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.sample_bodies.as_slice() != reference_asteroids() {
            for (index, (expected, found)) in reference_asteroids()
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceAsteroidEvidenceSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceAsteroidEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceAsteroidEvidenceSummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceAsteroidEvidenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceAsteroidEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the equatorial asteroid evidence slice.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEquatorialEvidenceSummary {
    /// Number of exact samples in the selected asteroid slice.
    pub sample_count: usize,
    /// Bodies covered by the exact asteroid evidence slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the asteroid evidence slice.
    pub epoch: Instant,
    /// Summary of the equatorial transform used to derive the equatorial slice.
    pub transform_note: &'static str,
}

/// Validation errors for a reference asteroid equatorial evidence summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceAsteroidEquatorialEvidenceSummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
    /// The summary transform note drifted from the current evidence slice.
    TransformNoteMismatch {
        expected: &'static str,
        found: &'static str,
    },
}

impl fmt::Display for ReferenceAsteroidEquatorialEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid equatorial evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid equatorial evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid equatorial evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid equatorial evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::TransformNoteMismatch { expected, found } => write!(
                f,
                "selected asteroid equatorial evidence transform note mismatch: expected '{expected}', found '{found}'"
            ),
        }
    }
}

impl std::error::Error for ReferenceAsteroidEquatorialEvidenceSummaryValidationError {}

impl ReferenceAsteroidEquatorialEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Selected asteroid equatorial evidence: {} exact J2000 samples at {} ({}) using a {}",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
            self.transform_note,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceAsteroidEquatorialEvidenceSummaryValidationError> {
        let evidence = reference_asteroid_equatorial_evidence();
        if evidence.is_empty() {
            return Err(ReferenceAsteroidEquatorialEvidenceSummaryValidationError::Empty);
        }

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.sample_bodies.as_slice() != reference_asteroids() {
            for (index, (expected, found)) in reference_asteroids()
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceAsteroidEquatorialEvidenceSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }
        if self.transform_note != "mean-obliquity equatorial transform" {
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::TransformNoteMismatch {
                    expected: "mean-obliquity equatorial transform",
                    found: self.transform_note,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceAsteroidEquatorialEvidenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceAsteroidEquatorialEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_asteroid_evidence_summary_details() -> Option<ReferenceAsteroidEvidenceSummary> {
    let evidence = reference_asteroid_evidence();
    evidence
        .first()
        .map(|first| ReferenceAsteroidEvidenceSummary {
            sample_count: evidence.len(),
            sample_bodies: reference_asteroids().to_vec(),
            epoch: first.epoch,
        })
}

fn reference_asteroid_equatorial_evidence_summary_details(
) -> Option<ReferenceAsteroidEquatorialEvidenceSummary> {
    let evidence = reference_asteroid_equatorial_evidence();
    evidence
        .first()
        .map(|first| ReferenceAsteroidEquatorialEvidenceSummary {
            sample_count: evidence.len(),
            sample_bodies: reference_asteroids().to_vec(),
            epoch: first.epoch,
            transform_note: "mean-obliquity equatorial transform",
        })
}

fn reference_asteroid_evidence_summary_from_slice(
    evidence: &[ReferenceAsteroidEvidence],
) -> Option<ReferenceAsteroidEvidenceSummary> {
    evidence
        .first()
        .map(|first| ReferenceAsteroidEvidenceSummary {
            sample_count: evidence.len(),
            sample_bodies: reference_asteroids().to_vec(),
            epoch: first.epoch,
        })
}

fn reference_asteroid_equatorial_evidence_summary_from_slice(
    evidence: &[ReferenceAsteroidEquatorialEvidence],
) -> Option<ReferenceAsteroidEquatorialEvidenceSummary> {
    evidence
        .first()
        .map(|first| ReferenceAsteroidEquatorialEvidenceSummary {
            sample_count: evidence.len(),
            sample_bodies: reference_asteroids().to_vec(),
            epoch: first.epoch,
            transform_note: "mean-obliquity equatorial transform",
        })
}

/// Returns the compact typed summary for the exact asteroid evidence slice.
pub fn reference_asteroid_evidence_summary() -> Option<ReferenceAsteroidEvidenceSummary> {
    reference_asteroid_evidence_summary_details()
}

/// Returns the compact typed summary for the equatorial asteroid evidence slice.
pub fn reference_asteroid_equatorial_evidence_summary(
) -> Option<ReferenceAsteroidEquatorialEvidenceSummary> {
    reference_asteroid_equatorial_evidence_summary_details()
}

fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_bodies(bodies: &[pleiades_backend::CelestialBody]) -> String {
    join_display(bodies)
}

fn format_coordinate_frames(frames: &[CoordinateFrame]) -> String {
    join_display(frames)
}

fn format_time_scales(time_scales: &[TimeScale]) -> String {
    join_display(time_scales)
}

fn format_zodiac_modes(zodiac_modes: &[ZodiacMode]) -> String {
    join_display(zodiac_modes)
}

fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    join_display(modes)
}

const ASTEROID_EQUATORIAL_TOLERANCE_DEGREES: f64 = 1e-12;
const ASTEROID_DISTANCE_TOLERANCE_AU: f64 = 1e-12;

#[derive(Clone, Debug, PartialEq)]
enum ReferenceAsteroidEvidenceValidationError {
    Empty,
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    EpochMismatch {
        index: usize,
        expected: Instant,
        found: Instant,
    },
    NonFiniteLongitude {
        index: usize,
        body: pleiades_backend::CelestialBody,
    },
    NonFiniteLatitude {
        index: usize,
        body: pleiades_backend::CelestialBody,
    },
    NonFiniteDistance {
        index: usize,
        body: pleiades_backend::CelestialBody,
    },
}

impl fmt::Display for ReferenceAsteroidEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("exact asteroid evidence is unavailable"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "exact asteroid evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "exact asteroid evidence epoch mismatch at index {index}: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::NonFiniteLongitude { index, body } => write!(
                f,
                "exact asteroid evidence longitude is non-finite at index {index} for body {body}"
            ),
            Self::NonFiniteLatitude { index, body } => write!(
                f,
                "exact asteroid evidence latitude is non-finite at index {index} for body {body}"
            ),
            Self::NonFiniteDistance { index, body } => write!(
                f,
                "exact asteroid evidence distance is non-finite at index {index} for body {body}"
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ReferenceAsteroidEquatorialEvidenceValidationError {
    Empty,
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    EpochMismatch {
        index: usize,
        expected: Instant,
        found: Instant,
    },
    RightAscensionMismatch {
        index: usize,
        body: pleiades_backend::CelestialBody,
    },
    DeclinationMismatch {
        index: usize,
        body: pleiades_backend::CelestialBody,
    },
    DistanceMismatch {
        index: usize,
        body: pleiades_backend::CelestialBody,
    },
}

impl fmt::Display for ReferenceAsteroidEquatorialEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("equatorial asteroid evidence is unavailable"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "equatorial asteroid evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "equatorial asteroid evidence epoch mismatch at index {index}: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::RightAscensionMismatch { index, body } => write!(
                f,
                "equatorial asteroid evidence right ascension diverges from the derived mean-obliquity transform at index {index} for body {body}"
            ),
            Self::DeclinationMismatch { index, body } => write!(
                f,
                "equatorial asteroid evidence declination diverges from the derived mean-obliquity transform at index {index} for body {body}"
            ),
            Self::DistanceMismatch { index, body } => write!(
                f,
                "equatorial asteroid evidence distance diverges from the derived mean-obliquity transform at index {index} for body {body}"
            ),
        }
    }
}

fn validate_reference_asteroid_evidence(
    evidence: &[ReferenceAsteroidEvidence],
) -> Result<(), ReferenceAsteroidEvidenceValidationError> {
    if evidence.is_empty() {
        return Err(ReferenceAsteroidEvidenceValidationError::Empty);
    }

    let expected_bodies = reference_asteroids();
    if evidence.len() != expected_bodies.len() {
        return Err(
            ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch {
                index: evidence.len(),
                expected: expected_bodies
                    .get(evidence.len())
                    .cloned()
                    .unwrap_or_else(|| expected_bodies[expected_bodies.len() - 1].clone()),
                found: evidence
                    .last()
                    .map(|sample| sample.body.clone())
                    .unwrap_or_else(|| expected_bodies[0].clone()),
            },
        );
    }

    let expected_epoch = reference_instant();
    for (index, (sample, expected_body)) in evidence.iter().zip(expected_bodies.iter()).enumerate()
    {
        if sample.body != *expected_body {
            return Err(
                ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch {
                    index,
                    expected: expected_body.clone(),
                    found: sample.body.clone(),
                },
            );
        }
        if sample.epoch != expected_epoch {
            return Err(ReferenceAsteroidEvidenceValidationError::EpochMismatch {
                index,
                expected: expected_epoch,
                found: sample.epoch,
            });
        }
        if !sample.longitude_deg.is_finite() {
            return Err(
                ReferenceAsteroidEvidenceValidationError::NonFiniteLongitude {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        if !sample.latitude_deg.is_finite() {
            return Err(
                ReferenceAsteroidEvidenceValidationError::NonFiniteLatitude {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        if !sample.distance_au.is_finite() {
            return Err(
                ReferenceAsteroidEvidenceValidationError::NonFiniteDistance {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
    }

    Ok(())
}

fn validate_reference_asteroid_equatorial_evidence(
    evidence: &[ReferenceAsteroidEquatorialEvidence],
) -> Result<(), ReferenceAsteroidEquatorialEvidenceValidationError> {
    if evidence.is_empty() {
        return Err(ReferenceAsteroidEquatorialEvidenceValidationError::Empty);
    }

    let expected_bodies = reference_asteroids();
    let exact_evidence = reference_asteroid_evidence();
    if evidence.len() != expected_bodies.len() || evidence.len() != exact_evidence.len() {
        return Err(
            ReferenceAsteroidEquatorialEvidenceValidationError::BodyOrderMismatch {
                index: evidence.len(),
                expected: expected_bodies
                    .get(evidence.len())
                    .cloned()
                    .unwrap_or_else(|| expected_bodies[expected_bodies.len() - 1].clone()),
                found: evidence
                    .last()
                    .map(|sample| sample.body.clone())
                    .unwrap_or_else(|| expected_bodies[0].clone()),
            },
        );
    }

    let expected_epoch = reference_instant();
    for (index, ((sample, expected_body), exact_sample)) in evidence
        .iter()
        .zip(expected_bodies.iter())
        .zip(exact_evidence.iter())
        .enumerate()
    {
        if sample.body != *expected_body {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::BodyOrderMismatch {
                    index,
                    expected: expected_body.clone(),
                    found: sample.body.clone(),
                },
            );
        }
        if sample.epoch != expected_epoch {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::EpochMismatch {
                    index,
                    expected: expected_epoch,
                    found: sample.epoch,
                },
            );
        }

        let exact_ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(exact_sample.longitude_deg),
            Latitude::from_degrees(exact_sample.latitude_deg),
            Some(exact_sample.distance_au),
        );
        let expected_equatorial = exact_ecliptic.to_equatorial(exact_sample.epoch.mean_obliquity());
        let actual_equatorial = &sample.equatorial;

        if (actual_equatorial.right_ascension.degrees()
            - expected_equatorial.right_ascension.degrees())
        .abs()
            > ASTEROID_EQUATORIAL_TOLERANCE_DEGREES
        {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::RightAscensionMismatch {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        if (actual_equatorial.declination.degrees() - expected_equatorial.declination.degrees())
            .abs()
            > ASTEROID_EQUATORIAL_TOLERANCE_DEGREES
        {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::DeclinationMismatch {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        match (
            actual_equatorial.distance_au,
            expected_equatorial.distance_au,
        ) {
            (Some(actual), Some(expected))
                if (actual - expected).abs() <= ASTEROID_DISTANCE_TOLERANCE_AU => {}
            (Some(_), Some(_)) => {
                return Err(
                    ReferenceAsteroidEquatorialEvidenceValidationError::DistanceMismatch {
                        index,
                        body: sample.body.clone(),
                    },
                );
            }
            _ => {
                return Err(
                    ReferenceAsteroidEquatorialEvidenceValidationError::DistanceMismatch {
                        index,
                        body: sample.body.clone(),
                    },
                );
            }
        }
    }

    Ok(())
}

/// Formats the exact asteroid evidence slice for release-facing reporting.
pub fn format_reference_asteroid_evidence_summary(
    evidence: &[ReferenceAsteroidEvidence],
) -> String {
    match validate_reference_asteroid_evidence(evidence) {
        Ok(()) => match reference_asteroid_evidence_summary_from_slice(evidence) {
            Some(summary) => match summary.validated_summary_line() {
                Ok(summary_line) => summary_line,
                Err(error) => format!("Selected asteroid evidence: unavailable ({error})"),
            },
            None => "Selected asteroid evidence: unavailable".to_string(),
        },
        Err(error) => format!("Selected asteroid evidence: unavailable ({error})"),
    }
}

/// Returns the release-facing exact asteroid evidence summary string.
pub fn reference_asteroid_evidence_summary_for_report() -> String {
    let evidence = reference_asteroid_evidence();
    match validate_reference_asteroid_evidence(evidence) {
        Ok(()) => match reference_asteroid_evidence_summary_details() {
            Some(summary) => match summary.validated_summary_line() {
                Ok(summary_line) => summary_line,
                Err(error) => format!("Selected asteroid evidence: unavailable ({error})"),
            },
            None => "Selected asteroid evidence: unavailable".to_string(),
        },
        Err(error) => format!("Selected asteroid evidence: unavailable ({error})"),
    }
}

/// Formats the equatorial asteroid evidence slice for release-facing reporting.
pub fn format_reference_asteroid_equatorial_evidence_summary(
    evidence: &[ReferenceAsteroidEquatorialEvidence],
) -> String {
    match validate_reference_asteroid_equatorial_evidence(evidence) {
        Ok(()) => match reference_asteroid_equatorial_evidence_summary_from_slice(evidence) {
            Some(summary) => match summary.validated_summary_line() {
                Ok(summary_line) => summary_line,
                Err(error) => {
                    format!("Selected asteroid equatorial evidence: unavailable ({error})")
                }
            },
            None => "Selected asteroid equatorial evidence: unavailable".to_string(),
        },
        Err(error) => format!("Selected asteroid equatorial evidence: unavailable ({error})"),
    }
}

/// Returns the release-facing equatorial asteroid evidence summary string.
pub fn reference_asteroid_equatorial_evidence_summary_for_report() -> String {
    let evidence = reference_asteroid_equatorial_evidence();
    match validate_reference_asteroid_equatorial_evidence(evidence) {
        Ok(()) => match reference_asteroid_equatorial_evidence_summary_details() {
            Some(summary) => match summary.validated_summary_line() {
                Ok(summary_line) => summary_line,
                Err(error) => {
                    format!("Selected asteroid equatorial evidence: unavailable ({error})")
                }
            },
            None => "Selected asteroid equatorial evidence: unavailable".to_string(),
        },
        Err(error) => format!("Selected asteroid equatorial evidence: unavailable ({error})"),
    }
}

/// Compact release-facing summary for the reference asteroid source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidSourceWindowSummary {
    /// Number of reference-asteroid samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ReferenceSnapshotSourceWindow>,
}

impl ReferenceAsteroidSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference asteroid source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the reference asteroid source windows still match the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceAsteroidSourceWindowSummaryValidationError> {
        let Some(expected) = reference_asteroid_source_window_summary_details() else {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated reference asteroid source window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceAsteroidSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Validation error for a reference asteroid source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceAsteroidSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in reference asteroid windows.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceAsteroidSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference asteroid source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceAsteroidSourceWindowSummaryValidationError {}

fn reference_asteroid_source_window_summary_details() -> Option<ReferenceAsteroidSourceWindowSummary>
{
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let source_windows = reference_snapshot_source_window_summary_details()?;
    let mut windows = Vec::new();
    for body in reference_asteroids() {
        if let Some(window) = source_windows
            .windows
            .iter()
            .find(|window| window.body == *body)
        {
            windows.push(window.clone());
        }
    }

    if windows.is_empty() {
        return None;
    }

    let asteroid_entries = entries
        .iter()
        .filter(|entry| is_reference_asteroid(&entry.body))
        .collect::<Vec<_>>();

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference asteroid source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference asteroid source windows should not be empty after collection");

    Some(ReferenceAsteroidSourceWindowSummary {
        sample_count: asteroid_entries.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epoch_count: asteroid_entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the reference asteroid source coverage.
pub fn reference_asteroid_source_window_summary() -> Option<ReferenceAsteroidSourceWindowSummary> {
    reference_asteroid_source_window_summary_details()
}

/// Returns the release-facing reference asteroid source window summary string.
pub fn reference_asteroid_source_window_summary_for_report() -> String {
    match reference_asteroid_source_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference asteroid source windows: unavailable ({error})"),
        },
        None => "Reference asteroid source windows: unavailable".to_string(),
    }
}

fn selected_asteroid_source_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| is_reference_asteroid(&entry.body))
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// A single body-window slice inside the expanded selected-asteroid source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSourceWindow {
    /// The selected asteroid covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl SelectedAsteroidSourceWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

/// Compact release-facing summary for the expanded selected-asteroid source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSourceSummary {
    /// Number of selected-asteroid samples in the expanded source slice.
    pub sample_count: usize,
    /// Bodies covered by the expanded selected-asteroid source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the expanded source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the expanded source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the expanded source slice.
    pub latest_epoch: Instant,
}

impl SelectedAsteroidSourceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Selected asteroid source evidence: {} source-backed samples across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the selected-asteroid evidence summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidSourceSummaryValidationError> {
        let Some(expected) = selected_asteroid_source_evidence_summary_details() else {
            return Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated selected-asteroid evidence summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Validation error for a selected-asteroid source evidence summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedAsteroidSourceSummaryValidationError {
    /// A summary field is out of sync with the checked-in selected-asteroid evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for SelectedAsteroidSourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the selected asteroid source evidence summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidSourceSummaryValidationError {}

/// Compact release-facing summary for the selected-asteroid source windows.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSourceWindowSummary {
    /// Number of selected-asteroid samples in the expanded source slice.
    pub sample_count: usize,
    /// Bodies covered by the expanded selected-asteroid source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the expanded source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the expanded source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the expanded source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<SelectedAsteroidSourceWindow>,
}

impl SelectedAsteroidSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(SelectedAsteroidSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Selected asteroid source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the selected-asteroid window summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidSourceWindowSummaryValidationError> {
        let Some(expected) = selected_asteroid_source_window_summary_details() else {
            return Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated selected-asteroid window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Validation error for a selected-asteroid source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedAsteroidSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in selected-asteroid windows.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for SelectedAsteroidSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the selected asteroid source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidSourceWindowSummaryValidationError {}

fn selected_asteroid_source_evidence_summary_details() -> Option<SelectedAsteroidSourceSummary> {
    let evidence = selected_asteroid_source_entries()?;
    let earliest_epoch = evidence
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("selected asteroid source evidence should not be empty after collection");
    let latest_epoch = evidence
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("selected asteroid source evidence should not be empty after collection");

    Some(SelectedAsteroidSourceSummary {
        sample_count: evidence.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epoch_count: evidence
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
    })
}

fn selected_asteroid_source_window_summary_details() -> Option<SelectedAsteroidSourceWindowSummary>
{
    let evidence = selected_asteroid_source_entries()?;
    let mut windows = Vec::new();
    for body in reference_asteroids() {
        let body_entries = evidence
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(SelectedAsteroidSourceWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("selected asteroid source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("selected asteroid source windows should not be empty after collection");

    Some(SelectedAsteroidSourceWindowSummary {
        sample_count: evidence.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epoch_count: evidence
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the expanded selected-asteroid source slice.
pub fn selected_asteroid_source_evidence_summary() -> Option<SelectedAsteroidSourceSummary> {
    selected_asteroid_source_evidence_summary_details()
}

/// Returns the compact typed summary for the selected-asteroid source windows.
pub fn selected_asteroid_source_window_summary() -> Option<SelectedAsteroidSourceWindowSummary> {
    selected_asteroid_source_window_summary_details()
}

/// Returns the release-facing expanded selected-asteroid source coverage summary string.
pub fn selected_asteroid_source_evidence_summary_for_report() -> String {
    match selected_asteroid_source_evidence_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Selected asteroid source evidence: unavailable ({error})"),
        },
        None => "Selected asteroid source evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing selected-asteroid source-window summary string.
pub fn selected_asteroid_source_window_summary_for_report() -> String {
    match selected_asteroid_source_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Selected asteroid source windows: unavailable ({error})"),
        },
        None => "Selected asteroid source windows: unavailable".to_string(),
    }
}

const SELECTED_ASTEROID_BOUNDARY_EPOCHS: &[f64] = &[2_451_914.5, 2_451_915.5];
const SELECTED_ASTEROID_TERMINAL_BOUNDARY_EPOCH_JD: f64 = 2_500_000.0;

fn selected_asteroid_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_reference_asteroid(&entry.body)
                        && SELECTED_ASTEROID_BOUNDARY_EPOCHS
                            .contains(&entry.epoch.julian_day.days())
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the selected-asteroid boundary-day evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epochs shared by the boundary slice.
    pub epochs: Vec<Instant>,
}

/// Validation errors for a selected-asteroid boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch list drifted from the current evidence slice.
    EpochOrderMismatch {
        index: usize,
        expected: Instant,
        found: Instant,
    },
}

impl fmt::Display for SelectedAsteroidBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid boundary evidence epoch order mismatch at index {index}: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidBoundarySummaryValidationError {}

impl SelectedAsteroidBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let epochs = match self.epochs.as_slice() {
            [] => String::from("(no epochs)"),
            [epoch] => format_instant(*epoch),
            [first, .., last] => format!("{}..{}", format_instant(*first), format_instant(*last)),
        };
        format!(
            "Selected asteroid boundary evidence: {} exact samples across {} epochs at {} ({})",
            self.sample_count,
            self.epochs.len(),
            epochs,
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidBoundarySummaryValidationError> {
        let evidence = selected_asteroid_boundary_entries()
            .ok_or(SelectedAsteroidBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.sample_bodies.as_slice() != reference_asteroids() {
            for (index, (expected, found)) in reference_asteroids()
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        SelectedAsteroidBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        let expected_epochs = SELECTED_ASTEROID_BOUNDARY_EPOCHS
            .iter()
            .copied()
            .map(|days| Instant::new(JulianDay::from_days(days), TimeScale::Tdb))
            .collect::<Vec<_>>();
        if self.epochs.len() != expected_epochs.len() {
            return Err(
                SelectedAsteroidBoundarySummaryValidationError::EpochOrderMismatch {
                    index: self.epochs.len(),
                    expected: expected_epochs[0],
                    found: self.epochs.first().copied().unwrap_or(expected_epochs[0]),
                },
            );
        }
        for (index, (expected, found)) in expected_epochs.iter().zip(self.epochs.iter()).enumerate()
        {
            if expected != found {
                return Err(
                    SelectedAsteroidBoundarySummaryValidationError::EpochOrderMismatch {
                        index,
                        expected: *expected,
                        found: *found,
                    },
                );
            }
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn selected_asteroid_boundary_summary_details() -> Option<SelectedAsteroidBoundarySummary> {
    let evidence = selected_asteroid_boundary_entries()?;
    let mut epochs = Vec::new();
    for entry in evidence {
        if epochs.last().copied() != Some(entry.epoch) {
            epochs.push(entry.epoch);
        }
    }
    Some(SelectedAsteroidBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epochs,
    })
}

/// Returns the compact typed summary for the selected-asteroid boundary-day evidence.
pub fn selected_asteroid_boundary_summary() -> Option<SelectedAsteroidBoundarySummary> {
    selected_asteroid_boundary_summary_details()
}

/// Returns the release-facing selected-asteroid boundary-day summary string.
pub fn selected_asteroid_boundary_summary_for_report() -> String {
    match selected_asteroid_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Selected asteroid boundary evidence: unavailable ({error})"),
        },
        None => "Selected asteroid boundary evidence: unavailable".to_string(),
    }
}

fn selected_asteroid_terminal_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_reference_asteroid(&entry.body)
                        && entry.epoch.julian_day.days()
                            == SELECTED_ASTEROID_TERMINAL_BOUNDARY_EPOCH_JD
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the terminal selected-asteroid boundary day.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidTerminalBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a selected-asteroid terminal boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidTerminalBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for SelectedAsteroidTerminalBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid terminal boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid terminal boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid terminal boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid terminal boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidTerminalBoundarySummaryValidationError {}

impl SelectedAsteroidTerminalBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference selected-asteroid terminal boundary evidence: {} exact samples at {} ({}); 2500-01-01 terminal boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidTerminalBoundarySummaryValidationError> {
        let evidence = selected_asteroid_terminal_boundary_entries()
            .ok_or(SelectedAsteroidTerminalBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidTerminalBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        SelectedAsteroidTerminalBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidTerminalBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                SelectedAsteroidTerminalBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidTerminalBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidTerminalBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn selected_asteroid_terminal_boundary_summary_details(
) -> Option<SelectedAsteroidTerminalBoundarySummary> {
    let evidence = selected_asteroid_terminal_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(SelectedAsteroidTerminalBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the terminal selected-asteroid boundary evidence.
pub fn selected_asteroid_terminal_boundary_summary(
) -> Option<SelectedAsteroidTerminalBoundarySummary> {
    selected_asteroid_terminal_boundary_summary_details()
}

/// Returns the release-facing terminal selected-asteroid boundary summary string.
pub fn selected_asteroid_terminal_boundary_summary_for_report() -> String {
    match selected_asteroid_terminal_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Selected asteroid terminal boundary evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid terminal boundary evidence: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the mixed-frame selected-asteroid batch parity slice.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidBatchParitySummary {
    /// Number of requests in the mixed-frame batch parity slice.
    pub request_count: usize,
    /// Bodies covered by the batch parity slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the selected-asteroid batch parity slice.
    pub epoch: Instant,
    /// Number of ecliptic-frame requests in the mixed-frame batch parity slice.
    pub ecliptic_count: usize,
    /// Number of equatorial-frame requests in the mixed-frame batch parity slice.
    pub equatorial_count: usize,
    /// Whether the batch and single-request results stayed in parity.
    pub parity_preserved: bool,
}

/// Validation errors for a selected-asteroid batch parity summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidBatchParitySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary request count drifted from the current evidence slice.
    RequestCountMismatch {
        request_count: usize,
        derived_request_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
    /// The summary frame mix drifted from the current evidence slice.
    FrameMixMismatch {
        ecliptic_count: usize,
        equatorial_count: usize,
        derived_ecliptic_count: usize,
        derived_equatorial_count: usize,
    },
    /// The batch/single parity posture drifted from the current evidence slice.
    ParityPreservedMismatch { expected: bool, found: bool },
}

impl fmt::Display for SelectedAsteroidBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid batch parity evidence is unavailable"),
            Self::RequestCountMismatch {
                request_count,
                derived_request_count,
            } => write!(
                f,
                "selected asteroid batch parity request count {request_count} does not match derived request count {derived_request_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid batch parity body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid batch parity epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::FrameMixMismatch {
                ecliptic_count,
                equatorial_count,
                derived_ecliptic_count,
                derived_equatorial_count,
            } => write!(
                f,
                "selected asteroid batch parity frame mix mismatch: expected {ecliptic_count} ecliptic and {equatorial_count} equatorial, found {derived_ecliptic_count} ecliptic and {derived_equatorial_count} equatorial"
            ),
            Self::ParityPreservedMismatch { expected, found } => write!(
                f,
                "selected asteroid batch parity preserved flag mismatch: expected {expected}, found {found}"
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidBatchParitySummaryValidationError {}

impl SelectedAsteroidBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let parity = if self.parity_preserved {
            "preserved"
        } else {
            "not preserved"
        };

        format!(
            "Selected asteroid batch parity: {} requests across {} bodies at {} ({}); frame mix: {} ecliptic, {} equatorial; batch/single parity {}",
            self.request_count,
            self.sample_bodies.len(),
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
            self.ecliptic_count,
            self.equatorial_count,
            parity,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidBatchParitySummaryValidationError> {
        let requests = reference_asteroid_batch_parity_requests()
            .ok_or(SelectedAsteroidBatchParitySummaryValidationError::Empty)?;
        let evidence = reference_asteroid_evidence();

        if self.request_count != requests.len() {
            return Err(
                SelectedAsteroidBatchParitySummaryValidationError::RequestCountMismatch {
                    request_count: self.request_count,
                    derived_request_count: requests.len(),
                },
            );
        }
        if self.sample_bodies.as_slice() != reference_asteroids() {
            for (index, (expected, found)) in reference_asteroids()
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        SelectedAsteroidBatchParitySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidBatchParitySummaryValidationError::RequestCountMismatch {
                    request_count: self.request_count,
                    derived_request_count: requests.len(),
                },
            );
        }
        if self.epoch != requests[0].instant {
            return Err(
                SelectedAsteroidBatchParitySummaryValidationError::EpochMismatch {
                    expected: requests[0].instant,
                    found: self.epoch,
                },
            );
        }

        let derived_ecliptic_count = requests
            .iter()
            .filter(|request| matches!(request.frame, CoordinateFrame::Ecliptic))
            .count();
        let derived_equatorial_count = requests.len() - derived_ecliptic_count;
        if self.ecliptic_count != derived_ecliptic_count
            || self.equatorial_count != derived_equatorial_count
        {
            return Err(
                SelectedAsteroidBatchParitySummaryValidationError::FrameMixMismatch {
                    ecliptic_count: self.ecliptic_count,
                    equatorial_count: self.equatorial_count,
                    derived_ecliptic_count,
                    derived_equatorial_count,
                },
            );
        }

        let backend = JplSnapshotBackend;
        let results = backend
            .positions(&requests)
            .map_err(|_| SelectedAsteroidBatchParitySummaryValidationError::Empty)?;

        let mut parity_preserved =
            results.len() == requests.len() && evidence.len() == requests.len();
        for ((request, result), expected) in
            requests.iter().zip(results.iter()).zip(evidence.iter())
        {
            parity_preserved &= result.body == request.body;
            parity_preserved &= result.instant == request.instant;
            parity_preserved &= result.frame == request.frame;
            parity_preserved &= result.quality == QualityAnnotation::Exact;

            let ecliptic = match result.ecliptic {
                Some(value) => value,
                None => {
                    parity_preserved = false;
                    continue;
                }
            };
            parity_preserved &=
                (ecliptic.longitude.degrees() - expected.longitude_deg).abs() < 1e-12;
            parity_preserved &= (ecliptic.latitude.degrees() - expected.latitude_deg).abs() < 1e-12;
            parity_preserved &= (ecliptic
                .distance_au
                .expect("selected asteroid batch rows should include distance")
                - expected.distance_au)
                .abs()
                < 1e-12;

            let equatorial = match result.equatorial {
                Some(value) => value,
                None => {
                    parity_preserved = false;
                    continue;
                }
            };
            parity_preserved &=
                equatorial == ecliptic.to_equatorial(result.instant.mean_obliquity());
        }

        if self.parity_preserved != parity_preserved {
            return Err(
                SelectedAsteroidBatchParitySummaryValidationError::ParityPreservedMismatch {
                    expected: parity_preserved,
                    found: self.parity_preserved,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn selected_asteroid_batch_parity_summary_details() -> Option<SelectedAsteroidBatchParitySummary> {
    let requests = reference_asteroid_batch_parity_requests()?;
    let evidence = reference_asteroid_evidence();
    let backend = JplSnapshotBackend;
    let results = backend.positions(&requests).ok()?;

    let mut parity_preserved = results.len() == requests.len() && evidence.len() == requests.len();
    for ((request, result), expected) in requests.iter().zip(results.iter()).zip(evidence.iter()) {
        parity_preserved &= result.body == request.body;
        parity_preserved &= result.instant == request.instant;
        parity_preserved &= result.frame == request.frame;
        parity_preserved &= result.quality == QualityAnnotation::Exact;

        let Some(ecliptic) = result.ecliptic else {
            parity_preserved = false;
            continue;
        };
        parity_preserved &= (ecliptic.longitude.degrees() - expected.longitude_deg).abs() < 1e-12;
        parity_preserved &= (ecliptic.latitude.degrees() - expected.latitude_deg).abs() < 1e-12;
        parity_preserved &= (ecliptic
            .distance_au
            .expect("selected asteroid batch rows should include distance")
            - expected.distance_au)
            .abs()
            < 1e-12;

        let Some(equatorial) = result.equatorial else {
            parity_preserved = false;
            continue;
        };
        parity_preserved &= equatorial == ecliptic.to_equatorial(result.instant.mean_obliquity());
    }

    let first = requests.first()?;
    Some(SelectedAsteroidBatchParitySummary {
        request_count: requests.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epoch: first.instant,
        ecliptic_count: requests
            .iter()
            .filter(|request| matches!(request.frame, CoordinateFrame::Ecliptic))
            .count(),
        equatorial_count: requests
            .iter()
            .filter(|request| matches!(request.frame, CoordinateFrame::Equatorial))
            .count(),
        parity_preserved,
    })
}

/// Returns the compact typed summary for the selected-asteroid batch-parity slice.
pub fn selected_asteroid_batch_parity_summary() -> Option<SelectedAsteroidBatchParitySummary> {
    selected_asteroid_batch_parity_summary_details()
}

/// Returns the release-facing selected-asteroid batch-parity summary string.
pub fn selected_asteroid_batch_parity_summary_for_report() -> String {
    match selected_asteroid_batch_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Selected asteroid batch parity: unavailable ({error})"),
        },
        None => "Selected asteroid batch parity: unavailable".to_string(),
    }
}

const REFERENCE_LUNAR_BOUNDARY_EPOCHS: [f64; 2] = [2_451_911.5, 2_451_912.5];
const REFERENCE_HIGH_CURVATURE_EPOCHS: [f64; 5] = [
    2_451_911.5,
    2_451_912.5,
    2_451_913.5,
    2_451_914.5,
    2_451_916.5,
];
const REFERENCE_MAJOR_BODY_BOUNDARY_EPOCH: f64 = 2_451_917.5;
const REFERENCE_MARS_JUPITER_BOUNDARY_EPOCH: f64 = 2_451_918.5;

fn reference_snapshot_lunar_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    entry.body == pleiades_backend::CelestialBody::Moon
                        && REFERENCE_LUNAR_BOUNDARY_EPOCHS.contains(&entry.epoch.julian_day.days())
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

fn reference_snapshot_high_curvature_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && REFERENCE_HIGH_CURVATURE_EPOCHS.contains(&entry.epoch.julian_day.days())
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the Moon high-curvature reference window.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceLunarBoundarySummary {
    /// Number of exact Moon samples in the reference window.
    pub sample_count: usize,
    /// Number of distinct epochs in the reference window.
    pub epoch_count: usize,
    /// Earliest epoch represented in the reference window.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the reference window.
    pub latest_epoch: Instant,
}

impl ReferenceLunarBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference lunar boundary evidence: {} exact Moon samples at {}..{}; high-curvature interpolation window",
            self.sample_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the Moon boundary summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceLunarBoundarySummaryValidationError> {
        let Some(expected) = reference_snapshot_lunar_boundary_summary_details() else {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated Moon boundary summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceLunarBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceLunarBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a Moon boundary summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceLunarBoundarySummaryValidationError {
    /// A summary field is out of sync with the checked-in lunar boundary evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceLunarBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference lunar boundary summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceLunarBoundarySummaryValidationError {}

fn reference_snapshot_lunar_boundary_summary_details() -> Option<ReferenceLunarBoundarySummary> {
    let entries = reference_snapshot_lunar_boundary_entries()?;
    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference lunar boundary evidence should not be empty after collection");
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference lunar boundary evidence should not be empty after collection");

    Some(ReferenceLunarBoundarySummary {
        sample_count: entries.len(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Returns the compact typed summary for the Moon high-curvature reference window.
pub fn reference_snapshot_lunar_boundary_summary() -> Option<ReferenceLunarBoundarySummary> {
    reference_snapshot_lunar_boundary_summary_details()
}

/// Returns the release-facing Moon high-curvature reference window summary string.
pub fn reference_snapshot_lunar_boundary_summary_for_report() -> String {
    match reference_snapshot_lunar_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference lunar boundary evidence: unavailable ({error})"),
        },
        None => "Reference lunar boundary evidence: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the major-body high-curvature reference window.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureSummary {
    /// Number of exact major-body samples in the reference window.
    pub sample_count: usize,
    /// Number of distinct bodies in the reference window.
    pub body_count: usize,
    /// Bodies represented by the reference window in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs in the reference window.
    pub epoch_count: usize,
    /// Earliest epoch represented in the reference window.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the reference window.
    pub latest_epoch: Instant,
}

impl ReferenceHighCurvatureSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference major-body high-curvature evidence: {} exact samples across {} bodies and {} epochs ({}..{}); bodies: {}; high-curvature interpolation window",
            self.sample_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.bodies),
        )
    }

    /// Returns `Ok(())` when the major-body high-curvature summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceHighCurvatureSummaryValidationError> {
        let Some(expected) = reference_snapshot_high_curvature_summary_details() else {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != expected.body_count {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync { field: "bodies" },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated major-body high-curvature summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHighCurvatureSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHighCurvatureSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a major-body high-curvature summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHighCurvatureSummaryValidationError {
    /// A summary field is out of sync with the checked-in high-curvature evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceHighCurvatureSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference high-curvature summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceHighCurvatureSummaryValidationError {}

fn reference_snapshot_high_curvature_summary_details() -> Option<ReferenceHighCurvatureSummary> {
    let entries = reference_snapshot_high_curvature_entries()?;
    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");

    let mut bodies = Vec::new();
    let mut epochs = BTreeSet::new();
    for entry in entries {
        if !bodies.contains(&entry.body) {
            bodies.push(entry.body.clone());
        }
        epochs.insert(entry.epoch.julian_day.days().to_bits());
    }

    Some(ReferenceHighCurvatureSummary {
        sample_count: entries.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Returns the compact typed summary for the major-body high-curvature reference window.
pub fn reference_snapshot_high_curvature_summary() -> Option<ReferenceHighCurvatureSummary> {
    reference_snapshot_high_curvature_summary_details()
}

/// Returns the release-facing major-body high-curvature reference window summary string.
pub fn reference_snapshot_high_curvature_summary_for_report() -> String {
    match reference_snapshot_high_curvature_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body high-curvature evidence: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature evidence: unavailable".to_string(),
    }
}

fn reference_snapshot_major_body_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days() == REFERENCE_MAJOR_BODY_BOUNDARY_EPOCH
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the major-body boundary-day reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceMajorBodyBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for ReferenceMajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceMajorBodyBoundarySummaryValidationError {}

impl ReferenceMajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference major-body boundary evidence: {} exact samples at {} ({}); 2001-01-08 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceMajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_major_body_boundary_entries()
            .ok_or(ReferenceMajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceMajorBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceMajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_major_body_boundary_summary_details(
) -> Option<ReferenceMajorBodyBoundarySummary> {
    let evidence = reference_snapshot_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceMajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the major-body boundary-day reference evidence.
pub fn reference_snapshot_major_body_boundary_summary() -> Option<ReferenceMajorBodyBoundarySummary>
{
    reference_snapshot_major_body_boundary_summary_details()
}

/// Returns the release-facing major-body boundary-day summary string.
pub fn reference_snapshot_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference major-body boundary evidence: unavailable".to_string(),
    }
}

fn reference_snapshot_mars_jupiter_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    entry.epoch.julian_day.days() == REFERENCE_MARS_JUPITER_BOUNDARY_EPOCH
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the Mars/Jupiter boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMarsJupiterBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a Mars/Jupiter boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceMarsJupiterBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for ReferenceMarsJupiterBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference Mars/Jupiter boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference Mars/Jupiter boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference Mars/Jupiter boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference Mars/Jupiter boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceMarsJupiterBoundarySummaryValidationError {}

impl ReferenceMarsJupiterBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference Mars/Jupiter boundary evidence: {} exact samples at {} ({}); 2001-01-09 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceMarsJupiterBoundarySummaryValidationError> {
        let evidence = reference_snapshot_mars_jupiter_boundary_entries()
            .ok_or(ReferenceMarsJupiterBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceMarsJupiterBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let expected_bodies = vec![
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
        ];
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceMarsJupiterBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceMarsJupiterBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceMarsJupiterBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceMarsJupiterBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMarsJupiterBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_mars_jupiter_boundary_summary_details(
) -> Option<ReferenceMarsJupiterBoundarySummary> {
    let evidence = reference_snapshot_mars_jupiter_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceMarsJupiterBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the Mars/Jupiter boundary reference evidence.
pub fn reference_snapshot_mars_jupiter_boundary_summary(
) -> Option<ReferenceMarsJupiterBoundarySummary> {
    reference_snapshot_mars_jupiter_boundary_summary_details()
}

/// Returns the release-facing Mars/Jupiter boundary summary string.
pub fn reference_snapshot_mars_jupiter_boundary_summary_for_report() -> String {
    match reference_snapshot_mars_jupiter_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference Mars/Jupiter boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference Mars/Jupiter boundary evidence: unavailable".to_string(),
    }
}

fn reference_snapshot_1749_major_body_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_1749_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the 1749-12-31 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1749MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 1749 major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1749MajorBodyBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for Reference1749MajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 1749 major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1749 major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1749 major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1749 major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1749MajorBodyBoundarySummaryValidationError {}

impl Reference1749MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1749 major-body boundary evidence: {} exact samples at {} ({}); 1749-12-31 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1749MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1749_major_body_boundary_entries()
            .ok_or(Reference1749MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1749MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        Reference1749MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1749MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1749MajorBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference1749MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1749MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_1749_major_body_boundary_summary_details(
) -> Option<Reference1749MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_1749_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1749MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1749 major-body boundary reference evidence.
pub fn reference_snapshot_1749_major_body_boundary_summary(
) -> Option<Reference1749MajorBodyBoundarySummary> {
    reference_snapshot_1749_major_body_boundary_summary_details()
}

/// Returns the release-facing 1749 major-body boundary summary string.
pub fn reference_snapshot_1749_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1749_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1749 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1749 major-body boundary evidence: unavailable".to_string(),
    }
}

fn reference_snapshot_early_major_body_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_REFERENCE_ONLY_EPOCH_JD
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the early major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceEarlyMajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for an early major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceEarlyMajorBodyBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for ReferenceEarlyMajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference early major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference early major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference early major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference early major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceEarlyMajorBodyBoundarySummaryValidationError {}

impl ReferenceEarlyMajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference early major-body boundary evidence: {} exact samples at {} ({})",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceEarlyMajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_early_major_body_boundary_entries()
            .ok_or(ReferenceEarlyMajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceEarlyMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceEarlyMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceEarlyMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceEarlyMajorBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceEarlyMajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceEarlyMajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_early_major_body_boundary_summary_details(
) -> Option<ReferenceEarlyMajorBodyBoundarySummary> {
    let evidence = reference_snapshot_early_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceEarlyMajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the early major-body boundary reference evidence.
pub fn reference_snapshot_early_major_body_boundary_summary(
) -> Option<ReferenceEarlyMajorBodyBoundarySummary> {
    reference_snapshot_early_major_body_boundary_summary_details()
}

/// Returns the release-facing early major-body boundary summary string.
pub fn reference_snapshot_early_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_early_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference early major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference early major-body boundary evidence: unavailable".to_string(),
    }
}

fn reference_snapshot_1800_major_body_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_1800_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the 1800-01-03 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1800MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for an 1800 major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1800MajorBodyBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for Reference1800MajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 1800 major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1800 major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1800 major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1800 major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1800MajorBodyBoundarySummaryValidationError {}

impl Reference1800MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1800 major-body boundary evidence: {} exact samples at {} ({}); 1800-01-03 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1800MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1800_major_body_boundary_entries()
            .ok_or(Reference1800MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1800MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        Reference1800MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1800MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1800MajorBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference1800MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1800MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_1800_major_body_boundary_summary_details(
) -> Option<Reference1800MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_1800_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1800MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1800 major-body boundary reference evidence.
pub fn reference_snapshot_1800_major_body_boundary_summary(
) -> Option<Reference1800MajorBodyBoundarySummary> {
    reference_snapshot_1800_major_body_boundary_summary_details()
}

/// Returns the release-facing 1800 major-body boundary summary string.
pub fn reference_snapshot_1800_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1800_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1800 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1800 major-body boundary evidence: unavailable".to_string(),
    }
}

fn reference_snapshot_2500_major_body_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2500_MAJOR_BODY_BOUNDARY_EPOCH_JD
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the 2500-01-01 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2500MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 2500 major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2500MajorBodyBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for Reference2500MajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 2500 major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2500 major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2500 major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2500 major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2500MajorBodyBoundarySummaryValidationError {}

impl Reference2500MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2500 major-body boundary evidence: {} exact samples at {} ({}); 2500-01-01 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2500MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_2500_major_body_boundary_entries()
            .ok_or(Reference2500MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2500MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        Reference2500MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2500MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2500MajorBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference2500MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2500MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_2500_major_body_boundary_summary_details(
) -> Option<Reference2500MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2500_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2500MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2500 major-body boundary reference evidence.
pub fn reference_snapshot_2500_major_body_boundary_summary(
) -> Option<Reference2500MajorBodyBoundarySummary> {
    reference_snapshot_2500_major_body_boundary_summary_details()
}

/// Returns the release-facing 2500 major-body boundary summary string.
pub fn reference_snapshot_2500_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2500_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2500 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2500 major-body boundary evidence: unavailable".to_string(),
    }
}

fn reference_snapshot_mars_outer_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    entry.body == pleiades_backend::CelestialBody::Mars
                        && matches!(
                            entry.epoch.julian_day.days(),
                            value if value == 2_600_000.0 || value == 2_634_167.0
                        )
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the Mars outer-boundary reference window.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMarsOuterBoundarySummary {
    /// Number of exact samples in the outer-boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the outer-boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs represented by the outer-boundary slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the outer-boundary slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the outer-boundary slice.
    pub latest_epoch: Instant,
}

/// Validation errors for a Mars outer-boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceMarsOuterBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch count drifted from the current evidence slice.
    EpochCountMismatch { expected: usize, found: usize },
    /// The summary earliest epoch drifted from the current evidence slice.
    EarliestEpochMismatch { expected: Instant, found: Instant },
    /// The summary latest epoch drifted from the current evidence slice.
    LatestEpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for ReferenceMarsOuterBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference Mars outer-boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference Mars outer-boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference Mars outer-boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochCountMismatch { expected, found } => write!(
                f,
                "reference Mars outer-boundary evidence epoch count mismatch: expected {expected}, found {found}"
            ),
            Self::EarliestEpochMismatch { expected, found } => write!(
                f,
                "reference Mars outer-boundary evidence earliest epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::LatestEpochMismatch { expected, found } => write!(
                f,
                "reference Mars outer-boundary evidence latest epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceMarsOuterBoundarySummaryValidationError {}

impl ReferenceMarsOuterBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference Mars outer-boundary evidence: {} exact samples at {}..{} ({}); outer boundary interpolation window",
            self.sample_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceMarsOuterBoundarySummaryValidationError> {
        let evidence = reference_snapshot_mars_outer_boundary_entries()
            .ok_or(ReferenceMarsOuterBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceMarsOuterBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceMarsOuterBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceMarsOuterBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut epochs = evidence
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>();
        let expected_epoch_count = epochs.len();
        if self.epoch_count != expected_epoch_count {
            return Err(
                ReferenceMarsOuterBoundarySummaryValidationError::EpochCountMismatch {
                    expected: expected_epoch_count,
                    found: self.epoch_count,
                },
            );
        }

        let expected_earliest_epoch = evidence
            .iter()
            .min_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .total_cmp(&right.epoch.julian_day.days())
            })
            .expect("reference Mars outer-boundary evidence should not be empty after collection")
            .epoch;
        let expected_latest_epoch = evidence
            .iter()
            .max_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .total_cmp(&right.epoch.julian_day.days())
            })
            .expect("reference Mars outer-boundary evidence should not be empty after collection")
            .epoch;

        if self.earliest_epoch != expected_earliest_epoch {
            return Err(
                ReferenceMarsOuterBoundarySummaryValidationError::EarliestEpochMismatch {
                    expected: expected_earliest_epoch,
                    found: self.earliest_epoch,
                },
            );
        }
        if self.latest_epoch != expected_latest_epoch {
            return Err(
                ReferenceMarsOuterBoundarySummaryValidationError::LatestEpochMismatch {
                    expected: expected_latest_epoch,
                    found: self.latest_epoch,
                },
            );
        }

        if epochs.remove(&self.earliest_epoch.julian_day.days().to_bits())
            && epochs.remove(&self.latest_epoch.julian_day.days().to_bits())
        {
            Ok(())
        } else {
            Err(
                ReferenceMarsOuterBoundarySummaryValidationError::EpochCountMismatch {
                    expected: expected_epoch_count,
                    found: self.epoch_count,
                },
            )
        }
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceMarsOuterBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMarsOuterBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_mars_outer_boundary_summary_details(
) -> Option<ReferenceMarsOuterBoundarySummary> {
    let evidence = reference_snapshot_mars_outer_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    let epochs = evidence
        .iter()
        .map(|entry| entry.epoch.julian_day.days().to_bits())
        .collect::<BTreeSet<_>>();
    let earliest_epoch = evidence
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .expect("reference Mars outer-boundary evidence should not be empty after collection")
        .epoch;
    let latest_epoch = evidence
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .expect("reference Mars outer-boundary evidence should not be empty after collection")
        .epoch;

    Some(ReferenceMarsOuterBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Returns the compact typed summary for the Mars outer-boundary reference evidence.
pub fn reference_snapshot_mars_outer_boundary_summary() -> Option<ReferenceMarsOuterBoundarySummary>
{
    reference_snapshot_mars_outer_boundary_summary_details()
}

/// Returns the release-facing Mars outer-boundary summary string.
pub fn reference_snapshot_mars_outer_boundary_summary_for_report() -> String {
    match reference_snapshot_mars_outer_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference Mars outer-boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference Mars outer-boundary evidence: unavailable".to_string(),
    }
}

/// A single body-window slice inside the major-body boundary-day reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMajorBodyBoundaryWindow {
    /// The body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ReferenceMajorBodyBoundaryWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

impl fmt::Display for ReferenceMajorBodyBoundaryWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the major-body boundary-day reference coverage windows.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMajorBodyBoundaryWindowSummary {
    /// Number of samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the boundary slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the boundary slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the boundary slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ReferenceMajorBodyBoundaryWindow>,
}

/// Validation errors for a major-body boundary window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceMajorBodyBoundaryWindowSummaryValidationError {
    /// The summary no longer matches the checked-in boundary window slice.
    DerivedSummaryMismatch,
}

impl fmt::Display for ReferenceMajorBodyBoundaryWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DerivedSummaryMismatch => write!(
                f,
                "the reference major-body boundary window summary is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceMajorBodyBoundaryWindowSummaryValidationError {}

impl ReferenceMajorBodyBoundaryWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceMajorBodyBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Reference major-body boundary windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current boundary window slice.
    pub fn validate(&self) -> Result<(), ReferenceMajorBodyBoundaryWindowSummaryValidationError> {
        let Some(expected) = reference_snapshot_major_body_boundary_window_summary_details() else {
            return Err(
                ReferenceMajorBodyBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        };

        if self != &expected {
            return Err(
                ReferenceMajorBodyBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns the validated summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceMajorBodyBoundaryWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMajorBodyBoundaryWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_major_body_boundary_window_summary_details(
) -> Option<ReferenceMajorBodyBoundaryWindowSummary> {
    let entries = reference_snapshot_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in entries {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    let mut windows = Vec::new();
    for body in &sample_bodies {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(ReferenceMajorBodyBoundaryWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference major-body boundary windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference major-body boundary windows should not be empty after collection");

    Some(ReferenceMajorBodyBoundaryWindowSummary {
        sample_count: entries.len(),
        sample_bodies,
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the major-body boundary-day reference coverage windows.
pub fn reference_snapshot_major_body_boundary_window_summary(
) -> Option<ReferenceMajorBodyBoundaryWindowSummary> {
    static SUMMARY: OnceLock<ReferenceMajorBodyBoundaryWindowSummary> = OnceLock::new();
    Some(
        SUMMARY
            .get_or_init(|| {
                reference_snapshot_major_body_boundary_window_summary_details().expect(
                    "reference major-body boundary windows should not be empty after collection",
                )
            })
            .clone(),
    )
}

/// Returns the release-facing major-body boundary-day window summary string.
pub fn reference_snapshot_major_body_boundary_window_summary_for_report() -> String {
    match reference_snapshot_major_body_boundary_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body boundary windows: unavailable ({error})")
            }
        },
        None => "Reference major-body boundary windows: unavailable".to_string(),
    }
}

/// A single epoch slice inside the reference snapshot boundary coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotBoundaryEpochCoverage {
    /// The epoch covered by this slice.
    pub epoch: Instant,
    /// Number of bodies covered at the epoch.
    pub body_count: usize,
    /// Bodies covered by the epoch slice in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
}

const REFERENCE_SPARSE_BOUNDARY_EPOCH_JD: f64 = 2_451_915.5;

impl ReferenceSnapshotBoundaryEpochCoverage {
    /// Returns a compact epoch-coverage summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let sparse_note = if self.epoch.julian_day.days() == REFERENCE_SPARSE_BOUNDARY_EPOCH_JD {
            "; sparse asteroid-only day"
        } else {
            ""
        };

        format!(
            "{}: {} bodies ({}){}",
            format_instant(self.epoch),
            self.body_count,
            format_bodies(&self.bodies),
            sparse_note,
        )
    }
}

impl fmt::Display for ReferenceSnapshotBoundaryEpochCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the reference snapshot boundary-window epoch coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotBoundaryEpochCoverageSummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Number of distinct epochs covered by the boundary slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the boundary slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the boundary slice.
    pub latest_epoch: Instant,
    /// Per-epoch body breakdown in first-seen order.
    pub windows: Vec<ReferenceSnapshotBoundaryEpochCoverage>,
}

/// Validation error for a boundary-window epoch summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in epoch coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference snapshot boundary epoch coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError {}

impl ReferenceSnapshotBoundaryEpochCoverageSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceSnapshotBoundaryEpochCoverage::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference snapshot boundary epoch coverage: {} exact samples across {} epochs ({}..{}); epochs: {}",
            self.sample_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the epoch coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError> {
        let Some(expected) = reference_snapshot_boundary_epoch_coverage_summary_details() else {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated epoch coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotBoundaryEpochCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn reference_snapshot_boundary_epoch_coverage_summary_details(
) -> Option<ReferenceSnapshotBoundaryEpochCoverageSummary> {
    let entries = reference_snapshot()
        .iter()
        .filter(|entry| (2_451_913.5..=2_451_917.5).contains(&entry.epoch.julian_day.days()))
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return None;
    }

    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .expect("reference snapshot boundary epoch coverage should not be empty after collection")
        .epoch;
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .expect("reference snapshot boundary epoch coverage should not be empty after collection")
        .epoch;

    let mut windows = BTreeMap::<u64, ReferenceSnapshotBoundaryEpochCoverage>::new();
    for entry in &entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        let window =
            windows
                .entry(epoch_bits)
                .or_insert_with(|| ReferenceSnapshotBoundaryEpochCoverage {
                    epoch: entry.epoch,
                    body_count: 0,
                    bodies: Vec::new(),
                });
        if !window.bodies.contains(&entry.body) {
            window.bodies.push(entry.body.clone());
            window.body_count = window.bodies.len();
        }
    }

    Some(ReferenceSnapshotBoundaryEpochCoverageSummary {
        sample_count: entries.len(),
        epoch_count: windows.len(),
        earliest_epoch,
        latest_epoch,
        windows: windows.into_values().collect(),
    })
}

/// Returns the compact typed summary for the reference snapshot boundary-window epoch coverage.
pub fn reference_snapshot_boundary_epoch_coverage_summary(
) -> Option<ReferenceSnapshotBoundaryEpochCoverageSummary> {
    reference_snapshot_boundary_epoch_coverage_summary_details()
}

/// Returns the release-facing reference snapshot boundary-window epoch coverage summary string.
pub fn reference_snapshot_boundary_epoch_coverage_summary_for_report() -> String {
    match reference_snapshot_boundary_epoch_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference snapshot boundary epoch coverage: unavailable ({error})")
            }
        },
        None => "Reference snapshot boundary epoch coverage: unavailable".to_string(),
    }
}

/// A single body-window slice inside the major-body high-curvature reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureWindow {
    /// The body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ReferenceHighCurvatureWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

impl fmt::Display for ReferenceHighCurvatureWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the major-body high-curvature reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureWindowSummary {
    /// Number of samples in the high-curvature slice.
    pub sample_count: usize,
    /// Bodies covered by the high-curvature slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the high-curvature slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the high-curvature slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the high-curvature slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ReferenceHighCurvatureWindow>,
}

impl ReferenceHighCurvatureWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceHighCurvatureWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference major-body high-curvature windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the high-curvature window summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceHighCurvatureWindowSummaryValidationError> {
        let Some(expected) = reference_snapshot_high_curvature_window_summary_details() else {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated high-curvature window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHighCurvatureWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHighCurvatureWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a high-curvature window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHighCurvatureWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in high-curvature windows.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceHighCurvatureWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference high-curvature window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceHighCurvatureWindowSummaryValidationError {}

fn reference_snapshot_high_curvature_window_summary_details(
) -> Option<ReferenceHighCurvatureWindowSummary> {
    let entries = reference_snapshot_high_curvature_entries()?;
    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");

    let mut sample_bodies = Vec::new();
    let mut windows = Vec::new();
    for entry in entries {
        if sample_bodies.contains(&entry.body) {
            continue;
        }
        let body_entries = entries
            .iter()
            .filter(|candidate| candidate.body == entry.body)
            .collect::<Vec<_>>();
        let body_earliest_epoch = body_entries
            .iter()
            .min_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .total_cmp(&right.epoch.julian_day.days())
            })
            .map(|candidate| candidate.epoch)
            .expect("reference high-curvature body window should not be empty after collection");
        let body_latest_epoch = body_entries
            .iter()
            .max_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .total_cmp(&right.epoch.julian_day.days())
            })
            .map(|candidate| candidate.epoch)
            .expect("reference high-curvature body window should not be empty after collection");

        sample_bodies.push(entry.body.clone());
        windows.push(ReferenceHighCurvatureWindow {
            body: entry.body.clone(),
            sample_count: body_entries.len(),
            epoch_count: body_entries
                .iter()
                .map(|candidate| candidate.epoch.julian_day.days().to_bits())
                .collect::<BTreeSet<_>>()
                .len(),
            earliest_epoch: body_earliest_epoch,
            latest_epoch: body_latest_epoch,
        });
    }

    Some(ReferenceHighCurvatureWindowSummary {
        sample_count: entries.len(),
        sample_bodies,
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the major-body high-curvature reference coverage.
pub fn reference_snapshot_high_curvature_window_summary(
) -> Option<ReferenceHighCurvatureWindowSummary> {
    reference_snapshot_high_curvature_window_summary_details()
}

/// Returns the release-facing major-body high-curvature window summary string.
pub fn reference_snapshot_high_curvature_window_summary_for_report() -> String {
    match reference_snapshot_high_curvature_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body high-curvature windows: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature windows: unavailable".to_string(),
    }
}

/// A single epoch slice inside the major-body high-curvature reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureEpochCoverage {
    /// The epoch covered by this slice.
    pub epoch: Instant,
    /// Number of bodies covered at the epoch.
    pub body_count: usize,
    /// Bodies covered by the epoch slice in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
}

impl ReferenceHighCurvatureEpochCoverage {
    /// Returns a compact epoch-coverage summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: {} bodies ({})",
            format_instant(self.epoch),
            self.body_count,
            format_bodies(&self.bodies),
        )
    }
}

impl fmt::Display for ReferenceHighCurvatureEpochCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the major-body high-curvature epoch coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureEpochCoverageSummary {
    /// Number of exact samples in the high-curvature slice.
    pub sample_count: usize,
    /// Number of distinct epochs covered by the high-curvature slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the high-curvature slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the high-curvature slice.
    pub latest_epoch: Instant,
    /// Per-epoch body breakdown in first-seen order.
    pub windows: Vec<ReferenceHighCurvatureEpochCoverage>,
}

impl ReferenceHighCurvatureEpochCoverageSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceHighCurvatureEpochCoverage::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference major-body high-curvature epoch coverage: {} exact samples across {} epochs ({}..{}); epochs: {}",
            self.sample_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the epoch coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceHighCurvatureEpochCoverageSummaryValidationError> {
        let Some(expected) = reference_snapshot_high_curvature_epoch_coverage_summary_details()
        else {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated epoch coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHighCurvatureEpochCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHighCurvatureEpochCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a high-curvature epoch summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHighCurvatureEpochCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in epoch coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceHighCurvatureEpochCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference high-curvature epoch coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceHighCurvatureEpochCoverageSummaryValidationError {}

fn reference_snapshot_high_curvature_epoch_coverage_summary_details(
) -> Option<ReferenceHighCurvatureEpochCoverageSummary> {
    let entries = reference_snapshot_high_curvature_entries()?;
    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");

    let mut windows = BTreeMap::<u64, ReferenceHighCurvatureEpochCoverage>::new();
    for entry in entries {
        let epoch_bits = entry.epoch.julian_day.days().to_bits();
        let window =
            windows
                .entry(epoch_bits)
                .or_insert_with(|| ReferenceHighCurvatureEpochCoverage {
                    epoch: entry.epoch,
                    body_count: 0,
                    bodies: Vec::new(),
                });
        if !window.bodies.contains(&entry.body) {
            window.bodies.push(entry.body.clone());
            window.body_count = window.bodies.len();
        }
    }

    Some(ReferenceHighCurvatureEpochCoverageSummary {
        sample_count: entries.len(),
        epoch_count: windows.len(),
        earliest_epoch,
        latest_epoch,
        windows: windows.into_values().collect(),
    })
}

/// Returns the compact typed summary for the major-body high-curvature epoch coverage.
pub fn reference_snapshot_high_curvature_epoch_coverage_summary(
) -> Option<ReferenceHighCurvatureEpochCoverageSummary> {
    reference_snapshot_high_curvature_epoch_coverage_summary_details()
}

/// Returns the release-facing major-body high-curvature epoch coverage summary string.
pub fn reference_snapshot_high_curvature_epoch_coverage_summary_for_report() -> String {
    match reference_snapshot_high_curvature_epoch_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body high-curvature epoch coverage: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature epoch coverage: unavailable".to_string(),
    }
}

const REFERENCE_SNAPSHOT_SOURCE_EXPECTED: &str =
    "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.";
const REFERENCE_SNAPSHOT_SOURCE_FALLBACK: &str = "NASA/JPL Horizons API vector tables (DE441)";
const REFERENCE_SNAPSHOT_COVERAGE_FALLBACK: &str =
    "major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01.";
const REFERENCE_SNAPSHOT_FRAME_TREATMENT: &str = "geocentric ecliptic J2000";
const INDEPENDENT_HOLDOUT_SOURCE_EXPECTED: &str =
    "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.";
const INDEPENDENT_HOLDOUT_SOURCE_FALLBACK: &str = "NASA/JPL Horizons API vector tables (DE441)";
const INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK: &str =
    "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000.";
const INDEPENDENT_HOLDOUT_COLUMNS: &str = "epoch_jd, body, x_km, y_km, z_km";

/// Backend-owned provenance summary for the checked-in reference snapshot source material.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSourceSummary {
    /// Source attribution for the checked-in reference snapshot.
    pub source: String,
    /// Body and epoch coverage described by the checked-in reference snapshot.
    pub coverage: String,
    /// Frame and coordinate posture described by the checked-in reference snapshot.
    pub frame_treatment: String,
    /// Reference epoch used by the checked-in snapshot.
    pub reference_epoch: Instant,
}

impl ReferenceSnapshotSourceSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotSourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankSource);
        }
        if has_surrounding_whitespace(&self.source) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                },
            );
        }
        if self.source != REFERENCE_SNAPSHOT_SOURCE_EXPECTED {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "source" },
            );
        }
        if self.coverage.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankCoverage);
        }
        if has_surrounding_whitespace(&self.coverage) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                },
            );
        }
        if self.coverage != REFERENCE_SNAPSHOT_COVERAGE_FALLBACK {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "coverage" },
            );
        }
        if self.frame_treatment.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankFrameTreatment);
        }
        if has_surrounding_whitespace(&self.frame_treatment) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "frame_treatment",
                },
            );
        }
        if self.frame_treatment != REFERENCE_SNAPSHOT_FRAME_TREATMENT {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "frame_treatment",
                },
            );
        }
        if self.reference_epoch != reference_instant() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::ReferenceEpochMismatch);
        }
        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot source: {}; coverage={}; {}; TDB reference epoch {}",
            self.source,
            self.coverage,
            self.frame_treatment,
            format_instant(self.reference_epoch),
        )
    }

    /// Returns a compact summary line after validating the reference snapshot source summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Structured validation errors for a reference snapshot provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSnapshotSourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty coverage label.
    BlankCoverage,
    /// The summary did not include a non-empty frame-treatment label.
    BlankFrameTreatment,
    /// The summary carried surrounding whitespace in one of its labels.
    SurroundedByWhitespace { field: &'static str },
    /// One of the canonical summary fields drifted from the checked-in slice.
    FieldOutOfSync { field: &'static str },
    /// The summary carried an unexpected reference epoch.
    ReferenceEpochMismatch,
}

impl ReferenceSnapshotSourceSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankSource => "blank source",
            Self::BlankCoverage => "blank coverage",
            Self::BlankFrameTreatment => "blank frame treatment",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
            Self::FieldOutOfSync { .. } => "field out of sync",
            Self::ReferenceEpochMismatch => "reference epoch mismatch",
        }
    }
}

impl fmt::Display for ReferenceSnapshotSourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::FieldOutOfSync { field } => write!(f, "{field} is out of sync"),
            Self::ReferenceEpochMismatch => f.write_str("reference epoch mismatch"),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ReferenceSnapshotSourceSummaryValidationError {}

impl fmt::Display for ReferenceSnapshotSourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the checked-in reference snapshot.
pub fn reference_snapshot_source_summary() -> ReferenceSnapshotSourceSummary {
    static SUMMARY: OnceLock<ReferenceSnapshotSourceSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = reference_snapshot_manifest();
            let source = manifest.source_or(REFERENCE_SNAPSHOT_SOURCE_FALLBACK);
            ReferenceSnapshotSourceSummary {
                source: source.to_string(),
                coverage: manifest
                    .coverage_or(REFERENCE_SNAPSHOT_COVERAGE_FALLBACK)
                    .to_string(),
                frame_treatment: "geocentric ecliptic J2000".to_string(),
                reference_epoch: reference_instant(),
            }
        })
        .clone()
}

/// Returns the source-material summary for the checked-in reference snapshot.
pub fn reference_snapshot_source_summary_for_report() -> String {
    if let Err(error) = reference_snapshot_manifest().validate() {
        return format!("Reference snapshot source: unavailable ({error})");
    }

    let summary = reference_snapshot_source_summary();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference snapshot source: unavailable ({error})"),
    }
}

/// A single body-window slice inside the checked-in reference snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSourceWindow {
    /// The snapshot body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ReferenceSnapshotSourceWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

impl fmt::Display for ReferenceSnapshotSourceWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the checked-in reference snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSourceWindowSummary {
    /// Number of reference-snapshot samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the checked-in source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ReferenceSnapshotSourceWindow>,
}

impl ReferenceSnapshotSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference snapshot source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the reference snapshot source windows still match the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotSourceWindowSummaryValidationError> {
        let Some(expected) = reference_snapshot_source_window_summary_details() else {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated reference snapshot source window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotSourceWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a reference snapshot source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSnapshotSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in reference snapshot source windows.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceSnapshotSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference snapshot source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotSourceWindowSummaryValidationError {}

fn reference_snapshot_source_window_summary_details() -> Option<ReferenceSnapshotSourceWindowSummary>
{
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut sample_bodies = Vec::new();
    for entry in entries {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    let mut windows = Vec::new();
    for body in &sample_bodies {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(ReferenceSnapshotSourceWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference snapshot source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference snapshot source windows should not be empty after collection");

    Some(ReferenceSnapshotSourceWindowSummary {
        sample_count: entries.len(),
        sample_bodies,
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the checked-in reference snapshot source coverage.
pub fn reference_snapshot_source_window_summary() -> Option<ReferenceSnapshotSourceWindowSummary> {
    reference_snapshot_source_window_summary_details()
}

/// Formats the checked-in reference snapshot source windows for release-facing reporting.
pub fn format_reference_snapshot_source_window_summary(
    summary: &ReferenceSnapshotSourceWindowSummary,
) -> String {
    summary.summary_line()
}

/// Returns the body-window summary for the checked-in reference snapshot.
pub fn reference_snapshot_source_window_summary_for_report() -> String {
    match reference_snapshot_source_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference snapshot source windows: unavailable ({error})"),
        },
        None => "Reference snapshot source windows: unavailable".to_string(),
    }
}

/// Returns the manifest summary for the checked-in reference snapshot.
pub fn reference_snapshot_manifest_summary() -> SnapshotManifestSummary {
    SnapshotManifestSummary {
        label: "Reference snapshot manifest",
        manifest: reference_snapshot_manifest().clone(),
        source_fallback: "unknown",
        coverage_fallback: "unknown",
    }
}

/// Returns the manifest summary for the checked-in reference snapshot.
pub fn reference_snapshot_manifest_summary_for_report() -> String {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    ));
    if let Err(error) = validate_snapshot_manifest_header_structure(
        manifest_text,
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        return format!("Reference snapshot manifest: unavailable ({error})");
    }

    let summary = reference_snapshot_manifest_summary();
    match summary.validate_with_expected_metadata(
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("Reference snapshot manifest: unavailable ({error})"),
    }
}

/// Backend-owned provenance summary for the checked-in hold-out snapshot source material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndependentHoldoutSourceSummary {
    /// Source attribution for the hold-out snapshot.
    pub source: String,
    /// Coverage note for the hold-out snapshot.
    pub coverage: String,
    /// CSV column layout for the hold-out snapshot.
    pub columns: String,
}

impl IndependentHoldoutSourceSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), IndependentHoldoutSourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankSource);
        }
        if has_surrounding_whitespace(&self.source) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                },
            );
        }
        if self.source != INDEPENDENT_HOLDOUT_SOURCE_EXPECTED {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "source" },
            );
        }
        if self.coverage.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankCoverage);
        }
        if has_surrounding_whitespace(&self.coverage) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                },
            );
        }
        if self.coverage != INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                    field: "coverage",
                },
            );
        }
        if self.columns.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankColumns);
        }
        if has_surrounding_whitespace(&self.columns) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "columns",
                },
            );
        }
        if self.columns != INDEPENDENT_HOLDOUT_COLUMNS {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "columns" },
            );
        }
        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Independent hold-out source: {}; coverage={}; columns={}",
            self.source, self.coverage, self.columns
        )
    }

    /// Returns a compact summary line after validating the hold-out snapshot source summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Structured validation errors for a hold-out snapshot provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndependentHoldoutSourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty coverage label.
    BlankCoverage,
    /// The summary did not include a non-empty columns label.
    BlankColumns,
    /// The summary carried surrounding whitespace in one of its labels.
    SurroundedByWhitespace { field: &'static str },
    /// One of the canonical summary fields drifted from the checked-in slice.
    FieldOutOfSync { field: &'static str },
}

impl IndependentHoldoutSourceSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankSource => "blank source",
            Self::BlankCoverage => "blank coverage",
            Self::BlankColumns => "blank columns",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
            Self::FieldOutOfSync { .. } => "field out of sync",
        }
    }
}

impl fmt::Display for IndependentHoldoutSourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::FieldOutOfSync { field } => write!(f, "{field} is out of sync"),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for IndependentHoldoutSourceSummaryValidationError {}

impl fmt::Display for IndependentHoldoutSourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the checked-in hold-out snapshot.
pub fn independent_holdout_source_summary() -> IndependentHoldoutSourceSummary {
    static SUMMARY: OnceLock<IndependentHoldoutSourceSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = independent_holdout_snapshot_manifest();
            IndependentHoldoutSourceSummary {
                source: manifest
                    .source_or(INDEPENDENT_HOLDOUT_SOURCE_FALLBACK)
                    .to_string(),
                coverage: manifest
                    .coverage_or(INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK)
                    .to_string(),
                columns: manifest.columns_summary(),
            }
        })
        .clone()
}

/// Returns the source-material summary for the checked-in hold-out snapshot.
pub fn independent_holdout_source_summary_for_report() -> String {
    if let Err(error) = independent_holdout_snapshot_manifest().validate() {
        return format!("Independent hold-out source: unavailable ({error})");
    }

    let summary = independent_holdout_source_summary();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Independent hold-out source: unavailable ({error})"),
    }
}

/// Returns the manifest summary for the checked-in hold-out snapshot.
pub fn independent_holdout_manifest_summary() -> SnapshotManifestSummary {
    SnapshotManifestSummary {
        label: "Independent hold-out manifest",
        manifest: independent_holdout_snapshot_manifest().clone(),
        source_fallback: "unknown",
        coverage_fallback: "unknown",
    }
}

/// Returns the manifest summary for the checked-in hold-out snapshot.
pub fn independent_holdout_manifest_summary_for_report() -> String {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/independent_holdout_snapshot.csv"
    ));
    if let Err(error) = validate_snapshot_manifest_header_structure(
        manifest_text,
        "Independent JPL Horizons hold-out snapshot used only for interpolation validation.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        return format!("Independent hold-out manifest: unavailable ({error})");
    }

    let summary = independent_holdout_manifest_summary();
    match summary.validate_with_expected_metadata(
        "Independent JPL Horizons hold-out snapshot used only for interpolation validation.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("Independent hold-out manifest: unavailable ({error})"),
    }
}

/// Returns the combined snapshot evidence summary used by validation and release reports.
pub fn jpl_snapshot_evidence_summary_for_report() -> String {
    [
        reference_snapshot_summary_for_report(),
        reference_snapshot_body_class_coverage_summary_for_report(),
        reference_snapshot_equatorial_parity_summary_for_report(),
        reference_snapshot_batch_parity_summary_for_report(),
        production_generation_snapshot_summary_for_report(),
        production_generation_source_summary_for_report(),
        reference_snapshot_source_summary_for_report(),
        reference_snapshot_source_window_summary_for_report(),
        reference_snapshot_major_body_boundary_summary_for_report(),
        reference_holdout_overlap_summary_for_report(),
        reference_snapshot_manifest_summary_for_report(),
        production_generation_boundary_source_summary_for_report(),
        production_generation_boundary_window_summary_for_report(),
        production_generation_boundary_body_class_coverage_summary_for_report(),
        production_generation_boundary_request_corpus_summary_for_report(),
        reference_asteroid_evidence_summary_for_report(),
        reference_asteroid_equatorial_evidence_summary_for_report(),
        reference_asteroid_source_window_summary_for_report(),
        selected_asteroid_terminal_boundary_summary_for_report(),
        comparison_snapshot_summary_for_report(),
        comparison_snapshot_body_class_coverage_summary_for_report(),
        comparison_snapshot_source_summary_for_report(),
        comparison_snapshot_source_window_summary_for_report(),
        comparison_snapshot_manifest_summary_for_report(),
        independent_holdout_snapshot_summary_for_report(),
        independent_holdout_snapshot_equatorial_parity_summary_for_report(),
        independent_holdout_snapshot_batch_parity_summary_for_report(),
        independent_holdout_source_summary_for_report(),
        independent_holdout_snapshot_source_window_summary_for_report(),
        independent_holdout_manifest_summary_for_report(),
        jpl_independent_holdout_summary_for_report(),
    ]
    .join(" | ")
}

/// Structured request policy for the current JPL snapshot backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JplSnapshotRequestPolicy {
    /// Coordinate frames the current snapshot backend exposes.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales accepted by the current snapshot backend.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes accepted by the current snapshot backend.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes accepted by the current snapshot backend.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the current snapshot backend accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
}

/// Validation error for a JPL request-policy summary that drifted from the current backend posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JplSnapshotRequestPolicyValidationError {
    /// One of the request-policy fields differs from the current backend posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplSnapshotRequestPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL snapshot request-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplSnapshotRequestPolicyValidationError {}

impl JplSnapshotRequestPolicy {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}",
            format_coordinate_frames(self.supported_frames),
            format_time_scales(self.supported_time_scales),
            format_zodiac_modes(self.supported_zodiac_modes),
            format_apparentness_modes(self.supported_apparentness),
            self.supports_topocentric_observer,
        )
    }

    /// Returns the compact summary line after validating the cached request policy.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplSnapshotRequestPolicyValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the summary against the current JPL snapshot backend posture.
    pub fn validate(&self) -> Result<(), JplSnapshotRequestPolicyValidationError> {
        if self.supported_frames != JPL_SNAPSHOT_REQUEST_POLICY.supported_frames {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_frames",
            });
        }
        if self.supported_time_scales != JPL_SNAPSHOT_REQUEST_POLICY.supported_time_scales {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_time_scales",
            });
        }
        if self.supported_zodiac_modes != JPL_SNAPSHOT_REQUEST_POLICY.supported_zodiac_modes {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_zodiac_modes",
            });
        }
        if self.supported_apparentness != JPL_SNAPSHOT_REQUEST_POLICY.supported_apparentness {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_apparentness",
            });
        }
        if self.supports_topocentric_observer
            != JPL_SNAPSHOT_REQUEST_POLICY.supports_topocentric_observer
        {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supports_topocentric_observer",
            });
        }
        Ok(())
    }
}

impl fmt::Display for JplSnapshotRequestPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const JPL_SNAPSHOT_REQUEST_POLICY: JplSnapshotRequestPolicy = JplSnapshotRequestPolicy {
    supported_frames: &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
    supported_time_scales: &[TimeScale::Tt, TimeScale::Tdb],
    supported_zodiac_modes: &[ZodiacMode::Tropical],
    supported_apparentness: &[Apparentness::Mean],
    supports_topocentric_observer: false,
};

/// Returns the current JPL snapshot request policy.
pub const fn jpl_snapshot_request_policy() -> JplSnapshotRequestPolicy {
    JPL_SNAPSHOT_REQUEST_POLICY
}

/// Returns the release-facing JPL snapshot request policy summary string.
pub fn jpl_snapshot_request_policy_summary_for_report() -> String {
    let policy = jpl_snapshot_request_policy();
    match policy.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("JPL snapshot request policy: unavailable ({error})"),
    }
}

/// A compact batch error-taxonomy summary for the current JPL snapshot backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplSnapshotBatchErrorTaxonomySummary {
    /// The body used for the supported batch check.
    pub supported_request_body: CelestialBody,
    /// The body used for the unsupported-body batch check.
    pub unsupported_request_body: CelestialBody,
    /// The error kind observed for the unsupported-body batch check.
    pub unsupported_error_kind: EphemerisErrorKind,
    /// The body used for the out-of-range batch check.
    pub out_of_range_request_body: CelestialBody,
    /// The error kind observed for the out-of-range batch check.
    pub out_of_range_error_kind: EphemerisErrorKind,
}

/// Structured errors for a JPL batch error-taxonomy summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JplSnapshotBatchErrorTaxonomySummaryValidationError {
    /// A summary field is out of sync with the current backend posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplSnapshotBatchErrorTaxonomySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL batch error-taxonomy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplSnapshotBatchErrorTaxonomySummaryValidationError {}

impl JplSnapshotBatchErrorTaxonomySummary {
    /// Returns the compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL batch error taxonomy: supported body {}; unsupported body {} -> {}; out-of-range {} -> {}",
            self.supported_request_body,
            self.unsupported_request_body,
            self.unsupported_error_kind,
            self.out_of_range_request_body,
            self.out_of_range_error_kind,
        )
    }

    /// Returns the compact summary line after validating the cached batch taxonomy.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplSnapshotBatchErrorTaxonomySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the summary against the current JPL snapshot backend posture.
    pub fn validate(&self) -> Result<(), JplSnapshotBatchErrorTaxonomySummaryValidationError> {
        if self.supported_request_body != CelestialBody::Ceres {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "supported_request_body",
                },
            );
        }
        if self.unsupported_request_body != CelestialBody::MeanNode {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "unsupported_request_body",
                },
            );
        }
        if self.unsupported_error_kind != EphemerisErrorKind::UnsupportedBody {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "unsupported_error_kind",
                },
            );
        }
        if self.out_of_range_request_body != CelestialBody::Ceres {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "out_of_range_request_body",
                },
            );
        }
        if self.out_of_range_error_kind != EphemerisErrorKind::OutOfRangeInstant {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "out_of_range_error_kind",
                },
            );
        }
        Ok(())
    }
}

impl fmt::Display for JplSnapshotBatchErrorTaxonomySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the control-sample batch corpus used by the current JPL batch
/// error taxonomy summary.
///
/// The requests preserve the supported-body, unsupported-body, and
/// out-of-range checks exercised by the release-facing taxonomy summary so
/// downstream tooling can reuse the exact batch shape without reconstructing it
/// inline.
pub fn jpl_snapshot_batch_error_taxonomy_requests() -> Vec<EphemerisRequest> {
    let supported_request = EphemerisRequest {
        body: CelestialBody::Ceres,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let unsupported_body_request = EphemerisRequest {
        body: CelestialBody::MeanNode,
        ..supported_request.clone()
    };
    let out_of_range_request = EphemerisRequest {
        body: CelestialBody::Ceres,
        instant: Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb),
        ..supported_request.clone()
    };

    vec![
        supported_request,
        unsupported_body_request,
        out_of_range_request,
    ]
}

/// This is a compatibility alias for [`jpl_snapshot_batch_error_taxonomy_requests`].
#[doc(alias = "jpl_snapshot_batch_error_taxonomy_requests")]
pub fn jpl_snapshot_batch_error_taxonomy_request_corpus() -> Vec<EphemerisRequest> {
    jpl_snapshot_batch_error_taxonomy_requests()
}

/// Returns a compact batch error-taxonomy summary for the current JPL snapshot backend.
pub fn jpl_snapshot_batch_error_taxonomy_summary(
) -> Result<JplSnapshotBatchErrorTaxonomySummary, JplSnapshotBatchErrorTaxonomySummaryValidationError>
{
    let backend = JplSnapshotBackend;

    let requests = jpl_snapshot_batch_error_taxonomy_requests();
    let supported_request = requests[0].clone();
    let unsupported_body_request = requests[1].clone();
    let out_of_range_request = requests[2].clone();

    let unsupported_body_error =
        match backend.positions(&[supported_request.clone(), unsupported_body_request]) {
            Ok(_) => {
                return Err(
                    JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                        field: "unsupported_body_batch",
                    },
                );
            }
            Err(error) => error,
        };
    if unsupported_body_error.kind != EphemerisErrorKind::UnsupportedBody {
        return Err(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "unsupported_body_error_kind",
            },
        );
    }

    let out_of_range_error = match backend.positions(&[out_of_range_request]) {
        Ok(_) => {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "out_of_range_batch",
                },
            );
        }
        Err(error) => error,
    };
    if out_of_range_error.kind != EphemerisErrorKind::OutOfRangeInstant {
        return Err(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "out_of_range_error_kind",
            },
        );
    }

    Ok(JplSnapshotBatchErrorTaxonomySummary {
        supported_request_body: CelestialBody::Ceres,
        unsupported_request_body: CelestialBody::MeanNode,
        unsupported_error_kind: EphemerisErrorKind::UnsupportedBody,
        out_of_range_request_body: CelestialBody::Ceres,
        out_of_range_error_kind: EphemerisErrorKind::OutOfRangeInstant,
    })
}

/// Returns the release-facing batch error-taxonomy summary for the current JPL snapshot backend.
pub fn jpl_snapshot_batch_error_taxonomy_summary_for_report() -> String {
    match jpl_snapshot_batch_error_taxonomy_summary() {
        Ok(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL batch error taxonomy: unavailable ({error})"),
        },
        Err(error) => format!("JPL batch error taxonomy: unavailable ({error})"),
    }
}

/// Returns the structured JPL snapshot frame-treatment summary.
pub const fn frame_treatment_summary_details() -> FrameTreatmentSummary {
    FrameTreatmentSummary::new(
        "checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform",
    )
}

/// Returns the current JPL snapshot frame-treatment summary.
pub fn frame_treatment_summary() -> &'static str {
    frame_treatment_summary_details().summary_line()
}

/// Returns the release-facing frame-treatment summary for the current JPL snapshot backend.
///
/// The backend-owned note is validated before the compact report line is
/// rendered, so a drifted summary becomes an unavailable report rather than a
/// stale cached string.
pub fn frame_treatment_summary_for_report() -> String {
    let summary = frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("JPL frame treatment unavailable ({error})"),
    }
}

/// Returns the comparison-only subset used by the stage-4 validation corpus.
pub fn comparison_snapshot() -> &'static [SnapshotEntry] {
    comparison_snapshot_entries()
}

/// Returns the comparison-snapshot request corpus in the requested frame.
///
/// The requests preserve the checked-in row order and retag the comparison rows
/// onto the TT request time scale currently used by the validation corpus, which
/// lets downstream tooling reuse the exact batch shape without reconstructing it
/// from the snapshot metadata in each caller.
pub fn comparison_snapshot_requests(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    Some(
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(entry.epoch.julian_day, TimeScale::Tt),
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect(),
    )
}

/// This is a compatibility alias for [`comparison_snapshot_requests`].
#[doc(alias = "comparison_snapshot_requests")]
pub fn comparison_snapshot_request_corpus(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_requests(frame)
}

/// Returns the ecliptic comparison-snapshot request corpus used by validation tooling.
///
/// This is a compatibility alias for [`comparison_snapshot_request_corpus`].
#[doc(alias = "comparison_snapshot_requests")]
pub fn comparison_snapshot_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_request_corpus(CoordinateFrame::Ecliptic)
}

/// Returns the ecliptic comparison-snapshot request corpus used by validation tooling.
///
/// This is a compatibility alias for [`comparison_snapshot_ecliptic_request_corpus`].
#[doc(alias = "comparison_snapshot_ecliptic_request_corpus")]
pub fn comparison_snapshot_ecliptic_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_ecliptic_request_corpus()
}

/// Returns the equatorial comparison-snapshot request corpus used by parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_requests`].
#[doc(alias = "comparison_snapshot_requests")]
pub fn comparison_snapshot_equatorial_parity_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_requests(CoordinateFrame::Equatorial)
}

/// Returns the equatorial comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_parity_requests")]
pub fn comparison_snapshot_equatorial_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_batch_parity_requests")]
pub fn comparison_snapshot_equatorial_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>>
{
    comparison_snapshot_equatorial_batch_parity_requests()
}

/// Returns the equatorial comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_request_corpus`].
#[doc(alias = "comparison_snapshot_equatorial_request_corpus")]
pub fn comparison_snapshot_equatorial_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_request_corpus()
}

/// Returns the equatorial comparison-snapshot request corpus used by parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_parity_requests")]
pub fn comparison_snapshot_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial comparison-snapshot request corpus used by parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_parity_requests")]
pub fn comparison_snapshot_equatorial_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_parity_requests()
}

/// Returns the mixed-frame comparison-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order and alternate between ecliptic
/// and equatorial frames so downstream tooling can reuse the exact validation
/// batch shape without reconstructing it from snapshot metadata.
pub fn comparison_snapshot_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    Some(
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(entry.epoch.julian_day, TimeScale::Tt),
                observer: None,
                frame: if index % 2 == 0 {
                    CoordinateFrame::Ecliptic
                } else {
                    CoordinateFrame::Equatorial
                },
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect(),
    )
}

/// Returns the mixed-frame comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_batch_parity_requests")]
pub fn comparison_snapshot_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_batch_parity_requests()
}

/// Returns the mixed-scale comparison-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order, keep the ecliptic frame, and
/// alternate TT/TDB labels so downstream tooling can reuse the exact validation
/// batch shape without reconstructing it from snapshot metadata.
pub fn comparison_snapshot_mixed_time_scale_batch_parity_requests() -> Option<Vec<EphemerisRequest>>
{
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    Some(
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(
                    entry.epoch.julian_day,
                    if index % 2 == 0 {
                        TimeScale::Tt
                    } else {
                        TimeScale::Tdb
                    },
                ),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect(),
    )
}

/// Returns the mixed-scale comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn comparison_snapshot_mixed_time_scale_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn comparison_snapshot_mixed_tt_tdb_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn comparison_snapshot_mixed_tt_tdb_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_time_scale_request_corpus`].
#[doc(alias = "comparison_snapshot_mixed_time_scale_request_corpus")]
pub fn comparison_snapshot_mixed_tt_tdb_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn comparison_snapshot_mixed_time_scale_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the parsed manifest for the comparison snapshot.
pub fn comparison_snapshot_manifest() -> &'static SnapshotManifest {
    static MANIFEST: OnceLock<SnapshotManifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        parse_snapshot_manifest(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/j2000_snapshot.csv"
        )))
    })
}

/// Returns the comparison-only body coverage used by validation tooling.
pub fn comparison_bodies() -> &'static [pleiades_backend::CelestialBody] {
    comparison_body_list()
}

/// Returns coarse leave-one-out interpolation checks derived from the checked-in
/// fixture.
///
/// Each sample removes a middle exact fixture epoch from the body-specific
/// snapshot rows, re-runs the backend's current interpolation path, and compares
/// the interpolated result with the held-out exact sample. The current fixture is
/// intentionally sparse, so these values are evidence for report transparency
/// rather than production interpolation tolerances.
pub fn interpolation_quality_samples() -> &'static [InterpolationQualitySample] {
    interpolation_quality_sample_list()
}

/// Returns the exact ecliptic request corpus used to derive the interpolation-quality samples.
///
/// The requests preserve the checked-in sample order and stored epochs from the
/// derivative fixture, so downstream validation and reproducibility tooling can
/// reuse the exact held-out batch slice without reconstructing it from the sample
/// metadata.
pub fn interpolation_quality_sample_requests() -> Option<Vec<EphemerisRequest>> {
    snapshot_entries().map(|_| {
        interpolation_quality_samples()
            .iter()
            .map(|sample| EphemerisRequest {
                body: sample.body.clone(),
                instant: sample.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the exact ecliptic request corpus used to derive the interpolation-quality samples.
///
/// This is a compatibility alias for [`interpolation_quality_sample_requests`].
#[doc(alias = "interpolation_quality_sample_requests")]
pub fn interpolation_quality_sample_request_corpus() -> Option<Vec<EphemerisRequest>> {
    interpolation_quality_sample_requests()
}

/// A compact interpolation-quality summary for the checked-in JPL snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct JplInterpolationQualitySummary {
    /// Total number of interpolation-quality samples.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the samples.
    pub body_count: usize,
    /// Number of distinct epochs represented by the samples.
    pub epoch_count: usize,
    /// Earliest epoch represented by the samples.
    pub earliest_epoch: Instant,
    /// Latest epoch represented by the samples.
    pub latest_epoch: Instant,
    /// Number of samples that used cubic interpolation.
    pub cubic_sample_count: usize,
    /// Number of samples that used quadratic interpolation.
    pub quadratic_sample_count: usize,
    /// Number of samples that used linear fallback interpolation.
    pub linear_sample_count: usize,
    /// Largest bracketing span among the samples.
    pub max_bracket_span_days: f64,
    /// Body associated with the largest bracketing span.
    pub max_bracket_span_body: String,
    /// Held-out epoch associated with the largest bracketing span.
    pub max_bracket_span_epoch: Instant,
    /// Mean bracketing span across the samples.
    pub mean_bracket_span_days: f64,
    /// Median bracketing span across the samples.
    pub median_bracket_span_days: f64,
    /// 95th percentile bracketing span across the samples.
    pub percentile_bracket_span_days: f64,
    /// Largest longitude error among the samples.
    pub max_longitude_error_deg: f64,
    /// Body associated with the largest longitude error.
    pub max_longitude_error_body: String,
    /// Held-out epoch associated with the largest longitude error.
    pub max_longitude_error_epoch: Instant,
    /// Mean longitude error across the samples.
    pub mean_longitude_error_deg: f64,
    /// Median longitude error across the samples.
    pub median_longitude_error_deg: f64,
    /// 95th percentile longitude error across the samples.
    pub percentile_longitude_error_deg: f64,
    /// Root-mean-square longitude error across the samples.
    pub rms_longitude_error_deg: f64,
    /// Largest latitude error among the samples.
    pub max_latitude_error_deg: f64,
    /// Body associated with the largest latitude error.
    pub max_latitude_error_body: String,
    /// Held-out epoch associated with the largest latitude error.
    pub max_latitude_error_epoch: Instant,
    /// Mean latitude error across the samples.
    pub mean_latitude_error_deg: f64,
    /// Median latitude error across the samples.
    pub median_latitude_error_deg: f64,
    /// 95th percentile latitude error across the samples.
    pub percentile_latitude_error_deg: f64,
    /// Root-mean-square latitude error across the samples.
    pub rms_latitude_error_deg: f64,
    /// Largest distance error among the samples.
    pub max_distance_error_au: f64,
    /// Body associated with the largest distance error.
    pub max_distance_error_body: String,
    /// Held-out epoch associated with the largest distance error.
    pub max_distance_error_epoch: Instant,
    /// Mean distance error across the samples.
    pub mean_distance_error_au: f64,
    /// Median distance error across the samples.
    pub median_distance_error_au: f64,
    /// 95th percentile distance error across the samples.
    pub percentile_distance_error_au: f64,
    /// Root-mean-square distance error across the samples.
    pub rms_distance_error_au: f64,
}

impl JplInterpolationQualitySummary {
    /// Returns the compact release-facing interpolation-quality summary line.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch_suffix(body: &str, epoch: Instant) -> String {
            if body.is_empty() {
                String::new()
            } else {
                format!(" ({body} @ {})", format_instant(epoch))
            }
        }

        format!(
            "JPL interpolation quality: {} samples across {} bodies and {} epochs ({} cubic, {} quadratic, {} linear), epoch window {} → {}; leave-one-out runtime interpolation evidence with worst-case bodies named, max bracket span={:.1} d{}; mean bracket span={:.1} d; median bracket span={:.1} d; p95 bracket span={:.1} d; max Δlon={:.12}°{}; mean Δlon={:.12}°; median Δlon={:.12}°; p95 Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}°{}; mean Δlat={:.12}°; median Δlat={:.12}°; p95 Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU{}; mean Δdist={:.12} AU; median Δdist={:.12} AU; p95 Δdist={:.12} AU; rms Δdist={:.12} AU; transparency evidence only, not a production tolerance envelope",
            self.sample_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            self.cubic_sample_count,
            self.quadratic_sample_count,
            self.linear_sample_count,
            self.max_bracket_span_days,
            format_body_epoch_suffix(&self.max_bracket_span_body, self.max_bracket_span_epoch),
            self.mean_bracket_span_days,
            self.median_bracket_span_days,
            self.percentile_bracket_span_days,
            self.max_longitude_error_deg,
            format_body_epoch_suffix(&self.max_longitude_error_body, self.max_longitude_error_epoch),
            self.mean_longitude_error_deg,
            self.median_longitude_error_deg,
            self.percentile_longitude_error_deg,
            self.rms_longitude_error_deg,
            self.max_latitude_error_deg,
            format_body_epoch_suffix(&self.max_latitude_error_body, self.max_latitude_error_epoch),
            self.mean_latitude_error_deg,
            self.median_latitude_error_deg,
            self.percentile_latitude_error_deg,
            self.rms_latitude_error_deg,
            self.max_distance_error_au,
            format_body_epoch_suffix(&self.max_distance_error_body, self.max_distance_error_epoch),
            self.mean_distance_error_au,
            self.median_distance_error_au,
            self.percentile_distance_error_au,
            self.rms_distance_error_au,
        )
    }

    /// Returns the validated compact interpolation-quality summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplInterpolationQualitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const JPL_INTERPOLATION_POSTURE_SOURCE: &str =
    "leave-one-out runtime interpolation evidence derived from the checked-in reference snapshot";
const JPL_INTERPOLATION_POSTURE_DETAIL: &str = "transparency evidence only";
const JPL_INTERPOLATION_POSTURE_ENVELOPE: &str = "not a production tolerance envelope";

/// A compact posture summary for the checked-in interpolation-quality evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplInterpolationPostureSummary {
    /// Source attribution for the interpolation-quality evidence posture.
    pub source: String,
    /// Release-facing posture label for the interpolation-quality evidence.
    pub detail: String,
    /// Explicit claim boundary for the interpolation-quality evidence.
    pub envelope: String,
}

impl JplInterpolationPostureSummary {
    /// Returns the compact release-facing interpolation posture summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL interpolation posture: source={}; detail={}; envelope={}",
            self.source, self.detail, self.envelope
        )
    }

    /// Validates that the posture summary still matches the checked-in evidence posture.
    pub fn validate(&self) -> Result<(), JplInterpolationPostureSummaryValidationError> {
        if self.source != JPL_INTERPOLATION_POSTURE_SOURCE {
            return Err(
                JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "source" },
            );
        }
        if self.detail != JPL_INTERPOLATION_POSTURE_DETAIL {
            return Err(
                JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "detail" },
            );
        }
        if self.envelope != JPL_INTERPOLATION_POSTURE_ENVELOPE {
            return Err(
                JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "envelope" },
            );
        }
        Ok(())
    }

    /// Returns the validated release-facing interpolation posture summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationPostureSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplInterpolationPostureSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured validation errors for the interpolation-quality summary.
#[derive(Clone, Debug, PartialEq)]
pub enum JplInterpolationQualitySummaryValidationError {
    /// The summary did not expose any samples.
    MissingSamples,
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary body list contained a duplicate body label.
    DuplicateBody { body: String },
    /// The summary body list contained a blank entry.
    BlankBody { index: usize },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// A summary metric was not finite and non-negative.
    MetricOutOfRange { field: &'static str },
    /// A peak-body label was blank despite the corresponding metric being populated.
    BlankPeakBody { field: &'static str },
    /// The interpolation-kind counts did not add up to the total sample count.
    InterpolationKindCountMismatch {
        sample_count: usize,
        kind_count: usize,
    },
    /// The summary no longer matches the derived interpolation evidence.
    DerivedSummaryMismatch,
}

impl JplInterpolationQualitySummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingSamples => "missing samples",
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BlankBody { .. } => "blank body",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::MetricOutOfRange { .. } => "metric out of range",
            Self::BlankPeakBody { .. } => "blank peak body",
            Self::InterpolationKindCountMismatch { .. } => "interpolation-kind count mismatch",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for JplInterpolationQualitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSamples | Self::MissingBodies | Self::MissingEpochs => {
                f.write_str(self.label())
            }
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(
                f,
                "body count {body_count} does not match body list length {bodies_len}"
            ),
            Self::DuplicateBody { body } => {
                write!(f, "body list contains duplicate body label `{body}`")
            }
            Self::BlankBody { index } => {
                write!(f, "body list entry {index} is blank")
            }
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range: earliest {} is after latest {}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            Self::MetricOutOfRange { field } => write!(
                f,
                "summary metric `{field}` is not a finite non-negative value"
            ),
            Self::BlankPeakBody { field } => {
                write!(f, "summary peak body label `{field}` is blank")
            }
            Self::InterpolationKindCountMismatch {
                sample_count,
                kind_count,
            } => write!(
                f,
                "interpolation-kind count {kind_count} does not match sample count {sample_count}"
            ),
            Self::DerivedSummaryMismatch => {
                f.write_str("summary no longer matches the derived interpolation evidence")
            }
        }
    }
}

impl std::error::Error for JplInterpolationQualitySummaryValidationError {}

/// Structured validation errors for the interpolation posture summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplInterpolationPostureSummaryValidationError {
    /// A summary field is out of sync with the checked-in evidence posture.
    FieldOutOfSync { field: &'static str },
}

impl JplInterpolationPostureSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::FieldOutOfSync { .. } => "field out of sync",
        }
    }
}

impl fmt::Display for JplInterpolationPostureSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL interpolation posture summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for JplInterpolationPostureSummaryValidationError {}

/// Returns the release-facing interpolation posture summary for the checked-in evidence slice.
pub fn jpl_interpolation_posture_summary() -> Option<JplInterpolationPostureSummary> {
    Some(JplInterpolationPostureSummary {
        source: JPL_INTERPOLATION_POSTURE_SOURCE.to_string(),
        detail: JPL_INTERPOLATION_POSTURE_DETAIL.to_string(),
        envelope: JPL_INTERPOLATION_POSTURE_ENVELOPE.to_string(),
    })
}

/// Formats the interpolation posture summary for release-facing reports.
pub fn format_jpl_interpolation_posture_summary(
    summary: &JplInterpolationPostureSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing interpolation posture summary string.
pub fn jpl_interpolation_posture_summary_for_report() -> String {
    match jpl_interpolation_posture_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL interpolation posture: unavailable ({error})"),
        },
        None => "JPL interpolation posture: unavailable".to_string(),
    }
}

fn validate_non_negative_metric(
    field: &'static str,
    value: f64,
) -> Result<(), JplInterpolationQualitySummaryValidationError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(JplInterpolationQualitySummaryValidationError::MetricOutOfRange { field })
    }
}

impl JplInterpolationQualitySummary {
    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySummaryValidationError> {
        if self.sample_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingSamples);
        }
        if self.body_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingBodies);
        }
        if self.epoch_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                JplInterpolationQualitySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }
        for (field, value) in [
            ("max_bracket_span_days", self.max_bracket_span_days),
            ("mean_bracket_span_days", self.mean_bracket_span_days),
            ("median_bracket_span_days", self.median_bracket_span_days),
            (
                "percentile_bracket_span_days",
                self.percentile_bracket_span_days,
            ),
            ("max_longitude_error_deg", self.max_longitude_error_deg),
            ("mean_longitude_error_deg", self.mean_longitude_error_deg),
            (
                "median_longitude_error_deg",
                self.median_longitude_error_deg,
            ),
            (
                "percentile_longitude_error_deg",
                self.percentile_longitude_error_deg,
            ),
            ("rms_longitude_error_deg", self.rms_longitude_error_deg),
            ("max_latitude_error_deg", self.max_latitude_error_deg),
            ("mean_latitude_error_deg", self.mean_latitude_error_deg),
            ("median_latitude_error_deg", self.median_latitude_error_deg),
            (
                "percentile_latitude_error_deg",
                self.percentile_latitude_error_deg,
            ),
            ("rms_latitude_error_deg", self.rms_latitude_error_deg),
            ("max_distance_error_au", self.max_distance_error_au),
            ("mean_distance_error_au", self.mean_distance_error_au),
            ("median_distance_error_au", self.median_distance_error_au),
            (
                "percentile_distance_error_au",
                self.percentile_distance_error_au,
            ),
            ("rms_distance_error_au", self.rms_distance_error_au),
        ] {
            validate_non_negative_metric(field, value)?;
        }
        if self.max_bracket_span_days > 0.0 && self.max_bracket_span_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_bracket_span_body",
                },
            );
        }
        if self.max_longitude_error_deg > 0.0 && self.max_longitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_longitude_error_body",
                },
            );
        }
        if self.max_latitude_error_deg > 0.0 && self.max_latitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_latitude_error_body",
                },
            );
        }
        if self.max_distance_error_au > 0.0 && self.max_distance_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_distance_error_body",
                },
            );
        }

        if self.sample_count
            != self.cubic_sample_count + self.quadratic_sample_count + self.linear_sample_count
        {
            return Err(
                JplInterpolationQualitySummaryValidationError::InterpolationKindCountMismatch {
                    sample_count: self.sample_count,
                    kind_count: self.cubic_sample_count
                        + self.quadratic_sample_count
                        + self.linear_sample_count,
                },
            );
        }
        if jpl_interpolation_quality_summary().as_ref() != Some(self) {
            return Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }
}

/// Distinct-body coverage for the interpolation-quality hold-out samples.
#[derive(Clone, Debug, PartialEq)]
pub struct JplInterpolationQualityKindCoverage {
    /// Total number of interpolation-quality samples.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the samples.
    pub body_count: usize,
    /// Bodies represented by the samples in first-seen order.
    pub bodies: Vec<String>,
    /// Number of distinct bodies represented by cubic interpolation samples.
    pub cubic_body_count: usize,
    /// Number of distinct bodies represented by quadratic interpolation samples.
    pub quadratic_body_count: usize,
    /// Number of distinct bodies represented by linear interpolation samples.
    pub linear_body_count: usize,
}

/// Returns the release-facing interpolation-quality summary for the checked-in
/// JPL snapshot.
pub fn jpl_interpolation_quality_summary() -> Option<JplInterpolationQualitySummary> {
    let samples = interpolation_quality_samples();
    if samples.is_empty() {
        return None;
    }

    let mut bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut cubic_sample_count = 0usize;
    let mut quadratic_sample_count = 0usize;
    let mut linear_sample_count = 0usize;
    let mut max_bracket_span_days: f64 = 0.0;
    let mut max_bracket_span_body = String::new();
    let mut max_bracket_span_epoch = samples[0].epoch;
    let mut total_bracket_span_days = 0.0;
    let mut bracket_spans = Vec::new();
    let mut max_longitude_error_deg: f64 = 0.0;
    let mut max_longitude_error_body = String::new();
    let mut max_longitude_error_epoch = samples[0].epoch;
    let mut total_longitude_error_deg = 0.0;
    let mut total_longitude_error_sq_deg = 0.0;
    let mut longitude_errors = Vec::new();
    let mut max_latitude_error_deg: f64 = 0.0;
    let mut max_latitude_error_body = String::new();
    let mut max_latitude_error_epoch = samples[0].epoch;
    let mut total_latitude_error_deg = 0.0;
    let mut total_latitude_error_sq_deg = 0.0;
    let mut latitude_errors = Vec::new();
    let mut max_distance_error_au: f64 = 0.0;
    let mut max_distance_error_body = String::new();
    let mut max_distance_error_epoch = samples[0].epoch;
    let mut total_distance_error_au = 0.0;
    let mut total_distance_error_sq_au = 0.0;
    let mut distance_errors = Vec::new();

    for sample in samples {
        bodies.insert(sample.body.to_string());
        epochs.insert(sample.epoch.julian_day.days().to_bits());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
        match sample.interpolation_kind {
            InterpolationQualityKind::Cubic => cubic_sample_count += 1,
            InterpolationQualityKind::Quadratic => quadratic_sample_count += 1,
            InterpolationQualityKind::Linear => linear_sample_count += 1,
        }
        total_bracket_span_days += sample.bracket_span_days;
        bracket_spans.push(sample.bracket_span_days);
        total_longitude_error_deg += sample.longitude_error_deg;
        total_longitude_error_sq_deg += sample.longitude_error_deg * sample.longitude_error_deg;
        longitude_errors.push(sample.longitude_error_deg);
        total_latitude_error_deg += sample.latitude_error_deg;
        total_latitude_error_sq_deg += sample.latitude_error_deg * sample.latitude_error_deg;
        latitude_errors.push(sample.latitude_error_deg);
        total_distance_error_au += sample.distance_error_au;
        total_distance_error_sq_au += sample.distance_error_au * sample.distance_error_au;
        distance_errors.push(sample.distance_error_au);
        if sample.bracket_span_days > max_bracket_span_days {
            max_bracket_span_days = sample.bracket_span_days;
            max_bracket_span_body = sample.body.to_string();
            max_bracket_span_epoch = sample.epoch;
        }
        if sample.longitude_error_deg > max_longitude_error_deg {
            max_longitude_error_deg = sample.longitude_error_deg;
            max_longitude_error_body = sample.body.to_string();
            max_longitude_error_epoch = sample.epoch;
        }
        if sample.latitude_error_deg > max_latitude_error_deg {
            max_latitude_error_deg = sample.latitude_error_deg;
            max_latitude_error_body = sample.body.to_string();
            max_latitude_error_epoch = sample.epoch;
        }
        if sample.distance_error_au > max_distance_error_au {
            max_distance_error_au = sample.distance_error_au;
            max_distance_error_body = sample.body.to_string();
            max_distance_error_epoch = sample.epoch;
        }
    }

    let sample_count = samples.len() as f64;

    Some(JplInterpolationQualitySummary {
        median_bracket_span_days: median_f64(&mut bracket_spans),
        percentile_bracket_span_days: percentile_f64(&mut bracket_spans, 0.95),
        sample_count: samples.len(),
        body_count: bodies.len(),
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
        cubic_sample_count,
        quadratic_sample_count,
        linear_sample_count,
        max_bracket_span_days,
        max_bracket_span_body,
        max_bracket_span_epoch,
        mean_bracket_span_days: total_bracket_span_days / sample_count,
        max_longitude_error_deg,
        max_longitude_error_body,
        max_longitude_error_epoch,
        mean_longitude_error_deg: total_longitude_error_deg / sample_count,
        median_longitude_error_deg: median_f64(&mut longitude_errors),
        percentile_longitude_error_deg: percentile_f64(&mut longitude_errors, 0.95),
        rms_longitude_error_deg: (total_longitude_error_sq_deg / sample_count).sqrt(),
        max_latitude_error_deg,
        max_latitude_error_body,
        max_latitude_error_epoch,
        mean_latitude_error_deg: total_latitude_error_deg / sample_count,
        median_latitude_error_deg: median_f64(&mut latitude_errors),
        percentile_latitude_error_deg: percentile_f64(&mut latitude_errors, 0.95),
        rms_latitude_error_deg: (total_latitude_error_sq_deg / sample_count).sqrt(),
        max_distance_error_au,
        max_distance_error_body,
        max_distance_error_epoch,
        mean_distance_error_au: total_distance_error_au / sample_count,
        median_distance_error_au: median_f64(&mut distance_errors),
        percentile_distance_error_au: percentile_f64(&mut distance_errors, 0.95),
        rms_distance_error_au: (total_distance_error_sq_au / sample_count).sqrt(),
    })
}

/// Returns the distinct-body coverage breakdown for the interpolation-quality
/// hold-out samples.
pub fn jpl_interpolation_quality_kind_coverage() -> Option<JplInterpolationQualityKindCoverage> {
    let samples = interpolation_quality_samples();
    if samples.is_empty() {
        return None;
    }

    let mut all_bodies = BTreeSet::new();
    let mut first_seen_bodies = Vec::new();
    let mut cubic_bodies = BTreeSet::new();
    let mut quadratic_bodies = BTreeSet::new();
    let mut linear_bodies = BTreeSet::new();

    for sample in samples {
        let body = sample.body.to_string();
        if all_bodies.insert(body.clone()) {
            first_seen_bodies.push(body.clone());
        }
        match sample.interpolation_kind {
            InterpolationQualityKind::Cubic => {
                cubic_bodies.insert(body);
            }
            InterpolationQualityKind::Quadratic => {
                quadratic_bodies.insert(body);
            }
            InterpolationQualityKind::Linear => {
                linear_bodies.insert(body);
            }
        }
    }

    Some(JplInterpolationQualityKindCoverage {
        sample_count: samples.len(),
        body_count: all_bodies.len(),
        bodies: first_seen_bodies,
        cubic_body_count: cubic_bodies.len(),
        quadratic_body_count: quadratic_bodies.len(),
        linear_body_count: linear_bodies.len(),
    })
}

impl JplInterpolationQualityKindCoverage {
    /// Returns the compact release-facing coverage summary line.
    pub fn summary_line(&self) -> String {
        let bodies = if self.bodies.is_empty() {
            "none".to_string()
        } else {
            self.bodies.join(", ")
        };

        format!(
            "JPL interpolation quality kind coverage: {} samples across {} bodies [{}] ({} cubic bodies, {} quadratic bodies, {} linear bodies)",
            self.sample_count,
            self.body_count,
            bodies,
            self.cubic_body_count,
            self.quadratic_body_count,
            self.linear_body_count,
        )
    }

    /// Returns a compact summary line after validating the coverage summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplInterpolationQualityKindCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl JplInterpolationQualityKindCoverage {
    /// Validates that the coverage summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySummaryValidationError> {
        if self.sample_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingSamples);
        }
        if self.body_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        let mut seen_bodies = BTreeSet::new();
        for (index, body) in self.bodies.iter().enumerate() {
            if body.trim().is_empty() {
                return Err(JplInterpolationQualitySummaryValidationError::BlankBody { index });
            }
            if !seen_bodies.insert(body) {
                return Err(
                    JplInterpolationQualitySummaryValidationError::DuplicateBody {
                        body: body.clone(),
                    },
                );
            }
        }

        if jpl_interpolation_quality_kind_coverage().as_ref() != Some(self) {
            return Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }
}

/// Returns the release-facing interpolation-quality summary for the checked-in
#[derive(Clone, Debug, PartialEq)]
pub struct JplIndependentHoldoutSummary {
    /// Total number of hold-out samples.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the samples.
    pub body_count: usize,
    /// Bodies represented by the samples in first-seen order.
    pub bodies: Vec<String>,
    /// Number of distinct epochs represented by the samples.
    pub epoch_count: usize,
    /// Earliest epoch represented by the samples.
    pub earliest_epoch: Instant,
    /// Latest epoch represented by the samples.
    pub latest_epoch: Instant,
    /// Largest longitude error among the samples.
    pub max_longitude_error_deg: f64,
    /// Body associated with the largest longitude error.
    pub max_longitude_error_body: String,
    /// Held-out epoch associated with the largest longitude error.
    pub max_longitude_error_epoch: Instant,
    /// Mean longitude error across the samples.
    pub mean_longitude_error_deg: f64,
    /// Median longitude error across the samples.
    pub median_longitude_error_deg: f64,
    /// 95th percentile longitude error across the samples.
    pub percentile_longitude_error_deg: f64,
    /// Root-mean-square longitude error across the samples.
    pub rms_longitude_error_deg: f64,
    /// Largest latitude error among the samples.
    pub max_latitude_error_deg: f64,
    /// Body associated with the largest latitude error.
    pub max_latitude_error_body: String,
    /// Held-out epoch associated with the largest latitude error.
    pub max_latitude_error_epoch: Instant,
    /// Mean latitude error across the samples.
    pub mean_latitude_error_deg: f64,
    /// Median latitude error across the samples.
    pub median_latitude_error_deg: f64,
    /// 95th percentile latitude error across the samples.
    pub percentile_latitude_error_deg: f64,
    /// Root-mean-square latitude error across the samples.
    pub rms_latitude_error_deg: f64,
    /// Largest distance error among the samples.
    pub max_distance_error_au: f64,
    /// Body associated with the largest distance error.
    pub max_distance_error_body: String,
    /// Held-out epoch associated with the largest distance error.
    pub max_distance_error_epoch: Instant,
    /// Mean distance error across the samples.
    pub mean_distance_error_au: f64,
    /// Median distance error across the samples.
    pub median_distance_error_au: f64,
    /// 95th percentile distance error across the samples.
    pub percentile_distance_error_au: f64,
    /// Root-mean-square distance error across the samples.
    pub rms_distance_error_au: f64,
}

/// Returns a compact validation summary for the independent hold-out rows.
pub fn jpl_independent_holdout_summary() -> Option<JplIndependentHoldoutSummary> {
    let entries = independent_holdout_snapshot_entries()?;

    let mut bodies = Vec::new();
    let mut seen_bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;
    let mut max_longitude_error_deg: f64 = 0.0;
    let mut max_longitude_error_body = String::new();
    let mut max_longitude_error_epoch = entries[0].epoch;
    let mut total_longitude_error_deg = 0.0;
    let mut total_longitude_error_sq_deg = 0.0;
    let mut longitude_errors = Vec::new();
    let mut max_latitude_error_deg: f64 = 0.0;
    let mut max_latitude_error_body = String::new();
    let mut max_latitude_error_epoch = entries[0].epoch;
    let mut total_latitude_error_deg = 0.0;
    let mut total_latitude_error_sq_deg = 0.0;
    let mut latitude_errors = Vec::new();
    let mut max_distance_error_au: f64 = 0.0;
    let mut max_distance_error_body = String::new();
    let mut max_distance_error_epoch = entries[0].epoch;
    let mut total_distance_error_au = 0.0;
    let mut total_distance_error_sq_au = 0.0;
    let mut distance_errors = Vec::new();

    for entry in entries {
        let body = entry.body.to_string();
        if seen_bodies.insert(body.clone()) {
            bodies.push(body);
        }
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }

        let interpolated = resolve_fixture_state(entry.body.clone(), entry.epoch.julian_day.days())
            .expect("independent hold-out rows should interpolate against the main snapshot")
            .entry;
        let exact_ecliptic = entry.ecliptic();
        let interpolated_ecliptic = interpolated.ecliptic();
        let exact_distance = exact_ecliptic.distance_au.unwrap_or_default();
        let interpolated_distance = interpolated_ecliptic.distance_au.unwrap_or_default();

        let longitude_error = angular_degrees_delta(
            exact_ecliptic.longitude.degrees(),
            interpolated_ecliptic.longitude.degrees(),
        );
        let latitude_error =
            (exact_ecliptic.latitude.degrees() - interpolated_ecliptic.latitude.degrees()).abs();
        let distance_error = (exact_distance - interpolated_distance).abs();

        total_longitude_error_deg += longitude_error;
        total_longitude_error_sq_deg += longitude_error * longitude_error;
        longitude_errors.push(longitude_error);
        total_latitude_error_deg += latitude_error;
        total_latitude_error_sq_deg += latitude_error * latitude_error;
        latitude_errors.push(latitude_error);
        total_distance_error_au += distance_error;
        total_distance_error_sq_au += distance_error * distance_error;
        distance_errors.push(distance_error);

        if longitude_error > max_longitude_error_deg {
            max_longitude_error_deg = longitude_error;
            max_longitude_error_body = entry.body.to_string();
            max_longitude_error_epoch = entry.epoch;
        }
        if latitude_error > max_latitude_error_deg {
            max_latitude_error_deg = latitude_error;
            max_latitude_error_body = entry.body.to_string();
            max_latitude_error_epoch = entry.epoch;
        }
        if distance_error > max_distance_error_au {
            max_distance_error_au = distance_error;
            max_distance_error_body = entry.body.to_string();
            max_distance_error_epoch = entry.epoch;
        }
    }

    let sample_count = entries.len() as f64;

    Some(JplIndependentHoldoutSummary {
        sample_count: entries.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
        max_longitude_error_deg,
        max_longitude_error_body,
        max_longitude_error_epoch,
        mean_longitude_error_deg: total_longitude_error_deg / sample_count,
        median_longitude_error_deg: median_f64(&mut longitude_errors),
        percentile_longitude_error_deg: percentile_f64(&mut longitude_errors, 0.95),
        rms_longitude_error_deg: (total_longitude_error_sq_deg / sample_count).sqrt(),
        max_latitude_error_deg,
        max_latitude_error_body,
        max_latitude_error_epoch,
        mean_latitude_error_deg: total_latitude_error_deg / sample_count,
        median_latitude_error_deg: median_f64(&mut latitude_errors),
        percentile_latitude_error_deg: percentile_f64(&mut latitude_errors, 0.95),
        rms_latitude_error_deg: (total_latitude_error_sq_deg / sample_count).sqrt(),
        max_distance_error_au,
        max_distance_error_body,
        max_distance_error_epoch,
        mean_distance_error_au: total_distance_error_au / sample_count,
        median_distance_error_au: median_f64(&mut distance_errors),
        percentile_distance_error_au: percentile_f64(&mut distance_errors, 0.95),
        rms_distance_error_au: (total_distance_error_sq_au / sample_count).sqrt(),
    })
}

impl JplIndependentHoldoutSummary {
    /// Returns the compact release-facing independent hold-out summary line.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch_suffix(body: &str, epoch: Instant) -> String {
            if body.is_empty() {
                String::new()
            } else {
                format!(" ({body} @ {})", format_instant(epoch))
            }
        }

        format!(
            "JPL independent hold-out: {} exact rows across {} bodies ({}) and {} epochs ({} → {}); max Δlon={:.12}°{}; mean Δlon={:.12}°; median Δlon={:.12}°; p95 Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}°{}; mean Δlat={:.12}°; median Δlat={:.12}°; p95 Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU{}; mean Δdist={:.12} AU; median Δdist={:.12} AU; p95 Δdist={:.12} AU; rms Δdist={:.12} AU; transparency evidence only, not a production tolerance envelope; independent JPL Horizons rows held out from the main snapshot corpus",
            self.sample_count,
            self.body_count,
            if self.bodies.is_empty() {
                "none".to_string()
            } else {
                self.bodies.join(", ")
            },
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            self.max_longitude_error_deg,
            format_body_epoch_suffix(&self.max_longitude_error_body, self.max_longitude_error_epoch),
            self.mean_longitude_error_deg,
            self.median_longitude_error_deg,
            self.percentile_longitude_error_deg,
            self.rms_longitude_error_deg,
            self.max_latitude_error_deg,
            format_body_epoch_suffix(&self.max_latitude_error_body, self.max_latitude_error_epoch),
            self.mean_latitude_error_deg,
            self.median_latitude_error_deg,
            self.percentile_latitude_error_deg,
            self.rms_latitude_error_deg,
            self.max_distance_error_au,
            format_body_epoch_suffix(&self.max_distance_error_body, self.max_distance_error_epoch),
            self.mean_distance_error_au,
            self.median_distance_error_au,
            self.percentile_distance_error_au,
            self.rms_distance_error_au,
        )
    }

    /// Returns the validated compact hold-out summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplIndependentHoldoutSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl JplIndependentHoldoutSummary {
    /// Validates that the hold-out summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySummaryValidationError> {
        if self.sample_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingSamples);
        }
        if self.body_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        let mut seen_bodies = BTreeSet::new();
        for (index, body) in self.bodies.iter().enumerate() {
            if body.trim().is_empty() {
                return Err(JplInterpolationQualitySummaryValidationError::BlankBody { index });
            }
            if !seen_bodies.insert(body) {
                return Err(
                    JplInterpolationQualitySummaryValidationError::DuplicateBody {
                        body: body.clone(),
                    },
                );
            }
        }

        if self.epoch_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                JplInterpolationQualitySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }
        for (field, value) in [
            ("max_longitude_error_deg", self.max_longitude_error_deg),
            ("mean_longitude_error_deg", self.mean_longitude_error_deg),
            (
                "median_longitude_error_deg",
                self.median_longitude_error_deg,
            ),
            (
                "percentile_longitude_error_deg",
                self.percentile_longitude_error_deg,
            ),
            ("rms_longitude_error_deg", self.rms_longitude_error_deg),
            ("max_latitude_error_deg", self.max_latitude_error_deg),
            ("mean_latitude_error_deg", self.mean_latitude_error_deg),
            ("median_latitude_error_deg", self.median_latitude_error_deg),
            (
                "percentile_latitude_error_deg",
                self.percentile_latitude_error_deg,
            ),
            ("rms_latitude_error_deg", self.rms_latitude_error_deg),
            ("max_distance_error_au", self.max_distance_error_au),
            ("mean_distance_error_au", self.mean_distance_error_au),
            ("median_distance_error_au", self.median_distance_error_au),
            (
                "percentile_distance_error_au",
                self.percentile_distance_error_au,
            ),
            ("rms_distance_error_au", self.rms_distance_error_au),
        ] {
            validate_non_negative_metric(field, value)?;
        }
        if self.max_longitude_error_deg > 0.0 && self.max_longitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_longitude_error_body",
                },
            );
        }
        if self.max_latitude_error_deg > 0.0 && self.max_latitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_latitude_error_body",
                },
            );
        }
        if self.max_distance_error_au > 0.0 && self.max_distance_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_distance_error_body",
                },
            );
        }
        if jpl_independent_holdout_summary().as_ref() != Some(self) {
            return Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }
}

/// Formats the independent hold-out summary for release-facing reporting.
pub fn format_jpl_independent_holdout_summary(summary: &JplIndependentHoldoutSummary) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("JPL independent hold-out: unavailable ({error})"),
    }
}

/// Returns the release-facing independent hold-out interpolation summary string.
pub fn jpl_independent_holdout_summary_for_report() -> String {
    match jpl_independent_holdout_summary() {
        Some(summary) => format_jpl_independent_holdout_summary(&summary),
        None => match independent_holdout_snapshot_error() {
            Some(error) => format!("JPL independent hold-out: unavailable ({error})"),
            None => "JPL independent hold-out: unavailable".to_string(),
        },
    }
}

fn median_f64(values: &mut [f64]) -> f64 {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn percentile_f64(values: &mut [f64], percentile: f64) -> f64 {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let percentile = percentile.clamp(0.0, 1.0);
    if values.len() == 1 {
        return values[0];
    }

    let position = percentile * (values.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        values[lower_index]
    } else {
        let weight = position - lower_index as f64;
        values[lower_index] * (1.0 - weight) + values[upper_index] * weight
    }
}

/// Formats the interpolation-quality summary for release-facing reports.
pub fn format_jpl_interpolation_quality_summary(
    summary: &JplInterpolationQualitySummary,
) -> String {
    summary.summary_line()
}

/// Formats the distinct-body interpolation-kind coverage for release-facing reports.
pub fn format_jpl_interpolation_quality_kind_coverage(
    coverage: &JplInterpolationQualityKindCoverage,
) -> String {
    coverage.summary_line()
}

/// Returns the release-facing interpolation-kind coverage summary string.
pub fn jpl_interpolation_quality_kind_coverage_for_report() -> String {
    match jpl_interpolation_quality_kind_coverage() {
        Some(coverage) => match coverage.validated_summary_line() {
            Ok(rendered) => rendered,
            Err(_) => "JPL interpolation quality kind coverage: unavailable".to_string(),
        },
        None => "JPL interpolation quality kind coverage: unavailable".to_string(),
    }
}

const JPL_INTERPOLATION_QUALITY_DERIVATION: &str =
    "leave-one-out interpolation evidence derived from the checked-in reference snapshot";

/// Backend-owned provenance summary for the interpolation-quality evidence slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplInterpolationQualitySourceSummary {
    /// Source attribution for the interpolation-quality evidence.
    pub source: String,
    /// Derivation note describing how the evidence slice was produced.
    pub derivation: String,
    /// Number of interpolation-quality samples in the evidence slice.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the evidence slice.
    pub body_count: usize,
    /// Number of distinct epochs represented by the evidence slice.
    pub epoch_count: usize,
}

/// Structured validation errors for an interpolation-quality provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplInterpolationQualitySourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty derivation note.
    BlankDerivation,
    /// The summary drifted away from the current derived evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplInterpolationQualitySourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSource => f.write_str("blank source"),
            Self::BlankDerivation => f.write_str("blank derivation"),
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL interpolation-quality source summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for JplInterpolationQualitySourceSummaryValidationError {}

impl JplInterpolationQualitySourceSummary {
    /// Returns a compact release-facing provenance line.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL interpolation quality source: {}; derivation={}; coverage: {} samples across {} bodies and {} epochs",
            self.source, self.derivation, self.sample_count, self.body_count, self.epoch_count,
        )
    }

    /// Returns a compact provenance line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(JplInterpolationQualitySourceSummaryValidationError::BlankSource);
        }
        if self.derivation.trim().is_empty() {
            return Err(JplInterpolationQualitySourceSummaryValidationError::BlankDerivation);
        }

        let reference_source = reference_snapshot_source_summary().source;
        if self.source != reference_source {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "source",
                },
            );
        }
        if self.derivation != JPL_INTERPOLATION_QUALITY_DERIVATION {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "derivation",
                },
            );
        }

        let derived_summary = jpl_interpolation_quality_summary().ok_or(
            JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                field: "derived_summary",
            },
        )?;
        if self.sample_count != derived_summary.sample_count {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != derived_summary.body_count {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.epoch_count != derived_summary.epoch_count {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for JplInterpolationQualitySourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the interpolation-quality evidence slice.
pub fn jpl_interpolation_quality_source_summary() -> Option<JplInterpolationQualitySourceSummary> {
    let summary = jpl_interpolation_quality_summary()?;
    Some(JplInterpolationQualitySourceSummary {
        source: reference_snapshot_source_summary().source,
        derivation: JPL_INTERPOLATION_QUALITY_DERIVATION.to_string(),
        sample_count: summary.sample_count,
        body_count: summary.body_count,
        epoch_count: summary.epoch_count,
    })
}

/// Returns the release-facing interpolation-quality provenance summary string.
pub fn jpl_interpolation_quality_source_summary_for_report() -> String {
    match jpl_interpolation_quality_source_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL interpolation quality source: unavailable ({error})"),
        },
        None => "JPL interpolation quality source: unavailable".to_string(),
    }
}

/// Formats the interpolation-quality summary together with the distinct-body coverage line.
pub fn format_jpl_interpolation_quality_summary_for_report() -> String {
    let source_summary = match jpl_interpolation_quality_source_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(rendered) => rendered,
            Err(_) => return "JPL interpolation quality: unavailable".to_string(),
        },
        None => return "JPL interpolation quality: unavailable".to_string(),
    };

    match jpl_interpolation_quality_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(mut rendered) => {
                rendered.insert_str(0, &format!("{}\n", source_summary));
                rendered.push('\n');
                rendered.push_str(&jpl_interpolation_quality_kind_coverage_for_report());
                rendered
            }
            Err(_) => "JPL interpolation quality: unavailable".to_string(),
        },
        None => "JPL interpolation quality: unavailable".to_string(),
    }
}

fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Interpolation path used for a hold-out quality sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpolationQualityKind {
    /// Four-point interpolation on a four-sample window.
    Cubic,
    /// Three-point interpolation on a three-sample window.
    Quadratic,
    /// Two-point linear fallback between adjacent samples.
    Linear,
}

impl InterpolationQualityKind {
    /// Human-readable label for release-facing reporting.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cubic => "cubic",
            Self::Quadratic => "quadratic",
            Self::Linear => "linear",
        }
    }
}

/// A coarse hold-out check for the snapshot backend's current interpolation path.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpolationQualitySample {
    /// Body evaluated by this check.
    pub body: pleiades_backend::CelestialBody,
    /// Held-out exact epoch used for comparison.
    pub epoch: Instant,
    /// Interpolation path selected for the held-out sample.
    pub interpolation_kind: InterpolationQualityKind,
    /// Span between the bracketing fixture entries in days.
    pub bracket_span_days: f64,
    /// Absolute wrapped longitude error in degrees.
    pub longitude_error_deg: f64,
    /// Absolute latitude error in degrees.
    pub latitude_error_deg: f64,
    /// Absolute distance error in astronomical units.
    pub distance_error_au: f64,
}

/// Validation errors for an interpolation-quality hold-out sample that drifted
/// away from the checked-in evidence.
#[derive(Clone, Debug, PartialEq)]
pub enum InterpolationQualitySampleValidationError {
    /// The stored epoch no longer uses TDB.
    NonTdbEpoch {
        /// Body evaluated by the sample.
        body: pleiades_backend::CelestialBody,
        /// The time scale that drifted into the sample.
        found: TimeScale,
    },
    /// A rendered field is no longer finite.
    NonFiniteField {
        /// Body evaluated by the sample.
        body: pleiades_backend::CelestialBody,
        /// Name of the field that drifted.
        field: &'static str,
    },
    /// A rendered field should stay non-negative.
    NegativeField {
        /// Body evaluated by the sample.
        body: pleiades_backend::CelestialBody,
        /// Name of the field that drifted.
        field: &'static str,
    },
}

impl fmt::Display for InterpolationQualitySampleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonTdbEpoch { body, found } => {
                write!(
                    f,
                    "interpolation sample for {body} must use TDB, found {found}"
                )
            }
            Self::NonFiniteField { body, field } => {
                write!(f, "interpolation sample for {body} has non-finite {field}")
            }
            Self::NegativeField { body, field } => {
                write!(f, "interpolation sample for {body} has negative {field}")
            }
        }
    }
}

impl std::error::Error for InterpolationQualitySampleValidationError {}

impl InterpolationQualitySample {
    /// Returns a compact release-facing summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "{} at {}: {} interpolation, bracket span {:.1} d, |Δlon|={:.12}°, |Δlat|={:.12}°, |Δdist|={:.12} AU",
            self.body,
            self.epoch.summary_line(),
            self.interpolation_kind.label(),
            self.bracket_span_days,
            self.longitude_error_deg,
            self.latitude_error_deg,
            self.distance_error_au,
        )
    }

    /// Returns `Ok(())` when the sample still matches the checked-in evidence.
    pub fn validate(&self) -> Result<(), InterpolationQualitySampleValidationError> {
        if self.epoch.scale != TimeScale::Tdb {
            return Err(InterpolationQualitySampleValidationError::NonTdbEpoch {
                body: self.body.clone(),
                found: self.epoch.scale,
            });
        }

        if !self.epoch.julian_day.days().is_finite() {
            return Err(InterpolationQualitySampleValidationError::NonFiniteField {
                body: self.body.clone(),
                field: "epoch",
            });
        }

        for (field, value) in [
            ("bracket_span_days", self.bracket_span_days),
            ("longitude_error_deg", self.longitude_error_deg),
            ("latitude_error_deg", self.latitude_error_deg),
            ("distance_error_au", self.distance_error_au),
        ] {
            if !value.is_finite() {
                return Err(InterpolationQualitySampleValidationError::NonFiniteField {
                    body: self.body.clone(),
                    field,
                });
            }
            if value < 0.0 {
                return Err(InterpolationQualitySampleValidationError::NegativeField {
                    body: self.body.clone(),
                    field,
                });
            }
        }

        Ok(())
    }
}

impl fmt::Display for InterpolationQualitySample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A reference-backend implementation backed by JPL Horizons fixture data.
#[derive(Debug, Default, Clone, Copy)]
pub struct JplSnapshotBackend;

impl JplSnapshotBackend {
    /// Creates a new snapshot backend.
    pub const fn new() -> Self {
        Self
    }
}

impl EphemerisBackend for JplSnapshotBackend {
    fn metadata(&self) -> BackendMetadata {
        let bodies = reference_bodies().to_vec();
        let epochs = reference_epochs();
        let dataset_missing = snapshot_error().is_some();
        BackendMetadata {
            id: BackendId::new("jpl-snapshot"),
            version: "0.1.0".to_string(),
            family: BackendFamily::ReferenceData,
            provenance: BackendProvenance {
                summary: "NASA/JPL Horizons DE441 geocentric fixture with exact epoch lookup, cubic interpolation on four-sample windows, and mean-obliquity equatorial output"
                    .to_string(),
                data_sources: vec![
                    "NASA/JPL Horizons API vector tables (DE441)".to_string(),
                    "Checked-in derivative CSV fixture: epoch_jd,body,x_km,y_km,z_km".to_string(),
                    "Cubic interpolation on four-sample windows, quadratic interpolation on three-sample windows, and linear fallback between adjacent same-body fixture samples".to_string(),
                ],
            },
            nominal_range: if dataset_missing {
                TimeRange::new(None, None)
            } else {
                TimeRange::new(epochs.first().copied(), epochs.last().copied())
            },
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: bodies,
            supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Exact,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: pleiades_backend::CelestialBody) -> bool {
        snapshot_entries()
            .map(|entries| entries.iter().any(|entry| entry.body == body))
            .unwrap_or(false)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        validate_request_policy(
            req,
            "the JPL snapshot backend",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            true,
            false,
        )?;

        validate_zodiac_policy(req, "the JPL snapshot backend", &[ZodiacMode::Tropical])?;

        validate_observer_policy(req, "the JPL snapshot backend", false)?;

        if let Some(error) = snapshot_error() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::MissingDataset,
                format!("the JPL snapshot corpus could not be loaded: {error}"),
            ));
        }

        let resolved = resolve_fixture_state(req.body.clone(), req.instant.julian_day.days())?;

        let mut result = EphemerisResult::new(
            BackendId::new("jpl-snapshot"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        let ecliptic = resolved.entry.ecliptic();
        result.ecliptic = Some(ecliptic);
        result.equatorial = Some(ecliptic.to_equatorial(req.instant.mean_obliquity()));
        result.motion = None::<Motion>;
        result.quality = resolved.quality;
        Ok(result)
    }
}

/// File-level metadata parsed from a checked-in JPL-style snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SnapshotManifest {
    /// Human-readable title comment from the fixture.
    pub title: Option<String>,
    /// Source comment from the fixture.
    pub source: Option<String>,
    /// Coverage comment from the fixture.
    pub coverage: Option<String>,
    /// Parsed columns comment from the fixture.
    pub columns: Vec<String>,
}

/// Structured validation errors for a parsed JPL snapshot manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotManifestValidationError {
    /// The manifest did not include a human-readable title comment.
    MissingTitle,
    /// The manifest did not include a source provenance comment.
    MissingSource,
    /// The manifest did not include any column names.
    MissingColumns,
    /// The manifest included a blank coverage comment after trimming.
    BlankCoverage,
    /// The manifest carried surrounding whitespace in a provenance field.
    SurroundedByWhitespace { field: &'static str },
    /// A parsed column name was blank after trimming.
    BlankColumn { index: usize },
    /// The manifest reused a column name after trimming.
    DuplicateColumn {
        first_index: usize,
        second_index: usize,
        name: String,
    },
}

impl SnapshotManifestValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingTitle => "missing title",
            Self::MissingSource => "missing source",
            Self::MissingColumns => "missing columns",
            Self::BlankCoverage => "blank coverage",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
            Self::BlankColumn { .. } => "blank column",
            Self::DuplicateColumn { .. } => "duplicate column",
        }
    }
}

impl fmt::Display for SnapshotManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::BlankColumn { index } => write!(f, "blank column at index {index}"),
            Self::DuplicateColumn {
                first_index,
                second_index,
                name,
            } => write!(
                f,
                "duplicate column '{name}' at index {second_index} (first seen at index {first_index})"
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for SnapshotManifestValidationError {}

impl SnapshotManifest {
    fn trimmed_or<'a>(value: Option<&'a str>, fallback: &'static str) -> Cow<'a, str> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => Cow::Borrowed(value),
            None => Cow::Borrowed(fallback),
        }
    }

    /// Returns the source label, or the provided fallback when the manifest omits it.
    pub fn source_or(&self, fallback: &'static str) -> Cow<'_, str> {
        Self::trimmed_or(self.source.as_deref(), fallback)
    }

    /// Returns the coverage label, or the provided fallback when the manifest omits it.
    pub fn coverage_or(&self, fallback: &'static str) -> Cow<'_, str> {
        Self::trimmed_or(self.coverage.as_deref(), fallback)
    }

    fn columns_summary(&self) -> String {
        if self.columns.is_empty() {
            "none".to_string()
        } else {
            self.columns.join(", ")
        }
    }

    /// Validates that the parsed manifest still exposes the expected title,
    /// source, optional coverage, and column metadata.
    pub fn validate(&self) -> Result<(), SnapshotManifestValidationError> {
        if self
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .is_none()
        {
            return Err(SnapshotManifestValidationError::MissingTitle);
        }
        if self
            .title
            .as_deref()
            .is_some_and(has_surrounding_whitespace)
        {
            return Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "title" });
        }
        if self
            .source
            .as_deref()
            .map(str::trim)
            .filter(|source| !source.is_empty())
            .is_none()
        {
            return Err(SnapshotManifestValidationError::MissingSource);
        }
        if self
            .source
            .as_deref()
            .is_some_and(has_surrounding_whitespace)
        {
            return Err(SnapshotManifestValidationError::SurroundedByWhitespace {
                field: "source",
            });
        }
        if matches!(self.coverage.as_deref(), Some(coverage) if coverage.trim().is_empty()) {
            return Err(SnapshotManifestValidationError::BlankCoverage);
        }
        if self
            .coverage
            .as_deref()
            .is_some_and(has_surrounding_whitespace)
        {
            return Err(SnapshotManifestValidationError::SurroundedByWhitespace {
                field: "coverage",
            });
        }
        if self.columns.is_empty() {
            return Err(SnapshotManifestValidationError::MissingColumns);
        }
        if let Some((index, _)) = self
            .columns
            .iter()
            .enumerate()
            .find(|(_, column)| column.trim().is_empty())
        {
            return Err(SnapshotManifestValidationError::BlankColumn { index });
        }

        let mut first_seen_columns = BTreeMap::new();
        for (index, column) in self.columns.iter().enumerate() {
            let name = column.trim();
            if let Some(first_index) = first_seen_columns.insert(name, index) {
                return Err(SnapshotManifestValidationError::DuplicateColumn {
                    first_index,
                    second_index: index,
                    name: name.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Formats the parsed manifest into the compact release-facing summary line.
    pub fn summary_line(&self, label: &str) -> String {
        self.summary_line_with_defaults(label, "unknown", "unknown")
    }

    /// Formats the parsed manifest using explicit default labels for missing provenance.
    pub fn summary_line_with_defaults(
        &self,
        label: &str,
        source_fallback: &'static str,
        coverage_fallback: &'static str,
    ) -> String {
        let title = self
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or("unknown");
        let source = self.source_or(source_fallback);
        let coverage = self.coverage_or(coverage_fallback);
        let columns = self.columns_summary();
        format!("{label}: {title}; source={source}; coverage={coverage}; columns={columns}")
    }
}

impl fmt::Display for SnapshotManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line_with_defaults("Snapshot manifest", "unknown", "unknown"))
    }
}

/// A typed manifest summary for JPL snapshot provenance reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotManifestSummary {
    /// Release-facing label for the manifest summary.
    pub label: &'static str,
    /// Parsed manifest to render.
    pub manifest: SnapshotManifest,
    /// Default source label used when the manifest omits one.
    pub source_fallback: &'static str,
    /// Default coverage label used when the manifest omits one.
    pub coverage_fallback: &'static str,
}

/// Structured validation errors for a JPL snapshot manifest summary wrapper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotManifestSummaryValidationError {
    /// The summary label was blank after trimming.
    BlankLabel,
    /// The summary label carried surrounding whitespace.
    SurroundedByWhitespace { field: &'static str },
    /// The nested manifest failed validation.
    Manifest(SnapshotManifestValidationError),
    /// The parsed provenance field does not match the expected release-facing value.
    MetadataMismatch {
        field: &'static str,
        expected: String,
        found: String,
    },
    /// The parsed column schema does not match the expected release-facing layout.
    ColumnsMismatch { expected: String, found: String },
}

impl fmt::Display for SnapshotManifestSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankLabel => f.write_str("blank label"),
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::Manifest(error) => write!(f, "manifest {error}"),
            Self::MetadataMismatch {
                field,
                expected,
                found,
            } => write!(f, "{field} mismatch: expected {expected} but found {found}"),
            Self::ColumnsMismatch { expected, found } => {
                write!(
                    f,
                    "column schema mismatch: expected {expected} but found {found}"
                )
            }
        }
    }
}

impl std::error::Error for SnapshotManifestSummaryValidationError {}

impl SnapshotManifestSummary {
    /// Validates that the wrapper still matches a usable manifest label and payload.
    pub fn validate(&self) -> Result<(), SnapshotManifestSummaryValidationError> {
        if self.label.trim().is_empty() {
            return Err(SnapshotManifestSummaryValidationError::BlankLabel);
        }
        if has_surrounding_whitespace(self.label) {
            return Err(
                SnapshotManifestSummaryValidationError::SurroundedByWhitespace { field: "label" },
            );
        }
        self.manifest
            .validate()
            .map_err(SnapshotManifestSummaryValidationError::Manifest)
    }

    /// Validates that the wrapper matches the expected provenance and column layout.
    pub fn validate_with_expected_metadata(
        &self,
        expected_title: &str,
        expected_source: &str,
        expected_coverage: &str,
        expected_columns: &[&str],
    ) -> Result<(), SnapshotManifestSummaryValidationError> {
        self.validate()?;

        let Some(title) = self.manifest.title.as_deref() else {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "title",
                expected: expected_title.to_string(),
                found: String::new(),
            });
        };
        if title != expected_title {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "title",
                expected: expected_title.to_string(),
                found: title.to_string(),
            });
        }

        let Some(source) = self.manifest.source.as_deref() else {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "source",
                expected: expected_source.to_string(),
                found: String::new(),
            });
        };
        if source != expected_source {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "source",
                expected: expected_source.to_string(),
                found: source.to_string(),
            });
        }

        let Some(coverage) = self.manifest.coverage.as_deref() else {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "coverage",
                expected: expected_coverage.to_string(),
                found: String::new(),
            });
        };
        if coverage != expected_coverage {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "coverage",
                expected: expected_coverage.to_string(),
                found: coverage.to_string(),
            });
        }

        if !self
            .manifest
            .columns
            .iter()
            .map(String::as_str)
            .eq(expected_columns.iter().copied())
        {
            return Err(SnapshotManifestSummaryValidationError::ColumnsMismatch {
                expected: expected_columns.join(", "),
                found: self.manifest.columns.join(", "),
            });
        }

        Ok(())
    }

    /// Validates that the wrapper matches the expected column layout.
    pub fn validate_with_expected_columns(
        &self,
        expected_columns: &[&str],
    ) -> Result<(), SnapshotManifestSummaryValidationError> {
        self.validate()?;

        if !self
            .manifest
            .columns
            .iter()
            .map(String::as_str)
            .eq(expected_columns.iter().copied())
        {
            return Err(SnapshotManifestSummaryValidationError::ColumnsMismatch {
                expected: expected_columns.join(", "),
                found: self.manifest.columns.join(", "),
            });
        }

        Ok(())
    }

    /// Returns the compact release-facing summary line for the manifest wrapper.
    pub fn summary_line(&self) -> String {
        self.manifest.summary_line_with_defaults(
            self.label,
            self.source_fallback,
            self.coverage_fallback,
        )
    }

    /// Returns the validated compact release-facing summary line for the manifest wrapper.
    pub fn validated_summary_line(&self) -> Result<String, SnapshotManifestSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the validated summary line after checking a specific column layout.
    pub fn validated_summary_line_with_expected_columns(
        &self,
        expected_columns: &[&str],
    ) -> Result<String, SnapshotManifestSummaryValidationError> {
        self.validate_with_expected_columns(expected_columns)?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SnapshotManifestSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn has_surrounding_whitespace(value: &str) -> bool {
    value.trim() != value || value.contains('\n') || value.contains('\r')
}

fn parse_snapshot_manifest(source: &str) -> SnapshotManifest {
    let mut manifest = SnapshotManifest::default();

    for line in source.lines() {
        let trimmed = line.trim();
        let Some(comment) = trimmed.strip_prefix('#') else {
            continue;
        };
        let comment = comment.trim();
        if let Some(value) = comment.strip_prefix("Source:") {
            manifest.source = Some(value.trim().to_string());
        } else if let Some(value) = comment.strip_prefix("Coverage:") {
            manifest.coverage = Some(value.trim().to_string());
        } else if let Some(value) = comment.strip_prefix("Columns:") {
            manifest.columns = value
                .split(',')
                .map(|column| column.trim().to_string())
                .collect();
        } else if manifest.title.is_none() && !comment.is_empty() {
            manifest.title = Some(comment.to_string());
        }
    }

    manifest
}

/// Structured validation errors for a checked-in snapshot manifest header block.
#[derive(Clone, Debug, Eq, PartialEq)]
enum SnapshotManifestHeaderStructureError {
    /// The manifest comment block contained an unexpected number of non-empty lines.
    CommentCountMismatch { expected: usize, found: usize },
    /// A specific manifest comment line drifted from the canonical header structure.
    CommentMismatch {
        index: usize,
        field: &'static str,
        expected: String,
        found: String,
    },
}

impl fmt::Display for SnapshotManifestHeaderStructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommentCountMismatch { expected, found } => write!(
                f,
                "unexpected manifest comment count: expected {expected}, found {found}"
            ),
            Self::CommentMismatch {
                index,
                field,
                expected,
                found,
            } => write!(
                f,
                "manifest comment {index} ({field}) mismatch: expected {expected} but found {found}"
            ),
        }
    }
}

impl std::error::Error for SnapshotManifestHeaderStructureError {}

fn validate_snapshot_manifest_header_structure(
    source: &str,
    expected_title: &str,
    expected_source: &str,
    expected_coverage: &str,
    expected_columns: &[&str],
) -> Result<(), SnapshotManifestHeaderStructureError> {
    let expected_comments = [
        expected_title.to_string(),
        format!("Source: {expected_source}"),
        format!("Coverage: {expected_coverage}"),
        format!("Columns: {}", expected_columns.join(",")),
    ];
    let comments = source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let comment = trimmed.strip_prefix('#')?.trim();
            if comment.is_empty() {
                None
            } else {
                Some(comment.to_string())
            }
        })
        .collect::<Vec<_>>();

    if comments.len() != expected_comments.len() {
        return Err(SnapshotManifestHeaderStructureError::CommentCountMismatch {
            expected: expected_comments.len(),
            found: comments.len(),
        });
    }

    for (index, (found, expected)) in comments.iter().zip(expected_comments.iter()).enumerate() {
        if found != expected {
            return Err(SnapshotManifestHeaderStructureError::CommentMismatch {
                index,
                field: match index {
                    0 => "title",
                    1 => "source",
                    2 => "coverage",
                    _ => "columns",
                },
                expected: expected.clone(),
                found: found.clone(),
            });
        }
    }

    Ok(())
}

/// Returns the parsed manifest for the checked-in reference snapshot.
pub fn reference_snapshot_manifest() -> &'static SnapshotManifest {
    static MANIFEST: OnceLock<SnapshotManifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        parse_snapshot_manifest(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/reference_snapshot.csv"
        )))
    })
}

/// Returns the parsed manifest for the checked-in hold-out snapshot.
pub fn independent_holdout_snapshot_manifest() -> &'static SnapshotManifest {
    static MANIFEST: OnceLock<SnapshotManifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        parse_snapshot_manifest(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/independent_holdout_snapshot.csv"
        )))
    })
}

/// One parsed record from the reference fixture.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotEntry {
    /// The body covered by the entry.
    pub body: pleiades_backend::CelestialBody,
    /// The epoch covered by the entry.
    pub epoch: Instant,
    /// Cartesian X position in kilometers.
    pub x_km: f64,
    /// Cartesian Y position in kilometers.
    pub y_km: f64,
    /// Cartesian Z position in kilometers.
    pub z_km: f64,
}

impl SnapshotEntry {
    fn ecliptic(&self) -> EclipticCoordinates {
        let radius_km =
            (self.x_km * self.x_km + self.y_km * self.y_km + self.z_km * self.z_km).sqrt();
        let longitude = Longitude::from_degrees(self.y_km.atan2(self.x_km).to_degrees());
        let latitude =
            Latitude::from_degrees((self.z_km / radius_km).clamp(-1.0, 1.0).asin().to_degrees());
        EclipticCoordinates::new(longitude, latitude, Some(radius_km / AU_IN_KM))
    }

    fn interpolate_linear(before: &Self, after: &Self, epoch_jd: f64) -> Self {
        let span_days = after.epoch.julian_day.days() - before.epoch.julian_day.days();
        let fraction = (epoch_jd - before.epoch.julian_day.days()) / span_days;
        Self {
            body: before.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lerp(before.x_km, after.x_km, fraction),
            y_km: lerp(before.y_km, after.y_km, fraction),
            z_km: lerp(before.z_km, after.z_km, fraction),
        }
    }

    fn interpolate_quadratic(a: &Self, b: &Self, c: &Self, epoch_jd: f64) -> Self {
        let xs = [
            a.epoch.julian_day.days(),
            b.epoch.julian_day.days(),
            c.epoch.julian_day.days(),
        ];
        Self {
            body: a.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lagrange_interpolate_3(epoch_jd, xs, [a.x_km, b.x_km, c.x_km]),
            y_km: lagrange_interpolate_3(epoch_jd, xs, [a.y_km, b.y_km, c.y_km]),
            z_km: lagrange_interpolate_3(epoch_jd, xs, [a.z_km, b.z_km, c.z_km]),
        }
    }

    fn interpolate_cubic(a: &Self, b: &Self, c: &Self, d: &Self, epoch_jd: f64) -> Self {
        let xs = [
            a.epoch.julian_day.days(),
            b.epoch.julian_day.days(),
            c.epoch.julian_day.days(),
            d.epoch.julian_day.days(),
        ];
        Self {
            body: a.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lagrange_interpolate_4(epoch_jd, xs, [a.x_km, b.x_km, c.x_km, d.x_km]),
            y_km: lagrange_interpolate_4(epoch_jd, xs, [a.y_km, b.y_km, c.y_km, d.y_km]),
            z_km: lagrange_interpolate_4(epoch_jd, xs, [a.z_km, b.z_km, c.z_km, d.z_km]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedFixtureState {
    entry: SnapshotEntry,
    quality: QualityAnnotation,
}

enum SnapshotState {
    Loaded(Vec<SnapshotEntry>),
    Failed(SnapshotLoadError),
}

impl SnapshotState {
    fn entries(&self) -> Option<&[SnapshotEntry]> {
        match self {
            Self::Loaded(entries) => Some(entries.as_slice()),
            Self::Failed(_) => None,
        }
    }

    fn error(&self) -> Option<&SnapshotLoadError> {
        match self {
            Self::Loaded(_) => None,
            Self::Failed(error) => Some(error),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SnapshotLoadError {
    line_number: usize,
    kind: SnapshotLoadErrorKind,
}

impl SnapshotLoadError {
    fn new(line_number: usize, kind: SnapshotLoadErrorKind) -> Self {
        Self { line_number, kind }
    }
}

impl fmt::Display for SnapshotLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line_number, self.kind)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum SnapshotLoadErrorKind {
    MissingColumn {
        column: &'static str,
    },
    UnexpectedExtraColumns,
    BlankBody,
    UnsupportedBody {
        body: String,
    },
    InvalidNumber {
        column: &'static str,
        value: String,
    },
    DuplicateEntry {
        body: String,
        epoch: Instant,
        first_line: usize,
    },
}

impl fmt::Display for SnapshotLoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColumn { column } => write!(f, "missing {column} column"),
            Self::UnexpectedExtraColumns => f.write_str("unexpected extra columns"),
            Self::BlankBody => f.write_str("blank body"),
            Self::UnsupportedBody { body } => write!(f, "unsupported body '{body}'"),
            Self::InvalidNumber { column, value } => {
                write!(f, "invalid {column} value '{value}'")
            }
            Self::DuplicateEntry {
                body,
                epoch,
                first_line,
            } => {
                write!(
                    f,
                    "duplicate row for body '{body}' at {} (first seen at line {first_line})",
                    format_instant(*epoch)
                )
            }
        }
    }
}

fn lerp(start: f64, end: f64, fraction: f64) -> f64 {
    start + (end - start) * fraction
}

fn lagrange_interpolate_3(x: f64, xs: [f64; 3], ys: [f64; 3]) -> f64 {
    let [x0, x1, x2] = xs;
    let [y0, y1, y2] = ys;

    let l0 = (x - x1) * (x - x2) / ((x0 - x1) * (x0 - x2));
    let l1 = (x - x0) * (x - x2) / ((x1 - x0) * (x1 - x2));
    let l2 = (x - x0) * (x - x1) / ((x2 - x0) * (x2 - x1));

    y0 * l0 + y1 * l1 + y2 * l2
}

fn lagrange_interpolate_4(x: f64, xs: [f64; 4], ys: [f64; 4]) -> f64 {
    let [x0, x1, x2, x3] = xs;
    let [y0, y1, y2, y3] = ys;

    let l0 = (x - x1) * (x - x2) * (x - x3) / ((x0 - x1) * (x0 - x2) * (x0 - x3));
    let l1 = (x - x0) * (x - x2) * (x - x3) / ((x1 - x0) * (x1 - x2) * (x1 - x3));
    let l2 = (x - x0) * (x - x1) * (x - x3) / ((x2 - x0) * (x2 - x1) * (x2 - x3));
    let l3 = (x - x0) * (x - x1) * (x - x2) / ((x3 - x0) * (x3 - x1) * (x3 - x2));

    y0 * l0 + y1 * l1 + y2 * l2 + y3 * l3
}

fn interpolate_fixture_state(
    entries: &[SnapshotEntry],
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Option<SnapshotEntry> {
    let mut body_entries = entries
        .iter()
        .filter(|entry| entry.body == body)
        .collect::<Vec<_>>();

    if body_entries.len() < 3 {
        return None;
    }

    body_entries.sort_by(|left, right| {
        left.epoch
            .julian_day
            .days()
            .partial_cmp(&right.epoch.julian_day.days())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let body_entry_count = body_entries.len();
    let mut ranked = body_entries
        .into_iter()
        .map(|entry| ((entry.epoch.julian_day.days() - epoch_jd).abs(), entry))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.1
                    .epoch
                    .julian_day
                    .days()
                    .partial_cmp(&right.1.epoch.julian_day.days())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let window_size = if body_entry_count >= 4 { 4 } else { 3 };
    let mut selected = ranked
        .into_iter()
        .take(window_size)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();

    match selected.len() {
        4 => {
            selected.sort_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .partial_cmp(&right.epoch.julian_day.days())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            Some(SnapshotEntry::interpolate_cubic(
                selected[0],
                selected[1],
                selected[2],
                selected[3],
                epoch_jd,
            ))
        }
        3 => {
            selected.sort_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .partial_cmp(&right.epoch.julian_day.days())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            Some(SnapshotEntry::interpolate_quadratic(
                selected[0],
                selected[1],
                selected[2],
                epoch_jd,
            ))
        }
        _ => None,
    }
}

fn angular_degrees_delta(left: f64, right: f64) -> f64 {
    let delta = (left - right + 180.0).rem_euclid(360.0) - 180.0;
    delta.abs()
}

fn snapshot_state() -> &'static SnapshotState {
    static STATE: OnceLock<SnapshotState> = OnceLock::new();
    STATE.get_or_init(|| match load_snapshot() {
        Ok(entries) => SnapshotState::Loaded(entries),
        Err(error) => SnapshotState::Failed(error),
    })
}

fn snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    snapshot_state().entries()
}

fn snapshot_error() -> Option<&'static SnapshotLoadError> {
    snapshot_state().error()
}

fn snapshot_bodies() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = snapshot_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

const REFERENCE_SNAPSHOT_1749_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_360_233.5;
const REFERENCE_SNAPSHOT_REFERENCE_ONLY_EPOCH_JD: f64 = 2_378_498.5;
const REFERENCE_SNAPSHOT_1800_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_378_499.0;
const REFERENCE_SNAPSHOT_2500_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_500_000.0;
const REFERENCE_SNAPSHOT_BOUNDARY_ONLY_EPOCH_JD: f64 = 2_451_917.5;

fn is_reference_snapshot_only_epoch(epoch: f64) -> bool {
    matches!(
        epoch,
        x if x == REFERENCE_SNAPSHOT_REFERENCE_ONLY_EPOCH_JD
            || x == REFERENCE_SNAPSHOT_BOUNDARY_ONLY_EPOCH_JD
    )
}

fn comparison_snapshot_entries() -> &'static [SnapshotEntry] {
    static SNAPSHOT: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    SNAPSHOT
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days() != 2_451_913.5
                        && !is_reference_snapshot_only_epoch(entry.epoch.julian_day.days())
                })
                .cloned()
                .collect()
        })
        .as_slice()
}

fn independent_holdout_state() -> &'static SnapshotState {
    static STATE: OnceLock<SnapshotState> = OnceLock::new();
    STATE.get_or_init(|| {
        match load_snapshot_from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/independent_holdout_snapshot.csv"
        ))) {
            Ok(entries) => SnapshotState::Loaded(entries),
            Err(error) => SnapshotState::Failed(error),
        }
    })
}

/// Returns the parsed independent hold-out fixture entries.
///
/// The entries preserve the checked-in order from the derivative CSV so
/// downstream validation and reproducibility tooling can rebuild the exact
/// hold-out request corpus without re-parsing the fixture.
pub fn independent_holdout_snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    independent_holdout_state().entries()
}

/// Returns the independent hold-out request corpus in the requested frame.
///
/// The requests preserve the checked-in row order and the stored epochs from
/// the derivative CSV. Callers can reuse this corpus for exact batch checks or
/// retag the returned instants with a different time-scale policy if needed.
pub fn independent_holdout_snapshot_requests(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`independent_holdout_snapshot_requests`].
#[doc(alias = "independent_holdout_snapshot_requests")]
pub fn independent_holdout_snapshot_request_corpus(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_requests(frame)
}

/// Returns the ecliptic independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_requests`].
#[doc(alias = "independent_holdout_snapshot_requests")]
pub fn independent_holdout_snapshot_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
}

/// Returns the ecliptic independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_ecliptic_request_corpus`].
#[doc(alias = "independent_holdout_snapshot_ecliptic_request_corpus")]
pub fn independent_holdout_snapshot_ecliptic_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_ecliptic_request_corpus()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_requests`].
#[doc(alias = "independent_holdout_snapshot_requests")]
pub fn independent_holdout_snapshot_equatorial_parity_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_requests(CoordinateFrame::Equatorial)
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_batch_parity_requests(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_batch_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_batch_parity_requests()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_request_corpus`].
#[doc(alias = "independent_holdout_snapshot_equatorial_request_corpus")]
pub fn independent_holdout_snapshot_equatorial_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_request_corpus()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_parity_requests()
}

/// Returns the mixed-scale independent hold-out request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order, alternate TT and TDB labels
/// per row, and keep the ecliptic frame so downstream tooling can reuse the
/// exact mixed-scale batch slice without reconstructing it from the snapshot metadata.
pub fn independent_holdout_snapshot_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_entries().map(|entries| {
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(
                    entry.epoch.julian_day,
                    if index % 2 == 0 {
                        TimeScale::Tt
                    } else {
                        TimeScale::Tdb
                    },
                ),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the mixed-scale independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_batch_parity_requests")]
pub fn independent_holdout_snapshot_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_time_scale_batch_parity_requests(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_time_scale_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_tt_tdb_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_time_scale_request_corpus`].
#[doc(alias = "independent_holdout_snapshot_mixed_time_scale_request_corpus")]
pub fn independent_holdout_snapshot_mixed_tt_tdb_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_time_scale_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
}

fn independent_holdout_bodies() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = independent_holdout_snapshot_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

fn independent_holdout_snapshot_error() -> Option<&'static SnapshotLoadError> {
    independent_holdout_state().error()
}

fn comparison_body_list() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            for entry in comparison_snapshot_entries() {
                if !bodies.contains(&entry.body) {
                    bodies.push(entry.body.clone());
                }
            }
            bodies
        })
        .as_slice()
}

fn reference_asteroid_list() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            for entry in snapshot_entries().into_iter().flatten() {
                if is_reference_asteroid(&entry.body) && !bodies.contains(&entry.body) {
                    bodies.push(entry.body.clone());
                }
            }
            bodies
        })
        .as_slice()
}

fn reference_asteroid_requests_with_frame_selector(
    frame_for_index: impl Fn(usize) -> CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    let evidence = reference_asteroid_evidence();
    if evidence.is_empty() {
        return None;
    }

    Some(
        evidence
            .iter()
            .enumerate()
            .map(|(index, sample)| EphemerisRequest {
                body: sample.body.clone(),
                instant: sample.epoch,
                observer: None,
                frame: frame_for_index(index),
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect(),
    )
}

fn reference_asteroid_evidence_list() -> &'static [ReferenceAsteroidEvidence] {
    static EVIDENCE: OnceLock<Vec<ReferenceAsteroidEvidence>> = OnceLock::new();
    EVIDENCE
        .get_or_init(|| {
            let mut evidence = Vec::new();
            let Some(entries) = snapshot_entries() else {
                return evidence;
            };

            for body in reference_asteroid_list() {
                if let Some(entry) = entries.iter().find(|entry| {
                    &entry.body == body && entry.epoch.julian_day.days() == REFERENCE_EPOCH_JD
                }) {
                    let ecliptic = entry.ecliptic();
                    evidence.push(ReferenceAsteroidEvidence {
                        body: body.clone(),
                        epoch: entry.epoch,
                        longitude_deg: ecliptic.longitude.degrees(),
                        latitude_deg: ecliptic.latitude.degrees(),
                        distance_au: ecliptic.distance_au.unwrap_or_default(),
                    });
                }
            }

            evidence
        })
        .as_slice()
}

fn reference_asteroid_equatorial_evidence_list() -> &'static [ReferenceAsteroidEquatorialEvidence] {
    static EVIDENCE: OnceLock<Vec<ReferenceAsteroidEquatorialEvidence>> = OnceLock::new();
    EVIDENCE
        .get_or_init(|| {
            reference_asteroid_evidence()
                .iter()
                .map(|sample| {
                    let ecliptic = EclipticCoordinates::new(
                        Longitude::from_degrees(sample.longitude_deg),
                        Latitude::from_degrees(sample.latitude_deg),
                        Some(sample.distance_au),
                    );
                    ReferenceAsteroidEquatorialEvidence {
                        body: sample.body.clone(),
                        epoch: sample.epoch,
                        equatorial: ecliptic.to_equatorial(sample.epoch.mean_obliquity()),
                    }
                })
                .collect()
        })
        .as_slice()
}

fn interpolation_quality_sample_list() -> &'static [InterpolationQualitySample] {
    static SAMPLES: OnceLock<Vec<InterpolationQualitySample>> = OnceLock::new();
    SAMPLES
        .get_or_init(|| {
            let mut samples = Vec::new();
            let Some(entries) = snapshot_entries() else {
                return samples;
            };

            let entries = entries
                .iter()
                .filter(|entry| !is_reference_snapshot_only_epoch(entry.epoch.julian_day.days()))
                .cloned()
                .collect::<Vec<_>>();

            for body in comparison_body_list() {
                let mut body_entries = entries
                    .iter()
                    .filter(|entry| &entry.body == body)
                    .collect::<Vec<_>>();
                body_entries.sort_by(|left, right| {
                    left.epoch
                        .julian_day
                        .days()
                        .partial_cmp(&right.epoch.julian_day.days())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                for window in body_entries.windows(3) {
                    let before = window[0];
                    let exact = window[1];
                    let after = window[2];
                    let epoch_jd = exact.epoch.julian_day.days();
                    let leave_one_out_entries = entries
                        .iter()
                        .filter(|entry| {
                            entry.body != exact.body || entry.epoch.julian_day.days() != epoch_jd
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    let interpolation_kind = match body_entries.len().saturating_sub(1) {
                        0..=2 => InterpolationQualityKind::Linear,
                        3 => InterpolationQualityKind::Quadratic,
                        _ => InterpolationQualityKind::Cubic,
                    };
                    let interpolated = resolve_fixture_state_from_entries(
                        &leave_one_out_entries,
                        exact.body.clone(),
                        epoch_jd,
                    )
                    .expect("held-out sample should still interpolate")
                    .entry;
                    let exact_ecliptic = exact.ecliptic();
                    let interpolated_ecliptic = interpolated.ecliptic();
                    let exact_distance = exact_ecliptic.distance_au.unwrap_or_default();
                    let interpolated_distance =
                        interpolated_ecliptic.distance_au.unwrap_or_default();

                    samples.push(InterpolationQualitySample {
                        body: exact.body.clone(),
                        epoch: exact.epoch,
                        interpolation_kind,
                        bracket_span_days: after.epoch.julian_day.days()
                            - before.epoch.julian_day.days(),
                        longitude_error_deg: angular_degrees_delta(
                            exact_ecliptic.longitude.degrees(),
                            interpolated_ecliptic.longitude.degrees(),
                        ),
                        latitude_error_deg: (exact_ecliptic.latitude.degrees()
                            - interpolated_ecliptic.latitude.degrees())
                        .abs(),
                        distance_error_au: (exact_distance - interpolated_distance).abs(),
                    });
                }
            }

            samples
        })
        .as_slice()
}

fn snapshot_instants() -> &'static [Instant] {
    static INSTANTS: OnceLock<Vec<Instant>> = OnceLock::new();
    INSTANTS
        .get_or_init(|| {
            let mut instants = Vec::new();
            if let Some(entries) = snapshot_entries() {
                for entry in entries {
                    if !instants.contains(&entry.epoch) {
                        instants.push(entry.epoch);
                    }
                }
            }
            instants
        })
        .as_slice()
}

fn resolve_fixture_state(
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Result<ResolvedFixtureState, EphemerisError> {
    let Some(entries) = snapshot_entries() else {
        return Err(EphemerisError::new(
            EphemerisErrorKind::MissingDataset,
            "the JPL fixture corpus is unavailable",
        ));
    };

    resolve_fixture_state_from_entries(entries, body, epoch_jd)
}

fn resolve_fixture_state_from_entries(
    entries: &[SnapshotEntry],
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Result<ResolvedFixtureState, EphemerisError> {
    let mut exact = None;
    let mut before = None;
    let mut after = None;
    let mut body_seen = false;

    for entry in entries.iter().filter(|entry| entry.body == body) {
        body_seen = true;
        let entry_jd = entry.epoch.julian_day.days();
        if entry_jd == epoch_jd {
            exact = Some(entry);
            break;
        }
        if entry_jd < epoch_jd
            && before.is_none_or(|candidate: &SnapshotEntry| {
                entry_jd > candidate.epoch.julian_day.days()
            })
        {
            before = Some(entry);
        }
        if entry_jd > epoch_jd
            && after.is_none_or(|candidate: &SnapshotEntry| {
                entry_jd < candidate.epoch.julian_day.days()
            })
        {
            after = Some(entry);
        }
    }

    if let Some(entry) = exact {
        return Ok(ResolvedFixtureState {
            entry: entry.clone(),
            quality: QualityAnnotation::Exact,
        });
    }

    if !body_seen {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedBody,
            format!("the JPL fixture corpus does not include {body}"),
        ));
    }

    if before.is_some() && after.is_some() {
        if let Some(entry) = interpolate_fixture_state(entries, body.clone(), epoch_jd) {
            return Ok(ResolvedFixtureState {
                entry,
                quality: QualityAnnotation::Interpolated,
            });
        }
    }

    match (before, after) {
        (Some(before), Some(after)) => Ok(ResolvedFixtureState {
            entry: SnapshotEntry::interpolate_linear(before, after, epoch_jd),
            quality: QualityAnnotation::Interpolated,
        }),
        _ => Err(EphemerisError::new(
            EphemerisErrorKind::OutOfRangeInstant,
            "the requested instant is outside adjacent JPL fixture samples for that body",
        )),
    }
}

fn load_snapshot() -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    load_snapshot_from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    )))
}

fn load_snapshot_from_str(source: &str) -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    let mut seen_entries = BTreeMap::new();

    source
        .lines()
        .enumerate()
        .map(|(index, line)| {
            parse_snapshot_line(index + 1, line).map(|entry| entry.map(|entry| (index + 1, entry)))
        })
        .try_fold(Vec::new(), |mut entries, record| {
            if let Some((line_number, entry)) = record? {
                let entry_key = (
                    entry.body.to_string(),
                    entry.epoch.julian_day.days().to_bits(),
                );
                if let Some(first_line) = seen_entries.get(&entry_key).copied() {
                    return Err(SnapshotLoadError::new(
                        line_number,
                        SnapshotLoadErrorKind::DuplicateEntry {
                            body: entry_key.0,
                            epoch: entry.epoch,
                            first_line,
                        },
                    ));
                }
                seen_entries.insert(entry_key, line_number);
                entries.push(entry);
            }
            Ok(entries)
        })
}

fn parse_snapshot_line(
    line_number: usize,
    line: &str,
) -> Result<Option<SnapshotEntry>, SnapshotLoadError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }

    let mut parts = trimmed.split(',').map(str::trim);
    let epoch_jd = next_part(&mut parts, line_number, "epoch")?;
    let body = next_part(&mut parts, line_number, "body")?;
    let x_km = next_part(&mut parts, line_number, "x")?;
    let y_km = next_part(&mut parts, line_number, "y")?;
    let z_km = next_part(&mut parts, line_number, "z")?;

    if parts.next().is_some() {
        return Err(SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::UnexpectedExtraColumns,
        ));
    }

    Ok(Some(SnapshotEntry {
        body: parse_body(body, line_number)?,
        epoch: Instant::new(
            JulianDay::from_days(parse_f64(epoch_jd, line_number, "epoch_jd")?),
            TimeScale::Tdb,
        ),
        x_km: parse_f64(x_km, line_number, "x_km")?,
        y_km: parse_f64(y_km, line_number, "y_km")?,
        z_km: parse_f64(z_km, line_number, "z_km")?,
    }))
}

fn next_part<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    line_number: usize,
    column: &'static str,
) -> Result<&'a str, SnapshotLoadError> {
    parts.next().ok_or_else(|| {
        SnapshotLoadError::new(line_number, SnapshotLoadErrorKind::MissingColumn { column })
    })
}

fn parse_body(
    body: &str,
    line_number: usize,
) -> Result<pleiades_backend::CelestialBody, SnapshotLoadError> {
    if body.is_empty() {
        return Err(SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::BlankBody,
        ));
    }

    let body = match body {
        "Sun" => pleiades_backend::CelestialBody::Sun,
        "Moon" => pleiades_backend::CelestialBody::Moon,
        "Mercury" => pleiades_backend::CelestialBody::Mercury,
        "Venus" => pleiades_backend::CelestialBody::Venus,
        "Mars" => pleiades_backend::CelestialBody::Mars,
        "Jupiter" => pleiades_backend::CelestialBody::Jupiter,
        "Saturn" => pleiades_backend::CelestialBody::Saturn,
        "Uranus" => pleiades_backend::CelestialBody::Uranus,
        "Neptune" => pleiades_backend::CelestialBody::Neptune,
        "Pluto" => pleiades_backend::CelestialBody::Pluto,
        "Ceres" => pleiades_backend::CelestialBody::Ceres,
        "Pallas" => pleiades_backend::CelestialBody::Pallas,
        "Juno" => pleiades_backend::CelestialBody::Juno,
        "Vesta" => pleiades_backend::CelestialBody::Vesta,
        other => {
            let Some((catalog, designation)) = other.split_once(':') else {
                return Err(SnapshotLoadError::new(
                    line_number,
                    SnapshotLoadErrorKind::UnsupportedBody {
                        body: other.to_string(),
                    },
                ));
            };

            let catalog = catalog.trim();
            let designation = designation.trim();
            if catalog.is_empty() || designation.is_empty() {
                return Err(SnapshotLoadError::new(
                    line_number,
                    SnapshotLoadErrorKind::UnsupportedBody {
                        body: other.to_string(),
                    },
                ));
            }

            pleiades_backend::CelestialBody::Custom(CustomBodyId::new(catalog, designation))
        }
    };

    Ok(body)
}

fn is_comparison_body(body: &pleiades_backend::CelestialBody) -> bool {
    matches!(
        body,
        pleiades_backend::CelestialBody::Sun
            | pleiades_backend::CelestialBody::Moon
            | pleiades_backend::CelestialBody::Mercury
            | pleiades_backend::CelestialBody::Venus
            | pleiades_backend::CelestialBody::Mars
            | pleiades_backend::CelestialBody::Jupiter
            | pleiades_backend::CelestialBody::Saturn
            | pleiades_backend::CelestialBody::Uranus
            | pleiades_backend::CelestialBody::Neptune
            | pleiades_backend::CelestialBody::Pluto
    )
}

fn is_reference_asteroid(body: &pleiades_backend::CelestialBody) -> bool {
    match body {
        pleiades_backend::CelestialBody::Ceres
        | pleiades_backend::CelestialBody::Pallas
        | pleiades_backend::CelestialBody::Juno
        | pleiades_backend::CelestialBody::Vesta => true,
        pleiades_backend::CelestialBody::Custom(custom) if custom.catalog == "asteroid" => true,
        _ => false,
    }
}

fn parse_f64(
    value: &str,
    line_number: usize,
    column: &'static str,
) -> Result<f64, SnapshotLoadError> {
    value.parse::<f64>().map_err(|_error| {
        SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::InvalidNumber {
                column,
                value: value.to_string(),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};

    #[test]
    fn reference_snapshot_covers_the_expected_bodies_and_epochs() {
        let metadata = JplSnapshotBackend::new().metadata();
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Sun));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Moon));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Pluto));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Ceres));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Pallas));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Juno));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Vesta));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Custom(CustomBodyId::new(
                "asteroid", "433-Eros"
            ))));
        assert_eq!(
            reference_asteroids(),
            [
                pleiades_backend::CelestialBody::Ceres,
                pleiades_backend::CelestialBody::Pallas,
                pleiades_backend::CelestialBody::Juno,
                pleiades_backend::CelestialBody::Vesta,
                pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            ]
        );
        assert!(metadata.nominal_range.start.is_some());
        assert!(metadata.nominal_range.end.is_some());
        let start = metadata
            .nominal_range
            .start
            .expect("start epoch should exist");
        let end = metadata.nominal_range.end.expect("end epoch should exist");
        assert!(start.julian_day.days() < end.julian_day.days());
        assert_eq!(reference_epochs().len(), 18);
        assert_eq!(
            reference_snapshot()
                .iter()
                .filter(|entry| entry.epoch.julian_day.days() == 2_400_000.0)
                .count(),
            10
        );
        assert_eq!(
            reference_snapshot()
                .iter()
                .filter(|entry| entry.epoch.julian_day.days() == 2_500_000.0)
                .count(),
            15
        );
        assert_eq!(
            reference_snapshot()
                .iter()
                .filter(|entry| entry.epoch.julian_day.days() == 2_600_000.0)
                .count(),
            1
        );
    }

    #[test]
    fn reference_snapshot_summary_reports_the_expected_coverage() {
        let summary =
            reference_snapshot_summary().expect("reference snapshot summary should exist");
        summary
            .validate()
            .expect("reference snapshot summary should validate");
        assert_eq!(summary.row_count, 195);
        assert_eq!(summary.body_count, 15);
        assert_eq!(summary.bodies, reference_bodies());
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.asteroid_row_count, 61);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_360_233.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "Reference snapshot coverage: 195 rows across 15 bodies and 18 epochs (61 asteroid rows; JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}",
                format_bodies(reference_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_snapshot_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_lunar_boundary_summary_reports_the_expected_window() {
        let summary = reference_snapshot_lunar_boundary_summary()
            .expect("reference lunar boundary summary should exist");
        assert_eq!(summary.sample_count, 2);
        assert_eq!(summary.epoch_count, 2);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_912.5);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference lunar boundary evidence: 2 exact Moon samples at JD 2451911.5 (TDB)..JD 2451912.5 (TDB); high-curvature interpolation window"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_lunar_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_summary_reports_the_expected_window() {
        let summary = reference_snapshot_high_curvature_summary()
            .expect("reference high-curvature summary should exist");
        assert_eq!(summary.sample_count, 50);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.bodies.len(), 10);
        assert_eq!(summary.epoch_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_916.5);
        assert_eq!(summary.bodies[0], pleiades_backend::CelestialBody::Sun);
        assert_eq!(summary.bodies[9], pleiades_backend::CelestialBody::Jupiter);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference major-body high-curvature evidence: 50 exact samples across 10 bodies and 5 epochs (JD 2451911.5 (TDB)..JD 2451916.5 (TDB)); bodies: Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter; high-curvature interpolation window"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_high_curvature_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_boundary_epoch_coverage_summary_reports_the_sparse_epochs() {
        let summary = reference_snapshot_boundary_epoch_coverage_summary()
            .expect("reference snapshot boundary epoch coverage summary should exist");
        assert_eq!(summary.sample_count, 57);
        assert_eq!(summary.epoch_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_913.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_917.5);
        assert_eq!(summary.windows.len(), 5);
        assert_eq!(summary.windows[0].body_count, 15);
        assert_eq!(summary.windows[2].body_count, 5);
        assert_eq!(
            summary.windows[2].bodies[0],
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.windows[2].bodies[4],
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(
            summary
                .summary_line()
                .contains("Reference snapshot boundary epoch coverage: 57 exact samples across 5 epochs (JD 2451913.5 (TDB)..JD 2451917.5 (TDB)); epochs:")
        );
        assert!(summary
            .summary_line()
            .contains("JD 2451915.5 (TDB): 5 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); sparse asteroid-only day"));
        assert_eq!(summary.summary_line(), "Reference snapshot boundary epoch coverage: 57 exact samples across 5 epochs (JD 2451913.5 (TDB)..JD 2451917.5 (TDB)); epochs: JD 2451913.5 (TDB): 15 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); JD 2451914.5 (TDB): 15 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); JD 2451915.5 (TDB): 5 bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); sparse asteroid-only day; JD 2451916.5 (TDB): 15 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); JD 2451917.5 (TDB): 7 bodies (Mars, Jupiter, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)");
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_boundary_epoch_coverage_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_boundary_epoch_coverage_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_boundary_epoch_coverage_summary()
            .expect("reference snapshot boundary epoch coverage summary should exist");
        summary.windows[2].body_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted boundary epoch coverage summary should fail validation");

        assert!(matches!(
            error,
            ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
                field: "windows"
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_boundary_epoch_coverage_summary_for_report(),
            reference_snapshot_boundary_epoch_coverage_summary()
                .expect("reference snapshot boundary epoch coverage summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_1749_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = reference_snapshot_1749_major_body_boundary_summary()
            .expect("reference 1749 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 9);
        assert_eq!(summary.sample_bodies.len(), 9);
        assert_eq!(summary.epoch.julian_day.days(), 2_360_233.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[1],
            pleiades_backend::CelestialBody::Moon
        );
        assert_eq!(
            summary.sample_bodies[2],
            pleiades_backend::CelestialBody::Mercury
        );
        assert_eq!(
            summary.sample_bodies[3],
            pleiades_backend::CelestialBody::Venus
        );
        assert_eq!(
            summary.sample_bodies[4],
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(
            summary.sample_bodies[5],
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(
            summary.sample_bodies[6],
            pleiades_backend::CelestialBody::Saturn
        );
        assert_eq!(
            summary.sample_bodies[7],
            pleiades_backend::CelestialBody::Uranus
        );
        assert_eq!(
            summary.sample_bodies[8],
            pleiades_backend::CelestialBody::Neptune
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference 1749 major-body boundary evidence: 9 exact samples at JD 2360233.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune); 1749-12-31 boundary sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_1749_major_body_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_major_body_boundary_summary_reports_the_boundary_day() {
        let summary = reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 2);
        assert_eq!(summary.sample_bodies.len(), 2);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_917.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(
            summary.sample_bodies[1],
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference major-body boundary evidence: 2 exact samples at JD 2451917.5 (TDB) (Mars, Jupiter); 2001-01-08 boundary sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_major_body_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_1749_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_1749_major_body_boundary_summary()
            .expect("reference 1749 major-body boundary summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted 1749 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference1749MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                sample_count: 10,
                derived_sample_count: 9
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_1749_major_body_boundary_summary_for_report(),
            reference_snapshot_1749_major_body_boundary_summary()
                .expect("reference 1749 major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                sample_count: 3,
                derived_sample_count: 2
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_major_body_boundary_summary_for_report(),
            reference_snapshot_major_body_boundary_summary()
                .expect("reference major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_major_body_boundary_summary_validation_rejects_body_drift() {
        let mut summary = reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Mars,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_major_body_boundary_summary_for_report(),
            reference_snapshot_major_body_boundary_summary()
                .expect("reference major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_mars_jupiter_boundary_summary_reports_the_boundary_day() {
        let summary = reference_snapshot_mars_jupiter_boundary_summary()
            .expect("reference Mars/Jupiter boundary summary should exist");
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.sample_bodies.len(), 3);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_918.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(
            summary.sample_bodies[1],
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(
            summary.sample_bodies[2],
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference Mars/Jupiter boundary evidence: 3 exact samples at JD 2451918.5 (TDB) (Mars, Jupiter, asteroid:433-Eros); 2001-01-09 boundary sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_mars_jupiter_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_mars_jupiter_boundary_summary_validation_rejects_body_drift() {
        let mut summary = reference_snapshot_mars_jupiter_boundary_summary()
            .expect("reference Mars/Jupiter boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted Mars/Jupiter boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceMarsJupiterBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Mars,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_mars_jupiter_boundary_summary_for_report(),
            reference_snapshot_mars_jupiter_boundary_summary()
                .expect("reference Mars/Jupiter boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_early_major_body_boundary_summary_reports_the_early_boundary_day() {
        let summary = reference_snapshot_early_major_body_boundary_summary()
            .expect("reference early major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference early major-body boundary evidence: 10 exact samples at JD 2378498.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto)"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_early_major_body_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_early_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_early_major_body_boundary_summary()
            .expect("reference early major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted early major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceEarlyMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_early_major_body_boundary_summary_for_report(),
            reference_snapshot_early_major_body_boundary_summary()
                .expect("reference early major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_1800_major_body_boundary_summary_reports_the_1800_boundary_day() {
        let summary = reference_snapshot_1800_major_body_boundary_summary()
            .expect("reference 1800 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 5);
        assert_eq!(summary.sample_bodies.len(), 5);
        assert_eq!(summary.epoch.julian_day.days(), 2_378_499.0);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(
            summary.sample_bodies[4],
            pleiades_backend::CelestialBody::Venus
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference 1800 major-body boundary evidence: 5 exact samples at JD 2378499.0 (TDB) (Mars, Mercury, Moon, Sun, Venus); 1800-01-03 boundary sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_1800_major_body_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_1800_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_1800_major_body_boundary_summary()
            .expect("reference 1800 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 1800 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference1800MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Mars,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_1800_major_body_boundary_summary_for_report(),
            reference_snapshot_1800_major_body_boundary_summary()
                .expect("reference 1800 major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2500_major_body_boundary_summary_reports_the_terminal_boundary_day() {
        let summary = reference_snapshot_2500_major_body_boundary_summary()
            .expect("reference 2500 major-body boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch.julian_day.days(), 2_500_000.0);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference 2500 major-body boundary evidence: 10 exact samples at JD 2500000.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2500-01-01 boundary sample"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_2500_major_body_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_2500_major_body_boundary_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_2500_major_body_boundary_summary()
            .expect("reference 2500 major-body boundary summary should exist");
        summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted 2500 major-body boundary summary should fail validation");

        assert!(matches!(
            error,
            Reference2500MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: pleiades_backend::CelestialBody::Sun,
                found: pleiades_backend::CelestialBody::Moon
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_2500_major_body_boundary_summary_for_report(),
            reference_snapshot_2500_major_body_boundary_summary()
                .expect("reference 2500 major-body boundary summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_window_summary_reports_the_expected_windows() {
        let summary = reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist");
        assert_eq!(summary.sample_count, 50);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_916.5);
        assert_eq!(
            summary.sample_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.sample_bodies[9],
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(
            summary.windows[0].body,
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(summary.windows[0].sample_count, 5);
        assert_eq!(summary.windows[0].epoch_count, 5);
        assert_eq!(
            summary.windows[9].body,
            pleiades_backend::CelestialBody::Jupiter
        );
        assert_eq!(summary.windows[9].sample_count, 5);
        assert_eq!(summary.windows[9].epoch_count, 5);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference major-body high-curvature windows: 50 source-backed samples across 10 bodies and 5 epochs (JD 2451911.5 (TDB)..JD 2451916.5 (TDB)); windows: Sun: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Moon: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Mercury: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Venus: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Saturn: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Uranus: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Neptune: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Pluto: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Mars: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Jupiter: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB)"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_high_curvature_window_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_window_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted high-curvature window summary should fail validation");

        assert!(matches!(
            error,
            ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_high_curvature_window_summary_for_report(),
            reference_snapshot_high_curvature_window_summary()
                .expect("reference high-curvature window summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_high_curvature_window_summary_validation_rejects_window_drift() {
        let mut summary = reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist");
        summary.windows[0].body = pleiades_backend::CelestialBody::Moon;

        let error = summary
            .validate()
            .expect_err("drifted high-curvature window summary should fail validation");

        assert!(matches!(
            error,
            ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync { field: "windows" }
        ));
        assert!(summary.validated_summary_line().is_err());
        assert_eq!(
            reference_snapshot_high_curvature_window_summary_for_report(),
            reference_snapshot_high_curvature_window_summary()
                .expect("reference high-curvature window summary should exist")
                .summary_line()
        );
    }

    #[test]
    fn reference_snapshot_lunar_boundary_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_lunar_boundary_summary()
            .expect("reference lunar boundary summary should exist");
        summary.sample_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted lunar boundary summary should fail validation");

        assert!(matches!(
            error,
            ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        ));
    }

    #[test]
    fn reference_snapshot_high_curvature_summary_validation_rejects_drift() {
        let mut summary = reference_snapshot_high_curvature_summary()
            .expect("reference high-curvature summary should exist");
        summary.body_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted high-curvature summary should fail validation");

        assert!(matches!(
            error,
            ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                field: "body_count"
            }
        ));
    }

    #[test]
    fn production_generation_snapshot_summary_reports_the_boundary_overlay() {
        let summary = production_generation_snapshot_summary()
            .expect("production-generation snapshot summary should exist");
        summary
            .validate()
            .expect("production-generation snapshot summary should validate");
        assert_eq!(summary.row_count, 195);
        assert_eq!(summary.body_count, 15);
        assert_eq!(summary.bodies, reference_bodies());
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.boundary_row_count, 34);
        assert_eq!(summary.boundary_body_count, 10);
        assert_eq!(
            summary.boundary_bodies,
            &[
                CelestialBody::Mars,
                CelestialBody::Jupiter,
                CelestialBody::Mercury,
                CelestialBody::Venus,
                CelestialBody::Saturn,
                CelestialBody::Uranus,
                CelestialBody::Neptune,
                CelestialBody::Sun,
                CelestialBody::Moon,
                CelestialBody::Pluto,
            ]
        );
        assert_eq!(summary.boundary_epoch_count, 8);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_360_233.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.boundary_earliest_epoch.julian_day.days(),
            2_400_000.0
        );
        assert_eq!(summary.boundary_latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "Production generation coverage: 195 rows across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; boundary overlay (Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000): 34 rows across 10 bodies and 8 epochs (JD 2400000.0 (TDB)..JD 2634167.0 (TDB)); boundary bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto",
                format_bodies(reference_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_snapshot_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            production_generation_source_summary_for_report(),
            format!(
                "Production generation source: {}; {}",
                reference_snapshot_source_summary_for_report(),
                production_generation_boundary_source_summary_for_report()
            )
        );
    }

    #[test]
    fn production_generation_snapshot_window_summary_reports_the_source_windows() {
        let summary = production_generation_snapshot_window_summary()
            .expect("production-generation source window summary should exist");
        summary
            .validate()
            .expect("production-generation source window summary should validate");
        assert_eq!(summary.sample_count, 195);
        assert_eq!(summary.sample_bodies.len(), 15);
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(summary.sample_bodies, reference_bodies());
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_360_233.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.windows[0].body, CelestialBody::Sun);
        assert!(summary.windows[0].sample_count >= 8);
        assert!(summary.windows[0].summary_line().starts_with("Sun: "));
        assert!(summary.summary_line().starts_with(
            "Production generation source windows: 195 source-backed samples across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); windows: "
        ));
        assert!(summary.summary_line().contains("Mars:"));
        assert!(summary.summary_line().contains("Jupiter:"));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_snapshot_window_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn production_generation_snapshot_window_summary_validation_rejects_body_order_drift() {
        let mut summary = production_generation_snapshot_window_summary()
            .expect("production-generation source window summary should exist");
        summary.sample_bodies.swap(0, 1);
        let error = summary
            .validate()
            .expect_err("body order drift should be rejected");
        assert!(matches!(
            error,
            ProductionGenerationSnapshotWindowSummaryValidationError::BodyOrderMismatch { .. }
        ));
    }

    #[test]
    fn production_generation_snapshot_window_summary_validation_rejects_derived_summary_drift() {
        let mut summary = production_generation_snapshot_window_summary()
            .expect("production-generation source window summary should exist");
        summary.sample_count += 1;
        let error = summary
            .validate()
            .expect_err("derived summary drift should be rejected");
        assert_eq!(
            error,
            ProductionGenerationSnapshotWindowSummaryValidationError::DerivedSummaryMismatch
        );
    }

    #[test]
    fn production_generation_snapshot_body_class_coverage_summary_reports_the_split() {
        let summary = production_generation_snapshot_body_class_coverage_summary()
            .expect("production-generation body-class coverage summary should exist");
        summary
            .validate()
            .expect("production-generation body-class coverage summary should validate");
        assert_eq!(summary.row_count, 195);
        assert_eq!(summary.major_bodies.len(), 10);
        assert_eq!(summary.asteroid_bodies.len(), 5);
        assert!(summary
            .summary_line()
            .starts_with("Production generation body-class coverage: major bodies: "));
        assert!(summary.summary_line().contains("selected asteroids: "));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_snapshot_body_class_coverage_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn production_generation_boundary_summary_reports_the_overlay() {
        let summary = production_generation_boundary_summary()
            .expect("production-generation boundary summary should exist");
        summary
            .validate()
            .expect("production-generation boundary summary should validate");
        assert_eq!(summary.row_count, 34);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.bodies, production_generation_boundary_body_list());
        assert_eq!(summary.epoch_count, 8);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_400_000.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            "Production generation boundary overlay: 34 rows across 10 bodies and 8 epochs (JD 2400000.0 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn production_generation_boundary_source_summary_reports_the_overlay_provenance() {
        let boundary_summary = production_generation_boundary_source_summary();
        let holdout_summary = independent_holdout_source_summary();
        boundary_summary
            .validate()
            .expect("production-generation boundary source summary should validate");
        assert_eq!(boundary_summary, holdout_summary);
        assert_eq!(
            format_production_generation_boundary_source_summary(&boundary_summary),
            "Production generation boundary overlay source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000.; columns=epoch_jd, body, x_km, y_km, z_km"
        );
        assert_eq!(
            production_generation_boundary_source_summary_for_report(),
            format_production_generation_boundary_source_summary(&boundary_summary)
        );
    }

    #[test]
    fn production_generation_boundary_window_summary_reports_the_overlay_windows() {
        let summary = production_generation_boundary_window_summary()
            .expect("production-generation boundary window summary should exist");
        assert_eq!(summary.sample_count, 34);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(
            summary.sample_bodies,
            production_generation_boundary_body_list().to_vec()
        );
        assert_eq!(summary.epoch_count, 8);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_400_000.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.windows[0].body, CelestialBody::Mars);
        assert_eq!(summary.windows[0].sample_count, 7);
        assert_eq!(summary.windows[0].epoch_count, 7);
        assert_eq!(
            summary.windows[0].summary_line(),
            format!(
                "Mars: 7 samples across 7 epochs at {}..{}",
                format_instant(summary.windows[0].earliest_epoch),
                format_instant(summary.windows[0].latest_epoch)
            )
        );
        assert!(summary.summary_line().starts_with("Production generation boundary windows: 34 source-backed samples across 10 bodies and 8 epochs (JD 2400000.0 (TDB)..JD 2634167.0 (TDB)); windows: "));
        assert!(summary.summary_line().contains("Mars: 7 samples across 7 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Jupiter: 6 samples across 6 epochs at JD 2400000.0 (TDB)..JD 2500000.0 (TDB)"));
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_boundary_window_summary_for_report(),
            summary.summary_line()
        );

        let mut drifted = summary.clone();
        drifted.sample_count += 1;
        assert!(drifted.validated_summary_line().is_err());
    }

    #[test]
    fn production_generation_boundary_body_class_coverage_summary_reports_the_overlay_body_classes()
    {
        let summary = production_generation_boundary_body_class_coverage_summary()
            .expect("production-generation boundary body-class coverage summary should exist");
        summary
            .validate()
            .expect("production-generation boundary body-class coverage summary should validate");
        assert_eq!(summary.row_count, 34);
        assert_eq!(summary.major_body_row_count, 34);
        assert_eq!(summary.major_bodies.len(), 10);
        assert_eq!(
            summary.major_bodies,
            production_generation_boundary_body_list().to_vec()
        );
        assert_eq!(summary.major_epoch_count, 8);
        assert_eq!(summary.major_windows.len(), 10);
        assert_eq!(summary.major_windows[0].body, CelestialBody::Mars);
        assert_eq!(summary.asteroid_row_count, 0);
        assert!(summary.asteroid_bodies.is_empty());
        assert_eq!(summary.asteroid_epoch_count, 0);
        assert!(summary.asteroid_windows.is_empty());
        assert!(summary.summary_line().starts_with(
            "Production generation boundary body-class coverage: major bodies: 34 rows across 10 bodies and 8 epochs; major windows: "
        ));
        assert!(summary
            .summary_line()
            .contains(&summary.major_windows[0].summary_line()));
        assert!(summary
            .summary_line()
            .contains(&summary.major_windows[2].summary_line()));
        assert!(summary.summary_line().contains(
            "selected asteroids: 0 rows across 0 bodies and 0 epochs; asteroid windows: "
        ));
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_boundary_body_class_coverage_summary_for_report(),
            summary.summary_line()
        );

        let mut drifted = summary.clone();
        drifted.row_count += 1;
        assert!(drifted.validated_summary_line().is_err());
    }

    #[test]
    fn production_generation_snapshot_requests_preserve_the_boundary_overlay() {
        let requests = production_generation_snapshot_requests(CoordinateFrame::Ecliptic)
            .expect("production-generation snapshot requests should exist");
        let entries = production_generation_snapshot_entries()
            .expect("production-generation snapshot entries should exist");
        let boundary_entries = production_generation_boundary_entries()
            .expect("production-generation boundary entries should exist");
        let boundary_requests =
            production_generation_boundary_requests(CoordinateFrame::Equatorial)
                .expect("production-generation boundary requests should exist");

        assert_eq!(requests.len(), entries.len());
        for (request, entry) in requests.iter().zip(entries.iter()) {
            assert_eq!(request.body, entry.body);
            assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }
        let reference_entries = reference_snapshot();
        let boundary_only_entries: Vec<_> = boundary_entries
            .iter()
            .filter(|entry| {
                !reference_entries
                    .iter()
                    .any(|reference| reference.body == entry.body && reference.epoch == entry.epoch)
            })
            .cloned()
            .collect();
        assert_eq!(
            &entries[entries.len() - boundary_only_entries.len()..],
            boundary_only_entries.as_slice()
        );
        assert_eq!(boundary_requests.len(), boundary_entries.len());
        for (request, entry) in boundary_requests.iter().zip(boundary_entries.iter()) {
            assert_eq!(request.body, entry.body);
            assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
            assert_eq!(request.frame, CoordinateFrame::Equatorial);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }
        assert_eq!(
            production_generation_boundary_request_corpus(CoordinateFrame::Equatorial),
            production_generation_boundary_requests(CoordinateFrame::Equatorial)
        );
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_body_count_drift() {
        let mut summary =
            reference_snapshot_summary().expect("reference snapshot summary should exist");
        summary.body_count += 1;

        let error = summary
            .validate()
            .expect_err("body-count drift should be rejected");
        assert_eq!(error.label(), "body count mismatch");
        assert!(error
            .to_string()
            .contains("body count 16 does not match body list length 15"));
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_body_order_drift() {
        let summary =
            reference_snapshot_summary().expect("reference snapshot summary should exist");
        let mut bodies = reference_bodies().to_vec();
        bodies.swap(0, 1);
        let leaked_bodies: &'static [pleiades_backend::CelestialBody] =
            Box::leak(bodies.into_boxed_slice());
        let summary = ReferenceSnapshotSummary {
            bodies: leaked_bodies,
            ..summary
        };

        let error = summary
            .validate()
            .expect_err("body-order drift should be rejected");
        assert_eq!(error.label(), "body order mismatch");
        assert!(error.to_string().contains("index 0"));
        assert!(error
            .to_string()
            .contains(&reference_bodies()[0].to_string()));
        assert!(error
            .to_string()
            .contains(&reference_bodies()[1].to_string()));
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_row_count_drift() {
        let mut summary =
            reference_snapshot_summary().expect("reference snapshot summary should exist");
        summary.row_count += 1;

        assert_eq!(
            summary.validate(),
            Err(ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_epoch_count_drift() {
        let mut summary =
            reference_snapshot_summary().expect("reference snapshot summary should exist");
        summary.epoch_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotSummaryValidationError::EpochCountMismatch {
                    epoch_count: 19,
                    derived_epoch_count: 18,
                }
            )
        ));
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_asteroid_row_count_drift() {
        let mut summary =
            reference_snapshot_summary().expect("reference snapshot summary should exist");
        summary.asteroid_row_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                    asteroid_row_count: 62,
                    derived_asteroid_row_count: 61,
                }
            )
        ));
    }

    #[test]
    fn reference_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
        let summary = reference_snapshot_equatorial_parity_summary()
            .expect("reference snapshot equatorial parity summary should exist");
        assert_eq!(summary.row_count, 195);
        assert_eq!(summary.body_count, 15);
        assert_eq!(summary.bodies, reference_bodies());
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_360_233.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "JPL reference snapshot equatorial parity: 195 rows across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; mean-obliquity transform against the checked-in ecliptic fixture",
                format_bodies(reference_bodies())
            )
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_snapshot_equatorial_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_equatorial_parity_summary_validation_rejects_body_count_drift() {
        let summary = ReferenceSnapshotEquatorialParitySummary {
            row_count: 2,
            body_count: 3,
            bodies: reference_bodies(),
            epoch_count: 1,
            earliest_epoch: reference_instant(),
            latest_epoch: reference_instant(),
        };

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotEquatorialParitySummaryValidationError::Snapshot(
                    ReferenceSnapshotSummaryValidationError::BodyCountMismatch {
                        body_count: 3,
                        bodies_len: 15,
                    }
                )
            )
        ));
    }

    #[test]
    fn reference_snapshot_batch_parity_summary_validation_rejects_derived_summary_drift() {
        let mut summary = reference_snapshot_batch_parity_summary()
            .expect("reference snapshot batch parity summary should exist");
        summary.snapshot.asteroid_row_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotBatchParitySummaryValidationError::Snapshot(
                    ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                        asteroid_row_count: 62,
                        derived_asteroid_row_count: 61,
                    }
                )
            )
        ));
    }

    #[test]
    fn reference_snapshot_batch_parity_summary_reports_the_expected_coverage() {
        let summary = reference_snapshot_batch_parity_summary()
            .expect("reference snapshot batch parity summary should exist");
        assert_eq!(summary.snapshot.row_count, 195);
        assert_eq!(summary.snapshot.body_count, 15);
        assert_eq!(summary.snapshot.bodies, reference_bodies());
        assert_eq!(summary.snapshot.epoch_count, 18);
        assert_eq!(
            summary.snapshot.earliest_epoch.julian_day.days(),
            2_360_233.5
        );
        assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.ecliptic_request_count, 98);
        assert_eq!(summary.equatorial_request_count, 97);
        assert_eq!(summary.exact_count, 195);
        assert_eq!(summary.interpolated_count, 0);
        assert_eq!(summary.approximate_count, 0);
        assert_eq!(summary.unknown_count, 0);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            summary.summary_line(),
            format!(
                "JPL reference snapshot batch parity: 195 rows across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; frame mix: 98 ecliptic, 97 equatorial; quality counts: Exact=195, Interpolated=0, Approximate=0, Unknown=0; batch/single parity preserved",
                format_bodies(reference_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_snapshot_batch_parity_summary_for_report(),
            summary.summary_line()
        );
        assert!(jpl_snapshot_evidence_summary_for_report().contains(
            "JPL reference snapshot batch parity: 195 rows across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
        assert!(jpl_snapshot_evidence_summary_for_report()
            .contains(&production_generation_snapshot_summary_for_report()));
        assert!(jpl_snapshot_evidence_summary_for_report()
            .contains(&production_generation_boundary_source_summary_for_report()));
        assert!(jpl_snapshot_evidence_summary_for_report()
            .contains(&production_generation_boundary_window_summary_for_report()));
        assert!(jpl_snapshot_evidence_summary_for_report()
            .contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(jpl_snapshot_evidence_summary_for_report()
            .contains(&production_generation_boundary_request_corpus_summary_for_report()));
    }

    #[test]
    fn reference_snapshot_batch_parity_summary_validation_rejects_request_count_mismatches() {
        let mut summary = reference_snapshot_batch_parity_summary()
            .expect("reference snapshot batch parity summary should exist");
        summary.equatorial_request_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(ReferenceSnapshotBatchParitySummaryValidationError::RequestCountMismatch { .. })
        ));
    }

    #[test]
    fn production_generation_snapshot_summary_reports_the_expected_coverage() {
        let summary = production_generation_snapshot_summary()
            .expect("production generation summary should exist");
        assert_eq!(summary.row_count, 195);
        assert_eq!(summary.body_count, 15);
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.boundary_row_count, 34);
        assert_eq!(summary.boundary_body_count, 10);
        assert_eq!(summary.boundary_epoch_count, 8);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            production_generation_snapshot_summary_for_report(),
            summary.summary_line()
        );
        assert!(summary.summary_line().contains("boundary overlay (Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000): 34 rows across 10 bodies and 8 epochs"));
    }

    #[test]
    fn production_generation_boundary_request_corpus_summary_reports_the_expected_coverage() {
        let summary =
            production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
                .expect("production generation boundary request corpus summary should exist");
        assert_eq!(summary.request_count, 34);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 8);
        assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
        assert_eq!(summary.time_scale, TimeScale::Tdb);
        assert_eq!(summary.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(summary.apparentness, Apparentness::Mean);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            production_generation_boundary_request_corpus_summary_for_report(),
            summary.summary_line()
        );
        assert!(summary
            .summary_line()
            .contains("observerless) across 10 bodies and 8 epochs"));
    }

    #[test]
    fn comparison_snapshot_summary_reports_the_expected_coverage() {
        let summary =
            comparison_snapshot_summary().expect("comparison snapshot summary should exist");
        assert_eq!(summary.row_count, 112);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 14);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_360_233.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.bodies.as_slice(), comparison_bodies());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Comparison snapshot coverage: 112 rows across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            comparison_snapshot_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn comparison_snapshot_body_class_coverage_summary_reports_the_expected_windows() {
        let summary = comparison_snapshot_body_class_coverage_summary()
            .expect("comparison snapshot body-class coverage summary should exist");

        assert_eq!(summary.row_count, 112);
        assert_eq!(summary.bodies.as_slice(), comparison_bodies());
        assert_eq!(summary.epoch_count, 14);
        assert_eq!(summary.windows.len(), summary.bodies.len());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            comparison_snapshot_body_class_coverage_summary_for_report(),
            summary.summary_line()
        );
        assert!(summary
            .summary_line()
            .starts_with("Comparison snapshot body-class coverage: 112 rows across 10 bodies and 14 epochs; bodies: "));
        assert!(summary.summary_line().contains("windows: Sun:"));
    }

    #[test]
    fn comparison_snapshot_requests_preserve_row_order_and_tt_frame() {
        let requests = comparison_snapshot_requests(CoordinateFrame::Ecliptic)
            .expect("comparison snapshot requests should exist");
        let entries = comparison_snapshot();

        assert_eq!(requests.len(), entries.len());
        for (request, entry) in requests.iter().zip(entries.iter()) {
            assert_eq!(request.body, entry.body);
            assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
            assert_eq!(request.instant.scale, TimeScale::Tt);
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }
    }

    #[test]
    fn comparison_snapshot_equatorial_parity_requests_remain_the_explicit_alias() {
        assert_eq!(
            comparison_snapshot_equatorial_parity_requests(),
            comparison_snapshot_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            comparison_snapshot_equatorial_request_corpus(),
            comparison_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            comparison_snapshot_equatorial_requests(),
            comparison_snapshot_equatorial_request_corpus()
        );
        assert_eq!(
            comparison_snapshot_equatorial_parity_request_corpus(),
            comparison_snapshot_equatorial_parity_requests()
        );
    }

    #[test]
    fn comparison_snapshot_batch_parity_summary_reports_the_expected_coverage() {
        let summary = comparison_snapshot_batch_parity_summary()
            .expect("comparison snapshot batch parity summary should exist");
        assert_eq!(summary.snapshot.row_count, 112);
        assert_eq!(summary.snapshot.body_count, 10);
        assert_eq!(summary.snapshot.epoch_count, 14);
        assert_eq!(
            summary.snapshot.earliest_epoch.julian_day.days(),
            2_360_233.5
        );
        assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.snapshot.bodies.as_slice(), comparison_bodies());
        assert_eq!(summary.ecliptic_request_count, 56);
        assert_eq!(summary.equatorial_request_count, 56);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            format!(
                "JPL comparison snapshot batch parity: 112 rows across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; frame mix: 56 ecliptic, 56 equatorial; quality counts: Exact=112, Interpolated=0, Approximate=0, Unknown=0; batch/single parity preserved",
                format_bodies(comparison_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            comparison_snapshot_batch_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn comparison_snapshot_batch_parity_summary_validation_rejects_request_count_mismatches() {
        let mut summary = comparison_snapshot_batch_parity_summary()
            .expect("comparison snapshot batch parity summary should exist");
        summary.equatorial_request_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(ComparisonSnapshotBatchParitySummaryValidationError::RequestCountMismatch { .. })
        ));
        assert!(matches!(
            summary.validated_summary_line(),
            Err(ComparisonSnapshotBatchParitySummaryValidationError::RequestCountMismatch { .. })
        ));
    }

    #[test]
    fn comparison_snapshot_batch_parity_requests_preserve_the_mixed_frame_slice() {
        let requests = comparison_snapshot_batch_parity_requests()
            .expect("comparison snapshot batch parity requests should exist");
        let entries = comparison_snapshot();

        assert_eq!(requests.len(), entries.len());
        for (index, (request, entry)) in requests.iter().zip(entries.iter()).enumerate() {
            assert_eq!(request.body, entry.body);
            assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
            assert_eq!(request.instant.scale, TimeScale::Tt);
            assert_eq!(
                request.frame,
                if index % 2 == 0 {
                    CoordinateFrame::Ecliptic
                } else {
                    CoordinateFrame::Equatorial
                }
            );
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }
    }

    #[test]
    fn comparison_snapshot_mixed_time_scale_batch_parity_requests_preserve_the_ecliptic_slice() {
        let requests = comparison_snapshot_mixed_time_scale_batch_parity_requests()
            .expect("comparison snapshot mixed TT/TDB batch parity requests should exist");
        let entries = comparison_snapshot();

        assert_eq!(requests.len(), entries.len());
        for (index, (request, entry)) in requests.iter().zip(entries.iter()).enumerate() {
            assert_eq!(request.body, entry.body);
            assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
            assert_eq!(
                request.instant.scale,
                if index % 2 == 0 {
                    TimeScale::Tt
                } else {
                    TimeScale::Tdb
                }
            );
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }
    }

    #[test]
    fn request_corpus_aliases_preserve_the_current_jpl_batch_shapes() {
        assert_eq!(
            reference_snapshot_request_corpus(CoordinateFrame::Ecliptic),
            reference_snapshot_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            reference_snapshot_ecliptic_request_corpus(),
            reference_snapshot_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            reference_snapshot_ecliptic_requests(),
            reference_snapshot_ecliptic_request_corpus()
        );
        assert_eq!(
            reference_snapshot_request_corpus(CoordinateFrame::Equatorial),
            reference_snapshot_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            reference_snapshot_equatorial_parity_requests(),
            reference_snapshot_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            reference_snapshot_equatorial_batch_parity_requests(),
            reference_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            reference_snapshot_equatorial_batch_parity_request_corpus(),
            reference_snapshot_equatorial_batch_parity_requests()
        );
        assert_eq!(
            reference_snapshot_equatorial_request_corpus(),
            reference_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            reference_snapshot_equatorial_requests(),
            reference_snapshot_equatorial_request_corpus()
        );
        assert_eq!(
            reference_snapshot_equatorial_parity_request_corpus(),
            reference_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            reference_snapshot_batch_parity_request_corpus(),
            reference_snapshot_batch_parity_requests()
        );
        assert_eq!(
            reference_snapshot_mixed_time_scale_batch_parity_request_corpus(),
            reference_snapshot_mixed_time_scale_batch_parity_requests()
        );
        assert_eq!(
            reference_snapshot_mixed_tt_tdb_batch_parity_request_corpus(),
            reference_snapshot_mixed_tt_tdb_batch_parity_requests()
        );
        assert_eq!(
            reference_snapshot_mixed_time_scale_request_corpus(),
            reference_snapshot_mixed_time_scale_batch_parity_requests()
        );
        assert_eq!(
            reference_snapshot_mixed_tt_tdb_request_corpus(),
            reference_snapshot_mixed_tt_tdb_batch_parity_requests()
        );
        assert_eq!(
            production_generation_boundary_request_corpus(CoordinateFrame::Ecliptic),
            production_generation_boundary_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            production_generation_boundary_request_corpus(CoordinateFrame::Equatorial),
            production_generation_boundary_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            comparison_snapshot_request_corpus(CoordinateFrame::Ecliptic),
            comparison_snapshot_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            comparison_snapshot_ecliptic_request_corpus(),
            comparison_snapshot_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            comparison_snapshot_ecliptic_requests(),
            comparison_snapshot_ecliptic_request_corpus()
        );
        assert_eq!(
            comparison_snapshot_request_corpus(CoordinateFrame::Equatorial),
            comparison_snapshot_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            comparison_snapshot_equatorial_batch_parity_requests(),
            comparison_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            comparison_snapshot_equatorial_batch_parity_request_corpus(),
            comparison_snapshot_equatorial_batch_parity_requests()
        );
        assert_eq!(
            comparison_snapshot_batch_parity_request_corpus(),
            comparison_snapshot_batch_parity_requests()
        );
        assert_eq!(
            comparison_snapshot_mixed_time_scale_batch_parity_request_corpus(),
            comparison_snapshot_mixed_time_scale_batch_parity_requests()
        );
        assert_eq!(
            comparison_snapshot_mixed_tt_tdb_batch_parity_request_corpus(),
            comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
        );
        assert_eq!(
            comparison_snapshot_mixed_time_scale_request_corpus(),
            comparison_snapshot_mixed_time_scale_batch_parity_requests()
        );
        assert_eq!(
            comparison_snapshot_mixed_tt_tdb_request_corpus(),
            comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_request_corpus(CoordinateFrame::Ecliptic),
            independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            independent_holdout_snapshot_ecliptic_request_corpus(),
            independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            independent_holdout_snapshot_ecliptic_requests(),
            independent_holdout_snapshot_ecliptic_request_corpus()
        );
        assert_eq!(
            independent_holdout_snapshot_request_corpus(CoordinateFrame::Equatorial),
            independent_holdout_snapshot_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            independent_holdout_snapshot_equatorial_batch_parity_requests(),
            independent_holdout_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_equatorial_batch_parity_request_corpus(),
            independent_holdout_snapshot_equatorial_batch_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_equatorial_request_corpus(),
            independent_holdout_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_equatorial_requests(),
            independent_holdout_snapshot_equatorial_request_corpus()
        );
        assert_eq!(
            independent_holdout_snapshot_equatorial_parity_request_corpus(),
            independent_holdout_snapshot_equatorial_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_batch_parity_request_corpus(),
            independent_holdout_snapshot_batch_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_mixed_time_scale_batch_parity_request_corpus(),
            independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_mixed_tt_tdb_batch_parity_request_corpus(),
            independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_mixed_time_scale_request_corpus(),
            independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
        );
        assert_eq!(
            independent_holdout_snapshot_mixed_tt_tdb_request_corpus(),
            independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
        );
        assert_eq!(
            reference_asteroid_request_corpus(CoordinateFrame::Ecliptic),
            reference_asteroid_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            reference_asteroid_request_corpus(CoordinateFrame::Equatorial),
            reference_asteroid_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            reference_asteroid_ecliptic_request_corpus(),
            reference_asteroid_requests(CoordinateFrame::Ecliptic)
        );
        assert_eq!(
            reference_asteroid_equatorial_request_corpus(),
            reference_asteroid_requests(CoordinateFrame::Equatorial)
        );
        assert_eq!(
            reference_asteroid_batch_parity_request_corpus(),
            reference_asteroid_batch_parity_requests()
        );
    }

    #[test]
    fn comparison_snapshot_summary_validation_rejects_duplicate_bodies() {
        let summary = ComparisonSnapshotSummary {
            row_count: 2,
            body_count: 2,
            bodies: vec![
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Moon,
            ],
            epoch_count: 1,
            earliest_epoch: reference_instant(),
            latest_epoch: reference_instant(),
        };
        assert!(matches!(
            summary.validate(),
            Err(ComparisonSnapshotSummaryValidationError::DuplicateBody {
                first_index: 0,
                second_index: 1,
                body,
            }) if body == "Moon"
        ));
    }

    #[test]
    fn comparison_snapshot_summary_validation_rejects_body_order_mismatch() {
        let mut summary = comparison_snapshot_summary().expect("comparison summary should exist");
        summary.bodies.swap(0, 1);
        let expected = comparison_body_list()[0].to_string();
        let found = comparison_body_list()[1].to_string();

        assert!(matches!(
            summary.validate(),
            Err(ComparisonSnapshotSummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: actual_expected,
                found: actual_found,
            }) if actual_expected == expected && actual_found == found
        ));
        assert!(matches!(
            summary.validated_summary_line(),
            Err(ComparisonSnapshotSummaryValidationError::BodyOrderMismatch {
                index: 0,
                expected: actual_expected,
                found: actual_found,
            }) if actual_expected == expected && actual_found == found
        ));
    }

    #[test]
    fn reference_asteroid_evidence_summary_reports_the_expected_coverage() {
        let summary = reference_asteroid_evidence_summary()
            .expect("reference asteroid evidence summary should exist");
        summary
            .validate()
            .expect("reference asteroid evidence summary should validate");
        assert_eq!(
            summary.summary_line(),
            "Selected asteroid evidence: 5 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_asteroid_evidence_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_asteroid_source_window_summary_reports_the_expanded_coverage() {
        let summary = reference_asteroid_source_window_summary()
            .expect("reference asteroid source window summary should exist");
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(summary.sample_count, 61);
        assert_eq!(summary.epoch_count, 13);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference asteroid source windows: 61 source-backed samples across 5 bodies and 13 epochs (JD 2451545.0 (TDB)..JD 2634167.0 (TDB)); windows: Ceres: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Pallas: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Juno: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Vesta: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 13 samples across 13 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB)"
        );
        assert_eq!(
            summary.summary_line(),
            reference_asteroid_source_window_summary_for_report()
        );
    }

    #[test]
    fn reference_asteroid_source_window_summary_validation_rejects_custom_body_drift() {
        let mut summary = reference_asteroid_source_window_summary()
            .expect("reference asteroid source window summary should exist");
        summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn reference_asteroid_source_window_summary_validation_rejects_window_order_drift() {
        let mut summary = reference_asteroid_source_window_summary()
            .expect("reference asteroid source window summary should exist");
        summary.windows.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_source_evidence_summary_reports_the_expanded_coverage() {
        let summary = selected_asteroid_source_evidence_summary()
            .expect("selected asteroid source evidence summary should exist");
        assert_eq!(
            summary.summary_line(),
            "Selected asteroid source evidence: 61 source-backed samples across 5 bodies and 13 epochs (JD 2451545.0 (TDB)..JD 2634167.0 (TDB)); bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros"
        );
        assert_eq!(
            summary.summary_line(),
            selected_asteroid_source_evidence_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_source_window_summary_reports_the_body_windows() {
        let summary = selected_asteroid_source_window_summary()
            .expect("selected asteroid source window summary should exist");
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(summary.sample_count, 61);
        assert_eq!(summary.epoch_count, 13);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Selected asteroid source windows: 61 source-backed samples across 5 bodies and 13 epochs (JD 2451545.0 (TDB)..JD 2634167.0 (TDB)); windows: Ceres: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Pallas: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Juno: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Vesta: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 13 samples across 13 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB)"
        );
        assert_eq!(
            summary.summary_line(),
            selected_asteroid_source_window_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_boundary_summary_reports_the_boundary_days() {
        let summary = selected_asteroid_boundary_summary()
            .expect("selected asteroid boundary summary should exist");
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.epochs.len(), 2);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Selected asteroid boundary evidence: 10 exact samples across 2 epochs at JD 2451914.5 (TDB)..JD 2451915.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        );
        assert_eq!(
            summary.summary_line(),
            selected_asteroid_boundary_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_terminal_boundary_summary_reports_the_terminal_boundary_day() {
        let summary = selected_asteroid_terminal_boundary_summary()
            .expect("selected asteroid terminal boundary summary should exist");
        assert_eq!(summary.sample_count, 5);
        assert_eq!(summary.sample_bodies, reference_asteroids().to_vec());
        assert_eq!(
            summary.epoch,
            Instant::new(JulianDay::from_days(2_500_000.0), TimeScale::Tdb)
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Reference selected-asteroid terminal boundary evidence: 5 exact samples at JD 2500000.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); 2500-01-01 terminal boundary sample"
        );
        assert_eq!(
            summary.summary_line(),
            selected_asteroid_terminal_boundary_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_terminal_boundary_summary_validation_rejects_epoch_drift() {
        let mut summary = selected_asteroid_terminal_boundary_summary()
            .expect("selected asteroid terminal boundary summary should exist");
        summary.epoch = Instant::new(JulianDay::from_days(2_500_001.0), TimeScale::Tdb);

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidTerminalBoundarySummaryValidationError::EpochMismatch {
                    expected: _,
                    found: _
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_batch_parity_summary_reports_the_expected_coverage() {
        let summary = selected_asteroid_batch_parity_summary()
            .expect("selected asteroid batch parity summary should exist");
        assert_eq!(summary.request_count, 5);
        assert_eq!(summary.sample_bodies, reference_asteroids().to_vec());
        assert_eq!(summary.epoch, reference_asteroid_evidence()[0].epoch);
        assert_eq!(summary.ecliptic_count, 3);
        assert_eq!(summary.equatorial_count, 2);
        assert!(summary.parity_preserved);
        summary
            .validate()
            .expect("selected asteroid batch parity summary should validate");
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            summary.summary_line(),
            "Selected asteroid batch parity: 5 requests across 5 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); frame mix: 3 ecliptic, 2 equatorial; batch/single parity preserved"
        );
        assert_eq!(
            summary.summary_line(),
            selected_asteroid_batch_parity_summary_for_report()
        );
    }

    #[test]
    fn selected_asteroid_source_evidence_summary_validation_rejects_sample_count_drift() {
        let mut summary = selected_asteroid_source_evidence_summary()
            .expect("selected asteroid source evidence summary should exist");
        summary.sample_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_source_evidence_summary_validation_rejects_custom_body_drift() {
        let mut summary = selected_asteroid_source_evidence_summary()
            .expect("selected asteroid source evidence summary should exist");
        summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_source_evidence_summary_validation_rejects_body_order_drift() {
        let mut summary = selected_asteroid_source_evidence_summary()
            .expect("selected asteroid source evidence summary should exist");
        summary.sample_bodies.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_source_window_summary_validation_rejects_sample_count_drift() {
        let mut summary = selected_asteroid_source_window_summary()
            .expect("selected asteroid source window summary should exist");
        summary.sample_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_source_window_summary_validation_rejects_epoch_count_drift() {
        let mut summary = selected_asteroid_source_window_summary()
            .expect("selected asteroid source window summary should exist");
        summary.epoch_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_source_window_summary_validation_rejects_custom_body_drift() {
        let mut summary = selected_asteroid_source_window_summary()
            .expect("selected asteroid source window summary should exist");
        summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_batch_parity_summary_validation_rejects_parity_drift() {
        let mut summary = selected_asteroid_batch_parity_summary()
            .expect("selected asteroid batch parity summary should exist");
        summary.parity_preserved = false;

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidBatchParitySummaryValidationError::ParityPreservedMismatch {
                    expected: true,
                    found: false,
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_boundary_summary_validation_rejects_body_order_drift() {
        let mut summary = selected_asteroid_boundary_summary()
            .expect("selected asteroid boundary summary should exist");
        summary.sample_bodies.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(SelectedAsteroidBoundarySummaryValidationError::BodyOrderMismatch { index: 0, .. })
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_boundary_summary_validation_rejects_epoch_order_drift() {
        let mut summary = selected_asteroid_boundary_summary()
            .expect("selected asteroid boundary summary should exist");
        summary.epochs.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidBoundarySummaryValidationError::EpochOrderMismatch { index: 0, .. }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn selected_asteroid_source_window_summary_validation_rejects_window_order_drift() {
        let mut summary = selected_asteroid_source_window_summary()
            .expect("selected asteroid source window summary should exist");
        summary.windows.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn reference_asteroid_evidence_summary_validation_rejects_body_order_drift() {
        let mut summary = reference_asteroid_evidence_summary()
            .expect("reference asteroid evidence summary should exist");
        summary.sample_bodies.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceAsteroidEvidenceSummaryValidationError::BodyOrderMismatch { index: 0, .. }
            )
        ));
    }

    #[test]
    fn reference_asteroid_evidence_validation_rejects_body_order_drift() {
        let mut evidence = reference_asteroid_evidence().to_vec();
        evidence.swap(0, 1);

        assert!(matches!(
            validate_reference_asteroid_evidence(&evidence),
            Err(ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch { index: 0, .. })
        ));
    }

    #[test]
    fn comparison_snapshot_manifest_parses_the_documented_header_comments() {
        let manifest = comparison_snapshot_manifest();
        let source_summary = comparison_snapshot_source_summary();
        assert_eq!(
            manifest.title.as_deref(),
            Some("JPL Horizons reference snapshot.")
        );
        assert_eq!(
            manifest.source.as_deref(),
            Some("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.")
        );
        assert_eq!(
            manifest.coverage.as_deref(),
            Some("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.")
        );
        assert_eq!(manifest.columns, ["body", "x_km", "y_km", "z_km"]);
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(
            source_summary.summary_line(),
            "Comparison snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km"
        );
        assert_eq!(source_summary.to_string(), source_summary.summary_line());
        assert_eq!(source_summary.validate(), Ok(()));
        assert_eq!(
            source_summary.validated_summary_line(),
            Ok(source_summary.summary_line())
        );
        assert_eq!(
            format_comparison_snapshot_source_summary(&source_summary),
            source_summary.summary_line()
        );
        assert_eq!(
            comparison_snapshot_source_summary_for_report(),
            source_summary.summary_line()
        );
        let source_window_summary = comparison_snapshot_source_window_summary()
            .expect("comparison snapshot source window summary should exist");
        assert_eq!(
            source_window_summary.summary_line(),
            "Comparison snapshot source windows: 112 source-backed samples across 10 bodies and 14 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); windows: Sun: 12 samples across 12 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); Moon: 12 samples across 12 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); Mercury: 12 samples across 12 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); Venus: 12 samples across 12 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); Mars: 14 samples across 14 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); Jupiter: 11 samples across 11 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); Saturn: 10 samples across 10 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); Uranus: 10 samples across 10 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); Neptune: 10 samples across 10 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); Pluto: 9 samples across 9 epochs at JD 2400000.0 (TDB)..JD 2500000.0 (TDB)"
        );
        assert_eq!(
            source_window_summary.to_string(),
            source_window_summary.summary_line()
        );
        assert_eq!(source_window_summary.validate(), Ok(()));
        assert_eq!(
            source_window_summary.validated_summary_line(),
            Ok(source_window_summary.summary_line())
        );
        assert_eq!(
            comparison_snapshot_source_window_summary_for_report(),
            source_window_summary.summary_line()
        );
        assert_eq!(
            format_comparison_snapshot_source_window_summary(&source_window_summary),
            source_window_summary.summary_line()
        );
        assert_eq!(
            format_validated_comparison_snapshot_source_summary_for_report(
                &source_summary,
                manifest,
            ),
            source_summary.summary_line()
        );
        let invalid_manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some(" ".to_string()),
            coverage: Some("coverage".to_string()),
            columns: vec!["body".to_string()],
        };
        assert_eq!(
            format_validated_comparison_snapshot_source_summary_for_report(
                &source_summary,
                &invalid_manifest,
            ),
            "Comparison snapshot source: unavailable (missing source)"
        );
        assert_eq!(
            manifest.summary_line("Comparison snapshot manifest"),
            "Comparison snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km"
        );
        assert_eq!(
            manifest.to_string(),
            "Snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km"
        );
        let comparison_summary = comparison_snapshot_manifest_summary();
        assert_eq!(
            comparison_summary.summary_line(),
            "Comparison snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km"
        );
        assert_eq!(
            comparison_summary.to_string(),
            comparison_summary.summary_line()
        );
        assert_eq!(
            comparison_snapshot_manifest_summary_for_report(),
            comparison_summary.summary_line()
        );
    }

    #[test]
    fn comparison_snapshot_source_summary_validation_reports_blank_fields() {
        let blank_source = ComparisonSnapshotSourceSummary {
            source: " ".to_string(),
            coverage: "coverage".to_string(),
            columns: "body, x_km, y_km, z_km".to_string(),
        };
        assert_eq!(
            blank_source.validate(),
            Err(ComparisonSnapshotSourceSummaryValidationError::BlankSource)
        );

        let blank_coverage = ComparisonSnapshotSourceSummary {
            source: "source".to_string(),
            coverage: "\t".to_string(),
            columns: "body, x_km, y_km, z_km".to_string(),
        };
        assert_eq!(
            blank_coverage.validate(),
            Err(ComparisonSnapshotSourceSummaryValidationError::BlankCoverage)
        );

        let blank_columns = ComparisonSnapshotSourceSummary {
            source: "source".to_string(),
            coverage: "coverage".to_string(),
            columns: "  ".to_string(),
        };
        assert_eq!(
            blank_columns.validate(),
            Err(ComparisonSnapshotSourceSummaryValidationError::BlankColumns)
        );

        let padded_source = ComparisonSnapshotSourceSummary {
            source: " source".to_string(),
            coverage: "coverage".to_string(),
            columns: "body, x_km, y_km, z_km".to_string(),
        };
        assert_eq!(
            padded_source.validate(),
            Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                }
            )
        );

        let multiline_columns = ComparisonSnapshotSourceSummary {
            source: "source".to_string(),
            coverage: "coverage".to_string(),
            columns: "body,\nx_km, y_km, z_km".to_string(),
        };
        assert_eq!(
            multiline_columns.validate(),
            Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "columns",
                }
            )
        );
    }

    #[test]
    fn comparison_snapshot_source_window_summary_reports_the_expected_body_windows() {
        let summary = comparison_snapshot_source_window_summary()
            .expect("comparison snapshot source window summary should exist");
        assert_eq!(summary.sample_count, 112);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.epoch_count, 14);
        assert_eq!(summary.windows.len(), 10);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.summary_line().contains("Comparison snapshot source windows: 112 source-backed samples across 10 bodies and 14 epochs"));
        assert!(summary.summary_line().contains(
            "Mars: 14 samples across 14 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB)"
        ));
        assert!(summary.summary_line().contains(
            "Pluto: 9 samples across 9 epochs at JD 2400000.0 (TDB)..JD 2500000.0 (TDB)"
        ));
        assert_eq!(
            comparison_snapshot_source_window_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            format_comparison_snapshot_source_window_summary(&summary),
            summary.summary_line()
        );
    }

    #[test]
    fn comparison_snapshot_source_window_summary_validation_rejects_drift() {
        let mut summary = comparison_snapshot_source_window_summary()
            .expect("comparison snapshot source window summary should exist");
        summary.sample_count += 1;
        assert_eq!(
            summary.validate(),
            Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count"
                }
            )
        );
        assert_eq!(
            summary.validated_summary_line(),
            Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count"
                }
            )
        );
    }

    #[test]
    fn snapshot_manifest_validation_reports_missing_required_metadata() {
        let manifest = SnapshotManifest {
            title: Some(" ".to_string()),
            source: None,
            coverage: Some("ignored".to_string()),
            columns: vec!["body".to_string(), "".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::MissingTitle)
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some(" ".to_string()),
            coverage: None,
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::MissingSource)
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: None,
            columns: Vec::new(),
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::MissingColumns)
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: Some(" ".to_string()),
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::BlankCoverage)
        );
        assert_eq!(
            SnapshotManifestValidationError::BlankCoverage.to_string(),
            "blank coverage"
        );
        assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body"
        );

        let manifest = SnapshotManifest {
            title: Some(" Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: None,
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "title" })
        );
        assert_eq!(
            SnapshotManifestValidationError::SurroundedByWhitespace { field: "title" }.to_string(),
            "title contains surrounding whitespace"
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some(" Example source".to_string()),
            coverage: None,
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "source" })
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: Some(" Coverage".to_string()),
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "coverage" })
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: None,
            columns: vec!["body".to_string(), "".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::BlankColumn { index: 1 })
        );
        assert_eq!(
            SnapshotManifestValidationError::BlankColumn { index: 1 }.to_string(),
            "blank column at index 1"
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: None,
            columns: vec!["body".to_string(), "body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::DuplicateColumn {
                first_index: 0,
                second_index: 1,
                name: "body".to_string(),
            })
        );
        assert_eq!(
            SnapshotManifestValidationError::DuplicateColumn {
                first_index: 0,
                second_index: 1,
                name: "body".to_string(),
            }
            .to_string(),
            "duplicate column 'body' at index 1 (first seen at index 0)"
        );
    }

    #[test]
    fn parsed_manifest_preserves_blank_coverage_for_validation() {
        let manifest = parse_snapshot_manifest(
            "# Example snapshot.\n# Source: Example source\n# Coverage:   \n# Columns: body\n",
        );

        assert_eq!(manifest.title.as_deref(), Some("Example snapshot."));
        assert_eq!(manifest.source.as_deref(), Some("Example source"));
        assert_eq!(manifest.coverage.as_deref(), Some(""));
        assert_eq!(manifest.columns, ["body"]);
        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::BlankCoverage)
        );
        assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body"
        );
    }

    #[test]
    fn parsed_manifest_rejects_surrounding_whitespace_in_provenance_fields() {
        let manifest = SnapshotManifest {
            title: Some(" Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: None,
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "title" })
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some(" Example source".to_string()),
            coverage: None,
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "source" })
        );

        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: Some(" Coverage".to_string()),
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "coverage" })
        );
    }

    #[test]
    fn parsed_manifest_preserves_blank_columns_for_validation() {
        let manifest = parse_snapshot_manifest(
            "# Example snapshot.\n# Source: Example source\n# Columns: body, , x_km\n",
        );

        assert_eq!(manifest.title.as_deref(), Some("Example snapshot."));
        assert_eq!(manifest.source.as_deref(), Some("Example source"));
        assert_eq!(manifest.columns, ["body", "", "x_km"]);
        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::BlankColumn { index: 1 })
        );
        assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body, , x_km"
        );
    }

    #[test]
    fn parsed_manifest_preserves_duplicate_columns_for_validation() {
        let manifest = parse_snapshot_manifest(
            "# Example snapshot.\n# Source: Example source\n# Columns: body, x_km, body\n",
        );

        assert_eq!(manifest.title.as_deref(), Some("Example snapshot."));
        assert_eq!(manifest.source.as_deref(), Some("Example source"));
        assert_eq!(manifest.columns, ["body", "x_km", "body"]);
        assert_eq!(
            manifest.validate(),
            Err(SnapshotManifestValidationError::DuplicateColumn {
                first_index: 0,
                second_index: 2,
                name: "body".to_string(),
            })
        );
        assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body, x_km, body"
        );
    }

    #[test]
    fn manifest_summary_validated_summary_line_returns_the_rendered_line() {
        let summary = SnapshotManifestSummary {
            label: "Example manifest",
            manifest: SnapshotManifest {
                title: Some("Example snapshot.".to_string()),
                source: Some("Example source".to_string()),
                coverage: Some("Example coverage".to_string()),
                columns: vec!["body".to_string(), "x_km".to_string()],
            },
            source_fallback: "unknown",
            coverage_fallback: "unknown",
        };

        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
    }

    #[test]
    fn manifest_summary_validated_summary_line_rejects_columns_drift() {
        let summary = SnapshotManifestSummary {
            label: "Reference snapshot manifest",
            manifest: SnapshotManifest {
                title: Some("Reference snapshot.".to_string()),
                source: Some("NASA/JPL Horizons API".to_string()),
                coverage: Some("Example coverage".to_string()),
                columns: vec![
                    "body".to_string(),
                    "x_km".to_string(),
                    "y_km".to_string(),
                    "z_km".to_string(),
                ],
            },
            source_fallback: "unknown",
            coverage_fallback: "unknown",
        };

        assert_eq!(
            summary.validated_summary_line_with_expected_columns(&[
                "epoch_jd", "body", "x_km", "y_km", "z_km",
            ]),
            Err(SnapshotManifestSummaryValidationError::ColumnsMismatch {
                expected: "epoch_jd, body, x_km, y_km, z_km".to_string(),
                found: "body, x_km, y_km, z_km".to_string(),
            })
        );
        assert_eq!(
            SnapshotManifestSummaryValidationError::ColumnsMismatch {
                expected: "epoch_jd, body, x_km, y_km, z_km".to_string(),
                found: "body, x_km, y_km, z_km".to_string(),
            }
            .to_string(),
            "column schema mismatch: expected epoch_jd, body, x_km, y_km, z_km but found body, x_km, y_km, z_km"
        );
    }

    #[test]
    fn manifest_summary_for_report_reports_validation_errors() {
        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: Some("".to_string()),
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            format_manifest_summary_for_report("Example manifest", &manifest),
            "Example manifest: unavailable (blank coverage)"
        );
    }

    #[test]
    fn validated_source_summary_for_report_reports_validation_errors() {
        let manifest = SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: None,
            coverage: Some("Example coverage".to_string()),
            columns: vec!["body".to_string()],
        };

        assert_eq!(
            format_validated_source_summary_for_report(
                "Example snapshot source",
                &manifest,
                || "should not render".to_string(),
            ),
            "Example snapshot source: unavailable (missing source)"
        );
    }

    #[test]
    fn reference_asteroid_equatorial_evidence_summary_reports_the_expected_coverage() {
        let summary = reference_asteroid_equatorial_evidence_summary()
            .expect("reference asteroid equatorial evidence summary should exist");
        summary
            .validate()
            .expect("reference asteroid equatorial evidence summary should validate");
        assert_eq!(
            summary.summary_line(),
            "Selected asteroid equatorial evidence: 5 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros) using a mean-obliquity equatorial transform"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_asteroid_equatorial_evidence_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_asteroid_equatorial_evidence_summary_validation_rejects_transform_drift() {
        let mut summary = reference_asteroid_equatorial_evidence_summary()
            .expect("reference asteroid equatorial evidence summary should exist");
        summary.transform_note = "broken transform";

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::TransformNoteMismatch {
                    expected: "mean-obliquity equatorial transform",
                    found: "broken transform",
                }
            )
        ));
    }

    #[test]
    fn reference_asteroid_equatorial_evidence_validation_rejects_transform_drift() {
        let mut evidence = reference_asteroid_equatorial_evidence().to_vec();
        let shifted_right_ascension = pleiades_types::Angle::from_degrees(
            evidence[0].equatorial.right_ascension.degrees() + 0.01,
        );
        evidence[0].equatorial = pleiades_types::EquatorialCoordinates::new(
            shifted_right_ascension,
            evidence[0].equatorial.declination,
            evidence[0].equatorial.distance_au,
        );

        assert!(matches!(
            validate_reference_asteroid_equatorial_evidence(&evidence),
            Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::RightAscensionMismatch {
                    index: 0,
                    ..
                }
            )
        ));
    }

    #[test]
    fn batch_query_preserves_reference_snapshot_order_and_equatorial_values() {
        let backend = JplSnapshotBackend;
        let requests = reference_snapshot_requests(CoordinateFrame::Equatorial)
            .expect("reference snapshot requests should exist");

        let results = backend
            .positions(&requests)
            .expect("batch query should preserve the reference snapshot order");

        assert_eq!(results.len(), requests.len());
        for (entry, result) in reference_snapshot().iter().zip(results.iter()) {
            assert_eq!(result.body, entry.body);
            assert_eq!(result.instant, entry.epoch);
            assert_eq!(result.frame, CoordinateFrame::Equatorial);
            assert_eq!(result.quality, QualityAnnotation::Exact);

            let ecliptic = result
                .ecliptic
                .expect("reference snapshot entries should include ecliptic coordinates");
            assert_eq!(ecliptic, entry.ecliptic());

            let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .expect("equatorial coordinates should be present for equatorial batch requests");
            assert_eq!(equatorial, expected_equatorial);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn batch_query_preserves_mixed_frame_requests_and_values() {
        let backend = JplSnapshotBackend;
        let requests = reference_snapshot_batch_parity_requests()
            .expect("reference snapshot batch parity requests should exist");

        let results = backend
            .positions(&requests)
            .expect("mixed frame batch query should preserve the reference snapshot order");

        assert_eq!(results.len(), requests.len());
        for ((request, result), entry) in requests
            .iter()
            .zip(results.iter())
            .zip(reference_snapshot().iter())
        {
            assert_eq!(result.body, entry.body);
            assert_eq!(result.instant, entry.epoch);
            assert_eq!(result.frame, request.frame);

            let ecliptic = result
                .ecliptic
                .expect("reference snapshot entries should include ecliptic coordinates");
            assert_eq!(ecliptic, entry.ecliptic());

            let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .expect("equatorial coordinates should be present for mixed frame batch requests");
            assert_eq!(equatorial, expected_equatorial);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn reference_snapshot_source_summary_reports_the_expected_provenance() {
        let summary = reference_snapshot_source_summary();

        assert_eq!(
            summary.source,
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
        );
        assert_eq!(
            summary.coverage,
            "major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01."
        );
        assert_eq!(summary.frame_treatment, "geocentric ecliptic J2000");
        assert_eq!(summary.reference_epoch.julian_day.days(), 2_451_545.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "Reference snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01.; geocentric ecliptic J2000; TDB reference epoch {}",
                format_instant(summary.reference_epoch)
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        let mut drifted_summary = summary.clone();
        drifted_summary.reference_epoch = drifted_summary
            .reference_epoch
            .with_time_scale_offset(TimeScale::Tt, 1.0);
        assert_eq!(
            drifted_summary.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::ReferenceEpochMismatch)
        );

        let mut drifted_source = summary.clone();
        drifted_source.source =
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables (drift)."
                .to_string();
        assert_eq!(
            drifted_source.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "source" })
        );

        let mut drifted_coverage = summary.clone();
        drifted_coverage.coverage = "major-body coverage drift".to_string();
        assert_eq!(
            drifted_coverage.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "coverage" }
            )
        );

        let mut drifted_frame_treatment = summary.clone();
        drifted_frame_treatment.frame_treatment = "geocentric ecliptic J2000 drift".to_string();
        assert_eq!(
            drifted_frame_treatment.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "frame_treatment"
                }
            )
        );
        assert_eq!(
            reference_snapshot_source_summary_for_report(),
            summary.summary_line()
        );

        let body_class_summary = reference_snapshot_body_class_coverage_summary()
            .expect("reference snapshot body-class coverage summary should exist");
        assert_eq!(body_class_summary.major_body_row_count, 134);
        assert_eq!(body_class_summary.major_bodies.len(), 10);
        assert_eq!(body_class_summary.major_epoch_count, 17);
        assert_eq!(body_class_summary.major_windows.len(), 10);
        assert_eq!(body_class_summary.asteroid_row_count, 61);
        assert_eq!(body_class_summary.asteroid_bodies.len(), 5);
        assert_eq!(body_class_summary.asteroid_epoch_count, 13);
        assert_eq!(body_class_summary.asteroid_windows.len(), 5);
        assert_eq!(body_class_summary.validate(), Ok(()));
        assert_eq!(
            body_class_summary.validated_summary_line(),
            Ok(body_class_summary.summary_line())
        );
        assert_eq!(
            reference_snapshot_body_class_coverage_summary_for_report(),
            body_class_summary.summary_line()
        );
        assert!(body_class_summary
            .summary_line()
            .contains("Reference snapshot body-class coverage: major bodies: 134 rows across 10 bodies and 17 epochs; major windows: "));
        assert!(body_class_summary.summary_line().contains(
            "selected asteroids: 61 rows across 5 bodies and 13 epochs; asteroid windows: "
        ));

        let window_summary = reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");
        assert_eq!(
            window_summary.windows.len(),
            window_summary.sample_bodies.len()
        );
        assert_eq!(
            window_summary.sample_bodies,
            reference_snapshot()
                .iter()
                .map(|entry| entry.body.clone())
                .fold(Vec::new(), |mut bodies, body| {
                    if !bodies.contains(&body) {
                        bodies.push(body);
                    }
                    bodies
                })
        );
        assert_eq!(window_summary.validate(), Ok(()));
        assert_eq!(
            window_summary.validated_summary_line(),
            Ok(window_summary.summary_line())
        );
        assert_eq!(window_summary.to_string(), window_summary.summary_line());
        assert_eq!(
            reference_snapshot_source_window_summary_for_report(),
            window_summary.summary_line()
        );
        assert!(window_summary
            .summary_line()
            .starts_with("Reference snapshot source windows: "));
        assert!(window_summary.summary_line().contains("Moon:"));
        assert!(window_summary.summary_line().contains("Pluto:"));
    }

    #[test]
    fn reference_snapshot_body_class_coverage_summary_reports_the_expected_body_classes() {
        let summary = reference_snapshot_body_class_coverage_summary()
            .expect("reference snapshot body-class coverage summary should exist");

        assert_eq!(summary.major_body_row_count, 134);
        assert_eq!(summary.major_bodies.len(), 10);
        assert_eq!(
            summary.major_bodies[0],
            pleiades_backend::CelestialBody::Sun
        );
        assert_eq!(
            summary.major_bodies[9],
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.major_epoch_count, 17);
        assert_eq!(summary.asteroid_row_count, 61);
        assert_eq!(summary.asteroid_bodies.len(), 5);
        assert_eq!(
            summary.asteroid_bodies[0],
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.asteroid_bodies[4],
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
        assert_eq!(summary.asteroid_epoch_count, 13);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_snapshot_body_class_coverage_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(summary.to_string(), summary.summary_line());
    }

    #[test]
    fn reference_snapshot_source_window_summary_reports_the_current_boundary_windows() {
        let summary = reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");

        assert_eq!(summary.sample_count, 195);
        assert_eq!(summary.sample_bodies.len(), 15);
        assert_eq!(summary.epoch_count, 18);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            reference_snapshot_source_window_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            concat!(
                "Reference snapshot source windows: 195 source-backed samples across 15 bodies and 18 epochs (JD 2360233.5 (TDB)..JD 2634167.0 (TDB)); windows: ",
                "Sun: 14 samples across 14 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); ",
                "Moon: 14 samples across 14 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); ",
                "Mercury: 14 samples across 14 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); ",
                "Venus: 14 samples across 14 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); ",
                "Mars: 17 samples across 17 epochs at JD 2360233.5 (TDB)..JD 2634167.0 (TDB); ",
                "Jupiter: 14 samples across 14 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); ",
                "Saturn: 12 samples across 12 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); ",
                "Uranus: 12 samples across 12 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); ",
                "Neptune: 12 samples across 12 epochs at JD 2360233.5 (TDB)..JD 2500000.0 (TDB); ",
                "Pluto: 11 samples across 11 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); ",
                "Ceres: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); ",
                "Pallas: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); ",
                "Juno: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); ",
                "Vesta: 12 samples across 12 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); ",
                "asteroid:433-Eros: 13 samples across 13 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB)"
            )
        );
    }

    #[test]
    fn reference_snapshot_body_class_coverage_summary_validation_rejects_row_count_drift() {
        let mut summary = reference_snapshot_body_class_coverage_summary()
            .expect("reference snapshot body-class coverage summary should exist");
        summary.major_body_row_count += 1;

        assert_eq!(
            summary.validate(),
            Err(
                ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_body_row_count",
                }
            )
        );
    }

    #[test]
    fn reference_holdout_overlap_summary_reports_the_current_overlap() {
        let summary = reference_holdout_overlap_summary()
            .expect("reference/hold-out overlap summary should exist");

        assert_eq!(summary.shared_sample_count, 34);
        assert_eq!(summary.shared_epoch_count, 8);
        assert_eq!(summary.shared_bodies.len(), 10);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_holdout_overlap_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            summary.summary_line(),
            format!(
                "Reference/hold-out overlap: 34 shared body-epoch pairs across 10 bodies and 8 epochs; bodies: {}",
                format_bodies(&summary.shared_bodies)
            )
        );
    }

    #[test]
    fn reference_snapshot_and_holdout_corpora_remain_anchored_to_the_checked_in_csvs() {
        struct SnapshotKeys {
            row_count: usize,
            bodies: BTreeSet<String>,
            epochs: BTreeSet<String>,
            pairs: BTreeSet<(String, String)>,
        }

        fn snapshot_keys(contents: &str) -> SnapshotKeys {
            let mut bodies = BTreeSet::new();
            let mut epochs = BTreeSet::new();
            let mut pairs = BTreeSet::new();
            let mut row_count = 0usize;

            for line in contents
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            {
                let mut columns = line.split(',');
                let epoch = columns
                    .next()
                    .expect("snapshot rows should include an epoch")
                    .trim()
                    .to_string();
                let body = columns
                    .next()
                    .expect("snapshot rows should include a body")
                    .trim()
                    .to_string();

                row_count += 1;
                epochs.insert(epoch.clone());
                bodies.insert(body.clone());
                pairs.insert((body, epoch));
            }

            SnapshotKeys {
                row_count,
                bodies,
                epochs,
                pairs,
            }
        }

        let reference = snapshot_keys(include_str!("../data/reference_snapshot.csv"));
        let holdout = snapshot_keys(include_str!("../data/independent_holdout_snapshot.csv"));

        assert_eq!(reference.row_count, 195);
        assert_eq!(reference.row_count, reference.pairs.len());
        assert_eq!(reference.bodies.len(), 15);
        assert_eq!(reference.epochs.len(), 18);
        assert_eq!(holdout.row_count, 34);
        assert_eq!(holdout.row_count, holdout.pairs.len());
        assert_eq!(holdout.bodies.len(), 10);
        assert_eq!(holdout.epochs.len(), 8);

        assert_eq!(
            reference_holdout_overlap_summary().map(|summary| summary.shared_sample_count),
            Some(34)
        );
        assert_eq!(
            reference.pairs.intersection(&holdout.pairs).count(),
            34,
            "reference and hold-out corpora should retain the documented 34 shared body-epoch pairs"
        );
        assert_eq!(reference.bodies.intersection(&holdout.bodies).count(), 10);
        assert_eq!(reference.epochs.intersection(&holdout.epochs).count(), 8);
    }

    #[test]
    fn reference_snapshot_source_window_summary_validation_rejects_window_order_drift() {
        let mut summary = reference_snapshot_source_window_summary()
            .expect("reference snapshot source window summary should exist");
        summary.windows.swap(0, 1);

        assert!(matches!(
            summary.validate(),
            Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn reference_snapshot_source_summary_validation_reports_blank_fields() {
        let blank_source = ReferenceSnapshotSourceSummary {
            source: " ".to_string(),
            coverage: "coverage".to_string(),
            frame_treatment: "geocentric ecliptic J2000".to_string(),
            reference_epoch: reference_instant(),
        };
        assert_eq!(
            blank_source.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankSource)
        );

        let blank_coverage = ReferenceSnapshotSourceSummary {
            source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            coverage: "\n".to_string(),
            frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            reference_epoch: reference_instant(),
        };
        assert_eq!(
            blank_coverage.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankCoverage)
        );

        let padded_coverage = ReferenceSnapshotSourceSummary {
            source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            coverage: " coverage ".to_string(),
            frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            reference_epoch: reference_instant(),
        };
        assert_eq!(
            padded_coverage.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                }
            )
        );

        let multiline_source = ReferenceSnapshotSourceSummary {
            source: "source\nline".to_string(),
            coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
            reference_epoch: reference_instant(),
        };
        assert_eq!(
            multiline_source.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                }
            )
        );

        let blank_frame_treatment = ReferenceSnapshotSourceSummary {
            source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            frame_treatment: "\n".to_string(),
            reference_epoch: reference_instant(),
        };
        assert_eq!(
            blank_frame_treatment.validate(),
            Err(ReferenceSnapshotSourceSummaryValidationError::BlankFrameTreatment)
        );

        let padded_frame_treatment = ReferenceSnapshotSourceSummary {
            source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
            coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
            frame_treatment: " geocentric ecliptic J2000 ".to_string(),
            reference_epoch: reference_instant(),
        };
        assert_eq!(
            padded_frame_treatment.validate(),
            Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "frame_treatment",
                }
            )
        );
    }

    #[test]
    fn independent_holdout_source_summary_reports_the_expected_provenance() {
        let summary = independent_holdout_source_summary();

        assert_eq!(
            summary.source,
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
        );
        assert_eq!(
            summary.coverage,
            "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000."
        );
        assert_eq!(summary.columns, "epoch_jd, body, x_km, y_km, z_km");
        assert_eq!(
            summary.summary_line(),
            "Independent hold-out source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000.; columns=epoch_jd, body, x_km, y_km, z_km"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            independent_holdout_source_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn independent_holdout_source_summary_validation_reports_blank_fields() {
        let blank_source = IndependentHoldoutSourceSummary {
            source: " ".to_string(),
            coverage: "coverage".to_string(),
            columns: "epoch_jd, body, x_km, y_km, z_km".to_string(),
        };
        assert_eq!(
            blank_source.validate(),
            Err(IndependentHoldoutSourceSummaryValidationError::BlankSource)
        );

        let blank_coverage = IndependentHoldoutSourceSummary {
            source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
            coverage: "\t".to_string(),
            columns: INDEPENDENT_HOLDOUT_COLUMNS.to_string(),
        };
        assert_eq!(
            blank_coverage.validate(),
            Err(IndependentHoldoutSourceSummaryValidationError::BlankCoverage)
        );

        let blank_columns = IndependentHoldoutSourceSummary {
            source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
            coverage: INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK.to_string(),
            columns: "  ".to_string(),
        };
        assert_eq!(
            blank_columns.validate(),
            Err(IndependentHoldoutSourceSummaryValidationError::BlankColumns)
        );

        let padded_columns = IndependentHoldoutSourceSummary {
            source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
            coverage: INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK.to_string(),
            columns: " epoch_jd, body, x_km, y_km, z_km ".to_string(),
        };
        assert_eq!(
            padded_columns.validate(),
            Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "columns",
                }
            )
        );

        let multiline_coverage = IndependentHoldoutSourceSummary {
            source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
            coverage: "coverage\nmore".to_string(),
            columns: INDEPENDENT_HOLDOUT_COLUMNS.to_string(),
        };
        assert_eq!(
            multiline_coverage.validate(),
            Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                }
            )
        );

        let mut drifted_summary = independent_holdout_source_summary();
        drifted_summary.source =
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables (drift)."
                .to_string();
        assert_eq!(
            drifted_summary.validate(),
            Err(IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "source" })
        );

        let mut drifted_coverage = independent_holdout_source_summary();
        drifted_coverage.coverage = "hold-out coverage drift".to_string();
        assert_eq!(
            drifted_coverage.validate(),
            Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                    field: "coverage"
                }
            )
        );

        let mut drifted_columns = independent_holdout_source_summary();
        drifted_columns.columns = "body, epoch_jd, x_km, y_km, z_km".to_string();
        assert_eq!(
            drifted_columns.validate(),
            Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "columns" }
            )
        );
    }

    #[test]
    fn independent_holdout_snapshot_summary_reports_the_expected_coverage() {
        let summary = independent_holdout_snapshot_summary()
            .expect("independent hold-out summary should exist");
        assert_eq!(summary.row_count, 34);
        assert_eq!(summary.body_count, 10);
        assert_eq!(
            summary.bodies,
            vec![
                "Mars", "Jupiter", "Mercury", "Venus", "Saturn", "Uranus", "Neptune", "Sun",
                "Moon", "Pluto",
            ]
        );
        assert_eq!(summary.epoch_count, 8);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_400_000.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            summary.summary_line(),
            "Independent hold-out coverage: 34 rows across 10 bodies and 8 epochs (JD 2400000.0 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            independent_holdout_snapshot_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn independent_holdout_snapshot_source_window_summary_reports_the_expected_windows() {
        let summary = independent_holdout_snapshot_source_window_summary()
            .expect("independent hold-out source window summary should exist");
        assert_eq!(summary.sample_count, 34);
        assert_eq!(summary.sample_bodies.len(), 10);
        assert_eq!(summary.sample_bodies, independent_holdout_bodies().to_vec());
        assert_eq!(summary.epoch_count, 8);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_400_000.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.windows.len(), 10);
        assert_eq!(
            summary.windows[0].body,
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(summary.windows[0].sample_count, 7);
        assert_eq!(summary.windows[0].epoch_count, 7);
        assert_eq!(
            summary.windows[0].earliest_epoch.julian_day.days(),
            2_451_545.0
        );
        assert_eq!(
            summary.windows[0].latest_epoch.julian_day.days(),
            2_634_167.0
        );
        assert_eq!(
            summary.windows[9].body,
            pleiades_backend::CelestialBody::Pluto
        );
        assert_eq!(summary.windows[9].sample_count, 2);
        assert_eq!(summary.windows[9].epoch_count, 2);
        assert_eq!(
            summary.windows[9].earliest_epoch.julian_day.days(),
            2_451_545.0
        );
        assert_eq!(
            summary.windows[9].latest_epoch.julian_day.days(),
            2_500_000.0
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            independent_holdout_snapshot_source_window_summary_for_report(),
            summary.summary_line()
        );
        assert!(summary.summary_line().contains(
            "Independent hold-out source windows: 34 source-backed samples across 10 bodies and 8 epochs"
        ));
    }

    #[test]
    fn independent_holdout_snapshot_summary_validation_rejects_duplicate_bodies() {
        let summary = IndependentHoldoutSnapshotSummary {
            row_count: 2,
            body_count: 2,
            bodies: vec!["Mars".to_string(), "Mars".to_string()],
            epoch_count: 1,
            earliest_epoch: reference_instant(),
            latest_epoch: reference_instant(),
        };
        assert!(matches!(
            summary.validate(),
            Err(IndependentHoldoutSnapshotSummaryValidationError::DuplicateBody {
                first_index: 0,
                second_index: 1,
                body,
            }) if body == "Mars"
        ));
    }

    #[test]
    fn independent_holdout_snapshot_summary_validation_rejects_body_order_drift() {
        let summary = independent_holdout_snapshot_summary()
            .expect("independent hold-out summary should exist");
        let mut bodies = summary.bodies.clone();
        bodies.swap(0, 1);
        let summary = IndependentHoldoutSnapshotSummary { bodies, ..summary };

        assert_eq!(
            summary.validate(),
            Err(IndependentHoldoutSnapshotSummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn independent_holdout_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
        let summary = independent_holdout_snapshot_equatorial_parity_summary()
            .expect("independent hold-out equatorial parity summary should exist");
        assert_eq!(summary.row_count, 34);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 8);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_400_000.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            "JPL independent hold-out equatorial parity: 34 rows across 10 bodies and 8 epochs (JD 2400000.0 (TDB)..JD 2634167.0 (TDB)); mean-obliquity transform against the checked-in ecliptic fixture"
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            independent_holdout_snapshot_equatorial_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn independent_holdout_snapshot_equatorial_parity_summary_validation_rejects_row_count_drift() {
        let summary = IndependentHoldoutSnapshotEquatorialParitySummary {
            row_count: 1,
            body_count: 2,
            epoch_count: 1,
            earliest_epoch: reference_instant(),
            latest_epoch: reference_instant(),
        };

        assert!(matches!(
            summary.validate(),
            Err(IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::BodyCountExceedsRowCount {
                body_count: 2,
                row_count: 1,
            })
        ));
    }

    #[test]
    fn independent_holdout_summary_reports_the_expected_envelope() {
        let summary =
            jpl_independent_holdout_summary().expect("independent hold-out summary should exist");
        assert_eq!(summary.sample_count, 34);
        assert_eq!(summary.body_count, 10);
        assert_eq!(
            summary.bodies,
            vec![
                "Mars", "Jupiter", "Mercury", "Venus", "Saturn", "Uranus", "Neptune", "Sun",
                "Moon", "Pluto",
            ]
        );
        assert_eq!(summary.epoch_count, 8);
        assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
        assert!(summary.max_longitude_error_deg.is_finite());
        assert!(summary.mean_longitude_error_deg.is_finite());
        assert!(summary.median_longitude_error_deg.is_finite());
        assert!(summary.percentile_longitude_error_deg.is_finite());
        assert!(summary.rms_longitude_error_deg.is_finite());
        assert!(summary.max_latitude_error_deg.is_finite());
        assert!(summary.mean_latitude_error_deg.is_finite());
        assert!(summary.median_latitude_error_deg.is_finite());
        assert!(summary.percentile_latitude_error_deg.is_finite());
        assert!(summary.rms_latitude_error_deg.is_finite());
        assert!(summary.max_distance_error_au.is_finite());
        assert!(summary.mean_distance_error_au.is_finite());
        assert!(summary.median_distance_error_au.is_finite());
        assert!(summary.percentile_distance_error_au.is_finite());
        assert!(summary.rms_distance_error_au.is_finite());
        assert!(!summary.max_longitude_error_body.is_empty());
        assert!(!summary.max_latitude_error_body.is_empty());
        assert!(!summary.max_distance_error_body.is_empty());

        assert_eq!(summary.to_string(), summary.summary_line());

        let rendered = format_jpl_independent_holdout_summary(&summary);
        assert!(rendered.contains("JPL independent hold-out:"));
        assert!(rendered.contains(
            "34 exact rows across 10 bodies (Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto) and 8 epochs"
        ));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(
            rendered.contains("transparency evidence only, not a production tolerance envelope")
        );
        assert!(rendered
            .contains("independent JPL Horizons rows held out from the main snapshot corpus"));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_longitude_error_body,
            format_instant(summary.max_longitude_error_epoch)
        )));
    }

    #[test]
    fn batch_query_preserves_independent_holdout_order_and_single_query_parity() {
        let backend = JplSnapshotBackend;
        let entries = independent_holdout_snapshot_entries()
            .expect("independent hold-out entries should exist");
        let requests = independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
            .expect("independent hold-out requests should exist");

        let results = backend
            .positions(&requests)
            .expect("batch query should resolve the independent hold-out rows");

        assert_eq!(results.len(), entries.len());
        for ((entry, request), batch_result) in
            entries.iter().zip(requests.iter()).zip(results.iter())
        {
            assert_eq!(batch_result.body, entry.body);
            assert_eq!(batch_result.instant, entry.epoch);
            assert_eq!(batch_result.frame, request.frame);
            assert_eq!(batch_result.zodiac_mode, request.zodiac_mode);
            assert_eq!(batch_result.apparent, request.apparent);
            let single = backend
                .position(request)
                .expect("single query should match the independent hold-out batch path");
            assert_eq!(batch_result, &single);
        }
    }

    #[test]
    fn batch_query_preserves_independent_holdout_order_and_mixed_time_scales() {
        let backend = JplSnapshotBackend;
        let entries = independent_holdout_snapshot_entries()
            .expect("independent hold-out entries should exist");
        let requests = independent_holdout_snapshot_batch_parity_requests()
            .expect("independent hold-out mixed-scale requests should exist");

        let results = backend
            .positions(&requests)
            .expect("mixed-scale batch query should resolve the independent hold-out rows");

        assert_eq!(results.len(), entries.len());
        for ((entry, request), batch_result) in
            entries.iter().zip(requests.iter()).zip(results.iter())
        {
            assert_eq!(batch_result.body, entry.body);
            assert_eq!(batch_result.instant, request.instant);
            assert_eq!(batch_result.frame, request.frame);
            assert_eq!(batch_result.zodiac_mode, request.zodiac_mode);
            assert_eq!(batch_result.apparent, request.apparent);
            assert_eq!(batch_result.instant.scale, request.instant.scale);

            let single = backend.position(request).expect(
                "single query should match the independent hold-out mixed-scale batch path",
            );
            assert_eq!(batch_result, &single);
        }
    }

    #[test]
    fn batch_query_preserves_independent_holdout_order_and_equatorial_values() {
        let backend = JplSnapshotBackend;
        let entries = independent_holdout_snapshot_entries()
            .expect("independent hold-out entries should exist");
        let requests = independent_holdout_snapshot_requests(CoordinateFrame::Equatorial)
            .expect("independent hold-out requests should exist");

        let results = backend
            .positions(&requests)
            .expect("equatorial batch query should resolve the independent hold-out rows");

        assert_eq!(results.len(), entries.len());
        for ((entry, request), batch_result) in
            entries.iter().zip(requests.iter()).zip(results.iter())
        {
            assert_eq!(batch_result.body, entry.body);
            assert_eq!(batch_result.instant, entry.epoch);
            assert_eq!(batch_result.frame, request.frame);
            assert_eq!(batch_result.zodiac_mode, request.zodiac_mode);
            assert_eq!(batch_result.apparent, request.apparent);
            let expected_equatorial = batch_result
                .ecliptic
                .expect("equatorial requests should still populate ecliptic coordinates")
                .to_equatorial(batch_result.instant.mean_obliquity());
            let equatorial = batch_result
                .equatorial
                .expect("equatorial coordinates should be present for the hold-out rows");
            assert_eq!(equatorial, expected_equatorial);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
            let single = backend
                .position(request)
                .expect("single query should match the independent hold-out equatorial batch path");
            assert_eq!(batch_result, &single);
        }
    }

    #[test]
    fn batch_query_preserves_independent_holdout_mixed_scale_order_and_single_query_parity() {
        let summary = independent_holdout_snapshot_batch_parity_summary()
            .expect("independent hold-out batch parity summary should exist");
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.snapshot.row_count, 34);
        assert_eq!(summary.snapshot.body_count, 10);
        assert_eq!(summary.tt_request_count, 17);
        assert_eq!(summary.tdb_request_count, 17);
        assert!(summary.parity_preserved);
        assert_eq!(
            summary.exact_count
                + summary.interpolated_count
                + summary.approximate_count
                + summary.unknown_count,
            summary.snapshot.row_count,
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));

        let rendered = format_independent_holdout_snapshot_batch_parity_summary(&summary);
        assert!(rendered.contains("JPL independent hold-out batch parity:"));
        assert!(rendered.contains(
            "34 requests across 10 bodies (Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto) and 8 epochs"
        ));
        assert!(rendered.contains("TT requests=17, TDB requests=17"));
        assert!(rendered.contains("quality counts:"));
        assert!(rendered.contains("order=preserved, single-query parity=preserved"));
    }

    #[test]
    fn independent_holdout_snapshot_batch_parity_summary_validation_rejects_parity_loss() {
        let mut summary = independent_holdout_snapshot_batch_parity_summary()
            .expect("independent hold-out batch parity summary should exist");
        summary.parity_preserved = false;

        assert!(matches!(
            summary.validate(),
            Err(IndependentHoldoutSnapshotBatchParitySummaryValidationError::ParityNotPreserved)
        ));
    }

    #[test]
    fn independent_holdout_snapshot_batch_parity_summary_validation_rejects_degenerate_time_scale_mix(
    ) {
        let mut summary = independent_holdout_snapshot_batch_parity_summary()
            .expect("independent hold-out batch parity summary should exist");
        summary.tt_request_count = summary.snapshot.row_count;
        summary.tdb_request_count = 0;

        assert!(matches!(
            summary.validate(),
            Err(IndependentHoldoutSnapshotBatchParitySummaryValidationError::TimeScaleMixMissing {
                tt_request_count,
                tdb_request_count,
            }) if tt_request_count == summary.snapshot.row_count && tdb_request_count == 0
        ));
    }

    #[test]
    fn jpl_snapshot_evidence_summary_combines_the_backend_reports() {
        let report = jpl_snapshot_evidence_summary_for_report();
        assert!(report.contains(&reference_snapshot_summary_for_report()));
        assert!(report.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
        assert!(report.contains(&reference_snapshot_equatorial_parity_summary_for_report()));
        assert!(report.contains(&reference_snapshot_source_summary_for_report()));
        assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
        assert!(report.contains(&reference_snapshot_major_body_boundary_summary_for_report()));
        assert!(report.contains(&reference_holdout_overlap_summary_for_report()));
        assert!(report.contains(&reference_snapshot_manifest_summary_for_report()));
        assert!(report.contains(&production_generation_snapshot_summary_for_report()));
        assert!(report.contains(&production_generation_source_summary_for_report()));
        assert!(report.contains(&production_generation_boundary_source_summary_for_report()));
        assert!(report.contains(&production_generation_boundary_window_summary_for_report()));
        assert!(report
            .contains(&production_generation_boundary_body_class_coverage_summary_for_report()));
        assert!(
            report.contains(&production_generation_boundary_request_corpus_summary_for_report())
        );
        assert!(report.contains(&reference_asteroid_evidence_summary_for_report()));
        assert!(report.contains(&reference_asteroid_equatorial_evidence_summary_for_report()));
        assert!(report.contains(&reference_asteroid_source_window_summary_for_report()));
        assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_source_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_source_window_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_manifest_summary_for_report()));
        assert!(report.contains(&independent_holdout_snapshot_summary_for_report()));
        assert!(
            report.contains(&independent_holdout_snapshot_equatorial_parity_summary_for_report())
        );
        assert!(report.contains(&independent_holdout_snapshot_batch_parity_summary_for_report()));
        assert!(report.contains(&independent_holdout_source_summary_for_report()));
        assert!(report.contains(&independent_holdout_snapshot_source_window_summary_for_report()));
        assert!(report.contains(&independent_holdout_manifest_summary_for_report()));
        assert!(report.contains(&jpl_independent_holdout_summary_for_report()));
    }

    #[test]
    fn reference_snapshot_manifest_parses_the_documented_header_comments() {
        let manifest = reference_snapshot_manifest();
        assert_eq!(
            manifest.title.as_deref(),
            Some("JPL Horizons reference snapshot.")
        );
        assert_eq!(
            manifest.source.as_deref(),
            Some("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.")
        );
        assert_eq!(manifest.coverage.as_deref(), Some("major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01."));
        assert_eq!(
            manifest.columns,
            ["epoch_jd", "body", "x_km", "y_km", "z_km"]
        );
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(
            manifest.summary_line("Reference snapshot manifest"),
            "Reference snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01.; columns=epoch_jd, body, x_km, y_km, z_km"
        );
    }

    #[test]
    fn reference_snapshot_manifest_summary_rejects_metadata_drift() {
        let summary = reference_snapshot_manifest_summary();
        let error = summary
            .validate_with_expected_metadata(
                "wrong title",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
                "major bodies sampled at 1749-12-31 for Sun through Neptune, inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; major bodies sampled at 1800-01-03 for Sun through Pluto; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2453000.5, and 2500000; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2451910.5 through 2451918.5, with 2451914.5, 2451915.5, and 2451918.5 boundary coverage, 2003-12-27, 2132-08-31, and 2500-01-01.",
                &["epoch_jd", "body", "x_km", "y_km", "z_km"],
            )
            .expect_err("reference snapshot manifest summary should reject title drift");

        assert!(matches!(
            error,
            SnapshotManifestSummaryValidationError::MetadataMismatch { field: "title", .. }
        ));
    }

    #[test]
    fn snapshot_manifest_header_structure_validation_rejects_comment_block_drift() {
        let duplicate_comment_block = "\
# JPL Horizons reference snapshot.
# Source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.
# Coverage: major bodies sampled at 1749-12-31 for Sun through Neptune
# Columns: epoch_jd,body,x_km,y_km,z_km
# Coverage: duplicate
2451545.0,Sun,1,2,3
";
        assert!(matches!(
            validate_snapshot_manifest_header_structure(
                duplicate_comment_block,
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
                "major bodies sampled at 1749-12-31 for Sun through Neptune",
                &["epoch_jd", "body", "x_km", "y_km", "z_km"],
            ),
            Err(SnapshotManifestHeaderStructureError::CommentCountMismatch {
                expected: 4,
                found: 5,
            })
        ));

        let swapped_comment_block = "\
# JPL Horizons reference snapshot.
# Coverage: major bodies sampled at 1749-12-31 for Sun through Neptune
# Source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.
# Columns: epoch_jd,body,x_km,y_km,z_km
2451545.0,Sun,1,2,3
";
        assert!(matches!(
            validate_snapshot_manifest_header_structure(
                swapped_comment_block,
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
                "major bodies sampled at 1749-12-31 for Sun through Neptune",
                &["epoch_jd", "body", "x_km", "y_km", "z_km"],
            ),
            Err(SnapshotManifestHeaderStructureError::CommentMismatch {
                index: 1,
                field: "source",
                ..
            })
        ));
    }

    #[test]
    fn independent_holdout_snapshot_manifest_parses_the_documented_header_comments() {
        let manifest = independent_holdout_snapshot_manifest();
        assert_eq!(manifest.title.as_deref(), Some("Independent JPL Horizons hold-out snapshot used only for interpolation validation."));
        assert_eq!(
            manifest.source.as_deref(),
            Some("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.")
        );
        assert_eq!(
            manifest.coverage.as_deref(),
            Some("Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000.")
        );
        assert_eq!(
            manifest.columns,
            ["epoch_jd", "body", "x_km", "y_km", "z_km"]
        );
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(
            manifest.summary_line("Independent hold-out manifest"),
            "Independent hold-out manifest: Independent JPL Horizons hold-out snapshot used only for interpolation validation.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2500000, and 2634167, plus Moon at 2451545, 2500000, and 2634167, plus Pluto at 2451545 and 2500000.; columns=epoch_jd, body, x_km, y_km, z_km"
        );
    }

    #[test]
    fn snapshot_manifest_summary_line_uses_provided_defaults() {
        let manifest = SnapshotManifest {
            title: Some("Example manifest.".to_string()),
            ..Default::default()
        };

        assert_eq!(
            manifest.summary_line_with_defaults(
                "Example manifest",
                "example source",
                "example coverage",
            ),
            "Example manifest: Example manifest.; source=example source; coverage=example coverage; columns=none"
        );
        assert_eq!(manifest.source_or("fallback source"), "fallback source");
        assert_eq!(
            manifest.coverage_or("fallback coverage"),
            "fallback coverage"
        );
    }

    #[test]
    fn comparison_snapshot_manifest_summary_uses_the_current_manifest() {
        let summary = comparison_snapshot_manifest_summary();

        assert_eq!(
            summary.validate_with_expected_metadata(
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
                "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
                &["body", "x_km", "y_km", "z_km"],
            ),
            Ok(())
        );
        assert_eq!(
            summary.summary_line(),
            comparison_snapshot_manifest_summary_for_report()
        );
        assert_eq!(summary.to_string(), summary.summary_line());
    }

    #[test]
    fn comparison_snapshot_manifest_summary_validation_rejects_metadata_drift() {
        let mut summary = comparison_snapshot_manifest_summary();
        summary.manifest.coverage = Some("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000, plus drift".to_string());

        assert_eq!(
            summary.validate_with_expected_metadata(
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
                "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
                &["body", "x_km", "y_km", "z_km"],
            ),
            Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "coverage",
                expected: "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.".to_string(),
                found: "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000, plus drift".to_string(),
            })
        );
    }

    #[test]
    fn comparison_snapshot_manifest_summary_validation_rejects_padded_label() {
        let mut summary = comparison_snapshot_manifest_summary();
        summary.label = " Comparison snapshot manifest ";

        assert_eq!(
            summary.validate(),
            Err(SnapshotManifestSummaryValidationError::SurroundedByWhitespace { field: "label" })
        );

        summary.label = "Comparison snapshot manifest\nrelease";
        assert_eq!(
            summary.validate(),
            Err(SnapshotManifestSummaryValidationError::SurroundedByWhitespace { field: "label" })
        );
    }

    #[test]
    fn reference_snapshot_summary_validation_rejects_duplicate_bodies() {
        let summary = ReferenceSnapshotSummary {
            row_count: 2,
            body_count: 2,
            bodies: &[
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Sun,
            ],
            epoch_count: 1,
            asteroid_row_count: 0,
            earliest_epoch: reference_instant(),
            latest_epoch: reference_instant(),
        };

        assert!(matches!(
            summary.validate(),
            Err(ReferenceSnapshotSummaryValidationError::DuplicateBody {
                first_index: 0,
                second_index: 1,
                body,
            }) if body == "Sun"
        ));
    }

    #[test]
    fn parser_reports_malformed_rows_without_panicking() {
        let error = load_snapshot_from_str("2451545.0,Sun,1.0,2.0\n")
            .expect_err("missing columns should be reported");
        assert!(format!("{error}").contains("missing z"));

        let error = load_snapshot_from_str("2451545.0,Comet,1.0,2.0,3.0\n")
            .expect_err("unsupported bodies should be reported");
        assert!(format!("{error}").contains("unsupported body 'Comet'"));

        let error = load_snapshot_from_str("2451545.0,,1.0,2.0,3.0\n")
            .expect_err("blank bodies should be reported");
        assert!(format!("{error}").contains("blank body"));
    }

    #[test]
    fn parser_rejects_duplicate_body_epoch_rows() {
        let error =
            load_snapshot_from_str("2451545.0,Sun,1.0,2.0,3.0\n2451545.0,Sun,4.0,5.0,6.0\n")
                .expect_err("duplicate body/epoch pairs should be reported");
        assert!(format!("{error}").contains("line 2"));
        assert!(format!("{error}").contains("duplicate row for body 'Sun'"));
        assert!(format!("{error}").contains("first seen at line 1"));
        assert!(format!("{error}").contains("JD 2451545.0 (TDB)"));
    }

    #[test]
    fn parser_accepts_custom_catalog_bodies() {
        let snapshot = load_snapshot_from_str("2451545.0,asteroid:433-Eros,-1.0,-2.0,-3.0\n")
            .expect("custom catalog bodies should parse");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(
            snapshot[0].body,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
    }

    #[test]
    fn quadratic_interpolation_matches_a_known_parabola() {
        let a = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(0.0), TimeScale::Tdb),
            x_km: 0.0,
            y_km: 1.0,
            z_km: 2.0,
        };
        let b = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
            x_km: 1.0,
            y_km: 6.0,
            z_km: 5.0,
        };
        let c = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
            x_km: 4.0,
            y_km: 15.0,
            z_km: 10.0,
        };

        let interpolated = SnapshotEntry::interpolate_quadratic(&a, &b, &c, 1.5);
        assert!((interpolated.x_km - 2.25).abs() < 1e-12);
        assert!((interpolated.y_km - 10.0).abs() < 1e-12);
        assert!((interpolated.z_km - 7.25).abs() < 1e-12);
    }

    #[test]
    fn cubic_interpolation_matches_a_known_cubic() {
        let a = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(0.0), TimeScale::Tdb),
            x_km: 0.0,
            y_km: 1.0,
            z_km: 2.0,
        };
        let b = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
            x_km: 1.0,
            y_km: 2.0,
            z_km: 3.0,
        };
        let c = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
            x_km: 8.0,
            y_km: 9.0,
            z_km: 10.0,
        };
        let d = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(3.0), TimeScale::Tdb),
            x_km: 27.0,
            y_km: 28.0,
            z_km: 29.0,
        };

        let interpolated = SnapshotEntry::interpolate_cubic(&a, &b, &c, &d, 1.5);
        assert!((interpolated.x_km - 3.375).abs() < 1e-12);
        assert!((interpolated.y_km - 4.375).abs() < 1e-12);
        assert!((interpolated.z_km - 5.375).abs() < 1e-12);
    }

    #[test]
    fn interpolation_uses_a_cubic_window_when_four_points_are_available() {
        let entries = [
            SnapshotEntry {
                body: pleiades_backend::CelestialBody::Moon,
                epoch: Instant::new(JulianDay::from_days(0.0), TimeScale::Tdb),
                x_km: 0.0,
                y_km: 1.0,
                z_km: 2.0,
            },
            SnapshotEntry {
                body: pleiades_backend::CelestialBody::Moon,
                epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
                x_km: 1.0,
                y_km: 2.0,
                z_km: 3.0,
            },
            SnapshotEntry {
                body: pleiades_backend::CelestialBody::Moon,
                epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
                x_km: 8.0,
                y_km: 9.0,
                z_km: 10.0,
            },
            SnapshotEntry {
                body: pleiades_backend::CelestialBody::Moon,
                epoch: Instant::new(JulianDay::from_days(3.0), TimeScale::Tdb),
                x_km: 27.0,
                y_km: 28.0,
                z_km: 29.0,
            },
        ];

        let interpolated =
            interpolate_fixture_state(&entries, pleiades_backend::CelestialBody::Moon, 1.5)
                .expect("four fixture points should produce an interpolated state");
        assert!((interpolated.x_km - 3.375).abs() < 1e-12);
        assert!((interpolated.y_km - 4.375).abs() < 1e-12);
        assert!((interpolated.z_km - 5.375).abs() < 1e-12);
    }

    #[test]
    fn j2000_sun_position_is_finite() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve");
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should be present")
            .is_finite());
    }

    #[test]
    fn j2000_equatorial_request_is_supported() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Equatorial,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        assert!(backend
            .metadata()
            .supported_frames
            .contains(&CoordinateFrame::Equatorial));

        let result = backend
            .position(&request)
            .expect("equatorial frame request should resolve");
        let ecliptic = result
            .ecliptic
            .expect("equatorial requests should still populate ecliptic coordinates");
        let expected = ecliptic.to_equatorial(request.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .expect("equatorial coordinates should be present");

        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        assert_eq!(equatorial, expected);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }

    #[test]
    fn observer_requests_are_rejected_explicitly() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: reference_instant(),
            observer: Some(pleiades_backend::ObserverLocation::new(
                pleiades_backend::Latitude::from_degrees(51.5),
                pleiades_backend::Longitude::from_degrees(0.0),
                None,
            )),
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let error = backend
            .position(&request)
            .expect_err("reference snapshot should reject topocentric requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    }

    #[test]
    fn apparent_requests_are_rejected_explicitly() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Apparent,
        };

        let error = backend
            .position(&request)
            .expect_err("reference snapshot should reject apparent-place requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    }

    #[test]
    fn batch_query_rejects_unsupported_time_scales_explicitly() {
        let backend = JplSnapshotBackend;
        let requests = vec![EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: Instant::new(JulianDay::from_days(REFERENCE_EPOCH_JD), TimeScale::Utc),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        }];

        let error = backend
            .positions(&requests)
            .expect_err("reference snapshot should reject unsupported batch time-scale requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    }

    #[test]
    fn batch_query_rejects_observer_requests_explicitly() {
        let backend = JplSnapshotBackend;
        let requests = vec![EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: reference_instant(),
            observer: Some(pleiades_backend::ObserverLocation::new(
                pleiades_backend::Latitude::from_degrees(51.5),
                pleiades_backend::Longitude::from_degrees(0.0),
                None,
            )),
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        }];

        let error = backend
            .positions(&requests)
            .expect_err("reference snapshot should reject topocentric batch requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    }

    #[test]
    fn batch_query_rejects_apparent_requests_explicitly() {
        let backend = JplSnapshotBackend;
        let requests = vec![EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Apparent,
        }];

        let error = backend
            .positions(&requests)
            .expect_err("reference snapshot should reject apparent batch requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    }

    #[test]
    fn reference_snapshot_mixed_time_scale_batch_parity_requests_preserve_the_ecliptic_slice() {
        let requests = reference_snapshot_mixed_time_scale_batch_parity_requests()
            .expect("reference snapshot mixed TT/TDB batch parity requests should exist");
        let entries = reference_snapshot();

        assert_eq!(requests.len(), entries.len());
        for (index, (request, entry)) in requests.iter().zip(entries.iter()).enumerate() {
            assert_eq!(request.body, entry.body);
            assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
            assert_eq!(
                request.instant.scale,
                if index % 2 == 0 {
                    TimeScale::Tt
                } else {
                    TimeScale::Tdb
                }
            );
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
        }
    }

    #[test]
    fn batch_query_preserves_mixed_time_scales_across_the_reference_snapshot() {
        let backend = JplSnapshotBackend;
        let requests = reference_snapshot_mixed_time_scale_batch_parity_requests()
            .expect("reference snapshot mixed TT/TDB batch parity requests should exist");

        let results = backend
            .positions(&requests)
            .expect("reference snapshot should preserve mixed TT/TDB batch requests");

        assert_eq!(results.len(), requests.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant.scale, request.instant.scale);
            let single = backend
                .position(request)
                .expect("single mixed-scale query should match the batch result");
            assert_eq!(single.body, result.body);
            assert_eq!(single.instant.scale, request.instant.scale);
            assert_eq!(single.quality, result.quality);

            let ecliptic = result
                .ecliptic
                .expect("reference snapshot should include ecliptic coordinates");
            let single_ecliptic = single
                .ecliptic
                .expect("single-query reference snapshot should include ecliptic coordinates");
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
    fn snapshot_data_matches_the_known_j2000_sun_longitude() {
        let entry = reference_snapshot()
            .iter()
            .find(|entry| {
                entry.body == pleiades_backend::CelestialBody::Sun
                    && entry.epoch.julian_day.days() == REFERENCE_EPOCH_JD
            })
            .expect("sun entry should exist at J2000");

        let longitude = entry.ecliptic().longitude.degrees();
        assert!((longitude - 280.3778227681435).abs() < 1e-9);
    }

    #[test]
    fn snapshot_backend_resolves_a_later_epoch() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Mars,
            instant: Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tt),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference fixture should resolve at the later epoch");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("reference fixture should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
    }

    #[test]
    fn snapshot_backend_interpolates_between_fixture_epochs() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Mars,
            instant: Instant::new(JulianDay::from_days(2_415_022.0), TimeScale::Tdb),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference fixture should interpolate between Mars samples");
        assert_eq!(result.quality, QualityAnnotation::Interpolated);
        let ecliptic = result
            .ecliptic
            .expect("interpolated fixture should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());
    }

    #[test]
    fn interpolation_quality_samples_are_reportable() {
        let samples = interpolation_quality_samples();
        assert_eq!(samples.len(), 102);
        assert!(samples.iter().all(|sample| {
            let epoch = sample.epoch.julian_day.days();
            let summary_line = sample.summary_line();
            (epoch == 2_378_499.0
                || epoch == 2_400_000.0
                || epoch == REFERENCE_EPOCH_JD
                || epoch == 2_451_910.5
                || epoch == 2_451_911.5
                || epoch == 2_451_912.5
                || epoch == 2_451_913.5
                || epoch == 2_451_914.5
                || epoch == 2_451_916.5
                || epoch == 2_451_918.5
                || epoch == 2_453_000.5
                || epoch == 2_500_000.0
                || epoch == 2_600_000.0)
                && sample.validate().is_ok()
                && summary_line.contains("TDB")
                && summary_line == sample.to_string()
                && sample.bracket_span_days > 0.0
                && sample.longitude_error_deg.is_finite()
                && sample.latitude_error_deg.is_finite()
                && sample.distance_error_au.is_finite()
                && matches!(
                    sample.interpolation_kind,
                    InterpolationQualityKind::Cubic
                        | InterpolationQualityKind::Quadratic
                        | InterpolationQualityKind::Linear
                )
        }));
        assert!(samples
            .iter()
            .any(|sample| sample.interpolation_kind == InterpolationQualityKind::Cubic));
        assert!(!samples
            .iter()
            .any(|sample| sample.interpolation_kind == InterpolationQualityKind::Quadratic));
        assert!(!samples
            .iter()
            .any(|sample| sample.interpolation_kind == InterpolationQualityKind::Linear));
        assert!(samples
            .iter()
            .any(|sample| sample.epoch.julian_day.days() == 2_400_000.0));
        assert!(samples
            .iter()
            .any(|sample| sample.epoch.julian_day.days() == 2_451_910.5));
        assert!(samples
            .iter()
            .any(|sample| sample.epoch.julian_day.days() == 2_451_911.5));
        assert!(samples
            .iter()
            .any(|sample| sample.epoch.julian_day.days() == 2_500_000.0));
        assert!(samples
            .iter()
            .any(|sample| sample.epoch.julian_day.days() == 2_600_000.0));
        assert!(samples
            .iter()
            .any(|sample| sample.body == pleiades_backend::CelestialBody::Mars));
    }

    #[test]
    fn interpolation_quality_sample_validation_rejects_non_tdb_epochs() {
        let mut sample = interpolation_quality_samples()[0].clone();
        sample.epoch = Instant::new(sample.epoch.julian_day, TimeScale::Tt);

        assert!(matches!(
            sample.validate(),
            Err(InterpolationQualitySampleValidationError::NonTdbEpoch {
                found: TimeScale::Tt,
                ..
            })
        ));
    }

    #[test]
    fn batch_query_preserves_interpolation_quality_samples_and_order() {
        let backend = JplSnapshotBackend;
        let samples = interpolation_quality_samples();
        let requests = interpolation_quality_sample_requests()
            .expect("interpolation-quality sample requests should exist");

        assert_eq!(requests.len(), samples.len());
        for (sample, request) in samples.iter().zip(requests.iter()) {
            assert_eq!(request.body, sample.body);
            assert_eq!(request.instant, sample.epoch);
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }

        let results = backend
            .positions(&requests)
            .expect("batch query should resolve the interpolation-quality samples");

        assert_eq!(results.len(), samples.len());
        for (sample, result) in samples.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.instant, sample.epoch);
            assert_eq!(result.frame, CoordinateFrame::Ecliptic);
            assert_eq!(result.apparent, Apparentness::Mean);
            assert_eq!(result.quality, QualityAnnotation::Exact);
            let ecliptic = result
                .ecliptic
                .expect("batch results should include ecliptic coordinates");
            assert!(ecliptic.longitude.degrees().is_finite());
            assert!(ecliptic.latitude.degrees().is_finite());
            assert!(ecliptic
                .distance_au
                .expect("distance should exist")
                .is_finite());
        }
    }

    #[test]
    fn interpolation_quality_sample_request_corpus_remains_the_explicit_alias() {
        assert_eq!(
            interpolation_quality_sample_request_corpus(),
            interpolation_quality_sample_requests()
        );
    }

    #[test]
    fn interpolation_quality_summary_reports_the_worst_case_labels() {
        let summary = jpl_interpolation_quality_summary().expect("summary should exist");
        assert_eq!(summary.sample_count, 102);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 13);
        assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
        assert_eq!(
            summary.cubic_sample_count
                + summary.quadratic_sample_count
                + summary.linear_sample_count,
            summary.sample_count
        );
        assert!(summary.cubic_sample_count > 0);
        assert_eq!(summary.quadratic_sample_count, 0);
        assert_eq!(summary.linear_sample_count, 0);
        assert!(summary.mean_bracket_span_days.is_finite());
        assert!(summary.median_bracket_span_days.is_finite());
        assert!(summary.percentile_bracket_span_days.is_finite());
        assert!(summary.mean_longitude_error_deg.is_finite());
        assert!(summary.median_longitude_error_deg.is_finite());
        assert!(summary.percentile_longitude_error_deg.is_finite());
        assert!(summary.rms_longitude_error_deg.is_finite());
        assert!(summary.mean_latitude_error_deg.is_finite());
        assert!(summary.median_latitude_error_deg.is_finite());
        assert!(summary.percentile_latitude_error_deg.is_finite());
        assert!(summary.rms_latitude_error_deg.is_finite());
        assert!(summary.mean_distance_error_au.is_finite());
        assert!(summary.median_distance_error_au.is_finite());
        assert!(summary.percentile_distance_error_au.is_finite());
        assert!(summary.rms_distance_error_au.is_finite());
        assert!(!summary.max_bracket_span_body.is_empty());
        assert!(!summary.max_longitude_error_body.is_empty());
        assert!(!summary.max_latitude_error_body.is_empty());
        assert!(!summary.max_distance_error_body.is_empty());

        assert_eq!(summary.to_string(), summary.summary_line());

        let rendered = format_jpl_interpolation_quality_summary(&summary);
        assert!(rendered.contains("cubic"));
        assert!(rendered.contains("quadratic"));
        assert!(rendered.contains("linear"));
        assert!(rendered.contains("102 samples across 10 bodies and 13 epochs"));
        assert!(rendered.contains("epoch window"));
        assert!(rendered.contains("mean bracket span="));
        assert!(rendered.contains("median bracket span="));
        assert!(rendered.contains("p95 bracket span="));
        assert!(rendered.contains("mean Δlon="));
        assert!(rendered.contains("median Δlon="));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("rms Δlon="));
        assert!(rendered.contains("mean Δlat="));
        assert!(rendered.contains("median Δlat="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("rms Δlat="));
        assert!(rendered.contains("mean Δdist="));
        assert!(rendered.contains("median Δdist="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered.contains("rms Δdist="));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_bracket_span_body,
            format_instant(summary.max_bracket_span_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_longitude_error_body,
            format_instant(summary.max_longitude_error_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_latitude_error_body,
            format_instant(summary.max_latitude_error_epoch)
        )));
        assert!(rendered.contains(&format!(
            "({} @ {}",
            summary.max_distance_error_body,
            format_instant(summary.max_distance_error_epoch)
        )));
        assert!(
            rendered.contains("transparency evidence only, not a production tolerance envelope")
        );
    }

    #[test]
    fn interpolation_quality_kind_coverage_reports_the_distinct_body_breakdown() {
        let coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        assert_eq!(coverage.sample_count, 102);
        assert_eq!(coverage.body_count, 10);
        assert_eq!(coverage.bodies.len(), coverage.body_count);
        assert!(!coverage.bodies.is_empty());
        assert!(coverage.cubic_body_count > 0);
        assert_eq!(coverage.quadratic_body_count, 0);
        assert_eq!(coverage.linear_body_count, 0);

        assert_eq!(coverage.to_string(), coverage.summary_line());
        assert_eq!(
            coverage.validated_summary_line(),
            Ok(coverage.summary_line())
        );

        let rendered = format_jpl_interpolation_quality_kind_coverage(&coverage);
        assert!(rendered.contains("JPL interpolation quality kind coverage:"));
        assert!(rendered.contains("102 samples across 10 bodies ["));
        assert!(rendered.contains(&coverage.bodies[0]));
        assert!(rendered.contains("cubic bodies"));
        assert!(rendered.contains("quadratic bodies"));
        assert!(rendered.contains("linear bodies"));
        assert_eq!(
            jpl_interpolation_quality_kind_coverage_for_report(),
            coverage.summary_line()
        );
    }

    #[test]
    fn interpolation_quality_summary_for_report_combines_source_summary_summary_and_coverage() {
        let source_summary =
            jpl_interpolation_quality_source_summary().expect("source summary should exist");
        let summary = jpl_interpolation_quality_summary().expect("summary should exist");
        let coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        let rendered = format_jpl_interpolation_quality_summary_for_report();

        assert!(rendered.contains(&source_summary.summary_line()));
        assert!(rendered.contains(&format_jpl_interpolation_quality_summary(&summary)));
        assert!(rendered.contains(&format_jpl_interpolation_quality_kind_coverage(&coverage)));
    }

    #[test]
    fn interpolation_posture_summary_reports_the_release_decision() {
        let summary = jpl_interpolation_posture_summary().expect("summary should exist");
        assert_eq!(summary.source, JPL_INTERPOLATION_POSTURE_SOURCE);
        assert_eq!(summary.detail, JPL_INTERPOLATION_POSTURE_DETAIL);
        assert_eq!(summary.envelope, JPL_INTERPOLATION_POSTURE_ENVELOPE);
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            jpl_interpolation_posture_summary_for_report(),
            summary.summary_line()
        );
        assert!(format_jpl_interpolation_posture_summary(&summary)
            .contains("JPL interpolation posture:"));
        assert!(summary
            .summary_line()
            .contains("transparency evidence only"));
        assert!(summary
            .summary_line()
            .contains("not a production tolerance envelope"));
    }

    #[test]
    fn interpolation_posture_summary_validation_rejects_drift() {
        let mut summary = jpl_interpolation_posture_summary().expect("summary should exist");
        summary.detail = "runtime production tolerance".to_string();
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "detail" })
        );
    }

    #[test]
    fn interpolation_quality_source_summary_reports_the_expected_provenance() {
        let summary =
            jpl_interpolation_quality_source_summary().expect("source summary should exist");

        assert_eq!(summary.source, reference_snapshot_source_summary().source);
        assert_eq!(summary.derivation, JPL_INTERPOLATION_QUALITY_DERIVATION);
        assert_eq!(summary.sample_count, 102);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 13);
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            jpl_interpolation_quality_source_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn interpolation_quality_summary_validated_summary_line_returns_the_rendered_line() {
        let summary = jpl_interpolation_quality_summary().expect("summary should exist");
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    }

    #[test]
    fn interpolation_quality_summary_validated_summary_line_rejects_drift() {
        let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
        summary.mean_longitude_error_deg += 1e-12;
        assert_eq!(
            summary.validated_summary_line(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn interpolation_quality_source_summary_validation_rejects_drift() {
        let mut summary =
            jpl_interpolation_quality_source_summary().expect("source summary should exist");
        summary.epoch_count += 1;
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count"
                }
            )
        );
        assert_eq!(
            JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                field: "epoch_count"
            }
            .to_string(),
            "the JPL interpolation-quality source summary field `epoch_count` is out of sync with the current evidence"
        );
    }

    #[test]
    fn interpolation_quality_kind_coverage_validated_summary_line_rejects_drift() {
        let mut coverage =
            jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        coverage.cubic_body_count += 1;
        assert_eq!(
            coverage.validated_summary_line(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_inconsistent_counts() {
        let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
        summary.sample_count = 0;
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::MissingSamples)
        );

        let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
        summary.cubic_sample_count += 1;
        let kind_count = summary.cubic_sample_count
            + summary.quadratic_sample_count
            + summary.linear_sample_count;
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::InterpolationKindCountMismatch {
                    sample_count: summary.sample_count,
                    kind_count,
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_non_finite_metrics() {
        let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
        summary.max_longitude_error_deg = f64::INFINITY;
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::MetricOutOfRange {
                    field: "max_longitude_error_deg",
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_blank_peak_bodies() {
        let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
        summary.max_latitude_error_body.clear();
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_latitude_error_body",
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_summary_validation_rejects_derived_summary_drift() {
        let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
        summary.mean_longitude_error_deg += 1e-12;
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn interpolation_quality_coverage_validation_rejects_inconsistent_bodies() {
        let mut coverage =
            jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        coverage.body_count += 1;
        assert_eq!(
            coverage.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::BodyCountMismatch {
                    body_count: coverage.body_count,
                    bodies_len: coverage.bodies.len(),
                }
            )
        );
    }

    #[test]
    fn interpolation_quality_coverage_validation_rejects_duplicate_bodies() {
        let mut coverage =
            jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        let duplicate = coverage.bodies[0].clone();
        coverage.bodies[1] = duplicate.clone();
        coverage.body_count = coverage.bodies.len();
        assert_eq!(
            coverage.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DuplicateBody { body: duplicate })
        );
    }

    #[test]
    fn interpolation_quality_coverage_validation_rejects_derived_summary_drift() {
        let mut coverage =
            jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        coverage.cubic_body_count += 1;
        assert_eq!(
            coverage.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn independent_holdout_summary_validation_rejects_inconsistent_ranges() {
        let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
        summary.epoch_count = 0;
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::MissingEpochs)
        );

        let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
        summary.earliest_epoch = Instant::new(JulianDay::from_days(2_600_000.0), TimeScale::Tdb);
        summary.latest_epoch = Instant::new(JulianDay::from_days(2_500_000.0), TimeScale::Tdb);
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: summary.earliest_epoch,
                    latest_epoch: summary.latest_epoch,
                }
            )
        );
    }

    #[test]
    fn independent_holdout_summary_validation_rejects_blank_bodies() {
        let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
        summary.bodies[1].clear();
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::BlankBody { index: 1 })
        );
    }

    #[test]
    fn independent_holdout_summary_validation_rejects_blank_peak_bodies() {
        let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
        summary.max_distance_error_body.clear();
        assert_eq!(
            summary.validate(),
            Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_distance_error_body",
                }
            )
        );
    }

    #[test]
    fn independent_holdout_summary_validation_rejects_derived_summary_drift() {
        let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
        summary.max_distance_error_au += 1e-12;
        assert_eq!(
            summary.validate(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
    }

    #[test]
    fn independent_holdout_summary_validated_summary_line_rejects_drift() {
        let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
        summary.sample_count += 1;
        assert_eq!(
            summary.validated_summary_line(),
            Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
        );
        assert_eq!(
            format_jpl_independent_holdout_summary(&summary),
            "JPL independent hold-out: unavailable (summary no longer matches the derived interpolation evidence)"
        );
    }

    #[test]
    fn frame_treatment_summary_documents_the_shared_mean_obliquity_transform() {
        let summary = frame_treatment_summary_details();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            "checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"
        );
        assert_eq!(frame_treatment_summary(), summary.summary_line());
        assert_eq!(frame_treatment_summary_for_report(), summary.summary_line());
        assert!(summary.summary_line().contains("mean-obliquity transform"));
    }

    #[test]
    fn request_policy_summary_is_displayable() {
        let policy = jpl_snapshot_request_policy();

        assert_eq!(policy.to_string(), policy.summary_line());
        assert_eq!(policy.validated_summary_line(), Ok(policy.summary_line()));
        assert_eq!(
            jpl_snapshot_request_policy_summary_for_report(),
            policy.summary_line()
        );
        assert!(policy
            .summary_line()
            .contains("frames=Ecliptic, Equatorial"));
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn request_policy_summary_validation_rejects_stale_posture() {
        let mut policy = jpl_snapshot_request_policy();
        policy.supports_topocentric_observer = true;

        let error = policy
            .validate()
            .expect_err("drifted JPL request-policy summaries should fail validation");

        assert_eq!(
            error,
            JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supports_topocentric_observer"
            }
        );
        assert_eq!(
            error.to_string(),
            "the JPL snapshot request-policy summary field `supports_topocentric_observer` is out of sync with the current posture"
        );
    }

    #[test]
    fn batch_error_taxonomy_request_corpus_matches_the_control_sample() {
        let requests = jpl_snapshot_batch_error_taxonomy_request_corpus();

        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].body, pleiades_backend::CelestialBody::Ceres);
        assert_eq!(requests[1].body, pleiades_backend::CelestialBody::MeanNode);
        assert_eq!(requests[2].body, pleiades_backend::CelestialBody::Ceres);
        assert_eq!(requests[0].instant, reference_instant());
        assert_eq!(requests[1].instant, reference_instant());
        assert_eq!(
            requests[2].instant,
            Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb)
        );
        assert!(requests.iter().all(|request| request.observer.is_none()));
        assert!(requests
            .iter()
            .all(|request| request.frame == CoordinateFrame::Ecliptic));
        assert!(requests
            .iter()
            .all(|request| request.zodiac_mode == ZodiacMode::Tropical));
        assert!(requests
            .iter()
            .all(|request| request.apparent == Apparentness::Mean));
    }

    #[test]
    fn batch_error_taxonomy_summary_matches_current_backend() {
        let summary = jpl_snapshot_batch_error_taxonomy_summary()
            .expect("the batch taxonomy summary should remain computable");
        assert_eq!(
            summary.summary_line(),
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            jpl_snapshot_batch_error_taxonomy_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(
            summary.supported_request_body,
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.unsupported_request_body,
            pleiades_backend::CelestialBody::MeanNode
        );
        assert_eq!(
            summary.unsupported_error_kind,
            EphemerisErrorKind::UnsupportedBody
        );
        assert_eq!(
            summary.out_of_range_request_body,
            pleiades_backend::CelestialBody::Ceres
        );
        assert_eq!(
            summary.out_of_range_error_kind,
            EphemerisErrorKind::OutOfRangeInstant
        );
    }

    #[test]
    fn batch_error_taxonomy_summary_validation_rejects_drifted_fields() {
        let summary = JplSnapshotBatchErrorTaxonomySummary {
            supported_request_body: pleiades_backend::CelestialBody::Sun,
            unsupported_request_body: pleiades_backend::CelestialBody::MeanNode,
            unsupported_error_kind: EphemerisErrorKind::UnsupportedBody,
            out_of_range_request_body: pleiades_backend::CelestialBody::Ceres,
            out_of_range_error_kind: EphemerisErrorKind::OutOfRangeInstant,
        };
        assert_eq!(
            summary.validate(),
            Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "supported_request_body"
                }
            )
        );
        assert_eq!(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "supported_request_body"
            }
            .to_string(),
            "the JPL batch error-taxonomy summary field `supported_request_body` is out of sync with the current posture"
        );
    }

    #[test]
    fn snapshot_backend_distinguishes_unsupported_body_from_out_of_range() {
        let backend = JplSnapshotBackend;
        let unsupported = EphemerisRequest {
            body: pleiades_backend::CelestialBody::MeanNode,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };
        let error = backend
            .position(&unsupported)
            .expect_err("missing bodies should not be reported as date-range errors");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);

        let out_of_range = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Ceres,
            instant: Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };
        let error = backend
            .position(&out_of_range)
            .expect_err("single-epoch bodies should report out-of-range requests");
        assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
    }

    #[test]
    fn batch_query_distinguishes_unsupported_body_from_out_of_range() {
        let backend = JplSnapshotBackend;
        let supported = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Ceres,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };
        let unsupported = EphemerisRequest {
            body: pleiades_backend::CelestialBody::MeanNode,
            ..supported.clone()
        };
        let unsupported_error = backend
            .positions(&[supported.clone(), unsupported])
            .expect_err("batch queries should preserve unsupported-body failures");
        assert_eq!(unsupported_error.kind, EphemerisErrorKind::UnsupportedBody);

        let out_of_range = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Ceres,
            instant: Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb),
            ..supported
        };
        let out_of_range_error = backend
            .positions(&[out_of_range])
            .expect_err("batch queries should preserve out-of-range failures");
        assert_eq!(
            out_of_range_error.kind,
            EphemerisErrorKind::OutOfRangeInstant
        );
    }

    #[test]
    fn snapshot_backend_resolves_ceres_at_j2000() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Ceres,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the asteroid entry");
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - 184.459642854516).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - 11.838531252961646).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - 2.2568850705531642).abs()
                < 1e-12
        );
    }

    #[test]
    fn snapshot_backend_resolves_mars_at_2600000() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Mars,
            instant: Instant::new(JulianDay::from_days(2_600_000.0), TimeScale::Tdb),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the Mars hold-out epoch");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - 56.24824943387116).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - (-0.18908796740844558)).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - 2.3186132195308553).abs()
                < 1e-12
        );
    }

    #[test]
    fn snapshot_backend_resolves_mars_at_2634167() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Mars,
            instant: Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tdb),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the Mars outer-boundary epoch");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        assert_eq!(result.instant, request.instant);

        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        let entry = reference_snapshot()
            .iter()
            .find(|entry| {
                entry.body == pleiades_backend::CelestialBody::Mars
                    && entry.epoch.julian_day.days() == 2_634_167.0
            })
            .expect("reference snapshot should include the Mars outer-boundary row");
        assert_eq!(ecliptic, entry.ecliptic());
    }

    #[test]
    fn snapshot_backend_resolves_major_bodies_at_1749_boundary() {
        let backend = JplSnapshotBackend;
        let epoch = Instant::new(JulianDay::from_days(2_360_233.5), TimeScale::Tdb);
        let entries = reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch == epoch)
            .collect::<Vec<_>>();

        assert_eq!(entries.len(), 9);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.body.clone())
                .collect::<Vec<_>>(),
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
            ]
        );

        for entry in entries {
            let request = EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            };

            let result = backend
                .position(&request)
                .expect("reference snapshot should resolve the 1749-12-31 boundary row");
            assert_eq!(result.quality, QualityAnnotation::Exact);
            assert_eq!(result.instant, request.instant);
            assert_eq!(result.body, request.body);
            assert_eq!(result.ecliptic, Some(entry.ecliptic()));
        }
    }

    #[test]
    fn snapshot_backend_resolves_major_bodies_at_1800_boundary() {
        let backend = JplSnapshotBackend;
        let epoch = Instant::new(JulianDay::from_days(2_378_498.5), TimeScale::Tdb);
        let entries = reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch == epoch)
            .collect::<Vec<_>>();

        assert_eq!(entries.len(), 10);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.body.clone())
                .collect::<Vec<_>>(),
            vec![
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Pluto,
            ]
        );

        for entry in entries {
            let request = EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            };

            let result = backend
                .position(&request)
                .expect("reference snapshot should resolve the 1800-01-03 boundary row");
            assert_eq!(result.quality, QualityAnnotation::Exact);
            assert_eq!(result.instant, request.instant);
            assert_eq!(result.body, request.body);
            assert_eq!(result.ecliptic, Some(entry.ecliptic()));
        }
    }

    #[test]
    fn snapshot_backend_resolves_named_asteroids_at_j2000() {
        let backend = JplSnapshotBackend;
        let cases = [
            (
                pleiades_backend::CelestialBody::Pallas,
                134.04575066840783,
                -48.351081494304466,
                1.4371532489145409,
            ),
            (
                pleiades_backend::CelestialBody::Juno,
                278.008461932084,
                9.450859010610209,
                4.084400792647673,
            ),
            (
                pleiades_backend::CelestialBody::Vesta,
                245.98418908965346,
                4.251902812654469,
                2.898586893865609,
            ),
            (
                pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
                236.28757472178148,
                -7.734019866618642,
                1.854402724550437,
            ),
        ];

        for (body, expected_longitude_deg, expected_latitude_deg, expected_distance_au) in cases {
            let request = EphemerisRequest {
                body,
                instant: reference_instant(),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            };

            let result = backend
                .position(&request)
                .expect("reference snapshot should resolve the asteroid entry");
            assert_eq!(result.quality, QualityAnnotation::Exact);

            let ecliptic = result
                .ecliptic
                .expect("reference snapshot should include ecliptic coordinates");
            assert!((ecliptic.longitude.degrees() - expected_longitude_deg).abs() < 1e-12);
            assert!((ecliptic.latitude.degrees() - expected_latitude_deg).abs() < 1e-12);
            assert!(
                (ecliptic.distance_au.expect("distance should exist") - expected_distance_au).abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn batch_query_preserves_reference_asteroid_order_and_values() {
        let backend = JplSnapshotBackend;
        let evidence = reference_asteroid_evidence();
        let requests = evidence
            .iter()
            .map(|sample| EphemerisRequest {
                body: sample.body.clone(),
                instant: sample.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch query should preserve the asteroid reference order");

        assert_eq!(results.len(), evidence.len());
        for (sample, result) in evidence.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.quality, QualityAnnotation::Exact);
            let ecliptic = result
                .ecliptic
                .expect("reference snapshot should include ecliptic coordinates");
            assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < 1e-12);
            assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-12);
            assert!(
                (ecliptic.distance_au.expect("distance should exist") - sample.distance_au).abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn batch_query_preserves_equatorial_frame_and_values() {
        let backend = JplSnapshotBackend;
        let evidence = reference_asteroid_equatorial_evidence();
        let requests = evidence
            .iter()
            .map(|sample| EphemerisRequest {
                body: sample.body.clone(),
                instant: sample.epoch,
                observer: None,
                frame: CoordinateFrame::Equatorial,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch equatorial query should preserve the asteroid reference order");

        assert_eq!(results.len(), evidence.len());
        for (sample, result) in evidence.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.frame, CoordinateFrame::Equatorial);
            let equatorial = result
                .equatorial
                .expect("reference snapshot should include equatorial coordinates");

            assert_eq!(equatorial, sample.equatorial);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn batch_query_preserves_reference_snapshot_order_and_ecliptic_values() {
        let backend = JplSnapshotBackend;
        let evidence = reference_snapshot();
        let requests = evidence
            .iter()
            .map(|sample| EphemerisRequest {
                body: sample.body.clone(),
                instant: sample.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch ecliptic query should preserve the reference snapshot order");

        assert_eq!(results.len(), evidence.len());
        for (sample, result) in evidence.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.instant, sample.epoch);
            assert_eq!(result.frame, CoordinateFrame::Ecliptic);
            assert_eq!(result.quality, QualityAnnotation::Exact);

            let ecliptic = result
                .ecliptic
                .expect("reference snapshot should include ecliptic coordinates");
            let expected = sample.ecliptic();
            assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-12);
            assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-12);
            assert!(
                (ecliptic.distance_au.expect("distance should exist")
                    - expected
                        .distance_au
                        .expect("expected distance should exist"))
                .abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn reference_asteroid_evidence_exposes_exact_j2000_samples() {
        let evidence = reference_asteroid_evidence();
        assert_eq!(evidence.len(), 5);
        assert_eq!(reference_asteroids().len(), evidence.len());
        assert!(evidence.iter().all(|sample| {
            sample.epoch.julian_day.days() == REFERENCE_EPOCH_JD
                && sample.longitude_deg.is_finite()
                && sample.latitude_deg.is_finite()
                && sample.distance_au.is_finite()
        }));
        assert_eq!(evidence[0].body, pleiades_backend::CelestialBody::Ceres);
        assert_eq!(evidence[1].body, pleiades_backend::CelestialBody::Pallas);
        assert_eq!(evidence[2].body, pleiades_backend::CelestialBody::Juno);
        assert_eq!(evidence[3].body, pleiades_backend::CelestialBody::Vesta);
        assert_eq!(
            evidence[4].body,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
        assert!((evidence[0].longitude_deg - 184.459642854516).abs() < 1e-12);
        assert!((evidence[4].distance_au - 1.854402724550437).abs() < 1e-12);
    }

    #[test]
    fn reference_asteroid_requests_preserve_the_exact_j2000_slice() {
        let backend = JplSnapshotBackend;
        let requests = reference_asteroid_requests(CoordinateFrame::Equatorial)
            .expect("selected asteroid requests should exist");
        let results = backend
            .positions(&requests)
            .expect("selected asteroid batch query should preserve the exact J2000 slice");

        assert_eq!(results.len(), requests.len());
        for ((sample, result), request) in reference_asteroid_evidence()
            .iter()
            .zip(results.iter())
            .zip(requests.iter())
        {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.instant, sample.epoch);
            assert_eq!(result.frame, request.frame);
            assert_eq!(result.quality, QualityAnnotation::Exact);

            let ecliptic = result
                .ecliptic
                .expect("selected asteroid batch rows should include ecliptic coordinates");
            assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < 1e-12);
            assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-12);
            assert!(
                (ecliptic.distance_au.expect("distance should exist") - sample.distance_au).abs()
                    < 1e-12
            );

            let equatorial = result
                .equatorial
                .expect("selected asteroid batch rows should include equatorial coordinates");
            let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
            assert_eq!(equatorial, expected_equatorial);
        }
    }

    #[test]
    fn reference_asteroid_batch_parity_requests_preserve_the_selected_j2000_slice() {
        let backend = JplSnapshotBackend;
        let requests = reference_asteroid_batch_parity_requests()
            .expect("selected asteroid batch parity requests should exist");
        let results = backend
            .positions(&requests)
            .expect("mixed-frame selected asteroid batch query should preserve the exact slice");

        assert_eq!(results.len(), requests.len());
        for ((sample, result), request) in reference_asteroid_evidence()
            .iter()
            .zip(results.iter())
            .zip(requests.iter())
        {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.instant, sample.epoch);
            assert_eq!(result.frame, request.frame);
            assert_eq!(result.quality, QualityAnnotation::Exact);

            let ecliptic = result
                .ecliptic
                .expect("selected asteroid batch rows should include ecliptic coordinates");
            let expected = EclipticCoordinates::new(
                Longitude::from_degrees(sample.longitude_deg),
                Latitude::from_degrees(sample.latitude_deg),
                Some(sample.distance_au),
            );
            assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-12);
            assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-12);
            assert!(
                (ecliptic.distance_au.expect("distance should exist")
                    - expected
                        .distance_au
                        .expect("expected distance should exist"))
                .abs()
                    < 1e-12
            );

            let equatorial = result
                .equatorial
                .expect("selected asteroid batch rows should include equatorial coordinates");
            assert_eq!(
                equatorial,
                ecliptic.to_equatorial(result.instant.mean_obliquity())
            );
        }
    }

    #[test]
    fn snapshot_backend_resolves_custom_asteroid_at_j2000() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Custom(CustomBodyId::new(
                "asteroid", "433-Eros",
            )),
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the custom asteroid entry");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - 236.28757472178148).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - (-7.734019866618642)).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - 1.854402724550437).abs()
                < 1e-12
        );
    }
}
