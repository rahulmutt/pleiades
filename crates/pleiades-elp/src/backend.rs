//! `ElpBackend` struct and `EphemerisBackend` implementation.

use pleiades_apparent::precess_ecliptic_vector_j2000_to_date;
use pleiades_apsides::{elements_from_state, points_from_elements, MU_EARTH_MOON_AU3_PER_DAY2};
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
/// and the osculating true node are served release-grade by `PackagedDataBackend` ahead of this
/// backend in the composite routing chain, so the ELP-local `Unsupported` apsis claims are not a
/// global gap. This backend's own `TrueNode` is the osculating ascending node formed from the
/// compact ELP Moon state (issue #127; it replaced Meeus's periodic-term-corrected mean node,
/// which reached 17′ from Swiss Ephemeris); it is reached by direct ELP and artifact-free
/// composite consumers and is held within 2′ of `SE_TRUE_NODE` by the `validate-true-node` gate.
/// Likewise `MeanApogee`/`MeanPerigee` here are the raw mean longitude-of-perigee element with
/// latitude 0, up to about 7′ in longitude and 5.1° in latitude from Swiss Ephemeris'
/// `SE_MEAN_APOG` point; routed charts get the Swiss Ephemeris point from `pleiades-data` (#90).
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

    /// Re-expresses an of-date geocentric ecliptic position in the J2000 mean
    /// equinox/ecliptic frame.
    ///
    /// Every ecliptic channel this backend emits passes through here, so the
    /// boundary frame is uniformly J2000 (consistent with every other
    /// first-party backend); consumers apply the forward J2000->date
    /// precession exactly once. The Meeus Ch. 47 series and polynomials are all
    /// referred to the mean equinox/ecliptic OF DATE, which is why this step is
    /// needed for the Moon and the lunar point channels alike (issue #57).
    fn ecliptic_of_date_to_j2000(
        longitude: Longitude,
        latitude: Latitude,
        distance_au: Option<f64>,
        jd_tt: f64,
    ) -> EclipticCoordinates {
        let precessed = pleiades_apparent::precess_ecliptic_date_to_j2000(
            longitude.degrees(),
            latitude.degrees(),
            jd_tt,
        )
        .expect("ELP of-date lon/lat precess cleanly to J2000");
        EclipticCoordinates::new(
            Longitude::from_degrees(precessed.longitude_deg),
            Latitude::from_degrees(precessed.latitude_deg),
            distance_au,
        )
    }

    pub(crate) fn moon_ecliptic_coordinates(days: f64) -> EclipticCoordinates {
        let jd_tt = crate::J2000 + days;
        let (longitude, latitude, distance_au) = crate::data::moonposition::position(jd_tt);
        Self::ecliptic_of_date_to_j2000(longitude, latitude, Some(distance_au), jd_tt)
    }

    /// Of-date position of a lunar point channel in the mean ecliptic of date:
    /// the mean node and mean apsides straight from the Meeus Ch. 47
    /// polynomials at latitude 0 and no distance, the true node as the
    /// osculating node with the orbit radius at the node. `None` for any body
    /// that is not a lunar point channel, or when the osculating node is
    /// undefined for the lunar state (a degenerate orbit, which the Moon never
    /// presents in practice).
    fn point_of_date(body: &CelestialBody, days: f64) -> Option<EclipticCoordinates> {
        let degrees = match body {
            CelestialBody::MeanNode => Self::mean_node_longitude(days),
            CelestialBody::TrueNode => return Self::osculating_true_node_of_date(days),
            CelestialBody::MeanApogee => Self::mean_apogee_longitude(days),
            CelestialBody::MeanPerigee => Self::mean_perigee_longitude(days),
            _ => return None,
        };
        Some(EclipticCoordinates::new(
            Longitude::from_degrees(degrees),
            Latitude::from_degrees(0.0),
            None,
        ))
    }

    /// J2000 boundary coordinates of a lunar point channel: the of-date point
    /// precessed back to J2000 exactly like the Moon. Expressed in J2000 the
    /// point carries a small non-zero latitude (≈±0.003° in 2026, the tilt
    /// between the two ecliptics); the consumer's forward J2000->date
    /// precession restores the of-date point on the ecliptic.
    fn point_ecliptic_j2000(body: &CelestialBody, days: f64) -> Option<EclipticCoordinates> {
        let of_date = Self::point_of_date(body, days)?;
        Some(Self::ecliptic_of_date_to_j2000(
            of_date.longitude,
            of_date.latitude,
            of_date.distance_au,
            crate::J2000 + days,
        ))
    }

    /// Geocentric J2000 Cartesian position of the compact ELP Moon, AU.
    fn moon_cartesian_j2000(days: f64) -> [f64; 3] {
        let moon = Self::moon_ecliptic_coordinates(days);
        let (sin_lon, cos_lon) = moon.longitude.degrees().to_radians().sin_cos();
        let (sin_lat, cos_lat) = moon.latitude.degrees().to_radians().sin_cos();
        let distance = moon
            .distance_au
            .expect("the ELP Moon series always carries a distance");
        [
            distance * cos_lat * cos_lon,
            distance * cos_lat * sin_lon,
            distance * sin_lat,
        ]
    }

    /// The osculating ascending node of the geocentric lunar orbit, in the mean
    /// ecliptic of date, with the orbit radius at the node as its distance.
    ///
    /// This is the same construction `PackagedDataBackend` uses for its
    /// release-grade `TrueNode`, applied to the compact ELP Moon: the J2000
    /// position is differenced over ±1e-4 day (Swiss Ephemeris'
    /// `NODE_CALC_INTV`) for the velocity, position and velocity are rotated
    /// once into the mean ecliptic of date, and the Keplerian node is formed
    /// there (forming it in J2000 and rotating the point would misplace a
    /// low-inclination node by ≈ tilt / sin i). Differencing in J2000 rather
    /// than of-date keeps the precession rate out of the velocity.
    ///
    /// Against Swiss Ephemeris `SE_TRUE_NODE` over 1900–2100 the result sits
    /// within 1.3′ (median 0.19′), set by the truncated Moon series; the
    /// Meeus Ch. 47 periodic-term node it replaced reached 17.4′ (issue #127).
    /// The chart layer's forward precession plus Δψ then reproduces
    /// `SE_TRUE_NODE` with nutation on.
    fn osculating_true_node_of_date(days: f64) -> Option<EclipticCoordinates> {
        const HALF_STEP_DAYS: f64 = 1.0e-4;
        let jd_tt = crate::J2000 + days;
        let position = Self::moon_cartesian_j2000(days);
        let before = Self::moon_cartesian_j2000(days - HALF_STEP_DAYS);
        let after = Self::moon_cartesian_j2000(days + HALF_STEP_DAYS);
        let velocity = [
            (after[0] - before[0]) / (2.0 * HALF_STEP_DAYS),
            (after[1] - before[1]) / (2.0 * HALF_STEP_DAYS),
            (after[2] - before[2]) / (2.0 * HALF_STEP_DAYS),
        ];
        let position = precess_ecliptic_vector_j2000_to_date(position, jd_tt).ok()?;
        let velocity = precess_ecliptic_vector_j2000_to_date(velocity, jd_tt).ok()?;
        let elements = elements_from_state(position, velocity, MU_EARTH_MOON_AU3_PER_DAY2).ok()?;
        let node = points_from_elements(&elements, false).ok()?.ascending;
        Some(EclipticCoordinates::new(
            Longitude::from_degrees(node.longitude_deg),
            Latitude::from_degrees(node.latitude_deg),
            Some(node.distance_au),
        ))
    }

    fn moon_ecliptic_of_date(days: f64) -> EclipticCoordinates {
        let (longitude, latitude, distance_au) =
            crate::data::moonposition::position(crate::J2000 + days);
        EclipticCoordinates::new(longitude, latitude, Some(distance_au))
    }

    fn mean_node_longitude(days: f64) -> f64 {
        pleiades_apsides::mean_lunar_node_longitude_of_date(crate::J2000 + days)
    }

    fn mean_perigee_longitude(days: f64) -> f64 {
        pleiades_apsides::mean_lunar_perigee_longitude_of_date(crate::J2000 + days)
    }

    /// The mean apogee *element*: mean perigee longitude + 180°, emitted with
    /// latitude 0. This is **not** the point Swiss Ephemeris reports as
    /// `SE_MEAN_APOG`, which lies on the inclined mean orbit (up to about 7′
    /// in longitude and 5.1° in latitude away). The routed chart chain serves
    /// the Swiss Ephemeris point from `PackagedDataBackend`; this channel
    /// remains for direct ELP consumers as a documented approximation (#90).
    fn mean_apogee_longitude(days: f64) -> f64 {
        series::normalize_degrees(Self::mean_perigee_longitude(days) + 180.0)
    }

    fn ecliptic_for_body(body: CelestialBody, days: f64) -> Option<EclipticCoordinates> {
        match body {
            CelestialBody::Moon => Some(Self::moon_ecliptic_coordinates(days)),
            point => Self::point_ecliptic_j2000(&point, days),
        }
    }

    fn motion(body: CelestialBody, days: f64) -> Option<Motion> {
        // The speed of the mean geometric place, as a central difference of the
        // series (not an apparent velocity; the chart and event layers add the
        // rate of the apparent-place correction). The ±0.5 day used before
        // biased it by h²/6 · λ‴: up to 30″/day for the Moon and 5.7″/day for
        // the osculating node (issue #140). The step balances that truncation
        // against the noise of the node, which is itself formed from a
        // differenced Moon velocity: against a fourth-order stencil of the
        // same positions, ±0.02 day holds the Moon within 0.05″/day and the
        // node within about 0.1″/day. A shorter step is better for the Moon
        // (0.004″/day at ±0.005 day) and worse for the node (0.2″/day).
        const HALF_SPAN_DAYS: f64 = 0.02;
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

    /// One request's result. `with_motion` adds the finite-difference speed,
    /// which costs two more lunar evaluations than the place itself.
    fn compute(
        &self,
        req: &EphemerisRequest,
        with_motion: bool,
    ) -> Result<EphemerisResult, EphemerisError> {
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
        match &body {
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
            point => {
                // Body support is validated above, so `None` here means the
                // osculating node is undefined for the lunar state.
                let Some(of_date) = Self::point_of_date(point, days) else {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        "the osculating lunar node is undefined for the ELP Moon state at this instant",
                    ));
                };
                // J2000 boundary, like the Moon.
                result.ecliptic = Some(Self::ecliptic_of_date_to_j2000(
                    of_date.longitude,
                    of_date.latitude,
                    of_date.distance_au,
                    crate::J2000 + days,
                ));
                // Mean-obliquity equatorial from the of-date point, like the Moon.
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    of_date.longitude,
                    of_date.latitude,
                    req.instant,
                    of_date.distance_au,
                ));
            }
        }
        if with_motion {
            result.motion = Self::motion(body, days);
        }
        Ok(result)
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
        self.compute(req, true)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.compute(req, false)
    }
}

#[cfg(test)]
mod tests {
    use super::ElpBackend;
    use crate::lunar_theory_supported_bodies;
    use pleiades_backend::{EphemerisBackend, EphemerisRequest, EphemerisResult};
    use pleiades_types::{Instant, JulianDay, TimeScale};

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

    #[test]
    fn position_without_motion_is_position_minus_motion() {
        let backend = ElpBackend;
        for jd in [2_460_763.5, 2_451_545.0, 2_433_282.5, 2_488_069.5] {
            for body in lunar_theory_supported_bodies() {
                let req = EphemerisRequest::new(
                    body.clone(),
                    Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
                );
                let full = backend.position(&req);
                let free = backend.position_without_motion(&req);
                match (full, free) {
                    (Ok(full), Ok(free)) => {
                        assert!(full.motion.is_some(), "{body:?} {jd}");
                        assert_eq!(
                            EphemerisResult {
                                motion: None,
                                ..full
                            },
                            free,
                            "{body:?} {jd}"
                        );
                    }
                    (Err(full), Err(free)) => assert_eq!(full, free, "{body:?} {jd}"),
                    (full, free) => panic!("{body:?} {jd}: {full:?} vs {free:?}"),
                }
            }
        }
    }
}
