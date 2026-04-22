//! Lunar backend boundary based on a compact pure-Rust analytical model.
//!
//! The full ELP series data is still planned, but this crate now provides a
//! usable Moon backend for the chart MVP by combining a low-precision lunar
//! orbit model with geocentric coordinate transforms.

#![forbid(unsafe_code)]

use pleiades_backend::{
    AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
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
}

impl EphemerisBackend for ElpBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: "Low-precision pure-Rust lunar backend using a compact analytical orbit model and geocentric reduction.".to_string(),
                data_sources: vec![
                    "Meeus-style truncated lunar orbit formulas".to_string(),
                    "Public ELP 2000/82 documentation for future expansion".to_string(),
                ],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![CelestialBody::Moon],
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
        body == CelestialBody::Moon
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if req.body != CelestialBody::Moon {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "the ELP MVP backend only serves the Moon",
            ));
        }

        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "the ELP MVP backend currently exposes tropical coordinates only",
            ));
        }

        if req.instant.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "the ELP MVP backend expects terrestrial time (TT)",
            ));
        }

        let coords = Self::geocentric_coordinates(Self::days_since_j2000(req.instant));
        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        result.ecliptic = Some(Self::to_ecliptic(coords));
        result.equatorial = Some(Self::to_equatorial(coords, req.instant));
        result.motion = Some(Motion::new(None, None, None));
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
    fn backend_only_supports_the_moon() {
        let backend = ElpBackend::new();
        assert!(backend.supports_body(CelestialBody::Moon));
        assert!(!backend.supports_body(CelestialBody::Sun));
    }

    #[test]
    fn j2000_moon_position_is_finite() {
        let backend = ElpBackend::new();
        let request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );
        let result = backend.position(&request).expect("moon query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert_eq!(result.quality, QualityAnnotation::Approximate);
    }
}
