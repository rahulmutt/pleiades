use super::*;
use pleiades_types::{EclipticCoordinates, Latitude, Longitude, Motion};

fn place(lon: f64, lat: f64, dist: Option<f64>) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        dist,
    )
}

fn sample(jd: f64, corrected: EclipticCoordinates, base: EclipticCoordinates) -> CorrectionSample {
    CorrectionSample {
        julian_day: jd,
        correction: Correction::between(&corrected, &base),
    }
}

#[test]
fn adds_the_correction_rate_to_each_channel() {
    // Correction grows by (0.2°, -0.1°, 0.01 AU) over one day.
    let earlier = sample(
        10.0,
        place(100.1, 1.0, Some(2.0)),
        place(100.0, 1.0, Some(2.0)),
    );
    let later = sample(
        11.0,
        place(100.3, 0.9, Some(2.01)),
        place(100.0, 1.0, Some(2.0)),
    );
    let out = apparent_motion(
        Motion::new(Some(1.0), Some(0.5), Some(0.0)),
        &earlier,
        &later,
    );
    assert!((out.longitude_deg_per_day.unwrap() - 1.2).abs() < 1e-12);
    assert!((out.latitude_deg_per_day.unwrap() - 0.4).abs() < 1e-12);
    assert!((out.distance_au_per_day.unwrap() - 0.01).abs() < 1e-12);
}

#[test]
fn correction_rate_wraps_across_zero() {
    // Corrected place crosses 0°: 359.9° then 0.1°, base fixed at 359.8°.
    let earlier = sample(10.0, place(359.9, 0.0, None), place(359.8, 0.0, None));
    let later = sample(11.0, place(0.1, 0.0, None), place(359.8, 0.0, None));
    let out = apparent_motion(Motion::new(Some(0.0), Some(0.0), None), &earlier, &later);
    assert!((out.longitude_deg_per_day.unwrap() - 0.2).abs() < 1e-9);
}

#[test]
fn empty_base_channels_stay_empty() {
    let earlier = sample(10.0, place(1.0, 0.0, Some(1.0)), place(0.0, 0.0, Some(1.0)));
    let later = sample(11.0, place(2.0, 0.0, Some(1.0)), place(0.0, 0.0, Some(1.0)));
    let out = apparent_motion(Motion::new(None, Some(0.0), None), &earlier, &later);
    assert_eq!(out.longitude_deg_per_day, None);
    assert_eq!(out.latitude_deg_per_day, Some(0.0));
    assert_eq!(out.distance_au_per_day, None);
}

#[test]
fn distance_speed_is_unchanged_when_a_sample_has_no_distance() {
    let earlier = sample(10.0, place(1.0, 0.0, None), place(0.0, 0.0, Some(1.0)));
    let later = sample(11.0, place(2.0, 0.0, Some(1.5)), place(0.0, 0.0, Some(1.0)));
    let out = apparent_motion(
        Motion::new(Some(0.0), Some(0.0), Some(0.25)),
        &earlier,
        &later,
    );
    assert_eq!(out.distance_au_per_day, Some(0.25));
}

#[test]
fn zero_span_gives_no_speed() {
    let a = sample(10.0, place(1.0, 0.0, Some(1.0)), place(0.0, 0.0, Some(1.0)));
    let out = apparent_motion(Motion::new(Some(1.0), Some(1.0), Some(1.0)), &a, &a);
    assert_eq!(out, Motion::new(None, None, None));
}

#[test]
fn non_finite_span_gives_no_speed() {
    let a = sample(10.0, place(1.0, 0.0, None), place(0.0, 0.0, None));
    let b = sample(f64::NAN, place(1.0, 0.0, None), place(0.0, 0.0, None));
    let out = apparent_motion(Motion::new(Some(1.0), Some(1.0), None), &a, &b);
    assert_eq!(out, Motion::new(None, None, None));
}
