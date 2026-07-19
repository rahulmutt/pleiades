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

    #[test]
    fn sun_applies_aberration_once_no_light_time_requery() {
        // At J2000, precession ≈ identity. The Sun routine must apply aberration
        // exactly once and NOT re-query light-time. Compare against a hand-built
        // single-aberration reference: precess (≈identity here) + Δψ + one annual
        // aberration term with ⊙ = λ.
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let sun_j2000 = fixed(280.0, 0.0, 0.983);
        let out = apparent_sun_position(instant, sun_j2000).unwrap();

        // Reference: same math, applied once.
        let jd = 2_451_545.0_f64;
        let p = crate::precession::precess_ecliptic_j2000_to_date(280.0, 0.0, jd).unwrap();
        let lambda = p.longitude_deg;
        let beta = p.latitude_deg;
        let ab = crate::aberration::annual_aberration(lambda, beta, lambda, jd);
        let nut = crate::nutation::nutation(jd).unwrap();
        let expected_lon =
            (lambda + (ab.d_lambda_arcsec + nut.delta_psi_arcsec) / 3600.0).rem_euclid(360.0);

        let diff_arcsec = (out.ecliptic.longitude.degrees() - expected_lon) * 3600.0;
        assert!(
            diff_arcsec.abs() < 1e-6,
            "Sun apparent lon off by {diff_arcsec}\""
        );

        // Distance is passed through unchanged (no re-query).
        assert_eq!(out.ecliptic.distance_au, Some(0.983));

        // Provenance: aberration once, no light-time iteration.
        assert!(
            !out.provenance.corrections.light_time,
            "light_time must be false for Sun"
        );
        assert!(out.provenance.corrections.annual_aberration);
        assert!(out.provenance.corrections.precession);
        assert!(out.provenance.corrections.nutation_longitude);
        assert_eq!(out.provenance.light_time_days, 0.0);
        assert_eq!(out.provenance.iterations, 0);
        // The single applied aberration term, recorded (≈ -20" for the Sun).
        assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-9);
    }

    #[test]
    fn apsis_position_is_precession_and_nutation_only_no_aberration() {
        // At J2000 precession ≈ identity, so the only shift is Δψ/3600 (a few
        // arcsec). There must be NO ~20" annual-aberration term and NO change to
        // latitude (aberration is what would move β; precession at J2000 does not).
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let j2000 = fixed(100.0, 5.0, 0.0025);
        let out = apparent_apsis_position(instant, j2000).unwrap();

        let dlon_arcsec = (out.ecliptic.longitude.degrees() - 100.0) * 3600.0;
        assert!(
            dlon_arcsec.abs() < 20.0,
            "lon shift {dlon_arcsec}\" must be precession+nutation only"
        );
        // Lower bound: nutation Δψ at J2000 ≈ -17"; a no-Δψ regression (precession ≈ 0 here)
        // would leave dlon ≈ 0 and pass the upper bound silently.
        assert!(
            dlon_arcsec.abs() > 1.0,
            "nutation Δψ not applied: lon shift only {dlon_arcsec}\""
        );
        assert!(
            out.provenance.nutation_longitude_arcsec.abs() > 1.0,
            "provenance nutation arcsec implausibly small: {}",
            out.provenance.nutation_longitude_arcsec
        );
        assert!(
            (out.ecliptic.latitude.degrees() - 5.0).abs() < 1e-6,
            "latitude must be unchanged by aberration"
        );
        assert_eq!(out.provenance.aberration_longitude_arcsec, 0.0);
        assert!(!out.provenance.corrections.annual_aberration);
        assert!(!out.provenance.corrections.light_time);
        assert!(out.provenance.corrections.precession);
        assert!(out.provenance.corrections.nutation_longitude);
        assert_eq!(out.provenance.iterations, 0);
    }
}
