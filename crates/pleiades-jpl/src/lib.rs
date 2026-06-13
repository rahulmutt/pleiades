//! JPL Horizons reference fixture backend for validation, comparison, and
//! selected asteroid support.
//!
//! This crate provides a narrow, source-backed backend based on a checked-in
//! JPL Horizons vector fixture. The backend serves exact states at fixture
//! epochs and uses cubic interpolation on four-sample windows when it can,
//! falling back to quadratic interpolation on three-sample windows and linear
//! interpolation between adjacent samples when fewer fixture points are
//! available. The checked-in ecliptic fixture can also be rotated into a
//! mean-obliquity equatorial frame for chart requests that prefer equatorial
//! output. This intentionally small derivative format proves the pure-Rust
//! reader/interpolator path before larger public JPL-derived corpora are added.
//!
//! The checked-in fixture also includes a small set of named asteroids and a
//! custom `catalog:designation` example so the shared body taxonomy can exercise
//! source-backed asteroid support without changing the comparison corpus used by
//! validation reports. Split manifest/row corpus parsing is available both for
//! in-memory text and for path-backed file inputs so corpus-generation tooling
//! can keep provenance and rows in separate checked artifacts when needed.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use pleiades_types::{
    Apparentness, CustomBodyId, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    TimeScale, ZodiacMode,
};

mod data;
pub use data::{
    selected_asteroid_source_2378498_summary, selected_asteroid_source_2378498_summary_for_report,
    selected_asteroid_source_2451917_summary, selected_asteroid_source_2451917_summary_for_report,
};

mod production_generation;
pub use production_generation::*;
use production_generation::{
    production_generation_boundary_body_list, production_generation_boundary_entries,
    production_generation_snapshot_bodies, production_generation_snapshot_body_list,
    PRODUCTION_GENERATION_BOUNDARY_COVERAGE, PRODUCTION_GENERATION_QUARTER_DAY_EPOCHS,
};

const REFERENCE_EPOCH_JD: f64 = 2_451_545.0;
const AU_IN_KM: f64 = 149_597_870.7;

mod reference_summary;
mod requests;
mod snapshot;
mod spk;
pub use spk::{
    generate_corpus_csv, CorpusRequest, SpkBackend, SpkBackendBuilder, SpkError, SpkErrorKind,
};
pub use reference_summary::*;
pub use requests::*;
pub use snapshot::*;

use reference_summary::format_bodies;

fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

mod backend;
pub use backend::*;
use backend::{
    angular_degrees_delta, comparison_body_list, comparison_snapshot_entries,
    has_surrounding_whitespace, independent_holdout_bodies, independent_holdout_snapshot_error,
    interpolation_quality_sample_list, is_comparison_body, is_reference_asteroid,
    reference_asteroid_equatorial_evidence_list, reference_asteroid_evidence_list,
    reference_asteroid_list, reference_asteroid_requests_with_frame_selector,
    resolve_fixture_state, snapshot_bodies, snapshot_entries, snapshot_instants,
    validate_snapshot_manifest_footprint, validate_snapshot_manifest_header_structure,
    REFERENCE_SNAPSHOT_1500_SELECTED_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_1600_SELECTED_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_1749_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_1750_SELECTED_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_1800_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_1900_SELECTED_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2200_SELECTED_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2400000_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451545_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451910_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451911_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451912_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451913_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451914_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451915_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451916_MAJOR_BODY_INTERIOR_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451917_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451917_MAJOR_BODY_BRIDGE_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451919_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2451920_MAJOR_BODY_INTERIOR_EPOCH_JD,
    REFERENCE_SNAPSHOT_2453000_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2500000_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2500_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2500_SELECTED_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_2600000_MAJOR_BODY_BOUNDARY_EPOCH_JD,
    REFERENCE_SNAPSHOT_REFERENCE_ONLY_EPOCH_JD,
};

#[cfg(test)]
mod test_support;
