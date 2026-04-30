//! Shared primitive and domain-adjacent types used across the workspace.
//!
//! These types define the vocabulary for angles, time scales, celestial body
//! identifiers, observer locations, coordinate frames, and catalog selections.
//! Higher-level crates build on these semantics without re-labelling the same
//! concepts in backend-specific ways.
//!
//! Enable the optional `serde` feature to serialize and deserialize the public
//! type vocabulary for interchange or caching workflows.
//!
//! # Examples
//!
//! ```
//! use pleiades_types::{Angle, Longitude};
//!
//! let angle = Angle::from_degrees(-30.0);
//! assert_eq!(angle.normalized_0_360().degrees(), 330.0);
//!
//! let lon = Longitude::from_degrees(390.0);
//! assert_eq!(lon.degrees(), 30.0);
//! ```

#![forbid(unsafe_code)]

use core::fmt;
use core::time::Duration;

/// An angular quantity measured in degrees.
///
/// `Angle` is intentionally neutral: it does not assume a normalization range.
/// Use [`Angle::normalized_0_360`] or [`Angle::normalized_signed`] when a
/// canonical wrap is required.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Angle(f64);

impl Angle {
    /// Creates a new angle measured in degrees.
    pub const fn from_degrees(degrees: f64) -> Self {
        Self(degrees)
    }

    /// Creates a new angle measured in radians.
    pub fn from_radians(radians: f64) -> Self {
        Self(radians.to_degrees())
    }

    /// Returns the underlying angle in degrees.
    pub const fn degrees(self) -> f64 {
        self.0
    }

    /// Returns the angle in radians.
    pub fn radians(self) -> f64 {
        self.0.to_radians()
    }

    /// Returns the angle normalized into the half-open range `[0, 360)`.
    pub fn normalized_0_360(self) -> Self {
        Self(self.0.rem_euclid(360.0))
    }

    /// Returns the angle normalized into the signed range `[-180, 180)`.
    pub fn normalized_signed(self) -> Self {
        let degrees = self.normalized_0_360().degrees();
        if degrees >= 180.0 {
            Self(degrees - 360.0)
        } else {
            Self(degrees)
        }
    }

    /// Returns `true` when the underlying numeric value is finite.
    pub const fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.0)
    }
}

/// A canonical ecliptic or longitude-like angle normalized into `[0, 360)`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Longitude(Angle);

impl Longitude {
    /// Creates a longitude normalized into `[0, 360)`.
    pub fn from_degrees(degrees: f64) -> Self {
        Self(Angle::from_degrees(degrees).normalized_0_360())
    }

    /// Returns the longitude in degrees, already normalized into `[0, 360)`.
    pub const fn degrees(self) -> f64 {
        self.0.degrees()
    }

    /// Returns the underlying angle wrapper.
    pub const fn angle(self) -> Angle {
        self.0
    }
}

impl From<Angle> for Longitude {
    fn from(value: Angle) -> Self {
        Self::from_degrees(value.degrees())
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A signed latitude-like angle measured in degrees.
///
/// Latitude values are not automatically clamped; the caller is expected to
/// provide values consistent with the relevant coordinate system.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Latitude(Angle);

impl Latitude {
    /// Creates a latitude measured in degrees.
    pub const fn from_degrees(degrees: f64) -> Self {
        Self(Angle::from_degrees(degrees))
    }

    /// Returns the latitude in degrees.
    pub const fn degrees(self) -> f64 {
        self.0.degrees()
    }

    /// Returns the underlying angle wrapper.
    pub const fn angle(self) -> Angle {
        self.0
    }
}

impl From<Angle> for Latitude {
    fn from(value: Angle) -> Self {
        Self::from_degrees(value.degrees())
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A Julian day expressed as a floating-point day count.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct JulianDay(f64);

impl JulianDay {
    /// Creates a new Julian day value.
    pub const fn from_days(days: f64) -> Self {
        Self(days)
    }

    /// Returns the raw floating-point day count.
    pub const fn days(self) -> f64 {
        self.0
    }

    /// Returns a Julian day shifted by the supplied number of SI seconds.
    ///
    /// This is a mechanical day-count operation. It does not choose or model a
    /// time-scale conversion policy by itself; callers must provide the offset
    /// appropriate for the source and target scales.
    pub fn add_seconds(self, seconds: f64) -> Self {
        Self(self.0 + seconds / SECONDS_PER_DAY)
    }
}

impl fmt::Display for JulianDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JD {}", self.0)
    }
}

/// A supported astronomical time scale.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum TimeScale {
    /// Coordinated Universal Time.
    Utc,
    /// Universal Time 1.
    Ut1,
    /// Terrestrial Time.
    Tt,
    /// Barycentric Dynamical Time.
    Tdb,
}

impl fmt::Display for TimeScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Utc => "UTC",
            Self::Ut1 => "UT1",
            Self::Tt => "TT",
            Self::Tdb => "TDB",
        };
        f.write_str(label)
    }
}

/// Number of SI seconds in one Julian day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// Error returned when a caller-provided time-scale conversion fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeScaleConversionError {
    /// Time scale required by the conversion helper.
    Expected {
        expected: TimeScale,
        actual: TimeScale,
    },
    /// The supplied offset was not a finite number of seconds.
    NonFiniteOffset,
}

impl TimeScaleConversionError {
    const fn expected(expected: TimeScale, actual: TimeScale) -> Self {
        Self::Expected { expected, actual }
    }

    const fn non_finite_offset() -> Self {
        Self::NonFiniteOffset
    }

    /// Returns a compact one-line rendering of the conversion failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::Expected { expected, actual } => format!(
                "time-scale conversion expected {}, got {}",
                expected, actual
            ),
            Self::NonFiniteOffset => "time-scale conversion offset must be finite".to_string(),
        }
    }
}

impl fmt::Display for TimeScaleConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for TimeScaleConversionError {}

/// Shared validation errors for [`TimeRange`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeRangeValidationError {
    /// A required bound contains a non-finite Julian day.
    NonFiniteBound {
        /// Which bound failed validation.
        bound: &'static str,
        /// The offending instant.
        instant: Instant,
    },
    /// The lower and upper bounds use different time scales.
    ScaleMismatch {
        /// The lower bound.
        start: Instant,
        /// The upper bound.
        end: Instant,
    },
    /// The upper bound precedes the lower bound.
    OutOfOrder {
        /// The lower bound.
        start: Instant,
        /// The upper bound.
        end: Instant,
    },
}

impl TimeRangeValidationError {
    const fn non_finite_bound(bound: &'static str, instant: Instant) -> Self {
        Self::NonFiniteBound { bound, instant }
    }

    const fn scale_mismatch(start: Instant, end: Instant) -> Self {
        Self::ScaleMismatch { start, end }
    }

    const fn out_of_order(start: Instant, end: Instant) -> Self {
        Self::OutOfOrder { start, end }
    }

    /// Returns a compact one-line rendering of the range validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonFiniteBound { bound, instant } => format!(
                "time range bound `{bound}` must be finite: {}",
                format_time_range_instant(*instant)
            ),
            Self::ScaleMismatch { start, end } => format!(
                "time range bounds must use the same time scale: start={}; end={}",
                format_time_range_instant(*start),
                format_time_range_instant(*end)
            ),
            Self::OutOfOrder { start, end } => format!(
                "time range end must not precede the start: start={}; end={}",
                format_time_range_instant(*start),
                format_time_range_instant(*end)
            ),
        }
    }
}

impl fmt::Display for TimeRangeValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for TimeRangeValidationError {}

/// A caller-supplied time-scale conversion policy.
///
/// The conversion stores the source and target time scales plus the explicit
/// `target - source` offset in SI seconds. It does not model Delta T,
/// leap seconds, DUT1, or relativistic TDB terms itself; it only packages the
/// caller's chosen rule so an instant can be retagged explicitly and
/// reproducibly. Its compact summary renders the structured field names
/// explicitly so release-facing diagnostics do not have to infer which side of
/// the conversion the offset applies to.
///
/// # Example
///
/// ```
/// use pleiades_types::{Instant, JulianDay, TimeScale, TimeScaleConversion};
///
/// let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
/// let converted = policy.apply(instant).expect("UT1-tagged instant");
///
/// assert_eq!(converted.scale, TimeScale::Tt);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeScaleConversion {
    /// The source time scale expected by the policy.
    pub source: TimeScale,
    /// The target time scale produced by the policy.
    pub target: TimeScale,
    /// The explicit `target - source` offset in SI seconds.
    pub offset_seconds: f64,
}

impl TimeScaleConversion {
    /// Creates a new caller-supplied time-scale conversion policy.
    pub const fn new(source: TimeScale, target: TimeScale, offset_seconds: f64) -> Self {
        Self {
            source,
            target,
            offset_seconds,
        }
    }

    /// Returns a compact one-line rendering of the caller-supplied policy.
    pub fn summary_line(&self) -> String {
        format!(
            "source={}; target={}; offset_seconds={} s",
            self.source, self.target, self.offset_seconds
        )
    }

    /// Validates the policy against a specific instant without retagging it.
    ///
    /// This is useful when a caller wants to preflight the explicit conversion
    /// contract before mutating the instant itself. The source scale must match
    /// the instant's current scale, and the offset must be finite.
    pub fn validate(self, instant: Instant) -> Result<(), TimeScaleConversionError> {
        if instant.scale != self.source {
            return Err(TimeScaleConversionError::expected(
                self.source,
                instant.scale,
            ));
        }

        checked_time_scale_offset(self.offset_seconds)?;
        Ok(())
    }

    /// Applies the policy to an instant.
    ///
    /// This is the mutating counterpart to [`TimeScaleConversion::validate`].
    /// The source scale must match the instant's current scale, and the offset
    /// must be finite. Otherwise the conversion fails with the same structured
    /// time-scale error used by the lower-level helpers.
    pub fn apply(self, instant: Instant) -> Result<Instant, TimeScaleConversionError> {
        self.validate(instant)?;
        Ok(instant.with_time_scale_offset(self.target, self.offset_seconds))
    }
}

impl fmt::Display for TimeScaleConversion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn checked_time_scale_offset(offset_seconds: f64) -> Result<f64, TimeScaleConversionError> {
    if offset_seconds.is_finite() {
        Ok(offset_seconds)
    } else {
        Err(TimeScaleConversionError::non_finite_offset())
    }
}

/// A Julian day tagged with a time scale.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instant {
    /// The numeric Julian day value.
    pub julian_day: JulianDay,
    /// The time scale used by the Julian day value.
    pub scale: TimeScale,
}

impl Instant {
    /// Creates a new instant from a Julian day and time scale.
    pub const fn new(julian_day: JulianDay, scale: TimeScale) -> Self {
        Self { julian_day, scale }
    }

    /// Returns a compact one-line rendering of the instant.
    pub fn summary_line(&self) -> String {
        format!("{} {}", self.julian_day, self.scale)
    }

    /// Applies a caller-supplied time-scale conversion policy.
    ///
    /// This helper is the generic counterpart to the source-specific
    /// `tt_from_*` / `tdb_from_*` methods. It lets callers package the explicit
    /// source, target, and offset choice into one typed record when they want
    /// to keep the conversion contract alongside the instant.
    pub fn with_time_scale_conversion(
        self,
        conversion: TimeScaleConversion,
    ) -> Result<Self, TimeScaleConversionError> {
        conversion.apply(self)
    }

    /// Validates a caller-supplied time-scale conversion policy without retagging the instant.
    ///
    /// This is the foundation-layer counterpart to
    /// [`TimeScaleConversion::validate`], which lets callers preflight the same
    /// explicit source/target/offset contract directly from an instant when
    /// they do not yet want to mutate it.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_types::{Instant, JulianDay, TimeScale, TimeScaleConversion};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    /// let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
    ///
    /// assert!(instant.validate_time_scale_conversion(policy).is_ok());
    /// ```
    pub fn validate_time_scale_conversion(
        self,
        conversion: TimeScaleConversion,
    ) -> Result<(), TimeScaleConversionError> {
        conversion.validate(self)
    }

    /// Returns this instant with a caller-supplied offset applied and a new time
    /// scale tag.
    ///
    /// The offset is expressed as `target - source` in SI seconds. For example,
    /// callers converting UT1 to TT can pass Delta T (`TT - UT1`) and set
    /// `target_scale` to [`TimeScale::Tt`]. This helper intentionally performs
    /// no leap-second, DUT1, Delta T, or relativistic modeling; it only makes the
    /// caller-provided policy explicit and reproducible. Callers should pass a
    /// finite offset; the validated signed helpers reject non-finite values
    /// before reaching this low-level retagging step.
    pub fn with_time_scale_offset(self, target_scale: TimeScale, offset_seconds: f64) -> Self {
        Self {
            julian_day: self.julian_day.add_seconds(offset_seconds),
            scale: target_scale,
        }
    }

    /// Returns this instant with a caller-supplied offset applied and a new time
    /// scale tag after validating the source scale and offset.
    ///
    /// This is the checked counterpart to [`Instant::with_time_scale_offset`].
    /// It keeps the same explicit `target - source` interpretation while
    /// rejecting non-finite offsets and mismatched source scales before the
    /// instant is retagged.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    /// let converted = instant
    ///     .with_time_scale_offset_checked(TimeScale::Tt, 64.184)
    ///     .expect("validated offset");
    ///
    /// assert_eq!(converted.scale, TimeScale::Tt);
    /// ```
    pub fn with_time_scale_offset_checked(
        self,
        target_scale: TimeScale,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        TimeScaleConversion::new(self.scale, target_scale, offset_seconds).apply(self)
    }

    /// Returns the mean obliquity of the ecliptic for this instant.
    ///
    /// The value uses the shared cubic approximation currently used throughout
    /// the workspace for mean-obliquity equatorial transforms. It is expressed
    /// as a typed angle so callers can pass it directly into coordinate
    /// conversion helpers.
    pub fn mean_obliquity(self) -> Angle {
        let t = (self.julian_day.days() - 2_451_545.0) / 36_525.0;
        Angle::from_degrees(
            23.439_291_111_111_11
                - 0.013_004_166_666_666_667 * t
                - 0.000_000_163_888_888_888_888_88 * t * t
                + 0.000_000_503_611_111_111_111_1 * t * t * t,
        )
    }

    /// Converts a UT1-tagged instant to TT using caller-supplied Delta T.
    ///
    /// `delta_t` must be the value `TT - UT1`. Use this when validation data or
    /// an application already has an explicit Delta T policy and wants to pass a
    /// TT instant to backends that require TT. UTC-to-TT conversion is not
    /// represented by this helper, because UTC also requires leap-second and
    /// DUT1 handling outside the current type layer.
    ///
    /// For signed `TT - UT1` policies, use [`Instant::tt_from_ut1_signed`].
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    /// let converted = instant.tt_from_ut1(Duration::from_secs_f64(64.184)).expect("UT1-tagged instant");
    ///
    /// assert_eq!(converted.scale, TimeScale::Tt);
    /// assert!(converted.julian_day.days() > instant.julian_day.days());
    /// ```
    pub fn tt_from_ut1(self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        if self.scale != TimeScale::Ut1 {
            return Err(TimeScaleConversionError::expected(
                TimeScale::Ut1,
                self.scale,
            ));
        }

        Ok(self.with_time_scale_offset(TimeScale::Tt, delta_t.as_secs_f64()))
    }

    /// Converts a UT1-tagged instant to TT using a caller-supplied signed offset.
    ///
    /// `offset_seconds` must be the already-chosen signed `TT - UT1` offset in
    /// SI seconds. The helper intentionally does not model leap seconds or
    /// DUT1 by itself; it only makes a caller-supplied UT1-to-TT policy explicit
    /// and reproducible for applications that need a TT-tagged request surface.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    /// let converted = instant.tt_from_ut1_signed(64.184).expect("UT1-tagged instant");
    ///
    /// assert_eq!(converted.scale, TimeScale::Tt);
    /// assert!(converted.julian_day.days() > instant.julian_day.days());
    /// ```
    pub fn tt_from_ut1_signed(self, offset_seconds: f64) -> Result<Self, TimeScaleConversionError> {
        if self.scale != TimeScale::Ut1 {
            return Err(TimeScaleConversionError::expected(
                TimeScale::Ut1,
                self.scale,
            ));
        }

        let offset_seconds = checked_time_scale_offset(offset_seconds)?;

        Ok(self.with_time_scale_offset(TimeScale::Tt, offset_seconds))
    }

    /// Converts a UTC-tagged instant to TT using caller-supplied offset.
    ///
    /// `delta_t` must be the already-chosen `TT - UTC` offset in SI seconds.
    /// The helper intentionally does not model leap seconds or DUT1 by itself;
    /// it only makes a caller-supplied UTC-to-TT policy explicit and
    /// reproducible for applications that start from civil time.
    ///
    /// For signed `TT - UTC` policies, use [`Instant::tt_from_utc_signed`].
    pub fn tt_from_utc(self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        if self.scale != TimeScale::Utc {
            return Err(TimeScaleConversionError::expected(
                TimeScale::Utc,
                self.scale,
            ));
        }

        Ok(self.with_time_scale_offset(TimeScale::Tt, delta_t.as_secs_f64()))
    }

    /// Converts a UTC-tagged instant to TT using a caller-supplied signed offset.
    ///
    /// `offset_seconds` must be the already-chosen signed `TT - UTC` offset in
    /// SI seconds. The helper intentionally does not model leap seconds or
    /// DUT1 by itself; it only makes a caller-supplied UTC-to-TT policy explicit
    /// and reproducible for applications that need a TT-tagged request surface.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    /// let converted = instant.tt_from_utc_signed(64.184).expect("UTC-tagged instant");
    ///
    /// assert_eq!(converted.scale, TimeScale::Tt);
    /// assert!(converted.julian_day.days() > instant.julian_day.days());
    /// ```
    pub fn tt_from_utc_signed(self, offset_seconds: f64) -> Result<Self, TimeScaleConversionError> {
        if self.scale != TimeScale::Utc {
            return Err(TimeScaleConversionError::expected(
                TimeScale::Utc,
                self.scale,
            ));
        }

        let offset_seconds = checked_time_scale_offset(offset_seconds)?;

        Ok(self.with_time_scale_offset(TimeScale::Tt, offset_seconds))
    }

    /// Converts a TT-tagged instant to TDB using a caller-supplied offset.
    ///
    /// `offset` must be the already-chosen `TDB - TT` offset in SI seconds.
    /// For signed TDB-TT policies, use [`Instant::tdb_from_tt_signed`]. The
    /// helper intentionally does not model relativistic terms by itself; it
    /// only makes a caller-supplied TT-to-TDB policy explicit and reproducible
    /// for applications that need a TDB-tagged request surface.
    pub fn tdb_from_tt(self, offset: Duration) -> Result<Self, TimeScaleConversionError> {
        if self.scale != TimeScale::Tt {
            return Err(TimeScaleConversionError::expected(
                TimeScale::Tt,
                self.scale,
            ));
        }

        Ok(self.with_time_scale_offset(TimeScale::Tdb, offset.as_secs_f64()))
    }

    /// Converts a TT-tagged instant to TDB using a caller-supplied signed offset.
    ///
    /// `offset_seconds` must be the already-chosen signed `TDB - TT` offset in
    /// SI seconds. The helper intentionally does not model relativistic terms by
    /// itself; it only makes a caller-supplied TT-to-TDB policy explicit and
    /// reproducible for applications that need a TDB-tagged request surface.
    pub fn tdb_from_tt_signed(self, offset_seconds: f64) -> Result<Self, TimeScaleConversionError> {
        if self.scale != TimeScale::Tt {
            return Err(TimeScaleConversionError::expected(
                TimeScale::Tt,
                self.scale,
            ));
        }

        let offset_seconds = checked_time_scale_offset(offset_seconds)?;

        Ok(self.with_time_scale_offset(TimeScale::Tdb, offset_seconds))
    }

    /// Converts a TDB-tagged instant to TT using a caller-supplied signed offset.
    ///
    /// `offset_seconds` must be the already-chosen signed `TT - TDB` offset in
    /// SI seconds. The helper intentionally does not model relativistic terms by
    /// itself; it only makes a caller-supplied TDB-to-TT policy explicit and
    /// reproducible for applications that need a TT-tagged request surface.
    pub fn tt_from_tdb(self, offset_seconds: f64) -> Result<Self, TimeScaleConversionError> {
        if self.scale != TimeScale::Tdb {
            return Err(TimeScaleConversionError::expected(
                TimeScale::Tdb,
                self.scale,
            ));
        }

        let offset_seconds = checked_time_scale_offset(offset_seconds)?;

        Ok(self.with_time_scale_offset(TimeScale::Tt, offset_seconds))
    }

    /// Converts a TDB-tagged instant to TT using a caller-supplied signed offset.
    ///
    /// This is an explicit alias for [`Instant::tt_from_tdb`] that mirrors the
    /// signed helper naming used for the other time-scale conversion policies.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let converted = instant.tt_from_tdb_signed(-0.001_657).expect("TDB-tagged instant");
    ///
    /// assert_eq!(converted.scale, TimeScale::Tt);
    /// assert!(converted.julian_day.days() < instant.julian_day.days());
    /// ```
    pub fn tt_from_tdb_signed(self, offset_seconds: f64) -> Result<Self, TimeScaleConversionError> {
        self.tt_from_tdb(offset_seconds)
    }

    /// Converts a UT1-tagged instant to TDB using caller-supplied TT-UT1 and
    /// TDB-TT offsets.
    ///
    /// `tt_offset` must be the already-chosen `TT - UT1` offset in SI
    /// seconds. `tdb_offset` must be the already-chosen `TDB - TT` offset in
    /// SI seconds. The helper intentionally does not model leap seconds,
    /// DUT1, or relativistic terms by itself; it only composes caller-supplied
    /// policy steps into a reproducible TDB-tagged instant.
    pub fn tdb_from_ut1(
        self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        let tt = self.tt_from_ut1(tt_offset)?;
        tt.tdb_from_tt(tdb_offset)
    }

    /// Converts a UT1-tagged instant to TDB using caller-supplied TT-UT1 and
    /// signed TDB-TT offsets.
    ///
    /// `tt_offset` must be the already-chosen `TT - UT1` offset in SI
    /// seconds. `tdb_offset_seconds` must be the already-chosen signed
    /// `TDB - TT` offset in SI seconds. The helper intentionally does not
    /// model leap seconds, DUT1, or relativistic terms by itself; it only
    /// composes caller-supplied policy steps into a reproducible TDB-tagged
    /// instant.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    /// let converted = instant
    ///     .tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
    ///     .expect("UT1-tagged instant");
    ///
    /// assert_eq!(converted.scale, TimeScale::Tdb);
    /// assert!(converted.julian_day.days() > instant.julian_day.days());
    /// ```
    pub fn tdb_from_ut1_signed(
        self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        let tt = self.tt_from_ut1(tt_offset)?;
        tt.tdb_from_tt_signed(tdb_offset_seconds)
    }

    /// Converts a UTC-tagged instant to TDB using caller-supplied TT-UTC and
    /// TDB-TT offsets.
    ///
    /// `tt_offset` must be the already-chosen `TT - UTC` offset in SI seconds.
    /// `tdb_offset` must be the already-chosen `TDB - TT` offset in SI
    /// seconds. The helper intentionally does not model leap seconds, DUT1, or
    /// relativistic terms by itself; it only composes caller-supplied policy
    /// steps into a reproducible TDB-tagged instant.
    pub fn tdb_from_utc(
        self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        let tt = self.tt_from_utc(tt_offset)?;
        tt.tdb_from_tt(tdb_offset)
    }

    /// Converts a UTC-tagged instant to TDB using caller-supplied TT-UTC and
    /// signed TDB-TT offsets.
    ///
    /// `tt_offset` must be the already-chosen `TT - UTC` offset in SI seconds.
    /// `tdb_offset_seconds` must be the already-chosen signed `TDB - TT`
    /// offset in SI seconds. The helper intentionally does not model leap
    /// seconds, DUT1, or relativistic terms by itself; it only composes
    /// caller-supplied policy steps into a reproducible TDB-tagged instant.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use pleiades_types::{Instant, JulianDay, TimeScale};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    /// let converted = instant
    ///     .tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
    ///     .expect("UTC-tagged instant");
    ///
    /// assert_eq!(converted.scale, TimeScale::Tdb);
    /// assert!(converted.julian_day.days() > instant.julian_day.days());
    /// ```
    pub fn tdb_from_utc_signed(
        self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        let tt = self.tt_from_utc(tt_offset)?;
        tt.tdb_from_tt_signed(tdb_offset_seconds)
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A geographic observer location.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ObserverLocation {
    /// Geographic latitude.
    pub latitude: Latitude,
    /// Geographic longitude, expressed in degrees east of Greenwich.
    pub longitude: Longitude,
    /// Optional elevation above sea level in meters.
    pub elevation_m: Option<f64>,
}

impl ObserverLocation {
    /// Creates a new observer location.
    pub const fn new(latitude: Latitude, longitude: Longitude, elevation_m: Option<f64>) -> Self {
        Self {
            latitude,
            longitude,
            elevation_m,
        }
    }

    /// Returns a compact one-line rendering of the observer location.
    pub fn summary_line(&self) -> String {
        let elevation = self
            .elevation_m
            .map(|value| format!("{value:.3} m"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "latitude={}, longitude={}, elevation={}",
            self.latitude, self.longitude, elevation
        )
    }
}

impl fmt::Display for ObserverLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// The coordinate frame requested from a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CoordinateFrame {
    /// Ecliptic longitude/latitude coordinates.
    Ecliptic,
    /// Equatorial right ascension/declination coordinates.
    Equatorial,
}

impl fmt::Display for CoordinateFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Ecliptic => "Ecliptic",
            Self::Equatorial => "Equatorial",
        };
        f.write_str(label)
    }
}

/// Whether coordinates should be interpreted in tropical or sidereal mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ZodiacMode {
    /// Tropical zodiac.
    Tropical,
    /// Sidereal zodiac using the selected ayanamsa definition.
    Sidereal { ayanamsa: Ayanamsa },
}

impl fmt::Display for ZodiacMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tropical => f.write_str("Tropical"),
            Self::Sidereal { ayanamsa } => write!(f, "Sidereal ({ayanamsa})"),
        }
    }
}

/// One of the twelve zodiac signs.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ZodiacSign {
    /// Aries 0°–30°.
    Aries,
    /// Taurus 30°–60°.
    Taurus,
    /// Gemini 60°–90°.
    Gemini,
    /// Cancer 90°–120°.
    Cancer,
    /// Leo 120°–150°.
    Leo,
    /// Virgo 150°–180°.
    Virgo,
    /// Libra 180°–210°.
    Libra,
    /// Scorpio 210°–240°.
    Scorpio,
    /// Sagittarius 240°–270°.
    Sagittarius,
    /// Capricorn 270°–300°.
    Capricorn,
    /// Aquarius 300°–330°.
    Aquarius,
    /// Pisces 330°–360°.
    Pisces,
}

impl ZodiacSign {
    /// Returns the sign corresponding to a normalized ecliptic longitude.
    pub fn from_longitude(longitude: Longitude) -> Self {
        match (longitude.degrees() / 30.0).floor() as usize % 12 {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            _ => Self::Pisces,
        }
    }

    /// Returns the sign's display name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aries => "Aries",
            Self::Taurus => "Taurus",
            Self::Gemini => "Gemini",
            Self::Cancer => "Cancer",
            Self::Leo => "Leo",
            Self::Virgo => "Virgo",
            Self::Libra => "Libra",
            Self::Scorpio => "Scorpio",
            Self::Sagittarius => "Sagittarius",
            Self::Capricorn => "Capricorn",
            Self::Aquarius => "Aquarius",
            Self::Pisces => "Pisces",
        }
    }
}

impl fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Whether a backend should prefer apparent or mean values where both exist.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Apparentness {
    /// Apparent values, including light-time and related corrections when available.
    Apparent,
    /// Mean values.
    Mean,
}

impl fmt::Display for Apparentness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Apparent => "Apparent",
            Self::Mean => "Mean",
        };
        f.write_str(label)
    }
}

/// The built-in and custom body identifiers recognized by the shared API.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CelestialBody {
    /// The Sun.
    Sun,
    /// The Moon.
    Moon,
    /// Mercury.
    Mercury,
    /// Venus.
    Venus,
    /// Mars.
    Mars,
    /// Jupiter.
    Jupiter,
    /// Saturn.
    Saturn,
    /// Uranus.
    Uranus,
    /// Neptune.
    Neptune,
    /// Pluto.
    Pluto,
    /// The mean lunar node.
    MeanNode,
    /// The true lunar node.
    TrueNode,
    /// The mean lunar apogee.
    MeanApogee,
    /// The true lunar apogee.
    TrueApogee,
    /// The mean lunar perigee.
    MeanPerigee,
    /// The true lunar perigee.
    TruePerigee,
    /// Ceres.
    Ceres,
    /// Pallas.
    Pallas,
    /// Juno.
    Juno,
    /// Vesta.
    Vesta,
    /// A body that is not yet one of the built-in identifiers.
    Custom(CustomBodyId),
}

impl CelestialBody {
    /// Returns a stable human-readable name for built-in bodies.
    pub const fn built_in_name(&self) -> Option<&'static str> {
        match self {
            Self::Sun => Some("Sun"),
            Self::Moon => Some("Moon"),
            Self::Mercury => Some("Mercury"),
            Self::Venus => Some("Venus"),
            Self::Mars => Some("Mars"),
            Self::Jupiter => Some("Jupiter"),
            Self::Saturn => Some("Saturn"),
            Self::Uranus => Some("Uranus"),
            Self::Neptune => Some("Neptune"),
            Self::Pluto => Some("Pluto"),
            Self::MeanNode => Some("Mean Node"),
            Self::TrueNode => Some("True Node"),
            Self::MeanApogee => Some("Mean Apogee"),
            Self::TrueApogee => Some("True Apogee"),
            Self::MeanPerigee => Some("Mean Perigee"),
            Self::TruePerigee => Some("True Perigee"),
            Self::Ceres => Some("Ceres"),
            Self::Pallas => Some("Pallas"),
            Self::Juno => Some("Juno"),
            Self::Vesta => Some("Vesta"),
            Self::Custom(_) => None,
        }
    }
}

/// A structured identifier for a custom body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomBodyId {
    /// A coarse namespace for the body source, such as `asteroid` or `hypothetical`.
    pub catalog: String,
    /// The designation within the namespace.
    pub designation: String,
}

impl CustomBodyId {
    /// Creates a new custom body identifier.
    pub fn new(catalog: impl Into<String>, designation: impl Into<String>) -> Self {
        Self {
            catalog: catalog.into(),
            designation: designation.into(),
        }
    }

    /// Validates the custom body identifier fields.
    ///
    /// The catalog and designation must both be non-empty, must not contain
    /// leading or trailing whitespace, and must not contain the `:` separator
    /// used by the display representation.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        validate_canonical_text("custom body id", "catalog", &self.catalog)?;
        validate_canonical_text("custom body id", "designation", &self.designation)?;

        if self.catalog.contains(':') {
            return Err(CustomDefinitionValidationError::contains_separator(
                "custom body id",
                "catalog",
                ':',
            ));
        }

        if self.designation.contains(':') {
            return Err(CustomDefinitionValidationError::contains_separator(
                "custom body id",
                "designation",
                ':',
            ));
        }

        Ok(())
    }
}

impl fmt::Display for CustomBodyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.catalog, self.designation)
    }
}

/// Validation failure for a custom body, house system, or ayanamsa definition.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CustomDefinitionValidationError {
    /// A required field was blank.
    Blank {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
    },
    /// A field carried leading or trailing whitespace.
    Padded {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
    },
    /// A field contained the `:` separator used by custom body identifiers.
    ContainsSeparator {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
        /// The separator that was rejected.
        separator: char,
    },
    /// A list of aliases contained a duplicate value.
    DuplicateAlias {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The duplicated alias.
        alias: String,
    },
    /// A label collides with a built-in descriptor or alias.
    ReservedLabel {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
        /// The reserved built-in label.
        label: String,
    },
    /// A numeric field was not finite.
    NonFinite {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
    },
    /// Two optional fields must be supplied together.
    IncompletePair {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The first required field.
        first: String,
        /// The second required field.
        second: String,
    },
}

impl CustomDefinitionValidationError {
    fn blank(subject: &'static str, field: impl Into<String>) -> Self {
        Self::Blank {
            subject,
            field: field.into(),
        }
    }

    fn padded(subject: &'static str, field: impl Into<String>) -> Self {
        Self::Padded {
            subject,
            field: field.into(),
        }
    }

    fn contains_separator(
        subject: &'static str,
        field: impl Into<String>,
        separator: char,
    ) -> Self {
        Self::ContainsSeparator {
            subject,
            field: field.into(),
            separator,
        }
    }

    fn duplicate_alias(subject: &'static str, alias: impl Into<String>) -> Self {
        Self::DuplicateAlias {
            subject,
            alias: alias.into(),
        }
    }

    fn reserved_label(
        subject: &'static str,
        field: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self::ReservedLabel {
            subject,
            field: field.into(),
            label: label.into(),
        }
    }

    fn non_finite(subject: &'static str, field: impl Into<String>) -> Self {
        Self::NonFinite {
            subject,
            field: field.into(),
        }
    }

    fn incomplete_pair(
        subject: &'static str,
        first: impl Into<String>,
        second: impl Into<String>,
    ) -> Self {
        Self::IncompletePair {
            subject,
            first: first.into(),
            second: second.into(),
        }
    }

    /// Returns a compact one-line rendering of the validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::Blank { subject, field } => format!("{subject} {field} must not be blank"),
            Self::Padded { subject, field } => {
                format!("{subject} {field} must not have leading or trailing whitespace")
            }
            Self::ContainsSeparator {
                subject,
                field,
                separator,
            } => format!("{subject} {field} must not contain '{separator}'"),
            Self::DuplicateAlias { subject, alias } => {
                format!("{subject} aliases must be unique: duplicate {alias}")
            }
            Self::ReservedLabel {
                subject,
                field,
                label,
            } => {
                format!("{subject} {field} must not match a built-in label: {label}")
            }
            Self::NonFinite { subject, field } => format!("{subject} {field} must be finite"),
            Self::IncompletePair {
                subject,
                first,
                second,
            } => format!("{subject} requires both {first} and {second} when one is present"),
        }
    }
}

impl fmt::Display for CustomDefinitionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for CustomDefinitionValidationError {}

fn validate_canonical_text(
    subject: &'static str,
    field: impl Into<String>,
    value: &str,
) -> Result<(), CustomDefinitionValidationError> {
    let field = field.into();
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CustomDefinitionValidationError::blank(subject, field));
    }

    if trimmed != value {
        return Err(CustomDefinitionValidationError::padded(subject, field));
    }

    Ok(())
}

impl fmt::Display for CelestialBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sun => f.write_str("Sun"),
            Self::Moon => f.write_str("Moon"),
            Self::Mercury => f.write_str("Mercury"),
            Self::Venus => f.write_str("Venus"),
            Self::Mars => f.write_str("Mars"),
            Self::Jupiter => f.write_str("Jupiter"),
            Self::Saturn => f.write_str("Saturn"),
            Self::Uranus => f.write_str("Uranus"),
            Self::Neptune => f.write_str("Neptune"),
            Self::Pluto => f.write_str("Pluto"),
            Self::MeanNode => f.write_str("Mean Node"),
            Self::TrueNode => f.write_str("True Node"),
            Self::MeanApogee => f.write_str("Mean Apogee"),
            Self::TrueApogee => f.write_str("True Apogee"),
            Self::MeanPerigee => f.write_str("Mean Perigee"),
            Self::TruePerigee => f.write_str("True Perigee"),
            Self::Ceres => f.write_str("Ceres"),
            Self::Pallas => f.write_str("Pallas"),
            Self::Juno => f.write_str("Juno"),
            Self::Vesta => f.write_str("Vesta"),
            Self::Custom(custom) => fmt::Display::fmt(custom, f),
        }
    }
}

impl CelestialBody {
    /// Validates the custom body identifier when this is a custom body.
    ///
    /// Built-in bodies are always valid; only the structured custom body
    /// identifier is checked. This keeps user-defined catalog entries from
    /// drifting into request or profile surfaces with blank or padded fields.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate(),
            _ => Ok(()),
        }
    }
}

/// A built-in or custom house system selection.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HouseSystem {
    /// Placidus.
    Placidus,
    /// Koch.
    Koch,
    /// Porphyry.
    Porphyry,
    /// Regiomontanus.
    Regiomontanus,
    /// Campanus.
    Campanus,
    /// Carter (poli-equatorial) houses.
    Carter,
    /// Horizon/Azimuth houses.
    Horizon,
    /// APC houses.
    Apc,
    /// Krusinski-Pisa-Goelzer houses.
    KrusinskiPisaGoelzer,
    /// Equal houses.
    Equal,
    /// Equal houses with the Midheaven on cusp 10.
    EqualMidheaven,
    /// Equal houses with the first house anchored at 0° Aries.
    EqualAries,
    /// Vehlow equal houses, with the Ascendant centered in house 1.
    Vehlow,
    /// Sripati houses.
    Sripati,
    /// Whole sign houses.
    WholeSign,
    /// Alcabitius.
    Alcabitius,
    /// Albategnius / Savard-A.
    Albategnius,
    /// Pullen sinusoidal delta (Neo-Porphyry).
    PullenSd,
    /// Pullen sinusoidal ratio.
    PullenSr,
    /// Meridian-style systems.
    Meridian,
    /// Axial variants documented by specific software.
    Axial,
    /// Topocentric (Polich-Page).
    Topocentric,
    /// Morinus.
    Morinus,
    /// Sunshine (Bob Makransky / Dieter Treindl family).
    Sunshine,
    /// Gauquelin sectors.
    Gauquelin,
    /// A custom house system definition.
    Custom(CustomHouseSystem),
}

impl fmt::Display for HouseSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Placidus => f.write_str("Placidus"),
            Self::Koch => f.write_str("Koch"),
            Self::Porphyry => f.write_str("Porphyry"),
            Self::Regiomontanus => f.write_str("Regiomontanus"),
            Self::Campanus => f.write_str("Campanus"),
            Self::Carter => f.write_str("Carter (poli-equatorial)"),
            Self::Horizon => f.write_str("Horizon/Azimuth"),
            Self::Apc => f.write_str("APC"),
            Self::KrusinskiPisaGoelzer => f.write_str("Krusinski-Pisa-Goelzer"),
            Self::Equal => f.write_str("Equal"),
            Self::EqualMidheaven => f.write_str("Equal (MC)"),
            Self::EqualAries => f.write_str("Equal (1=Aries)"),
            Self::Vehlow => f.write_str("Vehlow Equal"),
            Self::Sripati => f.write_str("Sripati"),
            Self::WholeSign => f.write_str("Whole Sign"),
            Self::Alcabitius => f.write_str("Alcabitius"),
            Self::Albategnius => f.write_str("Albategnius"),
            Self::PullenSd => f.write_str("Pullen SD"),
            Self::PullenSr => f.write_str("Pullen SR"),
            Self::Meridian => f.write_str("Meridian"),
            Self::Axial => f.write_str("Axial"),
            Self::Topocentric => f.write_str("Topocentric"),
            Self::Morinus => f.write_str("Morinus"),
            Self::Sunshine => f.write_str("Sunshine"),
            Self::Gauquelin => f.write_str("Gauquelin sectors"),
            Self::Custom(custom) => fmt::Display::fmt(custom, f),
        }
    }
}

impl HouseSystem {
    /// Validates the custom house-system definition when this is a custom variant.
    ///
    /// Built-in house systems are always valid; only structured custom labels,
    /// aliases, and notes are checked. This keeps malformed catalog entries
    /// from leaking into request summaries or release profiles.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate(),
            _ => Ok(()),
        }
    }

    /// Validates the custom house-system definition against reserved built-in labels.
    ///
    /// Built-in house systems are always valid; custom entries are checked with
    /// the same structural rules as [`validate`](Self::validate) and are then
    /// compared against a caller-supplied built-in label resolver.
    pub fn validate_against_reserved_labels(
        &self,
        is_reserved_label: impl Fn(&str) -> bool,
    ) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate_against_reserved_labels(is_reserved_label),
            _ => Ok(()),
        }
    }
}

/// A structured custom house-system definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomHouseSystem {
    /// Stable human-readable name.
    pub name: String,
    /// Optional alternative names or aliases.
    pub aliases: Vec<String>,
    /// Optional notes about formula, assumptions, or limits.
    pub notes: Option<String>,
}

impl CustomHouseSystem {
    /// Creates a custom house-system definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aliases: Vec::new(),
            notes: None,
        }
    }

    /// Validates the custom house-system definition.
    ///
    /// The canonical name must be non-empty and free of leading or trailing
    /// whitespace. Aliases and notes, when present, must follow the same
    /// canonical-text rule, and aliases must be unique after ASCII
    /// case-normalization, including against the canonical name.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        validate_canonical_text("custom house system", "name", &self.name)?;

        let mut aliases: Vec<&str> = Vec::with_capacity(self.aliases.len());
        for (index, alias) in self.aliases.iter().enumerate() {
            let field = format!("alias[{index}]");
            validate_canonical_text("custom house system", field, alias)?;

            if self.name.eq_ignore_ascii_case(alias)
                || aliases
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(alias))
            {
                return Err(CustomDefinitionValidationError::duplicate_alias(
                    "custom house system",
                    alias,
                ));
            }

            aliases.push(alias);
        }

        if let Some(notes) = &self.notes {
            validate_canonical_text("custom house system", "notes", notes)?;
        }

        Ok(())
    }

    /// Validates the custom house-system definition against reserved built-in labels.
    ///
    /// This is a stricter form of [`validate`](Self::validate) that also rejects
    /// a canonical name or alias that collides with a published built-in house-system
    /// label. That keeps user-defined catalog entries distinguishable from the
    /// built-in compatibility catalog when request validation or release profiles
    /// surface them.
    pub fn validate_against_reserved_labels(
        &self,
        is_reserved_label: impl Fn(&str) -> bool,
    ) -> Result<(), CustomDefinitionValidationError> {
        self.validate()?;

        if is_reserved_label(&self.name) {
            return Err(CustomDefinitionValidationError::reserved_label(
                "custom house system",
                "name",
                &self.name,
            ));
        }

        for (index, alias) in self.aliases.iter().enumerate() {
            if is_reserved_label(alias) {
                return Err(CustomDefinitionValidationError::reserved_label(
                    "custom house system",
                    format!("alias[{index}]"),
                    alias,
                ));
            }
        }

        Ok(())
    }
}

impl fmt::Display for CustomHouseSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;

        if !self.aliases.is_empty() {
            write!(f, " [aliases: {}]", self.aliases.join(", "))?;
        }

        if let Some(notes) = &self.notes {
            write!(f, " ({notes})")?;
        }

        Ok(())
    }
}

/// A built-in or custom ayanamsa selection.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Ayanamsa {
    /// Lahiri.
    Lahiri,
    /// Raman.
    Raman,
    /// Krishnamurti.
    Krishnamurti,
    /// Fagan/Bradley.
    FaganBradley,
    /// True Chitra.
    TrueChitra,
    /// True Citra.
    TrueCitra,
    /// Suryasiddhanta (Revati) / SS Revati.
    SuryasiddhantaRevati,
    /// Suryasiddhanta (Citra) / SS Citra.
    SuryasiddhantaCitra,
    /// J2000.0 reference-frame mode.
    J2000,
    /// J1900.0 reference-frame mode.
    J1900,
    /// B1950.0 reference-frame mode.
    B1950,
    /// True Revati.
    TrueRevati,
    /// True Mula.
    TrueMula,
    /// True Pushya.
    TruePushya,
    /// Udayagiri.
    Udayagiri,
    /// Djwhal Khul.
    DjwhalKhul,
    /// J. N. Bhasin.
    JnBhasin,
    /// Suryasiddhanta mean-sun variant.
    Suryasiddhanta499MeanSun,
    /// Aryabhata mean-sun variant.
    Aryabhata499MeanSun,
    /// The 1956 Indian Astronomical Ephemeris / ICRC Lahiri definition.
    LahiriIcrc,
    /// Lahiri's 1940 zero-date variant.
    Lahiri1940,
    /// Usha/Shashi, anchored to the Revati tradition.
    UshaShashi,
    /// Suryasiddhanta-equinox variant anchored in 499 CE.
    Suryasiddhanta499,
    /// Aryabhata-equinox variant anchored in 499 CE.
    Aryabhata499,
    /// Sassanian zero-point variant anchored in 564 CE.
    Sassanian,
    /// DeLuce ayanamsa.
    DeLuce,
    /// Yukteshwar ayanamsa.
    Yukteshwar,
    /// P.V.R. Narasimha Rao's Pushya-paksha ayanamsa.
    PvrPushyaPaksha,
    /// Sheoran ayanamsa.
    Sheoran,
    /// Hipparchus / Hipparchos ayanamsa.
    Hipparchus,
    /// Babylonian (Kugler 1).
    BabylonianKugler1,
    /// Babylonian (Kugler 2).
    BabylonianKugler2,
    /// Babylonian (Kugler 3).
    BabylonianKugler3,
    /// Babylonian (Huber).
    BabylonianHuber,
    /// Babylonian (Eta Piscium).
    BabylonianEtaPiscium,
    /// Babylonian (Aldebaran / 15 Tau).
    BabylonianAldebaran,
    /// Babylonian (House).
    BabylonianHouse,
    /// Babylonian (Sissy).
    BabylonianSissy,
    /// Babylonian (True Geoc).
    BabylonianTrueGeoc,
    /// Babylonian (True Topc).
    BabylonianTrueTopc,
    /// Babylonian (True Obs).
    BabylonianTrueObs,
    /// Babylonian (House Obs).
    BabylonianHouseObs,
    /// Babylonian (Britton).
    BabylonianBritton,
    /// Aryabhata (522 CE).
    Aryabhata522,
    /// Lahiri (VP285).
    LahiriVP285,
    /// Krishnamurti (VP291).
    KrishnamurtiVP291,
    /// True Sheoran.
    TrueSheoran,
    /// Galactic Center.
    GalacticCenter,
    /// Galactic Center (Rgilbrand).
    GalacticCenterRgilbrand,
    /// Galactic Center (Mardyks).
    GalacticCenterMardyks,
    /// Galactic Center (Mula/Wilhelm).
    GalacticCenterMulaWilhelm,
    /// Dhruva Galactic Center (Middle Mula).
    DhruvaGalacticCenterMula,
    /// Galactic Center (Cochrane).
    GalacticCenterCochrane,
    /// Galactic Equator.
    GalacticEquator,
    /// Galactic Equator (IAU 1958).
    GalacticEquatorIau1958,
    /// Galactic Equator (True).
    GalacticEquatorTrue,
    /// Galactic Equator (Mula).
    GalacticEquatorMula,
    /// Galactic Equator (Fiorenza).
    GalacticEquatorFiorenza,
    /// Valens Moon.
    ValensMoon,
    /// A custom ayanamsa formula or offset table.
    Custom(CustomAyanamsa),
}

impl fmt::Display for Ayanamsa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(custom) => write!(f, "{custom}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl Ayanamsa {
    /// Validates the custom ayanamsa definition when this is a custom variant.
    ///
    /// Built-in ayanamsas are always valid; only structured custom labels and
    /// offset metadata are checked. This keeps malformed sidereal definitions
    /// from leaking into request validation or release-facing text.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate(),
            _ => Ok(()),
        }
    }

    /// Validates the custom ayanamsa definition against reserved built-in labels.
    ///
    /// Built-in ayanamsas are always valid; custom entries are checked with the
    /// same structural rules as [`validate`](Self::validate) and are then compared
    /// against a caller-supplied built-in label resolver.
    pub fn validate_against_reserved_labels(
        &self,
        is_reserved_label: impl Fn(&str) -> bool,
    ) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate_against_reserved_labels(is_reserved_label),
            _ => Ok(()),
        }
    }
}

/// A structured custom ayanamsa definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct CustomAyanamsa {
    /// Stable human-readable name.
    pub name: String,
    /// Optional description of the formula or offset policy.
    pub description: Option<String>,
    /// Optional epoch the definition is tied to.
    pub epoch: Option<JulianDay>,
    /// Optional fixed offset in degrees for simple offset-based variants.
    pub offset_degrees: Option<Angle>,
}

impl CustomAyanamsa {
    /// Creates a custom ayanamsa definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            epoch: None,
            offset_degrees: None,
        }
    }

    /// Validates the custom ayanamsa definition.
    ///
    /// The canonical name and optional description must be non-empty and free
    /// of leading or trailing whitespace. If one of `epoch` or
    /// `offset_degrees` is supplied, the other must be supplied too so the
    /// sidereal offset can be reconstructed deterministically. Any supplied
    /// numeric values must also be finite.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        validate_canonical_text("custom ayanamsa", "name", &self.name)?;

        if let Some(description) = &self.description {
            validate_canonical_text("custom ayanamsa", "description", description)?;
        }

        match (self.epoch, self.offset_degrees) {
            (Some(epoch), Some(offset_degrees)) => {
                if !epoch.days().is_finite() {
                    return Err(CustomDefinitionValidationError::non_finite(
                        "custom ayanamsa",
                        "epoch",
                    ));
                }

                if !offset_degrees.is_finite() {
                    return Err(CustomDefinitionValidationError::non_finite(
                        "custom ayanamsa",
                        "offset_degrees",
                    ));
                }
            }
            (None, None) => {}
            (Some(_), None) => {
                return Err(CustomDefinitionValidationError::incomplete_pair(
                    "custom ayanamsa",
                    "epoch",
                    "offset_degrees",
                ));
            }
            (None, Some(_)) => {
                return Err(CustomDefinitionValidationError::incomplete_pair(
                    "custom ayanamsa",
                    "offset_degrees",
                    "epoch",
                ));
            }
        }

        Ok(())
    }

    /// Validates the custom ayanamsa definition against reserved built-in labels.
    ///
    /// This is a stricter form of [`validate`](Self::validate) that also rejects
    /// a canonical name that collides with a published built-in ayanamsa label.
    /// That keeps user-defined sidereal definitions distinguishable from the
    /// built-in compatibility catalog when request validation or release profiles
    /// surface them.
    pub fn validate_against_reserved_labels(
        &self,
        is_reserved_label: impl Fn(&str) -> bool,
    ) -> Result<(), CustomDefinitionValidationError> {
        self.validate()?;

        if is_reserved_label(&self.name) {
            return Err(CustomDefinitionValidationError::reserved_label(
                "custom ayanamsa",
                "name",
                &self.name,
            ));
        }

        Ok(())
    }
}

impl fmt::Display for CustomAyanamsa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;

        let mut details = Vec::new();
        if let Some(epoch) = self.epoch {
            details.push(format!("epoch: {epoch}"));
        }
        if let Some(offset_degrees) = self.offset_degrees {
            details.push(format!("offset: {offset_degrees}"));
        }
        if let Some(description) = &self.description {
            details.push(description.clone());
        }

        if !details.is_empty() {
            write!(f, " [{}]", details.join(", "))?;
        }

        Ok(())
    }
}

/// Ecliptic position data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EclipticCoordinates {
    /// Ecliptic longitude.
    pub longitude: Longitude,
    /// Ecliptic latitude.
    pub latitude: Latitude,
    /// Distance in astronomical units when available.
    pub distance_au: Option<f64>,
}

impl EclipticCoordinates {
    /// Creates a new ecliptic coordinate sample.
    pub const fn new(longitude: Longitude, latitude: Latitude, distance_au: Option<f64>) -> Self {
        Self {
            longitude,
            latitude,
            distance_au,
        }
    }

    /// Converts this ecliptic position into an equatorial position using the supplied obliquity.
    ///
    /// The transform is a pure geometric rotation: longitude/latitude are interpreted in the
    /// ecliptic frame, right ascension is normalized into `[0, 360)`, declination is signed, and
    /// any available distance is preserved.
    pub fn to_equatorial(self, obliquity: Angle) -> EquatorialCoordinates {
        let longitude = self.longitude.degrees().to_radians();
        let latitude = self.latitude.degrees().to_radians();
        let obliquity = obliquity.radians();

        let x = longitude.cos() * latitude.cos();
        let y =
            longitude.sin() * latitude.cos() * obliquity.cos() - latitude.sin() * obliquity.sin();
        let z =
            longitude.sin() * latitude.cos() * obliquity.sin() + latitude.sin() * obliquity.cos();

        EquatorialCoordinates::new(
            Angle::from_degrees(y.atan2(x).to_degrees()).normalized_0_360(),
            Latitude::from_degrees(z.atan2((x * x + y * y).sqrt()).to_degrees()),
            self.distance_au,
        )
    }
}

/// Equatorial position data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EquatorialCoordinates {
    /// Right ascension.
    pub right_ascension: Angle,
    /// Declination.
    pub declination: Latitude,
    /// Distance in astronomical units when available.
    pub distance_au: Option<f64>,
}

impl EquatorialCoordinates {
    /// Creates a new equatorial coordinate sample.
    pub const fn new(
        right_ascension: Angle,
        declination: Latitude,
        distance_au: Option<f64>,
    ) -> Self {
        Self {
            right_ascension,
            declination,
            distance_au,
        }
    }

    /// Validates that the sample is finite and normalized enough for release-facing frame checks.
    pub fn validate(&self) -> Result<(), CoordinateValidationError> {
        validate_finite_coordinate_value(
            "equatorial",
            "right_ascension",
            self.right_ascension.degrees(),
        )?;
        validate_right_ascension_range(self.right_ascension.degrees())?;
        validate_finite_coordinate_value("equatorial", "declination", self.declination.degrees())?;
        validate_latitude_range("equatorial", "declination", self.declination.degrees())?;
        if let Some(distance_au) = self.distance_au {
            validate_distance("equatorial", distance_au)?;
        }
        Ok(())
    }

    /// Converts this equatorial position back into an ecliptic position using the supplied obliquity.
    ///
    /// The transform is the inverse geometric rotation of [`EclipticCoordinates::to_equatorial`]:
    /// right ascension is interpreted as a normalized angle, declination is signed, and any
    /// available distance is preserved.
    pub fn to_ecliptic(self, obliquity: Angle) -> EclipticCoordinates {
        let right_ascension = self.right_ascension.degrees().to_radians();
        let declination = self.declination.degrees().to_radians();
        let obliquity = obliquity.radians();

        let x = right_ascension.cos() * declination.cos();
        let y = right_ascension.sin() * declination.cos() * obliquity.cos()
            + declination.sin() * obliquity.sin();
        let z = -right_ascension.sin() * declination.cos() * obliquity.sin()
            + declination.sin() * obliquity.cos();

        EclipticCoordinates::new(
            Longitude::from_degrees(y.atan2(x).to_degrees()),
            Latitude::from_degrees(z.atan2((x * x + y * y).sqrt()).to_degrees()),
            self.distance_au,
        )
    }
}

/// Validation errors for shared coordinate samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordinateValidationError {
    /// A stored numeric field was not finite.
    NonFiniteValue {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The field that failed validation.
        field: &'static str,
        /// The offending value.
        value: f64,
    },
    /// A stored latitude-like field fell outside the expected closed range.
    LatitudeOutOfRange {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The field that failed validation.
        field: &'static str,
        /// The offending value.
        value: f64,
    },
    /// A stored right-ascension field fell outside the expected half-open range.
    RightAscensionOutOfRange {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The field that failed validation.
        field: &'static str,
        /// The offending value.
        value: f64,
    },
    /// A stored distance channel was negative.
    NegativeDistance {
        /// The coordinate family being validated.
        coordinate: &'static str,
        /// The offending value.
        value: f64,
    },
}

impl CoordinateValidationError {
    /// Returns a compact one-line rendering of the validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonFiniteValue {
                coordinate,
                field,
                value,
            } => format!("{coordinate} coordinate field `{field}` must be finite, got {value}"),
            Self::LatitudeOutOfRange {
                coordinate,
                field,
                value,
            } => format!(
                "{coordinate} coordinate field `{field}` must stay within [-90, 90], got {value}"
            ),
            Self::RightAscensionOutOfRange {
                coordinate,
                field,
                value,
            } => format!(
                "{coordinate} coordinate field `{field}` must stay within [0, 360), got {value}"
            ),
            Self::NegativeDistance { coordinate, value } => format!(
                "{coordinate} coordinate field `distance_au` must be non-negative, got {value}"
            ),
        }
    }
}

impl fmt::Display for CoordinateValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for CoordinateValidationError {}

fn validate_finite_coordinate_value(
    coordinate: &'static str,
    field: &'static str,
    value: f64,
) -> Result<(), CoordinateValidationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(CoordinateValidationError::NonFiniteValue {
            coordinate,
            field,
            value,
        })
    }
}

fn validate_latitude_range(
    coordinate: &'static str,
    field: &'static str,
    value: f64,
) -> Result<(), CoordinateValidationError> {
    if (-90.0..=90.0).contains(&value) {
        Ok(())
    } else {
        Err(CoordinateValidationError::LatitudeOutOfRange {
            coordinate,
            field,
            value,
        })
    }
}

fn validate_right_ascension_range(value: f64) -> Result<(), CoordinateValidationError> {
    if (0.0..360.0).contains(&value) {
        Ok(())
    } else {
        Err(CoordinateValidationError::RightAscensionOutOfRange {
            coordinate: "equatorial",
            field: "right_ascension",
            value,
        })
    }
}

fn validate_distance(
    coordinate: &'static str,
    value: f64,
) -> Result<(), CoordinateValidationError> {
    if !value.is_finite() {
        return Err(CoordinateValidationError::NonFiniteValue {
            coordinate,
            field: "distance_au",
            value,
        });
    }
    if value < 0.0 {
        Err(CoordinateValidationError::NegativeDistance { coordinate, value })
    } else {
        Ok(())
    }
}

impl EclipticCoordinates {
    /// Validates that the sample is finite and physically sensible for frame conversions.
    pub fn validate(&self) -> Result<(), CoordinateValidationError> {
        validate_finite_coordinate_value("ecliptic", "longitude", self.longitude.degrees())?;
        validate_finite_coordinate_value("ecliptic", "latitude", self.latitude.degrees())?;
        validate_latitude_range("ecliptic", "latitude", self.latitude.degrees())?;
        if let Some(distance_au) = self.distance_au {
            validate_distance("ecliptic", distance_au)?;
        }
        Ok(())
    }
}

/// The coarse direction of longitudinal motion.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MotionDirection {
    /// Motion is prograde or direct.
    Direct,
    /// Motion is effectively stationary at the chosen precision.
    Stationary,
    /// Motion is retrograde.
    Retrograde,
}

impl fmt::Display for MotionDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Direct => "Direct",
            Self::Stationary => "Stationary",
            Self::Retrograde => "Retrograde",
        };
        f.write_str(label)
    }
}

/// Apparent motion data for a position sample.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// Longitude speed in degrees per day.
    pub longitude_deg_per_day: Option<f64>,
    /// Latitude speed in degrees per day.
    pub latitude_deg_per_day: Option<f64>,
    /// Distance speed in astronomical units per day.
    pub distance_au_per_day: Option<f64>,
}

/// Errors returned when motion samples contain non-finite values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionValidationError {
    /// A motion component contained a non-finite value.
    NonFiniteSpeed {
        /// Motion field name.
        field: &'static str,
        /// Observed value.
        value: f64,
    },
}

impl fmt::Display for MotionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteSpeed { field, value } => {
                write!(f, "motion field `{field}` must be finite, got {value}")
            }
        }
    }
}

impl std::error::Error for MotionValidationError {}

impl Motion {
    /// Creates a new motion sample.
    pub const fn new(
        longitude_deg_per_day: Option<f64>,
        latitude_deg_per_day: Option<f64>,
        distance_au_per_day: Option<f64>,
    ) -> Self {
        Self {
            longitude_deg_per_day,
            latitude_deg_per_day,
            distance_au_per_day,
        }
    }

    /// Returns the longitudinal motion speed when available.
    pub const fn longitude_speed(self) -> Option<f64> {
        self.longitude_deg_per_day
    }

    /// Returns the latitudinal motion speed when available.
    pub const fn latitude_speed(self) -> Option<f64> {
        self.latitude_deg_per_day
    }

    /// Returns the radial motion speed when available.
    pub const fn distance_speed(self) -> Option<f64> {
        self.distance_au_per_day
    }

    /// Returns a compact one-line summary of the motion sample.
    pub fn summary_line(self) -> String {
        let longitude = self
            .longitude_speed()
            .map(|value| format!("{value} deg/day"))
            .unwrap_or_else(|| "n/a".to_string());
        let latitude = self
            .latitude_speed()
            .map(|value| format!("{value} deg/day"))
            .unwrap_or_else(|| "n/a".to_string());
        let distance = self
            .distance_speed()
            .map(|value| format!("{value} au/day"))
            .unwrap_or_else(|| "n/a".to_string());

        format!("longitude={longitude}; latitude={latitude}; distance={distance}")
    }

    /// Validates that every populated motion component is finite.
    pub fn validate(self) -> Result<(), MotionValidationError> {
        for (field, value) in [
            ("longitude_deg_per_day", self.longitude_deg_per_day),
            ("latitude_deg_per_day", self.latitude_deg_per_day),
            ("distance_au_per_day", self.distance_au_per_day),
        ] {
            if let Some(value) = value {
                if !value.is_finite() {
                    return Err(MotionValidationError::NonFiniteSpeed { field, value });
                }
            }
        }

        Ok(())
    }

    /// Returns the coarse longitudinal motion direction when that speed is available.
    ///
    /// The classification is sign-based: positive speed is direct, negative speed is retrograde,
    /// and an exact zero speed is stationary. Non-finite longitudinal speeds are treated as
    /// unknown until the sample is validated.
    pub fn longitude_direction(self) -> Option<MotionDirection> {
        let speed = self.longitude_speed()?;
        if !speed.is_finite() {
            return None;
        }

        Some(if speed > 0.0 {
            MotionDirection::Direct
        } else if speed < 0.0 {
            MotionDirection::Retrograde
        } else {
            MotionDirection::Stationary
        })
    }
}

impl fmt::Display for Motion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A Julian-day interval.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    /// Inclusive lower bound.
    pub start: Option<Instant>,
    /// Inclusive upper bound.
    pub end: Option<Instant>,
}

impl TimeRange {
    /// Creates a new time range.
    pub const fn new(start: Option<Instant>, end: Option<Instant>) -> Self {
        Self { start, end }
    }

    /// Returns `true` if the given instant is inside the range.
    pub fn contains(&self, instant: Instant) -> bool {
        let after_start = self.start.is_none_or(|start| {
            same_scale_and_jd(instant, start)
                && instant.julian_day.days() >= start.julian_day.days()
        });
        let before_end = self.end.is_none_or(|end| {
            same_scale_and_jd(instant, end) && instant.julian_day.days() <= end.julian_day.days()
        });
        after_start && before_end
    }

    /// Validates the range bounds and ordering.
    ///
    /// Unbounded ranges are valid. When one or both bounds are present, the
    /// finite Julian-day requirement applies to each bound, the two bounds must
    /// use the same time scale, and the upper bound must not precede the lower
    /// bound.
    pub fn validate(self) -> Result<(), TimeRangeValidationError> {
        if let Some(start) = self.start {
            if !start.julian_day.days().is_finite() {
                return Err(TimeRangeValidationError::non_finite_bound("start", start));
            }
        }
        if let Some(end) = self.end {
            if !end.julian_day.days().is_finite() {
                return Err(TimeRangeValidationError::non_finite_bound("end", end));
            }
        }
        if let (Some(start), Some(end)) = (self.start, self.end) {
            if start.scale != end.scale {
                return Err(TimeRangeValidationError::scale_mismatch(start, end));
            }
            if start.julian_day.days() > end.julian_day.days() {
                return Err(TimeRangeValidationError::out_of_order(start, end));
            }
        }

        Ok(())
    }

    /// Returns a compact one-line rendering of the range.
    pub fn summary_line(&self) -> String {
        match (self.start, self.end) {
            (Some(start), Some(end)) => format!(
                "{} → {}",
                format_time_range_instant(start),
                format_time_range_instant(end)
            ),
            (Some(start), None) => format!("from {}", format_time_range_instant(start)),
            (None, Some(end)) => format!("through {}", format_time_range_instant(end)),
            (None, None) => "unbounded".to_string(),
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn format_time_range_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

fn same_scale_and_jd(a: Instant, b: Instant) -> bool {
    a.scale == b.scale && a.julian_day.days().is_finite() && b.julian_day.days().is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_normalization_wraps_correctly() {
        assert_eq!(
            Angle::from_degrees(-30.0).normalized_0_360().degrees(),
            330.0
        );
        assert_eq!(
            Angle::from_degrees(390.0).normalized_0_360().degrees(),
            30.0
        );
        assert_eq!(
            Angle::from_degrees(190.0).normalized_signed().degrees(),
            -170.0
        );
    }

    #[test]
    fn longitude_is_always_normalized() {
        assert_eq!(Longitude::from_degrees(390.0).degrees(), 30.0);
        assert_eq!(Longitude::from(Angle::from_degrees(-30.0)).degrees(), 330.0);
    }

    #[test]
    fn ecliptic_to_equatorial_preserves_zero_obliquity_identity() {
        let ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(123.45),
            Latitude::from_degrees(-6.75),
            Some(0.123),
        );

        assert_eq!(ecliptic.validate(), Ok(()));

        let equatorial = ecliptic.to_equatorial(Angle::from_degrees(0.0));

        assert_eq!(equatorial.validate(), Ok(()));
        assert_eq!(equatorial.right_ascension.degrees(), 123.45);
        assert!((equatorial.declination.degrees() + 6.75).abs() < 1e-12);
        assert_eq!(equatorial.distance_au, Some(0.123));
    }

    #[test]
    fn ecliptic_to_equatorial_rotates_by_mean_obliquity() {
        let ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(90.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        );

        assert_eq!(ecliptic.validate(), Ok(()));

        let equatorial = ecliptic.to_equatorial(Angle::from_degrees(23.439_291_11));

        assert_eq!(equatorial.validate(), Ok(()));
        assert_eq!(equatorial.right_ascension.degrees(), 90.0);
        assert!((equatorial.declination.degrees() - 23.439_291_11).abs() < 1e-10);
        assert_eq!(equatorial.distance_au, Some(1.0));
    }

    #[test]
    fn instant_mean_obliquity_matches_the_shared_cubic_approximation() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);

        assert_eq!(instant.mean_obliquity().degrees(), 23.439_291_111_111_11);
    }

    #[test]
    fn instant_has_a_compact_display() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);

        assert_eq!(instant.summary_line(), "JD 2451545 TDB");
        assert_eq!(instant.to_string(), "JD 2451545 TDB");
    }

    #[test]
    fn observer_location_has_a_compact_display() {
        let observer = ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(35.75),
        );

        assert_eq!(
            observer.summary_line(),
            "latitude=51.5°, longitude=359.9°, elevation=35.750 m"
        );
        assert_eq!(observer.to_string(), observer.summary_line());
    }

    #[test]
    fn equatorial_to_ecliptic_round_trip_uses_the_same_obliquity() {
        let ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(123.45),
            Latitude::from_degrees(-6.75),
            Some(0.123),
        );
        let obliquity = Angle::from_degrees(23.439_291_11);

        assert_eq!(ecliptic.validate(), Ok(()));

        let equatorial = ecliptic.to_equatorial(obliquity);
        assert_eq!(equatorial.validate(), Ok(()));
        let round_trip = equatorial.to_ecliptic(obliquity);

        assert_eq!(round_trip.validate(), Ok(()));
        assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
        assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
        assert_eq!(round_trip.distance_au, Some(0.123));
    }

    #[test]
    fn mean_obliquity_round_trip_stays_stable_across_quadrants() {
        let cases = [
            (
                EclipticCoordinates::new(
                    Longitude::from_degrees(359.2),
                    Latitude::from_degrees(11.75),
                    None,
                ),
                Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
            ),
            (
                EclipticCoordinates::new(
                    Longitude::from_degrees(27.5),
                    Latitude::from_degrees(-33.25),
                    Some(2.5),
                ),
                Instant::new(JulianDay::from_days(2_459_000.5), TimeScale::Tt),
            ),
        ];

        for (ecliptic, instant) in cases {
            assert_eq!(ecliptic.validate(), Ok(()));
            let obliquity = instant.mean_obliquity();
            let equatorial = ecliptic.to_equatorial(obliquity);
            assert_eq!(equatorial.validate(), Ok(()));
            let round_trip = equatorial.to_ecliptic(obliquity);

            assert_eq!(round_trip.validate(), Ok(()));
            assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
            assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
            assert_eq!(round_trip.distance_au, ecliptic.distance_au);
        }
    }

    #[test]
    fn built_in_body_names_are_stable() {
        assert_eq!(CelestialBody::Sun.built_in_name(), Some("Sun"));
        assert_eq!(CelestialBody::Sun.to_string(), "Sun");
        assert_eq!(
            CelestialBody::MeanApogee.built_in_name(),
            Some("Mean Apogee")
        );
        assert_eq!(CelestialBody::TruePerigee.to_string(), "True Perigee");
        assert_eq!(
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")).built_in_name(),
            None
        );
        assert_eq!(
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")).to_string(),
            "asteroid:433-Eros"
        );
    }

    #[test]
    fn custom_body_id_validate_rejects_blank_padding_and_separators() {
        assert_eq!(
            CustomBodyId::new("", "433-Eros")
                .validate()
                .expect_err("blank catalogs should be rejected")
                .to_string(),
            "custom body id catalog must not be blank"
        );

        assert_eq!(
            CustomBodyId::new("asteroid", " 433-Eros ")
                .validate()
                .expect_err("padded designations should be rejected")
                .to_string(),
            "custom body id designation must not have leading or trailing whitespace"
        );

        assert_eq!(
            CustomBodyId::new("asteroid:catalog", "433-Eros")
                .validate()
                .expect_err("separator characters should be rejected")
                .to_string(),
            "custom body id catalog must not contain ':'"
        );
    }

    #[test]
    fn celestial_body_validate_reuses_custom_body_checks() {
        assert!(CelestialBody::Sun.validate().is_ok());

        assert_eq!(
            CelestialBody::Custom(CustomBodyId::new("asteroid", " 433-Eros "))
                .validate()
                .expect_err("custom body identifiers should be validated")
                .to_string(),
            "custom body id designation must not have leading or trailing whitespace"
        );
    }

    #[test]
    fn custom_house_system_display_includes_aliases_and_notes() {
        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("MCH".to_string());
        custom.aliases.push("Test Houses".to_string());
        custom.notes = Some("based on a user-defined formula".to_string());

        assert_eq!(
            custom.to_string(),
            "My Custom Houses [aliases: MCH, Test Houses] (based on a user-defined formula)"
        );
    }

    #[test]
    fn custom_house_system_validate_rejects_whitespace_and_duplicate_aliases() {
        assert_eq!(
            CustomHouseSystem::new("   ")
                .validate()
                .expect_err("blank house system names should be rejected")
                .to_string(),
            "custom house system name must not be blank"
        );

        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("MCH".to_string());
        custom.aliases.push("MCH".to_string());

        assert_eq!(
            custom
                .validate()
                .expect_err("duplicate aliases should be rejected")
                .to_string(),
            "custom house system aliases must be unique: duplicate MCH"
        );

        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("MCH".to_string());
        custom.aliases.push("mch".to_string());

        assert_eq!(
            custom
                .validate()
                .expect_err("case-normalized aliases should be rejected")
                .to_string(),
            "custom house system aliases must be unique: duplicate mch"
        );

        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("my custom houses".to_string());

        assert_eq!(
            custom
                .validate()
                .expect_err("aliases should not duplicate the canonical name")
                .to_string(),
            "custom house system aliases must be unique: duplicate my custom houses"
        );

        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.notes = Some("  ".to_string());

        assert_eq!(
            custom
                .validate()
                .expect_err("blank notes should be rejected")
                .to_string(),
            "custom house system notes must not be blank"
        );
    }

    #[test]
    fn custom_house_system_validate_against_reserved_labels_rejects_builtin_collisions() {
        let custom = CustomHouseSystem::new("Equal");
        assert_eq!(
            custom
                .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Equal"))
                .expect_err("built-in labels should be rejected")
                .to_string(),
            "custom house system name must not match a built-in label: Equal"
        );

        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("Whole Sign".to_string());

        assert_eq!(
            custom
                .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Whole Sign"))
                .expect_err("built-in aliases should be rejected")
                .to_string(),
            "custom house system alias[0] must not match a built-in label: Whole Sign"
        );
    }

    #[test]
    fn house_systems_have_stable_display_names() {
        assert_eq!(HouseSystem::Placidus.to_string(), "Placidus");
        assert_eq!(HouseSystem::EqualMidheaven.to_string(), "Equal (MC)");
        assert_eq!(HouseSystem::Carter.to_string(), "Carter (poli-equatorial)");
        assert_eq!(HouseSystem::Gauquelin.to_string(), "Gauquelin sectors");

        let custom = CustomHouseSystem::new("My Custom Houses");
        assert_eq!(
            HouseSystem::Custom(custom.clone()).to_string(),
            custom.to_string()
        );
    }

    #[test]
    fn house_system_validate_reuses_the_structured_validator() {
        assert!(HouseSystem::WholeSign.validate().is_ok());

        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("MCH".to_string());
        custom.aliases.push("mch".to_string());

        assert_eq!(
            HouseSystem::Custom(custom)
                .validate()
                .expect_err("custom house systems should validate their aliases")
                .to_string(),
            "custom house system aliases must be unique: duplicate mch"
        );
    }

    #[test]
    fn time_scales_have_stable_display_names() {
        assert_eq!(TimeScale::Utc.to_string(), "UTC");
        assert_eq!(TimeScale::Ut1.to_string(), "UT1");
        assert_eq!(TimeScale::Tt.to_string(), "TT");
        assert_eq!(TimeScale::Tdb.to_string(), "TDB");
    }

    #[test]
    fn coordinate_frames_have_stable_display_names() {
        assert_eq!(CoordinateFrame::Ecliptic.to_string(), "Ecliptic");
        assert_eq!(CoordinateFrame::Equatorial.to_string(), "Equatorial");
    }

    #[test]
    fn zodiac_modes_have_stable_display_names() {
        assert_eq!(ZodiacMode::Tropical.to_string(), "Tropical");
        assert_eq!(
            ZodiacMode::Sidereal {
                ayanamsa: Ayanamsa::Lahiri
            }
            .to_string(),
            "Sidereal (Lahiri)"
        );
        assert_eq!(
            ZodiacMode::Sidereal {
                ayanamsa: Ayanamsa::Custom(CustomAyanamsa::new("My Custom Sidereal"))
            }
            .to_string(),
            "Sidereal (My Custom Sidereal)"
        );
    }

    #[test]
    fn custom_ayanamsas_have_stable_display_names() {
        let custom = CustomAyanamsa::new("My Custom Sidereal");

        assert_eq!(custom.to_string(), "My Custom Sidereal");
        assert_eq!(Ayanamsa::Custom(custom).to_string(), "My Custom Sidereal");
    }

    #[test]
    fn custom_ayanamsa_enum_validate_reuses_the_structured_validator() {
        assert!(Ayanamsa::Lahiri.validate().is_ok());

        let custom = CustomAyanamsa {
            name: "My Custom Sidereal".to_string(),
            description: Some("local calibration".to_string()),
            epoch: Some(JulianDay::from_days(2_451_545.0)),
            offset_degrees: None,
        };

        assert_eq!(
            Ayanamsa::Custom(custom)
                .validate()
                .expect_err("custom ayanamsas should validate their descriptor")
                .to_string(),
            "custom ayanamsa requires both epoch and offset_degrees when one is present"
        );
    }

    #[test]
    fn custom_ayanamsa_validate_rejects_padded_or_incomplete_definitions() {
        let mut custom = CustomAyanamsa::new("My Custom Sidereal");
        custom.description = Some("  ".to_string());

        assert_eq!(
            custom
                .validate()
                .expect_err("blank descriptions should be rejected")
                .to_string(),
            "custom ayanamsa description must not be blank"
        );

        let custom = CustomAyanamsa {
            name: "My Custom Sidereal".to_string(),
            description: Some("local calibration".to_string()),
            epoch: Some(JulianDay::from_days(2_451_545.0)),
            offset_degrees: None,
        };

        assert_eq!(
            custom
                .validate()
                .expect_err("partial offset definitions should be rejected")
                .to_string(),
            "custom ayanamsa requires both epoch and offset_degrees when one is present"
        );

        let custom = CustomAyanamsa {
            name: "My Custom Sidereal".to_string(),
            description: Some("local calibration".to_string()),
            epoch: Some(JulianDay::from_days(f64::INFINITY)),
            offset_degrees: Some(Angle::from_degrees(24.0)),
        };

        assert_eq!(
            custom
                .validate()
                .expect_err("non-finite epochs should be rejected")
                .to_string(),
            "custom ayanamsa epoch must be finite"
        );
    }

    #[test]
    fn custom_ayanamsa_validate_against_reserved_labels_rejects_builtin_collisions() {
        let custom = CustomAyanamsa::new("Lahiri");

        assert_eq!(
            custom
                .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Lahiri"))
                .expect_err("built-in labels should be rejected")
                .to_string(),
            "custom ayanamsa name must not match a built-in label: Lahiri"
        );
    }

    #[test]
    fn time_scale_conversion_errors_use_stable_display_labels() {
        let error = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
            .tt_from_ut1(Duration::from_secs(1))
            .expect_err("TT is not UT1");

        assert_eq!(
            error.to_string(),
            "time-scale conversion expected UT1, got TT"
        );
    }

    #[test]
    fn zodiac_signs_follow_longitude_bands() {
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(0.0)),
            ZodiacSign::Aries
        );
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(29.999)),
            ZodiacSign::Aries
        );
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(30.0)),
            ZodiacSign::Taurus
        );
    }

    #[test]
    fn time_range_checks_scale_and_julian_day() {
        let start = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let end = Instant::new(JulianDay::from_days(2451546.0), TimeScale::Tt);
        let range = TimeRange::new(Some(start), Some(end));

        assert!(range.contains(Instant::new(JulianDay::from_days(2451545.5), TimeScale::Tt)));
        assert!(!range.contains(Instant::new(
            JulianDay::from_days(2451545.5),
            TimeScale::Utc
        )));
        assert_eq!(
            range.summary_line(),
            "JD 2451545.0 (TT) → JD 2451546.0 (TT)"
        );
        assert_eq!(range.to_string(), range.summary_line());
        assert!(range.validate().is_ok());
        assert_eq!(TimeRange::new(Some(start), None).validate(), Ok(()));
        assert_eq!(
            TimeRange::new(Some(start), None).to_string(),
            "from JD 2451545.0 (TT)"
        );
        assert_eq!(
            TimeRange::new(None, Some(end)).to_string(),
            "through JD 2451546.0 (TT)"
        );
        assert_eq!(TimeRange::new(None, None).to_string(), "unbounded");
    }

    #[test]
    fn time_range_validation_rejects_non_finite_bounds_and_invalid_order() {
        let finite_start = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let finite_end = Instant::new(JulianDay::from_days(2451546.0), TimeScale::Tt);

        let error = TimeRange::new(
            Some(Instant::new(
                JulianDay::from_days(f64::INFINITY),
                TimeScale::Tt,
            )),
            Some(finite_end),
        )
        .validate()
        .expect_err("non-finite start bounds should fail validation");
        assert_eq!(
            error.summary_line(),
            "time range bound `start` must be finite: JD inf (TT)"
        );
        assert_eq!(error.to_string(), error.summary_line());

        let error = TimeRange::new(
            Some(finite_start),
            Some(Instant::new(
                JulianDay::from_days(f64::NEG_INFINITY),
                TimeScale::Tt,
            )),
        )
        .validate()
        .expect_err("non-finite end bounds should fail validation");
        assert_eq!(
            error.summary_line(),
            "time range bound `end` must be finite: JD -inf (TT)"
        );
        assert_eq!(error.to_string(), error.summary_line());

        let error = TimeRange::new(
            Some(Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt)),
            Some(Instant::new(
                JulianDay::from_days(2451546.0),
                TimeScale::Tdb,
            )),
        )
        .validate()
        .expect_err("mixed time-scale bounds should fail validation");
        assert_eq!(
            error.summary_line(),
            "time range bounds must use the same time scale: start=JD 2451545.0 (TT); end=JD 2451546.0 (TDB)"
        );
        assert_eq!(error.to_string(), error.summary_line());

        let error = TimeRange::new(Some(finite_end), Some(finite_start))
            .validate()
            .expect_err("out-of-order ranges should fail validation");
        assert_eq!(
            error.summary_line(),
            "time range end must not precede the start: start=JD 2451546.0 (TT); end=JD 2451545.0 (TT)"
        );
        assert_eq!(error.to_string(), error.summary_line());
    }

    #[test]
    fn caller_supplied_time_scale_offsets_shift_julian_days() {
        let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
        let tt = ut1
            .tt_from_ut1(Duration::from_secs_f64(64.184))
            .expect("UT1 to TT conversion should accept UT1 input");

        assert_eq!(tt.scale, TimeScale::Tt);
        assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_utc_to_tt() {
        let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
        let tt = utc
            .tt_from_utc(Duration::from_secs_f64(64.184))
            .expect("UTC to TT conversion should accept UTC input");

        assert_eq!(tt.scale, TimeScale::Tt);
        assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_utc_to_tt_with_signed_offset() {
        let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
        let tt = utc
            .tt_from_utc_signed(64.184)
            .expect("UTC to TT conversion should accept signed UTC input");

        assert_eq!(tt.scale, TimeScale::Tt);
        assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_tt_to_tdb() {
        let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let tdb = tt
            .tdb_from_tt(Duration::from_secs_f64(0.001_657))
            .expect("TT to TDB conversion should accept TT input");

        assert_eq!(tdb.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + 0.001_657 / 86_400.0;
        assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_tt_to_tdb_with_signed_offset() {
        let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let tdb = tt
            .tdb_from_tt_signed(-0.001_657)
            .expect("TT to TDB conversion should accept signed TT input");

        assert_eq!(tdb.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_tdb_to_tt() {
        let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        let tt = tdb
            .tt_from_tdb(-0.001_657)
            .expect("TDB to TT conversion should accept TDB input");

        assert_eq!(tt.scale, TimeScale::Tt);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((tt.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_tdb_to_tt_with_signed_offset() {
        let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        let tt = tdb
            .tt_from_tdb_signed(-0.001_657)
            .expect("TDB to TT conversion should accept signed TDB input");

        assert_eq!(tt.scale, TimeScale::Tt);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((tt.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_utc_to_tdb() {
        let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
        let tdb = utc
            .tdb_from_utc(
                Duration::from_secs_f64(64.184),
                Duration::from_secs_f64(0.001_657),
            )
            .expect("UTC to TDB conversion should accept UTC input");

        assert_eq!(tdb.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
        assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_utc_to_tdb_with_signed_offset() {
        let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
        let tdb = utc
            .tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
            .expect("UTC to TDB conversion should accept signed UTC input");

        assert_eq!(tdb.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
        assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tdb() {
        let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
        let tdb = ut1
            .tdb_from_ut1(
                Duration::from_secs_f64(64.184),
                Duration::from_secs_f64(0.001_657),
            )
            .expect("UT1 to TDB conversion should accept UT1 input");

        assert_eq!(tdb.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
        assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tdb_with_signed_offset() {
        let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
        let tdb = ut1
            .tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
            .expect("UT1 to TDB conversion should accept signed UT1 input");

        assert_eq!(tdb.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
        assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tt_with_signed_offset() {
        let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
        let tt = ut1
            .tt_from_ut1_signed(64.184)
            .expect("UT1 to TT conversion should accept signed UT1 input");

        assert_eq!(tt.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((tt.julian_day.days() - expected).abs() < 1e-12);
    }

    #[test]
    fn time_scale_helpers_reject_the_wrong_source_scale() {
        let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
        let ut1_error = utc
            .tt_from_ut1(Duration::from_secs(64))
            .expect_err("UTC is not UT1");

        assert!(matches!(
            ut1_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Ut1,
                actual: TimeScale::Utc,
            }
        ));

        let tdb_ut1_error = utc
            .tdb_from_ut1(Duration::from_secs(64), Duration::from_secs(1))
            .expect_err("UTC is not UT1 for UT1-to-TDB conversion");

        assert!(matches!(
            tdb_ut1_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Ut1,
                actual: TimeScale::Utc,
            }
        ));

        let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let utc_error = tt
            .tt_from_utc(Duration::from_secs(64))
            .expect_err("TT is not UTC");

        assert!(matches!(
            utc_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Utc,
                actual: TimeScale::Tt,
            }
        ));

        let utc_signed_error = tt
            .tt_from_utc_signed(64.0)
            .expect_err("TT is not UTC for signed UTC-to-TT conversion");

        assert!(matches!(
            utc_signed_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Utc,
                actual: TimeScale::Tt,
            }
        ));

        let tdb_error = tt
            .tdb_from_utc(Duration::from_secs(64), Duration::from_secs(1))
            .expect_err("TT is not UTC for UTC-to-TDB conversion");

        assert!(matches!(
            tdb_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Utc,
                actual: TimeScale::Tt,
            }
        ));

        let tt_error = utc
            .tt_from_tdb(-0.001_657)
            .expect_err("UTC is not TDB for TDB-to-TT conversion");

        assert!(matches!(
            tt_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Tdb,
                actual: TimeScale::Utc,
            }
        ));

        let ut1_signed_error = utc
            .tt_from_ut1_signed(64.0)
            .expect_err("UTC is not UT1 for signed UT1-to-TT conversion");

        assert!(matches!(
            ut1_signed_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Ut1,
                actual: TimeScale::Utc,
            }
        ));

        let wrong_scale_error = tt
            .tt_from_tdb(-0.001_657)
            .expect_err("TT is not TDB for TDB-to-TT conversion");

        assert!(matches!(
            wrong_scale_error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Tdb,
                actual: TimeScale::Tt,
            }
        ));
    }

    #[test]
    fn signed_time_scale_helpers_reject_non_finite_offsets() {
        let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
        let tt_nan_error = ut1
            .tt_from_ut1_signed(f64::NAN)
            .expect_err("non-finite UT1 offsets should be rejected");

        assert!(matches!(
            tt_nan_error,
            TimeScaleConversionError::NonFiniteOffset
        ));

        let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let tdb_inf_error = tt
            .tdb_from_tt_signed(f64::INFINITY)
            .expect_err("non-finite TDB offsets should be rejected");

        assert!(matches!(
            tdb_inf_error,
            TimeScaleConversionError::NonFiniteOffset
        ));

        let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        let tt_negative_inf_error = tdb
            .tt_from_tdb(f64::NEG_INFINITY)
            .expect_err("non-finite TDB-to-TT offsets should be rejected");

        assert!(matches!(
            tt_negative_inf_error,
            TimeScaleConversionError::NonFiniteOffset
        ));
    }

    #[test]
    fn time_scale_conversion_errors_render_stable_summary_lines() {
        let expected = TimeScaleConversionError::Expected {
            expected: TimeScale::Tt,
            actual: TimeScale::Utc,
        };
        assert_eq!(
            expected.summary_line(),
            "time-scale conversion expected TT, got UTC"
        );
        assert_eq!(expected.to_string(), expected.summary_line());

        let non_finite = TimeScaleConversionError::NonFiniteOffset;
        assert_eq!(
            non_finite.summary_line(),
            "time-scale conversion offset must be finite"
        );
        assert_eq!(non_finite.to_string(), non_finite.summary_line());
    }

    #[test]
    fn time_scale_conversion_policy_can_validate_and_apply_a_caller_supplied_rule() {
        let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
        let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);

        assert!(policy.validate(ut1).is_ok());
        assert!(ut1.validate_time_scale_conversion(policy).is_ok());

        let converted = policy
            .apply(ut1)
            .expect("caller-supplied policy should convert the source instant");

        assert_eq!(converted.scale, TimeScale::Tt);
        assert!((converted.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
        assert_eq!(
            policy.summary_line(),
            "source=UT1; target=TT; offset_seconds=64.184 s"
        );
        assert_eq!(policy.to_string(), policy.summary_line());
    }

    #[test]
    fn time_scale_conversion_policy_renders_signed_offsets_in_summary_lines() {
        let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);

        assert_eq!(
            policy.summary_line(),
            "source=TDB; target=TT; offset_seconds=-0.001657 s"
        );
        assert_eq!(policy.to_string(), policy.summary_line());
    }

    #[test]
    fn instant_validate_time_scale_conversion_rejects_mismatched_scales_and_non_finite_offsets() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);

        let error = instant
            .validate_time_scale_conversion(policy)
            .expect_err("policy should reject the wrong source scale");

        assert!(matches!(
            error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Ut1,
                actual: TimeScale::Tt,
            }
        ));

        let non_finite = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, f64::NAN);
        let error = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
            .validate_time_scale_conversion(non_finite)
            .expect_err("policy should reject non-finite offsets");

        assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));
    }

    #[test]
    fn time_scale_conversion_policy_rejects_mismatched_scales_and_non_finite_offsets() {
        let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
        let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let error = policy
            .validate(tt)
            .expect_err("policy should reject the wrong source scale");

        assert!(matches!(
            error,
            TimeScaleConversionError::Expected {
                expected: TimeScale::Ut1,
                actual: TimeScale::Tt,
            }
        ));

        let non_finite = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, f64::NAN);
        let error = non_finite
            .validate(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ))
            .expect_err("policy should reject non-finite offsets");

        assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));
        assert!(matches!(
            non_finite.apply(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt
            )),
            Err(TimeScaleConversionError::NonFiniteOffset)
        ));
    }

    #[test]
    fn instant_with_time_scale_offset_checked_rejects_non_finite_offsets() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let error = instant
            .with_time_scale_offset_checked(TimeScale::Tdb, f64::NEG_INFINITY)
            .expect_err("checked offset conversion should reject non-finite offsets");

        assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));

        let converted = instant
            .with_time_scale_offset_checked(TimeScale::Tdb, 0.001_657)
            .expect("checked offset conversion should accept finite offsets");

        assert_eq!(converted.scale, TimeScale::Tdb);
        assert!((converted.julian_day.days() - 2_451_545.000_000_019).abs() < 1e-12);
    }

    #[test]
    fn motion_accessors_return_the_original_speed_components() {
        let motion = Motion::new(Some(0.12), Some(-0.03), Some(0.000_4));

        assert_eq!(motion.longitude_speed(), Some(0.12));
        assert_eq!(motion.latitude_speed(), Some(-0.03));
        assert_eq!(motion.distance_speed(), Some(0.000_4));
    }

    #[test]
    fn motion_summary_line_matches_display() {
        let motion = Motion::new(Some(0.12), Some(-0.03), Some(0.000_4));

        assert_eq!(
            motion.summary_line(),
            "longitude=0.12 deg/day; latitude=-0.03 deg/day; distance=0.0004 au/day"
        );
        assert_eq!(motion.to_string(), motion.summary_line());
    }

    #[test]
    fn motion_direction_tracks_the_sign_of_longitude_speed() {
        assert_eq!(
            Motion::new(Some(0.12), None, None).longitude_direction(),
            Some(MotionDirection::Direct)
        );
        assert_eq!(
            Motion::new(Some(-0.04), None, None).longitude_direction(),
            Some(MotionDirection::Retrograde)
        );
        assert_eq!(
            Motion::new(Some(0.0), None, None).longitude_direction(),
            Some(MotionDirection::Stationary)
        );
        assert_eq!(Motion::new(None, None, None).longitude_direction(), None);
        assert_eq!(
            Motion::new(Some(f64::NAN), None, None).longitude_direction(),
            None
        );
    }

    #[test]
    fn motion_validation_rejects_non_finite_components() {
        let longitude = Motion::new(Some(f64::INFINITY), None, None);
        assert_eq!(
            longitude.validate(),
            Err(MotionValidationError::NonFiniteSpeed {
                field: "longitude_deg_per_day",
                value: f64::INFINITY,
            })
        );

        let latitude = Motion::new(None, Some(f64::NAN), None);
        assert!(matches!(
            latitude.validate(),
            Err(MotionValidationError::NonFiniteSpeed {
                field: "latitude_deg_per_day",
                value,
            }) if value.is_nan()
        ));

        let distance = Motion::new(None, None, Some(f64::NEG_INFINITY));
        assert_eq!(
            distance.validate(),
            Err(MotionValidationError::NonFiniteSpeed {
                field: "distance_au_per_day",
                value: f64::NEG_INFINITY,
            })
        );
    }

    #[test]
    fn coordinate_validation_rejects_non_finite_and_out_of_range_values() {
        let bad_ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(12.0),
            Latitude::from_degrees(91.0),
            Some(-1.0),
        );
        assert_eq!(
            bad_ecliptic.validate(),
            Err(CoordinateValidationError::LatitudeOutOfRange {
                coordinate: "ecliptic",
                field: "latitude",
                value: 91.0,
            })
        );
        assert_eq!(
            CoordinateValidationError::LatitudeOutOfRange {
                coordinate: "ecliptic",
                field: "latitude",
                value: 91.0,
            }
            .summary_line(),
            "ecliptic coordinate field `latitude` must stay within [-90, 90], got 91"
        );

        let bad_distance = EclipticCoordinates::new(
            Longitude::from_degrees(12.0),
            Latitude::from_degrees(12.0),
            Some(-1.0),
        );
        assert_eq!(
            bad_distance.validate(),
            Err(CoordinateValidationError::NegativeDistance {
                coordinate: "ecliptic",
                value: -1.0,
            })
        );

        let bad_non_finite_distance = EclipticCoordinates::new(
            Longitude::from_degrees(12.0),
            Latitude::from_degrees(12.0),
            Some(f64::NAN),
        );
        assert!(matches!(
            bad_non_finite_distance.validate(),
            Err(CoordinateValidationError::NonFiniteValue {
                coordinate: "ecliptic",
                field: "distance_au",
                value,
            }) if value.is_nan()
        ));

        let bad_equatorial = EquatorialCoordinates::new(
            Angle::from_degrees(360.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        );
        assert_eq!(
            bad_equatorial.validate(),
            Err(CoordinateValidationError::RightAscensionOutOfRange {
                coordinate: "equatorial",
                field: "right_ascension",
                value: 360.0,
            })
        );
        assert_eq!(
            CoordinateValidationError::NegativeDistance {
                coordinate: "ecliptic",
                value: -0.5,
            }
            .summary_line(),
            "ecliptic coordinate field `distance_au` must be non-negative, got -0.5"
        );
    }
}
