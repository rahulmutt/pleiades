//! Houses report/summary prose relocated from `pleiades-houses`
//! (report-surface relocation program, Slice B), including the former
//! `HouseCatalogValidationSummary::summary_line()` inherent method body.
//! Rendering only — the functional crate keeps the structured data and
//! their constructors.

// Verbatim relocation of a report-prose surface: some renderers are exercised
// only by this module's own tests or have no current in-crate caller.
#![allow(dead_code)]

use pleiades_houses::{
    built_in_house_systems, house_formula_families, house_system_code_aliases,
    latitude_sensitive_house_failure_modes, validate_house_system_code_aliases,
    HouseCatalogValidationSummary, HouseFormulaFamily, HouseSystemCodeAlias,
    HouseSystemCodeAliasValidationError,
};

/// Returns a compact one-line rendering of the Swiss-Ephemeris house-code alias table.
///
/// # Examples
///
/// ```text
/// let summary = house_system_code_aliases_summary_line();
/// assert!(summary.contains("P -> Placidus"));
/// ```
pub(crate) fn house_system_code_aliases_summary_line() -> String {
    house_system_code_aliases()
        .iter()
        .map(HouseSystemCodeAlias::summary_line)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns the alias table summary after validating the built-in alias inventory.
pub(crate) fn validated_house_system_code_aliases_summary_line(
) -> Result<String, HouseSystemCodeAliasValidationError> {
    validate_house_system_code_aliases()?;
    Ok(house_system_code_aliases_summary_line())
}

/// Returns a compact one-line rendering of the distinct built-in house formula families.
pub(crate) fn house_formula_families_summary_line() -> String {
    let families = house_formula_families();

    match families.as_slice() {
        [] => "none".to_string(),
        [single] => single.to_string(),
        _ => families
            .iter()
            .map(HouseFormulaFamily::to_string)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn format_string_summary(items: &[String]) -> String {
    match items {
        [] => "none".to_string(),
        [single] => single.clone(),
        _ => items.join(", "),
    }
}

/// Returns a compact one-line rendering of the latitude-sensitive failure-mode notes.
pub(crate) fn latitude_sensitive_house_failure_modes_summary_line() -> String {
    format_string_summary(&latitude_sensitive_house_failure_modes())
}

/// Returns the compact release-facing summary line for the house catalog validation state.
pub(crate) fn house_catalog_validation_summary_line(
    summary: &HouseCatalogValidationSummary,
) -> String {
    let formula_families = house_formula_families_summary_line();
    let latitude_sensitive_labels = built_in_house_systems()
        .iter()
        .filter(|entry| entry.latitude_sensitive)
        .map(|entry| entry.canonical_name)
        .collect::<Vec<_>>();
    let latitude_sensitive_count = latitude_sensitive_labels.len();
    let latitude_sensitive_labels = if latitude_sensitive_labels.is_empty() {
        "none".to_string()
    } else {
        latitude_sensitive_labels.join(", ")
    };

    let failure_modes = latitude_sensitive_house_failure_modes_summary_line();

    match &summary.validation_result {
        Ok(()) => format!(
            "house catalog validation: ok ({} entries, {} labels checked; baseline={}, release={}; formula families: {}; latitude-sensitive={}/{} entries; failure modes: {}; labels: {}; round-trip, alias uniqueness, and notes verified)",
            summary.entry_count,
            summary.label_count,
            summary.baseline_entry_count,
            summary.release_entry_count,
            formula_families,
            latitude_sensitive_count,
            summary.entry_count,
            failure_modes,
            latitude_sensitive_labels,
        ),
        Err(error) => format!(
            "house catalog validation: error: {} ({} entries; baseline={}, release={})",
            error,
            summary.entry_count,
            summary.baseline_entry_count,
            summary.release_entry_count,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pleiades_houses::{
        baseline_house_systems, house_catalog_validation_summary, house_formula_families,
        release_house_systems, HouseFormulaFamily,
    };

    #[test]
    fn house_catalog_validation_summary_reports_catalog_health() {
        let summary = house_catalog_validation_summary();
        let expected_formula_families = house_formula_families_summary_line();
        let expected_latitude_sensitive_labels = built_in_house_systems()
            .iter()
            .filter(|entry| entry.latitude_sensitive)
            .map(|entry| entry.canonical_name)
            .collect::<Vec<_>>()
            .join(", ");

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
        assert!(house_catalog_validation_summary_line(&summary)
            .contains("house catalog validation: ok"));
        assert!(house_catalog_validation_summary_line(&summary).contains("formula families:"));
        assert!(
            house_catalog_validation_summary_line(&summary).contains(&expected_formula_families)
        );
        assert!(house_catalog_validation_summary_line(&summary).contains("latitude-sensitive="));
        assert!(house_catalog_validation_summary_line(&summary).contains("failure modes:"));
        assert!(house_catalog_validation_summary_line(&summary)
            .contains(&expected_latitude_sensitive_labels));
        assert!(house_catalog_validation_summary_line(&summary)
            .contains("round-trip, alias uniqueness, and notes verified"));
    }

    #[test]
    fn swiss_ephemeris_house_system_code_aliases_summary_lines_match() {
        assert_eq!(
            house_system_code_aliases_summary_line(),
            "P -> Placidus, K -> Koch, R -> Regiomontanus, C -> Campanus, O -> Porphyry, D -> Equal (MC), E -> Equal, W -> Whole Sign, V -> Vehlow Equal, A -> Axial, H -> Horizon/Azimuth, B -> Alcabitius, M -> Morinus, S -> Sripati, I -> Sunshine, G -> Gauquelin sectors, T -> Topocentric, U -> Krusinski-Pisa-Goelzer, Axial Rotation -> Meridian, Axial rotation system -> Meridian, X -> Meridian, Y -> APC"
        );
        assert_eq!(
            validated_house_system_code_aliases_summary_line().unwrap(),
            house_system_code_aliases_summary_line()
        );
    }
}
