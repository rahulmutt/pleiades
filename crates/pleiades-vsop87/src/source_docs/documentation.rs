use pleiades_types::CelestialBody;
use std::collections::BTreeSet;
use std::fmt;

use crate::profiles::{
    body_catalog_entries, fallback_body_profiles, join_display, source_backed_body_order,
    source_backed_body_profiles, Vsop87BodySourceKind,
};

use super::spec::{source_specifications, Vsop87SourceSpecification};

/// Summary metrics for the current VSOP87 source-documentation catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationSummary {
    /// Number of source specifications described by the catalog.
    pub source_specification_count: usize,
    /// Number of source-backed body profiles described by the catalog.
    pub source_backed_profile_count: usize,
    /// Bodies that still use a source-backed planetary path rather than the fallback mean-element path.
    pub source_backed_bodies: Vec<CelestialBody>,
    /// Public source files currently represented by the catalog.
    pub source_files: Vec<&'static str>,
    /// Bodies currently served by generated-binary VSOP87B tables.
    pub generated_binary_bodies: Vec<CelestialBody>,
    /// Bodies currently served by vendored full-file source paths.
    pub vendored_full_file_bodies: Vec<CelestialBody>,
    /// Bodies currently served by truncated source slices.
    pub truncated_bodies: Vec<CelestialBody>,
    /// Number of vendored full-file body profiles.
    pub vendored_full_file_profile_count: usize,
    /// Number of generated-binary body profiles.
    pub generated_binary_profile_count: usize,
    /// Number of truncated-slice body profiles.
    pub truncated_profile_count: usize,
    /// Number of approximate fallback mean-element body profiles.
    pub fallback_profile_count: usize,
    /// Bodies that still use the approximate fallback mean-element path.
    pub fallback_bodies: Vec<CelestialBody>,
    /// Unique date-range notes carried by the source specifications.
    pub date_ranges: Vec<&'static str>,
}

impl Vsop87SourceDocumentationSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_documentation_summary(self)
    }

    /// Returns the rendered summary line after validating the cached catalog snapshot.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceDocumentationSummaryValidationError> {
        self.validate()?;
        source_documentation_health_summary()
            .validate()
            .map_err(
                |_| Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_documentation_health",
                },
            )?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Vsop87SourceDocumentationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error returned when the VSOP87 source-documentation summary drifts from the current
/// catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceDocumentationSummaryValidationError {
    /// A rendered summary field no longer matches the current source-documentation catalog.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the catalog.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87SourceDocumentationSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 source documentation summary field `{field}` is out of sync with the current source catalog"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceDocumentationSummaryValidationError {}

impl Vsop87SourceDocumentationSummary {
    /// Returns `Ok(())` when the summary still matches the current source-documentation catalog.
    pub fn validate(&self) -> Result<(), Vsop87SourceDocumentationSummaryValidationError> {
        let source_specs = source_specifications();
        let source_backed_profiles = source_backed_body_profiles();
        let fallback_profiles = fallback_body_profiles();
        let expected_source_files = source_specs
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>();
        let expected_date_ranges = {
            let mut date_ranges = source_specs
                .iter()
                .map(|spec| spec.date_range)
                .collect::<Vec<_>>();
            date_ranges.sort_unstable();
            date_ranges.dedup();
            date_ranges
        };
        let expected_generated_binary_bodies = source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();
        let expected_vendored_full_file_bodies = source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();
        let expected_truncated_bodies = source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();
        let expected_fallback_bodies = fallback_profiles
            .iter()
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();

        if self.source_specification_count != source_specs.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_specification_count",
                },
            );
        }
        if self.source_backed_profile_count != source_backed_profiles.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_backed_profile_count",
                },
            );
        }
        if self.source_backed_bodies != source_backed_body_order() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_backed_bodies",
                },
            );
        }
        if self.source_files != expected_source_files {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_files",
                },
            );
        }
        if self.generated_binary_bodies != expected_generated_binary_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "generated_binary_bodies",
                },
            );
        }
        if self.vendored_full_file_bodies != expected_vendored_full_file_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "vendored_full_file_bodies",
                },
            );
        }
        if self.truncated_bodies != expected_truncated_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "truncated_bodies",
                },
            );
        }
        if self.vendored_full_file_profile_count != expected_vendored_full_file_bodies.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "vendored_full_file_profile_count",
                },
            );
        }
        if self.generated_binary_profile_count != expected_generated_binary_bodies.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "generated_binary_profile_count",
                },
            );
        }
        if self.truncated_profile_count != expected_truncated_bodies.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "truncated_profile_count",
                },
            );
        }
        if self.fallback_profile_count != fallback_profiles.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "fallback_profile_count",
                },
            );
        }
        if self.fallback_bodies != expected_fallback_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "fallback_bodies",
                },
            );
        }
        if self.date_ranges != expected_date_ranges {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "date_ranges",
                },
            );
        }

        Ok(())
    }
}

/// Structured issue labels used by the VSOP87 source-documentation health check.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Vsop87SourceDocumentationHealthIssue {
    /// The number of source specifications does not match the number of public source files.
    SourceSpecificationFileCountMismatch,
    /// The documented source file order does not match the release-facing catalog order.
    SourceFileOrderMismatch,
    /// The source-backed body order does not match the current generated/vendored/truncated partition order.
    SourceBackedBodyOrderMismatch,
    /// The source-backed profile partition counts do not add up to the total source-backed profile count.
    SourceBackedProfilePartitionMismatch,
    /// The source-backed body list contains a duplicate body entry.
    SourceBackedBodyDuplicate,
    /// The fallback body list contains a duplicate body entry.
    FallbackBodyDuplicate,
    /// A source-backed body also appears in the fallback body list.
    SourceBackedFallbackBodyOverlap,
    /// The source-backed and fallback body profiles do not cover the full body catalog.
    BodyProfileCoverageMismatch,
    /// The source specification catalog count does not match the parsed source specification list.
    SourceSpecificationCatalogCountMismatch,
    /// The documented variant, coordinate family, frame, units, reduction, transform note, truncation policy, or date range drifted.
    DocumentedFieldMismatch,
}

impl Vsop87SourceDocumentationHealthIssue {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceSpecificationFileCountMismatch => {
                "source specification/file count mismatch"
            }
            Self::SourceFileOrderMismatch => "source file order mismatch",
            Self::SourceBackedBodyOrderMismatch => "source-backed body order mismatch",
            Self::SourceBackedProfilePartitionMismatch => {
                "source-backed profile partition mismatch"
            }
            Self::SourceBackedBodyDuplicate => "source-backed body duplicate",
            Self::FallbackBodyDuplicate => "fallback body duplicate",
            Self::SourceBackedFallbackBodyOverlap => "source-backed/fallback body overlap",
            Self::BodyProfileCoverageMismatch => "body profile coverage mismatch",
            Self::SourceSpecificationCatalogCountMismatch => {
                "source specification catalog count mismatch"
            }
            Self::DocumentedFieldMismatch => "documented field mismatch",
        }
    }
}

impl fmt::Display for Vsop87SourceDocumentationHealthIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Validation error returned when the VSOP87 source-documentation health summary
/// reports an inconsistent catalog state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationHealthError {
    summary: Vsop87SourceDocumentationHealthSummary,
}

impl Vsop87SourceDocumentationHealthError {
    /// Returns the inconsistent summary that triggered the validation failure.
    pub fn summary(&self) -> &Vsop87SourceDocumentationHealthSummary {
        &self.summary
    }

    /// Returns the release-facing summary line for the inconsistent catalog state.
    pub fn summary_line(&self) -> String {
        self.summary.summary_line()
    }
}

impl fmt::Display for Vsop87SourceDocumentationHealthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for Vsop87SourceDocumentationHealthError {}

/// Consistency check for the current VSOP87 source-documentation catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationHealthSummary {
    /// Whether the catalog counts line up with the internal body catalog.
    pub consistent: bool,
    /// Whether the documented source metadata stays aligned with the current
    /// VSOP87B policy for variant, frame, units, truncation, and date range.
    pub documentation_consistent: bool,
    /// Structured labels describing any catalog inconsistencies.
    pub issues: Vec<Vsop87SourceDocumentationHealthIssue>,
    /// Number of source specifications described by the catalog.
    pub source_specification_count: usize,
    /// Number of public source files represented by the catalog.
    pub source_file_count: usize,
    /// Public source files represented by the catalog in release-facing order.
    pub source_files: Vec<&'static str>,
    /// Number of source-backed body profiles described by the catalog.
    pub source_backed_profile_count: usize,
    /// Bodies that still use a source-backed planetary path rather than the fallback mean-element path.
    pub source_backed_bodies: Vec<CelestialBody>,
    /// Bodies in the current source-backed partition order.
    pub source_backed_partition_bodies: Vec<CelestialBody>,
    /// Bodies currently served by generated-binary VSOP87B tables.
    pub generated_binary_bodies: Vec<CelestialBody>,
    /// Bodies currently served by vendored full-file source paths.
    pub vendored_full_file_bodies: Vec<CelestialBody>,
    /// Bodies currently served by truncated source slices.
    pub truncated_bodies: Vec<CelestialBody>,
    /// Total number of body profiles in the internal catalog.
    pub body_profile_count: usize,
    /// Number of generated-binary body profiles.
    pub generated_binary_profile_count: usize,
    /// Number of vendored full-file body profiles.
    pub vendored_full_file_profile_count: usize,
    /// Number of truncated-slice body profiles.
    pub truncated_profile_count: usize,
    /// Number of approximate fallback mean-element body profiles.
    pub fallback_profile_count: usize,
    /// Bodies that still use the approximate fallback mean-element path.
    pub fallback_bodies: Vec<CelestialBody>,
}

impl Vsop87SourceDocumentationHealthSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_documentation_health_summary(self)
    }

    /// Validates that the summary represents a healthy, internally consistent
    /// VSOP87 source-documentation catalog.
    pub fn validate(&self) -> Result<(), Box<Vsop87SourceDocumentationHealthError>> {
        let source_backed_profile_partition_count = self.generated_binary_profile_count
            + self.vendored_full_file_profile_count
            + self.truncated_profile_count;
        let source_backed_list_count = self.generated_binary_bodies.len()
            + self.vendored_full_file_bodies.len()
            + self.truncated_bodies.len();
        let body_profile_list_count = source_backed_list_count + self.fallback_bodies.len();
        let structural_consistency = self.source_file_count == self.source_files.len()
            && self.source_specification_count == self.source_file_count
            && self.source_backed_profile_count == self.source_backed_bodies.len()
            && self.source_backed_profile_count == self.source_backed_partition_bodies.len()
            && self.source_backed_profile_count == source_backed_profile_partition_count
            && self.source_backed_profile_count == source_backed_list_count
            && self.body_profile_count
                == self.source_backed_profile_count + self.fallback_profile_count
            && self.body_profile_count == body_profile_list_count
            && self.generated_binary_profile_count == self.generated_binary_bodies.len()
            && self.vendored_full_file_profile_count == self.vendored_full_file_bodies.len()
            && self.truncated_profile_count == self.truncated_bodies.len()
            && self.fallback_profile_count == self.fallback_bodies.len()
            && self.source_backed_bodies == self.source_backed_partition_bodies
            && body_labels_are_unique(&self.source_backed_bodies)
            && body_labels_are_unique(&self.fallback_bodies)
            && body_lists_are_disjoint(&self.source_backed_bodies, &self.fallback_bodies);

        if self.consistent
            && self.documentation_consistent
            && self.issues.is_empty()
            && structural_consistency
        {
            Ok(())
        } else {
            Err(Box::new(Vsop87SourceDocumentationHealthError {
                summary: self.clone(),
            }))
        }
    }
}

impl fmt::Display for Vsop87SourceDocumentationHealthSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a summary of the current VSOP87 source-documentation catalog.
pub fn source_documentation_summary() -> Vsop87SourceDocumentationSummary {
    let source_specs = source_specifications();
    let source_backed_profiles = source_backed_body_profiles();
    let fallback_profiles = fallback_body_profiles();

    let fallback_bodies = fallback_profiles
        .iter()
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();

    let mut date_ranges = source_specs
        .iter()
        .map(|spec| spec.date_range)
        .collect::<Vec<_>>();
    date_ranges.sort_unstable();
    date_ranges.dedup();

    let source_backed_bodies = source_backed_body_order();
    let generated_binary_bodies = source_backed_profiles
        .iter()
        .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();
    let vendored_full_file_bodies = source_backed_profiles
        .iter()
        .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();
    let truncated_bodies = source_backed_profiles
        .iter()
        .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();
    let source_files = source_specs
        .iter()
        .map(|spec| spec.source_file)
        .collect::<Vec<_>>();

    Vsop87SourceDocumentationSummary {
        source_specification_count: source_specs.len(),
        source_backed_profile_count: source_backed_profiles.len(),
        source_backed_bodies,
        source_files,
        generated_binary_bodies,
        vendored_full_file_bodies,
        truncated_bodies,
        vendored_full_file_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
            .count(),
        generated_binary_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .count(),
        truncated_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .count(),
        fallback_profile_count: fallback_profiles.len(),
        fallback_bodies,
        date_ranges,
    }
}

pub(crate) fn format_celestial_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Formats the current VSOP87 source-documentation catalog for reporting.
pub fn format_source_documentation_summary(summary: &Vsop87SourceDocumentationSummary) -> String {
    let source_backed_bodies = if summary.source_backed_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.source_backed_bodies)
    };
    let fallback_bodies = if summary.fallback_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.fallback_bodies)
    };
    let source_files = if summary.source_files.is_empty() {
        "none".to_string()
    } else {
        summary.source_files.join(", ")
    };
    let date_ranges = if summary.date_ranges.is_empty() {
        "none".to_string()
    } else {
        summary.date_ranges.join("; ")
    };
    let generated_binary_bodies = if summary.generated_binary_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.generated_binary_bodies)
    };
    let vendored_full_file_bodies = if summary.vendored_full_file_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.vendored_full_file_bodies)
    };
    let truncated_bodies = if summary.truncated_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.truncated_bodies)
    };
    let fallback_profile_label = if summary.fallback_profile_count == 1 {
        "approximate fallback mean-element body profile"
    } else {
        "approximate fallback mean-element body profiles"
    };

    format!(
        "VSOP87 source documentation: {} source specs, {} source-backed body profiles, {} {} ({}); source-backed bodies: {}; source files: {}; source-backed breakdown: {} generated binary bodies ({}), {} vendored full-file bodies ({}), {} truncated slice bodies ({}); date ranges: {}",
        summary.source_specification_count,
        summary.source_backed_profile_count,
        summary.fallback_profile_count,
        fallback_profile_label,
        fallback_bodies,
        source_backed_bodies,
        source_files,
        summary.generated_binary_profile_count,
        generated_binary_bodies,
        summary.vendored_full_file_profile_count,
        vendored_full_file_bodies,
        summary.truncated_profile_count,
        truncated_bodies,
        date_ranges,
    )
}

/// Returns the release-facing summary string for the current VSOP87 source-documentation catalog.
///
/// The compact provenance line is rendered only after the catalog-health gate
/// confirms that the source/file/body partitioning still matches the current
/// canonical VSOP87 inputs.
pub fn source_documentation_summary_for_report() -> String {
    format_validated_source_documentation_summary_for_report(&source_documentation_summary())
}

/// Returns a consistency check for the current VSOP87 source-documentation catalog.
fn source_documentation_fields_are_consistent(source_specs: &[Vsop87SourceSpecification]) -> bool {
    source_specs.iter().all(|spec| spec.validate().is_ok())
}

/// Returns the health summary for the current VSOP87 source-documentation
/// catalog, cross-checking the documented counts and metadata against the
/// internal body catalog and flagging any drift as issues.
pub fn source_documentation_health_summary() -> Vsop87SourceDocumentationHealthSummary {
    let summary = source_documentation_summary();
    let source_specs = source_specifications();
    let body_profile_count = body_catalog_entries().len();
    let source_file_count = summary.source_files.len();
    let issues = source_documentation_health_issues(
        &summary,
        &source_specs,
        body_profile_count,
        source_file_count,
    );

    let consistent = issues.is_empty();
    let documentation_consistent = summary.source_specification_count == source_specs.len()
        && source_documentation_fields_are_consistent(&source_specs);

    let source_backed_partition_bodies = source_documentation_partition_bodies(&summary);

    Vsop87SourceDocumentationHealthSummary {
        consistent,
        documentation_consistent,
        issues,
        source_specification_count: summary.source_specification_count,
        source_file_count,
        source_files: summary.source_files,
        source_backed_profile_count: summary.source_backed_profile_count,
        source_backed_bodies: summary.source_backed_bodies,
        source_backed_partition_bodies,
        generated_binary_bodies: summary.generated_binary_bodies,
        vendored_full_file_bodies: summary.vendored_full_file_bodies,
        truncated_bodies: summary.truncated_bodies,
        body_profile_count,
        generated_binary_profile_count: summary.generated_binary_profile_count,
        vendored_full_file_profile_count: summary.vendored_full_file_profile_count,
        truncated_profile_count: summary.truncated_profile_count,
        fallback_profile_count: summary.fallback_profile_count,
        fallback_bodies: summary.fallback_bodies,
    }
}

/// Formats the current VSOP87 source-documentation health check for reporting.
pub fn format_source_documentation_health_summary(
    summary: &Vsop87SourceDocumentationHealthSummary,
) -> String {
    let issues = if summary.issues.is_empty() {
        String::new()
    } else {
        format!("; issues: {}", format_issue_labels(&summary.issues))
    };

    format!(
        "VSOP87 source documentation health: {} ({} source specs, {} source files, {} source-backed profiles, {} body profiles; {} generated binary profiles ({}), {} vendored full-file profiles ({}), {} truncated profiles ({}), {} approximate fallback profiles ({}); source files: {}; source-backed order: {}; source-backed partition order: {}; fallback order: {}; documented fields: {}){}",
        if summary.consistent { "ok" } else { "needs attention" },
        summary.source_specification_count,
        summary.source_file_count,
        summary.source_backed_profile_count,
        summary.body_profile_count,
        summary.generated_binary_profile_count,
        format_bodies(&summary.generated_binary_bodies),
        summary.vendored_full_file_profile_count,
        format_bodies(&summary.vendored_full_file_bodies),
        summary.truncated_profile_count,
        format_bodies(&summary.truncated_bodies),
        summary.fallback_profile_count,
        format_bodies(&summary.fallback_bodies),
        format_source_files(&summary.source_files),
        format_bodies(&summary.source_backed_bodies),
        format_bodies(&summary.source_backed_partition_bodies),
        format_bodies(&summary.fallback_bodies),
        if summary.documentation_consistent {
            "variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range"
        } else {
            "needs attention"
        },
        issues,
    )
}

/// Returns the source-backed partition order used by the VSOP87 source
/// documentation health check.
///
/// The generated-binary, vendored full-file, and truncated slices are kept in
/// this order so regeneration tooling and release reports can reuse the same
/// backend-owned partitioning without reconstructing it locally.
pub fn source_documentation_partition_bodies(
    summary: &Vsop87SourceDocumentationSummary,
) -> Vec<CelestialBody> {
    summary
        .generated_binary_bodies
        .iter()
        .chain(summary.vendored_full_file_bodies.iter())
        .chain(summary.truncated_bodies.iter())
        .cloned()
        .collect()
}

pub(crate) fn source_documentation_health_issues(
    summary: &Vsop87SourceDocumentationSummary,
    source_specs: &[Vsop87SourceSpecification],
    body_profile_count: usize,
    source_file_count: usize,
) -> Vec<Vsop87SourceDocumentationHealthIssue> {
    let expected_source_files = source_specs
        .iter()
        .map(|spec| spec.source_file)
        .collect::<Vec<_>>();
    let expected_source_backed_bodies = source_documentation_partition_bodies(summary);
    let mut issues = Vec::new();

    if summary.source_specification_count != source_file_count {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceSpecificationFileCountMismatch);
    }
    if summary.source_files != expected_source_files {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceFileOrderMismatch);
    }
    if summary.source_backed_bodies != expected_source_backed_bodies {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedBodyOrderMismatch);
    }
    if !body_labels_are_unique(&summary.source_backed_bodies) {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedBodyDuplicate);
    }
    if !body_labels_are_unique(&summary.fallback_bodies) {
        issues.push(Vsop87SourceDocumentationHealthIssue::FallbackBodyDuplicate);
    }
    if !body_lists_are_disjoint(&summary.source_backed_bodies, &summary.fallback_bodies) {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedFallbackBodyOverlap);
    }
    if summary.source_backed_profile_count
        != summary.generated_binary_profile_count
            + summary.vendored_full_file_profile_count
            + summary.truncated_profile_count
    {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedProfilePartitionMismatch);
    }
    if summary.source_backed_profile_count + summary.fallback_profile_count != body_profile_count {
        issues.push(Vsop87SourceDocumentationHealthIssue::BodyProfileCoverageMismatch);
    }
    if summary.source_specification_count != source_specs.len() {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceSpecificationCatalogCountMismatch);
    }
    if !source_documentation_fields_are_consistent(source_specs) {
        issues.push(Vsop87SourceDocumentationHealthIssue::DocumentedFieldMismatch);
    }

    issues
}

fn format_bodies(bodies: &[CelestialBody]) -> String {
    if bodies.is_empty() {
        "none".to_string()
    } else {
        bodies
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn format_issue_labels<T: fmt::Display>(issues: &[T]) -> String {
    if issues.is_empty() {
        "none".to_string()
    } else {
        join_display(issues)
    }
}

pub(crate) fn body_labels_are_unique(bodies: &[CelestialBody]) -> bool {
    let mut seen = BTreeSet::new();

    for body in bodies {
        if !seen.insert(body.to_string()) {
            return false;
        }
    }

    true
}

fn body_lists_are_disjoint(left: &[CelestialBody], right: &[CelestialBody]) -> bool {
    let mut seen = BTreeSet::new();

    for body in left {
        seen.insert(body.to_string());
    }

    right.iter().all(|body| !seen.contains(&body.to_string()))
}

fn format_source_files(source_files: &[&'static str]) -> String {
    if source_files.is_empty() {
        "none".to_string()
    } else {
        source_files.join(", ")
    }
}

pub(crate) fn format_validated_source_documentation_health_summary_for_report(
    summary: &Vsop87SourceDocumentationHealthSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source documentation health: unavailable ({error})"),
    }
}

pub(crate) fn format_validated_source_documentation_summary_for_report(
    summary: &Vsop87SourceDocumentationSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("VSOP87 source documentation: unavailable ({error})"),
    }
}

/// Returns the release-facing source-documentation health string.
pub fn source_documentation_health_summary_for_report() -> String {
    format_validated_source_documentation_health_summary_for_report(
        &source_documentation_health_summary(),
    )
}
