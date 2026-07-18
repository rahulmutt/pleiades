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
    Some(cartesian_au_to_ecliptic([
        p[0] + s[0],
        p[1] + s[1],
        p[2] + s[2],
    ]))
}

/// Spherical ecliptic state: position (lon, lat, dist) plus velocity rates (all in AU and rad/day).
#[derive(Clone, Copy, Debug)]
pub struct SphericalState {
    /// Ecliptic longitude, in radians.
    pub lon_rad: f64,
    /// Ecliptic latitude, in radians.
    pub lat_rad: f64,
    /// Radial distance, in astronomical units.
    pub dist_au: f64,
    /// Rate of change of ecliptic longitude, in radians per day.
    pub lon_rate_rad_per_day: f64,
    /// Rate of change of ecliptic latitude, in radians per day.
    pub lat_rate_rad_per_day: f64,
    /// Rate of change of distance, in astronomical units per day.
    pub dist_rate_au_per_day: f64,
}

/// Cartesian ecliptic state: position and velocity (all in AU and AU/day).
#[derive(Clone, Copy, Debug)]
pub struct CartesianState {
    /// Position vector `[x, y, z]` in the ecliptic frame, in astronomical units.
    pub pos_au: [f64; 3],
    /// Velocity vector `[vx, vy, vz]` in the ecliptic frame, in astronomical units per day.
    pub vel_au_per_day: [f64; 3],
}

/// Converts a spherical ecliptic state to Cartesian using the chain rule.
pub fn spherical_state_to_cartesian(s: SphericalState) -> CartesianState {
    let (sl, cl) = s.lon_rad.sin_cos();
    let (sb, cb) = s.lat_rad.sin_cos();
    let r = s.dist_au;
    let pos = [r * cb * cl, r * cb * sl, r * sb];
    let dr = s.dist_rate_au_per_day;
    let dl = s.lon_rate_rad_per_day;
    let db = s.lat_rate_rad_per_day;
    let vel = [
        dr * cb * cl - r * sb * cl * db - r * cb * sl * dl,
        dr * cb * sl - r * sb * sl * db + r * cb * cl * dl,
        dr * sb + r * cb * db,
    ];
    CartesianState {
        pos_au: pos,
        vel_au_per_day: vel,
    }
}

/// Converts a Cartesian ecliptic state back to spherical using the inverse chain rule.
pub fn cartesian_state_to_spherical(c: CartesianState) -> SphericalState {
    let [x, y, z] = c.pos_au;
    let [vx, vy, vz] = c.vel_au_per_day;
    let rho2 = x * x + y * y;
    let rho = rho2.sqrt();
    let r = (rho2 + z * z).sqrt();
    let dr = if r == 0.0 {
        0.0
    } else {
        (x * vx + y * vy + z * vz) / r
    };
    let dl = if rho2 == 0.0 {
        0.0
    } else {
        (x * vy - y * vx) / rho2
    };
    // β = atan2(z, ρ); dβ/dt = (ρ·vz − z·ρ̇)/r², where ρ̇ = (x·vx + y·vy)/ρ
    let drho = if rho == 0.0 {
        0.0
    } else {
        (x * vx + y * vy) / rho
    };
    let db = if r == 0.0 {
        0.0
    } else {
        (rho * vz - z * drho) / (r * r)
    };
    SphericalState {
        lon_rad: y.atan2(x),
        lat_rad: z.atan2(rho),
        dist_au: r,
        lon_rate_rad_per_day: dl,
        lat_rate_rad_per_day: db,
        dist_rate_au_per_day: dr,
    }
}

/// Derives a planet's heliocentric ecliptic from its geocentric ecliptic and
/// the geocentric Sun: `P_helio = P_geo − S_geo` (vector subtract in ecliptic-of-date).
pub fn heliocentric_from_geocentric(
    planet_geo: &EclipticCoordinates,
    sun_geo: &EclipticCoordinates,
) -> Option<EclipticCoordinates> {
    let p = ecliptic_to_cartesian_au(planet_geo)?;
    let s = ecliptic_to_cartesian_au(sun_geo)?;
    Some(cartesian_au_to_ecliptic([
        p[0] - s[0],
        p[1] - s[1],
        p[2] - s[2],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{EclipticCoordinates, Latitude, Longitude};

    #[test]
    fn velocity_round_trips_through_cartesian() {
        let s = SphericalState {
            lon_rad: 0.7,
            lat_rad: 0.2,
            dist_au: 1.5,
            lon_rate_rad_per_day: 0.01,
            lat_rate_rad_per_day: -0.003,
            dist_rate_au_per_day: 0.002,
        };
        let c = spherical_state_to_cartesian(s);
        let back = cartesian_state_to_spherical(c);
        assert!((back.lon_rad - s.lon_rad).abs() < 1e-10);
        assert!((back.lat_rad - s.lat_rad).abs() < 1e-10);
        assert!((back.dist_au - s.dist_au).abs() < 1e-10);
        assert!((back.lon_rate_rad_per_day - s.lon_rate_rad_per_day).abs() < 1e-10);
        assert!((back.lat_rate_rad_per_day - s.lat_rate_rad_per_day).abs() < 1e-10);
        assert!((back.dist_rate_au_per_day - s.dist_rate_au_per_day).abs() < 1e-10);
    }

    /// Forward-conversion ground-truth test: hand-derived Cartesian velocity for
    /// lon=0.7 rad, lat=0.2 rad, r=1.5 AU, dλ=0.01 rad/day, dβ=−0.003 rad/day, dr=0.002 AU/day.
    ///
    /// Chain-rule derivation:
    ///   vx = dr·cb·cl − r·sb·cl·dβ − r·cb·sl·dλ
    ///      = 0.002·0.980067·0.764842 − 1.5·0.198669·0.764842·(−0.003) − 1.5·0.980067·0.644218·0.01
    ///      ≈ −0.007287672746774
    ///   vy = dr·cb·sl − r·sb·sl·dβ + r·cb·cl·dλ
    ///      ≈  0.013082634760084
    ///   vz = dr·sb + r·cb·dβ
    ///      ≈ −0.004012960938695
    #[test]
    fn forward_conversion_matches_hand_derived_velocity() {
        let s = SphericalState {
            lon_rad: 0.7,
            lat_rad: 0.2,
            dist_au: 1.5,
            lon_rate_rad_per_day: 0.01,
            lat_rate_rad_per_day: -0.003,
            dist_rate_au_per_day: 0.002,
        };
        let c = spherical_state_to_cartesian(s);
        assert!(
            (c.vel_au_per_day[0] - (-0.007_287_672_746_774_f64)).abs() < 1e-10,
            "vx={} expected≈-0.007287672746774",
            c.vel_au_per_day[0]
        );
        assert!(
            (c.vel_au_per_day[1] - 0.013_082_634_760_084_f64).abs() < 1e-10,
            "vy={} expected≈0.013082634760084",
            c.vel_au_per_day[1]
        );
        assert!(
            (c.vel_au_per_day[2] - (-0.004_012_960_938_695_f64)).abs() < 1e-10,
            "vz={} expected≈-0.004012960938695",
            c.vel_au_per_day[2]
        );
    }

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

    #[test]
    fn spherical_to_cartesian_is_publicly_reachable_and_round_trips() {
        // Reach it through the crate root to prove the re-export exists.
        let s = crate::SphericalState {
            lon_rad: 1.0,
            lat_rad: 0.1,
            dist_au: 0.0025,
            lon_rate_rad_per_day: 0.2,
            lat_rate_rad_per_day: -0.01,
            dist_rate_au_per_day: 1e-6,
        };
        let c = crate::spherical_state_to_cartesian(s);
        let back = crate::cartesian_state_to_spherical(c);
        assert!((back.lon_rad - s.lon_rad).abs() < 1e-12);
        assert!((back.dist_au - s.dist_au).abs() < 1e-15);
    }

    mod properties {
        use super::*;
        use proptest::prelude::*;

        // Small positive helper: circular longitude difference in [0, 360).
        fn lon_gap(a: f64, b: f64) -> f64 {
            (a - b).rem_euclid(360.0)
        }

        // Small positive helper: circular longitude difference in [0, TAU) radians.
        fn lon_gap_rad(a: f64, b: f64) -> f64 {
            (a - b).rem_euclid(std::f64::consts::TAU)
        }

        proptest! {
            #[test]
            fn ecliptic_cartesian_roundtrips(
                lon in 0.0f64..360.0,
                lat in -85.0f64..85.0,   // away from the poles: longitude is ill-conditioned near ±90°
                dist in 0.1f64..100.0,
            ) {
                let v = ecliptic_to_cartesian_au(&ec(lon, lat, dist)).unwrap();
                let back = cartesian_au_to_ecliptic(v);
                let g = lon_gap(back.longitude.degrees(), lon);
                prop_assert!(!(1e-7..=360.0 - 1e-7).contains(&g), "lon {lon} -> {}", back.longitude.degrees());
                prop_assert!((back.latitude.degrees() - lat).abs() < 1e-7);
                prop_assert!((back.distance_au.unwrap() - dist).abs() < 1e-7 * dist);
            }

            #[test]
            fn spherical_cartesian_state_roundtrips(
                lon in 0.0f64..std::f64::consts::TAU,
                lat in -1.4f64..1.4,     // radians, away from ±π/2
                dist in 0.1f64..100.0,
                dlon in -0.1f64..0.1,
                dlat in -0.1f64..0.1,
                ddist in -0.1f64..0.1,
            ) {
                let s = SphericalState {
                    lon_rad: lon, lat_rad: lat, dist_au: dist,
                    lon_rate_rad_per_day: dlon, lat_rate_rad_per_day: dlat, dist_rate_au_per_day: ddist,
                };
                let back = cartesian_state_to_spherical(spherical_state_to_cartesian(s));
                let g = lon_gap_rad(back.lon_rad, lon);
                prop_assert!(
                    !(1e-9..=std::f64::consts::TAU - 1e-9).contains(&g),
                    "lon {lon} -> {}",
                    back.lon_rad
                );
                prop_assert!((back.lat_rad - lat).abs() < 1e-9);
                prop_assert!((back.dist_au - dist).abs() < 1e-9 * dist);
                prop_assert!((back.lon_rate_rad_per_day - dlon).abs() < 1e-9);
                prop_assert!((back.lat_rate_rad_per_day - dlat).abs() < 1e-9);
                prop_assert!((back.dist_rate_au_per_day - ddist).abs() < 1e-9);
            }

            #[test]
            fn helio_geo_inverse_via_sun(
                plon in 0.0f64..360.0, plat in -85.0f64..85.0, pdist in 0.5f64..50.0,
                slon in 0.0f64..360.0, slat in -85.0f64..85.0, sdist in 0.5f64..2.0,
            ) {
                let planet_geo = ec(plon, plat, pdist);
                let sun_geo = ec(slon, slat, sdist);
                let helio = heliocentric_from_geocentric(&planet_geo, &sun_geo).unwrap();
                let back = geocentric_from_heliocentric(&helio, &sun_geo).unwrap();
                let g = lon_gap(back.longitude.degrees(), plon);
                prop_assert!(!(1e-6..=360.0 - 1e-6).contains(&g), "plon {plon} -> {}", back.longitude.degrees());
                prop_assert!((back.latitude.degrees() - plat).abs() < 1e-6);
                prop_assert!((back.distance_au.unwrap() - pdist).abs() < 1e-6 * pdist);
            }
        }
    }
}
