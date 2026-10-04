//! Embedded planetary coefficient table modules.
//!
//! Each submodule holds the per-planet coefficient data and the parsing/evaluation
//! functions derived from the vendored public IMCCE/CELMECH VSOP87B source files.
//! These modules are kept verbatim so that regeneration tooling continues to
//! round-trip without reformatting.
//! `pluto_meeus` holds Meeus Table 37.A for Pluto, which the VSOP87 files exclude.

pub(crate) mod pluto_meeus;
pub(crate) mod vsop87b_earth;
pub(crate) mod vsop87b_jupiter;
pub(crate) mod vsop87b_mars;
pub(crate) mod vsop87b_mercury;
pub(crate) mod vsop87b_neptune;
pub(crate) mod vsop87b_saturn;
pub(crate) mod vsop87b_uranus;
pub(crate) mod vsop87b_venus;
