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
//! The asteroid pairs (issue #168) were measured on 2026-10-10 against the
//! SWIEPH (seas_18) asteroid corpus; their ceilings use × 1.4 and a 2.3"
//! floor on separation and longitude (from the Swiss Ephemeris vs sb441
//! spike, `docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md`),
//! recorded beside them.
//!
//! The true-node pairs (issue #168) were measured on 2026-10-16 against the
//! SWIEPH (sepl_18/semo_18) corpus: measured × 1.4, floored at 2.3"; the
//! point itself agrees within 0.94" over 1900-2100. The osculating-apogee,
//! mean-node and mean-apogee pairs (issue #168) were measured on 2026-10-17
//! against the same SWIEPH (sepl/semo) corpus, under the same × 1.4 and 2.3"
//! rule. Events inside a graze window are reported, not ceiling-checked.
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

/// Fail-closed floor for the asteroid corpus: every row of `asteroids.csv`
/// (10118, generated 2026-10-10), so a dropped event fails.
pub(crate) const MIN_ROWS_VALIDATED_ASTEROIDS: usize = 10_118;

/// Fail-closed floor for the lunar-point corpus: the rows of
/// `lunar-points.csv` (22600) outside its graze windows, which are compared by
/// count parity and not counted here (22600 less the 4 rows the windows of
/// `lunar-points-grazes.csv` hold).
pub(crate) const MIN_ROWS_VALIDATED_LUNAR_POINTS: usize = 22_596;

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
        // Asteroids (Swiss Ephemeris SWIEPH seas_18 corpus): measured × 1.4,
        // rounded up to two significant figures; neither ceiling is below
        // 2.3" (the 2.2584" Vesta longitude floor of Swiss Ephemeris seas_18
        // vs JPL sb441, rounded up).
        // measured max 1.445", 1.506" (geo; SWIEPH seas_18 corpus, 2026-10-10)
        "Sun-Ceres" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 2.623", 2.196" (geo; SWIEPH seas_18 corpus, 2026-10-10)
        "Sun-Pallas" => Some(Ceilings {
            sep_arcsec: 3.7,
            lon_arcsec: 3.1,
        }),
        // measured max 0.910", 0.969" (geo; SWIEPH seas_18 corpus, 2026-10-10)
        "Sun-Juno" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 2.459", 1.693" (geo; SWIEPH seas_18 corpus, 2026-10-10)
        "Sun-Vesta" => Some(Ceilings {
            sep_arcsec: 3.5,
            lon_arcsec: 2.4,
        }),
        // measured max 0.761", 0.764" (geo 1990–2030; SWIEPH seas_18 corpus,
        // 2026-10-10)
        "Moon-Ceres" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // The two pairs whose separation turns (issue #253). Near a turn the
        // relative speed falls toward zero, so the time residual grows (up to
        // 315 s and 980 s) and with it each body's motion over it, which the
        // longitude residual includes; the separation residual stays small.
        // measured max 2.284", 6.422" (geo; SWIEPH seas_18 corpus, 2026-10-10)
        "Mars-Vesta" => Some(Ceilings {
            sep_arcsec: 3.2,
            lon_arcsec: 9.0,
        }),
        // measured max 1.351", 9.211" (geo; SWIEPH seas_18 corpus, 2026-10-10)
        "Jupiter-Ceres" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 13.0,
        }),
        // True node (Swiss Ephemeris SWIEPH sepl_18/semo_18 corpus): measured
        // × 1.4, floored at 2.3"; the full gate measured overall maxima of
        // 0.861" separation and 0.953" longitude (2026-10-16; longitude: either
        // body's longitude, the engine's at its instant against the corpus's at
        // the reference instant).
        // The point itself agrees within 0.94" over 1900-2100.
        // measured max 0.530", 0.77" (geo; SWIEPH sepl/semo, spike 2026-10-16)
        "TrueNode-Sun" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.861", 0.81" (geo; SWIEPH sepl/semo, spike 2026-10-16)
        "TrueNode-Mars" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.602", 0.79" (geo; SWIEPH sepl/semo, spike 2026-10-16)
        "TrueNode-Jupiter" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.641", 0.71" (geo; SWIEPH sepl/semo, spike 2026-10-16)
        "TrueNode-Saturn" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.513", 0.95" (geo; SWIEPH sepl/semo, spike 2026-10-16)
        "TrueNode-Uranus" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.719", 0.46" (geo; SWIEPH sepl/semo, spike 2026-10-16)
        "TrueNode-Neptune" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.528", 0.93" (geo; SWIEPH sepl/semo, spike 2026-10-16)
        "TrueNode-Pluto" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // True apogee (osculating; SWIEPH sepl/semo corpus): measured × 1.4 of
        // the outside-window maximum, rounded up to two significant figures,
        // floored at 2.3"; events inside graze windows are reported, not
        // ceiling-checked.
        // measured max 2.324", 2.024" (geo; SWIEPH sepl/semo, 2026-10-17)
        "TrueApogee-Sun" => Some(Ceilings {
            sep_arcsec: 3.3,
            lon_arcsec: 2.9,
        }),
        // measured max 2.998", 1.075" (geo; SWIEPH sepl/semo, 2026-10-17)
        "TrueApogee-Jupiter" => Some(Ceilings {
            sep_arcsec: 4.2,
            lon_arcsec: 2.3,
        }),
        // measured max 3.113", 0.586" (geo; SWIEPH sepl/semo, 2026-10-17)
        "TrueApogee-Saturn" => Some(Ceilings {
            sep_arcsec: 4.4,
            lon_arcsec: 2.3,
        }),
        // measured max 2.483", 0.433" (geo; SWIEPH sepl/semo, 2026-10-17)
        "TrueApogee-Pluto" => Some(Ceilings {
            sep_arcsec: 3.5,
            lon_arcsec: 2.3,
        }),
        // Mean node and mean apogee (SWIEPH sepl/semo corpus): measured × 1.4
        // of the outside-window maximum, rounded up to two significant figures,
        // floored at 2.3"; events inside graze windows are reported, not
        // ceiling-checked.
        // measured max 0.219", 0.141" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanNode-Sun" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.822", 0.198" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanNode-Mars" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.603", 0.242" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanNode-Jupiter" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.218", 0.621" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanNode-Saturn" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.221", 0.633" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanNode-Uranus" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.285", 0.454" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanNode-Neptune" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.242", 0.382" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanNode-Pluto" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.244", 0.590" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanApogee-Sun" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.350", 1.000" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanApogee-Jupiter" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        // measured max 0.248", 2.232" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanApogee-Saturn" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 3.2,
        }),
        // measured max 0.280", 0.346" (geo; SWIEPH sepl/semo, 2026-10-17)
        "MeanApogee-Pluto" => Some(Ceilings {
            sep_arcsec: 2.3,
            lon_arcsec: 2.3,
        }),
        _ => None,
    }
}
