//! Measured-basis ceilings for the `validate-stations` gate.
//!
//! Measured on 2026-10-02 by running the gate with infinite ceilings against
//! the committed corpus; each ceiling is the largest value over the body's
//! corpus series (`geo`, and `mean` and `sid` where present) times 1.5,
//! rounded up to two significant figures. The measured value and its series
//! are recorded beside each ceiling.
//!
//! What the residual is. The corpus holds the zeros of Swiss Ephemeris'
//! (Moshier) longitude speed; the engine finds the zeros of the packaged
//! (DE440-derived) backend's. Near a station the longitude is a parabola in
//! time, so a small difference between the two speeds moves the zero by that
//! difference divided by the body's longitude acceleration at the station.
//! That is why slow bodies, whose acceleration is tiny, have large time
//! ceilings (Pluto about half an hour) while the longitude residual, which
//! is the position difference at the station, stays within a few arcseconds.
//!
//! The planets' mean signed time (engine minus corpus) is clearest for
//! Mercury: -4.2 s, with 99.5 % of its stations negative and both station
//! kinds alike. For the other planets it is consistent with the data but
//! buried in scatter (Venus -1.3 s, Saturn -2.8 s, Neptune +11.0 s, Pluto
//! -56.6 s). It is a reference convention, not an engine defect: Swiss Ephemeris' Moshier planet speed is a backward
//! difference over `PLAN_SPEED_INTV` = 0.0001 d (8.64 s) (`swemplan.c`,
//! `dx[i] = (xp[i] - x2[i]) / dt` with `x2` at `tjd - dt`), so its reported
//! speed is the speed 4.32 s earlier and its zero lands 4.32 s late. The
//! shift is included in the measured maxima; the gate does not compensate
//! for it.
//!
//! True node. Its speed hovers near zero for days, so a station instant is
//! ill-conditioned, and whether a graze crosses zero depends on the
//! ephemeris. The true-node ceilings (about three days in time, tens of
//! arcseconds in longitude) therefore make its comparison a coarse
//! existence-and-kind check, a station of the same kind within about three
//! days, and not a timing check. See `SEPARATION_DAYS` and the true-node arm
//! of `ceilings_for`.

/// Ceilings for one body: time between the engine's station and the
/// reference's, and the longitude difference at the station.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    pub(crate) time_s: f64,
    pub(crate) lon_arcsec: f64,
}

/// A true-node station is "separated" when its nearest neighbouring station
/// in its own list is at least this many days away. Closer pairs are grazes
/// of the speed against zero whose existence depends on the ephemeris, and
/// are not compared.
///
/// 2 days was the design value. The measurement on 2026-10-02 found one-sided
/// graze pairs up to about 2.5 days wide (corpus jd 2461966.56 R /
/// 2461969.08 D and 2455381.07 R / 2455383.58 D have no engine station within
/// 6 days), and 3.0 is the smallest value swept (2.0, 2.5, 3.0, 3.5, 4.0,
/// 5.0) at which every separated station has a same-kind counterpart.
pub(crate) const SEPARATION_DAYS: f64 = 3.0;

/// Fail-closed floor on compared stations: the count the gate compared on
/// 2026-10-02 (5542).
pub(crate) const MIN_ROWS_VALIDATED: usize = 5542;

/// Ceilings by corpus body name, or `None` for a body the gate does not cover.
pub(crate) fn ceilings_for(body_name: &str) -> Option<Ceilings> {
    match body_name {
        // measured max 9.3 s (geo), 0.381" (geo)
        "Mercury" => Some(Ceilings {
            time_s: 14.0,
            lon_arcsec: 0.58,
        }),
        // measured max 21.5 s (geo), 0.542" (geo)
        "Venus" => Some(Ceilings {
            time_s: 33.0,
            lon_arcsec: 0.82,
        }),
        // measured max 54.9 s (geo), 1.119" (geo)
        "Mars" => Some(Ceilings {
            time_s: 83.0,
            lon_arcsec: 1.7,
        }),
        // measured max 101.0 s (geo), 0.695" (geo)
        "Jupiter" => Some(Ceilings {
            time_s: 160.0,
            lon_arcsec: 1.1,
        }),
        // measured max 165.1 s (geo), 0.679" (sid)
        "Saturn" => Some(Ceilings {
            time_s: 250.0,
            lon_arcsec: 1.1,
        }),
        // measured max 318.7 s (geo), 0.505" (geo)
        "Uranus" => Some(Ceilings {
            time_s: 480.0,
            lon_arcsec: 0.76,
        }),
        // measured max 495.4 s (geo), 2.314" (geo)
        "Neptune" => Some(Ceilings {
            time_s: 750.0,
            lon_arcsec: 3.5,
        }),
        // measured max 1292.7 s (geo), 1.255" (geo)
        "Pluto" => Some(Ceilings {
            time_s: 2000.0,
            lon_arcsec: 1.9,
        }),
        // Coarse existence-and-kind check, not a timing check (see the module
        // comment). Measured at SEPARATION_DAYS = 3.0. The time ceiling is the
        // existence window on both sides, so it is taken from the larger of
        // two measurements: the corpus-side max residual 117467.6 s (1.36 d,
        // geo) and the engine-side largest distance to a same-kind corpus
        // station 2.048 d = 176947 s (geo, jd 2453126.80); 176947 s x 1.5 =
        // 265420 s, rounded up to 270000 s. Longitude: max 51.338" (geo),
        // x 1.5 -> 78". Of the 1166 compared corpus stations, 724 are within
        // 0.05 d of their counterpart, 1130 within 0.5 d, 1158 within 1 d, all
        // within 2 d.
        "TrueNode" => Some(Ceilings {
            time_s: 270_000.0,
            lon_arcsec: 78.0,
        }),
        _ => None,
    }
}
