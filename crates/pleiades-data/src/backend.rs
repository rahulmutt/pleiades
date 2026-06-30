use std::sync::Arc;

#[cfg(feature = "packaged-artifact-path")]
use std::path::Path;

use pleiades_apsides::{apsides, MU_EARTH_MOON_AU3_PER_DAY2};
use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    CelestialBody, CoordinateFrame, EclipticCoordinates, EphemerisBackend, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, EphemerisResult, Instant, JulianDay, Latitude, Longitude,
    Motion, QualityAnnotation, TimeScale, ZodiacMode,
};
use pleiades_compression::{spherical_state_to_cartesian, CompressedArtifact, SphericalState};

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

    fn osculating_apsis_position(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        let ecliptic = self.osculating_apsis_ecliptic(&req.body, req.instant)?;
        let equatorial = ecliptic.to_equatorial(req.instant.mean_obliquity());
        let motion = self.osculating_apsis_motion(&req.body, req.instant)?;

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

    fn osculating_apsis_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        let li = normalize_lookup_instant(instant);
        let ecl = self
            .artifact
            .lookup_ecliptic(&CelestialBody::Moon, li)
            .map_err(map_artifact_error)?;
        let mot = self
            .artifact
            .lookup_motion(&CelestialBody::Moon, li)
            .map_err(map_artifact_error)?;
        let dist = ecl.distance_au.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "packaged Moon lacks distance for osculating apsis",
            )
        })?;
        let state = SphericalState {
            lon_rad: ecl.longitude.degrees().to_radians(),
            lat_rad: ecl.latitude.degrees().to_radians(),
            dist_au: dist,
            lon_rate_rad_per_day: mot.longitude_deg_per_day.unwrap_or(0.0).to_radians(),
            lat_rate_rad_per_day: mot.latitude_deg_per_day.unwrap_or(0.0).to_radians(),
            dist_rate_au_per_day: mot.distance_au_per_day.unwrap_or(0.0),
        };
        let cart = spherical_state_to_cartesian(state);
        let aps = apsides(cart.pos_au, cart.vel_au_per_day, MU_EARTH_MOON_AU3_PER_DAY2).map_err(
            |_| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "osculating apsis undefined for the lunar state at this instant",
                )
            },
        )?;
        let point = match body {
            CelestialBody::TrueApogee => aps.apogee,
            CelestialBody::TruePerigee => aps.perigee,
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "not an osculating-apsis body",
                ))
            }
        };
        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(point.longitude_deg),
            Latitude::from_degrees(point.latitude_deg),
            Some(point.distance_au),
        ))
    }

    fn osculating_apsis_motion(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<Motion, EphemerisError> {
        const HALF_SPAN_DAYS: f64 = 0.5;
        let shift = |days: f64| {
            Instant::new(
                JulianDay::from_days(instant.julian_day.days() + days),
                instant.scale,
            )
        };
        let before = self.osculating_apsis_ecliptic(body, shift(-HALF_SPAN_DAYS))?;
        let after = self.osculating_apsis_ecliptic(body, shift(HALF_SPAN_DAYS))?;
        let span = 2.0 * HALF_SPAN_DAYS;

        let mut dlon = after.longitude.degrees() - before.longitude.degrees();
        while dlon > 180.0 {
            dlon -= 360.0;
        }
        while dlon < -180.0 {
            dlon += 360.0;
        }
        let dlon_per_day = dlon / span;
        let dlat_per_day = (after.latitude.degrees() - before.latitude.degrees()) / span;
        let ddist_per_day = match (before.distance_au, after.distance_au) {
            (Some(b), Some(a)) => Some((a - b) / span),
            _ => None,
        };
        Ok(Motion::new(
            Some(dlon_per_day),
            Some(dlat_per_day),
            ddist_per_day,
        ))
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
            body_claims: {
                let declared = crate::packaged_body_claims();
                let mut claims: Vec<_> = bodies
                    .iter()
                    .map(|body| {
                        declared
                            .iter()
                            .find(|c| &c.body == body)
                            .cloned()
                            .unwrap_or_else(|| pleiades_backend::BodyClaim::from(body.clone()))
                    })
                    .collect();
                claims.extend(crate::apsis_body_claims());
                claims
            },
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
        matches!(body, CelestialBody::TrueApogee | CelestialBody::TruePerigee)
            || self
                .artifact
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

        if matches!(
            req.body,
            CelestialBody::TrueApogee | CelestialBody::TruePerigee
        ) {
            return self.osculating_apsis_position(req);
        }

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
