use crate::*;

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
fn custom_ayanamsa_accepts_finite_offset_pair_and_rejects_non_finite() {
    // Both epoch and a FINITE offset present -> Ok. The deleted `!` in
    // `if !offset.is_finite()` would reject this finite pair.
    let finite = CustomAyanamsa {
        name: "Local Calibration".to_string(),
        description: None,
        epoch: Some(JulianDay::from_days(2_451_545.0)),
        offset_degrees: Some(Angle::from_degrees(24.0)),
    };
    assert_eq!(finite.validate(), Ok(()));

    // Both present but offset non-finite -> Err. The deleted `!` would accept it.
    let non_finite = CustomAyanamsa {
        name: "Local Calibration".to_string(),
        description: None,
        epoch: Some(JulianDay::from_days(2_451_545.0)),
        offset_degrees: Some(Angle::from_degrees(f64::INFINITY)),
    };
    assert!(non_finite.validate().is_err());
}

#[test]
fn ayanamsa_enum_validate_against_reserved_labels_checks_wrapped_custom() {
    // The enum method (and its Self::Custom arm) must forward to the wrapped
    // custom's reserved-label check. Existing tests call the struct method
    // directly and never traverse this arm.
    let wrapped = Ayanamsa::Custom(CustomAyanamsa::new("Lahiri"));
    assert!(wrapped
        .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Lahiri"))
        .is_err());
    // Built-in variants are always Ok (documents the `_ => Ok(())` arm).
    assert_eq!(
        Ayanamsa::Lahiri.validate_against_reserved_labels(|_| true),
        Ok(())
    );
}
