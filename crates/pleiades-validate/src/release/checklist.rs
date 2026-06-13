//! Release checklist posture summary and catalog constants.

use std::fmt;

use pleiades_core::{
    current_release_profile_identifiers, EphemerisError, EphemerisErrorKind,
    ReleaseProfileIdentifiers,
};

const RELEASE_CHECKLIST_REPOSITORY_MANAGED_RELEASE_GATES: [&str; 10] = [
    "[x] cargo fmt --all --check",
    "[x] cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "[x] cargo test --workspace",
    "[x] cargo run -q -p pleiades-validate -- workspace-audit",
    "[x] cargo run -q -p pleiades-validate -- release-smoke",
    "[x] cargo run -q -p pleiades-validate -- verify-compatibility-profile",
    "[x] cargo run -q -p pleiades-validate -- validate-artifact",
    "[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release",
    "[x] cargo run -q -p pleiades-validate -- benchmark --rounds 5",
    "[x] cargo run -q -p pleiades-validate -- report --rounds 5",
];

const RELEASE_CHECKLIST_MANUAL_BUNDLE_WORKFLOW: [&str; 4] = [
    "[x] cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release",
    "[x] cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release",
    "[x] docs/release-reproducibility.md",
    "[x] docs/release-reproducibility.md (broader source-corpus provenance contract)",
];

const RELEASE_CHECKLIST_BUNDLE_CONTENTS: [&str; 25] = [
    "[x] compatibility-profile.txt",
    "[x] compatibility-profile-summary.txt",
    "[x] release-notes.txt",
    "[x] release-notes-summary.txt",
    "[x] release-summary.txt",
    "[x] release-checklist.txt",
    "[x] release-checklist-summary.txt",
    "[x] house-formula-families-summary.txt",
    "[x] house-latitude-sensitive-summary.txt",
    "[x] house-latitude-sensitive-constraints-summary.txt",
    "[x] house-latitude-sensitive-failure-modes-summary.txt",
    "[x] backend-matrix.txt",
    "[x] backend-matrix-summary.txt",
    "[x] api-stability.txt",
    "[x] api-stability-summary.txt",
    "[x] validation-report-summary.txt",
    "[x] workspace-audit-summary.txt",
    "[x] validation-report.txt",
    "[x] lunar-reference-error-envelope-summary.txt",
    "[x] lunar-equatorial-reference-error-envelope-summary.txt",
    "[x] lunar-apparent-comparison-summary.txt",
    "[x] production-generation-boundary-window-summary.txt",
    "[x] bundle-manifest.txt",
    "[x] bundle-manifest.checksum.txt",
    "[x] verify-release-bundle",
];

const RELEASE_CHECKLIST_EXTERNAL_PUBLISHING_REMINDERS: [&str; 3] = [
    "[ ] tag and archive the release commit",
    "[ ] publish or attach the release bundle outside the workspace",
    "[ ] review any documented compatibility gaps before announcing the release",
];

pub(crate) fn release_checklist_repository_managed_release_gates() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_REPOSITORY_MANAGED_RELEASE_GATES
}

pub(crate) fn release_checklist_manual_bundle_workflow() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_MANUAL_BUNDLE_WORKFLOW
}

pub(crate) fn release_checklist_bundle_contents() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_BUNDLE_CONTENTS
}

pub(crate) fn release_checklist_external_publishing_reminders() -> &'static [&'static str] {
    &RELEASE_CHECKLIST_EXTERNAL_PUBLISHING_REMINDERS
}

/// A compact summary of the release checklist posture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseChecklistSummary {
    /// Release-profile identifiers used by the checklist.
    pub release_profile_identifiers: ReleaseProfileIdentifiers,
    /// Number of repository-managed release gates.
    pub repository_managed_release_gates: usize,
    /// Number of manual bundle workflow items.
    pub manual_bundle_workflow_items: usize,
    /// Number of bundle content items.
    pub bundle_contents_items: usize,
    /// Number of external publishing reminders.
    pub external_publishing_reminders: usize,
}

impl ReleaseChecklistSummary {
    /// Returns `Ok(())` when the compact summary still matches the release posture.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        self.release_profile_identifiers
            .validate()
            .map_err(|error| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                    "release checklist summary release-profile identifiers are invalid: {error}"
                ),
                )
            })?;

        let repository_managed_release_gates =
            release_checklist_repository_managed_release_gates().len();
        if self.repository_managed_release_gates != repository_managed_release_gates {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary repository-managed release gate count mismatch: expected {}, found {}",
                    repository_managed_release_gates, self.repository_managed_release_gates
                ),
            ));
        }

        let manual_bundle_workflow = release_checklist_manual_bundle_workflow().len();
        if self.manual_bundle_workflow_items != manual_bundle_workflow {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary manual bundle workflow count mismatch: expected {}, found {}",
                    manual_bundle_workflow, self.manual_bundle_workflow_items
                ),
            ));
        }

        let bundle_contents = release_checklist_bundle_contents().len();
        if self.bundle_contents_items != bundle_contents {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary bundle contents count mismatch: expected {}, found {}",
                    bundle_contents, self.bundle_contents_items
                ),
            ));
        }

        let external_publishing_reminders = release_checklist_external_publishing_reminders().len();
        if self.external_publishing_reminders != external_publishing_reminders {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "release checklist summary external publishing reminder count mismatch: expected {}, found {}",
                    external_publishing_reminders, self.external_publishing_reminders
                ),
            ));
        }

        Ok(())
    }

    /// Returns the compact summary used in release-facing reports.
    pub fn summary_line(&self) -> String {
        format!(
            "v{} compatibility={}, api-stability={}; repository-managed release gates={}; manual bundle workflow={}; bundle contents={}; external publishing reminders={}",
            ReleaseProfileIdentifiers::schema_version(),
            self.release_profile_identifiers.compatibility_profile_id,
            self.release_profile_identifiers.api_stability_profile_id,
            self.repository_managed_release_gates,
            self.manual_bundle_workflow_items,
            self.bundle_contents_items,
            self.external_publishing_reminders,
        )
    }

    /// Returns a compact summary line after validating the release posture.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReleaseChecklistSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the compact release-checklist summary derived from the release posture.
pub fn release_checklist_summary() -> ReleaseChecklistSummary {
    ReleaseChecklistSummary {
        release_profile_identifiers: current_release_profile_identifiers(),
        repository_managed_release_gates: release_checklist_repository_managed_release_gates()
            .len(),
        manual_bundle_workflow_items: release_checklist_manual_bundle_workflow().len(),
        bundle_contents_items: release_checklist_bundle_contents().len(),
        external_publishing_reminders: release_checklist_external_publishing_reminders().len(),
    }
}
