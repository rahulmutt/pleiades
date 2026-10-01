//! Conversions between spherical ecliptic rates and Cartesian velocities.
//!
//! Angles are degrees and degrees per day at the interface, distances are
//! astronomical units and AU per day. The frame is whatever frame the
//! spherical coordinates are in; these are pure geometry.

use pleiades_types::Motion;

/// Cartesian velocity (AU/day) of a point at spherical `(lon, lat, r)` moving
/// at the spherical rates in `motion`. `None` unless all three rates are known:
/// a velocity built from a partial rate set would describe a different motion.
pub(crate) fn cartesian_velocity(
    lon_deg: f64,
    lat_deg: f64,
    r_au: f64,
    motion: Option<Motion>,
) -> Option<[f64; 3]> {
    let motion = motion?;
    let lon_rate = motion.longitude_deg_per_day?.to_radians();
    let lat_rate = motion.latitude_deg_per_day?.to_radians();
    let r_rate = motion.distance_au_per_day?;
    let (sin_lon, cos_lon) = lon_deg.to_radians().sin_cos();
    let (sin_lat, cos_lat) = lat_deg.to_radians().sin_cos();
    Some([
        r_rate * cos_lat * cos_lon
            - r_au * sin_lat * cos_lon * lat_rate
            - r_au * cos_lat * sin_lon * lon_rate,
        r_rate * cos_lat * sin_lon - r_au * sin_lat * sin_lon * lat_rate
            + r_au * cos_lat * cos_lon * lon_rate,
        r_rate * sin_lat + r_au * cos_lat * lat_rate,
    ])
}

/// Spherical rates (deg/day, deg/day, AU/day) of a point with Cartesian
/// `position` (AU) and `velocity` (AU/day). A rate that is undefined — the
/// longitude and latitude rates on the polar axis, every rate at the origin —
/// is `None` rather than a non-finite number.
pub(crate) fn spherical_rates(position: [f64; 3], velocity: [f64; 3]) -> Motion {
    let [x, y, z] = position;
    let [vx, vy, vz] = velocity;
    let rho2 = x * x + y * y;
    let r = (rho2 + z * z).sqrt();
    let planar = x * vx + y * vy;
    let lon_rate = ((x * vy - y * vx) / rho2).to_degrees();
    let lat_rate = ((vz * rho2 - z * planar) / (r * r * rho2.sqrt())).to_degrees();
    let r_rate = (planar + z * vz) / r;
    let finite = |value: f64| value.is_finite().then_some(value);
    Motion::new(finite(lon_rate), finite(lat_rate), finite(r_rate))
}

#[cfg(test)]
mod tests;
