use crate::identity::BackendId;
use core::fmt;
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, CoordinateValidationError, EclipticCoordinates,
    EquatorialCoordinates, Instant, Motion, MotionValidationError, ZodiacMode,
};

/// Quality annotation for a backend result.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum QualityAnnotation {
    /// Exact or source-equivalent data.
    Exact,
    /// Interpolated from source samples.
    Interpolated,
    /// Approximate but still useful.
    Approximate,
    /// Quality is not yet published.
    Unknown,
}

impl QualityAnnotation {
    /// Returns a stable human-readable label for the quality annotation.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::Interpolated => "Interpolated",
            Self::Approximate => "Approximate",
            Self::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for QualityAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A backend result containing the requested coordinates where available.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct EphemerisResult {
    /// Backend that produced the result.
    pub backend_id: BackendId,
    /// Body that was queried.
    pub body: CelestialBody,
    /// Instant that was queried.
    pub instant: Instant,
    /// Coordinate frame of the result.
    pub frame: CoordinateFrame,
    /// Zodiac mode of the result.
    pub zodiac_mode: ZodiacMode,
    /// Whether apparent or mean values were requested.
    pub apparent: Apparentness,
    /// Ecliptic coordinates when available.
    pub ecliptic: Option<EclipticCoordinates>,
    /// Equatorial coordinates when available.
    pub equatorial: Option<EquatorialCoordinates>,
    /// Apparent motion when available.
    pub motion: Option<Motion>,
    /// Quality annotation for the result.
    pub quality: QualityAnnotation,
}

/// Errors returned when a backend result record no longer matches its stored data.
#[derive(Clone, Debug, PartialEq)]
pub enum EphemerisResultValidationError {
    /// The stored ecliptic coordinates are invalid.
    InvalidEcliptic(CoordinateValidationError),
    /// The stored equatorial coordinates are invalid.
    InvalidEquatorial(CoordinateValidationError),
    /// The stored motion sample is invalid.
    InvalidMotion(MotionValidationError),
}

impl fmt::Display for EphemerisResultValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEcliptic(error) => {
                write!(f, "backend result ecliptic is invalid: {error}")
            }
            Self::InvalidEquatorial(error) => {
                write!(f, "backend result equatorial is invalid: {error}")
            }
            Self::InvalidMotion(error) => write!(f, "backend result motion is invalid: {error}"),
        }
    }
}

impl std::error::Error for EphemerisResultValidationError {}

impl EphemerisResult {
    /// Creates an empty result shell with the request metadata filled in.
    pub fn new(
        backend_id: BackendId,
        body: CelestialBody,
        instant: Instant,
        frame: CoordinateFrame,
        zodiac_mode: ZodiacMode,
        apparent: Apparentness,
    ) -> Self {
        Self {
            backend_id,
            body,
            instant,
            frame,
            zodiac_mode,
            apparent,
            ecliptic: None,
            equatorial: None,
            motion: None,
            quality: QualityAnnotation::Unknown,
        }
    }

    /// Validates the stored coordinate and motion samples.
    pub fn validate(&self) -> Result<(), EphemerisResultValidationError> {
        if let Some(ecliptic) = &self.ecliptic {
            ecliptic
                .validate()
                .map_err(EphemerisResultValidationError::InvalidEcliptic)?;
        }

        if let Some(equatorial) = &self.equatorial {
            equatorial
                .validate()
                .map_err(EphemerisResultValidationError::InvalidEquatorial)?;
        }

        if let Some(motion) = &self.motion {
            motion
                .validate()
                .map_err(EphemerisResultValidationError::InvalidMotion)?;
        }

        Ok(())
    }

    /// Returns a compact one-line rendering of the backend result.
    ///
    /// The summary keeps the request-shape metadata alongside the available
    /// coordinate, motion, and quality fields so callers can compare a backend
    /// result without drilling into each optional channel manually.
    pub fn summary_line(&self) -> String {
        format!(
            "backend={}; body={}; instant={}; frame={}; zodiac={}; apparent={}; quality={}; ecliptic={}; equatorial={}; motion={}",
            self.backend_id,
            self.body,
            self.instant,
            self.frame,
            self.zodiac_mode,
            self.apparent,
            self.quality,
            format_optional_ecliptic_coordinates(self.ecliptic.as_ref()),
            format_optional_equatorial_coordinates(self.equatorial.as_ref()),
            format_optional_motion(self.motion.as_ref()),
        )
    }

    /// Returns a compact one-line rendering after validating the stored samples.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisResultValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for EphemerisResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn format_optional_ecliptic_coordinates(value: Option<&EclipticCoordinates>) -> String {
    value
        .map(|coordinates| {
            let distance = coordinates
                .distance_au
                .map(|distance| format!("{distance} AU"))
                .unwrap_or_else(|| "n/a".to_string());

            format!(
                "longitude={}, latitude={}, distance={}",
                coordinates.longitude, coordinates.latitude, distance
            )
        })
        .unwrap_or_else(|| "absent".to_string())
}

pub(crate) fn format_optional_equatorial_coordinates(
    value: Option<&EquatorialCoordinates>,
) -> String {
    value
        .map(|coordinates| {
            let distance = coordinates
                .distance_au
                .map(|distance| format!("{distance} AU"))
                .unwrap_or_else(|| "n/a".to_string());

            format!(
                "right_ascension={}, declination={}, distance={}",
                coordinates.right_ascension, coordinates.declination, distance
            )
        })
        .unwrap_or_else(|| "absent".to_string())
}

pub(crate) fn format_optional_motion(value: Option<&Motion>) -> String {
    value
        .map(|motion| {
            let longitude_speed = motion
                .longitude_deg_per_day
                .map(|speed| format!("{speed} deg/day"))
                .unwrap_or_else(|| "n/a".to_string());
            let latitude_speed = motion
                .latitude_deg_per_day
                .map(|speed| format!("{speed} deg/day"))
                .unwrap_or_else(|| "n/a".to_string());
            let distance_speed = motion
                .distance_au_per_day
                .map(|speed| format!("{speed} AU/day"))
                .unwrap_or_else(|| "n/a".to_string());

            format!(
                "longitude_speed={}, latitude_speed={}, distance_speed={}",
                longitude_speed, latitude_speed, distance_speed
            )
        })
        .unwrap_or_else(|| "absent".to_string())
}
