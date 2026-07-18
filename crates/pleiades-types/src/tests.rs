use super::*;
use core::time::Duration;

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
fn celestial_body_classes_cover_the_built_in_catalog() {
    assert_eq!(CelestialBody::Sun.class(), CelestialBodyClass::Luminary);
    assert_eq!(CelestialBody::Moon.class(), CelestialBodyClass::Luminary);
    assert_eq!(
        CelestialBody::Mercury.class(),
        CelestialBodyClass::MajorPlanet
    );
    assert_eq!(
        CelestialBody::Pluto.class(),
        CelestialBodyClass::MajorPlanet
    );
    assert_eq!(
        CelestialBody::MeanNode.class(),
        CelestialBodyClass::LunarPoint
    );
    assert_eq!(
        CelestialBody::TruePerigee.class(),
        CelestialBodyClass::LunarPoint
    );
    assert_eq!(
        CelestialBody::Ceres.class(),
        CelestialBodyClass::BuiltInAsteroid
    );
    assert_eq!(
        CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")).class(),
        CelestialBodyClass::Custom
    );
    assert_eq!(CelestialBodyClass::Luminary.label(), "luminary");
    assert_eq!(
        CelestialBodyClass::BuiltInAsteroid.to_string(),
        "built-in asteroid"
    );
}

#[test]
fn ecliptic_to_equatorial_preserves_zero_obliquity_identity() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(123.45),
        Latitude::from_degrees(-6.75),
        Some(0.123),
    );

    assert_eq!(ecliptic.validate(), Ok(()));

    let equatorial = ecliptic.to_equatorial(Angle::from_degrees(0.0));

    assert_eq!(equatorial.validate(), Ok(()));
    assert_eq!(equatorial.right_ascension.degrees(), 123.45);
    assert!((equatorial.declination.degrees() + 6.75).abs() < 1e-12);
    assert_eq!(equatorial.distance_au, Some(0.123));
}

#[test]
fn ecliptic_to_equatorial_rotates_by_mean_obliquity() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(90.0),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );

    assert_eq!(ecliptic.validate(), Ok(()));

    let equatorial = ecliptic.to_equatorial(Angle::from_degrees(23.439_291_11));

    assert_eq!(equatorial.validate(), Ok(()));
    assert_eq!(equatorial.right_ascension.degrees(), 90.0);
    assert!((equatorial.declination.degrees() - 23.439_291_11).abs() < 1e-10);
    assert_eq!(equatorial.distance_au, Some(1.0));
}

#[test]
fn instant_mean_obliquity_matches_the_shared_cubic_approximation() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);

    assert_eq!(instant.mean_obliquity().degrees(), 23.439_291_111_111_11);
}

#[test]
fn instant_has_a_compact_display() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);

    assert_eq!(instant.summary_line(), "JD 2451545 TDB");
    assert_eq!(instant.to_string(), "JD 2451545 TDB");
}

#[test]
fn observer_location_has_a_compact_display() {
    let observer = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(-0.1),
        Some(35.75),
    );

    assert_eq!(
        observer.summary_line(),
        "latitude=51.5°, longitude=359.9°, elevation=35.750 m"
    );
    assert_eq!(observer.to_string(), observer.summary_line());
}

#[test]
fn equatorial_to_ecliptic_round_trip_uses_the_same_obliquity() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(123.45),
        Latitude::from_degrees(-6.75),
        Some(0.123),
    );
    let obliquity = Angle::from_degrees(23.439_291_11);

    assert_eq!(ecliptic.validate(), Ok(()));

    let equatorial = ecliptic.to_equatorial(obliquity);
    assert_eq!(equatorial.validate(), Ok(()));
    let round_trip = equatorial.to_ecliptic(obliquity);

    assert_eq!(round_trip.validate(), Ok(()));
    assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
    assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
    assert_eq!(round_trip.distance_au, Some(0.123));
}

#[test]
fn mean_obliquity_round_trip_stays_stable_across_quadrants() {
    let cases = [
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(359.2),
                Latitude::from_degrees(11.75),
                None,
            ),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(27.5),
                Latitude::from_degrees(-33.25),
                Some(2.5),
            ),
            Instant::new(JulianDay::from_days(2_459_000.5), TimeScale::Tt),
        ),
    ];

    for (ecliptic, instant) in cases {
        assert_eq!(ecliptic.validate(), Ok(()));
        let obliquity = instant.mean_obliquity();
        let equatorial = ecliptic.to_equatorial(obliquity);
        assert_eq!(equatorial.validate(), Ok(()));
        let round_trip = equatorial.to_ecliptic(obliquity);

        assert_eq!(round_trip.validate(), Ok(()));
        assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
        assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
        assert_eq!(round_trip.distance_au, ecliptic.distance_au);
    }
}

#[test]
fn mean_obliquity_round_trip_stays_stable_near_the_poles() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let obliquity = instant.mean_obliquity();

    for ecliptic in [
        EclipticCoordinates::new(
            Longitude::from_degrees(0.025),
            Latitude::from_degrees(89.75),
            Some(0.42),
        ),
        EclipticCoordinates::new(
            Longitude::from_degrees(179.975),
            Latitude::from_degrees(-89.75),
            Some(0.42),
        ),
    ] {
        assert_eq!(ecliptic.validate(), Ok(()));
        let equatorial = ecliptic.to_equatorial(obliquity);
        assert_eq!(equatorial.validate(), Ok(()));
        let round_trip = equatorial.to_ecliptic(obliquity);

        assert_eq!(round_trip.validate(), Ok(()));
        assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
        assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
        assert_eq!(round_trip.distance_au, Some(0.42));
    }
}

#[test]
fn ecliptic_to_equatorial_normalizes_negative_right_ascension() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(180.0),
        Latitude::from_degrees(30.0),
        None,
    );
    let obliquity = Angle::from_degrees(23.439_291_11);

    assert_eq!(ecliptic.validate(), Ok(()));
    let equatorial = ecliptic.to_equatorial(obliquity);

    assert_eq!(equatorial.validate(), Ok(()));
    assert!(equatorial.right_ascension.degrees() >= 0.0);
    assert!(equatorial.right_ascension.degrees() < 360.0);
    assert!((equatorial.right_ascension.degrees() - 192.934_084_332_518_07).abs() < 1e-10);
    assert!((equatorial.declination.degrees() - 27.305_898_332_307_97).abs() < 1e-10);
}

#[test]
fn equatorial_to_ecliptic_round_trip_preserves_negative_declination_near_wraparound() {
    let equatorial = EquatorialCoordinates::new(
        Angle::from_degrees(359.5),
        Latitude::from_degrees(-27.5),
        Some(3.25),
    );
    let obliquity = Angle::from_degrees(23.439_291_11);

    assert_eq!(equatorial.validate(), Ok(()));

    let ecliptic = equatorial.to_ecliptic(obliquity);
    assert_eq!(ecliptic.validate(), Ok(()));
    let round_trip = ecliptic.to_equatorial(obliquity);

    assert_eq!(round_trip.validate(), Ok(()));
    assert!(
        (round_trip.right_ascension.degrees() - equatorial.right_ascension.degrees()).abs() < 1e-10
    );
    assert!((round_trip.declination.degrees() - equatorial.declination.degrees()).abs() < 1e-10);
    assert_eq!(round_trip.distance_au, equatorial.distance_au);
}

#[test]
fn equatorial_to_ecliptic_treats_negative_right_ascension_as_normalized_angle() {
    let obliquity = Angle::from_degrees(23.439_291_11);
    let normalized = EquatorialCoordinates::new(
        Angle::from_degrees(359.75),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );
    let wrapped = EquatorialCoordinates::new(
        Angle::from_degrees(-0.25),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );

    let normalized_ecliptic = normalized.to_ecliptic(obliquity);
    let wrapped_ecliptic = wrapped.to_ecliptic(obliquity);

    assert_eq!(normalized_ecliptic.validate(), Ok(()));
    assert_eq!(wrapped_ecliptic.validate(), Ok(()));
    assert!(
        (normalized_ecliptic.longitude.degrees() - wrapped_ecliptic.longitude.degrees()).abs()
            < 1e-10
    );
    assert!(
        (normalized_ecliptic.latitude.degrees() - wrapped_ecliptic.latitude.degrees()).abs()
            < 1e-10
    );
    assert_eq!(
        normalized_ecliptic.distance_au,
        wrapped_ecliptic.distance_au
    );

    let round_trip = wrapped_ecliptic.to_equatorial(obliquity);
    assert_eq!(round_trip.validate(), Ok(()));
    assert!(round_trip.right_ascension.degrees() >= 0.0);
    assert!(round_trip.right_ascension.degrees() < 360.0);
    assert!((round_trip.right_ascension.degrees() - 359.75).abs() < 1e-10);
}

#[test]
fn equatorial_to_ecliptic_treats_full_turn_right_ascension_as_normalized_angle() {
    let obliquity = Angle::from_degrees(23.439_291_11);
    let normalized = EquatorialCoordinates::new(
        Angle::from_degrees(0.25),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );
    let wrapped = EquatorialCoordinates::new(
        Angle::from_degrees(360.25),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );

    let normalized_ecliptic = normalized.to_ecliptic(obliquity);
    let wrapped_ecliptic = wrapped.to_ecliptic(obliquity);

    assert_eq!(normalized_ecliptic.validate(), Ok(()));
    assert_eq!(wrapped_ecliptic.validate(), Ok(()));
    assert!(
        (normalized_ecliptic.longitude.degrees() - wrapped_ecliptic.longitude.degrees()).abs()
            < 1e-10
    );
    assert!(
        (normalized_ecliptic.latitude.degrees() - wrapped_ecliptic.latitude.degrees()).abs()
            < 1e-10
    );
    assert_eq!(
        normalized_ecliptic.distance_au,
        wrapped_ecliptic.distance_au
    );

    let round_trip = wrapped_ecliptic.to_equatorial(obliquity);
    assert_eq!(round_trip.validate(), Ok(()));
    assert!(round_trip.right_ascension.degrees() >= 0.0);
    assert!(round_trip.right_ascension.degrees() < 360.0);
    assert!((round_trip.right_ascension.degrees() - 0.25).abs() < 1e-10);
}

#[test]
fn built_in_body_names_are_stable() {
    assert_eq!(CelestialBody::Sun.built_in_name(), Some("Sun"));
    assert_eq!(CelestialBody::Sun.to_string(), "Sun");
    assert_eq!(
        CelestialBody::MeanApogee.built_in_name(),
        Some("Mean Apogee")
    );
    assert_eq!(CelestialBody::TruePerigee.to_string(), "True Perigee");
    assert_eq!(
        CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")).built_in_name(),
        None
    );
    assert_eq!(
        CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")).to_string(),
        "asteroid:433-Eros"
    );
}

#[test]
fn custom_body_id_validate_rejects_blank_padding_and_separators() {
    assert_eq!(
        CustomBodyId::new("", "433-Eros")
            .validate()
            .expect_err("blank catalogs should be rejected")
            .to_string(),
        "custom body id catalog must not be blank"
    );

    assert_eq!(
        CustomBodyId::new("asteroid", " 433-Eros ")
            .validate()
            .expect_err("padded designations should be rejected")
            .to_string(),
        "custom body id designation must not have leading or trailing whitespace"
    );

    assert_eq!(
        CustomBodyId::new("asteroid:catalog", "433-Eros")
            .validate()
            .expect_err("separator characters should be rejected")
            .to_string(),
        "custom body id catalog must not contain ':'"
    );
}

#[test]
fn celestial_body_validate_reuses_custom_body_checks() {
    assert!(CelestialBody::Sun.validate().is_ok());

    assert_eq!(
        CelestialBody::Custom(CustomBodyId::new("asteroid", " 433-Eros "))
            .validate()
            .expect_err("custom body identifiers should be validated")
            .to_string(),
        "custom body id designation must not have leading or trailing whitespace"
    );
}

#[test]
fn custom_house_system_display_includes_aliases_and_notes() {
    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("MCH".to_string());
    custom.aliases.push("Test Houses".to_string());
    custom.notes = Some("based on a user-defined formula".to_string());

    assert_eq!(
        custom.to_string(),
        "My Custom Houses [aliases: MCH, Test Houses] (based on a user-defined formula)"
    );
}

#[test]
fn custom_house_system_validate_rejects_whitespace_and_duplicate_aliases() {
    assert_eq!(
        CustomHouseSystem::new("   ")
            .validate()
            .expect_err("blank house system names should be rejected")
            .to_string(),
        "custom house system name must not be blank"
    );

    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("MCH".to_string());
    custom.aliases.push("MCH".to_string());

    assert_eq!(
        custom
            .validate()
            .expect_err("duplicate aliases should be rejected")
            .to_string(),
        "custom house system aliases must be unique: duplicate MCH"
    );

    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("MCH".to_string());
    custom.aliases.push("mch".to_string());

    assert_eq!(
        custom
            .validate()
            .expect_err("case-normalized aliases should be rejected")
            .to_string(),
        "custom house system aliases must be unique: duplicate mch"
    );

    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("my custom houses".to_string());

    assert_eq!(
        custom
            .validate()
            .expect_err("aliases should not duplicate the canonical name")
            .to_string(),
        "custom house system aliases must be unique: duplicate my custom houses"
    );

    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.notes = Some("  ".to_string());

    assert_eq!(
        custom
            .validate()
            .expect_err("blank notes should be rejected")
            .to_string(),
        "custom house system notes must not be blank"
    );
}

#[test]
fn custom_house_system_validate_against_reserved_labels_rejects_builtin_collisions() {
    let custom = CustomHouseSystem::new("Equal");
    assert_eq!(
        custom
            .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Equal"))
            .expect_err("built-in labels should be rejected")
            .to_string(),
        "custom house system name must not match a built-in label: Equal"
    );

    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("Whole Sign".to_string());

    assert_eq!(
        custom
            .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Whole Sign"))
            .expect_err("built-in aliases should be rejected")
            .to_string(),
        "custom house system alias[0] must not match a built-in label: Whole Sign"
    );
}

#[test]
fn house_systems_have_stable_display_names() {
    assert_eq!(HouseSystem::Placidus.to_string(), "Placidus");
    assert_eq!(HouseSystem::EqualMidheaven.to_string(), "Equal (MC)");
    assert_eq!(HouseSystem::Carter.to_string(), "Carter (poli-equatorial)");
    assert_eq!(HouseSystem::Gauquelin.to_string(), "Gauquelin sectors");

    let custom = CustomHouseSystem::new("My Custom Houses");
    assert_eq!(
        HouseSystem::Custom(custom.clone()).to_string(),
        custom.to_string()
    );
}

#[test]
fn house_system_validate_reuses_the_structured_validator() {
    assert!(HouseSystem::WholeSign.validate().is_ok());

    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("MCH".to_string());
    custom.aliases.push("mch".to_string());

    assert_eq!(
        HouseSystem::Custom(custom)
            .validate()
            .expect_err("custom house systems should validate their aliases")
            .to_string(),
        "custom house system aliases must be unique: duplicate mch"
    );
}

#[test]
fn time_scales_have_stable_display_names() {
    assert_eq!(TimeScale::Utc.to_string(), "UTC");
    assert_eq!(TimeScale::Ut1.to_string(), "UT1");
    assert_eq!(TimeScale::Tt.to_string(), "TT");
    assert_eq!(TimeScale::Tdb.to_string(), "TDB");
}

#[test]
fn coordinate_frames_have_stable_display_names() {
    assert_eq!(CoordinateFrame::Ecliptic.to_string(), "Ecliptic");
    assert_eq!(CoordinateFrame::Equatorial.to_string(), "Equatorial");
}

#[test]
fn zodiac_modes_have_stable_display_names() {
    assert_eq!(ZodiacMode::Tropical.to_string(), "Tropical");
    assert_eq!(
        ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri
        }
        .to_string(),
        "Sidereal (Lahiri)"
    );
    assert_eq!(
        ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Custom(CustomAyanamsa::new("My Custom Sidereal"))
        }
        .to_string(),
        "Sidereal (My Custom Sidereal)"
    );
}

#[test]
fn custom_ayanamsas_have_stable_display_names() {
    let custom = CustomAyanamsa::new("My Custom Sidereal");

    assert_eq!(custom.to_string(), "My Custom Sidereal");
    assert_eq!(Ayanamsa::Custom(custom).to_string(), "My Custom Sidereal");
}

#[test]
fn custom_ayanamsa_enum_validate_reuses_the_structured_validator() {
    assert!(Ayanamsa::Lahiri.validate().is_ok());

    let custom = CustomAyanamsa {
        name: "My Custom Sidereal".to_string(),
        description: Some("local calibration".to_string()),
        epoch: Some(JulianDay::from_days(2_451_545.0)),
        offset_degrees: None,
    };

    assert_eq!(
        Ayanamsa::Custom(custom)
            .validate()
            .expect_err("custom ayanamsas should validate their descriptor")
            .to_string(),
        "custom ayanamsa requires both epoch and offset_degrees when one is present"
    );
}

#[test]
fn custom_ayanamsa_validate_rejects_padded_or_incomplete_definitions() {
    let mut custom = CustomAyanamsa::new("My Custom Sidereal");
    custom.description = Some("  ".to_string());

    assert_eq!(
        custom
            .validate()
            .expect_err("blank descriptions should be rejected")
            .to_string(),
        "custom ayanamsa description must not be blank"
    );

    let custom = CustomAyanamsa {
        name: "My Custom Sidereal".to_string(),
        description: Some("local calibration".to_string()),
        epoch: Some(JulianDay::from_days(2_451_545.0)),
        offset_degrees: None,
    };

    assert_eq!(
        custom
            .validate()
            .expect_err("partial offset definitions should be rejected")
            .to_string(),
        "custom ayanamsa requires both epoch and offset_degrees when one is present"
    );

    let custom = CustomAyanamsa {
        name: "My Custom Sidereal".to_string(),
        description: Some("local calibration".to_string()),
        epoch: Some(JulianDay::from_days(f64::INFINITY)),
        offset_degrees: Some(Angle::from_degrees(24.0)),
    };

    assert_eq!(
        custom
            .validate()
            .expect_err("non-finite epochs should be rejected")
            .to_string(),
        "custom ayanamsa epoch must be finite"
    );
}

#[test]
fn custom_ayanamsa_validate_against_reserved_labels_rejects_builtin_collisions() {
    let custom = CustomAyanamsa::new("Lahiri");

    assert_eq!(
        custom
            .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Lahiri"))
            .expect_err("built-in labels should be rejected")
            .to_string(),
        "custom ayanamsa name must not match a built-in label: Lahiri"
    );
}

#[test]
fn time_scale_conversion_errors_use_stable_display_labels() {
    let error = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
        .tt_from_ut1(Duration::from_secs(1))
        .expect_err("TT is not UT1");

    assert_eq!(
        error.to_string(),
        "time-scale conversion expected UT1, got TT"
    );
}

#[test]
fn zodiac_signs_follow_longitude_bands() {
    assert_eq!(
        ZodiacSign::from_longitude(Longitude::from_degrees(0.0)),
        ZodiacSign::Aries
    );
    assert_eq!(
        ZodiacSign::from_longitude(Longitude::from_degrees(29.999)),
        ZodiacSign::Aries
    );
    assert_eq!(
        ZodiacSign::from_longitude(Longitude::from_degrees(30.0)),
        ZodiacSign::Taurus
    );
}

#[test]
fn time_range_checks_scale_and_julian_day() {
    let start = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
    let end = Instant::new(JulianDay::from_days(2451546.0), TimeScale::Tt);
    let range = TimeRange::new(Some(start), Some(end));

    assert!(range.contains(Instant::new(JulianDay::from_days(2451545.5), TimeScale::Tt)));
    assert!(!range.contains(Instant::new(
        JulianDay::from_days(2451545.5),
        TimeScale::Utc
    )));
    assert_eq!(
        range.summary_line(),
        "JD 2451545.0 (TT) → JD 2451546.0 (TT)"
    );
    assert_eq!(range.to_string(), range.summary_line());
    assert!(range.validate().is_ok());
    assert_eq!(TimeRange::new(Some(start), None).validate(), Ok(()));
    assert_eq!(
        TimeRange::new(Some(start), None).to_string(),
        "from JD 2451545.0 (TT)"
    );
    assert_eq!(
        TimeRange::new(None, Some(end)).to_string(),
        "through JD 2451546.0 (TT)"
    );
    assert_eq!(TimeRange::new(None, None).to_string(), "unbounded");
}

#[test]
fn time_range_validation_rejects_non_finite_bounds_and_invalid_order() {
    let finite_start = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
    let finite_end = Instant::new(JulianDay::from_days(2451546.0), TimeScale::Tt);

    let error = TimeRange::new(
        Some(Instant::new(
            JulianDay::from_days(f64::INFINITY),
            TimeScale::Tt,
        )),
        Some(finite_end),
    )
    .validate()
    .expect_err("non-finite start bounds should fail validation");
    assert_eq!(
        error.summary_line(),
        "time range bound `start` must be finite: JD inf (TT)"
    );
    assert_eq!(error.to_string(), error.summary_line());

    let error = TimeRange::new(
        Some(finite_start),
        Some(Instant::new(
            JulianDay::from_days(f64::NEG_INFINITY),
            TimeScale::Tt,
        )),
    )
    .validate()
    .expect_err("non-finite end bounds should fail validation");
    assert_eq!(
        error.summary_line(),
        "time range bound `end` must be finite: JD -inf (TT)"
    );
    assert_eq!(error.to_string(), error.summary_line());

    let error = TimeRange::new(
        Some(Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt)),
        Some(Instant::new(
            JulianDay::from_days(2451546.0),
            TimeScale::Tdb,
        )),
    )
    .validate()
    .expect_err("mixed time-scale bounds should fail validation");
    assert_eq!(
            error.summary_line(),
            "time range bounds must use the same time scale: start=JD 2451545.0 (TT); end=JD 2451546.0 (TDB)"
        );
    assert_eq!(error.to_string(), error.summary_line());

    let error = TimeRange::new(Some(finite_end), Some(finite_start))
        .validate()
        .expect_err("out-of-order ranges should fail validation");
    assert_eq!(
        error.summary_line(),
        "time range end must not precede the start: start=JD 2451546.0 (TT); end=JD 2451545.0 (TT)"
    );
    assert_eq!(error.to_string(), error.summary_line());
}

#[test]
fn caller_supplied_time_scale_offsets_shift_julian_days() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tt = ut1
        .tt_from_ut1(Duration::from_secs_f64(64.184))
        .expect("UT1 to TT conversion should accept UT1 input");

    assert_eq!(tt.scale, TimeScale::Tt);
    assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tt() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tt = utc
        .tt_from_utc(Duration::from_secs_f64(64.184))
        .expect("UTC to TT conversion should accept UTC input");

    assert_eq!(tt.scale, TimeScale::Tt);
    assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tt_with_signed_offset() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tt = utc
        .tt_from_utc_signed(64.184)
        .expect("UTC to TT conversion should accept signed UTC input");

    assert_eq!(tt.scale, TimeScale::Tt);
    assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tt_to_tdb() {
    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let tdb = tt
        .tdb_from_tt(Duration::from_secs_f64(0.001_657))
        .expect("TT to TDB conversion should accept TT input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + 0.001_657 / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tt_to_tdb_with_signed_offset() {
    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let tdb = tt
        .tdb_from_tt_signed(-0.001_657)
        .expect("TT to TDB conversion should accept signed TT input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tdb_to_tt() {
    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let tt = tdb
        .tt_from_tdb(-0.001_657)
        .expect("TDB to TT conversion should accept TDB input");

    assert_eq!(tt.scale, TimeScale::Tt);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((tt.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tdb_to_tt_with_signed_offset() {
    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let tt = tdb
        .tt_from_tdb_signed(-0.001_657)
        .expect("TDB to TT conversion should accept signed TDB input");

    assert_eq!(tt.scale, TimeScale::Tt);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((tt.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tdb() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tdb = utc
        .tdb_from_utc(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UTC to TDB conversion should accept UTC input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tdb_with_signed_offset() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tdb = utc
        .tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC to TDB conversion should accept signed UTC input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tdb() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tdb = ut1
        .tdb_from_ut1(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UT1 to TDB conversion should accept UT1 input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tdb_with_signed_offset() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tdb = ut1
        .tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UT1 to TDB conversion should accept signed UT1 input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tt_with_signed_offset() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tt = ut1
        .tt_from_ut1_signed(64.184)
        .expect("UT1 to TT conversion should accept signed UT1 input");

    assert_eq!(tt.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((tt.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn time_scale_helpers_reject_the_wrong_source_scale() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let ut1_error = utc
        .tt_from_ut1(Duration::from_secs(64))
        .expect_err("UTC is not UT1");

    assert!(matches!(
        ut1_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Utc,
        }
    ));

    let tdb_ut1_error = utc
        .tdb_from_ut1(Duration::from_secs(64), Duration::from_secs(1))
        .expect_err("UTC is not UT1 for UT1-to-TDB conversion");

    assert!(matches!(
        tdb_ut1_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Utc,
        }
    ));

    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let utc_error = tt
        .tt_from_utc(Duration::from_secs(64))
        .expect_err("TT is not UTC");

    assert!(matches!(
        utc_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Utc,
            actual: TimeScale::Tt,
        }
    ));

    let utc_signed_error = tt
        .tt_from_utc_signed(64.0)
        .expect_err("TT is not UTC for signed UTC-to-TT conversion");

    assert!(matches!(
        utc_signed_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Utc,
            actual: TimeScale::Tt,
        }
    ));

    let tdb_error = tt
        .tdb_from_utc(Duration::from_secs(64), Duration::from_secs(1))
        .expect_err("TT is not UTC for UTC-to-TDB conversion");

    assert!(matches!(
        tdb_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Utc,
            actual: TimeScale::Tt,
        }
    ));

    let tt_error = utc
        .tt_from_tdb(-0.001_657)
        .expect_err("UTC is not TDB for TDB-to-TT conversion");

    assert!(matches!(
        tt_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Tdb,
            actual: TimeScale::Utc,
        }
    ));

    let ut1_signed_error = utc
        .tt_from_ut1_signed(64.0)
        .expect_err("UTC is not UT1 for signed UT1-to-TT conversion");

    assert!(matches!(
        ut1_signed_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Utc,
        }
    ));

    let wrong_scale_error = tt
        .tt_from_tdb(-0.001_657)
        .expect_err("TT is not TDB for TDB-to-TT conversion");

    assert!(matches!(
        wrong_scale_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Tdb,
            actual: TimeScale::Tt,
        }
    ));
}

#[test]
fn signed_time_scale_helpers_reject_non_finite_offsets() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tt_nan_error = ut1
        .tt_from_ut1_signed(f64::NAN)
        .expect_err("non-finite UT1 offsets should be rejected");

    assert!(matches!(
        tt_nan_error,
        TimeScaleConversionError::NonFiniteOffset
    ));

    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let tdb_inf_error = tt
        .tdb_from_tt_signed(f64::INFINITY)
        .expect_err("non-finite TDB offsets should be rejected");

    assert!(matches!(
        tdb_inf_error,
        TimeScaleConversionError::NonFiniteOffset
    ));

    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let tt_negative_inf_error = tdb
        .tt_from_tdb(f64::NEG_INFINITY)
        .expect_err("non-finite TDB-to-TT offsets should be rejected");

    assert!(matches!(
        tt_negative_inf_error,
        TimeScaleConversionError::NonFiniteOffset
    ));
}

#[test]
fn time_scale_conversion_errors_render_stable_summary_lines() {
    let expected = TimeScaleConversionError::Expected {
        expected: TimeScale::Tt,
        actual: TimeScale::Utc,
    };
    assert_eq!(
        expected.summary_line(),
        "time-scale conversion expected TT, got UTC"
    );
    assert_eq!(expected.to_string(), expected.summary_line());

    let non_finite = TimeScaleConversionError::NonFiniteOffset;
    assert_eq!(
        non_finite.summary_line(),
        "time-scale conversion offset must be finite"
    );
    assert_eq!(non_finite.to_string(), non_finite.summary_line());
}

#[test]
fn time_scale_conversion_policy_can_validate_and_apply_a_caller_supplied_rule() {
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);

    assert!(policy.validate(ut1).is_ok());
    assert!(ut1.validate_time_scale_conversion(policy).is_ok());

    let converted = policy
        .apply(ut1)
        .expect("caller-supplied policy should convert the source instant");

    assert_eq!(converted.scale, TimeScale::Tt);
    assert!((converted.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
    assert_eq!(
        policy.summary_line(),
        "source=UT1; target=TT; offset_seconds=64.184 s"
    );
    assert_eq!(policy.to_string(), policy.summary_line());
}

#[test]
fn time_scale_conversion_policy_renders_signed_offsets_in_summary_lines() {
    let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);

    assert_eq!(
        policy.summary_line(),
        "source=TDB; target=TT; offset_seconds=-0.001657 s"
    );
    assert_eq!(policy.to_string(), policy.summary_line());
}

#[test]
fn time_scale_conversion_policy_validated_summary_line_matches_the_plain_rendering() {
    let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);

    assert_eq!(
        policy
            .validated_summary_line(instant)
            .expect("policy should validate"),
        policy.summary_line()
    );
}

#[test]
fn time_scale_conversion_policy_accepts_signed_tdb_to_tt_validation() {
    let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);
    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);

    assert!(policy.validate(tdb).is_ok());
    assert!(tdb.validate_time_scale_conversion(policy).is_ok());

    let converted = policy
        .apply(tdb)
        .expect("signed TDB-to-TT policy should convert the source instant");

    assert_eq!(converted.scale, TimeScale::Tt);
    assert!(
        (converted.julian_day.days() - 2_451_544.999_999_981).abs() < 1e-12,
        "signed TDB-to-TT conversion should apply the caller-supplied offset"
    );
}

#[test]
fn instant_validate_time_scale_conversion_rejects_mismatched_scales_and_non_finite_offsets() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);

    let error = instant
        .validate_time_scale_conversion(policy)
        .expect_err("policy should reject the wrong source scale");

    assert!(matches!(
        error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Tt,
        }
    ));

    let non_finite = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, f64::NAN);
    let error = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
        .validate_time_scale_conversion(non_finite)
        .expect_err("policy should reject non-finite offsets");

    assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));
}

#[test]
fn time_scale_conversion_policy_rejects_mismatched_scales_and_non_finite_offsets() {
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let error = policy
        .validate(tt)
        .expect_err("policy should reject the wrong source scale");

    assert!(matches!(
        error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Tt,
        }
    ));

    let non_finite = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, f64::NAN);
    let error = non_finite
        .validate(Instant::new(
            JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .expect_err("policy should reject non-finite offsets");

    assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));
    assert!(matches!(
        non_finite.apply(Instant::new(
            JulianDay::from_days(2_451_545.0),
            TimeScale::Tt
        )),
        Err(TimeScaleConversionError::NonFiniteOffset)
    ));
}

#[test]
fn instant_with_time_scale_offset_checked_rejects_non_finite_offsets() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let error = instant
        .with_time_scale_offset_checked(TimeScale::Tdb, f64::NEG_INFINITY)
        .expect_err("checked offset conversion should reject non-finite offsets");

    assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));

    let converted = instant
        .with_time_scale_offset_checked(TimeScale::Tdb, 0.001_657)
        .expect("checked offset conversion should accept finite offsets");

    assert_eq!(converted.scale, TimeScale::Tdb);
    assert!((converted.julian_day.days() - 2_451_545.000_000_019).abs() < 1e-12);
}

#[test]
fn motion_accessors_return_the_original_speed_components() {
    let motion = Motion::new(Some(0.12), Some(-0.03), Some(0.000_4));

    assert_eq!(motion.longitude_speed(), Some(0.12));
    assert_eq!(motion.latitude_speed(), Some(-0.03));
    assert_eq!(motion.distance_speed(), Some(0.000_4));
}

#[test]
fn motion_summary_line_matches_display() {
    let motion = Motion::new(Some(0.12), Some(-0.03), Some(0.000_4));

    assert_eq!(
        motion.summary_line(),
        "longitude=0.12 deg/day; latitude=-0.03 deg/day; distance=0.0004 au/day"
    );
    assert_eq!(motion.to_string(), motion.summary_line());
}

#[test]
fn motion_direction_tracks_the_sign_of_longitude_speed() {
    assert_eq!(
        Motion::new(Some(0.12), None, None).longitude_direction(),
        Some(MotionDirection::Direct)
    );
    assert_eq!(
        Motion::new(Some(-0.04), None, None).longitude_direction(),
        Some(MotionDirection::Retrograde)
    );
    assert_eq!(
        Motion::new(Some(0.0), None, None).longitude_direction(),
        Some(MotionDirection::Stationary)
    );
    assert_eq!(Motion::new(None, None, None).longitude_direction(), None);
    assert_eq!(
        Motion::new(Some(f64::NAN), None, None).longitude_direction(),
        None
    );
}

#[test]
fn motion_validation_rejects_non_finite_components() {
    let longitude = Motion::new(Some(f64::INFINITY), None, None);
    assert_eq!(
        longitude.validate(),
        Err(MotionValidationError::NonFiniteSpeed {
            field: "longitude_deg_per_day",
            value: f64::INFINITY,
        })
    );

    let latitude = Motion::new(None, Some(f64::NAN), None);
    assert!(matches!(
        latitude.validate(),
        Err(MotionValidationError::NonFiniteSpeed {
            field: "latitude_deg_per_day",
            value,
        }) if value.is_nan()
    ));

    let distance = Motion::new(None, None, Some(f64::NEG_INFINITY));
    assert_eq!(
        distance.validate(),
        Err(MotionValidationError::NonFiniteSpeed {
            field: "distance_au_per_day",
            value: f64::NEG_INFINITY,
        })
    );
}

#[test]
fn coordinate_validation_rejects_non_finite_and_out_of_range_values() {
    let bad_ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(12.0),
        Latitude::from_degrees(91.0),
        Some(-1.0),
    );
    assert_eq!(
        bad_ecliptic.validate(),
        Err(CoordinateValidationError::LatitudeOutOfRange {
            coordinate: "ecliptic",
            field: "latitude",
            value: 91.0,
        })
    );
    assert_eq!(
        CoordinateValidationError::LatitudeOutOfRange {
            coordinate: "ecliptic",
            field: "latitude",
            value: 91.0,
        }
        .summary_line(),
        "ecliptic coordinate field `latitude` must stay within [-90, 90], got 91"
    );

    let bad_distance = EclipticCoordinates::new(
        Longitude::from_degrees(12.0),
        Latitude::from_degrees(12.0),
        Some(-1.0),
    );
    assert_eq!(
        bad_distance.validate(),
        Err(CoordinateValidationError::NegativeDistance {
            coordinate: "ecliptic",
            value: -1.0,
        })
    );

    let bad_non_finite_distance = EclipticCoordinates::new(
        Longitude::from_degrees(12.0),
        Latitude::from_degrees(12.0),
        Some(f64::NAN),
    );
    assert!(matches!(
        bad_non_finite_distance.validate(),
        Err(CoordinateValidationError::NonFiniteValue {
            coordinate: "ecliptic",
            field: "distance_au",
            value,
        }) if value.is_nan()
    ));

    let bad_equatorial = EquatorialCoordinates::new(
        Angle::from_degrees(360.0),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );
    assert_eq!(
        bad_equatorial.validate(),
        Err(CoordinateValidationError::RightAscensionOutOfRange {
            coordinate: "equatorial",
            field: "right_ascension",
            value: 360.0,
        })
    );
    assert_eq!(
        CoordinateValidationError::NegativeDistance {
            coordinate: "ecliptic",
            value: -0.5,
        }
        .summary_line(),
        "ecliptic coordinate field `distance_au` must be non-negative, got -0.5"
    );
}

#[test]
fn observer_location_validation_rejects_invalid_values() {
    let valid = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(-0.1),
        Some(45.0),
    );
    assert_eq!(valid.validate(), Ok(()));
    assert_eq!(
        valid.summary_line(),
        "latitude=51.5°, longitude=359.9°, elevation=45.000 m"
    );
    assert_eq!(valid.validated_summary_line(), Ok(valid.summary_line()));

    let bad_latitude = ObserverLocation::new(
        Latitude::from_degrees(91.0),
        Longitude::from_degrees(-0.1),
        None,
    );
    assert_eq!(
        bad_latitude.validate(),
        Err(ObserverLocationValidationError::LatitudeOutOfRange { value: 91.0 })
    );
    assert_eq!(
        ObserverLocationValidationError::LatitudeOutOfRange { value: 91.0 }.summary_line(),
        "observer latitude must stay within [-90, 90], got 91"
    );

    let bad_longitude = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(f64::NAN),
        None,
    );
    assert!(matches!(
        bad_longitude.validate(),
        Err(ObserverLocationValidationError::NonFiniteLongitude { value }) if value.is_nan()
    ));

    let bad_elevation = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(-0.1),
        Some(f64::INFINITY),
    );
    assert_eq!(
        bad_elevation.validate(),
        Err(ObserverLocationValidationError::NonFiniteElevation {
            value: f64::INFINITY,
        })
    );
    assert_eq!(
        ObserverLocationValidationError::NonFiniteElevation {
            value: f64::INFINITY,
        }
        .summary_line(),
        "observer elevation must be finite, got inf"
    );
    assert!(bad_elevation.validated_summary_line().is_err());
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
