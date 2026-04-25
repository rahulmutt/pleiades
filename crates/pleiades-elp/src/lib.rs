//! Lunar backend boundary based on a compact pure-Rust analytical model.
//!
//! The full ELP series data is still planned, but this crate now provides a
//! usable Moon-and-lunar-point backend for the chart MVP by combining a
//! compact Meeus-style truncated lunar position series with geocentric
//! coordinate transforms, Meeus-style mean node/perigee/apogee formulae, and
//! finite-difference mean-motion estimates.
//!
//! The current lunar-theory selection is intentionally explicit: the backend
//! exposes the Moon plus the mean/true node and mean apogee/perigee channels
//! through a small structured specification, and it also lists true apogee /
//! true perigee as unsupported bodies, so future source-backed ELP work can
//! attach provenance, supported channels, unsupported channels, and date-range
//! notes without changing the public API shape.
//!
//! See `docs/lunar-theory-policy.md` for the current baseline, validation
//! scope, and source/provenance posture.

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

mod moonposition;

const PACKAGE_NAME: &str = "pleiades-elp";
const J2000: f64 = 2_451_545.0;

/// Structured description of the current lunar-theory selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LunarTheorySpecification {
    /// Human-readable model name.
    pub model_name: &'static str,
    /// Stable identifier for the selected lunar-theory baseline.
    pub source_identifier: &'static str,
    /// Human-readable source/provenance note for the selected lunar baseline.
    pub source_material: &'static str,
    /// Redistribution or licensing posture for the selected baseline.
    pub redistribution_note: &'static str,
    /// Bodies/channels the current lunar baseline explicitly covers.
    pub supported_bodies: &'static [CelestialBody],
    /// Bodies/channels that are explicitly unsupported by this baseline.
    pub unsupported_bodies: &'static [CelestialBody],
    /// Notes the effective validation window or date-range posture.
    pub date_range_note: &'static str,
    /// Notes on the coordinate-frame treatment used by the baseline.
    pub frame_note: &'static str,
}

/// Returns the currently selected compact lunar-theory specification.
pub fn lunar_theory_specification() -> LunarTheorySpecification {
    const SUPPORTED_BODIES: &[CelestialBody] = &[
        CelestialBody::Moon,
        CelestialBody::MeanNode,
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::MeanPerigee,
    ];
    const UNSUPPORTED_BODIES: &[CelestialBody] =
        &[CelestialBody::TrueApogee, CelestialBody::TruePerigee];

    LunarTheorySpecification {
        model_name: "Compact Meeus-style truncated lunar baseline",
        source_identifier: "meeus-style-truncated-lunar-baseline",
        source_material:
            "Published lunar position, node, and mean-point formulas implemented as the current pure-Rust baseline; no vendored ELP coefficient files are used yet while full ELP coefficient selection remains pending",
        redistribution_note:
            "No external coefficient-file redistribution constraints apply to the current baseline because the implementation does not vendor ELP coefficient tables yet",
        supported_bodies: SUPPORTED_BODIES,
        unsupported_bodies: UNSUPPORTED_BODIES,
        date_range_note:
            "Validated against the published 1992-04-12 geocentric Moon example, J2000 node/perigee references, and nearby high-curvature regression windows; no full ELP coefficient range has been published yet",
        frame_note:
            "Geocentric ecliptic coordinates are produced directly from the truncated lunar series; equatorial coordinates are derived with a mean-obliquity transform",
    }
}

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

    fn moon_ecliptic_coordinates(days: f64) -> EclipticCoordinates {
        let (longitude, latitude, distance_au) = moonposition::position(J2000 + days);
        EclipticCoordinates::new(longitude, latitude, Some(distance_au))
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
                summary: format!(
                    "{} [{}] The backend exposes the Moon plus mean/true node and mean apogee/perigee channels as an explicit lunar-theory selection, while explicitly leaving true apogee/perigee unsupported for now.",
                    lunar_theory_specification().model_name,
                    lunar_theory_specification().source_identifier,
                ),
                data_sources: vec![
                    "Meeus-style truncated lunar orbit formulas implemented in pure Rust; see docs/lunar-theory-policy.md for the current baseline scope".to_string(),
                    lunar_theory_specification().source_identifier.to_string(),
                    lunar_theory_specification().source_material.to_string(),
                    lunar_theory_specification().redistribution_note.to_string(),
                    lunar_theory_specification().date_range_note.to_string(),
                    lunar_theory_specification().frame_note.to_string(),
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
                let coords = Self::moon_ecliptic_coordinates(days);
                result.ecliptic = Some(coords);
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    coords.longitude,
                    coords.latitude,
                    req.instant,
                    coords.distance_au,
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

fn normalize_degrees(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

fn signed_longitude_delta_degrees(start: f64, end: f64) -> f64 {
    (end - start + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_is_stable() {
        assert_eq!(PACKAGE_NAME, "pleiades-elp");
    }

    #[test]
    fn metadata_mentions_the_selected_lunar_theory() {
        let metadata = ElpBackend::new().metadata();
        let theory = lunar_theory_specification();

        assert!(metadata.provenance.summary.contains(theory.model_name));
        assert!(metadata
            .provenance
            .summary
            .contains(theory.source_identifier));
        assert!(metadata
            .provenance
            .summary
            .contains("true apogee/perigee unsupported"));
        assert!(metadata.provenance.data_sources.iter().any(
            |source| source.contains("Published lunar position, node, and mean-point formulas")
        ));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains(theory.source_identifier)));
        assert!(metadata.provenance.data_sources.iter().any(
            |source| source.contains("No external coefficient-file redistribution constraints")
        ));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("J2000")));
    }

    #[test]
    fn backend_supports_the_moon_and_lunar_nodes() {
        let backend = ElpBackend::new();
        assert!(backend.supports_body(CelestialBody::Moon));
        assert!(!backend.supports_body(CelestialBody::Sun));
    }

    #[test]
    fn published_moon_example_matches_reference() {
        let backend = ElpBackend::new();
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2_448_724.5),
            TimeScale::Tt,
        );
        let result = backend
            .position(&mean_request_at(CelestialBody::Moon, instant))
            .expect("moon query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let motion = result.motion.expect("motion should be populated");

        assert!((ecliptic.longitude.degrees() - 133.162_655).abs() < 1e-6);
        assert!((ecliptic.latitude.degrees() - -3.229_126).abs() < 1e-6);
        assert!(
            (ecliptic.distance_au.expect("moon distance should exist") * 149_597_870.700
                - 368_409.7)
                .abs()
                < 0.5
        );
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
    fn moon_samples_remain_finite_across_high_curvature_window() {
        let backend = ElpBackend::new();
        let instants = [J2000 - 1.0, J2000, J2000 + 1.0, J2000 + 2.0]
            .map(|days| Instant::new(pleiades_types::JulianDay::from_days(days), TimeScale::Tt));

        let mut previous_longitude: Option<f64> = None;
        let mut previous_distance: Option<f64> = None;

        for instant in instants {
            let result = backend
                .position(&mean_request_at(CelestialBody::Moon, instant))
                .expect("moon query should work");
            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let motion = result.motion.expect("motion should be populated");

            assert!(ecliptic.longitude.degrees().is_finite());
            assert!(ecliptic.latitude.degrees().is_finite());
            assert!(ecliptic
                .distance_au
                .expect("moon distance should exist")
                .is_finite());
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
            assert!(motion.longitude_deg_per_day.unwrap().abs() < 20.0);
            assert!(motion.latitude_deg_per_day.unwrap().abs() < 10.0);

            if let Some(previous_longitude) = previous_longitude {
                let delta = signed_longitude_delta_degrees(
                    previous_longitude,
                    ecliptic.longitude.degrees(),
                );
                assert!(delta.abs() > 1.0);
                assert!(delta.abs() < 20.0);
            }

            if let Some(previous_distance) = previous_distance {
                assert!(
                    (ecliptic.distance_au.expect("moon distance should exist") - previous_distance)
                        .abs()
                        < 0.02
                );
            }

            previous_longitude = Some(ecliptic.longitude.degrees());
            previous_distance = ecliptic.distance_au;
        }

        assert!(previous_longitude.is_some());
        assert!(previous_distance.is_some());
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
        let theory = lunar_theory_specification();

        assert_eq!(
            theory.model_name,
            "Compact Meeus-style truncated lunar baseline"
        );
        assert_eq!(
            theory.source_identifier,
            "meeus-style-truncated-lunar-baseline"
        );
        assert!(theory
            .source_material
            .contains("Published lunar position, node, and mean-point formulas"));
        assert!(theory
            .redistribution_note
            .contains("No external coefficient-file redistribution constraints"));
        assert!(theory.date_range_note.contains("1992-04-12"));
        assert!(theory.frame_note.contains("mean-obliquity"));
        assert_eq!(
            theory.supported_bodies,
            &[
                CelestialBody::Moon,
                CelestialBody::MeanNode,
                CelestialBody::TrueNode,
                CelestialBody::MeanApogee,
                CelestialBody::MeanPerigee,
            ]
        );
        assert_eq!(
            theory.unsupported_bodies,
            &[CelestialBody::TrueApogee, CelestialBody::TruePerigee]
        );

        assert!(backend.supports_body(CelestialBody::Moon));
        assert!(backend.supports_body(CelestialBody::MeanNode));
        assert!(backend.supports_body(CelestialBody::TrueNode));
        assert!(backend.supports_body(CelestialBody::MeanApogee));
        assert!(backend.supports_body(CelestialBody::MeanPerigee));
        assert!(!backend.supports_body(CelestialBody::TrueApogee));
        assert!(!backend.supports_body(CelestialBody::TruePerigee));
        assert!(!backend.supports_body(CelestialBody::Sun));

        let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
        for body in [
            CelestialBody::TrueApogee,
            CelestialBody::TruePerigee,
            CelestialBody::Sun,
        ] {
            let error = backend
                .position(&mean_request_at(body, instant))
                .expect_err("unsupported lunar bodies should fail explicitly");
            assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        }
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
