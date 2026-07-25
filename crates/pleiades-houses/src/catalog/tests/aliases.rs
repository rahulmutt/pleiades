//! Alias resolution: canonical labels, software-specific aliases, and
//! Swiss-Ephemeris short-code aliases.

use crate::catalog::*;

#[test]
fn aliases_resolve_to_builtin_systems() {
    assert_eq!(
        resolve_house_system("Polich-Page"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("Polich/Page"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("Topocentric house system"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("Topocentric table of houses"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("Polich-Page \"topocentric\" table of houses"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("Equal table of houses"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("Equal (from MC) table of houses"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal (MC) table of houses"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal/MC table of houses"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal (MC) house system"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Whole Sign table of houses"),
        Some(pleiades_types::HouseSystem::WholeSign)
    );
    assert_eq!(
        resolve_house_system("Whole Sign (house 1 = Aries) table of houses"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal (1=Aries) table of houses"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal/1=Aries table of houses"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal (1=Aries) house system"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Vehlow-equal table of houses"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("Vehlow Equal table of houses"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("Vehlow equal"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("Carter's poli-equatorial table of houses"),
        Some(pleiades_types::HouseSystem::Carter)
    );
    assert_eq!(
        resolve_house_system("Carter's poli-equatorial"),
        Some(pleiades_types::HouseSystem::Carter)
    );
    assert_eq!(
        resolve_house_system("APC, also known as \u{201C}Ram school\u{201D}, table of houses"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("Krusinski-Pisa-Goelzer table of houses"),
        Some(pleiades_types::HouseSystem::KrusinskiPisaGoelzer)
    );
    assert_eq!(
        resolve_house_system("Sunshine table of houses"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("Sunshine table of houses, by Bob Makransky"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("I sunshine"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("Gauquelin table of sectors"),
        Some(pleiades_types::HouseSystem::Gauquelin)
    );
    assert_eq!(
        resolve_house_system("whole sign houses"),
        Some(pleiades_types::HouseSystem::WholeSign)
    );
    assert_eq!(
        resolve_house_system("Whole Sign system"),
        Some(pleiades_types::HouseSystem::WholeSign)
    );
    assert_eq!(
        resolve_house_system("Whole Sign house system"),
        Some(pleiades_types::HouseSystem::WholeSign)
    );
    assert_eq!(
        resolve_house_system("Placidus table of houses"),
        Some(pleiades_types::HouseSystem::Placidus)
    );
    assert_eq!(
        resolve_house_system("Koch table of houses"),
        Some(pleiades_types::HouseSystem::Koch)
    );
    assert_eq!(
        resolve_house_system("w. koch"),
        Some(pleiades_types::HouseSystem::Koch)
    );
    assert_eq!(
        resolve_house_system("Koch houses"),
        Some(pleiades_types::HouseSystem::Koch)
    );
    assert_eq!(
        resolve_house_system("house system of the birth place"),
        Some(pleiades_types::HouseSystem::Koch)
    );
    assert_eq!(
        resolve_house_system("W Koch"),
        Some(pleiades_types::HouseSystem::Koch)
    );
    assert_eq!(
        resolve_house_system("ARMC"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Axial Rotation"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Axial rotation system"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Zariel"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Meridian house system"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("D"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("A equal"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("D equal / MC"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("E equal = A"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("W equal, whole sign"),
        Some(pleiades_types::HouseSystem::WholeSign)
    );
    assert_eq!(
        resolve_house_system("V equal Vehlow"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("X axial rotation system/ Meridian houses"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Y APC houses"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("T Polich/Page (\"topocentric\")"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("P"),
        Some(pleiades_types::HouseSystem::Placidus)
    );
    assert_eq!(
        resolve_house_system("K"),
        Some(pleiades_types::HouseSystem::Koch)
    );
    assert_eq!(
        resolve_house_system("R"),
        Some(pleiades_types::HouseSystem::Regiomontanus)
    );
    assert_eq!(
        resolve_house_system("C"),
        Some(pleiades_types::HouseSystem::Campanus)
    );
    assert_eq!(
        resolve_house_system("O"),
        Some(pleiades_types::HouseSystem::Porphyry)
    );
    assert_eq!(
        resolve_house_system("E"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("W"),
        Some(pleiades_types::HouseSystem::WholeSign)
    );
    assert_eq!(
        resolve_house_system("N"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("V"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("A"),
        Some(pleiades_types::HouseSystem::Axial)
    );
    assert_eq!(
        resolve_house_system("H"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("B"),
        Some(pleiades_types::HouseSystem::Alcabitius)
    );
    assert_eq!(
        resolve_house_system("M"),
        Some(pleiades_types::HouseSystem::Morinus)
    );
    assert_eq!(
        resolve_house_system("S"),
        Some(pleiades_types::HouseSystem::Sripati)
    );
    assert_eq!(
        resolve_house_system("I"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("G"),
        Some(pleiades_types::HouseSystem::Gauquelin)
    );
    assert_eq!(
        resolve_house_system("T"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("U"),
        Some(pleiades_types::HouseSystem::KrusinskiPisaGoelzer)
    );
    assert_eq!(
        resolve_house_system("X"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Y"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("Carter"),
        Some(pleiades_types::HouseSystem::Carter)
    );
    assert_eq!(
        resolve_house_system("Carter's poli-equatorial"),
        Some(pleiades_types::HouseSystem::Carter)
    );
    assert_eq!(
        resolve_house_system("T topocentric"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("U krusinski-pisa-goelzer"),
        Some(pleiades_types::HouseSystem::KrusinskiPisaGoelzer)
    );
    assert_eq!(
        resolve_house_system("Equal (from MC)"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal MC"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal/MC"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal/MC house system"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal Midheaven"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal Midheaven house system"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal Midheaven table of houses"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal (MC)"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal/MC = 10th"),
        Some(pleiades_types::HouseSystem::EqualMidheaven)
    );
    assert_eq!(
        resolve_house_system("Equal Aries"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal/1=Aries"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal/1=Aries house system"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal/1=0 Aries"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal (cusp 1 = 0° Aries)"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("vehlow"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("Vehlow house system"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("Vehlow Equal house system"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("Vehlow-equal"),
        Some(pleiades_types::HouseSystem::Vehlow)
    );
    assert_eq!(
        resolve_house_system("Wang"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("Equal house system"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("Equal House"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("Whole Sign (house 1 = Aries)"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("N whole sign houses, 1. house = Aries"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Whole sign houses, 1. house = Aries"),
        Some(pleiades_types::HouseSystem::EqualAries)
    );
    assert_eq!(
        resolve_house_system("Equal (cusp 1 = Asc)"),
        Some(pleiades_types::HouseSystem::Equal)
    );
    assert_eq!(
        resolve_house_system("Azimuth"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Horizontal"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Azimuthal"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Horizontal house system"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Horizontal table of houses"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Azimuth house system"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Azimuthal table of houses"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("horizon/azimuth"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("horizon/azimut"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Ram school"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("Ram's school"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("APC house system"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("WvA"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("Ascendant Parallel Circle"),
        Some(pleiades_types::HouseSystem::Apc)
    );
    assert_eq!(
        resolve_house_system("Krusinski"),
        Some(pleiades_types::HouseSystem::KrusinskiPisaGoelzer)
    );
    assert_eq!(
        resolve_house_system("Krusinski/Pisa/Goelzer"),
        Some(pleiades_types::HouseSystem::KrusinskiPisaGoelzer)
    );
    assert_eq!(
        resolve_house_system("Krusinski/Pisa/Goelzer house system"),
        Some(pleiades_types::HouseSystem::KrusinskiPisaGoelzer)
    );
    assert_eq!(
        resolve_house_system("Horizon house system"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Horizon/Azimuth house system"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Horizontal house system"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Azimuth house system"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Horizon/Azimuth table of houses"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Azimuthal house system"),
        Some(pleiades_types::HouseSystem::Horizon)
    );
    assert_eq!(
        resolve_house_system("Sunshine house system"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("Śrīpati"),
        Some(pleiades_types::HouseSystem::Sripati)
    );
    assert_eq!(
        resolve_house_system("S sripati"),
        Some(pleiades_types::HouseSystem::Sripati)
    );
    assert_eq!(
        resolve_house_system("Sripati house system"),
        Some(pleiades_types::HouseSystem::Sripati)
    );
    assert_eq!(
        resolve_house_system("Sripati table of houses"),
        Some(pleiades_types::HouseSystem::Sripati)
    );
    assert_eq!(
        resolve_house_system("Sunshine"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("Bob Makransky"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("Treindl Sunshine"),
        Some(pleiades_types::HouseSystem::Sunshine)
    );
    assert_eq!(
        resolve_house_system("G"),
        Some(pleiades_types::HouseSystem::Gauquelin)
    );
    assert_eq!(
        resolve_house_system("Gauquelin sectors"),
        Some(pleiades_types::HouseSystem::Gauquelin)
    );
    assert_eq!(
        resolve_house_system("Savard-A"),
        Some(pleiades_types::HouseSystem::Albategnius)
    );
    assert_eq!(
        resolve_house_system("Neo-Porphyry"),
        Some(pleiades_types::HouseSystem::PullenSd)
    );
    assert_eq!(
        resolve_house_system("Pullen (Sinusoidal Delta)"),
        Some(pleiades_types::HouseSystem::PullenSd)
    );
    assert_eq!(
        resolve_house_system("Pullen SD (Sinusoidal Delta)"),
        Some(pleiades_types::HouseSystem::PullenSd)
    );
    assert_eq!(
        resolve_house_system("Pullen SD table of houses"),
        Some(pleiades_types::HouseSystem::PullenSd)
    );
    assert_eq!(
        resolve_house_system("Pullen SD (Neo-Porphyry) table of houses"),
        Some(pleiades_types::HouseSystem::PullenSd)
    );
    assert_eq!(
        resolve_house_system("Pullen SD (Neo-Porphyry)"),
        Some(pleiades_types::HouseSystem::PullenSd)
    );
    assert_eq!(
        resolve_house_system("Pullen (Sinusoidal Ratio)"),
        Some(pleiades_types::HouseSystem::PullenSr)
    );
    assert_eq!(
        resolve_house_system("Pullen sinusoidal ratio"),
        Some(pleiades_types::HouseSystem::PullenSr)
    );
    assert_eq!(
        resolve_house_system("Pullen SR table of houses"),
        Some(pleiades_types::HouseSystem::PullenSr)
    );
    assert_eq!(
        resolve_house_system("Pullen SR (Sinusoidal Ratio) table of houses"),
        Some(pleiades_types::HouseSystem::PullenSr)
    );
    assert_eq!(
        resolve_house_system("Pullen SR (Sinusoidal Ratio)"),
        Some(pleiades_types::HouseSystem::PullenSr)
    );
}

#[test]
fn additional_release_house_aliases_resolve_to_builtin_systems() {
    assert_eq!(
        resolve_house_system("Polich Page"),
        Some(pleiades_types::HouseSystem::Topocentric)
    );
    assert_eq!(
        resolve_house_system("Poli-Equatorial"),
        Some(pleiades_types::HouseSystem::Carter)
    );
    assert_eq!(
        resolve_house_system("Equal Quadrant"),
        Some(pleiades_types::HouseSystem::Porphyry)
    );
    assert_eq!(
        resolve_house_system("Meridian table of houses"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Whole-sign"),
        Some(pleiades_types::HouseSystem::WholeSign)
    );
}

#[test]
fn swiss_ephemeris_house_system_code_aliases_are_unique_and_round_trip() {
    let aliases = house_system_code_aliases();
    let mut seen = std::collections::BTreeSet::new();

    for alias in aliases {
        assert!(seen.insert(alias.label.to_ascii_lowercase()));
        assert_eq!(
            resolve_house_system(alias.label),
            Some(alias.system.clone())
        );
    }

    assert_eq!(validate_house_system_code_aliases(), Ok(()));
    assert_eq!(aliases.len(), 22);
    assert_eq!(aliases[0].summary_line(), "P -> Placidus");
    assert_eq!(aliases[0].to_string(), "P -> Placidus");
    assert_eq!(
        resolve_house_system("axial rotation"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("axial rotation system"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("X"),
        Some(pleiades_types::HouseSystem::Meridian)
    );
    assert_eq!(
        resolve_house_system("Y"),
        Some(pleiades_types::HouseSystem::Apc)
    );
}

#[test]
fn house_system_code_alias_validation_rejects_duplicate_short_labels() {
    let aliases = [
        HouseSystemCodeAlias {
            label: "Axial Rotation",
            system: pleiades_types::HouseSystem::Meridian,
        },
        HouseSystemCodeAlias {
            label: "axial rotation",
            system: pleiades_types::HouseSystem::Meridian,
        },
    ];

    let error = validate_house_system_code_alias_entries(&aliases)
        .expect_err("duplicate labels should be rejected");
    assert!(matches!(
        error,
        HouseSystemCodeAliasValidationError::DuplicateLabel {
            label: "axial rotation"
        }
    ));
}

#[test]
fn house_system_code_alias_validate_rejects_normalization_and_round_trip_drift() {
    let valid_alias = HouseSystemCodeAlias {
        label: "P",
        system: pleiades_types::HouseSystem::Placidus,
    };
    assert_eq!(valid_alias.validate(), Ok(()));

    let mismatched_alias = HouseSystemCodeAlias {
        label: "P",
        system: pleiades_types::HouseSystem::Porphyry,
    };
    assert!(matches!(
        mismatched_alias.validate(),
        Err(HouseSystemCodeAliasValidationError::LabelDoesNotRoundTrip {
            label: "P",
            expected_system: pleiades_types::HouseSystem::Porphyry
        })
    ));

    let aliases = [HouseSystemCodeAlias {
        label: "P\n",
        system: pleiades_types::HouseSystem::Placidus,
    }];

    let error = validate_house_system_code_alias_entries(&aliases)
        .expect_err("line-break labels should be rejected");
    assert!(matches!(
        error,
        HouseSystemCodeAliasValidationError::LabelNotNormalized { label: "P\n" }
    ));
}

/// Pins every `HouseSystemCodeAliasValidationError` rendering exactly.
///
/// Kills `<impl Display for HouseSystemCodeAliasValidationError>::fmt ->
/// Ok(Default::default())` (428:9): that mutant writes nothing, producing an
/// empty error string. Nothing asserted these renderings before.
#[test]
fn alias_validation_errors_render_stable_diagnostics() {
    use pleiades_types::HouseSystem;

    assert_eq!(
        HouseSystemCodeAliasValidationError::EmptyAliasTable.to_string(),
        "the house-code alias table is empty",
        "pins EmptyAliasTable's Display text; update if the variant's rendering changes",
    );
    assert_eq!(
        HouseSystemCodeAliasValidationError::LabelNotNormalized { label: " P " }.to_string(),
        "the house-code alias label ` P ` is blank, contains surrounding whitespace, \
         or contains line breaks",
        "pins LabelNotNormalized's Display text; update if the variant's rendering changes",
    );
    assert_eq!(
        HouseSystemCodeAliasValidationError::DuplicateLabel { label: "P" }.to_string(),
        "the house-code alias table contains duplicate label `P`",
        "pins DuplicateLabel's Display text; update if the variant's rendering changes",
    );
    assert_eq!(
        HouseSystemCodeAliasValidationError::LabelDoesNotRoundTrip {
            label: "P",
            expected_system: HouseSystem::Koch,
        }
        .to_string(),
        format!(
            "the house-code alias label `P` does not round-trip to {}",
            HouseSystem::Koch
        ),
        "pins LabelDoesNotRoundTrip's Display text; update if the variant's rendering changes",
    );
}
