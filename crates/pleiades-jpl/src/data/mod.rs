//! Embedded selected-asteroid source-evidence modules.
//!
//! These modules carry the checked-in selected-asteroid source slices and the
//! release-facing summaries derived from them. They are kept whole and isolated
//! from the backend logic so the embedded source slices continue to round-trip.

mod selected_asteroid_2001;
mod selected_asteroid_2378498;

pub use selected_asteroid_2001::{
    selected_asteroid_source_2451917_summary, selected_asteroid_source_2451917_summary_for_report,
    SelectedAsteroidSource2451917Summary,
};
pub use selected_asteroid_2378498::{
    selected_asteroid_source_2378498_summary, selected_asteroid_source_2378498_summary_for_report,
    SelectedAsteroidSource2378498Summary,
};
