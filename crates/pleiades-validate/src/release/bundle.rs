//! Release bundle type, rendering, and manifest parsing.

use std::fmt;
use std::path::Path;

use super::bundle_verify::*;
use super::bundle_verify_helpers::*;
use crate::*;

/// A generated release bundle containing the compatibility profile, release-profile
/// identifiers, release notes, release checklist, backend matrix, API posture,
/// API stability summary, comparison-corpus summary, comparison-envelope summary,
/// comparison-corpus release-guard summary, reference-holdout overlap summary, catalog inventory summary, ayanamsa provenance summary, validation report summary,
/// artifact summary, packaged-artifact speed policy summary, packaged-artifact generation manifest, packaged-artifact generation manifest summary, packaged-artifact generation manifest checksum summary, packaged-artifact generation manifest checksum sidecar, benchmark-corpus summary,
/// benchmark report, validation report, and manifest.
#[derive(Clone, Debug)]
pub struct ReleaseBundle {
    /// Source revision recorded when the bundle was generated.
    pub source_revision: String,
    /// Workspace status recorded when the bundle was generated.
    pub workspace_status: String,
    /// Rust compiler version recorded when the bundle was generated.
    pub rustc_version: String,
    /// Cargo version recorded when the bundle was generated.
    pub cargo_version: String,
    /// Output directory chosen by the caller.
    pub output_dir: PathBuf,
    /// Path to the generated compatibility profile file.
    pub compatibility_profile_path: PathBuf,
    /// Path to the generated compatibility-profile summary file.
    pub compatibility_profile_summary_path: PathBuf,
    /// Path to the generated release notes file.
    pub release_notes_path: PathBuf,
    /// Path to the generated release notes summary file.
    pub release_notes_summary_path: PathBuf,
    /// Path to the generated release summary file.
    pub release_summary_path: PathBuf,
    /// Path to the generated release-profile identifiers file.
    pub release_profile_identifiers_path: PathBuf,
    /// Path to the generated release-profile identifiers summary file.
    pub release_profile_identifiers_summary_path: PathBuf,
    /// Path to the generated release-house-system canonical-names summary file.
    pub release_house_system_canonical_names_summary_path: PathBuf,
    /// Path to the generated release-ayanamsa canonical-names summary file.
    pub release_ayanamsa_canonical_names_summary_path: PathBuf,
    /// Path to the generated release-house-validation summary file.
    pub release_house_validation_summary_path: PathBuf,
    /// Path to the generated house-code-aliases summary file.
    pub house_code_aliases_summary_path: PathBuf,
    /// Path to the generated house-formula-families summary file.
    pub house_formula_families_summary_path: PathBuf,
    /// Path to the generated house-latitude-sensitive summary file.
    pub house_latitude_sensitive_summary_path: PathBuf,
    /// Path to the generated release checklist file.
    pub release_checklist_path: PathBuf,
    /// Path to the generated release checklist summary file.
    pub release_checklist_summary_path: PathBuf,
    /// Path to the generated backend capability matrix file.
    pub backend_matrix_path: PathBuf,
    /// Path to the generated backend capability matrix summary file.
    pub backend_matrix_summary_path: PathBuf,
    /// Path to the generated API stability posture file.
    pub api_stability_path: PathBuf,
    /// Path to the generated API stability summary file.
    pub api_stability_summary_path: PathBuf,
    /// Path to the generated comparison-envelope summary file.
    pub comparison_envelope_summary_path: PathBuf,
    /// Path to the generated comparison-body-class tolerance summary file.
    pub comparison_body_class_tolerance_summary_path: PathBuf,
    /// Path to the generated comparison-body-class error-envelope summary file.
    pub comparison_body_class_error_envelope_summary_path: PathBuf,
    /// Path to the generated comparison-corpus release-guard summary file.
    pub comparison_corpus_release_guard_summary_path: PathBuf,
    /// Path to the generated compatibility catalog inventory summary file.
    pub catalog_inventory_summary_path: PathBuf,
    /// Path to the generated validation report summary file.
    pub validation_report_summary_path: PathBuf,
    /// Path to the generated workspace-provenance summary file.
    pub workspace_provenance_summary_path: PathBuf,
    /// Path to the generated workspace-audit summary file.
    pub workspace_audit_summary_path: PathBuf,
    /// Path to the generated native-dependency audit summary file.
    pub native_dependency_audit_summary_path: PathBuf,
    /// Path to the generated artifact summary file.
    pub artifact_summary_path: PathBuf,
    /// Path to the generated packaged-artifact binary.
    pub packaged_artifact_path: PathBuf,
    /// Path to the generated packaged-artifact checksum sidecar.
    pub packaged_artifact_checksum_path: PathBuf,
    /// Path to the generated packaged-artifact profile coverage summary file.
    pub packaged_artifact_profile_coverage_summary_path: PathBuf,
    /// Path to the generated interpolation-quality sample request corpus summary file.
    pub interpolation_quality_request_corpus_summary_path: PathBuf,
    /// Path to the generated packaged-artifact storage summary file.
    pub packaged_artifact_storage_summary_path: PathBuf,
    /// Path to the generated packaged-artifact generation manifest file.
    pub packaged_artifact_generation_manifest_path: PathBuf,
    /// Path to the generated packaged-artifact generation manifest summary file.
    pub packaged_artifact_generation_manifest_summary_path: PathBuf,
    /// Path to the generated packaged-artifact generation manifest checksum summary file.
    pub packaged_artifact_generation_manifest_checksum_summary_path: PathBuf,
    /// Path to the generated packaged-artifact generation manifest checksum sidecar file.
    pub packaged_artifact_generation_manifest_checksum_path: PathBuf,
    /// Path to the generated benchmark report file.
    pub benchmark_report_path: PathBuf,
    /// Path to the generated validation report file.
    pub validation_report_path: PathBuf,
    /// Path to the generated bundle manifest.
    pub manifest_path: PathBuf,
    /// Path to the generated manifest checksum sidecar.
    pub manifest_checksum_path: PathBuf,
    /// Number of bytes written for the compatibility profile.
    pub compatibility_profile_bytes: usize,
    /// Number of bytes written for the compatibility-profile summary.
    pub compatibility_profile_summary_bytes: usize,
    /// Number of bytes written for the release notes.
    pub release_notes_bytes: usize,
    /// Number of bytes written for the release notes summary.
    pub release_notes_summary_bytes: usize,
    /// Number of bytes written for the release summary.
    pub release_summary_bytes: usize,
    /// Number of bytes written for the release-profile identifiers file.
    pub release_profile_identifiers_bytes: usize,
    /// Number of bytes written for the release-profile identifiers summary.
    pub release_profile_identifiers_summary_bytes: usize,
    /// Number of bytes written for the release-house-system canonical-names summary.
    pub release_house_system_canonical_names_summary_bytes: usize,
    /// Number of bytes written for the release-ayanamsa canonical-names summary.
    pub release_ayanamsa_canonical_names_summary_bytes: usize,
    /// Number of bytes written for the release-house-validation summary.
    pub release_house_validation_summary_bytes: usize,
    /// Number of bytes written for the house-code-aliases summary.
    pub house_code_aliases_summary_bytes: usize,
    /// Number of bytes written for the house-formula-families summary.
    pub house_formula_families_summary_bytes: usize,
    /// Number of bytes written for the house-latitude-sensitive summary.
    pub house_latitude_sensitive_summary_bytes: usize,
    /// Number of bytes written for the release checklist.
    pub release_checklist_bytes: usize,
    /// Number of bytes written for the release checklist summary.
    pub release_checklist_summary_bytes: usize,
    /// Number of bytes written for the backend capability matrix.
    pub backend_matrix_bytes: usize,
    /// Number of bytes written for the backend capability matrix summary.
    pub backend_matrix_summary_bytes: usize,
    /// Number of bytes written for the API stability posture.
    pub api_stability_bytes: usize,
    /// Number of bytes written for the API stability summary.
    pub api_stability_summary_bytes: usize,
    /// Number of bytes written for the comparison-envelope summary.
    pub comparison_envelope_summary_bytes: usize,
    /// Number of bytes written for the comparison-body-class tolerance summary.
    pub comparison_body_class_tolerance_summary_bytes: usize,
    /// Number of bytes written for the comparison-body-class error-envelope summary.
    pub comparison_body_class_error_envelope_summary_bytes: usize,
    /// Number of bytes written for the comparison-corpus release-guard summary.
    pub comparison_corpus_release_guard_summary_bytes: usize,
    /// Number of bytes written for the reference-holdout overlap summary.
    pub reference_holdout_overlap_summary_bytes: usize,
    /// Number of bytes written for the compatibility catalog inventory summary.
    pub catalog_inventory_summary_bytes: usize,
    /// Number of bytes written for the validation report summary.
    pub validation_report_summary_bytes: usize,
    /// Number of bytes written for the workspace-provenance summary.
    pub workspace_provenance_summary_bytes: usize,
    /// Number of bytes written for the workspace-audit summary.
    pub workspace_audit_summary_bytes: usize,
    /// Number of bytes written for the native-dependency audit summary.
    pub native_dependency_audit_summary_bytes: usize,
    /// Number of bytes written for the artifact summary.
    pub artifact_summary_bytes: usize,
    /// Number of bytes written for the packaged-artifact profile coverage summary.
    pub packaged_artifact_profile_coverage_summary_bytes: usize,
    /// Number of bytes written for the packaged-artifact generation manifest.
    pub packaged_artifact_generation_manifest_bytes: usize,
    /// Number of bytes written for the packaged-artifact generation manifest summary.
    pub packaged_artifact_generation_manifest_summary_bytes: usize,
    /// Number of bytes written for the packaged-artifact generation manifest checksum summary.
    pub packaged_artifact_generation_manifest_checksum_summary_bytes: usize,
    /// Number of bytes written for the packaged-artifact generation manifest checksum sidecar.
    pub packaged_artifact_generation_manifest_checksum_bytes: usize,
    /// Number of bytes written for the benchmark report.
    pub benchmark_report_bytes: usize,
    /// Number of bytes written for the validation report.
    pub validation_report_bytes: usize,
    /// Number of bytes written for the manifest checksum sidecar.
    pub manifest_checksum_bytes: usize,
    /// Deterministic checksum for the compatibility profile contents.
    pub compatibility_profile_checksum: u64,
    /// Deterministic checksum for the compatibility-profile summary contents.
    pub compatibility_profile_summary_checksum: u64,
    /// Deterministic checksum for the release notes contents.
    pub release_notes_checksum: u64,
    /// Deterministic checksum for the release notes summary contents.
    pub release_notes_summary_checksum: u64,
    /// Deterministic checksum for the release summary contents.
    pub release_summary_checksum: u64,
    /// Deterministic checksum for the release-profile identifiers contents.
    pub release_profile_identifiers_checksum: u64,
    /// Deterministic checksum for the release-profile identifiers summary contents.
    pub release_profile_identifiers_summary_checksum: u64,
    /// Deterministic checksum for the release-house-system canonical-names summary contents.
    pub release_house_system_canonical_names_summary_checksum: u64,
    /// Deterministic checksum for the release-ayanamsa canonical-names summary contents.
    pub release_ayanamsa_canonical_names_summary_checksum: u64,
    /// Deterministic checksum for the release-house-validation summary contents.
    pub release_house_validation_summary_checksum: u64,
    /// Deterministic checksum for the house-code-aliases summary contents.
    pub house_code_aliases_summary_checksum: u64,
    /// Deterministic checksum for the house-formula-families summary contents.
    pub house_formula_families_summary_checksum: u64,
    /// Deterministic checksum for the house-latitude-sensitive summary contents.
    pub house_latitude_sensitive_summary_checksum: u64,
    /// Deterministic checksum for the release checklist contents.
    pub release_checklist_checksum: u64,
    /// Deterministic checksum for the release checklist summary contents.
    pub release_checklist_summary_checksum: u64,
    /// Deterministic checksum for the backend capability matrix contents.
    pub backend_matrix_checksum: u64,
    /// Deterministic checksum for the backend capability matrix summary contents.
    pub backend_matrix_summary_checksum: u64,
    /// Deterministic checksum for the API stability posture contents.
    pub api_stability_checksum: u64,
    /// Deterministic checksum for the API stability summary contents.
    pub api_stability_summary_checksum: u64,
    /// Deterministic checksum for the comparison-envelope summary contents.
    pub comparison_envelope_summary_checksum: u64,
    /// Deterministic checksum for the comparison-body-class tolerance summary contents.
    pub comparison_body_class_tolerance_summary_checksum: u64,
    /// Deterministic checksum for the comparison-body-class error-envelope summary contents.
    pub comparison_body_class_error_envelope_summary_checksum: u64,
    /// Deterministic checksum for the comparison-corpus release-guard summary contents.
    pub comparison_corpus_release_guard_summary_checksum: u64,
    /// Deterministic checksum for the reference-holdout overlap summary contents.
    pub reference_holdout_overlap_summary_checksum: u64,
    /// Deterministic checksum for the compatibility catalog inventory summary contents.
    pub catalog_inventory_summary_checksum: u64,
    /// Deterministic checksum for the validation report summary contents.
    pub validation_report_summary_checksum: u64,
    /// Deterministic checksum for the workspace-provenance summary contents.
    pub workspace_provenance_summary_checksum: u64,
    /// Deterministic checksum for the workspace-audit summary contents.
    pub workspace_audit_summary_checksum: u64,
    /// Deterministic checksum for the native-dependency audit summary contents.
    pub native_dependency_audit_summary_checksum: u64,
    /// Deterministic checksum for the artifact summary contents.
    pub artifact_summary_checksum: u64,
    /// Deterministic checksum for the packaged-artifact profile coverage summary contents.
    pub packaged_artifact_profile_coverage_summary_checksum: u64,
    /// Deterministic checksum for the packaged-artifact generation manifest contents.
    pub packaged_artifact_generation_manifest_checksum: u64,
    /// Deterministic checksum for the packaged-artifact generation manifest summary contents.
    pub packaged_artifact_generation_manifest_summary_checksum: u64,
    /// Deterministic checksum for the packaged-artifact generation manifest checksum summary contents.
    pub packaged_artifact_generation_manifest_checksum_summary_checksum: u64,
    /// Deterministic checksum for the packaged-artifact generation manifest checksum sidecar contents.
    pub packaged_artifact_generation_manifest_checksum_checksum: u64,
    /// Deterministic checksum for the benchmark report contents.
    pub benchmark_report_checksum: u64,
    /// Deterministic checksum for the validation report contents.
    pub validation_report_checksum: u64,
    /// Deterministic checksum recorded in the manifest checksum sidecar.
    pub manifest_checksum: u64,
    /// Number of validation rounds recorded in the bundle manifest.
    pub validation_rounds: usize,
}

/// Errors produced while assembling a release bundle.
#[derive(Debug)]
pub enum ReleaseBundleError {
    /// File-system failure while creating or writing the bundle.
    Io(std::io::Error),
    /// Validation failure while rendering the compatibility profile, API posture, or report.
    Validation(EphemerisError),
    /// Release-bundle verification failed after writing or reading the staged artifacts.
    Verification(String),
}

impl fmt::Display for ReleaseBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Validation(error) => write!(f, "{error}"),
            Self::Verification(message) => {
                write!(f, "release bundle verification failed: {message}")
            }
        }
    }
}

impl std::error::Error for ReleaseBundleError {}

impl From<std::io::Error> for ReleaseBundleError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<EphemerisError> for ReleaseBundleError {
    fn from(error: EphemerisError) -> Self {
        Self::Validation(error)
    }
}

impl fmt::Display for ReleaseBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Release bundle")?;
        writeln!(f, "  output directory: {}", self.output_dir.display())?;
        writeln!(
            f,
            "  compatibility profile: {}",
            self.compatibility_profile_path.display()
        )?;
        writeln!(
            f,
            "  compatibility profile summary: {}",
            self.compatibility_profile_summary_path.display()
        )?;
        writeln!(f, "  release notes: {}", self.release_notes_path.display())?;
        writeln!(
            f,
            "  release notes summary: {}",
            self.release_notes_summary_path.display()
        )?;
        writeln!(
            f,
            "  release summary: {}",
            self.release_summary_path.display()
        )?;
        writeln!(
            f,
            "  release-profile identifiers: {}",
            self.release_profile_identifiers_path.display()
        )?;
        writeln!(
            f,
            "  release-profile identifiers summary: {}",
            self.release_profile_identifiers_summary_path.display()
        )?;
        writeln!(
            f,
            "  release-house-system canonical names summary: {}",
            self.release_house_system_canonical_names_summary_path
                .display()
        )?;
        writeln!(
            f,
            "  release-ayanamsa canonical names summary: {}",
            self.release_ayanamsa_canonical_names_summary_path.display()
        )?;
        writeln!(
            f,
            "  release-house-validation summary: {}",
            self.release_house_validation_summary_path.display()
        )?;
        writeln!(
            f,
            "  house-code-aliases summary: {}",
            self.house_code_aliases_summary_path.display()
        )?;
        writeln!(
            f,
            "  house formula families summary: {}",
            self.house_formula_families_summary_path.display()
        )?;
        writeln!(
            f,
            "  house latitude-sensitive summary: {}",
            self.house_latitude_sensitive_summary_path.display()
        )?;
        writeln!(
            f,
            "  house latitude-sensitive constraints summary: {}",
            self.output_dir
                .join("house-latitude-sensitive-constraints-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  house latitude-sensitive failure-modes summary: {}",
            self.output_dir
                .join("house-latitude-sensitive-failure-modes-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  release checklist: {}",
            self.release_checklist_path.display()
        )?;
        writeln!(
            f,
            "  release checklist summary: {}",
            self.release_checklist_summary_path.display()
        )?;
        writeln!(
            f,
            "  backend matrix: {}",
            self.backend_matrix_path.display()
        )?;
        writeln!(
            f,
            "  backend matrix summary: {}",
            self.backend_matrix_summary_path.display()
        )?;
        writeln!(
            f,
            "  API stability posture: {}",
            self.api_stability_path.display()
        )?;
        writeln!(
            f,
            "  API stability summary: {}",
            self.api_stability_summary_path.display()
        )?;
        writeln!(
            f,
            "  comparison-corpus summary: {}",
            self.output_dir
                .join("comparison-corpus-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  source-corpus summary: {}",
            self.output_dir.join("source-corpus-summary.txt").display()
        )?;
        writeln!(
            f,
            "  comparison-snapshot summary: {}",
            self.output_dir
                .join("comparison-snapshot-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  comparison-snapshot source summary: {}",
            self.output_dir
                .join("comparison-snapshot-source-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  comparison-snapshot body-class coverage summary: {}",
            self.output_dir
                .join("comparison-snapshot-body-class-coverage-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  comparison-snapshot manifest summary: {}",
            self.output_dir
                .join("comparison-snapshot-manifest-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  comparison-envelope summary: {}",
            self.comparison_envelope_summary_path.display()
        )?;
        writeln!(
            f,
            "  comparison-corpus release-guard summary: {}",
            self.comparison_corpus_release_guard_summary_path.display()
        )?;
        writeln!(
            f,
            "  reference snapshot bridge day summary: {}",
            self.output_dir
                .join("reference-snapshot-bridge-day-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot 2451917 major-body boundary summary: {}",
            self.output_dir
                .join("reference-snapshot-2451917-major-body-boundary-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot 2451918 major-body boundary summary: {}",
            self.output_dir
                .join("reference-snapshot-2451918-major-body-boundary-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot 2451919 major-body boundary summary: {}",
            self.output_dir
                .join("reference-snapshot-2451919-major-body-boundary-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot 2451916 major-body dense boundary summary: {}",
            self.output_dir
                .join("reference-snapshot-2451916-major-body-dense-boundary-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot sparse boundary summary: {}",
            self.output_dir
                .join("reference-snapshot-sparse-boundary-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot source summary: {}",
            self.output_dir
                .join("reference-snapshot-source-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot source window summary: {}",
            self.output_dir
                .join("reference-snapshot-source-window-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot manifest summary: {}",
            self.output_dir
                .join("reference-snapshot-manifest-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot body-class coverage summary: {}",
            self.output_dir
                .join("reference-snapshot-body-class-coverage-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot equatorial parity summary: {}",
            self.output_dir
                .join("reference-snapshot-equatorial-parity-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference asteroid source window summary: {}",
            self.output_dir
                .join("reference-asteroid-source-window-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference asteroid equatorial evidence summary: {}",
            self.output_dir
                .join("reference-asteroid-equatorial-evidence-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  independent-holdout source window summary: {}",
            self.output_dir
                .join("independent-holdout-source-window-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  independent-holdout quarter-day boundary summary: {}",
            self.output_dir
                .join("independent-holdout-quarter-day-boundary-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  production generation boundary request corpus summary: {}",
            self.output_dir
                .join("production-generation-boundary-request-corpus-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  production generation summary: {}",
            self.output_dir
                .join("production-generation-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  production generation source summary: {}",
            self.output_dir
                .join("production-generation-source-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  production generation source window summary: {}",
            self.output_dir
                .join("production-generation-source-window-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  production generation boundary window summary: {}",
            self.output_dir
                .join("production-generation-boundary-window-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  production generation quarter-day boundary summary: {}",
            self.output_dir
                .join("production-generation-quarter-day-boundary-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  reference snapshot summary: {}",
            self.output_dir
                .join("reference-snapshot-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  catalog inventory summary: {}",
            self.catalog_inventory_summary_path.display()
        )?;
        writeln!(
            f,
            "  custom-definition ayanamsa labels summary: {}",
            self.output_dir
                .join("custom-definition-ayanamsa-labels-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  request policy summary: {}",
            self.output_dir.join("request-policy-summary.txt").display()
        )?;
        writeln!(
            f,
            "  observer policy summary: {}",
            self.output_dir
                .join("observer-policy-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  apparentness policy summary: {}",
            self.output_dir
                .join("apparentness-policy-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  request-semantics summary: {}",
            self.output_dir
                .join("request-semantics-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  unsupported-modes summary: {}",
            self.output_dir
                .join("unsupported-modes-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  time-scale policy summary: {}",
            self.output_dir
                .join("time-scale-policy-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  delta-t policy summary: {}",
            self.output_dir.join("delta-t-policy-summary.txt").display()
        )?;
        writeln!(
            f,
            "  zodiac policy summary: {}",
            self.output_dir.join("zodiac-policy-summary.txt").display()
        )?;
        writeln!(
            f,
            "  native sidereal policy summary: {}",
            self.output_dir
                .join("native-sidereal-policy-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  lunar-theory limitations summary: {}",
            self.output_dir
                .join("lunar-theory-limitations-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  lunar-theory source selection summary: {}",
            self.output_dir
                .join("lunar-theory-source-selection-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  lunar-theory source family summary: {}",
            self.output_dir
                .join("lunar-theory-source-family-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  lunar source window summary: {}",
            self.output_dir
                .join("lunar-source-window-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  lunar-theory catalog validation summary: {}",
            self.output_dir
                .join("lunar-theory-catalog-validation-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  request surface summary: {}",
            self.output_dir
                .join("request-surface-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  compatibility caveats summary: {}",
            self.output_dir
                .join("compatibility-caveats-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  validation report summary: {}",
            self.validation_report_summary_path.display()
        )?;
        writeln!(
            f,
            "  workspace provenance summary: {}",
            self.workspace_provenance_summary_path.display()
        )?;
        writeln!(
            f,
            "  workspace audit summary: {}",
            self.workspace_audit_summary_path.display()
        )?;
        writeln!(
            f,
            "  native-dependency audit summary: {}",
            self.output_dir
                .join("native-dependency-audit-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  artifact summary: {}",
            self.artifact_summary_path.display()
        )?;
        writeln!(
            f,
            "  packaged-artifact profile coverage summary: {}",
            self.packaged_artifact_profile_coverage_summary_path
                .display()
        )?;
        writeln!(
            f,
            "  packaged-artifact phase-2 corpus alignment summary: {}",
            self.output_dir
                .join("packaged-artifact-phase2-corpus-alignment-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest: {}",
            self.packaged_artifact_generation_manifest_path.display()
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest summary: {}",
            self.packaged_artifact_generation_manifest_summary_path
                .display()
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest checksum summary: {}",
            self.packaged_artifact_generation_manifest_checksum_summary_path
                .display()
        )?;
        writeln!(
            f,
            "  benchmark-corpus summary: {}",
            self.output_dir
                .join("benchmark-corpus-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  selected asteroid source request corpus summary: {}",
            self.output_dir
                .join("selected-asteroid-source-request-corpus-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  selected asteroid source request corpus equatorial summary: {}",
            self.output_dir
                .join("selected-asteroid-source-request-corpus-equatorial-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  selected asteroid source window summary: {}",
            self.output_dir
                .join("selected-asteroid-source-window-summary.txt")
                .display()
        )?;
        writeln!(
            f,
            "  interpolation-quality sample request corpus summary: {}",
            self.interpolation_quality_request_corpus_summary_path
                .display()
        )?;
        writeln!(
            f,
            "  benchmark report: {}",
            self.benchmark_report_path.display()
        )?;
        writeln!(
            f,
            "  validation report: {}",
            self.validation_report_path.display()
        )?;
        writeln!(f, "  manifest: {}", self.manifest_path.display())?;
        writeln!(
            f,
            "  manifest checksum sidecar: {}",
            self.manifest_checksum_path.display()
        )?;
        writeln!(f, "  source revision: {}", self.source_revision)?;
        writeln!(f, "  workspace status: {}", self.workspace_status)?;
        writeln!(f, "  rustc version: {}", self.rustc_version)?;
        writeln!(f, "  cargo version: {}", self.cargo_version)?;
        writeln!(f, "  validation rounds: {}", self.validation_rounds)?;
        writeln!(
            f,
            "  compatibility profile bytes: {}",
            self.compatibility_profile_bytes
        )?;
        writeln!(
            f,
            "  compatibility profile summary bytes: {}",
            self.compatibility_profile_summary_bytes
        )?;
        writeln!(f, "  release notes bytes: {}", self.release_notes_bytes)?;
        writeln!(
            f,
            "  release notes summary bytes: {}",
            self.release_notes_summary_bytes
        )?;
        writeln!(f, "  release summary bytes: {}", self.release_summary_bytes)?;
        writeln!(
            f,
            "  release-profile identifiers bytes: {}",
            self.release_profile_identifiers_bytes
        )?;
        writeln!(
            f,
            "  release-profile identifiers summary bytes: {}",
            self.release_profile_identifiers_summary_bytes
        )?;
        writeln!(
            f,
            "  release-house-system canonical names summary bytes: {}",
            self.release_house_system_canonical_names_summary_bytes
        )?;
        writeln!(
            f,
            "  release-ayanamsa canonical names summary bytes: {}",
            self.release_ayanamsa_canonical_names_summary_bytes
        )?;
        writeln!(
            f,
            "  release checklist bytes: {}",
            self.release_checklist_bytes
        )?;
        writeln!(
            f,
            "  release checklist summary bytes: {}",
            self.release_checklist_summary_bytes
        )?;
        writeln!(
            f,
            "  compatibility profile checksum: 0x{:016x}",
            self.compatibility_profile_checksum
        )?;
        writeln!(
            f,
            "  compatibility profile summary checksum: 0x{:016x}",
            self.compatibility_profile_summary_checksum
        )?;
        writeln!(
            f,
            "  release notes checksum: 0x{:016x}",
            self.release_notes_checksum
        )?;
        writeln!(
            f,
            "  release notes summary checksum: 0x{:016x}",
            self.release_notes_summary_checksum
        )?;
        writeln!(
            f,
            "  release summary checksum: 0x{:016x}",
            self.release_summary_checksum
        )?;
        writeln!(
            f,
            "  release-profile identifiers checksum: 0x{:016x}",
            self.release_profile_identifiers_checksum
        )?;
        writeln!(
            f,
            "  release-profile identifiers summary checksum: 0x{:016x}",
            self.release_profile_identifiers_summary_checksum
        )?;
        writeln!(
            f,
            "  release-house-system canonical names summary checksum: 0x{:016x}",
            self.release_house_system_canonical_names_summary_checksum
        )?;
        writeln!(
            f,
            "  release-ayanamsa canonical names summary checksum: 0x{:016x}",
            self.release_ayanamsa_canonical_names_summary_checksum
        )?;
        writeln!(
            f,
            "  release checklist checksum: 0x{:016x}",
            self.release_checklist_checksum
        )?;
        writeln!(
            f,
            "  release checklist summary checksum: 0x{:016x}",
            self.release_checklist_summary_checksum
        )?;
        writeln!(f, "  backend matrix bytes: {}", self.backend_matrix_bytes)?;
        writeln!(
            f,
            "  backend matrix summary bytes: {}",
            self.backend_matrix_summary_bytes
        )?;
        writeln!(
            f,
            "  backend matrix checksum: 0x{:016x}",
            self.backend_matrix_checksum
        )?;
        writeln!(
            f,
            "  backend matrix summary checksum: 0x{:016x}",
            self.backend_matrix_summary_checksum
        )?;
        writeln!(
            f,
            "  API stability posture bytes: {}",
            self.api_stability_bytes
        )?;
        writeln!(
            f,
            "  API stability summary bytes: {}",
            self.api_stability_summary_bytes
        )?;
        writeln!(
            f,
            "  API stability posture checksum: 0x{:016x}",
            self.api_stability_checksum
        )?;
        writeln!(
            f,
            "  API stability summary checksum: 0x{:016x}",
            self.api_stability_summary_checksum
        )?;
        writeln!(
            f,
            "  comparison-envelope summary bytes: {}",
            self.comparison_envelope_summary_bytes
        )?;
        writeln!(
            f,
            "  comparison-body-class tolerance summary bytes: {}",
            self.comparison_body_class_tolerance_summary_bytes
        )?;
        writeln!(
            f,
            "  comparison-corpus release-guard summary bytes: {}",
            self.comparison_corpus_release_guard_summary_bytes
        )?;
        writeln!(
            f,
            "  reference-holdout overlap summary bytes: {}",
            self.reference_holdout_overlap_summary_bytes
        )?;
        writeln!(
            f,
            "  catalog inventory summary bytes: {}",
            self.catalog_inventory_summary_bytes
        )?;
        writeln!(
            f,
            "  validation report summary bytes: {}",
            self.validation_report_summary_bytes
        )?;
        writeln!(
            f,
            "  workspace audit summary bytes: {}",
            self.workspace_audit_summary_bytes
        )?;
        writeln!(
            f,
            "  native-dependency audit summary bytes: {}",
            self.native_dependency_audit_summary_bytes
        )?;
        writeln!(
            f,
            "  artifact summary bytes: {}",
            self.artifact_summary_bytes
        )?;
        writeln!(
            f,
            "  packaged-artifact profile coverage summary bytes: {}",
            self.packaged_artifact_profile_coverage_summary_bytes
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest bytes: {}",
            self.packaged_artifact_generation_manifest_bytes
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest summary bytes: {}",
            self.packaged_artifact_generation_manifest_summary_bytes
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest checksum summary bytes: {}",
            self.packaged_artifact_generation_manifest_checksum_summary_bytes
        )?;
        writeln!(
            f,
            "  benchmark report bytes: {}",
            self.benchmark_report_bytes
        )?;
        writeln!(
            f,
            "  validation report bytes: {}",
            self.validation_report_bytes
        )?;
        writeln!(
            f,
            "  manifest checksum bytes: {}",
            self.manifest_checksum_bytes
        )?;
        writeln!(
            f,
            "  comparison-envelope summary checksum: 0x{:016x}",
            self.comparison_envelope_summary_checksum
        )?;
        writeln!(
            f,
            "  comparison-body-class tolerance summary checksum: 0x{:016x}",
            self.comparison_body_class_tolerance_summary_checksum
        )?;
        writeln!(
            f,
            "  comparison-corpus release-guard summary checksum: 0x{:016x}",
            self.comparison_corpus_release_guard_summary_checksum
        )?;
        writeln!(
            f,
            "  reference-holdout overlap summary checksum: 0x{:016x}",
            self.reference_holdout_overlap_summary_checksum
        )?;
        writeln!(
            f,
            "  catalog inventory summary checksum: 0x{:016x}",
            self.catalog_inventory_summary_checksum
        )?;
        writeln!(
            f,
            "  validation report summary checksum: 0x{:016x}",
            self.validation_report_summary_checksum
        )?;
        writeln!(
            f,
            "  workspace audit summary checksum: 0x{:016x}",
            self.workspace_audit_summary_checksum
        )?;
        writeln!(
            f,
            "  native-dependency audit summary checksum: 0x{:016x}",
            self.native_dependency_audit_summary_checksum
        )?;
        writeln!(
            f,
            "  artifact summary checksum: 0x{:016x}",
            self.artifact_summary_checksum
        )?;
        writeln!(
            f,
            "  packaged-artifact profile coverage summary checksum: 0x{:016x}",
            self.packaged_artifact_profile_coverage_summary_checksum
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest checksum: 0x{:016x}",
            self.packaged_artifact_generation_manifest_checksum
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest summary checksum: 0x{:016x}",
            self.packaged_artifact_generation_manifest_summary_checksum
        )?;
        writeln!(
            f,
            "  packaged-artifact generation manifest checksum summary checksum: 0x{:016x}",
            self.packaged_artifact_generation_manifest_checksum_summary_checksum
        )?;
        writeln!(
            f,
            "  benchmark report checksum: 0x{:016x}",
            self.benchmark_report_checksum
        )?;
        writeln!(
            f,
            "  validation report checksum: 0x{:016x}",
            self.validation_report_checksum
        )?;
        writeln!(f, "  manifest checksum: 0x{:016x}", self.manifest_checksum)
    }
}

impl ReleaseBundle {
    /// Validates the release bundle metadata before it is surfaced to callers.
    ///
    /// The provenance fields are required to stay canonical one-line values so the
    /// release-bundle summary and manifest text remain stable.
    pub fn validate(&self) -> Result<(), ReleaseBundleError> {
        for (path, expected_name, label) in [
            (
                &self.compatibility_profile_path,
                "compatibility-profile.txt",
                "compatibility profile",
            ),
            (
                &self.compatibility_profile_summary_path,
                "compatibility-profile-summary.txt",
                "compatibility profile summary",
            ),
            (
                &self.release_notes_path,
                "release-notes.txt",
                "release notes",
            ),
            (
                &self.release_notes_summary_path,
                "release-notes-summary.txt",
                "release notes summary",
            ),
            (
                &self.release_summary_path,
                "release-summary.txt",
                "release summary",
            ),
            (
                &self.release_profile_identifiers_path,
                "release-profile-identifiers.txt",
                "release-profile identifiers",
            ),
            (
                &self.release_profile_identifiers_summary_path,
                "release-profile-identifiers-summary.txt",
                "release-profile identifiers summary",
            ),
            (
                &self.release_house_system_canonical_names_summary_path,
                "release-house-system-canonical-names-summary.txt",
                "release-house-system canonical names summary",
            ),
            (
                &self.release_ayanamsa_canonical_names_summary_path,
                "release-ayanamsa-canonical-names-summary.txt",
                "release-ayanamsa canonical names summary",
            ),
            (
                &self.release_house_validation_summary_path,
                "release-house-validation-summary.txt",
                "release house validation summary",
            ),
            (
                &self.house_code_aliases_summary_path,
                "house-code-aliases-summary.txt",
                "house code aliases summary",
            ),
            (
                &self.house_formula_families_summary_path,
                "house-formula-families-summary.txt",
                "house formula families summary",
            ),
            (
                &self.house_latitude_sensitive_summary_path,
                "house-latitude-sensitive-summary.txt",
                "house latitude-sensitive summary",
            ),
            (
                &self.release_checklist_path,
                "release-checklist.txt",
                "release checklist",
            ),
            (
                &self.release_checklist_summary_path,
                "release-checklist-summary.txt",
                "release checklist summary",
            ),
            (
                &self.backend_matrix_path,
                "backend-matrix.txt",
                "backend matrix",
            ),
            (
                &self.backend_matrix_summary_path,
                "backend-matrix-summary.txt",
                "backend matrix summary",
            ),
            (
                &self.api_stability_path,
                "api-stability.txt",
                "API stability",
            ),
            (
                &self.api_stability_summary_path,
                "api-stability-summary.txt",
                "API stability summary",
            ),
            (
                &self.output_dir.join("zodiac-policy-summary.txt"),
                "zodiac-policy-summary.txt",
                "zodiac policy summary",
            ),
            (
                &self.output_dir.join("native-sidereal-policy-summary.txt"),
                "native-sidereal-policy-summary.txt",
                "native sidereal policy summary",
            ),
            (
                &self.validation_report_summary_path,
                "validation-report-summary.txt",
                "validation report summary",
            ),
            (
                &self.workspace_provenance_summary_path,
                "workspace-provenance-summary.txt",
                "workspace provenance summary",
            ),
            (
                &self.workspace_audit_summary_path,
                "workspace-audit-summary.txt",
                "workspace audit summary",
            ),
            (
                &self.native_dependency_audit_summary_path,
                "native-dependency-audit-summary.txt",
                "native-dependency audit summary",
            ),
            (
                &self.artifact_summary_path,
                "artifact-summary.txt",
                "artifact summary",
            ),
            (
                &self.packaged_artifact_path,
                "packaged-artifact.bin",
                "packaged-artifact binary",
            ),
            (
                &self.packaged_artifact_checksum_path,
                "packaged-artifact.checksum.txt",
                "packaged-artifact checksum sidecar",
            ),
            (
                &self.packaged_artifact_profile_coverage_summary_path,
                "packaged-artifact-profile-coverage-summary.txt",
                "packaged-artifact profile coverage summary",
            ),
            (
                &self.interpolation_quality_request_corpus_summary_path,
                "interpolation-quality-request-corpus-summary.txt",
                "interpolation-quality sample request corpus summary",
            ),
            (
                &self.packaged_artifact_generation_manifest_path,
                "packaged-artifact-generation-manifest.txt",
                "packaged-artifact generation manifest",
            ),
            (
                &self.packaged_artifact_generation_manifest_checksum_summary_path,
                "packaged-artifact-generation-manifest-checksum-summary.txt",
                "packaged-artifact generation manifest checksum summary",
            ),
            (
                &self.benchmark_report_path,
                "benchmark-report.txt",
                "benchmark report",
            ),
            (
                &self.validation_report_path,
                "validation-report.txt",
                "validation report",
            ),
            (
                &self.manifest_path,
                "bundle-manifest.txt",
                "bundle manifest",
            ),
            (
                &self.manifest_checksum_path,
                "bundle-manifest.checksum.txt",
                "bundle manifest checksum sidecar",
            ),
        ] {
            let expected_path = self.output_dir.join(expected_name);
            if path != &expected_path {
                return Err(ReleaseBundleError::Verification(format!(
                    "unexpected {label} file path: expected {}, found {}",
                    expected_path.display(),
                    path.display()
                )));
            }
        }

        ensure_canonical_manifest_value(&self.source_revision, "source revision")?;
        ensure_canonical_manifest_value(&self.workspace_status, "workspace status")?;
        ensure_canonical_manifest_value(&self.rustc_version, "rustc version")?;
        ensure_canonical_manifest_value(&self.cargo_version, "cargo version")?;
        if self.validation_rounds == 0 {
            return Err(ReleaseBundleError::Verification(
                "release bundle validation rounds must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Writes a release bundle containing the compatibility profile, release-profile
/// identifiers, release notes, release notes summary, release summary, release checklist,
/// release checklist summary, backend matrix, API posture, API stability summary,
/// comparison-corpus summary, source-corpus summary, JPL provenance-only evidence summary, comparison-envelope summary, comparison-corpus release-guard summary, validation report summary, artifact summary,
/// packaged-artifact production-profile summary, packaged-artifact target-threshold summary,
/// packaged-artifact target-threshold state summary, packaged-artifact target-threshold scope envelopes summary, packaged-artifact phase-2 corpus alignment summary, packaged-artifact lookup-epoch policy summary, packaged-artifact generation policy summary, packaged-artifact normalized intermediate summary, packaged-artifact speed policy summary, production-generation summary, production-generation body-class coverage summary, production-generation boundary request corpus summary, production-generation source summary, production-generation manifest summary, production-generation manifest checksum summary, selected-asteroid source request corpus summary, packaged-artifact generation manifest, packaged-artifact generation manifest summary, packaged-artifact generation manifest checksum summary, packaged-artifact generation manifest checksum sidecar, benchmark report, validation report, and a manifest.
pub fn render_release_bundle(
    rounds: usize,
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let profile_text = current_compatibility_profile().to_string();
    let profile_summary_text = render_compatibility_profile_summary_text();
    let release_notes_text = render_release_notes_text();
    let release_summary_text = render_release_summary_text();
    let release_profile_identifiers = current_release_profile_identifiers();
    release_profile_identifiers
        .validate()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let release_profile_identifiers_text = format!(
        "Release profile identifiers: {}\n",
        release_profile_identifiers.summary_line()
    );
    let release_profile_identifiers_summary_text = render_release_profile_identifiers_summary();
    let release_checklist_text = render_release_checklist_text();
    let release_checklist_summary_text = render_release_checklist_summary_text();
    let backend_matrix_text = render_backend_matrix_report()?;
    let backend_matrix_summary_text = render_backend_matrix_summary();
    let api_stability_text = current_api_stability_profile().to_string();
    let api_stability_summary_text = render_api_stability_summary();
    let validation_report = build_validation_report(rounds)?;
    let validation_report_text = validation_report.to_string();
    let comparison_corpus_summary_text = render_comparison_corpus_summary_text();
    let source_corpus_summary_text = source_corpus_summary_for_report();
    let jpl_provenance_only_summary_text = jpl_provenance_only_summary_for_report();
    let comparison_envelope_summary_text = render_comparison_envelope_summary_text();
    let comparison_body_class_tolerance_summary_text =
        render_comparison_body_class_tolerance_summary_text();
    let comparison_corpus_release_guard_summary_text =
        render_comparison_corpus_release_guard_summary_text();
    let catalog_inventory_summary_text = render_catalog_inventory_summary();
    let catalog_posture_summary_text = render_catalog_posture_summary();
    let custom_definition_ayanamsa_labels_summary_text =
        render_custom_definition_ayanamsa_labels_summary();
    let ayanamsa_provenance_summary_text = format_ayanamsa_provenance_for_report();
    let validation_report_summary_text = render_validation_report_summary_text(&validation_report);
    let release_body_claims_summary = release_body_claims_summary_for_report();
    release_body_claims_summary
        .validated_summary_line()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let release_body_claims_summary_text = release_body_claims_summary.summary_line();
    let body_date_channel_claims_summary_text = render_body_date_channel_claims_summary_text();
    let pluto_fallback_summary = pluto_fallback_summary_for_report();
    pluto_fallback_summary
        .validated_summary_line()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let pluto_fallback_summary_text = pluto_fallback_summary.summary_line();
    let request_policy_summary_text = render_request_policy_summary_text();
    let observer_policy_summary_text = render_observer_policy_summary_text();
    let apparentness_policy_summary_text = render_apparentness_policy_summary_text();
    let request_semantics_summary_text = render_request_semantics_summary_text();
    let unsupported_modes_summary_text = render_unsupported_modes_summary_text();
    let time_scale_policy_summary_text = render_time_scale_policy_summary_text();
    let utc_convenience_policy_summary_text = render_utc_convenience_policy_summary_text();
    let delta_t_policy_summary_text = render_delta_t_policy_summary_text();
    let native_sidereal_policy_summary_text = render_native_sidereal_policy_summary_text();
    let zodiac_policy_summary_text = render_zodiac_policy_summary_text();
    let lunar_theory_limitations_summary_text = lunar_theory_limitations_summary_for_report();
    let lunar_theory_source_selection_summary_text =
        pleiades_elp::lunar_theory_source_selection_summary_for_report();
    let lunar_theory_source_family_summary_text =
        pleiades_elp::lunar_theory_source_family_summary_for_report();
    let lunar_source_window_summary_text = validated_lunar_source_window_summary_for_report()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let lunar_source_window_summary_checksum = checksum64(&lunar_source_window_summary_text);
    let lunar_reference_error_envelope_summary_text =
        render_lunar_reference_error_envelope_summary_text();
    let lunar_reference_error_envelope_summary_checksum =
        checksum64(&lunar_reference_error_envelope_summary_text);
    let lunar_equatorial_reference_error_envelope_summary_text =
        render_lunar_equatorial_reference_error_envelope_summary_text();
    let lunar_equatorial_reference_error_envelope_summary_checksum =
        checksum64(&lunar_equatorial_reference_error_envelope_summary_text);
    let lunar_apparent_comparison_summary_text = render_lunar_apparent_comparison_summary_text();
    let lunar_apparent_comparison_summary_checksum =
        checksum64(&lunar_apparent_comparison_summary_text);
    let lunar_theory_catalog_validation_summary =
        pleiades_elp::lunar_theory_catalog_validation_summary();
    lunar_theory_catalog_validation_summary
        .validation_result
        .as_ref()
        .map_err(|error| {
            ReleaseBundleError::Verification(format!(
                "lunar theory catalog validation summary is invalid: {error}"
            ))
        })?;
    let lunar_theory_catalog_validation_summary_text =
        validated_lunar_theory_catalog_validation_summary_for_report();
    let request_surface_summary_text = render_request_surface_summary_text();
    let compatibility_caveats_summary_text = render_compatibility_caveats_summary();
    let benchmark_corpus_summary_text = render_benchmark_corpus_summary_text();
    let chart_benchmark_corpus_summary_text = render_chart_benchmark_corpus_summary_text();
    let interpolation_quality_request_corpus_summary_text =
        interpolation_quality_sample_request_corpus_summary_for_report();
    let interpolation_quality_request_corpus_summary_checksum =
        checksum64(&interpolation_quality_request_corpus_summary_text);
    let selected_asteroid_source_request_corpus_summary_text =
        validated_selected_asteroid_source_request_corpus_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let selected_asteroid_source_request_corpus_summary_checksum =
        checksum64(&selected_asteroid_source_request_corpus_summary_text);
    let selected_asteroid_source_request_corpus_equatorial_summary_text =
        validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let selected_asteroid_source_request_corpus_equatorial_summary_checksum =
        checksum64(&selected_asteroid_source_request_corpus_equatorial_summary_text);
    let selected_asteroid_source_window_summary_text =
        validated_selected_asteroid_source_window_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let selected_asteroid_source_window_summary_checksum =
        checksum64(&selected_asteroid_source_window_summary_text);
    let benchmark_report_text = render_benchmark_report(rounds)?;
    let workspace_provenance_summary_text = workspace_provenance_summary_for_report();
    let workspace_provenance_summary_checksum = checksum64(&workspace_provenance_summary_text);
    let workspace_audit_summary_text = render_workspace_audit_summary()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let artifact_summary_text = render_artifact_summary()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let packaged_artifact_profile_coverage_summary_text =
        packaged_artifact_profile_coverage_summary_for_report();
    let packaged_artifact_access_summary_text = packaged_artifact_access_summary_for_report();
    let packaged_artifact_output_support_summary_text =
        packaged_artifact_output_support_summary_for_report();
    let packaged_artifact_fit_sample_classes_summary_text =
        packaged_artifact_fit_sample_classes_summary_for_report();
    let packaged_artifact_fit_threshold_violation_count_summary_text =
        packaged_artifact_fit_threshold_violation_count_for_report();
    let packaged_artifact_fit_threshold_violations_summary_text =
        packaged_artifact_fit_threshold_violation_summary_for_report();
    let packaged_artifact_body_cadence_summary_text =
        validated_packaged_artifact_body_cadence_summary_for_report();
    let packaged_artifact_body_class_span_cap_summary_text =
        validated_packaged_artifact_body_class_span_cap_summary_for_report();
    let packaged_artifact_normalized_intermediate_summary_text =
        validated_packaged_artifact_normalized_intermediate_summary_for_report();
    let packaged_artifact_speed_policy_summary_text =
        packaged_artifact_speed_policy_summary_for_report();
    let packaged_artifact_storage_summary_text =
        validated_packaged_artifact_storage_summary_for_report();
    let packaged_artifact_production_profile_summary_text =
        validated_packaged_artifact_production_profile_summary_for_report();
    let packaged_frame_treatment_summary_text = packaged_frame_treatment_summary_for_report();
    let packaged_artifact_target_threshold_summary_text =
        validated_packaged_artifact_target_threshold_summary_for_report();
    let packaged_artifact_target_threshold_state_summary_text =
        validated_packaged_artifact_target_threshold_state_for_report();
    let packaged_artifact_source_fit_holdout_sync_summary_text =
        validated_packaged_artifact_source_fit_holdout_sync_summary_for_report();
    let packaged_artifact_target_threshold_scope_envelopes_summary_text =
        validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report();
    let packaged_artifact_phase2_corpus_alignment_summary_text =
        validated_packaged_artifact_phase2_corpus_alignment_summary_for_report();
    let packaged_lookup_epoch_policy_summary_text =
        packaged_lookup_epoch_policy_summary_for_report();
    let packaged_artifact_generation_manifest_text =
        packaged_artifact_generation_manifest_for_report();
    let packaged_artifact_generation_manifest_summary_text =
        packaged_artifact_generation_manifest_for_report();
    let provenance = workspace_provenance();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let profile_summary_path = output_dir.join("compatibility-profile-summary.txt");
    let release_notes_path = output_dir.join("release-notes.txt");
    let release_notes_summary_path = output_dir.join("release-notes-summary.txt");
    let release_summary_path = output_dir.join("release-summary.txt");
    let release_profile_identifiers_path = output_dir.join("release-profile-identifiers.txt");
    let release_profile_identifiers_summary_path =
        output_dir.join("release-profile-identifiers-summary.txt");
    let release_house_system_canonical_names_summary_path =
        output_dir.join("release-house-system-canonical-names-summary.txt");
    let release_ayanamsa_canonical_names_summary_path =
        output_dir.join("release-ayanamsa-canonical-names-summary.txt");
    let release_house_validation_summary_path =
        output_dir.join("release-house-validation-summary.txt");
    let target_house_scope_summary_path = output_dir.join("target-house-scope-summary.txt");
    let target_ayanamsa_scope_summary_path = output_dir.join("target-ayanamsa-scope-summary.txt");
    let house_code_aliases_summary_path = output_dir.join("house-code-aliases-summary.txt");
    let house_formula_families_summary_path = output_dir.join("house-formula-families-summary.txt");
    let house_latitude_sensitive_summary_path =
        output_dir.join("house-latitude-sensitive-summary.txt");
    let house_latitude_sensitive_constraints_summary_path =
        output_dir.join("house-latitude-sensitive-constraints-summary.txt");
    let house_latitude_sensitive_failure_modes_summary_path =
        output_dir.join("house-latitude-sensitive-failure-modes-summary.txt");
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let release_checklist_summary_path = output_dir.join("release-checklist-summary.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let comparison_corpus_summary_path = output_dir.join("comparison-corpus-summary.txt");
    let source_corpus_summary_path = output_dir.join("source-corpus-summary.txt");
    let comparison_snapshot_summary_path = output_dir.join("comparison-snapshot-summary.txt");
    let comparison_snapshot_source_summary_path =
        output_dir.join("comparison-snapshot-source-summary.txt");
    let comparison_snapshot_source_window_summary_path =
        output_dir.join("comparison-snapshot-source-window-summary.txt");
    let comparison_snapshot_body_class_coverage_summary_path =
        output_dir.join("comparison-snapshot-body-class-coverage-summary.txt");
    let comparison_snapshot_manifest_summary_path =
        output_dir.join("comparison-snapshot-manifest-summary.txt");
    let comparison_envelope_summary_path = output_dir.join("comparison-envelope-summary.txt");
    let comparison_body_class_tolerance_summary_path =
        output_dir.join("comparison-body-class-tolerance-summary.txt");
    let comparison_body_class_error_envelope_summary_path =
        output_dir.join("comparison-body-class-error-envelope-summary.txt");
    let comparison_corpus_release_guard_summary_path =
        output_dir.join("comparison-corpus-release-guard-summary.txt");
    let comparison_corpus_guard_summary_path =
        output_dir.join("comparison-corpus-guard-summary.txt");
    let reference_holdout_overlap_summary_path =
        output_dir.join("reference-holdout-overlap-summary.txt");
    let reference_snapshot_bridge_day_summary_path =
        output_dir.join("reference-snapshot-bridge-day-summary.txt");
    let reference_snapshot_major_body_boundary_window_summary_path =
        output_dir.join("reference-snapshot-major-body-boundary-window-summary.txt");
    let reference_snapshot_boundary_epoch_coverage_summary_path =
        output_dir.join("reference-snapshot-boundary-epoch-coverage-summary.txt");
    let reference_snapshot_pre_bridge_boundary_summary_path =
        output_dir.join("reference-snapshot-pre-bridge-boundary-summary.txt");
    let reference_snapshot_2451917_major_body_boundary_summary_path =
        output_dir.join("reference-snapshot-2451917-major-body-boundary-summary.txt");
    let reference_snapshot_2451918_major_body_boundary_summary_path =
        output_dir.join("reference-snapshot-2451918-major-body-boundary-summary.txt");
    let reference_snapshot_2451919_major_body_boundary_summary_path =
        output_dir.join("reference-snapshot-2451919-major-body-boundary-summary.txt");
    let reference_snapshot_2451916_major_body_dense_boundary_summary_path =
        output_dir.join("reference-snapshot-2451916-major-body-dense-boundary-summary.txt");
    let reference_snapshot_sparse_boundary_summary_path =
        output_dir.join("reference-snapshot-sparse-boundary-summary.txt");
    let reference_snapshot_exact_j2000_evidence_summary_path =
        output_dir.join("reference-snapshot-exact-j2000-evidence-summary.txt");
    let reference_snapshot_source_summary_path =
        output_dir.join("reference-snapshot-source-summary.txt");
    let reference_snapshot_source_window_summary_path =
        output_dir.join("reference-snapshot-source-window-summary.txt");
    let reference_snapshot_manifest_summary_path =
        output_dir.join("reference-snapshot-manifest-summary.txt");
    let reference_snapshot_body_class_coverage_summary_path =
        output_dir.join("reference-snapshot-body-class-coverage-summary.txt");
    let reference_snapshot_equatorial_parity_summary_path =
        output_dir.join("reference-snapshot-equatorial-parity-summary.txt");
    let reference_asteroid_source_window_summary_path =
        output_dir.join("reference-asteroid-source-window-summary.txt");
    let reference_asteroid_equatorial_evidence_summary_path =
        output_dir.join("reference-asteroid-equatorial-evidence-summary.txt");
    let independent_holdout_source_window_summary_path =
        output_dir.join("independent-holdout-source-window-summary.txt");
    let independent_holdout_equatorial_parity_summary_path =
        output_dir.join("independent-holdout-equatorial-parity-summary.txt");
    let independent_holdout_body_class_coverage_summary_path =
        output_dir.join("independent-holdout-body-class-coverage-summary.txt");
    let independent_holdout_quarter_day_boundary_summary_path =
        output_dir.join("independent-holdout-quarter-day-boundary-summary.txt");
    let production_generation_boundary_source_summary_path =
        output_dir.join("production-generation-boundary-source-summary.txt");
    let production_generation_boundary_window_summary_path =
        output_dir.join("production-generation-boundary-window-summary.txt");
    let production_generation_boundary_request_corpus_summary_path =
        output_dir.join("production-generation-boundary-request-corpus-summary.txt");
    let production_generation_boundary_request_corpus_equatorial_summary_path =
        output_dir.join("production-generation-boundary-request-corpus-equatorial-summary.txt");
    let production_generation_summary_path = output_dir.join("production-generation-summary.txt");
    let production_generation_body_class_coverage_summary_path =
        output_dir.join("production-generation-body-class-coverage-summary.txt");
    let production_generation_source_summary_path =
        output_dir.join("production-generation-source-summary.txt");
    let production_generation_source_revision_summary_path =
        output_dir.join("production-generation-source-revision-summary.txt");
    let production_generation_source_window_summary_path =
        output_dir.join("production-generation-source-window-summary.txt");
    let production_generation_quarter_day_boundary_summary_path =
        output_dir.join("production-generation-quarter-day-boundary-summary.txt");
    let production_generation_corpus_shape_summary_path =
        output_dir.join("production-generation-corpus-shape-summary.txt");
    let catalog_posture_summary_path = output_dir.join("catalog-posture-summary.txt");
    let production_generation_manifest_summary_path =
        output_dir.join("production-generation-manifest-summary.txt");
    let production_generation_manifest_checksum_path =
        output_dir.join("production-generation-manifest-checksum-summary.txt");
    let reference_snapshot_summary_path = output_dir.join("reference-snapshot-summary.txt");
    let catalog_inventory_summary_path = output_dir.join("catalog-inventory-summary.txt");
    let custom_definition_ayanamsa_labels_summary_path =
        output_dir.join("custom-definition-ayanamsa-labels-summary.txt");
    let ayanamsa_provenance_summary_path = output_dir.join("ayanamsa-provenance-summary.txt");
    let validation_report_summary_path = output_dir.join("validation-report-summary.txt");
    let workspace_provenance_summary_path = output_dir.join("workspace-provenance-summary.txt");
    let release_body_claims_summary_path = output_dir.join("release-body-claims-summary.txt");
    let pluto_fallback_summary_path = output_dir.join("pluto-fallback-summary.txt");
    let request_policy_summary_path = output_dir.join("request-policy-summary.txt");
    let observer_policy_summary_path = output_dir.join("observer-policy-summary.txt");
    let apparentness_policy_summary_path = output_dir.join("apparentness-policy-summary.txt");
    let request_semantics_summary_path = output_dir.join("request-semantics-summary.txt");
    let unsupported_modes_summary_path = output_dir.join("unsupported-modes-summary.txt");
    let time_scale_policy_summary_path = output_dir.join("time-scale-policy-summary.txt");
    let utc_convenience_policy_summary_path = output_dir.join("utc-convenience-policy-summary.txt");
    let delta_t_policy_summary_path = output_dir.join("delta-t-policy-summary.txt");
    let native_sidereal_policy_summary_path = output_dir.join("native-sidereal-policy-summary.txt");
    let zodiac_policy_summary_path = output_dir.join("zodiac-policy-summary.txt");
    let lunar_theory_limitations_summary_path =
        output_dir.join("lunar-theory-limitations-summary.txt");
    let lunar_theory_source_selection_summary_path =
        output_dir.join("lunar-theory-source-selection-summary.txt");
    let lunar_theory_source_family_summary_path =
        output_dir.join("lunar-theory-source-family-summary.txt");
    let lunar_source_window_summary_path = output_dir.join("lunar-source-window-summary.txt");
    let lunar_reference_error_envelope_summary_path =
        output_dir.join("lunar-reference-error-envelope-summary.txt");
    let lunar_equatorial_reference_error_envelope_summary_path =
        output_dir.join("lunar-equatorial-reference-error-envelope-summary.txt");
    let lunar_apparent_comparison_summary_path =
        output_dir.join("lunar-apparent-comparison-summary.txt");
    let lunar_theory_catalog_validation_summary_path =
        output_dir.join("lunar-theory-catalog-validation-summary.txt");
    let request_surface_summary_path = output_dir.join("request-surface-summary.txt");
    let compatibility_caveats_summary_path = output_dir.join("compatibility-caveats-summary.txt");
    let workspace_audit_summary_path = output_dir.join("workspace-audit-summary.txt");
    let native_dependency_audit_summary_path =
        output_dir.join("native-dependency-audit-summary.txt");
    let artifact_summary_path = output_dir.join("artifact-summary.txt");
    let packaged_artifact_path = output_dir.join("packaged-artifact.bin");
    let packaged_artifact_checksum_path = output_dir.join("packaged-artifact.checksum.txt");
    let packaged_artifact_profile_coverage_summary_path =
        output_dir.join("packaged-artifact-profile-coverage-summary.txt");
    let packaged_artifact_access_summary_path =
        output_dir.join("packaged-artifact-access-summary.txt");
    let packaged_artifact_output_support_summary_path =
        output_dir.join("packaged-artifact-output-support-summary.txt");
    let packaged_artifact_fit_sample_classes_summary_path =
        output_dir.join("packaged-artifact-fit-sample-classes-summary.txt");
    let packaged_artifact_fit_threshold_violation_count_summary_path =
        output_dir.join("packaged-artifact-fit-threshold-violation-count-summary.txt");
    let packaged_artifact_fit_threshold_violations_summary_path =
        output_dir.join("packaged-artifact-fit-threshold-violations-summary.txt");
    let packaged_artifact_body_cadence_summary_path =
        output_dir.join("packaged-artifact-body-cadence-summary.txt");
    let packaged_artifact_body_class_span_cap_summary_path =
        output_dir.join("packaged-artifact-body-class-span-cap-summary.txt");
    let packaged_artifact_normalized_intermediate_summary_path =
        output_dir.join("packaged-artifact-normalized-intermediate-summary.txt");
    let packaged_artifact_speed_policy_summary_path =
        output_dir.join("packaged-artifact-speed-policy-summary.txt");
    let packaged_artifact_storage_summary_path =
        output_dir.join("packaged-artifact-storage-summary.txt");
    let packaged_artifact_production_profile_summary_path =
        output_dir.join("packaged-artifact-production-profile-summary.txt");
    let packaged_frame_treatment_summary_path =
        output_dir.join("packaged-frame-treatment-summary.txt");
    let packaged_artifact_target_threshold_summary_path =
        output_dir.join("packaged-artifact-target-threshold-summary.txt");
    let packaged_artifact_target_threshold_state_summary_path =
        output_dir.join("packaged-artifact-target-threshold-state-summary.txt");
    let packaged_artifact_source_fit_holdout_sync_summary_path =
        output_dir.join("packaged-artifact-source-fit-holdout-sync-summary.txt");
    let packaged_artifact_target_threshold_scope_envelopes_summary_path =
        output_dir.join("packaged-artifact-target-threshold-scope-envelopes-summary.txt");
    let packaged_artifact_phase2_corpus_alignment_summary_path =
        output_dir.join("packaged-artifact-phase2-corpus-alignment-summary.txt");
    let packaged_lookup_epoch_policy_summary_path =
        output_dir.join("packaged-lookup-epoch-policy-summary.txt");
    let packaged_artifact_generation_policy_summary_path =
        output_dir.join("packaged-artifact-generation-policy-summary.txt");
    let packaged_artifact_generation_residual_bodies_summary_path =
        output_dir.join("packaged-artifact-generation-residual-bodies-summary.txt");
    let packaged_artifact_regeneration_summary_path =
        output_dir.join("packaged-artifact-regeneration-summary.txt");
    let packaged_artifact_generation_manifest_path =
        output_dir.join("packaged-artifact-generation-manifest.txt");
    let packaged_artifact_generation_manifest_summary_path =
        output_dir.join("packaged-artifact-generation-manifest-summary.txt");
    let packaged_artifact_generation_manifest_checksum_summary_path =
        output_dir.join("packaged-artifact-generation-manifest-checksum-summary.txt");
    let packaged_artifact_generation_manifest_checksum_path =
        output_dir.join("packaged-artifact-generation-manifest.checksum.txt");
    let benchmark_corpus_summary_path = output_dir.join("benchmark-corpus-summary.txt");
    let chart_benchmark_corpus_summary_path = output_dir.join("chart-benchmark-corpus-summary.txt");
    let interpolation_quality_request_corpus_summary_path =
        output_dir.join("interpolation-quality-request-corpus-summary.txt");
    let selected_asteroid_source_request_corpus_summary_path =
        output_dir.join("selected-asteroid-source-request-corpus-summary.txt");
    let selected_asteroid_source_request_corpus_equatorial_summary_path =
        output_dir.join("selected-asteroid-source-request-corpus-equatorial-summary.txt");
    let selected_asteroid_source_window_summary_path =
        output_dir.join("selected-asteroid-source-window-summary.txt");
    let benchmark_report_path = output_dir.join("benchmark-report.txt");
    let report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let manifest_checksum_path = output_dir.join("bundle-manifest.checksum.txt");
    let compatibility_profile_checksum = checksum64(&profile_text);
    let compatibility_profile_summary_checksum = checksum64(&profile_summary_text);
    let release_notes_checksum = checksum64(&release_notes_text);
    let release_notes_summary_text = render_release_notes_summary_text();
    let release_notes_summary_checksum = checksum64(&release_notes_summary_text);
    let release_summary_checksum = checksum64(&release_summary_text);
    let release_profile_identifiers_checksum = checksum64(&release_profile_identifiers_text);
    let release_profile_identifiers_summary_checksum =
        checksum64(&release_profile_identifiers_summary_text);
    let release_house_system_canonical_names_summary_text =
        render_release_house_system_canonical_names_summary();
    let release_house_system_canonical_names_summary_checksum =
        checksum64(&release_house_system_canonical_names_summary_text);
    let release_ayanamsa_canonical_names_summary_text =
        render_release_ayanamsa_canonical_names_summary();
    let release_ayanamsa_canonical_names_summary_checksum =
        checksum64(&release_ayanamsa_canonical_names_summary_text);
    let release_house_validation_summary_text = release_house_validation_summary_for_report();
    let target_house_scope_summary_text = render_target_house_scope_summary();
    let target_ayanamsa_scope_summary_text = render_target_ayanamsa_scope_summary();
    let target_house_scope_summary_checksum = checksum64(&target_house_scope_summary_text);
    let target_ayanamsa_scope_summary_checksum = checksum64(&target_ayanamsa_scope_summary_text);
    let house_code_aliases_summary_text = validated_house_code_aliases_summary_for_report()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let house_code_aliases_summary_checksum = checksum64(&house_code_aliases_summary_text);
    let house_formula_families_summary_text = format_house_formula_families_for_report();
    let house_formula_families_summary_checksum = checksum64(&house_formula_families_summary_text);
    let house_latitude_sensitive_summary_text =
        format_latitude_sensitive_house_systems_for_report();
    let house_latitude_sensitive_summary_checksum =
        checksum64(&house_latitude_sensitive_summary_text);
    let house_latitude_sensitive_constraints_summary_text =
        format_latitude_sensitive_house_constraints_for_report();
    let house_latitude_sensitive_constraints_summary_checksum =
        checksum64(&house_latitude_sensitive_constraints_summary_text);
    let house_latitude_sensitive_failure_modes_summary_text =
        format_latitude_sensitive_house_failure_modes_for_report();
    let house_latitude_sensitive_failure_modes_summary_checksum =
        checksum64(&house_latitude_sensitive_failure_modes_summary_text);
    let release_house_validation_summary_checksum =
        checksum64(&release_house_validation_summary_text);
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let release_checklist_summary_checksum = checksum64(&release_checklist_summary_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let comparison_corpus_summary_checksum = checksum64(&comparison_corpus_summary_text);
    let source_corpus_summary_checksum = checksum64(&source_corpus_summary_text);
    let jpl_source_posture_summary_text = jpl_source_posture_summary_for_report();
    let jpl_source_posture_summary_checksum = checksum64(&jpl_source_posture_summary_text);
    let jpl_provenance_only_summary_checksum = checksum64(&jpl_provenance_only_summary_text);
    let comparison_snapshot_summary_text = comparison_snapshot_summary_for_report();
    let comparison_snapshot_summary_checksum = checksum64(&comparison_snapshot_summary_text);
    let comparison_snapshot_source_summary_text = comparison_snapshot_source_summary_for_report();
    let comparison_snapshot_source_summary_checksum =
        checksum64(&comparison_snapshot_source_summary_text);
    let comparison_snapshot_source_window_summary_text =
        comparison_snapshot_source_window_summary_for_report();
    let comparison_snapshot_source_window_summary_checksum =
        checksum64(&comparison_snapshot_source_window_summary_text);
    let comparison_snapshot_body_class_coverage_summary_text =
        validated_comparison_snapshot_body_class_coverage_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let comparison_snapshot_body_class_coverage_summary_checksum =
        checksum64(&comparison_snapshot_body_class_coverage_summary_text);
    let comparison_snapshot_manifest_summary_text =
        validated_comparison_snapshot_manifest_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let comparison_snapshot_manifest_summary_checksum =
        checksum64(&comparison_snapshot_manifest_summary_text);
    let comparison_envelope_summary_checksum = checksum64(&comparison_envelope_summary_text);
    let comparison_body_class_tolerance_summary_checksum =
        checksum64(&comparison_body_class_tolerance_summary_text);
    let comparison_body_class_error_envelope_summary_text =
        comparison_body_class_error_envelope_summary_for_report();
    let comparison_body_class_error_envelope_summary_checksum =
        checksum64(&comparison_body_class_error_envelope_summary_text);
    let comparison_corpus_release_guard_summary_checksum =
        checksum64(&comparison_corpus_release_guard_summary_text);
    let reference_holdout_overlap_summary_text =
        validated_reference_holdout_overlap_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let reference_holdout_overlap_summary_checksum =
        checksum64(&reference_holdout_overlap_summary_text);
    let reference_snapshot_bridge_day_summary_text =
        pleiades_jpl::validated_reference_snapshot_bridge_day_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let reference_snapshot_bridge_day_summary_checksum =
        checksum64(&reference_snapshot_bridge_day_summary_text);
    let reference_snapshot_major_body_boundary_window_summary_text =
        reference_snapshot_major_body_boundary_window_summary_for_report();
    let reference_snapshot_major_body_boundary_window_summary_checksum =
        checksum64(&reference_snapshot_major_body_boundary_window_summary_text);
    let reference_snapshot_boundary_epoch_coverage_summary_text =
        reference_snapshot_boundary_epoch_coverage_summary_for_report();
    let reference_snapshot_boundary_epoch_coverage_summary_checksum =
        checksum64(&reference_snapshot_boundary_epoch_coverage_summary_text);
    let reference_snapshot_pre_bridge_boundary_summary_text =
        reference_snapshot_pre_bridge_boundary_summary_for_report();
    let reference_snapshot_pre_bridge_boundary_summary_checksum =
        checksum64(&reference_snapshot_pre_bridge_boundary_summary_text);
    let reference_snapshot_2451917_major_body_boundary_summary_text =
        reference_snapshot_2451917_major_body_boundary_summary_for_report();
    let reference_snapshot_2451917_major_body_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451917_major_body_boundary_summary_text);
    let reference_snapshot_2451918_major_body_boundary_summary_text =
        reference_snapshot_2451918_major_body_boundary_summary_for_report();
    let reference_snapshot_2451918_major_body_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451918_major_body_boundary_summary_text);
    let reference_snapshot_2451919_major_body_boundary_summary_text =
        reference_snapshot_2451919_major_body_boundary_summary_for_report();
    let reference_snapshot_2451919_major_body_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451919_major_body_boundary_summary_text);
    let reference_snapshot_2451916_major_body_dense_boundary_summary_text =
        reference_snapshot_2451916_major_body_dense_boundary_summary_for_report();
    let reference_snapshot_2451916_major_body_dense_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451916_major_body_dense_boundary_summary_text);
    let reference_snapshot_sparse_boundary_summary_text =
        reference_snapshot_sparse_boundary_summary_for_report();
    let reference_snapshot_sparse_boundary_summary_checksum =
        checksum64(&reference_snapshot_sparse_boundary_summary_text);
    let reference_snapshot_exact_j2000_evidence_summary_text =
        reference_snapshot_exact_j2000_evidence_summary_for_report();
    let reference_snapshot_exact_j2000_evidence_summary_checksum =
        checksum64(&reference_snapshot_exact_j2000_evidence_summary_text);
    let reference_snapshot_source_summary_text = reference_snapshot_source_summary_for_report();
    let reference_snapshot_source_summary_checksum =
        checksum64(&reference_snapshot_source_summary_text);
    let reference_snapshot_source_window_summary_text =
        pleiades_jpl::validated_reference_snapshot_source_window_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let reference_snapshot_source_window_summary_checksum =
        checksum64(&reference_snapshot_source_window_summary_text);
    let reference_snapshot_manifest_summary_text = reference_snapshot_manifest_summary_for_report();
    let reference_snapshot_manifest_summary_checksum =
        checksum64(&reference_snapshot_manifest_summary_text);
    let reference_snapshot_body_class_coverage_summary_text =
        reference_snapshot_body_class_coverage_summary_for_report();
    let reference_snapshot_body_class_coverage_summary_checksum =
        checksum64(&reference_snapshot_body_class_coverage_summary_text);
    let reference_snapshot_equatorial_parity_summary_text =
        reference_snapshot_equatorial_parity_summary_for_report();
    let reference_snapshot_equatorial_parity_summary_checksum =
        checksum64(&reference_snapshot_equatorial_parity_summary_text);
    let reference_asteroid_source_window_summary_text =
        reference_asteroid_source_window_summary_for_report();
    let reference_asteroid_source_window_summary_checksum =
        checksum64(&reference_asteroid_source_window_summary_text);
    let reference_asteroid_equatorial_evidence_summary_text =
        reference_asteroid_equatorial_evidence_summary_for_report();
    let reference_asteroid_equatorial_evidence_summary_checksum =
        checksum64(&reference_asteroid_equatorial_evidence_summary_text);
    let independent_holdout_source_window_summary_text =
        independent_holdout_snapshot_source_window_summary_for_report();
    let independent_holdout_source_window_summary_checksum =
        checksum64(&independent_holdout_source_window_summary_text);
    let independent_holdout_equatorial_parity_summary_text =
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report();
    let independent_holdout_equatorial_parity_summary_checksum =
        checksum64(&independent_holdout_equatorial_parity_summary_text);
    let independent_holdout_body_class_coverage_summary_text =
        independent_holdout_snapshot_body_class_coverage_summary_for_report();
    let independent_holdout_body_class_coverage_summary_checksum =
        checksum64(&independent_holdout_body_class_coverage_summary_text);
    let independent_holdout_quarter_day_boundary_summary_text =
        independent_holdout_snapshot_quarter_day_boundary_summary_for_report();
    let independent_holdout_quarter_day_boundary_summary_checksum =
        checksum64(&independent_holdout_quarter_day_boundary_summary_text);
    let production_generation_boundary_source_summary_text =
        production_generation_boundary_source_summary_for_report();
    let production_generation_boundary_source_summary_checksum =
        checksum64(&production_generation_boundary_source_summary_text);
    let production_generation_boundary_window_summary_text =
        production_generation_boundary_window_summary_for_report();
    let production_generation_boundary_window_summary_checksum =
        checksum64(&production_generation_boundary_window_summary_text);
    let production_generation_boundary_request_corpus_summary_text =
        production_generation_boundary_request_corpus_summary_for_report();
    let production_generation_boundary_request_corpus_equatorial_summary_text =
        production_generation_boundary_request_corpus_equatorial_summary_for_report();
    let production_generation_boundary_request_corpus_summary_checksum =
        checksum64(&production_generation_boundary_request_corpus_summary_text);
    let production_generation_boundary_request_corpus_equatorial_summary_checksum =
        checksum64(&production_generation_boundary_request_corpus_equatorial_summary_text);
    let reference_snapshot_summary_text = reference_snapshot_summary_for_report();
    let reference_snapshot_summary_checksum = checksum64(&reference_snapshot_summary_text);
    let production_generation_summary_text = production_generation_snapshot_summary_for_report();
    let production_generation_summary_checksum = checksum64(&production_generation_summary_text);
    let production_generation_body_class_coverage_summary_text =
        validated_production_generation_body_class_coverage_summary_for_report();
    let production_generation_body_class_coverage_summary_checksum =
        checksum64(&production_generation_body_class_coverage_summary_text);
    let production_generation_source_summary_text =
        production_generation_source_summary_for_report();
    let production_generation_source_summary_checksum =
        checksum64(&production_generation_source_summary_text);
    ensure_production_generation_source_summary_matches_current_rendering(
        &production_generation_source_summary_text,
    )?;
    let production_generation_source_revision_summary_text =
        production_generation_source_revision_summary_for_report();
    let production_generation_source_revision_summary_checksum =
        checksum64(&production_generation_source_revision_summary_text);
    let production_generation_source_window_summary_text =
        production_generation_snapshot_window_summary_for_report();
    let production_generation_source_window_summary_checksum =
        checksum64(&production_generation_source_window_summary_text);
    let production_generation_quarter_day_boundary_summary_text =
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report();
    let production_generation_quarter_day_boundary_summary_checksum =
        checksum64(&production_generation_quarter_day_boundary_summary_text);
    let production_generation_corpus_shape_summary_text =
        production_generation_corpus_shape_summary_for_report();
    let production_generation_corpus_shape_summary_checksum =
        checksum64(&production_generation_corpus_shape_summary_text);
    let production_generation_manifest_summary_text =
        validated_production_generation_manifest_summary_text_for_report();
    let production_generation_manifest_summary_checksum =
        checksum64(&production_generation_manifest_summary_text);
    let production_generation_manifest_checksum_text =
        production_generation_manifest_checksum_for_report();
    let production_generation_manifest_checksum_checksum =
        checksum64(&production_generation_manifest_checksum_text);
    let catalog_inventory_summary_checksum = checksum64(&catalog_inventory_summary_text);
    let catalog_posture_summary_checksum = checksum64(&catalog_posture_summary_text);
    let custom_definition_ayanamsa_labels_summary_checksum =
        checksum64(&custom_definition_ayanamsa_labels_summary_text);
    let ayanamsa_provenance_summary_checksum = checksum64(&ayanamsa_provenance_summary_text);
    let validation_report_summary_checksum = checksum64(&validation_report_summary_text);
    let release_body_claims_summary_checksum = checksum64(release_body_claims_summary_text);
    let body_date_channel_claims_summary_checksum =
        checksum64(&body_date_channel_claims_summary_text);
    let pluto_fallback_summary_checksum = checksum64(pluto_fallback_summary_text);
    let request_policy_summary_checksum = checksum64(&request_policy_summary_text);
    let observer_policy_summary_checksum = checksum64(&observer_policy_summary_text);
    let apparentness_policy_summary_checksum = checksum64(&apparentness_policy_summary_text);
    let request_semantics_summary_checksum = checksum64(&request_semantics_summary_text);
    let unsupported_modes_summary_checksum = checksum64(&unsupported_modes_summary_text);
    let time_scale_policy_summary_checksum = checksum64(&time_scale_policy_summary_text);
    let utc_convenience_policy_summary_checksum = checksum64(&utc_convenience_policy_summary_text);
    let delta_t_policy_summary_checksum = checksum64(&delta_t_policy_summary_text);
    let native_sidereal_policy_summary_checksum = checksum64(&native_sidereal_policy_summary_text);
    let zodiac_policy_summary_checksum = checksum64(&zodiac_policy_summary_text);
    let lunar_theory_limitations_summary_checksum =
        checksum64(&lunar_theory_limitations_summary_text);
    let lunar_theory_source_selection_summary_checksum =
        checksum64(&lunar_theory_source_selection_summary_text);
    let lunar_theory_source_family_summary_checksum =
        checksum64(&lunar_theory_source_family_summary_text);
    let lunar_theory_catalog_validation_summary_checksum =
        checksum64(&lunar_theory_catalog_validation_summary_text);
    let request_surface_summary_checksum = checksum64(&request_surface_summary_text);
    let compatibility_caveats_summary_checksum = checksum64(&compatibility_caveats_summary_text);
    let workspace_audit_summary_checksum = checksum64(&workspace_audit_summary_text);
    let native_dependency_audit_summary_text = render_native_dependency_audit_summary()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let native_dependency_audit_summary_checksum =
        checksum64(&native_dependency_audit_summary_text);
    let artifact_summary_checksum = checksum64(&artifact_summary_text);
    let packaged_artifact_profile_coverage_summary_checksum =
        checksum64(&packaged_artifact_profile_coverage_summary_text);
    let packaged_artifact_access_summary_checksum =
        checksum64(&packaged_artifact_access_summary_text);
    let packaged_artifact_output_support_summary_checksum =
        checksum64(&packaged_artifact_output_support_summary_text);
    let packaged_artifact_fit_sample_classes_summary_checksum =
        checksum64(&packaged_artifact_fit_sample_classes_summary_text);
    let packaged_artifact_fit_threshold_violation_count_summary_checksum =
        checksum64(&packaged_artifact_fit_threshold_violation_count_summary_text);
    let packaged_artifact_fit_threshold_violations_summary_checksum =
        checksum64(&packaged_artifact_fit_threshold_violations_summary_text);
    let packaged_artifact_body_cadence_summary_checksum =
        checksum64(&packaged_artifact_body_cadence_summary_text);
    let packaged_artifact_body_class_span_cap_summary_checksum =
        checksum64(&packaged_artifact_body_class_span_cap_summary_text);
    let packaged_artifact_normalized_intermediate_summary_checksum =
        checksum64(&packaged_artifact_normalized_intermediate_summary_text);
    let packaged_artifact_speed_policy_summary_checksum =
        checksum64(&packaged_artifact_speed_policy_summary_text);
    let packaged_artifact_storage_summary_checksum =
        checksum64(&packaged_artifact_storage_summary_text);
    let packaged_artifact_production_profile_summary_checksum =
        checksum64(&packaged_artifact_production_profile_summary_text);
    let packaged_frame_treatment_summary_checksum =
        checksum64(&packaged_frame_treatment_summary_text);
    let packaged_artifact_target_threshold_summary_checksum =
        checksum64(&packaged_artifact_target_threshold_summary_text);
    let packaged_artifact_target_threshold_state_summary_checksum =
        checksum64(&packaged_artifact_target_threshold_state_summary_text);
    let packaged_artifact_source_fit_holdout_sync_summary_checksum =
        checksum64(&packaged_artifact_source_fit_holdout_sync_summary_text);
    let packaged_artifact_target_threshold_scope_envelopes_summary_checksum =
        checksum64(&packaged_artifact_target_threshold_scope_envelopes_summary_text);
    let packaged_artifact_phase2_corpus_alignment_summary_checksum =
        checksum64(&packaged_artifact_phase2_corpus_alignment_summary_text);
    let packaged_lookup_epoch_policy_summary_checksum =
        checksum64(&packaged_lookup_epoch_policy_summary_text);
    let packaged_artifact_generation_policy_summary_text =
        packaged_artifact_generation_policy_summary_for_report();
    let packaged_artifact_generation_policy_summary_checksum =
        checksum64(&packaged_artifact_generation_policy_summary_text);
    let packaged_artifact_generation_residual_bodies_summary_text =
        validated_packaged_artifact_generation_residual_bodies_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?;
    let packaged_artifact_generation_residual_bodies_summary_checksum =
        checksum64(&packaged_artifact_generation_residual_bodies_summary_text);
    let packaged_artifact_regeneration_summary_text =
        packaged_artifact_regeneration_summary_for_report();
    let packaged_artifact_regeneration_summary_checksum =
        checksum64(&packaged_artifact_regeneration_summary_text);
    let packaged_artifact_generation_manifest_checksum =
        checksum64(&packaged_artifact_generation_manifest_text);
    let packaged_artifact_generation_manifest_checksum_text =
        format!("0x{packaged_artifact_generation_manifest_checksum:016x}\n");
    let packaged_artifact_generation_manifest_checksum_summary_text =
        packaged_artifact_generation_manifest_checksum_for_report();
    let packaged_artifact_generation_manifest_checksum_summary_checksum =
        checksum64(&packaged_artifact_generation_manifest_checksum_summary_text);
    let packaged_artifact_generation_manifest_checksum_checksum =
        checksum64(&packaged_artifact_generation_manifest_checksum_text);
    let packaged_artifact_generation_manifest_summary_checksum =
        checksum64(&packaged_artifact_generation_manifest_summary_text);
    let benchmark_corpus_summary_checksum = checksum64(&benchmark_corpus_summary_text);
    let chart_benchmark_corpus_summary_checksum = checksum64(&chart_benchmark_corpus_summary_text);
    let benchmark_report_checksum = checksum64(&benchmark_report_text);
    let validation_report_checksum = checksum64(&validation_report_text);
    let packaged_artifact = packaged_artifact();
    let packaged_artifact_bytes = packaged_artifact_bytes();
    let packaged_artifact_bytes_checksum = checksum64_bytes(packaged_artifact_bytes);
    let packaged_artifact_checksum_text = format!("0x{:016x}\n", packaged_artifact.checksum);
    let packaged_artifact_checksum_text_checksum = checksum64(&packaged_artifact_checksum_text);
    let manifest_text = format!(
        "Release bundle manifest\nprofile: compatibility-profile.txt\nprofile checksum (fnv1a-64): 0x{compatibility_profile_checksum:016x}\nprofile summary: compatibility-profile-summary.txt\nprofile summary checksum (fnv1a-64): 0x{compatibility_profile_summary_checksum:016x}\nrelease notes: release-notes.txt\nrelease notes checksum (fnv1a-64): 0x{release_notes_checksum:016x}\nrelease notes summary: release-notes-summary.txt\nrelease notes summary checksum (fnv1a-64): 0x{release_notes_summary_checksum:016x}\nrelease summary: release-summary.txt\nrelease summary checksum (fnv1a-64): 0x{release_summary_checksum:016x}\nrelease-profile identifiers: release-profile-identifiers.txt\nrelease-profile identifiers checksum (fnv1a-64): 0x{release_profile_identifiers_checksum:016x}\nrelease-profile identifiers summary: release-profile-identifiers-summary.txt\nrelease-profile identifiers summary checksum (fnv1a-64): 0x{release_profile_identifiers_summary_checksum:016x}\nrelease-house-system-canonical-names summary: release-house-system-canonical-names-summary.txt\nrelease-house-system-canonical-names summary checksum (fnv1a-64): 0x{release_house_system_canonical_names_summary_checksum:016x}\nrelease-ayanamsa-canonical-names summary: release-ayanamsa-canonical-names-summary.txt\nrelease-ayanamsa-canonical-names summary checksum (fnv1a-64): 0x{release_ayanamsa_canonical_names_summary_checksum:016x}\nrelease-house-validation summary: release-house-validation-summary.txt\nrelease-house-validation summary checksum (fnv1a-64): 0x{release_house_validation_summary_checksum:016x}\ntarget-house-scope summary: target-house-scope-summary.txt\ntarget-house-scope summary checksum (fnv1a-64): 0x{target_house_scope_summary_checksum:016x}\ntarget-ayanamsa-scope summary: target-ayanamsa-scope-summary.txt\ntarget-ayanamsa-scope summary checksum (fnv1a-64): 0x{target_ayanamsa_scope_summary_checksum:016x}\nhouse code aliases summary: house-code-aliases-summary.txt\nhouse code aliases summary checksum (fnv1a-64): 0x{house_code_aliases_summary_checksum:016x}\nhouse formula families summary: house-formula-families-summary.txt\nhouse formula families summary checksum (fnv1a-64): 0x{house_formula_families_summary_checksum:016x}\nhouse latitude-sensitive summary: house-latitude-sensitive-summary.txt\nhouse latitude-sensitive summary checksum (fnv1a-64): 0x{house_latitude_sensitive_summary_checksum:016x}\nhouse latitude-sensitive constraints summary: house-latitude-sensitive-constraints-summary.txt\nhouse latitude-sensitive constraints summary checksum (fnv1a-64): 0x{house_latitude_sensitive_constraints_summary_checksum:016x}\nhouse latitude-sensitive failure-modes summary: house-latitude-sensitive-failure-modes-summary.txt\nhouse latitude-sensitive failure-modes summary checksum (fnv1a-64): 0x{house_latitude_sensitive_failure_modes_summary_checksum:016x}\nrelease checklist: release-checklist.txt\nrelease checklist checksum (fnv1a-64): 0x{release_checklist_checksum:016x}\nrelease checklist summary: release-checklist-summary.txt\nrelease checklist summary checksum (fnv1a-64): 0x{release_checklist_summary_checksum:016x}\nbackend matrix: backend-matrix.txt\nbackend matrix checksum (fnv1a-64): 0x{backend_matrix_checksum:016x}\nbackend matrix summary: backend-matrix-summary.txt\nbackend matrix summary checksum (fnv1a-64): 0x{backend_matrix_summary_checksum:016x}\napi stability posture: api-stability.txt\napi stability checksum (fnv1a-64): 0x{api_stability_checksum:016x}\napi stability summary: api-stability-summary.txt\napi stability summary checksum (fnv1a-64): 0x{api_stability_summary_checksum:016x}\ncomparison-corpus summary: comparison-corpus-summary.txt\ncomparison-corpus summary checksum (fnv1a-64): 0x{comparison_corpus_summary_checksum:016x}\nsource-corpus summary: source-corpus-summary.txt\nsource-corpus summary checksum (fnv1a-64): 0x{source_corpus_summary_checksum:016x}\njpl source posture summary: jpl-source-posture-summary.txt\njpl source posture summary checksum (fnv1a-64): 0x{jpl_source_posture_summary_checksum:016x}\njpl provenance-only evidence summary: jpl-provenance-only-summary.txt\njpl provenance-only evidence summary checksum (fnv1a-64): 0x{jpl_provenance_only_summary_checksum:016x}\ncomparison-snapshot summary: comparison-snapshot-summary.txt\ncomparison-snapshot summary checksum (fnv1a-64): 0x{comparison_snapshot_summary_checksum:016x}\ncomparison-snapshot source summary: comparison-snapshot-source-summary.txt\ncomparison-snapshot source summary checksum (fnv1a-64): 0x{comparison_snapshot_source_summary_checksum:016x}\ncomparison-snapshot source window summary: comparison-snapshot-source-window-summary.txt\ncomparison-snapshot source window summary checksum (fnv1a-64): 0x{comparison_snapshot_source_window_summary_checksum:016x}\ncomparison-snapshot body-class coverage summary: comparison-snapshot-body-class-coverage-summary.txt\ncomparison-snapshot body-class coverage summary checksum (fnv1a-64): 0x{comparison_snapshot_body_class_coverage_summary_checksum:016x}\ncomparison-snapshot manifest summary: comparison-snapshot-manifest-summary.txt\ncomparison-snapshot manifest summary checksum (fnv1a-64): 0x{comparison_snapshot_manifest_summary_checksum:016x}\ncomparison-envelope summary: comparison-envelope-summary.txt\ncomparison-envelope summary checksum (fnv1a-64): 0x{comparison_envelope_summary_checksum:016x}\ncomparison-body-class-tolerance summary: comparison-body-class-tolerance-summary.txt\ncomparison-body-class-tolerance summary checksum (fnv1a-64): 0x{comparison_body_class_tolerance_summary_checksum:016x}\ncomparison-body-class-error-envelope summary: comparison-body-class-error-envelope-summary.txt\ncomparison-body-class-error-envelope summary checksum (fnv1a-64): 0x{comparison_body_class_error_envelope_summary_checksum:016x}\ncomparison-corpus release-guard summary: comparison-corpus-release-guard-summary.txt\ncomparison-corpus release-guard summary checksum (fnv1a-64): 0x{comparison_corpus_release_guard_summary_checksum:016x}\nreference-holdout overlap summary: reference-holdout-overlap-summary.txt\nreference-holdout overlap summary checksum (fnv1a-64): 0x{reference_holdout_overlap_summary_checksum:016x}\nreference snapshot bridge day summary: reference-snapshot-bridge-day-summary.txt\nreference snapshot bridge day summary checksum (fnv1a-64): 0x{reference_snapshot_bridge_day_summary_checksum:016x}\nreference snapshot major-body boundary window summary: reference-snapshot-major-body-boundary-window-summary.txt\nreference snapshot major-body boundary window summary checksum (fnv1a-64): 0x{reference_snapshot_major_body_boundary_window_summary_checksum:016x}\nreference snapshot boundary epoch coverage summary: reference-snapshot-boundary-epoch-coverage-summary.txt\nreference snapshot boundary epoch coverage summary checksum (fnv1a-64): 0x{reference_snapshot_boundary_epoch_coverage_summary_checksum:016x}\nreference snapshot pre-bridge boundary summary: reference-snapshot-pre-bridge-boundary-summary.txt\nreference snapshot pre-bridge boundary summary checksum (fnv1a-64): 0x{reference_snapshot_pre_bridge_boundary_summary_checksum:016x}\nreference snapshot 2451917 major-body boundary summary: reference-snapshot-2451917-major-body-boundary-summary.txt\nreference snapshot 2451917 major-body boundary summary checksum (fnv1a-64): 0x{reference_snapshot_2451917_major_body_boundary_summary_checksum:016x}\nreference snapshot 2451918 major-body boundary summary: reference-snapshot-2451918-major-body-boundary-summary.txt\nreference snapshot 2451918 major-body boundary summary checksum (fnv1a-64): 0x{reference_snapshot_2451918_major_body_boundary_summary_checksum:016x}\nreference snapshot 2451919 major-body boundary summary: reference-snapshot-2451919-major-body-boundary-summary.txt\nreference snapshot 2451919 major-body boundary summary checksum (fnv1a-64): 0x{reference_snapshot_2451919_major_body_boundary_summary_checksum:016x}\nreference snapshot 2451916 major-body dense boundary summary: reference-snapshot-2451916-major-body-dense-boundary-summary.txt\nreference snapshot 2451916 major-body dense boundary summary checksum (fnv1a-64): 0x{reference_snapshot_2451916_major_body_dense_boundary_summary_checksum:016x}\nreference snapshot sparse boundary summary: reference-snapshot-sparse-boundary-summary.txt\nreference snapshot sparse boundary summary checksum (fnv1a-64): 0x{reference_snapshot_sparse_boundary_summary_checksum:016x}\nreference snapshot exact J2000 evidence summary: reference-snapshot-exact-j2000-evidence-summary.txt\nreference snapshot exact J2000 evidence summary checksum (fnv1a-64): 0x{reference_snapshot_exact_j2000_evidence_summary_checksum:016x}\nreference snapshot source summary: reference-snapshot-source-summary.txt\nreference snapshot source summary checksum (fnv1a-64): 0x{reference_snapshot_source_summary_checksum:016x}\nreference snapshot source window summary: reference-snapshot-source-window-summary.txt\nreference snapshot source window summary checksum (fnv1a-64): 0x{reference_snapshot_source_window_summary_checksum:016x}\nreference snapshot manifest summary: reference-snapshot-manifest-summary.txt\nreference snapshot manifest summary checksum (fnv1a-64): 0x{reference_snapshot_manifest_summary_checksum:016x}\nreference snapshot body-class coverage summary: reference-snapshot-body-class-coverage-summary.txt\nreference snapshot body-class coverage summary checksum (fnv1a-64): 0x{reference_snapshot_body_class_coverage_summary_checksum:016x}\nreference snapshot equatorial parity summary: reference-snapshot-equatorial-parity-summary.txt\nreference snapshot equatorial parity summary checksum (fnv1a-64): 0x{reference_snapshot_equatorial_parity_summary_checksum:016x}\nreference asteroid source window summary: reference-asteroid-source-window-summary.txt\nreference asteroid source window summary checksum (fnv1a-64): 0x{reference_asteroid_source_window_summary_checksum:016x}\nreference asteroid equatorial evidence summary: reference-asteroid-equatorial-evidence-summary.txt\nreference asteroid equatorial evidence summary checksum (fnv1a-64): 0x{reference_asteroid_equatorial_evidence_summary_checksum:016x}\nindependent-holdout source window summary: independent-holdout-source-window-summary.txt\nindependent-holdout source window summary checksum (fnv1a-64): 0x{independent_holdout_source_window_summary_checksum:016x}\nindependent-holdout equatorial parity summary: independent-holdout-equatorial-parity-summary.txt\nindependent-holdout equatorial parity summary checksum (fnv1a-64): 0x{independent_holdout_equatorial_parity_summary_checksum:016x}\nindependent-holdout body-class coverage summary: independent-holdout-body-class-coverage-summary.txt\nindependent-holdout body-class coverage summary checksum (fnv1a-64): 0x{independent_holdout_body_class_coverage_summary_checksum:016x}\nindependent-holdout quarter-day boundary summary: independent-holdout-quarter-day-boundary-summary.txt\nindependent-holdout quarter-day boundary summary checksum (fnv1a-64): 0x{independent_holdout_quarter_day_boundary_summary_checksum:016x}\nproduction generation boundary source summary: production-generation-boundary-source-summary.txt\nproduction generation boundary source summary checksum (fnv1a-64): 0x{production_generation_boundary_source_summary_checksum:016x}\nproduction generation boundary window summary: production-generation-boundary-window-summary.txt\nproduction generation boundary window summary checksum (fnv1a-64): 0x{production_generation_boundary_window_summary_checksum:016x}\nproduction generation boundary request corpus summary: production-generation-boundary-request-corpus-summary.txt\nproduction generation boundary request corpus summary checksum (fnv1a-64): 0x{production_generation_boundary_request_corpus_summary_checksum:016x}\nproduction generation boundary request corpus equatorial summary: production-generation-boundary-request-corpus-equatorial-summary.txt\nproduction generation boundary request corpus equatorial summary checksum (fnv1a-64): 0x{production_generation_boundary_request_corpus_equatorial_summary_checksum:016x}\nreference snapshot summary: reference-snapshot-summary.txt\nreference snapshot summary checksum (fnv1a-64): 0x{reference_snapshot_summary_checksum:016x}\nproduction generation summary: production-generation-summary.txt\nproduction generation summary checksum (fnv1a-64): 0x{production_generation_summary_checksum:016x}\nproduction generation body-class coverage summary: production-generation-body-class-coverage-summary.txt\nproduction generation body-class coverage summary checksum (fnv1a-64): 0x{production_generation_body_class_coverage_summary_checksum:016x}\nproduction generation source summary: production-generation-source-summary.txt\nproduction generation source summary checksum (fnv1a-64): 0x{production_generation_source_summary_checksum:016x}\nproduction generation source revision summary: production-generation-source-revision-summary.txt\nproduction generation source revision summary checksum (fnv1a-64): 0x{production_generation_source_revision_summary_checksum:016x}\nproduction generation source window summary: production-generation-source-window-summary.txt
production generation source window summary checksum (fnv1a-64): 0x{production_generation_source_window_summary_checksum:016x}
production generation quarter-day boundary summary: production-generation-quarter-day-boundary-summary.txt
production generation quarter-day boundary summary checksum (fnv1a-64): 0x{production_generation_quarter_day_boundary_summary_checksum:016x}
production generation corpus shape summary: production-generation-corpus-shape-summary.txt\nproduction generation corpus shape summary checksum (fnv1a-64): 0x{production_generation_corpus_shape_summary_checksum:016x}\nproduction generation manifest summary: production-generation-manifest-summary.txt\nproduction generation manifest summary checksum (fnv1a-64): 0x{production_generation_manifest_summary_checksum:016x}\nproduction generation manifest checksum summary: production-generation-manifest-checksum-summary.txt\nproduction generation manifest checksum summary checksum (fnv1a-64): 0x{production_generation_manifest_checksum_checksum:016x}\ncatalog inventory summary: catalog-inventory-summary.txt\ncatalog inventory summary checksum (fnv1a-64): 0x{catalog_inventory_summary_checksum:016x}\ncatalog posture summary: catalog-posture-summary.txt\ncatalog posture summary checksum (fnv1a-64): 0x{catalog_posture_summary_checksum:016x}\ncustom-definition ayanamsa labels summary: custom-definition-ayanamsa-labels-summary.txt\ncustom-definition ayanamsa labels summary checksum (fnv1a-64): 0x{custom_definition_ayanamsa_labels_summary_checksum:016x}\nayanamsa provenance summary: ayanamsa-provenance-summary.txt\nayanamsa provenance summary checksum (fnv1a-64): 0x{ayanamsa_provenance_summary_checksum:016x}\nvalidation report summary: validation-report-summary.txt\nvalidation report summary checksum (fnv1a-64): 0x{validation_report_summary_checksum:016x}\nworkspace provenance summary: workspace-provenance-summary.txt\nworkspace provenance summary checksum (fnv1a-64): 0x{workspace_provenance_summary_checksum:016x}\nrelease body claims summary: release-body-claims-summary.txt
release body claims summary checksum (fnv1a-64): 0x{release_body_claims_summary_checksum:016x}
body/date/channel claims summary: body-date-channel-claims-summary.txt
body/date/channel claims summary checksum (fnv1a-64): 0x{body_date_channel_claims_summary_checksum:016x}
pluto fallback summary: pluto-fallback-summary.txt
pluto fallback summary checksum (fnv1a-64): 0x{pluto_fallback_summary_checksum:016x}
request policy summary: request-policy-summary.txt
request policy summary checksum (fnv1a-64): 0x{request_policy_summary_checksum:016x}
observer policy summary: observer-policy-summary.txt
observer policy summary checksum (fnv1a-64): 0x{observer_policy_summary_checksum:016x}
apparentness policy summary: apparentness-policy-summary.txt
apparentness policy summary checksum (fnv1a-64): 0x{apparentness_policy_summary_checksum:016x}
request-semantics summary: request-semantics-summary.txt\nrequest-semantics summary checksum (fnv1a-64): 0x{request_semantics_summary_checksum:016x}\nunsupported-modes summary: unsupported-modes-summary.txt\nunsupported-modes summary checksum (fnv1a-64): 0x{unsupported_modes_summary_checksum:016x}\ntime-scale policy summary: time-scale-policy-summary.txt\ntime-scale policy summary checksum (fnv1a-64): 0x{time_scale_policy_summary_checksum:016x}\nutc-convenience policy summary: utc-convenience-policy-summary.txt\nutc-convenience policy summary checksum (fnv1a-64): 0x{utc_convenience_policy_summary_checksum:016x}\ndelta-t policy summary: delta-t-policy-summary.txt\ndelta-t policy summary checksum (fnv1a-64): 0x{delta_t_policy_summary_checksum:016x}\nnative sidereal policy summary: native-sidereal-policy-summary.txt\nnative sidereal policy summary checksum (fnv1a-64): 0x{native_sidereal_policy_summary_checksum:016x}\nzodiac policy summary: zodiac-policy-summary.txt\nzodiac policy summary checksum (fnv1a-64): 0x{zodiac_policy_summary_checksum:016x}\nlunar theory limitations summary: lunar-theory-limitations-summary.txt
lunar theory limitations summary checksum (fnv1a-64): 0x{lunar_theory_limitations_summary_checksum:016x}
lunar theory source selection summary: lunar-theory-source-selection-summary.txt
lunar theory source selection summary checksum (fnv1a-64): 0x{lunar_theory_source_selection_summary_checksum:016x}
lunar theory source family summary: lunar-theory-source-family-summary.txt
lunar theory source family summary checksum (fnv1a-64): 0x{lunar_theory_source_family_summary_checksum:016x}
lunar theory source window summary: lunar-source-window-summary.txt
lunar theory source window summary checksum (fnv1a-64): 0x{lunar_source_window_summary_checksum:016x}
lunar reference error envelope summary: lunar-reference-error-envelope-summary.txt
lunar reference error envelope summary checksum (fnv1a-64): 0x{lunar_reference_error_envelope_summary_checksum:016x}
lunar equatorial reference error envelope summary: lunar-equatorial-reference-error-envelope-summary.txt
lunar equatorial reference error envelope summary checksum (fnv1a-64): 0x{lunar_equatorial_reference_error_envelope_summary_checksum:016x}
lunar apparent comparison summary: lunar-apparent-comparison-summary.txt
lunar apparent comparison summary checksum (fnv1a-64): 0x{lunar_apparent_comparison_summary_checksum:016x}
lunar theory catalog validation summary: lunar-theory-catalog-validation-summary.txt
lunar theory catalog validation summary checksum (fnv1a-64): 0x{lunar_theory_catalog_validation_summary_checksum:016x}
request surface summary: request-surface-summary.txt\nrequest surface summary checksum (fnv1a-64): 0x{request_surface_summary_checksum:016x}\ncompatibility caveats summary: compatibility-caveats-summary.txt\ncompatibility caveats summary checksum (fnv1a-64): 0x{compatibility_caveats_summary_checksum:016x}\nworkspace audit summary: workspace-audit-summary.txt\nworkspace audit summary checksum (fnv1a-64): 0x{workspace_audit_summary_checksum:016x}\nnative-dependency audit summary: native-dependency-audit-summary.txt\nnative-dependency audit summary checksum (fnv1a-64): 0x{native_dependency_audit_summary_checksum:016x}\nartifact summary: artifact-summary.txt\nartifact summary checksum (fnv1a-64): 0x{artifact_summary_checksum:016x}\npackaged-artifact: packaged-artifact.bin\npackaged-artifact checksum (fnv1a-64): 0x{packaged_artifact_bytes_checksum:016x}\npackaged-artifact checksum sidecar: packaged-artifact.checksum.txt\npackaged-artifact checksum sidecar checksum (fnv1a-64): 0x{packaged_artifact_checksum_text_checksum:016x}\npackaged-artifact profile coverage summary: packaged-artifact-profile-coverage-summary.txt\npackaged-artifact profile coverage summary checksum (fnv1a-64): 0x{packaged_artifact_profile_coverage_summary_checksum:016x}\npackaged-artifact access summary: packaged-artifact-access-summary.txt\npackaged-artifact access summary checksum (fnv1a-64): 0x{packaged_artifact_access_summary_checksum:016x}\npackaged-artifact output support summary: packaged-artifact-output-support-summary.txt\npackaged-artifact output support summary checksum (fnv1a-64): 0x{packaged_artifact_output_support_summary_checksum:016x}\npackaged-artifact fit sample classes summary: packaged-artifact-fit-sample-classes-summary.txt\npackaged-artifact fit sample classes summary checksum (fnv1a-64): 0x{packaged_artifact_fit_sample_classes_summary_checksum:016x}\npackaged-artifact fit threshold violation count summary: packaged-artifact-fit-threshold-violation-count-summary.txt\npackaged-artifact fit threshold violation count summary checksum (fnv1a-64): 0x{packaged_artifact_fit_threshold_violation_count_summary_checksum:016x}\npackaged-artifact fit threshold violations summary: packaged-artifact-fit-threshold-violations-summary.txt\npackaged-artifact fit threshold violations summary checksum (fnv1a-64): 0x{packaged_artifact_fit_threshold_violations_summary_checksum:016x}\npackaged-artifact normalized intermediate summary: packaged-artifact-normalized-intermediate-summary.txt\npackaged-artifact normalized intermediate summary checksum (fnv1a-64): 0x{packaged_artifact_normalized_intermediate_summary_checksum:016x}\npackaged-artifact speed policy summary: packaged-artifact-speed-policy-summary.txt\npackaged-artifact speed policy summary checksum (fnv1a-64): 0x{packaged_artifact_speed_policy_summary_checksum:016x}\npackaged-artifact storage summary: packaged-artifact-storage-summary.txt\npackaged-artifact storage summary checksum (fnv1a-64): 0x{packaged_artifact_storage_summary_checksum:016x}\npackaged-artifact production-profile summary: packaged-artifact-production-profile-summary.txt\npackaged-artifact production-profile summary checksum (fnv1a-64): 0x{packaged_artifact_production_profile_summary_checksum:016x}\npackaged-frame-treatment summary: packaged-frame-treatment-summary.txt\npackaged-frame-treatment summary checksum (fnv1a-64): 0x{packaged_frame_treatment_summary_checksum:016x}\npackaged-artifact target-threshold summary: packaged-artifact-target-threshold-summary.txt
packaged-artifact target-threshold summary checksum (fnv1a-64): 0x{packaged_artifact_target_threshold_summary_checksum:016x}
packaged-artifact target-threshold state summary: packaged-artifact-target-threshold-state-summary.txt
packaged-artifact target-threshold state summary checksum (fnv1a-64): 0x{packaged_artifact_target_threshold_state_summary_checksum:016x}
packaged-artifact source-fit and hold-out sync summary: packaged-artifact-source-fit-holdout-sync-summary.txt
packaged-artifact source-fit and hold-out sync summary checksum (fnv1a-64): 0x{packaged_artifact_source_fit_holdout_sync_summary_checksum:016x}
packaged-artifact target-threshold scope envelopes summary: packaged-artifact-target-threshold-scope-envelopes-summary.txt
packaged-artifact target-threshold scope envelopes summary checksum (fnv1a-64): 0x{packaged_artifact_target_threshold_scope_envelopes_summary_checksum:016x}
packaged-artifact phase-2 corpus alignment summary: packaged-artifact-phase2-corpus-alignment-summary.txt
packaged-artifact phase-2 corpus alignment summary checksum (fnv1a-64): 0x{packaged_artifact_phase2_corpus_alignment_summary_checksum:016x}
packaged-artifact lookup-epoch policy summary: packaged-lookup-epoch-policy-summary.txt
packaged-artifact lookup-epoch policy summary checksum (fnv1a-64): 0x{packaged_lookup_epoch_policy_summary_checksum:016x}
packaged-artifact generation policy summary: packaged-artifact-generation-policy-summary.txt
packaged-artifact generation policy summary checksum (fnv1a-64): 0x{packaged_artifact_generation_policy_summary_checksum:016x}
packaged-artifact generation residual bodies summary: packaged-artifact-generation-residual-bodies-summary.txt
packaged-artifact generation residual bodies summary checksum (fnv1a-64): 0x{packaged_artifact_generation_residual_bodies_summary_checksum:016x}
packaged-artifact regeneration summary: packaged-artifact-regeneration-summary.txt
packaged-artifact regeneration summary checksum (fnv1a-64): 0x{packaged_artifact_regeneration_summary_checksum:016x}
packaged-artifact generation manifest: packaged-artifact-generation-manifest.txt
packaged-artifact generation manifest checksum (fnv1a-64): 0x{packaged_artifact_generation_manifest_checksum:016x}
packaged-artifact generation manifest checksum sidecar: packaged-artifact-generation-manifest.checksum.txt
packaged-artifact generation manifest checksum sidecar checksum (fnv1a-64): 0x{packaged_artifact_generation_manifest_checksum_checksum:016x}
packaged-artifact generation manifest summary: packaged-artifact-generation-manifest-summary.txt
packaged-artifact generation manifest summary checksum (fnv1a-64): 0x{packaged_artifact_generation_manifest_summary_checksum:016x}
packaged-artifact generation manifest checksum summary: packaged-artifact-generation-manifest-checksum-summary.txt
packaged-artifact generation manifest checksum summary checksum (fnv1a-64): 0x{packaged_artifact_generation_manifest_checksum_summary_checksum:016x}
benchmark-corpus summary: benchmark-corpus-summary.txt\nbenchmark-corpus summary checksum (fnv1a-64): 0x{benchmark_corpus_summary_checksum:016x}\nchart-benchmark-corpus summary: chart-benchmark-corpus-summary.txt\nchart-benchmark-corpus summary checksum (fnv1a-64): 0x{chart_benchmark_corpus_summary_checksum:016x}\nselected asteroid source request corpus summary: selected-asteroid-source-request-corpus-summary.txt\nselected asteroid source request corpus summary checksum (fnv1a-64): 0x{selected_asteroid_source_request_corpus_summary_checksum:016x}\nselected asteroid source request corpus equatorial summary: selected-asteroid-source-request-corpus-equatorial-summary.txt\nselected asteroid source request corpus equatorial summary checksum (fnv1a-64): 0x{selected_asteroid_source_request_corpus_equatorial_summary_checksum:016x}\nselected asteroid source window summary: selected-asteroid-source-window-summary.txt\nselected asteroid source window summary checksum (fnv1a-64): 0x{selected_asteroid_source_window_summary_checksum:016x}\ninterpolation-quality sample request corpus summary: interpolation-quality-request-corpus-summary.txt\ninterpolation-quality sample request corpus summary checksum (fnv1a-64): 0x{interpolation_quality_request_corpus_summary_checksum:016x}\nbenchmark report: benchmark-report.txt\nbenchmark report checksum (fnv1a-64): 0x{benchmark_report_checksum:016x}\nvalidation report: validation-report.txt\nvalidation report checksum (fnv1a-64): 0x{validation_report_checksum:016x}\nsource revision: {}\nworkspace status: {}\nrustc version: {}\ncargo version: {}\nprofile id: {}\napi stability posture id: {}\nvalidation rounds: {}\n",
        provenance.source_revision,
        provenance.workspace_status,
        provenance.rustc_version,
        provenance.cargo_version,
        current_compatibility_profile().profile_id,
        current_api_stability_profile().profile_id,
        rounds,
    );

    fs::write(&profile_path, profile_text.as_bytes())?;
    fs::write(&profile_summary_path, profile_summary_text.as_bytes())?;
    fs::write(&release_notes_path, release_notes_text.as_bytes())?;
    fs::write(
        &release_notes_summary_path,
        release_notes_summary_text.as_bytes(),
    )?;
    fs::write(&release_summary_path, release_summary_text.as_bytes())?;
    fs::write(
        &release_profile_identifiers_path,
        release_profile_identifiers_text.as_bytes(),
    )?;
    fs::write(
        &release_profile_identifiers_summary_path,
        release_profile_identifiers_summary_text.as_bytes(),
    )?;
    fs::write(
        &release_house_system_canonical_names_summary_path,
        release_house_system_canonical_names_summary_text.as_bytes(),
    )?;
    fs::write(
        &release_ayanamsa_canonical_names_summary_path,
        release_ayanamsa_canonical_names_summary_text.as_bytes(),
    )?;
    fs::write(
        &release_house_validation_summary_path,
        release_house_validation_summary_text.as_bytes(),
    )?;
    fs::write(
        &target_house_scope_summary_path,
        target_house_scope_summary_text.as_bytes(),
    )?;
    fs::write(
        &target_ayanamsa_scope_summary_path,
        target_ayanamsa_scope_summary_text.as_bytes(),
    )?;
    fs::write(
        &house_code_aliases_summary_path,
        house_code_aliases_summary_text.as_bytes(),
    )?;
    fs::write(
        &house_formula_families_summary_path,
        house_formula_families_summary_text.as_bytes(),
    )?;
    fs::write(
        &house_latitude_sensitive_summary_path,
        house_latitude_sensitive_summary_text.as_bytes(),
    )?;
    fs::write(
        &house_latitude_sensitive_constraints_summary_path,
        house_latitude_sensitive_constraints_summary_text.as_bytes(),
    )?;
    fs::write(
        &house_latitude_sensitive_failure_modes_summary_path,
        house_latitude_sensitive_failure_modes_summary_text.as_bytes(),
    )?;
    fs::write(
        &backend_matrix_summary_path,
        backend_matrix_summary_text.as_bytes(),
    )?;
    fs::write(&release_checklist_path, release_checklist_text.as_bytes())?;
    fs::write(
        &release_checklist_summary_path,
        release_checklist_summary_text.as_bytes(),
    )?;
    fs::write(&backend_matrix_path, backend_matrix_text.as_bytes())?;
    fs::write(&api_stability_path, api_stability_text.as_bytes())?;
    fs::write(
        &api_stability_summary_path,
        api_stability_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_corpus_summary_path,
        comparison_corpus_summary_text.as_bytes(),
    )?;
    fs::write(
        &source_corpus_summary_path,
        source_corpus_summary_text.as_bytes(),
    )?;
    fs::write(
        output_dir.join("jpl-source-posture-summary.txt"),
        jpl_source_posture_summary_text.as_bytes(),
    )?;
    fs::write(
        output_dir.join("jpl-provenance-only-summary.txt"),
        jpl_provenance_only_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_snapshot_summary_path,
        comparison_snapshot_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_snapshot_source_summary_path,
        comparison_snapshot_source_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_snapshot_source_window_summary_path,
        comparison_snapshot_source_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_snapshot_body_class_coverage_summary_path,
        comparison_snapshot_body_class_coverage_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_snapshot_manifest_summary_path,
        comparison_snapshot_manifest_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_envelope_summary_path,
        comparison_envelope_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_body_class_tolerance_summary_path,
        comparison_body_class_tolerance_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_body_class_error_envelope_summary_path,
        comparison_body_class_error_envelope_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_corpus_release_guard_summary_path,
        comparison_corpus_release_guard_summary_text.as_bytes(),
    )?;
    fs::write(
        &comparison_corpus_guard_summary_path,
        comparison_corpus_release_guard_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_holdout_overlap_summary_path,
        reference_holdout_overlap_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_bridge_day_summary_path,
        reference_snapshot_bridge_day_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_major_body_boundary_window_summary_path,
        reference_snapshot_major_body_boundary_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_boundary_epoch_coverage_summary_path,
        reference_snapshot_boundary_epoch_coverage_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_pre_bridge_boundary_summary_path,
        reference_snapshot_pre_bridge_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_2451917_major_body_boundary_summary_path,
        reference_snapshot_2451917_major_body_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_2451918_major_body_boundary_summary_path,
        reference_snapshot_2451918_major_body_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_2451919_major_body_boundary_summary_path,
        reference_snapshot_2451919_major_body_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_2451916_major_body_dense_boundary_summary_path,
        reference_snapshot_2451916_major_body_dense_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_sparse_boundary_summary_path,
        reference_snapshot_sparse_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_exact_j2000_evidence_summary_path,
        reference_snapshot_exact_j2000_evidence_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_source_summary_path,
        reference_snapshot_source_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_source_window_summary_path,
        reference_snapshot_source_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_manifest_summary_path,
        reference_snapshot_manifest_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_body_class_coverage_summary_path,
        reference_snapshot_body_class_coverage_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_equatorial_parity_summary_path,
        reference_snapshot_equatorial_parity_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_asteroid_source_window_summary_path,
        reference_asteroid_source_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_asteroid_equatorial_evidence_summary_path,
        reference_asteroid_equatorial_evidence_summary_text.as_bytes(),
    )?;
    fs::write(
        &independent_holdout_source_window_summary_path,
        independent_holdout_source_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &independent_holdout_equatorial_parity_summary_path,
        independent_holdout_equatorial_parity_summary_text.as_bytes(),
    )?;
    fs::write(
        &independent_holdout_body_class_coverage_summary_path,
        independent_holdout_body_class_coverage_summary_text.as_bytes(),
    )?;
    fs::write(
        &independent_holdout_quarter_day_boundary_summary_path,
        independent_holdout_quarter_day_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_boundary_source_summary_path,
        production_generation_boundary_source_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_boundary_window_summary_path,
        production_generation_boundary_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_boundary_request_corpus_summary_path,
        production_generation_boundary_request_corpus_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_boundary_request_corpus_equatorial_summary_path,
        production_generation_boundary_request_corpus_equatorial_summary_text.as_bytes(),
    )?;
    fs::write(
        &reference_snapshot_summary_path,
        reference_snapshot_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_summary_path,
        production_generation_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_body_class_coverage_summary_path,
        production_generation_body_class_coverage_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_source_summary_path,
        production_generation_source_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_source_revision_summary_path,
        production_generation_source_revision_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_source_window_summary_path,
        production_generation_source_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_quarter_day_boundary_summary_path,
        production_generation_quarter_day_boundary_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_corpus_shape_summary_path,
        production_generation_corpus_shape_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_manifest_summary_path,
        production_generation_manifest_summary_text.as_bytes(),
    )?;
    fs::write(
        &production_generation_manifest_checksum_path,
        production_generation_manifest_checksum_text.as_bytes(),
    )?;
    fs::write(
        &catalog_inventory_summary_path,
        catalog_inventory_summary_text.as_bytes(),
    )?;
    fs::write(
        &catalog_posture_summary_path,
        catalog_posture_summary_text.as_bytes(),
    )?;
    fs::write(
        &custom_definition_ayanamsa_labels_summary_path,
        custom_definition_ayanamsa_labels_summary_text.as_bytes(),
    )?;
    fs::write(
        &ayanamsa_provenance_summary_path,
        ayanamsa_provenance_summary_text.as_bytes(),
    )?;
    fs::write(
        &validation_report_summary_path,
        validation_report_summary_text.as_bytes(),
    )?;
    fs::write(
        &workspace_provenance_summary_path,
        workspace_provenance_summary_text.as_bytes(),
    )?;
    fs::write(
        &release_body_claims_summary_path,
        release_body_claims_summary_text.as_bytes(),
    )?;
    let body_date_channel_claims_summary_path =
        output_dir.join("body-date-channel-claims-summary.txt");
    fs::write(
        &body_date_channel_claims_summary_path,
        body_date_channel_claims_summary_text.as_bytes(),
    )?;
    fs::write(
        &pluto_fallback_summary_path,
        pluto_fallback_summary_text.as_bytes(),
    )?;
    fs::write(
        &request_policy_summary_path,
        request_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &observer_policy_summary_path,
        observer_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &apparentness_policy_summary_path,
        apparentness_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &request_semantics_summary_path,
        request_semantics_summary_text.as_bytes(),
    )?;
    fs::write(
        &unsupported_modes_summary_path,
        unsupported_modes_summary_text.as_bytes(),
    )?;
    fs::write(
        &time_scale_policy_summary_path,
        time_scale_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &utc_convenience_policy_summary_path,
        utc_convenience_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &delta_t_policy_summary_path,
        delta_t_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &native_sidereal_policy_summary_path,
        native_sidereal_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &zodiac_policy_summary_path,
        zodiac_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_theory_limitations_summary_path,
        lunar_theory_limitations_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_theory_source_selection_summary_path,
        lunar_theory_source_selection_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_theory_source_family_summary_path,
        lunar_theory_source_family_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_source_window_summary_path,
        lunar_source_window_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_reference_error_envelope_summary_path,
        lunar_reference_error_envelope_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_equatorial_reference_error_envelope_summary_path,
        lunar_equatorial_reference_error_envelope_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_apparent_comparison_summary_path,
        lunar_apparent_comparison_summary_text.as_bytes(),
    )?;
    fs::write(
        &lunar_theory_catalog_validation_summary_path,
        lunar_theory_catalog_validation_summary_text.as_bytes(),
    )?;
    fs::write(
        &request_surface_summary_path,
        request_surface_summary_text.as_bytes(),
    )?;
    fs::write(
        &compatibility_caveats_summary_path,
        compatibility_caveats_summary_text.as_bytes(),
    )?;
    fs::write(
        &workspace_audit_summary_path,
        workspace_audit_summary_text.as_bytes(),
    )?;
    fs::write(
        &native_dependency_audit_summary_path,
        native_dependency_audit_summary_text.as_bytes(),
    )?;
    let manifest_text = format!(
        "{manifest_text}packaged-artifact body cadence summary: packaged-artifact-body-cadence-summary.txt\npackaged-artifact body cadence summary checksum (fnv1a-64): 0x{packaged_artifact_body_cadence_summary_checksum:016x}\npackaged-artifact body-class span cap summary: packaged-artifact-body-class-span-cap-summary.txt\npackaged-artifact body-class span cap summary checksum (fnv1a-64): 0x{packaged_artifact_body_class_span_cap_summary_checksum:016x}\n"
    );
    let manifest_checksum = checksum64(&manifest_text);
    let manifest_checksum_text = format!("0x{manifest_checksum:016x}\n");
    fs::write(&artifact_summary_path, artifact_summary_text.as_bytes())?;
    fs::write(&packaged_artifact_path, packaged_artifact_bytes)?;
    fs::write(
        &packaged_artifact_checksum_path,
        packaged_artifact_checksum_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_profile_coverage_summary_path,
        packaged_artifact_profile_coverage_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_access_summary_path,
        packaged_artifact_access_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_output_support_summary_path,
        packaged_artifact_output_support_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_fit_sample_classes_summary_path,
        packaged_artifact_fit_sample_classes_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_fit_threshold_violation_count_summary_path,
        packaged_artifact_fit_threshold_violation_count_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_fit_threshold_violations_summary_path,
        packaged_artifact_fit_threshold_violations_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_body_cadence_summary_path,
        packaged_artifact_body_cadence_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_body_class_span_cap_summary_path,
        packaged_artifact_body_class_span_cap_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_normalized_intermediate_summary_path,
        packaged_artifact_normalized_intermediate_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_speed_policy_summary_path,
        packaged_artifact_speed_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_storage_summary_path,
        packaged_artifact_storage_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_production_profile_summary_path,
        packaged_artifact_production_profile_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_frame_treatment_summary_path,
        packaged_frame_treatment_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_target_threshold_summary_path,
        packaged_artifact_target_threshold_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_target_threshold_state_summary_path,
        packaged_artifact_target_threshold_state_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_source_fit_holdout_sync_summary_path,
        packaged_artifact_source_fit_holdout_sync_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_target_threshold_scope_envelopes_summary_path,
        packaged_artifact_target_threshold_scope_envelopes_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_phase2_corpus_alignment_summary_path,
        packaged_artifact_phase2_corpus_alignment_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_lookup_epoch_policy_summary_path,
        packaged_lookup_epoch_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_generation_policy_summary_path,
        packaged_artifact_generation_policy_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_generation_residual_bodies_summary_path,
        packaged_artifact_generation_residual_bodies_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_regeneration_summary_path,
        packaged_artifact_regeneration_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_generation_manifest_path,
        packaged_artifact_generation_manifest_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_generation_manifest_summary_path,
        packaged_artifact_generation_manifest_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_generation_manifest_checksum_summary_path,
        packaged_artifact_generation_manifest_checksum_summary_text.as_bytes(),
    )?;
    fs::write(
        &packaged_artifact_generation_manifest_checksum_path,
        packaged_artifact_generation_manifest_checksum_text.as_bytes(),
    )?;
    fs::write(
        &benchmark_corpus_summary_path,
        benchmark_corpus_summary_text.as_bytes(),
    )?;
    fs::write(
        &chart_benchmark_corpus_summary_path,
        chart_benchmark_corpus_summary_text.as_bytes(),
    )?;
    fs::write(
        &interpolation_quality_request_corpus_summary_path,
        interpolation_quality_request_corpus_summary_text.as_bytes(),
    )?;
    fs::write(
        &selected_asteroid_source_request_corpus_summary_path,
        selected_asteroid_source_request_corpus_summary_text.as_bytes(),
    )?;
    fs::write(
        &selected_asteroid_source_request_corpus_equatorial_summary_path,
        selected_asteroid_source_request_corpus_equatorial_summary_text.as_bytes(),
    )?;
    fs::write(
        &selected_asteroid_source_window_summary_path,
        selected_asteroid_source_window_summary_text.as_bytes(),
    )?;
    fs::write(&benchmark_report_path, benchmark_report_text.as_bytes())?;
    fs::write(&report_path, validation_report_text.as_bytes())?;
    fs::write(&manifest_path, manifest_text.as_bytes())?;
    fs::write(&manifest_checksum_path, manifest_checksum_text.as_bytes())?;

    verify_release_bundle_internal(output_dir, false)
}
