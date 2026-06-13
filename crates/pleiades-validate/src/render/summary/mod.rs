//! Policy, corpus, and evidence summary text rendering for the validation tool.

mod artifact;
mod backend;
mod policy;
mod release;
mod report;
mod writers;

pub(crate) use artifact::*;
pub(crate) use backend::*;
pub(crate) use policy::*;
pub(crate) use release::*;
pub(crate) use report::*;
pub(crate) use writers::*;

pub use backend::{
    render_api_stability_summary, render_backend_matrix_report, render_backend_matrix_summary,
};
pub use release::{
    current_request_surface_summary, render_release_profile_identifiers_summary,
    RequestSurfaceSummary,
};
