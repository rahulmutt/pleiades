//! Orchestrator: light-time-corrected J2000 position + Sun's longitude of date +
//! instant -> apparent ecliptic-of-date position with provenance. Applies, in
//! order: light-time, precession (J2000 -> mean equinox of date), nutation Δψ
//! (-> true equinox of date), then annual aberration.

use pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude};

use crate::aberration::annual_aberration;
use crate::error::{ApparentLightTimeError, ApparentPlaceError};
use crate::lighttime::apparent_via_light_time;
use crate::nutation::nutation;
use crate::precession::precess_ecliptic_j2000_to_date;
use crate::provenance::{ApparentProvenance, CorrectionSet, MODEL_SOURCES};

/// Default light-time iteration cap (planets converge in 2–3 steps).
pub const DEFAULT_MAX_ITERATIONS: u8 = 8;

/// An apparent ecliptic-of-date position and its provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApparentPosition {
    pub ecliptic: EclipticCoordinates,
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

    let apparent_lon =
        (lambda + (aberration.d_lambda_arcsec + nut.delta_psi_arcsec) / 3600.0).rem_euclid(360.0);
    let apparent_lat = beta + aberration.d_beta_arcsec / 3600.0;
    if !apparent_lon.is_finite() || !apparent_lat.is_finite() {
        return Err(ApparentLightTimeError::Apparent(
            ApparentPlaceError::NonFiniteCorrection {
                stage: "apparent-combine",
            },
        ));
    }

    // Precession shift in longitude for provenance, wrapped to (-180, 180].
    let mut precession_shift = lambda - lambda_j2000;
    if precession_shift > 180.0 {
        precession_shift -= 360.0;
    } else if precession_shift < -180.0 {
        precession_shift += 360.0;
    }

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(apparent_lon),
        Latitude::from_degrees(apparent_lat),
        light_timed.ecliptic.distance_au,
    );
    let provenance = ApparentProvenance {
        light_time_days: light_timed.light_time_days,
        iterations: light_timed.iterations,
        precession_longitude_arcsec: precession_shift * 3600.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    fn fixed(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        )
    }

    #[test]
    fn at_j2000_only_aberration_and_nutation_shift_longitude() {
        // At J2000 precession is the identity, so the shift from mean is only
        // (Δψ + Δλ)/3600, < ~0.01°, and the precession provenance is ~0.
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
            Ok(fixed(100.0, 0.0, 1.0))
        })
        .unwrap();
        let shift_arcsec = (out.ecliptic.longitude.degrees() - 100.0) * 3600.0;
        assert!(shift_arcsec.abs() < 40.0, "shift {shift_arcsec}\"");
        assert!(
            out.provenance.precession_longitude_arcsec.abs() < 1.0,
            "precession should be ~0 at J2000"
        );
        assert!(out.provenance.corrections.precession);
        assert!(out.provenance.corrections.nutation_longitude);
        assert!(out.provenance.iterations >= 1);
    }

    #[test]
    fn precession_dominates_far_from_j2000() {
        // One century from J2000, precession shifts longitude by ~1.4° (≫ the
        // arcsec aberration/nutation), and the provenance records ~5029".
        let jd = 2_451_545.0 + 36_525.0;
        let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
            Ok(fixed(100.0, 0.0, 1.0))
        })
        .unwrap();
        let shift_deg = out.ecliptic.longitude.degrees() - 100.0;
        assert!((shift_deg - 1.397).abs() < 0.02, "shift {shift_deg}°");
        assert!(
            (out.provenance.precession_longitude_arcsec - 5029.0).abs() < 80.0,
            "precession_lon {}\"",
            out.provenance.precession_longitude_arcsec
        );
    }

    #[test]
    fn latitude_moves_by_precession_and_aberration_only() {
        // At J2000, Δψ does not change latitude; only aberration's sub-arcsec Δβ does.
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
            Ok(fixed(100.0, 5.0, 1.0))
        })
        .unwrap();
        let dlat_arcsec = (out.ecliptic.latitude.degrees() - 5.0) * 3600.0;
        assert!(dlat_arcsec.abs() < 1.0, "Δβ {dlat_arcsec}\"");
    }
}
