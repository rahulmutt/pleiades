use super::*;
use crate::ephemeris::spherical_to_cartesian;
use pleiades_types::Motion;

fn full(lon: f64, lat: f64, dist: f64) -> Option<Motion> {
    Some(Motion::new(Some(lon), Some(lat), Some(dist)))
}

#[test]
fn circular_coplanar_orbit_has_the_analytic_velocity() {
    // r = 2 AU, longitude 90°, moving at 1°/day in the ecliptic plane:
    // the velocity is tangential, -x direction, magnitude r·n.
    let v = cartesian_velocity(90.0, 0.0, 2.0, full(1.0, 0.0, 0.0)).unwrap();
    let speed = 2.0 * 1.0_f64.to_radians();
    assert!((v[0] + speed).abs() < 1e-15, "{v:?}");
    assert!(v[1].abs() < 1e-15, "{v:?}");
    assert!(v[2].abs() < 1e-15, "{v:?}");
}

#[test]
fn rates_round_trip_through_cartesian() {
    let (lon, lat, r) = (217.3, -6.4, 5.2);
    let (lon_rate, lat_rate, r_rate) = (0.083, -0.0021, 0.0004);
    let p = spherical_to_cartesian(lon, lat, r);
    let v = cartesian_velocity(lon, lat, r, full(lon_rate, lat_rate, r_rate)).unwrap();
    let back = spherical_rates(p, v);
    assert!((back.longitude_deg_per_day.unwrap() - lon_rate).abs() < 1e-13);
    assert!((back.latitude_deg_per_day.unwrap() - lat_rate).abs() < 1e-13);
    assert!((back.distance_au_per_day.unwrap() - r_rate).abs() < 1e-13);
}

#[test]
fn rates_match_a_central_difference_on_an_inclined_orbit() {
    // Position as an explicit function of time; velocity by a tiny central
    // difference of the Cartesian position, independent of cartesian_velocity.
    let at =
        |t: f64| spherical_to_cartesian(40.0 + 0.5 * t, 7.0 * (0.03 * t).sin(), 1.5 + 0.01 * t);
    let h = 1e-4;
    let (a, b, p) = (at(-h), at(h), at(0.0));
    let v = [
        (b[0] - a[0]) / (2.0 * h),
        (b[1] - a[1]) / (2.0 * h),
        (b[2] - a[2]) / (2.0 * h),
    ];
    let rates = spherical_rates(p, v);
    assert!((rates.longitude_deg_per_day.unwrap() - 0.5).abs() < 1e-7);
    // d/dt[7 sin(0.03 t)] at t = 0 is 0.21.
    assert!((rates.latitude_deg_per_day.unwrap() - 0.21).abs() < 1e-7);
    assert!((rates.distance_au_per_day.unwrap() - 0.01).abs() < 1e-7);
}

#[test]
fn missing_rate_channel_gives_no_velocity() {
    assert!(cartesian_velocity(10.0, 1.0, 1.0, None).is_none());
    let partial = Some(Motion::new(Some(1.0), None, Some(0.0)));
    assert!(cartesian_velocity(10.0, 1.0, 1.0, partial).is_none());
}

#[test]
fn pole_has_no_longitude_or_latitude_rate() {
    let rates = spherical_rates([0.0, 0.0, 1.0], [0.01, 0.0, 0.002]);
    assert_eq!(rates.longitude_deg_per_day, None);
    assert_eq!(rates.latitude_deg_per_day, None);
    assert!((rates.distance_au_per_day.unwrap() - 0.002).abs() < 1e-15);
}

#[test]
fn origin_has_no_rates() {
    assert_eq!(
        spherical_rates([0.0; 3], [0.01, 0.0, 0.0]),
        Motion::new(None, None, None)
    );
}
