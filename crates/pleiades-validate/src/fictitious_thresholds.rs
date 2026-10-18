//! Measured-basis ceilings for the fictitious-body SE-parity gate, mirroring
//! the measured-basis convention of `eclipse_local_thresholds` /
//! `rise_trans_thresholds`. Each constant is set to ~1.4× the observed maximum
//! residual over all 19 bodies of the committed corpus, rounded clean, not
//! guessed.
//!
//! Measured per-body maxima (committed corpus, 570 rows = 19 bodies × 30
//! samples, geometric geocentric J2000 vs `swe_calc`
//! `SEFLG_MOSEPH|SEFLG_J2000|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL`), as
//! re-measured on 2026-10-18 after the element equinoxes moved to Vondrák 2011
//! precession: Vulcan sets the angular maxima at 0.054″ longitude and 0.059″
//! latitude; every other body is at most 0.0017″ (NeptuneLeverrier), Nibiru
//! 0.0001″. The largest distance residual is 2.71e-7 AU (PlutoLowell). See
//! `crate::fictitious_validation`'s module doc and
//! `data/fictitious-corpus/MANIFEST.md`.
//!
//! **Nibiru** (SE body 49, `seorbel.txt` equinox ~370 CE) no longer has its
//! own ceilings. Its former 1.262″ longitude / 0.710″ latitude residual was the
//! IAU-1976 precession its elements went through, against SE's Vondrák 2011;
//! the same error put the J1900 bodies at up to 0.459″.

/// Max ecliptic-longitude residual vs SE, arcseconds. Measured max 0.054″
/// (Vulcan).
pub const LONGITUDE_ARCSEC: f64 = 0.075;
/// Max ecliptic-latitude residual vs SE, arcseconds. Measured max 0.059″
/// (Vulcan).
pub const LATITUDE_ARCSEC: f64 = 0.085;
/// Max radial-distance residual vs SE, AU. Measured max 2.71e-7 AU
/// (PlutoLowell).
pub const DISTANCE_AU: f64 = 3.8e-7;
