//! `ElpBackend` struct and `EphemerisBackend` implementation.

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance, BodyClaim,
    ClaimEvidence, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, EquatorialCoordinates, Instant, Latitude,
    Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};

use crate::specification::{SUPPORTED_LUNAR_FRAMES, SUPPORTED_LUNAR_TIME_SCALES};
use crate::{
    lunar_theory_source_family_summary, lunar_theory_specification, lunar_theory_supported_bodies,
    series, PACKAGE_NAME,
};

/// A pure-Rust lunar backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct ElpBackend;

/// Returns the set of body claims for the ELP lunar backend.
///
/// Lunar bodies (Moon, Mean Node, True Node, Mean Apogee, Mean Perigee) are claimed as Constrained
/// with Moderate accuracy and AlgorithmicModel evidence. True Apogee and True Perigee are explicitly
/// listed as Unsupported by this backend. Note: the osculating true apogee/perigee (True Lilith)
/// are served release-grade by `PackagedDataBackend` ahead of this backend in the composite routing
/// chain, so the ELP-local `Unsupported` claim is no longer a global gap.
pub fn elp_body_claims() -> Vec<BodyClaim> {
    let mut claims: Vec<BodyClaim> = lunar_theory_supported_bodies()
        .iter()
        .cloned()
        .map(|body| {
            BodyClaim::constrained(
                body,
                AccuracyClass::Moderate,
                ClaimEvidence::AlgorithmicModel,
            )
        })
        .collect();
    claims.push(BodyClaim::unsupported(CelestialBody::TrueApogee));
    claims.push(BodyClaim::unsupported(CelestialBody::TruePerigee));
    claims
}

impl ElpBackend {
    /// Creates a new backend instance.
    pub const fn new() -> Self {
        Self
    }

    fn days_since_j2000(instant: Instant) -> f64 {
        instant.julian_day.days() - crate::J2000
    }

    pub(crate) fn moon_ecliptic_coordinates(days: f64) -> EclipticCoordinates {
        let jd_tt = crate::J2000 + days;
        let (longitude, latitude, distance_au) = crate::data::moonposition::position(jd_tt);
        // The Meeus Ch.47 series is referred to the mean equinox/ecliptic OF DATE.
        // Precess it back to J2000 so the backend emits a J2000 boundary frame,
        // consistent with every other first-party backend; the apparent pipeline
        // re-applies the forward J2000->date precession (no double-precession).
        let precessed = pleiades_apparent::precess_ecliptic_date_to_j2000(
            longitude.degrees(),
            latitude.degrees(),
            jd_tt,
        )
        .expect("ELP lunar lon/lat precess cleanly to J2000");
        EclipticCoordinates::new(
            Longitude::from_degrees(precessed.longitude_deg),
            Latitude::from_degrees(precessed.latitude_deg),
            Some(distance_au),
        )
    }

    fn moon_ecliptic_of_date(days: f64) -> EclipticCoordinates {
        let (longitude, latitude, distance_au) =
            crate::data::moonposition::position(crate::J2000 + days);
        EclipticCoordinates::new(longitude, latitude, Some(distance_au))
    }

    fn mean_node_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        series::normalize_degrees(
            125.044_547_9
                + (-1_934.136_289_1 + (0.002_075_4 + (1.0 / 476_441.0 - t / 60_616_000.0) * t) * t)
                    * t,
        )
    }

    fn mean_perigee_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        series::normalize_degrees(
            83.353_246_5
                + (4_069.013_728_7 + (-0.010_32 + (-1.0 / 80_053.0 + t / 18_999_000.0) * t) * t)
                    * t,
        )
    }

    fn mean_apogee_longitude(days: f64) -> f64 {
        series::normalize_degrees(Self::mean_perigee_longitude(days) + 180.0)
    }

    fn true_node_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        let mean_node = Self::mean_node_longitude(days).to_radians();
        let mean_elongation = series::normalize_degrees(
            297.850_192_1
                + (445_267.111_403_4
                    + (-0.001_881_9 + (1.0 / 545_868.0 - t / 113_065_000.0) * t) * t)
                    * t,
        )
        .to_radians();
        let solar_anomaly = series::normalize_degrees(
            357.529_109_2 + (35_999.050_290_9 + (-0.000_153_6 + t / 24_490_000.0) * t) * t,
        )
        .to_radians();
        let lunar_anomaly = series::normalize_degrees(
            134.963_396_4
                + (477_198.867_505_5 + (0.008_741_4 + (1.0 / 69_699.9 + t / 14_712_000.0) * t) * t)
                    * t,
        )
        .to_radians();
        let latitude_argument = series::normalize_degrees(
            93.272_095_0
                + (483_202.017_523_3
                    + (-0.003_653_9 + (-1.0 / 3_526_000.0 + t / 863_310_000.0) * t) * t)
                    * t,
        )
        .to_radians();

        series::normalize_degrees(
            mean_node.to_degrees()
                + (-1.4979 * (2.0 * (mean_elongation - latitude_argument)).sin()
                    - 0.15 * solar_anomaly.sin()
                    - 0.1226 * (2.0 * mean_elongation).sin()
                    + 0.1176 * (2.0 * latitude_argument).sin()
                    - 0.0801 * (2.0 * (lunar_anomaly - latitude_argument)).sin()),
        )
    }

    fn ecliptic_for_body(body: CelestialBody, days: f64) -> Option<EclipticCoordinates> {
        match body {
            CelestialBody::Moon => Some(Self::moon_ecliptic_coordinates(days)),
            CelestialBody::MeanNode => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::mean_node_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            CelestialBody::TrueNode => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::true_node_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            CelestialBody::MeanApogee => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::mean_apogee_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            CelestialBody::MeanPerigee => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::mean_perigee_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            _ => None,
        }
    }

    fn motion(body: CelestialBody, days: f64) -> Option<Motion> {
        // Match the planetary backend's chart-facing convention: these are
        // symmetric finite-difference rates for the same mean geometric model,
        // not apparent velocities from a full lunar theory.
        const HALF_SPAN_DAYS: f64 = 0.5;
        const FULL_SPAN_DAYS: f64 = HALF_SPAN_DAYS * 2.0;

        let before = Self::ecliptic_for_body(body.clone(), days - HALF_SPAN_DAYS)?;
        let after = Self::ecliptic_for_body(body, days + HALF_SPAN_DAYS)?;

        let longitude_speed = series::signed_longitude_delta_degrees(
            before.longitude.degrees(),
            after.longitude.degrees(),
        ) / FULL_SPAN_DAYS;
        let latitude_speed =
            (after.latitude.degrees() - before.latitude.degrees()) / FULL_SPAN_DAYS;
        let distance_speed = match (before.distance_au, after.distance_au) {
            (Some(before), Some(after)) => Some((after - before) / FULL_SPAN_DAYS),
            _ => None,
        };

        Some(Motion::new(
            Some(longitude_speed),
            Some(latitude_speed),
            distance_speed,
        ))
    }

    fn ecliptic_point_to_equatorial(
        longitude: Longitude,
        latitude: Latitude,
        instant: Instant,
        distance_au: Option<f64>,
    ) -> EquatorialCoordinates {
        EclipticCoordinates::new(longitude, latitude, distance_au)
            .to_equatorial(instant.mean_obliquity())
    }
}

impl EphemerisBackend for ElpBackend {
    fn metadata(&self) -> BackendMetadata {
        let theory = lunar_theory_specification();
        let source = theory.source_selection();
        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: format!(
                    "{} [{}; family: {}] {} The backend exposes the Moon plus mean/true node and mean apogee/perigee channels as an explicit lunar-theory selection, while explicitly leaving true apogee/perigee unsupported for now; {}",
                    theory.model_name,
                    source.identifier,
                    source.family,
                    source.citation,
                    source.license_note,
                ),
                data_sources: vec![
                    "Meeus-style truncated lunar orbit formulas implemented in pure Rust; see docs/lunar-theory-policy.md for the current baseline scope".to_string(),
                    {
                        let family = lunar_theory_source_family_summary();
                        match family.validate() {
                            Ok(()) => family.summary_line(),
                            Err(error) => format!("lunar source family: unavailable ({error})"),
                        }
                    },
                    source.identifier.to_string(),
                    source.citation.to_string(),
                    source.material.to_string(),
                    source.redistribution_note.to_string(),
                    theory.truncation_note.to_string(),
                    theory.unit_note.to_string(),
                    source.license_note.to_string(),
                    theory.date_range_note.to_string(),
                    theory.frame_note.to_string(),
                ],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_claims: elp_body_claims(),
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
        lunar_theory_supported_bodies().contains(&body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !self.supports_body(req.body.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "the ELP backend currently serves the Moon, lunar nodes, and mean lunar apogee/perigee only",
            ));
        }

        validate_zodiac_policy(req, "the ELP backend", &[ZodiacMode::Tropical])?;

        validate_request_policy(
            req,
            "the ELP backend",
            SUPPORTED_LUNAR_TIME_SCALES,
            SUPPORTED_LUNAR_FRAMES,
            true,
            false,
        )?;

        validate_observer_policy(req, "the ELP backend", false)?;

        let days = Self::days_since_j2000(req.instant);
        let body = req.body.clone();
        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        match body {
            CelestialBody::Moon => {
                let coords = Self::moon_ecliptic_coordinates(days); // J2000 boundary
                result.ecliptic = Some(coords);
                let of_date = Self::moon_ecliptic_of_date(days); // for mean-obliquity equatorial
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    of_date.longitude,
                    of_date.latitude,
                    req.instant,
                    of_date.distance_au,
                ));
            }
            CelestialBody::MeanNode => {
                let longitude = Longitude::from_degrees(Self::mean_node_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            CelestialBody::TrueNode => {
                let longitude = Longitude::from_degrees(Self::true_node_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            CelestialBody::MeanApogee => {
                let longitude = Longitude::from_degrees(Self::mean_apogee_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            CelestialBody::MeanPerigee => {
                let longitude = Longitude::from_degrees(Self::mean_perigee_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            _ => unreachable!("body support should be validated before position queries"),
        }
        result.motion = Self::motion(body, days);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::ElpBackend;
    use pleiades_backend::EphemerisBackend; // brings `.metadata()` into scope

    #[test]
    fn backend_metadata_source_family_line_is_stable() {
        let metadata = ElpBackend.metadata();
        let expected = "lunar source family: Meeus-style truncated analytical baseline [selected source=meeus-style-truncated-lunar-baseline; selected model=Compact Meeus-style truncated lunar baseline; selected key=source identifier=meeus-style-truncated-lunar-baseline; selected family key=source family=Meeus-style truncated analytical baseline; aliases=1]";
        assert!(
            metadata
                .provenance
                .data_sources
                .iter()
                .any(|s| s == expected),
            "source-family provenance line drifted:\n{:#?}",
            metadata.provenance.data_sources
        );
    }
}
