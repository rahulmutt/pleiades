//! Orchestrator: light-time-corrected J2000 position + Sun's longitude of date +
//! instant -> apparent ecliptic-of-date position with provenance. Applies, in
//! order: light-time, precession (J2000 -> mean equinox of date), nutation Δψ
//! (-> true equinox of date), then annual aberration. Gravitational
//! light-deflection is not applied (sub-arcsec except near the solar limb).

use pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude};

use crate::aberration::annual_aberration;
use crate::error::{ApparentLightTimeError, ApparentPlaceError};
use crate::lighttime::apparent_via_light_time;
use crate::nutation::nutation;
use crate::precession::precess_ecliptic_j2000_to_date;
use crate::provenance::{ApparentProvenance, CorrectionSet, MODEL_SOURCES};

/// Default light-time iteration cap (planets converge in 2–3 steps).
pub const DEFAULT_MAX_ITERATIONS: u8 = 8;

/// Combines mean-of-date ecliptic (λ, β) in degrees with arcsecond corrections
/// into an apparent (longitude, latitude) pair in degrees, normalizing longitude
/// to `[0, 360)` and failing closed on non-finite output.
///
/// The apsis path (nutation only, no aberration) calls this with
/// `d_lambda_arcsec = d_beta_arcsec = 0.0`.
fn combine_apparent(
    lambda_deg: f64,
    beta_deg: f64,
    d_lambda_arcsec: f64,
    d_beta_arcsec: f64,
    delta_psi_arcsec: f64,
    stage: &'static str,
) -> Result<(f64, f64), ApparentPlaceError> {
    let apparent_lon =
        (lambda_deg + (d_lambda_arcsec + delta_psi_arcsec) / 3600.0).rem_euclid(360.0);
    let apparent_lat = beta_deg + d_beta_arcsec / 3600.0;
    if !apparent_lon.is_finite() || !apparent_lat.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage });
    }
    Ok((apparent_lon, apparent_lat))
}

/// Longitude precession shift (λ − λ_J2000) for provenance, wrapped to
/// `(−180, 180]` degrees and returned in arcseconds.
fn precession_shift_arcsec(lambda_deg: f64, lambda_j2000_deg: f64) -> f64 {
    let mut shift = lambda_deg - lambda_j2000_deg;
    if shift > 180.0 {
        shift -= 360.0;
    } else if shift < -180.0 {
        shift += 360.0;
    }
    shift * 3600.0
}

/// An apparent ecliptic-of-date position and its provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApparentPosition {
    /// Apparent geocentric ecliptic coordinates, true equinox of date.
    pub ecliptic: EclipticCoordinates,
    /// Provenance recording which corrections were applied and their magnitudes.
    pub provenance: ApparentProvenance,
}

/// Computes the apparent ecliptic-of-date position for a body.
///
/// `query` returns the body's geocentric ecliptic position (J2000, with
/// `distance_au`) at a given instant in mean mode. `sun_true_longitude_of_date_deg`
/// is the Sun's true geometric longitude OF DATE at `instant` (the caller is
/// responsible for precessing it), supplied for the aberration term.
pub fn apparent_position<F, E>(
    instant: Instant,
    sun_true_longitude_of_date_deg: f64,
    max_iterations: u8,
    query: F,
) -> Result<ApparentPosition, ApparentLightTimeError<E>>
where
    F: FnMut(Instant) -> Result<EclipticCoordinates, E>,
{
    let light_timed = apparent_via_light_time(instant, max_iterations, query)?;
    let jd_tt = instant.julian_day.days();
    let lambda_j2000 = light_timed.ecliptic.longitude.degrees();
    let beta_j2000 = light_timed.ecliptic.latitude.degrees();

    let precessed = precess_ecliptic_j2000_to_date(lambda_j2000, beta_j2000, jd_tt)
        .map_err(ApparentLightTimeError::Apparent)?;
    let lambda = precessed.longitude_deg;
    let beta = precessed.latitude_deg;

    let aberration = annual_aberration(lambda, beta, sun_true_longitude_of_date_deg, jd_tt);
    let nut = nutation(jd_tt).map_err(ApparentLightTimeError::Apparent)?;

    let (apparent_lon, apparent_lat) = combine_apparent(
        lambda,
        beta,
        aberration.d_lambda_arcsec,
        aberration.d_beta_arcsec,
        nut.delta_psi_arcsec,
        "apparent-combine",
    )
    .map_err(ApparentLightTimeError::Apparent)?;

    let precession_longitude_arcsec = precession_shift_arcsec(lambda, lambda_j2000);

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(apparent_lon),
        Latitude::from_degrees(apparent_lat),
        light_timed.ecliptic.distance_au,
    );
    let provenance = ApparentProvenance {
        light_time_days: light_timed.light_time_days,
        iterations: light_timed.iterations,
        precession_longitude_arcsec,
        nutation_longitude_arcsec: nut.delta_psi_arcsec,
        aberration_longitude_arcsec: aberration.d_lambda_arcsec,
        corrections: CorrectionSet {
            light_time: true,
            precession: true,
            annual_aberration: true,
            nutation_longitude: true,
            diurnal_parallax: false,
            diurnal_aberration: false,
        },
        model_sources: MODEL_SOURCES,
    };
    Ok(ApparentPosition {
        ecliptic,
        provenance,
    })
}

/// Computes the apparent ecliptic-of-date position of the **geocentric Sun**,
/// applying annual aberration **exactly once** with no light-time re-query.
///
/// For a planet, light-time retardation and annual aberration are physically
/// distinct effects ("planetary aberration" = both). For the Sun they are the
/// *same* ~20.5″ Earth-orbital reflex effect (Meeus, *Astronomical Algorithms*
/// §25): re-querying the geocentric Sun at `t − τ` already displaces it by the
/// aberration amount, so applying a separate annual-aberration term on top
/// double-counts it. This routine therefore takes the Sun's instantaneous
/// (un-retarded) Mean/J2000 geocentric ecliptic position and applies precession,
/// nutation, and aberration once — never a light-time re-query.
///
/// The Sun's own true longitude of date supplies the `⊙` argument of the
/// aberration formula (`⊙ = λ`). Distance passes through unchanged (it is
/// essentially constant over the Sun's own light-time).
///
/// `corrections.light_time` is reported `false` even though this is an apparent
/// place: no light-time iteration is performed; the aberration term *is* the
/// light-time displacement for the Sun.
pub fn apparent_sun_position(
    instant: Instant,
    sun_geocentric_j2000: EclipticCoordinates,
) -> Result<ApparentPosition, ApparentPlaceError> {
    let jd_tt = instant.julian_day.days();
    let lambda_j2000 = sun_geocentric_j2000.longitude.degrees();
    let beta_j2000 = sun_geocentric_j2000.latitude.degrees();

    let precessed = precess_ecliptic_j2000_to_date(lambda_j2000, beta_j2000, jd_tt)?;
    let lambda = precessed.longitude_deg;
    let beta = precessed.latitude_deg;

    // Sun is its own aberration argument: ⊙ = λ. Applied ONCE.
    let aberration = annual_aberration(lambda, beta, lambda, jd_tt);
    let nut = nutation(jd_tt)?;

    let (apparent_lon, apparent_lat) = combine_apparent(
        lambda,
        beta,
        aberration.d_lambda_arcsec,
        aberration.d_beta_arcsec,
        nut.delta_psi_arcsec,
        "apparent-sun-combine",
    )?;

    let precession_longitude_arcsec = precession_shift_arcsec(lambda, lambda_j2000);

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(apparent_lon),
        Latitude::from_degrees(apparent_lat),
        sun_geocentric_j2000.distance_au,
    );
    let provenance = ApparentProvenance {
        light_time_days: 0.0,
        iterations: 0,
        precession_longitude_arcsec,
        nutation_longitude_arcsec: nut.delta_psi_arcsec,
        aberration_longitude_arcsec: aberration.d_lambda_arcsec,
        corrections: CorrectionSet {
            light_time: false,
            precession: true,
            annual_aberration: true,
            nutation_longitude: true,
            diurnal_parallax: false,
            diurnal_aberration: false,
        },
        model_sources: MODEL_SOURCES,
    };
    Ok(ApparentPosition {
        ecliptic,
        provenance,
    })
}

/// Computes the of-date ecliptic position of a **derived lunar apsis** (the
/// osculating True Apogee / True Perigee).
///
/// The apse line is a geometric direction, not a body: Swiss Ephemeris applies
/// neither light-time nor annual aberration to `SE_OSCU_APOG`. This routine
/// therefore rotates the J2000 mean direction to the **true ecliptic of date**
/// with precession + nutation in longitude ONLY. Distance passes through
/// unchanged.
pub fn apparent_apsis_position(
    instant: Instant,
    apsis_geocentric_j2000: EclipticCoordinates,
) -> Result<ApparentPosition, ApparentPlaceError> {
    let jd_tt = instant.julian_day.days();
    let lambda_j2000 = apsis_geocentric_j2000.longitude.degrees();
    let beta_j2000 = apsis_geocentric_j2000.latitude.degrees();

    let precessed = precess_ecliptic_j2000_to_date(lambda_j2000, beta_j2000, jd_tt)?;
    let lambda = precessed.longitude_deg;
    let beta = precessed.latitude_deg;
    let nut = nutation(jd_tt)?;

    let (apparent_lon, apparent_lat) = combine_apparent(
        lambda,
        beta,
        0.0,
        0.0,
        nut.delta_psi_arcsec,
        "apparent-apsis-combine",
    )?;

    let precession_longitude_arcsec = precession_shift_arcsec(lambda, lambda_j2000);

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(apparent_lon),
        Latitude::from_degrees(apparent_lat),
        apsis_geocentric_j2000.distance_au,
    );
    let provenance = ApparentProvenance {
        light_time_days: 0.0,
        iterations: 0,
        precession_longitude_arcsec,
        nutation_longitude_arcsec: nut.delta_psi_arcsec,
        aberration_longitude_arcsec: 0.0,
        corrections: CorrectionSet {
            light_time: false,
            precession: true,
            annual_aberration: false,
            nutation_longitude: true,
            diurnal_parallax: false,
            diurnal_aberration: false,
        },
        model_sources: MODEL_SOURCES,
    };
    Ok(ApparentPosition {
        ecliptic,
        provenance,
    })
}

#[cfg(test)]
mod tests;
