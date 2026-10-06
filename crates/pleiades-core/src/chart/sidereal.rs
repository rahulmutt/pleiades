use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{EphemerisError, EphemerisErrorKind};
use pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude, ZodiacMode};

/// Converts a tropical longitude into the requested zodiac mode.
///
/// Tropical mode returns the input unchanged. Sidereal mode subtracts the
/// resolved ayanamsa for the provided instant. The longitude is taken to be
/// on the mean equinox of date. The chart layer brings its own longitudes
/// there first: an apparent (true-equinox) one has nutation removed, so
/// nutation does not move a body through a sidereal zodiac (issue #120), and
/// a mean one, which the backends report on the J2000 equinox, is precessed
/// to the equinox of date (issue #164).
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

/// Converts a longitude on the **true** equinox of date (an apparent place,
/// a house cusp or an angle) into the requested zodiac mode.
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

/// The mean J2000 place `j2000` on the mean ecliptic and equinox of date
/// `julian_day` (TT). The distance passes through.
///
/// The backends report a mean place on the J2000 equinox, and an ayanamsa is
/// counted from the equinox of date, so a mean place takes this step before
/// [`sidereal_longitude`] (issue #164). It is the IAU 1976 precession the
/// apparent reduction and `pleiades-events`' mean-of-date frame use.
pub(super) fn mean_place_of_date(
    j2000: EclipticCoordinates,
    julian_day: f64,
) -> Result<EclipticCoordinates, EphemerisError> {
    let of_date = precess_ecliptic_j2000_to_date(
        j2000.longitude.degrees(),
        j2000.latitude.degrees(),
        julian_day,
    )
    .map_err(super::map_apparent_place_error)?;
    Ok(EclipticCoordinates::new(
        Longitude::from_degrees(of_date.longitude_deg),
        Latitude::from_degrees(of_date.latitude_deg),
        j2000.distance_au,
    ))
}
