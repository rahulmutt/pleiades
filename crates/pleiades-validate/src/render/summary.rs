//! Policy, corpus, and evidence summary text rendering for the validation tool.

use std::collections::BTreeMap;

use super::text::*;
use crate::*;

pub(crate) fn render_reference_snapshot_summary_text() -> String {
    format!(
        "Reference snapshot summary\n{}\n",
        reference_snapshot_summary_for_report()
    )
}

pub(crate) fn render_reference_snapshot_exact_j2000_evidence_text() -> String {
    format!(
        "Reference snapshot exact J2000 evidence summary\n{}\n",
        reference_snapshot_exact_j2000_evidence_summary_for_report()
    )
}

pub(crate) fn render_lunar_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar reference error envelope summary\n{}\n",
        lunar_reference_evidence_envelope_for_report()
    )
}

pub(crate) fn render_lunar_reference_evidence_summary_text() -> String {
    format!(
        "Lunar reference evidence summary\n{}\n",
        lunar_reference_evidence_summary_for_report()
    )
}

pub(crate) fn render_lunar_equatorial_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar equatorial reference error envelope summary\n{}\n",
        lunar_equatorial_reference_evidence_envelope_for_report()
    )
}

pub(crate) fn render_lunar_apparent_comparison_summary_text() -> String {
    format!(
        "Lunar apparent comparison summary\n{}\n",
        lunar_apparent_comparison_summary_for_report()
    )
}

pub(crate) fn render_frame_policy_summary_text() -> String {
    match frame_policy_summary_details().validated_summary_line() {
        Ok(summary) => format!("Frame policy summary\nFrame policy: {}\n", summary),
        Err(error) => format!("Frame policy summary\nFrame policy unavailable ({error})\n"),
    }
}

pub(crate) fn render_reference_holdout_overlap_summary_text() -> String {
    match validated_reference_holdout_overlap_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference/hold-out overlap: unavailable ({error})"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestPolicyReportKind {
    Policy,
    Semantics,
}

impl RequestPolicyReportKind {
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Policy => "Request policy summary\n",
            Self::Semantics => "Request semantics summary\n",
        }
    }

    pub(crate) const fn unavailable_prefix(self) -> &'static str {
        match self {
            Self::Policy => "Request policy summary unavailable",
            Self::Semantics => "Request semantics summary unavailable",
        }
    }
}

pub(crate) fn validate_request_policy_report_title(
    kind: RequestPolicyReportKind,
    title: &str,
) -> Result<(), String> {
    let expected = kind.title();
    if title != expected {
        return Err(format!("{} ({title})", kind.unavailable_prefix()));
    }
    Ok(())
}

pub(crate) fn render_request_policy_like_summary_text(
    title: &'static str,
    kind: RequestPolicyReportKind,
) -> String {
    let time_scale_policy = time_scale_policy_summary_for_report();
    if let Err(error) = validate_request_policy_report_title(kind, title) {
        return error;
    }

    let mut text = String::from(title);
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text
}

pub(crate) fn render_request_policy_summary_text() -> String {
    render_request_policy_like_summary_text(
        "Request policy summary\n",
        RequestPolicyReportKind::Policy,
    )
}

pub(crate) fn render_request_semantics_summary_text() -> String {
    use std::fmt::Write as _;

    let mut text = render_request_policy_like_summary_text(
        "Request semantics summary\n",
        RequestPolicyReportKind::Semantics,
    );
    let _ = writeln!(
        text,
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    );
    text
}

pub(crate) fn render_unsupported_modes_summary_text() -> String {
    format!(
        "Unsupported modes summary\nUnsupported modes: {}\n",
        unsupported_modes_summary_for_report()
    )
}

pub(crate) fn render_request_surface_summary_text() -> String {
    format!(
        "Request surface summary\n{}\n",
        request_surface_summary_for_report()
    )
}

pub(crate) fn render_comparison_tolerance_policy_summary_text_from_report(
    report: Result<ComparisonReport, String>,
) -> String {
    match report {
        Ok(report) => format!(
            "Comparison tolerance policy summary\nComparison tolerance policy: {}\n",
            format_comparison_tolerance_policy_for_report(&report)
        ),
        Err(error) => format!(
            "Comparison tolerance policy summary\nComparison tolerance policy unavailable ({error})\n"
        ),
    }
}

pub(crate) fn render_comparison_tolerance_policy_summary_text() -> String {
    render_comparison_tolerance_policy_summary_text_from_report(
        comparison_report_for_default_render(),
    )
}
pub(crate) fn render_comparison_tolerance_scope_coverage_summary_text_from_summary(
    summary: Result<ComparisonTolerancePolicySummary, String>,
) -> String {
    use std::fmt::Write as _;

    let summary = match summary {
        Ok(summary) => match summary.validate() {
            Ok(()) => summary,
            Err(error) => {
                return format!("Comparison tolerance scope coverage summary\nComparison tolerance scope coverage unavailable ({error})\n");
            }
        },
        Err(error) => {
            return format!("Comparison tolerance scope coverage summary\nComparison tolerance scope coverage unavailable ({error})\n");
        }
    };

    let mut text = String::from("Comparison tolerance scope coverage summary\n");
    let _ = writeln!(
        text,
        "Scope coverage posture: {} rows",
        summary.coverage.len()
    );
    for coverage in &summary.coverage {
        let _ = writeln!(text, "  {}", coverage.summary_line());
    }
    text
}

pub(crate) fn render_comparison_tolerance_scope_coverage_summary_text() -> String {
    let summary = match comparison_report_for_default_render() {
        Ok(report) => validated_comparison_tolerance_policy_summary_for_report(&report),
        Err(error) => Err(error),
    };

    render_comparison_tolerance_scope_coverage_summary_text_from_summary(summary)
}

pub(crate) fn render_comparison_body_class_tolerance_summary_text_from_summaries(
    summaries: Result<Vec<BodyClassToleranceSummary>, String>,
) -> String {
    use std::fmt::Write as _;

    let summaries = match summaries {
        Ok(summaries) => summaries,
        Err(error) => {
            return format!("Comparison body-class tolerance summary\nComparison body-class tolerance unavailable ({error})\n");
        }
    };

    if summaries.is_empty() {
        return "Comparison body-class tolerance summary\nComparison body-class tolerance unavailable (comparison report did not produce any body-class tolerance summaries)\n".to_string();
    }

    for summary in &summaries {
        if let Err(error) = summary.validate() {
            return format!("Comparison body-class tolerance summary\nComparison body-class tolerance unavailable ({error})\n");
        }
    }

    let mut text = String::from("Comparison body-class tolerance summary\n");
    let _ = writeln!(text, "Body-class tolerance posture: {}", summaries.len());
    for summary in summaries {
        let _ = writeln!(
            text,
            "  {}",
            format_body_class_tolerance_envelope_for_report(&summary)
        );
    }
    text
}

pub(crate) fn render_comparison_body_class_tolerance_summary_text() -> String {
    let summaries = match comparison_report_for_default_render() {
        Ok(report) => Ok(report.body_class_tolerance_summaries()),
        Err(error) => Err(error),
    };

    render_comparison_body_class_tolerance_summary_text_from_summaries(summaries)
}

pub(crate) fn render_comparison_body_class_tolerance_posture_summary_text() -> String {
    match validated_comparison_body_class_tolerance_posture_for_report() {
        Ok(summary) => format!(
            "Comparison body-class tolerance posture summary\n{}\n",
            summary
        ),
        Err(error) => format!(
            "Comparison body-class tolerance posture summary\nComparison body-class tolerance unavailable ({error})\n"
        ),
    }
}

pub(crate) fn render_comparison_envelope_summary_text() -> String {
    let report = match comparison_report_for_default_render() {
        Ok(report) => report,
        Err(error) => {
            return format!(
                "Comparison envelope summary\nComparison envelope unavailable ({error})\n"
            );
        }
    };
    let envelope = comparison_envelope_summary(&report.summary, &report.samples);
    let summary_line = envelope
        .validated_summary_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison envelope unavailable ({error})"));
    let percentile_line = envelope
        .validated_percentile_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison percentile envelope unavailable ({error})"));

    format!(
        "Comparison envelope summary\nSummary line: {summary_line}\nPercentile line: {percentile_line}\n"
    )
}

pub(crate) fn ensure_comparison_envelope_summary_matches_current_rendering(
    comparison_envelope_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_envelope_summary_text == render_comparison_envelope_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison envelope summary no longer matches the current comparison envelope posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_comparison_body_class_tolerance_summary_matches_current_rendering(
    comparison_body_class_tolerance_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_body_class_tolerance_summary_text
        == render_comparison_body_class_tolerance_summary_text()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison body-class tolerance summary no longer matches the current comparison body-class tolerance posture"
                .to_string(),
        ))
    }
}

pub(crate) fn validate_release_body_claims_posture(
    release_body_claims_summary: &str,
    pluto_fallback_summary: &str,
) -> Result<(), String> {
    validate_release_body_claims_posture_backend(
        release_body_claims_summary,
        pluto_fallback_summary,
    )
    .map_err(|error| error.to_string())
}

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
        "production-generation coverage and corpus shape remain aligned across the advertised 1500-2500 CE window; coverage={}; body-class coverage={}",
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
        release_body_claims: validated_release_body_claims_summary_line_for_report()
            .ok()?
            .to_string(),
        frame_policy: validated_frame_policy_summary_for_report(),
        production_generation_date_range: production_generation_date_range_for_report()?,
        production_generation_coverage: production_generation_snapshot_summary_for_report(),
        corpus_shape: validated_production_generation_corpus_shape_summary_for_report().ok()?,
        coverage_posture,
    })
}

pub(crate) fn format_release_body_claims_summary_for_report() -> String {
    let summary_line = match validated_release_body_claims_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => return format!("release-grade body claims unavailable ({error})"),
    };
    let pluto_line = match validated_pluto_fallback_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => return format!("release-grade body claims unavailable ({error})"),
    };
    if let Err(error) = validate_release_body_claims_posture(summary_line, pluto_line) {
        return format!("release-grade body claims unavailable ({error})");
    }
    summary_line.to_string()
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

    let release_body_claims_line = match validated_release_body_claims_summary_line_for_report() {
        Ok(line) => line,
        Err(error) => {
            return format!("Pluto fallback summary\nPluto fallback unavailable ({error})\n");
        }
    };
    if let Err(error) = validate_release_body_claims_posture(release_body_claims_line, policy_line)
    {
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

pub(crate) fn format_vsop87_request_policy_summary() -> String {
    vsop87_request_policy_summary_for_report()
}

pub(crate) fn format_vsop87_source_audit_summary() -> String {
    source_audit_summary_for_report()
}

pub(crate) fn format_packaged_artifact_profile_summary() -> String {
    packaged_artifact_profile_summary_with_body_coverage()
}

pub(crate) fn validated_packaged_artifact_output_support_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_output_support_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact output support: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_speed_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_speed_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact speed policy: unavailable ({error})"),
    }
}

pub(crate) fn validated_motion_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_speed_policy_summary_details();
    match summary.validate() {
        Ok(()) => format!("Motion policy: {}", summary.summary_line()),
        Err(error) => format!("Motion policy: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_access_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_access_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact access: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_generation_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_generation_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_body_cadence_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_body_cadence_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body cadence: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_body_class_span_cap_summary_for_report() -> String {
    format!(
        "Packaged-artifact body-class span caps: {}",
        pleiades_data::packaged_artifact_body_class_span_cap_entries_for_report()
    )
}

pub(crate) fn validated_packaged_artifact_normalized_intermediate_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_normalized_intermediate_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact normalized intermediates: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_storage_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_storage_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact storage/reconstruction: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_frame_treatment_summary_for_report() -> String {
    let summary = pleiades_data::packaged_frame_treatment_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged frame treatment: unavailable ({error})"),
    }
}

pub(crate) fn ensure_packaged_artifact_storage_summary_matches_current_rendering(
    packaged_artifact_storage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_storage_summary_text
        == validated_packaged_artifact_storage_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact storage summary no longer matches the current packaged-artifact storage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_frame_treatment_summary_matches_current_rendering(
    packaged_frame_treatment_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_frame_treatment_summary_text
        == validated_packaged_frame_treatment_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged frame treatment summary no longer matches the current packaged frame treatment posture"
                .to_string(),
        ))
    }
}

pub(crate) fn validated_packaged_artifact_target_threshold_state_for_report() -> String {
    pleiades_data::packaged_artifact_target_threshold_state_for_report()
}

pub(crate) fn validated_packaged_artifact_target_threshold_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_target_threshold_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged-artifact target thresholds: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report(
) -> String {
    let summary =
        pleiades_data::packaged_artifact_target_threshold_scope_envelopes_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("scope envelopes: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_source_fit_holdout_sync_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_source_fit_holdout_sync_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("source-fit and hold-out sync: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_phase2_corpus_alignment_summary_for_report() -> String {
    let summary = match pleiades_data::packaged_artifact_phase2_corpus_alignment_summary_details()
    {
        Some(summary) => summary,
        None => {
            return "Packaged-artifact phase-2 corpus alignment: unavailable (phase-2 corpus evidence should be available)".to_string()
        }
    };

    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged-artifact phase-2 corpus alignment: unavailable ({error})"),
    }
}

pub(crate) fn format_packaged_artifact_output_support_summary() -> String {
    validated_packaged_artifact_output_support_summary_for_report()
}

pub(crate) fn format_packaged_artifact_speed_policy_summary() -> String {
    validated_packaged_artifact_speed_policy_summary_for_report()
}

pub(crate) fn format_packaged_artifact_generation_policy_summary() -> String {
    validated_packaged_artifact_generation_policy_summary_for_report()
}

pub(crate) fn validate_packaged_artifact_generation_residual_bodies_summary(
    summary: &pleiades_compression::ArtifactResidualBodyCoverageSummary,
    artifact: &pleiades_compression::CompressedArtifact,
) -> Result<String, String> {
    summary
        .validated_summary_line_with_body_count(artifact)
        .map_err(|error| error.to_string())
}

pub(crate) fn validated_packaged_artifact_generation_residual_bodies_summary_for_report(
) -> Result<String, String> {
    validate_packaged_artifact_generation_residual_bodies_summary(
        &pleiades_data::packaged_artifact_generation_residual_bodies_summary_details(),
        packaged_artifact(),
    )
}

pub(crate) fn validated_packaged_artifact_production_profile_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_production_profile_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact production profile draft: unavailable ({error})"),
    }
}

pub(crate) fn format_packaged_artifact_storage_summary() -> String {
    validated_packaged_artifact_storage_summary_for_report()
}

pub(crate) fn format_packaged_artifact_access_summary() -> String {
    validated_packaged_artifact_access_summary_for_report()
}

pub(crate) fn format_packaged_frame_parity_summary() -> String {
    packaged_frame_parity_summary_for_report()
}

pub(crate) fn format_lunar_frame_treatment_summary() -> String {
    lunar_theory_frame_treatment_summary_for_report()
}

pub(crate) fn format_packaged_frame_treatment_summary() -> String {
    packaged_frame_treatment_summary_for_report()
}

pub(crate) fn format_comparison_snapshot_manifest_summary() -> String {
    match validated_comparison_snapshot_manifest_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Comparison snapshot manifest: unavailable ({error})"),
    }
}

pub(crate) fn render_validation_report_summary_text(report: &ValidationReport) -> String {
    use std::fmt::Write as _;

    if let Err(error) = report.validate() {
        return format!("Validation report summary unavailable ({error})");
    }

    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Validation report summary unavailable ({error})"),
    };
    let request_policy = request_policy_summary_for_report();
    let comparison_regressions = report.comparison.notable_regressions().len();
    let mut text = String::new();
    let _ = writeln!(text, "Validation report summary");
    let _ = writeln!(
        text,
        "Profile: {}",
        release_profiles.compatibility_profile_id
    );
    let _ = writeln!(
        text,
        "API stability posture: {}",
        release_profiles.api_stability_profile_id
    );
    let _ = writeln!(
        text,
        "Release profile identifiers: {}",
        validated_release_profile_identifiers_summary_for_report(&release_profiles)
    );
    let _ = writeln!(text, "Time-scale policy: {}", request_policy.time_scale);
    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );
    let utc_convenience_policy =
        pleiades_backend::validated_utc_convenience_policy_summary_for_report();
    let _ = writeln!(text, "UTC convenience policy: {}", utc_convenience_policy);
    let _ = writeln!(text, "Observer policy: {}", request_policy.observer);
    let _ = writeln!(text, "Apparentness policy: {}", request_policy.apparentness);
    let native_sidereal_policy =
        pleiades_backend::validated_native_sidereal_policy_summary_for_report();
    let _ = writeln!(text, "Native sidereal policy: {}", native_sidereal_policy);
    let _ = writeln!(text, "Frame policy: {}", request_policy.frame);
    let _ = writeln!(
        text,
        "Mean-obliquity frame round-trip: {}",
        mean_obliquity_frame_round_trip_summary_for_report()
    );
    let _ = writeln!(
        text,
        "Request policy: {}",
        format_request_policy_summary_for_report(&request_policy)
    );
    let _ = writeln!(text, "{}", request_surface_summary_for_report());
    let _ = writeln!(
        text,
        "Zodiac policy: {}",
        validated_zodiac_policy_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison corpus");
    let _ = writeln!(text, "  name: {}", report.comparison_corpus.name);
    let _ = writeln!(
        text,
        "  requests: {}",
        report.comparison_corpus.request_count
    );
    let _ = writeln!(text, "  epochs: {}", report.comparison_corpus.epoch_count);
    let _ = writeln!(
        text,
        "  epoch labels: {}",
        format_instant_list(&report.comparison_corpus.epochs)
    );
    let _ = writeln!(text, "  bodies: {}", report.comparison_corpus.body_count);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.comparison_corpus.apparentness
    );
    let _ = writeln!(text, "  {}", comparison_snapshot_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        comparison_snapshot_body_class_coverage_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        comparison_snapshot_source_summary_for_report()
    );
    let _ = writeln!(text, "  {}", format_comparison_snapshot_manifest_summary());
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => return format!("Comparison corpus summary unavailable ({error})"),
    };
    let _ = writeln!(text, "  release-grade guard: {release_grade_guard}");
    let _ = writeln!(
        text,
        "  Source corpus: {}",
        source_corpus_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Source corpus posture: {}",
        source_corpus_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Reference snapshot");
    let _ = writeln!(text, "  {}", reference_snapshot_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451911_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(text, "  {}", reference_snapshot_source_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_source_window_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_body_class_coverage_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_dense_boundary_summary_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "House validation corpus");
    let _ = writeln!(
        text,
        "  {}",
        house_validation_summary_line_for_report(&report.house_validation)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(
        text,
        "  samples: {}",
        report.comparison.summary.sample_count
    );
    let median = comparison_median_envelope_for_samples(&report.comparison.samples);
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°{}",
        report.comparison.summary.max_longitude_delta_deg,
        format_summary_body(&report.comparison.summary.max_longitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°{}",
        report.comparison.summary.max_latitude_delta_deg,
        format_summary_body(&report.comparison.summary.max_latitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}{}",
        report
            .comparison
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string()),
        format_summary_body(&report.comparison.summary.max_distance_delta_body)
    );
    let _ = writeln!(
        text,
        "  mean longitude delta: {:.12}°",
        report.comparison.summary.mean_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  median longitude delta: {:.12}°",
        median.longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  rms longitude delta: {:.12}°",
        report.comparison.summary.rms_longitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  mean latitude delta: {:.12}°",
        report.comparison.summary.mean_latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  median latitude delta: {:.12}°",
        median.latitude_delta_deg
    );
    let _ = writeln!(
        text,
        "  rms latitude delta: {:.12}°",
        report.comparison.summary.rms_latitude_delta_deg
    );
    if let Some(value) = report.comparison.summary.mean_distance_delta_au {
        let _ = writeln!(text, "  mean distance delta: {:.12} AU", value);
    }
    if let Some(value) = median.distance_delta_au {
        let _ = writeln!(text, "  median distance delta: {:.12} AU", value);
    }
    if let Some(value) = report.comparison.summary.rms_distance_delta_au {
        let _ = writeln!(text, "  rms distance delta: {:.12} AU", value);
    }
    let _ = writeln!(
        text,
        "  {}",
        format_comparison_percentile_envelope_for_report(&report.comparison.samples)
    );
    let _ = writeln!(text, "  notable regressions: {}", comparison_regressions);
    let _ = writeln!(
        text,
        "  regression bodies: {}",
        format_regression_bodies(&report.comparison.notable_regressions())
    );
    let _ = writeln!(
        text,
        "Comparison tolerance policy: {}",
        format_comparison_tolerance_policy_for_report(&report.comparison)
    );
    let _ = writeln!(
        text,
        "Comparison audit: {}",
        comparison_audit_summary_for_report(&report.comparison)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "JPL interpolation quality");
    let _ = writeln!(
        text,
        "  {}",
        format_jpl_interpolation_quality_summary_for_report()
    );
    let _ = writeln!(text, "  {}", jpl_independent_holdout_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        render_reference_holdout_overlap_summary_text()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_bridge_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451916_major_body_interior_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451918_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451919_major_body_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_2451920_major_body_interior_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_mars_jupiter_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_mars_outer_boundary_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        reference_snapshot_major_body_boundary_window_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        independent_holdout_snapshot_batch_parity_summary_text()
    );
    let _ = writeln!(
        text,
        "  {}",
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "JPL request policy: {}",
        jpl_snapshot_request_policy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "{}",
        jpl_snapshot_batch_error_taxonomy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "JPL frame treatment: {}",
        format_jpl_frame_treatment_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark summaries");
    let _ = writeln!(text, "Reference benchmark");
    let _ = writeln!(text, "  corpus: {}", report.reference_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.reference_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.reference_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.reference_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.reference_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.reference_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.reference_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Candidate benchmark");
    let _ = writeln!(text, "  corpus: {}", report.candidate_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.candidate_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.candidate_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.candidate_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.candidate_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.candidate_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.candidate_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-data benchmark");
    let _ = writeln!(text, "  corpus: {}", report.packaged_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.packaged_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.packaged_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.packaged_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.packaged_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.packaged_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.packaged_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged artifact decode benchmark");
    let _ = writeln!(
        text,
        "  artifact: {}",
        report.artifact_decode_benchmark.artifact_label
    );
    let _ = writeln!(
        text,
        "  source: {}",
        report.artifact_decode_benchmark.source
    );
    let _ = writeln!(
        text,
        "  rounds: {}",
        report.artifact_decode_benchmark.rounds
    );
    let _ = writeln!(
        text,
        "  decodes per round: {}",
        report.artifact_decode_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  encoded bytes: {}",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(
        text,
        "  ns/decode: {}",
        format_ns(report.artifact_decode_benchmark.nanoseconds_per_decode())
    );
    let _ = writeln!(
        text,
        "  decodes per second: {:.2} decodes/s",
        report.artifact_decode_benchmark.decodes_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Chart benchmark");
    let _ = writeln!(text, "  corpus: {}", report.chart_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.chart_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.chart_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.chart_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/chart: {}",
        format_ns(report.chart_benchmark.nanoseconds_per_chart())
    );
    let _ = writeln!(
        text,
        "  charts per second: {:.2} charts/s",
        report.chart_benchmark.charts_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "ELP lunar capability: {}",
        lunar_theory_capability_summary_for_report()
    );
    let _ = writeln!(
        text,
        "ELP lunar request policy: {}",
        lunar_theory_request_policy_summary()
    );
    let _ = writeln!(
        text,
        "ELP frame treatment: {}",
        format_lunar_frame_treatment_summary()
    );
    let _ = writeln!(
        text,
        "ELP lunar theory limitations: {}",
        lunar_theory_limitations_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_theory_source_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar reference");
    let _ = writeln!(text, "  {}", lunar_reference_evidence_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        lunar_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_reference_evidence_envelope_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar equatorial reference");
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_evidence_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  {}",
        lunar_equatorial_reference_evidence_envelope_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar apparent comparison");
    let _ = writeln!(text, "  {}", lunar_apparent_comparison_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Lunar source windows");
    let _ = writeln!(text, "  {}", lunar_source_window_summary_for_report());
    let _ = writeln!(text, "Lunar high-curvature continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        lunar_high_curvature_continuity_evidence_for_report()
    );
    let _ = writeln!(text, "Lunar high-curvature equatorial continuity evidence");
    let _ = writeln!(
        text,
        "  {}",
        lunar_high_curvature_equatorial_continuity_evidence_for_report()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Body comparison summaries");
    for summary in report.comparison.body_summaries() {
        let _ = writeln!(
            text,
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, rms Δdist={}",
            summary.body,
            summary.sample_count,
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_longitude_delta_deg,
            summary.rms_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary.mean_latitude_delta_deg,
            summary.rms_latitude_delta_deg,
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .max_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            summary
                .mean_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class error envelopes");
    for summary in report.comparison.body_class_summaries() {
        let max_longitude_body = summary
            .max_longitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_latitude_body = summary
            .max_latitude_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let max_distance_body = summary
            .max_distance_delta_body
            .as_ref()
            .map(|body| format!(" ({body})"))
            .unwrap_or_default();
        let _ = writeln!(
            text,
            "  {}: samples={}, max Δlon={:.12}°{}, mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, max Δlat={:.12}°{}, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, max Δdist={}{}, mean Δdist={}, median Δdist={}, p95 Δdist={}, rms Δdist={}",
            summary.class.label(),
            summary.sample_count,
            summary.max_longitude_delta_deg,
            max_longitude_body,
            summary.mean_longitude_delta_deg(),
            summary.median_longitude_delta_deg,
            summary.percentile_longitude_delta_deg,
            summary.rms_longitude_delta_deg(),
            summary.max_latitude_delta_deg,
            max_latitude_body,
            summary.mean_latitude_delta_deg(),
            summary.median_latitude_delta_deg,
            summary.percentile_latitude_delta_deg,
            summary.rms_latitude_delta_deg(),
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            max_distance_body,
            summary
                .mean_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .median_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .percentile_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class tolerance posture");
    for summary in report.comparison.body_class_tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}",
            format_body_class_tolerance_envelope_for_report(&summary)
        );
        if !summary.outside_bodies.is_empty() {
            let _ = writeln!(
                text,
                "    outside bodies: {}",
                format_bodies(&summary.outside_bodies)
            );
        }
        let _ = writeln!(
            text,
            "    mean Δlon={:.12}°, median Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={}, median Δdist={}, rms Δdist={}",
            summary.mean_longitude_delta_deg(),
            summary.median_longitude_delta_deg,
            summary.rms_longitude_delta_deg(),
            summary.mean_latitude_delta_deg(),
            summary.median_latitude_delta_deg,
            summary.rms_latitude_delta_deg(),
            summary
                .mean_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .median_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .rms_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Tolerance policy");
    write_tolerance_policy_text(&mut text, &report.comparison);
    let _ = writeln!(text);
    let _ = writeln!(text, "Expected tolerance status");
    for summary in report.comparison.tolerance_summaries() {
        let _ = writeln!(
            text,
            "  {}: profile={}, status={}, limit Δlon≤{:.6}°, margin Δlon={:+.12}°, limit Δlat≤{:.6}°, margin Δlat={:+.12}°, limit Δdist={}, margin Δdist={}, measured max Δlon={:.12}°, max Δlat={:.12}°, max Δdist={}",
            summary.body,
            summary.tolerance.profile,
            if summary.within_tolerance { "within" } else { "exceeded" },
            summary.tolerance.max_longitude_delta_deg,
            summary.longitude_margin_deg,
            summary.tolerance.max_latitude_delta_deg,
            summary.latitude_margin_deg,
            summary
                .tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary
                .distance_margin_au
                .map(|value| format!("{value:+.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            summary.max_longitude_delta_deg,
            summary.max_latitude_delta_deg,
            summary
                .max_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison tolerance audit");
    let (audit_body_count, audit_within_count, audit_outside_count, audit_regression_count) =
        comparison_audit_totals(&report.comparison);
    let _ = writeln!(text, "  command: compare-backends-audit");
    let _ = writeln!(
        text,
        "  status: {}",
        if audit_regression_count == 0 {
            "clean"
        } else {
            "regressions found"
        }
    );
    let _ = writeln!(text, "  bodies checked: {}", audit_body_count);
    let _ = writeln!(text, "  within tolerance bodies: {}", audit_within_count);
    let _ = writeln!(text, "  outside tolerance bodies: {}", audit_outside_count);
    let _ = writeln!(text, "  notable regressions: {}", audit_regression_count);
    let _ = writeln!(text);
    let house_validation_summary =
        house_validation_summary_line_for_report(&report.house_validation);
    let house_validation_summary = house_validation_summary
        .strip_prefix("House validation corpus: ")
        .unwrap_or(&house_validation_summary);
    let _ = writeln!(
        text,
        "House validation corpus: {}",
        house_validation_summary
    );
    let _ = writeln!(text, "{}", format_ayanamsa_catalog_validation_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "VSOP87 source-backed evidence");
    let _ = writeln!(text, "  {}", format_vsop87_source_documentation_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_source_documentation_health_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_frame_treatment_summary());
    let _ = writeln!(
        text,
        "  VSOP87 request policy: {}",
        format_vsop87_request_policy_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_source_audit_summary());
    let _ = writeln!(text, "  {}", generated_binary_audit_summary_for_report());
    let _ = writeln!(text, "  {}", format_vsop87_canonical_evidence_summary());
    let _ = writeln!(text, "  {}", format_vsop87_canonical_outlier_note_summary());
    let _ = writeln!(text, "  {}", format_vsop87_equatorial_evidence_summary());
    let _ = writeln!(text, "  {}", format_vsop87_j2000_batch_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j2000_ecliptic_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j2000_equatorial_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j1900_ecliptic_batch_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_supported_body_j1900_equatorial_batch_summary()
    );
    let _ = writeln!(text, "  {}", format_vsop87_mixed_batch_summary());
    let _ = writeln!(text, "  {}", format_vsop87_j1900_batch_summary());
    let _ = writeln!(text, "  {}", format_vsop87_body_evidence_summary());
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_source_body_class_evidence_summary()
    );
    let _ = writeln!(
        text,
        "  {}",
        format_vsop87_equatorial_body_class_evidence_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "ELP lunar theory specification");
    let _ = writeln!(text, "  {}", lunar_theory_catalog_summary_for_report());
    let _ = writeln!(
        text,
        "  {}",
        validated_lunar_theory_catalog_validation_summary_for_report()
    );
    let _ = writeln!(text, "  {}", lunar_theory_source_summary_for_report());
    let _ = writeln!(text, "  {}", lunar_theory_summary_for_report());
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-artifact profile");
    let _ = writeln!(text, "  {}", format_packaged_artifact_profile_summary());
    let _ = writeln!(
        text,
        "  Packaged-artifact output support: {}",
        format_packaged_artifact_output_support_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact speed policy: {}",
        format_packaged_artifact_speed_policy_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact storage/reconstruction: {}",
        format_packaged_artifact_storage_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact access: {}",
        format_packaged_artifact_access_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation policy: {}",
        format_packaged_artifact_generation_policy_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact normalized intermediates: {}",
        packaged_artifact_normalized_intermediate_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation residual bodies: {}",
        match validated_packaged_artifact_generation_residual_bodies_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => return format!("Validation report summary unavailable ({error})"),
        }
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target thresholds: {}",
        validated_packaged_artifact_target_threshold_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target-threshold state: {}",
        validated_packaged_artifact_target_threshold_state_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit envelope: {}",
        packaged_artifact_fit_envelope_summary_for_report()
    );
    let fit_margin_summary = report_summary_payload(
        packaged_artifact_fit_margin_summary_for_report(),
        "fit margins: ",
    );
    let fit_threshold_violation_count_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_count_for_report(),
        "fit threshold violations: ",
    );
    let fit_threshold_violation_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_summary_for_report(),
        "fit threshold violations: ",
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit margins: {}",
        fit_margin_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit threshold violation count: {}",
        fit_threshold_violation_count_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit threshold violations: {}",
        fit_threshold_violation_summary
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit sample classes: {}",
        packaged_artifact_fit_sample_classes_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact fit outliers: {}",
        packaged_artifact_fit_outlier_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact target-threshold scope envelopes: {}",
        validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact source-fit and hold-out sync: {}",
        validated_packaged_artifact_source_fit_holdout_sync_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact phase-2 corpus alignment: {}",
        validated_packaged_artifact_phase2_corpus_alignment_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact generation manifest: {}",
        packaged_artifact_generation_manifest_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged-artifact size: {} bytes",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(text, "  {}", packaged_request_policy_summary_for_report());
    let _ = writeln!(
        text,
        "  Packaged lookup epoch policy: {}",
        packaged_lookup_epoch_policy_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged batch parity: {}",
        packaged_mixed_tt_tdb_batch_parity_summary_for_report()
    );
    let _ = writeln!(
        text,
        "  Packaged frame parity: {}",
        format_packaged_frame_parity_summary()
    );
    let _ = writeln!(
        text,
        "  Packaged frame treatment: {}",
        format_packaged_frame_treatment_summary()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark summaries");
    let _ = writeln!(text, "Reference benchmark");
    let _ = writeln!(text, "  corpus: {}", report.reference_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.reference_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.reference_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.reference_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.reference_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.reference_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.reference_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Candidate benchmark");
    let _ = writeln!(text, "  corpus: {}", report.candidate_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.candidate_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.candidate_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.candidate_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.candidate_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.candidate_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.candidate_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-data benchmark");
    let _ = writeln!(text, "  corpus: {}", report.packaged_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.packaged_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.packaged_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.packaged_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/request (single): {}",
        format_ns(report.packaged_benchmark.nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  ns/request (batch): {}",
        format_ns(report.packaged_benchmark.batch_nanoseconds_per_request())
    );
    let _ = writeln!(
        text,
        "  batch throughput: {:.2} req/s",
        report.packaged_benchmark.batch_requests_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged artifact decode benchmark");
    let _ = writeln!(
        text,
        "  artifact: {}",
        report.artifact_decode_benchmark.artifact_label
    );
    let _ = writeln!(
        text,
        "  source: {}",
        report.artifact_decode_benchmark.source
    );
    let _ = writeln!(
        text,
        "  rounds: {}",
        report.artifact_decode_benchmark.rounds
    );
    let _ = writeln!(
        text,
        "  decodes per round: {}",
        report.artifact_decode_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  encoded bytes: {}",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let _ = writeln!(
        text,
        "  ns/decode: {}",
        format_ns(report.artifact_decode_benchmark.nanoseconds_per_decode())
    );
    let _ = writeln!(
        text,
        "  decodes per second: {:.2} decodes/s",
        report.artifact_decode_benchmark.decodes_per_second()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Chart benchmark");
    let _ = writeln!(text, "  corpus: {}", report.chart_benchmark.corpus_name);
    let _ = writeln!(
        text,
        "  apparentness: {}",
        report.chart_benchmark.apparentness
    );
    let _ = writeln!(text, "  rounds: {}", report.chart_benchmark.rounds);
    let _ = writeln!(
        text,
        "  samples per round: {}",
        report.chart_benchmark.sample_count
    );
    let _ = writeln!(
        text,
        "  ns/chart: {}",
        format_ns(report.chart_benchmark.nanoseconds_per_chart())
    );
    let _ = writeln!(
        text,
        "  charts per second: {:.2} charts/s",
        report.chart_benchmark.charts_per_second()
    );
    let _ = writeln!(text, "Release bundle verification: verify-release-bundle");
    let _ = writeln!(text, "Workspace audit: workspace-audit / audit");
    let _ = writeln!(
        text,
        "Compatibility profile summary: compatibility-profile-summary"
    );
    let _ = writeln!(text, "Release notes summary: release-notes-summary");
    let _ = writeln!(text, "Release checklist summary: release-checklist-summary");
    let _ = writeln!(text, "Release summary: release-summary");

    text
}

/// Renders a compact summary of the implemented backend capability matrix catalog.
pub fn render_backend_matrix_summary() -> String {
    render_backend_matrix_summary_text()
}

pub(crate) fn native_sidereal_posture_line(native_sidereal_count: usize) -> String {
    match native_sidereal_count {
        0 => "Native sidereal posture: unsupported across first-party backends".to_string(),
        1 => "Native sidereal posture: supported natively by 1 backend".to_string(),
        count => format!("Native sidereal posture: supported natively by {count} backends"),
    }
}

pub(crate) fn render_backend_matrix_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    };
    if let Err(error) = validated_compatibility_profile_for_report() {
        return format!("Backend matrix summary unavailable ({error})");
    }
    let catalog = implemented_backend_catalog();
    let mut family_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut bodies: Vec<String> = Vec::new();
    let mut frames: Vec<String> = Vec::new();
    let mut time_scales: Vec<String> = Vec::new();
    let mut deterministic_count = 0usize;
    let mut offline_count = 0usize;
    let mut batch_count = 0usize;
    let mut native_sidereal_count = 0usize;
    let mut bounded_nominal_range_count = 0usize;
    let mut open_ended_nominal_range_count = 0usize;
    let mut exact_accuracy_count = 0usize;
    let mut high_accuracy_count = 0usize;
    let mut moderate_accuracy_count = 0usize;
    let mut approximate_accuracy_count = 0usize;
    let mut unknown_accuracy_count = 0usize;
    let mut selected_asteroid_count = 0usize;
    let mut data_source_count = 0usize;
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();

    for entry in &catalog {
        *status_counts
            .entry(entry.implementation_status.label().to_string())
            .or_insert(0) += 1;

        *family_counts
            .entry(backend_family_label(&entry.metadata.family))
            .or_insert(0) += 1;
        deterministic_count += usize::from(entry.metadata.deterministic);
        offline_count += usize::from(entry.metadata.offline);
        batch_count += usize::from(entry.metadata.capabilities.batch);
        native_sidereal_count += usize::from(entry.metadata.capabilities.native_sidereal);
        if entry.metadata.nominal_range.start.is_some()
            || entry.metadata.nominal_range.end.is_some()
        {
            bounded_nominal_range_count += 1;
        } else {
            open_ended_nominal_range_count += 1;
        }
        match entry.metadata.accuracy {
            AccuracyClass::Exact => exact_accuracy_count += 1,
            AccuracyClass::High => high_accuracy_count += 1,
            AccuracyClass::Moderate => moderate_accuracy_count += 1,
            AccuracyClass::Approximate => approximate_accuracy_count += 1,
            AccuracyClass::Unknown => unknown_accuracy_count += 1,
            _ => unknown_accuracy_count += 1,
        }
        if selected_asteroid_coverage(&entry.metadata.body_coverage).is_some() {
            selected_asteroid_count += 1;
        }
        if !entry.metadata.provenance.data_sources.is_empty() {
            data_source_count += 1;
        }
        for body in &entry.metadata.body_coverage {
            push_unique(&mut bodies, body.to_string());
        }
        for frame in &entry.metadata.supported_frames {
            push_unique(&mut frames, frame.to_string());
        }
        for scale in &entry.metadata.supported_time_scales {
            push_unique(&mut time_scales, scale.to_string());
        }
    }

    let mut family_entries = family_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    family_entries.sort();

    let mut status_entries = status_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    status_entries.sort();

    let mut text = String::new();
    text.push_str("Backend matrix summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Backends: ");
    text.push_str(&catalog.len().to_string());
    text.push('\n');
    text.push_str("Families: ");
    text.push_str(&family_entries.join(", "));
    text.push('\n');
    text.push_str("Implementation statuses: ");
    text.push_str(&status_entries.join(", "));
    text.push('\n');
    text.push_str("Deterministic backends: ");
    text.push_str(&deterministic_count.to_string());
    text.push('\n');
    text.push_str("Offline backends: ");
    text.push_str(&offline_count.to_string());
    text.push('\n');
    text.push_str("Batch-capable backends: ");
    text.push_str(&batch_count.to_string());
    text.push('\n');
    text.push_str("Native sidereal backends: ");
    text.push_str(&native_sidereal_count.to_string());
    text.push('\n');
    text.push_str(&native_sidereal_posture_line(native_sidereal_count));
    text.push('\n');
    text.push_str("Nominal ranges: bounded: ");
    text.push_str(&bounded_nominal_range_count.to_string());
    text.push_str(", open-ended: ");
    text.push_str(&open_ended_nominal_range_count.to_string());
    text.push('\n');
    text.push_str("Accuracy classes: Exact: ");
    text.push_str(&exact_accuracy_count.to_string());
    text.push_str(", High: ");
    text.push_str(&high_accuracy_count.to_string());
    text.push_str(", Moderate: ");
    text.push_str(&moderate_accuracy_count.to_string());
    text.push_str(", Approximate: ");
    text.push_str(&approximate_accuracy_count.to_string());
    text.push_str(", Unknown: ");
    text.push_str(&unknown_accuracy_count.to_string());
    text.push('\n');
    text.push_str("Backends with selected asteroid coverage: ");
    text.push_str(&selected_asteroid_count.to_string());
    text.push('\n');
    text.push_str(&selected_asteroid_source_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_terminal_boundary_summary_for_report());
    text.push('\n');
    text.push_str("Comparison corpus release-grade guard: ");
    match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(summary) => text.push_str(summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Reference/hold-out overlap: ");
    text.push_str(&render_reference_holdout_overlap_summary_text());
    text.push('\n');
    text.push_str("JPL independent hold-out: ");
    text.push_str(&jpl_independent_holdout_summary_for_report());
    text.push('\n');
    text.push_str("Release-grade body claims: ");
    text.push_str(&format_release_body_claims_summary_for_report());
    text.push('\n');
    text.push_str("Body/date/channel claims: ");
    text.push_str(&format_body_date_channel_claims_summary_for_report());
    text.push('\n');
    text.push_str("Source corpus: ");
    text.push_str(&source_corpus_summary_for_report());
    text.push('\n');
    text.push_str("Source corpus posture: ");
    text.push_str(&source_corpus_posture_summary_for_report());
    text.push('\n');
    text.push_str("JPL source corpus contract: ");
    match required_labelled_summary_payload(
        jpl_source_corpus_contract_summary_for_report(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    ) {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Catalog posture: ");
    match core_validated_catalog_posture_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Target house scope: ");
    match core_validated_target_house_scope_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Target ayanamsa scope: ");
    match core_validated_target_ayanamsa_scope_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Pluto fallback: ");
    match validated_pluto_fallback_summary_line_for_report() {
        Ok(summary) => text.push_str(summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("House code aliases: ");
    match validated_house_code_aliases_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_boundary_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_sparse_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_pre_bridge_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1500_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1600_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1750_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2360234_major_body_interior_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1900_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_early_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1800_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2500_major_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str(&validated_production_generation_manifest_summary_text_for_report());
    text.push('\n');
    text.push_str("Production generation source revision: ");
    match validated_production_generation_source_revision_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Production generation source: ");
    text.push_str(&production_generation_source_summary_for_report());
    text.push('\n');
    text.push_str("Production generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&production_generation_snapshot_window_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation body-class coverage: ");
    text.push_str(&validated_production_generation_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation corpus shape: ");
    text.push_str(&production_generation_corpus_shape_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary request corpus equatorial: ");
    text.push_str(&production_generation_boundary_request_corpus_equatorial_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_source_corpus_contract_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&format_comparison_snapshot_manifest_summary());
    text.push('\n');
    if let Ok(report) = build_validation_report(SUMMARY_BENCHMARK_ROUNDS) {
        text.push_str("Comparison audit: compare-backends-audit; ");
        text.push_str(&comparison_audit_summary_for_report(&report.comparison));
        text.push('\n');
    }
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text.push_str(&request_surface_summary_for_report());
    text.push('\n');
    text.push_str("Frame policy: ");
    text.push_str(&validated_frame_policy_summary_for_report());
    text.push('\n');
    text.push_str("Mean-obliquity frame round-trip: ");
    text.push_str(&mean_obliquity_frame_round_trip_summary_for_report());
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&validated_zodiac_policy_summary_for_report());
    text.push('\n');
    text.push_str("Backends with external data sources: ");
    text.push_str(&data_source_count.to_string());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_health_summary());
    text.push('\n');
    text.push_str(&format_vsop87_frame_treatment_summary());
    text.push('\n');
    text.push_str("VSOP87 request policy: ");
    text.push_str(&format_vsop87_request_policy_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_audit_summary());
    text.push('\n');
    text.push_str(&generated_binary_audit_summary_for_report());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_outlier_note_summary());
    text.push('\n');
    text.push_str(&format_vsop87_equatorial_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j2000_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_mixed_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j1900_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_body_evidence_summary());
    text.push('\n');
    text.push_str(&lunar_theory_catalog_summary_for_report());
    text.push('\n');
    text.push_str(&validated_lunar_theory_catalog_validation_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_source_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference\n");
    text.push_str(&lunar_equatorial_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference batch parity\n");
    text.push_str(&lunar_equatorial_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_equatorial_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar source windows: ");
    text.push_str(&lunar_source_window_summary_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature continuity evidence\n");
    text.push_str(&lunar_high_curvature_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature equatorial continuity evidence\n");
    text.push_str(&lunar_high_curvature_equatorial_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Distinct bodies covered: ");
    text.push_str(&bodies.len().to_string());
    text.push_str(" (");
    text.push_str(&bodies.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct coordinate frames: ");
    text.push_str(&frames.len().to_string());
    text.push_str(" (");
    text.push_str(&frames.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct time scales: ");
    text.push_str(&time_scales.len().to_string());
    text.push_str(" (");
    text.push_str(&time_scales.join(", "));
    text.push_str(")\n");
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str("Time-scale policy: ");
    text.push_str(&format_time_scale_policy_summary_for_report(
        &time_scale_policy,
    ));
    text.push('\n');
    text.push_str("Delta T policy: ");
    text.push_str(&format_delta_t_policy_summary_for_report(
        &delta_t_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Observer policy: ");
    text.push_str(&format_observer_policy_summary_for_report(
        &pleiades_backend::observer_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Apparentness policy: ");
    text.push_str(&format_apparentness_policy_summary_for_report(
        &pleiades_backend::apparentness_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Native sidereal policy: ");
    text.push_str(&pleiades_backend::validated_native_sidereal_policy_summary_for_report());
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&validated_zodiac_policy_summary_for_report());
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&validated_release_profile_identifiers_summary_for_report(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

/// Renders a compact summary of the API stability posture.
pub fn render_api_stability_summary() -> String {
    render_api_stability_summary_text()
}

pub(crate) fn render_api_stability_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("API stability summary unavailable ({error})"),
    };

    match validated_api_stability_profile_for_report() {
        Ok(profile) => {
            let mut text = String::new();

            text.push_str("API stability summary\n");
            text.push_str("Profile: ");
            text.push_str(profile.profile_id);
            text.push('\n');
            text.push_str("Summary line: ");
            text.push_str(&profile.summary_line());
            text.push('\n');
            text.push_str("Compatibility profile: ");
            text.push_str(release_profiles.compatibility_profile_id);
            text.push('\n');
            text.push_str("Release profile identifiers: ");
            text.push_str(&validated_release_profile_identifiers_summary_for_report(
                &release_profiles,
            ));
            text.push('\n');
            text.push_str("Stable surfaces: ");
            text.push_str(&profile.stable_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Experimental surfaces: ");
            text.push_str(&profile.experimental_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Deprecation policy items: ");
            text.push_str(&profile.deprecation_policy.len().to_string());
            text.push('\n');
            text.push_str("Intentional limits: ");
            text.push_str(&profile.intentional_limits.len().to_string());
            text.push('\n');
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("Backend matrix summary: backend-matrix-summary\n");
            text.push_str("Release notes summary: release-notes-summary\n");
            text.push_str("Release checklist summary: release-checklist-summary\n");
            text.push_str("Release bundle verification: verify-release-bundle\n");
            text.push_str("See release-summary for the compact one-screen release overview.\n");

            text
        }
        Err(error) => format!("API stability summary unavailable ({error})"),
    }
}

pub(crate) fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

pub(crate) fn backend_family_label(family: &BackendFamily) -> String {
    family.to_string()
}

/// Renders a backend capability matrix for the implemented backend catalog.
pub fn render_backend_matrix_report() -> Result<String, EphemerisError> {
    validated_compatibility_profile_for_report().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("backend capability matrix unavailable ({error})"),
        )
    })?;
    let mut rendered = String::new();
    fmt::write(
        &mut rendered,
        format_args!("Implemented backend matrices\n\n"),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    let house_code_aliases =
        validated_house_code_aliases_summary_for_report().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("backend capability matrix unavailable ({error})"),
            )
        })?;

    fmt::write(
        &mut rendered,
        format_args!("House code aliases: {}\n\n", house_code_aliases),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    fmt::write(
        &mut rendered,
        format_args!(
            "Body/date/channel claims: {}\n\n",
            format_body_date_channel_claims_summary_for_report()
        ),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    for entry in implemented_backend_catalog() {
        validate_backend_matrix_entry(&entry)?;
        fmt::write(&mut rendered, format_args!("{}\n", entry.label)).map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
        fmt::write(
            &mut rendered,
            format_args!("{}\n\n", BackendMatrixDisplay(&entry)),
        )
        .map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
    }

    Ok(rendered)
}

pub(crate) fn validate_backend_matrix_entry(
    entry: &BackendMatrixEntry,
) -> Result<(), EphemerisError> {
    entry.metadata.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "backend matrix entry `{}` has invalid metadata: {error}",
                entry.label
            ),
        )
    })
}

pub(crate) struct BackendMatrixDisplay<'a>(&'a BackendMatrixEntry);

impl fmt::Display for BackendMatrixDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_backend_catalog_entry(f, self.0)
    }
}

pub(crate) fn write_corpus_summary(
    f: &mut fmt::Formatter<'_>,
    corpus: &CorpusSummary,
) -> fmt::Result {
    if let Err(error) = corpus.validate() {
        writeln!(f, "  corpus summary unavailable ({error})")?;
        return Ok(());
    }

    writeln!(f, "  name: {}", corpus.name)?;
    writeln!(f, "  description: {}", corpus.description)?;
    writeln!(f, "  Apparentness: {}", corpus.apparentness)?;
    writeln!(f, "  requests: {}", corpus.request_count)?;
    writeln!(f, "  epochs: {}", corpus.epoch_count)?;
    writeln!(f, "  epoch labels: {}", format_instant_list(&corpus.epochs))?;
    writeln!(f, "  bodies: {}", corpus.body_count)?;
    writeln!(
        f,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    )
}

pub(crate) fn write_corpus_summary_text(text: &mut String, corpus: &CorpusSummary) {
    use std::fmt::Write as _;

    if let Err(error) = corpus.validate() {
        let _ = writeln!(text, "  corpus summary unavailable ({error})");
        return;
    }

    let _ = writeln!(text, "  name: {}", corpus.name);
    let _ = writeln!(text, "  description: {}", corpus.description);
    let _ = writeln!(text, "  Apparentness: {}", corpus.apparentness);
    let _ = writeln!(text, "  requests: {}", corpus.request_count);
    let _ = writeln!(text, "  epochs: {}", corpus.epoch_count);
    let _ = writeln!(
        text,
        "  epoch labels: {}",
        format_instant_list(&corpus.epochs)
    );
    let _ = writeln!(text, "  bodies: {}", corpus.body_count);
    let _ = writeln!(
        text,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    );
}

pub(crate) fn write_backend_matrix(
    f: &mut fmt::Formatter<'_>,
    backend: &BackendMetadata,
) -> fmt::Result {
    writeln!(
        f,
        "  summary: {}",
        backend
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    writeln!(f, "  id: {}", backend.id)?;
    writeln!(f, "  version: {}", backend.version)?;
    writeln!(f, "  family: {}", backend.family)?;
    writeln!(f, "  family posture: {}", backend.family.posture())?;
    writeln!(f, "  accuracy: {}", backend.accuracy)?;
    writeln!(f, "  deterministic: {}", backend.deterministic)?;
    writeln!(f, "  offline: {}", backend.offline)?;
    writeln!(f, "  nominal range: {}", backend.nominal_range)?;
    writeln!(
        f,
        "  time scales: {}",
        format_time_scales(&backend.supported_time_scales)
    )?;
    writeln!(f, "  bodies: {}", format_bodies(&backend.body_coverage))?;
    if let Some(asteroids) = selected_asteroid_coverage(&backend.body_coverage) {
        writeln!(
            f,
            "  {}",
            selected_asteroid_coverage_summary_for_report(&asteroids)
        )?;
        if backend.id.as_str() == "jpl-snapshot" {
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_evidence_summary_for_report()
            )?;
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_window_summary_for_report()
            )?;
            writeln!(f, "  {}", selected_asteroid_boundary_summary_for_report())?;
            writeln!(f, "  {}", selected_asteroid_bridge_summary_for_report())?;
            let evidence = reference_asteroid_evidence();
            if let Some(first) = evidence.first() {
                writeln!(
                    f,
                    "  exact J2000 evidence: {} bodies at JD {:.1}",
                    evidence.len(),
                    first.epoch.julian_day.days()
                )?;
                for sample in evidence {
                    writeln!(
                        f,
                        "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                        sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                    )?;
                }
            }
            writeln!(
                f,
                "  {}",
                reference_snapshot_exact_j2000_evidence_summary_for_report()
            )?;
            writeln!(
                f,
                "  {}",
                reference_snapshot_major_body_bridge_summary_for_report()
            )?;
        }
    }
    writeln!(f, "  frames: {}", format_frames(&backend.supported_frames))?;
    writeln!(
        f,
        "  capabilities: {}",
        format_capabilities(&backend.capabilities)
    )?;
    writeln!(
        f,
        "  provenance: {}",
        backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    if !backend.provenance.data_sources.is_empty() {
        writeln!(
            f,
            "  provenance sources: {}",
            backend.provenance.data_sources.join("; ")
        )?;
    }
    Ok(())
}

pub(crate) fn write_backend_catalog_entry(
    f: &mut fmt::Formatter<'_>,
    entry: &BackendMatrixEntry,
) -> fmt::Result {
    write_backend_matrix(f, &entry.metadata)?;
    writeln!(
        f,
        "  implementation status: {}",
        entry.implementation_status.label()
    )?;
    writeln!(f, "  implementation note: {}", entry.status_note)?;
    if entry.metadata.id.as_str() == "pleiades-vsop87" {
        writeln!(f, "  body source profiles:")?;
        for profile in body_source_profiles() {
            writeln!(f, "    {}", profile.summary_line())?;
        }

        writeln!(f, "  source documentation:")?;
        for spec in source_specifications() {
            writeln!(
                f,
                "    {}: {} {} | {} | {} | {} | {} | {} | {} | {}",
                spec.body,
                spec.variant,
                spec.source_file,
                spec.coordinate_family,
                spec.frame,
                spec.units,
                spec.reduction,
                spec.transform_note,
                spec.truncation_policy,
                spec.date_range
            )?;
        }

        writeln!(f, "  source audit:")?;
        for audit in source_audits() {
            writeln!(
                f,
                "    {}: {} bytes, {} lines, {} terms, 0x{:016x}",
                audit.body,
                audit.byte_length,
                audit.line_count,
                audit.term_count,
                audit.fingerprint
            )?;
        }

        writeln!(f, "  generated binary audit:")?;
        writeln!(f, "    {}", generated_binary_audit_summary_for_report())?;

        writeln!(f, "  canonical J2000 VSOP87B evidence:")?;
        match vsop87_canonical_body_evidence() {
            Some(body_evidence) => {
                for evidence in body_evidence {
                    writeln!(
                        f,
                        "    {}: kind={} from {} — {} — Δlon={:.12}° / limit {:.12}° / margin {:+.12}°, Δlat={:.12}° / limit {:.12}° / margin {:+.12}°, Δdist={:.12} AU / limit {:.12} AU / margin {:+.12} AU",
                        evidence.body,
                        evidence.source_kind,
                        evidence.source_file,
                        if evidence.within_interim_limits {
                            evidence.provenance
                        } else {
                            "outside interim limits"
                        },
                        evidence.longitude_delta_deg,
                        evidence.longitude_limit_deg,
                        evidence.longitude_limit_deg - evidence.longitude_delta_deg,
                        evidence.latitude_delta_deg,
                        evidence.latitude_limit_deg,
                        evidence.latitude_limit_deg - evidence.latitude_delta_deg,
                        evidence.distance_delta_au,
                        evidence.distance_limit_au,
                        evidence.distance_limit_au - evidence.distance_delta_au
                    )?;
                }
            }
            None => {
                writeln!(f, "    unavailable")?;
            }
        }
        writeln!(
            f,
            "  body profile evidence summary: {}",
            format_vsop87_body_evidence_summary()
        )?;
    } else if entry.metadata.id.as_str() == "pleiades-elp" {
        let theory = lunar_theory_specification();
        writeln!(f, "  lunar theory specification:")?;
        writeln!(
            f,
            "    catalog summary: {}",
            lunar_theory_catalog_summary_for_report()
        )?;
        writeln!(
            f,
            "    catalog validation: {}",
            validated_lunar_theory_catalog_validation_summary_for_report()
        )?;
        writeln!(f, "    model: {}", theory.model_name)?;
        writeln!(
            f,
            "    source family: {}",
            pleiades_elp::lunar_theory_source_family().label()
        )?;
        writeln!(
            f,
            "    capability summary: {}",
            lunar_theory_capability_summary_for_report()
        )?;
        writeln!(
            f,
            "    specification summary: {}",
            lunar_theory_summary_for_report()
        )?;
        writeln!(f, "    source identifier: {}", theory.source_identifier)?;
        writeln!(f, "    source citation: {}", theory.source_citation)?;
        writeln!(f, "    source material: {}", theory.source_material)?;
        writeln!(f, "    redistribution note: {}", theory.redistribution_note)?;
        writeln!(f, "    license note: {}", theory.license_note)?;
        writeln!(
            f,
            "    supported bodies: {}",
            format_bodies(theory.supported_bodies)
        )?;
        writeln!(
            f,
            "    unsupported bodies: {}",
            format_bodies(theory.unsupported_bodies)
        )?;
        writeln!(
            f,
            "    request policy: {}",
            lunar_theory_request_policy_summary()
        )?;
        writeln!(f, "    validation window: {}", theory.validation_window)?;
        writeln!(f, "    date-range note: {}", theory.date_range_note)?;
        writeln!(f, "    frame note: {}", theory.frame_note)?;
        write_lunar_reference_evidence(f)?;
        write_lunar_equatorial_reference_evidence(f)?;
        write_lunar_apparent_comparison_evidence(f)?;
        write_lunar_source_window_evidence(f)?;
        writeln!(f, "  Lunar high-curvature continuity evidence:")?;
        writeln!(
            f,
            "    {}",
            lunar_high_curvature_continuity_evidence_for_report()
        )?;
        write_lunar_high_curvature_equatorial_continuity_evidence(f)?;
    }
    if entry.metadata.id.as_str() == "jpl-snapshot" {
        write_jpl_interpolation_quality(f)?;
        writeln!(
            f,
            "    {}",
            jpl_snapshot_batch_error_taxonomy_summary_for_report()
        )?;
    }
    writeln!(
        f,
        "  expected error classes: {}",
        format_error_kinds(entry.expected_error_kinds)
    )?;
    if entry.required_data_files.is_empty() {
        writeln!(f, "  required external data files: none")?;
    } else {
        writeln!(
            f,
            "  required external data files: {}",
            format_data_files(entry.required_data_files)
        )?;
    }
    Ok(())
}

pub(crate) fn write_jpl_interpolation_quality(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  interpolation quality checks:")?;
    let Some(summary) = jpl_interpolation_quality_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        format_jpl_interpolation_quality_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        jpl_interpolation_quality_kind_coverage_for_report()
    )?;
    writeln!(f, "    {}", jpl_interpolation_posture_summary_for_report())?;
    writeln!(f, "    {}", jpl_independent_holdout_summary_for_report())?;
    writeln!(f, "    {}", render_reference_holdout_overlap_summary_text())?;
    writeln!(
        f,
        "    {}",
        independent_holdout_snapshot_body_class_coverage_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        independent_holdout_snapshot_batch_parity_summary_text()
    )?;
    writeln!(
        f,
        "    {}",
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    )?;
    for sample in interpolation_quality_samples() {
        writeln!(f, "    {}", sample.summary_line())?;
    }
    writeln!(
        f,
        "    {}",
        jpl_interpolation_body_class_error_envelopes_for_report()
    )?;
    Ok(())
}

pub(crate) fn jpl_interpolation_quality_summary(
) -> Option<pleiades_jpl::JplInterpolationQualitySummary> {
    pleiades_jpl::jpl_interpolation_quality_summary()
}

pub(crate) fn format_jpl_interpolation_quality_summary(
    summary: &pleiades_jpl::JplInterpolationQualitySummary,
) -> String {
    pleiades_jpl::format_jpl_interpolation_quality_summary(summary)
}

pub(crate) fn write_lunar_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar reference:")?;
    let Some(summary) = lunar_reference_evidence_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        pleiades_elp::format_lunar_reference_evidence_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        pleiades_elp::lunar_reference_batch_parity_summary_for_report()
    )?;
    writeln!(f, "    {}", lunar_reference_evidence_envelope_for_report())?;
    for sample in lunar_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

pub(crate) fn write_lunar_equatorial_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar equatorial reference:")?;
    if lunar_equatorial_reference_evidence_summary().is_none() {
        writeln!(f, "    none")?;
        return Ok(());
    }

    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_evidence_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_batch_parity_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        lunar_equatorial_reference_evidence_envelope_for_report()
    )?;
    for sample in lunar_equatorial_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

pub(crate) fn write_lunar_apparent_comparison_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar apparent comparison:")?;
    let Some(summary) = lunar_apparent_comparison_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(f, "    {}", summary.summary_line())?;
    for sample in lunar_apparent_comparison_evidence() {
        writeln!(
            f,
            "    {} at JD {:.1}: apparent lon={:.12}°, apparent lat={:.12}°, apparent dist={:.12} AU, apparent RA={:.12}°, apparent Dec={:.12}°, note={}",
            sample.body,
            sample.epoch.julian_day.days(),
            sample.apparent_longitude_deg,
            sample.apparent_latitude_deg,
            sample.apparent_distance_au,
            sample.apparent_right_ascension_deg,
            sample.apparent_declination_deg,
            sample.note
        )?;
    }
    Ok(())
}

pub(crate) fn write_lunar_source_window_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar source windows:")?;
    writeln!(f, "    {}", lunar_source_window_summary_for_report())?;
    Ok(())
}

pub(crate) fn write_lunar_high_curvature_equatorial_continuity_evidence(
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(f, "  Lunar high-curvature equatorial continuity evidence:")?;
    writeln!(
        f,
        "    {}",
        lunar_high_curvature_equatorial_continuity_evidence_for_report()
    )?;
    Ok(())
}

pub(crate) fn write_comparison_summary(
    f: &mut fmt::Formatter<'_>,
    report: &ComparisonReport,
) -> fmt::Result {
    let summary = &report.summary;
    let comparison_envelope = comparison_envelope_summary(summary, &report.samples);
    let median = comparison_envelope.median;

    writeln!(f, "  samples: {}", summary.sample_count)?;
    writeln!(
        f,
        "  max longitude delta: {:.12}°",
        summary.max_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean longitude delta: {:.12}°",
        summary.mean_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  median longitude delta: {:.12}°",
        median.longitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms longitude delta: {:.12}°",
        summary.rms_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  max latitude delta: {:.12}°",
        summary.max_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean latitude delta: {:.12}°",
        summary.mean_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  median latitude delta: {:.12}°",
        median.latitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms latitude delta: {:.12}°",
        summary.rms_latitude_delta_deg
    )?;
    if let Some(value) = summary.max_distance_delta_au {
        writeln!(f, "  max distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.mean_distance_delta_au {
        writeln!(f, "  mean distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = median.distance_delta_au {
        writeln!(f, "  median distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.rms_distance_delta_au {
        writeln!(f, "  rms distance delta: {:.12} AU", value)?;
    }
    match comparison_envelope.validated_percentile_line(&report.samples) {
        Ok(line) => writeln!(f, "  {line}")?,
        Err(error) => writeln!(f, "  comparison percentile envelope unavailable ({error})")?,
    }
    Ok(())
}

pub(crate) fn format_body_comparison_summary_for_report(summary: &BodyComparisonSummary) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!(
            "body comparison summary for {} unavailable ({error})",
            summary.body
        ),
    }
}

pub(crate) fn write_body_comparison_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyComparisonSummary],
) -> fmt::Result {
    writeln!(f, "Body comparison summaries")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        writeln!(
            f,
            "  {}",
            format_body_comparison_summary_for_report(summary)
        )?;
    }
    Ok(())
}

pub(crate) fn write_body_class_envelopes(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
) -> fmt::Result {
    writeln!(f, "Body-class error envelopes")?;
    let summaries = body_class_summaries(samples);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

pub(crate) fn write_body_class_tolerance_posture(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
    backend_family: &BackendFamily,
) -> fmt::Result {
    writeln!(f, "Body-class tolerance posture")?;
    let summaries = body_class_tolerance_summaries(samples, backend_family);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

pub(crate) fn tolerance_backend_family_label(family: &BackendFamily) -> String {
    match family {
        BackendFamily::Algorithmic => "algorithmic".to_string(),
        BackendFamily::ReferenceData => "reference data".to_string(),
        BackendFamily::CompressedData => "compressed data".to_string(),
        BackendFamily::Composite => "composite".to_string(),
        BackendFamily::Other(value) => format!("other ({value})"),
        _ => "other (unknown)".to_string(),
    }
}

pub(crate) fn write_tolerance_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyToleranceSummary],
) -> fmt::Result {
    writeln!(f, "Expected tolerance status")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        match summary.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(
                f,
                "  body tolerance summary for {} unavailable ({error})",
                summary.body
            ),
        }?;
    }
    Ok(())
}

pub(crate) fn write_regression_section(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    findings: &[RegressionFinding],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    if findings.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in findings {
        match finding.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(f, "  regression finding unavailable ({error})"),
        }?;
    }
    Ok(())
}

pub(crate) fn write_regression_archive_section(
    f: &mut fmt::Formatter<'_>,
    archive: &RegressionArchive,
) -> fmt::Result {
    writeln!(f, "Archived regression cases")?;
    writeln!(f, "  corpus: {}", archive.corpus_name)?;
    if archive.cases.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in &archive.cases {
        match finding.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(f, "  regression finding unavailable ({error})"),
        }?;
    }
    Ok(())
}

pub(crate) fn write_reference_asteroid_section(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Selected asteroid coverage")?;
    let asteroids = reference_asteroids();
    if asteroids.is_empty() {
        writeln!(f, "  none")?;
    } else {
        writeln!(
            f,
            "  {}",
            selected_asteroid_coverage_summary_for_report(asteroids)
        )?;
        let evidence = reference_asteroid_evidence();
        if evidence.is_empty() {
            writeln!(f, "  exact J2000 evidence: unavailable")?;
        } else {
            writeln!(
                f,
                "  exact J2000 evidence: {} bodies at JD {:.1}",
                evidence.len(),
                evidence[0].epoch.julian_day.days()
            )?;
            for sample in evidence {
                writeln!(
                    f,
                    "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                    sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                )?;
            }
        }
        writeln!(
            f,
            "  note: comparison reports stay on the planetary subset while the JPL snapshot preserves selected asteroid coverage."
        )?;
    }
    Ok(())
}

pub(crate) fn regression_finding(
    sample: &ComparisonSample,
    backend_family: &BackendFamily,
) -> Option<RegressionFinding> {
    let tolerance = comparison_tolerance_for_body(&sample.body, backend_family);
    let mut notes = Vec::new();
    if sample.longitude_delta_deg >= tolerance.max_longitude_delta_deg {
        notes.push(format!(
            "longitude delta exceeds {:.1}°",
            tolerance.max_longitude_delta_deg
        ));
    }
    if sample.latitude_delta_deg >= tolerance.max_latitude_delta_deg {
        notes.push(format!(
            "latitude delta exceeds {:.2}°",
            tolerance.max_latitude_delta_deg
        ));
    }
    if sample
        .distance_delta_au
        .is_some_and(|value| value >= tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY))
    {
        notes.push(format!(
            "distance delta exceeds {:.3} AU",
            tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY)
        ));
    }

    if notes.is_empty() {
        return None;
    }

    Some(RegressionFinding {
        body: sample.body.clone(),
        longitude_delta_deg: sample.longitude_delta_deg,
        latitude_delta_deg: sample.latitude_delta_deg,
        distance_delta_au: sample.distance_delta_au,
        note: notes.join(", "),
    })
}

pub(crate) const JPL_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
];
pub(crate) const JPL_REQUIRED_DATA_FILES: &[&str] = &[
    "crates/pleiades-jpl/data/reference_snapshot.csv",
    "crates/pleiades-jpl/data/j2000_snapshot.csv",
];
pub(crate) const VSOP87_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
pub(crate) const ELP_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
pub(crate) const PACKAGED_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];
pub(crate) const COMPOSITE_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];

pub(crate) fn implemented_backend_catalog() -> Vec<BackendMatrixEntry> {
    vec![
        BackendMatrixEntry {
            label: "JPL snapshot reference backend",
            metadata: default_reference_backend().metadata(),
            implementation_status: BackendImplementationStatus::FixtureReference,
            status_note: "checked-in public-input derivative fixture with exact lookup and cubic interpolation on four-sample windows when available, with quadratic and linear fallbacks for sparser bodies; reference corpus now spans 357 rows across 16 bodies and 31 epochs with expanded bridge and boundary coverage, while the broader production reader remains planned",
            expected_error_kinds: JPL_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
        BackendMatrixEntry {
            label: "VSOP87 planetary backend",
            metadata: Vsop87Backend::new().metadata(),
            implementation_status: BackendImplementationStatus::PartialSourceBacked,
            status_note: "Sun through Neptune now use generated binary VSOP87B source tables derived from the vendored full-file inputs, and Pluto remains the current approximate mean-element fallback special case until a Pluto-specific source path is selected",
            expected_error_kinds: VSOP87_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "ELP lunar backend (Moon and lunar nodes)",
            metadata: ElpBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
            status_note: "compact lunar and lunar-point formulas provide the current deterministic baseline while documented production lunar-theory ingestion remains open",
            expected_error_kinds: ELP_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Packaged data backend",
            metadata: PackagedDataBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::DraftArtifact,
            status_note: "sample packaged artifact exercises lookup and profile plumbing; generated 1500-2500 production artifacts are Phase 2 work",
            expected_error_kinds: PACKAGED_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Composite routed backend",
            metadata: default_candidate_backend().metadata(),
            implementation_status: BackendImplementationStatus::RoutingFacade,
            status_note: "routes current planetary and lunar implementations for chart-facing validation without increasing underlying backend accuracy claims",
            expected_error_kinds: COMPOSITE_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
    ]
}

pub(crate) struct BackendMatrixEntry {
    pub(crate) label: &'static str,
    pub(crate) metadata: BackendMetadata,
    pub(crate) implementation_status: BackendImplementationStatus,
    pub(crate) status_note: &'static str,
    pub(crate) expected_error_kinds: &'static [EphemerisErrorKind],
    pub(crate) required_data_files: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BackendImplementationStatus {
    FixtureReference,
    PartialSourceBacked,
    PreliminaryAlgorithm,
    DraftArtifact,
    RoutingFacade,
}

impl BackendImplementationStatus {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::FixtureReference => "fixture-reference",
            Self::PartialSourceBacked => "partial-source-backed",
            Self::PreliminaryAlgorithm => "preliminary-algorithm",
            Self::DraftArtifact => "draft-artifact",
            Self::RoutingFacade => "routing-facade",
        }
    }
}

pub(crate) fn write_backend_catalog(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    catalog: &[BackendMatrixEntry],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for entry in catalog {
        writeln!(f, "{}", entry.label)?;
        write_backend_catalog_entry(f, entry)?;
        writeln!(f)?;
    }
    Ok(())
}

pub(crate) fn format_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(|body| body.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn selected_asteroid_coverage(bodies: &[CelestialBody]) -> Option<Vec<CelestialBody>> {
    let asteroids = bodies
        .iter()
        .filter(|body| is_selected_asteroid(body))
        .cloned()
        .collect::<Vec<_>>();

    if asteroids.is_empty() {
        None
    } else {
        Some(asteroids)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectedAsteroidCoverageSummary {
    pub(crate) body_count: usize,
    pub(crate) bodies: Vec<CelestialBody>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SelectedAsteroidCoverageSummaryValidationError {
    MissingBodies,
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    UnsupportedBody {
        index: usize,
        body: String,
    },
}

impl fmt::Display for SelectedAsteroidCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBodies => f.write_str("missing bodies"),
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match body list length {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(f, "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"),
            Self::UnsupportedBody { index, body } => write!(f, "body '{body}' at index {index} is not a selected asteroid"),
        }
    }
}

impl std::error::Error for SelectedAsteroidCoverageSummaryValidationError {}

impl SelectedAsteroidCoverageSummary {
    pub(crate) fn summary_line(&self) -> String {
        format!(
            "selected asteroid coverage: {} bodies ({})",
            self.body_count,
            format_bodies(&self.bodies)
        )
    }

    pub(crate) fn validate(&self) -> Result<(), SelectedAsteroidCoverageSummaryValidationError> {
        if self.body_count == 0 || self.bodies.is_empty() {
            return Err(SelectedAsteroidCoverageSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                SelectedAsteroidCoverageSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }
        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    SelectedAsteroidCoverageSummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .expect("duplicate body should have a first index"),
                        second_index: index,
                        body: body.to_string(),
                    },
                );
            }
            if !is_selected_asteroid(body) {
                return Err(
                    SelectedAsteroidCoverageSummaryValidationError::UnsupportedBody {
                        index,
                        body: body.to_string(),
                    },
                );
            }
        }

        Ok(())
    }

    pub(crate) fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

pub(crate) fn selected_asteroid_coverage_summary(
    bodies: &[CelestialBody],
) -> Option<SelectedAsteroidCoverageSummary> {
    selected_asteroid_coverage(bodies).map(|bodies| SelectedAsteroidCoverageSummary {
        body_count: bodies.len(),
        bodies,
    })
}

pub(crate) fn selected_asteroid_coverage_summary_for_report(bodies: &[CelestialBody]) -> String {
    match selected_asteroid_coverage_summary(bodies) {
        Some(summary) => summary
            .validated_summary_line()
            .unwrap_or_else(|error| format!("selected asteroid coverage: unavailable ({error})")),
        None => "selected asteroid coverage: unavailable".to_string(),
    }
}

pub(crate) fn is_selected_asteroid(body: &CelestialBody) -> bool {
    match body {
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => true,
        CelestialBody::Custom(custom) => custom.catalog == "asteroid",
        _ => false,
    }
}

pub(crate) fn format_frames(frames: &[CoordinateFrame]) -> String {
    frames
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_time_scales(scales: &[TimeScale]) -> String {
    scales
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_capabilities(capabilities: &BackendCapabilities) -> String {
    match capabilities.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("unavailable ({error})"),
    }
}

pub(crate) fn format_error_kinds(kinds: &[EphemerisErrorKind]) -> String {
    kinds
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_data_files(files: &[&str]) -> String {
    files.join("; ")
}

pub(crate) fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

pub(crate) fn format_instant_list(instants: &[Instant]) -> String {
    if instants.is_empty() {
        return "none".to_string();
    }

    instants
        .iter()
        .copied()
        .map(format_instant)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_ns(value: f64) -> String {
    format!("{value:.2}")
}

pub(crate) fn format_duration(duration: std::time::Duration) -> String {
    format!("{:.6}s", duration.as_secs_f64())
}
