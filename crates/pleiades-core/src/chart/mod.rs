//! Chart assembly and higher-level chart helpers built on top of backend position queries.
//!
//! The current chart façade keeps the workflow intentionally small: callers
//! provide a set of bodies, the façade queries the backend, and the result
//! captures the body placements plus their zodiac signs and the chart-level
//! apparentness requested for the snapshot. Apparent-place corrections are
//! applied in the engine layer; first-party backends are always queried in
//! `Mean` mode, and an apparent placement's speed is moved to the apparent
//! place along with its coordinates. House placement can be requested
//! explicitly for chart-aware consumers, which keeps the workflow practical
//! without hardwiring more chart logic than the façade needs.

#[cfg(test)]
mod apparent_motion_tests;
#[cfg(test)]
mod apparent_tier_tests;
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
    apparent_apsis_position, apparent_equatorial_of_date, apparent_position, apparent_sun_position,
    precess_ecliptic_j2000_to_date, ApparentLightTimeError, ApparentPlaceError, ApparentPosition,
    DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{
    Apparentness, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
};
use pleiades_houses::{calculate_houses, house_for_longitude, HouseRequest};
use pleiades_types::{
    CelestialBody, CoordinateFrame, Instant, JulianDay, Motion, ObserverLocation, ZodiacMode,
};

use errors::map_house_error;
use pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS};

fn map_apparent_error(error: ApparentLightTimeError<EphemerisError>) -> EphemerisError {
    match error {
        ApparentLightTimeError::Query(inner) => inner,
        ApparentLightTimeError::Apparent(inner) => EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("apparent-place computation failed: {inner}"),
        ),
    }
}

fn map_apparent_place_error(error: ApparentPlaceError) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("apparent-place computation failed: {error}"),
    )
}

/// Whether `body` is a lunar orbit point (mean or true node, apogee, perigee).
///
/// These are geometric directions of the lunar orbit, not bodies. The chart
/// rotates them to the true ecliptic of date with precession + nutation in
/// longitude only: no light-time re-query and no annual aberration, and — in a
/// topocentric chart — no diurnal parallax or diurnal aberration, so they stay
/// geocentric. Swiss Ephemeris does the same. Without this exemption a point
/// carrying a lunar-scale distance would be shifted by up to about 1° of
/// parallax (issues #58, #63, #90).
fn is_lunar_point(body: &CelestialBody) -> bool {
    body.class() == pleiades_types::CelestialBodyClass::LunarPoint
}

/// An instant of the apparent-speed difference and the Sun's true geometric
/// longitude of date there, in degrees.
#[derive(Clone, Copy)]
struct SunSample {
    instant: Instant,
    sun_lon: f64,
}

/// `instant` shifted by `offset_days`, in the same time scale.
fn offset_instant(instant: Instant, offset_days: f64) -> Instant {
    Instant::new(
        JulianDay::from_days(instant.julian_day.days() + offset_days),
        instant.scale,
    )
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
    /// # Examples
    ///
    /// Tropical natal chart computed from a real offline backend —
    /// `pleiades-data`'s packaged artifact (a dev-dependency here). The Sun and
    /// Moon are release-grade via the packaged-data artifact, so the chart both
    /// resolves a sign for the Sun and exposes the Ascendant/Midheaven.
    ///
    /// ```
    /// use pleiades_core::{ChartEngine, ChartRequest};
    /// use pleiades_data::packaged_backend;
    /// use pleiades_types::{
    ///     CelestialBody, HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation,
    ///     TimeScale,
    /// };
    ///
    /// // 2000-01-01 12:00 TT; observer ~51.5°N, 0.1°W (London), elevation unspecified.
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    /// let observer = ObserverLocation::new(
    ///     Latitude::from_degrees(51.5),
    ///     Longitude::from_degrees(-0.1),
    ///     None,
    /// );
    ///
    /// let request = ChartRequest::new(instant)
    ///     .with_observer(observer)
    ///     .with_house_system(HouseSystem::Placidus)
    ///     .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);
    ///
    /// let engine = ChartEngine::new(packaged_backend());
    /// let chart = engine.chart(&request).expect("packaged artifact covers Sun & Moon");
    ///
    /// assert!(chart.sign_for_body(&CelestialBody::Sun).is_some());
    /// assert!(chart.asc_mc().is_some());
    /// ```
    ///
    /// Sidereal chart with an explicit Lahiri ayanamsa. First-party and packaged
    /// backends are always queried tropically; for a non-native-sidereal backend
    /// the chart layer subtracts the resolved Lahiri offset itself, so the
    /// snapshot carries the requested sidereal `zodiac_mode` and still resolves a
    /// sign for the Sun.
    ///
    /// ```
    /// use pleiades_core::{ChartEngine, ChartRequest};
    /// use pleiades_data::packaged_backend;
    /// use pleiades_types::{
    ///     Ayanamsa, CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation,
    ///     TimeScale, ZodiacMode,
    /// };
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    /// let request = ChartRequest::new(instant)
    ///     .with_observer(ObserverLocation::new(
    ///         Latitude::from_degrees(51.5),
    ///         Longitude::from_degrees(-0.1),
    ///         None,
    ///     ))
    ///     .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa: Ayanamsa::Lahiri })
    ///     .with_bodies(vec![CelestialBody::Sun]);
    ///
    /// let chart = ChartEngine::new(packaged_backend())
    ///     .chart(&request)
    ///     .expect("packaged artifact covers the Sun");
    ///
    /// assert!(chart.sign_for_body(&CelestialBody::Sun).is_some());
    /// assert_eq!(
    ///     chart.zodiac_mode,
    ///     ZodiacMode::Sidereal { ayanamsa: Ayanamsa::Lahiri },
    /// );
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

        // Apparent place is a frame reduction (light-time, precession, nutation,
        // aberration) that applies to every body the backend serves, whatever
        // its claim tier: the tier describes the backend's geometric accuracy,
        // which the reduction neither needs nor changes (issue #113). The Sun's
        // longitude of date feeds the aberration term, so a backend that cannot
        // serve the Sun cannot serve an apparent chart; fail closed rather than
        // return mean J2000 under an `Apparent` label.
        let apparent_requested = matches!(request.apparentness, Apparentness::Apparent);
        let sun_true_longitude_of_date = if apparent_requested && !request.bodies.is_empty() {
            let sun_lon = self
                .query_sun_longitude_of_date(request.instant, &backend_zodiac_mode)
                .map_err(|error| {
                    EphemerisError::new(
                        error.kind,
                        format!(
                            "apparent place needs the Sun's geocentric longitude for the \
                             aberration term, but {backend_id} could not serve it at this \
                             instant: {}; request Apparentness::Mean for the backend's raw \
                             mean J2000 place",
                            error.message
                        ),
                    )
                })?;
            Some(sun_lon)
        } else {
            None
        };
        // The Sun at the instants the apparent speed is differenced over, shared
        // by every body in the chart. A neighbour the backend cannot serve is
        // simply absent; the speed then falls back to a one-sided difference.
        let speed_suns = sun_true_longitude_of_date.map(|sun_lon| {
            [-HALF_SPAN_DAYS, 0.0, HALF_SPAN_DAYS].map(|offset_days| {
                let instant = offset_instant(request.instant, offset_days);
                let sun_lon = if offset_days == 0.0 {
                    Some(sun_lon)
                } else {
                    self.query_sun_longitude_of_date(instant, &backend_zodiac_mode)
                        .ok()
                };
                sun_lon.map(|sun_lon| SunSample { instant, sun_lon })
            })
        });

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
                    let outcome = self.apparent_place(
                        &body,
                        request.instant,
                        sun_lon,
                        &backend_zodiac_mode,
                        &request.body_observer,
                    );
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
                            // The backend's speed describes the mean place; move it
                            // to the apparent place stored above.
                            if let Some(mean) = position.motion {
                                position.motion = self.apparent_motion(
                                    mean,
                                    &body,
                                    speed_suns.as_ref().map_or(&[], |suns| suns.as_slice()),
                                    &backend_zodiac_mode,
                                    &request.body_observer,
                                );
                            }
                            Some(outcome.provenance)
                        }
                        Err(_) => {
                            // Apparent place unavailable for this body (e.g. unreliable
                            // distance channel, out-of-range retarded epoch). Gracefully
                            // fall back to the mean position already stored in
                            // `position.ecliptic`; leave it and the sign unchanged so
                            // the chart succeeds. `position.apparent` records the
                            // downgrade; see FU-24 in docs/follow-ups.md.
                            position.apparent = Apparentness::Mean;
                            None
                        }
                    }
                } else {
                    position.apparent = Apparentness::Mean;
                    None
                };
                // Opt-in chart-layer topocentric correction (diurnal parallax + diurnal aberration).
                // Operates on the tropical apparent ecliptic produced above; the sidereal
                // ayanamsa re-apply (when requested) happens once below, after this block.
                let mut apparent = apparent;
                let topocentric_prov = if request.topocentric && !is_lunar_point(&body) {
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
                    let eq_equinoxes = pleiades_apparent::sidereal::equation_of_equinoxes(
                        nut.delta_psi_arcsec / 3600.0,
                        true_obliquity,
                    );
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
                        // Record that diurnal parallax and diurnal aberration were applied
                        // in the apparent-provenance correction flags.
                        if let Some(ref mut prov) = apparent {
                            prov.corrections.diurnal_parallax = true;
                            prov.corrections.diurnal_aberration = true;
                        }
                        Some(topo.provenance)
                    } else {
                        None
                    }
                } else {
                    None
                };
                // Derive the apparent equatorial of date from the final tropical
                // apparent ecliptic (geocentric or topocentric), BEFORE the
                // sidereal longitude shift so RA/Dec stay ayanamsa-independent.
                // Mean-fallback rows (apparent.is_none()) keep the backend's
                // mean-obliquity equatorial. Degrade gracefully if nutation is
                // unavailable for this instant.
                if apparent.is_some() {
                    if let Some(ecliptic) = position.ecliptic.as_ref() {
                        let jd_tt = request.instant.julian_day.days();
                        if let Ok(eq) = apparent_equatorial_of_date(*ecliptic, jd_tt) {
                            position.equatorial = Some(eq);
                        }
                    }
                }
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

    /// Apparent place of a body at `instant`.
    /// `sun_longitude_of_date` is the Sun's true geometric longitude of date at
    /// the same instant, the argument of the annual-aberration term.
    fn apparent_place(
        &self,
        body: &CelestialBody,
        instant: Instant,
        sun_longitude_of_date: f64,
        zodiac_mode: &ZodiacMode,
        body_observer: &Option<ObserverLocation>,
    ) -> Result<ApparentPosition, EphemerisError> {
        match body {
            // Sun: light-time and aberration are the same ~20.5″ effect, so the
            // Sun path applies aberration ONCE to the instantaneous (un-retarded)
            // geocentric Sun. Every other body goes through apparent_position,
            // whose geocentric light-time re-query already carries aberration
            // (no separate term, #93). observer = None keeps the aberration
            // argument geocentric.
            CelestialBody::Sun => self
                .query_mean_ecliptic(body, instant, zodiac_mode, None)
                .and_then(|sun_j2000| {
                    apparent_sun_position(instant, sun_j2000).map_err(map_apparent_place_error)
                }),
            // Lunar orbit point: a geometric direction (see `is_lunar_point`).
            // observer = None keeps it geocentric.
            body if is_lunar_point(body) => self
                .query_mean_ecliptic(body, instant, zodiac_mode, None)
                .and_then(|point_j2000| {
                    apparent_apsis_position(instant, point_j2000).map_err(map_apparent_place_error)
                }),
            _ => apparent_position::<_, EphemerisError>(
                instant,
                sun_longitude_of_date,
                DEFAULT_MAX_ITERATIONS,
                |instant| {
                    self.query_mean_ecliptic(body, instant, zodiac_mode, body_observer.clone())
                },
            )
            .map_err(map_apparent_error),
        }
    }

    /// Apparent minus mean place of a body at one instant of the speed difference.
    fn correction_sample(
        &self,
        body: &CelestialBody,
        sun: &SunSample,
        zodiac_mode: &ZodiacMode,
        body_observer: &Option<ObserverLocation>,
    ) -> Result<CorrectionSample, EphemerisError> {
        let apparent =
            self.apparent_place(body, sun.instant, sun.sun_lon, zodiac_mode, body_observer)?;
        // The mean place the backend's speed describes: the same query the
        // chart's position batch makes.
        let mean =
            self.query_mean_ecliptic(body, sun.instant, zodiac_mode, body_observer.clone())?;
        Ok(CorrectionSample {
            julian_day: sun.instant.julian_day.days(),
            correction: Correction::between(&apparent.ecliptic, &mean),
        })
    }

    /// Speed of a body's apparent place, from the backend's `mean` speed.
    ///
    /// `suns` holds the instants before, at and after the chart instant. The
    /// correction is differenced centrally over the outer two; when the backend
    /// cannot serve one of them (the chart instant sits at the edge of its
    /// range) the difference is one-sided. With neither neighbour the apparent
    /// speed is unknown and `None` is returned: the mean speed would describe a
    /// different place from the one the placement reports.
    fn apparent_motion(
        &self,
        mean: Motion,
        body: &CelestialBody,
        suns: &[Option<SunSample>],
        zodiac_mode: &ZodiacMode,
        body_observer: &Option<ObserverLocation>,
    ) -> Option<Motion> {
        let sample = |sun: &Option<SunSample>| {
            self.correction_sample(body, sun.as_ref()?, zodiac_mode, body_observer)
                .ok()
        };
        let [earlier, centre, later] = suns else {
            return None;
        };
        let (earlier, later) = match (sample(earlier), sample(later)) {
            (Some(earlier), Some(later)) => (earlier, later),
            (Some(earlier), None) => (earlier, sample(centre)?),
            (None, Some(later)) => (sample(centre)?, later),
            (None, None) => return None,
        };
        Some(apparent_motion(mean, &earlier, &later))
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
        instant: Instant,
        zodiac_mode: &ZodiacMode,
    ) -> Result<f64, EphemerisError> {
        // The Sun longitude for the aberration term must remain geocentric — pass None.
        let ecliptic = self.query_mean_ecliptic(
            &pleiades_types::CelestialBody::Sun,
            instant,
            zodiac_mode,
            None,
        )?;
        // Precess the Sun's J2000 longitude to of-date so the aberration term is consistent.
        let precessed = precess_ecliptic_j2000_to_date(
            ecliptic.longitude.degrees(),
            ecliptic.latitude.degrees(),
            instant.julian_day.days(),
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
