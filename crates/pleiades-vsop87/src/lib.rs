//! Formula-based planetary backend boundary built around VSOP87-style series
//! evaluation, low-precision orbital elements, and geocentric coordinate
//! transforms.
//!
//! This crate now provides a working pure-Rust algorithmic backend for the Sun
//! and major planets. The Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus,
//! and Neptune paths evaluate public IMCCE VSOP87B sources (heliocentric
//! spherical variables, J2000 ecliptic/equinox) transformed to geocentric
//! chart-facing coordinates. The Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus,
//! and Neptune paths now use generated binary tables derived from their vendored
//! source files. A maintainer-facing regeneration helper and
//! `regenerate-vsop87b-tables` binary keep those checked-in blobs reproducible
//! from the public source text. The backend accepts both TT and TDB requests as
//! dynamical-time inputs and still rejects UT-based requests explicitly. It
//! exposes mean-only tropical geocentric ecliptic/equatorial results and rejects
//! topocentric observer or apparent-place requests with structured errors. The source-backed
//! and fallback body-profile helpers are public so reproducibility tooling can reuse
//! the backend-owned catalog partition directly.
//!
//! The source-documentation helpers also expose the current public-file,
//! variant, frame, units, reduction, transform note, truncation policy, and
//! date-range fields as typed records. That makes the mixed generated-binary and
//! fallback provenance auditable without scraping the release text.
//!
//! ```rust
//! use pleiades_vsop87::{source_documentation_summary, source_specifications};
//! use pleiades_types::CelestialBody;
//!
//! let specs = source_specifications();
//! let summary = source_documentation_summary();
//!
//! assert_eq!(specs.len(), 8);
//! assert_eq!(summary.source_specification_count, specs.len());
//! assert_eq!(summary.generated_binary_profile_count, 8);
//! assert_eq!(summary.fallback_bodies, vec![CelestialBody::Pluto]);
//! assert!(specs.iter().all(|spec| spec.variant == "VSOP87B"));
//! assert!(specs.iter().all(|spec| spec.frame == "J2000 ecliptic/equinox"));
//! assert!(specs.iter().all(|spec| spec.units == "degrees and astronomical units"));
//! assert!(specs.iter().all(|spec| spec.truncation_policy.contains("generated binary coefficient table")));
//! assert!(specs.iter().all(|spec| spec.date_range.contains("J2000 canonical reference sample")));
//! ```
//!
//! Pluto still uses compact
//! Keplerian orbital elements,
//! a geocentric reduction step, and central-difference motion estimates so the
//! workspace has an end-to-end tropical chart path while the remaining Pluto-
//! specific source selection is added incrementally.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod backend;
mod elements;
mod profiles;
mod tables;
mod transforms;

// Imports and constants needed by the test module (via `use super::*`).
// Gated behind `#[cfg(test)]` to avoid "unused" warnings in non-test builds.
#[cfg(test)]
use pleiades_backend::{
    AccuracyClass, Apparentness, EphemerisBackend, EphemerisErrorKind, EphemerisRequest,
    QualityAnnotation,
};
#[cfg(test)]
use pleiades_types::{
    CelestialBody, CoordinateFrame, Instant, Latitude, Longitude, TimeScale, ZodiacMode,
};
#[cfg(test)]
use profiles::body_catalog_entries;
#[cfg(test)]
use transforms::signed_longitude_delta_degrees;

#[cfg(test)]
const PACKAGE_NAME: &str = "pleiades-vsop87";
#[cfg(test)]
const BACKEND_LABEL: &str = "the VSOP87 backend";
#[cfg(test)]
const J1900: f64 = 2_415_020.0;
#[cfg(test)]
const J2000: f64 = 2_451_545.0;

pub use profiles::{
    body_source_profiles, fallback_body_profiles, source_backed_body_order,
    source_backed_body_profiles, Vsop87BodySource, Vsop87BodySourceKind,
    Vsop87BodySourceValidationError,
};

mod source_docs;
pub use source_docs::*;

pub use backend::Vsop87Backend;

#[cfg(test)]
mod tests;
