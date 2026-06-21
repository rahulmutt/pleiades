//! Release body claims, profile identifiers, API stability, and request-surface summaries.

use std::fmt;
use std::sync::OnceLock;

use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BodyDateChannelClaimsSummary {
    pub(crate) release_body_claims: String,
    pub(crate) frame_policy: String,
    pub(crate) production_generation_date_range: String,
    pub(crate) production_generation_coverage: String,
    pub(crate) corpus_shape: String,
    pub(crate) coverage_posture: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BodyDateChannelClaimsSummaryValidationError {
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for BodyDateChannelClaimsSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the body/date/channel claims summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for BodyDateChannelClaimsSummaryValidationError {}

impl BodyDateChannelClaimsSummary {
    pub(crate) fn summary_line(&self) -> String {
        format!(
            "bodies={}; frame policy={}; date range={}; production generation coverage={}; corpus shape={}; coverage posture={}",
            self.release_body_claims,
            self.frame_policy,
            self.production_generation_date_range,
            self.production_generation_coverage,
            self.corpus_shape,
            self.coverage_posture
        )
    }

    pub(crate) fn validate(&self) -> Result<(), BodyDateChannelClaimsSummaryValidationError> {
        let expected = body_date_channel_claims_summary_details().ok_or(
            BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                field: "body_date_channel_claims_summary",
            },
        )?;

        if self.release_body_claims != expected.release_body_claims {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "release_body_claims",
                },
            );
        }
        if self.frame_policy != expected.frame_policy {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "frame_policy",
                },
            );
        }
        if self.production_generation_date_range != expected.production_generation_date_range {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "production_generation_date_range",
                },
            );
        }
        if self.production_generation_coverage != expected.production_generation_coverage {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "production_generation_coverage",
                },
            );
        }
        if self.corpus_shape != expected.corpus_shape {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "corpus_shape",
                },
            );
        }
        if self.coverage_posture != expected.coverage_posture {
            return Err(
                BodyDateChannelClaimsSummaryValidationError::FieldOutOfSync {
                    field: "coverage_posture",
                },
            );
        }
        Ok(())
    }

    pub(crate) fn validated_summary_line(
        &self,
    ) -> Result<String, BodyDateChannelClaimsSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

#[allow(dead_code)]
pub(crate) fn strip_report_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

pub(crate) fn production_generation_coverage_posture_for_report() -> Option<String> {
    let production_generation_coverage = required_summary_payload(
        pleiades_jpl::production_generation_snapshot_summary_for_report(),
        "Production generation coverage: ",
        "production generation coverage",
    )
    .ok()?;
    let production_generation_body_class_coverage = required_summary_payload(
        pleiades_jpl::production_generation_snapshot_body_class_coverage_summary_for_report(),
        "Production generation body-class coverage: ",
        "production generation body-class coverage",
    )
    .ok()?;
    validated_production_generation_corpus_shape_summary_for_report().ok()?;

    Some(format!(
        "production-generation coverage and corpus shape remain aligned across the advertised 1900-2100 CE window; coverage={}; body-class coverage={}",
        production_generation_coverage,
        production_generation_body_class_coverage,
    ))
}

pub(crate) fn production_generation_date_range_for_report() -> Option<String> {
    let production_generation_window =
        pleiades_jpl::production_generation_snapshot_window_summary()?;

    Some(format!(
        "{}..{}",
        format_instant(production_generation_window.earliest_epoch),
        format_instant(production_generation_window.latest_epoch)
    ))
}

pub(crate) fn body_date_channel_claims_summary_details() -> Option<BodyDateChannelClaimsSummary> {
    let coverage_posture = production_generation_coverage_posture_for_report()?;
    Some(BodyDateChannelClaimsSummary {
        release_body_claims: validated_release_body_claims_summary_line_for_report().ok()?,
        frame_policy: validated_frame_policy_summary_for_report(),
        production_generation_date_range: production_generation_date_range_for_report()?,
        production_generation_coverage: production_generation_snapshot_summary_for_report(),
        corpus_shape: validated_production_generation_corpus_shape_summary_for_report().ok()?,
        coverage_posture,
    })
}

pub(crate) fn format_release_body_claims_summary_for_report() -> String {
    let posture = derived_release_posture();
    if let Err(error) = validate_release_posture(&posture) {
        return format!("release-grade body claims unavailable ({error})");
    }
    posture.summary_line()
}

pub(crate) fn format_body_date_channel_claims_summary_for_report() -> String {
    let summary = match body_date_channel_claims_summary_details() {
        Some(summary) => summary,
        None => {
            return "body/date/channel claims unavailable (source corpus unavailable)".to_string()
        }
    };
    match summary.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("body/date/channel claims unavailable ({error})"),
    }
}

pub(crate) fn render_release_body_claims_summary_text() -> String {
    format!(
        "Release-grade body claims summary\nRelease-grade body claims: {}\n",
        format_release_body_claims_summary_for_report()
    )
}

pub(crate) fn render_body_date_channel_claims_summary_text() -> String {
    format!(
        "Body/date/channel claims summary\nBody/date/channel claims: {}\n",
        format_body_date_channel_claims_summary_for_report()
    )
}

pub(crate) fn render_pluto_fallback_summary_text_from_report(
    report: Result<ComparisonReport, String>,
) -> String {
    let policy_line = match validated_pluto_fallback_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => {
            return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
        }
    };

    if let Err(error) = validate_release_posture(&derived_release_posture()) {
        return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
    }

    let report = match report {
        Ok(report) => report,
        Err(error) => {
            return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
        }
    };
    let summary = match comparison_tolerance_policy_summary_details(&report)
        .entries
        .into_iter()
        .find(|entry| entry.scope == ComparisonToleranceScope::Pluto)
    {
        Some(summary) => summary,
        None => {
            return "Pluto fallback summary\nPluto fallback unavailable (comparison report is missing a Pluto scope entry)\n".to_string();
        }
    };
    match summary.validated_summary_line() {
        Ok(line) => format!(
            "Pluto fallback summary\nRelease-grade body claims: {}\nPluto fallback policy: {policy_line}\nPluto fallback: {line}\n",
            format_release_body_claims_summary_for_report()
        ),
        Err(error) => format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n"),
    }
}

pub(crate) fn render_pluto_fallback_summary_text() -> String {
    render_pluto_fallback_summary_text_from_report(comparison_report_for_default_render())
}

pub(crate) fn validated_api_stability_profile_for_report(
) -> Result<pleiades_core::ApiStabilityProfile, String> {
    let profile = current_api_stability_profile();
    profile.validate().map_err(|error| error.to_string())?;
    Ok(profile)
}

pub(crate) fn validated_compatibility_profile_for_report() -> Result<CompatibilityProfile, String> {
    let profile = current_compatibility_profile();
    profile.validate().map_err(|error| error.to_string())?;
    Ok(profile)
}

pub(crate) fn validated_release_profile_identifiers_for_report(
) -> Result<ReleaseProfileIdentifiers, String> {
    let release_profiles = current_release_profile_identifiers();
    release_profiles
        .validate()
        .map_err(|error| error.to_string())?;
    Ok(release_profiles)
}

pub(crate) fn validated_catalog_inventory_summary_for_report() -> Result<String, String> {
    validated_compatibility_profile_for_report()?;
    validated_release_profile_identifiers_for_report()?;
    core_validated_catalog_inventory_summary_for_report().map_err(|error| error.to_string())
}

#[cfg(test)]
pub(crate) fn validated_house_code_aliases_summary_for_profile(
    profile: &CompatibilityProfile,
) -> Result<String, String> {
    profile
        .validated_house_code_aliases_summary_line()
        .map_err(|error| error.to_string())
}

pub(crate) fn validated_house_code_aliases_summary_for_report() -> Result<String, String> {
    core_validated_house_code_aliases_summary_for_report().map_err(|error| error.to_string())
}

pub(crate) fn validated_release_profile_identifiers_summary_for_report(
    release_profiles: &ReleaseProfileIdentifiers,
) -> String {
    match core_validated_release_profile_identifiers_summary_for_report(release_profiles) {
        Ok(summary) => summary,
        Err(error) => format!("unavailable ({error})"),
    }
}

/// Renders the compact release-profile identifiers summary.
pub fn render_release_profile_identifiers_summary() -> String {
    render_release_profile_identifiers_summary_text()
}

pub(crate) fn render_release_profile_identifiers_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let release_profiles = match validated_release_profile_identifiers_for_report() {
                Ok(release_profiles) => release_profiles,
                Err(error) => {
                    return format!("Release profile identifiers summary unavailable ({error})");
                }
            };

            let mut text = String::new();
            text.push_str("Release profile identifiers summary\n");
            text.push_str("Summary line: ");
            text.push_str(&validated_release_profile_identifiers_summary_for_report(
                &release_profiles,
            ));
            text.push('\n');
            text.push_str("Compatibility profile: ");
            text.push_str(release_profiles.compatibility_profile_id);
            text.push('\n');
            text.push_str("API stability posture: ");
            text.push_str(release_profiles.api_stability_profile_id);
            text.push('\n');
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("API stability summary: api-stability-summary\n");
            text.push_str("Release summary: release-summary\n");

            text
        })
        .clone()
}

pub(crate) fn api_stability_summary_line_for_report() -> String {
    match validated_api_stability_profile_for_report() {
        Ok(profile) => profile.summary_line(),
        Err(error) => format!("API stability summary unavailable ({error})"),
    }
}

/// Compact inventory of the public request surfaces that are called out in the
/// time-observer policy and release-facing validation summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestSurfaceSummary {
    pub(crate) instant: &'static str,
    pub(crate) chart_request: &'static str,
    pub(crate) backend_request: &'static str,
    pub(crate) house_request: &'static str,
    pub(crate) request_policy: &'static str,
    pub(crate) cli_chart: &'static str,
}

impl RequestSurfaceSummary {
    /// Returns the current compact request-surface inventory.
    pub const fn current() -> Self {
        Self {
            instant: "pleiades-types::Instant (tagged instant plus caller-supplied retagging)",
            chart_request: "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)",
            backend_request:
                "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)",
            house_request: "pleiades-houses::HouseRequest (house-only observer calculations)",
            request_policy:
                "request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints)",
            cli_chart: "pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)",
        }
    }

    /// Validates that the cached inventory still matches the documented request surfaces.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        const EXPECTED_INSTANT: &str =
            "pleiades-types::Instant (tagged instant plus caller-supplied retagging)";
        const EXPECTED_CHART_REQUEST: &str =
            "pleiades-core::ChartRequest (chart assembly plus house-observer preflight)";
        const EXPECTED_BACKEND_REQUEST: &str =
            "pleiades-backend::EphemerisRequest (direct backend dispatch plus metadata preflight)";
        const EXPECTED_HOUSE_REQUEST: &str =
            "pleiades-houses::HouseRequest (house-only observer calculations)";
        const EXPECTED_REQUEST_POLICY: &str =
            "request-policy-summary / request-policy / request-semantics-summary / request-semantics / unsupported-modes-summary / unsupported-modes / utc-convenience-policy-summary / utc-convenience-policy / delta-t-policy-summary / delta-t-policy / zodiac-policy-summary / zodiac-policy / native-sidereal-policy-summary / native-sidereal-policy (compact request-policy report entrypoints)";
        const EXPECTED_CLI_CHART: &str = "pleiades-cli chart (explicit --tt|--tdb|--utc|--ut1 flags plus caller-supplied TT/TDB offset aliases: --tt-offset-seconds, --tt-from-utc-offset-seconds, --tt-from-ut1-offset-seconds, --tdb-offset-seconds, --tdb-from-utc-offset-seconds, --tdb-from-ut1-offset-seconds, --tdb-from-tt-offset-seconds, and --tt-from-tdb-offset-seconds; observer-bearing chart requests stay geocentric and use the observer only for houses)";

        validate_request_surface_label("instant", self.instant, EXPECTED_INSTANT)?;
        validate_request_surface_label(
            "chart request",
            self.chart_request,
            EXPECTED_CHART_REQUEST,
        )?;
        validate_request_surface_label(
            "backend request",
            self.backend_request,
            EXPECTED_BACKEND_REQUEST,
        )?;
        validate_request_surface_label(
            "house request",
            self.house_request,
            EXPECTED_HOUSE_REQUEST,
        )?;
        validate_request_surface_label(
            "request policy",
            self.request_policy,
            EXPECTED_REQUEST_POLICY,
        )?;
        validate_request_surface_label("CLI chart", self.cli_chart, EXPECTED_CLI_CHART)?;

        Ok(())
    }

    /// Returns the chart-help clause that spells out the explicit UTC/UT1 and
    /// TT/TDB aliases used by the chart CLI.
    pub fn validated_chart_help_clause(self) -> Result<&'static str, EphemerisError> {
        self.validate()?;
        Ok(self.cli_chart)
    }

    /// Returns the chart-help clause that spells out the explicit UTC/UT1 and
    /// TT/TDB aliases used by the chart CLI.
    pub const fn chart_help_clause(self) -> &'static str {
        self.cli_chart
    }

    /// Returns the compact `Primary request surfaces:` line.
    pub fn summary_line(self) -> String {
        format!(
            "Primary request surfaces: {}; {}; {}; {}; {}; {}",
            self.instant,
            self.chart_request,
            self.backend_request,
            self.house_request,
            self.request_policy,
            self.cli_chart,
        )
    }

    /// Validates the summary and returns its compact report line.
    pub fn validated_summary_line(self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for RequestSurfaceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn validate_request_surface_label(
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), EphemerisError> {
    if actual == expected {
        return Ok(());
    }

    Err(EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("primary request surface {field} mismatch: expected {expected}, found {actual}"),
    ))
}

/// Returns the current compact request-surface inventory.
pub const fn current_request_surface_summary() -> RequestSurfaceSummary {
    RequestSurfaceSummary::current()
}

pub(crate) fn request_surface_summary_for_report() -> String {
    let summary = RequestSurfaceSummary::current();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("primary request surfaces unavailable ({error})"),
    }
}
