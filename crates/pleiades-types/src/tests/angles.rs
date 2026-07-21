use crate::*;

#[test]
fn angle_normalization_wraps_correctly() {
    assert_eq!(
        Angle::from_degrees(-30.0).normalized_0_360().degrees(),
        330.0
    );
    assert_eq!(
        Angle::from_degrees(390.0).normalized_0_360().degrees(),
        30.0
    );
    assert_eq!(Angle::from_degrees(720.0).normalized_0_360().degrees(), 0.0);
    assert_eq!(
        Angle::from_degrees(190.0).normalized_signed().degrees(),
        -170.0
    );
    assert_eq!(
        Angle::from_degrees(180.0).normalized_signed().degrees(),
        -180.0
    );
    assert_eq!(
        Angle::from_degrees(-180.0).normalized_signed().degrees(),
        -180.0
    );
    assert_eq!(Longitude::from_degrees(-720.0).degrees(), 0.0);
    assert_eq!(Longitude::from_degrees(360.0).degrees(), 0.0);
}

#[test]
fn longitude_is_always_normalized() {
    assert_eq!(Longitude::from_degrees(390.0).degrees(), 30.0);
    assert_eq!(Longitude::from(Angle::from_degrees(-30.0)).degrees(), 330.0);
}

#[test]
fn angle_is_finite_and_latitude_from_angle_preserve_values() {
    // is_finite must track the value, not a constant (kills -> true / -> false).
    assert!(Angle::from_degrees(42.0).is_finite());
    assert!(!Angle::from_degrees(f64::NAN).is_finite());
    assert!(!Angle::from_degrees(f64::INFINITY).is_finite());

    // From<Angle> for Latitude must carry the value, not Default (0.0).
    let lat = Latitude::from(Angle::from_degrees(-33.5));
    assert!((lat.degrees() - (-33.5)).abs() < 1e-12);
}

mod angle_properties {
    use super::*;
    use proptest::prelude::*;

    // Bounded finite degrees: wide enough to exercise many 360° wraps, small
    // enough that absolute floating-point tolerances stay tight. Non-finite
    // inputs are deliberately excluded — normalization assumes finite values.
    fn finite_degrees() -> impl Strategy<Value = f64> {
        -1.0e4f64..1.0e4f64
    }

    proptest! {
        #[test]
        fn normalized_0_360_in_range(d in finite_degrees()) {
            let n = Angle::from_degrees(d).normalized_0_360().degrees();
            prop_assert!((0.0..360.0).contains(&n), "d={d} n={n}");
        }

        #[test]
        fn normalized_0_360_idempotent(d in finite_degrees()) {
            let once = Angle::from_degrees(d).normalized_0_360();
            let twice = once.normalized_0_360();
            prop_assert_eq!(once.degrees(), twice.degrees());
        }

        #[test]
        fn normalized_0_360_congruent_mod_360(d in finite_degrees()) {
            let n = Angle::from_degrees(d).normalized_0_360().degrees();
            let k = ((d - n) / 360.0).round();
            prop_assert!((d - n - k * 360.0).abs() < 1e-9, "d={d} n={n} k={k}");
        }

        #[test]
        fn normalized_signed_in_range(d in finite_degrees()) {
            let n = Angle::from_degrees(d).normalized_signed().degrees();
            prop_assert!((-180.0..180.0).contains(&n), "d={d} n={n}");
        }

        #[test]
        fn normalized_signed_idempotent(d in finite_degrees()) {
            let once = Angle::from_degrees(d).normalized_signed();
            let twice = once.normalized_signed();
            prop_assert_eq!(once.degrees(), twice.degrees());
        }

        #[test]
        fn degree_radian_roundtrip(d in finite_degrees()) {
            let back = Angle::from_radians(Angle::from_degrees(d).radians()).degrees();
            prop_assert!((back - d).abs() < 1e-9, "d={d} back={back}");
        }

        #[test]
        fn longitude_constructor_normalizes(d in finite_degrees()) {
            let l = Longitude::from_degrees(d).degrees();
            prop_assert!((0.0..360.0).contains(&l), "d={d} l={l}");
        }
    }
}
