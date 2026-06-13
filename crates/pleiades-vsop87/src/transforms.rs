/// Heliocentric Cartesian coordinates derived from spherical VSOP87B output.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HeliocentricCoordinates {
    pub(crate) xh: f64,
    pub(crate) yh: f64,
    pub(crate) zh: f64,
}

pub(crate) fn spherical_lbr_to_cartesian(
    longitude_rad: f64,
    latitude_rad: f64,
    radius_au: f64,
) -> HeliocentricCoordinates {
    let cos_latitude = latitude_rad.cos();
    HeliocentricCoordinates {
        xh: radius_au * cos_latitude * longitude_rad.cos(),
        yh: radius_au * cos_latitude * longitude_rad.sin(),
        zh: radius_au * latitude_rad.sin(),
    }
}

pub(crate) fn normalize_degrees(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

pub(crate) fn signed_longitude_delta_degrees(start: f64, end: f64) -> f64 {
    (end - start + 180.0).rem_euclid(360.0) - 180.0
}

pub(crate) fn pluralize_body_path(count: usize) -> &'static str {
    if count == 1 {
        "body path"
    } else {
        "body paths"
    }
}

pub(crate) fn solve_kepler(mean_anomaly_degrees: f64, eccentricity: f64) -> f64 {
    let m = mean_anomaly_degrees.to_radians();
    let mut e = m + eccentricity * m.sin() * (1.0 + eccentricity * m.cos());
    for _ in 0..10 {
        let delta = (e - eccentricity * e.sin() - m) / (1.0 - eccentricity * e.cos());
        e -= delta;
        if delta.abs() < 1e-12 {
            break;
        }
    }
    e.to_degrees()
}

pub(crate) fn true_anomaly_from_eccentric(
    eccentric_anomaly_degrees: f64,
    eccentricity: f64,
) -> f64 {
    let e = eccentric_anomaly_degrees.to_radians();
    let numerator = (1.0 + eccentricity).sqrt() * (e / 2.0).sin();
    let denominator = (1.0 - eccentricity).sqrt() * (e / 2.0).cos();
    (2.0 * numerator.atan2(denominator))
        .to_degrees()
        .rem_euclid(360.0)
}
