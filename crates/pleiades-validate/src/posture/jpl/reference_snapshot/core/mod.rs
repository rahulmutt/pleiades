//! Relocated reference-snapshot core renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::core`
//! (report-surface relocation program, Slice D). `general_a`/`general_b` are
//! copied in later Slice D tasks (8b/8c).

pub(crate) mod coverage;
pub(crate) mod evidence;
pub(crate) mod parity;

#[allow(unused_imports)]
pub(crate) use coverage::*;
#[allow(unused_imports)]
pub(crate) use evidence::*;
#[allow(unused_imports)]
pub(crate) use parity::*;
