//! ELP lunar-theory catalog report prose.

use pleiades_elp::{
    LunarTheoryCapabilitySummary, LunarTheoryCatalogSummary, LunarTheoryCatalogValidationSummary,
    LunarTheoryLimitationsSummary, LunarTheorySourceFamilySummary, LunarTheorySourceSelection,
};

/// Formats the compact validation summary for release-facing reporting.
pub(crate) fn format_lunar_theory_catalog_validation_summary(
    summary: &LunarTheoryCatalogValidationSummary,
) -> String {
    summary.summary_line()
}

/// Returns a compact release-facing summary of the lunar-theory catalog validation state.
pub(crate) fn lunar_theory_catalog_validation_summary_for_report() -> String {
    pleiades_elp::lunar_theory_catalog_validation_summary().summary_line()
}

pub(crate) fn format_validated_lunar_theory_source_selection_for_report(
    selection: &LunarTheorySourceSelection,
) -> String {
    match selection.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar source selection: unavailable ({error})"),
    }
}

/// Returns the compact source-selection summary for the current lunar baseline.
///
/// The helper validates the backend-owned source-selection record first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_source_selection_summary() -> String {
    format_validated_lunar_theory_source_selection_for_report(
        &pleiades_elp::lunar_theory_source_selection(),
    )
}

/// Returns the compact source-selection summary for report-facing consumers.
///
/// This keeps the report-facing naming aligned with the other typed lunar
/// provenance helpers used by `pleiades-validate`.
///
/// Public: the CLI (`pleiades-cli/src/cli.rs`) is a runtime consumer of this
/// symbol (report-surface relocation program, Slice B).
pub fn lunar_theory_source_selection_summary_for_report() -> String {
    lunar_theory_source_selection_summary()
}

/// Returns the validated compact source-selection summary for report-facing consumers.
pub(crate) fn validated_lunar_theory_source_selection_summary_for_report() -> Result<String, String>
{
    let selection = pleiades_elp::lunar_theory_source_selection();
    selection
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Formats a structured lunar source selection for reporting.
///
/// The formatter validates the provided selection against the current lunar-theory
/// baseline first so callers get the same fail-closed summary wording as the
/// report-facing accessor when a drifted selection is reused.
pub(crate) fn format_lunar_theory_source_selection(
    selection: &LunarTheorySourceSelection,
) -> String {
    format_validated_lunar_theory_source_selection_for_report(selection)
}

fn format_validated_lunar_theory_source_family_summary_for_report(
    summary: &LunarTheorySourceFamilySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("lunar source family: unavailable ({error})"),
    }
}

/// Returns the compact source-family summary for the current lunar baseline.
///
/// The helper validates the backend-owned source-family record first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_source_family_summary_for_report() -> String {
    format_validated_lunar_theory_source_family_summary_for_report(
        &pleiades_elp::lunar_theory_source_family_summary(),
    )
}

/// Formats the catalog summary for release-facing reporting.
pub(crate) fn format_lunar_theory_catalog_summary(summary: &LunarTheoryCatalogSummary) -> String {
    summary.summary_line()
}

pub(crate) fn format_validated_lunar_theory_catalog_summary_for_report(
    summary: &LunarTheoryCatalogSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar theory catalog: unavailable ({error})"),
    }
}

/// Returns the release-facing catalog summary string for the current lunar-theory selection.
///
/// The report helper validates the backend-owned catalog summary first so any
/// future drift in the rendered selection fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_catalog_summary_for_report() -> String {
    format_validated_lunar_theory_catalog_summary_for_report(
        &pleiades_elp::lunar_theory_catalog_summary(),
    )
}

/// Formats the capability summary for release-facing reporting.
pub(crate) fn format_lunar_theory_capability_summary(
    summary: &LunarTheoryCapabilitySummary,
) -> String {
    summary.summary_line()
}

fn format_validated_lunar_theory_capability_summary_for_report(
    summary: &LunarTheoryCapabilitySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar capability summary: unavailable ({error})"),
    }
}

/// Returns the release-facing capability summary string for the current lunar-theory selection.
///
/// The report helper validates the backend-owned capability summary first so any
/// future drift in the rendered selection fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_capability_summary_for_report() -> String {
    format_validated_lunar_theory_capability_summary_for_report(
        &pleiades_elp::lunar_theory_capability_summary(),
    )
}

/// Formats the compact lunar-theory limitations summary for release-facing reporting.
pub(crate) fn format_lunar_theory_limitations_summary(
    summary: &LunarTheoryLimitationsSummary,
) -> String {
    summary.summary_line()
}

fn format_validated_lunar_theory_limitations_summary_for_report(
    summary: &LunarTheoryLimitationsSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar theory limitations: unavailable ({error})"),
    }
}

/// Returns the release-facing one-line summary for the current lunar-theory limitations posture.
pub(crate) fn lunar_theory_limitations_summary_for_report() -> String {
    format_validated_lunar_theory_limitations_summary_for_report(
        &pleiades_elp::lunar_theory_limitations_summary(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lunar_theory_catalog_validation_summary_for_report_matches_the_summary_line() {
        let summary = pleiades_elp::lunar_theory_catalog_validation_summary();
        assert_eq!(
            format_lunar_theory_catalog_validation_summary(&summary),
            summary.summary_line()
        );
        assert_eq!(
            lunar_theory_catalog_validation_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn lunar_theory_source_selection_summary_for_report_matches_the_selection() {
        let selection = pleiades_elp::lunar_theory_source_selection();
        assert_eq!(
            lunar_theory_source_selection_summary(),
            selection.summary_line()
        );
        assert_eq!(
            lunar_theory_source_selection_summary_for_report(),
            selection.summary_line()
        );
        assert_eq!(
            format_lunar_theory_source_selection(&selection),
            selection.summary_line()
        );
        assert_eq!(
            validated_lunar_theory_source_selection_summary_for_report().unwrap(),
            selection.summary_line()
        );
    }

    #[test]
    fn format_validated_lunar_theory_source_selection_for_report_fails_closed_for_drifted_fields() {
        let selection = pleiades_elp::lunar_theory_source_selection();
        let drifted = LunarTheorySourceSelection {
            identifier: "not-the-current-selection",
            ..selection
        };
        assert_eq!(
            format_validated_lunar_theory_source_selection_for_report(&drifted),
            "lunar source selection: unavailable (the lunar source selection field `identifier` is out of sync with the current selection)"
        );
        assert_eq!(
            format_lunar_theory_source_selection(&drifted),
            "lunar source selection: unavailable (the lunar source selection field `identifier` is out of sync with the current selection)"
        );
    }

    #[test]
    fn lunar_theory_source_family_summary_for_report_matches_the_family_summary() {
        let family_summary = pleiades_elp::lunar_theory_source_family_summary();
        assert_eq!(
            lunar_theory_source_family_summary_for_report(),
            family_summary.summary_line()
        );
    }

    #[test]
    fn format_validated_lunar_theory_source_family_summary_for_report_fails_closed_for_drifted_fields(
    ) {
        let family_summary = pleiades_elp::lunar_theory_source_family_summary();
        let drifted = LunarTheorySourceFamilySummary {
            selected_source_identifier: "not-the-current-selection",
            ..family_summary
        };
        assert_eq!(
            format_validated_lunar_theory_source_family_summary_for_report(&drifted),
            "lunar source family: unavailable (the lunar source-family summary field `selected_source_identifier` is out of sync with the current selection)"
        );
    }

    #[test]
    fn lunar_theory_catalog_summary_for_report_matches_the_summary_line() {
        let catalog_summary = pleiades_elp::lunar_theory_catalog_summary();
        assert_eq!(
            format_lunar_theory_catalog_summary(&catalog_summary),
            catalog_summary.summary_line()
        );
        assert_eq!(
            lunar_theory_catalog_summary_for_report(),
            catalog_summary.summary_line()
        );
    }

    #[test]
    fn format_validated_lunar_theory_catalog_summary_for_report_fails_closed_for_drifted_fields() {
        let catalog_summary = pleiades_elp::lunar_theory_catalog_summary();
        let drifted = LunarTheoryCatalogSummary {
            selected_alias_count: catalog_summary.selected_alias_count + 1,
            ..catalog_summary
        };
        assert_eq!(
            format_validated_lunar_theory_catalog_summary_for_report(&drifted),
            "lunar theory catalog: unavailable (the lunar catalog summary field `selected_alias_count` is out of sync with the current catalog)"
        );
    }

    #[test]
    fn lunar_theory_capability_summary_for_report_matches_the_summary_line() {
        let capability_summary = pleiades_elp::lunar_theory_capability_summary();
        assert_eq!(
            format_lunar_theory_capability_summary(&capability_summary),
            capability_summary.summary_line()
        );
        assert_eq!(
            lunar_theory_capability_summary_for_report(),
            capability_summary.summary_line()
        );
    }

    #[test]
    fn format_validated_lunar_theory_capability_summary_for_report_fails_closed_for_drifted_fields()
    {
        let capability_summary = pleiades_elp::lunar_theory_capability_summary();
        let drifted = LunarTheoryCapabilitySummary {
            model_name: "Drifted lunar baseline",
            ..capability_summary
        };
        assert_eq!(
            format_validated_lunar_theory_capability_summary_for_report(&drifted),
            "lunar capability summary: unavailable (the lunar capability summary field `model_name` is out of sync with the current selection)"
        );
    }

    #[test]
    fn lunar_theory_limitations_summary_for_report_matches_the_summary_line() {
        let limitations_summary = pleiades_elp::lunar_theory_limitations_summary();
        assert_eq!(
            format_lunar_theory_limitations_summary(&limitations_summary),
            limitations_summary.summary_line()
        );
        assert_eq!(
            lunar_theory_limitations_summary_for_report(),
            limitations_summary.summary_line()
        );
    }

    #[test]
    fn format_validated_lunar_theory_limitations_summary_for_report_fails_closed_for_drifted_fields(
    ) {
        let limitations_summary = pleiades_elp::lunar_theory_limitations_summary();
        let drifted = LunarTheoryLimitationsSummary {
            unsupported_bodies: &[pleiades_types::CelestialBody::TrueApogee],
            ..limitations_summary
        };
        assert_eq!(
            format_validated_lunar_theory_limitations_summary_for_report(&drifted),
            "lunar theory limitations: unavailable (the lunar theory limitations summary field `unsupported_bodies` is out of sync with the current baseline)"
        );
    }
}
