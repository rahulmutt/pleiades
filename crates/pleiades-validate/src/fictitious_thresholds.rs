//! Measured-basis ceilings for the fictitious-body SE-parity gate, mirroring
//! the measured-basis convention of `eclipse_local_thresholds` /
//! `rise_trans_thresholds`. Each global constant is set to ~1.4× the observed
//! maximum residual over the 18 non-Nibiru bodies of the committed corpus
//! (Task 8), rounded clean, not guessed.
//!
//! Measured per-body maxima (committed corpus, 570 rows = 19 bodies × 30
//! samples, geometric geocentric J2000 vs `swe_calc`
//! `SEFLG_MOSEPH|SEFLG_J2000|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL`):
//! every one of the 18 non-Nibiru bodies matches SE to a fraction of an
//! arcsecond in longitude/latitude and a few microAU in distance — the
//! largest being NeptuneLeverrier at 0.459″ longitude, Vulcan at 0.059″
//! latitude, and NeptuneAdams at 2.35e-6 AU distance. See
//! `crate::fictitious_validation`'s module doc and
//! `data/fictitious-corpus/MANIFEST.md` for the full per-body table.
//!
//! **Nibiru** (SE body 49, `seorbel.txt` equinox ~370 AD — about 1630 years
//! from J2000) is the documented outlier: its osculating elements are carried
//! to the J2000 mean ecliptic through the same IAU-1976 precession helper
//! (`pleiades_fict::frame::rotate_ecliptic_to_j2000`) used by every other
//! body, well outside that model's most accurate range. Its residual (1.262″
//! longitude, 0.710″ latitude, 6.14e-6 AU) is still small in absolute terms
//! but measurably larger than the other 18 bodies (~2.7× longitude, ~12×
//! latitude, ~2.6× distance) — a real, explained precession-extrapolation
//! effect, not a bug. Per Task 8 Step 6, it is gated under its own wider
//! per-body ceilings (`NIBIRU_*` below, same ~1.4× convention applied to
//! Nibiru's own measured maxima) instead of being allowed to inflate the
//! global ceilings that gate the other 18 bodies.

/// Max ecliptic-longitude residual vs SE, arcseconds, for the 18 non-Nibiru
/// bodies. Measured max 0.459″ (NeptuneLeverrier).
pub const LONGITUDE_ARCSEC: f64 = 0.65;
/// Max ecliptic-latitude residual vs SE, arcseconds, for the 18 non-Nibiru
/// bodies. Measured max 0.059″ (Vulcan).
pub const LATITUDE_ARCSEC: f64 = 0.09;
/// Max radial-distance residual vs SE, AU, for the 18 non-Nibiru bodies.
/// Measured max 2.35e-6 AU (NeptuneAdams).
pub const DISTANCE_AU: f64 = 3.5e-6;

/// Nibiru-only per-body ceilings (see the module doc for why Nibiru gets a
/// carve-out instead of inflating the globals above). ~1.4× Nibiru's own
/// measured maxima.
/// Measured Nibiru max 1.262″ longitude.
pub const NIBIRU_LONGITUDE_ARCSEC: f64 = 1.8;
/// Measured Nibiru max 0.710″ latitude.
pub const NIBIRU_LATITUDE_ARCSEC: f64 = 1.0;
/// Measured Nibiru max 6.14e-6 AU distance.
pub const NIBIRU_DISTANCE_AU: f64 = 9.0e-6;
