//! selected asteroid summaries.

use core::fmt;
use std::sync::OnceLock;

use pleiades_backend::{EphemerisBackend, EphemerisRequest, QualityAnnotation};
use pleiades_types::{Apparentness, CoordinateFrame, Instant, TimeScale, ZodiacMode};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

pub(crate) fn selected_asteroid_source_entries() -> Option<&'static [SnapshotEntry]> {
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

/// Returns the selected-asteroid source request corpus in the requested frame.
pub fn selected_asteroid_source_requests(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_entries().map(|entries| {
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

/// This is a compatibility alias for [`selected_asteroid_source_requests`].
#[doc(alias = "selected_asteroid_source_requests")]
pub fn selected_asteroid_source_request_corpus(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_requests(frame)
}

/// Returns the selected-asteroid source request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`selected_asteroid_source_requests`].
#[doc(alias = "selected_asteroid_source_requests")]
pub fn selected_asteroid_source_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_requests(CoordinateFrame::Ecliptic)
}

/// Returns the selected-asteroid source request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`selected_asteroid_source_ecliptic_request_corpus`].
#[doc(alias = "selected_asteroid_source_ecliptic_request_corpus")]
pub fn selected_asteroid_source_ecliptic_requests() -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_ecliptic_request_corpus()
}

/// Returns the selected-asteroid source request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`selected_asteroid_source_requests`].
#[doc(alias = "selected_asteroid_source_requests")]
pub fn selected_asteroid_source_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_requests(CoordinateFrame::Equatorial)
}

/// Returns the selected-asteroid source request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`selected_asteroid_source_equatorial_request_corpus`].
#[doc(alias = "selected_asteroid_source_equatorial_request_corpus")]
pub fn selected_asteroid_source_equatorial_requests() -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_equatorial_request_corpus()
}

/// Returns the mixed-frame selected-asteroid source request corpus used by batch parity checks.
pub fn selected_asteroid_source_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_entries().map(|entries| {
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

/// This is a compatibility alias for [`selected_asteroid_source_batch_parity_requests`].
#[doc(alias = "selected_asteroid_source_batch_parity_requests")]
pub fn selected_asteroid_source_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    selected_asteroid_source_batch_parity_requests()
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
            "Selected asteroid source evidence: {} source-backed samples across {} bodies and {} epochs ({}..{}); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: {}",
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
            "Selected asteroid source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: {}",
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

pub(crate) fn selected_asteroid_source_evidence_summary_details(
) -> Option<SelectedAsteroidSourceSummary> {
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

pub(crate) fn selected_asteroid_source_window_summary_details(
) -> Option<SelectedAsteroidSourceWindowSummary> {
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

/// Returns the validated release-facing expanded selected-asteroid source coverage summary string.
pub fn validated_selected_asteroid_source_evidence_summary_for_report() -> Result<String, String> {
    let summary = selected_asteroid_source_evidence_summary()
        .ok_or_else(|| "selected asteroid source evidence unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
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

/// Returns the validated release-facing selected-asteroid source-window summary string.
pub fn validated_selected_asteroid_source_window_summary_for_report() -> Result<String, String> {
    let summary = selected_asteroid_source_window_summary()
        .ok_or_else(|| "selected asteroid source windows unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Compact release-facing summary for the selected-asteroid source request corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSourceRequestCorpusSummary {
    /// Number of generated requests.
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

/// Validation error for a selected-asteroid source request corpus summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedAsteroidSourceRequestCorpusSummaryValidationError {
    /// A summary field is out of sync with the checked-in request corpus.
    FieldOutOfSync { field: &'static str },
}

impl SelectedAsteroidSourceRequestCorpusSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::FieldOutOfSync { .. } => "field out of sync",
        }
    }
}

impl fmt::Display for SelectedAsteroidSourceRequestCorpusSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the selected asteroid source request corpus summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidSourceRequestCorpusSummaryValidationError {}

impl SelectedAsteroidSourceRequestCorpusSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Selected asteroid source request corpus: {} requests (frame={}; time scale={}; zodiac mode={}; apparentness={}; observerless) across {} bodies and {} epochs ({}..{}); bodies: {}",
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

    /// Returns `Ok(())` when the request corpus summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), SelectedAsteroidSourceRequestCorpusSummaryValidationError> {
        let Some(expected) = selected_asteroid_source_request_corpus_summary_details(self.frame)
        else {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "request_count",
                },
            );
        };

        if self.request_count != expected.request_count {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "request_count",
                },
            );
        }
        if self.body_count != expected.body_count {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.frame != expected.frame {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "frame",
                },
            );
        }
        if self.time_scale != expected.time_scale {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "time_scale",
                },
            );
        }
        if self.zodiac_mode != expected.zodiac_mode {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "zodiac_mode",
                },
            );
        }
        if self.apparentness != expected.apparentness {
            return Err(
                SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "apparentness",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated selected-asteroid request corpus summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidSourceRequestCorpusSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidSourceRequestCorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn selected_asteroid_source_request_corpus_summary_details(
    frame: CoordinateFrame,
) -> Option<SelectedAsteroidSourceRequestCorpusSummary> {
    let entries = selected_asteroid_source_entries()?;
    let requests = selected_asteroid_source_requests(frame)?;
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
            || request.instant != entry.epoch
            || request.frame != frame
            || request.instant.scale != time_scale
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

    Some(SelectedAsteroidSourceRequestCorpusSummary {
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

/// Returns the selected-asteroid source request corpus summary in the requested frame.
pub fn selected_asteroid_source_request_corpus_summary(
    frame: CoordinateFrame,
) -> Option<SelectedAsteroidSourceRequestCorpusSummary> {
    selected_asteroid_source_request_corpus_summary_details(frame)
}

/// Formats the selected-asteroid source request corpus for release-facing reporting.
pub fn format_selected_asteroid_source_request_corpus_summary(
    summary: &SelectedAsteroidSourceRequestCorpusSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing selected-asteroid source request corpus summary string for the requested frame.
pub fn selected_asteroid_source_request_corpus_summary_for_frame(frame: CoordinateFrame) -> String {
    match selected_asteroid_source_request_corpus_summary(frame) {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Selected asteroid source request corpus: unavailable ({error})")
            }
        },
        None => "Selected asteroid source request corpus: unavailable".to_string(),
    }
}

/// Returns the validated release-facing selected-asteroid source request corpus summary string for the requested frame.
pub fn validated_selected_asteroid_source_request_corpus_summary_for_frame(
    frame: CoordinateFrame,
) -> Result<String, String> {
    let summary = selected_asteroid_source_request_corpus_summary(frame)
        .ok_or_else(|| "selected asteroid source request corpus unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the release-facing selected-asteroid source request corpus summary string.
pub fn selected_asteroid_source_request_corpus_summary_for_report() -> String {
    selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Ecliptic)
}

/// Returns the validated release-facing selected-asteroid source request corpus summary string.
pub fn validated_selected_asteroid_source_request_corpus_summary_for_report(
) -> Result<String, String> {
    validated_selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Ecliptic)
}

/// Returns the release-facing equatorial selected-asteroid source request corpus summary string.
pub fn selected_asteroid_source_request_corpus_equatorial_summary_for_report() -> String {
    selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Equatorial)
}

/// Returns the validated release-facing equatorial selected-asteroid source request corpus summary string.
pub fn validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report(
) -> Result<String, String> {
    validated_selected_asteroid_source_request_corpus_summary_for_frame(CoordinateFrame::Equatorial)
}

pub(crate) fn selected_asteroid_source_2453000_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_reference_asteroid(&entry.body)
                        && entry.epoch.julian_day.days() == SELECTED_ASTEROID_SOURCE_2453000_EPOCH
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

/// Compact release-facing summary for the selected-asteroid 2003-12-27 source evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSource2453000Summary {
    /// Number of exact samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the source slice.
    pub epoch: Instant,
}

/// Validation errors for a selected-asteroid 2003-12-27 source summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidSource2453000SummaryValidationError {
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

impl fmt::Display for SelectedAsteroidSource2453000SummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid 2003-12-27 source evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid 2003-12-27 source evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid 2003-12-27 source evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid 2003-12-27 source evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidSource2453000SummaryValidationError {}

impl SelectedAsteroidSource2453000Summary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference selected-asteroid 2003-12-27 source evidence: {} exact samples at {} ({}); 2003-12-27 source sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidSource2453000SummaryValidationError> {
        let evidence = selected_asteroid_source_2453000_entries()
            .ok_or(SelectedAsteroidSource2453000SummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidSource2453000SummaryValidationError::SampleCountMismatch {
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
                        SelectedAsteroidSource2453000SummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidSource2453000SummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                SelectedAsteroidSource2453000SummaryValidationError::EpochMismatch {
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
    ) -> Result<String, SelectedAsteroidSource2453000SummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidSource2453000Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn selected_asteroid_source_2453000_summary_details(
) -> Option<SelectedAsteroidSource2453000Summary> {
    let evidence = selected_asteroid_source_2453000_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(SelectedAsteroidSource2453000Summary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the selected-asteroid 2003-12-27 source evidence.
pub fn selected_asteroid_source_2453000_summary() -> Option<SelectedAsteroidSource2453000Summary> {
    selected_asteroid_source_2453000_summary_details()
}

/// Returns the release-facing selected-asteroid 2003-12-27 source summary string.
pub fn selected_asteroid_source_2453000_summary_for_report() -> String {
    match selected_asteroid_source_2453000_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Selected asteroid 2003-12-27 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2003-12-27 source evidence: unavailable".to_string(),
    }
}

pub(crate) fn selected_asteroid_source_2500000_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_reference_asteroid(&entry.body)
                        && entry.epoch.julian_day.days() == SELECTED_ASTEROID_SOURCE_2500000_EPOCH
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

/// Compact release-facing summary for the selected-asteroid 2500000 source evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSource2500000Summary {
    /// Number of exact samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the source slice.
    pub epoch: Instant,
}

/// Validation errors for a selected-asteroid 2500000 source summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidSource2500000SummaryValidationError {
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

impl fmt::Display for SelectedAsteroidSource2500000SummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid 2500000 source evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid 2500000 source evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid 2500000 source evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid 2500000 source evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidSource2500000SummaryValidationError {}

impl SelectedAsteroidSource2500000Summary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference selected-asteroid 2500000 source evidence: {} exact samples at {} ({}); 2500000 source sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidSource2500000SummaryValidationError> {
        let evidence = selected_asteroid_source_2500000_entries()
            .ok_or(SelectedAsteroidSource2500000SummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidSource2500000SummaryValidationError::SampleCountMismatch {
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
                        SelectedAsteroidSource2500000SummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidSource2500000SummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                SelectedAsteroidSource2500000SummaryValidationError::EpochMismatch {
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
    ) -> Result<String, SelectedAsteroidSource2500000SummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidSource2500000Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn selected_asteroid_source_2500000_summary_details(
) -> Option<SelectedAsteroidSource2500000Summary> {
    let evidence = selected_asteroid_source_2500000_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(SelectedAsteroidSource2500000Summary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the selected-asteroid 2500000 source evidence.
pub fn selected_asteroid_source_2500000_summary() -> Option<SelectedAsteroidSource2500000Summary> {
    selected_asteroid_source_2500000_summary_details()
}

/// Returns the release-facing selected-asteroid 2500000 source summary string.
pub fn selected_asteroid_source_2500000_summary_for_report() -> String {
    match selected_asteroid_source_2500000_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Selected asteroid 2500000 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2500000 source evidence: unavailable".to_string(),
    }
}

pub(crate) fn selected_asteroid_source_2634167_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    reference_asteroids().contains(&entry.body)
                        && entry.epoch.julian_day.days() == SELECTED_ASTEROID_SOURCE_2634167_EPOCH
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

/// Compact release-facing summary for the selected-asteroid 2634167 source evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSource2634167Summary {
    /// Number of exact samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the source slice.
    pub epoch: Instant,
}

/// Validation errors for a selected-asteroid 2634167 source summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidSource2634167SummaryValidationError {
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

impl fmt::Display for SelectedAsteroidSource2634167SummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid 2634167 source evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid 2634167 source evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid 2634167 source evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid 2634167 source evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidSource2634167SummaryValidationError {}

impl SelectedAsteroidSource2634167Summary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference selected-asteroid 2634167 source evidence: {} exact samples at {} ({}); 2634167 source sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidSource2634167SummaryValidationError> {
        let evidence = selected_asteroid_source_2634167_entries()
            .ok_or(SelectedAsteroidSource2634167SummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidSource2634167SummaryValidationError::SampleCountMismatch {
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
                        SelectedAsteroidSource2634167SummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidSource2634167SummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                SelectedAsteroidSource2634167SummaryValidationError::EpochMismatch {
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
    ) -> Result<String, SelectedAsteroidSource2634167SummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidSource2634167Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn selected_asteroid_source_2634167_summary_details(
) -> Option<SelectedAsteroidSource2634167Summary> {
    let evidence = selected_asteroid_source_2634167_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(SelectedAsteroidSource2634167Summary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the selected-asteroid 2634167 source evidence.
pub fn selected_asteroid_source_2634167_summary() -> Option<SelectedAsteroidSource2634167Summary> {
    selected_asteroid_source_2634167_summary_details()
}

/// Returns the release-facing selected-asteroid 2634167 source summary string.
pub fn selected_asteroid_source_2634167_summary_for_report() -> String {
    match selected_asteroid_source_2634167_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Selected asteroid 2634167 source evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid 2634167 source evidence: unavailable".to_string(),
    }
}

pub(crate) fn selected_asteroid_bridge_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    reference_asteroids().contains(&entry.body)
                        && entry.epoch.julian_day.days() == SELECTED_ASTEROID_BRIDGE_EPOCH
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

/// Compact release-facing summary for the selected-asteroid bridge-day evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidBridgeSummary {
    /// Number of exact samples in the bridge slice.
    pub sample_count: usize,
    /// Bodies covered by the bridge slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the bridge slice.
    pub epoch: Instant,
}

/// Validation errors for a selected-asteroid bridge summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidBridgeSummaryValidationError {
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

impl fmt::Display for SelectedAsteroidBridgeSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid bridge evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid bridge evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid bridge evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid bridge evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidBridgeSummaryValidationError {}

impl SelectedAsteroidBridgeSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Selected asteroid bridge evidence: {} exact samples at {} ({}); bridge sample across the asteroid-only gap",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidBridgeSummaryValidationError> {
        let evidence = selected_asteroid_bridge_entries()
            .ok_or(SelectedAsteroidBridgeSummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidBridgeSummaryValidationError::SampleCountMismatch {
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
                        SelectedAsteroidBridgeSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidBridgeSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                SelectedAsteroidBridgeSummaryValidationError::EpochMismatch {
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
    ) -> Result<String, SelectedAsteroidBridgeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidBridgeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn selected_asteroid_bridge_summary_details() -> Option<SelectedAsteroidBridgeSummary> {
    let evidence = selected_asteroid_bridge_entries()?;
    Some(SelectedAsteroidBridgeSummary {
        sample_count: evidence.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the selected-asteroid bridge-day evidence.
pub fn selected_asteroid_bridge_summary() -> Option<SelectedAsteroidBridgeSummary> {
    selected_asteroid_bridge_summary_details()
}

/// Returns the release-facing selected-asteroid bridge-day summary string.
pub fn selected_asteroid_bridge_summary_for_report() -> String {
    match selected_asteroid_bridge_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Selected asteroid bridge evidence: unavailable ({error})"),
        },
        None => "Selected asteroid bridge evidence: unavailable".to_string(),
    }
}

pub(crate) fn selected_asteroid_dense_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    reference_asteroids().contains(&entry.body)
                        && entry.epoch.julian_day.days() == SELECTED_ASTEROID_DENSE_BOUNDARY_EPOCH
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

/// Compact release-facing summary for the dense selected-asteroid boundary day.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidDenseBoundarySummary {
    /// Number of exact samples in the dense boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the dense boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the dense boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a selected-asteroid dense boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidDenseBoundarySummaryValidationError {
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

impl fmt::Display for SelectedAsteroidDenseBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid dense boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid dense boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid dense boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid dense boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidDenseBoundarySummaryValidationError {}

impl SelectedAsteroidDenseBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Selected asteroid dense boundary evidence: {} exact samples at {} ({}); dense boundary day",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidDenseBoundarySummaryValidationError> {
        let evidence = selected_asteroid_dense_boundary_entries()
            .ok_or(SelectedAsteroidDenseBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidDenseBoundarySummaryValidationError::SampleCountMismatch {
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
                        SelectedAsteroidDenseBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidDenseBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                SelectedAsteroidDenseBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, SelectedAsteroidDenseBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SelectedAsteroidDenseBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn selected_asteroid_dense_boundary_summary_details(
) -> Option<SelectedAsteroidDenseBoundarySummary> {
    let evidence = selected_asteroid_dense_boundary_entries()?;
    Some(SelectedAsteroidDenseBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the dense selected-asteroid boundary evidence.
pub fn selected_asteroid_dense_boundary_summary() -> Option<SelectedAsteroidDenseBoundarySummary> {
    selected_asteroid_dense_boundary_summary_details()
}

/// Returns the release-facing dense selected-asteroid boundary summary string.
pub fn selected_asteroid_dense_boundary_summary_for_report() -> String {
    match selected_asteroid_dense_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Selected asteroid dense boundary evidence: unavailable ({error})")
            }
        },
        None => "Selected asteroid dense boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn selected_asteroid_boundary_entries() -> Option<&'static [SnapshotEntry]> {
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

pub(crate) fn selected_asteroid_boundary_summary_details() -> Option<SelectedAsteroidBoundarySummary>
{
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

pub(crate) fn selected_asteroid_terminal_boundary_entries() -> Option<&'static [SnapshotEntry]> {
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

pub(crate) fn selected_asteroid_terminal_boundary_summary_details(
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

pub(crate) fn selected_asteroid_batch_parity_summary_details(
) -> Option<SelectedAsteroidBatchParitySummary> {
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

#[cfg(test)]
mod tests;
