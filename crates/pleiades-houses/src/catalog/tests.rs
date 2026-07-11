use super::*;

#[test]
fn baseline_catalog_includes_required_milestone_entries() {
    let names: Vec<_> = baseline_house_systems()
        .iter()
        .map(|entry| entry.canonical_name)
        .collect();

    for expected in [
        "Placidus",
        "Koch",
        "Porphyry",
        "Regiomontanus",
        "Campanus",
        "Equal",
        "Whole Sign",
        "Alcabitius",
        "Meridian",
        "Axial",
        "Topocentric",
        "Morinus",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
}

#[test]
fn descriptor_summary_line_includes_aliases_formula_family_latitude_and_notes() {
    let descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Alias One", "Alias Two"],
        "Summary note",
        true,
        None,
    );

    let expected =
        "Equal (aliases: Alias One, Alias Two) [formula: Equal] [latitude-sensitive] — Summary note";
    assert_eq!(descriptor.summary_line(), expected);
    assert_eq!(
        descriptor.validated_summary_line(),
        Ok(expected.to_string())
    );
    assert_eq!(descriptor.to_string(), expected);
}

#[test]
fn validated_summary_line_rejects_descriptor_drift() {
    let descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Alias One"],
        " Summary note",
        true,
        None,
    );

    assert_eq!(
        descriptor.validated_summary_line(),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    );

    let alias = HouseSystemCodeAlias {
        label: " T",
        system: pleiades_types::HouseSystem::Topocentric,
    };

    assert_eq!(
        alias.validated_summary_line(),
        Err(HouseSystemCodeAliasValidationError::LabelNotNormalized { label: " T" })
    );
}

#[test]
fn formula_family_groups_the_built_in_house_systems_by_shape() {
    let equal =
        descriptor(&pleiades_types::HouseSystem::Equal).expect("equal should be catalogued");
    let whole_sign = descriptor(&pleiades_types::HouseSystem::WholeSign)
        .expect("whole sign should be catalogued");
    let quadrant =
        descriptor(&pleiades_types::HouseSystem::Placidus).expect("placidus should be catalogued");
    let equatorial =
        descriptor(&pleiades_types::HouseSystem::Meridian).expect("meridian should be catalogued");
    let great_circle =
        descriptor(&pleiades_types::HouseSystem::Horizon).expect("horizon should be catalogued");
    let solar_arc =
        descriptor(&pleiades_types::HouseSystem::Sunshine).expect("sunshine should be catalogued");
    let sector = descriptor(&pleiades_types::HouseSystem::Gauquelin)
        .expect("gauquelin should be catalogued");

    assert_eq!(equal.formula_family(), HouseFormulaFamily::Equal);
    assert_eq!(whole_sign.formula_family(), HouseFormulaFamily::WholeSign);
    assert_eq!(quadrant.formula_family(), HouseFormulaFamily::Quadrant);
    assert_eq!(
        equatorial.formula_family(),
        HouseFormulaFamily::EquatorialProjection
    );
    assert_eq!(
        great_circle.formula_family(),
        HouseFormulaFamily::GreatCircle
    );
    assert_eq!(solar_arc.formula_family(), HouseFormulaFamily::SolarArc);
    assert_eq!(sector.formula_family(), HouseFormulaFamily::Sector);
}

#[test]
fn built_in_house_systems_have_known_formula_families() {
    for entry in built_in_house_systems() {
        assert_ne!(
            entry.formula_family(),
            HouseFormulaFamily::Unknown,
            "{} should map to a known formula family",
            entry.canonical_name
        );
    }
}

#[test]
fn validation_errors_use_stable_house_system_display_names() {
    let error = HouseCatalogValidationError::LabelDoesNotRoundTrip {
        label: "Equal (MC) table of houses",
        expected_system: pleiades_types::HouseSystem::EqualMidheaven,
    };

    assert_eq!(
        error.to_string(),
        "the house catalog label `Equal (MC) table of houses` does not round-trip to Equal (MC)"
    );
}

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
fn release_additions_are_merged_into_the_built_in_catalog() {
    let names: Vec<_> = built_in_house_systems()
        .iter()
        .map(|entry| entry.canonical_name)
        .collect();

    for expected in [
        "Equal (MC)",
        "Equal (1=Aries)",
        "Vehlow Equal",
        "Sripati",
        "Carter (poli-equatorial)",
        "Horizon/Azimuth",
        "APC",
        "Krusinski-Pisa-Goelzer",
        "Albategnius",
        "Pullen SD",
        "Pullen SR",
        "Sunshine",
        "Gauquelin sectors",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
}

#[test]
fn release_descriptor_aliases_do_not_repeat_canonical_labels() {
    assert!(built_in_house_systems()
        .iter()
        .all(|entry| { !entry.aliases.contains(&entry.canonical_name) }));
}

#[test]
fn house_catalog_round_trips_all_built_ins_and_aliases() {
    use std::collections::HashSet;

    let built_in = built_in_house_systems();
    let mut unique_names = HashSet::new();

    assert_eq!(
        built_in.len(),
        baseline_house_systems().len() + release_house_systems().len()
    );

    for entry in baseline_house_systems()
        .iter()
        .chain(release_house_systems().iter())
    {
        assert!(
            unique_names.insert(entry.canonical_name),
            "duplicate canonical house-system name {}",
            entry.canonical_name
        );
        assert_eq!(
            descriptor(&entry.system).map(|d| d.canonical_name),
            Some(entry.canonical_name)
        );
        assert_eq!(
            resolve_house_system(entry.canonical_name),
            Some(entry.system.clone())
        );
        for alias in entry.aliases {
            assert_eq!(resolve_house_system(alias), Some(entry.system.clone()));
        }
    }

    for entry in built_in {
        assert!(unique_names.contains(entry.canonical_name));
    }
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
fn house_catalog_validation_summary_aggregates_catalog_fields() {
    let summary = house_catalog_validation_summary();

    assert_eq!(summary.entry_count, built_in_house_systems().len());
    assert_eq!(summary.baseline_entry_count, baseline_house_systems().len());
    assert_eq!(summary.release_entry_count, release_house_systems().len());
    assert_eq!(
        house_formula_families(),
        vec![
            HouseFormulaFamily::Equal,
            HouseFormulaFamily::WholeSign,
            HouseFormulaFamily::Quadrant,
            HouseFormulaFamily::EquatorialProjection,
            HouseFormulaFamily::GreatCircle,
            HouseFormulaFamily::SolarArc,
            HouseFormulaFamily::Sector,
        ]
    );
    assert!(summary.validation_result.is_ok());
}

#[test]
fn house_catalog_validation_rejects_duplicate_labels_and_round_trip_mismatches() {
    let duplicate_alias_entries = [HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Wang", "wang"],
        "notes",
        false,
        None,
    )];

    assert!(matches!(
        validate_house_catalog_entries(&duplicate_alias_entries),
        Err(HouseCatalogValidationError::DescriptorLabelCollision {
            label: "wang",
            canonical_name: "Equal"
        })
    ));

    let mismatched_entry = [HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Not Equal",
        &[],
        "notes",
        false,
        None,
    )];

    assert!(matches!(
        validate_house_catalog_entries(&mismatched_entry),
        Err(HouseCatalogValidationError::LabelDoesNotRoundTrip {
            label: "Not Equal",
            expected_system: pleiades_types::HouseSystem::Equal,
        })
    ));

    let blank_name_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "   ",
        &[],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        blank_name_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: "   ",
            field: "canonical name"
        })
    ));

    let padded_alias_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &[" Alias "],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        padded_alias_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: " Alias ",
            field: "alias"
        })
    ));

    let blank_notes_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &[],
        "   ",
        false,
        None,
    );
    assert!(matches!(
        blank_notes_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    ));

    let line_break_name_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equ\nal",
        &[],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        line_break_name_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: "Equ\nal",
            field: "canonical name"
        })
    ));

    let line_break_alias_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Al\nial"],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        line_break_alias_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: "Al\nial",
            field: "alias"
        })
    ));

    let line_break_notes_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &[],
        "notes\nline two",
        false,
        None,
    );
    assert!(matches!(
        line_break_notes_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    ));

    let duplicate_alias_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Wang", "wang"],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        duplicate_alias_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelCollision {
            label: "wang",
            canonical_name: "Equal"
        })
    ));

    let blank_notes_entry = [blank_notes_descriptor];
    assert!(matches!(
        validate_house_catalog_entries(&blank_notes_entry),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    ));
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

#[test]
fn release_grade_numeric_house_set_is_exactly_the_twenty_four_corpus_systems() {
    use pleiades_types::{CompatibilityClaimTier, HouseSystem};

    let release_grade: Vec<HouseSystem> = crate::built_in_house_systems()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .map(|d| d.system.clone())
        .collect();

    let expected = [
        // Twelve baseline corpus systems.
        HouseSystem::Placidus,
        HouseSystem::Koch,
        HouseSystem::Porphyry,
        HouseSystem::Regiomontanus,
        HouseSystem::Campanus,
        HouseSystem::Equal,
        HouseSystem::WholeSign,
        HouseSystem::Alcabitius,
        HouseSystem::Meridian,
        HouseSystem::Axial,
        HouseSystem::Topocentric,
        HouseSystem::Morinus,
        // Ten standard systems promoted in Phase 6.
        HouseSystem::EqualMidheaven,
        HouseSystem::EqualAries,
        HouseSystem::Vehlow,
        HouseSystem::Sripati,
        HouseSystem::Carter,
        HouseSystem::Apc,
        HouseSystem::KrusinskiPisaGoelzer,
        HouseSystem::Sunshine,
        HouseSystem::PullenSd,
        HouseSystem::PullenSr,
        // Gauquelin promoted in Phase 6 Task 5a: its 36 sectors now match SE
        // via the Placidus semi-arc division (corpus-backed by the sectors slice).
        HouseSystem::Gauquelin,
        // Horizon promoted in Phase 6 Task 5b: the SE 'H' azimuth convention was
        // corrected (+180° post-rotation, single 90° quarter-turn, strict-sign
        // latitude branch); now matches SE within the GreatCircle ceiling.
        HouseSystem::Horizon,
    ];

    assert_eq!(release_grade.len(), expected.len());
    for sys in expected {
        assert!(release_grade.contains(&sys), "missing {sys:?}");
    }
}

#[test]
fn latitude_sensitive_systems_carry_a_latitude_bound() {
    for descriptor in built_in_house_systems() {
        if descriptor.latitude_sensitive {
            assert!(
                descriptor.max_abs_latitude_deg.is_some(),
                "latitude-sensitive system {:?} must declare max_abs_latitude_deg",
                descriptor.system
            );
            let bound = descriptor.max_abs_latitude_deg.unwrap();
            assert!(
                (60.0..=89.0).contains(&bound),
                "{:?} bound {bound} out of expected polar range",
                descriptor.system
            );
        } else {
            assert!(
                descriptor.max_abs_latitude_deg.is_none(),
                "non-latitude-sensitive system {:?} must not declare a bound",
                descriptor.system
            );
        }
    }
}
