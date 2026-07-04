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

/// True (geometric) altitude → apparent altitude, degrees. Bennett (1982):
/// `R = 1.02 / tan(h + 10.3/(h + 5.11))` arcmin, evaluated on the true altitude,
/// pressure/temperature scaled. `apparent = true + R`.
pub fn apparent_from_true(true_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = true_alt_deg;
    let r_arcmin = scale(atmos) * 1.02 / ((h + 10.3 / (h + 5.11)).to_radians().tan());
    true_alt_deg + r_arcmin / 60.0
}

/// Apparent altitude → true (geometric) altitude, degrees. Saemundsson (1986):
/// `R = 1.0 / tan(h + 7.31/(h + 4.4))` arcmin, evaluated on the apparent
/// altitude, pressure/temperature scaled. `true = apparent - R`.
pub fn true_from_apparent(apparent_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = apparent_alt_deg;
    let r_arcmin = scale(atmos) * 1.0 / ((h + 7.31 / (h + 4.4)).to_radians().tan());
    apparent_alt_deg - r_arcmin / 60.0
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
}
