use crate::errors::{map_custom_definition_error, EphemerisError};
use crate::metadata::BackendMetadata;
use core::fmt;
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, Instant, ObserverLocation, TimeScale,
    TimeScaleConversion, TimeScaleConversionError, ZodiacMode,
};
use std::time::Duration;

/// A backend request.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct EphemerisRequest {
    /// Requested body.
    pub body: CelestialBody,
    /// Requested instant.
    pub instant: Instant,
    /// Optional observer location for topocentric calculations.
    pub observer: Option<ObserverLocation>,
    /// Requested coordinate frame.
    pub frame: CoordinateFrame,
    /// Requested zodiac mode.
    pub zodiac_mode: ZodiacMode,
    /// Whether apparent or mean values are preferred.
    pub apparent: Apparentness,
}

impl EphemerisRequest {
    /// Creates a new request with sensible defaults for a tropical geocentric query.
    ///
    /// The default apparentness is mean geometric output so a bare request stays
    /// compatible with the current mean-only first-party backends.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::{Apparentness, EphemerisRequest};
    /// use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode};
    ///
    /// let request = EphemerisRequest::new(
    ///     CelestialBody::Mars,
    ///     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    /// );
    ///
    /// assert_eq!(request.frame, CoordinateFrame::Ecliptic);
    /// assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
    /// assert_eq!(request.apparent, Apparentness::Mean);
    /// assert!(request.observer.is_none());
    /// ```
    pub fn new(body: CelestialBody, instant: Instant) -> Self {
        Self {
            body,
            instant,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        }
    }

    /// Returns a compact one-line rendering of the request shape.
    pub fn summary_line(&self) -> String {
        let observer = self
            .observer
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "geocentric".to_string());

        format!(
            "body={}; instant={}; frame={}; zodiac={}; apparent={}; observer={}",
            self.body, self.instant, self.frame, self.zodiac_mode, self.apparent, observer
        )
    }

    /// Validates any custom request identifiers embedded in this request.
    ///
    /// Built-in bodies and sidereal labels are always accepted. Custom bodies
    /// and custom ayanamsas are validated through their structured descriptor
    /// records so malformed user-defined entries fail before request dispatch.
    pub fn validate_custom_definitions(&self) -> Result<(), EphemerisError> {
        self.body
            .validate()
            .map_err(|error| map_custom_definition_error("request body", error))?;

        if let ZodiacMode::Sidereal { ayanamsa } = &self.zodiac_mode {
            ayanamsa
                .validate()
                .map_err(|error| map_custom_definition_error("sidereal ayanamsa", error))?;
        }

        Ok(())
    }

    /// Replaces the request instant with a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::with_time_scale_offset`].
    /// It preserves the rest of the request shape while letting direct backend
    /// callers stage explicit Delta T or TDB offsets before dispatch.
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

    /// Replaces the request instant with a caller-supplied offset after validation.
    ///
    /// This is the checked counterpart to [`EphemerisRequest::with_instant_time_scale_offset`].
    /// It rejects non-finite offsets and mismatched source scales before the
    /// request is retagged, which keeps the backend-level convenience available
    /// in a release-grade form.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::EphemerisRequest;
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// let request = EphemerisRequest::new(
    ///     CelestialBody::Sun,
    ///     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1),
    /// );
    /// let converted = request
    ///     .with_instant_time_scale_offset_checked(TimeScale::Tt, 64.184)
    ///     .expect("validated backend offset");
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

    /// Applies a caller-supplied time-scale conversion policy to the request instant.
    ///
    /// This is the generic counterpart to the source-specific offset helpers.
    /// It keeps the explicit source, target, and offset choice available as a
    /// typed policy object while preserving the rest of the request shape.
    pub fn with_time_scale_conversion(
        mut self,
        conversion: TimeScaleConversion,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = conversion.apply(self.instant)?;
        Ok(self)
    }

    /// Converts the request instant from TT to TDB using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_tt`].
    pub fn with_tdb_from_tt(mut self, offset: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt(offset)?;
        Ok(self)
    }

    /// Converts the request instant from TT to TDB using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_tt_signed`].
    pub fn with_tdb_from_tt_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from TDB to TT using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_tdb`].
    pub fn with_tt_from_tdb(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb(offset_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from TDB to TT using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_tdb_signed`].
    pub fn with_tt_from_tdb_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TT using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_ut1`].
    pub fn with_tt_from_ut1(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1(delta_t)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TT using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_ut1_signed`].
    pub fn with_tt_from_ut1_signed(
        mut self,
        delta_t_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1_signed(delta_t_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from UTC to TT using a caller-supplied offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_utc`].
    pub fn with_tt_from_utc(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc(delta_t)?;
        Ok(self)
    }

    /// Converts the request instant from UTC to TT using a caller-supplied signed offset.
    ///
    /// This is the backend-level counterpart to [`Instant::tt_from_utc_signed`].
    pub fn with_tt_from_utc_signed(
        mut self,
        delta_t_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc_signed(delta_t_seconds)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TDB using caller-supplied TT-UT1 and TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_ut1`].
    pub fn with_tdb_from_ut1(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_ut1(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the request instant from UT1 to TDB using caller-supplied TT-UT1 and signed TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_ut1_signed`].
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

    /// Converts the request instant from UTC to TDB using caller-supplied TT-UTC and TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_utc`].
    pub fn with_tdb_from_utc(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_utc(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the request instant from UTC to TDB using caller-supplied TT-UTC and signed TDB-TT offsets.
    ///
    /// This is the backend-level counterpart to [`Instant::tdb_from_utc_signed`].
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

    /// Validates a caller-supplied time-scale conversion policy without mutating the request.
    ///
    /// This mirrors [`TimeScaleConversion::validate`] at the backend-request
    /// layer so direct backend callers can preflight the explicit source/target/
    /// offset contract before choosing whether to apply it.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::EphemerisRequest;
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale, TimeScaleConversion};
    ///
    /// let request = EphemerisRequest::new(
    ///     CelestialBody::Sun,
    ///     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
    /// );
    /// let policy = TimeScaleConversion::new(TimeScale::Utc, TimeScale::Tt, 64.184);
    ///
    /// assert!(request.validate_time_scale_conversion(policy).is_ok());
    /// ```
    pub fn validate_time_scale_conversion(
        &self,
        conversion: TimeScaleConversion,
    ) -> Result<(), TimeScaleConversionError> {
        self.instant.validate_time_scale_conversion(conversion)
    }

    /// Validates this request against backend metadata.
    ///
    /// This is the method-form counterpart to [`crate::validate_request_against_metadata`].
    /// It keeps direct backend callers from having to import the free helper when
    /// they want to preflight a request before dispatch.
    pub fn validate_against_metadata(
        &self,
        metadata: &BackendMetadata,
    ) -> Result<(), EphemerisError> {
        crate::policy::current::validate_request_against_metadata(self, metadata)
    }
}

impl fmt::Display for EphemerisRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
