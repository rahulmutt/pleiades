//! Measured-basis ceilings for the `validate-pheno` SE-parity gate
//! (`swe_pheno`: phase angle, illuminated fraction, elongation, apparent
//! diameter, apparent magnitude), mirroring the measured-basis convention of
//! `nod_aps_thresholds` / `fictitious_thresholds` / `eclipse_local_thresholds`
//! / `lilith_validation`. Pinned in Task 7 from the actual per-metric residual
//! maxima measured over the committed 80-row pheno corpus (2026-07-07), each
//! ceiling set to ~1.4× the observed maximum, rounded up to a clean value —
//! replacing the Task 6 provisional (deliberately generous) placeholders. See
//! the SP-5 plan §Gate.
//!
//! All six measured maxima are sub-arcminute / sub-hundredth-magnitude — no
//! metric is anomalous (the sanity trigger, magnitude > ~0.2 for a
//! non-Saturn body, does not fire), so no coefficient audit against `swecl.c`
//! was needed before pinning.

/// Phase-angle residual vs Swiss Ephemeris, arcsec. Measured max 57.791329″
/// (Mercury) on 2026-07-07 corpus; ceiling ~1.4×.
pub const PHASE_ANGLE_ARCSEC: f64 = 85.0;
/// Illuminated-fraction residual (absolute). Measured max 0.00012207 on
/// 2026-07-07 corpus; ceiling ~1.4×.
pub const PHASE_FRACTION_ABS: f64 = 2e-4;
/// Elongation residual, arcsec. Measured max 20.966209″ (Uranus) on
/// 2026-07-07 corpus; ceiling ~1.4×.
pub const ELONGATION_ARCSEC: f64 = 30.0;
/// Apparent-diameter residual, arcsec. Measured max 0.192600″ (Moon) on
/// 2026-07-07 corpus; ceiling ~1.4×.
pub const DIAMETER_ARCSEC: f64 = 0.3;
/// Apparent-magnitude residual, all bodies except Saturn. Measured max
/// 0.002293 (Mercury) on 2026-07-07 corpus; ceiling ~1.4×.
pub const MAGNITUDE_ABS: f64 = 0.004;
/// Apparent-magnitude residual for Saturn (ring term is the widest).
/// Measured max 0.000418 on 2026-07-07 corpus; ceiling ~1.4×.
pub const SATURN_MAGNITUDE_ABS: f64 = 0.0006;
