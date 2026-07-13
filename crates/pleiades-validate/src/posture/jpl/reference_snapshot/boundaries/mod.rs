//! Relocated reference-snapshot boundary renderers copied from
//! `pleiades-jpl::reference_summary::reference_snapshot::boundaries`
//! (report-surface relocation program, Slice D). Mirrors jpl's own
//! `era_a`/`era_b`/`era_c`/`era_d` submodule split — `era_d` is copied in a
//! later Slice D task (9c).

pub(crate) mod era_a;
pub(crate) mod era_b;
pub(crate) mod era_c;

#[allow(unused_imports)]
pub(crate) use era_a::*;
#[allow(unused_imports)]
pub(crate) use era_b::*;
#[allow(unused_imports)]
pub(crate) use era_c::*;
