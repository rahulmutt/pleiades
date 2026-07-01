//! core parity summaries.

use core::fmt;

use pleiades_backend::{EphemerisBackend, QualityAnnotation};
use pleiades_types::{CoordinateFrame, Instant, TimeScale};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

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
        /// Number of ecliptic-frame requests carried by the summary.
        ecliptic_request_count: usize,
        /// Number of equatorial-frame requests carried by the summary.
        equatorial_request_count: usize,
        /// Row count carried by the summary under validation.
        row_count: usize,
    },
    /// The quality counts do not match the row count.
    QualityCountMismatch {
        /// Number of samples classified as exact (fixture-served).
        exact_count: usize,
        /// Number of samples classified as interpolated.
        interpolated_count: usize,
        /// Number of samples classified as approximate.
        approximate_count: usize,
        /// Number of samples with an unknown classification.
        unknown_count: usize,
        /// Row count carried by the summary under validation.
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

/// Returns the validated release-facing reference snapshot batch parity summary string.
pub fn validated_reference_snapshot_batch_parity_summary_for_report() -> Result<String, String> {
    let summary = reference_snapshot_batch_parity_summary()
        .ok_or_else(|| "JPL reference snapshot batch parity: unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Compact mixed TT/TDB batch parity for the checked-in reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotMixedTimeScaleBatchParitySummary {
    /// Reference snapshot body and epoch coverage used to build the mixed-scale slice.
    pub snapshot: ReferenceSnapshotSummary,
    /// Number of requests in the mixed-scale batch regression.
    pub request_count: usize,
    /// Number of bodies covered by the batch regression.
    pub body_count: usize,
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
    /// Whether the batch regression preserved request order.
    pub order_preserved: bool,
    /// Whether the batch regression preserved batch/single parity.
    pub single_query_parity_preserved: bool,
}

/// Validation error for a mixed TT/TDB reference-snapshot batch-parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError {
    /// The nested reference snapshot summary failed validation.
    Snapshot(ReferenceSnapshotSummaryValidationError),
    /// The number of mixed-scale requests does not match the row count.
    RequestCountMismatch {
        /// Number of requests issued on the TT time scale.
        tt_request_count: usize,
        /// Number of requests issued on the TDB time scale.
        tdb_request_count: usize,
        /// Row count carried by the summary under validation.
        row_count: usize,
    },
    /// The quality counts do not match the row count.
    QualityCountMismatch {
        /// Number of samples classified as exact (fixture-served).
        exact_count: usize,
        /// Number of samples classified as interpolated.
        interpolated_count: usize,
        /// Number of samples classified as approximate.
        approximate_count: usize,
        /// Number of samples with an unknown classification.
        unknown_count: usize,
        /// Row count carried by the summary under validation.
        row_count: usize,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl fmt::Display for ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(f, "reference snapshot validation failed: {error}"),
            Self::RequestCountMismatch {
                tt_request_count,
                tdb_request_count,
                row_count,
            } => write!(
                f,
                "request count {}+{} does not match row count {}",
                tt_request_count, tdb_request_count, row_count,
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

impl std::error::Error for ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError {}

impl ReferenceSnapshotMixedTimeScaleBatchParitySummary {
    /// Returns `Ok(())` when the summary still matches the current reference snapshot posture.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError> {
        self.snapshot
            .validate()
            .map_err(ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError::Snapshot)?;

        if self.request_count != self.snapshot.row_count {
            return Err(
                ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError::RequestCountMismatch {
                    tt_request_count: self.tt_request_count,
                    tdb_request_count: self.tdb_request_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if self.body_count != self.snapshot.body_count {
            return Err(
                ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError::DerivedSummaryMismatch,
            );
        }

        if self.tt_request_count + self.tdb_request_count != self.request_count {
            return Err(
                ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError::RequestCountMismatch {
                    tt_request_count: self.tt_request_count,
                    tdb_request_count: self.tdb_request_count,
                    row_count: self.request_count,
                },
            );
        }

        if self.tt_request_count == 0 || self.tdb_request_count == 0 {
            return Err(
                ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError::RequestCountMismatch {
                    tt_request_count: self.tt_request_count,
                    tdb_request_count: self.tdb_request_count,
                    row_count: self.request_count,
                },
            );
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.request_count
        {
            return Err(
                ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError::QualityCountMismatch {
                    exact_count: self.exact_count,
                    interpolated_count: self.interpolated_count,
                    approximate_count: self.approximate_count,
                    unknown_count: self.unknown_count,
                    row_count: self.request_count,
                },
            );
        }

        if !self.order_preserved || !self.single_query_parity_preserved {
            return Err(
                ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns the validated batch-parity summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotMixedTimeScaleBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let order = if self.order_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        let parity = if self.single_query_parity_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        format!(
            "JPL reference snapshot mixed TT/TDB batch parity: {} requests across {} bodies, TT requests={}, TDB requests={}; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; order={}, single-query parity={}",
            self.request_count,
            self.body_count,
            self.tt_request_count,
            self.tdb_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
            order,
            parity,
        )
    }
}

impl fmt::Display for ReferenceSnapshotMixedTimeScaleBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a compact mixed TT/TDB batch parity summary for the checked-in reference snapshot.
pub fn reference_snapshot_mixed_time_scale_batch_parity_summary(
) -> Option<ReferenceSnapshotMixedTimeScaleBatchParitySummary> {
    let backend = JplSnapshotBackend;
    let requests = reference_snapshot_mixed_time_scale_batch_parity_requests()?;
    let entries = reference_snapshot();
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

    for ((request, result), entry) in requests.iter().zip(results.iter()).zip(entries.iter()) {
        let single = backend.position(request).ok();
        single_query_parity &= single.as_ref().is_some_and(|single| single == result);

        order_preserved &= result.body == entry.body
            && result.instant.julian_day == entry.epoch.julian_day
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

    let snapshot = reference_snapshot_summary()?;
    let request_count = requests.len();
    let body_count = snapshot.body_count;
    let summary = ReferenceSnapshotMixedTimeScaleBatchParitySummary {
        snapshot,
        request_count,
        body_count,
        tt_request_count,
        tdb_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
        order_preserved,
        single_query_parity_preserved: single_query_parity,
    };

    debug_assert!(summary.validate().is_ok());
    Some(summary)
}

/// Formats the checked-in mixed TT/TDB reference snapshot batch parity summary for release-facing reporting.
pub fn format_reference_snapshot_mixed_time_scale_batch_parity_summary(
    summary: &ReferenceSnapshotMixedTimeScaleBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing mixed TT/TDB reference snapshot batch parity summary string.
#[doc(alias = "reference_snapshot_mixed_tt_tdb_batch_parity_summary")]
pub fn reference_snapshot_mixed_time_scale_batch_parity_summary_for_report() -> String {
    match reference_snapshot_mixed_time_scale_batch_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("JPL reference snapshot mixed TT/TDB batch parity: unavailable ({error})")
            }
        },
        None => "JPL reference snapshot mixed TT/TDB batch parity: unavailable".to_string(),
    }
}

/// Returns the validated release-facing mixed TT/TDB reference snapshot batch parity summary string.
#[doc(alias = "validated_reference_snapshot_mixed_tt_tdb_batch_parity_summary_for_report")]
pub fn validated_reference_snapshot_mixed_time_scale_batch_parity_summary_for_report(
) -> Result<String, String> {
    let summary = reference_snapshot_mixed_time_scale_batch_parity_summary().ok_or_else(|| {
        "JPL reference snapshot mixed TT/TDB batch parity: unavailable".to_string()
    })?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}
