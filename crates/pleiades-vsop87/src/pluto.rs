//! Which source serves Pluto at a given instant (issue #129).
//!
//! The VSOP87 theory excludes Pluto. Inside the validity window of Meeus
//! Table 37.A (1885–2099) the backend evaluates that periodic-term fit;
//! outside it, the mean-element orbit (issue #119). The fit degrades quickly
//! outside its window, and rejecting out-of-window instants would break an
//! algorithmic backend whose nominal range is unbounded, so the fallback
//! stays. At each edge Pluto jumps by up to about 0.6°: a root finder that
//! spans an edge sees the jump. A speed sample takes the path of its centre
//! instant for all of its samples, so speeds stay smooth up to the edge.

/// First instant served by the fit: 1885-01-01 0h TT (inclusive).
pub(crate) const PLUTO_FIT_START_JD: f64 = 2_409_542.5;
/// End of the fit's window: 2100-01-01 0h TT (exclusive).
pub(crate) const PLUTO_FIT_END_JD: f64 = 2_488_069.5;

/// The source that serves Pluto at an instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlutoPath {
    /// Meeus Table 37.A.
    PeriodicTermFit,
    /// The mean-element Keplerian orbit.
    MeanElements,
}

impl PlutoPath {
    /// The path for a TT/TDB Julian day.
    pub(crate) fn for_julian_day(jd_tt: f64) -> Self {
        if (PLUTO_FIT_START_JD..PLUTO_FIT_END_JD).contains(&jd_tt) {
            Self::PeriodicTermFit
        } else {
            Self::MeanElements
        }
    }
}
