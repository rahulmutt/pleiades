use core::{fmt, time::Duration};

use pleiades_ayanamsa::resolve_ayanamsa;
use pleiades_backend::{
    Apparentness, BackendMetadata, EphemerisError, EphemerisErrorKind, EphemerisRequest,
};
use pleiades_houses::{resolve_house_system, HouseRequest};
use pleiades_types::{
    CelestialBody, CoordinateFrame, HouseSystem, Instant, ObserverLocation, TimeScale,
    TimeScaleConversion, TimeScaleConversionError, ZodiacMode,
};

use super::errors::{map_custom_definition_error, map_house_error, map_observer_location_error};
use super::observer::{
    default_chart_bodies, render_house_system_label, ObserverPolicy, ObserverSummary,
    ObserverSummaryValidationError,
};

/// A request for chart assembly.
///
/// # Example
///
/// ```
/// use core::time::Duration;
/// use pleiades_core::ChartRequest;
/// use pleiades_types::{
///     HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
/// };
///
/// let request = ChartRequest::new(Instant::new(
///     JulianDay::from_days(2_451_545.0),
///     TimeScale::Utc,
/// ))
/// .with_observer(ObserverLocation::new(
///     Latitude::from_degrees(51.5),
///     Longitude::from_degrees(-0.1),
///     None,
/// ))
/// .with_house_system(HouseSystem::WholeSign)
/// .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
/// .expect("explicit time-scale conversion");
///
/// let summary = request.summary_line();
/// assert!(summary.contains("observer=house-only"));
/// assert!(summary.contains("house system=Whole Sign"));
/// assert_eq!(request.instant.scale, TimeScale::Tdb);
/// assert_eq!(request.house_system, Some(HouseSystem::WholeSign));
/// assert!(request.observer.is_some());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChartRequest {
    /// The instant to chart.
    pub instant: Instant,
    /// Optional observer location used for house calculations.
    ///
    /// Body positions are still queried geocentrically unless the separate
    /// `body_observer` field is set. A future topocentric chart mode should
    /// keep that position-mode split explicit instead of reusing this
    /// house-observer value implicitly.
    pub observer: Option<ObserverLocation>,
    /// Optional observer location used for topocentric body-position requests.
    pub body_observer: Option<ObserverLocation>,
    /// Bodies to include in the chart report.
    pub bodies: Vec<CelestialBody>,
    /// Desired zodiac mode.
    pub zodiac_mode: ZodiacMode,
    /// Preferred apparentness for backend position queries.
    pub apparentness: Apparentness,
    /// Optional house-system request.
    pub house_system: Option<HouseSystem>,
}

impl ChartRequest {
    /// Creates a default tropical chart request.
    pub fn new(instant: Instant) -> Self {
        Self {
            instant,
            observer: None,
            body_observer: None,
            bodies: default_chart_bodies().to_vec(),
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Mean,
            house_system: None,
        }
    }

    /// Sets the observer location.
    pub fn with_observer(mut self, observer: ObserverLocation) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Sets the observer location used for topocentric body-position requests.
    pub fn with_body_observer(mut self, body_observer: ObserverLocation) -> Self {
        self.body_observer = Some(body_observer);
        self
    }

    /// Replaces the chart instant with a caller-supplied time-scale offset.
    ///
    /// This is a mechanical convenience wrapper over
    /// [`Instant::with_time_scale_offset`]. It keeps the chosen conversion
    /// policy explicit without introducing any built-in Delta T or leap-second
    /// logic.
    pub fn with_instant_time_scale_offset(
        mut self,
        target_scale: TimeScale,
        offset_seconds: f64,
    ) -> Self {
        self.instant = self
            .instant
            .with_time_scale_offset(target_scale, offset_seconds);
        self
    }

    /// Replaces the chart instant with a caller-supplied time-scale offset after validation.
    ///
    /// This is the checked counterpart to [`ChartRequest::with_instant_time_scale_offset`].
    /// It validates the source scale and rejects non-finite offsets before the
    /// instant is retagged, which makes the raw offset convenience available in
    /// a release-grade form when the caller wants to stay at the chart façade.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ChartRequest;
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Ut1,
    /// ));
    /// let converted = request
    ///     .with_instant_time_scale_offset_checked(TimeScale::Tt, 64.184)
    ///     .expect("validated chart offset");
    ///
    /// assert_eq!(converted.instant.scale, TimeScale::Tt);
    /// ```
    pub fn with_instant_time_scale_offset_checked(
        mut self,
        target_scale: TimeScale,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .with_time_scale_offset_checked(target_scale, offset_seconds)?;
        Ok(self)
    }

    /// Applies a caller-supplied time-scale conversion policy to the chart
    /// instant.
    ///
    /// This is the generic counterpart to the source-specific conversion
    /// conveniences below. It keeps the explicit source, target, and offset
    /// choice available as a typed policy object while preserving the rest of
    /// the chart request shape.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ChartRequest;
    /// use pleiades_types::{Instant, JulianDay, TimeScale, TimeScaleConversion};
    ///
    /// let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tdb, 64.184);
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Ut1,
    /// ))
    /// .with_time_scale_conversion(policy)
    /// .expect("explicit time-scale policy");
    ///
    /// assert_eq!(request.instant.scale, TimeScale::Tdb);
    /// ```
    pub fn with_time_scale_conversion(
        mut self,
        conversion: TimeScaleConversion,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = conversion.apply(self.instant)?;
        Ok(self)
    }

    /// Validates a caller-supplied time-scale conversion policy without mutating the request.
    ///
    /// This mirrors [`TimeScaleConversion::validate`] at the chart façade so
    /// callers can preflight the explicit source/target/offset contract against
    /// the request instant before choosing whether to apply it.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ChartRequest;
    /// use pleiades_types::{Instant, JulianDay, TimeScale, TimeScaleConversion};
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Ut1,
    /// ));
    /// let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tdb, 64.184);
    ///
    /// assert!(request.validate_time_scale_conversion(policy).is_ok());
    /// ```
    pub fn validate_time_scale_conversion(
        &self,
        conversion: TimeScaleConversion,
    ) -> Result<(), TimeScaleConversionError> {
        self.instant.validate_time_scale_conversion(conversion)
    }

    /// Validates any custom identifiers embedded in the chart request.
    ///
    /// Built-in bodies, house systems, and ayanamsas are always accepted. When
    /// a custom body, custom house system, or custom ayanamsa is present, the
    /// structured descriptor is validated before chart assembly or backend
    /// metadata checks can proceed. Custom house-system and ayanamsa names are
    /// also checked against the built-in catalogs so user-defined labels stay
    /// distinguishable from the shipped compatibility profile.
    pub fn validate_custom_definitions(&self) -> Result<(), EphemerisError> {
        for body in &self.bodies {
            body.validate()
                .map_err(|error| map_custom_definition_error("chart body", error))?;
        }

        if let Some(house_system) = &self.house_system {
            house_system
                .validate_against_reserved_labels(|label| resolve_house_system(label).is_some())
                .map_err(|error| map_custom_definition_error("chart house system", error))?;
        }

        if let ZodiacMode::Sidereal { ayanamsa } = &self.zodiac_mode {
            ayanamsa
                .validate_against_reserved_labels(|label| resolve_ayanamsa(label).is_some())
                .map_err(|error| map_custom_definition_error("sidereal ayanamsa", error))?;
        }

        Ok(())
    }

    /// Validates the chart-level observer contract used for house calculations.
    ///
    /// Observer coordinates are checked whenever they are present, even when
    /// the request does not also ask for houses, so malformed house-observer
    /// or body-position observer data fails closed before chart assembly or
    /// report rendering uses it.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ChartRequest;
    /// use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Tt,
    /// ))
    /// .with_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(51.5),
    ///     Longitude::from_degrees(-0.1),
    ///     None,
    /// ));
    ///
    /// assert!(request.validate_observer_location().is_ok());
    /// ```
    pub fn validate_observer_location(&self) -> Result<(), EphemerisError> {
        if let Some(observer) = &self.observer {
            observer.validate().map_err(map_observer_location_error)?;
        }

        if let Some(observer) = &self.body_observer {
            observer.validate().map_err(map_observer_location_error)?;
        }

        Ok(())
    }

    /// Validates the chart-level observer contract used for house calculations.
    ///
    /// House requests require an observer location so the façade can keep the
    /// observer used for houses separate from the geocentric body-position
    /// backend requests. This preflight returns the same structured request
    /// error that chart assembly uses if the observer is missing, and it also
    /// reuses the observer-location validation so malformed observer
    /// coordinates fail before the façade gets to house derivation.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ChartRequest;
    /// use pleiades_types::{HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Tt,
    /// ))
    /// .with_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(51.5),
    ///     Longitude::from_degrees(-0.1),
    ///     None,
    /// ))
    /// .with_house_system(HouseSystem::WholeSign);
    ///
    /// assert!(request.validate_house_observer_policy().is_ok());
    /// ```
    pub fn validate_house_observer_policy(&self) -> Result<(), EphemerisError> {
        self.validate_observer_location()?;

        if let Some(system) = &self.house_system {
            let observer = self.observer.clone().ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "house placement requires an observer location",
                )
            })?;

            let house_request = HouseRequest::new(self.instant, observer, system.clone());
            house_request.validate().map_err(map_house_error)?;
        }

        Ok(())
    }

    /// Validates the chart request against backend metadata before chart assembly.
    ///
    /// This preflight keeps the chart façade aligned with the backend request
    /// contract: custom bodies, custom house systems, and custom ayanamsas are
    /// validated first; house requests still require an observer location; and
    /// each body request is checked against the backend's supported time scales,
    /// frames, apparentness mode, zodiac routing, and body coverage before the
    /// backend is asked to compute positions.
    pub fn validate_against_metadata(
        &self,
        metadata: &BackendMetadata,
    ) -> Result<(), EphemerisError> {
        self.validate_custom_definitions()?;
        self.validate_observer_location()?;
        self.validate_house_observer_policy()?;

        let backend_zodiac_mode = if matches!(self.zodiac_mode, ZodiacMode::Sidereal { .. })
            && metadata.capabilities.native_sidereal
        {
            self.zodiac_mode.clone()
        } else {
            ZodiacMode::Tropical
        };

        for body in &self.bodies {
            let body_request = EphemerisRequest {
                body: body.clone(),
                instant: self.instant,
                observer: self.body_observer.clone(),
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: backend_zodiac_mode.clone(),
                apparent: self.apparentness,
            };
            metadata.validate_request(&body_request)?;
        }

        Ok(())
    }

    /// Converts the chart instant from UT1 to TT using caller-supplied Delta T.
    ///
    /// This convenience method mirrors [`Instant::tt_from_ut1`] so chart callers
    /// can prepare a TT request surface before assembling houses or body
    /// positions.
    ///
    /// For signed `TT - UT1` policies, use [`ChartRequest::with_tt_from_ut1_signed`].
    pub fn with_tt_from_ut1(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1(delta_t)?;
        Ok(self)
    }

    /// Converts the chart instant from UT1 to TT using a caller-supplied signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_ut1_signed`] so chart
    /// callers can keep the chosen Delta T policy explicit when the `TT - UT1`
    /// correction is negative at the chosen epoch.
    pub fn with_tt_from_ut1_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TT using caller-supplied offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_utc`] so chart callers
    /// can keep civil-time inputs explicit without introducing built-in leap-
    /// second or Delta T modeling in the façade.
    ///
    /// For signed `TT - UTC` policies, use [`ChartRequest::with_tt_from_utc_signed`].
    pub fn with_tt_from_utc(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc(delta_t)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TT using a caller-supplied signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_utc_signed`] so chart
    /// callers can keep civil-time inputs explicit when the `TT - UTC`
    /// correction is negative at the chosen epoch.
    pub fn with_tt_from_utc_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from TT to TDB using caller-supplied offset.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_tt`] so chart
    /// callers can keep the chosen relativistic policy explicit without baking
    /// a built-in TDB model into the façade.
    pub fn with_tdb_from_tt(mut self, offset: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt(offset)?;
        Ok(self)
    }

    /// Converts the chart instant from TT to TDB using a caller-supplied
    /// signed offset.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_tt_signed`] so
    /// chart callers can keep the chosen relativistic policy explicit when the
    /// TDB-TT correction is negative at the chosen epoch.
    pub fn with_tdb_from_tt_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from TDB to TT using a caller-supplied
    /// signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_tdb`] so chart
    /// callers can keep the chosen relativistic policy explicit when they
    /// already start from a TDB-tagged instant.
    pub fn with_tt_from_tdb(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from TDB to TT using a caller-supplied
    /// signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_tdb_signed`] so chart
    /// callers can keep the chosen relativistic policy explicit while matching
    /// the signed helper naming used for the other time-scale conversions.
    pub fn with_tt_from_tdb_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from UT1 to TDB using caller-supplied
    /// TT-UT1 and TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_ut1`] so chart
    /// callers can prepare a TDB request surface from dynamical-time inputs
    /// without introducing built-in Delta T or relativistic modeling in the
    /// façade.
    pub fn with_tdb_from_ut1(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_ut1(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the chart instant from UT1 to TDB using caller-supplied
    /// TT-UT1 and signed TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_ut1_signed`] so
    /// chart callers can prepare a TDB request surface from dynamical-time
    /// inputs when the TDB-TT correction is negative at the chosen epoch.
    pub fn with_tdb_from_ut1_signed(
        mut self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .tdb_from_ut1_signed(tt_offset, tdb_offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TDB using caller-supplied
    /// TT-UTC and TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_utc`] so chart
    /// callers can prepare a TDB request surface from civil-time inputs
    /// without introducing built-in leap-second or relativistic modeling in the
    /// façade.
    pub fn with_tdb_from_utc(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_utc(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TDB using caller-supplied
    /// TT-UTC and signed TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_utc_signed`] so
    /// chart callers can prepare a TDB request surface from civil-time inputs
    /// when the TDB-TT correction is negative at the chosen epoch.
    pub fn with_tdb_from_utc_signed(
        mut self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .tdb_from_utc_signed(tt_offset, tdb_offset_seconds)?;
        Ok(self)
    }

    /// Sets the included bodies.
    pub fn with_bodies(mut self, bodies: Vec<CelestialBody>) -> Self {
        self.bodies = bodies;
        self
    }

    /// Sets the zodiac mode.
    pub fn with_zodiac_mode(mut self, zodiac_mode: ZodiacMode) -> Self {
        self.zodiac_mode = zodiac_mode;
        self
    }

    /// Sets the preferred apparentness.
    pub fn with_apparentness(mut self, apparentness: Apparentness) -> Self {
        self.apparentness = apparentness;
        self
    }

    /// Sets the house system.
    pub fn with_house_system(mut self, house_system: HouseSystem) -> Self {
        self.house_system = Some(house_system);
        self
    }

    /// Returns the observer policy implied by the current request shape.
    pub fn observer_policy(&self) -> ObserverPolicy {
        if self.observer.is_some() && self.house_system.is_some() {
            ObserverPolicy::HouseOnly
        } else {
            ObserverPolicy::Geocentric
        }
    }

    /// Returns the observer summary implied by the current request shape.
    pub fn observer_summary(&self) -> ObserverSummary {
        ObserverSummary::new(self.observer_policy(), self.observer.clone())
            .with_body_location(self.body_observer.clone())
    }

    /// Returns the observer summary implied by the current request shape after validation.
    pub fn validated_observer_summary(
        &self,
    ) -> Result<ObserverSummary, ObserverSummaryValidationError> {
        let summary = self.observer_summary();
        summary.validate()?;
        Ok(summary)
    }

    /// Returns the topocentric body-position observer label after validation.
    ///
    /// This is a convenience wrapper around
    /// [`ObserverSummary::validated_body_location_label()`] so chart-layer
    /// report code can inspect the optional body observer without
    /// reconstructing the summary record.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ChartRequest;
    /// use pleiades_types::{HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Tt,
    /// ))
    /// .with_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(51.5),
    ///     Longitude::from_degrees(-0.1),
    ///     None,
    /// ))
    /// .with_body_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(-33.9),
    ///     Longitude::from_degrees(151.2),
    ///     None,
    /// ))
    /// .with_house_system(HouseSystem::WholeSign);
    ///
    /// let label = request
    ///     .validated_body_observer_label()
    ///     .expect("valid body observer");
    /// assert!(label.contains("latitude=-33.9°"));
    /// ```
    pub fn validated_body_observer_label(&self) -> Result<String, ObserverSummaryValidationError> {
        self.observer_summary().validated_body_location_label()
    }

    /// Returns a compact one-line summary of the request shape.
    ///
    /// The summary is intended for diagnostics, validation reports, and
    /// release-facing policy notes. It uses the same compact terminology as
    /// `docs/time-observer-policy.md`:
    ///
    /// - the instant's explicit time scale is rendered directly,
    /// - observer-bearing chart requests are described as house-only when a
    ///   house system is also requested,
    /// - geocentric requests are described explicitly when no observer is set
    ///   or the observer is not used for house calculations,
    /// - house-observer and body-position observer locations are rendered
    ///   separately so house-only use stays explicit without implying a
    ///   topocentric body-position request,
    /// - the current zodiac mode, apparentness, body count, and house-system
    ///   selection stay visible in a single line.
    ///
    /// # Example
    ///
    /// ```
    /// use core::time::Duration;
    /// use pleiades_core::ChartRequest;
    /// use pleiades_types::{HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Utc,
    /// ))
    /// .with_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(51.5),
    ///     Longitude::from_degrees(-0.1),
    ///     None,
    /// ))
    /// .with_body_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(-33.9),
    ///     Longitude::from_degrees(151.2),
    ///     None,
    /// ))
    /// .with_house_system(HouseSystem::WholeSign)
    /// .with_tt_from_utc(Duration::from_secs_f64(64.184))
    /// .expect("explicit UTC to TT conversion");
    ///
    /// let summary = request.summary_line();
    /// assert!(summary.contains("observer=house-only"));
    /// assert!(summary.contains("body observer=latitude=-33.9°"));
    /// assert!(summary.contains("house system=Whole Sign"));
    /// ```
    ///
    /// [`fmt::Display`] renders the same text as this helper.
    pub fn summary_line(&self) -> String {
        let house_system = self
            .house_system
            .as_ref()
            .map_or_else(|| "none".to_string(), render_house_system_label);

        format!(
            "instant={} ({}); bodies={}; zodiac={}; apparentness={}; {}; house system={}",
            self.instant.julian_day,
            self.instant.scale,
            self.bodies.len(),
            self.zodiac_mode,
            self.apparentness,
            self.observer_summary(),
            house_system,
        )
    }

    /// Returns a compact one-line summary after validating the observer summary.
    pub fn validated_summary_line(&self) -> Result<String, ObserverSummaryValidationError> {
        self.validated_observer_summary()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ChartRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
