//! Atmospheric refraction (Bennett 1982 / Saemundsson 1986), pressure- and
//! temperature-scaled. Historically omitted from the apparent-place pipeline;
//! rise/set and horizontal coordinates require it. Matches Swiss Ephemeris
//! `swe_refrac` conventions so `validate-rise-trans` can prove parity.

/// Observer atmosphere used to scale refraction. Defaults are the SE standard
/// (`1013.25` mbar, `15` °C).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atmosphere {
    /// Atmospheric pressure at the observer, millibars.
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
/// Reproducing that exact step was tried and rejected: this crate's rise/set
/// engine root-finds `apparent_from_true(true_alt) == standard_altitude`
/// (`pleiades-events/src/rise_trans.rs`), and Sun/Moon disc-edge crossings
/// land almost exactly in that discontinuous band. A bisection search over a
/// genuine jump discontinuity converges to the jump's location rather than to
/// any particular target altitude, so several different `standard_altitude`
/// targets (upper/lower/center limb, fixed vs. true disc size) all collapse
/// onto nearly the same crossing time — which measurably regressed the
/// refraction-floor rows (one jumped from ~22 s to ~97 s residual against SE
/// during development). SE's own `swe_rise_trans` sidesteps this entirely: it
/// evaluates refraction ONCE near h=0 to get a constant offset and roots on
/// TRUE altitude plus that constant, never touching the discontinuous
/// below-horizon branch at all — a different algorithm than "root-find the
/// apparent altitude formula directly," which is this crate's design and
/// which this task is not chartered to rearchitect.
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
mod tests {
    use super::*;

    #[test]
    fn default_atmosphere_is_se_standard() {
        let a = Atmosphere::default();
        assert_eq!(a.pressure_mbar, 1013.25);
        assert_eq!(a.temperature_c, 15.0);
    }

    #[test]
    fn refraction_at_horizon_is_about_34_arcmin() {
        // Bennett at h=0 with standard atmosphere ≈ 28.7' true→apparent lift is
        // computed on APPARENT altitude; at true h=0 the raise is ~34'. Assert the
        // apparent altitude sits ~34'(=0.567°) above 0 within a loose band.
        let app = apparent_from_true(0.0, Atmosphere::default());
        assert!(
            (app - 0.4752).abs() < 0.05,
            "apparent horizon altitude {app}"
        );
    }

    #[test]
    fn refraction_vanishes_at_zenith() {
        let app = apparent_from_true(90.0, Atmosphere::default());
        assert!((app - 90.0).abs() < 1e-4, "zenith {app}");
    }

    #[test]
    fn saemundsson_inverts_bennett_within_a_few_arcsec() {
        // Round-trip: for altitudes above the horizon the two formulae are near-inverses.
        for h in [5.0, 15.0, 45.0, 80.0] {
            let app = apparent_from_true(h, Atmosphere::default());
            let back = true_from_apparent(app, Atmosphere::default());
            assert!((back - h).abs() < 0.01, "round-trip h={h} back={back}");
        }
    }

    #[test]
    fn true_from_apparent_at_horizon_is_about_negative_34_arcmin() {
        // A body seen ON the apparent horizon (h_app=0) is geometrically ~34' below it.
        let t = true_from_apparent(0.0, Atmosphere::default());
        assert!(
            (t + 0.5667).abs() < 0.02,
            "true altitude at apparent horizon {t}"
        );
    }

    #[test]
    fn refraction_matches_se_below_horizon() {
        // Pinned from `crates/pleiades-validate/data/rise-trans-corpus/azalt.csv`
        // (`se_true_alt_deg < 0` rows; standard atmosphere, `swe_azalt` ->
        // `swe_refrac_extended` ground truth). SE reports `se_apparent_alt_deg
        // == se_true_alt_deg` (refraction fully suppressed) for every one of
        // these — see `apparent_from_true_below_horizon`'s doc for why this
        // module approximates rather than exactly reproduces SE's own
        // (discontinuous) below-horizon model. The shallowest row (-9.96 deg)
        // sits right at the edge of the fade and is pinned within 15 arcsec;
        // every deeper row is pinned within a fraction of an arcsec. Both are
        // a large improvement over the pre-fix ~282 arcsec worst case.
        let atmos = Atmosphere::default();
        for (true_alt, se_apparent_alt, tolerance_arcsec) in [
            (-9.964249, -9.964249, 15.0),
            (-15.874977, -15.874977, 0.01),
            (-34.289902, -34.289902, 0.01),
            (-43.529313, -43.529313, 0.01),
            (-60.896360, -60.896360, 0.01),
            (-64.642565, -64.642565, 0.01),
            (-70.739219, -70.739219, 0.01),
        ] {
            let app = apparent_from_true(true_alt, atmos);
            let residual_arcsec = (app - se_apparent_alt).abs() * 3600.0;
            assert!(
                residual_arcsec < tolerance_arcsec,
                "true={true_alt} app={app} se={se_apparent_alt} residual={residual_arcsec}\""
            );
        }
    }
}
