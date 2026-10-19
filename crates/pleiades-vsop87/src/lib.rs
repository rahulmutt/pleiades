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
//! # Truncated series inside 1600–2600
//!
//! [`Vsop87Backend`], the default, evaluates truncated VSOP87B series for
//! instants in 1600-01-01..2600-01-01 TT (start inclusive, end exclusive; the
//! window is applied to the request's Julian day whatever its scale, TT or
//! TDB). A term is dropped when `|A|·0.6^α < 1e-8`, where `A` is its amplitude
//! and `α` the power of Julian millennia from J2000.0 multiplying its series.
//! The window spans T ∈ [−0.4, +0.6] millennia, so 0.6 is the largest |T| in
//! it (the 2600 edge) and a dropped term contributes less than 1e-8 radians or
//! AU anywhere inside it. Measured against the full series on a 200,000-instant
//! scan of the window, the largest drift in any body is under 0.3″: Mars is the
//! worst, at 0.26″ in longitude near 2508, then Venus at 0.17″; Pluto moves by
//! about 0.001″ through the Earth it subtracts. Each apparent sample is
//! 2.2–9.5× cheaper per body, and a Sun–Pluto set 3.4× (measured 2026-10-19).
//! Inside the window the VSOP87B bodies report
//! [`QualityAnnotation::Approximate`](pleiades_backend::QualityAnnotation::Approximate).
//! Outside the window the default backend evaluates the full series, bit for
//! bit as [`Vsop87FullSeriesBackend`] does, so its output steps at 1600-01-01
//! and 2600-01-01 by under 0.1″ (measured 0.085″, Venus, at 1600; 0.013″ at
//! 2600).
//!
//! [`Vsop87FullSeriesBackend`] evaluates every term everywhere: it reproduces
//! the published series exactly and has no edge step, at the cost of speed
//! inside the window.
//!
//! ```rust
//! use pleiades_backend::{EphemerisBackend, EphemerisRequest};
//! use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
//! use pleiades_vsop87::{Vsop87Backend, Vsop87FullSeriesBackend};
//!
//! let default = Vsop87Backend::new();
//! let full = Vsop87FullSeriesBackend::new();
//! let at = |jd: f64| {
//!     EphemerisRequest::new(
//!         CelestialBody::Mars,
//!         Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
//!     )
//! };
//!
//! // Outside 1600..2600 the two backends agree exactly.
//! let before = at(625_673.5);
//! assert_eq!(
//!     default.position(&before).unwrap().ecliptic,
//!     full.position(&before).unwrap().ecliptic,
//! );
//!
//! // Inside the window they differ, by well under 0.3 arcseconds.
//! let inside = at(2_451_545.0);
//! let d = default.position(&inside).unwrap().ecliptic.unwrap();
//! let f = full.position(&inside).unwrap().ecliptic.unwrap();
//! assert_ne!(d.longitude, f.longitude);
//! ```
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
//! Pluto, which the VSOP87 theory excludes, comes from the Meeus Table 37.A
//! periodic-term fit over 1885–2099 (arcsecond class) and from mean Keplerian
//! orbital elements outside that window, minus the same VSOP87B Earth.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod backend;
mod elements;
mod pluto;
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

pub use backend::{Vsop87Backend, Vsop87FullSeriesBackend};

#[cfg(test)]
mod tests;
