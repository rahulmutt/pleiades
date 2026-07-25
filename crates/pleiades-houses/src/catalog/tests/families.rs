//! Formula-family classification for the built-in house systems.

use crate::catalog::*;

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
