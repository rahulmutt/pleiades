//! Typed provenance describing which apparent-place corrections were applied
//! and how large they were.

use core::fmt;

/// Which corrections were applied to produce an apparent position.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CorrectionSet {
    /// Light-time (planetary aberration) was iterated.
    pub light_time: bool,
    /// Precession from J2000 to the equinox of date was applied.
    pub precession: bool,
    /// Annual aberration was applied.
    pub annual_aberration: bool,
    /// Nutation in longitude (Δψ) was applied.
    pub nutation_longitude: bool,
    /// Diurnal (geocentric) parallax was applied (topocentric place).
    pub diurnal_parallax: bool,
    /// Diurnal aberration was applied (topocentric place).
    pub diurnal_aberration: bool,
}

/// Data/model sources behind the apparent-place corrections.
pub const MODEL_SOURCES: &str =
    "precession (IAU-1976, Meeus 20.3/21.4); nutation-iau1980.csv (IAU-1980 truncated, Meeus Table 22.A); annual aberration (Meeus 23.2); light-time iteration; light-deflection omitted; diurnal parallax (Meeus 11/40, WGS84 ellipsoid); diurnal aberration (0.319\"·ρcosφ′); atmospheric refraction omitted";

/// Provenance describing how an apparent position was produced.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApparentProvenance {
    /// Light-time retardation applied, in days (0.0 when no light-time iteration
    /// was performed, e.g. the Sun and lunar-apsis paths).
    pub light_time_days: f64,
    /// Light-time iterations taken (0 when no iteration was performed).
    pub iterations: u8,
    /// Longitude shift from J2000->of-date precession, arcseconds (wrapped to
    /// (-180deg, 180deg] before scaling).
    pub precession_longitude_arcsec: f64,
    /// Nutation-in-longitude (Delta-psi) applied to longitude, arcseconds.
    pub nutation_longitude_arcsec: f64,
    /// Annual-aberration shift applied to longitude, arcseconds (0.0 on the
    /// aberration-free lunar-apsis path).
    pub aberration_longitude_arcsec: f64,
    /// Flags recording which corrections were applied.
    pub corrections: CorrectionSet,
    /// Human-readable list of the data/model sources behind the corrections
    /// (see [`MODEL_SOURCES`]).
    pub model_sources: &'static str,
}

impl ApparentProvenance {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        format!(
            "apparent-place light_time={:.6}d iters={} precession_lon={:.3}\" nutation_lon={:.3}\" aberration_lon={:.3}\"",
            self.light_time_days,
            self.iterations,
            self.precession_longitude_arcsec,
            self.nutation_longitude_arcsec,
            self.aberration_longitude_arcsec,
        )
    }
}

impl fmt::Display for ApparentProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Provenance for the topocentric (observer-centric) correction stage.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopocentricProvenance {
    /// Parallax shift applied to ecliptic longitude, arcseconds.
    pub parallax_longitude_arcsec: f64,
    /// Parallax shift applied to ecliptic latitude, arcseconds.
    pub parallax_latitude_arcsec: f64,
    /// Diurnal aberration magnitude applied, arcseconds.
    pub diurnal_aberration_arcsec: f64,
    /// Geocentric distance used for the parallax, AU.
    pub distance_au_used: f64,
}

impl TopocentricProvenance {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        format!(
            "topocentric parallax_lon={:.3}\" parallax_lat={:.3}\" diurnal_aberration={:.4}\" distance_au={:.6}",
            self.parallax_longitude_arcsec,
            self.parallax_latitude_arcsec,
            self.diurnal_aberration_arcsec,
            self.distance_au_used,
        )
    }
}

impl fmt::Display for TopocentricProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_line_is_nonempty_and_matches_display() {
        let p = ApparentProvenance {
            light_time_days: 0.028,
            iterations: 2,
            precession_longitude_arcsec: 1234.5,
            nutation_longitude_arcsec: -3.788,
            aberration_longitude_arcsec: -9.5,
            corrections: CorrectionSet {
                light_time: true,
                precession: true,
                annual_aberration: true,
                nutation_longitude: true,
                diurnal_parallax: false,
                diurnal_aberration: false,
            },
            model_sources: MODEL_SOURCES,
        };
        assert!(!p.summary_line().is_empty());
        assert_eq!(p.to_string(), p.summary_line());
        assert!(p.summary_line().contains("precession_lon"));
        assert!(p.summary_line().contains("nutation_lon"));
    }
}
