//! Validation corpus definitions and their compact summaries.

use std::fmt;
use std::sync::OnceLock;

use pleiades_core::{
    default_chart_bodies, Apparentness, CelestialBody, CoordinateFrame, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, Instant, JulianDay, TimeScale, ZodiacMode,
};
use pleiades_jpl::comparison_snapshot_requests;

/// A validation corpus made up of request samples.
#[derive(Clone, Debug)]
pub struct ValidationCorpus {
    /// Human-readable corpus name.
    pub name: String,
    /// Short description of what the corpus covers.
    pub description: &'static str,
    /// Apparentness mode used for the requests.
    pub apparentness: Apparentness,
    /// Requests sent to both backends.
    pub requests: Vec<EphemerisRequest>,
}

/// A compact summary of a validation corpus.
#[derive(Clone, Debug)]
pub struct CorpusSummary {
    /// Human-readable corpus name.
    pub name: String,
    /// Short description of what the corpus covers.
    pub description: &'static str,
    /// Apparentness mode used for the corpus requests.
    pub apparentness: Apparentness,
    /// Total number of requests in the corpus.
    pub request_count: usize,
    /// Number of unique instants covered by the corpus.
    pub epoch_count: usize,
    /// Unique instants covered by the corpus, preserved in chronological order.
    pub epochs: Vec<Instant>,
    /// Number of unique bodies covered by the corpus.
    pub body_count: usize,
    /// Earliest Julian day in the corpus.
    pub earliest_julian_day: f64,
    /// Latest Julian day in the corpus.
    pub latest_julian_day: f64,
}

impl CorpusSummary {
    /// Returns a compact one-line summary used by release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "corpus name={} apparentness={} requests={} epochs={} bodies={} julian day span={:.1} → {:.1}",
            self.name,
            self.apparentness,
            self.request_count,
            self.epoch_count,
            self.body_count,
            self.earliest_julian_day,
            self.latest_julian_day,
        )
    }

    /// Returns `Ok(())` when the corpus summary is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary name must not be blank",
            ));
        }

        if self.description.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary description must not be blank",
            ));
        }

        if self.request_count == 0 {
            if self.epoch_count != 0 || self.body_count != 0 || !self.epochs.is_empty() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "corpus summary with no requests must also have no epochs or bodies",
                ));
            }
        } else if self.epoch_count == 0 || self.body_count == 0 || self.epochs.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary with requests must also have epochs and bodies",
            ));
        }

        if self.epoch_count != self.epochs.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "corpus summary epoch-count mismatch: expected {}, found {}",
                    self.epoch_count,
                    self.epochs.len()
                ),
            ));
        }

        for (index, epoch) in self.epochs.iter().enumerate() {
            if !epoch.julian_day.days().is_finite() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "corpus summary epoch {} has a non-finite Julian day",
                        index + 1
                    ),
                ));
            }

            if self.epochs[..index]
                .iter()
                .any(|prior| prior.julian_day.days() >= epoch.julian_day.days())
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "corpus summary epochs must be strictly increasing: epoch {} is out of order",
                        index + 1
                    ),
                ));
            }
        }

        match self.epochs.first() {
            Some(first) => {
                if self.earliest_julian_day != first.julian_day.days() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        "corpus summary earliest Julian day does not match the first epoch",
                    ));
                }
            }
            None => {
                if self.earliest_julian_day != 0.0 || self.latest_julian_day != 0.0 {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        "corpus summary with no epochs must have a zero Julian day span",
                    ));
                }
            }
        }

        if let Some(last) = self.epochs.last() {
            if self.latest_julian_day != last.julian_day.days() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "corpus summary latest Julian day does not match the final epoch",
                ));
            }
        }

        if self.request_count > 0 && self.body_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "corpus summary with requests must cover at least one body",
            ));
        }

        Ok(())
    }
}

impl fmt::Display for CorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl ValidationCorpus {
    /// Creates the default JPL snapshot corpus.
    pub fn jpl_snapshot() -> Self {
        let requests = comparison_snapshot_requests(CoordinateFrame::Ecliptic)
            .expect("comparison snapshot requests should exist");

        Self {
            name: "JPL Horizons comparison window".to_string(),
            description: "Source-backed comparison corpus built from the checked-in JPL Horizons snapshot across a small set of reference epochs, restricted to the bodies shared by the algorithmic comparison backend.",
            apparentness: Apparentness::Mean,
            requests,
        }
    }

    /// Creates a representative benchmark corpus spanning the target 1500-2500 window.
    pub fn representative_window() -> Self {
        let bodies = default_chart_bodies();
        let instants = [
            Instant::new(JulianDay::from_days(2_268_559.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_268_924.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_305_448.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_329_555.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_390_550.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_512_176.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_573_171.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_597_642.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tt),
            Instant::new(JulianDay::from_days(2_634_532.0), TimeScale::Tt),
        ];

        Self::from_epochs(
            "Representative 1500-2500 window",
            "Eleven-epoch benchmark corpus that broadens the representative sweep with explicit guard epochs just outside the target span and mid-window coverage.",
            Apparentness::Mean,
            &instants,
            bodies,
        )
    }

    /// Returns a compact metadata summary for display purposes.
    pub fn summary(&self) -> CorpusSummary {
        let mut epochs = self
            .requests
            .iter()
            .map(|request| request.instant)
            .collect::<Vec<_>>();
        epochs.sort_by(|left, right| {
            left.julian_day
                .days()
                .partial_cmp(&right.julian_day.days())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        epochs.dedup();

        let mut bodies = Vec::new();
        for request in &self.requests {
            if !bodies.contains(&request.body) {
                bodies.push(request.body.clone());
            }
        }

        let earliest_julian_day = epochs
            .first()
            .map(|instant| instant.julian_day.days())
            .unwrap_or_default();
        let latest_julian_day = epochs
            .last()
            .map(|instant| instant.julian_day.days())
            .unwrap_or_default();

        CorpusSummary {
            name: self.name.clone(),
            description: self.description,
            apparentness: self.apparentness,
            request_count: self.requests.len(),
            epoch_count: epochs.len(),
            epochs,
            body_count: bodies.len(),
            earliest_julian_day,
            latest_julian_day,
        }
    }

    /// Returns an estimated heap footprint for the corpus data in bytes.
    pub fn estimated_heap_bytes(&self) -> usize {
        self.requests
            .capacity()
            .saturating_mul(std::mem::size_of::<EphemerisRequest>())
            .saturating_add(self.name.capacity())
    }

    fn from_epochs(
        name: impl Into<String>,
        description: &'static str,
        apparentness: Apparentness,
        instants: &[Instant],
        bodies: &[CelestialBody],
    ) -> Self {
        let requests = instants
            .iter()
            .copied()
            .flat_map(|instant| {
                bodies.iter().cloned().map(move |body| EphemerisRequest {
                    body,
                    instant,
                    observer: None,
                    frame: CoordinateFrame::Ecliptic,
                    zodiac_mode: ZodiacMode::Tropical,
                    apparent: apparentness,
                })
            })
            .collect();

        Self {
            name: name.into(),
            description,
            apparentness,
            requests,
        }
    }
}

/// Builds the default validation corpus.
pub fn default_corpus() -> ValidationCorpus {
    static CACHE: OnceLock<ValidationCorpus> = OnceLock::new();

    CACHE.get_or_init(ValidationCorpus::jpl_snapshot).clone()
}

/// Builds the release-grade comparison corpus with Pluto excluded from tolerance evidence.
pub fn release_grade_corpus() -> ValidationCorpus {
    static CACHE: OnceLock<ValidationCorpus> = OnceLock::new();

    CACHE
        .get_or_init(|| {
            let mut corpus = default_corpus();
            corpus.name = "JPL Horizons release-grade comparison window".to_string();
            corpus.description = "Release-grade comparison corpus built from the checked-in JPL Horizons snapshot, with Pluto excluded from tolerance evidence because Pluto remains an approximate fallback.";
            corpus
                .requests
                .retain(|request| request.body != CelestialBody::Pluto);
            corpus
        })
        .clone()
}

/// Creates the default benchmark corpus.
pub fn benchmark_corpus() -> ValidationCorpus {
    static CACHE: OnceLock<ValidationCorpus> = OnceLock::new();

    CACHE
        .get_or_init(ValidationCorpus::representative_window)
        .clone()
}

pub(crate) fn benchmark_timing_corpus() -> ValidationCorpus {
    static CACHE: OnceLock<ValidationCorpus> = OnceLock::new();

    CACHE
        .get_or_init(|| {
            ValidationCorpus::from_epochs(
                "Representative 1500-2500 window",
                "Reduced timing subset of the representative 1500-2500 benchmark corpus.",
                Apparentness::Mean,
                &[Instant::new(
                    JulianDay::from_days(2_451_545.0),
                    TimeScale::Tt,
                )],
                &[CelestialBody::Sun],
            )
        })
        .clone()
}
