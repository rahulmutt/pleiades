//! Formula-family classification for the built-in house systems.

use crate::catalog::*;
use pleiades_types::{CustomHouseSystem, HouseSystem};

/// Builds a descriptor for a user-defined custom system. The built-in catalog
/// contains no `Custom` entry, so this is the only way to reach the
/// `HouseSystem::Custom(_)` arm of `formula_family` and the Custom half of the
/// collector's skip guard.
fn custom_descriptor() -> HouseSystemDescriptor {
    HouseSystemDescriptor::new(
        HouseSystem::Custom(CustomHouseSystem::new("Probe Houses")),
        "Probe Houses",
        &[],
        "A user-defined system used only by tests.",
        false,
        None,
    )
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

/// Kills `delete match arm HouseSystem::Custom(_)` in `formula_family`
/// (219:13). With the arm deleted, a custom system falls through to
/// `_ => HouseFormulaFamily::Unknown`, so only an assertion distinguishing
/// `Custom` from `Unknown` catches it. No test constructed a custom descriptor
/// before.
#[test]
fn custom_systems_report_the_custom_formula_family_not_unknown() {
    let custom = custom_descriptor();

    assert_eq!(custom.formula_family(), HouseFormulaFamily::Custom);
    assert_ne!(
        custom.formula_family(),
        HouseFormulaFamily::Unknown,
        "a custom system must not be indistinguishable from a future built-in",
    );
    assert_eq!(HouseFormulaFamily::Custom.to_string(), "Custom");
}

/// Kills `replace || with &&` in `collect_house_formula_families` (645:49).
///
/// The guard is `family == Custom || family == Unknown -> continue`. Under
/// `&&` a Custom entry satisfies only the left operand, so the mutant stops
/// skipping it and leaks `Custom` into the public family list. The built-in
/// catalog contains no Custom or Unknown entry, so this is unreachable through
/// `house_formula_families()` — it must be driven through the private
/// slice-taking collector.
#[test]
fn the_family_collector_skips_custom_entries() {
    let entries = [custom_descriptor()];
    assert_eq!(
        collect_house_formula_families(&entries),
        Vec::<HouseFormulaFamily>::new(),
        "a Custom entry must not appear in the collected family list",
    );

    // A mixed slice: the real entry survives, the Custom one is dropped.
    let equal = descriptor(&HouseSystem::Equal)
        .expect("Equal is a built-in")
        .clone();
    let mixed = [custom_descriptor(), equal];
    assert_eq!(
        collect_house_formula_families(&mixed),
        vec![HouseFormulaFamily::Equal],
    );
}
