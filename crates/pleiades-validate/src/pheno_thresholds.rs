//! Provisional SP-5 pheno gate ceilings. Pinned from measured residuals in
//! Task 7 (SP-4 method: ~1.4× measured maxima). See the SP-5 plan §Gate.

/// Phase-angle residual vs Swiss Ephemeris, arcsec.
pub const PHASE_ANGLE_ARCSEC: f64 = 3600.0;
/// Illuminated-fraction residual (absolute).
pub const PHASE_FRACTION_ABS: f64 = 1e-2;
/// Elongation residual, arcsec.
pub const ELONGATION_ARCSEC: f64 = 3600.0;
/// Apparent-diameter residual, arcsec.
pub const DIAMETER_ARCSEC: f64 = 60.0;
/// Apparent-magnitude residual, all bodies except Saturn.
pub const MAGNITUDE_ABS: f64 = 0.5;
/// Apparent-magnitude residual for Saturn (ring term is the widest).
pub const SATURN_MAGNITUDE_ABS: f64 = 1.0;
