use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PackagedArtifactBodyCadence {
    Luminaries,
    InnerPlanets,
    OuterPlanets,
    Pluto,
    LunarPoints,
    SelectedAsteroids,
    CustomBodies,
}

impl PackagedArtifactBodyCadence {
    pub(crate) fn uses_dense_sampling(self) -> bool {
        matches!(
            self,
            Self::Luminaries
                | Self::Pluto
                | Self::LunarPoints
                | Self::SelectedAsteroids
                | Self::CustomBodies
        )
    }

    pub(crate) fn uses_dense_validation_sampling(self) -> bool {
        matches!(
            self,
            Self::Luminaries
                | Self::InnerPlanets
                | Self::OuterPlanets
                | Self::Pluto
                | Self::LunarPoints
                | Self::SelectedAsteroids
                | Self::CustomBodies
        )
    }

    pub(crate) fn uses_dense_residual_sample_lattice(self, kind: ChannelKind) -> bool {
        match kind {
            ChannelKind::Longitude | ChannelKind::Latitude => self.uses_dense_sampling(),
            ChannelKind::DistanceAu => {
                matches!(
                    self,
                    Self::InnerPlanets
                        | Self::OuterPlanets
                        | Self::Pluto
                        | Self::LunarPoints
                        | Self::SelectedAsteroids
                        | Self::CustomBodies
                )
            }
            _ => false,
        }
    }
}

pub(crate) fn packaged_artifact_body_cadence(body: &CelestialBody) -> PackagedArtifactBodyCadence {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => PackagedArtifactBodyCadence::Luminaries,
        CelestialBody::Mercury | CelestialBody::Venus | CelestialBody::Mars => {
            PackagedArtifactBodyCadence::InnerPlanets
        }
        CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune => PackagedArtifactBodyCadence::OuterPlanets,
        CelestialBody::Pluto => PackagedArtifactBodyCadence::Pluto,
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => PackagedArtifactBodyCadence::LunarPoints,
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => PackagedArtifactBodyCadence::SelectedAsteroids,
        CelestialBody::Custom(custom) if custom.catalog.eq_ignore_ascii_case("asteroid") => {
            PackagedArtifactBodyCadence::SelectedAsteroids
        }
        CelestialBody::Custom(_) => PackagedArtifactBodyCadence::CustomBodies,
        _ => PackagedArtifactBodyCadence::CustomBodies,
    }
}

pub(crate) fn packaged_artifact_target_threshold_scope_envelope_summary_details(
    scope: &'static str,
) -> PackagedArtifactTargetThresholdScopeSummary {
    let artifact = packaged_artifact();
    let bodies: Vec<CelestialBody> = artifact
        .bodies
        .iter()
        .filter(|body| packaged_artifact_body_scope(&body.body) == scope)
        .map(|body| body.body.clone())
        .collect();
    let body_count = bodies.len();
    let samples = packaged_artifact_fit_samples_for_current_artifact()
        .iter()
        .filter(|sample| packaged_artifact_body_scope(&sample.body) == scope)
        .cloned()
        .collect::<Vec<_>>();
    let expected_sample_count =
        packaged_artifact_fit_expected_sample_count_with_filter(artifact, |body| {
            packaged_artifact_body_scope(body) == scope
        });
    let fit_envelope =
        packaged_artifact_fit_envelope_summary_from_samples(&samples, expected_sample_count);
    PackagedArtifactTargetThresholdScopeSummary {
        scope,
        bodies,
        body_count,
        fit_envelope,
    }
}

fn packaged_artifact_target_threshold_scope_envelope_summaries_details(
) -> Vec<PackagedArtifactTargetThresholdScopeSummary> {
    PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES
        .iter()
        .copied()
        .map(packaged_artifact_target_threshold_scope_envelope_summary_details)
        .collect()
}

/// Returns the current packaged-artifact body-class target-threshold scope envelopes after validating the structured posture.
pub fn packaged_artifact_target_threshold_scope_envelopes_summary_details(
) -> PackagedArtifactTargetThresholdScopeEnvelopesSummary {
    PackagedArtifactTargetThresholdScopeEnvelopesSummary {
        scope_envelopes: packaged_artifact_target_threshold_scope_envelope_summaries_details(),
    }
}

pub(crate) fn format_scope_bodies(bodies: &[CelestialBody]) -> String {
    match bodies {
        [] => "0 (none)".to_string(),
        [single] => format!("1 ({single})"),
        _ => format!("{} ({})", bodies.len(), join_display(bodies)),
    }
}

pub(crate) fn packaged_artifact_quantization_scales_line() -> String {
    let artifact = packaged_artifact();
    let stored = packaged_artifact_channel_quantization_scales(artifact, false);
    let residual = packaged_artifact_channel_quantization_scales(artifact, true);

    if residual.is_empty() {
        format!("quantization scales: stored={stored}")
    } else {
        format!("quantization scales: stored={stored}; residual={residual}")
    }
}

fn packaged_artifact_channel_quantization_scales(
    artifact: &CompressedArtifact,
    residual_channels: bool,
) -> String {
    let entries = [
        ChannelKind::Longitude,
        ChannelKind::Latitude,
        ChannelKind::DistanceAu,
    ]
    .into_iter()
    .filter_map(|kind| {
        let mut exponents = artifact
            .bodies
            .iter()
            .flat_map(|body| body.segments.iter())
            .flat_map(|segment| {
                let channels = if residual_channels {
                    &segment.residual_channels
                } else {
                    &segment.channels
                };

                channels
                    .iter()
                    .filter(move |channel| channel.kind == kind)
                    .map(|channel| channel.scale_exponent)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        if exponents.is_empty() {
            None
        } else {
            exponents.sort_unstable();
            exponents.dedup();
            Some(format!("{kind}={}", join_display(&exponents)))
        }
    })
    .collect::<Vec<_>>();

    if entries.is_empty() {
        "none".to_string()
    } else {
        entries.join(", ")
    }
}

pub(crate) fn packaged_artifact_segment_span_bounds(artifact: &CompressedArtifact) -> (f64, f64) {
    let mut min_span_days: f64 = f64::INFINITY;
    let mut max_span_days: f64 = 0.0;

    for body in &artifact.bodies {
        for segment in &body.segments {
            let span_days = segment.end.julian_day.days() - segment.start.julian_day.days();
            min_span_days = min_span_days.min(span_days);
            max_span_days = max_span_days.max(span_days);
        }
    }

    if min_span_days.is_infinite() {
        (0.0, 0.0)
    } else {
        (min_span_days, max_span_days)
    }
}

pub(crate) fn packaged_artifact_channel_count(
    artifact: &CompressedArtifact,
    residual_channels: bool,
) -> usize {
    artifact
        .bodies
        .iter()
        .flat_map(|body| body.segments.iter())
        .map(|segment| {
            if residual_channels {
                segment.residual_channels.len()
            } else {
                segment.channels.len()
            }
        })
        .sum()
}

pub(crate) fn packaged_artifact_encoded_bytes(artifact: &CompressedArtifact) -> usize {
    artifact
        .encode()
        .expect("packaged artifact should be encodable")
        .len()
}

/// Structured normalized-intermediate provenance for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactNormalizedIntermediateSummary {
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Human-readable provenance/source summary.
    pub source: &'static str,
    /// Canonical source-revision summary for the checked-in production-generation corpus.
    pub source_revision: String,
    /// Stable identifier for the packaged-artifact profile.
    pub profile_id: &'static str,
    /// Covered time range for the normalized intermediate layout.
    pub time_range: TimeRange,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Per-channel quantization scales captured from the checked-in artifact.
    pub quantization_scales: String,
    /// Deterministic checksum of the rendered normalized-intermediate payload.
    pub checksum: u64,
    /// Bodies bundled into the packaged artifact.
    pub body_count: usize,
    /// Total segment count across all bundled bodies.
    pub segment_count: usize,
    /// Total count of segments carrying residual correction channels.
    pub residual_segment_count: usize,
    /// Total count of stored channels across all segments.
    pub stored_channel_count: usize,
    /// Total count of residual channels across all segments.
    pub residual_channel_count: usize,
    /// Smallest observed segment span in days.
    pub min_segment_span_days: f64,
    /// Largest observed segment span in days.
    pub max_segment_span_days: f64,
}

impl PackagedArtifactNormalizedIntermediateSummary {
    /// Returns the normalized-intermediate payload used for checksuming and rendering.
    pub(crate) fn summary_payload_line(&self) -> String {
        format!(
            "label={}; profile id={}; version={}; time range={}; source={}; source revision={}; body count={}; segments={}; residual-bearing segments={}; stored channels={}; residual channels={}; segment span days={:.12}..{:.12}; segment strategy={}; {}",
            self.label,
            self.profile_id,
            self.artifact_version,
            self.time_range,
            self.source,
            self.source_revision,
            self.body_count,
            self.segment_count,
            self.residual_segment_count,
            self.stored_channel_count,
            self.residual_channel_count,
            self.min_segment_span_days,
            self.max_segment_span_days,
            self.generation_policy.segment_strategy(),
            self.quantization_scales,
        )
    }

    /// Returns the normalized intermediates as a compact human-readable line.
    pub fn summary_fields_line(&self) -> String {
        format!(
            "{}; checksum=0x{:016x}",
            self.summary_payload_line(),
            self.checksum,
        )
    }

    /// Returns the normalized intermediates as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact normalized intermediates: {}",
            self.summary_fields_line()
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let artifact = packaged_artifact();
        if self.label != ARTIFACT_LABEL {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary label does not match the checked-in artifact label",
            ));
        }
        if self.artifact_version != artifact.header.version {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary artifact version does not match the checked-in packaged artifact version",
            ));
        }
        if self.source != packaged_artifact_source_text() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary source does not match the checked-in artifact source",
            ));
        }
        if self.source_revision != production_generation_source_summary_for_report() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary source revision does not match the checked-in production-generation source summary",
            ));
        }
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary profile id does not match the checked-in artifact profile id",
            ));
        }
        if self.time_range != artifact_time_range(artifact) {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary time range does not match the checked-in packaged artifact",
            ));
        }
        if self.generation_policy
            != PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows
        {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary generation policy does not match the checked-in packaged artifact",
            ));
        }
        if self.quantization_scales != packaged_artifact_quantization_scales_line() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary quantization scales do not match the checked-in packaged artifact",
            ));
        }
        if self.body_count != artifact.bodies.len() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary body count does not match the checked-in packaged artifact",
            ));
        }
        if self.segment_count != artifact.segment_count() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary segment count does not match the checked-in packaged artifact",
            ));
        }
        if self.residual_segment_count != artifact.residual_segment_count() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary residual segment count does not match the checked-in packaged artifact",
            ));
        }
        if self.stored_channel_count != packaged_artifact_channel_count(artifact, false) {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary stored channel count does not match the checked-in packaged artifact",
            ));
        }
        if self.residual_channel_count != packaged_artifact_channel_count(artifact, true) {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary residual channel count does not match the checked-in packaged artifact",
            ));
        }
        let (expected_min_segment_span_days, expected_max_segment_span_days) =
            packaged_artifact_segment_span_bounds(artifact);
        if self.min_segment_span_days != expected_min_segment_span_days {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary minimum segment span does not match the checked-in packaged artifact",
            ));
        }
        if self.max_segment_span_days != expected_max_segment_span_days {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact normalized intermediate summary maximum segment span does not match the checked-in packaged artifact",
            ));
        }

        let expected_checksum = fnv1a64(self.summary_payload_line().as_bytes());
        if self.checksum != expected_checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact normalized intermediate summary checksum 0x{:016x} does not match the current normalized-intermediate checksum 0x{:016x}",
                    self.checksum,
                    expected_checksum
                ),
            ));
        }

        Ok(())
    }

    /// Returns the validated normalized intermediates as a compact human-readable line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactNormalizedIntermediateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured regeneration provenance for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactRegenerationSummary {
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Human-readable provenance/source summary.
    pub source: &'static str,
    /// Canonical source-revision summary for the checked-in production-generation corpus.
    pub source_revision: String,
    /// Stable identifier for the packaged-artifact profile.
    pub profile_id: &'static str,
    /// Checksum of the checked-in packaged artifact.
    pub checksum: u64,
    /// Encoded size of the checked-in packaged artifact in bytes.
    pub artifact_size_bytes: usize,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Per-channel quantization scales captured from the checked-in artifact.
    pub quantization_scales: String,
    /// Bodies that carry residual correction channels in the packaged artifact.
    pub residual_bodies: Vec<CelestialBody>,
    /// Bodies bundled into the packaged artifact.
    pub bodies: Vec<CelestialBody>,
    /// Normalized intermediate layout captured from the checked-in artifact.
    pub normalized_intermediates: PackagedArtifactNormalizedIntermediateSummary,
    /// Fit envelope measured against the generation source samples.
    pub fit_envelope: PackagedArtifactFitEnvelopeSummary,
    /// Coverage summary for the checked-in JPL reference snapshot used for regeneration.
    pub reference_snapshot: Option<ReferenceSnapshotSummary>,
}

impl PackagedArtifactRegenerationSummary {
    /// Returns the bundled bodies as a compact human-readable line.
    pub fn body_coverage_line(&self) -> String {
        format!(
            "{} bundled bodies ({})",
            self.bodies.len(),
            join_display(&self.bodies)
        )
    }

    /// Returns the checked-in JPL snapshot coverage as a compact human-readable line.
    pub fn reference_snapshot_line(&self) -> String {
        self.reference_snapshot
            .map(|summary| format_reference_snapshot_summary(&summary))
            .unwrap_or_else(|| "Reference snapshot coverage: unavailable".to_string())
    }

    /// Returns the normalized intermediate layout as a compact human-readable line.
    pub fn normalized_intermediates_line(&self) -> String {
        self.normalized_intermediates.summary_fields_line()
    }

    /// Returns the residual-correction body coverage as a compact structured summary.
    pub fn residual_body_coverage_summary(&self) -> ArtifactResidualBodyCoverageSummary {
        ArtifactResidualBodyCoverageSummary::new(self.residual_bodies.clone())
    }

    /// Returns the residual-correction body list as a compact human-readable line.
    pub fn residual_body_line(&self) -> String {
        self.residual_body_coverage_summary()
            .summary_line_with_body_count()
    }

    /// Returns the generation policy as a compact human-readable line.
    pub fn generation_policy_line(&self) -> String {
        format!(
            "generation policy: {}",
            self.generation_policy.summary_line()
        )
    }

    /// Validates that every residual body is actually part of the bundled body list.
    pub(crate) fn validate_residual_body_subset(
        &self,
    ) -> Result<(), pleiades_compression::CompressionError> {
        for body in &self.residual_bodies {
            if !self.bodies.contains(body) {
                return Err(pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!(
                        "packaged artifact regeneration summary residual body {body} is not covered by the bundled body list"
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Validates that the regeneration summary stays aligned with the bundled
    /// body list, the current checked-in artifact metadata, and the checked-in
    /// reference snapshot coverage.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let artifact = packaged_artifact();

        if self.label != ARTIFACT_LABEL {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary label does not match the checked-in artifact label",
            ));
        }
        if self.source != packaged_artifact_source_text() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary source does not match the checked-in artifact source",
            ));
        }
        if self.source_revision != production_generation_source_summary_for_report() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary source revision does not match the checked-in production-generation source summary",
            ));
        }
        self.normalized_intermediates.validate().map_err(|error| {
            pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary normalized intermediates are invalid: {error}"
                ),
            )
        })?;
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary profile id does not match the checked-in artifact profile id",
            ));
        }
        if self.artifact_version != artifact.header.version {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary artifact version {} does not match the checked-in packaged artifact version {}",
                    self.artifact_version,
                    artifact.header.version
                ),
            ));
        }
        if self.checksum != artifact.checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary checksum 0x{:016x} does not match the checked-in packaged artifact checksum 0x{:016x}",
                    self.checksum,
                    artifact.checksum
                ),
            ));
        }
        let expected_artifact_size_bytes = packaged_artifact_encoded_bytes(artifact);
        if self.artifact_size_bytes != expected_artifact_size_bytes {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary artifact size {} bytes does not match the checked-in packaged artifact size {} bytes",
                    self.artifact_size_bytes,
                    expected_artifact_size_bytes
                ),
            ));
        }
        self.generation_policy.validate().map_err(|error| {
            pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary generation policy is invalid: {error}"
                ),
            )
        })?;

        if self.quantization_scales != packaged_artifact_quantization_scales_line() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary quantization scales do not match the checked-in packaged artifact",
            ));
        }

        self.residual_body_coverage_summary()
            .validate(artifact)
            .map_err(|error| {
                pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!(
                        "packaged artifact regeneration summary residual body coverage is invalid: {error}"
                    ),
                )
            })?;

        if self.reference_snapshot.is_none() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact regeneration summary is missing reference snapshot coverage",
            ));
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!("packaged artifact regeneration summary contains duplicate body entry {body}"),
                ));
            }
        }

        let expected_bodies = packaged_bodies();
        if self.bodies.as_slice() != expected_bodies {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration summary body list does not match the checked-in packaged body set: expected [{}]; got [{}]",
                    join_display(expected_bodies),
                    join_display(&self.bodies)
                ),
            ));
        }

        self.validate_residual_body_subset()?;

        self.fit_envelope.validate().map_err(|error| {
            pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!("packaged artifact regeneration fit envelope is invalid: {error}"),
            )
        })?;

        if let Some(reference_snapshot) = self.reference_snapshot {
            reference_snapshot.validate().map_err(|error| {
                pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    format!(
                        "packaged artifact regeneration reference snapshot is invalid: {error}"
                    ),
                )
            })?;
            for body in &self.bodies {
                if !reference_snapshot.bodies.contains(body) {
                    return Err(pleiades_compression::CompressionError::new(
                        pleiades_compression::CompressionErrorKind::InvalidFormat,
                        format!("packaged artifact regeneration body {body} is not covered by the reference snapshot"),
                    ));
                }
            }
            if self.reference_snapshot != reference_snapshot_summary() {
                return Err(pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    "packaged artifact regeneration summary reference snapshot does not match the checked-in reference snapshot summary",
                ));
            }
        }

        Ok(())
    }

    /// Returns the full packaged-artifact regeneration provenance summary.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact regeneration source: label={}; profile id={}; source={}; source revision={}; normalized intermediates: {}; checksum=0x{:016x}; artifact size={} bytes; {}; segment strategy={}; {}; {}; bundled bodies: {}; {}; fit envelope: {}; artifact version={}",
            self.label,
            self.profile_id,
            self.source,
            self.source_revision,
            self.normalized_intermediates_line(),
            self.checksum,
            self.artifact_size_bytes,
            self.generation_policy_line(),
            self.generation_policy.segment_strategy(),
            self.quantization_scales,
            self.residual_body_line(),
            self.body_coverage_line(),
            self.reference_snapshot_line(),
            self.fit_envelope.summary_line(),
            self.artifact_version,
        )
    }

    /// Returns the full packaged-artifact regeneration provenance summary after validating the structured posture.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactRegenerationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured packaged-artifact regeneration provenance.
pub fn packaged_artifact_regeneration_summary_details() -> PackagedArtifactRegenerationSummary {
    static SUMMARY: OnceLock<PackagedArtifactRegenerationSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let artifact = packaged_artifact();
            let summary = PackagedArtifactRegenerationSummary {
                label: ARTIFACT_LABEL,
                artifact_version: artifact.header.version,
                source: packaged_artifact_source_text(),
                source_revision: production_generation_source_summary_for_report(),
                profile_id: ARTIFACT_PROFILE_ID,
                checksum: artifact.checksum,
                artifact_size_bytes: packaged_artifact_encoded_bytes(artifact),
                generation_policy:
                    PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
                quantization_scales: packaged_artifact_quantization_scales_line(),
                residual_bodies: artifact.residual_bodies(),
                bodies: packaged_bodies().to_vec(),
                normalized_intermediates: packaged_artifact_normalized_intermediate_summary_details(
                ),
                fit_envelope: packaged_artifact_fit_envelope_summary_details(),
                reference_snapshot: reference_snapshot_summary(),
            };
            debug_assert!(summary.validate().is_ok());
            summary
        })
        .clone()
}

/// Returns the structured normalized-intermediate provenance.
pub fn packaged_artifact_normalized_intermediate_summary_details(
) -> PackagedArtifactNormalizedIntermediateSummary {
    let artifact = packaged_artifact();
    let payload_checksum = fnv1a64(
        format!(
            "label={}; profile id={}; version={}; time range={}; source={}; source revision={}; body count={}; segments={}; residual-bearing segments={}; stored channels={}; residual channels={}; segment span days={:.12}..{:.12}; segment strategy={}; {}",
            ARTIFACT_LABEL,
            ARTIFACT_PROFILE_ID,
            artifact.header.version,
            artifact_time_range(artifact),
            packaged_artifact_source_text(),
            production_generation_source_summary_for_report(),
            artifact.bodies.len(),
            artifact.segment_count(),
            artifact.residual_segment_count(),
            packaged_artifact_channel_count(artifact, false),
            packaged_artifact_channel_count(artifact, true),
            packaged_artifact_segment_span_bounds(artifact).0,
            packaged_artifact_segment_span_bounds(artifact).1,
            PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows.segment_strategy(),
            packaged_artifact_quantization_scales_line(),
        )
        .as_bytes(),
    );
    let summary = PackagedArtifactNormalizedIntermediateSummary {
        label: ARTIFACT_LABEL,
        artifact_version: artifact.header.version,
        source: packaged_artifact_source_text(),
        source_revision: production_generation_source_summary_for_report(),
        profile_id: ARTIFACT_PROFILE_ID,
        time_range: artifact_time_range(artifact),
        generation_policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
        quantization_scales: packaged_artifact_quantization_scales_line(),
        checksum: payload_checksum,
        body_count: artifact.bodies.len(),
        segment_count: artifact.segment_count(),
        residual_segment_count: artifact.residual_segment_count(),
        stored_channel_count: packaged_artifact_channel_count(artifact, false),
        residual_channel_count: packaged_artifact_channel_count(artifact, true),
        min_segment_span_days: packaged_artifact_segment_span_bounds(artifact).0,
        max_segment_span_days: packaged_artifact_segment_span_bounds(artifact).1,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}
