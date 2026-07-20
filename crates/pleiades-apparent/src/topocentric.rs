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
    /// Topocentric (observer-centric) ecliptic-of-date coordinates.
    pub ecliptic: EclipticCoordinates,
    /// Provenance recording the parallax and diurnal-aberration shifts applied.
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
    let distance_au = apparent
        .distance_au
        .ok_or(ApparentPlaceError::MissingDistance)?;

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
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "topocentric",
        });
    }
    let mut ra_topo = ty.atan2(tx);
    let dec_topo = (tz / topo_distance).asin();

    // Diurnal aberration: observer moves east at ω·ρcosφ′. Hour angle H = LAST - RA.
    let hour_angle =
        (local_sidereal_time_deg.to_radians() - ra_topo).rem_euclid(std::f64::consts::TAU);
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
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "topocentric",
        });
    }

    Ok(TopocentricPosition {
        ecliptic,
        provenance: TopocentricProvenance {
            parallax_longitude_arcsec: d_lon * 3600.0,
            parallax_latitude_arcsec: d_lat * 3600.0,
            diurnal_aberration_arcsec: (aberr_ra_arcsec * dec_topo.cos()).hypot(aberr_dec_arcsec),
            distance_au_used: distance_au,
        },
    })
}

#[cfg(test)]
mod tests;
