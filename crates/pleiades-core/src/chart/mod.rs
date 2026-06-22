//! Chart assembly and higher-level chart helpers built on top of backend position queries.
//!
//! The current chart façade keeps the workflow intentionally small: callers
//! provide a set of bodies, the façade queries the backend, and the result
//! captures the body placements plus their zodiac signs and the apparentness
//! mode used for the backend queries. House placement can be requested
//! explicitly for chart-aware consumers, which keeps the workflow practical
//! without hardwiring more chart logic than the façade needs.

mod aspects;
mod errors;
mod houses;
mod motion;
mod observer;
mod placement;
mod request;
mod sidereal;
mod signs;
mod snapshot;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use aspects::{
    validate_aspect_definitions, AspectDefinition, AspectDefinitionValidationError, AspectKind,
    AspectMatch, AspectSummary, AspectSummaryValidationError,
};
pub use houses::HouseSummary;
pub use motion::{MotionSummary, MotionSummaryValidationError};
pub use observer::{default_chart_bodies, ObserverPolicy, ObserverSummary};
pub use placement::BodyPlacement;
pub use request::{ChartRequest, CivilChartRequest};
pub use sidereal::sidereal_longitude;
pub use signs::SignSummary;
pub use snapshot::ChartSnapshot;

use pleiades_apparent::{
    apparent_position, precess_ecliptic_j2000_to_date, ApparentLightTimeError,
    DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{
    Apparentness, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
};
use pleiades_houses::{calculate_houses, house_for_longitude, HouseRequest};
use pleiades_types::{CoordinateFrame, ZodiacMode};

use errors::map_house_error;

fn map_apparent_error(error: ApparentLightTimeError<EphemerisError>) -> EphemerisError {
    match error {
        ApparentLightTimeError::Query(inner) => inner,
        ApparentLightTimeError::Apparent(inner) => EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("apparent-place computation failed: {inner}"),
        ),
    }
}

use crate::ChartEngine;

impl<B: EphemerisBackend> ChartEngine<B> {
    /// Assembles a basic chart snapshot from the backend.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::{
    ///     AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId,
    ///     BackendMetadata, BackendProvenance, BodyClaim, EphemerisBackend, EphemerisError,
    ///     EphemerisRequest, EphemerisResult, QualityAnnotation, TimeRange,
    /// };
    /// use pleiades_core::{ChartEngine, ChartRequest};
    /// use pleiades_types::{
    ///     Angle, CelestialBody, CoordinateFrame, EclipticCoordinates, HouseSystem, Instant,
    ///     JulianDay, Latitude, Longitude, ObserverLocation, TimeScale, ZodiacSign,
    /// };
    ///
    /// struct DemoBackend;
    ///
    /// impl EphemerisBackend for DemoBackend {
    ///     fn metadata(&self) -> BackendMetadata {
    ///         BackendMetadata {
    ///             id: BackendId::new("demo"),
    ///             version: "0.1.0".to_string(),
    ///             family: BackendFamily::Algorithmic,
    ///             provenance: BackendProvenance::new("demo chart backend"),
    ///             nominal_range: TimeRange::new(None, None),
    ///             supported_time_scales: vec![TimeScale::Tt],
    ///             body_claims: vec![BodyClaim::from(CelestialBody::Sun)],
    ///             supported_frames: vec![CoordinateFrame::Ecliptic],
    ///             capabilities: BackendCapabilities::default(),
    ///             accuracy: AccuracyClass::Approximate,
    ///             deterministic: true,
    ///             offline: true,
    ///         }
    ///     }
    ///
    ///     fn supports_body(&self, body: CelestialBody) -> bool {
    ///         body == CelestialBody::Sun
    ///     }
    ///
    ///     fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
    ///         let mut result = EphemerisResult::new(
    ///             BackendId::new("demo"),
    ///             request.body.clone(),
    ///             request.instant,
    ///             request.frame,
    ///             request.zodiac_mode.clone(),
    ///             request.apparent,
    ///         );
    ///         let ecliptic = EclipticCoordinates::new(
    ///             Longitude::from_degrees(15.0),
    ///             Latitude::from_degrees(0.0),
    ///             Some(1.0),
    ///         );
    ///         result.ecliptic = Some(ecliptic);
    ///         result.equatorial = Some(ecliptic.to_equatorial(Angle::from_degrees(23.4)));
    ///         result.motion = Some(pleiades_types::Motion::new(Some(1.0), None, None));
    ///         result.quality = QualityAnnotation::Exact;
    ///         Ok(result)
    ///     }
    /// }
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Utc,
    /// ))
    /// .with_tt_from_utc_signed(64.184)
    /// .expect("explicit UTC-to-TT conversion")
    /// .with_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(51.5),
    ///     Longitude::from_degrees(-0.1),
    ///     None,
    /// ))
    /// .with_house_system(HouseSystem::WholeSign)
    /// .with_bodies(vec![CelestialBody::Sun]);
    ///
    /// let snapshot = ChartEngine::new(DemoBackend)
    ///     .chart(&request)
    ///     .expect("demo chart should assemble");
    ///
    /// assert_eq!(snapshot.backend_id.as_str(), "demo");
    /// assert_eq!(snapshot.zodiac_mode, pleiades_types::ZodiacMode::Tropical);
    /// assert_eq!(snapshot.apparentness, Apparentness::Apparent);
    /// assert_eq!(snapshot.sign_for_body(&CelestialBody::Sun), Some(ZodiacSign::Aries));
    /// assert!(snapshot.houses.is_some());
    /// assert_eq!(snapshot.placements.len(), 1);
    /// ```
    ///
    /// The engine validates the backend metadata first so malformed backend
    /// inventory fails closed before the request shape is assembled. It then
    /// batches the body-position requests through the backend's `positions()`
    /// path so batch-aware backends can service the whole chart efficiently.
    pub fn chart(&self, request: &ChartRequest) -> Result<ChartSnapshot, EphemerisError> {
        request.validate_custom_definitions()?;

        let metadata = self.validated_metadata().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("backend metadata failed validation: {error}"),
            )
        })?;
        request.validate_against_metadata(&metadata)?;

        let backend_id = metadata.id.clone();
        let native_sidereal = matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. })
            && metadata.capabilities.native_sidereal;
        let backend_zodiac_mode = if native_sidereal {
            request.zodiac_mode.clone()
        } else {
            ZodiacMode::Tropical
        };

        let houses = if let Some(system) = &request.house_system {
            let observer = request.observer.clone().ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "house placement requires an observer location",
                )
            })?;

            let house_request = HouseRequest::new(request.instant, observer, system.clone());
            let mut snapshot = calculate_houses(&house_request).map_err(map_house_error)?;

            if matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. }) {
                for cusp in &mut snapshot.cusps {
                    *cusp = sidereal_longitude(*cusp, request.instant, &request.zodiac_mode)?;
                }
                snapshot.angles.ascendant = sidereal_longitude(
                    snapshot.angles.ascendant,
                    request.instant,
                    &request.zodiac_mode,
                )?;
                snapshot.angles.descendant = sidereal_longitude(
                    snapshot.angles.descendant,
                    request.instant,
                    &request.zodiac_mode,
                )?;
                snapshot.angles.midheaven = sidereal_longitude(
                    snapshot.angles.midheaven,
                    request.instant,
                    &request.zodiac_mode,
                )?;
                snapshot.angles.imum_coeli = sidereal_longitude(
                    snapshot.angles.imum_coeli,
                    request.instant,
                    &request.zodiac_mode,
                )?;
            }

            Some(snapshot)
        } else {
            None
        };

        let body_requests: Vec<_> = request
            .bodies
            .iter()
            .map(|body| EphemerisRequest {
                body: body.clone(),
                instant: request.instant,
                observer: request.body_observer.clone(),
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: backend_zodiac_mode.clone(),
                apparent: Apparentness::Mean,
            })
            .collect();
        let positions = self.backend.positions(&body_requests)?;
        if positions.len() != body_requests.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "{} returned {} result(s) for {} chart body request(s)",
                    backend_id,
                    positions.len(),
                    body_requests.len()
                ),
            ));
        }

        if request.topocentric {
            if matches!(request.apparentness, Apparentness::Mean) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "topocentric positions require apparent place; remove --mean",
                ));
            }
            if request.observer.is_none() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "topocentric positions require an observer location",
                ));
            }
        }

        let apparent_requested = matches!(request.apparentness, Apparentness::Apparent);
        let release_grade = metadata.release_grade_bodies();
        // Only query the Sun's longitude when there is at least one release-grade body
        // in the request. Non-release-grade bodies fall back to mean, so the Sun query
        // is unnecessary (and costly) when no body in the batch can be corrected.
        let has_release_grade_body = request.bodies.iter().any(|b| release_grade.contains(b));
        let sun_true_longitude_of_date = if apparent_requested && has_release_grade_body {
            Some(self.query_sun_longitude_of_date(request, &backend_zodiac_mode)?)
        } else {
            None
        };

        let placements = request
            .bodies
            .iter()
            .cloned()
            .zip(positions)
            .map(|(body, mut position)| {
                let sign = if matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. })
                    && !native_sidereal
                {
                    let instant = position.instant;
                    let longitude = position.ecliptic.as_mut().map(|coords| &mut coords.longitude);
                    let longitude = longitude.ok_or_else(|| {
                        EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            "sidereal chart assembly requires ecliptic coordinates from the backend",
                        )
                    })?;
                    *longitude = sidereal_longitude(*longitude, instant, &request.zodiac_mode)?;
                    Some(pleiades_types::ZodiacSign::from_longitude(*longitude))
                } else {
                    position
                        .ecliptic
                        .as_ref()
                        .map(|coords| pleiades_types::ZodiacSign::from_longitude(coords.longitude))
                };
                if matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. }) {
                    position.zodiac_mode = request.zodiac_mode.clone();
                }
                let house = houses.as_ref().and_then(|snapshot| {
                    position
                        .ecliptic
                        .as_ref()
                        .map(|coords| house_for_longitude(coords.longitude, &snapshot.cusps))
                });
                let apparent = if let Some(sun_lon) = sun_true_longitude_of_date {
                    if release_grade.contains(&body) {
                        let body_for_query = body.clone();
                        let body_observer_for_query = request.body_observer.clone();
                        let outcome = apparent_position::<_, EphemerisError>(
                            request.instant,
                            sun_lon,
                            DEFAULT_MAX_ITERATIONS,
                            |instant| self.query_mean_ecliptic(&body_for_query, instant, &backend_zodiac_mode, body_observer_for_query.clone()),
                        )
                        .map_err(map_apparent_error);
                        match outcome {
                            Ok(outcome) => {
                                if let Some(ecliptic) = position.ecliptic.as_mut() {
                                    // Store the tropical apparent ecliptic. The sidereal
                                    // ayanamsa re-apply (for non-native sidereal charts) is
                                    // deferred to after the topocentric block so it runs
                                    // exactly once on the final tropical longitude — whether
                                    // that is geocentric or topocentric apparent.
                                    *ecliptic = outcome.ecliptic;
                                }
                                position.apparent = Apparentness::Apparent;
                                Some(outcome.provenance)
                            }
                            Err(_) => {
                                // Apparent place unavailable for this release-grade body
                                // (e.g. unreliable distance channel, out-of-range retarded
                                // epoch). Gracefully fall back to the mean position already
                                // stored in `position.ecliptic`; leave it and the sign
                                // unchanged so the chart succeeds.
                                position.apparent = Apparentness::Mean;
                                None
                            }
                        }
                    } else {
                        // Non-release-grade: graceful mean fallback, not an error.
                        position.apparent = Apparentness::Mean;
                        None
                    }
                } else {
                    position.apparent = Apparentness::Mean;
                    None
                };
                // Opt-in chart-layer topocentric correction (diurnal parallax + diurnal aberration).
                // Operates on the tropical apparent ecliptic produced above; the sidereal
                // ayanamsa re-apply (when requested) happens once below, after this block.
                let topocentric_prov = if request.topocentric {
                    let observer = request.observer.as_ref().ok_or_else(|| {
                        EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            "topocentric chart requires an observer location",
                        )
                    })?;
                    let jd_tt = request.instant.julian_day.days();
                    let jd_ut1 = pleiades_time::ut1_jd_from_tt(jd_tt).map_err(|e| {
                        EphemerisError::new(EphemerisErrorKind::InvalidRequest, e.to_string())
                    })?;
                    let gmst = pleiades_time::gmst_degrees(jd_ut1);
                    let nut = pleiades_apparent::nutation::nutation(jd_tt)
                        .map_err(|e| map_apparent_error(pleiades_apparent::ApparentLightTimeError::Apparent(e)))?;
                    let mean_obliquity = pleiades_apparent::nutation::mean_obliquity_degrees(jd_tt);
                    let true_obliquity = mean_obliquity + nut.delta_eps_arcsec / 3600.0;
                    // Apparent sidereal time = GMST + equation of the equinoxes + east longitude.
                    let eq_equinoxes = (nut.delta_psi_arcsec / 3600.0) * true_obliquity.to_radians().cos();
                    let last = (gmst + eq_equinoxes + observer.longitude.degrees()).rem_euclid(360.0);
                    if let Some(ecliptic) = position.ecliptic.as_mut() {
                        let topo = pleiades_apparent::topocentric_position(
                            *ecliptic,
                            observer,
                            last,
                            true_obliquity,
                        )
                        .map_err(|e| map_apparent_error(pleiades_apparent::ApparentLightTimeError::Apparent(e)))?;
                        // Store the topocentric tropical ecliptic; do NOT apply sidereal here —
                        // the single unified re-apply below handles both the geocentric and
                        // topocentric apparent paths identically (ayanamsa exactly once).
                        *ecliptic = topo.ecliptic;
                        Some(topo.provenance)
                    } else {
                        None
                    }
                } else {
                    None
                };
                // For non-native sidereal charts, re-apply the ayanamsa to the final
                // tropical apparent longitude exactly once.  This covers both paths:
                //   • geocentric apparent (topocentric_prov is None)
                //   • topocentric apparent (topocentric_prov is Some)
                // The mean-fallback path already applied the ayanamsa in the pre-apparent
                // block above and does not reach here (topocentric is rejected in mean mode).
                if apparent.is_some()
                    && matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. })
                    && !native_sidereal
                {
                    if let Some(ecliptic) = position.ecliptic.as_mut() {
                        ecliptic.longitude = sidereal_longitude(
                            ecliptic.longitude,
                            request.instant,
                            &request.zodiac_mode,
                        )?;
                    }
                }

                // Re-derive the sign from the final (possibly apparent) longitude.
                let sign = position
                    .ecliptic
                    .as_ref()
                    .map(|coords| pleiades_types::ZodiacSign::from_longitude(coords.longitude))
                    .or(sign);
                Ok(BodyPlacement {
                    body,
                    position,
                    sign,
                    house,
                    apparent,
                    topocentric: topocentric_prov,
                })
            })
            .collect::<Result<Vec<_>, EphemerisError>>()?;

        Ok(ChartSnapshot {
            backend_id,
            instant: request.instant,
            observer: request.observer.clone(),
            body_observer: request.body_observer.clone(),
            zodiac_mode: request.zodiac_mode.clone(),
            apparentness: request.apparentness,
            houses,
            placements,
        })
    }

    fn query_mean_ecliptic(
        &self,
        body: &pleiades_types::CelestialBody,
        instant: pleiades_types::Instant,
        zodiac_mode: &ZodiacMode,
        observer: Option<pleiades_types::ObserverLocation>,
    ) -> Result<pleiades_types::EclipticCoordinates, EphemerisError> {
        let req = EphemerisRequest {
            body: body.clone(),
            instant,
            observer,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: zodiac_mode.clone(),
            apparent: Apparentness::Mean,
        };
        let result = self.backend.position(&req)?;
        result.ecliptic.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("apparent place requires ecliptic coordinates for {body}"),
            )
        })
    }

    fn query_sun_longitude_of_date(
        &self,
        request: &ChartRequest,
        zodiac_mode: &ZodiacMode,
    ) -> Result<f64, EphemerisError> {
        // The Sun longitude for the aberration term must remain geocentric — pass None.
        let ecliptic = self.query_mean_ecliptic(
            &pleiades_types::CelestialBody::Sun,
            request.instant,
            zodiac_mode,
            None,
        )?;
        // Precess the Sun's J2000 longitude to of-date so the aberration term is consistent.
        let precessed = precess_ecliptic_j2000_to_date(
            ecliptic.longitude.degrees(),
            ecliptic.latitude.degrees(),
            request.instant.julian_day.days(),
        )
        .map_err(|e| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("apparent-place Sun precession failed: {e}"),
            )
        })?;
        Ok(precessed.longitude_deg)
    }
}
