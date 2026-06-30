use super::*;

/// Structured fit envelope for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitEnvelopeSummary {
    /// Number of successfully measured segment samples.
    pub sample_count: usize,
    /// Number of planned segment samples for the current artifact layout.
    pub expected_sample_count: usize,
    /// Number of bundled bodies covered by the measured sample set.
    pub body_count: usize,
    /// Mean absolute longitude delta in degrees.
    pub mean_longitude_delta_degrees: f64,
    /// Mean absolute latitude delta in degrees.
    pub mean_latitude_delta_degrees: f64,
    /// Mean absolute distance delta in AU.
    pub mean_distance_delta_au: f64,
    /// Maximum absolute longitude delta in degrees.
    pub max_longitude_delta_degrees: f64,
    /// Maximum absolute latitude delta in degrees.
    pub max_latitude_delta_degrees: f64,
    /// Maximum absolute distance delta in AU.
    pub max_distance_delta_au: f64,
}

/// A packaged-artifact fit threshold violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedArtifactFitThresholdViolation {
    /// The field that exceeds the calibrated threshold.
    pub field: &'static str,
    /// The measured value, encoded as raw bits so the summary stays lossless.
    pub measured_bits: u64,
    /// The calibrated threshold, encoded as raw bits so the summary stays lossless.
    pub threshold_bits: u64,
    /// The amount by which the measured value exceeds the calibrated threshold.
    pub overage_bits: u64,
}

impl PackagedArtifactFitThresholdViolation {
    fn summary_line(&self) -> String {
        format!(
            "`{}` measured={:.12}, threshold={:.12}, overage={:+.12}",
            self.field,
            f64::from_bits(self.measured_bits),
            f64::from_bits(self.threshold_bits),
            f64::from_bits(self.overage_bits),
        )
    }
}

pub(crate) fn packaged_artifact_fit_threshold_violations_from_envelope_and_thresholds(
    envelope: &PackagedArtifactFitEnvelopeSummary,
    thresholds: &PackagedArtifactFitThresholdSummary,
) -> Vec<PackagedArtifactFitThresholdViolation> {
    let mut violations = Vec::new();

    macro_rules! check_threshold {
        ($field:literal, $measured:expr, $threshold:expr) => {
            if $measured > $threshold {
                violations.push(PackagedArtifactFitThresholdViolation {
                    field: $field,
                    measured_bits: $measured.to_bits(),
                    threshold_bits: $threshold.to_bits(),
                    overage_bits: ($measured - $threshold).to_bits(),
                });
            }
        };
    }

    check_threshold!(
        "mean_longitude_delta_degrees",
        envelope.mean_longitude_delta_degrees,
        thresholds.max_mean_longitude_delta_degrees
    );
    check_threshold!(
        "mean_latitude_delta_degrees",
        envelope.mean_latitude_delta_degrees,
        thresholds.max_mean_latitude_delta_degrees
    );
    check_threshold!(
        "mean_distance_delta_au",
        envelope.mean_distance_delta_au,
        thresholds.max_mean_distance_delta_au
    );
    check_threshold!(
        "max_longitude_delta_degrees",
        envelope.max_longitude_delta_degrees,
        thresholds.max_longitude_delta_degrees
    );
    check_threshold!(
        "max_latitude_delta_degrees",
        envelope.max_latitude_delta_degrees,
        thresholds.max_latitude_delta_degrees
    );
    check_threshold!(
        "max_distance_delta_au",
        envelope.max_distance_delta_au,
        thresholds.max_distance_delta_au
    );

    violations
}

/// Validation error for a packaged-artifact fit envelope that drifted from the current posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedArtifactFitEnvelopeSummaryValidationError {
    /// A rendered summary field no longer matches the current packaged-artifact fit envelope.
    FieldOutOfSync { field: &'static str },
    /// One or more measured fit fields exceed the calibrated packaged-artifact fit thresholds.
    ThresholdExceeded {
        violations: Vec<PackagedArtifactFitThresholdViolation>,
    },
}

impl PackagedArtifactFitEnvelopeSummaryValidationError {
    /// Returns the number of threshold violations captured by the validation error.
    pub fn violation_count(&self) -> usize {
        match self {
            Self::FieldOutOfSync { .. } => 0,
            Self::ThresholdExceeded { violations } => violations.len(),
        }
    }

    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit envelope summary field `{field}` is out of sync with the current posture"
            ),
            Self::ThresholdExceeded { violations } => {
                let rendered = violations
                    .iter()
                    .map(PackagedArtifactFitThresholdViolation::summary_line)
                    .collect::<Vec<_>>()
                    .join("; ");
                let violation_count = violations.len();
                let violation_label = if violation_count == 1 {
                    "violation"
                } else {
                    "violations"
                };
                format!(
                    "the packaged artifact fit envelope summary exceeds the calibrated fit thresholds ({violation_count} {violation_label}): {rendered}"
                )
            }
        }
    }
}

impl fmt::Display for PackagedArtifactFitEnvelopeSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitEnvelopeSummaryValidationError {}

/// Calibrated fit thresholds for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitThresholdSummary {
    /// Maximum allowed mean absolute longitude delta in degrees.
    pub max_mean_longitude_delta_degrees: f64,
    /// Maximum allowed mean absolute latitude delta in degrees.
    pub max_mean_latitude_delta_degrees: f64,
    /// Maximum allowed mean absolute distance delta in AU.
    pub max_mean_distance_delta_au: f64,
    /// Maximum allowed absolute longitude delta in degrees.
    pub max_longitude_delta_degrees: f64,
    /// Maximum allowed absolute latitude delta in degrees.
    pub max_latitude_delta_degrees: f64,
    /// Maximum allowed absolute distance delta in AU.
    pub max_distance_delta_au: f64,
}

/// Validation error for a packaged-artifact fit threshold summary that drifted from the current posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedArtifactFitThresholdSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact fit threshold posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactFitThresholdSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit threshold summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactFitThresholdSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitThresholdSummaryValidationError {}

impl PackagedArtifactFitThresholdSummary {
    /// Returns the calibrated fit thresholds as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "fit thresholds: mean Δlon≤{:.12}°, mean Δlat≤{:.12}°, mean Δdist≤{:.12} AU; max Δlon≤{:.12}°, max Δlat≤{:.12}°, max Δdist≤{:.12} AU",
            self.max_mean_longitude_delta_degrees,
            self.max_mean_latitude_delta_degrees,
            self.max_mean_distance_delta_au,
            self.max_longitude_delta_degrees,
            self.max_latitude_delta_degrees,
            self.max_distance_delta_au,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact fit thresholds.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitThresholdSummaryValidationError> {
        if self != &PACKAGED_ARTIFACT_FIT_THRESHOLD_SUMMARY {
            return Err(
                PackagedArtifactFitThresholdSummaryValidationError::FieldOutOfSync {
                    field: "thresholds",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated calibrated fit thresholds as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitThresholdSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitThresholdSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured fit margins for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitMarginSummary {
    /// Measured fit envelope for the current packaged artifact.
    pub envelope: PackagedArtifactFitEnvelopeSummary,
    /// Calibrated thresholds used to compute the current margins.
    pub thresholds: PackagedArtifactFitThresholdSummary,
}

impl PackagedArtifactFitMarginSummary {
    /// Returns the fit margins relative to the calibrated thresholds as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "fit margins: mean Δlon={:+.12}°, mean Δlat={:+.12}°, mean Δdist={:+.12} AU; max Δlon={:+.12}°, max Δlat={:+.12}°, max Δdist={:+.12} AU",
            self.thresholds.max_mean_longitude_delta_degrees - self.envelope.mean_longitude_delta_degrees,
            self.thresholds.max_mean_latitude_delta_degrees - self.envelope.mean_latitude_delta_degrees,
            self.thresholds.max_mean_distance_delta_au - self.envelope.mean_distance_delta_au,
            self.thresholds.max_longitude_delta_degrees - self.envelope.max_longitude_delta_degrees,
            self.thresholds.max_latitude_delta_degrees - self.envelope.max_latitude_delta_degrees,
            self.thresholds.max_distance_delta_au - self.envelope.max_distance_delta_au,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact fit posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let current_envelope = packaged_artifact_fit_envelope_summary_details();
        if self.envelope != current_envelope {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "envelope",
                },
            );
        }

        let current_thresholds = packaged_artifact_fit_threshold_summary_details();
        if self.thresholds != current_thresholds {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "thresholds",
                },
            );
        }

        self.envelope.validate_against_thresholds(&self.thresholds)
    }

    /// Returns the validated fit margins relative to the calibrated thresholds as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitEnvelopeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitMarginSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Packaged-artifact fit threshold violations captured for the current posture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactFitThresholdViolationsSummary {
    /// Threshold violations ordered by field as they appear in the envelope.
    pub violations: Vec<PackagedArtifactFitThresholdViolation>,
}

/// Validation error for a packaged-artifact fit threshold violation summary that drifted from the current posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedArtifactFitThresholdViolationsSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact fit threshold violation posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactFitThresholdViolationsSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit threshold violation summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactFitThresholdViolationsSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitThresholdViolationsSummaryValidationError {}

impl PackagedArtifactFitThresholdViolationsSummary {
    /// Returns the packaged-artifact fit threshold violations as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        let violation_count = self.violations.len();
        if violation_count == 0 {
            return "fit threshold violations: 0; details: none".to_string();
        }

        let rendered = self
            .violations
            .iter()
            .map(PackagedArtifactFitThresholdViolation::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        let violation_label = if violation_count == 1 {
            "violation"
        } else {
            "violations"
        };
        format!(
            "fit threshold violations: {violation_count} {violation_label}; details: {rendered}"
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact fit threshold violation posture.
    pub fn validate(
        &self,
    ) -> Result<(), PackagedArtifactFitThresholdViolationsSummaryValidationError> {
        let current = packaged_artifact_fit_threshold_violation_summary_details();
        if self == &current {
            Ok(())
        } else {
            Err(
                PackagedArtifactFitThresholdViolationsSummaryValidationError::FieldOutOfSync {
                    field: "violations",
                },
            )
        }
    }

    /// Returns the validated packaged-artifact fit threshold violations as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitThresholdViolationsSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitThresholdViolationsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl PackagedArtifactFitEnvelopeSummary {
    /// Returns `Ok(())` when the measured fit envelope stays within the calibrated thresholds.
    pub fn validate_against_thresholds(
        &self,
        thresholds: &PackagedArtifactFitThresholdSummary,
    ) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let violations = packaged_artifact_fit_threshold_violations_from_envelope_and_thresholds(
            self, thresholds,
        );

        if violations.is_empty() {
            Ok(())
        } else {
            Err(PackagedArtifactFitEnvelopeSummaryValidationError::ThresholdExceeded { violations })
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PackagedArtifactFitSample {
    pub(crate) body: CelestialBody,
    pub(crate) segment_start: Instant,
    pub(crate) segment_end: Instant,
    pub(crate) sample_instant: Instant,
    pub(crate) sample_fraction: f64,
    pub(crate) longitude_delta_degrees: f64,
    pub(crate) latitude_delta_degrees: f64,
    pub(crate) distance_delta_au: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct PackagedArtifactFitSegmentFamilyKey {
    segment_start_days_bits: u64,
    segment_start_scale: TimeScale,
    segment_end_days_bits: u64,
    segment_end_scale: TimeScale,
}

impl PackagedArtifactFitSegmentFamilyKey {
    pub(crate) fn from_sample(sample: &PackagedArtifactFitSample) -> Self {
        Self {
            segment_start_days_bits: sample.segment_start.julian_day.days().to_bits(),
            segment_start_scale: sample.segment_start.scale,
            segment_end_days_bits: sample.segment_end.julian_day.days().to_bits(),
            segment_end_scale: sample.segment_end.scale,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PackagedArtifactFitChannelFamilyAccumulator {
    sample_count: usize,
    worst_sample: Option<PackagedArtifactFitSample>,
}

impl PackagedArtifactFitChannelFamilyAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            sample_count: 0,
            worst_sample: None,
        }
    }

    pub(crate) fn push(&mut self, sample: &PackagedArtifactFitSample, channel: ChannelKind) {
        self.sample_count += 1;
        let should_replace = self
            .worst_sample
            .as_ref()
            .map(|existing| {
                let existing_delta = packaged_artifact_fit_channel_delta(existing, channel);
                let candidate_delta = packaged_artifact_fit_channel_delta(sample, channel);
                candidate_delta > existing_delta
                    || (candidate_delta == existing_delta
                        && (sample.segment_end.julian_day.days()
                            - sample.segment_start.julian_day.days())
                            < (existing.segment_end.julian_day.days()
                                - existing.segment_start.julian_day.days()))
            })
            .unwrap_or(true);

        if should_replace {
            self.worst_sample = Some(sample.clone());
        }
    }

    pub(crate) fn finish(self, channel: ChannelKind) -> Option<PackagedArtifactFitChannelOutlier> {
        self.worst_sample.as_ref().map(|sample| {
            PackagedArtifactFitChannelOutlier::from_sample(sample, channel, self.sample_count)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitChannelOutlier {
    /// Channel whose fit error is being tracked.
    pub channel: ChannelKind,
    /// Absolute delta for the channel.
    pub delta: f64,
    /// Inclusive segment start for the source interval.
    pub segment_start: Instant,
    /// Inclusive segment end for the source interval.
    pub segment_end: Instant,
    /// Segment span in days.
    pub segment_span_days: f64,
    /// Sample instant that produced the tracked delta.
    pub sample_instant: Instant,
    /// Sample position inside the segment, expressed as a normalized fraction.
    pub sample_fraction: f64,
    /// Number of fit samples that shared the same body/channel/segment family.
    pub sample_count: usize,
}

impl PackagedArtifactFitChannelOutlier {
    fn from_sample(
        sample: &PackagedArtifactFitSample,
        channel: ChannelKind,
        sample_count: usize,
    ) -> Self {
        Self {
            channel,
            delta: packaged_artifact_fit_channel_delta(sample, channel),
            segment_start: sample.segment_start,
            segment_end: sample.segment_end,
            segment_span_days: sample.segment_end.julian_day.days()
                - sample.segment_start.julian_day.days(),
            sample_instant: sample.sample_instant,
            sample_fraction: sample.sample_fraction,
            sample_count,
        }
    }

    fn delta_unit(&self) -> &'static str {
        match self.channel {
            ChannelKind::Longitude | ChannelKind::Latitude => "°",
            ChannelKind::DistanceAu => " AU",
            _ => unreachable!("unsupported packaged-artifact channel kind"),
        }
    }

    fn summary_line(&self) -> String {
        format!(
            "{}={:.12}{} @ {} (segment {} → {}, span={:.12} d, x={:.3}, samples={})",
            self.channel,
            self.delta,
            self.delta_unit(),
            self.sample_instant,
            self.segment_start,
            self.segment_end,
            self.segment_span_days,
            self.sample_fraction,
            self.sample_count,
        )
    }
}

impl fmt::Display for PackagedArtifactFitChannelOutlier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitBodyOutlierSummary {
    /// Bundled body whose fit outliers are summarized.
    pub body: CelestialBody,
    /// Worst sampled delta for each stored channel.
    pub channel_outliers: Vec<PackagedArtifactFitChannelOutlier>,
}

impl PackagedArtifactFitBodyOutlierSummary {
    pub(crate) fn summary_line(&self) -> String {
        format!("{}{{{}}}", self.body, join_display(&self.channel_outliers))
    }
}

impl fmt::Display for PackagedArtifactFitBodyOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitOutlierSummary {
    /// Number of bundled bodies represented in the outlier report.
    pub body_count: usize,
    /// Body-level summaries for the worst sampled fit deltas.
    pub body_summaries: Vec<PackagedArtifactFitBodyOutlierSummary>,
}

impl PackagedArtifactFitOutlierSummary {
    /// Returns the body/channel fit outliers as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "fit outliers: {} bundled bodies; {}",
            self.body_count,
            join_display(&self.body_summaries)
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged artifact.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitEnvelopeSummaryValidationError> {
        let expected = packaged_artifact_fit_outlier_summary_details();
        if self != &expected {
            return Err(
                PackagedArtifactFitEnvelopeSummaryValidationError::FieldOutOfSync {
                    field: "outlier summary",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body/channel fit outliers as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitEnvelopeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactFitChannelOutlierSummary {
    /// Channel-level summaries for the worst sampled fit deltas.
    pub channel_summaries: Vec<String>,
}

/// Validation error for a packaged-artifact fit outlier-by-channel summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactFitChannelOutlierSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl PackagedArtifactFitChannelOutlierSummaryValidationError {
    /// Returns the compact release-facing summary for the validation error.
    pub fn summary_line(&self) -> String {
        match self {
            Self::FieldOutOfSync { field } => format!(
                "the packaged artifact fit outlier-by-channel summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl fmt::Display for PackagedArtifactFitChannelOutlierSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for PackagedArtifactFitChannelOutlierSummaryValidationError {}

impl PackagedArtifactFitChannelOutlierSummary {
    /// Returns the channel-level fit outliers as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        if self.channel_summaries.is_empty() {
            "fit outliers by channel: none".to_string()
        } else {
            format!(
                "fit outliers by channel: {}",
                self.channel_summaries.join("; ")
            )
        }
    }

    /// Returns `Ok(())` when the summary still matches the current packaged artifact.
    pub fn validate(&self) -> Result<(), PackagedArtifactFitChannelOutlierSummaryValidationError> {
        let expected = packaged_artifact_fit_channel_outlier_summary_details();
        if self != &expected {
            return Err(
                PackagedArtifactFitChannelOutlierSummaryValidationError::FieldOutOfSync {
                    field: "channel_summaries",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated channel-level fit outliers as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactFitChannelOutlierSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactFitChannelOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn packaged_artifact_fit_channel_outlier_summary_for_channel(
    samples: &[PackagedArtifactFitSample],
    channel: ChannelKind,
) -> Option<String> {
    let mut families: HashMap<
        (CelestialBody, PackagedArtifactFitSegmentFamilyKey),
        PackagedArtifactFitChannelFamilyAccumulator,
    > = HashMap::new();

    for sample in samples {
        let family_key = PackagedArtifactFitSegmentFamilyKey::from_sample(sample);
        let entry = families
            .entry((sample.body.clone(), family_key))
            .or_insert_with(PackagedArtifactFitChannelFamilyAccumulator::new);
        entry.push(sample, channel);
    }

    let mut body_outliers: HashMap<CelestialBody, PackagedArtifactFitChannelOutlier> =
        HashMap::new();

    for ((body, _family_key), family) in families {
        let Some(candidate) = family.finish(channel) else {
            continue;
        };
        match body_outliers.get_mut(&body) {
            Some(existing) if existing.delta > candidate.delta => {}
            Some(existing)
                if existing.delta == candidate.delta
                    && existing.segment_span_days <= candidate.segment_span_days => {}
            Some(existing) => *existing = candidate,
            None => {
                body_outliers.insert(body, candidate);
            }
        }
    }

    if body_outliers.is_empty() {
        return None;
    }

    let mut body_entries = body_outliers
        .into_iter()
        .map(|(body, outlier)| format!("{body}{{{outlier}}}"))
        .collect::<Vec<_>>();
    body_entries.sort();

    Some(format!("{channel}{{{}}}", body_entries.join(", ")))
}

/// Returns the current packaged-artifact fit outliers by channel as a structured summary record.
pub fn packaged_artifact_fit_channel_outlier_summary_details(
) -> PackagedArtifactFitChannelOutlierSummary {
    let samples = packaged_artifact_fit_outlier_samples_for_current_artifact();
    let mut channel_summaries = Vec::new();

    for channel in [
        ChannelKind::DistanceAu,
        ChannelKind::Longitude,
        ChannelKind::Latitude,
    ] {
        if let Some(entry) =
            packaged_artifact_fit_channel_outlier_summary_for_channel(samples, channel)
        {
            channel_summaries.push(entry);
        }
    }

    PackagedArtifactFitChannelOutlierSummary { channel_summaries }
}

/// Returns the current packaged-artifact fit outliers by channel after validating the structured posture.
pub fn packaged_artifact_fit_channel_outlier_summary_for_report() -> String {
    let summary = packaged_artifact_fit_channel_outlier_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit outliers by channel: unavailable ({error})"),
    }
}

// SP1 draft baseline: thresholds are pinned to the measured worst case (overall
// envelope and the worst per-scope envelope) of the regenerated dense de440-backed
// draft artifact. Each body is measured against the SAME source it was fit from:
// major bodies against the dense de440 production reference corpus (1900–2100, ≥3
// entries/body, no extrapolation), selected-asteroid/custom bodies against the
// JplSnapshotBackend reference snapshot they were fit against. So the posture
// reflects measured reality, not an enforced target. These are sample-residual
// envelopes (not the per-body hold-out accuracy); they are finite and bounded.
// SP2 tunes these toward real accuracy goals.
pub(crate) const PACKAGED_ARTIFACT_FIT_MAX_MEAN_LONGITUDE_DELTA_DEGREES: f64 = 79.29937281518961;
pub(crate) const PACKAGED_ARTIFACT_FIT_MAX_MEAN_LATITUDE_DELTA_DEGREES: f64 = 3.32091915943210658213;
pub(crate) const PACKAGED_ARTIFACT_FIT_MAX_MEAN_DISTANCE_DELTA_AU: f64 = 5240.2473102557;
pub(crate) const PACKAGED_ARTIFACT_FIT_MAX_LONGITUDE_DELTA_DEGREES: f64 = 179.999799204804475;
pub(crate) const PACKAGED_ARTIFACT_FIT_MAX_LATITUDE_DELTA_DEGREES: f64 = 69.9571184739231;
pub(crate) const PACKAGED_ARTIFACT_FIT_MAX_DISTANCE_DELTA_AU: f64 = 10227288.989857685;

pub(crate) const PACKAGED_ARTIFACT_FIT_THRESHOLD_SUMMARY: PackagedArtifactFitThresholdSummary =
    PackagedArtifactFitThresholdSummary {
        max_mean_longitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_MEAN_LONGITUDE_DELTA_DEGREES,
        max_mean_latitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_MEAN_LATITUDE_DELTA_DEGREES,
        max_mean_distance_delta_au: PACKAGED_ARTIFACT_FIT_MAX_MEAN_DISTANCE_DELTA_AU,
        max_longitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_LONGITUDE_DELTA_DEGREES,
        max_latitude_delta_degrees: PACKAGED_ARTIFACT_FIT_MAX_LATITUDE_DELTA_DEGREES,
        max_distance_delta_au: PACKAGED_ARTIFACT_FIT_MAX_DISTANCE_DELTA_AU,
    };
