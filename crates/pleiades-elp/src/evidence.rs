use core::fmt;

use pleiades_backend::{
    Apparentness, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
};
use pleiades_types::{
    Angle, CelestialBody, CoordinateFrame, EquatorialCoordinates, Instant, Latitude, TimeRange,
    TimeScale, ZodiacMode,
};

use crate::backend::ElpBackend;
use crate::series::signed_longitude_delta_degrees;

/// A single canonical lunar evidence sample used by validation and reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarReferenceSample {
    /// Body or lunar point covered by the sample.
    pub body: CelestialBody,
    /// Reference epoch used for the sample.
    pub epoch: Instant,
    /// Ecliptic longitude in degrees.
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees.
    pub latitude_deg: f64,
    /// Geocentric distance in astronomical units, if available.
    pub distance_au: Option<f64>,
    /// Human-readable note describing the provenance of the sample.
    pub note: &'static str,
}

impl LunarReferenceSample {
    /// Returns `Ok(())` when the sample still represents a valid lunar evidence row.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if !matches!(
            self.body,
            CelestialBody::Moon
                | CelestialBody::MeanNode
                | CelestialBody::TrueNode
                | CelestialBody::MeanApogee
                | CelestialBody::MeanPerigee
        ) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample must use a supported lunar body",
            ));
        }
        if self.epoch.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample must use TT epochs",
            ));
        }
        if !self.longitude_deg.is_finite() || !self.latitude_deg.is_finite() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample must use finite ecliptic coordinates",
            ));
        }
        if let Some(distance_au) = self.distance_au {
            if !distance_au.is_finite() || distance_au <= 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "lunar reference sample must use a positive finite distance when present",
                ));
            }
        }
        if self.note.trim().is_empty() || self.note.trim() != self.note {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample note must be a non-blank canonical line",
            ));
        }

        Ok(())
    }

    /// Returns a compact, release-facing summary of the published sample.
    pub fn summary_line(&self) -> String {
        format!(
            "{} at JD {:.1}: lon={:.12}°, lat={:.12}°, dist={}, note={}",
            self.body,
            self.epoch.julian_day.days(),
            self.longitude_deg,
            self.latitude_deg,
            self.distance_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.note,
        )
    }
}

impl fmt::Display for LunarReferenceSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the canonical lunar evidence samples used by validation and reporting.
pub fn lunar_reference_evidence() -> &'static [LunarReferenceSample] {
    const SAMPLES: &[LunarReferenceSample] = &[
        LunarReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            longitude_deg: 133.162_655,
            latitude_deg: -3.229_126,
            distance_au: Some(368_409.7 / 149_597_870.700),
            note: "Published 1992-04-12 geocentric Moon example used as the compact lunar baseline regression sample",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(crate::J2000), TimeScale::Tt),
            longitude_deg: 125.044_547_9,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 mean node reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::TrueNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(crate::J2000), TimeScale::Tt),
            longitude_deg: 123.926_171_368_400_46,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 true node reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_419_914.5), TimeScale::Tt),
            longitude_deg: 0.0,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 1913-05-27 mean ascending node example used to anchor the lunar node model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_436_909.5), TimeScale::Tt),
            longitude_deg: 180.0,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 1959-12-07 mean ascending node example used to anchor the lunar node model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanPerigee,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_459_278.5), TimeScale::Tt),
            longitude_deg: 224.891_94,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 2021-03-05 mean perigee example used to anchor the lunar perigee model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanPerigee,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(crate::J2000), TimeScale::Tt),
            longitude_deg: 83.353_246_5,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 mean perigee reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanApogee,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(crate::J2000), TimeScale::Tt),
            longitude_deg: 263.353_246_5,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 mean apogee reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::TrueNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_419_914.5), TimeScale::Tt),
            longitude_deg: 0.876_3,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 1913-05-27 true ascending node example used to anchor the lunar node model",
        },
    ];

    SAMPLES
}

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
pub fn lunar_reference_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_reference_evidence()
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let mut request = EphemerisRequest::new(sample.body.clone(), sample.epoch);
            request.instant.scale = if index % 2 == 0 {
                TimeScale::Tt
            } else {
                TimeScale::Tdb
            };
            request
        })
        .collect()
}

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
///
/// This is a compatibility alias for [`lunar_reference_batch_parity_requests`].
#[doc(alias = "lunar_reference_batch_parity_requests")]
pub fn lunar_reference_batch_requests() -> Vec<EphemerisRequest> {
    lunar_reference_batch_parity_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
///
/// This is a compatibility alias for [`lunar_reference_batch_parity_requests`].
#[doc(alias = "lunar_reference_batch_parity_requests")]
pub fn lunar_reference_batch_request_corpus() -> Vec<EphemerisRequest> {
    lunar_reference_batch_parity_requests()
}

/// Returns the canonical lunar request corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_reference_batch_requests`].
#[doc(alias = "lunar_reference_requests")]
#[doc(alias = "lunar_reference_batch_requests")]
#[doc(alias = "lunar_reference_batch_request_corpus")]
pub fn lunar_reference_request_corpus() -> Vec<EphemerisRequest> {
    lunar_reference_batch_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
///
/// This is a compatibility alias for [`lunar_reference_batch_parity_requests`].
#[doc(alias = "lunar_reference_batch_parity_requests")]
pub fn lunar_reference_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_reference_batch_parity_requests()
}

/// A single canonical lunar equatorial evidence sample used by validation and reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarEquatorialReferenceSample {
    /// Body covered by the sample.
    pub body: CelestialBody,
    /// Reference epoch used for the sample.
    pub epoch: Instant,
    /// Expected equatorial coordinates.
    pub equatorial: EquatorialCoordinates,
    /// Human-readable note describing the provenance of the sample.
    pub note: &'static str,
}

impl LunarEquatorialReferenceSample {
    /// Returns `Ok(())` when the sample still represents a valid lunar equatorial evidence row.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body != CelestialBody::Moon {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample must use the Moon body",
            ));
        }
        if self.epoch.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample must use TT epochs",
            ));
        }
        if !self.equatorial.right_ascension.degrees().is_finite()
            || !self.equatorial.declination.degrees().is_finite()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample must use finite equatorial coordinates",
            ));
        }
        if let Some(distance_au) = self.equatorial.distance_au {
            if !distance_au.is_finite() || distance_au <= 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "lunar equatorial reference sample must use a positive finite distance when present",
                ));
            }
        }
        if self.note.trim().is_empty() || self.note.trim() != self.note {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample note must be a non-blank canonical line",
            ));
        }

        Ok(())
    }

    /// Returns a compact, release-facing summary of the published sample.
    pub fn summary_line(&self) -> String {
        format!(
            "{} at JD {:.1}: ra={:.12}°, dec={:.12}°, dist={}, note={}",
            self.body,
            self.epoch.julian_day.days(),
            self.equatorial.right_ascension.degrees(),
            self.equatorial.declination.degrees(),
            self.equatorial
                .distance_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.note,
        )
    }
}

impl fmt::Display for LunarEquatorialReferenceSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the canonical lunar equatorial evidence samples used by validation and reporting.
pub fn lunar_equatorial_reference_evidence() -> &'static [LunarEquatorialReferenceSample] {
    const SAMPLES: &[LunarEquatorialReferenceSample] = &[
        LunarEquatorialReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            equatorial: EquatorialCoordinates::new(
                Angle::from_degrees(134.688_470),
                Latitude::from_degrees(13.768_368),
                Some(368_409.7 / 149_597_870.700),
            ),
            note: "Published 1992-04-12 geocentric Moon RA/Dec example used to anchor the mean-obliquity equatorial transform",
        },
        LunarEquatorialReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            equatorial: EquatorialCoordinates::new(
                Angle::from_degrees(134.683_861_811_039_18),
                Latitude::from_degrees(13.769_414_994_266_76),
                Some(368_409.7 / 149_597_870.700),
            ),
            note: "Derived equatorial companion from the published 1992-04-12 geocentric Moon example using the shared mean-obliquity transform",
        },
        LunarEquatorialReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            equatorial: EquatorialCoordinates::new(
                Angle::from_degrees(134.688_469),
                Latitude::from_degrees(13.768_367),
                Some(368_409.7 / 149_597_870.700),
            ),
            note: "Reference-only published 1992-04-12 apparent geocentric Moon comparison datum reused as an additional equatorial cross-check",
        },
    ];

    SAMPLES
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
pub fn lunar_equatorial_reference_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_evidence()
        .iter()
        .map(|sample| {
            let mut request = EphemerisRequest::new(sample.body.clone(), sample.epoch);
            request.frame = CoordinateFrame::Equatorial;
            request.instant.scale = TimeScale::Tt;
            request.zodiac_mode = ZodiacMode::Tropical;
            request.apparent = Apparentness::Mean;
            request
        })
        .collect()
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_parity_requests`].
#[doc(alias = "lunar_equatorial_reference_batch_parity_requests")]
pub fn lunar_equatorial_reference_batch_requests() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_parity_requests()
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_parity_requests`].
#[doc(alias = "lunar_equatorial_reference_batch_parity_requests")]
pub fn lunar_equatorial_reference_batch_request_corpus() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_parity_requests()
}

/// Returns the canonical equatorial lunar request corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_requests`].
#[doc(alias = "lunar_equatorial_reference_requests")]
#[doc(alias = "lunar_equatorial_reference_batch_requests")]
#[doc(alias = "lunar_equatorial_reference_batch_request_corpus")]
pub fn lunar_equatorial_reference_request_corpus() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_requests()
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_parity_requests`].
#[doc(alias = "lunar_equatorial_reference_batch_parity_requests")]
pub fn lunar_equatorial_reference_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_parity_requests()
}

/// A compact summary of the canonical lunar equatorial reference batch-parity slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarEquatorialReferenceBatchParitySummary {
    /// Number of evidence samples in the checked-in batch slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the batch slice.
    pub body_count: usize,
    /// Coordinate frame covered by the batch slice.
    pub frame: CoordinateFrame,
    /// Whether the batch results preserved request order.
    pub order_preserved: bool,
    /// Whether batch results matched the corresponding single-query lookups.
    pub single_query_parity: bool,
}

/// Validation error for a lunar equatorial batch-parity summary that drifted from the current reference evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarEquatorialReferenceBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current evidence slice.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarEquatorialReferenceBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar equatorial reference batch parity summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for LunarEquatorialReferenceBatchParitySummaryValidationError {}

/// Returns a compact summary of the canonical lunar equatorial reference batch-parity slice.
pub fn lunar_equatorial_reference_batch_parity_summary(
) -> Option<LunarEquatorialReferenceBatchParitySummary> {
    let requests = lunar_equatorial_reference_batch_requests();
    if requests.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let bodies = requests
        .iter()
        .map(|request| request.body.to_string())
        .collect::<std::collections::BTreeSet<_>>();

    let results = backend.positions(&requests).ok()?;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for (request, result) in requests.iter().zip(results.iter()) {
        order_preserved &= result.body == request.body;

        let single = backend.position(request).ok()?;
        single_query_parity &= result.body == single.body
            && result.instant == single.instant
            && result.frame == single.frame
            && result.quality == single.quality
            && result.ecliptic == single.ecliptic
            && result.equatorial == single.equatorial
            && result.motion == single.motion;
    }

    Some(LunarEquatorialReferenceBatchParitySummary {
        sample_count: requests.len(),
        body_count: bodies.len(),
        frame: CoordinateFrame::Equatorial,
        order_preserved,
        single_query_parity,
    })
}

impl LunarEquatorialReferenceBatchParitySummary {
    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(
        &self,
    ) -> Result<(), LunarEquatorialReferenceBatchParitySummaryValidationError> {
        let samples = lunar_equatorial_reference_evidence();
        let expected_sample_count = samples.len();
        let expected_body_count = samples
            .iter()
            .map(|sample| sample.body.to_string())
            .collect::<std::collections::BTreeSet<_>>()
            .len();

        if self.sample_count != expected_sample_count {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != expected_body_count {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "frame",
                },
            );
        }
        if !self.order_preserved {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "order_preserved",
                },
            );
        }
        if !self.single_query_parity {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "single_query_parity",
                },
            );
        }

        Ok(())
    }

    /// Returns the release-facing one-line lunar equatorial batch parity summary.
    pub fn summary_line(&self) -> String {
        let order = if self.order_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        let parity = if self.single_query_parity {
            "preserved"
        } else {
            "needs attention"
        };

        format!(
            "lunar equatorial reference batch parity: {} requests across {} bodies, frame={}, order={}, single-query parity={}",
            self.sample_count,
            self.body_count,
            self.frame,
            order,
            parity,
        )
    }
}

impl fmt::Display for LunarEquatorialReferenceBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar equatorial batch-parity evidence for release-facing reporting.
pub fn format_lunar_equatorial_reference_batch_parity_summary(
    summary: &LunarEquatorialReferenceBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar equatorial batch-parity summary string.
pub fn lunar_equatorial_reference_batch_parity_summary_for_report() -> String {
    match lunar_equatorial_reference_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_equatorial_reference_batch_parity_summary(&summary),
            Err(error) => {
                format!("lunar equatorial reference batch parity: unavailable ({error})")
            }
        },
        None => "lunar equatorial reference batch parity: unavailable".to_string(),
    }
}

/// A compact summary of the canonical lunar reference evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarReferenceEvidenceSummary {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
}

/// Returns a compact summary of the canonical lunar reference evidence slice.
pub fn lunar_reference_evidence_summary() -> Option<LunarReferenceEvidenceSummary> {
    let samples = lunar_reference_evidence();
    if samples.is_empty() || samples.iter().any(|sample| sample.validate().is_err()) {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
    }

    Some(LunarReferenceEvidenceSummary {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
    })
}

impl LunarReferenceEvidenceSummary {
    /// Returns the release-facing one-line lunar reference evidence summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar reference evidence: {} samples across {} bodies, epoch range {}, validated against the published 1992-04-12 Moon example plus J2000 lunar-point anchors, including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example",
            self.sample_count,
            self.body_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let Some(expected) = lunar_reference_evidence_summary() else {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference evidence is unavailable",
            ));
        };

        if self != &expected {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "lunar reference evidence summary mismatch: expected {}, found {}",
                    expected.summary_line(),
                    self.summary_line()
                ),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for LunarReferenceEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar reference evidence summary for release-facing reporting.
pub fn format_lunar_reference_evidence_summary(summary: &LunarReferenceEvidenceSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar reference evidence summary string.
pub fn lunar_reference_evidence_summary_for_report() -> String {
    match lunar_reference_evidence_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_reference_evidence_summary(&summary),
            Err(error) => format!("lunar reference evidence: unavailable ({error})"),
        },
        None => "lunar reference evidence: unavailable".to_string(),
    }
}

fn lunar_exact_source_window_requests() -> Vec<EphemerisRequest> {
    let mut requests = lunar_reference_evidence()
        .iter()
        .filter(|sample| sample.body == CelestialBody::Moon)
        .map(|sample| EphemerisRequest::new(sample.body.clone(), sample.epoch))
        .collect::<Vec<_>>();
    requests.extend(lunar_high_curvature_continuity_requests());
    requests
}

fn lunar_source_window_requests() -> Vec<EphemerisRequest> {
    let mut requests = lunar_exact_source_window_requests();
    requests.extend(lunar_apparent_comparison_requests());
    requests
}

/// This is a compatibility alias for `lunar_source_window_requests`.
#[doc(alias = "lunar_source_window_requests")]
pub fn lunar_source_window_request_corpus() -> Vec<EphemerisRequest> {
    lunar_source_window_requests()
}

/// A compact summary of the broader lunar source-window evidence slice.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarSourceWindowSummary {
    /// Number of exact Moon samples across the exact source windows.
    pub exact_sample_count: usize,
    /// Number of reference-only apparent Moon samples across the apparent source windows.
    pub apparent_sample_count: usize,
    /// Number of distinct bodies covered by the broader source windows.
    pub body_count: usize,
    /// Number of exact source windows contributing to the broader slice.
    pub exact_window_count: usize,
    /// Number of apparent source windows contributing to the broader slice.
    pub apparent_window_count: usize,
    /// Earliest epoch represented in the broader source windows.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the broader source windows.
    pub latest_epoch: Instant,
}

/// Validation error for a lunar source-window summary that drifted from the current evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarSourceWindowSummaryValidationError {
    /// The summary no longer matches the current lunar source-window evidence.
    Unavailable,
    /// A rendered summary field no longer matches the current lunar source-window evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => write!(
                f,
                "the lunar source-window summary is unavailable from the current evidence"
            ),
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar source-window summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for LunarSourceWindowSummaryValidationError {}

impl LunarSourceWindowSummary {
    /// Returns the release-facing one-line lunar source-window summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar source windows: {} exact Moon samples across {} bodies in {} exact windows; {} reference-only apparent Moon samples across {} bodies in {} apparent windows, epoch range {}; exact windows: published 1992-04-12 geocentric Moon example; J2000 high-curvature continuity window; apparent windows: published 1992-04-12 apparent geocentric Moon comparison datum; published 1968-12-24 low-accuracy Meeus-style geocentric Moon example; published 2004-04-01 NASA RP 1349 apparent Moon table row; published 2006-09-07 EclipseWise apparent Moon coordinate row",
            self.exact_sample_count,
            self.body_count,
            self.exact_window_count,
            self.apparent_sample_count,
            self.body_count,
            self.apparent_window_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current lunar evidence.
    pub fn validate(&self) -> Result<(), LunarSourceWindowSummaryValidationError> {
        let Some(current) = lunar_source_window_summary() else {
            return Err(LunarSourceWindowSummaryValidationError::Unavailable);
        };

        if self.exact_sample_count != current.exact_sample_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "exact_sample_count",
            });
        }
        if self.apparent_sample_count != current.apparent_sample_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "apparent_sample_count",
            });
        }
        if self.body_count != current.body_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "body_count",
            });
        }
        if self.exact_window_count != current.exact_window_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "exact_window_count",
            });
        }
        if self.apparent_window_count != current.apparent_window_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "apparent_window_count",
            });
        }
        if self.earliest_epoch != current.earliest_epoch {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "earliest_epoch",
            });
        }
        if self.latest_epoch != current.latest_epoch {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "latest_epoch",
            });
        }

        Ok(())
    }

    /// Returns the release-facing one-line summary when the current evidence still matches.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarSourceWindowSummaryValidationError> {
        self.validate().map(|()| self.summary_line())
    }
}

impl fmt::Display for LunarSourceWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a compact summary of the broader lunar source-window evidence slice.
pub fn lunar_source_window_summary() -> Option<LunarSourceWindowSummary> {
    let exact_requests = lunar_exact_source_window_requests();
    let apparent_samples = lunar_apparent_comparison_evidence();
    if exact_requests.is_empty() && apparent_samples.is_empty() {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = exact_requests
        .first()
        .map(|request| request.instant)
        .or_else(|| apparent_samples.first().map(|sample| sample.epoch))
        .expect("lunar source window evidence should not be empty after guard");
    let mut latest_epoch = earliest_epoch;

    for request in &exact_requests {
        bodies.insert(request.body.to_string());
        if request.instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = request.instant;
        }
        if request.instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = request.instant;
        }
    }

    for sample in apparent_samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
    }

    Some(LunarSourceWindowSummary {
        exact_sample_count: exact_requests.len(),
        apparent_sample_count: apparent_samples.len(),
        body_count: bodies.len(),
        exact_window_count: 2,
        apparent_window_count: apparent_samples.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Formats the broader lunar source-window summary for release-facing reporting.
fn format_validated_lunar_source_window_summary_for_report(
    summary: &LunarSourceWindowSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar source windows: unavailable ({error})"),
    }
}

/// Formats the broader lunar source-window summary for release-facing reporting.
pub fn format_lunar_source_window_summary(summary: &LunarSourceWindowSummary) -> String {
    format_validated_lunar_source_window_summary_for_report(summary)
}

/// Returns the validated release-facing broader lunar source-window summary string.
pub fn validated_lunar_source_window_summary_for_report() -> Result<String, String> {
    lunar_source_window_summary()
        .ok_or_else(|| {
            "the lunar source-window summary is unavailable from the current evidence".to_string()
        })
        .and_then(|summary| {
            summary
                .validated_summary_line()
                .map_err(|error| error.to_string())
        })
}

/// Returns the release-facing broader lunar source-window summary string.
pub fn lunar_source_window_summary_for_report() -> String {
    match lunar_source_window_summary() {
        Some(summary) => format_validated_lunar_source_window_summary_for_report(&summary),
        None => "lunar source windows: unavailable".to_string(),
    }
}

/// A compact summary of the mixed TT/TDB lunar reference batch-parity evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarReferenceBatchParitySummary {
    /// Number of batch requests in the mixed-scale regression slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the mixed-scale regression slice.
    pub body_count: usize,
    /// Number of TT-tagged requests in the mixed-scale regression slice.
    pub tt_request_count: usize,
    /// Number of TDB-tagged requests in the mixed-scale regression slice.
    pub tdb_request_count: usize,
    /// Whether the batch results preserved request order.
    pub order_preserved: bool,
    /// Whether batch results matched the corresponding single-query lookups.
    pub single_query_parity: bool,
}

/// Validation error for a lunar mixed TT/TDB batch-parity summary that drifted
/// from the current reference evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarReferenceBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current evidence slice.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarReferenceBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar reference mixed TT/TDB batch parity summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for LunarReferenceBatchParitySummaryValidationError {}

/// Returns a compact summary of the mixed TT/TDB lunar reference batch parity slice.
pub fn lunar_reference_batch_parity_summary() -> Option<LunarReferenceBatchParitySummary> {
    let requests = lunar_reference_batch_requests();
    if requests.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let bodies = requests
        .iter()
        .map(|request| request.body.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let tt_request_count = requests
        .iter()
        .filter(|request| request.instant.scale == TimeScale::Tt)
        .count();
    let tdb_request_count = requests
        .iter()
        .filter(|request| request.instant.scale == TimeScale::Tdb)
        .count();

    let results = backend.positions(&requests).ok()?;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for (request, result) in requests.iter().zip(results.iter()) {
        order_preserved &= result.body == request.body;

        let single = backend.position(request).ok()?;
        single_query_parity &= result.body == single.body
            && result.instant == single.instant
            && result.frame == single.frame
            && result.quality == single.quality
            && result.ecliptic == single.ecliptic
            && result.equatorial == single.equatorial
            && result.motion == single.motion;
    }

    Some(LunarReferenceBatchParitySummary {
        sample_count: requests.len(),
        body_count: bodies.len(),
        tt_request_count,
        tdb_request_count,
        order_preserved,
        single_query_parity,
    })
}

impl LunarReferenceBatchParitySummary {
    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(&self) -> Result<(), LunarReferenceBatchParitySummaryValidationError> {
        let samples = lunar_reference_evidence();
        let expected_sample_count = samples.len();
        let expected_body_count = samples
            .iter()
            .map(|sample| sample.body.to_string())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let expected_tt_request_count = expected_sample_count.div_ceil(2);
        let expected_tdb_request_count = expected_sample_count / 2;

        if self.sample_count != expected_sample_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != expected_body_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.tt_request_count != expected_tt_request_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tt_request_count",
                },
            );
        }
        if self.tdb_request_count != expected_tdb_request_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tdb_request_count",
                },
            );
        }
        if !self.order_preserved {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "order_preserved",
                },
            );
        }
        if !self.single_query_parity {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "single_query_parity",
                },
            );
        }

        Ok(())
    }

    /// Returns the release-facing one-line lunar mixed-scale batch parity summary.
    pub fn summary_line(&self) -> String {
        let order = if self.order_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        let parity = if self.single_query_parity {
            "preserved"
        } else {
            "needs attention"
        };

        format!(
            "lunar reference mixed TT/TDB batch parity: {} requests across {} bodies, TT requests={}, TDB requests={}, order={}, single-query parity={}",
            self.sample_count,
            self.body_count,
            self.tt_request_count,
            self.tdb_request_count,
            order,
            parity,
        )
    }
}

impl fmt::Display for LunarReferenceBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar mixed TT/TDB batch-parity evidence for release-facing reporting.
pub fn format_lunar_reference_batch_parity_summary(
    summary: &LunarReferenceBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar mixed TT/TDB batch-parity summary string.
pub fn lunar_reference_batch_parity_summary_for_report() -> String {
    match lunar_reference_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_reference_batch_parity_summary(&summary),
            Err(error) => {
                format!("lunar reference mixed TT/TDB batch parity: unavailable ({error})")
            }
        },
        None => "lunar reference mixed TT/TDB batch parity: unavailable".to_string(),
    }
}

/// A compact summary of the canonical lunar equatorial reference evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarEquatorialReferenceEvidenceSummary {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
}

/// Returns a compact summary of the canonical lunar equatorial reference evidence slice.
pub fn lunar_equatorial_reference_evidence_summary(
) -> Option<LunarEquatorialReferenceEvidenceSummary> {
    let samples = lunar_equatorial_reference_evidence();
    if samples.is_empty() || samples.iter().any(|sample| sample.validate().is_err()) {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
    }

    Some(LunarEquatorialReferenceEvidenceSummary {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
    })
}

impl LunarEquatorialReferenceEvidenceSummary {
    /// Returns the release-facing one-line lunar equatorial reference evidence summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar equatorial reference evidence: {} samples across {} bodies, epoch range {}, validated against the published 1992-04-12 geocentric Moon RA/Dec example, a derived 1992 equatorial companion built from the published 1992 geocentric Moon example via the shared mean-obliquity transform, and a reference-only published 1992-04-12 apparent geocentric Moon comparison datum",
            self.sample_count,
            self.body_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let Some(expected) = lunar_equatorial_reference_evidence_summary() else {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference evidence is unavailable",
            ));
        };

        if self != &expected {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "lunar equatorial reference evidence summary mismatch: expected {}, found {}",
                    expected.summary_line(),
                    self.summary_line()
                ),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for LunarEquatorialReferenceEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar equatorial reference evidence summary for release-facing reporting.
pub fn format_lunar_equatorial_reference_evidence_summary(
    summary: &LunarEquatorialReferenceEvidenceSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar equatorial reference evidence summary string.
pub fn lunar_equatorial_reference_evidence_summary_for_report() -> String {
    match lunar_equatorial_reference_evidence_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_equatorial_reference_evidence_summary(&summary),
            Err(error) => format!("lunar equatorial reference evidence: unavailable ({error})"),
        },
        None => "lunar equatorial reference evidence: unavailable".to_string(),
    }
}

/// A reference-only apparent Moon comparison sample.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarApparentComparisonSample {
    /// Body covered by the sample.
    pub body: CelestialBody,
    /// Reference epoch used for the sample.
    pub epoch: Instant,
    /// Published apparent ecliptic longitude in degrees.
    pub apparent_longitude_deg: f64,
    /// Published apparent ecliptic latitude in degrees.
    pub apparent_latitude_deg: f64,
    /// Published apparent geocentric distance in astronomical units.
    pub apparent_distance_au: f64,
    /// Published apparent right ascension in degrees.
    pub apparent_right_ascension_deg: f64,
    /// Published apparent declination in degrees.
    pub apparent_declination_deg: f64,
    /// Human-readable note describing the provenance of the sample.
    pub note: &'static str,
}

impl LunarApparentComparisonSample {
    /// Returns `Ok(())` when the sample still represents a valid apparent Moon evidence row.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body != CelestialBody::Moon {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample must use the Moon body",
            ));
        }
        if self.epoch.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample must use TT epochs",
            ));
        }
        if !self.apparent_longitude_deg.is_finite()
            || !self.apparent_latitude_deg.is_finite()
            || !self.apparent_distance_au.is_finite()
            || self.apparent_distance_au <= 0.0
            || !self.apparent_right_ascension_deg.is_finite()
            || !self.apparent_declination_deg.is_finite()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample must use finite apparent coordinates",
            ));
        }
        if self.note.trim().is_empty() || self.note.trim() != self.note {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample note must be a non-blank canonical line",
            ));
        }

        Ok(())
    }

    /// Returns a compact, release-facing summary of the published sample.
    pub fn summary_line(&self) -> String {
        format!(
            "body={}; epoch={}; apparent lon/lat/dist={:+.6}°/{:+.6}°/{:.12} AU; apparent RA/Dec={:+.6}°/{:+.6}°; note={}",
            self.body,
            crate::format_instant(self.epoch),
            self.apparent_longitude_deg,
            self.apparent_latitude_deg,
            self.apparent_distance_au,
            self.apparent_right_ascension_deg,
            self.apparent_declination_deg,
            self.note,
        )
    }
}

impl fmt::Display for LunarApparentComparisonSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn lunar_apparent_comparison_requests_for_frame(frame: CoordinateFrame) -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_evidence()
        .iter()
        .map(|sample| {
            let mut request = EphemerisRequest::new(sample.body.clone(), sample.epoch);
            request.frame = frame;
            request.instant.scale = TimeScale::Tt;
            request.zodiac_mode = ZodiacMode::Tropical;
            request.apparent = Apparentness::Mean;
            request
        })
        .collect()
}

/// Returns the reference-only apparent Moon comparison sample used by validation and reporting.
pub fn lunar_apparent_comparison_evidence() -> &'static [LunarApparentComparisonSample] {
    const SAMPLES: &[LunarApparentComparisonSample] = &[
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            apparent_longitude_deg: 133.167_264,
            apparent_latitude_deg: -3.229_126,
            apparent_distance_au: 368_409.7 / 149_597_870.700,
            apparent_right_ascension_deg: 134.688_469,
            apparent_declination_deg: 13.768_367,
            note: "Published 1992-04-12 apparent geocentric Moon example used as a reference-only mean/apparent comparison datum",
        },
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(
                pleiades_types::JulianDay::from_days(2_440_214.916_7),
                TimeScale::Tt,
            ),
            apparent_longitude_deg: 336.242_307,
            apparent_latitude_deg: -2.480_685,
            apparent_distance_au: 376_090.0 / 149_597_870.700,
            apparent_right_ascension_deg: 331.293_23,
            apparent_declination_deg: -14.412_95,
            note: "Published 1968-12-24 low-accuracy Meeus-style geocentric Moon example at 10:00 UT used as a second reference-only mean/apparent comparison datum",
        },
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(
                pleiades_types::JulianDay::from_days(2_453_100.5),
                TimeScale::Tt,
            ),
            apparent_longitude_deg: 135.576_300,
            apparent_latitude_deg: 5.203_935,
            apparent_distance_au: 391_200.0 / 149_597_870.700,
            apparent_right_ascension_deg: 139.685_833_333_333_33,
            apparent_declination_deg: 21.130_333_333_333_333,
            note: "Published 2004-04-01 geocentric Moon table row from NASA RP 1349; apparent equatorial coordinates are published directly and the ecliptic row is derived from them using the shared mean-obliquity transform",
        },
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(
                pleiades_types::JulianDay::from_days(2_453_986.285_649),
                TimeScale::Tt,
            ),
            apparent_longitude_deg: 345.100_191_937_146_6,
            apparent_latitude_deg: -0.943_574_954_873_848_7,
            apparent_distance_au: 0.002_388_348_345_388_321_4,
            apparent_right_ascension_deg: 346.648_333_333_333_3,
            apparent_declination_deg: -6.740_444_444_444_445,
            note: "Published 2006-09-07 geocentric Moon coordinate row from EclipseWise; apparent equatorial coordinates are published directly and the ecliptic row is derived from them using the shared mean-obliquity transform",
        },
    ];

    SAMPLES
}

/// Returns the canonical ecliptic apparent-comparison request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests_for_frame(CoordinateFrame::Ecliptic)
}

/// This is a compatibility alias for [`lunar_apparent_comparison_requests`].
#[doc(alias = "lunar_apparent_comparison_requests")]
pub fn lunar_apparent_comparison_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests()
}

/// Returns the canonical ecliptic apparent-comparison batch-parity request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests()
}

/// This is a compatibility alias for [`lunar_apparent_comparison_batch_parity_requests`].
#[doc(alias = "lunar_apparent_comparison_batch_parity_requests")]
pub fn lunar_apparent_comparison_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_batch_parity_requests()
}

/// Returns the canonical equatorial apparent-comparison request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_equatorial_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests_for_frame(CoordinateFrame::Equatorial)
}

/// This is a compatibility alias for [`lunar_apparent_comparison_equatorial_requests`].
#[doc(alias = "lunar_apparent_comparison_equatorial_requests")]
pub fn lunar_apparent_comparison_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_equatorial_requests()
}

/// Returns the canonical equatorial apparent-comparison batch-parity request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_equatorial_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_equatorial_requests()
}

/// This is a compatibility alias for [`lunar_apparent_comparison_equatorial_batch_parity_requests`].
#[doc(alias = "lunar_apparent_comparison_equatorial_batch_parity_requests")]
pub fn lunar_apparent_comparison_equatorial_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_equatorial_batch_parity_requests()
}

/// A compact summary of the reference-only apparent Moon comparison evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarApparentComparisonSummary {
    /// Number of evidence samples in the checked-in comparison slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the comparison slice.
    pub body_count: usize,
    /// Earliest epoch covered by the comparison slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the comparison slice.
    pub latest_epoch: Instant,
    /// Epoch that produces the maximum apparent minus mean ecliptic longitude delta.
    pub max_ecliptic_longitude_epoch: Instant,
    /// Maximum apparent minus mean ecliptic longitude delta in degrees.
    pub max_ecliptic_longitude_delta_deg: f64,
    /// Mean absolute apparent minus mean ecliptic longitude delta in degrees.
    pub mean_ecliptic_longitude_delta_deg: f64,
    /// Median absolute apparent minus mean ecliptic longitude delta in degrees.
    pub median_ecliptic_longitude_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean ecliptic longitude delta in degrees.
    pub percentile_ecliptic_longitude_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean ecliptic latitude delta.
    pub max_ecliptic_latitude_epoch: Instant,
    /// Maximum apparent minus mean ecliptic latitude delta in degrees.
    pub max_ecliptic_latitude_delta_deg: f64,
    /// Mean absolute apparent minus mean ecliptic latitude delta in degrees.
    pub mean_ecliptic_latitude_delta_deg: f64,
    /// Median absolute apparent minus mean ecliptic latitude delta in degrees.
    pub median_ecliptic_latitude_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean ecliptic latitude delta in degrees.
    pub percentile_ecliptic_latitude_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean distance delta.
    pub max_ecliptic_distance_epoch: Instant,
    /// Maximum apparent minus mean distance delta in astronomical units.
    pub max_ecliptic_distance_delta_au: f64,
    /// Mean absolute apparent minus mean distance delta in astronomical units.
    pub mean_ecliptic_distance_delta_au: f64,
    /// Median absolute apparent minus mean distance delta in astronomical units.
    pub median_ecliptic_distance_delta_au: f64,
    /// 95th-percentile absolute apparent minus mean distance delta in astronomical units.
    pub percentile_ecliptic_distance_delta_au: f64,
    /// Epoch that produces the maximum apparent minus mean right ascension delta.
    pub max_right_ascension_epoch: Instant,
    /// Maximum apparent minus mean right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Mean absolute apparent minus mean right ascension delta in degrees.
    pub mean_right_ascension_delta_deg: f64,
    /// Median absolute apparent minus mean right ascension delta in degrees.
    pub median_right_ascension_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean right ascension delta in degrees.
    pub percentile_right_ascension_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean declination delta.
    pub max_declination_epoch: Instant,
    /// Maximum apparent minus mean declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Mean absolute apparent minus mean declination delta in degrees.
    pub mean_declination_delta_deg: f64,
    /// Median absolute apparent minus mean declination delta in degrees.
    pub median_declination_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean declination delta in degrees.
    pub percentile_declination_delta_deg: f64,
}

impl LunarApparentComparisonSummary {
    /// Validates that the summary still matches the checked-in apparent evidence slice.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let Some(expected) = lunar_apparent_comparison_summary() else {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison evidence is unavailable",
            ));
        };

        if self != &expected {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "lunar apparent comparison evidence summary mismatch: expected {}, found {}",
                    expected.summary_line(),
                    self.summary_line()
                ),
            ));
        }

        Ok(())
    }

    /// Returns the release-facing one-line apparent comparison summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar apparent comparison evidence: {} reference-only samples across {} bodies, epoch range {}, mean-only gap against the published apparent Moon examples, including the 1992-04-12, 1968-12-24, 2004-04-01, and 2006-09-07 examples: Δlon={:+.6}° @ {}; |Δlon| mean/median/p95={:.6}/{:.6}/{:.6}°; Δlat={:+.6}° @ {}; |Δlat| mean/median/p95={:.6}/{:.6}/{:.6}°; Δdist={:+.12} AU @ {}; |Δdist| mean/median/p95={:.12}/{:.12}/{:.12} AU; ΔRA={:+.6}° @ {}; |ΔRA| mean/median/p95={:.6}/{:.6}/{:.6}°; ΔDec={:+.6}° @ {}; |ΔDec| mean/median/p95={:.6}/{:.6}/{:.6}°; apparent requests remain unsupported",
            self.sample_count,
            self.body_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_ecliptic_longitude_delta_deg,
            crate::format_instant(self.max_ecliptic_longitude_epoch),
            self.mean_ecliptic_longitude_delta_deg,
            self.median_ecliptic_longitude_delta_deg,
            self.percentile_ecliptic_longitude_delta_deg,
            self.max_ecliptic_latitude_delta_deg,
            crate::format_instant(self.max_ecliptic_latitude_epoch),
            self.mean_ecliptic_latitude_delta_deg,
            self.median_ecliptic_latitude_delta_deg,
            self.percentile_ecliptic_latitude_delta_deg,
            self.max_ecliptic_distance_delta_au,
            crate::format_instant(self.max_ecliptic_distance_epoch),
            self.mean_ecliptic_distance_delta_au,
            self.median_ecliptic_distance_delta_au,
            self.percentile_ecliptic_distance_delta_au,
            self.max_right_ascension_delta_deg,
            crate::format_instant(self.max_right_ascension_epoch),
            self.mean_right_ascension_delta_deg,
            self.median_right_ascension_delta_deg,
            self.percentile_right_ascension_delta_deg,
            self.max_declination_delta_deg,
            crate::format_instant(self.max_declination_epoch),
            self.mean_declination_delta_deg,
            self.median_declination_delta_deg,
            self.percentile_declination_delta_deg,
        )
    }
}

impl fmt::Display for LunarApparentComparisonSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn lunar_apparent_comparison_baseline_sample(
    body: CelestialBody,
    epoch: Instant,
) -> Option<&'static LunarReferenceSample> {
    lunar_reference_evidence()
        .iter()
        .find(|sample| sample.body == body && sample.epoch == epoch)
}

fn lunar_apparent_comparison_equatorial_baseline_sample(
    body: CelestialBody,
    epoch: Instant,
) -> Option<&'static LunarEquatorialReferenceSample> {
    lunar_equatorial_reference_evidence()
        .iter()
        .find(|sample| sample.body == body && sample.epoch == epoch)
}

/// Returns a compact summary of the reference-only apparent Moon comparison evidence.
pub fn lunar_apparent_comparison_summary() -> Option<LunarApparentComparisonSummary> {
    let samples = lunar_apparent_comparison_evidence();
    if samples.is_empty() || samples.iter().any(|sample| sample.validate().is_err()) {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut max_ecliptic_longitude_epoch = samples[0].epoch;
    let mut max_ecliptic_longitude_delta_deg = 0.0;
    let mut max_ecliptic_longitude_delta_abs = 0.0;
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut max_ecliptic_latitude_epoch = samples[0].epoch;
    let mut max_ecliptic_latitude_delta_deg = 0.0;
    let mut max_ecliptic_latitude_delta_abs = 0.0;
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut max_ecliptic_distance_epoch = samples[0].epoch;
    let mut max_ecliptic_distance_delta_au = 0.0;
    let mut max_ecliptic_distance_delta_abs = 0.0;
    let mut distance_deltas = Vec::with_capacity(samples.len());
    let mut max_right_ascension_epoch = samples[0].epoch;
    let mut max_right_ascension_delta_deg = 0.0;
    let mut max_right_ascension_delta_abs = 0.0;
    let mut right_ascension_deltas = Vec::with_capacity(samples.len());
    let mut max_declination_epoch = samples[0].epoch;
    let mut max_declination_delta_deg = 0.0;
    let mut max_declination_delta_abs = 0.0;
    let mut declination_deltas = Vec::with_capacity(samples.len());

    let mut compared_sample_count = 0usize;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }

        let Some(mean_ecliptic) =
            lunar_apparent_comparison_baseline_sample(sample.body.clone(), sample.epoch)
        else {
            continue;
        };
        let Some(mean_equatorial) =
            lunar_apparent_comparison_equatorial_baseline_sample(sample.body.clone(), sample.epoch)
        else {
            continue;
        };

        compared_sample_count += 1;
        let longitude_delta = sample.apparent_longitude_deg - mean_ecliptic.longitude_deg;
        let latitude_delta = sample.apparent_latitude_deg - mean_ecliptic.latitude_deg;
        let distance_delta =
            sample.apparent_distance_au - mean_ecliptic.distance_au.unwrap_or_default();
        let right_ascension_delta = sample.apparent_right_ascension_deg
            - mean_equatorial.equatorial.right_ascension.degrees();
        let declination_delta =
            sample.apparent_declination_deg - mean_equatorial.equatorial.declination.degrees();

        let longitude_delta_abs = longitude_delta.abs();
        longitude_deltas.push(longitude_delta_abs);
        if longitude_delta_abs >= max_ecliptic_longitude_delta_abs {
            max_ecliptic_longitude_delta_abs = longitude_delta_abs;
            max_ecliptic_longitude_delta_deg = longitude_delta;
            max_ecliptic_longitude_epoch = sample.epoch;
        }
        let latitude_delta_abs = latitude_delta.abs();
        latitude_deltas.push(latitude_delta_abs);
        if latitude_delta_abs >= max_ecliptic_latitude_delta_abs {
            max_ecliptic_latitude_delta_abs = latitude_delta_abs;
            max_ecliptic_latitude_delta_deg = latitude_delta;
            max_ecliptic_latitude_epoch = sample.epoch;
        }
        let distance_delta_abs = distance_delta.abs();
        distance_deltas.push(distance_delta_abs);
        if distance_delta_abs >= max_ecliptic_distance_delta_abs {
            max_ecliptic_distance_delta_abs = distance_delta_abs;
            max_ecliptic_distance_delta_au = distance_delta;
            max_ecliptic_distance_epoch = sample.epoch;
        }
        let right_ascension_delta_abs = right_ascension_delta.abs();
        right_ascension_deltas.push(right_ascension_delta_abs);
        if right_ascension_delta_abs >= max_right_ascension_delta_abs {
            max_right_ascension_delta_abs = right_ascension_delta_abs;
            max_right_ascension_delta_deg = right_ascension_delta;
            max_right_ascension_epoch = sample.epoch;
        }
        let declination_delta_abs = declination_delta.abs();
        declination_deltas.push(declination_delta_abs);
        if declination_delta_abs >= max_declination_delta_abs {
            max_declination_delta_abs = declination_delta_abs;
            max_declination_delta_deg = declination_delta;
            max_declination_epoch = sample.epoch;
        }
    }

    if compared_sample_count == 0 {
        return None;
    }

    let mut longitude_deltas_for_median = longitude_deltas.clone();
    let mut longitude_deltas_for_percentile = longitude_deltas.clone();
    let mut latitude_deltas_for_median = latitude_deltas.clone();
    let mut latitude_deltas_for_percentile = latitude_deltas.clone();
    let mut distance_deltas_for_median = distance_deltas.clone();
    let mut distance_deltas_for_percentile = distance_deltas.clone();
    let mut right_ascension_deltas_for_median = right_ascension_deltas.clone();
    let mut right_ascension_deltas_for_percentile = right_ascension_deltas.clone();
    let mut declination_deltas_for_median = declination_deltas.clone();
    let mut declination_deltas_for_percentile = declination_deltas.clone();

    Some(LunarApparentComparisonSummary {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_ecliptic_longitude_epoch,
        max_ecliptic_longitude_delta_deg,
        mean_ecliptic_longitude_delta_deg: crate::mean_value(&longitude_deltas).unwrap_or(0.0),
        median_ecliptic_longitude_delta_deg: crate::median_value(&mut longitude_deltas_for_median)
            .unwrap_or(0.0),
        percentile_ecliptic_longitude_delta_deg: crate::percentile_value(
            &mut longitude_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_ecliptic_latitude_epoch,
        max_ecliptic_latitude_delta_deg,
        mean_ecliptic_latitude_delta_deg: crate::mean_value(&latitude_deltas).unwrap_or(0.0),
        median_ecliptic_latitude_delta_deg: crate::median_value(&mut latitude_deltas_for_median)
            .unwrap_or(0.0),
        percentile_ecliptic_latitude_delta_deg: crate::percentile_value(
            &mut latitude_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_ecliptic_distance_epoch,
        max_ecliptic_distance_delta_au,
        mean_ecliptic_distance_delta_au: crate::mean_value(&distance_deltas).unwrap_or(0.0),
        median_ecliptic_distance_delta_au: crate::median_value(&mut distance_deltas_for_median)
            .unwrap_or(0.0),
        percentile_ecliptic_distance_delta_au: crate::percentile_value(
            &mut distance_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_right_ascension_epoch,
        max_right_ascension_delta_deg,
        mean_right_ascension_delta_deg: crate::mean_value(&right_ascension_deltas).unwrap_or(0.0),
        median_right_ascension_delta_deg: crate::median_value(
            &mut right_ascension_deltas_for_median,
        )
        .unwrap_or(0.0),
        percentile_right_ascension_delta_deg: crate::percentile_value(
            &mut right_ascension_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_declination_epoch,
        max_declination_delta_deg,
        mean_declination_delta_deg: crate::mean_value(&declination_deltas).unwrap_or(0.0),
        median_declination_delta_deg: crate::median_value(&mut declination_deltas_for_median)
            .unwrap_or(0.0),
        percentile_declination_delta_deg: crate::percentile_value(
            &mut declination_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
    })
}

/// Formats the reference-only apparent Moon comparison summary for release-facing reporting.
pub fn format_lunar_apparent_comparison_summary(
    summary: &LunarApparentComparisonSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing one-line apparent comparison summary.
pub fn lunar_apparent_comparison_summary_for_report() -> String {
    match lunar_apparent_comparison_summary() {
        Some(summary) if summary.validate().is_ok() => {
            format_lunar_apparent_comparison_summary(&summary)
        }
        Some(_) | None => "lunar apparent comparison evidence: unavailable".to_string(),
    }
}

/// A compact summary of the lunar equatorial reference error envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarEquatorialReferenceEvidenceEnvelope {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
    /// Body with the maximum absolute right ascension delta.
    pub max_right_ascension_delta_body: CelestialBody,
    /// Epoch for the maximum absolute right ascension delta.
    pub max_right_ascension_delta_epoch: Instant,
    /// Maximum absolute right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Mean absolute right ascension delta in degrees across the evidence slice.
    pub mean_right_ascension_delta_deg: f64,
    /// Median absolute right ascension delta in degrees across the evidence slice.
    pub median_right_ascension_delta_deg: f64,
    /// 95th-percentile absolute right ascension delta in degrees across the evidence slice.
    pub percentile_right_ascension_delta_deg: f64,
    /// Body with the maximum absolute declination delta.
    pub max_declination_delta_body: CelestialBody,
    /// Epoch for the maximum absolute declination delta.
    pub max_declination_delta_epoch: Instant,
    /// Maximum absolute declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Mean absolute declination delta in degrees across the evidence slice.
    pub mean_declination_delta_deg: f64,
    /// Median absolute declination delta in degrees across the evidence slice.
    pub median_declination_delta_deg: f64,
    /// 95th-percentile absolute declination delta in degrees across the evidence slice.
    pub percentile_declination_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Epoch for the maximum absolute distance delta.
    pub max_distance_delta_epoch: Option<Instant>,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta in astronomical units across the samples that include distance.
    pub mean_distance_delta_au: Option<f64>,
    /// Median absolute distance delta in astronomical units across the samples that include distance.
    pub median_distance_delta_au: Option<f64>,
    /// 95th-percentile absolute distance delta in astronomical units across the samples that include distance.
    pub percentile_distance_delta_au: Option<f64>,
    /// Number of samples outside the current regression limits.
    pub outside_current_limits_count: usize,
    /// Bodies associated with samples outside the current regression limits.
    pub outlier_bodies: Vec<CelestialBody>,
    /// Whether every sample stayed within the current regression limits.
    pub within_current_limits: bool,
}

/// Returns the current lunar equatorial reference error envelope measured against the
/// checked-in reference slice.
pub fn lunar_equatorial_reference_evidence_envelope(
) -> Option<LunarEquatorialReferenceEvidenceEnvelope> {
    let samples = lunar_equatorial_reference_evidence();
    if samples.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut max_right_ascension_delta_body = samples[0].body.clone();
    let mut max_right_ascension_delta_epoch = samples[0].epoch;
    let mut max_right_ascension_delta_deg = 0.0;
    let mut total_right_ascension_delta_deg = 0.0;
    let mut right_ascension_deltas = Vec::with_capacity(samples.len());
    let mut outlier_bodies: Vec<CelestialBody> = Vec::new();
    let mut max_declination_delta_body = samples[0].body.clone();
    let mut max_declination_delta_epoch = samples[0].epoch;
    let mut max_declination_delta_deg = 0.0;
    let mut total_declination_delta_deg = 0.0;
    let mut declination_deltas = Vec::with_capacity(samples.len());
    let mut max_distance_delta_body = None;
    let mut max_distance_delta_epoch = None;
    let mut max_distance_delta_au = None;
    let mut total_distance_delta_au = 0.0;
    let mut distance_deltas = Vec::new();
    let mut distance_delta_sample_count = 0usize;
    let mut outside_current_limits_count = 0usize;
    let mut within_current_limits = true;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }

        let result = backend
            .position(&EphemerisRequest::new(sample.body.clone(), sample.epoch))
            .expect("the canonical lunar equatorial evidence samples should remain computable");
        let equatorial = result.equatorial.expect(
            "the canonical lunar equatorial evidence samples should include equatorial coordinates",
        );

        let right_ascension_delta_deg = signed_longitude_delta_degrees(
            sample.equatorial.right_ascension.degrees(),
            equatorial.right_ascension.degrees(),
        )
        .abs();
        let declination_delta_deg =
            (equatorial.declination.degrees() - sample.equatorial.declination.degrees()).abs();
        let distance_delta_au = match (sample.equatorial.distance_au, equatorial.distance_au) {
            (Some(expected), Some(actual)) => Some((actual - expected).abs()),
            _ => None,
        };

        let right_ascension_limit = 1e-2;
        let declination_limit = 1e-2;
        let distance_limit = 1e-8;
        let sample_within_limits = right_ascension_delta_deg <= right_ascension_limit
            && declination_delta_deg <= declination_limit
            && match distance_delta_au {
                Some(delta) => delta <= distance_limit,
                None => true,
            };
        within_current_limits &= sample_within_limits;
        if !sample_within_limits {
            outside_current_limits_count += 1;
            if !outlier_bodies.contains(&sample.body) {
                outlier_bodies.push(sample.body.clone());
            }
        }

        total_right_ascension_delta_deg += right_ascension_delta_deg;
        right_ascension_deltas.push(right_ascension_delta_deg);
        total_declination_delta_deg += declination_delta_deg;
        declination_deltas.push(declination_delta_deg);
        if let Some(delta) = distance_delta_au {
            total_distance_delta_au += delta;
            distance_deltas.push(delta);
            distance_delta_sample_count += 1;
        }

        if right_ascension_delta_deg > max_right_ascension_delta_deg {
            max_right_ascension_delta_deg = right_ascension_delta_deg;
            max_right_ascension_delta_body = sample.body.clone();
            max_right_ascension_delta_epoch = sample.epoch;
        }
        if declination_delta_deg > max_declination_delta_deg {
            max_declination_delta_deg = declination_delta_deg;
            max_declination_delta_body = sample.body.clone();
            max_declination_delta_epoch = sample.epoch;
        }
        if let Some(delta) = distance_delta_au {
            match max_distance_delta_au {
                Some(current_max) if delta <= current_max => {}
                _ => {
                    max_distance_delta_body = Some(sample.body.clone());
                    max_distance_delta_epoch = Some(sample.epoch);
                    max_distance_delta_au = Some(delta);
                }
            }
        }
    }

    let mut right_ascension_deltas_for_median = right_ascension_deltas.clone();
    let mut right_ascension_deltas_for_percentile = right_ascension_deltas;
    let mut declination_deltas_for_median = declination_deltas.clone();
    let mut declination_deltas_for_percentile = declination_deltas;
    let mut distance_deltas_for_median = distance_deltas.clone();
    let mut distance_deltas_for_percentile = distance_deltas;

    Some(LunarEquatorialReferenceEvidenceEnvelope {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_right_ascension_delta_body,
        max_right_ascension_delta_epoch,
        max_right_ascension_delta_deg,
        mean_right_ascension_delta_deg: total_right_ascension_delta_deg / samples.len() as f64,
        median_right_ascension_delta_deg: crate::median_value(
            &mut right_ascension_deltas_for_median,
        )
        .unwrap_or_default(),
        percentile_right_ascension_delta_deg: crate::percentile_value(
            &mut right_ascension_deltas_for_percentile,
            0.95,
        )
        .unwrap_or_default(),
        max_declination_delta_body,
        max_declination_delta_epoch,
        max_declination_delta_deg,
        mean_declination_delta_deg: total_declination_delta_deg / samples.len() as f64,
        median_declination_delta_deg: crate::median_value(&mut declination_deltas_for_median)
            .unwrap_or_default(),
        percentile_declination_delta_deg: crate::percentile_value(
            &mut declination_deltas_for_percentile,
            0.95,
        )
        .unwrap_or_default(),
        max_distance_delta_body,
        max_distance_delta_epoch,
        max_distance_delta_au,
        mean_distance_delta_au: (distance_delta_sample_count > 0)
            .then_some(total_distance_delta_au / distance_delta_sample_count as f64),
        median_distance_delta_au: crate::median_value(&mut distance_deltas_for_median),
        percentile_distance_delta_au: crate::percentile_value(
            &mut distance_deltas_for_percentile,
            0.95,
        ),
        outside_current_limits_count,
        outlier_bodies,
        within_current_limits,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LunarEvidenceEnvelopeValidationError {
    SampleCountTooSmall {
        envelope: &'static str,
        sample_count: usize,
    },
    BodyCountTooSmall {
        envelope: &'static str,
        body_count: usize,
    },
    InvalidEpochRange {
        envelope: &'static str,
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    NonFiniteMeasure {
        envelope: &'static str,
        field: &'static str,
    },
    DuplicateOutlierBody {
        envelope: &'static str,
        body: CelestialBody,
    },
    OutlierCountMismatch {
        envelope: &'static str,
        outside_current_limits_count: usize,
        sample_count: usize,
        outlier_bodies_len: usize,
    },
    WithinCurrentLimitsMismatch {
        envelope: &'static str,
        outside_current_limits_count: usize,
        within_current_limits: bool,
    },
}

impl fmt::Display for LunarEvidenceEnvelopeValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SampleCountTooSmall {
                envelope,
                sample_count,
            } => write!(f, "{envelope} has no samples ({sample_count})"),
            Self::BodyCountTooSmall {
                envelope,
                body_count,
            } => write!(f, "{envelope} has no bodies ({body_count})"),
            Self::InvalidEpochRange {
                envelope,
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "{envelope} has an invalid epoch range {}",
                TimeRange::new(Some(*earliest_epoch), Some(*latest_epoch)).summary_line()
            ),
            Self::NonFiniteMeasure { envelope, field } => {
                write!(f, "{envelope} field `{field}` is not finite")
            }
            Self::DuplicateOutlierBody { envelope, body } => {
                write!(f, "{envelope} has duplicate outlier body {body}")
            }
            Self::OutlierCountMismatch {
                envelope,
                outside_current_limits_count,
                sample_count,
                outlier_bodies_len,
            } => write!(
                f,
                "{envelope} reports {outside_current_limits_count} samples outside the limits across {sample_count} samples with {outlier_bodies_len} unique outlier bodies"
            ),
            Self::WithinCurrentLimitsMismatch {
                envelope,
                outside_current_limits_count,
                within_current_limits,
            } => write!(
                f,
                "{envelope} reports within_current_limits={within_current_limits} but outside_current_limits_count={outside_current_limits_count}"
            ),
        }
    }
}

impl std::error::Error for LunarEvidenceEnvelopeValidationError {}

#[allow(clippy::too_many_arguments)]
fn validate_lunar_evidence_envelope(
    envelope: &'static str,
    sample_count: usize,
    body_count: usize,
    earliest_epoch: Instant,
    latest_epoch: Instant,
    numeric_fields: &[(&'static str, f64)],
    distance_fields: &[(&'static str, Option<f64>)],
    outlier_bodies: &[CelestialBody],
    outside_current_limits_count: usize,
    within_current_limits: bool,
) -> Result<(), LunarEvidenceEnvelopeValidationError> {
    if sample_count == 0 {
        return Err(LunarEvidenceEnvelopeValidationError::SampleCountTooSmall {
            envelope,
            sample_count,
        });
    }
    if body_count == 0 {
        return Err(LunarEvidenceEnvelopeValidationError::BodyCountTooSmall {
            envelope,
            body_count,
        });
    }
    if earliest_epoch.julian_day.days() > latest_epoch.julian_day.days() {
        return Err(LunarEvidenceEnvelopeValidationError::InvalidEpochRange {
            envelope,
            earliest_epoch,
            latest_epoch,
        });
    }

    for (field, value) in numeric_fields {
        if !value.is_finite() {
            return Err(LunarEvidenceEnvelopeValidationError::NonFiniteMeasure { envelope, field });
        }
    }

    for (field, value) in distance_fields {
        if let Some(value) = value {
            if !value.is_finite() {
                return Err(LunarEvidenceEnvelopeValidationError::NonFiniteMeasure {
                    envelope,
                    field,
                });
            }
        }
    }

    let has_outlier_samples = outside_current_limits_count > 0;
    if outside_current_limits_count > sample_count
        || outlier_bodies.is_empty() == has_outlier_samples
    {
        return Err(LunarEvidenceEnvelopeValidationError::OutlierCountMismatch {
            envelope,
            outside_current_limits_count,
            sample_count,
            outlier_bodies_len: outlier_bodies.len(),
        });
    }
    if within_current_limits == has_outlier_samples {
        return Err(
            LunarEvidenceEnvelopeValidationError::WithinCurrentLimitsMismatch {
                envelope,
                outside_current_limits_count,
                within_current_limits,
            },
        );
    }

    let mut seen_outlier_bodies = Vec::with_capacity(outlier_bodies.len());
    for body in outlier_bodies {
        if seen_outlier_bodies.iter().any(|seen| seen == body) {
            return Err(LunarEvidenceEnvelopeValidationError::DuplicateOutlierBody {
                envelope,
                body: body.clone(),
            });
        }
        seen_outlier_bodies.push(body.clone());
    }

    Ok(())
}

impl LunarEquatorialReferenceEvidenceEnvelope {
    /// Returns `Ok(())` when the equatorial reference envelope remains internally consistent.
    pub(crate) fn validate(&self) -> Result<(), LunarEvidenceEnvelopeValidationError> {
        validate_lunar_evidence_envelope(
            "lunar equatorial reference error envelope",
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            &[
                (
                    "max_right_ascension_delta_deg",
                    self.max_right_ascension_delta_deg,
                ),
                (
                    "mean_right_ascension_delta_deg",
                    self.mean_right_ascension_delta_deg,
                ),
                (
                    "median_right_ascension_delta_deg",
                    self.median_right_ascension_delta_deg,
                ),
                (
                    "percentile_right_ascension_delta_deg",
                    self.percentile_right_ascension_delta_deg,
                ),
                ("max_declination_delta_deg", self.max_declination_delta_deg),
                (
                    "mean_declination_delta_deg",
                    self.mean_declination_delta_deg,
                ),
                (
                    "median_declination_delta_deg",
                    self.median_declination_delta_deg,
                ),
                (
                    "percentile_declination_delta_deg",
                    self.percentile_declination_delta_deg,
                ),
            ],
            &[
                ("max_distance_delta_au", self.max_distance_delta_au),
                ("mean_distance_delta_au", self.mean_distance_delta_au),
                ("median_distance_delta_au", self.median_distance_delta_au),
                (
                    "percentile_distance_delta_au",
                    self.percentile_distance_delta_au,
                ),
            ],
            &self.outlier_bodies,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }

    /// Returns the release-facing one-line lunar equatorial reference error envelope.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch(body: &CelestialBody, epoch: Instant) -> String {
            format!("{} @ {}", body, crate::format_instant(epoch))
        }

        let distance = match (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_epoch,
            self.max_distance_delta_au,
        ) {
            (Some(body), Some(epoch), Some(delta)) => {
                format!(
                    "; max Δdist={delta:.12} AU ({})",
                    format_body_epoch(body, epoch)
                )
            }
            _ => String::new(),
        };
        let mean_distance = self
            .mean_distance_delta_au
            .map(|value| format!("; mean Δdist={value:.12} AU"))
            .unwrap_or_default();
        let median_distance = self
            .median_distance_delta_au
            .map(|value| format!("; median Δdist={value:.12} AU"))
            .unwrap_or_default();
        let percentile_distance = self
            .percentile_distance_delta_au
            .map(|value| format!("; p95 Δdist={value:.12} AU"))
            .unwrap_or_default();
        let limit_note = "; limits: ΔRA≤1e-2°, ΔDec≤1e-2°, Δdist≤1e-8 AU";
        let outlier_note = if self.outlier_bodies.is_empty() {
            "; outliers=none".to_string()
        } else {
            format!("; outliers={}", crate::format_bodies(&self.outlier_bodies))
        };

        format!(
            "lunar equatorial reference error envelope: {} samples across {} bodies, epoch range {}, max ΔRA={:.12}° ({}), mean ΔRA={:.12}°, median ΔRA={:.12}°, p95 ΔRA={:.12}°, max ΔDec={:.12}° ({}), mean ΔDec={:.12}°, median ΔDec={:.12}°, p95 ΔDec={:.12}°{}{}{}{}{}{}; outside current limits={}; within current limits={}",
            self.sample_count,
            self.body_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_right_ascension_delta_deg,
            format_body_epoch(&self.max_right_ascension_delta_body, self.max_right_ascension_delta_epoch),
            self.mean_right_ascension_delta_deg,
            self.median_right_ascension_delta_deg,
            self.percentile_right_ascension_delta_deg,
            self.max_declination_delta_deg,
            format_body_epoch(&self.max_declination_delta_body, self.max_declination_delta_epoch),
            self.mean_declination_delta_deg,
            self.median_declination_delta_deg,
            self.percentile_declination_delta_deg,
            distance,
            mean_distance,
            median_distance,
            percentile_distance,
            limit_note,
            outlier_note,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }
}

impl fmt::Display for LunarEquatorialReferenceEvidenceEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar equatorial reference error envelope for release-facing reporting.
pub fn format_lunar_equatorial_reference_evidence_envelope(
    envelope: &LunarEquatorialReferenceEvidenceEnvelope,
) -> String {
    envelope.summary_line()
}

/// Returns the release-facing lunar equatorial reference error envelope string.
pub fn lunar_equatorial_reference_evidence_envelope_for_report() -> String {
    match lunar_equatorial_reference_evidence_envelope() {
        Some(envelope) => match envelope.validate() {
            Ok(()) => format_lunar_equatorial_reference_evidence_envelope(&envelope),
            Err(error) => {
                format!("lunar equatorial reference error envelope: unavailable ({error})")
            }
        },
        None => "lunar equatorial reference error envelope: unavailable".to_string(),
    }
}

/// Validation error for a lunar high-curvature continuity evidence slice.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LunarHighCurvatureEvidenceValidationError {
    /// The regression slice does not include enough samples to justify a continuity envelope.
    SampleCountTooSmall { sample_count: usize },
    /// The regression slice unexpectedly lost all bodies.
    BodyCountTooSmall { body_count: usize },
    /// The stored epoch bounds no longer describe a monotonic range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// A stored step metric is not finite.
    NonFiniteMeasure { field: &'static str },
    /// A stored step window is reversed.
    ReversedStepWindow { field: &'static str },
    /// The stored regression-limit flag drifted away from the derived thresholds.
    RegressionLimitMismatch {
        envelope: &'static str,
        within_regression_limits: bool,
        expected_within_regression_limits: bool,
    },
}

impl fmt::Display for LunarHighCurvatureEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SampleCountTooSmall { sample_count } => write!(
                f,
                "the lunar high-curvature continuity evidence has too few samples ({sample_count})"
            ),
            Self::BodyCountTooSmall { body_count } => write!(
                f,
                "the lunar high-curvature continuity evidence has no bodies ({body_count})"
            ),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "the lunar high-curvature continuity evidence has an invalid epoch range {}",
                TimeRange::new(Some(*earliest_epoch), Some(*latest_epoch)).summary_line()
            ),
            Self::NonFiniteMeasure { field } => write!(
                f,
                "the lunar high-curvature continuity evidence field `{field}` is not finite"
            ),
            Self::ReversedStepWindow { field } => write!(
                f,
                "the lunar high-curvature continuity evidence field `{field}` has a reversed step window"
            ),
            Self::RegressionLimitMismatch {
                envelope,
                within_regression_limits,
                expected_within_regression_limits,
            } => write!(
                f,
                "the {envelope} stored regression-limit flag `{within_regression_limits}` does not match the derived limits (`{expected_within_regression_limits}`)"
            ),
        }
    }
}

impl std::error::Error for LunarHighCurvatureEvidenceValidationError {}

fn validate_high_curvature_continuity_window(
    sample_count: usize,
    body_count: usize,
    earliest_epoch: Instant,
    latest_epoch: Instant,
    step_fields: [(&'static str, Instant, Instant, f64); 3],
) -> Result<(), LunarHighCurvatureEvidenceValidationError> {
    if sample_count < 2 {
        return Err(
            LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count },
        );
    }
    if body_count == 0 {
        return Err(LunarHighCurvatureEvidenceValidationError::BodyCountTooSmall { body_count });
    }
    if earliest_epoch.julian_day.days() > latest_epoch.julian_day.days() {
        return Err(
            LunarHighCurvatureEvidenceValidationError::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            },
        );
    }

    for (field, start_epoch, end_epoch, measure) in step_fields {
        if !measure.is_finite() {
            return Err(LunarHighCurvatureEvidenceValidationError::NonFiniteMeasure { field });
        }
        if start_epoch.julian_day.days() > end_epoch.julian_day.days() {
            return Err(LunarHighCurvatureEvidenceValidationError::ReversedStepWindow { field });
        }
    }

    Ok(())
}

/// A compact summary of the lunar high-curvature continuity evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LunarHighCurvatureContinuityEnvelope {
    /// Number of continuity samples in the regression slice.
    pub(crate) sample_count: usize,
    /// Number of distinct bodies covered by the regression slice.
    pub(crate) body_count: usize,
    /// Earliest epoch covered by the regression slice.
    pub(crate) earliest_epoch: Instant,
    /// Latest epoch covered by the regression slice.
    pub(crate) latest_epoch: Instant,
    /// Epoch at the start of the largest adjacent longitude step.
    pub(crate) max_longitude_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent longitude step.
    pub(crate) max_longitude_step_end_epoch: Instant,
    /// Largest adjacent longitude step in degrees.
    pub(crate) max_longitude_step_deg: f64,
    /// Epoch at the start of the largest adjacent latitude step.
    pub(crate) max_latitude_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent latitude step.
    pub(crate) max_latitude_step_end_epoch: Instant,
    /// Largest adjacent latitude step in degrees.
    pub(crate) max_latitude_step_deg: f64,
    /// Epoch at the start of the largest adjacent distance step.
    pub(crate) max_distance_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent distance step.
    pub(crate) max_distance_step_end_epoch: Instant,
    /// Largest adjacent distance step in astronomical units.
    pub(crate) max_distance_step_au: f64,
    /// Whether every adjacent step stayed within the current regression limits.
    pub(crate) within_regression_limits: bool,
}

impl LunarHighCurvatureContinuityEnvelope {
    /// Returns the compact release-facing summary line for this continuity evidence slice.
    pub(crate) fn summary_line(&self) -> String {
        format!(
            "lunar high-curvature continuity evidence: {} samples across {} bodies, epoch range {}, max adjacent Δlon={:.12}° ({} → {}), max adjacent Δlat={:.12}° ({} → {}), max adjacent Δdist={:.12} AU ({} → {}), regression limits: Δlon≤{:.1}°, Δlat≤{:.1}°, Δdist≤{:.2} AU; within regression limits={}",
            self.sample_count,
            self.body_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_longitude_step_deg,
            crate::format_instant(self.max_longitude_step_start_epoch),
            crate::format_instant(self.max_longitude_step_end_epoch),
            self.max_latitude_step_deg,
            crate::format_instant(self.max_latitude_step_start_epoch),
            crate::format_instant(self.max_latitude_step_end_epoch),
            self.max_distance_step_au,
            crate::format_instant(self.max_distance_step_start_epoch),
            crate::format_instant(self.max_distance_step_end_epoch),
            crate::specification::LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG,
            crate::specification::LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG,
            crate::specification::LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU,
            self.within_regression_limits,
        )
    }

    /// Returns the compact summary line after validating the continuity evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarHighCurvatureEvidenceValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    pub(crate) fn validate(&self) -> Result<(), LunarHighCurvatureEvidenceValidationError> {
        validate_high_curvature_continuity_window(
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            [
                (
                    "max_longitude_step_deg",
                    self.max_longitude_step_start_epoch,
                    self.max_longitude_step_end_epoch,
                    self.max_longitude_step_deg,
                ),
                (
                    "max_latitude_step_deg",
                    self.max_latitude_step_start_epoch,
                    self.max_latitude_step_end_epoch,
                    self.max_latitude_step_deg,
                ),
                (
                    "max_distance_step_au",
                    self.max_distance_step_start_epoch,
                    self.max_distance_step_end_epoch,
                    self.max_distance_step_au,
                ),
            ],
        )?;

        let expected_within_regression_limits = self.max_longitude_step_deg
            <= crate::specification::LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG
            && self.max_latitude_step_deg
                <= crate::specification::LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG
            && self.max_distance_step_au
                <= crate::specification::LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;
        if self.within_regression_limits != expected_within_regression_limits {
            return Err(
                LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                    envelope: "lunar high-curvature continuity evidence",
                    within_regression_limits: self.within_regression_limits,
                    expected_within_regression_limits,
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for LunarHighCurvatureContinuityEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn lunar_high_curvature_requests(frame: CoordinateFrame) -> Vec<EphemerisRequest> {
    crate::specification::LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS
        .into_iter()
        .map(|instant| {
            let mut request = EphemerisRequest::new(CelestialBody::Moon, instant);
            request.frame = frame;
            request
        })
        .collect()
}

/// Returns the lunar high-curvature continuity request corpus used by the nearby-motion regression slice.
pub fn lunar_high_curvature_continuity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_requests(CoordinateFrame::Ecliptic)
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
pub fn lunar_high_curvature_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
#[doc(alias = "lunar_high_curvature_request_corpus")]
pub fn lunar_high_curvature_continuity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
#[doc(alias = "lunar_high_curvature_continuity_request_corpus")]
pub fn lunar_high_curvature_continuity_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
#[doc(alias = "lunar_high_curvature_continuity_request_corpus")]
pub fn lunar_high_curvature_continuity_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// Returns the lunar high-curvature equatorial continuity request corpus used by the nearby-motion regression slice.
pub fn lunar_high_curvature_equatorial_continuity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_requests(CoordinateFrame::Equatorial)
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
pub fn lunar_high_curvature_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
#[doc(alias = "lunar_high_curvature_equatorial_request_corpus")]
pub fn lunar_high_curvature_equatorial_continuity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
#[doc(alias = "lunar_high_curvature_equatorial_continuity_request_corpus")]
pub fn lunar_high_curvature_equatorial_continuity_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
#[doc(alias = "lunar_high_curvature_equatorial_continuity_request_corpus")]
pub fn lunar_high_curvature_equatorial_continuity_batch_parity_request_corpus(
) -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

pub(crate) fn lunar_high_curvature_continuity_envelope(
) -> Option<LunarHighCurvatureContinuityEnvelope> {
    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let requests = lunar_high_curvature_continuity_requests();
    let sample_count = requests.len();
    let mut earliest_epoch = requests[0].instant;
    let mut latest_epoch = requests[0].instant;
    let mut previous_sample: Option<(Instant, f64, f64, f64)> = None;
    let mut max_longitude_step_start_epoch = requests[0].instant;
    let mut max_longitude_step_end_epoch = requests[0].instant;
    let mut max_longitude_step_deg = 0.0;
    let mut max_latitude_step_start_epoch = requests[0].instant;
    let mut max_latitude_step_end_epoch = requests[0].instant;
    let mut max_latitude_step_deg = 0.0;
    let mut max_distance_step_start_epoch = requests[0].instant;
    let mut max_distance_step_end_epoch = requests[0].instant;
    let mut max_distance_step_au = 0.0;
    let mut within_regression_limits = true;

    for request in &requests {
        let instant = request.instant;
        let result = backend
            .position(request)
            .expect("the high-curvature lunar continuity samples should remain computable");
        let ecliptic = result.ecliptic.expect(
            "the high-curvature lunar continuity samples should include ecliptic coordinates",
        );
        let longitude = ecliptic.longitude.degrees();
        let latitude = ecliptic.latitude.degrees();
        let distance = ecliptic
            .distance_au
            .expect("the high-curvature lunar continuity samples should include distance");

        bodies.insert(result.body.to_string());
        if instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = instant;
        }
        if instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = instant;
        }

        if let Some((previous_epoch, previous_longitude, previous_latitude, previous_distance)) =
            previous_sample
        {
            let longitude_step =
                signed_longitude_delta_degrees(previous_longitude, longitude).abs();
            let latitude_step = (latitude - previous_latitude).abs();
            let distance_step = (distance - previous_distance).abs();

            within_regression_limits &= longitude_step
                <= crate::specification::LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG
                && latitude_step <= crate::specification::LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG
                && distance_step <= crate::specification::LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;

            if longitude_step > max_longitude_step_deg {
                max_longitude_step_deg = longitude_step;
                max_longitude_step_start_epoch = previous_epoch;
                max_longitude_step_end_epoch = instant;
            }
            if latitude_step > max_latitude_step_deg {
                max_latitude_step_deg = latitude_step;
                max_latitude_step_start_epoch = previous_epoch;
                max_latitude_step_end_epoch = instant;
            }
            if distance_step > max_distance_step_au {
                max_distance_step_au = distance_step;
                max_distance_step_start_epoch = previous_epoch;
                max_distance_step_end_epoch = instant;
            }
        }

        previous_sample = Some((instant, longitude, latitude, distance));
    }

    Some(LunarHighCurvatureContinuityEnvelope {
        sample_count,
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_longitude_step_start_epoch,
        max_longitude_step_end_epoch,
        max_longitude_step_deg,
        max_latitude_step_start_epoch,
        max_latitude_step_end_epoch,
        max_latitude_step_deg,
        max_distance_step_start_epoch,
        max_distance_step_end_epoch,
        max_distance_step_au,
        within_regression_limits,
    })
}

fn format_lunar_high_curvature_continuity_evidence_envelope(
    envelope: &LunarHighCurvatureContinuityEnvelope,
) -> String {
    match envelope.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(_) => "lunar high-curvature continuity evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar high-curvature continuity evidence string.
pub fn lunar_high_curvature_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_continuity_envelope() {
        Some(envelope) => format_lunar_high_curvature_continuity_evidence_envelope(&envelope),
        _ => "lunar high-curvature continuity evidence: unavailable".to_string(),
    }
}

const LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG: f64 = 20.0;
const LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG: f64 = 10.0;

/// A compact summary of the lunar high-curvature equatorial continuity evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LunarHighCurvatureEquatorialContinuityEnvelope {
    /// Number of continuity samples in the regression slice.
    pub(crate) sample_count: usize,
    /// Number of distinct bodies covered by the regression slice.
    pub(crate) body_count: usize,
    /// Earliest epoch covered by the regression slice.
    pub(crate) earliest_epoch: Instant,
    /// Latest epoch covered by the regression slice.
    pub(crate) latest_epoch: Instant,
    /// Epoch at the start of the largest adjacent right ascension step.
    pub(crate) max_right_ascension_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent right ascension step.
    pub(crate) max_right_ascension_step_end_epoch: Instant,
    /// Largest adjacent right ascension step in degrees.
    pub(crate) max_right_ascension_step_deg: f64,
    /// Epoch at the start of the largest adjacent declination step.
    pub(crate) max_declination_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent declination step.
    pub(crate) max_declination_step_end_epoch: Instant,
    /// Largest adjacent declination step in degrees.
    pub(crate) max_declination_step_deg: f64,
    /// Epoch at the start of the largest adjacent distance step.
    pub(crate) max_distance_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent distance step.
    pub(crate) max_distance_step_end_epoch: Instant,
    /// Largest adjacent distance step in astronomical units.
    pub(crate) max_distance_step_au: f64,
    /// Whether every adjacent step stayed within the current regression limits.
    pub(crate) within_regression_limits: bool,
}

impl LunarHighCurvatureEquatorialContinuityEnvelope {
    /// Returns the compact release-facing summary line for this equatorial evidence slice.
    pub(crate) fn summary_line(&self) -> String {
        format!(
            "lunar high-curvature equatorial continuity evidence: {} samples across {} bodies, epoch range {}, max adjacent ΔRA={:.12}° ({} → {}), max adjacent ΔDec={:.12}° ({} → {}), max adjacent Δdist={:.12} AU ({} → {}), regression limits: ΔRA≤{:.1}°, ΔDec≤{:.1}°, Δdist≤{:.2} AU; within regression limits={}",
            self.sample_count,
            self.body_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_right_ascension_step_deg,
            crate::format_instant(self.max_right_ascension_step_start_epoch),
            crate::format_instant(self.max_right_ascension_step_end_epoch),
            self.max_declination_step_deg,
            crate::format_instant(self.max_declination_step_start_epoch),
            crate::format_instant(self.max_declination_step_end_epoch),
            self.max_distance_step_au,
            crate::format_instant(self.max_distance_step_start_epoch),
            crate::format_instant(self.max_distance_step_end_epoch),
            LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG,
            LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG,
            crate::specification::LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU,
            self.within_regression_limits,
        )
    }

    /// Returns the compact summary line after validating the equatorial continuity evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarHighCurvatureEvidenceValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    pub(crate) fn validate(&self) -> Result<(), LunarHighCurvatureEvidenceValidationError> {
        validate_high_curvature_continuity_window(
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            [
                (
                    "max_right_ascension_step_deg",
                    self.max_right_ascension_step_start_epoch,
                    self.max_right_ascension_step_end_epoch,
                    self.max_right_ascension_step_deg,
                ),
                (
                    "max_declination_step_deg",
                    self.max_declination_step_start_epoch,
                    self.max_declination_step_end_epoch,
                    self.max_declination_step_deg,
                ),
                (
                    "max_distance_step_au",
                    self.max_distance_step_start_epoch,
                    self.max_distance_step_end_epoch,
                    self.max_distance_step_au,
                ),
            ],
        )?;

        let expected_within_regression_limits = self.max_right_ascension_step_deg
            <= LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG
            && self.max_declination_step_deg <= LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG
            && self.max_distance_step_au
                <= crate::specification::LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;
        if self.within_regression_limits != expected_within_regression_limits {
            return Err(
                LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                    envelope: "lunar high-curvature equatorial continuity evidence",
                    within_regression_limits: self.within_regression_limits,
                    expected_within_regression_limits,
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for LunarHighCurvatureEquatorialContinuityEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn lunar_high_curvature_equatorial_continuity_envelope(
) -> Option<LunarHighCurvatureEquatorialContinuityEnvelope> {
    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let requests = lunar_high_curvature_equatorial_continuity_requests();
    let sample_count = requests.len();
    let mut earliest_epoch = requests[0].instant;
    let mut latest_epoch = requests[0].instant;
    let mut previous_sample: Option<(Instant, f64, f64, f64)> = None;
    let mut max_right_ascension_step_start_epoch = requests[0].instant;
    let mut max_right_ascension_step_end_epoch = requests[0].instant;
    let mut max_right_ascension_step_deg = 0.0;
    let mut max_declination_step_start_epoch = requests[0].instant;
    let mut max_declination_step_end_epoch = requests[0].instant;
    let mut max_declination_step_deg = 0.0;
    let mut max_distance_step_start_epoch = requests[0].instant;
    let mut max_distance_step_end_epoch = requests[0].instant;
    let mut max_distance_step_au = 0.0;
    let mut within_regression_limits = true;

    for request in &requests {
        let instant = request.instant;
        let result = backend
            .position(request)
            .expect("the high-curvature lunar continuity samples should remain computable");
        let equatorial = result.equatorial.expect(
            "the high-curvature lunar continuity samples should include equatorial coordinates",
        );
        let right_ascension = equatorial.right_ascension.degrees();
        let declination = equatorial.declination.degrees();
        let distance = equatorial
            .distance_au
            .expect("the high-curvature lunar continuity samples should include distance");

        bodies.insert(result.body.to_string());
        if instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = instant;
        }
        if instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = instant;
        }

        if let Some((
            previous_epoch,
            previous_right_ascension,
            previous_declination,
            previous_distance,
        )) = previous_sample
        {
            let right_ascension_step =
                signed_longitude_delta_degrees(previous_right_ascension, right_ascension).abs();
            let declination_step = (declination - previous_declination).abs();
            let distance_step = (distance - previous_distance).abs();

            within_regression_limits &= right_ascension_step
                <= LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG
                && declination_step <= LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG
                && distance_step <= crate::specification::LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;

            if right_ascension_step > max_right_ascension_step_deg {
                max_right_ascension_step_deg = right_ascension_step;
                max_right_ascension_step_start_epoch = previous_epoch;
                max_right_ascension_step_end_epoch = instant;
            }
            if declination_step > max_declination_step_deg {
                max_declination_step_deg = declination_step;
                max_declination_step_start_epoch = previous_epoch;
                max_declination_step_end_epoch = instant;
            }
            if distance_step > max_distance_step_au {
                max_distance_step_au = distance_step;
                max_distance_step_start_epoch = previous_epoch;
                max_distance_step_end_epoch = instant;
            }
        }

        previous_sample = Some((instant, right_ascension, declination, distance));
    }

    Some(LunarHighCurvatureEquatorialContinuityEnvelope {
        sample_count,
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_right_ascension_step_start_epoch,
        max_right_ascension_step_end_epoch,
        max_right_ascension_step_deg,
        max_declination_step_start_epoch,
        max_declination_step_end_epoch,
        max_declination_step_deg,
        max_distance_step_start_epoch,
        max_distance_step_end_epoch,
        max_distance_step_au,
        within_regression_limits,
    })
}

fn format_lunar_high_curvature_equatorial_continuity_evidence_envelope(
    envelope: &LunarHighCurvatureEquatorialContinuityEnvelope,
) -> String {
    match envelope.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(_) => "lunar high-curvature equatorial continuity evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar high-curvature equatorial continuity evidence string.
pub fn lunar_high_curvature_equatorial_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_equatorial_continuity_envelope() {
        Some(envelope) => {
            format_lunar_high_curvature_equatorial_continuity_evidence_envelope(&envelope)
        }
        _ => "lunar high-curvature equatorial continuity evidence: unavailable".to_string(),
    }
}

/// A compact summary of the lunar reference error envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarReferenceEvidenceEnvelope {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: CelestialBody,
    /// Epoch for the maximum absolute longitude delta.
    pub max_longitude_delta_epoch: Instant,
    /// Maximum absolute longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Mean absolute longitude delta in degrees across the evidence slice.
    pub mean_longitude_delta_deg: f64,
    /// Median absolute longitude delta in degrees across the evidence slice.
    pub median_longitude_delta_deg: f64,
    /// 95th-percentile absolute longitude delta in degrees across the evidence slice.
    pub percentile_longitude_delta_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Epoch for the maximum absolute latitude delta.
    pub max_latitude_delta_epoch: Instant,
    /// Maximum absolute latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta in degrees across the evidence slice.
    pub mean_latitude_delta_deg: f64,
    /// Median absolute latitude delta in degrees across the evidence slice.
    pub median_latitude_delta_deg: f64,
    /// 95th-percentile absolute latitude delta in degrees across the evidence slice.
    pub percentile_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Epoch for the maximum absolute distance delta.
    pub max_distance_delta_epoch: Option<Instant>,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta in astronomical units across the samples that include distance.
    pub mean_distance_delta_au: Option<f64>,
    /// Median absolute distance delta in astronomical units across the samples that include distance.
    pub median_distance_delta_au: Option<f64>,
    /// 95th-percentile absolute distance delta in astronomical units across the samples that include distance.
    pub percentile_distance_delta_au: Option<f64>,
    /// Number of samples outside the current regression limits.
    pub outside_current_limits_count: usize,
    /// Bodies associated with samples outside the current regression limits.
    pub outlier_bodies: Vec<CelestialBody>,
    /// Whether every sample stayed within the current regression limits.
    pub within_current_limits: bool,
}

/// Returns the current lunar reference error envelope measured against the
/// checked-in reference slice.
pub fn lunar_reference_evidence_envelope() -> Option<LunarReferenceEvidenceEnvelope> {
    let samples = lunar_reference_evidence();
    if samples.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut max_longitude_delta_body = samples[0].body.clone();
    let mut max_longitude_delta_epoch = samples[0].epoch;
    let mut max_longitude_delta_deg = 0.0;
    let mut total_longitude_delta_deg = 0.0;
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut max_latitude_delta_body = samples[0].body.clone();
    let mut max_latitude_delta_epoch = samples[0].epoch;
    let mut max_latitude_delta_deg = 0.0;
    let mut total_latitude_delta_deg = 0.0;
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut outlier_bodies: Vec<CelestialBody> = Vec::new();
    let mut max_distance_delta_body = None;
    let mut max_distance_delta_epoch = None;
    let mut max_distance_delta_au = None;
    let mut total_distance_delta_au = 0.0;
    let mut distance_deltas = Vec::new();
    let mut distance_delta_sample_count = 0usize;
    let mut outside_current_limits_count = 0usize;
    let mut within_current_limits = true;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }

        let result = backend
            .position(&EphemerisRequest::new(sample.body.clone(), sample.epoch))
            .expect("the canonical lunar evidence samples should remain computable");
        let ecliptic = result
            .ecliptic
            .expect("the canonical lunar evidence samples should include ecliptic coordinates");

        let longitude_delta_deg =
            signed_longitude_delta_degrees(sample.longitude_deg, ecliptic.longitude.degrees())
                .abs();
        let latitude_delta_deg = (ecliptic.latitude.degrees() - sample.latitude_deg).abs();
        let distance_delta_au = match (sample.distance_au, ecliptic.distance_au) {
            (Some(expected), Some(actual)) => Some((actual - expected).abs()),
            _ => None,
        };

        let longitude_limit = if sample.body == CelestialBody::MeanNode {
            1e-1
        } else {
            1e-4
        };
        let latitude_limit = 1e-4;
        let distance_limit = 1e-8;
        let sample_within_limits = longitude_delta_deg <= longitude_limit
            && latitude_delta_deg <= latitude_limit
            && match distance_delta_au {
                Some(delta) => delta <= distance_limit,
                None => true,
            };
        within_current_limits &= sample_within_limits;
        if !sample_within_limits {
            outside_current_limits_count += 1;
            if !outlier_bodies.contains(&sample.body) {
                outlier_bodies.push(sample.body.clone());
            }
        }

        total_longitude_delta_deg += longitude_delta_deg;
        longitude_deltas.push(longitude_delta_deg);
        total_latitude_delta_deg += latitude_delta_deg;
        latitude_deltas.push(latitude_delta_deg);
        if let Some(delta) = distance_delta_au {
            total_distance_delta_au += delta;
            distance_deltas.push(delta);
            distance_delta_sample_count += 1;
        }

        if longitude_delta_deg > max_longitude_delta_deg {
            max_longitude_delta_deg = longitude_delta_deg;
            max_longitude_delta_body = sample.body.clone();
            max_longitude_delta_epoch = sample.epoch;
        }
        if latitude_delta_deg > max_latitude_delta_deg {
            max_latitude_delta_deg = latitude_delta_deg;
            max_latitude_delta_body = sample.body.clone();
            max_latitude_delta_epoch = sample.epoch;
        }
        if let Some(delta) = distance_delta_au {
            match max_distance_delta_au {
                Some(current_max) if delta <= current_max => {}
                _ => {
                    max_distance_delta_body = Some(sample.body.clone());
                    max_distance_delta_epoch = Some(sample.epoch);
                    max_distance_delta_au = Some(delta);
                }
            }
        }
    }

    let mut longitude_deltas_for_median = longitude_deltas.clone();
    let mut longitude_deltas_for_percentile = longitude_deltas;
    let mut latitude_deltas_for_median = latitude_deltas.clone();
    let mut latitude_deltas_for_percentile = latitude_deltas;
    let mut distance_deltas_for_median = distance_deltas.clone();
    let mut distance_deltas_for_percentile = distance_deltas;

    Some(LunarReferenceEvidenceEnvelope {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_longitude_delta_body,
        max_longitude_delta_epoch,
        max_longitude_delta_deg,
        mean_longitude_delta_deg: total_longitude_delta_deg / samples.len() as f64,
        median_longitude_delta_deg: crate::median_value(&mut longitude_deltas_for_median)
            .unwrap_or_default(),
        percentile_longitude_delta_deg: crate::percentile_value(
            &mut longitude_deltas_for_percentile,
            0.95,
        )
        .unwrap_or_default(),
        max_latitude_delta_body,
        max_latitude_delta_epoch,
        max_latitude_delta_deg,
        mean_latitude_delta_deg: total_latitude_delta_deg / samples.len() as f64,
        median_latitude_delta_deg: crate::median_value(&mut latitude_deltas_for_median)
            .unwrap_or_default(),
        percentile_latitude_delta_deg: crate::percentile_value(
            &mut latitude_deltas_for_percentile,
            0.95,
        )
        .unwrap_or_default(),
        max_distance_delta_body,
        max_distance_delta_epoch,
        max_distance_delta_au,
        mean_distance_delta_au: (distance_delta_sample_count > 0)
            .then_some(total_distance_delta_au / distance_delta_sample_count as f64),
        median_distance_delta_au: crate::median_value(&mut distance_deltas_for_median),
        percentile_distance_delta_au: crate::percentile_value(
            &mut distance_deltas_for_percentile,
            0.95,
        ),
        outside_current_limits_count,
        outlier_bodies,
        within_current_limits,
    })
}

impl LunarReferenceEvidenceEnvelope {
    /// Returns `Ok(())` when the reference envelope remains internally consistent.
    pub(crate) fn validate(&self) -> Result<(), LunarEvidenceEnvelopeValidationError> {
        validate_lunar_evidence_envelope(
            "lunar reference error envelope",
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            &[
                ("max_longitude_delta_deg", self.max_longitude_delta_deg),
                ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
                (
                    "median_longitude_delta_deg",
                    self.median_longitude_delta_deg,
                ),
                (
                    "percentile_longitude_delta_deg",
                    self.percentile_longitude_delta_deg,
                ),
                ("max_latitude_delta_deg", self.max_latitude_delta_deg),
                ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
                ("median_latitude_delta_deg", self.median_latitude_delta_deg),
                (
                    "percentile_latitude_delta_deg",
                    self.percentile_latitude_delta_deg,
                ),
            ],
            &[
                ("max_distance_delta_au", self.max_distance_delta_au),
                ("mean_distance_delta_au", self.mean_distance_delta_au),
                ("median_distance_delta_au", self.median_distance_delta_au),
                (
                    "percentile_distance_delta_au",
                    self.percentile_distance_delta_au,
                ),
            ],
            &self.outlier_bodies,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }

    /// Returns the release-facing one-line lunar reference error envelope.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch(body: &CelestialBody, epoch: Instant) -> String {
            format!("{} @ {}", body, crate::format_instant(epoch))
        }

        let distance = match (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_epoch,
            self.max_distance_delta_au,
        ) {
            (Some(body), Some(epoch), Some(delta)) => {
                format!(
                    "; max Δdist={delta:.12} AU ({})",
                    format_body_epoch(body, epoch)
                )
            }
            _ => String::new(),
        };
        let mean_distance = self
            .mean_distance_delta_au
            .map(|value| format!("; mean Δdist={value:.12} AU"))
            .unwrap_or_default();
        let median_distance = self
            .median_distance_delta_au
            .map(|value| format!("; median Δdist={value:.12} AU"))
            .unwrap_or_default();
        let percentile_distance = self
            .percentile_distance_delta_au
            .map(|value| format!("; p95 Δdist={value:.12} AU"))
            .unwrap_or_default();
        let limit_note = "; limits: Δlon≤1e-4° (1e-1° for mean node), Δlat≤1e-4°, Δdist≤1e-8 AU";
        let outlier_note = if self.outlier_bodies.is_empty() {
            "; outliers=none".to_string()
        } else {
            format!("; outliers={}", crate::format_bodies(&self.outlier_bodies))
        };

        format!(
            "lunar reference error envelope: {} samples across {} bodies, epoch range {}, max Δlon={:.12}° ({}), mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, max Δlat={:.12}° ({}), mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°{}{}{}{}{}{}; outside current limits={}; within current limits={}",
            self.sample_count,
            self.body_count,
            crate::format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_longitude_delta_deg,
            format_body_epoch(&self.max_longitude_delta_body, self.max_longitude_delta_epoch),
            self.mean_longitude_delta_deg,
            self.median_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.max_latitude_delta_deg,
            format_body_epoch(&self.max_latitude_delta_body, self.max_latitude_delta_epoch),
            self.mean_latitude_delta_deg,
            self.median_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            distance,
            mean_distance,
            median_distance,
            percentile_distance,
            limit_note,
            outlier_note,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }
}

impl fmt::Display for LunarReferenceEvidenceEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar reference error envelope for release-facing reporting.
pub fn format_lunar_reference_evidence_envelope(
    envelope: &LunarReferenceEvidenceEnvelope,
) -> String {
    envelope.summary_line()
}

/// Returns the release-facing lunar reference error envelope string.
pub fn lunar_reference_evidence_envelope_for_report() -> String {
    match lunar_reference_evidence_envelope() {
        Some(envelope) => match envelope.validate() {
            Ok(()) => format_lunar_reference_evidence_envelope(&envelope),
            Err(error) => {
                format!("lunar reference error envelope: unavailable ({error})")
            }
        },
        None => "lunar reference error envelope: unavailable".to_string(),
    }
}
