//! Topocentric correction: diurnal parallax + diurnal aberration applied to a
//! geocentric apparent ecliptic-of-date position. Pure; the caller supplies the
//! local apparent sidereal time and obliquity of date.

use pleiades_types::{Angle, EclipticCoordinates, Latitude, Longitude, ObserverLocation};

use crate::error::ApparentPlaceError;
use crate::parallax::{ObserverGeocentric, AU_IN_EARTH_RADII};
use crate::provenance::TopocentricProvenance;

/// Diurnal aberration constant in arcseconds (0.0213 s of time × 15).
pub const DIURNAL_ABERRATION_ARCSEC: f64 = 0.319_2;

/// A topocentric ecliptic-of-date position and its provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopocentricPosition {
    pub ecliptic: EclipticCoordinates,
    pub provenance: TopocentricProvenance,
}

/// Applies diurnal parallax and diurnal aberration to a geocentric apparent
/// ecliptic position.
///
/// `apparent` is the geocentric apparent ecliptic-of-date position (must carry
/// `distance_au`). `local_sidereal_time_deg` is the observer's local apparent
/// sidereal time (degrees). `obliquity_deg` is the true obliquity of date.
pub fn topocentric_position(
    apparent: EclipticCoordinates,
    observer: &ObserverLocation,
    local_sidereal_time_deg: f64,
    obliquity_deg: f64,
) -> Result<TopocentricPosition, ApparentPlaceError> {
    let distance_au = apparent.distance_au.ok_or(ApparentPlaceError::MissingDistance)?;

    let obliquity = Angle::from_degrees(obliquity_deg);
    let equatorial = apparent.to_equatorial(obliquity);
    let ra = equatorial.right_ascension.degrees().to_radians();
    let dec = equatorial.declination.degrees().to_radians();

    // Geocentric body vector in Earth equatorial radii.
    let d = distance_au * AU_IN_EARTH_RADII;
    let bx = d * dec.cos() * ra.cos();
    let by = d * dec.cos() * ra.sin();
    let bz = d * dec.sin();

    // Observer vector and topocentric (observer-relative) vector.
    let geo = ObserverGeocentric::from_location(observer);
    let [ox, oy, oz] = geo.equatorial_vector(local_sidereal_time_deg);
    let tx = bx - ox;
    let ty = by - oy;
    let tz = bz - oz;
    let topo_distance = (tx * tx + ty * ty + tz * tz).sqrt();
    if !topo_distance.is_finite() || topo_distance <= 0.0 {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "topocentric" });
    }
    let mut ra_topo = ty.atan2(tx);
    let dec_topo = (tz / topo_distance).asin();

    // Diurnal aberration: observer moves east at ω·ρcosφ′. Hour angle H = LAST - RA.
    let hour_angle = (local_sidereal_time_deg.to_radians() - ra_topo).rem_euclid(std::f64::consts::TAU);
    let aberr_ra_arcsec =
        DIURNAL_ABERRATION_ARCSEC * geo.rho_cos_phi_prime * hour_angle.cos() / dec_topo.cos();
    let aberr_dec_arcsec =
        DIURNAL_ABERRATION_ARCSEC * geo.rho_cos_phi_prime * hour_angle.sin() * dec_topo.sin();
    ra_topo += (aberr_ra_arcsec / 3600.0).to_radians();
    let dec_topo = dec_topo + (aberr_dec_arcsec / 3600.0).to_radians();

    let equatorial_topo = pleiades_types::EquatorialCoordinates::new(
        Angle::from_degrees(ra_topo.to_degrees()).normalized_0_360(),
        Latitude::from_degrees(dec_topo.to_degrees()),
        Some(topo_distance / AU_IN_EARTH_RADII),
    );
    let ecliptic_topo = equatorial_topo.to_ecliptic(obliquity);

    let mut d_lon = ecliptic_topo.longitude.degrees() - apparent.longitude.degrees();
    if d_lon > 180.0 {
        d_lon -= 360.0;
    } else if d_lon < -180.0 {
        d_lon += 360.0;
    }
    let d_lat = ecliptic_topo.latitude.degrees() - apparent.latitude.degrees();

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(ecliptic_topo.longitude.degrees().rem_euclid(360.0)),
        ecliptic_topo.latitude,
        ecliptic_topo.distance_au,
    );
    if !ecliptic.longitude.degrees().is_finite() || !ecliptic.latitude.degrees().is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "topocentric" });
    }

    Ok(TopocentricPosition {
        ecliptic,
        provenance: TopocentricProvenance {
            parallax_longitude_arcsec: d_lon * 3600.0,
            parallax_latitude_arcsec: d_lat * 3600.0,
            diurnal_aberration_arcsec: aberr_ra_arcsec.hypot(aberr_dec_arcsec),
            distance_au_used: distance_au,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ecl(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        )
    }

    fn observer(lat: f64) -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(0.0),
            Some(0.0),
        )
    }

    #[test]
    fn moon_parallax_is_about_one_degree() {
        // Moon at ~0.00257 AU (60.3 Earth radii). For an observer with the Moon
        // near the horizon the parallax approaches ~0.95°. Assert it is large.
        let out = topocentric_position(ecl(100.0, 0.0, 0.002_57), &observer(0.0), 100.0, 23.4)
            .unwrap();
        let shift = out.provenance.parallax_longitude_arcsec.hypot(
            out.provenance.parallax_latitude_arcsec,
        ) / 3600.0;
        assert!(shift > 0.3, "moon parallax {shift}° too small");
    }

    #[test]
    fn distant_body_parallax_is_negligible() {
        // A body at 30 AU: parallax < 1".
        let out = topocentric_position(ecl(100.0, 0.0, 30.0), &observer(0.0), 100.0, 23.4)
            .unwrap();
        let shift = out.provenance.parallax_longitude_arcsec.hypot(
            out.provenance.parallax_latitude_arcsec,
        );
        assert!(shift < 1.0, "distant parallax {shift}\" too large");
    }

    #[test]
    fn missing_distance_errors() {
        let no_dist = EclipticCoordinates::new(
            Longitude::from_degrees(100.0),
            Latitude::from_degrees(0.0),
            None,
        );
        let err = topocentric_position(no_dist, &observer(0.0), 100.0, 23.4).unwrap_err();
        assert_eq!(err, ApparentPlaceError::MissingDistance);
    }

    #[test]
    fn diurnal_aberration_is_sub_arcsec() {
        let out = topocentric_position(ecl(100.0, 0.0, 1.0), &observer(0.0), 100.0, 23.4)
            .unwrap();
        assert!(
            out.provenance.diurnal_aberration_arcsec < 0.36,
            "diurnal aberration {}\"",
            out.provenance.diurnal_aberration_arcsec
        );
    }
}
