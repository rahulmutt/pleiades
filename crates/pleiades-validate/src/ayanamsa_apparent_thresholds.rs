//! Ceilings for `validate-ayanamsa-apparent` (issue #164 (c)): pleiades'
//! apparent-star correction (`pleiades_core::apparent_star_ayanamsa_correction`)
//! against Swiss Ephemeris's apparent − geometric ayanamsa, 1040 rows.
//!
//! Sized `ceil(1.4 × measured max)` to 0.01″, measured 2026-10-07 with
//! `measure_ayanamsa_apparent_maxima`: uniform rows 0.0106″, conjunction rows
//! 0.4925″ (δ Cnc behind the solar disc, where the Meeus Sun's ~0.01° error is
//! large against the star's 278″ closest approach).
//!
//! Both maxima are δ Cnc rows (True Pushya and True Sheoran share the anchor):
//! uniform at JD 2456415.303333, conjunction at JD 2475133.560438. Every
//! other star's conjunction rows stay within 0.044″ (ζ Psc), the rest within
//! 0.01″. The five galactic-equator modes Swiss Ephemeris does not aberrate
//! agree exactly (both sides 0).

/// Rows spread over 1900–2100 (aberration-dominated). Target 0.10″.
pub(crate) const UNIFORM_CEILING_ARCSEC: f64 = 0.02; // measured max 0.0106"
/// Rows within a day of the anchor star's solar conjunction.
pub(crate) const CONJUNCTION_CEILING_ARCSEC: f64 = 0.69; // measured max 0.4925"
