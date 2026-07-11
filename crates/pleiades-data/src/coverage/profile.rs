use super::*;

/// Structured production-profile skeleton for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactProductionProfileSummary {
    /// Stable identifier for the production-profile skeleton.
    pub profile_id: &'static str,
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Covered time range for the packaged artifact.
    pub time_range: TimeRange,
    /// Provenance summary for the checked-in production-generation corpus.
    pub source_provenance: String,
    /// Bodies bundled into the packaged artifact.
    pub body_coverage: PackagedBodyCoverageSummary,
    /// Capability profile encoded by the packaged artifact.
    pub artifact_profile: ArtifactProfile,
    /// Output speed policy encoded by the packaged artifact.
    pub speed_policy: pleiades_compression::SpeedPolicy,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Request policy encoded by the packaged artifact.
    pub request_policy: PackagedRequestPolicySummary,
    /// Lookup-epoch policy encoded by the packaged artifact.
    pub lookup_epoch_policy: PackagedLookupEpochPolicy,
    /// Frame-treatment policy encoded by the packaged artifact.
    pub frame_treatment: PackagedFrameTreatmentSummary,
    /// Storage/reconstruction policy encoded by the packaged artifact.
    pub storage_summary: PackagedArtifactStorageSummary,
    /// Release-facing statement about the packaged-artifact target thresholds.
    pub target_thresholds: PackagedArtifactTargetThresholdSummary,
}

/// Validation error for a packaged artifact production-profile skeleton that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactProductionProfileSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactProductionProfileSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact production profile summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactProductionProfileSummaryValidationError {}

impl PackagedArtifactProductionProfileSummary {
    /// Returns the production-profile skeleton as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact production profile draft: profile id={}; label={}; version={}; time range={}; source provenance={}; body coverage={}; artifact profile={}; output support={}; speed policy={}; generation policy={}; segment strategy={}; request policy={}; lookup epoch policy={}; frame treatment={}; storage/reconstruction={}; {}",
            self.profile_id,
            self.label,
            self.artifact_version,
            self.time_range,
            self.source_provenance,
            self.body_coverage,
            self.artifact_profile,
            self.artifact_profile.output_support_entries_summary_line(),
            self.speed_policy,
            self.generation_policy,
            self.generation_policy.segment_strategy(),
            self.request_policy,
            self.lookup_epoch_policy.summary_line(),
            self.frame_treatment,
            self.storage_summary,
            self.target_thresholds,
        )
    }

    /// Returns `Ok(())` when the production-profile skeleton still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactProductionProfileSummaryValidationError> {
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "profile_id",
                },
            );
        }
        if self.label != ARTIFACT_LABEL {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "label",
                },
            );
        }
        if self.artifact_version != packaged_artifact().header.version {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "artifact_version",
                },
            );
        }
        if self.time_range != artifact_time_range(packaged_artifact()) {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "time_range",
                },
            );
        }
        if self.source_provenance != production_generation_source_summary_for_report() {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "source_provenance",
                },
            );
        }
        self.body_coverage.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "body_coverage",
            }
        })?;
        if self.artifact_profile != packaged_artifact_profile_summary_details().profile {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "artifact_profile",
                },
            );
        }
        if self.speed_policy != self.artifact_profile.speed_policy {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "speed_policy",
                },
            );
        }
        self.generation_policy.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "generation_policy",
            }
        })?;
        self.request_policy.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "request_policy",
            }
        })?;
        if self.lookup_epoch_policy != packaged_lookup_epoch_policy_summary_details().policy {
            return Err(
                PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                    field: "lookup_epoch_policy",
                },
            );
        }
        self.frame_treatment.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "frame_treatment",
            }
        })?;
        self.storage_summary.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "storage_summary",
            }
        })?;
        self.target_thresholds.validate().map_err(|_| {
            PackagedArtifactProductionProfileSummaryValidationError::FieldOutOfSync {
                field: "target_thresholds",
            }
        })?;

        Ok(())
    }

    /// Returns the validated production-profile skeleton summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactProductionProfileSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactProductionProfileSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact production-profile skeleton.
pub fn packaged_artifact_production_profile_summary_details(
) -> PackagedArtifactProductionProfileSummary {
    let artifact = packaged_artifact();
    let profile_summary = packaged_artifact_profile_summary_details();
    let speed_policy = profile_summary.profile.speed_policy;
    let summary = PackagedArtifactProductionProfileSummary {
        profile_id: ARTIFACT_PROFILE_ID,
        label: ARTIFACT_LABEL,
        artifact_version: artifact.header.version,
        time_range: artifact_time_range(artifact),
        source_provenance: production_generation_source_summary_for_report(),
        body_coverage: packaged_body_coverage_summary_details(),
        artifact_profile: profile_summary.profile,
        speed_policy,
        generation_policy: PackagedArtifactGenerationPolicy::AdjacentSameBodyQuadraticWindows,
        request_policy: packaged_request_policy_summary_details(),
        lookup_epoch_policy: packaged_lookup_epoch_policy_summary_details().policy,
        frame_treatment: packaged_frame_treatment_summary_details(),
        storage_summary: packaged_artifact_storage_summary_details(),
        target_thresholds: packaged_artifact_target_threshold_summary_details(),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact production-profile draft summary.
pub fn packaged_artifact_production_profile_summary() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_production_profile_summary_details();
            match summary.validated_summary_line() {
                Ok(rendered) => rendered,
                Err(error) => {
                    format!("Packaged artifact production profile draft: unavailable ({error})")
                }
            }
        })
        .as_str()
}

/// Structured generation parameters for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactGeneratorParameters {
    /// Stable identifier for the generation profile.
    pub profile_id: &'static str,
    /// Human-readable generation label.
    pub label: &'static str,
    /// Version of the packaged artifact format.
    pub artifact_version: u16,
    /// Covered time range for the packaged artifact.
    pub time_range: TimeRange,
    /// Provenance summary for the checked-in production-generation corpus.
    pub source_provenance: String,
    /// Deterministic checksum of the checked-in packaged artifact.
    pub checksum: u64,
    /// Encoded size of the checked-in packaged artifact in bytes.
    pub artifact_size_bytes: usize,
    /// Bodies bundled into the packaged artifact.
    pub body_coverage: PackagedBodyCoverageSummary,
    /// Residual-bearing body coverage encoded by the packaged artifact.
    pub residual_body_coverage: ArtifactResidualBodyCoverageSummary,
    /// Capability profile encoded by the packaged artifact.
    pub artifact_profile: ArtifactProfile,
    /// Output speed policy encoded by the packaged artifact.
    pub speed_policy: pleiades_compression::SpeedPolicy,
    /// Generation policy used to turn reference snapshots into segments.
    pub generation_policy: PackagedArtifactGenerationPolicy,
    /// Request policy encoded by the packaged artifact.
    pub request_policy: PackagedRequestPolicySummary,
    /// Lookup-epoch policy encoded by the packaged artifact.
    pub lookup_epoch_policy: PackagedLookupEpochPolicy,
    /// Frame-treatment policy encoded by the packaged artifact.
    pub frame_treatment: PackagedFrameTreatmentSummary,
    /// Storage/reconstruction policy encoded by the packaged artifact.
    pub storage_summary: PackagedArtifactStorageSummary,
    /// Release-facing statement about the packaged-artifact target thresholds.
    pub target_thresholds: PackagedArtifactTargetThresholdSummary,
}

impl PackagedArtifactGeneratorParameters {
    /// Returns the generator parameters as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact generator parameters: profile id={}; label={}; version={}; time range={}; source provenance={}; checksum=0x{:016x}; artifact size={} bytes; body coverage={}; residual bodies={}; artifact profile={}; output support={}; speed policy={}; generation policy={}; segment strategy={}; request policy={}; lookup epoch policy={}; frame treatment={}; storage/reconstruction={}; {}",
            self.profile_id,
            self.label,
            self.artifact_version,
            self.time_range,
            self.source_provenance,
            self.checksum,
            self.artifact_size_bytes,
            self.body_coverage,
            self.residual_body_coverage.summary_line_with_body_count(),
            self.artifact_profile,
            self.artifact_profile.output_support_entries_summary_line(),
            self.speed_policy,
            self.generation_policy,
            self.generation_policy.segment_strategy(),
            self.request_policy,
            self.lookup_epoch_policy.summary_line(),
            self.frame_treatment,
            self.storage_summary,
            self.target_thresholds,
        )
    }

    /// Returns `Ok(())` when the parameters still match the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let current = packaged_artifact_production_profile_summary_details();
        let artifact = packaged_artifact();

        if self.profile_id != current.profile_id {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters profile id does not match the current production profile",
            ));
        }
        if self.label != current.label {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters label does not match the current production profile",
            ));
        }
        if self.artifact_version != current.artifact_version {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters version does not match the current production profile",
            ));
        }
        if self.time_range != current.time_range {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters time range does not match the current production profile",
            ));
        }
        if self.source_provenance != current.source_provenance {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters source provenance does not match the current production profile",
            ));
        }
        if self.checksum != artifact.checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters checksum does not match the current packaged artifact",
            ));
        }
        let expected_artifact_size_bytes = packaged_artifact_encoded_bytes(artifact);
        if self.artifact_size_bytes != expected_artifact_size_bytes {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact generator parameters artifact size {} bytes does not match the current packaged artifact size {} bytes",
                    self.artifact_size_bytes,
                    expected_artifact_size_bytes
                ),
            ));
        }
        if self.body_coverage != current.body_coverage {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters body coverage does not match the current production profile",
            ));
        }
        if self.residual_body_coverage != artifact.residual_body_coverage_summary() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters residual body coverage does not match the current packaged artifact",
            ));
        }
        if self.artifact_profile != current.artifact_profile {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters artifact profile does not match the current production profile",
            ));
        }
        if self.speed_policy != current.speed_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters speed policy does not match the current production profile",
            ));
        }
        if self.generation_policy != current.generation_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters generation policy does not match the current production profile",
            ));
        }
        if self.request_policy != current.request_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters request policy does not match the current production profile",
            ));
        }
        if self.lookup_epoch_policy != current.lookup_epoch_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters lookup epoch policy does not match the current production profile",
            ));
        }
        if self.frame_treatment != current.frame_treatment {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters frame-treatment policy does not match the current production profile",
            ));
        }
        if self.storage_summary != current.storage_summary {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters storage summary does not match the current production profile",
            ));
        }
        self.target_thresholds
            .state
            .validate_production_ready()
            .map_err(|_| {
                pleiades_compression::CompressionError::new(
                    pleiades_compression::CompressionErrorKind::InvalidFormat,
                    "packaged artifact generator parameters target thresholds do not match the current production profile",
                )
            })?;
        if self.target_thresholds != current.target_thresholds {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact generator parameters target thresholds do not match the current production profile",
            ));
        }

        Ok(())
    }

    /// Returns the validated generator parameters summary line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactGeneratorParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn packaged_artifact_generation_manifest_checksum(
    parameters: &PackagedArtifactGeneratorParameters,
    regeneration: &PackagedArtifactRegenerationSummary,
) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(parameters.summary_line().as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(regeneration.summary_line().as_bytes());
    fnv1a64(&bytes)
}

/// Structured deterministic manifest for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactGenerationManifest {
    /// Generator parameters used to produce the packaged artifact.
    pub parameters: PackagedArtifactGeneratorParameters,
    /// Regeneration provenance anchored to the checked-in artifact and source snapshot.
    pub regeneration: PackagedArtifactRegenerationSummary,
    /// Deterministic checksum of the rendered manifest content.
    pub manifest_checksum: u64,
}

impl PackagedArtifactGenerationManifest {
    /// Returns the deterministic manifest as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged artifact generation manifest: manifest checksum=0x{:016x}; {}; regeneration={}",
            self.manifest_checksum, self.parameters, self.regeneration,
        )
    }

    /// Returns `Ok(())` when the manifest still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        self.parameters.validate()?;
        self.regeneration.validate()?;

        let expected_checksum =
            packaged_artifact_generation_manifest_checksum(&self.parameters, &self.regeneration);
        if self.manifest_checksum != expected_checksum {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact generation manifest checksum 0x{:016x} does not match the current packaged-artifact manifest checksum 0x{:016x}",
                    self.manifest_checksum,
                    expected_checksum
                ),
            ));
        }

        Ok(())
    }

    /// Returns the validated manifest summary line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactGenerationManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact generator parameters.
pub fn packaged_artifact_generator_parameters_details() -> PackagedArtifactGeneratorParameters {
    let summary = packaged_artifact_production_profile_summary_details();
    let regeneration = packaged_artifact_regeneration_summary_details();
    let parameters = PackagedArtifactGeneratorParameters {
        profile_id: summary.profile_id,
        label: summary.label,
        artifact_version: summary.artifact_version,
        time_range: summary.time_range,
        source_provenance: summary.source_provenance,
        checksum: regeneration.checksum,
        artifact_size_bytes: regeneration.artifact_size_bytes,
        body_coverage: summary.body_coverage,
        residual_body_coverage: regeneration.residual_body_coverage_summary(),
        artifact_profile: summary.artifact_profile,
        speed_policy: summary.speed_policy,
        generation_policy: summary.generation_policy,
        request_policy: summary.request_policy,
        lookup_epoch_policy: summary.lookup_epoch_policy,
        frame_treatment: summary.frame_treatment,
        storage_summary: summary.storage_summary,
        target_thresholds: summary.target_thresholds,
    };
    debug_assert!(parameters.validate().is_ok());
    parameters
}

/// Returns the current deterministic packaged-artifact generation manifest.
pub fn packaged_artifact_generation_manifest_details() -> PackagedArtifactGenerationManifest {
    let parameters = packaged_artifact_generator_parameters_details();
    let regeneration = packaged_artifact_regeneration_summary_details();
    let manifest = PackagedArtifactGenerationManifest {
        manifest_checksum: packaged_artifact_generation_manifest_checksum(
            &parameters,
            &regeneration,
        ),
        parameters,
        regeneration,
    };
    debug_assert!(manifest.validate().is_ok());
    manifest
}

/// Returns the current deterministic packaged-artifact generation manifest.
pub fn packaged_artifact_generation_manifest() -> &'static str {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = packaged_artifact_generation_manifest_details();
            match manifest.validated_summary_line() {
                Ok(rendered) => rendered,
                Err(error) => {
                    format!("Packaged artifact generation manifest: unavailable ({error})")
                }
            }
        })
        .as_str()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactProfileSummary {
    /// Number of bundled bodies that share the packaged artifact profile.
    pub body_count: usize,
    /// Bodies bundled under the packaged artifact profile.
    pub bodies: Vec<CelestialBody>,
    /// Byte-order policy encoded by the packaged artifact.
    pub endian_policy: EndianPolicy,
    /// Capability profile encoded by the packaged artifact.
    pub profile: ArtifactProfile,
}

impl PackagedArtifactProfileSummary {
    /// Returns the packaged artifact profile coverage as a typed summary.
    pub fn profile_coverage_summary(&self) -> ArtifactProfileCoverageSummary {
        ArtifactProfileCoverageSummary::new(self.profile.clone(), self.bodies.clone())
    }

    /// Validates that the packaged artifact profile summary is internally
    /// consistent with its bundled body list, byte-order policy, and embedded capability profile.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        let artifact = packaged_artifact();
        if self.endian_policy != artifact.header.endian_policy {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact profile byte-order policy does not match the checked-in packaged artifact header",
            ));
        }
        if self.profile != artifact.profile_coverage_summary().profile {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact profile metadata does not match the checked-in packaged artifact profile",
            ));
        }

        if self.body_count != self.bodies.len() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                "packaged artifact profile body count does not match bundled body list",
            ));
        }

        if self.bodies.is_empty() {
            let coverage = self.profile_coverage_summary();
            coverage.validate()?;
            return Ok(());
        }

        if self
            .bodies
            .iter()
            .enumerate()
            .any(|(index, body)| self.bodies[..index].contains(body))
        {
            let coverage = self.profile_coverage_summary();
            coverage.validate()?;
            return Ok(());
        }

        if self.bodies.as_slice() != packaged_bodies() {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact profile bundled body list does not match the checked-in packaged body set: expected [{}]; got [{}]",
                    join_display(packaged_bodies()),
                    join_display(&self.bodies)
                ),
            ));
        }

        let coverage = self.profile_coverage_summary();
        coverage.validate()?;

        Ok(())
    }

    /// Returns the validated packaged artifact profile summary line.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the validated packaged artifact profile summary line with bundled bodies.
    pub fn validated_summary_line_with_bodies(
        &self,
    ) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line_with_bodies())
    }

    /// Returns the validated packaged artifact profile summary line with output support.
    pub fn validated_summary_line_with_output_support(
        &self,
    ) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(format!(
            "{}; output support: {}",
            self.summary_line_with_bodies(),
            self.profile.validated_output_support_summary_line()?
        ))
    }

    /// Renders the packaged artifact profile into a release-facing summary line.
    pub fn summary_line(&self) -> String {
        let coverage = self.profile_coverage_summary();
        format!(
            "byte order: {}; {}",
            self.endian_policy,
            coverage.summary_line()
        )
    }

    /// Returns the packaged artifact profile's output-support summary line.
    pub fn output_support_summary_line(&self) -> String {
        self.profile.output_support_summary_line()
    }

    /// Renders the packaged artifact profile with its bundled body list.
    pub fn summary_line_with_bodies(&self) -> String {
        let coverage = self.profile_coverage_summary();
        format!(
            "byte order: {}; {}",
            self.endian_policy,
            coverage.summary_line_with_bodies(),
        )
    }

    /// Renders the packaged artifact profile together with the built-in output
    /// support posture used by the current packaged artifact.
    pub fn summary_line_with_output_support(&self) -> String {
        format!(
            "{}; output support: {}",
            self.summary_line_with_bodies(),
            self.output_support_summary_line()
        )
    }
}

impl fmt::Display for PackagedArtifactProfileSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact profile summary record.
pub fn packaged_artifact_profile_summary_details() -> PackagedArtifactProfileSummary {
    let artifact = packaged_artifact();
    let coverage = artifact.profile_coverage_summary();
    let summary = PackagedArtifactProfileSummary {
        body_count: coverage.body_count,
        bodies: coverage.bodies,
        endian_policy: artifact.header.endian_policy,
        profile: coverage.profile,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the current packaged-artifact profile coverage summary record.
pub fn packaged_artifact_profile_coverage_summary_details() -> ArtifactProfileCoverageSummary {
    packaged_artifact_profile_summary_details().profile_coverage_summary()
}

/// Returns the current packaged-artifact profile summary.
///
/// The summary is validated before it is rendered so release-facing callers
/// see an explicit unavailable marker if the bundled profile metadata drifts.
pub fn packaged_artifact_profile_summary() -> String {
    render_packaged_artifact_profile_summary(&packaged_artifact_profile_summary_details(), false)
}

/// Returns the current packaged-artifact profile summary with bundled body coverage.
///
/// The summary is validated before it is rendered so release-facing callers
/// see an explicit unavailable marker if the bundled profile metadata drifts.
pub fn packaged_artifact_profile_summary_with_body_coverage() -> String {
    render_packaged_artifact_profile_summary(&packaged_artifact_profile_summary_details(), true)
}

/// Returns the current packaged-artifact profile summary with the output-support posture.
///
/// The summary is validated before it is rendered so release-facing callers
/// see an explicit unavailable marker if the bundled profile metadata drifts.
pub fn packaged_artifact_profile_summary_with_output_support() -> String {
    let summary = packaged_artifact_profile_summary_details();
    match summary.validated_summary_line_with_output_support() {
        Ok(line) => line,
        Err(error) => {
            format!("Packaged artifact profile with output support: unavailable ({error})")
        }
    }
}

/// Structured output-support semantics for the packaged artifact profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedArtifactOutputSupportSummary {
    /// Capability profile encoded by the packaged artifact.
    pub profile: ArtifactProfile,
}

fn validate_packaged_artifact_output_support_profile(
    profile: &ArtifactProfile,
) -> Result<(), pleiades_compression::CompressionError> {
    let expected_states = [
        (
            ArtifactOutput::EclipticCoordinates,
            ArtifactOutputSupport::Derived,
        ),
        (
            ArtifactOutput::EquatorialCoordinates,
            ArtifactOutputSupport::Derived,
        ),
        (
            ArtifactOutput::ApparentCorrections,
            ArtifactOutputSupport::Unsupported,
        ),
        (
            ArtifactOutput::TopocentricCoordinates,
            ArtifactOutputSupport::Unsupported,
        ),
        (
            ArtifactOutput::SiderealCoordinates,
            ArtifactOutputSupport::Unsupported,
        ),
        (ArtifactOutput::Motion, ArtifactOutputSupport::Derived),
    ];

    for (output, expected_support) in expected_states {
        let actual_support = profile.output_support(output);
        if actual_support != expected_support {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact output support summary is out of sync with the bundled artifact profile field `output_support[{output}]`: expected {expected_support}, found {actual_support}"
                ),
            ));
        }
    }

    Ok(())
}

impl PackagedArtifactOutputSupportSummary {
    /// Validates that the embedded artifact profile is internally consistent
    /// and still advertises the packaged artifact's current built-in output
    /// support posture.
    pub fn validate(&self) -> Result<(), pleiades_compression::CompressionError> {
        self.profile.validate()?;
        validate_packaged_artifact_output_support_profile(&self.profile)
    }

    /// Returns the validated output-support posture for the packaged artifact profile.
    pub fn validated_summary_line(&self) -> Result<String, pleiades_compression::CompressionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Renders the packaged artifact profile's output-support semantics.
    pub fn summary_line(&self) -> String {
        self.profile.output_support_summary_line()
    }
}

impl fmt::Display for PackagedArtifactOutputSupportSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact output-support summary record.
pub fn packaged_artifact_output_support_summary_details() -> PackagedArtifactOutputSupportSummary {
    let summary = PackagedArtifactOutputSupportSummary {
        profile: packaged_artifact_profile_summary_details().profile,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Structured speed-policy semantics for the packaged artifact profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackagedArtifactSpeedPolicySummary {
    /// Speed policy encoded by the packaged artifact.
    pub policy: SpeedPolicy,
}

/// Validation error for the packaged-data speed-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactSpeedPolicySummaryValidationError {
    /// A summary field is out of sync with the current packaged-data posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactSpeedPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact speed-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactSpeedPolicySummaryValidationError {}

impl PackagedArtifactSpeedPolicySummary {
    /// Returns the packaged artifact speed policy as a compact human-readable line.
    pub fn summary_line(self) -> String {
        format!(
            "{}; motion output support={}",
            self.policy,
            self.policy.motion_output_support()
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-data posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactSpeedPolicySummaryValidationError> {
        let current_policy = packaged_artifact_profile_summary_details()
            .profile
            .speed_policy;
        if self.policy != current_policy {
            return Err(
                PackagedArtifactSpeedPolicySummaryValidationError::FieldOutOfSync {
                    field: "policy",
                },
            );
        }
        Ok(())
    }

    /// Returns the validated packaged artifact speed-policy summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactSpeedPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactSpeedPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const PACKAGED_ARTIFACT_SPEED_POLICY_SUMMARY: PackagedArtifactSpeedPolicySummary =
    PackagedArtifactSpeedPolicySummary {
        policy: SpeedPolicy::FittedDerivative,
    };

/// Returns the current packaged-artifact speed-policy summary record.
pub fn packaged_artifact_speed_policy_summary_details() -> PackagedArtifactSpeedPolicySummary {
    let summary = PACKAGED_ARTIFACT_SPEED_POLICY_SUMMARY;
    debug_assert!(summary.validate().is_ok());
    summary
}

pub(crate) fn render_packaged_artifact_profile_summary(
    summary: &PackagedArtifactProfileSummary,
    with_bodies: bool,
) -> String {
    if with_bodies {
        match summary.validated_summary_line_with_bodies() {
            Ok(line) => line,
            Err(error) => {
                format!("Packaged artifact profile with bundled bodies: unavailable ({error})")
            }
        }
    } else {
        match summary.validated_summary_line() {
            Ok(line) => line,
            Err(error) => format!("Packaged artifact profile: unavailable ({error})"),
        }
    }
}
