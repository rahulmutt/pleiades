//! Time primitives: [`JulianDay`], [`TimeScale`], [`TimeScaleConversion`], and [`Instant`].

use core::fmt;
use core::time::Duration;

use crate::angles::Angle;

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

/// J2000.0 mean obliquity of the ecliptic, degrees (IAU 1976 constant term).
/// Single source of truth shared by the SPK ICRF→ecliptic reduction
/// (`pleiades-jpl`), the J2000→date precession (`pleiades-apparent`), and the
/// constant term of [`Instant::mean_obliquity`].
pub const OBLIQUITY_J2000_DEG: f64 = 23.439_291_111_111_11;

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
    pub(crate) const fn expected(expected: TimeScale, actual: TimeScale) -> Self {
        Self::Expected { expected, actual }
    }

    pub(crate) const fn non_finite_offset() -> Self {
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
/// assert_eq!(policy.summary_line(), "source=UT1; target=TT; offset_seconds=64.184 s");
/// assert_eq!(converted.scale, TimeScale::Tt);
/// ```
///
/// ```
/// use pleiades_types::{Instant, JulianDay, TimeScale, TimeScaleConversion};
///
/// let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
///
/// assert!(policy.validate(instant).is_ok());
/// assert_eq!(policy.summary_line(), "source=TDB; target=TT; offset_seconds=-0.001657 s");
/// assert_eq!(policy.validated_summary_line(instant).unwrap(), policy.summary_line());
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

    /// Returns the compact summary line after validating the policy.
    ///
    /// This keeps the fail-closed rendering path co-located with the explicit
    /// conversion contract when a caller wants to report the policy before
    /// mutating the instant.
    pub fn validated_summary_line(
        &self,
        instant: Instant,
    ) -> Result<String, TimeScaleConversionError> {
        self.validate(instant)?;
        Ok(self.summary_line())
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

pub(crate) fn checked_time_scale_offset(
    offset_seconds: f64,
) -> Result<f64, TimeScaleConversionError> {
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
            OBLIQUITY_J2000_DEG
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
