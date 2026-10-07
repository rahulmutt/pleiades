use std::sync::Arc;

#[cfg(feature = "packaged-artifact-path")]
use std::path::Path;

use pleiades_apparent::{
    precess_ecliptic_date_to_j2000, precess_ecliptic_vector_j2000_to_date, ApparentPlaceError,
};
use pleiades_apsides::{
    apsides, elements_from_state, mean_lunar_elements_of_date, points_from_elements,
    MOON_MEAN_SEMI_MAJOR_AU, MU_EARTH_MOON_AU3_PER_DAY2,
};
use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    CelestialBody, CoordinateFrame, EclipticCoordinates, EphemerisBackend, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, EphemerisResult, Instant, JulianDay, Latitude, Longitude,
    Motion, QualityAnnotation, TimeScale, ZodiacMode,
};
use pleiades_compression::{
    spherical_state_to_cartesian, CartesianState, CompressedArtifact, SphericalState,
};

use crate::coverage::packaged_body_coverage_summary_details;
#[cfg(feature = "packaged-artifact-path")]
use crate::data::packaged_artifact_from_path;
use crate::data::{packaged_artifact, packaged_artifact_from_bytes, PackagedArtifactLoadError};
use crate::lookup::{
    packaged_artifact_access_summary, packaged_artifact_storage_summary,
    packaged_frame_treatment_summary, packaged_request_policy_summary,
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

    /// Assembles a derived lunar point (osculating or mean apsis or node) into a
    /// backend result: J2000 ecliptic from `eval`, J2000 equatorial (`to_j2000_equatorial`),
    /// extrapolated central-difference motion, `Interpolated` quality.
    fn derived_point_position(
        &self,
        req: &EphemerisRequest,
        eval: &dyn Fn(Instant) -> Result<EclipticCoordinates, EphemerisError>,
    ) -> Result<EphemerisResult, EphemerisError> {
        let ecliptic = eval(req.instant)?;
        let equatorial = ecliptic.to_j2000_equatorial();
        let motion = self.derived_point_motion(req.instant, eval)?;

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

    /// The packaged Moon's geocentric J2000 Cartesian state at `instant`: the
    /// shared input of every derived lunar point.
    fn moon_state_j2000(&self, instant: Instant) -> Result<CartesianState, EphemerisError> {
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
                "packaged Moon lacks distance for a derived lunar point",
            )
        })?;
        Ok(spherical_state_to_cartesian(SphericalState {
            lon_rad: ecl.longitude.degrees().to_radians(),
            lat_rad: ecl.latitude.degrees().to_radians(),
            dist_au: dist,
            lon_rate_rad_per_day: mot.longitude_deg_per_day.unwrap_or(0.0).to_radians(),
            lat_rate_rad_per_day: mot.latitude_deg_per_day.unwrap_or(0.0).to_radians(),
            dist_rate_au_per_day: mot.distance_au_per_day.unwrap_or(0.0),
        }))
    }

    fn osculating_apsis_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        let cart = self.moon_state_j2000(instant)?;
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

    /// Osculating ascending node of the geocentric lunar orbit, J2000 boundary
    /// frame.
    ///
    /// The node is the intersection of the orbit plane with the *reference*
    /// plane, so it must be formed in the plane consumers will read it in: the
    /// mean ecliptic of date (spec §1; forming it in J2000 and rotating the
    /// point would misplace it by ≈ tilt/sin(i) ≈ 0.04°). Position and velocity
    /// are rotated J2000 → mean-of-date, the ellipse is formed there, and the
    /// node point is precessed back to J2000 like the ELP point channels
    /// (issue #57). The chart layer's forward precession + Δψ then reproduces
    /// Swiss Ephemeris `SE_TRUE_NODE`.
    fn osculating_node_ecliptic(
        &self,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        let jd_tt = instant.julian_day.days();
        let cart = self.moon_state_j2000(instant)?;
        let pos = rotate_j2000_to_mean_of_date(cart.pos_au, jd_tt)?;
        let vel = rotate_j2000_to_mean_of_date(cart.vel_au_per_day, jd_tt)?;
        let undefined = |_| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "osculating node undefined for the lunar state at this instant",
            )
        };
        let elements =
            elements_from_state(pos, vel, MU_EARTH_MOON_AU3_PER_DAY2).map_err(undefined)?;
        let node = points_from_elements(&elements, false)
            .map_err(undefined)?
            .ascending;
        let j2000 = precess_ecliptic_date_to_j2000(node.longitude_deg, node.latitude_deg, jd_tt)
            .map_err(map_precession_error)?;
        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(j2000.longitude_deg),
            Latitude::from_degrees(j2000.latitude_deg),
            Some(node.distance_au),
        ))
    }

    /// A mean lunar point (mean node, mean apogee or mean perigee), J2000
    /// boundary frame.
    ///
    /// The point is placed on the Moon's mean orbit in the mean ecliptic of
    /// date — the apsides through the inclined orbit, as Swiss Ephemeris'
    /// `SE_MEAN_APOG` is, not as the raw longitude-of-perigee element — and
    /// precessed back to J2000 like the osculating node. The elements are
    /// analytic, but the point is served only inside the packaged window so
    /// the backend's advertised range holds for every body it answers.
    fn mean_lunar_point_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        // Window probe: the Moon series spans the packaged window.
        self.artifact
            .lookup_ecliptic(&CelestialBody::Moon, normalize_lookup_instant(instant))
            .map_err(map_artifact_error)?;

        let jd_tt = instant.julian_day.days();
        let points =
            points_from_elements(&mean_lunar_elements_of_date(jd_tt), false).map_err(|_| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "mean lunar point undefined at this instant",
                )
            })?;
        let (point, distance_au) = match body {
            // Swiss Ephemeris reports the mean lunar distance for SE_MEAN_NODE,
            // not the orbit radius at the node.
            CelestialBody::MeanNode => (points.ascending, MOON_MEAN_SEMI_MAJOR_AU),
            CelestialBody::MeanApogee => (points.aphelion, points.aphelion.distance_au),
            CelestialBody::MeanPerigee => (points.perihelion, points.perihelion.distance_au),
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "not a mean lunar point",
                ))
            }
        };
        let j2000 = precess_ecliptic_date_to_j2000(point.longitude_deg, point.latitude_deg, jd_tt)
            .map_err(map_precession_error)?;
        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(j2000.longitude_deg),
            Latitude::from_degrees(j2000.latitude_deg),
            Some(distance_au),
        ))
    }

    /// Motion of a derived lunar point: central differences over ±0.5 and
    /// ±0.25 day, Richardson-extrapolated to cancel their leading truncation
    /// term, `(4·D(0.25) − D(0.5)) / 3`.
    ///
    /// The osculating points oscillate within a fortnight, so a plain ±0.5-day
    /// difference reads their speed low wherever it peaks: the true node by
    /// about 3″/day in its short direct spells, enough to hide them (#108).
    /// A shorter plain difference is no better, because it amplifies the small
    /// steps in the packaged Moon's velocity between fitted segments. Measured
    /// against the derivative of the node and apogee formed from the exact
    /// DE440 Moon, 2017–2027 every 0.05 day: true node max 0.71″/day (rms
    /// 0.07″) against 3.9″/day for the ±0.5-day difference; true apogee max
    /// 13″/day (rms 2.3″) against 161″/day.
    ///
    /// A probe that falls outside the packaged window degrades to `None`
    /// channels rather than failing the position. The outer probes sit at
    /// ±0.5 day as before, so the same instants degrade.
    fn derived_point_motion(
        &self,
        instant: Instant,
        eval: &dyn Fn(Instant) -> Result<EclipticCoordinates, EphemerisError>,
    ) -> Result<Motion, EphemerisError> {
        const OUTER_HALF_SPAN_DAYS: f64 = 0.5;
        const INNER_HALF_SPAN_DAYS: f64 = 0.25;
        let probe = |days: f64| -> Result<Option<EclipticCoordinates>, EphemerisError> {
            let shifted = Instant::new(
                JulianDay::from_days(instant.julian_day.days() + days),
                instant.scale,
            );
            match eval(shifted) {
                Ok(e) => Ok(Some(e)),
                Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => Ok(None),
                Err(e) => Err(e),
            }
        };
        let (Some(outer_before), Some(inner_before), Some(inner_after), Some(outer_after)) = (
            probe(-OUTER_HALF_SPAN_DAYS)?,
            probe(-INNER_HALF_SPAN_DAYS)?,
            probe(INNER_HALF_SPAN_DAYS)?,
            probe(OUTER_HALF_SPAN_DAYS)?,
        ) else {
            return Ok(Motion::new(None, None, None));
        };

        // One rate per span, then the extrapolation.
        let extrapolate = |outer: f64, inner: f64| (4.0 * inner - outer) / 3.0;
        let rate = |before: f64, after: f64, half_span: f64| (after - before) / (2.0 * half_span);
        let lon_rate = |before: &EclipticCoordinates, after: &EclipticCoordinates, half_span| {
            let mut dlon = after.longitude.degrees() - before.longitude.degrees();
            while dlon > 180.0 {
                dlon -= 360.0;
            }
            while dlon < -180.0 {
                dlon += 360.0;
            }
            dlon / (2.0 * half_span)
        };
        let lat_rate = |before: &EclipticCoordinates, after: &EclipticCoordinates, half_span| {
            rate(
                before.latitude.degrees(),
                after.latitude.degrees(),
                half_span,
            )
        };
        let dist_rate =
            |before: &EclipticCoordinates, after: &EclipticCoordinates, half_span| match (
                before.distance_au,
                after.distance_au,
            ) {
                (Some(b), Some(a)) => Some(rate(b, a, half_span)),
                _ => None,
            };

        let dlon_per_day = extrapolate(
            lon_rate(&outer_before, &outer_after, OUTER_HALF_SPAN_DAYS),
            lon_rate(&inner_before, &inner_after, INNER_HALF_SPAN_DAYS),
        );
        let dlat_per_day = extrapolate(
            lat_rate(&outer_before, &outer_after, OUTER_HALF_SPAN_DAYS),
            lat_rate(&inner_before, &inner_after, INNER_HALF_SPAN_DAYS),
        );
        let ddist_per_day = match (
            dist_rate(&outer_before, &outer_after, OUTER_HALF_SPAN_DAYS),
            dist_rate(&inner_before, &inner_after, INNER_HALF_SPAN_DAYS),
        ) {
            (Some(outer), Some(inner)) => Some(extrapolate(outer, inner)),
            _ => None,
        };
        Ok(Motion::new(
            Some(dlon_per_day),
            Some(dlat_per_day),
            ddist_per_day,
        ))
    }
}

/// Rotates a J2000 mean-ecliptic vector into the mean ecliptic of date at
/// `jd_tt`, preserving its magnitude. The same map applies to position and
/// velocity vectors alike; the ELP backend's osculating node shares it.
fn rotate_j2000_to_mean_of_date(v: [f64; 3], jd_tt: f64) -> Result<[f64; 3], EphemerisError> {
    precess_ecliptic_vector_j2000_to_date(v, jd_tt).map_err(map_precession_error)
}

fn map_precession_error(e: ApparentPlaceError) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("precession failed for derived lunar point: {e}"),
    )
}

impl EphemerisBackend for PackagedDataBackend {
    fn metadata(&self) -> BackendMetadata {
        let artifact = self.artifact();
        let bodies = artifact
            .bodies
            .iter()
            .map(|series| series.body.clone())
            .filter(|body| !crate::is_carried_but_unserved(body))
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
                    packaged_body_coverage_summary_details()
                        .validated_summary_line()
                        .unwrap_or_else(|error| {
                            format!("Packaged body set: unavailable ({error})")
                        }),
                    packaged_request_policy_summary().to_string(),
                    packaged_frame_treatment_summary().to_string(),
                    packaged_artifact_storage_summary().to_string(),
                    packaged_artifact_access_summary().to_string(),
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
                claims.extend(crate::true_node_body_claims());
                claims.extend(crate::mean_lunar_point_body_claims());
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
        matches!(
            body,
            CelestialBody::TrueApogee
                | CelestialBody::TruePerigee
                | CelestialBody::TrueNode
                | CelestialBody::MeanNode
                | CelestialBody::MeanApogee
                | CelestialBody::MeanPerigee
        ) || (!crate::is_carried_but_unserved(&body)
            && self
                .artifact
                .bodies
                .iter()
                .any(|series| series.body == body))
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
            let body = req.body.clone();
            return self.derived_point_position(req, &|i| self.osculating_apsis_ecliptic(&body, i));
        }
        if req.body == CelestialBody::TrueNode {
            return self.derived_point_position(req, &|i| self.osculating_node_ecliptic(i));
        }
        if matches!(
            req.body,
            CelestialBody::MeanNode | CelestialBody::MeanApogee | CelestialBody::MeanPerigee
        ) {
            let body = req.body.clone();
            return self.derived_point_position(req, &|i| self.mean_lunar_point_ecliptic(&body, i));
        }

        if crate::is_carried_but_unserved(&req.body) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                format!(
                    "packaged data carries {} but does not serve it: its segments are fitted \
                     to rows too sparse to interpolate. Serve it from pleiades_jpl::SpkBackend \
                     with a JPL kernel",
                    req.body
                ),
            ));
        }

        let lookup_instant = normalize_lookup_instant(req.instant);
        let ecliptic = self
            .artifact
            .lookup_ecliptic(&req.body, lookup_instant)
            .map_err(map_artifact_error)?;
        let equatorial = ecliptic.to_j2000_equatorial();
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

#[cfg(test)]
mod coupling_fixture_tests {
    use super::*;

    /// Golden fixture pinning `PackagedDataBackend::metadata()`'s
    /// `provenance.data_sources` to its pre-Slice-C rendered values.
    ///
    /// This test exists to prove byte-identity when the five report-prose
    /// renderers backing `data_sources` are replaced with an inline rebuild
    /// from retained structured accessors (Slice C, Task 2). It must keep
    /// passing after that rebuild — if it fails, the rebuild drifted; fix
    /// the rebuild, never this fixture.
    #[test]
    fn backend_metadata_data_sources_is_stable() {
        use pleiades_backend::EphemerisBackend;
        let metadata = PackagedDataBackend::default().metadata();
        let expected: &[&str] = &[
            "Packaged body set: 11 bundled bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros)",
            "Packaged request policy: geocentric-only; frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false; lookup epoch policy=TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction",
            "checked-in compressed artifact stores J2000 ecliptic coordinates directly; equatorial coordinates are reconstructed from the stored channels and J2000 mean-obliquity transform",
            "Quantized linear segments stored in pleiades-compression artifact format; body-indexed segment tables support random access by body and lookup time across the advertised range; ecliptic and equatorial coordinates are reconstructed at runtime from stored channels; apparent, topocentric, and sidereal outputs remain unsupported; motion/speed is derived from fitted segment derivatives",
            "packaged artifact access: checked-in fixture only; explicit artifact-path loading disabled",
        ];
        assert_eq!(
            metadata.provenance.data_sources, expected,
            "backend metadata data_sources drifted:\n{:#?}",
            metadata.provenance.data_sources
        );
    }
}
