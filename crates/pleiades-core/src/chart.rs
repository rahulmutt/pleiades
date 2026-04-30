//! Chart assembly and higher-level chart helpers built on top of backend position queries.
//!
//! The current chart façade keeps the workflow intentionally small: callers
//! provide a set of bodies, the façade queries the backend, and the result
//! captures the body placements plus their zodiac signs and the apparentness
//! mode used for the backend queries. House placement can be requested
//! explicitly for chart-aware consumers, which keeps the workflow practical
//! without hardwiring more chart logic than the façade needs.

use core::{fmt, time::Duration};

use pleiades_ayanamsa::{resolve_ayanamsa, sidereal_offset};
use pleiades_backend::{
    validate_request_against_metadata, Apparentness, BackendMetadata, EphemerisBackend,
    EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
};
use pleiades_houses::{
    calculate_houses, house_for_longitude, resolve_house_system, HouseRequest, HouseSnapshot,
};
use pleiades_types::{
    Angle, CelestialBody, CoordinateFrame, CustomDefinitionValidationError, HouseSystem, Instant,
    Longitude, Motion, MotionDirection, ObserverLocation, ObserverLocationValidationError,
    TimeScale, TimeScaleConversion, TimeScaleConversionError, ZodiacMode, ZodiacSign,
};

use crate::ChartEngine;

/// Default bodies included in a basic chart report.
pub const fn default_chart_bodies() -> &'static [CelestialBody] {
    &[
        CelestialBody::Sun,
        CelestialBody::Moon,
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

fn render_house_system_label(house_system: &HouseSystem) -> String {
    match house_system {
        HouseSystem::Custom(custom) => custom.to_string(),
        other => crate::house_system_descriptor(other)
            .map(|descriptor| descriptor.canonical_name.to_string())
            .unwrap_or_else(|| "Custom".to_string()),
    }
}

fn render_observer_location_label(observer: Option<&ObserverLocation>) -> String {
    observer
        .map(ObserverLocation::summary_line)
        .unwrap_or_else(|| "none".to_string())
}

/// Observer-policy summary for chart assembly and chart snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObserverPolicy {
    /// Body positions are queried geocentrically.
    Geocentric,
    /// The observer is used for house calculations only.
    HouseOnly,
}

impl ObserverPolicy {
    /// Returns the stable display label for the observer policy.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Geocentric => "geocentric",
            Self::HouseOnly => "house-only",
        }
    }

    /// Returns the compact one-line rendering of the observer policy.
    ///
    /// This mirrors [`core::fmt::Display`] so request and snapshot summaries can reuse a
    /// single typed vocabulary without duplicating the policy label table.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::ObserverPolicy;
    ///
    /// assert_eq!(ObserverPolicy::HouseOnly.summary_line(), "house-only");
    /// assert_eq!(ObserverPolicy::Geocentric.to_string(), "geocentric");
    /// ```
    pub const fn summary_line(self) -> &'static str {
        self.as_str()
    }
}

impl fmt::Display for ObserverPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Observer-policy and location summary for chart requests and snapshots.
///
/// The house-observer location and the optional body-position observer are
/// kept as separate typed fields so report text can distinguish them clearly.
///
/// # Example
///
/// ```
/// use pleiades_core::{ObserverPolicy, ObserverSummary};
/// use pleiades_types::{Latitude, Longitude, ObserverLocation};
///
/// let summary = ObserverSummary::new(
///     ObserverPolicy::HouseOnly,
///     Some(ObserverLocation::new(
///         Latitude::from_degrees(12.5),
///         Longitude::from_degrees(45.0),
///         Some(100.0),
///     )),
/// )
/// .with_body_location(Some(ObserverLocation::new(
///     Latitude::from_degrees(-33.9),
///     Longitude::from_degrees(151.2),
///     None,
/// )));
///
/// let rendered = summary
///     .validated_summary_line()
///     .expect("valid observer summary");
/// assert!(rendered.contains("observer=house-only"));
/// assert!(rendered.contains("observer location=latitude=12.5°"));
/// assert!(rendered.contains("body observer=latitude=-33.9°"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ObserverSummary {
    /// The observer posture implied by the chart shape.
    pub policy: ObserverPolicy,
    /// The optional observer location rendered in report output.
    pub location: Option<ObserverLocation>,
    /// The optional topocentric body-position observer rendered in report output.
    pub body_location: Option<ObserverLocation>,
}

/// Errors returned when an observer summary no longer matches the chart observer posture.
#[derive(Clone, Debug, PartialEq)]
pub enum ObserverSummaryValidationError {
    /// A house-only summary no longer carries an observer location.
    HouseOnlyMissingObserver,
    /// The rendered observer location is invalid.
    InvalidObserverLocation(ObserverLocationValidationError),
    /// The rendered body-position observer location is invalid.
    InvalidBodyPosition(ObserverLocationValidationError),
}

impl fmt::Display for ObserverSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HouseOnlyMissingObserver => {
                f.write_str("observer summary for house-only posture requires an observer location")
            }
            Self::InvalidObserverLocation(error) => {
                write!(f, "observer summary location is invalid: {error}")
            }
            Self::InvalidBodyPosition(error) => {
                write!(f, "observer summary body location is invalid: {error}")
            }
        }
    }
}

impl std::error::Error for ObserverSummaryValidationError {}

impl ObserverSummary {
    /// Creates a new observer summary from the typed policy and optional location.
    pub const fn new(policy: ObserverPolicy, location: Option<ObserverLocation>) -> Self {
        Self {
            policy,
            location,
            body_location: None,
        }
    }

    /// Sets the optional body-position observer rendered in report output.
    pub fn with_body_location(mut self, body_location: Option<ObserverLocation>) -> Self {
        self.body_location = body_location;
        self
    }

    /// Returns the stored observer location as a compact label.
    pub fn location_label(&self) -> String {
        render_observer_location_label(self.location.as_ref())
    }

    /// Returns the stored body-position observer location as a compact label.
    pub fn body_location_label(&self) -> String {
        render_observer_location_label(self.body_location.as_ref())
    }

    /// Validates that the observer summary still matches the current observer posture.
    pub fn validate(&self) -> Result<(), ObserverSummaryValidationError> {
        if self.policy == ObserverPolicy::HouseOnly && self.location.is_none() {
            return Err(ObserverSummaryValidationError::HouseOnlyMissingObserver);
        }

        if let Some(observer) = &self.location {
            observer
                .validate()
                .map_err(ObserverSummaryValidationError::InvalidObserverLocation)?;
        }

        if let Some(observer) = &self.body_location {
            observer
                .validate()
                .map_err(ObserverSummaryValidationError::InvalidBodyPosition)?;
        }

        Ok(())
    }

    /// Returns the stored observer location as a compact label after validation.
    pub fn validated_location_label(&self) -> Result<String, ObserverSummaryValidationError> {
        self.validate()?;
        Ok(self.location_label())
    }

    /// Returns a compact one-line rendering of the observer posture and location.
    pub fn summary_line(&self) -> String {
        format!(
            "observer={}; observer location={}; body observer={}",
            self.policy,
            self.location_label(),
            self.body_location_label(),
        )
    }

    /// Returns a compact one-line rendering after validating the observer summary.
    pub fn validated_summary_line(&self) -> Result<String, ObserverSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ObserverSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

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
            validate_request_against_metadata(&body_request, metadata)?;
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

/// A single body placement within a chart.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyPlacement {
    /// The queried body.
    pub body: CelestialBody,
    /// The raw backend result.
    pub position: EphemerisResult,
    /// The body’s zodiac sign in the requested mode, when ecliptic longitude is available.
    pub sign: Option<ZodiacSign>,
    /// The one-based house number, when house placement was requested.
    pub house: Option<usize>,
}

/// A chart snapshot suitable for user-facing reports.
///
/// The snapshot keeps the observer posture explicit even when the chart does
/// not include houses. An observer location by itself still leaves the body
/// placements geocentric unless the separate body-position observer is set;
/// the `house-only` policy only appears when the snapshot also carries computed
/// houses.
///
/// # Example
///
/// ```
/// use pleiades_core::{Apparentness, BackendId, ChartSnapshot};
/// use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale, ZodiacMode};
///
/// let snapshot = ChartSnapshot {
///     backend_id: BackendId::new("demo"),
///     instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
///     observer: Some(ObserverLocation::new(
///         Latitude::from_degrees(51.5),
///         Longitude::from_degrees(-0.1),
///         None,
///     )),
///     body_observer: None,
///     zodiac_mode: ZodiacMode::Tropical,
///     apparentness: Apparentness::Mean,
///     houses: None,
///     placements: Vec::new(),
/// };
///
/// let summary = snapshot.summary_line();
/// assert!(summary.contains("observer=geocentric"));
/// assert!(summary.contains("house system=none"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSnapshot {
    /// Backend that produced the chart.
    pub backend_id: crate::BackendId,
    /// Instant that was charted.
    pub instant: Instant,
    /// Optional observer location.
    pub observer: Option<ObserverLocation>,
    /// Optional observer location used for topocentric body positions.
    pub body_observer: Option<ObserverLocation>,
    /// Zodiac mode used for the chart.
    pub zodiac_mode: ZodiacMode,
    /// Apparentness used for the backend position queries.
    pub apparentness: Apparentness,
    /// Optional house snapshot.
    pub houses: Option<HouseSnapshot>,
    /// Ordered body placements.
    pub placements: Vec<BodyPlacement>,
}

impl ChartSnapshot {
    /// Returns the number of body placements in the chart.
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// Returns `true` if the chart contains no body placements.
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// Returns the observer policy implied by the computed chart snapshot.
    pub fn observer_policy(&self) -> ObserverPolicy {
        if self.observer.is_some() && self.houses.is_some() {
            ObserverPolicy::HouseOnly
        } else {
            ObserverPolicy::Geocentric
        }
    }

    /// Returns the observer summary implied by the computed chart snapshot.
    pub fn observer_summary(&self) -> ObserverSummary {
        ObserverSummary::new(self.observer_policy(), self.observer.clone())
            .with_body_location(self.body_observer.clone())
    }

    /// Returns the observer summary implied by the computed chart snapshot after validation.
    pub fn validated_observer_summary(
        &self,
    ) -> Result<ObserverSummary, ObserverSummaryValidationError> {
        let summary = self.observer_summary();
        summary.validate()?;
        Ok(summary)
    }

    /// Returns a compact one-line summary of the computed chart snapshot.
    ///
    /// The summary is intended for diagnostics, validation reports, and
    /// release-facing snapshot notes. It mirrors the request-shape vocabulary
    /// used by [`ChartRequest::summary_line`] while adding the backend ID and
    /// the computed placement count so callers can compare the assembled chart
    /// against the original request at a glance. The stored house observer and
    /// body-position observer locations, when present, are rendered separately
    /// from the observer policy so the geocentric-versus-house-only split
    /// stays explicit.
    pub fn summary_line(&self) -> String {
        let house_system = self.houses.as_ref().map_or_else(
            || "none".to_string(),
            |houses| render_house_system_label(&houses.system),
        );
        let house_cusp_count = self.houses.as_ref().map_or(0, |houses| houses.cusps.len());

        format!(
            "backend={}; instant={} ({}); placements={}; zodiac={}; apparentness={}; {}; house system={}; house cusps={}",
            self.backend_id,
            self.instant.julian_day,
            self.instant.scale,
            self.placements.len(),
            self.zodiac_mode,
            self.apparentness,
            self.observer_summary(),
            house_system,
            house_cusp_count,
        )
    }

    /// Returns a compact one-line summary after validating the observer summary.
    pub fn validated_summary_line(&self) -> Result<String, ObserverSummaryValidationError> {
        self.validated_observer_summary()?;
        Ok(self.summary_line())
    }

    /// Returns the first placement for a requested body, if present.
    pub fn placement_for(&self, body: &CelestialBody) -> Option<&BodyPlacement> {
        self.placements
            .iter()
            .find(|placement| &placement.body == body)
    }

    /// Returns the motion direction for a requested body, if it is known.
    pub fn motion_direction_for(&self, body: &CelestialBody) -> Option<MotionDirection> {
        self.placement_for(body)?.motion_direction()
    }

    /// Returns the backend motion sample for a requested body, if it is available.
    pub fn motion_for_body(&self, body: &CelestialBody) -> Option<&Motion> {
        self.placement_for(body)?.motion()
    }

    /// Returns the longitudinal motion speed for a requested body, if it is available.
    pub fn longitude_speed_for(&self, body: &CelestialBody) -> Option<f64> {
        self.placement_for(body)?.longitude_speed()
    }

    /// Returns the latitudinal motion speed for a requested body, if it is available.
    pub fn latitude_speed_for(&self, body: &CelestialBody) -> Option<f64> {
        self.placement_for(body)?.latitude_speed()
    }

    /// Returns the radial motion speed for a requested body, if it is available.
    pub fn distance_speed_for(&self, body: &CelestialBody) -> Option<f64> {
        self.placement_for(body)?.distance_speed()
    }

    /// Returns the sign for a requested body, if sign placement was computed.
    pub fn sign_for_body(&self, body: &CelestialBody) -> Option<ZodiacSign> {
        self.placement_for(body)?.sign
    }

    /// Returns the house number for a requested body, if house placement was computed.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::{Apparentness, BackendId, EphemerisResult};
    /// use pleiades_core::{BodyPlacement, ChartSnapshot};
    /// use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode, ZodiacSign};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    /// let position = EphemerisResult::new(
    ///     BackendId::new("demo"),
    ///     CelestialBody::Sun,
    ///     instant,
    ///     CoordinateFrame::Ecliptic,
    ///     ZodiacMode::Tropical,
    ///     Apparentness::Mean,
    /// );
    ///
    /// let chart = ChartSnapshot {
    ///     backend_id: BackendId::new("demo"),
    ///     instant,
    ///     observer: None,
    ///     body_observer: None,
    ///     zodiac_mode: ZodiacMode::Tropical,
    ///     apparentness: Apparentness::Mean,
    ///     houses: None,
    ///     placements: vec![BodyPlacement {
    ///         body: CelestialBody::Sun,
    ///         position,
    ///         sign: Some(ZodiacSign::Aries),
    ///         house: Some(1),
    ///     }],
    /// };
    ///
    /// assert_eq!(chart.house_for_body(&CelestialBody::Sun), Some(1));
    /// assert_eq!(chart.house_for_body(&CelestialBody::Moon), None);
    /// ```
    pub fn house_for_body(&self, body: &CelestialBody) -> Option<usize> {
        self.placement_for(body)?.house
    }

    /// Returns the angular separation between two bodies when both have ecliptic longitudes.
    pub fn angular_separation(&self, left: &CelestialBody, right: &CelestialBody) -> Option<Angle> {
        let left = self
            .placement_for(left)?
            .position
            .ecliptic
            .as_ref()?
            .longitude;
        let right = self
            .placement_for(right)?
            .position
            .ecliptic
            .as_ref()?
            .longitude;
        Some(angle_separation(left, right))
    }

    /// Returns all placements assigned to a requested zodiac sign.
    pub fn placements_in_sign(&self, sign: ZodiacSign) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(move |placement| placement.sign == Some(sign))
    }

    /// Returns all placements assigned to a requested house number.
    pub fn placements_in_house(&self, house: usize) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(move |placement| placement.house == Some(house))
    }

    /// Returns the placements that are currently classified as retrograde.
    pub fn retrograde_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements.iter().filter(|placement| {
            matches!(
                placement.motion_direction(),
                Some(MotionDirection::Retrograde)
            )
        })
    }

    /// Returns the placements that are currently classified as stationary.
    pub fn stationary_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements.iter().filter(|placement| {
            matches!(
                placement.motion_direction(),
                Some(MotionDirection::Stationary)
            )
        })
    }

    /// Returns the placements whose motion direction could not be determined.
    pub fn unknown_motion_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(|placement| placement.motion_direction().is_none())
    }

    /// Returns the placements that are currently classified with a requested motion direction.
    pub fn placements_with_motion_direction(
        &self,
        direction: MotionDirection,
    ) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(move |placement| placement.motion_direction() == Some(direction))
    }

    /// Returns the occupied zodiac signs in canonical zodiac order.
    pub fn occupied_signs(&self) -> Vec<ZodiacSign> {
        self.sign_summary().occupied_signs()
    }

    /// Returns the occupied house numbers in ascending order.
    pub fn occupied_houses(&self) -> Vec<usize> {
        self.house_summary().occupied_houses()
    }

    /// Returns the placements that are currently classified as direct.
    pub fn direct_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements_with_motion_direction(MotionDirection::Direct)
    }

    /// Returns a summary of the chart's zodiac-sign placements.
    pub fn sign_summary(&self) -> SignSummary {
        let mut summary = SignSummary::default();

        for placement in &self.placements {
            if let Some(sign) = placement.sign {
                summary.increment(sign);
            }
        }

        summary
    }

    /// Returns a summary of the chart's occupied houses.
    pub fn house_summary(&self) -> HouseSummary {
        let mut summary = HouseSummary::default();

        for placement in &self.placements {
            match placement.house {
                Some(house) => summary.increment(house),
                None => summary.unknown += 1,
            }
        }

        summary
    }

    /// Returns only the sign counts tied for the highest occupancy.
    pub fn dominant_sign_summary(&self) -> SignSummary {
        dominant_sign_summary(self.sign_summary())
    }

    /// Returns only the house counts tied for the highest occupancy.
    pub fn dominant_house_summary(&self) -> HouseSummary {
        dominant_house_summary(self.house_summary())
    }

    /// Returns a summary of the chart's motion-direction classifications.
    pub fn motion_summary(&self) -> MotionSummary {
        let mut summary = MotionSummary::default();

        for placement in &self.placements {
            match placement.motion_direction() {
                Some(MotionDirection::Direct) => summary.direct += 1,
                Some(MotionDirection::Stationary) => summary.stationary += 1,
                Some(MotionDirection::Retrograde) => summary.retrograde += 1,
                None => summary.unknown += 1,
            }
        }

        summary
    }

    /// Returns a summary of the major aspects found in the chart.
    pub fn aspect_summary(&self) -> AspectSummary {
        AspectSummary::from_matches(&self.major_aspects())
    }

    /// Returns the major aspects found in the chart using the built-in orb defaults.
    pub fn major_aspects(&self) -> Vec<AspectMatch> {
        self.aspects_with_definitions_checked(default_aspect_definitions())
            .expect("default aspect definitions should be valid")
    }

    /// Returns aspect matches using custom aspect definitions after validating them.
    pub fn aspects_with_definitions_checked(
        &self,
        definitions: &[AspectDefinition],
    ) -> Result<Vec<AspectMatch>, AspectDefinitionValidationError> {
        validate_aspect_definitions(definitions)?;
        Ok(self.aspects_with_definitions(definitions))
    }

    /// Returns aspect matches using custom aspect definitions.
    pub fn aspects_with_definitions(&self, definitions: &[AspectDefinition]) -> Vec<AspectMatch> {
        let mut matches = Vec::new();

        for (left_index, left) in self.placements.iter().enumerate() {
            let Some(left_longitude) = left
                .position
                .ecliptic
                .as_ref()
                .map(|coords| coords.longitude)
            else {
                continue;
            };

            for right in self.placements.iter().skip(left_index + 1) {
                let Some(right_longitude) = right
                    .position
                    .ecliptic
                    .as_ref()
                    .map(|coords| coords.longitude)
                else {
                    continue;
                };

                let separation = angle_separation(left_longitude, right_longitude);
                if let Some(definition) = best_aspect_definition(separation, definitions) {
                    matches.push(AspectMatch {
                        left: left.body.clone(),
                        right: right.body.clone(),
                        kind: definition.kind,
                        separation,
                        orb: Angle::from_degrees(
                            (separation.degrees() - definition.exact_degrees).abs(),
                        ),
                    });
                }
            }
        }

        matches
    }
}

impl BodyPlacement {
    /// Returns the backend motion sample when the backend supplied motion data.
    pub fn motion(&self) -> Option<&Motion> {
        self.position.motion.as_ref()
    }

    /// Returns the coarse direction of longitudinal motion when the backend supplied motion data.
    pub fn motion_direction(&self) -> Option<MotionDirection> {
        let motion = self.motion()?;
        motion.validate().ok()?;
        motion.longitude_direction()
    }

    /// Returns the longitudinal motion speed when the backend supplied motion data.
    pub fn longitude_speed(&self) -> Option<f64> {
        self.motion()?.longitude_speed()
    }

    /// Returns the latitudinal motion speed when the backend supplied motion data.
    pub fn latitude_speed(&self) -> Option<f64> {
        self.motion()?.latitude_speed()
    }

    /// Returns the radial motion speed when the backend supplied motion data.
    pub fn distance_speed(&self) -> Option<f64> {
        self.motion()?.distance_speed()
    }

    /// Returns a compact one-line summary of the placement row used in chart reports.
    pub fn summary_line(&self) -> String {
        let longitude = self
            .position
            .ecliptic
            .as_ref()
            .map(|coords| format!("{}", coords.longitude))
            .unwrap_or_else(|| "n/a".to_string());
        let sign = self
            .sign
            .map(|sign| sign.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        let house = self
            .house
            .map(|house| house.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        let motion = self
            .motion_direction()
            .map(|direction| direction.to_string())
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "{:<12} {:>9}  {:<10}  {:>3}  {:<10}  {}",
            self.body, longitude, sign, house, motion, self.position.quality,
        )
    }
}

impl fmt::Display for BodyPlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A summary of zodiac-sign placements in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SignSummary {
    /// Placements in Aries.
    pub aries: usize,
    /// Placements in Taurus.
    pub taurus: usize,
    /// Placements in Gemini.
    pub gemini: usize,
    /// Placements in Cancer.
    pub cancer: usize,
    /// Placements in Leo.
    pub leo: usize,
    /// Placements in Virgo.
    pub virgo: usize,
    /// Placements in Libra.
    pub libra: usize,
    /// Placements in Scorpio.
    pub scorpio: usize,
    /// Placements in Sagittarius.
    pub sagittarius: usize,
    /// Placements in Capricorn.
    pub capricorn: usize,
    /// Placements in Aquarius.
    pub aquarius: usize,
    /// Placements in Pisces.
    pub pisces: usize,
}

impl SignSummary {
    fn increment(&mut self, sign: ZodiacSign) {
        match sign {
            ZodiacSign::Aries => self.aries += 1,
            ZodiacSign::Taurus => self.taurus += 1,
            ZodiacSign::Gemini => self.gemini += 1,
            ZodiacSign::Cancer => self.cancer += 1,
            ZodiacSign::Leo => self.leo += 1,
            ZodiacSign::Virgo => self.virgo += 1,
            ZodiacSign::Libra => self.libra += 1,
            ZodiacSign::Scorpio => self.scorpio += 1,
            ZodiacSign::Sagittarius => self.sagittarius += 1,
            ZodiacSign::Capricorn => self.capricorn += 1,
            ZodiacSign::Aquarius => self.aquarius += 1,
            ZodiacSign::Pisces => self.pisces += 1,
            _ => {}
        }
    }

    /// Returns `true` when the snapshot contains at least one sign placement.
    pub fn has_known_signs(self) -> bool {
        self.aries
            + self.taurus
            + self.gemini
            + self.cancer
            + self.leo
            + self.virgo
            + self.libra
            + self.scorpio
            + self.sagittarius
            + self.capricorn
            + self.aquarius
            + self.pisces
            > 0
    }

    /// Returns the occupied zodiac signs in canonical zodiac order.
    pub fn occupied_signs(self) -> Vec<ZodiacSign> {
        let mut signs = Vec::new();
        for (count, sign) in [
            (self.aries, ZodiacSign::Aries),
            (self.taurus, ZodiacSign::Taurus),
            (self.gemini, ZodiacSign::Gemini),
            (self.cancer, ZodiacSign::Cancer),
            (self.leo, ZodiacSign::Leo),
            (self.virgo, ZodiacSign::Virgo),
            (self.libra, ZodiacSign::Libra),
            (self.scorpio, ZodiacSign::Scorpio),
            (self.sagittarius, ZodiacSign::Sagittarius),
            (self.capricorn, ZodiacSign::Capricorn),
            (self.aquarius, ZodiacSign::Aquarius),
            (self.pisces, ZodiacSign::Pisces),
        ] {
            if count > 0 {
                signs.push(sign);
            }
        }
        signs
    }
}

fn dominant_sign_summary(summary: SignSummary) -> SignSummary {
    let max = [
        summary.aries,
        summary.taurus,
        summary.gemini,
        summary.cancer,
        summary.leo,
        summary.virgo,
        summary.libra,
        summary.scorpio,
        summary.sagittarius,
        summary.capricorn,
        summary.aquarius,
        summary.pisces,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if max == 0 {
        return SignSummary::default();
    }

    SignSummary {
        aries: if summary.aries == max {
            summary.aries
        } else {
            0
        },
        taurus: if summary.taurus == max {
            summary.taurus
        } else {
            0
        },
        gemini: if summary.gemini == max {
            summary.gemini
        } else {
            0
        },
        cancer: if summary.cancer == max {
            summary.cancer
        } else {
            0
        },
        leo: if summary.leo == max { summary.leo } else { 0 },
        virgo: if summary.virgo == max {
            summary.virgo
        } else {
            0
        },
        libra: if summary.libra == max {
            summary.libra
        } else {
            0
        },
        scorpio: if summary.scorpio == max {
            summary.scorpio
        } else {
            0
        },
        sagittarius: if summary.sagittarius == max {
            summary.sagittarius
        } else {
            0
        },
        capricorn: if summary.capricorn == max {
            summary.capricorn
        } else {
            0
        },
        aquarius: if summary.aquarius == max {
            summary.aquarius
        } else {
            0
        },
        pisces: if summary.pisces == max {
            summary.pisces
        } else {
            0
        },
    }
}

impl fmt::Display for SignSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_any = false;
        for (count, sign) in [
            (self.aries, ZodiacSign::Aries),
            (self.taurus, ZodiacSign::Taurus),
            (self.gemini, ZodiacSign::Gemini),
            (self.cancer, ZodiacSign::Cancer),
            (self.leo, ZodiacSign::Leo),
            (self.virgo, ZodiacSign::Virgo),
            (self.libra, ZodiacSign::Libra),
            (self.scorpio, ZodiacSign::Scorpio),
            (self.sagittarius, ZodiacSign::Sagittarius),
            (self.capricorn, ZodiacSign::Capricorn),
            (self.aquarius, ZodiacSign::Aquarius),
            (self.pisces, ZodiacSign::Pisces),
        ] {
            if count == 0 {
                continue;
            }
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} {}", count, sign)?;
        }

        if !wrote_any {
            f.write_str("no sign placements")?;
        }

        Ok(())
    }
}

/// A summary of occupied houses in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HouseSummary {
    /// Placements in the first house.
    pub first: usize,
    /// Placements in the second house.
    pub second: usize,
    /// Placements in the third house.
    pub third: usize,
    /// Placements in the fourth house.
    pub fourth: usize,
    /// Placements in the fifth house.
    pub fifth: usize,
    /// Placements in the sixth house.
    pub sixth: usize,
    /// Placements in the seventh house.
    pub seventh: usize,
    /// Placements in the eighth house.
    pub eighth: usize,
    /// Placements in the ninth house.
    pub ninth: usize,
    /// Placements in the tenth house.
    pub tenth: usize,
    /// Placements in the eleventh house.
    pub eleventh: usize,
    /// Placements in the twelfth house.
    pub twelfth: usize,
    /// Placements without an assigned house.
    pub unknown: usize,
}

impl HouseSummary {
    fn increment(&mut self, house: usize) {
        match house {
            1 => self.first += 1,
            2 => self.second += 1,
            3 => self.third += 1,
            4 => self.fourth += 1,
            5 => self.fifth += 1,
            6 => self.sixth += 1,
            7 => self.seventh += 1,
            8 => self.eighth += 1,
            9 => self.ninth += 1,
            10 => self.tenth += 1,
            11 => self.eleventh += 1,
            12 => self.twelfth += 1,
            _ => self.unknown += 1,
        }
    }

    /// Returns `true` when the snapshot contains at least one assigned house placement.
    pub fn has_known_houses(self) -> bool {
        self.first
            + self.second
            + self.third
            + self.fourth
            + self.fifth
            + self.sixth
            + self.seventh
            + self.eighth
            + self.ninth
            + self.tenth
            + self.eleventh
            + self.twelfth
            > 0
    }

    /// Returns the occupied house numbers in ascending order.
    pub fn occupied_houses(self) -> Vec<usize> {
        let mut houses = Vec::new();
        for (count, house) in [
            (self.first, 1usize),
            (self.second, 2),
            (self.third, 3),
            (self.fourth, 4),
            (self.fifth, 5),
            (self.sixth, 6),
            (self.seventh, 7),
            (self.eighth, 8),
            (self.ninth, 9),
            (self.tenth, 10),
            (self.eleventh, 11),
            (self.twelfth, 12),
        ] {
            if count > 0 {
                houses.push(house);
            }
        }
        houses
    }
}

fn dominant_house_summary(summary: HouseSummary) -> HouseSummary {
    let max = [
        summary.first,
        summary.second,
        summary.third,
        summary.fourth,
        summary.fifth,
        summary.sixth,
        summary.seventh,
        summary.eighth,
        summary.ninth,
        summary.tenth,
        summary.eleventh,
        summary.twelfth,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if max == 0 {
        return HouseSummary::default();
    }

    HouseSummary {
        first: if summary.first == max {
            summary.first
        } else {
            0
        },
        second: if summary.second == max {
            summary.second
        } else {
            0
        },
        third: if summary.third == max {
            summary.third
        } else {
            0
        },
        fourth: if summary.fourth == max {
            summary.fourth
        } else {
            0
        },
        fifth: if summary.fifth == max {
            summary.fifth
        } else {
            0
        },
        sixth: if summary.sixth == max {
            summary.sixth
        } else {
            0
        },
        seventh: if summary.seventh == max {
            summary.seventh
        } else {
            0
        },
        eighth: if summary.eighth == max {
            summary.eighth
        } else {
            0
        },
        ninth: if summary.ninth == max {
            summary.ninth
        } else {
            0
        },
        tenth: if summary.tenth == max {
            summary.tenth
        } else {
            0
        },
        eleventh: if summary.eleventh == max {
            summary.eleventh
        } else {
            0
        },
        twelfth: if summary.twelfth == max {
            summary.twelfth
        } else {
            0
        },
        unknown: 0,
    }
}

impl fmt::Display for HouseSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_any = false;
        for (count, house) in [
            (self.first, 1usize),
            (self.second, 2),
            (self.third, 3),
            (self.fourth, 4),
            (self.fifth, 5),
            (self.sixth, 6),
            (self.seventh, 7),
            (self.eighth, 8),
            (self.ninth, 9),
            (self.tenth, 10),
            (self.eleventh, 11),
            (self.twelfth, 12),
        ] {
            if count == 0 {
                continue;
            }
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} in {} house", count, house_ordinal(house))?;
        }

        if self.unknown > 0 {
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} unassigned", self.unknown)?;
        }

        if !wrote_any {
            f.write_str("no house placements")?;
        }

        Ok(())
    }
}

/// A summary of motion-direction classifications in a chart snapshot.
///
/// The counts track the current chart placements in the order used by the
/// compact report text: direct, stationary, retrograde, then unknown.
///
/// # Example
///
/// ```
/// use pleiades_core::MotionSummary;
///
/// let summary = MotionSummary {
///     direct: 2,
///     stationary: 1,
///     retrograde: 3,
///     unknown: 0,
/// };
///
/// assert_eq!(summary.summary_line(), "2 direct, 1 stationary, 3 retrograde, 0 unknown");
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert_eq!(summary.validate(6), Ok(()));
/// assert!(summary.has_known_motion());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MotionSummary {
    /// Placements classified as direct.
    pub direct: usize,
    /// Placements classified as stationary.
    pub stationary: usize,
    /// Placements classified as retrograde.
    pub retrograde: usize,
    /// Placements without enough motion data to classify.
    pub unknown: usize,
}

/// Errors returned when a motion summary no longer matches the chart placements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionSummaryValidationError {
    /// The total motion-summary count did not match the chart placement count.
    PlacementCountMismatch {
        /// Expected number of placements.
        expected: usize,
        /// Motion-summary total that was observed.
        actual: usize,
    },
}

impl fmt::Display for MotionSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlacementCountMismatch { expected, actual } => write!(
                f,
                "motion summary placement count mismatch: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for MotionSummaryValidationError {}

impl MotionSummary {
    /// Returns `true` when the snapshot contains at least one known motion classification.
    pub fn has_known_motion(self) -> bool {
        self.direct + self.stationary + self.retrograde > 0
    }

    /// Validates that the summary covers the expected number of placements.
    pub fn validate(self, placement_count: usize) -> Result<(), MotionSummaryValidationError> {
        let actual = self.direct + self.stationary + self.retrograde + self.unknown;
        if actual == placement_count {
            Ok(())
        } else {
            Err(MotionSummaryValidationError::PlacementCountMismatch {
                expected: placement_count,
                actual,
            })
        }
    }

    /// Returns a compact one-line summary of the motion classifications in the snapshot.
    pub fn summary_line(self) -> String {
        format!(
            "{} direct, {} stationary, {} retrograde, {} unknown",
            self.direct, self.stationary, self.retrograde, self.unknown
        )
    }
}

impl fmt::Display for MotionSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A summary of major aspect matches in a chart snapshot.
///
/// # Example
///
/// ```
/// use pleiades_core::AspectSummary;
///
/// let summary = AspectSummary {
///     conjunction: 0,
///     sextile: 1,
///     square: 0,
///     trine: 2,
///     opposition: 0,
/// };
///
/// assert_eq!(summary.summary_line(), "1 Sextile, 2 Trine");
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert!(summary.has_known_aspects());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AspectSummary {
    /// Placements matched by conjunction.
    pub conjunction: usize,
    /// Placements matched by sextile.
    pub sextile: usize,
    /// Placements matched by square.
    pub square: usize,
    /// Placements matched by trine.
    pub trine: usize,
    /// Placements matched by opposition.
    pub opposition: usize,
}

impl AspectSummary {
    fn from_matches(matches: &[AspectMatch]) -> Self {
        let mut summary = Self::default();

        for aspect in matches {
            summary.increment(aspect.kind);
        }

        summary
    }

    fn increment(&mut self, kind: AspectKind) {
        match kind {
            AspectKind::Conjunction => self.conjunction += 1,
            AspectKind::Sextile => self.sextile += 1,
            AspectKind::Square => self.square += 1,
            AspectKind::Trine => self.trine += 1,
            AspectKind::Opposition => self.opposition += 1,
        }
    }

    /// Returns `true` when the snapshot contains at least one major aspect match.
    pub fn has_known_aspects(self) -> bool {
        self.conjunction + self.sextile + self.square + self.trine + self.opposition > 0
    }

    /// Returns a compact one-line summary of the major aspect families in the snapshot.
    pub fn summary_line(self) -> String {
        let mut summary = String::new();
        for (count, aspect) in [
            (self.conjunction, AspectKind::Conjunction),
            (self.sextile, AspectKind::Sextile),
            (self.square, AspectKind::Square),
            (self.trine, AspectKind::Trine),
            (self.opposition, AspectKind::Opposition),
        ] {
            if count == 0 {
                continue;
            }
            if !summary.is_empty() {
                summary.push_str(", ");
            }
            summary.push_str(&count.to_string());
            summary.push(' ');
            summary.push_str(aspect.name());
        }

        if summary.is_empty() {
            summary.push_str("no major aspects");
        }

        summary
    }
}

impl fmt::Display for AspectSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A major aspect family in the chart façade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AspectKind {
    /// Bodies are aligned near the same longitude.
    Conjunction,
    /// Bodies are separated by roughly 60 degrees.
    Sextile,
    /// Bodies are separated by roughly 90 degrees.
    Square,
    /// Bodies are separated by roughly 120 degrees.
    Trine,
    /// Bodies are separated by roughly 180 degrees.
    Opposition,
}

impl AspectKind {
    /// Returns the canonical display name for the aspect family.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Conjunction => "Conjunction",
            Self::Sextile => "Sextile",
            Self::Square => "Square",
            Self::Trine => "Trine",
            Self::Opposition => "Opposition",
        }
    }
}

impl fmt::Display for AspectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Validation failure for a configurable aspect definition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AspectDefinitionValidationError {
    /// The exact aspect angle was not finite or fell outside the supported half-circle range.
    ExactDegreesOutOfRange { exact_degrees: f64 },
    /// The orb was not finite or fell outside the supported half-circle range.
    OrbDegreesOutOfRange { orb_degrees: f64 },
}

impl AspectDefinitionValidationError {
    /// Returns a compact one-line summary of the validation failure.
    pub fn summary_line(self) -> String {
        match self {
            Self::ExactDegreesOutOfRange { exact_degrees } => format!(
                "aspect definition exact_degrees must be finite and between 0 and 180 degrees; found {exact_degrees}"
            ),
            Self::OrbDegreesOutOfRange { orb_degrees } => format!(
                "aspect definition orb_degrees must be finite and between 0 and 180 degrees; found {orb_degrees}"
            ),
        }
    }
}

impl fmt::Display for AspectDefinitionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for AspectDefinitionValidationError {}

/// An aspect definition used for configurable matching.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectDefinition {
    /// The aspect family.
    pub kind: AspectKind,
    /// The exact angular separation that defines the aspect.
    pub exact_degrees: f64,
    /// The maximum allowed orb in degrees.
    pub orb_degrees: f64,
}

impl AspectDefinition {
    /// Creates a new aspect definition.
    pub const fn new(kind: AspectKind, exact_degrees: f64, orb_degrees: f64) -> Self {
        Self {
            kind,
            exact_degrees,
            orb_degrees,
        }
    }

    /// Returns a compact one-line rendering of the configured aspect.
    ///
    /// This keeps the configurable aspect vocabulary aligned with the other
    /// typed chart summaries while making the angle parameters easy to audit.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::{AspectDefinition, AspectKind};
    ///
    /// let definition = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
    /// assert_eq!(definition.summary_line(), "kind=Sextile; exact_degrees=60°; orb_degrees=4°");
    /// assert_eq!(definition.to_string(), definition.summary_line());
    /// ```
    pub fn summary_line(self) -> String {
        format!(
            "kind={}; exact_degrees={}°; orb_degrees={}°",
            self.kind, self.exact_degrees, self.orb_degrees
        )
    }

    /// Validates that the configured exact angle and orb remain finite and within range.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::{AspectDefinition, AspectKind};
    ///
    /// let definition = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
    /// assert!(definition.validate().is_ok());
    /// ```
    pub fn validate(self) -> Result<(), AspectDefinitionValidationError> {
        if !self.exact_degrees.is_finite() || !(0.0..=180.0).contains(&self.exact_degrees) {
            return Err(AspectDefinitionValidationError::ExactDegreesOutOfRange {
                exact_degrees: self.exact_degrees,
            });
        }

        if !self.orb_degrees.is_finite() || !(0.0..=180.0).contains(&self.orb_degrees) {
            return Err(AspectDefinitionValidationError::OrbDegreesOutOfRange {
                orb_degrees: self.orb_degrees,
            });
        }

        Ok(())
    }
}

impl fmt::Display for AspectDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validates a slice of aspect definitions.
///
/// # Example
///
/// ```
/// use pleiades_core::{validate_aspect_definitions, AspectDefinition, AspectKind};
///
/// let definitions = [
///     AspectDefinition::new(AspectKind::Conjunction, 0.0, 8.0),
///     AspectDefinition::new(AspectKind::Opposition, 180.0, 8.0),
/// ];
///
/// assert!(validate_aspect_definitions(&definitions).is_ok());
/// ```
pub fn validate_aspect_definitions(
    definitions: &[AspectDefinition],
) -> Result<(), AspectDefinitionValidationError> {
    definitions
        .iter()
        .copied()
        .try_for_each(AspectDefinition::validate)
}

/// A matched aspect between two bodies.
#[derive(Clone, Debug, PartialEq)]
pub struct AspectMatch {
    /// The body on the left side of the pairing.
    pub left: CelestialBody,
    /// The body on the right side of the pairing.
    pub right: CelestialBody,
    /// The matched aspect family.
    pub kind: AspectKind,
    /// The measured angular separation between the bodies.
    pub separation: Angle,
    /// The absolute orb from the exact aspect angle.
    pub orb: Angle,
}

impl fmt::Display for AspectMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} (separation: {}, orb: {})",
            self.left, self.kind, self.right, self.separation, self.orb
        )
    }
}

impl fmt::Display for ChartSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Backend: {}", self.backend_id)?;
        writeln!(
            f,
            "Instant: {} ({})",
            self.instant.julian_day, self.instant.scale
        )?;
        writeln!(
            f,
            "Time-scale policy: caller-supplied instant scale; no built-in Delta T or relativistic model"
        )?;
        if let Some(observer) = &self.observer {
            writeln!(
                f,
                "Observer: {} / {}",
                observer.latitude, observer.longitude
            )?;
            if self.houses.is_some() {
                writeln!(
                    f,
                    "Observer policy: used for houses only; body positions remain geocentric"
                )?;
            } else {
                writeln!(
                    f,
                    "Observer policy: geocentric body positions; no house observer supplied"
                )?;
            }
        } else {
            writeln!(
                f,
                "Observer policy: geocentric body positions; no house observer supplied"
            )?;
        }
        writeln!(
            f,
            "Frame policy: ecliptic body positions are requested from the backend; house calculations remain separate"
        )?;
        writeln!(f, "Zodiac mode: {}", self.zodiac_mode)?;
        writeln!(f, "Apparentness: {}", self.apparentness)?;
        writeln!(
            f,
            "Apparentness policy: {}",
            match self.apparentness {
                Apparentness::Mean => {
                    "mean geometric request by default; apparent corrections require backend support"
                }
                Apparentness::Apparent => {
                    "apparent request explicitly requested; backend support required"
                }
                _ => "requested apparentness is backend-dependent",
            }
        )?;
        if let Some(houses) = &self.houses {
            let house_name = crate::house_system_descriptor(&houses.system)
                .map(|descriptor| descriptor.canonical_name)
                .unwrap_or("Custom");
            writeln!(f, "House system: {}", house_name)?;
            writeln!(f, "House cusps:")?;
            for (index, cusp) in houses.cusps.iter().enumerate() {
                writeln!(f, "  - {:>2}: {}", index + 1, cusp)?;
            }
        }
        writeln!(f, "Bodies:")?;
        for placement in &self.placements {
            writeln!(f, "  - {}", placement.summary_line())?;
        }

        let sign_summary = self.sign_summary();
        if sign_summary.has_known_signs() {
            writeln!(f, "Sign summary: {}", sign_summary)?;
        }

        let dominant_sign_summary = self.dominant_sign_summary();
        if dominant_sign_summary.has_known_signs() && dominant_sign_summary != sign_summary {
            writeln!(f, "Dominant sign summary: {}", dominant_sign_summary)?;
        }

        let house_summary = self.house_summary();
        if house_summary.has_known_houses() {
            writeln!(f, "House summary: {}", house_summary)?;
        }

        let dominant_house_summary = self.dominant_house_summary();
        if dominant_house_summary.has_known_houses() && dominant_house_summary != house_summary {
            writeln!(f, "Dominant house summary: {}", dominant_house_summary)?;
        }

        let motion_summary = self.motion_summary();
        if motion_summary.has_known_motion() || motion_summary.unknown > 0 {
            match motion_summary.validate(self.placements.len()) {
                Ok(()) => writeln!(f, "Motion summary: {}", motion_summary)?,
                Err(error) => writeln!(f, "Motion summary unavailable ({error})")?,
            }
        }

        let stationary_bodies: Vec<String> = self
            .stationary_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !stationary_bodies.is_empty() {
            writeln!(f, "Stationary bodies: {}", stationary_bodies.join(", "))?;
        }

        let unknown_motion_bodies: Vec<String> = self
            .unknown_motion_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !unknown_motion_bodies.is_empty() {
            writeln!(
                f,
                "Unknown motion bodies: {}",
                unknown_motion_bodies.join(", ")
            )?;
        }

        let retrograde_bodies: Vec<String> = self
            .retrograde_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !retrograde_bodies.is_empty() {
            writeln!(f, "Retrograde bodies: {}", retrograde_bodies.join(", "))?;
        }

        let aspects = self.major_aspects();
        let aspect_summary = AspectSummary::from_matches(&aspects);
        if aspect_summary.has_known_aspects() {
            writeln!(f, "Aspect summary: {}", aspect_summary)?;
        }
        if !aspects.is_empty() {
            writeln!(f, "Aspects:")?;
            for aspect in aspects {
                writeln!(f, "  - {}", aspect)?;
            }
        }

        Ok(())
    }
}

const DEFAULT_ASPECT_DEFINITIONS: [AspectDefinition; 5] = [
    AspectDefinition::new(AspectKind::Conjunction, 0.0, 8.0),
    AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0),
    AspectDefinition::new(AspectKind::Square, 90.0, 6.0),
    AspectDefinition::new(AspectKind::Trine, 120.0, 6.0),
    AspectDefinition::new(AspectKind::Opposition, 180.0, 8.0),
];

fn default_aspect_definitions() -> &'static [AspectDefinition] {
    &DEFAULT_ASPECT_DEFINITIONS
}

fn angle_separation(left: Longitude, right: Longitude) -> Angle {
    let difference = (right.degrees() - left.degrees()).rem_euclid(360.0);
    let separation = if difference > 180.0 {
        360.0 - difference
    } else {
        difference
    };
    Angle::from_degrees(separation)
}

fn house_ordinal(house: usize) -> &'static str {
    match house {
        1 => "1st",
        2 => "2nd",
        3 => "3rd",
        4 => "4th",
        5 => "5th",
        6 => "6th",
        7 => "7th",
        8 => "8th",
        9 => "9th",
        10 => "10th",
        11 => "11th",
        12 => "12th",
        _ => "unknown",
    }
}

fn best_aspect_definition(
    separation: Angle,
    definitions: &[AspectDefinition],
) -> Option<AspectDefinition> {
    definitions
        .iter()
        .copied()
        .filter(|definition| {
            (separation.degrees() - definition.exact_degrees).abs() <= definition.orb_degrees
        })
        .min_by(|left, right| {
            let left_orb = (separation.degrees() - left.exact_degrees).abs();
            let right_orb = (separation.degrees() - right.exact_degrees).abs();
            left_orb
                .partial_cmp(&right_orb)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
}

/// Converts a tropical longitude into the requested zodiac mode.
///
/// Tropical mode returns the input unchanged. Sidereal mode subtracts the
/// resolved ayanamsa for the provided instant.
///
/// # Example
///
/// ```
/// use pleiades_core::sidereal_longitude;
/// use pleiades_types::{Ayanamsa, Instant, JulianDay, Longitude, TimeScale, ZodiacMode, ZodiacSign};
///
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let tropical = Longitude::from_degrees(15.0);
/// let sidereal = sidereal_longitude(
///     tropical,
///     instant,
///     &ZodiacMode::Sidereal {
///         ayanamsa: Ayanamsa::Lahiri,
///     },
/// )
/// .expect("Lahiri sidereal conversion should work");
///
/// assert_eq!(ZodiacSign::from_longitude(sidereal), ZodiacSign::Pisces);
/// ```
pub fn sidereal_longitude(
    longitude: Longitude,
    instant: Instant,
    zodiac_mode: &ZodiacMode,
) -> Result<Longitude, EphemerisError> {
    match zodiac_mode {
        ZodiacMode::Tropical => Ok(longitude),
        ZodiacMode::Sidereal { ayanamsa } => {
            let offset = sidereal_offset(ayanamsa, instant).ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "sidereal conversion requires an ayanamsa with reference offset metadata",
                )
            })?;
            Ok(Longitude::from_degrees(
                longitude.degrees() - offset.degrees(),
            ))
        }
        _ => Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "unsupported zodiac mode",
        )),
    }
}

fn map_observer_location_error(error: ObserverLocationValidationError) -> EphemerisError {
    EphemerisError::new(EphemerisErrorKind::InvalidObserver, error.to_string())
}

fn map_house_error(error: pleiades_houses::HouseError) -> EphemerisError {
    let kind = match error.kind {
        pleiades_houses::HouseErrorKind::UnsupportedHouseSystem => {
            EphemerisErrorKind::InvalidRequest
        }
        pleiades_houses::HouseErrorKind::InvalidLatitude
        | pleiades_houses::HouseErrorKind::InvalidLongitude
        | pleiades_houses::HouseErrorKind::InvalidElevation => EphemerisErrorKind::InvalidObserver,
        pleiades_houses::HouseErrorKind::InvalidObliquity => EphemerisErrorKind::InvalidRequest,
        pleiades_houses::HouseErrorKind::NumericalFailure => EphemerisErrorKind::NumericalFailure,
        _ => EphemerisErrorKind::InvalidRequest,
    };

    EphemerisError::new(kind, error.message)
}

fn map_custom_definition_error(
    subject: &'static str,
    error: CustomDefinitionValidationError,
) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("{subject} is invalid: {error}"),
    )
}

impl<B: EphemerisBackend> ChartEngine<B> {
    /// Assembles a basic chart snapshot from the backend.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::{
    ///     AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId,
    ///     BackendMetadata, BackendProvenance, EphemerisBackend, EphemerisError, EphemerisRequest,
    ///     EphemerisResult, QualityAnnotation, TimeRange,
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
    ///             body_coverage: vec![CelestialBody::Sun],
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
    /// assert_eq!(snapshot.apparentness, Apparentness::Mean);
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
                frame: pleiades_types::CoordinateFrame::Ecliptic,
                zodiac_mode: backend_zodiac_mode.clone(),
                apparent: request.apparentness,
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
                    Some(ZodiacSign::from_longitude(*longitude))
                } else {
                    position
                        .ecliptic
                        .as_ref()
                        .map(|coords| ZodiacSign::from_longitude(coords.longitude))
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
                Ok(BodyPlacement {
                    body,
                    position,
                    sign,
                    house,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use pleiades_backend::{
        AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
        BackendProvenance, QualityAnnotation,
    };
    use pleiades_types::{EclipticCoordinates, Latitude, Longitude, ObserverLocation, TimeScale};

    struct ToyChartBackend;

    #[derive(Clone)]
    struct RecordingChartBackend {
        observers: Arc<Mutex<Vec<Option<ObserverLocation>>>>,
    }

    #[derive(Clone)]
    struct MeanOnlyRecordingChartBackend {
        observers: Arc<Mutex<Vec<Option<ObserverLocation>>>>,
    }

    impl EphemerisBackend for ToyChartBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("toy-chart"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("toy chart backend"),
                nominal_range: pleiades_types::TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
                supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        }

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            if request.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidObserver,
                    "toy chart backend is geocentric only",
                ));
            }

            let longitude = match request.body {
                CelestialBody::Sun => Longitude::from_degrees(15.0),
                CelestialBody::Moon => Longitude::from_degrees(45.0),
                _ => {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::UnsupportedBody,
                        "unsupported",
                    ))
                }
            };
            let mut result = EphemerisResult::new(
                BackendId::new("toy-chart"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            );
            result.quality = QualityAnnotation::Approximate;
            result.ecliptic = Some(EclipticCoordinates::new(
                longitude,
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }
    }

    impl EphemerisBackend for RecordingChartBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("recording-chart"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("recording chart backend"),
                nominal_range: pleiades_types::TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    topocentric: true,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            matches!(body, CelestialBody::Sun)
        }

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.observers
                .lock()
                .expect("observer log should be lockable")
                .push(request.observer.clone());
            let mut result = EphemerisResult::new(
                BackendId::new("recording-chart"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            );
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(15.0),
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }
    }

    impl EphemerisBackend for MeanOnlyRecordingChartBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("mean-only-recording-chart"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("mean-only recording chart backend"),
                nominal_range: pleiades_types::TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    apparent: false,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            matches!(body, CelestialBody::Sun)
        }

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.observers
                .lock()
                .expect("observer log should be lockable")
                .push(request.observer.clone());
            let mut result = EphemerisResult::new(
                BackendId::new("mean-only-recording-chart"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            );
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(15.0),
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }
    }

    #[derive(Clone)]
    struct BatchRecordingChartBackend {
        observers: Arc<Mutex<Vec<Option<ObserverLocation>>>>,
        batch_calls: Arc<Mutex<usize>>,
    }

    impl EphemerisBackend for BatchRecordingChartBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("batch-recording-chart"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("batch recording chart backend"),
                nominal_range: pleiades_types::TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
                supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: true,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        }

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.observers
                .lock()
                .expect("observer log should be lockable")
                .push(request.observer.clone());
            let longitude = match request.body {
                CelestialBody::Sun => Longitude::from_degrees(15.0),
                CelestialBody::Moon => Longitude::from_degrees(45.0),
                _ => {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::UnsupportedBody,
                        "unsupported",
                    ))
                }
            };
            let mut result = EphemerisResult::new(
                BackendId::new("batch-recording-chart"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            );
            result.quality = QualityAnnotation::Approximate;
            result.ecliptic = Some(EclipticCoordinates::new(
                longitude,
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }

        fn positions(
            &self,
            reqs: &[EphemerisRequest],
        ) -> Result<Vec<EphemerisResult>, EphemerisError> {
            *self
                .batch_calls
                .lock()
                .expect("batch call log should be lockable") += 1;
            reqs.iter().map(|req| self.position(req)).collect()
        }
    }

    #[test]
    fn default_body_list_contains_the_luminaries() {
        let bodies = default_chart_bodies();
        assert!(bodies.contains(&CelestialBody::Sun));
        assert!(bodies.contains(&CelestialBody::Moon));
    }

    #[test]
    fn sidereal_longitude_applies_ayanamsa() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let sidereal = sidereal_longitude(
            Longitude::from_degrees(5.0),
            instant,
            &ZodiacMode::Sidereal {
                ayanamsa: crate::Ayanamsa::Lahiri,
            },
        )
        .expect("sidereal conversion should work");

        assert_eq!(ZodiacSign::from_longitude(sidereal), ZodiacSign::Pisces);
    }

    #[test]
    fn chart_snapshot_assigns_signs() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

        let chart = engine.chart(&request).expect("chart should render");
        assert_eq!(chart.backend_id.as_str(), "toy-chart");
        assert_eq!(chart.len(), 2);
        assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Aries));
        assert_eq!(chart.placements[1].sign, Some(ZodiacSign::Taurus));
        assert_eq!(chart.sign_summary().aries, 1);
        assert_eq!(chart.sign_summary().taurus, 1);
        let rendered = chart.to_string();
        assert!(rendered.contains("Sun"));
        assert!(rendered.contains("Moon"));
        assert!(rendered.contains("Sign summary: 1 Aries, 1 Taurus"));
        assert!(rendered.contains("Instant: JD 2451545 (TT)"));
        assert!(rendered.contains(
            "Frame policy: ecliptic body positions are requested from the backend; house calculations remain separate"
        ));
        assert!(rendered.contains("Apparentness policy: mean geometric request by default; apparent corrections require backend support"));
    }

    #[test]
    fn chart_snapshot_without_observer_renders_geocentric_policy_line() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            body_observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Mean,
            houses: None,
            placements: vec![],
        };

        let rendered = chart.to_string();
        assert_eq!(
            chart.summary_line(),
            "backend=toy-chart; instant=JD 2451545 (TT); placements=0; zodiac=Tropical; apparentness=Mean; observer=geocentric; observer location=none; body observer=none; house system=none; house cusps=0"
        );
        assert!(rendered
            .contains("Observer policy: geocentric body positions; no house observer supplied"));
        assert!(!rendered
            .contains("Observer policy: used for houses only; body positions remain geocentric"));
    }

    #[test]
    fn chart_snapshot_exposes_dominant_sign_and_house_summaries() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            body_observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Apparent,
            houses: None,
            placements: vec![
                BodyPlacement {
                    body: CelestialBody::Sun,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Sun,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Moon,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Moon,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Mars,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Mars,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
                BodyPlacement {
                    body: CelestialBody::Mercury,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Mercury,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
                BodyPlacement {
                    body: CelestialBody::Jupiter,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Jupiter,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Gemini),
                    house: Some(8),
                },
            ],
        };

        assert_eq!(
            chart.dominant_sign_summary(),
            SignSummary {
                aries: 2,
                taurus: 2,
                gemini: 0,
                cancer: 0,
                leo: 0,
                virgo: 0,
                libra: 0,
                scorpio: 0,
                sagittarius: 0,
                capricorn: 0,
                aquarius: 0,
                pisces: 0,
            }
        );
        assert_eq!(
            chart.dominant_house_summary(),
            HouseSummary {
                first: 2,
                second: 2,
                third: 0,
                fourth: 0,
                fifth: 0,
                sixth: 0,
                seventh: 0,
                eighth: 0,
                ninth: 0,
                tenth: 0,
                eleventh: 0,
                twelfth: 0,
                unknown: 0,
            }
        );

        let rendered = chart.to_string();
        assert!(rendered.contains("Dominant sign summary: 2 Aries, 2 Taurus"));
        assert!(rendered.contains("Dominant house summary: 2 in 1st house, 2 in 2nd house"));
    }

    #[test]
    fn chart_snapshot_supports_sidereal_signs() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun])
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        });

        let chart = engine
            .chart(&request)
            .expect("sidereal chart should render");
        assert_eq!(chart.zodiac_mode, request.zodiac_mode);
        assert_eq!(
            chart.placements[0].position.zodiac_mode,
            request.zodiac_mode
        );
        assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Pisces));
        let rendered = chart.to_string();
        assert!(rendered.contains("Zodiac mode: Sidereal (Lahiri)"));
    }

    #[test]
    fn chart_snapshot_preserves_apparentness_choice() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun])
        .with_apparentness(Apparentness::Apparent);

        let chart = engine
            .chart(&request)
            .expect("apparent chart should render");
        assert_eq!(
            chart.placements[0].position.apparent,
            Apparentness::Apparent
        );
        assert_eq!(chart.apparentness, Apparentness::Apparent);
        let rendered = chart.to_string();
        assert!(rendered.contains("Apparentness: Apparent"));
        assert!(rendered.contains(
            "Apparentness policy: apparent request explicitly requested; backend support required"
        ));
        assert!(rendered.contains("Time-scale policy: caller-supplied instant scale; no built-in Delta T or relativistic model"));
    }

    #[test]
    fn chart_snapshot_supports_house_placement() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_observer(pleiades_types::ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            None,
        ))
        .with_house_system(crate::HouseSystem::WholeSign)
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

        let chart = engine
            .chart(&request)
            .expect("house-aware chart should render");
        assert!(chart.houses.is_some());
        assert_eq!(chart.placements.len(), 2);
        assert!(chart
            .placements
            .iter()
            .all(|placement| placement.house.is_some()));
        let rendered = chart.to_string();
        assert_eq!(chart.observer_policy(), ObserverPolicy::HouseOnly);
        assert_eq!(
            chart.summary_line(),
            "backend=toy-chart; instant=JD 2451545 (TT); placements=2; zodiac=Tropical; apparentness=Mean; observer=house-only; observer location=latitude=0°, longitude=0°, elevation=n/a; body observer=none; house system=Whole Sign; house cusps=12"
        );
        assert!(rendered.contains("House system: Whole Sign"));
        assert!(rendered
            .contains("Observer policy: used for houses only; body positions remain geocentric"));
    }

    #[test]
    fn chart_snapshot_keeps_house_observer_out_of_body_position_requests() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(125.0),
        ))
        .with_house_system(crate::HouseSystem::WholeSign)
        .with_bodies(vec![CelestialBody::Sun]);

        let chart = engine
            .chart(&request)
            .expect("chart should render with a house observer");

        assert!(chart.houses.is_some());
        assert_eq!(chart.observer, request.observer);
        let observers = observers.lock().expect("observer log should be lockable");
        assert_eq!(observers.len(), 1);
        assert!(observers.iter().all(Option::is_none));
    }

    #[test]
    fn chart_snapshot_passes_body_observers_through_topocentric_requests() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let body_observer = ObserverLocation::new(
            Latitude::from_degrees(35.0),
            Longitude::from_degrees(-80.0),
            Some(50.0),
        );
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_body_observer(body_observer.clone())
        .with_bodies(vec![CelestialBody::Sun]);

        let chart = engine
            .chart(&request)
            .expect("chart should render with a topocentric body observer");

        assert_eq!(chart.body_observer, request.body_observer);
        assert_eq!(chart.observer, request.observer);
        assert!(chart.summary_line().contains("body observer=latitude=35°"));
        let observers = observers.lock().expect("observer log should be lockable");
        assert_eq!(observers.as_slice(), &[Some(body_observer)]);
    }

    #[test]
    fn chart_snapshot_rejects_unsupported_body_before_backend_position() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Moon]);

        let error = engine
            .chart(&request)
            .expect_err("chart should reject unsupported body coverage before dispatch");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        assert!(error
            .message
            .contains("recording-chart does not support Moon"));
        let observers = observers.lock().expect("observer log should be lockable");
        assert!(observers.is_empty());
    }

    #[test]
    fn chart_snapshot_with_observer_but_without_houses_stays_geocentric() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            None,
        ))
        .with_bodies(vec![CelestialBody::Sun]);

        let chart = engine
            .chart(&request)
            .expect("chart should render without house calculations");

        assert!(chart.houses.is_none());
        assert_eq!(chart.observer, request.observer);
        assert_eq!(chart.observer_policy(), ObserverPolicy::Geocentric);
        assert_eq!(
            chart.summary_line(),
            "backend=recording-chart; instant=JD 2451545 (TT); placements=1; zodiac=Tropical; apparentness=Mean; observer=geocentric; observer location=latitude=51.5°, longitude=359.9°, elevation=n/a; body observer=none; house system=none; house cusps=0"
        );
        assert!(chart
            .to_string()
            .contains("Observer policy: geocentric body positions; no house observer supplied"));
        let observers = observers.lock().expect("observer log should be lockable");
        assert_eq!(observers.len(), 1);
        assert!(observers.iter().all(Option::is_none));
    }

    #[test]
    fn chart_snapshot_rejects_non_finite_house_observer_elevation() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(f64::NAN),
        ))
        .with_house_system(crate::HouseSystem::Topocentric)
        .with_bodies(vec![CelestialBody::Sun]);

        let error = engine
            .chart(&request)
            .expect_err("chart should reject non-finite house observer elevation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error.message.contains("observer elevation must be finite"));
        let observers = observers.lock().expect("observer log should be lockable");
        assert!(observers.is_empty());
    }

    #[test]
    fn chart_snapshot_rejects_non_finite_house_observer_longitude() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(f64::NAN),
            None,
        ))
        .with_house_system(crate::HouseSystem::Equal)
        .with_bodies(vec![CelestialBody::Sun]);

        let error = engine
            .chart(&request)
            .expect_err("chart should reject non-finite house observer longitude");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error.message.contains("observer longitude must be finite"));
        let observers = observers.lock().expect("observer log should be lockable");
        assert!(observers.is_empty());
    }

    #[test]
    fn chart_snapshot_uses_backend_batch_queries_for_body_positions() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let batch_calls = Arc::new(Mutex::new(0));
        let engine = ChartEngine::new(BatchRecordingChartBackend {
            observers: Arc::clone(&observers),
            batch_calls: Arc::clone(&batch_calls),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

        let chart = engine
            .chart(&request)
            .expect("chart should render through the backend batch path");

        assert_eq!(chart.placements.len(), 2);
        assert_eq!(chart.placements[0].body, CelestialBody::Sun);
        assert_eq!(chart.placements[1].body, CelestialBody::Moon);
        assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Aries));
        assert_eq!(chart.placements[1].sign, Some(ZodiacSign::Taurus));
        assert_eq!(
            *batch_calls
                .lock()
                .expect("batch call log should be lockable"),
            1
        );
        let observers = observers.lock().expect("observer log should be lockable");
        assert_eq!(observers.len(), 2);
        assert!(observers.iter().all(Option::is_none));
    }

    #[test]
    fn chart_request_summary_line_reflects_the_default_request_shape() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ));

        assert_eq!(request.observer_policy(), ObserverPolicy::Geocentric);
        assert_eq!(
            request.summary_line(),
            "instant=JD 2451545 (TT); bodies=10; zodiac=Tropical; apparentness=Mean; observer=geocentric; observer location=none; body observer=none; house system=none"
        );
        assert_eq!(request.to_string(), request.summary_line());
    }

    #[test]
    fn observer_summary_renders_the_policy_location_and_body_location() {
        let summary = ObserverSummary::new(
            ObserverPolicy::HouseOnly,
            Some(ObserverLocation::new(
                Latitude::from_degrees(12.5),
                Longitude::from_degrees(45.0),
                Some(100.0),
            )),
        )
        .with_body_location(Some(ObserverLocation::new(
            Latitude::from_degrees(-33.9),
            Longitude::from_degrees(151.2),
            None,
        )));

        assert_eq!(summary.policy, ObserverPolicy::HouseOnly);
        assert_eq!(
            summary.location_label(),
            "latitude=12.5°, longitude=45°, elevation=100.000 m"
        );
        assert_eq!(
            summary.body_location_label(),
            "latitude=-33.9°, longitude=151.2°, elevation=n/a"
        );
        assert_eq!(
            summary
                .validated_location_label()
                .expect("valid observer summary location"),
            "latitude=12.5°, longitude=45°, elevation=100.000 m"
        );
        assert_eq!(
            summary
                .validated_summary_line()
                .expect("valid observer summary"),
            summary.summary_line()
        );
        assert_eq!(
            summary.summary_line(),
            "observer=house-only; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=latitude=-33.9°, longitude=151.2°, elevation=n/a"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
    }

    #[test]
    fn observer_summary_validate_rejects_house_only_without_location() {
        let summary = ObserverSummary::new(ObserverPolicy::HouseOnly, None);

        let error = summary
            .validate()
            .expect_err("house-only summaries should require an observer location");
        assert_eq!(
            error,
            ObserverSummaryValidationError::HouseOnlyMissingObserver
        );
        assert_eq!(
            error.to_string(),
            "observer summary for house-only posture requires an observer location"
        );
    }

    #[test]
    fn observer_summary_validate_rejects_invalid_locations() {
        let summary = ObserverSummary::new(
            ObserverPolicy::Geocentric,
            Some(ObserverLocation::new(
                Latitude::from_degrees(12.5),
                Longitude::from_degrees(f64::NAN),
                Some(100.0),
            )),
        );

        let error = summary
            .validate()
            .expect_err("observer summaries should reject invalid locations");
        assert!(matches!(
            error,
            ObserverSummaryValidationError::InvalidObserverLocation(
                ObserverLocationValidationError::NonFiniteLongitude { value }
            ) if value.is_nan()
        ));
        assert!(error
            .to_string()
            .contains("observer summary location is invalid: observer longitude must be finite"));
    }

    #[test]
    fn observer_summary_validate_rejects_invalid_body_locations() {
        let summary = ObserverSummary::new(ObserverPolicy::Geocentric, None).with_body_location(
            Some(ObserverLocation::new(
                Latitude::from_degrees(12.5),
                Longitude::from_degrees(f64::NAN),
                Some(100.0),
            )),
        );

        let error = summary
            .validate()
            .expect_err("observer summaries should reject invalid body locations");
        assert!(matches!(
            error,
            ObserverSummaryValidationError::InvalidBodyPosition(
                ObserverLocationValidationError::NonFiniteLongitude { value }
            ) if value.is_nan()
        ));
        assert!(error.to_string().contains(
            "observer summary body location is invalid: observer longitude must be finite"
        ));
    }

    #[test]
    fn chart_request_observer_summary_matches_the_rendered_policy_line() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(100.0),
        ))
        .with_house_system(crate::HouseSystem::WholeSign);

        let observer = request.observer_summary();

        assert_eq!(observer.policy, ObserverPolicy::HouseOnly);
        assert_eq!(
            observer.summary_line(),
            "observer=house-only; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none"
        );
        assert!(request
            .summary_line()
            .contains(observer.summary_line().as_str()));
    }

    #[test]
    fn chart_snapshot_observer_summary_matches_the_rendered_policy_line() {
        let snapshot = ChartSnapshot {
            backend_id: crate::BackendId::new("demo"),
            instant: Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ),
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(12.5),
                Longitude::from_degrees(45.0),
                Some(100.0),
            )),
            body_observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Mean,
            houses: None,
            placements: Vec::new(),
        };

        let observer = snapshot.observer_summary();

        assert_eq!(observer.policy, ObserverPolicy::Geocentric);
        assert_eq!(
            observer.summary_line(),
            "observer=geocentric; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none"
        );
        assert!(snapshot
            .summary_line()
            .contains(observer.summary_line().as_str()));
    }

    #[test]
    fn chart_request_validated_summary_line_rejects_invalid_observer_locations() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(f64::NAN),
            Some(100.0),
        ));

        let error = request
            .validated_summary_line()
            .expect_err("validated request summaries should reject invalid locations");
        assert!(matches!(
            error,
            ObserverSummaryValidationError::InvalidObserverLocation(
                ObserverLocationValidationError::NonFiniteLongitude { value }
            ) if value.is_nan()
        ));
    }

    #[test]
    fn chart_snapshot_validated_summary_line_rejects_invalid_observer_locations() {
        let snapshot = ChartSnapshot {
            backend_id: crate::BackendId::new("demo"),
            instant: Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ),
            observer: Some(ObserverLocation::new(
                Latitude::from_degrees(12.5),
                Longitude::from_degrees(f64::NAN),
                Some(100.0),
            )),
            body_observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Mean,
            houses: None,
            placements: Vec::new(),
        };

        let error = snapshot
            .validated_summary_line()
            .expect_err("validated snapshot summaries should reject invalid locations");
        assert!(matches!(
            error,
            ObserverSummaryValidationError::InvalidObserverLocation(
                ObserverLocationValidationError::NonFiniteLongitude { value }
            ) if value.is_nan()
        ));
    }

    #[test]
    fn chart_request_summary_line_reflects_observer_and_house_system_policy() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(100.0),
        ))
        .with_house_system(crate::HouseSystem::WholeSign)
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        })
        .with_apparentness(Apparentness::Apparent);

        assert_eq!(request.observer_policy(), ObserverPolicy::HouseOnly);
        assert_eq!(request.observer_summary().policy, ObserverPolicy::HouseOnly);
        assert_eq!(ObserverPolicy::HouseOnly.summary_line(), "house-only");
        assert_eq!(
            request.summary_line(),
            "instant=JD 2451545 (UTC); bodies=2; zodiac=Sidereal (Lahiri); apparentness=Apparent; observer=house-only; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none; house system=Whole Sign"
        );
    }

    #[test]
    fn chart_request_summary_line_stays_geocentric_without_a_house_system() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(100.0),
        ))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

        assert_eq!(request.observer_policy(), ObserverPolicy::Geocentric);
        assert_eq!(
            request.summary_line(),
            "instant=JD 2451545 (TT); bodies=2; zodiac=Tropical; apparentness=Mean; observer=geocentric; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none; house system=none"
        );
    }

    #[test]
    fn chart_request_summary_line_preserves_custom_house_system_details() {
        let mut custom = pleiades_types::CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("My Alias".to_string());
        custom.notes = Some("uses a local calibration".to_string());

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_house_system(crate::HouseSystem::Custom(custom));

        assert_eq!(
            request.summary_line(),
            "instant=JD 2451545 (TT); bodies=10; zodiac=Tropical; apparentness=Mean; observer=geocentric; observer location=none; body observer=none; house system=My Custom Houses [aliases: My Alias] (uses a local calibration)"
        );
    }

    #[test]
    fn chart_request_validation_rejects_invalid_custom_definitions_before_backend_dispatch() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let mut custom = pleiades_types::CustomHouseSystem::new("   ");
        custom.aliases.push("MCH".to_string());

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            None,
        ))
        .with_house_system(crate::HouseSystem::Custom(custom));

        let validation_error = engine
            .validate_chart_request(&request)
            .expect_err("blank custom house systems should be rejected before metadata checks");
        assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(validation_error
            .message
            .contains("chart house system is invalid: custom house system name must not be blank"));

        let chart_error = engine
            .chart(&request)
            .expect_err("blank custom house systems should be rejected before backend dispatch");
        assert_eq!(chart_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(chart_error
            .message
            .contains("chart house system is invalid: custom house system name must not be blank"));
        assert!(observers
            .lock()
            .expect("observer log should be lockable")
            .is_empty());
    }

    #[test]
    fn chart_request_validation_rejects_custom_definitions_that_collide_with_builtins() {
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::new(Mutex::new(Vec::new())),
        });

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            None,
        ))
        .with_house_system(crate::HouseSystem::Custom(
            pleiades_types::CustomHouseSystem::new("Equal"),
        ));

        let validation_error = engine
            .validate_chart_request(&request)
            .expect_err("built-in house names should be rejected for custom house systems");
        assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(validation_error
            .message
            .contains("chart house system is invalid: custom house system name must not match a built-in label: Equal"));

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            None,
        ))
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Custom(pleiades_types::CustomAyanamsa::new("Lahiri")),
        });

        let validation_error = engine
            .validate_chart_request(&request)
            .expect_err("built-in ayanamsa names should be rejected for custom ayanamsas");
        assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(validation_error
            .message
            .contains("sidereal ayanamsa is invalid: custom ayanamsa name must not match a built-in label: Lahiri"));
    }

    #[test]
    fn chart_request_validation_rejects_apparent_requests_before_backend_dispatch() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(MeanOnlyRecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun])
        .with_apparentness(Apparentness::Apparent);

        let validation_error = engine
            .validate_chart_request(&request)
            .expect_err("apparent chart requests should be rejected before backend dispatch");
        assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(validation_error
            .message
            .contains("currently returns mean geometric coordinates only; apparent corrections are not implemented"));

        let chart_error = engine
            .chart(&request)
            .expect_err("apparent chart requests should be rejected before backend dispatch");
        assert_eq!(chart_error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(chart_error
            .message
            .contains("currently returns mean geometric coordinates only; apparent corrections are not implemented"));
        assert!(observers
            .lock()
            .expect("observer log should be lockable")
            .is_empty());
    }

    #[test]
    fn chart_request_validation_rejects_house_requests_without_observers_before_backend_dispatch() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun])
        .with_house_system(crate::HouseSystem::WholeSign);

        let validation_error = engine
            .validate_chart_request(&request)
            .expect_err("house requests should require an observer before backend dispatch");
        assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            validation_error.message,
            "house placement requires an observer location"
        );

        let chart_error = engine
            .chart(&request)
            .expect_err("house requests should require an observer before backend dispatch");
        assert_eq!(chart_error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            chart_error.message,
            "house placement requires an observer location"
        );
        assert!(observers
            .lock()
            .expect("observer log should be lockable")
            .is_empty());
    }

    #[test]
    fn chart_request_can_apply_a_caller_supplied_time_scale_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_instant_time_scale_offset(TimeScale::Tt, 64.184);

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_apply_a_checked_caller_supplied_time_scale_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_instant_time_scale_offset_checked(TimeScale::Tt, 64.184)
        .expect("UT1 chart request should accept a checked caller-supplied offset");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_apply_a_caller_supplied_time_scale_conversion_policy() {
        let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tdb, 64.184);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_time_scale_conversion(policy)
        .expect("UT1 chart request should accept a caller-supplied policy");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
        assert_eq!(
            policy.summary_line(),
            "source=UT1; target=TDB; offset_seconds=64.184 s"
        );
    }

    #[test]
    fn chart_request_caller_supplied_conversion_preserves_custom_house_metadata() {
        let mut custom = pleiades_types::CustomHouseSystem::new("My UTC Custom Houses");
        custom.aliases.push("My UTC Alias".to_string());
        custom.notes = Some("uses a local UTC calibration".to_string());

        let observer = ObserverLocation::new(
            Latitude::from_degrees(34.5),
            Longitude::from_degrees(-118.25),
            Some(75.0),
        );

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_observer(observer)
        .with_house_system(HouseSystem::Custom(custom))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        })
        .with_apparentness(Apparentness::Apparent);

        let converted = request
            .clone()
            .with_time_scale_conversion(TimeScaleConversion::new(
                TimeScale::Utc,
                TimeScale::Tdb,
                64.184,
            ))
            .expect("UTC chart request should accept a caller-supplied policy");

        assert_eq!(converted.instant.scale, TimeScale::Tdb);
        assert_eq!(converted.observer, request.observer);
        assert_eq!(converted.bodies, request.bodies);
        assert_eq!(converted.zodiac_mode, request.zodiac_mode);
        assert_eq!(converted.apparentness, request.apparentness);
        assert_eq!(converted.house_system, request.house_system);

        let summary = converted.summary_line();
        assert!(summary.contains("(TDB);"));
        assert!(summary.contains("bodies=2;"));
        assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
        assert!(summary.contains("apparentness=Apparent;"));
        assert!(summary.contains("observer=house-only;"));
        assert!(summary.contains(
            "house system=My UTC Custom Houses [aliases: My UTC Alias] (uses a local UTC calibration)"
        ));
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tt() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tt_from_ut1(Duration::from_secs_f64(64.184))
        .expect("UT1 chart request should convert to TT");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tt_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tt_from_ut1_signed(64.184)
        .expect("UT1 chart request should accept signed TT offsets");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tt() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tt_from_utc(Duration::from_secs_f64(64.184))
        .expect("UTC chart request should convert to TT");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tt_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tt_from_utc_signed(64.184)
        .expect("UTC chart request should accept signed TT offsets");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tt_to_tdb() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_tdb_from_tt(Duration::from_secs_f64(0.001_657))
        .expect("TT chart request should convert to TDB");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tt_to_tdb_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_tdb_from_tt_signed(-0.001_657)
        .expect("TT chart request should accept signed TDB offsets");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tdb_to_tt() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ))
        .with_tt_from_tdb(-0.001_657)
        .expect("TDB chart request should convert to TT");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tdb_to_tt_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ))
        .with_tt_from_tdb_signed(-0.001_657)
        .expect("TDB chart request should accept signed TT offsets");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tdb() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tdb_from_ut1(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UT1 chart request should convert to TDB");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tdb_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UT1 chart request should accept signed TDB offsets");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tdb() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tdb_from_utc(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UTC chart request should convert to TDB");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tdb_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC chart request should accept signed TDB offsets");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_signed_time_scale_helpers_reject_non_finite_offsets() {
        let checked_request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ));
        let checked_error = checked_request
            .with_instant_time_scale_offset_checked(TimeScale::Tt, f64::NAN)
            .expect_err("UT1 chart request should reject non-finite checked offsets");
        assert!(matches!(
            checked_error,
            TimeScaleConversionError::NonFiniteOffset
        ));

        let tt_request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ));
        let tt_error = tt_request
            .with_tdb_from_tt_signed(f64::NAN)
            .expect_err("TT chart request should reject non-finite TDB offsets");
        assert!(matches!(
            tt_error,
            TimeScaleConversionError::NonFiniteOffset
        ));

        let ut1_request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ));
        let ut1_error = ut1_request
            .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), f64::INFINITY)
            .expect_err("UT1 chart request should reject non-finite TDB offsets");
        assert!(matches!(
            ut1_error,
            TimeScaleConversionError::NonFiniteOffset
        ));

        let utc_request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ));
        let utc_error = utc_request
            .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), f64::NEG_INFINITY)
            .expect_err("UTC chart request should reject non-finite TDB offsets");
        assert!(matches!(
            utc_error,
            TimeScaleConversionError::NonFiniteOffset
        ));
    }

    #[test]
    fn chart_request_time_scale_conversions_preserve_the_rest_of_the_request_shape() {
        let mut custom = pleiades_types::CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("My Alias".to_string());
        custom.notes = Some("uses a local calibration".to_string());

        let observer = ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(100.0),
        );

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_observer(observer)
        .with_house_system(HouseSystem::Custom(custom))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        })
        .with_apparentness(Apparentness::Apparent);

        let converted = request
            .clone()
            .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
            .expect("UTC chart request should accept signed TDB offsets");

        assert_eq!(converted.instant.scale, TimeScale::Tdb);
        assert_eq!(converted.observer, request.observer);
        assert_eq!(converted.bodies, request.bodies);
        assert_eq!(converted.zodiac_mode, request.zodiac_mode);
        assert_eq!(converted.apparentness, request.apparentness);
        assert_eq!(converted.house_system, request.house_system);
        let summary = converted.summary_line();
        assert!(summary.contains("(TDB);"));
        assert!(summary.contains("bodies=2;"));
        assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
        assert!(summary.contains("apparentness=Apparent;"));
        assert!(summary.contains("observer=house-only;"));
        assert!(summary.contains(
            "house system=My Custom Houses [aliases: My Alias] (uses a local calibration)"
        ));
    }

    #[test]
    fn chart_request_time_scale_conversions_preserve_the_rest_of_the_request_shape_for_ut1() {
        let mut custom = pleiades_types::CustomHouseSystem::new("My UT1 Custom Houses");
        custom.aliases.push("My UT1 Alias".to_string());
        custom.notes = Some("uses a local UT1 calibration".to_string());

        let observer = ObserverLocation::new(
            Latitude::from_degrees(-22.75),
            Longitude::from_degrees(135.5),
            Some(250.0),
        );

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_observer(observer)
        .with_house_system(HouseSystem::Custom(custom))
        .with_bodies(vec![CelestialBody::Mars, CelestialBody::Saturn])
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        })
        .with_apparentness(Apparentness::Apparent);

        let converted = request
            .clone()
            .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
            .expect("UT1 chart request should accept signed TDB offsets");

        assert_eq!(converted.instant.scale, TimeScale::Tdb);
        assert_eq!(converted.observer, request.observer);
        assert_eq!(converted.bodies, request.bodies);
        assert_eq!(converted.zodiac_mode, request.zodiac_mode);
        assert_eq!(converted.apparentness, request.apparentness);
        assert_eq!(converted.house_system, request.house_system);
        let summary = converted.summary_line();
        assert!(summary.contains("(TDB);"));
        assert!(summary.contains("bodies=2;"));
        assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
        assert!(summary.contains("apparentness=Apparent;"));
        assert!(summary.contains("observer=house-only;"));
        assert!(summary.contains(
            "house system=My UT1 Custom Houses [aliases: My UT1 Alias] (uses a local UT1 calibration)"
        ));
    }

    #[test]
    fn chart_request_validate_time_scale_conversion_matches_the_policy_helper() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ));
        let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tdb, 64.184);

        assert!(request.validate_time_scale_conversion(policy).is_ok());

        let mismatched = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, 64.184);
        let error = request
            .validate_time_scale_conversion(mismatched)
            .expect_err("chart request should reject the wrong source scale");
        assert!(matches!(
            error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Tt,
                actual: TimeScale::Ut1,
            }
        ));

        let non_finite = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, f64::NAN);
        assert!(matches!(
            request.validate_time_scale_conversion(non_finite),
            Err(TimeScaleConversionError::NonFiniteOffset)
        ));
    }

    #[test]
    fn chart_request_validate_house_observer_policy_requires_an_observer_for_house_requests() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_house_system(crate::HouseSystem::WholeSign);

        let error = request
            .validate_house_observer_policy()
            .expect_err("house requests should require an observer location");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert_eq!(
            error.message,
            "house placement requires an observer location"
        );
    }

    #[test]
    fn chart_request_validate_house_observer_policy_allows_house_requests_with_observers() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(100.0),
        ))
        .with_house_system(crate::HouseSystem::WholeSign);

        assert!(request.validate_house_observer_policy().is_ok());
    }

    #[test]
    fn chart_request_validate_observer_location_rejects_invalid_observers_without_houses() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(45.0),
            Some(100.0),
        ));

        let error = request
            .validate_observer_location()
            .expect_err("observer validation should reject out-of-range latitude");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error
            .message
            .contains("observer latitude must stay within [-90, 90], got 95"));
    }

    #[test]
    fn chart_request_validate_observer_location_rejects_invalid_body_observers_without_houses() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_body_observer(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(45.0),
            Some(100.0),
        ));

        let error = request
            .validate_observer_location()
            .expect_err("body-observer validation should reject out-of-range latitude");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error
            .message
            .contains("observer latitude must stay within [-90, 90], got 95"));
    }

    #[test]
    fn chart_request_validate_house_observer_policy_rejects_invalid_observers_without_houses() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(f64::NAN),
            Some(100.0),
        ));

        let error = request
            .validate_house_observer_policy()
            .expect_err("house-observer policy should reject invalid observer coordinates");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error.message.contains("observer longitude must be finite"));
    }

    #[test]
    fn chart_request_time_scale_conversions_preserve_the_rest_of_the_request_shape_for_tdb() {
        let mut custom = pleiades_types::CustomHouseSystem::new("My TDB Custom Houses");
        custom.aliases.push("My TDB Alias".to_string());
        custom.notes = Some("uses a local TDB calibration".to_string());

        let observer = ObserverLocation::new(
            Latitude::from_degrees(34.5),
            Longitude::from_degrees(-118.25),
            Some(75.0),
        );

        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ))
        .with_observer(observer)
        .with_house_system(HouseSystem::Custom(custom))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        })
        .with_apparentness(Apparentness::Apparent);

        let converted = request
            .clone()
            .with_tt_from_tdb_signed(-0.001_657)
            .expect("TDB chart request should accept signed TT offsets");

        assert_eq!(converted.instant.scale, TimeScale::Tt);
        assert_eq!(converted.observer, request.observer);
        assert_eq!(converted.bodies, request.bodies);
        assert_eq!(converted.zodiac_mode, request.zodiac_mode);
        assert_eq!(converted.apparentness, request.apparentness);
        assert_eq!(converted.house_system, request.house_system);
        let summary = converted.summary_line();
        assert!(summary.contains("(TT);"));
        assert!(summary.contains("bodies=2;"));
        assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
        assert!(summary.contains("apparentness=Apparent;"));
        assert!(summary.contains("observer=house-only;"));
        assert!(summary.contains(
            "house system=My TDB Custom Houses [aliases: My TDB Alias] (uses a local TDB calibration)"
        ));
    }

    #[test]
    fn body_placement_exposes_motion_direction() {
        let mut result = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Mars,
            Instant::new(
                pleiades_types::JulianDay::from_days(2451545.0),
                TimeScale::Tt,
            ),
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        result.motion = Some(pleiades_types::Motion::new(
            Some(-0.012),
            Some(0.004),
            Some(0.0012),
        ));

        let placement = BodyPlacement {
            body: CelestialBody::Mars,
            position: result,
            sign: None,
            house: None,
        };

        assert_eq!(
            placement.motion_direction(),
            Some(MotionDirection::Retrograde)
        );
        assert_eq!(placement.longitude_speed(), Some(-0.012));
        assert_eq!(placement.latitude_speed(), Some(0.004));
        assert_eq!(placement.distance_speed(), Some(0.0012));
        assert_eq!(
            placement
                .motion()
                .and_then(|motion| motion.longitude_deg_per_day),
            Some(-0.012)
        );
    }

    #[test]
    fn body_placement_treats_non_finite_motion_as_unknown() {
        let mut result = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Mars,
            Instant::new(
                pleiades_types::JulianDay::from_days(2451545.0),
                TimeScale::Tt,
            ),
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        result.motion = Some(pleiades_types::Motion::new(
            Some(0.012),
            Some(f64::NAN),
            Some(0.0012),
        ));

        let placement = BodyPlacement {
            body: CelestialBody::Mars,
            position: result,
            sign: None,
            house: None,
        };

        assert_eq!(placement.motion_direction(), None);
    }

    #[test]
    fn motion_summary_validates_against_placement_count() {
        let summary = MotionSummary {
            direct: 2,
            stationary: 1,
            retrograde: 3,
            unknown: 0,
        };

        assert_eq!(summary.validate(6), Ok(()));
        assert_eq!(
            summary.summary_line(),
            "2 direct, 1 stationary, 3 retrograde, 0 unknown"
        );

        let error = MotionSummary {
            direct: 1,
            stationary: 1,
            retrograde: 0,
            unknown: 1,
        }
        .validate(4)
        .expect_err("mismatched motion summary counts should fail validation");

        assert_eq!(
            error,
            MotionSummaryValidationError::PlacementCountMismatch {
                expected: 4,
                actual: 3,
            }
        );
    }

    #[test]
    fn body_placement_summary_line_matches_display() {
        let mut result = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Venus,
            Instant::new(
                pleiades_types::JulianDay::from_days(2451545.0),
                TimeScale::Tt,
            ),
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(123.25),
            Latitude::from_degrees(-5.5),
            Some(0.987),
        ));
        result.motion = Some(pleiades_types::Motion::new(
            Some(-0.012),
            Some(0.004),
            Some(0.0012),
        ));

        let placement = BodyPlacement {
            body: CelestialBody::Venus,
            position: result,
            sign: Some(ZodiacSign::Leo),
            house: Some(7),
        };

        let summary = placement.summary_line();
        assert_eq!(summary, placement.to_string());
        assert!(summary.contains("Venus"));
        assert!(summary.contains("123.25°"));
        assert!(summary.contains("Leo"));
        assert!(summary.contains("7"));
        assert!(summary.contains("Retrograde"));
        assert!(summary.contains(&placement.position.quality.to_string()));
    }

    #[test]
    fn chart_snapshot_supports_body_lookup_and_retrograde_summary() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let mut retrograde = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Mars,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        retrograde.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(90.0),
            Latitude::from_degrees(0.0),
            None,
        ));
        retrograde.motion = Some(pleiades_types::Motion::new(Some(-0.01), None, None));

        let mut direct = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Sun,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        direct.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(15.0),
            Latitude::from_degrees(0.0),
            None,
        ));
        direct.motion = Some(pleiades_types::Motion::new(Some(0.01), None, None));

        let mut stationary = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Mercury,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        stationary.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(45.0),
            Latitude::from_degrees(0.0),
            None,
        ));
        stationary.motion = Some(pleiades_types::Motion::new(Some(0.0), None, None));

        let mut unknown = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Jupiter,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        unknown.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(105.0),
            Latitude::from_degrees(0.0),
            None,
        ));

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            body_observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Apparent,
            houses: None,
            placements: vec![
                BodyPlacement {
                    body: CelestialBody::Sun,
                    position: direct,
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Mercury,
                    position: stationary,
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
                BodyPlacement {
                    body: CelestialBody::Mars,
                    position: retrograde,
                    sign: Some(ZodiacSign::Cancer),
                    house: Some(8),
                },
                BodyPlacement {
                    body: CelestialBody::Jupiter,
                    position: unknown,
                    sign: Some(ZodiacSign::Leo),
                    house: Some(9),
                },
            ],
        };

        assert!(chart.placement_for(&CelestialBody::Sun).is_some());
        assert_eq!(
            chart.sign_for_body(&CelestialBody::Sun),
            Some(ZodiacSign::Aries)
        );
        assert_eq!(
            chart.sign_for_body(&CelestialBody::Mars),
            Some(ZodiacSign::Cancer)
        );
        assert_eq!(chart.house_for_body(&CelestialBody::Sun), Some(1));
        assert_eq!(chart.house_for_body(&CelestialBody::Mars), Some(8));
        assert_eq!(chart.house_for_body(&CelestialBody::Mercury), Some(2));
        assert_eq!(
            chart.occupied_signs(),
            vec![
                ZodiacSign::Aries,
                ZodiacSign::Taurus,
                ZodiacSign::Cancer,
                ZodiacSign::Leo,
            ]
        );
        assert_eq!(chart.occupied_houses(), vec![1, 2, 8, 9]);
        assert_eq!(
            chart.motion_direction_for(&CelestialBody::Mars),
            Some(MotionDirection::Retrograde)
        );
        assert_eq!(chart.longitude_speed_for(&CelestialBody::Sun), Some(0.01));
        assert_eq!(chart.latitude_speed_for(&CelestialBody::Sun), None);
        assert_eq!(chart.distance_speed_for(&CelestialBody::Sun), None);
        assert_eq!(
            chart
                .motion_for_body(&CelestialBody::Mars)
                .and_then(|motion| motion.longitude_deg_per_day),
            Some(-0.01)
        );
        assert_eq!(
            chart
                .placements_in_sign(ZodiacSign::Cancer)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mars]
        );
        assert!(chart
            .placements_in_sign(ZodiacSign::Pisces)
            .next()
            .is_none());
        assert_eq!(
            chart
                .placements_in_house(2)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mercury]
        );
        assert_eq!(
            chart
                .placements_in_house(8)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mars]
        );
        assert!(chart.placements_in_house(12).next().is_none());
        assert_eq!(chart.stationary_placements().count(), 1);
        assert_eq!(chart.unknown_motion_placements().count(), 1);
        assert_eq!(chart.retrograde_placements().count(), 1);
        assert_eq!(
            chart.house_summary(),
            HouseSummary {
                first: 1,
                second: 1,
                third: 0,
                fourth: 0,
                fifth: 0,
                sixth: 0,
                seventh: 0,
                eighth: 1,
                ninth: 1,
                tenth: 0,
                eleventh: 0,
                twelfth: 0,
                unknown: 0,
            }
        );
        assert_eq!(
            chart.motion_summary(),
            MotionSummary {
                direct: 1,
                stationary: 1,
                retrograde: 1,
                unknown: 1,
            }
        );
        assert_eq!(
            chart.motion_summary().summary_line(),
            chart.motion_summary().to_string()
        );
        assert_eq!(
            chart
                .placements_with_motion_direction(MotionDirection::Direct)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Sun]
        );
        assert_eq!(
            chart
                .placements_with_motion_direction(MotionDirection::Stationary)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mercury]
        );
        assert_eq!(
            chart
                .direct_placements()
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Sun]
        );
        let rendered = chart.to_string();
        assert!(rendered.contains(
            "House summary: 1 in 1st house, 1 in 2nd house, 1 in 8th house, 1 in 9th house"
        ));
        assert!(
            rendered.contains("Motion summary: 1 direct, 1 stationary, 1 retrograde, 1 unknown")
        );
        assert!(rendered.contains("Stationary bodies: Mercury"));
        assert!(rendered.contains("Unknown motion bodies: Jupiter"));
        assert!(rendered.contains("Retrograde bodies: Mars"));
    }

    #[test]
    fn motion_summary_summary_line_matches_display_order() {
        let summary = MotionSummary {
            direct: 2,
            stationary: 1,
            retrograde: 3,
            unknown: 4,
        };

        assert_eq!(
            summary.summary_line(),
            "2 direct, 1 stationary, 3 retrograde, 4 unknown"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert!(summary.has_known_motion());
    }

    #[test]
    fn chart_snapshot_exposes_major_aspects_and_angular_separation() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let mut sun = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Sun,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        sun.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(15.0),
            Latitude::from_degrees(0.0),
            None,
        ));

        let mut moon = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Moon,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        moon.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(75.0),
            Latitude::from_degrees(0.0),
            None,
        ));

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            body_observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Apparent,
            houses: None,
            placements: vec![
                BodyPlacement {
                    body: CelestialBody::Sun,
                    position: sun,
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Moon,
                    position: moon,
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
            ],
        };

        assert_eq!(
            chart.angular_separation(&CelestialBody::Sun, &CelestialBody::Moon),
            Some(Angle::from_degrees(60.0))
        );

        let aspects = chart.major_aspects();
        assert_eq!(aspects.len(), 1);
        let aspect = &aspects[0];
        assert_eq!(aspect.left, CelestialBody::Sun);
        assert_eq!(aspect.right, CelestialBody::Moon);
        assert_eq!(aspect.kind, AspectKind::Sextile);
        assert_eq!(aspect.separation, Angle::from_degrees(60.0));
        assert_eq!(aspect.orb, Angle::from_degrees(0.0));
        assert_eq!(
            chart.aspect_summary(),
            AspectSummary {
                conjunction: 0,
                sextile: 1,
                square: 0,
                trine: 0,
                opposition: 0,
            }
        );
        assert_eq!(chart.aspect_summary().summary_line(), "1 Sextile");
        assert_eq!(chart.aspect_summary().to_string(), "1 Sextile");
        let rendered = chart.to_string();
        assert!(rendered.contains("Aspect summary: 1 Sextile"));
        assert!(rendered.contains("Aspects:"));
        assert!(rendered.contains("Sun Sextile Moon"));
    }

    #[test]
    fn chart_snapshot_renders_custom_body_identifiers() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let custom_body =
            CelestialBody::Custom(pleiades_types::CustomBodyId::new("asteroid", "433-Eros"));
        let mut result = EphemerisResult::new(
            BackendId::new("toy-chart"),
            custom_body.clone(),
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        result.motion = Some(pleiades_types::Motion::new(Some(-0.01), None, None));

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            body_observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Apparent,
            houses: None,
            placements: vec![BodyPlacement {
                body: custom_body,
                position: result,
                sign: None,
                house: None,
            }],
        };

        let rendered = chart.to_string();
        assert!(rendered.contains("asteroid:433-Eros"));
        assert!(rendered.contains("Retrograde bodies: asteroid:433-Eros"));
    }

    #[test]
    fn aspect_definition_summary_line_and_validation_remain_typed() {
        let definition = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
        assert_eq!(
            definition.summary_line(),
            "kind=Sextile; exact_degrees=60°; orb_degrees=4°"
        );
        assert_eq!(definition.to_string(), definition.summary_line());
    }

    #[test]
    fn aspect_definition_validation_rejects_non_finite_and_out_of_range_values() {
        let valid = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
        assert!(valid.validate().is_ok());
        assert!(validate_aspect_definitions(&[valid]).is_ok());

        let exact_nan = AspectDefinition::new(AspectKind::Conjunction, f64::NAN, 8.0);
        let exact_error = exact_nan
            .validate()
            .expect_err("NaN exact degrees should be rejected");
        assert!(exact_error
            .summary_line()
            .contains("exact_degrees must be finite and between 0 and 180 degrees"));
        assert_eq!(exact_error.to_string(), exact_error.summary_line());

        let orb_negative = AspectDefinition::new(AspectKind::Square, 90.0, -1.0);
        let orb_error = orb_negative
            .validate()
            .expect_err("negative orbs should be rejected");
        assert!(orb_error
            .summary_line()
            .contains("orb_degrees must be finite and between 0 and 180 degrees"));
        assert_eq!(orb_error.to_string(), orb_error.summary_line());

        let exact_out_of_range = AspectDefinition::new(AspectKind::Trine, 181.0, 6.0);
        assert!(exact_out_of_range
            .validate()
            .expect_err("angles outside the supported half-circle should be rejected")
            .summary_line()
            .contains("found 181"));

        assert!(validate_aspect_definitions(&[
            valid,
            AspectDefinition::new(AspectKind::Opposition, 180.0, 8.0),
        ])
        .is_ok());
    }
}
