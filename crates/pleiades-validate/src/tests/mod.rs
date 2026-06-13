//! White-box unit tests for the validate crate.
//!
//! The former 17k-line `tests.rs` monolith was fanned out into per-module test
//! files (declared below) plus a shared `test_support` module. Each child reuses
//! the crate-root scope via `use super::*` and the shared fixtures via
//! `use super::test_support::*`.

use super::*;

mod test_support;

mod comparison;
mod compatibility;
mod corpus;
mod release_bundle_verify_a;
mod release_bundle_verify_b;
mod release_checklist;
mod release_workspace_audit;
mod render_catalog;
mod render_packaged_artifact;
mod render_request;
mod report;
mod snapshot_render;
