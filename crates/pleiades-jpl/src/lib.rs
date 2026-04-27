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
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    FrameTreatmentSummary, QualityAnnotation,
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
    reference_snapshot_summary().map(|summary| ReferenceSnapshotEquatorialParitySummary {
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
        Some(summary) => format_reference_snapshot_equatorial_parity_summary(&summary),
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
    let requests = reference_snapshot()
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
        .collect::<Vec<_>>();
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

impl ReferenceSnapshotBatchParitySummary {
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
        Some(summary) => format_reference_snapshot_batch_parity_summary(&summary),
        None => "JPL reference snapshot batch parity: unavailable".to_string(),
    }
}

impl ReferenceSnapshotSummary {
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
        Some(summary) => format_reference_snapshot_summary(&summary),
        None => "Reference snapshot coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the comparison snapshot used by validation.
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
}

impl fmt::Display for IndependentHoldoutSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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
        Some(summary) => format_independent_holdout_snapshot_summary(&summary),
        None => match independent_holdout_snapshot_error() {
            Some(error) => format!("Independent hold-out coverage: unavailable ({error})"),
            None => "Independent hold-out coverage: unavailable".to_string(),
        },
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
    let requests = entries
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
        .collect::<Vec<_>>();
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

impl IndependentHoldoutSnapshotBatchParitySummary {
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
        Some(summary) => format_independent_holdout_snapshot_batch_parity_summary(&summary),
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
        Some(summary) => format_independent_holdout_snapshot_equatorial_parity_summary(&summary),
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

impl ComparisonSnapshotSourceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Comparison snapshot source: {}; coverage={}; columns={}",
            self.source, self.coverage, self.columns
        )
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

/// Returns the source/material summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_source_summary_for_report() -> String {
    comparison_snapshot_source_summary().summary_line()
}

/// Returns the manifest summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_manifest_summary_for_report() -> String {
    comparison_snapshot_manifest().summary_line("Comparison snapshot manifest")
}

impl ComparisonSnapshotSummary {
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
        Some(summary) => format_comparison_snapshot_summary(&summary),
        None => "Comparison snapshot coverage: unavailable".to_string(),
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

/// Returns the exact J2000 asteroid equatorial evidence samples derived from the reference snapshot.
pub fn reference_asteroid_equatorial_evidence() -> &'static [ReferenceAsteroidEquatorialEvidence] {
    reference_asteroid_equatorial_evidence_list()
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

/// Formats the exact asteroid evidence slice for release-facing reporting.
pub fn format_reference_asteroid_evidence_summary(
    evidence: &[ReferenceAsteroidEvidence],
) -> String {
    if evidence.is_empty() {
        "Selected asteroid evidence: unavailable".to_string()
    } else {
        format!(
            "Selected asteroid evidence: {} exact J2000 samples at {} ({})",
            evidence.len(),
            format_instant(evidence[0].epoch),
            format_bodies(reference_asteroids())
        )
    }
}

/// Returns the release-facing exact asteroid evidence summary string.
pub fn reference_asteroid_evidence_summary_for_report() -> String {
    format_reference_asteroid_evidence_summary(reference_asteroid_evidence())
}

/// Formats the equatorial asteroid evidence slice for release-facing reporting.
pub fn format_reference_asteroid_equatorial_evidence_summary(
    evidence: &[ReferenceAsteroidEquatorialEvidence],
) -> String {
    if evidence.is_empty() {
        "Selected asteroid equatorial evidence: unavailable".to_string()
    } else {
        format!(
            "Selected asteroid equatorial evidence: {} exact J2000 samples at {} ({}) using a mean-obliquity equatorial transform",
            evidence.len(),
            format_instant(evidence[0].epoch),
            format_bodies(reference_asteroids())
        )
    }
}

/// Returns the release-facing equatorial asteroid evidence summary string.
pub fn reference_asteroid_equatorial_evidence_summary_for_report() -> String {
    format_reference_asteroid_equatorial_evidence_summary(reference_asteroid_equatorial_evidence())
}

const REFERENCE_SNAPSHOT_SOURCE_FALLBACK: &str = "NASA/JPL Horizons API vector tables (DE441)";
const INDEPENDENT_HOLDOUT_SOURCE_FALLBACK: &str = "NASA/JPL Horizons API vector tables (DE441)";
const INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK: &str =
    "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Saturn at 2400000, 2451545, and 2500000.";

/// Backend-owned provenance summary for the checked-in reference snapshot source material.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSourceSummary {
    /// Source attribution for the checked-in reference snapshot.
    pub source: String,
    /// Frame and coordinate posture described by the checked-in reference snapshot.
    pub frame_treatment: String,
    /// Reference epoch used by the checked-in snapshot.
    pub reference_epoch: Instant,
}

impl ReferenceSnapshotSourceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot source: {}; {}; TDB reference epoch {}",
            self.source,
            self.frame_treatment,
            format_instant(self.reference_epoch),
        )
    }
}

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
                frame_treatment: "geocentric ecliptic J2000".to_string(),
                reference_epoch: reference_instant(),
            }
        })
        .clone()
}

/// Returns the source-material summary for the checked-in reference snapshot.
pub fn reference_snapshot_source_summary_for_report() -> String {
    reference_snapshot_source_summary().summary_line()
}

/// Returns the manifest summary for the checked-in reference snapshot.
pub fn reference_snapshot_manifest_summary_for_report() -> String {
    reference_snapshot_manifest().summary_line("Reference snapshot manifest")
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
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Independent hold-out source: {}; coverage={}; columns={}",
            self.source, self.coverage, self.columns
        )
    }
}

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
    independent_holdout_source_summary().summary_line()
}

/// Returns the manifest summary for the checked-in hold-out snapshot.
pub fn independent_holdout_manifest_summary_for_report() -> String {
    independent_holdout_snapshot_manifest().summary_line("Independent hold-out manifest")
}

/// Returns the combined snapshot evidence summary used by validation and release reports.
pub fn jpl_snapshot_evidence_summary_for_report() -> String {
    format!(
        "{} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {}",
        reference_snapshot_summary_for_report(),
        reference_snapshot_equatorial_parity_summary_for_report(),
        reference_snapshot_batch_parity_summary_for_report(),
        reference_snapshot_source_summary_for_report(),
        reference_snapshot_manifest_summary_for_report(),
        reference_asteroid_evidence_summary_for_report(),
        reference_asteroid_equatorial_evidence_summary_for_report(),
        comparison_snapshot_summary_for_report(),
        comparison_snapshot_source_summary_for_report(),
        comparison_snapshot_manifest_summary_for_report(),
        independent_holdout_snapshot_summary_for_report(),
        independent_holdout_snapshot_equatorial_parity_summary_for_report(),
        independent_holdout_snapshot_batch_parity_summary_for_report(),
        independent_holdout_source_summary_for_report(),
        independent_holdout_manifest_summary_for_report(),
        jpl_independent_holdout_summary_for_report(),
    )
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
    jpl_snapshot_request_policy().to_string()
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

/// Returns the comparison-only subset used by the stage-4 validation corpus.
pub fn comparison_snapshot() -> &'static [SnapshotEntry] {
    comparison_snapshot_entries()
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
}

impl fmt::Display for JplInterpolationQualitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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
}

impl fmt::Display for JplInterpolationQualityKindCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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
}

impl fmt::Display for JplIndependentHoldoutSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the independent hold-out summary for release-facing reporting.
pub fn format_jpl_independent_holdout_summary(summary: &JplIndependentHoldoutSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out interpolation summary string.
pub fn jpl_independent_holdout_summary_for_report() -> String {
    match jpl_independent_holdout_summary() {
        Some(summary) => summary.to_string(),
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

/// Formats the interpolation-quality summary together with the distinct-body coverage line.
pub fn format_jpl_interpolation_quality_summary_for_report() -> String {
    match (
        jpl_interpolation_quality_summary(),
        jpl_interpolation_quality_kind_coverage(),
    ) {
        (Some(summary), Some(coverage)) => {
            let mut rendered = summary.to_string();
            rendered.push('\n');
            rendered.push_str(&coverage.to_string());
            rendered
        }
        (Some(summary), None) => {
            let mut rendered = summary.to_string();
            rendered.push('\n');
            rendered.push_str("JPL interpolation quality kind coverage: unavailable");
            rendered
        }
        (None, _) => "JPL interpolation quality: unavailable".to_string(),
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
            Self::BlankColumn { .. } => "blank column",
            Self::DuplicateColumn { .. } => "duplicate column",
        }
    }
}

impl fmt::Display for SnapshotManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            .source
            .as_deref()
            .map(str::trim)
            .filter(|source| !source.is_empty())
            .is_none()
        {
            return Err(SnapshotManifestValidationError::MissingSource);
        }
        if matches!(self.coverage.as_deref(), Some(coverage) if coverage.trim().is_empty()) {
            return Err(SnapshotManifestValidationError::BlankCoverage);
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
    MissingColumn { column: &'static str },
    UnexpectedExtraColumns,
    UnsupportedBody { body: String },
    InvalidNumber { column: &'static str, value: String },
    DuplicateEntry { body: String, epoch: Instant },
}

impl fmt::Display for SnapshotLoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColumn { column } => write!(f, "missing {column} column"),
            Self::UnexpectedExtraColumns => f.write_str("unexpected extra columns"),
            Self::UnsupportedBody { body } => write!(f, "unsupported body '{body}'"),
            Self::InvalidNumber { column, value } => {
                write!(f, "invalid {column} value '{value}'")
            }
            Self::DuplicateEntry { body, epoch } => {
                write!(
                    f,
                    "duplicate row for body '{body}' at {}",
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

fn comparison_snapshot_entries() -> &'static [SnapshotEntry] {
    static SNAPSHOT: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    SNAPSHOT
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| is_comparison_body(&entry.body))
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

fn independent_holdout_snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    independent_holdout_state().entries()
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
    let mut seen_entries = BTreeSet::new();

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
                if !seen_entries.insert(entry_key.clone()) {
                    return Err(SnapshotLoadError::new(
                        line_number,
                        SnapshotLoadErrorKind::DuplicateEntry {
                            body: entry_key.0,
                            epoch: entry.epoch,
                        },
                    ));
                }
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
        assert_eq!(reference_epochs().len(), 6);
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
            10
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
        assert_eq!(summary.row_count, 46);
        assert_eq!(summary.body_count, 15);
        assert_eq!(summary.bodies, reference_bodies());
        assert_eq!(summary.epoch_count, 6);
        assert_eq!(summary.asteroid_row_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_499.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "Reference snapshot coverage: 46 rows across 15 bodies and 6 epochs (5 asteroid rows; JD 2378499.0 (TDB)..JD 2634167.0 (TDB)); bodies: {}",
                format_bodies(reference_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
        let summary = reference_snapshot_equatorial_parity_summary()
            .expect("reference snapshot equatorial parity summary should exist");
        assert_eq!(summary.row_count, 46);
        assert_eq!(summary.body_count, 15);
        assert_eq!(summary.bodies, reference_bodies());
        assert_eq!(summary.epoch_count, 6);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_499.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "JPL reference snapshot equatorial parity: 46 rows across 15 bodies and 6 epochs (JD 2378499.0 (TDB)..JD 2634167.0 (TDB)); bodies: {}; mean-obliquity transform against the checked-in ecliptic fixture",
                format_bodies(reference_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_equatorial_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_snapshot_batch_parity_summary_reports_the_expected_coverage() {
        let summary = reference_snapshot_batch_parity_summary()
            .expect("reference snapshot batch parity summary should exist");
        assert_eq!(summary.snapshot.row_count, 46);
        assert_eq!(summary.snapshot.body_count, 15);
        assert_eq!(summary.snapshot.bodies, reference_bodies());
        assert_eq!(summary.snapshot.epoch_count, 6);
        assert_eq!(
            summary.snapshot.earliest_epoch.julian_day.days(),
            2_378_499.0
        );
        assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.ecliptic_request_count, 23);
        assert_eq!(summary.equatorial_request_count, 23);
        assert_eq!(summary.exact_count, 46);
        assert_eq!(summary.interpolated_count, 0);
        assert_eq!(summary.approximate_count, 0);
        assert_eq!(summary.unknown_count, 0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "JPL reference snapshot batch parity: 46 rows across 15 bodies and 6 epochs (JD 2378499.0 (TDB)..JD 2634167.0 (TDB)); bodies: {}; frame mix: 23 ecliptic, 23 equatorial; quality counts: Exact=46, Interpolated=0, Approximate=0, Unknown=0; batch/single parity preserved",
                format_bodies(reference_bodies())
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_batch_parity_summary_for_report(),
            summary.summary_line()
        );
        assert!(jpl_snapshot_evidence_summary_for_report().contains(
            "JPL reference snapshot batch parity: 46 rows across 15 bodies and 6 epochs (JD 2378499.0 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
    }

    #[test]
    fn comparison_snapshot_summary_reports_the_expected_coverage() {
        let summary =
            comparison_snapshot_summary().expect("comparison snapshot summary should exist");
        assert_eq!(summary.row_count, 41);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 6);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_499.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.bodies.as_slice(), comparison_bodies());
        assert_eq!(
            summary.summary_line(),
            "Comparison snapshot coverage: 41 rows across 10 bodies and 6 epochs (JD 2378499.0 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Mercury, Moon, Sun, Venus, Jupiter, Saturn, Uranus, Neptune, Pluto"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            comparison_snapshot_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn reference_asteroid_evidence_summary_reports_the_expected_coverage() {
        let report = reference_asteroid_evidence_summary_for_report();
        assert_eq!(report, "Selected asteroid evidence: 5 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)");
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
        assert_eq!(manifest.coverage, None);
        assert_eq!(manifest.columns, ["body", "x_km", "y_km", "z_km"]);
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(
            source_summary.summary_line(),
            "Comparison snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km"
        );
        assert_eq!(source_summary.to_string(), source_summary.summary_line());
        assert_eq!(
            format_comparison_snapshot_source_summary(&source_summary),
            source_summary.summary_line()
        );
        assert_eq!(
            comparison_snapshot_source_summary_for_report(),
            source_summary.summary_line()
        );
        assert_eq!(
            manifest.summary_line("Comparison snapshot manifest"),
            "Comparison snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=unknown; columns=body, x_km, y_km, z_km"
        );
        assert_eq!(
            manifest.to_string(),
            "Snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=unknown; columns=body, x_km, y_km, z_km"
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
    fn reference_asteroid_equatorial_evidence_summary_reports_the_expected_coverage() {
        let report = reference_asteroid_equatorial_evidence_summary_for_report();
        assert_eq!(report, "Selected asteroid equatorial evidence: 5 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros) using a mean-obliquity equatorial transform");
    }

    #[test]
    fn batch_query_preserves_reference_snapshot_order_and_equatorial_values() {
        let backend = JplSnapshotBackend;
        let requests = reference_snapshot()
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame: CoordinateFrame::Equatorial,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect::<Vec<_>>();

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
        let requests = reference_snapshot()
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
            .collect::<Vec<_>>();

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
        assert_eq!(summary.frame_treatment, "geocentric ecliptic J2000");
        assert_eq!(summary.reference_epoch.julian_day.days(), 2_451_545.0);
        assert_eq!(
            summary.summary_line(),
            format!(
                "Reference snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; geocentric ecliptic J2000; TDB reference epoch {}",
                format_instant(summary.reference_epoch)
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            reference_snapshot_source_summary_for_report(),
            summary.summary_line()
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
            "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Saturn at 2400000, 2451545, and 2500000."
        );
        assert_eq!(summary.columns, "epoch_jd, body, x_km, y_km, z_km");
        assert_eq!(
            summary.summary_line(),
            "Independent hold-out source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Saturn at 2400000, 2451545, and 2500000.; columns=epoch_jd, body, x_km, y_km, z_km"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            independent_holdout_source_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn independent_holdout_snapshot_summary_reports_the_expected_coverage() {
        let summary = independent_holdout_snapshot_summary()
            .expect("independent hold-out summary should exist");
        assert_eq!(summary.row_count, 9);
        assert_eq!(summary.body_count, 3);
        assert_eq!(summary.bodies, vec!["Mars", "Jupiter", "Saturn"]);
        assert_eq!(summary.epoch_count, 6);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_400_000.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_500_000.0);
        assert_eq!(
            summary.summary_line(),
            "Independent hold-out coverage: 9 rows across 3 bodies and 6 epochs (JD 2400000.0 (TDB)..JD 2500000.0 (TDB)); bodies: Mars, Jupiter, Saturn"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            independent_holdout_snapshot_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn independent_holdout_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
        let summary = independent_holdout_snapshot_equatorial_parity_summary()
            .expect("independent hold-out equatorial parity summary should exist");
        assert_eq!(summary.row_count, 9);
        assert_eq!(summary.body_count, 3);
        assert_eq!(summary.epoch_count, 6);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_400_000.0);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_500_000.0);
        assert_eq!(
            summary.summary_line(),
            "JPL independent hold-out equatorial parity: 9 rows across 3 bodies and 6 epochs (JD 2400000.0 (TDB)..JD 2500000.0 (TDB)); mean-obliquity transform against the checked-in ecliptic fixture"
        );
        assert_eq!(
            independent_holdout_snapshot_equatorial_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn independent_holdout_summary_reports_the_expected_envelope() {
        let summary =
            jpl_independent_holdout_summary().expect("independent hold-out summary should exist");
        assert_eq!(summary.sample_count, 9);
        assert_eq!(summary.body_count, 3);
        assert_eq!(summary.bodies, vec!["Mars", "Jupiter", "Saturn"]);
        assert_eq!(summary.epoch_count, 6);
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
        assert!(
            rendered.contains("9 exact rows across 3 bodies (Mars, Jupiter, Saturn) and 6 epochs")
        );
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
        let requests = entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect::<Vec<_>>();

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
        let requests = entries
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
            .collect::<Vec<_>>();

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
        let requests = entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame: CoordinateFrame::Equatorial,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect::<Vec<_>>();

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
        assert_eq!(summary.snapshot.row_count, 9);
        assert_eq!(summary.snapshot.body_count, 3);
        assert_eq!(summary.tt_request_count, 5);
        assert_eq!(summary.tdb_request_count, 4);
        assert!(summary.parity_preserved);
        assert_eq!(
            summary.exact_count
                + summary.interpolated_count
                + summary.approximate_count
                + summary.unknown_count,
            summary.snapshot.row_count,
        );

        let rendered = format_independent_holdout_snapshot_batch_parity_summary(&summary);
        assert!(rendered.contains("JPL independent hold-out batch parity:"));
        assert!(
            rendered.contains("9 requests across 3 bodies (Mars, Jupiter, Saturn) and 6 epochs")
        );
        assert!(rendered.contains("TT requests=5, TDB requests=4"));
        assert!(rendered.contains("quality counts:"));
        assert!(rendered.contains("order=preserved, single-query parity=preserved"));
    }

    #[test]
    fn jpl_snapshot_evidence_summary_combines_the_backend_reports() {
        let report = jpl_snapshot_evidence_summary_for_report();
        assert!(report.contains(&reference_snapshot_summary_for_report()));
        assert!(report.contains(&reference_snapshot_equatorial_parity_summary_for_report()));
        assert!(report.contains(&reference_snapshot_source_summary_for_report()));
        assert!(report.contains(&reference_snapshot_manifest_summary_for_report()));
        assert!(report.contains(&reference_asteroid_evidence_summary_for_report()));
        assert!(report.contains(&reference_asteroid_equatorial_evidence_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_source_summary_for_report()));
        assert!(report.contains(&comparison_snapshot_manifest_summary_for_report()));
        assert!(report.contains(&independent_holdout_snapshot_summary_for_report()));
        assert!(
            report.contains(&independent_holdout_snapshot_equatorial_parity_summary_for_report())
        );
        assert!(report.contains(&independent_holdout_snapshot_batch_parity_summary_for_report()));
        assert!(report.contains(&independent_holdout_source_summary_for_report()));
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
        assert_eq!(manifest.coverage.as_deref(), Some("inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; outer planets and Pluto sampled at J2000 and 2132."));
        assert_eq!(
            manifest.columns,
            ["epoch_jd", "body", "x_km", "y_km", "z_km"]
        );
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(
            manifest.summary_line("Reference snapshot manifest"),
            "Reference snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=inner planets sampled across 1800-2500, with an additional 2406 Mars hold-out; outer planets and Pluto sampled at J2000 and 2132.; columns=epoch_jd, body, x_km, y_km, z_km"
        );
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
            Some("Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Saturn at 2400000, 2451545, and 2500000.")
        );
        assert_eq!(
            manifest.columns,
            ["epoch_jd", "body", "x_km", "y_km", "z_km"]
        );
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(
            manifest.summary_line("Independent hold-out manifest"),
            "Independent hold-out manifest: Independent JPL Horizons hold-out snapshot used only for interpolation validation.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Saturn at 2400000, 2451545, and 2500000.; columns=epoch_jd, body, x_km, y_km, z_km"
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
    fn parser_reports_malformed_rows_without_panicking() {
        let error = load_snapshot_from_str("2451545.0,Sun,1.0,2.0\n")
            .expect_err("missing columns should be reported");
        assert!(format!("{error}").contains("missing z"));

        let error = load_snapshot_from_str("2451545.0,Comet,1.0,2.0,3.0\n")
            .expect_err("unsupported bodies should be reported");
        assert!(format!("{error}").contains("unsupported body 'Comet'"));
    }

    #[test]
    fn parser_rejects_duplicate_body_epoch_rows() {
        let error =
            load_snapshot_from_str("2451545.0,Sun,1.0,2.0,3.0\n2451545.0,Sun,4.0,5.0,6.0\n")
                .expect_err("duplicate body/epoch pairs should be reported");
        assert!(format!("{error}").contains("line 2"));
        assert!(format!("{error}").contains("duplicate row for body 'Sun'"));
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

        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
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

        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
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

        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
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

        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    }

    #[test]
    fn batch_query_preserves_mixed_time_scales_across_the_reference_snapshot() {
        let backend = JplSnapshotBackend;
        let requests = reference_snapshot()
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
            .collect::<Vec<_>>();

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
        assert_eq!(samples.len(), 21);
        assert!(samples.iter().all(|sample| {
            let epoch = sample.epoch.julian_day.days();
            (epoch == 2_400_000.0
                || epoch == REFERENCE_EPOCH_JD
                || epoch == 2_500_000.0
                || epoch == 2_600_000.0)
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
        assert!(samples
            .iter()
            .all(|sample| { sample.interpolation_kind != InterpolationQualityKind::Quadratic }));
        assert!(samples
            .iter()
            .any(|sample| sample.interpolation_kind == InterpolationQualityKind::Linear));
        assert!(samples
            .iter()
            .any(|sample| sample.epoch.julian_day.days() == 2_400_000.0));
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
    fn batch_query_preserves_interpolation_quality_samples_and_order() {
        let backend = JplSnapshotBackend;
        let samples = interpolation_quality_samples();
        let requests = samples
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
    fn interpolation_quality_summary_reports_the_worst_case_labels() {
        let summary = jpl_interpolation_quality_summary().expect("summary should exist");
        assert_eq!(summary.sample_count, 21);
        assert_eq!(summary.body_count, 10);
        assert_eq!(summary.epoch_count, 4);
        assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
        assert_eq!(
            summary.cubic_sample_count
                + summary.quadratic_sample_count
                + summary.linear_sample_count,
            summary.sample_count
        );
        assert!(summary.cubic_sample_count > 0);
        assert_eq!(summary.quadratic_sample_count, 0);
        assert!(summary.linear_sample_count > 0);
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
        assert!(rendered.contains("21 samples across 10 bodies and 4 epochs"));
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
        assert_eq!(coverage.sample_count, 21);
        assert_eq!(coverage.body_count, 10);
        assert_eq!(coverage.bodies.len(), coverage.body_count);
        assert!(!coverage.bodies.is_empty());
        assert!(coverage.cubic_body_count > 0);
        assert_eq!(coverage.quadratic_body_count, 0);
        assert!(coverage.linear_body_count > 0);

        assert_eq!(coverage.to_string(), coverage.summary_line());

        let rendered = format_jpl_interpolation_quality_kind_coverage(&coverage);
        assert!(rendered.contains("JPL interpolation quality kind coverage:"));
        assert!(rendered.contains("21 samples across 10 bodies ["));
        assert!(rendered.contains(&coverage.bodies[0]));
        assert!(rendered.contains("cubic bodies"));
        assert!(rendered.contains("quadratic bodies"));
        assert!(rendered.contains("linear bodies"));
    }

    #[test]
    fn interpolation_quality_summary_for_report_combines_summary_and_coverage() {
        let summary = jpl_interpolation_quality_summary().expect("summary should exist");
        let coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
        let rendered = format_jpl_interpolation_quality_summary_for_report();

        assert!(rendered.contains(&format_jpl_interpolation_quality_summary(&summary)));
        assert!(rendered.contains(&format_jpl_interpolation_quality_kind_coverage(&coverage)));
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
        assert!(summary.summary_line().contains("mean-obliquity transform"));
    }

    #[test]
    fn request_policy_summary_is_displayable() {
        let policy = jpl_snapshot_request_policy();

        assert_eq!(policy.to_string(), policy.summary_line());
        assert_eq!(
            jpl_snapshot_request_policy_summary_for_report(),
            policy.summary_line()
        );
        assert!(policy
            .summary_line()
            .contains("frames=Ecliptic, Equatorial"));
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
            instant: Instant::new(JulianDay::from_days(2_451_546.0), TimeScale::Tdb),
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
