//! Full and truncated VSOP87B series, loaded lazily from the generated tables.
//!
//! The truncated copy drops terms whose largest possible contribution inside
//! the supported epoch window is negligible; see
//! `docs/superpowers/specs/2026-10-18-vsop87-series-truncation-design.md`.

use std::sync::OnceLock;

use super::vsop87b_earth::{
    evaluate, parse_generated_vsop87b_tables, SphericalLbr, Vsop87SeriesTables, Vsop87Term,
};

/// Which copy of the VSOP87B series to evaluate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SeriesFidelity {
    /// Every term of the published series.
    Full,
    /// Terms below [`TRUNCATION_THRESHOLD`] over the supported window removed.
    Truncated,
}

/// First instant (inclusive) at which the default backend evaluates the
/// truncated series: 1600-01-01 00:00 TT.
pub(crate) const TRUNCATED_WINDOW_START_JD: f64 = 2_305_447.5;

/// End (exclusive) of the default backend's truncated window:
/// 2600-01-01 00:00 TT.
pub(crate) const TRUNCATED_WINDOW_END_JD: f64 = 2_670_690.5;

/// The fidelity the default backend uses at `julian_day_tt` (the request's
/// Julian day whatever its scale, TT or TDB, which differ by milliseconds):
/// truncated inside `[TRUNCATED_WINDOW_START_JD, TRUNCATED_WINDOW_END_JD)`,
/// full outside it, where the dropped terms grow as `(|T| / t_max)^alpha`.
pub(crate) fn default_fidelity_at(julian_day_tt: f64) -> SeriesFidelity {
    if (TRUNCATED_WINDOW_START_JD..TRUNCATED_WINDOW_END_JD).contains(&julian_day_tt) {
        SeriesFidelity::Truncated
    } else {
        SeriesFidelity::Full
    }
}

/// Smallest kept term amplitude, in the series' own units (radians or AU),
/// evaluated at the edge of the supported window.
pub(crate) const TRUNCATION_THRESHOLD: f64 = 1e-8;

/// Largest |T| inside the window, in Julian millennia from J2000.0:
/// 1600–2600 CE is T ∈ [−0.4, +0.6], so 0.6 is the 2600 edge. Lowering it
/// would let the dropped terms exceed [`TRUNCATION_THRESHOLD`] near 2600.
pub(crate) const TRUNCATION_T_MAX_MILLENNIA: f64 = 0.6;

/// Keeps a term when `|amplitude| * t_max^alpha >= threshold`, where `alpha`
/// is the term's power of T (its index in the per-variable vector). Empty
/// power vectors are retained so the power index stays aligned.
pub(crate) fn truncate_tables(
    full: &Vsop87SeriesTables,
    threshold: f64,
    t_max_millennia: f64,
) -> Vsop87SeriesTables {
    let truncate = |variable: &[Vec<Vsop87Term>]| -> Vec<Vec<Vsop87Term>> {
        variable
            .iter()
            .enumerate()
            .map(|(alpha, terms)| {
                let scale = t_max_millennia.powi(alpha as i32);
                terms
                    .iter()
                    .filter(|term| term.amplitude.abs() * scale >= threshold)
                    .copied()
                    .collect()
            })
            .collect()
    };
    Vsop87SeriesTables {
        longitude: truncate(&full.longitude),
        latitude: truncate(&full.latitude),
        radius: truncate(&full.radius),
    }
}

/// One planet's embedded series with lazily built full and truncated tables.
pub(crate) struct Vsop87SeriesSource {
    bytes: &'static [u8],
    full: OnceLock<Vsop87SeriesTables>,
    truncated: OnceLock<Vsop87SeriesTables>,
}

impl Vsop87SeriesSource {
    pub(crate) const fn new(bytes: &'static [u8]) -> Self {
        Self {
            bytes,
            full: OnceLock::new(),
            truncated: OnceLock::new(),
        }
    }

    /// The tables for `fidelity`. The truncated copy is derived from the full
    /// tables, so asking for it also keeps the full tables resident.
    pub(crate) fn tables(&self, fidelity: SeriesFidelity) -> &Vsop87SeriesTables {
        let full = self
            .full
            .get_or_init(|| parse_generated_vsop87b_tables(self.bytes));
        match fidelity {
            SeriesFidelity::Full => full,
            SeriesFidelity::Truncated => self.truncated.get_or_init(|| {
                truncate_tables(full, TRUNCATION_THRESHOLD, TRUNCATION_T_MAX_MILLENNIA)
            }),
        }
    }

    pub(crate) fn lbr(&self, julian_day_tt: f64, fidelity: SeriesFidelity) -> SphericalLbr {
        // VSOP87 uses Julian millennia from J2000.0.
        let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
        let tables = self.tables(fidelity);
        SphericalLbr {
            longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
                .rem_euclid(core::f64::consts::TAU),
            latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
            radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
        }
    }
}
