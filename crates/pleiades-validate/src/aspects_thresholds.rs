//! Ceilings for the `validate-aspects` gate.
//!
//! Not yet measured: every ceiling is infinite and both floors are 1.

/// Ceilings for one pair, over all of its groups and angles.
#[allow(dead_code)] // wired in the next commit (#84)
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    /// The time between the engine's event and the reference's, multiplied
    /// by the pair's relative longitude speed at the event: the time
    /// residual expressed as an angle.
    pub(crate) sep_arcsec: f64,
    /// The longitude difference of either body at the event.
    pub(crate) lon_arcsec: f64,
}

/// Fail-closed floor on compared events for the full gate.
#[allow(dead_code)] // wired in the next commit (#84)
pub(crate) const MIN_ROWS_VALIDATED: usize = 1;

/// Fail-closed floor for the release-battery subset (the `mean` group).
#[allow(dead_code)] // wired in the next commit (#84)
pub(crate) const MIN_ROWS_VALIDATED_MEAN_SUBSET: usize = 1;

/// Ceilings by corpus pair name (`"Mars-Saturn"`), or `None` for a pair the
/// gate does not cover.
#[allow(dead_code)] // wired in the next commit (#84)
pub(crate) fn ceilings_for(pair: &str) -> Option<Ceilings> {
    let unmeasured = Ceilings {
        sep_arcsec: f64::INFINITY,
        lon_arcsec: f64::INFINITY,
    };
    match pair {
        "Sun-Moon" | "Sun-Mercury" | "Mercury-Venus" | "Venus-Mars" | "Mars-Jupiter"
        | "Mars-Saturn" | "Jupiter-Saturn" | "Saturn-Pluto" => Some(unmeasured),
        _ => None,
    }
}
