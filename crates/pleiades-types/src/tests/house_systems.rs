use crate::*;

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
fn house_system_enum_validate_against_reserved_labels_checks_wrapped_custom() {
    let wrapped = HouseSystem::Custom(CustomHouseSystem::new("Equal"));
    assert!(wrapped
        .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Equal"))
        .is_err());
    assert_eq!(
        HouseSystem::Placidus.validate_against_reserved_labels(|_| true),
        Ok(())
    );
}
