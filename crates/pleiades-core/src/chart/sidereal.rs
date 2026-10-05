use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{EphemerisError, EphemerisErrorKind};
use pleiades_types::{Instant, Longitude, ZodiacMode};

/// Converts a tropical longitude into the requested zodiac mode.
///
/// Tropical mode returns the input unchanged. Sidereal mode subtracts the
/// resolved ayanamsa for the provided instant. The longitude is taken to be
/// on the mean equinox; the chart layer moves an apparent (true-equinox)
/// longitude to the mean equinox first, so nutation does not move a body
/// through a sidereal zodiac (issue #120).
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

/// Rate at which the requested zodiac mode's ayanamsa grows at `instant`, in
/// degrees per day: what [`sidereal_longitude`] takes off a longitude speed.
/// Zero in tropical mode.
///
/// The ayanamsa is differenced centrally over ±`half_span_days`. It is smooth
/// and nearly linear (general precession, about 3.8e-5 deg/day), so the span
/// carries no measurable truncation.
pub(super) fn ayanamsa_rate_deg_per_day(
    instant: Instant,
    zodiac_mode: &ZodiacMode,
    half_span_days: f64,
) -> Result<f64, EphemerisError> {
    let ayanamsa = match zodiac_mode {
        ZodiacMode::Tropical => return Ok(0.0),
        ZodiacMode::Sidereal { ayanamsa } => ayanamsa,
        _ => {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "unsupported zodiac mode",
            ))
        }
    };
    let ayanamsa_deg = |offset_days: f64| {
        sidereal_offset(ayanamsa, super::offset_instant(instant, offset_days))
            .map(|offset| offset.degrees())
            .ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "sidereal conversion requires an ayanamsa with reference offset metadata",
                )
            })
    };
    Ok((ayanamsa_deg(half_span_days)? - ayanamsa_deg(-half_span_days)?) / (2.0 * half_span_days))
}

/// Converts a longitude on the **true** equinox of date (an apparent place)
/// into the requested zodiac mode.
///
/// A sidereal longitude is referred to the mean equinox of date, so nutation
/// in longitude (`nutation_longitude_arcsec`, Δψ) is removed before the
/// ayanamsa is subtracted and nutation does not move a body through a
/// sidereal zodiac. This is the Swiss Ephemeris `SEFLG_SIDEREAL` convention
/// and the one `pleiades-events` reads crossings in (issue #120). Tropical
/// mode returns the input unchanged.
pub(super) fn sidereal_longitude_of_true_equinox(
    longitude: Longitude,
    nutation_longitude_arcsec: f64,
    instant: Instant,
    zodiac_mode: &ZodiacMode,
) -> Result<Longitude, EphemerisError> {
    if matches!(zodiac_mode, ZodiacMode::Tropical) {
        return Ok(longitude);
    }
    let mean_equinox =
        Longitude::from_degrees(longitude.degrees() - nutation_longitude_arcsec / 3600.0);
    sidereal_longitude(mean_equinox, instant, zodiac_mode)
}
