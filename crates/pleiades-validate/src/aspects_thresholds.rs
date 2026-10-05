//! Measured-basis ceilings for the `validate-aspects` gate.
//!
//! Measured on 2026-10-02 by running the gate with infinite ceilings against
//! the committed corpus; each ceiling is the largest value over the pair's
//! corpus groups and angles times 1.5, rounded up to two significant
//! figures. The measured value and its group are recorded beside each
//! ceiling.
//!
//! What the residuals are. The corpus holds the instants at which the
//! difference of two Swiss Ephemeris (Moshier) longitudes equals an angle;
//! the engine finds the same instants in the packaged (DE440-derived)
//! backend's longitudes. A difference between the two separations moves the
//! instant by that difference divided by the pair's relative longitude
//! speed, which falls to zero where the separation turns. A time ceiling
//! would therefore mean nothing for a slow pair near a station, so the gate
//! bounds the time residual multiplied by the corpus's relative speed at
//! the event: the separation residual, in arcseconds. The largest time
//! residual per pair is reported by the gate, with no ceiling of its own.
//!
//! The longitude ceiling bounds the difference of either body's longitude
//! at the event. It includes the body's motion over the time residual.

/// Ceilings for one pair, over all of its groups and angles.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    /// The time between the engine's event and the reference's, multiplied
    /// by the pair's relative longitude speed at the event: the time
    /// residual expressed as an angle.
    pub(crate) sep_arcsec: f64,
    /// The longitude difference of either body at the event.
    pub(crate) lon_arcsec: f64,
}

/// Fail-closed floor on compared events: the count the gate compared on
/// 2026-10-02 (10359).
pub(crate) const MIN_ROWS_VALIDATED: usize = 10_359;

/// Fail-closed floor for the release-battery subset (the `mean` group, see
/// `validate_aspects_corpus_subset`): the count that subset compared on
/// 2026-10-02 (372).
pub(crate) const MIN_ROWS_VALIDATED_MEAN_SUBSET: usize = 372;

/// Ceilings by corpus pair name (`"Mars-Saturn"`), or `None` for a pair the
/// gate does not cover.
pub(crate) fn ceilings_for(pair: &str) -> Option<Ceilings> {
    match pair {
        // measured max 2.894" (geo), 0.394" (geo)
        "Sun-Moon" => Some(Ceilings {
            sep_arcsec: 4.4,
            lon_arcsec: 0.6,
        }),
        // measured max 0.146" (geo), 0.400" (geo)
        "Sun-Mercury" => Some(Ceilings {
            sep_arcsec: 0.22,
            lon_arcsec: 0.6,
        }),
        // measured max 0.473" (geo; mean 0.384"), 2.324" (geo; mean 2.319")
        "Mercury-Venus" => Some(Ceilings {
            sep_arcsec: 0.71,
            lon_arcsec: 3.5,
        }),
        // measured max 0.958" (geo), 1.489" (geo)
        "Venus-Mars" => Some(Ceilings {
            sep_arcsec: 1.5,
            lon_arcsec: 2.3,
        }),
        // measured max 2.211" (geo; helio 0.530"), 0.801" (geo; helio 0.747")
        "Mars-Jupiter" => Some(Ceilings {
            sep_arcsec: 3.4,
            lon_arcsec: 1.3,
        }),
        // measured max 1.114" (geo; mean 0.825"), 1.075" (geo; mean 0.639")
        "Mars-Saturn" => Some(Ceilings {
            sep_arcsec: 1.7,
            lon_arcsec: 1.7,
        }),
        // measured max 1.119" (geo), 2.231" (geo)
        "Jupiter-Saturn" => Some(Ceilings {
            sep_arcsec: 1.7,
            lon_arcsec: 3.4,
        }),
        // measured max 1.565" (geo), 1.662" (geo)
        "Saturn-Pluto" => Some(Ceilings {
            sep_arcsec: 2.4,
            lon_arcsec: 2.5,
        }),
        _ => None,
    }
}
