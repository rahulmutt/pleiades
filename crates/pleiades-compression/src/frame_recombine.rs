//! Ecliptic spherical ↔ Cartesian (AU) and geocentric/heliocentric recombination.

use pleiades_types::{EclipticCoordinates, Latitude, Longitude};

/// Converts ecliptic spherical (deg, deg, AU) to ecliptic Cartesian (AU).
/// Returns `None` when distance is absent — recombination requires a radius.
pub fn ecliptic_to_cartesian_au(coords: &EclipticCoordinates) -> Option<[f64; 3]> {
    let r = coords.distance_au?;
    let lon = coords.longitude.degrees().to_radians();
    let lat = coords.latitude.degrees().to_radians();
    Some([
        r * lat.cos() * lon.cos(),
        r * lat.cos() * lon.sin(),
        r * lat.sin(),
    ])
}

/// Converts ecliptic Cartesian (AU) back to ecliptic spherical. Longitude is
/// normalized to [0, 360) by `Longitude::from_degrees`.
pub fn cartesian_au_to_ecliptic(v: [f64; 3]) -> EclipticCoordinates {
    let [x, y, z] = v;
    let radius = (x * x + y * y + z * z).sqrt();
    let longitude = Longitude::from_degrees(y.atan2(x).to_degrees());
    let latitude = if radius == 0.0 {
        Latitude::from_degrees(0.0)
    } else {
        Latitude::from_degrees((z / radius).clamp(-1.0, 1.0).asin().to_degrees())
    };
    EclipticCoordinates::new(longitude, latitude, Some(radius))
}

/// Reconstructs geocentric ecliptic from a planet's heliocentric ecliptic and
/// the geocentric Sun: `P_geo = P_helio + S_geo` (vector add in ecliptic-of-date).
pub fn geocentric_from_heliocentric(
    planet_helio: &EclipticCoordinates,
    sun_geo: &EclipticCoordinates,
) -> Option<EclipticCoordinates> {
    let p = ecliptic_to_cartesian_au(planet_helio)?;
    let s = ecliptic_to_cartesian_au(sun_geo)?;
    Some(cartesian_au_to_ecliptic([p[0] + s[0], p[1] + s[1], p[2] + s[2]]))
}

/// Derives a planet's heliocentric ecliptic from its geocentric ecliptic and
/// the geocentric Sun: `P_helio = P_geo − S_geo` (vector subtract in ecliptic-of-date).
pub fn heliocentric_from_geocentric(
    planet_geo: &EclipticCoordinates,
    sun_geo: &EclipticCoordinates,
) -> Option<EclipticCoordinates> {
    let p = ecliptic_to_cartesian_au(planet_geo)?;
    let s = ecliptic_to_cartesian_au(sun_geo)?;
    Some(cartesian_au_to_ecliptic([p[0] - s[0], p[1] - s[1], p[2] - s[2]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{EclipticCoordinates, Latitude, Longitude};

    fn ec(lon: f64, lat: f64, r: f64) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(r),
        )
    }

    #[test]
    fn cartesian_round_trips_within_tolerance() {
        let original = ec(123.456, -4.321, 9.87);
        let v = ecliptic_to_cartesian_au(&original).unwrap();
        let back = cartesian_au_to_ecliptic(v);
        assert!((back.longitude.degrees() - 123.456).abs() < 1e-9);
        assert!((back.latitude.degrees() - (-4.321)).abs() < 1e-9);
        assert!((back.distance_au.unwrap() - 9.87).abs() < 1e-9);
    }

    #[test]
    fn helio_and_geo_are_inverse_via_sun() {
        // Known truth: planet geocentric, Sun geocentric. Heliocentric = geo - sun;
        // reconstructing geo = helio + sun must return the original geocentric value.
        let planet_geo = ec(200.0, 1.5, 19.2);
        let sun_geo = ec(95.0, 0.0, 1.0);
        let helio = heliocentric_from_geocentric(&planet_geo, &sun_geo).unwrap();
        let geo_back = geocentric_from_heliocentric(&helio, &sun_geo).unwrap();
        assert!((geo_back.longitude.degrees() - 200.0).abs() < 1e-9);
        assert!((geo_back.latitude.degrees() - 1.5).abs() < 1e-9);
        assert!((geo_back.distance_au.unwrap() - 19.2).abs() < 1e-9);
    }

    #[test]
    fn missing_distance_yields_none() {
        let no_dist = EclipticCoordinates::new(
            Longitude::from_degrees(10.0),
            Latitude::from_degrees(0.0),
            None,
        );
        assert!(ecliptic_to_cartesian_au(&no_dist).is_none());
    }
}
