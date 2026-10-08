//! Atmospheric refraction, pressure- and temperature-scaled. Historically
//! omitted from the apparent-place pipeline; rise/set and horizontal
//! coordinates require it.
//!
//! Two models, each matching the Swiss Ephemeris function it stands in for:
//!
//! - [`apparent_from_true`] / [`true_from_apparent`] (Bennett 1982 /
//!   Saemundsson 1986, `swe_refrac` conventions) convert an altitude, for
//!   horizontal coordinates (`swe_azalt`).
//! - [`horizon_refraction_deg`] (Sinclair's formula with SE's factor,
//!   `swe_refrac_extended`) is the constant refraction rise/set roots the
//!   true altitude against, as `swe_rise_trans` does (issue #242).

/// Observer atmosphere used to scale refraction, with the meaning Swiss
/// Ephemeris gives its `atpress`/`attemp` arguments: an `Atmosphere` built
/// from the arguments of an SE call gives that call's refraction.
///
/// The default is a standard sea-level atmosphere at 15 °C (`1013.25` mbar,
/// `15` °C). That is **not** what SE's own default `swe_rise_trans` call
/// uses: SE code that passes `atpress = 0, attemp = 0` is
/// [`Atmosphere::SE_DEFAULT_CALL`]. At the horizon the two differ by 189″ of
/// refraction; on 1 July 2026 the default's sunrise is 14 s later at the
/// equator, 27 s at 51° N and 44 s at 60° N.
///
/// `pressure_mbar == 0.0` is not a vacuum: as in SE, it asks for the
/// pressure to be estimated from the observer's elevation (see
/// [`Atmosphere::at_elevation`]). To leave refraction out, turn it off where
/// the API offers that (for rise/set, `RiseSetOptions::refraction`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atmosphere {
    /// Atmospheric pressure at the observer, millibars; `0.0` means
    /// "estimate from the observer's elevation".
    pub pressure_mbar: f64,
    /// Atmospheric temperature at the observer, degrees Celsius.
    pub temperature_c: f64,
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            pressure_mbar: 1013.25,
            temperature_c: 15.0,
        }
    }
}

impl Atmosphere {
    /// The atmosphere of Swiss Ephemeris' default call, `atpress = 0,
    /// attemp = 0`: the pressure is estimated from the elevation (1013.25
    /// mbar at sea level) and the temperature is 0 °C.
    pub const SE_DEFAULT_CALL: Atmosphere = Atmosphere {
        pressure_mbar: 0.0,
        temperature_c: 0.0,
    };

    /// This atmosphere with a zero pressure replaced by SE's estimate for
    /// an observer `elevation_m` metres above sea level,
    /// `1013.25 (1 - 0.0065 h / 288)^5.255` mbar (`swe_azalt`,
    /// `swe_rise_trans`). Any other pressure is returned unchanged.
    pub fn at_elevation(self, elevation_m: f64) -> Atmosphere {
        if self.pressure_mbar != 0.0 {
            return self;
        }
        Atmosphere {
            pressure_mbar: 1013.25 * (1.0 - 0.0065 * elevation_m / 288.0).powf(5.255),
            ..self
        }
    }
}

/// SE's temperature lapse rate for refraction, K/m (`SE_LAPSE_RATE`).
const SE_LAPSE_RATE: f64 = 0.0065;

/// SE's Earth radius for the horizon dip, metres (`EARTH_RADIUS`).
const SE_EARTH_RADIUS_M: f64 = 6_378_136.6;

/// Astronomical refraction at an apparent altitude, degrees: Sinclair's
/// formula with SE's pressure/temperature factor (`swecl.c`
/// `calc_astronomical_refr`).
fn sinclair_refraction_deg(apparent_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = apparent_alt_deg;
    let r = if h > 17.904104638432 {
        0.97 / h.to_radians().tan()
    } else {
        (34.46 + 4.23 * h + 0.004 * h * h) / (1.0 + 0.505 * h + 0.0845 * h * h)
    };
    (atmos.pressure_mbar - 80.0)
        / 930.0
        / (1.0 + 0.00008 * (r + 39.0) * (atmos.temperature_c - 10.0))
        * r
        / 60.0
}

/// Dip of the sea horizon for an observer `elevation_m` above sea level,
/// degrees, zero or negative (`swecl.c` `calc_dip`, after Thom 1973).
fn horizon_dip_deg(elevation_m: f64, atmos: Atmosphere) -> f64 {
    let krefr = (0.0342 + SE_LAPSE_RATE) / (0.154 * 0.0238);
    let t = 273.15 + atmos.temperature_c;
    let d = 1.0 - 1.8480 * krefr * atmos.pressure_mbar / t / t;
    -(1.0 / (1.0 + elevation_m / SE_EARTH_RADIUS_M))
        .acos()
        .to_degrees()
        * d.sqrt()
}

/// The refraction rise/set uses, degrees: SE's apparent→true refraction at
/// the apparent altitude the event is defined at (`swe_refrac_extended`,
/// `SE_APP_TO_TRUE`). A body is on that horizon when its true altitude is
/// `apparent_horizon_deg` minus this value.
///
/// This is how Swiss Ephemeris' `swe_rise_trans` treats refraction: it
/// evaluates it once, at the horizon, and roots the true altitude against
/// that constant. Evaluating a true→apparent formula on the moving true
/// altitude (as [`apparent_from_true`] does) gives a different horizon
/// refraction, 158″ more at 15 °C, and a different temperature dependence.
///
/// An apparent horizon below the dip of the sea horizon cannot be seen
/// from `elevation_m`, and gets no refraction, as in SE. `atmos` is used
/// as given: resolve a zero pressure with [`Atmosphere::at_elevation`]
/// first.
pub fn horizon_refraction_deg(
    apparent_horizon_deg: f64,
    elevation_m: f64,
    atmos: Atmosphere,
) -> f64 {
    if apparent_horizon_deg < horizon_dip_deg(elevation_m, atmos) {
        return 0.0;
    }
    sinclair_refraction_deg(apparent_horizon_deg, atmos)
}

fn scale(atmos: Atmosphere) -> f64 {
    (atmos.pressure_mbar / 1010.0) * (283.0 / (273.0 + atmos.temperature_c))
}

/// Below this true/apparent altitude, Bennett/Saemundsson are still
/// well-behaved (their `tan` singularities sit at h=-5.11 deg / h=-4.4 deg
/// respectively, safely past this point) — so the `h >= 0` formula is used
/// unmodified all the way down to here. This exactly reproduces the
/// pre-Task-17 behavior for every altitude a rise/set crossing or azalt call
/// actually reaches near the horizon (verified: the committed corpus's
/// refraction-floor rows never cross below -1 deg), so nothing in that
/// regime regresses.
const BELOW_HORIZON_BLEND_START_DEG: f64 = -1.0;

/// SE's own `swe_refrac_extended` treats altitudes below -10 deg as having no
/// meaningful refraction at all (`swecl.c`'s `SE_TRUE_TO_APP` branch:
/// `if (inalt < -10) return inalt;`) — every committed corpus row at or below
/// this line reports `se_apparent_alt_deg == se_true_alt_deg` exactly. This
/// module holds that same identity below this altitude.
const BELOW_HORIZON_BLEND_END_DEG: f64 = -10.0;

fn bennett_refraction_arcmin(h: f64, atmos: Atmosphere) -> f64 {
    scale(atmos) * 1.02 / ((h + 10.3 / (h + 5.11)).to_radians().tan())
}

fn saemundsson_refraction_arcmin(h: f64, atmos: Atmosphere) -> f64 {
    scale(atmos) * 1.0 / ((h + 7.31 / (h + 4.4)).to_radians().tan())
}

/// Below-horizon (`true_alt_deg < 0`) branch of `apparent_from_true`.
///
/// Reading the committed `azalt.csv` corpus's `se_true_alt_deg < 0` rows
/// shows SE reports `se_apparent_alt_deg == se_true_alt_deg` (refraction
/// entirely suppressed) for every one of them — the shallowest is -9.96 deg.
/// The vendored SE source confirms why: `swe_azalt` computes refraction via
/// `swe_refrac_extended`, which (a) returns the input unchanged outright below
/// -10 deg, and (b) even above -10 deg, discards the computed refraction
/// (falls back to identity) whenever the resulting apparent altitude would
/// still be below the horizon dip — SE's below-horizon refraction model is a
/// genuinely discontinuous step between "full refraction" and "none",
/// switching abruptly right around h=-0.5 deg for a standard atmosphere.
///
/// Reproducing that exact step was tried and rejected: the rise/set engine
/// then root-found `apparent_from_true(true_alt) == standard_altitude`, and
/// Sun/Moon disc-edge crossings land almost exactly in that discontinuous
/// band. A bisection search over a genuine jump discontinuity converges to
/// the jump's location rather than to any particular target altitude, so
/// several different `standard_altitude` targets (upper/lower/center limb,
/// fixed vs. true disc size) all collapsed onto nearly the same crossing
/// time — which measurably regressed rise/set rows (one jumped from ~22 s to
/// ~97 s residual against SE during development). SE's own `swe_rise_trans`
/// sidesteps this entirely: it evaluates refraction ONCE at the horizon and
/// roots on TRUE altitude against that constant, never touching the
/// discontinuous below-horizon branch at all. Rise/set now does the same
/// (issue #242, [`horizon_refraction_deg`]) and no longer calls this
/// function, which serves horizontal coordinates only.
///
/// So instead: hold Bennett's own refraction value fixed at its (well-behaved,
/// singularity-free-here) `BELOW_HORIZON_BLEND_START_DEG` figure, then fade it
/// linearly to zero by `BELOW_HORIZON_BLEND_END_DEG`, matching SE's
/// documented "held or blended down" description without introducing a jump.
/// This is smooth and monotonic everywhere, leaves `h >= -1 deg` completely
/// unchanged (protecting the refraction-floor rows), and matches the
/// corpus's below-horizon values to within ~9 arcsec at the shallowest tested
/// row (-9.96 deg, right at the edge of the fade) and exactly (0 arcsec) for
/// every deeper row — a large improvement over the prior ~282 arcsec worst
/// case, achieved with a physical clamp/blend rather than a per-row fit.
fn apparent_from_true_below_horizon(true_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = true_alt_deg;
    if h >= BELOW_HORIZON_BLEND_START_DEG {
        return h + bennett_refraction_arcmin(h, atmos) / 60.0;
    }
    if h <= BELOW_HORIZON_BLEND_END_DEG {
        return h;
    }
    let anchor_deg = bennett_refraction_arcmin(BELOW_HORIZON_BLEND_START_DEG, atmos) / 60.0;
    let fade = (h - BELOW_HORIZON_BLEND_END_DEG)
        / (BELOW_HORIZON_BLEND_START_DEG - BELOW_HORIZON_BLEND_END_DEG);
    h + anchor_deg * fade
}

/// Below-horizon (`apparent_alt_deg < 0`) branch of `true_from_apparent`,
/// mirroring `apparent_from_true_below_horizon`'s hold-then-fade shape with
/// Saemundsson in place of Bennett, for the same reasons (Saemundsson's own
/// singularity at h=-4.4 deg sits inside the naive blend range, and
/// continuity with the `h >= -1 deg` branch matters for round-tripping).
fn true_from_apparent_below_horizon(apparent_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = apparent_alt_deg;
    if h >= BELOW_HORIZON_BLEND_START_DEG {
        return h - saemundsson_refraction_arcmin(h, atmos) / 60.0;
    }
    if h <= BELOW_HORIZON_BLEND_END_DEG {
        return h;
    }
    let anchor_deg = saemundsson_refraction_arcmin(BELOW_HORIZON_BLEND_START_DEG, atmos) / 60.0;
    let fade = (h - BELOW_HORIZON_BLEND_END_DEG)
        / (BELOW_HORIZON_BLEND_START_DEG - BELOW_HORIZON_BLEND_END_DEG);
    h - anchor_deg * fade
}

/// True (geometric) altitude → apparent altitude, degrees. At/above the
/// horizon (`h >= 0`): Bennett (1982), `R = 1.02 / tan(h + 10.3/(h + 5.11))`
/// arcmin, evaluated on the true altitude, pressure/temperature scaled;
/// `apparent = true + R`. Below the horizon (`h < 0`): see
/// `apparent_from_true_below_horizon`'s doc for SE's below-horizon behavior
/// and why this crate approximates rather than exactly reproduces it.
pub fn apparent_from_true(true_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = true_alt_deg;
    if h < 0.0 {
        return apparent_from_true_below_horizon(h, atmos);
    }
    true_alt_deg + bennett_refraction_arcmin(h, atmos) / 60.0
}

/// Apparent altitude → true (geometric) altitude, degrees. At/above the
/// horizon (`h >= 0`): Saemundsson (1986), `R = 1.0 / tan(h + 7.31/(h + 4.4))`
/// arcmin, evaluated on the apparent altitude, pressure/temperature scaled;
/// `true = apparent - R`. Below the horizon (`h < 0`): see
/// `true_from_apparent_below_horizon`'s doc.
pub fn true_from_apparent(apparent_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = apparent_alt_deg;
    if h < 0.0 {
        return true_from_apparent_below_horizon(h, atmos);
    }
    apparent_alt_deg - saemundsson_refraction_arcmin(h, atmos) / 60.0
}

#[cfg(test)]
mod tests;
