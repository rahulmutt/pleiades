//! Observer geocentric position on the WGS84 ellipsoid and the equatorial
//! rectangular vector used for diurnal parallax. Pure geometry (Meeus Ch. 11).

use pleiades_types::ObserverLocation;

/// WGS84 equatorial radius, meters.
pub const EARTH_EQUATORIAL_RADIUS_M: f64 = 6_378_137.0;
/// WGS84 polar/equatorial axis ratio (b/a).
pub const EARTH_B_OVER_A: f64 = 0.996_647_189;
/// One astronomical unit expressed in Earth equatorial radii.
pub const AU_IN_EARTH_RADII: f64 = 23_454.779;

/// Geocentric quantities `ρ·sinφ′` and `ρ·cosφ′` for an observer (Meeus 11).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObserverGeocentric {
    /// `ρ·sinφ′`, dimensionless (fraction of Earth equatorial radius).
    pub rho_sin_phi_prime: f64,
    /// `ρ·cosφ′`, dimensionless (fraction of Earth equatorial radius).
    pub rho_cos_phi_prime: f64,
}

impl ObserverGeocentric {
    /// Computes `ρ·sinφ′` and `ρ·cosφ′` from geodetic latitude and elevation
    /// (Meeus Ch. 11). Elevation defaults to sea level when absent.
    pub fn from_location(observer: &ObserverLocation) -> Self {
        let phi = observer.latitude.degrees().to_radians();
        let height_m = observer.elevation_m.unwrap_or(0.0);
        let u = (EARTH_B_OVER_A * phi.tan()).atan();
        let h_over_a = height_m / EARTH_EQUATORIAL_RADIUS_M;
        let rho_sin_phi_prime = EARTH_B_OVER_A * u.sin() + h_over_a * phi.sin();
        let rho_cos_phi_prime = u.cos() + h_over_a * phi.cos();
        Self {
            rho_sin_phi_prime,
            rho_cos_phi_prime,
        }
    }

    /// Observer rectangular vector in the equatorial frame, in Earth equatorial
    /// radii, given local apparent sidereal time in degrees.
    pub fn equatorial_vector(self, local_sidereal_time_deg: f64) -> [f64; 3] {
        let lst = local_sidereal_time_deg.to_radians();
        [
            self.rho_cos_phi_prime * lst.cos(),
            self.rho_cos_phi_prime * lst.sin(),
            self.rho_sin_phi_prime,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Latitude, Longitude};

    fn loc(lat: f64, lon: f64, elev: Option<f64>) -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(lon),
            elev,
        )
    }

    #[test]
    fn palomar_matches_meeus_chapter_11() {
        // Meeus Ch. 11 worked example: Palomar, φ=+33.356111°, H=1706 m →
        // ρ·sinφ′ = 0.546861, ρ·cosφ′ = 0.836339.
        let g = ObserverGeocentric::from_location(&loc(33.356_111, 0.0, Some(1706.0)));
        assert!((g.rho_sin_phi_prime - 0.546_861).abs() < 1e-5, "{g:?}");
        assert!((g.rho_cos_phi_prime - 0.836_339).abs() < 1e-5, "{g:?}");
    }

    #[test]
    fn equator_sea_level_is_unit_in_plane() {
        let g = ObserverGeocentric::from_location(&loc(0.0, 0.0, Some(0.0)));
        assert!((g.rho_cos_phi_prime - 1.0).abs() < 1e-6, "{g:?}");
        assert!(g.rho_sin_phi_prime.abs() < 1e-6, "{g:?}");
    }

    #[test]
    fn vector_at_lst_zero_points_along_x() {
        let g = ObserverGeocentric::from_location(&loc(0.0, 0.0, Some(0.0)));
        let v = g.equatorial_vector(0.0);
        assert!((v[0] - 1.0).abs() < 1e-6, "{v:?}");
        assert!(v[1].abs() < 1e-6 && v[2].abs() < 1e-6, "{v:?}");
    }
}
