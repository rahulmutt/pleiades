//! SE-parity ceilings for the `validate-occultations` gate (SP-6). Each is set
//! from the measured per-metric residual maximum × ~1.4, matching the
//! `pheno_thresholds`/`rise_trans_thresholds` convention. Provisional until
//! Task 11 measures the real residuals against the committed corpus.

/// Contact/maximum instant residual vs SE, seconds (well-conditioned).
pub const CONTACT_SECONDS: f64 = 6.0;
/// Contact/maximum instant residual near grazing/limb chords, seconds.
pub const CONTACT_SECONDS_GRAZING: f64 = 30.0;
/// Covered-diameter fraction (magnitude) residual.
pub const MAGNITUDE_ABS: f64 = 0.01;
/// Covered-area fraction (obscuration) residual.
pub const OBSCURATION_ABS: f64 = 0.01;
/// Sub-lunar point residual (great-circle), arcminutes.
pub const SUBLUNAR_ARCMIN: f64 = 6.0;
