//! Relocated reference-snapshot renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot` (report-surface
//! relocation program, Slice D). Mirrors jpl's own submodule split
//! (`core`, `boundaries`) — `boundaries` is copied in a later Slice D task.

pub(crate) mod core;

#[allow(unused_imports)]
pub(crate) use core::*;
