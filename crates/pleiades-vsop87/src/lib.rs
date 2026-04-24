//! Formula-based planetary backend boundary built around low-precision orbital
//! elements and geocentric coordinate transforms.
//!
//! This crate now provides a working pure-Rust algorithmic backend for the Sun
//! and major planets. The implementation uses compact Keplerian orbital elements,
//! a geocentric reduction step, and central-difference motion estimates so the
//! workspace has an end-to-end tropical chart path before the full VSOP87 series
//! data arrives.

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

const PACKAGE_NAME: &str = "pleiades-vsop87";
const J2000: f64 = 2_451_545.0;

/// A pure-Rust planetary backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct Vsop87Backend;

impl Vsop87Backend {
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

    fn supported_bodies() -> &'static [CelestialBody] {
        &[
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
            CelestialBody::Pluto,
        ]
    }

    fn earth_elements(days: f64) -> OrbitalElements {
        OrbitalElements::new(
            0.0,
            0.0,
            282.9404 + 4.70935e-5 * days,
            1.000000,
            0.016709 - 1.151e-9 * days,
            356.0470 + 0.985_600_258_5 * days,
        )
    }

    fn orbital_elements(body: CelestialBody, days: f64) -> Option<OrbitalElements> {
        match body {
            CelestialBody::Mercury => Some(OrbitalElements::new(
                48.3313 + 3.24587e-5 * days,
                7.0047 + 5.00e-8 * days,
                29.1241 + 1.01444e-5 * days,
                0.387098,
                0.205635 + 5.59e-10 * days,
                168.6562 + 4.092_334_436_8 * days,
            )),
            CelestialBody::Venus => Some(OrbitalElements::new(
                76.6799 + 2.46590e-5 * days,
                3.3946 + 2.75e-8 * days,
                54.8910 + 1.38374e-5 * days,
                0.72333,
                0.006773 - 1.302e-9 * days,
                48.0052 + 1.602_130_224_4 * days,
            )),
            CelestialBody::Mars => Some(OrbitalElements::new(
                49.5574 + 2.11081e-5 * days,
                1.8497 - 1.78e-8 * days,
                286.5016 + 2.92961e-5 * days,
                1.523688,
                0.093405 + 2.516e-9 * days,
                18.6021 + 0.524_020_776_6 * days,
            )),
            CelestialBody::Jupiter => Some(OrbitalElements::new(
                100.4542 + 2.76854e-5 * days,
                1.3030 - 1.557e-7 * days,
                273.8777 + 1.64505e-5 * days,
                5.20256,
                0.048498 + 4.469e-9 * days,
                19.8950 + 0.083_085_300_1 * days,
            )),
            CelestialBody::Saturn => Some(OrbitalElements::new(
                113.6634 + 2.38980e-5 * days,
                2.4886 - 1.081e-7 * days,
                339.3939 + 2.97661e-5 * days,
                9.55475,
                0.055546 - 9.499e-9 * days,
                316.9670 + 0.033_444_228_2 * days,
            )),
            CelestialBody::Uranus => Some(OrbitalElements::new(
                74.0005 + 1.3978e-5 * days,
                0.7733 + 1.9e-8 * days,
                96.6612 + 3.0565e-5 * days,
                19.18171 - 1.55e-8 * days,
                0.047318 + 7.45e-9 * days,
                142.5905 + 0.011_725_806 * days,
            )),
            CelestialBody::Neptune => Some(OrbitalElements::new(
                131.7806 + 3.0173e-5 * days,
                1.7700 - 2.55e-7 * days,
                272.8461 - 6.027e-6 * days,
                30.05826 + 3.313e-8 * days,
                0.008606 + 2.15e-9 * days,
                260.2471 + 0.005_995_147 * days,
            )),
            CelestialBody::Pluto => Some(OrbitalElements::new(
                110.30347,
                17.14175,
                113.76329,
                39.481_686_77,
                0.248_807_66,
                14.53 + 0.003_96 * days,
            )),
            _ => None,
        }
    }

    fn heliocentric_coordinates(elements: OrbitalElements) -> HeliocentricCoordinates {
        let mean_anomaly = normalize_degrees(elements.mean_anomaly);
        let eccentric_anomaly = solve_kepler(mean_anomaly, elements.eccentricity);
        let true_anomaly = true_anomaly_from_eccentric(eccentric_anomaly, elements.eccentricity);
        let eccentric_anomaly_rad = eccentric_anomaly.to_radians();
        let radius =
            elements.semi_major_axis * (1.0 - elements.eccentricity * eccentric_anomaly_rad.cos());

        let node = elements.ascending_node.to_radians();
        let inclination = elements.inclination.to_radians();
        let perihelion = elements.argument_of_perihelion.to_radians();
        let lon = (true_anomaly + perihelion).to_radians();

        let xh = radius * (node.cos() * lon.cos() - node.sin() * lon.sin() * inclination.cos());
        let yh = radius * (node.sin() * lon.cos() + node.cos() * lon.sin() * inclination.cos());
        let zh = radius * (lon.sin() * inclination.sin());

        HeliocentricCoordinates { xh, yh, zh }
    }

    fn geocentric_coordinates(body: CelestialBody, days: f64) -> Option<HeliocentricCoordinates> {
        let earth = Self::heliocentric_coordinates(Self::earth_elements(days));
        if body == CelestialBody::Sun {
            return Some(HeliocentricCoordinates {
                xh: -earth.xh,
                yh: -earth.yh,
                zh: -earth.zh,
            });
        }

        let target = Self::heliocentric_coordinates(Self::orbital_elements(body, days)?);
        Some(HeliocentricCoordinates {
            xh: target.xh - earth.xh,
            yh: target.yh - earth.yh,
            zh: target.zh - earth.zh,
        })
    }

    fn distance_au(coords: HeliocentricCoordinates) -> f64 {
        coords.xh.hypot(coords.yh.hypot(coords.zh))
    }

    fn to_ecliptic(coords: HeliocentricCoordinates) -> EclipticCoordinates {
        let longitude = Longitude::from_degrees(coords.yh.atan2(coords.xh).to_degrees());
        let latitude = Latitude::from_degrees(
            coords
                .zh
                .atan2((coords.xh * coords.xh + coords.yh * coords.yh).sqrt())
                .to_degrees(),
        );
        EclipticCoordinates::new(longitude, latitude, Some(Self::distance_au(coords)))
    }

    fn to_equatorial(coords: HeliocentricCoordinates, instant: Instant) -> EquatorialCoordinates {
        let obliquity = Self::mean_obliquity_degrees(instant).to_radians();
        let xeq = coords.xh;
        let yeq = coords.yh * obliquity.cos() - coords.zh * obliquity.sin();
        let zeq = coords.yh * obliquity.sin() + coords.zh * obliquity.cos();

        EquatorialCoordinates::new(
            Angle::from_degrees(yeq.atan2(xeq).to_degrees()).normalized_0_360(),
            Latitude::from_degrees(zeq.atan2((xeq * xeq + yeq * yeq).sqrt()).to_degrees()),
            Some(Self::distance_au(coords)),
        )
    }

    fn motion(body: CelestialBody, days: f64) -> Option<Motion> {
        // A symmetric one-day span gives stable chart-facing daily rates while
        // keeping the preliminary element model simple and deterministic. These
        // are finite-difference estimates of the same mean geocentric model, not
        // apparent velocities from a full VSOP87/light-time reduction.
        const HALF_SPAN_DAYS: f64 = 0.5;
        const FULL_SPAN_DAYS: f64 = HALF_SPAN_DAYS * 2.0;

        let before = Self::to_ecliptic(Self::geocentric_coordinates(
            body.clone(),
            days - HALF_SPAN_DAYS,
        )?);
        let after = Self::to_ecliptic(Self::geocentric_coordinates(body, days + HALF_SPAN_DAYS)?);

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
}

impl EphemerisBackend for Vsop87Backend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: "Low-precision pure-Rust planetary backend based on compact Keplerian orbital elements and geocentric reduction.".to_string(),
                data_sources: vec![
                    "Paul Schlyter-style mean orbital elements for the Sun and planets".to_string(),
                    "Meeus-style coordinate transforms for geocentric reduction".to_string(),
                ],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: Self::supported_bodies().to_vec(),
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
        Self::supported_bodies().contains(&body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "the VSOP87 MVP backend currently exposes tropical coordinates only",
            ));
        }

        if req.instant.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "the VSOP87 MVP backend expects terrestrial time (TT)",
            ));
        }

        let days = Self::days_since_j2000(req.instant);
        let geocentric = Self::geocentric_coordinates(req.body.clone(), days).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "requested body is not implemented in the VSOP87 MVP backend",
            )
        })?;

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        result.ecliptic = Some(Self::to_ecliptic(geocentric));
        result.equatorial = Some(Self::to_equatorial(geocentric, req.instant));
        result.motion = Self::motion(req.body.clone(), days);
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug)]
struct OrbitalElements {
    ascending_node: f64,
    inclination: f64,
    argument_of_perihelion: f64,
    semi_major_axis: f64,
    eccentricity: f64,
    mean_anomaly: f64,
}

impl OrbitalElements {
    const fn new(
        ascending_node: f64,
        inclination: f64,
        argument_of_perihelion: f64,
        semi_major_axis: f64,
        eccentricity: f64,
        mean_anomaly: f64,
    ) -> Self {
        Self {
            ascending_node,
            inclination,
            argument_of_perihelion,
            semi_major_axis,
            eccentricity,
            mean_anomaly,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HeliocentricCoordinates {
    xh: f64,
    yh: f64,
    zh: f64,
}

fn normalize_degrees(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

fn signed_longitude_delta_degrees(start: f64, end: f64) -> f64 {
    (end - start + 180.0).rem_euclid(360.0) - 180.0
}

fn solve_kepler(mean_anomaly_degrees: f64, eccentricity: f64) -> f64 {
    let m = mean_anomaly_degrees.to_radians();
    let mut e = m + eccentricity * m.sin() * (1.0 + eccentricity * m.cos());
    for _ in 0..10 {
        let delta = (e - eccentricity * e.sin() - m) / (1.0 - eccentricity * e.cos());
        e -= delta;
        if delta.abs() < 1e-12 {
            break;
        }
    }
    e.to_degrees()
}

fn true_anomaly_from_eccentric(eccentric_anomaly_degrees: f64, eccentricity: f64) -> f64 {
    let e = eccentric_anomaly_degrees.to_radians();
    let numerator = (1.0 + eccentricity).sqrt() * (e / 2.0).sin();
    let denominator = (1.0 - eccentricity).sqrt() * (e / 2.0).cos();
    (2.0 * numerator.atan2(denominator))
        .to_degrees()
        .rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_is_stable() {
        assert_eq!(PACKAGE_NAME, "pleiades-vsop87");
    }

    #[test]
    fn backend_reports_major_planets() {
        let backend = Vsop87Backend::new();
        assert!(backend.supports_body(CelestialBody::Sun));
        assert!(backend.supports_body(CelestialBody::Mars));
        assert!(!backend.supports_body(CelestialBody::Moon));
    }

    #[test]
    fn j2000_sun_position_is_finite() {
        let backend = Vsop87Backend::new();
        let request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );
        let result = backend.position(&request).expect("sun query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert_eq!(result.quality, QualityAnnotation::Approximate);
    }

    #[test]
    fn finite_difference_motion_is_reported_for_supported_bodies() {
        let backend = Vsop87Backend::new();
        let request = EphemerisRequest::new(
            CelestialBody::Mars,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );
        let result = backend.position(&request).expect("Mars query should work");
        let motion = result.motion.expect("motion should be populated");

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
    }

    #[test]
    fn signed_longitude_delta_wraps_across_zero_aries() {
        assert_eq!(signed_longitude_delta_degrees(359.5, 0.5), 1.0);
        assert_eq!(signed_longitude_delta_degrees(0.5, 359.5), -1.0);
    }
}
