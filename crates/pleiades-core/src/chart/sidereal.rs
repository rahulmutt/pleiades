use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{EphemerisError, EphemerisErrorKind};
use pleiades_types::{Instant, Longitude, ZodiacMode};

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
