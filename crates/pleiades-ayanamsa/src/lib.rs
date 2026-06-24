//! Ayanamsa catalog definitions and compatibility metadata.
//!
//! This crate currently focuses on the catalog layer: it enumerates the
//! baseline built-in ayanamsas, their common aliases, and notes about their
//! intended interoperability role. It also carries the baseline epoch/offset
//! metadata used by the chart-layer sidereal conversion helper, plus a first
//! stage-6 batch of historical anchor-point variants so the release profile can
//! distinguish baseline coverage from broader compatibility breadth.
//!
//! # Examples
//!
//! ```
//! use pleiades_ayanamsa::{baseline_ayanamsas, resolve_ayanamsa};
//!
//! let catalog = baseline_ayanamsas();
//! assert!(catalog.iter().any(|entry| entry.canonical_name == "Lahiri"));
//!
//! assert_eq!(resolve_ayanamsa("KP"), Some(pleiades_types::Ayanamsa::Krishnamurti));
//! assert_eq!(resolve_ayanamsa("Krishnamurti Paddhati"), Some(pleiades_types::Ayanamsa::Krishnamurti));
//! assert_eq!(resolve_ayanamsa("Krishnamurti ayanamsa"), Some(pleiades_types::Ayanamsa::Krishnamurti));
//! ```

#![forbid(unsafe_code)]

mod catalog;
mod lookup;
mod model;
mod precession;
pub mod thresholds;

// Re-export model types at the crate root to preserve the public API surface.
pub use model::{
    AyanamsaCatalogValidationError, AyanamsaCatalogValidationSummary,
    AyanamsaCatalogValidationSummaryValidationError, AyanamsaDescriptor, AyanamsaMetadataCoverage,
    AyanamsaMetadataCoverageValidationError, AyanamsaProvenanceExample, AyanamsaProvenanceSummary,
    AyanamsaProvenanceSummaryValidationError,
};

// Re-export lookup functions at the crate root to preserve the public API surface.
pub use lookup::{
    ayanamsa_catalog_validation_summary, baseline_ayanamsas, built_in_ayanamsas,
    custom_definition_ayanamsa_labels, custom_definition_example_ayanamsa_labels, descriptor,
    metadata_coverage, provenance_sample_ayanamsas, provenance_summary,
    reference_offset_sample_ayanamsas, release_ayanamsas, resolve_ayanamsa, sidereal_offset,
    validate_ayanamsa_catalog, validated_provenance_summary_for_report,
};

#[cfg(test)]
mod tests;
