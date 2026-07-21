use crate::*;

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
