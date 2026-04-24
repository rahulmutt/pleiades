//! Lunar backend boundary based on a compact pure-Rust analytical model.
//!
//! The full ELP series data is still planned, but this crate now provides a
//! usable Moon-and-lunar-point backend for the chart MVP by combining a
//! low-precision lunar orbit model with geocentric coordinate transforms,
//! Meeus-style mean node/perigee/apogee formulae, and finite-difference
//! mean-motion estimates.

#![forbid(unsafe_code)]

use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    Angle, CelestialBody, CoordinateFrame, EclipticCoordinates, EquatorialCoordinates, Instant,
    Latitude, Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};

const PACKAGE_NAME: &str = "pleiades-elp";
const J2000: f64 = 2_451_545.0;
const EARTH_RADIUS_KM: f64 = 6_378.14;
const AU_IN_KM: f64 = 149_597_870.700;

/// A pure-Rust lunar backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct ElpBackend;

impl ElpBackend {
    /// Creates a new backend instance.
    pub const fn new() -> Self {
        Self
    }

    fn days_since_j2000(instant: Instant) -> f64 {
        instant.julian_day.days() - J2000
    }

    fn julian_centuries(instant: Instant) -> f64 {
        Self::days_since_j2000(instant) / 36_525.0
    }

    fn mean_obliquity_degrees(instant: Instant) -> f64 {
        let t = Self::julian_centuries(instant);
        23.439_291_111_111_11
            - 0.013_004_166_666_666_667 * t
            - 0.000_000_163_888_888_888_888_88 * t * t
            + 0.000_000_503_611_111_111_111_1 * t * t * t
    }

    fn geocentric_coordinates(days: f64) -> HeliocentricLikeCoordinates {
        let node = (125.1228 - 0.052_953_808_3 * days).to_radians();
        let inclination = 5.1454_f64.to_radians();
        let periapsis = (318.0634 + 0.164_357_322_3 * days).to_radians();
        let semi_major_axis = 60.2666;
        let eccentricity = 0.054900;
        let mean_anomaly = normalize_degrees(115.3654 + 13.064_992_950_9 * days).to_radians();

        let eccentric_anomaly = solve_kepler(mean_anomaly, eccentricity);
        let xv = semi_major_axis * (eccentric_anomaly.cos() - eccentricity);
        let yv =
            semi_major_axis * (1.0 - eccentricity * eccentricity).sqrt() * eccentric_anomaly.sin();
        let true_anomaly = yv.atan2(xv);
        let radius_re = (xv * xv + yv * yv).sqrt();

        let longitude = true_anomaly + periapsis;
        let x = radius_re
            * (node.cos() * longitude.cos() - node.sin() * longitude.sin() * inclination.cos());
        let y = radius_re
            * (node.sin() * longitude.cos() + node.cos() * longitude.sin() * inclination.cos());
        let z = radius_re * (longitude.sin() * inclination.sin());

        HeliocentricLikeCoordinates {
            x,
            y,
            z,
            distance_re: radius_re,
        }
    }

    fn to_ecliptic(coords: HeliocentricLikeCoordinates) -> EclipticCoordinates {
        let longitude = Longitude::from_degrees(coords.y.atan2(coords.x).to_degrees());
        let latitude = Latitude::from_degrees(
            coords
                .z
                .atan2((coords.x * coords.x + coords.y * coords.y).sqrt())
                .to_degrees(),
        );
        let distance_km = coords.distance_re * EARTH_RADIUS_KM;
        EclipticCoordinates::new(longitude, latitude, Some(distance_km / AU_IN_KM))
    }

    fn to_equatorial(
        coords: HeliocentricLikeCoordinates,
        instant: Instant,
    ) -> EquatorialCoordinates {
        let obliquity = Self::mean_obliquity_degrees(instant).to_radians();
        let xeq = coords.x;
        let yeq = coords.y * obliquity.cos() - coords.z * obliquity.sin();
        let zeq = coords.y * obliquity.sin() + coords.z * obliquity.cos();
        EquatorialCoordinates::new(
            Angle::from_degrees(yeq.atan2(xeq).to_degrees()).normalized_0_360(),
            Latitude::from_degrees(zeq.atan2((xeq * xeq + yeq * yeq).sqrt()).to_degrees()),
            Some(coords.distance_re * EARTH_RADIUS_KM / AU_IN_KM),
        )
    }

    fn mean_node_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        normalize_degrees(
            125.044_547_9
                + (-1_934.136_289_1 + (0.002_075_4 + (1.0 / 476_441.0 - t / 60_616_000.0) * t) * t)
                    * t,
        )
    }

    fn mean_perigee_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        normalize_degrees(
            83.353_246_5
                + (4_069.013_728_7 + (-0.010_32 + (-1.0 / 80_053.0 + t / 18_999_000.0) * t) * t)
                    * t,
        )
    }

    fn mean_apogee_longitude(days: f64) -> f64 {
        normalize_degrees(Self::mean_perigee_longitude(days) + 180.0)
    }

    fn true_node_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        let mean_node = Self::mean_node_longitude(days).to_radians();
        let mean_elongation = normalize_degrees(
            297.850_192_1
                + (445_267.111_403_4
                    + (-0.001_881_9 + (1.0 / 545_868.0 - t / 113_065_000.0) * t) * t)
                    * t,
        )
        .to_radians();
        let solar_anomaly = normalize_degrees(
            357.529_109_2 + (35_999.050_290_9 + (-0.000_153_6 + t / 24_490_000.0) * t) * t,
        )
        .to_radians();
        let lunar_anomaly = normalize_degrees(
            134.963_396_4
                + (477_198.867_505_5 + (0.008_741_4 + (1.0 / 69_699.9 + t / 14_712_000.0) * t) * t)
                    * t,
        )
        .to_radians();
        let latitude_argument = normalize_degrees(
            93.272_095_0
                + (483_202.017_523_3
                    + (-0.003_653_9 + (-1.0 / 3_526_000.0 + t / 863_310_000.0) * t) * t)
                    * t,
        )
        .to_radians();

        normalize_degrees(
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
            CelestialBody::Moon => Some(Self::to_ecliptic(Self::geocentric_coordinates(days))),
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

        let longitude_speed =
            signed_longitude_delta_degrees(before.longitude.degrees(), after.longitude.degrees())
                / FULL_SPAN_DAYS;
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
        let obliquity = Self::mean_obliquity_degrees(instant).to_radians();
        let longitude = longitude.degrees().to_radians();
        let latitude = latitude.degrees().to_radians();
        let x = longitude.cos() * latitude.cos();
        let y =
            longitude.sin() * latitude.cos() * obliquity.cos() - latitude.sin() * obliquity.sin();
        let z =
            longitude.sin() * latitude.cos() * obliquity.sin() + latitude.sin() * obliquity.cos();

        EquatorialCoordinates::new(
            Angle::from_degrees(y.atan2(x).to_degrees()).normalized_0_360(),
            Latitude::from_degrees(z.atan2((x * x + y * y).sqrt()).to_degrees()),
            distance_au,
        )
    }
}

impl EphemerisBackend for ElpBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: "Low-precision pure-Rust lunar backend using a compact analytical orbit model, geocentric reduction, Meeus-style mean lunar point formulae, and finite-difference mean-motion estimates.".to_string(),
                data_sources: vec![
                    "Meeus-style truncated lunar orbit formulas".to_string(),
                    "Meeus-style mean node and mean lunar perigee/apogee formulae".to_string(),
                    "Public ELP 2000/82 documentation for future expansion".to_string(),
                ],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![
                CelestialBody::Moon,
                CelestialBody::MeanNode,
                CelestialBody::TrueNode,
                CelestialBody::MeanApogee,
                CelestialBody::MeanPerigee,
            ],
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
            CelestialBody::Moon
                | CelestialBody::MeanNode
                | CelestialBody::TrueNode
                | CelestialBody::MeanApogee
                | CelestialBody::MeanPerigee
        )
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !self.supports_body(req.body.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "the ELP backend currently serves the Moon, lunar nodes, and mean lunar apogee/perigee only",
            ));
        }

        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "the ELP backend currently exposes tropical coordinates only",
            ));
        }

        if req.instant.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "the ELP backend expects terrestrial time (TT)",
            ));
        }

        if req.apparent != Apparentness::Mean {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "the ELP backend currently returns mean geometric coordinates only; apparent corrections are not implemented",
            ));
        }

        if req.observer.is_some() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidObserver,
                "the ELP backend is geocentric only; topocentric lunar positions are not implemented",
            ));
        }

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
                let coords = Self::geocentric_coordinates(days);
                result.ecliptic = Some(Self::to_ecliptic(coords));
                result.equatorial = Some(Self::to_equatorial(coords, req.instant));
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

#[derive(Clone, Copy, Debug)]
struct HeliocentricLikeCoordinates {
    x: f64,
    y: f64,
    z: f64,
    distance_re: f64,
}

fn normalize_degrees(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

fn signed_longitude_delta_degrees(start: f64, end: f64) -> f64 {
    (end - start + 180.0).rem_euclid(360.0) - 180.0
}

fn solve_kepler(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let mut e = mean_anomaly
        + eccentricity * mean_anomaly.sin() * (1.0 + eccentricity * mean_anomaly.cos());
    for _ in 0..10 {
        let delta = (e - eccentricity * e.sin() - mean_anomaly) / (1.0 - eccentricity * e.cos());
        e -= delta;
        if delta.abs() < 1e-12 {
            break;
        }
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_is_stable() {
        assert_eq!(PACKAGE_NAME, "pleiades-elp");
    }

    #[test]
    fn backend_supports_the_moon_and_lunar_nodes() {
        let backend = ElpBackend::new();
        assert!(backend.supports_body(CelestialBody::Moon));
        assert!(!backend.supports_body(CelestialBody::Sun));
    }

    #[test]
    fn j2000_moon_position_is_finite() {
        let backend = ElpBackend::new();
        let request = mean_request(CelestialBody::Moon);
        let result = backend.position(&request).expect("moon query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let motion = result.motion.expect("motion should be populated");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(motion
            .longitude_deg_per_day
            .expect("longitude speed should exist")
            .is_finite());
        assert!(motion
            .latitude_deg_per_day
            .expect("latitude speed should exist")
            .is_finite());
        assert!(motion
            .distance_au_per_day
            .expect("distance speed should exist")
            .is_finite());
        assert_eq!(result.quality, QualityAnnotation::Approximate);
    }

    #[test]
    fn j2000_mean_and_true_nodes_are_available() {
        let backend = ElpBackend::new();
        let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);

        let mean = backend
            .position(&mean_request_at(CelestialBody::MeanNode, instant))
            .expect("mean node query should work");
        let mean_ecliptic = mean.ecliptic.expect("mean node ecliptic should exist");
        assert!((mean_ecliptic.longitude.degrees() - 125.044_547_9).abs() < 1e-9);
        assert_eq!(mean_ecliptic.latitude.degrees(), 0.0);
        assert!(mean.equatorial.is_some());
        let mean_motion = mean.motion.expect("mean node motion should be populated");
        assert!(mean_motion
            .longitude_deg_per_day
            .expect("mean node longitude speed should exist")
            .is_finite());
        assert_eq!(mean_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(mean_motion.distance_au_per_day, None);

        let true_node = backend
            .position(&mean_request_at(CelestialBody::TrueNode, instant))
            .expect("true node query should work");
        let true_ecliptic = true_node.ecliptic.expect("true node ecliptic should exist");
        assert!((true_ecliptic.longitude.degrees() - 123.926_171_368_400_46).abs() < 1e-9);
        assert_eq!(true_ecliptic.latitude.degrees(), 0.0);
        assert!(true_node.equatorial.is_some());
        let true_motion = true_node
            .motion
            .expect("true node motion should be populated");
        assert!(true_motion
            .longitude_deg_per_day
            .expect("true node longitude speed should exist")
            .is_finite());
        assert_eq!(true_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(true_motion.distance_au_per_day, None);
    }

    #[test]
    fn j2000_mean_apogee_and_perigee_are_available() {
        let backend = ElpBackend::new();
        let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);

        let perigee = backend
            .position(&mean_request_at(CelestialBody::MeanPerigee, instant))
            .expect("mean perigee query should work");
        let perigee_ecliptic = perigee
            .ecliptic
            .expect("mean perigee ecliptic should exist");
        assert!((perigee_ecliptic.longitude.degrees() - 83.353_246_5).abs() < 1e-9);
        assert_eq!(perigee_ecliptic.latitude.degrees(), 0.0);
        assert_eq!(perigee_ecliptic.distance_au, None);
        assert!(perigee.equatorial.is_some());
        let perigee_motion = perigee
            .motion
            .expect("mean perigee motion should be populated");
        assert!(perigee_motion
            .longitude_deg_per_day
            .expect("mean perigee longitude speed should exist")
            .is_finite());
        assert_eq!(perigee_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(perigee_motion.distance_au_per_day, None);

        let apogee = backend
            .position(&mean_request_at(CelestialBody::MeanApogee, instant))
            .expect("mean apogee query should work");
        let apogee_ecliptic = apogee.ecliptic.expect("mean apogee ecliptic should exist");
        assert!((apogee_ecliptic.longitude.degrees() - 263.353_246_5).abs() < 1e-9);
        assert_eq!(apogee_ecliptic.latitude.degrees(), 0.0);
        assert_eq!(apogee_ecliptic.distance_au, None);
        assert!(apogee.equatorial.is_some());
        let apogee_motion = apogee
            .motion
            .expect("mean apogee motion should be populated");
        assert!(apogee_motion
            .longitude_deg_per_day
            .expect("mean apogee longitude speed should exist")
            .is_finite());
        assert_eq!(apogee_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(apogee_motion.distance_au_per_day, None);
    }

    #[test]
    fn backend_supports_lunar_points() {
        let backend = ElpBackend::new();
        assert!(backend.supports_body(CelestialBody::MeanNode));
        assert!(backend.supports_body(CelestialBody::TrueNode));
        assert!(backend.supports_body(CelestialBody::MeanApogee));
        assert!(backend.supports_body(CelestialBody::MeanPerigee));
        assert!(!backend.supports_body(CelestialBody::TrueApogee));
        assert!(!backend.supports_body(CelestialBody::TruePerigee));
        assert!(!backend.supports_body(CelestialBody::Sun));
    }

    #[test]
    fn apparent_requests_are_rejected_explicitly() {
        let backend = ElpBackend::new();
        let request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );

        let error = backend
            .position(&request)
            .expect_err("apparent requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    }

    #[test]
    fn topocentric_requests_are_rejected_explicitly() {
        let backend = ElpBackend::new();
        let mut request = mean_request(CelestialBody::Moon);
        request.observer = Some(pleiades_types::ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(0.0),
            None,
        ));

        let error = backend
            .position(&request)
            .expect_err("topocentric requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    }

    fn mean_request(body: CelestialBody) -> EphemerisRequest {
        mean_request_at(
            body,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        )
    }

    fn mean_request_at(body: CelestialBody, instant: Instant) -> EphemerisRequest {
        let mut request = EphemerisRequest::new(body, instant);
        request.apparent = Apparentness::Mean;
        request
    }
}
