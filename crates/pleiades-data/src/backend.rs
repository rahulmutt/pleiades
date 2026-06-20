use std::sync::Arc;

#[cfg(feature = "packaged-artifact-path")]
use std::path::Path;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    CelestialBody, CoordinateFrame, EphemerisBackend, EphemerisError, EphemerisErrorKind,
    EphemerisRequest, EphemerisResult, QualityAnnotation, TimeScale, ZodiacMode,
};
use pleiades_compression::CompressedArtifact;

use crate::coverage::packaged_body_coverage_summary;
#[cfg(feature = "packaged-artifact-path")]
use crate::data::packaged_artifact_from_path;
use crate::data::{packaged_artifact, packaged_artifact_from_bytes, PackagedArtifactLoadError};
use crate::lookup::{
    packaged_artifact_access_summary_for_report, packaged_artifact_storage_summary_for_report,
    packaged_frame_treatment_summary_for_report, packaged_request_policy_summary_for_report,
};
use crate::regenerate::{artifact_time_range, map_artifact_error, normalize_lookup_instant};
use crate::PACKAGE_NAME;

/// A packaged compressed-data backend.
#[derive(Debug, Clone)]
pub struct PackagedDataBackend {
    artifact: Arc<CompressedArtifact>,
}

impl Default for PackagedDataBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PackagedDataBackend {
    /// Creates a new packaged-data backend backed by the checked-in fixture.
    pub fn new() -> Self {
        Self::from_artifact(packaged_artifact().clone())
    }

    /// Creates a packaged-data backend from an explicit artifact.
    pub fn from_artifact(artifact: CompressedArtifact) -> Self {
        Self {
            artifact: Arc::new(artifact),
        }
    }

    /// Creates a packaged-data backend from decoded artifact bytes.
    ///
    /// See [`crate::packaged_backend_from_bytes`] for an end-to-end example that
    /// encodes the checked-in artifact and reloads it through this constructor.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackagedArtifactLoadError> {
        Ok(Self::from_artifact(
            packaged_artifact_from_bytes(bytes).map_err(PackagedArtifactLoadError::Decode)?,
        ))
    }

    #[cfg(feature = "packaged-artifact-path")]
    /// Creates a packaged-data backend from an artifact file.
    ///
    /// See [`crate::packaged_backend_from_path`] for an end-to-end example that writes
    /// the checked-in artifact to a temporary file and reloads it through this
    /// constructor.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, PackagedArtifactLoadError> {
        Ok(Self::from_artifact(packaged_artifact_from_path(path)?))
    }

    fn artifact(&self) -> &CompressedArtifact {
        &self.artifact
    }
}

impl EphemerisBackend for PackagedDataBackend {
    fn metadata(&self) -> BackendMetadata {
        let artifact = self.artifact();
        let bodies = artifact
            .bodies
            .iter()
            .map(|series| series.body.clone())
            .collect::<Vec<_>>();
        let range = artifact_time_range(artifact);

        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: format!(
                "{} checksum:{:016x}",
                artifact.header.version, artifact.checksum
            ),
            family: BackendFamily::CompressedData,
            provenance: BackendProvenance {
                summary: artifact.header.source.clone(),
                data_sources: vec![
                    packaged_body_coverage_summary(),
                    packaged_request_policy_summary_for_report(),
                    packaged_frame_treatment_summary_for_report(),
                    packaged_artifact_storage_summary_for_report(),
                    packaged_artifact_access_summary_for_report(),
                ],
            },
            nominal_range: range,
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
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.artifact
            .bodies
            .iter()
            .any(|series| series.body == body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !matches!(req.instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "packaged data only supports TT or TDB requests",
            ));
        }

        validate_request_policy(
            req,
            "packaged data",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            true,
            false,
        )?;

        validate_zodiac_policy(req, "packaged data", &[ZodiacMode::Tropical])?;

        validate_observer_policy(req, "packaged data", false)?;

        let lookup_instant = normalize_lookup_instant(req.instant);
        let ecliptic = self
            .artifact
            .lookup_ecliptic(&req.body, lookup_instant)
            .map_err(map_artifact_error)?;
        let equatorial = ecliptic.to_equatorial(req.instant.mean_obliquity());
        let motion = self
            .artifact
            .lookup_motion(&req.body, lookup_instant)
            .map_err(map_artifact_error)?;

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(ecliptic);
        result.equatorial = Some(equatorial);
        result.motion = Some(motion);
        result.quality = QualityAnnotation::Interpolated;
        Ok(result)
    }
}
