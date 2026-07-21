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

#[test]
fn zodiac_sign_names_and_display_are_stable() {
    let cases = [
        (ZodiacSign::Aries, "Aries"),
        (ZodiacSign::Taurus, "Taurus"),
        (ZodiacSign::Gemini, "Gemini"),
        (ZodiacSign::Cancer, "Cancer"),
        (ZodiacSign::Leo, "Leo"),
        (ZodiacSign::Virgo, "Virgo"),
        (ZodiacSign::Libra, "Libra"),
        (ZodiacSign::Scorpio, "Scorpio"),
        (ZodiacSign::Sagittarius, "Sagittarius"),
        (ZodiacSign::Capricorn, "Capricorn"),
        (ZodiacSign::Aquarius, "Aquarius"),
        (ZodiacSign::Pisces, "Pisces"),
    ];
    for (sign, expected) in cases {
        assert_eq!(sign.name(), expected);
        assert_eq!(sign.to_string(), expected);
    }
}

#[test]
fn zodiac_signs_cover_every_thirty_degree_band() {
    let bands = [
        (15.0, ZodiacSign::Aries),
        (45.0, ZodiacSign::Taurus),
        (75.0, ZodiacSign::Gemini),
        (105.0, ZodiacSign::Cancer),
        (135.0, ZodiacSign::Leo),
        (165.0, ZodiacSign::Virgo),
        (195.0, ZodiacSign::Libra),
        (225.0, ZodiacSign::Scorpio),
        (255.0, ZodiacSign::Sagittarius),
        (285.0, ZodiacSign::Capricorn),
        (315.0, ZodiacSign::Aquarius),
        (345.0, ZodiacSign::Pisces),
    ];
    for (deg, expected) in bands {
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(deg)),
            expected,
            "longitude {deg} should map to {expected:?}"
        );
    }
    // Wraparound: normalization must reduce a full turn before band dispatch.
    assert_eq!(
        ZodiacSign::from_longitude(Longitude::from_degrees(360.0)),
        ZodiacSign::Aries
    );
    assert_eq!(
        ZodiacSign::from_longitude(Longitude::from_degrees(780.0)),
        ZodiacSign::Gemini
    );
}
