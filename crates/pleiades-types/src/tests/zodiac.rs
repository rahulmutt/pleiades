use crate::*;

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
