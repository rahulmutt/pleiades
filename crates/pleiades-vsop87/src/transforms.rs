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
    let cos_latitude = libm::cos(latitude_rad);
    HeliocentricCoordinates {
        xh: radius_au * cos_latitude * libm::cos(longitude_rad),
        yh: radius_au * cos_latitude * libm::sin(longitude_rad),
        zh: radius_au * libm::sin(latitude_rad),
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
    let mut e = m + eccentricity * libm::sin(m) * (1.0 + eccentricity * libm::cos(m));
    for _ in 0..10 {
        let delta = (e - eccentricity * libm::sin(e) - m) / (1.0 - eccentricity * libm::cos(e));
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
    let numerator = (1.0 + eccentricity).sqrt() * libm::sin(e / 2.0);
    let denominator = (1.0 - eccentricity).sqrt() * libm::cos(e / 2.0);
    (2.0 * libm::atan2(numerator, denominator))
        .to_degrees()
        .rem_euclid(360.0)
}
