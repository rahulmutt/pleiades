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

#[test]
fn topocentric_provenance_summary_line_and_display_are_stable() {
    let p = TopocentricProvenance {
        parallax_longitude_arcsec: 12.5,
        parallax_latitude_arcsec: -3.25,
        diurnal_aberration_arcsec: 0.3192,
        distance_au_used: 1.5,
    };
    assert_eq!(
        p.summary_line(),
        "topocentric parallax_lon=12.500\" parallax_lat=-3.250\" diurnal_aberration=0.3192\" distance_au=1.500000"
    );
    assert_eq!(p.to_string(), p.summary_line());
}
