//! Angular-arithmetic helpers used by the series evaluation routines.

pub(crate) fn normalize_degrees(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

pub(crate) fn signed_longitude_delta_degrees(start: f64, end: f64) -> f64 {
    (end - start + 180.0).rem_euclid(360.0) - 180.0
}
