//! production generation summaries.

use core::fmt;
use std::sync::OnceLock;

use pleiades_types::{CoordinateFrame, Instant};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

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
    /// Number of rows contributed by the quarter-day selected-body boundary samples.
    pub quarter_day_row_count: usize,
    /// Number of distinct bodies contributed by the quarter-day selected-body boundary samples.
    pub quarter_day_body_count: usize,
    /// Bodies contributed by the quarter-day selected-body boundary samples in first-seen order.
    pub quarter_day_bodies: &'static [pleiades_backend::CelestialBody],
    /// Number of distinct epochs represented by the quarter-day selected-body boundary samples.
    pub quarter_day_epoch_count: usize,
    /// Earliest epoch represented in the quarter-day selected-body boundary samples.
    pub quarter_day_earliest_epoch: Instant,
    /// Latest epoch represented in the quarter-day selected-body boundary samples.
    pub quarter_day_latest_epoch: Instant,
}

/// Structured validation errors for the production-generation coverage summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationSnapshotSummaryValidationError {
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        /// Distinct-body count carried by the summary.
        body_count: usize,
        /// Number of bodies actually listed in the summary.
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        /// Index of the first occurrence in the compared pair.
        first_index: usize,
        /// Index of the second (duplicate) occurrence in the compared pair.
        second_index: usize,
        /// Body designation involved in the mismatch.
        body: String,
    },
    /// The summary body order drifted from the checked-in production corpus.
    BodyOrderMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Value expected from the current evidence slice.
        expected: String,
        /// Value recorded in the summary under validation.
        found: String,
    },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        /// Earliest epoch carried by the summary.
        earliest_epoch: Instant,
        /// Latest epoch carried by the summary.
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
}

/// Returns the production-generation coverage summary used in release-facing reporting.
pub fn production_generation_snapshot_summary() -> Option<ProductionGenerationSnapshotSummary> {
    static SUMMARY: OnceLock<Option<ProductionGenerationSnapshotSummary>> = OnceLock::new();
    *SUMMARY.get_or_init(|| {
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

        let mut quarter_day_earliest_epoch = None;
        let mut quarter_day_latest_epoch = None;
        let mut quarter_day_epochs = BTreeSet::new();
        let mut quarter_day_bodies = Vec::new();
        let mut quarter_day_row_count = 0usize;
        for entry in boundary_entries {
            let epoch_days = entry.epoch.julian_day.days();
            if PRODUCTION_GENERATION_QUARTER_DAY_EPOCHS.contains(&epoch_days) {
                quarter_day_row_count += 1;
                quarter_day_epochs.insert(epoch_days.to_bits());
                quarter_day_earliest_epoch.get_or_insert(entry.epoch);
                quarter_day_latest_epoch = Some(entry.epoch);
                if !quarter_day_bodies.contains(&entry.body) {
                    quarter_day_bodies.push(entry.body.clone());
                }
            }
        }
        if quarter_day_row_count == 0
            || quarter_day_epochs.len() != 2
            || quarter_day_bodies.is_empty()
        {
            return None;
        }
        let quarter_day_earliest_epoch = quarter_day_earliest_epoch?;
        let quarter_day_latest_epoch = quarter_day_latest_epoch?;
        let quarter_day_bodies: &'static [pleiades_backend::CelestialBody] =
            Box::leak(quarter_day_bodies.into_boxed_slice());

        Some(ProductionGenerationSnapshotSummary {
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
            quarter_day_row_count,
            quarter_day_body_count: quarter_day_bodies.len(),
            quarter_day_bodies,
            quarter_day_epoch_count: quarter_day_epochs.len(),
            quarter_day_earliest_epoch,
            quarter_day_latest_epoch,
        })
    })
}

/// Deterministic revision metadata for the checked-in CSV fixtures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionGenerationSourceRevisionSummary {
    /// Checksum of the checked-in reference snapshot fixture.
    pub reference_snapshot_checksum: u64,
    /// Checksum of the checked-in independent hold-out snapshot fixture.
    pub independent_holdout_snapshot_checksum: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Validation errors for a production-generation source-revision summary that drifted from the current evidence.
pub enum ProductionGenerationSourceRevisionSummaryValidationError {
    /// The revision summary no longer matches the current fixture checksums.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ProductionGenerationSourceRevisionSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the production generation source revision summary field `{field}` is out of sync with the current fixture checksums"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationSourceRevisionSummaryValidationError {}

impl ProductionGenerationSourceRevisionSummary {
    /// Returns `Ok(())` when the summary still matches the current fixture checksums.
    pub fn validate(&self) -> Result<(), ProductionGenerationSourceRevisionSummaryValidationError> {
        if self != &production_generation_source_revision_summary() {
            return Err(
                ProductionGenerationSourceRevisionSummaryValidationError::FieldOutOfSync {
                    field: "summary",
                },
            );
        }

        Ok(())
    }
}

/// Combined provenance for the production-generation corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationSourceSummary {
    /// Source summary for the reference snapshot.
    pub reference_summary: ReferenceSnapshotSourceSummary,
    /// Source summary for the independent hold-out boundary overlay.
    pub boundary_summary: IndependentHoldoutSourceSummary,
    /// Source-window summary for the merged production-generation corpus.
    pub source_windows: ProductionGenerationSnapshotWindowSummary,
    /// Deterministic revision metadata for the checked-in CSV fixtures.
    pub source_revision: ProductionGenerationSourceRevisionSummary,
}

impl ProductionGenerationSourceSummary {
    /// Returns `Ok(())` when the source summary still matches the derived corpus evidence.
    pub fn validate(&self) -> Result<(), ProductionGenerationSourceSummaryValidationError> {
        self.reference_summary
            .validate()
            .map_err(ProductionGenerationSourceSummaryValidationError::Reference)?;
        self.boundary_summary
            .validate()
            .map_err(ProductionGenerationSourceSummaryValidationError::Boundary)?;
        self.source_windows
            .validate()
            .map_err(ProductionGenerationSourceSummaryValidationError::SourceWindows)?;
        let expected_source_windows = production_generation_snapshot_window_summary()
            .ok_or(ProductionGenerationSourceSummaryValidationError::SourceWindowsMismatch)?;
        if self.source_windows != expected_source_windows {
            return Err(ProductionGenerationSourceSummaryValidationError::SourceWindowsMismatch);
        }
        if self.source_revision != production_generation_source_revision_summary() {
            return Err(ProductionGenerationSourceSummaryValidationError::SourceRevisionMismatch);
        }
        reference_snapshot_exact_j2000_evidence_summary().ok_or(
            ProductionGenerationSourceSummaryValidationError::ReferenceExactJ2000EvidenceUnavailable,
        )?;

        Ok(())
    }
}

/// Structured validation errors for the production-generation provenance summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationSourceSummaryValidationError {
    /// The reference snapshot source summary drifted from the checked-in evidence.
    Reference(ReferenceSnapshotSourceSummaryValidationError),
    /// The boundary overlay source summary drifted from the checked-in evidence.
    Boundary(IndependentHoldoutSourceSummaryValidationError),
    /// The source-window summary drifted from the checked-in evidence.
    SourceWindows(ProductionGenerationSnapshotWindowSummaryValidationError),
    /// The source-window summary no longer matches the current derived corpus evidence.
    SourceWindowsMismatch,
    /// The source-density floors summary no longer matches the current derived corpus evidence.
    SourceDensityMismatch,
    /// The body-class cadence summary no longer matches the current derived corpus evidence.
    BodyClassCadenceMismatch,
    /// The ecliptic and equatorial boundary-request corpora no longer share the same epoch count.
    BoundaryRequestCorpusEpochCountMismatch {
        /// Number of distinct ecliptic-frame epochs carried by the summary.
        ecliptic_epoch_count: usize,
        /// Number of distinct equatorial-frame epochs carried by the summary.
        equatorial_epoch_count: usize,
    },
    /// The deterministic revision summary drifted from the checked-in fixture contents.
    SourceRevisionMismatch,
    /// The reference snapshot exact J2000 evidence is unavailable.
    ReferenceExactJ2000EvidenceUnavailable,
    /// The rendered summary text drifted from the expected release-facing provenance fragments.
    RenderedSummaryOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ProductionGenerationSourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reference(error) => write!(f, "reference summary validation failed: {error}"),
            Self::Boundary(error) => write!(f, "boundary summary validation failed: {error}"),
            Self::SourceWindows(error) => {
                write!(f, "source window summary validation failed: {error}")
            }
            Self::SourceWindowsMismatch => f.write_str("source windows mismatch"),
            Self::SourceDensityMismatch => f.write_str("source density floors mismatch"),
            Self::BodyClassCadenceMismatch => f.write_str("body-class cadence mismatch"),
            Self::BoundaryRequestCorpusEpochCountMismatch {
                ecliptic_epoch_count,
                equatorial_epoch_count,
            } => write!(
                f,
                "boundary request corpus epoch counts differ: ecliptic={ecliptic_epoch_count}, equatorial={equatorial_epoch_count}"
            ),
            Self::SourceRevisionMismatch => f.write_str("source revision mismatch"),
            Self::ReferenceExactJ2000EvidenceUnavailable => {
                f.write_str("reference snapshot exact J2000 evidence unavailable")
            }
            Self::RenderedSummaryOutOfSync { field } => write!(
                f,
                "rendered production-generation source summary field `{field}` is out of sync"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationSourceSummaryValidationError {}

/// Returns the combined source provenance for the production-generation corpus.
///
/// The production-generation inputs remain the checked-in CSV fixtures that are
/// parsed in pure Rust via `include_str!`, so the regeneration path stays fully
/// deterministic and hashable from repository contents alone.
pub fn production_generation_source_summary() -> ProductionGenerationSourceSummary {
    ProductionGenerationSourceSummary {
        reference_summary: reference_snapshot_source_summary(),
        boundary_summary: production_generation_boundary_source_summary(),
        source_windows: production_generation_snapshot_window_summary()
            .expect("production generation source windows should exist"),
        source_revision: production_generation_source_revision_summary(),
    }
}

/// Compact release-facing contract summary for the production-generation corpus shape.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationCorpusShapeSummary {
    /// Source provenance summary for the merged production-generation corpus.
    pub source_summary: ProductionGenerationSourceSummary,
    /// Boundary request corpus used to validate apparentness and frame posture in ecliptic coordinates.
    pub boundary_request_corpus_ecliptic: ProductionGenerationBoundaryRequestCorpusSummary,
    /// Boundary request corpus used to validate apparentness and frame posture in equatorial coordinates.
    pub boundary_request_corpus_equatorial: ProductionGenerationBoundaryRequestCorpusSummary,
}

/// Structured validation errors for the production-generation corpus shape summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationCorpusShapeSummaryValidationError {
    /// The nested source summary drifted from the checked-in evidence.
    Source(ProductionGenerationSourceSummaryValidationError),
    /// The nested ecliptic boundary request corpus drifted from the checked-in evidence.
    BoundaryRequestCorpusEcliptic(ProductionGenerationBoundaryRequestCorpusSummaryValidationError),
    /// The nested equatorial boundary request corpus drifted from the checked-in evidence.
    BoundaryRequestCorpusEquatorial(
        ProductionGenerationBoundaryRequestCorpusSummaryValidationError,
    ),
    /// A derived field drifted from the current checked-in corpus posture.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ProductionGenerationCorpusShapeSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => write!(f, "source summary validation failed: {error}"),
            Self::BoundaryRequestCorpusEcliptic(error) => {
                write!(
                    f,
                    "ecliptic boundary request corpus validation failed: {error}"
                )
            }
            Self::BoundaryRequestCorpusEquatorial(error) => {
                write!(
                    f,
                    "equatorial boundary request corpus validation failed: {error}"
                )
            }
            Self::FieldOutOfSync { field } => write!(
                f,
                "the production-generation corpus shape field `{field}` is out of sync"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationCorpusShapeSummaryValidationError {}

pub(crate) fn validate_production_generation_boundary_request_corpus_frame_parity(
    boundary_request_corpus_ecliptic: &ProductionGenerationBoundaryRequestCorpusSummary,
    boundary_request_corpus_equatorial: &ProductionGenerationBoundaryRequestCorpusSummary,
) -> Result<(), ProductionGenerationCorpusShapeSummaryValidationError> {
    let parity_fields = [
        (
            "boundary request corpus parity (request_count)",
            boundary_request_corpus_ecliptic.request_count
                == boundary_request_corpus_equatorial.request_count,
        ),
        (
            "boundary request corpus parity (body_count)",
            boundary_request_corpus_ecliptic.body_count
                == boundary_request_corpus_equatorial.body_count,
        ),
        (
            "boundary request corpus parity (bodies)",
            boundary_request_corpus_ecliptic.bodies == boundary_request_corpus_equatorial.bodies,
        ),
        (
            "boundary request corpus parity (epoch_count)",
            boundary_request_corpus_ecliptic.epoch_count
                == boundary_request_corpus_equatorial.epoch_count,
        ),
        (
            "boundary request corpus parity (earliest_epoch)",
            boundary_request_corpus_ecliptic.earliest_epoch
                == boundary_request_corpus_equatorial.earliest_epoch,
        ),
        (
            "boundary request corpus parity (latest_epoch)",
            boundary_request_corpus_ecliptic.latest_epoch
                == boundary_request_corpus_equatorial.latest_epoch,
        ),
        (
            "boundary request corpus parity (time_scale)",
            boundary_request_corpus_ecliptic.time_scale
                == boundary_request_corpus_equatorial.time_scale,
        ),
        (
            "boundary request corpus parity (zodiac_mode)",
            boundary_request_corpus_ecliptic.zodiac_mode
                == boundary_request_corpus_equatorial.zodiac_mode,
        ),
        (
            "boundary request corpus parity (apparentness)",
            boundary_request_corpus_ecliptic.apparentness
                == boundary_request_corpus_equatorial.apparentness,
        ),
    ];

    for (field, is_parity) in parity_fields {
        if !is_parity {
            return Err(
                ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync { field },
            );
        }
    }

    Ok(())
}

impl ProductionGenerationCorpusShapeSummary {
    /// Returns `Ok(())` when the corpus-shape summary still matches the current corpus posture.
    pub fn validate(&self) -> Result<(), ProductionGenerationCorpusShapeSummaryValidationError> {
        self.source_summary
            .validate()
            .map_err(ProductionGenerationCorpusShapeSummaryValidationError::Source)?;
        self.boundary_request_corpus_ecliptic.validate().map_err(
            ProductionGenerationCorpusShapeSummaryValidationError::BoundaryRequestCorpusEcliptic,
        )?;
        self.boundary_request_corpus_equatorial.validate().map_err(
            ProductionGenerationCorpusShapeSummaryValidationError::BoundaryRequestCorpusEquatorial,
        )?;

        let expected_source_summary = production_generation_source_summary();
        if self.source_summary != expected_source_summary {
            return Err(
                ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync {
                    field: "source summary",
                },
            );
        }

        let expected_boundary_request_corpus_ecliptic =
            production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
                .ok_or(
                    ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync {
                        field: "boundary request corpus (ecliptic)",
                    },
                )?;
        if self.boundary_request_corpus_ecliptic != expected_boundary_request_corpus_ecliptic {
            return Err(
                ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync {
                    field: "boundary request corpus (ecliptic)",
                },
            );
        }

        let expected_boundary_request_corpus_equatorial =
            production_generation_boundary_request_corpus_summary(CoordinateFrame::Equatorial)
                .ok_or(
                    ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync {
                        field: "boundary request corpus (equatorial)",
                    },
                )?;
        if self.boundary_request_corpus_equatorial != expected_boundary_request_corpus_equatorial {
            return Err(
                ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync {
                    field: "boundary request corpus (equatorial)",
                },
            );
        }

        validate_production_generation_boundary_request_corpus_frame_parity(
            &self.boundary_request_corpus_ecliptic,
            &self.boundary_request_corpus_equatorial,
        )?;

        Ok(())
    }
}

/// Returns the compact production-generation corpus-shape summary.
pub fn production_generation_corpus_shape_summary() -> Option<ProductionGenerationCorpusShapeSummary>
{
    Some(ProductionGenerationCorpusShapeSummary {
        source_summary: production_generation_source_summary(),
        boundary_request_corpus_ecliptic: production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )?,
        boundary_request_corpus_equatorial: production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Equatorial,
        )?,
    })
}

/// Returns the deterministic revision metadata for the checked-in
/// production-generation CSV fixtures (reference snapshot and independent
/// hold-out checksums). Promoted to `pub` (Slice D Task 10a) so
/// `pleiades-validate`'s relocated
/// `production_generation_source_revision_summary_for_report` renderer can
/// call it cross-crate.
pub fn production_generation_source_revision_summary() -> ProductionGenerationSourceRevisionSummary
{
    let reference_checksum = checksum64(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    )));
    let holdout_checksum = checksum64(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/independent_holdout_snapshot.csv"
    )));

    ProductionGenerationSourceRevisionSummary {
        reference_snapshot_checksum: reference_checksum,
        independent_holdout_snapshot_checksum: holdout_checksum,
    }
}

/// Compact release-facing summary for the production-generation manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionGenerationManifestSummary {
    /// Source provenance summary for the merged production-generation corpus.
    pub source_summary: ProductionGenerationSourceSummary,
    /// Coverage summary for the merged production-generation corpus.
    pub coverage_summary: ProductionGenerationSnapshotSummary,
    /// Body-class coverage summary for the merged production-generation corpus.
    pub body_class_coverage_summary: ProductionGenerationSnapshotBodyClassCoverageSummary,
    /// Boundary-overlay summary for the merged production-generation corpus.
    pub boundary_summary: ProductionGenerationBoundarySummary,
    /// Boundary window summary for the merged production-generation corpus.
    pub boundary_window_summary: ProductionGenerationBoundaryWindowSummary,
    /// Boundary request corpus summary for the merged production-generation corpus.
    pub boundary_request_corpus_summary: ProductionGenerationBoundaryRequestCorpusSummary,
}

/// Structured validation errors for a production-generation manifest summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionGenerationManifestSummaryValidationError {
    /// The nested source summary drifted from the current corpus evidence.
    Source(ProductionGenerationSourceSummaryValidationError),
    /// The nested coverage summary drifted from the current corpus evidence.
    Coverage(ProductionGenerationSnapshotSummaryValidationError),
    /// The nested body-class coverage summary drifted from the current corpus evidence.
    BodyClassCoverage(ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError),
    /// The nested boundary summary drifted from the current corpus evidence.
    Boundary(ProductionGenerationBoundarySummaryValidationError),
    /// The nested boundary-window summary drifted from the current corpus evidence.
    BoundaryWindow(ProductionGenerationBoundaryWindowSummaryValidationError),
    /// The nested boundary-request corpus summary drifted from the current corpus evidence.
    BoundaryRequestCorpus(ProductionGenerationBoundaryRequestCorpusSummaryValidationError),
    /// A rendered field drifted from the current corpus evidence.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ProductionGenerationManifestSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => write!(f, "source validation failed: {error}"),
            Self::Coverage(error) => write!(f, "coverage validation failed: {error}"),
            Self::BodyClassCoverage(error) => {
                write!(f, "body-class coverage validation failed: {error}")
            }
            Self::Boundary(error) => write!(f, "boundary summary validation failed: {error}"),
            Self::BoundaryWindow(error) => {
                write!(f, "boundary-window summary validation failed: {error}")
            }
            Self::BoundaryRequestCorpus(error) => {
                write!(f, "boundary-request corpus validation failed: {error}")
            }
            Self::FieldOutOfSync { field } => write!(
                f,
                "the production-generation manifest summary field `{field}` is out of sync with the current corpus evidence"
            ),
        }
    }
}

impl std::error::Error for ProductionGenerationManifestSummaryValidationError {}

impl ProductionGenerationManifestSummary {
    /// Returns `Ok(())` when the manifest summary still matches the derived corpus evidence.
    pub fn validate(&self) -> Result<(), ProductionGenerationManifestSummaryValidationError> {
        self.source_summary
            .validate()
            .map_err(ProductionGenerationManifestSummaryValidationError::Source)?;
        let expected_source_summary = production_generation_source_summary();
        if self.source_summary != expected_source_summary {
            return Err(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "source_summary",
                },
            );
        }

        self.coverage_summary
            .validate()
            .map_err(ProductionGenerationManifestSummaryValidationError::Coverage)?;
        let expected_coverage_summary = production_generation_snapshot_summary().ok_or(
            ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                field: "coverage_summary",
            },
        )?;
        if self.coverage_summary != expected_coverage_summary {
            return Err(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "coverage_summary",
                },
            );
        }

        self.body_class_coverage_summary
            .validate()
            .map_err(ProductionGenerationManifestSummaryValidationError::BodyClassCoverage)?;
        let expected_body_class_coverage_summary =
            production_generation_snapshot_body_class_coverage_summary().ok_or(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "body_class_coverage_summary",
                },
            )?;
        if self.body_class_coverage_summary != expected_body_class_coverage_summary {
            return Err(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "body_class_coverage_summary",
                },
            );
        }

        self.boundary_summary
            .validate()
            .map_err(ProductionGenerationManifestSummaryValidationError::Boundary)?;
        let expected_boundary_summary = production_generation_boundary_summary().ok_or(
            ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                field: "boundary_summary",
            },
        )?;
        if self.boundary_summary != expected_boundary_summary {
            return Err(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "boundary_summary",
                },
            );
        }

        self.boundary_window_summary
            .validate()
            .map_err(ProductionGenerationManifestSummaryValidationError::BoundaryWindow)?;
        let expected_boundary_window_summary = production_generation_boundary_window_summary()
            .ok_or(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "boundary_window_summary",
                },
            )?;
        if self.boundary_window_summary != expected_boundary_window_summary {
            return Err(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "boundary_window_summary",
                },
            );
        }

        self.boundary_request_corpus_summary
            .validate()
            .map_err(ProductionGenerationManifestSummaryValidationError::BoundaryRequestCorpus)?;
        let expected_boundary_request_corpus_summary =
            production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
                .ok_or(
                    ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                        field: "boundary_request_corpus_summary",
                    },
                )?;
        if self.boundary_request_corpus_summary != expected_boundary_request_corpus_summary {
            return Err(
                ProductionGenerationManifestSummaryValidationError::FieldOutOfSync {
                    field: "boundary_request_corpus_summary",
                },
            );
        }

        Ok(())
    }
}

/// Returns the compact production-generation manifest summary for release-facing reports.
pub fn production_generation_manifest_summary() -> Option<ProductionGenerationManifestSummary> {
    Some(ProductionGenerationManifestSummary {
        source_summary: production_generation_source_summary(),
        coverage_summary: production_generation_snapshot_summary()?,
        body_class_coverage_summary: production_generation_snapshot_body_class_coverage_summary()?,
        boundary_summary: production_generation_boundary_summary()?,
        boundary_window_summary: production_generation_boundary_window_summary()?,
        boundary_request_corpus_summary: production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )?,
    })
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

impl ProductionGenerationSnapshotWindow {}

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
        /// Distinct-body count carried by the summary.
        body_count: usize,
        /// Number of bodies actually listed in the summary.
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        /// Index of the first occurrence in the compared pair.
        first_index: usize,
        /// Index of the second (duplicate) occurrence in the compared pair.
        second_index: usize,
        /// Body designation involved in the mismatch.
        body: String,
    },
    /// The summary included a blank body label.
    BlankBody {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
    },
    /// The summary body order diverged from the checked-in merged corpus.
    BodyOrderMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Value expected from the current evidence slice.
        expected: String,
        /// Value recorded in the summary under validation.
        found: String,
    },
    /// The summary did not include any epochs.
    MissingEpochs,
    /// The summary reported an invalid epoch range.
    InvalidEpochRange {
        /// Earliest epoch carried by the summary.
        earliest_epoch: Instant,
        /// Latest epoch carried by the summary.
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

impl ProductionGenerationSnapshotWindowSummary {
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
}

pub(crate) fn production_generation_snapshot_window_summary_details(
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
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
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

        if self.row_count != expected.row_count {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }
        if self.major_body_row_count != expected.major_body_row_count {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_body_row_count",
                },
            );
        }
        if self.major_bodies != expected.major_bodies {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_bodies",
                },
            );
        }
        if self.major_epoch_count != expected.major_epoch_count {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_epoch_count",
                },
            );
        }
        if self.major_windows != expected.major_windows {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_windows",
                },
            );
        }
        if self.asteroid_row_count != expected.asteroid_row_count {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_row_count",
                },
            );
        }
        if self.asteroid_bodies != expected.asteroid_bodies {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_bodies",
                },
            );
        }
        if self.asteroid_epoch_count != expected.asteroid_epoch_count {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_epoch_count",
                },
            );
        }
        if self.asteroid_windows != expected.asteroid_windows {
            return Err(
                ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "asteroid_windows",
                },
            );
        }

        Ok(())
    }
}

pub(crate) fn production_generation_snapshot_body_class_coverage_summary_details(
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

#[cfg(test)]
mod tests;

/// Returns the release-facing production-generation source summary string.
pub fn production_generation_source_summary_for_report() -> String {
    let summary = production_generation_source_summary();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Production generation source: unavailable ({error})"),
    }
}

impl ProductionGenerationSourceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let cadence_fragment = production_generation_source_cadence_fragment(self)
            .unwrap_or_else(|error| format!("cadence unavailable ({error})"));
        let body_class_cadence_fragment =
            production_generation_source_body_class_cadence_fragment()
                .unwrap_or_else(|error| format!("body-class cadence unavailable ({error})"));
        let source_density_fragment = production_generation_source_density_summary_for_report()
            .unwrap_or_else(|error| format!("source density floors unavailable ({error})"));

        format!(
            "Production generation source: strategy=documented hybrid fixture corpus; {}; {}; source windows={}; reference snapshot exact J2000 evidence={}; evidence classes=reference, hold-out, boundary overlay, provenance-only; input path=checked-in CSV fixtures via include_str! reference_snapshot.csv and independent_holdout_snapshot.csv; license posture=public-source provenance only; checked-in fixtures remain repository-local regression data; {}; generation command=generate-packaged-artifact --check (consuming the checked-in CSV fixtures); file format=comma-separated values; schema=epoch_jd, body, x_km, y_km, z_km; columns=epoch_jd, body, x_km, y_km, z_km; frame=geocentric ecliptic J2000; time scale=TDB; apparentness=Mean; parser=pure-Rust and deterministic; checksum expectation=byte-identical fixture contents; {}; {}; {}; reference and hold-out rows remain separate; redistribution posture=repository-checked regression fixtures, not a broad public corpus",
            self.reference_summary.summary_line(),
            format_production_generation_boundary_source_summary(&self.boundary_summary),
            strip_report_prefix(
                &self.source_windows.summary_line(),
                "Production generation source windows: ",
            ),
            strip_report_prefix(
                &reference_snapshot_exact_j2000_evidence_summary_for_report(),
                "Reference snapshot exact J2000 evidence: ",
            ),
            self.source_revision.summary_line(),
            cadence_fragment,
            body_class_cadence_fragment,
            source_density_fragment,
        )
    }
    /// Returns a compact summary line after validating the source summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

pub(crate) fn production_generation_source_body_class_cadence_fragment(
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    let snapshot = production_generation_snapshot_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::BodyClassCadenceMismatch)?;
    let boundary = production_generation_boundary_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::BodyClassCadenceMismatch)?;

    Ok(format!(
        "body-class cadence=reference major bodies: {} epochs; reference selected asteroids: {} epochs; boundary major bodies: {} epochs; boundary selected asteroids: {} epochs",
        snapshot.major_epoch_count,
        snapshot.asteroid_epoch_count,
        boundary.major_epoch_count,
        boundary.asteroid_epoch_count,
    ))
}

pub(crate) fn production_generation_source_cadence_fragment(
    summary: &ProductionGenerationSourceSummary,
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    let boundary_request_corpus_ecliptic =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
            .ok_or(ProductionGenerationSourceSummaryValidationError::SourceWindowsMismatch)?;
    let boundary_request_corpus_equatorial =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Equatorial)
            .ok_or(ProductionGenerationSourceSummaryValidationError::SourceWindowsMismatch)?;

    production_generation_source_cadence_fragment_from_counts(
        summary.source_windows.epoch_count,
        boundary_request_corpus_ecliptic.epoch_count,
        boundary_request_corpus_equatorial.epoch_count,
    )
}

/// Returns a compact source-density summary for the production-generation corpus.
pub fn production_generation_source_density_summary_for_report(
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    let snapshot = production_generation_snapshot_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::SourceDensityMismatch)?;
    let boundary = production_generation_boundary_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::SourceDensityMismatch)?;

    Ok(format!(
        "source density floors=reference major bodies: {} epochs minimum; reference selected asteroids: {} epochs minimum; boundary major bodies: {} epochs minimum; boundary selected asteroids: {} epochs minimum",
        snapshot.major_epoch_count,
        snapshot.asteroid_epoch_count,
        boundary.major_epoch_count,
        boundary.asteroid_epoch_count,
    ))
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
}

impl ProductionGenerationSourceRevisionSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "source revision=reference_snapshot.csv checksum=0x{reference_snapshot_checksum:016x}; independent_holdout_snapshot.csv checksum=0x{independent_holdout_snapshot_checksum:016x}",
            reference_snapshot_checksum = self.reference_snapshot_checksum,
            independent_holdout_snapshot_checksum = self.independent_holdout_snapshot_checksum,
        )
    }
}

pub(crate) fn production_generation_source_cadence_fragment_from_counts(
    source_window_epoch_count: usize,
    boundary_epoch_count_ecliptic: usize,
    boundary_epoch_count_equatorial: usize,
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    if boundary_epoch_count_ecliptic != boundary_epoch_count_equatorial {
        return Err(
            ProductionGenerationSourceSummaryValidationError::BoundaryRequestCorpusEpochCountMismatch {
                ecliptic_epoch_count: boundary_epoch_count_ecliptic,
                equatorial_epoch_count: boundary_epoch_count_equatorial,
            },
        );
    }

    Ok(format!(
        "cadence={} reference epochs and {} boundary epochs",
        source_window_epoch_count, boundary_epoch_count_ecliptic
    ))
}

impl fmt::Display for ProductionGenerationSnapshotWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
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
}
