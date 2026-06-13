use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, EquatorialCoordinates, Instant, Latitude,
    Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};

use crate::elements::OrbitalElements;
use crate::profiles::{body_source_profiles, source_kind_for_body, Vsop87BodySourceKind};
use crate::tables;
use crate::transforms::{
    normalize_degrees, pluralize_body_path, signed_longitude_delta_degrees, solve_kepler,
    spherical_lbr_to_cartesian, true_anomaly_from_eccentric, HeliocentricCoordinates,
};

const PACKAGE_NAME: &str = "pleiades-vsop87";
const BACKEND_LABEL: &str = "the VSOP87 backend";
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

    pub(crate) fn supported_bodies() -> &'static [CelestialBody] {
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
        if body == CelestialBody::Sun {
            return Some(Self::geocentric_sun_from_vsop87b(days));
        }

        if matches!(
            body,
            CelestialBody::Mercury
                | CelestialBody::Venus
                | CelestialBody::Mars
                | CelestialBody::Jupiter
                | CelestialBody::Saturn
                | CelestialBody::Uranus
                | CelestialBody::Neptune
        ) {
            let earth = Self::heliocentric_earth_from_vsop87b(days);
            let target = match body {
                CelestialBody::Mercury => Self::heliocentric_mercury_from_vsop87b(days),
                CelestialBody::Venus => Self::heliocentric_venus_from_vsop87b(days),
                CelestialBody::Mars => Self::heliocentric_mars_from_vsop87b(days),
                CelestialBody::Jupiter => Self::heliocentric_jupiter_from_vsop87b(days),
                CelestialBody::Saturn => Self::heliocentric_saturn_from_vsop87b(days),
                CelestialBody::Uranus => Self::heliocentric_uranus_from_vsop87b(days),
                CelestialBody::Neptune => Self::heliocentric_neptune_from_vsop87b(days),
                _ => unreachable!("body was checked above"),
            };
            return Some(HeliocentricCoordinates {
                xh: target.xh - earth.xh,
                yh: target.yh - earth.yh,
                zh: target.zh - earth.zh,
            });
        }

        let earth = Self::heliocentric_coordinates(Self::earth_elements(days));
        let target = Self::heliocentric_coordinates(Self::orbital_elements(body, days)?);
        Some(HeliocentricCoordinates {
            xh: target.xh - earth.xh,
            yh: target.yh - earth.yh,
            zh: target.zh - earth.zh,
        })
    }

    fn geocentric_sun_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let earth = Self::heliocentric_earth_from_vsop87b(days);
        HeliocentricCoordinates {
            xh: -earth.xh,
            yh: -earth.yh,
            zh: -earth.zh,
        }
    }

    fn heliocentric_earth_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let earth = tables::vsop87b_earth::earth_lbr(J2000 + days);
        spherical_lbr_to_cartesian(earth.longitude_rad, earth.latitude_rad, earth.radius_au)
    }

    fn heliocentric_mercury_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let mercury = tables::vsop87b_mercury::mercury_lbr(J2000 + days);
        spherical_lbr_to_cartesian(
            mercury.longitude_rad,
            mercury.latitude_rad,
            mercury.radius_au,
        )
    }

    fn heliocentric_venus_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let venus = tables::vsop87b_venus::venus_lbr(J2000 + days);
        spherical_lbr_to_cartesian(venus.longitude_rad, venus.latitude_rad, venus.radius_au)
    }

    fn heliocentric_mars_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let mars = tables::vsop87b_mars::mars_lbr(J2000 + days);
        spherical_lbr_to_cartesian(mars.longitude_rad, mars.latitude_rad, mars.radius_au)
    }

    fn heliocentric_jupiter_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let jupiter = tables::vsop87b_jupiter::jupiter_lbr(J2000 + days);
        spherical_lbr_to_cartesian(
            jupiter.longitude_rad,
            jupiter.latitude_rad,
            jupiter.radius_au,
        )
    }

    fn heliocentric_saturn_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let saturn = tables::vsop87b_saturn::saturn_lbr(J2000 + days);
        spherical_lbr_to_cartesian(saturn.longitude_rad, saturn.latitude_rad, saturn.radius_au)
    }

    fn heliocentric_uranus_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let uranus = tables::vsop87b_uranus::uranus_lbr(J2000 + days);
        spherical_lbr_to_cartesian(uranus.longitude_rad, uranus.latitude_rad, uranus.radius_au)
    }

    fn heliocentric_neptune_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let neptune = tables::vsop87b_neptune::neptune_lbr(J2000 + days);
        spherical_lbr_to_cartesian(
            neptune.longitude_rad,
            neptune.latitude_rad,
            neptune.radius_au,
        )
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
        Self::to_ecliptic(coords).to_equatorial(instant.mean_obliquity())
    }

    fn motion(body: CelestialBody, days: f64) -> Option<Motion> {
        // A symmetric one-day span gives stable chart-facing daily rates while
        // keeping the finite-difference mean geocentric model simple and
        // deterministic. These are finite-difference estimates of the same mean
        // geocentric model, not apparent velocities from a full VSOP87/light-time
        // reduction.
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
        let source_profiles = body_source_profiles();
        let vendored_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
            .count();
        let generated_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .count();
        let truncated_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .count();
        let fallback_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::MeanOrbitalElements)
            .count();

        let vendored_path_label = pluralize_body_path(vendored_count);
        let generated_path_label = pluralize_body_path(generated_count);
        let truncated_path_label = pluralize_body_path(truncated_count);
        let fallback_path_label = pluralize_body_path(fallback_count);

        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: if generated_count == 0 && truncated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else if vendored_count == 0 && truncated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {generated_count} generated binary VSOP87B {generated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else if generated_count > 0 && truncated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {generated_count} generated binary VSOP87B {generated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else if generated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {truncated_count} source-backed truncated VSOP87B {truncated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {generated_count} generated binary VSOP87B {generated_path_label}, {truncated_count} source-backed truncated VSOP87B {truncated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                },
                data_sources: crate::source_specifications()
                    .into_iter()
                    .map(|spec| {
                        format!(
                            "{}: IMCCE/CELMECH {} {} ({}, {}, {}, {}, {}, {}, {})",
                            spec.body,
                            spec.variant,
                            spec.source_file,
                            spec.coordinate_family,
                            spec.frame,
                            spec.units,
                            spec.reduction,
                            spec.transform_note,
                            spec.truncation_policy,
                            spec.date_range,
                        )
                    })
                    .chain([
                        "Paul Schlyter-style mean orbital elements for planets outside the source-backed VSOP87 coefficient tables".to_string(),
                        "Meeus-style coordinate transforms for geocentric reduction".to_string(),
                    ])
                    .collect(),
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
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
        validate_zodiac_policy(req, BACKEND_LABEL, &[ZodiacMode::Tropical])?;

        validate_request_policy(
            req,
            BACKEND_LABEL,
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            true,
            false,
        )?;

        validate_observer_policy(req, BACKEND_LABEL, false)?;

        let days = Self::days_since_j2000(req.instant);
        let geocentric = Self::geocentric_coordinates(req.body.clone(), days).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                format!("requested body is not implemented in {BACKEND_LABEL}"),
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
        result.quality = match source_kind_for_body(req.body.clone()) {
            Some(Vsop87BodySourceKind::VendoredVsop87b)
            | Some(Vsop87BodySourceKind::GeneratedBinaryVsop87b) => QualityAnnotation::Exact,
            Some(Vsop87BodySourceKind::TruncatedVsop87b)
            | Some(Vsop87BodySourceKind::MeanOrbitalElements)
            | None => QualityAnnotation::Approximate,
        };
        result.ecliptic = Some(Self::to_ecliptic(geocentric));
        result.equatorial = Some(Self::to_equatorial(geocentric, req.instant));
        result.motion = Self::motion(req.body.clone(), days);
        Ok(result)
    }
}
