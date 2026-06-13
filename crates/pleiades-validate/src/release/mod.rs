//! Release-facing summaries: checklist, bundle, workspace audit, and notes.

mod checklist;
mod workspace_audit;

pub(crate) use checklist::{
    release_checklist_bundle_contents, release_checklist_external_publishing_reminders,
    release_checklist_manual_bundle_workflow, release_checklist_repository_managed_release_gates,
};
pub use checklist::{release_checklist_summary, ReleaseChecklistSummary};
pub(crate) use workspace_audit::render_workspace_audit_summary_text;
pub use workspace_audit::{
    workspace_audit_summary, WorkspaceAuditReport, WorkspaceAuditSummary, WorkspaceAuditViolation,
};
